//! Integration tests for the Mercury 1:1 session engine.
//!
//! Exercises the signed pre-key bundle binding, the bidirectional Double
//! Ratchet handshake, replay/tamper rejection, encrypted-pickle persistence,
//! and carriage of session ciphertext through the existing message envelope.

use mercury_core::{ClientMessageEnvelope, ConversationPolicyState, MessageKind, ProtocolSuite};
use mercury_keys::{DeviceKeyPair, IdentityKeyPair, MlKemKeyPair};
use mercury_session::{
    HybridFirstFlight, HybridMessage, HybridRatchetSession, HybridSession, LocalSessionAccount,
    MercurySession, PQ_MAX_SKIP, RatchetMessage, SessionCiphertext, SessionError,
    SignedPreKeyBundle,
};

/// A freshly-built account + identity + ML-KEM keypair + published bundle, the
/// common setup. The `pq_keypair` is the recipient's post-quantum key (its public
/// half is in the bundle); the initiator encapsulates to it for the hybrid flight.
struct Party {
    account: LocalSessionAccount,
    identity: IdentityKeyPair,
    pq_keypair: MlKemKeyPair,
    bundle: SignedPreKeyBundle,
}

fn new_party(one_time_keys: usize) -> Party {
    let mut account = LocalSessionAccount::new();
    let identity = IdentityKeyPair::generate();
    let pq_keypair = MlKemKeyPair::generate();
    let bundle = account.publish_bundle(&identity, &pq_keypair.public(), one_time_keys);
    Party {
        account,
        identity,
        pq_keypair,
        bundle,
    }
}

// ---------------------------------------------------------------------------
// Bundle binding
// ---------------------------------------------------------------------------

#[test]
fn published_bundle_verifies() {
    let party = new_party(5);
    assert!(party.bundle.verify().is_ok());
}

#[test]
fn flipped_signature_byte_fails_verify() {
    let mut party = new_party(3);
    party.bundle.signature[10] ^= 0x01;
    assert_eq!(party.bundle.verify(), Err(SessionError::BundleSignature));
}

#[test]
fn corrupt_account_id_fails_verify() {
    // Flip an account_id byte. The signature was computed over the original
    // account_id, so verification must reject before the id-binding check; the
    // contract only requires that a corrupt bundle is rejected. We assert it is
    // one of the two binding failures.
    let mut party = new_party(3);
    party.bundle.account_id[0] ^= 0xFF;
    let err = party.bundle.verify().unwrap_err();
    assert!(
        matches!(
            err,
            SessionError::AccountIdMismatch | SessionError::BundleSignature
        ),
        "expected a binding failure, got {err:?}"
    );
}

#[test]
fn account_id_mismatch_with_valid_signature_is_detected() {
    // Re-sign a bundle whose account_id has been swapped to a *different* valid
    // identity's id while keeping the original signer. This isolates the
    // account-id binding check: signature verifies, but the id does not match
    // the signer's identity key, so we must get AccountIdMismatch.
    let mut party = new_party(3);
    let other = IdentityKeyPair::generate();
    let foreign_id = mercury_keys::account_id_from_identity_key(&other.public());
    party.bundle.account_id = foreign_id;

    // Recompute a valid signature over the tampered bundle using the ORIGINAL
    // signer so the signature check passes and only the id binding can fail.
    party.bundle.signature = sign_bundle(&party.identity, &party.bundle);

    assert_eq!(party.bundle.verify(), Err(SessionError::AccountIdMismatch));
}

/// Reconstruct the canonical `bundle_tbs` (v2 layout: adds the length-prefixed
/// ML-KEM PQ key) and sign it with `identity`. Mirrors the in-crate signing logic
/// so tests can forge specific edge cases.
fn sign_bundle(identity: &IdentityKeyPair, bundle: &SignedPreKeyBundle) -> [u8; 64] {
    let mut tbs = Vec::new();
    tbs.extend_from_slice(b"mercury-prekey-bundle-v2");
    tbs.extend_from_slice(&bundle.account_id);
    tbs.extend_from_slice(&bundle.curve25519);
    tbs.extend_from_slice(&bundle.ed25519);
    tbs.extend_from_slice(&bundle.one_time_key);
    match bundle.fallback {
        Some(fb) => {
            tbs.push(1u8);
            tbs.extend_from_slice(&fb);
        }
        None => tbs.push(0u8),
    }
    tbs.extend_from_slice(&(bundle.pq_kem.len() as u32).to_le_bytes());
    tbs.extend_from_slice(&bundle.pq_kem);
    identity.sign(&tbs)
}

// ---------------------------------------------------------------------------
// Handshake + bidirectional ratchet
// ---------------------------------------------------------------------------

#[test]
fn handshake_and_bidirectional_ratchet() {
    let a = new_party(5);
    let mut b = new_party(5);

    // A establishes outbound toward B and sends the first (pre-key) message.
    let mut a_session = MercurySession::establish_outbound(&a.account, &b.bundle)
        .expect("A should establish outbound to B");
    let ct_m1 = a_session.encrypt(b"m1").expect("encrypt m1");
    assert!(
        ct_m1.is_pre_key(),
        "first outbound message must be a pre-key message"
    );

    // B establishes inbound from A's pre-key ciphertext, recovering m1.
    let a_curve = a.account.curve25519_key();
    let (mut b_session, m1) = MercurySession::establish_inbound(&mut b.account, a_curve, &ct_m1)
        .expect("B should establish inbound from A");
    assert_eq!(m1, b"m1");

    // B -> A.
    let ct_m2 = b_session.encrypt(b"m2").expect("encrypt m2");
    assert_eq!(a_session.decrypt(&ct_m2).expect("A decrypts m2"), b"m2");

    // A -> B.
    let ct_m3 = a_session.encrypt(b"m3").expect("encrypt m3");
    assert_eq!(b_session.decrypt(&ct_m3).expect("B decrypts m3"), b"m3");

    // Both ends negotiated Olm v2 (untruncated MAC). This is the standing
    // assertion that backs the V2 downgrade guard: establish_* reject anything
    // that is not v2 before returning it.
    assert!(a_session.is_version_2(), "outbound session must be Olm v2");
    assert!(b_session.is_version_2(), "inbound session must be Olm v2");
}

// ---------------------------------------------------------------------------
// V2 downgrade guard (security)
// ---------------------------------------------------------------------------

#[test]
fn established_sessions_report_version_2() {
    // A direct attestation that both established sides are Olm v2. The
    // establish_* methods reject any non-v2 session with VersionDowngrade, so a
    // truncated-MAC (v1) session can never be handed back.
    let a = new_party(5);
    let mut b = new_party(5);

    let mut a_session =
        MercurySession::establish_outbound(&a.account, &b.bundle).expect("outbound");
    assert!(a_session.is_version_2(), "outbound must be v2");

    let ct_m1 = a_session.encrypt(b"m1").expect("encrypt m1");
    let a_curve = a.account.curve25519_key();
    let (b_session, _) =
        MercurySession::establish_inbound(&mut b.account, a_curve, &ct_m1).expect("inbound");
    assert!(b_session.is_version_2(), "inbound must be v2");
}

#[test]
fn establish_inbound_rejects_v1_downgrade() {
    // SECURITY: vodozemac derives the Olm version from the incoming pre-key
    // message, so a sender can offer a v1 (truncated-8-byte-MAC) pre-key message
    // intending to create a v1 session even though our account passes
    // version_2(). We synthesize a genuine v1 pre-key message with a raw
    // vodozemac account and confirm establish_inbound REJECTS it (no v1 session
    // is ever returned).
    //
    // Defense is layered. With the `experimental-session-config` feature (which
    // this crate enables), vodozemac 0.10 itself compares the message-derived
    // version against the `expected_config` we pass and rejects a v1 message at
    // `create_inbound_session` time -> we surface that as `SessionCreation`. Our
    // own post-creation `session_config() != version_2()` check is the
    // belt-and-suspenders backstop that yields `VersionDowngrade` should that
    // upstream comparison ever be relaxed. The security contract is "never
    // return a non-v2 inbound session"; either rejection satisfies it, so we
    // accept both error variants.
    let mut b = new_party(5);

    // A v1 sender, built directly on vodozemac, targeting B's published bundle.
    let attacker = vodozemac::olm::Account::new();
    let attacker_curve = attacker.curve25519_key().to_bytes();
    let mut v1_session = attacker
        .create_outbound_session(
            vodozemac::olm::SessionConfig::version_1(),
            vodozemac::Curve25519PublicKey::from_bytes(b.bundle.curve25519),
            vodozemac::Curve25519PublicKey::from_bytes(b.bundle.one_time_key),
        )
        .expect("v1 outbound session against B's bundle");

    // Encrypt under v1: the result is a v1 pre-key message.
    let (msg_type, body) = v1_session
        .encrypt(b"downgrade")
        .expect("v1 encrypt")
        .to_parts();
    let v1_pre_key = SessionCiphertext {
        msg_type: msg_type as u8,
        body,
    };
    assert!(
        v1_pre_key.is_pre_key(),
        "synthesized v1 message must be a pre-key message"
    );

    // No v1 session may be handed back: reject with either the upstream
    // config-mismatch (SessionCreation) or our downgrade backstop.
    let err = MercurySession::establish_inbound(&mut b.account, attacker_curve, &v1_pre_key)
        .map(|_| ())
        .expect_err("v1 pre-key message must be rejected");
    assert!(
        matches!(
            err,
            SessionError::VersionDowngrade | SessionError::SessionCreation
        ),
        "expected a downgrade rejection, got {err:?}"
    );
}

#[test]
fn establish_inbound_rejects_normal_message() {
    let a = new_party(5);
    let mut b = new_party(5);

    let mut a_session =
        MercurySession::establish_outbound(&a.account, &b.bundle).expect("outbound");
    let ct_m1 = a_session.encrypt(b"m1").expect("encrypt m1");
    let a_curve = a.account.curve25519_key();
    let (mut b_session, _) =
        MercurySession::establish_inbound(&mut b.account, a_curve, &ct_m1).expect("inbound");

    // Once B has replied and A has decrypted, A's next message is a NORMAL
    // message. Feeding a normal message to establish_inbound must be rejected.
    let ct_reply = b_session.encrypt(b"reply").expect("encrypt reply");
    a_session.decrypt(&ct_reply).expect("A decrypts reply");
    let normal = a_session
        .encrypt(b"normal-now")
        .expect("encrypt normal-now");
    assert!(!normal.is_pre_key());

    let mut fresh = new_party(5);
    // `MercurySession` has no `Debug` (it holds ratchet secrets), so map the Ok
    // arm away before asserting on the error.
    let result =
        MercurySession::establish_inbound(&mut fresh.account, a_curve, &normal).map(|_| ());
    assert_eq!(result, Err(SessionError::NotPreKeyMessage));
}

// ---------------------------------------------------------------------------
// Negative: replay and tamper
// ---------------------------------------------------------------------------

#[test]
fn replayed_ciphertext_is_rejected() {
    let a = new_party(5);
    let mut b = new_party(5);

    let mut a_session =
        MercurySession::establish_outbound(&a.account, &b.bundle).expect("outbound");
    let ct_m1 = a_session.encrypt(b"m1").expect("encrypt m1");
    let a_curve = a.account.curve25519_key();
    let (mut b_session, _) =
        MercurySession::establish_inbound(&mut b.account, a_curve, &ct_m1).expect("inbound");

    // Establish the channel both directions so A sends normal messages.
    let ct_b = b_session.encrypt(b"hi").expect("encrypt hi");
    a_session.decrypt(&ct_b).expect("A decrypts hi");

    let ct = a_session.encrypt(b"once").expect("encrypt once");
    assert_eq!(b_session.decrypt(&ct).expect("first decrypt"), b"once");
    // Replaying the same ciphertext must fail: the message key is consumed.
    assert_eq!(b_session.decrypt(&ct), Err(SessionError::Decryption));
}

#[test]
fn tampered_body_is_rejected() {
    let a = new_party(5);
    let mut b = new_party(5);

    let mut a_session =
        MercurySession::establish_outbound(&a.account, &b.bundle).expect("outbound");
    let ct_m1 = a_session.encrypt(b"m1").expect("encrypt m1");
    let a_curve = a.account.curve25519_key();
    let (mut b_session, _) =
        MercurySession::establish_inbound(&mut b.account, a_curve, &ct_m1).expect("inbound");
    let ct_b = b_session.encrypt(b"hi").expect("encrypt hi");
    a_session.decrypt(&ct_b).expect("A decrypts hi");

    let mut ct = a_session.encrypt(b"intact").expect("encrypt intact");
    // Flip a byte near the end of the body (the MAC/ciphertext region).
    let last = ct.body.len() - 1;
    ct.body[last] ^= 0x01;
    assert_eq!(b_session.decrypt(&ct), Err(SessionError::Decryption));
}

// ---------------------------------------------------------------------------
// Persistence (encrypted pickle)
// ---------------------------------------------------------------------------

#[test]
fn pickle_roundtrip_continues_session() {
    let a = new_party(5);
    let mut b = new_party(5);

    let mut a_session =
        MercurySession::establish_outbound(&a.account, &b.bundle).expect("outbound");
    let ct_m1 = a_session.encrypt(b"m1").expect("encrypt m1");
    let a_curve = a.account.curve25519_key();
    let (mut b_session, _) =
        MercurySession::establish_inbound(&mut b.account, a_curve, &ct_m1).expect("inbound");
    let ct_b = b_session.encrypt(b"hello").expect("encrypt hello");
    a_session.decrypt(&ct_b).expect("A decrypts hello");

    // Persist B's live session, restore it, and keep decrypting from A.
    let key = [7u8; 32];
    let blob = b_session.to_encrypted_pickle(&key);
    let mut restored =
        MercurySession::from_encrypted_pickle(&blob, &key).expect("restore with correct key");

    let ct_next = a_session
        .encrypt(b"after-restore")
        .expect("encrypt after-restore");
    assert_eq!(
        restored
            .decrypt(&ct_next)
            .expect("restored session decrypts"),
        b"after-restore"
    );

    // Wrong pickle key must fail closed.
    let wrong = [9u8; 32];
    assert_eq!(
        MercurySession::from_encrypted_pickle(&blob, &wrong).map(|_| ()),
        Err(SessionError::Pickle)
    );
}

#[test]
fn account_pickle_roundtrip() {
    let mut account = LocalSessionAccount::new();
    let identity = IdentityKeyPair::generate();
    let pq = MlKemKeyPair::generate();
    let _ = account.publish_bundle(&identity, &pq.public(), 3);

    let key = [3u8; 32];
    let blob = account.to_encrypted_pickle(&key);
    let restored =
        LocalSessionAccount::from_encrypted_pickle(&blob, &key).expect("restore account");
    // Identity (curve) key is stable across the pickle roundtrip.
    assert_eq!(restored.curve25519_key(), account.curve25519_key());

    assert_eq!(
        LocalSessionAccount::from_encrypted_pickle(&blob, &[0u8; 32]).map(|_| ()),
        Err(SessionError::Pickle)
    );
}

#[test]
fn repeated_publish_bundle_keeps_otk_state_consistent() {
    // OTK accounting must be atomic: publish_bundle generates the keys, selects
    // the bundle's OTK from that fresh set, then marks them published. Calling it
    // twice must yield two independently valid bundles whose one-time keys differ
    // (the second draws from a freshly generated, not-yet-published set).
    let mut account = LocalSessionAccount::new();
    let identity = IdentityKeyPair::generate();
    let pq = MlKemKeyPair::generate();

    let first = account.publish_bundle(&identity, &pq.public(), 3);
    let second = account.publish_bundle(&identity, &pq.public(), 3);

    assert!(first.verify().is_ok(), "first bundle must verify");
    assert!(second.verify().is_ok(), "second bundle must verify");
    assert_ne!(
        first.one_time_key, second.one_time_key,
        "second bundle must offer a fresh one-time key, not a reused or stale one"
    );

    // Both freshly published one-time keys remain usable for a real handshake,
    // confirming neither bundle was marked published without a live key behind
    // it. Establish an inbound session from a peer that used the SECOND bundle.
    let a = new_party(5);
    let mut a_session =
        MercurySession::establish_outbound(&a.account, &second).expect("outbound to 2nd bundle");
    let ct = a_session.encrypt(b"hi-2nd").expect("encrypt");
    let (mut inbound, pt) =
        MercurySession::establish_inbound(&mut account, a.account.curve25519_key(), &ct)
            .expect("inbound from 2nd bundle");
    assert_eq!(pt, b"hi-2nd");

    // Round-trip a reply so the channel is proven live in both directions.
    let reply = inbound.encrypt(b"ack").expect("encrypt ack");
    assert_eq!(a_session.decrypt(&reply).expect("decrypt ack"), b"ack");
}

// ---------------------------------------------------------------------------
// Ciphertext wire round-trip
// ---------------------------------------------------------------------------

#[test]
fn session_ciphertext_bytes_roundtrip() {
    let ct = SessionCiphertext {
        msg_type: 1,
        body: vec![1, 2, 3, 4, 5],
    };
    let bytes = ct.to_bytes();
    let parsed = SessionCiphertext::from_bytes(&bytes).expect("parse");
    assert_eq!(parsed, ct);

    // Empty input is malformed.
    assert_eq!(
        SessionCiphertext::from_bytes(&[]),
        Err(SessionError::Decryption)
    );
}

// ---------------------------------------------------------------------------
// Envelope wiring: carry session ciphertext through the existing policy gate
// ---------------------------------------------------------------------------

#[test]
fn session_ciphertext_flows_through_envelope() {
    // Build a real session ciphertext.
    let a = new_party(5);
    let mut b = new_party(5);
    let mut a_session =
        MercurySession::establish_outbound(&a.account, &b.bundle).expect("outbound");
    let ct_m1 = a_session.encrypt(b"m1").expect("encrypt m1");
    let a_curve = a.account.curve25519_key();
    let (mut b_session, _) =
        MercurySession::establish_inbound(&mut b.account, a_curve, &ct_m1).expect("inbound");
    let ct_reply = b_session.encrypt(b"hi").expect("encrypt hi");
    a_session.decrypt(&ct_reply).expect("A decrypts hi");

    // The application payload we want to deliver is a normal session message.
    let session_ct = a_session
        .encrypt(b"envelope-carried-secret")
        .expect("encrypt envelope-carried-secret");
    let payload = session_ct.to_bytes();

    // Seal the payload inside the existing envelope, gated by the policy:
    // ClassicalDev(257) suite, Application(1) kind, a valid policy context.
    let recipient = DeviceKeyPair::generate();
    let envelope = ClientMessageEnvelope {
        version: 1,
        suite: ProtocolSuite::ClassicalDev,
        conversation_id_len: 32,
        sender_account_id_len: 32,
        sender_device_id_len: 32,
        epoch: 7,
        sequence: 42,
        kind: MessageKind::Application,
        payload_len: 0, // sealing recomputes it
        critical_flags: 0,
        noncritical_flags: 0,
    };
    let context = ConversationPolicyState {
        expected_epoch: 7,
        expected_sequence: 42,
        minimum_suite: ProtocolSuite::ClassicalDev,
        max_payload_len: 8192,
    };

    let sealed = mercury_message::seal_message(&recipient.public(), &payload, envelope, &context)
        .expect("policy must ACCEPT and sealing must succeed");

    // Wire round-trip of the sealed message object.
    let wire = mercury_message::to_bytes(&sealed);
    let parsed = mercury_message::from_bytes(&wire).expect("envelope bytes round-trip");
    assert_eq!(parsed, sealed);

    // Open the envelope, recover the payload bytes, rebuild the ciphertext, and
    // decrypt it on B's session: end-to-end the session ciphertext survived the
    // envelope + policy path.
    let recovered_payload =
        mercury_message::open_message(&recipient, &parsed, &context).expect("envelope opens");
    assert_eq!(recovered_payload, payload);

    let recovered_ct = SessionCiphertext::from_bytes(&recovered_payload).expect("ct parse");
    assert_eq!(
        b_session
            .decrypt(&recovered_ct)
            .expect("B decrypts carried ciphertext"),
        b"envelope-carried-secret"
    );
}

// ---------------------------------------------------------------------------
// Post-quantum hybrid first flight (PQ-augmented bundle + outer ML-KEM AEAD)
// ---------------------------------------------------------------------------

#[test]
fn bundle_carries_an_authenticated_pq_key() {
    // The bundle's ML-KEM key is the right length and is covered by the identity
    // signature (verify() already passed in published_bundle_verifies).
    let party = new_party(5);
    assert_eq!(party.bundle.pq_kem.len(), 1184);
    assert!(party.bundle.verify().is_ok());
    assert!(party.bundle.pq_public().is_ok());
}

#[test]
fn a_tampered_pq_key_breaks_the_identity_signature() {
    // Flipping a byte of the bundle's PQ key must break the identity signature,
    // because the PQ key is folded into the signed to-be-signed bytes.
    let mut party = new_party(5);
    party.bundle.pq_kem[0] ^= 0x01;
    assert_eq!(party.bundle.verify(), Err(SessionError::BundleSignature));
}

#[test]
fn a_wrong_length_pq_key_is_rejected() {
    let mut party = new_party(5);
    party.bundle.pq_kem.truncate(1183);
    assert_eq!(party.bundle.verify(), Err(SessionError::InvalidKey));
}

#[test]
fn hybrid_first_flight_round_trips() {
    // Alice establishes outbound toward Bob with the PQ first flight; Bob
    // decapsulates + unwraps + completes the inbound handshake, recovering the
    // bootstrap plaintext. Then the (classical inner) ratchet carries a follow-up.
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();

    let (mut a_session, flight) =
        MercurySession::establish_outbound_hybrid(&alice, &bob.bundle, b"pq bootstrap hello")
            .expect("alice outbound hybrid");

    let (mut b_session, recovered) = MercurySession::establish_inbound_hybrid(
        &mut bob.account,
        &bob.pq_keypair,
        alice.curve25519_key(),
        &flight,
    )
    .expect("bob inbound hybrid");
    assert_eq!(recovered, b"pq bootstrap hello");

    // The established sessions are real Olm v2 and carry further messages.
    assert!(a_session.is_version_2() && b_session.is_version_2());
    let ct = a_session.encrypt(b"after the pq flight").unwrap();
    assert_eq!(b_session.decrypt(&ct).unwrap(), b"after the pq flight");
}

#[test]
fn a_tampered_mlkem_ciphertext_fails_the_flight() {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let (_a, mut flight) =
        MercurySession::establish_outbound_hybrid(&alice, &bob.bundle, b"secret").unwrap();
    flight.mlkem_ciphertext[0] ^= 0x01; // implicit rejection -> wrong key -> AEAD fails
    assert!(matches!(
        MercurySession::establish_inbound_hybrid(
            &mut bob.account,
            &bob.pq_keypair,
            alice.curve25519_key(),
            &flight,
        ),
        Err(SessionError::FirstFlightFailed)
    ));
}

#[test]
fn a_tampered_outer_layer_fails_the_flight() {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let (_a, mut flight) =
        MercurySession::establish_outbound_hybrid(&alice, &bob.bundle, b"secret").unwrap();
    let last = flight.outer.len() - 1; // inside the AEAD ciphertext/tag
    flight.outer[last] ^= 0x01;
    assert!(matches!(
        MercurySession::establish_inbound_hybrid(
            &mut bob.account,
            &bob.pq_keypair,
            alice.curve25519_key(),
            &flight,
        ),
        Err(SessionError::FirstFlightFailed)
    ));
}

#[test]
fn the_wrong_pq_keypair_fails_the_flight() {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let (_a, flight) =
        MercurySession::establish_outbound_hybrid(&alice, &bob.bundle, b"secret").unwrap();
    // A different ML-KEM keypair decapsulates to an unrelated secret -> outer fails.
    let wrong_pq = MlKemKeyPair::generate();
    assert!(matches!(
        MercurySession::establish_inbound_hybrid(
            &mut bob.account,
            &wrong_pq,
            alice.curve25519_key(),
            &flight,
        ),
        Err(SessionError::FirstFlightFailed)
    ));
}

#[test]
fn classical_establishment_still_works_unchanged() {
    // The classical (non-hybrid) path is unaffected by the PQ additions: a normal
    // outbound/inbound handshake still round-trips.
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let mut a_session = MercurySession::establish_outbound(&alice, &bob.bundle).unwrap();
    let ct = a_session.encrypt(b"classical still works").unwrap();
    let (_b_session, recovered) =
        MercurySession::establish_inbound(&mut bob.account, alice.curve25519_key(), &ct).unwrap();
    assert_eq!(recovered, b"classical still works");
}

// ---------------------------------------------------------------------------
// Continuous post-quantum layer (HybridSession): every message is PQ-wrapped
// ---------------------------------------------------------------------------

/// Establish a `HybridSession` pair (Alice initiates, Bob accepts), returning both
/// live sessions. The bootstrap plaintext is asserted to round-trip.
fn hybrid_pair(bootstrap: &[u8]) -> (HybridSession, HybridSession, LocalSessionAccount) {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let alice_curve = alice.curve25519_key();
    let (alice_session, flight) =
        HybridSession::initiate(&alice, &bob.bundle, bootstrap).expect("alice initiate");
    let (bob_session, recovered) =
        HybridSession::accept(&mut bob.account, &bob.pq_keypair, alice_curve, &flight)
            .expect("bob accept");
    assert_eq!(recovered, bootstrap, "bootstrap plaintext must round-trip");
    (alice_session, bob_session, alice)
}

#[test]
fn hybrid_session_bootstrap_and_continuous_round_trip() {
    let (mut alice, mut bob, _a) = hybrid_pair(b"pq continuous bootstrap");
    assert!(
        alice.is_version_2() && bob.is_version_2(),
        "inner Olm must be v2"
    );

    // Every subsequent message round-trips through the continuous PQ layer, both
    // directions, across many messages.
    for i in 0..16u32 {
        let pa = format!("alice msg {i}");
        let ma = alice.encrypt(pa.as_bytes()).expect("alice encrypt");
        assert_eq!(bob.decrypt(&ma).expect("bob decrypt"), pa.as_bytes());

        let pb = format!("bob msg {i}");
        let mb = bob.encrypt(pb.as_bytes()).expect("bob encrypt");
        assert_eq!(alice.decrypt(&mb).expect("alice decrypt"), pb.as_bytes());
    }
}

#[test]
fn continuous_indices_advance_per_direction() {
    // The initiator already consumed i2r index 0 for the bootstrap, so its first
    // post-bootstrap message is index 1. The responder's reply chain (r2i) starts
    // fresh at index 0.
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot");
    let a1 = alice.encrypt(b"a-first").expect("a enc");
    assert_eq!(a1.index, 1, "initiator's first post-bootstrap index is 1");
    let a2 = alice.encrypt(b"a-second").expect("a enc");
    assert_eq!(a2.index, 2);
    let _ = bob.decrypt(&a1).unwrap();
    let _ = bob.decrypt(&a2).unwrap();

    let b1 = bob.encrypt(b"b-first").expect("b enc");
    assert_eq!(b1.index, 0, "responder's first reply index is 0");
    assert_eq!(alice.decrypt(&b1).unwrap(), b"b-first");
}

#[test]
fn each_continuous_message_actually_carries_the_outer_pq_layer() {
    // The wrapped bytes must be longer than a bare Olm message by at least the
    // outer nonce (24) + tag (16), and two encryptions of identical plaintext must
    // produce distinct outer bytes (fresh nonce + advancing chain key).
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot");
    let m1 = alice.encrypt(b"same").expect("enc");
    let m2 = alice.encrypt(b"same").expect("enc");
    assert_ne!(
        m1.outer, m2.outer,
        "fresh nonce + ratcheted key must yield distinct outer ciphertexts"
    );
    assert!(
        m1.outer.len() >= 24 + 16,
        "outer must include nonce + AEAD tag"
    );
    assert_eq!(bob.decrypt(&m1).unwrap(), b"same");
    assert_eq!(bob.decrypt(&m2).unwrap(), b"same");
}

#[test]
fn a_replayed_continuous_message_is_rejected() {
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot");
    let m = alice.encrypt(b"once").expect("enc");
    assert_eq!(bob.decrypt(&m).expect("first decrypt"), b"once");
    // Replaying the same continuous message must fail: the outer index is consumed.
    assert!(matches!(
        bob.decrypt(&m),
        Err(SessionError::HybridChainFailed)
    ));
}

#[test]
fn a_tampered_continuous_outer_layer_does_not_desync_the_chain() {
    // A forged/tampered message must be rejected AND must not advance the receive
    // chain (the two-phase commit), so the legitimate message at that index still
    // decrypts afterwards.
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot");
    let m1 = alice.encrypt(b"first").expect("enc");

    let mut tampered = m1.clone();
    let last = tampered.outer.len() - 1;
    tampered.outer[last] ^= 0x01;
    assert!(
        matches!(bob.decrypt(&tampered), Err(SessionError::HybridChainFailed)),
        "tampered outer layer must be rejected"
    );

    // No desync: the genuine message still decrypts.
    assert_eq!(
        bob.decrypt(&m1)
            .expect("genuine message after a forgery attempt"),
        b"first"
    );
}

#[test]
fn a_tampered_continuous_index_does_not_desync_the_chain() {
    // Moving a message to a different (in-window) index breaks the AAD binding, so
    // it is rejected — and, via the two-phase commit, leaves the chain untouched so
    // the genuine message still decrypts.
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot");
    let m1 = alice.encrypt(b"first").expect("enc");

    let mut moved = m1.clone();
    moved.index = m1.index + 1; // claim a later slot than it was sealed for
    assert!(
        matches!(bob.decrypt(&moved), Err(SessionError::HybridChainFailed)),
        "index-shifted message must be rejected"
    );

    assert_eq!(
        bob.decrypt(&m1)
            .expect("genuine message after an index-shift attempt"),
        b"first"
    );
}

#[test]
fn out_of_order_continuous_messages_within_bound_round_trip() {
    // The outer layer tolerates reordering via cached skipped keys: deliver three
    // messages in reverse, all decrypt to the right plaintext.
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot");
    let m1 = alice.encrypt(b"one").expect("enc");
    let m2 = alice.encrypt(b"two").expect("enc");
    let m3 = alice.encrypt(b"three").expect("enc");

    assert_eq!(bob.decrypt(&m3).expect("decrypt m3 first"), b"three");
    assert_eq!(bob.decrypt(&m1).expect("decrypt m1 from cache"), b"one");
    assert_eq!(bob.decrypt(&m2).expect("decrypt m2 from cache"), b"two");

    // Replaying a now-consumed skipped index is still rejected.
    assert!(matches!(
        bob.decrypt(&m1),
        Err(SessionError::HybridChainFailed)
    ));

    // The channel keeps working in order afterwards.
    let m4 = alice.encrypt(b"four").expect("enc");
    assert_eq!(bob.decrypt(&m4).expect("decrypt m4"), b"four");
}

#[test]
fn a_continuous_index_too_far_ahead_is_rejected() {
    // A gap larger than the skip bound is refused before any key derivation. We
    // forge an index far beyond the receive chain; the outer bytes are irrelevant
    // because the gap check fails closed first.
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot");
    // Consume the real index-1 message so the receive chain is at a known point.
    let m1 = alice.encrypt(b"real").expect("enc");
    assert_eq!(bob.decrypt(&m1).unwrap(), b"real");

    let absurd = HybridMessage {
        index: 5_000_000,
        outer: vec![0u8; 64],
    };
    assert!(matches!(
        bob.decrypt(&absurd),
        Err(SessionError::HybridChainFailed)
    ));

    // And the chain is unharmed: the next genuine in-order message still decrypts.
    let m2 = alice.encrypt(b"still-fine").expect("enc");
    assert_eq!(bob.decrypt(&m2).unwrap(), b"still-fine");
}

#[test]
fn a_genuine_message_past_the_skip_bound_is_rejected_by_the_bound_not_the_aead() {
    // Isolates the gap-bound from AEAD failure: the rejected message here is a
    // GENUINE, correctly-sealed message (its outer AEAD would authenticate at its
    // own index), yet it is refused PURELY by the skip-bound check in prepare(),
    // which fires BEFORE any key derivation or AEAD. This is the DoS guard — without
    // it, a far-ahead index would force prepare() to walk and derive a key per
    // skipped slot. We send just past the bound so the gap is exactly the first
    // value the bound rejects.
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot"); // bob.recv chain at index 1
    let over_bound = 1 + PQ_MAX_SKIP + 1; // recv.index(1) + bound + 1 == first over-bound index
    let mut msgs = Vec::with_capacity(over_bound as usize);
    for _ in 0..over_bound {
        msgs.push(alice.encrypt(b"x").expect("enc"));
    }
    let last = msgs.last().expect("sent at least one").clone();
    assert_eq!(
        last.index, over_bound,
        "last genuine message must sit exactly one past the skip bound"
    );

    // Genuine message, but its gap (over_bound - 1 == PQ_MAX_SKIP + 1) exceeds the
    // bound, so prepare() rejects it before touching its (otherwise valid) AEAD.
    assert!(matches!(
        bob.decrypt(&last),
        Err(SessionError::HybridChainFailed)
    ));

    // Two-phase commit: the rejected message left the receive chain untouched, so
    // the genuine in-order message at index 1 still decrypts.
    assert_eq!(
        bob.decrypt(&msgs[0])
            .expect("genuine in-order message after the rejection"),
        b"x"
    );
}

#[test]
fn the_wrong_pq_keypair_cannot_accept_a_hybrid_session() {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let alice_curve = alice.curve25519_key();
    let (_alice_session, flight) =
        HybridSession::initiate(&alice, &bob.bundle, b"secret").expect("initiate");

    // A different ML-KEM keypair decapsulates to an unrelated secret -> the chains
    // are seeded wrong -> the first-flight outer layer fails closed.
    let wrong_pq = MlKemKeyPair::generate();
    assert!(matches!(
        HybridSession::accept(&mut bob.account, &wrong_pq, alice_curve, &flight),
        Err(SessionError::HybridChainFailed)
    ));
}

#[test]
fn a_tampered_mlkem_ciphertext_fails_hybrid_accept() {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let alice_curve = alice.curve25519_key();
    let (_alice_session, mut flight) =
        HybridSession::initiate(&alice, &bob.bundle, b"secret").expect("initiate");
    flight.mlkem_ciphertext[0] ^= 0x01; // implicit rejection -> wrong secret -> outer fails
    assert!(matches!(
        HybridSession::accept(&mut bob.account, &bob.pq_keypair, alice_curve, &flight),
        Err(SessionError::HybridChainFailed)
    ));
}

#[test]
fn a_tampered_hybrid_first_flight_outer_is_rejected() {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let alice_curve = alice.curve25519_key();
    let (_alice_session, mut flight) =
        HybridSession::initiate(&alice, &bob.bundle, b"secret").expect("initiate");
    let last = flight.outer.len() - 1;
    flight.outer[last] ^= 0x01;
    assert!(matches!(
        HybridSession::accept(&mut bob.account, &bob.pq_keypair, alice_curve, &flight),
        Err(SessionError::HybridChainFailed)
    ));
}

#[test]
fn hybrid_initiate_rejects_an_unauthenticated_bundle() {
    // A bundle whose identity signature has been broken must be refused before any
    // encapsulation: HybridSession::initiate verifies first.
    let mut bob = new_party(5);
    bob.bundle.signature[0] ^= 0x01;
    let alice = LocalSessionAccount::new();
    assert!(matches!(
        HybridSession::initiate(&alice, &bob.bundle, b"secret"),
        Err(SessionError::BundleSignature)
    ));
}

// ---------------------------------------------------------------------------
// Post-quantum ratchet over Olm (HybridRatchetSession): break-in recovery
// ---------------------------------------------------------------------------

/// Establish a `HybridRatchetSession` pair (Alice initiates, Bob accepts), asserting the
/// bootstrap plaintext round-trips. Bob's ML-KEM keypair is moved into the responder
/// ratchet (it is the responder's initial ratchet key), so it is consumed here.
fn hybrid_ratchet_pair(bootstrap: &[u8]) -> (HybridRatchetSession, HybridRatchetSession) {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let alice_curve = alice.curve25519_key();
    let (alice_session, flight) =
        HybridRatchetSession::initiate(&alice, &bob.bundle, bootstrap).expect("alice initiate");
    let bob_pq = bob.pq_keypair; // move the responder's bundle ML-KEM key into the ratchet
    let (bob_session, recovered) =
        HybridRatchetSession::accept(&mut bob.account, bob_pq, alice_curve, &flight)
            .expect("bob accept");
    assert_eq!(recovered, bootstrap, "bootstrap plaintext must round-trip");
    (alice_session, bob_session)
}

#[test]
fn hybrid_ratchet_bootstrap_and_bidirectional_round_trip() {
    let (mut alice, mut bob) = hybrid_ratchet_pair(b"pq ratchet bootstrap");
    assert!(
        alice.is_version_2() && bob.is_version_2(),
        "inner Olm must be v2"
    );
    // Many turns; each direction change forces a fresh ML-KEM ratchet epoch under the
    // hood, while the inner Olm ratchet also advances.
    for i in 0..16u32 {
        let a = alice
            .encrypt(format!("alice {i}").as_bytes())
            .expect("a enc");
        assert_eq!(
            bob.decrypt(&a).expect("b dec"),
            format!("alice {i}").as_bytes()
        );
        let b = bob.encrypt(format!("bob {i}").as_bytes()).expect("b enc");
        assert_eq!(
            alice.decrypt(&b).expect("a dec"),
            format!("bob {i}").as_bytes()
        );
    }
}

#[test]
fn hybrid_ratchet_carries_several_messages_per_epoch() {
    let (mut alice, mut bob) = hybrid_ratchet_pair(b"boot");
    // Three messages from Alice with no reply: one PQ epoch, inner Olm sending chain.
    let m0 = alice.encrypt(b"a0").expect("enc");
    let m1 = alice.encrypt(b"a1").expect("enc");
    let m2 = alice.encrypt(b"a2").expect("enc");
    assert_eq!(bob.decrypt(&m0).expect("dec"), b"a0");
    assert_eq!(bob.decrypt(&m1).expect("dec"), b"a1");
    assert_eq!(bob.decrypt(&m2).expect("dec"), b"a2");
}

#[test]
fn hybrid_ratchet_rejects_a_tampered_message_without_desync() {
    let (mut alice, mut bob) = hybrid_ratchet_pair(b"boot");
    let good = alice.encrypt(b"first").expect("enc");
    let mut bad: RatchetMessage = good.clone();
    let last = bad.outer.len() - 1;
    bad.outer[last] ^= 0x01;
    assert!(
        matches!(bob.decrypt(&bad), Err(SessionError::RatchetFailed)),
        "tampered outer ratchet layer must be rejected"
    );
    // No desync: the genuine message still decrypts.
    assert_eq!(bob.decrypt(&good).expect("genuine after forgery"), b"first");
}

#[test]
fn hybrid_ratchet_rejects_replay() {
    let (mut alice, mut bob) = hybrid_ratchet_pair(b"boot");
    let m = alice.encrypt(b"once").expect("enc");
    assert_eq!(bob.decrypt(&m).expect("first decrypt"), b"once");
    assert!(matches!(bob.decrypt(&m), Err(SessionError::RatchetFailed)));
}

#[test]
fn the_wrong_pq_keypair_cannot_accept_a_ratchet_session() {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let alice_curve = alice.curve25519_key();
    let (_alice_session, flight) =
        HybridRatchetSession::initiate(&alice, &bob.bundle, b"secret").expect("initiate");
    // A different ML-KEM keypair decapsulates to an unrelated secret -> outer ratchet
    // layer fails closed, so the inner Olm handshake is never run.
    let wrong_pq = MlKemKeyPair::generate();
    assert!(matches!(
        HybridRatchetSession::accept(&mut bob.account, wrong_pq, alice_curve, &flight),
        Err(SessionError::RatchetFailed)
    ));
}

#[test]
fn a_hybrid_ratchet_session_survives_an_encrypted_pickle() {
    // Establish, exchange a few messages (advancing the ratchet), then PICKLE both sides,
    // restore them from the encrypted blobs, and confirm the conversation CONTINUES — the
    // post-quantum ratchet state (root, chains, ML-KEM keypair, skipped keys) survived.
    let (mut alice, mut bob) = hybrid_ratchet_pair(b"boot");
    let m = alice.encrypt(b"before pickle").unwrap();
    assert_eq!(bob.decrypt(&m).unwrap(), b"before pickle");
    let r = bob.encrypt(b"a reply").unwrap();
    assert_eq!(alice.decrypt(&r).unwrap(), b"a reply");

    let key = [0x5au8; 32];
    let a_blob = alice.to_encrypted_pickle(&key);
    let b_blob = bob.to_encrypted_pickle(&key);
    drop(alice);
    drop(bob);

    let mut alice2 = HybridRatchetSession::from_encrypted_pickle(&a_blob, &key).expect("restore a");
    let mut bob2 = HybridRatchetSession::from_encrypted_pickle(&b_blob, &key).expect("restore b");

    // The restored sessions keep talking, both directions.
    for i in 0..4u32 {
        let pa = format!("after restore alice {i}");
        let m = alice2.encrypt(pa.as_bytes()).unwrap();
        assert_eq!(bob2.decrypt(&m).unwrap(), pa.as_bytes());
        let pb = format!("after restore bob {i}");
        let r = bob2.encrypt(pb.as_bytes()).unwrap();
        assert_eq!(alice2.decrypt(&r).unwrap(), pb.as_bytes());
    }

    // Fail closed: a wrong key and a garbage blob are rejected, never a panic.
    assert!(matches!(
        HybridRatchetSession::from_encrypted_pickle(&a_blob, &[9u8; 32]),
        Err(SessionError::Pickle)
    ));
    assert!(HybridRatchetSession::from_encrypted_pickle(b"garbage", &key).is_err());
    for len in 0..300usize {
        let _ = HybridRatchetSession::from_encrypted_pickle(&vec![0xa5u8; len], &key);
    }
}

#[test]
fn a_pickled_session_preserves_its_skipped_keys_across_a_restart() {
    // Pickle a session that is MID out-of-order delivery (it holds cached skipped keys),
    // restore it, and confirm the earlier (out-of-order) messages still decrypt — proving
    // the skipped-key cache + its FIFO sequence survive the pickle.
    let (mut alice, mut bob) = hybrid_ratchet_pair(b"boot");
    let m0 = alice.encrypt(b"zero").unwrap();
    let m1 = alice.encrypt(b"one").unwrap();
    let m2 = alice.encrypt(b"two").unwrap();
    // Bob receives only the LAST first, caching skipped keys for m0 and m1.
    assert_eq!(bob.decrypt(&m2).unwrap(), b"two");

    let key = [0x33u8; 32];
    let blob = bob.to_encrypted_pickle(&key);
    drop(bob);
    let mut bob2 = HybridRatchetSession::from_encrypted_pickle(&blob, &key).expect("restore");

    // The restored session still holds the skipped keys: the earlier messages decrypt.
    assert_eq!(bob2.decrypt(&m0).unwrap(), b"zero");
    assert_eq!(bob2.decrypt(&m1).unwrap(), b"one");
    // And a replay of an already-consumed skipped index is still rejected after restore.
    assert!(matches!(
        bob2.decrypt(&m0),
        Err(SessionError::RatchetFailed)
    ));
}

#[test]
fn hybrid_ratchet_initiate_rejects_an_unauthenticated_bundle() {
    let mut bob = new_party(5);
    bob.bundle.signature[0] ^= 0x01;
    let alice = LocalSessionAccount::new();
    assert!(matches!(
        HybridRatchetSession::initiate(&alice, &bob.bundle, b"secret"),
        Err(SessionError::BundleSignature)
    ));
}

// ---------------------------------------------------------------------------
// Canonical versioned wire codecs for the PQ message types
// ---------------------------------------------------------------------------

#[test]
fn hybrid_first_flight_survives_a_wire_round_trip() {
    let mut bob = new_party(5);
    let alice = LocalSessionAccount::new();
    let alice_curve = alice.curve25519_key();
    let (_a, flight) = HybridSession::initiate(&alice, &bob.bundle, b"boot").expect("initiate");

    let wire = flight.to_bytes();
    let parsed = HybridFirstFlight::from_bytes(&wire).expect("round-trip parse");
    assert_eq!(parsed, flight, "wire round-trip must preserve the value");

    // The parsed-from-wire flight still establishes the session and recovers the plaintext.
    let (_b, recovered) =
        HybridSession::accept(&mut bob.account, &bob.pq_keypair, alice_curve, &parsed)
            .expect("accept parsed flight");
    assert_eq!(recovered, b"boot");

    // Fail closed: empty, wrong version, and a truncated frame are all rejected.
    assert!(matches!(
        HybridFirstFlight::from_bytes(&[]),
        Err(SessionError::FirstFlightFailed)
    ));
    let mut wrong_ver = wire.clone();
    wrong_ver[0] ^= 0xff;
    assert!(matches!(
        HybridFirstFlight::from_bytes(&wrong_ver),
        Err(SessionError::FirstFlightFailed)
    ));
    // A frame too short to hold the version + full ML-KEM ciphertext + a minimal outer
    // layer is rejected at parse. (A 1-byte truncation of the variable-length `outer` is
    // NOT a structural error — it is caught by the AEAD at decrypt, which is correct.)
    assert!(matches!(
        HybridFirstFlight::from_bytes(&wire[..100]),
        Err(SessionError::FirstFlightFailed)
    ));
}

#[test]
fn hybrid_message_survives_a_wire_round_trip() {
    let (mut alice, mut bob, _a) = hybrid_pair(b"boot");
    let m = alice.encrypt(b"over the wire").expect("enc");
    let wire = m.to_bytes();
    let parsed = HybridMessage::from_bytes(&wire).expect("round-trip parse");
    assert_eq!(parsed, m);
    assert_eq!(
        bob.decrypt(&parsed).expect("decrypt parsed"),
        b"over the wire"
    );

    assert!(matches!(
        HybridMessage::from_bytes(&[1u8, 2, 3]),
        Err(SessionError::HybridChainFailed)
    ));
}

#[test]
fn ratchet_message_survives_a_wire_round_trip() {
    let (mut alice, mut bob) = hybrid_ratchet_pair(b"boot");
    let m = alice.encrypt(b"over the wire").expect("enc");
    let wire = m.to_bytes();
    let parsed = RatchetMessage::from_bytes(&wire).expect("round-trip parse");
    assert_eq!(parsed, m);
    assert_eq!(
        bob.decrypt(&parsed).expect("decrypt parsed"),
        b"over the wire"
    );

    assert!(matches!(
        RatchetMessage::from_bytes(&[1u8; 16]),
        Err(SessionError::RatchetFailed)
    ));
}
