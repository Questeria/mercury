use mercury_core::{
    OutboundSendDecision, OutboundSendReason, PrototypeRelayServer, PrototypeRelaySubmitRequest,
    RelayQueueItemState, RelayQueueReason,
};
use mercury_policy::{
    RELAY_SUBMIT_ACCEPT, RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN, RELAY_SUBMIT_SEND_GATE_REJECTED,
};

#[test]
fn prototype_relay_enqueues_accepted_opaque_submission() {
    let mut server = PrototypeRelayServer::default();
    let outcome = server.submit(valid_request());

    let item = server
        .get_item(&route_id())
        .expect("accepted submission should be queued");

    assert!(outcome.submission.accepted);
    assert_eq!(outcome.submission.reason_code, RELAY_SUBMIT_ACCEPT);
    assert!(outcome.queue.accepted);
    assert_eq!(outcome.queue.next_state, RelayQueueItemState::Pending);
    assert_eq!(server.len(), 1);
    assert_eq!(item.state, RelayQueueItemState::Pending);
    assert_eq!(item.ciphertext_len(), 128);
    assert_eq!(item.sealed_header_len(), 96);
    assert!(item.has_ciphertext());
}

#[test]
fn prototype_relay_rejects_policy_failed_submissions_without_queueing() {
    let mut server = PrototypeRelayServer::default();

    let rejected_send = server.submit(request_with_send(rejected_send()));
    assert!(!rejected_send.submission.accepted);
    assert_eq!(
        rejected_send.submission.reason_code,
        RELAY_SUBMIT_SEND_GATE_REJECTED
    );
    assert!(!rejected_send.queue.accepted);
    assert_eq!(
        rejected_send.queue.reason,
        RelayQueueReason::RelaySubmissionRejected
    );
    assert!(server.is_empty());

    let plaintext_identity = server.submit(PrototypeRelaySubmitRequest::new(
        accepted_send(),
        &route_id(),
        &replay_token(),
        300,
        86400,
        &ciphertext(),
        1048576,
        &sealed_header(),
        1,
        3,
        100,
        120,
    ));
    assert!(!plaintext_identity.submission.accepted);
    assert_eq!(
        plaintext_identity.submission.reason_code,
        RELAY_SUBMIT_PLAINTEXT_IDENTITY_FORBIDDEN
    );
    assert!(server.is_empty());
}

#[test]
fn prototype_relay_rejects_duplicate_replay_tokens() {
    let mut server = PrototypeRelayServer::default();
    assert!(server.submit(valid_request()).queue.accepted);

    let duplicate = server.submit(PrototypeRelaySubmitRequest::new(
        accepted_send(),
        &[8; 32],
        &replay_token(),
        300,
        86400,
        &ciphertext(),
        1048576,
        &sealed_header(),
        0,
        3,
        100,
        120,
    ));

    assert!(!duplicate.queue.accepted);
    assert_eq!(
        duplicate.queue.reason,
        RelayQueueReason::ReplayTokenDuplicate
    );
    assert!(server.get_item(&[8; 32]).is_none());
    assert_eq!(server.len(), 1);
}

#[test]
fn prototype_relay_delivers_ciphertext_once_and_clears_stored_payload() {
    let mut server = PrototypeRelayServer::default();
    server.submit(valid_request());

    let delivery = server.deliver(&route_id(), 130);
    assert!(delivery.queue.accepted);
    assert_eq!(delivery.queue.next_state, RelayQueueItemState::Delivered);
    assert_eq!(delivery.ciphertext, ciphertext());
    assert_eq!(delivery.sealed_header, sealed_header());

    let item = server
        .get_item(&route_id())
        .expect("delivered queue item should retain metadata");
    assert_eq!(item.state, RelayQueueItemState::Delivered);
    assert_eq!(item.ciphertext_len(), 0);
    assert_eq!(item.sealed_header_len(), 0);

    let second_delivery = server.deliver(&route_id(), 131);
    assert!(!second_delivery.queue.accepted);
    assert_eq!(
        second_delivery.queue.reason,
        RelayQueueReason::ItemAlreadyDelivered
    );
    assert!(second_delivery.ciphertext.is_empty());
}

#[test]
fn prototype_relay_expire_and_delete_clear_payloads() {
    let mut server = PrototypeRelayServer::default();
    server.submit(valid_request());

    let too_early = server.expire(&route_id(), 399);
    assert!(!too_early.accepted);
    assert_eq!(too_early.reason, RelayQueueReason::ItemNotExpired);

    let expired = server.expire(&route_id(), 400);
    assert!(expired.accepted);
    assert_eq!(expired.next_state, RelayQueueItemState::Expired);
    let item = server
        .get_item(&route_id())
        .expect("expired item remains as metadata");
    assert!(!item.has_ciphertext());

    let deleted = server.delete(&route_id(), 401);
    assert!(deleted.accepted);
    assert_eq!(deleted.next_state, RelayQueueItemState::Deleted);
    let item = server
        .get_item(&route_id())
        .expect("deleted tombstone remains");
    assert_eq!(item.state, RelayQueueItemState::Deleted);
    assert!(!item.has_ciphertext());
}

#[test]
fn prototype_relay_route_is_reusable_for_an_ongoing_conversation() {
    // End-to-end shape of the conversation-killer fix: ONE route must carry message after
    // message. Submit msg1, deliver it (route -> Delivered tombstone), then submit msg2 to the
    // SAME route with a fresh replay token and a later window: the second submit must be ACCEPTED
    // (re-arming the slot to Pending) and be independently deliverable with ITS OWN payload.
    // Before the fix the second submit was rejected `ItemAlreadyDelivered` and the route stayed
    // dead until the 7-day reaper — i.e. no two-message conversation was possible at all.
    let mut server = PrototypeRelayServer::default();

    // msg1: submit + deliver.
    assert!(server.submit(valid_request()).queue.accepted);
    let d1 = server.deliver(&route_id(), 130);
    assert!(d1.queue.accepted);
    assert_eq!(d1.queue.next_state, RelayQueueItemState::Delivered);
    assert_eq!(d1.ciphertext, ciphertext());

    // msg2 on the SAME route (fresh replay token, distinct payload, later time window).
    let second_token = [11u8; 32];
    let second_ciphertext = [24u8; 128];
    let outcome2 = server.submit(PrototypeRelaySubmitRequest::new(
        accepted_send(),
        &ROUTE_ID,
        &second_token,
        300,
        86400,
        &second_ciphertext,
        1048576,
        &SEALED_HEADER,
        0,
        3,
        200,
        200,
    ));
    assert!(
        outcome2.queue.accepted,
        "a delivered route must accept the next message, got {:?}",
        outcome2.queue.reason
    );
    assert_eq!(outcome2.queue.next_state, RelayQueueItemState::Pending);

    // msg2 delivers independently, carrying its own ciphertext (not a stale msg1 payload).
    let d2 = server.deliver(&route_id(), 210);
    assert!(d2.queue.accepted);
    assert_eq!(d2.queue.next_state, RelayQueueItemState::Delivered);
    assert_eq!(d2.ciphertext, second_ciphertext.to_vec());
}

fn valid_request<'a>() -> PrototypeRelaySubmitRequest<'a> {
    request_with_send(accepted_send())
}

fn request_with_send(send: OutboundSendDecision) -> PrototypeRelaySubmitRequest<'static> {
    PrototypeRelaySubmitRequest::new(
        send,
        &ROUTE_ID,
        &REPLAY_TOKEN,
        300,
        86400,
        &CIPHERTEXT,
        1048576,
        &SEALED_HEADER,
        0,
        3,
        100,
        120,
    )
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

fn rejected_send() -> OutboundSendDecision {
    OutboundSendDecision {
        accepted: false,
        can_send: false,
        can_persist_ciphertext: false,
        requires_user_action: true,
        reason: OutboundSendReason::MessagePolicyRejected,
    }
}

fn route_id() -> Vec<u8> {
    ROUTE_ID.to_vec()
}

fn replay_token() -> Vec<u8> {
    REPLAY_TOKEN.to_vec()
}

fn ciphertext() -> Vec<u8> {
    CIPHERTEXT.to_vec()
}

fn sealed_header() -> Vec<u8> {
    SEALED_HEADER.to_vec()
}

const ROUTE_ID: [u8; 32] = [7; 32];
const REPLAY_TOKEN: [u8; 32] = [9; 32];
const CIPHERTEXT: [u8; 128] = [42; 128];
const SEALED_HEADER: [u8; 96] = [5; 96];
