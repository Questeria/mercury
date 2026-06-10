use std::{
    collections::BTreeMap,
    convert::Infallible,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mercury_core::{
    AcceptedMediaObjectIndexStoreWrite, LocalStoreRecordKind, MERCURY_MAX_MEDIA_OBJECT_BYTES,
    MediaObjectIndexInput, MediaObjectIndexReason, MediaObjectIndexRecord,
    MediaObjectIndexStoreAdapter, MediaObjectIndexStoreReason, MediaObjectIndexStoreWrite,
    MediaObjectLifecycleState, PrototypeFileMediaObjectIndexStore, PrototypeMediaObjectIndexStore,
    delete_media_object_index_record, put_media_object_index_record,
    read_media_object_index_record,
};

const OBJECT_ID: [u8; 32] = [53; 32];
const CONTENT_DIGEST: [u8; 32] = [59; 32];
const MEDIA_KEY_COMMITMENT: [u8; 32] = [61; 32];

#[test]
fn adapter_writes_only_after_media_index_gate_accepts() {
    let mut store = RecordingMediaIndexStore::default();

    let accepted = put_media_object_index_record(&mut store, valid_write(valid_index()))
        .expect("accepted media index write should not fail");
    assert!(accepted.accepted);
    assert_eq!(accepted.reason, MediaObjectIndexStoreReason::Accepted);
    assert_eq!(accepted.record_count, 1);
    assert_eq!(store.put_calls, 1);

    let mut rejected_index = valid_index();
    rejected_index.plaintext_metadata_bytes = 1;
    let rejected = put_media_object_index_record(&mut store, valid_write(rejected_index))
        .expect("rejected media index write should not fail");

    assert!(!rejected.accepted);
    assert_eq!(rejected.reason, MediaObjectIndexStoreReason::IndexRejected);
    assert_eq!(
        rejected.index.reason,
        MediaObjectIndexReason::PlaintextMetadataForbidden
    );
    assert_eq!(rejected.record_count, 1);
    assert_eq!(store.put_calls, 1);
}

#[test]
fn adapter_read_and_delete_use_opaque_object_ids() {
    let mut store = RecordingMediaIndexStore::default();

    put_media_object_index_record(&mut store, valid_write(valid_index()))
        .expect("accepted media index write should not fail");

    let read = read_media_object_index_record(&store, &OBJECT_ID)
        .expect("media index read should not fail")
        .expect("media index record should exist");
    assert_eq!(read.object_id, OBJECT_ID.to_vec());
    assert_eq!(read.content_digest, CONTENT_DIGEST.to_vec());
    assert!(!read.plaintext_bytes_exposed);

    delete_media_object_index_record(&mut store, &OBJECT_ID)
        .expect("media index delete should not fail");
    assert!(
        read_media_object_index_record(&store, &OBJECT_ID)
            .expect("media index lookup should not fail")
            .is_none()
    );
}

#[test]
fn prototype_stores_satisfy_media_index_adapter_boundary() {
    let mut memory_store = PrototypeMediaObjectIndexStore::default();
    let memory_decision =
        put_media_object_index_record(&mut memory_store, valid_write(valid_index()))
            .expect("prototype memory media index write should not fail");
    assert!(memory_decision.accepted);
    assert!(
        read_media_object_index_record(&memory_store, &OBJECT_ID)
            .expect("prototype memory read should not fail")
            .is_some()
    );

    let root = temp_root("file-adapter");
    let mut file_store = PrototypeFileMediaObjectIndexStore::new(root.clone());
    let file_decision = put_media_object_index_record(&mut file_store, valid_write(valid_index()))
        .expect("prototype file media index write should not fail");
    assert!(file_decision.accepted);
    assert_eq!(file_decision.record_count, 1);
    assert!(
        read_media_object_index_record(&file_store, &OBJECT_ID)
            .expect("prototype file read should not fail")
            .is_some()
    );

    cleanup(root);
}

#[derive(Default)]
struct RecordingMediaIndexStore {
    put_calls: usize,
    records: BTreeMap<Vec<u8>, MediaObjectIndexRecord>,
}

impl MediaObjectIndexStoreAdapter for RecordingMediaIndexStore {
    type Error = Infallible;

    fn put_accepted_media_object_index(
        &mut self,
        write: AcceptedMediaObjectIndexStoreWrite<'_>,
    ) -> Result<(), Self::Error> {
        self.put_calls += 1;
        let input = write.write();
        let index = write.decision().index;
        let record = MediaObjectIndexRecord {
            object_id: input.object_id.to_vec(),
            content_digest: input.content_digest.to_vec(),
            media_key_commitment: input.media_key_commitment.to_vec(),
            lifecycle_state: index.lifecycle_state,
            ciphertext_len: input.index.ciphertext_len,
            has_local_cache: index.has_local_cache,
            has_remote_object: index.has_remote_object,
            content_digest_verified: input.index.content_digest_verified,
            retention_hold_active: input.index.retention_hold_active,
            plaintext_bytes_exposed: false,
        };
        self.records.insert(record.object_id.clone(), record);
        Ok(())
    }

    fn read_media_object_index(
        &self,
        object_id: &[u8],
    ) -> Result<Option<MediaObjectIndexRecord>, Self::Error> {
        Ok(self.records.get(object_id).cloned())
    }

    fn delete_media_object_index(&mut self, object_id: &[u8]) -> Result<(), Self::Error> {
        self.records.remove(object_id);
        Ok(())
    }

    fn media_object_index_record_count(&self) -> Result<usize, Self::Error> {
        Ok(self.records.len())
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

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "mercury-media-index-adapter-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn cleanup(root: PathBuf) {
    let _ = fs::remove_dir_all(root);
}
