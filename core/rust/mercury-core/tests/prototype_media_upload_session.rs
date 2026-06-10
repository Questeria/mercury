use mercury_core::{
    ComponentReasons, LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope,
    LocalStorePayloadKind, LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreSealRequest,
    LocalStoreSealingReason, LocalStoreSealingSuite, LocalStoreWriteReason,
    MERCURY_MAX_MEDIA_OBJECT_BYTES, MediaObjectStoreReason, OutboundSendDecision,
    OutboundSendReason, PolicyDecision, PrototypeMediaUploadSession,
    PrototypeMediaUploadSessionEventKind, PrototypeMediaUploadSessionInput,
    PrototypeMediaUploadSessionReason,
};

#[test]
fn media_upload_session_seals_checks_and_persists_ciphertext_only() {
    let mut session = PrototypeMediaUploadSession::default();
    let outcome = session.run(valid_session_input());

    assert!(outcome.completed);
    assert_eq!(outcome.reason, PrototypeMediaUploadSessionReason::Completed);
    assert!(outcome.seal.expect("seal").accepted);
    assert!(outcome.media.expect("media").accepted);
    assert!(outcome.store_write.expect("store write").accepted);
    assert_eq!(outcome.local_store_records, 1);
    assert_eq!(outcome.crypto_seal_calls, 1);
    assert!(outcome.sealed_ciphertext_len > MEDIA_PLAINTEXT.len());
    assert_eq!(outcome.stored_ciphertext_len, outcome.sealed_ciphertext_len);
    assert!(!outcome.plaintext_exposed);
    assert_eq!(session.events().len(), 5);
    assert_eq!(
        session.events().first().expect("first event").kind,
        PrototypeMediaUploadSessionEventKind::UploadStarted
    );
    assert_eq!(
        session.events().last().expect("last event").kind,
        PrototypeMediaUploadSessionEventKind::UploadFinished
    );
    assert_eq!(
        session
            .events()
            .last()
            .expect("last event")
            .view()
            .kind_label,
        "upload_finished"
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
        .expect("media ciphertext should be cached");
    assert_eq!(record.record_kind, LocalStoreRecordKind::MediaCiphertext);
    assert_eq!(record.payload_kind, LocalStorePayloadKind::Sealed);
    assert_ne!(record.bytes, MEDIA_PLAINTEXT);
}

#[test]
fn media_upload_session_rejects_plaintext_upload_without_store_write() {
    let mut input = valid_session_input();
    input.plaintext_upload_bytes = MEDIA_PLAINTEXT.len() as i32;

    let mut session = PrototypeMediaUploadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaUploadSessionReason::MediaObjectStoreRejected
    );
    assert_eq!(
        outcome.media.expect("media").reason,
        MediaObjectStoreReason::PlaintextUploadForbidden
    );
    assert!(outcome.store_write.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.crypto_seal_calls, 1);
    assert!(!outcome.plaintext_exposed);
    assert_eq!(
        session.events().last().expect("terminal event").kind,
        PrototypeMediaUploadSessionEventKind::MediaObjectStoreEvaluated
    );
    assert!(session.events().last().expect("terminal event").terminal);
}

#[test]
fn media_upload_session_stops_when_local_seal_rejects() {
    let mut input = valid_session_input();
    input.seal_request = seal_request(
        LocalStoreRecordKind::MediaPlaintext,
        MEDIA_PLAINTEXT.len() as i32,
        Some(policy_decision(true)),
    );

    let mut session = PrototypeMediaUploadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaUploadSessionReason::LocalStoreSealRejected
    );
    assert_eq!(
        outcome.seal.expect("seal").reason,
        LocalStoreSealingReason::RecordCannotBeSealed
    );
    assert!(outcome.media.is_none());
    assert!(outcome.store_write.is_none());
    assert_eq!(outcome.local_store_records, 0);
    assert_eq!(outcome.crypto_seal_calls, 0);
}

#[test]
fn media_upload_session_keeps_store_policy_as_final_guard() {
    let mut input = valid_session_input();
    input.store_record_kind = LocalStoreRecordKind::MediaPlaintext;

    let mut session = PrototypeMediaUploadSession::default();
    let outcome = session.run(input);

    assert!(!outcome.completed);
    assert_eq!(
        outcome.reason,
        PrototypeMediaUploadSessionReason::LocalStoreWriteRejected
    );
    assert!(outcome.media.expect("media").accepted);
    assert_eq!(
        outcome.store_write.expect("store write").reason,
        LocalStoreWriteReason::PlaintextRecordForbidden
    );
    assert_eq!(outcome.local_store_records, 0);
}

#[test]
fn media_upload_session_reasons_and_events_have_stable_codes_and_labels() {
    let reasons = [
        (PrototypeMediaUploadSessionReason::Completed, 0, "completed"),
        (
            PrototypeMediaUploadSessionReason::LocalStoreSealRejected,
            1,
            "local_store_seal_rejected",
        ),
        (
            PrototypeMediaUploadSessionReason::MediaObjectStoreRejected,
            2,
            "media_object_store_rejected",
        ),
        (
            PrototypeMediaUploadSessionReason::LocalStoreWriteRejected,
            3,
            "local_store_write_rejected",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }

    let events = [
        (
            PrototypeMediaUploadSessionEventKind::UploadStarted,
            1,
            "upload_started",
        ),
        (
            PrototypeMediaUploadSessionEventKind::LocalStoreSealEvaluated,
            2,
            "local_store_seal_evaluated",
        ),
        (
            PrototypeMediaUploadSessionEventKind::MediaObjectStoreEvaluated,
            3,
            "media_object_store_evaluated",
        ),
        (
            PrototypeMediaUploadSessionEventKind::LocalStoreWriteEvaluated,
            4,
            "local_store_write_evaluated",
        ),
        (
            PrototypeMediaUploadSessionEventKind::UploadFinished,
            5,
            "upload_finished",
        ),
    ];

    for (event, code, label) in events {
        assert_eq!(event.code(), code);
        assert_eq!(event.label(), label);
    }
}

fn valid_session_input() -> PrototypeMediaUploadSessionInput<'static> {
    PrototypeMediaUploadSessionInput {
        seal_request: seal_request(
            LocalStoreRecordKind::MediaCiphertext,
            MEDIA_PLAINTEXT.len() as i32,
            Some(policy_decision(true)),
        ),
        plaintext: &MEDIA_PLAINTEXT,
        outbound_send: outbound_send(true),
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

fn outbound_send(accepted: bool) -> OutboundSendDecision {
    OutboundSendDecision {
        accepted,
        can_send: accepted,
        can_persist_ciphertext: accepted,
        requires_user_action: !accepted,
        reason: if accepted {
            OutboundSendReason::Accepted
        } else {
            OutboundSendReason::MessagePolicyRejected
        },
    }
}

const MEDIA_PLAINTEXT: [u8; 64] = [17; 64];
const OBJECT_ID: [u8; 32] = [7; 32];
const SEALED_HEADER: [u8; 96] = [9; 96];
const CONTENT_DIGEST: [u8; 32] = [11; 32];
const MEDIA_KEY_COMMITMENT: [u8; 32] = [13; 32];
