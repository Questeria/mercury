use mercury_core::{
    PrototypeSealedAuditProofCache, SealedAuditProofBundleDecision, SealedAuditProofBundleReason,
    SealedAuditProofCacheDecision, SealedAuditProofCacheReason, SealedAuditProofCacheWrite,
    put_sealed_audit_proof_cache_record,
};

const PROOF_BUNDLE_DIGEST: [u8; 32] = [0xB1; 32];
const OTHER_PROOF_BUNDLE_DIGEST: [u8; 32] = [0xB2; 32];
const EVENT_HASH: [u8; 32] = [0xB3; 32];
const OTHER_EVENT_HASH: [u8; 32] = [0xB4; 32];
const CHECKPOINT_DIGEST: [u8; 32] = [0xB5; 32];
const POLICY_SNAPSHOT_DIGEST: [u8; 32] = [0xB6; 32];
const SHORT_DIGEST: [u8; 16] = [0xB7; 16];

#[test]
fn proof_cache_persists_only_accepted_digest_records() {
    let mut cache = PrototypeSealedAuditProofCache::default();
    let decision = put_sealed_audit_proof_cache_record(&mut cache, valid_write())
        .expect("prototype proof cache is infallible");

    assert!(decision.accepted);
    assert_eq!(decision.reason, SealedAuditProofCacheReason::Accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert_eq!(decision.event_sequence, 42);
    assert_eq!(decision.log_index, 42);
    assert_eq!(decision.checkpoint_size, 43);
    assert_eq!(decision.verifier_policy_epoch, 7);
    assert!(decision.can_verify_offline);
    assert!(decision.can_show_ui_status);
    assert!(decision.can_refresh_monitor);
    assert!(!decision.requires_policy_refresh);
    assert!(!decision.requires_witness_refresh);
    assert!(!decision.requires_cache_recovery);
    assert!(decision.append_only);
    assert!(decision.keeps_digest_only);
    assert!(!decision.keeps_plaintext_metadata);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(cache.len(), 1);

    let record = cache
        .get_by_digest(&PROOF_BUNDLE_DIGEST)
        .expect("accepted proof bundle should persist");
    assert_eq!(record.proof_bundle_digest, PROOF_BUNDLE_DIGEST);
    assert_eq!(record.event_hash, EVENT_HASH);
    assert_eq!(record.checkpoint_digest, CHECKPOINT_DIGEST);
    assert_eq!(
        record.verifier_policy_snapshot_digest,
        POLICY_SNAPSHOT_DIGEST
    );
    assert_eq!(record.event_sequence, 42);
    assert_eq!(record.log_index, 42);
    assert_eq!(record.checkpoint_size, 43);
    assert_eq!(record.verifier_policy_epoch, 7);
    assert_eq!(record.verified_at_s, 1_769_990_430);
    assert_eq!(record.witness_timestamp_s, 1_769_990_400);
    assert!(!record.recovered_from_cache_loss);
    assert!(!record.plaintext_bytes_exposed);
    assert!(cache.get_by_event_hash(&EVENT_HASH).is_some());
    assert_eq!(cache.highest_log_index(), Some(42));
}

#[test]
fn proof_cache_rejects_bundle_gate_failures_and_bad_shapes_without_mutation() {
    let mut cache = PrototypeSealedAuditProofCache::default();

    let rejected_bundle = SealedAuditProofCacheWrite {
        proof_bundle_decision: rejected_proof_bundle_decision(false),
        ..valid_write()
    };
    assert_rejected(
        cache.put(rejected_bundle),
        SealedAuditProofCacheReason::ProofBundleRejected,
    );
    assert!(cache.is_empty());

    let plaintext_bundle = SealedAuditProofCacheWrite {
        proof_bundle_decision: rejected_proof_bundle_decision(true),
        ..valid_write()
    };
    let decision = cache.put(plaintext_bundle);
    assert_rejected_with(
        decision,
        SealedAuditProofCacheReason::ProofBundleRejected,
        false,
        false,
        false,
        true,
    );

    let bad_format = SealedAuditProofCacheWrite {
        cache_format_version: 2,
        ..valid_write()
    };
    assert_rejected(
        cache.put(bad_format),
        SealedAuditProofCacheReason::BadRecordShape,
    );

    let bad_digest = SealedAuditProofCacheWrite {
        proof_bundle_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        cache.put(bad_digest),
        SealedAuditProofCacheReason::BadRecordShape,
    );

    assert!(cache.is_empty());
}

#[test]
fn proof_cache_rejects_duplicates_and_rollback_indexes_without_mutation() {
    let mut cache = PrototypeSealedAuditProofCache::default();
    assert!(cache.put(valid_write()).accepted);

    let duplicate_digest = SealedAuditProofCacheWrite {
        event_hash: &OTHER_EVENT_HASH,
        ..valid_write()
    };
    assert_rejected_with_count(
        cache.put(duplicate_digest),
        SealedAuditProofCacheReason::DuplicateProof,
        1,
    );
    assert_eq!(cache.len(), 1);

    let duplicate_event = SealedAuditProofCacheWrite {
        proof_bundle_digest: &OTHER_PROOF_BUNDLE_DIGEST,
        ..valid_write()
    };
    assert_rejected_with_count(
        cache.put(duplicate_event),
        SealedAuditProofCacheReason::DuplicateProof,
        1,
    );
    assert_eq!(cache.len(), 1);

    let rollback = SealedAuditProofCacheWrite {
        proof_bundle_decision: accepted_proof_bundle_decision_for(41, 41, 43),
        proof_bundle_digest: &OTHER_PROOF_BUNDLE_DIGEST,
        event_hash: &OTHER_EVENT_HASH,
        event_sequence: 41,
        log_index: 41,
        ..valid_write()
    };
    assert_rejected_with_count(
        cache.put(rollback),
        SealedAuditProofCacheReason::RollbackIndex,
        1,
    );
    assert_eq!(cache.len(), 1);
}

#[test]
fn proof_cache_rejects_stale_policy_offline_failures_and_plaintext_metadata() {
    let stale_policy = SealedAuditProofCacheWrite {
        verifier_policy_epoch: 6,
        ..valid_write()
    };
    assert_rejected_with(
        evaluate(stale_policy),
        SealedAuditProofCacheReason::PolicySnapshotStale,
        true,
        false,
        false,
        false,
    );

    let failed_offline_verifier = SealedAuditProofCacheWrite {
        offline_verification_passed: false,
        ..valid_write()
    };
    assert_rejected_with(
        evaluate(failed_offline_verifier),
        SealedAuditProofCacheReason::OfflineVerifierRejected,
        false,
        true,
        false,
        false,
    );

    let no_monitor_freshness = SealedAuditProofCacheWrite {
        monitor_freshness_checked: false,
        ..valid_write()
    };
    assert_rejected_with(
        evaluate(no_monitor_freshness),
        SealedAuditProofCacheReason::OfflineVerifierRejected,
        false,
        true,
        false,
        false,
    );

    let plaintext = SealedAuditProofCacheWrite {
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    assert_rejected_with(
        evaluate(plaintext),
        SealedAuditProofCacheReason::PlaintextMetadataForbidden,
        false,
        false,
        false,
        true,
    );
}

#[test]
fn proof_cache_requires_authenticated_recovery_and_append_only_encrypted_storage() {
    let unauthenticated_recovery = SealedAuditProofCacheWrite {
        proof_bundle_decision: accepted_recovery_proof_bundle_decision(),
        recovery_bundle_authenticated: true,
        recovery_requires_user_verification: false,
        ..valid_write()
    };
    assert_rejected_with(
        evaluate(unauthenticated_recovery),
        SealedAuditProofCacheReason::CacheRecoveryRequired,
        false,
        false,
        true,
        false,
    );

    let unencrypted_cache = SealedAuditProofCacheWrite {
        cache_record_encrypted: false,
        ..valid_write()
    };
    assert_rejected(
        evaluate(unencrypted_cache),
        SealedAuditProofCacheReason::AppendOnlyGuardMissing,
    );

    let mut cache = PrototypeSealedAuditProofCache::default();
    let authenticated_recovery = SealedAuditProofCacheWrite {
        proof_bundle_decision: accepted_recovery_proof_bundle_decision(),
        recovery_bundle_authenticated: true,
        recovery_requires_user_verification: true,
        ..valid_write()
    };
    let decision = cache.put(authenticated_recovery);
    assert!(decision.accepted);
    assert!(decision.requires_cache_recovery);
    assert!(
        cache
            .get_by_digest(&PROOF_BUNDLE_DIGEST)
            .expect("accepted recovery proof should persist")
            .recovered_from_cache_loss
    );
}

#[test]
fn proof_cache_allows_next_log_index_with_fresh_digest_and_event_hash() {
    let mut cache = PrototypeSealedAuditProofCache::default();
    assert!(cache.put(valid_write()).accepted);

    let next = SealedAuditProofCacheWrite {
        proof_bundle_decision: accepted_proof_bundle_decision_for(43, 43, 44),
        proof_bundle_digest: &OTHER_PROOF_BUNDLE_DIGEST,
        event_hash: &OTHER_EVENT_HASH,
        event_sequence: 43,
        log_index: 43,
        checkpoint_size: 44,
        ..valid_write()
    };
    let decision = cache.put(next);

    assert!(decision.accepted);
    assert_eq!(decision.record_count, 2);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.highest_log_index(), Some(43));
}

#[test]
fn proof_cache_reasons_have_stable_codes_and_labels() {
    let cases = [
        (SealedAuditProofCacheReason::Accepted, 0, "ACCEPTED"),
        (
            SealedAuditProofCacheReason::ProofBundleRejected,
            1,
            "PROOF_BUNDLE_REJECTED",
        ),
        (
            SealedAuditProofCacheReason::DuplicateProof,
            2,
            "DUPLICATE_PROOF",
        ),
        (
            SealedAuditProofCacheReason::RollbackIndex,
            3,
            "ROLLBACK_INDEX",
        ),
        (
            SealedAuditProofCacheReason::PolicySnapshotStale,
            4,
            "POLICY_SNAPSHOT_STALE",
        ),
        (
            SealedAuditProofCacheReason::PlaintextMetadataForbidden,
            5,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            SealedAuditProofCacheReason::CacheRecoveryRequired,
            6,
            "CACHE_RECOVERY_REQUIRED",
        ),
        (
            SealedAuditProofCacheReason::BadRecordShape,
            7,
            "BAD_RECORD_SHAPE",
        ),
        (
            SealedAuditProofCacheReason::OfflineVerifierRejected,
            8,
            "OFFLINE_VERIFIER_REJECTED",
        ),
        (
            SealedAuditProofCacheReason::AppendOnlyGuardMissing,
            9,
            "APPEND_ONLY_GUARD_MISSING",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn evaluate(write: SealedAuditProofCacheWrite<'_>) -> SealedAuditProofCacheDecision {
    mercury_core::evaluate_sealed_audit_proof_cache_write(write)
}

fn assert_rejected(decision: SealedAuditProofCacheDecision, reason: SealedAuditProofCacheReason) {
    assert_rejected_with(decision, reason, false, false, false, false);
}

fn assert_rejected_with_count(
    decision: SealedAuditProofCacheDecision,
    reason: SealedAuditProofCacheReason,
    record_count: usize,
) {
    assert_rejected(decision, reason);
    assert_eq!(decision.record_count, record_count);
}

fn assert_rejected_with(
    decision: SealedAuditProofCacheDecision,
    reason: SealedAuditProofCacheReason,
    requires_policy_refresh: bool,
    requires_witness_refresh: bool,
    requires_cache_recovery: bool,
    plaintext_bytes_exposed: bool,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.reason, reason);
    assert!(!decision.persisted_record);
    assert!(!decision.can_verify_offline);
    assert!(!decision.can_show_ui_status);
    assert!(!decision.can_refresh_monitor);
    assert_eq!(decision.requires_policy_refresh, requires_policy_refresh);
    assert_eq!(decision.requires_witness_refresh, requires_witness_refresh);
    assert_eq!(decision.requires_cache_recovery, requires_cache_recovery);
    assert!(decision.append_only);
    assert!(decision.keeps_digest_only);
    assert!(!decision.keeps_plaintext_metadata);
    assert_eq!(decision.plaintext_bytes_exposed, plaintext_bytes_exposed);
}

fn valid_write() -> SealedAuditProofCacheWrite<'static> {
    SealedAuditProofCacheWrite {
        proof_bundle_decision: accepted_proof_bundle_decision(),
        cache_format_version: 1,
        proof_bundle_digest: &PROOF_BUNDLE_DIGEST,
        event_hash: &EVENT_HASH,
        checkpoint_digest: &CHECKPOINT_DIGEST,
        verifier_policy_snapshot_digest: &POLICY_SNAPSHOT_DIGEST,
        event_sequence: 42,
        log_index: 42,
        checkpoint_size: 43,
        verifier_policy_epoch: 7,
        verified_at_s: 1_769_990_430,
        witness_timestamp_s: 1_769_990_400,
        offline_verification_passed: true,
        monitor_freshness_checked: true,
        cache_record_encrypted: true,
        append_only_guard: true,
        plaintext_metadata_fields: 0,
        recovery_bundle_authenticated: false,
        recovery_requires_user_verification: false,
    }
}

const fn accepted_proof_bundle_decision() -> SealedAuditProofBundleDecision {
    accepted_proof_bundle_decision_for(42, 42, 43)
}

const fn accepted_proof_bundle_decision_for(
    event_sequence: i64,
    log_index: i64,
    checkpoint_size: i64,
) -> SealedAuditProofBundleDecision {
    SealedAuditProofBundleDecision {
        accepted: true,
        reason: SealedAuditProofBundleReason::Accepted,
        event_sequence,
        log_index,
        checkpoint_size,
        verifier_policy_epoch: 7,
        can_verify_offline: true,
        can_persist_proof_bundle: true,
        can_show_ui_status: true,
        can_recover_proof_cache: false,
        requires_policy_refresh: false,
        requires_witness_refresh: false,
        requires_proof_cache_recovery: false,
        requires_redaction: false,
        plaintext_bytes_exposed: false,
    }
}

const fn accepted_recovery_proof_bundle_decision() -> SealedAuditProofBundleDecision {
    SealedAuditProofBundleDecision {
        can_recover_proof_cache: true,
        requires_proof_cache_recovery: true,
        ..accepted_proof_bundle_decision()
    }
}

const fn rejected_proof_bundle_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditProofBundleDecision {
    SealedAuditProofBundleDecision {
        accepted: false,
        reason: SealedAuditProofBundleReason::WitnessClientRejected,
        event_sequence: 42,
        log_index: 42,
        checkpoint_size: 43,
        verifier_policy_epoch: 7,
        can_verify_offline: false,
        can_persist_proof_bundle: false,
        can_show_ui_status: false,
        can_recover_proof_cache: false,
        requires_policy_refresh: false,
        requires_witness_refresh: false,
        requires_proof_cache_recovery: false,
        requires_redaction: false,
        plaintext_bytes_exposed,
    }
}
