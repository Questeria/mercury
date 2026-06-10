//! Durable username storage: claimed handles survive a relay restart. The store is exercised
//! directly (round trip + fail-closed corruption handling), through the registry (write-through
//! claims), and end-to-end through the KT Axum router across a simulated restart — "boot 2"
//! rebuilds a fresh AKD log from the persisted bindings and the served inclusion proof is
//! verified CLIENT-SIDE against the VRF key pinned during "boot 1" (the key is stable because
//! both boots derive it from the same persisted seed; only the log contents are rebuilt).
//!
//! Deliberately NOT tested: the `ClaimResult::PersistFailed` branch itself. Forcing a redb
//! write failure portably (full disk / yanked volume) is not possible from safe, deterministic
//! test code, and faking it (e.g. a mock store) would test the mock, not the code. The branch
//! is fail-closed BY CONSTRUCTION: `UsernameRegistry::claim` calls `durable.put()` BEFORE the
//! in-memory insert and returns `PersistFailed` without inserting on `Err` — there is no code
//! path that mutates the map when the disk write fails (see `src/username.rs`). The corrupt-row
//! test below covers the other fail-closed direction (a bad row is an error at load, never
//! truncated into a "valid" account id).

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use mercury_keys::{
    IdentityKeyPair, account_id_from_identity_key, normalize_username, sign_username_claim,
};
use mercury_kt::{EpochHash, KeyTransparencyProofStatus, KtDirectory, verify_inclusion};
use mercury_relay::{
    ClaimResult, DurableUsernameStore, KtState, UsernameLookupResponse, UsernameRegistry,
    VrfKeyResponse, hex, kt_router, kt_username_label,
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn post_json(app: Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn raw_get(app: Router, uri: &str) -> (StatusCode, Vec<u8>) {
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, bytes)
}

fn claim_body(id: &IdentityKeyPair, username: &str) -> Value {
    let name = normalize_username(username).unwrap();
    let sig = sign_username_claim(id, &name);
    json!({
        "identity_pub": hex::encode(id.public().as_bytes()),
        "username": username,
        "pop_sig": hex::encode(&sig),
    })
}

#[test]
fn store_round_trip_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("usernames.redb");

    {
        let store = DurableUsernameStore::open(&path).unwrap();
        store.put("alice", &[1u8; 32]).unwrap();
        store.put("bob", &[2u8; 32]).unwrap();
        assert_eq!(store.load_all().unwrap().len(), 2);
    } // store dropped -> db file closed (the "restart")

    let store = DurableUsernameStore::open(&path).unwrap();
    let loaded = store.load_all().unwrap();
    // redb iterates &str keys in order, so the result is deterministic.
    assert_eq!(
        loaded,
        vec![
            ("alice".to_string(), [1u8; 32]),
            ("bob".to_string(), [2u8; 32]),
        ]
    );
}

#[test]
fn a_corrupt_account_id_width_is_an_error_not_a_truncation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("usernames.redb");

    // Plant a 31-byte value directly in the same redb table the store uses — simulating
    // on-disk corruption the public API can never produce (put() only accepts [u8; 32]).
    {
        const USERNAMES: redb::TableDefinition<&str, &[u8]> =
            redb::TableDefinition::new("mercury_usernames_v1");
        let db = redb::Database::create(&path).unwrap();
        let txn = db.begin_write().unwrap();
        {
            let mut table = txn.open_table(USERNAMES).unwrap();
            table.insert("mallory", [9u8; 31].as_slice()).unwrap();
        }
        txn.commit().unwrap();
    }

    // load_all must FAIL CLOSED: an error, never a truncated/padded "account id".
    let store = DurableUsernameStore::open(&path).unwrap();
    assert!(store.load_all().is_err());
    // And the registry refuses to boot over a corrupt store, for the same reason.
    assert!(UsernameRegistry::with_durable(store).is_err());
}

#[test]
fn a_claim_written_through_the_registry_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("usernames.redb");
    let alice = [1u8; 32];

    {
        let store = DurableUsernameStore::open(&path).unwrap();
        let mut reg = UsernameRegistry::with_durable(store).unwrap();
        assert_eq!(reg.claim("alice", alice), ClaimResult::Created);
        assert_eq!(reg.lookup("alice"), Some(alice));
    } // registry + store dropped (the "restart")

    // A fresh registry over the same file finds the binding, and first-claimant-wins
    // still holds across the restart: another account cannot take the handle.
    let store = DurableUsernameStore::open(&path).unwrap();
    let mut reg = UsernameRegistry::with_durable(store).unwrap();
    assert_eq!(reg.lookup("alice"), Some(alice));
    assert_eq!(reg.claim("alice", [2u8; 32]), ClaimResult::Taken);
    assert_eq!(reg.claim("alice", alice), ClaimResult::AlreadyOwned);
    assert_eq!(reg.len(), 1);
}

/// The full restart story, end-to-end through the HTTP router: a handle claimed in "boot 1"
/// is still resolvable in "boot 2" with an inclusion proof that verifies CLIENT-SIDE against
/// the VRF key pinned in boot 1 — exactly what a real client across a real relay restart sees.
#[tokio::test]
async fn a_claim_survives_a_restart_and_the_rebuilt_proof_verifies_client_side() {
    const VRF_SEED: [u8; 32] = [7u8; 32]; // production: persisted via MERCURY_KT_VRF_SEED
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("usernames.redb");
    let alice = IdentityKeyPair::generate();
    let alice_id = account_id_from_identity_key(&alice.public());

    // ---- Boot 1: claim "alice" through the router; pin the VRF key as a client would. ----
    let pinned_vrf = {
        let kt_dir = KtDirectory::with_vrf_seed(VRF_SEED).await.unwrap();
        let store = DurableUsernameStore::open(&path).unwrap();
        let registry = UsernameRegistry::with_durable(store).unwrap();
        let app = kt_router(KtState::with_registry(kt_dir, registry));

        let (status, body) =
            post_json(app.clone(), "/username/claim", claim_body(&alice, "Alice")).await;
        assert!(
            status == StatusCode::OK || status == StatusCode::CREATED,
            "claim accepted: {status}"
        );
        assert_eq!(body["created"], json!(true));

        let (status, bytes) = raw_get(app, "/kt/vrf-key").await;
        assert_eq!(status, StatusCode::OK);
        let vrf: VrfKeyResponse = serde_json::from_slice(&bytes).unwrap();
        hex::decode(&vrf.vrf_public_key).unwrap()
    }; // router, registry, store, directory all dropped — the relay "restarts"

    // ---- Boot 2: fresh directory from the SAME seed; rebuild the log from the store. ----
    let mut kt_dir = KtDirectory::with_vrf_seed(VRF_SEED).await.unwrap();
    let store = DurableUsernameStore::open(&path).unwrap();
    let bindings = store.load_all().unwrap();
    assert_eq!(bindings.len(), 1, "the claim survived the restart on disk");
    // Re-register every persisted binding in ONE epoch (what the server binary does at boot).
    let updates: Vec<(String, Vec<u8>)> = bindings
        .iter()
        .map(|(name, id)| (kt_username_label(name), id.to_vec()))
        .collect();
    kt_dir.register_batch(&updates).await.unwrap();
    let registry = UsernameRegistry::with_durable(store).unwrap();
    let app = kt_router(KtState::with_registry(kt_dir, registry));

    // The handle resolves, and the client-side proof check passes under the key pinned in
    // boot 1 — the restart is invisible to per-lookup verification.
    let (status, bytes) = raw_get(app, "/username/alice").await;
    assert_eq!(status, StatusCode::OK);
    let lookup: UsernameLookupResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(hex::decode(&lookup.account_id).unwrap(), alice_id);
    let mut root = [0u8; 32];
    root.copy_from_slice(&hex::decode(&lookup.root_hash).unwrap());
    assert_eq!(
        verify_inclusion(
            &pinned_vrf,
            &EpochHash(lookup.epoch, root),
            "username:alice",
            &alice_id,
            lookup.proof.clone(),
        ),
        KeyTransparencyProofStatus::Verified,
    );

    // And a relay that lies about the rebuilt binding still fails the client check.
    assert_eq!(
        verify_inclusion(
            &pinned_vrf,
            &EpochHash(lookup.epoch, root),
            "username:alice",
            &[0xabu8; 32],
            lookup.proof,
        ),
        KeyTransparencyProofStatus::Invalid,
    );
}
