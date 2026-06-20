//! The username registry, driven through the KT Axum router. A claim is committed to the real AKD
//! log; a lookup is served with a REAL inclusion proof that is verified CLIENT-SIDE against the
//! directory's VRF key — exactly as a client would. The relay is untrusted: a tampered binding or
//! checkpoint fails the client's `verify_inclusion`, and first-claimant-wins is enforced.

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
use mercury_relay::{KtState, UsernameLookupResponse, VrfKeyResponse, hex, kt_router};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn build() -> Router {
    let dir = KtDirectory::new().await.unwrap();
    kt_router(KtState::new(dir))
}

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

#[tokio::test]
async fn claim_then_lookup_proof_verifies_client_side() {
    let app = build().await;
    let alice = IdentityKeyPair::generate();
    let alice_id = account_id_from_identity_key(&alice.public());

    // Claim "Alice" (mixed case; the relay normalizes to "alice").
    let (status, body) =
        post_json(app.clone(), "/username/claim", claim_body(&alice, "Alice")).await;
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "claim accepted: {status}"
    );
    assert_eq!(body["created"], json!(true));
    assert_eq!(body["username"], json!("alice"));

    // Pin the directory VRF key the way a client does on first contact: GET /kt/vrf-key.
    let (status, bytes) = raw_get(app.clone(), "/kt/vrf-key").await;
    assert_eq!(status, StatusCode::OK);
    let vrf: VrfKeyResponse = serde_json::from_slice(&bytes).unwrap();
    let pinned_vrf = hex::decode(&vrf.vrf_public_key).unwrap();

    // Look up "alice" -> account id + inclusion proof.
    let (status, bytes) = raw_get(app, "/username/alice").await;
    assert_eq!(status, StatusCode::OK);
    let lookup: UsernameLookupResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(hex::decode(&lookup.account_id).unwrap(), alice_id);

    // The client reconstructs the checkpoint and verifies the proof binds username:alice -> alice_id
    // against the PINNED VRF key. This is the trust boundary.
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

    // A relay that LIED about the account id (claimed alice -> some attacker id) fails the client
    // check: the proof only verifies for the value actually committed to the log.
    let attacker_id = [0xabu8; 32];
    assert_eq!(
        verify_inclusion(
            &pinned_vrf,
            &EpochHash(lookup.epoch, root),
            "username:alice",
            &attacker_id,
            lookup.proof,
        ),
        KeyTransparencyProofStatus::Invalid,
    );
}

#[tokio::test]
async fn first_claimant_wins() {
    let app = build().await;
    let alice = IdentityKeyPair::generate();
    let mallory = IdentityKeyPair::generate();

    let (status, _) = post_json(
        app.clone(),
        "/username/claim",
        claim_body(&alice, "handle1"),
    )
    .await;
    assert!(status == StatusCode::OK || status == StatusCode::CREATED);

    // A different account cannot take the claimed handle.
    let (status, _) = post_json(
        app.clone(),
        "/username/claim",
        claim_body(&mallory, "handle1"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // The original owner re-confirming is idempotent (created:false), not a conflict.
    let (status, body) = post_json(app, "/username/claim", claim_body(&alice, "handle1")).await;
    assert!(status == StatusCode::OK || status == StatusCode::CREATED);
    assert_eq!(body["created"], json!(false));
}

#[tokio::test]
async fn claim_with_a_bad_proof_is_forbidden() {
    let app = build().await;
    let alice = IdentityKeyPair::generate();
    // Sign a claim for "realname" but submit it for "fakename" — the proof must not verify.
    let sig = sign_username_claim(&alice, "realname");
    let body = json!({
        "identity_pub": hex::encode(alice.public().as_bytes()),
        "username": "fakename",
        "pop_sig": hex::encode(&sig),
    });
    let (status, _) = post_json(app, "/username/claim", body).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn illegal_username_is_rejected() {
    let app = build().await;
    let alice = IdentityKeyPair::generate();
    // "ab" is too short; normalize rejects it, so the relay 400s before any crypto.
    let body = json!({
        "identity_pub": hex::encode(alice.public().as_bytes()),
        "username": "ab",
        "pop_sig": hex::encode(&[0u8; 64]),
    });
    let (status, _) = post_json(app, "/username/claim", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unknown_username_is_404() {
    let app = build().await;
    let (status, _) = raw_get(app, "/username/ghost").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// Regression: the KT router was previously merged WITHOUT any rate-limit layer, leaving the
// epoch-advancing /username/claim uncapped per source. It must now flood-limit. The cloned router
// shares the same Arc<RateLimiter>, and in-process there's no ConnectInfo so client_key collapses
// to one shared key — so the cap applies across these requests. The limiter runs as a layer BEFORE
// the handler, so it counts every request (each here 400s on a deliberately-bad body) and rejects
// over the cap with 429 before any Ed25519 verify / AKD epoch advance.
#[tokio::test]
async fn kt_username_claim_is_flood_limited_per_source() {
    let app = build().await;
    let bad = json!({ "username": "x", "identity_pub": "00", "pop_sig": "00" });
    // KT_CLAIM_MAX_PER_WINDOW is 8: the first 8 are under the cap, the 9th is over it.
    for i in 0..8 {
        let (status, _) = post_json(app.clone(), "/username/claim", bad.clone()).await;
        assert_ne!(
            status,
            StatusCode::TOO_MANY_REQUESTS,
            "request {i} is under the cap and must not be rate-limited"
        );
    }
    let (status, _) = post_json(app.clone(), "/username/claim", bad.clone()).await;
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "the 9th claim from one source must be flood-limited (429)"
    );
}
