# Security Research Cycle 29: Split-View Incident Evidence And Private Reports

Generated: 2026-05-28

## Sources Reviewed

- SoK: SCT Auditing in Certificate Transparency: <https://research.google/pubs/sok-sct-auditing-in-certificate-transparency/>
- SoK: SCT Auditing in Certificate Transparency: <https://arxiv.org/abs/2203.01661>
- C2SP Transparency Log Cosignatures: <https://c2sp.org/tlog-cosignature>
- C2SP Transparency Log Witness Protocol: <https://c2sp.org/tlog-witness>
- C2SP Offline-Verifiable Transparency Log Proofs: <https://c2sp.org/tlog-proof>
- CONIKS research and open problems: <https://coniks-sys.github.io/research.html>
- Parakeet: Practical Key Transparency for End-to-End Encrypted Messaging: <https://eprint.iacr.org/2023/081.pdf>
- TAP: Transparent and Privacy-Preserving Data Services: <https://www.usenix.org/system/files/usenixsecurity23-reijsbergen.pdf>

## Finding

The verifier policy store blocks stale or selector-bearing monitor state before UI/platform clients can treat audit status as safe. The next risk is incident handling itself: split-view, missing-proof, and monitor-failure evidence can accidentally become a plaintext diagnostics channel if the report flow stores raw subjects, raw selectors, or unbound operator claims.

The CT auditing SoK is especially relevant because it highlights that private proof checking is only half the problem: missing-entry reports also need privacy. C2SP witness/cosignature work reinforces that clients should rely on checkpoint-bound, quorum-verifiable statements to resist split views. CONIKS research notes that accountability for inconsistencies remains difficult: systems need a way to distinguish malicious operators, system error, and compromise without collapsing user privacy. Parakeet and TAP point in the same direction: transparency systems need verifiable commitments, consistency, and privacy-preserving data structures rather than raw public data dumps.

For Mercury, the incident surface should therefore be an accepted-only store that consumes a verifier-policy decision, accepts only digest-bound evidence, and exposes UI-safe incident states without storing plaintext audit selectors.

## Increment

Added the sealed audit incident evidence store:

- `SealedAuditIncidentEvidenceReason`
- `SealedAuditIncidentEvidenceWrite`
- `SealedAuditIncidentEvidenceRecord`
- `SealedAuditIncidentEvidenceDecision`
- `AcceptedSealedAuditIncidentEvidenceWrite`
- `SealedAuditIncidentEvidenceStore`
- `PrototypeSealedAuditIncidentEvidenceStore`
- `evaluate_sealed_audit_incident_evidence(...)`
- `put_sealed_audit_incident_evidence_record(...)`
- focused core tests
- five prototype fixtures
- five backend command envelopes
- UI-facing fixture and command docs

The store accepts only verifier-policy-approved, digest-only, encrypted, append-only incident records with blinded missing-proof reports, private monitor reports, verified contradiction proofs for split-view evidence, witness/operator quorum, accountability routing, retry/backoff metadata, and zero plaintext selectors.

## Security Impact

Mercury now has eight sealed-audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness
4. witness client and private monitor operation readiness
5. proof bundle persistence and offline verification readiness
6. accepted-only proof-cache persistence
7. verifier policy snapshot and private monitor freshness readiness
8. accepted-only incident evidence and privacy-preserving report readiness

The eighth layer prevents split-view and missing-proof handling from becoming a side channel. UI and platform code can show incident state from capability booleans while the stored evidence remains digest-only.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_incident_evidence
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
```

Simulator checks:

```powershell
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_incident_evidence_ready
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_incident_evidence_ready
```

This target is implemented by `docs/139_SEALED_AUDIT_RECOVERY_EXPORT.md` and researched in `docs/140_SECURITY_RESEARCH_CYCLE_30.md`.

## Next Research Target

Design the production sealed-audit database adapter and private report transport boundary:

- durable encrypted database schemas for event, proof-cache, policy, incident, and recovery-export records
- private report transport for missing-proof and split-view incidents
- anti-replay and rate-limited incident report submission
- privacy-preserving sync scheduling without update-pattern leakage
- migration and crash-recovery drills for the sealed-audit database
