# Anonymous Issuer Witness Audit

Generated: 2026-05-28

## Status

Mercury now has a backend witness/auditor gate for anonymous credential issuer keys:

```text
AnonymousIssuerWitnessAuditInput
AnonymousIssuerWitnessAuditDecision
AnonymousIssuerWitnessAuditReason
evaluate_anonymous_issuer_witness_audit(...)
```

`AnonymousCredentialIssuerTrustInput` consumes the witness-audit decision. Issuer trust now rejects with `ISSUER_WITNESS_AUDIT_REJECTED` if the witness/auditor gate rejects, even when the base key-transparency decision is otherwise consistent.

## Accepted Audit

Accepted audit requires:

- consistent key transparency
- 32-byte signed-tree-head digest
- 32-byte inclusion-root digest
- monotonic tree size
- configured witness quorum
- at least two independent operators
- fresh audit age
- zero split-view reports
- 64-byte auditor signature
- zero plaintext partitioning fields

Accepted output enables:

```text
can_use_issuer_key = true
has_witness_quorum = true
detects_split_view = true
protects_anonymity_set = true
plaintext_bytes_exposed = false
```

## Rejection Classes

Stable rejection labels:

```text
KEY_TRANSPARENCY_REJECTED
BAD_SIGNED_TREE_HEAD
TREE_SIZE_ROLLBACK
WITNESS_QUORUM_MISSING
OPERATOR_DIVERSITY_MISSING
AUDIT_STALE
SPLIT_VIEW_REPORTED
AUDITOR_SIGNATURE_MISSING
PLAINTEXT_PARTITIONING_METADATA
```

## Verification

Run:

```powershell
cargo test -p mercury-core --test anonymous_issuer_witness_audit
cargo test -p mercury-core --test anonymous_credential_issuer_trust
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_credential_issuer_trust_witness_audit_rejected
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

Map this gate to a production issuer transparency deployment with independent witnesses or auditors, signed checkpoints, split-view evidence handling, and a small-deployment operations model.
