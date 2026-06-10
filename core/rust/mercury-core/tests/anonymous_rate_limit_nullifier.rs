use mercury_core::{
    AnonymousGroupMembershipProofDecision, AnonymousGroupMembershipProofReason,
    AnonymousRateLimitCredentialKind, AnonymousRateLimitNullifierDecision,
    AnonymousRateLimitNullifierInput, AnonymousRateLimitNullifierReason,
    evaluate_anonymous_rate_limit_nullifier,
};

#[test]
fn nullifier_window_accepts_bound_arc_redemption() {
    let decision = evaluate_anonymous_rate_limit_nullifier(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_record_nullifier);
    assert!(decision.can_redeem_this_window);
    assert!(decision.can_rate_limit_without_identity);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_rekey);
    assert!(!decision.requires_user_action);
    assert!(decision.forbids_plaintext_rate_limit_metadata);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, AnonymousRateLimitNullifierReason::Accepted);
}

#[test]
fn nullifier_window_requires_accepted_membership_proof() {
    let mut input = valid_input();
    input.membership_proof = rejected_membership_proof();
    let decision = input.evaluate();

    assert_rejected(
        decision,
        AnonymousRateLimitNullifierReason::MembershipProofRejected,
    );
    assert!(decision.requires_sync);
}

#[test]
fn nullifier_window_requires_unique_opaque_nullifier_storage() {
    let mut bad_nullifier = valid_input();
    bad_nullifier.nullifier_len = 16;
    assert_rejected(
        bad_nullifier.evaluate(),
        AnonymousRateLimitNullifierReason::BadNullifier,
    );

    let mut spent = valid_input();
    spent.nullifier_already_spent = true;
    let spent_decision = spent.evaluate();
    assert_rejected(
        spent_decision,
        AnonymousRateLimitNullifierReason::NullifierAlreadySpent,
    );
    assert!(spent_decision.requires_user_action);

    let mut store_unavailable = valid_input();
    store_unavailable.nullifier_store_available = false;
    let unavailable_decision = store_unavailable.evaluate();
    assert_rejected(
        unavailable_decision,
        AnonymousRateLimitNullifierReason::NullifierStoreUnavailable,
    );
    assert!(unavailable_decision.requires_sync);

    let mut plaintext_store = valid_input();
    plaintext_store.nullifier_store_opaque = false;
    let plaintext_decision = plaintext_store.evaluate();
    assert_rejected(
        plaintext_decision,
        AnonymousRateLimitNullifierReason::NullifierStoreNotOpaque,
    );
    assert!(plaintext_decision.requires_rekey);
    assert!(plaintext_decision.requires_user_action);
}

#[test]
fn nullifier_window_requires_context_binding_and_fresh_windows() {
    let mut unbound = valid_input();
    unbound.bound_to_route = false;
    assert_rejected(
        unbound.evaluate(),
        AnonymousRateLimitNullifierReason::ContextNotBound,
    );

    let mut bad_redemption_context = valid_input();
    bad_redemption_context.redemption_context_len = 0;
    assert_rejected(
        bad_redemption_context.evaluate(),
        AnonymousRateLimitNullifierReason::BadRedemptionContext,
    );

    let mut bad_credential_context = valid_input();
    bad_credential_context.credential_context_len = 0;
    assert_rejected(
        bad_credential_context.evaluate(),
        AnonymousRateLimitNullifierReason::BadCredentialContext,
    );

    let mut bad_window = valid_input();
    bad_window.now_s = bad_window.window_start_s - 1;
    assert_rejected(
        bad_window.evaluate(),
        AnonymousRateLimitNullifierReason::BadWindow,
    );

    let mut expired_window = valid_input();
    expired_window.now_s = expired_window.window_end_s;
    let expired_decision = expired_window.evaluate();
    assert_rejected(
        expired_decision,
        AnonymousRateLimitNullifierReason::WindowExpired,
    );
    assert!(expired_decision.requires_sync);
}

#[test]
fn nullifier_window_enforces_presentation_limits() {
    let mut bad_limit = valid_input();
    bad_limit.presentation_limit = 0;
    assert_rejected(
        bad_limit.evaluate(),
        AnonymousRateLimitNullifierReason::BadPresentationLimit,
    );

    let mut over_max = valid_input();
    over_max.presentation_limit = 9;
    over_max.max_presentation_limit = 8;
    assert_rejected(
        over_max.evaluate(),
        AnonymousRateLimitNullifierReason::BadPresentationLimit,
    );

    let mut exhausted = valid_input();
    exhausted.presentation_count = exhausted.presentation_limit;
    let exhausted_decision = exhausted.evaluate();
    assert_rejected(
        exhausted_decision,
        AnonymousRateLimitNullifierReason::PresentationLimitExceeded,
    );
    assert!(exhausted_decision.requires_user_action);

    let mut one_time = valid_input();
    one_time.credential_kind = AnonymousRateLimitCredentialKind::OneTimeRedemption;
    one_time.presentation_limit = 2;
    let one_time_decision = one_time.evaluate();
    assert_rejected(
        one_time_decision,
        AnonymousRateLimitNullifierReason::OneTimeRequiresSingleUse,
    );
    assert!(one_time_decision.requires_rekey);
}

#[test]
fn nullifier_window_rejects_plaintext_rate_limit_metadata() {
    let mut input = valid_input();
    input.plaintext_rate_limit_fields = 1;
    let decision = input.evaluate();

    assert_rejected(
        decision,
        AnonymousRateLimitNullifierReason::PlaintextRateLimitMetadata,
    );
    assert!(decision.requires_rekey);
    assert!(decision.requires_user_action);
}

#[test]
fn nullifier_window_reasons_and_kinds_have_stable_codes_and_labels() {
    let kinds = [
        (
            AnonymousRateLimitCredentialKind::OneTimeRedemption,
            1,
            "one_time_redemption",
        ),
        (AnonymousRateLimitCredentialKind::ArcWindow, 2, "arc_window"),
    ];

    for (kind, code, label) in kinds {
        assert_eq!(kind.code(), code);
        assert_eq!(kind.label(), label);
    }

    let reasons = [
        (AnonymousRateLimitNullifierReason::Accepted, 0, "ACCEPTED"),
        (
            AnonymousRateLimitNullifierReason::MembershipProofRejected,
            1,
            "MEMBERSHIP_PROOF_REJECTED",
        ),
        (
            AnonymousRateLimitNullifierReason::BadNullifier,
            2,
            "BAD_NULLIFIER",
        ),
        (
            AnonymousRateLimitNullifierReason::NullifierAlreadySpent,
            3,
            "NULLIFIER_ALREADY_SPENT",
        ),
        (
            AnonymousRateLimitNullifierReason::NullifierStoreUnavailable,
            4,
            "NULLIFIER_STORE_UNAVAILABLE",
        ),
        (
            AnonymousRateLimitNullifierReason::NullifierStoreNotOpaque,
            5,
            "NULLIFIER_STORE_NOT_OPAQUE",
        ),
        (
            AnonymousRateLimitNullifierReason::ContextNotBound,
            6,
            "CONTEXT_NOT_BOUND",
        ),
        (
            AnonymousRateLimitNullifierReason::BadRedemptionContext,
            7,
            "BAD_REDEMPTION_CONTEXT",
        ),
        (
            AnonymousRateLimitNullifierReason::BadCredentialContext,
            8,
            "BAD_CREDENTIAL_CONTEXT",
        ),
        (
            AnonymousRateLimitNullifierReason::BadWindow,
            9,
            "BAD_WINDOW",
        ),
        (
            AnonymousRateLimitNullifierReason::WindowExpired,
            10,
            "WINDOW_EXPIRED",
        ),
        (
            AnonymousRateLimitNullifierReason::BadPresentationLimit,
            11,
            "BAD_PRESENTATION_LIMIT",
        ),
        (
            AnonymousRateLimitNullifierReason::PresentationLimitExceeded,
            12,
            "PRESENTATION_LIMIT_EXCEEDED",
        ),
        (
            AnonymousRateLimitNullifierReason::OneTimeRequiresSingleUse,
            13,
            "ONE_TIME_REQUIRES_SINGLE_USE",
        ),
        (
            AnonymousRateLimitNullifierReason::PlaintextRateLimitMetadata,
            14,
            "PLAINTEXT_RATE_LIMIT_METADATA",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> AnonymousRateLimitNullifierInput {
    AnonymousRateLimitNullifierInput {
        membership_proof: accepted_membership_proof(),
        credential_kind: AnonymousRateLimitCredentialKind::ArcWindow,
        nullifier_len: 32,
        nullifier_already_spent: false,
        nullifier_store_available: true,
        nullifier_store_opaque: true,
        bound_to_route: true,
        bound_to_group_epoch: true,
        redemption_context_len: 32,
        credential_context_len: 32,
        window_start_s: 1000,
        window_end_s: 1300,
        now_s: 1100,
        presentation_count: 1,
        presentation_limit: 8,
        max_presentation_limit: 8,
        plaintext_rate_limit_fields: 0,
    }
}

fn accepted_membership_proof() -> AnonymousGroupMembershipProofDecision {
    AnonymousGroupMembershipProofDecision {
        accepted: true,
        can_authenticate_member: true,
        can_redeem_once: true,
        can_rate_limit_anonymously: true,
        requires_sync: false,
        requires_rekey: false,
        requires_user_action: false,
        forbids_plaintext_member_identity: true,
        plaintext_bytes_exposed: false,
        reason: AnonymousGroupMembershipProofReason::Accepted,
    }
}

fn rejected_membership_proof() -> AnonymousGroupMembershipProofDecision {
    AnonymousGroupMembershipProofDecision {
        accepted: false,
        can_authenticate_member: false,
        can_redeem_once: false,
        can_rate_limit_anonymously: false,
        requires_sync: true,
        requires_rekey: false,
        requires_user_action: false,
        forbids_plaintext_member_identity: true,
        plaintext_bytes_exposed: false,
        reason: AnonymousGroupMembershipProofReason::GroupRejected,
    }
}

fn assert_rejected(
    decision: AnonymousRateLimitNullifierDecision,
    reason: AnonymousRateLimitNullifierReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_record_nullifier);
    assert!(!decision.can_redeem_this_window);
    assert!(!decision.can_rate_limit_without_identity);
    assert!(decision.forbids_plaintext_rate_limit_metadata);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
