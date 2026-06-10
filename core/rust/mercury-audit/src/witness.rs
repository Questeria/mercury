//! Witness cosignatures over an audit checkpoint — the split-view defense for a
//! `WitnessedTransparencyLog`. Independent operators each Ed25519-cosign the log's
//! [`SealedAuditCheckpoint`] signing bytes; a quorum of DISTINCT witnesses across
//! DISTINCT operators is what satisfies the sealed-audit gate's witness
//! requirement (so a single operator cannot spin up many witnesses to fake a
//! quorum, and a malicious log cannot show different roots to different parties
//! without a witness noticing).
//!
//! Same Ed25519 cosignature pattern as mercury-kt's key-transparency witness
//! machinery, but the witnesses sign the AUDIT checkpoint preimage (a distinct
//! domain), so an audit cosignature can never be replayed as a KT tree head.

use ed25519_dalek::{Signature, VerifyingKey};

use crate::checkpoint::SealedAuditCheckpoint;

/// A transparency witness: an Ed25519 verifying key plus the operator that runs
/// it. The gate counts distinct OPERATORS (not just keys) so one operator running
/// several witnesses cannot alone satisfy the quorum.
#[derive(Debug, Clone, Copy)]
pub struct AuditWitness {
    pub key: VerifyingKey,
    pub operator_id: u32,
}

/// One witness's cosignature over a checkpoint: an index into the witness set and
/// the 64-byte Ed25519 signature over the checkpoint's canonical signing bytes.
#[derive(Debug, Clone, Copy)]
pub struct WitnessCosignature {
    pub witness_index: usize,
    pub signature: [u8; 64],
}

/// The verified witness quorum over a checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WitnessQuorum {
    /// Distinct witnesses (deduplicated by key) whose cosignature verified.
    pub verified_witness_count: i32,
    /// Distinct operators among those verified witnesses.
    pub operator_count: i32,
}

/// Cosign `checkpoint` as a witness (test/helper): produce the 64-byte Ed25519
/// signature over its canonical signing bytes.
pub fn cosign_checkpoint(
    signing_key: &ed25519_dalek::SigningKey,
    checkpoint: &SealedAuditCheckpoint,
) -> [u8; 64] {
    use ed25519_dalek::Signer as _;
    signing_key.sign(&checkpoint.signing_bytes()).to_bytes()
}

/// One genuinely-verified, deduplicated witness cosignature over a checkpoint: the
/// witness key, its operator, and the 64-byte signature that passed strict
/// verification over the checkpoint preimage.
#[derive(Debug, Clone, Copy)]
struct VerifiedCosignature {
    key_bytes: [u8; 32],
    operator_id: u32,
    signature: [u8; 64],
}

/// The shared verification core: the deduplicated set of cosignatures that REALLY
/// verify (strict) over `checkpoint`, in input order. Both the quorum counts and
/// the transparency receipt derive from THIS one result, so they can never drift —
/// the receipt always contains exactly the cosignatures the quorum counts. A
/// malformed, forged, mis-indexed, or duplicate cosignature is skipped (it can
/// never inflate the set): fail-closed.
fn verified_cosignature_set(
    checkpoint: &SealedAuditCheckpoint,
    witnesses: &[AuditWitness],
    cosignatures: &[WitnessCosignature],
) -> Vec<VerifiedCosignature> {
    let preimage = checkpoint.signing_bytes();
    let mut verified: Vec<VerifiedCosignature> = Vec::new();

    for cosignature in cosignatures {
        let Some(witness) = witnesses.get(cosignature.witness_index) else {
            continue;
        };
        let signature = Signature::from_bytes(&cosignature.signature);
        if witness.key.verify_strict(&preimage, &signature).is_err() {
            continue;
        }
        let key_bytes = witness.key.to_bytes();
        if verified.iter().any(|v| v.key_bytes == key_bytes) {
            continue; // a witness's cosignature counts once
        }
        verified.push(VerifiedCosignature {
            key_bytes,
            operator_id: witness.operator_id,
            signature: cosignature.signature,
        });
    }

    verified
}

/// Verify `cosignatures` over `checkpoint` against the `witnesses` set, returning
/// the quorum actually achieved. Each cosignature is counted ONLY if its
/// `witness_index` is in range AND its Ed25519 signature verifies (strict) over
/// the checkpoint signing bytes; verified witnesses are deduplicated by key, and
/// operators by `operator_id`. A malformed, forged, mis-indexed, or duplicate
/// cosignature is simply not counted — it can never INFLATE the quorum
/// (fail-closed): the returned counts are a floor on genuine independent witnesses.
pub fn verify_witnessed_checkpoint(
    checkpoint: &SealedAuditCheckpoint,
    witnesses: &[AuditWitness],
    cosignatures: &[WitnessCosignature],
) -> WitnessQuorum {
    let verified = verified_cosignature_set(checkpoint, witnesses, cosignatures);
    let mut operators: Vec<u32> = Vec::new();
    for v in &verified {
        if !operators.contains(&v.operator_id) {
            operators.push(v.operator_id);
        }
    }
    WitnessQuorum {
        verified_witness_count: i32::try_from(verified.len()).unwrap_or(i32::MAX),
        operator_count: i32::try_from(operators.len()).unwrap_or(i32::MAX),
    }
}

/// The deduplicated, verify_strict-passing cosignatures over `checkpoint`, in input
/// order — exactly the set counted by [`verify_witnessed_checkpoint`] (same core).
/// A witnessed transparency receipt is the concatenation of these 64-byte
/// signatures, so its length reflects the REAL number of independent witnesses and
/// can never be a fabricated blob: `witness_count == verified_cosignatures().len()`
/// always holds.
pub fn verified_cosignatures(
    checkpoint: &SealedAuditCheckpoint,
    witnesses: &[AuditWitness],
    cosignatures: &[WitnessCosignature],
) -> Vec<[u8; 64]> {
    verified_cosignature_set(checkpoint, witnesses, cosignatures)
        .into_iter()
        .map(|v| v.signature)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn checkpoint() -> SealedAuditCheckpoint {
        SealedAuditCheckpoint {
            tree_size: 7,
            root_hash: [0x33; 32],
            timestamp_s: 1_700_000_000,
        }
    }

    /// `n` witness keys; operator ids assigned by `operators[i]`.
    fn witnesses(seeds: &[u8], operators: &[u32]) -> (Vec<SigningKey>, Vec<AuditWitness>) {
        let keys: Vec<SigningKey> = seeds
            .iter()
            .map(|s| SigningKey::from_bytes(&[*s; 32]))
            .collect();
        let set: Vec<AuditWitness> = keys
            .iter()
            .zip(operators)
            .map(|(k, op)| AuditWitness {
                key: k.verifying_key(),
                operator_id: *op,
            })
            .collect();
        (keys, set)
    }

    #[test]
    fn a_genuine_quorum_is_counted() {
        // 3 witnesses across 2 operators all cosign -> quorum {3, 2}.
        let cp = checkpoint();
        let (keys, set) = witnesses(&[1, 2, 3], &[10, 10, 20]);
        let cosigs: Vec<WitnessCosignature> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| WitnessCosignature {
                witness_index: i,
                signature: cosign_checkpoint(k, &cp),
            })
            .collect();
        let quorum = verify_witnessed_checkpoint(&cp, &set, &cosigs);
        assert_eq!(quorum.verified_witness_count, 3);
        assert_eq!(quorum.operator_count, 2);
    }

    #[test]
    fn a_forged_cosignature_is_not_counted() {
        let cp = checkpoint();
        let (keys, set) = witnesses(&[1, 2], &[10, 20]);
        let mut cosigs: Vec<WitnessCosignature> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| WitnessCosignature {
                witness_index: i,
                signature: cosign_checkpoint(k, &cp),
            })
            .collect();
        // Tamper the second cosignature.
        cosigs[1].signature[0] ^= 0x01;
        let quorum = verify_witnessed_checkpoint(&cp, &set, &cosigs);
        assert_eq!(quorum.verified_witness_count, 1);
        assert_eq!(quorum.operator_count, 1);
    }

    #[test]
    fn a_cosignature_over_a_different_checkpoint_is_not_counted() {
        let cp = checkpoint();
        let other = SealedAuditCheckpoint { tree_size: 8, ..cp };
        let (keys, set) = witnesses(&[1], &[10]);
        // Cosign the OTHER checkpoint, present it for `cp`.
        let cosigs = [WitnessCosignature {
            witness_index: 0,
            signature: cosign_checkpoint(&keys[0], &other),
        }];
        assert_eq!(
            verify_witnessed_checkpoint(&cp, &set, &cosigs).verified_witness_count,
            0
        );
    }

    #[test]
    fn a_repeated_witness_counts_once() {
        let cp = checkpoint();
        let (keys, set) = witnesses(&[1], &[10]);
        let sig = cosign_checkpoint(&keys[0], &cp);
        let cosigs = [
            WitnessCosignature {
                witness_index: 0,
                signature: sig,
            },
            WitnessCosignature {
                witness_index: 0,
                signature: sig,
            },
        ];
        let quorum = verify_witnessed_checkpoint(&cp, &set, &cosigs);
        assert_eq!(quorum.verified_witness_count, 1);
        assert_eq!(quorum.operator_count, 1);
    }

    #[test]
    fn an_out_of_range_index_is_skipped() {
        let cp = checkpoint();
        let (keys, set) = witnesses(&[1], &[10]);
        let cosigs = [
            WitnessCosignature {
                witness_index: 0,
                signature: cosign_checkpoint(&keys[0], &cp),
            },
            WitnessCosignature {
                witness_index: 99, // out of range
                signature: [0u8; 64],
            },
        ];
        assert_eq!(
            verify_witnessed_checkpoint(&cp, &set, &cosigs).verified_witness_count,
            1
        );
    }
}
