use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mercury_core::{
    ComponentReasons, LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope,
    LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreSealRequest, LocalStoreSealingSuite,
    MERCURY_MAX_MEDIA_OBJECT_BYTES, MediaObjectIndexInput, MediaObjectIndexReason,
    MediaObjectIndexStoreReason, MediaObjectIndexStoreWrite, MediaObjectLifecycleState,
    MediaServiceAdapterKind, OutboundSendDecision, OutboundSendReason, PolicyDecision,
    PrototypeFileMediaObjectIndexStore, PrototypeIndexedMediaUploadSession,
    PrototypeIndexedMediaUploadSessionEventKind, PrototypeIndexedMediaUploadSessionInput,
    PrototypeIndexedMediaUploadSessionReason, PrototypeMediaServiceUploadSessionInput,
    PrototypeMediaUploadSessionInput,
};

#[test]
fn indexed_media_upload_persists_manifest_after_service_upload() {
    let mut session = PrototypeIndexedMediaUploadSession::default();
    let outcome = session.run(valid_indexed_upload_input(valid_index_store_write(
        valid_index(),
    )));

    assert!(outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaUploadSessionReason::Completed
    );
    assert!(outcome.service_upload.completed);
    assert_eq!(outcome.service_upload.service_upload_calls, 1);
    let index_store = outcome.index_store.expect("index store decision");
    assert!(index_store.accepted);
    assert_eq!(index_store.reason, MediaObjectIndexStoreReason::Accepted);
    assert!(index_store.persisted_record);
    assert_eq!(outcome.index_store_records, 1);
    assert_eq!(session.index_store().len(), 1);
    assert!(!outcome.plaintext_exposed);
    assert!(
        session
            .events()
            .iter()
            .all(|event| !event.plaintext_bytes_exposed)
    );
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeIndexedMediaUploadSessionEventKind::IndexedUploadFinished
    );
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn indexed_media_upload_can_use_external_media_index_adapter() {
    let root = temp_root("upload-adapter");
    let mut index_store = PrototypeFileMediaObjectIndexStore::new(root.clone());
    let mut session = PrototypeIndexedMediaUploadSession::default();

    let outcome = session
        .run_with_index_store(
            &mut index_store,
            valid_indexed_upload_input(valid_index_store_write(valid_index())),
        )
        .expect("file media index adapter should not fail");

    assert!(outcome.completed);
    assert_eq!(outcome.index_store_records, 1);
    assert!(
        index_store
            .get(&OBJECT_ID)
            .expect("file media index read should succeed")
            .is_some()
    );
    assert!(session.index_store().is_empty());

    cleanup(root);
}

#[test]
fn indexed_media_upload_stops_before_index_write_when_service_upload_rejects() {
    let mut input = valid_indexed_upload_input(valid_index_store_write(valid_index()));
    input.service_upload.service_authenticated = false;

    let mut session = PrototypeIndexedMediaUploadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaUploadSessionReason::MediaServiceUploadRejected
    );
    assert!(!outcome.service_upload.completed);
    assert!(outcome.index_store.is_none());
    assert_eq!(outcome.index_store_records, 0);
    assert!(session.index_store().is_empty());
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeIndexedMediaUploadSessionEventKind::MediaServiceUploadEvaluated
    );
}

#[test]
fn indexed_media_upload_stops_when_index_store_rejects() {
    let mut index = valid_index();
    index.plaintext_metadata_bytes = 1;

    let mut session = PrototypeIndexedMediaUploadSession::default();
    let outcome = session.run(valid_indexed_upload_input(valid_index_store_write(index)));

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaUploadSessionReason::MediaObjectIndexStoreRejected
    );
    assert!(outcome.service_upload.completed);
    let store = outcome.index_store.expect("index store decision");
    assert_eq!(store.reason, MediaObjectIndexStoreReason::IndexRejected);
    assert_eq!(
        store.index.reason,
        MediaObjectIndexReason::PlaintextMetadataForbidden
    );
    assert!(!store.persisted_record);
    assert_eq!(outcome.index_store_records, 0);
    assert!(session.index_store().is_empty());
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeIndexedMediaUploadSessionEventKind::MediaObjectIndexStoreEvaluated
    );
}

#[test]
fn indexed_media_upload_reasons_and_events_have_stable_codes_and_labels() {
    let reasons = [
        (
            PrototypeIndexedMediaUploadSessionReason::Completed,
            0,
            "completed",
        ),
        (
            PrototypeIndexedMediaUploadSessionReason::MediaServiceUploadRejected,
            1,
            "media_service_upload_rejected",
        ),
        (
            PrototypeIndexedMediaUploadSessionReason::MediaObjectIndexStoreRejected,
            2,
            "media_object_index_store_rejected",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let events = [
        (
            PrototypeIndexedMediaUploadSessionEventKind::IndexedUploadStarted,
            1,
            "indexed_upload_started",
        ),
        (
            PrototypeIndexedMediaUploadSessionEventKind::MediaServiceUploadEvaluated,
            2,
            "media_service_upload_evaluated",
        ),
        (
            PrototypeIndexedMediaUploadSessionEventKind::MediaObjectIndexStoreEvaluated,
            3,
            "media_object_index_store_evaluated",
        ),
        (
            PrototypeIndexedMediaUploadSessionEventKind::IndexedUploadFinished,
            4,
            "indexed_upload_finished",
        ),
    ];

    for (event, code, label) in events {
        assert_eq!(event.code(), code);
        assert_eq!(event.label(), label);
    }
}

fn valid_indexed_upload_input(
    index_store: MediaObjectIndexStoreWrite<'static>,
) -> PrototypeIndexedMediaUploadSessionInput<'static> {
    PrototypeIndexedMediaUploadSessionInput {
        service_upload: valid_service_upload_input(),
        index_store,
    }
}

fn valid_index_store_write(index: MediaObjectIndexInput) -> MediaObjectIndexStoreWrite<'static> {
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

fn valid_service_upload_input() -> PrototypeMediaServiceUploadSessionInput<'static> {
    PrototypeMediaServiceUploadSessionInput {
        media_upload: valid_media_upload_input(),
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        service_authenticated: true,
        upload_authorized: true,
        object_namespace_bound: true,
        content_digest_verified: true,
        allow_development_adapter: false,
    }
}

fn valid_media_upload_input() -> PrototypeMediaUploadSessionInput<'static> {
    PrototypeMediaUploadSessionInput {
        seal_request: seal_request(
            LocalStoreRecordKind::MediaCiphertext,
            MEDIA_PLAINTEXT.len() as i32,
            Some(policy_decision(true)),
        ),
        plaintext: &MEDIA_PLAINTEXT,
        outbound_send: OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        },
        object_id: &OBJECT_ID,
        max_ciphertext_len: MERCURY_MAX_MEDIA_OBJECT_BYTES,
        sealed_header: &SEALED_HEADER,
        content_digest: &CONTENT_DIGEST,
        media_key_commitment: &MEDIA_KEY_COMMITMENT,
        plaintext_upload_bytes: 0,
        automatic_download_requested: false,
        store_record_kind: LocalStoreRecordKind::MediaCiphertext,
    }
}

fn seal_request(
    record_kind: LocalStoreRecordKind,
    plaintext_len: i32,
    policy_decision: Option<PolicyDecision>,
) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        locator(),
        record_kind,
        key(record_kind.policy().key_scope),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        plaintext_len,
        policy_decision,
    )
}

fn locator() -> LocalStoreRecordLocator<'static> {
    LocalStoreRecordLocator::new("conversation-7", "media-object-42")
}

fn key(scope: LocalStoreKeyScope) -> LocalStoreKeyDescriptor {
    let binding = match scope {
        LocalStoreKeyScope::AccountRoot => LocalStoreKeyBinding::account(32),
        LocalStoreKeyScope::DeviceLocal => LocalStoreKeyBinding::device(32, 32),
        LocalStoreKeyScope::Conversation => LocalStoreKeyBinding::conversation(32, 32),
        LocalStoreKeyScope::RoomEpoch => LocalStoreKeyBinding::room_epoch(32, 32, 7),
        LocalStoreKeyScope::Media => LocalStoreKeyBinding::media(32, 32, 7),
        LocalStoreKeyScope::Audit => LocalStoreKeyBinding::audit(32),
    };

    LocalStoreKeyDescriptor::new(
        scope,
        LocalStoreSealingSuite::MercuryLocalStoreV1,
        1,
        binding,
    )
}

fn policy_decision(accepted: bool) -> PolicyDecision {
    PolicyDecision {
        accepted,
        reason_code: if accepted { 0 } else { 1 },
        audit_class: 0,
        components: ComponentReasons {
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    }
}

const MEDIA_PLAINTEXT: [u8; 64] = [17; 64];
const OBJECT_ID: [u8; 32] = [7; 32];
const SEALED_HEADER: [u8; 96] = [9; 96];
const CONTENT_DIGEST: [u8; 32] = [11; 32];
const MEDIA_KEY_COMMITMENT: [u8; 32] = [13; 32];

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "mercury-indexed-upload-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn cleanup(root: PathBuf) {
    let _ = fs::remove_dir_all(root);
}
