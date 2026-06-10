use mercury_core::{
    ComponentReasons, LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope,
    LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreSealRequest, LocalStoreSealingSuite,
    MERCURY_MAX_MEDIA_OBJECT_BYTES, MediaObjectStoreReason, MediaServiceAdapterKind,
    MediaServiceAdapterReason, OutboundSendDecision, OutboundSendReason, PolicyDecision,
    PrototypeMediaServiceUploadSession, PrototypeMediaServiceUploadSessionEventKind,
    PrototypeMediaServiceUploadSessionInput, PrototypeMediaServiceUploadSessionReason,
    PrototypeMediaUploadSessionInput,
};

#[test]
fn media_service_upload_session_completes_plaintext_free_path() {
    let mut session = PrototypeMediaServiceUploadSession::default();
    let outcome = session.run(valid_service_upload_input());

    assert!(outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaServiceUploadSessionReason::Completed
    );
    assert!(outcome.media_upload.completed);
    assert!(outcome.media_service.expect("service").accepted);
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.service_upload_calls, 1);
    assert_eq!(session.service_upload_calls(), 1);
    assert!(outcome.sealed_ciphertext_len > MEDIA_PLAINTEXT.len());
    assert_eq!(outcome.stored_ciphertext_len, outcome.sealed_ciphertext_len);
    assert!(!outcome.plaintext_exposed);
    assert_eq!(session.events().len(), 4);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeMediaServiceUploadSessionEventKind::ServiceUploadStarted
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeMediaServiceUploadSessionEventKind::ServiceUploadFinished
    );
    assert_eq!(
        session
            .events()
            .last()
            .expect("last event")
            .view()
            .kind_label,
        "service_upload_finished"
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
fn media_service_upload_session_stops_when_media_upload_rejects() {
    let mut input = valid_service_upload_input();
    input.media_upload.plaintext_upload_bytes = MEDIA_PLAINTEXT.len() as i32;

    let mut session = PrototypeMediaServiceUploadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaServiceUploadSessionReason::MediaUploadRejected
    );
    assert_eq!(
        outcome.media_upload.media.expect("media decision").reason,
        MediaObjectStoreReason::PlaintextUploadForbidden
    );
    assert!(outcome.media_service.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.service_upload_calls, 0);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeMediaServiceUploadSessionEventKind::MediaUploadSessionEvaluated
    );
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn media_service_upload_session_stops_when_service_adapter_rejects() {
    let mut input = valid_service_upload_input();
    input.service_authenticated = false;

    let mut session = PrototypeMediaServiceUploadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaServiceUploadSessionReason::MediaServiceAdapterRejected
    );
    assert!(outcome.media_upload.completed);
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.service_upload_calls, 0);
    assert_eq!(
        outcome.media_service.expect("service").reason,
        MediaServiceAdapterReason::ServiceAuthenticationMissing
    );
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeMediaServiceUploadSessionEventKind::MediaServiceAdapterEvaluated
    );
}

#[test]
fn media_service_upload_session_reasons_and_events_have_stable_codes_and_labels() {
    let reasons = [
        (
            PrototypeMediaServiceUploadSessionReason::Completed,
            0,
            "completed",
        ),
        (
            PrototypeMediaServiceUploadSessionReason::MediaUploadRejected,
            1,
            "media_upload_rejected",
        ),
        (
            PrototypeMediaServiceUploadSessionReason::MediaServiceAdapterRejected,
            2,
            "media_service_adapter_rejected",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let events = [
        (
            PrototypeMediaServiceUploadSessionEventKind::ServiceUploadStarted,
            1,
            "service_upload_started",
        ),
        (
            PrototypeMediaServiceUploadSessionEventKind::MediaUploadSessionEvaluated,
            2,
            "media_upload_session_evaluated",
        ),
        (
            PrototypeMediaServiceUploadSessionEventKind::MediaServiceAdapterEvaluated,
            3,
            "media_service_adapter_evaluated",
        ),
        (
            PrototypeMediaServiceUploadSessionEventKind::ServiceUploadFinished,
            4,
            "service_upload_finished",
        ),
    ];

    for (event, code, label) in events {
        assert_eq!(event.code(), code);
        assert_eq!(event.label(), label);
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
