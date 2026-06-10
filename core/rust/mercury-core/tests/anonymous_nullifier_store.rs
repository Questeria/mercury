use mercury_core::{
    AnonymousGroupMembershipProofDecision, AnonymousGroupMembershipProofReason,
    AnonymousNullifierStoreReason, AnonymousNullifierStoreWrite, AnonymousRateLimitCredentialKind,
    AnonymousRateLimitNullifierDecision, AnonymousRateLimitNullifierInput,
    AnonymousRateLimitNullifierReason, PrototypeAnonymousNullifierStore,
    evaluate_anonymous_nullifier_store_write, put_anonymous_nullifier_record,
};

const NULLIFIER: [u8; 32] = [0xA7; 32];
const OTHER_NULLIFIER: [u8; 32] = [0xB8; 32];
const REDEMPTION_CONTEXT_DIGEST: [u8; 32] = [0xC9; 32];
const CREDENTIAL_CONTEXT_DIGEST: [u8; 32] = [0xDA; 32];
const SHORT_DIGEST: [u8; 16] = [0xE1; 16];

#[test]
fn nullifier_store_persists_only_accepted_opaque_nullifiers() {
    let mut store = PrototypeAnonymousNullifierStore::default();
    let decision = put_anonymous_nullifier_record(&mut store, valid_write())
        .expect("prototype store cannot fail");

    assert!(decision.accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.reason, AnonymousNullifierStoreReason::Accepted);
    assert_eq!(decision.presentation_count_after, 2);
    assert_eq!(decision.record_count, 1);
    assert!(decision.keeps_context_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store.get(&NULLIFIER).expect("nullifier should persist");
    assert_eq!(record.nullifier, NULLIFIER);
    assert_eq!(record.redemption_context_digest, REDEMPTION_CONTEXT_DIGEST);
    assert_eq!(record.credential_context_digest, CREDENTIAL_CONTEXT_DIGEST);
    assert_eq!(
        record.credential_kind,
        AnonymousRateLimitCredentialKind::ArcWindow
    );
    assert_eq!(record.presentation_count, 2);
    assert_eq!(record.presentation_limit, 8);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn nullifier_store_rejects_gate_failures_and_bad_shapes() {
    let rejected_gate = AnonymousNullifierStoreWrite {
        nullifier_decision: rejected_nullifier(),
        ..valid_write()
    };
    assert_rejected(
        evaluate_anonymous_nullifier_store_write(rejected_gate),
        AnonymousNullifierStoreReason::NullifierGateRejected,
    );

    let bad_nullifier = AnonymousNullifierStoreWrite {
        nullifier: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_anonymous_nullifier_store_write(bad_nullifier),
        AnonymousNullifierStoreReason::BadNullifier,
    );

    let bad_redemption = AnonymousNullifierStoreWrite {
        redemption_context_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_anonymous_nullifier_store_write(bad_redemption),
        AnonymousNullifierStoreReason::BadRedemptionContextDigest,
    );

    let bad_credential = AnonymousNullifierStoreWrite {
        credential_context_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        evaluate_anonymous_nullifier_store_write(bad_credential),
        AnonymousNullifierStoreReason::BadCredentialContextDigest,
    );
}

#[test]
fn nullifier_store_rejects_replays_limits_windows_and_plaintext_metadata() {
    let mut store = PrototypeAnonymousNullifierStore::default();
    let first = store.put(valid_write());
    assert!(first.accepted);

    let replay = store.put(valid_write());
    assert_rejected(
        replay,
        AnonymousNullifierStoreReason::NullifierAlreadyRecorded,
    );
    assert_eq!(replay.record_count, 1);
    assert_eq!(store.len(), 1);

    let exhausted = AnonymousNullifierStoreWrite {
        nullifier: &OTHER_NULLIFIER,
        presentation_count_before: 8,
        ..valid_write()
    };
    assert_rejected(
        evaluate_anonymous_nullifier_store_write(exhausted),
        AnonymousNullifierStoreReason::PresentationLimitExceeded,
    );

    let bad_window = AnonymousNullifierStoreWrite {
        nullifier: &OTHER_NULLIFIER,
        window_end_s: 999,
        ..valid_write()
    };
    assert_rejected(
        evaluate_anonymous_nullifier_store_write(bad_window),
        AnonymousNullifierStoreReason::BadWindow,
    );

    let plaintext = AnonymousNullifierStoreWrite {
        nullifier: &OTHER_NULLIFIER,
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    assert_rejected(
        evaluate_anonymous_nullifier_store_write(plaintext),
        AnonymousNullifierStoreReason::PlaintextMetadataForbidden,
    );
}

#[test]
fn nullifier_store_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (AnonymousNullifierStoreReason::Accepted, 0, "ACCEPTED"),
        (
            AnonymousNullifierStoreReason::NullifierGateRejected,
            1,
            "NULLIFIER_GATE_REJECTED",
        ),
        (
            AnonymousNullifierStoreReason::BadNullifier,
            2,
            "BAD_NULLIFIER",
        ),
        (
            AnonymousNullifierStoreReason::BadRedemptionContextDigest,
            3,
            "BAD_REDEMPTION_CONTEXT_DIGEST",
        ),
        (
            AnonymousNullifierStoreReason::BadCredentialContextDigest,
            4,
            "BAD_CREDENTIAL_CONTEXT_DIGEST",
        ),
        (AnonymousNullifierStoreReason::BadWindow, 5, "BAD_WINDOW"),
        (
            AnonymousNullifierStoreReason::PresentationLimitExceeded,
            6,
            "PRESENTATION_LIMIT_EXCEEDED",
        ),
        (
            AnonymousNullifierStoreReason::PlaintextMetadataForbidden,
            7,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            AnonymousNullifierStoreReason::NullifierAlreadyRecorded,
            8,
            "NULLIFIER_ALREADY_RECORDED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_write() -> AnonymousNullifierStoreWrite<'static> {
    AnonymousNullifierStoreWrite {
        nullifier: &NULLIFIER,
        redemption_context_digest: &REDEMPTION_CONTEXT_DIGEST,
        credential_context_digest: &CREDENTIAL_CONTEXT_DIGEST,
        credential_kind: AnonymousRateLimitCredentialKind::ArcWindow,
        nullifier_decision: accepted_nullifier(),
        window_start_s: 1000,
        window_end_s: 1300,
        presentation_count_before: 1,
        presentation_limit: 8,
        plaintext_metadata_fields: 0,
    }
}

fn accepted_nullifier() -> AnonymousRateLimitNullifierDecision {
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
    .evaluate()
}

fn rejected_nullifier() -> AnonymousRateLimitNullifierDecision {
    AnonymousRateLimitNullifierDecision {
        accepted: false,
        can_record_nullifier: false,
        can_redeem_this_window: false,
        can_rate_limit_without_identity: false,
        requires_sync: false,
        requires_rekey: false,
        requires_user_action: true,
        forbids_plaintext_rate_limit_metadata: true,
        plaintext_bytes_exposed: false,
        reason: AnonymousRateLimitNullifierReason::NullifierAlreadySpent,
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

fn assert_rejected(
    decision: mercury_core::AnonymousNullifierStoreDecision,
    reason: AnonymousNullifierStoreReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert!(decision.keeps_context_digest_only);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
