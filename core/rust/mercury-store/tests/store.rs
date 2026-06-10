//! End-to-end tests for the redb-backed production local store.
//!
//! Every test drives the store through the **existing** mercury-core gates
//! (keychain-unlock -> unlock -> production-open) and the gate-checked
//! `*_production_local_store_*` free functions. Records are sealed by the real
//! `MercuryLocalStoreV1CryptoProvider`. Nothing bypasses the gates, and the dev
//! keychain backend means no OS keychain is ever touched.

use mercury_core::{
    LocalStoreKeyBinding, LocalStoreKeyDescriptor, LocalStoreKeyScope, LocalStoreOpenRequest,
    LocalStoreOpenResult, LocalStoreProductionOpenReason, LocalStoreRecordKind,
    LocalStoreRecordLocator, LocalStoreSealOutput, LocalStoreSealRequest, LocalStoreSealResult,
    LocalStoreSealingSuite, MercuryLocalStoreV1CryptoProvider,
    build_sealed_local_store_write_request, open_local_store_record, open_production_local_store,
    put_production_local_store_record, read_production_local_store_record,
    seal_local_store_plaintext,
};
use mercury_store::{HeaderParams, MercuryKeychainAdapter, MercuryRedbStore};
use tempfile::TempDir;

const ROOT_KEY: [u8; 32] = [7u8; 32];
const NAMESPACE: &str = "device-namespace";
const RECORD_ID: &str = "device-secret-1";
const PLAINTEXT: &[u8] = b"the-device-secret-plaintext-payload";

/// A `DeviceLocal`-scoped key descriptor whose binding satisfies the sealing
/// gate (account + device id lengths both positive).
fn device_key() -> LocalStoreKeyDescriptor {
    LocalStoreKeyDescriptor::new(
        LocalStoreKeyScope::DeviceLocal,
        LocalStoreSealingSuite::MercuryLocalStoreV1,
        1,
        LocalStoreKeyBinding::device(16, 16),
    )
}

fn seal_request(plaintext_len: usize) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        LocalStoreRecordLocator::new(NAMESPACE, RECORD_ID),
        LocalStoreRecordKind::DeviceSecret,
        device_key(),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        i32::try_from(plaintext_len).unwrap(),
        None,
    )
}

/// Seal `PLAINTEXT` with the real provider, returning the sealing output.
fn seal_plaintext(provider: &mut MercuryLocalStoreV1CryptoProvider) -> LocalStoreSealOutput {
    match seal_local_store_plaintext(provider, seal_request(PLAINTEXT.len()), PLAINTEXT)
        .expect("seal should not error")
    {
        LocalStoreSealResult::Sealed(output) => output,
        LocalStoreSealResult::Rejected(decision) => panic!("seal rejected: {decision:?}"),
    }
}

/// Drive keychain-unlock -> unlock -> production-open and assert acceptance,
/// then call the gate-checked open free function on the store.
fn open_through_gates(store: &mut MercuryRedbStore, keychain: &MercuryKeychainAdapter) {
    let keychain_decision = keychain.keychain_unlock_input().evaluate();
    assert!(keychain_decision.accepted, "keychain gate must accept");

    let unlock_decision = keychain_decision.unlock_input.evaluate();
    assert!(unlock_decision.accepted, "unlock gate must accept");

    let open_input = store.header_open_input(keychain_decision.unlock_input);
    let open_decision =
        open_production_local_store(store, open_input).expect("open should not error");
    assert!(open_decision.accepted, "production-open gate must accept");
    assert_eq!(
        open_decision.reason,
        LocalStoreProductionOpenReason::Accepted
    );
}

#[test]
fn end_to_end_seal_put_read_open_recovers_plaintext() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.redb");
    let keychain = MercuryKeychainAdapter::dev_memory(ROOT_KEY);
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(keychain.root_key().unwrap());

    let mut store = MercuryRedbStore::open_or_create(&path, HeaderParams::v1()).unwrap();
    open_through_gates(&mut store, &keychain);

    // Seal, then write through the gate-checked free function.
    let sealed = seal_plaintext(&mut provider);
    let write_request =
        build_sealed_local_store_write_request(seal_request(PLAINTEXT.len()), &sealed.sealed_bytes)
            .expect("sealed write request should build");
    let write_decision =
        put_production_local_store_record(&mut store, write_request).expect("put should not error");
    assert!(write_decision.accepted, "gate-checked write must accept");

    // Read back through the gate-checked free function.
    let locator = LocalStoreRecordLocator::new(NAMESPACE, RECORD_ID);
    let read = read_production_local_store_record(&store, locator)
        .expect("read should not error")
        .expect("record should exist");
    assert_eq!(read.record_kind, LocalStoreRecordKind::DeviceSecret);
    assert_eq!(read.bytes, sealed.sealed_bytes);

    // Open with the provider -> recovers the EXACT plaintext.
    let open_request = LocalStoreOpenRequest::new(
        seal_request(PLAINTEXT.len()),
        &sealed.nonce,
        &read.bytes,
        sealed.authentication_tag_len,
    );
    let opened =
        open_local_store_record(&mut provider, open_request).expect("open should not error");
    match opened {
        LocalStoreOpenResult::Opened(plaintext) => assert_eq!(plaintext, PLAINTEXT),
        LocalStoreOpenResult::Rejected(decision) => panic!("open rejected: {decision:?}"),
    }
}

#[test]
fn record_persists_across_drop_and_reopen() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.redb");
    let keychain = MercuryKeychainAdapter::dev_memory(ROOT_KEY);
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(keychain.root_key().unwrap());
    let sealed = seal_plaintext(&mut provider);

    {
        let mut store = MercuryRedbStore::open_or_create(&path, HeaderParams::v1()).unwrap();
        open_through_gates(&mut store, &keychain);
        let write_request = build_sealed_local_store_write_request(
            seal_request(PLAINTEXT.len()),
            &sealed.sealed_bytes,
        )
        .unwrap();
        put_production_local_store_record(&mut store, write_request).unwrap();
        // store dropped here -> redb commits are durable.
    }

    let mut store = MercuryRedbStore::open_or_create(&path, HeaderParams::v1()).unwrap();
    open_through_gates(&mut store, &keychain);
    let locator = LocalStoreRecordLocator::new(NAMESPACE, RECORD_ID);
    let read = read_production_local_store_record(&store, locator)
        .unwrap()
        .expect("record should survive reopen");
    // Still sealed (equals ciphertext, not plaintext).
    assert_eq!(read.bytes, sealed.sealed_bytes);
    assert_ne!(read.bytes.as_slice(), PLAINTEXT);
}

#[test]
fn stored_bytes_are_sealed_not_plaintext() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.redb");
    let keychain = MercuryKeychainAdapter::dev_memory(ROOT_KEY);
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(keychain.root_key().unwrap());
    let sealed = seal_plaintext(&mut provider);

    let mut store = MercuryRedbStore::open_or_create(&path, HeaderParams::v1()).unwrap();
    open_through_gates(&mut store, &keychain);
    let write_request =
        build_sealed_local_store_write_request(seal_request(PLAINTEXT.len()), &sealed.sealed_bytes)
            .unwrap();
    put_production_local_store_record(&mut store, write_request).unwrap();
    drop(store);

    // The plaintext must not appear anywhere in the on-disk redb file.
    let raw = std::fs::read(&path).unwrap();
    assert!(
        !contains_subslice(&raw, PLAINTEXT),
        "plaintext leaked into the on-disk store"
    );
    // The sealed ciphertext (what we actually persisted) is present.
    assert!(
        contains_subslice(&raw, &sealed.sealed_bytes),
        "sealed ciphertext should be present on disk"
    );
}

#[test]
fn header_open_input_accepted_by_production_open_gate() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.redb");
    let keychain = MercuryKeychainAdapter::dev_memory(ROOT_KEY);

    let store = MercuryRedbStore::open_or_create(&path, HeaderParams::v1()).unwrap();
    let keychain_decision = keychain.keychain_unlock_input().evaluate();
    let open_input = store.header_open_input(keychain_decision.unlock_input);
    let decision = open_input.evaluate();
    assert!(decision.accepted);
    assert_eq!(decision.reason, LocalStoreProductionOpenReason::Accepted);
}

#[test]
fn wrong_suite_header_rejected_by_production_open_gate() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.redb");
    let keychain = MercuryKeychainAdapter::dev_memory(ROOT_KEY);

    // Force a header with an unknown suite code.
    let mut bad_header = HeaderParams::v1();
    bad_header.suite_code = 999;
    let store = MercuryRedbStore::open_or_create(&path, bad_header).unwrap();
    assert_eq!(store.header().suite_code, 999);

    let keychain_decision = keychain.keychain_unlock_input().evaluate();
    let open_input = store.header_open_input(keychain_decision.unlock_input);
    let decision = open_input.evaluate();
    assert!(!decision.accepted, "wrong suite must be rejected");
    assert_eq!(
        decision.reason,
        LocalStoreProductionOpenReason::HeaderSuiteMismatch
    );
}

#[test]
fn wrong_magic_header_input_rejected_by_production_open_gate() {
    // The store always reports magic-matches=true for a real file, so we model
    // a corrupt-magic open by constructing the input directly with the store's
    // manifest fields but header_magic_matches=false (what a caller would build
    // if the on-disk magic failed to verify).
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.redb");
    let keychain = MercuryKeychainAdapter::dev_memory(ROOT_KEY);
    let store = MercuryRedbStore::open_or_create(&path, HeaderParams::v1()).unwrap();

    let keychain_decision = keychain.keychain_unlock_input().evaluate();
    let mut open_input = store.header_open_input(keychain_decision.unlock_input);
    open_input.header_magic_matches = false;
    let decision = open_input.evaluate();
    assert!(!decision.accepted, "wrong magic must be rejected");
    assert_eq!(
        decision.reason,
        LocalStoreProductionOpenReason::HeaderMagicMismatch
    );
}

#[test]
fn delete_removes_record() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("store.redb");
    let keychain = MercuryKeychainAdapter::dev_memory(ROOT_KEY);
    let mut provider = MercuryLocalStoreV1CryptoProvider::new(keychain.root_key().unwrap());
    let sealed = seal_plaintext(&mut provider);

    let mut store = MercuryRedbStore::open_or_create(&path, HeaderParams::v1()).unwrap();
    open_through_gates(&mut store, &keychain);
    let write_request =
        build_sealed_local_store_write_request(seal_request(PLAINTEXT.len()), &sealed.sealed_bytes)
            .unwrap();
    put_production_local_store_record(&mut store, write_request).unwrap();

    let locator = LocalStoreRecordLocator::new(NAMESPACE, RECORD_ID);
    assert!(
        read_production_local_store_record(&store, locator)
            .unwrap()
            .is_some()
    );

    use mercury_core::EncryptedLocalStoreAdapter;
    store.delete_record(locator).unwrap();
    assert!(
        read_production_local_store_record(&store, locator)
            .unwrap()
            .is_none(),
        "deleted record must read back as None"
    );
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}
