use mercury_core::{
    AnonymousIssuerWitnessAuditInput, AnonymousIssuerWitnessAuditReason, KeyTransparencyDecision,
    KeyTransparencyProofInput, KeyTransparencyProofStatus, KeyTransparencyReason,
    KeyTransparencyState, KeyTransparencyWitnessStatus, evaluate_anonymous_issuer_witness_audit,
    evaluate_key_transparency,
};

#[test]
fn issuer_witness_audit_accepts_fresh_quorum_and_diverse_auditors() {
    let decision = evaluate_anonymous_issuer_witness_audit(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_use_issuer_key);
    assert!(decision.has_witness_quorum);
    assert!(decision.detects_split_view);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_rekey);
    assert!(!decision.requires_user_action);
    assert!(decision.protects_anonymity_set);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, AnonymousIssuerWitnessAuditReason::Accepted);
}

#[test]
fn issuer_witness_audit_requires_consistent_transparency_and_tree_shape() {
    let rejected_transparency = AnonymousIssuerWitnessAuditInput {
        key_transparency: KeyTransparencyDecision {
            state: KeyTransparencyState::Inconsistent,
            reason: KeyTransparencyReason::ConsistencyProofInvalid,
            requires_user_action: true,
        },
        ..valid_input()
    };
    let rejected = rejected_transparency.evaluate();
    assert_rejected(
        rejected,
        AnonymousIssuerWitnessAuditReason::KeyTransparencyRejected,
    );
    assert!(rejected.requires_rekey);
    assert!(rejected.requires_user_action);

    let bad_sth = AnonymousIssuerWitnessAuditInput {
        signed_tree_head_len: 16,
        ..valid_input()
    };
    assert_rejected(
        bad_sth.evaluate(),
        AnonymousIssuerWitnessAuditReason::BadSignedTreeHead,
    );

    let rollback = AnonymousIssuerWitnessAuditInput {
        current_tree_size: 11,
        previous_tree_size: 12,
        ..valid_input()
    };
    let rollback_decision = rollback.evaluate();
    assert_rejected(
        rollback_decision,
        AnonymousIssuerWitnessAuditReason::TreeSizeRollback,
    );
    assert!(rollback_decision.requires_rekey);
}

#[test]
fn issuer_witness_audit_requires_quorum_diversity_and_freshness() {
    let missing_quorum = AnonymousIssuerWitnessAuditInput {
        required_witness_count: 3,
        verified_witness_count: 2,
        ..valid_input()
    };
    let quorum_decision = missing_quorum.evaluate();
    assert_rejected(
        quorum_decision,
        AnonymousIssuerWitnessAuditReason::WitnessQuorumMissing,
    );
    assert!(quorum_decision.requires_sync);

    let no_diversity = AnonymousIssuerWitnessAuditInput {
        independent_operator_count: 1,
        ..valid_input()
    };
    let diversity_decision = no_diversity.evaluate();
    assert_rejected(
        diversity_decision,
        AnonymousIssuerWitnessAuditReason::OperatorDiversityMissing,
    );
    assert!(diversity_decision.requires_rekey);

    let stale = AnonymousIssuerWitnessAuditInput {
        audit_age_s: 301,
        max_audit_age_s: 300,
        ..valid_input()
    };
    assert_rejected(
        stale.evaluate(),
        AnonymousIssuerWitnessAuditReason::AuditStale,
    );
}

#[test]
fn issuer_witness_audit_rejects_split_views_bad_signatures_and_partitioning_metadata() {
    let split_view = AnonymousIssuerWitnessAuditInput {
        split_view_reports: 1,
        ..valid_input()
    };
    let split_decision = split_view.evaluate();
    assert_rejected(
        split_decision,
        AnonymousIssuerWitnessAuditReason::SplitViewReported,
    );
    assert!(split_decision.requires_rekey);

    let bad_signature = AnonymousIssuerWitnessAuditInput {
        auditor_signature_len: 0,
        ..valid_input()
    };
    assert_rejected(
        bad_signature.evaluate(),
        AnonymousIssuerWitnessAuditReason::AuditorSignatureMissing,
    );

    let plaintext = AnonymousIssuerWitnessAuditInput {
        plaintext_partitioning_fields: 1,
        ..valid_input()
    };
    let plaintext_decision = plaintext.evaluate();
    assert_rejected(
        plaintext_decision,
        AnonymousIssuerWitnessAuditReason::PlaintextPartitioningMetadata,
    );
    assert!(!plaintext_decision.plaintext_bytes_exposed);
}

#[test]
fn issuer_witness_audit_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (AnonymousIssuerWitnessAuditReason::Accepted, 0, "ACCEPTED"),
        (
            AnonymousIssuerWitnessAuditReason::KeyTransparencyRejected,
            1,
            "KEY_TRANSPARENCY_REJECTED",
        ),
        (
            AnonymousIssuerWitnessAuditReason::BadSignedTreeHead,
            2,
            "BAD_SIGNED_TREE_HEAD",
        ),
        (
            AnonymousIssuerWitnessAuditReason::TreeSizeRollback,
            3,
            "TREE_SIZE_ROLLBACK",
        ),
        (
            AnonymousIssuerWitnessAuditReason::WitnessQuorumMissing,
            4,
            "WITNESS_QUORUM_MISSING",
        ),
        (
            AnonymousIssuerWitnessAuditReason::OperatorDiversityMissing,
            5,
            "OPERATOR_DIVERSITY_MISSING",
        ),
        (
            AnonymousIssuerWitnessAuditReason::AuditStale,
            6,
            "AUDIT_STALE",
        ),
        (
            AnonymousIssuerWitnessAuditReason::SplitViewReported,
            7,
            "SPLIT_VIEW_REPORTED",
        ),
        (
            AnonymousIssuerWitnessAuditReason::AuditorSignatureMissing,
            8,
            "AUDITOR_SIGNATURE_MISSING",
        ),
        (
            AnonymousIssuerWitnessAuditReason::PlaintextPartitioningMetadata,
            9,
            "PLAINTEXT_PARTITIONING_METADATA",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> AnonymousIssuerWitnessAuditInput {
    AnonymousIssuerWitnessAuditInput {
        key_transparency: evaluate_key_transparency(valid_transparency_proof()),
        signed_tree_head_len: 32,
        inclusion_root_len: 32,
        previous_tree_size: 12,
        current_tree_size: 13,
        required_witness_count: 2,
        verified_witness_count: 3,
        independent_operator_count: 2,
        audit_age_s: 60,
        max_audit_age_s: 300,
        split_view_reports: 0,
        auditor_signature_len: 64,
        plaintext_partitioning_fields: 0,
    }
}

fn valid_transparency_proof() -> KeyTransparencyProofInput {
    KeyTransparencyProofInput {
        inclusion: KeyTransparencyProofStatus::Verified,
        consistency: KeyTransparencyProofStatus::Verified,
        key_history: KeyTransparencyProofStatus::Verified,
        witness: KeyTransparencyWitnessStatus::QuorumSatisfied,
        require_witness: true,
        previous_tree_size: 12,
        current_tree_size: 13,
        proof_age_s: 60,
        max_proof_age_s: 300,
    }
}

fn assert_rejected(
    decision: mercury_core::AnonymousIssuerWitnessAuditDecision,
    reason: AnonymousIssuerWitnessAuditReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_use_issuer_key);
    assert!(!decision.has_witness_quorum);
    assert!(!decision.detects_split_view);
    assert!(!decision.protects_anonymity_set);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
