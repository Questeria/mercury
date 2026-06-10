//! Property tests for the real `MercuryLocalStoreV1CryptoProvider`
//! (XChaCha20-Poly1305) exercised through the public sealing seam.
//!
//! These complement the fixed-vector integration tests in
//! `local_store_crypto_real_provider.rs` by sweeping random root keys and
//! random plaintext lengths: seal/open round-trips, authenticated tamper
//! detection on both the ciphertext and the nonce, and key separation.

use mercury_core::{
    LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope, LocalStoreOpenRequest,
    LocalStoreOpenResult, LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreSealOutput,
    LocalStoreSealRequest, LocalStoreSealResult, LocalStoreSealingSuite,
    MercuryLocalStoreCryptoError, MercuryLocalStoreV1CryptoProvider, open_local_store_record,
    seal_local_store_plaintext,
};
use proptest::prelude::*;

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

fn seal_request(plaintext_len: i32) -> LocalStoreSealRequest<'static> {
    let scope = LocalStoreKeyScope::Conversation;
    let key = LocalStoreKeyDescriptor::new(
        scope,
        LocalStoreSealingSuite::MercuryLocalStoreV1,
        1,
        LocalStoreKeyBinding::conversation(32, 32),
    );
    LocalStoreSealRequest::new(
        LocalStoreRecordLocator::new("conversation-7", "record-42"),
        LocalStoreRecordKind::ConversationSecret,
        key,
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        plaintext_len,
        None,
    )
}

proptest! {
    /// Property (a): seal-then-open round-trips for a random root key and
    /// random plaintext, recovering the exact bytes.
    #[test]
    fn seal_then_open_round_trips(
        root_key in any::<[u8; 32]>(),
        plaintext in proptest::collection::vec(any::<u8>(), 1..=4096),
    ) {
        let mut provider = MercuryLocalStoreV1CryptoProvider::new(root_key);
        let request = seal_request(plaintext.len() as i32);
        let sealed = seal(&mut provider, request, &plaintext);

        let open = LocalStoreOpenRequest::new(
            request,
            &sealed.nonce,
            &sealed.sealed_bytes,
            sealed.authentication_tag_len,
        );
        prop_assert_eq!(
            open_local_store_record(&mut provider, open).expect("open should succeed"),
            LocalStoreOpenResult::Opened(plaintext)
        );
    }

    /// Property (b): flipping any one byte of the sealed bytes fails the AEAD
    /// authentication, so open returns `Err(OpenFailed)`.
    #[test]
    fn flipped_ciphertext_byte_fails_open(
        root_key in any::<[u8; 32]>(),
        plaintext in proptest::collection::vec(any::<u8>(), 1..=4096),
        flip_mask in 1u8..=u8::MAX,
        // index selector resolved against the sealed length below
        idx_seed in any::<usize>(),
    ) {
        let mut provider = MercuryLocalStoreV1CryptoProvider::new(root_key);
        let request = seal_request(plaintext.len() as i32);
        let sealed = seal(&mut provider, request, &plaintext);

        let mut tampered = sealed.sealed_bytes.clone();
        let idx = idx_seed % tampered.len();
        tampered[idx] ^= flip_mask; // nonzero mask guarantees a real change

        let open = LocalStoreOpenRequest::new(
            request,
            &sealed.nonce,
            &tampered,
            sealed.authentication_tag_len,
        );
        prop_assert_eq!(
            open_local_store_record(&mut provider, open),
            Err(MercuryLocalStoreCryptoError::OpenFailed)
        );
    }

    /// Property (b'): flipping any one byte of the nonce fails the AEAD
    /// authentication, so open returns `Err(OpenFailed)`.
    #[test]
    fn flipped_nonce_byte_fails_open(
        root_key in any::<[u8; 32]>(),
        plaintext in proptest::collection::vec(any::<u8>(), 1..=4096),
        flip_mask in 1u8..=u8::MAX,
        idx_seed in any::<usize>(),
    ) {
        let mut provider = MercuryLocalStoreV1CryptoProvider::new(root_key);
        let request = seal_request(plaintext.len() as i32);
        let sealed = seal(&mut provider, request, &plaintext);

        let mut tampered_nonce = sealed.nonce.clone();
        let idx = idx_seed % tampered_nonce.len();
        tampered_nonce[idx] ^= flip_mask;

        let open = LocalStoreOpenRequest::new(
            request,
            &tampered_nonce,
            &sealed.sealed_bytes,
            sealed.authentication_tag_len,
        );
        prop_assert_eq!(
            open_local_store_record(&mut provider, open),
            Err(MercuryLocalStoreCryptoError::OpenFailed)
        );
    }

    /// Property (c): opening with a different 32-byte root key derives a
    /// different subkey, so open returns `Err(OpenFailed)`.
    #[test]
    fn different_root_key_fails_open(
        root_key in any::<[u8; 32]>(),
        other_key in any::<[u8; 32]>(),
        plaintext in proptest::collection::vec(any::<u8>(), 1..=4096),
    ) {
        prop_assume!(root_key != other_key);
        let mut provider = MercuryLocalStoreV1CryptoProvider::new(root_key);
        let request = seal_request(plaintext.len() as i32);
        let sealed = seal(&mut provider, request, &plaintext);

        let mut other = MercuryLocalStoreV1CryptoProvider::new(other_key);
        let open = LocalStoreOpenRequest::new(
            request,
            &sealed.nonce,
            &sealed.sealed_bytes,
            sealed.authentication_tag_len,
        );
        prop_assert_eq!(
            open_local_store_record(&mut other, open),
            Err(MercuryLocalStoreCryptoError::OpenFailed)
        );
    }
}
