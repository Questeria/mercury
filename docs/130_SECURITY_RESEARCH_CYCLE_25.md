# Security Research Cycle 25: Witness Client And Private Monitor Gate

Generated: 2026-05-28

## Sources Reviewed

- C2SP transparency log witness protocol: <https://c2sp.org/tlog-witness>
- C2SP transparency log cosignatures: <https://c2sp.org/tlog-cosignature>
- C2SP transparency log checkpoints: <https://c2sp.org/tlog-checkpoint>
- C2SP transparency log proofs: <https://c2sp.org/tlog-proof>
- Building a Transparent Keyserver: <https://words.filippo.io/keyserver-tlog/>
- SoK: Log Based Transparency Enhancing Technologies: <https://arxiv.org/abs/2305.01378>
- Verifiable Light-Weight Monitoring for Certificate Transparency Logs: <https://arxiv.org/abs/1711.03952>

## Finding

The checkpoint gate established that Mercury should only publish witnessed checkpoints when the local event store, checkpoint shape, cosignature policy, monitor privacy, and recovery state are acceptable. The next risk is the production witness client itself becoming a bypass or ambiguity layer.

The witness protocol has several operational requirements that should be represented before a network adapter is written:

- The `add-checkpoint` request carries an old size, consistency proof lines, an empty line, and then the checkpoint.
- Clients must not send more than 63 consistency proof lines.
- A witness returns `409 Conflict` when the submitted old size does not match the latest checkpoint it has cosigned.
- A witness returns `422 Unprocessable Entity` when the Merkle consistency proof does not verify.
- Witnesses must persist the new checkpoint before responding, and the check/update must be atomic to avoid rollback races.
- Clients must ignore unknown cosignatures.
- Clients and monitors rely on configured policies, not arbitrary signatures, to decide which cosignature quorum is strong enough.
- Privacy-preserving transparency systems should avoid monitoring APIs that reveal user identifiers or monitored subjects.

## Increment

Added the sealed audit witness client gate:

- `SealedAuditWitnessClientReason`
- `SealedAuditWitnessClientInput`
- `SealedAuditWitnessClientDecision`
- `evaluate_sealed_audit_witness_client(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing docs for fixture and command consumption

The gate checks policy binding, witness key pins, operator diversity, endpoint hardening, request shape, proof-line bound, response status mapping, known cosignature quorum, atomic persistence, split-view alert routing, private monitor retrieval, VRF/blinded monitor selectors, and authenticated recovery.

## Security Impact

Mercury now has four sealed-audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness
4. witness client and private monitor operation readiness

The fourth layer prevents production network code from treating a witness response as useful unless it is policy-bound, quorum-valid, atomically persisted, privacy-preserving, and alertable on conflict or split-view evidence.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_witness_client
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_witness_client_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_witness_client_ready
```

The full preflight passed before this increment was committed.

## Next Research Target

Implemented in `docs/131_SEALED_AUDIT_PROOF_BUNDLE.md` and `docs/132_SECURITY_RESEARCH_CYCLE_26.md`.

Next target after that increment is the durable proof-cache adapter and offline verifier runner:

- serialized proof-bundle schema and migration policy
- verifier policy snapshot storage and rotation
- background verification schedule
- authenticated proof-cache recovery protocol
- privacy-preserving monitor freshness refresh
- UI state transitions for proof verified, proof stale, recovery required, and policy refresh required
