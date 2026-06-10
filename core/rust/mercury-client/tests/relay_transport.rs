//! Transport integration tests: a full two-client end-to-end-encrypted exchange over
//! (1) the in-memory transport and (2) the REAL Mercury relay started on an ephemeral
//! port. The relay only ever sees opaque ciphertext; the plaintext is protected by the
//! post-quantum ratchet entirely inside the clients.

use std::sync::Arc;
use std::time::Duration;

use mercury_client::{InMemoryTransport, MercuryClient, PollAuth, RelayTransport, Transport};
use mercury_relay::{InMemoryQueueStore, NoopPushSender, QueueStore, RelayState, router};

#[test]
fn in_memory_transport_carries_a_full_conversation() {
    let transport = InMemoryTransport::new();
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let a_id = alice.account_id();
    let b_id = bob.account_id();

    let card = bob.publish_card();
    let (route, flight) = alice.initiate(&card, b"boot").unwrap();
    assert_eq!(route, b_id);
    transport.submit(&route, &flight).unwrap();
    let blob = transport
        .poll(&b_id, &PollAuth::unauthenticated())
        .unwrap()
        .expect("flight queued");
    assert_eq!(bob.receive(&blob).unwrap(), (a_id, b"boot".to_vec()));

    // Many messages, both directions, through the (multi-slot) in-memory transport.
    for i in 0..8u32 {
        let a_text = format!("alice {i}");
        let blob = alice.send(&b_id, a_text.as_bytes()).unwrap();
        transport.submit(&b_id, &blob).unwrap();
        let got = transport
            .poll(&b_id, &PollAuth::unauthenticated())
            .unwrap()
            .expect("queued");
        assert_eq!(bob.receive(&got).unwrap().1, a_text.as_bytes());

        let b_text = format!("bob {i}");
        let blob = bob.send(&a_id, b_text.as_bytes()).unwrap();
        transport.submit(&a_id, &blob).unwrap();
        let got = transport
            .poll(&a_id, &PollAuth::unauthenticated())
            .unwrap()
            .expect("queued");
        assert_eq!(alice.receive(&got).unwrap().1, b_text.as_bytes());
    }
}

/// Start the real relay on an ephemeral loopback port; returns its base URL.
fn start_relay() -> String {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async move {
            let store: Arc<tokio::sync::Mutex<dyn QueueStore>> =
                Arc::new(tokio::sync::Mutex::new(InMemoryQueueStore::new()));
            let app = router(RelayState::new(store, Arc::new(NoopPushSender)));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind ephemeral port");
            let port = listener.local_addr().expect("local addr").port();
            tx.send(port).expect("send port");
            axum::serve(listener, app).await.expect("relay serve");
        });
    });
    let port = rx.recv().expect("relay port");
    // Give the accept loop a moment to come up.
    std::thread::sleep(Duration::from_millis(200));
    format!("http://127.0.0.1:{port}")
}

fn poll_until(transport: &RelayTransport, client: &MercuryClient) -> Vec<u8> {
    let route = client.account_id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let auth = client.sign_poll_auth(ts);
    for _ in 0..100 {
        if let Some(blob) = transport.poll(&route, &auth).expect("poll the relay") {
            return blob;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("no message delivered by the relay within the timeout");
}

#[test]
fn two_clients_exchange_through_the_running_relay() {
    let base = start_relay();
    let transport = RelayTransport::new(&base);

    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let a_id = alice.account_id();
    let b_id = bob.account_id();

    // Bob publishes a card (out-of-band); Alice opens a session and submits the first
    // flight to Bob's route through the REAL relay.
    let card = bob.publish_card();
    let (route, flight) = alice.initiate(&card, b"hello over the relay").unwrap();
    assert_eq!(route, b_id);
    transport
        .submit(&route, &flight)
        .expect("submit first flight");

    // Bob polls his route, receives the opaque blob, and decrypts it end-to-end.
    let blob = poll_until(&transport, &bob);
    let (from, msg) = bob.receive(&blob).unwrap();
    assert_eq!(from, a_id);
    assert_eq!(msg, b"hello over the relay");

    // Bob replies through the relay on Alice's route.
    let reply = bob.send(&a_id, b"hi alice, got it").unwrap();
    transport.submit(&a_id, &reply).expect("submit reply");
    let blob = poll_until(&transport, &alice);
    let (from, msg) = alice.receive(&blob).unwrap();
    assert_eq!(from, b_id);
    assert_eq!(msg, b"hi alice, got it");
}

#[test]
fn clients_discover_each_other_through_the_relay_directory() {
    let base = start_relay();
    let transport = RelayTransport::new(&base);

    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let a_id = alice.account_id();
    let b_id = bob.account_id();

    // Bob publishes his contact card to the REAL relay's directory.
    bob.publish_to_directory(&transport).expect("publish card");

    // Alice discovers Bob from his account id alone, verifies the fetched card, then opens a
    // session and sends the first flight through the relay — no out-of-band card exchange.
    let card = alice
        .fetch_contact(&transport, &b_id)
        .expect("fetch + verify card");
    assert_eq!(card.account_id(), b_id);
    let (route, flight) = alice.initiate(&card, b"hello via directory").unwrap();
    assert_eq!(route, b_id);
    transport
        .submit(&route, &flight)
        .expect("submit first flight");

    let blob = poll_until(&transport, &bob);
    let (from, msg) = bob.receive(&blob).unwrap();
    assert_eq!(from, a_id);
    assert_eq!(msg, b"hello via directory");

    // A fetch for an account id with no published card is a clean not-found.
    assert_eq!(
        alice.fetch_contact(&transport, &[0x12u8; 32]).unwrap_err(),
        mercury_client::ClientError::ContactNotFound
    );
}

#[test]
fn sign_poll_auth_proves_route_ownership() {
    // The client's poll authorization must be a valid route-ownership proof for its OWN route —
    // exactly what the hardened relay's `verify_relay_route_pop` will check on poll/ack/delete.
    let client = MercuryClient::new();
    let route = client.account_id();
    let auth = client.sign_poll_auth(1_700_000_000);

    assert!(
        mercury_keys::verify_relay_route_pop(
            &auth.identity_pub,
            &route,
            mercury_keys::RelayRouteOperation::Poll,
            auth.timestamp_s,
            &auth.pop_sig
        ),
        "sign_poll_auth must produce a proof the relay accepts for our own route"
    );
    // The same proof does NOT authorize a route we do not own.
    assert!(!mercury_keys::verify_relay_route_pop(
        &auth.identity_pub,
        &[0x42u8; 32],
        mercury_keys::RelayRouteOperation::Poll,
        auth.timestamp_s,
        &auth.pop_sig
    ));
    // A tampered timestamp breaks it (so the relay's freshness check is non-bypassable).
    assert!(!mercury_keys::verify_relay_route_pop(
        &auth.identity_pub,
        &route,
        mercury_keys::RelayRouteOperation::Poll,
        auth.timestamp_s + 1,
        &auth.pop_sig
    ));
    // The poll proof is bound to the Poll operation: it must NOT authorize an ack or delete,
    // so a captured poll proof cannot drain/destroy the queue via another endpoint.
    assert!(!mercury_keys::verify_relay_route_pop(
        &auth.identity_pub,
        &route,
        mercury_keys::RelayRouteOperation::Delete,
        auth.timestamp_s,
        &auth.pop_sig
    ));
}
