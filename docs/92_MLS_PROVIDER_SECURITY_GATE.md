# MLS Provider Security Gate

Generated: 2026-05-28

## Status

Mercury now has an MLS provider-security gate in `mercury-core`:

```text
MlsProviderSecurityInput
MlsProviderSecurityDecision
MlsProviderSecurityReason
evaluate_mls_provider_security(...)
```

This is not a production MLS implementation and does not implement ML-KEM, HPKE, MLS tree crypto, signatures, or AEAD. It is the contract a future provider adapter must satisfy before `GroupChatDecision` can accept an MLS-backed group.

## Accepted Provider Contract

The accepted provider path requires:

- provider configured
- selected suite supported by the provider
- selected suite not below the room minimum suite floor
- selected suite backed by the required ML-KEM parameter set
- PQ/traditional hybrid KEM component present for Mercury hybrid suite classes
- PQ-signature readiness when the room/provider asks for it
- suite id bound to the group context
- downgrade evidence verified
- known-answer tests passed for the provider mapping
- secret zeroization available
- unsafe crypto backend flag unset
- plaintext key-export fields equal zero

Accepted output enables:

```text
can_use_provider = true
can_open_mls_group = true
```

Rejected output blocks group readiness when the group protocol is MLS. `evaluate_group_chat(...)` maps this to:

```text
MLS_PROVIDER_SECURITY_REJECTED
```

## Reason Labels

Stable provider-security labels:

```text
ACCEPTED
PROVIDER_MISSING
SUITE_UNSUPPORTED
DOWNGRADE_BELOW_FLOOR
ML_KEM_PARAMETER_SET_REQUIRED
PQ_TRADITIONAL_HYBRID_REQUIRED
PQ_SIGNATURE_REQUIRED
SUITE_CONTEXT_BINDING_MISSING
DOWNGRADE_EVIDENCE_MISSING
KNOWN_ANSWER_TESTS_MISSING
SECRET_ZEROIZATION_MISSING
UNSAFE_CRYPTO_BACKEND
PLAINTEXT_KEY_EXPORT_FORBIDDEN
```

## Fixture Surface

The group chat fixture input now exposes:

```text
mls_provider_security_accepted
mls_provider_security_reason_label
mls_provider_security_requires_mls_setup
mls_provider_security_requires_pq_upgrade
mls_provider_security_requires_user_action
```

New checked fixture:

```text
group_chat_mls_provider_security_required
```

MLS provider evidence persistence is exposed separately through:

```text
mls_provider_evidence_store_ready
mls_provider_evidence_store_gate_rejected
mls_provider_evidence_store_duplicate_rejected
mls_provider_evidence_store_plaintext_rejected
mls_provider_evidence_use_ready
mls_provider_evidence_use_missing
mls_provider_evidence_use_expired
mls_provider_evidence_use_suite_mismatch
mls_provider_evidence_use_plaintext_rejected
```

Backend command envelopes expose the group-chat and provider-evidence states through `run_group_chat_*`, `run_mls_provider_evidence_store_*`, and `run_mls_provider_evidence_use_*`, including:

```text
run_group_chat_mls_ready
run_group_chat_mls_provider_security_required
run_mls_provider_evidence_store_ready
run_mls_provider_evidence_store_gate_rejected
run_mls_provider_evidence_store_duplicate_rejected
run_mls_provider_evidence_store_plaintext_rejected
run_mls_provider_evidence_use_ready
run_mls_provider_evidence_use_missing
run_mls_provider_evidence_use_expired
run_mls_provider_evidence_use_suite_mismatch
run_mls_provider_evidence_use_plaintext_rejected
```

MLS provider adapter selection is exposed separately through:

```text
mls_provider_adapter_selection_ready
mls_provider_adapter_selection_provider_rejected
mls_provider_adapter_selection_pq_draft_rejected
mls_provider_adapter_selection_storage_rejected
mls_provider_adapter_selection_supply_chain_rejected
run_mls_provider_adapter_selection_ready
run_mls_provider_adapter_selection_provider_rejected
run_mls_provider_adapter_selection_pq_draft_rejected
run_mls_provider_adapter_selection_storage_rejected
run_mls_provider_adapter_selection_supply_chain_rejected
```

See `docs/118_MLS_PROVIDER_ADAPTER_SELECTION.md` for the concrete library/backend/profile provenance gate.

## Research Basis

- IETF Datatracker lists `draft-ietf-mls-pq-ciphersuites-04` as the current MLS PQ ciphersuite draft: https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/
- IETF `draft-ietf-hpke-pq-04` defines ML-KEM and PQ/traditional hybrid HPKE KEMs, including ML-KEM-768, ML-KEM-1024, X25519+ML-KEM-768, P-256+ML-KEM-768, and P-384+ML-KEM-1024: https://www.ietf.org/archive/id/draft-ietf-hpke-pq-04.html
- NIST FIPS 203 standardizes ML-KEM with parameter sets ML-KEM-512, ML-KEM-768, and ML-KEM-1024: https://csrc.nist.gov/pubs/fips/203/final
- NIST ACVP ML-KEM JSON validation vectors cover FIPS 203 ML-KEM key-generation and encapsulation/decapsulation testing, while implementation-lifecycle evidence such as zeroization still needs separate provider evidence: https://pages.nist.gov/ACVP/draft-celi-acvp-ml-kem.html
- NIST announced the first finalized PQC standards in August 2024, including ML-KEM, ML-DSA, and SLH-DSA: https://csrc.nist.gov/News/2024/postquantum-cryptography-fips-approved

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_provider_security
cargo test -p mercury-core --test mls_provider_evidence_store
cargo test -p mercury-core --test group_chat_readiness
cargo test -p mercury-core --test group_message_transcript
cargo test -p mercury-core --test anonymous_group_membership_proof
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype group_chat_mls_provider_security_required
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_evidence_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_evidence_use_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_group_chat_mls_provider_security_required
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_use_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

When a production MLS provider is selected, first pass `MlsProviderAdapterSelectionDecision`. Then map `hybrid_pq_mls_768` and `hybrid_pq_mls_1024` to real provider ciphersuite identifiers, provider KAT results, zeroization evidence, and downgrade-evidence records, persist digest-only evidence through `MlsProviderEvidenceStoreAdapter`, and require accepted `MlsProviderEvidenceUseDecision` before treating evidence as current.
