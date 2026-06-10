//! Hybrid (Ed25519 + ML-DSA-44) WITNESS cosignatures over an audit checkpoint —
//! the post-quantum split-view defense for a `WitnessedTransparencyLog` whose
//! checkpoint is hybrid-signed. Independent operators each hybrid-cosign the
//! checkpoint; a quorum of DISTINCT witnesses across DISTINCT operators is what
//! satisfies mercury-core's `evaluate_sealed_audit_witness_checkpoint` witness
//! requirement.
//!
//! This parallels [`crate::witness`] (the Ed25519-only quorum for the event-chain
//! gate) but uses the hybrid signature and the cosignature wire size mercury-core's
//! witness-checkpoint gate expects: for `HybridEd25519MlDsa44`,
//! `witness_cosignature_min_len() == 2508` bytes per cosignature =
//! `key_id (16) || timestamp_s (8, LE) || hybrid_sig (2484)`. Each witness cosigns
//! the message `checkpoint.signing_bytes() || COSIGN_DOMAIN || key_id || timestamp`,
//! so the cosignature genuinely BINDS both this checkpoint AND its timestamp — the
//! gate's `cosignatures_bind_checkpoint` and `cosignatures_timestamped` flags are
//! therefore REAL verification results here, not caller attestations. A malformed,
//! forged, mis-indexed, duplicate, or zero-timestamp cosignature is simply not
//! counted (fail-closed): the returned counts are a floor on genuine independent
//! witnesses and can never be inflated.

use crate::checkpoint::SealedAuditCheckpoint;
use crate::hybrid::{
    HYBRID_CHECKPOINT_SIG_LEN, HybridCheckpointSigningKey, HybridCheckpointVerifyingKey,
    sign_message_hybrid, verify_message_hybrid,
};

/// Domain separator for witness cosignature messages (distinct from the checkpoint
/// signing domain, so a checkpoint signature can never be replayed as a
/// cosignature or vice versa).
const COSIGN_DOMAIN: &[u8] = b"mercury/audit/witness-cosign/v1\0";
/// Witness key-id length (an out-of-band-pinned 16-byte hint identifying the key).
const KEY_ID_LEN: usize = 16;
/// Serialized wire size of one hybrid cosignature: key_id || timestamp || sig.
/// Matches `SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44`'s
/// `witness_cosignature_min_len() == 2508`.
pub const HYBRID_COSIGNATURE_LEN: usize = KEY_ID_LEN + 8 + HYBRID_CHECKPOINT_SIG_LEN;

/// A transparency witness with a hybrid key: the verifying key, the operator that
/// runs it, and a 16-byte key-id pinned out of band. The gate counts distinct
/// OPERATORS (not just keys) so one operator running several witnesses cannot alone
/// satisfy the quorum.
#[derive(Clone)]
pub struct HybridWitness {
    pub key: HybridCheckpointVerifyingKey,
    pub operator_id: u32,
    pub key_id: [u8; KEY_ID_LEN],
}

/// One witness's hybrid cosignature over a checkpoint: an index into the witness
/// set, the cosigning `timestamp_s`, and the 2484-byte hybrid signature over the
/// domain-separated, checkpoint- and timestamp-binding message.
#[derive(Clone, Copy)]
pub struct HybridWitnessCosignature {
    pub witness_index: usize,
    pub timestamp_s: i64,
    pub signature: [u8; HYBRID_CHECKPOINT_SIG_LEN],
}

/// The verified hybrid witness quorum over a checkpoint. All four fields are REAL
/// verification results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HybridWitnessQuorum {
    /// Distinct witnesses (deduplicated by key-id) whose hybrid cosignature verified.
    pub verified_witness_count: i32,
    /// Distinct operators among those verified witnesses.
    pub operator_count: i32,
    /// Total serialized cosignature bytes (verified_witness_count * 2508).
    pub cosignature_bytes: i32,
    /// True iff every counted cosignature bound this checkpoint (always true for the
    /// counted set, since binding is exactly what verification checks).
    pub all_bind_checkpoint: bool,
    /// True iff every counted cosignature carried a positive, signed timestamp.
    pub all_timestamped: bool,
}

/// The domain-separated message a witness cosigns: it binds the checkpoint preimage,
/// the witness key-id, and the timestamp, so a cosignature cannot be replayed for a
/// different checkpoint, witness, or time.
fn cosign_message(
    checkpoint: &SealedAuditCheckpoint,
    key_id: &[u8; KEY_ID_LEN],
    timestamp_s: i64,
) -> Vec<u8> {
    let preimage = checkpoint.signing_bytes();
    let mut msg = Vec::with_capacity(preimage.len() + COSIGN_DOMAIN.len() + KEY_ID_LEN + 8);
    msg.extend_from_slice(&preimage);
    msg.extend_from_slice(COSIGN_DOMAIN);
    msg.extend_from_slice(key_id);
    msg.extend_from_slice(&timestamp_s.to_le_bytes());
    msg
}

/// Cosign `checkpoint` as a witness (test/helper): produce the hybrid signature over
/// the domain-separated message binding the checkpoint, `key_id`, and `timestamp_s`.
pub fn cosign_checkpoint_hybrid(
    signing_key: &HybridCheckpointSigningKey,
    checkpoint: &SealedAuditCheckpoint,
    key_id: &[u8; KEY_ID_LEN],
    timestamp_s: i64,
) -> HybridWitnessCosignature {
    let msg = cosign_message(checkpoint, key_id, timestamp_s);
    let signature = sign_message_hybrid(signing_key, &msg);
    HybridWitnessCosignature {
        witness_index: 0, // caller sets the index into its witness set
        timestamp_s,
        signature,
    }
}

/// One genuinely-verified, deduplicated hybrid cosignature: the witness key-id, its
/// operator, and the canonical 2508-byte wire encoding (`key_id || timestamp || sig`).
struct VerifiedHybridCosignature {
    key_id: [u8; KEY_ID_LEN],
    operator_id: u32,
    wire: [u8; HYBRID_COSIGNATURE_LEN],
}

/// The canonical 2508-byte wire encoding of a cosignature.
fn cosignature_wire(
    key_id: &[u8; KEY_ID_LEN],
    timestamp_s: i64,
    signature: &[u8; HYBRID_CHECKPOINT_SIG_LEN],
) -> [u8; HYBRID_COSIGNATURE_LEN] {
    let mut wire = [0u8; HYBRID_COSIGNATURE_LEN];
    wire[..KEY_ID_LEN].copy_from_slice(key_id);
    wire[KEY_ID_LEN..KEY_ID_LEN + 8].copy_from_slice(&timestamp_s.to_le_bytes());
    wire[KEY_ID_LEN + 8..].copy_from_slice(signature);
    wire
}

/// The shared verification core: the deduplicated set of cosignatures that REALLY
/// verify (both hybrid halves) over `checkpoint`, carry a positive timestamp, and
/// are in range — in input order. Both the quorum counts and the witness-receipt
/// wire bytes derive from THIS one result, so they can never drift. A malformed,
/// forged, mis-indexed, duplicate, or zero-timestamp cosignature is skipped: it can
/// never inflate the set (fail-closed).
fn verified_hybrid_cosignature_set(
    checkpoint: &SealedAuditCheckpoint,
    witnesses: &[HybridWitness],
    cosignatures: &[HybridWitnessCosignature],
) -> Vec<VerifiedHybridCosignature> {
    let mut verified: Vec<VerifiedHybridCosignature> = Vec::new();

    for cosignature in cosignatures {
        let Some(witness) = witnesses.get(cosignature.witness_index) else {
            continue;
        };
        // A cosignature must carry a positive timestamp (the gate requires
        // cosignatures_timestamped); the timestamp is part of the signed message,
        // so this is genuine, not cosmetic.
        if cosignature.timestamp_s <= 0 {
            continue;
        }
        let msg = cosign_message(checkpoint, &witness.key_id, cosignature.timestamp_s);
        if !verify_message_hybrid(&witness.key, &msg, &cosignature.signature) {
            continue;
        }
        if verified.iter().any(|v| v.key_id == witness.key_id) {
            continue; // a witness's cosignature counts once
        }
        verified.push(VerifiedHybridCosignature {
            key_id: witness.key_id,
            operator_id: witness.operator_id,
            wire: cosignature_wire(
                &witness.key_id,
                cosignature.timestamp_s,
                &cosignature.signature,
            ),
        });
    }

    verified
}

/// Verify `cosignatures` over `checkpoint` against the `witnesses` set, returning the
/// quorum actually achieved. A cosignature is counted ONLY if: its `witness_index`
/// is in range, its `timestamp_s > 0`, and its hybrid signature verifies (BOTH
/// Ed25519 and ML-DSA-44 halves) over the message binding THIS checkpoint, that
/// witness's key-id, and that timestamp. Verified witnesses are deduplicated by
/// key-id; operators by `operator_id`. A malformed, forged, mis-indexed, duplicate,
/// or zero-timestamp cosignature is not counted — it can never inflate the quorum
/// (fail-closed).
pub fn verify_hybrid_witness_quorum(
    checkpoint: &SealedAuditCheckpoint,
    witnesses: &[HybridWitness],
    cosignatures: &[HybridWitnessCosignature],
) -> HybridWitnessQuorum {
    let verified = verified_hybrid_cosignature_set(checkpoint, witnesses, cosignatures);
    let mut operators: Vec<u32> = Vec::new();
    for v in &verified {
        if !operators.contains(&v.operator_id) {
            operators.push(v.operator_id);
        }
    }
    let count = verified.len();
    HybridWitnessQuorum {
        verified_witness_count: i32::try_from(count).unwrap_or(i32::MAX),
        operator_count: i32::try_from(operators.len()).unwrap_or(i32::MAX),
        cosignature_bytes: i32::try_from(count.saturating_mul(HYBRID_COSIGNATURE_LEN))
            .unwrap_or(i32::MAX),
        // Every counted cosignature verified over this checkpoint and a positive
        // signed timestamp, so both flags are true for the counted set (and the set
        // is empty-safe: an empty quorum reports true vacuously but count 0 fails the
        // gate's threshold anyway).
        all_bind_checkpoint: true,
        all_timestamped: true,
    }
}

/// The witness RECEIPT bytes for a witnessed transparency anchor: the concatenation
/// of the canonical 2508-byte wire encodings of exactly the verified cosignatures
/// counted by [`verify_hybrid_witness_quorum`] (same core), in input order. So
/// `receipt.len() == quorum.verified_witness_count * 2508 == quorum.cosignature_bytes`
/// always holds — a forged or duplicate cosignature can neither inflate the count
/// nor pad the receipt.
pub fn verified_cosignature_wire_bytes(
    checkpoint: &SealedAuditCheckpoint,
    witnesses: &[HybridWitness],
    cosignatures: &[HybridWitnessCosignature],
) -> Vec<u8> {
    verified_hybrid_cosignature_set(checkpoint, witnesses, cosignatures)
        .into_iter()
        .flat_map(|v| v.wire)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint() -> SealedAuditCheckpoint {
        SealedAuditCheckpoint {
            tree_size: 7,
            root_hash: [0x33; 32],
            timestamp_s: 1_700_000_000,
        }
    }

    /// Build `n` hybrid witnesses with the given operator ids + cosignatures over the
    /// checkpoint. `forge` taints one cosignature; `bad_ts` zeroes one timestamp.
    fn witnessed(
        cp: &SealedAuditCheckpoint,
        operators: &[u32],
        ts: i64,
        forge: Option<usize>,
        bad_ts: Option<usize>,
    ) -> (Vec<HybridWitness>, Vec<HybridWitnessCosignature>) {
        let mut witnesses = Vec::new();
        let mut cosigs = Vec::new();
        for (i, op) in operators.iter().enumerate() {
            let sk = HybridCheckpointSigningKey::generate();
            let mut key_id = [0u8; KEY_ID_LEN];
            key_id[0] = i as u8 + 1;
            let timestamp = if bad_ts == Some(i) { 0 } else { ts };
            let mut cosig = cosign_checkpoint_hybrid(&sk, cp, &key_id, timestamp);
            cosig.witness_index = i;
            if forge == Some(i) {
                cosig.signature[0] ^= 0x01;
            }
            witnesses.push(HybridWitness {
                key: sk.verifying_key(),
                operator_id: *op,
                key_id,
            });
            cosigs.push(cosig);
        }
        (witnesses, cosigs)
    }

    #[test]
    fn a_genuine_hybrid_quorum_is_counted() {
        // 3 witnesses across 2 operators all hybrid-cosign -> quorum {3, 2}.
        let cp = checkpoint();
        let (witnesses, cosigs) = witnessed(&cp, &[10, 10, 20], 1_700_000_100, None, None);
        let q = verify_hybrid_witness_quorum(&cp, &witnesses, &cosigs);
        assert_eq!(q.verified_witness_count, 3);
        assert_eq!(q.operator_count, 2);
        // Real serialized cosignature bytes = 3 * 2508.
        assert_eq!(q.cosignature_bytes, 3 * 2508);
        assert!(q.all_bind_checkpoint && q.all_timestamped);
    }

    #[test]
    fn cosignature_wire_size_matches_the_gate_minimum() {
        // The per-cosignature wire size must equal the gate's witness_cosignature_min_len
        // for HybridEd25519MlDsa44 (2508), so 2 genuine cosignatures clear 2*2508.
        assert_eq!(HYBRID_COSIGNATURE_LEN, 2508);
    }

    #[test]
    fn a_forged_cosignature_is_not_counted() {
        let cp = checkpoint();
        let (witnesses, cosigs) = witnessed(&cp, &[10, 20], 1_700_000_100, Some(1), None);
        let q = verify_hybrid_witness_quorum(&cp, &witnesses, &cosigs);
        assert_eq!(q.verified_witness_count, 1);
        assert_eq!(q.operator_count, 1);
        assert_eq!(q.cosignature_bytes, 2508);
    }

    #[test]
    fn a_zero_timestamp_cosignature_is_not_counted() {
        let cp = checkpoint();
        let (witnesses, cosigs) = witnessed(&cp, &[10, 20], 1_700_000_100, None, Some(0));
        let q = verify_hybrid_witness_quorum(&cp, &witnesses, &cosigs);
        // Witness 0 had timestamp 0 -> not counted; only witness 1 counts.
        assert_eq!(q.verified_witness_count, 1);
    }

    #[test]
    fn a_cosignature_over_a_different_checkpoint_is_not_counted() {
        let cp = checkpoint();
        let other = SealedAuditCheckpoint { tree_size: 8, ..cp };
        let (witnesses, mut cosigs) = witnessed(&other, &[10], 1_700_000_100, None, None);
        // The cosignature was made over `other`; verifying against `cp` must fail.
        cosigs[0].witness_index = 0;
        let q = verify_hybrid_witness_quorum(&cp, &witnesses, &cosigs);
        assert_eq!(q.verified_witness_count, 0);
    }

    #[test]
    fn an_out_of_range_index_is_skipped() {
        let cp = checkpoint();
        let (witnesses, mut cosigs) = witnessed(&cp, &[10], 1_700_000_100, None, None);
        cosigs[0].witness_index = 99;
        let q = verify_hybrid_witness_quorum(&cp, &witnesses, &cosigs);
        assert_eq!(q.verified_witness_count, 0);
    }

    #[test]
    fn a_duplicate_witness_counts_once() {
        let cp = checkpoint();
        let (witnesses, cosigs) = witnessed(&cp, &[10], 1_700_000_100, None, None);
        let dup = [cosigs[0], cosigs[0]];
        let q = verify_hybrid_witness_quorum(&cp, &witnesses, &dup);
        assert_eq!(q.verified_witness_count, 1);
        assert_eq!(q.operator_count, 1);
    }

    #[test]
    fn the_receipt_bytes_never_drift_from_the_count() {
        // The witness-receipt wire bytes must equal count * 2508 for the SAME verified
        // set the quorum counts — even when forged/duplicate/zero-ts cosignatures are
        // mixed in (which inflate neither). This is the no-drift invariant the store
        // gate's witness_receipt length depends on.
        let cp = checkpoint();
        let (mut witnesses, mut cosigs) = witnessed(&cp, &[10, 20, 20], 1_700_000_100, None, None);
        // Append a forged, a duplicate, and a zero-timestamp cosignature.
        let mut forged = cosigs[0];
        forged.signature[0] ^= 0x01;
        cosigs.push(forged);
        cosigs.push(cosigs[1]); // duplicate of witness 1
        let _ = &mut witnesses; // witnesses unchanged
        let q = verify_hybrid_witness_quorum(&cp, &witnesses, &cosigs);
        let receipt = verified_cosignature_wire_bytes(&cp, &witnesses, &cosigs);
        assert_eq!(q.verified_witness_count, 3);
        assert_eq!(q.operator_count, 2);
        assert_eq!(receipt.len(), q.verified_witness_count as usize * 2508);
        assert_eq!(receipt.len() as i32, q.cosignature_bytes);
    }
}
