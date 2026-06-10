use mercury_core::{
    LocalStoreRecordKind, LocalStoreSealingDecision, LocalStoreSealingReason,
    MERCURY_MAX_MEDIA_OBJECT_BYTES, MediaObjectStoreInput, MediaObjectStoreReason,
    OutboundSendDecision, OutboundSendReason,
};

#[test]
fn encrypted_media_object_can_upload_without_plaintext_exposure() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_upload);
    assert!(decision.can_persist_local_ciphertext);
    assert!(!decision.requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, MediaObjectStoreReason::Accepted);
}

#[test]
fn plaintext_upload_and_auto_download_are_forbidden_before_media_service() {
    let mut plaintext = valid_input();
    plaintext.plaintext_bytes = 1;
    assert_rejected(
        plaintext.evaluate(),
        MediaObjectStoreReason::PlaintextUploadForbidden,
    );

    let mut auto_download = valid_input();
    auto_download.automatic_download_requested = true;
    assert_rejected(
        auto_download.evaluate(),
        MediaObjectStoreReason::AutomaticDownloadForbidden,
    );
}

#[test]
fn send_or_sealing_rejection_blocks_media_upload() {
    let mut rejected_send = valid_input();
    rejected_send.outbound_send = OutboundSendDecision {
        accepted: false,
        can_send: false,
        can_persist_ciphertext: false,
        requires_user_action: true,
        reason: OutboundSendReason::MessagePolicyRejected,
    };
    let send_decision = rejected_send.evaluate();
    assert!(!send_decision.accepted);
    assert_eq!(
        send_decision.reason,
        MediaObjectStoreReason::OutboundSendRejected
    );
    assert!(send_decision.requires_user_action);

    let mut rejected_seal = valid_input();
    rejected_seal.media_sealing = media_sealing(false, LocalStoreRecordKind::MediaCiphertext);
    assert_rejected(
        rejected_seal.evaluate(),
        MediaObjectStoreReason::MediaSealingRejected,
    );

    let mut wrong_kind = valid_input();
    wrong_kind.media_sealing = media_sealing(true, LocalStoreRecordKind::MessageCiphertext);
    assert_rejected(
        wrong_kind.evaluate(),
        MediaObjectStoreReason::MediaRecordKindMismatch,
    );
}

#[test]
fn media_object_metadata_is_opaque_and_bounded() {
    let mut bad_object_id = valid_input();
    bad_object_id.object_id_len = 16;
    assert_rejected(
        bad_object_id.evaluate(),
        MediaObjectStoreReason::BadObjectIdLength,
    );

    let mut empty_ciphertext = valid_input();
    empty_ciphertext.ciphertext_len = 0;
    assert_rejected(
        empty_ciphertext.evaluate(),
        MediaObjectStoreReason::BadCiphertextLength,
    );

    let mut too_large = valid_input();
    too_large.ciphertext_len = MERCURY_MAX_MEDIA_OBJECT_BYTES + 1;
    assert_rejected(
        too_large.evaluate(),
        MediaObjectStoreReason::CiphertextTooLarge,
    );

    let mut bad_header = valid_input();
    bad_header.sealed_header_len = 4097;
    assert_rejected(
        bad_header.evaluate(),
        MediaObjectStoreReason::BadSealedHeaderLength,
    );

    let mut bad_digest = valid_input();
    bad_digest.content_digest_len = 16;
    assert_rejected(
        bad_digest.evaluate(),
        MediaObjectStoreReason::BadContentDigestLength,
    );

    let mut bad_commitment = valid_input();
    bad_commitment.media_key_commitment_len = 24;
    assert_rejected(
        bad_commitment.evaluate(),
        MediaObjectStoreReason::BadMediaKeyCommitmentLength,
    );
}

#[test]
fn media_object_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MediaObjectStoreReason::Accepted, 0, "ACCEPTED"),
        (
            MediaObjectStoreReason::PlaintextUploadForbidden,
            1,
            "PLAINTEXT_UPLOAD_FORBIDDEN",
        ),
        (
            MediaObjectStoreReason::AutomaticDownloadForbidden,
            2,
            "AUTOMATIC_DOWNLOAD_FORBIDDEN",
        ),
        (
            MediaObjectStoreReason::OutboundSendRejected,
            3,
            "OUTBOUND_SEND_REJECTED",
        ),
        (
            MediaObjectStoreReason::MediaSealingRejected,
            4,
            "MEDIA_SEALING_REJECTED",
        ),
        (
            MediaObjectStoreReason::MediaRecordKindMismatch,
            5,
            "MEDIA_RECORD_KIND_MISMATCH",
        ),
        (
            MediaObjectStoreReason::BadObjectIdLength,
            6,
            "BAD_OBJECT_ID_LENGTH",
        ),
        (
            MediaObjectStoreReason::BadCiphertextLength,
            7,
            "BAD_CIPHERTEXT_LENGTH",
        ),
        (
            MediaObjectStoreReason::CiphertextTooLarge,
            8,
            "CIPHERTEXT_TOO_LARGE",
        ),
        (
            MediaObjectStoreReason::BadSealedHeaderLength,
            9,
            "BAD_SEALED_HEADER_LENGTH",
        ),
        (
            MediaObjectStoreReason::BadContentDigestLength,
            10,
            "BAD_CONTENT_DIGEST_LENGTH",
        ),
        (
            MediaObjectStoreReason::BadMediaKeyCommitmentLength,
            11,
            "BAD_MEDIA_KEY_COMMITMENT_LENGTH",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(
    decision: mercury_core::MediaObjectStoreDecision,
    reason: MediaObjectStoreReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_upload);
    assert!(!decision.can_persist_local_ciphertext);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}

fn valid_input() -> MediaObjectStoreInput {
    MediaObjectStoreInput {
        outbound_send: OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        },
        media_sealing: media_sealing(true, LocalStoreRecordKind::MediaCiphertext),
        object_id_len: 32,
        ciphertext_len: 4096,
        max_ciphertext_len: MERCURY_MAX_MEDIA_OBJECT_BYTES,
        sealed_header_len: 96,
        content_digest_len: 32,
        media_key_commitment_len: 32,
        plaintext_bytes: 0,
        automatic_download_requested: false,
    }
}

fn media_sealing(accepted: bool, record_kind: LocalStoreRecordKind) -> LocalStoreSealingDecision {
    LocalStoreSealingDecision {
        accepted,
        reason: if accepted {
            LocalStoreSealingReason::Accepted
        } else {
            LocalStoreSealingReason::PolicyDecisionRejected
        },
        record_policy: record_kind.policy(),
    }
}
