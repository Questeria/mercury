//! Integration tests for the Mercury client runtime: two clients establish a 1:1
//! post-quantum ratchet session and exchange end-to-end-encrypted messages, plus
//! tamper/replay/fuzz/fail-closed checks.

use mercury_client::{ClientError, ContactCard, MercuryClient};

#[test]
fn two_clients_exchange_end_to_end_bidirectionally() {
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let a_id = alice.account_id();
    let b_id = bob.account_id();
    assert_ne!(a_id, b_id);

    // Bob publishes a contact card; Alice opens a session and sends the first message.
    let card = bob.publish_card();
    assert_eq!(card.account_id(), b_id);
    let (route, flight) = alice.initiate(&card, b"hello bob").expect("initiate");
    assert_eq!(route, b_id, "the route is the recipient's account id");

    // Bob accepts the first flight and recovers the plaintext.
    let (from, msg) = bob.receive(&flight).expect("bob receives first flight");
    assert_eq!(from, a_id);
    assert_eq!(msg, b"hello bob");
    assert!(bob.has_session(&a_id) && alice.has_session(&b_id));

    // Now both directions work, across many messages (the ratchet advances each turn).
    for i in 0..16u32 {
        let a_text = format!("alice says {i}");
        let blob = alice.send(&b_id, a_text.as_bytes()).expect("alice send");
        let (from, got) = bob.receive(&blob).expect("bob recv");
        assert_eq!(from, a_id);
        assert_eq!(got, a_text.as_bytes());

        let b_text = format!("bob says {i}");
        let blob = bob.send(&a_id, b_text.as_bytes()).expect("bob send");
        let (from, got) = alice.receive(&blob).expect("alice recv");
        assert_eq!(from, b_id);
        assert_eq!(got, b_text.as_bytes());
    }
}

#[test]
fn out_of_order_messages_round_trip() {
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let b_id = bob.account_id();
    let card = bob.publish_card();
    let (_route, flight) = alice.initiate(&card, b"boot").unwrap();
    assert_eq!(bob.receive(&flight).unwrap().1, b"boot");

    // Alice sends three; Bob receives them reversed (the ratchet's skipped-key cache).
    let m0 = alice.send(&b_id, b"one").unwrap();
    let m1 = alice.send(&b_id, b"two").unwrap();
    let m2 = alice.send(&b_id, b"three").unwrap();
    assert_eq!(bob.receive(&m2).unwrap().1, b"three");
    assert_eq!(bob.receive(&m0).unwrap().1, b"one");
    assert_eq!(bob.receive(&m1).unwrap().1, b"two");
}

#[test]
fn a_tampered_blob_is_rejected_without_losing_the_session() {
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let b_id = bob.account_id();
    let card = bob.publish_card();
    let (_r, flight) = alice.initiate(&card, b"boot").unwrap();
    bob.receive(&flight).unwrap();

    let good = alice.send(&b_id, b"genuine").unwrap();
    let mut bad = good.clone();
    let last = bad.len() - 1;
    bad[last] ^= 0x01;
    assert!(bob.receive(&bad).is_err(), "tampered blob must be rejected");
    // The session survives: the genuine message still decrypts.
    assert_eq!(bob.receive(&good).unwrap().1, b"genuine");
}

#[test]
fn a_replayed_message_is_rejected() {
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let b_id = bob.account_id();
    let card = bob.publish_card();
    let (_r, flight) = alice.initiate(&card, b"boot").unwrap();
    bob.receive(&flight).unwrap();

    let m = alice.send(&b_id, b"once").unwrap();
    assert_eq!(bob.receive(&m).unwrap().1, b"once");
    assert!(bob.receive(&m).is_err(), "a replay must be rejected");
}

#[test]
fn a_third_party_cannot_open_the_first_flight() {
    // Eve publishes her own card (so she has a retained key) but cannot decapsulate a
    // flight that Alice encapsulated to Bob's key.
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let mut eve = MercuryClient::new();
    let card = bob.publish_card();
    let _eve_card = eve.publish_card();
    let (_r, flight) = alice.initiate(&card, b"for bob only").unwrap();
    assert!(
        eve.receive(&flight).is_err(),
        "a third party must not open a flight addressed to someone else"
    );
}

// (The self-addressed-first-flight guard is now a white-box test in lib.rs, because the
// outer sealed-sender envelope wraps the inner frame the guard inspects.)

#[test]
fn sending_without_a_session_fails_closed() {
    let mut alice = MercuryClient::new();
    let nobody = [9u8; 32];
    assert_eq!(alice.send(&nobody, b"hi"), Err(ClientError::UnknownPeer));
}

#[test]
fn contact_card_json_round_trips() {
    let mut bob = MercuryClient::new();
    let card = bob.publish_card();
    let json = card.to_json();
    let parsed = ContactCard::from_json(&json).expect("card json parses");
    assert_eq!(parsed.account_id(), card.account_id());

    // A real (parsed-from-json) card still establishes a session.
    let mut alice = MercuryClient::new();
    let (_r, flight) = alice.initiate(&parsed, b"via json card").unwrap();
    assert_eq!(bob.receive(&flight).unwrap().1, b"via json card");

    // Malformed json fails closed.
    assert!(matches!(
        ContactCard::from_json("not json"),
        Err(ClientError::Malformed)
    ));
}

#[test]
fn a_client_survives_an_encrypted_pickle() {
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let a_id = alice.account_id();
    let b_id = bob.account_id();

    let card = bob.publish_card();
    let (_route, flight) = alice.initiate(&card, b"boot").unwrap();
    assert_eq!(bob.receive(&flight).unwrap().1, b"boot");
    let m = alice.send(&b_id, b"before restart").unwrap();
    assert_eq!(bob.receive(&m).unwrap().1, b"before restart");

    // Pickle both clients, drop them, restore from the encrypted blobs.
    let key = [0x11u8; 32];
    let a_blob = alice.to_encrypted_pickle(&key);
    let b_blob = bob.to_encrypted_pickle(&key);
    drop(alice);
    drop(bob);
    let mut alice2 = MercuryClient::from_encrypted_pickle(&a_blob, &key).expect("restore alice");
    let mut bob2 = MercuryClient::from_encrypted_pickle(&b_blob, &key).expect("restore bob");
    assert_eq!(alice2.account_id(), a_id);
    assert_eq!(bob2.account_id(), b_id);
    assert!(alice2.has_session(&b_id) && bob2.has_session(&a_id));

    // The conversation continues across the restart, both directions.
    let m = alice2.send(&b_id, b"after restart").unwrap();
    assert_eq!(bob2.receive(&m).unwrap().1, b"after restart");
    let r = bob2.send(&a_id, b"reply after restart").unwrap();
    assert_eq!(alice2.receive(&r).unwrap().1, b"reply after restart");

    // Fail closed: a wrong key and a garbage blob are rejected, never a panic.
    assert!(matches!(
        MercuryClient::from_encrypted_pickle(&a_blob, &[9u8; 32]),
        Err(ClientError::Pickle)
    ));
    assert!(MercuryClient::from_encrypted_pickle(b"x", &key).is_err());
}

#[test]
fn a_client_restores_every_peer_session_from_one_pickle() {
    // D holds CONCURRENT sessions with THREE peers; one encrypted pickle must restore them ALL — each
    // ratchet session AND its matching peer_device entry — not just the first. The single-peer test
    // above cannot catch a restore that truncates to one or mismatches the device keys; the normal,
    // multi-peer case (what a user actually has after a few conversations) was previously untested.
    let mut d = MercuryClient::new();
    let d_id = d.account_id();

    let mut peers: Vec<(MercuryClient, [u8; 32])> = Vec::new();
    for _ in 0..3 {
        let mut p = MercuryClient::new();
        let p_id = p.account_id();
        let card = p.publish_card();
        // D initiates to P; P receives the first flight -> both hold the session, and D records P's
        // sealed-sender device key (peer_device[P]) from the card.
        let (_route, flight) = d.initiate(&card, b"hello").unwrap();
        assert_eq!(p.receive(&flight).unwrap().1, b"hello");
        assert!(d.has_session(&p_id), "D holds this peer's session pre-pickle");
        peers.push((p, p_id));
    }

    // One pickle, drop, restore.
    let key = [0x22u8; 32];
    let blob = d.to_encrypted_pickle(&key);
    drop(d);
    let mut d2 = MercuryClient::from_encrypted_pickle(&blob, &key).expect("restore D");
    assert_eq!(d2.account_id(), d_id);

    // EVERY peer survived the restore: has_session holds, a send succeeds (a lost peer_device entry
    // would be Err(UnknownPeer) even WITH the session present), the peer decrypts it, and its reply
    // decrypts back on D2 — proving both halves of the per-peer state restored, for all three.
    for (p, p_id) in peers.iter_mut() {
        assert!(d2.has_session(p_id), "restored D2 still holds this peer's session");
        let m = d2
            .send(p_id, b"after restart")
            .expect("send to a restored peer (session + peer_device both survived)");
        assert_eq!(p.receive(&m).unwrap().1, b"after restart");
        let r = p.send(&d_id, b"reply").unwrap();
        assert_eq!(d2.receive(&r).unwrap().1, b"reply");
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

#[test]
fn receive_survives_arbitrary_bytes() {
    // The inbound parser + crypto must never panic on arbitrary/malformed blobs.
    let mut rng = Rng(0x1234_5678_9abc_def0);
    for _ in 0..5_000 {
        let mut client = MercuryClient::new();
        let _ = client.publish_card(); // give it a retained key so the accept path is reachable
        let len = (rng.next() % 2600) as usize;
        let mut data: Vec<u8> = (0..len).map(|_| (rng.next() & 0xff) as u8).collect();
        // Bias some inputs toward valid framing so the parser proceeds past the header.
        if data.len() >= 2 {
            data[0] = 1;
            data[1] = (rng.next() & 1) as u8;
        }
        let _ = client.receive(&data); // only Ok/Err, never a panic
    }
}
