use mercury_core::{
    ComponentReasons, LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope,
    LocalStoreOpenReason, LocalStorePayloadKind, LocalStoreRecordKind, LocalStoreRecordLocator,
    LocalStoreSealRequest, LocalStoreSealResult, LocalStoreSealingSuite, LocalStoreWriteReason,
    MediaServiceAdapterKind, MediaServiceDownloadInput, MediaServiceDownloadReason,
    PrototypeLocalStoreCryptoProvider, PrototypeMediaDownloadSession,
    PrototypeMediaDownloadSessionEventKind, PrototypeMediaDownloadSessionInput,
    PrototypeMediaDownloadSessionReason, seal_local_store_plaintext,
};

#[test]
fn media_download_session_caches_and_opens_ciphertext_without_plaintext_events() {
    let sealed = sealed_media();
    let mut session = PrototypeMediaDownloadSession::default();
    let outcome = session.run(valid_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    ));

    assert!(outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaDownloadSessionReason::Completed
    );
    assert!(outcome.download.accepted);
    assert!(outcome.store_write.expect("store write").accepted);
    assert!(outcome.open.expect("open").accepted);
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.stored_ciphertext_len, sealed.sealed_bytes.len());
    assert_eq!(outcome.opened_plaintext_len, MEDIA_PLAINTEXT.len());
    assert_eq!(outcome.service_download_calls, 1);
    assert_eq!(outcome.crypto_open_calls, 1);
    assert!(!outcome.plaintext_exposed);
    assert_eq!(session.service_download_calls(), 1);
    assert_eq!(session.events().len(), 5);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeMediaDownloadSessionEventKind::DownloadStarted
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeMediaDownloadSessionEventKind::DownloadFinished
    );
    assert!(session.events().last().expect("last event").terminal);
    assert!(
        session
            .events()
            .iter()
            .all(|event| !event.plaintext_bytes_exposed)
    );

    let record = session
        .local_store()
        .get_record(locator())
        .expect("downloaded ciphertext should be cached");
    assert_eq!(record.record_kind, LocalStoreRecordKind::MediaCiphertext);
    assert_eq!(record.payload_kind, LocalStorePayloadKind::Sealed);
    assert_eq!(record.bytes.len(), sealed.sealed_bytes.len());
}

#[test]
fn media_download_session_stops_when_download_gate_rejects() {
    let sealed = sealed_media();
    let mut input = valid_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.download.plaintext_preview_bytes = 1;

    let mut session = PrototypeMediaDownloadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaDownloadSessionReason::MediaServiceDownloadRejected
    );
    assert_eq!(
        outcome.download.reason,
        MediaServiceDownloadReason::PlaintextPreviewForbidden
    );
    assert!(outcome.store_write.is_none());
    assert!(outcome.open.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.service_download_calls, 0);
    assert_eq!(outcome.crypto_open_calls, 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeMediaDownloadSessionEventKind::MediaServiceDownloadEvaluated
    );
}

#[test]
fn media_download_session_stops_when_store_write_rejects() {
    let sealed = sealed_media();
    let mut input = valid_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.store_record_kind = LocalStoreRecordKind::MediaPlaintext;

    let mut session = PrototypeMediaDownloadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaDownloadSessionReason::LocalStoreWriteRejected
    );
    assert!(outcome.download.accepted);
    assert_eq!(
        outcome.store_write.expect("store write").reason,
        LocalStoreWriteReason::PlaintextRecordForbidden
    );
    assert!(outcome.open.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.service_download_calls, 1);
    assert_eq!(outcome.crypto_open_calls, 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeMediaDownloadSessionEventKind::LocalStoreWriteEvaluated
    );
}

#[test]
fn media_download_session_stops_when_local_open_rejects() {
    let sealed = sealed_media();
    let mut session = PrototypeMediaDownloadSession::default();
    let outcome = session.run(valid_session_input(
        &sealed.sealed_bytes,
        &[7; 12],
        sealed.authentication_tag_len,
    ));

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaDownloadSessionReason::LocalStoreOpenRejected
    );
    assert!(outcome.download.accepted);
    assert!(outcome.store_write.expect("store write").accepted);
    assert_eq!(
        outcome.open.expect("open").reason,
        LocalStoreOpenReason::BadNonceLength
    );
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.stored_ciphertext_len, sealed.sealed_bytes.len());
    assert_eq!(outcome.opened_plaintext_len, 0);
    assert_eq!(outcome.service_download_calls, 1);
    assert_eq!(outcome.crypto_open_calls, 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeMediaDownloadSessionEventKind::LocalStoreOpenEvaluated
    );
}

#[test]
fn media_download_session_uses_downloaded_ciphertext_length_for_gate() {
    let sealed = sealed_media();
    let mut input = valid_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.download.ciphertext_len = 1;

    let mut session = PrototypeMediaDownloadSession::default();
    let outcome = session.run(input);

    assert!(outcome.completed);
    assert_eq!(outcome.stored_ciphertext_len, sealed.sealed_bytes.len());
}

#[test]
fn media_download_session_reasons_and_events_have_stable_codes_and_labels() {
    let reasons = [
        (
            PrototypeMediaDownloadSessionReason::Completed,
            0,
            "completed",
        ),
        (
            PrototypeMediaDownloadSessionReason::MediaServiceDownloadRejected,
            1,
            "media_service_download_rejected",
        ),
        (
            PrototypeMediaDownloadSessionReason::LocalStoreWriteRejected,
            2,
            "local_store_write_rejected",
        ),
        (
            PrototypeMediaDownloadSessionReason::LocalStoreOpenRejected,
            3,
            "local_store_open_rejected",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let events = [
        (
            PrototypeMediaDownloadSessionEventKind::DownloadStarted,
            1,
            "download_started",
        ),
        (
            PrototypeMediaDownloadSessionEventKind::MediaServiceDownloadEvaluated,
            2,
            "media_service_download_evaluated",
        ),
        (
            PrototypeMediaDownloadSessionEventKind::LocalStoreWriteEvaluated,
            3,
            "local_store_write_evaluated",
        ),
        (
            PrototypeMediaDownloadSessionEventKind::LocalStoreOpenEvaluated,
            4,
            "local_store_open_evaluated",
        ),
        (
            PrototypeMediaDownloadSessionEventKind::DownloadFinished,
            5,
            "download_finished",
        ),
    ];

    for (event, code, label) in events {
        assert_eq!(event.code(), code);
        assert_eq!(event.label(), label);
    }
}

fn valid_session_input<'a>(
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

const MEDIA_PLAINTEXT: [u8; 64] = [17; 64];
