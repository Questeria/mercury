#![forbid(unsafe_code)]
//! Mercury application controller — the real send/receive/session surface the UI drives.
//!
//! [`AppController`] owns one Mercury identity, discovers peers through the relay's prekey
//! directory, opens a post-quantum ratchet session per peer (via [`mercury_client`]), and
//! keeps a UI-facing [`ChatMessage`] log. It is the integration layer that lets a real
//! end-to-end-encrypted conversation drive the UI instead of the in-app TypeScript simulator.
//!
//! Layering (kept deliberately clean):
//!   * confidentiality / forward secrecy / post-compromise security — delegated to
//!     [`mercury_client`] (the inner ratchet + the sealed-sender envelope). NO crypto here.
//!   * admission / policy decisions — the separate decision bridge (`mercury-bindings`),
//!     which stays plaintext-free. This controller carries plaintext between the UI and the
//!     client runtime, which the decision bridge intentionally must not.
//!   * transport — any [`mercury_client::Transport`] (the in-memory test double or the real
//!     HTTP relay).
//!
//! The controller is transport- and runtime-agnostic so a thin FFI/Tauri/HTTP shim can drive
//! it from the UI; that shim (and the UI simulator replacement) is a later increment. This
//! crate is fully headless-testable: two controllers over a shared in-memory transport hold a
//! complete conversation.

use std::collections::{HashMap, HashSet};

use mercury_bindings::{
    mercury_accept_received_ciphertext, mercury_bootstrap_status, mercury_prepare_send,
};
use mercury_client::{
    ClientError, ContactCard, MercuryClient, PollAuth, Transport, TransportError,
    UsernameClaimOutcome, open_blob, seal_blob,
};
use mercury_core::{
    AccountRecoveryInput, AccountRecoveryMethod, ClientBootstrapDecision, ClientBootstrapReason,
    ClientReceiveDecision, ClientReceiveReason, OutboundSendDecision, OutboundSendReason,
    PlatformDecisionView, SecureBackupRestoreEnvelopeSuite, SecureBackupRestoreInput,
    SecureBackupRestoreReason, SecureBackupRestoreScope, SecureBackupRestoreTransport,
};
use mercury_mls::MlsEngine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use zeroize::Zeroize;

/// Account-id width, in bytes.
const ID_LEN: usize = 32;

/// Marker prefix that distinguishes an in-band CONTROL message (e.g. delete-for-everyone) from a
/// normal text message INSIDE the sealed ratchet payload. It begins with ESC (`0x1b`), which no
/// normal chat text starts with, so a real message is never mistaken for a control op (and a
/// user typing anything can never forge one). The control op itself is JSON after this prefix.
const CONTROL_SENTINEL: &[u8] = b"\x1bMERCURY-CTRL\x1b";

/// SINGLE-FRAME attachment cap, in bytes (512 KiB): a file at or under this rides ONE sealed `att`
/// frame. The relay's per-message ciphertext ceiling is 1 MiB; base64 (4/3) plus JSON framing plus
/// sealed-envelope overhead must fit under it, so this leaves honest headroom. LARGER files (up to
/// [`MAX_ATTACHMENT_TOTAL_BYTES`]) are split into ordered chunked `attc` frames — shipped, not
/// faked. Files beyond that ceiling still need a real blob transport (a later increment).
pub const MAX_ATTACHMENT_BYTES: usize = 512 * 1024;

/// Maximum TOTAL raw size for a CHUNKED attachment (4 MiB). Files over [`MAX_ATTACHMENT_BYTES`]
/// are split into [`ATTACHMENT_CHUNK_RAW`]-sized frames that ride the same sealed channel in
/// order (the outbox preserves ordering across full mailboxes); the receiver reassembles them
/// transiently and logs ONE attachment message. Both sides need a chunk-aware build — an older
/// receiver silently ignores the unknown frames (disclosed in the UI). Larger files still need a
/// real blob transport — a later increment.
pub const MAX_ATTACHMENT_TOTAL_BYTES: usize = 4 * 1024 * 1024;
/// Raw bytes per chunk frame (384 KiB raw ≈ 512 KiB base64 + framing, comfortably under the
/// relay's 1 MiB ciphertext ceiling).
pub const ATTACHMENT_CHUNK_RAW: usize = 384 * 1024;
/// How long a partially-received chunked transfer may sit before it is evicted (seconds).
const ATTACHMENT_REASSEMBLY_TTL_S: i64 = 600;

/// Global ceiling on concurrent chunked-attachment reassemblies across ALL senders. The per-sender
/// cap (2) bounds one peer; this bounds the aggregate so many minted identities can't each open
/// transfers and balloon memory (worst case stays under this × [`MAX_ATTACHMENT_TOTAL_BYTES`]). A
/// new transfer past the ceiling is dropped (fail-closed); the sender retries once a slot frees.
const MAX_CONCURRENT_REASSEMBLIES: usize = 8;

/// A partially-received chunked attachment (transient — never persisted; an app restart simply
/// drops the partial transfer and the sender's copy stays intact).
struct PartialAttachment {
    name: String,
    mime: String,
    total: u32,
    parts: HashMap<u32, Vec<u8>>,
    bytes: usize,
    started_s: i64,
}

/// Maximum members in a group, INCLUDING yourself. Group traffic fans out over each member's 1:1
/// sealed channel (N-1 sealed sends per message), so the cap keeps bandwidth + the single-slot
/// mailbox pressure honest for v1. A relay-side group route would lift it — a later increment.
pub const MAX_GROUP_MEMBERS: usize = 16;
/// Maximum group display-name length, in characters.
pub const MAX_GROUP_NAME_CHARS: usize = 48;

/// Reduce a group display name to a safe, single-line form: control characters stripped,
/// whitespace collapsed, length-capped, never empty.
fn sanitize_group_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .filter(|c| !c.is_control())
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let capped: String = collapsed.chars().take(MAX_GROUP_NAME_CHARS).collect();
    if capped.is_empty() {
        "Group".to_string()
    } else {
        capped
    }
}

/// Everything the controller knows about one group BESIDES the MLS cryptographic state (which
/// lives in the [`MlsEngine`]). Persisted in the encrypted snapshot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupMeta {
    /// Display name (sanitized; set by the creator, carried in the invite).
    pub name: String,
    /// The group's creator/administrator: the only member who admits invitees and processes
    /// leaves (v1 is creator-administered — disclosed, not hidden).
    pub creator: [u8; ID_LEN],
    /// COMMITTED members (present in the MLS roster), self included.
    pub members: Vec<[u8; ID_LEN]>,
    /// Invited-but-not-yet-joined members (creator-side bookkeeping only).
    #[serde(default)]
    pub pending: Vec<[u8; ID_LEN]>,
    /// True once the creator closed the group (read-only locally from then on).
    #[serde(default)]
    pub ended: bool,
}

/// SHA-256 of a message body, lowercase hex — how a delivery receipt references the message it
/// confirms. No plaintext crosses the wire in a receipt: the digest is computed independently on
/// both ends from the body they already share over the sealed channel.
fn body_digest(body: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(body);
    hex_encode_vec(&h.finalize())
}

/// Lowercase-hex of arbitrary bytes (the fixed-width `hex_encode` below covers only account ids).
fn hex_encode_vec(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Backup-file magic prefixes. `MERCBAK2` (current) seals under a memory-hard Argon2id key;
/// `MERCBAK1` (legacy) used PBKDF2-HMAC-SHA512 and is still accepted on import so older backups
/// restore. The Argon2id header is self-describing (algorithm + params) for forward-compatibility.
const BACKUP_MAGIC_V2: &[u8] = b"MERCBAK2";
const BACKUP_MAGIC: &[u8] = b"MERCBAK1";
/// KDF algorithm tag stored in a `MERCBAK2` header.
const BACKUP_KDF_ARGON2ID: u8 = 1;
/// CURRENT backup KDF parameters: Argon2id, 64 MiB / 3 passes / 1 lane — memory-hard (far harder to
/// crack on GPU/ASIC than PBKDF2), matching the `mercury-backup` recovery profile and exceeding the
/// OWASP Argon2id floor (>=19 MiB, >=2 passes).
const BACKUP_ARGON2_MEM_KIB: u32 = 64 * 1024;
const BACKUP_ARGON2_ITERS: u32 = 3;
const BACKUP_ARGON2_LANES: u32 = 1;
/// Sanity caps applied when IMPORTING a `MERCBAK2` header, so a hostile/corrupt file can't make
/// Argon2id allocate absurd memory on restore — fail closed rather than OOM.
const BACKUP_ARGON2_MAX_MEM_KIB: u32 = 1024 * 1024; // 1 GiB
const BACKUP_ARGON2_MAX_ITERS: u32 = 64;
const BACKUP_ARGON2_MAX_LANES: u32 = 16;
/// LEGACY PBKDF2-HMAC-SHA512 rounds — used ONLY to restore `MERCBAK1` backups from older builds.
const BACKUP_KDF_ITERATIONS: u32 = 210_000;
/// Minimum backup passphrase length, in characters. The backup file's security IS the passphrase.
pub const MIN_BACKUP_PASSPHRASE_CHARS: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppBackupKdf {
    Argon2id,
    Pbkdf2HmacSha512,
}

impl AppBackupKdf {
    const fn code(self) -> u8 {
        match self {
            Self::Argon2id => BACKUP_KDF_ARGON2ID,
            Self::Pbkdf2HmacSha512 => 0,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Argon2id => "argon2id",
            Self::Pbkdf2HmacSha512 => "pbkdf2-hmac-sha512",
        }
    }
}

/// Non-secret restore plan for a passphrase-sealed app backup.
///
/// This is intentionally separate from `mercury-backup`'s recovery-code archive plan:
/// arbitrary user passphrases are not represented as 192-bit recovery codes. The
/// `high_assurance_gate_*` fields expose that boundary without changing the legacy
/// desktop backup format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppBackupRestorePlan {
    pub format_version: u8,
    pub kdf_algorithm_code: u8,
    pub kdf_algorithm_label: &'static str,
    pub kdf_memory_mib: u32,
    pub kdf_iterations: u32,
    pub kdf_parallelism: u32,
    pub stores_explicit_kdf_profile: bool,
    pub uses_current_kdf_profile: bool,
    pub legacy_format: bool,
    pub reseal_recommended: bool,
    pub sealed_snapshot_len: usize,
    pub restore_allowed: bool,
    pub reason_label: &'static str,
    pub high_assurance_gate_accepted: bool,
    pub high_assurance_gate_reason: SecureBackupRestoreReason,
    pub high_assurance_gate_reason_label: &'static str,
}

struct ParsedAppBackup<'a> {
    plan: AppBackupRestorePlan,
    kdf: AppBackupKdf,
    salt: [u8; 16],
    sealed: &'a [u8],
}

fn passphrase_restore_gate_reason(kdf_memory_mib: u32, kdf_iterations: u32) -> SecureBackupRestoreReason {
    let account_recovery = AccountRecoveryInput {
        recovery_requested: true,
        method: AccountRecoveryMethod::HighEntropyRecoveryKey,
        high_security_account: false,
        recovery_key_entropy_bits: 0,
        recovery_key_digest_len: 0,
        threshold_shares: 0,
        threshold_required: 0,
        threshold_approvals: 0,
        device_approval_present: false,
        server_authenticated: false,
        server_rate_limited: false,
        backup_encrypted: true,
        plaintext_backup_fields: 0,
        rotates_device_secret: true,
        audit_digest_len: 32,
    }
    .evaluate();
    let decision = SecureBackupRestoreInput {
        account_recovery,
        scope: SecureBackupRestoreScope::AccountAndMlsState,
        transport: SecureBackupRestoreTransport::UserRecoveryKeyArchive,
        envelope_suite: SecureBackupRestoreEnvelopeSuite::XChaCha20Poly1305Blake3,
        high_security_account: false,
        backup_key_entropy_bits: 0,
        backup_key_digest_len: 0,
        kdf_memory_cost_mib: kdf_memory_mib as i32,
        kdf_iterations: kdf_iterations as i32,
        device_approval_present: false,
        threshold_shares: 0,
        threshold_required: 0,
        threshold_approvals: 0,
        server_authenticated: false,
        server_rate_limited: false,
        opaque_account_identifier: true,
        backup_encrypted: true,
        plaintext_export_fields: 0,
        os_plaintext_backup_excluded: true,
        mls_state_included: true,
        mls_state_sealed: true,
        mls_epoch_bound: true,
        restore_rotates_device_secret: true,
        restore_rekeys_groups: true,
        archive_manifest_authenticated: true,
        replay_nonce_len: 24,
        audit_digest_len: 32,
        retention_days: 30,
        max_retention_days: 90,
    }
    .evaluate();
    decision.reason
}

fn parse_app_backup(backup: &[u8]) -> Result<ParsedAppBackup<'_>, AppError> {
    if let Some(rest) = backup.strip_prefix(BACKUP_MAGIC_V2) {
        if rest.len() < 29 || rest[0] != BACKUP_KDF_ARGON2ID {
            return Err(AppError::BackupInvalid);
        }
        let mem_kib = u32::from_le_bytes([rest[1], rest[2], rest[3], rest[4]]);
        let iters = u32::from_le_bytes([rest[5], rest[6], rest[7], rest[8]]);
        let lanes = u32::from_le_bytes([rest[9], rest[10], rest[11], rest[12]]);
        if mem_kib > BACKUP_ARGON2_MAX_MEM_KIB
            || !(1..=BACKUP_ARGON2_MAX_ITERS).contains(&iters)
            || !(1..=BACKUP_ARGON2_MAX_LANES).contains(&lanes)
        {
            return Err(AppError::BackupInvalid);
        }
        let salt: [u8; 16] = rest[13..29]
            .try_into()
            .map_err(|_| AppError::BackupInvalid)?;
        let sealed = &rest[29..];
        let kdf = AppBackupKdf::Argon2id;
        let kdf_memory_mib = mem_kib / 1024;
        let high_assurance_gate_reason = passphrase_restore_gate_reason(kdf_memory_mib, iters);
        let uses_current_kdf_profile = mem_kib == BACKUP_ARGON2_MEM_KIB
            && iters == BACKUP_ARGON2_ITERS
            && lanes == BACKUP_ARGON2_LANES;
        return Ok(ParsedAppBackup {
            plan: AppBackupRestorePlan {
                format_version: 2,
                kdf_algorithm_code: kdf.code(),
                kdf_algorithm_label: kdf.label(),
                kdf_memory_mib,
                kdf_iterations: iters,
                kdf_parallelism: lanes,
                stores_explicit_kdf_profile: true,
                uses_current_kdf_profile,
                legacy_format: false,
                reseal_recommended: !uses_current_kdf_profile,
                sealed_snapshot_len: sealed.len(),
                restore_allowed: true,
                reason_label: "ACCEPTED_COMPATIBILITY_RESTORE",
                high_assurance_gate_accepted: high_assurance_gate_reason
                    == SecureBackupRestoreReason::Accepted,
                high_assurance_gate_reason,
                high_assurance_gate_reason_label: high_assurance_gate_reason.label(),
            },
            kdf,
            salt,
            sealed,
        });
    }

    if let Some(rest) = backup.strip_prefix(BACKUP_MAGIC) {
        if rest.len() < 16 {
            return Err(AppError::BackupInvalid);
        }
        let (salt, sealed) = rest.split_at(16);
        let salt: [u8; 16] = salt.try_into().map_err(|_| AppError::BackupInvalid)?;
        let kdf = AppBackupKdf::Pbkdf2HmacSha512;
        let high_assurance_gate_reason = passphrase_restore_gate_reason(0, BACKUP_KDF_ITERATIONS);
        return Ok(ParsedAppBackup {
            plan: AppBackupRestorePlan {
                format_version: 1,
                kdf_algorithm_code: kdf.code(),
                kdf_algorithm_label: kdf.label(),
                kdf_memory_mib: 0,
                kdf_iterations: BACKUP_KDF_ITERATIONS,
                kdf_parallelism: 1,
                stores_explicit_kdf_profile: false,
                uses_current_kdf_profile: false,
                legacy_format: true,
                reseal_recommended: true,
                sealed_snapshot_len: sealed.len(),
                restore_allowed: true,
                reason_label: "ACCEPTED_LEGACY_RESEAL_RECOMMENDED",
                high_assurance_gate_accepted: high_assurance_gate_reason
                    == SecureBackupRestoreReason::Accepted,
                high_assurance_gate_reason,
                high_assurance_gate_reason_label: high_assurance_gate_reason.label(),
            },
            kdf,
            salt,
            sealed,
        });
    }

    Err(AppError::BackupInvalid)
}

/// Stretch a passphrase into the 32-byte seal key with the CURRENT KDF: memory-hard Argon2id
/// (audited RustCrypto `argon2`), domain-separated by the per-export random salt. Bad parameters
/// (only reachable from a corrupt import header — export uses fixed valid params) fail closed.
fn derive_backup_key_argon2(
    passphrase: &str,
    salt: &[u8],
    mem_kib: u32,
    iters: u32,
    lanes: u32,
) -> Result<[u8; 32], AppError> {
    let params = argon2::Params::new(mem_kib, iters, lanes, Some(32))
        .map_err(|_| AppError::BackupInvalid)?;
    let kdf = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut key = [0u8; 32];
    kdf.hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| AppError::BackupInvalid)?;
    Ok(key)
}

/// LEGACY: stretch a passphrase via PBKDF2-HMAC-SHA512 — used only to restore `MERCBAK1` backups.
fn derive_backup_key(passphrase: &str, salt: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha512>>(
        passphrase.as_bytes(),
        salt,
        BACKUP_KDF_ITERATIONS,
        &mut key,
    )
    .expect("pbkdf2 output length is valid");
    key
}

/// serde default for booleans that must be ON for snapshots written before the field existed.
fn default_true() -> bool {
    true
}

/// A short human label for an account id in system notices ("a1b2c3…d4e5").
fn short_label(id: &[u8; ID_LEN]) -> String {
    let h = hex_encode(id);
    format!("{}…{}", &h[..6], &h[h.len() - 4..])
}

/// Reduce an attacker-supplied filename to a safe display/save name: the final path component
/// only (no traversal), control characters stripped, length-capped, never empty. The receiver
/// chooses where a file is saved; the sender only ever suggests a NAME.
fn sanitize_filename(raw: &str) -> String {
    let last = raw
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_start_matches('.');
    let cleaned: String = last
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        .take(120)
        .collect();
    if cleaned.is_empty() {
        "attachment.bin".to_string()
    } else {
        cleaned
    }
}

/// Local wall-clock, Unix seconds. Stamps messages + evaluates disappearing-message expiry.
fn now_unix_s() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Whether message `m` has passed its conversation's disappearing TTL as of `now`. A message with
/// no timestamp (`ts <= 0`, i.e. logged before the feature) NEVER auto-expires, and a conversation
/// with no timer set never expires. Pure (takes no `self`) so the prune in `poll` can run inside a
/// `retain` without a borrow conflict.
fn msg_expired(disappearing: &HashMap<[u8; ID_LEN], u64>, m: &ChatMessage, now: i64) -> bool {
    if m.ts <= 0 {
        return false;
    }
    match parse_id(&m.peer) {
        Ok(id) => disappearing
            .get(&id)
            .is_some_and(|&ttl| ttl > 0 && now.saturating_sub(m.ts) >= ttl as i64),
        Err(_) => false,
    }
}

/// Direction of a message relative to this controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Sent by us.
    Outgoing,
    /// Received from a peer.
    Incoming,
}

/// An attachment carried inside a message: files ride the SAME sealed ratchet channel as text
/// (in-band control frames), so the relay sees only ciphertext and there is no separate blob store
/// to trust. A file at or under [`MAX_ATTACHMENT_BYTES`] rides one frame; larger files up to
/// [`MAX_ATTACHMENT_TOTAL_BYTES`] (4 MiB) are chunked. Honestly a documents/photos feature, not a
/// large-media pipeline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// Display filename (sanitized to its final path component on receive).
    pub name: String,
    /// MIME type as declared by the SENDER — advisory for rendering (e.g. inline image preview),
    /// never trusted for anything security-relevant.
    pub mime: String,
    /// The file bytes, base64 (the form they cross the IPC boundary in).
    pub data_b64: String,
}

/// One message in the UI-facing conversation log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub direction: Direction,
    /// Lowercase-hex account id of the OTHER party (the recipient of an outgoing message, the
    /// sender of an incoming one).
    pub peer: String,
    /// The message body as UTF-8 text. Incoming non-UTF-8 bytes are decoded lossily: the 1:1
    /// chat product treats bodies as text; binary payloads ride [`ChatMessage::att`]. The
    /// cryptographic transport itself is byte-exact regardless.
    pub text: String,
    /// A local monotonic sequence number for stable UI ordering. This is NOT the ratchet
    /// message index and carries no protocol meaning.
    pub seq: u64,
    /// Wall-clock creation time (local Unix seconds) — drives disappearing-message expiry and the
    /// rendered timestamp. `#[serde(default)]` (0) so messages logged before this field existed
    /// never auto-expire.
    #[serde(default)]
    pub ts: i64,
    /// Outgoing only: the peer's device confirmed receipt (an in-band `rcpt` control message that
    /// rides the same sealed channel). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub delivered: bool,
    /// Outgoing only: queued locally (the recipient's single-slot relay mailbox was full) and
    /// being retried in order by the outbox. Cleared when the relay accepts it.
    #[serde(default)]
    pub pending: bool,
    /// The attachment carried by this message, if any.
    #[serde(default)]
    pub att: Option<Attachment>,
    /// Outgoing only: SHA-256 (hex) of the sealed plaintext body — what an inbound `rcpt` control
    /// frame references to mark THIS message delivered. Empty for incoming/legacy messages.
    #[serde(default)]
    pub digest: String,
    /// GROUP messages only: the lowercase-hex account id of the actual SENDER (in a group, `peer`
    /// holds the group id, so attribution needs its own field). `None` for 1:1 messages and group
    /// system notices. The value is MLS-attributed (the sender credential inside the group
    /// ciphertext), cross-checked against the authenticated 1:1 carrier — never taken on faith.
    #[serde(default)]
    pub from: Option<String>,
    /// Outgoing 1:1 only: the peer's device reported the message was READ (their conversation
    /// was open + focused) — only sent when the PEER has read receipts enabled. `delivered` and
    /// `read` are distinct, honest states; neither is ever inferred.
    #[serde(default)]
    pub read: bool,
}

/// An application-level error. Coarse and free of any key or plaintext material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppError {
    /// An underlying client/crypto operation failed (bad/unverifiable card, decryption,
    /// replay, unknown peer, ...).
    Client(ClientError),
    /// The transport (relay / directory) could not be reached or refused the request. The
    /// string carries the underlying [`mercury_client::TransportError`] detail — network
    /// failure vs. a relay rejection *with its reason* (e.g. the recipient's single-slot
    /// mailbox already holds an undelivered item) — so a caller, including an AI participant,
    /// can reason about the failure instead of seeing an opaque "transport failed".
    Transport(String),
    /// A send was attempted to a peer with neither an established session nor a known contact
    /// card. Call [`AppController::add_contact`] first.
    NoContact,
    /// An account-id string was not exactly 32 bytes of lowercase hex.
    BadAccountId,
    /// A persisted encrypted snapshot could not be restored: the wrong at-rest key, or a
    /// corrupt/foreign/tampered blob. Fail-closed — never a partial restore.
    Persistence,
    /// A send was attempted to a BLOCKED contact. Unblock them first.
    Blocked,
    /// An attachment payload was not valid base64.
    AttachmentInvalid,
    /// An attachment exceeds [`MAX_ATTACHMENT_TOTAL_BYTES`] (4 MiB, the chunked cap) or is empty.
    AttachmentTooLarge,
    /// A backup blob could not be opened: wrong passphrase, or a corrupt/foreign/tampered file.
    /// Fail-closed — never a partial restore.
    BackupInvalid,
    /// A backup passphrase shorter than [`MIN_BACKUP_PASSPHRASE_CHARS`] — refused, because the
    /// exported file's entire security is the passphrase.
    WeakPassphrase,
    /// A group operation failed (engine error, unknown group, ended group, invalid name/members).
    /// The string is a human-readable, key/plaintext-free reason.
    Group(String),
}

impl From<ClientError> for AppError {
    fn from(e: ClientError) -> Self {
        AppError::Client(e)
    }
}

impl core::fmt::Display for AppError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AppError::Client(e) => write!(f, "client error: {e}"),
            AppError::Transport(detail) => write!(f, "transport failed: {detail}"),
            AppError::NoContact => f.write_str("no session or contact card for that peer"),
            AppError::BadAccountId => f.write_str("account id must be 32-byte lowercase hex"),
            AppError::Persistence => f.write_str("encrypted snapshot could not be restored"),
            AppError::Blocked => f.write_str("that contact is blocked"),
            AppError::AttachmentInvalid => f.write_str("attachment data is not valid base64"),
            AppError::AttachmentTooLarge => {
                f.write_str("attachment is empty or larger than the 4 MiB limit")
            }
            AppError::BackupInvalid => {
                f.write_str("backup could not be opened (wrong passphrase or damaged file)")
            }
            AppError::WeakPassphrase => {
                f.write_str("backup passphrase must be at least 9 characters")
            }
            AppError::Group(reason) => write!(f, "group error: {reason}"),
        }
    }
}

impl std::error::Error for AppError {}

/// The application controller: one identity, its conversations, and a transport.
///
/// Generic over the [`Transport`] so the same logic runs over the in-memory test double and
/// the real HTTP relay.
pub struct AppController<T: Transport> {
    client: MercuryClient,
    transport: T,
    /// Verified contact cards for peers we can open a conversation with.
    contacts: HashMap<[u8; ID_LEN], ContactCard>,
    /// The full message log, in local (arrival/send) order.
    log: Vec<ChatMessage>,
    /// Next local sequence number.
    seq: u64,
    /// Whether the most recent `receive` attempt succeeded — drives the REAL "last receive"
    /// decision view. Starts true (no failure observed yet).
    last_receive_ok: bool,
    /// MUTED peers: the UI suppresses notifications for these conversations. Local + persisted;
    /// delivery is unaffected.
    muted: HashSet<[u8; ID_LEN]>,
    /// BLOCKED peers: outgoing sends are refused, and incoming messages are decrypted-then-dropped
    /// (the ratchet still advances, but the message is never logged / shown / notified). Reversible
    /// via `unblock`. Local + persisted.
    blocked: HashSet<[u8; ID_LEN]>,
    /// Per-conversation disappearing-message TTL in seconds (peer id -> secs); absent/0 = off.
    /// Local + persisted; conveyed to the peer via a `timer` control message so both sides expire.
    disappearing: HashMap<[u8; ID_LEN], u64>,
    /// The directory VRF key this account has PINNED for username key-transparency verification.
    /// Set out-of-band by the desktop shell for the default relay (baked into the signed build), or
    /// on the first username resolve (trust-on-first-use) for a custom relay; a later relay key
    /// rotation then fails the proof. Local + persisted.
    kt_vrf_pin: Option<Vec<u8>>,
    /// The directory's LOG public key pinned out-of-band (baked into the signed build for the
    /// default relay): when set, every username lookup must also present a tree head SIGNED by
    /// this key, with the proof's epoch bounded by it. Local + persisted.
    kt_log_pin: Option<[u8; ID_LEN]>,
    /// Peers this user has marked VERIFIED after comparing safety numbers out-of-band. Purely a
    /// local, persisted attestation the UI surfaces — it grants no capability and is cleared only
    /// by the user. The honest meaning: "I compared the safety number over a channel I trust."
    verified: HashSet<[u8; ID_LEN]>,
    /// Human aliases for peers (peer id -> handle), learned when a contact is added BY USERNAME
    /// (the handle that was resolved + proof-verified at add time). Display-only; the id stays the
    /// cryptographic identity. Local + persisted.
    aliases: HashMap<[u8; ID_LEN], String>,
    /// The username THIS account successfully claimed (normalized), if any. Display-only.
    my_username: Option<String>,
    /// FIFO outbox: messages the relay refused because the peer's single-slot mailbox still held an
    /// undelivered item. Retried IN ORDER on every poll tick (order matters for the ratchet), so a
    /// fast typist never loses a message — it just shows "pending" until the peer drains their
    /// mailbox. Each entry: (peer, sealed blob, log seq of the pending ChatMessage).
    outbox: Vec<([u8; ID_LEN], Vec<u8>, u64)>,
    /// TRANSIENT (never persisted): conversations whose EXISTING messages changed since the last
    /// poll response (a delivery receipt arrived, or a pending message was accepted), so the UI
    /// refreshes their history without treating anything as a new arrival.
    dirty_peers: HashSet<String>,
    /// TRANSIENT (never persisted): the highest read-watermark already SENT per peer, so focus
    /// events don't re-send identical watermarks every second.
    read_sent: HashMap<[u8; ID_LEN], i64>,
    /// TRANSIENT (never persisted): in-flight chunked-attachment reassembly, keyed by
    /// (sender, transfer id). Strictly capped (per-transfer total + at most 2 concurrent per
    /// sender) and TTL-evicted, so a hostile peer cannot balloon memory.
    pending_chunks: HashMap<([u8; ID_LEN], String), PartialAttachment>,
    /// TRANSIENT (never persisted): set by [`AppController::poll`] when a poll actually changed
    /// durable state (a session was accepted, a one-time prekey was consumed or republished, the
    /// outbox drained, or a disappearing prune ran) even if it surfaced NO new message and NO dirty
    /// conversation — e.g. a control-only first-flight (a group invite). The shell reads it via
    /// [`AppController::take_state_dirty`] to decide whether an otherwise-idle poll must re-seal the
    /// snapshot, so such state can never be lost on a quit/restart (and a consumed prekey can never
    /// be silently re-opened). Reset on read.
    state_dirty: bool,
    /// Whether THIS account sends READ receipts (the privacy toggle; delivery acks are protocol
    /// reliability and always flow). Local + persisted; default on.
    send_read_receipts: bool,
    /// PINNED conversations (1:1 peers or group ids) — a display preference the rail sorts by.
    /// Local + persisted.
    pinned: HashSet<[u8; ID_LEN]>,
    /// The persistent MLS group engine (lazily created at first group use). Its exported state
    /// (MLS secrets) rides the encrypted snapshot like every other secret.
    mls: Option<MlsEngine>,
    /// Group metadata, keyed by the 32-byte group id. The MLS roster is the cryptographic truth;
    /// this mirror is what the UI renders and what survives in the snapshot.
    groups: HashMap<[u8; ID_LEN], GroupMeta>,
}

/// A sealed, at-rest snapshot of an [`AppController`]'s full state — the encrypted client pickle
/// (identity + device + every ratchet session), the verified contact cards, the message log, and
/// the local ordering counters. The plaintext form is secret-bearing (the message log is
/// plaintext), so it is only ever produced sealed by [`AppController::to_encrypted_snapshot`], and
/// the serialized plaintext is zeroized after sealing / before restore returns.
#[derive(Serialize, Deserialize)]
struct AppSnapshot {
    /// `MercuryClient::to_encrypted_pickle` output (the crypto state, already sealed under the key).
    client_pickle: Vec<u8>,
    /// `(account-id bytes, verified card)` pairs — the `contacts` map, order-independent.
    contacts: Vec<([u8; ID_LEN], ContactCard)>,
    /// The full UI-facing message log, in local order (plaintext bodies — sealed by the outer blob).
    log: Vec<ChatMessage>,
    seq: u64,
    last_receive_ok: bool,
    /// Muted peer ids. `#[serde(default)]` so a snapshot written BEFORE this field existed restores
    /// with none muted — backward-compatible (the AI participant's on-disk snapshot keeps loading).
    #[serde(default)]
    muted: Vec<[u8; ID_LEN]>,
    /// Blocked peer ids. `#[serde(default)]` for the same backward-compatibility.
    #[serde(default)]
    blocked: Vec<[u8; ID_LEN]>,
    /// Per-conversation disappearing TTLs `(peer id, secs)`. `#[serde(default)]` for back-compat.
    #[serde(default)]
    disappearing: Vec<([u8; ID_LEN], u64)>,
    /// The pinned directory VRF key (username KT). `#[serde(default)]` so older snapshots restore
    /// with nothing pinned (they re-pin on first use).
    #[serde(default)]
    kt_vrf_pin: Option<Vec<u8>>,
    /// Pinned KT LOG public key. `#[serde(default)]` for back-compat.
    #[serde(default)]
    kt_log_pin: Option<[u8; ID_LEN]>,
    /// Safety-number-verified peer ids. `#[serde(default)]` for back-compat.
    #[serde(default)]
    verified: Vec<[u8; ID_LEN]>,
    /// `(peer id, username alias)` pairs. `#[serde(default)]` for back-compat.
    #[serde(default)]
    aliases: Vec<([u8; ID_LEN], String)>,
    /// This account's claimed username. `#[serde(default)]` for back-compat.
    #[serde(default)]
    my_username: Option<String>,
    /// Queued-but-unaccepted outgoing blobs `(peer, blob, seq)`. `#[serde(default)]` for
    /// back-compat. Sealed ciphertext + ids only — but the whole snapshot is sealed anyway.
    #[serde(default)]
    outbox: Vec<([u8; ID_LEN], Vec<u8>, u64)>,
    /// Read-receipt privacy toggle. Defaults to ON for snapshots written before it existed.
    #[serde(default = "default_true")]
    send_read_receipts: bool,
    /// Pinned conversation ids. `#[serde(default)]` for back-compat.
    #[serde(default)]
    pinned: Vec<[u8; ID_LEN]>,
    /// Exported [`MlsEngine`] state (MLS group secrets — protected by this snapshot's outer
    /// seal, zeroized on drop). `#[serde(default)]` for back-compat.
    #[serde(default)]
    mls_state: Option<Vec<u8>>,
    /// `(group id, metadata)` pairs. `#[serde(default)]` for back-compat.
    #[serde(default)]
    groups: Vec<([u8; ID_LEN], GroupMeta)>,
}

impl Drop for AppSnapshot {
    fn drop(&mut self) {
        // Scrub the transient plaintext copy (the sealed blob is the protected at-rest artifact).
        // Mirrors mercury-client's `ClientSnapshot` Drop. The live controller's own log is separate.
        for msg in self.log.iter_mut() {
            msg.text.zeroize();
            msg.peer.zeroize();
            if let Some(att) = msg.att.as_mut() {
                att.name.zeroize();
                att.data_b64.zeroize();
            }
        }
        self.client_pickle.zeroize();
        if let Some(mls) = self.mls_state.as_mut() {
            mls.zeroize();
        }
    }
}

impl<T: Transport> AppController<T> {
    /// Create a controller with a fresh identity over `transport`.
    pub fn new(transport: T) -> Self {
        Self {
            client: MercuryClient::new(),
            transport,
            contacts: HashMap::new(),
            log: Vec::new(),
            seq: 0,
            last_receive_ok: true,
            muted: HashSet::new(),
            blocked: HashSet::new(),
            disappearing: HashMap::new(),
            kt_vrf_pin: None,
            kt_log_pin: None,
            verified: HashSet::new(),
            aliases: HashMap::new(),
            my_username: None,
            outbox: Vec::new(),
            dirty_peers: HashSet::new(),
            read_sent: HashMap::new(),
            pending_chunks: HashMap::new(),
            state_dirty: false,
            send_read_receipts: true,
            pinned: HashSet::new(),
            mls: None,
            groups: HashMap::new(),
        }
    }

    /// Seal this controller's FULL state (identity, device, every ratchet session, the verified
    /// contacts, and the message log) into a single at-rest blob under `key`, so the same account
    /// and conversations survive a restart. The crypto state is captured by
    /// [`MercuryClient::to_encrypted_pickle`]; the whole snapshot is then sealed (the message log is
    /// plaintext, so this outer seal is what protects it at rest). The serialized plaintext is
    /// zeroized before return. No crypto lives in this crate — the seal is `mercury_client::seal_blob`
    /// (XChaCha20-Poly1305). `key` must come from a real device secret (the OS keychain); this crate
    /// never persists it.
    pub fn to_encrypted_snapshot(&self, key: &[u8; 32]) -> Vec<u8> {
        let snapshot = AppSnapshot {
            client_pickle: self.client.to_encrypted_pickle(key),
            contacts: self
                .contacts
                .iter()
                .map(|(id, c)| (*id, c.clone()))
                .collect(),
            log: self.log.clone(),
            seq: self.seq,
            last_receive_ok: self.last_receive_ok,
            muted: self.muted.iter().copied().collect(),
            blocked: self.blocked.iter().copied().collect(),
            disappearing: self.disappearing.iter().map(|(k, v)| (*k, *v)).collect(),
            kt_vrf_pin: self.kt_vrf_pin.clone(),
            kt_log_pin: self.kt_log_pin,
            verified: self.verified.iter().copied().collect(),
            aliases: self.aliases.iter().map(|(k, v)| (*k, v.clone())).collect(),
            my_username: self.my_username.clone(),
            outbox: self.outbox.clone(),
            send_read_receipts: self.send_read_receipts,
            pinned: self.pinned.iter().copied().collect(),
            mls_state: self
                .mls
                .as_ref()
                .and_then(|engine| engine.export_state().ok()),
            groups: self
                .groups
                .iter()
                .map(|(id, meta)| (*id, meta.clone()))
                .collect(),
        };
        let mut plain = serde_json::to_vec(&snapshot).expect("an app snapshot always serializes");
        let blob = seal_blob(key, &plain);
        plain.zeroize();
        blob
    }

    /// Restore a controller from [`AppController::to_encrypted_snapshot`], over `transport`. Fails
    /// closed ([`AppError::Persistence`]) on the wrong `key`, a corrupt/foreign/tampered blob, or a
    /// malformed snapshot — never a partial restore. The decrypted plaintext is zeroized.
    pub fn from_encrypted_snapshot(
        transport: T,
        blob: &[u8],
        key: &[u8; 32],
    ) -> Result<Self, AppError> {
        let mut plain = open_blob(key, blob).map_err(|_| AppError::Persistence)?;
        let snapshot: AppSnapshot =
            serde_json::from_slice(&plain).map_err(|_| AppError::Persistence)?;
        plain.zeroize();
        let client = MercuryClient::from_encrypted_pickle(&snapshot.client_pickle, key)
            .map_err(|_| AppError::Persistence)?;
        // Clone (not move) out of `snapshot` so its `Drop` still runs and scrubs the plaintext copy.
        let controller = Self {
            client,
            transport,
            contacts: snapshot
                .contacts
                .iter()
                .map(|(id, c)| (*id, c.clone()))
                .collect(),
            log: snapshot.log.clone(),
            seq: snapshot.seq,
            last_receive_ok: snapshot.last_receive_ok,
            muted: snapshot.muted.iter().copied().collect(),
            blocked: snapshot.blocked.iter().copied().collect(),
            disappearing: snapshot.disappearing.iter().map(|&(k, v)| (k, v)).collect(),
            kt_vrf_pin: snapshot.kt_vrf_pin.clone(),
            kt_log_pin: snapshot.kt_log_pin,
            verified: snapshot.verified.iter().copied().collect(),
            aliases: snapshot
                .aliases
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
            my_username: snapshot.my_username.clone(),
            outbox: snapshot.outbox.clone(),
            dirty_peers: HashSet::new(),
            read_sent: HashMap::new(),
            pending_chunks: HashMap::new(),
            state_dirty: false,
            send_read_receipts: snapshot.send_read_receipts,
            pinned: snapshot.pinned.iter().copied().collect(),
            // Restore the MLS engine fail-closed: a present-but-unopenable state is a corrupt
            // snapshot (the outer seal already authenticated the bytes, so this should never
            // fire in practice) — surface it rather than silently dropping every group.
            mls: match snapshot.mls_state.as_deref() {
                Some(bytes) => {
                    Some(MlsEngine::import_state(bytes).map_err(|_| AppError::Persistence)?)
                }
                None => None,
            },
            groups: snapshot
                .groups
                .iter()
                .map(|(id, meta)| (*id, meta.clone()))
                .collect(),
        };
        Ok(controller)
    }

    /// This account's id as lowercase hex (what a peer adds to reach us).
    pub fn account_id(&self) -> String {
        hex_encode(&self.client.account_id())
    }

    /// This account's id as raw bytes.
    pub fn account_id_bytes(&self) -> [u8; ID_LEN] {
        self.client.account_id()
    }

    /// Our route (= account id) plus a fresh route-ownership proof, for an out-of-band long-poll
    /// wait ([`Transport::wait`]) that must NOT hold the controller for its whole duration. A caller
    /// holding `AppController` behind a lock grabs this under a brief lock, releases the lock, then
    /// waits with a separate transport handle — so a multi-second wait never blocks send/poll. The
    /// proof is stamped with the relay-synced clock and stays fresh well within the relay's window.
    /// `None` before an identity exists. Read-only: signing does not mutate state.
    pub fn poll_auth(&self) -> Option<([u8; ID_LEN], PollAuth)> {
        let me = self.client.account_id();
        if me == [0u8; ID_LEN] {
            return None;
        }
        Some((me, self.client.sign_poll_auth(self.transport.proof_time())))
    }

    /// Publish our contact card to the relay directory so peers can discover us by account id.
    pub fn publish_self(&mut self) -> Result<(), AppError> {
        self.client
            .publish_to_directory(&self.transport)
            .map_err(|e| AppError::Transport(e.to_string()))
    }

    /// Fetch and VERIFY a peer's contact card from the directory and remember it, so we can
    /// open a conversation. Fails closed if the directory has no card, the card does not
    /// verify, or its account id does not match the one requested.
    pub fn add_contact(&mut self, account_id_hex: &str) -> Result<(), AppError> {
        let id = parse_id(account_id_hex)?;
        let card = self.client.fetch_contact(&self.transport, &id)?;
        self.contacts.insert(id, card);
        Ok(())
    }

    /// Whether we can already reach `peer_hex` — either an established session or a known
    /// contact card. Returns false for a malformed id.
    pub fn can_send(&self, peer_hex: &str) -> bool {
        match parse_id(peer_hex) {
            Ok(id) => {
                self.client.has_session(&id)
                    || self.contacts.contains_key(&id)
                    // A live (not-ended) group is sendable: the "contact" is the MLS group itself.
                    || self.groups.get(&id).is_some_and(|g| !g.ended)
            }
            Err(_) => false,
        }
    }

    /// Compute the mutual identity safety number with `peer_hex` for OUT-OF-BAND verification.
    /// Returns `{ "digits": <60 chars>, "grouped": <12 space-separated groups of 5> }`. Both
    /// parties derive the SAME number; comparing them over a trusted channel (in person, or a
    /// recognizable voice/video call) confirms there is no man-in-the-middle on the identity keys
    /// — the human backstop behind key transparency. Requires a stored, verified contact card for
    /// the peer (call [`AppController::add_contact`] first); fails closed otherwise.
    pub fn safety_number(&self, peer_hex: &str) -> Result<Value, AppError> {
        let id = parse_id(peer_hex)?;
        let card = self.contacts.get(&id).ok_or(AppError::NoContact)?;
        let sn = self.client.safety_number(card).ok_or(AppError::NoContact)?;
        Ok(json!({ "digits": sn.digits(), "grouped": sn.grouped() }))
    }

    /// Mint a short-lived, single-use PAIRING CODE that brokers our contact card, so a peer can add
    /// us by typing a human-friendly code instead of our 64-hex account id. Returns
    /// `{ "code": "<grouped code>" }`. The code expires in a few minutes and works once.
    pub fn pairing_code(&mut self) -> Result<Value, AppError> {
        let code = self
            .client
            .publish_pairing(&self.transport)
            .map_err(|e| match e {
                ClientError::Transport => AppError::Transport("pairing broker unreachable".into()),
                other => AppError::Client(other),
            })?;
        Ok(json!({ "code": code }))
    }

    /// Add a contact from a PAIRING CODE: fetch + verify the brokered card and remember it. Returns
    /// `{ "account_id": "<hex>" }` (who was added) so the UI can confirm. Fails closed if the code
    /// is unknown/expired/used or the brokered card does not self-authenticate.
    pub fn add_by_pairing(&mut self, code: &str) -> Result<Value, AppError> {
        let card = self.client.fetch_by_pairing(&self.transport, code)?;
        let id = card.account_id();
        self.contacts.insert(id, card);
        Ok(json!({ "account_id": hex_encode(&id) }))
    }

    /// Claim a USERNAME (key-transparency-backed handle) for this account, first-claimant-wins.
    /// Returns `{ "status": "created" | "already_owned" | "taken" | "rejected" | "unavailable",
    /// "username": "<normalized>" }`. `rejected` means the handle is illegal (charset/length) or the
    /// proof failed; `taken` means another account already holds it.
    pub fn claim_username(&mut self, username: &str) -> Result<Value, AppError> {
        let outcome = self
            .client
            .claim_username(&self.transport, username)
            .map_err(|e| match e {
                ClientError::Transport => {
                    AppError::Transport("username registry unreachable".into())
                }
                other => AppError::Client(other),
            })?;
        // Echo the NORMALIZED handle (trim + lowercase = the registry's canonical form for any valid
        // handle), so the UI names what was actually claimed ("@alice", not the typed "@Alice").
        let normalized = username.trim().to_ascii_lowercase();
        let status = match outcome {
            UsernameClaimOutcome::Created => "created",
            UsernameClaimOutcome::AlreadyOwned => "already_owned",
            UsernameClaimOutcome::Taken => "taken",
            UsernameClaimOutcome::Rejected => "rejected",
            UsernameClaimOutcome::Unavailable => "unavailable",
        };
        // Remember OUR claimed handle (display-only) so the UI can show "@alice" in the rail.
        if matches!(
            outcome,
            UsernameClaimOutcome::Created | UsernameClaimOutcome::AlreadyOwned
        ) {
            self.my_username = Some(normalized.clone());
        }
        Ok(json!({ "status": status, "username": normalized }))
    }

    /// Add a contact by USERNAME via key transparency: resolve the handle to an account id, VERIFY
    /// the inclusion proof against our pinned directory VRF key (pinning it on first use), then fetch
    /// + verify the contact card and remember it. Returns `{ "account_id": "<hex>", "tofu": <bool> }`
    /// — `tofu` is true when this resolve trusted the relay-served key on first use (the UI should
    /// surface that). Fails closed if the handle is unknown, the proof does not verify (a forged or
    /// equivocating binding), or no directory key is available.
    pub fn add_by_username(&mut self, username: &str) -> Result<Value, AppError> {
        let resolved = self
            .client
            .resolve_username(
                &self.transport,
                username,
                self.kt_vrf_pin.as_deref(),
                self.kt_log_pin.as_ref(),
            )
            .map_err(|e| match e {
                ClientError::Transport => {
                    AppError::Transport("username registry unreachable".into())
                }
                other => AppError::Client(other),
            })?;
        // Trust-on-first-use: pin the key the proof verified against, so a later rotation is caught.
        if self.kt_vrf_pin.is_none() {
            self.kt_vrf_pin = Some(resolved.vrf_used.clone());
        }
        let id = resolved.account_id;
        self.contacts.insert(id, resolved.card);
        // Remember the proof-verified handle as this contact's display alias ("@bob", not 64 hex).
        // resolve_username already validated + normalized the real lookup; this trims the user's
        // typed form to the same canonical shape for display.
        self.aliases
            .insert(id, username.trim().to_ascii_lowercase());
        Ok(json!({ "account_id": hex_encode(&id), "tofu": resolved.tofu }))
    }

    /// Pin the username-directory VRF key OUT-OF-BAND if none is pinned yet. Called by the desktop
    /// shell at startup with the key BAKED INTO THE SIGNED BUILD for the default relay — closing
    /// the trust-on-first-use bootstrap for default-relay users (a malicious-from-first-contact
    /// relay can no longer serve its own key; the proof fails against the baked pin). A custom
    /// relay (MERCURY_RELAY_URL) does NOT get the baked pin and stays TOFU, honestly.
    pub fn pin_kt_vrf_if_absent(&mut self, vrf_key: &[u8]) {
        if self.kt_vrf_pin.is_none() && !vrf_key.is_empty() {
            self.kt_vrf_pin = Some(vrf_key.to_vec());
        }
    }

    /// Pin the directory's LOG public key out-of-band if none is pinned yet (same trust story as
    /// [`AppController::pin_kt_vrf_if_absent`] — shipped inside the signed build).
    pub fn pin_kt_log_if_absent(&mut self, log_key: &[u8; ID_LEN]) {
        if self.kt_log_pin.is_none() {
            self.kt_log_pin = Some(*log_key);
        }
    }

    /// Mark `peer_hex` VERIFIED: the user attests they compared safety numbers over a trusted
    /// channel. Local + persisted; display-only (grants no capability). Fails on a malformed id.
    pub fn set_verified(&mut self, peer_hex: &str) -> Result<(), AppError> {
        let id = parse_id(peer_hex)?;
        self.verified.insert(id);
        Ok(())
    }

    /// Clear the VERIFIED mark (e.g. after a safety-number change warning).
    pub fn unset_verified(&mut self, peer_hex: &str) -> Result<(), AppError> {
        let id = parse_id(peer_hex)?;
        self.verified.remove(&id);
        Ok(())
    }

    /// The user is LOOKING at `peer_hex`'s 1:1 conversation (open + focused): if our read-receipt
    /// toggle is on and there are newly-seen incoming messages, send the peer a `read` watermark
    /// over the sealed channel (best-effort, silent). Groups are excluded v1 (an N×N receipt
    /// storm — disclosed in docs/GROUPS.md).
    pub fn mark_read(&mut self, peer_hex: &str) -> Result<(), AppError> {
        let peer = parse_id(peer_hex)?;
        if !self.send_read_receipts || self.groups.contains_key(&peer) {
            return Ok(());
        }
        let latest_in = self
            .log
            .iter()
            .filter(|m| m.peer == *peer_hex && m.direction == Direction::Incoming && m.ts > 0)
            .map(|m| m.ts)
            .max();
        let Some(upto) = latest_in else {
            return Ok(());
        };
        // Only signal when the watermark advanced (no chatter on every focus event).
        if self.read_sent.get(&peer).copied().unwrap_or(0) >= upto {
            return Ok(());
        }
        if self.client.has_session(&peer) {
            let mut payload = CONTROL_SENTINEL.to_vec();
            payload.extend_from_slice(
                &serde_json::to_vec(&json!({ "op": "read", "upto": upto }))
                    .expect("control json serializes"),
            );
            if let Ok(blob) = self.client.send(&peer, &payload) {
                // `client.send` advanced the outbound ratchet — DURABLE state. mark_read is not in
                // the shell's "mutating command" set (it is usually a no-op on focus), so flag the
                // change explicitly: a read-then-quit MUST persist the advance, or on restart the
                // next outbound message would reuse a consumed ratchet index and desync the peer.
                self.state_dirty = true;
                if self.transport.submit(&peer, &blob).is_ok() {
                    self.read_sent.insert(peer, upto);
                }
            }
        }
        Ok(())
    }

    /// Set the read-receipt privacy toggle (delivery acks always flow; READ signals are opt-out).
    pub fn set_read_receipts(&mut self, on: bool) {
        self.send_read_receipts = on;
    }

    /// Pin / unpin a conversation (1:1 peer or group) — a persisted display preference.
    pub fn set_pinned(&mut self, peer_hex: &str, on: bool) -> Result<(), AppError> {
        let id = parse_id(peer_hex)?;
        if on {
            self.pinned.insert(id);
        } else {
            self.pinned.remove(&id);
        }
        Ok(())
    }

    /// Send a file to `peer_hex` over the SAME sealed ratchet channel as text (in-band `att`/`attc`
    /// control frames): the relay sees only ciphertext, and there is no blob store to trust.
    /// `data_b64` is the file's base64 bytes (how it crosses the IPC boundary). A file at or under
    /// [`MAX_ATTACHMENT_BYTES`] rides one frame; larger files up to [`MAX_ATTACHMENT_TOTAL_BYTES`]
    /// (4 MiB) are chunked. Queues through the same outbox as text when the peer's mailbox is full.
    pub fn send_attachment(
        &mut self,
        peer_hex: &str,
        name: &str,
        mime: &str,
        data_b64: &str,
    ) -> Result<Value, AppError> {
        use base64::Engine as _;
        let peer = parse_id(peer_hex)?;
        let raw = base64::engine::general_purpose::STANDARD
            .decode(data_b64)
            .map_err(|_| AppError::AttachmentInvalid)?;
        if raw.is_empty() || raw.len() > MAX_ATTACHMENT_TOTAL_BYTES {
            return Err(AppError::AttachmentTooLarge);
        }
        // Files over the single-frame cap are CHUNKED: ordered `attc` frames over the same sealed
        // channel, reassembled transiently by the receiver into one attachment message.
        if raw.len() > MAX_ATTACHMENT_BYTES {
            return self.send_attachment_chunked(&peer, name, mime, &raw, data_b64);
        }
        let name = sanitize_filename(name);
        let mime = if mime.trim().is_empty() {
            "application/octet-stream".to_string()
        } else {
            mime.trim().to_string()
        };
        let mut payload = CONTROL_SENTINEL.to_vec();
        payload.extend_from_slice(
            &serde_json::to_vec(
                &json!({ "op": "att", "name": name, "mime": mime, "data": data_b64 }),
            )
            .expect("control json serializes"),
        );
        let blob = self.seal_for(&peer, &payload)?;
        let accepted = self.dispatch(&peer, blob)?;
        let digest = body_digest(&payload);
        let att = Attachment {
            name: name.clone(),
            mime,
            data_b64: data_b64.to_string(),
        };
        let msg = self.push_full(
            Direction::Outgoing,
            peer,
            format!("\u{1f4ce} {name}"),
            Some(att),
        );
        self.finish_outgoing(msg.seq, digest, accepted);
        Ok(json!({ "seq": msg.seq, "pending": !accepted }))
    }

    /// Chunked variant of [`AppController::send_attachment`] for files over the single-frame cap
    /// (up to [`MAX_ATTACHMENT_TOTAL_BYTES`]). Frames carry (transfer id, index, total); the
    /// outbox preserves their order even across a full mailbox. The sender's copy is marked
    /// delivered when the receiver's completion receipt (over the LAST chunk's body) arrives.
    fn send_attachment_chunked(
        &mut self,
        peer: &[u8; ID_LEN],
        name: &str,
        mime: &str,
        raw: &[u8],
        data_b64: &str,
    ) -> Result<Value, AppError> {
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD;
        let name = sanitize_filename(name);
        let mime = if mime.trim().is_empty() {
            "application/octet-stream".to_string()
        } else {
            mime.trim().to_string()
        };
        let mut id_bytes = [0u8; 8];
        getrandom::getrandom(&mut id_bytes).map_err(|_| AppError::AttachmentInvalid)?;
        let id = hex_encode_vec(&id_bytes);
        let chunks: Vec<&[u8]> = raw.chunks(ATTACHMENT_CHUNK_RAW).collect();
        let total = chunks.len() as u32;
        let mut any_queued = false;
        let mut last_digest = String::new();
        for (idx, chunk) in chunks.iter().enumerate() {
            let mut payload = CONTROL_SENTINEL.to_vec();
            payload.extend_from_slice(
                &serde_json::to_vec(&json!({
                    "op": "attc",
                    "id": id,
                    "name": name,
                    "mime": mime,
                    "idx": idx as u32,
                    "total": total,
                    "data": b64.encode(chunk),
                }))
                .expect("control json serializes"),
            );
            last_digest = body_digest(&payload);
            let blob = self.seal_for(peer, &payload)?;
            if !self.dispatch(peer, blob)? {
                any_queued = true;
            }
        }
        let att = Attachment {
            name: name.clone(),
            mime,
            data_b64: data_b64.to_string(),
        };
        let msg = self.push_full(
            Direction::Outgoing,
            *peer,
            format!("\u{1f4ce} {name}"),
            Some(att),
        );
        self.finish_outgoing(msg.seq, last_digest, !any_queued);
        Ok(json!({ "seq": msg.seq, "pending": any_queued }))
    }

    /// Export this account as a PASSPHRASE-SEALED backup file. Current format (`MERCBAK2`):
    /// `magic(8) || algo(1) || mem_kib(4 LE) || iters(4 LE) || lanes(4 LE) || salt(16) || sealed`,
    /// where `sealed` is the SAME audited snapshot seal used at rest (XChaCha20-Poly1305) under a
    /// key stretched from the passphrase by memory-hard Argon2id (self-describing header, fresh
    /// random salt per export). The backup contains EVERYTHING (identity, sessions, contacts,
    /// history) — its security is exactly the passphrase's strength, so a minimum length is enforced
    /// and the UI says so plainly. Older `MERCBAK1` (PBKDF2) backups still restore via
    /// [`AppController::import_backup`].
    pub fn export_backup(&self, passphrase: &str) -> Result<Vec<u8>, AppError> {
        if passphrase.len() < MIN_BACKUP_PASSPHRASE_CHARS {
            return Err(AppError::WeakPassphrase);
        }
        let mut salt = [0u8; 16];
        getrandom::getrandom(&mut salt).map_err(|_| AppError::Persistence)?;
        let mut key = derive_backup_key_argon2(
            passphrase,
            &salt,
            BACKUP_ARGON2_MEM_KIB,
            BACKUP_ARGON2_ITERS,
            BACKUP_ARGON2_LANES,
        )?;
        let sealed = self.to_encrypted_snapshot(&key);
        key.zeroize();
        // magic(8) || algo(1) || mem_kib(4) || iters(4) || lanes(4) || salt(16) || sealed
        let mut out = Vec::with_capacity(BACKUP_MAGIC_V2.len() + 29 + sealed.len());
        out.extend_from_slice(BACKUP_MAGIC_V2);
        out.push(BACKUP_KDF_ARGON2ID);
        out.extend_from_slice(&BACKUP_ARGON2_MEM_KIB.to_le_bytes());
        out.extend_from_slice(&BACKUP_ARGON2_ITERS.to_le_bytes());
        out.extend_from_slice(&BACKUP_ARGON2_LANES.to_le_bytes());
        out.extend_from_slice(&salt);
        out.extend_from_slice(&sealed);
        Ok(out)
    }

    /// Restore an account from a passphrase-sealed backup produced by
    /// [`AppController::export_backup`]. Fails closed ([`AppError::BackupInvalid`]) on a wrong
    /// passphrase, a truncated/foreign file, or any tamper — never a partial restore. The caller
    /// (the desktop shell) replaces the live controller with the returned one and re-seals it
    /// under the device key.
    pub fn import_backup(transport: T, backup: &[u8], passphrase: &str) -> Result<Self, AppError> {
        // Current format: `MERCBAK2` with a self-describing memory-hard Argon2id header.
        if let Some(rest) = backup.strip_prefix(BACKUP_MAGIC_V2) {
            // algo(1) + mem_kib(4) + iters(4) + lanes(4) + salt(16) = 29 header bytes, then sealed.
            if rest.len() < 29 || rest[0] != BACKUP_KDF_ARGON2ID {
                return Err(AppError::BackupInvalid);
            }
            let mem_kib = u32::from_le_bytes([rest[1], rest[2], rest[3], rest[4]]);
            let iters = u32::from_le_bytes([rest[5], rest[6], rest[7], rest[8]]);
            let lanes = u32::from_le_bytes([rest[9], rest[10], rest[11], rest[12]]);
            // Fail CLOSED on absurd parameters (a hostile/corrupt file): refuse rather than let
            // Argon2id allocate gigabytes on restore.
            if mem_kib > BACKUP_ARGON2_MAX_MEM_KIB
                || !(1..=BACKUP_ARGON2_MAX_ITERS).contains(&iters)
                || !(1..=BACKUP_ARGON2_MAX_LANES).contains(&lanes)
            {
                return Err(AppError::BackupInvalid);
            }
            let salt: [u8; 16] = rest[13..29]
                .try_into()
                .map_err(|_| AppError::BackupInvalid)?;
            let sealed = &rest[29..];
            let mut key = derive_backup_key_argon2(passphrase, &salt, mem_kib, iters, lanes)?;
            let restored = Self::from_encrypted_snapshot(transport, sealed, &key)
                .map_err(|_| AppError::BackupInvalid);
            key.zeroize();
            return restored;
        }
        // Legacy format: `MERCBAK1` (PBKDF2-HMAC-SHA512) — kept so backups from older builds restore.
        if let Some(rest) = backup.strip_prefix(BACKUP_MAGIC) {
            if rest.len() < 16 {
                return Err(AppError::BackupInvalid);
            }
            let (salt, sealed) = rest.split_at(16);
            let salt: [u8; 16] = salt.try_into().map_err(|_| AppError::BackupInvalid)?;
            let mut key = derive_backup_key(passphrase, &salt);
            let restored = Self::from_encrypted_snapshot(transport, sealed, &key)
                .map_err(|_| AppError::BackupInvalid);
            key.zeroize();
            return restored;
        }
        Err(AppError::BackupInvalid)
    }

    /// Delete a conversation LOCALLY ("delete for me"): drop its messages from the log AND the
    /// stored contact card, so it leaves the conversation list. Idempotent. The peer is NOT
    /// notified and keeps their own copy (use a delete-for-everyone control message to also remove
    /// it from their device). The ratchet session, if one exists, is left intact — a later message
    /// from the peer simply re-opens the conversation rather than failing to decrypt.
    pub fn delete_conversation(&mut self, peer_hex: &str) -> Result<(), AppError> {
        let id = parse_id(peer_hex)?;
        // Deleting a GROUP conversation = leaving the group (notifies the creator so the epoch
        // rotates) — there is no "keep membership but hide it" state to fake.
        if self.groups.contains_key(&id) {
            return self.leave_group(peer_hex);
        }
        let key = hex_encode(&id);
        self.log.retain(|m| m.peer != key);
        self.contacts.remove(&id);
        Ok(())
    }

    /// Delete a conversation on BOTH sides ("delete for everyone"): send the peer a sealed,
    /// forward-secret CONTROL message that wipes the shared message history on their device (and
    /// leaves a "deleted by the other person" notice), then delete the conversation locally here.
    /// The control rides the same ratchet as a normal message, so the relay only ever sees
    /// ciphertext. Fails closed if the peer is blocked or unreachable — NOTHING is deleted, so the
    /// user can retry or fall back to the local-only [`AppController::delete_conversation`].
    pub fn delete_for_everyone(&mut self, peer_hex: &str) -> Result<(), AppError> {
        let peer = parse_id(peer_hex)?;
        if self.blocked.contains(&peer) {
            return Err(AppError::Blocked);
        }
        let mut payload = CONTROL_SENTINEL.to_vec();
        payload.extend_from_slice(
            &serde_json::to_vec(&json!({ "op": "delete" })).expect("control json serializes"),
        );
        let blob = if self.client.has_session(&peer) {
            self.client.send(&peer, &payload)?
        } else {
            let card = self.contacts.get(&peer).ok_or(AppError::NoContact)?;
            let (_route, blob) = self.client.initiate(card, &payload)?;
            blob
        };
        self.transport
            .submit(&peer, &blob)
            .map_err(|e| AppError::Transport(e.to_string()))?;
        // The remote control is on its way; now remove the conversation locally.
        let key = hex_encode(&peer);
        self.log.retain(|m| m.peer != key);
        self.contacts.remove(&peer);
        Ok(())
    }

    /// Best-effort "delete for everyone, everywhere": send a delete-for-everyone control to EVERY
    /// 1:1 conversation and end/leave EVERY group, so copies on REACHABLE peers' devices are removed
    /// too. The account-delete flow calls this for the "also wipe from peers" scope, immediately
    /// before the local erase. Every send is best-effort — an offline relay or a blocked peer is
    /// tallied, never fatal: the caller erases this device regardless, because the user asked to
    /// delete. (Keys are snapshotted first because the per-conversation calls mutate `contacts` /
    /// `groups` as they go.) It cannot guarantee removal — a peer who is offline forever, or who
    /// kept their own copy, is beyond our reach, and the UI says so plainly.
    pub fn delete_for_everyone_all(&mut self) -> Value {
        let peers: Vec<String> = self.contacts.keys().map(|id| hex_encode(id)).collect();
        let groups: Vec<String> = self.groups.keys().map(|id| hex_encode(id)).collect();
        let mut peers_wiped = 0u32;
        let mut groups_ended = 0u32;
        let mut failures = 0u32;
        for p in &peers {
            match self.delete_for_everyone(p) {
                Ok(()) => peers_wiped += 1,
                Err(_) => failures += 1,
            }
        }
        for g in &groups {
            match self.leave_group(g) {
                Ok(()) => groups_ended += 1,
                Err(_) => failures += 1,
            }
        }
        json!({ "peers_wiped": peers_wiped, "groups_ended": groups_ended, "failures": failures })
    }

    /// Apply an in-band control message from `from` (the bytes AFTER [`CONTROL_SENTINEL`]). Returns
    /// `true` iff a delete-for-everyone was applied (the conversation's history was wiped), so the
    /// poll loop can surface a single notice line. Unknown / malformed ops are ignored — a
    /// forward-compatible no-op, never a panic.
    /// Apply one in-band control frame from `from`. Returns the ChatMessage to SURFACE for it, if
    /// any (a delete notice, an attachment bubble); silent ops (timer, receipt) and unknown ops
    /// (forward-compat: newer peers may send frames we don't know yet) return None and are
    /// dropped without rendering anything.
    fn handle_control(&mut self, from: &[u8; ID_LEN], rest: &[u8]) -> Option<ChatMessage> {
        let Ok(Value::Object(obj)) = serde_json::from_slice::<Value>(rest) else {
            return None;
        };
        // Group ops (g-prefixed) have their own handler + trust rules.
        if let Some(op) = obj.get("op").and_then(Value::as_str) {
            if op.starts_with('g') {
                return self.handle_group_control(from, op, &obj);
            }
        }
        match obj.get("op").and_then(Value::as_str) {
            Some("delete") => {
                let key = hex_encode(from);
                self.log.retain(|m| m.peer != key);
                Some(self.push(
                    Direction::Incoming,
                    *from,
                    "\u{1f5d1}\u{fe0f} The other person deleted this conversation.".to_string(),
                ))
            }
            Some("timer") => {
                let secs = obj.get("secs").and_then(Value::as_u64).unwrap_or(0);
                if secs == 0 {
                    self.disappearing.remove(from);
                } else {
                    self.disappearing.insert(*from, secs);
                }
                None
            }
            // Read watermark: the peer's device reports it has READ our 1:1 messages up to
            // `upto` (a wall-clock watermark; cross-device clock skew can shade it by seconds —
            // cosmetic, never security-relevant). Only DELIVERED messages can become read; the
            // peer only sends this when their read-receipt toggle is on.
            Some("read") => {
                let upto = obj.get("upto").and_then(Value::as_i64).unwrap_or(0);
                if upto > 0 {
                    let key = hex_encode(from);
                    let mut changed = false;
                    for m in self.log.iter_mut().filter(|m| {
                        m.peer == key
                            && m.direction == Direction::Outgoing
                            && m.delivered
                            && !m.read
                            && m.ts > 0
                            && m.ts <= upto
                    }) {
                        m.read = true;
                        changed = true;
                    }
                    if changed {
                        self.dirty_peers.insert(key);
                    }
                }
                None
            }
            // Delivery receipt: the peer's device confirmed it received the message whose body
            // hashes to `digest`. Mark the OLDEST matching undelivered outgoing copy (duplicates
            // hash identically; oldest-first keeps repeated texts converging) and flag the
            // conversation dirty so the UI refreshes its history.
            Some("rcpt") => {
                let digest = obj.get("digest").and_then(Value::as_str).unwrap_or("");
                if !digest.is_empty() {
                    let key = hex_encode(from);
                    if let Some(m) = self.log.iter_mut().find(|m| {
                        m.peer == key
                            && m.direction == Direction::Outgoing
                            && !m.delivered
                            && m.digest == digest
                    }) {
                        m.delivered = true;
                        self.dirty_peers.insert(key);
                    }
                }
                None
            }
            // Attachment: a small file that rode the sealed channel. Logged as a real incoming
            // message with the attachment payload; the filename is sanitized to its final path
            // component so a hostile name can never traverse directories on save.
            Some("att") => {
                let name = sanitize_filename(obj.get("name").and_then(Value::as_str).unwrap_or(""));
                let mime = obj
                    .get("mime")
                    .and_then(Value::as_str)
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let data_b64 = obj.get("data").and_then(Value::as_str).unwrap_or("");
                // Fail closed on malformed/oversized payloads: base64 must decode and the raw
                // size must respect the cap (a peer running modified software does not get to
                // bloat our snapshot).
                use base64::Engine as _;
                let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(data_b64) else {
                    return None;
                };
                if raw.is_empty() || raw.len() > MAX_ATTACHMENT_BYTES {
                    return None;
                }
                let att = Attachment {
                    name: name.clone(),
                    mime,
                    data_b64: data_b64.to_string(),
                };
                Some(self.push_full(
                    Direction::Incoming,
                    *from,
                    format!("\u{1f4ce} {name}"),
                    Some(att),
                ))
            }
            // One frame of a CHUNKED attachment. Reassembled transiently under strict caps:
            // per-transfer total <= MAX_ATTACHMENT_TOTAL_BYTES, at most 2 concurrent transfers
            // per sender, stale partials TTL-evicted. Every malformed frame drops the whole
            // transfer (fail closed) — never a partial/garbled file.
            Some("attc") => {
                use base64::Engine as _;
                let now = now_unix_s();
                // TTL-evict stale partials (cheap: the map is tiny by construction).
                self.pending_chunks
                    .retain(|_, p| now.saturating_sub(p.started_s) < ATTACHMENT_REASSEMBLY_TTL_S);
                let id = obj.get("id").and_then(Value::as_str)?.to_string();
                if id.len() > 32 {
                    return None;
                }
                let idx = obj.get("idx").and_then(Value::as_u64)? as u32;
                let total = obj.get("total").and_then(Value::as_u64)? as u32;
                let max_chunks = (MAX_ATTACHMENT_TOTAL_BYTES / ATTACHMENT_CHUNK_RAW + 1) as u32;
                if total == 0 || total > max_chunks || idx >= total {
                    return None;
                }
                let Ok(chunk) = base64::engine::general_purpose::STANDARD
                    .decode(obj.get("data").and_then(Value::as_str)?)
                else {
                    self.pending_chunks.remove(&(*from, id));
                    return None;
                };
                if chunk.is_empty() || chunk.len() > ATTACHMENT_CHUNK_RAW {
                    self.pending_chunks.remove(&(*from, id));
                    return None;
                }
                let key = (*from, id);
                if !self.pending_chunks.contains_key(&key) {
                    // Global ceiling across ALL senders (aggregate memory bound), so many minted
                    // identities can't each open transfers to balloon memory past the per-sender cap.
                    if self.pending_chunks.len() >= MAX_CONCURRENT_REASSEMBLIES {
                        return None;
                    }
                    // Cap concurrent transfers per sender (hostile-peer memory bound).
                    let inflight = self
                        .pending_chunks
                        .keys()
                        .filter(|(sender, _)| sender == from)
                        .count();
                    if inflight >= 2 {
                        return None;
                    }
                    self.pending_chunks.insert(
                        key.clone(),
                        PartialAttachment {
                            name: sanitize_filename(
                                obj.get("name").and_then(Value::as_str).unwrap_or(""),
                            ),
                            mime: obj
                                .get("mime")
                                .and_then(Value::as_str)
                                .unwrap_or("application/octet-stream")
                                .to_string(),
                            total,
                            parts: HashMap::new(),
                            bytes: 0,
                            started_s: now,
                        },
                    );
                }
                let complete = {
                    let partial = self.pending_chunks.get_mut(&key)?;
                    if partial.total != total || partial.parts.contains_key(&idx) {
                        // Inconsistent framing or a duplicate index: drop the transfer.
                        self.pending_chunks.remove(&key);
                        return None;
                    }
                    partial.bytes += chunk.len();
                    if partial.bytes > MAX_ATTACHMENT_TOTAL_BYTES {
                        self.pending_chunks.remove(&key);
                        return None;
                    }
                    partial.parts.insert(idx, chunk);
                    partial.parts.len() as u32 == partial.total
                };
                if !complete {
                    return None;
                }
                let partial = self.pending_chunks.remove(&key)?;
                let mut assembled = Vec::with_capacity(partial.bytes);
                for i in 0..partial.total {
                    assembled.extend_from_slice(partial.parts.get(&i)?);
                }
                let att = Attachment {
                    name: partial.name.clone(),
                    mime: partial.mime,
                    data_b64: base64::engine::general_purpose::STANDARD.encode(&assembled),
                };
                Some(self.push_full(
                    Direction::Incoming,
                    *from,
                    format!("\u{1f4ce} {}", partial.name),
                    Some(att),
                ))
            }
            _ => None,
        }
    }

    /// Clear a conversation's message history while KEEPING the contact (the connection stays; only
    /// the stored messages go). Idempotent. Local-only — the peer's copy is unaffected.
    pub fn clear_history(&mut self, peer_hex: &str) -> Result<(), AppError> {
        let id = parse_id(peer_hex)?;
        let key = hex_encode(&id);
        self.log.retain(|m| m.peer != key);
        Ok(())
    }

    /// Mute a conversation — the UI suppresses notifications for `peer_hex`. Persisted, idempotent.
    /// Delivery is unaffected (messages still arrive and log).
    pub fn mute(&mut self, peer_hex: &str) -> Result<(), AppError> {
        self.muted.insert(parse_id(peer_hex)?);
        Ok(())
    }

    /// Unmute a conversation. Idempotent.
    pub fn unmute(&mut self, peer_hex: &str) -> Result<(), AppError> {
        self.muted.remove(&parse_id(peer_hex)?);
        Ok(())
    }

    /// Block a contact: future sends to them are refused ([`AppError::Blocked`]) and incoming
    /// messages from them are dropped on poll (decrypted then discarded — the ratchet still
    /// advances, so unblocking resumes cleanly). Persisted, idempotent, reversible via `unblock`.
    pub fn block(&mut self, peer_hex: &str) -> Result<(), AppError> {
        self.blocked.insert(parse_id(peer_hex)?);
        Ok(())
    }

    /// Unblock a contact, restoring send + receive. Idempotent.
    pub fn unblock(&mut self, peer_hex: &str) -> Result<(), AppError> {
        self.blocked.remove(&parse_id(peer_hex)?);
        Ok(())
    }

    /// Set (or clear, with `secs == 0`) the per-conversation disappearing-message TTL for
    /// `peer_hex`, and tell the peer via a sealed `timer` control message so BOTH sides expire on
    /// the same schedule. The local setting is applied first; the control is best-effort over the
    /// same forward-secret ratchet (only attempted for a reachable, non-blocked peer). Existing
    /// messages with no timestamp never auto-expire; the timer applies going forward.
    pub fn set_disappearing(&mut self, peer_hex: &str, secs: u64) -> Result<(), AppError> {
        let id = parse_id(peer_hex)?;
        if secs == 0 {
            self.disappearing.remove(&id);
        } else {
            self.disappearing.insert(id, secs);
        }
        if !self.blocked.contains(&id) {
            let mut payload = CONTROL_SENTINEL.to_vec();
            payload.extend_from_slice(
                &serde_json::to_vec(&json!({ "op": "timer", "secs": secs }))
                    .expect("control json serializes"),
            );
            let blob = if self.client.has_session(&id) {
                Some(self.client.send(&id, &payload)?)
            } else if let Some(card) = self.contacts.get(&id) {
                Some(self.client.initiate(card, &payload)?.1)
            } else {
                None
            };
            if let Some(blob) = blob {
                self.transport
                    .submit(&id, &blob)
                    .map_err(|e| AppError::Transport(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// The per-conversation flags the UI renders: muted/blocked/verified id lists, disappearing
    /// TTLs, username aliases (peer hex -> handle), and this account's own claimed handle.
    pub fn conversation_flags(&self) -> Value {
        let muted: Vec<String> = self.muted.iter().map(|id| hex_encode(id)).collect();
        let blocked: Vec<String> = self.blocked.iter().map(|id| hex_encode(id)).collect();
        let verified: Vec<String> = self.verified.iter().map(|id| hex_encode(id)).collect();
        let disappearing: serde_json::Map<String, Value> = self
            .disappearing
            .iter()
            .map(|(id, &secs)| (hex_encode(id), json!(secs)))
            .collect();
        let aliases: serde_json::Map<String, Value> = self
            .aliases
            .iter()
            .map(|(id, handle)| (hex_encode(id), json!(handle)))
            .collect();
        let groups: serde_json::Map<String, Value> = self
            .groups
            .iter()
            .map(|(id, meta)| {
                (
                    hex_encode(id),
                    json!({
                        "name": meta.name,
                        "creator": hex_encode(&meta.creator),
                        "members": meta.members.iter().map(|m| hex_encode(m)).collect::<Vec<_>>(),
                        "pending": meta.pending.iter().map(|m| hex_encode(m)).collect::<Vec<_>>(),
                        "ended": meta.ended,
                    }),
                )
            })
            .collect();
        let pinned: Vec<String> = self.pinned.iter().map(|id| hex_encode(id)).collect();
        json!({
            "muted": muted,
            "blocked": blocked,
            "verified": verified,
            "disappearing": disappearing,
            "aliases": aliases,
            "my_username": self.my_username,
            "groups": groups,
            "pinned": pinned,
            "read_receipts": self.send_read_receipts,
        })
    }

    /// Send `text` to `peer_hex`. Opens the session from the stored contact card on the first
    /// message, then rides the established ratchet for the rest. The sealed blob is submitted
    /// to the peer's route and the outgoing message is logged. Fails closed (no session and no
    /// contact card -> [`AppError::NoContact`]).
    pub fn send(&mut self, peer_hex: &str, text: &str) -> Result<(), AppError> {
        let peer = parse_id(peer_hex)?;
        let body = text.as_bytes().to_vec();
        let blob = self.seal_for(&peer, &body)?;
        let accepted = self.dispatch(&peer, blob)?;
        let digest = body_digest(&body);
        let msg = self.push(Direction::Outgoing, peer, text.to_string());
        self.finish_outgoing(msg.seq, digest, accepted);
        Ok(())
    }

    /// Seal `body` to `peer` on the established ratchet (or open the session from the stored card
    /// on the first message). Fails closed: blocked peer, or no session AND no card.
    fn seal_for(&mut self, peer: &[u8; ID_LEN], body: &[u8]) -> Result<Vec<u8>, AppError> {
        if self.blocked.contains(peer) {
            return Err(AppError::Blocked);
        }
        if self.client.has_session(peer) {
            Ok(self.client.send(peer, body)?)
        } else {
            let card = self.contacts.get(peer).ok_or(AppError::NoContact)?;
            let (_route, blob) = self.client.initiate(card, body)?;
            Ok(blob)
        }
    }

    /// Submit a sealed blob, or QUEUE it in the FIFO outbox when it cannot be delivered right now.
    /// Returns `Ok(true)` = the relay accepted it; `Ok(false)` = queued (the peer's single-slot
    /// mailbox still holds an undelivered item, or the relay is unreachable — store-and-forward);
    /// `Err` only on a hard relay rejection (malformed/oversized — a real failure to surface).
    /// ORDER MATTERS for the ratchet: if anything is already queued for this peer, the new blob
    /// goes BEHIND it unconditionally (never submitted out of order).
    fn dispatch(&mut self, peer: &[u8; ID_LEN], blob: Vec<u8>) -> Result<bool, AppError> {
        let queued_behind = self.outbox.iter().any(|(p, _, _)| p == peer);
        if queued_behind {
            self.outbox.push((*peer, blob, self.seq));
            return Ok(false);
        }
        match self.transport.submit(peer, &blob) {
            Ok(()) => Ok(true),
            // The recipient hasn't drained their single-slot mailbox yet, or the relay is briefly
            // unreachable: hold the message and retry in order on every poll tick. The user sees
            // "pending", never a lost message or a scary error for typing two messages quickly.
            Err(TransportError::Rejected(r)) if r == "item_already_pending" => {
                self.outbox.push((*peer, blob, self.seq));
                Ok(false)
            }
            Err(TransportError::Network(_)) => {
                self.outbox.push((*peer, blob, self.seq));
                Ok(false)
            }
            Err(e) => Err(AppError::Transport(e.to_string())),
        }
    }

    /// Stamp the just-logged outgoing message (`seq`) with its receipt digest and pending state.
    fn finish_outgoing(&mut self, seq: u64, digest: String, accepted: bool) {
        if let Some(m) = self.log.iter_mut().find(|m| m.seq == seq) {
            m.digest = digest;
            m.pending = !accepted;
        }
    }

    /// Retry queued outgoing blobs IN ORDER, one peer-lane at a time. A lane whose mailbox is
    /// still full is skipped wholesale (order preserved); a network failure stops the whole pass
    /// (retried next tick). Each delivery flips its message off "pending" and marks the
    /// conversation dirty so the UI refreshes it.
    fn drain_outbox(&mut self) -> bool {
        let mut full_lanes: HashSet<[u8; ID_LEN]> = HashSet::new();
        let mut delivered = false;
        let mut i = 0;
        while i < self.outbox.len() {
            let (peer, blob, seq) = self.outbox[i].clone();
            if full_lanes.contains(&peer) || self.blocked.contains(&peer) {
                i += 1;
                continue;
            }
            match self.transport.submit(&peer, &blob) {
                Ok(()) => {
                    if let Some(m) = self.log.iter_mut().find(|m| m.seq == seq) {
                        m.pending = false;
                    }
                    self.dirty_peers.insert(hex_encode(&peer));
                    self.outbox.remove(i); // same index now holds the next entry
                    delivered = true;
                }
                Err(TransportError::Rejected(r)) if r == "item_already_pending" => {
                    full_lanes.insert(peer);
                    i += 1;
                }
                Err(_) => break, // relay unreachable — retry the whole queue next tick
            }
        }
        delivered
    }

    /// Best-effort delivery receipt for a just-received `body`: an in-band `rcpt` control frame
    /// over the same sealed channel. Cosmetic — every failure is swallowed (never queued, never
    /// surfaced); older clients silently ignore unknown control ops, so this is version-safe.
    fn send_receipt(&mut self, peer: &[u8; ID_LEN], body: &[u8]) {
        let mut payload = CONTROL_SENTINEL.to_vec();
        payload.extend_from_slice(
            &serde_json::to_vec(&json!({ "op": "rcpt", "digest": body_digest(body) }))
                .expect("control json serializes"),
        );
        if let Ok(blob) = self.client.send(peer, &payload) {
            let _ = self.transport.submit(peer, &blob);
        }
    }

    /// Poll the relay for inbound messages addressed to us, decrypt them, log them, and return
    /// the newly-received messages. A blob that fails to decrypt is SKIPPED — the ratchet
    /// rejects it and the loop moves on, never panicking and never desyncing the session
    /// (fail-closed).
    pub fn poll(&mut self) -> Result<Vec<ChatMessage>, AppError> {
        // First, retry anything the outbox is holding (order-preserving store-and-forward).
        let drained = self.drain_outbox();
        // Prune any messages past their conversation's disappearing TTL before draining new ones.
        // Pruned conversations are marked dirty so the UI refreshes them AND the shell knows this
        // poll changed state (it skips the snapshot re-seal on truly idle polls).
        let now = now_unix_s();
        let dis = self.disappearing.clone();
        let expired: Vec<String> = self
            .log
            .iter()
            .filter(|m| msg_expired(&dis, m, now))
            .map(|m| m.peer.clone())
            .collect();
        let pruned = !expired.is_empty();
        if pruned {
            self.log.retain(|m| !msg_expired(&dis, m, now));
            for peer in expired {
                self.dirty_peers.insert(peer);
            }
        }
        let me = self.client.account_id();
        // Prove we own this route so the relay authorizes draining OUR queue. One fresh proof
        // covers the whole drain (the relay accepts it for each poll within its freshness
        // window); the in-memory transport ignores it.
        let auth = self.client.sign_poll_auth(self.transport.proof_time());
        let mut fresh = Vec::new();
        // Any blob we pull from the relay mutates DURABLE crypto state even when it surfaces no
        // message: a first-flight (1:1 OR a group invite) accepts a session and consumes a one-time
        // prekey; any received frame advances the ratchet. So drain == a state change that must be
        // persisted, independent of whether `fresh`/`dirty_peers` end up non-empty.
        let mut received_any = false;
        while let Some(blob) = self
            .transport
            .poll(&me, &auth)
            .map_err(|e| AppError::Transport(e.to_string()))?
        {
            received_any = true;
            match self.client.receive(&blob) {
                Ok((from, body)) => {
                    self.last_receive_ok = true;
                    // A blocked sender's message is consumed (the ratchet has advanced) but
                    // discarded — never logged, returned, or notified.
                    if self.blocked.contains(&from) {
                        continue;
                    }
                    // An in-band CONTROL frame (delete-for-everyone, disappearing timer, delivery
                    // receipt, attachment) — never rendered as raw text. Some controls surface a
                    // message (a delete notice, an attachment bubble); most are silent.
                    if body.starts_with(CONTROL_SENTINEL) {
                        if let Some(msg) =
                            self.handle_control(&from, &body[CONTROL_SENTINEL.len()..])
                        {
                            // An attachment is a real arrival the peer expects a receipt for.
                            if msg.att.is_some() {
                                self.send_receipt(&from, &body);
                            }
                            fresh.push(msg);
                        }
                        continue;
                    }
                    let text = String::from_utf8_lossy(&body).into_owned();
                    fresh.push(self.push(Direction::Incoming, from, text));
                    // Confirm receipt over the same sealed channel (best-effort, cosmetic).
                    self.send_receipt(&from, &body);
                }
                // A blob that fails to decrypt is SKIPPED (the ratchet rejected it); record the
                // real outcome so the "last receive" decision view reflects it.
                Err(_) => self.last_receive_ok = false,
            }
        }
        // The published card carries ONE one-time inbound key; accepting a first flight consumes
        // it. Re-publish a fresh card immediately so the NEXT person who wants to reach us (e.g.
        // a fellow group member we've never talked to) finds a usable key — without this, only
        // the first-ever initiator could connect. Best-effort: a transient directory failure
        // retries on the next poll tick.
        let needed_republish = self.client.needs_republish();
        if needed_republish {
            let _ = self.publish_self();
        }
        // Record that this poll changed durable state (so an otherwise-"idle" poll still gets
        // re-sealed by the shell). Without this, a control-only first-flight — e.g. a group invite
        // that adds a session + burns a prekey + triggers a republish but returns no message and no
        // dirty conversation — would be lost on the next quit/restart, and the consumed one-time
        // prekey would silently re-open (a first-flight replay window).
        if drained || pruned || received_any || needed_republish {
            self.state_dirty = true;
        }
        Ok(fresh)
    }

    /// Conversations whose EXISTING messages changed since the last call (receipt arrived /
    /// pending message accepted), drained for the poll response. The UI refreshes these histories
    /// without counting anything as a new arrival.
    pub fn take_dirty_peers(&mut self) -> Vec<String> {
        self.dirty_peers.drain().collect()
    }

    /// Whether the most recent [`poll`](Self::poll) changed DURABLE state (session accepted, prekey
    /// consumed/republished, outbox drained, disappearing prune) — even if it surfaced no message
    /// or dirty conversation. Returns the flag and clears it. The desktop shell uses this to force a
    /// snapshot re-seal on a poll that would otherwise look idle, so silent crypto-state moves (a
    /// group invite's control-only first-flight) survive a quit/restart. Reset on read.
    pub fn take_state_dirty(&mut self) -> bool {
        std::mem::take(&mut self.state_dirty)
    }

    /// The full message log, in local order.
    pub fn log(&self) -> &[ChatMessage] {
        &self.log
    }

    /// The messages exchanged with one peer, in local order.
    pub fn history(&self, peer_hex: &str) -> Vec<ChatMessage> {
        let now = now_unix_s();
        self.log
            .iter()
            .filter(|m| m.peer == peer_hex && !msg_expired(&self.disappearing, m, now))
            .cloned()
            .collect()
    }

    /// The full log as a JSON array (the UI boundary).
    pub fn log_json(&self) -> String {
        serde_json::to_string(&self.log).expect("the message log always serializes")
    }

    /// Build the REAL `PlatformDecisionView`s for the current state via the genuine mercury-core
    /// constructors (`mercury-bindings`). These are NOT fabricated — each reflects this
    /// controller's ACTUAL state:
    ///   * `bootstrap`: we are bootstrapped iff we hold a valid (non-zero) account id / identity;
    ///   * `outbound`: the real [`Self::can_send`] for `peer` (an established session or a verified
    ///     contact card) — a peer we cannot reach yields a genuine REJECT, never a faked ACCEPT;
    ///   * `receive`: whether our last real [`Self::poll`] receive succeeded.
    ///
    /// `peer_hex` is the conversation the UI is showing; `None` means the outbound view has no
    /// target and is reported as not-sendable.
    pub fn decision_views(&self, peer_hex: Option<&str>) -> DecisionViews {
        let account_ok = self.account_id_bytes() != [0u8; ID_LEN];
        let bootstrap = mercury_bootstrap_status(ClientBootstrapDecision {
            accepted: account_ok,
            can_start_sync: account_ok,
            can_decrypt_local_store: account_ok,
            can_open_message_ui: account_ok,
            requires_sync: false,
            requires_recovery: false,
            requires_user_action: !account_ok,
            reason: if account_ok {
                ClientBootstrapReason::Accepted
            } else {
                ClientBootstrapReason::MissingAccountId
            },
        });

        let can_send = peer_hex.is_some_and(|p| self.can_send(p));
        let outbound = mercury_prepare_send(OutboundSendDecision {
            accepted: can_send,
            can_send,
            can_persist_ciphertext: can_send,
            requires_user_action: !can_send,
            reason: if can_send {
                OutboundSendReason::Accepted
            } else {
                OutboundSendReason::DeviceTrustRejected
            },
        });

        // Every message the UI shows WAS accepted by the real receive gate (that is why it is
        // shown); a skipped (undecryptable) blob flips `last_receive_ok` to false. `mercury-client`'s
        // receive error is COARSE — it does not carry the precise core reason — so a failure is
        // reported as the conservative, retry-oriented `OrderingGap` (requires_client_retry): a
        // transient "could not place/decrypt this blob, resync" state, NOT a specific policy or
        // trust DENIAL we cannot prove (which would falsely accuse the sender). The direction is
        // fail-safe — a failure can only ever render as a non-accept, never as a false ACCEPT —
        // and this coarse→reason mapping is a documented residual, not a fabrication.
        let recv_ok = self.last_receive_ok;
        let receive = mercury_accept_received_ciphertext(ClientReceiveDecision {
            accepted: recv_ok,
            can_decrypt: recv_ok,
            can_persist_ciphertext: recv_ok,
            can_expose_to_ui: recv_ok,
            requires_client_retry: !recv_ok,
            requires_user_action: false,
            reason: if recv_ok {
                ClientReceiveReason::Accepted
            } else {
                ClientReceiveReason::OrderingGap
            },
        });

        DecisionViews {
            bootstrap,
            outbound,
            receive,
        }
    }

    fn push(&mut self, direction: Direction, peer: [u8; ID_LEN], text: String) -> ChatMessage {
        self.push_full(direction, peer, text, None)
    }

    fn push_full(
        &mut self,
        direction: Direction,
        peer: [u8; ID_LEN],
        text: String,
        att: Option<Attachment>,
    ) -> ChatMessage {
        self.push_message(direction, peer, text, att, None)
    }

    fn push_message(
        &mut self,
        direction: Direction,
        peer: [u8; ID_LEN],
        text: String,
        att: Option<Attachment>,
        from: Option<String>,
    ) -> ChatMessage {
        let msg = ChatMessage {
            direction,
            peer: hex_encode(&peer),
            text,
            seq: self.seq,
            ts: now_unix_s(),
            delivered: false,
            pending: false,
            att,
            digest: String::new(),
            from,
            read: false,
        };
        self.seq += 1;
        self.log.push(msg.clone());
        msg
    }

    // -----------------------------------------------------------------------
    // Groups (Mercury Groups v1 — see docs/GROUPS.md for the honest design)
    //
    // Classical MLS (audited RustCrypto provider; the X-Wing PQ suite stays gated off until its
    // libcrux pins are patched upstream — running 1:1 chats remain PQ-hybrid). Every group frame
    // (invite, key package, welcome, commit, message) rides the EXISTING verified 1:1 sealed
    // channels as control ops, so the relay is untouched and sees only the same opaque 1:1
    // ciphertext it always did (it can infer fan-out timing — disclosed). v1 is
    // creator-administered: the creator invites, admits, processes leaves, and can close the
    // group. Invitees are auto-admitted ONLY if the inviter is an existing verified contact.
    // -----------------------------------------------------------------------

    /// Lazily create (or return) the persistent MLS engine for this account.
    fn mls_engine(&mut self) -> Result<&MlsEngine, AppError> {
        if self.mls.is_none() {
            let me = self.client.account_id();
            let engine = MlsEngine::new(&me)
                .map_err(|e| AppError::Group(format!("group engine init failed: {e}")))?;
            self.mls = Some(engine);
        }
        Ok(self.mls.as_ref().expect("engine just ensured"))
    }

    /// Seal + dispatch one group CONTROL frame to `to` over the 1:1 channel (outbox-aware).
    /// Best-effort per recipient: an unreachable/blocked member is skipped (returns false) —
    /// group consistency converges when they next interact or via the creator's commits.
    ///
    /// Fellow group members are not necessarily mutual 1:1 contacts (Bob and Carol may both know
    /// only the creator), so delivery AUTO-FETCHES a missing member's contact card from the
    /// directory. That is sound: the member identity came from the MLS roster (admitted by the
    /// group's creator), and the fetched card is independently SELF-AUTHENTICATING (its account
    /// id is the hash of its identity key) — the relay cannot substitute anyone. Fetching a card
    /// creates no visible 1:1 conversation; it only enables sealed delivery.
    fn send_group_control(&mut self, to: &[u8; ID_LEN], body: &Value) -> bool {
        if *to == self.client.account_id() {
            return true; // never loop a frame back to ourselves
        }
        if !self.client.has_session(to) && !self.contacts.contains_key(to) {
            match self.client.fetch_contact(&self.transport, to) {
                Ok(card) => {
                    self.contacts.insert(*to, card);
                }
                Err(_) => return false, // member unreachable right now — skip, retry next frame
            }
        }
        let mut payload = CONTROL_SENTINEL.to_vec();
        payload.extend_from_slice(&serde_json::to_vec(body).expect("control json serializes"));
        match self.seal_for(to, &payload) {
            Ok(blob) => self.dispatch(to, blob).is_ok(),
            Err(_) => false,
        }
    }

    /// Create a group named `name` and invite `member_hexes` (existing contacts only, at most
    /// [`MAX_GROUP_MEMBERS`] - 1). Returns `{ "group": "<gid hex>" }`. Invitations are key-package
    /// requests over each member's verified 1:1 channel; members are admitted as their key
    /// packages arrive (see the `gkp` handler).
    pub fn create_group(&mut self, name: &str, member_hexes: &[String]) -> Result<Value, AppError> {
        let name = sanitize_group_name(name);
        if member_hexes.is_empty() {
            return Err(AppError::Group("invite at least one contact".into()));
        }
        if member_hexes.len() > MAX_GROUP_MEMBERS - 1 {
            return Err(AppError::Group(format!(
                "groups are capped at {MAX_GROUP_MEMBERS} members for now"
            )));
        }
        let me = self.client.account_id();
        let mut invited = Vec::new();
        for hex in member_hexes {
            let id = parse_id(hex)?;
            if id == me {
                continue;
            }
            // Invitees must be REAL, already-verified contacts (a card or a live session): group
            // membership never bypasses the 1:1 trust path.
            if !(self.client.has_session(&id) || self.contacts.contains_key(&id)) {
                return Err(AppError::Group(format!(
                    "{} is not a contact yet — add them first",
                    &hex[..hex.len().min(10)]
                )));
            }
            if !invited.contains(&id) {
                invited.push(id);
            }
        }
        if invited.is_empty() {
            return Err(AppError::Group("invite at least one contact".into()));
        }

        let mut gid = [0u8; ID_LEN];
        getrandom::getrandom(&mut gid).map_err(|_| AppError::Group("no entropy".into()))?;
        self.mls_engine()?
            .create_group(&gid)
            .map_err(|e| AppError::Group(format!("create failed: {e}")))?;
        self.groups.insert(
            gid,
            GroupMeta {
                name: name.clone(),
                creator: me,
                members: vec![me],
                pending: invited.clone(),
                ended: false,
            },
        );
        // Invite each member: a key-package request carrying the group's name. Their app replies
        // with a fresh single-use key package (gkp), which admits them.
        let gid_hex = hex_encode(&gid);
        for member in &invited {
            let body = json!({ "op": "gkpr", "gid": gid_hex, "name": name });
            self.send_group_control(&member.clone(), &body);
        }
        self.push_message(
            Direction::Incoming,
            gid,
            format!("\u{1f465} Group \u{201c}{name}\u{201d} created — invitations sent."),
            None,
            None,
        );
        Ok(json!({ "group": gid_hex }))
    }

    /// Send `text` to every member of group `gid_hex` (one MLS ciphertext, fanned out over each
    /// member's 1:1 sealed channel, outbox-aware). Fails closed on an unknown or ended group.
    pub fn group_send(&mut self, gid_hex: &str, text: &str) -> Result<(), AppError> {
        let gid = parse_id(gid_hex)?;
        let meta = self
            .groups
            .get(&gid)
            .ok_or_else(|| AppError::Group("no such group".into()))?
            .clone();
        if meta.ended {
            return Err(AppError::Group("this group was closed".into()));
        }
        let wire = self
            .mls_engine()?
            .send(&gid, text.as_bytes())
            .map_err(|e| AppError::Group(format!("encrypt failed: {e}")))?;
        use base64::Engine as _;
        let body = json!({
            "op": "gmsg",
            "gid": gid_hex,
            "mls": base64::engine::general_purpose::STANDARD.encode(&wire),
        });
        let me = self.client.account_id();
        let mut any_queued = false;
        for member in meta.members.iter().filter(|m| **m != me) {
            // Outbox-aware: a full mailbox queues rather than failing the whole group send.
            if !self.send_group_control(&member.clone(), &body) {
                any_queued = true;
            }
        }
        let msg = self.push_message(Direction::Outgoing, gid, text.to_string(), None, None);
        if any_queued {
            if let Some(m) = self.log.iter_mut().find(|m| m.seq == msg.seq) {
                m.pending = true;
            }
        }
        Ok(())
    }

    /// Leave group `gid_hex`. As a MEMBER: notify the creator (who commits the removal so the
    /// epoch rotates) and forget the group locally either way. As the CREATOR: close the group
    /// for everyone (`gend`) — v1 groups are creator-administered and do not outlive their admin.
    pub fn leave_group(&mut self, gid_hex: &str) -> Result<(), AppError> {
        let gid = parse_id(gid_hex)?;
        let meta = self
            .groups
            .get(&gid)
            .ok_or_else(|| AppError::Group("no such group".into()))?
            .clone();
        let me = self.client.account_id();
        if meta.creator == me {
            let body = json!({ "op": "gend", "gid": gid_hex });
            for member in meta.members.iter().filter(|m| **m != me) {
                self.send_group_control(&member.clone(), &body);
            }
        } else {
            let body = json!({ "op": "glve", "gid": gid_hex });
            self.send_group_control(&meta.creator.clone(), &body);
        }
        if let Some(engine) = self.mls.as_ref() {
            let _ = engine.leave_local(&gid);
        }
        self.groups.remove(&gid);
        let key = hex_encode(&gid);
        self.log.retain(|m| m.peer != key);
        Ok(())
    }

    /// Handle one inbound GROUP control op (already authenticated by the 1:1 channel: `from` is
    /// the verified sender). Returns a message to surface, if any. Every parse failure or
    /// out-of-protocol frame is dropped silently (fail-closed, forward-compatible).
    fn handle_group_control(
        &mut self,
        from: &[u8; ID_LEN],
        op: &str,
        obj: &serde_json::Map<String, Value>,
    ) -> Option<ChatMessage> {
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD;
        let gid_hex = obj.get("gid").and_then(Value::as_str)?;
        let gid = parse_id(gid_hex).ok()?;
        let me = self.client.account_id();

        match op {
            // Invitation: reply with a fresh single-use key package — but ONLY to an existing
            // verified contact (group invites never bypass the 1:1 trust path).
            "gkpr" => {
                // Reply with a fresh MLS key package ONLY to an existing CONTACT (someone we added
                // or replied to). A bare session is NOT enough: an unsolicited first-flight creates
                // a session before this handler runs, so gating on `has_session` would let a
                // STRANGER who merely knows our account id elicit our key package and pull us into
                // their group. `contacts` is the real, user-chosen trust signal. (Mutual-contact
                // requirement disclosed in docs/GROUPS.md.)
                if !self.contacts.contains_key(from) {
                    return None;
                }
                let kp = self.mls_engine().ok()?.fresh_key_package().ok()?;
                let body = json!({ "op": "gkp", "gid": gid_hex, "kp": b64.encode(&kp) });
                self.send_group_control(&from.clone(), &body);
                None
            }
            // A key package back from an invitee: admit them (creator only), send their Welcome,
            // and fan the commit to every OTHER existing member.
            "gkp" => {
                let meta = self.groups.get(&gid)?.clone();
                if meta.creator != me || meta.ended || !meta.pending.contains(from) {
                    return None;
                }
                if meta.members.len() >= MAX_GROUP_MEMBERS {
                    return None;
                }
                let kp = b64.decode(obj.get("kp").and_then(Value::as_str)?).ok()?;
                let outcome = self.mls_engine().ok()?.add_member(&gid, &kp).ok()?;
                // The MLS credential inside the key package must BE the inviter saw — the member
                // we invited. A mismatch is a substitution attempt: drop it.
                if outcome.added_identity != from.as_slice() {
                    return None;
                }
                let meta = self.groups.get_mut(&gid)?;
                meta.pending.retain(|p| p != from);
                if !meta.members.contains(from) {
                    meta.members.push(*from);
                }
                let members = meta.members.clone();
                let name = meta.name.clone();
                let welcome = json!({
                    "op": "gwel",
                    "gid": gid_hex,
                    "name": name,
                    "welcome": b64.encode(&outcome.welcome_wire),
                });
                self.send_group_control(&from.clone(), &welcome);
                let commit = json!({
                    "op": "gcmt",
                    "gid": gid_hex,
                    "commit": b64.encode(&outcome.commit_wire),
                });
                for member in members.iter().filter(|m| **m != me && *m != from) {
                    self.send_group_control(&member.clone(), &commit);
                }
                self.dirty_peers.insert(gid_hex.to_string());
                Some(self.push_message(
                    Direction::Incoming,
                    gid,
                    format!("\u{1f465} {} joined.", short_label(from)),
                    None,
                    None,
                ))
            }
            // Our Welcome: join the group the creator admitted us to.
            "gwel" => {
                // Only JOIN at the invitation of an existing CONTACT (the creator we added or
                // replied to). A stranger who first-flighted us has a session but is NOT a contact,
                // so they cannot force us into a group they control. Fail closed otherwise.
                if !self.contacts.contains_key(from) {
                    return None;
                }
                if self.groups.contains_key(&gid) {
                    return None; // already joined (duplicate welcome)
                }
                let welcome = b64
                    .decode(obj.get("welcome").and_then(Value::as_str)?)
                    .ok()?;
                let joined = self.mls_engine().ok()?.join_from_welcome(&welcome).ok()?;
                if joined != gid {
                    return None; // welcome for a different group than claimed — drop
                }
                let name =
                    sanitize_group_name(obj.get("name").and_then(Value::as_str).unwrap_or("Group"));
                let roster = self.mls_engine().ok()?.roster(&gid).unwrap_or_default();
                let members: Vec<[u8; ID_LEN]> = roster
                    .into_iter()
                    .filter_map(|identity| <[u8; ID_LEN]>::try_from(identity.as_slice()).ok())
                    .collect();
                self.groups.insert(
                    gid,
                    GroupMeta {
                        name: name.clone(),
                        creator: *from,
                        members,
                        pending: Vec::new(),
                        ended: false,
                    },
                );
                Some(self.push_message(
                    Direction::Incoming,
                    gid,
                    format!("\u{1f465} You joined \u{201c}{name}\u{201d}."),
                    None,
                    None,
                ))
            }
            // A membership-change commit: advance our group view, then mirror the MLS roster.
            "gcmt" => {
                // Membership commits are CREATOR-administered. MLS itself has no creator concept —
                // ANY admitted member can craft a valid Add/Remove commit — so commit authority is
                // pinned to the 1:1-authenticated creator carrier. A commit from any other member is
                // a silent membership-tampering attempt (injecting an outsider into, or evicting
                // members from, the E2E group): drop it. Every legitimate gcmt originates from the
                // creator (emitted by the `gkp` and `glve` arms).
                let creator = match self.groups.get(&gid) {
                    Some(meta) => meta.creator,
                    None => return None,
                };
                if creator != *from {
                    return None;
                }
                let commit = b64
                    .decode(obj.get("commit").and_then(Value::as_str)?)
                    .ok()?;
                self.mls_engine().ok()?.apply_commit(&gid, &commit).ok()?;
                let roster = self.mls_engine().ok()?.roster(&gid).unwrap_or_default();
                let members: Vec<[u8; ID_LEN]> = roster
                    .into_iter()
                    .filter_map(|identity| <[u8; ID_LEN]>::try_from(identity.as_slice()).ok())
                    .collect();
                let still_in = members.contains(&me);
                let meta = self.groups.get_mut(&gid)?;
                meta.members = members;
                self.dirty_peers.insert(gid_hex.to_string());
                if !still_in {
                    meta.ended = true;
                    return Some(self.push_message(
                        Direction::Incoming,
                        gid,
                        "\u{1f465} You were removed from this group.".to_string(),
                        None,
                        None,
                    ));
                }
                None
            }
            // A group application message: decrypt + attribute via MLS, cross-checked against
            // the authenticated 1:1 carrier.
            "gmsg" => {
                let meta = self.groups.get(&gid)?;
                if meta.ended {
                    return None;
                }
                let wire = b64.decode(obj.get("mls").and_then(Value::as_str)?).ok()?;
                let (sender, plaintext) = self.mls_engine().ok()?.receive(&gid, &wire).ok()?;
                // The MLS-attributed sender must be the 1:1 peer who carried the frame — a member
                // relaying someone else's authorship is a spoof attempt; drop it.
                if sender != from.as_slice() {
                    return None;
                }
                let text = String::from_utf8_lossy(&plaintext).into_owned();
                Some(self.push_message(
                    Direction::Incoming,
                    gid,
                    text,
                    None,
                    Some(hex_encode(from)),
                ))
            }
            // A member announces they are leaving: the CREATOR commits the removal (epoch
            // rotation = the leaver cryptographically loses access) and fans the commit out.
            "glve" => {
                let meta = self.groups.get(&gid)?.clone();
                if meta.creator != me || !meta.members.contains(from) {
                    return None;
                }
                let commit = self.mls_engine().ok()?.remove_member(&gid, from).ok()?;
                let meta = self.groups.get_mut(&gid)?;
                meta.members.retain(|m| m != from);
                let members = meta.members.clone();
                let body = json!({
                    "op": "gcmt",
                    "gid": gid_hex,
                    "commit": b64.encode(&commit),
                });
                for member in members.iter().filter(|m| **m != me) {
                    self.send_group_control(&member.clone(), &body);
                }
                self.dirty_peers.insert(gid_hex.to_string());
                Some(self.push_message(
                    Direction::Incoming,
                    gid,
                    format!("\u{1f465} {} left.", short_label(from)),
                    None,
                    None,
                ))
            }
            // The creator closed the group: read-only from here, state forgotten.
            "gend" => {
                let meta = self.groups.get_mut(&gid)?;
                if meta.creator != *from || meta.ended {
                    return None;
                }
                meta.ended = true;
                if let Some(engine) = self.mls.as_ref() {
                    let _ = engine.leave_local(&gid);
                }
                Some(self.push_message(
                    Direction::Incoming,
                    gid,
                    "\u{1f465} The group was closed by its creator.".to_string(),
                    None,
                    None,
                ))
            }
            _ => None,
        }
    }

    /// Drive the controller from a single JSON command — the FFI/Tauri/HTTP-ready surface the
    /// UI calls. Input: `{"cmd":"<name>", ...args}`. Output: `{"ok":true,"result":<...>}` or
    /// `{"ok":false,"error":"<message>"}`. NEVER panics: a malformed/unknown command or a
    /// failing operation all return a JSON error object (fail-closed). This is the single entry
    /// point a thin FFI/Tauri/HTTP shim wraps, so the wire surface stays small and auditable.
    ///
    /// Commands: `account_id`, `publish_self`, `add_contact{account_id}`, `can_send{peer}`,
    /// `send{peer,text}`, `poll`, `history{peer}`, `log`, `decisions{peer?}`, `safety_number{peer}`,
    /// `delete_conversation{peer}`, `clear_history{peer}`, `delete_for_everyone{peer}`,
    /// `mute{peer}`, `unmute{peer}`, `block{peer}`, `unblock{peer}`, `conversation_flags`,
    /// `set_disappearing{peer,secs}`, `pairing_code`, `add_by_pairing{code}`,
    /// `claim_username{username}`, `add_by_username{username}`, `set_verified{peer}`,
    /// `unset_verified{peer}`, `send_attachment{peer,name,mime,data_b64}`,
    /// `backup_export{passphrase}`, `create_group{name,members}`, `group_send{group,text}`,
    /// `leave_group{group}`.
    pub fn handle_command(&mut self, json_command: &str) -> String {
        let response = match serde_json::from_str::<Command>(json_command) {
            Ok(cmd) => self.run_command(cmd),
            Err(_) => CommandResponse::error("malformed or unknown command"),
        };
        serde_json::to_string(&response).expect("a command response always serializes")
    }

    fn run_command(&mut self, cmd: Command) -> CommandResponse {
        match cmd {
            Command::AccountId => CommandResponse::ok(json!({ "account_id": self.account_id() })),
            Command::PublishSelf => unit_result(self.publish_self()),
            Command::AddContact { account_id } => unit_result(self.add_contact(&account_id)),
            Command::CanSend { peer } => {
                CommandResponse::ok(json!({ "can_send": self.can_send(&peer) }))
            }
            Command::Send { peer, text } => unit_result(self.send(&peer, &text)),
            Command::Poll => match self.poll() {
                Ok(messages) => {
                    // `updated` = conversations whose EXISTING messages changed (a receipt landed,
                    // a pending send was accepted) — the UI refreshes those histories without
                    // counting anything as a new arrival. Additive field: older UIs ignore it.
                    let updated = self.take_dirty_peers();
                    CommandResponse::ok(json!({ "messages": messages, "updated": updated }))
                }
                Err(e) => CommandResponse::error(e),
            },
            Command::History { peer } => {
                CommandResponse::ok(json!({ "messages": self.history(&peer) }))
            }
            Command::Log => CommandResponse::ok(json!({ "messages": self.log() })),
            Command::Decisions { peer } => {
                match serde_json::to_value(self.decision_views(peer.as_deref())) {
                    Ok(views) => CommandResponse::ok(views),
                    Err(_) => CommandResponse::error("decision serialization failed"),
                }
            }
            Command::SafetyNumber { peer } => match self.safety_number(&peer) {
                Ok(value) => CommandResponse::ok(value),
                Err(e) => CommandResponse::error(e),
            },
            Command::DeleteConversation { peer } => unit_result(self.delete_conversation(&peer)),
            Command::ClearHistory { peer } => unit_result(self.clear_history(&peer)),
            Command::DeleteForEveryone { peer } => unit_result(self.delete_for_everyone(&peer)),
            Command::Mute { peer } => unit_result(self.mute(&peer)),
            Command::Unmute { peer } => unit_result(self.unmute(&peer)),
            Command::Block { peer } => unit_result(self.block(&peer)),
            Command::Unblock { peer } => unit_result(self.unblock(&peer)),
            Command::ConversationFlags => CommandResponse::ok(self.conversation_flags()),
            Command::SetDisappearing { peer, secs } => {
                unit_result(self.set_disappearing(&peer, secs))
            }
            Command::PairingCode => match self.pairing_code() {
                Ok(value) => CommandResponse::ok(value),
                Err(e) => CommandResponse::error(e),
            },
            Command::AddByPairing { code } => match self.add_by_pairing(&code) {
                Ok(value) => CommandResponse::ok(value),
                Err(e) => CommandResponse::error(e),
            },
            Command::ClaimUsername { username } => match self.claim_username(&username) {
                Ok(value) => CommandResponse::ok(value),
                Err(e) => CommandResponse::error(e),
            },
            Command::AddByUsername { username } => match self.add_by_username(&username) {
                Ok(value) => CommandResponse::ok(value),
                Err(e) => CommandResponse::error(e),
            },
            Command::SetVerified { peer } => unit_result(self.set_verified(&peer)),
            Command::UnsetVerified { peer } => unit_result(self.unset_verified(&peer)),
            Command::SendAttachment {
                peer,
                name,
                mime,
                data_b64,
            } => match self.send_attachment(&peer, &name, &mime, &data_b64) {
                Ok(value) => CommandResponse::ok(value),
                Err(e) => CommandResponse::error(e),
            },
            Command::BackupExport { passphrase } => match self.export_backup(&passphrase) {
                Ok(bytes) => {
                    use base64::Engine as _;
                    CommandResponse::ok(json!({
                        "data_b64": base64::engine::general_purpose::STANDARD.encode(bytes)
                    }))
                }
                Err(e) => CommandResponse::error(e),
            },
            Command::CreateGroup { name, members } => match self.create_group(&name, &members) {
                Ok(value) => CommandResponse::ok(value),
                Err(e) => CommandResponse::error(e),
            },
            Command::GroupSend { group, text } => unit_result(self.group_send(&group, &text)),
            Command::LeaveGroup { group } => unit_result(self.leave_group(&group)),
            Command::MarkRead { peer } => unit_result(self.mark_read(&peer)),
            Command::SetReadReceipts { on } => {
                self.set_read_receipts(on);
                CommandResponse::ok(json!({}))
            }
            Command::SetPinned { peer, on } => unit_result(self.set_pinned(&peer, on)),
        }
    }
}

/// The three REAL decision views the UI inspector renders, each a genuine mercury-core
/// `PlatformDecisionView` built from this controller's actual state (never fabricated).
#[derive(Serialize)]
pub struct DecisionViews {
    pub bootstrap: PlatformDecisionView,
    pub outbound: PlatformDecisionView,
    pub receive: PlatformDecisionView,
}

/// A UI command, deserialized from `{"cmd":"<name>", ...}`. An unknown name or a missing field
/// fails deserialization, which [`AppController::handle_command`] turns into a fail-closed error
/// response — there is no "default" or silently-ignored command.
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum Command {
    AccountId,
    PublishSelf,
    AddContact {
        account_id: String,
    },
    CanSend {
        peer: String,
    },
    Send {
        peer: String,
        text: String,
    },
    Poll,
    History {
        peer: String,
    },
    Log,
    Decisions {
        #[serde(default)]
        peer: Option<String>,
    },
    SafetyNumber {
        peer: String,
    },
    DeleteConversation {
        peer: String,
    },
    ClearHistory {
        peer: String,
    },
    DeleteForEveryone {
        peer: String,
    },
    Mute {
        peer: String,
    },
    Unmute {
        peer: String,
    },
    Block {
        peer: String,
    },
    Unblock {
        peer: String,
    },
    ConversationFlags,
    SetDisappearing {
        peer: String,
        secs: u64,
    },
    PairingCode,
    AddByPairing {
        code: String,
    },
    ClaimUsername {
        username: String,
    },
    AddByUsername {
        username: String,
    },
    SetVerified {
        peer: String,
    },
    UnsetVerified {
        peer: String,
    },
    SendAttachment {
        peer: String,
        name: String,
        mime: String,
        data_b64: String,
    },
    BackupExport {
        passphrase: String,
    },
    CreateGroup {
        name: String,
        members: Vec<String>,
    },
    GroupSend {
        group: String,
        text: String,
    },
    LeaveGroup {
        group: String,
    },
    MarkRead {
        peer: String,
    },
    SetReadReceipts {
        on: bool,
    },
    SetPinned {
        peer: String,
        on: bool,
    },
}

/// The JSON response envelope for [`AppController::handle_command`]: `ok` plus exactly one of
/// `result` (success) or `error` (a human-readable, key/plaintext-free message).
#[derive(Debug, Serialize)]
struct CommandResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl CommandResponse {
    fn ok(result: Value) -> Self {
        Self {
            ok: true,
            result: Some(result),
            error: None,
        }
    }
    fn error(e: impl core::fmt::Display) -> Self {
        Self {
            ok: false,
            result: None,
            error: Some(e.to_string()),
        }
    }
}

/// Map a unit-returning controller op into a command response.
fn unit_result(r: Result<(), AppError>) -> CommandResponse {
    match r {
        Ok(()) => CommandResponse::ok(Value::Null),
        Err(e) => CommandResponse::error(e),
    }
}

/// Lowercase-hex encode.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).expect("nibble < 16"));
        s.push(char::from_digit((b & 0x0f) as u32, 16).expect("nibble < 16"));
    }
    s
}

/// Parse a 32-byte account id from lowercase hex. Fails closed ([`AppError::BadAccountId`]) on
/// the wrong length or any non-hex digit.
fn parse_id(s: &str) -> Result<[u8; ID_LEN], AppError> {
    if s.len() != ID_LEN * 2 {
        return Err(AppError::BadAccountId);
    }
    let mut out = [0u8; ID_LEN];
    let bytes = s.as_bytes();
    for (i, slot) in out.iter_mut().enumerate() {
        let hi = (bytes[2 * i] as char)
            .to_digit(16)
            .ok_or(AppError::BadAccountId)?;
        let lo = (bytes[2 * i + 1] as char)
            .to_digit(16)
            .ok_or(AppError::BadAccountId)?;
        *slot = (hi * 16 + lo) as u8;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips() {
        let id = [0xab; ID_LEN];
        assert_eq!(parse_id(&hex_encode(&id)).unwrap(), id);
    }

    #[test]
    fn parse_id_is_fail_closed() {
        assert_eq!(parse_id("zz"), Err(AppError::BadAccountId)); // wrong length
        assert_eq!(parse_id(&"gg".repeat(32)), Err(AppError::BadAccountId)); // non-hex
        assert_eq!(parse_id(&"ab".repeat(31)), Err(AppError::BadAccountId)); // 31 bytes
    }

    #[test]
    fn decision_views_reflect_real_state_not_fabricated() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();

        let before = alice.decision_views(Some(&bob_id));
        // Bootstrap is genuinely accepted — Alice holds a real identity.
        assert!(before.bootstrap.accepted);
        assert_eq!(before.bootstrap.reason_label, "ACCEPTED");
        // BEFORE verifying Bob: the outbound view is a REAL reject (no session, no contact card) —
        // proving the view is NOT a hardcoded ACCEPT.
        assert!(!before.outbound.accepted);
        assert!(!before.outbound.can_send);
        assert_ne!(before.outbound.reason_label, "ACCEPTED");

        // AFTER verifying Bob's card: the outbound view flips to a REAL accept.
        alice.add_contact(&bob_id).unwrap();
        let after = alice.decision_views(Some(&bob_id));
        assert!(after.outbound.accepted);
        assert!(after.outbound.can_send);
        assert_eq!(after.outbound.reason_label, "ACCEPTED");
        assert_eq!(after.outbound.source, "outbound_send");

        // No conversation target -> not sendable.
        assert!(!alice.decision_views(None).outbound.accepted);

        // Receive view: the default last-receive is OK -> a real ACCEPT.
        assert!(after.receive.accepted);
        assert_eq!(after.receive.reason_label, "ACCEPTED");
        // A coarse receive FAILURE is reported HONESTLY: a conservative, retry-oriented ORDERING_GAP
        // (fail-safe — never a false ACCEPT), NOT the old MESSAGE_POLICY_REJECTED which would name a
        // specific policy denial the coarse client error cannot actually prove.
        alice.last_receive_ok = false;
        let failed = alice.decision_views(Some(&bob_id)).receive;
        assert!(!failed.accepted);
        assert!(failed.requires_client_retry);
        assert_eq!(failed.reason_label, "ORDERING_GAP");
        assert_ne!(failed.reason_label, "MESSAGE_POLICY_REJECTED");

        // The `decisions` command surfaces the same real views as JSON.
        let json =
            alice.handle_command(&format!("{{\"cmd\":\"decisions\",\"peer\":\"{bob_id}\"}}"));
        assert!(json.contains("\"source\":\"outbound_send\""));
        assert!(json.contains("\"source\":\"client_bootstrap\""));
        assert!(json.contains("\"source\":\"client_receive\""));
    }

    /// Poll every controller for a FIXED number of full rounds — converges multi-hop group
    /// protocol exchanges (invite -> key package -> welcome/commit -> ...). Fixed rounds, not
    /// quiet-detection: several control hops (e.g. answering an invite with a key package) are
    /// deliberately silent, so "no visible message this round" does not mean "nothing in flight".
    fn pump(ctrls: &mut [&mut AppController<mercury_client::InMemoryTransport>]) {
        for _ in 0..10 {
            for c in ctrls.iter_mut() {
                c.poll().unwrap();
            }
        }
    }

    #[test]
    fn three_member_group_full_lifecycle() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        let mut carol = AppController::new(transport.clone());
        for c in [&mut alice, &mut bob, &mut carol] {
            c.publish_self().unwrap();
        }
        let (a_id, b_id, c_id) = (alice.account_id(), bob.account_id(), carol.account_id());
        alice.add_contact(&b_id).unwrap();
        alice.add_contact(&c_id).unwrap();
        // Group invites require the invitee to ALREADY trust the creator (mutual contact), so a
        // stranger can't pull you into a group — Bob and Carol add Alice too.
        bob.add_contact(&a_id).unwrap();
        carol.add_contact(&a_id).unwrap();
        // Establish live 1:1 sessions (group control frames ride them).
        alice.send(&b_id, "hi bob").unwrap();
        alice.send(&c_id, "hi carol").unwrap();
        pump(&mut [&mut alice, &mut bob, &mut carol]);

        // Alice creates the group; invitations + admissions converge over polls.
        let gid = alice
            .create_group("Project Mercury", &[b_id.clone(), c_id.clone()])
            .unwrap()["group"]
            .as_str()
            .unwrap()
            .to_string();
        pump(&mut [&mut alice, &mut bob, &mut carol]);

        // Everyone agrees on the roster (3 members) and the name.
        for c in [&alice, &bob, &carol] {
            let flags = c.conversation_flags();
            let g = &flags["groups"][&gid];
            assert_eq!(g["name"], json!("Project Mercury"));
            assert_eq!(g["members"].as_array().unwrap().len(), 3, "full roster");
            assert_eq!(g["ended"], json!(false));
        }

        // Group messages flow with MLS sender attribution.
        alice.group_send(&gid, "hello group").unwrap();
        pump(&mut [&mut alice, &mut bob, &mut carol]);
        for c in [&bob, &carol] {
            let hist = c.history(&gid);
            let m = hist.iter().find(|m| m.text == "hello group").unwrap();
            assert_eq!(m.from.as_deref(), Some(alice.account_id().as_str()));
        }
        bob.group_send(&gid, "hi from bob").unwrap();
        pump(&mut [&mut alice, &mut bob, &mut carol]);
        assert!(alice.history(&gid).iter().any(|m| m.text == "hi from bob"));
        assert!(carol.history(&gid).iter().any(|m| m.text == "hi from bob"));

        // Carol leaves: the creator commits the removal; Bob applies it; the epoch rotates so a
        // post-leave message NEVER reaches Carol.
        carol.leave_group(&gid).unwrap();
        pump(&mut [&mut alice, &mut bob, &mut carol]);
        let flags = alice.conversation_flags();
        assert_eq!(
            flags["groups"][&gid]["members"].as_array().unwrap().len(),
            2,
            "Carol removed from the roster"
        );
        alice.group_send(&gid, "after carol left").unwrap();
        pump(&mut [&mut alice, &mut bob, &mut carol]);
        assert!(
            bob.history(&gid)
                .iter()
                .any(|m| m.text == "after carol left")
        );
        assert!(
            carol.history(&gid).is_empty(),
            "a departed member's local state is gone and it cannot read post-leave messages"
        );

        // The group survives a full encrypted-snapshot restart (MLS state rides the seal).
        let key = [9u8; 32];
        let blob = alice.to_encrypted_snapshot(&key);
        let mut alice2 =
            AppController::from_encrypted_snapshot(transport.clone(), &blob, &key).unwrap();
        alice2.group_send(&gid, "after restart").unwrap();
        pump(&mut [&mut alice2, &mut bob]);
        assert!(
            bob.history(&gid).iter().any(|m| m.text == "after restart"),
            "restored MLS state still encrypts for the group"
        );

        // The creator closes the group: members see it ended and read-only.
        alice2.leave_group(&gid).unwrap();
        pump(&mut [&mut alice2, &mut bob]);
        let flags = bob.conversation_flags();
        assert_eq!(flags["groups"][&gid]["ended"], json!(true));
        assert!(
            bob.group_send(&gid, "too late").is_err(),
            "ended group refuses sends"
        );
    }

    #[test]
    fn group_invites_require_an_existing_contact() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        alice.publish_self().unwrap();
        bob.publish_self().unwrap();
        let b_id = bob.account_id();

        // Inviting a non-contact fails closed at the creator.
        assert!(alice.create_group("g", &["ab".repeat(32)]).is_err());

        // Alice adds Bob and first-flights him, so Bob now has a SESSION with Alice — but Bob never
        // added Alice, so she is NOT in his contacts. To Bob, an unsolicited inviter is a stranger:
        // a bare session (which any unsolicited first-flight creates) must not be enough to join.
        alice.add_contact(&b_id).unwrap();
        alice.send(&b_id, "hi").unwrap();
        pump(&mut [&mut alice, &mut bob]);

        // Alice tries to pull Bob into a group. Bob's gkpr handler drops the invite (Alice isn't a
        // contact of his), so Bob never replies with a key package and never joins.
        let gid = alice.create_group("Surprise", &[b_id.clone()]).unwrap()["group"]
            .as_str()
            .unwrap()
            .to_string();
        pump(&mut [&mut alice, &mut bob]);
        assert!(
            bob.conversation_flags()["groups"]
                .as_object()
                .map(|g| g.is_empty())
                .unwrap_or(true),
            "a non-contact's group invite is dropped: Bob never joins"
        );
        assert_eq!(
            alice.conversation_flags()["groups"][&gid]["members"]
                .as_array()
                .unwrap()
                .len(),
            1,
            "Bob is never admitted — only Alice is a member"
        );
    }

    #[test]
    fn a_state_changing_poll_is_flagged_dirty_even_with_no_surfaced_message() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        alice.publish_self().unwrap();
        bob.publish_self().unwrap();
        let b_id = bob.account_id();

        // A truly idle poll (nothing pending) changes nothing — the shell may skip the re-seal.
        bob.poll().unwrap();
        assert!(!bob.take_state_dirty(), "an idle poll is not state-dirty");

        // Alice (NOT a contact of Bob) first-flights Bob a group invite. Bob's poll ACCEPTS the
        // session + CONSUMES a one-time prekey, but the gkpr is dropped (non-contact), so the poll
        // surfaces no message and no dirty conversation. The durable session/prekey change must
        // STILL be flagged — otherwise a quit/restart loses the session and silently re-opens the
        // consumed prekey (a first-flight replay window). This is the regression the fix closes.
        alice.add_contact(&b_id).unwrap();
        alice.create_group("Surprise", &[b_id.clone()]).unwrap();
        let surfaced = bob.poll().unwrap();
        assert!(
            surfaced.is_empty(),
            "the non-contact invite surfaces no message"
        );
        assert!(
            bob.take_dirty_peers().is_empty(),
            "and no dirty conversation"
        );
        assert!(
            bob.take_state_dirty(),
            "but the accepted session + consumed prekey MUST be flagged for persistence"
        );
    }

    #[test]
    fn mark_read_that_sends_a_watermark_flags_state_dirty() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        alice.publish_self().unwrap();
        bob.publish_self().unwrap();
        let (a_id, b_id) = (alice.account_id(), bob.account_id());
        alice.add_contact(&b_id).unwrap();
        bob.add_contact(&a_id).unwrap();

        // Bob messages Alice; Alice receives it (poll's own state change cleared below).
        bob.send(&a_id, "hi alice").unwrap();
        alice.poll().unwrap();
        alice.take_state_dirty();

        // Alice opens the conversation: mark_read sends a `read` watermark, which advances Alice's
        // outbound ratchet. That is durable state the shell MUST persist even though mark_read is
        // not a "mutating" command — otherwise a read-then-quit desyncs the next message.
        alice.mark_read(&b_id).unwrap();
        assert!(
            alice.take_state_dirty(),
            "a mark_read that sends a watermark advanced the ratchet and MUST flag state_dirty"
        );

        // A second mark_read with no newer incoming sends nothing (watermark unchanged) — no
        // ratchet advance, so it is NOT state-dirty (no wasteful re-seal on every focus event).
        alice.mark_read(&b_id).unwrap();
        assert!(
            !alice.take_state_dirty(),
            "a no-op mark_read (watermark unchanged) must not flag state_dirty"
        );
    }

    #[test]
    fn delivery_receipts_mark_the_senders_copy_delivered() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();

        alice.send(&bob_id, "hello").unwrap();
        let sent = alice.history(&bob_id);
        assert!(!sent[0].delivered, "not delivered before Bob polls");
        assert!(!sent[0].digest.is_empty(), "outgoing carries its digest");

        // Bob polls (receives + auto-sends a rcpt control); Alice polls (consumes the rcpt).
        bob.poll().unwrap();
        let fresh = alice.poll().unwrap();
        assert!(
            fresh.is_empty(),
            "a receipt is silent — never a chat bubble"
        );
        let after = alice.history(&bob_id);
        assert!(after[0].delivered, "Alice's copy is now marked delivered");
        // The conversation is flagged dirty exactly once so the UI refreshes it.
        let dirty = alice.take_dirty_peers();
        assert_eq!(dirty, vec![bob_id.clone()]);
    }

    #[test]
    fn a_full_mailbox_queues_in_the_outbox_and_drains_in_order() {
        // The REAL single-slot semantics live in the relay; the in-memory transport is multi-slot,
        // so exercise dispatch()'s queue-behind rule + drain ordering directly.
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();
        let bob_raw = alice
            .log
            .is_empty()
            .then(|| parse_id(&bob_id).unwrap())
            .unwrap();

        // Seed the outbox as if an earlier send hit a full mailbox; later sends must queue BEHIND
        // it (ratchet order), not race past it.
        alice.send(&bob_id, "first").unwrap();
        let first_blob = {
            // Seal a second message and force-queue it (simulating item_already_pending).
            let blob = alice.seal_for(&bob_raw, b"second").unwrap();
            alice.outbox.push((bob_raw, blob, alice.seq));
            alice.push(Direction::Outgoing, bob_raw, "second".to_string())
        };
        assert_eq!(alice.outbox.len(), 1);
        // A THIRD send while the lane is queued goes behind it without submitting.
        alice.send(&bob_id, "third").unwrap();
        assert_eq!(alice.outbox.len(), 2, "third queued behind second");
        let third = alice.history(&bob_id).last().cloned().unwrap();
        assert!(third.pending, "queued message shows pending");

        // Poll drains the outbox in order; Bob then receives second -> third.
        alice.poll().unwrap();
        assert!(alice.outbox.is_empty(), "outbox drained");
        assert!(
            alice.history(&bob_id).iter().all(|m| !m.pending),
            "nothing pending after the drain"
        );
        let got: Vec<String> = bob.poll().unwrap().iter().map(|m| m.text.clone()).collect();
        assert_eq!(
            got,
            vec!["first", "second", "third"],
            "strict order preserved"
        );
        let _ = first_blob;
    }

    /// A fault-injecting test transport that models the relay's REAL single-slot mailbox + transient
    /// network failures, so the store-and-forward error arms in `dispatch()` / `drain_outbox()`
    /// (which the multi-slot, infallible `InMemoryTransport` never triggers) are exercised end to end
    /// rather than simulated by poking `outbox`. The opaque prekey directory (publish/fetch a contact
    /// card) is delegated to a real `InMemoryTransport`; only `submit`/`poll` are intercepted.
    #[derive(Clone)]
    struct FaultTransport {
        dir: mercury_client::InMemoryTransport,
        st: std::sync::Arc<std::sync::Mutex<FaultState>>,
    }

    struct FaultState {
        /// ONE undelivered blob per route — the relay's single-slot mailbox.
        slot: std::collections::HashMap<[u8; 32], Vec<u8>>,
        /// While true, `submit` returns a transient `Network` error (relay briefly unreachable).
        fail_submit: bool,
    }

    impl FaultTransport {
        fn new() -> Self {
            Self {
                dir: mercury_client::InMemoryTransport::new(),
                st: std::sync::Arc::new(std::sync::Mutex::new(FaultState {
                    slot: std::collections::HashMap::new(),
                    fail_submit: false,
                })),
            }
        }
        fn set_fail_submit(&self, v: bool) {
            self.st.lock().unwrap().fail_submit = v;
        }
    }

    impl Transport for FaultTransport {
        fn submit(&self, route: &[u8; 32], blob: &[u8]) -> Result<(), TransportError> {
            let mut s = self.st.lock().unwrap();
            if s.fail_submit {
                return Err(TransportError::Network("relay unreachable".to_string()));
            }
            if s.slot.contains_key(route) {
                // The recipient hasn't drained their single slot yet — the relay's real reason.
                return Err(TransportError::Rejected("item_already_pending".to_string()));
            }
            s.slot.insert(*route, blob.to_vec());
            Ok(())
        }
        fn poll(
            &self,
            route: &[u8; 32],
            _auth: &PollAuth,
        ) -> Result<Option<Vec<u8>>, TransportError> {
            Ok(self.st.lock().unwrap().slot.remove(route))
        }
        fn publish_card(
            &self,
            identity_pub: &[u8; 32],
            card: &[u8],
            pop_sig: &[u8; 64],
        ) -> Result<(), TransportError> {
            self.dir.publish_card(identity_pub, card, pop_sig)
        }
        fn fetch_card(&self, account_id: &[u8; 32]) -> Result<Option<Vec<u8>>, TransportError> {
            self.dir.fetch_card(account_id)
        }
    }

    #[test]
    fn full_mailbox_rejection_queues_then_drains_over_a_single_slot_transport() {
        // Drive the REAL dispatch() item_already_pending arm end to end (not by poking the outbox):
        // a single-slot relay rejects the 2nd send to an undrained route, so it queues + drains.
        let transport = FaultTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();

        // First send fills Bob's single slot.
        alice.send(&bob_id, "A").unwrap();
        // Second send hits item_already_pending → dispatch queues it (never errors, never lost).
        alice.send(&bob_id, "B").unwrap();
        assert_eq!(alice.outbox.len(), 1, "B queued behind the undrained A");
        assert!(
            alice.history(&bob_id).last().unwrap().pending,
            "the queued send shows pending, not an error"
        );

        // Bob drains A (frees the slot); Alice's next poll flushes B in order.
        let got_a: Vec<String> = bob.poll().unwrap().iter().map(|m| m.text.clone()).collect();
        assert_eq!(got_a, vec!["A"]);
        alice.poll().unwrap();
        assert!(alice.outbox.is_empty(), "outbox drained once the slot freed");
        assert!(
            alice.history(&bob_id).iter().all(|m| !m.pending),
            "nothing pending after the drain"
        );
        let got_b: Vec<String> = bob.poll().unwrap().iter().map(|m| m.text.clone()).collect();
        assert_eq!(got_b, vec!["B"]);
    }

    #[test]
    fn a_transient_network_error_on_send_queues_and_later_drains() {
        // The relay being briefly unreachable on send must NOT drop or error the message — it queues
        // (pending) and drains on the next poll once the relay returns. Drives dispatch()'s Network
        // enqueue arm directly.
        let transport = FaultTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();

        transport.set_fail_submit(true);
        alice.send(&bob_id, "during the blip").unwrap();
        assert_eq!(
            alice.outbox.len(),
            1,
            "a transient network error queues the send, never loses it"
        );
        assert!(
            alice.history(&bob_id).last().unwrap().pending,
            "shows pending, not a scary error"
        );

        transport.set_fail_submit(false);
        alice.poll().unwrap();
        assert!(alice.outbox.is_empty(), "the queued send drained after recovery");
        let got: Vec<String> = bob.poll().unwrap().iter().map(|m| m.text.clone()).collect();
        assert_eq!(got, vec!["during the blip"]);
    }

    #[test]
    fn a_queued_send_survives_a_snapshot_restart() {
        // The single most user-visible durability promise: a message "sent" while offline survives an
        // app close/crash and is delivered after restart. Exercises AppSnapshot.outbox + post-restore
        // drain (the existing round-trip test never carries a NON-empty outbox).
        let key = [9u8; 32];
        let transport = FaultTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();

        // Relay unreachable → the send queues in the outbox (the user "sent" it while offline).
        transport.set_fail_submit(true);
        alice.send(&bob_id, "queued while offline").unwrap();
        assert_eq!(alice.outbox.len(), 1);

        // App closes with the unsent message: seal a snapshot, drop, restore a fresh controller.
        let blob = alice.to_encrypted_snapshot(&key);
        drop(alice);
        let mut restored =
            AppController::from_encrypted_snapshot(transport.clone(), &blob, &key).unwrap();

        // The queued send SURVIVED the restart.
        assert_eq!(
            restored.outbox.len(),
            1,
            "the queued send survived the snapshot round-trip"
        );
        assert!(
            restored.history(&bob_id).last().unwrap().pending,
            "still pending after restore"
        );

        // Relay recovers → a poll drains the RESTORED outbox; Bob receives the message.
        transport.set_fail_submit(false);
        restored.poll().unwrap();
        assert!(restored.outbox.is_empty(), "the restored outbox drains after recovery");
        let got: Vec<String> = bob.poll().unwrap().iter().map(|m| m.text.clone()).collect();
        assert_eq!(got, vec!["queued while offline"]);
    }

    #[test]
    fn attachments_ride_the_sealed_channel_end_to_end() {
        use base64::Engine as _;
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();

        let payload = b"fake-png-bytes".to_vec();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&payload);
        alice
            .send_attachment(&bob_id, "../..\\evil photo.png", "image/png", &b64)
            .unwrap();

        let got = bob.poll().unwrap();
        assert_eq!(got.len(), 1);
        let att = got[0].att.as_ref().expect("attachment present");
        // The hostile path was reduced to a safe basename on BOTH sides.
        assert_eq!(att.name, "evil photo.png");
        assert_eq!(att.mime, "image/png");
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(&att.data_b64)
                .unwrap(),
            payload,
            "bytes survive the sealed round-trip"
        );

        // Over the single-frame cap now CHUNKS (covered by its own test); only non-base64 and
        // over-TOTAL inputs fail closed here.
        assert_eq!(
            alice
                .send_attachment(&bob_id, "x", "", "@@not-base64@@")
                .unwrap_err(),
            AppError::AttachmentInvalid
        );
    }

    #[test]
    fn chunked_attachments_reassemble_end_to_end() {
        use base64::Engine as _;
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();

        // A 1.5 MiB payload -> multiple attc frames -> ONE reassembled attachment at Bob's end.
        let payload: Vec<u8> = (0..(1_536 * 1024)).map(|i| (i % 251) as u8).collect();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&payload);
        alice
            .send_attachment(&bob_id, "big.bin", "application/octet-stream", &b64)
            .unwrap();
        let got = bob.poll().unwrap();
        assert_eq!(got.len(), 1, "exactly one assembled attachment message");
        let att = got[0].att.as_ref().expect("attachment present");
        assert_eq!(
            base64::engine::general_purpose::STANDARD
                .decode(&att.data_b64)
                .unwrap(),
            payload,
            "bytes identical after chunked reassembly"
        );
        assert!(bob.pending_chunks.is_empty(), "no leftover partial state");

        // The completion receipt (over the last chunk) marks Alice's copy delivered.
        alice.poll().unwrap();
        assert!(alice.history(&bob_id).last().unwrap().delivered);

        // Over the TOTAL cap fails closed at the sender.
        let too_big =
            base64::engine::general_purpose::STANDARD
                .encode(vec![0u8; MAX_ATTACHMENT_TOTAL_BYTES + 1]);
        assert_eq!(
            alice
                .send_attachment(&bob_id, "x", "", &too_big)
                .unwrap_err(),
            AppError::AttachmentTooLarge
        );
    }

    #[test]
    fn read_receipts_flow_and_respect_the_privacy_toggle() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        let alice_id = alice.account_id();
        alice.add_contact(&bob_id).unwrap();

        alice.send(&bob_id, "see me?").unwrap();
        bob.poll().unwrap(); // delivered receipt goes back
        alice.poll().unwrap();
        assert!(alice.history(&bob_id).last().unwrap().delivered);
        assert!(!alice.history(&bob_id).last().unwrap().read);

        // Bob looks at the conversation -> read watermark -> Alice's copy turns read.
        bob.mark_read(&alice_id).unwrap();
        alice.poll().unwrap();
        assert!(alice.history(&bob_id).last().unwrap().read);

        // With Bob's toggle OFF, a later message never turns read (delivered still flows).
        bob.set_read_receipts(false);
        alice.send(&bob_id, "again").unwrap();
        bob.poll().unwrap();
        bob.mark_read(&alice_id).unwrap();
        alice.poll().unwrap();
        let last = alice.history(&bob_id).last().unwrap().clone();
        assert!(last.delivered);
        assert!(!last.read, "no read signal when the peer's toggle is off");
    }

    #[test]
    fn backup_export_import_round_trips_and_fails_closed() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();
        alice.send(&bob_id, "remember me").unwrap();
        alice.set_verified(&bob_id).unwrap();

        // A weak passphrase is refused outright.
        assert_eq!(
            alice.export_backup("short").unwrap_err(),
            AppError::WeakPassphrase
        );

        let backup = alice.export_backup("correct horse battery").unwrap();
        // Restore on a "new device" (same in-memory world): full state comes back.
        let restored =
            AppController::import_backup(transport.clone(), &backup, "correct horse battery")
                .unwrap();
        assert_eq!(restored.account_id(), alice.account_id());
        assert_eq!(restored.history(&bob_id).len(), 1);
        assert!(restored.verified.contains(&parse_id(&bob_id).unwrap()));

        // Wrong passphrase / truncated / foreign bytes all fail closed. (`unwrap_err` needs Debug
        // on the Ok type, which the controller deliberately lacks — match instead.)
        let err_of = |r: Result<AppController<InMemoryTransport>, AppError>| match r {
            Ok(_) => panic!("a bad backup/passphrase must NOT restore"),
            Err(e) => e,
        };
        assert_eq!(
            err_of(AppController::import_backup(
                transport.clone(),
                &backup,
                "wrong passphrase!"
            )),
            AppError::BackupInvalid
        );
        assert_eq!(
            err_of(AppController::import_backup(
                transport.clone(),
                &backup[..20],
                "correct horse battery"
            )),
            AppError::BackupInvalid
        );
        assert_eq!(
            err_of(AppController::import_backup(
                transport,
                b"junk",
                "correct horse battery"
            )),
            AppError::BackupInvalid
        );
    }

    #[test]
    fn current_export_is_argon2id_and_legacy_pbkdf2_backups_still_restore() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();
        alice.send(&bob_id, "legacy hello").unwrap();
        let pass = "correct horse battery";

        // The CURRENT export is the memory-hard MERCBAK2 (Argon2id) format with a self-describing
        // header carrying the exact parameters used.
        let v2 = alice.export_backup(pass).unwrap();
        assert!(v2.starts_with(BACKUP_MAGIC_V2));
        assert_eq!(v2[8], BACKUP_KDF_ARGON2ID);
        assert_eq!(
            u32::from_le_bytes([v2[9], v2[10], v2[11], v2[12]]),
            BACKUP_ARGON2_MEM_KIB
        );

        // Hand-build a LEGACY MERCBAK1 (PBKDF2) backup exactly as an older build would have, and
        // prove it STILL restores through the back-compat import path.
        let salt = [42u8; 16];
        let key = derive_backup_key(pass, &salt);
        let sealed = alice.to_encrypted_snapshot(&key);
        let mut legacy = Vec::new();
        legacy.extend_from_slice(BACKUP_MAGIC);
        legacy.extend_from_slice(&salt);
        legacy.extend_from_slice(&sealed);
        let restored = AppController::import_backup(transport.clone(), &legacy, pass).unwrap();
        assert_eq!(restored.account_id(), alice.account_id());
        assert_eq!(restored.history(&bob_id).len(), 1);

        // A MERCBAK2 header advertising absurd Argon2 memory is refused BEFORE any allocation
        // (fail closed — a hostile/corrupt file cannot OOM the importer).
        let err_of = |r: Result<AppController<InMemoryTransport>, AppError>| match r {
            Ok(_) => panic!("an out-of-range KDF header must NOT restore"),
            Err(e) => e,
        };
        let mut hostile = Vec::new();
        hostile.extend_from_slice(BACKUP_MAGIC_V2);
        hostile.push(BACKUP_KDF_ARGON2ID);
        hostile.extend_from_slice(&(BACKUP_ARGON2_MAX_MEM_KIB + 1).to_le_bytes());
        hostile.extend_from_slice(&BACKUP_ARGON2_ITERS.to_le_bytes());
        hostile.extend_from_slice(&BACKUP_ARGON2_LANES.to_le_bytes());
        hostile.extend_from_slice(&[0u8; 16]);
        hostile.extend_from_slice(&[0u8; 64]);
        assert_eq!(
            err_of(AppController::import_backup(transport, &hostile, pass)),
            AppError::BackupInvalid
        );
    }

    #[test]
    fn verified_aliases_and_username_survive_the_snapshot() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        alice.add_contact(&bob_id).unwrap();
        alice.set_verified(&bob_id).unwrap();
        alice
            .aliases
            .insert(parse_id(&bob_id).unwrap(), "bob".to_string());
        alice.my_username = Some("alice".to_string());
        alice.pin_kt_vrf_if_absent(&[9u8; 32]);
        // A later pin attempt must NOT overwrite the existing one.
        alice.pin_kt_vrf_if_absent(&[1u8; 32]);

        let key = [3u8; 32];
        let blob = alice.to_encrypted_snapshot(&key);
        let back = AppController::from_encrypted_snapshot(transport, &blob, &key).unwrap();
        assert!(back.verified.contains(&parse_id(&bob_id).unwrap()));
        assert_eq!(
            back.aliases.get(&parse_id(&bob_id).unwrap()).unwrap(),
            "bob"
        );
        assert_eq!(back.my_username.as_deref(), Some("alice"));
        assert_eq!(back.kt_vrf_pin.as_deref(), Some(&[9u8; 32][..]));
        // conversation_flags surfaces all of it to the UI.
        let flags = back.conversation_flags();
        assert_eq!(flags["verified"][0], json!(bob_id));
        assert_eq!(flags["aliases"][&bob_id], json!("bob"));
        assert_eq!(flags["my_username"], json!("alice"));
    }

    #[test]
    fn pairing_code_then_add_by_pairing_round_trips() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        let alice_id = alice.account_id();

        // Alice mints a pairing code; Bob redeems it and the response names whose card it was.
        let code = alice.pairing_code().unwrap()["code"]
            .as_str()
            .unwrap()
            .to_string();
        let added = bob.add_by_pairing(&code).unwrap();
        assert_eq!(added["account_id"], json!(alice_id));
        assert!(bob.can_send(&alice_id), "Bob can now reach Alice");

        // Single-use: the same code cannot be redeemed twice.
        assert!(bob.add_by_pairing(&code).is_err());
        // A nonsense code is a clean error, not a panic.
        assert!(bob.add_by_pairing("NOPE-NOPE").is_err());
    }

    #[test]
    fn claim_username_is_first_claimant_wins() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());

        // First claim creates; the same account re-confirming is idempotent.
        assert_eq!(
            alice.claim_username("Alice").unwrap()["status"],
            json!("created")
        );
        assert_eq!(
            alice.claim_username("alice").unwrap()["status"],
            json!("already_owned")
        );
        // A different account is refused (first-claimant-wins).
        assert_eq!(
            bob.claim_username("alice").unwrap()["status"],
            json!("taken")
        );
        // An illegal handle is rejected without a binding.
        assert_eq!(
            alice.claim_username("ab").unwrap()["status"],
            json!("rejected")
        );
    }

    #[test]
    fn add_by_username_fails_closed_without_a_verifiable_proof() {
        // The in-memory transport faithfully stores the registry + cards but cannot produce a real
        // key-transparency inclusion proof, so a username resolve MUST fail closed rather than trust
        // an unverified binding. (The real verified path is covered end-to-end in mercury-client's
        // `discovery` integration test against the running relay.)
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        assert_eq!(
            bob.claim_username("bob").unwrap()["status"],
            json!("created")
        );
        let bob_id = bob.account_id();

        // No directory key/proof is verifiable over the double -> the resolve errors, and crucially
        // NOTHING is added (no contact, no silently-trusted binding).
        assert!(alice.add_by_username("bob").is_err());
        assert!(!alice.can_send(&bob_id), "fail-closed: Bob was not added");
    }

    #[test]
    fn delete_conversation_and_clear_history_remove_local_messages() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();

        alice.add_contact(&bob_id).unwrap();
        alice.send(&bob_id, "hello bob").unwrap();
        assert_eq!(alice.history(&bob_id).len(), 1);

        // clear_history drops the messages but keeps the contact/session (still sendable).
        alice.clear_history(&bob_id).unwrap();
        assert!(alice.history(&bob_id).is_empty());
        assert!(
            alice.can_send(&bob_id),
            "clear_history must keep the contact/session"
        );

        // a fresh message logs again; delete_conversation then clears the log too.
        alice.send(&bob_id, "again").unwrap();
        assert_eq!(alice.history(&bob_id).len(), 1);
        alice.delete_conversation(&bob_id).unwrap();
        assert!(
            alice.history(&bob_id).is_empty(),
            "delete_conversation clears the log"
        );

        // Idempotent: deleting an already-gone conversation is a clean no-op, not an error.
        assert!(alice.delete_conversation(&bob_id).is_ok());
        assert!(alice.clear_history(&bob_id).is_ok());
    }

    #[test]
    fn snapshot_without_mute_block_fields_still_deserializes_empty() {
        // Backward-compat: a snapshot written BEFORE the muted/blocked fields existed must still
        // load, both defaulting to empty — so the AI participant's existing on-disk snapshot (and
        // every shipped 0.1.x user's) keeps working after this upgrade.
        let legacy =
            r#"{"client_pickle":[],"contacts":[],"log":[],"seq":0,"last_receive_ok":true}"#;
        let snap: AppSnapshot = serde_json::from_str(legacy).expect("legacy snapshot deserializes");
        assert!(snap.muted.is_empty());
        assert!(snap.blocked.is_empty());
    }

    #[test]
    fn block_refuses_send_drops_incoming_and_mute_survives_snapshot() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        let alice_id = alice.account_id();

        alice.add_contact(&bob_id).unwrap();
        alice.send(&bob_id, "hi bob").unwrap();
        assert_eq!(
            bob.poll().unwrap().len(),
            1,
            "first message delivers normally"
        );

        // Bob blocks Alice: Bob's send to Alice is refused, and Alice's next message is dropped on
        // poll (consumed to keep the ratchet in sync, but never logged/returned).
        bob.block(&alice_id).unwrap();
        assert_eq!(bob.send(&alice_id, "nope"), Err(AppError::Blocked));
        alice.send(&bob_id, "still there?").unwrap();
        assert_eq!(
            bob.poll().unwrap().len(),
            0,
            "blocked sender's message is dropped"
        );

        // Unblock restores delivery (the ratchet stayed synced through the dropped message).
        bob.unblock(&alice_id).unwrap();
        alice.send(&bob_id, "hello again").unwrap();
        assert_eq!(bob.poll().unwrap().len(), 1, "unblock restores delivery");

        // Mute shows in conversation_flags AND survives an encrypted-snapshot round-trip.
        bob.mute(&alice_id).unwrap();
        let muted_now = bob.conversation_flags();
        assert!(
            muted_now["muted"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str() == Some(alice_id.as_str())),
            "alice is muted"
        );
        let key = [9u8; 32];
        let blob = bob.to_encrypted_snapshot(&key);
        let bob2 = AppController::from_encrypted_snapshot(transport.clone(), &blob, &key).unwrap();
        assert!(
            bob2.conversation_flags()["muted"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str() == Some(alice_id.as_str())),
            "mute survives snapshot restore"
        );
    }

    #[test]
    fn delete_for_everyone_wipes_history_on_both_sides_with_a_notice() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        let alice_id = alice.account_id();

        // Build a two-message conversation present in BOTH logs.
        alice.add_contact(&bob_id).unwrap();
        alice.send(&bob_id, "hi").unwrap();
        bob.poll().unwrap();
        bob.send(&alice_id, "hey").unwrap();
        alice.poll().unwrap();
        assert_eq!(alice.history(&bob_id).len(), 2);
        assert_eq!(bob.history(&alice_id).len(), 2);

        // Alice deletes for everyone → gone on her side immediately.
        alice.delete_for_everyone(&bob_id).unwrap();
        assert!(
            alice.history(&bob_id).is_empty(),
            "removed on the sender's side"
        );

        // Bob polls → receives the sealed control, his history is wiped, exactly one notice surfaces
        // (rendered as a normal incoming line, NOT a stray control payload).
        let fresh = bob.poll().unwrap();
        assert_eq!(fresh.len(), 1, "exactly the deletion notice");
        assert!(fresh[0].text.contains("deleted this conversation"));
        let hist = bob.history(&alice_id);
        assert_eq!(
            hist.len(),
            1,
            "prior messages wiped; only the notice remains"
        );
        assert!(hist[0].text.contains("deleted this conversation"));
    }

    #[test]
    fn delete_for_everyone_all_wipes_every_conversation_and_reports_counts() {
        use mercury_client::InMemoryTransport;
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        let mut carol = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        carol.publish_self().unwrap();
        let bob_id = bob.account_id();
        let carol_id = carol.account_id();
        let alice_id = alice.account_id();

        alice.add_contact(&bob_id).unwrap();
        alice.add_contact(&carol_id).unwrap();
        alice.send(&bob_id, "hi bob").unwrap();
        alice.send(&carol_id, "hi carol").unwrap();
        bob.poll().unwrap();
        carol.poll().unwrap();
        assert_eq!(alice.history(&bob_id).len(), 1);
        assert_eq!(alice.history(&carol_id).len(), 1);

        // "Also wipe from peers": every 1:1 gets a delete-for-everyone; nothing left on this side.
        let report = alice.delete_for_everyone_all();
        assert_eq!(report["peers_wiped"].as_u64(), Some(2));
        assert_eq!(report["groups_ended"].as_u64(), Some(0));
        assert_eq!(report["failures"].as_u64(), Some(0));
        assert!(alice.history(&bob_id).is_empty());
        assert!(alice.history(&carol_id).is_empty());

        // Each reachable peer receives the deletion control and wipes their copy.
        let b = bob.poll().unwrap();
        assert!(
            b.iter()
                .any(|m| m.text.contains("deleted this conversation"))
        );
        assert!(
            bob.history(&alice_id)
                .iter()
                .all(|m| m.text.contains("deleted this conversation")),
            "bob's prior messages from alice are wiped; only the notice remains"
        );
        let c = carol.poll().unwrap();
        assert!(
            c.iter()
                .any(|m| m.text.contains("deleted this conversation"))
        );
    }

    #[test]
    fn disappearing_expiry_logic_and_timer_propagation() {
        use mercury_client::InMemoryTransport;
        use std::collections::HashMap;

        // Pure expiry logic.
        let peer = [7u8; ID_LEN];
        let peer_hex = hex_encode(&peer);
        let mk = |ts: i64| ChatMessage {
            direction: Direction::Incoming,
            peer: peer_hex.clone(),
            text: "x".to_string(),
            seq: 0,
            ts,
            delivered: false,
            pending: false,
            att: None,
            digest: String::new(),
            from: None,
            read: false,
        };
        let mut dis: HashMap<[u8; ID_LEN], u64> = HashMap::new();
        assert!(
            !msg_expired(&dis, &mk(100), 1_000_000),
            "no timer -> never expires"
        );
        dis.insert(peer, 3600); // 1h
        assert!(
            !msg_expired(&dis, &mk(0), 1_000_000),
            "ts=0 (legacy) never expires"
        );
        assert!(
            !msg_expired(&dis, &mk(999_990), 1_000_000),
            "10s old < 1h -> kept"
        );
        assert!(
            msg_expired(&dis, &mk(996_000), 1_000_000),
            "4000s old > 1h -> expired"
        );

        // Timer propagates to the peer via the sealed control message + shows in flags.
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        let alice_id = alice.account_id();

        alice.add_contact(&bob_id).unwrap();
        alice.set_disappearing(&bob_id, 86_400).unwrap(); // 1 day
        assert_eq!(
            alice.conversation_flags()["disappearing"][bob_id.as_str()].as_u64(),
            Some(86_400)
        );
        // Bob receives the timer control: no visible message, but his side records the same TTL.
        assert!(
            bob.poll().unwrap().is_empty(),
            "timer control is not a chat bubble"
        );
        assert_eq!(
            bob.conversation_flags()["disappearing"][alice_id.as_str()].as_u64(),
            Some(86_400)
        );

        // Clearing (secs=0) removes it.
        alice.set_disappearing(&bob_id, 0).unwrap();
        assert!(
            alice.conversation_flags()["disappearing"]
                .as_object()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn encrypted_snapshot_round_trips_identity_sessions_contacts_and_log() {
        use mercury_client::InMemoryTransport;
        let key = [7u8; 32];
        let transport = InMemoryTransport::new();
        let mut alice = AppController::new(transport.clone());
        let mut bob = AppController::new(transport.clone());
        bob.publish_self().unwrap();
        let bob_id = bob.account_id();
        let alice_id = alice.account_id();

        // Alice verifies Bob and sends — this opens a real ratchet session + logs the message.
        alice.add_contact(&bob_id).unwrap();
        let sent = alice.handle_command(&format!(
            "{{\"cmd\":\"send\",\"peer\":\"{bob_id}\",\"text\":\"before snapshot\"}}"
        ));
        assert!(sent.contains("\"ok\":true"), "send failed: {sent}");

        // Snapshot Alice, then restore a FRESH controller from the sealed blob.
        let blob = alice.to_encrypted_snapshot(&key);
        let mut alice2 =
            AppController::from_encrypted_snapshot(transport.clone(), &blob, &key).unwrap();

        // Identity is STABLE across the restart (the whole point — not a brand-new account).
        assert_eq!(alice2.account_id(), alice_id);
        // The message log survived.
        let hist = alice2.handle_command(&format!("{{\"cmd\":\"history\",\"peer\":\"{bob_id}\"}}"));
        assert!(hist.contains("before snapshot"), "log not restored: {hist}");
        // The ratchet SESSION survived: alice2 keeps sending on the RESTORED session, and Bob
        // decrypts both the pre-snapshot message and the post-restore one (in order).
        let sent2 = alice2.handle_command(&format!(
            "{{\"cmd\":\"send\",\"peer\":\"{bob_id}\",\"text\":\"after restore\"}}"
        ));
        assert!(
            sent2.contains("\"ok\":true"),
            "post-restore send failed: {sent2}"
        );
        let polled = bob.handle_command("{\"cmd\":\"poll\"}");
        assert!(
            polled.contains("before snapshot"),
            "bob missed pre-snapshot: {polled}"
        );
        assert!(
            polled.contains("after restore"),
            "bob missed post-restore: {polled}"
        );

        // The WRONG key fails closed — no partial restore, no panic.
        let wrong = [9u8; 32];
        assert_eq!(
            AppController::from_encrypted_snapshot(transport.clone(), &blob, &wrong).err(),
            Some(AppError::Persistence)
        );
        // A truncated/corrupt blob also fails closed.
        assert_eq!(
            AppController::from_encrypted_snapshot(transport, &blob[..blob.len() / 2], &key).err(),
            Some(AppError::Persistence)
        );
    }
}
