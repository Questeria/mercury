#![recursion_limit = "256"]

use mercury_core::{
    AccountRecoveryDecision, AccountRecoveryInput, AccountRecoveryMethod, ActorKind,
    AiConnectorDecision, AiConnectorInput, AiConnectorRuntimeKind, AiGrantFacts, AiLifecycleFacts,
    AiParticipantAction, AiParticipantRequest, AiPolicyFacts,
    AnonymousCredentialIssuerTrustDecision, AnonymousCredentialIssuerTrustInput,
    AnonymousGroupMembershipProofDecision, AnonymousGroupMembershipProofInput,
    AnonymousGroupMembershipProofScheme, AnonymousIssuerWitnessAuditInput,
    AnonymousNullifierStoreDecision, AnonymousNullifierStoreWrite,
    AnonymousRateLimitCredentialKind, AnonymousRateLimitNullifierDecision,
    AnonymousRateLimitNullifierInput, AuthenticatedRelaySourceDecision,
    AuthenticatedRelaySourceInput, AuthenticatedRelayTransportState, ClientBootstrapDecision,
    ClientBootstrapReason, ClientReceiveDecision, ClientReceiveReason, ClientReceiveReplayState,
    ComponentReasons, DeviceTrustDecision, DeviceTrustReason, GroupChatCryptoSuite,
    GroupChatDecision, GroupChatInput, GroupChatProtocol, GroupMessageTranscriptDecision,
    GroupMessageTranscriptInput, GroupRelayEnvelopeDecision, GroupRelayEnvelopeInput,
    InboundSyncDecision, InboundSyncInput, InboundSyncSourceState, KeyTransparencyProofInput,
    KeyTransparencyProofStatus, KeyTransparencyWitnessStatus, LocalStoreCrashRecoveryState,
    LocalStoreDatabaseAdapterKind, LocalStoreDatabaseAdapterSelectionDecision,
    LocalStoreDatabaseAdapterSelectionInput, LocalStoreDatabaseBindingKind,
    LocalStoreDatabaseCipher, LocalStoreDatabaseEngine, LocalStoreDatabaseKdf,
    LocalStoreDatabaseLicenseKind, LocalStoreDatabaseSecurityDecision,
    LocalStoreDatabaseSecurityInput, LocalStoreDatabaseTargetPlatform, LocalStoreKeyBinding,
    LocalStoreKeyDescriptor, LocalStoreKeyScope, LocalStoreKeychainBackend,
    LocalStoreKeychainProtection, LocalStoreKeychainUnlockDecision, LocalStoreKeychainUnlockInput,
    LocalStoreOpenRequest, LocalStoreOpenResult, LocalStorePayload,
    LocalStoreProductionOpenDecision, LocalStoreProductionOpenInput, LocalStoreRecordKind,
    LocalStoreRecordLocator, LocalStoreSealOutput, LocalStoreSealRequest, LocalStoreSealResult,
    LocalStoreSealingDecision, LocalStoreSealingReason, LocalStoreSealingSuite,
    LocalStoreUnlockDatabaseHeaderState, LocalStoreUnlockDecision, LocalStoreUnlockInput,
    LocalStoreUnlockSecretState, LocalStoreWriteRequest, MERCURY_LOCAL_STORE_MIN_KDF_ITERATIONS,
    MERCURY_LOCAL_STORE_PAGE_SIZE, MERCURY_LOCAL_STORE_VERSION, MERCURY_MAX_MEDIA_OBJECT_BYTES,
    MERCURY_MEDIA_OBJECT_INDEX_VERSION, MediaObjectIndexDecision, MediaObjectIndexInput,
    MediaObjectIndexProductionOpenDecision, MediaObjectIndexProductionOpenInput,
    MediaObjectIndexStoreDecision, MediaObjectIndexStoreWrite, MediaObjectLifecycleState,
    MediaObjectStoreDecision, MediaObjectStoreInput, MediaRetentionDecision, MediaRetentionInput,
    MediaRetentionOperation, MediaServiceAdapterDecision, MediaServiceAdapterInput,
    MediaServiceAdapterKind, MediaServiceDownloadDecision, MediaServiceDownloadInput,
    MlsCommitAdmissionDecision, MlsCommitAdmissionInput, MlsCommitReplayStoreDecision,
    MlsCommitReplayStoreWrite, MlsKeyPackageAdmissionDecision, MlsKeyPackageAdmissionInput,
    MlsKeyPackageConsumeStoreDecision, MlsKeyPackageConsumeStoreWrite,
    MlsMembershipTransactionDecision, MlsMembershipTransactionWrite, MlsProviderAdapterKind,
    MlsProviderAdapterSelectionDecision, MlsProviderAdapterSelectionInput,
    MlsProviderCryptoBackendKind, MlsProviderEvidenceStoreDecision, MlsProviderEvidenceStoreRecord,
    MlsProviderEvidenceStoreWrite, MlsProviderEvidenceUseDecision, MlsProviderEvidenceUseInput,
    MlsProviderImplementationLicenseKind, MlsProviderProtocolProfile, MlsProviderSecurityInput,
    MlsWelcomeAdmissionDecision, MlsWelcomeAdmissionInput, MlsWelcomeReplayStoreDecision,
    MlsWelcomeReplayStoreWrite, MlsWelcomeSendOutboxDecision, MlsWelcomeSendOutboxWrite,
    OutboundSendDecision, OutboundSendReason, PipelineAuditClass, PipelineReason,
    PlatformDecisionView, PlatformLocalStoreAdapterDecision, PlatformLocalStoreAdapterInput,
    PlatformLocalStoreAdapterKind, PlatformLocalStoreRuntime, PolicyDecision,
    PrototypeAiParticipantBackend, PrototypeAnonymousNullifierStore,
    PrototypeAuthenticatedInboundSyncSessionInput, PrototypeAuthenticatedInboundSyncSessionOutcome,
    PrototypeBackendCommand, PrototypeBackendCommandKind, PrototypeBackendCommandView,
    PrototypeBackendSession, PrototypeBackendSessionInput, PrototypeEncryptedLocalStore,
    PrototypeInboundSyncSession, PrototypeInboundSyncSessionOutcome,
    PrototypeIndexedMediaCleanupSession, PrototypeIndexedMediaCleanupSessionInput,
    PrototypeIndexedMediaCleanupSessionOutcome, PrototypeIndexedMediaDownloadSession,
    PrototypeIndexedMediaDownloadSessionInput, PrototypeIndexedMediaDownloadSessionOutcome,
    PrototypeIndexedMediaUploadSession, PrototypeIndexedMediaUploadSessionInput,
    PrototypeIndexedMediaUploadSessionOutcome, PrototypeLocalStoreCryptoProvider,
    PrototypeMediaCleanupSession, PrototypeMediaCleanupSessionInput,
    PrototypeMediaCleanupSessionOutcome, PrototypeMediaDownloadSession,
    PrototypeMediaDownloadSessionInput, PrototypeMediaDownloadSessionOutcome,
    PrototypeMediaObjectIndexStore, PrototypeMediaServiceUploadSession,
    PrototypeMediaServiceUploadSessionInput, PrototypeMediaServiceUploadSessionOutcome,
    PrototypeMediaUploadSession, PrototypeMediaUploadSessionInput,
    PrototypeMediaUploadSessionOutcome, PrototypeMlsCommitReplayStore,
    PrototypeMlsKeyPackageConsumeStore, PrototypeMlsMembershipTransactionStore,
    PrototypeMlsProviderEvidenceStore, PrototypeMlsWelcomeReplayStore,
    PrototypeMlsWelcomeSendOutbox, PrototypeProductionStoreSessionInput,
    PrototypeProductionStoreSessionOutcome, PrototypeReceiveSession, PrototypeReceiveSessionInput,
    PrototypeReceiveSessionOutcome, PrototypeRelayServer, PrototypeRelaySubmitRequest,
    PrototypeSealedAuditEventStore, PrototypeSealedAuditIncidentEvidenceStore,
    PrototypeSealedAuditPrivateReportGatewayEvidenceStore, PrototypeSealedAuditPrivateReportOutbox,
    PrototypeSealedAuditPrivateReportReceiptStore,
    PrototypeSealedAuditPrivateReportReconciliationStore, PrototypeSealedAuditProofCache,
    PrototypeSealedAuditRecoveryExportStore, PrototypeSealedAuditVerifierPolicyStore,
    RelaySubmissionDecision, SealedAuditAnchorKind, SealedAuditCheckpointSignatureAlgorithm,
    SealedAuditDatabaseAdapterDecision, SealedAuditDatabaseAdapterInput, SealedAuditEnvelopeSuite,
    SealedAuditEventChainDecision, SealedAuditEventChainInput, SealedAuditEventKind,
    SealedAuditEventStoreDecision, SealedAuditEventStoreReason, SealedAuditEventStoreWrite,
    SealedAuditIncidentEvidenceDecision, SealedAuditIncidentEvidenceReason,
    SealedAuditIncidentEvidenceWrite, SealedAuditPrivateReportGatewayEvidenceDecision,
    SealedAuditPrivateReportGatewayEvidenceWrite, SealedAuditPrivateReportOutboxDecision,
    SealedAuditPrivateReportOutboxWrite, SealedAuditPrivateReportReceiptDecision,
    SealedAuditPrivateReportReceiptWrite, SealedAuditPrivateReportReconciliationDecision,
    SealedAuditPrivateReportReconciliationWrite, SealedAuditPrivateReportTransportDecision,
    SealedAuditPrivateReportTransportInput, SealedAuditProofBundleDecision,
    SealedAuditProofBundleInput, SealedAuditProofCacheDecision, SealedAuditProofCacheWrite,
    SealedAuditRecoveryExportDecision, SealedAuditRecoveryExportWrite,
    SealedAuditVerifierPolicyDecision, SealedAuditVerifierPolicyReason,
    SealedAuditVerifierPolicySnapshot, SealedAuditWitnessCheckpointDecision,
    SealedAuditWitnessCheckpointInput, SealedAuditWitnessClientDecision,
    SealedAuditWitnessClientInput, SecureBackupRestoreDecision, SecureBackupRestoreEnvelopeSuite,
    SecureBackupRestoreInput, SecureBackupRestoreScope, SecureBackupRestoreTransport,
    build_sealed_local_store_write_request, evaluate_anonymous_credential_issuer_trust,
    evaluate_anonymous_issuer_witness_audit, evaluate_authenticated_relay_source,
    evaluate_inbound_sync, evaluate_key_transparency, evaluate_media_object_store,
    evaluate_mls_commit_admission, evaluate_mls_key_package_admission,
    evaluate_mls_provider_adapter_selection, evaluate_mls_provider_evidence_use,
    evaluate_mls_provider_security, evaluate_mls_welcome_admission,
    evaluate_sealed_audit_event_chain, evaluate_sealed_audit_proof_bundle,
    evaluate_sealed_audit_witness_checkpoint, evaluate_sealed_audit_witness_client,
    open_local_store_record, put_anonymous_nullifier_record, put_mls_commit_replay_record,
    put_mls_key_package_consumption_record, put_mls_membership_transaction_record,
    put_mls_provider_evidence_record, put_mls_welcome_replay_record,
    put_mls_welcome_send_outbox_record, put_sealed_audit_event_record,
    put_sealed_audit_incident_evidence_record,
    put_sealed_audit_private_report_gateway_evidence_record,
    put_sealed_audit_private_report_outbox_record, put_sealed_audit_private_report_receipt_record,
    put_sealed_audit_private_report_reconciliation_record, put_sealed_audit_proof_cache_record,
    put_sealed_audit_recovery_export_record, put_sealed_audit_verifier_policy_snapshot,
    run_prototype_production_store_session, seal_local_store_plaintext,
};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFixture {
    BootstrapAccepted,
    BootstrapSyncIncomplete,
    BootstrapRecoveryRequired,
    OutboundSendAccepted,
    OutboundSendMessagePolicyRejected,
    ClientReceiveAccepted,
    ClientReceiveOrderingGap,
    ClientReceiveSenderTrustAction,
    PolicyAiGrantRejected,
    PolicyAiLifecycleExpired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformFixtureDescriptor {
    pub name: &'static str,
    pub fixture: PlatformFixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrototypeFixture {
    LocalStoreSealedMessage,
    LocalStoreUnlockReady,
    LocalStoreUnlockAppLockRequired,
    LocalStoreUnlockRecoveryRequired,
    LocalStoreUnlockPlaintextCacheForbidden,
    AccountRecoveryHighEntropyReady,
    AccountRecoveryLowEntropyPinForbidden,
    AccountRecoveryThresholdQuorumRequired,
    AccountRecoveryPlaintextBackupForbidden,
    AccountRecoveryKeyRotationRequired,
    SecureBackupRestoreReady,
    SecureBackupRestoreRecoveryRejected,
    SecureBackupRestorePlaintextRejected,
    SecureBackupRestoreMlsRekeyRejected,
    SecureBackupRestoreCloudPolicyRejected,
    SealedAuditEventChainReady,
    SealedAuditEventChainPlaintextRejected,
    SealedAuditEventChainRollbackRejected,
    SealedAuditEventChainWitnessRejected,
    SealedAuditEventChainBindingRejected,
    SealedAuditEventStoreReady,
    SealedAuditEventStoreChainRejected,
    SealedAuditEventStoreDuplicateRejected,
    SealedAuditEventStoreRollbackRejected,
    SealedAuditEventStorePlaintextRejected,
    SealedAuditWitnessCheckpointReady,
    SealedAuditWitnessCheckpointStoreRejected,
    SealedAuditWitnessCheckpointQuorumRejected,
    SealedAuditWitnessCheckpointSplitViewRejected,
    SealedAuditWitnessCheckpointPrivacyRejected,
    SealedAuditWitnessClientReady,
    SealedAuditWitnessClientConflict,
    SealedAuditWitnessClientUnavailable,
    SealedAuditWitnessClientPolicyRejected,
    SealedAuditWitnessClientMonitorPrivacyRejected,
    SealedAuditProofBundleReady,
    SealedAuditProofBundleClientRejected,
    SealedAuditProofBundleStaleWitness,
    SealedAuditProofBundlePolicyRejected,
    SealedAuditProofBundlePrivacyRejected,
    SealedAuditProofCacheReady,
    SealedAuditProofCacheBundleRejected,
    SealedAuditProofCacheDuplicateRejected,
    SealedAuditProofCachePolicyStale,
    SealedAuditProofCachePlaintextRejected,
    SealedAuditVerifierPolicyReady,
    SealedAuditVerifierPolicyExpired,
    SealedAuditVerifierPolicyKeyRotationRequired,
    SealedAuditVerifierPolicyMonitorPrivacyRejected,
    SealedAuditVerifierPolicyPlaintextRejected,
    SealedAuditIncidentEvidenceReady,
    SealedAuditIncidentEvidencePolicyRejected,
    SealedAuditIncidentEvidenceMissingProofReport,
    SealedAuditIncidentEvidenceSplitView,
    SealedAuditIncidentEvidencePlaintextRejected,
    SealedAuditRecoveryExportReady,
    SealedAuditRecoveryExportIncidentRejected,
    SealedAuditRecoveryExportQuorumRequired,
    SealedAuditRecoveryExportRollbackRejected,
    SealedAuditRecoveryExportPlaintextRejected,
    SealedAuditDatabaseAdapterReady,
    SealedAuditDatabaseAdapterEncryptionRejected,
    SealedAuditDatabaseAdapterAppendOnlyRejected,
    SealedAuditPrivateReportTransportReady,
    SealedAuditPrivateReportTransportPlaintextRejected,
    SealedAuditPrivateReportOutboxReady,
    SealedAuditPrivateReportOutboxTransportRejected,
    SealedAuditPrivateReportOutboxReplayRejected,
    SealedAuditPrivateReportOutboxRateLimitRejected,
    SealedAuditPrivateReportOutboxPlaintextRejected,
    SealedAuditPrivateReportReceiptReady,
    SealedAuditPrivateReportReceiptOutboxRejected,
    SealedAuditPrivateReportReceiptMissing,
    SealedAuditPrivateReportReceiptTransparencyRejected,
    SealedAuditPrivateReportReceiptPlaintextRejected,
    SealedAuditPrivateReportReconciliationReady,
    SealedAuditPrivateReportReconciliationReceiptRejected,
    SealedAuditPrivateReportReconciliationRetryRejected,
    SealedAuditPrivateReportReconciliationFalseDeliveryRejected,
    SealedAuditPrivateReportReconciliationPlaintextRejected,
    SealedAuditPrivateReportGatewayEvidenceReady,
    SealedAuditPrivateReportGatewayEvidenceReconciliationRejected,
    SealedAuditPrivateReportGatewayEvidenceUnavailableRejected,
    SealedAuditPrivateReportGatewayEvidenceAccountabilityRejected,
    SealedAuditPrivateReportGatewayEvidencePlaintextRejected,
    GroupChatMlsReady,
    GroupChatMlsSetupRequired,
    GroupChatMembershipSyncRequired,
    GroupChatPlaintextMetadataForbidden,
    GroupChatHighSecurityMlsRequired,
    GroupChatHighSecurityPqRequired,
    GroupChatMlsProviderSecurityRequired,
    MlsProviderEvidenceStoreReady,
    MlsProviderEvidenceStoreGateRejected,
    MlsProviderEvidenceStoreDuplicateRejected,
    MlsProviderEvidenceStorePlaintextRejected,
    MlsProviderEvidenceUseReady,
    MlsProviderEvidenceUseMissing,
    MlsProviderEvidenceUseExpired,
    MlsProviderEvidenceUseSuiteMismatch,
    MlsProviderEvidenceUsePlaintextRejected,
    MlsProviderAdapterSelectionReady,
    MlsProviderAdapterSelectionProviderRejected,
    MlsProviderAdapterSelectionPqDraftRejected,
    MlsProviderAdapterSelectionStorageRejected,
    MlsProviderAdapterSelectionSupplyChainRejected,
    MlsKeyPackageAdmissionReady,
    MlsKeyPackageAdmissionGroupRejected,
    MlsKeyPackageAdmissionLifetimeRejected,
    MlsKeyPackageAdmissionSuiteMismatch,
    MlsKeyPackageAdmissionCredentialRejected,
    MlsKeyPackageAdmissionReplayRejected,
    MlsKeyPackageAdmissionPlaintextRejected,
    MlsKeyPackageConsumeStoreReady,
    MlsKeyPackageConsumeStoreAdmissionRejected,
    MlsKeyPackageConsumeStoreDuplicateRejected,
    MlsKeyPackageConsumeStoreBadShape,
    MlsKeyPackageConsumeStorePlaintextRejected,
    MlsWelcomeSendOutboxReady,
    MlsWelcomeSendOutboxConsumeRejected,
    MlsWelcomeSendOutboxDuplicateTransactionRejected,
    MlsWelcomeSendOutboxKeyPackageQueued,
    MlsWelcomeSendOutboxBadShape,
    MlsWelcomeSendOutboxPlaintextRejected,
    MlsMembershipTransactionReady,
    MlsMembershipTransactionBindingRejected,
    MlsMembershipTransactionStorageRejected,
    MlsMembershipTransactionDuplicateRejected,
    MlsMembershipTransactionPlaintextRejected,
    LocalStoreDatabaseSecurityReady,
    LocalStoreDatabaseSecurityPlaintextRejected,
    LocalStoreDatabaseSecurityWalRejected,
    LocalStoreDatabaseSecurityBackupRejected,
    LocalStoreDatabaseSecuritySecretLifecycleRejected,
    LocalStoreDatabaseAdapterSelectionReady,
    LocalStoreDatabaseAdapterSelectionLicenseRejected,
    LocalStoreDatabaseAdapterSelectionFipsRejected,
    LocalStoreDatabaseAdapterSelectionMigrationRejected,
    LocalStoreDatabaseAdapterSelectionSupplyChainRejected,
    MlsWelcomeAdmissionReady,
    MlsWelcomeAdmissionSecretsMissing,
    MlsWelcomeAdmissionTreeRejected,
    MlsWelcomeAdmissionConfirmationRejected,
    MlsWelcomeAdmissionTieBreakRejected,
    MlsWelcomeAdmissionReplayRejected,
    MlsWelcomeAdmissionPlaintextRejected,
    MlsWelcomeReplayStoreReady,
    MlsWelcomeReplayStoreAdmissionRejected,
    MlsWelcomeReplayStoreDuplicateRejected,
    MlsWelcomeReplayStoreKeyPackageReused,
    MlsWelcomeReplayStoreBadShape,
    MlsWelcomeReplayStorePlaintextRejected,
    MlsCommitAdmissionReady,
    MlsCommitAdmissionBadEpoch,
    MlsCommitAdmissionAuthRejected,
    MlsCommitAdmissionPathRejected,
    MlsCommitAdmissionTieBreakRejected,
    MlsCommitAdmissionReplayRejected,
    MlsCommitAdmissionPlaintextRejected,
    MlsCommitReplayStoreReady,
    MlsCommitReplayStoreAdmissionRejected,
    MlsCommitReplayStoreDuplicateRejected,
    MlsCommitReplayStoreLocalMemberRemoved,
    MlsCommitReplayStorePlaintextRejected,
    GroupMessageTranscriptReady,
    GroupMessageTranscriptSyncRequired,
    GroupMessageTranscriptRekeyRequired,
    GroupMessageTranscriptStoreBindingRejected,
    AnonymousCredentialIssuerTrustReady,
    AnonymousCredentialIssuerTrustTransparencyRequired,
    AnonymousCredentialIssuerTrustRevoked,
    AnonymousCredentialIssuerTrustPartitioningMetadataRejected,
    AnonymousCredentialIssuerTrustWitnessAuditRejected,
    AnonymousGroupMembershipProofReady,
    AnonymousGroupMembershipProofHighSecurityPqRequired,
    AnonymousGroupMembershipProofReplayRejected,
    AnonymousGroupMembershipProofRouteBindingRequired,
    AnonymousGroupMembershipProofPlaintextIdentityRejected,
    AnonymousRateLimitNullifierReady,
    AnonymousRateLimitNullifierReplayRejected,
    AnonymousRateLimitNullifierLimitExceeded,
    AnonymousRateLimitNullifierOpaqueStoreRequired,
    AnonymousNullifierStoreReady,
    AnonymousNullifierStoreReplayRejected,
    AnonymousNullifierStorePlaintextMetadataRejected,
    GroupRelayEnvelopeReady,
    GroupRelayEnvelopeTranscriptSyncRequired,
    GroupRelayEnvelopeTranscriptRekeyRequired,
    GroupRelayEnvelopeMissingDeliveryToken,
    GroupRelayEnvelopePlaintextMetadataRejected,
    LocalStoreProductionOpenReady,
    LocalStoreProductionOpenWalReplayRequired,
    LocalStoreProductionOpenPlaintextKeySlotForbidden,
    LocalStoreProductionOpenAppLockRequired,
    LocalStoreKeychainAndroidReady,
    LocalStoreKeychainUserAuthRequired,
    LocalStoreKeychainExportableSecretForbidden,
    LocalStoreKeychainDevelopmentBackendForbidden,
    ProductionStoreSessionHappyPath,
    ProductionStoreSessionKeychainRejected,
    ProductionStoreSessionWalReplayRequired,
    ProductionStoreSessionWriteRejected,
    PlatformLocalStoreAdapterDesktopReady,
    PlatformLocalStoreAdapterMobileHardwareRequired,
    PlatformLocalStoreAdapterPlaintextForbidden,
    PlatformLocalStoreAdapterAppLockRequired,
    ReceiveSessionHappyPath,
    ReceiveSessionAckRejected,
    ReceiveSessionOrderingGap,
    ReceiveSessionStoreWriteRejected,
    InboundSyncDeliveryReady,
    InboundSyncIdle,
    InboundSyncBootstrapBlocked,
    InboundSyncTransportOffline,
    InboundSyncPlaintextPreviewForbidden,
    AuthenticatedRelaySourceDeliveryReady,
    AuthenticatedRelaySourceIdle,
    AuthenticatedRelaySourceAuthRejected,
    AuthenticatedRelaySourcePlaintextForbidden,
    InboundSyncSessionHappyPath,
    InboundSyncSessionIdle,
    InboundSyncSessionSyncRejected,
    InboundSyncSessionReceiveRejected,
    MediaObjectStoreUploadReady,
    MediaObjectStorePlaintextRejected,
    MediaObjectStoreAutoDownloadRejected,
    MediaObjectStoreOversizeRejected,
    MediaUploadSessionHappyPath,
    MediaUploadSessionPlaintextRejected,
    MediaUploadSessionSealRejected,
    MediaUploadSessionStoreWriteRejected,
    MediaServiceAdapterReady,
    MediaServiceAdapterAuthMissing,
    MediaServiceAdapterPlaintextForbidden,
    MediaServiceAdapterDigestUnverified,
    MediaServiceUploadSessionHappyPath,
    MediaServiceUploadSessionMediaRejected,
    MediaServiceUploadSessionAuthRejected,
    MediaServiceUploadSessionDigestUnverified,
    MediaServiceDownloadReady,
    MediaServiceDownloadPlaintextPreviewRejected,
    MediaServiceDownloadAuthMissing,
    MediaServiceDownloadDigestUnverified,
    MediaDownloadSessionHappyPath,
    MediaDownloadSessionDownloadRejected,
    MediaDownloadSessionStoreWriteRejected,
    MediaDownloadSessionOpenRejected,
    MediaRetentionDeleteAndEvictReady,
    MediaRetentionRetainReady,
    MediaRetentionHoldRejected,
    MediaRetentionAuthMissing,
    MediaCleanupSessionHappyPath,
    MediaCleanupSessionRetainReady,
    MediaCleanupSessionRetentionRejected,
    MediaCleanupSessionCacheAbsent,
    MediaObjectIndexRemoteAndLocalReady,
    MediaObjectIndexAbsentUploadReady,
    MediaObjectIndexDeletePendingReady,
    MediaObjectIndexDeletedTerminal,
    MediaObjectIndexPlaintextMetadataRejected,
    MediaObjectIndexBadLifecycleRejected,
    MediaObjectIndexStoreWriteReady,
    MediaObjectIndexStoreIndexRejected,
    MediaObjectIndexStoreBadObjectRejected,
    MediaObjectIndexStoreDeletedSnapshot,
    MediaObjectIndexProductionOpenReady,
    MediaObjectIndexProductionOpenWalReplayRequired,
    MediaObjectIndexProductionOpenPlaintextMetadataForbidden,
    MediaObjectIndexProductionOpenNamespaceUnbound,
    IndexedMediaUploadSessionHappyPath,
    IndexedMediaUploadSessionServiceRejected,
    IndexedMediaUploadSessionIndexStoreRejected,
    IndexedMediaDownloadSessionHappyPath,
    IndexedMediaDownloadSessionManifestRejected,
    IndexedMediaDownloadSessionNotDownloadable,
    IndexedMediaDownloadSessionDownloadRejected,
    IndexedMediaCleanupSessionHappyPath,
    IndexedMediaCleanupSessionManifestRejected,
    IndexedMediaCleanupSessionNotCleanable,
    IndexedMediaCleanupSessionCleanupRejected,
    CryptoSealOpenRoundtrip,
    RelayDeliveryOnce,
    AiParticipantDraftAccepted,
    AiConnectorLocalDraftReady,
    AiConnectorRemoteForbidden,
    AiConnectorPlaintextBridgeRejected,
    AiConnectorRetentionRejected,
    AiConnectorUserSelectionRequired,
    BackendSessionHappyPath,
    BackendSessionBootstrapBlocked,
    BackendSessionRelayRejected,
    BackendSessionAiRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrototypeFixtureDescriptor {
    pub name: &'static str,
    pub fixture: PrototypeFixture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCommandDescriptor {
    pub name: &'static str,
    pub actor_kind: ActorKind,
    pub command_kind: PrototypeBackendCommandKind,
    pub result_fixture: PrototypeFixture,
}

pub const PLATFORM_BRIDGE_REQUEST_ID_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformBridgeOperation {
    PlatformFixture,
    PrototypeFixture,
    BackendCommand,
}

impl PlatformBridgeOperation {
    pub const fn code(self) -> i32 {
        match self {
            Self::PlatformFixture => 1,
            Self::PrototypeFixture => 2,
            Self::BackendCommand => 3,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::PlatformFixture => "platform_fixture",
            Self::PrototypeFixture => "prototype_fixture",
            Self::BackendCommand => "backend_command",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "platform_fixture" => Some(Self::PlatformFixture),
            "prototype_fixture" => Some(Self::PrototypeFixture),
            "backend_command" => Some(Self::BackendCommand),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformBridgeReason {
    Accepted,
    BadRequestIdLength,
    PlaintextPayloadForbidden,
    UnknownOperation,
    UnknownTarget,
}

impl PlatformBridgeReason {
    pub const fn code(self) -> i32 {
        match self {
            Self::Accepted => 0,
            Self::BadRequestIdLength => 1,
            Self::PlaintextPayloadForbidden => 2,
            Self::UnknownOperation => 3,
            Self::UnknownTarget => 4,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::BadRequestIdLength => "bad_request_id_length",
            Self::PlaintextPayloadForbidden => "plaintext_payload_forbidden",
            Self::UnknownOperation => "unknown_operation",
            Self::UnknownTarget => "unknown_target",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformBridgeRequest<'a> {
    pub request_id: &'a str,
    pub operation: PlatformBridgeOperation,
    pub target: &'a str,
    pub plaintext_payload_len: usize,
}

impl<'a> PlatformBridgeRequest<'a> {
    pub const fn new(
        request_id: &'a str,
        operation: PlatformBridgeOperation,
        target: &'a str,
        plaintext_payload_len: usize,
    ) -> Self {
        Self {
            request_id,
            operation,
            target,
            plaintext_payload_len,
        }
    }
}

pub const PLATFORM_FIXTURES: [PlatformFixtureDescriptor; 10] = [
    PlatformFixtureDescriptor {
        name: "bootstrap_accepted",
        fixture: PlatformFixture::BootstrapAccepted,
    },
    PlatformFixtureDescriptor {
        name: "bootstrap_sync_incomplete",
        fixture: PlatformFixture::BootstrapSyncIncomplete,
    },
    PlatformFixtureDescriptor {
        name: "bootstrap_recovery_required",
        fixture: PlatformFixture::BootstrapRecoveryRequired,
    },
    PlatformFixtureDescriptor {
        name: "outbound_send_accepted",
        fixture: PlatformFixture::OutboundSendAccepted,
    },
    PlatformFixtureDescriptor {
        name: "outbound_send_message_policy_rejected",
        fixture: PlatformFixture::OutboundSendMessagePolicyRejected,
    },
    PlatformFixtureDescriptor {
        name: "client_receive_accepted",
        fixture: PlatformFixture::ClientReceiveAccepted,
    },
    PlatformFixtureDescriptor {
        name: "client_receive_ordering_gap",
        fixture: PlatformFixture::ClientReceiveOrderingGap,
    },
    PlatformFixtureDescriptor {
        name: "client_receive_sender_trust_action",
        fixture: PlatformFixture::ClientReceiveSenderTrustAction,
    },
    PlatformFixtureDescriptor {
        name: "policy_ai_grant_rejected",
        fixture: PlatformFixture::PolicyAiGrantRejected,
    },
    PlatformFixtureDescriptor {
        name: "policy_ai_lifecycle_expired",
        fixture: PlatformFixture::PolicyAiLifecycleExpired,
    },
];

pub const PROTOTYPE_FIXTURES: [PrototypeFixtureDescriptor; 292] = [
    PrototypeFixtureDescriptor {
        name: "local_store_sealed_message",
        fixture: PrototypeFixture::LocalStoreSealedMessage,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_unlock_ready",
        fixture: PrototypeFixture::LocalStoreUnlockReady,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_unlock_app_lock_required",
        fixture: PrototypeFixture::LocalStoreUnlockAppLockRequired,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_unlock_recovery_required",
        fixture: PrototypeFixture::LocalStoreUnlockRecoveryRequired,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_unlock_plaintext_cache_forbidden",
        fixture: PrototypeFixture::LocalStoreUnlockPlaintextCacheForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "account_recovery_high_entropy_ready",
        fixture: PrototypeFixture::AccountRecoveryHighEntropyReady,
    },
    PrototypeFixtureDescriptor {
        name: "account_recovery_low_entropy_pin_forbidden",
        fixture: PrototypeFixture::AccountRecoveryLowEntropyPinForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "account_recovery_threshold_quorum_required",
        fixture: PrototypeFixture::AccountRecoveryThresholdQuorumRequired,
    },
    PrototypeFixtureDescriptor {
        name: "account_recovery_plaintext_backup_forbidden",
        fixture: PrototypeFixture::AccountRecoveryPlaintextBackupForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "account_recovery_key_rotation_required",
        fixture: PrototypeFixture::AccountRecoveryKeyRotationRequired,
    },
    PrototypeFixtureDescriptor {
        name: "secure_backup_restore_ready",
        fixture: PrototypeFixture::SecureBackupRestoreReady,
    },
    PrototypeFixtureDescriptor {
        name: "secure_backup_restore_recovery_rejected",
        fixture: PrototypeFixture::SecureBackupRestoreRecoveryRejected,
    },
    PrototypeFixtureDescriptor {
        name: "secure_backup_restore_plaintext_rejected",
        fixture: PrototypeFixture::SecureBackupRestorePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "secure_backup_restore_mls_rekey_rejected",
        fixture: PrototypeFixture::SecureBackupRestoreMlsRekeyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "secure_backup_restore_cloud_policy_rejected",
        fixture: PrototypeFixture::SecureBackupRestoreCloudPolicyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_chain_ready",
        fixture: PrototypeFixture::SealedAuditEventChainReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_chain_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditEventChainPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_chain_rollback_rejected",
        fixture: PrototypeFixture::SealedAuditEventChainRollbackRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_chain_witness_rejected",
        fixture: PrototypeFixture::SealedAuditEventChainWitnessRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_chain_binding_rejected",
        fixture: PrototypeFixture::SealedAuditEventChainBindingRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_store_ready",
        fixture: PrototypeFixture::SealedAuditEventStoreReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_store_chain_rejected",
        fixture: PrototypeFixture::SealedAuditEventStoreChainRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_store_duplicate_rejected",
        fixture: PrototypeFixture::SealedAuditEventStoreDuplicateRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_store_rollback_rejected",
        fixture: PrototypeFixture::SealedAuditEventStoreRollbackRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_event_store_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditEventStorePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_checkpoint_ready",
        fixture: PrototypeFixture::SealedAuditWitnessCheckpointReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_checkpoint_store_rejected",
        fixture: PrototypeFixture::SealedAuditWitnessCheckpointStoreRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_checkpoint_quorum_rejected",
        fixture: PrototypeFixture::SealedAuditWitnessCheckpointQuorumRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_checkpoint_split_view_rejected",
        fixture: PrototypeFixture::SealedAuditWitnessCheckpointSplitViewRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_checkpoint_privacy_rejected",
        fixture: PrototypeFixture::SealedAuditWitnessCheckpointPrivacyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_client_ready",
        fixture: PrototypeFixture::SealedAuditWitnessClientReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_client_conflict",
        fixture: PrototypeFixture::SealedAuditWitnessClientConflict,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_client_unavailable",
        fixture: PrototypeFixture::SealedAuditWitnessClientUnavailable,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_client_policy_rejected",
        fixture: PrototypeFixture::SealedAuditWitnessClientPolicyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_witness_client_monitor_privacy_rejected",
        fixture: PrototypeFixture::SealedAuditWitnessClientMonitorPrivacyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_bundle_ready",
        fixture: PrototypeFixture::SealedAuditProofBundleReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_bundle_client_rejected",
        fixture: PrototypeFixture::SealedAuditProofBundleClientRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_bundle_stale_witness",
        fixture: PrototypeFixture::SealedAuditProofBundleStaleWitness,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_bundle_policy_rejected",
        fixture: PrototypeFixture::SealedAuditProofBundlePolicyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_bundle_privacy_rejected",
        fixture: PrototypeFixture::SealedAuditProofBundlePrivacyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_cache_ready",
        fixture: PrototypeFixture::SealedAuditProofCacheReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_cache_bundle_rejected",
        fixture: PrototypeFixture::SealedAuditProofCacheBundleRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_cache_duplicate_rejected",
        fixture: PrototypeFixture::SealedAuditProofCacheDuplicateRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_cache_policy_stale",
        fixture: PrototypeFixture::SealedAuditProofCachePolicyStale,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_proof_cache_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditProofCachePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_verifier_policy_ready",
        fixture: PrototypeFixture::SealedAuditVerifierPolicyReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_verifier_policy_expired",
        fixture: PrototypeFixture::SealedAuditVerifierPolicyExpired,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_verifier_policy_key_rotation_required",
        fixture: PrototypeFixture::SealedAuditVerifierPolicyKeyRotationRequired,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_verifier_policy_monitor_privacy_rejected",
        fixture: PrototypeFixture::SealedAuditVerifierPolicyMonitorPrivacyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_verifier_policy_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditVerifierPolicyPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_incident_evidence_ready",
        fixture: PrototypeFixture::SealedAuditIncidentEvidenceReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_incident_evidence_policy_rejected",
        fixture: PrototypeFixture::SealedAuditIncidentEvidencePolicyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_incident_evidence_missing_proof_report",
        fixture: PrototypeFixture::SealedAuditIncidentEvidenceMissingProofReport,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_incident_evidence_split_view",
        fixture: PrototypeFixture::SealedAuditIncidentEvidenceSplitView,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_incident_evidence_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditIncidentEvidencePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_recovery_export_ready",
        fixture: PrototypeFixture::SealedAuditRecoveryExportReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_recovery_export_incident_rejected",
        fixture: PrototypeFixture::SealedAuditRecoveryExportIncidentRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_recovery_export_quorum_required",
        fixture: PrototypeFixture::SealedAuditRecoveryExportQuorumRequired,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_recovery_export_rollback_rejected",
        fixture: PrototypeFixture::SealedAuditRecoveryExportRollbackRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_recovery_export_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditRecoveryExportPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_database_adapter_ready",
        fixture: PrototypeFixture::SealedAuditDatabaseAdapterReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_database_adapter_encryption_rejected",
        fixture: PrototypeFixture::SealedAuditDatabaseAdapterEncryptionRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_database_adapter_append_only_rejected",
        fixture: PrototypeFixture::SealedAuditDatabaseAdapterAppendOnlyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_transport_ready",
        fixture: PrototypeFixture::SealedAuditPrivateReportTransportReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_transport_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportTransportPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_outbox_ready",
        fixture: PrototypeFixture::SealedAuditPrivateReportOutboxReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_outbox_transport_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportOutboxTransportRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_outbox_replay_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportOutboxReplayRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_outbox_rate_limit_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportOutboxRateLimitRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_outbox_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportOutboxPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_receipt_ready",
        fixture: PrototypeFixture::SealedAuditPrivateReportReceiptReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_receipt_outbox_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportReceiptOutboxRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_receipt_missing",
        fixture: PrototypeFixture::SealedAuditPrivateReportReceiptMissing,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_receipt_transparency_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportReceiptTransparencyRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_receipt_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportReceiptPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_reconciliation_ready",
        fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_reconciliation_receipt_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationReceiptRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_reconciliation_retry_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationRetryRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_reconciliation_false_delivery_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationFalseDeliveryRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_reconciliation_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_gateway_evidence_ready",
        fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceReady,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_gateway_evidence_reconciliation_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceReconciliationRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_gateway_evidence_unavailable_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceUnavailableRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_gateway_evidence_accountability_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceAccountabilityRejected,
    },
    PrototypeFixtureDescriptor {
        name: "sealed_audit_private_report_gateway_evidence_plaintext_rejected",
        fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidencePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "group_chat_mls_ready",
        fixture: PrototypeFixture::GroupChatMlsReady,
    },
    PrototypeFixtureDescriptor {
        name: "group_chat_mls_setup_required",
        fixture: PrototypeFixture::GroupChatMlsSetupRequired,
    },
    PrototypeFixtureDescriptor {
        name: "group_chat_membership_sync_required",
        fixture: PrototypeFixture::GroupChatMembershipSyncRequired,
    },
    PrototypeFixtureDescriptor {
        name: "group_chat_plaintext_metadata_forbidden",
        fixture: PrototypeFixture::GroupChatPlaintextMetadataForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "group_chat_high_security_mls_required",
        fixture: PrototypeFixture::GroupChatHighSecurityMlsRequired,
    },
    PrototypeFixtureDescriptor {
        name: "group_chat_high_security_pq_required",
        fixture: PrototypeFixture::GroupChatHighSecurityPqRequired,
    },
    PrototypeFixtureDescriptor {
        name: "group_chat_mls_provider_security_required",
        fixture: PrototypeFixture::GroupChatMlsProviderSecurityRequired,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_store_ready",
        fixture: PrototypeFixture::MlsProviderEvidenceStoreReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_store_gate_rejected",
        fixture: PrototypeFixture::MlsProviderEvidenceStoreGateRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_store_duplicate_rejected",
        fixture: PrototypeFixture::MlsProviderEvidenceStoreDuplicateRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_store_plaintext_rejected",
        fixture: PrototypeFixture::MlsProviderEvidenceStorePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_use_ready",
        fixture: PrototypeFixture::MlsProviderEvidenceUseReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_use_missing",
        fixture: PrototypeFixture::MlsProviderEvidenceUseMissing,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_use_expired",
        fixture: PrototypeFixture::MlsProviderEvidenceUseExpired,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_use_suite_mismatch",
        fixture: PrototypeFixture::MlsProviderEvidenceUseSuiteMismatch,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_evidence_use_plaintext_rejected",
        fixture: PrototypeFixture::MlsProviderEvidenceUsePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_adapter_selection_ready",
        fixture: PrototypeFixture::MlsProviderAdapterSelectionReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_adapter_selection_provider_rejected",
        fixture: PrototypeFixture::MlsProviderAdapterSelectionProviderRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_adapter_selection_pq_draft_rejected",
        fixture: PrototypeFixture::MlsProviderAdapterSelectionPqDraftRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_adapter_selection_storage_rejected",
        fixture: PrototypeFixture::MlsProviderAdapterSelectionStorageRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_provider_adapter_selection_supply_chain_rejected",
        fixture: PrototypeFixture::MlsProviderAdapterSelectionSupplyChainRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_admission_ready",
        fixture: PrototypeFixture::MlsKeyPackageAdmissionReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_admission_group_rejected",
        fixture: PrototypeFixture::MlsKeyPackageAdmissionGroupRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_admission_lifetime_rejected",
        fixture: PrototypeFixture::MlsKeyPackageAdmissionLifetimeRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_admission_suite_mismatch",
        fixture: PrototypeFixture::MlsKeyPackageAdmissionSuiteMismatch,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_admission_credential_rejected",
        fixture: PrototypeFixture::MlsKeyPackageAdmissionCredentialRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_admission_replay_rejected",
        fixture: PrototypeFixture::MlsKeyPackageAdmissionReplayRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_admission_plaintext_rejected",
        fixture: PrototypeFixture::MlsKeyPackageAdmissionPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_consume_store_ready",
        fixture: PrototypeFixture::MlsKeyPackageConsumeStoreReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_consume_store_admission_rejected",
        fixture: PrototypeFixture::MlsKeyPackageConsumeStoreAdmissionRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_consume_store_duplicate_rejected",
        fixture: PrototypeFixture::MlsKeyPackageConsumeStoreDuplicateRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_consume_store_bad_shape",
        fixture: PrototypeFixture::MlsKeyPackageConsumeStoreBadShape,
    },
    PrototypeFixtureDescriptor {
        name: "mls_key_package_consume_store_plaintext_rejected",
        fixture: PrototypeFixture::MlsKeyPackageConsumeStorePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_send_outbox_ready",
        fixture: PrototypeFixture::MlsWelcomeSendOutboxReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_send_outbox_consume_rejected",
        fixture: PrototypeFixture::MlsWelcomeSendOutboxConsumeRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_send_outbox_duplicate_transaction_rejected",
        fixture: PrototypeFixture::MlsWelcomeSendOutboxDuplicateTransactionRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_send_outbox_key_package_queued",
        fixture: PrototypeFixture::MlsWelcomeSendOutboxKeyPackageQueued,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_send_outbox_bad_shape",
        fixture: PrototypeFixture::MlsWelcomeSendOutboxBadShape,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_send_outbox_plaintext_rejected",
        fixture: PrototypeFixture::MlsWelcomeSendOutboxPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_membership_transaction_ready",
        fixture: PrototypeFixture::MlsMembershipTransactionReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_membership_transaction_binding_rejected",
        fixture: PrototypeFixture::MlsMembershipTransactionBindingRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_membership_transaction_storage_rejected",
        fixture: PrototypeFixture::MlsMembershipTransactionStorageRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_membership_transaction_duplicate_rejected",
        fixture: PrototypeFixture::MlsMembershipTransactionDuplicateRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_membership_transaction_plaintext_rejected",
        fixture: PrototypeFixture::MlsMembershipTransactionPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_security_ready",
        fixture: PrototypeFixture::LocalStoreDatabaseSecurityReady,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_security_plaintext_rejected",
        fixture: PrototypeFixture::LocalStoreDatabaseSecurityPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_security_wal_rejected",
        fixture: PrototypeFixture::LocalStoreDatabaseSecurityWalRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_security_backup_rejected",
        fixture: PrototypeFixture::LocalStoreDatabaseSecurityBackupRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_security_secret_lifecycle_rejected",
        fixture: PrototypeFixture::LocalStoreDatabaseSecuritySecretLifecycleRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_adapter_selection_ready",
        fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionReady,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_adapter_selection_license_rejected",
        fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionLicenseRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_adapter_selection_fips_rejected",
        fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionFipsRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_adapter_selection_migration_rejected",
        fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionMigrationRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_database_adapter_selection_supply_chain_rejected",
        fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionSupplyChainRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_admission_ready",
        fixture: PrototypeFixture::MlsWelcomeAdmissionReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_admission_secrets_missing",
        fixture: PrototypeFixture::MlsWelcomeAdmissionSecretsMissing,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_admission_tree_rejected",
        fixture: PrototypeFixture::MlsWelcomeAdmissionTreeRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_admission_confirmation_rejected",
        fixture: PrototypeFixture::MlsWelcomeAdmissionConfirmationRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_admission_tie_break_rejected",
        fixture: PrototypeFixture::MlsWelcomeAdmissionTieBreakRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_admission_replay_rejected",
        fixture: PrototypeFixture::MlsWelcomeAdmissionReplayRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_admission_plaintext_rejected",
        fixture: PrototypeFixture::MlsWelcomeAdmissionPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_replay_store_ready",
        fixture: PrototypeFixture::MlsWelcomeReplayStoreReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_replay_store_admission_rejected",
        fixture: PrototypeFixture::MlsWelcomeReplayStoreAdmissionRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_replay_store_duplicate_rejected",
        fixture: PrototypeFixture::MlsWelcomeReplayStoreDuplicateRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_replay_store_key_package_reused",
        fixture: PrototypeFixture::MlsWelcomeReplayStoreKeyPackageReused,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_replay_store_bad_shape",
        fixture: PrototypeFixture::MlsWelcomeReplayStoreBadShape,
    },
    PrototypeFixtureDescriptor {
        name: "mls_welcome_replay_store_plaintext_rejected",
        fixture: PrototypeFixture::MlsWelcomeReplayStorePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_admission_ready",
        fixture: PrototypeFixture::MlsCommitAdmissionReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_admission_bad_epoch",
        fixture: PrototypeFixture::MlsCommitAdmissionBadEpoch,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_admission_auth_rejected",
        fixture: PrototypeFixture::MlsCommitAdmissionAuthRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_admission_path_rejected",
        fixture: PrototypeFixture::MlsCommitAdmissionPathRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_admission_tie_break_rejected",
        fixture: PrototypeFixture::MlsCommitAdmissionTieBreakRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_admission_replay_rejected",
        fixture: PrototypeFixture::MlsCommitAdmissionReplayRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_admission_plaintext_rejected",
        fixture: PrototypeFixture::MlsCommitAdmissionPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_replay_store_ready",
        fixture: PrototypeFixture::MlsCommitReplayStoreReady,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_replay_store_admission_rejected",
        fixture: PrototypeFixture::MlsCommitReplayStoreAdmissionRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_replay_store_duplicate_rejected",
        fixture: PrototypeFixture::MlsCommitReplayStoreDuplicateRejected,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_replay_store_local_member_removed",
        fixture: PrototypeFixture::MlsCommitReplayStoreLocalMemberRemoved,
    },
    PrototypeFixtureDescriptor {
        name: "mls_commit_replay_store_plaintext_rejected",
        fixture: PrototypeFixture::MlsCommitReplayStorePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "group_message_transcript_ready",
        fixture: PrototypeFixture::GroupMessageTranscriptReady,
    },
    PrototypeFixtureDescriptor {
        name: "group_message_transcript_sync_required",
        fixture: PrototypeFixture::GroupMessageTranscriptSyncRequired,
    },
    PrototypeFixtureDescriptor {
        name: "group_message_transcript_rekey_required",
        fixture: PrototypeFixture::GroupMessageTranscriptRekeyRequired,
    },
    PrototypeFixtureDescriptor {
        name: "group_message_transcript_store_binding_rejected",
        fixture: PrototypeFixture::GroupMessageTranscriptStoreBindingRejected,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_credential_issuer_trust_ready",
        fixture: PrototypeFixture::AnonymousCredentialIssuerTrustReady,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_credential_issuer_trust_transparency_required",
        fixture: PrototypeFixture::AnonymousCredentialIssuerTrustTransparencyRequired,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_credential_issuer_trust_revoked",
        fixture: PrototypeFixture::AnonymousCredentialIssuerTrustRevoked,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_credential_issuer_trust_partitioning_metadata_rejected",
        fixture: PrototypeFixture::AnonymousCredentialIssuerTrustPartitioningMetadataRejected,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_credential_issuer_trust_witness_audit_rejected",
        fixture: PrototypeFixture::AnonymousCredentialIssuerTrustWitnessAuditRejected,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_group_membership_proof_ready",
        fixture: PrototypeFixture::AnonymousGroupMembershipProofReady,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_group_membership_proof_high_security_pq_required",
        fixture: PrototypeFixture::AnonymousGroupMembershipProofHighSecurityPqRequired,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_group_membership_proof_replay_rejected",
        fixture: PrototypeFixture::AnonymousGroupMembershipProofReplayRejected,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_group_membership_proof_route_binding_required",
        fixture: PrototypeFixture::AnonymousGroupMembershipProofRouteBindingRequired,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_group_membership_proof_plaintext_identity_rejected",
        fixture: PrototypeFixture::AnonymousGroupMembershipProofPlaintextIdentityRejected,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_rate_limit_nullifier_ready",
        fixture: PrototypeFixture::AnonymousRateLimitNullifierReady,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_rate_limit_nullifier_replay_rejected",
        fixture: PrototypeFixture::AnonymousRateLimitNullifierReplayRejected,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_rate_limit_nullifier_limit_exceeded",
        fixture: PrototypeFixture::AnonymousRateLimitNullifierLimitExceeded,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_rate_limit_nullifier_opaque_store_required",
        fixture: PrototypeFixture::AnonymousRateLimitNullifierOpaqueStoreRequired,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_nullifier_store_ready",
        fixture: PrototypeFixture::AnonymousNullifierStoreReady,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_nullifier_store_replay_rejected",
        fixture: PrototypeFixture::AnonymousNullifierStoreReplayRejected,
    },
    PrototypeFixtureDescriptor {
        name: "anonymous_nullifier_store_plaintext_metadata_rejected",
        fixture: PrototypeFixture::AnonymousNullifierStorePlaintextMetadataRejected,
    },
    PrototypeFixtureDescriptor {
        name: "group_relay_envelope_ready",
        fixture: PrototypeFixture::GroupRelayEnvelopeReady,
    },
    PrototypeFixtureDescriptor {
        name: "group_relay_envelope_transcript_sync_required",
        fixture: PrototypeFixture::GroupRelayEnvelopeTranscriptSyncRequired,
    },
    PrototypeFixtureDescriptor {
        name: "group_relay_envelope_transcript_rekey_required",
        fixture: PrototypeFixture::GroupRelayEnvelopeTranscriptRekeyRequired,
    },
    PrototypeFixtureDescriptor {
        name: "group_relay_envelope_missing_delivery_token",
        fixture: PrototypeFixture::GroupRelayEnvelopeMissingDeliveryToken,
    },
    PrototypeFixtureDescriptor {
        name: "group_relay_envelope_plaintext_metadata_rejected",
        fixture: PrototypeFixture::GroupRelayEnvelopePlaintextMetadataRejected,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_production_open_ready",
        fixture: PrototypeFixture::LocalStoreProductionOpenReady,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_production_open_wal_replay_required",
        fixture: PrototypeFixture::LocalStoreProductionOpenWalReplayRequired,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_production_open_plaintext_key_slot_forbidden",
        fixture: PrototypeFixture::LocalStoreProductionOpenPlaintextKeySlotForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_production_open_app_lock_required",
        fixture: PrototypeFixture::LocalStoreProductionOpenAppLockRequired,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_keychain_android_ready",
        fixture: PrototypeFixture::LocalStoreKeychainAndroidReady,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_keychain_user_auth_required",
        fixture: PrototypeFixture::LocalStoreKeychainUserAuthRequired,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_keychain_exportable_secret_forbidden",
        fixture: PrototypeFixture::LocalStoreKeychainExportableSecretForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "local_store_keychain_development_backend_forbidden",
        fixture: PrototypeFixture::LocalStoreKeychainDevelopmentBackendForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "production_store_session_happy_path",
        fixture: PrototypeFixture::ProductionStoreSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "production_store_session_keychain_rejected",
        fixture: PrototypeFixture::ProductionStoreSessionKeychainRejected,
    },
    PrototypeFixtureDescriptor {
        name: "production_store_session_wal_replay_required",
        fixture: PrototypeFixture::ProductionStoreSessionWalReplayRequired,
    },
    PrototypeFixtureDescriptor {
        name: "production_store_session_write_rejected",
        fixture: PrototypeFixture::ProductionStoreSessionWriteRejected,
    },
    PrototypeFixtureDescriptor {
        name: "platform_local_store_adapter_desktop_ready",
        fixture: PrototypeFixture::PlatformLocalStoreAdapterDesktopReady,
    },
    PrototypeFixtureDescriptor {
        name: "platform_local_store_adapter_mobile_hardware_required",
        fixture: PrototypeFixture::PlatformLocalStoreAdapterMobileHardwareRequired,
    },
    PrototypeFixtureDescriptor {
        name: "platform_local_store_adapter_plaintext_forbidden",
        fixture: PrototypeFixture::PlatformLocalStoreAdapterPlaintextForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "platform_local_store_adapter_app_lock_required",
        fixture: PrototypeFixture::PlatformLocalStoreAdapterAppLockRequired,
    },
    PrototypeFixtureDescriptor {
        name: "receive_session_happy_path",
        fixture: PrototypeFixture::ReceiveSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "receive_session_ack_rejected",
        fixture: PrototypeFixture::ReceiveSessionAckRejected,
    },
    PrototypeFixtureDescriptor {
        name: "receive_session_ordering_gap",
        fixture: PrototypeFixture::ReceiveSessionOrderingGap,
    },
    PrototypeFixtureDescriptor {
        name: "receive_session_store_write_rejected",
        fixture: PrototypeFixture::ReceiveSessionStoreWriteRejected,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_delivery_ready",
        fixture: PrototypeFixture::InboundSyncDeliveryReady,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_idle",
        fixture: PrototypeFixture::InboundSyncIdle,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_bootstrap_blocked",
        fixture: PrototypeFixture::InboundSyncBootstrapBlocked,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_transport_offline",
        fixture: PrototypeFixture::InboundSyncTransportOffline,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_plaintext_preview_forbidden",
        fixture: PrototypeFixture::InboundSyncPlaintextPreviewForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "authenticated_relay_source_delivery_ready",
        fixture: PrototypeFixture::AuthenticatedRelaySourceDeliveryReady,
    },
    PrototypeFixtureDescriptor {
        name: "authenticated_relay_source_idle",
        fixture: PrototypeFixture::AuthenticatedRelaySourceIdle,
    },
    PrototypeFixtureDescriptor {
        name: "authenticated_relay_source_auth_rejected",
        fixture: PrototypeFixture::AuthenticatedRelaySourceAuthRejected,
    },
    PrototypeFixtureDescriptor {
        name: "authenticated_relay_source_plaintext_forbidden",
        fixture: PrototypeFixture::AuthenticatedRelaySourcePlaintextForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_session_happy_path",
        fixture: PrototypeFixture::InboundSyncSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_session_idle",
        fixture: PrototypeFixture::InboundSyncSessionIdle,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_session_sync_rejected",
        fixture: PrototypeFixture::InboundSyncSessionSyncRejected,
    },
    PrototypeFixtureDescriptor {
        name: "inbound_sync_session_receive_rejected",
        fixture: PrototypeFixture::InboundSyncSessionReceiveRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_store_upload_ready",
        fixture: PrototypeFixture::MediaObjectStoreUploadReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_store_plaintext_rejected",
        fixture: PrototypeFixture::MediaObjectStorePlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_store_auto_download_rejected",
        fixture: PrototypeFixture::MediaObjectStoreAutoDownloadRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_store_oversize_rejected",
        fixture: PrototypeFixture::MediaObjectStoreOversizeRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_upload_session_happy_path",
        fixture: PrototypeFixture::MediaUploadSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "media_upload_session_plaintext_rejected",
        fixture: PrototypeFixture::MediaUploadSessionPlaintextRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_upload_session_seal_rejected",
        fixture: PrototypeFixture::MediaUploadSessionSealRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_upload_session_store_write_rejected",
        fixture: PrototypeFixture::MediaUploadSessionStoreWriteRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_adapter_ready",
        fixture: PrototypeFixture::MediaServiceAdapterReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_adapter_auth_missing",
        fixture: PrototypeFixture::MediaServiceAdapterAuthMissing,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_adapter_plaintext_forbidden",
        fixture: PrototypeFixture::MediaServiceAdapterPlaintextForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_adapter_digest_unverified",
        fixture: PrototypeFixture::MediaServiceAdapterDigestUnverified,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_upload_session_happy_path",
        fixture: PrototypeFixture::MediaServiceUploadSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_upload_session_media_rejected",
        fixture: PrototypeFixture::MediaServiceUploadSessionMediaRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_upload_session_auth_rejected",
        fixture: PrototypeFixture::MediaServiceUploadSessionAuthRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_upload_session_digest_unverified",
        fixture: PrototypeFixture::MediaServiceUploadSessionDigestUnverified,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_download_ready",
        fixture: PrototypeFixture::MediaServiceDownloadReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_download_plaintext_preview_rejected",
        fixture: PrototypeFixture::MediaServiceDownloadPlaintextPreviewRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_download_auth_missing",
        fixture: PrototypeFixture::MediaServiceDownloadAuthMissing,
    },
    PrototypeFixtureDescriptor {
        name: "media_service_download_digest_unverified",
        fixture: PrototypeFixture::MediaServiceDownloadDigestUnverified,
    },
    PrototypeFixtureDescriptor {
        name: "media_download_session_happy_path",
        fixture: PrototypeFixture::MediaDownloadSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "media_download_session_download_rejected",
        fixture: PrototypeFixture::MediaDownloadSessionDownloadRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_download_session_store_write_rejected",
        fixture: PrototypeFixture::MediaDownloadSessionStoreWriteRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_download_session_open_rejected",
        fixture: PrototypeFixture::MediaDownloadSessionOpenRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_retention_delete_and_evict_ready",
        fixture: PrototypeFixture::MediaRetentionDeleteAndEvictReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_retention_retain_ready",
        fixture: PrototypeFixture::MediaRetentionRetainReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_retention_hold_rejected",
        fixture: PrototypeFixture::MediaRetentionHoldRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_retention_auth_missing",
        fixture: PrototypeFixture::MediaRetentionAuthMissing,
    },
    PrototypeFixtureDescriptor {
        name: "media_cleanup_session_happy_path",
        fixture: PrototypeFixture::MediaCleanupSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "media_cleanup_session_retain_ready",
        fixture: PrototypeFixture::MediaCleanupSessionRetainReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_cleanup_session_retention_rejected",
        fixture: PrototypeFixture::MediaCleanupSessionRetentionRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_cleanup_session_cache_absent",
        fixture: PrototypeFixture::MediaCleanupSessionCacheAbsent,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_remote_and_local_ready",
        fixture: PrototypeFixture::MediaObjectIndexRemoteAndLocalReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_absent_upload_ready",
        fixture: PrototypeFixture::MediaObjectIndexAbsentUploadReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_delete_pending_ready",
        fixture: PrototypeFixture::MediaObjectIndexDeletePendingReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_deleted_terminal",
        fixture: PrototypeFixture::MediaObjectIndexDeletedTerminal,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_plaintext_metadata_rejected",
        fixture: PrototypeFixture::MediaObjectIndexPlaintextMetadataRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_bad_lifecycle_rejected",
        fixture: PrototypeFixture::MediaObjectIndexBadLifecycleRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_store_write_ready",
        fixture: PrototypeFixture::MediaObjectIndexStoreWriteReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_store_index_rejected",
        fixture: PrototypeFixture::MediaObjectIndexStoreIndexRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_store_bad_object_rejected",
        fixture: PrototypeFixture::MediaObjectIndexStoreBadObjectRejected,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_store_deleted_snapshot",
        fixture: PrototypeFixture::MediaObjectIndexStoreDeletedSnapshot,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_production_open_ready",
        fixture: PrototypeFixture::MediaObjectIndexProductionOpenReady,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_production_open_wal_replay_required",
        fixture: PrototypeFixture::MediaObjectIndexProductionOpenWalReplayRequired,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_production_open_plaintext_metadata_forbidden",
        fixture: PrototypeFixture::MediaObjectIndexProductionOpenPlaintextMetadataForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "media_object_index_production_open_namespace_unbound",
        fixture: PrototypeFixture::MediaObjectIndexProductionOpenNamespaceUnbound,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_upload_session_happy_path",
        fixture: PrototypeFixture::IndexedMediaUploadSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_upload_session_service_rejected",
        fixture: PrototypeFixture::IndexedMediaUploadSessionServiceRejected,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_upload_session_index_store_rejected",
        fixture: PrototypeFixture::IndexedMediaUploadSessionIndexStoreRejected,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_download_session_happy_path",
        fixture: PrototypeFixture::IndexedMediaDownloadSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_download_session_manifest_rejected",
        fixture: PrototypeFixture::IndexedMediaDownloadSessionManifestRejected,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_download_session_not_downloadable",
        fixture: PrototypeFixture::IndexedMediaDownloadSessionNotDownloadable,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_download_session_download_rejected",
        fixture: PrototypeFixture::IndexedMediaDownloadSessionDownloadRejected,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_cleanup_session_happy_path",
        fixture: PrototypeFixture::IndexedMediaCleanupSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_cleanup_session_manifest_rejected",
        fixture: PrototypeFixture::IndexedMediaCleanupSessionManifestRejected,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_cleanup_session_not_cleanable",
        fixture: PrototypeFixture::IndexedMediaCleanupSessionNotCleanable,
    },
    PrototypeFixtureDescriptor {
        name: "indexed_media_cleanup_session_cleanup_rejected",
        fixture: PrototypeFixture::IndexedMediaCleanupSessionCleanupRejected,
    },
    PrototypeFixtureDescriptor {
        name: "crypto_seal_open_roundtrip",
        fixture: PrototypeFixture::CryptoSealOpenRoundtrip,
    },
    PrototypeFixtureDescriptor {
        name: "relay_delivery_once",
        fixture: PrototypeFixture::RelayDeliveryOnce,
    },
    PrototypeFixtureDescriptor {
        name: "ai_participant_draft_accepted",
        fixture: PrototypeFixture::AiParticipantDraftAccepted,
    },
    PrototypeFixtureDescriptor {
        name: "ai_connector_local_draft_ready",
        fixture: PrototypeFixture::AiConnectorLocalDraftReady,
    },
    PrototypeFixtureDescriptor {
        name: "ai_connector_remote_forbidden",
        fixture: PrototypeFixture::AiConnectorRemoteForbidden,
    },
    PrototypeFixtureDescriptor {
        name: "ai_connector_plaintext_bridge_rejected",
        fixture: PrototypeFixture::AiConnectorPlaintextBridgeRejected,
    },
    PrototypeFixtureDescriptor {
        name: "ai_connector_retention_rejected",
        fixture: PrototypeFixture::AiConnectorRetentionRejected,
    },
    PrototypeFixtureDescriptor {
        name: "ai_connector_user_selection_required",
        fixture: PrototypeFixture::AiConnectorUserSelectionRequired,
    },
    PrototypeFixtureDescriptor {
        name: "backend_session_happy_path",
        fixture: PrototypeFixture::BackendSessionHappyPath,
    },
    PrototypeFixtureDescriptor {
        name: "backend_session_bootstrap_blocked",
        fixture: PrototypeFixture::BackendSessionBootstrapBlocked,
    },
    PrototypeFixtureDescriptor {
        name: "backend_session_relay_rejected",
        fixture: PrototypeFixture::BackendSessionRelayRejected,
    },
    PrototypeFixtureDescriptor {
        name: "backend_session_ai_rejected",
        fixture: PrototypeFixture::BackendSessionAiRejected,
    },
];

pub const BACKEND_COMMANDS: [BackendCommandDescriptor; 259] = [
    BackendCommandDescriptor {
        name: "run_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSessionHappyPath,
        result_fixture: PrototypeFixture::BackendSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_session_bootstrap_blocked",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSessionBootstrapBlocked,
        result_fixture: PrototypeFixture::BackendSessionBootstrapBlocked,
    },
    BackendCommandDescriptor {
        name: "run_session_relay_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSessionRelayRejected,
        result_fixture: PrototypeFixture::BackendSessionRelayRejected,
    },
    BackendCommandDescriptor {
        name: "run_session_ai_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSessionAiRejected,
        result_fixture: PrototypeFixture::BackendSessionAiRejected,
    },
    BackendCommandDescriptor {
        name: "local_ai_draft_assist",
        actor_kind: ActorKind::LocalAi,
        command_kind: PrototypeBackendCommandKind::RunLocalAiDraftAssist,
        result_fixture: PrototypeFixture::AiParticipantDraftAccepted,
    },
    BackendCommandDescriptor {
        name: "run_production_store_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunProductionStoreSessionHappyPath,
        result_fixture: PrototypeFixture::ProductionStoreSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_production_store_session_keychain_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunProductionStoreSessionKeychainRejected,
        result_fixture: PrototypeFixture::ProductionStoreSessionKeychainRejected,
    },
    BackendCommandDescriptor {
        name: "run_production_store_session_wal_replay_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunProductionStoreSessionWalReplayRequired,
        result_fixture: PrototypeFixture::ProductionStoreSessionWalReplayRequired,
    },
    BackendCommandDescriptor {
        name: "run_production_store_session_write_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunProductionStoreSessionWriteRejected,
        result_fixture: PrototypeFixture::ProductionStoreSessionWriteRejected,
    },
    BackendCommandDescriptor {
        name: "run_platform_local_store_adapter_desktop_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunPlatformLocalStoreAdapterDesktopReady,
        result_fixture: PrototypeFixture::PlatformLocalStoreAdapterDesktopReady,
    },
    BackendCommandDescriptor {
        name: "run_platform_local_store_adapter_mobile_hardware_required",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunPlatformLocalStoreAdapterMobileHardwareRequired,
        result_fixture: PrototypeFixture::PlatformLocalStoreAdapterMobileHardwareRequired,
    },
    BackendCommandDescriptor {
        name: "run_platform_local_store_adapter_plaintext_forbidden",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunPlatformLocalStoreAdapterPlaintextForbidden,
        result_fixture: PrototypeFixture::PlatformLocalStoreAdapterPlaintextForbidden,
    },
    BackendCommandDescriptor {
        name: "run_platform_local_store_adapter_app_lock_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunPlatformLocalStoreAdapterAppLockRequired,
        result_fixture: PrototypeFixture::PlatformLocalStoreAdapterAppLockRequired,
    },
    BackendCommandDescriptor {
        name: "run_receive_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunReceiveSessionHappyPath,
        result_fixture: PrototypeFixture::ReceiveSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_receive_session_ack_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunReceiveSessionAckRejected,
        result_fixture: PrototypeFixture::ReceiveSessionAckRejected,
    },
    BackendCommandDescriptor {
        name: "run_receive_session_ordering_gap",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunReceiveSessionOrderingGap,
        result_fixture: PrototypeFixture::ReceiveSessionOrderingGap,
    },
    BackendCommandDescriptor {
        name: "run_receive_session_store_write_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunReceiveSessionStoreWriteRejected,
        result_fixture: PrototypeFixture::ReceiveSessionStoreWriteRejected,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_delivery_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncDeliveryReady,
        result_fixture: PrototypeFixture::InboundSyncDeliveryReady,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_idle",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncIdle,
        result_fixture: PrototypeFixture::InboundSyncIdle,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_bootstrap_blocked",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncBootstrapBlocked,
        result_fixture: PrototypeFixture::InboundSyncBootstrapBlocked,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_transport_offline",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncTransportOffline,
        result_fixture: PrototypeFixture::InboundSyncTransportOffline,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_plaintext_preview_forbidden",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncPlaintextPreviewForbidden,
        result_fixture: PrototypeFixture::InboundSyncPlaintextPreviewForbidden,
    },
    BackendCommandDescriptor {
        name: "run_authenticated_relay_source_delivery_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAuthenticatedRelaySourceDeliveryReady,
        result_fixture: PrototypeFixture::AuthenticatedRelaySourceDeliveryReady,
    },
    BackendCommandDescriptor {
        name: "run_authenticated_relay_source_idle",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAuthenticatedRelaySourceIdle,
        result_fixture: PrototypeFixture::AuthenticatedRelaySourceIdle,
    },
    BackendCommandDescriptor {
        name: "run_authenticated_relay_source_auth_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAuthenticatedRelaySourceAuthRejected,
        result_fixture: PrototypeFixture::AuthenticatedRelaySourceAuthRejected,
    },
    BackendCommandDescriptor {
        name: "run_authenticated_relay_source_plaintext_forbidden",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAuthenticatedRelaySourcePlaintextForbidden,
        result_fixture: PrototypeFixture::AuthenticatedRelaySourcePlaintextForbidden,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncSessionHappyPath,
        result_fixture: PrototypeFixture::InboundSyncSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_session_idle",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncSessionIdle,
        result_fixture: PrototypeFixture::InboundSyncSessionIdle,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_session_sync_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncSessionSyncRejected,
        result_fixture: PrototypeFixture::InboundSyncSessionSyncRejected,
    },
    BackendCommandDescriptor {
        name: "run_inbound_sync_session_receive_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunInboundSyncSessionReceiveRejected,
        result_fixture: PrototypeFixture::InboundSyncSessionReceiveRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_object_store_upload_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectStoreUploadReady,
        result_fixture: PrototypeFixture::MediaObjectStoreUploadReady,
    },
    BackendCommandDescriptor {
        name: "run_media_object_store_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectStorePlaintextRejected,
        result_fixture: PrototypeFixture::MediaObjectStorePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_object_store_auto_download_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectStoreAutoDownloadRejected,
        result_fixture: PrototypeFixture::MediaObjectStoreAutoDownloadRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_object_store_oversize_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectStoreOversizeRejected,
        result_fixture: PrototypeFixture::MediaObjectStoreOversizeRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_upload_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaUploadSessionHappyPath,
        result_fixture: PrototypeFixture::MediaUploadSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_media_upload_session_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaUploadSessionPlaintextRejected,
        result_fixture: PrototypeFixture::MediaUploadSessionPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_upload_session_seal_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaUploadSessionSealRejected,
        result_fixture: PrototypeFixture::MediaUploadSessionSealRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_upload_session_store_write_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaUploadSessionStoreWriteRejected,
        result_fixture: PrototypeFixture::MediaUploadSessionStoreWriteRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_service_adapter_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceAdapterReady,
        result_fixture: PrototypeFixture::MediaServiceAdapterReady,
    },
    BackendCommandDescriptor {
        name: "run_media_service_adapter_auth_missing",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceAdapterAuthMissing,
        result_fixture: PrototypeFixture::MediaServiceAdapterAuthMissing,
    },
    BackendCommandDescriptor {
        name: "run_media_service_adapter_plaintext_forbidden",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceAdapterPlaintextForbidden,
        result_fixture: PrototypeFixture::MediaServiceAdapterPlaintextForbidden,
    },
    BackendCommandDescriptor {
        name: "run_media_service_adapter_digest_unverified",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceAdapterDigestUnverified,
        result_fixture: PrototypeFixture::MediaServiceAdapterDigestUnverified,
    },
    BackendCommandDescriptor {
        name: "run_media_service_upload_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceUploadSessionHappyPath,
        result_fixture: PrototypeFixture::MediaServiceUploadSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_media_service_upload_session_media_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceUploadSessionMediaRejected,
        result_fixture: PrototypeFixture::MediaServiceUploadSessionMediaRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_service_upload_session_auth_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceUploadSessionAuthRejected,
        result_fixture: PrototypeFixture::MediaServiceUploadSessionAuthRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_service_upload_session_digest_unverified",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceUploadSessionDigestUnverified,
        result_fixture: PrototypeFixture::MediaServiceUploadSessionDigestUnverified,
    },
    BackendCommandDescriptor {
        name: "run_media_service_download_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceDownloadReady,
        result_fixture: PrototypeFixture::MediaServiceDownloadReady,
    },
    BackendCommandDescriptor {
        name: "run_media_service_download_plaintext_preview_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceDownloadPlaintextPreviewRejected,
        result_fixture: PrototypeFixture::MediaServiceDownloadPlaintextPreviewRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_service_download_auth_missing",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceDownloadAuthMissing,
        result_fixture: PrototypeFixture::MediaServiceDownloadAuthMissing,
    },
    BackendCommandDescriptor {
        name: "run_media_service_download_digest_unverified",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaServiceDownloadDigestUnverified,
        result_fixture: PrototypeFixture::MediaServiceDownloadDigestUnverified,
    },
    BackendCommandDescriptor {
        name: "run_media_download_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaDownloadSessionHappyPath,
        result_fixture: PrototypeFixture::MediaDownloadSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_media_download_session_download_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaDownloadSessionDownloadRejected,
        result_fixture: PrototypeFixture::MediaDownloadSessionDownloadRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_download_session_store_write_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaDownloadSessionStoreWriteRejected,
        result_fixture: PrototypeFixture::MediaDownloadSessionStoreWriteRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_download_session_open_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaDownloadSessionOpenRejected,
        result_fixture: PrototypeFixture::MediaDownloadSessionOpenRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_retention_delete_and_evict_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaRetentionDeleteAndEvictReady,
        result_fixture: PrototypeFixture::MediaRetentionDeleteAndEvictReady,
    },
    BackendCommandDescriptor {
        name: "run_media_retention_retain_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaRetentionRetainReady,
        result_fixture: PrototypeFixture::MediaRetentionRetainReady,
    },
    BackendCommandDescriptor {
        name: "run_media_retention_hold_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaRetentionHoldRejected,
        result_fixture: PrototypeFixture::MediaRetentionHoldRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_retention_auth_missing",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaRetentionAuthMissing,
        result_fixture: PrototypeFixture::MediaRetentionAuthMissing,
    },
    BackendCommandDescriptor {
        name: "run_media_cleanup_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaCleanupSessionHappyPath,
        result_fixture: PrototypeFixture::MediaCleanupSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_media_cleanup_session_retain_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaCleanupSessionRetainReady,
        result_fixture: PrototypeFixture::MediaCleanupSessionRetainReady,
    },
    BackendCommandDescriptor {
        name: "run_media_cleanup_session_retention_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaCleanupSessionRetentionRejected,
        result_fixture: PrototypeFixture::MediaCleanupSessionRetentionRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_cleanup_session_cache_absent",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaCleanupSessionCacheAbsent,
        result_fixture: PrototypeFixture::MediaCleanupSessionCacheAbsent,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_remote_and_local_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexRemoteAndLocalReady,
        result_fixture: PrototypeFixture::MediaObjectIndexRemoteAndLocalReady,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_absent_upload_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexAbsentUploadReady,
        result_fixture: PrototypeFixture::MediaObjectIndexAbsentUploadReady,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_delete_pending_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexDeletePendingReady,
        result_fixture: PrototypeFixture::MediaObjectIndexDeletePendingReady,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_deleted_terminal",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexDeletedTerminal,
        result_fixture: PrototypeFixture::MediaObjectIndexDeletedTerminal,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_plaintext_metadata_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexPlaintextMetadataRejected,
        result_fixture: PrototypeFixture::MediaObjectIndexPlaintextMetadataRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_bad_lifecycle_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexBadLifecycleRejected,
        result_fixture: PrototypeFixture::MediaObjectIndexBadLifecycleRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_store_write_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexStoreWriteReady,
        result_fixture: PrototypeFixture::MediaObjectIndexStoreWriteReady,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_store_index_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexStoreIndexRejected,
        result_fixture: PrototypeFixture::MediaObjectIndexStoreIndexRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_store_bad_object_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexStoreBadObjectRejected,
        result_fixture: PrototypeFixture::MediaObjectIndexStoreBadObjectRejected,
    },
    BackendCommandDescriptor {
        name: "run_media_object_index_store_deleted_snapshot",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMediaObjectIndexStoreDeletedSnapshot,
        result_fixture: PrototypeFixture::MediaObjectIndexStoreDeletedSnapshot,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_upload_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaUploadSessionHappyPath,
        result_fixture: PrototypeFixture::IndexedMediaUploadSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_upload_session_service_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaUploadSessionServiceRejected,
        result_fixture: PrototypeFixture::IndexedMediaUploadSessionServiceRejected,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_upload_session_index_store_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaUploadSessionIndexStoreRejected,
        result_fixture: PrototypeFixture::IndexedMediaUploadSessionIndexStoreRejected,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_download_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaDownloadSessionHappyPath,
        result_fixture: PrototypeFixture::IndexedMediaDownloadSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_download_session_manifest_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaDownloadSessionManifestRejected,
        result_fixture: PrototypeFixture::IndexedMediaDownloadSessionManifestRejected,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_download_session_not_downloadable",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaDownloadSessionNotDownloadable,
        result_fixture: PrototypeFixture::IndexedMediaDownloadSessionNotDownloadable,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_download_session_download_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaDownloadSessionDownloadRejected,
        result_fixture: PrototypeFixture::IndexedMediaDownloadSessionDownloadRejected,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_cleanup_session_happy_path",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaCleanupSessionHappyPath,
        result_fixture: PrototypeFixture::IndexedMediaCleanupSessionHappyPath,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_cleanup_session_manifest_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaCleanupSessionManifestRejected,
        result_fixture: PrototypeFixture::IndexedMediaCleanupSessionManifestRejected,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_cleanup_session_not_cleanable",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaCleanupSessionNotCleanable,
        result_fixture: PrototypeFixture::IndexedMediaCleanupSessionNotCleanable,
    },
    BackendCommandDescriptor {
        name: "run_indexed_media_cleanup_session_cleanup_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunIndexedMediaCleanupSessionCleanupRejected,
        result_fixture: PrototypeFixture::IndexedMediaCleanupSessionCleanupRejected,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_credential_issuer_trust_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousCredentialIssuerTrustReady,
        result_fixture: PrototypeFixture::AnonymousCredentialIssuerTrustReady,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_credential_issuer_trust_transparency_required",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunAnonymousCredentialIssuerTrustTransparencyRequired,
        result_fixture: PrototypeFixture::AnonymousCredentialIssuerTrustTransparencyRequired,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_credential_issuer_trust_revoked",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousCredentialIssuerTrustRevoked,
        result_fixture: PrototypeFixture::AnonymousCredentialIssuerTrustRevoked,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_credential_issuer_trust_partitioning_metadata_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunAnonymousCredentialIssuerTrustPartitioningMetadataRejected,
        result_fixture: PrototypeFixture::AnonymousCredentialIssuerTrustPartitioningMetadataRejected,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_credential_issuer_trust_witness_audit_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunAnonymousCredentialIssuerTrustWitnessAuditRejected,
        result_fixture: PrototypeFixture::AnonymousCredentialIssuerTrustWitnessAuditRejected,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_group_membership_proof_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousGroupMembershipProofReady,
        result_fixture: PrototypeFixture::AnonymousGroupMembershipProofReady,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_group_membership_proof_high_security_pq_required",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunAnonymousGroupMembershipProofHighSecurityPqRequired,
        result_fixture: PrototypeFixture::AnonymousGroupMembershipProofHighSecurityPqRequired,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_group_membership_proof_replay_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousGroupMembershipProofReplayRejected,
        result_fixture: PrototypeFixture::AnonymousGroupMembershipProofReplayRejected,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_group_membership_proof_route_binding_required",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunAnonymousGroupMembershipProofRouteBindingRequired,
        result_fixture: PrototypeFixture::AnonymousGroupMembershipProofRouteBindingRequired,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_group_membership_proof_plaintext_identity_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunAnonymousGroupMembershipProofPlaintextIdentityRejected,
        result_fixture: PrototypeFixture::AnonymousGroupMembershipProofPlaintextIdentityRejected,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_rate_limit_nullifier_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousRateLimitNullifierReady,
        result_fixture: PrototypeFixture::AnonymousRateLimitNullifierReady,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_rate_limit_nullifier_replay_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousRateLimitNullifierReplayRejected,
        result_fixture: PrototypeFixture::AnonymousRateLimitNullifierReplayRejected,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_rate_limit_nullifier_limit_exceeded",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousRateLimitNullifierLimitExceeded,
        result_fixture: PrototypeFixture::AnonymousRateLimitNullifierLimitExceeded,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_rate_limit_nullifier_opaque_store_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousRateLimitNullifierOpaqueStoreRequired,
        result_fixture: PrototypeFixture::AnonymousRateLimitNullifierOpaqueStoreRequired,
    },
    BackendCommandDescriptor {
        name: "run_group_relay_envelope_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupRelayEnvelopeReady,
        result_fixture: PrototypeFixture::GroupRelayEnvelopeReady,
    },
    BackendCommandDescriptor {
        name: "run_group_relay_envelope_transcript_sync_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupRelayEnvelopeTranscriptSyncRequired,
        result_fixture: PrototypeFixture::GroupRelayEnvelopeTranscriptSyncRequired,
    },
    BackendCommandDescriptor {
        name: "run_group_relay_envelope_transcript_rekey_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupRelayEnvelopeTranscriptRekeyRequired,
        result_fixture: PrototypeFixture::GroupRelayEnvelopeTranscriptRekeyRequired,
    },
    BackendCommandDescriptor {
        name: "run_group_relay_envelope_missing_delivery_token",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupRelayEnvelopeMissingDeliveryToken,
        result_fixture: PrototypeFixture::GroupRelayEnvelopeMissingDeliveryToken,
    },
    BackendCommandDescriptor {
        name: "run_group_relay_envelope_plaintext_metadata_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupRelayEnvelopePlaintextMetadataRejected,
        result_fixture: PrototypeFixture::GroupRelayEnvelopePlaintextMetadataRejected,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_nullifier_store_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousNullifierStoreReady,
        result_fixture: PrototypeFixture::AnonymousNullifierStoreReady,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_nullifier_store_replay_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunAnonymousNullifierStoreReplayRejected,
        result_fixture: PrototypeFixture::AnonymousNullifierStoreReplayRejected,
    },
    BackendCommandDescriptor {
        name: "run_anonymous_nullifier_store_plaintext_metadata_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunAnonymousNullifierStorePlaintextMetadataRejected,
        result_fixture: PrototypeFixture::AnonymousNullifierStorePlaintextMetadataRejected,
    },
    BackendCommandDescriptor {
        name: "run_group_chat_mls_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupChatMlsReady,
        result_fixture: PrototypeFixture::GroupChatMlsReady,
    },
    BackendCommandDescriptor {
        name: "run_group_chat_mls_setup_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupChatMlsSetupRequired,
        result_fixture: PrototypeFixture::GroupChatMlsSetupRequired,
    },
    BackendCommandDescriptor {
        name: "run_group_chat_membership_sync_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupChatMembershipSyncRequired,
        result_fixture: PrototypeFixture::GroupChatMembershipSyncRequired,
    },
    BackendCommandDescriptor {
        name: "run_group_chat_plaintext_metadata_forbidden",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupChatPlaintextMetadataForbidden,
        result_fixture: PrototypeFixture::GroupChatPlaintextMetadataForbidden,
    },
    BackendCommandDescriptor {
        name: "run_group_chat_high_security_mls_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupChatHighSecurityMlsRequired,
        result_fixture: PrototypeFixture::GroupChatHighSecurityMlsRequired,
    },
    BackendCommandDescriptor {
        name: "run_group_chat_high_security_pq_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupChatHighSecurityPqRequired,
        result_fixture: PrototypeFixture::GroupChatHighSecurityPqRequired,
    },
    BackendCommandDescriptor {
        name: "run_group_chat_mls_provider_security_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunGroupChatMlsProviderSecurityRequired,
        result_fixture: PrototypeFixture::GroupChatMlsProviderSecurityRequired,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_store_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceStoreReady,
        result_fixture: PrototypeFixture::MlsProviderEvidenceStoreReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_store_gate_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceStoreGateRejected,
        result_fixture: PrototypeFixture::MlsProviderEvidenceStoreGateRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_store_duplicate_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceStoreDuplicateRejected,
        result_fixture: PrototypeFixture::MlsProviderEvidenceStoreDuplicateRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_store_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceStorePlaintextRejected,
        result_fixture: PrototypeFixture::MlsProviderEvidenceStorePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_use_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceUseReady,
        result_fixture: PrototypeFixture::MlsProviderEvidenceUseReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_use_missing",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceUseMissing,
        result_fixture: PrototypeFixture::MlsProviderEvidenceUseMissing,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_use_expired",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceUseExpired,
        result_fixture: PrototypeFixture::MlsProviderEvidenceUseExpired,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_use_suite_mismatch",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceUseSuiteMismatch,
        result_fixture: PrototypeFixture::MlsProviderEvidenceUseSuiteMismatch,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_evidence_use_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderEvidenceUsePlaintextRejected,
        result_fixture: PrototypeFixture::MlsProviderEvidenceUsePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_adapter_selection_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderAdapterSelectionReady,
        result_fixture: PrototypeFixture::MlsProviderAdapterSelectionReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_adapter_selection_provider_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderAdapterSelectionProviderRejected,
        result_fixture: PrototypeFixture::MlsProviderAdapterSelectionProviderRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_adapter_selection_pq_draft_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderAdapterSelectionPqDraftRejected,
        result_fixture: PrototypeFixture::MlsProviderAdapterSelectionPqDraftRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_adapter_selection_storage_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderAdapterSelectionStorageRejected,
        result_fixture: PrototypeFixture::MlsProviderAdapterSelectionStorageRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_provider_adapter_selection_supply_chain_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsProviderAdapterSelectionSupplyChainRejected,
        result_fixture: PrototypeFixture::MlsProviderAdapterSelectionSupplyChainRejected,
    },
    BackendCommandDescriptor {
        name: "run_secure_backup_restore_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSecureBackupRestoreReady,
        result_fixture: PrototypeFixture::SecureBackupRestoreReady,
    },
    BackendCommandDescriptor {
        name: "run_secure_backup_restore_recovery_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSecureBackupRestoreRecoveryRejected,
        result_fixture: PrototypeFixture::SecureBackupRestoreRecoveryRejected,
    },
    BackendCommandDescriptor {
        name: "run_secure_backup_restore_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSecureBackupRestorePlaintextRejected,
        result_fixture: PrototypeFixture::SecureBackupRestorePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_secure_backup_restore_mls_rekey_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSecureBackupRestoreMlsRekeyRejected,
        result_fixture: PrototypeFixture::SecureBackupRestoreMlsRekeyRejected,
    },
    BackendCommandDescriptor {
        name: "run_secure_backup_restore_cloud_policy_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSecureBackupRestoreCloudPolicyRejected,
        result_fixture: PrototypeFixture::SecureBackupRestoreCloudPolicyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_chain_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventChainReady,
        result_fixture: PrototypeFixture::SealedAuditEventChainReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_chain_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventChainPlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditEventChainPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_chain_rollback_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventChainRollbackRejected,
        result_fixture: PrototypeFixture::SealedAuditEventChainRollbackRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_chain_witness_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventChainWitnessRejected,
        result_fixture: PrototypeFixture::SealedAuditEventChainWitnessRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_chain_binding_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventChainBindingRejected,
        result_fixture: PrototypeFixture::SealedAuditEventChainBindingRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_store_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventStoreReady,
        result_fixture: PrototypeFixture::SealedAuditEventStoreReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_store_chain_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventStoreChainRejected,
        result_fixture: PrototypeFixture::SealedAuditEventStoreChainRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_store_duplicate_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventStoreDuplicateRejected,
        result_fixture: PrototypeFixture::SealedAuditEventStoreDuplicateRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_store_rollback_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventStoreRollbackRejected,
        result_fixture: PrototypeFixture::SealedAuditEventStoreRollbackRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_event_store_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditEventStorePlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditEventStorePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_checkpoint_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessCheckpointReady,
        result_fixture: PrototypeFixture::SealedAuditWitnessCheckpointReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_checkpoint_store_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessCheckpointStoreRejected,
        result_fixture: PrototypeFixture::SealedAuditWitnessCheckpointStoreRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_checkpoint_quorum_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessCheckpointQuorumRejected,
        result_fixture: PrototypeFixture::SealedAuditWitnessCheckpointQuorumRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_checkpoint_split_view_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessCheckpointSplitViewRejected,
        result_fixture: PrototypeFixture::SealedAuditWitnessCheckpointSplitViewRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_checkpoint_privacy_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessCheckpointPrivacyRejected,
        result_fixture: PrototypeFixture::SealedAuditWitnessCheckpointPrivacyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_client_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessClientReady,
        result_fixture: PrototypeFixture::SealedAuditWitnessClientReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_client_conflict",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessClientConflict,
        result_fixture: PrototypeFixture::SealedAuditWitnessClientConflict,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_client_unavailable",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessClientUnavailable,
        result_fixture: PrototypeFixture::SealedAuditWitnessClientUnavailable,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_client_policy_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessClientPolicyRejected,
        result_fixture: PrototypeFixture::SealedAuditWitnessClientPolicyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_witness_client_monitor_privacy_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditWitnessClientMonitorPrivacyRejected,
        result_fixture: PrototypeFixture::SealedAuditWitnessClientMonitorPrivacyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_bundle_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofBundleReady,
        result_fixture: PrototypeFixture::SealedAuditProofBundleReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_bundle_client_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofBundleClientRejected,
        result_fixture: PrototypeFixture::SealedAuditProofBundleClientRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_bundle_stale_witness",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofBundleStaleWitness,
        result_fixture: PrototypeFixture::SealedAuditProofBundleStaleWitness,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_bundle_policy_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofBundlePolicyRejected,
        result_fixture: PrototypeFixture::SealedAuditProofBundlePolicyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_bundle_privacy_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofBundlePrivacyRejected,
        result_fixture: PrototypeFixture::SealedAuditProofBundlePrivacyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_cache_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofCacheReady,
        result_fixture: PrototypeFixture::SealedAuditProofCacheReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_cache_bundle_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofCacheBundleRejected,
        result_fixture: PrototypeFixture::SealedAuditProofCacheBundleRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_cache_duplicate_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofCacheDuplicateRejected,
        result_fixture: PrototypeFixture::SealedAuditProofCacheDuplicateRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_cache_policy_stale",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofCachePolicyStale,
        result_fixture: PrototypeFixture::SealedAuditProofCachePolicyStale,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_proof_cache_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditProofCachePlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditProofCachePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_verifier_policy_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditVerifierPolicyReady,
        result_fixture: PrototypeFixture::SealedAuditVerifierPolicyReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_verifier_policy_expired",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditVerifierPolicyExpired,
        result_fixture: PrototypeFixture::SealedAuditVerifierPolicyExpired,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_verifier_policy_key_rotation_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditVerifierPolicyKeyRotationRequired,
        result_fixture: PrototypeFixture::SealedAuditVerifierPolicyKeyRotationRequired,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_verifier_policy_monitor_privacy_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditVerifierPolicyMonitorPrivacyRejected,
        result_fixture: PrototypeFixture::SealedAuditVerifierPolicyMonitorPrivacyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_verifier_policy_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditVerifierPolicyPlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditVerifierPolicyPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_incident_evidence_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditIncidentEvidenceReady,
        result_fixture: PrototypeFixture::SealedAuditIncidentEvidenceReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_incident_evidence_policy_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditIncidentEvidencePolicyRejected,
        result_fixture: PrototypeFixture::SealedAuditIncidentEvidencePolicyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_incident_evidence_missing_proof_report",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditIncidentEvidenceMissingProofReport,
        result_fixture: PrototypeFixture::SealedAuditIncidentEvidenceMissingProofReport,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_incident_evidence_split_view",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditIncidentEvidenceSplitView,
        result_fixture: PrototypeFixture::SealedAuditIncidentEvidenceSplitView,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_incident_evidence_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditIncidentEvidencePlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditIncidentEvidencePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_recovery_export_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditRecoveryExportReady,
        result_fixture: PrototypeFixture::SealedAuditRecoveryExportReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_recovery_export_incident_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditRecoveryExportIncidentRejected,
        result_fixture: PrototypeFixture::SealedAuditRecoveryExportIncidentRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_recovery_export_quorum_required",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditRecoveryExportQuorumRequired,
        result_fixture: PrototypeFixture::SealedAuditRecoveryExportQuorumRequired,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_recovery_export_rollback_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditRecoveryExportRollbackRejected,
        result_fixture: PrototypeFixture::SealedAuditRecoveryExportRollbackRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_recovery_export_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditRecoveryExportPlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditRecoveryExportPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_database_adapter_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditDatabaseAdapterReady,
        result_fixture: PrototypeFixture::SealedAuditDatabaseAdapterReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_database_adapter_encryption_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditDatabaseAdapterEncryptionRejected,
        result_fixture: PrototypeFixture::SealedAuditDatabaseAdapterEncryptionRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_database_adapter_append_only_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditDatabaseAdapterAppendOnlyRejected,
        result_fixture: PrototypeFixture::SealedAuditDatabaseAdapterAppendOnlyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_transport_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportTransportReady,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportTransportReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_transport_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportTransportPlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportTransportPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_outbox_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportOutboxReady,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportOutboxReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_outbox_transport_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportOutboxTransportRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportOutboxTransportRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_outbox_replay_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportOutboxReplayRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportOutboxReplayRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_outbox_rate_limit_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportOutboxRateLimitRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportOutboxRateLimitRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_outbox_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportOutboxPlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportOutboxPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_receipt_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportReceiptReady,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReceiptReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_receipt_outbox_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportReceiptOutboxRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReceiptOutboxRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_receipt_missing",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportReceiptMissing,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReceiptMissing,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_receipt_transparency_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportReceiptTransparencyRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReceiptTransparencyRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_receipt_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportReceiptPlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReceiptPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_reconciliation_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportReconciliationReady,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_reconciliation_receipt_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunSealedAuditPrivateReportReconciliationReceiptRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationReceiptRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_reconciliation_retry_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunSealedAuditPrivateReportReconciliationRetryRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationRetryRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_reconciliation_false_delivery_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportReconciliationFalseDeliveryRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationFalseDeliveryRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_reconciliation_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunSealedAuditPrivateReportReconciliationPlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportReconciliationPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_gateway_evidence_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunSealedAuditPrivateReportGatewayEvidenceReady,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceReady,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_gateway_evidence_reconciliation_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunSealedAuditPrivateReportGatewayEvidenceReconciliationRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceReconciliationRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_gateway_evidence_unavailable_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunSealedAuditPrivateReportGatewayEvidenceUnavailableRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceUnavailableRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_gateway_evidence_accountability_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunSealedAuditPrivateReportGatewayEvidenceAccountabilityRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceAccountabilityRejected,
    },
    BackendCommandDescriptor {
        name: "run_sealed_audit_private_report_gateway_evidence_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunSealedAuditPrivateReportGatewayEvidencePlaintextRejected,
        result_fixture: PrototypeFixture::SealedAuditPrivateReportGatewayEvidencePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_admission_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageAdmissionReady,
        result_fixture: PrototypeFixture::MlsKeyPackageAdmissionReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_admission_group_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageAdmissionGroupRejected,
        result_fixture: PrototypeFixture::MlsKeyPackageAdmissionGroupRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_admission_lifetime_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageAdmissionLifetimeRejected,
        result_fixture: PrototypeFixture::MlsKeyPackageAdmissionLifetimeRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_admission_suite_mismatch",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageAdmissionSuiteMismatch,
        result_fixture: PrototypeFixture::MlsKeyPackageAdmissionSuiteMismatch,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_admission_credential_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageAdmissionCredentialRejected,
        result_fixture: PrototypeFixture::MlsKeyPackageAdmissionCredentialRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_admission_replay_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageAdmissionReplayRejected,
        result_fixture: PrototypeFixture::MlsKeyPackageAdmissionReplayRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_admission_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageAdmissionPlaintextRejected,
        result_fixture: PrototypeFixture::MlsKeyPackageAdmissionPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_consume_store_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageConsumeStoreReady,
        result_fixture: PrototypeFixture::MlsKeyPackageConsumeStoreReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_consume_store_admission_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageConsumeStoreAdmissionRejected,
        result_fixture: PrototypeFixture::MlsKeyPackageConsumeStoreAdmissionRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_consume_store_duplicate_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageConsumeStoreDuplicateRejected,
        result_fixture: PrototypeFixture::MlsKeyPackageConsumeStoreDuplicateRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_consume_store_bad_shape",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageConsumeStoreBadShape,
        result_fixture: PrototypeFixture::MlsKeyPackageConsumeStoreBadShape,
    },
    BackendCommandDescriptor {
        name: "run_mls_key_package_consume_store_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsKeyPackageConsumeStorePlaintextRejected,
        result_fixture: PrototypeFixture::MlsKeyPackageConsumeStorePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_send_outbox_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeSendOutboxReady,
        result_fixture: PrototypeFixture::MlsWelcomeSendOutboxReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_send_outbox_consume_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeSendOutboxConsumeRejected,
        result_fixture: PrototypeFixture::MlsWelcomeSendOutboxConsumeRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_send_outbox_duplicate_transaction_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeSendOutboxDuplicateTransactionRejected,
        result_fixture: PrototypeFixture::MlsWelcomeSendOutboxDuplicateTransactionRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_send_outbox_key_package_queued",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeSendOutboxKeyPackageQueued,
        result_fixture: PrototypeFixture::MlsWelcomeSendOutboxKeyPackageQueued,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_send_outbox_bad_shape",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeSendOutboxBadShape,
        result_fixture: PrototypeFixture::MlsWelcomeSendOutboxBadShape,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_send_outbox_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeSendOutboxPlaintextRejected,
        result_fixture: PrototypeFixture::MlsWelcomeSendOutboxPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_membership_transaction_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsMembershipTransactionReady,
        result_fixture: PrototypeFixture::MlsMembershipTransactionReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_membership_transaction_binding_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsMembershipTransactionBindingRejected,
        result_fixture: PrototypeFixture::MlsMembershipTransactionBindingRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_membership_transaction_storage_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsMembershipTransactionStorageRejected,
        result_fixture: PrototypeFixture::MlsMembershipTransactionStorageRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_membership_transaction_duplicate_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsMembershipTransactionDuplicateRejected,
        result_fixture: PrototypeFixture::MlsMembershipTransactionDuplicateRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_membership_transaction_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsMembershipTransactionPlaintextRejected,
        result_fixture: PrototypeFixture::MlsMembershipTransactionPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_security_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunLocalStoreDatabaseSecurityReady,
        result_fixture: PrototypeFixture::LocalStoreDatabaseSecurityReady,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_security_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunLocalStoreDatabaseSecurityPlaintextRejected,
        result_fixture: PrototypeFixture::LocalStoreDatabaseSecurityPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_security_wal_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunLocalStoreDatabaseSecurityWalRejected,
        result_fixture: PrototypeFixture::LocalStoreDatabaseSecurityWalRejected,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_security_backup_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunLocalStoreDatabaseSecurityBackupRejected,
        result_fixture: PrototypeFixture::LocalStoreDatabaseSecurityBackupRejected,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_security_secret_lifecycle_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunLocalStoreDatabaseSecuritySecretLifecycleRejected,
        result_fixture: PrototypeFixture::LocalStoreDatabaseSecuritySecretLifecycleRejected,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_adapter_selection_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunLocalStoreDatabaseAdapterSelectionReady,
        result_fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionReady,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_adapter_selection_license_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunLocalStoreDatabaseAdapterSelectionLicenseRejected,
        result_fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionLicenseRejected,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_adapter_selection_fips_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunLocalStoreDatabaseAdapterSelectionFipsRejected,
        result_fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionFipsRejected,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_adapter_selection_migration_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunLocalStoreDatabaseAdapterSelectionMigrationRejected,
        result_fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionMigrationRejected,
    },
    BackendCommandDescriptor {
        name: "run_local_store_database_adapter_selection_supply_chain_rejected",
        actor_kind: ActorKind::Human,
        command_kind:
            PrototypeBackendCommandKind::RunLocalStoreDatabaseAdapterSelectionSupplyChainRejected,
        result_fixture: PrototypeFixture::LocalStoreDatabaseAdapterSelectionSupplyChainRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_admission_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeAdmissionReady,
        result_fixture: PrototypeFixture::MlsWelcomeAdmissionReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_admission_secrets_missing",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeAdmissionSecretsMissing,
        result_fixture: PrototypeFixture::MlsWelcomeAdmissionSecretsMissing,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_admission_tree_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeAdmissionTreeRejected,
        result_fixture: PrototypeFixture::MlsWelcomeAdmissionTreeRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_admission_confirmation_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeAdmissionConfirmationRejected,
        result_fixture: PrototypeFixture::MlsWelcomeAdmissionConfirmationRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_admission_tie_break_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeAdmissionTieBreakRejected,
        result_fixture: PrototypeFixture::MlsWelcomeAdmissionTieBreakRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_admission_replay_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeAdmissionReplayRejected,
        result_fixture: PrototypeFixture::MlsWelcomeAdmissionReplayRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_admission_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeAdmissionPlaintextRejected,
        result_fixture: PrototypeFixture::MlsWelcomeAdmissionPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_admission_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitAdmissionReady,
        result_fixture: PrototypeFixture::MlsCommitAdmissionReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_admission_bad_epoch",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitAdmissionBadEpoch,
        result_fixture: PrototypeFixture::MlsCommitAdmissionBadEpoch,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_admission_auth_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitAdmissionAuthRejected,
        result_fixture: PrototypeFixture::MlsCommitAdmissionAuthRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_admission_path_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitAdmissionPathRejected,
        result_fixture: PrototypeFixture::MlsCommitAdmissionPathRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_admission_tie_break_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitAdmissionTieBreakRejected,
        result_fixture: PrototypeFixture::MlsCommitAdmissionTieBreakRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_admission_replay_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitAdmissionReplayRejected,
        result_fixture: PrototypeFixture::MlsCommitAdmissionReplayRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_admission_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitAdmissionPlaintextRejected,
        result_fixture: PrototypeFixture::MlsCommitAdmissionPlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_replay_store_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitReplayStoreReady,
        result_fixture: PrototypeFixture::MlsCommitReplayStoreReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_replay_store_admission_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitReplayStoreAdmissionRejected,
        result_fixture: PrototypeFixture::MlsCommitReplayStoreAdmissionRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_replay_store_duplicate_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitReplayStoreDuplicateRejected,
        result_fixture: PrototypeFixture::MlsCommitReplayStoreDuplicateRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_replay_store_local_member_removed",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitReplayStoreLocalMemberRemoved,
        result_fixture: PrototypeFixture::MlsCommitReplayStoreLocalMemberRemoved,
    },
    BackendCommandDescriptor {
        name: "run_mls_commit_replay_store_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsCommitReplayStorePlaintextRejected,
        result_fixture: PrototypeFixture::MlsCommitReplayStorePlaintextRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_replay_store_ready",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeReplayStoreReady,
        result_fixture: PrototypeFixture::MlsWelcomeReplayStoreReady,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_replay_store_admission_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeReplayStoreAdmissionRejected,
        result_fixture: PrototypeFixture::MlsWelcomeReplayStoreAdmissionRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_replay_store_duplicate_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeReplayStoreDuplicateRejected,
        result_fixture: PrototypeFixture::MlsWelcomeReplayStoreDuplicateRejected,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_replay_store_key_package_reused",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeReplayStoreKeyPackageReused,
        result_fixture: PrototypeFixture::MlsWelcomeReplayStoreKeyPackageReused,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_replay_store_bad_shape",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeReplayStoreBadShape,
        result_fixture: PrototypeFixture::MlsWelcomeReplayStoreBadShape,
    },
    BackendCommandDescriptor {
        name: "run_mls_welcome_replay_store_plaintext_rejected",
        actor_kind: ActorKind::Human,
        command_kind: PrototypeBackendCommandKind::RunMlsWelcomeReplayStorePlaintextRejected,
        result_fixture: PrototypeFixture::MlsWelcomeReplayStorePlaintextRejected,
    },
];

pub const fn mercury_bootstrap_status(decision: ClientBootstrapDecision) -> PlatformDecisionView {
    PlatformDecisionView::from_bootstrap(decision)
}

pub const fn mercury_prepare_send(decision: OutboundSendDecision) -> PlatformDecisionView {
    PlatformDecisionView::from_outbound_send(decision)
}

pub const fn mercury_accept_received_ciphertext(
    decision: ClientReceiveDecision,
) -> PlatformDecisionView {
    PlatformDecisionView::from_client_receive(decision)
}

pub const fn mercury_policy_status(decision: PolicyDecision) -> PlatformDecisionView {
    PlatformDecisionView::from_policy(decision)
}

pub fn platform_fixture_by_name(name: &str) -> Option<PlatformFixture> {
    PLATFORM_FIXTURES
        .iter()
        .find(|descriptor| descriptor.name == name)
        .map(|descriptor| descriptor.fixture)
}

pub fn prototype_fixture_by_name(name: &str) -> Option<PrototypeFixture> {
    PROTOTYPE_FIXTURES
        .iter()
        .find(|descriptor| descriptor.name == name)
        .map(|descriptor| descriptor.fixture)
}

pub fn backend_command_by_name(name: &str) -> Option<BackendCommandDescriptor> {
    BACKEND_COMMANDS
        .iter()
        .find(|descriptor| descriptor.name == name)
        .copied()
}

pub const fn platform_fixture_view(fixture: PlatformFixture) -> PlatformDecisionView {
    match fixture {
        PlatformFixture::BootstrapAccepted => mercury_bootstrap_status(ClientBootstrapDecision {
            accepted: true,
            can_start_sync: true,
            can_decrypt_local_store: true,
            can_open_message_ui: true,
            requires_sync: false,
            requires_recovery: false,
            requires_user_action: false,
            reason: ClientBootstrapReason::Accepted,
        }),
        PlatformFixture::BootstrapSyncIncomplete => {
            mercury_bootstrap_status(ClientBootstrapDecision {
                accepted: false,
                can_start_sync: true,
                can_decrypt_local_store: false,
                can_open_message_ui: false,
                requires_sync: true,
                requires_recovery: false,
                requires_user_action: false,
                reason: ClientBootstrapReason::SyncIncomplete,
            })
        }
        PlatformFixture::BootstrapRecoveryRequired => {
            mercury_bootstrap_status(ClientBootstrapDecision {
                accepted: false,
                can_start_sync: false,
                can_decrypt_local_store: false,
                can_open_message_ui: false,
                requires_sync: false,
                requires_recovery: true,
                requires_user_action: true,
                reason: ClientBootstrapReason::RecoveryRequired,
            })
        }
        PlatformFixture::OutboundSendAccepted => mercury_prepare_send(OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        }),
        PlatformFixture::OutboundSendMessagePolicyRejected => {
            mercury_prepare_send(OutboundSendDecision {
                accepted: false,
                can_send: false,
                can_persist_ciphertext: false,
                requires_user_action: true,
                reason: OutboundSendReason::MessagePolicyRejected,
            })
        }
        PlatformFixture::ClientReceiveAccepted => {
            mercury_accept_received_ciphertext(ClientReceiveDecision {
                accepted: true,
                can_decrypt: true,
                can_persist_ciphertext: true,
                can_expose_to_ui: true,
                requires_client_retry: false,
                requires_user_action: false,
                reason: ClientReceiveReason::Accepted,
            })
        }
        PlatformFixture::ClientReceiveOrderingGap => {
            mercury_accept_received_ciphertext(ClientReceiveDecision {
                accepted: false,
                can_decrypt: false,
                can_persist_ciphertext: false,
                can_expose_to_ui: false,
                requires_client_retry: true,
                requires_user_action: false,
                reason: ClientReceiveReason::OrderingGap,
            })
        }
        PlatformFixture::ClientReceiveSenderTrustAction => {
            mercury_accept_received_ciphertext(ClientReceiveDecision {
                accepted: true,
                can_decrypt: true,
                can_persist_ciphertext: true,
                can_expose_to_ui: true,
                requires_client_retry: false,
                requires_user_action: true,
                reason: ClientReceiveReason::Accepted,
            })
        }
        PlatformFixture::PolicyAiGrantRejected => {
            mercury_policy_status(policy_decision(PipelineReason::AiGrantReject))
        }
        PlatformFixture::PolicyAiLifecycleExpired => {
            mercury_policy_status(policy_decision(PipelineReason::AiLifecycleReject))
        }
    }
}

pub fn platform_fixture_json(fixture: PlatformFixture) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&platform_fixture_view(fixture))
}

pub fn prototype_fixture_value(fixture: PrototypeFixture) -> Value {
    match fixture {
        PrototypeFixture::LocalStoreSealedMessage => local_store_sealed_message_fixture(),
        PrototypeFixture::LocalStoreUnlockReady => local_store_unlock_ready_fixture(),
        PrototypeFixture::LocalStoreUnlockAppLockRequired => {
            local_store_unlock_app_lock_required_fixture()
        }
        PrototypeFixture::LocalStoreUnlockRecoveryRequired => {
            local_store_unlock_recovery_required_fixture()
        }
        PrototypeFixture::LocalStoreUnlockPlaintextCacheForbidden => {
            local_store_unlock_plaintext_cache_forbidden_fixture()
        }
        PrototypeFixture::AccountRecoveryHighEntropyReady => {
            account_recovery_high_entropy_ready_fixture()
        }
        PrototypeFixture::AccountRecoveryLowEntropyPinForbidden => {
            account_recovery_low_entropy_pin_forbidden_fixture()
        }
        PrototypeFixture::AccountRecoveryThresholdQuorumRequired => {
            account_recovery_threshold_quorum_required_fixture()
        }
        PrototypeFixture::AccountRecoveryPlaintextBackupForbidden => {
            account_recovery_plaintext_backup_forbidden_fixture()
        }
        PrototypeFixture::AccountRecoveryKeyRotationRequired => {
            account_recovery_key_rotation_required_fixture()
        }
        PrototypeFixture::SecureBackupRestoreReady => secure_backup_restore_ready_fixture(),
        PrototypeFixture::SecureBackupRestoreRecoveryRejected => {
            secure_backup_restore_recovery_rejected_fixture()
        }
        PrototypeFixture::SecureBackupRestorePlaintextRejected => {
            secure_backup_restore_plaintext_rejected_fixture()
        }
        PrototypeFixture::SecureBackupRestoreMlsRekeyRejected => {
            secure_backup_restore_mls_rekey_rejected_fixture()
        }
        PrototypeFixture::SecureBackupRestoreCloudPolicyRejected => {
            secure_backup_restore_cloud_policy_rejected_fixture()
        }
        PrototypeFixture::SealedAuditEventChainReady => sealed_audit_event_chain_ready_fixture(),
        PrototypeFixture::SealedAuditEventChainPlaintextRejected => {
            sealed_audit_event_chain_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditEventChainRollbackRejected => {
            sealed_audit_event_chain_rollback_rejected_fixture()
        }
        PrototypeFixture::SealedAuditEventChainWitnessRejected => {
            sealed_audit_event_chain_witness_rejected_fixture()
        }
        PrototypeFixture::SealedAuditEventChainBindingRejected => {
            sealed_audit_event_chain_binding_rejected_fixture()
        }
        PrototypeFixture::SealedAuditEventStoreReady => sealed_audit_event_store_ready_fixture(),
        PrototypeFixture::SealedAuditEventStoreChainRejected => {
            sealed_audit_event_store_chain_rejected_fixture()
        }
        PrototypeFixture::SealedAuditEventStoreDuplicateRejected => {
            sealed_audit_event_store_duplicate_rejected_fixture()
        }
        PrototypeFixture::SealedAuditEventStoreRollbackRejected => {
            sealed_audit_event_store_rollback_rejected_fixture()
        }
        PrototypeFixture::SealedAuditEventStorePlaintextRejected => {
            sealed_audit_event_store_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditWitnessCheckpointReady => {
            sealed_audit_witness_checkpoint_ready_fixture()
        }
        PrototypeFixture::SealedAuditWitnessCheckpointStoreRejected => {
            sealed_audit_witness_checkpoint_store_rejected_fixture()
        }
        PrototypeFixture::SealedAuditWitnessCheckpointQuorumRejected => {
            sealed_audit_witness_checkpoint_quorum_rejected_fixture()
        }
        PrototypeFixture::SealedAuditWitnessCheckpointSplitViewRejected => {
            sealed_audit_witness_checkpoint_split_view_rejected_fixture()
        }
        PrototypeFixture::SealedAuditWitnessCheckpointPrivacyRejected => {
            sealed_audit_witness_checkpoint_privacy_rejected_fixture()
        }
        PrototypeFixture::SealedAuditWitnessClientReady => {
            sealed_audit_witness_client_ready_fixture()
        }
        PrototypeFixture::SealedAuditWitnessClientConflict => {
            sealed_audit_witness_client_conflict_fixture()
        }
        PrototypeFixture::SealedAuditWitnessClientUnavailable => {
            sealed_audit_witness_client_unavailable_fixture()
        }
        PrototypeFixture::SealedAuditWitnessClientPolicyRejected => {
            sealed_audit_witness_client_policy_rejected_fixture()
        }
        PrototypeFixture::SealedAuditWitnessClientMonitorPrivacyRejected => {
            sealed_audit_witness_client_monitor_privacy_rejected_fixture()
        }
        PrototypeFixture::SealedAuditProofBundleReady => sealed_audit_proof_bundle_ready_fixture(),
        PrototypeFixture::SealedAuditProofBundleClientRejected => {
            sealed_audit_proof_bundle_client_rejected_fixture()
        }
        PrototypeFixture::SealedAuditProofBundleStaleWitness => {
            sealed_audit_proof_bundle_stale_witness_fixture()
        }
        PrototypeFixture::SealedAuditProofBundlePolicyRejected => {
            sealed_audit_proof_bundle_policy_rejected_fixture()
        }
        PrototypeFixture::SealedAuditProofBundlePrivacyRejected => {
            sealed_audit_proof_bundle_privacy_rejected_fixture()
        }
        PrototypeFixture::SealedAuditProofCacheReady => sealed_audit_proof_cache_ready_fixture(),
        PrototypeFixture::SealedAuditProofCacheBundleRejected => {
            sealed_audit_proof_cache_bundle_rejected_fixture()
        }
        PrototypeFixture::SealedAuditProofCacheDuplicateRejected => {
            sealed_audit_proof_cache_duplicate_rejected_fixture()
        }
        PrototypeFixture::SealedAuditProofCachePolicyStale => {
            sealed_audit_proof_cache_policy_stale_fixture()
        }
        PrototypeFixture::SealedAuditProofCachePlaintextRejected => {
            sealed_audit_proof_cache_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditVerifierPolicyReady => {
            sealed_audit_verifier_policy_ready_fixture()
        }
        PrototypeFixture::SealedAuditVerifierPolicyExpired => {
            sealed_audit_verifier_policy_expired_fixture()
        }
        PrototypeFixture::SealedAuditVerifierPolicyKeyRotationRequired => {
            sealed_audit_verifier_policy_key_rotation_required_fixture()
        }
        PrototypeFixture::SealedAuditVerifierPolicyMonitorPrivacyRejected => {
            sealed_audit_verifier_policy_monitor_privacy_rejected_fixture()
        }
        PrototypeFixture::SealedAuditVerifierPolicyPlaintextRejected => {
            sealed_audit_verifier_policy_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditIncidentEvidenceReady => {
            sealed_audit_incident_evidence_ready_fixture()
        }
        PrototypeFixture::SealedAuditIncidentEvidencePolicyRejected => {
            sealed_audit_incident_evidence_policy_rejected_fixture()
        }
        PrototypeFixture::SealedAuditIncidentEvidenceMissingProofReport => {
            sealed_audit_incident_evidence_missing_proof_report_fixture()
        }
        PrototypeFixture::SealedAuditIncidentEvidenceSplitView => {
            sealed_audit_incident_evidence_split_view_fixture()
        }
        PrototypeFixture::SealedAuditIncidentEvidencePlaintextRejected => {
            sealed_audit_incident_evidence_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditRecoveryExportReady => {
            sealed_audit_recovery_export_ready_fixture()
        }
        PrototypeFixture::SealedAuditRecoveryExportIncidentRejected => {
            sealed_audit_recovery_export_incident_rejected_fixture()
        }
        PrototypeFixture::SealedAuditRecoveryExportQuorumRequired => {
            sealed_audit_recovery_export_quorum_required_fixture()
        }
        PrototypeFixture::SealedAuditRecoveryExportRollbackRejected => {
            sealed_audit_recovery_export_rollback_rejected_fixture()
        }
        PrototypeFixture::SealedAuditRecoveryExportPlaintextRejected => {
            sealed_audit_recovery_export_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditDatabaseAdapterReady => {
            sealed_audit_database_adapter_ready_fixture()
        }
        PrototypeFixture::SealedAuditDatabaseAdapterEncryptionRejected => {
            sealed_audit_database_adapter_encryption_rejected_fixture()
        }
        PrototypeFixture::SealedAuditDatabaseAdapterAppendOnlyRejected => {
            sealed_audit_database_adapter_append_only_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportTransportReady => {
            sealed_audit_private_report_transport_ready_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportTransportPlaintextRejected => {
            sealed_audit_private_report_transport_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportOutboxReady => {
            sealed_audit_private_report_outbox_ready_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportOutboxTransportRejected => {
            sealed_audit_private_report_outbox_transport_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportOutboxReplayRejected => {
            sealed_audit_private_report_outbox_replay_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportOutboxRateLimitRejected => {
            sealed_audit_private_report_outbox_rate_limit_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportOutboxPlaintextRejected => {
            sealed_audit_private_report_outbox_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReceiptReady => {
            sealed_audit_private_report_receipt_ready_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReceiptOutboxRejected => {
            sealed_audit_private_report_receipt_outbox_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReceiptMissing => {
            sealed_audit_private_report_receipt_missing_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReceiptTransparencyRejected => {
            sealed_audit_private_report_receipt_transparency_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReceiptPlaintextRejected => {
            sealed_audit_private_report_receipt_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReconciliationReady => {
            sealed_audit_private_report_reconciliation_ready_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReconciliationReceiptRejected => {
            sealed_audit_private_report_reconciliation_receipt_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReconciliationRetryRejected => {
            sealed_audit_private_report_reconciliation_retry_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReconciliationFalseDeliveryRejected => {
            sealed_audit_private_report_reconciliation_false_delivery_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportReconciliationPlaintextRejected => {
            sealed_audit_private_report_reconciliation_plaintext_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceReady => {
            sealed_audit_private_report_gateway_evidence_ready_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceReconciliationRejected => {
            sealed_audit_private_report_gateway_evidence_reconciliation_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceUnavailableRejected => {
            sealed_audit_private_report_gateway_evidence_unavailable_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportGatewayEvidenceAccountabilityRejected => {
            sealed_audit_private_report_gateway_evidence_accountability_rejected_fixture()
        }
        PrototypeFixture::SealedAuditPrivateReportGatewayEvidencePlaintextRejected => {
            sealed_audit_private_report_gateway_evidence_plaintext_rejected_fixture()
        }
        PrototypeFixture::GroupChatMlsReady => group_chat_mls_ready_fixture(),
        PrototypeFixture::GroupChatMlsSetupRequired => group_chat_mls_setup_required_fixture(),
        PrototypeFixture::GroupChatMembershipSyncRequired => {
            group_chat_membership_sync_required_fixture()
        }
        PrototypeFixture::GroupChatPlaintextMetadataForbidden => {
            group_chat_plaintext_metadata_forbidden_fixture()
        }
        PrototypeFixture::GroupChatHighSecurityMlsRequired => {
            group_chat_high_security_mls_required_fixture()
        }
        PrototypeFixture::GroupChatHighSecurityPqRequired => {
            group_chat_high_security_pq_required_fixture()
        }
        PrototypeFixture::GroupChatMlsProviderSecurityRequired => {
            group_chat_mls_provider_security_required_fixture()
        }
        PrototypeFixture::MlsProviderEvidenceStoreReady => {
            mls_provider_evidence_store_ready_fixture()
        }
        PrototypeFixture::MlsProviderEvidenceStoreGateRejected => {
            mls_provider_evidence_store_gate_rejected_fixture()
        }
        PrototypeFixture::MlsProviderEvidenceStoreDuplicateRejected => {
            mls_provider_evidence_store_duplicate_rejected_fixture()
        }
        PrototypeFixture::MlsProviderEvidenceStorePlaintextRejected => {
            mls_provider_evidence_store_plaintext_rejected_fixture()
        }
        PrototypeFixture::MlsProviderEvidenceUseReady => mls_provider_evidence_use_ready_fixture(),
        PrototypeFixture::MlsProviderEvidenceUseMissing => {
            mls_provider_evidence_use_missing_fixture()
        }
        PrototypeFixture::MlsProviderEvidenceUseExpired => {
            mls_provider_evidence_use_expired_fixture()
        }
        PrototypeFixture::MlsProviderEvidenceUseSuiteMismatch => {
            mls_provider_evidence_use_suite_mismatch_fixture()
        }
        PrototypeFixture::MlsProviderEvidenceUsePlaintextRejected => {
            mls_provider_evidence_use_plaintext_rejected_fixture()
        }
        PrototypeFixture::MlsProviderAdapterSelectionReady => {
            mls_provider_adapter_selection_ready_fixture()
        }
        PrototypeFixture::MlsProviderAdapterSelectionProviderRejected => {
            mls_provider_adapter_selection_provider_rejected_fixture()
        }
        PrototypeFixture::MlsProviderAdapterSelectionPqDraftRejected => {
            mls_provider_adapter_selection_pq_draft_rejected_fixture()
        }
        PrototypeFixture::MlsProviderAdapterSelectionStorageRejected => {
            mls_provider_adapter_selection_storage_rejected_fixture()
        }
        PrototypeFixture::MlsProviderAdapterSelectionSupplyChainRejected => {
            mls_provider_adapter_selection_supply_chain_rejected_fixture()
        }
        PrototypeFixture::MlsKeyPackageAdmissionReady => mls_key_package_admission_ready_fixture(),
        PrototypeFixture::MlsKeyPackageAdmissionGroupRejected => {
            mls_key_package_admission_group_rejected_fixture()
        }
        PrototypeFixture::MlsKeyPackageAdmissionLifetimeRejected => {
            mls_key_package_admission_lifetime_rejected_fixture()
        }
        PrototypeFixture::MlsKeyPackageAdmissionSuiteMismatch => {
            mls_key_package_admission_suite_mismatch_fixture()
        }
        PrototypeFixture::MlsKeyPackageAdmissionCredentialRejected => {
            mls_key_package_admission_credential_rejected_fixture()
        }
        PrototypeFixture::MlsKeyPackageAdmissionReplayRejected => {
            mls_key_package_admission_replay_rejected_fixture()
        }
        PrototypeFixture::MlsKeyPackageAdmissionPlaintextRejected => {
            mls_key_package_admission_plaintext_rejected_fixture()
        }
        PrototypeFixture::MlsKeyPackageConsumeStoreReady => {
            mls_key_package_consume_store_ready_fixture()
        }
        PrototypeFixture::MlsKeyPackageConsumeStoreAdmissionRejected => {
            mls_key_package_consume_store_admission_rejected_fixture()
        }
        PrototypeFixture::MlsKeyPackageConsumeStoreDuplicateRejected => {
            mls_key_package_consume_store_duplicate_rejected_fixture()
        }
        PrototypeFixture::MlsKeyPackageConsumeStoreBadShape => {
            mls_key_package_consume_store_bad_shape_fixture()
        }
        PrototypeFixture::MlsKeyPackageConsumeStorePlaintextRejected => {
            mls_key_package_consume_store_plaintext_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeSendOutboxReady => mls_welcome_send_outbox_ready_fixture(),
        PrototypeFixture::MlsWelcomeSendOutboxConsumeRejected => {
            mls_welcome_send_outbox_consume_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeSendOutboxDuplicateTransactionRejected => {
            mls_welcome_send_outbox_duplicate_transaction_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeSendOutboxKeyPackageQueued => {
            mls_welcome_send_outbox_key_package_queued_fixture()
        }
        PrototypeFixture::MlsWelcomeSendOutboxBadShape => {
            mls_welcome_send_outbox_bad_shape_fixture()
        }
        PrototypeFixture::MlsWelcomeSendOutboxPlaintextRejected => {
            mls_welcome_send_outbox_plaintext_rejected_fixture()
        }
        PrototypeFixture::MlsMembershipTransactionReady => {
            mls_membership_transaction_ready_fixture()
        }
        PrototypeFixture::MlsMembershipTransactionBindingRejected => {
            mls_membership_transaction_binding_rejected_fixture()
        }
        PrototypeFixture::MlsMembershipTransactionStorageRejected => {
            mls_membership_transaction_storage_rejected_fixture()
        }
        PrototypeFixture::MlsMembershipTransactionDuplicateRejected => {
            mls_membership_transaction_duplicate_rejected_fixture()
        }
        PrototypeFixture::MlsMembershipTransactionPlaintextRejected => {
            mls_membership_transaction_plaintext_rejected_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseSecurityReady => {
            local_store_database_security_ready_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseSecurityPlaintextRejected => {
            local_store_database_security_plaintext_rejected_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseSecurityWalRejected => {
            local_store_database_security_wal_rejected_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseSecurityBackupRejected => {
            local_store_database_security_backup_rejected_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseSecuritySecretLifecycleRejected => {
            local_store_database_security_secret_lifecycle_rejected_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseAdapterSelectionReady => {
            local_store_database_adapter_selection_ready_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseAdapterSelectionLicenseRejected => {
            local_store_database_adapter_selection_license_rejected_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseAdapterSelectionFipsRejected => {
            local_store_database_adapter_selection_fips_rejected_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseAdapterSelectionMigrationRejected => {
            local_store_database_adapter_selection_migration_rejected_fixture()
        }
        PrototypeFixture::LocalStoreDatabaseAdapterSelectionSupplyChainRejected => {
            local_store_database_adapter_selection_supply_chain_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeAdmissionReady => mls_welcome_admission_ready_fixture(),
        PrototypeFixture::MlsWelcomeAdmissionSecretsMissing => {
            mls_welcome_admission_secrets_missing_fixture()
        }
        PrototypeFixture::MlsWelcomeAdmissionTreeRejected => {
            mls_welcome_admission_tree_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeAdmissionConfirmationRejected => {
            mls_welcome_admission_confirmation_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeAdmissionTieBreakRejected => {
            mls_welcome_admission_tie_break_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeAdmissionReplayRejected => {
            mls_welcome_admission_replay_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeAdmissionPlaintextRejected => {
            mls_welcome_admission_plaintext_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeReplayStoreReady => mls_welcome_replay_store_ready_fixture(),
        PrototypeFixture::MlsWelcomeReplayStoreAdmissionRejected => {
            mls_welcome_replay_store_admission_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeReplayStoreDuplicateRejected => {
            mls_welcome_replay_store_duplicate_rejected_fixture()
        }
        PrototypeFixture::MlsWelcomeReplayStoreKeyPackageReused => {
            mls_welcome_replay_store_key_package_reused_fixture()
        }
        PrototypeFixture::MlsWelcomeReplayStoreBadShape => {
            mls_welcome_replay_store_bad_shape_fixture()
        }
        PrototypeFixture::MlsWelcomeReplayStorePlaintextRejected => {
            mls_welcome_replay_store_plaintext_rejected_fixture()
        }
        PrototypeFixture::MlsCommitAdmissionReady => mls_commit_admission_ready_fixture(),
        PrototypeFixture::MlsCommitAdmissionBadEpoch => mls_commit_admission_bad_epoch_fixture(),
        PrototypeFixture::MlsCommitAdmissionAuthRejected => {
            mls_commit_admission_auth_rejected_fixture()
        }
        PrototypeFixture::MlsCommitAdmissionPathRejected => {
            mls_commit_admission_path_rejected_fixture()
        }
        PrototypeFixture::MlsCommitAdmissionTieBreakRejected => {
            mls_commit_admission_tie_break_rejected_fixture()
        }
        PrototypeFixture::MlsCommitAdmissionReplayRejected => {
            mls_commit_admission_replay_rejected_fixture()
        }
        PrototypeFixture::MlsCommitAdmissionPlaintextRejected => {
            mls_commit_admission_plaintext_rejected_fixture()
        }
        PrototypeFixture::MlsCommitReplayStoreReady => mls_commit_replay_store_ready_fixture(),
        PrototypeFixture::MlsCommitReplayStoreAdmissionRejected => {
            mls_commit_replay_store_admission_rejected_fixture()
        }
        PrototypeFixture::MlsCommitReplayStoreDuplicateRejected => {
            mls_commit_replay_store_duplicate_rejected_fixture()
        }
        PrototypeFixture::MlsCommitReplayStoreLocalMemberRemoved => {
            mls_commit_replay_store_local_member_removed_fixture()
        }
        PrototypeFixture::MlsCommitReplayStorePlaintextRejected => {
            mls_commit_replay_store_plaintext_rejected_fixture()
        }
        PrototypeFixture::GroupMessageTranscriptReady => group_message_transcript_ready_fixture(),
        PrototypeFixture::GroupMessageTranscriptSyncRequired => {
            group_message_transcript_sync_required_fixture()
        }
        PrototypeFixture::GroupMessageTranscriptRekeyRequired => {
            group_message_transcript_rekey_required_fixture()
        }
        PrototypeFixture::GroupMessageTranscriptStoreBindingRejected => {
            group_message_transcript_store_binding_rejected_fixture()
        }
        PrototypeFixture::AnonymousCredentialIssuerTrustReady => {
            anonymous_credential_issuer_trust_ready_fixture()
        }
        PrototypeFixture::AnonymousCredentialIssuerTrustTransparencyRequired => {
            anonymous_credential_issuer_trust_transparency_required_fixture()
        }
        PrototypeFixture::AnonymousCredentialIssuerTrustRevoked => {
            anonymous_credential_issuer_trust_revoked_fixture()
        }
        PrototypeFixture::AnonymousCredentialIssuerTrustPartitioningMetadataRejected => {
            anonymous_credential_issuer_trust_partitioning_metadata_rejected_fixture()
        }
        PrototypeFixture::AnonymousCredentialIssuerTrustWitnessAuditRejected => {
            anonymous_credential_issuer_trust_witness_audit_rejected_fixture()
        }
        PrototypeFixture::AnonymousGroupMembershipProofReady => {
            anonymous_group_membership_proof_ready_fixture()
        }
        PrototypeFixture::AnonymousGroupMembershipProofHighSecurityPqRequired => {
            anonymous_group_membership_proof_high_security_pq_required_fixture()
        }
        PrototypeFixture::AnonymousGroupMembershipProofReplayRejected => {
            anonymous_group_membership_proof_replay_rejected_fixture()
        }
        PrototypeFixture::AnonymousGroupMembershipProofRouteBindingRequired => {
            anonymous_group_membership_proof_route_binding_required_fixture()
        }
        PrototypeFixture::AnonymousGroupMembershipProofPlaintextIdentityRejected => {
            anonymous_group_membership_proof_plaintext_identity_rejected_fixture()
        }
        PrototypeFixture::AnonymousRateLimitNullifierReady => {
            anonymous_rate_limit_nullifier_ready_fixture()
        }
        PrototypeFixture::AnonymousRateLimitNullifierReplayRejected => {
            anonymous_rate_limit_nullifier_replay_rejected_fixture()
        }
        PrototypeFixture::AnonymousRateLimitNullifierLimitExceeded => {
            anonymous_rate_limit_nullifier_limit_exceeded_fixture()
        }
        PrototypeFixture::AnonymousRateLimitNullifierOpaqueStoreRequired => {
            anonymous_rate_limit_nullifier_opaque_store_required_fixture()
        }
        PrototypeFixture::AnonymousNullifierStoreReady => anonymous_nullifier_store_ready_fixture(),
        PrototypeFixture::AnonymousNullifierStoreReplayRejected => {
            anonymous_nullifier_store_replay_rejected_fixture()
        }
        PrototypeFixture::AnonymousNullifierStorePlaintextMetadataRejected => {
            anonymous_nullifier_store_plaintext_metadata_rejected_fixture()
        }
        PrototypeFixture::GroupRelayEnvelopeReady => group_relay_envelope_ready_fixture(),
        PrototypeFixture::GroupRelayEnvelopeTranscriptSyncRequired => {
            group_relay_envelope_transcript_sync_required_fixture()
        }
        PrototypeFixture::GroupRelayEnvelopeTranscriptRekeyRequired => {
            group_relay_envelope_transcript_rekey_required_fixture()
        }
        PrototypeFixture::GroupRelayEnvelopeMissingDeliveryToken => {
            group_relay_envelope_missing_delivery_token_fixture()
        }
        PrototypeFixture::GroupRelayEnvelopePlaintextMetadataRejected => {
            group_relay_envelope_plaintext_metadata_rejected_fixture()
        }
        PrototypeFixture::LocalStoreProductionOpenReady => {
            local_store_production_open_ready_fixture()
        }
        PrototypeFixture::LocalStoreProductionOpenWalReplayRequired => {
            local_store_production_open_wal_replay_required_fixture()
        }
        PrototypeFixture::LocalStoreProductionOpenPlaintextKeySlotForbidden => {
            local_store_production_open_plaintext_key_slot_forbidden_fixture()
        }
        PrototypeFixture::LocalStoreProductionOpenAppLockRequired => {
            local_store_production_open_app_lock_required_fixture()
        }
        PrototypeFixture::LocalStoreKeychainAndroidReady => {
            local_store_keychain_android_ready_fixture()
        }
        PrototypeFixture::LocalStoreKeychainUserAuthRequired => {
            local_store_keychain_user_auth_required_fixture()
        }
        PrototypeFixture::LocalStoreKeychainExportableSecretForbidden => {
            local_store_keychain_exportable_secret_forbidden_fixture()
        }
        PrototypeFixture::LocalStoreKeychainDevelopmentBackendForbidden => {
            local_store_keychain_development_backend_forbidden_fixture()
        }
        PrototypeFixture::ProductionStoreSessionHappyPath => {
            production_store_session_happy_path_fixture()
        }
        PrototypeFixture::ProductionStoreSessionKeychainRejected => {
            production_store_session_keychain_rejected_fixture()
        }
        PrototypeFixture::ProductionStoreSessionWalReplayRequired => {
            production_store_session_wal_replay_required_fixture()
        }
        PrototypeFixture::ProductionStoreSessionWriteRejected => {
            production_store_session_write_rejected_fixture()
        }
        PrototypeFixture::PlatformLocalStoreAdapterDesktopReady => {
            platform_local_store_adapter_desktop_ready_fixture()
        }
        PrototypeFixture::PlatformLocalStoreAdapterMobileHardwareRequired => {
            platform_local_store_adapter_mobile_hardware_required_fixture()
        }
        PrototypeFixture::PlatformLocalStoreAdapterPlaintextForbidden => {
            platform_local_store_adapter_plaintext_forbidden_fixture()
        }
        PrototypeFixture::PlatformLocalStoreAdapterAppLockRequired => {
            platform_local_store_adapter_app_lock_required_fixture()
        }
        PrototypeFixture::ReceiveSessionHappyPath => receive_session_happy_path_fixture(),
        PrototypeFixture::ReceiveSessionAckRejected => receive_session_ack_rejected_fixture(),
        PrototypeFixture::ReceiveSessionOrderingGap => receive_session_ordering_gap_fixture(),
        PrototypeFixture::ReceiveSessionStoreWriteRejected => {
            receive_session_store_write_rejected_fixture()
        }
        PrototypeFixture::InboundSyncDeliveryReady => inbound_sync_delivery_ready_fixture(),
        PrototypeFixture::InboundSyncIdle => inbound_sync_idle_fixture(),
        PrototypeFixture::InboundSyncBootstrapBlocked => inbound_sync_bootstrap_blocked_fixture(),
        PrototypeFixture::InboundSyncTransportOffline => inbound_sync_transport_offline_fixture(),
        PrototypeFixture::InboundSyncPlaintextPreviewForbidden => {
            inbound_sync_plaintext_preview_forbidden_fixture()
        }
        PrototypeFixture::AuthenticatedRelaySourceDeliveryReady => {
            authenticated_relay_source_delivery_ready_fixture()
        }
        PrototypeFixture::AuthenticatedRelaySourceIdle => authenticated_relay_source_idle_fixture(),
        PrototypeFixture::AuthenticatedRelaySourceAuthRejected => {
            authenticated_relay_source_auth_rejected_fixture()
        }
        PrototypeFixture::AuthenticatedRelaySourcePlaintextForbidden => {
            authenticated_relay_source_plaintext_forbidden_fixture()
        }
        PrototypeFixture::InboundSyncSessionHappyPath => inbound_sync_session_happy_path_fixture(),
        PrototypeFixture::InboundSyncSessionIdle => inbound_sync_session_idle_fixture(),
        PrototypeFixture::InboundSyncSessionSyncRejected => {
            inbound_sync_session_sync_rejected_fixture()
        }
        PrototypeFixture::InboundSyncSessionReceiveRejected => {
            inbound_sync_session_receive_rejected_fixture()
        }
        PrototypeFixture::MediaObjectStoreUploadReady => media_object_store_upload_ready_fixture(),
        PrototypeFixture::MediaObjectStorePlaintextRejected => {
            media_object_store_plaintext_rejected_fixture()
        }
        PrototypeFixture::MediaObjectStoreAutoDownloadRejected => {
            media_object_store_auto_download_rejected_fixture()
        }
        PrototypeFixture::MediaObjectStoreOversizeRejected => {
            media_object_store_oversize_rejected_fixture()
        }
        PrototypeFixture::MediaUploadSessionHappyPath => media_upload_session_happy_path_fixture(),
        PrototypeFixture::MediaUploadSessionPlaintextRejected => {
            media_upload_session_plaintext_rejected_fixture()
        }
        PrototypeFixture::MediaUploadSessionSealRejected => {
            media_upload_session_seal_rejected_fixture()
        }
        PrototypeFixture::MediaUploadSessionStoreWriteRejected => {
            media_upload_session_store_write_rejected_fixture()
        }
        PrototypeFixture::MediaServiceAdapterReady => media_service_adapter_ready_fixture(),
        PrototypeFixture::MediaServiceAdapterAuthMissing => {
            media_service_adapter_auth_missing_fixture()
        }
        PrototypeFixture::MediaServiceAdapterPlaintextForbidden => {
            media_service_adapter_plaintext_forbidden_fixture()
        }
        PrototypeFixture::MediaServiceAdapterDigestUnverified => {
            media_service_adapter_digest_unverified_fixture()
        }
        PrototypeFixture::MediaServiceUploadSessionHappyPath => {
            media_service_upload_session_happy_path_fixture()
        }
        PrototypeFixture::MediaServiceUploadSessionMediaRejected => {
            media_service_upload_session_media_rejected_fixture()
        }
        PrototypeFixture::MediaServiceUploadSessionAuthRejected => {
            media_service_upload_session_auth_rejected_fixture()
        }
        PrototypeFixture::MediaServiceUploadSessionDigestUnverified => {
            media_service_upload_session_digest_unverified_fixture()
        }
        PrototypeFixture::MediaServiceDownloadReady => media_service_download_ready_fixture(),
        PrototypeFixture::MediaServiceDownloadPlaintextPreviewRejected => {
            media_service_download_plaintext_preview_rejected_fixture()
        }
        PrototypeFixture::MediaServiceDownloadAuthMissing => {
            media_service_download_auth_missing_fixture()
        }
        PrototypeFixture::MediaServiceDownloadDigestUnverified => {
            media_service_download_digest_unverified_fixture()
        }
        PrototypeFixture::MediaDownloadSessionHappyPath => {
            media_download_session_happy_path_fixture()
        }
        PrototypeFixture::MediaDownloadSessionDownloadRejected => {
            media_download_session_download_rejected_fixture()
        }
        PrototypeFixture::MediaDownloadSessionStoreWriteRejected => {
            media_download_session_store_write_rejected_fixture()
        }
        PrototypeFixture::MediaDownloadSessionOpenRejected => {
            media_download_session_open_rejected_fixture()
        }
        PrototypeFixture::MediaRetentionDeleteAndEvictReady => {
            media_retention_delete_and_evict_ready_fixture()
        }
        PrototypeFixture::MediaRetentionRetainReady => media_retention_retain_ready_fixture(),
        PrototypeFixture::MediaRetentionHoldRejected => media_retention_hold_rejected_fixture(),
        PrototypeFixture::MediaRetentionAuthMissing => media_retention_auth_missing_fixture(),
        PrototypeFixture::MediaCleanupSessionHappyPath => {
            media_cleanup_session_happy_path_fixture()
        }
        PrototypeFixture::MediaCleanupSessionRetainReady => {
            media_cleanup_session_retain_ready_fixture()
        }
        PrototypeFixture::MediaCleanupSessionRetentionRejected => {
            media_cleanup_session_retention_rejected_fixture()
        }
        PrototypeFixture::MediaCleanupSessionCacheAbsent => {
            media_cleanup_session_cache_absent_fixture()
        }
        PrototypeFixture::MediaObjectIndexRemoteAndLocalReady => {
            media_object_index_remote_and_local_ready_fixture()
        }
        PrototypeFixture::MediaObjectIndexAbsentUploadReady => {
            media_object_index_absent_upload_ready_fixture()
        }
        PrototypeFixture::MediaObjectIndexDeletePendingReady => {
            media_object_index_delete_pending_ready_fixture()
        }
        PrototypeFixture::MediaObjectIndexDeletedTerminal => {
            media_object_index_deleted_terminal_fixture()
        }
        PrototypeFixture::MediaObjectIndexPlaintextMetadataRejected => {
            media_object_index_plaintext_metadata_rejected_fixture()
        }
        PrototypeFixture::MediaObjectIndexBadLifecycleRejected => {
            media_object_index_bad_lifecycle_rejected_fixture()
        }
        PrototypeFixture::MediaObjectIndexStoreWriteReady => {
            media_object_index_store_write_ready_fixture()
        }
        PrototypeFixture::MediaObjectIndexStoreIndexRejected => {
            media_object_index_store_index_rejected_fixture()
        }
        PrototypeFixture::MediaObjectIndexStoreBadObjectRejected => {
            media_object_index_store_bad_object_rejected_fixture()
        }
        PrototypeFixture::MediaObjectIndexStoreDeletedSnapshot => {
            media_object_index_store_deleted_snapshot_fixture()
        }
        PrototypeFixture::MediaObjectIndexProductionOpenReady => {
            media_object_index_production_open_ready_fixture()
        }
        PrototypeFixture::MediaObjectIndexProductionOpenWalReplayRequired => {
            media_object_index_production_open_wal_replay_required_fixture()
        }
        PrototypeFixture::MediaObjectIndexProductionOpenPlaintextMetadataForbidden => {
            media_object_index_production_open_plaintext_metadata_forbidden_fixture()
        }
        PrototypeFixture::MediaObjectIndexProductionOpenNamespaceUnbound => {
            media_object_index_production_open_namespace_unbound_fixture()
        }
        PrototypeFixture::IndexedMediaUploadSessionHappyPath => {
            indexed_media_upload_session_happy_path_fixture()
        }
        PrototypeFixture::IndexedMediaUploadSessionServiceRejected => {
            indexed_media_upload_session_service_rejected_fixture()
        }
        PrototypeFixture::IndexedMediaUploadSessionIndexStoreRejected => {
            indexed_media_upload_session_index_store_rejected_fixture()
        }
        PrototypeFixture::IndexedMediaDownloadSessionHappyPath => {
            indexed_media_download_session_happy_path_fixture()
        }
        PrototypeFixture::IndexedMediaDownloadSessionManifestRejected => {
            indexed_media_download_session_manifest_rejected_fixture()
        }
        PrototypeFixture::IndexedMediaDownloadSessionNotDownloadable => {
            indexed_media_download_session_not_downloadable_fixture()
        }
        PrototypeFixture::IndexedMediaDownloadSessionDownloadRejected => {
            indexed_media_download_session_download_rejected_fixture()
        }
        PrototypeFixture::IndexedMediaCleanupSessionHappyPath => {
            indexed_media_cleanup_session_happy_path_fixture()
        }
        PrototypeFixture::IndexedMediaCleanupSessionManifestRejected => {
            indexed_media_cleanup_session_manifest_rejected_fixture()
        }
        PrototypeFixture::IndexedMediaCleanupSessionNotCleanable => {
            indexed_media_cleanup_session_not_cleanable_fixture()
        }
        PrototypeFixture::IndexedMediaCleanupSessionCleanupRejected => {
            indexed_media_cleanup_session_cleanup_rejected_fixture()
        }
        PrototypeFixture::CryptoSealOpenRoundtrip => crypto_seal_open_roundtrip_fixture(),
        PrototypeFixture::RelayDeliveryOnce => relay_delivery_once_fixture(),
        PrototypeFixture::AiParticipantDraftAccepted => ai_participant_draft_accepted_fixture(),
        PrototypeFixture::AiConnectorLocalDraftReady => ai_connector_local_draft_ready_fixture(),
        PrototypeFixture::AiConnectorRemoteForbidden => ai_connector_remote_forbidden_fixture(),
        PrototypeFixture::AiConnectorPlaintextBridgeRejected => {
            ai_connector_plaintext_bridge_rejected_fixture()
        }
        PrototypeFixture::AiConnectorRetentionRejected => ai_connector_retention_rejected_fixture(),
        PrototypeFixture::AiConnectorUserSelectionRequired => {
            ai_connector_user_selection_required_fixture()
        }
        PrototypeFixture::BackendSessionHappyPath => backend_session_happy_path_fixture(),
        PrototypeFixture::BackendSessionBootstrapBlocked => {
            backend_session_bootstrap_blocked_fixture()
        }
        PrototypeFixture::BackendSessionRelayRejected => backend_session_relay_rejected_fixture(),
        PrototypeFixture::BackendSessionAiRejected => backend_session_ai_rejected_fixture(),
    }
}

pub fn prototype_fixture_json(fixture: PrototypeFixture) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&prototype_fixture_value(fixture))
}

pub fn backend_command_value(descriptor: BackendCommandDescriptor) -> Value {
    let command =
        PrototypeBackendCommand::new(32, descriptor.actor_kind, descriptor.command_kind, 0);

    json!({
        "command": backend_command_view_value(command.view()),
        "result": prototype_fixture_value(descriptor.result_fixture),
    })
}

pub fn backend_command_json(descriptor: BackendCommandDescriptor) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&backend_command_value(descriptor))
}

pub fn platform_bridge_response_value(request: PlatformBridgeRequest<'_>) -> Value {
    platform_bridge_response_from_parts(
        request.request_id,
        request.operation.label(),
        request.target,
        request.plaintext_payload_len,
    )
}

pub fn platform_bridge_handle_json(request_json: &str) -> serde_json::Result<String> {
    let value: Value = serde_json::from_str(request_json)?;
    let request_id = value
        .get("request_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let operation = value.get("operation").and_then(Value::as_str).unwrap_or("");
    let target = value
        .get("target")
        .or_else(|| value.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut plaintext_payload_len = value
        .get("plaintext_payload_len")
        .and_then(Value::as_u64)
        .and_then(|len| usize::try_from(len).ok())
        .unwrap_or(0);

    if value.get("plaintext_payload").is_some()
        || value
            .get("payload")
            .and_then(|payload| payload.get("plaintext"))
            .is_some()
    {
        plaintext_payload_len = plaintext_payload_len.max(1);
    }

    serde_json::to_string_pretty(&platform_bridge_response_from_parts(
        request_id,
        operation,
        target,
        plaintext_payload_len,
    ))
}

fn backend_command_view_value(view: PrototypeBackendCommandView) -> Value {
    json!({
        "command_id_len": view.command_id_len,
        "actor_kind_code": view.actor_kind_code,
        "actor_kind_label": view.actor_kind_label,
        "command_kind_code": view.command_kind_code,
        "command_kind_label": view.command_kind_label,
        "accepted": view.accepted,
        "reason_code": view.reason_code,
        "reason_label": view.reason_label,
        "can_run_session": view.can_run_session,
        "can_request_ai_draft": view.can_request_ai_draft,
        "emits_event_stream": view.emits_event_stream,
        "plaintext_payload_len": view.plaintext_payload_len,
    })
}

fn platform_bridge_response_from_parts(
    request_id: &str,
    operation_label: &str,
    target: &str,
    plaintext_payload_len: usize,
) -> Value {
    let operation = PlatformBridgeOperation::from_label(operation_label);
    let body_candidate =
        operation.and_then(|operation| platform_bridge_body_value(operation, target));
    let reason = if request_id.len() != PLATFORM_BRIDGE_REQUEST_ID_LEN {
        PlatformBridgeReason::BadRequestIdLength
    } else if plaintext_payload_len != 0 {
        PlatformBridgeReason::PlaintextPayloadForbidden
    } else if operation.is_none() {
        PlatformBridgeReason::UnknownOperation
    } else if body_candidate.is_none() {
        PlatformBridgeReason::UnknownTarget
    } else {
        PlatformBridgeReason::Accepted
    };

    let body = if reason == PlatformBridgeReason::Accepted {
        body_candidate.expect("accepted bridge body")
    } else {
        Value::Null
    };

    json!({
        "bridge": {
            "request_id_len": request_id.len(),
            "operation_code": operation.map(PlatformBridgeOperation::code).unwrap_or(0),
            "operation_label": operation
                .map(PlatformBridgeOperation::label)
                .unwrap_or(operation_label),
            "target": target,
            "accepted": reason == PlatformBridgeReason::Accepted,
            "reason_code": reason.code(),
            "reason_label": reason.label(),
            "plaintext_payload_len": plaintext_payload_len,
        },
        "body": body,
    })
}

fn platform_bridge_body_value(operation: PlatformBridgeOperation, target: &str) -> Option<Value> {
    match operation {
        PlatformBridgeOperation::PlatformFixture => platform_fixture_by_name(target)
            .and_then(|fixture| serde_json::to_value(platform_fixture_view(fixture)).ok()),
        PlatformBridgeOperation::PrototypeFixture => {
            prototype_fixture_by_name(target).map(prototype_fixture_value)
        }
        PlatformBridgeOperation::BackendCommand => {
            backend_command_by_name(target).map(backend_command_value)
        }
    }
}

const fn policy_decision(reason: PipelineReason) -> PolicyDecision {
    PolicyDecision {
        accepted: matches!(reason, PipelineReason::Accept),
        reason_code: reason.code(),
        audit_class: match reason {
            PipelineReason::AiGrantReject => PipelineAuditClass::AiPolicyReject.code(),
            PipelineReason::AiLifecycleReject => PipelineAuditClass::AiLifecyclePolicyReject.code(),
            PipelineReason::Accept => PipelineAuditClass::AcceptedPolicyDecision.code(),
            _ => PipelineAuditClass::PipelineContractReject.code(),
        },
        components: ComponentReasons {
            envelope_reason: 0,
            room_epoch_reason: 0,
            ai_grant_reason: 0,
            ai_lifecycle_reason: 0,
        },
    }
}

fn local_store_sealed_message_fixture() -> Value {
    let mut store = PrototypeEncryptedLocalStore::default();
    let decision = store.put_record(LocalStoreWriteRequest::new(
        store_locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStorePayload::sealed(&SEALED_MESSAGE_BYTES),
        Some(store_policy_decision(true)),
    ));
    let record = store
        .get_record(store_locator("conversation-7", "message-42"))
        .expect("fixture should store accepted sealed record");

    json!({
        "fixture": "local_store_sealed_message",
        "surface": "prototype_local_store",
        "record_count": store.len(),
        "write": {
            "accepted": decision.accepted,
            "reason": format!("{:?}", decision.reason),
            "plaintext_class": format!("{:?}", decision.record_policy.plaintext_class),
            "key_scope": format!("{:?}", decision.record_policy.key_scope),
        },
        "records": [
            {
                "namespace": record.namespace,
                "record_id": record.record_id,
                "record_kind": format!("{:?}", record.record_kind),
                "payload_kind": format!("{:?}", record.payload_kind),
                "byte_len": record.bytes.len(),
                "policy_accepted": record
                    .policy_decision
                    .map(|policy| policy.accepted)
                    .unwrap_or(false),
                "write_accepted": record.write_decision.accepted,
                "write_reason": format!("{:?}", record.write_decision.reason),
                "plaintext_bytes_exposed": false,
            }
        ],
    })
}

fn local_store_unlock_ready_fixture() -> Value {
    local_store_unlock_fixture("local_store_unlock_ready", valid_local_store_unlock_input())
}

fn local_store_unlock_app_lock_required_fixture() -> Value {
    let mut input = valid_local_store_unlock_input();
    input.app_lock_satisfied = false;
    local_store_unlock_fixture("local_store_unlock_app_lock_required", input)
}

fn local_store_unlock_recovery_required_fixture() -> Value {
    let mut input = valid_local_store_unlock_input();
    input.recovery_required = true;
    local_store_unlock_fixture("local_store_unlock_recovery_required", input)
}

fn local_store_unlock_plaintext_cache_forbidden_fixture() -> Value {
    let mut input = valid_local_store_unlock_input();
    input.plaintext_cache_records = 1;
    local_store_unlock_fixture("local_store_unlock_plaintext_cache_forbidden", input)
}

fn local_store_unlock_fixture(name: &'static str, input: LocalStoreUnlockInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "local_store_unlock",
        "input": {
            "store_version": input.store_version,
            "keychain_available": input.keychain_available,
            "device_secret": format!("{:?}", input.device_secret),
            "database_header": format!("{:?}", input.database_header),
            "app_lock_satisfied": input.app_lock_satisfied,
            "recovery_required": input.recovery_required,
            "plaintext_cache_records": input.plaintext_cache_records,
        },
        "decision": local_store_unlock_decision_value(decision),
    })
}

fn local_store_unlock_decision_value(decision: LocalStoreUnlockDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_open_database": decision.can_open_database,
        "can_unseal_device_secret": decision.can_unseal_device_secret,
        "can_load_message_keys": decision.can_load_message_keys,
        "requires_user_auth": decision.requires_user_auth,
        "requires_recovery": decision.requires_recovery,
        "requires_migration": decision.requires_migration,
        "requires_destructive_repair": decision.requires_destructive_repair,
    })
}

fn account_recovery_high_entropy_ready_fixture() -> Value {
    account_recovery_fixture(
        "account_recovery_high_entropy_ready",
        valid_account_recovery_input(),
    )
}

fn account_recovery_low_entropy_pin_forbidden_fixture() -> Value {
    let mut input = valid_account_recovery_input();
    input.method = AccountRecoveryMethod::LowEntropyPin;
    input.recovery_key_entropy_bits = 32;
    account_recovery_fixture("account_recovery_low_entropy_pin_forbidden", input)
}

fn account_recovery_threshold_quorum_required_fixture() -> Value {
    let mut input = valid_account_recovery_input();
    input.method = AccountRecoveryMethod::ThresholdRecovery;
    input.threshold_shares = 3;
    input.threshold_required = 2;
    input.threshold_approvals = 1;
    account_recovery_fixture("account_recovery_threshold_quorum_required", input)
}

fn account_recovery_plaintext_backup_forbidden_fixture() -> Value {
    let mut input = valid_account_recovery_input();
    input.plaintext_backup_fields = 1;
    account_recovery_fixture("account_recovery_plaintext_backup_forbidden", input)
}

fn account_recovery_key_rotation_required_fixture() -> Value {
    let mut input = valid_account_recovery_input();
    input.high_security_account = true;
    input.recovery_key_entropy_bits = 192;
    input.rotates_device_secret = false;
    account_recovery_fixture("account_recovery_key_rotation_required", input)
}

fn account_recovery_fixture(name: &'static str, input: AccountRecoveryInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "account_recovery",
        "input": {
            "recovery_requested": input.recovery_requested,
            "method_code": input.method.code(),
            "method_label": input.method.label(),
            "high_security_account": input.high_security_account,
            "recovery_key_entropy_bits": input.recovery_key_entropy_bits,
            "recovery_key_digest_len": input.recovery_key_digest_len,
            "threshold_shares": input.threshold_shares,
            "threshold_required": input.threshold_required,
            "threshold_approvals": input.threshold_approvals,
            "device_approval_present": input.device_approval_present,
            "server_authenticated": input.server_authenticated,
            "server_rate_limited": input.server_rate_limited,
            "backup_encrypted": input.backup_encrypted,
            "plaintext_backup_fields": input.plaintext_backup_fields,
            "rotates_device_secret": input.rotates_device_secret,
            "audit_digest_len": input.audit_digest_len,
        },
        "decision": account_recovery_decision_value(decision),
    })
}

fn account_recovery_decision_value(decision: AccountRecoveryDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "method_code": decision.method.code(),
        "method_label": decision.method.label(),
        "can_start_recovery": decision.can_start_recovery,
        "can_restore_device_secret": decision.can_restore_device_secret,
        "requires_user_action": decision.requires_user_action,
        "requires_server_setup": decision.requires_server_setup,
        "requires_key_rotation": decision.requires_key_rotation,
        "forbids_low_entropy_recovery": decision.forbids_low_entropy_recovery,
        "forbids_plaintext_backup": decision.forbids_plaintext_backup,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn secure_backup_restore_ready_fixture() -> Value {
    secure_backup_restore_fixture(
        "secure_backup_restore_ready",
        valid_secure_backup_restore_input(),
    )
}

fn secure_backup_restore_recovery_rejected_fixture() -> Value {
    let mut recovery = valid_account_recovery_input();
    recovery.method = AccountRecoveryMethod::LowEntropyPin;
    recovery.recovery_key_entropy_bits = 32;

    let mut input = valid_secure_backup_restore_input();
    input.account_recovery = recovery.evaluate();
    secure_backup_restore_fixture("secure_backup_restore_recovery_rejected", input)
}

fn secure_backup_restore_plaintext_rejected_fixture() -> Value {
    let mut input = valid_secure_backup_restore_input();
    input.plaintext_export_fields = 1;
    secure_backup_restore_fixture("secure_backup_restore_plaintext_rejected", input)
}

fn secure_backup_restore_mls_rekey_rejected_fixture() -> Value {
    let mut input = valid_secure_backup_restore_input();
    input.restore_rekeys_groups = false;
    secure_backup_restore_fixture("secure_backup_restore_mls_rekey_rejected", input)
}

fn secure_backup_restore_cloud_policy_rejected_fixture() -> Value {
    let mut input = valid_secure_backup_restore_input();
    input.os_plaintext_backup_excluded = false;
    secure_backup_restore_fixture("secure_backup_restore_cloud_policy_rejected", input)
}

fn secure_backup_restore_fixture(name: &'static str, input: SecureBackupRestoreInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "secure_backup_restore",
        "input": {
            "account_recovery_accepted": input.account_recovery.accepted,
            "account_recovery_reason_code": input.account_recovery.reason.code(),
            "account_recovery_reason_label": input.account_recovery.reason.label(),
            "scope_code": input.scope.code(),
            "scope_label": input.scope.label(),
            "transport_code": input.transport.code(),
            "transport_label": input.transport.label(),
            "envelope_suite_code": input.envelope_suite.code(),
            "envelope_suite_label": input.envelope_suite.label(),
            "high_security_account": input.high_security_account,
            "backup_key_entropy_bits": input.backup_key_entropy_bits,
            "backup_key_digest_len": input.backup_key_digest_len,
            "kdf_memory_cost_mib": input.kdf_memory_cost_mib,
            "kdf_iterations": input.kdf_iterations,
            "device_approval_present": input.device_approval_present,
            "threshold_shares": input.threshold_shares,
            "threshold_required": input.threshold_required,
            "threshold_approvals": input.threshold_approvals,
            "server_authenticated": input.server_authenticated,
            "server_rate_limited": input.server_rate_limited,
            "opaque_account_identifier": input.opaque_account_identifier,
            "backup_encrypted": input.backup_encrypted,
            "plaintext_export_fields": input.plaintext_export_fields,
            "os_plaintext_backup_excluded": input.os_plaintext_backup_excluded,
            "mls_state_included": input.mls_state_included,
            "mls_state_sealed": input.mls_state_sealed,
            "mls_epoch_bound": input.mls_epoch_bound,
            "restore_rotates_device_secret": input.restore_rotates_device_secret,
            "restore_rekeys_groups": input.restore_rekeys_groups,
            "archive_manifest_authenticated": input.archive_manifest_authenticated,
            "replay_nonce_len": input.replay_nonce_len,
            "audit_digest_len": input.audit_digest_len,
            "retention_days": input.retention_days,
            "max_retention_days": input.max_retention_days,
        },
        "decision": secure_backup_restore_decision_value(decision),
    })
}

fn secure_backup_restore_decision_value(decision: SecureBackupRestoreDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "scope_code": decision.scope.code(),
        "scope_label": decision.scope.label(),
        "transport_code": decision.transport.code(),
        "transport_label": decision.transport.label(),
        "envelope_suite_code": decision.envelope_suite.code(),
        "envelope_suite_label": decision.envelope_suite.label(),
        "can_create_backup": decision.can_create_backup,
        "can_restore_device": decision.can_restore_device,
        "can_restore_mls_state": decision.can_restore_mls_state,
        "can_use_cloud_storage": decision.can_use_cloud_storage,
        "requires_user_action": decision.requires_user_action,
        "requires_server_setup": decision.requires_server_setup,
        "requires_device_rekey": decision.requires_device_rekey,
        "requires_group_rekey": decision.requires_group_rekey,
        "requires_backup_reconfiguration": decision.requires_backup_reconfiguration,
        "forbids_plaintext_export": decision.forbids_plaintext_export,
        "forbids_os_plaintext_backup": decision.forbids_os_plaintext_backup,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_event_chain_ready_fixture() -> Value {
    sealed_audit_event_chain_fixture(
        "sealed_audit_event_chain_ready",
        valid_sealed_audit_event_chain_input(),
    )
}

fn sealed_audit_event_chain_plaintext_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_event_chain_input();
    input.plaintext_field_count = 1;
    sealed_audit_event_chain_fixture("sealed_audit_event_chain_plaintext_rejected", input)
}

fn sealed_audit_event_chain_rollback_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_event_chain_input();
    input.rollback_resistant_store = false;
    sealed_audit_event_chain_fixture("sealed_audit_event_chain_rollback_rejected", input)
}

fn sealed_audit_event_chain_witness_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_event_chain_input();
    input.witness_count = 1;
    sealed_audit_event_chain_fixture("sealed_audit_event_chain_witness_rejected", input)
}

fn sealed_audit_event_chain_binding_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_event_chain_input();
    input.room_epoch_digest_len = 0;
    sealed_audit_event_chain_fixture("sealed_audit_event_chain_binding_rejected", input)
}

fn sealed_audit_event_chain_fixture(
    name: &'static str,
    input: SealedAuditEventChainInput,
) -> Value {
    let decision = evaluate_sealed_audit_event_chain(input);
    let chain_shape = json!({
        "event_sequence": input.event_sequence,
        "previous_chain_size": input.previous_chain_size,
        "previous_event_hash_len": input.previous_event_hash_len,
        "event_hash_len": input.event_hash_len,
        "record_digest_len": input.record_digest_len,
        "merkle_leaf_hash_len": input.merkle_leaf_hash_len,
        "merkle_root_hash_len": input.merkle_root_hash_len,
        "monotonic_counter_present": input.monotonic_counter_present,
        "monotonic_counter_increases": input.monotonic_counter_increases,
    });
    let binding = json!({
        "device_binding_digest_len": input.device_binding_digest_len,
        "actor_binding_digest_len": input.actor_binding_digest_len,
        "epoch_binding_digest_len": input.epoch_binding_digest_len,
        "room_epoch_digest_len": input.room_epoch_digest_len,
        "critical_event_bound": input.critical_event_bound,
        "event_sealed": input.event_sealed,
        "aad_binds_event_context": input.aad_binds_event_context,
        "plaintext_field_count": input.plaintext_field_count,
        "plaintext_payload_bytes": input.plaintext_payload_bytes,
    });
    let transparency = json!({
        "signed_checkpoint_present": input.signed_checkpoint_present,
        "checkpoint_signature_len": input.checkpoint_signature_len,
        "checkpoint_timestamp_s": input.checkpoint_timestamp_s,
        "checkpoint_size": input.checkpoint_size,
        "previous_checkpoint_size": input.previous_checkpoint_size,
        "inclusion_proof_verified": input.inclusion_proof_verified,
        "consistency_proof_verified": input.consistency_proof_verified,
        "transparency_receipt_present": input.transparency_receipt_present,
        "witness_count": input.witness_count,
        "witness_threshold": input.witness_threshold,
        "witness_operator_count": input.witness_operator_count,
    });
    let storage = json!({
        "storage_append_only": input.storage_append_only,
        "storage_transactional": input.storage_transactional,
        "rollback_resistant_store": input.rollback_resistant_store,
        "local_store_sealed": input.local_store_sealed,
        "forward_secret_rotated": input.forward_secret_rotated,
        "previous_key_material_deleted": input.previous_key_material_deleted,
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_event_chain",
        "input": {
            "event_kind_code": input.event_kind.code(),
            "event_kind_label": input.event_kind.label(),
            "anchor_kind_code": input.anchor_kind.code(),
            "anchor_kind_label": input.anchor_kind.label(),
            "envelope_suite_code": input.envelope_suite.code(),
            "envelope_suite_label": input.envelope_suite.label(),
            "chain_shape": chain_shape,
            "binding": binding,
            "transparency": transparency,
            "storage": storage,
        },
        "decision": sealed_audit_event_chain_decision_value(decision),
    })
}

fn sealed_audit_event_chain_decision_value(decision: SealedAuditEventChainDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "event_kind_code": decision.event_kind.code(),
        "event_kind_label": decision.event_kind.label(),
        "anchor_kind_code": decision.anchor_kind.code(),
        "anchor_kind_label": decision.anchor_kind.label(),
        "envelope_suite_code": decision.envelope_suite.code(),
        "envelope_suite_label": decision.envelope_suite.label(),
        "event_sequence": decision.event_sequence,
        "can_append_event": decision.can_append_event,
        "can_verify_inclusion": decision.can_verify_inclusion,
        "can_publish_transparency_receipt": decision.can_publish_transparency_receipt,
        "can_detect_rollback": decision.can_detect_rollback,
        "requires_storage_repair": decision.requires_storage_repair,
        "requires_transparency_setup": decision.requires_transparency_setup,
        "requires_redaction": decision.requires_redaction,
        "requires_key_rotation": decision.requires_key_rotation,
        "tamper_evident": decision.tamper_evident,
        "append_only": decision.append_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

const SEALED_AUDIT_EVENT_HASH: [u8; 32] = [0xB1; 32];
const SEALED_AUDIT_OTHER_EVENT_HASH: [u8; 32] = [0xB2; 32];
const SEALED_AUDIT_PREVIOUS_EVENT_HASH: [u8; 32] = [0xB3; 32];
const SEALED_AUDIT_RECORD_DIGEST: [u8; 32] = [0xB4; 32];
const SEALED_AUDIT_OTHER_RECORD_DIGEST: [u8; 32] = [0xB5; 32];
const SEALED_AUDIT_MERKLE_ROOT_HASH: [u8; 32] = [0xB6; 32];
const SEALED_AUDIT_CHECKPOINT_ID: [u8; 32] = [0xB7; 32];
const SEALED_AUDIT_OTHER_CHECKPOINT_ID: [u8; 32] = [0xB8; 32];
const SEALED_AUDIT_CHECKPOINT_SIGNATURE: [u8; 64] = [0xB9; 64];
const SEALED_AUDIT_TRANSPARENCY_RECEIPT: [u8; 96] = [0xBA; 96];
const SEALED_AUDIT_WITNESS_RECEIPT: [u8; 96] = [0xBB; 96];
const SEALED_AUDIT_PROOF_BUNDLE_DIGEST: [u8; 32] = [0xBC; 32];
const SEALED_AUDIT_PROOF_CHECKPOINT_DIGEST: [u8; 32] = [0xBE; 32];
const SEALED_AUDIT_POLICY_SNAPSHOT_DIGEST: [u8; 32] = [0xBF; 32];
const SEALED_AUDIT_PROOF_CACHE_DIGEST: [u8; 32] = [0xC0; 32];
const SEALED_AUDIT_VERIFIER_POLICY_DIGEST: [u8; 32] = [0xC1; 32];
const SEALED_AUDIT_NEXT_VERIFIER_POLICY_DIGEST: [u8; 32] = [0xC2; 32];
const SEALED_AUDIT_LOG_KEY_PINSET_DIGEST: [u8; 32] = [0xC3; 32];
const SEALED_AUDIT_WITNESS_KEY_PINSET_DIGEST: [u8; 32] = [0xC4; 32];
const SEALED_AUDIT_MONITOR_QUERY_PLAN_DIGEST: [u8; 32] = [0xC5; 32];
const SEALED_AUDIT_INCIDENT_ID: [u8; 32] = [0xD1; 32];
const SEALED_AUDIT_CONTRADICTION_DIGEST: [u8; 32] = [0xD2; 32];
const SEALED_AUDIT_MISSING_PROOF_REPORT_DIGEST: [u8; 32] = [0xD3; 32];
const SEALED_AUDIT_MONITOR_REPORT_DIGEST: [u8; 32] = [0xD4; 32];
const SEALED_AUDIT_ACCOUNTABILITY_ROUTE_DIGEST: [u8; 32] = [0xD5; 32];
const SEALED_AUDIT_RECOVERY_EXPORT_MANIFEST_DIGEST: [u8; 32] = [0xE1; 32];
const SEALED_AUDIT_NEXT_RECOVERY_EXPORT_MANIFEST_DIGEST: [u8; 32] = [0xE2; 32];
const SEALED_AUDIT_DEVICE_SET_DIGEST: [u8; 32] = [0xE3; 32];
const SEALED_AUDIT_RECOVERY_POLICY_DIGEST: [u8; 32] = [0xE4; 32];
const SEALED_AUDIT_EXPORT_CIPHERTEXT_DIGEST: [u8; 32] = [0xE5; 32];
const SEALED_AUDIT_RESTORE_AUTHORIZATION_DIGEST: [u8; 32] = [0xE6; 32];
const SEALED_AUDIT_SYNC_STATE_DIGEST: [u8; 32] = [0xE7; 32];
const SEALED_AUDIT_DATABASE_PROFILE_DIGEST: [u8; 32] = [0xE8; 32];
const SEALED_AUDIT_DATABASE_SCHEMA_DIGEST: [u8; 32] = [0xE9; 32];
const SEALED_AUDIT_EVENT_TABLE_DIGEST: [u8; 32] = [0xEA; 32];
const SEALED_AUDIT_PROOF_CACHE_TABLE_DIGEST: [u8; 32] = [0xEB; 32];
const SEALED_AUDIT_VERIFIER_POLICY_TABLE_DIGEST: [u8; 32] = [0xEC; 32];
const SEALED_AUDIT_INCIDENT_EVIDENCE_TABLE_DIGEST: [u8; 32] = [0xED; 32];
const SEALED_AUDIT_RECOVERY_EXPORT_TABLE_DIGEST: [u8; 32] = [0xEE; 32];
const SEALED_AUDIT_CHECKPOINT_TABLE_DIGEST: [u8; 32] = [0xEF; 32];
const SEALED_AUDIT_DATABASE_MIGRATION_PLAN_DIGEST: [u8; 32] = [0xF0; 32];
const SEALED_AUDIT_DATABASE_CRASH_RECOVERY_PLAN_DIGEST: [u8; 32] = [0xF1; 32];
const SEALED_AUDIT_REPORT_TRANSPORT_CONFIG_DIGEST: [u8; 32] = [0xF2; 32];
const SEALED_AUDIT_OHTTP_GATEWAY_KEY_DIGEST: [u8; 32] = [0xF3; 32];
const SEALED_AUDIT_OHTTP_RELAY_POLICY_DIGEST: [u8; 32] = [0xF4; 32];
const SEALED_AUDIT_PRIVACY_PASS_ISSUER_KEY_DIGEST: [u8; 32] = [0xF5; 32];
const SEALED_AUDIT_PRIVATE_REPORT_OUTBOX_DIGEST: [u8; 32] = [0xF6; 32];
const SEALED_AUDIT_REPORT_REPLAY_WINDOW_DIGEST: [u8; 32] = [0xF7; 32];
const SEALED_AUDIT_REPORT_RATE_LIMIT_BUCKET_DIGEST: [u8; 32] = [0xF8; 32];
const SEALED_AUDIT_REPORT_RETRY_BACKOFF_DIGEST: [u8; 32] = [0xF9; 32];
const SEALED_AUDIT_INCIDENT_REPORT_SCHEMA_DIGEST: [u8; 32] = [0xFA; 32];
const SEALED_AUDIT_REPORT_AUDIT_CHECKPOINT_DIGEST: [u8; 32] = [0xFB; 32];
const SEALED_AUDIT_PRIVATE_REPORT_ID: [u8; 32] = [0xFC; 32];
const SEALED_AUDIT_PRIVATE_REPORT_PAYLOAD_DIGEST: [u8; 32] = [0xFE; 32];
const SEALED_AUDIT_PRIVATE_REPORT_REQUEST_TRANSCRIPT_DIGEST: [u8; 32] = [0xF1; 32];
const SEALED_AUDIT_PRIVATE_REPORT_RESPONSE_TRANSCRIPT_DIGEST: [u8; 32] = [0xF2; 32];
const SEALED_AUDIT_PRIVATE_REPORT_RECEIPT_ID: [u8; 32] = [0xC6; 32];
const SEALED_AUDIT_GATEWAY_RECEIPT_DIGEST: [u8; 32] = [0xC7; 32];
const SEALED_AUDIT_GATEWAY_SIGNATURE_KEY_DIGEST: [u8; 32] = [0xC8; 32];
const SEALED_AUDIT_GATEWAY_KEY_TRANSPARENCY_CHECKPOINT_DIGEST: [u8; 32] = [0xC9; 32];
const SEALED_AUDIT_GATEWAY_KEY_CONSISTENCY_PROOF_DIGEST: [u8; 32] = [0xCA; 32];
const SEALED_AUDIT_GATEWAY_KEY_ROTATION_DIGEST: [u8; 32] = [0xCB; 32];
const SEALED_AUDIT_MONITOR_SUBMISSION_PROOF_DIGEST: [u8; 32] = [0xCC; 32];
const SEALED_AUDIT_BLINDED_FAILURE_CLASS_DIGEST: [u8; 32] = [0xCD; 32];
const SEALED_AUDIT_RETRY_COMPLETION_DIGEST: [u8; 32] = [0xCE; 32];
const SEALED_AUDIT_PRIVATE_REPORT_RECONCILIATION_ID: [u8; 32] = [0xA5; 32];
const SEALED_AUDIT_PENDING_OUTBOX_DIGEST: [u8; 32] = [0xA7; 32];
const SEALED_AUDIT_RETRY_SCHEDULE_DIGEST: [u8; 32] = [0xA8; 32];
const SEALED_AUDIT_RATE_LIMIT_STATE_DIGEST: [u8; 32] = [0xA9; 32];
const SEALED_AUDIT_DELIVERED_STATE_DIGEST: [u8; 32] = [0xAA; 32];
const SEALED_AUDIT_FAILURE_BUCKET_DIGEST: [u8; 32] = [0xAB; 32];
const SEALED_AUDIT_CRASH_RECOVERY_CURSOR_DIGEST: [u8; 32] = [0xAC; 32];
const SEALED_AUDIT_PRIVATE_REPORT_GATEWAY_EVIDENCE_ID: [u8; 32] = [0xAD; 32];
const SEALED_AUDIT_UNAVAILABLE_EVIDENCE_DIGEST: [u8; 32] = [0xAE; 32];
const SEALED_AUDIT_RELAY_OBSERVATION_DIGEST: [u8; 32] = [0xAF; 32];
const SEALED_AUDIT_GATEWAY_ERROR_DIGEST: [u8; 32] = [0xB0; 32];
const SEALED_AUDIT_TARGET_ABSENCE_DIGEST: [u8; 32] = [0xB1; 32];
const SEALED_AUDIT_RETRY_EXHAUSTION_DIGEST: [u8; 32] = [0xB2; 32];
const SEALED_AUDIT_GATEWAY_KEY_STATE_DIGEST: [u8; 32] = [0xB3; 32];

fn sealed_audit_event_store_ready_fixture() -> Value {
    sealed_audit_event_store_fixture(
        "sealed_audit_event_store_ready",
        valid_sealed_audit_event_store_write(),
        false,
    )
}

fn sealed_audit_event_store_chain_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_event_store_write();
    write.chain_decision = rejected_sealed_audit_chain_decision();
    sealed_audit_event_store_fixture("sealed_audit_event_store_chain_rejected", write, false)
}

fn sealed_audit_event_store_duplicate_rejected_fixture() -> Value {
    sealed_audit_event_store_fixture(
        "sealed_audit_event_store_duplicate_rejected",
        valid_sealed_audit_event_store_write(),
        true,
    )
}

fn sealed_audit_event_store_rollback_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_event_store_write();
    write.event_sequence = 41;
    write.chain_decision = accepted_sealed_audit_chain_decision_for_sequence(41);
    write.event_hash = &SEALED_AUDIT_OTHER_EVENT_HASH;
    write.record_digest = &SEALED_AUDIT_OTHER_RECORD_DIGEST;
    write.checkpoint_id = &SEALED_AUDIT_OTHER_CHECKPOINT_ID;
    sealed_audit_event_store_fixture("sealed_audit_event_store_rollback_rejected", write, true)
}

fn sealed_audit_event_store_plaintext_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_event_store_write();
    write.plaintext_metadata_fields = 1;
    sealed_audit_event_store_fixture("sealed_audit_event_store_plaintext_rejected", write, false)
}

fn sealed_audit_event_store_fixture(
    name: &'static str,
    write: SealedAuditEventStoreWrite<'static>,
    seed_duplicate: bool,
) -> Value {
    let mut store = PrototypeSealedAuditEventStore::default();
    if seed_duplicate {
        let _ = put_sealed_audit_event_record(&mut store, valid_sealed_audit_event_store_write())
            .expect("prototype sealed audit event store cannot fail");
    }
    let decision = put_sealed_audit_event_record(&mut store, write)
        .expect("prototype sealed audit event store cannot fail");

    json!({
        "fixture": name,
        "surface": "sealed_audit_event_store",
        "input": {
            "chain_decision_accepted": write.chain_decision.accepted,
            "chain_decision_reason_code": write.chain_decision.reason.code(),
            "chain_decision_reason_label": write.chain_decision.reason.label(),
            "event_sequence": write.event_sequence,
            "event_hash_len": write.event_hash.len(),
            "previous_event_hash_len": write.previous_event_hash.len(),
            "record_digest_len": write.record_digest.len(),
            "merkle_root_hash_len": write.merkle_root_hash.len(),
            "checkpoint_id_len": write.checkpoint_id.len(),
            "checkpoint_signature_len": write.checkpoint_signature.len(),
            "transparency_receipt_len": write.transparency_receipt.len(),
            "witness_receipt_len": write.witness_receipt.len(),
            "event_kind_code": write.event_kind.code(),
            "event_kind_label": write.event_kind.label(),
            "anchor_kind_code": write.anchor_kind.code(),
            "anchor_kind_label": write.anchor_kind.label(),
            "sealed_payload_len": write.sealed_payload_len,
            "plaintext_metadata_fields": write.plaintext_metadata_fields,
            "append_only_guard": write.append_only_guard,
            "checkpoint_binds_chain": write.checkpoint_binds_chain,
            "receipt_binds_checkpoint": write.receipt_binds_checkpoint,
            "seed_duplicate": seed_duplicate,
        },
        "decision": sealed_audit_event_store_decision_value(decision),
        "store": {
            "record_count": store.len(),
            "has_sequence": store.get_by_sequence(write.event_sequence).is_some(),
            "has_event_hash": store.get_by_hash(write.event_hash).is_some(),
            "checkpoint_recorded": store.checkpoint_recorded(write.checkpoint_id),
            "highest_event_sequence": store.highest_event_sequence(),
        },
    })
}

fn sealed_audit_event_store_decision_value(decision: SealedAuditEventStoreDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "event_sequence": decision.event_sequence,
        "can_publish_receipt": decision.can_publish_receipt,
        "can_detect_replay": decision.can_detect_replay,
        "append_only": decision.append_only,
        "keeps_digest_only": decision.keeps_digest_only,
        "keeps_plaintext_metadata": decision.keeps_plaintext_metadata,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_witness_checkpoint_ready_fixture() -> Value {
    sealed_audit_witness_checkpoint_fixture(
        "sealed_audit_witness_checkpoint_ready",
        valid_sealed_audit_witness_checkpoint_input(),
    )
}

fn sealed_audit_witness_checkpoint_store_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_witness_checkpoint_input();
    input.store_decision = rejected_sealed_audit_event_store_decision();
    sealed_audit_witness_checkpoint_fixture("sealed_audit_witness_checkpoint_store_rejected", input)
}

fn sealed_audit_witness_checkpoint_quorum_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_witness_checkpoint_input();
    input.witness_count = 1;
    sealed_audit_witness_checkpoint_fixture(
        "sealed_audit_witness_checkpoint_quorum_rejected",
        input,
    )
}

fn sealed_audit_witness_checkpoint_split_view_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_witness_checkpoint_input();
    input.split_view_evidence_present = true;
    sealed_audit_witness_checkpoint_fixture(
        "sealed_audit_witness_checkpoint_split_view_rejected",
        input,
    )
}

fn sealed_audit_witness_checkpoint_privacy_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_witness_checkpoint_input();
    input.monitor_query_uses_private_retrieval = false;
    input.monitor_query_plaintext_selectors = 1;
    input.monitor_receives_only_digests = false;
    sealed_audit_witness_checkpoint_fixture(
        "sealed_audit_witness_checkpoint_privacy_rejected",
        input,
    )
}

fn sealed_audit_witness_checkpoint_fixture(
    name: &'static str,
    input: SealedAuditWitnessCheckpointInput,
) -> Value {
    let decision = evaluate_sealed_audit_witness_checkpoint(input);
    let checkpoint = json!({
        "checkpoint_origin_len": input.checkpoint_origin_len,
        "log_id_digest_len": input.log_id_digest_len,
        "checkpoint_timestamp_s": input.checkpoint_timestamp_s,
        "checkpoint_size": input.checkpoint_size,
        "previous_checkpoint_size": input.previous_checkpoint_size,
        "checkpoint_root_hash_len": input.checkpoint_root_hash_len,
        "checkpoint_signature_len": input.checkpoint_signature_len,
        "signing_key_id_digest_len": input.signing_key_id_digest_len,
        "signing_key_not_expired": input.signing_key_not_expired,
        "signing_key_rotation_window_valid": input.signing_key_rotation_window_valid,
        "previous_signing_key_retained_for_verification": input.previous_signing_key_retained_for_verification,
        "consistency_proof_verified": input.consistency_proof_verified,
        "consistency_proof_hash_count": input.consistency_proof_hash_count,
    });
    let witness = json!({
        "witness_count": input.witness_count,
        "witness_threshold": input.witness_threshold,
        "witness_operator_count": input.witness_operator_count,
        "witness_key_pins_present": input.witness_key_pins_present,
        "witness_cosignature_bytes": input.witness_cosignature_bytes,
        "cosignatures_timestamped": input.cosignatures_timestamped,
        "cosignatures_bind_checkpoint": input.cosignatures_bind_checkpoint,
        "split_view_evidence_present": input.split_view_evidence_present,
    });
    let monitor = json!({
        "monitor_query_uses_private_retrieval": input.monitor_query_uses_private_retrieval,
        "monitor_query_plaintext_selectors": input.monitor_query_plaintext_selectors,
        "monitor_receives_only_digests": input.monitor_receives_only_digests,
    });
    let recovery = json!({
        "local_latest_checkpoint_available": input.local_latest_checkpoint_available,
        "recovery_checkpoint_authenticated": input.recovery_checkpoint_authenticated,
        "recovery_requires_user_verification": input.recovery_requires_user_verification,
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_witness_checkpoint",
        "input": {
            "store_decision": sealed_audit_event_store_decision_value(input.store_decision),
            "anchor_kind_code": input.anchor_kind.code(),
            "anchor_kind_label": input.anchor_kind.label(),
            "signature_algorithm_code": input.signature_algorithm.code(),
            "signature_algorithm_label": input.signature_algorithm.label(),
            "checkpoint": checkpoint,
            "witness": witness,
            "monitor": monitor,
            "recovery": recovery,
        },
        "decision": sealed_audit_witness_checkpoint_decision_value(decision),
    })
}

fn sealed_audit_witness_checkpoint_decision_value(
    decision: SealedAuditWitnessCheckpointDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "anchor_kind_code": decision.anchor_kind.code(),
        "anchor_kind_label": decision.anchor_kind.label(),
        "signature_algorithm_code": decision.signature_algorithm.code(),
        "signature_algorithm_label": decision.signature_algorithm.label(),
        "store_event_sequence": decision.store_event_sequence,
        "checkpoint_size": decision.checkpoint_size,
        "witness_threshold": decision.witness_threshold,
        "can_publish_checkpoint": decision.can_publish_checkpoint,
        "can_request_witness_cosignature": decision.can_request_witness_cosignature,
        "can_monitor_privately": decision.can_monitor_privately,
        "can_detect_split_view": decision.can_detect_split_view,
        "requires_witness_repair": decision.requires_witness_repair,
        "requires_key_rotation": decision.requires_key_rotation,
        "requires_user_warning": decision.requires_user_warning,
        "requires_local_recovery": decision.requires_local_recovery,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn valid_sealed_audit_witness_checkpoint_input() -> SealedAuditWitnessCheckpointInput {
    SealedAuditWitnessCheckpointInput {
        store_decision: accepted_sealed_audit_event_store_decision(),
        anchor_kind: SealedAuditAnchorKind::WitnessedTransparencyLog,
        signature_algorithm: SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44,
        checkpoint_origin_len: 32,
        log_id_digest_len: 32,
        checkpoint_timestamp_s: 1_769_990_400,
        checkpoint_size: 43,
        previous_checkpoint_size: 42,
        checkpoint_root_hash_len: 32,
        checkpoint_signature_len: 2484,
        signing_key_id_digest_len: 32,
        signing_key_not_expired: true,
        signing_key_rotation_window_valid: true,
        previous_signing_key_retained_for_verification: true,
        consistency_proof_verified: true,
        consistency_proof_hash_count: 6,
        witness_count: 3,
        witness_threshold: 2,
        witness_operator_count: 3,
        witness_key_pins_present: true,
        witness_cosignature_bytes: 5016,
        cosignatures_timestamped: true,
        cosignatures_bind_checkpoint: true,
        split_view_evidence_present: false,
        monitor_query_uses_private_retrieval: true,
        monitor_query_plaintext_selectors: 0,
        monitor_receives_only_digests: true,
        local_latest_checkpoint_available: true,
        recovery_checkpoint_authenticated: false,
        recovery_requires_user_verification: false,
    }
}

fn sealed_audit_witness_client_ready_fixture() -> Value {
    sealed_audit_witness_client_fixture(
        "sealed_audit_witness_client_ready",
        valid_sealed_audit_witness_client_input(),
    )
}

fn sealed_audit_witness_client_conflict_fixture() -> Value {
    let mut input = valid_sealed_audit_witness_client_input();
    input.response_status_code = 409;
    input.response_latest_size = 41;
    sealed_audit_witness_client_fixture("sealed_audit_witness_client_conflict", input)
}

fn sealed_audit_witness_client_unavailable_fixture() -> Value {
    let mut input = valid_sealed_audit_witness_client_input();
    input.response_status_code = 503;
    sealed_audit_witness_client_fixture("sealed_audit_witness_client_unavailable", input)
}

fn sealed_audit_witness_client_policy_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_witness_client_input();
    input.policy_not_expired = false;
    sealed_audit_witness_client_fixture("sealed_audit_witness_client_policy_rejected", input)
}

fn sealed_audit_witness_client_monitor_privacy_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_witness_client_input();
    input.monitor_query_uses_private_retrieval = false;
    input.monitor_query_uses_vrf_or_blinded_selector = false;
    input.monitor_query_plaintext_selectors = 1;
    input.monitor_receives_only_digests = false;
    sealed_audit_witness_client_fixture(
        "sealed_audit_witness_client_monitor_privacy_rejected",
        input,
    )
}

fn sealed_audit_witness_client_fixture(
    name: &'static str,
    input: SealedAuditWitnessClientInput,
) -> Value {
    let decision = evaluate_sealed_audit_witness_client(input);
    let policy = json!({
        "policy_digest_len": input.policy_digest_len,
        "policy_epoch": input.policy_epoch,
        "policy_not_expired": input.policy_not_expired,
        "policy_binds_log_origin": input.policy_binds_log_origin,
        "policy_binds_witness_operators": input.policy_binds_witness_operators,
        "log_public_key_pin_count": input.log_public_key_pin_count,
        "witness_key_pin_count": input.witness_key_pin_count,
        "witness_operator_count": input.witness_operator_count,
        "witness_quorum_threshold": input.witness_quorum_threshold,
    });
    let endpoints = json!({
        "submission_endpoint_count": input.submission_endpoint_count,
        "monitor_endpoint_count": input.monitor_endpoint_count,
        "endpoints_use_https_or_bastion": input.endpoints_use_https_or_bastion,
        "endpoint_tls_pins_present": input.endpoint_tls_pins_present,
    });
    let request = json!({
        "request_old_size": input.request_old_size,
        "request_checkpoint_size": input.request_checkpoint_size,
        "request_consistency_proof_hash_count": input.request_consistency_proof_hash_count,
        "request_body_binds_policy_epoch": input.request_body_binds_policy_epoch,
        "request_body_plaintext_selector_count": input.request_body_plaintext_selector_count,
    });
    let response = json!({
        "response_status_code": input.response_status_code,
        "response_latest_size": input.response_latest_size,
        "response_cosignature_count": input.response_cosignature_count,
        "response_known_cosignature_count": input.response_known_cosignature_count,
        "response_operator_count": input.response_operator_count,
        "response_cosignatures_timestamped": input.response_cosignatures_timestamped,
        "response_cosignatures_bind_checkpoint": input.response_cosignatures_bind_checkpoint,
        "persist_latest_checkpoint_atomically": input.persist_latest_checkpoint_atomically,
        "split_view_alert_delivery_configured": input.split_view_alert_delivery_configured,
    });
    let monitor = json!({
        "monitor_query_uses_private_retrieval": input.monitor_query_uses_private_retrieval,
        "monitor_query_uses_vrf_or_blinded_selector": input.monitor_query_uses_vrf_or_blinded_selector,
        "monitor_query_plaintext_selectors": input.monitor_query_plaintext_selectors,
        "monitor_receives_only_digests": input.monitor_receives_only_digests,
    });
    let recovery = json!({
        "recovery_checkpoint_authenticated": input.recovery_checkpoint_authenticated,
        "recovery_requires_user_verification": input.recovery_requires_user_verification,
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_witness_client",
        "input": {
            "checkpoint_decision": sealed_audit_witness_checkpoint_decision_value(
                input.checkpoint_decision,
            ),
            "policy": policy,
            "endpoints": endpoints,
            "request": request,
            "response": response,
            "monitor": monitor,
            "recovery": recovery,
        },
        "decision": sealed_audit_witness_client_decision_value(decision),
    })
}

fn sealed_audit_witness_client_decision_value(decision: SealedAuditWitnessClientDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "checkpoint_size": decision.checkpoint_size,
        "policy_epoch": decision.policy_epoch,
        "witness_quorum_threshold": decision.witness_quorum_threshold,
        "response_status_code": decision.response_status_code,
        "can_submit_add_checkpoint": decision.can_submit_add_checkpoint,
        "can_publish_witnessed_checkpoint": decision.can_publish_witnessed_checkpoint,
        "can_monitor_privately": decision.can_monitor_privately,
        "can_retry_witness_conflict": decision.can_retry_witness_conflict,
        "can_alert_split_view": decision.can_alert_split_view,
        "requires_policy_rotation": decision.requires_policy_rotation,
        "requires_witness_repair": decision.requires_witness_repair,
        "requires_operator_alert": decision.requires_operator_alert,
        "requires_local_recovery": decision.requires_local_recovery,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn valid_sealed_audit_witness_client_input() -> SealedAuditWitnessClientInput {
    SealedAuditWitnessClientInput {
        checkpoint_decision: evaluate_sealed_audit_witness_checkpoint(
            valid_sealed_audit_witness_checkpoint_input(),
        ),
        policy_digest_len: 32,
        policy_epoch: 7,
        policy_not_expired: true,
        policy_binds_log_origin: true,
        policy_binds_witness_operators: true,
        log_public_key_pin_count: 1,
        witness_key_pin_count: 3,
        witness_operator_count: 3,
        witness_quorum_threshold: 2,
        submission_endpoint_count: 3,
        monitor_endpoint_count: 2,
        endpoints_use_https_or_bastion: true,
        endpoint_tls_pins_present: true,
        request_old_size: 42,
        request_checkpoint_size: 43,
        request_consistency_proof_hash_count: 6,
        request_body_binds_policy_epoch: true,
        request_body_plaintext_selector_count: 0,
        response_status_code: 200,
        response_latest_size: 43,
        response_cosignature_count: 3,
        response_known_cosignature_count: 3,
        response_operator_count: 3,
        response_cosignatures_timestamped: true,
        response_cosignatures_bind_checkpoint: true,
        persist_latest_checkpoint_atomically: true,
        split_view_alert_delivery_configured: true,
        monitor_query_uses_private_retrieval: true,
        monitor_query_uses_vrf_or_blinded_selector: true,
        monitor_query_plaintext_selectors: 0,
        monitor_receives_only_digests: true,
        recovery_checkpoint_authenticated: false,
        recovery_requires_user_verification: false,
    }
}

fn sealed_audit_proof_bundle_ready_fixture() -> Value {
    sealed_audit_proof_bundle_fixture(
        "sealed_audit_proof_bundle_ready",
        valid_sealed_audit_proof_bundle_input(),
    )
}

fn sealed_audit_proof_bundle_client_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_proof_bundle_input();
    input.witness_client_decision = rejected_sealed_audit_witness_client_decision(false);
    sealed_audit_proof_bundle_fixture("sealed_audit_proof_bundle_client_rejected", input)
}

fn sealed_audit_proof_bundle_stale_witness_fixture() -> Value {
    let mut input = valid_sealed_audit_proof_bundle_input();
    input.witness_timestamp_s = input.verification_time_s - input.max_witness_age_s - 1;
    sealed_audit_proof_bundle_fixture("sealed_audit_proof_bundle_stale_witness", input)
}

fn sealed_audit_proof_bundle_policy_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_proof_bundle_input();
    input.verifier_policy_epoch = 6;
    sealed_audit_proof_bundle_fixture("sealed_audit_proof_bundle_policy_rejected", input)
}

fn sealed_audit_proof_bundle_privacy_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_proof_bundle_input();
    input.plaintext_selector_count = 1;
    input.ui_status_digest_only = false;
    sealed_audit_proof_bundle_fixture("sealed_audit_proof_bundle_privacy_rejected", input)
}

fn sealed_audit_proof_bundle_fixture(
    name: &'static str,
    input: SealedAuditProofBundleInput,
) -> Value {
    let decision = evaluate_sealed_audit_proof_bundle(input);
    let proof = json!({
        "bundle_format_version": input.bundle_format_version,
        "proof_bundle_persisted": input.proof_bundle_persisted,
        "event_sequence": input.event_sequence,
        "event_hash_len": input.event_hash_len,
        "leaf_hash_len": input.leaf_hash_len,
        "log_index": input.log_index,
        "checkpoint_size": input.checkpoint_size,
        "inclusion_proof_hash_count": input.inclusion_proof_hash_count,
        "inclusion_proof_verified": input.inclusion_proof_verified,
        "inclusion_root_matches_checkpoint": input.inclusion_root_matches_checkpoint,
        "consistency_proof_hash_count": input.consistency_proof_hash_count,
        "consistency_proof_verified": input.consistency_proof_verified,
        "extra_data_authenticated_or_opaque": input.extra_data_authenticated_or_opaque,
    });
    let cache = json!({
        "proof_cache_digest_len": input.proof_cache_digest_len,
        "proof_cache_encrypted": input.proof_cache_encrypted,
        "proof_cache_append_only": input.proof_cache_append_only,
        "local_proof_cache_available": input.local_proof_cache_available,
        "proof_cache_recovery_authenticated": input.proof_cache_recovery_authenticated,
        "proof_cache_recovery_user_verified": input.proof_cache_recovery_user_verified,
    });
    let policy = json!({
        "verifier_policy_snapshot_digest_len": input.verifier_policy_snapshot_digest_len,
        "verifier_policy_epoch": input.verifier_policy_epoch,
        "verifier_policy_matches_witness_policy": input.verifier_policy_matches_witness_policy,
        "verifier_log_key_pin_count": input.verifier_log_key_pin_count,
        "verifier_witness_key_pin_count": input.verifier_witness_key_pin_count,
        "verifier_witness_threshold": input.verifier_witness_threshold,
        "verified_witness_cosignature_count": input.verified_witness_cosignature_count,
    });
    let freshness = json!({
        "witness_timestamp_s": input.witness_timestamp_s,
        "verification_time_s": input.verification_time_s,
        "max_witness_age_s": input.max_witness_age_s,
        "monitor_freshness_checked": input.monitor_freshness_checked,
    });
    let privacy = json!({
        "audit_subject_digest_len": input.audit_subject_digest_len,
        "plaintext_selector_count": input.plaintext_selector_count,
        "ui_status_digest_only": input.ui_status_digest_only,
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_proof_bundle",
        "input": {
            "witness_client_decision": sealed_audit_witness_client_decision_value(
                input.witness_client_decision,
            ),
            "proof": proof,
            "cache": cache,
            "policy": policy,
            "freshness": freshness,
            "privacy": privacy,
        },
        "decision": sealed_audit_proof_bundle_decision_value(decision),
    })
}

fn sealed_audit_proof_bundle_decision_value(decision: SealedAuditProofBundleDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "event_sequence": decision.event_sequence,
        "log_index": decision.log_index,
        "checkpoint_size": decision.checkpoint_size,
        "verifier_policy_epoch": decision.verifier_policy_epoch,
        "can_verify_offline": decision.can_verify_offline,
        "can_persist_proof_bundle": decision.can_persist_proof_bundle,
        "can_show_ui_status": decision.can_show_ui_status,
        "can_recover_proof_cache": decision.can_recover_proof_cache,
        "requires_policy_refresh": decision.requires_policy_refresh,
        "requires_witness_refresh": decision.requires_witness_refresh,
        "requires_proof_cache_recovery": decision.requires_proof_cache_recovery,
        "requires_redaction": decision.requires_redaction,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn valid_sealed_audit_proof_bundle_input() -> SealedAuditProofBundleInput {
    SealedAuditProofBundleInput {
        witness_client_decision: evaluate_sealed_audit_witness_client(
            valid_sealed_audit_witness_client_input(),
        ),
        bundle_format_version: 1,
        proof_bundle_persisted: true,
        proof_cache_digest_len: 32,
        proof_cache_encrypted: true,
        proof_cache_append_only: true,
        local_proof_cache_available: true,
        proof_cache_recovery_authenticated: false,
        proof_cache_recovery_user_verified: false,
        verifier_policy_snapshot_digest_len: 32,
        verifier_policy_epoch: 7,
        verifier_policy_matches_witness_policy: true,
        verifier_log_key_pin_count: 1,
        verifier_witness_key_pin_count: 3,
        verifier_witness_threshold: 2,
        verified_witness_cosignature_count: 3,
        event_sequence: 42,
        event_hash_len: 32,
        leaf_hash_len: 32,
        log_index: 42,
        checkpoint_size: 43,
        inclusion_proof_hash_count: 6,
        inclusion_proof_verified: true,
        inclusion_root_matches_checkpoint: true,
        consistency_proof_hash_count: 6,
        consistency_proof_verified: true,
        witness_timestamp_s: 1_769_990_400,
        verification_time_s: 1_769_990_430,
        max_witness_age_s: 900,
        monitor_freshness_checked: true,
        extra_data_authenticated_or_opaque: true,
        audit_subject_digest_len: 32,
        plaintext_selector_count: 0,
        ui_status_digest_only: true,
    }
}

const fn rejected_sealed_audit_witness_client_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditWitnessClientDecision {
    SealedAuditWitnessClientDecision {
        accepted: false,
        reason: mercury_core::SealedAuditWitnessClientReason::WitnessUnavailable,
        checkpoint_size: 43,
        policy_epoch: 7,
        witness_quorum_threshold: 2,
        response_status_code: 503,
        can_submit_add_checkpoint: false,
        can_publish_witnessed_checkpoint: false,
        can_monitor_privately: false,
        can_retry_witness_conflict: false,
        can_alert_split_view: true,
        requires_policy_rotation: false,
        requires_witness_repair: true,
        requires_operator_alert: false,
        requires_local_recovery: false,
        plaintext_bytes_exposed,
    }
}

fn sealed_audit_proof_cache_ready_fixture() -> Value {
    sealed_audit_proof_cache_fixture(
        "sealed_audit_proof_cache_ready",
        valid_sealed_audit_proof_cache_write(),
        false,
    )
}

fn sealed_audit_proof_cache_bundle_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_proof_cache_write();
    write.proof_bundle_decision = rejected_sealed_audit_proof_bundle_decision(false);
    sealed_audit_proof_cache_fixture("sealed_audit_proof_cache_bundle_rejected", write, false)
}

fn sealed_audit_proof_cache_duplicate_rejected_fixture() -> Value {
    sealed_audit_proof_cache_fixture(
        "sealed_audit_proof_cache_duplicate_rejected",
        valid_sealed_audit_proof_cache_write(),
        true,
    )
}

fn sealed_audit_proof_cache_policy_stale_fixture() -> Value {
    let mut write = valid_sealed_audit_proof_cache_write();
    write.verifier_policy_epoch = 6;
    sealed_audit_proof_cache_fixture("sealed_audit_proof_cache_policy_stale", write, false)
}

fn sealed_audit_proof_cache_plaintext_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_proof_cache_write();
    write.plaintext_metadata_fields = 1;
    sealed_audit_proof_cache_fixture("sealed_audit_proof_cache_plaintext_rejected", write, false)
}

fn sealed_audit_proof_cache_fixture(
    name: &'static str,
    write: SealedAuditProofCacheWrite<'_>,
    prepopulate_duplicate: bool,
) -> Value {
    let mut cache = PrototypeSealedAuditProofCache::default();
    if prepopulate_duplicate {
        let prepopulate = cache.put(valid_sealed_audit_proof_cache_write());
        debug_assert!(prepopulate.accepted);
    }

    let decision = put_sealed_audit_proof_cache_record(&mut cache, write)
        .expect("prototype sealed audit proof cache cannot fail");
    let record = cache
        .get_by_digest(write.proof_bundle_digest)
        .map(sealed_audit_proof_cache_record_value);

    let cache_state = json!({
        "record_count": cache.len(),
        "has_record": record.is_some(),
        "has_event_hash": cache.get_by_event_hash(write.event_hash).is_some(),
        "highest_log_index": cache.highest_log_index(),
    });
    let input = json!({
        "proof_bundle_decision": sealed_audit_proof_bundle_decision_value(
            write.proof_bundle_decision,
        ),
        "cache_format_version": write.cache_format_version,
        "proof_bundle_digest_len": write.proof_bundle_digest.len(),
        "event_hash_len": write.event_hash.len(),
        "checkpoint_digest_len": write.checkpoint_digest.len(),
        "verifier_policy_snapshot_digest_len": write.verifier_policy_snapshot_digest.len(),
        "event_sequence": write.event_sequence,
        "log_index": write.log_index,
        "checkpoint_size": write.checkpoint_size,
        "verifier_policy_epoch": write.verifier_policy_epoch,
        "verified_at_s": write.verified_at_s,
        "witness_timestamp_s": write.witness_timestamp_s,
        "offline_verification_passed": write.offline_verification_passed,
        "monitor_freshness_checked": write.monitor_freshness_checked,
        "cache_record_encrypted": write.cache_record_encrypted,
        "append_only_guard": write.append_only_guard,
        "plaintext_metadata_fields": write.plaintext_metadata_fields,
        "recovery_bundle_authenticated": write.recovery_bundle_authenticated,
        "recovery_requires_user_verification": write.recovery_requires_user_verification,
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_proof_cache",
        "input": input,
        "decision": sealed_audit_proof_cache_decision_value(decision),
        "cache": cache_state,
        "record": record,
    })
}

fn sealed_audit_proof_cache_decision_value(decision: SealedAuditProofCacheDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "event_sequence": decision.event_sequence,
        "log_index": decision.log_index,
        "checkpoint_size": decision.checkpoint_size,
        "verifier_policy_epoch": decision.verifier_policy_epoch,
        "can_verify_offline": decision.can_verify_offline,
        "can_show_ui_status": decision.can_show_ui_status,
        "can_refresh_monitor": decision.can_refresh_monitor,
        "requires_policy_refresh": decision.requires_policy_refresh,
        "requires_witness_refresh": decision.requires_witness_refresh,
        "requires_cache_recovery": decision.requires_cache_recovery,
        "append_only": decision.append_only,
        "keeps_digest_only": decision.keeps_digest_only,
        "keeps_plaintext_metadata": decision.keeps_plaintext_metadata,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_proof_cache_record_value(
    record: &mercury_core::SealedAuditProofCacheRecord,
) -> Value {
    json!({
        "proof_bundle_digest_len": record.proof_bundle_digest.len(),
        "event_hash_len": record.event_hash.len(),
        "checkpoint_digest_len": record.checkpoint_digest.len(),
        "verifier_policy_snapshot_digest_len": record.verifier_policy_snapshot_digest.len(),
        "event_sequence": record.event_sequence,
        "log_index": record.log_index,
        "checkpoint_size": record.checkpoint_size,
        "verifier_policy_epoch": record.verifier_policy_epoch,
        "verified_at_s": record.verified_at_s,
        "witness_timestamp_s": record.witness_timestamp_s,
        "recovered_from_cache_loss": record.recovered_from_cache_loss,
        "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
    })
}

fn valid_sealed_audit_proof_cache_write() -> SealedAuditProofCacheWrite<'static> {
    SealedAuditProofCacheWrite {
        proof_bundle_decision: evaluate_sealed_audit_proof_bundle(
            valid_sealed_audit_proof_bundle_input(),
        ),
        cache_format_version: 1,
        proof_bundle_digest: &SEALED_AUDIT_PROOF_BUNDLE_DIGEST,
        event_hash: &SEALED_AUDIT_EVENT_HASH,
        checkpoint_digest: &SEALED_AUDIT_PROOF_CHECKPOINT_DIGEST,
        verifier_policy_snapshot_digest: &SEALED_AUDIT_POLICY_SNAPSHOT_DIGEST,
        event_sequence: 42,
        log_index: 42,
        checkpoint_size: 43,
        verifier_policy_epoch: 7,
        verified_at_s: 1_769_990_430,
        witness_timestamp_s: 1_769_990_400,
        offline_verification_passed: true,
        monitor_freshness_checked: true,
        cache_record_encrypted: true,
        append_only_guard: true,
        plaintext_metadata_fields: 0,
        recovery_bundle_authenticated: false,
        recovery_requires_user_verification: false,
    }
}

const fn rejected_sealed_audit_proof_bundle_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditProofBundleDecision {
    SealedAuditProofBundleDecision {
        accepted: false,
        reason: mercury_core::SealedAuditProofBundleReason::WitnessClientRejected,
        event_sequence: 42,
        log_index: 42,
        checkpoint_size: 43,
        verifier_policy_epoch: 7,
        can_verify_offline: false,
        can_persist_proof_bundle: false,
        can_show_ui_status: false,
        can_recover_proof_cache: false,
        requires_policy_refresh: false,
        requires_witness_refresh: false,
        requires_proof_cache_recovery: false,
        requires_redaction: false,
        plaintext_bytes_exposed,
    }
}

fn sealed_audit_verifier_policy_ready_fixture() -> Value {
    sealed_audit_verifier_policy_fixture(
        "sealed_audit_verifier_policy_ready",
        valid_sealed_audit_verifier_policy_snapshot(),
    )
}

fn sealed_audit_verifier_policy_expired_fixture() -> Value {
    let mut snapshot = valid_sealed_audit_verifier_policy_snapshot();
    snapshot.verification_time_s = snapshot.expires_at_s;
    sealed_audit_verifier_policy_fixture("sealed_audit_verifier_policy_expired", snapshot)
}

fn sealed_audit_verifier_policy_key_rotation_required_fixture() -> Value {
    let mut snapshot = valid_sealed_audit_verifier_policy_snapshot();
    snapshot.policy_epoch = 8;
    snapshot.policy_snapshot_digest = &SEALED_AUDIT_NEXT_VERIFIER_POLICY_DIGEST;
    snapshot.previous_policy_snapshot_digest = &SEALED_AUDIT_VERIFIER_POLICY_DIGEST;
    snapshot.key_rotation_required = true;
    snapshot.key_rotation_authenticated = false;
    sealed_audit_verifier_policy_fixture(
        "sealed_audit_verifier_policy_key_rotation_required",
        snapshot,
    )
}

fn sealed_audit_verifier_policy_monitor_privacy_rejected_fixture() -> Value {
    let mut snapshot = valid_sealed_audit_verifier_policy_snapshot();
    snapshot.monitor_last_refresh_s = 1_769_980_000;
    sealed_audit_verifier_policy_fixture(
        "sealed_audit_verifier_policy_monitor_privacy_rejected",
        snapshot,
    )
}

fn sealed_audit_verifier_policy_plaintext_rejected_fixture() -> Value {
    let mut snapshot = valid_sealed_audit_verifier_policy_snapshot();
    snapshot.monitor_query_plaintext_selector_count = 1;
    snapshot.plaintext_metadata_fields = 1;
    sealed_audit_verifier_policy_fixture(
        "sealed_audit_verifier_policy_plaintext_rejected",
        snapshot,
    )
}

fn sealed_audit_verifier_policy_fixture(
    name: &'static str,
    snapshot: SealedAuditVerifierPolicySnapshot<'_>,
) -> Value {
    let mut store = PrototypeSealedAuditVerifierPolicyStore::default();
    let decision = put_sealed_audit_verifier_policy_snapshot(&mut store, snapshot)
        .expect("prototype sealed audit verifier policy store cannot fail");
    let record = store
        .get_by_digest(snapshot.policy_snapshot_digest)
        .map(sealed_audit_verifier_policy_record_value);

    let store_state = json!({
        "snapshot_count": store.len(),
        "has_snapshot": record.is_some(),
        "latest_policy_epoch": store.latest().map(|record| record.policy_epoch),
    });
    let input = json!({
        "proof_cache_decision": sealed_audit_proof_cache_decision_value(
            snapshot.proof_cache_decision,
        ),
        "policy_format_version": snapshot.policy_format_version,
        "policy_snapshot_digest_len": snapshot.policy_snapshot_digest.len(),
        "previous_policy_snapshot_digest_len": snapshot.previous_policy_snapshot_digest.len(),
        "log_key_pinset_digest_len": snapshot.log_key_pinset_digest.len(),
        "witness_key_pinset_digest_len": snapshot.witness_key_pinset_digest.len(),
        "monitor_query_plan_digest_len": snapshot.monitor_query_plan_digest.len(),
        "proof_cache_digest_len": snapshot.proof_cache_digest.len(),
        "policy_epoch": snapshot.policy_epoch,
        "imported_at_s": snapshot.imported_at_s,
        "verification_time_s": snapshot.verification_time_s,
        "expires_at_s": snapshot.expires_at_s,
        "proof_cache_log_index": snapshot.proof_cache_log_index,
        "latest_checked_log_index": snapshot.latest_checked_log_index,
        "log_key_pin_count": snapshot.log_key_pin_count,
        "witness_key_pin_count": snapshot.witness_key_pin_count,
        "witness_quorum_threshold": snapshot.witness_quorum_threshold,
        "private_monitor_endpoint_count": snapshot.private_monitor_endpoint_count,
        "monitor_last_refresh_s": snapshot.monitor_last_refresh_s,
        "monitor_freshness_max_age_s": snapshot.monitor_freshness_max_age_s,
        "monitor_query_plaintext_selector_count":
            snapshot.monitor_query_plaintext_selector_count,
        "policy_signature_verified": snapshot.policy_signature_verified,
        "policy_consistency_proof_verified": snapshot.policy_consistency_proof_verified,
        "offline_reverification_passed": snapshot.offline_reverification_passed,
        "key_rotation_required": snapshot.key_rotation_required,
        "key_rotation_authenticated": snapshot.key_rotation_authenticated,
        "split_view_evidence_count": snapshot.split_view_evidence_count,
        "scheduler_state_encrypted": snapshot.scheduler_state_encrypted,
        "scheduler_append_only": snapshot.scheduler_append_only,
        "plaintext_metadata_fields": snapshot.plaintext_metadata_fields,
        "ui_status_digest_only": snapshot.ui_status_digest_only,
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_verifier_policy",
        "input": input,
        "decision": sealed_audit_verifier_policy_decision_value(decision),
        "store": store_state,
        "record": record,
    })
}

fn sealed_audit_verifier_policy_decision_value(
    decision: SealedAuditVerifierPolicyDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_snapshot": decision.persisted_snapshot,
        "snapshot_count": decision.snapshot_count,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
        "can_verify_offline": decision.can_verify_offline,
        "can_schedule_private_monitor": decision.can_schedule_private_monitor,
        "can_show_ui_status": decision.can_show_ui_status,
        "requires_policy_refresh": decision.requires_policy_refresh,
        "requires_monitor_refresh": decision.requires_monitor_refresh,
        "requires_key_rotation": decision.requires_key_rotation,
        "escalates_split_view": decision.escalates_split_view,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_verifier_policy_record_value(
    record: &mercury_core::SealedAuditVerifierPolicyRecord,
) -> Value {
    json!({
        "policy_snapshot_digest_len": record.policy_snapshot_digest.len(),
        "previous_policy_snapshot_digest_len": record.previous_policy_snapshot_digest.len(),
        "log_key_pinset_digest_len": record.log_key_pinset_digest.len(),
        "witness_key_pinset_digest_len": record.witness_key_pinset_digest.len(),
        "monitor_query_plan_digest_len": record.monitor_query_plan_digest.len(),
        "proof_cache_digest_len": record.proof_cache_digest.len(),
        "policy_epoch": record.policy_epoch,
        "imported_at_s": record.imported_at_s,
        "verification_time_s": record.verification_time_s,
        "expires_at_s": record.expires_at_s,
        "proof_cache_log_index": record.proof_cache_log_index,
        "latest_checked_log_index": record.latest_checked_log_index,
        "monitor_last_refresh_s": record.monitor_last_refresh_s,
        "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
    })
}

fn valid_sealed_audit_verifier_policy_snapshot() -> SealedAuditVerifierPolicySnapshot<'static> {
    SealedAuditVerifierPolicySnapshot {
        proof_cache_decision: accepted_sealed_audit_proof_cache_decision(),
        policy_format_version: 1,
        policy_snapshot_digest: &SEALED_AUDIT_VERIFIER_POLICY_DIGEST,
        previous_policy_snapshot_digest: &[],
        log_key_pinset_digest: &SEALED_AUDIT_LOG_KEY_PINSET_DIGEST,
        witness_key_pinset_digest: &SEALED_AUDIT_WITNESS_KEY_PINSET_DIGEST,
        monitor_query_plan_digest: &SEALED_AUDIT_MONITOR_QUERY_PLAN_DIGEST,
        proof_cache_digest: &SEALED_AUDIT_PROOF_CACHE_DIGEST,
        policy_epoch: 7,
        imported_at_s: 1_769_990_000,
        verification_time_s: 1_769_991_000,
        expires_at_s: 1_769_994_000,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        log_key_pin_count: 1,
        witness_key_pin_count: 3,
        witness_quorum_threshold: 2,
        private_monitor_endpoint_count: 2,
        monitor_last_refresh_s: 1_769_990_700,
        monitor_freshness_max_age_s: 3_600,
        monitor_query_plaintext_selector_count: 0,
        policy_signature_verified: true,
        policy_consistency_proof_verified: true,
        offline_reverification_passed: true,
        key_rotation_required: false,
        key_rotation_authenticated: false,
        split_view_evidence_count: 0,
        scheduler_state_encrypted: true,
        scheduler_append_only: true,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn accepted_sealed_audit_proof_cache_decision() -> SealedAuditProofCacheDecision {
    let mut cache = PrototypeSealedAuditProofCache::default();
    put_sealed_audit_proof_cache_record(&mut cache, valid_sealed_audit_proof_cache_write())
        .expect("prototype sealed audit proof cache cannot fail")
}

fn sealed_audit_incident_evidence_ready_fixture() -> Value {
    sealed_audit_incident_evidence_fixture(
        "sealed_audit_incident_evidence_ready",
        valid_sealed_audit_incident_evidence_write(),
    )
}

fn sealed_audit_incident_evidence_policy_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_incident_evidence_write();
    write.verifier_policy_decision = rejected_sealed_audit_verifier_policy_decision(false);
    sealed_audit_incident_evidence_fixture("sealed_audit_incident_evidence_policy_rejected", write)
}

fn sealed_audit_incident_evidence_missing_proof_report_fixture() -> Value {
    let mut write = valid_sealed_audit_incident_evidence_write();
    write.missing_proof_report_blinded = false;
    sealed_audit_incident_evidence_fixture(
        "sealed_audit_incident_evidence_missing_proof_report",
        write,
    )
}

fn sealed_audit_incident_evidence_split_view_fixture() -> Value {
    let mut write = valid_sealed_audit_incident_evidence_write();
    write.contradiction_proof_verified = false;
    sealed_audit_incident_evidence_fixture("sealed_audit_incident_evidence_split_view", write)
}

fn sealed_audit_incident_evidence_plaintext_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_incident_evidence_write();
    write.plaintext_selector_count = 1;
    write.plaintext_metadata_fields = 1;
    sealed_audit_incident_evidence_fixture(
        "sealed_audit_incident_evidence_plaintext_rejected",
        write,
    )
}

fn sealed_audit_incident_evidence_fixture(
    name: &'static str,
    write: SealedAuditIncidentEvidenceWrite<'_>,
) -> Value {
    let mut store = PrototypeSealedAuditIncidentEvidenceStore::default();
    let decision = put_sealed_audit_incident_evidence_record(&mut store, write)
        .expect("prototype sealed audit incident evidence store cannot fail");
    let record = store
        .get_by_incident_id(write.incident_id)
        .map(sealed_audit_incident_evidence_record_value);

    let store_state = json!({
        "record_count": store.len(),
        "has_record": record.is_some(),
    });
    let input = json!({
        "verifier_policy_decision": sealed_audit_verifier_policy_decision_value(
            write.verifier_policy_decision,
        ),
        "incident_format_version": write.incident_format_version,
        "incident_id_len": write.incident_id.len(),
        "verifier_policy_digest_len": write.verifier_policy_digest.len(),
        "proof_cache_digest_len": write.proof_cache_digest.len(),
        "checkpoint_digest_len": write.checkpoint_digest.len(),
        "witness_operator_digest_len": write.witness_operator_digest.len(),
        "contradiction_digest_len": write.contradiction_digest.len(),
        "missing_proof_report_digest_len": write.missing_proof_report_digest.len(),
        "monitor_report_digest_len": write.monitor_report_digest.len(),
        "accountability_route_digest_len": write.accountability_route_digest.len(),
        "policy_epoch": write.policy_epoch,
        "proof_cache_log_index": write.proof_cache_log_index,
        "latest_checked_log_index": write.latest_checked_log_index,
        "reported_at_s": write.reported_at_s,
        "evidence_observed_at_s": write.evidence_observed_at_s,
        "split_view_evidence_count": write.split_view_evidence_count,
        "missing_proof_count": write.missing_proof_count,
        "monitor_failure_count": write.monitor_failure_count,
        "operator_signature_count": write.operator_signature_count,
        "witness_quorum_threshold": write.witness_quorum_threshold,
        "incident_signature_verified": write.incident_signature_verified,
        "contradiction_proof_verified": write.contradiction_proof_verified,
        "missing_proof_report_blinded": write.missing_proof_report_blinded,
        "monitor_report_private": write.monitor_report_private,
        "accountability_route_configured": write.accountability_route_configured,
        "escalation_ack_required": write.escalation_ack_required,
        "retry_after_s": write.retry_after_s,
        "suppression_authenticated": write.suppression_authenticated,
        "store_record_encrypted": write.store_record_encrypted,
        "append_only_guard": write.append_only_guard,
        "plaintext_selector_count": write.plaintext_selector_count,
        "plaintext_metadata_fields": write.plaintext_metadata_fields,
        "ui_status_digest_only": write.ui_status_digest_only,
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_incident_evidence",
        "input": input,
        "decision": sealed_audit_incident_evidence_decision_value(decision),
        "store": store_state,
        "record": record,
    })
}

fn sealed_audit_incident_evidence_decision_value(
    decision: SealedAuditIncidentEvidenceDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
        "can_escalate_incident": decision.can_escalate_incident,
        "can_report_privately": decision.can_report_privately,
        "can_show_ui_status": decision.can_show_ui_status,
        "requires_missing_proof_report": decision.requires_missing_proof_report,
        "requires_split_view_escalation": decision.requires_split_view_escalation,
        "requires_operator_accountability": decision.requires_operator_accountability,
        "requires_retry_backoff": decision.requires_retry_backoff,
        "suppressed_by_authenticated_policy": decision.suppressed_by_authenticated_policy,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_incident_evidence_record_value(
    record: &mercury_core::SealedAuditIncidentEvidenceRecord,
) -> Value {
    json!({
        "incident_id_len": record.incident_id.len(),
        "verifier_policy_digest_len": record.verifier_policy_digest.len(),
        "proof_cache_digest_len": record.proof_cache_digest.len(),
        "checkpoint_digest_len": record.checkpoint_digest.len(),
        "witness_operator_digest_len": record.witness_operator_digest.len(),
        "contradiction_digest_len": record.contradiction_digest.len(),
        "missing_proof_report_digest_len": record.missing_proof_report_digest.len(),
        "monitor_report_digest_len": record.monitor_report_digest.len(),
        "accountability_route_digest_len": record.accountability_route_digest.len(),
        "policy_epoch": record.policy_epoch,
        "proof_cache_log_index": record.proof_cache_log_index,
        "latest_checked_log_index": record.latest_checked_log_index,
        "reported_at_s": record.reported_at_s,
        "evidence_observed_at_s": record.evidence_observed_at_s,
        "split_view_evidence_count": record.split_view_evidence_count,
        "missing_proof_count": record.missing_proof_count,
        "monitor_failure_count": record.monitor_failure_count,
        "operator_signature_count": record.operator_signature_count,
        "witness_quorum_threshold": record.witness_quorum_threshold,
        "can_escalate_incident": record.can_escalate_incident,
        "can_report_privately": record.can_report_privately,
        "suppressed_by_authenticated_policy": record.suppressed_by_authenticated_policy,
        "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
    })
}

fn valid_sealed_audit_incident_evidence_write() -> SealedAuditIncidentEvidenceWrite<'static> {
    SealedAuditIncidentEvidenceWrite {
        verifier_policy_decision: accepted_sealed_audit_verifier_policy_decision(),
        incident_format_version: 1,
        incident_id: &SEALED_AUDIT_INCIDENT_ID,
        verifier_policy_digest: &SEALED_AUDIT_VERIFIER_POLICY_DIGEST,
        proof_cache_digest: &SEALED_AUDIT_PROOF_CACHE_DIGEST,
        checkpoint_digest: &SEALED_AUDIT_PROOF_CHECKPOINT_DIGEST,
        witness_operator_digest: &SEALED_AUDIT_WITNESS_KEY_PINSET_DIGEST,
        contradiction_digest: &SEALED_AUDIT_CONTRADICTION_DIGEST,
        missing_proof_report_digest: &SEALED_AUDIT_MISSING_PROOF_REPORT_DIGEST,
        monitor_report_digest: &SEALED_AUDIT_MONITOR_REPORT_DIGEST,
        accountability_route_digest: &SEALED_AUDIT_ACCOUNTABILITY_ROUTE_DIGEST,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        reported_at_s: 1_769_991_000,
        evidence_observed_at_s: 1_769_990_900,
        split_view_evidence_count: 1,
        missing_proof_count: 1,
        monitor_failure_count: 1,
        operator_signature_count: 2,
        witness_quorum_threshold: 2,
        incident_signature_verified: true,
        contradiction_proof_verified: true,
        missing_proof_report_blinded: true,
        monitor_report_private: true,
        accountability_route_configured: true,
        escalation_ack_required: true,
        retry_after_s: 300,
        suppression_authenticated: false,
        store_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn accepted_sealed_audit_verifier_policy_decision() -> SealedAuditVerifierPolicyDecision {
    let mut store = PrototypeSealedAuditVerifierPolicyStore::default();
    put_sealed_audit_verifier_policy_snapshot(
        &mut store,
        valid_sealed_audit_verifier_policy_snapshot(),
    )
    .expect("prototype sealed audit verifier policy store cannot fail")
}

const fn rejected_sealed_audit_verifier_policy_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditVerifierPolicyDecision {
    SealedAuditVerifierPolicyDecision {
        accepted: false,
        reason: SealedAuditVerifierPolicyReason::ProofCacheRejected,
        persisted_snapshot: false,
        snapshot_count: 0,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_verify_offline: false,
        can_schedule_private_monitor: false,
        can_show_ui_status: false,
        requires_policy_refresh: true,
        requires_monitor_refresh: false,
        requires_key_rotation: false,
        escalates_split_view: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
    }
}

fn sealed_audit_recovery_export_ready_fixture() -> Value {
    sealed_audit_recovery_export_fixture(
        "sealed_audit_recovery_export_ready",
        valid_sealed_audit_recovery_export_write(),
    )
}

fn sealed_audit_recovery_export_incident_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_recovery_export_write();
    write.incident_evidence_decision = rejected_sealed_audit_incident_evidence_decision(false);
    sealed_audit_recovery_export_fixture("sealed_audit_recovery_export_incident_rejected", write)
}

fn sealed_audit_recovery_export_quorum_required_fixture() -> Value {
    let mut write = valid_sealed_audit_recovery_export_write();
    write.restore_quorum_met = false;
    write.approving_device_count = 1;
    sealed_audit_recovery_export_fixture("sealed_audit_recovery_export_quorum_required", write)
}

fn sealed_audit_recovery_export_rollback_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_recovery_export_write();
    write.export_manifest_digest = &SEALED_AUDIT_NEXT_RECOVERY_EXPORT_MANIFEST_DIGEST;
    write.export_sequence = 1;
    write.previous_export_sequence = 1;
    write.previous_export_manifest_digest = &SEALED_AUDIT_RECOVERY_EXPORT_MANIFEST_DIGEST;
    write.previous_export_bound = true;
    sealed_audit_recovery_export_fixture("sealed_audit_recovery_export_rollback_rejected", write)
}

fn sealed_audit_recovery_export_plaintext_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_recovery_export_write();
    write.plaintext_selector_count = 1;
    write.plaintext_metadata_fields = 1;
    sealed_audit_recovery_export_fixture("sealed_audit_recovery_export_plaintext_rejected", write)
}

fn sealed_audit_recovery_export_fixture(
    name: &'static str,
    write: SealedAuditRecoveryExportWrite<'_>,
) -> Value {
    let mut store = PrototypeSealedAuditRecoveryExportStore::default();
    let decision = put_sealed_audit_recovery_export_record(&mut store, write)
        .expect("prototype sealed audit recovery export store cannot fail");
    let record = store
        .get_by_digest(write.export_manifest_digest)
        .map(sealed_audit_recovery_export_record_value);

    let store_state = json!({
        "record_count": store.len(),
        "has_record": record.is_some(),
        "latest_export_sequence": store.latest().map(|record| record.export_sequence),
    });
    let input = json!({
        "incident_evidence_decision": sealed_audit_incident_evidence_decision_value(
            write.incident_evidence_decision,
        ),
        "export_format_version": write.export_format_version,
        "export_manifest_digest_len": write.export_manifest_digest.len(),
        "previous_export_manifest_digest_len": write.previous_export_manifest_digest.len(),
        "device_set_digest_len": write.device_set_digest.len(),
        "recovery_policy_digest_len": write.recovery_policy_digest.len(),
        "verifier_policy_digest_len": write.verifier_policy_digest.len(),
        "proof_cache_digest_len": write.proof_cache_digest.len(),
        "incident_id_len": write.incident_id.len(),
        "incident_evidence_digest_len": write.incident_evidence_digest.len(),
        "export_ciphertext_digest_len": write.export_ciphertext_digest.len(),
        "restore_authorization_digest_len": write.restore_authorization_digest.len(),
        "sync_state_digest_len": write.sync_state_digest.len(),
        "audit_log_checkpoint_digest_len": write.audit_log_checkpoint_digest.len(),
        "export_sequence": write.export_sequence,
        "previous_export_sequence": write.previous_export_sequence,
        "policy_epoch": write.policy_epoch,
        "proof_cache_log_index": write.proof_cache_log_index,
        "latest_checked_log_index": write.latest_checked_log_index,
        "created_at_s": write.created_at_s,
        "expires_at_s": write.expires_at_s,
        "restored_at_s": write.restored_at_s,
        "device_count": write.device_count,
        "device_quorum_threshold": write.device_quorum_threshold,
        "approving_device_count": write.approving_device_count,
        "recovery_share_count": write.recovery_share_count,
        "recovery_share_threshold": write.recovery_share_threshold,
        "manifest_signature_verified": write.manifest_signature_verified,
        "device_binding_verified": write.device_binding_verified,
        "recovery_policy_verified": write.recovery_policy_verified,
        "export_ciphertext_encrypted": write.export_ciphertext_encrypted,
        "export_ciphertext_authenticated": write.export_ciphertext_authenticated,
        "restore_authorization_verified": write.restore_authorization_verified,
        "restore_quorum_met": write.restore_quorum_met,
        "rollback_guard_verified": write.rollback_guard_verified,
        "previous_export_bound": write.previous_export_bound,
        "cross_device_sync_private": write.cross_device_sync_private,
        "incident_selectors_redacted": write.incident_selectors_redacted,
        "audit_log_checkpoint_verified": write.audit_log_checkpoint_verified,
        "store_record_encrypted": write.store_record_encrypted,
        "append_only_guard": write.append_only_guard,
        "plaintext_selector_count": write.plaintext_selector_count,
        "plaintext_metadata_fields": write.plaintext_metadata_fields,
        "ui_status_digest_only": write.ui_status_digest_only,
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_recovery_export",
        "input": input,
        "decision": sealed_audit_recovery_export_decision_value(decision),
        "store": store_state,
        "record": record,
    })
}

fn sealed_audit_recovery_export_decision_value(
    decision: SealedAuditRecoveryExportDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "export_sequence": decision.export_sequence,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
        "can_export_state": decision.can_export_state,
        "can_restore_state": decision.can_restore_state,
        "can_sync_cross_device": decision.can_sync_cross_device,
        "requires_restore_quorum": decision.requires_restore_quorum,
        "requires_policy_refresh": decision.requires_policy_refresh,
        "rejects_rollback": decision.rejects_rollback,
        "requires_device_binding": decision.requires_device_binding,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_recovery_export_record_value(
    record: &mercury_core::SealedAuditRecoveryExportRecord,
) -> Value {
    json!({
        "export_manifest_digest_len": record.export_manifest_digest.len(),
        "previous_export_manifest_digest_len": record.previous_export_manifest_digest.len(),
        "device_set_digest_len": record.device_set_digest.len(),
        "recovery_policy_digest_len": record.recovery_policy_digest.len(),
        "verifier_policy_digest_len": record.verifier_policy_digest.len(),
        "proof_cache_digest_len": record.proof_cache_digest.len(),
        "incident_id_len": record.incident_id.len(),
        "incident_evidence_digest_len": record.incident_evidence_digest.len(),
        "export_ciphertext_digest_len": record.export_ciphertext_digest.len(),
        "restore_authorization_digest_len": record.restore_authorization_digest.len(),
        "sync_state_digest_len": record.sync_state_digest.len(),
        "audit_log_checkpoint_digest_len": record.audit_log_checkpoint_digest.len(),
        "export_sequence": record.export_sequence,
        "previous_export_sequence": record.previous_export_sequence,
        "policy_epoch": record.policy_epoch,
        "proof_cache_log_index": record.proof_cache_log_index,
        "latest_checked_log_index": record.latest_checked_log_index,
        "created_at_s": record.created_at_s,
        "expires_at_s": record.expires_at_s,
        "restored_at_s": record.restored_at_s,
        "device_count": record.device_count,
        "device_quorum_threshold": record.device_quorum_threshold,
        "approving_device_count": record.approving_device_count,
        "recovery_share_count": record.recovery_share_count,
        "recovery_share_threshold": record.recovery_share_threshold,
        "can_export_state": record.can_export_state,
        "can_restore_state": record.can_restore_state,
        "can_sync_cross_device": record.can_sync_cross_device,
        "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
    })
}

fn sealed_audit_database_adapter_ready_fixture() -> Value {
    sealed_audit_database_adapter_fixture(
        "sealed_audit_database_adapter_ready",
        valid_sealed_audit_database_adapter_input(),
    )
}

fn sealed_audit_database_adapter_encryption_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_database_adapter_input();
    input.all_tables_encrypted = false;
    input.wal_encrypted = false;
    sealed_audit_database_adapter_fixture(
        "sealed_audit_database_adapter_encryption_rejected",
        input,
    )
}

fn sealed_audit_database_adapter_append_only_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_database_adapter_input();
    input.append_only_incident_table = false;
    input.monotonic_sequence_constraints = false;
    sealed_audit_database_adapter_fixture(
        "sealed_audit_database_adapter_append_only_rejected",
        input,
    )
}

fn sealed_audit_database_adapter_fixture(
    name: &'static str,
    input: SealedAuditDatabaseAdapterInput<'_>,
) -> Value {
    let decision = input.evaluate();
    json!({
        "fixture": name,
        "surface": "sealed_audit_database_adapter",
        "input": sealed_audit_database_adapter_input_value(input),
        "decision": sealed_audit_database_adapter_decision_value(decision),
    })
}

fn sealed_audit_database_adapter_input_value(input: SealedAuditDatabaseAdapterInput<'_>) -> Value {
    json!({
        "recovery_export_decision": sealed_audit_recovery_export_decision_value(
            input.recovery_export_decision,
        ),
        "database_selection_decision": local_store_database_adapter_selection_decision_value(
            input.database_selection_decision,
        ),
        "adapter_format_version": input.adapter_format_version,
        "database_profile_digest_len": input.database_profile_digest.len(),
        "schema_digest_len": input.schema_digest.len(),
        "event_table_digest_len": input.event_table_digest.len(),
        "proof_cache_table_digest_len": input.proof_cache_table_digest.len(),
        "verifier_policy_table_digest_len": input.verifier_policy_table_digest.len(),
        "incident_evidence_table_digest_len": input.incident_evidence_table_digest.len(),
        "recovery_export_table_digest_len": input.recovery_export_table_digest.len(),
        "checkpoint_table_digest_len": input.checkpoint_table_digest.len(),
        "migration_plan_digest_len": input.migration_plan_digest.len(),
        "crash_recovery_plan_digest_len": input.crash_recovery_plan_digest.len(),
        "latest_export_sequence": input.latest_export_sequence,
        "policy_epoch": input.policy_epoch,
        "proof_cache_log_index": input.proof_cache_log_index,
        "latest_checked_log_index": input.latest_checked_log_index,
        "plaintext_header_bytes": input.plaintext_header_bytes,
        "plaintext_selector_count": input.plaintext_selector_count,
        "plaintext_metadata_fields": input.plaintext_metadata_fields,
        "all_tables_encrypted": input.all_tables_encrypted,
        "wal_encrypted": input.wal_encrypted,
        "temp_store_memory_only": input.temp_store_memory_only,
        "page_authentication_enabled": input.page_authentication_enabled,
        "platform_key_wrapping_enabled": input.platform_key_wrapping_enabled,
        "key_rotation_supported": input.key_rotation_supported,
        "cipher_integrity_check_passed": input.cipher_integrity_check_passed,
        "append_only_event_table": input.append_only_event_table,
        "append_only_proof_cache_table": input.append_only_proof_cache_table,
        "append_only_policy_table": input.append_only_policy_table,
        "append_only_incident_table": input.append_only_incident_table,
        "append_only_recovery_export_table": input.append_only_recovery_export_table,
        "monotonic_sequence_constraints": input.monotonic_sequence_constraints,
        "duplicate_digest_constraints": input.duplicate_digest_constraints,
        "transactional_batch_writes": input.transactional_batch_writes,
        "wal_checkpoint_policy_verified": input.wal_checkpoint_policy_verified,
        "deterministic_migration_tested": input.deterministic_migration_tested,
        "crash_recovery_drill_passed": input.crash_recovery_drill_passed,
        "plaintext_free_schema": input.plaintext_free_schema,
        "ui_status_digest_only": input.ui_status_digest_only,
    })
}

fn sealed_audit_database_adapter_decision_value(
    decision: SealedAuditDatabaseAdapterDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_open_database": decision.can_open_database,
        "can_persist_sealed_audit": decision.can_persist_sealed_audit,
        "can_run_migration": decision.can_run_migration,
        "requires_database_encryption": decision.requires_database_encryption,
        "requires_append_only_guard": decision.requires_append_only_guard,
        "requires_migration_drill": decision.requires_migration_drill,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "latest_export_sequence": decision.latest_export_sequence,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
    })
}

fn sealed_audit_private_report_transport_ready_fixture() -> Value {
    sealed_audit_private_report_transport_fixture(
        "sealed_audit_private_report_transport_ready",
        valid_sealed_audit_private_report_transport_input(),
    )
}

fn sealed_audit_private_report_transport_plaintext_rejected_fixture() -> Value {
    let mut input = valid_sealed_audit_private_report_transport_input();
    input.plaintext_selector_count = 1;
    input.report_payload_digest_only = false;
    sealed_audit_private_report_transport_fixture(
        "sealed_audit_private_report_transport_plaintext_rejected",
        input,
    )
}

fn sealed_audit_private_report_transport_fixture(
    name: &'static str,
    input: SealedAuditPrivateReportTransportInput<'_>,
) -> Value {
    let decision = input.evaluate();
    json!({
        "fixture": name,
        "surface": "sealed_audit_private_report_transport",
        "input": sealed_audit_private_report_transport_input_value(input),
        "decision": sealed_audit_private_report_transport_decision_value(decision),
    })
}

fn sealed_audit_private_report_transport_input_value(
    input: SealedAuditPrivateReportTransportInput<'_>,
) -> Value {
    json!({
        "database_adapter_decision": sealed_audit_database_adapter_decision_value(
            input.database_adapter_decision,
        ),
        "report_format_version": input.report_format_version,
        "report_transport_config_digest_len": input.report_transport_config_digest.len(),
        "ohttp_gateway_key_digest_len": input.ohttp_gateway_key_digest.len(),
        "ohttp_relay_policy_digest_len": input.ohttp_relay_policy_digest.len(),
        "privacy_pass_issuer_key_digest_len": input.privacy_pass_issuer_key_digest.len(),
        "report_outbox_digest_len": input.report_outbox_digest.len(),
        "replay_window_digest_len": input.replay_window_digest.len(),
        "rate_limit_bucket_digest_len": input.rate_limit_bucket_digest.len(),
        "retry_backoff_digest_len": input.retry_backoff_digest.len(),
        "incident_report_schema_digest_len": input.incident_report_schema_digest.len(),
        "audit_checkpoint_digest_len": input.audit_checkpoint_digest.len(),
        "policy_epoch": input.policy_epoch,
        "proof_cache_log_index": input.proof_cache_log_index,
        "latest_checked_log_index": input.latest_checked_log_index,
        "report_window_s": input.report_window_s,
        "max_reports_per_window": input.max_reports_per_window,
        "ohttp_relay_configured": input.ohttp_relay_configured,
        "ohttp_gateway_key_pinned": input.ohttp_gateway_key_pinned,
        "ohttp_target_state_free": input.ohttp_target_state_free,
        "hpke_request_encryption": input.hpke_request_encryption,
        "gateway_response_authenticated": input.gateway_response_authenticated,
        "privacy_pass_tokens_required": input.privacy_pass_tokens_required,
        "privacy_pass_issuer_key_pinned": input.privacy_pass_issuer_key_pinned,
        "anonymous_rate_limit_enforced": input.anonymous_rate_limit_enforced,
        "report_payload_encrypted": input.report_payload_encrypted,
        "report_payload_digest_only": input.report_payload_digest_only,
        "selector_blinding_enabled": input.selector_blinding_enabled,
        "report_outbox_encrypted": input.report_outbox_encrypted,
        "retry_backoff_enabled": input.retry_backoff_enabled,
        "replay_guard_enabled": input.replay_guard_enabled,
        "duplicate_report_rejected": input.duplicate_report_rejected,
        "constant_size_padding_enabled": input.constant_size_padding_enabled,
        "no_cookie_or_auth_state": input.no_cookie_or_auth_state,
        "private_monitor_route_used": input.private_monitor_route_used,
        "ui_status_digest_only": input.ui_status_digest_only,
        "plaintext_selector_count": input.plaintext_selector_count,
        "plaintext_metadata_fields": input.plaintext_metadata_fields,
    })
}

fn sealed_audit_private_report_transport_decision_value(
    decision: SealedAuditPrivateReportTransportDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_submit_private_report": decision.can_submit_private_report,
        "can_retry_safely": decision.can_retry_safely,
        "requires_private_transport": decision.requires_private_transport,
        "requires_replay_guard": decision.requires_replay_guard,
        "requires_rate_limit_token": decision.requires_rate_limit_token,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
    })
}

fn sealed_audit_private_report_outbox_ready_fixture() -> Value {
    sealed_audit_private_report_outbox_fixture(
        "sealed_audit_private_report_outbox_ready",
        valid_sealed_audit_private_report_outbox_write(),
    )
}

fn sealed_audit_private_report_outbox_transport_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_outbox_write();
    write.transport_decision = rejected_sealed_audit_private_report_transport_decision(false);
    sealed_audit_private_report_outbox_fixture(
        "sealed_audit_private_report_outbox_transport_rejected",
        write,
    )
}

fn sealed_audit_private_report_outbox_replay_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_outbox_write();
    write.replay_window_bound = false;
    sealed_audit_private_report_outbox_fixture(
        "sealed_audit_private_report_outbox_replay_rejected",
        write,
    )
}

fn sealed_audit_private_report_outbox_rate_limit_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_outbox_write();
    write.privacy_pass_token_present = false;
    sealed_audit_private_report_outbox_fixture(
        "sealed_audit_private_report_outbox_rate_limit_rejected",
        write,
    )
}

fn sealed_audit_private_report_outbox_plaintext_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_outbox_write();
    write.plaintext_selector_count = 1;
    write.plaintext_metadata_fields = 1;
    sealed_audit_private_report_outbox_fixture(
        "sealed_audit_private_report_outbox_plaintext_rejected",
        write,
    )
}

fn sealed_audit_private_report_outbox_fixture(
    name: &'static str,
    write: SealedAuditPrivateReportOutboxWrite<'_>,
) -> Value {
    let mut store = PrototypeSealedAuditPrivateReportOutbox::default();
    let decision = put_sealed_audit_private_report_outbox_record(&mut store, write)
        .expect("prototype sealed audit private report outbox cannot fail");
    let record = store
        .get_by_id(write.report_id)
        .map(sealed_audit_private_report_outbox_record_value);

    let store_state = json!({
        "record_count": store.len(),
        "has_record": record.is_some(),
        "latest_report_sequence": store.latest().map(|record| record.report_sequence),
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_private_report_outbox",
        "input": sealed_audit_private_report_outbox_input_value(write),
        "decision": sealed_audit_private_report_outbox_decision_value(decision),
        "store": store_state,
        "record": record,
    })
}

fn sealed_audit_private_report_outbox_input_value(
    write: SealedAuditPrivateReportOutboxWrite<'_>,
) -> Value {
    json!({
        "transport_decision": sealed_audit_private_report_transport_decision_value(
            write.transport_decision,
        ),
        "report_format_version": write.report_format_version,
        "report_id_len": write.report_id.len(),
        "previous_report_id_len": write.previous_report_id.len(),
        "incident_id_len": write.incident_id.len(),
        "report_payload_digest_len": write.report_payload_digest.len(),
        "report_schema_digest_len": write.report_schema_digest.len(),
        "ohttp_gateway_key_digest_len": write.ohttp_gateway_key_digest.len(),
        "ohttp_relay_policy_digest_len": write.ohttp_relay_policy_digest.len(),
        "privacy_pass_token_digest_len": write.privacy_pass_token_digest.len(),
        "rate_limit_bucket_digest_len": write.rate_limit_bucket_digest.len(),
        "replay_window_digest_len": write.replay_window_digest.len(),
        "retry_backoff_digest_len": write.retry_backoff_digest.len(),
        "request_transcript_digest_len": write.request_transcript_digest.len(),
        "response_transcript_digest_len": write.response_transcript_digest.len(),
        "audit_checkpoint_digest_len": write.audit_checkpoint_digest.len(),
        "report_sequence": write.report_sequence,
        "previous_report_sequence": write.previous_report_sequence,
        "policy_epoch": write.policy_epoch,
        "proof_cache_log_index": write.proof_cache_log_index,
        "latest_checked_log_index": write.latest_checked_log_index,
        "created_at_s": write.created_at_s,
        "expires_at_s": write.expires_at_s,
        "next_retry_after_s": write.next_retry_after_s,
        "send_attempt_count": write.send_attempt_count,
        "max_send_attempts": write.max_send_attempts,
        "report_window_s": write.report_window_s,
        "max_reports_per_window": write.max_reports_per_window,
        "ohttp_request_encapsulated": write.ohttp_request_encapsulated,
        "gateway_response_encapsulated": write.gateway_response_encapsulated,
        "gateway_response_authenticated": write.gateway_response_authenticated,
        "relay_gateway_separated": write.relay_gateway_separated,
        "no_cookie_or_auth_state": write.no_cookie_or_auth_state,
        "private_route_selected": write.private_route_selected,
        "privacy_pass_token_present": write.privacy_pass_token_present,
        "privacy_pass_token_bound": write.privacy_pass_token_bound,
        "privacy_pass_token_spent_once": write.privacy_pass_token_spent_once,
        "anonymous_rate_limit_enforced": write.anonymous_rate_limit_enforced,
        "replay_window_bound": write.replay_window_bound,
        "duplicate_report_rejected": write.duplicate_report_rejected,
        "retry_backoff_persisted": write.retry_backoff_persisted,
        "report_payload_encrypted": write.report_payload_encrypted,
        "outbox_record_encrypted": write.outbox_record_encrypted,
        "append_only_guard": write.append_only_guard,
        "plaintext_selector_count": write.plaintext_selector_count,
        "plaintext_metadata_fields": write.plaintext_metadata_fields,
        "ui_status_digest_only": write.ui_status_digest_only,
    })
}

fn sealed_audit_private_report_outbox_decision_value(
    decision: SealedAuditPrivateReportOutboxDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "report_sequence": decision.report_sequence,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
        "can_enqueue_report": decision.can_enqueue_report,
        "can_submit_now": decision.can_submit_now,
        "can_retry_safely": decision.can_retry_safely,
        "requires_private_transport": decision.requires_private_transport,
        "requires_replay_guard": decision.requires_replay_guard,
        "requires_rate_limit_token": decision.requires_rate_limit_token,
        "requires_policy_refresh": decision.requires_policy_refresh,
        "requires_route_privacy": decision.requires_route_privacy,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_private_report_outbox_record_value(
    record: &mercury_core::SealedAuditPrivateReportOutboxRecord,
) -> Value {
    json!({
        "report_id_len": record.report_id.len(),
        "previous_report_id_len": record.previous_report_id.len(),
        "incident_id_len": record.incident_id.len(),
        "report_payload_digest_len": record.report_payload_digest.len(),
        "report_schema_digest_len": record.report_schema_digest.len(),
        "ohttp_gateway_key_digest_len": record.ohttp_gateway_key_digest.len(),
        "ohttp_relay_policy_digest_len": record.ohttp_relay_policy_digest.len(),
        "privacy_pass_token_digest_len": record.privacy_pass_token_digest.len(),
        "rate_limit_bucket_digest_len": record.rate_limit_bucket_digest.len(),
        "replay_window_digest_len": record.replay_window_digest.len(),
        "retry_backoff_digest_len": record.retry_backoff_digest.len(),
        "request_transcript_digest_len": record.request_transcript_digest.len(),
        "response_transcript_digest_len": record.response_transcript_digest.len(),
        "audit_checkpoint_digest_len": record.audit_checkpoint_digest.len(),
        "report_sequence": record.report_sequence,
        "previous_report_sequence": record.previous_report_sequence,
        "policy_epoch": record.policy_epoch,
        "proof_cache_log_index": record.proof_cache_log_index,
        "latest_checked_log_index": record.latest_checked_log_index,
        "created_at_s": record.created_at_s,
        "expires_at_s": record.expires_at_s,
        "next_retry_after_s": record.next_retry_after_s,
        "send_attempt_count": record.send_attempt_count,
        "max_send_attempts": record.max_send_attempts,
        "can_submit_now": record.can_submit_now,
        "can_retry_safely": record.can_retry_safely,
        "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
    })
}

fn sealed_audit_private_report_receipt_ready_fixture() -> Value {
    sealed_audit_private_report_receipt_fixture(
        "sealed_audit_private_report_receipt_ready",
        valid_sealed_audit_private_report_receipt_write(),
    )
}

fn sealed_audit_private_report_receipt_outbox_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_receipt_write();
    write.outbox_decision = rejected_sealed_audit_private_report_outbox_decision(false);
    sealed_audit_private_report_receipt_fixture(
        "sealed_audit_private_report_receipt_outbox_rejected",
        write,
    )
}

fn sealed_audit_private_report_receipt_missing_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_receipt_write();
    write.gateway_receipt_signature_verified = false;
    sealed_audit_private_report_receipt_fixture(
        "sealed_audit_private_report_receipt_missing",
        write,
    )
}

fn sealed_audit_private_report_receipt_transparency_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_receipt_write();
    write.gateway_key_consistency_verified = false;
    sealed_audit_private_report_receipt_fixture(
        "sealed_audit_private_report_receipt_transparency_rejected",
        write,
    )
}

fn sealed_audit_private_report_receipt_plaintext_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_receipt_write();
    write.plaintext_selector_count = 1;
    write.plaintext_metadata_fields = 1;
    sealed_audit_private_report_receipt_fixture(
        "sealed_audit_private_report_receipt_plaintext_rejected",
        write,
    )
}

fn sealed_audit_private_report_receipt_fixture(
    name: &'static str,
    write: SealedAuditPrivateReportReceiptWrite<'_>,
) -> Value {
    let mut store = PrototypeSealedAuditPrivateReportReceiptStore::default();
    let decision = put_sealed_audit_private_report_receipt_record(&mut store, write)
        .expect("prototype sealed audit private report receipt store cannot fail");
    let record = store
        .get_by_id(write.receipt_id)
        .map(sealed_audit_private_report_receipt_record_value);

    let store_state = json!({
        "record_count": store.len(),
        "has_record": record.is_some(),
        "latest_receipt_sequence": store.latest().map(|record| record.receipt_sequence),
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_private_report_receipt",
        "input": sealed_audit_private_report_receipt_input_value(write),
        "decision": sealed_audit_private_report_receipt_decision_value(decision),
        "store": store_state,
        "record": record,
    })
}

fn sealed_audit_private_report_receipt_input_value(
    write: SealedAuditPrivateReportReceiptWrite<'_>,
) -> Value {
    json!({
        "outbox_decision": sealed_audit_private_report_outbox_decision_value(
            write.outbox_decision,
        ),
        "receipt_format_version": write.receipt_format_version,
        "receipt_id_len": write.receipt_id.len(),
        "previous_receipt_id_len": write.previous_receipt_id.len(),
        "report_id_len": write.report_id.len(),
        "gateway_receipt_digest_len": write.gateway_receipt_digest.len(),
        "gateway_signature_key_digest_len": write.gateway_signature_key_digest.len(),
        "gateway_key_transparency_checkpoint_digest_len":
            write.gateway_key_transparency_checkpoint_digest.len(),
        "gateway_key_consistency_proof_digest_len":
            write.gateway_key_consistency_proof_digest.len(),
        "gateway_key_rotation_digest_len": write.gateway_key_rotation_digest.len(),
        "relay_policy_digest_len": write.relay_policy_digest.len(),
        "response_transcript_digest_len": write.response_transcript_digest.len(),
        "monitor_submission_proof_digest_len": write.monitor_submission_proof_digest.len(),
        "blinded_failure_class_digest_len": write.blinded_failure_class_digest.len(),
        "retry_completion_digest_len": write.retry_completion_digest.len(),
        "audit_checkpoint_digest_len": write.audit_checkpoint_digest.len(),
        "receipt_sequence": write.receipt_sequence,
        "previous_receipt_sequence": write.previous_receipt_sequence,
        "report_sequence": write.report_sequence,
        "policy_epoch": write.policy_epoch,
        "proof_cache_log_index": write.proof_cache_log_index,
        "latest_checked_log_index": write.latest_checked_log_index,
        "submitted_at_s": write.submitted_at_s,
        "acknowledged_at_s": write.acknowledged_at_s,
        "expires_at_s": write.expires_at_s,
        "gateway_log_tree_size": write.gateway_log_tree_size,
        "previous_gateway_log_tree_size": write.previous_gateway_log_tree_size,
        "delivery_attempt_count": write.delivery_attempt_count,
        "max_delivery_attempts": write.max_delivery_attempts,
        "gateway_receipt_signature_verified": write.gateway_receipt_signature_verified,
        "receipt_binds_report_id": write.receipt_binds_report_id,
        "receipt_binds_response_transcript": write.receipt_binds_response_transcript,
        "receipt_binds_gateway_key": write.receipt_binds_gateway_key,
        "gateway_key_transparency_verified": write.gateway_key_transparency_verified,
        "gateway_key_consistency_verified": write.gateway_key_consistency_verified,
        "gateway_key_not_stale": write.gateway_key_not_stale,
        "gateway_key_rotation_authenticated": write.gateway_key_rotation_authenticated,
        "relay_policy_bound": write.relay_policy_bound,
        "monitor_submission_proof_verified": write.monitor_submission_proof_verified,
        "monitor_route_private": write.monitor_route_private,
        "completion_state_monotonic": write.completion_state_monotonic,
        "delivery_replay_rejected": write.delivery_replay_rejected,
        "duplicate_receipt_rejected": write.duplicate_receipt_rejected,
        "blinded_failure_classification": write.blinded_failure_classification,
        "retry_completion_persisted": write.retry_completion_persisted,
        "report_marked_delivered_only_after_receipt":
            write.report_marked_delivered_only_after_receipt,
        "receipt_record_encrypted": write.receipt_record_encrypted,
        "append_only_guard": write.append_only_guard,
        "plaintext_selector_count": write.plaintext_selector_count,
        "plaintext_metadata_fields": write.plaintext_metadata_fields,
        "ui_status_digest_only": write.ui_status_digest_only,
    })
}

fn sealed_audit_private_report_receipt_decision_value(
    decision: SealedAuditPrivateReportReceiptDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "receipt_sequence": decision.receipt_sequence,
        "report_sequence": decision.report_sequence,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
        "can_mark_delivered": decision.can_mark_delivered,
        "can_stop_retrying": decision.can_stop_retrying,
        "can_show_delivery_status": decision.can_show_delivery_status,
        "requires_private_report_outbox": decision.requires_private_report_outbox,
        "requires_receipt": decision.requires_receipt,
        "requires_gateway_transparency": decision.requires_gateway_transparency,
        "requires_delivery_replay_guard": decision.requires_delivery_replay_guard,
        "requires_monitor_proof": decision.requires_monitor_proof,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_private_report_receipt_record_value(
    record: &mercury_core::SealedAuditPrivateReportReceiptRecord,
) -> Value {
    json!({
        "receipt_id_len": record.receipt_id.len(),
        "previous_receipt_id_len": record.previous_receipt_id.len(),
        "report_id_len": record.report_id.len(),
        "gateway_receipt_digest_len": record.gateway_receipt_digest.len(),
        "gateway_signature_key_digest_len": record.gateway_signature_key_digest.len(),
        "gateway_key_transparency_checkpoint_digest_len":
            record.gateway_key_transparency_checkpoint_digest.len(),
        "gateway_key_consistency_proof_digest_len":
            record.gateway_key_consistency_proof_digest.len(),
        "gateway_key_rotation_digest_len": record.gateway_key_rotation_digest.len(),
        "relay_policy_digest_len": record.relay_policy_digest.len(),
        "response_transcript_digest_len": record.response_transcript_digest.len(),
        "monitor_submission_proof_digest_len": record.monitor_submission_proof_digest.len(),
        "blinded_failure_class_digest_len": record.blinded_failure_class_digest.len(),
        "retry_completion_digest_len": record.retry_completion_digest.len(),
        "audit_checkpoint_digest_len": record.audit_checkpoint_digest.len(),
        "receipt_sequence": record.receipt_sequence,
        "previous_receipt_sequence": record.previous_receipt_sequence,
        "report_sequence": record.report_sequence,
        "policy_epoch": record.policy_epoch,
        "proof_cache_log_index": record.proof_cache_log_index,
        "latest_checked_log_index": record.latest_checked_log_index,
        "submitted_at_s": record.submitted_at_s,
        "acknowledged_at_s": record.acknowledged_at_s,
        "expires_at_s": record.expires_at_s,
        "gateway_log_tree_size": record.gateway_log_tree_size,
        "previous_gateway_log_tree_size": record.previous_gateway_log_tree_size,
        "delivery_attempt_count": record.delivery_attempt_count,
        "max_delivery_attempts": record.max_delivery_attempts,
        "can_mark_delivered": record.can_mark_delivered,
        "can_stop_retrying": record.can_stop_retrying,
        "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
    })
}

fn sealed_audit_private_report_reconciliation_ready_fixture() -> Value {
    sealed_audit_private_report_reconciliation_fixture(
        "sealed_audit_private_report_reconciliation_ready",
        valid_sealed_audit_private_report_reconciliation_write(),
    )
}

fn sealed_audit_private_report_reconciliation_receipt_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_reconciliation_write();
    write.receipt_decision = rejected_sealed_audit_private_report_receipt_decision(false);
    sealed_audit_private_report_reconciliation_fixture(
        "sealed_audit_private_report_reconciliation_receipt_rejected",
        write,
    )
}

fn sealed_audit_private_report_reconciliation_retry_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_reconciliation_write();
    write.retry_schedule_bound = false;
    sealed_audit_private_report_reconciliation_fixture(
        "sealed_audit_private_report_reconciliation_retry_rejected",
        write,
    )
}

fn sealed_audit_private_report_reconciliation_false_delivery_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_reconciliation_write();
    write.receipt_present = false;
    sealed_audit_private_report_reconciliation_fixture(
        "sealed_audit_private_report_reconciliation_false_delivery_rejected",
        write,
    )
}

fn sealed_audit_private_report_reconciliation_plaintext_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_reconciliation_write();
    write.plaintext_selector_count = 1;
    write.plaintext_metadata_fields = 1;
    sealed_audit_private_report_reconciliation_fixture(
        "sealed_audit_private_report_reconciliation_plaintext_rejected",
        write,
    )
}

fn sealed_audit_private_report_reconciliation_fixture(
    name: &'static str,
    write: SealedAuditPrivateReportReconciliationWrite<'_>,
) -> Value {
    let mut store = PrototypeSealedAuditPrivateReportReconciliationStore::default();
    let decision = put_sealed_audit_private_report_reconciliation_record(&mut store, write)
        .expect("prototype sealed audit private report reconciliation store cannot fail");
    let record = store
        .get_by_id(write.reconciliation_id)
        .map(sealed_audit_private_report_reconciliation_record_value);

    let store_state = json!({
        "record_count": store.len(),
        "has_record": record.is_some(),
        "latest_reconciliation_sequence":
            store.latest().map(|record| record.reconciliation_sequence),
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_private_report_reconciliation",
        "input": sealed_audit_private_report_reconciliation_input_value(write),
        "decision": sealed_audit_private_report_reconciliation_decision_value(decision),
        "store": store_state,
        "record": record,
    })
}

fn sealed_audit_private_report_reconciliation_input_value(
    write: SealedAuditPrivateReportReconciliationWrite<'_>,
) -> Value {
    json!({
        "receipt_decision": sealed_audit_private_report_receipt_decision_value(
            write.receipt_decision,
        ),
        "reconciliation_format_version": write.reconciliation_format_version,
        "reconciliation_id_len": write.reconciliation_id.len(),
        "previous_reconciliation_id_len": write.previous_reconciliation_id.len(),
        "report_id_len": write.report_id.len(),
        "receipt_id_len": write.receipt_id.len(),
        "pending_outbox_digest_len": write.pending_outbox_digest.len(),
        "retry_schedule_digest_len": write.retry_schedule_digest.len(),
        "rate_limit_state_digest_len": write.rate_limit_state_digest.len(),
        "delivered_state_digest_len": write.delivered_state_digest.len(),
        "failure_bucket_digest_len": write.failure_bucket_digest.len(),
        "operator_accountability_route_digest_len":
            write.operator_accountability_route_digest.len(),
        "crash_recovery_cursor_digest_len": write.crash_recovery_cursor_digest.len(),
        "audit_checkpoint_digest_len": write.audit_checkpoint_digest.len(),
        "reconciliation_sequence": write.reconciliation_sequence,
        "previous_reconciliation_sequence": write.previous_reconciliation_sequence,
        "report_sequence": write.report_sequence,
        "receipt_sequence": write.receipt_sequence,
        "policy_epoch": write.policy_epoch,
        "proof_cache_log_index": write.proof_cache_log_index,
        "latest_checked_log_index": write.latest_checked_log_index,
        "created_at_s": write.created_at_s,
        "next_retry_after_s": write.next_retry_after_s,
        "expires_at_s": write.expires_at_s,
        "retry_attempt_count": write.retry_attempt_count,
        "max_retry_attempts": write.max_retry_attempts,
        "reports_remaining_in_window": write.reports_remaining_in_window,
        "window_resets_at_s": write.window_resets_at_s,
        "receipt_present": write.receipt_present,
        "pending_outbox_bound": write.pending_outbox_bound,
        "delivered_state_requires_receipt": write.delivered_state_requires_receipt,
        "retry_schedule_bound": write.retry_schedule_bound,
        "retry_after_monotonic": write.retry_after_monotonic,
        "duplicate_retry_rejected": write.duplicate_retry_rejected,
        "retry_idempotency_key_bound": write.retry_idempotency_key_bound,
        "no_retry_after_delivered": write.no_retry_after_delivered,
        "rate_limit_window_bound": write.rate_limit_window_bound,
        "rate_limit_token_spend_preserved": write.rate_limit_token_spend_preserved,
        "retry_does_not_mint_new_report": write.retry_does_not_mint_new_report,
        "crash_recovery_cursor_bound": write.crash_recovery_cursor_bound,
        "resumes_pending_only": write.resumes_pending_only,
        "operator_accountability_route_bound": write.operator_accountability_route_bound,
        "missing_receipt_escalates": write.missing_receipt_escalates,
        "blinded_failure_bucket_only": write.blinded_failure_bucket_only,
        "reconciliation_record_encrypted": write.reconciliation_record_encrypted,
        "append_only_guard": write.append_only_guard,
        "plaintext_selector_count": write.plaintext_selector_count,
        "plaintext_metadata_fields": write.plaintext_metadata_fields,
        "ui_status_digest_only": write.ui_status_digest_only,
    })
}

fn sealed_audit_private_report_reconciliation_decision_value(
    decision: SealedAuditPrivateReportReconciliationDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "reconciliation_sequence": decision.reconciliation_sequence,
        "report_sequence": decision.report_sequence,
        "receipt_sequence": decision.receipt_sequence,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
        "can_reconcile_delivery": decision.can_reconcile_delivery,
        "can_schedule_retry": decision.can_schedule_retry,
        "can_show_retry_status": decision.can_show_retry_status,
        "requires_private_report_receipt": decision.requires_private_report_receipt,
        "requires_retry_schedule": decision.requires_retry_schedule,
        "requires_rate_limit_continuity": decision.requires_rate_limit_continuity,
        "rejects_false_delivery": decision.rejects_false_delivery,
        "requires_operator_accountability": decision.requires_operator_accountability,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_private_report_reconciliation_record_value(
    record: &mercury_core::SealedAuditPrivateReportReconciliationRecord,
) -> Value {
    json!({
        "reconciliation_id_len": record.reconciliation_id.len(),
        "previous_reconciliation_id_len": record.previous_reconciliation_id.len(),
        "report_id_len": record.report_id.len(),
        "receipt_id_len": record.receipt_id.len(),
        "pending_outbox_digest_len": record.pending_outbox_digest.len(),
        "retry_schedule_digest_len": record.retry_schedule_digest.len(),
        "rate_limit_state_digest_len": record.rate_limit_state_digest.len(),
        "delivered_state_digest_len": record.delivered_state_digest.len(),
        "failure_bucket_digest_len": record.failure_bucket_digest.len(),
        "operator_accountability_route_digest_len":
            record.operator_accountability_route_digest.len(),
        "crash_recovery_cursor_digest_len": record.crash_recovery_cursor_digest.len(),
        "audit_checkpoint_digest_len": record.audit_checkpoint_digest.len(),
        "reconciliation_sequence": record.reconciliation_sequence,
        "previous_reconciliation_sequence": record.previous_reconciliation_sequence,
        "report_sequence": record.report_sequence,
        "receipt_sequence": record.receipt_sequence,
        "policy_epoch": record.policy_epoch,
        "proof_cache_log_index": record.proof_cache_log_index,
        "latest_checked_log_index": record.latest_checked_log_index,
        "created_at_s": record.created_at_s,
        "next_retry_after_s": record.next_retry_after_s,
        "expires_at_s": record.expires_at_s,
        "retry_attempt_count": record.retry_attempt_count,
        "max_retry_attempts": record.max_retry_attempts,
        "reports_remaining_in_window": record.reports_remaining_in_window,
        "window_resets_at_s": record.window_resets_at_s,
        "can_reconcile_delivery": record.can_reconcile_delivery,
        "can_schedule_retry": record.can_schedule_retry,
        "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
    })
}

fn sealed_audit_private_report_gateway_evidence_ready_fixture() -> Value {
    sealed_audit_private_report_gateway_evidence_fixture(
        "sealed_audit_private_report_gateway_evidence_ready",
        valid_sealed_audit_private_report_gateway_evidence_write(),
    )
}

fn sealed_audit_private_report_gateway_evidence_reconciliation_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_gateway_evidence_write();
    write.reconciliation_decision =
        rejected_sealed_audit_private_report_reconciliation_decision(false);
    sealed_audit_private_report_gateway_evidence_fixture(
        "sealed_audit_private_report_gateway_evidence_reconciliation_rejected",
        write,
    )
}

fn sealed_audit_private_report_gateway_evidence_unavailable_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_gateway_evidence_write();
    write.no_client_asserted_unavailability = false;
    sealed_audit_private_report_gateway_evidence_fixture(
        "sealed_audit_private_report_gateway_evidence_unavailable_rejected",
        write,
    )
}

fn sealed_audit_private_report_gateway_evidence_accountability_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_gateway_evidence_write();
    write.accountability_route_bound = false;
    sealed_audit_private_report_gateway_evidence_fixture(
        "sealed_audit_private_report_gateway_evidence_accountability_rejected",
        write,
    )
}

fn sealed_audit_private_report_gateway_evidence_plaintext_rejected_fixture() -> Value {
    let mut write = valid_sealed_audit_private_report_gateway_evidence_write();
    write.plaintext_selector_count = 1;
    write.plaintext_metadata_fields = 1;
    sealed_audit_private_report_gateway_evidence_fixture(
        "sealed_audit_private_report_gateway_evidence_plaintext_rejected",
        write,
    )
}

fn sealed_audit_private_report_gateway_evidence_fixture(
    name: &'static str,
    write: SealedAuditPrivateReportGatewayEvidenceWrite<'_>,
) -> Value {
    let mut store = PrototypeSealedAuditPrivateReportGatewayEvidenceStore::default();
    let decision = put_sealed_audit_private_report_gateway_evidence_record(&mut store, write)
        .expect("prototype sealed audit private report gateway evidence store cannot fail");
    let record = store
        .get_by_id(write.evidence_id)
        .map(sealed_audit_private_report_gateway_evidence_record_value);

    let store_state = json!({
        "record_count": store.len(),
        "has_record": record.is_some(),
        "latest_evidence_sequence": store.latest().map(|record| record.evidence_sequence),
    });

    json!({
        "fixture": name,
        "surface": "sealed_audit_private_report_gateway_evidence",
        "input": sealed_audit_private_report_gateway_evidence_input_value(write),
        "decision": sealed_audit_private_report_gateway_evidence_decision_value(decision),
        "store": store_state,
        "record": record,
    })
}

fn sealed_audit_private_report_gateway_evidence_input_value(
    write: SealedAuditPrivateReportGatewayEvidenceWrite<'_>,
) -> Value {
    json!({
        "reconciliation_decision": sealed_audit_private_report_reconciliation_decision_value(
            write.reconciliation_decision,
        ),
        "evidence_format_version": write.evidence_format_version,
        "evidence_id_len": write.evidence_id.len(),
        "previous_evidence_id_len": write.previous_evidence_id.len(),
        "reconciliation_id_len": write.reconciliation_id.len(),
        "report_id_len": write.report_id.len(),
        "receipt_id_len": write.receipt_id.len(),
        "unavailable_evidence_digest_len": write.unavailable_evidence_digest.len(),
        "relay_observation_digest_len": write.relay_observation_digest.len(),
        "gateway_error_digest_len": write.gateway_error_digest.len(),
        "target_absence_digest_len": write.target_absence_digest.len(),
        "retry_exhaustion_digest_len": write.retry_exhaustion_digest.len(),
        "rate_limit_state_digest_len": write.rate_limit_state_digest.len(),
        "gateway_key_state_digest_len": write.gateway_key_state_digest.len(),
        "accountability_route_digest_len": write.accountability_route_digest.len(),
        "blinded_failure_bucket_digest_len": write.blinded_failure_bucket_digest.len(),
        "monitor_submission_digest_len": write.monitor_submission_digest.len(),
        "audit_checkpoint_digest_len": write.audit_checkpoint_digest.len(),
        "evidence_sequence": write.evidence_sequence,
        "previous_evidence_sequence": write.previous_evidence_sequence,
        "reconciliation_sequence": write.reconciliation_sequence,
        "report_sequence": write.report_sequence,
        "receipt_sequence": write.receipt_sequence,
        "policy_epoch": write.policy_epoch,
        "proof_cache_log_index": write.proof_cache_log_index,
        "latest_checked_log_index": write.latest_checked_log_index,
        "created_at_s": write.created_at_s,
        "expires_at_s": write.expires_at_s,
        "retry_attempt_count": write.retry_attempt_count,
        "max_retry_attempts": write.max_retry_attempts,
        "gateway_status_code": write.gateway_status_code,
        "reconciliation_bound": write.reconciliation_bound,
        "unavailable_evidence_gateway_authenticated":
            write.unavailable_evidence_gateway_authenticated,
        "relay_observation_signed": write.relay_observation_signed,
        "target_absence_proof_bound": write.target_absence_proof_bound,
        "gateway_timeout_or_5xx_classified": write.gateway_timeout_or_5xx_classified,
        "no_client_asserted_unavailability": write.no_client_asserted_unavailability,
        "retry_exhaustion_bound": write.retry_exhaustion_bound,
        "rate_limit_continuity_bound": write.rate_limit_continuity_bound,
        "gateway_key_state_bound": write.gateway_key_state_bound,
        "accountability_route_bound": write.accountability_route_bound,
        "operator_escalation_bound": write.operator_escalation_bound,
        "blinded_failure_bucket_only": write.blinded_failure_bucket_only,
        "monitor_route_private": write.monitor_route_private,
        "incident_visible_only_after_policy": write.incident_visible_only_after_policy,
        "evidence_record_encrypted": write.evidence_record_encrypted,
        "append_only_guard": write.append_only_guard,
        "plaintext_selector_count": write.plaintext_selector_count,
        "plaintext_metadata_fields": write.plaintext_metadata_fields,
        "ui_status_digest_only": write.ui_status_digest_only,
    })
}

fn sealed_audit_private_report_gateway_evidence_decision_value(
    decision: SealedAuditPrivateReportGatewayEvidenceDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "evidence_sequence": decision.evidence_sequence,
        "reconciliation_sequence": decision.reconciliation_sequence,
        "report_sequence": decision.report_sequence,
        "receipt_sequence": decision.receipt_sequence,
        "policy_epoch": decision.policy_epoch,
        "proof_cache_log_index": decision.proof_cache_log_index,
        "latest_checked_log_index": decision.latest_checked_log_index,
        "can_raise_gateway_incident": decision.can_raise_gateway_incident,
        "can_notify_operator": decision.can_notify_operator,
        "can_show_unavailable_status": decision.can_show_unavailable_status,
        "requires_private_report_reconciliation":
            decision.requires_private_report_reconciliation,
        "requires_unavailable_evidence": decision.requires_unavailable_evidence,
        "requires_accountability_route": decision.requires_accountability_route,
        "requires_retry_exhaustion": decision.requires_retry_exhaustion,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn sealed_audit_private_report_gateway_evidence_record_value(
    record: &mercury_core::SealedAuditPrivateReportGatewayEvidenceRecord,
) -> Value {
    json!({
        "evidence_id_len": record.evidence_id.len(),
        "previous_evidence_id_len": record.previous_evidence_id.len(),
        "reconciliation_id_len": record.reconciliation_id.len(),
        "report_id_len": record.report_id.len(),
        "receipt_id_len": record.receipt_id.len(),
        "unavailable_evidence_digest_len": record.unavailable_evidence_digest.len(),
        "relay_observation_digest_len": record.relay_observation_digest.len(),
        "gateway_error_digest_len": record.gateway_error_digest.len(),
        "target_absence_digest_len": record.target_absence_digest.len(),
        "retry_exhaustion_digest_len": record.retry_exhaustion_digest.len(),
        "rate_limit_state_digest_len": record.rate_limit_state_digest.len(),
        "gateway_key_state_digest_len": record.gateway_key_state_digest.len(),
        "accountability_route_digest_len": record.accountability_route_digest.len(),
        "blinded_failure_bucket_digest_len": record.blinded_failure_bucket_digest.len(),
        "monitor_submission_digest_len": record.monitor_submission_digest.len(),
        "audit_checkpoint_digest_len": record.audit_checkpoint_digest.len(),
        "evidence_sequence": record.evidence_sequence,
        "previous_evidence_sequence": record.previous_evidence_sequence,
        "reconciliation_sequence": record.reconciliation_sequence,
        "report_sequence": record.report_sequence,
        "receipt_sequence": record.receipt_sequence,
        "policy_epoch": record.policy_epoch,
        "proof_cache_log_index": record.proof_cache_log_index,
        "latest_checked_log_index": record.latest_checked_log_index,
        "created_at_s": record.created_at_s,
        "expires_at_s": record.expires_at_s,
        "retry_attempt_count": record.retry_attempt_count,
        "max_retry_attempts": record.max_retry_attempts,
        "gateway_status_code": record.gateway_status_code,
        "can_raise_gateway_incident": record.can_raise_gateway_incident,
        "can_notify_operator": record.can_notify_operator,
        "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
    })
}

fn valid_sealed_audit_database_adapter_input() -> SealedAuditDatabaseAdapterInput<'static> {
    let recovery_export_decision = accepted_sealed_audit_recovery_export_decision();
    SealedAuditDatabaseAdapterInput {
        recovery_export_decision,
        database_selection_decision: valid_local_store_database_adapter_selection_input()
            .evaluate(),
        adapter_format_version: 1,
        database_profile_digest: &SEALED_AUDIT_DATABASE_PROFILE_DIGEST,
        schema_digest: &SEALED_AUDIT_DATABASE_SCHEMA_DIGEST,
        event_table_digest: &SEALED_AUDIT_EVENT_TABLE_DIGEST,
        proof_cache_table_digest: &SEALED_AUDIT_PROOF_CACHE_TABLE_DIGEST,
        verifier_policy_table_digest: &SEALED_AUDIT_VERIFIER_POLICY_TABLE_DIGEST,
        incident_evidence_table_digest: &SEALED_AUDIT_INCIDENT_EVIDENCE_TABLE_DIGEST,
        recovery_export_table_digest: &SEALED_AUDIT_RECOVERY_EXPORT_TABLE_DIGEST,
        checkpoint_table_digest: &SEALED_AUDIT_CHECKPOINT_TABLE_DIGEST,
        migration_plan_digest: &SEALED_AUDIT_DATABASE_MIGRATION_PLAN_DIGEST,
        crash_recovery_plan_digest: &SEALED_AUDIT_DATABASE_CRASH_RECOVERY_PLAN_DIGEST,
        latest_export_sequence: recovery_export_decision.export_sequence,
        policy_epoch: recovery_export_decision.policy_epoch,
        proof_cache_log_index: recovery_export_decision.proof_cache_log_index,
        latest_checked_log_index: recovery_export_decision.latest_checked_log_index,
        plaintext_header_bytes: 0,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        all_tables_encrypted: true,
        wal_encrypted: true,
        temp_store_memory_only: true,
        page_authentication_enabled: true,
        platform_key_wrapping_enabled: true,
        key_rotation_supported: true,
        cipher_integrity_check_passed: true,
        append_only_event_table: true,
        append_only_proof_cache_table: true,
        append_only_policy_table: true,
        append_only_incident_table: true,
        append_only_recovery_export_table: true,
        monotonic_sequence_constraints: true,
        duplicate_digest_constraints: true,
        transactional_batch_writes: true,
        wal_checkpoint_policy_verified: true,
        deterministic_migration_tested: true,
        crash_recovery_drill_passed: true,
        plaintext_free_schema: true,
        ui_status_digest_only: true,
    }
}

fn valid_sealed_audit_private_report_transport_input()
-> SealedAuditPrivateReportTransportInput<'static> {
    let database_adapter_decision = accepted_sealed_audit_database_adapter_decision();
    SealedAuditPrivateReportTransportInput {
        database_adapter_decision,
        report_format_version: 1,
        report_transport_config_digest: &SEALED_AUDIT_REPORT_TRANSPORT_CONFIG_DIGEST,
        ohttp_gateway_key_digest: &SEALED_AUDIT_OHTTP_GATEWAY_KEY_DIGEST,
        ohttp_relay_policy_digest: &SEALED_AUDIT_OHTTP_RELAY_POLICY_DIGEST,
        privacy_pass_issuer_key_digest: &SEALED_AUDIT_PRIVACY_PASS_ISSUER_KEY_DIGEST,
        report_outbox_digest: &SEALED_AUDIT_PRIVATE_REPORT_OUTBOX_DIGEST,
        replay_window_digest: &SEALED_AUDIT_REPORT_REPLAY_WINDOW_DIGEST,
        rate_limit_bucket_digest: &SEALED_AUDIT_REPORT_RATE_LIMIT_BUCKET_DIGEST,
        retry_backoff_digest: &SEALED_AUDIT_REPORT_RETRY_BACKOFF_DIGEST,
        incident_report_schema_digest: &SEALED_AUDIT_INCIDENT_REPORT_SCHEMA_DIGEST,
        audit_checkpoint_digest: &SEALED_AUDIT_REPORT_AUDIT_CHECKPOINT_DIGEST,
        policy_epoch: database_adapter_decision.policy_epoch,
        proof_cache_log_index: database_adapter_decision.proof_cache_log_index,
        latest_checked_log_index: database_adapter_decision.latest_checked_log_index,
        report_window_s: 3600,
        max_reports_per_window: 3,
        ohttp_relay_configured: true,
        ohttp_gateway_key_pinned: true,
        ohttp_target_state_free: true,
        hpke_request_encryption: true,
        gateway_response_authenticated: true,
        privacy_pass_tokens_required: true,
        privacy_pass_issuer_key_pinned: true,
        anonymous_rate_limit_enforced: true,
        report_payload_encrypted: true,
        report_payload_digest_only: true,
        selector_blinding_enabled: true,
        report_outbox_encrypted: true,
        retry_backoff_enabled: true,
        replay_guard_enabled: true,
        duplicate_report_rejected: true,
        constant_size_padding_enabled: true,
        no_cookie_or_auth_state: true,
        private_monitor_route_used: true,
        ui_status_digest_only: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
    }
}

fn accepted_sealed_audit_recovery_export_decision() -> SealedAuditRecoveryExportDecision {
    let mut store = PrototypeSealedAuditRecoveryExportStore::default();
    put_sealed_audit_recovery_export_record(&mut store, valid_sealed_audit_recovery_export_write())
        .expect("prototype sealed audit recovery export store cannot fail")
}

fn accepted_sealed_audit_database_adapter_decision() -> SealedAuditDatabaseAdapterDecision {
    valid_sealed_audit_database_adapter_input().evaluate()
}

fn valid_sealed_audit_private_report_outbox_write() -> SealedAuditPrivateReportOutboxWrite<'static>
{
    let transport_decision = accepted_sealed_audit_private_report_transport_decision();
    SealedAuditPrivateReportOutboxWrite {
        transport_decision,
        report_format_version: 1,
        report_id: &SEALED_AUDIT_PRIVATE_REPORT_ID,
        previous_report_id: &[],
        incident_id: &SEALED_AUDIT_INCIDENT_ID,
        report_payload_digest: &SEALED_AUDIT_PRIVATE_REPORT_PAYLOAD_DIGEST,
        report_schema_digest: &SEALED_AUDIT_INCIDENT_REPORT_SCHEMA_DIGEST,
        ohttp_gateway_key_digest: &SEALED_AUDIT_OHTTP_GATEWAY_KEY_DIGEST,
        ohttp_relay_policy_digest: &SEALED_AUDIT_OHTTP_RELAY_POLICY_DIGEST,
        privacy_pass_token_digest: &SEALED_AUDIT_PRIVACY_PASS_ISSUER_KEY_DIGEST,
        rate_limit_bucket_digest: &SEALED_AUDIT_REPORT_RATE_LIMIT_BUCKET_DIGEST,
        replay_window_digest: &SEALED_AUDIT_REPORT_REPLAY_WINDOW_DIGEST,
        retry_backoff_digest: &SEALED_AUDIT_REPORT_RETRY_BACKOFF_DIGEST,
        request_transcript_digest: &SEALED_AUDIT_PRIVATE_REPORT_REQUEST_TRANSCRIPT_DIGEST,
        response_transcript_digest: &SEALED_AUDIT_PRIVATE_REPORT_RESPONSE_TRANSCRIPT_DIGEST,
        audit_checkpoint_digest: &SEALED_AUDIT_REPORT_AUDIT_CHECKPOINT_DIGEST,
        report_sequence: 1,
        previous_report_sequence: 0,
        policy_epoch: transport_decision.policy_epoch,
        proof_cache_log_index: transport_decision.proof_cache_log_index,
        latest_checked_log_index: transport_decision.latest_checked_log_index,
        created_at_s: 1_769_991_000,
        expires_at_s: 1_769_994_000,
        next_retry_after_s: 1_769_991_060,
        send_attempt_count: 0,
        max_send_attempts: 3,
        report_window_s: 3600,
        max_reports_per_window: 3,
        ohttp_request_encapsulated: true,
        gateway_response_encapsulated: true,
        gateway_response_authenticated: true,
        relay_gateway_separated: true,
        no_cookie_or_auth_state: true,
        private_route_selected: true,
        privacy_pass_token_present: true,
        privacy_pass_token_bound: true,
        privacy_pass_token_spent_once: true,
        anonymous_rate_limit_enforced: true,
        replay_window_bound: true,
        duplicate_report_rejected: true,
        retry_backoff_persisted: true,
        report_payload_encrypted: true,
        outbox_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn accepted_sealed_audit_private_report_outbox_decision() -> SealedAuditPrivateReportOutboxDecision
{
    let mut store = PrototypeSealedAuditPrivateReportOutbox::default();
    put_sealed_audit_private_report_outbox_record(
        &mut store,
        valid_sealed_audit_private_report_outbox_write(),
    )
    .expect("prototype sealed audit private report outbox cannot fail")
}

const fn rejected_sealed_audit_private_report_outbox_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditPrivateReportOutboxDecision {
    SealedAuditPrivateReportOutboxDecision {
        accepted: false,
        reason: mercury_core::SealedAuditPrivateReportOutboxReason::PlaintextMetadataForbidden,
        persisted_record: false,
        record_count: 0,
        report_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_enqueue_report: false,
        can_submit_now: false,
        can_retry_safely: false,
        requires_private_transport: false,
        requires_replay_guard: false,
        requires_rate_limit_token: false,
        requires_policy_refresh: false,
        requires_route_privacy: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
    }
}

fn valid_sealed_audit_private_report_receipt_write() -> SealedAuditPrivateReportReceiptWrite<'static>
{
    let outbox_decision = accepted_sealed_audit_private_report_outbox_decision();
    SealedAuditPrivateReportReceiptWrite {
        outbox_decision,
        receipt_format_version: 1,
        receipt_id: &SEALED_AUDIT_PRIVATE_REPORT_RECEIPT_ID,
        previous_receipt_id: &[],
        report_id: &SEALED_AUDIT_PRIVATE_REPORT_ID,
        gateway_receipt_digest: &SEALED_AUDIT_GATEWAY_RECEIPT_DIGEST,
        gateway_signature_key_digest: &SEALED_AUDIT_GATEWAY_SIGNATURE_KEY_DIGEST,
        gateway_key_transparency_checkpoint_digest:
            &SEALED_AUDIT_GATEWAY_KEY_TRANSPARENCY_CHECKPOINT_DIGEST,
        gateway_key_consistency_proof_digest: &SEALED_AUDIT_GATEWAY_KEY_CONSISTENCY_PROOF_DIGEST,
        gateway_key_rotation_digest: &SEALED_AUDIT_GATEWAY_KEY_ROTATION_DIGEST,
        relay_policy_digest: &SEALED_AUDIT_OHTTP_RELAY_POLICY_DIGEST,
        response_transcript_digest: &SEALED_AUDIT_PRIVATE_REPORT_RESPONSE_TRANSCRIPT_DIGEST,
        monitor_submission_proof_digest: &SEALED_AUDIT_MONITOR_SUBMISSION_PROOF_DIGEST,
        blinded_failure_class_digest: &SEALED_AUDIT_BLINDED_FAILURE_CLASS_DIGEST,
        retry_completion_digest: &SEALED_AUDIT_RETRY_COMPLETION_DIGEST,
        audit_checkpoint_digest: &SEALED_AUDIT_REPORT_AUDIT_CHECKPOINT_DIGEST,
        receipt_sequence: 1,
        previous_receipt_sequence: 0,
        report_sequence: outbox_decision.report_sequence,
        policy_epoch: outbox_decision.policy_epoch,
        proof_cache_log_index: outbox_decision.proof_cache_log_index,
        latest_checked_log_index: outbox_decision.latest_checked_log_index,
        submitted_at_s: 1_769_991_000,
        acknowledged_at_s: 1_769_991_030,
        expires_at_s: 1_769_994_000,
        gateway_log_tree_size: 51,
        previous_gateway_log_tree_size: 50,
        delivery_attempt_count: 1,
        max_delivery_attempts: 3,
        gateway_receipt_signature_verified: true,
        receipt_binds_report_id: true,
        receipt_binds_response_transcript: true,
        receipt_binds_gateway_key: true,
        gateway_key_transparency_verified: true,
        gateway_key_consistency_verified: true,
        gateway_key_not_stale: true,
        gateway_key_rotation_authenticated: true,
        relay_policy_bound: true,
        monitor_submission_proof_verified: true,
        monitor_route_private: true,
        completion_state_monotonic: true,
        delivery_replay_rejected: true,
        duplicate_receipt_rejected: true,
        blinded_failure_classification: true,
        retry_completion_persisted: true,
        report_marked_delivered_only_after_receipt: true,
        receipt_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn accepted_sealed_audit_private_report_receipt_decision() -> SealedAuditPrivateReportReceiptDecision
{
    let mut store = PrototypeSealedAuditPrivateReportReceiptStore::default();
    put_sealed_audit_private_report_receipt_record(
        &mut store,
        valid_sealed_audit_private_report_receipt_write(),
    )
    .expect("prototype sealed audit private report receipt cannot fail")
}

const fn rejected_sealed_audit_private_report_receipt_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditPrivateReportReceiptDecision {
    SealedAuditPrivateReportReceiptDecision {
        accepted: false,
        reason: mercury_core::SealedAuditPrivateReportReceiptReason::PlaintextMetadataForbidden,
        persisted_record: false,
        record_count: 0,
        receipt_sequence: 1,
        report_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_mark_delivered: false,
        can_stop_retrying: false,
        can_show_delivery_status: false,
        requires_private_report_outbox: false,
        requires_receipt: false,
        requires_gateway_transparency: false,
        requires_delivery_replay_guard: false,
        requires_monitor_proof: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
    }
}

fn valid_sealed_audit_private_report_reconciliation_write()
-> SealedAuditPrivateReportReconciliationWrite<'static> {
    let receipt_decision = accepted_sealed_audit_private_report_receipt_decision();
    SealedAuditPrivateReportReconciliationWrite {
        receipt_decision,
        reconciliation_format_version: 1,
        reconciliation_id: &SEALED_AUDIT_PRIVATE_REPORT_RECONCILIATION_ID,
        previous_reconciliation_id: &[],
        report_id: &SEALED_AUDIT_PRIVATE_REPORT_ID,
        receipt_id: &SEALED_AUDIT_PRIVATE_REPORT_RECEIPT_ID,
        pending_outbox_digest: &SEALED_AUDIT_PENDING_OUTBOX_DIGEST,
        retry_schedule_digest: &SEALED_AUDIT_RETRY_SCHEDULE_DIGEST,
        rate_limit_state_digest: &SEALED_AUDIT_RATE_LIMIT_STATE_DIGEST,
        delivered_state_digest: &SEALED_AUDIT_DELIVERED_STATE_DIGEST,
        failure_bucket_digest: &SEALED_AUDIT_FAILURE_BUCKET_DIGEST,
        operator_accountability_route_digest: &SEALED_AUDIT_ACCOUNTABILITY_ROUTE_DIGEST,
        crash_recovery_cursor_digest: &SEALED_AUDIT_CRASH_RECOVERY_CURSOR_DIGEST,
        audit_checkpoint_digest: &SEALED_AUDIT_REPORT_AUDIT_CHECKPOINT_DIGEST,
        reconciliation_sequence: 1,
        previous_reconciliation_sequence: 0,
        report_sequence: receipt_decision.report_sequence,
        receipt_sequence: receipt_decision.receipt_sequence,
        policy_epoch: receipt_decision.policy_epoch,
        proof_cache_log_index: receipt_decision.proof_cache_log_index,
        latest_checked_log_index: receipt_decision.latest_checked_log_index,
        created_at_s: 1_769_991_035,
        next_retry_after_s: 1_769_991_060,
        expires_at_s: 1_769_994_000,
        retry_attempt_count: 1,
        max_retry_attempts: 3,
        reports_remaining_in_window: 2,
        window_resets_at_s: 1_769_994_600,
        receipt_present: true,
        pending_outbox_bound: true,
        delivered_state_requires_receipt: true,
        retry_schedule_bound: true,
        retry_after_monotonic: true,
        duplicate_retry_rejected: true,
        retry_idempotency_key_bound: true,
        no_retry_after_delivered: true,
        rate_limit_window_bound: true,
        rate_limit_token_spend_preserved: true,
        retry_does_not_mint_new_report: true,
        crash_recovery_cursor_bound: true,
        resumes_pending_only: true,
        operator_accountability_route_bound: true,
        missing_receipt_escalates: true,
        blinded_failure_bucket_only: true,
        reconciliation_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn accepted_sealed_audit_private_report_reconciliation_decision()
-> SealedAuditPrivateReportReconciliationDecision {
    let mut store = PrototypeSealedAuditPrivateReportReconciliationStore::default();
    put_sealed_audit_private_report_reconciliation_record(
        &mut store,
        valid_sealed_audit_private_report_reconciliation_write(),
    )
    .expect("prototype sealed audit private report reconciliation store cannot fail")
}

const fn rejected_sealed_audit_private_report_reconciliation_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditPrivateReportReconciliationDecision {
    SealedAuditPrivateReportReconciliationDecision {
        accepted: false,
        reason:
            mercury_core::SealedAuditPrivateReportReconciliationReason::PlaintextMetadataForbidden,
        persisted_record: false,
        record_count: 0,
        reconciliation_sequence: 1,
        report_sequence: 1,
        receipt_sequence: 1,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_reconcile_delivery: false,
        can_schedule_retry: false,
        can_show_retry_status: false,
        requires_private_report_receipt: false,
        requires_retry_schedule: false,
        requires_rate_limit_continuity: false,
        rejects_false_delivery: false,
        requires_operator_accountability: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
    }
}

fn valid_sealed_audit_private_report_gateway_evidence_write()
-> SealedAuditPrivateReportGatewayEvidenceWrite<'static> {
    let reconciliation_decision = accepted_sealed_audit_private_report_reconciliation_decision();
    SealedAuditPrivateReportGatewayEvidenceWrite {
        reconciliation_decision,
        evidence_format_version: 1,
        evidence_id: &SEALED_AUDIT_PRIVATE_REPORT_GATEWAY_EVIDENCE_ID,
        previous_evidence_id: &[],
        reconciliation_id: &SEALED_AUDIT_PRIVATE_REPORT_RECONCILIATION_ID,
        report_id: &SEALED_AUDIT_PRIVATE_REPORT_ID,
        receipt_id: &SEALED_AUDIT_PRIVATE_REPORT_RECEIPT_ID,
        unavailable_evidence_digest: &SEALED_AUDIT_UNAVAILABLE_EVIDENCE_DIGEST,
        relay_observation_digest: &SEALED_AUDIT_RELAY_OBSERVATION_DIGEST,
        gateway_error_digest: &SEALED_AUDIT_GATEWAY_ERROR_DIGEST,
        target_absence_digest: &SEALED_AUDIT_TARGET_ABSENCE_DIGEST,
        retry_exhaustion_digest: &SEALED_AUDIT_RETRY_EXHAUSTION_DIGEST,
        rate_limit_state_digest: &SEALED_AUDIT_RATE_LIMIT_STATE_DIGEST,
        gateway_key_state_digest: &SEALED_AUDIT_GATEWAY_KEY_STATE_DIGEST,
        accountability_route_digest: &SEALED_AUDIT_ACCOUNTABILITY_ROUTE_DIGEST,
        blinded_failure_bucket_digest: &SEALED_AUDIT_FAILURE_BUCKET_DIGEST,
        monitor_submission_digest: &SEALED_AUDIT_MONITOR_SUBMISSION_PROOF_DIGEST,
        audit_checkpoint_digest: &SEALED_AUDIT_REPORT_AUDIT_CHECKPOINT_DIGEST,
        evidence_sequence: 1,
        previous_evidence_sequence: 0,
        reconciliation_sequence: reconciliation_decision.reconciliation_sequence,
        report_sequence: reconciliation_decision.report_sequence,
        receipt_sequence: reconciliation_decision.receipt_sequence,
        policy_epoch: reconciliation_decision.policy_epoch,
        proof_cache_log_index: reconciliation_decision.proof_cache_log_index,
        latest_checked_log_index: reconciliation_decision.latest_checked_log_index,
        created_at_s: 1_769_991_090,
        expires_at_s: 1_769_994_000,
        retry_attempt_count: 3,
        max_retry_attempts: 3,
        gateway_status_code: 503,
        reconciliation_bound: true,
        unavailable_evidence_gateway_authenticated: true,
        relay_observation_signed: true,
        target_absence_proof_bound: true,
        gateway_timeout_or_5xx_classified: true,
        no_client_asserted_unavailability: true,
        retry_exhaustion_bound: true,
        rate_limit_continuity_bound: true,
        gateway_key_state_bound: true,
        accountability_route_bound: true,
        operator_escalation_bound: true,
        blinded_failure_bucket_only: true,
        monitor_route_private: true,
        incident_visible_only_after_policy: true,
        evidence_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn accepted_sealed_audit_private_report_transport_decision()
-> SealedAuditPrivateReportTransportDecision {
    valid_sealed_audit_private_report_transport_input().evaluate()
}

const fn rejected_sealed_audit_private_report_transport_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditPrivateReportTransportDecision {
    SealedAuditPrivateReportTransportDecision {
        accepted: false,
        reason: mercury_core::SealedAuditPrivateReportTransportReason::PlaintextMetadataForbidden,
        can_submit_private_report: false,
        can_retry_safely: false,
        requires_private_transport: true,
        requires_replay_guard: false,
        requires_rate_limit_token: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
    }
}

fn valid_sealed_audit_recovery_export_write() -> SealedAuditRecoveryExportWrite<'static> {
    SealedAuditRecoveryExportWrite {
        incident_evidence_decision: accepted_sealed_audit_incident_evidence_decision(),
        export_format_version: 1,
        export_manifest_digest: &SEALED_AUDIT_RECOVERY_EXPORT_MANIFEST_DIGEST,
        previous_export_manifest_digest: &[],
        device_set_digest: &SEALED_AUDIT_DEVICE_SET_DIGEST,
        recovery_policy_digest: &SEALED_AUDIT_RECOVERY_POLICY_DIGEST,
        verifier_policy_digest: &SEALED_AUDIT_VERIFIER_POLICY_DIGEST,
        proof_cache_digest: &SEALED_AUDIT_PROOF_CACHE_DIGEST,
        incident_id: &SEALED_AUDIT_INCIDENT_ID,
        incident_evidence_digest: &SEALED_AUDIT_CONTRADICTION_DIGEST,
        export_ciphertext_digest: &SEALED_AUDIT_EXPORT_CIPHERTEXT_DIGEST,
        restore_authorization_digest: &SEALED_AUDIT_RESTORE_AUTHORIZATION_DIGEST,
        sync_state_digest: &SEALED_AUDIT_SYNC_STATE_DIGEST,
        audit_log_checkpoint_digest: &SEALED_AUDIT_PROOF_CHECKPOINT_DIGEST,
        export_sequence: 1,
        previous_export_sequence: 0,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        created_at_s: 1_769_991_000,
        expires_at_s: 1_769_994_000,
        restored_at_s: 1_769_991_050,
        device_count: 3,
        device_quorum_threshold: 2,
        approving_device_count: 2,
        recovery_share_count: 3,
        recovery_share_threshold: 2,
        manifest_signature_verified: true,
        device_binding_verified: true,
        recovery_policy_verified: true,
        export_ciphertext_encrypted: true,
        export_ciphertext_authenticated: true,
        restore_authorization_verified: true,
        restore_quorum_met: true,
        rollback_guard_verified: true,
        previous_export_bound: false,
        cross_device_sync_private: true,
        incident_selectors_redacted: true,
        audit_log_checkpoint_verified: true,
        store_record_encrypted: true,
        append_only_guard: true,
        plaintext_selector_count: 0,
        plaintext_metadata_fields: 0,
        ui_status_digest_only: true,
    }
}

fn accepted_sealed_audit_incident_evidence_decision() -> SealedAuditIncidentEvidenceDecision {
    let mut store = PrototypeSealedAuditIncidentEvidenceStore::default();
    put_sealed_audit_incident_evidence_record(
        &mut store,
        valid_sealed_audit_incident_evidence_write(),
    )
    .expect("prototype sealed audit incident evidence store cannot fail")
}

const fn rejected_sealed_audit_incident_evidence_decision(
    plaintext_bytes_exposed: bool,
) -> SealedAuditIncidentEvidenceDecision {
    SealedAuditIncidentEvidenceDecision {
        accepted: false,
        reason: SealedAuditIncidentEvidenceReason::VerifierPolicyRejected,
        persisted_record: false,
        record_count: 0,
        policy_epoch: 7,
        proof_cache_log_index: 42,
        latest_checked_log_index: 45,
        can_escalate_incident: false,
        can_report_privately: false,
        can_show_ui_status: false,
        requires_missing_proof_report: false,
        requires_split_view_escalation: false,
        requires_operator_accountability: false,
        requires_retry_backoff: false,
        suppressed_by_authenticated_policy: false,
        keeps_digest_only: true,
        plaintext_bytes_exposed,
    }
}

fn accepted_sealed_audit_event_store_decision() -> SealedAuditEventStoreDecision {
    let mut store = PrototypeSealedAuditEventStore::default();
    put_sealed_audit_event_record(&mut store, valid_sealed_audit_event_store_write())
        .expect("prototype sealed audit event store cannot fail")
}

const fn rejected_sealed_audit_event_store_decision() -> SealedAuditEventStoreDecision {
    SealedAuditEventStoreDecision {
        accepted: false,
        reason: SealedAuditEventStoreReason::ChainRejected,
        persisted_record: false,
        record_count: 0,
        event_sequence: 42,
        can_publish_receipt: false,
        can_detect_replay: true,
        append_only: true,
        keeps_digest_only: true,
        keeps_plaintext_metadata: false,
        plaintext_bytes_exposed: false,
    }
}

fn valid_sealed_audit_event_store_write() -> SealedAuditEventStoreWrite<'static> {
    SealedAuditEventStoreWrite {
        chain_decision: accepted_sealed_audit_chain_decision_for_sequence(42),
        event_sequence: 42,
        event_hash: &SEALED_AUDIT_EVENT_HASH,
        previous_event_hash: &SEALED_AUDIT_PREVIOUS_EVENT_HASH,
        record_digest: &SEALED_AUDIT_RECORD_DIGEST,
        merkle_root_hash: &SEALED_AUDIT_MERKLE_ROOT_HASH,
        checkpoint_id: &SEALED_AUDIT_CHECKPOINT_ID,
        checkpoint_signature: &SEALED_AUDIT_CHECKPOINT_SIGNATURE,
        transparency_receipt: &SEALED_AUDIT_TRANSPARENCY_RECEIPT,
        witness_receipt: &SEALED_AUDIT_WITNESS_RECEIPT,
        event_kind: SealedAuditEventKind::MlsCommit,
        anchor_kind: SealedAuditAnchorKind::WitnessedTransparencyLog,
        sealed_payload_len: 512,
        plaintext_metadata_fields: 0,
        append_only_guard: true,
        checkpoint_binds_chain: true,
        receipt_binds_checkpoint: true,
    }
}

fn accepted_sealed_audit_chain_decision_for_sequence(
    sequence: i64,
) -> SealedAuditEventChainDecision {
    let mut input = valid_sealed_audit_event_chain_input();
    input.event_sequence = sequence;
    input.previous_chain_size = sequence;
    input.previous_checkpoint_size = sequence;
    input.checkpoint_size = sequence + 1;
    evaluate_sealed_audit_event_chain(input)
}

fn rejected_sealed_audit_chain_decision() -> SealedAuditEventChainDecision {
    let mut input = valid_sealed_audit_event_chain_input();
    input.storage_append_only = false;
    evaluate_sealed_audit_event_chain(input)
}

fn group_chat_mls_ready_fixture() -> Value {
    group_chat_fixture("group_chat_mls_ready", valid_group_chat_input())
}

fn group_chat_mls_setup_required_fixture() -> Value {
    let mut input = valid_group_chat_input();
    input.mls_provider_configured = false;
    let mut provider_security =
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768);
    provider_security.provider_configured = false;
    input.mls_provider_security = provider_security.evaluate();
    group_chat_fixture("group_chat_mls_setup_required", input)
}

fn group_chat_membership_sync_required_fixture() -> Value {
    let mut input = valid_group_chat_input();
    input.membership_transition_pending = true;
    group_chat_fixture("group_chat_membership_sync_required", input)
}

fn group_chat_plaintext_metadata_forbidden_fixture() -> Value {
    let mut input = valid_group_chat_input();
    input.plaintext_member_metadata_fields = 1;
    group_chat_fixture("group_chat_plaintext_metadata_forbidden", input)
}

fn group_chat_high_security_mls_required_fixture() -> Value {
    let mut input = valid_group_chat_input();
    input.room_mode = mercury_core::RoomMode::HighSecurity;
    input.protocol = GroupChatProtocol::TransitionalPairwiseFanout;
    input.mls_provider_configured = false;
    let mut provider_security =
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768);
    provider_security.provider_configured = false;
    input.mls_provider_security = provider_security.evaluate();
    group_chat_fixture("group_chat_high_security_mls_required", input)
}

fn group_chat_high_security_pq_required_fixture() -> Value {
    let mut input = valid_group_chat_input();
    input.room_mode = mercury_core::RoomMode::HighSecurity;
    input.crypto_suite = GroupChatCryptoSuite::HybridPqMls768;
    group_chat_fixture("group_chat_high_security_pq_required", input)
}

fn group_chat_mls_provider_security_required_fixture() -> Value {
    let mut input = valid_group_chat_input();
    let mut provider_security =
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768);
    provider_security.known_answer_tests_passed = false;
    input.mls_provider_security = provider_security.evaluate();
    group_chat_fixture("group_chat_mls_provider_security_required", input)
}

fn group_chat_fixture(name: &'static str, input: GroupChatInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "group_chat",
        "input": {
            "protocol_code": input.protocol.code(),
            "protocol_label": input.protocol.label(),
            "crypto_suite_code": input.crypto_suite.code(),
            "crypto_suite_label": input.crypto_suite.label(),
            "room_mode_code": input.room_mode.code(),
            "member_count": input.member_count,
            "active_member_devices": input.active_member_devices,
            "local_device_is_member": input.local_device_is_member,
            "room_state_available": input.room_state_available,
            "group_secret_sealed": input.group_secret_sealed,
            "membership_transition_pending": input.membership_transition_pending,
            "current_epoch": input.current_epoch,
            "local_epoch": input.local_epoch,
            "key_transparency_ready": input.key_transparency_ready,
            "mls_provider_configured": input.mls_provider_configured,
            "mls_provider_security_accepted": input.mls_provider_security.accepted,
            "mls_provider_security_reason_label": input.mls_provider_security.reason.label(),
            "mls_provider_security_requires_mls_setup": input.mls_provider_security.requires_mls_setup,
            "mls_provider_security_requires_pq_upgrade": input.mls_provider_security.requires_pq_upgrade,
            "mls_provider_security_requires_user_action": input.mls_provider_security.requires_user_action,
            "plaintext_member_metadata_fields": input.plaintext_member_metadata_fields,
        },
        "decision": group_chat_decision_value(decision),
    })
}

fn group_chat_decision_value(decision: GroupChatDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "protocol_code": decision.protocol.code(),
        "protocol_label": decision.protocol.label(),
        "crypto_suite_code": decision.crypto_suite.code(),
        "crypto_suite_label": decision.crypto_suite.label(),
        "can_open_group": decision.can_open_group,
        "can_send_message": decision.can_send_message,
        "can_change_membership": decision.can_change_membership,
        "requires_sync": decision.requires_sync,
        "requires_mls_setup": decision.requires_mls_setup,
        "requires_pq_upgrade": decision.requires_pq_upgrade,
        "requires_user_action": decision.requires_user_action,
        "forbids_server_plaintext": decision.forbids_server_plaintext,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn group_message_transcript_ready_fixture() -> Value {
    group_message_transcript_fixture(
        "group_message_transcript_ready",
        valid_group_message_transcript_input(),
    )
}

fn group_message_transcript_sync_required_fixture() -> Value {
    let mut input = valid_group_message_transcript_input();
    input.message_epoch = input.local_epoch - 1;
    group_message_transcript_fixture("group_message_transcript_sync_required", input)
}

fn group_message_transcript_rekey_required_fixture() -> Value {
    let mut input = valid_group_message_transcript_input();
    input.sender_data_sealed = false;
    group_message_transcript_fixture("group_message_transcript_rekey_required", input)
}

fn group_message_transcript_store_binding_rejected_fixture() -> Value {
    let mut input = valid_group_message_transcript_input();
    input.local_store_seal = group_message_transcript_seal_request(6, 32);
    group_message_transcript_fixture("group_message_transcript_store_binding_rejected", input)
}

fn group_message_transcript_fixture(
    name: &'static str,
    input: GroupMessageTranscriptInput<'static>,
) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "group_message_transcript",
        "input": {
            "group_chat_accepted": input.group_chat.accepted,
            "group_chat_reason_label": input.group_chat.reason.label(),
            "group_chat_protocol_label": input.group_chat.protocol.label(),
            "group_chat_crypto_suite_label": input.group_chat.crypto_suite.label(),
            "outbound_send_accepted": input.outbound_send.accepted,
            "outbound_send_reason_label": input.outbound_send.reason.label(),
            "outbound_can_send": input.outbound_send.can_send,
            "outbound_can_persist_ciphertext": input.outbound_send.can_persist_ciphertext,
            "local_store_record_kind": input.local_store_seal.record_kind.label(),
            "local_store_key_scope": format!("{:?}", input.local_store_seal.key.scope),
            "local_store_room_epoch": input.local_store_seal.key.binding.room_epoch,
            "local_store_group_id_len": input.local_store_seal.key.binding.conversation_id_len,
            "group_id_len": input.group_id_len,
            "message_epoch": input.message_epoch,
            "local_epoch": input.local_epoch,
            "sender_leaf_index": input.sender_leaf_index,
            "sender_generation": input.sender_generation,
            "group_context_digest_len": input.group_context_digest_len,
            "confirmed_transcript_hash_len": input.confirmed_transcript_hash_len,
            "sender_data_sealed": input.sender_data_sealed,
            "application_payload_sealed": input.application_payload_sealed,
            "reuse_guard_len": input.reuse_guard_len,
            "used_generation_deleted": input.used_generation_deleted,
        },
        "decision": group_message_transcript_decision_value(decision),
    })
}

fn group_message_transcript_decision_value(decision: GroupMessageTranscriptDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_persist_ciphertext": decision.can_persist_ciphertext,
        "can_submit_to_relay": decision.can_submit_to_relay,
        "requires_sync": decision.requires_sync,
        "requires_rekey": decision.requires_rekey,
        "requires_user_action": decision.requires_user_action,
        "forbids_plaintext": decision.forbids_plaintext,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn anonymous_credential_issuer_trust_ready_fixture() -> Value {
    anonymous_credential_issuer_trust_fixture(
        "anonymous_credential_issuer_trust_ready",
        valid_anonymous_credential_issuer_trust_input(),
    )
}

fn anonymous_credential_issuer_trust_transparency_required_fixture() -> Value {
    let mut input = valid_anonymous_credential_issuer_trust_input();
    input.key_transparency = evaluate_key_transparency(KeyTransparencyProofInput {
        inclusion: KeyTransparencyProofStatus::Missing,
        ..valid_key_transparency_proof_input()
    });
    anonymous_credential_issuer_trust_fixture(
        "anonymous_credential_issuer_trust_transparency_required",
        input,
    )
}

fn anonymous_credential_issuer_trust_revoked_fixture() -> Value {
    let mut input = valid_anonymous_credential_issuer_trust_input();
    input.issuer_key_revoked = true;
    anonymous_credential_issuer_trust_fixture("anonymous_credential_issuer_trust_revoked", input)
}

fn anonymous_credential_issuer_trust_partitioning_metadata_rejected_fixture() -> Value {
    let mut input = valid_anonymous_credential_issuer_trust_input();
    input.opaque_partitioning_metadata_bits = 1;
    anonymous_credential_issuer_trust_fixture(
        "anonymous_credential_issuer_trust_partitioning_metadata_rejected",
        input,
    )
}

fn anonymous_credential_issuer_trust_witness_audit_rejected_fixture() -> Value {
    let mut audit_input = valid_anonymous_issuer_witness_audit_input();
    audit_input.split_view_reports = 1;
    let mut input = valid_anonymous_credential_issuer_trust_input();
    input.issuer_witness_audit = audit_input.evaluate();
    anonymous_credential_issuer_trust_fixture(
        "anonymous_credential_issuer_trust_witness_audit_rejected",
        input,
    )
}

fn anonymous_credential_issuer_trust_fixture(
    name: &'static str,
    input: AnonymousCredentialIssuerTrustInput,
) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "anonymous_credential_issuer_trust",
        "input": {
            "key_transparency_state": format!("{:?}", input.key_transparency.state),
            "key_transparency_reason": format!("{:?}", input.key_transparency.reason),
            "issuer_witness_audit_accepted": input.issuer_witness_audit.accepted,
            "issuer_witness_audit_reason_label": input.issuer_witness_audit.reason.label(),
            "issuer_witness_audit_requires_sync": input.issuer_witness_audit.requires_sync,
            "issuer_witness_audit_requires_rekey": input.issuer_witness_audit.requires_rekey,
            "issuer_witness_audit_requires_user_action": input.issuer_witness_audit.requires_user_action,
            "issuer_key_id_len": input.issuer_key_id_len,
            "issuer_directory_inclusion_verified": input.issuer_directory_inclusion_verified,
            "issuer_key_bound_to_challenge": input.issuer_key_bound_to_challenge,
            "active_issuer_key_count": input.active_issuer_key_count,
            "max_active_issuer_key_count": input.max_active_issuer_key_count,
            "directory_age_s": input.directory_age_s,
            "max_directory_age_s": input.max_directory_age_s,
            "key_not_before_s": input.key_not_before_s,
            "key_not_after_s": input.key_not_after_s,
            "now_s": input.now_s,
            "revocation_status_fresh": input.revocation_status_fresh,
            "issuer_key_revoked": input.issuer_key_revoked,
            "opaque_partitioning_metadata_bits": input.opaque_partitioning_metadata_bits,
        },
        "decision": anonymous_credential_issuer_trust_decision_value(decision),
    })
}

fn anonymous_credential_issuer_trust_decision_value(
    decision: AnonymousCredentialIssuerTrustDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_issue_or_verify_tokens": decision.can_issue_or_verify_tokens,
        "can_use_for_anonymous_membership_proof": decision.can_use_for_anonymous_membership_proof,
        "requires_sync": decision.requires_sync,
        "requires_rekey": decision.requires_rekey,
        "requires_user_action": decision.requires_user_action,
        "protects_anonymity_set": decision.protects_anonymity_set,
    })
}

fn anonymous_group_membership_proof_ready_fixture() -> Value {
    anonymous_group_membership_proof_fixture(
        "anonymous_group_membership_proof_ready",
        valid_anonymous_group_membership_proof_input(),
    )
}

fn anonymous_group_membership_proof_high_security_pq_required_fixture() -> Value {
    let mut input = valid_anonymous_group_membership_proof_input();
    input.scheme = AnonymousGroupMembershipProofScheme::BbsUnlinkablePresentation;
    input.scheme_post_quantum_safe = false;
    anonymous_group_membership_proof_fixture(
        "anonymous_group_membership_proof_high_security_pq_required",
        input,
    )
}

fn anonymous_group_membership_proof_replay_rejected_fixture() -> Value {
    let mut input = valid_anonymous_group_membership_proof_input();
    input.replay_nullifier_seen = true;
    anonymous_group_membership_proof_fixture(
        "anonymous_group_membership_proof_replay_rejected",
        input,
    )
}

fn anonymous_group_membership_proof_route_binding_required_fixture() -> Value {
    let mut input = valid_anonymous_group_membership_proof_input();
    input.route_bound = false;
    anonymous_group_membership_proof_fixture(
        "anonymous_group_membership_proof_route_binding_required",
        input,
    )
}

fn anonymous_group_membership_proof_plaintext_identity_rejected_fixture() -> Value {
    let mut input = valid_anonymous_group_membership_proof_input();
    input.plaintext_member_identifier_fields = 1;
    anonymous_group_membership_proof_fixture(
        "anonymous_group_membership_proof_plaintext_identity_rejected",
        input,
    )
}

fn anonymous_group_membership_proof_fixture(
    name: &'static str,
    input: AnonymousGroupMembershipProofInput,
) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "anonymous_group_membership_proof",
        "input": {
            "group_chat_accepted": input.group_chat.accepted,
            "group_chat_reason_label": input.group_chat.reason.label(),
            "group_chat_crypto_suite_label": input.group_chat.crypto_suite.label(),
            "scheme_code": input.scheme.code(),
            "scheme_label": input.scheme.label(),
            "issuer_trust_accepted": input.issuer_trust.accepted,
            "issuer_trust_reason_label": input.issuer_trust.reason.label(),
            "issuer_trust_requires_sync": input.issuer_trust.requires_sync,
            "issuer_trust_requires_rekey": input.issuer_trust.requires_rekey,
            "issuer_trust_requires_user_action": input.issuer_trust.requires_user_action,
            "high_security_room": input.high_security_room,
            "scheme_post_quantum_safe": input.scheme_post_quantum_safe,
            "issuer_key_id_len": input.issuer_key_id_len,
            "challenge_digest_len": input.challenge_digest_len,
            "presentation_nonce_len": input.presentation_nonce_len,
            "proof_len": input.proof_len,
            "presentation_header_bound": input.presentation_header_bound,
            "group_epoch_bound": input.group_epoch_bound,
            "route_bound": input.route_bound,
            "replay_nullifier_len": input.replay_nullifier_len,
            "replay_nullifier_seen": input.replay_nullifier_seen,
            "issued_at_s": input.issued_at_s,
            "expires_at_s": input.expires_at_s,
            "now_s": input.now_s,
            "plaintext_member_identifier_fields": input.plaintext_member_identifier_fields,
        },
        "decision": anonymous_group_membership_proof_decision_value(decision),
    })
}

fn anonymous_group_membership_proof_decision_value(
    decision: AnonymousGroupMembershipProofDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_authenticate_member": decision.can_authenticate_member,
        "can_redeem_once": decision.can_redeem_once,
        "can_rate_limit_anonymously": decision.can_rate_limit_anonymously,
        "requires_sync": decision.requires_sync,
        "requires_rekey": decision.requires_rekey,
        "requires_user_action": decision.requires_user_action,
        "forbids_plaintext_member_identity": decision.forbids_plaintext_member_identity,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn anonymous_rate_limit_nullifier_ready_fixture() -> Value {
    anonymous_rate_limit_nullifier_fixture(
        "anonymous_rate_limit_nullifier_ready",
        valid_anonymous_rate_limit_nullifier_input(),
    )
}

fn anonymous_rate_limit_nullifier_replay_rejected_fixture() -> Value {
    let mut input = valid_anonymous_rate_limit_nullifier_input();
    input.nullifier_already_spent = true;
    anonymous_rate_limit_nullifier_fixture("anonymous_rate_limit_nullifier_replay_rejected", input)
}

fn anonymous_rate_limit_nullifier_limit_exceeded_fixture() -> Value {
    let mut input = valid_anonymous_rate_limit_nullifier_input();
    input.presentation_count = input.presentation_limit;
    anonymous_rate_limit_nullifier_fixture("anonymous_rate_limit_nullifier_limit_exceeded", input)
}

fn anonymous_rate_limit_nullifier_opaque_store_required_fixture() -> Value {
    let mut input = valid_anonymous_rate_limit_nullifier_input();
    input.nullifier_store_opaque = false;
    anonymous_rate_limit_nullifier_fixture(
        "anonymous_rate_limit_nullifier_opaque_store_required",
        input,
    )
}

fn anonymous_rate_limit_nullifier_fixture(
    name: &'static str,
    input: AnonymousRateLimitNullifierInput,
) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "anonymous_rate_limit_nullifier",
        "input": {
            "membership_proof_accepted": input.membership_proof.accepted,
            "membership_proof_reason_label": input.membership_proof.reason.label(),
            "credential_kind_code": input.credential_kind.code(),
            "credential_kind_label": input.credential_kind.label(),
            "nullifier_len": input.nullifier_len,
            "nullifier_already_spent": input.nullifier_already_spent,
            "nullifier_store_available": input.nullifier_store_available,
            "nullifier_store_opaque": input.nullifier_store_opaque,
            "bound_to_route": input.bound_to_route,
            "bound_to_group_epoch": input.bound_to_group_epoch,
            "redemption_context_len": input.redemption_context_len,
            "credential_context_len": input.credential_context_len,
            "window_start_s": input.window_start_s,
            "window_end_s": input.window_end_s,
            "now_s": input.now_s,
            "presentation_count": input.presentation_count,
            "presentation_limit": input.presentation_limit,
            "max_presentation_limit": input.max_presentation_limit,
            "plaintext_rate_limit_fields": input.plaintext_rate_limit_fields,
        },
        "decision": anonymous_rate_limit_nullifier_decision_value(decision),
    })
}

fn anonymous_rate_limit_nullifier_decision_value(
    decision: AnonymousRateLimitNullifierDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_record_nullifier": decision.can_record_nullifier,
        "can_redeem_this_window": decision.can_redeem_this_window,
        "can_rate_limit_without_identity": decision.can_rate_limit_without_identity,
        "requires_sync": decision.requires_sync,
        "requires_rekey": decision.requires_rekey,
        "requires_user_action": decision.requires_user_action,
        "forbids_plaintext_rate_limit_metadata": decision.forbids_plaintext_rate_limit_metadata,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

const ANONYMOUS_NULLIFIER_BYTES: [u8; 32] = [0xA7; 32];
const ANONYMOUS_NULLIFIER_REDEMPTION_CONTEXT_DIGEST: [u8; 32] = [0xC9; 32];
const ANONYMOUS_NULLIFIER_CREDENTIAL_CONTEXT_DIGEST: [u8; 32] = [0xDA; 32];
const MLS_PROVIDER_EVIDENCE_ID: [u8; 32] = [0x31; 32];
const MLS_PROVIDER_ID_DIGEST: [u8; 32] = [0x41; 32];
const MLS_PROVIDER_SUITE_EVIDENCE_DIGEST: [u8; 32] = [0x42; 32];
const MLS_PROVIDER_KAT_EVIDENCE_DIGEST: [u8; 32] = [0x43; 32];
const MLS_PROVIDER_DOWNGRADE_EVIDENCE_DIGEST: [u8; 32] = [0x44; 32];
const MLS_PROVIDER_ZEROIZATION_EVIDENCE_DIGEST: [u8; 32] = [0x45; 32];
const MLS_KEY_PACKAGE_GROUP_ID: [u8; 32] = [0x71; 32];
const MLS_KEY_PACKAGE_OTHER_GROUP_ID: [u8; 32] = [0x72; 32];
const MLS_KEY_PACKAGE_HASH: [u8; 32] = [0x73; 32];
const MLS_KEY_PACKAGE_ADDED_MEMBER_REF: [u8; 32] = [0x75; 32];
const MLS_KEY_PACKAGE_WELCOME_SEND_TRANSACTION_DIGEST: [u8; 32] = [0x76; 32];
const MLS_KEY_PACKAGE_SHORT_DIGEST: [u8; 16] = [0x77; 16];
const MLS_WELCOME_SEND_GROUP_ID: [u8; 32] = [0x81; 32];
const MLS_WELCOME_SEND_KEY_PACKAGE_HASH: [u8; 32] = [0x82; 32];
const MLS_WELCOME_SEND_ADDED_MEMBER_REF: [u8; 32] = [0x84; 32];
const MLS_WELCOME_SEND_TRANSACTION_DIGEST: [u8; 32] = [0x85; 32];
const MLS_WELCOME_SEND_OTHER_TRANSACTION_DIGEST: [u8; 32] = [0x86; 32];
const MLS_WELCOME_SEND_COMMIT_HASH: [u8; 32] = [0x87; 32];
const MLS_WELCOME_SEND_CIPHERTEXT_HASH: [u8; 32] = [0x88; 32];
const MLS_WELCOME_SEND_DELIVERY_ROUTE_ID: [u8; 32] = [0x89; 32];
const MLS_WELCOME_SEND_REPLAY_TOKEN: [u8; 32] = [0x8A; 32];
const MLS_WELCOME_SEND_SHORT_DIGEST: [u8; 16] = [0x8B; 16];
const MLS_MEMBERSHIP_TRANSACTION_GROUP_ID: [u8; 32] = [0x91; 32];
const MLS_MEMBERSHIP_TRANSACTION_OTHER_GROUP_ID: [u8; 32] = [0x92; 32];
const MLS_MEMBERSHIP_TRANSACTION_COMMIT_HASH: [u8; 32] = [0x93; 32];
const MLS_MEMBERSHIP_TRANSACTION_KEY_PACKAGE_HASH: [u8; 32] = [0x94; 32];
const MLS_MEMBERSHIP_TRANSACTION_WELCOME_SEND_DIGEST: [u8; 32] = [0x95; 32];
const MLS_MEMBERSHIP_TRANSACTION_DIGEST: [u8; 32] = [0x96; 32];
const MLS_MEMBERSHIP_TRANSACTION_ADDED_MEMBER_REF: [u8; 32] = [0x99; 32];
const MLS_MEMBERSHIP_TRANSACTION_WELCOME_CIPHERTEXT_HASH: [u8; 32] = [0x9A; 32];
const MLS_MEMBERSHIP_TRANSACTION_DELIVERY_ROUTE_ID: [u8; 32] = [0x9B; 32];
const MLS_MEMBERSHIP_TRANSACTION_REPLAY_TOKEN: [u8; 32] = [0x9C; 32];
const MLS_COMMIT_GROUP_ID: [u8; 32] = [0x51; 32];
const MLS_COMMIT_HASH: [u8; 32] = [0x52; 32];
const MLS_WELCOME_GROUP_ID: [u8; 32] = [0x61; 32];
const MLS_WELCOME_HASH: [u8; 32] = [0x62; 32];
const MLS_WELCOME_OTHER_HASH: [u8; 32] = [0x63; 32];
const MLS_WELCOME_KEY_PACKAGE_REF: [u8; 32] = [0x64; 32];
const MLS_WELCOME_TREE_HASH: [u8; 32] = [0x66; 32];
const MLS_WELCOME_CONFIRMED_TRANSCRIPT_HASH: [u8; 32] = [0x67; 32];
const MLS_WELCOME_GROUP_STATE_COMMIT_DIGEST: [u8; 32] = [0x68; 32];
const MLS_WELCOME_SHORT_DIGEST: [u8; 16] = [0x69; 16];

fn anonymous_nullifier_store_ready_fixture() -> Value {
    anonymous_nullifier_store_fixture(
        "anonymous_nullifier_store_ready",
        valid_anonymous_nullifier_store_write(),
        false,
    )
}

fn anonymous_nullifier_store_replay_rejected_fixture() -> Value {
    anonymous_nullifier_store_fixture(
        "anonymous_nullifier_store_replay_rejected",
        valid_anonymous_nullifier_store_write(),
        true,
    )
}

fn anonymous_nullifier_store_plaintext_metadata_rejected_fixture() -> Value {
    let mut write = valid_anonymous_nullifier_store_write();
    write.plaintext_metadata_fields = 1;
    anonymous_nullifier_store_fixture(
        "anonymous_nullifier_store_plaintext_metadata_rejected",
        write,
        false,
    )
}

fn anonymous_nullifier_store_fixture(
    name: &'static str,
    write: AnonymousNullifierStoreWrite<'static>,
    seed_replay: bool,
) -> Value {
    let mut store = PrototypeAnonymousNullifierStore::default();
    if seed_replay {
        let _ = put_anonymous_nullifier_record(&mut store, valid_anonymous_nullifier_store_write())
            .expect("prototype nullifier store cannot fail");
    }
    let decision = put_anonymous_nullifier_record(&mut store, write)
        .expect("prototype nullifier store cannot fail");

    json!({
        "fixture": name,
        "surface": "anonymous_nullifier_store",
        "input": {
            "nullifier_len": write.nullifier.len(),
            "redemption_context_digest_len": write.redemption_context_digest.len(),
            "credential_context_digest_len": write.credential_context_digest.len(),
            "credential_kind_code": write.credential_kind.code(),
            "credential_kind_label": write.credential_kind.label(),
            "nullifier_decision_accepted": write.nullifier_decision.accepted,
            "nullifier_decision_reason_label": write.nullifier_decision.reason.label(),
            "window_start_s": write.window_start_s,
            "window_end_s": write.window_end_s,
            "presentation_count_before": write.presentation_count_before,
            "presentation_limit": write.presentation_limit,
            "plaintext_metadata_fields": write.plaintext_metadata_fields,
            "seed_replay": seed_replay,
        },
        "decision": anonymous_nullifier_store_decision_value(decision),
        "store": {
            "record_count": store.len(),
            "has_record": store.get(write.nullifier).is_some(),
        },
    })
}

fn anonymous_nullifier_store_decision_value(decision: AnonymousNullifierStoreDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "presentation_count_after": decision.presentation_count_after,
        "record_count": decision.record_count,
        "keeps_context_digest_only": decision.keeps_context_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_provider_evidence_store_ready_fixture() -> Value {
    mls_provider_evidence_store_fixture(
        "mls_provider_evidence_store_ready",
        valid_mls_provider_evidence_store_write(),
        false,
    )
}

fn mls_provider_evidence_store_gate_rejected_fixture() -> Value {
    let mut input = valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768);
    input.known_answer_tests_passed = false;
    let mut write = valid_mls_provider_evidence_store_write();
    write.provider_security = evaluate_mls_provider_security(input);
    mls_provider_evidence_store_fixture("mls_provider_evidence_store_gate_rejected", write, false)
}

fn mls_provider_evidence_store_duplicate_rejected_fixture() -> Value {
    mls_provider_evidence_store_fixture(
        "mls_provider_evidence_store_duplicate_rejected",
        valid_mls_provider_evidence_store_write(),
        true,
    )
}

fn mls_provider_evidence_store_plaintext_rejected_fixture() -> Value {
    let mut write = valid_mls_provider_evidence_store_write();
    write.plaintext_evidence_fields = 1;
    mls_provider_evidence_store_fixture(
        "mls_provider_evidence_store_plaintext_rejected",
        write,
        false,
    )
}

fn mls_provider_evidence_store_fixture(
    name: &'static str,
    write: MlsProviderEvidenceStoreWrite<'static>,
    seed_duplicate: bool,
) -> Value {
    let mut store = PrototypeMlsProviderEvidenceStore::default();
    if seed_duplicate {
        let _ =
            put_mls_provider_evidence_record(&mut store, valid_mls_provider_evidence_store_write())
                .expect("prototype MLS provider evidence store cannot fail");
    }
    let decision = put_mls_provider_evidence_record(&mut store, write)
        .expect("prototype MLS provider evidence store cannot fail");

    json!({
        "fixture": name,
        "surface": "mls_provider_evidence_store",
        "input": {
            "evidence_id_len": write.evidence_id.len(),
            "provider_id_digest_len": write.provider_id_digest.len(),
            "suite_evidence_digest_len": write.suite_evidence_digest.len(),
            "kat_evidence_digest_len": write.kat_evidence_digest.len(),
            "downgrade_evidence_digest_len": write.downgrade_evidence_digest.len(),
            "zeroization_evidence_digest_len": write.zeroization_evidence_digest.len(),
            "provider_security_accepted": write.provider_security.accepted,
            "provider_security_reason_code": write.provider_security.reason.code(),
            "provider_security_reason_label": write.provider_security.reason.label(),
            "provider_security_suite_code": write.provider_security.suite.code(),
            "provider_security_suite_label": write.provider_security.suite.label(),
            "validated_at_s": write.validated_at_s,
            "expires_at_s": write.expires_at_s,
            "plaintext_evidence_fields": write.plaintext_evidence_fields,
            "seed_duplicate": seed_duplicate,
        },
        "decision": mls_provider_evidence_store_decision_value(decision),
        "store": {
            "record_count": store.len(),
            "has_record": store.get(write.evidence_id).is_some(),
        },
    })
}

fn mls_provider_evidence_store_decision_value(decision: MlsProviderEvidenceStoreDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "can_use_as_provider_evidence": decision.can_use_as_provider_evidence,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_provider_evidence_use_ready_fixture() -> Value {
    mls_provider_evidence_use_fixture(
        "mls_provider_evidence_use_ready",
        Some(stored_mls_provider_evidence_record()),
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768).evaluate(),
        GroupChatCryptoSuite::HybridPqMls768,
        1_100,
    )
}

fn mls_provider_evidence_use_missing_fixture() -> Value {
    mls_provider_evidence_use_fixture(
        "mls_provider_evidence_use_missing",
        None,
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768).evaluate(),
        GroupChatCryptoSuite::HybridPqMls768,
        1_100,
    )
}

fn mls_provider_evidence_use_expired_fixture() -> Value {
    mls_provider_evidence_use_fixture(
        "mls_provider_evidence_use_expired",
        Some(stored_mls_provider_evidence_record()),
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768).evaluate(),
        GroupChatCryptoSuite::HybridPqMls768,
        1_300,
    )
}

fn mls_provider_evidence_use_suite_mismatch_fixture() -> Value {
    mls_provider_evidence_use_fixture(
        "mls_provider_evidence_use_suite_mismatch",
        Some(stored_mls_provider_evidence_record()),
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768).evaluate(),
        GroupChatCryptoSuite::HybridPqMls1024,
        1_100,
    )
}

fn mls_provider_evidence_use_plaintext_rejected_fixture() -> Value {
    let mut record = stored_mls_provider_evidence_record();
    record.plaintext_bytes_exposed = true;
    mls_provider_evidence_use_fixture(
        "mls_provider_evidence_use_plaintext_rejected",
        Some(record),
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768).evaluate(),
        GroupChatCryptoSuite::HybridPqMls768,
        1_100,
    )
}

fn mls_provider_evidence_use_fixture(
    name: &'static str,
    record: Option<MlsProviderEvidenceStoreRecord>,
    provider_security: mercury_core::MlsProviderSecurityDecision,
    required_suite: GroupChatCryptoSuite,
    now_s: i64,
) -> Value {
    let input = MlsProviderEvidenceUseInput {
        record: record.as_ref(),
        provider_security,
        required_suite,
        now_s,
    };
    let decision = evaluate_mls_provider_evidence_use(input);

    json!({
        "fixture": name,
        "surface": "mls_provider_evidence_use",
        "input": {
            "record_present": record.is_some(),
            "record_suite_code": record.as_ref().map(|record| record.suite.code()),
            "record_suite_label": record.as_ref().map(|record| record.suite.label()),
            "record_validated_at_s": record.as_ref().map(|record| record.validated_at_s),
            "record_expires_at_s": record.as_ref().map(|record| record.expires_at_s),
            "record_plaintext_bytes_exposed": record
                .as_ref()
                .map(|record| record.plaintext_bytes_exposed)
                .unwrap_or(false),
            "provider_security_accepted": provider_security.accepted,
            "provider_security_reason_code": provider_security.reason.code(),
            "provider_security_reason_label": provider_security.reason.label(),
            "provider_security_suite_code": provider_security.suite.code(),
            "provider_security_suite_label": provider_security.suite.label(),
            "required_suite_code": required_suite.code(),
            "required_suite_label": required_suite.label(),
            "now_s": now_s,
        },
        "decision": mls_provider_evidence_use_decision_value(decision),
    })
}

fn mls_provider_evidence_use_decision_value(decision: MlsProviderEvidenceUseDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_use_provider_evidence": decision.can_use_provider_evidence,
        "requires_provider_validation": decision.requires_provider_validation,
        "requires_pq_upgrade": decision.requires_pq_upgrade,
        "requires_user_action": decision.requires_user_action,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_provider_adapter_selection_ready_fixture() -> Value {
    mls_provider_adapter_selection_fixture(
        "mls_provider_adapter_selection_ready",
        valid_mls_provider_adapter_selection_input(),
    )
}

fn mls_provider_adapter_selection_provider_rejected_fixture() -> Value {
    let mut input = valid_mls_provider_adapter_selection_input();
    let mut provider_security =
        valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768);
    provider_security.known_answer_tests_passed = false;
    input.provider_security = provider_security.evaluate();
    mls_provider_adapter_selection_fixture(
        "mls_provider_adapter_selection_provider_rejected",
        input,
    )
}

fn mls_provider_adapter_selection_pq_draft_rejected_fixture() -> Value {
    let mut input = valid_mls_provider_adapter_selection_input();
    input.pq_draft_version_pinned = false;
    mls_provider_adapter_selection_fixture(
        "mls_provider_adapter_selection_pq_draft_rejected",
        input,
    )
}

fn mls_provider_adapter_selection_storage_rejected_fixture() -> Value {
    let mut input = valid_mls_provider_adapter_selection_input();
    input.storage_provider_transactional = false;
    mls_provider_adapter_selection_fixture("mls_provider_adapter_selection_storage_rejected", input)
}

fn mls_provider_adapter_selection_supply_chain_rejected_fixture() -> Value {
    let mut input = valid_mls_provider_adapter_selection_input();
    input.sbom_present = false;
    mls_provider_adapter_selection_fixture(
        "mls_provider_adapter_selection_supply_chain_rejected",
        input,
    )
}

fn mls_provider_adapter_selection_fixture(
    name: &'static str,
    input: MlsProviderAdapterSelectionInput,
) -> Value {
    json!({
        "fixture": name,
        "surface": "mls_provider_adapter_selection",
        "input": {
            "provider_security_accepted": input.provider_security.accepted,
            "provider_security_reason_code": input.provider_security.reason.code(),
            "provider_security_reason_label": input.provider_security.reason.label(),
            "provider_security_suite_code": input.provider_security.suite.code(),
            "provider_security_suite_label": input.provider_security.suite.label(),
            "adapter_kind_code": input.adapter_kind.code(),
            "adapter_kind_label": input.adapter_kind.label(),
            "crypto_backend_code": input.crypto_backend.code(),
            "crypto_backend_label": input.crypto_backend.label(),
            "protocol_profile_code": input.protocol_profile.code(),
            "protocol_profile_label": input.protocol_profile.label(),
            "license_kind_code": input.license_kind.code(),
            "license_kind_label": input.license_kind.label(),
            "source_verified": input.source_verified,
            "license_allows_distribution": input.license_allows_distribution,
            "rfc9420_conformance_tests_passed": input.rfc9420_conformance_tests_passed,
            "pq_draft_version_pinned": input.pq_draft_version_pinned,
            "ml_kem_standardized": input.ml_kem_standardized,
            "pq_signature_standardized_when_required": input.pq_signature_standardized_when_required,
            "kat_vectors_passed": input.kat_vectors_passed,
            "interop_tests_passed": input.interop_tests_passed,
            "storage_provider_seals_group_state": input.storage_provider_seals_group_state,
            "storage_provider_transactional": input.storage_provider_transactional,
            "secret_zeroization_audited": input.secret_zeroization_audited,
            "memory_hardening_enabled": input.memory_hardening_enabled,
            "downgrade_tests_passed": input.downgrade_tests_passed,
            "transcript_hash_binding_verified": input.transcript_hash_binding_verified,
            "unsafe_features_enabled": input.unsafe_features_enabled,
            "plaintext_export_enabled": input.plaintext_export_enabled,
            "release_artifact_signed": input.release_artifact_signed,
            "sbom_present": input.sbom_present,
            "cve_monitoring_enabled": input.cve_monitoring_enabled,
        },
        "decision": mls_provider_adapter_selection_decision_value(
            evaluate_mls_provider_adapter_selection(input)
        ),
    })
}

fn mls_provider_adapter_selection_decision_value(
    decision: MlsProviderAdapterSelectionDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_link_provider": decision.can_link_provider,
        "can_open_mls_group": decision.can_open_mls_group,
        "can_change_membership": decision.can_change_membership,
        "can_ship_release": decision.can_ship_release,
        "requires_mls_setup": decision.requires_mls_setup,
        "requires_pq_upgrade": decision.requires_pq_upgrade,
        "requires_license_review": decision.requires_license_review,
        "requires_supply_chain_review": decision.requires_supply_chain_review,
        "requires_interop_review": decision.requires_interop_review,
        "requires_storage_review": decision.requires_storage_review,
        "forbids_plaintext_key_export": decision.forbids_plaintext_key_export,
        "adapter_kind_code": decision.adapter_kind_code,
        "adapter_kind_label": decision.adapter_kind_label,
        "crypto_backend_code": decision.crypto_backend_code,
        "crypto_backend_label": decision.crypto_backend_label,
        "protocol_profile_code": decision.protocol_profile_code,
        "protocol_profile_label": decision.protocol_profile_label,
        "license_kind_code": decision.license_kind_code,
        "license_kind_label": decision.license_kind_label,
        "suite_code": decision.suite_code,
        "suite_label": decision.suite_label,
        "provider_security_reason": decision.provider_security_reason.label(),
    })
}

fn stored_mls_provider_evidence_record() -> MlsProviderEvidenceStoreRecord {
    let mut store = PrototypeMlsProviderEvidenceStore::default();
    let decision =
        put_mls_provider_evidence_record(&mut store, valid_mls_provider_evidence_store_write())
            .expect("prototype MLS provider evidence store cannot fail");
    assert!(decision.accepted);
    store
        .get(&MLS_PROVIDER_EVIDENCE_ID)
        .expect("record should be written")
        .clone()
}

fn mls_key_package_admission_ready_fixture() -> Value {
    mls_key_package_admission_fixture(
        "mls_key_package_admission_ready",
        valid_mls_key_package_admission_input(),
    )
}

fn mls_key_package_admission_group_rejected_fixture() -> Value {
    let mut group = valid_group_chat_input();
    group.membership_transition_pending = true;
    let mut input = valid_mls_key_package_admission_input();
    input.group_chat = group.evaluate();
    mls_key_package_admission_fixture("mls_key_package_admission_group_rejected", input)
}

fn mls_key_package_admission_lifetime_rejected_fixture() -> Value {
    let mut input = valid_mls_key_package_admission_input();
    input.now_s = input.lifetime_not_after_s;
    mls_key_package_admission_fixture("mls_key_package_admission_lifetime_rejected", input)
}

fn mls_key_package_admission_suite_mismatch_fixture() -> Value {
    let mut input = valid_mls_key_package_admission_input();
    input.key_package_suite = GroupChatCryptoSuite::HybridPqMls1024;
    mls_key_package_admission_fixture("mls_key_package_admission_suite_mismatch", input)
}

fn mls_key_package_admission_credential_rejected_fixture() -> Value {
    let mut input = valid_mls_key_package_admission_input();
    input.credential_valid = false;
    mls_key_package_admission_fixture("mls_key_package_admission_credential_rejected", input)
}

fn mls_key_package_admission_replay_rejected_fixture() -> Value {
    let mut input = valid_mls_key_package_admission_input();
    input.key_package_hash_already_used = true;
    mls_key_package_admission_fixture("mls_key_package_admission_replay_rejected", input)
}

fn mls_key_package_admission_plaintext_rejected_fixture() -> Value {
    let mut input = valid_mls_key_package_admission_input();
    input.plaintext_identity_fields = 1;
    mls_key_package_admission_fixture("mls_key_package_admission_plaintext_rejected", input)
}

fn mls_key_package_admission_fixture(
    name: &'static str,
    input: MlsKeyPackageAdmissionInput,
) -> Value {
    let decision = evaluate_mls_key_package_admission(input);

    json!({
        "fixture": name,
        "surface": "mls_key_package_admission",
        "input": {
            "group_chat_accepted": input.group_chat.accepted,
            "group_chat_reason_code": input.group_chat.reason.code(),
            "group_chat_reason_label": input.group_chat.reason.label(),
            "group_chat_protocol_code": input.group_chat.protocol.code(),
            "group_chat_protocol_label": input.group_chat.protocol.label(),
            "group_chat_suite_code": input.group_chat.crypto_suite.code(),
            "group_chat_suite_label": input.group_chat.crypto_suite.label(),
            "group_protocol_version": input.group_protocol_version,
            "key_package_protocol_version": input.key_package_protocol_version,
            "group_suite_code": input.group_suite.code(),
            "group_suite_label": input.group_suite.label(),
            "key_package_suite_code": input.key_package_suite.code(),
            "key_package_suite_label": input.key_package_suite.label(),
            "leaf_node_valid": input.leaf_node_valid,
            "leaf_signature_valid": input.leaf_signature_valid,
            "key_package_signature_valid": input.key_package_signature_valid,
            "credential_valid": input.credential_valid,
            "required_capabilities_present": input.required_capabilities_present,
            "credential_supported_by_group": input.credential_supported_by_group,
            "lifetime_not_before_s": input.lifetime_not_before_s,
            "lifetime_not_after_s": input.lifetime_not_after_s,
            "now_s": input.now_s,
            "max_lifetime_s": input.max_lifetime_s,
            "leaf_source_key_package": input.leaf_source_key_package,
            "extensions_supported": input.extensions_supported,
            "encryption_key_reuses_init_key": input.encryption_key_reuses_init_key,
            "init_key_len": input.init_key_len,
            "key_package_hash_len": input.key_package_hash_len,
            "key_package_hash_already_used": input.key_package_hash_already_used,
            "plaintext_identity_fields": input.plaintext_identity_fields,
        },
        "decision": mls_key_package_admission_decision_value(decision),
    })
}

fn mls_key_package_admission_decision_value(decision: MlsKeyPackageAdmissionDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_add_member": decision.can_add_member,
        "can_send_welcome": decision.can_send_welcome,
        "requires_sync": decision.requires_sync,
        "requires_mls_setup": decision.requires_mls_setup,
        "requires_pq_upgrade": decision.requires_pq_upgrade,
        "requires_user_action": decision.requires_user_action,
        "prevents_key_reuse": decision.prevents_key_reuse,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_key_package_consume_store_ready_fixture() -> Value {
    mls_key_package_consume_store_fixture(
        "mls_key_package_consume_store_ready",
        valid_mls_key_package_consume_store_write(),
        false,
    )
}

fn mls_key_package_consume_store_admission_rejected_fixture() -> Value {
    let mut input = valid_mls_key_package_consume_store_write();
    let mut admission_input = valid_mls_key_package_admission_input();
    admission_input.key_package_hash_len = 31;
    input.key_package_admission = admission_input.evaluate();
    mls_key_package_consume_store_fixture(
        "mls_key_package_consume_store_admission_rejected",
        input,
        false,
    )
}

fn mls_key_package_consume_store_duplicate_rejected_fixture() -> Value {
    let mut input = valid_mls_key_package_consume_store_write();
    input.group_id = &MLS_KEY_PACKAGE_OTHER_GROUP_ID;
    mls_key_package_consume_store_fixture(
        "mls_key_package_consume_store_duplicate_rejected",
        input,
        true,
    )
}

fn mls_key_package_consume_store_bad_shape_fixture() -> Value {
    let mut input = valid_mls_key_package_consume_store_write();
    input.welcome_send_transaction_digest = &MLS_KEY_PACKAGE_SHORT_DIGEST;
    mls_key_package_consume_store_fixture("mls_key_package_consume_store_bad_shape", input, false)
}

fn mls_key_package_consume_store_plaintext_rejected_fixture() -> Value {
    let mut input = valid_mls_key_package_consume_store_write();
    input.plaintext_metadata_fields = 1;
    mls_key_package_consume_store_fixture(
        "mls_key_package_consume_store_plaintext_rejected",
        input,
        false,
    )
}

fn mls_key_package_consume_store_fixture(
    name: &'static str,
    write: MlsKeyPackageConsumeStoreWrite<'static>,
    seed_duplicate: bool,
) -> Value {
    let mut store = PrototypeMlsKeyPackageConsumeStore::default();
    if seed_duplicate {
        let _ = put_mls_key_package_consumption_record(
            &mut store,
            valid_mls_key_package_consume_store_write(),
        )
        .expect("prototype MLS KeyPackage consume store cannot fail");
    }
    let decision = put_mls_key_package_consumption_record(&mut store, write)
        .expect("prototype MLS KeyPackage consume store cannot fail");

    json!({
        "fixture": name,
        "surface": "mls_key_package_consume_store",
        "input": {
            "key_package_admission_accepted": write.key_package_admission.accepted,
            "key_package_admission_reason_code": write.key_package_admission.reason.code(),
            "key_package_admission_reason_label": write.key_package_admission.reason.label(),
            "group_id_len": write.group_id.len(),
            "key_package_hash_len": write.key_package_hash.len(),
            "added_member_ref_len": write.added_member_ref.len(),
            "welcome_send_transaction_digest_len": write.welcome_send_transaction_digest.len(),
            "consumed_at_s": write.consumed_at_s,
            "plaintext_metadata_fields": write.plaintext_metadata_fields,
            "seed_duplicate": seed_duplicate,
        },
        "decision": mls_key_package_consume_store_decision_value(decision),
        "store": {
            "record_count": store.len(),
            "has_record": store.get(write.key_package_hash).is_some(),
            "global_duplicate_check": {
                "other_group_id_len": MLS_KEY_PACKAGE_OTHER_GROUP_ID.len(),
                "keyed_by_key_package_hash": true,
            },
        },
    })
}

fn mls_key_package_consume_store_decision_value(
    decision: MlsKeyPackageConsumeStoreDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "can_consume_key_package_once": decision.can_consume_key_package_once,
        "can_send_welcome_once": decision.can_send_welcome_once,
        "prevents_key_package_reuse": decision.prevents_key_package_reuse,
        "binds_added_member_ref": decision.binds_added_member_ref,
        "binds_welcome_send_transaction": decision.binds_welcome_send_transaction,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_welcome_send_outbox_ready_fixture() -> Value {
    mls_welcome_send_outbox_fixture(
        "mls_welcome_send_outbox_ready",
        valid_mls_welcome_send_outbox_write(),
        false,
    )
}

fn mls_welcome_send_outbox_consume_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_send_outbox_write();
    let mut consume_store = PrototypeMlsKeyPackageConsumeStore::default();
    let mut consume_write = valid_mls_welcome_send_key_package_consumption_write();
    consume_write.welcome_send_transaction_digest = &MLS_WELCOME_SEND_SHORT_DIGEST;
    input.key_package_consumption =
        put_mls_key_package_consumption_record(&mut consume_store, consume_write)
            .expect("prototype MLS KeyPackage consume store cannot fail");
    mls_welcome_send_outbox_fixture("mls_welcome_send_outbox_consume_rejected", input, false)
}

fn mls_welcome_send_outbox_duplicate_transaction_rejected_fixture() -> Value {
    mls_welcome_send_outbox_fixture(
        "mls_welcome_send_outbox_duplicate_transaction_rejected",
        valid_mls_welcome_send_outbox_write(),
        true,
    )
}

fn mls_welcome_send_outbox_key_package_queued_fixture() -> Value {
    let mut input = valid_mls_welcome_send_outbox_write();
    input.welcome_send_transaction_digest = &MLS_WELCOME_SEND_OTHER_TRANSACTION_DIGEST;
    mls_welcome_send_outbox_fixture("mls_welcome_send_outbox_key_package_queued", input, true)
}

fn mls_welcome_send_outbox_bad_shape_fixture() -> Value {
    let mut input = valid_mls_welcome_send_outbox_write();
    input.delivery_route_id = &MLS_WELCOME_SEND_SHORT_DIGEST;
    mls_welcome_send_outbox_fixture("mls_welcome_send_outbox_bad_shape", input, false)
}

fn mls_welcome_send_outbox_plaintext_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_send_outbox_write();
    input.plaintext_metadata_fields = 1;
    mls_welcome_send_outbox_fixture("mls_welcome_send_outbox_plaintext_rejected", input, false)
}

fn mls_welcome_send_outbox_fixture(
    name: &'static str,
    write: MlsWelcomeSendOutboxWrite<'static>,
    seed_duplicate: bool,
) -> Value {
    let mut outbox = PrototypeMlsWelcomeSendOutbox::default();
    if seed_duplicate {
        let _ =
            put_mls_welcome_send_outbox_record(&mut outbox, valid_mls_welcome_send_outbox_write())
                .expect("prototype MLS Welcome send outbox cannot fail");
    }
    let decision = put_mls_welcome_send_outbox_record(&mut outbox, write)
        .expect("prototype MLS Welcome send outbox cannot fail");

    json!({
        "fixture": name,
        "surface": "mls_welcome_send_outbox",
        "input": {
            "key_package_consumption_accepted": write.key_package_consumption.accepted,
            "key_package_consumption_reason_code": write.key_package_consumption.reason.code(),
            "key_package_consumption_reason_label": write.key_package_consumption.reason.label(),
            "commit_admission_accepted": write.commit_admission.accepted,
            "commit_admission_reason_code": write.commit_admission.reason.code(),
            "commit_admission_reason_label": write.commit_admission.reason.label(),
            "group_id_len": write.group_id.len(),
            "key_package_hash_len": write.key_package_hash.len(),
            "added_member_ref_len": write.added_member_ref.len(),
            "welcome_send_transaction_digest_len": write.welcome_send_transaction_digest.len(),
            "commit_hash_len": write.commit_hash.len(),
            "welcome_ciphertext_hash_len": write.welcome_ciphertext_hash.len(),
            "delivery_route_id_len": write.delivery_route_id.len(),
            "replay_token_len": write.replay_token.len(),
            "created_at_s": write.created_at_s,
            "expires_at_s": write.expires_at_s,
            "plaintext_metadata_fields": write.plaintext_metadata_fields,
            "seed_duplicate": seed_duplicate,
        },
        "decision": mls_welcome_send_outbox_decision_value(decision),
        "outbox": {
            "record_count": outbox.len(),
            "has_record": outbox.get(write.welcome_send_transaction_digest).is_some(),
            "unique_transaction_digest": true,
            "unique_key_package_hash": true,
        },
    })
}

fn mls_welcome_send_outbox_decision_value(decision: MlsWelcomeSendOutboxDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "can_enqueue_welcome_once": decision.can_enqueue_welcome_once,
        "can_send_welcome_after_commit": decision.can_send_welcome_after_commit,
        "consumes_key_package": decision.consumes_key_package,
        "binds_welcome_send_transaction": decision.binds_welcome_send_transaction,
        "binds_commit": decision.binds_commit,
        "binds_delivery_route": decision.binds_delivery_route,
        "prevents_duplicate_outbox": decision.prevents_duplicate_outbox,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_membership_transaction_ready_fixture() -> Value {
    mls_membership_transaction_fixture(
        "mls_membership_transaction_ready",
        valid_mls_membership_transaction_write(),
        false,
    )
}

fn mls_membership_transaction_binding_rejected_fixture() -> Value {
    let mut write = valid_mls_membership_transaction_write();
    write.outbox_group_id = &MLS_MEMBERSHIP_TRANSACTION_OTHER_GROUP_ID;
    mls_membership_transaction_fixture("mls_membership_transaction_binding_rejected", write, false)
}

fn mls_membership_transaction_storage_rejected_fixture() -> Value {
    let mut write = valid_mls_membership_transaction_write();
    write.single_storage_transaction = false;
    mls_membership_transaction_fixture("mls_membership_transaction_storage_rejected", write, false)
}

fn mls_membership_transaction_duplicate_rejected_fixture() -> Value {
    mls_membership_transaction_fixture(
        "mls_membership_transaction_duplicate_rejected",
        valid_mls_membership_transaction_write(),
        true,
    )
}

fn mls_membership_transaction_plaintext_rejected_fixture() -> Value {
    let mut write = valid_mls_membership_transaction_write();
    write.plaintext_metadata_fields = 1;
    mls_membership_transaction_fixture(
        "mls_membership_transaction_plaintext_rejected",
        write,
        false,
    )
}

fn mls_membership_transaction_fixture(
    name: &'static str,
    write: MlsMembershipTransactionWrite<'static>,
    seed_duplicate: bool,
) -> Value {
    let mut store = PrototypeMlsMembershipTransactionStore::default();
    if seed_duplicate {
        let _ = put_mls_membership_transaction_record(
            &mut store,
            valid_mls_membership_transaction_write(),
        )
        .expect("prototype MLS membership transaction store cannot fail");
    }
    let decision = put_mls_membership_transaction_record(&mut store, write)
        .expect("prototype MLS membership transaction store cannot fail");

    json!({
        "fixture": name,
        "surface": "mls_membership_transaction",
        "input": {
            "commit_replay_accepted": write.commit_replay.accepted,
            "commit_replay_reason_code": write.commit_replay.reason.code(),
            "commit_replay_reason_label": write.commit_replay.reason.label(),
            "key_package_consumption_accepted": write.key_package_consumption.accepted,
            "key_package_consumption_reason_code": write.key_package_consumption.reason.code(),
            "key_package_consumption_reason_label": write.key_package_consumption.reason.label(),
            "welcome_send_outbox_accepted": write.welcome_send_outbox.accepted,
            "welcome_send_outbox_reason_code": write.welcome_send_outbox.reason.code(),
            "welcome_send_outbox_reason_label": write.welcome_send_outbox.reason.label(),
            "group_id_len": write.group_id.len(),
            "commit_hash_len": write.commit_hash.len(),
            "key_package_hash_len": write.key_package_hash.len(),
            "welcome_send_transaction_digest_len": write.welcome_send_transaction_digest.len(),
            "membership_transaction_digest_len": write.membership_transaction_digest.len(),
            "created_at_s": write.created_at_s,
            "single_storage_transaction": write.single_storage_transaction,
            "serializable_isolation": write.serializable_isolation,
            "durable_commit": write.durable_commit,
            "unique_commit_hash_constraint": write.unique_commit_hash_constraint,
            "unique_key_package_hash_constraint": write.unique_key_package_hash_constraint,
            "unique_welcome_transaction_constraint": write.unique_welcome_transaction_constraint,
            "outbox_worker_idempotent": write.outbox_worker_idempotent,
            "crash_recovery_reconciles_pending_welcome": write.crash_recovery_reconciles_pending_welcome,
            "binding_group_ids_match": write.group_id == write.commit_replay_group_id
                && write.group_id == write.key_package_group_id
                && write.group_id == write.outbox_group_id,
            "binding_commit_hashes_match": write.commit_hash == write.commit_replay_commit_hash
                && write.commit_hash == write.outbox_commit_hash,
            "binding_key_package_hashes_match": write.key_package_hash == write.key_package_hash_from_consumption
                && write.key_package_hash == write.outbox_key_package_hash,
            "binding_welcome_transaction_digests_match": write.welcome_send_transaction_digest == write.key_package_welcome_send_transaction_digest
                && write.welcome_send_transaction_digest == write.outbox_welcome_send_transaction_digest,
            "plaintext_metadata_fields": write.plaintext_metadata_fields,
            "seed_duplicate": seed_duplicate,
        },
        "decision": mls_membership_transaction_decision_value(decision),
        "store": {
            "record_count": store.len(),
            "has_record": store.get(write.membership_transaction_digest).is_some(),
            "unique_membership_transaction_digest": true,
            "transaction_marker_digest_only": true,
        },
    })
}

fn mls_membership_transaction_decision_value(decision: MlsMembershipTransactionDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "can_commit_membership_change_once": decision.can_commit_membership_change_once,
        "can_advance_epoch": decision.can_advance_epoch,
        "can_send_welcome_from_outbox": decision.can_send_welcome_from_outbox,
        "binds_commit_key_package_welcome": decision.binds_commit_key_package_welcome,
        "uses_single_storage_transaction": decision.uses_single_storage_transaction,
        "uses_serializable_isolation": decision.uses_serializable_isolation,
        "has_durable_commit": decision.has_durable_commit,
        "enforces_unique_constraints": decision.enforces_unique_constraints,
        "has_idempotent_worker": decision.has_idempotent_worker,
        "has_crash_recovery": decision.has_crash_recovery,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_welcome_admission_ready_fixture() -> Value {
    mls_welcome_admission_fixture(
        "mls_welcome_admission_ready",
        valid_mls_welcome_admission_input(),
    )
}

fn mls_welcome_admission_secrets_missing_fixture() -> Value {
    let mut input = valid_mls_welcome_admission_input();
    input.matching_encrypted_group_secrets = false;
    mls_welcome_admission_fixture("mls_welcome_admission_secrets_missing", input)
}

fn mls_welcome_admission_tree_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_admission_input();
    input.ratchet_tree_hash_matches = false;
    mls_welcome_admission_fixture("mls_welcome_admission_tree_rejected", input)
}

fn mls_welcome_admission_confirmation_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_admission_input();
    input.confirmation_tag_valid = false;
    mls_welcome_admission_fixture("mls_welcome_admission_confirmation_rejected", input)
}

fn mls_welcome_admission_tie_break_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_admission_input();
    input.commit_won_tie_break = false;
    mls_welcome_admission_fixture("mls_welcome_admission_tie_break_rejected", input)
}

fn mls_welcome_admission_replay_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_admission_input();
    input.welcome_hash_already_processed = true;
    mls_welcome_admission_fixture("mls_welcome_admission_replay_rejected", input)
}

fn mls_welcome_admission_plaintext_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_admission_input();
    input.plaintext_group_metadata_fields = 1;
    mls_welcome_admission_fixture("mls_welcome_admission_plaintext_rejected", input)
}

fn mls_welcome_admission_fixture(name: &'static str, input: MlsWelcomeAdmissionInput) -> Value {
    let decision = evaluate_mls_welcome_admission(input);

    json!({
        "fixture": name,
        "surface": "mls_welcome_admission",
        "input": {
            "key_package_admission_accepted": input.key_package_admission.accepted,
            "key_package_admission_reason_code": input.key_package_admission.reason.code(),
            "key_package_admission_reason_label": input.key_package_admission.reason.label(),
            "welcome_cipher_suite_code": input.welcome_cipher_suite.code(),
            "welcome_cipher_suite_label": input.welcome_cipher_suite.label(),
            "key_package_suite_code": input.key_package_suite.code(),
            "key_package_suite_label": input.key_package_suite.label(),
            "group_info_suite_code": input.group_info_suite.code(),
            "group_info_suite_label": input.group_info_suite.label(),
            "matching_encrypted_group_secrets": input.matching_encrypted_group_secrets,
            "group_secrets_decrypted": input.group_secrets_decrypted,
            "psks_available": input.psks_available,
            "resumption_psk_count": input.resumption_psk_count,
            "encrypted_group_info_decrypted": input.encrypted_group_info_decrypted,
            "group_info_signature_valid": input.group_info_signature_valid,
            "group_id_unique_locally": input.group_id_unique_locally,
            "ratchet_tree_available_confidentially": input.ratchet_tree_available_confidentially,
            "ratchet_tree_hash_matches": input.ratchet_tree_hash_matches,
            "ratchet_tree_parent_hash_valid": input.ratchet_tree_parent_hash_valid,
            "ratchet_tree_leaves_valid": input.ratchet_tree_leaves_valid,
            "ratchet_tree_unmerged_leaves_valid": input.ratchet_tree_unmerged_leaves_valid,
            "ratchet_tree_unique_encryption_keys": input.ratchet_tree_unique_encryption_keys,
            "own_leaf_found": input.own_leaf_found,
            "own_leaf_matches_key_package": input.own_leaf_matches_key_package,
            "path_secret_valid": input.path_secret_valid,
            "epoch_secret_derived": input.epoch_secret_derived,
            "confirmed_transcript_hash_len": input.confirmed_transcript_hash_len,
            "confirmation_tag_valid": input.confirmation_tag_valid,
            "commit_won_tie_break": input.commit_won_tie_break,
            "group_epoch": input.group_epoch,
            "reinit_psk_used": input.reinit_psk_used,
            "reinit_epoch_is_one": input.reinit_epoch_is_one,
            "welcome_hash_len": input.welcome_hash_len,
            "welcome_hash_already_processed": input.welcome_hash_already_processed,
            "plaintext_group_metadata_fields": input.plaintext_group_metadata_fields,
        },
        "decision": mls_welcome_admission_decision_value(decision),
    })
}

fn mls_welcome_admission_decision_value(decision: MlsWelcomeAdmissionDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_join_group": decision.can_join_group,
        "can_initialize_epoch": decision.can_initialize_epoch,
        "can_open_group": decision.can_open_group,
        "requires_sync": decision.requires_sync,
        "requires_mls_setup": decision.requires_mls_setup,
        "requires_pq_upgrade": decision.requires_pq_upgrade,
        "requires_user_action": decision.requires_user_action,
        "requires_tree_fetch": decision.requires_tree_fetch,
        "prevents_welcome_replay": decision.prevents_welcome_replay,
        "forbids_plaintext_group_metadata": decision.forbids_plaintext_group_metadata,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_welcome_replay_store_ready_fixture() -> Value {
    mls_welcome_replay_store_fixture(
        "mls_welcome_replay_store_ready",
        valid_mls_welcome_replay_store_write(),
        false,
    )
}

fn mls_welcome_replay_store_admission_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_replay_store_write();
    let mut admission_input = valid_mls_welcome_admission_input();
    admission_input.welcome_hash_len = 31;
    input.welcome_admission = admission_input.evaluate();
    mls_welcome_replay_store_fixture("mls_welcome_replay_store_admission_rejected", input, false)
}

fn mls_welcome_replay_store_duplicate_rejected_fixture() -> Value {
    mls_welcome_replay_store_fixture(
        "mls_welcome_replay_store_duplicate_rejected",
        valid_mls_welcome_replay_store_write(),
        true,
    )
}

fn mls_welcome_replay_store_key_package_reused_fixture() -> Value {
    let mut input = valid_mls_welcome_replay_store_write();
    input.welcome_hash = &MLS_WELCOME_OTHER_HASH;
    mls_welcome_replay_store_fixture("mls_welcome_replay_store_key_package_reused", input, true)
}

fn mls_welcome_replay_store_bad_shape_fixture() -> Value {
    let mut input = valid_mls_welcome_replay_store_write();
    input.consumed_key_package_ref = &MLS_WELCOME_SHORT_DIGEST;
    mls_welcome_replay_store_fixture("mls_welcome_replay_store_bad_shape", input, false)
}

fn mls_welcome_replay_store_plaintext_rejected_fixture() -> Value {
    let mut input = valid_mls_welcome_replay_store_write();
    input.plaintext_metadata_fields = 1;
    mls_welcome_replay_store_fixture("mls_welcome_replay_store_plaintext_rejected", input, false)
}

fn mls_welcome_replay_store_fixture(
    name: &'static str,
    write: MlsWelcomeReplayStoreWrite<'static>,
    seed_duplicate: bool,
) -> Value {
    let mut store = PrototypeMlsWelcomeReplayStore::default();
    if seed_duplicate {
        let _ = put_mls_welcome_replay_record(&mut store, valid_mls_welcome_replay_store_write())
            .expect("prototype MLS Welcome replay store cannot fail");
    }
    let decision = put_mls_welcome_replay_record(&mut store, write)
        .expect("prototype MLS Welcome replay store cannot fail");

    json!({
        "fixture": name,
        "surface": "mls_welcome_replay_store",
        "input": {
            "welcome_admission_accepted": write.welcome_admission.accepted,
            "welcome_admission_reason_code": write.welcome_admission.reason.code(),
            "welcome_admission_reason_label": write.welcome_admission.reason.label(),
            "welcome_admission_can_join_group": write.welcome_admission.can_join_group,
            "welcome_admission_can_initialize_epoch": write.welcome_admission.can_initialize_epoch,
            "welcome_admission_can_open_group": write.welcome_admission.can_open_group,
            "group_id_len": write.group_id.len(),
            "welcome_hash_len": write.welcome_hash.len(),
            "consumed_key_package_ref_len": write.consumed_key_package_ref.len(),
            "tree_hash_len": write.tree_hash.len(),
            "confirmed_transcript_hash_len": write.confirmed_transcript_hash.len(),
            "group_state_commit_digest_len": write.group_state_commit_digest.len(),
            "epoch": write.epoch,
            "joined_at_s": write.joined_at_s,
            "init_key_deleted": write.init_key_deleted,
            "group_state_committed": write.group_state_committed,
            "plaintext_metadata_fields": write.plaintext_metadata_fields,
            "seed_duplicate": seed_duplicate,
        },
        "decision": mls_welcome_replay_store_decision_value(decision),
        "store": {
            "record_count": store.len(),
            "has_record": store.get(write.group_id, write.welcome_hash).is_some(),
        },
    })
}

fn mls_welcome_replay_store_decision_value(decision: MlsWelcomeReplayStoreDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "can_initialize_group_once": decision.can_initialize_group_once,
        "can_open_group": decision.can_open_group,
        "consumes_key_package": decision.consumes_key_package,
        "deletes_init_key": decision.deletes_init_key,
        "binds_tree_hash": decision.binds_tree_hash,
        "binds_confirmed_transcript_hash": decision.binds_confirmed_transcript_hash,
        "commits_group_state_transactionally": decision.commits_group_state_transactionally,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_commit_admission_ready_fixture() -> Value {
    mls_commit_admission_fixture(
        "mls_commit_admission_ready",
        valid_mls_commit_admission_input(),
    )
}

fn mls_commit_admission_bad_epoch_fixture() -> Value {
    let mut input = valid_mls_commit_admission_input();
    input.commit_epoch += 1;
    mls_commit_admission_fixture("mls_commit_admission_bad_epoch", input)
}

fn mls_commit_admission_auth_rejected_fixture() -> Value {
    let mut input = valid_mls_commit_admission_input();
    input.commit_membership_tag_valid = false;
    mls_commit_admission_fixture("mls_commit_admission_auth_rejected", input)
}

fn mls_commit_admission_path_rejected_fixture() -> Value {
    let mut input = valid_mls_commit_admission_input();
    input.update_path_secret_decryptable = false;
    mls_commit_admission_fixture("mls_commit_admission_path_rejected", input)
}

fn mls_commit_admission_tie_break_rejected_fixture() -> Value {
    let mut input = valid_mls_commit_admission_input();
    input.commit_won_tie_break = false;
    mls_commit_admission_fixture("mls_commit_admission_tie_break_rejected", input)
}

fn mls_commit_admission_replay_rejected_fixture() -> Value {
    let mut input = valid_mls_commit_admission_input();
    input.commit_hash_already_processed = true;
    mls_commit_admission_fixture("mls_commit_admission_replay_rejected", input)
}

fn mls_commit_admission_plaintext_rejected_fixture() -> Value {
    let mut input = valid_mls_commit_admission_input();
    input.plaintext_commit_metadata_fields = 1;
    mls_commit_admission_fixture("mls_commit_admission_plaintext_rejected", input)
}

fn mls_commit_admission_fixture(name: &'static str, input: MlsCommitAdmissionInput) -> Value {
    let decision = evaluate_mls_commit_admission(input);

    json!({
        "fixture": name,
        "surface": "mls_commit_admission",
        "input": {
            "group_chat_accepted": input.group_chat.accepted,
            "group_chat_reason_code": input.group_chat.reason.code(),
            "group_chat_reason_label": input.group_chat.reason.label(),
            "group_chat_protocol_code": input.group_chat.protocol.code(),
            "group_chat_protocol_label": input.group_chat.protocol.label(),
            "group_chat_suite_code": input.group_chat.crypto_suite.code(),
            "group_chat_suite_label": input.group_chat.crypto_suite.label(),
            "current_epoch": input.current_epoch,
            "commit_epoch": input.commit_epoch,
            "external_commit": input.external_commit,
            "sender_is_member": input.sender_is_member,
            "sender_type_new_member_commit": input.sender_type_new_member_commit,
            "external_init_present": input.external_init_present,
            "commit_signature_valid": input.commit_signature_valid,
            "commit_membership_tag_valid": input.commit_membership_tag_valid,
            "proposal_list_valid": input.proposal_list_valid,
            "referenced_proposals_available": input.referenced_proposals_available,
            "application_policy_accepts_proposals": input.application_policy_accepts_proposals,
            "duplicate_update_or_remove_targets": input.duplicate_update_or_remove_targets,
            "committer_update_present": input.committer_update_present,
            "committer_remove_present": input.committer_remove_present,
            "path_required": input.path_required,
            "update_path_present": input.update_path_present,
            "update_path_leaf_valid": input.update_path_leaf_valid,
            "update_path_leaf_source_commit": input.update_path_leaf_source_commit,
            "update_path_parent_hash_valid": input.update_path_parent_hash_valid,
            "update_path_secret_decryptable": input.update_path_secret_decryptable,
            "ratchet_tree_hash_matches": input.ratchet_tree_hash_matches,
            "provisional_group_context_bound": input.provisional_group_context_bound,
            "epoch_secret_derived": input.epoch_secret_derived,
            "confirmed_transcript_hash_len": input.confirmed_transcript_hash_len,
            "confirmation_tag_valid": input.confirmation_tag_valid,
            "commit_won_tie_break": input.commit_won_tie_break,
            "commit_hash_len": input.commit_hash_len,
            "commit_hash_already_processed": input.commit_hash_already_processed,
            "removes_local_member": input.removes_local_member,
            "plaintext_commit_metadata_fields": input.plaintext_commit_metadata_fields,
        },
        "decision": mls_commit_admission_decision_value(decision),
    })
}

fn mls_commit_admission_decision_value(decision: MlsCommitAdmissionDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_apply_commit": decision.can_apply_commit,
        "can_initialize_epoch": decision.can_initialize_epoch,
        "can_continue_group": decision.can_continue_group,
        "local_member_removed": decision.local_member_removed,
        "requires_sync": decision.requires_sync,
        "requires_mls_setup": decision.requires_mls_setup,
        "requires_tree_repair": decision.requires_tree_repair,
        "requires_rekey": decision.requires_rekey,
        "requires_user_action": decision.requires_user_action,
        "prevents_commit_replay": decision.prevents_commit_replay,
        "forbids_plaintext_commit_metadata": decision.forbids_plaintext_commit_metadata,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn mls_commit_replay_store_ready_fixture() -> Value {
    mls_commit_replay_store_fixture(
        "mls_commit_replay_store_ready",
        valid_mls_commit_replay_store_write(),
        false,
    )
}

fn mls_commit_replay_store_admission_rejected_fixture() -> Value {
    let mut input = valid_mls_commit_replay_store_write();
    let mut admission_input = valid_mls_commit_admission_input();
    admission_input.commit_hash_len = 31;
    input.commit_admission = admission_input.evaluate();
    mls_commit_replay_store_fixture("mls_commit_replay_store_admission_rejected", input, false)
}

fn mls_commit_replay_store_duplicate_rejected_fixture() -> Value {
    mls_commit_replay_store_fixture(
        "mls_commit_replay_store_duplicate_rejected",
        valid_mls_commit_replay_store_write(),
        true,
    )
}

fn mls_commit_replay_store_local_member_removed_fixture() -> Value {
    let mut input = valid_mls_commit_replay_store_write();
    let mut admission_input = valid_mls_commit_admission_input();
    admission_input.removes_local_member = true;
    input.commit_admission = admission_input.evaluate();
    mls_commit_replay_store_fixture("mls_commit_replay_store_local_member_removed", input, false)
}

fn mls_commit_replay_store_plaintext_rejected_fixture() -> Value {
    let mut input = valid_mls_commit_replay_store_write();
    input.plaintext_metadata_fields = 1;
    mls_commit_replay_store_fixture("mls_commit_replay_store_plaintext_rejected", input, false)
}

fn mls_commit_replay_store_fixture(
    name: &'static str,
    write: MlsCommitReplayStoreWrite<'static>,
    seed_duplicate: bool,
) -> Value {
    let mut store = PrototypeMlsCommitReplayStore::default();
    if seed_duplicate {
        let _ = put_mls_commit_replay_record(&mut store, valid_mls_commit_replay_store_write())
            .expect("prototype MLS Commit replay store cannot fail");
    }
    let decision = put_mls_commit_replay_record(&mut store, write)
        .expect("prototype MLS Commit replay store cannot fail");

    json!({
        "fixture": name,
        "surface": "mls_commit_replay_store",
        "input": {
            "commit_admission_accepted": write.commit_admission.accepted,
            "commit_admission_reason_code": write.commit_admission.reason.code(),
            "commit_admission_reason_label": write.commit_admission.reason.label(),
            "commit_admission_can_apply_commit": write.commit_admission.can_apply_commit,
            "commit_admission_can_continue_group": write.commit_admission.can_continue_group,
            "commit_admission_local_member_removed": write.commit_admission.local_member_removed,
            "group_id_len": write.group_id.len(),
            "commit_hash_len": write.commit_hash.len(),
            "epoch": write.epoch,
            "applied_at_s": write.applied_at_s,
            "plaintext_metadata_fields": write.plaintext_metadata_fields,
            "seed_duplicate": seed_duplicate,
        },
        "decision": mls_commit_replay_store_decision_value(decision),
        "store": {
            "record_count": store.len(),
            "has_record": store.get(write.group_id, write.commit_hash).is_some(),
        },
    })
}

fn mls_commit_replay_store_decision_value(decision: MlsCommitReplayStoreDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "can_apply_commit_once": decision.can_apply_commit_once,
        "can_continue_group": decision.can_continue_group,
        "local_member_removed": decision.local_member_removed,
        "keeps_digest_only": decision.keeps_digest_only,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn group_relay_envelope_ready_fixture() -> Value {
    group_relay_envelope_fixture(
        "group_relay_envelope_ready",
        valid_group_relay_envelope_input(),
    )
}

fn group_relay_envelope_transcript_sync_required_fixture() -> Value {
    let mut input = valid_group_relay_envelope_input();
    let mut transcript_input = valid_group_message_transcript_input();
    transcript_input.message_epoch = transcript_input.local_epoch - 1;
    input.transcript = transcript_input.evaluate();
    group_relay_envelope_fixture("group_relay_envelope_transcript_sync_required", input)
}

fn group_relay_envelope_transcript_rekey_required_fixture() -> Value {
    let mut input = valid_group_relay_envelope_input();
    let mut transcript_input = valid_group_message_transcript_input();
    transcript_input.sender_data_sealed = false;
    input.transcript = transcript_input.evaluate();
    group_relay_envelope_fixture("group_relay_envelope_transcript_rekey_required", input)
}

fn group_relay_envelope_missing_delivery_token_fixture() -> Value {
    let mut input = valid_group_relay_envelope_input();
    input.delivery_token_len = 0;
    group_relay_envelope_fixture("group_relay_envelope_missing_delivery_token", input)
}

fn group_relay_envelope_plaintext_metadata_rejected_fixture() -> Value {
    let mut input = valid_group_relay_envelope_input();
    input.plaintext_sender_fields = 1;
    group_relay_envelope_fixture("group_relay_envelope_plaintext_metadata_rejected", input)
}

fn group_relay_envelope_fixture(name: &'static str, input: GroupRelayEnvelopeInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "group_relay_envelope",
        "input": {
            "transcript_accepted": input.transcript.accepted,
            "transcript_reason_label": input.transcript.reason.label(),
            "transcript_can_submit_to_relay": input.transcript.can_submit_to_relay,
            "transcript_requires_sync": input.transcript.requires_sync,
            "transcript_requires_rekey": input.transcript.requires_rekey,
            "transcript_requires_user_action": input.transcript.requires_user_action,
            "relay_submission_accepted": input.relay_submission.accepted,
            "relay_submission_reason_code": input.relay_submission.reason_code,
            "relay_submission_audit_class": input.relay_submission.audit_class,
            "delivery_token_len": input.delivery_token_len,
            "delivery_token_bound_to_route": input.delivery_token_bound_to_route,
            "sender_certificate_sealed": input.sender_certificate_sealed,
            "anonymous_membership_proof_accepted": input.anonymous_membership_proof.accepted,
            "anonymous_membership_proof_reason_label": input.anonymous_membership_proof.reason.label(),
            "anonymous_membership_proof_requires_sync": input.anonymous_membership_proof.requires_sync,
            "anonymous_membership_proof_requires_rekey": input.anonymous_membership_proof.requires_rekey,
            "anonymous_membership_proof_requires_user_action": input.anonymous_membership_proof.requires_user_action,
            "anonymous_membership_proof_len": input.anonymous_membership_proof_len,
            "anonymous_rate_limit_accepted": input.anonymous_rate_limit.accepted,
            "anonymous_rate_limit_reason_label": input.anonymous_rate_limit.reason.label(),
            "anonymous_rate_limit_requires_sync": input.anonymous_rate_limit.requires_sync,
            "anonymous_rate_limit_requires_rekey": input.anonymous_rate_limit.requires_rekey,
            "anonymous_rate_limit_requires_user_action": input.anonymous_rate_limit.requires_user_action,
            "sealed_envelope_len": input.sealed_envelope_len,
            "plaintext_sender_fields": input.plaintext_sender_fields,
            "plaintext_group_fields": input.plaintext_group_fields,
        },
        "decision": group_relay_envelope_decision_value(decision),
    })
}

fn group_relay_envelope_decision_value(decision: GroupRelayEnvelopeDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_enqueue_relay": decision.can_enqueue_relay,
        "requires_sync": decision.requires_sync,
        "requires_rekey": decision.requires_rekey,
        "requires_user_action": decision.requires_user_action,
        "forbids_plaintext_sender": decision.forbids_plaintext_sender,
        "forbids_plaintext_group": decision.forbids_plaintext_group,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
    })
}

fn local_store_production_open_ready_fixture() -> Value {
    local_store_production_open_fixture(
        "local_store_production_open_ready",
        valid_local_store_production_open_input(),
    )
}

fn local_store_production_open_wal_replay_required_fixture() -> Value {
    let mut input = valid_local_store_production_open_input();
    input.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;
    local_store_production_open_fixture("local_store_production_open_wal_replay_required", input)
}

fn local_store_production_open_plaintext_key_slot_forbidden_fixture() -> Value {
    let mut input = valid_local_store_production_open_input();
    input.plaintext_key_slots = 1;
    local_store_production_open_fixture(
        "local_store_production_open_plaintext_key_slot_forbidden",
        input,
    )
}

fn local_store_production_open_app_lock_required_fixture() -> Value {
    let mut input = valid_local_store_production_open_input();
    input.unlock.app_lock_satisfied = false;
    local_store_production_open_fixture("local_store_production_open_app_lock_required", input)
}

fn local_store_production_open_fixture(
    name: &'static str,
    input: LocalStoreProductionOpenInput,
) -> Value {
    let decision = input.evaluate();
    let header_suite_label = LocalStoreSealingSuite::from_code(input.header_suite_code)
        .map(|suite| suite.label())
        .unwrap_or("unknown");

    json!({
        "fixture": name,
        "surface": "local_store_production_open",
        "input": {
            "unlock": {
                "store_version": input.unlock.store_version,
                "keychain_available": input.unlock.keychain_available,
                "device_secret": format!("{:?}", input.unlock.device_secret),
                "database_header": format!("{:?}", input.unlock.database_header),
                "app_lock_satisfied": input.unlock.app_lock_satisfied,
                "recovery_required": input.unlock.recovery_required,
                "plaintext_cache_records": input.unlock.plaintext_cache_records,
            },
            "manifest": {
                "header_magic_matches": input.header_magic_matches,
                "header_suite_code": input.header_suite_code,
                "header_suite_label": header_suite_label,
                "header_nonce_len": input.header_nonce_len,
                "header_tag_len": input.header_tag_len,
                "required_key_slots": input.required_key_slots,
                "sealed_key_slots": input.sealed_key_slots,
                "plaintext_key_slots": input.plaintext_key_slots,
                "root_key_scope": format!("{:?}", input.root_key_scope),
                "root_key_generation": input.root_key_generation,
                "crash_recovery": input.crash_recovery.label(),
            },
        },
        "decision": local_store_production_open_decision_value(decision),
    })
}

fn local_store_production_open_decision_value(decision: LocalStoreProductionOpenDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_open_database": decision.can_open_database,
        "can_replay_wal": decision.can_replay_wal,
        "can_load_records": decision.can_load_records,
        "can_load_message_keys": decision.can_load_message_keys,
        "requires_user_auth": decision.requires_user_auth,
        "requires_recovery": decision.requires_recovery,
        "requires_migration": decision.requires_migration,
        "requires_crash_recovery": decision.requires_crash_recovery,
        "requires_destructive_repair": decision.requires_destructive_repair,
        "unlock_decision": local_store_unlock_decision_value(decision.unlock_decision),
    })
}

fn local_store_keychain_android_ready_fixture() -> Value {
    local_store_keychain_fixture(
        "local_store_keychain_android_ready",
        valid_local_store_keychain_unlock_input(),
    )
}

fn local_store_keychain_user_auth_required_fixture() -> Value {
    let mut input = valid_local_store_keychain_unlock_input();
    input.user_auth_required = true;
    input.user_auth_satisfied = false;
    local_store_keychain_fixture("local_store_keychain_user_auth_required", input)
}

fn local_store_keychain_exportable_secret_forbidden_fixture() -> Value {
    let mut input = valid_local_store_keychain_unlock_input();
    input.device_secret_exportable = true;
    local_store_keychain_fixture("local_store_keychain_exportable_secret_forbidden", input)
}

fn local_store_keychain_development_backend_forbidden_fixture() -> Value {
    let mut input = valid_local_store_keychain_unlock_input();
    input.backend = LocalStoreKeychainBackend::DevelopmentMemory;
    input.protection = LocalStoreKeychainProtection::DevelopmentOnly;
    local_store_keychain_fixture("local_store_keychain_development_backend_forbidden", input)
}

fn local_store_keychain_fixture(name: &'static str, input: LocalStoreKeychainUnlockInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "local_store_keychain_unlock",
        "input": {
            "store_version": input.store_version,
            "backend_code": input.backend.code(),
            "backend_label": input.backend.label(),
            "backend_available": input.backend_available,
            "protection_code": input.protection.code(),
            "protection_label": input.protection.label(),
            "allow_development_backend": input.allow_development_backend,
            "user_auth_required": input.user_auth_required,
            "user_auth_satisfied": input.user_auth_satisfied,
            "device_secret": format!("{:?}", input.device_secret),
            "device_secret_exportable": input.device_secret_exportable,
            "database_header": format!("{:?}", input.database_header),
            "recovery_required": input.recovery_required,
            "plaintext_cache_records": input.plaintext_cache_records,
        },
        "decision": local_store_keychain_decision_value(decision),
    })
}

fn local_store_keychain_decision_value(decision: LocalStoreKeychainUnlockDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "can_build_unlock_input": decision.can_build_unlock_input,
        "requires_user_auth": decision.requires_user_auth,
        "requires_recovery": decision.requires_recovery,
        "requires_destructive_repair": decision.requires_destructive_repair,
        "backend_code": decision.backend_code,
        "backend_label": decision.backend_label,
        "protection_code": decision.protection_code,
        "protection_label": decision.protection_label,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "unlock_input": {
            "store_version": decision.unlock_input.store_version,
            "keychain_available": decision.unlock_input.keychain_available,
            "device_secret": format!("{:?}", decision.unlock_input.device_secret),
            "database_header": format!("{:?}", decision.unlock_input.database_header),
            "app_lock_satisfied": decision.unlock_input.app_lock_satisfied,
            "recovery_required": decision.unlock_input.recovery_required,
            "plaintext_cache_records": decision.unlock_input.plaintext_cache_records,
        },
    })
}

fn production_store_session_happy_path_fixture() -> Value {
    production_store_session_fixture(
        "production_store_session_happy_path",
        valid_production_store_session_input(),
    )
}

fn production_store_session_keychain_rejected_fixture() -> Value {
    let mut input = valid_production_store_session_input();
    input.keychain.device_secret_exportable = true;
    production_store_session_fixture("production_store_session_keychain_rejected", input)
}

fn production_store_session_wal_replay_required_fixture() -> Value {
    let mut input = valid_production_store_session_input();
    input.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;
    production_store_session_fixture("production_store_session_wal_replay_required", input)
}

fn production_store_session_write_rejected_fixture() -> Value {
    let mut input = valid_production_store_session_input();
    input.write_request = LocalStoreWriteRequest::new(
        store_locator("conversation-7", "message-42"),
        LocalStoreRecordKind::MessagePlaintext,
        LocalStorePayload::public_metadata(b"plaintext"),
        Some(store_policy_decision(true)),
    );
    production_store_session_fixture("production_store_session_write_rejected", input)
}

fn production_store_session_fixture(
    name: &'static str,
    input: PrototypeProductionStoreSessionInput<'_>,
) -> Value {
    let mut store = PrototypeEncryptedLocalStore::default();
    let outcome = run_prototype_production_store_session(&mut store, input)
        .expect("prototype production store session is infallible");

    json!({
        "fixture": name,
        "surface": "prototype_production_store_session",
        "input": {
            "keychain": {
                "backend_label": input.keychain.backend.label(),
                "backend_available": input.keychain.backend_available,
                "protection_label": input.keychain.protection.label(),
                "allow_development_backend": input.keychain.allow_development_backend,
                "user_auth_required": input.keychain.user_auth_required,
                "user_auth_satisfied": input.keychain.user_auth_satisfied,
                "device_secret": format!("{:?}", input.keychain.device_secret),
                "device_secret_exportable": input.keychain.device_secret_exportable,
                "database_header": format!("{:?}", input.keychain.database_header),
                "recovery_required": input.keychain.recovery_required,
                "plaintext_cache_records": input.keychain.plaintext_cache_records,
            },
            "manifest": {
                "header_magic_matches": input.header_magic_matches,
                "header_suite_code": input.header_suite_code,
                "header_nonce_len": input.header_nonce_len,
                "header_tag_len": input.header_tag_len,
                "required_key_slots": input.required_key_slots,
                "sealed_key_slots": input.sealed_key_slots,
                "plaintext_key_slots": input.plaintext_key_slots,
                "root_key_scope": format!("{:?}", input.root_key_scope),
                "root_key_generation": input.root_key_generation,
                "crash_recovery": input.crash_recovery.label(),
            },
            "write": {
                "record_kind": input.write_request.record_kind.label(),
                "payload_kind": input.write_request.payload.kind().label(),
                "payload_len": input.write_request.payload.bytes().len(),
            },
        },
        "outcome": production_store_session_outcome_value(outcome),
    })
}

fn production_store_session_outcome_value(
    outcome: PrototypeProductionStoreSessionOutcome,
) -> Value {
    json!({
        "accepted": outcome.accepted,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "open_attempted": outcome.open_attempted,
        "wal_replayed": outcome.wal_replayed,
        "plaintext_exposed": outcome.plaintext_exposed,
        "keychain_decision": local_store_keychain_decision_value(outcome.keychain_decision),
        "unlock_decision": local_store_unlock_decision_value(outcome.unlock_decision),
        "production_open_decision": local_store_production_open_decision_value(outcome.production_open_decision),
        "write_decision": outcome.write_decision.map(|decision| json!({
            "accepted": decision.accepted,
            "reason": format!("{:?}", decision.reason),
        })),
        "read_record": outcome.read_record.map(|record| json!({
            "namespace": record.namespace,
            "record_id": record.record_id,
            "record_kind": record.record_kind.label(),
            "payload_kind": record.payload_kind.label(),
            "byte_len": record.bytes.len(),
        })),
    })
}

fn platform_local_store_adapter_desktop_ready_fixture() -> Value {
    platform_local_store_adapter_fixture(
        "platform_local_store_adapter_desktop_ready",
        valid_platform_local_store_adapter_input(PlatformLocalStoreRuntime::Desktop),
    )
}

fn platform_local_store_adapter_mobile_hardware_required_fixture() -> Value {
    let mut input = valid_platform_local_store_adapter_input(PlatformLocalStoreRuntime::Mobile);
    input.hardware_backed_key_store = false;
    platform_local_store_adapter_fixture(
        "platform_local_store_adapter_mobile_hardware_required",
        input,
    )
}

fn platform_local_store_adapter_plaintext_forbidden_fixture() -> Value {
    let mut input = valid_platform_local_store_adapter_input(PlatformLocalStoreRuntime::Desktop);
    input.adapter_kind = PlatformLocalStoreAdapterKind::PlaintextFileStore;
    platform_local_store_adapter_fixture("platform_local_store_adapter_plaintext_forbidden", input)
}

fn platform_local_store_adapter_app_lock_required_fixture() -> Value {
    let mut input = valid_platform_local_store_adapter_input(PlatformLocalStoreRuntime::Desktop);
    input.app_lock_satisfied = false;
    platform_local_store_adapter_fixture("platform_local_store_adapter_app_lock_required", input)
}

fn platform_local_store_adapter_fixture(
    name: &'static str,
    input: PlatformLocalStoreAdapterInput,
) -> Value {
    json!({
        "fixture": name,
        "surface": "platform_local_store_adapter",
        "input": {
            "runtime_code": input.runtime.code(),
            "runtime_label": input.runtime.label(),
            "adapter_kind_code": input.adapter_kind.code(),
            "adapter_kind_label": input.adapter_kind.label(),
            "database_root_present": input.database_root_present,
            "os_keychain_available": input.os_keychain_available,
            "hardware_backed_key_store": input.hardware_backed_key_store,
            "app_lock_satisfied": input.app_lock_satisfied,
            "allow_development_adapters": input.allow_development_adapters,
        },
        "decision": platform_local_store_adapter_decision_value(input.evaluate()),
    })
}

fn platform_local_store_adapter_decision_value(
    decision: PlatformLocalStoreAdapterDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason_code(),
        "reason_label": decision.reason_label(),
        "can_open_adapter": decision.can_open_adapter,
        "requires_user_auth": decision.requires_user_auth,
        "requires_install_setup": decision.requires_install_setup,
        "requires_hardware_backing": decision.requires_hardware_backing,
        "forbids_plaintext_storage": decision.forbids_plaintext_storage,
    })
}

fn local_store_database_security_ready_fixture() -> Value {
    local_store_database_security_fixture(
        "local_store_database_security_ready",
        valid_local_store_database_security_input(),
    )
}

fn local_store_database_security_plaintext_rejected_fixture() -> Value {
    let mut input = valid_local_store_database_security_input();
    input.engine = LocalStoreDatabaseEngine::PlainSqlite;
    local_store_database_security_fixture("local_store_database_security_plaintext_rejected", input)
}

fn local_store_database_security_wal_rejected_fixture() -> Value {
    let mut input = valid_local_store_database_security_input();
    input.encrypted_wal = false;
    local_store_database_security_fixture("local_store_database_security_wal_rejected", input)
}

fn local_store_database_security_backup_rejected_fixture() -> Value {
    let mut input = valid_local_store_database_security_input();
    input.os_cloud_backup_excluded = false;
    local_store_database_security_fixture("local_store_database_security_backup_rejected", input)
}

fn local_store_database_security_secret_lifecycle_rejected_fixture() -> Value {
    let mut input = valid_local_store_database_security_input();
    input.zeroizes_key_material = false;
    local_store_database_security_fixture(
        "local_store_database_security_secret_lifecycle_rejected",
        input,
    )
}

fn local_store_database_security_fixture(
    name: &'static str,
    input: LocalStoreDatabaseSecurityInput,
) -> Value {
    json!({
        "fixture": name,
        "surface": "local_store_database_security",
        "input": {
            "platform_adapter": platform_local_store_adapter_decision_value(input.platform_adapter),
            "production_open": local_store_production_open_decision_value(input.production_open),
            "engine_code": input.engine.code(),
            "engine_label": input.engine.label(),
            "cipher_code": input.cipher.code(),
            "cipher_label": input.cipher.label(),
            "kdf_code": input.kdf.code(),
            "kdf_label": input.kdf.label(),
            "kdf_iterations": input.kdf_iterations,
            "page_size": input.page_size,
            "per_page_random_nonce": input.per_page_random_nonce,
            "per_page_authentication": input.per_page_authentication,
            "encryption_key_separate_from_mac_key": input.encryption_key_separate_from_mac_key,
            "unique_database_salt": input.unique_database_salt,
            "raw_key_wrapped_by_platform_keystore": input.raw_key_wrapped_by_platform_keystore,
            "encrypted_wal": input.encrypted_wal,
            "encrypted_journal": input.encrypted_journal,
            "temp_store_memory_only": input.temp_store_memory_only,
            "plaintext_header_bytes": input.plaintext_header_bytes,
            "os_cloud_backup_excluded": input.os_cloud_backup_excluded,
            "backup_uses_consistent_encrypted_snapshot": input.backup_uses_consistent_encrypted_snapshot,
            "secure_delete_enabled": input.secure_delete_enabled,
            "memory_locking_enabled": input.memory_locking_enabled,
            "zeroizes_key_material": input.zeroizes_key_material,
            "crash_recovery_tested": input.crash_recovery_tested,
            "plaintext_metadata_fields": input.plaintext_metadata_fields,
            "sqlite_extension_loading_enabled": input.sqlite_extension_loading_enabled,
            "debug_plaintext_export_enabled": input.debug_plaintext_export_enabled,
        },
        "decision": local_store_database_security_decision_value(input.evaluate()),
    })
}

fn local_store_database_security_decision_value(
    decision: LocalStoreDatabaseSecurityDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_open_database": decision.can_open_database,
        "can_load_records": decision.can_load_records,
        "can_load_message_keys": decision.can_load_message_keys,
        "can_host_mls_transactions": decision.can_host_mls_transactions,
        "requires_user_auth": decision.requires_user_auth,
        "requires_install_setup": decision.requires_install_setup,
        "requires_hardware_backing": decision.requires_hardware_backing,
        "requires_recovery": decision.requires_recovery,
        "requires_migration": decision.requires_migration,
        "requires_crash_recovery": decision.requires_crash_recovery,
        "requires_destructive_repair": decision.requires_destructive_repair,
        "requires_backup_reconfiguration": decision.requires_backup_reconfiguration,
        "forbids_plaintext_storage": decision.forbids_plaintext_storage,
        "engine_code": decision.engine_code,
        "engine_label": decision.engine_label,
        "cipher_code": decision.cipher_code,
        "cipher_label": decision.cipher_label,
        "kdf_code": decision.kdf_code,
        "kdf_label": decision.kdf_label,
        "platform_adapter_reason": decision.platform_adapter_reason.label(),
        "production_open_reason": decision.production_open_reason.label(),
    })
}

fn local_store_database_adapter_selection_ready_fixture() -> Value {
    local_store_database_adapter_selection_fixture(
        "local_store_database_adapter_selection_ready",
        valid_local_store_database_adapter_selection_input(),
    )
}

fn local_store_database_adapter_selection_license_rejected_fixture() -> Value {
    let mut input = valid_local_store_database_adapter_selection_input();
    input.license_kind = LocalStoreDatabaseLicenseKind::TrialEvaluation;
    local_store_database_adapter_selection_fixture(
        "local_store_database_adapter_selection_license_rejected",
        input,
    )
}

fn local_store_database_adapter_selection_fips_rejected_fixture() -> Value {
    let mut input = valid_local_store_database_adapter_selection_input();
    input.adapter_kind = LocalStoreDatabaseAdapterKind::SqlCipherEnterpriseFips;
    input.license_kind = LocalStoreDatabaseLicenseKind::EnterpriseFips;
    input.fips_required = true;
    input.fips_module_validated = false;
    local_store_database_adapter_selection_fixture(
        "local_store_database_adapter_selection_fips_rejected",
        input,
    )
}

fn local_store_database_adapter_selection_migration_rejected_fixture() -> Value {
    let mut input = valid_local_store_database_adapter_selection_input();
    input.deterministic_migration_tested = false;
    local_store_database_adapter_selection_fixture(
        "local_store_database_adapter_selection_migration_rejected",
        input,
    )
}

fn local_store_database_adapter_selection_supply_chain_rejected_fixture() -> Value {
    let mut input = valid_local_store_database_adapter_selection_input();
    input.sbom_present = false;
    local_store_database_adapter_selection_fixture(
        "local_store_database_adapter_selection_supply_chain_rejected",
        input,
    )
}

fn local_store_database_adapter_selection_fixture(
    name: &'static str,
    input: LocalStoreDatabaseAdapterSelectionInput,
) -> Value {
    json!({
        "fixture": name,
        "surface": "local_store_database_adapter_selection",
        "input": {
            "database_security": local_store_database_security_decision_value(input.database_security),
            "adapter_kind_code": input.adapter_kind.code(),
            "adapter_kind_label": input.adapter_kind.label(),
            "binding_kind_code": input.binding_kind.code(),
            "binding_kind_label": input.binding_kind.label(),
            "target_platform_code": input.target_platform.code(),
            "target_platform_label": input.target_platform.label(),
            "license_kind_code": input.license_kind.code(),
            "license_kind_label": input.license_kind.label(),
            "sqlcipher_major_version": input.sqlcipher_major_version,
            "sqlite_source_verified": input.sqlite_source_verified,
            "sqlcipher_source_verified": input.sqlcipher_source_verified,
            "platform_package_supported": input.platform_package_supported,
            "license_allows_redistribution": input.license_allows_redistribution,
            "crypto_provider_documented": input.crypto_provider_documented,
            "fips_required": input.fips_required,
            "fips_module_validated": input.fips_module_validated,
            "fips_runtime_self_tests_available": input.fips_runtime_self_tests_available,
            "fips_mode_checked_at_runtime": input.fips_mode_checked_at_runtime,
            "compile_has_codec": input.compile_has_codec,
            "compile_has_sqlcipher_extra_init_shutdown": input.compile_has_sqlcipher_extra_init_shutdown,
            "temp_store_memory_configured": input.temp_store_memory_configured,
            "extension_loading_disabled": input.extension_loading_disabled,
            "trusted_schema_disabled": input.trusted_schema_disabled,
            "secure_delete_configured": input.secure_delete_configured,
            "cipher_memory_security_enabled": input.cipher_memory_security_enabled,
            "cipher_integrity_check_on_open": input.cipher_integrity_check_on_open,
            "sqlcipher_compatibility_current_major": input.sqlcipher_compatibility_current_major,
            "deterministic_migration_tested": input.deterministic_migration_tested,
            "crash_recovery_drill_passed": input.crash_recovery_drill_passed,
            "release_artifacts_signed": input.release_artifacts_signed,
            "sbom_present": input.sbom_present,
            "cve_monitoring_enabled": input.cve_monitoring_enabled,
            "debug_sqlcipher_logging_enabled": input.debug_sqlcipher_logging_enabled,
        },
        "decision": local_store_database_adapter_selection_decision_value(input.evaluate()),
    })
}

fn local_store_database_adapter_selection_decision_value(
    decision: LocalStoreDatabaseAdapterSelectionDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_link_adapter": decision.can_link_adapter,
        "can_open_database": decision.can_open_database,
        "can_ship_release": decision.can_ship_release,
        "can_host_mls_transactions": decision.can_host_mls_transactions,
        "requires_license_review": decision.requires_license_review,
        "requires_fips_attestation": decision.requires_fips_attestation,
        "requires_migration_drill": decision.requires_migration_drill,
        "requires_supply_chain_review": decision.requires_supply_chain_review,
        "requires_platform_packaging": decision.requires_platform_packaging,
        "forbids_plaintext_storage": decision.forbids_plaintext_storage,
        "adapter_kind_code": decision.adapter_kind_code,
        "adapter_kind_label": decision.adapter_kind_label,
        "binding_kind_code": decision.binding_kind_code,
        "binding_kind_label": decision.binding_kind_label,
        "target_platform_code": decision.target_platform_code,
        "target_platform_label": decision.target_platform_label,
        "license_kind_code": decision.license_kind_code,
        "license_kind_label": decision.license_kind_label,
        "database_security_reason": decision.database_security_reason.label(),
    })
}

fn receive_session_happy_path_fixture() -> Value {
    receive_session_fixture("receive_session_happy_path", valid_receive_session_input())
}

fn receive_session_ack_rejected_fixture() -> Value {
    let mut input = valid_receive_session_input();
    input.ack_token_len = 12;
    receive_session_fixture("receive_session_ack_rejected", input)
}

fn receive_session_ordering_gap_fixture() -> Value {
    let mut input = valid_receive_session_input();
    input.receive_replay_state = ClientReceiveReplayState::FutureGap;
    receive_session_fixture("receive_session_ordering_gap", input)
}

fn receive_session_store_write_rejected_fixture() -> Value {
    let mut input = valid_receive_session_input();
    input.store_record_kind = LocalStoreRecordKind::MessagePlaintext;
    receive_session_fixture("receive_session_store_write_rejected", input)
}

fn receive_session_fixture(name: &'static str, input: PrototypeReceiveSessionInput<'_>) -> Value {
    let mut session = PrototypeReceiveSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_receive_session",
        "input": {
            "route_id_len": input.relay_submit.route_id.len(),
            "replay_token_len": input.relay_submit.replay_token.len(),
            "ciphertext_len": input.relay_submit.ciphertext.len(),
            "sealed_header_len": input.relay_submit.sealed_header.len(),
            "delivery_now_s": input.delivery_now_s,
            "ack_seen": input.ack_seen,
            "acknowledged_at_s": input.acknowledged_at_s,
            "max_ack_delay_s": input.max_ack_delay_s,
            "ack_token_len": input.ack_token_len,
            "ciphertext_digest_len": input.ciphertext_digest_len,
            "delivery_tag_len": input.delivery_tag_len,
            "receive_replay_state": format!("{:?}", input.receive_replay_state),
            "store_record_kind": input.store_record_kind.label(),
            "plaintext_identity_fields": input.plaintext_identity_fields,
        },
        "outcome": receive_session_outcome_value(outcome),
        "events": events,
    })
}

fn receive_session_outcome_value(outcome: PrototypeReceiveSessionOutcome) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "local_store_records": outcome.local_store_records,
        "relay_items": outcome.relay_items,
        "delivered_ciphertext_len": outcome.delivered_ciphertext_len,
        "delivered_sealed_header_len": outcome.delivered_sealed_header_len,
        "plaintext_exposed": outcome.plaintext_exposed,
        "relay_submission": {
            "accepted": outcome.relay_submission.accepted,
            "reason_code": outcome.relay_submission.reason_code,
            "audit_class": outcome.relay_submission.audit_class,
        },
        "relay_queue": {
            "accepted": outcome.relay_queue.accepted,
            "next_state": format!("{:?}", outcome.relay_queue.next_state),
            "persist_item": outcome.relay_queue.persist_item,
            "delete_item": outcome.relay_queue.delete_item,
            "reason": format!("{:?}", outcome.relay_queue.reason),
        },
        "relay_delivery": outcome.relay_delivery.map(|decision| json!({
            "accepted": decision.accepted,
            "next_state": format!("{:?}", decision.next_state),
            "delete_item": decision.delete_item,
            "reason": format!("{:?}", decision.reason),
        })),
        "delivery_ack": outcome.delivery_ack.map(|decision| json!({
            "accepted": decision.accepted,
            "duplicate": decision.duplicate,
            "retain_hash_audit": decision.retain_hash_audit,
            "delete_queue_record": decision.delete_queue_record,
            "requires_client_retry": decision.requires_client_retry,
            "reason": format!("{:?}", decision.reason),
        })),
        "client_receive": outcome.client_receive.map(|decision| json!({
            "accepted": decision.accepted,
            "can_decrypt": decision.can_decrypt,
            "can_persist_ciphertext": decision.can_persist_ciphertext,
            "can_expose_to_ui": decision.can_expose_to_ui,
            "requires_client_retry": decision.requires_client_retry,
            "requires_user_action": decision.requires_user_action,
            "reason_code": decision.reason.code(),
            "reason_label": decision.reason.label(),
        })),
        "store_write": outcome.store_write.map(|decision| json!({
            "accepted": decision.accepted,
            "reason": format!("{:?}", decision.reason),
        })),
    })
}

fn inbound_sync_delivery_ready_fixture() -> Value {
    inbound_sync_fixture("inbound_sync_delivery_ready", valid_inbound_sync_input())
}

fn inbound_sync_idle_fixture() -> Value {
    let mut input = valid_inbound_sync_input();
    input.pending_delivery = false;
    input.route_id_len = 0;
    inbound_sync_fixture("inbound_sync_idle", input)
}

fn inbound_sync_bootstrap_blocked_fixture() -> Value {
    let mut input = valid_inbound_sync_input();
    input.bootstrap = blocked_inbound_sync_bootstrap();
    inbound_sync_fixture("inbound_sync_bootstrap_blocked", input)
}

fn inbound_sync_transport_offline_fixture() -> Value {
    let mut input = valid_inbound_sync_input();
    input.source_state = InboundSyncSourceState::Offline;
    inbound_sync_fixture("inbound_sync_transport_offline", input)
}

fn inbound_sync_plaintext_preview_forbidden_fixture() -> Value {
    let mut input = valid_inbound_sync_input();
    input.plaintext_notification_preview_len = 24;
    inbound_sync_fixture("inbound_sync_plaintext_preview_forbidden", input)
}

fn inbound_sync_fixture(name: &'static str, input: InboundSyncInput) -> Value {
    let decision = evaluate_inbound_sync(input);

    json!({
        "fixture": name,
        "surface": "inbound_sync_gate",
        "input": {
            "bootstrap_can_start_sync": input.bootstrap.can_start_sync,
            "bootstrap_reason_code": input.bootstrap.reason.code(),
            "bootstrap_reason_label": input.bootstrap.reason.label(),
            "source_state": format!("{:?}", input.source_state),
            "pending_delivery": input.pending_delivery,
            "route_id_len": input.route_id_len,
            "poll_batch_limit": input.poll_batch_limit,
            "plaintext_notification_preview_len": input.plaintext_notification_preview_len,
        },
        "decision": inbound_sync_decision_value(decision),
    })
}

fn inbound_sync_decision_value(decision: InboundSyncDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "can_poll_relay": decision.can_poll_relay,
        "can_run_receive_session": decision.can_run_receive_session,
        "can_update_replay_checkpoint": decision.can_update_replay_checkpoint,
        "requires_network_retry": decision.requires_network_retry,
        "requires_user_action": decision.requires_user_action,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
    })
}

fn authenticated_relay_source_delivery_ready_fixture() -> Value {
    authenticated_relay_source_fixture(
        "authenticated_relay_source_delivery_ready",
        valid_authenticated_relay_source_input(),
    )
}

fn authenticated_relay_source_idle_fixture() -> Value {
    let mut input = valid_authenticated_relay_source_input();
    input.pending_delivery = false;
    input.route_id_len = 0;
    authenticated_relay_source_fixture("authenticated_relay_source_idle", input)
}

fn authenticated_relay_source_auth_rejected_fixture() -> Value {
    let mut input = valid_authenticated_relay_source_input();
    input.server_authenticated = false;
    authenticated_relay_source_fixture("authenticated_relay_source_auth_rejected", input)
}

fn authenticated_relay_source_plaintext_forbidden_fixture() -> Value {
    let mut input = valid_authenticated_relay_source_input();
    input.plaintext_identity_fields = 1;
    authenticated_relay_source_fixture("authenticated_relay_source_plaintext_forbidden", input)
}

fn authenticated_relay_source_fixture(
    name: &'static str,
    input: AuthenticatedRelaySourceInput,
) -> Value {
    let decision = evaluate_authenticated_relay_source(input);
    let inbound_sync = decision
        .into_inbound_sync_input(accepted_inbound_sync_bootstrap())
        .evaluate();

    json!({
        "fixture": name,
        "surface": "authenticated_relay_source",
        "input": {
            "transport_code": input.transport.code(),
            "transport_label": input.transport.label(),
            "session_ticket_len": input.session_ticket_len,
            "device_credential_len": input.device_credential_len,
            "server_auth_tag_len": input.server_auth_tag_len,
            "server_authenticated": input.server_authenticated,
            "route_key_authenticated": input.route_key_authenticated,
            "replay_window_valid": input.replay_window_valid,
            "pending_delivery": input.pending_delivery,
            "route_id_len": input.route_id_len,
            "poll_batch_limit": input.poll_batch_limit,
            "plaintext_notification_preview_len": input.plaintext_notification_preview_len,
            "plaintext_identity_fields": input.plaintext_identity_fields,
        },
        "decision": authenticated_relay_source_decision_value(decision),
        "inbound_sync": inbound_sync_decision_value(inbound_sync),
    })
}

fn authenticated_relay_source_decision_value(decision: AuthenticatedRelaySourceDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "source_state": format!("{:?}", decision.source_state),
        "pending_delivery": decision.pending_delivery,
        "route_id_len": decision.route_id_len,
        "poll_batch_limit": decision.poll_batch_limit,
        "plaintext_notification_preview_len": decision.plaintext_notification_preview_len,
        "can_poll_relay": decision.can_poll_relay,
        "can_run_receive_session": decision.can_run_receive_session,
        "requires_network_retry": decision.requires_network_retry,
        "requires_user_action": decision.requires_user_action,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
    })
}

fn inbound_sync_session_happy_path_fixture() -> Value {
    inbound_sync_session_fixture(
        "inbound_sync_session_happy_path",
        valid_authenticated_inbound_sync_session_input(),
    )
}

fn inbound_sync_session_idle_fixture() -> Value {
    let mut input = valid_authenticated_inbound_sync_session_input();
    input.relay_source.pending_delivery = false;
    input.relay_source.route_id_len = 0;
    inbound_sync_session_fixture("inbound_sync_session_idle", input)
}

fn inbound_sync_session_sync_rejected_fixture() -> Value {
    let mut input = valid_authenticated_inbound_sync_session_input();
    input.bootstrap = blocked_inbound_sync_bootstrap();
    inbound_sync_session_fixture("inbound_sync_session_sync_rejected", input)
}

fn inbound_sync_session_receive_rejected_fixture() -> Value {
    let mut input = valid_authenticated_inbound_sync_session_input();
    input.receive.ack_token_len = 12;
    inbound_sync_session_fixture("inbound_sync_session_receive_rejected", input)
}

fn inbound_sync_session_fixture(
    name: &'static str,
    input: PrototypeAuthenticatedInboundSyncSessionInput<'_>,
) -> Value {
    let mut session = PrototypeInboundSyncSession::default();
    let outcome = session.run_authenticated_source(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_inbound_sync_session",
        "input": {
            "bootstrap_can_start_sync": input.bootstrap.can_start_sync,
            "bootstrap_reason_code": input.bootstrap.reason.code(),
            "bootstrap_reason_label": input.bootstrap.reason.label(),
            "relay_source_transport_label": input.relay_source.transport.label(),
            "relay_source_pending_delivery": input.relay_source.pending_delivery,
            "relay_source_route_id_len": input.relay_source.route_id_len,
            "relay_source_plaintext_notification_preview_len": input.relay_source.plaintext_notification_preview_len,
            "relay_source_plaintext_identity_fields": input.relay_source.plaintext_identity_fields,
            "receive_ack_token_len": input.receive.ack_token_len,
            "receive_replay_state": format!("{:?}", input.receive.receive_replay_state),
            "receive_store_record_kind": input.receive.store_record_kind.label(),
        },
        "relay_source": authenticated_inbound_sync_session_source_value(outcome),
        "outcome": inbound_sync_session_outcome_value(outcome.session),
        "events": events,
    })
}

fn authenticated_inbound_sync_session_source_value(
    outcome: PrototypeAuthenticatedInboundSyncSessionOutcome,
) -> Value {
    authenticated_relay_source_decision_value(outcome.relay_source)
}

fn inbound_sync_session_outcome_value(outcome: PrototypeInboundSyncSessionOutcome) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "receive_ran": outcome.receive_ran,
        "local_store_records": outcome.local_store_records,
        "relay_items": outcome.relay_items,
        "plaintext_exposed": outcome.plaintext_exposed,
        "sync": inbound_sync_decision_value(outcome.sync),
        "receive": outcome.receive.map(receive_session_outcome_value),
    })
}

fn media_object_store_upload_ready_fixture() -> Value {
    media_object_store_fixture(
        "media_object_store_upload_ready",
        valid_media_object_store_input(),
    )
}

fn media_object_store_plaintext_rejected_fixture() -> Value {
    let mut input = valid_media_object_store_input();
    input.plaintext_bytes = 1;
    media_object_store_fixture("media_object_store_plaintext_rejected", input)
}

fn media_object_store_auto_download_rejected_fixture() -> Value {
    let mut input = valid_media_object_store_input();
    input.automatic_download_requested = true;
    media_object_store_fixture("media_object_store_auto_download_rejected", input)
}

fn media_object_store_oversize_rejected_fixture() -> Value {
    let mut input = valid_media_object_store_input();
    input.ciphertext_len = MERCURY_MAX_MEDIA_OBJECT_BYTES + 1;
    media_object_store_fixture("media_object_store_oversize_rejected", input)
}

fn media_object_store_fixture(name: &'static str, input: MediaObjectStoreInput) -> Value {
    let decision = evaluate_media_object_store(input);

    json!({
        "fixture": name,
        "surface": "media_object_store",
        "input": {
            "outbound_send_accepted": input.outbound_send.accepted,
            "outbound_send_reason_code": input.outbound_send.reason.code(),
            "outbound_send_reason_label": input.outbound_send.reason.label(),
            "media_sealing_accepted": input.media_sealing.accepted,
            "media_record_kind": input.media_sealing.record_policy.kind.label(),
            "object_id_len": input.object_id_len,
            "ciphertext_len": input.ciphertext_len,
            "max_ciphertext_len": input.max_ciphertext_len,
            "sealed_header_len": input.sealed_header_len,
            "content_digest_len": input.content_digest_len,
            "media_key_commitment_len": input.media_key_commitment_len,
            "plaintext_bytes": input.plaintext_bytes,
            "automatic_download_requested": input.automatic_download_requested,
        },
        "decision": media_object_store_decision_value(decision),
    })
}

fn media_object_store_decision_value(decision: MediaObjectStoreDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "can_upload": decision.can_upload,
        "can_persist_local_ciphertext": decision.can_persist_local_ciphertext,
        "requires_user_action": decision.requires_user_action,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
    })
}

fn media_upload_session_happy_path_fixture() -> Value {
    media_upload_session_fixture(
        "media_upload_session_happy_path",
        valid_media_upload_session_input(),
    )
}

fn media_upload_session_plaintext_rejected_fixture() -> Value {
    let mut input = valid_media_upload_session_input();
    input.plaintext_upload_bytes = MEDIA_UPLOAD_PLAINTEXT.len() as i32;
    media_upload_session_fixture("media_upload_session_plaintext_rejected", input)
}

fn media_upload_session_seal_rejected_fixture() -> Value {
    let mut input = valid_media_upload_session_input();
    input.seal_request = seal_request(
        LocalStoreRecordKind::MediaPlaintext,
        MEDIA_UPLOAD_PLAINTEXT.len() as i32,
        Some(store_policy_decision(true)),
    );
    media_upload_session_fixture("media_upload_session_seal_rejected", input)
}

fn media_upload_session_store_write_rejected_fixture() -> Value {
    let mut input = valid_media_upload_session_input();
    input.store_record_kind = LocalStoreRecordKind::MediaPlaintext;
    media_upload_session_fixture("media_upload_session_store_write_rejected", input)
}

fn media_upload_session_fixture(
    name: &'static str,
    input: PrototypeMediaUploadSessionInput<'_>,
) -> Value {
    let mut session = PrototypeMediaUploadSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_media_upload_session",
        "input": {
            "seal_record_kind": input.seal_request.record_kind.label(),
            "plaintext_len": input.plaintext.len(),
            "outbound_send_accepted": input.outbound_send.accepted,
            "outbound_send_reason_code": input.outbound_send.reason.code(),
            "outbound_send_reason_label": input.outbound_send.reason.label(),
            "object_id_len": input.object_id.len(),
            "max_ciphertext_len": input.max_ciphertext_len,
            "sealed_header_len": input.sealed_header.len(),
            "content_digest_len": input.content_digest.len(),
            "media_key_commitment_len": input.media_key_commitment.len(),
            "plaintext_upload_bytes": input.plaintext_upload_bytes,
            "automatic_download_requested": input.automatic_download_requested,
            "store_record_kind": input.store_record_kind.label(),
        },
        "outcome": media_upload_session_outcome_value(outcome),
        "events": events,
    })
}

fn media_upload_session_outcome_value(outcome: PrototypeMediaUploadSessionOutcome) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "local_store_records": outcome.local_store_records,
        "sealed_ciphertext_len": outcome.sealed_ciphertext_len,
        "stored_ciphertext_len": outcome.stored_ciphertext_len,
        "crypto_seal_calls": outcome.crypto_seal_calls,
        "plaintext_exposed": outcome.plaintext_exposed,
        "seal": outcome.seal.map(|decision| json!({
            "accepted": decision.accepted,
            "reason": format!("{:?}", decision.reason),
            "record_kind": decision.record_policy.kind.label(),
        })),
        "media": outcome.media.map(media_object_store_decision_value),
        "store_write": outcome.store_write.map(|decision| json!({
            "accepted": decision.accepted,
            "reason": format!("{:?}", decision.reason),
            "record_kind": decision.record_policy.kind.label(),
        })),
    })
}

fn media_service_adapter_ready_fixture() -> Value {
    media_service_adapter_fixture(
        "media_service_adapter_ready",
        valid_media_service_adapter_input(),
    )
}

fn media_service_adapter_auth_missing_fixture() -> Value {
    let mut input = valid_media_service_adapter_input();
    input.service_authenticated = false;
    media_service_adapter_fixture("media_service_adapter_auth_missing", input)
}

fn media_service_adapter_plaintext_forbidden_fixture() -> Value {
    let mut input = valid_media_service_adapter_input();
    input.adapter_kind = MediaServiceAdapterKind::PlaintextDebugStore;
    media_service_adapter_fixture("media_service_adapter_plaintext_forbidden", input)
}

fn media_service_adapter_digest_unverified_fixture() -> Value {
    let mut input = valid_media_service_adapter_input();
    input.content_digest_verified = false;
    media_service_adapter_fixture("media_service_adapter_digest_unverified", input)
}

fn media_service_adapter_fixture(name: &'static str, input: MediaServiceAdapterInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "media_service_adapter",
        "input": {
            "adapter_kind_code": input.adapter_kind.code(),
            "adapter_kind_label": input.adapter_kind.label(),
            "media_object_store_accepted": input.media_object_store.accepted,
            "media_object_store_reason_code": input.media_object_store.reason.code(),
            "media_object_store_reason_label": input.media_object_store.reason.label(),
            "service_authenticated": input.service_authenticated,
            "upload_authorized": input.upload_authorized,
            "object_namespace_bound": input.object_namespace_bound,
            "content_digest_verified": input.content_digest_verified,
            "allow_development_adapter": input.allow_development_adapter,
        },
        "decision": media_service_adapter_decision_value(decision),
    })
}

fn media_service_adapter_decision_value(decision: MediaServiceAdapterDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "can_upload_object": decision.can_upload_object,
        "can_persist_remote_ciphertext": decision.can_persist_remote_ciphertext,
        "requires_network_setup": decision.requires_network_setup,
        "requires_user_action": decision.requires_user_action,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "forbids_plaintext_upload": decision.forbids_plaintext_upload,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
    })
}

fn media_service_upload_session_happy_path_fixture() -> Value {
    media_service_upload_session_fixture(
        "media_service_upload_session_happy_path",
        valid_media_service_upload_session_input(),
    )
}

fn media_service_upload_session_media_rejected_fixture() -> Value {
    let mut input = valid_media_service_upload_session_input();
    input.media_upload.plaintext_upload_bytes = MEDIA_UPLOAD_PLAINTEXT.len() as i32;
    media_service_upload_session_fixture("media_service_upload_session_media_rejected", input)
}

fn media_service_upload_session_auth_rejected_fixture() -> Value {
    let mut input = valid_media_service_upload_session_input();
    input.service_authenticated = false;
    media_service_upload_session_fixture("media_service_upload_session_auth_rejected", input)
}

fn media_service_upload_session_digest_unverified_fixture() -> Value {
    let mut input = valid_media_service_upload_session_input();
    input.content_digest_verified = false;
    media_service_upload_session_fixture("media_service_upload_session_digest_unverified", input)
}

fn media_service_upload_session_fixture(
    name: &'static str,
    input: PrototypeMediaServiceUploadSessionInput<'_>,
) -> Value {
    let mut session = PrototypeMediaServiceUploadSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_media_service_upload_session",
        "input": {
            "media_plaintext_len": input.media_upload.plaintext.len(),
            "media_plaintext_upload_bytes": input.media_upload.plaintext_upload_bytes,
            "media_store_record_kind": input.media_upload.store_record_kind.label(),
            "adapter_kind_code": input.adapter_kind.code(),
            "adapter_kind_label": input.adapter_kind.label(),
            "service_authenticated": input.service_authenticated,
            "upload_authorized": input.upload_authorized,
            "object_namespace_bound": input.object_namespace_bound,
            "content_digest_verified": input.content_digest_verified,
            "allow_development_adapter": input.allow_development_adapter,
        },
        "outcome": media_service_upload_session_outcome_value(outcome),
        "events": events,
    })
}

fn media_service_upload_session_outcome_value(
    outcome: PrototypeMediaServiceUploadSessionOutcome,
) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "local_store_records": outcome.local_store_records,
        "sealed_ciphertext_len": outcome.sealed_ciphertext_len,
        "stored_ciphertext_len": outcome.stored_ciphertext_len,
        "service_upload_calls": outcome.service_upload_calls,
        "plaintext_exposed": outcome.plaintext_exposed,
        "media_upload": media_upload_session_outcome_value(outcome.media_upload),
        "media_service": outcome.media_service.map(media_service_adapter_decision_value),
    })
}

fn media_service_download_ready_fixture() -> Value {
    media_service_download_fixture(
        "media_service_download_ready",
        valid_media_service_download_input(4096),
    )
}

fn media_service_download_plaintext_preview_rejected_fixture() -> Value {
    let mut input = valid_media_service_download_input(4096);
    input.plaintext_preview_bytes = 1;
    media_service_download_fixture("media_service_download_plaintext_preview_rejected", input)
}

fn media_service_download_auth_missing_fixture() -> Value {
    let mut input = valid_media_service_download_input(4096);
    input.service_authenticated = false;
    media_service_download_fixture("media_service_download_auth_missing", input)
}

fn media_service_download_digest_unverified_fixture() -> Value {
    let mut input = valid_media_service_download_input(4096);
    input.content_digest_verified = false;
    media_service_download_fixture("media_service_download_digest_unverified", input)
}

fn media_service_download_fixture(name: &'static str, input: MediaServiceDownloadInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "media_service_download",
        "input": {
            "adapter_kind_code": input.adapter_kind.code(),
            "adapter_kind_label": input.adapter_kind.label(),
            "service_authenticated": input.service_authenticated,
            "download_authorized": input.download_authorized,
            "object_namespace_bound": input.object_namespace_bound,
            "content_digest_verified": input.content_digest_verified,
            "allow_development_adapter": input.allow_development_adapter,
            "object_id_len": input.object_id_len,
            "ciphertext_len": input.ciphertext_len,
            "max_ciphertext_len": input.max_ciphertext_len,
            "sealed_header_len": input.sealed_header_len,
            "content_digest_len": input.content_digest_len,
            "media_key_commitment_len": input.media_key_commitment_len,
            "plaintext_preview_bytes": input.plaintext_preview_bytes,
            "automatic_download_requested": input.automatic_download_requested,
        },
        "decision": media_service_download_decision_value(decision),
    })
}

fn media_service_download_decision_value(decision: MediaServiceDownloadDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "can_download_object": decision.can_download_object,
        "can_persist_local_ciphertext": decision.can_persist_local_ciphertext,
        "requires_network_setup": decision.requires_network_setup,
        "requires_user_action": decision.requires_user_action,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "forbids_plaintext_preview": decision.forbids_plaintext_preview,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
    })
}

fn media_download_session_happy_path_fixture() -> Value {
    let sealed = media_download_session_sealed_media();
    media_download_session_fixture(
        "media_download_session_happy_path",
        valid_media_download_session_input(
            &sealed.sealed_bytes,
            &sealed.nonce,
            sealed.authentication_tag_len,
        ),
    )
}

fn media_download_session_download_rejected_fixture() -> Value {
    let sealed = media_download_session_sealed_media();
    let mut input = valid_media_download_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.download.plaintext_preview_bytes = 1;
    media_download_session_fixture("media_download_session_download_rejected", input)
}

fn media_download_session_store_write_rejected_fixture() -> Value {
    let sealed = media_download_session_sealed_media();
    let mut input = valid_media_download_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.store_record_kind = LocalStoreRecordKind::MediaPlaintext;
    media_download_session_fixture("media_download_session_store_write_rejected", input)
}

fn media_download_session_open_rejected_fixture() -> Value {
    let sealed = media_download_session_sealed_media();
    let bad_nonce = [7_u8; 12];
    media_download_session_fixture(
        "media_download_session_open_rejected",
        valid_media_download_session_input(
            &sealed.sealed_bytes,
            &bad_nonce,
            sealed.authentication_tag_len,
        ),
    )
}

fn media_download_session_fixture(
    name: &'static str,
    input: PrototypeMediaDownloadSessionInput<'_>,
) -> Value {
    let mut session = PrototypeMediaDownloadSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_media_download_session",
        "input": {
            "adapter_kind_code": input.download.adapter_kind.code(),
            "adapter_kind_label": input.download.adapter_kind.label(),
            "download_authorized": input.download.download_authorized,
            "service_authenticated": input.download.service_authenticated,
            "object_namespace_bound": input.download.object_namespace_bound,
            "content_digest_verified": input.download.content_digest_verified,
            "downloaded_ciphertext_len": input.downloaded_ciphertext.len(),
            "store_record_kind": input.store_record_kind.label(),
            "nonce_len": input.nonce.len(),
            "authentication_tag_len": input.authentication_tag_len,
            "plaintext_preview_bytes": input.download.plaintext_preview_bytes,
            "automatic_download_requested": input.download.automatic_download_requested,
        },
        "outcome": media_download_session_outcome_value(outcome),
        "events": events,
    })
}

fn media_download_session_outcome_value(outcome: PrototypeMediaDownloadSessionOutcome) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "local_store_records": outcome.local_store_records,
        "stored_ciphertext_len": outcome.stored_ciphertext_len,
        "opened_plaintext_len": outcome.opened_plaintext_len,
        "service_download_calls": outcome.service_download_calls,
        "crypto_open_calls": outcome.crypto_open_calls,
        "plaintext_exposed": outcome.plaintext_exposed,
        "download": media_service_download_decision_value(outcome.download),
        "store_write": outcome.store_write.map(|decision| json!({
            "accepted": decision.accepted,
            "reason": format!("{:?}", decision.reason),
            "record_kind": decision.record_policy.kind.label(),
        })),
        "open": outcome.open.map(|decision| json!({
            "accepted": decision.accepted,
            "reason": format!("{:?}", decision.reason),
            "record_kind": decision.record_policy.kind.label(),
        })),
    })
}

fn media_retention_delete_and_evict_ready_fixture() -> Value {
    media_retention_fixture(
        "media_retention_delete_and_evict_ready",
        valid_media_retention_input(),
    )
}

fn media_retention_retain_ready_fixture() -> Value {
    let mut input = valid_media_retention_input();
    input.operation = MediaRetentionOperation::Retain;
    input.service_authenticated = false;
    input.delete_authorized = false;
    media_retention_fixture("media_retention_retain_ready", input)
}

fn media_retention_hold_rejected_fixture() -> Value {
    let mut input = valid_media_retention_input();
    input.retention_hold_active = true;
    media_retention_fixture("media_retention_hold_rejected", input)
}

fn media_retention_auth_missing_fixture() -> Value {
    let mut input = valid_media_retention_input();
    input.service_authenticated = false;
    media_retention_fixture("media_retention_auth_missing", input)
}

fn media_retention_fixture(name: &'static str, input: MediaRetentionInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "media_retention",
        "input": {
            "operation_code": input.operation.code(),
            "operation_label": input.operation.label(),
            "adapter_kind_code": input.adapter_kind.code(),
            "adapter_kind_label": input.adapter_kind.label(),
            "record_kind": input.record_kind.label(),
            "service_authenticated": input.service_authenticated,
            "delete_authorized": input.delete_authorized,
            "object_namespace_bound": input.object_namespace_bound,
            "content_digest_verified": input.content_digest_verified,
            "allow_development_adapter": input.allow_development_adapter,
            "user_delete_requested": input.user_delete_requested,
            "cache_eviction_requested": input.cache_eviction_requested,
            "retention_hold_active": input.retention_hold_active,
            "object_id_len": input.object_id_len,
            "content_digest_len": input.content_digest_len,
            "plaintext_bytes": input.plaintext_bytes,
        },
        "decision": media_retention_decision_value(decision),
    })
}

fn media_retention_decision_value(decision: MediaRetentionDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "operation_code": decision.operation.code(),
        "operation_label": decision.operation.label(),
        "can_delete_remote_object": decision.can_delete_remote_object,
        "can_evict_local_cache": decision.can_evict_local_cache,
        "keeps_audit_hash": decision.keeps_audit_hash,
        "requires_network_setup": decision.requires_network_setup,
        "requires_user_action": decision.requires_user_action,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "forbids_plaintext_deletion": decision.forbids_plaintext_deletion,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
    })
}

fn media_cleanup_session_happy_path_fixture() -> Value {
    media_cleanup_session_fixture(
        "media_cleanup_session_happy_path",
        valid_media_cleanup_session_input(true),
    )
}

fn media_cleanup_session_retain_ready_fixture() -> Value {
    let mut input = valid_media_cleanup_session_input(true);
    input.retention.operation = MediaRetentionOperation::Retain;
    input.retention.service_authenticated = false;
    input.retention.delete_authorized = false;
    media_cleanup_session_fixture("media_cleanup_session_retain_ready", input)
}

fn media_cleanup_session_retention_rejected_fixture() -> Value {
    let mut input = valid_media_cleanup_session_input(true);
    input.retention.retention_hold_active = true;
    media_cleanup_session_fixture("media_cleanup_session_retention_rejected", input)
}

fn media_cleanup_session_cache_absent_fixture() -> Value {
    let mut input = valid_media_cleanup_session_input(false);
    input.retention.operation = MediaRetentionOperation::EvictLocalCache;
    input.retention.user_delete_requested = false;
    input.retention.cache_eviction_requested = true;
    input.retention.service_authenticated = false;
    input.retention.delete_authorized = false;
    media_cleanup_session_fixture("media_cleanup_session_cache_absent", input)
}

fn media_cleanup_session_fixture(
    name: &'static str,
    input: PrototypeMediaCleanupSessionInput<'_>,
) -> Value {
    let mut session = PrototypeMediaCleanupSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_media_cleanup_session",
        "input": {
            "operation_code": input.retention.operation.code(),
            "operation_label": input.retention.operation.label(),
            "adapter_kind_code": input.retention.adapter_kind.code(),
            "adapter_kind_label": input.retention.adapter_kind.label(),
            "record_kind": input.retention.record_kind.label(),
            "service_authenticated": input.retention.service_authenticated,
            "delete_authorized": input.retention.delete_authorized,
            "object_namespace_bound": input.retention.object_namespace_bound,
            "content_digest_verified": input.retention.content_digest_verified,
            "user_delete_requested": input.retention.user_delete_requested,
            "cache_eviction_requested": input.retention.cache_eviction_requested,
            "retention_hold_active": input.retention.retention_hold_active,
            "seed_local_cache": input.seed_local_cache,
            "cached_ciphertext_len": input.cached_ciphertext.len(),
        },
        "outcome": media_cleanup_session_outcome_value(outcome),
        "events": events,
    })
}

fn media_cleanup_session_outcome_value(outcome: PrototypeMediaCleanupSessionOutcome) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "remote_delete_calls": outcome.remote_delete_calls,
        "local_cache_delete_attempted": outcome.local_cache_delete_attempted,
        "local_cache_deleted": outcome.local_cache_deleted,
        "local_store_records": outcome.local_store_records,
        "seeded_cache_ciphertext_len": outcome.seeded_cache_ciphertext_len,
        "plaintext_exposed": outcome.plaintext_exposed,
        "retention": media_retention_decision_value(outcome.retention),
    })
}

fn media_object_index_remote_and_local_ready_fixture() -> Value {
    media_object_index_fixture(
        "media_object_index_remote_and_local_ready",
        valid_media_object_index_input(),
    )
}

fn media_object_index_absent_upload_ready_fixture() -> Value {
    let mut input = valid_media_object_index_input();
    input.lifecycle_state = MediaObjectLifecycleState::Absent;
    input.local_cache_present = false;
    input.remote_object_present = false;
    input.remote_service_record_present = false;
    media_object_index_fixture("media_object_index_absent_upload_ready", input)
}

fn media_object_index_delete_pending_ready_fixture() -> Value {
    let mut input = valid_media_object_index_input();
    input.lifecycle_state = MediaObjectLifecycleState::DeletePending;
    input.local_cache_present = false;
    input.remote_object_present = true;
    input.remote_service_record_present = true;
    media_object_index_fixture("media_object_index_delete_pending_ready", input)
}

fn media_object_index_deleted_terminal_fixture() -> Value {
    let mut input = valid_media_object_index_input();
    input.lifecycle_state = MediaObjectLifecycleState::Deleted;
    input.local_cache_present = false;
    input.remote_object_present = false;
    input.remote_service_record_present = false;
    media_object_index_fixture("media_object_index_deleted_terminal", input)
}

fn media_object_index_plaintext_metadata_rejected_fixture() -> Value {
    let mut input = valid_media_object_index_input();
    input.plaintext_metadata_bytes = 1;
    media_object_index_fixture("media_object_index_plaintext_metadata_rejected", input)
}

fn media_object_index_bad_lifecycle_rejected_fixture() -> Value {
    let mut input = valid_media_object_index_input();
    input.lifecycle_state = MediaObjectLifecycleState::RemoteStored;
    input.local_cache_present = true;
    input.remote_object_present = true;
    media_object_index_fixture("media_object_index_bad_lifecycle_rejected", input)
}

fn media_object_index_fixture(name: &'static str, input: MediaObjectIndexInput) -> Value {
    json!({
        "fixture": name,
        "surface": "media_object_index",
        "input": {
            "lifecycle_state_code": input.lifecycle_state.code(),
            "lifecycle_state_label": input.lifecycle_state.label(),
            "record_kind": input.record_kind.label(),
            "object_id_len": input.object_id_len,
            "content_digest_len": input.content_digest_len,
            "media_key_commitment_len": input.media_key_commitment_len,
            "ciphertext_len": input.ciphertext_len,
            "max_ciphertext_len": input.max_ciphertext_len,
            "plaintext_metadata_bytes": input.plaintext_metadata_bytes,
            "content_digest_verified": input.content_digest_verified,
            "local_cache_present": input.local_cache_present,
            "remote_object_present": input.remote_object_present,
            "remote_service_record_present": input.remote_service_record_present,
            "retention_hold_active": input.retention_hold_active,
        },
        "decision": media_object_index_decision_value(input.evaluate()),
    })
}

fn media_object_index_decision_value(decision: MediaObjectIndexDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "lifecycle_state_code": decision.lifecycle_state.code(),
        "lifecycle_state_label": decision.lifecycle_state.label(),
        "can_upload": decision.can_upload,
        "can_download": decision.can_download,
        "can_cleanup": decision.can_cleanup,
        "has_local_cache": decision.has_local_cache,
        "has_remote_object": decision.has_remote_object,
        "keeps_audit_hash": decision.keeps_audit_hash,
        "requires_user_action": decision.requires_user_action,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
    })
}

fn media_object_index_store_write_ready_fixture() -> Value {
    media_object_index_store_fixture(
        "media_object_index_store_write_ready",
        valid_media_object_index_store_write(valid_media_object_index_input()),
    )
}

fn media_object_index_store_index_rejected_fixture() -> Value {
    let mut input = valid_media_object_index_input();
    input.plaintext_metadata_bytes = 1;
    media_object_index_store_fixture(
        "media_object_index_store_index_rejected",
        valid_media_object_index_store_write(input),
    )
}

fn media_object_index_store_bad_object_rejected_fixture() -> Value {
    media_object_index_store_fixture(
        "media_object_index_store_bad_object_rejected",
        MediaObjectIndexStoreWrite {
            object_id: &[13; 16],
            content_digest: &MEDIA_CONTENT_DIGEST,
            media_key_commitment: &MEDIA_KEY_COMMITMENT,
            index: valid_media_object_index_input(),
        },
    )
}

fn media_object_index_store_deleted_snapshot_fixture() -> Value {
    let mut input = valid_media_object_index_input();
    input.lifecycle_state = MediaObjectLifecycleState::Deleted;
    input.local_cache_present = false;
    input.remote_object_present = false;
    input.remote_service_record_present = false;
    media_object_index_store_fixture(
        "media_object_index_store_deleted_snapshot",
        valid_media_object_index_store_write(input),
    )
}

fn media_object_index_store_fixture(
    name: &'static str,
    write: MediaObjectIndexStoreWrite<'_>,
) -> Value {
    let mut store = PrototypeMediaObjectIndexStore::default();
    let decision = store.write(write);
    let record = store.get(write.object_id).map(|record| {
        json!({
            "object_id_len": record.object_id.len(),
            "content_digest_len": record.content_digest.len(),
            "media_key_commitment_len": record.media_key_commitment.len(),
            "lifecycle_state_code": record.lifecycle_state.code(),
            "lifecycle_state_label": record.lifecycle_state.label(),
            "ciphertext_len": record.ciphertext_len,
            "has_local_cache": record.has_local_cache,
            "has_remote_object": record.has_remote_object,
            "content_digest_verified": record.content_digest_verified,
            "retention_hold_active": record.retention_hold_active,
            "plaintext_bytes_exposed": record.plaintext_bytes_exposed,
        })
    });

    json!({
        "fixture": name,
        "surface": "media_object_index_store",
        "input": {
            "object_id_len": write.object_id.len(),
            "content_digest_len": write.content_digest.len(),
            "media_key_commitment_len": write.media_key_commitment.len(),
            "index_lifecycle_state_code": write.index.lifecycle_state.code(),
            "index_lifecycle_state_label": write.index.lifecycle_state.label(),
            "index_plaintext_metadata_bytes": write.index.plaintext_metadata_bytes,
            "index_local_cache_present": write.index.local_cache_present,
            "index_remote_object_present": write.index.remote_object_present,
            "index_remote_service_record_present": write.index.remote_service_record_present,
        },
        "decision": media_object_index_store_decision_value(decision),
        "record": record,
    })
}

fn media_object_index_store_decision_value(decision: MediaObjectIndexStoreDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "persisted_record": decision.persisted_record,
        "record_count": decision.record_count,
        "keeps_audit_hash": decision.keeps_audit_hash,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "index": media_object_index_decision_value(decision.index),
    })
}

fn media_object_index_production_open_ready_fixture() -> Value {
    media_object_index_production_open_fixture(
        "media_object_index_production_open_ready",
        valid_media_object_index_production_open_input(),
    )
}

fn media_object_index_production_open_wal_replay_required_fixture() -> Value {
    let mut input = valid_media_object_index_production_open_input();
    input.crash_recovery = LocalStoreCrashRecoveryState::WalReplayRequired;
    media_object_index_production_open_fixture(
        "media_object_index_production_open_wal_replay_required",
        input,
    )
}

fn media_object_index_production_open_plaintext_metadata_forbidden_fixture() -> Value {
    let mut input = valid_media_object_index_production_open_input();
    input.plaintext_metadata_rows = 1;
    media_object_index_production_open_fixture(
        "media_object_index_production_open_plaintext_metadata_forbidden",
        input,
    )
}

fn media_object_index_production_open_namespace_unbound_fixture() -> Value {
    let mut input = valid_media_object_index_production_open_input();
    input.object_namespace_bound = false;
    media_object_index_production_open_fixture(
        "media_object_index_production_open_namespace_unbound",
        input,
    )
}

fn media_object_index_production_open_fixture(
    name: &'static str,
    input: MediaObjectIndexProductionOpenInput,
) -> Value {
    let decision = input.evaluate();
    let header_suite_label = LocalStoreSealingSuite::from_code(input.header_suite_code)
        .map(|suite| suite.label())
        .unwrap_or("unknown");

    json!({
        "fixture": name,
        "surface": "media_object_index_production_open",
        "input": {
            "index_version": input.index_version,
            "header": {
                "header_magic_matches": input.header_magic_matches,
                "header_suite_code": input.header_suite_code,
                "header_suite_label": header_suite_label,
                "header_nonce_len": input.header_nonce_len,
                "header_tag_len": input.header_tag_len,
            },
            "manifest": {
                "plaintext_metadata_rows": input.plaintext_metadata_rows,
                "plaintext_cache_paths": input.plaintext_cache_paths,
                "object_id_index_present": input.object_id_index_present,
                "content_digest_index_present": input.content_digest_index_present,
                "lifecycle_index_present": input.lifecycle_index_present,
                "object_namespace_bound": input.object_namespace_bound,
                "media_service_authenticated": input.media_service_authenticated,
                "crash_recovery": input.crash_recovery.label(),
            },
        },
        "decision": media_object_index_production_open_decision_value(decision),
    })
}

fn media_object_index_production_open_decision_value(
    decision: MediaObjectIndexProductionOpenDecision,
) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "can_open_index": decision.can_open_index,
        "can_replay_wal": decision.can_replay_wal,
        "can_load_manifests": decision.can_load_manifests,
        "can_write_manifests": decision.can_write_manifests,
        "can_use_remote_objects": decision.can_use_remote_objects,
        "requires_network_setup": decision.requires_network_setup,
        "requires_migration": decision.requires_migration,
        "requires_crash_recovery": decision.requires_crash_recovery,
        "requires_destructive_repair": decision.requires_destructive_repair,
    })
}

fn indexed_media_upload_session_happy_path_fixture() -> Value {
    indexed_media_upload_session_fixture(
        "indexed_media_upload_session_happy_path",
        valid_indexed_media_upload_session_input(),
    )
}

fn indexed_media_upload_session_service_rejected_fixture() -> Value {
    let mut input = valid_indexed_media_upload_session_input();
    input.service_upload.media_upload.plaintext_upload_bytes = MEDIA_UPLOAD_PLAINTEXT.len() as i32;
    indexed_media_upload_session_fixture("indexed_media_upload_session_service_rejected", input)
}

fn indexed_media_upload_session_index_store_rejected_fixture() -> Value {
    let mut input = valid_indexed_media_upload_session_input();
    input.index_store.index.plaintext_metadata_bytes = 1;
    indexed_media_upload_session_fixture("indexed_media_upload_session_index_store_rejected", input)
}

fn indexed_media_upload_session_fixture(
    name: &'static str,
    input: PrototypeIndexedMediaUploadSessionInput<'_>,
) -> Value {
    let mut session = PrototypeIndexedMediaUploadSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_indexed_media_upload_session",
        "input": {
            "media_plaintext_len": input.service_upload.media_upload.plaintext.len(),
            "media_plaintext_upload_bytes": input
                .service_upload
                .media_upload
                .plaintext_upload_bytes,
            "service_authenticated": input.service_upload.service_authenticated,
            "upload_authorized": input.service_upload.upload_authorized,
            "content_digest_verified": input.service_upload.content_digest_verified,
            "index_object_id_len": input.index_store.object_id.len(),
            "index_content_digest_len": input.index_store.content_digest.len(),
            "index_media_key_commitment_len": input.index_store.media_key_commitment.len(),
            "index_lifecycle_state_code": input.index_store.index.lifecycle_state.code(),
            "index_lifecycle_state_label": input.index_store.index.lifecycle_state.label(),
            "index_plaintext_metadata_bytes": input.index_store.index.plaintext_metadata_bytes,
        },
        "outcome": indexed_media_upload_session_outcome_value(outcome),
        "events": events,
    })
}

fn indexed_media_upload_session_outcome_value(
    outcome: PrototypeIndexedMediaUploadSessionOutcome,
) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "index_store_records": outcome.index_store_records,
        "plaintext_exposed": outcome.plaintext_exposed,
        "service_upload": media_service_upload_session_outcome_value(outcome.service_upload),
        "index_store": outcome.index_store.map(media_object_index_store_decision_value),
    })
}

fn indexed_media_download_session_happy_path_fixture() -> Value {
    let sealed = media_download_session_sealed_media();
    indexed_media_download_session_fixture(
        "indexed_media_download_session_happy_path",
        valid_indexed_media_download_session_input(
            &sealed.sealed_bytes,
            &sealed.nonce,
            sealed.authentication_tag_len,
        ),
    )
}

fn indexed_media_download_session_manifest_rejected_fixture() -> Value {
    let sealed = media_download_session_sealed_media();
    let mut input = valid_indexed_media_download_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.index_store.index.plaintext_metadata_bytes = 1;
    indexed_media_download_session_fixture(
        "indexed_media_download_session_manifest_rejected",
        input,
    )
}

fn indexed_media_download_session_not_downloadable_fixture() -> Value {
    let sealed = media_download_session_sealed_media();
    let mut input = valid_indexed_media_download_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.index_store.index.lifecycle_state = MediaObjectLifecycleState::Absent;
    input.index_store.index.local_cache_present = false;
    input.index_store.index.remote_object_present = false;
    input.index_store.index.remote_service_record_present = false;
    indexed_media_download_session_fixture("indexed_media_download_session_not_downloadable", input)
}

fn indexed_media_download_session_download_rejected_fixture() -> Value {
    let sealed = media_download_session_sealed_media();
    let mut input = valid_indexed_media_download_session_input(
        &sealed.sealed_bytes,
        &sealed.nonce,
        sealed.authentication_tag_len,
    );
    input.download.download.plaintext_preview_bytes = 1;
    indexed_media_download_session_fixture(
        "indexed_media_download_session_download_rejected",
        input,
    )
}

fn indexed_media_download_session_fixture(
    name: &'static str,
    input: PrototypeIndexedMediaDownloadSessionInput<'_>,
) -> Value {
    let mut session = PrototypeIndexedMediaDownloadSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_indexed_media_download_session",
        "input": {
            "index_object_id_len": input.index_store.object_id.len(),
            "index_content_digest_len": input.index_store.content_digest.len(),
            "index_media_key_commitment_len": input.index_store.media_key_commitment.len(),
            "index_lifecycle_state_code": input.index_store.index.lifecycle_state.code(),
            "index_lifecycle_state_label": input.index_store.index.lifecycle_state.label(),
            "index_plaintext_metadata_bytes": input.index_store.index.plaintext_metadata_bytes,
            "download_ciphertext_len": input.download.downloaded_ciphertext.len(),
            "download_plaintext_preview_bytes": input.download.download.plaintext_preview_bytes,
            "download_automatic_requested": input.download.download.automatic_download_requested,
            "store_record_kind": input.download.store_record_kind.label(),
        },
        "outcome": indexed_media_download_session_outcome_value(outcome),
        "events": events,
    })
}

fn indexed_media_download_session_outcome_value(
    outcome: PrototypeIndexedMediaDownloadSessionOutcome,
) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "index_store_records": outcome.index_store_records,
        "plaintext_exposed": outcome.plaintext_exposed,
        "index_store": media_object_index_store_decision_value(outcome.index_store),
        "download": outcome.download.map(media_download_session_outcome_value),
    })
}

fn indexed_media_cleanup_session_happy_path_fixture() -> Value {
    indexed_media_cleanup_session_fixture(
        "indexed_media_cleanup_session_happy_path",
        valid_indexed_media_cleanup_session_input(true),
    )
}

fn indexed_media_cleanup_session_manifest_rejected_fixture() -> Value {
    let mut input = valid_indexed_media_cleanup_session_input(true);
    input.index_store.index.plaintext_metadata_bytes = 1;
    indexed_media_cleanup_session_fixture("indexed_media_cleanup_session_manifest_rejected", input)
}

fn indexed_media_cleanup_session_not_cleanable_fixture() -> Value {
    let mut input = valid_indexed_media_cleanup_session_input(true);
    input.index_store.index.lifecycle_state = MediaObjectLifecycleState::Absent;
    input.index_store.index.local_cache_present = false;
    input.index_store.index.remote_object_present = false;
    input.index_store.index.remote_service_record_present = false;
    indexed_media_cleanup_session_fixture("indexed_media_cleanup_session_not_cleanable", input)
}

fn indexed_media_cleanup_session_cleanup_rejected_fixture() -> Value {
    let mut input = valid_indexed_media_cleanup_session_input(true);
    input.cleanup.retention.retention_hold_active = true;
    indexed_media_cleanup_session_fixture("indexed_media_cleanup_session_cleanup_rejected", input)
}

fn indexed_media_cleanup_session_fixture(
    name: &'static str,
    input: PrototypeIndexedMediaCleanupSessionInput<'_>,
) -> Value {
    let mut session = PrototypeIndexedMediaCleanupSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": name,
        "surface": "prototype_indexed_media_cleanup_session",
        "input": {
            "index_object_id_len": input.index_store.object_id.len(),
            "index_content_digest_len": input.index_store.content_digest.len(),
            "index_media_key_commitment_len": input.index_store.media_key_commitment.len(),
            "index_lifecycle_state_code": input.index_store.index.lifecycle_state.code(),
            "index_lifecycle_state_label": input.index_store.index.lifecycle_state.label(),
            "index_plaintext_metadata_bytes": input.index_store.index.plaintext_metadata_bytes,
            "cleanup_operation_code": input.cleanup.retention.operation.code(),
            "cleanup_operation_label": input.cleanup.retention.operation.label(),
            "cleanup_retention_hold_active": input.cleanup.retention.retention_hold_active,
            "cleanup_seed_local_cache": input.cleanup.seed_local_cache,
            "cleanup_cached_ciphertext_len": input.cleanup.cached_ciphertext.len(),
        },
        "outcome": indexed_media_cleanup_session_outcome_value(outcome),
        "events": events,
    })
}

fn indexed_media_cleanup_session_outcome_value(
    outcome: PrototypeIndexedMediaCleanupSessionOutcome,
) -> Value {
    json!({
        "completed": outcome.completed,
        "reason_code": outcome.reason.code(),
        "reason_label": outcome.reason.label(),
        "index_store_records": outcome.index_store_records,
        "plaintext_exposed": outcome.plaintext_exposed,
        "index_store": media_object_index_store_decision_value(outcome.index_store),
        "cleanup": outcome.cleanup.map(media_cleanup_session_outcome_value),
    })
}

fn crypto_seal_open_roundtrip_fixture() -> Value {
    let plaintext = b"selected ciphertext cache";
    let mut crypto = PrototypeLocalStoreCryptoProvider::default();
    let request = seal_request(
        LocalStoreRecordKind::MessageCiphertext,
        plaintext.len() as i32,
        Some(store_policy_decision(true)),
    );

    let sealed = match seal_local_store_plaintext(&mut crypto, request, plaintext)
        .expect("prototype crypto is infallible")
    {
        LocalStoreSealResult::Sealed(output) => output,
        LocalStoreSealResult::Rejected(decision) => {
            panic!("fixture seal should be accepted: {:?}", decision.reason)
        }
    };

    let write = build_sealed_local_store_write_request(request, &sealed.sealed_bytes)
        .expect("fixture sealed output should build a write request");
    let write_decision = write.evaluate();
    let open_request = LocalStoreOpenRequest::new(
        request,
        &sealed.nonce,
        &sealed.sealed_bytes,
        sealed.authentication_tag_len,
    );
    let open = open_local_store_record(&mut crypto, open_request)
        .expect("prototype crypto open is infallible");
    let opened_plaintext_len = match &open {
        LocalStoreOpenResult::Opened(bytes) => bytes.len(),
        LocalStoreOpenResult::Rejected(_) => 0,
    };

    json!({
        "fixture": "crypto_seal_open_roundtrip",
        "surface": "prototype_local_store_crypto",
        "record_kind": format!("{:?}", request.record_kind),
        "plaintext_len": plaintext.len(),
        "nonce_len": sealed.nonce.len(),
        "sealed_bytes_len": sealed.sealed_bytes.len(),
        "authentication_tag_len": sealed.authentication_tag_len,
        "seal_calls": crypto.seal_calls(),
        "open_calls": crypto.open_calls(),
        "write_request": {
            "accepted": write_decision.accepted,
            "reason": format!("{:?}", write_decision.reason),
        },
        "open": {
            "accepted": matches!(open, LocalStoreOpenResult::Opened(_)),
            "opened_plaintext_len": opened_plaintext_len,
            "plaintext_matches_expected": matches!(
                open,
                LocalStoreOpenResult::Opened(bytes) if bytes == plaintext
            ),
            "plaintext_bytes_exposed": false,
        },
    })
}

fn relay_delivery_once_fixture() -> Value {
    let mut server = PrototypeRelayServer::default();
    let submit = server.submit(valid_relay_request());
    let queued = server
        .get_item(&RELAY_ROUTE_ID)
        .expect("fixture relay item should be queued");
    let queued_state = format!("{:?}", queued.state);
    let queued_ciphertext_len = queued.ciphertext_len();
    let queued_sealed_header_len = queued.sealed_header_len();

    let delivery = server.deliver(&RELAY_ROUTE_ID, 130);
    let delivered = server
        .get_item(&RELAY_ROUTE_ID)
        .expect("fixture delivered item should remain as metadata");

    json!({
        "fixture": "relay_delivery_once",
        "surface": "prototype_relay_server",
        "server_item_count": server.len(),
        "submission": {
            "accepted": submit.submission.accepted,
            "reason_code": submit.submission.reason_code,
        },
        "enqueue": {
            "accepted": submit.queue.accepted,
            "reason": format!("{:?}", submit.queue.reason),
            "state": queued_state,
            "ciphertext_len": queued_ciphertext_len,
            "sealed_header_len": queued_sealed_header_len,
        },
        "delivery": {
            "accepted": delivery.queue.accepted,
            "reason": format!("{:?}", delivery.queue.reason),
            "state": format!("{:?}", delivery.queue.next_state),
            "ciphertext_delivered_len": delivery.ciphertext.len(),
            "sealed_header_delivered_len": delivery.sealed_header.len(),
        },
        "post_delivery": {
            "state": format!("{:?}", delivered.state),
            "ciphertext_retained": delivered.has_ciphertext(),
            "ciphertext_len": delivered.ciphertext_len(),
            "sealed_header_len": delivered.sealed_header_len(),
        },
    })
}

fn ai_participant_draft_accepted_fixture() -> Value {
    let mut backend = PrototypeAiParticipantBackend::default();
    let request = AiParticipantRequest::new(
        AiParticipantAction::DraftReply,
        valid_ai_policy(),
        true,
        true,
        3,
        32,
        32,
        0,
    );
    let decision = backend.handle(request);
    let audit = backend
        .audit_records()
        .first()
        .expect("fixture should write an AI audit record");

    json!({
        "fixture": "ai_participant_draft_accepted",
        "surface": "prototype_ai_participant",
        "request": {
            "action": format!("{:?}", request.action),
            "participant_visible": request.participant_visible,
            "grant_visible": request.grant_visible,
            "selected_message_count": request.selected_message_count,
            "input_digest_len": request.input_digest_len,
            "output_digest_len": request.output_digest_len,
            "plaintext_identity_fields": request.plaintext_identity_fields,
        },
        "decision": {
            "accepted": decision.accepted,
            "reason": format!("{:?}", decision.reason),
            "can_receive_selected_context": decision.can_receive_selected_context,
            "can_emit_draft": decision.can_emit_draft,
            "can_send_message": decision.can_send_message,
            "can_use_tool": decision.can_use_tool,
            "can_store_prompt": decision.can_store_prompt,
            "can_train": decision.can_train,
            "requires_user_confirmation": decision.requires_user_confirmation,
        },
        "audit": {
            "record_count": backend.len(),
            "action": format!("{:?}", audit.action),
            "accepted": audit.accepted,
            "reason": format!("{:?}", audit.reason),
            "input_digest_len": audit.input_digest_len,
            "output_digest_len": audit.output_digest_len,
            "plaintext_bytes_exposed": false,
        },
    })
}

fn ai_connector_local_draft_ready_fixture() -> Value {
    ai_connector_fixture("ai_connector_local_draft_ready", valid_ai_connector_input())
}

fn ai_connector_remote_forbidden_fixture() -> Value {
    let mut input = valid_ai_connector_input();
    input.runtime_kind = AiConnectorRuntimeKind::RemoteProvider;
    ai_connector_fixture("ai_connector_remote_forbidden", input)
}

fn ai_connector_plaintext_bridge_rejected_fixture() -> Value {
    let mut input = valid_ai_connector_input();
    input.plaintext_bridge_fields = 1;
    ai_connector_fixture("ai_connector_plaintext_bridge_rejected", input)
}

fn ai_connector_retention_rejected_fixture() -> Value {
    let mut input = valid_ai_connector_input();
    input.prompt_retention_enabled = true;
    ai_connector_fixture("ai_connector_retention_rejected", input)
}

fn ai_connector_user_selection_required_fixture() -> Value {
    let mut input = valid_ai_connector_input();
    input.runtime_user_selected = false;
    ai_connector_fixture("ai_connector_user_selection_required", input)
}

fn ai_connector_fixture(name: &'static str, input: AiConnectorInput) -> Value {
    let decision = input.evaluate();

    json!({
        "fixture": name,
        "surface": "ai_connector",
        "input": {
            "participant_request": {
                "action": format!("{:?}", input.participant_request.action),
                "participant_visible": input.participant_request.participant_visible,
                "grant_visible": input.participant_request.grant_visible,
                "selected_message_count": input.participant_request.selected_message_count,
                "input_digest_len": input.participant_request.input_digest_len,
                "output_digest_len": input.participant_request.output_digest_len,
                "plaintext_identity_fields": input.participant_request.plaintext_identity_fields,
            },
            "runtime_kind_code": input.runtime_kind.code(),
            "runtime_kind_label": input.runtime_kind.label(),
            "runtime_user_selected": input.runtime_user_selected,
            "model_user_selected": input.model_user_selected,
            "connector_authenticated": input.connector_authenticated,
            "model_integrity_verified": input.model_integrity_verified,
            "allow_development_runtime": input.allow_development_runtime,
            "allow_remote_runtime": input.allow_remote_runtime,
            "high_security_room": input.high_security_room,
            "context_digest_len": input.context_digest_len,
            "draft_output_digest_len": input.draft_output_digest_len,
            "plaintext_bridge_fields": input.plaintext_bridge_fields,
            "prompt_retention_enabled": input.prompt_retention_enabled,
            "training_enabled": input.training_enabled,
            "direct_send_enabled": input.direct_send_enabled,
            "tool_execution_enabled": input.tool_execution_enabled,
        },
        "decision": ai_connector_decision_value(decision),
    })
}

fn ai_connector_decision_value(decision: AiConnectorDecision) -> Value {
    json!({
        "accepted": decision.accepted,
        "reason_code": decision.reason.code(),
        "reason_label": decision.reason.label(),
        "runtime_kind_code": decision.runtime_kind.code(),
        "runtime_kind_label": decision.runtime_kind.label(),
        "can_call_model": decision.can_call_model,
        "can_emit_draft": decision.can_emit_draft,
        "can_send_message": decision.can_send_message,
        "can_use_tool": decision.can_use_tool,
        "requires_user_review": decision.requires_user_review,
        "requires_user_setup": decision.requires_user_setup,
        "requires_model_setup": decision.requires_model_setup,
        "forbids_prompt_retention": decision.forbids_prompt_retention,
        "forbids_training": decision.forbids_training,
        "plaintext_bytes_exposed": decision.plaintext_bytes_exposed,
        "participant": {
            "accepted": decision.participant.accepted,
            "reason": format!("{:?}", decision.participant.reason),
            "can_receive_selected_context": decision.participant.can_receive_selected_context,
            "can_emit_draft": decision.participant.can_emit_draft,
            "can_send_message": decision.participant.can_send_message,
            "can_use_tool": decision.participant.can_use_tool,
            "can_store_prompt": decision.participant.can_store_prompt,
            "can_train": decision.participant.can_train,
            "requires_user_confirmation": decision.participant.requires_user_confirmation,
        },
    })
}

fn backend_session_happy_path_fixture() -> Value {
    backend_session_fixture("backend_session_happy_path", valid_backend_session_input())
}

fn backend_session_bootstrap_blocked_fixture() -> Value {
    let mut input = valid_backend_session_input();
    input.bootstrap = ClientBootstrapDecision {
        accepted: false,
        can_start_sync: true,
        can_decrypt_local_store: false,
        can_open_message_ui: false,
        requires_sync: true,
        requires_recovery: false,
        requires_user_action: false,
        reason: ClientBootstrapReason::SyncIncomplete,
    };

    backend_session_fixture("backend_session_bootstrap_blocked", input)
}

fn backend_session_relay_rejected_fixture() -> Value {
    let mut input = valid_backend_session_input();
    input.outbound_send = OutboundSendDecision {
        accepted: false,
        can_send: false,
        can_persist_ciphertext: false,
        requires_user_action: true,
        reason: OutboundSendReason::MessagePolicyRejected,
    };

    backend_session_fixture("backend_session_relay_rejected", input)
}

fn backend_session_ai_rejected_fixture() -> Value {
    let mut input = valid_backend_session_input();
    input.ai_request.grant_visible = false;

    backend_session_fixture("backend_session_ai_rejected", input)
}

fn backend_session_fixture(
    fixture: &'static str,
    input: PrototypeBackendSessionInput<'_>,
) -> Value {
    let mut session = PrototypeBackendSession::default();
    let outcome = session.run(input);
    let events = session
        .events()
        .iter()
        .map(|event| {
            let view = event.view();
            json!({
                "sequence": view.sequence,
                "kind_code": view.kind_code,
                "kind_label": view.kind_label,
                "accepted": view.accepted,
                "terminal": view.terminal,
                "reason_code": view.reason_code,
                "reason_label": view.reason_label,
                "plaintext_bytes_exposed": view.plaintext_bytes_exposed,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "fixture": fixture,
        "surface": "prototype_backend_session",
        "completed": outcome.completed,
        "reason": format!("{:?}", outcome.reason),
        "bootstrap": {
            "accepted": outcome.bootstrap.accepted,
            "reason": format!("{:?}", outcome.bootstrap.reason),
            "can_open_message_ui": outcome.bootstrap.can_open_message_ui,
        },
        "crypto": {
            "seal_calls": outcome.crypto_seal_calls,
            "open_calls": outcome.crypto_open_calls,
            "delivered_ciphertext_len": outcome.delivered_ciphertext_len,
            "opened_plaintext_len": outcome.opened_plaintext_len,
            "plaintext_bytes_exposed": false,
        },
        "local_store": {
            "record_count": outcome.local_store_records,
            "write_accepted": outcome
                .store_write
                .map(|decision| decision.accepted)
                .unwrap_or(false),
        },
        "relay": {
            "item_count": outcome.relay_items,
            "submission_accepted": outcome
                .relay_submission
                .map(|decision| decision.accepted)
                .unwrap_or(false),
            "delivery_accepted": outcome
                .relay_delivery
                .map(|decision| decision.accepted)
                .unwrap_or(false),
            "sealed_header_delivered_len": outcome.delivered_sealed_header_len,
        },
        "ai": {
            "accepted": outcome.ai.map(|decision| decision.accepted).unwrap_or(false),
            "can_emit_draft": outcome
                .ai
                .map(|decision| decision.can_emit_draft)
                .unwrap_or(false),
            "can_send_message": outcome
                .ai
                .map(|decision| decision.can_send_message)
                .unwrap_or(false),
            "audit_record_count": outcome.ai_audit_records,
        },
        "events": events,
    })
}

fn valid_ai_connector_input() -> AiConnectorInput {
    AiConnectorInput {
        participant_request: AiParticipantRequest::new(
            AiParticipantAction::DraftReply,
            valid_ai_policy(),
            true,
            true,
            3,
            32,
            32,
            0,
        ),
        runtime_kind: AiConnectorRuntimeKind::LocalDevice,
        runtime_user_selected: true,
        model_user_selected: true,
        connector_authenticated: true,
        model_integrity_verified: true,
        allow_development_runtime: false,
        allow_remote_runtime: false,
        high_security_room: false,
        context_digest_len: 32,
        draft_output_digest_len: 32,
        plaintext_bridge_fields: 0,
        prompt_retention_enabled: false,
        training_enabled: false,
        direct_send_enabled: false,
        tool_execution_enabled: false,
    }
}

fn valid_backend_session_input() -> PrototypeBackendSessionInput<'static> {
    PrototypeBackendSessionInput {
        bootstrap: ClientBootstrapDecision {
            accepted: true,
            can_start_sync: true,
            can_decrypt_local_store: true,
            can_open_message_ui: true,
            requires_sync: false,
            requires_recovery: false,
            requires_user_action: false,
            reason: ClientBootstrapReason::Accepted,
        },
        seal_request: seal_request(
            LocalStoreRecordKind::MessageCiphertext,
            SESSION_PLAINTEXT.len() as i32,
            Some(store_policy_decision(true)),
        ),
        plaintext: SESSION_PLAINTEXT,
        outbound_send: OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        },
        route_id: &RELAY_ROUTE_ID,
        replay_token: &RELAY_REPLAY_TOKEN,
        queue_ttl_s: 300,
        max_queue_ttl_s: 86400,
        max_ciphertext_len: 1048576,
        sealed_header: &RELAY_SEALED_HEADER,
        plaintext_identity_fields: 0,
        padding_bucket: 3,
        created_at_s: 100,
        now_s: 120,
        delivery_now_s: 130,
        ai_request: AiParticipantRequest::new(
            AiParticipantAction::DraftReply,
            valid_ai_policy(),
            true,
            true,
            3,
            32,
            32,
            0,
        ),
    }
}

fn valid_receive_session_input() -> PrototypeReceiveSessionInput<'static> {
    PrototypeReceiveSessionInput {
        relay_submit: PrototypeRelaySubmitRequest::new(
            OutboundSendDecision {
                accepted: true,
                can_send: true,
                can_persist_ciphertext: true,
                requires_user_action: false,
                reason: OutboundSendReason::Accepted,
            },
            &RELAY_ROUTE_ID,
            &RELAY_REPLAY_TOKEN,
            300,
            86400,
            &RELAY_CIPHERTEXT,
            1048576,
            &RELAY_SEALED_HEADER,
            0,
            3,
            100,
            120,
        ),
        delivery_now_s: 130,
        ack_seen: false,
        acknowledged_at_s: 140,
        max_ack_delay_s: 300,
        ack_token_len: 32,
        ciphertext_digest_len: 32,
        delivery_tag_len: 32,
        receive_replay_state: ClientReceiveReplayState::NewInOrder,
        sender_device_trust: trusted_device(),
        message_policy: store_policy_decision(true),
        ciphertext_sealing: local_store_sealing_decision(true),
        store_locator: store_locator("conversation-7", "inbound-message-42"),
        store_record_kind: LocalStoreRecordKind::MessageCiphertext,
        plaintext_identity_fields: 0,
    }
}

fn valid_inbound_sync_input() -> InboundSyncInput {
    InboundSyncInput {
        bootstrap: accepted_inbound_sync_bootstrap(),
        source_state: InboundSyncSourceState::Ready,
        pending_delivery: true,
        route_id_len: 32,
        poll_batch_limit: 25,
        plaintext_notification_preview_len: 0,
    }
}

fn valid_authenticated_relay_source_input() -> AuthenticatedRelaySourceInput {
    AuthenticatedRelaySourceInput {
        transport: AuthenticatedRelayTransportState::Ready,
        session_ticket_len: 32,
        device_credential_len: 32,
        server_auth_tag_len: 32,
        server_authenticated: true,
        route_key_authenticated: true,
        replay_window_valid: true,
        pending_delivery: true,
        route_id_len: 32,
        poll_batch_limit: 25,
        plaintext_notification_preview_len: 0,
        plaintext_identity_fields: 0,
    }
}

fn valid_media_object_store_input() -> MediaObjectStoreInput {
    MediaObjectStoreInput {
        outbound_send: OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        },
        media_sealing: media_store_sealing_decision(true),
        object_id_len: 32,
        ciphertext_len: 4096,
        max_ciphertext_len: MERCURY_MAX_MEDIA_OBJECT_BYTES,
        sealed_header_len: 96,
        content_digest_len: 32,
        media_key_commitment_len: 32,
        plaintext_bytes: 0,
        automatic_download_requested: false,
    }
}

fn valid_media_upload_session_input() -> PrototypeMediaUploadSessionInput<'static> {
    PrototypeMediaUploadSessionInput {
        seal_request: seal_request(
            LocalStoreRecordKind::MediaCiphertext,
            MEDIA_UPLOAD_PLAINTEXT.len() as i32,
            Some(store_policy_decision(true)),
        ),
        plaintext: MEDIA_UPLOAD_PLAINTEXT,
        outbound_send: OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        },
        object_id: &MEDIA_OBJECT_ID,
        max_ciphertext_len: MERCURY_MAX_MEDIA_OBJECT_BYTES,
        sealed_header: &MEDIA_SEALED_HEADER,
        content_digest: &MEDIA_CONTENT_DIGEST,
        media_key_commitment: &MEDIA_KEY_COMMITMENT,
        plaintext_upload_bytes: 0,
        automatic_download_requested: false,
        store_record_kind: LocalStoreRecordKind::MediaCiphertext,
    }
}

fn valid_media_service_adapter_input() -> MediaServiceAdapterInput {
    MediaServiceAdapterInput {
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        media_object_store: valid_media_object_store_input().evaluate(),
        service_authenticated: true,
        upload_authorized: true,
        object_namespace_bound: true,
        content_digest_verified: true,
        allow_development_adapter: false,
    }
}

fn valid_media_service_upload_session_input() -> PrototypeMediaServiceUploadSessionInput<'static> {
    PrototypeMediaServiceUploadSessionInput {
        media_upload: valid_media_upload_session_input(),
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        service_authenticated: true,
        upload_authorized: true,
        object_namespace_bound: true,
        content_digest_verified: true,
        allow_development_adapter: false,
    }
}

fn valid_media_service_download_input(ciphertext_len: i32) -> MediaServiceDownloadInput {
    MediaServiceDownloadInput {
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        service_authenticated: true,
        download_authorized: true,
        object_namespace_bound: true,
        content_digest_verified: true,
        allow_development_adapter: false,
        object_id_len: 32,
        ciphertext_len,
        max_ciphertext_len: MERCURY_MAX_MEDIA_OBJECT_BYTES,
        sealed_header_len: 96,
        content_digest_len: 32,
        media_key_commitment_len: 32,
        plaintext_preview_bytes: 0,
        automatic_download_requested: false,
    }
}

fn valid_media_retention_input() -> MediaRetentionInput {
    MediaRetentionInput {
        operation: MediaRetentionOperation::DeleteRemoteAndEvictLocalCache,
        adapter_kind: MediaServiceAdapterKind::ProductionObjectStore,
        record_kind: LocalStoreRecordKind::MediaCiphertext,
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

fn valid_media_cleanup_session_input(
    seed_local_cache: bool,
) -> PrototypeMediaCleanupSessionInput<'static> {
    PrototypeMediaCleanupSessionInput {
        retention: valid_media_retention_input(),
        cache_locator: LocalStoreRecordLocator::new("conversation-7", "media-object-42"),
        cached_ciphertext: &MEDIA_CLEANUP_CIPHERTEXT,
        seed_local_cache,
    }
}

fn valid_media_object_index_input() -> MediaObjectIndexInput {
    MediaObjectIndexInput {
        lifecycle_state: MediaObjectLifecycleState::RemoteAndLocalCached,
        record_kind: LocalStoreRecordKind::MediaCiphertext,
        object_id_len: 32,
        content_digest_len: 32,
        media_key_commitment_len: 32,
        ciphertext_len: 4096,
        max_ciphertext_len: MERCURY_MAX_MEDIA_OBJECT_BYTES,
        plaintext_metadata_bytes: 0,
        content_digest_verified: true,
        local_cache_present: true,
        remote_object_present: true,
        remote_service_record_present: true,
        retention_hold_active: false,
    }
}

fn valid_media_object_index_store_write(
    index: MediaObjectIndexInput,
) -> MediaObjectIndexStoreWrite<'static> {
    MediaObjectIndexStoreWrite {
        object_id: &MEDIA_OBJECT_ID,
        content_digest: &MEDIA_CONTENT_DIGEST,
        media_key_commitment: &MEDIA_KEY_COMMITMENT,
        index,
    }
}

fn valid_media_object_index_production_open_input() -> MediaObjectIndexProductionOpenInput {
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

fn valid_indexed_media_upload_session_input() -> PrototypeIndexedMediaUploadSessionInput<'static> {
    PrototypeIndexedMediaUploadSessionInput {
        service_upload: valid_media_service_upload_session_input(),
        index_store: valid_media_object_index_store_write(valid_media_object_index_input()),
    }
}

fn valid_indexed_media_download_session_input<'a>(
    ciphertext: &'a [u8],
    nonce: &'a [u8],
    authentication_tag_len: i32,
) -> PrototypeIndexedMediaDownloadSessionInput<'a> {
    PrototypeIndexedMediaDownloadSessionInput {
        index_store: valid_media_object_index_store_write(valid_media_object_index_input()),
        download: valid_media_download_session_input(ciphertext, nonce, authentication_tag_len),
    }
}

fn valid_indexed_media_cleanup_session_input(
    seed_local_cache: bool,
) -> PrototypeIndexedMediaCleanupSessionInput<'static> {
    PrototypeIndexedMediaCleanupSessionInput {
        index_store: valid_media_object_index_store_write(valid_media_object_index_input()),
        cleanup: valid_media_cleanup_session_input(seed_local_cache),
    }
}

fn valid_media_download_session_input<'a>(
    ciphertext: &'a [u8],
    nonce: &'a [u8],
    authentication_tag_len: i32,
) -> PrototypeMediaDownloadSessionInput<'a> {
    PrototypeMediaDownloadSessionInput {
        download: valid_media_service_download_input(ciphertext.len() as i32),
        open_seal_request: seal_request(
            LocalStoreRecordKind::MediaCiphertext,
            MEDIA_UPLOAD_PLAINTEXT.len() as i32,
            Some(store_policy_decision(true)),
        ),
        downloaded_ciphertext: ciphertext,
        nonce,
        authentication_tag_len,
        store_record_kind: LocalStoreRecordKind::MediaCiphertext,
    }
}

fn media_download_session_sealed_media() -> LocalStoreSealOutput {
    let mut crypto = PrototypeLocalStoreCryptoProvider::default();
    match seal_local_store_plaintext(
        &mut crypto,
        seal_request(
            LocalStoreRecordKind::MediaCiphertext,
            MEDIA_UPLOAD_PLAINTEXT.len() as i32,
            Some(store_policy_decision(true)),
        ),
        MEDIA_UPLOAD_PLAINTEXT,
    )
    .expect("prototype crypto is infallible")
    {
        LocalStoreSealResult::Sealed(output) => output,
        LocalStoreSealResult::Rejected(decision) => {
            panic!("fixture seal should be accepted: {:?}", decision.reason)
        }
    }
}

fn valid_authenticated_inbound_sync_session_input()
-> PrototypeAuthenticatedInboundSyncSessionInput<'static> {
    PrototypeAuthenticatedInboundSyncSessionInput {
        bootstrap: accepted_inbound_sync_bootstrap(),
        relay_source: valid_authenticated_relay_source_input(),
        receive: valid_receive_session_input(),
    }
}

fn accepted_inbound_sync_bootstrap() -> ClientBootstrapDecision {
    ClientBootstrapDecision {
        accepted: true,
        can_start_sync: true,
        can_decrypt_local_store: true,
        can_open_message_ui: true,
        requires_sync: false,
        requires_recovery: false,
        requires_user_action: false,
        reason: ClientBootstrapReason::Accepted,
    }
}

fn blocked_inbound_sync_bootstrap() -> ClientBootstrapDecision {
    ClientBootstrapDecision {
        accepted: false,
        can_start_sync: false,
        can_decrypt_local_store: false,
        can_open_message_ui: false,
        requires_sync: false,
        requires_recovery: true,
        requires_user_action: true,
        reason: ClientBootstrapReason::RecoveryRequired,
    }
}

fn trusted_device() -> DeviceTrustDecision {
    DeviceTrustDecision {
        trusted: true,
        can_send: true,
        requires_user_action: false,
        reason: DeviceTrustReason::Trusted,
    }
}

fn local_store_sealing_decision(accepted: bool) -> LocalStoreSealingDecision {
    LocalStoreSealingDecision {
        accepted,
        reason: if accepted {
            LocalStoreSealingReason::Accepted
        } else {
            LocalStoreSealingReason::PolicyDecisionRejected
        },
        record_policy: LocalStoreRecordKind::MessageCiphertext.policy(),
    }
}

fn media_store_sealing_decision(accepted: bool) -> LocalStoreSealingDecision {
    LocalStoreSealingDecision {
        accepted,
        reason: if accepted {
            LocalStoreSealingReason::Accepted
        } else {
            LocalStoreSealingReason::PolicyDecisionRejected
        },
        record_policy: LocalStoreRecordKind::MediaCiphertext.policy(),
    }
}

fn store_locator<'a>(namespace: &'a str, record_id: &'a str) -> LocalStoreRecordLocator<'a> {
    LocalStoreRecordLocator::new(namespace, record_id)
}

fn seal_request(
    record_kind: LocalStoreRecordKind,
    plaintext_len: i32,
    policy_decision: Option<PolicyDecision>,
) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        store_locator("conversation-7", "record-42"),
        record_kind,
        local_store_key(record_kind.policy().key_scope),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        plaintext_len,
        policy_decision,
    )
}

fn local_store_key(scope: LocalStoreKeyScope) -> LocalStoreKeyDescriptor {
    let binding = match scope {
        LocalStoreKeyScope::AccountRoot => LocalStoreKeyBinding::account(32),
        LocalStoreKeyScope::DeviceLocal => LocalStoreKeyBinding::device(32, 32),
        LocalStoreKeyScope::Conversation => LocalStoreKeyBinding::conversation(32, 32),
        LocalStoreKeyScope::RoomEpoch => LocalStoreKeyBinding::room_epoch(32, 32, 7),
        LocalStoreKeyScope::Media => LocalStoreKeyBinding::media(32, 32, 7),
        LocalStoreKeyScope::Audit => LocalStoreKeyBinding::audit(32),
    };

    LocalStoreKeyDescriptor::new(
        scope,
        LocalStoreSealingSuite::MercuryLocalStoreV1,
        1,
        binding,
    )
}

fn store_policy_decision(accepted: bool) -> PolicyDecision {
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

fn valid_local_store_unlock_input() -> LocalStoreUnlockInput {
    LocalStoreUnlockInput {
        store_version: MERCURY_LOCAL_STORE_VERSION,
        keychain_available: true,
        device_secret: LocalStoreUnlockSecretState::PresentSealed,
        database_header: LocalStoreUnlockDatabaseHeaderState::Authenticated,
        app_lock_satisfied: true,
        recovery_required: false,
        plaintext_cache_records: 0,
    }
}

fn valid_account_recovery_input() -> AccountRecoveryInput {
    AccountRecoveryInput {
        recovery_requested: true,
        method: AccountRecoveryMethod::HighEntropyRecoveryKey,
        high_security_account: false,
        recovery_key_entropy_bits: 128,
        recovery_key_digest_len: 32,
        threshold_shares: 3,
        threshold_required: 2,
        threshold_approvals: 2,
        device_approval_present: true,
        server_authenticated: true,
        server_rate_limited: true,
        backup_encrypted: true,
        plaintext_backup_fields: 0,
        rotates_device_secret: false,
        audit_digest_len: 32,
    }
}

fn valid_secure_backup_restore_input() -> SecureBackupRestoreInput {
    let mut recovery = valid_account_recovery_input();
    recovery.method = AccountRecoveryMethod::CloudBackup;
    recovery.recovery_key_entropy_bits = 192;
    recovery.threshold_shares = 5;
    recovery.threshold_required = 3;
    recovery.threshold_approvals = 3;
    recovery.rotates_device_secret = true;

    SecureBackupRestoreInput {
        account_recovery: recovery.evaluate(),
        scope: SecureBackupRestoreScope::AccountAndMlsState,
        transport: SecureBackupRestoreTransport::CloudObjectStore,
        envelope_suite: SecureBackupRestoreEnvelopeSuite::XChaCha20Poly1305Blake3,
        high_security_account: false,
        backup_key_entropy_bits: 192,
        backup_key_digest_len: 32,
        kdf_memory_cost_mib: 64,
        kdf_iterations: 3,
        device_approval_present: true,
        threshold_shares: 5,
        threshold_required: 3,
        threshold_approvals: 3,
        server_authenticated: true,
        server_rate_limited: true,
        opaque_account_identifier: true,
        backup_encrypted: true,
        plaintext_export_fields: 0,
        os_plaintext_backup_excluded: true,
        mls_state_included: true,
        mls_state_sealed: true,
        mls_epoch_bound: true,
        restore_rotates_device_secret: true,
        restore_rekeys_groups: true,
        archive_manifest_authenticated: true,
        replay_nonce_len: 24,
        audit_digest_len: 32,
        retention_days: 30,
        max_retention_days: 45,
    }
}

fn valid_sealed_audit_event_chain_input() -> SealedAuditEventChainInput {
    SealedAuditEventChainInput {
        event_kind: SealedAuditEventKind::MlsCommit,
        anchor_kind: SealedAuditAnchorKind::WitnessedTransparencyLog,
        envelope_suite: SealedAuditEnvelopeSuite::XChaCha20Poly1305Blake3,
        event_sequence: 42,
        previous_chain_size: 42,
        previous_event_hash_len: 32,
        event_hash_len: 32,
        record_digest_len: 32,
        merkle_leaf_hash_len: 32,
        merkle_root_hash_len: 32,
        event_sealed: true,
        aad_binds_event_context: true,
        plaintext_field_count: 0,
        plaintext_payload_bytes: 0,
        monotonic_counter_present: true,
        monotonic_counter_increases: true,
        device_binding_digest_len: 32,
        actor_binding_digest_len: 32,
        epoch_binding_digest_len: 32,
        room_epoch_digest_len: 32,
        critical_event_bound: true,
        signed_checkpoint_present: true,
        checkpoint_signature_len: 64,
        checkpoint_timestamp_s: 1_769_990_400,
        checkpoint_size: 43,
        previous_checkpoint_size: 42,
        inclusion_proof_verified: true,
        consistency_proof_verified: true,
        transparency_receipt_present: true,
        witness_count: 3,
        witness_threshold: 2,
        witness_operator_count: 2,
        storage_append_only: true,
        storage_transactional: true,
        rollback_resistant_store: true,
        local_store_sealed: true,
        forward_secret_rotated: true,
        previous_key_material_deleted: true,
    }
}

fn valid_group_chat_input() -> GroupChatInput {
    GroupChatInput {
        protocol: GroupChatProtocol::Mls,
        crypto_suite: GroupChatCryptoSuite::HybridPqMls768,
        room_mode: mercury_core::RoomMode::Standard,
        member_count: 5,
        active_member_devices: 5,
        local_device_is_member: true,
        room_state_available: true,
        group_secret_sealed: true,
        membership_transition_pending: false,
        current_epoch: 7,
        local_epoch: 7,
        key_transparency_ready: true,
        mls_provider_configured: true,
        mls_provider_security: evaluate_mls_provider_security(valid_mls_provider_security_input(
            GroupChatCryptoSuite::HybridPqMls768,
        )),
        plaintext_member_metadata_fields: 0,
    }
}

fn valid_mls_provider_security_input(suite: GroupChatCryptoSuite) -> MlsProviderSecurityInput {
    MlsProviderSecurityInput {
        provider_configured: true,
        selected_suite: suite,
        minimum_suite: suite,
        provider_supports_selected_suite: true,
        ml_kem_parameter_set: suite.required_ml_kem_parameter_set(),
        classical_kem_component_present: suite.requires_pq_traditional_hybrid(),
        requires_pq_signatures: matches!(suite, GroupChatCryptoSuite::HybridPqMls1024),
        pq_signature_ready: matches!(suite, GroupChatCryptoSuite::HybridPqMls1024),
        suite_id_bound_to_group_context: true,
        downgrade_evidence_verified: true,
        known_answer_tests_passed: true,
        secret_zeroization_available: true,
        unsafe_crypto_backend: false,
        plaintext_key_export_fields: 0,
    }
}

fn valid_mls_provider_adapter_selection_input() -> MlsProviderAdapterSelectionInput {
    MlsProviderAdapterSelectionInput {
        provider_security: valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls768)
            .evaluate(),
        adapter_kind: MlsProviderAdapterKind::OpenMls,
        crypto_backend: MlsProviderCryptoBackendKind::LibcruxHybridPq,
        protocol_profile: MlsProviderProtocolProfile::DraftPqHybrid,
        license_kind: MlsProviderImplementationLicenseKind::Mit,
        source_verified: true,
        license_allows_distribution: true,
        rfc9420_conformance_tests_passed: true,
        pq_draft_version_pinned: true,
        ml_kem_standardized: true,
        pq_signature_standardized_when_required: true,
        kat_vectors_passed: true,
        interop_tests_passed: true,
        storage_provider_seals_group_state: true,
        storage_provider_transactional: true,
        secret_zeroization_audited: true,
        memory_hardening_enabled: true,
        downgrade_tests_passed: true,
        transcript_hash_binding_verified: true,
        unsafe_features_enabled: false,
        plaintext_export_enabled: false,
        release_artifact_signed: true,
        sbom_present: true,
        cve_monitoring_enabled: true,
    }
}

fn valid_mls_key_package_admission_input() -> MlsKeyPackageAdmissionInput {
    MlsKeyPackageAdmissionInput {
        group_chat: valid_group_chat_input().evaluate(),
        group_protocol_version: 1,
        key_package_protocol_version: 1,
        group_suite: GroupChatCryptoSuite::HybridPqMls768,
        key_package_suite: GroupChatCryptoSuite::HybridPqMls768,
        leaf_node_valid: true,
        leaf_signature_valid: true,
        key_package_signature_valid: true,
        credential_valid: true,
        required_capabilities_present: true,
        credential_supported_by_group: true,
        lifetime_not_before_s: 1_000,
        lifetime_not_after_s: 1_300,
        now_s: 1_100,
        max_lifetime_s: 600,
        leaf_source_key_package: true,
        extensions_supported: true,
        encryption_key_reuses_init_key: false,
        init_key_len: 32,
        key_package_hash_len: 32,
        key_package_hash_already_used: false,
        plaintext_identity_fields: 0,
    }
}

fn valid_mls_key_package_consume_store_write() -> MlsKeyPackageConsumeStoreWrite<'static> {
    MlsKeyPackageConsumeStoreWrite {
        key_package_admission: valid_mls_key_package_admission_input().evaluate(),
        group_id: &MLS_KEY_PACKAGE_GROUP_ID,
        key_package_hash: &MLS_KEY_PACKAGE_HASH,
        added_member_ref: &MLS_KEY_PACKAGE_ADDED_MEMBER_REF,
        welcome_send_transaction_digest: &MLS_KEY_PACKAGE_WELCOME_SEND_TRANSACTION_DIGEST,
        consumed_at_s: 1_100,
        plaintext_metadata_fields: 0,
    }
}

fn valid_mls_welcome_send_key_package_consumption_write() -> MlsKeyPackageConsumeStoreWrite<'static>
{
    MlsKeyPackageConsumeStoreWrite {
        key_package_admission: valid_mls_key_package_admission_input().evaluate(),
        group_id: &MLS_WELCOME_SEND_GROUP_ID,
        key_package_hash: &MLS_WELCOME_SEND_KEY_PACKAGE_HASH,
        added_member_ref: &MLS_WELCOME_SEND_ADDED_MEMBER_REF,
        welcome_send_transaction_digest: &MLS_WELCOME_SEND_TRANSACTION_DIGEST,
        consumed_at_s: 1_100,
        plaintext_metadata_fields: 0,
    }
}

fn valid_mls_welcome_send_outbox_write() -> MlsWelcomeSendOutboxWrite<'static> {
    let mut consume_store = PrototypeMlsKeyPackageConsumeStore::default();
    let key_package_consumption = put_mls_key_package_consumption_record(
        &mut consume_store,
        valid_mls_welcome_send_key_package_consumption_write(),
    )
    .expect("prototype MLS KeyPackage consume store cannot fail");

    MlsWelcomeSendOutboxWrite {
        key_package_consumption,
        commit_admission: valid_mls_commit_admission_input().evaluate(),
        group_id: &MLS_WELCOME_SEND_GROUP_ID,
        key_package_hash: &MLS_WELCOME_SEND_KEY_PACKAGE_HASH,
        added_member_ref: &MLS_WELCOME_SEND_ADDED_MEMBER_REF,
        welcome_send_transaction_digest: &MLS_WELCOME_SEND_TRANSACTION_DIGEST,
        commit_hash: &MLS_WELCOME_SEND_COMMIT_HASH,
        welcome_ciphertext_hash: &MLS_WELCOME_SEND_CIPHERTEXT_HASH,
        delivery_route_id: &MLS_WELCOME_SEND_DELIVERY_ROUTE_ID,
        replay_token: &MLS_WELCOME_SEND_REPLAY_TOKEN,
        created_at_s: 1_100,
        expires_at_s: 1_400,
        plaintext_metadata_fields: 0,
    }
}

fn valid_mls_membership_transaction_key_package_consumption_write()
-> MlsKeyPackageConsumeStoreWrite<'static> {
    MlsKeyPackageConsumeStoreWrite {
        key_package_admission: valid_mls_key_package_admission_input().evaluate(),
        group_id: &MLS_MEMBERSHIP_TRANSACTION_GROUP_ID,
        key_package_hash: &MLS_MEMBERSHIP_TRANSACTION_KEY_PACKAGE_HASH,
        added_member_ref: &MLS_MEMBERSHIP_TRANSACTION_ADDED_MEMBER_REF,
        welcome_send_transaction_digest: &MLS_MEMBERSHIP_TRANSACTION_WELCOME_SEND_DIGEST,
        consumed_at_s: 1_100,
        plaintext_metadata_fields: 0,
    }
}

fn valid_mls_membership_transaction_welcome_send_outbox_write() -> MlsWelcomeSendOutboxWrite<'static>
{
    let mut consume_store = PrototypeMlsKeyPackageConsumeStore::default();
    let key_package_consumption = put_mls_key_package_consumption_record(
        &mut consume_store,
        valid_mls_membership_transaction_key_package_consumption_write(),
    )
    .expect("prototype MLS KeyPackage consume store cannot fail");

    MlsWelcomeSendOutboxWrite {
        key_package_consumption,
        commit_admission: valid_mls_commit_admission_input().evaluate(),
        group_id: &MLS_MEMBERSHIP_TRANSACTION_GROUP_ID,
        key_package_hash: &MLS_MEMBERSHIP_TRANSACTION_KEY_PACKAGE_HASH,
        added_member_ref: &MLS_MEMBERSHIP_TRANSACTION_ADDED_MEMBER_REF,
        welcome_send_transaction_digest: &MLS_MEMBERSHIP_TRANSACTION_WELCOME_SEND_DIGEST,
        commit_hash: &MLS_MEMBERSHIP_TRANSACTION_COMMIT_HASH,
        welcome_ciphertext_hash: &MLS_MEMBERSHIP_TRANSACTION_WELCOME_CIPHERTEXT_HASH,
        delivery_route_id: &MLS_MEMBERSHIP_TRANSACTION_DELIVERY_ROUTE_ID,
        replay_token: &MLS_MEMBERSHIP_TRANSACTION_REPLAY_TOKEN,
        created_at_s: 1_100,
        expires_at_s: 1_400,
        plaintext_metadata_fields: 0,
    }
}

fn valid_mls_membership_transaction_commit_replay_store_write() -> MlsCommitReplayStoreWrite<'static>
{
    MlsCommitReplayStoreWrite {
        commit_admission: valid_mls_commit_admission_input().evaluate(),
        group_id: &MLS_MEMBERSHIP_TRANSACTION_GROUP_ID,
        commit_hash: &MLS_MEMBERSHIP_TRANSACTION_COMMIT_HASH,
        epoch: 7,
        applied_at_s: 1_100,
        plaintext_metadata_fields: 0,
    }
}

fn valid_mls_membership_transaction_write() -> MlsMembershipTransactionWrite<'static> {
    let mut commit_store = PrototypeMlsCommitReplayStore::default();
    let commit_replay = put_mls_commit_replay_record(
        &mut commit_store,
        valid_mls_membership_transaction_commit_replay_store_write(),
    )
    .expect("prototype MLS Commit replay store cannot fail");

    let mut consume_store = PrototypeMlsKeyPackageConsumeStore::default();
    let key_package_consumption = put_mls_key_package_consumption_record(
        &mut consume_store,
        valid_mls_membership_transaction_key_package_consumption_write(),
    )
    .expect("prototype MLS KeyPackage consume store cannot fail");

    let mut welcome_outbox = PrototypeMlsWelcomeSendOutbox::default();
    let welcome_send_outbox = put_mls_welcome_send_outbox_record(
        &mut welcome_outbox,
        valid_mls_membership_transaction_welcome_send_outbox_write(),
    )
    .expect("prototype MLS Welcome send outbox cannot fail");

    MlsMembershipTransactionWrite {
        commit_replay,
        key_package_consumption,
        welcome_send_outbox,
        group_id: &MLS_MEMBERSHIP_TRANSACTION_GROUP_ID,
        commit_hash: &MLS_MEMBERSHIP_TRANSACTION_COMMIT_HASH,
        key_package_hash: &MLS_MEMBERSHIP_TRANSACTION_KEY_PACKAGE_HASH,
        welcome_send_transaction_digest: &MLS_MEMBERSHIP_TRANSACTION_WELCOME_SEND_DIGEST,
        membership_transaction_digest: &MLS_MEMBERSHIP_TRANSACTION_DIGEST,
        commit_replay_group_id: &MLS_MEMBERSHIP_TRANSACTION_GROUP_ID,
        commit_replay_commit_hash: &MLS_MEMBERSHIP_TRANSACTION_COMMIT_HASH,
        key_package_group_id: &MLS_MEMBERSHIP_TRANSACTION_GROUP_ID,
        key_package_hash_from_consumption: &MLS_MEMBERSHIP_TRANSACTION_KEY_PACKAGE_HASH,
        key_package_welcome_send_transaction_digest:
            &MLS_MEMBERSHIP_TRANSACTION_WELCOME_SEND_DIGEST,
        outbox_group_id: &MLS_MEMBERSHIP_TRANSACTION_GROUP_ID,
        outbox_key_package_hash: &MLS_MEMBERSHIP_TRANSACTION_KEY_PACKAGE_HASH,
        outbox_commit_hash: &MLS_MEMBERSHIP_TRANSACTION_COMMIT_HASH,
        outbox_welcome_send_transaction_digest: &MLS_MEMBERSHIP_TRANSACTION_WELCOME_SEND_DIGEST,
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

fn valid_mls_welcome_admission_input() -> MlsWelcomeAdmissionInput {
    MlsWelcomeAdmissionInput {
        key_package_admission: valid_mls_key_package_admission_input().evaluate(),
        welcome_cipher_suite: GroupChatCryptoSuite::HybridPqMls768,
        key_package_suite: GroupChatCryptoSuite::HybridPqMls768,
        group_info_suite: GroupChatCryptoSuite::HybridPqMls768,
        matching_encrypted_group_secrets: true,
        group_secrets_decrypted: true,
        psks_available: true,
        resumption_psk_count: 0,
        encrypted_group_info_decrypted: true,
        group_info_signature_valid: true,
        group_id_unique_locally: true,
        ratchet_tree_available_confidentially: true,
        ratchet_tree_hash_matches: true,
        ratchet_tree_parent_hash_valid: true,
        ratchet_tree_leaves_valid: true,
        ratchet_tree_unmerged_leaves_valid: true,
        ratchet_tree_unique_encryption_keys: true,
        own_leaf_found: true,
        own_leaf_matches_key_package: true,
        path_secret_valid: true,
        epoch_secret_derived: true,
        confirmed_transcript_hash_len: 32,
        confirmation_tag_valid: true,
        commit_won_tie_break: true,
        group_epoch: 8,
        reinit_psk_used: false,
        reinit_epoch_is_one: false,
        welcome_hash_len: 32,
        welcome_hash_already_processed: false,
        plaintext_group_metadata_fields: 0,
    }
}

fn valid_mls_commit_admission_input() -> MlsCommitAdmissionInput {
    MlsCommitAdmissionInput {
        group_chat: valid_group_chat_input().evaluate(),
        current_epoch: 7,
        commit_epoch: 7,
        external_commit: false,
        sender_is_member: true,
        sender_type_new_member_commit: false,
        external_init_present: false,
        commit_signature_valid: true,
        commit_membership_tag_valid: true,
        proposal_list_valid: true,
        referenced_proposals_available: true,
        application_policy_accepts_proposals: true,
        duplicate_update_or_remove_targets: false,
        committer_update_present: false,
        committer_remove_present: false,
        path_required: true,
        update_path_present: true,
        update_path_leaf_valid: true,
        update_path_leaf_source_commit: true,
        update_path_parent_hash_valid: true,
        update_path_secret_decryptable: true,
        ratchet_tree_hash_matches: true,
        provisional_group_context_bound: true,
        epoch_secret_derived: true,
        confirmed_transcript_hash_len: 32,
        confirmation_tag_valid: true,
        commit_won_tie_break: true,
        commit_hash_len: 32,
        commit_hash_already_processed: false,
        removes_local_member: false,
        plaintext_commit_metadata_fields: 0,
    }
}

fn valid_mls_commit_replay_store_write() -> MlsCommitReplayStoreWrite<'static> {
    MlsCommitReplayStoreWrite {
        commit_admission: valid_mls_commit_admission_input().evaluate(),
        group_id: &MLS_COMMIT_GROUP_ID,
        commit_hash: &MLS_COMMIT_HASH,
        epoch: 7,
        applied_at_s: 1_100,
        plaintext_metadata_fields: 0,
    }
}

fn valid_mls_welcome_replay_store_write() -> MlsWelcomeReplayStoreWrite<'static> {
    MlsWelcomeReplayStoreWrite {
        welcome_admission: valid_mls_welcome_admission_input().evaluate(),
        group_id: &MLS_WELCOME_GROUP_ID,
        welcome_hash: &MLS_WELCOME_HASH,
        consumed_key_package_ref: &MLS_WELCOME_KEY_PACKAGE_REF,
        tree_hash: &MLS_WELCOME_TREE_HASH,
        confirmed_transcript_hash: &MLS_WELCOME_CONFIRMED_TRANSCRIPT_HASH,
        group_state_commit_digest: &MLS_WELCOME_GROUP_STATE_COMMIT_DIGEST,
        epoch: 8,
        joined_at_s: 1_100,
        init_key_deleted: true,
        group_state_committed: true,
        plaintext_metadata_fields: 0,
    }
}

fn valid_mls_provider_evidence_store_write() -> MlsProviderEvidenceStoreWrite<'static> {
    MlsProviderEvidenceStoreWrite {
        evidence_id: &MLS_PROVIDER_EVIDENCE_ID,
        provider_id_digest: &MLS_PROVIDER_ID_DIGEST,
        suite_evidence_digest: &MLS_PROVIDER_SUITE_EVIDENCE_DIGEST,
        kat_evidence_digest: &MLS_PROVIDER_KAT_EVIDENCE_DIGEST,
        downgrade_evidence_digest: &MLS_PROVIDER_DOWNGRADE_EVIDENCE_DIGEST,
        zeroization_evidence_digest: &MLS_PROVIDER_ZEROIZATION_EVIDENCE_DIGEST,
        provider_security: evaluate_mls_provider_security(valid_mls_provider_security_input(
            GroupChatCryptoSuite::HybridPqMls768,
        )),
        validated_at_s: 1_000,
        expires_at_s: 1_300,
        plaintext_evidence_fields: 0,
    }
}

fn valid_group_message_transcript_input() -> GroupMessageTranscriptInput<'static> {
    GroupMessageTranscriptInput {
        group_chat: valid_group_chat_input().evaluate(),
        outbound_send: OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        },
        local_store_seal: group_message_transcript_seal_request(7, 32),
        group_id_len: 32,
        message_epoch: 7,
        local_epoch: 7,
        sender_leaf_index: 2,
        sender_generation: 4,
        group_context_digest_len: 32,
        confirmed_transcript_hash_len: 32,
        sender_data_sealed: true,
        application_payload_sealed: true,
        reuse_guard_len: 4,
        used_generation_deleted: true,
    }
}

fn group_message_transcript_seal_request(
    room_epoch: i32,
    group_id_len: i32,
) -> LocalStoreSealRequest<'static> {
    LocalStoreSealRequest::new(
        store_locator("group-7", "message-42"),
        LocalStoreRecordKind::MessageCiphertext,
        LocalStoreKeyDescriptor::new(
            LocalStoreKeyScope::RoomEpoch,
            LocalStoreSealingSuite::MercuryLocalStoreV1,
            1,
            LocalStoreKeyBinding::room_epoch(32, group_id_len, room_epoch),
        ),
        LocalStoreSealingSuite::MercuryLocalStoreV1.nonce_len(),
        64,
        Some(store_policy_decision(true)),
    )
}

fn valid_group_relay_envelope_input() -> GroupRelayEnvelopeInput {
    GroupRelayEnvelopeInput {
        transcript: valid_group_message_transcript_input().evaluate(),
        relay_submission: RelaySubmissionDecision {
            accepted: true,
            reason_code: 0,
            audit_class: 0,
        },
        delivery_token_len: 12,
        delivery_token_bound_to_route: true,
        sender_certificate_sealed: true,
        anonymous_membership_proof: valid_anonymous_group_membership_proof_input().evaluate(),
        anonymous_membership_proof_len: 64,
        anonymous_rate_limit: valid_anonymous_rate_limit_nullifier_input().evaluate(),
        sealed_envelope_len: 128,
        plaintext_sender_fields: 0,
        plaintext_group_fields: 0,
    }
}

fn valid_anonymous_rate_limit_nullifier_input() -> AnonymousRateLimitNullifierInput {
    AnonymousRateLimitNullifierInput {
        membership_proof: valid_anonymous_group_membership_proof_input().evaluate(),
        credential_kind: AnonymousRateLimitCredentialKind::ArcWindow,
        nullifier_len: 32,
        nullifier_already_spent: false,
        nullifier_store_available: true,
        nullifier_store_opaque: true,
        bound_to_route: true,
        bound_to_group_epoch: true,
        redemption_context_len: 32,
        credential_context_len: 32,
        window_start_s: 1000,
        window_end_s: 1300,
        now_s: 1100,
        presentation_count: 1,
        presentation_limit: 8,
        max_presentation_limit: 8,
        plaintext_rate_limit_fields: 0,
    }
}

fn valid_anonymous_nullifier_store_write() -> AnonymousNullifierStoreWrite<'static> {
    AnonymousNullifierStoreWrite {
        nullifier: &ANONYMOUS_NULLIFIER_BYTES,
        redemption_context_digest: &ANONYMOUS_NULLIFIER_REDEMPTION_CONTEXT_DIGEST,
        credential_context_digest: &ANONYMOUS_NULLIFIER_CREDENTIAL_CONTEXT_DIGEST,
        credential_kind: AnonymousRateLimitCredentialKind::ArcWindow,
        nullifier_decision: valid_anonymous_rate_limit_nullifier_input().evaluate(),
        window_start_s: 1000,
        window_end_s: 1300,
        presentation_count_before: 1,
        presentation_limit: 8,
        plaintext_metadata_fields: 0,
    }
}

fn valid_anonymous_group_membership_proof_input() -> AnonymousGroupMembershipProofInput {
    AnonymousGroupMembershipProofInput {
        group_chat: {
            let mut group = valid_group_chat_input();
            group.room_mode = mercury_core::RoomMode::HighSecurity;
            group.crypto_suite = GroupChatCryptoSuite::HybridPqMls1024;
            group.mls_provider_security = evaluate_mls_provider_security(
                valid_mls_provider_security_input(GroupChatCryptoSuite::HybridPqMls1024),
            );
            group.evaluate()
        },
        scheme: AnonymousGroupMembershipProofScheme::PqGroupWrapper,
        issuer_trust: evaluate_anonymous_credential_issuer_trust(
            valid_anonymous_credential_issuer_trust_input(),
        ),
        high_security_room: true,
        scheme_post_quantum_safe: true,
        issuer_key_id_len: 32,
        challenge_digest_len: 32,
        presentation_nonce_len: 32,
        proof_len: 128,
        presentation_header_bound: true,
        group_epoch_bound: true,
        route_bound: true,
        replay_nullifier_len: 32,
        replay_nullifier_seen: false,
        issued_at_s: 1000,
        expires_at_s: 1300,
        now_s: 1100,
        plaintext_member_identifier_fields: 0,
    }
}

fn valid_anonymous_credential_issuer_trust_input() -> AnonymousCredentialIssuerTrustInput {
    AnonymousCredentialIssuerTrustInput {
        key_transparency: evaluate_key_transparency(valid_key_transparency_proof_input()),
        issuer_witness_audit: evaluate_anonymous_issuer_witness_audit(
            valid_anonymous_issuer_witness_audit_input(),
        ),
        issuer_key_id_len: 32,
        issuer_directory_inclusion_verified: true,
        issuer_key_bound_to_challenge: true,
        active_issuer_key_count: 2,
        max_active_issuer_key_count: 8,
        directory_age_s: 60,
        max_directory_age_s: 300,
        key_not_before_s: 1000,
        key_not_after_s: 1300,
        now_s: 1100,
        revocation_status_fresh: true,
        issuer_key_revoked: false,
        opaque_partitioning_metadata_bits: 0,
    }
}

fn valid_anonymous_issuer_witness_audit_input() -> AnonymousIssuerWitnessAuditInput {
    AnonymousIssuerWitnessAuditInput {
        key_transparency: evaluate_key_transparency(valid_key_transparency_proof_input()),
        signed_tree_head_len: 32,
        inclusion_root_len: 32,
        previous_tree_size: 12,
        current_tree_size: 13,
        required_witness_count: 2,
        verified_witness_count: 3,
        independent_operator_count: 2,
        audit_age_s: 60,
        max_audit_age_s: 300,
        split_view_reports: 0,
        auditor_signature_len: 64,
        plaintext_partitioning_fields: 0,
    }
}

fn valid_key_transparency_proof_input() -> KeyTransparencyProofInput {
    KeyTransparencyProofInput {
        inclusion: KeyTransparencyProofStatus::Verified,
        consistency: KeyTransparencyProofStatus::Verified,
        key_history: KeyTransparencyProofStatus::Verified,
        witness: KeyTransparencyWitnessStatus::QuorumSatisfied,
        require_witness: true,
        previous_tree_size: 12,
        current_tree_size: 13,
        proof_age_s: 60,
        max_proof_age_s: 300,
    }
}

fn valid_local_store_production_open_input() -> LocalStoreProductionOpenInput {
    LocalStoreProductionOpenInput {
        unlock: valid_local_store_unlock_input(),
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

fn valid_local_store_keychain_unlock_input() -> LocalStoreKeychainUnlockInput {
    LocalStoreKeychainUnlockInput {
        store_version: MERCURY_LOCAL_STORE_VERSION,
        backend: LocalStoreKeychainBackend::AndroidKeystore,
        backend_available: true,
        protection: LocalStoreKeychainProtection::HardwareBacked,
        allow_development_backend: false,
        user_auth_required: false,
        user_auth_satisfied: false,
        device_secret: LocalStoreUnlockSecretState::PresentSealed,
        device_secret_exportable: false,
        database_header: LocalStoreUnlockDatabaseHeaderState::Authenticated,
        recovery_required: false,
        plaintext_cache_records: 0,
    }
}

fn valid_production_store_session_input() -> PrototypeProductionStoreSessionInput<'static> {
    PrototypeProductionStoreSessionInput {
        keychain: valid_local_store_keychain_unlock_input(),
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
        write_request: LocalStoreWriteRequest::new(
            store_locator("conversation-7", "message-42"),
            LocalStoreRecordKind::MessageCiphertext,
            LocalStorePayload::sealed(b"sealed-message"),
            Some(store_policy_decision(true)),
        ),
    }
}

fn valid_platform_local_store_adapter_input(
    runtime: PlatformLocalStoreRuntime,
) -> PlatformLocalStoreAdapterInput {
    PlatformLocalStoreAdapterInput {
        runtime,
        adapter_kind: PlatformLocalStoreAdapterKind::ProductionEncryptedDatabase,
        database_root_present: true,
        os_keychain_available: true,
        hardware_backed_key_store: true,
        app_lock_satisfied: true,
        allow_development_adapters: false,
    }
}

fn valid_local_store_database_security_input() -> LocalStoreDatabaseSecurityInput {
    LocalStoreDatabaseSecurityInput {
        platform_adapter: valid_platform_local_store_adapter_input(
            PlatformLocalStoreRuntime::Desktop,
        )
        .evaluate(),
        production_open: valid_local_store_production_open_input().evaluate(),
        engine: LocalStoreDatabaseEngine::SqlCipherV4,
        cipher: LocalStoreDatabaseCipher::Aes256CbcHmacSha512,
        kdf: LocalStoreDatabaseKdf::RawKeyFromPlatformKeystore,
        kdf_iterations: MERCURY_LOCAL_STORE_MIN_KDF_ITERATIONS,
        page_size: MERCURY_LOCAL_STORE_PAGE_SIZE,
        per_page_random_nonce: true,
        per_page_authentication: true,
        encryption_key_separate_from_mac_key: true,
        unique_database_salt: true,
        raw_key_wrapped_by_platform_keystore: true,
        encrypted_wal: true,
        encrypted_journal: true,
        temp_store_memory_only: true,
        plaintext_header_bytes: 0,
        os_cloud_backup_excluded: true,
        backup_uses_consistent_encrypted_snapshot: true,
        secure_delete_enabled: true,
        memory_locking_enabled: true,
        zeroizes_key_material: true,
        crash_recovery_tested: true,
        plaintext_metadata_fields: 0,
        sqlite_extension_loading_enabled: false,
        debug_plaintext_export_enabled: false,
    }
}

fn valid_local_store_database_adapter_selection_input() -> LocalStoreDatabaseAdapterSelectionInput {
    LocalStoreDatabaseAdapterSelectionInput {
        database_security: valid_local_store_database_security_input().evaluate(),
        adapter_kind: LocalStoreDatabaseAdapterKind::SqlCipherCommunity,
        binding_kind: LocalStoreDatabaseBindingKind::RusqliteBundledSqlcipher,
        target_platform: LocalStoreDatabaseTargetPlatform::Windows,
        license_kind: LocalStoreDatabaseLicenseKind::CommunityBsd,
        sqlcipher_major_version: 4,
        sqlite_source_verified: true,
        sqlcipher_source_verified: true,
        platform_package_supported: true,
        license_allows_redistribution: true,
        crypto_provider_documented: true,
        fips_required: false,
        fips_module_validated: false,
        fips_runtime_self_tests_available: false,
        fips_mode_checked_at_runtime: false,
        compile_has_codec: true,
        compile_has_sqlcipher_extra_init_shutdown: true,
        temp_store_memory_configured: true,
        extension_loading_disabled: true,
        trusted_schema_disabled: true,
        secure_delete_configured: true,
        cipher_memory_security_enabled: true,
        cipher_integrity_check_on_open: true,
        sqlcipher_compatibility_current_major: true,
        deterministic_migration_tested: true,
        crash_recovery_drill_passed: true,
        release_artifacts_signed: true,
        sbom_present: true,
        cve_monitoring_enabled: true,
        debug_sqlcipher_logging_enabled: false,
    }
}

fn valid_relay_request<'a>() -> PrototypeRelaySubmitRequest<'a> {
    PrototypeRelaySubmitRequest::new(
        OutboundSendDecision {
            accepted: true,
            can_send: true,
            can_persist_ciphertext: true,
            requires_user_action: false,
            reason: OutboundSendReason::Accepted,
        },
        &RELAY_ROUTE_ID,
        &RELAY_REPLAY_TOKEN,
        300,
        86400,
        &RELAY_CIPHERTEXT,
        1048576,
        &RELAY_SEALED_HEADER,
        0,
        3,
        100,
        120,
    )
}

fn valid_ai_policy() -> AiPolicyFacts {
    AiPolicyFacts {
        grant: AiGrantFacts {
            version: 1,
            principal_kind: 2,
            room_mode: 1,
            ai_mode: 1,
            ttl_s: 300,
            approver_count: 1,
            read_scope: 1,
            write_scope: 1,
            tool_scope: 1,
            retention_mode: 0,
            training_allowed: 0,
            prompt_store_allowed: 0,
        },
        lifecycle: AiLifecycleFacts {
            version: 1,
            grant_state: 1,
            revoke_reason: 0,
            now_s: 100,
            expires_at_s: 400,
            room_mode: 1,
            access_kind: 0,
            epoch_rotated: 0,
        },
    }
}

const SEALED_MESSAGE_BYTES: [u8; 32] = [42; 32];
const SESSION_PLAINTEXT: &[u8] = b"session payload to seal";
const RELAY_ROUTE_ID: [u8; 32] = [7; 32];
const RELAY_REPLAY_TOKEN: [u8; 32] = [9; 32];
const RELAY_CIPHERTEXT: [u8; 128] = [42; 128];
const RELAY_SEALED_HEADER: [u8; 96] = [5; 96];
const MEDIA_UPLOAD_PLAINTEXT: &[u8] = b"prototype media upload payload";
const MEDIA_OBJECT_ID: [u8; 32] = [3; 32];
const MEDIA_SEALED_HEADER: [u8; 96] = [4; 96];
const MEDIA_CONTENT_DIGEST: [u8; 32] = [6; 32];
const MEDIA_KEY_COMMITMENT: [u8; 32] = [8; 32];
const MEDIA_CLEANUP_CIPHERTEXT: [u8; 64] = [91; 64];
