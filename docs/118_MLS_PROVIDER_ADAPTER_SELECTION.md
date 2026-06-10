# MLS Provider Adapter Selection Gate

Generated: 2026-05-28

## Status

Mercury now has an MLS provider adapter-selection gate in `mercury-core`:

```text
MlsProviderAdapterKind
MlsProviderCryptoBackendKind
MlsProviderProtocolProfile
MlsProviderImplementationLicenseKind
MlsProviderAdapterSelectionInput
MlsProviderAdapterSelectionDecision
MlsProviderAdapterSelectionReason
evaluate_mls_provider_adapter_selection(...)
```

This is not a production MLS provider. It is the deployment/provenance gate a future provider adapter must satisfy before Mercury links a real MLS library behind the existing group-chat, KeyPackage, Welcome, Commit, and membership transaction contracts.

## Accepted Adapter Contract

The accepted path requires:

- accepted `MlsProviderSecurityDecision`
- non-custom, known MLS provider adapter kind
- crypto backend allowed for the selected suite
- protocol profile matching the selected suite class
- redistribution-safe implementation license
- verified provider source provenance
- RFC 9420 conformance tests passed
- pinned MLS PQ draft version for hybrid PQ suites
- standardized ML-KEM when hybrid PQ is used
- standardized PQ signature evidence for high-security PQ suites
- known-answer vectors and interop tests passed
- storage provider seals group state and is transactional
- audited secret zeroization and memory hardening
- downgrade tests and transcript-hash binding tests passed
- no unsafe/debug crypto features
- no plaintext key export
- signed release artifact, SBOM, and CVE monitoring

Accepted output enables:

```text
can_link_provider = true
can_open_mls_group = true
can_change_membership = true
can_ship_release = true
forbids_plaintext_key_export = true
```

## Reason Labels

Stable adapter-selection labels:

```text
ACCEPTED
PROVIDER_SECURITY_REJECTED
ADAPTER_KIND_REJECTED
CRYPTO_BACKEND_REJECTED
PROTOCOL_PROFILE_REJECTED
LICENSE_REJECTED
SOURCE_AUTHENTICITY_MISSING
RFC9420_CONFORMANCE_MISSING
PQ_DRAFT_PIN_MISSING
ML_KEM_STANDARD_MISSING
PQ_SIGNATURE_STANDARD_MISSING
KAT_OR_INTEROP_MISSING
STORAGE_PROVIDER_UNSAFE
SECRET_LIFECYCLE_UNSAFE
DOWNGRADE_TEST_MISSING
TRANSCRIPT_BINDING_MISSING
UNSAFE_FEATURES_ENABLED
PLAINTEXT_EXPORT_ENABLED
RELEASE_ARTIFACT_UNVERIFIED
SBOM_OR_CVE_MONITORING_MISSING
```

## Fixture Surface

Checked fixtures:

```text
mls_provider_adapter_selection_ready
mls_provider_adapter_selection_provider_rejected
mls_provider_adapter_selection_pq_draft_rejected
mls_provider_adapter_selection_storage_rejected
mls_provider_adapter_selection_supply_chain_rejected
```

Backend command envelopes:

```text
run_mls_provider_adapter_selection_ready
run_mls_provider_adapter_selection_provider_rejected
run_mls_provider_adapter_selection_pq_draft_rejected
run_mls_provider_adapter_selection_storage_rejected
run_mls_provider_adapter_selection_supply_chain_rejected
```

## Research Basis

- RFC 9420 specifies the MLS protocol and its group key establishment model: https://www.rfc-editor.org/info/rfc9420/
- RFC 9750 describes the MLS architecture and emphasizes FS/PCS dependence on correct secret deletion: https://www.rfc-editor.org/info/rfc9750
- IETF `draft-ietf-mls-pq-ciphersuites-04` is current draft work for ML-KEM and hybrid MLS ciphersuites: https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/
- NIST FIPS 203 standardizes ML-KEM and its 512/768/1024 parameter sets: https://csrc.nist.gov/pubs/fips/203/final
- NIST FIPS 204 standardizes ML-DSA for PQ signatures: https://csrc.nist.gov/pubs/fips/204/final
- OpenMLS documents RFC 9420 implementation status, provider traits, libcrux provider work, storage-provider sensitivity, and debug features: https://github.com/openmls/openmls
- `mls-rs` documents RFC 9420 conformance, configurable storage/provider traits, provider choices, and its third-party audit status: https://docs.rs/mls-rs/latest/mls_rs/

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_provider_adapter_selection
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_adapter_selection_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_adapter_selection_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

After UI/platform choices identify the actual provider target, add a production provider integration behind this gate. The first implementation should keep the adapter disabled unless the selected dependency, feature flags, storage provider, release artifact, and provider evidence all satisfy this decision.
