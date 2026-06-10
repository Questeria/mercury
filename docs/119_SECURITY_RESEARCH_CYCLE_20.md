# Security Research Cycle 20: MLS Provider Adapter Provenance

Generated: 2026-05-28

## Sources Reviewed

- RFC 9420, The Messaging Layer Security Protocol: <https://www.rfc-editor.org/info/rfc9420/>
- RFC 9750, The Messaging Layer Security Architecture: <https://www.rfc-editor.org/info/rfc9750>
- IETF MLS PQ ciphersuite draft, `draft-ietf-mls-pq-ciphersuites-04`: <https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/>
- NIST FIPS 203, ML-KEM: <https://csrc.nist.gov/pubs/fips/203/final>
- NIST FIPS 204, ML-DSA: <https://csrc.nist.gov/pubs/fips/204/final>
- OpenMLS repository and documentation entry points: <https://github.com/openmls/openmls>
- OpenMLS v0.6 release notes for libcrux/high-assurance PQ provider and storage API changes: <https://blog.openmls.tech/posts/2024-09-04-v0_6-release/>
- OpenMLS storage-provider persistence notes: <https://book.openmls.tech/user_manual/persistence.html>
- `mls-rs` crate documentation: <https://docs.rs/mls-rs/latest/mls_rs/>

## Finding

The next security risk after provider-security evidence is provider selection itself. A future Mercury adapter could pass abstract provider-security inputs but still be unsafe to ship if the linked library or crypto backend lacks source provenance, RFC 9420 conformance, PQ draft pinning, standardized ML-KEM/ML-DSA posture, KAT/interop evidence, safe group-state storage, secret deletion guarantees, release signing, SBOM, or CVE monitoring.

The research supports a strict adapter-selection gate:

- RFC 9420 is the stable protocol baseline for MLS group key establishment, authentication, Commit/Welcome semantics, FS, and PCS.
- RFC 9750 makes secret deletion and local state handling an architectural security property, not just an implementation detail.
- MLS PQ ciphersuites remain active draft work as of `draft-ietf-mls-pq-ciphersuites-04`, so Mercury must pin the exact draft profile before treating hybrid PQ suites as shippable.
- FIPS 203 standardizes ML-KEM, and FIPS 204 standardizes ML-DSA; Mercury should not accept generic "Kyber/Dilithium-like" claims without mapping to these standards.
- OpenMLS and `mls-rs` both expose provider/storage abstraction surfaces, which is useful for Mercury, but also means Mercury must independently police provider choice, feature flags, storage implementation, and release provenance.
- OpenMLS has explicit debug features that can expose sensitive content or key material; Mercury must reject unsafe/debug feature selections.
- `mls-rs` documents RFC conformance but no full third-party security audit, so Mercury should require its own KAT, interop, and supply-chain evidence regardless of library choice.

## Increment

Added an MLS provider adapter-selection gate that:

- requires accepted `MlsProviderSecurityDecision`
- rejects custom or unknown MLS adapters
- rejects crypto backends that do not match the selected suite class
- rejects protocol profile mismatches and experimental/unknown profiles
- rejects unknown or non-distributable implementation licenses
- requires source authenticity and RFC 9420 conformance tests
- requires PQ draft version pinning, standardized ML-KEM, and high-security PQ-signature standard evidence where applicable
- requires KATs and interop tests
- requires sealed transactional provider storage
- requires audited zeroization and memory hardening
- requires downgrade and transcript binding tests
- rejects unsafe features and plaintext export
- requires signed artifacts, SBOM, and CVE monitoring
- exposes checked fixtures and backend commands for accepted, provider-rejected, PQ-draft-rejected, storage-rejected, and supply-chain-rejected states

## Security Impact

Mercury now has a hard gate between "MLS provider behavior appears safe" and "this concrete library/backend/profile may be linked and shipped." That blocks accidental production use of an experimental provider, unpinned PQ draft mapping, unsafe debug build, non-transactional storage provider, or unverifiable supply chain.

## Verification

Focused checks:

```powershell
cargo test -p mercury-core --test mls_provider_adapter_selection
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo test -p mercury-bindings --test platform_bridge
```

Simulator checks:

```powershell
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_adapter_selection_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_adapter_selection_ready
```

## Next Research Target

Study sealed audit logs, tamper-evident local event chains, and transparency logs for security-critical Mercury state transitions. The next backend increment should make device enrollment, recovery, backup creation, group membership changes, and AI authorization leave digest-only audit evidence without storing plaintext metadata.
