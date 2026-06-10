use mercury_core::{
    RelayQueueDecision, RelayQueueInput, RelayQueueItemState, RelayQueueOperation,
    RelayQueueReason, RelaySubmissionDecision, evaluate_relay_queue,
};

#[test]
fn enqueue_accepts_valid_relay_submission_once() {
    let decision = evaluate_relay_queue(input(RelayQueueOperation::Enqueue));

    assert!(decision.accepted);
    assert_eq!(decision.next_state, RelayQueueItemState::Pending);
    assert!(decision.persist_item);
    assert!(!decision.delete_item);
    assert_eq!(decision.reason, RelayQueueReason::Accepted);
}

#[test]
fn enqueue_rejects_rejected_submission_and_duplicate_replay_token() {
    let mut rejected = input(RelayQueueOperation::Enqueue);
    rejected.submission = relay_submission(false);
    let decision = evaluate_relay_queue(rejected);
    assert_rejected(
        decision,
        RelayQueueItemState::Absent,
        RelayQueueReason::RelaySubmissionRejected,
    );

    let mut duplicate = input(RelayQueueOperation::Enqueue);
    duplicate.replay_token_seen = true;
    let decision = evaluate_relay_queue(duplicate);
    assert_rejected(
        decision,
        RelayQueueItemState::Absent,
        RelayQueueReason::ReplayTokenDuplicate,
    );
}

#[test]
fn enqueue_rejects_existing_queue_item_states() {
    let mut pending = input(RelayQueueOperation::Enqueue);
    pending.state = RelayQueueItemState::Pending;

    let decision = evaluate_relay_queue(pending);

    assert_rejected(
        decision,
        RelayQueueItemState::Pending,
        RelayQueueReason::ItemAlreadyPending,
    );
}

#[test]
fn enqueue_reuses_a_route_whose_previous_item_is_terminal() {
    // Regression for the conversation-killer: a route must be REUSABLE. Once the previous message
    // is terminal (the recipient received it = Delivered, it timed out = Expired, or it was
    // acked/deleted = Deleted), the next enqueue is accepted and re-arms the slot as Pending. Only
    // a still-`Pending` item blocks (single-slot backpressure). Before the fix EVERY non-Absent
    // state rejected the enqueue, and terminal records are reaped only TERMINAL_RETENTION_S (7
    // days) after expiry — so a route went dead for a week after its first delivered message,
    // breaking every conversation past the first round-trip.
    for terminal in [
        RelayQueueItemState::Delivered,
        RelayQueueItemState::Expired,
        RelayQueueItemState::Deleted,
    ] {
        let mut reuse = input(RelayQueueOperation::Enqueue);
        reuse.state = terminal;

        let decision = evaluate_relay_queue(reuse);

        assert!(
            decision.accepted,
            "enqueue over terminal state {terminal:?} must re-arm the route for the next message"
        );
        assert_eq!(decision.next_state, RelayQueueItemState::Pending);
        assert!(decision.persist_item);
        assert!(!decision.delete_item);
        assert_eq!(decision.reason, RelayQueueReason::Accepted);
    }
}

#[test]
fn deliver_pending_item_deletes_ciphertext_after_delivery() {
    let decision = evaluate_relay_queue(input(RelayQueueOperation::Deliver));

    assert!(decision.accepted);
    assert_eq!(decision.next_state, RelayQueueItemState::Delivered);
    assert!(!decision.persist_item);
    assert!(decision.delete_item);
}

#[test]
fn deliver_rejects_expired_or_terminal_items() {
    let mut expired_now = input(RelayQueueOperation::Deliver);
    expired_now.now_s = expired_now.expires_at_s;
    let decision = evaluate_relay_queue(expired_now);
    assert_rejected(
        decision,
        RelayQueueItemState::Pending,
        RelayQueueReason::ItemExpired,
    );

    let mut delivered = input(RelayQueueOperation::Deliver);
    delivered.state = RelayQueueItemState::Delivered;
    let decision = evaluate_relay_queue(delivered);
    assert_rejected(
        decision,
        RelayQueueItemState::Delivered,
        RelayQueueReason::ItemAlreadyDelivered,
    );
}

#[test]
fn expiry_only_accepts_after_deadline_and_deletes_ciphertext() {
    let too_early = evaluate_relay_queue(input(RelayQueueOperation::Expire));
    assert_rejected(
        too_early,
        RelayQueueItemState::Pending,
        RelayQueueReason::ItemNotExpired,
    );

    let mut due = input(RelayQueueOperation::Expire);
    due.now_s = due.expires_at_s;
    let decision = evaluate_relay_queue(due);

    assert!(decision.accepted);
    assert_eq!(decision.next_state, RelayQueueItemState::Expired);
    assert!(!decision.persist_item);
    assert!(decision.delete_item);
}

#[test]
fn delete_is_idempotence_guarded() {
    let decision = evaluate_relay_queue(input(RelayQueueOperation::Delete));

    assert!(decision.accepted);
    assert_eq!(decision.next_state, RelayQueueItemState::Deleted);
    assert!(decision.delete_item);

    let mut already_deleted = input(RelayQueueOperation::Delete);
    already_deleted.state = RelayQueueItemState::Deleted;
    let decision = evaluate_relay_queue(already_deleted);
    assert_rejected(
        decision,
        RelayQueueItemState::Deleted,
        RelayQueueReason::ItemAlreadyDeleted,
    );
}

#[test]
fn bad_time_window_blocks_every_operation() {
    let mut bad = input(RelayQueueOperation::Enqueue);
    bad.expires_at_s = bad.created_at_s;

    let decision = evaluate_relay_queue(bad);

    assert_rejected(
        decision,
        RelayQueueItemState::Absent,
        RelayQueueReason::BadTimeWindow,
    );
}

fn input(operation: RelayQueueOperation) -> RelayQueueInput {
    let state = match operation {
        RelayQueueOperation::Enqueue => RelayQueueItemState::Absent,
        RelayQueueOperation::Deliver
        | RelayQueueOperation::Expire
        | RelayQueueOperation::Delete => RelayQueueItemState::Pending,
    };

    RelayQueueInput {
        operation,
        submission: relay_submission(true),
        state,
        replay_token_seen: false,
        created_at_s: 100,
        expires_at_s: 400,
        now_s: 120,
    }
}

fn relay_submission(accepted: bool) -> RelaySubmissionDecision {
    RelaySubmissionDecision {
        accepted,
        reason_code: if accepted { 0 } else { 3 },
        audit_class: if accepted { 1 } else { 3 },
    }
}

fn assert_rejected(
    decision: RelayQueueDecision,
    next_state: RelayQueueItemState,
    reason: RelayQueueReason,
) {
    assert!(!decision.accepted);
    assert_eq!(decision.next_state, next_state);
    assert!(!decision.persist_item);
    assert!(!decision.delete_item);
    assert_eq!(decision.reason, reason);
}
