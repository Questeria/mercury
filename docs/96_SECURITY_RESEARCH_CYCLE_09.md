# Security Research Cycle 09

Generated: 2026-05-28

## Scope

This cycle refreshed the MLS provider work against the latest MLS PQ draft state and NIST ML-KEM validation material, then added the missing persistence boundary for provider evidence.

## Sources

- IETF Datatracker, ML-KEM and Hybrid Cipher Suites for Messaging Layer Security: https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/
- NIST FIPS 203, Module-Lattice-Based Key-Encapsulation Mechanism Standard: https://csrc.nist.gov/pubs/fips/203/final
- NIST ACVP ML-KEM JSON Specification: https://pages.nist.gov/ACVP/draft-celi-acvp-ml-kem.html

## Finding

The current MLS PQ draft has advanced to `draft-ietf-mls-pq-ciphersuites-04`. It continues to separate PQ confidentiality from full PQ authentication, which supports Mercury's existing distinction between standard hybrid MLS posture and high-security PQ-signature readiness.

NIST FIPS 203 gives Mercury a stable ML-KEM baseline, and the NIST ACVP ML-KEM JSON specification provides a concrete validation-vector shape for future key generation and encapsulation/decapsulation evidence. However, ACVP vector validation alone is not sufficient for Mercury's provider acceptance boundary because zeroization and other implementation-lifecycle requirements are not fully testable by an ACVP server.

## Implemented Increment

Added an MLS provider evidence-store boundary in `mercury-core`.

The new store accepts only provider-security-approved records and persists digest-only evidence for:

- provider identity
- suite mapping
- ML-KEM KAT/vector validation
- downgrade checks
- zeroization checks

It rejects:

- unaccepted provider-security decisions
- malformed evidence ids or digests
- invalid validation windows
- plaintext evidence fields
- duplicate evidence ids

Added checked fixtures and backend command envelopes for ready, gate-rejected, duplicate-rejected, and plaintext-rejected branches.

## Limitations

This is still not a production MLS provider. It records the shape and storage policy for future evidence but does not generate the evidence itself. The next production step is selecting an MLS provider/library and binding real validation artifacts into this store.

## Next Questions

- Which production MLS provider exposes enough ciphersuite, transcript, and secret-lifecycle evidence to satisfy this store without patching upstream?
- Should Mercury require external ACVP/CAVP validation for all production ML-KEM providers, or allow development-only self-test vectors behind a non-production flag?
- What zeroization evidence format should be accepted: provider self-attestation, build-time sanitizer output, manual audit digest, or multiple independent digests?
