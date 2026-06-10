use mercury_core::{
    GroupChatCryptoSuite, MlsProviderEvidenceStoreReason, MlsProviderEvidenceStoreRecord,
    MlsProviderEvidenceStoreWrite, MlsProviderEvidenceUseInput, MlsProviderEvidenceUseReason,
    MlsProviderSecurityDecision, MlsProviderSecurityInput, MlsProviderSecurityReason,
    PrototypeMlsProviderEvidenceStore, evaluate_mls_provider_evidence_store_write,
    evaluate_mls_provider_evidence_use, evaluate_mls_provider_security,
    put_mls_provider_evidence_record,
};

const EVIDENCE_ID: [u8; 32] = [0x31; 32];
const PROVIDER_ID_DIGEST: [u8; 32] = [0x41; 32];
const SUITE_EVIDENCE_DIGEST: [u8; 32] = [0x42; 32];
const KAT_EVIDENCE_DIGEST: [u8; 32] = [0x43; 32];
const DOWNGRADE_EVIDENCE_DIGEST: [u8; 32] = [0x44; 32];
const ZEROIZATION_EVIDENCE_DIGEST: [u8; 32] = [0x45; 32];
const SHORT_DIGEST: [u8; 16] = [0x46; 16];

#[test]
fn provider_evidence_store_persists_only_accepted_digest_records() {
    let mut store = PrototypeMlsProviderEvidenceStore::default();
    let decision = put_mls_provider_evidence_record(&mut store, valid_write())
        .expect("prototype store is infallible");

    assert_eq!(decision.reason, MlsProviderEvidenceStoreReason::Accepted);
    assert!(decision.accepted);
    assert!(decision.persisted_record);
    assert!(decision.can_use_as_provider_evidence);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.record_count, 1);
    assert_eq!(store.len(), 1);

    let record = store.get(&EVIDENCE_ID).expect("record should be written");
    assert_eq!(record.evidence_id, EVIDENCE_ID);
    assert_eq!(record.provider_id_digest, PROVIDER_ID_DIGEST);
    assert_eq!(record.suite, GroupChatCryptoSuite::HybridPqMls768);
    assert_eq!(record.suite_evidence_digest, SUITE_EVIDENCE_DIGEST);
    assert_eq!(record.kat_evidence_digest, KAT_EVIDENCE_DIGEST);
    assert_eq!(record.downgrade_evidence_digest, DOWNGRADE_EVIDENCE_DIGEST);
    assert_eq!(
        record.zeroization_evidence_digest,
        ZEROIZATION_EVIDENCE_DIGEST
    );
    assert_eq!(record.validated_at_s, 1_000);
    assert_eq!(record.expires_at_s, 1_300);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn provider_evidence_store_rejects_security_gate_and_bad_shapes() {
    let mut write = valid_write();
    write.provider_security = rejected_provider_security();
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::ProviderSecurityRejected,
    );

    let mut write = valid_write();
    write.evidence_id = &SHORT_DIGEST;
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::BadEvidenceId,
    );

    let mut write = valid_write();
    write.provider_id_digest = &SHORT_DIGEST;
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::BadProviderIdDigest,
    );

    let mut write = valid_write();
    write.suite_evidence_digest = &SHORT_DIGEST;
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::BadSuiteEvidenceDigest,
    );

    let mut write = valid_write();
    write.kat_evidence_digest = &SHORT_DIGEST;
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::BadKatEvidenceDigest,
    );

    let mut write = valid_write();
    write.downgrade_evidence_digest = &SHORT_DIGEST;
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::BadDowngradeEvidenceDigest,
    );

    let mut write = valid_write();
    write.zeroization_evidence_digest = &SHORT_DIGEST;
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::BadZeroizationEvidenceDigest,
    );
}

#[test]
fn provider_evidence_store_rejects_bad_window_plaintext_and_duplicates() {
    let mut write = valid_write();
    write.expires_at_s = write.validated_at_s;
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::BadValidationWindow,
    );

    let mut write = valid_write();
    write.plaintext_evidence_fields = 1;
    assert_rejected(
        evaluate_mls_provider_evidence_store_write(write),
        MlsProviderEvidenceStoreReason::PlaintextEvidenceForbidden,
    );

    let mut store = PrototypeMlsProviderEvidenceStore::default();
    let first = put_mls_provider_evidence_record(&mut store, valid_write())
        .expect("prototype store is infallible");
    assert!(first.accepted);

    let duplicate = put_mls_provider_evidence_record(&mut store, valid_write())
        .expect("prototype store is infallible");
    assert_eq!(
        duplicate.reason,
        MlsProviderEvidenceStoreReason::EvidenceAlreadyRecorded
    );
    assert!(!duplicate.accepted);
    assert!(!duplicate.persisted_record);
    assert_eq!(duplicate.record_count, 1);
    assert_eq!(store.len(), 1);
}

#[test]
fn provider_evidence_use_accepts_fresh_matching_digest_record() {
    let record = accepted_record();
    let decision = evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
        record: Some(&record),
        provider_security: accepted_provider_security(),
        required_suite: GroupChatCryptoSuite::HybridPqMls768,
        now_s: 1_100,
    });

    assert_eq!(decision.reason, MlsProviderEvidenceUseReason::Accepted);
    assert!(decision.accepted);
    assert!(decision.can_use_provider_evidence);
    assert!(!decision.requires_provider_validation);
    assert!(!decision.requires_pq_upgrade);
    assert!(!decision.requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn provider_evidence_use_rejects_missing_rejected_expired_and_mismatched_records() {
    assert_use_rejected(
        evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: None,
            provider_security: accepted_provider_security(),
            required_suite: GroupChatCryptoSuite::HybridPqMls768,
            now_s: 1_100,
        }),
        MlsProviderEvidenceUseReason::RecordMissing,
    );

    let record = accepted_record();
    assert_use_rejected(
        evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: Some(&record),
            provider_security: rejected_provider_security(),
            required_suite: GroupChatCryptoSuite::HybridPqMls768,
            now_s: 1_100,
        }),
        MlsProviderEvidenceUseReason::ProviderSecurityRejected,
    );

    assert_use_rejected(
        evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: Some(&record),
            provider_security: accepted_provider_security(),
            required_suite: GroupChatCryptoSuite::HybridPqMls768,
            now_s: 1_300,
        }),
        MlsProviderEvidenceUseReason::EvidenceExpired,
    );

    assert_use_rejected(
        evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: Some(&record),
            provider_security: accepted_provider_security(),
            required_suite: GroupChatCryptoSuite::HybridPqMls1024,
            now_s: 1_100,
        }),
        MlsProviderEvidenceUseReason::SuiteMismatch,
    );
}

#[test]
fn provider_evidence_use_rejects_not_yet_valid_bad_shape_and_plaintext_taint() {
    let record = accepted_record();
    assert_use_rejected(
        evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: Some(&record),
            provider_security: accepted_provider_security(),
            required_suite: GroupChatCryptoSuite::HybridPqMls768,
            now_s: 999,
        }),
        MlsProviderEvidenceUseReason::EvidenceNotYetValid,
    );

    let mut bad_shape = accepted_record();
    bad_shape.kat_evidence_digest.clear();
    assert_use_rejected(
        evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: Some(&bad_shape),
            provider_security: accepted_provider_security(),
            required_suite: GroupChatCryptoSuite::HybridPqMls768,
            now_s: 1_100,
        }),
        MlsProviderEvidenceUseReason::BadEvidenceShape,
    );

    let mut plaintext = accepted_record();
    plaintext.plaintext_bytes_exposed = true;
    let decision = evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
        record: Some(&plaintext),
        provider_security: accepted_provider_security(),
        required_suite: GroupChatCryptoSuite::HybridPqMls768,
        now_s: 1_100,
    });
    assert_eq!(
        decision.reason,
        MlsProviderEvidenceUseReason::PlaintextEvidenceDetected
    );
    assert!(!decision.accepted);
    assert!(decision.plaintext_bytes_exposed);
}

#[test]
fn provider_evidence_store_reasons_have_stable_codes_and_labels() {
    let cases = [
        (MlsProviderEvidenceStoreReason::Accepted, 0, "ACCEPTED"),
        (
            MlsProviderEvidenceStoreReason::ProviderSecurityRejected,
            1,
            "PROVIDER_SECURITY_REJECTED",
        ),
        (
            MlsProviderEvidenceStoreReason::BadEvidenceId,
            2,
            "BAD_EVIDENCE_ID",
        ),
        (
            MlsProviderEvidenceStoreReason::BadProviderIdDigest,
            3,
            "BAD_PROVIDER_ID_DIGEST",
        ),
        (
            MlsProviderEvidenceStoreReason::BadSuiteEvidenceDigest,
            4,
            "BAD_SUITE_EVIDENCE_DIGEST",
        ),
        (
            MlsProviderEvidenceStoreReason::BadKatEvidenceDigest,
            5,
            "BAD_KAT_EVIDENCE_DIGEST",
        ),
        (
            MlsProviderEvidenceStoreReason::BadDowngradeEvidenceDigest,
            6,
            "BAD_DOWNGRADE_EVIDENCE_DIGEST",
        ),
        (
            MlsProviderEvidenceStoreReason::BadZeroizationEvidenceDigest,
            7,
            "BAD_ZEROIZATION_EVIDENCE_DIGEST",
        ),
        (
            MlsProviderEvidenceStoreReason::BadValidationWindow,
            8,
            "BAD_VALIDATION_WINDOW",
        ),
        (
            MlsProviderEvidenceStoreReason::PlaintextEvidenceForbidden,
            9,
            "PLAINTEXT_EVIDENCE_FORBIDDEN",
        ),
        (
            MlsProviderEvidenceStoreReason::EvidenceAlreadyRecorded,
            10,
            "EVIDENCE_ALREADY_RECORDED",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

#[test]
fn provider_evidence_use_reasons_have_stable_codes_and_labels() {
    let cases = [
        (MlsProviderEvidenceUseReason::Accepted, 0, "ACCEPTED"),
        (
            MlsProviderEvidenceUseReason::RecordMissing,
            1,
            "RECORD_MISSING",
        ),
        (
            MlsProviderEvidenceUseReason::ProviderSecurityRejected,
            2,
            "PROVIDER_SECURITY_REJECTED",
        ),
        (
            MlsProviderEvidenceUseReason::SuiteMismatch,
            3,
            "SUITE_MISMATCH",
        ),
        (
            MlsProviderEvidenceUseReason::EvidenceNotYetValid,
            4,
            "EVIDENCE_NOT_YET_VALID",
        ),
        (
            MlsProviderEvidenceUseReason::EvidenceExpired,
            5,
            "EVIDENCE_EXPIRED",
        ),
        (
            MlsProviderEvidenceUseReason::BadEvidenceShape,
            6,
            "BAD_EVIDENCE_SHAPE",
        ),
        (
            MlsProviderEvidenceUseReason::PlaintextEvidenceDetected,
            7,
            "PLAINTEXT_EVIDENCE_DETECTED",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(
    decision: mercury_core::MlsProviderEvidenceStoreDecision,
    reason: MlsProviderEvidenceStoreReason,
) {
    assert_eq!(decision.reason, reason);
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert!(!decision.can_use_as_provider_evidence);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);
}

fn assert_use_rejected(
    decision: mercury_core::MlsProviderEvidenceUseDecision,
    reason: MlsProviderEvidenceUseReason,
) {
    assert_eq!(decision.reason, reason);
    assert!(!decision.accepted);
    assert!(!decision.can_use_provider_evidence);
}

fn accepted_record() -> MlsProviderEvidenceStoreRecord {
    let mut store = PrototypeMlsProviderEvidenceStore::default();
    let decision = put_mls_provider_evidence_record(&mut store, valid_write())
        .expect("prototype store is infallible");
    assert!(decision.accepted);
    store
        .get(&EVIDENCE_ID)
        .expect("record should be written")
        .clone()
}

fn valid_write() -> MlsProviderEvidenceStoreWrite<'static> {
    MlsProviderEvidenceStoreWrite {
        evidence_id: &EVIDENCE_ID,
        provider_id_digest: &PROVIDER_ID_DIGEST,
        suite_evidence_digest: &SUITE_EVIDENCE_DIGEST,
        kat_evidence_digest: &KAT_EVIDENCE_DIGEST,
        downgrade_evidence_digest: &DOWNGRADE_EVIDENCE_DIGEST,
        zeroization_evidence_digest: &ZEROIZATION_EVIDENCE_DIGEST,
        provider_security: accepted_provider_security(),
        validated_at_s: 1_000,
        expires_at_s: 1_300,
        plaintext_evidence_fields: 0,
    }
}

fn accepted_provider_security() -> MlsProviderSecurityDecision {
    evaluate_mls_provider_security(MlsProviderSecurityInput {
        provider_configured: true,
        selected_suite: GroupChatCryptoSuite::HybridPqMls768,
        minimum_suite: GroupChatCryptoSuite::HybridPqMls768,
        provider_supports_selected_suite: true,
        ml_kem_parameter_set: 768,
        classical_kem_component_present: true,
        requires_pq_signatures: false,
        pq_signature_ready: false,
        suite_id_bound_to_group_context: true,
        downgrade_evidence_verified: true,
        known_answer_tests_passed: true,
        secret_zeroization_available: true,
        unsafe_crypto_backend: false,
        plaintext_key_export_fields: 0,
    })
}

fn rejected_provider_security() -> MlsProviderSecurityDecision {
    let mut input = MlsProviderSecurityInput {
        provider_configured: true,
        selected_suite: GroupChatCryptoSuite::HybridPqMls768,
        minimum_suite: GroupChatCryptoSuite::HybridPqMls768,
        provider_supports_selected_suite: true,
        ml_kem_parameter_set: 768,
        classical_kem_component_present: true,
        requires_pq_signatures: false,
        pq_signature_ready: false,
        suite_id_bound_to_group_context: true,
        downgrade_evidence_verified: true,
        known_answer_tests_passed: true,
        secret_zeroization_available: true,
        unsafe_crypto_backend: false,
        plaintext_key_export_fields: 0,
    };
    input.known_answer_tests_passed = false;
    let decision = evaluate_mls_provider_security(input);
    assert_eq!(
        decision.reason,
        MlsProviderSecurityReason::KnownAnswerTestsMissing
    );
    decision
}
