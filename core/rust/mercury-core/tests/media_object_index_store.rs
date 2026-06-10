use mercury_core::{
    LocalStoreRecordKind, MERCURY_MAX_MEDIA_OBJECT_BYTES, MediaObjectIndexInput,
    MediaObjectIndexReason, MediaObjectIndexStoreReason, MediaObjectIndexStoreWrite,
    MediaObjectLifecycleState, PrototypeMediaObjectIndexStore,
};

const OBJECT_ID: [u8; 32] = [17; 32];
const CONTENT_DIGEST: [u8; 32] = [23; 32];
const MEDIA_KEY_COMMITMENT: [u8; 32] = [31; 32];

#[test]
fn store_persists_accepted_manifest_without_plaintext_exposure() {
    let mut store = PrototypeMediaObjectIndexStore::default();
    let decision = store.write(valid_write(valid_index()));

    assert!(decision.accepted);
    assert_eq!(decision.reason, MediaObjectIndexStoreReason::Accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert!(decision.keeps_audit_hash);
    assert!(!decision.plaintext_bytes_exposed);
    assert!(decision.index.can_download);
    assert!(decision.index.can_cleanup);

    let record = store.get(&OBJECT_ID).expect("record should be persisted");
    assert_eq!(
        record.lifecycle_state,
        MediaObjectLifecycleState::RemoteAndLocalCached
    );
    assert_eq!(record.object_id, OBJECT_ID.to_vec());
    assert_eq!(record.content_digest, CONTENT_DIGEST.to_vec());
    assert_eq!(record.media_key_commitment, MEDIA_KEY_COMMITMENT.to_vec());
    assert_eq!(record.ciphertext_len, 4096);
    assert!(record.has_local_cache);
    assert!(record.has_remote_object);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn store_rejects_unaccepted_index_without_writing() {
    let mut store = PrototypeMediaObjectIndexStore::default();
    let mut index = valid_index();
    index.plaintext_metadata_bytes = 1;

    let decision = store.write(valid_write(index));

    assert!(!decision.accepted);
    assert_eq!(decision.reason, MediaObjectIndexStoreReason::IndexRejected);
    assert_eq!(
        decision.index.reason,
        MediaObjectIndexReason::PlaintextMetadataForbidden
    );
    assert!(!decision.persisted_record);
    assert_eq!(decision.record_count, 0);
    assert!(store.is_empty());
}

#[test]
fn store_validates_opaque_bytes_before_writing() {
    let mut store = PrototypeMediaObjectIndexStore::default();
    let mut index = valid_index();
    index.object_id_len = 32;
    let bad_object_id = store.write(MediaObjectIndexStoreWrite {
        object_id: &[1; 16],
        content_digest: &CONTENT_DIGEST,
        media_key_commitment: &MEDIA_KEY_COMMITMENT,
        index,
    });

    assert!(!bad_object_id.accepted);
    assert_eq!(
        bad_object_id.reason,
        MediaObjectIndexStoreReason::BadObjectIdLength
    );
    assert!(!bad_object_id.persisted_record);

    let bad_digest = store.write(MediaObjectIndexStoreWrite {
        object_id: &OBJECT_ID,
        content_digest: &[2; 16],
        media_key_commitment: &MEDIA_KEY_COMMITMENT,
        index: valid_index(),
    });

    assert!(!bad_digest.accepted);
    assert_eq!(
        bad_digest.reason,
        MediaObjectIndexStoreReason::BadContentDigestLength
    );

    let bad_commitment = store.write(MediaObjectIndexStoreWrite {
        object_id: &OBJECT_ID,
        content_digest: &CONTENT_DIGEST,
        media_key_commitment: &[3; 16],
        index: valid_index(),
    });

    assert!(!bad_commitment.accepted);
    assert_eq!(
        bad_commitment.reason,
        MediaObjectIndexStoreReason::BadMediaKeyCommitmentLength
    );
    assert!(store.is_empty());
}

#[test]
fn store_updates_lifecycle_snapshot_for_same_object() {
    let mut store = PrototypeMediaObjectIndexStore::default();
    let mut local = valid_index();
    local.lifecycle_state = MediaObjectLifecycleState::LocalCached;
    local.remote_object_present = false;
    local.remote_service_record_present = false;

    assert!(store.write(valid_write(local)).accepted);
    assert_eq!(store.len(), 1);
    assert_eq!(
        store.get(&OBJECT_ID).expect("local record").lifecycle_state,
        MediaObjectLifecycleState::LocalCached
    );

    let remote_and_local = valid_index();
    assert!(store.write(valid_write(remote_and_local)).accepted);
    assert_eq!(store.len(), 1);
    let record = store.get(&OBJECT_ID).expect("updated record");
    assert_eq!(
        record.lifecycle_state,
        MediaObjectLifecycleState::RemoteAndLocalCached
    );
    assert!(record.has_local_cache);
    assert!(record.has_remote_object);
}

#[test]
fn store_can_persist_terminal_deleted_audit_snapshot() {
    let mut store = PrototypeMediaObjectIndexStore::default();
    let mut deleted = valid_index();
    deleted.lifecycle_state = MediaObjectLifecycleState::Deleted;
    deleted.local_cache_present = false;
    deleted.remote_object_present = false;
    deleted.remote_service_record_present = false;

    let decision = store.write(valid_write(deleted));

    assert!(decision.accepted);
    assert!(!decision.index.can_upload);
    assert!(!decision.index.can_download);
    assert!(!decision.index.can_cleanup);
    assert!(decision.keeps_audit_hash);
    let record = store.get(&OBJECT_ID).expect("deleted record");
    assert_eq!(record.lifecycle_state, MediaObjectLifecycleState::Deleted);
    assert!(!record.has_local_cache);
    assert!(!record.has_remote_object);
}

#[test]
fn store_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MediaObjectIndexStoreReason::Accepted, 0, "ACCEPTED"),
        (
            MediaObjectIndexStoreReason::BadObjectIdLength,
            1,
            "BAD_OBJECT_ID_LENGTH",
        ),
        (
            MediaObjectIndexStoreReason::BadContentDigestLength,
            2,
            "BAD_CONTENT_DIGEST_LENGTH",
        ),
        (
            MediaObjectIndexStoreReason::BadMediaKeyCommitmentLength,
            3,
            "BAD_MEDIA_KEY_COMMITMENT_LENGTH",
        ),
        (
            MediaObjectIndexStoreReason::IndexRejected,
            4,
            "INDEX_REJECTED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_write(index: MediaObjectIndexInput) -> MediaObjectIndexStoreWrite<'static> {
    MediaObjectIndexStoreWrite {
        object_id: &OBJECT_ID,
        content_digest: &CONTENT_DIGEST,
        media_key_commitment: &MEDIA_KEY_COMMITMENT,
        index,
    }
}

fn valid_index() -> MediaObjectIndexInput {
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
