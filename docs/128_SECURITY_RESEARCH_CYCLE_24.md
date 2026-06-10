# Security Research Cycle 24: Witnessed Checkpoint Operations

Generated: 2026-05-28

## Sources Reviewed

- C2SP transparency log checkpoints: <https://c2sp.org/tlog-checkpoint>
- C2SP transparency log cosignatures: <https://c2sp.org/tlog-cosignature>
- C2SP transparency log witness protocol: <https://c2sp.org/tlog-witness>
- C2SP transparency log proofs: <https://c2sp.org/tlog-proof>
- transparency-dev witness overview: <https://pkg.go.dev/github.com/transparency-dev/witness>
- transparency.dev, Building a Transparent Keyserver: <https://blog.transparency.dev/building-a-transparent-keyserver>
- transparency-dev Tessera witnessing notes: <https://github.com/transparency-dev/tessera>
- IETF draft, Gossiping in CT: <https://datatracker.ietf.org/doc/draft-ietf-trans-gossip/01/>
- Hicks, SoK: Log Based Transparency Enhancing Technologies: <https://arxiv.org/abs/2305.01378>
- NIST FIPS 204, Module-Lattice-Based Digital Signature Standard: <https://csrc.nist.gov/pubs/fips/204/final>

## Finding

The sealed audit store makes audit records replay-resistant locally, but it does not by itself prove that all clients, monitors, and operators see the same audit log view.

The reviewed transparency-log specifications point to a stricter operation model:

- A checkpoint is a signed Merkle tree head with origin, tree size, and root hash.
- Logs must not sign inconsistent checkpoints.
- Witnesses countersign only after checking consistency against their last observed checkpoint.
- Clients should verify a policy-defined quorum of witness cosignatures before trusting a logged inclusion proof.
- The C2SP witness protocol bounds submitted consistency proof lines and returns conflict state when the witness has already observed a different latest checkpoint.
- Modern new deployments should prefer ML-DSA-44 cosignatures; Mercury uses this as the reason to require PQ or hybrid checkpoint signatures at the policy gate.
- Monitoring and transparency can create privacy hazards if queries reveal the user, room, device, or conversation being monitored.

## Increment

Added the sealed audit witness/checkpoint gate:

- `SealedAuditCheckpointSignatureAlgorithm`
- `SealedAuditWitnessCheckpointReason`
- `SealedAuditWitnessCheckpointInput`
- `SealedAuditWitnessCheckpointDecision`
- `evaluate_sealed_audit_witness_checkpoint(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing docs for fixture and command consumption

The gate checks store acceptance, checkpoint shape, PQ/hybrid signature policy, signing-key rotation state, consistency proofs, witness quorum, operator diversity, key pinning, cosignature binding, split-view evidence, monitor privacy, and local checkpoint recovery state.

## Security Impact

Mercury can now distinguish three audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness

That separation makes the future production publisher safer: a networked witness adapter can be wired behind an explicit decision object instead of being allowed to publish raw or partially checked audit state.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_witness_checkpoint
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_witness_checkpoint_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_witness_checkpoint_ready
```

The full preflight passed before this increment was committed.

## Next Research Target

This target is implemented by `docs/129_SEALED_AUDIT_WITNESS_CLIENT.md` and `docs/130_SECURITY_RESEARCH_CYCLE_25.md`.

The next research target is proof bundle persistence and offline verification: proof cache integrity, verifier policy snapshots, witness timestamp freshness, recovery after proof cache loss, and UI-safe proof status reporting without leaking audit selectors.
