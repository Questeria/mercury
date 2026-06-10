use std::convert::Infallible;

use mercury_core::{
    AcceptedMediaObjectIndexProductionOpen, AcceptedMediaObjectIndexStoreWrite,
    AcceptedMediaObjectIndexWalReplay, LocalStoreCrashRecoveryState, LocalStoreSealingSuite,
    MERCURY_MEDIA_OBJECT_INDEX_VERSION, MediaObjectIndexProductionOpenDecision,
    MediaObjectIndexProductionOpenInput, MediaObjectIndexProductionOpenReason,
    MediaObjectIndexRecord, MediaObjectIndexStoreAdapter, ProductionMediaObjectIndexStoreAdapter,
    open_production_media_object_index, replay_production_media_object_index_wal,
};

#[test]
fn production_open_accepts_clean_ready_manifest() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_open_index);
    assert!(!decision.can_replay_wal);
    assert!(decision.can_load_manifests);
    assert!(decision.can_write_manifests);
    assert!(decision.can_use_remote_objects);
    assert!(!decision.requires_network_setup);
    assert!(!decision.requires_migration);
    assert!(!decision.requires_crash_recovery);
    assert!(!decision.requires_destructive_repair);
    assert_eq!(
        decision.reason,
        MediaObjectIndexProductionOpenReason::Accepted
    );
    assert_eq!(decision.reason.code(), 0);
    assert_eq!(decision.reason.label(), "ACCEPTED");
}

#[test]
fn production_open_rejects_bad_header_shape() {
    let mut bad_version = valid_input();
    bad_version.index_version = MERCURY_MEDIA_OBJECT_INDEX_VERSION + 1;
    let bad_version_decision = bad_version.evaluate();
    assert_rejected(
        bad_version_decision,
        MediaObjectIndexProductionOpenReason::UnsupportedIndexVersion,
    );
    assert!(bad_version_decision.requires_migration);

    let mut bad_magic = valid_input();
    bad_magic.header_magic_matches = false;
    let bad_magic_decision = bad_magic.evaluate();
    assert_rejected(
        bad_magic_decision,
        MediaObjectIndexProductionOpenReason::HeaderMagicMismatch,
    );
    assert!(bad_magic_decision.requires_destructive_repair);

    let mut bad_suite = valid_input();
    bad_suite.header_suite_code = 999;
    let bad_suite_decision = bad_suite.evaluate();
    assert_rejected(
        bad_suite_decision,
        MediaObjectIndexProductionOpenReason::HeaderSuiteMismatch,
    );
    assert!(bad_suite_decision.requires_migration);

    let mut bad_nonce = valid_input();
    bad_nonce.header_nonce_len = LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len() - 1;
    let bad_nonce_decision = bad_nonce.evaluate();
    assert_rejected(
        bad_nonce_decision,
        MediaObjectIndexProductionOpenReason::BadHeaderNonceLength,
    );
    assert!(bad_nonce_decision.requires_destructive_repair);

    let mut bad_tag = valid_input();
    bad_tag.header_tag_len =
        LocalStoreSealingSuite::MercuryLocalStoreV1.authentication_tag_len() - 1;
    let bad_tag_decision = bad_tag.evaluate();
    assert_rejected(
        bad_tag_decision,
        MediaObjectIndexProductionOpenReason::BadHeaderTagLength,
    );
    assert!(bad_tag_decision.requires_destructive_repair);
}

#[test]
fn production_open_rejects_plaintext_metadata_and_cache_paths() {
    let mut plaintext_metadata = valid_input();
    plaintext_metadata.plaintext_metadata_rows = 1;
    let plaintext_metadata_decision = plaintext_metadata.evaluate();
    assert_rejected(
        plaintext_metadata_decision,
        MediaObjectIndexProductionOpenReason::PlaintextMetadataForbidden,
    );
    assert!(plaintext_metadata_decision.requires_destructive_repair);

    let mut plaintext_path = valid_input();
    plaintext_path.plaintext_cache_paths = 1;
    let plaintext_path_decision = plaintext_path.evaluate();
    assert_rejected(
        plaintext_path_decision,
        MediaObjectIndexProductionOpenReason::PlaintextCachePathForbidden,
    );
    assert!(plaintext_path_decision.requires_destructive_repair);
}

#[test]
fn production_open_rejects_missing_indexes_and_network_binding() {
    let mut missing_object_id = valid_input();
    missing_object_id.object_id_index_present = false;
    let missing_object_id_decision = missing_object_id.evaluate();
    assert_rejected(
        missing_object_id_decision,
        MediaObjectIndexProductionOpenReason::ObjectIdIndexMissing,
    );
    assert!(missing_object_id_decision.requires_migration);

    let mut missing_digest = valid_input();
    missing_digest.content_digest_index_present = false;
    let missing_digest_decision = missing_digest.evaluate();
    assert_rejected(
        missing_digest_decision,
        MediaObjectIndexProductionOpenReason::ContentDigestIndexMissing,
    );
    assert!(missing_digest_decision.requires_migration);

    let mut missing_lifecycle = valid_input();
    missing_lifecycle.lifecycle_index_present = false;
    let missing_lifecycle_decision = missing_lifecycle.evaluate();
    assert_rejected(
        missing_lifecycle_decision,
        MediaObjectIndexProductionOpenReason::LifecycleIndexMissing,
    );
    assert!(missing_lifecycle_decision.requires_migration);

    let mut namespace_unbound = valid_input();
    namespace_unbound.object_namespace_bound = false;
    let namespace_unbound_decision = namespace_unbound.evaluate();
    assert_rejected(
        namespace_unbound_decision,
        MediaObjectIndexProductionOpenReason::ObjectNamespaceUnbound,
    );
    assert!(namespace_unbound_decision.requires_network_setup);

    let mut unauthenticated = valid_input();
    unauthenticated.media_service_authenticated = false;
    let unauthenticated_decision = unauthenticated.evaluate();
    assert_rejected(
        unauthenticated_decision,
        MediaObjectIndexProductionOpenReason::MediaServiceUnauthenticated,
    );
    assert!(unauthenticated_decision.requires_network_setup);
}

#[test]
fn production_open_separates_crash_recovery_states() {
    let mut replay = valid_input();
    replay.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;
    let replay_decision = replay.evaluate();
    assert_rejected(
        replay_decision,
        MediaObjectIndexProductionOpenReason::WalReplayRequired,
    );
    assert!(replay_decision.can_replay_wal);
    assert!(replay_decision.requires_crash_recovery);
    assert!(!replay_decision.requires_destructive_repair);

    let mut dirty = valid_input();
    dirty.crash_recovery = LocalStoreCrashRecoveryState::DirtyWithoutWal;
    let dirty_decision = dirty.evaluate();
    assert_rejected(
        dirty_decision,
        MediaObjectIndexProductionOpenReason::DirtyShutdownWithoutWal,
    );
    assert!(!dirty_decision.can_replay_wal);
    assert!(dirty_decision.requires_destructive_repair);

    let mut failed = valid_input();
    failed.crash_recovery = LocalStoreCrashRecoveryState::ReplayFailed;
    let failed_decision = failed.evaluate();
    assert_rejected(
        failed_decision,
        MediaObjectIndexProductionOpenReason::WalReplayFailed,
    );
    assert!(!failed_decision.can_replay_wal);
    assert!(failed_decision.requires_destructive_repair);
}

#[test]
fn production_adapter_opens_and_replays_only_accepted_states() {
    let mut store = RecordingProductionMediaIndexStore::default();

    let accepted = open_production_media_object_index(&mut store, valid_input())
        .expect("open should not fail");
    assert!(accepted.accepted);
    assert_eq!(store.open_calls, 1);
    assert_eq!(store.replay_calls, 0);

    let mut rejected_input = valid_input();
    rejected_input.plaintext_metadata_rows = 1;
    let rejected = open_production_media_object_index(&mut store, rejected_input)
        .expect("rejected decision should not call adapter open");
    assert!(!rejected.accepted);
    assert_eq!(store.open_calls, 1);

    let replayed = replay_production_media_object_index_wal(&mut store, accepted)
        .expect("accepted state should not replay wal");
    assert!(!replayed);
    assert_eq!(store.replay_calls, 0);

    let mut replay_input = valid_input();
    replay_input.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;
    let replay_decision = replay_input.evaluate();
    let replayed = replay_production_media_object_index_wal(&mut store, replay_decision)
        .expect("wal replay should not fail");
    assert!(replayed);
    assert_eq!(store.replay_calls, 1);
}

#[test]
fn production_open_reasons_have_stable_codes_and_labels() {
    let reasons = [
        (
            MediaObjectIndexProductionOpenReason::Accepted,
            0,
            "ACCEPTED",
        ),
        (
            MediaObjectIndexProductionOpenReason::UnsupportedIndexVersion,
            1,
            "UNSUPPORTED_INDEX_VERSION",
        ),
        (
            MediaObjectIndexProductionOpenReason::HeaderMagicMismatch,
            2,
            "HEADER_MAGIC_MISMATCH",
        ),
        (
            MediaObjectIndexProductionOpenReason::HeaderSuiteMismatch,
            3,
            "HEADER_SUITE_MISMATCH",
        ),
        (
            MediaObjectIndexProductionOpenReason::BadHeaderNonceLength,
            4,
            "BAD_HEADER_NONCE_LENGTH",
        ),
        (
            MediaObjectIndexProductionOpenReason::BadHeaderTagLength,
            5,
            "BAD_HEADER_TAG_LENGTH",
        ),
        (
            MediaObjectIndexProductionOpenReason::PlaintextMetadataForbidden,
            6,
            "PLAINTEXT_METADATA_FORBIDDEN",
        ),
        (
            MediaObjectIndexProductionOpenReason::PlaintextCachePathForbidden,
            7,
            "PLAINTEXT_CACHE_PATH_FORBIDDEN",
        ),
        (
            MediaObjectIndexProductionOpenReason::ObjectIdIndexMissing,
            8,
            "OBJECT_ID_INDEX_MISSING",
        ),
        (
            MediaObjectIndexProductionOpenReason::ContentDigestIndexMissing,
            9,
            "CONTENT_DIGEST_INDEX_MISSING",
        ),
        (
            MediaObjectIndexProductionOpenReason::LifecycleIndexMissing,
            10,
            "LIFECYCLE_INDEX_MISSING",
        ),
        (
            MediaObjectIndexProductionOpenReason::ObjectNamespaceUnbound,
            11,
            "OBJECT_NAMESPACE_UNBOUND",
        ),
        (
            MediaObjectIndexProductionOpenReason::MediaServiceUnauthenticated,
            12,
            "MEDIA_SERVICE_UNAUTHENTICATED",
        ),
        (
            MediaObjectIndexProductionOpenReason::WalReplayRequired,
            13,
            "WAL_REPLAY_REQUIRED",
        ),
        (
            MediaObjectIndexProductionOpenReason::DirtyShutdownWithoutWal,
            14,
            "DIRTY_SHUTDOWN_WITHOUT_WAL",
        ),
        (
            MediaObjectIndexProductionOpenReason::WalReplayFailed,
            15,
            "WAL_REPLAY_FAILED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn valid_input() -> MediaObjectIndexProductionOpenInput {
    MediaObjectIndexProductionOpenInput {
        index_version: MERCURY_MEDIA_OBJECT_INDEX_VERSION,
        header_magic_matches: true,
        header_suite_code: LocalStoreSealingSuite::MercuryLocalStoreV1.code(),
        header_nonce_len: LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        header_tag_len: LocalStoreSealingSuite::MercuryLocalStoreV1.authentication_tag_len(),
        plaintext_metadata_rows: 0,
        plaintext_cache_paths: 0,
        object_id_index_present: true,
        content_digest_index_present: true,
        lifecycle_index_present: true,
        object_namespace_bound: true,
        media_service_authenticated: true,
        crash_recovery: LocalStoreCrashRecoveryState::Clean,
    }
}

fn assert_rejected(
    decision: MediaObjectIndexProductionOpenDecision,
    reason: MediaObjectIndexProductionOpenReason,
) {
    assert!(!decision.accepted);
    assert!(!decision.can_open_index);
    assert!(!decision.can_load_manifests);
    assert!(!decision.can_write_manifests);
    assert!(!decision.can_use_remote_objects);
    assert_eq!(decision.reason, reason);
}

#[derive(Default)]
struct RecordingProductionMediaIndexStore {
    open_calls: usize,
    replay_calls: usize,
}

impl MediaObjectIndexStoreAdapter for RecordingProductionMediaIndexStore {
    type Error = Infallible;

    fn put_accepted_media_object_index(
        &mut self,
        _write: AcceptedMediaObjectIndexStoreWrite<'_>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn read_media_object_index(
        &self,
        _object_id: &[u8],
    ) -> Result<Option<MediaObjectIndexRecord>, Self::Error> {
        Ok(None)
    }

    fn delete_media_object_index(&mut self, _object_id: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn media_object_index_record_count(&self) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

impl ProductionMediaObjectIndexStoreAdapter for RecordingProductionMediaIndexStore {
    fn open_index(
        &mut self,
        open: AcceptedMediaObjectIndexProductionOpen,
    ) -> Result<(), Self::Error> {
        assert!(open.decision().accepted);
        self.open_calls += 1;
        Ok(())
    }

    fn replay_wal(&mut self, replay: AcceptedMediaObjectIndexWalReplay) -> Result<(), Self::Error> {
        assert!(replay.decision().can_replay_wal);
        self.replay_calls += 1;
        Ok(())
    }
}
