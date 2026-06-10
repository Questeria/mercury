use mercury_core::{
    GroupChatCryptoSuite, GroupChatInput, GroupChatProtocol, GroupChatReason,
    MERCURY_MAX_GROUP_CHAT_MEMBERS, MlsProviderSecurityInput, RoomMode, evaluate_group_chat,
    evaluate_mls_provider_security,
};

#[test]
fn group_chat_accepts_mls_ready_small_group() {
    let decision = evaluate_group_chat(valid_input());

    assert!(decision.accepted);
    assert_eq!(decision.reason, GroupChatReason::Accepted);
    assert_eq!(decision.protocol, GroupChatProtocol::Mls);
    assert_eq!(decision.crypto_suite, GroupChatCryptoSuite::HybridPqMls768);
    assert!(decision.can_open_group);
    assert!(decision.can_send_message);
    assert!(decision.can_change_membership);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_mls_setup);
    assert!(!decision.requires_pq_upgrade);
    assert!(!decision.requires_user_action);
    assert!(decision.forbids_server_plaintext);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn group_chat_requires_real_group_member_bounds_and_local_membership() {
    let mut direct = valid_input();
    direct.member_count = 2;
    assert_rejected(direct.evaluate(), GroupChatReason::NotEnoughMembers);

    let mut too_large = valid_input();
    too_large.member_count = MERCURY_MAX_GROUP_CHAT_MEMBERS + 1;
    too_large.active_member_devices = too_large.member_count;
    assert_rejected(too_large.evaluate(), GroupChatReason::MemberLimitExceeded);

    let mut not_member = valid_input();
    not_member.local_device_is_member = false;
    assert_rejected(not_member.evaluate(), GroupChatReason::LocalDeviceNotMember);

    let mut missing_device = valid_input();
    missing_device.active_member_devices = missing_device.member_count - 1;
    assert_rejected(
        missing_device.evaluate(),
        GroupChatReason::ActiveMemberDeviceMissing,
    );
}

#[test]
fn group_chat_requires_synced_room_state_secret_epoch_and_transparency() {
    let mut missing_room = valid_input();
    missing_room.room_state_available = false;
    let missing_room_decision = missing_room.evaluate();
    assert_rejected(missing_room_decision, GroupChatReason::RoomStateMissing);
    assert!(missing_room_decision.requires_sync);

    let mut missing_secret = valid_input();
    missing_secret.group_secret_sealed = false;
    let missing_secret_decision = missing_secret.evaluate();
    assert_rejected(missing_secret_decision, GroupChatReason::GroupSecretMissing);
    assert!(missing_secret_decision.requires_sync);

    let mut pending = valid_input();
    pending.membership_transition_pending = true;
    let pending_decision = pending.evaluate();
    assert_rejected(
        pending_decision,
        GroupChatReason::MembershipTransitionPending,
    );
    assert!(pending_decision.requires_sync);
    assert!(pending_decision.requires_user_action);

    let mut stale_epoch = valid_input();
    stale_epoch.local_epoch = stale_epoch.current_epoch - 1;
    let stale_decision = stale_epoch.evaluate();
    assert_rejected(stale_decision, GroupChatReason::EpochNotCurrent);
    assert!(stale_decision.requires_sync);

    let mut transparency = valid_input();
    transparency.key_transparency_ready = false;
    let transparency_decision = transparency.evaluate();
    assert_rejected(
        transparency_decision,
        GroupChatReason::KeyTransparencyNotReady,
    );
    assert!(transparency_decision.requires_sync);
}

#[test]
fn group_chat_blocks_plaintext_metadata_and_missing_mls_setup() {
    let mut plaintext = valid_input();
    plaintext.plaintext_member_metadata_fields = 1;
    assert_rejected(
        plaintext.evaluate(),
        GroupChatReason::PlaintextMetadataForbidden,
    );

    let mut missing_mls = valid_input();
    missing_mls.mls_provider_configured = false;
    let missing_mls_decision = missing_mls.evaluate();
    assert_rejected(missing_mls_decision, GroupChatReason::MlsProviderMissing);
    assert!(missing_mls_decision.requires_mls_setup);
}

#[test]
fn group_chat_requires_accepted_mls_provider_security() {
    let mut weak_provider = valid_input();
    let mut provider_input =
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768);
    provider_input.known_answer_tests_passed = false;
    weak_provider.mls_provider_security = provider_input.evaluate();
    let weak_decision = weak_provider.evaluate();

    assert_rejected(weak_decision, GroupChatReason::MlsProviderSecurityRejected);
    assert!(weak_decision.requires_mls_setup);
    assert!(!weak_decision.requires_pq_upgrade);
    assert!(weak_decision.requires_user_action);

    let mut mismatched_suite = valid_input();
    mismatched_suite.mls_provider_security = evaluate_mls_provider_security(
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls1024),
    );
    let mismatch_decision = mismatched_suite.evaluate();
    assert_rejected(
        mismatch_decision,
        GroupChatReason::MlsProviderSecurityRejected,
    );
    assert!(mismatch_decision.requires_pq_upgrade);
    assert!(mismatch_decision.requires_user_action);
}

#[test]
fn high_security_group_requires_mls_not_transitional_fanout() {
    let mut high_security = valid_input();
    high_security.room_mode = RoomMode::HighSecurity;
    high_security.protocol = GroupChatProtocol::TransitionalPairwiseFanout;
    high_security.mls_provider_configured = false;
    let decision = high_security.evaluate();

    assert_rejected(decision, GroupChatReason::HighSecurityRequiresMls);
    assert!(decision.requires_mls_setup);
    assert!(decision.requires_user_action);

    high_security.protocol = GroupChatProtocol::Mls;
    high_security.mls_provider_configured = true;
    high_security.crypto_suite = GroupChatCryptoSuite::HybridPqMls1024;
    high_security.mls_provider_security = evaluate_mls_provider_security(
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls1024),
    );
    assert!(high_security.evaluate().accepted);
}

#[test]
fn high_security_group_requires_post_quantum_hybrid_suite() {
    let mut high_security = valid_input();
    high_security.room_mode = RoomMode::HighSecurity;
    high_security.crypto_suite = GroupChatCryptoSuite::ClassicalMls128;
    let classical_decision = high_security.evaluate();
    assert_rejected(
        classical_decision,
        GroupChatReason::HighSecurityRequiresPqHybridSuite,
    );
    assert!(classical_decision.requires_mls_setup);
    assert!(classical_decision.requires_pq_upgrade);
    assert!(classical_decision.requires_user_action);

    high_security.crypto_suite = GroupChatCryptoSuite::HybridPqMls768;
    let standard_pq_decision = high_security.evaluate();
    assert_rejected(
        standard_pq_decision,
        GroupChatReason::HighSecurityRequiresPqHybridSuite,
    );
    assert!(standard_pq_decision.requires_pq_upgrade);

    high_security.crypto_suite = GroupChatCryptoSuite::HybridPqMls1024;
    high_security.mls_provider_security = evaluate_mls_provider_security(
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls1024),
    );
    assert!(high_security.evaluate().accepted);
}

#[test]
fn standard_group_can_use_transitional_fanout_only_as_explicit_protocol() {
    let mut transitional = valid_input();
    transitional.protocol = GroupChatProtocol::TransitionalPairwiseFanout;
    transitional.mls_provider_configured = false;
    let decision = transitional.evaluate();

    assert!(decision.accepted);
    assert_eq!(
        decision.protocol,
        GroupChatProtocol::TransitionalPairwiseFanout
    );
    assert!(decision.forbids_server_plaintext);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn group_chat_protocols_and_reasons_have_stable_codes_and_labels() {
    let protocols = [
        (GroupChatProtocol::Mls, 1, "mls"),
        (
            GroupChatProtocol::TransitionalPairwiseFanout,
            2,
            "transitional_pairwise_fanout",
        ),
    ];

    for (protocol, code, label) in protocols {
        assert_eq!(protocol.code(), code);
        assert_eq!(protocol.label(), label);
    }

    let suites = [
        (
            GroupChatCryptoSuite::ClassicalMls128,
            1,
            "classical_mls_128",
            false,
        ),
        (
            GroupChatCryptoSuite::HybridPqMls768,
            2,
            "hybrid_pq_mls_768",
            false,
        ),
        (
            GroupChatCryptoSuite::HybridPqMls1024,
            3,
            "hybrid_pq_mls_1024",
            true,
        ),
    ];

    for (suite, code, label, high_security_ready) in suites {
        assert_eq!(suite.code(), code);
        assert_eq!(suite.label(), label);
        assert_eq!(suite.is_high_security_pq(), high_security_ready);
    }

    let reasons = [
        (GroupChatReason::Accepted, 0, "ACCEPTED"),
        (GroupChatReason::NotEnoughMembers, 1, "NOT_ENOUGH_MEMBERS"),
        (
            GroupChatReason::MemberLimitExceeded,
            2,
            "MEMBER_LIMIT_EXCEEDED",
        ),
        (
            GroupChatReason::LocalDeviceNotMember,
            3,
            "LOCAL_DEVICE_NOT_MEMBER",
        ),
        (
            GroupChatReason::ActiveMemberDeviceMissing,
            4,
            "ACTIVE_MEMBER_DEVICE_MISSING",
        ),
        (GroupChatReason::RoomStateMissing, 5, "ROOM_STATE_MISSING"),
        (
            GroupChatReason::GroupSecretMissing,
            6,
            "GROUP_SECRET_MISSING",
        ),
        (
            GroupChatReason::MembershipTransitionPending,
            7,
            "MEMBERSHIP_TRANSITION_PENDING",
        ),
        (GroupChatReason::EpochNotCurrent, 8, "EPOCH_NOT_CURRENT"),
        (
            GroupChatReason::KeyTransparencyNotReady,
            9,
            "KEY_TRANSPARENCY_NOT_READY",
        ),
        (
            GroupChatReason::MlsProviderMissing,
            10,
            "MLS_PROVIDER_MISSING",
        ),
        (
            GroupChatReason::HighSecurityRequiresMls,
            11,
            "HIGH_SECURITY_REQUIRES_MLS",
        ),
        (
            GroupChatReason::PlaintextMetadataForbidden,
            12,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            GroupChatReason::HighSecurityRequiresPqHybridSuite,
            13,
            "HIGH_SECURITY_REQUIRES_PQ_HYBRID_SUITE",
        ),
        (
            GroupChatReason::MlsProviderSecurityRejected,
            14,
            "MLS_PROVIDER_SECURITY_REJECTED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> GroupChatInput {
    GroupChatInput {
        protocol: GroupChatProtocol::Mls,
        crypto_suite: GroupChatCryptoSuite::HybridPqMls768,
        room_mode: RoomMode::Standard,
        member_count: 5,
        active_member_devices: 5,
        local_device_is_member: true,
        room_state_available: true,
        group_secret_sealed: true,
        membership_transition_pending: false,
        current_epoch: 7,
        local_epoch: 7,
        key_transparency_ready: true,
        mls_provider_configured: true,
        mls_provider_security: evaluate_mls_provider_security(valid_mls_provider_security_input(
            GroupChatCryptoSuite::HybridPqMls768,
        )),
        plaintext_member_metadata_fields: 0,
    }
}

fn valid_mls_provider_security_input(suite: GroupChatCryptoSuite) -> MlsProviderSecurityInput {
    MlsProviderSecurityInput {
        provider_configured: true,
        selected_suite: suite,
        minimum_suite: suite,
        provider_supports_selected_suite: true,
        ml_kem_parameter_set: suite.required_ml_kem_parameter_set(),
        classical_kem_component_present: suite.requires_pq_traditional_hybrid(),
        requires_pq_signatures: matches!(suite, GroupChatCryptoSuite::HybridPqMls1024),
        pq_signature_ready: matches!(suite, GroupChatCryptoSuite::HybridPqMls1024),
        suite_id_bound_to_group_context: true,
        downgrade_evidence_verified: true,
        known_answer_tests_passed: true,
        secret_zeroization_available: true,
        unsafe_crypto_backend: false,
        plaintext_key_export_fields: 0,
    }
}

fn assert_rejected(decision: mercury_core::GroupChatDecision, reason: GroupChatReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_open_group);
    assert!(!decision.can_send_message);
    assert!(!decision.can_change_membership);
    assert!(decision.forbids_server_plaintext);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
