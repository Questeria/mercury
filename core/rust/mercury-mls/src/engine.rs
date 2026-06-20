//! PERSISTENT MLS group engine: one long-lived MLS identity per Mercury
//! account, whose ENTIRE state -- the account signer plus every group's
//! secrets -- exports to bytes ([`MlsEngine::export_state`]) that the app seals
//! inside its encrypted snapshot, and imports back after a restart
//! ([`MlsEngine::import_state`]).
//!
//! This is the CLASSICAL MLS engine: [`DEFAULT_CIPHERSUITE`] (RFC 9420
//! X25519 / AES-128-GCM / SHA-256 / Ed25519) on OpenMLS's RustCrypto provider,
//! same as the rest of this crate. The PQ X-Wing path stays feature-gated OFF
//! per the Cargo.toml note (open libcrux RUSTSEC advisories); a PQ engine is a
//! later increment, not silently substituted here.
//!
//! Where [`crate::MlsMember`] is an ephemeral per-process member, the engine is
//! the app-facing, restart-surviving account object. Every method takes `&self`
//! and loads the group from the engine's storage per operation
//! ([`MlsGroup::load`]); OpenMLS 0.8 writes every mutation back into the
//! storage provider automatically, so the engine's [`MemoryStorage`] map is at
//! all times the single source of truth that [`MlsEngine::export_state`]
//! snapshots. (Per-group operations are not transactional across threads: the
//! app drives one engine from one logical owner, which is how mercury-app uses
//! it.)
//!
//! SECRETS WARNING: the bytes returned by [`MlsEngine::export_state`] contain
//! LIVE MLS SECRETS -- the account's signature PRIVATE key and every group's
//! epoch / ratchet secrets. They are NOT encrypted here. The caller MUST seal
//! them (Mercury stores them only inside the app's encrypted snapshot) and MUST
//! zeroize any transient plaintext copies it makes. Internally the engine
//! scrubs its own transient copies (zeroize-on-drop) where practical; the
//! returned buffer itself is necessarily the caller's to protect.
//!
//! Fail-closed parsing: every untrusted-bytes entry point (presented
//! KeyPackages, Welcomes, application messages, commits, and imported state)
//! parses under the crate's panic guard ([`guard_hostile_bytes`]) with explicit
//! validation -- malformed input returns `Err`, never a crash, in every build
//! profile.

use core::fmt;
use std::collections::HashMap;
use std::sync::RwLock;

use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::{MemoryStorage, RustCrypto};
use tls_codec::{Deserialize as _, Serialize as _};
use zeroize::Zeroizing;

use crate::{DEFAULT_CIPHERSUITE, MlsError, guard_hostile_bytes};

/// The engine's OpenMLS provider: the exact `OpenMlsRustCrypto` shape
/// (`RustCrypto` as both crypto and rand provider, [`MemoryStorage`] as
/// storage), re-declared here ONLY to add what the stock provider lacks -- a
/// constructor that accepts an existing [`MemoryStorage`], so
/// [`MlsEngine::import_state`] can rebuild the provider around a restored
/// storage map. The storage map is reachable via `MemoryStorage`'s public
/// `values` field, which is what [`MlsEngine::export_state`] snapshots.
struct SealableProvider {
    crypto: RustCrypto,
    key_store: MemoryStorage,
}

impl SealableProvider {
    /// Build a provider around `key_store` (empty for a new engine, restored
    /// for an imported one). The crypto/rand backend is freshly constructed --
    /// it holds no persistent state, only an OS-seeded RNG.
    fn new(key_store: MemoryStorage) -> Self {
        Self {
            crypto: RustCrypto::default(),
            key_store,
        }
    }
}

impl OpenMlsProvider for SealableProvider {
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = MemoryStorage;

    fn storage(&self) -> &Self::StorageProvider {
        &self.key_store
    }

    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }

    fn rand(&self) -> &Self::RandProvider {
        &self.crypto
    }
}

/// The full fan-out of one [`MlsEngine::add_member`] call: the COMMIT to
/// deliver to every EXISTING member (each applies it via
/// [`MlsEngine::apply_commit`] or desyncs), the Welcome to deliver to the newly
/// added member (it joins via [`MlsEngine::join_from_welcome`]), and the added
/// member's verified identity (the BasicCredential bytes from its VALIDATED
/// KeyPackage -- what the roster will show for it). Both wires are canonical
/// MLS TLS bytes the relay transits opaquely.
#[derive(Debug, Clone)]
pub struct AddOutcome {
    /// Serialized commit for the group's existing members.
    pub commit_wire: Vec<u8>,
    /// Serialized Welcome for the newly added member.
    pub welcome_wire: Vec<u8>,
    /// The added member's identity (BasicCredential bytes), taken from the
    /// VALIDATED KeyPackage -- never from unverified wire fields.
    pub added_identity: Vec<u8>,
}

/// Version byte of the engine-state wire format ([`MlsEngine::export_state`]).
const STATE_VERSION: u8 = 0x01;

/// A persistent MLS account engine: one long-lived identity + signature
/// keypair, and an OpenMLS storage holding every group's full state. See the
/// module docs for the persistence model and the SECRETS warning on
/// [`MlsEngine::export_state`].
pub struct MlsEngine {
    /// The account identity, embedded as the BasicCredential in every leaf
    /// this engine occupies. Public, not secret.
    identity: Vec<u8>,
    /// The account's long-lived signature keypair. Also stored inside the
    /// provider's storage (via `SignatureKeyPair::store`) so the two halves of
    /// the exported state can be cross-checked on import.
    signer: SignatureKeyPair,
    /// Crypto backend + the storage map that IS the persistent state.
    provider: SealableProvider,
}

/// Redact the engine state: the storage map and signer hold live MLS secrets,
/// so `Debug` shows only the public identity (mirrors `SignatureKeyPair`'s own
/// redacting `Debug` upstream).
impl fmt::Debug for MlsEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MlsEngine")
            .field("identity", &self.identity)
            .field("state", &"***")
            .finish()
    }
}

impl MlsEngine {
    /// Create a NEW engine for `identity` (the account's public identity
    /// bytes): generates the long-lived signature keypair under
    /// [`DEFAULT_CIPHERSUITE`]'s scheme and stores it in the engine's storage
    /// so it round-trips through [`MlsEngine::export_state`].
    pub fn new(identity: &[u8]) -> Result<Self, MlsError> {
        let provider = SealableProvider::new(MemoryStorage::default());
        let signer = SignatureKeyPair::new(DEFAULT_CIPHERSUITE.signature_algorithm())
            .map_err(|_| MlsError::SignatureKeygen)?;
        signer
            .store(provider.storage())
            .map_err(|_| MlsError::SignatureKeygen)?;
        Ok(Self {
            identity: identity.to_vec(),
            signer,
            provider,
        })
    }

    /// The account identity this engine signs as (BasicCredential bytes).
    pub fn identity(&self) -> &[u8] {
        &self.identity
    }

    /// The engine's credential + public signature key, as embedded in every
    /// leaf node this engine occupies.
    fn credential_with_key(&self) -> CredentialWithKey {
        CredentialWithKey {
            credential: BasicCredential::new(self.identity.clone()).into(),
            signature_key: self.signer.public().into(),
        }
    }

    /// Load `group_id`'s state from the engine's storage. Every group-keyed
    /// operation goes through this: OpenMLS 0.8 persists all mutations back
    /// into storage automatically, so a per-operation load always observes the
    /// latest state. Absent group => [`MlsError::GroupNotFound`].
    fn load_group(&self, group_id: &[u8; 32]) -> Result<MlsGroup, MlsError> {
        MlsGroup::load(self.provider.storage(), &GroupId::from_slice(group_id))
            .map_err(|_| MlsError::GroupNotFound)?
            .ok_or(MlsError::GroupNotFound)
    }

    /// Whether this engine holds persisted state for `group_id` (a fresh
    /// storage load, not a cached flag).
    pub fn has_group(&self, group_id: &[u8; 32]) -> bool {
        self.load_group(group_id).is_ok()
    }

    /// Mint a fresh KeyPackage for this identity and return its canonical TLS
    /// wire bytes -- what the account publishes so others can invite it. The
    /// private init/encryption keys land in the engine's storage (and so
    /// survive [`MlsEngine::export_state`] / import: a KeyPackage minted before
    /// a restart still admits the invite that arrives after it).
    ///
    /// SINGLE-USE is the CALLER's concern: MLS KeyPackages must be consumed at
    /// most once, and OpenMLS does not stop a remote group from re-adding the
    /// same wire bytes. mercury-app mints a fresh KeyPackage per invite and the
    /// crate's consume-store gate ([`crate::InMemoryConsumeStore`] /
    /// `put_mls_key_package_consumption_record`) rejects a reused hash.
    pub fn fresh_key_package(&self) -> Result<Vec<u8>, MlsError> {
        let bundle = KeyPackage::builder()
            .build(
                DEFAULT_CIPHERSUITE,
                &self.provider,
                &self.signer,
                self.credential_with_key(),
            )
            .map_err(|_| MlsError::KeyPackageBuild)?;
        bundle
            .key_package()
            .tls_serialize_detached()
            .map_err(|_| MlsError::WelcomeSerialize)
    }

    /// Create a new group under the 32-byte `group_id` with this engine as
    /// founder -- the same configuration as [`crate::MlsMember::create_group`]
    /// ([`DEFAULT_CIPHERSUITE`] + ratchet tree embedded in Welcomes, so a
    /// joiner needs no out-of-band tree). OpenMLS persists the group into the
    /// engine's storage on creation. Fails closed ([`MlsError::GroupCreate`])
    /// if the id is already occupied: silently re-creating would clobber a
    /// live group's secrets.
    pub fn create_group(&self, group_id: &[u8; 32]) -> Result<(), MlsError> {
        if self.has_group(group_id) {
            return Err(MlsError::GroupCreate);
        }
        let config = MlsGroupCreateConfig::builder()
            .ciphersuite(DEFAULT_CIPHERSUITE)
            // Embed the ratchet tree in the Welcome so a joiner can build the
            // group from the Welcome alone (no out-of-band tree delivery).
            .use_ratchet_tree_extension(true)
            .build();
        MlsGroup::new_with_group_id(
            &self.provider,
            &self.signer,
            &config,
            GroupId::from_slice(group_id),
            self.credential_with_key(),
        )
        .map_err(|_| MlsError::GroupCreate)?;
        Ok(())
    }

    /// Join a group from a relay-delivered Welcome (canonical TLS wire bytes)
    /// and return the joined 32-byte group id. The bytes are UNTRUSTED:
    /// parsing + staging run under the panic guard and fail closed on
    /// malformed / truncated / mutated input. The group id is validated to be
    /// exactly 32 bytes BEFORE the group is persisted -- a Welcome carrying any
    /// other id shape is rejected ([`MlsError::BadDigestLength`]) without
    /// leaving unaddressable state behind.
    pub fn join_from_welcome(&self, welcome_wire: &[u8]) -> Result<[u8; 32], MlsError> {
        guard_hostile_bytes(Err(MlsError::JoinGroup), || {
            let welcome = match MlsMessageIn::tls_deserialize(&mut &welcome_wire[..])
                .map_err(|_| MlsError::MessageDecode)?
                .extract()
            {
                MlsMessageBodyIn::Welcome(welcome) => welcome,
                _ => return Err(MlsError::MessageDecode),
            };
            let staged = StagedWelcome::new_from_welcome(
                &self.provider,
                &MlsGroupJoinConfig::default(),
                welcome,
                None,
            )
            .map_err(|_| MlsError::JoinGroup)?;
            // Fail closed BEFORE persisting: Mercury addresses groups by
            // 32-byte id, so any other shape is unusable by this engine.
            let group_id: [u8; 32] = staged
                .group_context()
                .group_id()
                .as_slice()
                .try_into()
                .map_err(|_| MlsError::BadDigestLength)?;
            staged
                .into_group(&self.provider)
                .map_err(|_| MlsError::JoinGroup)?;
            Ok(group_id)
        })
    }

    /// Add a member to `group_id` from its PRESENTED KeyPackage wire bytes.
    /// The bytes are UNTRUSTED: under the panic guard they are parsed
    /// (`KeyPackageIn::tls_deserialize`) and fully VALIDATED by OpenMLS
    /// (`validate`: leaf + KeyPackage signatures, extensions, lifetime,
    /// init != encryption key, protocol version) -- any failure rejects the
    /// add. The commit is merged locally (this engine advances to the new
    /// epoch); deliver [`AddOutcome::commit_wire`] to every EXISTING member and
    /// [`AddOutcome::welcome_wire`] to the new one.
    pub fn add_member(
        &self,
        group_id: &[u8; 32],
        key_package_wire: &[u8],
    ) -> Result<AddOutcome, MlsError> {
        let mut group = self.load_group(group_id)?;
        let key_package = guard_hostile_bytes(Err(MlsError::AddMember), || {
            let key_package_in = KeyPackageIn::tls_deserialize(&mut &key_package_wire[..])
                .map_err(|_| MlsError::MessageDecode)?;
            key_package_in
                .validate(self.provider.crypto(), ProtocolVersion::Mls10)
                .map_err(|_| MlsError::AddMember)
        })?;
        // The identity we report is read from the VALIDATED package's leaf
        // credential -- fail closed if it is not a BasicCredential.
        let added_identity = credential_identity(key_package.leaf_node().credential().clone())?;

        let (commit, welcome, _group_info) = group
            .add_members(
                &self.provider,
                &self.signer,
                std::slice::from_ref(&key_package),
            )
            .map_err(|_| MlsError::AddMember)?;
        group
            .merge_pending_commit(&self.provider)
            .map_err(|_| MlsError::AddMember)?;

        Ok(AddOutcome {
            commit_wire: commit
                .tls_serialize_detached()
                .map_err(|_| MlsError::WelcomeSerialize)?,
            welcome_wire: welcome
                .tls_serialize_detached()
                .map_err(|_| MlsError::WelcomeSerialize)?,
            added_identity,
        })
    }

    /// Remove EVERY member whose BasicCredential identity equals `member_identity` (the app removes by
    /// IDENTITY -- it never knows signature keys -- and MLS permits two leaves to carry the SAME
    /// identity, e.g. a second device or a relay re-presenting a KeyPackage). Removing only the first
    /// matching leaf would leave the duplicate a full member of the rotated epoch, still able to
    /// decrypt -- silently breaking membership forward secrecy. So all matching leaves are removed in
    /// ONE commit. Merge locally + return the serialized commit for the remaining members.
    /// [`MlsError::MemberNotFound`] if NO leaf carries that identity.
    pub fn remove_member(
        &self,
        group_id: &[u8; 32],
        member_identity: &[u8],
    ) -> Result<Vec<u8>, MlsError> {
        let mut group = self.load_group(group_id)?;
        let indices: Vec<_> = group
            .members()
            .filter(|member| {
                BasicCredential::try_from(member.credential.clone())
                    .is_ok_and(|credential| credential.identity() == member_identity)
            })
            .map(|member| member.index)
            .collect();
        if indices.is_empty() {
            return Err(MlsError::MemberNotFound);
        }
        let (commit, _welcome, _group_info) = group
            .remove_members(&self.provider, &self.signer, &indices)
            .map_err(|_| MlsError::RemoveMember)?;
        group
            .merge_pending_commit(&self.provider)
            .map_err(|_| MlsError::RemoveMember)?;
        commit
            .tls_serialize_detached()
            .map_err(|_| MlsError::WelcomeSerialize)
    }

    /// Encrypt an application `plaintext` for `group_id` and return the
    /// canonical MLS wire bytes -- the opaque frame the relay transits without
    /// ever seeing plaintext.
    pub fn send(&self, group_id: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, MlsError> {
        let mut group = self.load_group(group_id)?;
        group
            .create_message(&self.provider, &self.signer, plaintext)
            .map_err(|_| MlsError::MessageEncrypt)?
            .tls_serialize_detached()
            .map_err(|_| MlsError::WelcomeSerialize)
    }

    /// Decrypt a received application message and return `(sender identity,
    /// plaintext)`. The sender identity is the BasicCredential bytes of the
    /// message's VERIFIED credential (OpenMLS authenticated the sender's
    /// membership + signature before we read it) -- fail closed
    /// ([`MlsError::CredentialUnsupported`]) if it is not a BasicCredential.
    /// APPLICATION MESSAGES ONLY: a commit, proposal, or external-join frame
    /// is rejected (commits go through [`MlsEngine::apply_commit`]). The bytes
    /// are UNTRUSTED and parse under the panic guard: wrong group, tampering,
    /// truncation, or garbage fails closed, never crashes.
    pub fn receive(
        &self,
        group_id: &[u8; 32],
        wire: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), MlsError> {
        let mut group = self.load_group(group_id)?;
        guard_hostile_bytes(Err(MlsError::MessageDecode), || {
            let protocol = MlsMessageIn::tls_deserialize(&mut &wire[..])
                .map_err(|_| MlsError::MessageDecode)?
                .try_into_protocol_message()
                .map_err(|_| MlsError::MessageDecode)?;
            // Refuse non-application frames BEFORE processing: staging a
            // misrouted commit here would mutate group state on the wrong
            // path (commits go through [`MlsEngine::apply_commit`]).
            if protocol.content_type() != ContentType::Application {
                return Err(MlsError::MessageDecrypt);
            }
            let processed = group
                .process_message(&self.provider, protocol)
                .map_err(|_| MlsError::MessageDecrypt)?;
            let sender_identity = credential_identity(processed.credential().clone())?;
            match processed.into_content() {
                ProcessedMessageContent::ApplicationMessage(message) => {
                    Ok((sender_identity, message.into_bytes()))
                }
                _ => Err(MlsError::MessageDecrypt),
            }
        })
    }

    /// Apply a received membership-change COMMIT (produced by another member's
    /// [`MlsEngine::add_member`] / [`MlsEngine::remove_member`]), advancing
    /// this engine's view of the group to the new epoch. ONLY a commit frame
    /// is accepted; anything else (application message, proposal, external
    /// join) is rejected with [`MlsError::CommitProcess`] BEFORE processing --
    /// so a misrouted application message keeps its one-time ratchet secret
    /// and can still be received afterwards. UNTRUSTED bytes; guarded, fails
    /// closed.
    pub fn apply_commit(&self, group_id: &[u8; 32], commit_wire: &[u8]) -> Result<(), MlsError> {
        let mut group = self.load_group(group_id)?;
        guard_hostile_bytes(Err(MlsError::CommitProcess), || {
            let protocol = MlsMessageIn::tls_deserialize(&mut &commit_wire[..])
                .map_err(|_| MlsError::MessageDecode)?
                .try_into_protocol_message()
                .map_err(|_| MlsError::MessageDecode)?;
            // Refuse non-commit frames BEFORE processing: opening a misrouted
            // application message here would CONSUME its one-time ratchet
            // secret (forward secrecy) and destroy a message the caller meant
            // to [`MlsEngine::receive`].
            if protocol.content_type() != ContentType::Commit {
                return Err(MlsError::CommitProcess);
            }
            let processed = group
                .process_message(&self.provider, protocol)
                .map_err(|_| MlsError::CommitProcess)?;
            match processed.into_content() {
                ProcessedMessageContent::StagedCommitMessage(staged_commit) => group
                    .merge_staged_commit(&self.provider, *staged_commit)
                    .map_err(|_| MlsError::CommitProcess),
                _ => Err(MlsError::CommitProcess),
            }
        })
    }

    /// The group's member identities (BasicCredential bytes per leaf), this
    /// engine's own leaf included. Fails closed on a non-basic credential
    /// rather than skipping or mislabeling a member.
    pub fn roster(&self, group_id: &[u8; 32]) -> Result<Vec<Vec<u8>>, MlsError> {
        let group = self.load_group(group_id)?;
        group
            .members()
            .map(|member| credential_identity(member.credential))
            .collect()
    }

    /// Forget `group_id` LOCALLY: delete all of its persisted state from the
    /// engine's storage (public group, epoch/message secrets, config, PSKs,
    /// pending proposals). This does NOT signal the group -- a proper exit is a
    /// remove commit by a remaining member; this is the local cleanup for a
    /// group this account has left or been removed from. The account signature
    /// key is shared across groups and is deliberately NOT removed.
    pub fn leave_local(&self, group_id: &[u8; 32]) -> Result<(), MlsError> {
        let mut group = self.load_group(group_id)?;
        group
            .delete(self.provider.storage())
            .map_err(|_| MlsError::GroupDelete)
    }

    /// Export the engine's ENTIRE persistent state -- identity, signer, and a
    /// snapshot of the full OpenMLS storage map (every group's secrets) -- as
    /// the version-0x01 length-prefixed format below. Deterministic: entries
    /// are sorted by key, so the same state always yields the same bytes.
    ///
    /// ```text
    /// [version: 1 byte = 0x01]
    /// [identity_len: u32 LE][identity]
    /// [signer_json_len: u32 LE][signer serde_json]
    /// [entry_count: u32 LE]
    /// entry_count x ( [k_len: u32 LE][k][v_len: u32 LE][v] )
    /// ```
    ///
    /// THE RETURNED BYTES CONTAIN MLS SECRETS (signature private key, group
    /// epoch + ratchet secrets). The caller MUST seal them -- Mercury stores
    /// them only inside the app's encrypted snapshot -- and zeroize transient
    /// copies. The engine zeroizes its own intermediate copies on drop. After
    /// exporting for a restart-restore, do not keep BOTH the original engine
    /// and the import alive and active: two live copies of one ratchet fork
    /// the hash chain.
    pub fn export_state(&self) -> Result<Vec<u8>, MlsError> {
        // Transient secret copies are zeroized on drop; the returned buffer is
        // the caller's to seal.
        let signer_json =
            Zeroizing::new(serde_json::to_vec(&self.signer).map_err(|_| MlsError::StateExport)?);
        let entries: Vec<(Vec<u8>, Zeroizing<Vec<u8>>)> = {
            let values = self
                .provider
                .key_store
                .values
                .read()
                .map_err(|_| MlsError::StateExport)?;
            let mut entries: Vec<(Vec<u8>, Zeroizing<Vec<u8>>)> = values
                .iter()
                .map(|(key, value)| (key.clone(), Zeroizing::new(value.clone())))
                .collect();
            // Sort for determinism (HashMap iteration order is random).
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            entries
        };

        let mut out = Vec::new();
        out.push(STATE_VERSION);
        push_chunk(&mut out, &self.identity)?;
        push_chunk(&mut out, &signer_json)?;
        let entry_count = u32::try_from(entries.len()).map_err(|_| MlsError::StateExport)?;
        out.extend_from_slice(&entry_count.to_le_bytes());
        for (key, value) in &entries {
            push_chunk(&mut out, key)?;
            push_chunk(&mut out, value)?;
        }
        Ok(out)
    }

    /// Rebuild an engine from [`MlsEngine::export_state`] bytes. FAIL-CLOSED
    /// parsing under the panic guard: every read is length-checked first, the
    /// version byte must be exactly 0x01, the signer JSON must decode to a
    /// signature keypair whose stored entry is present in the restored storage
    /// (the two halves of a genuine export always agree), and trailing garbage
    /// is rejected -- any violation is [`MlsError::StateImport`], never a
    /// panic. The caller should zeroize its plaintext copy of `bytes` after a
    /// successful import.
    pub fn import_state(bytes: &[u8]) -> Result<Self, MlsError> {
        guard_hostile_bytes(Err(MlsError::StateImport), || {
            let mut cursor = 0usize;
            if take(bytes, &mut cursor, 1)? != [STATE_VERSION] {
                return Err(MlsError::StateImport);
            }
            let identity_len = take_u32(bytes, &mut cursor)?;
            let identity = take(bytes, &mut cursor, identity_len)?.to_vec();
            let signer_json_len = take_u32(bytes, &mut cursor)?;
            let signer_json = take(bytes, &mut cursor, signer_json_len)?;
            let signer: SignatureKeyPair =
                serde_json::from_slice(signer_json).map_err(|_| MlsError::StateImport)?;

            let entry_count = take_u32(bytes, &mut cursor)?;
            // No pre-allocation from the untrusted count: a lying header would
            // be an allocation bomb; truncation fails on the reads instead.
            let mut values = HashMap::new();
            for _ in 0..entry_count {
                let key_len = take_u32(bytes, &mut cursor)?;
                let key = take(bytes, &mut cursor, key_len)?.to_vec();
                let value_len = take_u32(bytes, &mut cursor)?;
                let value = take(bytes, &mut cursor, value_len)?.to_vec();
                values.insert(key, value);
            }
            if cursor != bytes.len() {
                return Err(MlsError::StateImport); // trailing garbage
            }

            let provider = SealableProvider::new(MemoryStorage {
                values: RwLock::new(values),
            });
            // Cross-check the two halves: a genuine export always carries the
            // signer's own stored entry inside the storage map. A state file
            // assembled from mismatched halves fails closed here instead of
            // misbehaving later.
            SignatureKeyPair::read(
                provider.storage(),
                signer.public(),
                DEFAULT_CIPHERSUITE.signature_algorithm(),
            )
            .ok_or(MlsError::StateImport)?;

            Ok(Self {
                identity,
                signer,
                provider,
            })
        })
    }
}

/// Extract the BasicCredential identity bytes from a VERIFIED credential --
/// fail closed ([`MlsError::CredentialUnsupported`]) on any other credential
/// type rather than attributing bytes the engine cannot vouch for.
fn credential_identity(credential: Credential) -> Result<Vec<u8>, MlsError> {
    BasicCredential::try_from(credential)
        .map(|basic| basic.identity().to_vec())
        .map_err(|_| MlsError::CredentialUnsupported)
}

/// Append `[len: u32 LE][bytes]` to `out`; rejects chunks beyond u32 range.
fn push_chunk(out: &mut Vec<u8>, chunk: &[u8]) -> Result<(), MlsError> {
    let len = u32::try_from(chunk.len()).map_err(|_| MlsError::StateExport)?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(chunk);
    Ok(())
}

/// Take exactly `len` bytes at `cursor`, length-checked BEFORE the read --
/// truncated input fails closed, no slicing panic possible.
fn take<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8], MlsError> {
    let end = cursor.checked_add(len).ok_or(MlsError::StateImport)?;
    if end > bytes.len() {
        return Err(MlsError::StateImport);
    }
    let chunk = &bytes[*cursor..end];
    *cursor = end;
    Ok(chunk)
}

/// Take a u32 LE length prefix (as usize) at `cursor`, length-checked.
fn take_u32(bytes: &[u8], cursor: &mut usize) -> Result<usize, MlsError> {
    let raw: [u8; 4] = take(bytes, cursor, 4)?
        .try_into()
        .map_err(|_| MlsError::StateImport)?;
    Ok(u32::from_le_bytes(raw) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALICE: &[u8] = b"alice@mercury";
    const BOB: &[u8] = b"bob@mercury";
    const CAROL: &[u8] = b"carol@mercury";

    fn engine(identity: &[u8]) -> MlsEngine {
        MlsEngine::new(identity).unwrap()
    }

    /// Founder + two joiners, fully converged on `group_id`: A creates, adds B
    /// (B joins), adds C (B applies the commit, C joins). Every engine is at
    /// the same epoch when this returns.
    fn three_member_group(group_id: &[u8; 32]) -> (MlsEngine, MlsEngine, MlsEngine) {
        let a = engine(ALICE);
        let b = engine(BOB);
        let c = engine(CAROL);
        a.create_group(group_id).unwrap();
        let add_b = a
            .add_member(group_id, &b.fresh_key_package().unwrap())
            .unwrap();
        assert_eq!(b.join_from_welcome(&add_b.welcome_wire).unwrap(), *group_id);
        let add_c = a
            .add_member(group_id, &c.fresh_key_package().unwrap())
            .unwrap();
        b.apply_commit(group_id, &add_c.commit_wire).unwrap();
        assert_eq!(c.join_from_welcome(&add_c.welcome_wire).unwrap(), *group_id);
        (a, b, c)
    }

    fn sorted(mut roster: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        roster.sort();
        roster
    }

    #[test]
    fn two_engines_full_group_lifecycle() {
        let a = engine(ALICE);
        let b = engine(BOB);
        let group_id = [11u8; 32];

        a.create_group(&group_id).unwrap();
        // has_group is a FRESH storage load, so this proves creation persisted.
        assert!(a.has_group(&group_id));
        assert!(!b.has_group(&group_id));

        // B publishes a KeyPackage; A validates + adds; B joins from the
        // Welcome and recovers the same 32-byte group id.
        let outcome = a
            .add_member(&group_id, &b.fresh_key_package().unwrap())
            .unwrap();
        assert_eq!(outcome.added_identity, BOB);
        assert!(!outcome.commit_wire.is_empty());
        assert_eq!(
            b.join_from_welcome(&outcome.welcome_wire).unwrap(),
            group_id
        );
        assert!(b.has_group(&group_id));

        // A -> B with sender attribution. (A merged the add commit when it
        // added B; B joined at that epoch -- no extra processing needed.)
        let wire = a.send(&group_id, b"hello bob").unwrap();
        let (sender, plaintext) = b.receive(&group_id, &wire).unwrap();
        assert_eq!(sender, ALICE);
        assert_eq!(plaintext, b"hello bob");

        // B -> A likewise.
        let wire = b.send(&group_id, b"hi alice").unwrap();
        let (sender, plaintext) = a.receive(&group_id, &wire).unwrap();
        assert_eq!(sender, BOB);
        assert_eq!(plaintext, b"hi alice");
    }

    #[test]
    fn third_member_and_commit_fanout() {
        let a = engine(ALICE);
        let b = engine(BOB);
        let c = engine(CAROL);
        let group_id = [22u8; 32];

        a.create_group(&group_id).unwrap();
        let add_b = a
            .add_member(&group_id, &b.fresh_key_package().unwrap())
            .unwrap();
        assert_eq!(b.join_from_welcome(&add_b.welcome_wire).unwrap(), group_id);

        // A adds C: A merges immediately, C joins from the Welcome -- but B is
        // still at the pre-add epoch until it applies the commit.
        let add_c = a
            .add_member(&group_id, &c.fresh_key_package().unwrap())
            .unwrap();
        assert_eq!(add_c.added_identity, CAROL);
        assert_eq!(c.join_from_welcome(&add_c.welcome_wire).unwrap(), group_id);

        // B MUST apply the add commit BEFORE it can open C-epoch traffic: the
        // same wire fails closed first, decrypts after.
        let from_a = a.send(&group_id, b"welcome carol").unwrap();
        assert!(b.receive(&group_id, &from_a).is_err());
        b.apply_commit(&group_id, &add_c.commit_wire).unwrap();
        let (sender, plaintext) = b.receive(&group_id, &from_a).unwrap();
        assert_eq!(sender, ALICE);
        assert_eq!(plaintext, b"welcome carol");

        // C opens A's message too, with correct attribution.
        let (sender, plaintext) = c.receive(&group_id, &from_a).unwrap();
        assert_eq!(sender, ALICE);
        assert_eq!(plaintext, b"welcome carol");

        // All three exchange; every receiver attributes the true sender.
        let from_b = b.send(&group_id, b"hi from bob").unwrap();
        assert_eq!(
            a.receive(&group_id, &from_b).unwrap(),
            (BOB.to_vec(), b"hi from bob".to_vec())
        );
        assert_eq!(
            c.receive(&group_id, &from_b).unwrap(),
            (BOB.to_vec(), b"hi from bob".to_vec())
        );
        let from_c = c.send(&group_id, b"hi from carol").unwrap();
        assert_eq!(
            a.receive(&group_id, &from_c).unwrap(),
            (CAROL.to_vec(), b"hi from carol".to_vec())
        );
        assert_eq!(
            b.receive(&group_id, &from_c).unwrap(),
            (CAROL.to_vec(), b"hi from carol".to_vec())
        );
    }

    #[test]
    fn remove_member_rotates_epoch() {
        let group_id = [33u8; 32];
        let (a, b, c) = three_member_group(&group_id);

        // A removes C BY IDENTITY (the app never knows signature keys).
        let commit = a.remove_member(&group_id, CAROL).unwrap();
        b.apply_commit(&group_id, &commit).unwrap();
        // Re-applying the same commit fails closed (epoch already advanced).
        assert!(b.apply_commit(&group_id, &commit).is_err());

        // Post-removal traffic decrypts for the remaining member...
        let wire = a.send(&group_id, b"carol is gone").unwrap();
        assert_eq!(
            b.receive(&group_id, &wire).unwrap(),
            (ALICE.to_vec(), b"carol is gone".to_vec())
        );
        // ...but NOT for the removed one: its stale group view is locked out
        // of the rotated epoch -- forward secrecy on membership change.
        assert!(c.receive(&group_id, &wire).is_err());

        // Both surviving views agree C is out of the roster.
        let expected = sorted(vec![ALICE.to_vec(), BOB.to_vec()]);
        assert_eq!(sorted(a.roster(&group_id).unwrap()), expected);
        assert_eq!(sorted(b.roster(&group_id).unwrap()), expected);

        // Removing an identity that is not in the group fails closed.
        assert_eq!(
            a.remove_member(&group_id, b"mallory@mercury"),
            Err(MlsError::MemberNotFound)
        );

        // The removed member forgets the dead group locally.
        assert!(c.has_group(&group_id));
        c.leave_local(&group_id).unwrap();
        assert!(!c.has_group(&group_id));
        assert_eq!(c.leave_local(&group_id), Err(MlsError::GroupNotFound));
    }

    #[test]
    fn state_survives_export_import() {
        let group_id = [44u8; 32];
        let (a, b, c) = three_member_group(&group_id);

        // Exchange BEFORE the export so in-flight ratchet positions are part
        // of the persisted state, not just a fresh group.
        let wire = a.send(&group_id, b"pre-export from alice").unwrap();
        b.receive(&group_id, &wire).unwrap();
        c.receive(&group_id, &wire).unwrap();
        let wire = b.send(&group_id, b"pre-export from bob").unwrap();
        a.receive(&group_id, &wire).unwrap();
        c.receive(&group_id, &wire).unwrap();

        let roster_before = sorted(a.roster(&group_id).unwrap());
        let exported = a.export_state().unwrap();
        let restored = MlsEngine::import_state(&exported).unwrap();
        // The original engine must not act after the restore takes over: two
        // live copies of one ratchet would fork the hash chain.
        drop(a);

        assert_eq!(restored.identity(), ALICE);
        assert!(restored.has_group(&group_id));
        assert_eq!(sorted(restored.roster(&group_id).unwrap()), roster_before);

        // Continuity INBOUND: a message sent to the group after the restart
        // decrypts on the restored engine.
        let from_b = b.send(&group_id, b"post-import ping").unwrap();
        assert_eq!(
            restored.receive(&group_id, &from_b).unwrap(),
            (BOB.to_vec(), b"post-import ping".to_vec())
        );

        // Continuity OUTBOUND: the restored engine still speaks as alice and
        // both other members decrypt + attribute it.
        let from_restored = restored.send(&group_id, b"post-import pong").unwrap();
        assert_eq!(
            b.receive(&group_id, &from_restored).unwrap(),
            (ALICE.to_vec(), b"post-import pong".to_vec())
        );
        assert_eq!(
            c.receive(&group_id, &from_restored).unwrap(),
            (ALICE.to_vec(), b"post-import pong".to_vec())
        );
    }

    #[test]
    fn export_is_deterministic_and_round_trips_byte_identically() {
        let group_id = [50u8; 32];
        let (a, _b, _c) = three_member_group(&group_id);

        // Same state -> same bytes (entries are sorted, not HashMap-ordered).
        let first = a.export_state().unwrap();
        let second = a.export_state().unwrap();
        assert_eq!(first, second);

        // import -> export reproduces the exact bytes: nothing is lost or
        // reordered across the round trip.
        let restored = MlsEngine::import_state(&first).unwrap();
        assert_eq!(restored.export_state().unwrap(), first);
    }

    #[test]
    fn minted_key_package_survives_restart() {
        // The realistic invite flow: B mints + publishes a KeyPackage, the app
        // restarts (export/import), THEN the invite arrives. The KeyPackage's
        // private keys live in the exported storage, so the restored engine
        // can still join.
        let a = engine(ALICE);
        let b = engine(BOB);
        let key_package = b.fresh_key_package().unwrap();
        let restored_b = MlsEngine::import_state(&b.export_state().unwrap()).unwrap();
        drop(b);

        let group_id = [88u8; 32];
        a.create_group(&group_id).unwrap();
        let outcome = a.add_member(&group_id, &key_package).unwrap();
        assert_eq!(
            restored_b.join_from_welcome(&outcome.welcome_wire).unwrap(),
            group_id
        );

        let wire = restored_b.send(&group_id, b"alive after restart").unwrap();
        assert_eq!(
            a.receive(&group_id, &wire).unwrap(),
            (BOB.to_vec(), b"alive after restart".to_vec())
        );
    }

    #[test]
    fn remove_member_removes_all_leaves_sharing_an_identity() {
        // MLS permits two leaves with the SAME BasicCredential identity (a second device, or a relay
        // re-presenting a KeyPackage). The app removes by IDENTITY, so "remove bob" MUST evict EVERY
        // bob leaf -- removing only the first would leave the duplicate a full member of the rotated
        // epoch, still able to decrypt (a silent membership-forward-secrecy break).
        let group_id = [0x5b; 32];
        let a = engine(ALICE);
        let b1 = engine(BOB);
        let b2 = engine(BOB); // a SECOND, independent engine carrying the SAME identity bytes
        a.create_group(&group_id).unwrap();

        let add_b1 = a
            .add_member(&group_id, &b1.fresh_key_package().unwrap())
            .unwrap();
        assert_eq!(b1.join_from_welcome(&add_b1.welcome_wire).unwrap(), group_id);
        let add_b2 = a
            .add_member(&group_id, &b2.fresh_key_package().unwrap())
            .unwrap();
        b1.apply_commit(&group_id, &add_b2.commit_wire).unwrap();
        assert_eq!(b2.join_from_welcome(&add_b2.welcome_wire).unwrap(), group_id);

        // Two leaves, both BOB: the roster carries BOB twice, and a baseline send reaches both.
        assert_eq!(
            a.roster(&group_id)
                .unwrap()
                .into_iter()
                .filter(|id| id.as_slice() == BOB)
                .count(),
            2,
            "two BOB leaves exist before removal"
        );
        let pre = a.send(&group_id, b"before removal").unwrap();
        assert_eq!(b1.receive(&group_id, &pre).unwrap().1, b"before removal");
        assert_eq!(b2.receive(&group_id, &pre).unwrap().1, b"before removal");

        // Remove BOB: with the fix, BOTH leaves go in one commit -> no BOB remains on A's roster.
        let remove_commit = a.remove_member(&group_id, BOB).unwrap();
        assert_eq!(
            a.roster(&group_id)
                .unwrap()
                .into_iter()
                .filter(|id| id.as_slice() == BOB)
                .count(),
            0,
            "every BOB leaf removed (removing only the first would leave the duplicate a full member)"
        );

        // End-to-end: a post-removal send reaches NEITHER bob engine -- even the duplicate is evicted.
        let _ = b1.apply_commit(&group_id, &remove_commit);
        let _ = b2.apply_commit(&group_id, &remove_commit);
        let post = a.send(&group_id, b"after removal").unwrap();
        assert!(
            b1.receive(&group_id, &post).is_err(),
            "the first bob leaf is removed"
        );
        assert!(
            b2.receive(&group_id, &post).is_err(),
            "the duplicate bob leaf is removed too"
        );
    }

    #[test]
    fn fresh_key_packages_are_single_group() {
        let a = engine(ALICE);
        let b = engine(BOB);
        let group_one = [55u8; 32];
        let group_two = [56u8; 32];
        a.create_group(&group_one).unwrap();
        a.create_group(&group_two).unwrap();

        // B's KeyPackage is consumed by the first add.
        let key_package = b.fresh_key_package().unwrap();
        let first = a.add_member(&group_one, &key_package).unwrap();
        assert_eq!(b.join_from_welcome(&first.welcome_wire).unwrap(), group_one);

        // Re-presenting the SAME wire bytes to ANOTHER group: OpenMLS
        // re-validates the signatures (they still verify), so the add succeeds
        // CRYPTOGRAPHICALLY -- MLS itself does not police cross-group reuse at
        // the adder. Returning at all (Ok or Err) is the no-panic guarantee
        // this test pins. SINGLE-USE ENFORCEMENT IS THE CALLER'S JOB:
        // mercury-app mints a fresh KeyPackage per invite, and the crate's
        // consume-store gate (put_mls_key_package_consumption_record) rejects
        // a reused KeyPackage hash.
        match a.add_member(&group_two, &key_package) {
            Ok(outcome) => assert_eq!(outcome.added_identity, BOB),
            Err(_) => {} // also acceptable: rejected without panicking
        }
    }

    #[test]
    fn import_state_fails_closed() {
        let group_id = [66u8; 32];
        let (a, _b, _c) = three_member_group(&group_id);
        let exported = a.export_state().unwrap();

        // Control: the intact bytes import fine (failures below are caused by
        // the corruption, not a broken importer).
        assert!(MlsEngine::import_state(&exported).is_ok());

        // Empty + truncated at many offsets: every cut fails closed.
        assert!(MlsEngine::import_state(&[]).is_err());
        for cut in (0..exported.len()).step_by(97) {
            assert!(
                MlsEngine::import_state(&exported[..cut]).is_err(),
                "truncation at {cut} must be rejected"
            );
        }
        assert!(MlsEngine::import_state(&exported[..exported.len() - 1]).is_err());

        // Unknown / flipped version byte.
        let mut wrong_version = exported.clone();
        wrong_version[0] = 0x02;
        assert!(MlsEngine::import_state(&wrong_version).is_err());
        wrong_version[0] = 0x00;
        assert!(MlsEngine::import_state(&wrong_version).is_err());

        // Trailing garbage after a structurally complete state.
        let mut padded = exported.clone();
        padded.push(0u8);
        assert!(MlsEngine::import_state(&padded).is_err());

        // Corrupt the signer JSON in place (its first byte can no longer open
        // a JSON object) -> serde rejects -> fail closed.
        let mut bad_signer = exported.clone();
        let signer_json_at = 1 + 4 + ALICE.len() + 4;
        bad_signer[signer_json_at] = b'X';
        assert!(MlsEngine::import_state(&bad_signer).is_err());

        // Arbitrary deterministic garbage. Reaching the end of this test IS
        // the no-panic proof: every call above returned an Err normally.
        let garbage: Vec<u8> = (0u32..512)
            .map(|i| (i.wrapping_mul(2_654_435_761) >> 24) as u8)
            .collect();
        assert!(MlsEngine::import_state(&garbage).is_err());
    }

    #[test]
    fn receive_rejects_cross_group_and_garbage() {
        let group_one = [77u8; 32];
        let group_two = [78u8; 32];
        let (a, b, _c) = three_member_group(&group_one);

        // A second, unrelated group that B is NOT in (a 1-member group still
        // encrypts to itself).
        a.create_group(&group_two).unwrap();
        let alien = a.send(&group_two, b"other room").unwrap();

        // Cross-group: a frame for group_two cannot open against B's
        // group_one view.
        assert!(b.receive(&group_one, &alien).is_err());
        // Unknown group id: fails closed before touching the bytes.
        assert_eq!(
            b.receive(&group_two, &alien).unwrap_err(),
            MlsError::GroupNotFound
        );

        // Garbage / empty / truncated frames: Err, never a panic.
        assert!(b.receive(&group_one, &[0u8; 64]).is_err());
        assert!(b.receive(&group_one, &[]).is_err());
        let real = a.send(&group_one, b"real").unwrap();
        assert!(b.receive(&group_one, &real[..real.len() / 2]).is_err());

        // The failures above did not poison B's state: the intact frame opens.
        assert_eq!(
            b.receive(&group_one, &real).unwrap(),
            (ALICE.to_vec(), b"real".to_vec())
        );
    }

    #[test]
    fn apply_commit_rejects_non_commits() {
        let group_id = [99u8; 32];
        let (a, b, _c) = three_member_group(&group_id);

        // An application message is NOT a commit: rejected BEFORE processing,
        // so its one-time ratchet secret is not consumed by the wrong path.
        let app_wire = a.send(&group_id, b"not a commit").unwrap();
        assert!(b.apply_commit(&group_id, &app_wire).is_err());
        // Garbage is rejected too.
        assert!(b.apply_commit(&group_id, &[0u8; 32]).is_err());
        // B's group state is intact: the app message still decrypts.
        assert_eq!(
            b.receive(&group_id, &app_wire).unwrap(),
            (ALICE.to_vec(), b"not a commit".to_vec())
        );
    }

    #[test]
    fn roster_identities_match() {
        let group_id = [12u8; 32];
        let (a, b, c) = three_member_group(&group_id);
        let expected = sorted(vec![ALICE.to_vec(), BOB.to_vec(), CAROL.to_vec()]);
        // Every member's view lists the same three identities, self included.
        assert_eq!(sorted(a.roster(&group_id).unwrap()), expected);
        assert_eq!(sorted(b.roster(&group_id).unwrap()), expected);
        assert_eq!(sorted(c.roster(&group_id).unwrap()), expected);
    }

    #[test]
    fn missing_group_fails_closed_everywhere() {
        let lonely = engine(b"solo@mercury");
        let group_id = [13u8; 32];
        assert!(!lonely.has_group(&group_id));
        assert_eq!(
            lonely.send(&group_id, b"x").unwrap_err(),
            MlsError::GroupNotFound
        );
        assert_eq!(
            lonely.receive(&group_id, b"x").unwrap_err(),
            MlsError::GroupNotFound
        );
        assert_eq!(
            lonely.roster(&group_id).unwrap_err(),
            MlsError::GroupNotFound
        );
        assert_eq!(
            lonely.remove_member(&group_id, BOB).unwrap_err(),
            MlsError::GroupNotFound
        );
        assert_eq!(
            lonely.apply_commit(&group_id, b"x").unwrap_err(),
            MlsError::GroupNotFound
        );
        assert_eq!(
            lonely.leave_local(&group_id).unwrap_err(),
            MlsError::GroupNotFound
        );
        let other = engine(b"other@mercury");
        assert_eq!(
            lonely
                .add_member(&group_id, &other.fresh_key_package().unwrap())
                .unwrap_err(),
            MlsError::GroupNotFound
        );
    }

    #[test]
    fn create_group_refuses_to_clobber() {
        let a = engine(ALICE);
        let group_id = [14u8; 32];
        a.create_group(&group_id).unwrap();
        // Re-creating an occupied id would wipe a live group's secrets.
        assert_eq!(a.create_group(&group_id), Err(MlsError::GroupCreate));
        // The original group is untouched.
        assert_eq!(a.roster(&group_id).unwrap(), vec![ALICE.to_vec()]);
    }

    #[test]
    fn add_member_rejects_hostile_key_package_bytes() {
        let a = engine(ALICE);
        let group_id = [15u8; 32];
        a.create_group(&group_id).unwrap();
        // Garbage, empty, and truncated KeyPackage wires all fail closed.
        assert!(a.add_member(&group_id, &[0u8; 64]).is_err());
        assert!(a.add_member(&group_id, &[]).is_err());
        let b = engine(BOB);
        let key_package = b.fresh_key_package().unwrap();
        assert!(
            a.add_member(&group_id, &key_package[..key_package.len() / 2])
                .is_err()
        );
        // A tampered byte breaks the signature -> validation rejects.
        let mut tampered = key_package.clone();
        let last = tampered.len() - 1;
        tampered[last] ^= 0xFF;
        assert!(a.add_member(&group_id, &tampered).is_err());
        // The intact package still admits (the rejections above were the
        // corruption, not a broken validator).
        assert!(a.add_member(&group_id, &key_package).is_ok());
    }
}
