use mercury_core::{
    LocalStoreRecordKind, MERCURY_MAX_MEDIA_OBJECT_BYTES, MediaObjectIndexDecision,
    MediaObjectIndexInput, MediaObjectIndexReason, MediaObjectLifecycleState,
};

#[test]
fn remote_and_local_manifest_can_download_and_cleanup_without_plaintext() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert_eq!(
        decision.lifecycle_state,
        MediaObjectLifecycleState::RemoteAndLocalCached
    );
    assert!(!decision.can_upload);
    assert!(decision.can_download);
    assert!(decision.can_cleanup);
    assert!(decision.has_local_cache);
    assert!(decision.has_remote_object);
    assert!(decision.keeps_audit_hash);
    assert!(!decision.requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, MediaObjectIndexReason::Accepted);
}

#[test]
fn absent_manifest_can_only_start_upload() {
    let mut input = valid_input();
    input.lifecycle_state = MediaObjectLifecycleState::Absent;
    input.local_cache_present = false;
    input.remote_object_present = false;
    input.remote_service_record_present = false;

    let decision = input.evaluate();

    assert!(decision.accepted);
    assert!(decision.can_upload);
    assert!(!decision.can_download);
    assert!(!decision.can_cleanup);
    assert!(!decision.has_local_cache);
    assert!(!decision.has_remote_object);
}

#[test]
fn deleted_manifest_is_terminal_and_not_reusable() {
    let mut input = valid_input();
    input.lifecycle_state = MediaObjectLifecycleState::Deleted;
    input.local_cache_present = false;
    input.remote_object_present = false;
    input.remote_service_record_present = false;

    let decision = input.evaluate();

    assert!(decision.accepted);
    assert!(decision.lifecycle_state.is_terminal());
    assert!(!decision.can_upload);
    assert!(!decision.can_download);
    assert!(!decision.can_cleanup);
    assert!(decision.keeps_audit_hash);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn delete_pending_requires_cleanup_and_blocks_download() {
    let mut input = valid_input();
    input.lifecycle_state = MediaObjectLifecycleState::DeletePending;
    input.local_cache_present = false;
    input.remote_object_present = true;
    input.remote_service_record_present = true;

    let decision = input.evaluate();
    assert!(decision.accepted);
    assert!(!decision.can_upload);
    assert!(!decision.can_download);
    assert!(decision.can_cleanup);

    input.retention_hold_active = true;
    let held = input.evaluate();
    assert!(held.accepted);
    assert!(!held.can_cleanup);
    assert!(held.requires_user_action);
}

#[test]
fn plaintext_wrong_kind_and_bad_metadata_are_rejected() {
    let mut plaintext = valid_input();
    plaintext.plaintext_metadata_bytes = 1;
    let plaintext_decision = plaintext.evaluate();
    assert_rejected(
        plaintext_decision,
        MediaObjectIndexReason::PlaintextMetadataForbidden,
    );
    assert!(plaintext_decision.requires_user_action);

    let mut wrong_kind = valid_input();
    wrong_kind.record_kind = LocalStoreRecordKind::MediaPlaintext;
    assert_rejected(
        wrong_kind.evaluate(),
        MediaObjectIndexReason::MediaRecordKindMismatch,
    );

    let mut bad_object = valid_input();
    bad_object.object_id_len = 16;
    assert_rejected(
        bad_object.evaluate(),
        MediaObjectIndexReason::BadObjectIdLength,
    );

    let mut bad_digest = valid_input();
    bad_digest.content_digest_len = 16;
    assert_rejected(
        bad_digest.evaluate(),
        MediaObjectIndexReason::BadContentDigestLength,
    );

    let mut bad_commitment = valid_input();
    bad_commitment.media_key_commitment_len = 16;
    assert_rejected(
        bad_commitment.evaluate(),
        MediaObjectIndexReason::BadMediaKeyCommitmentLength,
    );
}

#[test]
fn manifest_ciphertext_bounds_and_digest_verification_are_enforced() {
    let mut local_without_ciphertext = valid_input();
    local_without_ciphertext.ciphertext_len = 0;
    assert_rejected(
        local_without_ciphertext.evaluate(),
        MediaObjectIndexReason::LocalCacheWithoutCiphertext,
    );

    let mut bad_ciphertext = valid_input();
    bad_ciphertext.lifecycle_state = MediaObjectLifecycleState::RemoteStored;
    bad_ciphertext.local_cache_present = false;
    bad_ciphertext.ciphertext_len = 0;
    assert_rejected(
        bad_ciphertext.evaluate(),
        MediaObjectIndexReason::BadCiphertextLength,
    );

    let mut too_large = valid_input();
    too_large.ciphertext_len = MERCURY_MAX_MEDIA_OBJECT_BYTES + 1;
    assert_rejected(
        too_large.evaluate(),
        MediaObjectIndexReason::CiphertextTooLarge,
    );

    let mut digest_unverified = valid_input();
    digest_unverified.content_digest_verified = false;
    assert_rejected(
        digest_unverified.evaluate(),
        MediaObjectIndexReason::ContentDigestUnverified,
    );
}

#[test]
fn lifecycle_presence_and_remote_service_record_must_match() {
    let mut mismatched_lifecycle = valid_input();
    mismatched_lifecycle.lifecycle_state = MediaObjectLifecycleState::RemoteStored;
    mismatched_lifecycle.local_cache_present = true;
    mismatched_lifecycle.remote_object_present = true;
    assert_rejected(
        mismatched_lifecycle.evaluate(),
        MediaObjectIndexReason::BadLifecycleState,
    );

    let mut delete_pending_empty = valid_input();
    delete_pending_empty.lifecycle_state = MediaObjectLifecycleState::DeletePending;
    delete_pending_empty.local_cache_present = false;
    delete_pending_empty.remote_object_present = false;
    delete_pending_empty.remote_service_record_present = false;
    assert_rejected(
        delete_pending_empty.evaluate(),
        MediaObjectIndexReason::BadLifecycleState,
    );

    let mut missing_remote_record = valid_input();
    missing_remote_record.remote_service_record_present = false;
    assert_rejected(
        missing_remote_record.evaluate(),
        MediaObjectIndexReason::RemoteWithoutServiceRecord,
    );
}

#[test]
fn media_object_index_states_and_reasons_have_stable_codes_and_labels() {
    let states = [
        (MediaObjectLifecycleState::Absent, 0, "absent"),
        (MediaObjectLifecycleState::LocalCached, 1, "local_cached"),
        (MediaObjectLifecycleState::RemoteStored, 2, "remote_stored"),
        (
            MediaObjectLifecycleState::RemoteAndLocalCached,
            3,
            "remote_and_local_cached",
        ),
        (
            MediaObjectLifecycleState::DeletePending,
            4,
            "delete_pending",
        ),
        (MediaObjectLifecycleState::Deleted, 5, "deleted"),
    ];

    for (state, code, label) in states {
        assert_eq!(state.code(), code);
        assert_eq!(state.label(), label);
    }

    let reasons = [
        (MediaObjectIndexReason::Accepted, 0, "ACCEPTED"),
        (
            MediaObjectIndexReason::PlaintextMetadataForbidden,
            1,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            MediaObjectIndexReason::MediaRecordKindMismatch,
            2,
            "MEDIA_RECORD_KIND_MISMATCH",
        ),
        (
            MediaObjectIndexReason::BadObjectIdLength,
            3,
            "BAD_OBJECT_ID_LENGTH",
        ),
        (
            MediaObjectIndexReason::BadContentDigestLength,
            4,
            "BAD_CONTENT_DIGEST_LENGTH",
        ),
        (
            MediaObjectIndexReason::BadMediaKeyCommitmentLength,
            5,
            "BAD_MEDIA_KEY_COMMITMENT_LENGTH",
        ),
        (
            MediaObjectIndexReason::BadCiphertextLength,
            6,
            "BAD_CIPHERTEXT_LENGTH",
        ),
        (
            MediaObjectIndexReason::CiphertextTooLarge,
            7,
            "CIPHERTEXT_TOO_LARGE",
        ),
        (
            MediaObjectIndexReason::ContentDigestUnverified,
            8,
            "CONTENT_DIGEST_UNVERIFIED",
        ),
        (
            MediaObjectIndexReason::LocalCacheWithoutCiphertext,
            9,
            "LOCAL_CACHE_WITHOUT_CIPHERTEXT",
        ),
        (
            MediaObjectIndexReason::RemoteWithoutServiceRecord,
            10,
            "REMOTE_WITHOUT_SERVICE_RECORD",
        ),
        (
            MediaObjectIndexReason::BadLifecycleState,
            11,
            "BAD_LIFECYCLE_STATE",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(decision: MediaObjectIndexDecision, reason: MediaObjectIndexReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_upload);
    assert!(!decision.can_download);
    assert!(!decision.can_cleanup);
    assert!(!decision.has_local_cache);
    assert!(!decision.has_remote_object);
    assert!(decision.keeps_audit_hash);
    assert!(!decision.plaintext_bytes_exposed);
    assert_eq!(decision.reason, reason);
}

fn valid_input() -> MediaObjectIndexInput {
    MediaObjectIndexInput {
        lifecycle_state: MediaObjectLifecycleState::RemoteAndLocalCached,
        record_kind: LocalStoreRecordKind::MediaCiphertext,
        object_id_len: 32,
        content_digest_len: 32,
        media_key_commitment_len: 32,
        ciphertext_len: 4096,
        max_ciphertext_len: MERCURY_MAX_MEDIA_OBJECT_BYTES,
        plaintext_metadata_bytes: 0,
        content_digest_verified: true,
        local_cache_present: true,
        remote_object_present: true,
        remote_service_record_present: true,
        retention_hold_active: false,
    }
}
