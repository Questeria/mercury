use mercury_core::{ClientMessageEnvelope, ConversationPolicyState, MessageKind, ProtocolSuite};
use mercury_keys::DeviceKeyPair;
use mercury_message::{
    MessageError, PAD_BUCKET, SEALED_OVERHEAD, from_bytes, open_message, padded_len, seal_message,
    to_bytes,
};
use proptest::prelude::*;

/// A valid envelope mirroring the accepted human message in
/// `mercury-core/tests/client_message_policy.rs`. `payload_len` is a placeholder
/// (sealing recomputes it).
fn valid_envelope() -> ClientMessageEnvelope {
    ClientMessageEnvelope {
        version: 1,
        // Classical suite: these tests exercise the classical seal_message path
        // (the hybrid path + suite are covered in the lib's hybrid_message_tests).
        suite: ProtocolSuite::ClassicalDev,
        conversation_id_len: 32,
        sender_account_id_len: 32,
        sender_device_id_len: 32,
        epoch: 7,
        sequence: 42,
        kind: MessageKind::Application,
        payload_len: 0,
        critical_flags: 0,
        noncritical_flags: 0,
    }
}

fn valid_context() -> ConversationPolicyState {
    ConversationPolicyState {
        expected_epoch: 7,
        expected_sequence: 42,
        minimum_suite: ProtocolSuite::ClassicalDev,
        max_payload_len: 8192,
    }
}

#[test]
fn round_trip_recovers_plaintext_and_sets_payload_len() {
    let recipient = DeviceKeyPair::generate();
    let plaintext = b"hello mercury";

    let sealed = seal_message(
        &recipient.public(),
        plaintext,
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();

    assert_eq!(
        sealed.envelope.payload_len as usize,
        sealed.ciphertext.len()
    );
    // The sealed length reflects the PADDED plaintext (bucketed), not the raw length.
    assert_eq!(
        sealed.envelope.payload_len as usize,
        padded_len(plaintext.len()).unwrap() + SEALED_OVERHEAD
    );

    let wire = to_bytes(&sealed);
    let parsed = from_bytes(&wire).unwrap();
    assert_eq!(parsed, sealed);

    let recovered = open_message(&recipient, &parsed, &valid_context()).unwrap();
    assert_eq!(recovered, plaintext);
}

#[test]
fn policy_reject_produces_no_sealed_message() {
    let recipient = DeviceKeyPair::generate();
    let mut envelope = valid_envelope();
    envelope.epoch = 6; // mismatches context.expected_epoch (7) -> BAD_EPOCH

    let err = seal_message(&recipient.public(), b"x", envelope, &valid_context()).unwrap_err();
    assert!(matches!(err, MessageError::PolicyRejected(_)));
}

#[test]
fn policy_reject_on_suite_downgrade() {
    let recipient = DeviceKeyPair::generate();
    let mut envelope = valid_envelope();
    envelope.suite = ProtocolSuite::ClassicalDev; // 257 < minimum 513 -> DOWNGRADED_SUITE
    let mut context = valid_context();
    context.minimum_suite = ProtocolSuite::HybridPqDev;

    let err = seal_message(&recipient.public(), b"x", envelope, &context).unwrap_err();
    assert!(matches!(err, MessageError::PolicyRejected(_)));
}

#[test]
fn tampered_ciphertext_fails_open() {
    let recipient = DeviceKeyPair::generate();
    let mut sealed = seal_message(
        &recipient.public(),
        b"secret",
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();

    sealed.ciphertext[SEALED_OVERHEAD] ^= 0x01;

    let err = open_message(&recipient, &sealed, &valid_context()).unwrap_err();
    assert!(matches!(err, MessageError::Open(_)));
}

#[test]
fn wrong_recipient_cannot_open() {
    let recipient = DeviceKeyPair::generate();
    let other = DeviceKeyPair::generate();
    let sealed = seal_message(
        &recipient.public(),
        b"secret",
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();

    let err = open_message(&other, &sealed, &valid_context()).unwrap_err();
    assert!(matches!(err, MessageError::Open(_)));
}

#[test]
fn payload_len_mismatch_is_rejected() {
    let recipient = DeviceKeyPair::generate();
    let mut sealed = seal_message(
        &recipient.public(),
        b"secret",
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();

    sealed.envelope.payload_len += 1;

    let err = open_message(&recipient, &sealed, &valid_context()).unwrap_err();
    assert!(matches!(err, MessageError::PayloadLenMismatch));
}

#[test]
fn truncated_bytes_are_malformed() {
    let recipient = DeviceKeyPair::generate();
    let sealed = seal_message(
        &recipient.public(),
        b"secret",
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();
    let wire = to_bytes(&sealed);

    let err = from_bytes(&wire[..wire.len() - 1]).unwrap_err();
    assert!(matches!(err, MessageError::Malformed));
}

#[test]
fn trailing_bytes_are_malformed() {
    let recipient = DeviceKeyPair::generate();
    let sealed = seal_message(
        &recipient.public(),
        b"secret",
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();
    let mut wire = to_bytes(&sealed);
    wire.push(0u8);

    let err = from_bytes(&wire).unwrap_err();
    assert!(matches!(err, MessageError::Malformed));
}

#[test]
fn unknown_suite_code_is_unsupported() {
    let recipient = DeviceKeyPair::generate();
    let sealed = seal_message(
        &recipient.public(),
        b"secret",
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();
    let mut wire = to_bytes(&sealed);

    // Suite code is the 2nd i32 field (bytes 4..8); overwrite with a bad code.
    wire[4..8].copy_from_slice(&999i32.to_le_bytes());

    let err = from_bytes(&wire).unwrap_err();
    assert!(matches!(err, MessageError::UnsupportedSuite));
}

#[test]
fn unknown_kind_code_is_unsupported() {
    let recipient = DeviceKeyPair::generate();
    let sealed = seal_message(
        &recipient.public(),
        b"secret",
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();
    let mut wire = to_bytes(&sealed);

    // Kind code is the 8th i32 field (bytes 28..32); overwrite with a bad code.
    wire[28..32].copy_from_slice(&99i32.to_le_bytes());

    let err = from_bytes(&wire).unwrap_err();
    assert!(matches!(err, MessageError::UnsupportedKind));
}

#[test]
fn different_messages_in_the_same_bucket_seal_to_equal_lengths() {
    // The metadata-hardening property: two DIFFERENT plaintexts that fall in the
    // same padding bucket produce sealed objects of IDENTICAL length, so the wire
    // length leaks only the bucket, not the exact message size.
    let recipient = DeviceKeyPair::generate();
    // Both fit in the first 256-byte bucket (minus the 4-byte length prefix).
    let short = b"hi";
    let longer = vec![0x42u8; 200];
    let a = seal_message(
        &recipient.public(),
        short,
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();
    let b = seal_message(
        &recipient.public(),
        &longer,
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();
    assert_eq!(a.ciphertext.len(), b.ciphertext.len());
    assert_eq!(a.envelope.payload_len, b.envelope.payload_len);
    // And both round-trip back to their true (different) contents.
    assert_eq!(
        open_message(&recipient, &a, &valid_context()).unwrap(),
        short
    );
    assert_eq!(
        open_message(&recipient, &b, &valid_context()).unwrap(),
        longer
    );
}

#[test]
fn sealed_length_is_always_a_whole_number_of_buckets() {
    let recipient = DeviceKeyPair::generate();
    for len in [0usize, 1, 251, 252, 253, 255, 256, 257, 600] {
        let pt = vec![0x7eu8; len];
        let sealed =
            seal_message(&recipient.public(), &pt, valid_envelope(), &valid_context()).unwrap();
        let padded = sealed.ciphertext.len() - SEALED_OVERHEAD;
        assert_eq!(
            padded % PAD_BUCKET,
            0,
            "padded body must be a bucket multiple"
        );
        assert_eq!(padded, padded_len(len).unwrap());
        // Exactly reversible regardless of how much padding was added.
        assert_eq!(
            open_message(&recipient, &sealed, &valid_context()).unwrap(),
            pt
        );
    }
}

#[test]
fn padding_over_max_payload_len_fails_closed() {
    // A plaintext that fits under max_payload_len in its RAW form but whose padded +
    // sealed length rounds OVER the cap must be rejected by the policy gate (fail
    // closed) — never silently sent. Here max_payload_len is tightened so the padding
    // is what pushes it over.
    let recipient = DeviceKeyPair::generate();
    let mut context = valid_context();
    // 256-byte bucket + 104 overhead = 360 sealed; set the cap just below that.
    context.max_payload_len = 300;
    let err = seal_message(&recipient.public(), b"small", valid_envelope(), &context).unwrap_err();
    assert!(
        matches!(err, MessageError::PolicyRejected(_)),
        "padded-over-cap must fail closed, got {err:?}"
    );
}

#[test]
fn empty_plaintext_pads_and_round_trips() {
    let recipient = DeviceKeyPair::generate();
    let sealed =
        seal_message(&recipient.public(), b"", valid_envelope(), &valid_context()).unwrap();
    // Empty message still occupies a full first bucket (so even "no content" hides).
    assert_eq!(sealed.ciphertext.len(), PAD_BUCKET + SEALED_OVERHEAD);
    assert_eq!(
        open_message(&recipient, &sealed, &valid_context()).unwrap(),
        b""
    );
}

proptest! {
    #[test]
    fn from_bytes_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
        let _ = from_bytes(&bytes);
    }

    #[test]
    fn round_trip_arbitrary_plaintext(plaintext in proptest::collection::vec(any::<u8>(), 0..=4096)) {
        let recipient = DeviceKeyPair::generate();
        let sealed =
            seal_message(&recipient.public(), &plaintext, valid_envelope(), &valid_context())
                .unwrap();

        prop_assert_eq!(
            sealed.envelope.payload_len as usize,
            padded_len(plaintext.len()).unwrap() + SEALED_OVERHEAD
        );

        let wire = to_bytes(&sealed);
        let parsed = from_bytes(&wire).unwrap();
        prop_assert_eq!(&parsed, &sealed);

        let recovered = open_message(&recipient, &parsed, &valid_context()).unwrap();
        prop_assert_eq!(recovered, plaintext);
    }
}
