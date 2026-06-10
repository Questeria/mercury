# MLS Provider Evidence Use Gate

Generated: 2026-05-28

## Status

Mercury now has a read-time gate for stored MLS provider evidence:

```text
MlsProviderEvidenceUseInput
MlsProviderEvidenceUseDecision
MlsProviderEvidenceUseReason
evaluate_mls_provider_evidence_use(...)
```

The evidence store controls what may be persisted. This gate controls whether a persisted evidence record is current and safe enough to use for provider readiness.

## Accepted Use Contract

Accepted provider evidence requires:

- a stored evidence record
- accepted provider-security decision
- provider-security suite matching the required suite
- evidence record suite matching the required suite
- all evidence digests still shaped as 32-byte digests
- validation window not expired
- validation time not in the future
- no plaintext-tainted evidence record

Accepted output enables:

```text
can_use_provider_evidence = true
```

## Rejection Labels

Stable labels:

```text
RECORD_MISSING
PROVIDER_SECURITY_REJECTED
SUITE_MISMATCH
EVIDENCE_NOT_YET_VALID
EVIDENCE_EXPIRED
BAD_EVIDENCE_SHAPE
PLAINTEXT_EVIDENCE_DETECTED
```

## Fixture Surface

Checked fixtures:

```text
mls_provider_evidence_use_ready
mls_provider_evidence_use_missing
mls_provider_evidence_use_expired
mls_provider_evidence_use_suite_mismatch
mls_provider_evidence_use_plaintext_rejected
```

Backend command envelopes:

```text
run_mls_provider_evidence_use_ready
run_mls_provider_evidence_use_missing
run_mls_provider_evidence_use_expired
run_mls_provider_evidence_use_suite_mismatch
run_mls_provider_evidence_use_plaintext_rejected
```

## Research Basis

The gate follows from the MLS/PQ provider evidence model documented in `docs/95_MLS_PROVIDER_EVIDENCE_STORE.md`: validation artifacts must be stored as digest-only evidence, but stale or suite-mismatched evidence must not be usable for current MLS provider readiness.

Primary sources remain:

- IETF MLS PQ ciphersuites datatracker: https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/
- NIST FIPS 203 ML-KEM: https://csrc.nist.gov/pubs/fips/203/final
- NIST ACVP ML-KEM JSON validation shape: https://pages.nist.gov/ACVP/draft-celi-acvp-ml-kem.html

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_provider_evidence_store
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_evidence_use_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_use_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

When the production MLS provider adapter is implemented behind `MlsProviderAdapterSelectionDecision`, group readiness should consume this use decision so stale, missing, or suite-mismatched provider evidence cannot open a group room.
