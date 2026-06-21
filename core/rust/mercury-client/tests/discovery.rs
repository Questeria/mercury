//! End-to-end discovery over the REAL relay: the pairing-code broker and the key-transparency
//! username registry, each driving a full two-client encrypted exchange. The relay (main router +
//! KT router) runs on an ephemeral port; every card and binding is re-verified client-side.

use std::sync::Arc;
use std::time::Duration;

use mercury_client::{
    ClientError, MercuryClient, PollAuth, RelayTransport, Transport, TransportError,
    UsernameClaimOutcome, UsernameLookup,
};
use mercury_kt::KtDirectory;
use mercury_relay::{
    InMemoryQueueStore, KtState, NoopPushSender, QueueStore, RelayState, kt_router, router,
};

/// Start the real relay (main + KT routers) on an ephemeral loopback port; returns its base URL.
fn start_relay_full() -> String {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async move {
            let store: Arc<tokio::sync::Mutex<dyn QueueStore>> =
                Arc::new(tokio::sync::Mutex::new(InMemoryQueueStore::new()));
            let kt = KtState::new(KtDirectory::new().await.expect("kt directory"));
            let app = router(RelayState::new(store, Arc::new(NoopPushSender))).merge(kt_router(kt));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind ephemeral port");
            let port = listener.local_addr().expect("local addr").port();
            tx.send(port).expect("send port");
            axum::serve(listener, app).await.expect("relay serve");
        });
    });
    let port = rx.recv().expect("relay port");
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
fn pairing_code_brokers_a_card_for_a_full_conversation() {
    let base = start_relay_full();
    let transport = RelayTransport::new(&base);

    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let a_id = alice.account_id();
    let b_id = bob.account_id();

    // Bob mints a pairing code (brokering his card); Alice enters it and fetches+verifies the card.
    let code = bob.publish_pairing(&transport).expect("mint pairing code");
    assert!(code.contains('-'), "code is human-grouped: {code}");
    let card = alice
        .fetch_by_pairing(&transport, &code)
        .expect("fetch + verify by code");
    assert_eq!(card.account_id(), b_id, "the code resolved to Bob's card");

    // A full session bootstraps from the pairing-fetched card.
    let (route, flight) = alice.initiate(&card, b"hi via pairing code").unwrap();
    transport.submit(&route, &flight).expect("submit flight");
    let blob = poll_until(&transport, &bob);
    let (from, msg) = bob.receive(&blob).unwrap();
    assert_eq!((from, msg), (a_id, b"hi via pairing code".to_vec()));

    // Single-use: the same code cannot be fetched again.
    assert_eq!(
        alice.fetch_by_pairing(&transport, &code).unwrap_err(),
        ClientError::ContactNotFound
    );
}

#[test]
fn username_registry_resolves_a_verified_card() {
    let base = start_relay_full();
    let transport = RelayTransport::new(&base);

    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let b_id = bob.account_id();

    // Bob must have a fetchable card (username resolves to an account id, then fetches its card).
    bob.publish_to_directory(&transport).expect("publish card");
    // Bob claims "bob".
    assert_eq!(
        bob.claim_username(&transport, "Bob").unwrap(),
        UsernameClaimOutcome::Created
    );
    // Re-claiming by the same account is idempotent.
    assert_eq!(
        bob.claim_username(&transport, "bob").unwrap(),
        UsernameClaimOutcome::AlreadyOwned
    );
    // A different account cannot take it.
    assert_eq!(
        alice.claim_username(&transport, "bob").unwrap(),
        UsernameClaimOutcome::Taken
    );

    // Alice resolves "bob" (trust-on-first-use: no key pinned yet). The inclusion proof verifies
    // against the served VRF key, and the resolved card self-authenticates to Bob's account id.
    let resolved = alice
        .resolve_username(&transport, "bob", None, None)
        .expect("resolve + verify username");
    assert_eq!(resolved.account_id, b_id);
    assert_eq!(resolved.card.account_id(), b_id);
    assert!(resolved.tofu, "first resolve trusted the served key");
    assert!(!resolved.vrf_used.is_empty());

    // Pinning the (correct) VRF key, a second resolve verifies WITHOUT trust-on-first-use.
    let pinned = resolved.vrf_used.clone();
    let again = alice
        .resolve_username(&transport, "bob", Some(&pinned), None)
        .expect("resolve against the pinned key");
    assert!(!again.tofu);

    // A WRONG pinned key makes the proof fail closed — the binding is rejected, never trusted.
    let wrong = vec![0xabu8; pinned.len()];
    assert_eq!(
        alice
            .resolve_username(&transport, "bob", Some(&wrong), None)
            .unwrap_err(),
        ClientError::UsernameProofInvalid
    );

    // An unregistered handle is a clean not-found.
    assert_eq!(
        alice
            .resolve_username(&transport, "ghost", None, None)
            .unwrap_err(),
        ClientError::ContactNotFound
    );

    // SIGNED-HEAD binding: with the GENUINE log key pinned, the resolve verifies the relay's
    // signed tree head and succeeds; with a WRONG pinned log key it fails closed.
    let keys = transport
        .kt_public_keys()
        .expect("fetch kt keys")
        .expect("kt keys served");
    let resolved = alice
        .resolve_username(&transport, "bob", Some(&pinned), Some(&keys.log_public_key))
        .expect("resolve with signed-head binding");
    assert_eq!(resolved.account_id, b_id);
    let wrong_log = [0x42u8; 32];
    assert_eq!(
        alice
            .resolve_username(&transport, "bob", Some(&pinned), Some(&wrong_log))
            .unwrap_err(),
        ClientError::UsernameProofInvalid
    );
}

/// Delegates everything to the real relay transport EXCEPT it replays a captured, OLDER signed tree
/// head + the matching username lookup — i.e. a malicious-after-first-contact relay rolling the
/// append-only directory backwards. Both are genuinely log-signed and the inclusion proof is valid;
/// only the client's monotonicity check can catch it.
struct RollbackTransport {
    inner: RelayTransport,
    old_head: (u64, [u8; 32], i64, [u8; 64]),
    old_lookup: UsernameLookup,
}

impl Transport for RollbackTransport {
    fn submit(&self, route: &[u8; 32], blob: &[u8]) -> Result<(), TransportError> {
        self.inner.submit(route, blob)
    }
    fn poll(&self, route: &[u8; 32], auth: &PollAuth) -> Result<Option<Vec<u8>>, TransportError> {
        self.inner.poll(route, auth)
    }
    fn publish_card(
        &self,
        identity_pub: &[u8; 32],
        card: &[u8],
        pop_sig: &[u8; 64],
    ) -> Result<(), TransportError> {
        self.inner.publish_card(identity_pub, card, pop_sig)
    }
    fn fetch_card(&self, account_id: &[u8; 32]) -> Result<Option<Vec<u8>>, TransportError> {
        self.inner.fetch_card(account_id)
    }
    fn kt_signed_tree_head(
        &self,
    ) -> Result<Option<(u64, [u8; 32], i64, [u8; 64])>, TransportError> {
        Ok(Some(self.old_head)) // the rollback: serve the OLD head
    }
    fn lookup_username(&self, _name: &str) -> Result<Option<UsernameLookup>, TransportError> {
        Ok(Some(self.old_lookup.clone()))
    }
}

#[test]
fn resolve_username_rejects_a_directory_rollback() {
    let base = start_relay_full();
    let transport = RelayTransport::new(&base);

    let mut alice = MercuryClient::new();
    let mut bob = MercuryClient::new();
    let b_id = bob.account_id();

    bob.publish_to_directory(&transport).expect("publish card");
    bob.claim_username(&transport, "bob").expect("claim bob"); // -> directory epoch advances

    // Capture the directory's head + bob's lookup AT THIS earlier epoch — exactly what a rolling-back
    // relay would replay later. Both are genuinely log-signed + validly included.
    let old_head = transport
        .kt_signed_tree_head()
        .expect("sth")
        .expect("a head is served");
    let old_lookup = transport
        .lookup_username("bob")
        .expect("lookup")
        .expect("bob is registered");

    // The directory moves FORWARD (a second account claims a handle), so the head is now newer.
    let mut carol = MercuryClient::new();
    carol.publish_to_directory(&transport).expect("publish carol");
    carol.claim_username(&transport, "carol").expect("claim carol");

    let keys = transport.kt_public_keys().expect("keys").expect("served");
    let pinned_vrf = keys.vrf_public_key.clone();
    let pinned_log = keys.log_public_key;

    // Alice resolves bob against the CURRENT (newer) head — this sets her anti-rollback baseline.
    let resolved = alice
        .resolve_username(&transport, "bob", Some(&pinned_vrf), Some(&pinned_log))
        .expect("resolve at the current head");
    assert_eq!(resolved.account_id, b_id);

    // Now the relay replays the OLDER head + matching lookup. The signature verifies and the proof is
    // valid — ONLY the monotonicity check rejects it.
    let rolled_back = RollbackTransport {
        inner: RelayTransport::new(&base),
        old_head,
        old_lookup,
    };
    assert_eq!(
        alice
            .resolve_username(&rolled_back, "bob", Some(&pinned_vrf), Some(&pinned_log))
            .unwrap_err(),
        ClientError::DirectoryRollback,
        "a genuinely-signed OLDER head is rejected as a rollback"
    );

    // The baseline does not break legitimate forward resolution: the current head still resolves.
    alice
        .resolve_username(&transport, "bob", Some(&pinned_vrf), Some(&pinned_log))
        .expect("the current head still resolves after the rejected rollback");
}
