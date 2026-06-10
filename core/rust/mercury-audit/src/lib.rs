//! Mercury sealed-audit transparency engine (audit / Stage 12).
//!
//! Produces the inputs for mercury-core's sealed-audit event-chain policy gate
//! (`evaluate_sealed_audit_event_chain`) from REAL audit data — an RFC 6962
//! append-only Merkle hash tree over the event records, a hash-chained event log,
//! an Ed25519 signed checkpoint, and (for witnessed anchors) a witness quorum
//! reusing mercury-kt's signed-tree-head machinery. The engine does the crypto;
//! the gate decides policy, mirroring mercury-kt / mercury-mls / mercury-backup.
//!
//! Built so far (increment 1a): the RFC 6962 Merkle tree + inclusion proofs
//! ([`merkle`]). The hash-chained event log, consistency proofs, the signed
//! checkpoint, and the gate-input producer are later increments. Only audited
//! SHA-256 is used.

#![forbid(unsafe_code)]

pub mod checkpoint;
pub mod events;
pub mod hybrid;
pub mod hybrid_witness;
pub mod merkle;
pub mod witness;

pub use checkpoint::{SealedAuditCheckpoint, sign_checkpoint, verify_checkpoint};
pub use events::{
    AuditEvent, AuditEventLog, SealedAuditCheckpointSignatureAlgorithm,
    SealedAuditEventChainDecision, SealedAuditEventChainInput, SealedAuditEventStoreDecision,
    SealedAuditEventStoreReason, SealedAuditEventStoreWrite, SealedAuditStoreWrite,
    SealedAuditWitnessCheckpointDecision, SealedAuditWitnessCheckpointInput,
    SealedAuditWitnessCheckpointReason, SealedAuditWitnessClientDecision,
    SealedAuditWitnessClientInput, SealedAuditWitnessClientReason, StagedAppend,
    StorageAttestation, WitnessCheckpointAttestation, WitnessClientAttestation,
    build_witness_client_input, evaluate_sealed_audit_event_chain,
    evaluate_sealed_audit_event_store_write, evaluate_sealed_audit_witness_checkpoint,
    evaluate_sealed_audit_witness_client,
};
pub use merkle::{
    consistency_proof, inclusion_proof, leaf_hash, merkle_root, verify_consistency,
    verify_inclusion,
};
pub use witness::{
    AuditWitness, WitnessCosignature, WitnessQuorum, cosign_checkpoint, verified_cosignatures,
    verify_witnessed_checkpoint,
};
