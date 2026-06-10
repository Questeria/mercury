# Mercury Security Architecture

## Security Goal

Mercury should protect message content, identity keys, device state, and conversation membership against network observers, compromised servers, routine cloud compromise, account takeover attempts, and supply-chain attacks as far as practical for a small-scale messenger.

Mercury should also be honest about the hard limits: no messaging app can cryptographically protect plaintext once it is displayed on a compromised endpoint, screen-captured by a participant, exported by a malicious client, or leaked by an invited AI/provider.

## Threat Model

Primary adversaries:

- Network observers on Wi-Fi, ISP, carrier, corporate, or hostile-country networks.
- Compromised delivery, media, authentication, or storage servers.
- Metadata collectors mapping who talks to whom, when, where, and how often.
- Account takeover via stolen devices, phishing, SIM swap, malicious linked devices, or recovery abuse.
- Device compromise via malware, spyware, screen capture, keyboard capture, OS backup extraction, or forensic access.
- Supply-chain compromise through dependencies, CI, build workers, signing keys, or update infrastructure.
- Insider or legal pressure against the Mercury operator.
- Abuse actors attempting spam, harassment, scams, or illegal content distribution.

Out of scope for cryptographic guarantees:

- A fully compromised endpoint while messages are visible.
- A malicious participant who is legitimately in a conversation.
- A user voluntarily exporting or screenshotting plaintext.
- Global passive traffic correlation without additional mixnet/proxy infrastructure.

## Security Invariants

- Private messages are never intentionally readable by Mercury servers.
- Message encryption is default for all private and group chats.
- Server-side storage contains encrypted envelopes, encrypted media blobs, delivery metadata with short TTLs, and minimum account state.
- The server cannot silently add a new user, device, or AI to a conversation without client-visible membership/key changes.
- Clients verify identity-key and device-key changes through key transparency and manual safety verification.
- Local plaintext is not written to analytics, crash logs, debug logs, notifications, link previews, or server-side moderation queues.
- Recovery and backup are opt-in and never server-readable by default.
- AI access is governed by explicit signed grants and can be revoked.

## Protocol Direction

### 1:1 Sessions

Use a Signal-family design:

- Session setup: PQXDH direction.
- Message encryption: Double Ratchet now, Triple Ratchet direction as mature implementations become available.
- Classical component: X25519/Ed25519 or XEdDSA-compatible identity model.
- Post-quantum component: ML-KEM hybrid key agreement.
- Multi-device session management: Sesame-style device/session tracking.

Implementation rule: use mature protocol libraries such as `libsignal` if licensing and integration constraints permit. Do not implement ratchets or PQXDH from scratch.

### Groups

Use MLS as the target group protocol.

- MLS provides epochs, membership changes, scalable group key agreement, and authenticated application messages.
- OpenMLS or `mls-rs` are candidate implementation paths.
- Pairwise fanout may be acceptable for the first tiny local prototype, but it must be marked transitional.

Implementation rule: do not implement the MLS tree, key schedule, or commit logic from scratch.

### Post-Quantum Direction

- Default PQ KEM target: ML-KEM-768.
- High-security mode candidate: ML-KEM-1024.
- Signatures: track ML-DSA and SLH-DSA for transparency logs, release signing, and future protocol signatures where appropriate.
- Use hybrid composition with classical primitives and domain-separated HKDF.
- Avoid downgrade paths. Protocol suite ids must be explicit and validated.

## Key Transparency

Mercury needs an append-only, auditable key directory:

- Account id to identity/device key mapping.
- Inclusion proofs for current keys.
- Consistency proofs for log history.
- Key-history proofs on safety checks.
- Witnesses or auditors so the server cannot self-certify everything.
- Privacy-preserving lookup design so the directory does not become a social graph.

Manual safety-number or QR verification remains necessary. Key transparency detects server equivocation and surprise key substitution; it does not prove a real-world identity by itself.

## Metadata Minimization

Default design:

- Random account ids rather than mandatory phone numbers.
- Invite links, QR codes, or out-of-band trust establishment.
- Optional aliases, not global public discoverability by default.
- No server-side group titles, avatars, profile names, or durable social graph.
- Short-lived encrypted delivery queues.
- Separate authentication, delivery, media, push, and abuse systems.
- Sealed-sender-style delivery research as a first-class roadmap item.

High-risk mode:

- No notification previews.
- No rich link previews.
- No automatic media download.
- Only contacts, mutual groups, or invite-approved senders.
- Optional proxy/relay support where practical.

## Local Device Security

Mobile:

- Use platform keystore or secure enclave where available.
- Encrypt local databases with app-level keys.
- Ask for contacts, media, camera, microphone, and notification permissions only when needed.
- Isolate risky attachment parsing and media rendering.
- Avoid plaintext OS backups unless explicitly enabled and clearly marked.
- Show linked devices and device additions prominently.

Desktop:

- Desktop is a separately approved device, not a weak mirror.
- Store secrets in OS keychain where possible.
- Support app passphrase and auto-lock.
- Encrypt local database.
- Sign updates.
- Disable remote code loading in webview shells.
- Use strict CSP and least-privilege IPC if Tauri/Electron is used.

## Backup And Recovery

Preferred modes:

- Device-to-device transfer by QR or short authenticated code.
- Local encrypted archive controlled by the user.
- Opt-in cloud backup encrypted by high-entropy recovery key.

Avoid:

- Server-readable cloud backups.
- Low-entropy PIN recovery without hardened online rate limiting, threshold design, or enclave-backed protection.
- Silent backup to OS/cloud locations that the user does not understand.

## Server Architecture

Server services should assume compromise:

- Delivery relay: short-lived encrypted queues.
- Media service: encrypted object blobs and short-lived references.
- Authentication: minimal account state and device registration.
- Key transparency: append-only log and auditor/witness interfaces.
- Push service: opaque notifications with no message text.
- Abuse service: rate limits, recipient consent, reports, and coarse behavior signals.

Operational controls:

- Internal mTLS/service identity.
- Default-deny network paths.
- HSM/KMS-backed server signing keys.
- Separate signing keys from CI.
- Immutable deploy artifacts.
- No long-lived cloud credentials.
- Minimal admin surface with break-glass logging.
- Auditable access logs that avoid sensitive user metadata.

## Abuse Handling Without Plaintext Access

Mercury should not build hidden plaintext moderation. Use:

- Client-side block, mute, report, and group-invite controls.
- User-initiated reports with selected plaintext voluntarily attached.
- Unknown-sender request queues.
- Rate limits based on account age, delivery tokens, recipient consent, invite graph, and coarse behavior.
- Spam throttles before delivery.
- Privacy-preserving abuse telemetry where feasible.

Policy statement: Mercury cannot inspect private messages unless a participant chooses to report selected content.

## Supply Chain And Release Security

Minimum bar before public security claims:

- Signed releases for every platform.
- Reproducible builds for clients and critical server components.
- Hermetic CI with pinned dependencies.
- SBOM and provenance attestations.
- Dependency review and vulnerability scanning.
- Threshold signing for releases.
- Emergency revocation and rollback protection.
- Public security advisories and patch process.

Frameworks to map against:

- NIST SSDF SP 800-218.
- OWASP MASVS for mobile.
- SLSA for build provenance.
- The Update Framework for update metadata.
- Reproducible Builds guidance.
- NIST SP 800-53, NIAP App PP, and NSA CSfC for high-assurance direction.

## Assurance Roadmap

Evidence Mercury should accumulate:

- Written threat model and security invariants.
- Protocol specification with test vectors.
- Fuzzing harnesses for parsers and envelope validation.
- Property tests for state machines.
- Formal models for critical protocol transitions where practical.
- Reproducible builds.
- Independent cryptography and application-security audits.
- Red-team exercises for AI grant enforcement, update compromise, linked-device abuse, and key transparency equivocation.

