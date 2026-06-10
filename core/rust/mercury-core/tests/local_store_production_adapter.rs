use std::{collections::BTreeMap, convert::Infallible};

use mercury_core::{
    AcceptedLocalStoreProductionOpen, AcceptedLocalStoreWalReplay, AcceptedLocalStoreWrite,
    ComponentReasons, EncryptedLocalStoreAdapter, LocalStoreCrashRecoveryState, LocalStoreKeyScope,
    LocalStorePayload, LocalStoreProductionOpenInput, LocalStoreProductionOpenReason,
    LocalStoreReadRecord, LocalStoreRecordKind, LocalStoreRecordLocator, LocalStoreSealingSuite,
    LocalStoreUnlockDatabaseHeaderState, LocalStoreUnlockInput, LocalStoreUnlockSecretState,
    LocalStoreWriteRequest, MERCURY_LOCAL_STORE_VERSION, PolicyDecision,
    ProductionLocalStoreAdapter, open_production_local_store, put_production_local_store_record,
    read_production_local_store_record, replay_production_local_store_wal,
};

#[test]
fn production_adapter_opens_only_after_accepted_gate() {
    let mut store = RecordingProductionStore::default();

    let accepted = open_production_local_store(&mut store, valid_open_input())
        .expect("accepted open should not fail");
    assert!(accepted.accepted);
    assert_eq!(store.open_calls, 1);
    assert_eq!(
        store.last_open_reason,
        Some(LocalStoreProductionOpenReason::Accepted)
    );

    let mut blocked = valid_open_input();
    blocked.unlock.app_lock_satisfied = false;
    let blocked_decision =
        open_production_local_store(&mut store, blocked).expect("rejected open should not fail");

    assert!(!blocked_decision.accepted);
    assert_eq!(
        blocked_decision.reason,
        LocalStoreProductionOpenReason::UnlockRejected
    );
    assert_eq!(store.open_calls, 1);
}

#[test]
fn production_adapter_replays_wal_only_for_replay_decision() {
    let mut store = RecordingProductionStore::default();
    let mut replay_input = valid_open_input();
    replay_input.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;
    let replay_decision = replay_input.evaluate();

    let replayed = replay_production_local_store_wal(&mut store, replay_decision)
        .expect("WAL replay should not fail");
    assert!(replayed);
    assert_eq!(store.replay_calls, 1);
    assert_eq!(
        store.last_replay_reason,
        Some(LocalStoreProductionOpenReason::WalReplayRequired)
    );

    let accepted_decision = valid_open_input().evaluate();
    let skipped = replay_production_local_store_wal(&mut store, accepted_decision)
        .expect("non-replay decision should not fail");
    assert!(!skipped);
    assert_eq!(store.replay_calls, 1);
}

#[test]
fn production_adapter_reads_and_writes_only_policy_accepted_records() {
    let mut store = RecordingProductionStore::default();
    let locator = locator();

    let accepted = put_production_local_store_record(
        &mut store,
        LocalStoreWriteRequest::new(
            locator,
            LocalStoreRecordKind::MessageCiphertext,
            LocalStorePayload::sealed(b"sealed-message"),
            Some(policy_decision(true)),
        ),
    )
    .expect("accepted write should not fail");
    assert!(accepted.accepted);
    assert_eq!(store.put_calls, 1);

    let read = read_production_local_store_record(&store, locator)
        .expect("read should not fail")
        .expect("record should exist");
    assert_eq!(read.namespace, "conversation-7");
    assert_eq!(read.record_id, "message-42");
    assert_eq!(read.record_kind, LocalStoreRecordKind::MessageCiphertext);
    assert_eq!(read.bytes, b"sealed-message");

    let rejected = put_production_local_store_record(
        &mut store,
        LocalStoreWriteRequest::new(
            locator,
            LocalStoreRecordKind::MessagePlaintext,
            LocalStorePayload::public_metadata(b"plaintext"),
            Some(policy_decision(true)),
        ),
    )
    .expect("rejected write should not fail");
    assert!(!rejected.accepted);
    assert_eq!(store.put_calls, 1);
}

#[test]
fn prototype_store_satisfies_production_adapter_trait_shape() {
    let mut store = mercury_core::PrototypeEncryptedLocalStore::default();
    let open_decision = open_production_local_store(&mut store, valid_open_input())
        .expect("prototype adapter open should not fail");
    assert!(open_decision.accepted);

    let decision = put_production_local_store_record(
        &mut store,
        LocalStoreWriteRequest::new(
            locator(),
            LocalStoreRecordKind::ConversationSecret,
            LocalStorePayload::sealed(b"sealed-conversation-secret"),
            None,
        ),
    )
    .expect("prototype adapter write should not fail");
    assert!(decision.accepted);

    let record = read_production_local_store_record(&store, locator())
        .expect("prototype adapter read should not fail")
        .expect("record should exist");
    assert_eq!(record.record_kind, LocalStoreRecordKind::ConversationSecret);
    assert_eq!(record.bytes, b"sealed-conversation-secret");
}

#[derive(Default)]
struct RecordingProductionStore {
    open_calls: usize,
    replay_calls: usize,
    put_calls: usize,
    last_open_reason: Option<LocalStoreProductionOpenReason>,
    last_replay_reason: Option<LocalStoreProductionOpenReason>,
    records: BTreeMap<(String, String), LocalStoreReadRecord>,
}

impl EncryptedLocalStoreAdapter for RecordingProductionStore {
    type Error = Infallible;

    fn put_accepted_record(
        &mut self,
        write: AcceptedLocalStoreWrite<'_>,
    ) -> Result<(), Self::Error> {
        self.put_calls += 1;
        let request = write.request();
        let namespace = request.locator.namespace.to_owned();
        let record_id = request.locator.record_id.to_owned();
        self.records.insert(
            (namespace.clone(), record_id.clone()),
            LocalStoreReadRecord::new(
                namespace,
                record_id,
                request.record_kind,
                request.payload.kind(),
                request.payload.bytes(),
            ),
        );
        Ok(())
    }

    fn delete_record(&mut self, locator: LocalStoreRecordLocator<'_>) -> Result<(), Self::Error> {
        self.records
            .remove(&(locator.namespace.to_owned(), locator.record_id.to_owned()));
        Ok(())
    }
}

impl ProductionLocalStoreAdapter for RecordingProductionStore {
    fn open_database(&mut self, open: AcceptedLocalStoreProductionOpen) -> Result<(), Self::Error> {
        self.open_calls += 1;
        self.last_open_reason = Some(open.decision().reason);
        Ok(())
    }

    fn replay_wal(&mut self, replay: AcceptedLocalStoreWalReplay) -> Result<(), Self::Error> {
        self.replay_calls += 1;
        self.last_replay_reason = Some(replay.decision().reason);
        Ok(())
    }

    fn read_record(
        &self,
        locator: LocalStoreRecordLocator<'_>,
    ) -> Result<Option<LocalStoreReadRecord>, Self::Error> {
        Ok(self
            .records
            .get(&(locator.namespace.to_owned(), locator.record_id.to_owned()))
            .cloned())
    }
}

fn valid_open_input() -> LocalStoreProductionOpenInput {
    LocalStoreProductionOpenInput {
        unlock: LocalStoreUnlockInput {
            store_version: MERCURY_LOCAL_STORE_VERSION,
            keychain_available: true,
            device_secret: LocalStoreUnlockSecretState::PresentSealed,
            database_header: LocalStoreUnlockDatabaseHeaderState::Authenticated,
            app_lock_satisfied: true,
            recovery_required: false,
            plaintext_cache_records: 0,
        },
        header_magic_matches: true,
        header_suite_code: LocalStoreSealingSuite::MercuryLocalStoreV1.code(),
        header_nonce_len: LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        header_tag_len: LocalStoreSealingSuite::MercuryLocalStoreV1.authentication_tag_len(),
        required_key_slots: 1,
        sealed_key_slots: 1,
        plaintext_key_slots: 0,
        root_key_scope: LocalStoreKeyScope::DeviceLocal,
        root_key_generation: 1,
        crash_recovery: LocalStoreCrashRecoveryState::Clean,
    }
}

fn locator() -> LocalStoreRecordLocator<'static> {
    LocalStoreRecordLocator::new("conversation-7", "message-42")
}

fn policy_decision(accepted: bool) -> PolicyDecision {
    PolicyDecision {
        accepted,
        reason_code: if accepted { 0 } else { 1 },
        audit_class: 0,
        components: ComponentReasons {
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    }
}
