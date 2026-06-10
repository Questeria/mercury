use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use mercury_core::{
    ComponentReasons, LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope,
    LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreSealRequest, LocalStoreSealResult,
    LocalStoreSealingSuite, MediaObjectIndexInput, MediaObjectIndexStoreReason,
    MediaObjectIndexStoreWrite, MediaObjectLifecycleState, MediaServiceAdapterKind,
    MediaServiceDownloadInput, PrototypeFileMediaObjectIndexStore,
    PrototypeIndexedMediaDownloadSession, PrototypeIndexedMediaDownloadSessionEventKind,
    PrototypeIndexedMediaDownloadSessionInput, PrototypeIndexedMediaDownloadSessionReason,
    PrototypeLocalStoreCryptoProvider, PrototypeMediaDownloadSessionInput,
    PrototypeMediaDownloadSessionReason, seal_local_store_plaintext,
};

#[test]
fn indexed_media_download_runs_after_downloadable_manifest() {
    let sealed = sealed_media();
    let mut session = PrototypeIndexedMediaDownloadSession::default();
    let outcome = session.run(valid_indexed_download_input(
        valid_index_store_write(valid_media_object_index()),
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    ));

    assert!(outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaDownloadSessionReason::Completed
    );
    assert!(outcome.index_store.accepted);
    assert!(outcome.index_store.index.can_download);
    let download = outcome.download.expect("download should run");
    assert!(download.completed);
    assert_eq!(download.local_store_records, 1);
    assert_eq!(outcome.index_store_records, 1);
    assert_eq!(session.index_store().len(), 1);
    assert_eq!(session.media_download_session().service_download_calls(), 1);
    assert_eq!(session.events().len(), 4);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeIndexedMediaDownloadSessionEventKind::IndexedDownloadStarted
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeIndexedMediaDownloadSessionEventKind::IndexedDownloadFinished
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
fn indexed_media_download_can_use_external_media_index_adapter() {
    let sealed = sealed_media();
    let root = temp_root("download-adapter");
    let mut index_store = PrototypeFileMediaObjectIndexStore::new(root.clone());
    let mut session = PrototypeIndexedMediaDownloadSession::default();

    let outcome = session
        .run_with_index_store(
            &mut index_store,
            valid_indexed_download_input(
                valid_index_store_write(valid_media_object_index()),
                &sealed.sealed_bytes,
                &sealed.nonce,
                sealed.authentication_tag_len,
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
fn indexed_media_download_stops_when_manifest_store_rejects() {
    let sealed = sealed_media();
    let mut index = valid_media_object_index();
    index.plaintext_metadata_bytes = 1;

    let mut session = PrototypeIndexedMediaDownloadSession::default();
    let outcome = session.run(valid_indexed_download_input(
        valid_index_store_write(index),
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    ));

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaDownloadSessionReason::MediaObjectIndexStoreRejected
    );
    assert_eq!(
        outcome.index_store.reason,
        MediaObjectIndexStoreReason::IndexRejected
    );
    assert!(outcome.download.is_none());
    assert_eq!(outcome.index_store_records, 0);
    assert_eq!(session.media_download_session().service_download_calls(), 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeIndexedMediaDownloadSessionEventKind::MediaObjectIndexStoreEvaluated
    );
}

#[test]
fn indexed_media_download_stops_when_manifest_is_not_downloadable() {
    let sealed = sealed_media();
    let mut index = valid_media_object_index();
    index.lifecycle_state = MediaObjectLifecycleState::Absent;
    index.local_cache_present = false;
    index.remote_object_present = false;
    index.remote_service_record_present = false;

    let mut session = PrototypeIndexedMediaDownloadSession::default();
    let outcome = session.run(valid_indexed_download_input(
        valid_index_store_write(index),
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    ));

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaDownloadSessionReason::MediaObjectNotDownloadable
    );
    assert!(outcome.index_store.accepted);
    assert!(!outcome.index_store.index.can_download);
    assert!(outcome.download.is_none());
    assert_eq!(outcome.index_store_records, 1);
    assert_eq!(session.media_download_session().service_download_calls(), 0);
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn indexed_media_download_stops_when_download_session_rejects() {
    let sealed = sealed_media();
    let mut input = valid_indexed_download_input(
        valid_index_store_write(valid_media_object_index()),
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.download.download.plaintext_preview_bytes = 1;

    let mut session = PrototypeIndexedMediaDownloadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeIndexedMediaDownloadSessionReason::MediaDownloadRejected
    );
    assert!(outcome.index_store.accepted);
    assert_eq!(outcome.index_store_records, 1);
    let download = outcome.download.expect("download should be evaluated");
    assert!(!download.completed);
    assert_eq!(
        download.reason,
        PrototypeMediaDownloadSessionReason::MediaServiceDownloadRejected
    );
    assert_eq!(session.media_download_session().service_download_calls(), 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeIndexedMediaDownloadSessionEventKind::MediaDownloadSessionEvaluated
    );
}

#[test]
fn indexed_media_download_reasons_and_events_have_stable_codes_and_labels() {
    let reasons = [
        (
            PrototypeIndexedMediaDownloadSessionReason::Completed,
            0,
            "completed",
        ),
        (
            PrototypeIndexedMediaDownloadSessionReason::MediaObjectIndexStoreRejected,
            1,
            "media_object_index_store_rejected",
        ),
        (
            PrototypeIndexedMediaDownloadSessionReason::MediaObjectNotDownloadable,
            2,
            "media_object_not_downloadable",
        ),
        (
            PrototypeIndexedMediaDownloadSessionReason::MediaDownloadRejected,
            3,
            "media_download_rejected",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let events = [
        (
            PrototypeIndexedMediaDownloadSessionEventKind::IndexedDownloadStarted,
            1,
            "indexed_download_started",
        ),
        (
            PrototypeIndexedMediaDownloadSessionEventKind::MediaObjectIndexStoreEvaluated,
            2,
            "media_object_index_store_evaluated",
        ),
        (
            PrototypeIndexedMediaDownloadSessionEventKind::MediaDownloadSessionEvaluated,
            3,
            "media_download_session_evaluated",
        ),
        (
            PrototypeIndexedMediaDownloadSessionEventKind::IndexedDownloadFinished,
            4,
            "indexed_download_finished",
        ),
    ];

    for (event, code, label) in events {
        assert_eq!(event.code(), code);
        assert_eq!(event.label(), label);
    }
}

fn valid_indexed_download_input<'a>(
    index_store: MediaObjectIndexStoreWrite<'a>,
    ciphertext: &'a [u8],
    nonce: &'a [u8],
    authentication_tag_len: i32,
) -> PrototypeIndexedMediaDownloadSessionInput<'a> {
    PrototypeIndexedMediaDownloadSessionInput {
        index_store,
        download: valid_download_input(ciphertext, nonce, authentication_tag_len),
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
        ciphertext_len: 80,
        max_ciphertext_len: mercury_core::MERCURY_MAX_MEDIA_OBJECT_BYTES,
        plaintext_metadata_bytes: 0,
        content_digest_verified: true,
        local_cache_present: true,
        remote_object_present: true,
        remote_service_record_present: true,
        retention_hold_active: false,
    }
}

fn valid_download_input<'a>(
    ciphertext: &'a [u8],
    nonce: &'a [u8],
    authentication_tag_len: i32,
) -> PrototypeMediaDownloadSessionInput<'a> {
    PrototypeMediaDownloadSessionInput {
        download: MediaServiceDownloadInput {
            adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
            service_authenticated: true,
            download_authorized: true,
            object_namespace_bound: true,
            content_digest_verified: true,
            allow_development_adapter: false,
            object_id_len: 32,
            ciphertext_len: ciphertext.len() as i32,
            max_ciphertext_len: mercury_core::MERCURY_MAX_MEDIA_OBJECT_BYTES,
            sealed_header_len: 96,
            content_digest_len: 32,
            media_key_commitment_len: 32,
            plaintext_preview_bytes: 0,
            automatic_download_requested: false,
        },
        open_seal_request: seal_request(
            LocalStoreRecordKind::MediaCiphertext,
            MEDIA_PLAINTEXT.len() as i32,
        ),
        downloaded_ciphertext: ciphertext,
        nonce,
        authentication_tag_len,
        store_record_kind: LocalStoreRecordKind::MediaCiphertext,
    }
}

fn sealed_media() -> mercury_core::LocalStoreSealOutput {
    let mut crypto = PrototypeLocalStoreCryptoProvider::default();
    match seal_local_store_plaintext(
        &mut crypto,
        seal_request(
            LocalStoreRecordKind::MediaCiphertext,
            MEDIA_PLAINTEXT.len() as i32,
        ),
        &MEDIA_PLAINTEXT,
    )
    .expect("prototype crypto is infallible")
    {
        LocalStoreSealResult::Sealed(output) => output,
        LocalStoreSealResult::Rejected(decision) => {
            panic!("fixture seal should be accepted: {:?}", decision.reason)
        }
    }
}

fn seal_request(
    record_kind: LocalStoreRecordKind,
    plaintext_len: i32,
) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        locator(),
        record_kind,
        key(record_kind.policy().key_scope),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        plaintext_len,
        Some(policy_decision(true)),
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

fn policy_decision(accepted: bool) -> mercury_core::PolicyDecision {
    mercury_core::PolicyDecision {
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

const MEDIA_OBJECT_ID: [u8; 32] = [3; 32];
const MEDIA_CONTENT_DIGEST: [u8; 32] = [6; 32];
const MEDIA_KEY_COMMITMENT: [u8; 32] = [8; 32];
const MEDIA_PLAINTEXT: [u8; 64] = [17; 64];

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "mercury-indexed-download-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn cleanup(root: PathBuf) {
    let _ = fs::remove_dir_all(root);
}
