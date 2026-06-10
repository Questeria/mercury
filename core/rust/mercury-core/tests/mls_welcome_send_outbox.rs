use mercury_core::{
    MlsCommitAdmissionDecision, MlsCommitAdmissionReason, MlsKeyPackageConsumeStoreDecision,
    MlsKeyPackageConsumeStoreReason, MlsWelcomeSendOutboxReason, MlsWelcomeSendOutboxWrite,
    PrototypeMlsWelcomeSendOutbox,
};

static GROUP_ID: [u8; 32] = [0x81; 32];
static KEY_PACKAGE_HASH: [u8; 32] = [0x82; 32];
static OTHER_KEY_PACKAGE_HASH: [u8; 32] = [0x83; 32];
static ADDED_MEMBER_REF: [u8; 32] = [0x84; 32];
static WELCOME_SEND_TRANSACTION_DIGEST: [u8; 32] = [0x85; 32];
static OTHER_WELCOME_SEND_TRANSACTION_DIGEST: [u8; 32] = [0x86; 32];
static COMMIT_HASH: [u8; 32] = [0x87; 32];
static WELCOME_CIPHERTEXT_HASH: [u8; 32] = [0x88; 32];
static DELIVERY_ROUTE_ID: [u8; 32] = [0x89; 32];
static REPLAY_TOKEN: [u8; 32] = [0x8a; 32];
static SHORT_DIGEST: [u8; 16] = [0x8b; 16];

#[test]
fn accepted_outbox_persists_digest_only_welcome_send_record() {
    let mut outbox = PrototypeMlsWelcomeSendOutbox::default();

    let decision = outbox.put(valid_write());

    assert!(decision.accepted);
    assert_eq!(decision.reason, MlsWelcomeSendOutboxReason::Accepted);
    assert!(decision.persisted_record);
    assert_eq!(decision.record_count, 1);
    assert!(decision.can_enqueue_welcome_once);
    assert!(decision.can_send_welcome_after_commit);
    assert!(decision.consumes_key_package);
    assert!(decision.binds_welcome_send_transaction);
    assert!(decision.binds_commit);
    assert!(decision.binds_delivery_route);
    assert!(decision.prevents_duplicate_outbox);
    assert!(decision.keeps_digest_only);
    assert!(!decision.plaintext_bytes_exposed);

    let record = outbox
        .get(&WELCOME_SEND_TRANSACTION_DIGEST)
        .expect("accepted outbox record");
    assert_eq!(record.group_id, GROUP_ID);
    assert_eq!(record.key_package_hash, KEY_PACKAGE_HASH);
    assert_eq!(record.added_member_ref, ADDED_MEMBER_REF);
    assert_eq!(
        record.welcome_send_transaction_digest,
        WELCOME_SEND_TRANSACTION_DIGEST
    );
    assert_eq!(record.commit_hash, COMMIT_HASH);
    assert_eq!(record.welcome_ciphertext_hash, WELCOME_CIPHERTEXT_HASH);
    assert_eq!(record.delivery_route_id, DELIVERY_ROUTE_ID);
    assert_eq!(record.replay_token, REPLAY_TOKEN);
    assert_eq!(record.created_at_s, 1_100);
    assert_eq!(record.expires_at_s, 1_400);
    assert!(!record.plaintext_bytes_exposed);
}

#[test]
fn outbox_rejects_unaccepted_consumption_commit_bad_shapes_and_plaintext() {
    let rejected_consumption = MlsWelcomeSendOutboxWrite {
        key_package_consumption: rejected_key_package_consumption(),
        ..valid_write()
    };
    assert_rejected(
        rejected_consumption,
        MlsWelcomeSendOutboxReason::KeyPackageConsumeStoreRejected,
    );

    let rejected_commit = MlsWelcomeSendOutboxWrite {
        commit_admission: rejected_commit_admission(),
        ..valid_write()
    };
    assert_rejected(
        rejected_commit,
        MlsWelcomeSendOutboxReason::CommitAdmissionRejected,
    );

    let bad_group = MlsWelcomeSendOutboxWrite {
        group_id: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(bad_group, MlsWelcomeSendOutboxReason::BadGroupId);

    let bad_key_package = MlsWelcomeSendOutboxWrite {
        key_package_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        bad_key_package,
        MlsWelcomeSendOutboxReason::BadKeyPackageHash,
    );

    let bad_member_ref = MlsWelcomeSendOutboxWrite {
        added_member_ref: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        bad_member_ref,
        MlsWelcomeSendOutboxReason::BadAddedMemberRef,
    );

    let bad_transaction = MlsWelcomeSendOutboxWrite {
        welcome_send_transaction_digest: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        bad_transaction,
        MlsWelcomeSendOutboxReason::BadWelcomeSendTransactionDigest,
    );

    let bad_commit_hash = MlsWelcomeSendOutboxWrite {
        commit_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(bad_commit_hash, MlsWelcomeSendOutboxReason::BadCommitHash);

    let bad_welcome_hash = MlsWelcomeSendOutboxWrite {
        welcome_ciphertext_hash: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(
        bad_welcome_hash,
        MlsWelcomeSendOutboxReason::BadWelcomeCiphertextHash,
    );

    let bad_route = MlsWelcomeSendOutboxWrite {
        delivery_route_id: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(bad_route, MlsWelcomeSendOutboxReason::BadDeliveryRouteId);

    let bad_replay_token = MlsWelcomeSendOutboxWrite {
        replay_token: &SHORT_DIGEST,
        ..valid_write()
    };
    assert_rejected(bad_replay_token, MlsWelcomeSendOutboxReason::BadReplayToken);

    let bad_created_at = MlsWelcomeSendOutboxWrite {
        created_at_s: -1,
        ..valid_write()
    };
    assert_rejected(bad_created_at, MlsWelcomeSendOutboxReason::BadCreatedAt);

    let bad_expires_at = MlsWelcomeSendOutboxWrite {
        expires_at_s: 1_100,
        ..valid_write()
    };
    assert_rejected(bad_expires_at, MlsWelcomeSendOutboxReason::BadExpiresAt);

    let plaintext = MlsWelcomeSendOutboxWrite {
        plaintext_metadata_fields: 1,
        ..valid_write()
    };
    let decision = PrototypeMlsWelcomeSendOutbox::default().put(plaintext);
    assert_eq!(
        decision.reason,
        MlsWelcomeSendOutboxReason::PlaintextMetadataForbidden
    );
    assert!(decision.plaintext_bytes_exposed);
    assert!(!decision.can_enqueue_welcome_once);
}

#[test]
fn outbox_rejects_duplicate_transaction_and_key_package() {
    let mut outbox = PrototypeMlsWelcomeSendOutbox::default();
    assert!(outbox.put(valid_write()).accepted);

    let replay = outbox.put(valid_write());
    assert_eq!(
        replay.reason,
        MlsWelcomeSendOutboxReason::WelcomeSendTransactionAlreadyQueued
    );
    assert_eq!(replay.record_count, 1);
    assert!(!replay.accepted);

    let same_key_package_new_transaction = MlsWelcomeSendOutboxWrite {
        welcome_send_transaction_digest: &OTHER_WELCOME_SEND_TRANSACTION_DIGEST,
        ..valid_write()
    };
    let key_reuse = outbox.put(same_key_package_new_transaction);
    assert_eq!(
        key_reuse.reason,
        MlsWelcomeSendOutboxReason::KeyPackageAlreadyQueued
    );
    assert_eq!(key_reuse.record_count, 1);
    assert!(!key_reuse.accepted);

    let other_key_package = MlsWelcomeSendOutboxWrite {
        key_package_hash: &OTHER_KEY_PACKAGE_HASH,
        welcome_send_transaction_digest: &OTHER_WELCOME_SEND_TRANSACTION_DIGEST,
        ..valid_write()
    };
    assert!(outbox.put(other_key_package).accepted);
    assert_eq!(outbox.len(), 2);
}

#[test]
fn outbox_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (MlsWelcomeSendOutboxReason::Accepted, 0, "ACCEPTED"),
        (
            MlsWelcomeSendOutboxReason::KeyPackageConsumeStoreRejected,
            1,
            "KEY_PACKAGE_CONSUME_STORE_REJECTED",
        ),
        (
            MlsWelcomeSendOutboxReason::CommitAdmissionRejected,
            2,
            "COMMIT_ADMISSION_REJECTED",
        ),
        (MlsWelcomeSendOutboxReason::BadGroupId, 3, "BAD_GROUP_ID"),
        (
            MlsWelcomeSendOutboxReason::BadKeyPackageHash,
            4,
            "BAD_KEY_PACKAGE_HASH",
        ),
        (
            MlsWelcomeSendOutboxReason::BadAddedMemberRef,
            5,
            "BAD_ADDED_MEMBER_REF",
        ),
        (
            MlsWelcomeSendOutboxReason::BadWelcomeSendTransactionDigest,
            6,
            "BAD_WELCOME_SEND_TRANSACTION_DIGEST",
        ),
        (
            MlsWelcomeSendOutboxReason::BadCommitHash,
            7,
            "BAD_COMMIT_HASH",
        ),
        (
            MlsWelcomeSendOutboxReason::BadWelcomeCiphertextHash,
            8,
            "BAD_WELCOME_CIPHERTEXT_HASH",
        ),
        (
            MlsWelcomeSendOutboxReason::BadDeliveryRouteId,
            9,
            "BAD_DELIVERY_ROUTE_ID",
        ),
        (
            MlsWelcomeSendOutboxReason::BadReplayToken,
            10,
            "BAD_REPLAY_TOKEN",
        ),
        (
            MlsWelcomeSendOutboxReason::BadCreatedAt,
            11,
            "BAD_CREATED_AT",
        ),
        (
            MlsWelcomeSendOutboxReason::BadExpiresAt,
            12,
            "BAD_EXPIRES_AT",
        ),
        (
            MlsWelcomeSendOutboxReason::PlaintextMetadataForbidden,
            13,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            MlsWelcomeSendOutboxReason::WelcomeSendTransactionAlreadyQueued,
            14,
            "WELCOME_SEND_TRANSACTION_ALREADY_QUEUED",
        ),
        (
            MlsWelcomeSendOutboxReason::KeyPackageAlreadyQueued,
            15,
            "KEY_PACKAGE_ALREADY_QUEUED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_write() -> MlsWelcomeSendOutboxWrite<'static> {
    MlsWelcomeSendOutboxWrite {
        key_package_consumption: accepted_key_package_consumption(),
        commit_admission: accepted_commit_admission(),
        group_id: &GROUP_ID,
        key_package_hash: &KEY_PACKAGE_HASH,
        added_member_ref: &ADDED_MEMBER_REF,
        welcome_send_transaction_digest: &WELCOME_SEND_TRANSACTION_DIGEST,
        commit_hash: &COMMIT_HASH,
        welcome_ciphertext_hash: &WELCOME_CIPHERTEXT_HASH,
        delivery_route_id: &DELIVERY_ROUTE_ID,
        replay_token: &REPLAY_TOKEN,
        created_at_s: 1_100,
        expires_at_s: 1_400,
        plaintext_metadata_fields: 0,
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

fn accepted_commit_admission() -> MlsCommitAdmissionDecision {
    MlsCommitAdmissionDecision {
        accepted: true,
        reason: MlsCommitAdmissionReason::Accepted,
        can_apply_commit: true,
        can_initialize_epoch: true,
        can_continue_group: true,
        local_member_removed: false,
        requires_sync: false,
        requires_mls_setup: false,
        requires_tree_repair: false,
        requires_rekey: false,
        requires_user_action: false,
        prevents_commit_replay: true,
        forbids_plaintext_commit_metadata: true,
        plaintext_bytes_exposed: false,
    }
}

fn rejected_commit_admission() -> MlsCommitAdmissionDecision {
    MlsCommitAdmissionDecision {
        accepted: false,
        reason: MlsCommitAdmissionReason::CommitTieBreakRejected,
        can_apply_commit: false,
        can_initialize_epoch: false,
        can_continue_group: false,
        local_member_removed: false,
        requires_sync: true,
        requires_mls_setup: false,
        requires_tree_repair: false,
        requires_rekey: false,
        requires_user_action: false,
        prevents_commit_replay: true,
        forbids_plaintext_commit_metadata: true,
        plaintext_bytes_exposed: false,
    }
}

fn assert_rejected(write: MlsWelcomeSendOutboxWrite<'static>, reason: MlsWelcomeSendOutboxReason) {
    let decision = PrototypeMlsWelcomeSendOutbox::default().put(write);
    assert_eq!(decision.reason, reason);
    assert!(!decision.accepted);
    assert!(!decision.persisted_record);
    assert_eq!(decision.record_count, 0);
    assert!(!decision.can_enqueue_welcome_once);
    assert!(!decision.can_send_welcome_after_commit);
    assert!(decision.keeps_digest_only);
}
