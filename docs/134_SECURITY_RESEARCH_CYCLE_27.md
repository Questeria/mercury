# Security Research Cycle 27: Durable Proof Cache And Offline Verifier Boundary

Generated: 2026-05-28

## Sources Reviewed

- C2SP transparency log proofs: <https://c2sp.org/tlog-proof>
- C2SP transparency log witness protocol: <https://c2sp.org/tlog-witness>
- Tile-Based Transparency Logs: <https://transparency.dev/articles/tile-based-logs/>
- Transparency Logs: A Verifiable Transport Layer: <https://transparency.dev/articles/logs-a-verifiable-transport-layer/>
- SoK: Log Based Transparency Enhancing Technologies: <https://arxiv.org/abs/2305.01378>

## Finding

The proof-bundle gate established that Mercury should accept offline audit proof status only when a proof is witness-client-approved, verifier-policy-bound, inclusion-proof-verified, consistency-evidence-backed, fresh, and selector-free.

The next risk is treating proof bundles as transient UI data. C2SP proof bundles are explicitly offline-verifiable against trusted log and witness keys. Transparency.dev's tile-based log work emphasizes cacheable proof material and static responses, while its verifier guidance stresses that local verifiers should keep their own database and periodically check it for consistency and duplicates. The SoK literature also highlights the privacy and query risks around transparency systems.

For Mercury, this means the proof cache must be an accepted-only encrypted adapter boundary. UI/platform code should read proof-cache status, not construct proof trust on its own.

## Increment

Added the sealed audit proof-cache adapter:

- `SealedAuditProofCacheReason`
- `SealedAuditProofCacheWrite`
- `SealedAuditProofCacheRecord`
- `SealedAuditProofCacheDecision`
- `AcceptedSealedAuditProofCacheWrite`
- `SealedAuditProofCacheAdapter`
- `PrototypeSealedAuditProofCache`
- `evaluate_sealed_audit_proof_cache_write(...)`
- `put_sealed_audit_proof_cache_record(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The adapter accepts only proof-bundle-approved, digest-only, encrypted, append-only cache records that pass offline verification, monitor freshness checks, policy snapshot binding, duplicate checks, rollback-index checks, and authenticated cache recovery checks.

## Security Impact

Mercury now has six sealed-audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness
4. witness client and private monitor operation readiness
5. proof bundle persistence and offline verification readiness
6. accepted-only proof-cache persistence

The sixth layer prevents production storage code from persisting proof status unless it remains policy-bound, replayable offline, duplicate-safe, rollback-resistant, encrypted, append-only, digest-only, and selector-free.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_proof_cache
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_proof_cache_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_proof_cache_ready
```

The full preflight passed before this increment was committed.

## Next Research Target

Cycle 28 implemented the verifier policy snapshot store and private monitor freshness scheduler boundary. The next research target is the sealed audit split-view evidence and privacy-preserving incident report boundary:

- contradiction evidence storage without plaintext audit selectors
- missing-proof or monitor-failure report flow
- policy-bound escalation routing
- witness/operator accountability state
- UI-safe incident states for split view, stale monitor, and unverifiable proof-cache records
