//! Production local encrypted store for Mercury (Phase 2).
//!
//! A pure-Rust, ACID, on-disk store that persists **only sealed ciphertext**
//! plus minimal metadata. Records are sealed by the audited
//! [`mercury_core::MercuryLocalStoreV1CryptoProvider`] (XChaCha20-Poly1305)
//! before they ever reach this crate; the store itself implements no crypto.
//!
//! Persistence is provided by [`redb`], a pure-Rust embedded key/value store
//! with copy-on-write B-trees and crash-safe commits — no C, SQLCipher, or
//! OpenSSL dependency. Because every redb transaction is atomic and durable, a
//! reopened database is always [`mercury_core::LocalStoreCrashRecoveryState::Clean`]
//! and the WAL-replay path is a no-op (see [`MercuryRedbStore::replay_wal`]).
//!
//! The device root key never lives in this crate except inside
//! [`MercuryKeychainAdapter`], where it is zeroized on drop and redacted from
//! `Debug`. In production the key is fetched from the OS keychain via
//! [`keyring`]; tests use the in-memory development backend.
//!
//! This crate plugs in **behind** the existing mercury-core gates: callers run
//! the keychain-unlock, unlock, and production-open gates (using
//! [`MercuryRedbStore::header_open_input`]) and the gate-checked
//! `*_production_local_store_*` free functions. Nothing here bypasses or
//! reimplements that policy.

#![forbid(unsafe_code)]

use core::fmt;

use mercury_core::{
    AcceptedLocalStoreProductionOpen, AcceptedLocalStoreWalReplay, AcceptedLocalStoreWrite,
    ComponentReasons, EncryptedLocalStoreAdapter, LocalStoreCrashRecoveryState, LocalStoreKeyScope,
    LocalStoreKeychainBackend, LocalStoreKeychainProtection, LocalStoreKeychainUnlockInput,
    LocalStorePayloadKind, LocalStoreProductionOpenInput, LocalStoreReadRecord,
    LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreSealingSuite,
    LocalStoreUnlockDatabaseHeaderState, LocalStoreUnlockInput, LocalStoreUnlockSecretState,
    MERCURY_LOCAL_STORE_VERSION, PolicyDecision, ProductionLocalStoreAdapter,
};
use redb::{Database, ReadableTable, Table, TableDefinition};
use zeroize::Zeroize;

/// Magic identifying a Mercury redb production store header.
const STORE_MAGIC: &[u8] = b"MERCURY_REDB_STORE_V1\0";

/// `DeviceLocal` key scope code, mirrored from mercury-core's encoding so the
/// on-disk manifest is self-describing.
const ROOT_KEY_SCOPE_DEVICE_LOCAL: i32 = 2;

const META_TABLE: TableDefinition<'_, &str, &[u8]> = TableDefinition::new("meta");
const RECORDS_TABLE: TableDefinition<'_, &[u8], &[u8]> = TableDefinition::new("records");

const META_KEY_MAGIC: &str = "magic";
const META_KEY_SUITE_CODE: &str = "suite_code";
const META_KEY_NONCE_LEN: &str = "nonce_len";
const META_KEY_TAG_LEN: &str = "tag_len";
const META_KEY_SEALED_KEY_SLOTS: &str = "sealed_key_slots";
const META_KEY_REQUIRED_KEY_SLOTS: &str = "required_key_slots";
const META_KEY_PLAINTEXT_KEY_SLOTS: &str = "plaintext_key_slots";
const META_KEY_ROOT_KEY_SCOPE: &str = "root_key_scope";
const META_KEY_ROOT_KEY_GENERATION: &str = "root_key_generation";
const META_KEY_CRASH_RECOVERY: &str = "crash_recovery";

/// Parameters written into a store's header manifest at creation time.
///
/// These are the values the mercury-core production-open gate validates. The
/// defaults from [`HeaderParams::v1`] satisfy that gate; tests deliberately
/// construct malformed headers to confirm the gate rejects them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderParams {
    pub suite_code: i32,
    pub nonce_len: i32,
    pub tag_len: i32,
    pub sealed_key_slots: i32,
    pub required_key_slots: i32,
    pub plaintext_key_slots: i32,
    pub root_key_scope_code: i32,
    pub root_key_generation: i32,
}

impl HeaderParams {
    /// The canonical `MercuryLocalStoreV1` header: one sealed device-local key
    /// slot, no plaintext slots, generation 1. Accepted by the open gate.
    pub fn v1() -> Self {
        let suite = LocalStoreSealingSuite::MercuryLocalStoreV1;
        Self {
            suite_code: suite.code(),
            nonce_len: suite.nonce_len(),
            tag_len: suite.authentication_tag_len(),
            sealed_key_slots: 1,
            required_key_slots: 1,
            plaintext_key_slots: 0,
            root_key_scope_code: ROOT_KEY_SCOPE_DEVICE_LOCAL,
            root_key_generation: 1,
        }
    }
}

impl Default for HeaderParams {
    fn default() -> Self {
        Self::v1()
    }
}

/// Errors from the redb-backed production store.
///
/// `Debug` and `Display` never include record bytes, payload bytes, or key
/// material — only structural descriptions and the underlying redb error text.
#[derive(Debug)]
pub enum StoreError {
    /// A redb transaction, table, storage, commit, or open error. Boxed because
    /// `redb::Error` is large and would otherwise bloat every `Result`.
    Database(Box<redb::Error>),
    /// The on-disk header manifest is missing a field, malformed, or its magic
    /// does not match — the file is not a valid Mercury redb store.
    Corruption(&'static str),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Database(error) => write!(f, "local store database error: {error}"),
            StoreError::Corruption(detail) => write!(f, "local store corruption: {detail}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Database(error) => Some(error.as_ref()),
            StoreError::Corruption(_) => None,
        }
    }
}

macro_rules! redb_from {
    ($($ty:path),+ $(,)?) => {
        $(impl From<$ty> for StoreError {
            fn from(error: $ty) -> Self {
                StoreError::Database(Box::new(error.into()))
            }
        })+
    };
}

redb_from!(
    redb::Error,
    redb::DatabaseError,
    redb::TransactionError,
    redb::TableError,
    redb::StorageError,
    redb::CommitError,
);

/// A pure-Rust, ACID, on-disk Mercury local store backed by [`redb`].
///
/// Persists only sealed ciphertext and minimal metadata. Implements both
/// [`EncryptedLocalStoreAdapter`] and [`ProductionLocalStoreAdapter`] so the
/// gate-checked mercury-core free functions drive it.
#[derive(Debug)]
pub struct MercuryRedbStore {
    db: Database,
    header: HeaderParams,
}

impl MercuryRedbStore {
    /// Open an existing store at `path`, or create one whose header manifest is
    /// written from `header`. On an existing file the on-disk manifest wins and
    /// `header` is ignored, so a caller cannot silently reinterpret a store.
    pub fn open_or_create(
        path: impl AsRef<std::path::Path>,
        header: HeaderParams,
    ) -> Result<Self, StoreError> {
        let db = Database::create(path)?;

        // Ensure both tables exist and the header manifest is present. On an
        // already-initialized store this only opens the tables.
        let write = db.begin_write()?;
        {
            let _records = write.open_table(RECORDS_TABLE)?;
            let mut meta = write.open_table(META_TABLE)?;
            if meta.get(META_KEY_MAGIC)?.is_none() {
                write_header(&mut meta, header)?;
            }
        }
        write.commit()?;

        let header = read_header(&db)?;
        Ok(Self { db, header })
    }

    /// The header manifest persisted in this store.
    pub fn header(&self) -> HeaderParams {
        self.header
    }

    /// Build the production-open gate input from the persisted manifest and a
    /// caller-supplied unlock input (typically the one a keychain adapter
    /// produced). Run [`LocalStoreProductionOpenInput::evaluate`] on the result;
    /// it reports `crash_recovery = Clean` because redb commits are atomic.
    pub fn header_open_input(
        &self,
        unlock: LocalStoreUnlockInput,
    ) -> LocalStoreProductionOpenInput {
        LocalStoreProductionOpenInput {
            unlock,
            header_magic_matches: true,
            header_suite_code: self.header.suite_code,
            header_nonce_len: self.header.nonce_len,
            header_tag_len: self.header.tag_len,
            required_key_slots: self.header.required_key_slots,
            sealed_key_slots: self.header.sealed_key_slots,
            plaintext_key_slots: self.header.plaintext_key_slots,
            root_key_scope: LocalStoreKeyScope::DeviceLocal,
            root_key_generation: self.header.root_key_generation,
            crash_recovery: LocalStoreCrashRecoveryState::Clean,
        }
    }
}

impl EncryptedLocalStoreAdapter for MercuryRedbStore {
    type Error = StoreError;

    fn put_accepted_record(
        &mut self,
        write: AcceptedLocalStoreWrite<'_>,
    ) -> Result<(), Self::Error> {
        let request = write.request();
        let key = record_key(request.locator);
        let value = encode_record(
            request.record_kind,
            request.payload.kind(),
            request.payload.bytes(),
            request.policy_decision,
        );

        let txn = self.db.begin_write()?;
        {
            let mut records = txn.open_table(RECORDS_TABLE)?;
            records.insert(key.as_slice(), value.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    fn delete_record(&mut self, locator: LocalStoreRecordLocator<'_>) -> Result<(), Self::Error> {
        let key = record_key(locator);
        let txn = self.db.begin_write()?;
        {
            let mut records = txn.open_table(RECORDS_TABLE)?;
            records.remove(key.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }
}

impl ProductionLocalStoreAdapter for MercuryRedbStore {
    fn open_database(
        &mut self,
        _open: AcceptedLocalStoreProductionOpen,
    ) -> Result<(), Self::Error> {
        // The database file is already open (opened/created in `open_or_create`
        // before the gate runs). Reaching here means the production-open gate
        // accepted the header, so there is nothing further to unlock.
        Ok(())
    }

    fn replay_wal(&mut self, _replay: AcceptedLocalStoreWalReplay) -> Result<(), Self::Error> {
        // redb commits are atomic and durable: a successfully reopened database
        // is always crash-consistent, so the manifest reports `Clean` and the
        // gate never asks to replay. This no-op exists only to satisfy the
        // trait for the (unreachable, for this backend) replay branch.
        Ok(())
    }

    fn read_record(
        &self,
        locator: LocalStoreRecordLocator<'_>,
    ) -> Result<Option<LocalStoreReadRecord>, Self::Error> {
        let key = record_key(locator);
        let txn = self.db.begin_read()?;
        let records = txn.open_table(RECORDS_TABLE)?;
        let Some(value) = records.get(key.as_slice())? else {
            return Ok(None);
        };
        let (record_kind, payload_kind, bytes, _policy) = decode_record(value.value())?;
        Ok(Some(LocalStoreReadRecord::new(
            locator.namespace,
            locator.record_id,
            record_kind,
            payload_kind,
            bytes,
        )))
    }
}

/// Canonical record key: length-prefixed namespace followed by record id, so
/// distinct (namespace, record_id) pairs never collide.
fn record_key(locator: LocalStoreRecordLocator<'_>) -> Vec<u8> {
    let namespace = locator.namespace.as_bytes();
    let record_id = locator.record_id.as_bytes();
    let mut key = Vec::with_capacity(8 + namespace.len() + record_id.len());
    key.extend_from_slice(&(namespace.len() as u32).to_le_bytes());
    key.extend_from_slice(namespace);
    key.extend_from_slice(&(record_id.len() as u32).to_le_bytes());
    key.extend_from_slice(record_id);
    key
}

/// Encode a record value: record-kind code, payload-kind code, optional policy
/// decision, then the (sealed) payload bytes. For sealed payloads these bytes
/// are ciphertext — plaintext is never written.
fn encode_record(
    record_kind: LocalStoreRecordKind,
    payload_kind: LocalStorePayloadKind,
    bytes: &[u8],
    policy: Option<PolicyDecision>,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(16 + bytes.len());
    out.extend_from_slice(&record_kind.code().to_le_bytes());
    out.extend_from_slice(&payload_kind.code().to_le_bytes());
    encode_policy_decision(&mut out, policy);
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(bytes);
    out
}

fn encode_policy_decision(out: &mut Vec<u8>, policy: Option<PolicyDecision>) {
    match policy {
        Some(decision) => {
            out.extend_from_slice(&1i32.to_le_bytes());
            out.extend_from_slice(&i32::from(decision.accepted).to_le_bytes());
            out.extend_from_slice(&decision.reason_code.to_le_bytes());
            out.extend_from_slice(&decision.audit_class.to_le_bytes());
            out.extend_from_slice(&decision.components.envelope_reason.to_le_bytes());
            out.extend_from_slice(&decision.components.room_epoch_reason.to_le_bytes());
            out.extend_from_slice(&decision.components.ai_grant_reason.to_le_bytes());
            out.extend_from_slice(&decision.components.ai_lifecycle_reason.to_le_bytes());
        }
        None => out.extend_from_slice(&0i32.to_le_bytes()),
    }
}

#[allow(clippy::type_complexity)]
fn decode_record(
    bytes: &[u8],
) -> Result<
    (
        LocalStoreRecordKind,
        LocalStorePayloadKind,
        Vec<u8>,
        Option<PolicyDecision>,
    ),
    StoreError,
> {
    let mut cursor = Cursor::new(bytes);
    let record_kind = LocalStoreRecordKind::from_code(cursor.read_i32()?)
        .ok_or(StoreError::Corruption("unknown record kind"))?;
    let payload_kind = LocalStorePayloadKind::from_code(cursor.read_i32()?)
        .ok_or(StoreError::Corruption("unknown payload kind"))?;
    let policy = decode_policy_decision(&mut cursor)?;
    let payload_len = cursor.read_u64()?;
    let payload_len =
        usize::try_from(payload_len).map_err(|_| StoreError::Corruption("payload too large"))?;
    let payload = cursor.read_bytes(payload_len)?.to_vec();
    Ok((record_kind, payload_kind, payload, policy))
}

fn decode_policy_decision(cursor: &mut Cursor<'_>) -> Result<Option<PolicyDecision>, StoreError> {
    match cursor.read_i32()? {
        0 => Ok(None),
        1 => {
            let accepted = cursor.read_i32()? != 0;
            let reason_code = cursor.read_i32()?;
            let audit_class = cursor.read_i32()?;
            let envelope_reason = cursor.read_i32()?;
            let room_epoch_reason = cursor.read_i32()?;
            let ai_grant_reason = cursor.read_i32()?;
            let ai_lifecycle_reason = cursor.read_i32()?;
            Ok(Some(PolicyDecision {
                accepted,
                reason_code,
                audit_class,
                components: ComponentReasons {
                    envelope_reason,
                    room_epoch_reason,
                    ai_grant_reason,
                    ai_lifecycle_reason,
                },
            }))
        }
        _ => Err(StoreError::Corruption("invalid policy-decision flag")),
    }
}

fn write_header(meta: &mut Table<'_, &str, &[u8]>, header: HeaderParams) -> Result<(), StoreError> {
    meta.insert(META_KEY_MAGIC, STORE_MAGIC)?;
    insert_meta_i32(meta, META_KEY_SUITE_CODE, header.suite_code)?;
    insert_meta_i32(meta, META_KEY_NONCE_LEN, header.nonce_len)?;
    insert_meta_i32(meta, META_KEY_TAG_LEN, header.tag_len)?;
    insert_meta_i32(meta, META_KEY_SEALED_KEY_SLOTS, header.sealed_key_slots)?;
    insert_meta_i32(meta, META_KEY_REQUIRED_KEY_SLOTS, header.required_key_slots)?;
    insert_meta_i32(
        meta,
        META_KEY_PLAINTEXT_KEY_SLOTS,
        header.plaintext_key_slots,
    )?;
    insert_meta_i32(meta, META_KEY_ROOT_KEY_SCOPE, header.root_key_scope_code)?;
    insert_meta_i32(
        meta,
        META_KEY_ROOT_KEY_GENERATION,
        header.root_key_generation,
    )?;
    // Clean by construction: redb has no separate WAL to be dirty against.
    insert_meta_i32(
        meta,
        META_KEY_CRASH_RECOVERY,
        LocalStoreCrashRecoveryState::Clean.code(),
    )?;
    Ok(())
}

fn insert_meta_i32(
    meta: &mut Table<'_, &str, &[u8]>,
    key: &'static str,
    value: i32,
) -> Result<(), StoreError> {
    meta.insert(key, value.to_le_bytes().as_slice())?;
    Ok(())
}

fn read_header(db: &Database) -> Result<HeaderParams, StoreError> {
    let txn = db.begin_read()?;
    let meta = txn.open_table(META_TABLE)?;

    let magic = meta
        .get(META_KEY_MAGIC)?
        .ok_or(StoreError::Corruption("missing header magic"))?;
    if magic.value() != STORE_MAGIC {
        return Err(StoreError::Corruption("header magic mismatch"));
    }

    Ok(HeaderParams {
        suite_code: read_meta_i32(&meta, META_KEY_SUITE_CODE)?,
        nonce_len: read_meta_i32(&meta, META_KEY_NONCE_LEN)?,
        tag_len: read_meta_i32(&meta, META_KEY_TAG_LEN)?,
        sealed_key_slots: read_meta_i32(&meta, META_KEY_SEALED_KEY_SLOTS)?,
        required_key_slots: read_meta_i32(&meta, META_KEY_REQUIRED_KEY_SLOTS)?,
        plaintext_key_slots: read_meta_i32(&meta, META_KEY_PLAINTEXT_KEY_SLOTS)?,
        root_key_scope_code: read_meta_i32(&meta, META_KEY_ROOT_KEY_SCOPE)?,
        root_key_generation: read_meta_i32(&meta, META_KEY_ROOT_KEY_GENERATION)?,
    })
}

fn read_meta_i32(
    meta: &impl ReadableTable<&'static str, &'static [u8]>,
    key: &'static str,
) -> Result<i32, StoreError> {
    let guard = meta
        .get(key)?
        .ok_or(StoreError::Corruption("missing header field"))?;
    let bytes: [u8; 4] = guard
        .value()
        .try_into()
        .map_err(|_| StoreError::Corruption("malformed header field"))?;
    Ok(i32::from_le_bytes(bytes))
}

/// Minimal little-endian cursor over a record byte slice.
struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], StoreError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(StoreError::Corruption("record length overflow"))?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or(StoreError::Corruption("truncated record"))?;
        self.offset = end;
        Ok(slice)
    }

    fn read_i32(&mut self) -> Result<i32, StoreError> {
        let bytes: [u8; 4] = self
            .read_bytes(4)?
            .try_into()
            .map_err(|_| StoreError::Corruption("truncated i32"))?;
        Ok(i32::from_le_bytes(bytes))
    }

    fn read_u64(&mut self) -> Result<u64, StoreError> {
        let bytes: [u8; 8] = self
            .read_bytes(8)?
            .try_into()
            .map_err(|_| StoreError::Corruption("truncated u64"))?;
        Ok(u64::from_le_bytes(bytes))
    }
}

/// Source of the 32-byte device root key for a [`MercuryKeychainAdapter`].
enum KeySource {
    /// Production: fetch/store the sealed key in the OS keychain.
    Os { service: String, account: String },
    /// Development/test: an in-memory key, never touching the OS keychain.
    DevMemory([u8; 32]),
}

impl Drop for KeySource {
    fn drop(&mut self) {
        if let KeySource::DevMemory(key) = self {
            key.zeroize();
        }
    }
}

impl fmt::Debug for KeySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeySource::Os { service, account } => f
                .debug_struct("Os")
                .field("service", service)
                .field("account", account)
                .finish(),
            // Never print the in-memory key material.
            KeySource::DevMemory(_) => f
                .debug_struct("DevMemory")
                .field("key", &"<redacted>")
                .finish(),
        }
    }
}

/// Adapter that unlocks a Mercury local store with an OS keychain (production)
/// or an in-memory key (development/tests).
///
/// Produces a [`LocalStoreKeychainUnlockInput`] for the mercury-core
/// keychain-unlock gate with the per-OS backend and `OsProtected` /
/// non-exportable / sealed-secret posture the gate requires.
#[derive(Debug)]
pub struct MercuryKeychainAdapter {
    source: KeySource,
}

impl MercuryKeychainAdapter {
    /// Production adapter backed by the OS keychain identified by
    /// (`service`, `account`). The backend is selected for the current target
    /// OS; key access happens in [`Self::root_key`] / [`Self::provision_root_key`].
    pub fn os(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            source: KeySource::Os {
                service: service.into(),
                account: account.into(),
            },
        }
    }

    /// Development/test adapter with an in-memory root key. Uses the
    /// `DevelopmentMemory` backend and sets `allow_development_backend = true`
    /// so the gate accepts it. Never touches the OS keychain.
    pub fn dev_memory(sealed_key: [u8; 32]) -> Self {
        Self {
            source: KeySource::DevMemory(sealed_key),
        }
    }

    const fn is_development(&self) -> bool {
        matches!(self.source, KeySource::DevMemory(_))
    }

    /// The keychain backend for this adapter: `DevelopmentMemory` for the dev
    /// adapter, otherwise the credential store native to the build target.
    pub const fn backend(&self) -> LocalStoreKeychainBackend {
        if self.is_development() {
            LocalStoreKeychainBackend::DevelopmentMemory
        } else if cfg!(target_os = "ios") {
            LocalStoreKeychainBackend::IosKeychain
        } else if cfg!(target_os = "android") {
            LocalStoreKeychainBackend::AndroidKeystore
        } else if cfg!(target_os = "macos") {
            LocalStoreKeychainBackend::MacosKeychain
        } else if cfg!(target_os = "windows") {
            LocalStoreKeychainBackend::WindowsCredentialVault
        } else {
            LocalStoreKeychainBackend::LinuxSecretService
        }
    }

    /// Build the keychain-unlock gate input. Run
    /// [`LocalStoreKeychainUnlockInput::evaluate`] on the result; its
    /// `unlock_input` field then feeds the unlock and production-open gates.
    pub const fn keychain_unlock_input(&self) -> LocalStoreKeychainUnlockInput {
        let development = self.is_development();
        LocalStoreKeychainUnlockInput {
            store_version: MERCURY_LOCAL_STORE_VERSION,
            backend: self.backend(),
            backend_available: true,
            protection: if development {
                LocalStoreKeychainProtection::DevelopmentOnly
            } else {
                LocalStoreKeychainProtection::OsProtected
            },
            allow_development_backend: development,
            user_auth_required: false,
            user_auth_satisfied: false,
            device_secret: LocalStoreUnlockSecretState::PresentSealed,
            device_secret_exportable: false,
            database_header: LocalStoreUnlockDatabaseHeaderState::Authenticated,
            recovery_required: false,
            plaintext_cache_records: 0,
        }
    }

    /// Fetch the 32-byte device root key.
    ///
    /// For the OS adapter the key is read from the keychain. A genuinely-absent key returns
    /// [`KeychainError::NotFound`] (the first-run signal — see [`Self::provision_root_key`]); this
    /// is DISTINCT from a transient backend failure ([`KeychainError::Keyring`]: vault locked,
    /// access denied, Secret Service not up at login). The caller MUST treat them differently —
    /// minting a replacement key on a transient error would orphan the snapshot sealed under the
    /// real key and irreversibly lose the account. For the dev adapter the in-memory key is
    /// returned. Feed the result straight into
    /// [`mercury_core::MercuryLocalStoreV1CryptoProvider::new`], which zeroizes it on drop.
    pub fn root_key(&self) -> Result<[u8; 32], KeychainError> {
        match &self.source {
            KeySource::DevMemory(key) => Ok(*key),
            KeySource::Os { service, account } => {
                let entry =
                    keyring::Entry::new(service, account).map_err(KeychainError::Keyring)?;
                let secret = entry.get_secret().map_err(|e| match e {
                    keyring::Error::NoEntry => KeychainError::NotFound,
                    other => KeychainError::Keyring(other),
                })?;
                secret
                    .as_slice()
                    .try_into()
                    .map_err(|_| KeychainError::MalformedSecret)
            }
        }
    }

    /// Store a caller-supplied 32-byte device root key in the OS keychain
    /// (first-run provisioning). No-op for the dev adapter. The key bytes are
    /// passed straight to the OS keychain and never persisted by this crate.
    ///
    /// Refuses to overwrite an existing key: provisioning is strictly first-run, and clobbering a
    /// present key would orphan any snapshot sealed under it. A pre-existing entry therefore returns
    /// [`KeychainError::AlreadyProvisioned`] (belt-and-suspenders alongside the caller's own
    /// `NotFound`-gated provisioning).
    pub fn provision_root_key(&self, key: &[u8; 32]) -> Result<(), KeychainError> {
        match &self.source {
            KeySource::DevMemory(_) => Ok(()),
            KeySource::Os { service, account } => {
                let entry =
                    keyring::Entry::new(service, account).map_err(KeychainError::Keyring)?;
                match entry.get_secret() {
                    Ok(_) => Err(KeychainError::AlreadyProvisioned),
                    Err(keyring::Error::NoEntry) => {
                        entry.set_secret(key).map_err(KeychainError::Keyring)
                    }
                    Err(other) => Err(KeychainError::Keyring(other)),
                }
            }
        }
    }

    /// Permanently delete the device root key from the OS keychain — account erasure ("delete my
    /// data"). No-op for the dev adapter. IDEMPOTENT: an already-absent key is success, since the
    /// goal state is simply "no key here". After this returns Ok, any snapshot that was sealed under
    /// the key is cryptographically unopenable, so the caller MUST also delete the snapshot file;
    /// the two together leave nothing recoverable on the device.
    pub fn delete_root_key(&self) -> Result<(), KeychainError> {
        match &self.source {
            KeySource::DevMemory(_) => Ok(()),
            KeySource::Os { service, account } => {
                let entry =
                    keyring::Entry::new(service, account).map_err(KeychainError::Keyring)?;
                match entry.delete_credential() {
                    Ok(()) => Ok(()),
                    // Already gone — erasure is idempotent; the desired end state holds.
                    Err(keyring::Error::NoEntry) => Ok(()),
                    Err(other) => Err(KeychainError::Keyring(other)),
                }
            }
        }
    }
}

/// Errors from OS-keychain key access. Never includes key bytes.
#[derive(Debug)]
pub enum KeychainError {
    /// No device key has been provisioned yet (first run). DISTINCT from [`Self::Keyring`] so a
    /// caller never mistakes "absent" for a transient "unavailable" (or vice-versa) and overwrites
    /// a real key — only this variant authorizes first-run provisioning.
    NotFound,
    /// A device key is already provisioned; provisioning again is refused because it would orphan
    /// the snapshot sealed under the existing key.
    AlreadyProvisioned,
    /// The underlying `keyring` backend failed transiently (locked, denied, unavailable, ...).
    /// Does NOT include the absent case — that is [`Self::NotFound`].
    Keyring(keyring::Error),
    /// The stored secret was not exactly 32 bytes.
    MalformedSecret,
}

impl fmt::Display for KeychainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeychainError::NotFound => f.write_str("no device key provisioned"),
            KeychainError::AlreadyProvisioned => f.write_str("a device key is already provisioned"),
            KeychainError::Keyring(error) => write!(f, "keychain backend error: {error}"),
            KeychainError::MalformedSecret => f.write_str("stored device key is not 32 bytes"),
        }
    }
}

impl std::error::Error for KeychainError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KeychainError::Keyring(error) => Some(error),
            KeychainError::NotFound
            | KeychainError::AlreadyProvisioned
            | KeychainError::MalformedSecret => None,
        }
    }
}
