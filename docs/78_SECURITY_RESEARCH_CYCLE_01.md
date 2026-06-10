# Security Research Cycle 01

Generated: 2026-05-28

## Scope

This cycle refreshed Mercury's group-encryption direction against current primary sources and converted the finding into one backend hardening increment.

## Primary Sources Checked

- IETF RFC 9420, The Messaging Layer Security (MLS) Protocol: https://www.rfc-editor.org/rfc/rfc9420.html
- IETF RFC 9750, The Messaging Layer Security (MLS) Architecture: https://www.rfc-editor.org/rfc/rfc9750.html
- IETF draft-ietf-mls-pq-ciphersuites, ML-KEM and Hybrid Cipher Suites for Messaging Layer Security: https://datatracker.ietf.org/doc/draft-ietf-mls-pq-ciphersuites/
- NIST FIPS 203, Module-Lattice-Based Key-Encapsulation Mechanism Standard: https://csrc.nist.gov/pubs/fips/203/final
- Signal PQXDH specification: https://signal.org/docs/specifications/pqxdh/

## Finding

Mercury already treated MLS as the production group-chat target, but the readiness gate only distinguished protocol readiness from transitional fanout. For high-security rooms, that left a future integration path where "MLS" could accidentally mean a classical-only MLS ciphersuite.

The research direction is clear enough to harden the policy now:

- MLS is the correct target for efficient asynchronous group security.
- PQ migration should use standardized ML-KEM or hybrid ML-KEM/classical designs rather than custom cryptography.
- High-security Mercury groups should fail closed unless the selected MLS provider can satisfy a high-security PQ-hybrid suite class.

## Implemented Increment

`GroupChatCryptoSuite` now models suite policy classes:

```text
classical_mls_128
hybrid_pq_mls_768
hybrid_pq_mls_1024
```

Standard MLS group fixtures default to `hybrid_pq_mls_768`. High-security group readiness now requires `hybrid_pq_mls_1024`; otherwise `evaluate_group_chat(...)` rejects with:

```text
HIGH_SECURITY_REQUIRES_PQ_HYBRID_SUITE
requires_pq_upgrade = true
```

The new `group_chat_high_security_pq_required` fixture exposes this state to UI and platform integration.

## Next Research Targets

- Map `hybrid_pq_mls_768` and `hybrid_pq_mls_1024` to the final MLS provider ciphersuite registry once the IETF MLS PQ draft stabilizes.
- Add a group send/receive transcript gate that binds group id, epoch, sender, generation, transcript context, and local-store room-epoch sealing before ciphertext persistence or relay submit.
- Review MLS provider candidates for memory zeroization, audited dependencies, deterministic test vectors, no unsafe default crypto, and PQ/hybrid roadmap.
