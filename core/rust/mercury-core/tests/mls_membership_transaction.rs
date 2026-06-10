use mercury_core::{
    MlsCommitReplayStoreDecision, MlsCommitReplayStoreReason, MlsKeyPackageConsumeStoreDecision,
    MlsKeyPackageConsumeStoreReason, MlsMembershipTransactionReason, MlsMembershipTransactionWrite,
    MlsWelcomeSendOutboxDecision, MlsWelcomeSendOutboxReason,
    PrototypeMlsMembershipTransactionStore, evaluate_mls_membership_transaction_write,
};

static GROUP_ID: [u8; 32] = [0x91; 32];
static OTHER_GROUP_ID: [u8; 32] = [0x92; 32];
static COMMIT_HASH: [u8; 32] = [0x93; 32];
static KEY_PACKAGE_HASH: [u8; 32] = [0x94; 32];
static WELCOME_SEND_TRANSACTION_DIGEST: [u8; 32] = [0x95; 32];
static MEMBERSHIP_TRANSACTION_DIGEST: [u8; 32] = [0x96; 32];
static OTHER_MEMBERSHIP_TRANSACTION_DIGEST: [u8; 32] = [0x97; 32];
static SHORT_DIGEST: [u8; 16] = [0x98; 16];

#[test]
fn accepted_membership_transaction_persists_digest_only_witness() {
    let mut store = PrototypeMlsMembershipTransactionStore::default();

    let decision = store.put(valid_write());

    assert!(decision.accepted);
    assert_eq!(decision.reason, MlsMembershipTransactionReason::Accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert!(decision.can_commit_membership_change_once);
    assert!(decision.can_advance_epoch);
    assert!(decision.can_send_welcome_from_outbox);
    assert!(decision.binds_commit_key_package_welcome);
    assert!(decision.uses_single_storage_transaction);
    assert!(decision.uses_serializable_isolation);
    assert!(decision.has_durable_commit);
    assert!(decision.enforces_unique_constraints);
    assert!(decision.has_idempotent_worker);
    assert!(decision.has_crash_recovery);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = store
        .get(&MEMBERSHIP_TRANSACTION_DIGEST)
        .expect("accepted transaction witness");
    assert_eq!(record.group_id, GROUP_ID);
    assert_eq!(record.commit_hash, COMMIT_HASH);
    assert_eq!(record.key_package_hash, KEY_PACKAGE_HASH);
    assert_eq!(
        record.welcome_send_transaction_digest,
        WELCOME_SEND_TRANSACTION_DIGEST
    );
    assert_eq!(
        record.membership_transaction_digest,
        MEMBERSHIP_TRANSACTION_DIGEST
    );
    assert_eq!(record.created_at_s, 1_100);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn transaction_rejects_failed_component_gates_bad_shapes_and_binding_mismatch() {
    let rejected_commit = MlsMembershipTransactionWrite {
        commit_replay: rejected_commit_replay(),
        ..valid_write()
    };
    assert_rejected(
        rejected_commit,
        MlsMembershipTransactionReason::CommitReplayStoreRejected,
    );

    let rejected_consumption = MlsMembershipTransactionWrite {
        key_package_consumption: rejected_key_package_consumption(),
        ..valid_write()
    };
    assert_rejected(
        rejected_consumption,
        MlsMembershipTransactionReason::KeyPackageConsumeStoreRejected,
    );

    let rejected_outbox = MlsMembershipTransactionWrite {
        welcome_send_outbox: rejected_welcome_send_outbox(),
        ..valid_write()
    };
    assert_rejected(
        rejected_outbox,
        MlsMembershipTransactionReason::WelcomeSendOutboxRejected,
    );

    let bad_group = MlsMembershipTransactionWrite {
        group_id: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(bad_group, MlsMembershipTransactionReason::BadGroupId);

    let bad_commit = MlsMembershipTransactionWrite {
        commit_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(bad_commit, MlsMembershipTransactionReason::BadCommitHash);

    let bad_key_package = MlsMembershipTransactionWrite {
        key_package_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        bad_key_package,
        MlsMembershipTransactionReason::BadKeyPackageHash,
    );

    let bad_welcome_transaction = MlsMembershipTransactionWrite {
        welcome_send_transaction_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        bad_welcome_transaction,
        MlsMembershipTransactionReason::BadWelcomeSendTransactionDigest,
    );

    let bad_transaction = MlsMembershipTransactionWrite {
        membership_transaction_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        bad_transaction,
        MlsMembershipTransactionReason::BadMembershipTransactionDigest,
    );

    let bad_created_at = MlsMembershipTransactionWrite {
        created_at_s: -1,
        ..valid_write()
    };
    assert_rejected(bad_created_at, MlsMembershipTransactionReason::BadCreatedAt);

    let mismatch = MlsMembershipTransactionWrite {
        outbox_group_id: &OTHER_GROUP_ID,
        ..valid_write()
    };
    assert_rejected(mismatch, MlsMembershipTransactionReason::BindingMismatch);
}

#[test]
fn transaction_rejects_weak_storage_worker_recovery_and_plaintext() {
    let non_atomic = MlsMembershipTransactionWrite {
        single_storage_transaction: false,
        ..valid_write()
    };
    assert_rejected(
        non_atomic,
        MlsMembershipTransactionReason::AtomicTransactionMissing,
    );

    let weak_isolation = MlsMembershipTransactionWrite {
        serializable_isolation: false,
        ..valid_write()
    };
    assert_rejected(
        weak_isolation,
        MlsMembershipTransactionReason::SerializableIsolationMissing,
    );

    let non_durable = MlsMembershipTransactionWrite {
        durable_commit: false,
        ..valid_write()
    };
    assert_rejected(
        non_durable,
        MlsMembershipTransactionReason::DurableCommitMissing,
    );

    let missing_unique = MlsMembershipTransactionWrite {
        unique_key_package_hash_constraint: false,
        ..valid_write()
    };
    assert_rejected(
        missing_unique,
        MlsMembershipTransactionReason::UniqueConstraintsMissing,
    );

    let non_idempotent = MlsMembershipTransactionWrite {
        outbox_worker_idempotent: false,
        ..valid_write()
    };
    assert_rejected(
        non_idempotent,
        MlsMembershipTransactionReason::IdempotentWorkerMissing,
    );

    let no_recovery = MlsMembershipTransactionWrite {
        crash_recovery_reconciles_pending_welcome: false,
        ..valid_write()
    };
    assert_rejected(
        no_recovery,
        MlsMembershipTransactionReason::CrashRecoveryMissing,
    );

    let plaintext = MlsMembershipTransactionWrite {
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let decision = evaluate_mls_membership_transaction_write(plaintext);
    assert_eq!(
        decision.reason,
        MlsMembershipTransactionReason::PlaintextMetadataForbidden
    );
    assert!(decision.plaintext_bytes_exposed);
    assert!(!decision.accepted);
}

#[test]
fn transaction_rejects_duplicate_transaction_witnesses() {
    let mut store = PrototypeMlsMembershipTransactionStore::default();
    assert!(store.put(valid_write()).accepted);

    let replay = store.put(valid_write());
    assert_eq!(
        replay.reason,
        MlsMembershipTransactionReason::TransactionAlreadyRecorded
    );
    assert_eq!(replay.record_count, 1);
    assert!(!replay.accepted);

    let other = MlsMembershipTransactionWrite {
        membership_transaction_digest: &OTHER_MEMBERSHIP_TRANSACTION_DIGEST,
        ..valid_write()
    };
    assert!(store.put(other).accepted);
    assert_eq!(store.len(), 2);
}

#[test]
fn transaction_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsMembershipTransactionReason::Accepted, 0, "ACCEPTED"),
        (
            MlsMembershipTransactionReason::CommitReplayStoreRejected,
            1,
            "COMMIT_REPLAY_STORE_REJECTED",
        ),
        (
            MlsMembershipTransactionReason::KeyPackageConsumeStoreRejected,
            2,
            "KEY_PACKAGE_CONSUME_STORE_REJECTED",
        ),
        (
            MlsMembershipTransactionReason::WelcomeSendOutboxRejected,
            3,
            "WELCOME_SEND_OUTBOX_REJECTED",
        ),
        (
            MlsMembershipTransactionReason::BadGroupId,
            4,
            "BAD_GROUP_ID",
        ),
        (
            MlsMembershipTransactionReason::BadCommitHash,
            5,
            "BAD_COMMIT_HASH",
        ),
        (
            MlsMembershipTransactionReason::BadKeyPackageHash,
            6,
            "BAD_KEY_PACKAGE_HASH",
        ),
        (
            MlsMembershipTransactionReason::BadWelcomeSendTransactionDigest,
            7,
            "BAD_WELCOME_SEND_TRANSACTION_DIGEST",
        ),
        (
            MlsMembershipTransactionReason::BadMembershipTransactionDigest,
            8,
            "BAD_MEMBERSHIP_TRANSACTION_DIGEST",
        ),
        (
            MlsMembershipTransactionReason::BadCreatedAt,
            9,
            "BAD_CREATED_AT",
        ),
        (
            MlsMembershipTransactionReason::BindingMismatch,
            10,
            "BINDING_MISMATCH",
        ),
        (
            MlsMembershipTransactionReason::AtomicTransactionMissing,
            11,
            "ATOMIC_TRANSACTION_MISSING",
        ),
        (
            MlsMembershipTransactionReason::SerializableIsolationMissing,
            12,
            "SERIALIZABLE_ISOLATION_MISSING",
        ),
        (
            MlsMembershipTransactionReason::DurableCommitMissing,
            13,
            "DURABLE_COMMIT_MISSING",
        ),
        (
            MlsMembershipTransactionReason::UniqueConstraintsMissing,
            14,
            "UNIQUE_CONSTRAINTS_MISSING",
        ),
        (
            MlsMembershipTransactionReason::IdempotentWorkerMissing,
            15,
            "IDEMPOTENT_WORKER_MISSING",
        ),
        (
            MlsMembershipTransactionReason::CrashRecoveryMissing,
            16,
            "CRASH_RECOVERY_MISSING",
        ),
        (
            MlsMembershipTransactionReason::PlaintextMetadataForbidden,
            17,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            MlsMembershipTransactionReason::TransactionAlreadyRecorded,
            18,
            "TRANSACTION_ALREADY_RECORDED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_write() -> MlsMembershipTransactionWrite<'static> {
    MlsMembershipTransactionWrite {
        commit_replay: accepted_commit_replay(),
        key_package_consumption: accepted_key_package_consumption(),
        welcome_send_outbox: accepted_welcome_send_outbox(),
        group_id: &GROUP_ID,
        commit_hash: &COMMIT_HASH,
        key_package_hash: &KEY_PACKAGE_HASH,
        welcome_send_transaction_digest: &WELCOME_SEND_TRANSACTION_DIGEST,
        membership_transaction_digest: &MEMBERSHIP_TRANSACTION_DIGEST,
        commit_replay_group_id: &GROUP_ID,
        commit_replay_commit_hash: &COMMIT_HASH,
        key_package_group_id: &GROUP_ID,
        key_package_hash_from_consumption: &KEY_PACKAGE_HASH,
        key_package_welcome_send_transaction_digest: &WELCOME_SEND_TRANSACTION_DIGEST,
        outbox_group_id: &GROUP_ID,
        outbox_key_package_hash: &KEY_PACKAGE_HASH,
        outbox_commit_hash: &COMMIT_HASH,
        outbox_welcome_send_transaction_digest: &WELCOME_SEND_TRANSACTION_DIGEST,
        created_at_s: 1_100,
        single_storage_transaction: true,
        serializable_isolation: true,
        durable_commit: true,
        unique_commit_hash_constraint: true,
        unique_key_package_hash_constraint: true,
        unique_welcome_transaction_constraint: true,
        outbox_worker_idempotent: true,
        crash_recovery_reconciles_pending_welcome: true,
        plaintext_metadata_fields: 0,
    }
}

fn accepted_commit_replay() -> MlsCommitReplayStoreDecision {
    MlsCommitReplayStoreDecision {
        accepted: true,
        reason: MlsCommitReplayStoreReason::Accepted,
        persisted_record: true,
        record_count: 1,
        can_apply_commit_once: true,
        can_continue_group: true,
        local_member_removed: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

fn rejected_commit_replay() -> MlsCommitReplayStoreDecision {
    MlsCommitReplayStoreDecision {
        accepted: false,
        reason: MlsCommitReplayStoreReason::CommitAlreadyRecorded,
        persisted_record: false,
        record_count: 1,
        can_apply_commit_once: false,
        can_continue_group: false,
        local_member_removed: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

fn accepted_key_package_consumption() -> MlsKeyPackageConsumeStoreDecision {
    MlsKeyPackageConsumeStoreDecision {
        accepted: true,
        reason: MlsKeyPackageConsumeStoreReason::Accepted,
        persisted_record: true,
        record_count: 1,
        can_consume_key_package_once: true,
        can_send_welcome_once: true,
        prevents_key_package_reuse: true,
        binds_added_member_ref: true,
        binds_welcome_send_transaction: true,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

fn rejected_key_package_consumption() -> MlsKeyPackageConsumeStoreDecision {
    MlsKeyPackageConsumeStoreDecision {
        accepted: false,
        reason: MlsKeyPackageConsumeStoreReason::KeyPackageAlreadyConsumed,
        persisted_record: false,
        record_count: 1,
        can_consume_key_package_once: false,
        can_send_welcome_once: false,
        prevents_key_package_reuse: true,
        binds_added_member_ref: false,
        binds_welcome_send_transaction: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

fn accepted_welcome_send_outbox() -> MlsWelcomeSendOutboxDecision {
    MlsWelcomeSendOutboxDecision {
        accepted: true,
        reason: MlsWelcomeSendOutboxReason::Accepted,
        persisted_record: true,
        record_count: 1,
        can_enqueue_welcome_once: true,
        can_send_welcome_after_commit: true,
        consumes_key_package: true,
        binds_welcome_send_transaction: true,
        binds_commit: true,
        binds_delivery_route: true,
        prevents_duplicate_outbox: true,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

fn rejected_welcome_send_outbox() -> MlsWelcomeSendOutboxDecision {
    MlsWelcomeSendOutboxDecision {
        accepted: false,
        reason: MlsWelcomeSendOutboxReason::WelcomeSendTransactionAlreadyQueued,
        persisted_record: false,
        record_count: 1,
        can_enqueue_welcome_once: false,
        can_send_welcome_after_commit: false,
        consumes_key_package: false,
        binds_welcome_send_transaction: false,
        binds_commit: false,
        binds_delivery_route: false,
        prevents_duplicate_outbox: true,
        keeps_digest_only: true,
        plaintext_bytes_exposed: false,
    }
}

fn assert_rejected(
    write: MlsMembershipTransactionWrite<'static>,
    reason: MlsMembershipTransactionReason,
) {
    let decision = evaluate_mls_membership_transaction_write(write);
    assert_eq!(decision.reason, reason);
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert_eq!(decision.record_count, 0);
    assert!(!decision.can_commit_membership_change_once);
    assert!(!decision.can_advance_epoch);
    assert!(!decision.can_send_welcome_from_outbox);
    assert!(decision.keeps_digest_only);
}
