# Relay Submission Policy

Generated: 2026-05-28

## Status

Mercury now has a Helix-backed relay submission policy mirrored across:

- `policy/relay_submit_policy_v1.json`
- `helix/policy/relay_submit.hx`
- `helix/tests/relay_submit_test.hx`
- `vectors/relay_submit/*.json`
- `tools/check_relay_submit_vectors.py`
- `core/rust/mercury-policy`
- `core/rust/mercury-core`

This policy defines what an encrypted queue item must look like before a Mercury relay may accept it. The relay boundary does not decrypt messages, inspect plaintext, or decide whether a user or AI may send. It consumes the final client outbound-send decision and rejects anything the client core refused.

## Relay-Visible Inputs

The policy allows only minimal relay-facing facts:

- outbound send gate reason
- opaque route id length
- opaque replay token length
- queue TTL and client-configured TTL cap
- ciphertext length and client-configured ciphertext cap
- sealed header length
- plaintext identity field count
- padding bucket

Plaintext account ids, device ids, conversation ids, room ids, room epochs, and AI principal ids are represented by `plaintext_identity_fields`. The only accepted value is zero.

## Security Rules

The evaluator rejects:

- unknown policy versions
- unknown outbound send gate reason codes
- any outbound send gate result other than accepted
- route ids outside 16 to 128 bytes
- replay tokens not exactly 32 bytes
- nonpositive queue TTLs
- queue TTLs above the client cap or above 604800 seconds
- missing ciphertext
- ciphertext above the client cap or above 4194304 bytes
- missing sealed relay headers
- sealed relay headers above 4096 bytes
- any plaintext identity fields
- unknown padding buckets

Rejected submissions should not be queued by the server. The server may log only the audit class and opaque operational counters, never plaintext identifiers.

## Verification

The relay submission policy is checked by:

```powershell
python .\tools\check_relay_submit_vectors.py
python .\tools\check_policy_contract.py
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
cargo test --workspace
```

The Rust client-core wrapper is:

```text
RelaySubmissionInput
evaluate_relay_submission(RelaySubmissionInput) -> RelaySubmissionDecision
```

The Helix implementation is staged deliberately: metadata, lifetime, ciphertext, and send-gate validators each stay within the current compiler codegen limit of six integer parameters, then `mercury_relay_decide_v1(...)` composes their reason codes.

## Next Step

The relay queue contract is documented in `docs/25_RELAY_QUEUE_CONTRACT.md`.
