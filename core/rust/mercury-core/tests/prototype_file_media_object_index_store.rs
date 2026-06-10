use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mercury_core::{
    LocalStoreRecordKind, MERCURY_MAX_MEDIA_OBJECT_BYTES, MediaObjectIndexInput,
    MediaObjectIndexReason, MediaObjectIndexStoreReason, MediaObjectIndexStoreWrite,
    MediaObjectLifecycleState, PrototypeFileMediaObjectIndexStore,
};

const OBJECT_ID: [u8; 32] = [41; 32];
const CONTENT_DIGEST: [u8; 32] = [43; 32];
const MEDIA_KEY_COMMITMENT: [u8; 32] = [47; 32];

#[test]
fn file_media_index_persists_accepted_manifest_across_reopen() {
    let root = temp_root("persist");
    let path;

    {
        let mut store = PrototypeFileMediaObjectIndexStore::new(root.clone());
        path = store.record_path(&OBJECT_ID);

        let decision = store
            .write(valid_write(valid_index()))
            .expect("file media index write should succeed");

        assert!(decision.accepted);
        assert_eq!(decision.reason, MediaObjectIndexStoreReason::Accepted);
        assert!(decision.persisted_record);
        assert_eq!(decision.record_count, 1);
        assert!(decision.keeps_audit_hash);
        assert!(!decision.plaintext_bytes_exposed);
        assert!(path.exists());
    }

    let reopened = PrototypeFileMediaObjectIndexStore::new(root.clone());
    let record = reopened
        .get(&OBJECT_ID)
        .expect("file media index read should succeed")
        .expect("accepted media index should exist after reopen");

    assert_eq!(record.object_id, OBJECT_ID.to_vec());
    assert_eq!(record.content_digest, CONTENT_DIGEST.to_vec());
    assert_eq!(record.media_key_commitment, MEDIA_KEY_COMMITMENT.to_vec());
    assert_eq!(
        record.lifecycle_state,
        MediaObjectLifecycleState::RemoteAndLocalCached
    );
    assert_eq!(record.ciphertext_len, 4096);
    assert!(record.has_local_cache);
    assert!(record.has_remote_object);
    assert!(record.content_digest_verified);
    assert!(!record.retention_hold_active);
    assert!(!record.plaintext_bytes_exposed);
    assert_eq!(reopened.len().expect("len should succeed"), 1);

    cleanup(root);
}

#[test]
fn file_media_index_rejected_plaintext_metadata_creates_no_file() {
    let root = temp_root("reject");
    let mut store = PrototypeFileMediaObjectIndexStore::new(root.clone());
    let path = store.record_path(&OBJECT_ID);
    let mut index = valid_index();
    index.plaintext_metadata_bytes = 1;

    let decision = store
        .write(valid_write(index))
        .expect("rejected media index write should return decision");

    assert!(!decision.accepted);
    assert_eq!(decision.reason, MediaObjectIndexStoreReason::IndexRejected);
    assert_eq!(
        decision.index.reason,
        MediaObjectIndexReason::PlaintextMetadataForbidden
    );
    assert!(!decision.persisted_record);
    assert_eq!(decision.record_count, 0);
    assert!(!path.exists());
    assert!(
        store
            .get(&OBJECT_ID)
            .expect("missing media index lookup should succeed")
            .is_none()
    );

    cleanup(root);
}

#[test]
fn file_media_index_rejected_replacement_preserves_existing_record() {
    let root = temp_root("replace");
    let mut store = PrototypeFileMediaObjectIndexStore::new(root.clone());

    let accepted = store
        .write(valid_write(valid_index()))
        .expect("accepted media index write should succeed");
    assert!(accepted.accepted);

    let mut rejected_index = valid_index();
    rejected_index.content_digest_verified = false;
    let rejected = store
        .write(valid_write(rejected_index))
        .expect("rejected media index replacement should return decision");
    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, MediaObjectIndexStoreReason::IndexRejected);
    assert_eq!(
        rejected.index.reason,
        MediaObjectIndexReason::ContentDigestUnverified
    );

    let record = store
        .get(&OBJECT_ID)
        .expect("existing media index should be readable")
        .expect("original media index should remain");
    assert_eq!(record.ciphertext_len, 4096);
    assert!(record.content_digest_verified);
    assert_eq!(store.len().expect("len should succeed"), 1);

    cleanup(root);
}

#[test]
fn file_media_index_delete_removes_durable_manifest() {
    let root = temp_root("delete");
    let mut store = PrototypeFileMediaObjectIndexStore::new(root.clone());

    assert!(
        store
            .write(valid_write(valid_index()))
            .expect("accepted media index write should succeed")
            .accepted
    );
    assert!(
        store
            .get(&OBJECT_ID)
            .expect("media index should be readable")
            .is_some()
    );
    assert!(store.delete(&OBJECT_ID).expect("delete should succeed"));
    assert!(
        !store
            .delete(&OBJECT_ID)
            .expect("second delete should succeed")
    );
    assert!(
        store
            .get(&OBJECT_ID)
            .expect("deleted media index lookup should succeed")
            .is_none()
    );
    assert!(store.is_empty().expect("empty check should succeed"));

    cleanup(root);
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

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "mercury-file-media-index-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn cleanup(root: PathBuf) {
    let _ = fs::remove_dir_all(root);
}
