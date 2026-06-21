//! Secure-backup engine for Mercury (Stage 9).
//!
//! Encrypts a backup archive under a key derived from a high-entropy recovery
//! code via **Argon2id**, and PRODUCES the inputs for mercury-core's backup /
//! recovery policy gates (`evaluate_account_recovery` ->
//! `evaluate_secure_backup_restore`). The engine does the crypto; the gates
//! decide policy -- mirroring how `mercury-mls` feeds the MLS gates.
//!
//! The recovery code is a fresh 192-bit CSPRNG secret the user keeps out of band;
//! the derived key encrypts the archive with XChaCha20-Poly1305 (BLAKE3 manifest
//! and a key-identity digest). Deployment/topology facts the engine cannot prove
//! cryptographically (recovery-service authentication + rate-limiting, OS-level
//! plaintext-backup exclusion, opaque account identifiers for server transports)
//! are supplied as an explicit [`BackupDeploymentAttestation`] -- never fabricated
//! -- following the same honest pattern as the MLS adapter-selection attestation.
//!
//! Only audited RustCrypto / BLAKE3 primitives are used; nothing is hand-rolled.
//! The recovery code and derived key zeroize on drop and `BackupError` carries no
//! secret material.

#![forbid(unsafe_code)]

use core::fmt;

use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, OsRng, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use subtle::ConstantTimeEq;
use zeroize::{ZeroizeOnDrop, Zeroizing};

pub use mercury_core::{
    AccountRecoveryDecision, SecureBackupRestoreDecision, SecureBackupRestoreEnvelopeSuite,
    SecureBackupRestoreInput, SecureBackupRestoreScope, SecureBackupRestoreTransport,
    evaluate_account_recovery, evaluate_secure_backup_restore,
};
use mercury_core::{AccountRecoveryInput, AccountRecoveryMethod};

/// Domain separation for the backup manifest / AEAD associated data.
const DOMAIN: &[u8] = b"mercury-secure-backup-v1";
const MAGIC: [u8; 4] = *b"MBAK";
const VERSION_V1: u8 = 1;
const VERSION_V2: u8 = 2;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const DIGEST_LEN: usize = 32;
const KDF_METADATA_LEN: usize = 1 + 4 + 4 + 4;
/// v1 header: magic(4)+version(1)+salt(16)+nonce(24)+key_digest(32)+audit_digest(32).
const HEADER_V1_LEN: usize = 4 + 1 + SALT_LEN + NONCE_LEN + DIGEST_LEN + DIGEST_LEN;
/// v2 header: v1 header plus algorithm(1)+memory_mib(u32)+iterations(u32)+parallelism(u32).
const HEADER_V2_LEN: usize = HEADER_V1_LEN + KDF_METADATA_LEN;
const TAG_LEN: usize = 16;

/// Recovery-code entropy: a fresh 24-byte (192-bit) CSPRNG secret, matching the
/// gate's standard-account minimum (`backup_key_entropy_bits >= 192`).
const RECOVERY_CODE_LEN: usize = 24;

/// Argon2id parameters, chosen to satisfy the gate's KDF floor
/// (`kdf_memory_cost_mib >= 64`, `kdf_iterations >= 3`).
const KDF_MEMORY_MIB: i32 = 64;
const KDF_ITERATIONS: u32 = 3;
const KDF_PARALLELISM: u32 = 1;

/// The Argon2id floor an archive's OWN KDF profile must meet for [`open_backup`] to touch it — the
/// same floor the restore gate (`mercury_core::evaluate_secure_backup_restore`) enforces. A weaker
/// profile is brute-forceable, so a self-consistent-but-weak archive is REFUSED rather than silently
/// opened. The honest producer always meets it (`create_backup` hard-codes `CURRENT_V2`); only a
/// forked / downgraded producer's archive trips this.
const MIN_KDF_MEMORY_MIB: u32 = 64;
const MIN_KDF_ITERATIONS: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum BackupKdfAlgorithm {
    Argon2id = 1,
}

impl BackupKdfAlgorithm {
    const fn code(self) -> u8 {
        self as u8
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Argon2id => "argon2id",
        }
    }

    fn from_code(code: u8) -> Result<Self, BackupError> {
        match code {
            1 => Ok(Self::Argon2id),
            _ => Err(BackupError::Malformed),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct BackupKdfParams {
    algorithm: BackupKdfAlgorithm,
    memory_mib: u32,
    iterations: u32,
    parallelism: u32,
}

impl BackupKdfParams {
    const LEGACY_V1: Self = Self {
        algorithm: BackupKdfAlgorithm::Argon2id,
        memory_mib: KDF_MEMORY_MIB as u32,
        iterations: KDF_ITERATIONS,
        parallelism: KDF_PARALLELISM,
    };

    const CURRENT_V2: Self = Self::LEGACY_V1;

    fn memory_kib(self) -> Result<u32, BackupError> {
        self.memory_mib
            .checked_mul(1024)
            .ok_or(BackupError::KeyDerivation)
    }

    fn append_to(self, out: &mut Vec<u8>) {
        out.push(self.algorithm.code());
        out.extend_from_slice(&self.memory_mib.to_le_bytes());
        out.extend_from_slice(&self.iterations.to_le_bytes());
        out.extend_from_slice(&self.parallelism.to_le_bytes());
    }

    fn read_from(bytes: &[u8]) -> Result<Self, BackupError> {
        if bytes.len() < KDF_METADATA_LEN {
            return Err(BackupError::Malformed);
        }
        Ok(Self {
            algorithm: BackupKdfAlgorithm::from_code(bytes[0])?,
            memory_mib: u32::from_le_bytes(
                bytes[1..5].try_into().map_err(|_| BackupError::Malformed)?,
            ),
            iterations: u32::from_le_bytes(
                bytes[5..9].try_into().map_err(|_| BackupError::Malformed)?,
            ),
            parallelism: u32::from_le_bytes(
                bytes[9..13]
                    .try_into()
                    .map_err(|_| BackupError::Malformed)?,
            ),
        })
    }
}

/// Restore-planning status for a sealed backup's encoded KDF profile.
///
/// This is deliberately separate from the backup/restore gate:
/// - the gate answers whether a profile is acceptable to use at all
/// - this status answers whether a successfully opened archive should be
///   re-sealed under Mercury's current self-describing profile afterwards
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackupKdfMigrationStatus {
    /// The archive already carries Mercury's current self-describing KDF
    /// profile and does not need a post-restore rewrite.
    CurrentProfile,
    /// The archive can still be restored, but it should be re-sealed under the
    /// current profile afterwards because its KDF metadata is legacy or does
    /// not match Mercury's current defaults.
    ResealRecommended,
}

impl BackupKdfMigrationStatus {
    pub const fn code(self) -> i32 {
        match self {
            Self::CurrentProfile => 1,
            Self::ResealRecommended => 2,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::CurrentProfile => "current_profile",
            Self::ResealRecommended => "reseal_recommended",
        }
    }

    pub const fn recommends_reseal(self) -> bool {
        matches!(self, Self::ResealRecommended)
    }
}

/// Non-secret audit summary for a restore-planning decision.
///
/// This intentionally carries only metadata already visible in the sealed
/// backup header plus the restore gate outcome. It never includes recovery-code-
/// derived material, plaintext archive bytes, or derived backup keys.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BackupRestoreAuditRecord {
    pub sealed_backup_version: u8,
    pub kdf_algorithm_code: u8,
    pub kdf_algorithm_label: &'static str,
    pub kdf_memory_mib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub stores_explicit_kdf_profile: bool,
    pub uses_current_kdf_profile: bool,
    pub kdf_migration_status_code: i32,
    pub kdf_migration_status_label: &'static str,
    pub accepted: bool,
    pub reason_code: i32,
    pub reason_label: &'static str,
}

/// Restore-planning output derived from a sealed archive plus the current
/// deployment attestation.
///
/// This lets a real restore flow re-evaluate the backup gate from the archive's
/// encoded KDF profile instead of depending on the creation-time gate input.
pub struct BackupRestorePlan {
    pub restore_input: SecureBackupRestoreInput,
    pub restore_decision: SecureBackupRestoreDecision,
    pub kdf_migration_status: BackupKdfMigrationStatus,
    pub audit_record: BackupRestoreAuditRecord,
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut out = [0u8; N];
    OsRng.fill_bytes(&mut out);
    out
}

/// A fresh high-entropy recovery code (192 bits). The user keeps it out of band;
/// it is the ONLY way to derive the backup key. Zeroized on drop; `Debug` redacted.
#[derive(ZeroizeOnDrop)]
pub struct RecoveryCode([u8; RECOVERY_CODE_LEN]);

impl RecoveryCode {
    /// The recovery code's CSPRNG entropy in bits (reported to the gate).
    pub const ENTROPY_BITS: i32 = (RECOVERY_CODE_LEN as i32) * 8;

    /// Generate a fresh random recovery code from the system CSPRNG.
    pub fn generate() -> Self {
        Self(random_bytes::<RECOVERY_CODE_LEN>())
    }

    /// Reconstruct a recovery code from its raw bytes (as the user re-entered it).
    pub fn from_bytes(bytes: [u8; RECOVERY_CODE_LEN]) -> Self {
        Self(bytes)
    }

    /// The raw recovery-code bytes (to render for the user). Handle with care — returned in a
    /// `Zeroizing` wrapper so the rendered copy is wiped when the caller drops it.
    pub fn to_bytes(&self) -> Zeroizing<[u8; RECOVERY_CODE_LEN]> {
        Zeroizing::new(self.0)
    }
}

impl fmt::Debug for RecoveryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryCode(<redacted>)")
    }
}

/// Deployment / topology facts the backup engine cannot prove cryptographically.
/// The operator supplies its REAL status (the engine never fabricates these -- a
/// false attestation makes the gate honestly refuse the backup):
/// - the recovery SERVICE authenticates the recovering user and rate-limits
///   attempts (required by the account-recovery gate, anti-brute-force);
/// - the OS excludes the backup blob from OS-level plaintext backups (iOS Data
///   Protection / Android `allowBackup=false`);
/// - for server-backed transports, the account identifier stored with the backup
///   is opaque (no plaintext account id);
/// - the client's RESTORE flow rotates the device secret on restore (a
///   client-flow behavior the backup engine itself does not perform -- restoring
///   a backup recovers the old secrets, and the client then mints a fresh device
///   key so a compromised old device does not carry over);
/// - threshold/hardware fields, used only for those transports.
#[derive(Debug, Clone, Copy)]
pub struct BackupDeploymentAttestation {
    pub server_authenticated: bool,
    pub server_rate_limited: bool,
    pub os_plaintext_backup_excluded: bool,
    pub opaque_account_identifier: bool,
    pub restore_rotates_device_secret: bool,
    pub device_approval_present: bool,
    pub threshold_shares: i32,
    pub threshold_required: i32,
    pub threshold_approvals: i32,
}

impl BackupDeploymentAttestation {
    /// A standard fully-attested deployment for a USER-held recovery-key archive:
    /// the recovery service authenticates + rate-limits, the OS excludes the blob
    /// from plaintext backup, and the account identifier is opaque. Threshold /
    /// hardware fields are left zero/false (unused for that transport). Use only
    /// when these facts genuinely hold.
    pub const fn user_recovery_archive() -> Self {
        Self {
            server_authenticated: true,
            server_rate_limited: true,
            os_plaintext_backup_excluded: true,
            opaque_account_identifier: true,
            restore_rotates_device_secret: true,
            device_approval_present: false,
            threshold_shares: 0,
            threshold_required: 0,
            threshold_approvals: 0,
        }
    }
}

/// Errors from creating/opening a secure backup. Carries no secret material.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackupError {
    /// Argon2id key derivation failed (bad parameters).
    KeyDerivation,
    /// AEAD sealing of the archive failed.
    Seal,
    /// The sealed backup blob is malformed (bad magic/version, too short, or
    /// trailing/!exact framing).
    Malformed,
    /// The recovery code did not derive the key this backup was sealed under.
    WrongRecoveryCode,
    /// AEAD opening failed: wrong key, tampering, or corruption.
    OpenFailed,
    /// The archive's own KDF profile is below the Argon2id floor (memory < 64 MiB or iterations < 3),
    /// so it is brute-forceable; refused even though its manifest is internally consistent.
    WeakKdf,
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackupError::KeyDerivation => f.write_str("backup key derivation failed"),
            BackupError::Seal => f.write_str("backup sealing failed"),
            BackupError::Malformed => f.write_str("malformed sealed backup"),
            BackupError::WrongRecoveryCode => f.write_str("wrong recovery code"),
            BackupError::OpenFailed => f.write_str("backup opening failed"),
            BackupError::WeakKdf => {
                f.write_str("backup KDF profile is below the minimum work factor")
            }
        }
    }
}

impl std::error::Error for BackupError {}

/// The encrypted backup blob (to persist to disk / upload to the store) plus the
/// public framing needed to re-derive + verify the key on restore.
#[derive(Clone, PartialEq, Eq)]
pub struct SealedBackup {
    version: u8,
    kdf: BackupKdfParams,
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
    backup_key_digest: [u8; DIGEST_LEN],
    audit_digest: [u8; DIGEST_LEN],
    sealed_archive: Vec<u8>,
}

impl SealedBackup {
    /// Canonical on-disk / on-store bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let header_len = match self.version {
            VERSION_V1 => HEADER_V1_LEN,
            VERSION_V2 => HEADER_V2_LEN,
            _ => HEADER_V2_LEN,
        };
        let mut out = Vec::with_capacity(header_len + self.sealed_archive.len());
        out.extend_from_slice(&MAGIC);
        out.push(self.version);
        if self.version == VERSION_V2 {
            self.kdf.append_to(&mut out);
        }
        out.extend_from_slice(&self.salt);
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&self.backup_key_digest);
        out.extend_from_slice(&self.audit_digest);
        out.extend_from_slice(&self.sealed_archive);
        out
    }

    /// Parse a sealed backup from its canonical bytes. Fails closed on a bad
    /// magic/version or short framing (no panic -- the length guard precedes all
    /// slicing).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BackupError> {
        if bytes.len() < HEADER_V1_LEN + TAG_LEN {
            return Err(BackupError::Malformed);
        }
        if bytes[..4] != MAGIC {
            return Err(BackupError::Malformed);
        }
        let version = bytes[4];
        let (kdf, mut off) = match version {
            VERSION_V1 => (BackupKdfParams::LEGACY_V1, 5),
            VERSION_V2 => {
                if bytes.len() < HEADER_V2_LEN + TAG_LEN {
                    return Err(BackupError::Malformed);
                }
                (
                    BackupKdfParams::read_from(&bytes[5..5 + KDF_METADATA_LEN])?,
                    5 + KDF_METADATA_LEN,
                )
            }
            _ => return Err(BackupError::Malformed),
        };
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&bytes[off..off + SALT_LEN]);
        off += SALT_LEN;
        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&bytes[off..off + NONCE_LEN]);
        off += NONCE_LEN;
        let mut backup_key_digest = [0u8; DIGEST_LEN];
        backup_key_digest.copy_from_slice(&bytes[off..off + DIGEST_LEN]);
        off += DIGEST_LEN;
        let mut audit_digest = [0u8; DIGEST_LEN];
        audit_digest.copy_from_slice(&bytes[off..off + DIGEST_LEN]);
        off += DIGEST_LEN;
        Ok(Self {
            version,
            kdf,
            salt,
            nonce,
            backup_key_digest,
            audit_digest,
            sealed_archive: bytes[off..].to_vec(),
        })
    }

    /// Whether this archive encodes explicit KDF metadata in its header.
    ///
    /// Legacy v1 archives restore correctly, but they rely on Mercury's baked-in
    /// legacy parameters instead of carrying a self-describing profile.
    pub fn stores_explicit_kdf_profile(&self) -> bool {
        self.version == VERSION_V2
    }

    /// Whether this archive already uses Mercury's current self-describing KDF
    /// profile.
    pub fn uses_current_kdf_profile(&self) -> bool {
        self.version == VERSION_V2 && self.kdf == BackupKdfParams::CURRENT_V2
    }

    /// Restore-planning hint for callers that want to accept legacy archives
    /// but migrate them to Mercury's current profile after a successful open.
    pub fn kdf_migration_status(&self) -> BackupKdfMigrationStatus {
        if self.uses_current_kdf_profile() {
            BackupKdfMigrationStatus::CurrentProfile
        } else {
            BackupKdfMigrationStatus::ResealRecommended
        }
    }

    /// Reconstruct the restore gate input from the archive's encoded KDF
    /// profile plus the caller's current restore context and deployment
    /// attestation.
    ///
    /// The archive does not yet encode scope, transport, or retention policy,
    /// so the caller must supply the restore context it is actually attempting.
    pub fn plan_restore(
        &self,
        scope: SecureBackupRestoreScope,
        transport: SecureBackupRestoreTransport,
        attestation: &BackupDeploymentAttestation,
        retention_days: i32,
        max_retention_days: i32,
    ) -> BackupRestorePlan {
        let restore_input = build_restore_input(
            self.kdf,
            scope,
            transport,
            attestation,
            retention_days,
            max_retention_days,
        );
        let restore_decision = restore_input.evaluate();
        let kdf_migration_status = self.kdf_migration_status();
        BackupRestorePlan {
            restore_input,
            restore_decision,
            kdf_migration_status,
            audit_record: BackupRestoreAuditRecord {
                sealed_backup_version: self.version,
                kdf_algorithm_code: self.kdf.algorithm.code(),
                kdf_algorithm_label: self.kdf.algorithm.label(),
                kdf_memory_mib: self.kdf.memory_mib,
                kdf_iterations: self.kdf.iterations,
                kdf_parallelism: self.kdf.parallelism,
                stores_explicit_kdf_profile: self.stores_explicit_kdf_profile(),
                uses_current_kdf_profile: self.uses_current_kdf_profile(),
                kdf_migration_status_code: kdf_migration_status.code(),
                kdf_migration_status_label: kdf_migration_status.label(),
                accepted: restore_decision.accepted,
                reason_code: restore_decision.reason.code(),
                reason_label: restore_decision.reason.label(),
            },
        }
    }
}

impl fmt::Debug for SealedBackup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SealedBackup")
            .field("archive_len", &self.sealed_archive.len())
            .finish_non_exhaustive()
    }
}

/// The result of creating a backup: the recovery code (show the user once), the
/// sealed blob (persist/upload), and the engine-produced gate input (call
/// `.evaluate()` to obtain the policy decision). Real restore flows should
/// derive a fresh input from [`SealedBackup::plan_restore`] so the decision uses
/// the current deployment attestation and the archive's encoded KDF profile.
pub struct CreatedBackup {
    pub recovery_code: RecoveryCode,
    pub sealed: SealedBackup,
    pub restore_input: SecureBackupRestoreInput,
}

/// `DOMAIN || VERSION || salt || nonce`, bound as AEAD associated data + hashed
/// into the audit digest so the archive cannot be transplanted onto a different
/// salt/nonce framing.
fn backup_manifest(
    version: u8,
    kdf: BackupKdfParams,
    salt: &[u8; SALT_LEN],
    nonce: &[u8; NONCE_LEN],
) -> Vec<u8> {
    let mut manifest =
        Vec::with_capacity(DOMAIN.len() + 1 + KDF_METADATA_LEN + SALT_LEN + NONCE_LEN);
    manifest.extend_from_slice(DOMAIN);
    manifest.push(version);
    if version == VERSION_V2 {
        kdf.append_to(&mut manifest);
    }
    manifest.extend_from_slice(salt);
    manifest.extend_from_slice(nonce);
    manifest
}

fn digests_match(actual: &[u8; DIGEST_LEN], expected: &[u8; DIGEST_LEN]) -> bool {
    actual.ct_eq(expected).unwrap_u8() == 1
}

/// Argon2id-derive the 32-byte backup key from the recovery code + salt.
fn derive_backup_key(
    code: &RecoveryCode,
    salt: &[u8; SALT_LEN],
    kdf: BackupKdfParams,
) -> Result<Zeroizing<[u8; 32]>, BackupError> {
    let params = argon2::Params::new(kdf.memory_kib()?, kdf.iterations, kdf.parallelism, Some(32))
        .map_err(|_| BackupError::KeyDerivation)?;
    let algorithm = match kdf.algorithm {
        BackupKdfAlgorithm::Argon2id => argon2::Algorithm::Argon2id,
    };
    let argon2 = argon2::Argon2::new(algorithm, argon2::Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(&code.0, salt, &mut *key)
        .map_err(|_| BackupError::KeyDerivation)?;
    Ok(key)
}

/// Produce the account-recovery decision the backup gate depends on. The crypto
/// facts are the engine's (192-bit recovery code, 32-byte key digest, encrypted
/// backup, no plaintext fields); the server-authentication / rate-limiting facts
/// come from `attestation`.
fn account_recovery_decision(attestation: &BackupDeploymentAttestation) -> AccountRecoveryDecision {
    evaluate_account_recovery(AccountRecoveryInput {
        recovery_requested: true,
        method: AccountRecoveryMethod::HighEntropyRecoveryKey,
        high_security_account: false,
        recovery_key_entropy_bits: RecoveryCode::ENTROPY_BITS,
        recovery_key_digest_len: DIGEST_LEN as i32,
        threshold_shares: attestation.threshold_shares,
        threshold_required: attestation.threshold_required,
        threshold_approvals: attestation.threshold_approvals,
        device_approval_present: attestation.device_approval_present,
        server_authenticated: attestation.server_authenticated,
        server_rate_limited: attestation.server_rate_limited,
        backup_encrypted: true,
        plaintext_backup_fields: 0,
        rotates_device_secret: attestation.restore_rotates_device_secret,
        audit_digest_len: DIGEST_LEN as i32,
    })
}

fn build_restore_input(
    kdf: BackupKdfParams,
    scope: SecureBackupRestoreScope,
    transport: SecureBackupRestoreTransport,
    attestation: &BackupDeploymentAttestation,
    retention_days: i32,
    max_retention_days: i32,
) -> SecureBackupRestoreInput {
    SecureBackupRestoreInput {
        account_recovery: account_recovery_decision(attestation),
        scope,
        transport,
        envelope_suite: SecureBackupRestoreEnvelopeSuite::XChaCha20Poly1305Blake3,
        high_security_account: false,
        backup_key_entropy_bits: RecoveryCode::ENTROPY_BITS,
        backup_key_digest_len: DIGEST_LEN as i32,
        kdf_memory_cost_mib: kdf.memory_mib as i32,
        kdf_iterations: kdf.iterations as i32,
        device_approval_present: attestation.device_approval_present,
        threshold_shares: attestation.threshold_shares,
        threshold_required: attestation.threshold_required,
        threshold_approvals: attestation.threshold_approvals,
        server_authenticated: attestation.server_authenticated,
        server_rate_limited: attestation.server_rate_limited,
        opaque_account_identifier: attestation.opaque_account_identifier,
        backup_encrypted: true,
        plaintext_export_fields: 0,
        os_plaintext_backup_excluded: attestation.os_plaintext_backup_excluded,
        mls_state_included: false,
        mls_state_sealed: false,
        mls_epoch_bound: false,
        restore_rotates_device_secret: attestation.restore_rotates_device_secret,
        restore_rekeys_groups: false,
        archive_manifest_authenticated: true,
        replay_nonce_len: NONCE_LEN as i32,
        audit_digest_len: DIGEST_LEN as i32,
        retention_days,
        max_retention_days,
    }
}

/// Create a secure backup of `archive`. Generates a fresh recovery code, derives
/// the backup key (Argon2id), encrypts the archive, and produces the
/// `SecureBackupRestoreInput` the caller evaluates against the gate.
///
/// `scope` / `transport` select the backup shape (this engine targets the
/// non-MLS, recovery-key-archive path: `AccountSecretsOnly` +
/// `UserRecoveryKeyArchive`); MLS-state backup is a later increment.
pub fn create_backup(
    archive: &[u8],
    scope: SecureBackupRestoreScope,
    transport: SecureBackupRestoreTransport,
    attestation: &BackupDeploymentAttestation,
    retention_days: i32,
    max_retention_days: i32,
) -> Result<CreatedBackup, BackupError> {
    create_backup_with_profile(
        archive,
        scope,
        transport,
        attestation,
        retention_days,
        max_retention_days,
        VERSION_V2,
        BackupKdfParams::CURRENT_V2,
    )
}

fn create_backup_with_profile(
    archive: &[u8],
    scope: SecureBackupRestoreScope,
    transport: SecureBackupRestoreTransport,
    attestation: &BackupDeploymentAttestation,
    retention_days: i32,
    max_retention_days: i32,
    version: u8,
    kdf: BackupKdfParams,
) -> Result<CreatedBackup, BackupError> {
    let recovery_code = RecoveryCode::generate();
    let salt = random_bytes::<SALT_LEN>();
    let nonce = random_bytes::<NONCE_LEN>();

    let backup_key = derive_backup_key(&recovery_code, &salt, kdf)?;
    let backup_key_digest: [u8; DIGEST_LEN] = *blake3::hash(&*backup_key).as_bytes();

    let manifest = backup_manifest(version, kdf, &salt, &nonce);
    let audit_digest: [u8; DIGEST_LEN] = *blake3::hash(&manifest).as_bytes();

    let cipher = XChaCha20Poly1305::new_from_slice(&*backup_key).map_err(|_| BackupError::Seal)?;
    drop(backup_key); // Zeroizing: wipes now (and on any early return), minimizing the key lifetime
    let sealed_archive = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: archive,
                aad: &manifest,
            },
        )
        .map_err(|_| BackupError::Seal)?;

    let sealed = SealedBackup {
        version,
        kdf,
        salt,
        nonce,
        backup_key_digest,
        audit_digest,
        sealed_archive,
    };

    let restore_input = build_restore_input(
        kdf,
        scope,
        transport,
        attestation,
        retention_days,
        max_retention_days,
    );

    Ok(CreatedBackup {
        recovery_code,
        sealed,
        restore_input,
    })
}

/// Open a sealed backup with the recovery `code`, returning the recovered archive
/// plaintext. Re-derives the key (Argon2id), checks it against the stored
/// key-identity digest (fast reject of a wrong code), verifies the manifest
/// audit digest, then AEAD-opens. Fails closed on a wrong code, tampering, or
/// malformed framing.
pub fn open_backup(code: &RecoveryCode, sealed: &SealedBackup) -> Result<Vec<u8>, BackupError> {
    // Enforce the KDF FLOOR before any work: a weak-Argon2id archive (a forked/downgraded producer
    // could mint one whose manifest is internally consistent) is brute-forceable. Refuse it here so
    // restore safety does not depend on the caller having run the restore gate (`plan_restore`) first.
    if sealed.kdf.memory_mib < MIN_KDF_MEMORY_MIB || sealed.kdf.iterations < MIN_KDF_ITERATIONS {
        return Err(BackupError::WeakKdf);
    }
    let backup_key = derive_backup_key(code, &sealed.salt, sealed.kdf)?;
    let derived_digest: [u8; DIGEST_LEN] = *blake3::hash(&*backup_key).as_bytes();
    if !digests_match(&derived_digest, &sealed.backup_key_digest) {
        return Err(BackupError::WrongRecoveryCode); // backup_key (Zeroizing) wipes on drop
    }

    let manifest = backup_manifest(sealed.version, sealed.kdf, &sealed.salt, &sealed.nonce);
    let audit_digest: [u8; DIGEST_LEN] = *blake3::hash(&manifest).as_bytes();
    if !digests_match(&audit_digest, &sealed.audit_digest) {
        return Err(BackupError::Malformed);
    }

    let cipher =
        XChaCha20Poly1305::new_from_slice(&*backup_key).map_err(|_| BackupError::OpenFailed)?;
    drop(backup_key);
    cipher
        .decrypt(
            XNonce::from_slice(&sealed.nonce),
            Payload {
                msg: &sealed.sealed_archive,
                aad: &manifest,
            },
        )
        .map_err(|_| BackupError::OpenFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mercury_core::SecureBackupRestoreReason;

    const SCOPE: SecureBackupRestoreScope = SecureBackupRestoreScope::AccountSecretsOnly;
    const TRANSPORT: SecureBackupRestoreTransport =
        SecureBackupRestoreTransport::UserRecoveryKeyArchive;

    fn create(archive: &[u8]) -> CreatedBackup {
        create_backup(
            archive,
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
        )
        .expect("backup creates")
    }

    #[test]
    fn backup_round_trips_under_its_recovery_code() {
        let archive = b"account secrets: identity key, device key, prekeys";
        let created = create(archive);
        assert_eq!(
            open_backup(&created.recovery_code, &created.sealed).unwrap(),
            archive
        );
    }

    #[test]
    fn backup_blob_serializes_and_reopens() {
        let archive = b"a backup archive to persist and restore";
        let created = create(archive);
        let blob = created.sealed.to_bytes();
        let reparsed = SealedBackup::from_bytes(&blob).unwrap();
        assert_eq!(reparsed, created.sealed);
        assert_eq!(
            open_backup(&created.recovery_code, &reparsed).unwrap(),
            archive
        );
    }

    #[test]
    fn v1_backups_still_restore_and_round_trip_exactly() {
        let archive = b"legacy backup created before kdf metadata";
        let created = create_backup_with_profile(
            archive,
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
            VERSION_V1,
            BackupKdfParams::LEGACY_V1,
        )
        .expect("legacy backup creates");
        let blob = created.sealed.to_bytes();
        assert_eq!(blob[4], VERSION_V1);
        let reparsed = SealedBackup::from_bytes(&blob).unwrap();
        assert_eq!(reparsed, created.sealed);
        assert_eq!(reparsed.to_bytes(), blob);
        assert_eq!(
            open_backup(&created.recovery_code, &reparsed).unwrap(),
            archive
        );
    }

    #[test]
    fn open_backup_refuses_a_sub_floor_kdf_archive() {
        // A forked/downgraded producer could mint a self-consistent archive with a WEAK Argon2id
        // profile. open_backup must REFUSE it (brute-forceable) -- not rely on the caller having run
        // the restore gate first. (The honest create_backup can't produce this; it hard-codes the
        // strong CURRENT_V2 profile.)
        let weak = BackupKdfParams {
            algorithm: BackupKdfAlgorithm::Argon2id,
            memory_mib: 8, // below the 64 MiB floor
            iterations: 1, // below the 3-iteration floor
            parallelism: 1,
        };
        let created = create_backup_with_profile(
            b"a brute-forceable archive",
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
            VERSION_V2,
            weak,
        )
        .expect("the weak producer can still seal it");
        assert!(
            matches!(
                open_backup(&created.recovery_code, &created.sealed),
                Err(BackupError::WeakKdf)
            ),
            "a sub-floor archive is refused before any key derivation"
        );

        // A normal (CURRENT_V2) archive is unaffected -- it still opens + round-trips exactly.
        let ok = create(b"a strong archive");
        assert_eq!(
            open_backup(&ok.recovery_code, &ok.sealed).unwrap(),
            b"a strong archive"
        );
    }

    #[test]
    fn new_backups_record_kdf_metadata_in_the_header() {
        let created = create(b"header metadata");
        let blob = created.sealed.to_bytes();
        assert_eq!(created.sealed.version, VERSION_V2);
        assert_eq!(blob[4], VERSION_V2);
        assert_eq!(blob[5], BackupKdfAlgorithm::Argon2id.code());
        assert_eq!(u32::from_le_bytes(blob[6..10].try_into().unwrap()), 64);
        assert_eq!(u32::from_le_bytes(blob[10..14].try_into().unwrap()), 3);
        assert_eq!(u32::from_le_bytes(blob[14..18].try_into().unwrap()), 1);
    }

    #[test]
    fn current_v2_backups_do_not_need_reseal() {
        let created = create(b"current profile");

        assert!(created.sealed.stores_explicit_kdf_profile());
        assert!(created.sealed.uses_current_kdf_profile());
        assert_eq!(
            created.sealed.kdf_migration_status(),
            BackupKdfMigrationStatus::CurrentProfile
        );
        assert!(!created.sealed.kdf_migration_status().recommends_reseal());
    }

    #[test]
    fn legacy_v1_backups_recommend_reseal_after_restore() {
        let created = create_backup_with_profile(
            b"legacy profile",
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
            VERSION_V1,
            BackupKdfParams::LEGACY_V1,
        )
        .expect("legacy backup creates");

        assert!(!created.sealed.stores_explicit_kdf_profile());
        assert!(!created.sealed.uses_current_kdf_profile());
        assert_eq!(
            created.sealed.kdf_migration_status(),
            BackupKdfMigrationStatus::ResealRecommended
        );
        assert!(created.sealed.kdf_migration_status().recommends_reseal());
    }

    #[test]
    fn nondefault_v2_profiles_recommend_reseal_after_restore() {
        let created = create_backup_with_profile(
            b"stronger custom profile",
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
            VERSION_V2,
            BackupKdfParams {
                algorithm: BackupKdfAlgorithm::Argon2id,
                memory_mib: 256,
                iterations: 4,
                parallelism: 1,
            },
        )
        .expect("custom backup creates");
        let reparsed = SealedBackup::from_bytes(&created.sealed.to_bytes()).unwrap();

        assert!(reparsed.stores_explicit_kdf_profile());
        assert!(!reparsed.uses_current_kdf_profile());
        assert_eq!(
            reparsed.kdf_migration_status(),
            BackupKdfMigrationStatus::ResealRecommended
        );
        assert!(reparsed.kdf_migration_status().recommends_reseal());
        assert_eq!(
            open_backup(&created.recovery_code, &reparsed).unwrap(),
            b"stronger custom profile"
        );
    }

    #[test]
    fn restore_plan_reuses_the_archive_kdf_profile() {
        let created = create_backup_with_profile(
            b"stronger custom profile",
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
            VERSION_V2,
            BackupKdfParams {
                algorithm: BackupKdfAlgorithm::Argon2id,
                memory_mib: 256,
                iterations: 4,
                parallelism: 1,
            },
        )
        .expect("custom backup creates");

        let plan = created.sealed.plan_restore(
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
        );
        assert_eq!(plan.restore_input.kdf_memory_cost_mib, 256);
        assert_eq!(plan.restore_input.kdf_iterations, 4);
        assert!(plan.restore_decision.accepted);
        assert_eq!(
            plan.kdf_migration_status,
            BackupKdfMigrationStatus::ResealRecommended
        );
        assert_eq!(plan.audit_record.sealed_backup_version, VERSION_V2);
        assert_eq!(plan.audit_record.kdf_algorithm_label, "argon2id");
        assert_eq!(plan.audit_record.kdf_memory_mib, 256);
        assert_eq!(plan.audit_record.kdf_iterations, 4);
        assert_eq!(plan.audit_record.kdf_parallelism, 1);
        assert!(plan.audit_record.stores_explicit_kdf_profile);
        assert!(!plan.audit_record.uses_current_kdf_profile);
        assert_eq!(plan.audit_record.kdf_migration_status_code, 2);
        assert_eq!(
            plan.audit_record.kdf_migration_status_label,
            "reseal_recommended"
        );
        assert!(plan.audit_record.accepted);
        assert_eq!(plan.audit_record.reason_code, 0);
        assert_eq!(plan.audit_record.reason_label, "ACCEPTED");
    }

    #[test]
    fn restore_plan_marks_legacy_archives_for_reseal() {
        let created = create_backup_with_profile(
            b"legacy profile",
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
            VERSION_V1,
            BackupKdfParams::LEGACY_V1,
        )
        .expect("legacy backup creates");

        let plan = created.sealed.plan_restore(
            SCOPE,
            TRANSPORT,
            &BackupDeploymentAttestation::user_recovery_archive(),
            30,
            90,
        );
        assert!(plan.restore_decision.accepted);
        assert_eq!(
            plan.kdf_migration_status,
            BackupKdfMigrationStatus::ResealRecommended
        );
        assert!(plan.kdf_migration_status.recommends_reseal());
        assert_eq!(plan.audit_record.sealed_backup_version, VERSION_V1);
        assert!(!plan.audit_record.stores_explicit_kdf_profile);
        assert!(!plan.audit_record.uses_current_kdf_profile);
        assert_eq!(
            plan.audit_record.kdf_migration_status_label,
            "reseal_recommended"
        );
    }

    #[test]
    fn restore_plan_rechecks_current_attestation() {
        let created = create(b"secrets");
        let mut attestation = BackupDeploymentAttestation::user_recovery_archive();
        attestation.server_authenticated = false;

        let plan = created
            .sealed
            .plan_restore(SCOPE, TRANSPORT, &attestation, 30, 90);
        assert!(!plan.restore_decision.accepted);
        assert_eq!(
            plan.restore_decision.reason,
            SecureBackupRestoreReason::AccountRecoveryRejected
        );
        assert!(!plan.audit_record.accepted);
        assert_eq!(plan.audit_record.reason_code, 1);
        assert_eq!(plan.audit_record.reason_label, "ACCOUNT_RECOVERY_REJECTED");
        assert_eq!(
            plan.audit_record.kdf_migration_status_label,
            "current_profile"
        );
    }

    #[test]
    fn the_engine_produces_a_gate_accepted_backup() {
        // The REAL engine parameters must satisfy evaluate_secure_backup_restore.
        let created = create(b"secrets");
        let decision = created.restore_input.evaluate();
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert!(decision.can_create_backup);
        assert!(decision.forbids_plaintext_export);
        assert!(decision.forbids_os_plaintext_backup);
        assert!(!decision.plaintext_bytes_exposed);
    }

    #[test]
    fn the_engine_reports_honest_parameters() {
        // Guard that the gate-input fields are the engine's real values, not
        // hand-tuned to the floor blindly.
        let created = create(b"secrets");
        let input = created.restore_input;
        assert_eq!(input.backup_key_entropy_bits, 192);
        assert_eq!(input.backup_key_digest_len, 32);
        assert_eq!(input.kdf_memory_cost_mib, 64);
        assert_eq!(input.kdf_iterations, 3);
        assert_eq!(input.replay_nonce_len, 24);
        assert_eq!(input.audit_digest_len, 32);
        assert!(input.backup_encrypted);
        assert_eq!(input.plaintext_export_fields, 0);
        assert!(input.account_recovery.accepted);
    }

    #[test]
    fn wrong_recovery_code_is_rejected() {
        let created = create(b"secrets");
        let wrong = RecoveryCode::generate();
        assert_eq!(
            open_backup(&wrong, &created.sealed),
            Err(BackupError::WrongRecoveryCode)
        );
    }

    #[test]
    fn tampered_archive_fails_closed() {
        let mut created = create(b"tamper target archive bytes");
        let last = created.sealed.sealed_archive.len() - 1;
        created.sealed.sealed_archive[last] ^= 0x01;
        assert_eq!(
            open_backup(&created.recovery_code, &created.sealed),
            Err(BackupError::OpenFailed)
        );
    }

    #[test]
    fn tampered_backup_key_digest_is_rejected() {
        let mut created = create(b"secrets");
        created.sealed.backup_key_digest[0] ^= 0x01;
        assert_eq!(
            open_backup(&created.recovery_code, &created.sealed),
            Err(BackupError::WrongRecoveryCode)
        );
    }

    #[test]
    fn tampered_audit_digest_is_rejected() {
        let mut created = create(b"secrets");
        created.sealed.audit_digest[0] ^= 0x01;
        assert_eq!(
            open_backup(&created.recovery_code, &created.sealed),
            Err(BackupError::Malformed)
        );
    }

    #[test]
    fn from_bytes_fails_closed() {
        let created = create(b"x");
        let blob = created.sealed.to_bytes();
        assert_eq!(SealedBackup::from_bytes(&[]), Err(BackupError::Malformed));
        assert_eq!(
            SealedBackup::from_bytes(&blob[..HEADER_V2_LEN - 1]),
            Err(BackupError::Malformed)
        );
        let mut bad_magic = blob.clone();
        bad_magic[0] ^= 0xff;
        assert_eq!(
            SealedBackup::from_bytes(&bad_magic),
            Err(BackupError::Malformed)
        );
    }

    #[test]
    fn missing_recovery_server_authentication_makes_the_gate_reject() {
        // A false deployment attestation (recovery service not authenticated) is
        // forwarded honestly -> the account-recovery dependency rejects, so the
        // backup gate rejects with AccountRecoveryRejected. The engine never
        // papers over a deployment that has not earned the backup.
        let mut attestation = BackupDeploymentAttestation::user_recovery_archive();
        attestation.server_authenticated = false;
        let created = create_backup(b"secrets", SCOPE, TRANSPORT, &attestation, 30, 90).unwrap();
        let decision = created.restore_input.evaluate();
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SecureBackupRestoreReason::AccountRecoveryRejected
        );
    }

    #[test]
    fn missing_device_secret_rotation_makes_the_gate_reject() {
        // `restore_rotates_device_secret` is a client RESTORE-FLOW fact the engine
        // forwards from the attestation, not fabricates: a deployment whose restore
        // does not rotate the device secret is rejected (RestoreRekeyMissing).
        let mut attestation = BackupDeploymentAttestation::user_recovery_archive();
        attestation.restore_rotates_device_secret = false;
        let created = create_backup(b"secrets", SCOPE, TRANSPORT, &attestation, 30, 90).unwrap();
        let decision = created.restore_input.evaluate();
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SecureBackupRestoreReason::RestoreRekeyMissing
        );
    }
}
