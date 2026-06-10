//! An Ed25519 SIGNED CHECKPOINT over the audit log's Merkle root — the audit
//! analogue of an RFC 6962 / Certificate Transparency signed tree head. The
//! sealed-audit event-chain gate requires a signed checkpoint
//! (`checkpoint_signature_len == 64`, `checkpoint_timestamp_s > 0`); this module
//! produces and verifies it.
//!
//! The signed preimage is canonical and DOMAIN-SEPARATED (a dedicated audit
//! domain distinct from mercury-kt's key-transparency STH domain), so an audit
//! checkpoint signature can never be replayed as a KT tree head or vice versa.

use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey};

/// Audit-checkpoint signing domain (NUL-terminated, distinct from KT's STH domain).
const CHECKPOINT_DOMAIN: &[u8] = b"mercury/audit/checkpoint/v1\0";

/// A checkpoint committing the audit Merkle tree at a point in time: the tree
/// `tree_size`, its Merkle `root_hash`, and a `timestamp_s`. Signed by the log's
/// Ed25519 key; the size + root let a verifier bind inclusion/consistency proofs
/// to an authenticated head.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealedAuditCheckpoint {
    pub tree_size: u64,
    pub root_hash: [u8; 32],
    pub timestamp_s: i64,
}

impl SealedAuditCheckpoint {
    /// The canonical, domain-separated signing preimage that the log key signs and
    /// any verifier reconstructs: `DOMAIN || tree_size(LE) || root_hash || timestamp(LE)`.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(CHECKPOINT_DOMAIN.len() + 8 + 32 + 8);
        bytes.extend_from_slice(CHECKPOINT_DOMAIN);
        bytes.extend_from_slice(&self.tree_size.to_le_bytes());
        bytes.extend_from_slice(&self.root_hash);
        bytes.extend_from_slice(&self.timestamp_s.to_le_bytes());
        bytes
    }
}

/// Sign a checkpoint with the log's Ed25519 `signing_key`, returning the 64-byte
/// signature over the canonical preimage.
pub fn sign_checkpoint(signing_key: &SigningKey, checkpoint: &SealedAuditCheckpoint) -> [u8; 64] {
    signing_key.sign(&checkpoint.signing_bytes()).to_bytes()
}

/// Verify a checkpoint signature against the log's `verifying_key` (strict
/// verification — rejects non-canonical / malleable signatures). Fails closed on
/// any tampering of the checkpoint or signature, or a wrong key.
pub fn verify_checkpoint(
    verifying_key: &VerifyingKey,
    checkpoint: &SealedAuditCheckpoint,
    signature: &[u8; 64],
) -> bool {
    let signature = Signature::from_bytes(signature);
    verifying_key
        .verify_strict(&checkpoint.signing_bytes(), &signature)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn key() -> SigningKey {
        // Deterministic test key (NOT for production); production uses an OS-CSPRNG
        // key persisted alongside the log.
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn checkpoint() -> SealedAuditCheckpoint {
        SealedAuditCheckpoint {
            tree_size: 42,
            root_hash: [0x5a; 32],
            timestamp_s: 1_700_000_000,
        }
    }

    #[test]
    fn sign_and_verify_round_trip() {
        let signing_key = key();
        let cp = checkpoint();
        let sig = sign_checkpoint(&signing_key, &cp);
        assert_eq!(sig.len(), 64);
        assert!(verify_checkpoint(&signing_key.verifying_key(), &cp, &sig));
    }

    #[test]
    fn a_tampered_checkpoint_fails() {
        let signing_key = key();
        let vk = signing_key.verifying_key();
        let cp = checkpoint();
        let sig = sign_checkpoint(&signing_key, &cp);

        // Tampered root.
        let mut bad_root = cp;
        bad_root.root_hash[0] ^= 0x01;
        assert!(!verify_checkpoint(&vk, &bad_root, &sig));
        // Tampered size.
        let bad_size = SealedAuditCheckpoint {
            tree_size: 43,
            ..cp
        };
        assert!(!verify_checkpoint(&vk, &bad_size, &sig));
        // Tampered timestamp.
        let bad_ts = SealedAuditCheckpoint {
            timestamp_s: cp.timestamp_s + 1,
            ..cp
        };
        assert!(!verify_checkpoint(&vk, &bad_ts, &sig));
    }

    #[test]
    fn a_tampered_signature_fails() {
        let signing_key = key();
        let cp = checkpoint();
        let mut sig = sign_checkpoint(&signing_key, &cp);
        sig[0] ^= 0x01;
        assert!(!verify_checkpoint(&signing_key.verifying_key(), &cp, &sig));
    }

    #[test]
    fn a_wrong_key_fails() {
        let cp = checkpoint();
        let sig = sign_checkpoint(&key(), &cp);
        let other = SigningKey::from_bytes(&[9u8; 32]);
        assert!(!verify_checkpoint(&other.verifying_key(), &cp, &sig));
    }

    #[test]
    fn domain_separation_changes_the_preimage() {
        // The signing preimage must begin with the audit domain (so it cannot be
        // confused with a KT STH preimage).
        let cp = checkpoint();
        assert!(cp.signing_bytes().starts_with(CHECKPOINT_DOMAIN));
    }
}
