use mercury_core::{
    AnonymousGroupMembershipProofDecision, AnonymousGroupMembershipProofReason,
    AnonymousRateLimitNullifierDecision, AnonymousRateLimitNullifierReason,
    GroupMessageTranscriptDecision, GroupMessageTranscriptReason, GroupRelayEnvelopeInput,
    GroupRelayEnvelopeReason, RelaySubmissionDecision, evaluate_group_relay_envelope,
};
use mercury_policy::{RELAY_SUBMIT_ACCEPT, RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN};

#[test]
fn group_relay_envelope_accepts_metadata_hidden_group_submit() {
    let decision = evaluate_group_relay_envelope(valid_input());

    assert!(decision.accepted);
    assert!(decision.can_enqueue_relay);
    assert!(!decision.requires_sync);
    assert!(!decision.requires_rekey);
    assert!(!decision.requires_user_action);
    assert!(decision.forbids_plaintext_sender);
    assert!(decision.forbids_plaintext_group);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, GroupRelayEnvelopeReason::Accepted);
}

#[test]
fn group_relay_envelope_requires_accepted_transcript_and_relay_submission() {
    let mut sync_required = valid_input();
    sync_required.transcript = rejected_transcript(GroupMessageTranscriptReason::EpochMismatch);
    sync_required.transcript.requires_sync = true;
    let sync_decision = sync_required.evaluate();
    assert_rejected(sync_decision, GroupRelayEnvelopeReason::TranscriptRejected);
    assert!(sync_decision.requires_sync);
    assert!(!sync_decision.requires_rekey);

    let mut rekey_required = valid_input();
    rekey_required.transcript =
        rejected_transcript(GroupMessageTranscriptReason::SenderDataNotSealed);
    rekey_required.transcript.requires_rekey = true;
    let rekey_decision = rekey_required.evaluate();
    assert_rejected(rekey_decision, GroupRelayEnvelopeReason::TranscriptRejected);
    assert!(!rekey_decision.requires_sync);
    assert!(rekey_decision.requires_rekey);

    let mut relay_rejected = valid_input();
    relay_rejected.relay_submission = RelaySubmissionDecision {
        accepted: false,
        reason_code: RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN,
        audit_class: 0,
    };
    let relay_decision = relay_rejected.evaluate();
    assert_rejected(
        relay_decision,
        GroupRelayEnvelopeReason::RelaySubmissionRejected,
    );
}

#[test]
fn group_relay_envelope_requires_sealed_sender_style_auth_material() {
    let mut bad_token = valid_input();
    bad_token.delivery_token_len = 0;
    let bad_token_decision = bad_token.evaluate();
    assert_rejected(
        bad_token_decision,
        GroupRelayEnvelopeReason::MissingDeliveryToken,
    );
    assert!(bad_token_decision.requires_user_action);

    let mut unbound_token = valid_input();
    unbound_token.delivery_token_bound_to_route = false;
    assert_rejected(
        unbound_token.evaluate(),
        GroupRelayEnvelopeReason::DeliveryTokenNotRouteBound,
    );

    let mut unsealed_certificate = valid_input();
    unsealed_certificate.sender_certificate_sealed = false;
    assert_rejected(
        unsealed_certificate.evaluate(),
        GroupRelayEnvelopeReason::SenderCertificateNotSealed,
    );

    let mut missing_membership_proof = valid_input();
    missing_membership_proof.anonymous_membership_proof_len = 0;
    assert_rejected(
        missing_membership_proof.evaluate(),
        GroupRelayEnvelopeReason::AnonymousMembershipProofMissing,
    );

    let mut rejected_membership_proof = valid_input();
    rejected_membership_proof.anonymous_membership_proof = rejected_membership_proof_decision();
    let rejected_membership_decision = rejected_membership_proof.evaluate();
    assert_rejected(
        rejected_membership_decision,
        GroupRelayEnvelopeReason::AnonymousMembershipProofRejected,
    );
    assert!(rejected_membership_decision.requires_sync);

    let mut rejected_rate_limit = valid_input();
    rejected_rate_limit.anonymous_rate_limit = rejected_rate_limit_decision();
    let rejected_rate_limit_decision = rejected_rate_limit.evaluate();
    assert_rejected(
        rejected_rate_limit_decision,
        GroupRelayEnvelopeReason::AnonymousRateLimitRejected,
    );
    assert!(rejected_rate_limit_decision.requires_rekey);
    assert!(rejected_rate_limit_decision.requires_user_action);

    let mut missing_envelope = valid_input();
    missing_envelope.sealed_envelope_len = 0;
    let missing_envelope_decision = missing_envelope.evaluate();
    assert_rejected(
        missing_envelope_decision,
        GroupRelayEnvelopeReason::SealedEnvelopeMissing,
    );
    assert!(missing_envelope_decision.requires_rekey);
}

#[test]
fn group_relay_envelope_rejects_plaintext_sender_or_group_metadata() {
    let mut sender_metadata = valid_input();
    sender_metadata.plaintext_sender_fields = 1;
    assert_rejected(
        sender_metadata.evaluate(),
        GroupRelayEnvelopeReason::PlaintextSenderMetadata,
    );

    let mut group_metadata = valid_input();
    group_metadata.plaintext_group_fields = 1;
    assert_rejected(
        group_metadata.evaluate(),
        GroupRelayEnvelopeReason::PlaintextGroupMetadata,
    );
}

#[test]
fn group_relay_envelope_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (GroupRelayEnvelopeReason::Accepted, 0, "ACCEPTED"),
        (
            GroupRelayEnvelopeReason::TranscriptRejected,
            1,
            "TRANSCRIPT_REJECTED",
        ),
        (
            GroupRelayEnvelopeReason::RelaySubmissionRejected,
            2,
            "RELAY_SUBMISSION_REJECTED",
        ),
        (
            GroupRelayEnvelopeReason::MissingDeliveryToken,
            3,
            "MISSING_DELIVERY_TOKEN",
        ),
        (
            GroupRelayEnvelopeReason::DeliveryTokenNotRouteBound,
            4,
            "DELIVERY_TOKEN_NOT_ROUTE_BOUND",
        ),
        (
            GroupRelayEnvelopeReason::SenderCertificateNotSealed,
            5,
            "SENDER_CERTIFICATE_NOT_SEALED",
        ),
        (
            GroupRelayEnvelopeReason::AnonymousMembershipProofMissing,
            6,
            "ANONYMOUS_MEMBERSHIP_PROOF_MISSING",
        ),
        (
            GroupRelayEnvelopeReason::SealedEnvelopeMissing,
            7,
            "SEALED_ENVELOPE_MISSING",
        ),
        (
            GroupRelayEnvelopeReason::PlaintextSenderMetadata,
            8,
            "PLAINTEXT_SENDER_METADATA",
        ),
        (
            GroupRelayEnvelopeReason::PlaintextGroupMetadata,
            9,
            "PLAINTEXT_GROUP_METADATA",
        ),
        (
            GroupRelayEnvelopeReason::AnonymousMembershipProofRejected,
            10,
            "ANONYMOUS_MEMBERSHIP_PROOF_REJECTED",
        ),
        (
            GroupRelayEnvelopeReason::AnonymousRateLimitRejected,
            11,
            "ANONYMOUS_RATE_LIMIT_REJECTED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> GroupRelayEnvelopeInput {
    GroupRelayEnvelopeInput {
        transcript: accepted_transcript(),
        relay_submission: accepted_relay_submission(),
        delivery_token_len: 12,
        delivery_token_bound_to_route: true,
        sender_certificate_sealed: true,
        anonymous_membership_proof: accepted_membership_proof_decision(),
        anonymous_membership_proof_len: 64,
        anonymous_rate_limit: accepted_rate_limit_decision(),
        sealed_envelope_len: 128,
        plaintext_sender_fields: 0,
        plaintext_group_fields: 0,
    }
}

fn accepted_transcript() -> GroupMessageTranscriptDecision {
    GroupMessageTranscriptDecision {
        accepted: true,
        can_persist_ciphertext: true,
        can_submit_to_relay: true,
        requires_sync: false,
        requires_rekey: false,
        requires_user_action: false,
        forbids_plaintext: true,
        plaintext_bytes_exposed: false,
        reason: GroupMessageTranscriptReason::Accepted,
    }
}

fn rejected_transcript(reason: GroupMessageTranscriptReason) -> GroupMessageTranscriptDecision {
    GroupMessageTranscriptDecision {
        accepted: false,
        can_persist_ciphertext: false,
        can_submit_to_relay: false,
        requires_sync: false,
        requires_rekey: false,
        requires_user_action: false,
        forbids_plaintext: true,
        plaintext_bytes_exposed: false,
        reason,
    }
}

fn accepted_relay_submission() -> RelaySubmissionDecision {
    RelaySubmissionDecision {
        accepted: true,
        reason_code: RELAY_SUBMIT_ACCEPT,
        audit_class: 0,
    }
}

fn accepted_membership_proof_decision() -> AnonymousGroupMembershipProofDecision {
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

fn rejected_membership_proof_decision() -> AnonymousGroupMembershipProofDecision {
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

fn accepted_rate_limit_decision() -> AnonymousRateLimitNullifierDecision {
    AnonymousRateLimitNullifierDecision {
        accepted: true,
        can_record_nullifier: true,
        can_redeem_this_window: true,
        can_rate_limit_without_identity: true,
        requires_sync: false,
        requires_rekey: false,
        requires_user_action: false,
        forbids_plaintext_rate_limit_metadata: true,
        plaintext_bytes_exposed: false,
        reason: AnonymousRateLimitNullifierReason::Accepted,
    }
}

fn rejected_rate_limit_decision() -> AnonymousRateLimitNullifierDecision {
    AnonymousRateLimitNullifierDecision {
        accepted: false,
        can_record_nullifier: false,
        can_redeem_this_window: false,
        can_rate_limit_without_identity: false,
        requires_sync: false,
        requires_rekey: true,
        requires_user_action: true,
        forbids_plaintext_rate_limit_metadata: true,
        plaintext_bytes_exposed: false,
        reason: AnonymousRateLimitNullifierReason::NullifierStoreNotOpaque,
    }
}

fn assert_rejected(
    decision: mercury_core::GroupRelayEnvelopeDecision,
    reason: GroupRelayEnvelopeReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_enqueue_relay);
    assert!(decision.forbids_plaintext_sender);
    assert!(decision.forbids_plaintext_group);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}
