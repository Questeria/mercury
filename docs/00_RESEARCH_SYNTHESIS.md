# Mercury Research Synthesis

Generated: 2026-05-27

## Executive Conclusion

Mercury should not try to beat existing secure messengers by inventing new cryptography. It should try to beat them by combining the best known secure-messaging primitives with stricter defaults, stronger metadata minimization, visible AI participation, reproducible builds, and a Helix-backed policy layer that makes security-critical state transitions easier to inspect and test.

The strongest product statement Mercury can honestly target now is:

> Mercury is designed to exceed mainstream messenger privacy and security for small groups by minimizing server knowledge, using reviewed end-to-end encryption protocols, hardening endpoints, and making AI access explicit and cryptographically scoped.

The stronger claim, "better than the best government systems," should be treated as an aspiration, not a launch claim. Mercury can pursue government-grade assurance patterns, but classified systems are evaluated under specific threat models, hardware constraints, operational procedures, and certification programs. Mercury earns stronger claims only through audits, formal evidence, reproducible builds, and operational maturity.

## Multi-Agent Findings

### Protocol Cryptography

Use a Signal/MLS direction, not a Telegram-style custom protocol direction.

- 1:1 messaging should follow Signal-family protocol design: PQXDH for asynchronous session setup and Double Ratchet or Triple Ratchet direction for message encryption.
- Group messaging should be MLS-first for real groups. Sender-key fanout can be considered only as a narrow MVP or broadcast optimization.
- Post-quantum support should be hybrid, not PQ-only. Use NIST-standard ML-KEM with classical X25519-style agreement and domain-separated HKDF.
- Key transparency is mandatory for a serious product, with manual safety-number verification as fallback.
- Mercury should not hand-roll ratchets, MLS, PQXDH, HPKE, AEAD, KDFs, password hashing, random generation, or key transparency proofs.

### Security And Operations

Mercury should be "minimum knowledge, maximum verifiability."

- The server should assume compromise and retain as little user data as possible.
- Metadata resistance must be a product requirement from the start, not a later privacy feature.
- Device compromise is the hard boundary for E2EE. Mercury can reduce blast radius but cannot promise secrecy from a compromised endpoint.
- Reproducible, signed releases are existential for a secure messenger. A subverted update defeats cryptography.
- Abuse controls must avoid hidden plaintext access. Use user reports, rate limits, recipient consent, invite controls, and client-side blocking.

### AI Participation

AI must be a visible participant or a scoped context recipient, never an invisible server feature.

- An AI has an account, devices, keys, membership state, grants, and audit events.
- AI joins rooms by invitation and receives only explicitly granted context.
- Users can revoke AI access, rotate group epochs where appropriate, and inspect why an AI saw or sent something.
- Local AI should be the default private mode. Remote enclave AI can be opt-in. Remote provider AI must be explicit context sharing.
- Prompt injection is assumed to remain a residual risk, so enforcement belongs outside the model in a policy engine.

### Helix Integration

Use Helix now where it is strongest: deterministic policy and verification.

- Good first Helix modules: envelope validators, replay policy, device lifecycle rules, group membership policy, audit classification, and provenance traces.
- Keep cryptographic primitives, ratchets, MLS tree crypto, randomness, secure deletion, and key material lifecycle in mature Rust/platform libraries initially.
- Use the Python-based Helix compiler as the current production compile/check path.
- Treat Linux/WSL execution of emitted binaries and FFI integration as staged milestones.

## Non-Negotiable Product Decisions

1. E2EE is default, not a secret-chat mode.
2. No mandatory phone-number identity.
3. No server-readable cloud history by default.
4. No hidden AI access to encrypted rooms.
5. No custom cryptographic primitives.
6. No plaintext analytics, crash logs, link previews, or server moderation pipeline.
7. No high-assurance public claim before reproducible builds and independent audit.

## Open Design Questions

- Should the first client shell be Flutter, React Native plus Tauri, native Swift/Kotlin plus Rust core, or another path?
- Should small-group MVP use pairwise fanout before MLS, or should Mercury absorb MLS complexity from the start?
- Should the server be Rust-only for consistency with crypto libraries, or Go/Rust split for operational speed?
- What is the first AI mode: local-only on desktop, remote provider opt-in, or local gateway from the user's existing AI chat?
- What identity model should ship first: random account IDs only, invite links, usernames, or optional phone/email aliases?

## Primary Sources

- Signal PQXDH: https://signal.org/docs/specifications/pqxdh/
- Signal Double Ratchet: https://signal.org/docs/specifications/doubleratchet/
- Signal Sesame: https://signal.org/docs/specifications/sesame/
- IETF MLS protocol, RFC 9420: https://www.rfc-editor.org/rfc/rfc9420.html
- MLS architecture, RFC 9750: https://www.rfc-editor.org/rfc/rfc9750.html
- NIST FIPS 203, ML-KEM: https://csrc.nist.gov/pubs/fips/203/final
- NIST FIPS 204, ML-DSA: https://csrc.nist.gov/pubs/fips/204/final
- NIST FIPS 205, SLH-DSA: https://csrc.nist.gov/pubs/fips/205/final
- HPKE, RFC 9180: https://www.rfc-editor.org/rfc/rfc9180.html
- OWASP MASVS: https://mas.owasp.org/MASVS/
- OWASP LLM Top 10: https://owasp.org/www-project-top-10-for-large-language-model-applications/
- NIST SSDF SP 800-218: https://csrc.nist.gov/pubs/sp/800/218/final
- NIST Zero Trust SP 800-207: https://www.nist.gov/publications/zero-trust-architecture-0
- The Update Framework: https://theupdateframework.io/spec/
- SLSA: https://slsa.dev/spec/latest/
- Reproducible Builds: https://reproducible-builds.org/docs
- NSA CSfC: https://www.nsa.gov/Resources/Commercial-Solutions-for-Classified-Program/Overview/

