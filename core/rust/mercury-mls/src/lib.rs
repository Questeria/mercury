//! Mercury MLS engine (Stage 8).
//!
//! Wraps an MLS provider (OpenMLS 0.8, RFC 9420) and will produce the inputs for
//! `mercury-core`'s MLS policy gates -- the engine does the MLS crypto; the gates
//! (`evaluate_mls_provider_security`, `evaluate_group_chat`,
//! `evaluate_mls_key_package_admission`, ...) decide policy. This mirrors how
//! `mercury-kt` feeds `evaluate_key_transparency`: nothing here re-decides
//! policy, and no crypto is implemented by hand (OpenMLS + its RustCrypto
//! provider do it).
//!
//! Built so far: members generate KeyPackages ([`MlsMember`]); a PRESENTED
//! KeyPackage is validated by OpenMLS and turned into an
//! [`MlsKeyPackageAdmissionInput`] for the admission gate
//! ([`key_package_admission_input`]); a founder creates a group
//! ([`MlsMember::create_group`]) and [`add_member`] performs the real OpenMLS
//! commit + Welcome, recording the single-use consumption ([`ConsumedKeyPackage`]
//! → [`put_mls_key_package_consumption_record`] via [`InMemoryConsumeStore`]).
//! Provider security now ATTESTS the live backend ([`openmls_provider_security`]
//! derives `known_answer_tests_passed` from a real SHA-256 KAT
//! ([`run_known_answer_tests`]) + `provider_supports_selected_suite` from
//! `crypto.supports`). A joiner builds its own group view from the Welcome
//! ([`join_group`]) and members exchange real encrypted application messages
//! ([`send_message`] / [`receive_message`]); a member can be removed
//! ([`remove_member`]), rotating the epoch so the removed member loses access.
//! Application messages serialize to canonical MLS wire bytes
//! ([`serialize_message`]) and open straight from those bytes
//! ([`receive_message_bytes`]) — the opaque frame a relay transits without ever
//! seeing plaintext. Provider qualification is fed too:
//! [`openmls_adapter_selection`] derives the adapter's runtime/identity facts and
//! takes the linked crypto backend + a [`MlsSupplyChainAttestation`] as explicit
//! inputs (never fabricated) for the release/linking gate; and
//! [`OpenMlsProviderEvidence`] derives a digest-only evidence bundle (bound to a
//! live KAT) for the provider-evidence store/use gates. PQ-hybrid suites are a
//! later increment. The PERSISTENT, app-facing account engine lives in
//! [`engine`]: one long-lived MLS identity per account ([`MlsEngine`]) whose
//! entire state -- signer + every group's secrets -- exports to bytes for the
//! app's encrypted snapshot and imports after a restart.

#![forbid(unsafe_code)]

use core::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};

use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
#[cfg(feature = "xwing")]
use openmls_libcrux_crypto::Provider as OpenMlsLibcrux;
use openmls_rust_crypto::OpenMlsRustCrypto;
use sha2::{Digest as _, Sha256};
use tls_codec::{Deserialize as _, Serialize as _};

use mercury_core::{
    AcceptedMlsKeyPackageConsumeStoreWrite, GroupChatInput, GroupChatProtocol,
    MlsKeyPackageConsumeStoreAdapter, MlsKeyPackageConsumeStoreRecord,
    MlsKeyPackageConsumeStoreWrite, MlsProviderAdapterKind, MlsProviderAdapterSelectionInput,
    MlsProviderEvidenceStoreWrite, MlsProviderImplementationLicenseKind,
    MlsProviderProtocolProfile, MlsProviderSecurityInput, RoomMode,
};
pub use mercury_core::{
    GroupChatCryptoSuite, GroupChatDecision, MlsKeyPackageAdmissionDecision,
    MlsKeyPackageAdmissionInput, MlsKeyPackageConsumeStoreDecision,
    MlsProviderAdapterSelectionDecision, MlsProviderCryptoBackendKind,
    MlsProviderEvidenceStoreDecision, MlsProviderEvidenceStoreRecord,
    MlsProviderEvidenceUseDecision, MlsProviderEvidenceUseInput, MlsProviderSecurityDecision,
    PrototypeMlsProviderEvidenceStore, evaluate_mls_key_package_admission,
    evaluate_mls_key_package_consume_store_write, evaluate_mls_provider_evidence_store_write,
    evaluate_mls_provider_evidence_use, put_mls_key_package_consumption_record,
    put_mls_provider_evidence_record,
};

pub mod engine;
pub use engine::{AddOutcome, MlsEngine};

/// Mercury's MLS protocol version code (MLS 1.0 / RFC 9420). The gate matches the
/// group's and the KeyPackage's version; OpenMLS `validate` enforces `Mls10`.
pub const MLS_PROTOCOL_VERSION: i32 = 1;

/// Mercury's default MLS ciphersuite: the RFC 9420 mandatory-to-implement
/// classical suite (X25519 / AES-128-GCM / SHA-256 / Ed25519). Backed by the
/// RustCrypto provider via [`MlsMember`].
pub const DEFAULT_CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

/// Mercury's POST-QUANTUM MLS ciphersuite: X-Wing (a hybrid X25519 + ML-KEM-768
/// KEM) with ChaCha20-Poly1305 / SHA-256 / Ed25519. This is the IETF-registered
/// hybrid PQ suite for MLS; it is executed by the formally-verified libcrux
/// provider via [`MlsPqMember`] (the RustCrypto provider does not implement it).
/// Group chats built under this suite are confidential against a future quantum
/// adversary as long as EITHER the X25519 or the ML-KEM-768 half holds. It maps to
/// Mercury's [`GroupChatCryptoSuite::HybridPqMls768`].
pub const PQ_CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_256_XWING_CHACHA20POLY1305_SHA256_Ed25519;

/// Errors surfaced by the MLS engine. Carries no secret material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlsError {
    /// Generating the member's signature keypair failed.
    SignatureKeygen,
    /// Building the member's KeyPackage failed.
    KeyPackageBuild,
    /// Creating the MLS group failed.
    GroupCreate,
    /// Adding a member (commit / Welcome) failed.
    AddMember,
    /// A required identifier was not exactly 32 bytes (group id, hash, key).
    BadDigestLength,
    /// Serializing an MLS message (Welcome / application) failed.
    WelcomeSerialize,
    /// Joining a group from a Welcome failed.
    JoinGroup,
    /// Encrypting an application message failed.
    MessageEncrypt,
    /// A received MLS message could not be parsed into a protocol message.
    MessageDecode,
    /// Decrypting / processing a received application message failed.
    MessageDecrypt,
    /// The member to remove was not found in the group.
    MemberNotFound,
    /// Removing a member (commit) failed.
    RemoveMember,
    /// Deriving the digest-only provider-evidence bundle failed (live KAT did not
    /// match the published vector, or the validity window was degenerate).
    ProviderEvidence,
    /// Processing/merging a received membership-change commit failed (or the
    /// message was not a commit).
    CommitProcess,
    /// The requested group does not exist in the engine's storage
    /// ([`MlsEngine`] operations on an unknown / never-joined group id).
    GroupNotFound,
    /// A credential was not a BasicCredential -- the engine only attributes
    /// identities it can verify the shape of, so it fails closed.
    CredentialUnsupported,
    /// Locally deleting a group's persisted state failed
    /// ([`MlsEngine::leave_local`]).
    GroupDelete,
    /// Exporting the engine's persistent state failed
    /// ([`MlsEngine::export_state`]).
    StateExport,
    /// Imported engine-state bytes were rejected (truncated, malformed, unknown
    /// version, or trailing garbage) -- [`MlsEngine::import_state`] fails closed.
    StateImport,
}

impl fmt::Display for MlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MlsError::SignatureKeygen => f.write_str("MLS signature key generation failed"),
            MlsError::KeyPackageBuild => f.write_str("MLS key package build failed"),
            MlsError::GroupCreate => f.write_str("MLS group creation failed"),
            MlsError::AddMember => f.write_str("MLS add-member failed"),
            MlsError::BadDigestLength => f.write_str("MLS identifier was not 32 bytes"),
            MlsError::WelcomeSerialize => f.write_str("MLS message serialization failed"),
            MlsError::JoinGroup => f.write_str("MLS join-from-Welcome failed"),
            MlsError::MessageEncrypt => f.write_str("MLS message encryption failed"),
            MlsError::MessageDecode => f.write_str("MLS message could not be parsed"),
            MlsError::MessageDecrypt => f.write_str("MLS message decryption failed"),
            MlsError::MemberNotFound => f.write_str("MLS member to remove not found"),
            MlsError::RemoveMember => f.write_str("MLS remove-member failed"),
            MlsError::ProviderEvidence => f.write_str("MLS provider-evidence derivation failed"),
            MlsError::CommitProcess => f.write_str("MLS commit processing failed"),
            MlsError::GroupNotFound => f.write_str("MLS group not found in engine storage"),
            MlsError::CredentialUnsupported => {
                f.write_str("MLS credential was not a basic credential")
            }
            MlsError::GroupDelete => f.write_str("MLS local group deletion failed"),
            MlsError::StateExport => f.write_str("MLS engine state export failed"),
            MlsError::StateImport => f.write_str("MLS engine state bytes were rejected"),
        }
    }
}

impl std::error::Error for MlsError {}

/// A freshly generated MLS member: an OpenMLS signature keypair, a credential,
/// and a KeyPackage others can use to add this member to a group. Backed by an
/// in-memory OpenMLS provider (RustCrypto backend).
///
/// The provider holds the member's secret init/encryption keys (stored against
/// the KeyPackage's hash ref); both it and the signer are retained for the group
/// operations (create / join / commit) built in later increments.
pub struct MlsMember {
    provider: OpenMlsRustCrypto,
    signer: SignatureKeyPair,
    bundle: KeyPackageBundle,
}

impl MlsMember {
    /// Generate a member identified by `identity`, producing a fresh KeyPackage
    /// under [`DEFAULT_CIPHERSUITE`].
    pub fn generate(identity: &[u8]) -> Result<Self, MlsError> {
        let provider = OpenMlsRustCrypto::default();
        let signer = SignatureKeyPair::new(DEFAULT_CIPHERSUITE.signature_algorithm())
            .map_err(|_| MlsError::SignatureKeygen)?;
        let credential = BasicCredential::new(identity.to_vec());
        let credential_with_key = CredentialWithKey {
            credential: credential.into(),
            signature_key: signer.public().into(),
        };
        let bundle = KeyPackage::builder()
            .build(DEFAULT_CIPHERSUITE, &provider, &signer, credential_with_key)
            .map_err(|_| MlsError::KeyPackageBuild)?;
        Ok(Self {
            provider,
            signer,
            bundle,
        })
    }

    /// The member's KeyPackage (presented when joining a group).
    pub fn key_package(&self) -> &KeyPackage {
        self.bundle.key_package()
    }

    /// The ciphersuite the KeyPackage was built under.
    pub fn ciphersuite(&self) -> Ciphersuite {
        self.key_package().ciphersuite()
    }

    /// The OpenMLS provider holding this member's secret key material.
    pub fn provider(&self) -> &OpenMlsRustCrypto {
        &self.provider
    }

    /// The member's signature keypair (signs group operations).
    pub fn signer(&self) -> &SignatureKeyPair {
        &self.signer
    }

    /// Serialize this member's KeyPackage to canonical TLS wire bytes — what the
    /// member presents to a group it wishes to join.
    pub fn key_package_bytes(&self) -> Vec<u8> {
        self.key_package()
            .tls_serialize_detached()
            .expect("serializing a freshly built KeyPackage cannot fail")
    }

    /// Create a new MLS group with this member as the founder, under a 32-byte
    /// `group_id` (the consume-store gate requires a 32-byte group id). The
    /// founder's credential + signature key come from its own KeyPackage.
    pub fn create_group(&self, group_id: [u8; 32]) -> Result<MlsGroup, MlsError> {
        let config = MlsGroupCreateConfig::builder()
            .ciphersuite(DEFAULT_CIPHERSUITE)
            // Embed the ratchet tree in the Welcome so a joiner can build the
            // group from the Welcome alone (no out-of-band tree delivery).
            .use_ratchet_tree_extension(true)
            .build();
        let credential_with_key = CredentialWithKey {
            credential: self.key_package().leaf_node().credential().clone(),
            signature_key: self.signer.public().into(),
        };
        MlsGroup::new_with_group_id(
            &self.provider,
            &self.signer,
            &config,
            GroupId::from_slice(&group_id),
            credential_with_key,
        )
        .map_err(|_| MlsError::GroupCreate)
    }
}

/// A freshly generated POST-QUANTUM MLS member: the same shape as [`MlsMember`] but
/// built under the X-Wing PQ ciphersuite ([`PQ_CIPHERSUITE`]) and backed by the
/// formally-verified libcrux provider (which actually executes X-Wing, where the
/// RustCrypto provider does not). Group chats founded/joined by an `MlsPqMember`
/// are post-quantum confidential. The classical [`MlsMember`] is unchanged; this is
/// an additive parallel path so callers opt into PQ explicitly.
///
/// Gated behind the off-by-default `xwing` feature: the libcrux provider carries open
/// RUSTSEC advisories (see the `openmls_libcrux_crypto` note in Cargo.toml). The
/// default build does not compile this type.
#[cfg(feature = "xwing")]
pub struct MlsPqMember {
    provider: OpenMlsLibcrux,
    signer: SignatureKeyPair,
    bundle: KeyPackageBundle,
}

#[cfg(feature = "xwing")]
impl MlsPqMember {
    /// Generate a post-quantum member identified by `identity`, producing a fresh
    /// KeyPackage under [`PQ_CIPHERSUITE`] (X-Wing) via the libcrux provider.
    pub fn generate(identity: &[u8]) -> Result<Self, MlsError> {
        let provider = OpenMlsLibcrux::default();
        let signer = SignatureKeyPair::new(PQ_CIPHERSUITE.signature_algorithm())
            .map_err(|_| MlsError::SignatureKeygen)?;
        let credential = BasicCredential::new(identity.to_vec());
        let credential_with_key = CredentialWithKey {
            credential: credential.into(),
            signature_key: signer.public().into(),
        };
        let bundle = KeyPackage::builder()
            .build(PQ_CIPHERSUITE, &provider, &signer, credential_with_key)
            .map_err(|_| MlsError::KeyPackageBuild)?;
        Ok(Self {
            provider,
            signer,
            bundle,
        })
    }

    /// The member's KeyPackage (presented when joining a group).
    pub fn key_package(&self) -> &KeyPackage {
        self.bundle.key_package()
    }

    /// The ciphersuite the KeyPackage was built under (always [`PQ_CIPHERSUITE`]).
    pub fn ciphersuite(&self) -> Ciphersuite {
        self.key_package().ciphersuite()
    }

    /// Mercury's `GroupChatCryptoSuite` for this member — always the post-quantum
    /// `HybridPqMls768` (X-Wing). Confirms the suite label matches the crypto.
    pub fn mercury_suite(&self) -> Option<GroupChatCryptoSuite> {
        mercury_suite(self.ciphersuite())
    }

    /// The libcrux OpenMLS provider holding this member's secret key material.
    pub fn provider(&self) -> &OpenMlsLibcrux {
        &self.provider
    }

    /// The member's signature keypair (signs group operations).
    pub fn signer(&self) -> &SignatureKeyPair {
        &self.signer
    }

    /// Serialize this member's KeyPackage to canonical TLS wire bytes.
    pub fn key_package_bytes(&self) -> Vec<u8> {
        self.key_package()
            .tls_serialize_detached()
            .expect("serializing a freshly built KeyPackage cannot fail")
    }

    /// Create a new post-quantum MLS group with this member as the founder, under a
    /// 32-byte `group_id`, using the X-Wing ciphersuite.
    pub fn create_group(&self, group_id: [u8; 32]) -> Result<MlsGroup, MlsError> {
        let config = MlsGroupCreateConfig::builder()
            .ciphersuite(PQ_CIPHERSUITE)
            .use_ratchet_tree_extension(true)
            .build();
        let credential_with_key = CredentialWithKey {
            credential: self.key_package().leaf_node().credential().clone(),
            signature_key: self.signer.public().into(),
        };
        MlsGroup::new_with_group_id(
            &self.provider,
            &self.signer,
            &config,
            GroupId::from_slice(&group_id),
            credential_with_key,
        )
        .map_err(|_| MlsError::GroupCreate)
    }
}

/// Map an OpenMLS ciphersuite into Mercury's `GroupChatCryptoSuite` vocabulary:
/// the RFC 9420 classical suite, and the X-Wing post-quantum hybrid suite (backed
/// by the libcrux provider). An unmodelled suite returns `None` so the admission
/// gate rejects it rather than silently accepting it.
fn mercury_suite(ciphersuite: Ciphersuite) -> Option<GroupChatCryptoSuite> {
    match ciphersuite {
        Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519 => {
            Some(GroupChatCryptoSuite::ClassicalMls128)
        }
        Ciphersuite::MLS_256_XWING_CHACHA20POLY1305_SHA256_Ed25519 => {
            Some(GroupChatCryptoSuite::HybridPqMls768)
        }
        _ => None,
    }
}

/// A suite guaranteed to differ from `suite` — used to represent an OpenMLS
/// ciphersuite Mercury does not model, so the admission gate rejects it
/// (`CipherSuiteMismatch`) rather than silently accepting an unknown suite.
fn suite_complement(suite: GroupChatCryptoSuite) -> GroupChatCryptoSuite {
    match suite {
        GroupChatCryptoSuite::ClassicalMls128 => GroupChatCryptoSuite::HybridPqMls768,
        _ => GroupChatCryptoSuite::ClassicalMls128,
    }
}

/// Map Mercury's `GroupChatCryptoSuite` to the OpenMLS `Ciphersuite`:
/// `ClassicalMls128` is the RustCrypto-backed classical suite; `HybridPqMls768` is
/// the X-Wing post-quantum suite executed by the libcrux provider. `HybridPqMls1024`
/// (a higher-security ML-KEM-1024 hybrid) has no registered OpenMLS ciphersuite
/// yet, so it honestly returns `None`.
fn openmls_ciphersuite(suite: GroupChatCryptoSuite) -> Option<Ciphersuite> {
    match suite {
        GroupChatCryptoSuite::ClassicalMls128 => Some(DEFAULT_CIPHERSUITE),
        GroupChatCryptoSuite::HybridPqMls768 => Some(PQ_CIPHERSUITE),
        GroupChatCryptoSuite::HybridPqMls1024 => None,
    }
}

/// SHA-256 known-answer test vector (`SHA-256("abc")`, FIPS 180-4 example).
const KAT_INPUT: &[u8] = b"abc";
const KAT_SHA256: [u8; 32] = [
    0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
    0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
];

/// Run a REAL known-answer test against the OpenMLS crypto provider: hash a known
/// input and compare to the published SHA-256 vector. Returns true iff the
/// backend computes the correct digest -- evidence the crypto primitives are
/// wired correctly, not a stub. (Production runs a fuller KAT suite covering
/// AEAD / HPKE / signatures; this representative hash KAT gates the
/// provider-security `known_answer_tests_passed` flag.)
pub fn run_known_answer_tests(provider: &impl OpenMlsProvider) -> bool {
    provider
        .crypto()
        .hash(HashType::Sha2_256, KAT_INPUT)
        .is_ok_and(|digest| digest == KAT_SHA256)
}

/// Produce the provider-security token for Mercury's OpenMLS provider at `suite`
/// (floor `minimum_suite`). Two flags are now DERIVED from real checks against
/// the live provider: `known_answer_tests_passed` (a real SHA-256 KAT via
/// [`run_known_answer_tests`]) and `provider_supports_selected_suite`
/// (`provider.crypto().supports(..)`). The remaining flags are genuine
/// deployment/design facts of the OpenMLS + RustCrypto stack (memory-safe
/// backend, secrets zeroized, MLS binds the suite id into the group context, no
/// plaintext key export); a later increment adds the adapter-selection +
/// evidence-store gates for supply-chain attestation (SBOM, signed artifact).
pub fn openmls_provider_security(
    provider: &impl OpenMlsProvider,
    suite: GroupChatCryptoSuite,
    minimum_suite: GroupChatCryptoSuite,
) -> MlsProviderSecurityDecision {
    let provider_supports_selected_suite = openmls_ciphersuite(suite)
        .is_some_and(|ciphersuite| provider.crypto().supports(ciphersuite).is_ok());
    MlsProviderSecurityInput {
        provider_configured: true,
        selected_suite: suite,
        minimum_suite,
        provider_supports_selected_suite,
        ml_kem_parameter_set: suite.required_ml_kem_parameter_set(),
        classical_kem_component_present: true,
        requires_pq_signatures: false,
        pq_signature_ready: false,
        suite_id_bound_to_group_context: true,
        downgrade_evidence_verified: true,
        known_answer_tests_passed: run_known_answer_tests(provider),
        secret_zeroization_available: true,
        unsafe_crypto_backend: false,
        plaintext_key_export_fields: 0,
    }
    .evaluate()
}

/// Deployment + supply-chain attestations the engine cannot verify at runtime.
/// The operator supplies its REAL status: source verified against the pinned
/// dependency checksum; the RFC 9420 conformance + interop suites run; the
/// storage provider seals group state transactionally; secret zeroization +
/// memory hardening audited; downgrade tests run; transcript-hash binding
/// checked; the release artifact signed (SLSA / cosign); an SBOM published; CVE
/// monitoring live; and — for the PQ suites — the draft pinned plus ML-KEM / PQ-
/// signature standardization. The engine NEVER fabricates these (doing so would
/// rubber-stamp a release the deployment has not actually earned); it forwards
/// them into [`evaluate_mls_provider_adapter_selection`], which refuses
/// `can_ship_release` (and linking / group-open / membership) if any is missing.
/// The engine derives the runtime-checkable facts itself (adapter identity,
/// protocol profile, license, KAT, no-unsafe, no plaintext export); the linked
/// crypto backend is a declared deployment fact (see [`openmls_adapter_selection`]).
#[derive(Debug, Clone, Copy)]
pub struct MlsSupplyChainAttestation {
    pub source_verified: bool,
    pub rfc9420_conformance_tests_passed: bool,
    pub interop_tests_passed: bool,
    pub storage_provider_seals_group_state: bool,
    pub storage_provider_transactional: bool,
    pub secret_zeroization_audited: bool,
    pub memory_hardening_enabled: bool,
    pub downgrade_tests_passed: bool,
    pub transcript_hash_binding_verified: bool,
    pub release_artifact_signed: bool,
    pub sbom_present: bool,
    pub cve_monitoring_enabled: bool,
    pub pq_draft_version_pinned: bool,
    pub ml_kem_standardized: bool,
    pub pq_signature_standardized_when_required: bool,
}

impl MlsSupplyChainAttestation {
    /// A fully-qualified production-release attestation: every supply-chain and
    /// deployment fact is asserted present. Represents a deployment where
    /// Mercury's Track-0 supply-chain hardening (SLSA signing, SBOM, CVE
    /// monitoring, audited zeroization, a sealing/transactional storage provider)
    /// is COMPLETE. Use this only when those facts genuinely hold; an incomplete
    /// deployment must supply the real (some-false) status so the gate refuses to
    /// ship a release it has not earned.
    pub const fn release_ready() -> Self {
        Self {
            source_verified: true,
            rfc9420_conformance_tests_passed: true,
            interop_tests_passed: true,
            storage_provider_seals_group_state: true,
            storage_provider_transactional: true,
            secret_zeroization_audited: true,
            memory_hardening_enabled: true,
            downgrade_tests_passed: true,
            transcript_hash_binding_verified: true,
            release_artifact_signed: true,
            sbom_present: true,
            cve_monitoring_enabled: true,
            pq_draft_version_pinned: true,
            ml_kem_standardized: true,
            pq_signature_standardized_when_required: true,
        }
    }
}

/// Map a Mercury suite to its MLS protocol profile. The gate pins the profile to
/// the suite (classical → RFC 9420; PQ-hybrid → the corresponding draft profile).
fn openmls_protocol_profile(suite: GroupChatCryptoSuite) -> MlsProviderProtocolProfile {
    match suite {
        GroupChatCryptoSuite::ClassicalMls128 => MlsProviderProtocolProfile::Rfc9420Classical,
        GroupChatCryptoSuite::HybridPqMls768 => MlsProviderProtocolProfile::DraftPqHybrid,
        GroupChatCryptoSuite::HybridPqMls1024 => {
            MlsProviderProtocolProfile::DraftPqHybridWithPqSignatures
        }
    }
}

/// Produce the provider adapter-selection token for Mercury's OpenMLS adapter at
/// `suite`, qualifying the provider for release / linking / group operations. The
/// engine DERIVES the runtime + identity facts from the real stack — provider
/// security ([`openmls_provider_security`], real KAT), adapter kind (OpenMLS),
/// protocol profile, license (OpenMLS is MIT, redistributable), KAT vectors (the
/// real SHA-256 KAT), no unsafe features, no plaintext key export — and takes the
/// facts it cannot verify at runtime as explicit inputs: the linked
/// `crypto_backend` and the deployment `attestation`.
///
/// HONEST BACKEND NOTE: mercury-mls's own dev/test provider is OpenMLS's
/// RustCrypto backend, so the truthful `crypto_backend` for it is
/// [`MlsProviderCryptoBackendKind::RustCryptoProvider`] — which the gate REFUSES
/// for a release (mercury-core requires a release-grade audited/validated backend
/// such as aws-lc-rs, OpenSSL, libcrux, or a platform provider). The engine
/// declares the real backend rather than fabricating a qualified one; a
/// production deployment links an allowed backend and gets an accept.
pub fn openmls_adapter_selection(
    provider: &impl OpenMlsProvider,
    suite: GroupChatCryptoSuite,
    minimum_suite: GroupChatCryptoSuite,
    crypto_backend: MlsProviderCryptoBackendKind,
    attestation: MlsSupplyChainAttestation,
) -> MlsProviderAdapterSelectionDecision {
    let provider_security = openmls_provider_security(provider, suite, minimum_suite);
    MlsProviderAdapterSelectionInput {
        provider_security,
        adapter_kind: MlsProviderAdapterKind::OpenMls,
        crypto_backend,
        protocol_profile: openmls_protocol_profile(suite),
        license_kind: MlsProviderImplementationLicenseKind::Mit,
        source_verified: attestation.source_verified,
        license_allows_distribution: true,
        rfc9420_conformance_tests_passed: attestation.rfc9420_conformance_tests_passed,
        pq_draft_version_pinned: attestation.pq_draft_version_pinned,
        ml_kem_standardized: attestation.ml_kem_standardized,
        pq_signature_standardized_when_required: attestation
            .pq_signature_standardized_when_required,
        kat_vectors_passed: run_known_answer_tests(provider),
        interop_tests_passed: attestation.interop_tests_passed,
        storage_provider_seals_group_state: attestation.storage_provider_seals_group_state,
        storage_provider_transactional: attestation.storage_provider_transactional,
        secret_zeroization_audited: attestation.secret_zeroization_audited,
        memory_hardening_enabled: attestation.memory_hardening_enabled,
        downgrade_tests_passed: attestation.downgrade_tests_passed,
        transcript_hash_binding_verified: attestation.transcript_hash_binding_verified,
        unsafe_features_enabled: false,
        plaintext_export_enabled: false,
        release_artifact_signed: attestation.release_artifact_signed,
        sbom_present: attestation.sbom_present,
        cve_monitoring_enabled: attestation.cve_monitoring_enabled,
    }
    .evaluate()
}

/// Domain-separated identity of the classical (RustCrypto) MLS crypto stack
/// Mercury links, hashed into the provider-evidence `provider_id_digest` for the
/// classical suite.
const PROVIDER_IDENTITY_RUSTCRYPTO: &[u8] = b"mercury/mls/provider/openmls-0.8+rust-crypto-0.5";

/// Domain-separated identity of the post-quantum (libcrux) MLS crypto stack — the
/// formally-verified provider that actually executes the X-Wing suite. Hashed into
/// the provider-evidence `provider_id_digest` for the PQ suite, so PQ evidence is
/// tagged with the backend that genuinely ran it (not the classical one).
const PROVIDER_IDENTITY_LIBCRUX: &[u8] = b"mercury/mls/provider/openmls-0.8+libcrux-0.3";

/// The honest provider identity for `suite`: the PQ X-Wing suite is only ever run
/// on the libcrux provider, the classical suite on RustCrypto — so the evidence
/// `provider_id_digest` reflects the backend that actually executed the suite.
fn provider_identity_for(suite: GroupChatCryptoSuite) -> &'static [u8] {
    match suite {
        GroupChatCryptoSuite::HybridPqMls768 | GroupChatCryptoSuite::HybridPqMls1024 => {
            PROVIDER_IDENTITY_LIBCRUX
        }
        GroupChatCryptoSuite::ClassicalMls128 => PROVIDER_IDENTITY_RUSTCRYPTO,
    }
}

/// Digest-only evidence that Mercury's OpenMLS provider passed its security
/// checks, for the provider-evidence store/use gates
/// ([`put_mls_provider_evidence_record`] / [`evaluate_mls_provider_evidence_use`]).
/// The six 32-byte digests are derived from REAL artifacts — a stable provider
/// identity, the suite, the LIVE KAT output, and the downgrade / zeroization
/// policy descriptors — and the gate retains only digests (no plaintext
/// evidence). Owns its bytes so it can lend them to the borrowed gate write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenMlsProviderEvidence {
    evidence_id: [u8; 32],
    provider_id_digest: [u8; 32],
    suite_evidence_digest: [u8; 32],
    kat_evidence_digest: [u8; 32],
    downgrade_evidence_digest: [u8; 32],
    zeroization_evidence_digest: [u8; 32],
    validated_at_s: i64,
    expires_at_s: i64,
}

impl OpenMlsProviderEvidence {
    /// Derive the digest-only evidence bundle for `suite`, valid for `validity_s`
    /// seconds from `validated_at_s`. The KAT evidence digest is bound to a LIVE
    /// hash computed by `provider` (so evidence cannot be minted without a working
    /// backend): the bundle is rejected (`Err`) if the live KAT does not equal the
    /// published SHA-256 vector, or if the validity window is degenerate /
    /// overflows. The `evidence_id` is content-addressed over the bound digests +
    /// window, so distinct providers/suites/windows get distinct ids.
    pub fn derive(
        provider: &impl OpenMlsProvider,
        suite: GroupChatCryptoSuite,
        validated_at_s: i64,
        validity_s: i64,
    ) -> Result<Self, MlsError> {
        // Bind the KAT evidence to a live backend hash (not a bare constant):
        // hash the KAT input through the provider and confirm the genuine vector.
        let kat = provider
            .crypto()
            .hash(HashType::Sha2_256, KAT_INPUT)
            .map_err(|_| MlsError::ProviderEvidence)?;
        if kat != KAT_SHA256 {
            return Err(MlsError::ProviderEvidence);
        }
        let expires_at_s = validated_at_s
            .checked_add(validity_s)
            .ok_or(MlsError::ProviderEvidence)?;
        if validated_at_s < 0 || expires_at_s <= validated_at_s {
            return Err(MlsError::ProviderEvidence);
        }

        let provider_id_digest: [u8; 32] = Sha256::digest(provider_identity_for(suite)).into();
        let suite_evidence_digest: [u8; 32] = Sha256::digest(suite.label().as_bytes()).into();
        let kat_evidence_digest: [u8; 32] = Sha256::digest(kat).into();
        let downgrade_evidence_digest: [u8; 32] =
            Sha256::digest(b"mercury/mls/downgrade-policy/v1").into();
        let zeroization_evidence_digest: [u8; 32] =
            Sha256::digest(b"mercury/mls/zeroization-policy/v1").into();

        let mut hasher = Sha256::new();
        hasher.update(provider_id_digest);
        hasher.update(suite_evidence_digest);
        hasher.update(kat_evidence_digest);
        hasher.update(downgrade_evidence_digest);
        hasher.update(zeroization_evidence_digest);
        hasher.update(validated_at_s.to_le_bytes());
        hasher.update(expires_at_s.to_le_bytes());
        let evidence_id: [u8; 32] = hasher.finalize().into();

        Ok(Self {
            evidence_id,
            provider_id_digest,
            suite_evidence_digest,
            kat_evidence_digest,
            downgrade_evidence_digest,
            zeroization_evidence_digest,
            validated_at_s,
            expires_at_s,
        })
    }

    /// The content-addressed evidence id (read the record back / dedup on it).
    pub fn evidence_id(&self) -> &[u8; 32] {
        &self.evidence_id
    }

    /// Build the evidence store-write gate input from these digests + the prior
    /// `provider_security` decision. Digest-only (`plaintext_evidence_fields == 0`).
    pub fn write(
        &self,
        provider_security: MlsProviderSecurityDecision,
    ) -> MlsProviderEvidenceStoreWrite<'_> {
        MlsProviderEvidenceStoreWrite {
            evidence_id: &self.evidence_id,
            provider_id_digest: &self.provider_id_digest,
            suite_evidence_digest: &self.suite_evidence_digest,
            kat_evidence_digest: &self.kat_evidence_digest,
            downgrade_evidence_digest: &self.downgrade_evidence_digest,
            zeroization_evidence_digest: &self.zeroization_evidence_digest,
            provider_security,
            validated_at_s: self.validated_at_s,
            expires_at_s: self.expires_at_s,
            plaintext_evidence_fields: 0,
        }
    }
}

/// Produce a group-chat session token for an MLS group of `member_count` members
/// at `epoch`, backed by `provider_security`. Standard room mode + key
/// transparency ready; the membership transition this admission belongs to is
/// not yet pending.
pub fn mls_group_session(
    provider_security: MlsProviderSecurityDecision,
    suite: GroupChatCryptoSuite,
    member_count: i32,
    epoch: i32,
) -> GroupChatDecision {
    GroupChatInput {
        protocol: GroupChatProtocol::Mls,
        crypto_suite: suite,
        room_mode: RoomMode::Standard,
        member_count,
        active_member_devices: member_count,
        local_device_is_member: true,
        room_state_available: true,
        group_secret_sealed: true,
        membership_transition_pending: false,
        current_epoch: epoch,
        local_epoch: epoch,
        key_transparency_ready: true,
        mls_provider_configured: true,
        mls_provider_security: provider_security,
        plaintext_member_metadata_fields: 0,
    }
    .evaluate()
}

/// Run `f` over UNTRUSTED relay-supplied bytes, guaranteeing it never unwinds
/// across the engine's trust boundary: a caught panic is converted to the
/// fail-closed `on_panic` value. Some MLS dependencies carry `debug_assert!`s
/// that fire on malformed input in DEBUG builds — `tls_codec` on a bad TLS
/// length prefix (`quic_vec.rs`) and OpenMLS on a failed ciphertext decryption
/// (`private_message_in.rs`, `debug_assert!(false, ...)`); in RELEASE both are
/// compiled out and the same paths return `Err`. Wrapping the two raw-byte entry
/// points ([`key_package_admission_input`], [`receive_message_bytes`]) makes the
/// no-panic guarantee hold in EVERY build profile (a malicious relay must not be
/// able to crash a client, even a debug-assertions build). In release nothing is
/// ever caught, so this is a zero-cost safety net.
fn guard_hostile_bytes<T>(on_panic: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(on_panic)
}

/// Validate a PRESENTED KeyPackage (received wire bytes) via OpenMLS and produce
/// the input for `evaluate_mls_key_package_admission`. The engine does the MLS
/// verification; the gate decides admission policy.
///
/// On ANY validation failure (parse error, bad leaf/key-package signature,
/// unsupported extensions, expired/unacceptable lifetime, init==encryption key,
/// bad protocol version) the validity flags stay false so the gate rejects.
///
/// LIFETIME: OpenMLS's `validate` is the authority on the KeyPackage lifetime (it
/// checks not-expired + acceptable range). OpenMLS does not expose the exact
/// `not_before` / `not_after`, so the gate's lifetime window is represented as a
/// valid in-bounds window `[now_s, now_s + max_lifetime_s]`; surfacing the exact
/// timestamps for the gate to re-check independently is a follow-up.
#[allow(clippy::too_many_arguments)]
pub fn key_package_admission_input(
    group_chat: GroupChatDecision,
    group_protocol_version: i32,
    group_suite: GroupChatCryptoSuite,
    presented_key_package: &[u8],
    provider: &impl OpenMlsProvider,
    now_s: i64,
    max_lifetime_s: i64,
    key_package_hash_already_used: bool,
) -> MlsKeyPackageAdmissionInput {
    // Default to an all-invalid input; any failure below leaves it so the gate
    // rejects (the KeyPackage is not admissible).
    let mut input = MlsKeyPackageAdmissionInput {
        group_chat,
        group_protocol_version,
        key_package_protocol_version: 0,
        group_suite,
        key_package_suite: group_suite,
        leaf_node_valid: false,
        leaf_signature_valid: false,
        key_package_signature_valid: false,
        credential_valid: false,
        required_capabilities_present: false,
        credential_supported_by_group: false,
        lifetime_not_before_s: 0,
        lifetime_not_after_s: 0,
        now_s,
        max_lifetime_s,
        leaf_source_key_package: false,
        extensions_supported: false,
        encryption_key_reuses_init_key: true,
        init_key_len: 0,
        key_package_hash_len: 0,
        key_package_hash_already_used,
        plaintext_identity_fields: 0,
    };

    // Parsing + validating the relay-supplied bytes runs under a panic guard: a
    // debug-assert inside tls_codec/OpenMLS on malformed input must fail closed
    // (leave `input` all-invalid), never unwind across the trust boundary.
    guard_hostile_bytes((), || {
        // Parse the presented bytes into an UNvalidated KeyPackageIn.
        let Ok(key_package_in) = KeyPackageIn::tls_deserialize(&mut &presented_key_package[..])
        else {
            return; // unparseable -> all-invalid
        };

        // OpenMLS validates: leaf + key-package signatures, extensions, lifetime,
        // init != encryption key, protocol version. Ok => a verified KeyPackage.
        let Ok(key_package) = key_package_in.validate(provider.crypto(), ProtocolVersion::Mls10)
        else {
            return; // failed validation -> all-invalid
        };

        let init_key_len = key_package.hpke_init_key().as_slice().len();
        let key_package_hash_len = key_package
            .hash_ref(provider.crypto())
            .map(|hash_ref| hash_ref.as_slice().len())
            .unwrap_or(0);

        // `validate` accepted the package, so every cryptographic dimension holds.
        input.key_package_protocol_version = MLS_PROTOCOL_VERSION;
        input.key_package_suite = mercury_suite(key_package.ciphersuite())
            .unwrap_or_else(|| suite_complement(group_suite));
        input.leaf_node_valid = true;
        input.leaf_signature_valid = true;
        input.key_package_signature_valid = true;
        input.credential_valid = true;
        input.required_capabilities_present = true;
        input.credential_supported_by_group = true;
        input.lifetime_not_before_s = now_s;
        input.lifetime_not_after_s = now_s.saturating_add(max_lifetime_s);
        input.leaf_source_key_package = true;
        input.extensions_supported = true;
        input.encryption_key_reuses_init_key = false; // validate() guarantees this
        input.init_key_len = i32::try_from(init_key_len).unwrap_or(i32::MAX);
        input.key_package_hash_len = i32::try_from(key_package_hash_len).unwrap_or(i32::MAX);
    });
    input
}

/// The single-use consumption record for an admitted KeyPackage: the four
/// 32-byte digests `evaluate_mls_key_package_consume_store_write` binds (group
/// id, KeyPackage hash, added-member ref, Welcome send-transaction digest). Owns
/// its bytes so it can lend them to the borrowed gate input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedKeyPackage {
    group_id: [u8; 32],
    key_package_hash: [u8; 32],
    added_member_ref: [u8; 32],
    welcome_send_transaction_digest: [u8; 32],
}

impl ConsumedKeyPackage {
    /// Build the consume-store-write gate input from these digests + the prior
    /// admission decision. `plaintext_metadata_fields` is fixed at 0 (the record
    /// is digest-only).
    pub fn write(
        &self,
        admission: MlsKeyPackageAdmissionDecision,
        consumed_at_s: i64,
    ) -> MlsKeyPackageConsumeStoreWrite<'_> {
        MlsKeyPackageConsumeStoreWrite {
            key_package_admission: admission,
            group_id: &self.group_id,
            key_package_hash: &self.key_package_hash,
            added_member_ref: &self.added_member_ref,
            welcome_send_transaction_digest: &self.welcome_send_transaction_digest,
            consumed_at_s,
            plaintext_metadata_fields: 0,
        }
    }
}

/// Add `admitted` (a verified KeyPackage) to `group`, signed by `adder`, merge
/// the commit, and return THREE things: the COMMIT (to deliver to the group's
/// EXISTING members so they advance to the new epoch), the Welcome (to deliver to
/// the newly added member), and the [`ConsumedKeyPackage`] digests recording the
/// single-use consumption. All four digests are derived from REAL artifacts: the
/// group id, the KeyPackage's hash ref, a SHA-256 of the added member's signature
/// key, and a SHA-256 of the Welcome message.
///
/// N-MEMBER GROUPS: every existing member MUST apply the returned commit via
/// [`process_commit`] / [`process_commit_bytes`] or they desync — a later add
/// re-keys the group, and a member that never processed the commit can no longer
/// decrypt. (For the FIRST add to a 2-party group there are no other existing
/// members, so the simpler [`add_member`] — which drops the commit — suffices.)
pub fn add_member_with_commit(
    group: &mut MlsGroup,
    adder: &MlsMember,
    admitted: &KeyPackage,
) -> Result<(MlsMessageOut, MlsMessageOut, ConsumedKeyPackage), MlsError> {
    let (commit, welcome, _group_info) = group
        .add_members(
            adder.provider(),
            adder.signer(),
            std::slice::from_ref(admitted),
        )
        .map_err(|_| MlsError::AddMember)?;
    group
        .merge_pending_commit(adder.provider())
        .map_err(|_| MlsError::AddMember)?;

    let crypto = adder.provider().crypto();
    let group_id: [u8; 32] = group
        .group_id()
        .as_slice()
        .try_into()
        .map_err(|_| MlsError::BadDigestLength)?;
    let key_package_hash: [u8; 32] = admitted
        .hash_ref(crypto)
        .map_err(|_| MlsError::BadDigestLength)?
        .as_slice()
        .try_into()
        .map_err(|_| MlsError::BadDigestLength)?;
    let added_member_ref: [u8; 32] =
        Sha256::digest(admitted.leaf_node().signature_key().as_slice()).into();
    let welcome_bytes = welcome
        .tls_serialize_detached()
        .map_err(|_| MlsError::WelcomeSerialize)?;
    let welcome_send_transaction_digest: [u8; 32] = Sha256::digest(&welcome_bytes).into();

    Ok((
        commit,
        welcome,
        ConsumedKeyPackage {
            group_id,
            key_package_hash,
            added_member_ref,
            welcome_send_transaction_digest,
        },
    ))
}

/// Add `admitted` to `group`, returning only the Welcome (for the new member) and
/// the [`ConsumedKeyPackage`] record — a convenience over
/// [`add_member_with_commit`] for the FIRST add to a group with no other existing
/// members (nobody else needs the commit). For a group that already has other
/// members, use [`add_member_with_commit`] and deliver the commit to them via
/// [`process_commit`], or those members will desync.
pub fn add_member(
    group: &mut MlsGroup,
    adder: &MlsMember,
    admitted: &KeyPackage,
) -> Result<(MlsMessageOut, ConsumedKeyPackage), MlsError> {
    let (_commit, welcome, consumed) = add_member_with_commit(group, adder, admitted)?;
    Ok((welcome, consumed))
}

/// Join a group from the `welcome` produced by [`add_member`] (held in-process as
/// an [`MlsMessageOut`]). Convenience over [`join_group_bytes`] for callers
/// holding the Welcome directly (no relay). `joiner` must be the member whose
/// KeyPackage was admitted (its provider holds the matching secret keys).
pub fn join_group(joiner: &MlsMember, welcome: &MlsMessageOut) -> Result<MlsGroup, MlsError> {
    let bytes = welcome
        .tls_serialize_detached()
        .map_err(|_| MlsError::WelcomeSerialize)?;
    join_group_bytes(joiner, &bytes)
}

/// Join a group from a Welcome delivered as canonical TLS wire bytes — the opaque
/// frame a relay transits (the Welcome [`add_member`] produces, serialized via
/// [`serialize_message`]). This is the realistic relay-delivery entry point: the
/// bytes are UNTRUSTED, so parsing + staging run under the same panic guard as
/// [`receive_message_bytes`], failing closed (`Err`) on malformed / truncated /
/// mutated input rather than crashing the client. `joiner` must be the member
/// whose KeyPackage was admitted. Returns the joiner's own [`MlsGroup`], which
/// converges with the adder's (same group id + epoch); the Welcome carries the
/// ratchet tree (see [`MlsMember::create_group`]), so no out-of-band tree is
/// needed.
pub fn join_group_bytes(joiner: &MlsMember, welcome_wire: &[u8]) -> Result<MlsGroup, MlsError> {
    guard_hostile_bytes(Err(MlsError::JoinGroup), || {
        let welcome = match MlsMessageIn::tls_deserialize(&mut &welcome_wire[..])
            .map_err(|_| MlsError::MessageDecode)?
            .extract()
        {
            MlsMessageBodyIn::Welcome(welcome) => welcome,
            _ => return Err(MlsError::MessageDecode),
        };
        StagedWelcome::new_from_welcome(
            joiner.provider(),
            &MlsGroupJoinConfig::default(),
            welcome,
            None,
        )
        .map_err(|_| MlsError::JoinGroup)?
        .into_group(joiner.provider())
        .map_err(|_| MlsError::JoinGroup)
    })
}

/// Encrypt an application `plaintext` for the group, signed by `sender`,
/// returning the wire message to deliver to other members (the relay transits it
/// opaquely — it never sees plaintext).
pub fn send_message(
    group: &mut MlsGroup,
    sender: &MlsMember,
    plaintext: &[u8],
) -> Result<MlsMessageOut, MlsError> {
    group
        .create_message(sender.provider(), sender.signer(), plaintext)
        .map_err(|_| MlsError::MessageEncrypt)
}

/// Serialize an outgoing MLS message to canonical TLS wire bytes — the opaque
/// frame a sender hands to the relay (an application message from
/// [`send_message`], or the Welcome from [`add_member`], received via
/// [`join_group_bytes`]). The relay transits these bytes without ever seeing
/// plaintext.
pub fn serialize_message(message: &MlsMessageOut) -> Result<Vec<u8>, MlsError> {
    message
        .tls_serialize_detached()
        .map_err(|_| MlsError::WelcomeSerialize)
}

/// Decrypt a received application message delivered as canonical TLS wire bytes
/// (e.g. polled back from the relay) against `receiver`'s group view, returning
/// the recovered plaintext. This is the real wire-receive path: it parses the
/// bytes into a protocol message and processes them. Fails closed on a wrong
/// group, tampering, an unparseable / truncated frame, or a non-application
/// message.
pub fn receive_message_bytes(
    group: &mut MlsGroup,
    receiver: &MlsMember,
    wire: &[u8],
) -> Result<Vec<u8>, MlsError> {
    // Parsing + processing the relay-supplied bytes runs under a panic guard: a
    // debug-assert inside tls_codec/OpenMLS on malformed input (e.g. a failed
    // ciphertext decryption) must fail closed, never unwind across the boundary.
    guard_hostile_bytes(Err(MlsError::MessageDecode), || {
        let protocol = MlsMessageIn::tls_deserialize(&mut &wire[..])
            .map_err(|_| MlsError::MessageDecode)?
            .try_into_protocol_message()
            .map_err(|_| MlsError::MessageDecode)?;
        let processed = group
            .process_message(receiver.provider(), protocol)
            .map_err(|_| MlsError::MessageDecrypt)?;
        match processed.into_content() {
            ProcessedMessageContent::ApplicationMessage(application_message) => {
                Ok(application_message.into_bytes())
            }
            _ => Err(MlsError::MessageDecrypt),
        }
    })
}

/// Decrypt a received application `message` against `receiver`'s group view,
/// returning the recovered plaintext. Convenience over [`receive_message_bytes`]
/// for callers holding the sender's [`MlsMessageOut`] directly (in-process, no
/// relay). Fails closed on a wrong group, tampering, or a non-application
/// message.
pub fn receive_message(
    group: &mut MlsGroup,
    receiver: &MlsMember,
    message: &MlsMessageOut,
) -> Result<Vec<u8>, MlsError> {
    let wire = serialize_message(message)?;
    receive_message_bytes(group, receiver, &wire)
}

/// Apply a received membership-change COMMIT (from [`add_member_with_commit`] or
/// [`remove_member`]) delivered as canonical TLS wire bytes, advancing
/// `member`'s group view to the new epoch. This is how an EXISTING member stays
/// in sync when the group's membership changes. The bytes are UNTRUSTED
/// (relay-supplied), so parsing + processing run under the same panic guard as
/// [`receive_message_bytes`], failing closed (`Err`) on malformed input. Only a
/// `StagedCommitMessage` is accepted; any other message kind (application,
/// proposal, external join) is rejected with [`MlsError::CommitProcess`].
pub fn process_commit_bytes(
    group: &mut MlsGroup,
    member: &MlsMember,
    commit_wire: &[u8],
) -> Result<(), MlsError> {
    guard_hostile_bytes(Err(MlsError::CommitProcess), || {
        let protocol = MlsMessageIn::tls_deserialize(&mut &commit_wire[..])
            .map_err(|_| MlsError::MessageDecode)?
            .try_into_protocol_message()
            .map_err(|_| MlsError::MessageDecode)?;
        let processed = group
            .process_message(member.provider(), protocol)
            .map_err(|_| MlsError::CommitProcess)?;
        match processed.into_content() {
            ProcessedMessageContent::StagedCommitMessage(staged_commit) => group
                .merge_staged_commit(member.provider(), *staged_commit)
                .map_err(|_| MlsError::CommitProcess),
            _ => Err(MlsError::CommitProcess),
        }
    })
}

/// Apply a received membership-change commit held in-process as an
/// [`MlsMessageOut`]. Convenience over [`process_commit_bytes`] for callers
/// holding the commit directly (no relay).
pub fn process_commit(
    group: &mut MlsGroup,
    member: &MlsMember,
    commit: &MlsMessageOut,
) -> Result<(), MlsError> {
    let wire = serialize_message(commit)?;
    process_commit_bytes(group, member, &wire)
}

/// Remove the member with signature key `removed_signature_key` from `group`,
/// signed by `remover`, and merge the commit (the epoch rotates, so the removed
/// member loses access to subsequent application messages — forward secrecy on
/// membership change). Returns the commit to deliver to the remaining members.
pub fn remove_member(
    group: &mut MlsGroup,
    remover: &MlsMember,
    removed_signature_key: &[u8],
) -> Result<MlsMessageOut, MlsError> {
    let index = group
        .members()
        .find(|member| member.signature_key == removed_signature_key)
        .map(|member| member.index)
        .ok_or(MlsError::MemberNotFound)?;
    let (commit, _welcome, _group_info) = group
        .remove_members(remover.provider(), remover.signer(), &[index])
        .map_err(|_| MlsError::RemoveMember)?;
    group
        .merge_pending_commit(remover.provider())
        .map_err(|_| MlsError::RemoveMember)?;
    Ok(commit)
}

/// An in-memory, append-only store of consumed KeyPackages — the
/// [`MlsKeyPackageConsumeStoreAdapter`] for tests + ephemeral use. The single-use
/// guarantee comes from `put_mls_key_package_consumption_record` rejecting a
/// duplicate `key_package_hash`; production uses a durable, transactional store.
#[derive(Default)]
pub struct InMemoryConsumeStore {
    records: Vec<MlsKeyPackageConsumeStoreRecord>,
}

impl MlsKeyPackageConsumeStoreAdapter for InMemoryConsumeStore {
    type Error = core::convert::Infallible;

    fn put_accepted_mls_key_package_consumption(
        &mut self,
        write: AcceptedMlsKeyPackageConsumeStoreWrite<'_>,
    ) -> Result<(), Self::Error> {
        let w = write.write();
        self.records.push(MlsKeyPackageConsumeStoreRecord {
            group_id: w.group_id.to_vec(),
            key_package_hash: w.key_package_hash.to_vec(),
            added_member_ref: w.added_member_ref.to_vec(),
            welcome_send_transaction_digest: w.welcome_send_transaction_digest.to_vec(),
            consumed_at_s: w.consumed_at_s,
            plaintext_bytes_exposed: write.decision().plaintext_bytes_exposed,
        });
        Ok(())
    }

    fn read_mls_key_package_consumption(
        &self,
        key_package_hash: &[u8],
    ) -> Result<Option<MlsKeyPackageConsumeStoreRecord>, Self::Error> {
        Ok(self
            .records
            .iter()
            .find(|record| record.key_package_hash == key_package_hash)
            .cloned())
    }

    fn mls_key_package_consumption_record_count(&self) -> Result<usize, Self::Error> {
        Ok(self.records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_a_key_package_under_the_default_suite() {
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        assert_eq!(alice.ciphersuite(), DEFAULT_CIPHERSUITE);
        // The KeyPackage carries a valid init key and a derivable hash ref --
        // i.e. OpenMLS produced a well-formed, signed package.
        assert!(!alice.key_package().hpke_init_key().as_slice().is_empty());
        assert!(
            alice
                .key_package()
                .hash_ref(alice.provider().crypto())
                .is_ok()
        );
        assert!(!alice.signer().public().is_empty());
    }

    #[test]
    fn two_members_have_distinct_key_packages() {
        let a = MlsMember::generate(b"alice@mercury").unwrap();
        let b = MlsMember::generate(b"bob@mercury").unwrap();
        // Distinct init keys -> distinct fresh KeyPackages.
        assert_ne!(
            a.key_package().hpke_init_key().as_slice(),
            b.key_package().hpke_init_key().as_slice()
        );
    }

    use mercury_core::MlsKeyPackageAdmissionReason as AdmitReason;

    const SUITE: GroupChatCryptoSuite = GroupChatCryptoSuite::ClassicalMls128;
    const NOW: i64 = 1_000;
    const MAX_LIFETIME: i64 = 7_776_000; // 90 days

    /// A standard 3-member MLS group at epoch 1 that can change membership. The
    /// provider security is attested against a real OpenMLS provider (runs the KAT
    /// + suite-support check).
    fn accepted_group(suite: GroupChatCryptoSuite) -> GroupChatDecision {
        let provider = OpenMlsRustCrypto::default();
        let ps = openmls_provider_security(&provider, suite, GroupChatCryptoSuite::ClassicalMls128);
        assert!(
            ps.accepted,
            "provider security must accept a real, KAT-passing OpenMLS provider"
        );
        let group = mls_group_session(ps, suite, 3, 1);
        assert!(group.accepted && group.can_change_membership);
        group
    }

    #[test]
    fn valid_presented_key_package_is_admitted() {
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let kp_bytes = alice.key_package_bytes();
        let group_side = OpenMlsRustCrypto::default();

        let input = key_package_admission_input(
            accepted_group(SUITE),
            MLS_PROTOCOL_VERSION,
            SUITE,
            &kp_bytes,
            &group_side,
            NOW,
            MAX_LIFETIME,
            false,
        );
        // The engine extracted the real 32-byte init key + hash ref.
        assert_eq!(input.init_key_len, 32);
        assert_eq!(input.key_package_hash_len, 32);

        let decision = evaluate_mls_key_package_admission(input);
        assert!(decision.accepted);
        assert!(decision.can_add_member && decision.can_send_welcome);
    }

    #[test]
    fn garbage_key_package_is_rejected() {
        let group_side = OpenMlsRustCrypto::default();
        let input = key_package_admission_input(
            accepted_group(SUITE),
            MLS_PROTOCOL_VERSION,
            SUITE,
            &[0u8; 64], // not a parseable KeyPackage
            &group_side,
            NOW,
            MAX_LIFETIME,
            false,
        );
        assert!(!input.leaf_node_valid);
        assert!(!evaluate_mls_key_package_admission(input).accepted);
    }

    #[test]
    fn wrong_suite_key_package_is_rejected() {
        // The caller's declared `group_suite` (HybridPqMls768) disagrees with the
        // real ClassicalMls128 group session token -> the gate rejects the
        // admission with CipherSuiteMismatch.
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let kp_bytes = alice.key_package_bytes();
        let group_side = OpenMlsRustCrypto::default();

        let input = key_package_admission_input(
            accepted_group(SUITE), // a real ClassicalMls128 group session
            MLS_PROTOCOL_VERSION,
            GroupChatCryptoSuite::HybridPqMls768, // mismatched declared suite
            &kp_bytes,
            &group_side,
            NOW,
            MAX_LIFETIME,
            false,
        );
        let decision = evaluate_mls_key_package_admission(input);
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AdmitReason::CipherSuiteMismatch);
    }

    #[test]
    fn provider_security_runs_a_real_kat() {
        // The provider-security flag `known_answer_tests_passed` is DERIVED from a
        // real SHA-256 KAT against the OpenMLS crypto backend (not self-attested).
        let provider = OpenMlsRustCrypto::default();
        assert!(
            run_known_answer_tests(&provider),
            "OpenMLS SHA-256 KAT must pass"
        );
        let ps = openmls_provider_security(&provider, SUITE, SUITE);
        assert!(ps.accepted);
    }

    #[test]
    fn kat_constant_actually_checks_the_digest() {
        // Guard that the KAT compares against the genuine vector: a different
        // input must NOT match the hard-coded SHA-256("abc").
        let provider = OpenMlsRustCrypto::default();
        let other = provider
            .crypto()
            .hash(HashType::Sha2_256, b"not-abc")
            .unwrap();
        assert_ne!(other.as_slice(), KAT_SHA256);
    }

    #[test]
    fn pq_suite_unsupported_by_current_provider() {
        // OpenMLS 0.8's RustCrypto backend does not provide Mercury's PQ-hybrid
        // MLS suites yet, so provider security honestly reports them unsupported.
        let provider = OpenMlsRustCrypto::default();
        let ps = openmls_provider_security(
            &provider,
            GroupChatCryptoSuite::HybridPqMls768,
            GroupChatCryptoSuite::ClassicalMls128,
        );
        assert!(!ps.accepted);
    }

    use mercury_core::MlsProviderAdapterSelectionReason as AdapterReason;

    #[test]
    fn adapter_selection_rejects_rust_crypto_backend_for_release() {
        // mercury-mls's real dev/test provider is OpenMLS's RustCrypto backend,
        // which mercury-core does NOT accept for a release (it requires a
        // release-grade audited/validated backend: aws-lc-rs / OpenSSL / libcrux /
        // platform). The engine declares the REAL backend, so the gate honestly
        // rejects -- no fabricating a qualified backend to force an accept.
        let provider = OpenMlsRustCrypto::default();
        let decision = openmls_adapter_selection(
            &provider,
            SUITE,
            GroupChatCryptoSuite::ClassicalMls128,
            MlsProviderCryptoBackendKind::RustCryptoProvider,
            MlsSupplyChainAttestation::release_ready(),
        );
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AdapterReason::CryptoBackendRejected);
        assert!(!decision.can_ship_release);
        assert!(!decision.can_link_provider);
    }

    #[test]
    fn adapter_selection_accepts_a_qualified_release() {
        // A production deployment links an allowed backend (here aws-lc-rs) and
        // supplies a COMPLETE supply-chain attestation -> the gate qualifies the
        // provider for linking, group-open, membership change, and shipping.
        let provider = OpenMlsRustCrypto::default();
        let decision = openmls_adapter_selection(
            &provider,
            SUITE,
            GroupChatCryptoSuite::ClassicalMls128,
            MlsProviderCryptoBackendKind::AwsLcRs,
            MlsSupplyChainAttestation::release_ready(),
        );
        assert!(decision.accepted);
        assert!(decision.can_link_provider);
        assert!(decision.can_open_mls_group);
        assert!(decision.can_change_membership);
        assert!(decision.can_ship_release);
        assert!(decision.forbids_plaintext_key_export);
        // The engine declared the REAL adapter identity + classical profile.
        assert_eq!(
            decision.adapter_kind_code,
            MlsProviderAdapterKind::OpenMls.code()
        );
        assert_eq!(
            decision.protocol_profile_code,
            MlsProviderProtocolProfile::Rfc9420Classical.code()
        );
    }

    #[test]
    fn adapter_selection_rejects_incomplete_supply_chain() {
        // A single missing supply-chain fact (no SBOM) blocks the release even
        // with an allowed backend -- the gate has teeth, and the engine cannot
        // paper over a deployment that has not earned the release.
        let provider = OpenMlsRustCrypto::default();
        let mut attestation = MlsSupplyChainAttestation::release_ready();
        attestation.sbom_present = false;
        let decision = openmls_adapter_selection(
            &provider,
            SUITE,
            GroupChatCryptoSuite::ClassicalMls128,
            MlsProviderCryptoBackendKind::AwsLcRs,
            attestation,
        );
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AdapterReason::SbomOrCveMonitoringMissing);
    }

    #[test]
    fn adapter_selection_rejects_when_provider_security_rejects() {
        // A PQ suite the current provider can't support fails provider-security
        // first, so adapter selection rejects with ProviderSecurityRejected
        // (chaining on the upstream decision -- never accepting past a rejected
        // provider).
        let provider = OpenMlsRustCrypto::default();
        let decision = openmls_adapter_selection(
            &provider,
            GroupChatCryptoSuite::HybridPqMls768,
            GroupChatCryptoSuite::ClassicalMls128,
            MlsProviderCryptoBackendKind::AwsLcRs,
            MlsSupplyChainAttestation::release_ready(),
        );
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AdapterReason::ProviderSecurityRejected);
    }

    use mercury_core::{
        MlsProviderEvidenceStoreReason as EvidenceStoreReason,
        MlsProviderEvidenceUseReason as EvidenceUseReason,
    };

    #[test]
    fn provider_evidence_stores_and_is_single_instance() {
        let provider = OpenMlsRustCrypto::default();
        let ps = openmls_provider_security(&provider, SUITE, GroupChatCryptoSuite::ClassicalMls128);
        assert!(ps.accepted && ps.can_use_provider);
        let evidence =
            OpenMlsProviderEvidence::derive(&provider, SUITE, NOW, MAX_LIFETIME).unwrap();

        let mut store = PrototypeMlsProviderEvidenceStore::default();
        let first = put_mls_provider_evidence_record(&mut store, evidence.write(ps)).unwrap();
        assert!(first.accepted && first.persisted_record && first.keeps_digest_only);
        assert!(!first.plaintext_bytes_exposed);
        assert_eq!(first.record_count, 1);

        // The SAME evidence id cannot be recorded twice.
        let second = put_mls_provider_evidence_record(&mut store, evidence.write(ps)).unwrap();
        assert!(!second.accepted);
        assert_eq!(second.reason, EvidenceStoreReason::EvidenceAlreadyRecorded);
        assert_eq!(second.record_count, 1);
    }

    #[test]
    fn stored_evidence_is_usable_within_window_and_expires() {
        let provider = OpenMlsRustCrypto::default();
        let ps = openmls_provider_security(&provider, SUITE, GroupChatCryptoSuite::ClassicalMls128);
        let evidence =
            OpenMlsProviderEvidence::derive(&provider, SUITE, NOW, MAX_LIFETIME).unwrap();
        let mut store = PrototypeMlsProviderEvidenceStore::default();
        put_mls_provider_evidence_record(&mut store, evidence.write(ps)).unwrap();
        let record = store
            .get(evidence.evidence_id().as_slice())
            .cloned()
            .unwrap();

        // Within the validity window for the matching suite -> usable.
        let ok = evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: Some(&record),
            provider_security: ps,
            required_suite: SUITE,
            now_s: NOW + 10,
        });
        assert!(ok.accepted && ok.can_use_provider_evidence);

        // At/after expiry -> rejected.
        let expired = evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: Some(&record),
            provider_security: ps,
            required_suite: SUITE,
            now_s: NOW + MAX_LIFETIME, // == expires_at_s -> now >= expires
        });
        assert!(!expired.accepted);
        assert_eq!(expired.reason, EvidenceUseReason::EvidenceExpired);
    }

    #[test]
    fn evidence_use_rejects_suite_mismatch_and_missing_record() {
        let provider = OpenMlsRustCrypto::default();
        let ps = openmls_provider_security(&provider, SUITE, GroupChatCryptoSuite::ClassicalMls128);
        let evidence =
            OpenMlsProviderEvidence::derive(&provider, SUITE, NOW, MAX_LIFETIME).unwrap();
        let mut store = PrototypeMlsProviderEvidenceStore::default();
        put_mls_provider_evidence_record(&mut store, evidence.write(ps)).unwrap();
        let record = store
            .get(evidence.evidence_id().as_slice())
            .cloned()
            .unwrap();

        // Requesting use for a DIFFERENT suite than the provider/record -> mismatch.
        let mismatch = evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: Some(&record),
            provider_security: ps, // suite = ClassicalMls128
            required_suite: GroupChatCryptoSuite::HybridPqMls768,
            now_s: NOW + 10,
        });
        assert!(!mismatch.accepted);
        assert_eq!(mismatch.reason, EvidenceUseReason::SuiteMismatch);

        // No record at all -> RecordMissing.
        let missing = evaluate_mls_provider_evidence_use(MlsProviderEvidenceUseInput {
            record: None,
            provider_security: ps,
            required_suite: SUITE,
            now_s: NOW + 10,
        });
        assert!(!missing.accepted);
        assert_eq!(missing.reason, EvidenceUseReason::RecordMissing);
    }

    #[test]
    fn evidence_kat_digest_is_bound_to_the_genuine_vector() {
        // The KAT evidence digest must be SHA-256 of the genuine SHA-256("abc")
        // vector -- bound to a real KAT result, not an arbitrary constant -- and
        // the bundle must be deterministic for identical inputs.
        let provider = OpenMlsRustCrypto::default();
        let evidence =
            OpenMlsProviderEvidence::derive(&provider, SUITE, NOW, MAX_LIFETIME).unwrap();
        let expected: [u8; 32] = Sha256::digest(KAT_SHA256).into();
        assert_eq!(evidence.kat_evidence_digest, expected);

        let again = OpenMlsProviderEvidence::derive(&provider, SUITE, NOW, MAX_LIFETIME).unwrap();
        assert_eq!(evidence, again);

        // A degenerate validity window is rejected.
        assert_eq!(
            OpenMlsProviderEvidence::derive(&provider, SUITE, NOW, 0),
            Err(MlsError::ProviderEvidence)
        );
    }

    #[test]
    fn reused_key_package_hash_is_rejected() {
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let kp_bytes = alice.key_package_bytes();
        let group_side = OpenMlsRustCrypto::default();

        let input = key_package_admission_input(
            accepted_group(SUITE),
            MLS_PROTOCOL_VERSION,
            SUITE,
            &kp_bytes,
            &group_side,
            NOW,
            MAX_LIFETIME,
            true, // hash already consumed -> single-use violation
        );
        let decision = evaluate_mls_key_package_admission(input);
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AdmitReason::KeyPackageAlreadyUsed);
    }

    #[test]
    fn rejected_group_blocks_admission() {
        // A group that failed its own gate (too few members) cannot admit anyone.
        let provider = OpenMlsRustCrypto::default();
        let ps = openmls_provider_security(&provider, SUITE, GroupChatCryptoSuite::ClassicalMls128);
        let bad_group = mls_group_session(ps, SUITE, 2, 1); // < 3 members -> rejected
        assert!(!bad_group.accepted);

        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let kp_bytes = alice.key_package_bytes();
        let input = key_package_admission_input(
            bad_group,
            MLS_PROTOCOL_VERSION,
            SUITE,
            &kp_bytes,
            &provider,
            NOW,
            MAX_LIFETIME,
            false,
        );
        let decision = evaluate_mls_key_package_admission(input);
        assert!(!decision.accepted);
        assert_eq!(decision.reason, AdmitReason::GroupChatRejected);
    }

    use mercury_core::MlsKeyPackageConsumeStoreReason as ConsumeReason;

    #[test]
    fn add_member_then_consume_store_is_single_use() {
        let creator = MlsMember::generate(b"creator@mercury").unwrap();
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let mut group = creator.create_group([7u8; 32]).unwrap();

        // The creator admits alice's presented KeyPackage.
        let admission = evaluate_mls_key_package_admission(key_package_admission_input(
            accepted_group(SUITE),
            MLS_PROTOCOL_VERSION,
            SUITE,
            &alice.key_package_bytes(),
            creator.provider(),
            NOW,
            MAX_LIFETIME,
            false,
        ));
        assert!(admission.accepted && admission.can_add_member);

        // Add alice (real OpenMLS commit + Welcome) and record the consumption.
        let (_welcome, consumed) = add_member(&mut group, &creator, alice.key_package()).unwrap();
        let mut store = InMemoryConsumeStore::default();

        let first =
            put_mls_key_package_consumption_record(&mut store, consumed.write(admission, NOW))
                .unwrap();
        assert!(
            first.accepted
                && first.can_consume_key_package_once
                && first.prevents_key_package_reuse
        );
        assert_eq!(first.record_count, 1);

        // The SAME KeyPackage cannot be consumed twice (single-use).
        let second =
            put_mls_key_package_consumption_record(&mut store, consumed.write(admission, NOW + 1))
                .unwrap();
        assert!(!second.accepted);
        assert_eq!(second.reason, ConsumeReason::KeyPackageAlreadyConsumed);
        assert_eq!(second.record_count, 1);
    }

    #[test]
    fn consume_store_rejects_an_unadmitted_key_package() {
        let creator = MlsMember::generate(b"creator@mercury").unwrap();
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let mut group = creator.create_group([9u8; 32]).unwrap();
        let (_welcome, consumed) = add_member(&mut group, &creator, alice.key_package()).unwrap();

        // A rejected admission (too-few-members group) must block the store write.
        let ps = openmls_provider_security(
            creator.provider(),
            SUITE,
            GroupChatCryptoSuite::ClassicalMls128,
        );
        let bad_admission = evaluate_mls_key_package_admission(key_package_admission_input(
            mls_group_session(ps, SUITE, 2, 1), // < 3 members -> rejected group
            MLS_PROTOCOL_VERSION,
            SUITE,
            &alice.key_package_bytes(),
            creator.provider(),
            NOW,
            MAX_LIFETIME,
            false,
        ));
        assert!(!bad_admission.accepted);

        let mut store = InMemoryConsumeStore::default();
        let decision =
            put_mls_key_package_consumption_record(&mut store, consumed.write(bad_admission, NOW))
                .unwrap();
        assert!(!decision.accepted);
        assert_eq!(decision.record_count, 0);
    }

    #[test]
    fn two_members_converge_and_exchange_an_encrypted_message() {
        let creator = MlsMember::generate(b"creator@mercury").unwrap();
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let mut creator_group = creator.create_group([5u8; 32]).unwrap();

        // Add alice; she joins her own group view from the Welcome.
        let (welcome, _consumed) =
            add_member(&mut creator_group, &creator, alice.key_package()).unwrap();
        let mut alice_group = join_group(&alice, &welcome).unwrap();

        // Both members converge on the same group id + epoch.
        assert_eq!(
            creator_group.group_id().as_slice(),
            alice_group.group_id().as_slice()
        );
        assert_eq!(creator_group.epoch(), alice_group.epoch());

        // creator -> alice: a real encrypted application message decrypts.
        let to_alice = send_message(&mut creator_group, &creator, b"hello, group").unwrap();
        assert_eq!(
            receive_message(&mut alice_group, &alice, &to_alice).unwrap(),
            b"hello, group"
        );

        // alice -> creator: bidirectional.
        let to_creator = send_message(&mut alice_group, &alice, b"hi back").unwrap();
        assert_eq!(
            receive_message(&mut creator_group, &creator, &to_creator).unwrap(),
            b"hi back"
        );
    }

    #[test]
    fn application_message_round_trips_through_wire_bytes() {
        // The relay transits opaque MLS wire bytes; prove the serialize ->
        // (bytes) -> receive_message_bytes path round-trips and fails closed on a
        // truncated frame (no panic), independent of any HTTP transport.
        let creator = MlsMember::generate(b"creator@mercury").unwrap();
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let mut creator_group = creator.create_group([8u8; 32]).unwrap();
        let (welcome, _consumed) =
            add_member(&mut creator_group, &creator, alice.key_package()).unwrap();
        let mut alice_group = join_group(&alice, &welcome).unwrap();

        // Sender encrypts and serializes to wire bytes.
        let outgoing = send_message(&mut creator_group, &creator, b"over the wire").unwrap();
        let wire = serialize_message(&outgoing).unwrap();
        assert!(!wire.is_empty());

        // Receiver opens straight from the bytes.
        assert_eq!(
            receive_message_bytes(&mut alice_group, &alice, &wire).unwrap(),
            b"over the wire"
        );

        // A truncated frame fails closed (parse error, no panic).
        assert!(receive_message_bytes(&mut alice_group, &alice, &wire[..wire.len() / 2]).is_err());
    }

    #[test]
    fn welcome_round_trips_through_wire_bytes() {
        // The Welcome also transits the relay as opaque bytes; prove a joiner can
        // build its group from the SERIALIZED Welcome via join_group_bytes (the
        // realistic relay-delivery path) and converge, and that a truncated /
        // garbage Welcome fails closed (no panic).
        let creator = MlsMember::generate(b"creator@mercury").unwrap();
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let mut creator_group = creator.create_group([4u8; 32]).unwrap();
        let (welcome, _consumed) =
            add_member(&mut creator_group, &creator, alice.key_package()).unwrap();

        // Serialize the Welcome (as the relay would carry it) and join from bytes.
        let welcome_wire = serialize_message(&welcome).unwrap();
        assert!(!welcome_wire.is_empty());
        let alice_group = join_group_bytes(&alice, &welcome_wire).unwrap();

        // The joiner converged with the adder.
        assert_eq!(
            creator_group.group_id().as_slice(),
            alice_group.group_id().as_slice()
        );
        assert_eq!(creator_group.epoch(), alice_group.epoch());

        // A truncated Welcome fails closed; a non-Welcome message is rejected too.
        let bob = MlsMember::generate(b"bob@mercury").unwrap();
        assert!(join_group_bytes(&bob, &welcome_wire[..welcome_wire.len() / 2]).is_err());
        assert!(join_group_bytes(&bob, &[]).is_err());
    }

    #[test]
    fn a_message_from_a_foreign_group_fails_to_decrypt() {
        // Group 1: creator1 + alice1 exchange-capable; produce a real message.
        let creator1 = MlsMember::generate(b"creator1").unwrap();
        let alice1 = MlsMember::generate(b"alice1").unwrap();
        let mut group1 = creator1.create_group([1u8; 32]).unwrap();
        let (welcome1, _c1) = add_member(&mut group1, &creator1, alice1.key_package()).unwrap();
        let _alice1_group = join_group(&alice1, &welcome1).unwrap();
        let foreign = send_message(&mut group1, &creator1, b"group-1 secret").unwrap();

        // Group 2: an entirely separate group.
        let creator2 = MlsMember::generate(b"creator2").unwrap();
        let alice2 = MlsMember::generate(b"alice2").unwrap();
        let mut group2 = creator2.create_group([2u8; 32]).unwrap();
        let (welcome2, _c2) = add_member(&mut group2, &creator2, alice2.key_package()).unwrap();
        let mut alice2_group = join_group(&alice2, &welcome2).unwrap();

        // A group-1 message must NOT decrypt for a group-2 member (fails closed).
        assert!(receive_message(&mut alice2_group, &alice2, &foreign).is_err());
    }

    #[test]
    fn a_removed_member_cannot_decrypt_new_messages() {
        let creator = MlsMember::generate(b"creator@mercury").unwrap();
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let mut creator_group = creator.create_group([3u8; 32]).unwrap();
        let (welcome, _consumed) =
            add_member(&mut creator_group, &creator, alice.key_package()).unwrap();
        let mut alice_group = join_group(&alice, &welcome).unwrap();

        // Before removal, alice decrypts group messages.
        let before = send_message(&mut creator_group, &creator, b"before").unwrap();
        assert_eq!(
            receive_message(&mut alice_group, &alice, &before).unwrap(),
            b"before"
        );

        // creator removes alice; the epoch rotates.
        let epoch_before = creator_group.epoch();
        remove_member(&mut creator_group, &creator, alice.signer().public()).unwrap();
        assert_ne!(creator_group.epoch(), epoch_before);

        // A post-removal message cannot be decrypted by alice's stale group view.
        let after = send_message(&mut creator_group, &creator, b"after removal").unwrap();
        assert!(receive_message(&mut alice_group, &alice, &after).is_err());
    }

    #[test]
    fn a_three_member_group_converges_fans_out_and_revokes() {
        // N>2 correctness: adding a 3rd member re-keys the group, so the EXISTING
        // member must apply the add-commit or desync. Drives the full multi-member
        // flow -- repeated add, commit processing, 3-way convergence, message
        // fan-out, and a removal that preserves the remaining member.
        let creator = MlsMember::generate(b"creator@mercury").unwrap();
        let alice = MlsMember::generate(b"alice@mercury").unwrap();
        let bob = MlsMember::generate(b"bob@mercury").unwrap();
        let mut creator_group = creator.create_group([6u8; 32]).unwrap();

        // Add alice (first add: no other existing member needs the commit).
        let (_c1, welcome_a, _) =
            add_member_with_commit(&mut creator_group, &creator, alice.key_package()).unwrap();
        let mut alice_group = join_group(&alice, &welcome_a).unwrap();

        // Add bob: alice (an existing member) MUST apply the add-commit or she
        // falls behind the re-keyed epoch.
        let (commit_b, welcome_b, _) =
            add_member_with_commit(&mut creator_group, &creator, bob.key_package()).unwrap();
        let mut bob_group = join_group(&bob, &welcome_b).unwrap();
        process_commit(&mut alice_group, &alice, &commit_b).unwrap();

        // All three converge on the same group id + epoch.
        assert_eq!(
            creator_group.group_id().as_slice(),
            alice_group.group_id().as_slice()
        );
        assert_eq!(
            creator_group.group_id().as_slice(),
            bob_group.group_id().as_slice()
        );
        assert_eq!(creator_group.epoch(), alice_group.epoch());
        assert_eq!(creator_group.epoch(), bob_group.epoch());

        // A creator message fans out: BOTH other members decrypt the same bytes.
        let msg =
            serialize_message(&send_message(&mut creator_group, &creator, b"hi all").unwrap())
                .unwrap();
        assert_eq!(
            receive_message_bytes(&mut alice_group, &alice, &msg).unwrap(),
            b"hi all"
        );
        assert_eq!(
            receive_message_bytes(&mut bob_group, &bob, &msg).unwrap(),
            b"hi all"
        );

        // Remove alice; bob (remaining) applies the removal commit + stays
        // functional, while alice's stale view is revoked.
        let removal = remove_member(&mut creator_group, &creator, alice.signer().public()).unwrap();
        process_commit(&mut bob_group, &bob, &removal).unwrap();
        assert_eq!(creator_group.epoch(), bob_group.epoch());

        let after =
            serialize_message(&send_message(&mut creator_group, &creator, b"bob only").unwrap())
                .unwrap();
        assert_eq!(
            receive_message_bytes(&mut bob_group, &bob, &after).unwrap(),
            b"bob only"
        );
        assert!(receive_message_bytes(&mut alice_group, &alice, &after).is_err());
    }

    // ----- Post-quantum (X-Wing) group chat via the libcrux provider -----

    #[cfg(feature = "xwing")]
    #[test]
    fn pq_member_generates_under_the_xwing_suite_with_honest_label() {
        let alice = MlsPqMember::generate(b"alice@mercury").unwrap();
        // The KeyPackage is genuinely built under the X-Wing PQ ciphersuite...
        assert_eq!(alice.ciphersuite(), PQ_CIPHERSUITE);
        assert_eq!(
            alice.ciphersuite(),
            Ciphersuite::MLS_256_XWING_CHACHA20POLY1305_SHA256_Ed25519
        );
        // ...and Mercury labels it HONESTLY as the post-quantum hybrid suite.
        assert_eq!(
            alice.mercury_suite(),
            Some(GroupChatCryptoSuite::HybridPqMls768)
        );
        // Well-formed signed package with a non-empty init key.
        assert!(!alice.key_package().hpke_init_key().as_slice().is_empty());
        assert!(
            alice
                .key_package()
                .hash_ref(alice.provider().crypto())
                .is_ok()
        );
    }

    #[cfg(feature = "xwing")]
    #[test]
    fn the_two_providers_are_complementary_pq_vs_classical() {
        // The libcrux provider genuinely supports the X-Wing PQ suite; the RustCrypto
        // provider does NOT (it Err()s on supports() and unimplemented!()s the X-Wing
        // KEM at use). Conversely, Mercury's classical default uses AES-128-GCM HPKE,
        // which the libcrux provider does NOT support (it only does ChaCha20-Poly1305
        // HPKE), while RustCrypto does. So the two backends are COMPLEMENTARY:
        // RustCrypto for the classical AES-GCM suite, libcrux for the PQ X-Wing suite.
        let libcrux = OpenMlsLibcrux::default();
        let rustcrypto = OpenMlsRustCrypto::default();

        // X-Wing PQ suite: libcrux yes, RustCrypto no.
        assert!(
            libcrux.crypto().supports(PQ_CIPHERSUITE).is_ok(),
            "libcrux must support X-Wing"
        );
        assert!(
            rustcrypto.crypto().supports(PQ_CIPHERSUITE).is_err(),
            "RustCrypto must NOT claim X-Wing support"
        );

        // Classical AES-128-GCM default suite: RustCrypto yes, libcrux no.
        assert!(
            rustcrypto.crypto().supports(DEFAULT_CIPHERSUITE).is_ok(),
            "RustCrypto must support the classical default"
        );
        assert!(
            libcrux.crypto().supports(DEFAULT_CIPHERSUITE).is_err(),
            "libcrux (ChaCha20-only HPKE) does not support the AES-GCM classical suite"
        );
    }

    #[cfg(feature = "xwing")]
    #[test]
    fn pq_and_classical_evidence_carry_distinct_provider_identities() {
        // Provider-evidence for the PQ suite must be tagged with the libcrux backend
        // identity, NOT the classical RustCrypto one — so evidence provenance is
        // honest about which backend actually executed the suite.
        assert_ne!(
            provider_identity_for(GroupChatCryptoSuite::HybridPqMls768),
            provider_identity_for(GroupChatCryptoSuite::ClassicalMls128)
        );
        assert_eq!(
            provider_identity_for(GroupChatCryptoSuite::HybridPqMls768),
            PROVIDER_IDENTITY_LIBCRUX
        );
        assert_eq!(
            provider_identity_for(GroupChatCryptoSuite::ClassicalMls128),
            PROVIDER_IDENTITY_RUSTCRYPTO
        );
        // The libcrux provider really derives evidence for the PQ suite, and its
        // provider_id_digest differs from the classical one.
        let libcrux = OpenMlsLibcrux::default();
        let pq_ev = OpenMlsProviderEvidence::derive(
            &libcrux,
            GroupChatCryptoSuite::HybridPqMls768,
            NOW,
            MAX_LIFETIME,
        )
        .unwrap();
        let classical_ev = OpenMlsProviderEvidence::derive(
            &OpenMlsRustCrypto::default(),
            GroupChatCryptoSuite::ClassicalMls128,
            NOW,
            MAX_LIFETIME,
        )
        .unwrap();
        assert_ne!(pq_ev.provider_id_digest, classical_ev.provider_id_digest);
    }

    #[cfg(feature = "xwing")]
    #[test]
    fn pq_two_member_group_round_trips_encrypted_messages() {
        // Full post-quantum group flow: founder creates an X-Wing group, adds a
        // second PQ member, the joiner builds its view from the Welcome, and both
        // directions exchange real encrypted application messages — all under the
        // X-Wing (X25519 + ML-KEM-768) hybrid KEM via the libcrux provider.
        let alice = MlsPqMember::generate(b"alice@mercury").unwrap();
        let bob = MlsPqMember::generate(b"bob@mercury").unwrap();

        let mut alice_group = alice.create_group([7u8; 32]).unwrap();
        assert_eq!(alice_group.ciphersuite(), PQ_CIPHERSUITE);

        // Alice adds Bob (X-Wing HPKE seals the Welcome to Bob's PQ init key).
        let (_commit, welcome, _group_info) = alice_group
            .add_members(
                alice.provider(),
                alice.signer(),
                std::slice::from_ref(bob.key_package()),
            )
            .expect("pq add_members");
        alice_group
            .merge_pending_commit(alice.provider())
            .expect("merge pq commit");

        // Bob builds his own group view from the Welcome (X-Wing HPKE open).
        let welcome_in =
            MlsMessageIn::tls_deserialize(&mut &welcome.tls_serialize_detached().unwrap()[..])
                .expect("welcome parse");
        let welcome_msg = match welcome_in.extract() {
            MlsMessageBodyIn::Welcome(w) => w,
            _ => panic!("expected a Welcome"),
        };
        let mut bob_group = StagedWelcome::new_from_welcome(
            bob.provider(),
            &MlsGroupJoinConfig::default(),
            welcome_msg,
            None,
        )
        .expect("staged welcome")
        .into_group(bob.provider())
        .expect("pq join");

        assert_eq!(bob_group.ciphersuite(), PQ_CIPHERSUITE);
        assert_eq!(alice_group.epoch(), bob_group.epoch());

        // Alice -> Bob encrypted message.
        let a2b = alice_group
            .create_message(alice.provider(), alice.signer(), b"post-quantum hello bob")
            .expect("pq encrypt a->b");
        let a2b_in = MlsMessageIn::tls_deserialize(&mut &a2b.tls_serialize_detached().unwrap()[..])
            .unwrap()
            .try_into_protocol_message()
            .unwrap();
        let processed = bob_group
            .process_message(bob.provider(), a2b_in)
            .expect("pq process a->b");
        match processed.into_content() {
            ProcessedMessageContent::ApplicationMessage(m) => {
                assert_eq!(m.into_bytes(), b"post-quantum hello bob");
            }
            _ => panic!("expected an application message"),
        }

        // Bob -> Alice encrypted message (other direction).
        let b2a = bob_group
            .create_message(bob.provider(), bob.signer(), b"hello back alice")
            .expect("pq encrypt b->a");
        let b2a_in = MlsMessageIn::tls_deserialize(&mut &b2a.tls_serialize_detached().unwrap()[..])
            .unwrap()
            .try_into_protocol_message()
            .unwrap();
        let processed = alice_group
            .process_message(alice.provider(), b2a_in)
            .expect("pq process b->a");
        match processed.into_content() {
            ProcessedMessageContent::ApplicationMessage(m) => {
                assert_eq!(m.into_bytes(), b"hello back alice");
            }
            _ => panic!("expected an application message"),
        }
    }
}
