//! KT witness-cosignature serving (the split-view-detection rails). A pinned witness co-signs the
//! relay's CURRENT published head; the relay verifies + stores it and serves it in the witnessed
//! bundle, so a client can verify the cosignature against the witness key it pinned out-of-band.
//!
//! Honest scope: this provides the RAILS. Real split-view resistance additionally needs the
//! witnesses to actually be DEPLOYED + submitting, AND a client policy that requires a quorum
//! (`kt_witness_status`, which also wants an auditor signature + >=2 independent operators — the
//! full gate is exercised in mercury-kt's witness tests). The relay cannot manufacture the
//! protection; it only collects + serves what real witnesses sign.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use ed25519_dalek::{Signer, SigningKey};
use http_body_util::BodyExt;
use mercury_kt::{KtDirectory, SignedTreeHead, Witness, verify_witness_cosignature};
use mercury_relay::{KtState, WitnessedSthResponse, hex, kt_router};
use serde_json::{Value, json};
use tower::ServiceExt;

const LOG_SEED: [u8; 32] = [5u8; 32];
const WITNESS_SEED: [u8; 32] = [3u8; 32];

fn witness_signing_key() -> SigningKey {
    SigningKey::from_bytes(&WITNESS_SEED)
}

async fn build_with_witness() -> (Router, SigningKey) {
    let wk = witness_signing_key();
    let witness = Witness::from_ed25519_bytes(0, &wk.verifying_key().to_bytes()).unwrap();
    let dir = KtDirectory::with_vrf_seed(LOG_SEED).await.unwrap();
    (kt_router(KtState::new(dir).with_witnesses(vec![witness])), wk)
}

async fn get_json<T: serde::de::DeserializeOwned>(app: Router, uri: &str) -> T {
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "GET {uri}");
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn post_json(app: Router, uri: &str, body: Value) -> StatusCode {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap(),
    )
    .await
    .unwrap()
    .status()
}

fn decode32(h: &str) -> [u8; 32] {
    <[u8; 32]>::try_from(hex::decode(h).unwrap().as_slice()).unwrap()
}
fn decode64(h: &str) -> [u8; 64] {
    <[u8; 64]>::try_from(hex::decode(h).unwrap().as_slice()).unwrap()
}

fn head_of(r: &WitnessedSthResponse) -> SignedTreeHead {
    SignedTreeHead {
        tree_size: r.tree_size,
        root_hash: decode32(&r.root_hash),
        timestamp_s: r.timestamp_s,
    }
}

fn cosign_body(r: &WitnessedSthResponse, witness_index: usize, sig: &[u8; 64]) -> Value {
    json!({
        "tree_size": r.tree_size,
        "root_hash": r.root_hash,
        "timestamp_s": r.timestamp_s,
        "witness_index": witness_index,
        "signature": hex::encode(sig),
    })
}

#[tokio::test]
async fn witness_cosignature_is_collected_and_served_verifiably() {
    let (app, wk) = build_with_witness().await;

    // The published head starts with no cosignatures and advertises the one pinned witness.
    let head: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    assert!(head.cosignatures.is_empty(), "no cosignatures yet");
    assert_eq!(head.witnesses.len(), 1, "the pinned witness is advertised");
    assert!(!head.log_signature.is_empty(), "the log signature is served");

    // The witness reconstructs the canonical head bytes and co-signs them (as a deployed witness would).
    let sth = head_of(&head);
    let sig = wk.sign(&sth.signing_bytes()).to_bytes();
    assert_eq!(
        post_json(app.clone(), "/kt/witness/cosign", cosign_body(&head, 0, &sig)).await,
        StatusCode::OK,
        "a valid cosignature over the published head is accepted",
    );

    // It is now served, and verifies CLIENT-SIDE against the pinned witness key over the same head.
    let head2: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    assert_eq!(head2.cosignatures.len(), 1, "the cosignature is served");
    assert_eq!(head2.timestamp_s, head.timestamp_s, "the head is stable (same bytes witnesses signed)");
    let served = decode64(&head2.cosignatures[0].signature);
    let pinned = Witness::from_ed25519_bytes(0, &wk.verifying_key().to_bytes()).unwrap();
    assert!(
        verify_witness_cosignature(&pinned, &sth, &served),
        "the served cosignature verifies under the pinned witness key over the head",
    );
    // ...and does NOT verify over a different head (a relay cannot pass it off as covering other bytes).
    let other = SignedTreeHead { root_hash: [0xFF; 32], ..sth };
    assert!(!verify_witness_cosignature(&pinned, &other, &served), "it only covers the head it signed");

    // A re-submission by the same witness de-duplicates (still exactly one).
    assert_eq!(
        post_json(app.clone(), "/kt/witness/cosign", cosign_body(&head, 0, &sig)).await,
        StatusCode::OK,
    );
    let head3: WitnessedSthResponse = get_json(app, "/kt/sth/witnessed").await;
    assert_eq!(head3.cosignatures.len(), 1, "a witness re-submission replaces, never duplicates");
}

#[tokio::test]
async fn a_cosignature_over_a_head_the_relay_did_not_publish_is_rejected() {
    let (app, wk) = build_with_witness().await;
    let head: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    // The witness signs a head with a DIFFERENT timestamp than the relay published.
    let forged = SignedTreeHead { timestamp_s: head.timestamp_s - 100, ..head_of(&head) };
    let sig = wk.sign(&forged.signing_bytes()).to_bytes();
    let mut body = cosign_body(&head, 0, &sig);
    body["timestamp_s"] = json!(head.timestamp_s - 100);
    assert_eq!(
        post_json(app, "/kt/witness/cosign", body).await,
        StatusCode::CONFLICT,
        "a cosignature over a head the relay did not publish is rejected (409 -> re-fetch + re-cosign)",
    );
}

#[tokio::test]
async fn a_signature_not_by_the_pinned_witness_is_rejected() {
    let (app, _wk) = build_with_witness().await;
    let head: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    let sth = head_of(&head);
    // An attacker key (not the pinned witness) signs the REAL head and claims witness_index 0.
    let attacker = SigningKey::from_bytes(&[9u8; 32]);
    let sig = attacker.sign(&sth.signing_bytes()).to_bytes();
    assert_eq!(
        post_json(app, "/kt/witness/cosign", cosign_body(&head, 0, &sig)).await,
        StatusCode::UNAUTHORIZED,
        "a signature that is not the pinned witness-at-that-index's is rejected (401)",
    );
}

#[tokio::test]
async fn a_witness_index_outside_the_allow_list_is_rejected() {
    let (app, wk) = build_with_witness().await;
    let head: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    let sig = wk.sign(&head_of(&head).signing_bytes()).to_bytes();
    // Only index 0 is configured; index 5 is unknown.
    assert_eq!(
        post_json(app, "/kt/witness/cosign", cosign_body(&head, 5, &sig)).await,
        StatusCode::BAD_REQUEST,
        "a witness_index outside the pinned allow-list is rejected (400)",
    );
}

#[tokio::test]
async fn witnessing_disabled_serves_the_head_with_no_cosignatures() {
    // No witnesses configured (the default).
    let dir = KtDirectory::with_vrf_seed(LOG_SEED).await.unwrap();
    let app = kt_router(KtState::new(dir));

    let head: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    assert!(head.witnesses.is_empty(), "no witnesses advertised");
    assert!(head.cosignatures.is_empty());
    assert!(!head.log_signature.is_empty(), "the head + log signature are still served");

    // A submission is refused outright (no witness exists at any index).
    let sig = [0u8; 64];
    assert_eq!(
        post_json(app, "/kt/witness/cosign", cosign_body(&head, 0, &sig)).await,
        StatusCode::BAD_REQUEST,
    );
}

#[tokio::test]
async fn auditor_plus_two_independent_operators_reach_quorum_satisfied() {
    use ed25519_dalek::VerifyingKey;
    use mercury_kt::{KeyTransparencyWitnessStatus, WitnessCosignature, verify_witnessed_tree_head};
    use mercury_relay::VrfKeyResponse;

    // 2 witnesses across 2 INDEPENDENT operators (ids 1 + 2) + a separate auditor.
    let w0 = SigningKey::from_bytes(&[11u8; 32]);
    let w1 = SigningKey::from_bytes(&[12u8; 32]);
    let auditor = SigningKey::from_bytes(&[13u8; 32]);
    let dir = KtDirectory::with_vrf_seed(LOG_SEED).await.unwrap();
    let app = kt_router(
        KtState::new(dir)
            .with_witnesses(vec![
                Witness::from_ed25519_bytes(1, &w0.verifying_key().to_bytes()).unwrap(),
                Witness::from_ed25519_bytes(2, &w1.verifying_key().to_bytes()).unwrap(),
            ])
            .with_auditor(
                Witness::from_ed25519_bytes(0, &auditor.verifying_key().to_bytes()).unwrap(),
            ),
    );

    // Publish the head; both witnesses + the auditor co-sign the SAME canonical bytes.
    let head: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    let sth = head_of(&head);
    let msg = sth.signing_bytes();
    assert_eq!(
        post_json(app.clone(), "/kt/witness/cosign", cosign_body(&head, 0, &w0.sign(&msg).to_bytes()))
            .await,
        StatusCode::OK
    );
    assert_eq!(
        post_json(app.clone(), "/kt/witness/cosign", cosign_body(&head, 1, &w1.sign(&msg).to_bytes()))
            .await,
        StatusCode::OK
    );
    let auditor_body = json!({
        "tree_size": head.tree_size,
        "root_hash": head.root_hash,
        "timestamp_s": head.timestamp_s,
        "signature": hex::encode(&auditor.sign(&msg).to_bytes()),
    });
    assert_eq!(
        post_json(app.clone(), "/kt/auditor/cosign", auditor_body).await,
        StatusCode::OK,
        "the pinned auditor's signature over the published head is accepted"
    );

    // The served bundle now carries 2 cosignatures + the auditor signature + the auditor key.
    let bundle: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    assert_eq!(bundle.cosignatures.len(), 2, "both witness cosignatures served");
    let auditor_sig =
        decode64(bundle.auditor_signature.as_deref().expect("auditor signature served"));
    assert!(bundle.auditor_public_key.is_some(), "the pinned auditor key is advertised");

    // CLIENT-SIDE: with the pinned log key, the 2 witnesses, and the auditor, the FULL quorum gate
    // PASSES — previously unreachable (kt_witness_status is hard-wired to Invalid without an auditor
    // signature, and the relay had no way to serve one).
    let vrf: VrfKeyResponse = get_json(app.clone(), "/kt/vrf-key").await;
    let log_pub = VerifyingKey::from_bytes(&decode32(&vrf.log_public_key)).unwrap();
    let cosigs: Vec<WitnessCosignature> = bundle
        .cosignatures
        .iter()
        .map(|c| WitnessCosignature {
            witness_index: c.witness_index,
            signature: decode64(&c.signature),
        })
        .collect();
    let pinned = [
        Witness::from_ed25519_bytes(1, &w0.verifying_key().to_bytes()).unwrap(),
        Witness::from_ed25519_bytes(2, &w1.verifying_key().to_bytes()).unwrap(),
    ];
    let v = verify_witnessed_tree_head(
        &sth,
        &log_pub,
        &decode64(&bundle.log_signature),
        &auditor.verifying_key(),
        &auditor_sig,
        &pinned,
        &cosigs,
        sth.root_hash,
        sth.tree_size,
        sth.timestamp_s,
    );
    assert_eq!(
        v.kt_witness_status(2),
        KeyTransparencyWitnessStatus::QuorumSatisfied,
        "2 witnesses across 2 operators + the auditor reach QuorumSatisfied"
    );

    // Rejection rail: the auditor endpoint refuses a signature that is NOT the pinned auditor's.
    let forged = json!({
        "tree_size": bundle.tree_size,
        "root_hash": bundle.root_hash,
        "timestamp_s": bundle.timestamp_s,
        "signature": hex::encode(&w0.sign(&head_of(&bundle).signing_bytes()).to_bytes()),
    });
    assert_eq!(
        post_json(app, "/kt/auditor/cosign", forged).await,
        StatusCode::UNAUTHORIZED,
        "a signature not by the pinned auditor is rejected"
    );
}

#[tokio::test]
async fn healthz_reports_kt_epoch_and_witness_freshness() {
    use mercury_relay::HealthResponse;

    // Fresh relay, ONE witness, no auditor, no head published yet.
    let (app, wk) = build_with_witness().await;

    let h: HealthResponse = get_json(app.clone(), "/healthz").await;
    assert_eq!(h.status, "ok");
    assert_eq!(h.kt_epoch, Some(0), "a fresh directory is epoch 0");
    assert_eq!(h.witnesses_configured, 1);
    assert!(!h.auditor_configured, "no auditor configured in this fixture");
    assert_eq!(h.witness_cosig_count, 0, "nothing cosigned yet");
    assert!(!h.auditor_present);
    assert_eq!(h.witness_head_age_s, None, "no head published yet");

    // Publish the head + cosign it; /healthz now reflects the cosignature + a fresh head age.
    let head: WitnessedSthResponse = get_json(app.clone(), "/kt/sth/witnessed").await;
    let sig = wk.sign(&head_of(&head).signing_bytes()).to_bytes();
    assert_eq!(
        post_json(app.clone(), "/kt/witness/cosign", cosign_body(&head, 0, &sig)).await,
        StatusCode::OK
    );
    let h2: HealthResponse = get_json(app, "/healthz").await;
    assert_eq!(h2.witness_cosig_count, 1, "the cosignature shows up in health");
    let age = h2.witness_head_age_s.expect("a head is now published");
    assert!(age >= 0, "head age is a real elapsed value");
}
