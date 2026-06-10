# Security Research Cycle 26: Proof Bundle And Offline Verification Gate

Generated: 2026-05-28

## Sources Reviewed

- C2SP transparency log proofs: <https://c2sp.org/tlog-proof>
- C2SP transparency log checkpoints: <https://c2sp.org/tlog-checkpoint>
- C2SP transparency log witness protocol: <https://c2sp.org/tlog-witness>
- C2SP transparency log cosignatures: <https://c2sp.org/tlog-cosignature>
- Building a Transparent Keyserver: <https://words.filippo.io/keyserver-tlog/>
- SoK: Log Based Transparency Enhancing Technologies: <https://arxiv.org/abs/2305.01378>

## Finding

The witness-client gate prevents Mercury from trusting raw witness responses. The next risk is losing or misreporting proof material after a checkpoint has been witnessed.

The C2SP transparency log proof format separates a checkpoint from compact inclusion proof material. C2SP checkpoints define the signed log state, witness cosignatures bind witness signatures to the checkpoint, and the witness protocol requires bounded consistency proof material when advancing checkpoints. Transparency-system surveys also emphasize monitor and auditor roles, which means proof status should be locally verifiable without sending plaintext audit selectors to a server.

For Mercury, the proof bundle must therefore be a local, digest-only, policy-bound cache entry rather than a UI convenience payload.

## Increment

Added the sealed audit proof-bundle gate:

- `SealedAuditProofBundleReason`
- `SealedAuditProofBundleInput`
- `SealedAuditProofBundleDecision`
- `evaluate_sealed_audit_proof_bundle(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The gate checks accepted witness-client state, verifier policy snapshot shape, log and witness key pins, witness threshold and cosignature count, persisted proof bundle shape, encrypted append-only proof cache state, inclusion proof verification, consistency proof evidence, witness timestamp freshness, private monitor freshness evidence, authenticated cache recovery, and selector-free UI status.

## Security Impact

Mercury now has five sealed-audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness
4. witness client and private monitor operation readiness
5. proof bundle persistence and offline verification readiness

The fifth layer prevents UI/platform code from treating audit proof state as ready unless it is tied to an accepted witness client, locally persisted as encrypted append-only cache state, verifier-policy-bound, fresh, inclusion-proof-verified, consistency-evidence-backed, and free of plaintext audit selectors.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_proof_bundle
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_proof_bundle_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_proof_bundle_ready
```

The full preflight passed before this increment was committed.

## Next Research Target

Cycle 27 implemented the durable proof-cache adapter boundary. The next research target is the verifier policy snapshot store and private monitor freshness scheduler:

- policy snapshot import, expiry, and rotation
- scheduled offline re-verification of cached proof records
- monitor freshness refresh without plaintext audit selectors
- split-view escalation state for proof-cache mismatches
- UI state transitions for proof verified, proof stale, recovery required, and policy refresh required
