//! Stage 6: the relay's key-transparency directory service. A REAL AKD inclusion
//! proof transits the relay over HTTP and is verified CLIENT-SIDE against a
//! pinned VRF key. The relay is untrusted -- serving a proof confers no trust, so
//! a tampered checkpoint fails the client's verification.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use mercury_kt::{
    EpochHash, KeyTransparencyProofStatus, KtDirectory, SignedTreeHead, verify_consistency,
    verify_inclusion, verify_key_history, verify_signed_tree_head,
};
use mercury_relay::{
    ConsistencyResponse, InclusionResponse, KeyHistoryResponse, KtState, SignedTreeHeadResponse,
    hex, kt_router,
};
use tower::ServiceExt;

const LABEL: &str = "device:alice:a1f3";
const KEY: &[u8] = b"alice-device-key";

#[tokio::test]
async fn relay_serves_inclusion_proof_that_client_verifies() {
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY).await.unwrap();
    // Pin the VRF key OUT-OF-BAND -- never fetched from the untrusted relay.
    let pinned_pk = dir.public_key().await.unwrap();

    let app = kt_router(KtState::new(dir));
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/kt/inclusion/{LABEL}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: InclusionResponse = serde_json::from_slice(&bytes).unwrap();

    // Reconstruct the checkpoint from the wire and verify the served proof
    // against the PINNED key -- this client-side check is the trust boundary.
    let root = hex::decode(&body.root_hash).unwrap();
    let mut root_arr = [0u8; 32];
    root_arr.copy_from_slice(&root);
    let proof = body.proof;

    assert_eq!(
        verify_inclusion(
            &pinned_pk,
            &EpochHash(body.epoch, root_arr),
            LABEL,
            KEY,
            proof.clone()
        ),
        KeyTransparencyProofStatus::Verified,
    );

    // A tampered checkpoint root (what a malicious relay might forge) fails the
    // client check -- serving the proof did not confer trust.
    let mut tampered = root_arr;
    tampered[0] ^= 0xff;
    assert_eq!(
        verify_inclusion(
            &pinned_pk,
            &EpochHash(body.epoch, tampered),
            LABEL,
            KEY,
            proof
        ),
        KeyTransparencyProofStatus::Invalid,
    );
}

#[tokio::test]
async fn unknown_label_is_404() {
    // A label the directory never published has no proof; the relay returns an
    // opaque 404 and nothing else.
    let dir = KtDirectory::new().await.unwrap();
    let app = kt_router(KtState::new(dir));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/kt/inclusion/device:nobody:0000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn relay_serves_consistency_proof_that_client_verifies() {
    let mut dir = KtDirectory::new().await.unwrap();
    // Two publishes -> two epochs; pin both checkpoints out-of-band.
    let ep1 = dir.register(LABEL, KEY).await.unwrap();
    let ep2 = dir.register("device:bob:b2", b"bob-key").await.unwrap();

    let app = kt_router(KtState::new(dir));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/kt/consistency?from=1&to=2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: ConsistencyResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!((body.from_epoch, body.to_epoch), (1, 2));

    // Verify the served append-only proof against the PINNED checkpoints.
    assert_eq!(
        verify_consistency(&[ep1, ep2], body.proof).await,
        KeyTransparencyProofStatus::Verified,
    );
}

#[tokio::test]
async fn relay_serves_key_history_that_client_verifies() {
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY).await.unwrap();
    dir.register(LABEL, b"alice-key-v2").await.unwrap(); // rotation
    let pinned_pk = dir.public_key().await.unwrap();

    let app = kt_router(KtState::new(dir));
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/kt/history/{LABEL}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: KeyHistoryResponse = serde_json::from_slice(&bytes).unwrap();

    let root = hex::decode(&body.root_hash).unwrap();
    let mut root_arr = [0u8; 32];
    root_arr.copy_from_slice(&root);

    assert_eq!(
        verify_key_history(
            &pinned_pk,
            &EpochHash(body.epoch, root_arr),
            LABEL,
            body.proof
        ),
        KeyTransparencyProofStatus::Verified,
    );
}

#[tokio::test]
async fn relay_serves_log_signed_tree_head_that_client_verifies() {
    let mut dir = KtDirectory::new().await.unwrap();
    dir.register(LABEL, KEY).await.unwrap(); // epoch 1 -> a head exists
    // Pin the LOG key OUT-OF-BAND (distinct from the VRF key; never from the relay).
    let pinned_log_key = dir.log_public_key();

    let app = kt_router(KtState::new(dir));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/kt/sth")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: SignedTreeHeadResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.tree_size, 1);

    let mut root = [0u8; 32];
    root.copy_from_slice(&hex::decode(&body.root_hash).unwrap());
    let mut sig = [0u8; 64];
    sig.copy_from_slice(&hex::decode(&body.log_signature).unwrap());
    let sth = SignedTreeHead {
        tree_size: body.tree_size,
        root_hash: root,
        timestamp_s: body.timestamp_s,
    };

    // The served head verifies against the PINNED log key...
    assert!(verify_signed_tree_head(&pinned_log_key, &sth, &sig));

    // ...and a tampered head (forged root) does not -- serving conferred no trust.
    let mut forged = sth;
    forged.root_hash[0] ^= 0xff;
    assert!(!verify_signed_tree_head(&pinned_log_key, &forged, &sig));
}

#[tokio::test]
async fn sth_for_empty_directory_is_a_verifiable_epoch_zero_head() {
    // An empty directory still has a valid (empty) tree at epoch 0; the log signs
    // and serves that head, and it verifies against the pinned log key. Serving a
    // truthful empty head is correct -- there is nothing to hide or 404.
    let dir = KtDirectory::new().await.unwrap();
    let pinned_log_key = dir.log_public_key();

    let app = kt_router(KtState::new(dir));
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/kt/sth")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: SignedTreeHeadResponse = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body.tree_size, 0, "empty directory is at epoch 0");

    let mut root = [0u8; 32];
    root.copy_from_slice(&hex::decode(&body.root_hash).unwrap());
    let mut sig = [0u8; 64];
    sig.copy_from_slice(&hex::decode(&body.log_signature).unwrap());
    let sth = SignedTreeHead {
        tree_size: body.tree_size,
        root_hash: root,
        timestamp_s: body.timestamp_s,
    };
    assert!(verify_signed_tree_head(&pinned_log_key, &sth, &sig));
}
