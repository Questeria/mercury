# Backend Loop Stop For UI

Generated: 2026-05-28

## Status

The backend loop is stopped at the UI boundary. The current backend contract is ready for UI integration, and the repo-wide preflight passed after the latest backend changes.

Last verification command:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

Result:

```text
Mercury preflight: OK
```

## Latest Pushed Backend Increments

- Anonymous rate-limit nullifier gate for ARC-style bounded anonymous abuse control.
- Anonymous nullifier store boundary for accepted-only opaque nullifier persistence and replay rejection.
- Anonymous credential issuer-trust gate for transparency, freshness, revocation, and partitioning-metadata safety.
- Anonymous issuer witness/auditor gate for split-view, witness-quorum, audit-freshness, and operator-diversity checks.
- MLS provider-security gate for ML-KEM/hybrid suite mapping, downgrade evidence, KATs, zeroization, and plaintext key-export rejection.
- MLS provider evidence-store boundary for digest-only accepted provider validation evidence, duplicate rejection, and plaintext evidence rejection.
- MLS provider evidence-use gate for missing, expired, suite-mismatched, malformed, or plaintext-tainted evidence rejection at read time.
- MLS provider adapter-selection gate for concrete MLS library/backend/profile provenance, RFC 9420 conformance, pinned PQ draft mapping, standardized ML-KEM/ML-DSA evidence, safe provider storage, secret lifecycle, signed artifacts, SBOM, and CVE monitoring.
- MLS KeyPackage admission gate for group-bound, suite-bound, lifetime-current, replay-fresh, credential-valid, and plaintext-free membership adds.
- MLS KeyPackage consume-store boundary for sender-side one-time KeyPackage consumption, Welcome-send transaction binding, global duplicate rejection, and plaintext metadata rejection before Welcome sending.
- MLS Welcome send outbox boundary for digest-only queued Welcome records, accepted Commit binding, delivery-route binding, duplicate transaction rejection, duplicate KeyPackage queued rejection, and plaintext metadata rejection.
- MLS membership transaction witness for one durable, serializable storage transaction binding accepted Commit replay, KeyPackage consumption, Welcome outbox insertion, unique constraints, idempotent worker behavior, crash recovery, duplicate transaction rejection, and plaintext metadata rejection.
- Local store database security gate for SQLCipher-style encrypted page storage, per-page authentication, encrypted WAL/journals, memory-only temp files, safe backup policy, platform-keystore wrapping, key zeroization, and plaintext database rejection.
- Local store database adapter selection gate for SQLCipher/custom adapter provenance, platform package support, redistribution-safe licensing, FIPS runtime evidence, hardened SQLite settings, deterministic migration and crash drills, signed artifacts, SBOM, and CVE monitoring.
- Secure backup/restore gate for accepted recovery, high-entropy archive keys, KDF policy, server-backed rate limiting, opaque identifiers, plaintext export rejection, OS backup exclusion, MLS epoch binding, restore rekeying, tamper evidence, replay protection, audit digests, and bounded retention.
- Sealed audit event-chain gate for digest-only security event logging with hash-chain continuity, monotonic counters, event-context binding, signed checkpoints, Merkle proofs, transparency receipts, witness quorum, rollback-resistant storage, and forward-secret audit key rotation.
- Sealed audit event-store boundary for accepted-only audit persistence, duplicate sequence/hash/checkpoint rejection, rollback rejection, checkpoint and receipt binding, append-only guard enforcement, and plaintext metadata rejection.
- Sealed audit witness/checkpoint gate for store-accepted checkpoint publication, PQ/hybrid checkpoint signatures, C2SP-style checkpoint shape, consistency proof limits, witness quorum/operator diversity/key pins, timestamped checkpoint-bound cosignatures, private monitor queries, split-view evidence rejection, and authenticated local checkpoint recovery.
- Sealed audit witness-client gate for C2SP-style add-checkpoint operation, witness policy/key pinning, endpoint hardening, bounded consistency proof requests, witness conflict/unavailability handling, known cosignature quorum, atomic latest-checkpoint persistence, split-view alert routing, and private monitor queries.
- Sealed audit proof-bundle gate for offline audit verification, accepted witness-client binding, persisted digest-only proof cache entries, verifier policy snapshots, inclusion and consistency proof evidence, witness freshness, authenticated cache recovery, and selector-free UI status.
- Sealed audit proof-cache adapter for accepted-only, digest-only, encrypted, append-only proof persistence with offline verifier success, monitor freshness, policy snapshot binding, duplicate and rollback rejection, authenticated recovery, and selector-free metadata.
- Sealed audit verifier policy store for accepted-only signed policy snapshots, consistency-proof checks, key-rotation authentication, private monitor freshness, encrypted append-only scheduler state, split-view escalation routing, and selector-free metadata.
- Sealed audit incident evidence store for accepted-only split-view, missing-proof, and private-monitor incident evidence with verifier-policy binding, digest-only contradiction reports, blinded missing-proof reports, private monitor reports, witness/operator accountability, retry/backoff state, encrypted append-only records, and selector-free metadata.
- Sealed audit recovery export store for accepted-only cross-device audit recovery with incident-evidence binding, encrypted/authenticated export manifests, restore quorum, device binding, rollback protection, private sync, redacted selectors, audit-checkpoint binding, encrypted append-only storage, and digest-only UI status.
- Sealed audit database adapter and private report transport gates for recovery-export-approved encrypted storage, append-only schema constraints, migration/crash drills, OHTTP-style report routing, anonymous rate limits, replay guards, encrypted report outbox state, and selector-free payloads.
- Sealed audit private report outbox for accepted-only report submission persistence with transport binding, digest-only OHTTP request/response transcripts, encrypted payload and outbox state, replay-window binding, duplicate rejection, retry/backoff persistence, anonymous rate-limit token spend-once state, route privacy, and selector-free UI status.
- Sealed audit private report receipt store for accepted-only delivery completion with gateway receipt signatures, report/response/key binding, gateway-key transparency and consistency evidence, authenticated key rotation, private monitor proof, blinded failure classification, retry completion persistence, duplicate receipt rejection, delivery replay rejection, encrypted receipt records, and selector-free UI status.
- Sealed audit private report reconciliation store for accepted-only retry and delivery-state reconciliation with receipt binding, retry idempotency, duplicate retry rejection, anonymous rate-limit continuity, no retry after delivered receipt, crash-recovery cursor binding, false-delivery rejection, operator-accountability routing, blinded failure buckets, encrypted append-only records, and selector-free UI status.
- Sealed audit private report gateway-evidence store for accepted-only unavailable-gateway and operator-accountability evidence with reconciliation binding, gateway-authenticated unavailable evidence, signed relay observations, target absence proof, no client-asserted unavailability, retry exhaustion, rate-limit continuity, gateway key binding, blinded failure buckets, private monitor routing, policy-gated incident visibility, encrypted append-only records, and selector-free UI status.
- MLS Welcome admission gate for receiving-side group joins with verified group secrets, GroupInfo, ratchet-tree integrity, confirmation tags, winning Commit ordering, replay protection, and plaintext-free group metadata.
- MLS Welcome replay-store boundary for accepted Welcome persistence with consumed KeyPackage refs, init-key deletion, tree/transcript hash binding, transactional group-state commit evidence, duplicate rejection, and plaintext metadata rejection.
- MLS Commit admission gate for current-epoch Commit processing with sender/auth validation, proposal policy checks, update-path/tree validation, transcript confirmation, tie-break handling, replay protection, and plaintext-free Commit metadata.
- MLS Commit replay-store boundary for digest-only accepted Commit persistence, duplicate replay rejection, local-member removal terminal records, and plaintext metadata rejection.
- Anonymous group membership proof gate for PQ-safe high-security proof posture and route/epoch/nullifier binding.
- Group relay envelope gate for metadata-hidden group enqueue readiness.
- Backend command envelopes for group chat readiness, issuer trust, anonymous group proof, anonymous nullifier, and group relay envelope accepted and blocked states.
- Production AI connector gate and production media object index open gate.

## Ready For UI

- Stable platform bridge JSON for platform fixtures, prototype fixtures, and backend commands.
- C ABI bridge in `core/rust/mercury-ffi` with checked header `core/rust/mercury-ffi/include/mercury_ffi.h`.
- Account recovery gate and accepted-only recovery service adapter boundary.
- Account recovery prototype fixtures for ready, low-entropy rejection, threshold quorum, plaintext backup rejection, and high-security key rotation.
- Group chat readiness gate and prototype fixtures for MLS-ready, MLS setup required, membership sync required, plaintext metadata forbidden, high-security MLS-required, high-security PQ-required, and MLS provider-security rejected states.
- MLS provider evidence-store fixtures and commands for accepted digest-only validation evidence, provider-gate rejection, duplicate evidence rejection, and plaintext evidence rejection.
- MLS provider evidence-use fixtures and commands for current evidence readiness, missing record, expiry, suite mismatch, and plaintext-taint states.
- MLS provider adapter-selection fixtures and commands for accepted concrete provider selection, provider-security rejection, unpinned PQ draft rejection, unsafe storage rejection, and missing supply-chain evidence.
- Secure backup/restore fixtures and commands for accepted cloud archive readiness, rejected account recovery, plaintext export rejection, missing MLS group rekey, and unsafe OS backup policy.
- Sealed audit event-chain fixtures and commands for accepted witnessed audit readiness, plaintext event rejection, local rollback rejection, witness quorum rejection, and event-binding rejection.
- Sealed audit event-store fixtures and commands for accepted digest-only persistence, chain-gate rejection, duplicate sequence rejection, rollback rejection, checkpoint/receipt binding, and plaintext metadata rejection.
- Sealed audit witness/checkpoint fixtures and commands for accepted checkpoint publication, store rejection, witness quorum rejection, split-view evidence rejection, and monitor privacy rejection.
- Sealed audit witness-client fixtures and commands for accepted witness operation, witness conflict, witness unavailability, policy rejection, and monitor privacy rejection.
- Sealed audit proof-bundle fixtures and commands for accepted offline verification, rejected witness-client state, stale witness timestamps, verifier policy rejection, and privacy/selector leakage rejection.
- Sealed audit proof-cache fixtures and commands for accepted proof persistence, rejected proof bundles, duplicate proof rejection, stale policy snapshots, and plaintext metadata rejection.
- Sealed audit verifier policy fixtures and commands for accepted policy snapshots, expired policy, key rotation required, stale private monitor freshness, and plaintext metadata rejection.
- Sealed audit incident evidence fixtures and commands for accepted incident persistence, verifier-policy rejection, unblinded missing-proof reports, unverified split-view contradiction evidence, and plaintext metadata rejection.
- Sealed audit recovery export fixtures and commands for accepted recovery/export persistence, incident-evidence rejection, missing restore quorum, rollback rejection, and plaintext metadata rejection.
- Sealed audit database adapter and private report transport fixtures and commands for accepted encrypted storage/report routing, database encryption rejection, append-only rejection, and plaintext report transport rejection.
- Sealed audit private report outbox fixtures and commands for accepted report outbox persistence, transport rejection, replay rejection, anonymous rate-limit token rejection, and plaintext metadata rejection.
- Sealed audit private report receipt fixtures and commands for accepted delivery receipt persistence, outbox rejection, missing receipt, gateway transparency rejection, and plaintext metadata rejection.
- Sealed audit private report reconciliation fixtures and commands for accepted retry reconciliation, receipt rejection, retry/idempotency rejection, false-delivery rejection, and plaintext metadata rejection.
- Sealed audit private report gateway-evidence fixtures and commands for accepted unavailable-gateway evidence, reconciliation rejection, forged/unavailable evidence rejection, accountability-route rejection, and plaintext metadata rejection.
- MLS KeyPackage admission fixtures and commands for accepted add-member readiness, rejected group state, lifetime expiry, suite mismatch, invalid credential, replayed KeyPackage hash, and plaintext identity rejection.
- MLS KeyPackage consume-store fixtures and commands for accepted one-time consumption, admission rejection, duplicate KeyPackage hash rejection, malformed digest rejection, Welcome-send transaction binding, and plaintext metadata rejection.
- MLS Welcome send outbox fixtures and commands for accepted durable queueing, consume rejection, duplicate transaction rejection, duplicate KeyPackage queued rejection, malformed delivery-route rejection, accepted Commit binding, and plaintext metadata rejection.
- MLS membership transaction fixtures and commands for accepted one-transaction membership commit, binding mismatch rejection, missing atomic storage guarantee rejection, duplicate transaction marker rejection, idempotent worker and crash-recovery requirements, and plaintext metadata rejection.
- Local store database security fixtures and commands for accepted SQLCipher-style storage, plaintext database rejection, plaintext WAL/journal rejection, unsafe backup policy rejection, and secret lifecycle rejection before production stores host message keys or MLS records.
- Local store database adapter selection fixtures and commands for accepted SQLCipher adapter selection, license rejection, FIPS evidence rejection, missing migration drills, and missing supply-chain evidence before a production encrypted database adapter is treated as shippable.
- MLS Welcome admission fixtures and commands for accepted join readiness, missing encrypted group secrets, ratchet-tree rejection, confirmation-tag rejection, losing Commit tie-break rejection, replayed Welcome hash rejection, and plaintext group metadata rejection.
- MLS Welcome replay-store fixtures and commands for accepted Welcome persistence, admission rejection, duplicate Welcome hash rejection, malformed digest/state shape rejection, consumed KeyPackage/init-key binding, transactional group-state commit evidence, and plaintext metadata rejection.
- MLS Commit admission fixtures and commands for accepted epoch advancement, bad epoch rejection, authentication rejection, update-path/tree rejection, losing Commit tie-break rejection, replayed Commit hash rejection, and plaintext Commit metadata rejection.
- MLS Commit replay-store fixtures and commands for accepted digest-only Commit persistence, admission rejection, duplicate Commit hash rejection, terminal local-member removal, and plaintext metadata rejection.
- Group message transcript gate and checked prototype fixtures for accepted send, sync-required, rekey-required, and local-store epoch binding rejection states before persistence or relay submit.
- Anonymous credential issuer-trust gate and checked prototype fixtures requiring issuer key transparency, issuer-directory inclusion, bounded active issuer keys, freshness, revocation safety, challenge binding, and zero partitioning metadata before anonymous credential use.
- Anonymous issuer witness/auditor checks feed issuer trust and reject split-view, stale audit, missing quorum, missing operator diversity, and plaintext partitioning metadata.
- Anonymous group membership proof gate and checked prototype fixtures requiring accepted group readiness, PQ-safe proof posture for high-security rooms, challenge/nonce/header binding, group-epoch binding, route binding, replay nullifier freshness, proof freshness, and zero plaintext member identifiers before proof acceptance.
- Anonymous rate-limit nullifier gate and checked prototype fixtures requiring accepted membership proof, opaque nullifier storage, route/epoch binding, context binding, fresh windows, bounded presentation limits, and zero plaintext rate-limit metadata before relay enqueue.
- Anonymous nullifier store fixtures and commands for accepted persistence, duplicate replay rejection, and plaintext metadata rejection.
- Group relay envelope gate and checked prototype fixtures requiring accepted transcript state, accepted relay submission, sealed-sender delivery token, sealed sender certificate, accepted anonymous membership proof, and zero plaintext sender/group metadata before relay enqueue.
- Backend command envelopes for group chat readiness, issuer trust, anonymous group proof, anonymous nullifier, MLS provider evidence-store/use, MLS provider adapter-selection, secure backup/restore, sealed audit event-chain/store/witness-checkpoint/witness-client/proof-bundle/proof-cache/verifier-policy/incident-evidence/recovery-export/database-adapter/private-report-transport/private-report-outbox/private-report-receipt/private-report-reconciliation readiness, MLS KeyPackage admission, MLS KeyPackage consume-store, MLS Welcome send outbox, MLS membership transaction, MLS Welcome admission, MLS Welcome replay-store, MLS Commit admission, MLS Commit replay-store, and group relay envelope states, so UI/platform clients can inspect accepted and blocked group security states without bypassing command authorization.
- Production AI connector gate that keeps model calls draft-only, user-selected, authenticated, integrity-checked, and digest-only.
- Production media object index open gate plus indexed upload/download/cleanup session contracts.
- Full non-UI fixture and command list in `docs/44_NON_UI_BACKLOG_READY_FOR_UI.md`.

## Stop Reason

The remaining backend work is not blocked by failing tests; it is blocked by choices that should come from UI/platform/product integration or production deployment design:

- desktop/mobile platform package target for the C ABI bridge
- production keychain/keystore bindings
- production encrypted database adapter implementation behind the local-store database security and adapter-selection gates
- production recovery service transport, approval UX, and secure backup/archive builder behind the account-recovery and secure-backup gates
- production durable sealed audit database, checkpoint signing service, witness operation, monitor deployment, encrypted proof cache, verifier policy database, private monitor scheduler, incident evidence database, recovery-export database, private report outbox/submission transcript, private report delivery receipt, private report retry reconciliation, private report unavailable-gateway accountability evidence, and private incident report transport behind the sealed-audit event-chain/store/witness-checkpoint/witness-client/proof-bundle/proof-cache/verifier-policy/incident-evidence/recovery-export/database-adapter/private-report-transport/private-report-outbox/private-report-receipt/private-report-reconciliation boundaries
- production relay deployment and operations model
- production AI model execution adapter and local/remote runtime choice
- production media object service and CDN/storage integration
- production MLS provider integration and PQ ciphersuite mapping behind the checked group chat readiness, provider adapter-selection, provider evidence-store/use, KeyPackage admission, KeyPackage consume-store, Welcome send outbox, membership transaction, Welcome admission, Welcome replay-store, Commit admission, and Commit replay-store gates
- production group message transcript adapter and backend-command wiring
- production sealed-sender/group relay envelope adapter and anonymous credential provider
- production private-set/nullifier database and issuer consistency witness/auditor deployment

Backend should resume when the UI agent has picked a platform stack, a fixture/bridge gap appears, or one production service choice is ready to be implemented behind its checked adapter.
