//! Directory tests: a client discovers a peer's contact card through the (untrusted) relay
//! directory instead of an out-of-band exchange, and ALWAYS re-verifies the card before use.
//!
//! Two security boundaries are exercised:
//!   * PUBLISH authorization — the relay (and the faithful in-memory double) only store a card
//!     under the slot derived from a valid proof of possession, so nobody can publish over
//!     someone else's slot.
//!   * FETCH verification — every fetched card is re-verified locally (signature + account-id
//!     binding + request-id match), so even a malicious relay that returns a planted card
//!     cannot trick the client into using the wrong/forged one. The `MaliciousRelay` mock
//!     below returns an attacker-chosen card for ANY fetch to prove this.

use mercury_client::{
    ClientError, InMemoryTransport, MercuryClient, PollAuth, Transport, TransportError,
};
use mercury_keys::IdentityKeyPair;

/// A malicious/buggy relay double: returns a planted card for ANY fetch (ignoring the
/// requested account id) and accepts every publish. Used to prove the client re-verifies a
/// fetched card rather than trusting the directory.
struct MaliciousRelay {
    planted: Vec<u8>,
}

impl Transport for MaliciousRelay {
    fn submit(&self, _route: &[u8; 32], _blob: &[u8]) -> Result<(), TransportError> {
        Ok(())
    }
    fn poll(&self, _route: &[u8; 32], _auth: &PollAuth) -> Result<Option<Vec<u8>>, TransportError> {
        Ok(None)
    }
    fn publish_card(
        &self,
        _identity_pub: &[u8; 32],
        _card: &[u8],
        _pop_sig: &[u8; 64],
    ) -> Result<(), TransportError> {
        Ok(())
    }
    fn fetch_card(&self, _account_id: &[u8; 32]) -> Result<Option<Vec<u8>>, TransportError> {
        Ok(Some(self.planted.clone()))
    }
}

#[test]
fn discover_and_initiate_through_the_directory() {
    let transport = InMemoryTransport::new();
    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let a_id = alice.account_id();
    let b_id = bob.account_id();

    // Bob publishes his card to the directory (authorized by his proof of possession).
    bob.publish_to_directory(&transport).unwrap();

    // Alice discovers Bob purely from his account id, verifies the fetched card, and opens a
    // session against it.
    let card = alice
        .fetch_contact(&transport, &b_id)
        .expect("verified card");
    assert_eq!(card.account_id(), b_id);
    let (route, flight) = alice.initiate(&card, b"hello from the directory").unwrap();
    assert_eq!(route, b_id);
    transport.submit(&route, &flight).unwrap();

    let blob = transport
        .poll(&b_id, &PollAuth::unauthenticated())
        .unwrap()
        .expect("flight queued");
    let (from, msg) = bob.receive(&blob).unwrap();
    assert_eq!(from, a_id);
    assert_eq!(msg, b"hello from the directory");

    // The conversation continues both ways over the established session.
    let reply = bob.send(&a_id, b"hi alice").unwrap();
    transport.submit(&a_id, &reply).unwrap();
    assert_eq!(
        alice
            .receive(
                &transport
                    .poll(&a_id, &PollAuth::unauthenticated())
                    .unwrap()
                    .unwrap()
            )
            .unwrap(),
        (b_id, b"hi alice".to_vec())
    );
}

#[test]
fn in_memory_transport_rejects_an_unauthorized_publish() {
    // The faithful relay double enforces the proof of possession: a corrupted proof is
    // refused, never stored.
    let transport = InMemoryTransport::new();
    let identity = IdentityKeyPair::generate();
    let card = b"an opaque card";
    let mut pop = mercury_keys::sign_directory_publish(&identity, card);
    pop[0] ^= 0x01;
    assert!(
        transport
            .publish_card(identity.public().as_bytes(), card, &pop)
            .is_err()
    );
    // And a valid proof IS accepted, landing in the derived slot.
    let good = mercury_keys::sign_directory_publish(&identity, card);
    assert!(
        transport
            .publish_card(identity.public().as_bytes(), card, &good)
            .is_ok()
    );
}

#[test]
fn fetch_contact_rejects_an_account_id_mismatch() {
    // A malicious relay returns some OTHER user's (perfectly valid) card for the account id
    // Alice asked about. Alice must reject it rather than establish a session with the wrong
    // person.
    let mut bob = MercuryClient::new();
    let b_id = bob.account_id();
    let relay = MaliciousRelay {
        planted: bob.publish_card().to_json().into_bytes(),
    };

    let alice = MercuryClient::new();
    let other_id = [0x99u8; 32];
    assert_ne!(other_id, b_id);
    // The relay returns Bob's card, but Alice asked for `other_id` -> reject.
    assert_eq!(
        alice.fetch_contact(&relay, &other_id).unwrap_err(),
        ClientError::DirectoryMismatch
    );
}

#[test]
fn fetch_contact_rejects_a_tampered_card() {
    // The relay flips a byte of the bundle signature. The identity signature no longer
    // verifies, so the card is rejected — the relay cannot forge a card without the identity
    // private key.
    let mut bob = MercuryClient::new();
    let b_id = bob.account_id();
    let mut card = bob.publish_card();
    card.bundle.signature[0] ^= 0x01;
    let relay = MaliciousRelay {
        planted: card.to_json().into_bytes(),
    };

    let alice = MercuryClient::new();
    let result = alice.fetch_contact(&relay, &b_id);
    assert!(
        matches!(result, Err(ClientError::Session(_))),
        "a tampered bundle signature must fail verification, got {result:?}"
    );
}

#[test]
fn fetch_contact_rejects_a_tampered_account_id_binding() {
    // The attacker rewrites the card's account id to match the requested slot, hoping the
    // request-binding check passes. But the bundle signature covers the account id, so verify()
    // fails the signature/binding check. Fail-closed either way.
    let mut bob = MercuryClient::new();
    let mut card = bob.publish_card();
    let forged_slot = [0x55u8; 32];
    card.bundle.account_id = forged_slot;
    let relay = MaliciousRelay {
        planted: card.to_json().into_bytes(),
    };

    let alice = MercuryClient::new();
    let result = alice.fetch_contact(&relay, &forged_slot);
    assert!(
        matches!(result, Err(ClientError::Session(_))),
        "a rebound account id must fail the bundle signature/binding check, got {result:?}"
    );
}

#[test]
fn fetch_contact_not_found_is_distinct() {
    let transport = InMemoryTransport::new();
    let alice = MercuryClient::new();
    let result = alice.fetch_contact(&transport, &[0x77u8; 32]);
    assert_eq!(result.unwrap_err(), ClientError::ContactNotFound);
}

#[test]
fn fetch_contact_rejects_garbage_card_bytes() {
    let relay = MaliciousRelay {
        planted: b"this is not a json card".to_vec(),
    };
    let alice = MercuryClient::new();
    let result = alice.fetch_contact(&relay, &[0x33u8; 32]);
    assert_eq!(result.unwrap_err(), ClientError::Malformed);
}
