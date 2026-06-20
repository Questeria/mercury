# KT witnessing — split-view detection, and its honest limits

Mercury's username key-transparency (KT) log is served by an **untrusted relay**. A client already
verifies every proof against keys it pinned, so the relay cannot forge a binding. The one thing
proofs alone cannot catch is a relay that shows **different histories to different people** — a
*split view* (equivocation): internally-consistent, but forked.

**Witnessing** closes that gap the standard CT / Sigsum way: independent **witnesses** co-sign the
relay's published tree heads, and a client that requires a **quorum** of witness cosignatures on the
head its proofs are bound to will reject a forked head no witness vouched for.

This document states exactly what the relay now does — and, just as importantly, what witnessing
does **not** give you until witnesses are actually deployed.

---

## What the relay provides (the rails)

Two endpoints, plus an allow-list:

- **`GET /kt/sth/witnessed`** → the relay's current **published head** (`tree_size`, `root_hash`,
  `timestamp_s`), its **log signature**, the **witness cosignatures** collected for *exactly* those
  bytes, and the pinned witness set (informational). The published head is held **stable per epoch**
  (a fixed timestamp, refreshed at most hourly) so that independent witnesses all co-sign the *same*
  canonical bytes a client will later fetch.
- **`POST /kt/witness/cosign`** → a witness submits its cosignature over the current published head.
  The relay **fails closed**: it stores the cosignature only if it names a configured witness index,
  matches the published head exactly (`tree_size` + `root_hash` + `timestamp_s`), and is that
  witness's genuine Ed25519 signature (`verify_strict`) over the head's canonical bytes. A
  mismatched/stale head is rejected `409` (re-fetch + re-cosign); a non-witness signature `401`; an
  unknown index `400`. The endpoint is rate-limited per source.

Serving these confers **no trust**: the relay cannot forge a cosignature for a witness key it does
not hold, and a client binds the witnessed head to its own proofs — a head that does not match is
rejected by `mercury_kt::verify_witnessed_tree_head`.

## Relay configuration

```
MERCURY_KT_WITNESSES = "0:<64-hex-ed25519-pubkey>,1:<64-hex-ed25519-pubkey>,..."
```

Each entry is `operatorId:hexPublicKey`. **Order matters**: the index in this list is the canonical
`witness_index` clients must also pin out-of-band. A malformed entry refuses startup (fail-closed —
a typo cannot silently drop a witness from the quorum a client expects). Unset = witnessing disabled:
`/kt/sth/witnessed` still serves the head + log signature, just with no cosignatures, and submissions
are refused.

## The witness protocol (what a deployed witness does)

1. `GET /kt/sth/witnessed`, pin/verify the log signature against the log key, and check **append-only
   consistency** from the last head it co-signed (a witness must never co-sign a head that is not a
   consistent extension of what it already vouched for — that is the whole point).
2. Co-sign the head's canonical `signing_bytes()` with its Ed25519 key.
3. `POST /kt/witness/cosign` with `{tree_size, root_hash, timestamp_s, witness_index, signature}`.
4. Re-cosign when the head changes (a new claim) or refreshes (hourly), i.e. on a `409`.

## What this does NOT give you yet (honest residuals)

- **Witnesses must actually be deployed and independent.** Code alone adds **zero** equivocation
  resistance. The protection is exactly as strong as the set of *independent operators* running
  witnesses against this relay. `kt_witness_status` requires **≥ 2 independent operators** for
  `QuorumSatisfied`, so run witnesses on genuinely separate infrastructure/parties — N keys held by
  one operator is not a quorum.
- **The client must require a quorum.** A client that fetches the witnessed bundle but does not gate
  on `kt_witness_status` gains nothing. That client policy is not changed by default (so a
  witness-less deployment still works); enabling it is a deliberate client-side step.
- **The auditor signature is not served here.** `kt_witness_status` also wants a designated
  **auditor** signature over the same head (an append-only auditor). The relay serves witness
  cosignatures only; obtaining the auditor signature is a separate role/endpoint not yet built. Until
  it exists, `kt_witness_status` returns `Invalid` (the gate is intentionally strict), so witnessing
  today is **detection rails + verification machinery**, not yet a turnkey end-to-end quorum gate.
- **Cosignatures are in-memory.** They are ephemeral by design — witnesses re-cosign after any relay
  restart or head refresh. Nothing is persisted.

In short: this commit makes the relay **capable** of carrying witness cosignatures correctly and
verifiably. Turning that capability into real split-view resistance is an **operational** step
(deploy independent witnesses + an auditor, and enable the client quorum policy), not something the
relay can manufacture by itself. Stated here rather than implied.
