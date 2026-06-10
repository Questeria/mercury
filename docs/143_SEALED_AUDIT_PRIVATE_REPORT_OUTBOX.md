# Sealed Audit Private Report Outbox

Generated: 2026-05-28

## Purpose

Mercury now has a checked accepted-only outbox boundary for private sealed-audit incident report submission.

The outbox sits behind the private report transport gate. It rejects report records unless the transport already proved OHTTP-style route privacy, HPKE-protected requests, authenticated gateway responses, anonymous rate-limit controls, replay guards, retry safety, encrypted payload state, and selector-free status.

This keeps "can submit a private report" separate from "has durably recorded a safe report submission attempt." The second boundary matters for crash recovery, duplicate suppression, retry/backoff, and auditability.

## Core Surface

Implemented in `core/rust/mercury-core/src/lib.rs`:

- `SealedAuditPrivateReportOutboxReason`
- `SealedAuditPrivateReportOutboxWrite`
- `SealedAuditPrivateReportOutboxRecord`
- `SealedAuditPrivateReportOutboxDecision`
- `AcceptedSealedAuditPrivateReportOutboxWrite`
- `SealedAuditPrivateReportOutboxStore`
- `PrototypeSealedAuditPrivateReportOutbox`
- `evaluate_sealed_audit_private_report_outbox(...)`
- `put_sealed_audit_private_report_outbox_record(...)`

## Accepted Requirements

Accepted private report outbox decisions require:

- accepted private report transport decision
- matching policy epoch, proof epoch, and audit log index
- digest-only report id, payload, OHTTP request transcript, and gateway response transcript
- encrypted payload and encrypted outbox storage
- append-only outbox guard
- monotonic sequence advancement
- prior report binding when retrying after the first sequence
- replay-window binding
- duplicate report rejection
- retry backoff persistence
- Privacy Pass-style token binding and spend-once state
- anonymous rate-limit enforcement
- OHTTP encapsulated request and gateway response
- relay/gateway separation
- no cookie, account, or reusable authentication state
- private route selection
- digest-only UI status

Rejected records do not mutate the store.

## Prototype Fixtures

Checked-in fixtures:

```text
sealed_audit_private_report_outbox_ready
sealed_audit_private_report_outbox_transport_rejected
sealed_audit_private_report_outbox_replay_rejected
sealed_audit_private_report_outbox_rate_limit_rejected
sealed_audit_private_report_outbox_plaintext_rejected
```

Backend command envelopes:

```text
run_sealed_audit_private_report_outbox_ready
run_sealed_audit_private_report_outbox_transport_rejected
run_sealed_audit_private_report_outbox_replay_rejected
run_sealed_audit_private_report_outbox_rate_limit_rejected
run_sealed_audit_private_report_outbox_plaintext_rejected
```

## Security Impact

Mercury now has eleven sealed-audit layers:

1. event-chain validity
2. accepted-only local audit persistence
3. witnessed checkpoint publication readiness
4. witness client and private monitor operation readiness
5. proof bundle persistence and offline verification readiness
6. accepted-only proof-cache persistence
7. verifier policy snapshot and private monitor freshness readiness
8. accepted-only incident evidence and privacy-preserving report readiness
9. accepted-only recovery/export and cross-device incident sync readiness
10. production sealed-audit database and private report transport readiness
11. accepted-only private report outbox and submission transcript persistence

The eleventh layer prevents incident-report retry, crash recovery, or submission status UI from bypassing route privacy, anonymous rate limits, replay protection, encrypted outbox storage, or selector redaction.

## Verification

Focused checks:

```powershell
cargo fmt
cargo test -p mercury-core --test sealed_audit_private_report_outbox
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
```

Simulator checks:

```powershell
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --prototype sealed_audit_private_report_outbox_ready
cargo run -q -p mercury-bindings --bin mercury-ui-sim -- --command run_sealed_audit_private_report_outbox_ready
```

Run the full preflight before pushing the increment:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
