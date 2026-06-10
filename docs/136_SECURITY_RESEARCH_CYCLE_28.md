# Security Research Cycle 28: Verifier Policy Store And Private Monitor Scheduler

Generated: 2026-05-28

## Sources Reviewed

- Transparency Logs: A Verifiable Transport Layer: <https://transparency.dev/articles/logs-a-verifiable-transport-layer/>
- Building a Transparent Keyserver: <https://blog.transparency.dev/building-a-transparent-keyserver>
- SoK: SCT Auditing in Certificate Transparency: <https://research.google/pubs/sok-sct-auditing-in-certificate-transparency/>
- SoK: Log Based Transparency Enhancing Technologies: <https://arxiv.org/abs/2305.01378>
- SoK: SCT Auditing in Certificate Transparency: <https://arxiv.org/abs/2203.01661>

## Finding

The proof-cache adapter prevents Mercury from persisting unapproved proof records. The next risk is treating verifier policy and monitor freshness as ambient application settings.

Transparency verifier guidance emphasizes that proof checks are not enough: verifiers need their own local database, timely entry verification, duplicate checks, and policy-aware interpretation of log entries. The transparent keyserver work also makes policy an explicit parseable object rather than a single hardcoded verifier key, and it uses witness cosigning to reduce split-view risk. CT auditing research highlights the privacy problem: direct proof queries and reports can reveal the subject being checked.

For Mercury, verifier policy and private monitor freshness must therefore be a checked local state machine. UI/platform code should consume policy status and monitor freshness decisions, not assemble trust from raw keys, endpoints, or selector-bearing queries.

## Increment

Added the sealed audit verifier policy store:

- `SealedAuditVerifierPolicyReason`
- `SealedAuditVerifierPolicySnapshot`
- `SealedAuditVerifierPolicyRecord`
- `SealedAuditVerifierPolicyDecision`
- `AcceptedSealedAuditVerifierPolicySnapshot`
- `SealedAuditVerifierPolicyStore`
- `PrototypeSealedAuditVerifierPolicyStore`
- `evaluate_sealed_audit_verifier_policy_snapshot(...)`
- `put_sealed_audit_verifier_policy_snapshot(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The store accepts only proof-cache-approved, digest-only policy snapshots that are signature-verified, consistency-proof-verified, non-expired, key-rotation-authenticated when required, monitor-fresh, scheduler-encrypted, append-only, and selector-free.

## Security Impact

Mercury now has seven sealed-audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness
4. witness client and private monitor operation readiness
5. proof bundle persistence and offline verification readiness
6. accepted-only proof-cache persistence
7. verifier policy snapshot and private monitor freshness readiness

The seventh layer prevents local UI/platform code from relying on stale verifier policy, unverified key rotations, stale monitor state, split-view evidence, or plaintext monitor selectors.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_verifier_policy
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_verifier_policy_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_verifier_policy_ready
```

The full preflight passed before this increment was committed.

## Next Research Target

Implemented in `docs/137_SEALED_AUDIT_INCIDENT_EVIDENCE.md` and researched in `docs/138_SECURITY_RESEARCH_CYCLE_29.md`: the sealed audit split-view evidence and privacy-preserving incident report boundary:

- contradiction evidence storage without plaintext audit selectors
- missing-proof or monitor-failure report flow
- policy-bound escalation routing
- witness/operator accountability state
- UI-safe incident states for split view, stale monitor, and unverifiable proof-cache records

Next research target after that increment is the sealed-audit recovery/export ceremony and cross-device incident sync boundary.
