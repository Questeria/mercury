# Security Research Cycle 15: Sender-Side KeyPackage Consumption

Generated: 2026-05-28

## Primary Source Refresh

- RFC 9420, KeyPackage reuse: <https://www.rfc-editor.org/rfc/rfc9420.html#section-16.8>
- RFC 9750, KeyPackage and Welcome deployment guidance: <https://datatracker.ietf.org/doc/html/rfc9750#section-5.1>
- MLS PQ ciphersuite draft: <https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/>
- NIST FIPS 203, ML-KEM: <https://csrc.nist.gov/pubs/fips/203/final>

The research correction for this pass is that KeyPackage replay protection needs both sides:

- sender/add-member side: consume the KeyPackage once before Welcome sending
- receiver/join side: reject replayed Welcome and consumed KeyPackage state before group initialization

The receiving-side Welcome replay store already covered the second side. This increment adds the first side.

## Implementation Increment

Added an MLS KeyPackage consume-store boundary that:

- accepts only after MLS KeyPackage admission accepts
- requires add-member and Welcome-send capability from the admission decision
- rejects malformed group ids, KeyPackage hashes, added-member refs, and Welcome-send transaction digests
- rejects invalid consumption timestamps
- rejects plaintext metadata fields
- persists digest-only accepted records
- rejects duplicate KeyPackage hashes globally, even across different groups
- exposes checked fixtures and backend commands for accepted, admission-rejected, duplicate, bad-shape, and plaintext-blocked states

## Security Effect

This closes the sender-side one-time KeyPackage gap before production MLS provider integration. Future provider code now has a checked adapter boundary that can be implemented as an atomic check-and-put tied to a durable Welcome-send transaction. That prevents two dangerous cases:

- using the same KeyPackage to add the same or different groups
- treating Welcome sending as allowed after admission but before durable one-time consumption persistence

## PQ Note

The current MLS PQ ciphersuite work remains draft-stage. Mercury should keep treating `hybrid_pq_mls_768` and `hybrid_pq_mls_1024` as policy classes until provider selection maps them to audited MLS ciphersuite identifiers, KAT evidence, downgrade evidence, and provider build identity.

## Verification

Focused checks passed during development:

```powershell
cargo test -p mercury-core --test mls_key_package_consume_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test backend_commands
```

Full repo preflight remains the final merge gate:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Research Target

The follow-on Welcome-send outbox boundary now closes the crash window between KeyPackage consumption and queued Welcome delivery. Next, move from prototype MLS store boundaries to production atomicity: provider API behavior for KeyPackage fetch, delivery-service race handling, one-transaction Commit/consume/outbox persistence, and cross-device KeyPackage depletion/refresh policy.
