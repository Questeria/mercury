//! The append-only audit EVENT LOG + the producer for mercury-core's sealed-audit
//! event-chain gate.
//!
//! Each appended event is sealed (by the caller) and committed three ways: a
//! hash CHAIN (`event_hash[i] = SHA-256(prev || record_digest || sequence ||
//! kind)`), an RFC 6962 MERKLE leaf (so any event has an inclusion proof and any
//! two log sizes have a consistency proof — both really verified here), and an
//! Ed25519 SIGNED CHECKPOINT over the Merkle root. The engine assembles a
//! [`SealedAuditEventChainInput`] for `evaluate_sealed_audit_event_chain` using
//! those real artifacts; storage/forward-secrecy facts the engine cannot observe
//! (the durable store is append-only / transactional / rollback-resistant / sealed
//! at rest; the prior epoch key was rotated + wiped) are an explicit caller
//! [`StorageAttestation`], never fabricated. This first increment targets the
//! `LocalHashChain` anchor (no witness quorum / transparency receipt); the
//! witnessed anchor is a later increment.

use sha2::{Digest as _, Sha256};

use ed25519_dalek::SigningKey;
use mercury_core::{SealedAuditAnchorKind, SealedAuditEnvelopeSuite, SealedAuditEventKind};

use crate::checkpoint::{SealedAuditCheckpoint, sign_checkpoint, verify_checkpoint};
use crate::hybrid::{
    HYBRID_CHECKPOINT_SIG_LEN, HybridCheckpointSigningKey, HybridCheckpointVerifyingKey,
    sign_checkpoint_hybrid,
};
use crate::hybrid_witness::{
    HybridWitness, HybridWitnessCosignature, verify_hybrid_witness_quorum,
};
use crate::merkle::{
    consistency_proof, inclusion_proof, leaf_hash, merkle_root, verify_consistency,
    verify_inclusion,
};
use crate::witness::{
    AuditWitness, WitnessCosignature, verified_cosignatures, verify_witnessed_checkpoint,
};

pub use mercury_core::{
    SealedAuditCheckpointSignatureAlgorithm, SealedAuditEventChainDecision,
    SealedAuditEventChainInput, SealedAuditEventStoreDecision, SealedAuditEventStoreReason,
    SealedAuditEventStoreWrite, SealedAuditProofBundleDecision, SealedAuditProofBundleInput,
    SealedAuditProofBundleReason, SealedAuditWitnessCheckpointDecision,
    SealedAuditWitnessCheckpointInput, SealedAuditWitnessCheckpointReason,
    SealedAuditWitnessClientDecision, SealedAuditWitnessClientInput,
    SealedAuditWitnessClientReason, evaluate_sealed_audit_event_chain,
    evaluate_sealed_audit_event_store_write, evaluate_sealed_audit_proof_bundle,
    evaluate_sealed_audit_witness_checkpoint, evaluate_sealed_audit_witness_client,
};

const RECORD_DOMAIN: &[u8] = b"mercury/audit/record/v1";
const EVENT_DOMAIN: &[u8] = b"mercury/audit/event/v1";
const DEVICE_DOMAIN: &[u8] = b"mercury/audit/bind/device/v1";
const ACTOR_DOMAIN: &[u8] = b"mercury/audit/bind/actor/v1";
const EPOCH_DOMAIN: &[u8] = b"mercury/audit/bind/epoch/v1";
const ROOM_EPOCH_DOMAIN: &[u8] = b"mercury/audit/bind/room-epoch/v1";
/// Domain for the 32-byte checkpoint id committing a signed checkpoint.
const CHECKPOINT_ID_DOMAIN: &[u8] = b"mercury/audit/checkpoint-id/v1";
/// Genesis link for the event hash chain (previous-hash of the first event).
const GENESIS: [u8; 32] = [0u8; 32];
const DIGEST_LEN: i32 = 32;

fn digest(domain: &[u8], data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(data);
    hasher.finalize().into()
}

/// One audit event to append: the (already AEAD-sealed) `sealed_record`, the
/// event `kind`, and the identity key material the event binds (device / actor /
/// epoch, and the room epoch for MLS/backup events). The engine digests the
/// binding material; the caller seals the record with those digests in its AEAD
/// associated data (attested via [`StorageAttestation::aad_binds_event_context`]).
#[derive(Debug, Clone, Copy)]
pub struct AuditEvent<'a> {
    pub sealed_record: &'a [u8],
    pub kind: SealedAuditEventKind,
    pub device_binding: &'a [u8],
    pub actor_binding: &'a [u8],
    pub epoch_binding: &'a [u8],
    pub room_epoch_binding: &'a [u8],
}

/// Deployment / storage facts the engine cannot observe from the event bytes —
/// supplied by the caller (the durable-store + sealing layer) and NEVER
/// fabricated by the engine. A `false` makes the gate honestly refuse the append.
#[derive(Debug, Clone, Copy)]
pub struct StorageAttestation {
    pub event_sealed: bool,
    pub aad_binds_event_context: bool,
    pub critical_event_bound: bool,
    pub storage_append_only: bool,
    pub storage_transactional: bool,
    pub rollback_resistant_store: bool,
    pub local_store_sealed: bool,
    pub forward_secret_rotated: bool,
    pub previous_key_material_deleted: bool,
}

impl StorageAttestation {
    /// A standard fully-attested local sealed store: the event is sealed with
    /// context-binding AAD, and the durable store is append-only, transactional,
    /// rollback-resistant, and sealed at rest with prior epoch key material
    /// rotated + wiped. Use only when these genuinely hold.
    pub const fn local_sealed_store() -> Self {
        Self {
            event_sealed: true,
            aad_binds_event_context: true,
            critical_event_bound: true,
            storage_append_only: true,
            storage_transactional: true,
            rollback_resistant_store: true,
            local_store_sealed: true,
            forward_secret_rotated: true,
            previous_key_material_deleted: true,
        }
    }
}

/// An append-only audit event log: an Ed25519-keyed, hash-chained, RFC 6962
/// Merkle-backed transparency log. Each [`AuditEventLog::append`] returns the
/// gate input for that append.
pub struct AuditEventLog {
    signing_key: SigningKey,
    /// Optional hybrid (Ed25519 + ML-DSA-44) checkpoint key. When present, every
    /// staged checkpoint is ALSO hybrid-signed, enabling the post-quantum witnessed
    /// transparency path; when `None`, the log behaves exactly as the Ed25519-only
    /// `new` constructor (the LocalHashChain path is byte-identical).
    hybrid_key: Option<HybridCheckpointSigningKey>,
    leaves: Vec<[u8; 32]>,
    event_hashes: Vec<[u8; 32]>,
    checkpoint_size: u64,
}

impl AuditEventLog {
    /// Create a log keyed by `signing_key` (the log's Ed25519 checkpoint key,
    /// pinned out of band by verifiers). No hybrid key: the LocalHashChain path.
    pub fn new(signing_key: SigningKey) -> Self {
        Self {
            signing_key,
            hybrid_key: None,
            leaves: Vec::new(),
            event_hashes: Vec::new(),
            checkpoint_size: 0,
        }
    }

    /// Create a log with BOTH an Ed25519 checkpoint key and a hybrid
    /// (Ed25519 + ML-DSA-44) checkpoint key. The hybrid signature is what the
    /// sealed-audit witness gate requires (a post-quantum / hybrid algorithm); the
    /// Ed25519 key still drives the LocalHashChain chain + store artifacts unchanged.
    pub fn new_with_hybrid(
        signing_key: SigningKey,
        hybrid_key: HybridCheckpointSigningKey,
    ) -> Self {
        Self {
            signing_key,
            hybrid_key: Some(hybrid_key),
            leaves: Vec::new(),
            event_hashes: Vec::new(),
            checkpoint_size: 0,
        }
    }

    /// The log's Ed25519 checkpoint verifying key (pin this out of band).
    pub fn verifying_key(&self) -> ed25519_dalek::VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// The log's hybrid checkpoint verifying key, if it has a hybrid key (pin this
    /// out of band alongside the Ed25519 key for the witnessed transparency path).
    pub fn hybrid_verifying_key(&self) -> Option<HybridCheckpointVerifyingKey> {
        self.hybrid_key.as_ref().map(|k| k.verifying_key())
    }

    /// The current Merkle root committing every appended event.
    pub fn merkle_root(&self) -> [u8; 32] {
        merkle_root(&self.leaves)
    }

    /// The current number of events in the log.
    pub fn len(&self) -> usize {
        self.event_hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.event_hashes.is_empty()
    }

    /// The Merkle root over the first `size` events (an earlier checkpoint's root),
    /// or `None` if `size` exceeds the log. A verifier uses this to obtain the
    /// previously-trusted checkpoint root a proof bundle is consistent against.
    pub fn merkle_root_at(&self, size: usize) -> Option<[u8; 32]> {
        if size > self.leaves.len() {
            return None;
        }
        Some(merkle_root(&self.leaves[..size]))
    }

    /// Append `event` for the default `LocalHashChain` anchor and produce its gate
    /// input. Convenience wrapper over [`AuditEventLog::stage`] +
    /// [`StagedAppend::finalize_local`].
    pub fn append(
        &mut self,
        event: &AuditEvent<'_>,
        attestation: &StorageAttestation,
        timestamp_s: i64,
    ) -> SealedAuditEventChainInput {
        self.stage(event, attestation, timestamp_s).finalize_local()
    }

    /// Append `event` to the log and STAGE its gate input without yet committing to
    /// an anchor. The hash chain, Merkle root + inclusion + consistency proofs, and
    /// the Ed25519 checkpoint are all computed and the proofs really verified here
    /// (fail-closed if a proof did not verify). The returned [`StagedAppend`] exposes
    /// the freshly signed [`SealedAuditCheckpoint`] so the caller can gather witness
    /// cosignatures over it, then finalize as either a `LocalHashChain`
    /// ([`StagedAppend::finalize_local`]) or `WitnessedTransparencyLog`
    /// ([`StagedAppend::finalize_witnessed`]) anchor. `attestation` supplies the
    /// storage/forward-secrecy facts; `timestamp_s` stamps the checkpoint (> 0).
    pub fn stage(
        &mut self,
        event: &AuditEvent<'_>,
        attestation: &StorageAttestation,
        timestamp_s: i64,
    ) -> StagedAppend {
        let sequence = self.event_hashes.len();
        let previous_event_hash_len = if sequence > 0 { DIGEST_LEN } else { 0 };
        let previous_hash = self.event_hashes.last().copied().unwrap_or(GENESIS);

        // Record digest + identity-binding digests; the event hash COMMITS them
        // all, so each event is tamper-evidently bound to exactly which device,
        // actor, and epoch produced it (the binding digests are real, not just a
        // length the gate checks).
        let record_digest = digest(RECORD_DOMAIN, event.sealed_record);
        let device_digest = digest(DEVICE_DOMAIN, event.device_binding);
        let actor_digest = digest(ACTOR_DOMAIN, event.actor_binding);
        let epoch_digest = digest(EPOCH_DOMAIN, event.epoch_binding);
        let room_epoch_digest = digest(ROOM_EPOCH_DOMAIN, event.room_epoch_binding);
        let event_hash = {
            let mut hasher = Sha256::new();
            hasher.update(EVENT_DOMAIN);
            hasher.update(previous_hash);
            hasher.update(record_digest);
            hasher.update((sequence as u64).to_le_bytes());
            hasher.update(event.kind.code().to_le_bytes());
            hasher.update(device_digest);
            hasher.update(actor_digest);
            hasher.update(epoch_digest);
            hasher.update(room_epoch_digest);
            <[u8; 32]>::from(hasher.finalize())
        };

        // Append the Merkle leaf over the event hash + advance the chain.
        let leaf = leaf_hash(&event_hash);
        self.leaves.push(leaf);
        self.event_hashes.push(event_hash);
        let new_size = self.leaves.len();
        let root = merkle_root(&self.leaves);

        // REAL inclusion proof for the new event, verified inline.
        let inclusion = inclusion_proof(&self.leaves, sequence);
        let inclusion_verified = verify_inclusion(&leaf, sequence, new_size, &inclusion, &root);

        // REAL consistency proof: the prior log is an append-only prefix of the new
        // one. The first event is trivially consistent with the empty prior log. The
        // proof's hash count is captured (the real RFC 6962 audit-path length) for the
        // witness-checkpoint gate, which bounds it to 0..=63.
        let (consistency_verified, consistency_proof_hash_count) = if sequence == 0 {
            (true, 0i32)
        } else {
            let previous_root = merkle_root(&self.leaves[..sequence]);
            let proof = consistency_proof(&self.leaves, sequence);
            let count = i32::try_from(proof.len()).unwrap_or(i32::MAX);
            let verified = verify_consistency(sequence, new_size, &proof, &previous_root, &root);
            (verified, count)
        };

        // Ed25519 signed checkpoint over the new Merkle root.
        let checkpoint = SealedAuditCheckpoint {
            tree_size: new_size as u64,
            root_hash: root,
            timestamp_s,
        };
        let checkpoint_signature = sign_checkpoint(&self.signing_key, &checkpoint);
        // If the log has a hybrid key, ALSO hybrid-sign the same checkpoint (the
        // post-quantum half of the witnessed transparency path). Both halves sign
        // the identical canonical preimage as the Ed25519 checkpoint signature.
        let hybrid_checkpoint_signature = self
            .hybrid_key
            .as_ref()
            .map(|k| sign_checkpoint_hybrid(k, &checkpoint));
        let hybrid_verifying_key = self.hybrid_key.as_ref().map(|k| k.verifying_key());
        let previous_checkpoint_size = i64::try_from(self.checkpoint_size).unwrap_or(i64::MAX);
        self.checkpoint_size = new_size as u64;

        let input = SealedAuditEventChainInput {
            event_kind: event.kind,
            anchor_kind: SealedAuditAnchorKind::LocalHashChain,
            envelope_suite: SealedAuditEnvelopeSuite::XChaCha20Poly1305Blake3,
            event_sequence: i64::try_from(sequence).unwrap_or(i64::MAX),
            previous_chain_size: i64::try_from(sequence).unwrap_or(i64::MAX),
            previous_event_hash_len,
            event_hash_len: DIGEST_LEN,
            record_digest_len: DIGEST_LEN,
            merkle_leaf_hash_len: DIGEST_LEN,
            merkle_root_hash_len: DIGEST_LEN,
            event_sealed: attestation.event_sealed,
            aad_binds_event_context: attestation.aad_binds_event_context,
            plaintext_field_count: 0,
            plaintext_payload_bytes: 0,
            monotonic_counter_present: true,
            monotonic_counter_increases: true,
            device_binding_digest_len: DIGEST_LEN,
            actor_binding_digest_len: DIGEST_LEN,
            epoch_binding_digest_len: DIGEST_LEN,
            room_epoch_digest_len: DIGEST_LEN,
            critical_event_bound: attestation.critical_event_bound,
            signed_checkpoint_present: true,
            checkpoint_signature_len: i32::try_from(checkpoint_signature.len()).unwrap_or(i32::MAX),
            checkpoint_timestamp_s: timestamp_s,
            checkpoint_size: i64::try_from(new_size).unwrap_or(i64::MAX),
            previous_checkpoint_size,
            inclusion_proof_verified: inclusion_verified,
            consistency_proof_verified: consistency_verified,
            // LocalHashChain anchor: no transparency receipt / witness quorum.
            transparency_receipt_present: false,
            witness_count: 0,
            witness_threshold: 0,
            witness_operator_count: 0,
            storage_append_only: attestation.storage_append_only,
            storage_transactional: attestation.storage_transactional,
            rollback_resistant_store: attestation.rollback_resistant_store,
            local_store_sealed: attestation.local_store_sealed,
            forward_secret_rotated: attestation.forward_secret_rotated,
            previous_key_material_deleted: attestation.previous_key_material_deleted,
        };
        StagedAppend {
            input,
            checkpoint,
            verifying_key: self.signing_key.verifying_key(),
            hybrid_checkpoint_signature,
            hybrid_verifying_key,
            consistency_proof_hash_count,
            event_hash,
            previous_event_hash: if sequence > 0 {
                Some(previous_hash)
            } else {
                None
            },
            record_digest,
            merkle_root: root,
            checkpoint_signature,
            sealed_payload_len: i32::try_from(event.sealed_record.len()).unwrap_or(i32::MAX),
        }
    }

    /// Produce the gate input for the OFFLINE-VERIFIER proof-bundle gate
    /// ([`evaluate_sealed_audit_proof_bundle`]): a verifier proves that the event at
    /// `log_index` is included in the witnessed checkpoint of size `checkpoint_size`
    /// (root `checkpoint_root`), AND that that checkpoint is append-only consistent
    /// with a previously-trusted checkpoint (`previous_checkpoint_size` /
    /// `previous_checkpoint_root`). Returns `None` if the indices are out of range
    /// (`log_index >= checkpoint_size`, `checkpoint_size > len`, or
    /// `previous_checkpoint_size` not in `1..=checkpoint_size`).
    ///
    /// The proofs are REAL: the inclusion proof is recomputed over this log's leaves
    /// and `inclusion_proof_verified` / `inclusion_root_matches_checkpoint` are the
    /// genuine verification results against `checkpoint_root` (a tampered root or a
    /// wrong log_index fails closed); the consistency proof between the two sizes is
    /// recomputed and verified. `witness_client_decision` is the REAL upstream
    /// decision (the gate binds `checkpoint_size`, `verifier_policy_epoch`, and
    /// `verifier_witness_threshold` to it). Verifier-policy / proof-cache / witness
    /// freshness facts the engine cannot observe are the explicit caller
    /// `attestation`, never fabricated.
    #[allow(clippy::too_many_arguments)]
    pub fn proof_bundle_input(
        &self,
        witness_client_decision: SealedAuditWitnessClientDecision,
        checkpoint_root: &[u8; 32],
        checkpoint_size: usize,
        previous_checkpoint_size: usize,
        previous_checkpoint_root: &[u8; 32],
        log_index: usize,
        audit_subject_digest: &[u8; 32],
        attestation: &ProofBundleAttestation,
    ) -> Option<SealedAuditProofBundleInput> {
        // Bounds: the proven event and both checkpoint sizes must be within this log.
        if checkpoint_size > self.leaves.len()
            || log_index >= checkpoint_size
            || previous_checkpoint_size < 1
            || previous_checkpoint_size > checkpoint_size
        {
            return None;
        }
        let leaves = &self.leaves[..checkpoint_size];

        // REAL inclusion proof for the event at log_index, verified against the
        // witnessed checkpoint root.
        let inclusion = inclusion_proof(leaves, log_index);
        let inclusion_proof_verified = verify_inclusion(
            &self.leaves[log_index],
            log_index,
            checkpoint_size,
            &inclusion,
            checkpoint_root,
        );
        // REAL check that the checkpoint root commits exactly this log's first
        // `checkpoint_size` leaves (recomputed, not asserted).
        let inclusion_root_matches_checkpoint = merkle_root(leaves) == *checkpoint_root;

        // REAL consistency proof: the witnessed checkpoint is an append-only superset
        // of the previously-trusted checkpoint.
        let consistency = consistency_proof(leaves, previous_checkpoint_size);
        let consistency_proof_hash_count = i32::try_from(consistency.len()).unwrap_or(i32::MAX);
        let consistency_proof_verified = verify_consistency(
            previous_checkpoint_size,
            checkpoint_size,
            &consistency,
            previous_checkpoint_root,
            checkpoint_root,
        );

        Some(SealedAuditProofBundleInput {
            witness_client_decision,
            bundle_format_version: attestation.bundle_format_version,
            proof_bundle_persisted: attestation.proof_bundle_persisted,
            proof_cache_digest_len: attestation.proof_cache_digest_len,
            proof_cache_encrypted: attestation.proof_cache_encrypted,
            proof_cache_append_only: attestation.proof_cache_append_only,
            local_proof_cache_available: attestation.local_proof_cache_available,
            proof_cache_recovery_authenticated: attestation.proof_cache_recovery_authenticated,
            proof_cache_recovery_user_verified: attestation.proof_cache_recovery_user_verified,
            verifier_policy_snapshot_digest_len: attestation.verifier_policy_snapshot_digest_len,
            // Bound to the REAL upstream witness-client decision.
            verifier_policy_epoch: witness_client_decision.policy_epoch,
            verifier_policy_matches_witness_policy: attestation
                .verifier_policy_matches_witness_policy,
            verifier_log_key_pin_count: attestation.verifier_log_key_pin_count,
            verifier_witness_key_pin_count: attestation.verifier_witness_key_pin_count,
            verifier_witness_threshold: witness_client_decision.witness_quorum_threshold,
            verified_witness_cosignature_count: attestation.verified_witness_cosignature_count,
            event_sequence: i64::try_from(log_index).unwrap_or(i64::MAX),
            event_hash_len: DIGEST_LEN,
            leaf_hash_len: DIGEST_LEN,
            log_index: i64::try_from(log_index).unwrap_or(i64::MAX),
            checkpoint_size: i64::try_from(checkpoint_size).unwrap_or(i64::MAX),
            inclusion_proof_hash_count: i32::try_from(inclusion.len()).unwrap_or(i32::MAX),
            inclusion_proof_verified,
            inclusion_root_matches_checkpoint,
            consistency_proof_hash_count,
            consistency_proof_verified,
            witness_timestamp_s: attestation.witness_timestamp_s,
            verification_time_s: attestation.verification_time_s,
            max_witness_age_s: attestation.max_witness_age_s,
            monitor_freshness_checked: attestation.monitor_freshness_checked,
            extra_data_authenticated_or_opaque: attestation.extra_data_authenticated_or_opaque,
            audit_subject_digest_len: i32::try_from(audit_subject_digest.len()).unwrap_or(i32::MAX),
            plaintext_selector_count: 0,
            ui_status_digest_only: attestation.ui_status_digest_only,
        })
    }
}

/// A staged append: the log has been advanced and its checkpoint signed, but the
/// gate input is not yet bound to an anchor. The caller takes [`StagedAppend::checkpoint`]
/// to gather witness cosignatures (for a witnessed anchor), then finalizes.
///
/// Finalizing does not re-touch the log — the leaf is already appended — so the
/// LocalHashChain and WitnessedTransparencyLog paths differ only in the anchor /
/// witness / transparency-receipt fields, never in the committed Merkle history.
pub struct StagedAppend {
    input: SealedAuditEventChainInput,
    checkpoint: SealedAuditCheckpoint,
    verifying_key: ed25519_dalek::VerifyingKey,
    hybrid_checkpoint_signature: Option<[u8; HYBRID_CHECKPOINT_SIG_LEN]>,
    hybrid_verifying_key: Option<HybridCheckpointVerifyingKey>,
    consistency_proof_hash_count: i32,
    event_hash: [u8; 32],
    previous_event_hash: Option<[u8; 32]>,
    record_digest: [u8; 32],
    merkle_root: [u8; 32],
    checkpoint_signature: [u8; 64],
    sealed_payload_len: i32,
}

impl StagedAppend {
    /// The freshly signed checkpoint over the new Merkle root. A witnessed anchor
    /// collects independent-operator Ed25519 cosignatures over `checkpoint.signing_bytes()`.
    pub fn checkpoint(&self) -> &SealedAuditCheckpoint {
        &self.checkpoint
    }

    /// The hybrid (Ed25519 + ML-DSA-44) checkpoint signature over this checkpoint,
    /// present only when the log was created with [`AuditEventLog::new_with_hybrid`].
    /// This 2484-byte signature is the post-quantum head receipt the witnessed
    /// transparency path publishes.
    pub fn hybrid_checkpoint_signature(&self) -> Option<[u8; HYBRID_CHECKPOINT_SIG_LEN]> {
        self.hybrid_checkpoint_signature
    }

    /// The hybrid verifying key for this staged checkpoint, if the log has one (pin
    /// out of band; a witness cosigns the checkpoint this key signed).
    pub fn hybrid_verifying_key(&self) -> Option<&HybridCheckpointVerifyingKey> {
        self.hybrid_verifying_key.as_ref()
    }

    /// Finalize as a `LocalHashChain` anchor (no witness quorum / transparency
    /// receipt) — the gate input exactly as [`AuditEventLog::append`] returns.
    pub fn finalize_local(self) -> SealedAuditEventChainInput {
        self.input
    }

    /// Finalize as a `WitnessedTransparencyLog` anchor. `cosignatures` are verified
    /// against `witnesses` over THIS checkpoint via [`verify_witnessed_checkpoint`];
    /// the gate input's `witness_count` / `witness_operator_count` are set from the
    /// REAL verified quorum (forged or duplicate cosignatures cannot inflate them),
    /// and `witness_threshold` is the deployment's required quorum (the gate needs
    /// `>= 2`). The transparency receipt is set from real artifacts — the signed
    /// checkpoint plus the verified inclusion proof IS an authenticated inclusion
    /// receipt — never fabricated. If the quorum or proofs fall short, the gate
    /// honestly rejects.
    pub fn finalize_witnessed(
        mut self,
        witnesses: &[AuditWitness],
        cosignatures: &[WitnessCosignature],
        witness_threshold: i32,
    ) -> SealedAuditEventChainInput {
        let quorum = verify_witnessed_checkpoint(&self.checkpoint, witnesses, cosignatures);
        self.input.anchor_kind = SealedAuditAnchorKind::WitnessedTransparencyLog;
        self.input.witness_count = quorum.verified_witness_count;
        self.input.witness_operator_count = quorum.operator_count;
        self.input.witness_threshold = witness_threshold;
        self.input.transparency_receipt_present =
            self.input.signed_checkpoint_present && self.input.inclusion_proof_verified;
        self.input
    }

    /// Produce the durable-store WRITE for a `LocalHashChain` anchor from the same
    /// real artifacts as the chain gate input. The event hash, previous-event hash,
    /// record digest, Merkle root, and Ed25519 checkpoint signature are the genuine
    /// ones computed at [`AuditEventLog::stage`]; the 32-byte `checkpoint_id` is a
    /// SHA-256 commitment to the signed checkpoint, so it really binds this write to
    /// the signed head (`checkpoint_binds_chain` is a verified equality, not a
    /// hard-coded claim). `append_only_guard` is the durable store's append-only
    /// guarantee for this write — a deployment fact the caller attests, never
    /// fabricated. The chain decision is computed here from the real chain input, so
    /// if the chain gate rejected, the store gate honestly rejects too.
    pub fn local_store_write(&self, append_only_guard: bool) -> SealedAuditStoreWrite {
        let chain_decision = evaluate_sealed_audit_event_chain(self.input);
        let checkpoint_id = digest(CHECKPOINT_ID_DOMAIN, &self.checkpoint.signing_bytes());
        let checkpoint_binds_chain = self.merkle_root == self.checkpoint.root_hash;
        SealedAuditStoreWrite {
            chain_decision,
            event_sequence: self.input.event_sequence,
            event_hash: self.event_hash,
            previous_event_hash: self
                .previous_event_hash
                .map(|h| h.to_vec())
                .unwrap_or_default(),
            record_digest: self.record_digest,
            merkle_root_hash: self.merkle_root,
            checkpoint_id,
            checkpoint_signature: self.checkpoint_signature,
            transparency_receipt: Vec::new(),
            witness_receipt: Vec::new(),
            event_kind: self.input.event_kind,
            anchor_kind: self.input.anchor_kind,
            sealed_payload_len: self.sealed_payload_len,
            append_only_guard,
            checkpoint_binds_chain,
            receipt_binds_checkpoint: false,
        }
    }

    /// Produce the durable-store WRITE for a `WitnessedTransparencyLog` anchor.
    /// `cosignatures` are verified against `witnesses` over THIS checkpoint; the
    /// `witness_count` / `witness_operator_count` gate inputs come from the REAL
    /// verified quorum, and the `witness_receipt` is the concatenation of exactly
    /// those verified 64-byte cosignatures (same verification core, so the receipt
    /// length always reflects the real witness count — a forged or duplicate cosig
    /// can neither inflate the count nor pad the receipt). The `transparency_receipt`
    /// is the genuine 64-byte Ed25519 checkpoint signature (an authenticated head
    /// receipt), and `receipt_binds_checkpoint` is set from a REAL verification of
    /// that signature against the log's key over this checkpoint (not hard-coded).
    /// `witness_threshold` is the deployment's required quorum (the gate needs `>= 2`
    /// witnesses across `>= 2` operators); `append_only_guard` is the caller's
    /// durable-store attestation. If the quorum, proofs, or receipt fall short, the
    /// store gate honestly rejects.
    pub fn witnessed_store_write(
        &self,
        witnesses: &[AuditWitness],
        cosignatures: &[WitnessCosignature],
        witness_threshold: i32,
        append_only_guard: bool,
    ) -> SealedAuditStoreWrite {
        // One verification core feeds BOTH the quorum counts and the receipt bytes.
        let quorum = verify_witnessed_checkpoint(&self.checkpoint, witnesses, cosignatures);
        let witness_receipt: Vec<u8> =
            verified_cosignatures(&self.checkpoint, witnesses, cosignatures)
                .into_iter()
                .flatten()
                .collect();

        // Copy the (Copy) chain input out of `&self` so the witnessed-anchor fields
        // can be set without consuming the staged append — one StagedAppend can yield
        // both this store-write and a witness-checkpoint input.
        let mut input = self.input;
        input.anchor_kind = SealedAuditAnchorKind::WitnessedTransparencyLog;
        input.witness_count = quorum.verified_witness_count;
        input.witness_operator_count = quorum.operator_count;
        input.witness_threshold = witness_threshold;
        input.transparency_receipt_present =
            input.signed_checkpoint_present && input.inclusion_proof_verified;

        let chain_decision = evaluate_sealed_audit_event_chain(input);
        let checkpoint_id = digest(CHECKPOINT_ID_DOMAIN, &self.checkpoint.signing_bytes());
        let checkpoint_binds_chain = self.merkle_root == self.checkpoint.root_hash;
        // The transparency receipt IS the checkpoint signature; binding is a REAL
        // Ed25519 verification of it against the log key over this checkpoint.
        let receipt_binds_checkpoint = verify_checkpoint(
            &self.verifying_key,
            &self.checkpoint,
            &self.checkpoint_signature,
        );

        SealedAuditStoreWrite {
            chain_decision,
            event_sequence: input.event_sequence,
            event_hash: self.event_hash,
            previous_event_hash: self
                .previous_event_hash
                .map(|h| h.to_vec())
                .unwrap_or_default(),
            record_digest: self.record_digest,
            merkle_root_hash: self.merkle_root,
            checkpoint_id,
            checkpoint_signature: self.checkpoint_signature,
            transparency_receipt: self.checkpoint_signature.to_vec(),
            witness_receipt,
            event_kind: input.event_kind,
            anchor_kind: input.anchor_kind,
            sealed_payload_len: self.sealed_payload_len,
            append_only_guard,
            checkpoint_binds_chain,
            receipt_binds_checkpoint,
        }
    }

    /// Produce the gate input for the sealed-audit WITNESS-CHECKPOINT gate
    /// ([`evaluate_sealed_audit_witness_checkpoint`]) for a hybrid-signed witnessed
    /// transparency log. Returns `None` unless the log has a hybrid checkpoint key
    /// (the gate requires a post-quantum / hybrid algorithm), so a classical-only log
    /// cannot reach this path.
    ///
    /// The cryptographic facts are REAL: `signature_algorithm` is the genuine
    /// `HybridEd25519MlDsa44` (whose `checkpoint_signature_min_len()` is 2484, the
    /// exact length of the hybrid signature this log produced); `consistency_proof_verified`
    /// and `consistency_proof_hash_count` come from the actual consistency proof
    /// verified at [`AuditEventLog::stage`]; the witness `witness_count` /
    /// `witness_operator_count` / `witness_cosignature_bytes` /
    /// `cosignatures_timestamped` / `cosignatures_bind_checkpoint` come from
    /// [`verify_hybrid_witness_quorum`] over THIS checkpoint (forged / duplicate /
    /// stale cosignatures cannot inflate them); `store_decision` is the real
    /// decision from the durable-store gate. Deployment facts the engine cannot
    /// observe (signing-key expiry / rotation window / previous-key retention,
    /// private-monitor retrieval, local-checkpoint availability, recovery state) are
    /// the explicit caller `attestation`, never fabricated. `checkpoint_origin` is the
    /// log's human-readable origin string (1..=255 bytes); `log_id_digest` and
    /// `signing_key_id_digest` are 32-byte out-of-band-pinned identifiers.
    #[allow(clippy::too_many_arguments)]
    pub fn witnessed_checkpoint_input(
        &self,
        hybrid_witnesses: &[HybridWitness],
        hybrid_cosignatures: &[HybridWitnessCosignature],
        witness_threshold: i32,
        store_decision: SealedAuditEventStoreDecision,
        attestation: &WitnessCheckpointAttestation,
        checkpoint_origin: &[u8],
        log_id_digest: &[u8; 32],
        signing_key_id_digest: &[u8; 32],
    ) -> Option<SealedAuditWitnessCheckpointInput> {
        // No hybrid key -> no post-quantum checkpoint signature -> this gate (which
        // requires a PQ/hybrid algorithm) is honestly unreachable.
        let hybrid_signature = self.hybrid_checkpoint_signature?;

        // REAL hybrid witness quorum over this checkpoint (counts, operator
        // diversity, serialized bytes, and the timestamp/binding flags are all
        // verification results, never echoed).
        let quorum =
            verify_hybrid_witness_quorum(&self.checkpoint, hybrid_witnesses, hybrid_cosignatures);

        Some(SealedAuditWitnessCheckpointInput {
            store_decision,
            anchor_kind: SealedAuditAnchorKind::WitnessedTransparencyLog,
            signature_algorithm: SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44,
            checkpoint_origin_len: i32::try_from(checkpoint_origin.len()).unwrap_or(i32::MAX),
            log_id_digest_len: i32::try_from(log_id_digest.len()).unwrap_or(i32::MAX),
            checkpoint_timestamp_s: self.checkpoint.timestamp_s,
            checkpoint_size: self.input.checkpoint_size,
            previous_checkpoint_size: self.input.previous_checkpoint_size,
            checkpoint_root_hash_len: DIGEST_LEN,
            checkpoint_signature_len: i32::try_from(hybrid_signature.len()).unwrap_or(i32::MAX),
            signing_key_id_digest_len: i32::try_from(signing_key_id_digest.len())
                .unwrap_or(i32::MAX),
            signing_key_not_expired: attestation.signing_key_not_expired,
            signing_key_rotation_window_valid: attestation.signing_key_rotation_window_valid,
            previous_signing_key_retained_for_verification: attestation
                .previous_signing_key_retained_for_verification,
            consistency_proof_verified: self.input.consistency_proof_verified,
            consistency_proof_hash_count: self.consistency_proof_hash_count,
            witness_count: quorum.verified_witness_count,
            witness_threshold,
            witness_operator_count: quorum.operator_count,
            witness_key_pins_present: true,
            witness_cosignature_bytes: quorum.cosignature_bytes,
            cosignatures_timestamped: quorum.all_timestamped,
            cosignatures_bind_checkpoint: quorum.all_bind_checkpoint,
            split_view_evidence_present: false,
            monitor_query_uses_private_retrieval: attestation.monitor_query_uses_private_retrieval,
            monitor_query_plaintext_selectors: 0,
            monitor_receives_only_digests: attestation.monitor_receives_only_digests,
            local_latest_checkpoint_available: attestation.local_latest_checkpoint_available,
            recovery_checkpoint_authenticated: attestation.recovery_checkpoint_authenticated,
            recovery_requires_user_verification: attestation.recovery_requires_user_verification,
        })
    }
}

/// Deployment facts the audit engine cannot observe from the checkpoint bytes,
/// supplied by the caller (the log operator + key-management + monitor layers) for
/// the witness-checkpoint gate, NEVER fabricated. A `false` makes the gate honestly
/// refuse: signing-key lifecycle (`signing_key_not_expired`,
/// `signing_key_rotation_window_valid`, `previous_signing_key_retained_for_verification`),
/// private-monitor posture (`monitor_query_uses_private_retrieval`,
/// `monitor_receives_only_digests`), and local checkpoint / recovery state
/// (`local_latest_checkpoint_available`, `recovery_checkpoint_authenticated`,
/// `recovery_requires_user_verification`).
#[derive(Debug, Clone, Copy)]
pub struct WitnessCheckpointAttestation {
    pub signing_key_not_expired: bool,
    pub signing_key_rotation_window_valid: bool,
    pub previous_signing_key_retained_for_verification: bool,
    pub monitor_query_uses_private_retrieval: bool,
    pub monitor_receives_only_digests: bool,
    pub local_latest_checkpoint_available: bool,
    pub recovery_checkpoint_authenticated: bool,
    pub recovery_requires_user_verification: bool,
}

impl WitnessCheckpointAttestation {
    /// A standard fully-attested deployment: the signing key is valid and within its
    /// rotation window with the prior key retained for verification, the monitor uses
    /// private retrieval receiving only digests, and the latest checkpoint is locally
    /// available. Use only when these genuinely hold.
    pub const fn standard() -> Self {
        Self {
            signing_key_not_expired: true,
            signing_key_rotation_window_valid: true,
            previous_signing_key_retained_for_verification: true,
            monitor_query_uses_private_retrieval: true,
            monitor_receives_only_digests: true,
            local_latest_checkpoint_available: true,
            recovery_checkpoint_authenticated: true,
            recovery_requires_user_verification: true,
        }
    }
}

/// The witness-client POLICY + NETWORK + RESPONSE facts for submitting a checkpoint
/// to the witness operators and collecting their cosignatures. These are facts the
/// audit engine cannot derive from the checkpoint bytes — the verifier-policy
/// snapshot, the pinned key/operator counts and quorum threshold, the submission /
/// monitor endpoint posture, the HTTP response and the cosignatures the witnesses
/// returned, and the local-recovery state. The caller (the witness-client transport
/// layer) supplies its REAL observed values; the engine never fabricates them. A
/// bad value makes the gate honestly refuse. `request_old_size` /
/// `request_consistency_proof_hash_count` describe the consistency proof the client
/// sent (old tree size and audit-path length); the checkpoint size the request and
/// response are bound to is taken from the REAL `checkpoint_decision`, not attested.
#[derive(Debug, Clone, Copy)]
pub struct WitnessClientAttestation {
    pub policy_digest_len: i32,
    pub policy_epoch: i64,
    pub policy_not_expired: bool,
    pub policy_binds_log_origin: bool,
    pub policy_binds_witness_operators: bool,
    pub log_public_key_pin_count: i32,
    pub witness_key_pin_count: i32,
    pub witness_operator_count: i32,
    pub submission_endpoint_count: i32,
    pub monitor_endpoint_count: i32,
    pub endpoints_use_https_or_bastion: bool,
    pub endpoint_tls_pins_present: bool,
    pub request_old_size: i64,
    pub request_consistency_proof_hash_count: i32,
    pub request_body_binds_policy_epoch: bool,
    pub response_status_code: i32,
    pub response_cosignature_count: i32,
    pub response_known_cosignature_count: i32,
    pub response_operator_count: i32,
    pub response_cosignatures_timestamped: bool,
    pub response_cosignatures_bind_checkpoint: bool,
    pub persist_latest_checkpoint_atomically: bool,
    pub split_view_alert_delivery_configured: bool,
    pub monitor_query_uses_private_retrieval: bool,
    pub monitor_query_uses_vrf_or_blinded_selector: bool,
    pub monitor_receives_only_digests: bool,
    pub recovery_checkpoint_authenticated: bool,
    pub recovery_requires_user_verification: bool,
}

impl WitnessClientAttestation {
    /// A standard fully-attested witness client for a quorum of `threshold` witnesses
    /// across `operator_count` operators at `policy_epoch`: a fresh policy binding the
    /// log origin + witness operators, pinned keys, HTTPS+TLS-pinned endpoints, a
    /// successful (200) witness response with `threshold` timestamped checkpoint-bound
    /// cosignatures atomically persisted, split-view alerting configured, and a
    /// private VRF/blinded digests-only monitor. `request_old_size` /
    /// `request_consistency_proof_hash_count` are the consistency proof the client
    /// sent. Use only when these genuinely hold.
    pub fn standard(
        policy_epoch: i64,
        threshold: i32,
        operator_count: i32,
        request_old_size: i64,
        request_consistency_proof_hash_count: i32,
    ) -> Self {
        Self {
            policy_digest_len: 32,
            policy_epoch,
            policy_not_expired: true,
            policy_binds_log_origin: true,
            policy_binds_witness_operators: true,
            log_public_key_pin_count: 1,
            witness_key_pin_count: threshold,
            witness_operator_count: operator_count,
            submission_endpoint_count: threshold,
            monitor_endpoint_count: 1,
            endpoints_use_https_or_bastion: true,
            endpoint_tls_pins_present: true,
            request_old_size,
            request_consistency_proof_hash_count,
            request_body_binds_policy_epoch: true,
            response_status_code: 200,
            response_cosignature_count: threshold,
            response_known_cosignature_count: threshold,
            response_operator_count: operator_count,
            response_cosignatures_timestamped: true,
            response_cosignatures_bind_checkpoint: true,
            persist_latest_checkpoint_atomically: true,
            split_view_alert_delivery_configured: true,
            monitor_query_uses_private_retrieval: true,
            monitor_query_uses_vrf_or_blinded_selector: true,
            monitor_receives_only_digests: true,
            recovery_checkpoint_authenticated: true,
            recovery_requires_user_verification: true,
        }
    }
}

/// Build the gate input for the sealed-audit WITNESS-CLIENT gate
/// ([`evaluate_sealed_audit_witness_client`]) from a REAL witnessed-checkpoint
/// decision plus the witness-client transport `attestation`. The
/// `witness_quorum_threshold`, `request_checkpoint_size`, and `response_latest_size`
/// are bound to the REAL `checkpoint_decision` (its `witness_threshold` and
/// `checkpoint_size`), so the request/response the client claims to have exchanged
/// must match the checkpoint actually witnessed — the policy/network/response facts
/// are the caller's honest observations, but they are anchored to the real checkpoint.
pub fn build_witness_client_input(
    checkpoint_decision: SealedAuditWitnessCheckpointDecision,
    attestation: &WitnessClientAttestation,
) -> SealedAuditWitnessClientInput {
    SealedAuditWitnessClientInput {
        checkpoint_decision,
        policy_digest_len: attestation.policy_digest_len,
        policy_epoch: attestation.policy_epoch,
        policy_not_expired: attestation.policy_not_expired,
        policy_binds_log_origin: attestation.policy_binds_log_origin,
        policy_binds_witness_operators: attestation.policy_binds_witness_operators,
        log_public_key_pin_count: attestation.log_public_key_pin_count,
        witness_key_pin_count: attestation.witness_key_pin_count,
        witness_operator_count: attestation.witness_operator_count,
        // Bound to the REAL witnessed-checkpoint decision's threshold + size.
        witness_quorum_threshold: checkpoint_decision.witness_threshold,
        submission_endpoint_count: attestation.submission_endpoint_count,
        monitor_endpoint_count: attestation.monitor_endpoint_count,
        endpoints_use_https_or_bastion: attestation.endpoints_use_https_or_bastion,
        endpoint_tls_pins_present: attestation.endpoint_tls_pins_present,
        request_old_size: attestation.request_old_size,
        request_checkpoint_size: checkpoint_decision.checkpoint_size,
        request_consistency_proof_hash_count: attestation.request_consistency_proof_hash_count,
        request_body_binds_policy_epoch: attestation.request_body_binds_policy_epoch,
        request_body_plaintext_selector_count: 0,
        response_status_code: attestation.response_status_code,
        response_latest_size: checkpoint_decision.checkpoint_size,
        response_cosignature_count: attestation.response_cosignature_count,
        response_known_cosignature_count: attestation.response_known_cosignature_count,
        response_operator_count: attestation.response_operator_count,
        response_cosignatures_timestamped: attestation.response_cosignatures_timestamped,
        response_cosignatures_bind_checkpoint: attestation.response_cosignatures_bind_checkpoint,
        persist_latest_checkpoint_atomically: attestation.persist_latest_checkpoint_atomically,
        split_view_alert_delivery_configured: attestation.split_view_alert_delivery_configured,
        monitor_query_uses_private_retrieval: attestation.monitor_query_uses_private_retrieval,
        monitor_query_uses_vrf_or_blinded_selector: attestation
            .monitor_query_uses_vrf_or_blinded_selector,
        monitor_query_plaintext_selectors: 0,
        monitor_receives_only_digests: attestation.monitor_receives_only_digests,
        recovery_checkpoint_authenticated: attestation.recovery_checkpoint_authenticated,
        recovery_requires_user_verification: attestation.recovery_requires_user_verification,
    }
}

/// Verifier-side facts the audit engine cannot derive from the log leaves, for the
/// offline proof-bundle gate: the verifier's pinned policy snapshot + key counts,
/// the proof-cache storage posture, the witness-checkpoint freshness window, and the
/// bundle/UI privacy flags. The caller (the offline verifier) supplies its REAL
/// values; the engine never fabricates them. The inclusion + consistency proofs and
/// their verified bools are NOT here — those are computed for real in
/// [`AuditEventLog::proof_bundle_input`].
#[derive(Debug, Clone, Copy)]
pub struct ProofBundleAttestation {
    pub bundle_format_version: i32,
    pub proof_bundle_persisted: bool,
    pub proof_cache_digest_len: i32,
    pub proof_cache_encrypted: bool,
    pub proof_cache_append_only: bool,
    pub local_proof_cache_available: bool,
    pub proof_cache_recovery_authenticated: bool,
    pub proof_cache_recovery_user_verified: bool,
    pub verifier_policy_snapshot_digest_len: i32,
    pub verifier_policy_matches_witness_policy: bool,
    pub verifier_log_key_pin_count: i32,
    pub verifier_witness_key_pin_count: i32,
    pub verified_witness_cosignature_count: i32,
    pub witness_timestamp_s: i64,
    pub verification_time_s: i64,
    pub max_witness_age_s: i64,
    pub monitor_freshness_checked: bool,
    pub extra_data_authenticated_or_opaque: bool,
    pub ui_status_digest_only: bool,
}

impl ProofBundleAttestation {
    /// A standard fully-attested offline verifier for a `threshold`-witness quorum:
    /// bundle format v1 persisted to an encrypted append-only locally-available proof
    /// cache, a 32-byte pinned policy snapshot matching the witness policy with the
    /// log key + `threshold` witness keys pinned and `threshold` cosignatures
    /// verified, a fresh witness checkpoint (`witness_timestamp_s` checked at
    /// `verification_time_s`, within `max_witness_age_s`), and digests-only UI. Use
    /// only when these genuinely hold.
    pub fn standard(
        threshold: i32,
        witness_timestamp_s: i64,
        verification_time_s: i64,
        max_witness_age_s: i64,
    ) -> Self {
        Self {
            bundle_format_version: 1,
            proof_bundle_persisted: true,
            proof_cache_digest_len: 32,
            proof_cache_encrypted: true,
            proof_cache_append_only: true,
            local_proof_cache_available: true,
            proof_cache_recovery_authenticated: true,
            proof_cache_recovery_user_verified: true,
            verifier_policy_snapshot_digest_len: 32,
            verifier_policy_matches_witness_policy: true,
            verifier_log_key_pin_count: 1,
            verifier_witness_key_pin_count: threshold,
            verified_witness_cosignature_count: threshold,
            witness_timestamp_s,
            verification_time_s,
            max_witness_age_s,
            monitor_freshness_checked: true,
            extra_data_authenticated_or_opaque: true,
            ui_status_digest_only: true,
        }
    }
}

/// An owned, durable-store WRITE for one accepted audit event: every byte artifact
/// the sealed-audit event-STORE gate ([`evaluate_sealed_audit_event_store_write`])
/// needs, produced from the SAME real chain artifacts as the event-chain gate
/// input. Owns its bytes so it can outlive the [`StagedAppend`]; borrow them for
/// the gate via [`SealedAuditStoreWrite::as_gate_input`].
pub struct SealedAuditStoreWrite {
    chain_decision: SealedAuditEventChainDecision,
    event_sequence: i64,
    event_hash: [u8; 32],
    previous_event_hash: Vec<u8>,
    record_digest: [u8; 32],
    merkle_root_hash: [u8; 32],
    checkpoint_id: [u8; 32],
    checkpoint_signature: [u8; 64],
    transparency_receipt: Vec<u8>,
    witness_receipt: Vec<u8>,
    event_kind: SealedAuditEventKind,
    anchor_kind: SealedAuditAnchorKind,
    sealed_payload_len: i32,
    append_only_guard: bool,
    checkpoint_binds_chain: bool,
    receipt_binds_checkpoint: bool,
}

impl SealedAuditStoreWrite {
    /// Borrow the owned artifacts into the gate's input view. Metadata fields are
    /// always 0 (the engine stores only sealed payloads + digests, never plaintext).
    pub fn as_gate_input(&self) -> SealedAuditEventStoreWrite<'_> {
        SealedAuditEventStoreWrite {
            chain_decision: self.chain_decision,
            event_sequence: self.event_sequence,
            event_hash: &self.event_hash,
            previous_event_hash: &self.previous_event_hash,
            record_digest: &self.record_digest,
            merkle_root_hash: &self.merkle_root_hash,
            checkpoint_id: &self.checkpoint_id,
            checkpoint_signature: &self.checkpoint_signature,
            transparency_receipt: &self.transparency_receipt,
            witness_receipt: &self.witness_receipt,
            event_kind: self.event_kind,
            anchor_kind: self.anchor_kind,
            sealed_payload_len: self.sealed_payload_len,
            plaintext_metadata_fields: 0,
            append_only_guard: self.append_only_guard,
            checkpoint_binds_chain: self.checkpoint_binds_chain,
            receipt_binds_checkpoint: self.receipt_binds_checkpoint,
        }
    }

    /// Evaluate this write against the sealed-audit event-store gate.
    pub fn evaluate(&self) -> SealedAuditEventStoreDecision {
        evaluate_sealed_audit_event_store_write(self.as_gate_input())
    }

    /// The 32-byte checkpoint id (SHA-256 commitment to the signed checkpoint).
    pub fn checkpoint_id(&self) -> [u8; 32] {
        self.checkpoint_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mercury_core::SealedAuditEventChainReason;

    fn log() -> AuditEventLog {
        AuditEventLog::new(SigningKey::from_bytes(&[3u8; 32]))
    }

    fn event<'a>(record: &'a [u8], kind: SealedAuditEventKind) -> AuditEvent<'a> {
        AuditEvent {
            sealed_record: record,
            kind,
            device_binding: b"device-key-material",
            actor_binding: b"actor-id-material",
            epoch_binding: b"epoch-material",
            room_epoch_binding: b"room-epoch-material",
        }
    }

    #[test]
    fn the_first_event_is_gate_accepted() {
        let mut log = log();
        let input = log.append(
            &event(b"sealed event 0", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            1_700_000_000,
        );
        // The engine genuinely verified its own proofs.
        assert!(input.inclusion_proof_verified);
        assert!(input.consistency_proof_verified);
        // First event: sequence 0, no previous hash required.
        assert_eq!(input.event_sequence, 0);
        assert_eq!(input.previous_event_hash_len, 0);
        assert_eq!(input.checkpoint_signature_len, 64);

        let decision = evaluate_sealed_audit_event_chain(input);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert!(decision.can_append_event && decision.tamper_evident && decision.append_only);
        assert!(!decision.plaintext_bytes_exposed);
    }

    #[test]
    fn a_chain_of_events_each_gate_accepts_with_real_proofs() {
        let mut log = log();
        // An MlsCommit event needs the room-epoch binding (the engine supplies 32).
        let kinds = [
            SealedAuditEventKind::DeviceKeyChange,
            SealedAuditEventKind::MlsCommit,
            SealedAuditEventKind::AiGrant,
            SealedAuditEventKind::BackupRestore,
            SealedAuditEventKind::MediaRetention,
        ];
        for (i, kind) in kinds.into_iter().enumerate() {
            let record = format!("sealed event {i}");
            let input = log.append(
                &event(record.as_bytes(), kind),
                &StorageAttestation::local_sealed_store(),
                1_700_000_000 + i as i64,
            );
            assert_eq!(input.event_sequence, i as i64);
            if i > 0 {
                assert_eq!(input.previous_event_hash_len, 32);
            }
            // Real proofs verified, and the checkpoint strictly advances.
            assert!(input.inclusion_proof_verified && input.consistency_proof_verified);
            assert_eq!(input.previous_checkpoint_size, i as i64);
            assert_eq!(input.checkpoint_size, (i + 1) as i64);

            let decision = evaluate_sealed_audit_event_chain(input);
            assert!(
                decision.accepted,
                "event {i} reason = {:?}",
                decision.reason
            );
        }
        assert_eq!(log.len(), 5);
    }

    #[test]
    fn the_engine_reports_real_digest_and_checkpoint_shapes() {
        let mut log = log();
        let input = log.append(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            42,
        );
        assert_eq!(input.event_hash_len, 32);
        assert_eq!(input.record_digest_len, 32);
        assert_eq!(input.merkle_leaf_hash_len, 32);
        assert_eq!(input.merkle_root_hash_len, 32);
        assert_eq!(input.device_binding_digest_len, 32);
        assert_eq!(input.room_epoch_digest_len, 32);
        assert_eq!(input.plaintext_field_count, 0);
        assert_eq!(input.plaintext_payload_bytes, 0);
        assert!(input.monotonic_counter_present && input.monotonic_counter_increases);
        assert!(input.signed_checkpoint_present);
        assert_eq!(input.checkpoint_timestamp_s, 42);
    }

    #[test]
    fn a_non_append_only_store_makes_the_gate_reject() {
        // A false storage attestation is forwarded honestly -> the gate rejects.
        let mut log = log();
        let mut attestation = StorageAttestation::local_sealed_store();
        attestation.storage_append_only = false;
        let input = log.append(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &attestation,
            1,
        );
        let decision = evaluate_sealed_audit_event_chain(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditEventChainReason::AppendOnlyStorageMissing
        );
    }

    #[test]
    fn identity_bindings_are_committed_in_the_event_hash() {
        // Same record + kind + everything else, but a different DEVICE binding must
        // yield a different committed Merkle root -- the event hash binds the
        // identity that produced the event, not just its bytes.
        let mut a = log();
        let mut b = log();
        let mut ev = event(
            b"identical record bytes",
            SealedAuditEventKind::DeviceKeyChange,
        );
        a.append(&ev, &StorageAttestation::local_sealed_store(), 1);
        ev.device_binding = b"a DIFFERENT device produced this";
        b.append(&ev, &StorageAttestation::local_sealed_store(), 1);
        assert_ne!(a.merkle_root(), b.merkle_root());
    }

    #[test]
    fn missing_forward_secrecy_makes_the_gate_reject() {
        let mut log = log();
        let mut attestation = StorageAttestation::local_sealed_store();
        attestation.forward_secret_rotated = false;
        let input = log.append(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &attestation,
            1,
        );
        let decision = evaluate_sealed_audit_event_chain(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditEventChainReason::ForwardSecrecyMissing
        );
    }

    /// Build witnesses + cosignatures over a staged checkpoint. `operators[i]` is
    /// witness `i`'s operator id; `forge` taints one cosignature (so it must not be
    /// counted). The cosignatures are made over the staged checkpoint's REAL bytes.
    fn witnessed(
        staged: &StagedAppend,
        seeds: &[u8],
        operators: &[u32],
        forge: Option<usize>,
    ) -> (Vec<AuditWitness>, Vec<WitnessCosignature>) {
        use crate::witness::cosign_checkpoint;
        let cp = *staged.checkpoint();
        let keys: Vec<SigningKey> = seeds
            .iter()
            .map(|s| SigningKey::from_bytes(&[*s; 32]))
            .collect();
        let witnesses = keys
            .iter()
            .zip(operators)
            .map(|(k, op)| AuditWitness {
                key: k.verifying_key(),
                operator_id: *op,
            })
            .collect();
        let mut cosigs: Vec<WitnessCosignature> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| WitnessCosignature {
                witness_index: i,
                signature: cosign_checkpoint(k, &cp),
            })
            .collect();
        if let Some(j) = forge {
            cosigs[j].signature[0] ^= 0x01;
        }
        (witnesses, cosigs)
    }

    #[test]
    fn a_witnessed_anchor_with_a_real_quorum_is_gate_accepted() {
        use mercury_core::SealedAuditAnchorKind;
        let mut log = log();
        let staged = log.stage(
            &event(b"sealed witnessed event", SealedAuditEventKind::MlsCommit),
            &StorageAttestation::local_sealed_store(),
            1_700_000_000,
        );
        // 3 witnesses across 2 distinct operators all cosign the staged checkpoint.
        let (witnesses, cosigs) = witnessed(&staged, &[11, 12, 13], &[1, 1, 2], None);
        let input = staged.finalize_witnessed(&witnesses, &cosigs, 2);

        assert!(matches!(
            input.anchor_kind,
            SealedAuditAnchorKind::WitnessedTransparencyLog
        ));
        // Counts come from the REAL verified quorum.
        assert_eq!(input.witness_count, 3);
        assert_eq!(input.witness_operator_count, 2);
        assert_eq!(input.witness_threshold, 2);
        // Receipt derived from real artifacts (signed checkpoint + verified inclusion).
        assert!(input.transparency_receipt_present);
        assert!(input.inclusion_proof_verified && input.consistency_proof_verified);

        let decision = evaluate_sealed_audit_event_chain(input);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
    }

    #[test]
    fn a_witnessed_anchor_without_operator_diversity_is_rejected() {
        // Two witnesses but only ONE operator -> the gate refuses the quorum.
        let mut log = log();
        let staged = log.stage(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            1,
        );
        let (witnesses, cosigs) = witnessed(&staged, &[11, 12], &[1, 1], None);
        let input = staged.finalize_witnessed(&witnesses, &cosigs, 2);
        assert_eq!(input.witness_operator_count, 1);

        let decision = evaluate_sealed_audit_event_chain(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditEventChainReason::WitnessQuorumMissing
        );
    }

    #[test]
    fn a_forged_cosignature_cannot_inflate_the_witnessed_quorum() {
        // Two witnesses across two operators, but the second cosignature is forged,
        // so the real quorum is only 1 -> below threshold -> the gate rejects. Proves
        // the witness counts are genuine, not just the requested threshold echoed.
        let mut log = log();
        let staged = log.stage(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            1,
        );
        let (witnesses, cosigs) = witnessed(&staged, &[11, 12], &[1, 2], Some(1));
        let input = staged.finalize_witnessed(&witnesses, &cosigs, 2);
        assert_eq!(input.witness_count, 1);
        assert_eq!(input.witness_operator_count, 1);

        let decision = evaluate_sealed_audit_event_chain(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditEventChainReason::WitnessQuorumMissing
        );
    }

    #[test]
    fn staging_then_finalizing_local_matches_a_plain_append() {
        // The staged path must not perturb the committed history: stage + finalize
        // local is exactly the LocalHashChain anchor a plain append produces.
        use mercury_core::SealedAuditAnchorKind;
        let mut log = log();
        let staged = log.stage(
            &event(b"only event", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            7,
        );
        let input = staged.finalize_local();
        assert!(matches!(
            input.anchor_kind,
            SealedAuditAnchorKind::LocalHashChain
        ));
        assert_eq!(input.witness_count, 0);
        assert!(!input.transparency_receipt_present);
        assert_eq!(log.len(), 1);
        assert!(evaluate_sealed_audit_event_chain(input).accepted);
    }

    #[test]
    fn a_local_store_write_is_gate_accepted() {
        let mut log = log();
        let write = log
            .stage(
                &event(b"sealed event 0", SealedAuditEventKind::DeviceKeyChange),
                &StorageAttestation::local_sealed_store(),
                1_700_000_000,
            )
            .local_store_write(true);
        // The store write carries the REAL 32-byte artifacts + 64-byte signature.
        assert_eq!(write.checkpoint_id().len(), 32);
        let input = write.as_gate_input();
        assert_eq!(input.event_hash.len(), 32);
        assert_eq!(input.record_digest.len(), 32);
        assert_eq!(input.merkle_root_hash.len(), 32);
        assert_eq!(input.checkpoint_id.len(), 32);
        assert_eq!(input.checkpoint_signature.len(), 64);
        assert_eq!(input.previous_event_hash.len(), 0); // seq 0 has no previous
        assert!(input.sealed_payload_len > 0);
        assert!(input.checkpoint_binds_chain);
        assert_eq!(input.plaintext_metadata_fields, 0);

        let decision = write.evaluate();
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert_eq!(decision.reason, SealedAuditEventStoreReason::Accepted);
        assert!(!decision.plaintext_bytes_exposed);
    }

    #[test]
    fn a_store_write_without_an_append_only_guard_is_rejected() {
        // append_only_guard is a caller-attested deployment fact; a false makes the
        // store gate honestly refuse the write.
        let mut log = log();
        let write = log
            .stage(
                &event(b"x", SealedAuditEventKind::DeviceKeyChange),
                &StorageAttestation::local_sealed_store(),
                1,
            )
            .local_store_write(false);
        let decision = write.evaluate();
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditEventStoreReason::AppendOnlyGuardMissing
        );
    }

    #[test]
    fn a_store_write_for_a_chain_rejected_event_is_rejected() {
        // The store gate depends on the chain gate: a false storage attestation makes
        // the chain decision reject, so the store write is rejected as ChainRejected
        // (the store cannot accept what the chain refused).
        let mut log = log();
        let mut attestation = StorageAttestation::local_sealed_store();
        attestation.storage_append_only = false;
        let write = log
            .stage(
                &event(b"x", SealedAuditEventKind::DeviceKeyChange),
                &attestation,
                1,
            )
            .local_store_write(true);
        let decision = write.evaluate();
        assert!(!decision.accepted);
        assert_eq!(decision.reason, SealedAuditEventStoreReason::ChainRejected);
    }

    #[test]
    fn a_chain_of_store_writes_advances_the_previous_hash() {
        let mut log = log();
        for i in 0..4 {
            let record = format!("sealed event {i}");
            let write = log
                .stage(
                    &event(record.as_bytes(), SealedAuditEventKind::DeviceKeyChange),
                    &StorageAttestation::local_sealed_store(),
                    1_700_000_000 + i as i64,
                )
                .local_store_write(true);
            let input = write.as_gate_input();
            assert_eq!(input.event_sequence, i as i64);
            if i > 0 {
                assert_eq!(input.previous_event_hash.len(), 32);
            }
            let decision = write.evaluate();
            assert!(
                decision.accepted,
                "event {i} reason = {:?}",
                decision.reason
            );
        }
    }

    #[test]
    fn a_witnessed_store_write_with_a_real_quorum_is_gate_accepted() {
        let mut log = log();
        let staged = log.stage(
            &event(b"sealed witnessed event", SealedAuditEventKind::MlsCommit),
            &StorageAttestation::local_sealed_store(),
            1_700_000_000,
        );
        // Capture the checkpoint to independently recompute the quorum below.
        let cp = *staged.checkpoint();
        // 3 witnesses across 2 distinct operators all cosign.
        let (witnesses, cosigs) = witnessed(&staged, &[11, 12, 13], &[1, 1, 2], None);
        let write = staged.witnessed_store_write(&witnesses, &cosigs, 2, true);

        let input = write.as_gate_input();
        // The transparency receipt is the 64-byte checkpoint signature.
        assert_eq!(input.transparency_receipt.len(), 64);
        assert!(input.receipt_binds_checkpoint);
        // NO DRIFT: the witness receipt is exactly the verified cosignatures, 64 each,
        // and its length matches the independently-recomputed verified witness count.
        let quorum = verify_witnessed_checkpoint(&cp, &witnesses, &cosigs);
        assert_eq!(quorum.verified_witness_count, 3);
        assert_eq!(
            input.witness_receipt.len(),
            quorum.verified_witness_count as usize * 64
        );

        let decision = write.evaluate();
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert_eq!(decision.reason, SealedAuditEventStoreReason::Accepted);
    }

    #[test]
    fn a_witnessed_store_write_short_of_quorum_is_rejected() {
        // Two witnesses across two operators, but one cosignature is forged, so the
        // REAL quorum is 1 (< threshold 2) -> the chain gate rejects -> the store gate
        // rejects as ChainRejected. The receipt holds only the 1 genuine cosignature.
        let mut log = log();
        let staged = log.stage(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            1,
        );
        let cp = *staged.checkpoint();
        let (witnesses, cosigs) = witnessed(&staged, &[11, 12], &[1, 2], Some(1));
        let write = staged.witnessed_store_write(&witnesses, &cosigs, 2, true);

        let input = write.as_gate_input();
        let quorum = verify_witnessed_checkpoint(&cp, &witnesses, &cosigs);
        assert_eq!(quorum.verified_witness_count, 1);
        // Receipt reflects only the genuine cosignature (forged one cannot pad it).
        assert_eq!(input.witness_receipt.len(), 64);

        let decision = write.evaluate();
        assert!(!decision.accepted);
        assert_eq!(decision.reason, SealedAuditEventStoreReason::ChainRejected);
    }

    fn hybrid_log() -> AuditEventLog {
        AuditEventLog::new_with_hybrid(
            SigningKey::from_bytes(&[3u8; 32]),
            crate::hybrid::HybridCheckpointSigningKey::generate(),
        )
    }

    /// Build hybrid witnesses + cosignatures over a staged checkpoint for the
    /// witness-checkpoint path. `operators[i]` is witness `i`'s operator id (so
    /// `&[10, 10, 20]` is 3 witnesses across 2 operators); each witness is an
    /// INDEPENDENT hybrid key (not the log key).
    fn hybrid_witnessed(
        staged: &StagedAppend,
        operators: &[u32],
    ) -> (Vec<HybridWitness>, Vec<HybridWitnessCosignature>) {
        use crate::hybrid::HybridCheckpointSigningKey;
        use crate::hybrid_witness::cosign_checkpoint_hybrid;
        let cp = *staged.checkpoint();
        let mut witnesses = Vec::new();
        let mut cosigs = Vec::new();
        for (i, op) in operators.iter().enumerate() {
            let sk = HybridCheckpointSigningKey::generate();
            let mut key_id = [0u8; 16];
            key_id[0] = i as u8 + 1;
            let mut cosig = cosign_checkpoint_hybrid(&sk, &cp, &key_id, 1_700_000_100);
            cosig.witness_index = i;
            witnesses.push(HybridWitness {
                key: sk.verifying_key(),
                operator_id: *op,
                key_id,
            });
            cosigs.push(cosig);
        }
        (witnesses, cosigs)
    }

    #[test]
    fn a_hybrid_witnessed_checkpoint_is_gate_accepted() {
        let mut log = hybrid_log();
        let staged = log.stage(
            &event(b"sealed witnessed event", SealedAuditEventKind::MlsCommit),
            &StorageAttestation::local_sealed_store(),
            1_700_000_000,
        );
        // A real accepted durable-store decision for this event.
        let store_decision = staged.local_store_write(true).evaluate();
        assert!(store_decision.accepted);
        // 3 hybrid witnesses across 2 operators all cosign.
        let (witnesses, cosigs) = hybrid_witnessed(&staged, &[10, 10, 20]);
        let input = staged
            .witnessed_checkpoint_input(
                &witnesses,
                &cosigs,
                2,
                store_decision,
                &WitnessCheckpointAttestation::standard(),
                b"mercury-audit-log",
                &[7u8; 32],
                &[8u8; 32],
            )
            .expect("a hybrid log yields a witnessed-checkpoint input");
        // The signature is the genuine hybrid algorithm + length the gate requires.
        assert!(matches!(
            input.signature_algorithm,
            SealedAuditCheckpointSignatureAlgorithm::HybridEd25519MlDsa44
        ));
        assert_eq!(input.checkpoint_signature_len, 2484);
        // Witness fields are the REAL verified quorum.
        assert_eq!(input.witness_count, 3);
        assert_eq!(input.witness_operator_count, 2);
        assert_eq!(input.witness_cosignature_bytes, 3 * 2508);
        assert!(input.cosignatures_timestamped && input.cosignatures_bind_checkpoint);

        let decision = evaluate_sealed_audit_witness_checkpoint(input);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert!(decision.can_publish_checkpoint && decision.can_request_witness_cosignature);
    }

    #[test]
    fn a_classical_only_log_has_no_witnessed_checkpoint_input() {
        // A log without a hybrid key cannot produce the PQ checkpoint signature the
        // witness gate requires, so the producer honestly returns None.
        let mut log = log();
        let staged = log.stage(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            1,
        );
        let store_decision = staged.local_store_write(true).evaluate();
        let (witnesses, cosigs) = hybrid_witnessed(&staged, &[10, 20]);
        assert!(
            staged
                .witnessed_checkpoint_input(
                    &witnesses,
                    &cosigs,
                    2,
                    store_decision,
                    &WitnessCheckpointAttestation::standard(),
                    b"mercury-audit-log",
                    &[7u8; 32],
                    &[8u8; 32],
                )
                .is_none()
        );
    }

    #[test]
    fn a_short_hybrid_quorum_is_rejected() {
        // Two hybrid witnesses but only ONE operator -> operator diversity fails ->
        // the gate rejects the quorum (the counts are real, not echoed).
        let mut log = hybrid_log();
        let staged = log.stage(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            1,
        );
        let store_decision = staged.local_store_write(true).evaluate();
        let (witnesses, cosigs) = hybrid_witnessed(&staged, &[10, 10]);
        let input = staged
            .witnessed_checkpoint_input(
                &witnesses,
                &cosigs,
                2,
                store_decision,
                &WitnessCheckpointAttestation::standard(),
                b"mercury-audit-log",
                &[7u8; 32],
                &[8u8; 32],
            )
            .unwrap();
        assert_eq!(input.witness_operator_count, 1);
        let decision = evaluate_sealed_audit_witness_checkpoint(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditWitnessCheckpointReason::WitnessQuorumRejected
        );
    }

    #[test]
    fn a_rejected_store_decision_makes_the_witness_checkpoint_reject() {
        // append_only_guard = false -> the store gate rejects -> the witness-checkpoint
        // gate honestly refuses (it cannot witness what was not durably stored).
        let mut log = hybrid_log();
        let staged = log.stage(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            1,
        );
        let store_decision = staged.local_store_write(false).evaluate();
        assert!(!store_decision.accepted);
        let (witnesses, cosigs) = hybrid_witnessed(&staged, &[10, 10, 20]);
        let input = staged
            .witnessed_checkpoint_input(
                &witnesses,
                &cosigs,
                2,
                store_decision,
                &WitnessCheckpointAttestation::standard(),
                b"mercury-audit-log",
                &[7u8; 32],
                &[8u8; 32],
            )
            .unwrap();
        let decision = evaluate_sealed_audit_witness_checkpoint(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditWitnessCheckpointReason::StoreRejected
        );
    }

    /// Build an ACCEPTED witnessed-checkpoint decision for the witness-client tests.
    fn accepted_checkpoint_decision() -> SealedAuditWitnessCheckpointDecision {
        let mut log = hybrid_log();
        let staged = log.stage(
            &event(b"sealed witnessed event", SealedAuditEventKind::MlsCommit),
            &StorageAttestation::local_sealed_store(),
            1_700_000_000,
        );
        let store_decision = staged.local_store_write(true).evaluate();
        let (witnesses, cosigs) = hybrid_witnessed(&staged, &[10, 10, 20]);
        let input = staged
            .witnessed_checkpoint_input(
                &witnesses,
                &cosigs,
                2,
                store_decision,
                &WitnessCheckpointAttestation::standard(),
                b"mercury-audit-log",
                &[7u8; 32],
                &[8u8; 32],
            )
            .unwrap();
        let decision = evaluate_sealed_audit_witness_checkpoint(input);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        decision
    }

    #[test]
    fn a_witness_client_submission_is_gate_accepted() {
        let checkpoint_decision = accepted_checkpoint_decision();
        // The witness client submitted to a 2-of-2-operator quorum at policy epoch 9.
        let attestation = WitnessClientAttestation::standard(9, 2, 2, 0, 0);
        let input = build_witness_client_input(checkpoint_decision, &attestation);
        // The request/response size is bound to the REAL checkpoint, threshold to the
        // REAL witnessed-checkpoint decision.
        assert_eq!(
            input.request_checkpoint_size,
            checkpoint_decision.checkpoint_size
        );
        assert_eq!(
            input.witness_quorum_threshold,
            checkpoint_decision.witness_threshold
        );
        let decision = evaluate_sealed_audit_witness_client(input);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert!(decision.can_publish_witnessed_checkpoint && decision.can_monitor_privately);
    }

    #[test]
    fn a_witness_client_on_a_rejected_checkpoint_is_rejected() {
        // A rejected witnessed-checkpoint decision cannot be published by the client.
        let mut log = hybrid_log();
        let staged = log.stage(
            &event(b"x", SealedAuditEventKind::DeviceKeyChange),
            &StorageAttestation::local_sealed_store(),
            1,
        );
        let store_decision = staged.local_store_write(true).evaluate();
        let (witnesses, cosigs) = hybrid_witnessed(&staged, &[10, 10]); // 1 operator
        let input = staged
            .witnessed_checkpoint_input(
                &witnesses,
                &cosigs,
                2,
                store_decision,
                &WitnessCheckpointAttestation::standard(),
                b"mercury-audit-log",
                &[7u8; 32],
                &[8u8; 32],
            )
            .unwrap();
        let checkpoint_decision = evaluate_sealed_audit_witness_checkpoint(input);
        assert!(!checkpoint_decision.accepted);

        let attestation = WitnessClientAttestation::standard(9, 2, 2, 0, 0);
        let client_input = build_witness_client_input(checkpoint_decision, &attestation);
        let decision = evaluate_sealed_audit_witness_client(client_input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditWitnessClientReason::CheckpointGateRejected
        );
    }

    #[test]
    fn a_witness_server_conflict_is_a_retryable_rejection() {
        // A 409 from a witness server signals a conflicting checkpoint -> the client
        // gate rejects as a retryable WitnessConflict (no publish).
        let checkpoint_decision = accepted_checkpoint_decision();
        let mut attestation = WitnessClientAttestation::standard(9, 2, 2, 0, 0);
        attestation.response_status_code = 409;
        let input = build_witness_client_input(checkpoint_decision, &attestation);
        let decision = evaluate_sealed_audit_witness_client(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditWitnessClientReason::WitnessConflict
        );
        assert!(decision.can_retry_witness_conflict);
    }

    #[test]
    fn a_witness_client_short_of_quorum_responses_is_rejected() {
        // The witness servers returned only 1 cosignature for a threshold-2 quorum ->
        // the client gate rejects the response.
        let checkpoint_decision = accepted_checkpoint_decision();
        let mut attestation = WitnessClientAttestation::standard(9, 2, 2, 0, 0);
        attestation.response_cosignature_count = 1;
        attestation.response_known_cosignature_count = 1;
        let input = build_witness_client_input(checkpoint_decision, &attestation);
        let decision = evaluate_sealed_audit_witness_client(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditWitnessClientReason::WitnessResponseRejected
        );
    }

    /// A complete witnessed log with `n` events on a hybrid key, plus an ACCEPTED
    /// witness-client decision over its final checkpoint, for the proof-bundle tests.
    /// Returns (log, witness_client_decision, checkpoint_root, checkpoint_size).
    fn witnessed_log(
        n: usize,
    ) -> (
        AuditEventLog,
        SealedAuditWitnessClientDecision,
        [u8; 32],
        usize,
    ) {
        let mut log = hybrid_log();
        // Append n events; capture the final checkpoint via the last stage.
        let mut last_checkpoint_decision = None;
        for i in 0..n {
            let record = format!("sealed event {i}");
            let staged = log.stage(
                &event(record.as_bytes(), SealedAuditEventKind::MlsCommit),
                &StorageAttestation::local_sealed_store(),
                1_700_000_000 + i as i64,
            );
            let store_decision = staged.local_store_write(true).evaluate();
            let (witnesses, cosigs) = hybrid_witnessed(&staged, &[10, 10, 20]);
            let cp_input = staged
                .witnessed_checkpoint_input(
                    &witnesses,
                    &cosigs,
                    2,
                    store_decision,
                    &WitnessCheckpointAttestation::standard(),
                    b"mercury-audit-log",
                    &[7u8; 32],
                    &[8u8; 32],
                )
                .unwrap();
            last_checkpoint_decision = Some(evaluate_sealed_audit_witness_checkpoint(cp_input));
        }
        let checkpoint_decision = last_checkpoint_decision.unwrap();
        assert!(checkpoint_decision.accepted);
        let client_attestation = WitnessClientAttestation::standard(9, 2, 2, 0, 0);
        let client_decision = evaluate_sealed_audit_witness_client(build_witness_client_input(
            checkpoint_decision,
            &client_attestation,
        ));
        assert!(client_decision.accepted);
        let checkpoint_size = log.len();
        let root = log.merkle_root();
        (log, client_decision, root, checkpoint_size)
    }

    #[test]
    fn an_offline_proof_bundle_is_gate_accepted() {
        // 4 events; verify the event at log_index 1 is included in the size-4
        // witnessed checkpoint, consistent with a previously-trusted size-2 checkpoint.
        let (log, client_decision, root, checkpoint_size) = witnessed_log(4);
        let previous_size = 2;
        let previous_root = log.merkle_root_at(previous_size).unwrap();
        let input = log
            .proof_bundle_input(
                client_decision,
                &root,
                checkpoint_size,
                previous_size,
                &previous_root,
                1,
                &[0x44; 32],
                &ProofBundleAttestation::standard(2, 1_700_000_500, 1_700_000_600, 86_400),
            )
            .expect("indices in range");
        // The inclusion + consistency proofs were really verified against the root.
        assert!(input.inclusion_proof_verified && input.inclusion_root_matches_checkpoint);
        assert!(input.consistency_proof_verified);
        assert_eq!(input.checkpoint_size, checkpoint_size as i64);

        let decision = evaluate_sealed_audit_proof_bundle(input);
        assert!(decision.accepted, "reason = {:?}", decision.reason);
        assert!(decision.can_verify_offline);
    }

    #[test]
    fn a_proof_bundle_against_a_tampered_root_is_rejected() {
        // Flipping a bit of the checkpoint root must make the REAL inclusion proof
        // fail to reconstruct it -> the gate rejects InclusionProofRejected.
        let (log, client_decision, root, checkpoint_size) = witnessed_log(4);
        let mut bad_root = root;
        bad_root[0] ^= 0x01;
        let previous_size = 2;
        let previous_root = log.merkle_root_at(previous_size).unwrap();
        let input = log
            .proof_bundle_input(
                client_decision,
                &bad_root,
                checkpoint_size,
                previous_size,
                &previous_root,
                1,
                &[0x44; 32],
                &ProofBundleAttestation::standard(2, 1_700_000_500, 1_700_000_600, 86_400),
            )
            .unwrap();
        // The forged root is load-bearing: the engine's REAL inclusion AND
        // consistency proofs reconstruct the GENUINE root, so neither matches the
        // tampered one. The gate checks the consistency-proof shape before the
        // inclusion proof, so the reported reason is ProofShapeRejected — either way
        // a forged checkpoint root cannot be accepted (fail-closed).
        assert!(!input.inclusion_root_matches_checkpoint);
        assert!(!input.consistency_proof_verified);
        let decision = evaluate_sealed_audit_proof_bundle(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditProofBundleReason::ProofShapeRejected
        );
    }

    #[test]
    fn a_proof_bundle_with_only_a_tampered_inclusion_is_rejected() {
        // Isolate the gate's INCLUSION branch: start from a genuinely-accepted bundle
        // (real root, real proofs) and force only inclusion_proof_verified false, so
        // the consistency/shape checks still pass and the gate must reject precisely
        // at the inclusion step.
        let (log, client_decision, root, checkpoint_size) = witnessed_log(4);
        let previous_size = 2;
        let previous_root = log.merkle_root_at(previous_size).unwrap();
        let mut input = log
            .proof_bundle_input(
                client_decision,
                &root,
                checkpoint_size,
                previous_size,
                &previous_root,
                1,
                &[0x44; 32],
                &ProofBundleAttestation::standard(2, 1_700_000_500, 1_700_000_600, 86_400),
            )
            .unwrap();
        assert!(evaluate_sealed_audit_proof_bundle(input).accepted);
        input.inclusion_proof_verified = false;
        let decision = evaluate_sealed_audit_proof_bundle(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditProofBundleReason::InclusionProofRejected
        );
    }

    #[test]
    fn a_proof_bundle_with_a_stale_witness_checkpoint_is_rejected() {
        // verification_time_s - witness_timestamp_s exceeds max_witness_age_s -> the
        // gate rejects the bundle as stale.
        let (log, client_decision, root, checkpoint_size) = witnessed_log(2);
        let previous_root = log.merkle_root_at(1).unwrap();
        let input = log
            .proof_bundle_input(
                client_decision,
                &root,
                checkpoint_size,
                1,
                &previous_root,
                0,
                &[0x44; 32],
                // Witness signed at t=1000 but verified at t=1_000_000 with max age 100.
                &ProofBundleAttestation::standard(2, 1_000, 1_000_000, 100),
            )
            .unwrap();
        let decision = evaluate_sealed_audit_proof_bundle(input);
        assert!(!decision.accepted);
        assert_eq!(
            decision.reason,
            SealedAuditProofBundleReason::WitnessFreshnessRejected
        );
    }

    #[test]
    fn a_proof_bundle_out_of_range_returns_none() {
        // log_index >= checkpoint_size is out of range -> None (no fabricated input).
        let (log, client_decision, root, checkpoint_size) = witnessed_log(2);
        let previous_root = log.merkle_root_at(1).unwrap();
        assert!(
            log.proof_bundle_input(
                client_decision,
                &root,
                checkpoint_size,
                1,
                &previous_root,
                checkpoint_size, // out of range (== size)
                &[0x44; 32],
                &ProofBundleAttestation::standard(2, 1_700_000_500, 1_700_000_600, 86_400),
            )
            .is_none()
        );
    }
}
