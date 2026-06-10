# Group Chat Support

Generated: 2026-05-28

## Status

Mercury now has a backend group-chat readiness gate in `mercury-core`:

```text
GroupChatProtocol
GroupChatCryptoSuite
GroupChatInput
GroupChatDecision
GroupChatReason
MlsProviderSecurityInput
MlsProviderSecurityDecision
MlsProviderSecurityReason
MlsProviderEvidenceStoreWrite
MlsProviderEvidenceStoreDecision
MlsProviderEvidenceStoreAdapter
MlsProviderEvidenceUseDecision
MlsProviderAdapterSelectionInput
MlsProviderAdapterSelectionDecision
MlsKeyPackageAdmissionInput
MlsKeyPackageAdmissionDecision
MlsKeyPackageAdmissionReason
MlsKeyPackageConsumeStoreWrite
MlsKeyPackageConsumeStoreDecision
MlsKeyPackageConsumeStoreAdapter
MlsMembershipTransactionWrite
MlsMembershipTransactionDecision
MlsMembershipTransactionAdapter
MlsWelcomeAdmissionInput
MlsWelcomeAdmissionDecision
MlsWelcomeAdmissionReason
MlsWelcomeReplayStoreWrite
MlsWelcomeReplayStoreDecision
MlsWelcomeReplayStoreAdapter
MlsCommitAdmissionInput
MlsCommitAdmissionDecision
MlsCommitAdmissionReason
MlsCommitReplayStoreWrite
MlsCommitReplayStoreDecision
MlsCommitReplayStoreAdapter
evaluate_group_chat(...)
evaluate_mls_provider_security(...)
put_mls_provider_evidence_record(...)
evaluate_mls_provider_evidence_use(...)
evaluate_mls_provider_adapter_selection(...)
evaluate_mls_key_package_admission(...)
put_mls_key_package_consumption_record(...)
put_mls_membership_transaction_record(...)
evaluate_mls_welcome_admission(...)
put_mls_welcome_replay_record(...)
evaluate_mls_commit_admission(...)
put_mls_commit_replay_record(...)
```

This is not the final MLS implementation. It is the security contract a future mobile, desktop, relay, or MLS provider integration must satisfy before the platform can open a group room, send a group message, or change membership.

## Accepted Group

The accepted path is intentionally narrow:

- group size must be at least 3 members
- group size must not exceed `MERCURY_MAX_GROUP_CHAT_MEMBERS` (128)
- local device must be a member
- every active member must have an active device
- room state must be available locally
- group secret must be sealed
- membership transition must not be pending
- current epoch and local epoch must match and be greater than zero
- key transparency must be ready
- plaintext member metadata fields must be zero
- MLS provider must be configured for MLS groups
- MLS provider security must be accepted for MLS groups
- high-security groups must use MLS
- high-security groups must use the `hybrid_pq_mls_1024` suite class

Accepted output enables:

```text
can_open_group = true
can_send_message = true
can_change_membership = true
```

Accepted output always keeps:

```text
forbids_server_plaintext = true
plaintext_bytes_exposed = false
```

## Protocol Posture

MLS is the production target for group chat. `TransitionalPairwiseFanout` exists only as an explicit small-scale standard/sensitive-room bridge while MLS provider integration is not complete. It is forbidden for high-security groups.

Do not treat transitional fanout as a final group protocol, and do not expose it as available in high-security rooms.

## Suite Posture

The current suite classes are:

```text
classical_mls_128
hybrid_pq_mls_768
hybrid_pq_mls_1024
```

`hybrid_pq_mls_768` is the default MLS target for standard group fixtures. `hybrid_pq_mls_1024` is required for high-security group readiness. The names are policy classes, not a homegrown cryptographic implementation; the production provider must map them to audited MLS/HPKE/PQ implementations.

`MlsProviderSecurityDecision` is the provider mapping gate. It requires the selected suite to be supported, not downgraded below the room floor, backed by the expected ML-KEM parameter set, PQ/traditional hybrid KEM posture for hybrid suite classes, suite-id binding to group context, downgrade evidence, known-answer tests, secret zeroization, no unsafe crypto backend flag, and zero plaintext key-export fields. High-security provider checks also require PQ-signature readiness.

`MlsProviderEvidenceStoreAdapter` is the digest-only persistence boundary for provider validation evidence. It accepts records only after provider security is accepted, stores provider/suite/KAT/downgrade/zeroization evidence digests, rejects duplicate evidence ids, and rejects plaintext evidence fields.

`MlsProviderEvidenceUseDecision` is the read-time freshness gate. It rejects missing evidence, expired evidence, future-dated evidence, suite mismatches, malformed evidence digests, and plaintext-tainted evidence before stored provider evidence can count as current readiness.

`MlsProviderAdapterSelectionDecision` is the concrete library/backend/profile gate. It rejects provider-security failures, custom or unknown MLS libraries, crypto backend/suite mismatches, protocol profile mismatches, non-distributable licenses, missing source provenance, missing RFC 9420 conformance evidence, unpinned MLS PQ draft mappings, missing ML-KEM/ML-DSA standard evidence, missing KAT/interop evidence, unsafe provider storage, weak secret lifecycle, missing downgrade/transcript-binding tests, unsafe debug features, plaintext key export, unsigned artifacts, and missing SBOM/CVE monitoring before a real provider can be linked or shipped.

`MlsKeyPackageAdmissionDecision` is the membership-add gate. It rejects unready group state, protocol/suite mismatch, invalid leaf/signature/credential/capability state, bad or stale KeyPackage lifetimes, unsupported extensions, init/encryption key reuse, malformed init/hash lengths, reused KeyPackage hashes, and plaintext identity fields before a future MLS provider can consume a KeyPackage or send a Welcome.

`MlsKeyPackageConsumeStoreAdapter` is the sender-side persistence boundary for one-time KeyPackage consumption. It persists accepted KeyPackage consumption only after KeyPackage admission accepts, binds the added member reference and Welcome-send transaction digest, rejects duplicate KeyPackage hashes globally, and rejects plaintext metadata fields before a future MLS provider can send a Welcome.

`MlsMembershipTransactionAdapter` is the sender-side transaction witness for add-member durability. It accepts only when Commit replay persistence, KeyPackage consumption, and Welcome send outbox insertion have all accepted and are cross-bound under one durable, serializable storage transaction with unique constraints, idempotent outbox worker behavior, crash recovery, and zero plaintext metadata.

`MlsWelcomeAdmissionDecision` is the receiving-side join gate. It rejects missing encrypted group secrets, suite mismatch, bad GroupInfo, missing PSKs, ratchet-tree integrity failures, local leaf mismatch, path/epoch/confirmation failures, losing Commit tie-break state, replayed Welcome hashes, and plaintext group metadata before a future MLS provider can initialize or open a newly joined group.

`MlsWelcomeReplayStoreAdapter` is the receiving-side persistence boundary for Welcome replay and KeyPackage consumption state. It persists accepted Welcome hashes only after Welcome admission accepts, binds the consumed KeyPackage ref, tree hash, confirmed transcript hash, init-key deletion, and transactional group-state commit evidence, rejects duplicate Welcome hashes, rejects reused KeyPackage refs, and rejects plaintext metadata fields.

`MlsCommitAdmissionDecision` is the epoch-advance gate. It rejects Commit messages unless the enclosing epoch matches the current group epoch, the sender/authentication state is valid, referenced proposals and application policy are valid, required update paths and ratchet-tree checks pass, transcript and confirmation evidence match, deterministic tie-break handling accepts the Commit, the Commit hash is fresh, and plaintext Commit metadata is absent.

`MlsCommitReplayStoreAdapter` is the digest-only persistence boundary for Commit replay state. It persists accepted Commit hashes only after Commit admission accepts, rejects duplicate Commit hashes per group, carries terminal local-member removal state, and rejects plaintext metadata fields.

## Rejection Classes

Stable rejection labels:

```text
NOT_ENOUGH_MEMBERS
MEMBER_LIMIT_EXCEEDED
LOCAL_DEVICE_NOT_MEMBER
ACTIVE_MEMBER_DEVICE_MISSING
ROOM_STATE_MISSING
GROUP_SECRET_MISSING
MEMBERSHIP_TRANSITION_PENDING
EPOCH_NOT_CURRENT
KEY_TRANSPARENCY_NOT_READY
MLS_PROVIDER_MISSING
HIGH_SECURITY_REQUIRES_MLS
PLAINTEXT_METADATA_FORBIDDEN
HIGH_SECURITY_REQUIRES_PQ_HYBRID_SUITE
MLS_PROVIDER_SECURITY_REJECTED
```

The decision separates:

- `requires_sync`
- `requires_mls_setup`
- `requires_pq_upgrade`
- `requires_user_action`

## Checked Fixtures

Prototype fixtures:

```text
group_chat_mls_ready
group_chat_mls_setup_required
group_chat_membership_sync_required
group_chat_plaintext_metadata_forbidden
group_chat_high_security_mls_required
group_chat_high_security_pq_required
group_chat_mls_provider_security_required
mls_provider_evidence_store_ready
mls_provider_evidence_store_gate_rejected
mls_provider_evidence_store_duplicate_rejected
mls_provider_evidence_store_plaintext_rejected
mls_provider_evidence_use_ready
mls_provider_evidence_use_missing
mls_provider_evidence_use_expired
mls_provider_evidence_use_suite_mismatch
mls_provider_evidence_use_plaintext_rejected
mls_provider_adapter_selection_ready
mls_provider_adapter_selection_provider_rejected
mls_provider_adapter_selection_pq_draft_rejected
mls_provider_adapter_selection_storage_rejected
mls_provider_adapter_selection_supply_chain_rejected
mls_key_package_admission_ready
mls_key_package_admission_group_rejected
mls_key_package_admission_lifetime_rejected
mls_key_package_admission_suite_mismatch
mls_key_package_admission_credential_rejected
mls_key_package_admission_replay_rejected
mls_key_package_admission_plaintext_rejected
mls_key_package_consume_store_ready
mls_key_package_consume_store_admission_rejected
mls_key_package_consume_store_duplicate_rejected
mls_key_package_consume_store_bad_shape
mls_key_package_consume_store_plaintext_rejected
mls_welcome_send_outbox_ready
mls_welcome_send_outbox_consume_rejected
mls_welcome_send_outbox_duplicate_transaction_rejected
mls_welcome_send_outbox_key_package_queued
mls_welcome_send_outbox_bad_shape
mls_welcome_send_outbox_plaintext_rejected
mls_membership_transaction_ready
mls_membership_transaction_binding_rejected
mls_membership_transaction_storage_rejected
mls_membership_transaction_duplicate_rejected
mls_membership_transaction_plaintext_rejected
mls_welcome_admission_ready
mls_welcome_admission_secrets_missing
mls_welcome_admission_tree_rejected
mls_welcome_admission_confirmation_rejected
mls_welcome_admission_tie_break_rejected
mls_welcome_admission_replay_rejected
mls_welcome_admission_plaintext_rejected
mls_welcome_replay_store_ready
mls_welcome_replay_store_admission_rejected
mls_welcome_replay_store_duplicate_rejected
mls_welcome_replay_store_key_package_reused
mls_welcome_replay_store_bad_shape
mls_welcome_replay_store_plaintext_rejected
mls_commit_admission_ready
mls_commit_admission_bad_epoch
mls_commit_admission_auth_rejected
mls_commit_admission_path_rejected
mls_commit_admission_tie_break_rejected
mls_commit_admission_replay_rejected
mls_commit_admission_plaintext_rejected
mls_commit_replay_store_ready
mls_commit_replay_store_admission_rejected
mls_commit_replay_store_duplicate_rejected
mls_commit_replay_store_local_member_removed
mls_commit_replay_store_plaintext_rejected
```

These fixtures expose accepted MLS-ready group state, missing MLS provider setup, membership sync/remediation, plaintext metadata rejection, high-security MLS enforcement, high-security PQ-suite enforcement, provider-security rejection, provider-evidence persistence checks, provider-evidence current-readiness checks, KeyPackage admission checks, KeyPackage consume-store checks, Welcome send outbox checks, Welcome admission checks, Welcome replay-store checks, Commit admission checks, and Commit replay-store checks through the simulator.
They also expose the membership transaction witness that binds accepted Commit replay, consumed KeyPackage, queued Welcome, storage atomicity, unique constraints, idempotent workers, and crash recovery before an add-member operation may be treated as committed.

## UI Contract

The UI must use `GroupChatDecision` capability booleans:

- open a group room only when `can_open_group = true`
- enable group send only when `can_send_message = true`
- enable add/remove member controls only when `can_change_membership = true`
- route `requires_sync = true` to sync or remediation
- route `requires_mls_setup = true` to setup/disabled state, not UI-side crypto fallback
- route `requires_pq_upgrade = true` to suite/provider upgrade, not a user override
- route `requires_user_action = true` to an explicit user action state
- treat `MLS_PROVIDER_SECURITY_REJECTED` as a backend provider hardening block, not a user override
- treat MLS provider evidence-store rejection as a backend/provider integration issue, not a user override
- treat MLS provider evidence-use rejection as requiring provider validation refresh or provider remediation, not a user override
- treat MLS KeyPackage admission rejection as a member-add hard stop; do not send a Welcome or retry with plaintext/member metadata
- treat MLS KeyPackage consume-store rejection as a sender-side replay, race, or persistence hard stop; do not send a Welcome unless KeyPackage admission and consume-store persistence both succeed
- treat MLS Welcome send outbox rejection as a sender-side durability hard stop; do not send a Welcome inline unless the backend has accepted the durable outbox record
- treat MLS membership transaction rejection as an add-member commit hard stop; do not advance epoch state, show the member as committed, or dispatch a Welcome unless the transaction witness accepts
- treat MLS Welcome admission rejection as a join/open hard stop; do not initialize local group state from rejected Welcome material
- treat MLS Welcome replay-store rejection as a duplicate/replay, consumed-KeyPackage, init-key-deletion, or transactional persistence hard stop; do not initialize a joined group unless accepted Welcome admission and replay-store persistence both succeed
- treat MLS Commit admission rejection as an epoch-advance hard stop; do not apply membership, tree, or send-state changes from rejected Commit material
- treat MLS Commit replay-store rejection as a duplicate/replay or persistence hard stop; do not apply a Commit twice or continue after failed replay persistence

Do not:

- infer group readiness from local member count or cached room data
- display server-side group names, avatars, or titles as trusted plaintext unless a future core/backend contract marks them as local encrypted metadata
- implement pairwise fanout for high-security group rooms
- retry membership changes while `membership_transition_pending` is true

## Verification

Run:

```powershell
cargo test -p mercury-core --test group_chat_readiness
cargo test -p mercury-core --test mls_provider_security
cargo test -p mercury-core --test mls_provider_evidence_store
cargo test -p mercury-core --test mls_key_package_admission
cargo test -p mercury-core --test mls_key_package_consume_store
cargo test -p mercury-core --test mls_welcome_send_outbox
cargo test -p mercury-core --test mls_membership_transaction
cargo test -p mercury-core --test mls_welcome_admission
cargo test -p mercury-core --test mls_welcome_replay_store
cargo test -p mercury-core --test mls_commit_admission
cargo test -p mercury-core --test mls_commit_replay_store
cargo test -p mercury-core --test group_message_transcript
cargo test -p mercury-core --test anonymous_credential_issuer_trust
cargo test -p mercury-core --test anonymous_issuer_witness_audit
cargo test -p mercury-core --test anonymous_group_membership_proof
cargo test -p mercury-core --test anonymous_rate_limit_nullifier
cargo test -p mercury-core --test anonymous_nullifier_store
cargo test -p mercury-core --test group_relay_envelope
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype group_chat_mls_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype group_chat_mls_provider_security_required
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_evidence_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_provider_evidence_use_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_key_package_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_key_package_consume_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_welcome_send_outbox_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_membership_transaction_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_welcome_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_welcome_replay_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_commit_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_commit_replay_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_credential_issuer_trust_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_group_membership_proof_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_rate_limit_nullifier_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype anonymous_nullifier_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype group_relay_envelope_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_credential_issuer_trust_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_group_membership_proof_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_rate_limit_nullifier_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_anonymous_nullifier_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_group_chat_mls_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_group_chat_mls_provider_security_required
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_provider_evidence_use_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_key_package_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_key_package_consume_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_send_outbox_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_membership_transaction_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_welcome_replay_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_commit_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_commit_replay_store_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_group_relay_envelope_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused readiness test covers accepted MLS-ready groups, member bounds, local membership, active devices, synced room state, sealed group secrets, pending transitions, epoch freshness, key transparency, plaintext metadata rejection, MLS provider setup, provider-security rejection, high-security MLS enforcement, transitional fanout constraints, and stable codes/labels.

The MLS provider-security test covers accepted hybrid ML-KEM provider posture, missing provider, unsupported suites, downgrade floors, ML-KEM parameter-set requirements, PQ/traditional hybrid requirements, high-security PQ-signature readiness, suite context binding, downgrade evidence, known-answer tests, secret zeroization, unsafe backend rejection, plaintext key-export rejection, and stable reason labels.

The MLS provider evidence-store test covers accepted digest-only evidence persistence, provider-security rejection propagation, evidence digest shape validation, validation-window checks, plaintext evidence rejection, duplicate evidence rejection, and stable reason labels.

The MLS provider evidence-use test covers fresh accepted evidence, missing evidence, provider-security rejection propagation, expired evidence, future-dated evidence, suite mismatch, malformed evidence, plaintext-tainted evidence, and stable reason labels.

The MLS KeyPackage admission test covers accepted admission, rejected group state, protocol/suite mismatch, bad lifetime windows, too-long and stale lifetimes, invalid leaf/signature/credential/capability states, unsupported extensions, init/encryption key reuse, bad key/hash shape, replayed KeyPackage hashes, plaintext identity rejection, and stable reason labels.

The MLS KeyPackage consume-store test covers accepted digest-only KeyPackage consumption persistence, rejected KeyPackage admission propagation, group id, KeyPackage hash, added-member ref, Welcome-send transaction digest, consumption-time shape, duplicate KeyPackage hash rejection across groups, plaintext metadata rejection, and stable reason labels.

The MLS Welcome send outbox test covers accepted digest-only Welcome queueing, rejected KeyPackage consumption propagation, rejected Commit admission propagation, group id, KeyPackage hash, added-member ref, Welcome-send transaction digest, Commit hash, Welcome ciphertext hash, route id, replay token, timestamp shape, duplicate transaction rejection, duplicate KeyPackage queue rejection, plaintext metadata rejection, and stable reason labels.

The MLS membership transaction test covers accepted digest-only transaction witnesses, rejected component gates, group/Commit/KeyPackage/Welcome binding mismatch, bad digest and timestamp shape, missing atomic storage, weak isolation, missing durability, missing unique constraints, non-idempotent outbox workers, missing crash recovery, duplicate transaction markers, plaintext metadata rejection, and stable reason labels.

The MLS Welcome admission test covers accepted Welcome processing, rejected KeyPackage admission, missing encrypted group secrets, suite mismatch, decrypt/PSK/GroupInfo failures, group-id reuse, ratchet-tree integrity failures, local leaf mismatch, path/epoch/confirmation failures, losing Commit tie-breaks, bad/replayed Welcome hashes, plaintext group metadata rejection, and stable reason labels.

The MLS Welcome replay-store test covers accepted Welcome persistence, rejected Welcome admission propagation, group id, Welcome hash, consumed KeyPackage ref, tree hash, confirmed transcript hash, group-state commit digest, epoch/joined-time shape, init-key deletion, transactional group-state commit, duplicate Welcome hash rejection, consumed KeyPackage rejection, plaintext metadata rejection, and stable reason labels.

The MLS Commit admission test covers accepted current-epoch Commit processing, group rejection propagation, bad epoch, sender/auth rejection, proposal-list and application-policy rejection, update-path/tree failures, transcript/confirmation failures, losing Commit tie-breaks, malformed/replayed Commit hashes, terminal local-member removal, plaintext Commit metadata rejection, and stable reason labels.

The MLS Commit replay-store test covers accepted digest-only Commit persistence, rejected Commit admission propagation, group id and Commit hash shape, epoch/applied-time shape, duplicate Commit hash rejection per group, terminal local-member removal persistence, plaintext metadata rejection, and stable reason labels.

The group message transcript test covers accepted MLS application-message context, accepted group/send gates, group id, epoch freshness, sender leaf index, sender generation, transcript context digests, sealed sender data, sealed application payload, four-byte reuse guard, local room-epoch store binding, and used-generation deletion.

The anonymous credential issuer-trust test covers issuer transparency, witness/auditor rejection, directory inclusion, key-id shape, freshness, active-key partitioning risk, revocation, challenge binding, partitioning metadata rejection, and stable reason labels.

The anonymous issuer witness-audit test covers accepted quorum and operator diversity, transparency rejection, signed-tree-head shape, tree rollback, missing quorum, stale audit, split-view reports, auditor signature requirements, plaintext partitioning metadata rejection, and stable reason labels.

The anonymous group membership proof test covers bound anonymous proof acceptance, group rejection propagation, issuer-trust rejection propagation, high-security PQ-proof enforcement, challenge/nonce/proof shape, presentation-header binding, group-epoch binding, route binding, replay nullifier handling, proof freshness, plaintext member identity rejection, and stable reason/scheme labels.

The anonymous rate-limit nullifier test covers accepted ARC-style windows, membership proof rejection propagation, nullifier shape, replayed/spent nullifier rejection, nullifier store safety, route/epoch binding, context binding, window validity, presentation limits, one-time credential single-use enforcement, plaintext rate-limit metadata rejection, and stable reason/kind labels.

The anonymous nullifier store test covers accepted-only opaque persistence, duplicate/replay rejection, digest shape validation, presentation-window exhaustion, plaintext metadata rejection, and stable reason labels.

The group relay envelope test covers accepted metadata-hidden group relay submit, transcript rejection propagation, relay submission rejection, sealed-sender delivery token requirements, anonymous membership proof requirements, anonymous rate-limit requirements, sealed envelope presence, plaintext sender metadata rejection, and plaintext group metadata rejection.

Backend command envelopes now expose group chat readiness, issuer trust, anonymous group proof, anonymous rate-limit nullifier, nullifier store, MLS provider evidence-store/use, MLS provider adapter-selection, secure backup/restore, MLS KeyPackage admission, MLS KeyPackage consume-store, MLS Welcome send outbox, MLS membership transaction, MLS Welcome admission, MLS Welcome replay-store, MLS Commit admission, MLS Commit replay-store, and group relay envelope accepted and blocked states through `run_group_chat_*`, `run_anonymous_credential_issuer_trust_*`, `run_anonymous_group_membership_proof_*`, `run_anonymous_rate_limit_nullifier_*`, `run_anonymous_nullifier_store_*`, `run_mls_provider_evidence_store_*`, `run_mls_provider_evidence_use_*`, `run_mls_provider_adapter_selection_*`, `run_secure_backup_restore_*`, `run_mls_key_package_admission_*`, `run_mls_key_package_consume_store_*`, `run_mls_welcome_send_outbox_*`, `run_mls_membership_transaction_*`, `run_mls_welcome_admission_*`, `run_mls_welcome_replay_store_*`, `run_mls_commit_admission_*`, `run_mls_commit_replay_store_*`, and `run_group_relay_envelope_*`.

## Next Backend Step

Select and integrate a production MLS provider and encrypted storage adapter behind the checked local-store database security, local-store database adapter selection, group chat readiness, provider adapter-selection, provider evidence-store/use, secure backup/restore, KeyPackage admission, KeyPackage consume-store, Welcome send outbox, membership transaction, Welcome admission, Welcome replay-store, Commit admission, and Commit replay-store gates, map suite classes to audited MLS/PQ ciphersuites, then connect the group message transcript and relay envelope gates to production send/receive adapters.
