# Mercury Roadmap

## Phase 0: Research And Design Packet

Status: in progress.

Deliverables:

- Repository initialized.
- Research synthesis.
- Threat model and architecture direction.
- AI participant model.
- Helix integration plan.
- Roadmap.

Exit gate:

- Planning packet exists in repo.
- Next code increment is small and well-scoped.

## Phase 1: Local Protocol Policy Prototype

Status: started.

Goal: prove Mercury's policy layer can be specified and tested before networking exists.

Deliverables:

- `helix/policy/envelope.hx`.
- Golden vectors for valid and rejected envelopes.
- Python script invoking `helixc.check`.
- Reason-code table.
- Initial Rust mirror plan, even if Rust code waits until Phase 2.

Exit gate:

- Helix policy checks pass. Done for the scalar envelope validator.
- Golden vectors are reviewed. Initial vector set exists.
- Downgrade, oversize, unknown critical flag, bad sequence, and bad epoch cases reject deterministically. Done in Helix test harness.
- Rust mirror and vector runner exist. Partial: Rust library and integration test exist; Python vector runner works; GitHub Actions is configured to run Rust tests on Linux; local Rust test execution is blocked by missing `link.exe`.

## Phase 2: Minimal Cryptographic Client Core

Goal: local-only encrypted message objects with mature crypto libraries.

Deliverables:

- Rust client-core crate.
- Canonical serialization.
- Local identity and device key generation.
- Envelope encryption/decryption through audited libraries.
- Test vectors for message objects.
- No network.

Exit gate:

- Unit tests and fuzz harnesses pass.
- No custom crypto primitives.
- Local plaintext does not enter logs.

## Phase 3: 1:1 Messaging Over A Relay

Goal: first real Mercury conversation between two devices.

Deliverables:

- Delivery relay with short-lived encrypted queues.
- 1:1 session setup using Signal-family library path.
- Device approval and safety verification.
- Desktop developer client.
- Minimal mobile prototype or mobile shell decision.

Exit gate:

- Server cannot read message content.
- Identity-key changes are visible.
- Delivery metadata TTL is documented and enforced.
- Basic abuse throttles exist without plaintext access.

## Phase 4: Mobile And Desktop First-Class Clients

Goal: Mercury works naturally away from the computer.

Deliverables:

- Mobile client path selected and implemented.
- Desktop client path selected and implemented.
- Encrypted local store.
- OS keychain/keystore integration.
- Linked-device approval and remote unlink.
- High-security mode toggles.

Exit gate:

- No notification previews in high-security mode.
- No plaintext crash analytics.
- Linked devices visible and revocable.
- App-lock and local database encryption tested.

## Phase 5: Groups, Attachments, And Backup

Goal: small groups become safe and usable.

Deliverables:

- MLS-based group messaging or explicitly transitional tiny-group fanout.
- Group chat readiness gate and checked fixtures. Done.
- High-security group readiness requires a PQ-hybrid MLS suite class. Done.
- Group message transcript gate binds MLS context and room-epoch ciphertext persistence. Done.
- Client-encrypted attachments.
- Encrypted media object store.
- Device-to-device transfer.
- Opt-in local encrypted backup.
- Cloud backup design with high-entropy recovery key, if pursued.

Exit gate:

- Group UI obeys backend group readiness capability flags.
- Membership changes are visible and cryptographically reflected.
- Removed devices cannot send accepted group messages.
- Attachments are encrypted before upload.
- Backups are not server-readable.

## Phase 6: AI Participant MVP

Status: started at the policy layer.

Goal: safe AI access without breaking the E2EE model.

Deliverables:

- Local AI draft/summarize mode for selected messages.
- AI grant object.
- Helix AI grant validator.
- AI audit event object.
- UI state showing when AI is active and what it can access.

Exit gate:

- AI cannot access a room without a grant. Policy scaffold started.
- AI cannot send without configured permission. Phase 1 grant policy rejects autonomous send.
- Remote provider mode is explicit and labeled. Phase 1 grant policy represents it and rejects it in sensitive/high-security rooms.
- Revoke path works. Pending.

## Phase 7: High-Assurance Hardening

Goal: earn stronger security claims.

Deliverables:

- Reproducible builds.
- Signed releases.
- SBOM and provenance.
- Fuzzing and property-test suite.
- Key transparency prototype.
- Independent security review plan.
- Formal model targets identified.

Exit gate:

- Release signing keys are separated from CI.
- Reproducible build instructions work on a clean machine.
- Security invariants map to tests or formal checks.
- Audit backlog is tracked.

## Phase 8: Post-Quantum And Metadata Resistance

Goal: strengthen against future cryptanalytic and metadata threats.

Deliverables:

- Hybrid PQ suite negotiation.
- ML-KEM test vectors.
- Downgrade-resistant suite validation.
- Sealed-sender-style delivery research prototype.
- Optional proxy/relay mode.
- Privacy-preserving contact discovery decision.

Exit gate:

- Hybrid suites cannot silently downgrade.
- Metadata retained by the server is measured and documented.
- Key transparency and sender-hiding designs are compatible.

## Phase 9: Helix Deepening

Goal: use Helix more heavily as compiler maturity allows.

Deliverables:

- Additional policy modules in Helix.
- WSL/Linux lane executing Helix-emitted tests.
- Rust/Helix differential tests.
- FFI pilot if compiler and linker path is ready.
- Formal proof-obligation archive in CI.

Exit gate:

- Helix artifacts catch real policy regressions.
- Production runtime still uses mature crypto libraries.
- Any increased Helix responsibility has a rollback plan.

## Immediate Next Step

Build Phase 1 only:

- Create the Helix envelope validator.
- Create three to five golden vectors.
- Add one PowerShell check script.
- Run the Python Helix compiler in check-only mode.

This is small enough to finish and verify, but foundational enough to shape the whole project.
