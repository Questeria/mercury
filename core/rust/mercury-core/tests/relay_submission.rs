use mercury_core::{
    OutboundSendDecision, OutboundSendReason, RelaySubmissionInput, evaluate_relay_submission,
};
use mercury_policy::{
    RELAY_SUBMIT_ACCEPT, RELAY_SUBMIT_AUDIT_ACCEPTED_RELAY_SUBMISSION,
    RELAY_SUBMIT_AUDIT_CLIENT_SEND_REJECT, RELAY_SUBMIT_AUDIT_RELAY_METADATA_REJECT,
    RELAY_SUBMIT_AUDIT_RELAY_RETENTION_REJECT, RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN,
    RELAY_SUBMIT_SEND_GATE_REJECTED, RELAY_SUBMIT_TTL_TOO_LONG,
};

#[test]
fn accepted_send_gate_allows_relay_submission() {
    let decision = evaluate_relay_submission(valid_submission());

    assert!(decision.accepted);
    assert_eq!(decision.reason_code, RELAY_SUBMIT_ACCEPT);
    assert_eq!(
        decision.audit_class,
        RELAY_SUBMIT_AUDIT_ACCEPTED_RELAY_SUBMISSION
    );
}

#[test]
fn rejected_send_gate_blocks_relay_submission() {
    let mut input = valid_submission();
    input.outbound_send = rejected_send(OutboundSendReason::MessagePolicyRejected);

    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert_eq!(decision.reason_code, RELAY_SUBMIT_SEND_GATE_REJECTED);
    assert_eq!(decision.audit_class, RELAY_SUBMIT_AUDIT_CLIENT_SEND_REJECT);
}

#[test]
fn plaintext_identity_metadata_blocks_relay_submission() {
    let mut input = valid_submission();
    input.plaintext_identity_fields = 1;

    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert_eq!(
        decision.reason_code,
        RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN
    );
    assert_eq!(
        decision.audit_class,
        RELAY_SUBMIT_AUDIT_RELAY_METADATA_REJECT
    );
}

#[test]
fn queue_ttl_cannot_exceed_client_cap() {
    let mut input = valid_submission();
    input.queue_ttl_s = 90000;
    input.max_queue_ttl_s = 86400;

    let decision = input.evaluate();

    assert!(!decision.accepted);
    assert_eq!(decision.reason_code, RELAY_SUBMIT_TTL_TOO_LONG);
    assert_eq!(
        decision.audit_class,
        RELAY_SUBMIT_AUDIT_RELAY_RETENTION_REJECT
    );
}

fn valid_submission() -> RelaySubmissionInput {
    RelaySubmissionInput {
        version: 1,
        outbound_send: accepted_send(),
        route_id_len: 32,
        replay_token_len: 32,
        queue_ttl_s: 300,
        max_queue_ttl_s: 86400,
        ciphertext_len: 2048,
        max_ciphertext_len: 1048576,
        sealed_header_len: 96,
        plaintext_identity_fields: 0,
        padding_bucket: 3,
    }
}

fn accepted_send() -> OutboundSendDecision {
    OutboundSendDecision {
        accepted: true,
        can_send: true,
        can_persist_ciphertext: true,
        requires_user_action: false,
        reason: OutboundSendReason::Accepted,
    }
}

fn rejected_send(reason: OutboundSendReason) -> OutboundSendDecision {
    OutboundSendDecision {
        accepted: false,
        can_send: false,
        can_persist_ciphertext: false,
        requires_user_action: true,
        reason,
    }
}
