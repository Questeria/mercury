use mercury_core::{
    ComponentReasons, LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope,
    LocalStoreOpenReason, LocalStoreOpenRequest, LocalStoreOpenResult, LocalStoreRecordKind,
    LocalStoreRecordLocator, LocalStoreSealRequest, LocalStoreSealResult, LocalStoreSealingReason,
    LocalStoreSealingSuite, LocalStoreWriteReason, PolicyDecision, PrototypeEncryptedLocalStore,
    PrototypeLocalStoreCryptoProvider, build_sealed_local_store_write_request,
    open_local_store_record, put_local_store_record, seal_local_store_plaintext,
};

#[test]
fn prototype_crypto_seals_and_stores_valid_plaintext() {
    let mut crypto = PrototypeLocalStoreCryptoProvider::default();
    let plaintext = b"selected ciphertext cache";
    let request = seal_request(
        LocalStoreRecordKind::MessageCiphertext,
        plaintext.len() as i32,
        Some(policy_decision(true)),
    );

    let sealed = match seal_local_store_plaintext(&mut crypto, request, plaintext)
        .expect("prototype crypto is infallible")
    {
        LocalStoreSealResult::Sealed(output) => output,
        LocalStoreSealResult::Rejected(decision) => {
            panic!("expected seal acceptance, got {:?}", decision.reason)
        }
    };

    assert_eq!(crypto.seal_calls(), 1);
    assert_eq!(sealed.nonce.len(), 24);
    assert_eq!(sealed.authentication_tag_len, 16);
    assert_ne!(sealed.sealed_bytes, plaintext);

    let write = build_sealed_local_store_write_request(request, &sealed.sealed_bytes)
        .expect("valid sealed output should build a write request");
    let mut store = PrototypeEncryptedLocalStore::default();
    let write_decision =
        put_local_store_record(&mut store, write).expect("prototype store is infallible");

    assert!(write_decision.accepted);
    assert_eq!(store.len(), 1);
}

#[test]
fn prototype_crypto_rejects_before_provider_for_bad_seal_requests() {
    let mut crypto = PrototypeLocalStoreCryptoProvider::default();
    let plaintext = b"message";

    let mismatch = seal_local_store_plaintext(
        &mut crypto,
        seal_request(
            LocalStoreRecordKind::MessageCiphertext,
            99,
            Some(policy_decision(true)),
        ),
        plaintext,
    )
    .expect("prototype crypto is infallible");
    assert_seal_rejected(mismatch, LocalStoreSealingReason::BadPlaintextLength);

    let rejected_policy = seal_local_store_plaintext(
        &mut crypto,
        seal_request(
            LocalStoreRecordKind::MessageCiphertext,
            plaintext.len() as i32,
            Some(policy_decision(false)),
        ),
        plaintext,
    )
    .expect("prototype crypto is infallible");
    assert_seal_rejected(
        rejected_policy,
        LocalStoreSealingReason::PolicyDecisionRejected,
    );
    assert_eq!(crypto.seal_calls(), 0);
}

#[test]
fn prototype_crypto_opens_valid_sealed_bytes() {
    let mut crypto = PrototypeLocalStoreCryptoProvider::default();
    let plaintext = b"round trip local plaintext";
    let request = seal_request(
        LocalStoreRecordKind::ConversationSecret,
        plaintext.len() as i32,
        None,
    );
    let sealed = match seal_local_store_plaintext(&mut crypto, request, plaintext)
        .expect("prototype crypto is infallible")
    {
        LocalStoreSealResult::Sealed(output) => output,
        LocalStoreSealResult::Rejected(decision) => {
            panic!("expected seal acceptance, got {:?}", decision.reason)
        }
    };
    let open = LocalStoreOpenRequest::new(
        request,
        &sealed.nonce,
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );

    let opened =
        open_local_store_record(&mut crypto, open).expect("prototype crypto is infallible");

    assert_eq!(crypto.open_calls(), 1);
    assert_eq!(opened, LocalStoreOpenResult::Opened(plaintext.to_vec()));
}

#[test]
fn prototype_crypto_rejects_bad_open_metadata_before_provider() {
    let mut crypto = PrototypeLocalStoreCryptoProvider::default();
    let plaintext = b"message";
    let request = seal_request(
        LocalStoreRecordKind::ConversationSecret,
        plaintext.len() as i32,
        None,
    );
    let sealed = match seal_local_store_plaintext(&mut crypto, request, plaintext)
        .expect("prototype crypto is infallible")
    {
        LocalStoreSealResult::Sealed(output) => output,
        LocalStoreSealResult::Rejected(decision) => {
            panic!("expected seal acceptance, got {:?}", decision.reason)
        }
    };

    let bad_nonce = LocalStoreOpenRequest::new(
        request,
        &[1; 12],
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );
    assert_open_rejected(
        open_local_store_record(&mut crypto, bad_nonce).expect("prototype crypto is infallible"),
        LocalStoreOpenReason::BadNonceLength,
    );

    let bad_tag = LocalStoreOpenRequest::new(request, &sealed.nonce, &sealed.sealed_bytes, 8);
    assert_open_rejected(
        open_local_store_record(&mut crypto, bad_tag).expect("prototype crypto is infallible"),
        LocalStoreOpenReason::BadAuthenticationTagLength,
    );

    assert_eq!(crypto.open_calls(), 0);
}

#[test]
fn sealed_write_request_still_rejects_wrong_payload_class() {
    let mut crypto = PrototypeLocalStoreCryptoProvider::default();
    let plaintext = b"audit";
    let request = seal_request(
        LocalStoreRecordKind::PolicyDecisionAudit,
        plaintext.len() as i32,
        Some(policy_decision(false)),
    );
    let result = seal_local_store_plaintext(&mut crypto, request, plaintext)
        .expect("prototype crypto is infallible");

    assert_seal_rejected(result, LocalStoreSealingReason::RecordCannotBeSealed);
    assert_eq!(crypto.seal_calls(), 0);

    let write_decision = mercury_core::LocalStoreWriteRequest::new(
        locator(),
        LocalStoreRecordKind::PolicyDecisionAudit,
        mercury_core::LocalStorePayload::sealed(b"sealed-audit"),
        Some(policy_decision(false)),
    )
    .evaluate();
    assert!(!write_decision.accepted);
    assert_eq!(
        write_decision.reason,
        LocalStoreWriteReason::PayloadClassMismatch
    );
}

fn assert_seal_rejected(result: LocalStoreSealResult, reason: LocalStoreSealingReason) {
    match result {
        LocalStoreSealResult::Sealed(_) => panic!("expected seal rejection"),
        LocalStoreSealResult::Rejected(decision) => {
            assert!(!decision.accepted);
            assert_eq!(decision.reason, reason);
        }
    }
}

fn assert_open_rejected(result: LocalStoreOpenResult, reason: LocalStoreOpenReason) {
    match result {
        LocalStoreOpenResult::Opened(_) => panic!("expected open rejection"),
        LocalStoreOpenResult::Rejected(decision) => {
            assert!(!decision.accepted);
            assert_eq!(decision.reason, reason);
        }
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
    LocalStoreRecordLocator::new("conversation-7", "record-42")
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
