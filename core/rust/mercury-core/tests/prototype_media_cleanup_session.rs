use mercury_core::{
    LocalStoreRecordKind, LocalStoreRecordLocator, MediaRetentionInput, MediaRetentionOperation,
    MediaRetentionReason, MediaServiceAdapterKind, PrototypeMediaCleanupSession,
    PrototypeMediaCleanupSessionEventKind, PrototypeMediaCleanupSessionInput,
    PrototypeMediaCleanupSessionReason,
};

#[test]
fn media_cleanup_session_deletes_remote_and_evicts_local_cache_without_plaintext_events() {
    let mut session = PrototypeMediaCleanupSession::default();
    let outcome = session.run(valid_session_input(true));

    assert!(outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaCleanupSessionReason::Completed
    );
    assert!(outcome.retention.accepted);
    assert_eq!(outcome.remote_delete_calls, 1);
    assert_eq!(session.remote_delete_calls(), 1);
    assert!(outcome.local_cache_delete_attempted);
    assert!(outcome.local_cache_deleted);
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.seeded_cache_ciphertext_len, MEDIA_CIPHERTEXT.len());
    assert!(!outcome.plaintext_exposed);
    assert!(session.local_store().get_record(locator()).is_none());
    assert_eq!(session.events().len(), 5);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeMediaCleanupSessionEventKind::CleanupStarted
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeMediaCleanupSessionEventKind::CleanupFinished
    );
    assert!(session.events().last().expect("last event").terminal);
    assert!(
        session
            .events()
            .iter()
            .all(|event| !event.plaintext_bytes_exposed)
    );
}

#[test]
fn media_cleanup_session_retain_keeps_cache_and_skips_side_effects() {
    let mut input = valid_session_input(true);
    input.retention.operation = MediaRetentionOperation::Retain;
    input.retention.service_authenticated = false;
    input.retention.delete_authorized = false;

    let mut session = PrototypeMediaCleanupSession::default();
    let outcome = session.run(input);

    assert!(outcome.completed);
    assert_eq!(outcome.remote_delete_calls, 0);
    assert!(!outcome.local_cache_delete_attempted);
    assert!(!outcome.local_cache_deleted);
    assert_eq!(outcome.local_store_records, 1);
    assert!(
        session
            .local_store()
            .get_record(locator())
            .expect("cache should remain")
            .bytes
            == MEDIA_CIPHERTEXT
    );
    assert_eq!(session.events().len(), 3);
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeMediaCleanupSessionEventKind::CleanupFinished
    );
}

#[test]
fn media_cleanup_session_stops_when_retention_rejects() {
    let mut input = valid_session_input(true);
    input.retention.retention_hold_active = true;

    let mut session = PrototypeMediaCleanupSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaCleanupSessionReason::MediaRetentionRejected
    );
    assert_eq!(
        outcome.retention.reason,
        MediaRetentionReason::RetentionHoldActive
    );
    assert_eq!(outcome.remote_delete_calls, 0);
    assert!(!outcome.local_cache_delete_attempted);
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeMediaCleanupSessionEventKind::MediaRetentionEvaluated
    );
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn media_cleanup_session_local_eviction_is_idempotent_for_absent_cache() {
    let mut input = valid_session_input(false);
    input.retention.operation = MediaRetentionOperation::EvictLocalCache;
    input.retention.user_delete_requested = false;
    input.retention.cache_eviction_requested = true;
    input.retention.service_authenticated = false;
    input.retention.delete_authorized = false;

    let mut session = PrototypeMediaCleanupSession::default();
    let outcome = session.run(input);

    assert!(outcome.completed);
    assert_eq!(outcome.remote_delete_calls, 0);
    assert!(outcome.local_cache_delete_attempted);
    assert!(!outcome.local_cache_deleted);
    assert_eq!(outcome.local_store_records, 0);
}

#[test]
fn media_cleanup_session_reasons_and_events_have_stable_codes_and_labels() {
    let reasons = [
        (
            PrototypeMediaCleanupSessionReason::Completed,
            0,
            "completed",
        ),
        (
            PrototypeMediaCleanupSessionReason::MediaRetentionRejected,
            1,
            "media_retention_rejected",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let events = [
        (
            PrototypeMediaCleanupSessionEventKind::CleanupStarted,
            1,
            "cleanup_started",
        ),
        (
            PrototypeMediaCleanupSessionEventKind::MediaRetentionEvaluated,
            2,
            "media_retention_evaluated",
        ),
        (
            PrototypeMediaCleanupSessionEventKind::RemoteDeleteEvaluated,
            3,
            "remote_delete_evaluated",
        ),
        (
            PrototypeMediaCleanupSessionEventKind::LocalCacheDeleteEvaluated,
            4,
            "local_cache_delete_evaluated",
        ),
        (
            PrototypeMediaCleanupSessionEventKind::CleanupFinished,
            5,
            "cleanup_finished",
        ),
    ];

    for (event, code, label) in events {
        assert_eq!(event.code(), code);
        assert_eq!(event.label(), label);
    }
}

fn valid_session_input(seed_local_cache: bool) -> PrototypeMediaCleanupSessionInput<'static> {
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

const MEDIA_CIPHERTEXT: [u8; 64] = [91; 64];
