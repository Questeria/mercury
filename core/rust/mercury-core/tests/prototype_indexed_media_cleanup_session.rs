use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mercury_core::{
    LocalStoreRecordKind, LocalStoreRecordLocator, MediaObjectIndexInput,
    MediaObjectIndexStoreReason, MediaObjectIndexStoreWrite, MediaObjectLifecycleState,
    MediaRetentionInput, MediaRetentionOperation, MediaRetentionReason, MediaServiceAdapterKind,
    PrototypeFileMediaObjectIndexStore, PrototypeIndexedMediaCleanupSession,
    PrototypeIndexedMediaCleanupSessionEventKind, PrototypeIndexedMediaCleanupSessionInput,
    PrototypeIndexedMediaCleanupSessionReason, PrototypeMediaCleanupSessionInput,
    PrototypeMediaCleanupSessionReason,
};

#[test]
fn indexed_media_cleanup_runs_after_cleanable_manifest() {
    let mut session = PrototypeIndexedMediaCleanupSession::default();
    let outcome = session.run(valid_indexed_cleanup_input(
        valid_index_store_write(valid_media_object_index()),
        valid_cleanup_input(true),
    ));

    assert!(outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaCleanupSessionReason::Completed
    );
    assert!(outcome.index_store.accepted);
    assert!(outcome.index_store.index.can_cleanup);
    let cleanup = outcome.cleanup.expect("cleanup should run");
    assert!(cleanup.completed);
    assert_eq!(cleanup.remote_delete_calls, 1);
    assert!(cleanup.local_cache_delete_attempted);
    assert!(cleanup.local_cache_deleted);
    assert_eq!(outcome.index_store_records, 1);
    assert_eq!(session.index_store().len(), 1);
    assert_eq!(session.media_cleanup_session().remote_delete_calls(), 1);
    assert_eq!(session.events().len(), 4);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeIndexedMediaCleanupSessionEventKind::IndexedCleanupStarted
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeIndexedMediaCleanupSessionEventKind::IndexedCleanupFinished
    );
    assert!(session.events().last().expect("last event").terminal);
    assert!(!outcome.plaintext_exposed);
    assert!(
        session
            .events()
            .iter()
            .all(|event| !event.plaintext_bytes_exposed)
    );
}

#[test]
fn indexed_media_cleanup_can_use_external_media_index_adapter() {
    let root = temp_root("cleanup-adapter");
    let mut index_store = PrototypeFileMediaObjectIndexStore::new(root.clone());
    let mut session = PrototypeIndexedMediaCleanupSession::default();

    let outcome = session
        .run_with_index_store(
            &mut index_store,
            valid_indexed_cleanup_input(
                valid_index_store_write(valid_media_object_index()),
                valid_cleanup_input(true),
            ),
        )
        .expect("file media index adapter should not fail");

    assert!(outcome.completed);
    assert_eq!(outcome.index_store_records, 1);
    assert!(
        index_store
            .get(&MEDIA_OBJECT_ID)
            .expect("file media index read should succeed")
            .is_some()
    );
    assert!(session.index_store().is_empty());

    cleanup(root);
}

#[test]
fn indexed_media_cleanup_stops_when_manifest_store_rejects() {
    let mut index = valid_media_object_index();
    index.plaintext_metadata_bytes = 1;

    let mut session = PrototypeIndexedMediaCleanupSession::default();
    let outcome = session.run(valid_indexed_cleanup_input(
        valid_index_store_write(index),
        valid_cleanup_input(true),
    ));

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaCleanupSessionReason::MediaObjectIndexStoreRejected
    );
    assert_eq!(
        outcome.index_store.reason,
        MediaObjectIndexStoreReason::IndexRejected
    );
    assert!(outcome.cleanup.is_none());
    assert_eq!(outcome.index_store_records, 0);
    assert_eq!(session.media_cleanup_session().remote_delete_calls(), 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeIndexedMediaCleanupSessionEventKind::MediaObjectIndexStoreEvaluated
    );
}

#[test]
fn indexed_media_cleanup_stops_when_manifest_is_not_cleanable() {
    let mut index = valid_media_object_index();
    index.lifecycle_state = MediaObjectLifecycleState::Absent;
    index.local_cache_present = false;
    index.remote_object_present = false;
    index.remote_service_record_present = false;

    let mut session = PrototypeIndexedMediaCleanupSession::default();
    let outcome = session.run(valid_indexed_cleanup_input(
        valid_index_store_write(index),
        valid_cleanup_input(true),
    ));

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaCleanupSessionReason::MediaObjectNotCleanable
    );
    assert!(outcome.index_store.accepted);
    assert!(!outcome.index_store.index.can_cleanup);
    assert!(outcome.cleanup.is_none());
    assert_eq!(outcome.index_store_records, 1);
    assert_eq!(session.media_cleanup_session().remote_delete_calls(), 0);
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn indexed_media_cleanup_stops_when_cleanup_session_rejects() {
    let mut cleanup = valid_cleanup_input(true);
    cleanup.retention.retention_hold_active = true;

    let mut session = PrototypeIndexedMediaCleanupSession::default();
    let outcome = session.run(valid_indexed_cleanup_input(
        valid_index_store_write(valid_media_object_index()),
        cleanup,
    ));

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaCleanupSessionReason::MediaCleanupRejected
    );
    assert!(outcome.index_store.accepted);
    assert_eq!(outcome.index_store_records, 1);
    let cleanup = outcome.cleanup.expect("cleanup should be evaluated");
    assert!(!cleanup.completed);
    assert_eq!(
        cleanup.reason,
        PrototypeMediaCleanupSessionReason::MediaRetentionRejected
    );
    assert_eq!(
        cleanup.retention.reason,
        MediaRetentionReason::RetentionHoldActive
    );
    assert_eq!(session.media_cleanup_session().remote_delete_calls(), 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeIndexedMediaCleanupSessionEventKind::MediaCleanupSessionEvaluated
    );
}

#[test]
fn indexed_media_cleanup_reasons_and_events_have_stable_codes_and_labels() {
    let reasons = [
        (
            PrototypeIndexedMediaCleanupSessionReason::Completed,
            0,
            "completed",
        ),
        (
            PrototypeIndexedMediaCleanupSessionReason::MediaObjectIndexStoreRejected,
            1,
            "media_object_index_store_rejected",
        ),
        (
            PrototypeIndexedMediaCleanupSessionReason::MediaObjectNotCleanable,
            2,
            "media_object_not_cleanable",
        ),
        (
            PrototypeIndexedMediaCleanupSessionReason::MediaCleanupRejected,
            3,
            "media_cleanup_rejected",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let events = [
        (
            PrototypeIndexedMediaCleanupSessionEventKind::IndexedCleanupStarted,
            1,
            "indexed_cleanup_started",
        ),
        (
            PrototypeIndexedMediaCleanupSessionEventKind::MediaObjectIndexStoreEvaluated,
            2,
            "media_object_index_store_evaluated",
        ),
        (
            PrototypeIndexedMediaCleanupSessionEventKind::MediaCleanupSessionEvaluated,
            3,
            "media_cleanup_session_evaluated",
        ),
        (
            PrototypeIndexedMediaCleanupSessionEventKind::IndexedCleanupFinished,
            4,
            "indexed_cleanup_finished",
        ),
    ];

    for (event, code, label) in events {
        assert_eq!(event.code(), code);
        assert_eq!(event.label(), label);
    }
}

fn valid_indexed_cleanup_input<'a>(
    index_store: MediaObjectIndexStoreWrite<'a>,
    cleanup: PrototypeMediaCleanupSessionInput<'a>,
) -> PrototypeIndexedMediaCleanupSessionInput<'a> {
    PrototypeIndexedMediaCleanupSessionInput {
        index_store,
        cleanup,
    }
}

fn valid_index_store_write(index: MediaObjectIndexInput) -> MediaObjectIndexStoreWrite<'static> {
    MediaObjectIndexStoreWrite {
        object_id: &MEDIA_OBJECT_ID,
        content_digest: &MEDIA_CONTENT_DIGEST,
        media_key_commitment: &MEDIA_KEY_COMMITMENT,
        index,
    }
}

fn valid_media_object_index() -> MediaObjectIndexInput {
    MediaObjectIndexInput {
        lifecycle_state: MediaObjectLifecycleState::RemoteAndLocalCached,
        record_kind: LocalStoreRecordKind::MediaCiphertext,
        object_id_len: 32,
        content_digest_len: 32,
        media_key_commitment_len: 32,
        ciphertext_len: MEDIA_CIPHERTEXT.len() as i32,
        max_ciphertext_len: mercury_core::MERCURY_MAX_MEDIA_OBJECT_BYTES,
        plaintext_metadata_bytes: 0,
        content_digest_verified: true,
        local_cache_present: true,
        remote_object_present: true,
        remote_service_record_present: true,
        retention_hold_active: false,
    }
}

fn valid_cleanup_input(seed_local_cache: bool) -> PrototypeMediaCleanupSessionInput<'static> {
    PrototypeMediaCleanupSessionInput {
        retention: valid_retention_input(),
        cache_locator: locator(),
        cached_ciphertext: &MEDIA_CIPHERTEXT,
        seed_local_cache,
    }
}

fn valid_retention_input() -> MediaRetentionInput {
    MediaRetentionInput {
        operation: MediaRetentionOperation::DeleteRemoteAndEvictLocalCache,
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        record_kind: LocalStoreRecordKind::MediaCiphertext,
        service_authenticated: true,
        delete_authorized: true,
        object_namespace_bound: true,
        content_digest_verified: true,
        allow_development_adapter: false,
        user_delete_requested: true,
        cache_eviction_requested: false,
        retention_hold_active: false,
        object_id_len: 32,
        content_digest_len: 32,
        plaintext_bytes: 0,
    }
}

fn locator() -> LocalStoreRecordLocator<'static> {
    LocalStoreRecordLocator::new("conversation-7", "media-object-42")
}

const MEDIA_OBJECT_ID: [u8; 32] = [3; 32];
const MEDIA_CONTENT_DIGEST: [u8; 32] = [6; 32];
const MEDIA_KEY_COMMITMENT: [u8; 32] = [8; 32];
const MEDIA_CIPHERTEXT: [u8; 64] = [91; 64];

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "mercury-indexed-cleanup-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn cleanup(root: PathBuf) {
    let _ = fs::remove_dir_all(root);
}
