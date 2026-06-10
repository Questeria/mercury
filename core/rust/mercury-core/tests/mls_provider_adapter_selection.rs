use mercury_core::{
    GroupChatCryptoSuite, MlsProviderAdapterKind, MlsProviderAdapterSelectionInput,
    MlsProviderAdapterSelectionReason, MlsProviderCryptoBackendKind,
    MlsProviderImplementationLicenseKind, MlsProviderProtocolProfile, MlsProviderSecurityInput,
    MlsProviderSecurityReason,
};

#[test]
fn adapter_selection_accepts_verified_openmls_libcrux_hybrid_provider() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_link_provider);
    assert!(decision.can_open_mls_group);
    assert!(decision.can_change_membership);
    assert!(decision.can_ship_release);
    assert!(!decision.requires_mls_setup);
    assert!(!decision.requires_pq_upgrade);
    assert!(!decision.requires_license_review);
    assert!(!decision.requires_supply_chain_review);
    assert!(!decision.requires_interop_review);
    assert!(!decision.requires_storage_review);
    assert!(decision.forbids_plaintext_key_export);
    assert_eq!(decision.reason, MlsProviderAdapterSelectionReason::Accepted);
    assert_eq!(decision.adapter_kind_label, "openmls");
    assert_eq!(decision.crypto_backend_label, "libcrux_hybrid_pq");
    assert_eq!(decision.protocol_profile_label, "draft_pq_hybrid");
    assert_eq!(decision.license_kind_label, "mit");
    assert_eq!(decision.suite_label, "hybrid_pq_mls_768");
    assert_eq!(
        decision.provider_security_reason,
        MlsProviderSecurityReason::Accepted
    );
}

#[test]
fn adapter_selection_rejects_provider_adapter_backend_and_profile_gaps() {
    let mut provider_rejected = valid_input();
    let mut provider_security = valid_provider_security_input(GroupChatCryptoSuite::HybridPqMls768);
    provider_security.known_answer_tests_passed = false;
    provider_rejected.provider_security = provider_security.evaluate();
    let provider_rejected_decision = provider_rejected.evaluate();
    assert_rejected(
        provider_rejected_decision,
        MlsProviderAdapterSelectionReason::ProviderSecurityRejected,
    );
    assert!(provider_rejected_decision.requires_mls_setup);
    assert_eq!(
        provider_rejected_decision.provider_security_reason,
        MlsProviderSecurityReason::KnownAnswerTestsMissing
    );

    let mut custom = valid_input();
    custom.adapter_kind = MlsProviderAdapterKind::CustomMls;
    let custom_decision = custom.evaluate();
    assert_rejected(
        custom_decision,
        MlsProviderAdapterSelectionReason::AdapterKindRejected,
    );
    assert!(custom_decision.requires_supply_chain_review);

    let mut weak_backend = valid_input();
    weak_backend.crypto_backend = MlsProviderCryptoBackendKind::RustCryptoProvider;
    let weak_backend_decision = weak_backend.evaluate();
    assert_rejected(
        weak_backend_decision,
        MlsProviderAdapterSelectionReason::CryptoBackendRejected,
    );
    assert!(weak_backend_decision.requires_pq_upgrade);

    let mut wrong_profile = valid_input();
    wrong_profile.protocol_profile = MlsProviderProtocolProfile::Rfc9420Classical;
    assert_rejected(
        wrong_profile.evaluate(),
        MlsProviderAdapterSelectionReason::ProtocolProfileRejected,
    );
}

#[test]
fn adapter_selection_rejects_license_source_and_conformance_gaps() {
    let mut license = valid_input();
    license.license_kind = MlsProviderImplementationLicenseKind::Unknown;
    let license_decision = license.evaluate();
    assert_rejected(
        license_decision,
        MlsProviderAdapterSelectionReason::LicenseRejected,
    );
    assert!(license_decision.requires_license_review);

    let mut source = valid_input();
    source.source_verified = false;
    let source_decision = source.evaluate();
    assert_rejected(
        source_decision,
        MlsProviderAdapterSelectionReason::SourceAuthenticityMissing,
    );
    assert!(source_decision.requires_supply_chain_review);

    let mut conformance = valid_input();
    conformance.rfc9420_conformance_tests_passed = false;
    let conformance_decision = conformance.evaluate();
    assert_rejected(
        conformance_decision,
        MlsProviderAdapterSelectionReason::Rfc9420ConformanceMissing,
    );
    assert!(conformance_decision.requires_interop_review);
}

#[test]
fn adapter_selection_requires_pinned_standardized_pq_suite_material() {
    let mut draft = valid_input();
    draft.pq_draft_version_pinned = false;
    let draft_decision = draft.evaluate();
    assert_rejected(
        draft_decision,
        MlsProviderAdapterSelectionReason::PqDraftPinMissing,
    );
    assert!(draft_decision.requires_pq_upgrade);

    let mut ml_kem = valid_input();
    ml_kem.ml_kem_standardized = false;
    assert_rejected(
        ml_kem.evaluate(),
        MlsProviderAdapterSelectionReason::MlKemStandardMissing,
    );

    let mut pq_sig = valid_input();
    pq_sig.provider_security =
        valid_provider_security_input(GroupChatCryptoSuite::HybridPqMls1024).evaluate();
    pq_sig.protocol_profile = MlsProviderProtocolProfile::DraftPqHybridWithPqSignatures;
    pq_sig.pq_signature_standardized_when_required = false;
    assert_rejected(
        pq_sig.evaluate(),
        MlsProviderAdapterSelectionReason::PqSignatureStandardMissing,
    );

    pq_sig.pq_signature_standardized_when_required = true;
    assert!(pq_sig.evaluate().accepted);
}

#[test]
fn adapter_selection_rejects_interop_storage_secret_lifecycle_and_binding_gaps() {
    let mut kat = valid_input();
    kat.kat_vectors_passed = false;
    let kat_decision = kat.evaluate();
    assert_rejected(
        kat_decision,
        MlsProviderAdapterSelectionReason::KatOrInteropMissing,
    );
    assert!(kat_decision.requires_interop_review);

    let mut storage = valid_input();
    storage.storage_provider_transactional = false;
    let storage_decision = storage.evaluate();
    assert_rejected(
        storage_decision,
        MlsProviderAdapterSelectionReason::StorageProviderUnsafe,
    );
    assert!(storage_decision.requires_storage_review);

    let mut lifecycle = valid_input();
    lifecycle.secret_zeroization_audited = false;
    assert_rejected(
        lifecycle.evaluate(),
        MlsProviderAdapterSelectionReason::SecretLifecycleUnsafe,
    );

    let mut downgrade = valid_input();
    downgrade.downgrade_tests_passed = false;
    assert_rejected(
        downgrade.evaluate(),
        MlsProviderAdapterSelectionReason::DowngradeTestMissing,
    );

    let mut transcript = valid_input();
    transcript.transcript_hash_binding_verified = false;
    assert_rejected(
        transcript.evaluate(),
        MlsProviderAdapterSelectionReason::TranscriptBindingMissing,
    );
}

#[test]
fn adapter_selection_rejects_debug_plaintext_release_and_supply_chain_gaps() {
    let mut unsafe_features = valid_input();
    unsafe_features.unsafe_features_enabled = true;
    assert_rejected(
        unsafe_features.evaluate(),
        MlsProviderAdapterSelectionReason::UnsafeFeaturesEnabled,
    );

    let mut plaintext = valid_input();
    plaintext.plaintext_export_enabled = true;
    assert_rejected(
        plaintext.evaluate(),
        MlsProviderAdapterSelectionReason::PlaintextExportEnabled,
    );

    let mut unsigned = valid_input();
    unsigned.release_artifact_signed = false;
    assert_rejected(
        unsigned.evaluate(),
        MlsProviderAdapterSelectionReason::ReleaseArtifactUnverified,
    );

    let mut sbom = valid_input();
    sbom.sbom_present = false;
    assert_rejected(
        sbom.evaluate(),
        MlsProviderAdapterSelectionReason::SbomOrCveMonitoringMissing,
    );
}

#[test]
fn adapter_selection_reason_and_profile_labels_are_stable() {
    let reasons = [
        (MlsProviderAdapterSelectionReason::Accepted, 0, "ACCEPTED"),
        (
            MlsProviderAdapterSelectionReason::ProviderSecurityRejected,
            1,
            "PROVIDER_SECURITY_REJECTED",
        ),
        (
            MlsProviderAdapterSelectionReason::AdapterKindRejected,
            2,
            "ADAPTER_KIND_REJECTED",
        ),
        (
            MlsProviderAdapterSelectionReason::CryptoBackendRejected,
            3,
            "CRYPTO_BACKEND_REJECTED",
        ),
        (
            MlsProviderAdapterSelectionReason::ProtocolProfileRejected,
            4,
            "PROTOCOL_PROFILE_REJECTED",
        ),
        (
            MlsProviderAdapterSelectionReason::LicenseRejected,
            5,
            "LICENSE_REJECTED",
        ),
        (
            MlsProviderAdapterSelectionReason::SourceAuthenticityMissing,
            6,
            "SOURCE_AUTHENTICITY_MISSING",
        ),
        (
            MlsProviderAdapterSelectionReason::Rfc9420ConformanceMissing,
            7,
            "RFC9420_CONFORMANCE_MISSING",
        ),
        (
            MlsProviderAdapterSelectionReason::PqDraftPinMissing,
            8,
            "PQ_DRAFT_PIN_MISSING",
        ),
        (
            MlsProviderAdapterSelectionReason::MlKemStandardMissing,
            9,
            "ML_KEM_STANDARD_MISSING",
        ),
        (
            MlsProviderAdapterSelectionReason::PqSignatureStandardMissing,
            10,
            "PQ_SIGNATURE_STANDARD_MISSING",
        ),
        (
            MlsProviderAdapterSelectionReason::KatOrInteropMissing,
            11,
            "KAT_OR_INTEROP_MISSING",
        ),
        (
            MlsProviderAdapterSelectionReason::StorageProviderUnsafe,
            12,
            "STORAGE_PROVIDER_UNSAFE",
        ),
        (
            MlsProviderAdapterSelectionReason::SecretLifecycleUnsafe,
            13,
            "SECRET_LIFECYCLE_UNSAFE",
        ),
        (
            MlsProviderAdapterSelectionReason::DowngradeTestMissing,
            14,
            "DOWNGRADE_TEST_MISSING",
        ),
        (
            MlsProviderAdapterSelectionReason::TranscriptBindingMissing,
            15,
            "TRANSCRIPT_BINDING_MISSING",
        ),
        (
            MlsProviderAdapterSelectionReason::UnsafeFeaturesEnabled,
            16,
            "UNSAFE_FEATURES_ENABLED",
        ),
        (
            MlsProviderAdapterSelectionReason::PlaintextExportEnabled,
            17,
            "PLAINTEXT_EXPORT_ENABLED",
        ),
        (
            MlsProviderAdapterSelectionReason::ReleaseArtifactUnverified,
            18,
            "RELEASE_ARTIFACT_UNVERIFIED",
        ),
        (
            MlsProviderAdapterSelectionReason::SbomOrCveMonitoringMissing,
            19,
            "SBOM_OR_CVE_MONITORING_MISSING",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    assert_eq!(MlsProviderAdapterKind::MlsRs.label(), "mls_rs");
    assert_eq!(MlsProviderAdapterKind::NativePlatformMls.code(), 3);
    assert_eq!(MlsProviderCryptoBackendKind::AwsLcRs.label(), "aws_lc_rs");
    assert_eq!(
        MlsProviderCryptoBackendKind::PlatformCryptoProvider.code(),
        5
    );
    assert_eq!(
        MlsProviderProtocolProfile::DraftPqHybridWithPqSignatures.label(),
        "draft_pq_hybrid_with_pq_signatures"
    );
    assert_eq!(
        MlsProviderImplementationLicenseKind::DualApacheMit.code(),
        3
    );
}

fn valid_input() -> MlsProviderAdapterSelectionInput {
    MlsProviderAdapterSelectionInput {
        provider_security: valid_provider_security_input(GroupChatCryptoSuite::HybridPqMls768)
            .evaluate(),
        adapter_kind: MlsProviderAdapterKind::OpenMls,
        crypto_backend: MlsProviderCryptoBackendKind::LibcruxHybridPq,
        protocol_profile: MlsProviderProtocolProfile::DraftPqHybrid,
        license_kind: MlsProviderImplementationLicenseKind::Mit,
        source_verified: true,
        license_allows_distribution: true,
        rfc9420_conformance_tests_passed: true,
        pq_draft_version_pinned: true,
        ml_kem_standardized: true,
        pq_signature_standardized_when_required: true,
        kat_vectors_passed: true,
        interop_tests_passed: true,
        storage_provider_seals_group_state: true,
        storage_provider_transactional: true,
        secret_zeroization_audited: true,
        memory_hardening_enabled: true,
        downgrade_tests_passed: true,
        transcript_hash_binding_verified: true,
        unsafe_features_enabled: false,
        plaintext_export_enabled: false,
        release_artifact_signed: true,
        sbom_present: true,
        cve_monitoring_enabled: true,
    }
}

fn valid_provider_security_input(suite: GroupChatCryptoSuite) -> MlsProviderSecurityInput {
    MlsProviderSecurityInput {
        provider_configured: true,
        selected_suite: suite,
        minimum_suite: suite,
        provider_supports_selected_suite: true,
        ml_kem_parameter_set: suite.required_ml_kem_parameter_set(),
        classical_kem_component_present: suite.requires_pq_traditional_hybrid(),
        requires_pq_signatures: suite.is_high_security_pq(),
        pq_signature_ready: suite.is_high_security_pq(),
        suite_id_bound_to_group_context: true,
        downgrade_evidence_verified: true,
        known_answer_tests_passed: true,
        secret_zeroization_available: true,
        unsafe_crypto_backend: false,
        plaintext_key_export_fields: 0,
    }
}

fn assert_rejected(
    decision: mercury_core::MlsProviderAdapterSelectionDecision,
    reason: MlsProviderAdapterSelectionReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_link_provider);
    assert!(!decision.can_open_mls_group);
    assert!(!decision.can_change_membership);
    assert!(!decision.can_ship_release);
    assert_eq!(decision.reason, reason);
    assert!(decision.forbids_plaintext_key_export);
}
