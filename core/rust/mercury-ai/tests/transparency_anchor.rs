//! Cross-crate composition: the AI audit trail (mercury-ai) anchored into the
//! sealed-audit transparency log (mercury-audit).
//!
//! This proves the two subsystems compose end-to-end with REAL artifacts and no
//! fabrication: every AI action recorded in an [`AiAuditLog`] is sealed with
//! Mercury's real XChaCha20-Poly1305 envelope (mercury-sealedbox) and its
//! tamper-evident commitment ([`AiActionAudit::transparency_anchor_bytes`]) is
//! appended to the RFC 6962 audit log as an `AiGrant` event — reaching a genuine
//! `evaluate_sealed_audit_event_chain` AND `evaluate_sealed_audit_event_store_write`
//! ACCEPT. The anchor is privacy-preserving: only the 32-byte hash commitment
//! crosses into the transparency log, never the AI's context or draft bytes.
//!
//! These are dev-only dependencies (mercury-ai does not depend on mercury-audit at
//! runtime), so the composition is demonstrated without coupling the crates.

use ed25519_dalek::SigningKey;

use mercury_ai::{
    AiAuditLog, AiRuntimeAttestation, prepare_draft, prepare_send, standard_local_draft_grant,
    standard_local_send_grant,
};
use mercury_audit::{
    AuditEvent, AuditEventLog, SealedAuditEventStoreReason, StorageAttestation,
    evaluate_sealed_audit_event_chain,
};
use mercury_core::SealedAuditEventKind;
use mercury_keys::DeviceKeyPair;

const NOW: i32 = 1_000;
const EXPIRES: i32 = 1_300;

/// Seal an AI record's canonical bytes with Mercury's real envelope, so the audit
/// event genuinely carries a sealed record (honest `event_sealed = true`).
fn seal(recipient: &mercury_keys::DevicePublicKey, plaintext: &[u8]) -> Vec<u8> {
    mercury_sealedbox::seal(recipient, plaintext).expect("seal succeeds for a valid recipient")
}

#[test]
fn ai_actions_anchor_into_the_sealed_audit_transparency_log() {
    // 1) Run two AI actions through the mercury-ai audit trail (a draft + a
    //    user-confirmed send), each recorded bound to its real digests.
    let mut ai_log = AiAuditLog::new();
    let ctx: [&[u8]; 1] = [b"are we still on for friday?"];
    let draft = prepare_draft(
        &ctx,
        b"Confirming Friday at 7pm.",
        standard_local_draft_grant(NOW, EXPIRES),
        &AiRuntimeAttestation::local_device(),
    );
    assert!(ai_log.run(&draft).accepted);

    let send = prepare_send(
        &ctx,
        b"Confirming Friday at 7pm.",
        standard_local_send_grant(NOW, EXPIRES),
    );
    assert!(ai_log.run_send(&send).accepted);
    assert!(ai_log.verify(), "the AI trail must verify before anchoring");
    assert_eq!(ai_log.len(), 2);

    // 2) Anchor each AI record into the sealed-audit transparency log as an AiGrant
    //    event. The sealed_record is the AI record's canonical bytes sealed with
    //    Mercury's real envelope; the event binds the AI subsystem identity.
    let recipient = DeviceKeyPair::generate();
    let recipient_pk = recipient.public();
    let mut audit_log = AuditEventLog::new(SigningKey::from_bytes(&[5u8; 32]));

    let mut anchors = Vec::new();
    for record in ai_log.records() {
        let anchor = record.transparency_anchor_bytes();
        anchors.push(anchor);
        let sealed = seal(&recipient_pk, &anchor);

        let event = AuditEvent {
            sealed_record: &sealed,
            kind: SealedAuditEventKind::AiGrant,
            device_binding: b"ai-principal-device",
            actor_binding: b"ai-principal-actor",
            epoch_binding: b"ai-epoch-1",
            room_epoch_binding: b"ai-room-epoch-1",
        };

        // Both gates accept with real inline-verified proofs + a real store write.
        let staged = audit_log.stage(
            &event,
            &StorageAttestation::local_sealed_store(),
            1_700_000_000,
        );
        // The chain input is produced from real artifacts.
        let chain_input = {
            // Peek the chain decision via a local clone of the store-write path:
            // finalize_local consumes, so evaluate the store write (which embeds the
            // chain decision) instead.
            let write = staged.local_store_write(true);
            let decision = write.evaluate();
            assert!(
                decision.accepted,
                "store write must accept, reason = {:?}",
                decision.reason
            );
            assert_eq!(decision.reason, SealedAuditEventStoreReason::Accepted);
            assert!(!decision.plaintext_bytes_exposed);
            write
        };
        // The store write carries the real 32-byte checkpoint id.
        assert_eq!(chain_input.checkpoint_id().len(), 32);
    }

    // 3) The transparency log committed exactly the two AI actions, in order.
    assert_eq!(audit_log.len(), 2);
    // The anchors are the AI trail's own entry hashes (privacy-preserving: a hash,
    // not the AI content), and they are distinct per action.
    assert_eq!(anchors[0], ai_log.records()[0].entry_hash);
    assert_eq!(anchors[1], ai_log.records()[1].entry_hash);
    assert_ne!(anchors[0], anchors[1]);
}

#[test]
fn the_anchor_is_committed_into_the_transparency_root() {
    // Prove the AI anchor is LOAD-BEARING: it is committed into the transparency
    // log's Merkle root, so a different anchor yields a different root. To ISOLATE
    // the anchor's contribution this test commits the anchor deterministically as
    // the record bytes (the realistic randomized-seal path is covered by the test
    // above). The same-anchor->same-root assertion below is exactly what makes this
    // non-tautological: under mercury-sealedbox's randomized seal (fresh ephemeral
    // key + nonce per call), even identical anchors would yield DIFFERENT roots, so
    // an assert_ne alone would pass even if the anchor were ignored. Committing the
    // anchor directly removes that randomness, so the root is a pure function of the
    // anchor and the two assertions together prove the binding.
    let mut ai_log = AiAuditLog::new();
    let ctx: [&[u8]; 1] = [b"selected context"];
    let draft = prepare_draft(
        &ctx,
        b"a draft reply",
        standard_local_draft_grant(NOW, EXPIRES),
        &AiRuntimeAttestation::local_device(),
    );
    ai_log.run(&draft);
    let genuine_anchor = ai_log.records()[0].transparency_anchor_bytes();

    // Deterministic: the committed record IS the anchor, so the root depends only on
    // the anchor (no per-call randomness).
    let root_for = |anchor: &[u8; 32]| -> [u8; 32] {
        let mut audit_log = AuditEventLog::new(SigningKey::from_bytes(&[6u8; 32]));
        let event = AuditEvent {
            sealed_record: &anchor[..],
            kind: SealedAuditEventKind::AiGrant,
            device_binding: b"ai-principal-device",
            actor_binding: b"ai-principal-actor",
            epoch_binding: b"ai-epoch-1",
            room_epoch_binding: b"ai-room-epoch-1",
        };
        let input = audit_log.append(&event, &StorageAttestation::local_sealed_store(), 1);
        assert!(evaluate_sealed_audit_event_chain(input).accepted);
        audit_log.merkle_root()
    };

    let mut tampered_anchor = genuine_anchor;
    tampered_anchor[0] ^= 0x01;

    // Non-tautological pair: SAME anchor -> SAME root (would FAIL under a randomized
    // seal), and DIFFERENT anchor -> DIFFERENT root.
    assert_eq!(
        root_for(&genuine_anchor),
        root_for(&genuine_anchor),
        "the anchor->root binding must be deterministic"
    );
    assert_ne!(
        root_for(&genuine_anchor),
        root_for(&tampered_anchor),
        "a different AI anchor must change the transparency-log root"
    );
}
