# Mercury — Metadata & Privacy Posture

This is the honest account of what Mercury's **content encryption does and does not hide** at the
metadata layer — what a curious or malicious **relay**, and a passive/active **network observer**,
can still learn *despite* end-to-end encryption. It exists so the product never overclaims: Mercury is
a content-confidential, sender-sealed messenger with **real, documented metadata exposure**, not a
metadata-private (mixnet-grade) one.

If you only read one line: **the relay never learns *what* you said or (from the message) *who sent
it*, but it does learn *who receives*, *how much*, *when*, and *who looks up whom*.**

## What IS protected

- **Message content** — end-to-end encrypted with a hybrid post-quantum Double Ratchet (ML-KEM-768 +
  classical), key-committing AEAD. The relay moves opaque bytes it cannot read.
- **Sender identity (sealed-sender)** — the sender's account id lives *inside* the sealed outer
  envelope (`MercuryClient::send`/`initiate` seal the inner frame to the recipient's device key). A
  `/relay/submit` carries **no** sender-identifying field, so the relay never learns *who sent* a
  message from its payload. A fresh ephemeral key per message means the envelope is not linkable by a
  static sender key either.
- **No party named in the clear** — the cleartext transport/envelope header carries only
  version/suite/length/epoch/sequence/kind/flag scalars; it does **not** contain a `conversation_id`,
  `sender_account_id`, or `recipient` field, and `plaintext_identity_fields` is gate-enforced to 0.

## What LEAKS, to whom, and why

| # | Leak | To whom | Nature |
|---|------|---------|--------|
| 1 | **Recipient identity** — the route is the recipient's stable account id, reused for every message and shared with the directory/KT slots (a cross-linkable global id) | relay; on-path observer if TLS is stripped | **inherent** to a mailbox that must address a recipient |
| 2 | **Social graph** — the relay can correlate `(source IP, recipient route, time)` to reconstruct who-talks-to-whom, and count/time per-recipient traffic | relay (IP↔route); observer (recipient route) | **mostly inherent** without sender anonymization |
| 3 | **Message size class** — plaintext is padded to 256-byte buckets, so exact sizes are hidden *within* a bucket, but the relay sees the bucket; a typing-indicator/receipt (bucket 1) is distinguishable from a photo (many buckets) | relay; observer (record length) | **partly fixable** (collapse the small-message classes) |
| 4 | **Send→deliver timing** — a submit immediately wakes the recipient's long-poll, so in→out timing on a route is directly correlatable; no mixing/cover traffic exists | relay (both events); observer (both legs) | **inherent** without a mixnet / batched delivery |
| 5 | **Contact-discovery intent** — `GET /directory/fetch/{account_id}` and `GET /username/{name}` carry the queried id / plaintext handle in the URL, revealing who is looking up whom *before* any message | relay; observer | **partly inherent** (full fix = private contact discovery / PIR) |

Sealed-sender is **application-layer**: it hides the sender from the message payload, but the
submitter's **source IP** is still visible to the relay. Network-layer sender anonymity needs Tor or a
proxy and is out of scope of the protocol.

## Fixable in code (worth doing) vs inherent (must be documented, not "fixed")

**Fixable — concrete improvements, in priority order:**

1. **Blinded / rotating recipient routes** (leaks 1, 2) — the single highest-leverage change: derive
   per-epoch or per-sender pseudonymous mailbox tags the recipient can still poll, so the route stops
   being a permanent global recipient identifier. Reduces (cannot erase) recipient addressing.
2. **Collapse small-message size classes** (leak 3) — pad all control + short-text messages to one
   common floor so typing-indicators, receipts, and short texts are length-indistinguishable. Cheap;
   removes the "is this small-talk or media" signal for the common case.
3. **Bucketed / oblivious handle lookups** (leak 5) — hashed + k-anonymity-bucketed handle resolution
   so the exact handle isn't sent in the clear. Partial.
4. **Policy: never log or persist `source-IP ↔ route` tuples** (leak 2) — the relay does not today;
   keep it that way and state it as a non-goal.

**Inherent — needs a different design (mixnet / PIR / Tor), documented as a known limitation:**

- **Recipient addressing** (leak 1) — a store-and-forward relay must know where to deliver; blinding
  reduces linkability but something recipient-addressable is always present.
- **Timing & volume correlation** (leak 4) — defeating submit→deliver linkage and per-route volume
  needs a mixnet, batched/randomized-delay delivery, and cover traffic. None exist today; bolting them
  on is a major design change, not a patch. **This is the weakest axis** and should be stated plainly.
- **Network-layer sender de-anonymization** via source IP — needs Tor/a proxy in front of submit.
- **Pre-conversation discovery** (leak 5) — a full fix is private contact discovery / PIR, a
  research-grade addition. KT/VRF gives *integrity* of the username→account binding (you cannot be
  lied to about who `alice` is), but **not privacy** of the lookup.

## Honest bottom line

Mercury protects **content** and **sender-from-payload** strongly and honestly — the relay is genuinely
content-blind and sender-blind at the message layer. The metadata story is the opposite of hidden: a
curious/malicious relay with network visibility learns who receives, how much, and when, and who is
looking up whom. The most valuable concrete code change is **blinding the recipient route**; the
**timing/volume axis is inherent** to the store-and-forward design and is the honest limitation to
state up front rather than paper over. Do not market Mercury as metadata-private until at least the
recipient-route blinding and small-message padding land, and the timing axis is addressed by a mixing
layer.
