//! Hybrid Ed25519 + ML-DSA-44 signed checkpoint — the post-quantum / hybrid
//! checkpoint signature mercury-core's sealed-audit WITNESS gate requires
//! (`SealedAuditCheckpointSignatureAlgorithm::is_post_quantum_or_hybrid`).
//!
//! The signature is the concatenation of a classical Ed25519 signature (64 bytes)
//! and an ML-DSA-44 / FIPS 204 signature (2420 bytes) = 2484 bytes, matching
//! `SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44`
//! (`checkpoint_signature_min_len() == 2484`). Verification requires BOTH halves to
//! verify (a logical AND), so a forged checkpoint must break BOTH a battle-tested
//! classical scheme AND the post-quantum scheme. The classical Ed25519 floor
//! protects against any latent flaw in the (newer) ML-DSA implementation, while the
//! ML-DSA half protects against a future quantum break of Ed25519. This is the
//! IETF/NIST composite pattern (`id-MLDSA44-Ed25519`), adapted to sign the audit
//! checkpoint preimage directly.
//!
//! ML-DSA-44 is RustCrypto's `ml-dsa` (pure Rust, FIPS 204) — deliberately the same
//! crypto supply chain as mercury-keys' `ml-kem`, so the workspace has one
//! post-quantum vendor / audit surface rather than two. The hybrid construction
//! means any choice of ML-DSA backend keeps the Ed25519 security floor. Both halves
//! sign the SAME domain-separated checkpoint preimage
//! (`SealedAuditCheckpoint::signing_bytes`), so neither half can be replayed for a
//! different checkpoint.
//!
//! **Verified-backend evaluation (libcrux-ml-dsa).** Cryspen's `libcrux-ml-dsa` is
//! formally verified (hax/F*), pure-Rust with a portable backend (no AVX2 required,
//! builds on Windows), implements FIPS-204 ML-DSA-44 with the SAME 2420-byte
//! signature / 1312-byte verifying-key sizes — so the 2484-byte hybrid framing here
//! would be unchanged. It was evaluated as a swap-in for the un-audited RustCrypto
//! `ml-dsa`. It is DEFERRED, not adopted, because its API differs materially: it
//! exposes free functions over `MLDSAKeyPair`/`MLDSASignature` taking EXPLICIT
//! caller-supplied randomness (`KEY_GENERATION_RANDOMNESS_SIZE` /
//! `SIGNING_RANDOMNESS_SIZE`), rather than the RustCrypto `signature::{Generate,
//! Signer, Verifier, Keypair}` trait API this module is built on. Swapping would
//! rewrite the keygen/sign/verify calls and the randomness model inside this
//! 5×-certified hybrid checkpoint crate — risk not justified while the Ed25519 floor
//! already mitigates ml-dsa being un-audited. Revisit if RustCrypto `ml-dsa` is
//! found wanting or a verified backend becomes a hard requirement.

use ed25519_dalek::{
    Signature as EdSignature, Signer as _, SigningKey as EdSigningKey,
    VerifyingKey as EdVerifyingKey,
};
use ml_dsa::signature::{Keypair as _, Signer as _, Verifier as _};
use ml_dsa::{
    Generate as _, MlDsa44, Signature as MlSignature, SigningKey as MlSigningKey,
    VerifyingKey as MlVerifyingKey,
};
use rand_core::OsRng;

use crate::checkpoint::SealedAuditCheckpoint;

/// Ed25519 signature length (the classical half).
const ED25519_SIG_LEN: usize = 64;
/// ML-DSA-44 signature length (the post-quantum half).
const ML_DSA_44_SIG_LEN: usize = 2420;
/// Hybrid checkpoint signature length = Ed25519 then ML-DSA-44 = 2484 bytes. Matches
/// `SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44`.
pub const HYBRID_CHECKPOINT_SIG_LEN: usize = ED25519_SIG_LEN + ML_DSA_44_SIG_LEN;

/// Ed25519 public-key length.
const ED25519_VK_LEN: usize = 32;
/// ML-DSA-44 public-key length.
const ML_DSA_44_VK_LEN: usize = 1312;
/// Hybrid verifying-key length = Ed25519 then ML-DSA-44 = 1344 bytes (for pinning).
pub const HYBRID_VK_LEN: usize = ED25519_VK_LEN + ML_DSA_44_VK_LEN;

/// A hybrid checkpoint SIGNING key: an Ed25519 key and an ML-DSA-44 key. The log
/// holds this and signs every checkpoint with both. The Ed25519 half zeroizes on
/// drop (ed25519-dalek `zeroize` feature).
pub struct HybridCheckpointSigningKey {
    ed: EdSigningKey,
    ml: MlSigningKey<MlDsa44>,
}

impl HybridCheckpointSigningKey {
    /// Generate a fresh hybrid key from the OS CSPRNG (both halves independently).
    pub fn generate() -> Self {
        let ed = EdSigningKey::generate(&mut OsRng);
        let ml = MlSigningKey::<MlDsa44>::generate();
        Self { ed, ml }
    }

    /// Construct from existing halves (e.g. keys persisted out of band).
    pub fn from_parts(ed: EdSigningKey, ml: MlSigningKey<MlDsa44>) -> Self {
        Self { ed, ml }
    }

    /// The hybrid verifying key to pin out of band.
    pub fn verifying_key(&self) -> HybridCheckpointVerifyingKey {
        HybridCheckpointVerifyingKey {
            ed: self.ed.verifying_key(),
            ml: self.ml.verifying_key(),
        }
    }
}

/// A hybrid checkpoint VERIFYING key: the Ed25519 and ML-DSA-44 public keys a
/// verifier pins out of band to check checkpoint signatures.
#[derive(Clone)]
pub struct HybridCheckpointVerifyingKey {
    ed: EdVerifyingKey,
    ml: MlVerifyingKey<MlDsa44>,
}

impl HybridCheckpointVerifyingKey {
    /// Construct from existing public-key halves.
    pub fn from_parts(ed: EdVerifyingKey, ml: MlVerifyingKey<MlDsa44>) -> Self {
        Self { ed, ml }
    }

    /// The canonical pinnable encoding: Ed25519 public key (32 bytes) then ML-DSA-44
    /// public key (1312 bytes) = 1344 bytes.
    pub fn to_bytes(&self) -> [u8; HYBRID_VK_LEN] {
        let ed = self.ed.to_bytes();
        let ml_enc = self.ml.encode();
        let ml: &[u8] = ml_enc.as_ref();
        let mut out = [0u8; HYBRID_VK_LEN];
        out[..ED25519_VK_LEN].copy_from_slice(&ed);
        out[ED25519_VK_LEN..].copy_from_slice(ml);
        out
    }
}

/// Sign an arbitrary `message` with the hybrid key: Ed25519 signature then
/// ML-DSA-44 signature, both over the SAME `message` bytes. Returns the fixed
/// 2484-byte hybrid signature. This is the core both the checkpoint signer and the
/// witness cosigner use, so every hybrid signature in the audit log is the same
/// Ed25519-AND-ML-DSA construction.
pub fn sign_message_hybrid(
    signing_key: &HybridCheckpointSigningKey,
    message: &[u8],
) -> [u8; HYBRID_CHECKPOINT_SIG_LEN] {
    let ed_sig: [u8; ED25519_SIG_LEN] = signing_key.ed.sign(message).to_bytes();
    let ml_sig = signing_key.ml.sign(message);
    let ml_enc = ml_sig.encode();
    let ml_bytes: &[u8] = ml_enc.as_ref();

    let mut out = [0u8; HYBRID_CHECKPOINT_SIG_LEN];
    out[..ED25519_SIG_LEN].copy_from_slice(&ed_sig);
    out[ED25519_SIG_LEN..].copy_from_slice(ml_bytes);
    out
}

/// Verify a hybrid signature over an arbitrary `message`: BOTH the Ed25519 half
/// (strict — rejects non-canonical / malleable) AND the ML-DSA-44 half must verify
/// over the SAME `message`. Fails closed on a wrong length, a malformed half, a
/// wrong key, a tampered message, or either half failing — a forgery must break
/// BOTH schemes.
pub fn verify_message_hybrid(
    verifying_key: &HybridCheckpointVerifyingKey,
    message: &[u8],
    signature: &[u8],
) -> bool {
    if signature.len() != HYBRID_CHECKPOINT_SIG_LEN {
        return false;
    }
    let (ed_bytes, ml_bytes) = signature.split_at(ED25519_SIG_LEN);

    // Ed25519 half (strict verification).
    let Ok(ed_arr) = <&[u8; ED25519_SIG_LEN]>::try_from(ed_bytes) else {
        return false;
    };
    let ed_sig = EdSignature::from_bytes(ed_arr);
    let ed_ok = verifying_key.ed.verify_strict(message, &ed_sig).is_ok();

    // ML-DSA-44 half.
    let ml_ok = match MlSignature::<MlDsa44>::try_from(ml_bytes) {
        Ok(ml_sig) => verifying_key.ml.verify(message, &ml_sig).is_ok(),
        Err(_) => false,
    };

    // BOTH halves must verify.
    ed_ok && ml_ok
}

/// Sign `checkpoint` with the hybrid key over its canonical domain-separated
/// [`SealedAuditCheckpoint::signing_bytes`] preimage. Thin wrapper over
/// [`sign_message_hybrid`].
pub fn sign_checkpoint_hybrid(
    signing_key: &HybridCheckpointSigningKey,
    checkpoint: &SealedAuditCheckpoint,
) -> [u8; HYBRID_CHECKPOINT_SIG_LEN] {
    sign_message_hybrid(signing_key, &checkpoint.signing_bytes())
}

/// Verify a hybrid checkpoint signature over the canonical checkpoint preimage.
/// Thin wrapper over [`verify_message_hybrid`]; BOTH halves must verify.
pub fn verify_checkpoint_hybrid(
    verifying_key: &HybridCheckpointVerifyingKey,
    checkpoint: &SealedAuditCheckpoint,
    signature: &[u8],
) -> bool {
    verify_message_hybrid(verifying_key, &checkpoint.signing_bytes(), signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint() -> SealedAuditCheckpoint {
        SealedAuditCheckpoint {
            tree_size: 42,
            root_hash: [0x5a; 32],
            timestamp_s: 1_700_000_000,
        }
    }

    #[test]
    fn sign_and_verify_round_trip() {
        let sk = HybridCheckpointSigningKey::generate();
        let vk = sk.verifying_key();
        let cp = checkpoint();
        let sig = sign_checkpoint_hybrid(&sk, &cp);
        // The hybrid signature has the exact length mercury-core's gate expects.
        assert_eq!(sig.len(), HYBRID_CHECKPOINT_SIG_LEN);
        assert_eq!(sig.len(), 2484);
        assert!(verify_checkpoint_hybrid(&vk, &cp, &sig));
        assert_eq!(vk.to_bytes().len(), HYBRID_VK_LEN);
        assert_eq!(vk.to_bytes().len(), 1344);
    }

    #[test]
    fn a_tampered_ed25519_half_fails() {
        let sk = HybridCheckpointSigningKey::generate();
        let vk = sk.verifying_key();
        let cp = checkpoint();
        let mut sig = sign_checkpoint_hybrid(&sk, &cp);
        sig[0] ^= 0x01; // flip a bit in the Ed25519 half
        assert!(!verify_checkpoint_hybrid(&vk, &cp, &sig));
    }

    #[test]
    fn a_tampered_ml_dsa_half_fails() {
        let sk = HybridCheckpointSigningKey::generate();
        let vk = sk.verifying_key();
        let cp = checkpoint();
        let mut sig = sign_checkpoint_hybrid(&sk, &cp);
        let last = sig.len() - 1;
        sig[last] ^= 0x01; // flip a bit in the ML-DSA half
        assert!(!verify_checkpoint_hybrid(&vk, &cp, &sig));
    }

    #[test]
    fn a_wrong_key_fails() {
        let sk = HybridCheckpointSigningKey::generate();
        let other = HybridCheckpointSigningKey::generate();
        let cp = checkpoint();
        let sig = sign_checkpoint_hybrid(&sk, &cp);
        assert!(!verify_checkpoint_hybrid(&other.verifying_key(), &cp, &sig));
    }

    #[test]
    fn a_tampered_checkpoint_fails() {
        let sk = HybridCheckpointSigningKey::generate();
        let vk = sk.verifying_key();
        let cp = checkpoint();
        let sig = sign_checkpoint_hybrid(&sk, &cp);
        let mut bad = cp;
        bad.root_hash[0] ^= 0x01;
        assert!(!verify_checkpoint_hybrid(&vk, &bad, &sig));
        let bad_size = SealedAuditCheckpoint {
            tree_size: cp.tree_size + 1,
            ..cp
        };
        assert!(!verify_checkpoint_hybrid(&vk, &bad_size, &sig));
    }

    #[test]
    fn a_wrong_length_signature_fails() {
        let sk = HybridCheckpointSigningKey::generate();
        let vk = sk.verifying_key();
        let cp = checkpoint();
        let sig = sign_checkpoint_hybrid(&sk, &cp);
        // Truncated, over-long, and empty all fail closed.
        assert!(!verify_checkpoint_hybrid(&vk, &cp, &sig[..sig.len() - 1]));
        let mut extended = sig.to_vec();
        extended.push(0u8);
        assert!(!verify_checkpoint_hybrid(&vk, &cp, &extended));
        assert!(!verify_checkpoint_hybrid(&vk, &cp, &[]));
    }

    #[test]
    fn both_halves_are_load_bearing() {
        // Splice a VALID ed half over checkpoint A with a VALID ml half over a
        // DIFFERENT checkpoint B. Each half is individually a real signature, but
        // verifying over A must FAIL because the ml half binds B — proving the ml
        // half genuinely binds the message (not just a structural check), and that
        // verification is an AND. The symmetric splice proves the ed half is
        // load-bearing too.
        let sk = HybridCheckpointSigningKey::generate();
        let vk = sk.verifying_key();
        let cp_a = checkpoint();
        let cp_b = SealedAuditCheckpoint {
            tree_size: 99,
            root_hash: [0x11; 32],
            timestamp_s: 1_800_000_000,
        };
        let sig_a = sign_checkpoint_hybrid(&sk, &cp_a);
        let sig_b = sign_checkpoint_hybrid(&sk, &cp_b);

        // ed(A) then ml(B), verified over A -> ml half is over B -> fails.
        let mut ed_a_ml_b = [0u8; HYBRID_CHECKPOINT_SIG_LEN];
        ed_a_ml_b[..ED25519_SIG_LEN].copy_from_slice(&sig_a[..ED25519_SIG_LEN]);
        ed_a_ml_b[ED25519_SIG_LEN..].copy_from_slice(&sig_b[ED25519_SIG_LEN..]);
        assert!(
            !verify_checkpoint_hybrid(&vk, &cp_a, &ed_a_ml_b),
            "ml half must be load-bearing"
        );

        // ed(B) then ml(A), verified over A -> ed half is over B -> fails.
        let mut ed_b_ml_a = [0u8; HYBRID_CHECKPOINT_SIG_LEN];
        ed_b_ml_a[..ED25519_SIG_LEN].copy_from_slice(&sig_b[..ED25519_SIG_LEN]);
        ed_b_ml_a[ED25519_SIG_LEN..].copy_from_slice(&sig_a[ED25519_SIG_LEN..]);
        assert!(
            !verify_checkpoint_hybrid(&vk, &cp_a, &ed_b_ml_a),
            "ed half must be load-bearing"
        );

        // Sanity: the genuine full signature over A still verifies.
        assert!(verify_checkpoint_hybrid(&vk, &cp_a, &sig_a));
    }
}
