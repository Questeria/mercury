# Security Research Cycle 08

Generated: 2026-05-28

## Scope

This cycle refreshed Mercury's group-chat provider posture against current primary sources for MLS post-quantum ciphersuites, PQ HPKE, and NIST PQC standards.

## Sources

- IETF draft, ML-KEM and Hybrid Cipher Suites for Messaging Layer Security: https://www.ietf.org/archive/id/draft-ietf-mls-pq-ciphersuites-02.html
- IETF draft, Post-Quantum and Post-Quantum/Traditional Hybrid Algorithms for HPKE: https://www.ietf.org/archive/id/draft-ietf-hpke-pq-04.html
- NIST FIPS 203, Module-Lattice-Based Key-Encapsulation Mechanism Standard: https://csrc.nist.gov/pubs/fips/203/final
- NIST PQC FIPS approval announcement: https://csrc.nist.gov/News/2024/postquantum-cryptography-fips-approved

## Finding

Mercury already required high-security groups to use an MLS PQ-hybrid suite class, but `mls_provider_configured = true` was too weak as a production boundary. A future adapter could accidentally mark a provider as configured without proving that the selected suite maps to ML-KEM, carries a hybrid KEM component where required, binds the suite id to group context, verifies downgrade evidence, passes KATs, zeroizes secrets, and forbids plaintext key export.

Current MLS PQ drafts also distinguish post-quantum confidentiality from post-quantum authentication. Mercury should keep high-security provider checks able to require PQ-signature readiness instead of treating every ML-KEM suite as full post-quantum security.

## Implemented Increment

Added `MlsProviderSecurityInput`, `MlsProviderSecurityDecision`, `MlsProviderSecurityReason`, and `evaluate_mls_provider_security(...)` in `mercury-core`.

`GroupChatInput` now carries an `MlsProviderSecurityDecision`. MLS group readiness rejects with `MLS_PROVIDER_SECURITY_REJECTED` unless the provider-security decision is accepted and matches the group suite.

Added focused tests for:

- accepted hybrid ML-KEM provider posture
- missing provider
- unsupported suite
- downgrade below suite floor
- weak or missing ML-KEM parameter set
- missing PQ/traditional hybrid component
- missing PQ signature readiness when required
- missing suite-context binding
- missing downgrade evidence
- missing KATs
- missing zeroization
- unsafe crypto backend
- plaintext key export
- stable reason labels/codes

Added checked fixture:

```text
fixtures/prototypes/group_chat_mls_provider_security_required.json
```

The simulator exposes it as:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype group_chat_mls_provider_security_required
```

## Limitations

This remains a policy/contract gate. It does not implement production MLS, ML-KEM, HPKE, ML-DSA, SLH-DSA, or provider KAT execution. The provider adapter must supply evidence to this gate once the production MLS implementation is selected.

## Next Questions

- Choose production MLS provider candidates and audit whether their APIs expose stable ciphersuite identifiers, KAT status, zeroization behavior, and transcript binding evidence.
- Decide whether high-security rooms should require PQ signatures everywhere, or only for new high-security groups after provider/library support stabilizes.
- Add provider-evidence records to local encrypted store once a production adapter exists.
