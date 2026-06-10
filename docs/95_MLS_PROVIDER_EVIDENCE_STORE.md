# MLS Provider Evidence Store

Generated: 2026-05-28

## Status

Mercury now has a digest-only evidence-store boundary for MLS provider validation:

```text
MlsProviderEvidenceStoreWrite
MlsProviderEvidenceStoreDecision
MlsProviderEvidenceStoreReason
MlsProviderEvidenceStoreAdapter
PrototypeMlsProviderEvidenceStore
put_mls_provider_evidence_record(...)
MlsProviderEvidenceUseInput
MlsProviderEvidenceUseDecision
evaluate_mls_provider_evidence_use(...)
```

This does not implement MLS, ML-KEM, HPKE, ML-DSA, or provider validation. It is the accepted-only storage contract for evidence produced by a future audited provider adapter.

## Accepted Write Contract

The store accepts a record only when:

- the provider-security decision is accepted and usable
- evidence id is 32 bytes
- provider id digest is 32 bytes
- suite evidence digest is 32 bytes
- ML-KEM known-answer/vector evidence digest is 32 bytes
- downgrade evidence digest is 32 bytes
- zeroization evidence digest is 32 bytes
- validation window is positive and expires after validation time
- plaintext evidence fields are zero
- the evidence id has not already been recorded

Accepted records persist only opaque digests and validation metadata. The store never accepts plaintext provider evidence, raw KAT vectors, raw keys, raw transcripts, plaintext suite metadata, or plaintext key-export fields.

## Reason Labels

Stable labels:

```text
ACCEPTED
PROVIDER_SECURITY_REJECTED
BAD_EVIDENCE_ID
BAD_PROVIDER_ID_DIGEST
BAD_SUITE_EVIDENCE_DIGEST
BAD_KAT_EVIDENCE_DIGEST
BAD_DOWNGRADE_EVIDENCE_DIGEST
BAD_ZEROIZATION_EVIDENCE_DIGEST
BAD_VALIDATION_WINDOW
PLAINTEXT_EVIDENCE_FORBIDDEN
EVIDENCE_ALREADY_RECORDED
```

## Fixture Surface

Checked fixtures:

```text
mls_provider_evidence_store_ready
mls_provider_evidence_store_gate_rejected
mls_provider_evidence_store_duplicate_rejected
mls_provider_evidence_store_plaintext_rejected
```

Backend command envelopes:

```text
run_mls_provider_evidence_store_ready
run_mls_provider_evidence_store_gate_rejected
run_mls_provider_evidence_store_duplicate_rejected
run_mls_provider_evidence_store_plaintext_rejected
```

The fixture JSON exposes `keeps_digest_only`, `can_use_as_provider_evidence`, `plaintext_bytes_exposed`, and store record count so UI/platform clients can verify the security boundary without reimplementing store policy.

Read-time provider evidence use is exposed through:

```text
mls_provider_evidence_use_ready
mls_provider_evidence_use_missing
mls_provider_evidence_use_expired
mls_provider_evidence_use_suite_mismatch
mls_provider_evidence_use_plaintext_rejected
```

These fixtures reject missing, expired, suite-mismatched, malformed, or plaintext-tainted provider evidence before the record can count as current provider readiness.

## Research Basis

- IETF Datatracker now lists `draft-ietf-mls-pq-ciphersuites-04` as the current MLS PQ ciphersuite draft: https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/
- The MLS PQ draft registers ML-KEM and hybrid MLS ciphersuites and distinguishes PQ confidentiality from full PQ authentication in suites with traditional signatures.
- NIST FIPS 203 standardizes ML-KEM and is the baseline for Mercury ML-KEM parameter-set policy: https://csrc.nist.gov/pubs/fips/203/final
- NIST ACVP's ML-KEM JSON specification covers ML-KEM key-generation and encapsulation/decapsulation validation vectors for FIPS 203, but explicitly leaves some implementation requirements such as zeroization outside ACVP-server testing: https://pages.nist.gov/ACVP/draft-celi-acvp-ml-kem.html

## Design Consequence

Mercury stores separate digest slots for:

- suite mapping evidence
- KAT/vector evidence
- downgrade evidence
- zeroization evidence

This keeps the future production adapter honest: passing ML-KEM vectors alone is not enough to persist a provider as safe for MLS group use.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_provider_evidence_store
cargo test -p mercury-core --test mls_provider_security
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_evidence_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_evidence_use_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_use_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

When a production MLS provider is selected, its adapter should generate these evidence digests from real ciphersuite identifiers, ACVP or equivalent KAT output, downgrade transcript checks, zeroization/secret-lifecycle audit output, and provider build identity.
