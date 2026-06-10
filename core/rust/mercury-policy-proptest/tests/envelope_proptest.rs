//! Property tests for `validate_envelope` and its sub-validators.
//!
//! The validators are pure total functions over scalar `i32` inputs, so these
//! properties exercise totality, the first-reject composition law, the
//! sub-validator agreement law, and the audit-class mapping.

use mercury_policy::{
    ACCEPT, AUDIT_ACCEPTED_MESSAGE, AUDIT_DOWNGRADE_ATTEMPT, AUDIT_POLICY_REJECT,
    AUDIT_SIZE_REJECT, EnvelopeInput, audit_class_for_reason, validate_content_v1,
    validate_envelope, validate_identity_v1, validate_order_v1,
};
use proptest::prelude::*;

/// Arbitrary envelope over the full `i32` range for every field.
fn any_envelope() -> impl Strategy<Value = EnvelopeInput> {
    (
        // proptest tuples cap at 12 elements, so nest the 15 fields.
        (
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
        ),
        (
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
            any::<i32>(),
        ),
    )
        .prop_map(
            |(
                (
                    version,
                    suite_id,
                    conversation_id_len,
                    sender_account_id_len,
                    sender_device_id_len,
                    epoch,
                    sequence,
                    message_kind,
                ),
                (
                    payload_len,
                    critical_flags,
                    noncritical_flags,
                    expected_epoch,
                    expected_sequence,
                    min_suite_id,
                    max_payload_len,
                ),
            )| EnvelopeInput {
                version,
                suite_id,
                conversation_id_len,
                sender_account_id_len,
                sender_device_id_len,
                epoch,
                sequence,
                message_kind,
                payload_len,
                critical_flags,
                noncritical_flags,
                expected_epoch,
                expected_sequence,
                min_suite_id,
                max_payload_len,
            },
        )
}

/// A constructed envelope that satisfies every sub-validator (so ACCEPTs).
fn valid_envelope() -> EnvelopeInput {
    EnvelopeInput {
        version: 1,
        suite_id: 513,
        conversation_id_len: 32,
        sender_account_id_len: 32,
        sender_device_id_len: 32,
        epoch: 5,
        sequence: 9,
        message_kind: 1,
        payload_len: 100,
        critical_flags: 0,
        noncritical_flags: 0,
        expected_epoch: 5,
        expected_sequence: 9,
        min_suite_id: 257,
        max_payload_len: 4096,
    }
}

fn identity_reason(input: &EnvelopeInput) -> i32 {
    validate_identity_v1(
        input.version,
        input.suite_id,
        input.min_suite_id,
        input.conversation_id_len,
        input.sender_account_id_len,
        input.sender_device_id_len,
    )
}

fn order_reason(input: &EnvelopeInput) -> i32 {
    validate_order_v1(
        input.epoch,
        input.sequence,
        input.expected_epoch,
        input.expected_sequence,
    )
}

fn content_reason(input: &EnvelopeInput) -> i32 {
    validate_content_v1(
        input.message_kind,
        input.payload_len,
        input.critical_flags,
        input.noncritical_flags,
        input.max_payload_len,
    )
}

proptest! {
    /// Property 1: total function — never panics over arbitrary `i32` fields.
    #[test]
    fn validate_envelope_never_panics(input in any_envelope()) {
        let _ = validate_envelope(input);
    }

    /// Property 2: envelope ACCEPTs iff all three sub-validators ACCEPT.
    #[test]
    fn accept_iff_all_subvalidators_accept(input in any_envelope()) {
        let all_accept = identity_reason(&input) == ACCEPT
            && order_reason(&input) == ACCEPT
            && content_reason(&input) == ACCEPT;
        prop_assert_eq!(validate_envelope(input) == ACCEPT, all_accept);
    }

    /// Property 3: first-reject priority — identity over order over content.
    #[test]
    fn first_reject_priority(input in any_envelope()) {
        let id = identity_reason(&input);
        let order = order_reason(&input);
        let content = content_reason(&input);
        let envelope = validate_envelope(input);
        if id != ACCEPT {
            prop_assert_eq!(envelope, id);
        } else if order != ACCEPT {
            prop_assert_eq!(envelope, order);
        } else {
            prop_assert_eq!(envelope, content);
        }
    }

    /// Property 4: audit class of an envelope reason is always a declared class.
    #[test]
    fn audit_class_always_valid(input in any_envelope()) {
        let class = audit_class_for_reason(validate_envelope(input));
        prop_assert!(matches!(
            class,
            AUDIT_ACCEPTED_MESSAGE
                | AUDIT_POLICY_REJECT
                | AUDIT_DOWNGRADE_ATTEMPT
                | AUDIT_SIZE_REJECT
        ));
    }

    /// Property 5: a constructed valid envelope always ACCEPTs (and so do all
    /// its sub-validators).
    #[test]
    fn constructed_valid_envelope_accepts(
        // Vary the free dimensions that must not affect acceptance.
        noncritical_flags in any::<i32>(),
        epoch in 1i32..1_000_000,
        sequence in 0i32..1_000_000,
        payload_len in 0i32..=4096,
        critical_flags in 0i32..=3,
    ) {
        let mut input = valid_envelope();
        input.noncritical_flags = noncritical_flags;
        input.epoch = epoch;
        input.expected_epoch = epoch;
        input.sequence = sequence;
        input.expected_sequence = sequence;
        input.payload_len = payload_len;
        input.critical_flags = critical_flags;
        prop_assert_eq!(validate_envelope(input), ACCEPT);
    }

    /// Property 6: flipping one field out of its valid range yields non-ACCEPT.
    #[test]
    fn one_field_out_of_range_rejects(field in 0u8..9) {
        let mut input = valid_envelope();
        match field {
            // identity dimensions
            0 => input.version = 2,                 // only 1 supported
            1 => input.suite_id = 42,               // unsupported suite
            2 => input.conversation_id_len = 7,     // below 8
            3 => input.sender_account_id_len = 200, // above 128
            4 => input.sender_device_id_len = 0,    // below 8
            // order dimensions (break the equality the validator requires)
            5 => input.expected_epoch = input.epoch + 1,
            // content dimensions
            6 => input.message_kind = 9,            // unknown kind
            7 => input.payload_len = input.max_payload_len + 1, // too large
            _ => input.critical_flags = 4,          // above 3
        }
        prop_assert_ne!(validate_envelope(input), ACCEPT);
    }
}
