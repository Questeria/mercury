//! Application-controller tests: two controllers over a shared in-memory transport hold a
//! complete end-to-end-encrypted conversation — discovery through the directory, the first
//! flight, replies, ordering, and fail-closed error paths. The plaintext never leaves the
//! controllers; the transport only ever moves opaque sealed blobs.

use mercury_app::{AppController, AppError, Direction};
use mercury_client::InMemoryTransport;

#[test]
fn two_apps_hold_a_conversation_via_the_directory() {
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport.clone());
    let mut bob = AppController::new(transport.clone());
    let alice_id = alice.account_id();
    let bob_id = bob.account_id();

    // Bob publishes himself; Alice discovers him by account id and opens the conversation.
    bob.publish_self().unwrap();
    alice.add_contact(&bob_id).unwrap();
    assert!(alice.can_send(&bob_id));
    alice.send(&bob_id, "hi bob").unwrap();

    // Bob polls, sees the first message (which also establishes his side of the session).
    let inbox = bob.poll().unwrap();
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].direction, Direction::Incoming);
    assert_eq!(inbox[0].text, "hi bob");
    assert_eq!(inbox[0].peer, alice_id);

    // Bob replies on the established session (no add_contact needed on his side).
    bob.send(&alice_id, "hey alice").unwrap();
    let inbox = alice.poll().unwrap();
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].text, "hey alice");
    assert_eq!(inbox[0].peer, bob_id);

    // Each side's history with the other holds both messages, in order.
    assert_eq!(alice.history(&bob_id).len(), 2);
    assert_eq!(bob.history(&alice_id).len(), 2);
    assert_eq!(alice.history(&bob_id)[0].direction, Direction::Outgoing);
    assert_eq!(alice.history(&bob_id)[1].direction, Direction::Incoming);
}

#[test]
fn many_messages_both_directions() {
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport.clone());
    let mut bob = AppController::new(transport.clone());
    let alice_id = alice.account_id();
    let bob_id = bob.account_id();

    bob.publish_self().unwrap();
    alice.add_contact(&bob_id).unwrap();
    alice.send(&bob_id, "boot").unwrap();
    assert_eq!(bob.poll().unwrap()[0].text, "boot");

    for i in 0..12u32 {
        let a = format!("alice {i}");
        alice.send(&bob_id, &a).unwrap();
        assert_eq!(bob.poll().unwrap()[0].text, a);

        let b = format!("bob {i}");
        bob.send(&alice_id, &b).unwrap();
        assert_eq!(alice.poll().unwrap()[0].text, b);
    }
    // 1 boot + 12 each direction = 25 messages in each controller's log.
    assert_eq!(alice.log().len(), 25);
    assert_eq!(bob.log().len(), 25);
}

#[test]
fn a_batch_of_inbound_messages_polls_in_order() {
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport.clone());
    let mut bob = AppController::new(transport.clone());
    let bob_id = bob.account_id();

    bob.publish_self().unwrap();
    alice.add_contact(&bob_id).unwrap();
    alice.send(&bob_id, "one").unwrap();
    alice.send(&bob_id, "two").unwrap();
    alice.send(&bob_id, "three").unwrap();

    // A single poll drains all three queued messages, in order.
    let inbox = bob.poll().unwrap();
    let texts: Vec<&str> = inbox.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(texts, ["one", "two", "three"]);
}

#[test]
fn send_without_a_contact_fails_closed() {
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport);
    let nobody = "00".repeat(32);
    assert_eq!(alice.send(&nobody, "hi"), Err(AppError::NoContact));
    assert!(!alice.can_send(&nobody));
}

#[test]
fn add_contact_for_an_unpublished_id_fails_closed() {
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport);
    let nobody = "11".repeat(32);
    // No card in the directory -> a client (ContactNotFound) error, surfaced as AppError.
    assert!(matches!(
        alice.add_contact(&nobody),
        Err(AppError::Client(_))
    ));
}

#[test]
fn malformed_account_ids_are_rejected() {
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport);
    assert_eq!(alice.send("nothex", "hi"), Err(AppError::BadAccountId));
    assert_eq!(alice.add_contact("zz"), Err(AppError::BadAccountId));
    assert_eq!(
        alice.add_contact(&"ab".repeat(31)),
        Err(AppError::BadAccountId)
    );
    assert!(!alice.can_send("not-a-valid-id"));
}

#[test]
fn poll_skips_a_garbage_blob_without_panic() {
    // A non-decryptable blob addressed to us must be skipped, not panic or surface as a
    // message. (Simulates relay noise / a misrouted or corrupt blob.)
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport.clone());
    let me = alice.account_id_bytes();
    use mercury_client::Transport as _;
    transport
        .submit(&me, b"this is not a sealed mercury blob")
        .unwrap();
    let got = alice.poll().unwrap();
    assert!(got.is_empty());
    assert!(alice.log().is_empty());
}

#[test]
fn the_transport_only_sees_opaque_blobs() {
    // The plaintext must never appear on the wire: the in-memory transport carries only the
    // sealed blob, which does not contain the message text.
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport.clone());
    let mut bob = AppController::new(transport.clone());
    let bob_id = bob.account_id();
    bob.publish_self().unwrap();
    alice.add_contact(&bob_id).unwrap();

    let secret = "the eagle lands at midnight";
    alice.send(&bob_id, secret).unwrap();

    // Peek the raw blob the transport is about to deliver to Bob: it must not contain the
    // plaintext anywhere.
    use mercury_client::{PollAuth, Transport as _};
    let blob = transport
        .poll(&bob.account_id_bytes(), &PollAuth::unauthenticated())
        .unwrap()
        .expect("queued");
    assert!(
        !blob.windows(secret.len()).any(|w| w == secret.as_bytes()),
        "plaintext leaked onto the transport"
    );
}

#[test]
fn the_log_serializes_to_json() {
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport.clone());
    let mut bob = AppController::new(transport.clone());
    let bob_id = bob.account_id();
    bob.publish_self().unwrap();
    alice.add_contact(&bob_id).unwrap();
    alice.send(&bob_id, "hello").unwrap();

    let json = alice.log_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["direction"], "outgoing");
    assert_eq!(arr[0]["text"], "hello");
    assert_eq!(arr[0]["peer"], bob_id);
}

// --- JSON command surface (the FFI/HTTP-ready bridge) -----------------------

use serde_json::{Value, json};

fn cmd(app: &mut AppController<InMemoryTransport>, command: Value) -> Value {
    let out = app.handle_command(&command.to_string());
    serde_json::from_str(&out).expect("a command response is always valid JSON")
}

#[test]
fn a_full_conversation_runs_purely_over_json_commands() {
    let transport = InMemoryTransport::new();
    let mut alice = AppController::new(transport.clone());
    let mut bob = AppController::new(transport.clone());

    // Identities, via the command surface.
    let alice_id = cmd(&mut alice, json!({"cmd": "account_id"}))["result"]["account_id"]
        .as_str()
        .unwrap()
        .to_string();
    let bob_id = cmd(&mut bob, json!({"cmd": "account_id"}))["result"]["account_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Bob publishes; Alice discovers + opens the conversation; all via JSON.
    assert_eq!(cmd(&mut bob, json!({"cmd": "publish_self"}))["ok"], true);
    assert_eq!(
        cmd(
            &mut alice,
            json!({"cmd": "add_contact", "account_id": bob_id})
        )["ok"],
        true
    );
    assert_eq!(
        cmd(&mut alice, json!({"cmd": "can_send", "peer": bob_id}))["result"]["can_send"],
        true
    );
    assert_eq!(
        cmd(
            &mut alice,
            json!({"cmd": "send", "peer": bob_id, "text": "hi over json"})
        )["ok"],
        true
    );

    // Bob polls via JSON and sees the message.
    let inbox = cmd(&mut bob, json!({"cmd": "poll"}));
    let msgs = inbox["result"]["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0]["text"], "hi over json");
    assert_eq!(msgs[0]["direction"], "incoming");
    assert_eq!(msgs[0]["peer"], alice_id);

    // Bob replies via JSON; Alice polls.
    assert_eq!(
        cmd(
            &mut bob,
            json!({"cmd": "send", "peer": alice_id, "text": "ack"})
        )["ok"],
        true
    );
    let inbox = cmd(&mut alice, json!({"cmd": "poll"}));
    assert_eq!(inbox["result"]["messages"][0]["text"], "ack");

    // History via JSON.
    let hist = cmd(&mut alice, json!({"cmd": "history", "peer": bob_id}));
    assert_eq!(hist["result"]["messages"].as_array().unwrap().len(), 2);
}

#[test]
fn the_command_surface_is_fail_closed() {
    let transport = InMemoryTransport::new();
    let mut app = AppController::new(transport);

    // Not JSON at all.
    let r: Value = serde_json::from_str(&app.handle_command("}{ not json")).unwrap();
    assert_eq!(r["ok"], false);
    assert!(r["error"].is_string());

    // Unknown command.
    assert_eq!(cmd(&mut app, json!({"cmd": "self_destruct"}))["ok"], false);

    // Missing required field (send without text).
    assert_eq!(
        cmd(&mut app, json!({"cmd": "send", "peer": "00"}))["ok"],
        false
    );

    // A well-formed command whose operation fails surfaces ok:false, not a panic.
    let bad = cmd(
        &mut app,
        json!({"cmd": "add_contact", "account_id": "nothex"}),
    );
    assert_eq!(bad["ok"], false);
    assert!(bad["error"].is_string());

    // send to an unknown peer (valid id, no contact) -> ok:false.
    let nobody = "11".repeat(32);
    assert_eq!(
        cmd(
            &mut app,
            json!({"cmd": "send", "peer": nobody, "text": "x"})
        )["ok"],
        false
    );
}

#[test]
fn handle_command_never_panics_on_arbitrary_input() {
    let transport = InMemoryTransport::new();
    let mut app = AppController::new(transport);
    let probes = [
        "",
        "[]",
        "null",
        "42",
        "\"string\"",
        "{}",
        "{\"cmd\":123}",
        "{\"cmd\":\"poll\",\"extra\":[1,2,3]}",
        "{\"cmd\":\"send\"}",
        "{\"cmd\":\"add_contact\",\"account_id\":null}",
    ];
    for p in probes {
        // Must always return valid JSON, never panic.
        let out = app.handle_command(p);
        let _: Value = serde_json::from_str(&out).expect("response is valid JSON");
    }
}
