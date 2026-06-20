//! Stage 6 witness / signed-tree-head dimension.
//!
//! Authenticates that the directory HEAD the inclusion / consistency /
//! key-history proofs were verified against is the genuine, witnessed head --
//! not one a malicious relay forked just for this client (a "split view"). This
//! is the dimension the prior audits flagged as the remaining gap: until the
//! head is authenticated, internally-consistent proofs about a forked head would
//! verify.
//!
//! Construction (standard CT / Sigsum-style cosigning): the log signs a
//! [`SignedTreeHead`] over canonical, domain-separated bytes; a quorum of
//! independent WITNESSES co-sign the SAME bytes; a designated AUDITOR (which has
//! checked append-only consistency) also signs. [`verify_witnessed_tree_head`]
//! VERIFIES those Ed25519 signatures and returns a [`WitnessVerification`] of
//! pure, relay-independent facts (signature validity, distinct-witness and
//! independent-operator counts, freshness derived from the SIGNED timestamp).
//! Two thin layers sit on top, WITHOUT re-doing any crypto:
//!   * [`WitnessVerification::kt_witness_status`] -> the coarse
//!     [`KeyTransparencyWitnessStatus`] the KT bundle feeds to
//!     `evaluate_key_transparency`;
//!   * [`WitnessVerification::issuer_audit_input`] -> the
//!     [`mercury_core::AnonymousIssuerWitnessAuditInput`] that
//!     `evaluate_anonymous_issuer_witness_audit` decides (it layers on the
//!     separately-decided KT decision + the client's policy).
//!
//! Keeping verification free of any KT decision or policy is deliberate: the
//! coarse status feeds the KT gate, whose decision in turn feeds the issuer
//! audit -- so verification must not depend on either.

use ed25519_dalek::{Signature, VerifyingKey};

use mercury_core::{
    AnonymousIssuerWitnessAuditInput, KeyTransparencyDecision, KeyTransparencyWitnessStatus,
};

/// Domain separator: every STH signature is over `DOMAIN ‖ fields`, so a
/// signature can never be replayed as some other Mercury (or non-Mercury)
/// message. Exactly 18 bytes (17 chars + NUL).
const STH_DOMAIN: &[u8; 18] = b"mercury/kt/sth/v1\0";

/// Length of the canonical signing preimage: domain ‖ tree_size(8) ‖ root(32) ‖
/// timestamp(8).
const STH_SIGNING_LEN: usize = 18 + 8 + 32 + 8;

/// A log's signed commitment to its directory state at one epoch: the AKD
/// `(tree_size == epoch, root_hash)` plus the wall-clock second the log signed
/// it (which drives freshness).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignedTreeHead {
    /// The directory size / epoch this head commits to (monotonic).
    pub tree_size: u64,
    /// The AKD root digest at that epoch (the same root the inclusion /
    /// key-history proofs are anchored to).
    pub root_hash: [u8; 32],
    /// Unix seconds at which the log signed this head.
    pub timestamp_s: i64,
}

impl SignedTreeHead {
    /// Canonical, domain-separated bytes the log, every witness, and the auditor
    /// all sign over. Fixed-width little-endian fields -> unambiguous, with no
    /// field-boundary confusion or length-extension surface. Public because it is
    /// the agreed wire preimage: independent log/witness/auditor signers must
    /// produce signatures over EXACTLY these bytes for the verifier to accept.
    pub fn signing_bytes(&self) -> [u8; STH_SIGNING_LEN] {
        let mut buf = [0u8; STH_SIGNING_LEN];
        buf[..18].copy_from_slice(STH_DOMAIN);
        buf[18..26].copy_from_slice(&self.tree_size.to_le_bytes());
        buf[26..58].copy_from_slice(&self.root_hash);
        buf[58..66].copy_from_slice(&self.timestamp_s.to_le_bytes());
        buf
    }
}

/// A witness operator's pinned co-signing key, tagged with which INDEPENDENT
/// operator runs it (so the gate can require operator diversity, not just key
/// count -- N keys run by one operator is not a quorum).
#[derive(Clone, Copy, Debug)]
pub struct Witness {
    /// The witness's pinned Ed25519 verifying key.
    pub key: VerifyingKey,
    /// Identifier of the independent operator that runs this witness.
    pub operator_id: u32,
}

impl Witness {
    /// Build a witness from its operator id + raw 32-byte Ed25519 public key. Returns `None` if the
    /// bytes are not a valid Ed25519 verifying key — so a relay's witness allow-list cannot be
    /// configured with a junk key that would silently never verify.
    pub fn from_ed25519_bytes(operator_id: u32, key_bytes: &[u8; 32]) -> Option<Self> {
        VerifyingKey::from_bytes(key_bytes)
            .ok()
            .map(|key| Self { key, operator_id })
    }

    /// The pinned verifying key's raw 32 bytes (for serving / display).
    pub fn key_bytes(&self) -> [u8; 32] {
        self.key.to_bytes()
    }
}

/// One witness co-signature over a signed tree head: which pinned witness signed
/// (index into the client's pinned witness set) and the raw Ed25519 signature.
#[derive(Clone, Copy, Debug)]
pub struct WitnessCosignature {
    /// Index into the caller's pinned `witnesses` slice.
    pub witness_index: usize,
    /// The Ed25519 signature bytes over the STH's canonical signing bytes.
    pub signature: [u8; 64],
}

/// The DERIVED, relay-independent result of verifying a witnessed signed tree
/// head. Carries no KT decision and no policy -- those are layered on by
/// [`WitnessVerification::kt_witness_status`] and
/// [`WitnessVerification::issuer_audit_input`]. The length fields are the gate's
/// success flags: 32 / 64 ONLY when the corresponding signature verified against
/// a head bound to the KT proofs' head.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessVerification {
    signed_tree_head_len: i32,
    inclusion_root_len: i32,
    verified_witness_count: i32,
    independent_operator_count: i32,
    auditor_signature_len: i32,
    audit_age_s: i64,
    current_tree_size: i64,
}

fn verify_sig(key: &VerifyingKey, message: &[u8], signature: &[u8; 64]) -> bool {
    // `verify_strict` rejects small-order / non-canonical keys and signatures --
    // the secure choice for cosigning.
    key.verify_strict(message, &Signature::from_bytes(signature))
        .is_ok()
}

/// Verify ONE witness co-signature over a signed tree head — the relay's per-submission check when
/// collecting cosignatures to serve. Uses the same strict Ed25519 verification (`verify_strict`,
/// rejecting small-order / non-canonical inputs) as the full bundle verifier, over the STH's
/// canonical signing bytes. Returns true iff `signature` is `witness`'s genuine signature of `sth`.
pub fn verify_witness_cosignature(
    witness: &Witness,
    sth: &SignedTreeHead,
    signature: &[u8; 64],
) -> bool {
    verify_sig(&witness.key, &sth.signing_bytes(), signature)
}

/// Verify a witnessed signed tree head into pure, relay-independent facts.
/// Everything returned is DERIVED from real signature verification:
///   * the STH must commit to EXACTLY `expected_root` + `expected_tree_size`
///     (the head the KT proofs used) -- otherwise it is not evidence about those
///     proofs and the length flags are 0 so the gate rejects (this is the
///     binding that authenticates the proofs' head);
///   * `signed_tree_head_len` / `inclusion_root_len` are 32 only if the LOG's
///     signature over the canonical bytes verifies;
///   * the verified-witness count is the number of DISTINCT pinned witnesses
///     whose co-signature verifies (a repeated witness counts once), and the
///     independent-operator count is the distinct operators among them;
///   * `auditor_signature_len` is 64 only if the auditor's signature verifies;
///   * `audit_age_s` is derived from the SIGNED timestamp (`now_s - timestamp`);
///     a future-dated head yields a NEGATIVE age, which the gate rejects as stale
///     (so a relay cannot post-date a head to fake freshness). The true value is
///     reported on purpose -- the gate, not the engine, owns the policy.
///
/// The log, witness, and auditor keys MUST be distinct pinned keys: all three
/// roles sign the same canonical STH bytes, so role separation comes entirely
/// from verifying each signature against its own pinned key.
#[allow(clippy::too_many_arguments)]
pub fn verify_witnessed_tree_head(
    sth: &SignedTreeHead,
    log_key: &VerifyingKey,
    log_signature: &[u8; 64],
    auditor_key: &VerifyingKey,
    auditor_signature: &[u8; 64],
    witnesses: &[Witness],
    cosignatures: &[WitnessCosignature],
    expected_root: [u8; 32],
    expected_tree_size: u64,
    now_s: i64,
) -> WitnessVerification {
    let message = sth.signing_bytes();

    // The witnessed head MUST be the same head the KT proofs bound to. If it is
    // not, the STH is not evidence about those proofs at all -> treat as bad.
    let head_bound = sth.root_hash == expected_root && sth.tree_size == expected_tree_size;
    let log_ok = head_bound && verify_sig(log_key, &message, log_signature);

    let signed_tree_head_len = if log_ok {
        sth.root_hash.len() as i32
    } else {
        0
    };
    let inclusion_root_len = if log_ok {
        expected_root.len() as i32
    } else {
        0
    };

    // Count DISTINCT validly-co-signing witnesses + their distinct operators.
    let mut verified_keys: Vec<[u8; 32]> = Vec::new();
    let mut operators: Vec<u32> = Vec::new();
    for cosig in cosignatures {
        let Some(witness) = witnesses.get(cosig.witness_index) else {
            continue; // references an unknown witness -> ignore
        };
        let key_bytes = witness.key.to_bytes();
        if verified_keys.contains(&key_bytes) {
            continue; // one witness counts at most once, regardless of repeats
        }
        if verify_sig(&witness.key, &message, &cosig.signature) {
            verified_keys.push(key_bytes);
            if !operators.contains(&witness.operator_id) {
                operators.push(witness.operator_id);
            }
        }
    }

    let auditor_ok = verify_sig(auditor_key, &message, auditor_signature);

    WitnessVerification {
        signed_tree_head_len,
        inclusion_root_len,
        verified_witness_count: i32::try_from(verified_keys.len()).unwrap_or(i32::MAX),
        independent_operator_count: i32::try_from(operators.len()).unwrap_or(i32::MAX),
        auditor_signature_len: if auditor_ok { 64 } else { 0 },
        audit_age_s: now_s.saturating_sub(sth.timestamp_s),
        current_tree_size: i64::try_from(expected_tree_size).unwrap_or(i64::MAX),
    }
}

impl WitnessVerification {
    /// The COARSE [`KeyTransparencyWitnessStatus`] the KT bundle feeds to
    /// `evaluate_key_transparency`. A structurally broken signed tree head or
    /// auditor signature is `Invalid` (tampering); a sound signed head whose
    /// quorum meets `required_witness_count` (which must be > 0) across >= 2
    /// independent operators is `QuorumSatisfied`; anything sound-but-short is
    /// `QuorumMissing`.
    pub fn kt_witness_status(&self, required_witness_count: i32) -> KeyTransparencyWitnessStatus {
        if self.signed_tree_head_len != 32
            || self.inclusion_root_len != 32
            || self.auditor_signature_len != 64
        {
            return KeyTransparencyWitnessStatus::Invalid;
        }
        if required_witness_count > 0
            && self.verified_witness_count >= required_witness_count
            && self.independent_operator_count >= 2
        {
            KeyTransparencyWitnessStatus::QuorumSatisfied
        } else {
            KeyTransparencyWitnessStatus::QuorumMissing
        }
    }

    /// Assemble the issuer-witness-audit gate input by layering the
    /// (separately-decided) `key_transparency` decision and the caller's
    /// policy/observation inputs onto this verification. The signature / count /
    /// freshness fields come straight from verification; `previous_tree_size`
    /// (the client's last witnessed size, for the rollback check),
    /// `required_witness_count`, `max_audit_age_s`, and `split_view_reports` are
    /// legitimately the client's, not the relay's.
    pub fn issuer_audit_input(
        &self,
        key_transparency: KeyTransparencyDecision,
        previous_tree_size: i64,
        required_witness_count: i32,
        max_audit_age_s: i64,
        split_view_reports: i32,
    ) -> AnonymousIssuerWitnessAuditInput {
        AnonymousIssuerWitnessAuditInput {
            key_transparency,
            signed_tree_head_len: self.signed_tree_head_len,
            inclusion_root_len: self.inclusion_root_len,
            previous_tree_size,
            current_tree_size: self.current_tree_size,
            required_witness_count,
            verified_witness_count: self.verified_witness_count,
            independent_operator_count: self.independent_operator_count,
            audit_age_s: self.audit_age_s,
            max_audit_age_s,
            split_view_reports,
            auditor_signature_len: self.auditor_signature_len,
            // The witnessing wire format carries no plaintext partitioning metadata.
            plaintext_partitioning_fields: 0,
        }
    }
}

/// Client-side: verify the LOG's signature over a served [`SignedTreeHead`]
/// against the pinned log public key. Returns true iff the Ed25519 signature
/// over the canonical STH bytes verifies -- the check a client runs on a head
/// fetched from the untrusted relay before trusting its `(epoch, root)`. (The
/// full protocol then also gathers witness co-signatures over the same head;
/// see [`verify_witnessed_tree_head`].)
pub fn verify_signed_tree_head(
    log_public_key: &[u8; 32],
    sth: &SignedTreeHead,
    signature: &[u8; 64],
) -> bool {
    let Ok(key) = VerifyingKey::from_bytes(log_public_key) else {
        return false;
    };
    verify_sig(&key, &sth.signing_bytes(), signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use mercury_core::{
        AnonymousIssuerWitnessAuditReason, KeyTransparencyReason, KeyTransparencyState,
        evaluate_anonymous_issuer_witness_audit,
    };

    const ROOT: [u8; 32] = [7u8; 32];
    const SIZE: u64 = 9;
    /// Previous witnessed tree size (one epoch back) as the gate's i64.
    const PREV: i64 = SIZE as i64 - 1;

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn sth() -> SignedTreeHead {
        SignedTreeHead {
            tree_size: SIZE,
            root_hash: ROOT,
            timestamp_s: 1_000,
        }
    }

    fn sign(k: &SigningKey, sth: &SignedTreeHead) -> [u8; 64] {
        k.sign(&sth.signing_bytes()).to_bytes()
    }

    /// A KT decision the issuer-witness gate accepts as its precondition.
    fn consistent_kt() -> KeyTransparencyDecision {
        KeyTransparencyDecision {
            state: KeyTransparencyState::Consistent,
            reason: KeyTransparencyReason::Consistent,
            requires_user_action: false,
        }
    }

    /// Verify a fully-valid witnessed head: log + auditor + 3 witnesses across 3
    /// operators, evaluated 30s after the signed timestamp.
    fn valid_verification() -> WitnessVerification {
        let sth = sth();
        let log = key(1);
        let auditor = key(2);
        let w1 = key(10);
        let w2 = key(11);
        let w3 = key(12);
        let witnesses = vec![
            Witness {
                key: w1.verifying_key(),
                operator_id: 100,
            },
            Witness {
                key: w2.verifying_key(),
                operator_id: 200,
            },
            Witness {
                key: w3.verifying_key(),
                operator_id: 300,
            },
        ];
        let cosigs = vec![
            WitnessCosignature {
                witness_index: 0,
                signature: sign(&w1, &sth),
            },
            WitnessCosignature {
                witness_index: 1,
                signature: sign(&w2, &sth),
            },
            WitnessCosignature {
                witness_index: 2,
                signature: sign(&w3, &sth),
            },
        ];
        verify_witnessed_tree_head(
            &sth,
            &log.verifying_key(),
            &sign(&log, &sth),
            &auditor.verifying_key(),
            &sign(&auditor, &sth),
            &witnesses,
            &cosigs,
            ROOT,
            SIZE,
            1_030,
        )
    }

    #[test]
    fn valid_witnessed_head_satisfies_quorum_and_gate_accepts() {
        let v = valid_verification();
        assert_eq!(v.verified_witness_count, 3);
        assert_eq!(v.independent_operator_count, 3);
        assert_eq!(v.signed_tree_head_len, 32);
        assert_eq!(v.auditor_signature_len, 64);
        assert_eq!(v.audit_age_s, 30); // derived from the signed timestamp

        assert_eq!(
            v.kt_witness_status(3),
            KeyTransparencyWitnessStatus::QuorumSatisfied
        );

        let decision = evaluate_anonymous_issuer_witness_audit(v.issuer_audit_input(
            consistent_kt(),
            PREV,
            3,
            300,
            0,
        ));
        assert!(decision.accepted);
        assert!(decision.has_witness_quorum);
        assert!(decision.detects_split_view);
        assert_eq!(decision.reason, AnonymousIssuerWitnessAuditReason::Accepted);
    }

    #[test]
    fn forged_log_signature_is_a_bad_signed_tree_head() {
        let sth = sth();
        let wrong_log = key(99); // not the pinned log key
        let auditor = key(2);
        let v = verify_witnessed_tree_head(
            &sth,
            &key(1).verifying_key(), // pinned log key...
            &sign(&wrong_log, &sth), // ...but signed by the wrong key
            &auditor.verifying_key(),
            &sign(&auditor, &sth),
            &[],
            &[],
            ROOT,
            SIZE,
            1_030,
        );
        assert_eq!(v.signed_tree_head_len, 0);
        assert_eq!(
            v.kt_witness_status(1),
            KeyTransparencyWitnessStatus::Invalid
        );
        let decision = evaluate_anonymous_issuer_witness_audit(v.issuer_audit_input(
            consistent_kt(),
            PREV,
            1,
            300,
            0,
        ));
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            AnonymousIssuerWitnessAuditReason::BadSignedTreeHead
        );
    }

    #[test]
    fn forged_auditor_signature_is_rejected() {
        // Valid log + a full witness quorum, but the auditor signature is by the
        // wrong key: auditor_signature_len must be 0 and the gate must refuse.
        let sth = sth();
        let log = key(1);
        let wrong_auditor = key(98); // not the pinned auditor key
        let w1 = key(10);
        let w2 = key(11);
        let witnesses = vec![
            Witness {
                key: w1.verifying_key(),
                operator_id: 100,
            },
            Witness {
                key: w2.verifying_key(),
                operator_id: 200,
            },
        ];
        let cosigs = vec![
            WitnessCosignature {
                witness_index: 0,
                signature: sign(&w1, &sth),
            },
            WitnessCosignature {
                witness_index: 1,
                signature: sign(&w2, &sth),
            },
        ];
        let v = verify_witnessed_tree_head(
            &sth,
            &log.verifying_key(),
            &sign(&log, &sth),
            &key(2).verifying_key(),     // pinned auditor key...
            &sign(&wrong_auditor, &sth), // ...but signed by the wrong key
            &witnesses,
            &cosigs,
            ROOT,
            SIZE,
            1_030,
        );
        assert_eq!(v.auditor_signature_len, 0);
        assert_eq!(
            v.kt_witness_status(2),
            KeyTransparencyWitnessStatus::Invalid
        );
        let decision = evaluate_anonymous_issuer_witness_audit(v.issuer_audit_input(
            consistent_kt(),
            PREV,
            2,
            300,
            0,
        ));
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            AnonymousIssuerWitnessAuditReason::AuditorSignatureMissing
        );
    }

    #[test]
    fn sth_for_a_different_head_is_rejected() {
        // A validly-signed STH, but for a root that is NOT the head the KT proofs
        // used -> not evidence about those proofs -> bad STH.
        let sth = sth();
        let log = key(1);
        let auditor = key(2);
        let v = verify_witnessed_tree_head(
            &sth,
            &log.verifying_key(),
            &sign(&log, &sth),
            &auditor.verifying_key(),
            &sign(&auditor, &sth),
            &[],
            &[],
            [8u8; 32], // expected_root != sth.root_hash
            SIZE,
            1_030,
        );
        assert_eq!(v.signed_tree_head_len, 0);
        let decision = evaluate_anonymous_issuer_witness_audit(v.issuer_audit_input(
            consistent_kt(),
            PREV,
            1,
            300,
            0,
        ));
        assert_eq!(
            decision.reason,
            AnonymousIssuerWitnessAuditReason::BadSignedTreeHead
        );
    }

    #[test]
    fn invalid_and_duplicate_cosignatures_do_not_count() {
        let sth = sth();
        let log = key(1);
        let auditor = key(2);
        let w1 = key(10);
        let w2 = key(11);
        let witnesses = vec![
            Witness {
                key: w1.verifying_key(),
                operator_id: 100,
            },
            Witness {
                key: w2.verifying_key(),
                operator_id: 200,
            },
        ];
        let cosigs = vec![
            WitnessCosignature {
                witness_index: 0,
                signature: sign(&w1, &sth),
            },
            WitnessCosignature {
                witness_index: 0,
                signature: sign(&w1, &sth),
            }, // dup
            WitnessCosignature {
                witness_index: 1,
                signature: [0u8; 64],
            }, // bad sig
            WitnessCosignature {
                witness_index: 9,
                signature: sign(&w2, &sth),
            }, // unknown idx
        ];
        let v = verify_witnessed_tree_head(
            &sth,
            &log.verifying_key(),
            &sign(&log, &sth),
            &auditor.verifying_key(),
            &sign(&auditor, &sth),
            &witnesses,
            &cosigs,
            ROOT,
            SIZE,
            1_030,
        );
        assert_eq!(v.verified_witness_count, 1, "only w1 counts once");
        assert_eq!(v.independent_operator_count, 1);
        // Quorum of 2 not met -> the gate reports WitnessQuorumMissing.
        let decision = evaluate_anonymous_issuer_witness_audit(v.issuer_audit_input(
            consistent_kt(),
            PREV,
            2,
            300,
            0,
        ));
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            AnonymousIssuerWitnessAuditReason::WitnessQuorumMissing
        );
    }

    #[test]
    fn single_operator_quorum_fails_diversity() {
        // Two valid witnesses, but the SAME operator -> not independent.
        let sth = sth();
        let log = key(1);
        let auditor = key(2);
        let w1 = key(10);
        let w2 = key(11);
        let witnesses = vec![
            Witness {
                key: w1.verifying_key(),
                operator_id: 100,
            },
            Witness {
                key: w2.verifying_key(),
                operator_id: 100,
            }, // same operator
        ];
        let cosigs = vec![
            WitnessCosignature {
                witness_index: 0,
                signature: sign(&w1, &sth),
            },
            WitnessCosignature {
                witness_index: 1,
                signature: sign(&w2, &sth),
            },
        ];
        let v = verify_witnessed_tree_head(
            &sth,
            &log.verifying_key(),
            &sign(&log, &sth),
            &auditor.verifying_key(),
            &sign(&auditor, &sth),
            &witnesses,
            &cosigs,
            ROOT,
            SIZE,
            1_030,
        );
        assert_eq!(v.verified_witness_count, 2);
        assert_eq!(v.independent_operator_count, 1);
        let decision = evaluate_anonymous_issuer_witness_audit(v.issuer_audit_input(
            consistent_kt(),
            PREV,
            2,
            300,
            0,
        ));
        assert_eq!(
            decision.reason,
            AnonymousIssuerWitnessAuditReason::OperatorDiversityMissing
        );
    }

    #[test]
    fn stale_signed_tree_head_is_rejected() {
        // The valid bundle, but evaluated far in the future: freshness (derived
        // from the signed timestamp) now exceeds the window.
        let v = {
            let sth = sth();
            let log = key(1);
            let auditor = key(2);
            let w1 = key(10);
            let w2 = key(11);
            let witnesses = vec![
                Witness {
                    key: w1.verifying_key(),
                    operator_id: 100,
                },
                Witness {
                    key: w2.verifying_key(),
                    operator_id: 200,
                },
            ];
            let cosigs = vec![
                WitnessCosignature {
                    witness_index: 0,
                    signature: sign(&w1, &sth),
                },
                WitnessCosignature {
                    witness_index: 1,
                    signature: sign(&w2, &sth),
                },
            ];
            verify_witnessed_tree_head(
                &sth,
                &log.verifying_key(),
                &sign(&log, &sth),
                &auditor.verifying_key(),
                &sign(&auditor, &sth),
                &witnesses,
                &cosigs,
                ROOT,
                SIZE,
                10_000, // now: ~9000s after the signed timestamp
            )
        };
        assert_eq!(v.audit_age_s, 9_000);
        let decision = evaluate_anonymous_issuer_witness_audit(v.issuer_audit_input(
            consistent_kt(),
            PREV,
            2,
            300,
            0,
        ));
        assert_eq!(
            decision.reason,
            AnonymousIssuerWitnessAuditReason::AuditStale
        );
    }

    #[test]
    fn issuer_audit_requires_consistent_key_transparency() {
        // If the KT decision layered on is not Consistent, the issuer gate refuses
        // up front -- regardless of a perfectly good witnessed head.
        let inconsistent = KeyTransparencyDecision {
            state: KeyTransparencyState::Inconsistent,
            reason: KeyTransparencyReason::InclusionProofInvalid,
            requires_user_action: true,
        };
        let v = valid_verification();
        let decision = evaluate_anonymous_issuer_witness_audit(v.issuer_audit_input(
            inconsistent,
            PREV,
            3,
            300,
            0,
        ));
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            AnonymousIssuerWitnessAuditReason::KeyTransparencyRejected
        );
    }
}
