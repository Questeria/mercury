# Security Research Cycle 14: MLS Welcome Replay And KeyPackage Consumption

Generated: 2026-05-28

## Primary Source Refresh

- RFC 9420, Joining via Welcome: <https://www.rfc-editor.org/rfc/rfc9420.html#section-12.4.3.1>
- RFC 9420, KeyPackage reuse: <https://www.rfc-editor.org/rfc/rfc9420.html#section-16.8>
- RFC 9750, KeyPackage/Welcome deployment guidance: <https://datatracker.ietf.org/doc/html/rfc9750#section-5.1>
- RFC 9750, fork handling: <https://datatracker.ietf.org/doc/html/rfc9750#section-5.2>
- RFC 9750, application replay considerations: <https://datatracker.ietf.org/doc/html/rfc9750#section-8.6>
- MLS PQ ciphersuite draft: <https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/>
- NIST FIPS 203, ML-KEM: <https://csrc.nist.gov/pubs/fips/203/final>
- NIST FIPS 204, ML-DSA: <https://csrc.nist.gov/pubs/fips/204/final>

The important correction from this research pass is that an exact Welcome digest cache is useful but incomplete. It blocks byte-identical replay, but it does not by itself prove that a KeyPackage was consumed once, the init key was deleted, the resulting tree/transcript state was bound, or the group state was durably committed.

## Implementation Increment

Added an MLS Welcome replay-store boundary that:

- accepts only after MLS Welcome admission accepts
- rejects rejected admission decisions before storage is touched
- rejects malformed group ids, Welcome hashes, consumed KeyPackage refs, tree hashes, transcript hashes, and group-state commit digests
- rejects non-positive epochs and invalid join timestamps
- rejects missing init-key deletion
- rejects missing transactional group-state commit evidence
- rejects plaintext metadata fields
- persists digest-only accepted Welcome records
- rejects duplicate Welcome hashes per group
- rejects reused consumed KeyPackage refs
- exposes checked fixtures and backend commands for accepted, rejected, duplicate, bad-shape, and plaintext-blocked states

## Security Effect

This closes a receiving-side replay and crash-recovery gap before group initialization. UI and platform adapters now have a backend decision that says not only "the Welcome was admitted", but also "the accepted Welcome was durably bound to a consumed KeyPackage, init-key deletion, tree/transcript state, and committed local group state."

## PQ Note

The current PQ MLS ciphersuite work is still draft-stage, while ML-KEM and ML-DSA are finalized NIST standards. Mercury should keep its current suite names as policy classes until the production provider maps them to audited MLS ciphersuite identifiers, KAT evidence, downgrade evidence, and provider build identity.

## Verification

Focused checks passed during development:

```powershell
cargo test -p mercury-core --test mls_welcome_replay_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Full repo preflight remains the final merge gate:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Research Target

Move from prototype replay boundaries to production MLS provider mapping: provider choice, PQ ciphersuite identifiers, KeyPackage consumption API behavior, transcript evidence, epoch authenticator comparison, and crash-safe local group-state persistence.
