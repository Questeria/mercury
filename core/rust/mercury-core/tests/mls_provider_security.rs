use mercury_core::{
    GroupChatCryptoSuite, MlsProviderSecurityInput, MlsProviderSecurityReason,
    evaluate_mls_provider_security,
};

#[test]
fn mls_provider_security_accepts_bound_hybrid_ml_kem_suite() {
    let decision = evaluate_mls_provider_security(valid_input());

    assert!(decision.accepted);
    assert_eq!(decision.reason, MlsProviderSecurityReason::Accepted);
    assert_eq!(decision.suite, GroupChatCryptoSuite::HybridPqMls768);
    assert!(decision.can_use_provider);
    assert!(decision.can_open_mls_group);
    assert!(!decision.requires_mls_setup);
    assert!(!decision.requires_pq_upgrade);
    assert!(!decision.requires_user_action);
}

#[test]
fn mls_provider_security_rejects_missing_unsupported_or_downgraded_suites() {
    let mut missing = valid_input();
    missing.provider_configured = false;
    let missing_decision = missing.evaluate();
    assert_rejected(missing_decision, MlsProviderSecurityReason::ProviderMissing);
    assert!(missing_decision.requires_mls_setup);
    assert!(!missing_decision.requires_user_action);

    let mut unsupported = valid_input();
    unsupported.provider_supports_selected_suite = false;
    let unsupported_decision = unsupported.evaluate();
    assert_rejected(
        unsupported_decision,
        MlsProviderSecurityReason::SuiteUnsupported,
    );
    assert!(unsupported_decision.requires_pq_upgrade);

    let mut downgraded = valid_input();
    downgraded.selected_suite = GroupChatCryptoSuite::ClassicalMls128;
    let downgraded_decision = downgraded.evaluate();
    assert_rejected(
        downgraded_decision,
        MlsProviderSecurityReason::DowngradeBelowFloor,
    );
    assert!(downgraded_decision.requires_pq_upgrade);
}

#[test]
fn mls_provider_security_requires_ml_kem_hybrid_and_pq_signature_when_requested() {
    let mut weak_ml_kem = valid_input();
    weak_ml_kem.ml_kem_parameter_set = 512;
    assert_rejected(
        weak_ml_kem.evaluate(),
        MlsProviderSecurityReason::MlKemParameterSetRequired,
    );

    let mut pure_pq_when_hybrid_required = valid_input();
    pure_pq_when_hybrid_required.classical_kem_component_present = false;
    assert_rejected(
        pure_pq_when_hybrid_required.evaluate(),
        MlsProviderSecurityReason::PqTraditionalHybridRequired,
    );

    let mut pq_signature = valid_input();
    pq_signature.selected_suite = GroupChatCryptoSuite::HybridPqMls1024;
    pq_signature.minimum_suite = GroupChatCryptoSuite::HybridPqMls1024;
    pq_signature.ml_kem_parameter_set = 1024;
    pq_signature.requires_pq_signatures = true;
    pq_signature.pq_signature_ready = false;
    assert_rejected(
        pq_signature.evaluate(),
        MlsProviderSecurityReason::PqSignatureRequired,
    );

    pq_signature.pq_signature_ready = true;
    assert!(pq_signature.evaluate().accepted);
}

#[test]
fn mls_provider_security_rejects_missing_binding_evidence_kats_and_hardening() {
    let mut binding = valid_input();
    binding.suite_id_bound_to_group_context = false;
    assert_rejected(
        binding.evaluate(),
        MlsProviderSecurityReason::SuiteContextBindingMissing,
    );

    let mut downgrade = valid_input();
    downgrade.downgrade_evidence_verified = false;
    assert_rejected(
        downgrade.evaluate(),
        MlsProviderSecurityReason::DowngradeEvidenceMissing,
    );

    let mut kats = valid_input();
    kats.known_answer_tests_passed = false;
    assert_rejected(
        kats.evaluate(),
        MlsProviderSecurityReason::KnownAnswerTestsMissing,
    );

    let mut zeroization = valid_input();
    zeroization.secret_zeroization_available = false;
    assert_rejected(
        zeroization.evaluate(),
        MlsProviderSecurityReason::SecretZeroizationMissing,
    );

    let mut unsafe_backend = valid_input();
    unsafe_backend.unsafe_crypto_backend = true;
    assert_rejected(
        unsafe_backend.evaluate(),
        MlsProviderSecurityReason::UnsafeCryptoBackend,
    );

    let mut plaintext_export = valid_input();
    plaintext_export.plaintext_key_export_fields = 1;
    assert_rejected(
        plaintext_export.evaluate(),
        MlsProviderSecurityReason::PlaintextKeyExportForbidden,
    );
}

#[test]
fn mls_provider_security_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsProviderSecurityReason::Accepted, 0, "ACCEPTED"),
        (
            MlsProviderSecurityReason::ProviderMissing,
            1,
            "PROVIDER_MISSING",
        ),
        (
            MlsProviderSecurityReason::SuiteUnsupported,
            2,
            "SUITE_UNSUPPORTED",
        ),
        (
            MlsProviderSecurityReason::DowngradeBelowFloor,
            3,
            "DOWNGRADE_BELOW_FLOOR",
        ),
        (
            MlsProviderSecurityReason::MlKemParameterSetRequired,
            4,
            "ML_KEM_PARAMETER_SET_REQUIRED",
        ),
        (
            MlsProviderSecurityReason::PqTraditionalHybridRequired,
            5,
            "PQ_TRADITIONAL_HYBRID_REQUIRED",
        ),
        (
            MlsProviderSecurityReason::PqSignatureRequired,
            6,
            "PQ_SIGNATURE_REQUIRED",
        ),
        (
            MlsProviderSecurityReason::SuiteContextBindingMissing,
            7,
            "SUITE_CONTEXT_BINDING_MISSING",
        ),
        (
            MlsProviderSecurityReason::DowngradeEvidenceMissing,
            8,
            "DOWNGRADE_EVIDENCE_MISSING",
        ),
        (
            MlsProviderSecurityReason::KnownAnswerTestsMissing,
            9,
            "KNOWN_ANSWER_TESTS_MISSING",
        ),
        (
            MlsProviderSecurityReason::SecretZeroizationMissing,
            10,
            "SECRET_ZEROIZATION_MISSING",
        ),
        (
            MlsProviderSecurityReason::UnsafeCryptoBackend,
            11,
            "UNSAFE_CRYPTO_BACKEND",
        ),
        (
            MlsProviderSecurityReason::PlaintextKeyExportForbidden,
            12,
            "PLAINTEXT_KEY_EXPORT_FORBIDDEN",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> MlsProviderSecurityInput {
    MlsProviderSecurityInput {
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
    }
}

fn assert_rejected(
    decision: mercury_core::MlsProviderSecurityDecision,
    reason: MlsProviderSecurityReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_use_provider);
    assert!(!decision.can_open_mls_group);
    assert!(decision.requires_mls_setup);
    assert_eq!(decision.reason, reason);
}
