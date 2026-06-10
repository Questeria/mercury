//! Integration tests for the real `MercuryLocalStoreV1CryptoProvider`
//! (XChaCha20-Poly1305) bound into the existing local-store sealing seam.
//!
//! These exercise the properties that belong to *this* integration:
//! seal/open round-trip, a fresh nonce per seal, authenticated tamper
//! detection, AEAD associated-data binding of the record locator, and key
//! separation across generations. The underlying AEAD's spec conformance is
//! covered by the audited `chacha20poly1305` crate's own test vectors.

use mercury_core::{
    ComponentReasons, LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope,
    LocalStoreOpenRequest, LocalStoreOpenResult, LocalStoreRecordKind, LocalStoreRecordLocator,
    LocalStoreSealOutput, LocalStoreSealRequest, LocalStoreSealResult, LocalStoreSealingSuite,
    MercuryLocalStoreCryptoError, MercuryLocalStoreV1CryptoProvider, PolicyDecision,
    open_local_store_record, seal_local_store_plaintext,
};

const TEST_ROOT_KEY: [u8; 32] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32,
];

#[test]
fn real_provider_round_trips_local_plaintext() {
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(TEST_ROOT_KEY);
    let plaintext = b"mercury local store real ciphertext";
    let request = seal_request(
        LocalStoreRecordKind::ConversationSecret,
        plaintext.len() as i32,
        None,
    );

    let sealed = seal(&mut provider, request, plaintext);
    assert_eq!(sealed.nonce.len(), 24);
    assert_eq!(sealed.authentication_tag_len, 16);
    assert_eq!(sealed.sealed_bytes.len(), plaintext.len() + 16);
    assert_ne!(sealed.sealed_bytes, plaintext.to_vec());

    let open = LocalStoreOpenRequest::new(
        request,
        &sealed.nonce,
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );
    let opened = open_local_store_record(&mut provider, open).expect("open should succeed");
    assert_eq!(opened, LocalStoreOpenResult::Opened(plaintext.to_vec()));
}

#[test]
fn real_provider_round_trips_message_ciphertext_with_accepted_policy() {
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(TEST_ROOT_KEY);
    let plaintext = b"encrypted message object bytes";
    let request = seal_request(
        LocalStoreRecordKind::MessageCiphertext,
        plaintext.len() as i32,
        Some(policy_decision(true)),
    );

    let sealed = seal(&mut provider, request, plaintext);
    let open = LocalStoreOpenRequest::new(
        request,
        &sealed.nonce,
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );
    assert_eq!(
        open_local_store_record(&mut provider, open).expect("open should succeed"),
        LocalStoreOpenResult::Opened(plaintext.to_vec())
    );
}

#[test]
fn real_provider_uses_fresh_nonce_each_seal() {
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(TEST_ROOT_KEY);
    let plaintext = b"same plaintext sealed twice";
    let request = seal_request(
        LocalStoreRecordKind::ConversationSecret,
        plaintext.len() as i32,
        None,
    );

    let first = seal(&mut provider, request, plaintext);
    let second = seal(&mut provider, request, plaintext);

    assert_ne!(first.nonce, second.nonce);
    assert_ne!(first.sealed_bytes, second.sealed_bytes);
}

#[test]
fn real_provider_rejects_ciphertext_tamper() {
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(TEST_ROOT_KEY);
    let plaintext = b"authenticated payload";
    let request = seal_request(
        LocalStoreRecordKind::ConversationSecret,
        plaintext.len() as i32,
        None,
    );
    let sealed = seal(&mut provider, request, plaintext);

    let mut tampered = sealed.sealed_bytes.clone();
    tampered[0] ^= 0xff;
    let open = LocalStoreOpenRequest::new(
        request,
        &sealed.nonce,
        &tampered,
        sealed.authentication_tag_len,
    );

    assert_eq!(
        open_local_store_record(&mut provider, open),
        Err(MercuryLocalStoreCryptoError::OpenFailed)
    );
}

#[test]
fn real_provider_rejects_nonce_tamper() {
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(TEST_ROOT_KEY);
    let plaintext = b"authenticated payload";
    let request = seal_request(
        LocalStoreRecordKind::ConversationSecret,
        plaintext.len() as i32,
        None,
    );
    let sealed = seal(&mut provider, request, plaintext);

    let mut tampered_nonce = sealed.nonce.clone();
    tampered_nonce[0] ^= 0xff;
    let open = LocalStoreOpenRequest::new(
        request,
        &tampered_nonce,
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );

    assert_eq!(
        open_local_store_record(&mut provider, open),
        Err(MercuryLocalStoreCryptoError::OpenFailed)
    );
}

#[test]
fn real_provider_binds_ciphertext_to_record_locator() {
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(TEST_ROOT_KEY);
    let plaintext = b"bound to its slot";
    let sealed_request = seal_request_at(
        locator_for("record-42"),
        LocalStoreRecordKind::ConversationSecret,
        1,
        plaintext.len() as i32,
        None,
    );
    let sealed = seal(&mut provider, sealed_request, plaintext);

    // A different record locator yields different AEAD associated data, so the
    // ciphertext cannot be opened against the wrong record slot.
    let other_request = seal_request_at(
        locator_for("record-99"),
        LocalStoreRecordKind::ConversationSecret,
        1,
        plaintext.len() as i32,
        None,
    );
    let open = LocalStoreOpenRequest::new(
        other_request,
        &sealed.nonce,
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );

    assert_eq!(
        open_local_store_record(&mut provider, open),
        Err(MercuryLocalStoreCryptoError::OpenFailed)
    );
}

#[test]
fn real_provider_separates_keys_by_generation() {
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(TEST_ROOT_KEY);
    let plaintext = b"old generation ciphertext";
    let gen1 = seal_request_at(
        locator_for("record-42"),
        LocalStoreRecordKind::ConversationSecret,
        1,
        plaintext.len() as i32,
        None,
    );
    let sealed = seal(&mut provider, gen1, plaintext);

    // A rotated key generation derives a different subkey, so prior-generation
    // ciphertext does not open under the new generation.
    let gen2 = seal_request_at(
        locator_for("record-42"),
        LocalStoreRecordKind::ConversationSecret,
        2,
        plaintext.len() as i32,
        None,
    );
    let open = LocalStoreOpenRequest::new(
        gen2,
        &sealed.nonce,
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );

    assert_eq!(
        open_local_store_record(&mut provider, open),
        Err(MercuryLocalStoreCryptoError::OpenFailed)
    );
}

#[test]
fn real_provider_separates_keys_by_conversation_namespace() {
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(TEST_ROOT_KEY);
    let plaintext = b"conversation scoped ciphertext";
    let request_a = seal_request_at(
        LocalStoreRecordLocator::new("conversation-a", "record-1"),
        LocalStoreRecordKind::ConversationSecret,
        1,
        plaintext.len() as i32,
        None,
    );
    let sealed = seal(&mut provider, request_a, plaintext);

    // Same key scope/generation/record-id but a different conversation
    // namespace derives a different subkey, so the ciphertext does not open.
    let request_b = seal_request_at(
        LocalStoreRecordLocator::new("conversation-b", "record-1"),
        LocalStoreRecordKind::ConversationSecret,
        1,
        plaintext.len() as i32,
        None,
    );
    let open = LocalStoreOpenRequest::new(
        request_b,
        &sealed.nonce,
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );

    assert_eq!(
        open_local_store_record(&mut provider, open),
        Err(MercuryLocalStoreCryptoError::OpenFailed)
    );
}

fn seal(
    provider: &mut MercuryLocalStoreV1CryptoProvider,
    request: LocalStoreSealRequest<'_>,
    plaintext: &[u8],
) -> LocalStoreSealOutput {
    match seal_local_store_plaintext(provider, request, plaintext).expect("seal should succeed") {
        LocalStoreSealResult::Sealed(output) => output,
        LocalStoreSealResult::Rejected(decision) => {
            panic!("expected seal acceptance, got {:?}", decision.reason)
        }
    }
}

fn seal_request(
    record_kind: LocalStoreRecordKind,
    plaintext_len: i32,
    policy: Option<PolicyDecision>,
) -> LocalStoreSealRequest<'static> {
    seal_request_at(
        locator_for("record-42"),
        record_kind,
        1,
        plaintext_len,
        policy,
    )
}

fn seal_request_at(
    locator: LocalStoreRecordLocator<'static>,
    record_kind: LocalStoreRecordKind,
    generation: i32,
    plaintext_len: i32,
    policy: Option<PolicyDecision>,
) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        locator,
        record_kind,
        key_with_generation(record_kind.policy().key_scope, generation),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        plaintext_len,
        policy,
    )
}

fn locator_for(record_id: &'static str) -> LocalStoreRecordLocator<'static> {
    LocalStoreRecordLocator::new("conversation-7", record_id)
}

fn key_with_generation(scope: LocalStoreKeyScope, generation: i32) -> LocalStoreKeyDescriptor {
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
        generation,
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
