//! Spine integration test: a REAL end-to-end-encrypted message object transits
//! the relay through the authoritative mercury-core gates and opens to the
//! original plaintext on the other side.
//!
//! This composes the critical path S1 (sealed message object: seal -> canonical
//! serialize) with S4 (the relay: submit -> poll -> ack, gated). The relay only
//! ever sees opaque bytes; the message is sealed/opened entirely client-side.
//! It is a regression guard that the real serialized message survives the
//! relay's admission gates and size limits and comes back byte-for-byte
//! openable -- something neither the relay tests (opaque hex payloads) nor the
//! message tests (no relay) exercise on their own.

use std::sync::{Arc, LazyLock};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header::CONTENT_TYPE},
};
use http_body_util::BodyExt;
use mercury_core::{ClientMessageEnvelope, ConversationPolicyState, MessageKind, ProtocolSuite};
use mercury_keys::{
    DeviceKeyPair, IdentityKeyPair, MlKemKeyPair, RelayRouteOperation,
    account_id_from_identity_key, sign_relay_route_pop,
};
use mercury_media::{AttachmentReference, seal_attachment};
use mercury_message::{
    from_bytes, open_message, open_message_hybrid, seal_message, seal_message_hybrid, to_bytes,
};
use mercury_mls::{
    MlsMember, add_member, join_group, join_group_bytes, receive_message, receive_message_bytes,
    send_message, serialize_message,
};
use mercury_relay::http::{HEADER_IDENTITY_PUB, HEADER_POP_TIMESTAMP, HEADER_ROUTE_POP};
use mercury_relay::{InMemoryQueueStore, NoopPushSender, QueueStore, RelayState, router};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower::ServiceExt;

const REPLAY_HEX: &str = "0909090909090909090909090909090909090909090909090909090909090909";
const AUTH32_HEX: &str = "0101010101010101010101010101010101010101010101010101010101010101";
const TAG_HEX: &str = "1010101010101010101010101010101010101010";

/// A fixed test identity; its account id is the 32-byte route the relay authenticates against.
static TEST_IDENTITY: LazyLock<IdentityKeyPair> = LazyLock::new(IdentityKeyPair::generate);

fn route_hex() -> String {
    to_hex(&account_id_from_identity_key(&TEST_IDENTITY.public()))
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn build() -> Router {
    let store: Arc<Mutex<dyn QueueStore>> = Arc::new(Mutex::new(InMemoryQueueStore::new()));
    let push = Arc::new(NoopPushSender);
    router(RelayState::new(store, push))
}

fn valid_envelope() -> ClientMessageEnvelope {
    ClientMessageEnvelope {
        version: 1,
        // Classical suite: this spine test seals via the classical seal_message
        // path (the relay is suite-agnostic; it transits opaque bytes).
        suite: ProtocolSuite::ClassicalDev,
        conversation_id_len: 32,
        sender_account_id_len: 32,
        sender_device_id_len: 32,
        epoch: 7,
        sequence: 42,
        kind: MessageKind::Application,
        payload_len: 0,
        critical_flags: 0,
        noncritical_flags: 0,
    }
}

fn hybrid_envelope() -> ClientMessageEnvelope {
    ClientMessageEnvelope {
        suite: ProtocolSuite::HybridPqDev,
        ..valid_envelope()
    }
}

fn valid_context() -> ConversationPolicyState {
    ConversationPolicyState {
        expected_epoch: 7,
        expected_sequence: 42,
        minimum_suite: ProtocolSuite::ClassicalDev,
        max_payload_len: 8192,
    }
}

fn submit_body(ciphertext_hex: &str) -> Value {
    json!({
        "route_id": route_hex(),
        "replay_token": REPLAY_HEX,
        "ciphertext": ciphertext_hex,
        "sealed_header": "05".repeat(96), // 96 bytes, inside the relay gate window
        "queue_ttl_s": 300,
        "max_queue_ttl_s": 86400,
        "max_ciphertext_len": 1048576,
        "padding_bucket": 3,
        "outbound_send": {
            "accepted": true,
            "can_send": true,
            "can_persist_ciphertext": true,
            "requires_user_action": false,
            "reason_code": 0
        }
    })
}

fn ack_body() -> Value {
    json!({
        "ack_token": AUTH32_HEX,
        "ciphertext_digest": AUTH32_HEX,
        "delivery_tag": TAG_HEX,
        "delivered_at_s": 100,
        "acknowledged_at_s": 130
    })
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(body).unwrap()))
        .unwrap()
}

fn authed(method: &str, uri: &str, body: Option<&Value>) -> Request<Body> {
    // Sign a fresh route-ownership proof with the fixed test identity (which owns route_hex()),
    // over the 32-byte route in the uri's last segment — the real auth the hardened relay checks.
    let route_seg = uri.rsplit('/').next().unwrap_or("");
    let route32: [u8; 32] = mercury_relay::hex::decode(route_seg)
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b).ok())
        .unwrap_or([0u8; 32]);
    let ts = mercury_relay::store::now_unix_seconds();
    let operation = match uri.rsplit('/').nth(1) {
        Some("ack") => RelayRouteOperation::Ack,
        Some("queue") => RelayRouteOperation::Delete,
        _ => RelayRouteOperation::Poll,
    };
    let sig = sign_relay_route_pop(&TEST_IDENTITY, &route32, operation, ts);
    let mut b = Request::builder()
        .method(method)
        .uri(uri)
        .header(
            HEADER_IDENTITY_PUB,
            to_hex(TEST_IDENTITY.public().as_bytes()),
        )
        .header(HEADER_POP_TIMESTAMP, ts.to_string())
        .header(HEADER_ROUTE_POP, to_hex(&sig));
    match body {
        Some(v) => {
            b = b.header(CONTENT_TYPE, "application/json");
            b.body(Body::from(serde_json::to_vec(v).unwrap())).unwrap()
        }
        None => b.body(Body::empty()).unwrap(),
    }
}

async fn json_of(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn call(router: &Router, req: Request<Body>) -> axum::response::Response {
    router.clone().oneshot(req).await.unwrap()
}

#[tokio::test]
async fn real_sealed_message_transits_relay_and_opens() {
    // S1: seal a real message + serialize to canonical wire bytes.
    let recipient = DeviceKeyPair::generate();
    let plaintext = b"mercury spine: a real sealed message over the relay";
    let sealed = seal_message(
        &recipient.public(),
        plaintext,
        valid_envelope(),
        &valid_context(),
    )
    .expect("seal must succeed for an accepted envelope");
    let wire = to_bytes(&sealed);
    let wire_hex = to_hex(&wire);

    // seal_message succeeded, which means the envelope policy ACCEPTED this
    // message (it returns Err on a policy reject) -- so the accepted
    // `outbound_send` witness in submit_body is consistent with a policy-clean
    // message, not a free-floating approval. The relay independently re-runs
    // evaluate_relay_submission on the submission; the companion test below
    // proves it refuses a rejected outbound_send witness.
    let app = build();
    let resp = call(&app, post_json("/relay/submit", &submit_body(&wire_hex))).await;
    assert_eq!(
        resp.status(),
        StatusCode::ACCEPTED,
        "relay must accept a well-formed real message object as its opaque payload"
    );

    // Poll it back -- the relay returns the payload byte-for-byte.
    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let got_hex = json_of(resp).await["ciphertext"]
        .as_str()
        .expect("delivery carries opaque ciphertext")
        .to_string();
    assert_eq!(
        got_hex, wire_hex,
        "relay must transit the payload unchanged"
    );

    // S1 (receive side): decode + parse + open -> exact original plaintext.
    let got = mercury_relay::hex::decode(&got_hex).expect("hex decodes");
    let parsed = from_bytes(&got).expect("canonical wire parses");
    let recovered =
        open_message(&recipient, &parsed, &valid_context()).expect("message opens for recipient");
    assert_eq!(
        recovered, plaintext,
        "the message that transited the relay must open to the original plaintext"
    );

    // Complete delivery: ack moves it to Delivered and clears the payload.
    let resp = call(
        &app,
        authed(
            "POST",
            &format!("/relay/ack/{}", route_hex()),
            Some(&ack_body()),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_of(resp).await["accepted"], true);
}

#[tokio::test]
async fn real_hybrid_message_transits_relay_and_opens() {
    // S11 x S4: a REAL post-quantum hybrid (X25519 + ML-KEM-768) message object
    // transits the relay and opens to the original plaintext. The relay is
    // suite-agnostic (opaque bytes), so this proves the larger PQ wire format
    // survives the submit -> poll -> ack path end-to-end.
    let recipient_x = DeviceKeyPair::generate();
    let recipient_pq = MlKemKeyPair::generate();
    let plaintext = b"mercury spine: a real POST-QUANTUM hybrid message over the relay";
    let sealed = seal_message_hybrid(
        &recipient_x.public(),
        &recipient_pq.public(),
        plaintext,
        hybrid_envelope(),
        &valid_context(),
    )
    .expect("hybrid seal must succeed for an accepted HybridPqDev envelope");
    let wire_hex = to_hex(&to_bytes(&sealed));

    let app = build();
    let resp = call(&app, post_json("/relay/submit", &submit_body(&wire_hex))).await;
    assert_eq!(
        resp.status(),
        StatusCode::ACCEPTED,
        "relay must accept the opaque hybrid message object"
    );

    // Poll it back byte-for-byte.
    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let got_hex = json_of(resp).await["ciphertext"]
        .as_str()
        .expect("delivery carries opaque ciphertext")
        .to_string();
    assert_eq!(
        got_hex, wire_hex,
        "relay must transit the hybrid payload unchanged"
    );

    // Receive side: decode + parse + open via the hybrid path -> original plaintext.
    let got = mercury_relay::hex::decode(&got_hex).expect("hex decodes");
    let parsed = from_bytes(&got).expect("canonical wire parses");
    let recovered = open_message_hybrid(&recipient_x, &recipient_pq, &parsed, &valid_context())
        .expect("hybrid message opens for recipient");
    assert_eq!(
        recovered, plaintext,
        "the hybrid message that transited the relay must open to the original plaintext"
    );

    let resp = call(
        &app,
        authed(
            "POST",
            &format!("/relay/ack/{}", route_hex()),
            Some(&ack_body()),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_of(resp).await["accepted"], true);
}

#[tokio::test]
async fn relay_rejects_when_outbound_send_gate_rejects_a_real_message() {
    // Same real message, but the client's outbound-send decision is a reject:
    // the relay must refuse to admit it (the gate is the admission path, not a
    // relay shortcut) -- so a blocked send never reaches the queue.
    let recipient = DeviceKeyPair::generate();
    let sealed = seal_message(
        &recipient.public(),
        b"should never be queued",
        valid_envelope(),
        &valid_context(),
    )
    .unwrap();
    let wire_hex = to_hex(&to_bytes(&sealed));

    let app = build();
    let mut body = submit_body(&wire_hex);
    body["outbound_send"] = json!({
        "accepted": false,
        "can_send": false,
        "can_persist_ciphertext": false,
        "requires_user_action": true,
        "reason_code": 4
    });
    let resp = call(&app, post_json("/relay/submit", &body)).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json_of(resp).await["reason"], "send_gate_rejected");
}

#[tokio::test]
async fn real_group_message_transits_relay_and_opens() {
    // S8 x S4: a REAL MLS (RFC 9420) GROUP application message transits the relay
    // and opens to the original plaintext. A two-member group converges on
    // OpenMLS 0.8.1 (create -> add -> join from Welcome); the sender encrypts an
    // application message and serializes it to canonical MLS wire bytes, and those
    // opaque bytes ride the relay's submit -> poll -> ack path. The relay is
    // suite/format-agnostic: it never sees plaintext and transits the MLS frame
    // byte-for-byte -- proving group messaging composes with the relay the same
    // way the 1:1 sealed (S1) and hybrid-PQ (S11) message objects already do.
    let creator = MlsMember::generate(b"creator@mercury").expect("member generates");
    let alice = MlsMember::generate(b"alice@mercury").expect("member generates");
    let mut creator_group = creator.create_group([0x5a; 32]).expect("group creates");
    let (welcome, _consumed) =
        add_member(&mut creator_group, &creator, alice.key_package()).expect("add member");
    let mut alice_group = join_group(&alice, &welcome).expect("alice joins from welcome");

    let plaintext = b"mercury spine: a real MLS group message over the relay";
    let outgoing = send_message(&mut creator_group, &creator, plaintext).expect("encrypt");
    let wire = serialize_message(&outgoing).expect("serialize to MLS wire bytes");
    let wire_hex = to_hex(&wire);

    let app = build();
    let resp = call(&app, post_json("/relay/submit", &submit_body(&wire_hex))).await;
    assert_eq!(
        resp.status(),
        StatusCode::ACCEPTED,
        "relay must accept the opaque MLS group message as its payload"
    );

    // Poll it back byte-for-byte.
    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let got_hex = json_of(resp).await["ciphertext"]
        .as_str()
        .expect("delivery carries opaque ciphertext")
        .to_string();
    assert_eq!(
        got_hex, wire_hex,
        "relay must transit the MLS payload unchanged"
    );

    // Receive side: decode + open via the MLS wire path -> exact original plaintext.
    let got = mercury_relay::hex::decode(&got_hex).expect("hex decodes");
    let recovered = receive_message_bytes(&mut alice_group, &alice, &got)
        .expect("MLS group message opens for the co-member");
    assert_eq!(
        recovered, plaintext,
        "the MLS message that transited the relay must open to the original plaintext"
    );

    let resp = call(
        &app,
        authed(
            "POST",
            &format!("/relay/ack/{}", route_hex()),
            Some(&ack_body()),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_of(resp).await["accepted"], true);
}

#[tokio::test]
async fn real_group_welcome_transits_relay_and_joins() {
    // S8 x S4 (group-bootstrap leg): the WELCOME that adds a member to an MLS
    // group transits the UNTRUSTED relay as opaque bytes, and the new member
    // builds her group view from the polled-back bytes via join_group_bytes (the
    // realistic relay-delivery path). This proves the full group-JOIN handshake --
    // not just steady-state messaging -- composes with the relay. A short
    // in-process exchange then confirms the relay-bootstrapped group is genuinely
    // functional (converged + able to decrypt), so the join wasn't merely parsed.
    let creator = MlsMember::generate(b"creator@mercury").expect("member generates");
    let alice = MlsMember::generate(b"alice@mercury").expect("member generates");
    let mut creator_group = creator.create_group([0x2b; 32]).expect("group creates");
    let (welcome, _consumed) =
        add_member(&mut creator_group, &creator, alice.key_package()).expect("add member");
    let welcome_wire = serialize_message(&welcome).expect("serialize the Welcome");
    let welcome_hex = to_hex(&welcome_wire);

    // The Welcome rides the relay's submit -> poll path as an opaque payload.
    let app = build();
    let resp = call(&app, post_json("/relay/submit", &submit_body(&welcome_hex))).await;
    assert_eq!(
        resp.status(),
        StatusCode::ACCEPTED,
        "relay must accept the opaque MLS Welcome as its payload"
    );
    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let got_hex = json_of(resp).await["ciphertext"]
        .as_str()
        .expect("delivery carries the opaque Welcome")
        .to_string();
    assert_eq!(
        got_hex, welcome_hex,
        "relay must transit the Welcome unchanged"
    );

    // The joiner builds her group view from the RELAY-DELIVERED bytes.
    let got = mercury_relay::hex::decode(&got_hex).expect("hex decodes");
    let mut alice_group =
        join_group_bytes(&alice, &got).expect("alice joins from the relay-delivered Welcome");

    // She converged with the adder (same group id + epoch)...
    assert_eq!(
        creator_group.group_id().as_slice(),
        alice_group.group_id().as_slice()
    );
    assert_eq!(creator_group.epoch(), alice_group.epoch());

    // ...and the bootstrapped group is functional: a real message decrypts.
    let msg = send_message(&mut creator_group, &creator, b"welcome aboard").expect("encrypt");
    assert_eq!(
        receive_message(&mut alice_group, &alice, &msg).expect("decrypt"),
        b"welcome aboard"
    );

    let resp = call(
        &app,
        authed(
            "POST",
            &format!("/relay/ack/{}", route_hex()),
            Some(&ack_body()),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_of(resp).await["accepted"], true);
}

#[tokio::test]
async fn real_attachment_reference_transits_relay_and_opens_blob() {
    // S9 x S1 x S4: a REAL attachment, end to end. The media is encrypted to a
    // sealed blob (which would go to a separate, untrusted blob store); the content
    // key + the blob's SHA-256 digest + length are packed into an
    // AttachmentReference, sealed INSIDE the E2E message, and that message transits
    // the relay. The recipient opens the message, recovers the reference, fetches
    // the blob from the "store", and verifies+decrypts it. A substituted/tampered
    // blob is rejected end to end because the digest is authenticated by the
    // (E2E-sealed) message -- the untrusted store cannot swap the blob.
    let recipient = DeviceKeyPair::generate();
    let media = b"the actual attachment bytes -- pretend this is a photo/file".repeat(40);
    let (content_key, sealed_blob) = seal_attachment(&media).expect("seal attachment");
    let reference = AttachmentReference::new(content_key, &sealed_blob, media.len() as u64);

    // The reference (key + blob digest + len) is the E2E message payload.
    let payload = reference.to_bytes();
    let sealed = seal_message(
        &recipient.public(),
        &payload,
        valid_envelope(),
        &valid_context(),
    )
    .expect("seal the attachment-reference message");
    let wire_hex = to_hex(&to_bytes(&sealed));

    // The MESSAGE transits the relay (the large blob goes to a separate store).
    let app = build();
    let resp = call(&app, post_json("/relay/submit", &submit_body(&wire_hex))).await;
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let resp = call(
        &app,
        authed("GET", &format!("/relay/poll/{}", route_hex()), None),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let got_hex = json_of(resp).await["ciphertext"]
        .as_str()
        .expect("delivery carries the opaque message")
        .to_string();
    assert_eq!(
        got_hex, wire_hex,
        "relay must transit the message unchanged"
    );

    // Recipient opens the message and recovers the attachment reference.
    let got = mercury_relay::hex::decode(&got_hex).expect("hex decodes");
    let parsed = from_bytes(&got).expect("message parses");
    let recovered_payload =
        open_message(&recipient, &parsed, &valid_context()).expect("message opens");
    let recovered_ref =
        AttachmentReference::from_bytes(&recovered_payload).expect("reference parses");

    // The "blob store" hands back the genuine blob -> verify against the
    // authenticated digest + decrypt to the exact original media.
    assert_eq!(
        recovered_ref
            .open(&sealed_blob)
            .expect("blob verifies + opens"),
        media,
        "the attachment must decrypt to the original media end to end"
    );

    // A tampered/substituted blob is rejected (the digest is message-authenticated).
    let mut tampered = sealed_blob.clone();
    let mid = tampered.len() / 2;
    tampered[mid] ^= 0x01;
    assert!(
        recovered_ref.open(&tampered).is_err(),
        "a store-substituted blob must be rejected end to end"
    );

    let resp = call(
        &app,
        authed(
            "POST",
            &format!("/relay/ack/{}", route_hex()),
            Some(&ack_body()),
        ),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_of(resp).await["accepted"], true);
}
