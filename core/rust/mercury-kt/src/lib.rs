#![forbid(unsafe_code)]
//! Mercury Key Transparency engine (Stage 6).
//!
//! Real AKD-backed device-key transparency: an auditable directory mapping a
//! device label -> its current key, with cryptographic INCLUSION proofs a
//! client verifies against a pinned directory public key. The engine's job is
//! to verify AKD proofs and PRODUCE the [`KeyTransparencyProofInput`] that
//! `mercury_core::evaluate_key_transparency` consumes -- it never re-decides
//! transparency policy (the gate does that).
//!
//! This now wires three of the four proof dimensions end-to-end against real
//! AKD: INCLUSION (a label maps to the expected key under a pinned checkpoint),
//! CONSISTENCY (the directory only ever appended -- never deleted or rewrote a
//! committed entry -- between two pinned checkpoints), and KEY-HISTORY (the
//! full, ordered rotation chain for a label is provable). The client path is
//! [`verify_transparency_bundle`], which binds all three proofs to a SINGLE
//! directory head (so a relay cannot mix valid-but-unrelated proofs) and derives
//! the gate's anti-rollback tree sizes from the pinned epoch chain. The
//! witness/auditor quorum (a signed tree head authenticating that head, which is
//! also what makes proof freshness trustworthy) is the remaining increment;
//! [`proof_input`] already takes it as an explicit per-dimension status so it
//! drops in without changing callers.

use akd::ecvrf::{VRFKeyStorage, VrfError};
use akd::storage::memory::AsyncInMemoryDatabase;
use akd::storage::types::DbRecord;
// `Database` (batch_set) + `StorageUtil` (batch_get_all_direct) drive snapshot/restore; the inner
// db's records are dumped/reloaded directly because StorageManager::get_db is test-cfg-gated.
use akd::storage::{Database, DbSetState, StorageManager, StorageUtil};
use std::path::{Path, PathBuf};
use akd::{
    AkdLabel, AkdValue, AzksParallelismConfig, Directory, HistoryParams, HistoryVerificationParams,
};
use ed25519_dalek::{Signer, SigningKey};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha512};
use zeroize::{Zeroize, ZeroizeOnDrop};

// Re-export the AKD proof + checkpoint types the verify_* functions consume, so
// a consumer that receives a proof over the wire can name + deserialize it.
pub use akd::{AppendOnlyProof, EpochHash, HistoryProof, LookupProof};
pub use mercury_core::{
    KeyTransparencyProofInput, KeyTransparencyProofStatus, KeyTransparencyWitnessStatus,
};

mod witness;
pub use witness::{
    SignedTreeHead, Witness, WitnessCosignature, WitnessVerification, verify_signed_tree_head,
    verify_witnessed_tree_head,
};

// (`verify_against_signed_head` below is the head-authenticating client capstone.)

/// AKD configuration profile (the WhatsApp-style one that WhatsApp KT uses).
type Config = akd::WhatsAppV1Configuration;
// Directory's 2nd generic is the Database itself; `new` wraps it in a
// StorageManager. The 3rd is the VRF key store.
type AkdDirectory = Directory<Config, AsyncInMemoryDatabase, DirectoryVrf>;

/// A PER-DIRECTORY VRF key store. AKD's `HardCodedAkdVRF` is a fixed demo key
/// shared by every instance, so client key-pinning would be meaningless against
/// it; this holds a real, distinct 32-byte VRF private key per directory (a
/// random one for tests, or a persisted one for production), giving each
/// directory a distinct, pinnable VRF public key.
///
/// SECURITY: the VRF private key is sensitive (it binds labels to tree
/// positions); it zeroizes on drop and, in production, MUST be generated once
/// and PERSISTED in secure storage off the relay's serving path -- regenerating
/// it would change the public key and invalidate every previously-pinned proof.
#[derive(Clone, ZeroizeOnDrop)]
struct DirectoryVrf {
    private_key: [u8; 32],
}

#[async_trait::async_trait]
impl VRFKeyStorage for DirectoryVrf {
    async fn retrieve(&self) -> Result<Vec<u8>, VrfError> {
        Ok(self.private_key.to_vec())
    }
}

/// An in-memory device-key transparency directory backed by real AKD.
///
/// In production the storage backend is durable + replicated and the directory
/// lives on the relay (Stage 4); the in-memory store keeps this engine + its
/// tests self-contained.
pub struct KtDirectory {
    dir: AkdDirectory,
    /// A SHARED handle (the akd in-memory db is `Arc<DashMap>`-backed and `Clone`, so this clone
    /// points at the SAME records the `dir` reads/writes) used to dump the full record set for
    /// at-rest persistence. `StorageManager::get_db()` is test-cfg-gated, so KtDirectory keeps its
    /// own clone instead of reaching back through the manager.
    db: AsyncInMemoryDatabase,
    /// The LOG's Ed25519 key for signing tree heads. Distinct from the VRF key
    /// (different purpose), derived from the same persisted seed so it is stable
    /// across restarts. Clients pin its public half to verify served heads.
    log_key: SigningKey,
    /// When `Some`, [`KtDirectory::snapshot`] persists the AKD state here (set only by
    /// [`KtDirectory::with_vrf_seed_persistent`]); `None` for ephemeral / test directories.
    snapshot_path: Option<PathBuf>,
}

/// Why restoring (or first reading) a persisted AKD snapshot failed. The relay boot path treats
/// EVERY variant as FAIL-CLOSED — refuse to start. A corrupt/tampered/unreadable KT snapshot must
/// never be silently papered over by rebuilding from the username bindings, because that would
/// reset the epoch history (the exact bug persistence exists to fix) and could mask on-disk tamper.
/// Only a genuinely ABSENT snapshot file is the (non-error) first-run / migration path.
#[derive(Debug)]
pub enum KtPersistError {
    /// Reading or writing the snapshot file failed.
    Io(std::io::Error),
    /// The snapshot bytes did not deserialize to an AKD record set.
    Deserialize(serde_json::Error),
    /// An akd storage/directory operation failed (stringified, since the akd traits surface both
    /// `StorageError` and `AkdError`).
    Akd(String),
    /// The records deserialized but are not a usable AKD state (e.g. not exactly one Azks record).
    CorruptSnapshot(String),
}

impl std::fmt::Display for KtPersistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KtPersistError::Io(e) => write!(f, "KT snapshot I/O error: {e}"),
            KtPersistError::Deserialize(e) => write!(f, "KT snapshot deserialize error: {e}"),
            KtPersistError::Akd(m) => write!(f, "KT snapshot akd error: {m}"),
            KtPersistError::CorruptSnapshot(m) => write!(f, "KT snapshot corrupt: {m}"),
        }
    }
}
impl std::error::Error for KtPersistError {}

/// Domain separator for deriving the log signing key from the directory seed.
const LOG_KEY_DOMAIN: &[u8] = b"mercury/kt/log-key/v1";

/// Derive the directory's log signing key from its seed via a domain-separated
/// hash, so one persisted seed yields BOTH a stable VRF key and a stable,
/// DISTINCT log key (the raw seed is the VRF key; this KDF output is the log
/// key). One-way: the log seed reveals nothing about the VRF seed.
fn derive_log_key(seed: &[u8; 32]) -> SigningKey {
    let mut hasher = Sha512::new();
    hasher.update(LOG_KEY_DOMAIN);
    hasher.update(seed);
    let digest = hasher.finalize();
    let mut log_seed = [0u8; 32];
    log_seed.copy_from_slice(&digest[..32]);
    let key = SigningKey::from_bytes(&log_seed);
    // Wipe the transient seed copy; the key (with the dalek `zeroize` feature)
    // wipes itself on drop.
    log_seed.zeroize();
    key
}

impl KtDirectory {
    /// Create a fresh empty directory with a NEWLY GENERATED per-directory VRF
    /// key from the OS CSPRNG. Convenient for tests + ephemeral directories; a
    /// production directory that must outlive the process should instead use
    /// [`KtDirectory::with_vrf_seed`] with a persisted key.
    pub async fn new() -> Result<Self, akd::errors::AkdError> {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        Self::with_vrf_seed(seed).await
    }

    /// Create a fresh empty directory using the supplied 32-byte VRF private
    /// key. Production loads this from secure storage so the directory's pinnable
    /// public key is STABLE across restarts (regenerating it would invalidate
    /// every proof clients pinned against the old key).
    pub async fn with_vrf_seed(vrf_private_key: [u8; 32]) -> Result<Self, akd::errors::AkdError> {
        Self::from_db(AsyncInMemoryDatabase::new(), vrf_private_key, None).await
    }

    /// Production boot entry: the same VRF seed, plus a `snapshot_path`. If a snapshot file exists
    /// there it is RESTORED — the directory resumes its exact epoch history, so consistency proofs
    /// span the restart. If it does NOT exist, this is the first-run / migration path (a fresh
    /// empty directory the caller then rebuilds from the durable username bindings). Returns
    /// `(directory, restored)` where `restored` is true iff a snapshot was loaded.
    ///
    /// FAIL-CLOSED: an unreadable / undeserializable / structurally-corrupt snapshot, or a restore
    /// failure, is returned as [`KtPersistError`] — the caller MUST refuse to start, never silently
    /// rebuild (which would reset epoch history, the bug this fixes, and could mask on-disk tamper).
    pub async fn with_vrf_seed_persistent(
        vrf_private_key: [u8; 32],
        snapshot_path: impl AsRef<Path>,
    ) -> Result<(Self, bool), KtPersistError> {
        let path = snapshot_path.as_ref().to_path_buf();
        let db = AsyncInMemoryDatabase::new();
        let restored = if path.exists() {
            let bytes = std::fs::read(&path).map_err(KtPersistError::Io)?;
            let records: Vec<DbRecord> =
                serde_json::from_slice(&bytes).map_err(KtPersistError::Deserialize)?;
            if records.is_empty() {
                return Err(KtPersistError::CorruptSnapshot("empty record set".into()));
            }
            // A valid AKD state has exactly ONE Azks (the tree root + epoch counter). Anything else
            // is a truncated / duplicated / tampered dump — fail closed rather than load it.
            let azks_count = records.iter().filter(|r| matches!(r, DbRecord::Azks(_))).count();
            if azks_count != 1 {
                return Err(KtPersistError::CorruptSnapshot(format!(
                    "expected exactly 1 Azks record, found {azks_count}"
                )));
            }
            db.batch_set(records, DbSetState::General)
                .await
                .map_err(|e| KtPersistError::CorruptSnapshot(format!("batch_set failed: {e:?}")))?;
            true
        } else {
            false
        };
        let me = Self::from_db(db, vrf_private_key, Some(path))
            .await
            .map_err(|e| KtPersistError::Akd(format!("{e:?}")))?;
        Ok((me, restored))
    }

    /// Wrap an (already restored, or empty) in-memory db in a Directory, keeping a shared handle to
    /// the db for snapshotting. `Directory::new` reuses an existing Azks from storage when present,
    /// so a restored db resumes its epoch history instead of minting a fresh epoch 0.
    async fn from_db(
        db: AsyncInMemoryDatabase,
        vrf_private_key: [u8; 32],
        snapshot_path: Option<PathBuf>,
    ) -> Result<Self, akd::errors::AkdError> {
        let storage = StorageManager::new_no_cache(db.clone());
        let vrf = DirectoryVrf {
            private_key: vrf_private_key,
        };
        let dir = Directory::new(storage, vrf, AzksParallelismConfig::default()).await?;
        Ok(Self {
            dir,
            db,
            log_key: derive_log_key(&vrf_private_key),
            snapshot_path,
        })
    }

    /// Atomically persist the FULL AKD record set to the configured snapshot path (a no-op when
    /// none is set, e.g. ephemeral / test directories). The relay calls this after every
    /// epoch-advancing publish — inside the registry critical section, AFTER the durable username
    /// write — so the on-disk AKD never lags a recorded binding. Crash-atomic: tmp-write + fsync +
    /// atomic rename, plus a best-effort parent-dir fsync on Unix so the rename itself is durable.
    pub async fn snapshot(&self) -> Result<(), KtPersistError> {
        let Some(path) = &self.snapshot_path else {
            return Ok(());
        };
        let records: Vec<DbRecord> = self
            .db
            .batch_get_all_direct()
            .await
            .map_err(|e| KtPersistError::Akd(format!("dump failed: {e:?}")))?;
        let bytes = serde_json::to_vec(&records).map_err(KtPersistError::Deserialize)?;
        let tmp = path.with_extension("tmp");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&tmp).map_err(KtPersistError::Io)?;
            f.write_all(&bytes).map_err(KtPersistError::Io)?;
            f.sync_all().map_err(KtPersistError::Io)?;
        }
        std::fs::rename(&tmp, path).map_err(KtPersistError::Io)?;
        #[cfg(unix)]
        if let Some(dir) = path.parent() {
            if let Ok(d) = std::fs::File::open(dir) {
                let _ = d.sync_all();
            }
        }
        Ok(())
    }

    /// Publish (or rotate) a device's key under `label`. Returns the new
    /// `(epoch, root_hash)` checkpoint.
    pub async fn register(
        &mut self,
        label: &str,
        key: &[u8],
    ) -> Result<EpochHash, akd::errors::AkdError> {
        self.dir
            .publish(vec![(AkdLabel::from(label), AkdValue(key.to_vec()))])
            .await
    }

    /// Publish a BATCH of device-key updates in a SINGLE epoch. Real key
    /// transparency at scale batches many registrations/rotations per epoch --
    /// one tree update and one signed tree head amortized over all of them --
    /// rather than minting an epoch per key. Returns the one checkpoint that
    /// covers every update in the batch. Labels within a batch must be distinct
    /// (AKD rejects a duplicate label in a single publish).
    pub async fn register_batch(
        &mut self,
        updates: &[(String, Vec<u8>)],
    ) -> Result<EpochHash, akd::errors::AkdError> {
        // Reject an empty batch rather than minting an epoch with no key updates
        // (that would advance the log + signed tree head while attesting nothing,
        // leaving a misleading gap in the transparency history). AKD itself
        // rejects duplicate labels in a publish, so distinctness is enforced too.
        if updates.is_empty() {
            return Err(akd::errors::AkdError::Directory(
                akd::errors::DirectoryError::Publish(
                    "cannot publish an empty batch (would mint an empty epoch)".to_string(),
                ),
            ));
        }
        let publish: Vec<(AkdLabel, AkdValue)> = updates
            .iter()
            .map(|(label, key)| (AkdLabel::from(label.as_str()), AkdValue(key.clone())))
            .collect();
        self.dir.publish(publish).await
    }

    /// Produce a cryptographic inclusion (lookup) proof for `label`, together
    /// with the directory checkpoint it is relative to.
    pub async fn prove_inclusion(
        &self,
        label: &str,
    ) -> Result<(LookupProof, EpochHash), akd::errors::AkdError> {
        self.dir.lookup(AkdLabel::from(label)).await
    }

    /// Produce a CONSISTENCY (append-only) proof that the directory evolved from
    /// `start_epoch` to `end_epoch` WITHOUT removing or rewriting any committed
    /// entry. The client verifies it against the per-epoch root hashes it has
    /// pinned (one checkpoint per epoch boundary the proof spans), so a relay
    /// that quietly forks the log or rewrites history cannot pass it.
    pub async fn prove_consistency(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<AppendOnlyProof, akd::errors::AkdError> {
        self.dir.audit(start_epoch, end_epoch).await
    }

    /// Produce a full KEY-HISTORY proof for `label`: the complete, ordered chain
    /// of every version (key rotation) committed for that label, anchored to the
    /// directory's current checkpoint. The client verifies the whole chain so an
    /// inserted or omitted rotation is detectable.
    pub async fn prove_key_history(
        &self,
        label: &str,
    ) -> Result<(HistoryProof, EpochHash), akd::errors::AkdError> {
        self.dir
            .key_history(&AkdLabel::from(label), HistoryParams::Complete)
            .await
    }

    /// The directory's VRF public key. Clients pin this out-of-band and verify
    /// every proof against it.
    ///
    /// This is now a REAL per-directory key derived from the instance's VRF
    /// private key (see [`DirectoryVrf`] / [`KtDirectory::with_vrf_seed`]): two
    /// directories produce distinct public keys, so a proof from one fails
    /// verification against the other's pinned key (real VRF-key-substitution
    /// defense). Production must persist the VRF private key so this stays stable
    /// across restarts.
    pub async fn public_key(&self) -> Result<Vec<u8>, akd::errors::AkdError> {
        Ok(self.dir.get_public_key().await?.as_bytes().to_vec())
    }

    /// The LOG's Ed25519 public key (32 bytes). Clients pin this to verify the
    /// directory's signed tree heads; it is DISTINCT from the VRF [`public_key`].
    pub fn log_public_key(&self) -> [u8; 32] {
        self.log_key.verifying_key().to_bytes()
    }

    /// Produce a SIGNED TREE HEAD over the directory's current checkpoint: the
    /// log's Ed25519 signature over `(tree_size = epoch, root_hash, timestamp_s)`.
    /// Clients verify it with [`verify_signed_tree_head`] against the pinned log
    /// key, then (in the full protocol) gather witness co-signatures over the
    /// same head. `timestamp_s` is supplied by the caller so the directory stays
    /// clock-free; it drives downstream freshness.
    pub async fn signed_tree_head(
        &self,
        timestamp_s: i64,
    ) -> Result<(SignedTreeHead, [u8; 64]), akd::errors::AkdError> {
        let checkpoint = self.dir.get_epoch_hash().await?;
        let sth = SignedTreeHead {
            tree_size: checkpoint.epoch(),
            root_hash: checkpoint.hash(),
            timestamp_s,
        };
        let signature = self.log_key.sign(&sth.signing_bytes()).to_bytes();
        Ok((sth, signature))
    }
}

/// Client-side: verify an inclusion proof against a pinned `public_key` and the
/// `checkpoint` it claims, returning the gate's proof status. `Verified` iff the
/// AKD proof checks out AND the recovered value equals the `expected_key` the
/// caller believes is current (so a valid proof for a *different* key is
/// surfaced as `Invalid` -- a key substitution the gate then rejects).
pub fn verify_inclusion(
    public_key: &[u8],
    checkpoint: &EpochHash,
    label: &str,
    expected_key: &[u8],
    proof: LookupProof,
) -> KeyTransparencyProofStatus {
    // A device key (and the pinned VRF key) is never empty; reject rather than
    // risk a vacuous match against an empty committed value.
    if expected_key.is_empty() || public_key.is_empty() {
        return KeyTransparencyProofStatus::Invalid;
    }
    match akd::client::lookup_verify::<Config>(
        public_key,
        checkpoint.hash(),
        checkpoint.epoch(),
        AkdLabel::from(label),
        proof,
    ) {
        Ok(result) if result.value.0.as_slice() == expected_key => {
            KeyTransparencyProofStatus::Verified
        }
        _ => KeyTransparencyProofStatus::Invalid,
    }
}

/// Client-side: verify a CONSISTENCY (append-only) proof linking a sequence of
/// pinned epoch checkpoints, returning the gate's proof status. `checkpoints`
/// are the root hashes the client has pinned, in STRICTLY INCREASING epoch
/// order, one per epoch boundary the proof spans -- so `checkpoints.len()` must
/// be `proof.epochs.len() + 1` (AKD's `audit_verify` enforces this). `Verified`
/// iff every listed epoch transition only ADDED leaves; a transition that
/// deleted or rewrote a committed key fails the append-only hash and is surfaced
/// as `Invalid` (a log-rewrite / split-view the gate then rejects).
pub async fn verify_consistency(
    checkpoints: &[EpochHash],
    proof: AppendOnlyProof,
) -> KeyTransparencyProofStatus {
    // A consistency claim links at least a start and an end checkpoint; fewer
    // than two cannot bound any transition, so reject rather than vacuously pass.
    if checkpoints.len() < 2 {
        return KeyTransparencyProofStatus::Invalid;
    }
    // BIND the proof to the pinned EPOCH chain, not just its hashes. On its own
    // `audit_verify` only checks the root hashes line up -- it would accept a
    // valid append-only proof for a *different* epoch window whose hashes happen
    // to match the pinned ones, and it would accept a degenerate two-equal-epoch
    // chain. So we additionally require the pinned checkpoints to be a strictly
    // CONSECUTIVE increasing epoch chain AND each proof step's start epoch to
    // equal the chain's -- i.e. the proof attests exactly the transitions the
    // client pinned, in order.
    if proof.epochs.len() + 1 != checkpoints.len() {
        return KeyTransparencyProofStatus::Invalid;
    }
    for i in 0..checkpoints.len() - 1 {
        let start = checkpoints[i].epoch();
        // `start + 1` must not wrap: u64::MAX is not a reachable directory epoch,
        // and a wrap would let epoch 0 masquerade as "consecutive" to it. Use
        // checked_add so the guard fails CLOSED at the boundary instead.
        match start.checked_add(1) {
            Some(next) if checkpoints[i + 1].epoch() == next && proof.epochs[i] == start => {}
            _ => return KeyTransparencyProofStatus::Invalid,
        }
    }
    let hashes: Vec<akd::Digest> = checkpoints.iter().map(EpochHash::hash).collect();
    match akd::auditor::audit_verify::<Config>(hashes, proof).await {
        Ok(()) => KeyTransparencyProofStatus::Verified,
        Err(_) => KeyTransparencyProofStatus::Invalid,
    }
}

/// Client-side: verify a KEY-HISTORY proof against a pinned `public_key` and the
/// `checkpoint` it is anchored to, returning the gate's proof status. `Verified`
/// iff EVERY update proof in the rotation chain checks out against the pinned
/// VRF key and checkpoint root. We verify with the default (strict) params, so a
/// history containing any tombstoned / unprovable value fails verification --
/// surfaced as `Invalid` so the gate rejects rather than trusting an
/// unverifiable rotation chain.
pub fn verify_key_history(
    public_key: &[u8],
    checkpoint: &EpochHash,
    label: &str,
    proof: HistoryProof,
) -> KeyTransparencyProofStatus {
    // The pinned VRF key is never empty; reject rather than hand an empty key to
    // the verifier (mirrors the inclusion guard).
    if public_key.is_empty() {
        return KeyTransparencyProofStatus::Invalid;
    }
    match akd::client::key_history_verify::<Config>(
        public_key,
        checkpoint.hash(),
        checkpoint.epoch(),
        AkdLabel::from(label),
        proof,
        HistoryVerificationParams::default(),
    ) {
        // Default (strict) params reject tombstoned values during verification,
        // so a successful return means every version in the chain was proven.
        Ok(_) => KeyTransparencyProofStatus::Verified,
        Err(_) => KeyTransparencyProofStatus::Invalid,
    }
}

/// The three relay-served proofs for one device-key lookup, verified together as
/// a single bound bundle by [`verify_transparency_bundle`].
pub struct TransparencyBundle {
    /// Inclusion (lookup) proof: the label maps to the expected key.
    pub inclusion: LookupProof,
    /// Consistency (append-only) proof linking the client's last-pinned
    /// checkpoint(s) up to the current head.
    pub consistency: AppendOnlyProof,
    /// Full key-history (rotation chain) proof for the label.
    pub key_history: HistoryProof,
}

/// Represent an AKD epoch (a u64 monotonic counter from 1) as the gate's i64
/// "tree size". Saturates rather than wrapping, so an absurd epoch can never flip
/// negative and defeat the gate's `current < previous` rollback check. Epochs
/// this large are unreachable in practice; the saturation just keeps the verifier
/// fail-closed at the type boundary.
fn epoch_as_tree_size(epoch: u64) -> i64 {
    i64::try_from(epoch).unwrap_or(i64::MAX)
}

/// Verify a transparency bundle as ONE bound decision and assemble the gate
/// input. This is the composition `mercury_core::evaluate_key_transparency`
/// relies on: instead of three independently-supplied checkpoints (which a relay
/// could mix -- a valid inclusion proof at one epoch paired with an unrelated
/// consistency/history proof at another), all three proofs are bound to a SINGLE
/// `current` head:
///   * inclusion and key-history are verified against `current` (its epoch AND
///     root, via [`verify_inclusion`] / [`verify_key_history`]);
///   * the consistency proof must link the client's `pinned_chain` (its
///     last-trusted checkpoints, OLDEST first) ending exactly at `current` as
///     its final element, with [`verify_consistency`] binding the proof's epochs
///     to that chain.
///
/// The anti-rollback tree sizes handed to the gate are DERIVED from the chain's
/// first + last epoch (monotonic by construction) -- not trusted from the caller
/// -- so the gate's rollback check is fed real values.
///
/// On `current` AUTHENTICATION: this function takes `current` (the tail of
/// `pinned_chain`) as given. For an end-to-end check that first authenticates the
/// head via the log's signature, call [`verify_against_signed_head`], which wraps
/// this and now DERIVES the `proof_age_s` freshness window from the signed tree
/// head's authenticated timestamp (a future-dated or stale head is rejected). This
/// lower-level entry still takes `proof_age_s` directly.
#[allow(clippy::too_many_arguments)]
pub async fn verify_transparency_bundle(
    public_key: &[u8],
    pinned_chain: &[EpochHash],
    label: &str,
    expected_key: &[u8],
    bundle: TransparencyBundle,
    witness: KeyTransparencyWitnessStatus,
    require_witness: bool,
    proof_age_s: i64,
    max_proof_age_s: i64,
) -> KeyTransparencyProofInput {
    // Need at least a previous + current checkpoint to bound an append-only
    // span; without it there is nothing to prove, so fail every dimension closed.
    let (previous, current) = match (pinned_chain.first(), pinned_chain.last()) {
        (Some(previous), Some(current)) if pinned_chain.len() >= 2 => (previous, current),
        _ => {
            return proof_input(
                KeyTransparencyProofStatus::Invalid,
                KeyTransparencyProofStatus::Invalid,
                KeyTransparencyProofStatus::Invalid,
                witness,
                require_witness,
                0,
                0,
                proof_age_s,
                max_proof_age_s,
            );
        }
    };

    // All three dimensions bind to the same `current` head.
    let inclusion = verify_inclusion(public_key, current, label, expected_key, bundle.inclusion);
    let key_history = verify_key_history(public_key, current, label, bundle.key_history);
    let consistency = verify_consistency(pinned_chain, bundle.consistency).await;

    proof_input(
        inclusion,
        consistency,
        key_history,
        witness,
        require_witness,
        epoch_as_tree_size(previous.epoch()), // DERIVED not trusted
        epoch_as_tree_size(current.epoch()),  // tree size at the current head
        proof_age_s,
        max_proof_age_s,
    )
}

/// Client-side END-TO-END verification against a directory's AUTHENTICATED head.
/// This is the head-authenticating wrapper around [`verify_transparency_bundle`]:
/// rather than trust a caller-asserted head, it verifies the LOG's signature on a
/// [`SignedTreeHead`] (the caller supplies the witness-quorum status for that same
/// head), then uses the authenticated `(tree_size, root)` as the `current`
/// checkpoint the inclusion + key-history + consistency proofs must bind to.
/// `previous_pinned` is the checkpoint the client last trusted; the consistency
/// proof must link it up to the authenticated head.
///
/// If the head's log signature does not verify, EVERY dimension is forced closed
/// -- an unauthenticated head is no evidence at all. This closes the limitation
/// noted on [`verify_transparency_bundle`], where `current` was caller-asserted:
/// here the head is cryptographically authenticated before anything binds to it.
#[allow(clippy::too_many_arguments)]
pub async fn verify_against_signed_head(
    vrf_public_key: &[u8],
    log_public_key: &[u8; 32],
    signed_tree_head: &SignedTreeHead,
    log_signature: &[u8; 64],
    previous_pinned: &EpochHash,
    label: &str,
    expected_key: &[u8],
    bundle: TransparencyBundle,
    witness: KeyTransparencyWitnessStatus,
    require_witness: bool,
    now_s: i64,
    max_proof_age_s: i64,
) -> KeyTransparencyProofInput {
    // Freshness is DERIVED from the head's OWN signed timestamp (authenticated by the signature
    // check below), not caller-asserted: a future-dated head yields a negative age the gate
    // rejects as BadFreshnessWindow, and a stale head exceeds `max_proof_age_s`.
    let proof_age_s = now_s.saturating_sub(signed_tree_head.timestamp_s);

    // Authenticate the head FIRST: a forged/unsigned head is not a head, so the
    // proofs hanging off it cannot be trusted either.
    if !verify_signed_tree_head(log_public_key, signed_tree_head, log_signature) {
        return proof_input(
            KeyTransparencyProofStatus::Invalid,
            KeyTransparencyProofStatus::Invalid,
            KeyTransparencyProofStatus::Invalid,
            witness,
            require_witness,
            0,
            0,
            proof_age_s,
            max_proof_age_s,
        );
    }

    // The authenticated head is the `current` checkpoint everything binds to.
    let current = EpochHash(signed_tree_head.tree_size, signed_tree_head.root_hash);
    let chain = [previous_pinned.clone(), current];
    verify_transparency_bundle(
        vrf_public_key,
        &chain,
        label,
        expected_key,
        bundle,
        witness,
        require_witness,
        proof_age_s,
        max_proof_age_s,
    )
    .await
}

/// Assemble the gate input from per-dimension verified statuses + log
/// freshness. The decision itself is `mercury_core::evaluate_key_transparency`.
///
/// Prefer [`verify_transparency_bundle`] on the client path: it binds the three
/// proofs to one head and derives the tree sizes. This lower-level assembler
/// takes every field directly and so trusts the caller to have bound them; the
/// `proof_age_s` / `max_proof_age_s` freshness window in particular is only
/// meaningful once a signed tree head (witness increment) provides a trustworthy
/// timestamp.
#[allow(clippy::too_many_arguments)]
pub fn proof_input(
    inclusion: KeyTransparencyProofStatus,
    consistency: KeyTransparencyProofStatus,
    key_history: KeyTransparencyProofStatus,
    witness: KeyTransparencyWitnessStatus,
    require_witness: bool,
    previous_tree_size: i64,
    current_tree_size: i64,
    proof_age_s: i64,
    max_proof_age_s: i64,
) -> KeyTransparencyProofInput {
    KeyTransparencyProofInput {
        inclusion,
        consistency,
        key_history,
        witness,
        require_witness,
        previous_tree_size,
        current_tree_size,
        proof_age_s,
        max_proof_age_s,
    }
}
