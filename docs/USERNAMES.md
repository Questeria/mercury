# Usernames & pairing codes — design and honest limits

Two friendlier ways to connect than copy-pasting a 64-hex account id landed in `0.1.21`:

- **Pairing codes** — a short, single-use code the relay brokers for a few minutes.
- **Usernames** — a human handle backed by the relay's key-transparency (KT) log.

Both are *convenience surfaces*. **Neither changes Mercury's trust model.** A contact card is always
re-verified on add (it is self-authenticating — its account id is the hash of its identity key), and
the **safety number** remains the only thing that proves you are talking to the real person. This
document states exactly what each method does and, just as importantly, what it does **not** yet do.

---

## Pairing codes (`#72`)

**Flow.** Your client asks the relay to broker your contact card under a random code
(`POST /pair/publish`, authorized by an Ed25519 *pairing proof of possession* over the card so the
broker can't be stuffed with unsigned bytes). You read the code to someone; they enter it
(`GET /pair/fetch/{code}`), fetch the card **once**, and re-verify it locally before adding you.

**Properties.**
- **Single-use**: the code is deleted on the first successful fetch (deliver-once).
- **Short-lived**: it expires after `PAIRING_TTL_S` (10 minutes).
- **Unguessable**: 10 symbols over a 32-symbol alphabet ≈ 50 bits; the fetch endpoint is
  flood-limited per source, so blind guessing within the TTL window is not feasible.
- **Opaque to the relay**: the relay never parses the card; it stores bytes under a random code.

**Honest limits.**
- A pairing code is *bearer* authority for that short window: anyone who sees the code before the
  intended recipient can fetch your card and connect as if they were you. Treat it like a one-time
  password — share it over a channel only the right person sees, and **verify the safety number**
  afterward.
- The relay can withhold or substitute a *different valid* card (an availability/confusion issue, not
  a forgery — the fetcher sees exactly which account id they are about to add). The safety number
  closes this.

---

## Usernames (`#73`) — key-transparency-backed

**Flow.** You **claim** a handle (`POST /username/claim`, authorized by a *username-claim proof of
possession* over the normalized handle, so the relay binds it only to *your* account id),
first-claimant-wins. The claim is committed to the relay's **AKD key-transparency log**. To add
someone, your client **looks up** the handle (`GET /username/{name}`), which returns the account id
**and a cryptographic inclusion proof**. Your client verifies that proof against the directory's VRF
key and only then fetches + verifies the (independently self-authenticating) contact card.

**What the inclusion proof buys you.** Once you have pinned the directory's VRF key, the relay
**cannot forge or alter** a `handle → account_id` binding without the proof failing your local
`verify_inclusion` check. The handle resolves to the account the real owner claimed, or the lookup
fails closed — it never silently resolves to an impostor.

**Normalization / anti-confusable.** Handles are normalized to lowercase ASCII `[a-z0-9_]`, 3–20
chars, first char a letter (`mercury_keys::normalize_username`). This removes case-only collisions
and the Unicode-homoglyph surface (e.g. a Cyrillic `а` is rejected, not confused with `a`). Both the
claimer (when signing) and any looker-up normalize through the same function, so they agree on the
exact label committed to the log.

### Residuals (NOT yet closed — disclosed, not hidden)

1. **VRF-key bootstrap is trust-on-first-use (TOFU) — only for CUSTOM relays now.** For the built-in
   **default relay this is CLOSED**: the directory's VRF key *and* log key are baked into the signed
   client build (`ui/app/src-tauri/src/main.rs`, `pin_kt_vrf_if_absent` / `pin_kt_log_if_absent`), so
   a default-relay user never trusts a relay-served key on first use — every proof is checked against
   the build-pinned key, and a relay that serves a different key fails closed immediately. For a
   **custom** relay, the client still pins the key it serves the first time it resolves a username
   (`GET /kt/vrf-key`) and verifies every later proof against that pinned copy — so a *later* key
   rotation is detected, but a relay malicious *from your very first contact* could serve its own key.
   **Closing it for custom relays:** distribute their VRF key out-of-band (the same machinery that
   already powers the default relay's baked pin); only out-of-band distribution for third parties
   remains.
2. **No independent witnesses / gossip yet.** Even with a pinned VRF key, a single log can *equivocate*
   — show different append-only histories to different clients (a split view). For the default relay,
   a username resolution now also verifies the relay's **signed tree head** against the build-pinned
   **log key** and binds the inclusion proof to that exact signed `(epoch, root)` (`resolve_username`),
   so a forged lookup root or a head the genuine log never signed is rejected — but a malicious
   log-key *holder* could still sign **different** heads for different clients. Full KT closes that
   with independent **witnesses** co-signing tree heads and **gossip** between clients. The relay
   already serves consistency proofs (`/kt/consistency`) and signed tree heads (`/kt/sth`); wiring a
   witness/gossip loop is the remaining work.
3. **Availability.** The relay can refuse to serve a lookup or claim (denial of service). It cannot
   forge a binding (that's what the proof prevents), only withhold one.
4. **Durability: the registry IS durable; the KT log is rebuilt at boot.** The username registry
   now persists to a durable redb volume when `MERCURY_USERNAME_DB` is set (production sets it —
   see `deploy/fly.toml`): **claims survive restarts**, first-claimant-wins holds across reboots,
   and a fresh claim is committed to disk *before* it is accepted (fail-closed — a failed disk
   write is a retryable 503, never an in-memory-only "success" a restart would revoke). What
   *remains*: the AKD transparency log itself is still in-memory — at boot the relay re-registers
   every persisted binding into a fresh log in one epoch. Because the VRF key is stable (derived
   from a persisted secret seed), per-lookup inclusion proofs still verify against the key clients
   pinned; but **epoch history resets**, so consistency proofs spanning a restart are not
   meaningful. **Closing the rest:** a durable AKD storage backend that preserves epoch history.
   Pairing codes are intentionally ephemeral, so losing them on restart is by design.
5. **The safety number is still the backstop.** A username, like every other method, gets you a card;
   comparing the safety number over a trusted channel is what proves no MITM. The UI says so.

### Why this is shipped now, honestly

The cryptographic core is **real and tested**: first-claimant-wins is enforced; claims are committed
to a genuine AKD log; lookups carry real inclusion proofs; the client verifies them and **fails
closed** on a forged binding, a tampered checkpoint, a wrong/rotated key, or a missing directory key
(see `core/rust/mercury-relay/tests/username_kt.rs` and `core/rust/mercury-client/tests/discovery.rs`).
What remains — out-of-band VRF pinning and witness/gossip — is operational + a known multi-step
follow-on, documented here rather than papered over. Until then, **verify the safety number**.
