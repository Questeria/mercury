use std::convert::Infallible;

use mercury_core::{
    MediaRetentionAdapter, MediaRetentionDecision, MediaRetentionInput, MediaRetentionOperation,
    MediaRetentionReason, MediaServiceAdapterKind, apply_media_retention_with_adapter,
};

#[test]
fn media_retention_accepts_remote_delete_and_local_cache_eviction() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert_eq!(
        decision.operation,
        MediaRetentionOperation::DeleteRemoteAndEvictLocalCache
    );
    assert!(decision.can_delete_remote_object);
    assert!(decision.can_evict_local_cache);
    assert!(decision.keeps_audit_hash);
    assert!(!decision.requires_network_setup);
    assert!(!decision.requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
    assert!(decision.forbids_plaintext_deletion);
    assert_eq!(decision.reason, MediaRetentionReason::Accepted);
}

#[test]
fn media_retention_calls_adapter_only_for_accepted_cleanup_actions() {
    let mut adapter = RecordingRetentionAdapter::default();
    let accepted = apply_media_retention_with_adapter(&mut adapter, valid_input())
        .expect("recording adapter is infallible");

    assert!(accepted.accepted);
    assert_eq!(adapter.apply_calls, 1);
    assert_eq!(
        adapter.last_operation,
        Some(MediaRetentionOperation::DeleteRemoteAndEvictLocalCache)
    );

    let mut retain = valid_input();
    retain.operation = MediaRetentionOperation::Retain;
    let retained = apply_media_retention_with_adapter(&mut adapter, retain)
        .expect("recording adapter is infallible");

    assert!(retained.accepted);
    assert!(!retained.can_delete_remote_object);
    assert!(!retained.can_evict_local_cache);
    assert_eq!(adapter.apply_calls, 1);

    let mut bad = valid_input();
    bad.plaintext_bytes = 1;
    let rejected = apply_media_retention_with_adapter(&mut adapter, bad)
        .expect("recording adapter is infallible");

    assert!(!rejected.accepted);
    assert_eq!(adapter.apply_calls, 1);
}

#[test]
fn media_retention_retain_is_safe_noop_with_audit_hash() {
    let mut input = valid_input();
    input.operation = MediaRetentionOperation::Retain;
    input.service_authenticated = false;
    input.delete_authorized = false;

    let decision = input.evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.operation, MediaRetentionOperation::Retain);
    assert!(!decision.can_delete_remote_object);
    assert!(!decision.can_evict_local_cache);
    assert!(decision.keeps_audit_hash);
    assert!(!decision.requires_network_setup);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn local_cache_eviction_can_run_without_network_but_requires_trigger() {
    let mut input = valid_input();
    input.operation = MediaRetentionOperation::EvictLocalCache;
    input.service_authenticated = false;
    input.delete_authorized = false;
    input.user_delete_requested = false;
    input.cache_eviction_requested = true;

    let decision = input.evaluate();
    assert!(decision.accepted);
    assert!(!decision.can_delete_remote_object);
    assert!(decision.can_evict_local_cache);
    assert!(!decision.requires_network_setup);

    input.cache_eviction_requested = false;
    assert_rejected(
        input.evaluate(),
        MediaRetentionReason::CacheEvictionNotRequested,
    );
}

#[test]
fn plaintext_wrong_kind_and_malformed_metadata_never_delete() {
    let mut plaintext = valid_input();
    plaintext.plaintext_bytes = 1;
    let plaintext_decision = plaintext.evaluate();
    assert_rejected(
        plaintext_decision,
        MediaRetentionReason::PlaintextDeletionForbidden,
    );
    assert!(plaintext_decision.requires_user_action);

    let mut wrong_kind = valid_input();
    wrong_kind.record_kind = mercury_core::LocalStoreRecordKind::MediaPlaintext;
    assert_rejected(
        wrong_kind.evaluate(),
        MediaRetentionReason::MediaRecordKindMismatch,
    );

    let mut bad_object = valid_input();
    bad_object.object_id_len = 16;
    assert_rejected(
        bad_object.evaluate(),
        MediaRetentionReason::BadObjectIdLength,
    );

    let mut bad_digest = valid_input();
    bad_digest.content_digest_len = 16;
    assert_rejected(
        bad_digest.evaluate(),
        MediaRetentionReason::BadContentDigestLength,
    );
}

#[test]
fn retention_hold_and_missing_user_delete_block_remote_delete() {
    let mut legal_hold = valid_input();
    legal_hold.retention_hold_active = true;
    let hold_decision = legal_hold.evaluate();
    assert_rejected(hold_decision, MediaRetentionReason::RetentionHoldActive);
    assert!(hold_decision.requires_user_action);

    let mut missing_user_delete = valid_input();
    missing_user_delete.user_delete_requested = false;
    let user_decision = missing_user_delete.evaluate();
    assert_rejected(user_decision, MediaRetentionReason::UserDeleteRequired);
    assert!(user_decision.requires_user_action);
}

#[test]
fn remote_delete_requires_production_authenticated_authorized_service() {
    let mut plaintext_adapter = valid_input();
    plaintext_adapter.adapter_kind = MediaServiceAdapterKind::PlaintextDebugStore;
    assert_rejected(
        plaintext_adapter.evaluate(),
        MediaRetentionReason::PlaintextAdapterForbidden,
    );

    let mut development = valid_input();
    development.adapter_kind = MediaServiceAdapterKind::DevelopmentMemoryObjectStore;
    assert_rejected(
        development.evaluate(),
        MediaRetentionReason::DevelopmentAdapterForbidden,
    );

    development.allow_development_adapter = true;
    assert!(development.evaluate().accepted);

    let mut auth = valid_input();
    auth.service_authenticated = false;
    let auth_decision = auth.evaluate();
    assert_rejected(
        auth_decision,
        MediaRetentionReason::ServiceAuthenticationMissing,
    );
    assert!(auth_decision.requires_network_setup);
    assert!(auth_decision.requires_user_action);

    let mut authorization = valid_input();
    authorization.delete_authorized = false;
    let authorization_decision = authorization.evaluate();
    assert_rejected(
        authorization_decision,
        MediaRetentionReason::DeleteAuthorizationMissing,
    );
    assert!(authorization_decision.requires_network_setup);
    assert!(authorization_decision.requires_user_action);

    let mut namespace = valid_input();
    namespace.object_namespace_bound = false;
    assert_rejected(
        namespace.evaluate(),
        MediaRetentionReason::ObjectNamespaceUnbound,
    );

    let mut digest = valid_input();
    digest.content_digest_verified = false;
    assert_rejected(
        digest.evaluate(),
        MediaRetentionReason::ContentDigestUnverified,
    );
}

#[test]
fn media_retention_reasons_and_operations_have_stable_codes_and_labels() {
    let operations = [
        (MediaRetentionOperation::Retain, 0, "retain"),
        (
            MediaRetentionOperation::EvictLocalCache,
            1,
            "evict_local_cache",
        ),
        (
            MediaRetentionOperation::DeleteRemoteObject,
            2,
            "delete_remote_object",
        ),
        (
            MediaRetentionOperation::DeleteRemoteAndEvictLocalCache,
            3,
            "delete_remote_and_evict_local_cache",
        ),
    ];

    for (operation, code, label) in operations {
        assert_eq!(operation.code(), code);
        assert_eq!(operation.label(), label);
    }

    let reasons = [
        (MediaRetentionReason::Accepted, 0, "ACCEPTED"),
        (
            MediaRetentionReason::PlaintextDeletionForbidden,
            1,
            "PLAINTEXT_DELETION_FORBIDDEN",
        ),
        (
            MediaRetentionReason::MediaRecordKindMismatch,
            2,
            "MEDIA_RECORD_KIND_MISMATCH",
        ),
        (
            MediaRetentionReason::BadObjectIdLength,
            3,
            "BAD_OBJECT_ID_LENGTH",
        ),
        (
            MediaRetentionReason::BadContentDigestLength,
            4,
            "BAD_CONTENT_DIGEST_LENGTH",
        ),
        (
            MediaRetentionReason::RetentionHoldActive,
            5,
            "RETENTION_HOLD_ACTIVE",
        ),
        (
            MediaRetentionReason::UserDeleteRequired,
            6,
            "USER_DELETE_REQUIRED",
        ),
        (
            MediaRetentionReason::CacheEvictionNotRequested,
            7,
            "CACHE_EVICTION_NOT_REQUESTED",
        ),
        (
            MediaRetentionReason::PlaintextAdapterForbidden,
            8,
            "PLAINTEXT_ADAPTER_FORBIDDEN",
        ),
        (
            MediaRetentionReason::DevelopmentAdapterForbidden,
            9,
            "DEVELOPMENT_ADAPTER_FORBIDDEN",
        ),
        (
            MediaRetentionReason::ServiceAuthenticationMissing,
            10,
            "SERVICE_AUTHENTICATION_MISSING",
        ),
        (
            MediaRetentionReason::DeleteAuthorizationMissing,
            11,
            "DELETE_AUTHORIZATION_MISSING",
        ),
        (
            MediaRetentionReason::ObjectNamespaceUnbound,
            12,
            "OBJECT_NAMESPACE_UNBOUND",
        ),
        (
            MediaRetentionReason::ContentDigestUnverified,
            13,
            "CONTENT_DIGEST_UNVERIFIED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(decision: MediaRetentionDecision, reason: MediaRetentionReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_delete_remote_object);
    assert!(!decision.can_evict_local_cache);
    assert!(decision.keeps_audit_hash);
    assert!(!decision.plaintext_bytes_exposed);
    assert!(decision.forbids_plaintext_deletion);
    assert_eq!(decision.reason, reason);
}

fn valid_input() -> MediaRetentionInput {
    MediaRetentionInput {
        operation: MediaRetentionOperation::DeleteRemoteAndEvictLocalCache,
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        record_kind: mercury_core::LocalStoreRecordKind::MediaCiphertext,
        service_authenticated: true,
        delete_authorized: true,
        object_namespace_bound: true,
        content_digest_verified: true,
        allow_development_adapter: false,
        user_delete_requested: true,
        cache_eviction_requested: false,
        retention_hold_active: false,
        object_id_len: 32,
        content_digest_len: 32,
        plaintext_bytes: 0,
    }
}

#[derive(Default)]
struct RecordingRetentionAdapter {
    apply_calls: usize,
    last_operation: Option<MediaRetentionOperation>,
}

impl MediaRetentionAdapter for RecordingRetentionAdapter {
    type Error = Infallible;

    fn apply_accepted_media_retention(
        &mut self,
        accepted: mercury_core::AcceptedMediaRetention,
    ) -> Result<(), Self::Error> {
        self.apply_calls += 1;
        self.last_operation = Some(accepted.decision().operation);
        Ok(())
    }
}
