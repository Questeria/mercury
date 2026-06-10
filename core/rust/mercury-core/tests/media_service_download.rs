use std::convert::Infallible;

use mercury_core::{
    MediaServiceAdapterKind, MediaServiceDownloadAdapter, MediaServiceDownloadDecision,
    MediaServiceDownloadInput, MediaServiceDownloadReason, download_media_object_with_adapter,
};

#[test]
fn media_service_download_accepts_production_object_store() {
    let decision = valid_input().evaluate();

    assert!(decision.accepted);
    assert!(decision.can_download_object);
    assert!(decision.can_persist_local_ciphertext);
    assert!(!decision.requires_network_setup);
    assert!(!decision.requires_user_action);
    assert!(!decision.plaintext_bytes_exposed);
    assert!(decision.forbids_plaintext_preview);
    assert_eq!(decision.reason, MediaServiceDownloadReason::Accepted);
}

#[test]
fn media_service_download_calls_adapter_only_after_accepted_gate() {
    let mut adapter = RecordingDownloadAdapter::default();
    let accepted = download_media_object_with_adapter(&mut adapter, valid_input())
        .expect("recording adapter is infallible");

    assert!(accepted.accepted);
    assert_eq!(adapter.download_calls, 1);
    assert_eq!(
        adapter.last_reason,
        Some(MediaServiceDownloadReason::Accepted)
    );

    let mut bad = valid_input();
    bad.plaintext_preview_bytes = 1;
    let rejected = download_media_object_with_adapter(&mut adapter, bad)
        .expect("recording adapter infallible");

    assert!(!rejected.accepted);
    assert_eq!(adapter.download_calls, 1);
}

#[test]
fn media_service_download_rejects_plaintext_auto_download_and_dev_paths() {
    let mut plaintext = valid_input();
    plaintext.plaintext_preview_bytes = 16;
    assert_rejected(
        plaintext.evaluate(),
        MediaServiceDownloadReason::PlaintextPreviewForbidden,
    );

    let mut auto = valid_input();
    auto.automatic_download_requested = true;
    assert_rejected(
        auto.evaluate(),
        MediaServiceDownloadReason::AutomaticDownloadForbidden,
    );

    let mut plaintext_adapter = valid_input();
    plaintext_adapter.adapter_kind = MediaServiceAdapterKind::PlaintextDebugStore;
    assert_rejected(
        plaintext_adapter.evaluate(),
        MediaServiceDownloadReason::PlaintextAdapterForbidden,
    );

    let mut development = valid_input();
    development.adapter_kind = MediaServiceAdapterKind::DevelopmentMemoryObjectStore;
    assert_rejected(
        development.evaluate(),
        MediaServiceDownloadReason::DevelopmentAdapterForbidden,
    );

    development.allow_development_adapter = true;
    assert!(development.evaluate().accepted);
}

#[test]
fn media_service_download_reports_auth_namespace_and_digest_requirements() {
    let mut auth = valid_input();
    auth.service_authenticated = false;
    let auth_decision = auth.evaluate();
    assert_rejected(
        auth_decision,
        MediaServiceDownloadReason::ServiceAuthenticationMissing,
    );
    assert!(auth_decision.requires_network_setup);
    assert!(auth_decision.requires_user_action);

    let mut download_auth = valid_input();
    download_auth.download_authorized = false;
    let download_auth_decision = download_auth.evaluate();
    assert_rejected(
        download_auth_decision,
        MediaServiceDownloadReason::DownloadAuthorizationMissing,
    );
    assert!(download_auth_decision.requires_network_setup);
    assert!(download_auth_decision.requires_user_action);

    let mut namespace = valid_input();
    namespace.object_namespace_bound = false;
    assert_rejected(
        namespace.evaluate(),
        MediaServiceDownloadReason::ObjectNamespaceUnbound,
    );

    let mut digest = valid_input();
    digest.content_digest_verified = false;
    assert_rejected(
        digest.evaluate(),
        MediaServiceDownloadReason::ContentDigestUnverified,
    );
}

#[test]
fn media_service_download_metadata_is_opaque_and_bounded() {
    let mut bad_object = valid_input();
    bad_object.object_id_len = 16;
    assert_rejected(
        bad_object.evaluate(),
        MediaServiceDownloadReason::BadObjectIdLength,
    );

    let mut empty_ciphertext = valid_input();
    empty_ciphertext.ciphertext_len = 0;
    assert_rejected(
        empty_ciphertext.evaluate(),
        MediaServiceDownloadReason::BadCiphertextLength,
    );

    let mut too_large = valid_input();
    too_large.ciphertext_len = mercury_core::MERCURY_MAX_MEDIA_OBJECT_BYTES + 1;
    assert_rejected(
        too_large.evaluate(),
        MediaServiceDownloadReason::CiphertextTooLarge,
    );

    let mut bad_header = valid_input();
    bad_header.sealed_header_len = 4097;
    assert_rejected(
        bad_header.evaluate(),
        MediaServiceDownloadReason::BadSealedHeaderLength,
    );

    let mut bad_digest = valid_input();
    bad_digest.content_digest_len = 16;
    assert_rejected(
        bad_digest.evaluate(),
        MediaServiceDownloadReason::BadContentDigestLength,
    );

    let mut bad_commitment = valid_input();
    bad_commitment.media_key_commitment_len = 24;
    assert_rejected(
        bad_commitment.evaluate(),
        MediaServiceDownloadReason::BadMediaKeyCommitmentLength,
    );
}

#[test]
fn media_service_download_reasons_have_stable_codes_and_labels() {
    let cases = [
        (MediaServiceDownloadReason::Accepted, 0, "ACCEPTED"),
        (
            MediaServiceDownloadReason::PlaintextPreviewForbidden,
            1,
            "PLAINTEXT_PREVIEW_FORBIDDEN",
        ),
        (
            MediaServiceDownloadReason::AutomaticDownloadForbidden,
            2,
            "AUTOMATIC_DOWNLOAD_FORBIDDEN",
        ),
        (
            MediaServiceDownloadReason::PlaintextAdapterForbidden,
            3,
            "PLAINTEXT_ADAPTER_FORBIDDEN",
        ),
        (
            MediaServiceDownloadReason::DevelopmentAdapterForbidden,
            4,
            "DEVELOPMENT_ADAPTER_FORBIDDEN",
        ),
        (
            MediaServiceDownloadReason::ServiceAuthenticationMissing,
            5,
            "SERVICE_AUTHENTICATION_MISSING",
        ),
        (
            MediaServiceDownloadReason::DownloadAuthorizationMissing,
            6,
            "DOWNLOAD_AUTHORIZATION_MISSING",
        ),
        (
            MediaServiceDownloadReason::ObjectNamespaceUnbound,
            7,
            "OBJECT_NAMESPACE_UNBOUND",
        ),
        (
            MediaServiceDownloadReason::ContentDigestUnverified,
            8,
            "CONTENT_DIGEST_UNVERIFIED",
        ),
        (
            MediaServiceDownloadReason::BadObjectIdLength,
            9,
            "BAD_OBJECT_ID_LENGTH",
        ),
        (
            MediaServiceDownloadReason::BadCiphertextLength,
            10,
            "BAD_CIPHERTEXT_LENGTH",
        ),
        (
            MediaServiceDownloadReason::CiphertextTooLarge,
            11,
            "CIPHERTEXT_TOO_LARGE",
        ),
        (
            MediaServiceDownloadReason::BadSealedHeaderLength,
            12,
            "BAD_SEALED_HEADER_LENGTH",
        ),
        (
            MediaServiceDownloadReason::BadContentDigestLength,
            13,
            "BAD_CONTENT_DIGEST_LENGTH",
        ),
        (
            MediaServiceDownloadReason::BadMediaKeyCommitmentLength,
            14,
            "BAD_MEDIA_KEY_COMMITMENT_LENGTH",
        ),
    ];

    for (reason, code, label) in cases {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.label(), label);
    }
}

fn assert_rejected(decision: MediaServiceDownloadDecision, reason: MediaServiceDownloadReason) {
    assert!(!decision.accepted);
    assert!(!decision.can_download_object);
    assert!(!decision.can_persist_local_ciphertext);
    assert!(!decision.plaintext_bytes_exposed);
    assert!(decision.forbids_plaintext_preview);
    assert_eq!(decision.reason, reason);
}

fn valid_input() -> MediaServiceDownloadInput {
    MediaServiceDownloadInput {
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        service_authenticated: true,
        download_authorized: true,
        object_namespace_bound: true,
        content_digest_verified: true,
        allow_development_adapter: false,
        object_id_len: 32,
        ciphertext_len: 4096,
        max_ciphertext_len: mercury_core::MERCURY_MAX_MEDIA_OBJECT_BYTES,
        sealed_header_len: 96,
        content_digest_len: 32,
        media_key_commitment_len: 32,
        plaintext_preview_bytes: 0,
        automatic_download_requested: false,
    }
}

#[derive(Default)]
struct RecordingDownloadAdapter {
    download_calls: usize,
    last_reason: Option<MediaServiceDownloadReason>,
}

impl MediaServiceDownloadAdapter for RecordingDownloadAdapter {
    type Error = Infallible;

    fn download_accepted_media(
        &mut self,
        accepted: mercury_core::AcceptedMediaServiceDownload,
    ) -> Result<(), Self::Error> {
        self.download_calls += 1;
        self.last_reason = Some(accepted.decision().reason);
        Ok(())
    }
}
