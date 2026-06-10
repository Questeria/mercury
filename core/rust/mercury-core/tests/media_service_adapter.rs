use std::convert::Infallible;

use mercury_core::{
    AcceptedMediaServiceUpload, MediaObjectStoreDecision, MediaObjectStoreReason,
    MediaServiceAdapter, MediaServiceAdapterInput, MediaServiceAdapterKind,
    MediaServiceAdapterReason, upload_media_object_with_adapter,
};

#[test]
fn media_service_adapter_accepts_production_object_store() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert_eq!(decision.reason, MediaServiceAdapterReason::Accepted);
    assert_eq!(decision.reason_code(), 0);
    assert_eq!(decision.reason_label(), "ACCEPTED");
    assert!(decision.can_upload_object);
    assert!(decision.can_persist_remote_ciphertext);
    assert!(decision.forbids_plaintext_upload);
    assert!(!decision.requires_network_setup);
    assert!(!decision.requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
}

#[test]
fn media_service_adapter_uploads_only_after_accepted_gate() {
    let mut adapter = RecordingMediaAdapter::default();

    let accepted = upload_media_object_with_adapter(&mut adapter, valid_input())
        .expect("accepted media service adapter should not fail");

    assert!(accepted.accepted);
    assert_eq!(adapter.upload_calls, 1);
    assert_eq!(
        adapter.last_reason,
        Some(MediaServiceAdapterReason::Accepted)
    );

    let mut rejected_input = valid_input();
    rejected_input.media_object_store = rejected_media_decision(true);
    let rejected = upload_media_object_with_adapter(&mut adapter, rejected_input)
        .expect("rejected media service adapter should not fail");

    assert!(!rejected.accepted);
    assert_eq!(
        rejected.reason,
        MediaServiceAdapterReason::MediaObjectStoreRejected
    );
    assert_eq!(adapter.upload_calls, 1);
}

#[test]
fn media_service_adapter_rejects_media_gate_plaintext_and_development_paths() {
    let mut media_rejected = valid_input();
    media_rejected.media_object_store = rejected_media_decision(true);
    let media_decision = media_rejected.evaluate();

    assert!(!media_decision.accepted);
    assert_eq!(
        media_decision.reason,
        MediaServiceAdapterReason::MediaObjectStoreRejected
    );
    assert!(media_decision.requires_user_action);
    assert!(media_decision.forbids_plaintext_upload);

    let mut plaintext = valid_input();
    plaintext.adapter_kind = MediaServiceAdapterKind::PlaintextDebugStore;
    let plaintext_decision = plaintext.evaluate();

    assert!(!plaintext_decision.accepted);
    assert_eq!(
        plaintext_decision.reason,
        MediaServiceAdapterReason::PlaintextAdapterForbidden
    );
    assert!(plaintext_decision.requires_user_action);
    assert!(!plaintext_decision.plaintext_bytes_exposed);

    let mut development = valid_input();
    development.adapter_kind = MediaServiceAdapterKind::DevelopmentMemoryObjectStore;
    development.allow_development_adapter = false;
    let development_decision = development.evaluate();

    assert!(!development_decision.accepted);
    assert_eq!(
        development_decision.reason,
        MediaServiceAdapterReason::DevelopmentAdapterForbidden
    );
}

#[test]
fn media_service_adapter_reports_auth_namespace_and_digest_requirements() {
    let mut service_auth = valid_input();
    service_auth.service_authenticated = false;
    let service_auth_decision = service_auth.evaluate();

    assert!(!service_auth_decision.accepted);
    assert_eq!(
        service_auth_decision.reason,
        MediaServiceAdapterReason::ServiceAuthenticationMissing
    );
    assert!(service_auth_decision.requires_network_setup);

    let mut upload_auth = valid_input();
    upload_auth.upload_authorized = false;
    let upload_auth_decision = upload_auth.evaluate();

    assert!(!upload_auth_decision.accepted);
    assert_eq!(
        upload_auth_decision.reason,
        MediaServiceAdapterReason::UploadAuthorizationMissing
    );
    assert!(upload_auth_decision.requires_network_setup);

    let mut namespace = valid_input();
    namespace.object_namespace_bound = false;
    let namespace_decision = namespace.evaluate();

    assert!(!namespace_decision.accepted);
    assert_eq!(
        namespace_decision.reason,
        MediaServiceAdapterReason::ObjectNamespaceUnbound
    );

    let mut digest = valid_input();
    digest.content_digest_verified = false;
    let digest_decision = digest.evaluate();

    assert!(!digest_decision.accepted);
    assert_eq!(
        digest_decision.reason,
        MediaServiceAdapterReason::ContentDigestUnverified
    );
}

#[test]
fn media_service_adapter_kinds_and_reasons_have_stable_codes_and_labels() {
    let kinds = [
        (
            MediaServiceAdapterKind::ProductionObjectStore,
            1,
            "production_object_store",
        ),
        (
            MediaServiceAdapterKind::DevelopmentMemoryObjectStore,
            2,
            "development_memory_object_store",
        ),
        (
            MediaServiceAdapterKind::PlaintextDebugStore,
            3,
            "plaintext_debug_store",
        ),
    ];

    for (kind, code, label) in kinds {
        assert_eq!(kind.code(), code);
        assert_eq!(kind.label(), label);
    }

    let reasons = [
        (MediaServiceAdapterReason::Accepted, 0, "ACCEPTED"),
        (
            MediaServiceAdapterReason::MediaObjectStoreRejected,
            1,
            "MEDIA_OBJECT_STORE_REJECTED",
        ),
        (
            MediaServiceAdapterReason::PlaintextAdapterForbidden,
            2,
            "PLAINTEXT_ADAPTER_FORBIDDEN",
        ),
        (
            MediaServiceAdapterReason::DevelopmentAdapterForbidden,
            3,
            "DEVELOPMENT_ADAPTER_FORBIDDEN",
        ),
        (
            MediaServiceAdapterReason::ServiceAuthenticationMissing,
            4,
            "SERVICE_AUTHENTICATION_MISSING",
        ),
        (
            MediaServiceAdapterReason::UploadAuthorizationMissing,
            5,
            "UPLOAD_AUTHORIZATION_MISSING",
        ),
        (
            MediaServiceAdapterReason::ObjectNamespaceUnbound,
            6,
            "OBJECT_NAMESPACE_UNBOUND",
        ),
        (
            MediaServiceAdapterReason::ContentDigestUnverified,
            7,
            "CONTENT_DIGEST_UNVERIFIED",
        ),
    ];

    for (reason, code, label) in reasons {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

#[derive(Default)]
struct RecordingMediaAdapter {
    upload_calls: usize,
    last_reason: Option<MediaServiceAdapterReason>,
}

impl MediaServiceAdapter for RecordingMediaAdapter {
    type Error = Infallible;

    fn upload_accepted_media(
        &mut self,
        accepted: AcceptedMediaServiceUpload,
    ) -> Result<(), Self::Error> {
        self.upload_calls += 1;
        self.last_reason = Some(accepted.decision().reason);
        Ok(())
    }
}

fn valid_input() -> MediaServiceAdapterInput {
    MediaServiceAdapterInput {
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        media_object_store: accepted_media_decision(),
        service_authenticated: true,
        upload_authorized: true,
        object_namespace_bound: true,
        content_digest_verified: true,
        allow_development_adapter: false,
    }
}

fn accepted_media_decision() -> MediaObjectStoreDecision {
    MediaObjectStoreDecision {
        accepted: true,
        can_upload: true,
        can_persist_local_ciphertext: true,
        requires_user_action: false,
        plaintext_bytes_exposed: false,
        reason: MediaObjectStoreReason::Accepted,
    }
}

fn rejected_media_decision(requires_user_action: bool) -> MediaObjectStoreDecision {
    MediaObjectStoreDecision {
        accepted: false,
        can_upload: false,
        can_persist_local_ciphertext: false,
        requires_user_action,
        plaintext_bytes_exposed: false,
        reason: MediaObjectStoreReason::PlaintextUploadForbidden,
    }
}
