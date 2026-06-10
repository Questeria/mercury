# Group chat — design and honest limits (Mercury Groups v1)

Group messaging shipped in `0.1.25`. This document states exactly how it works and what its
boundaries are — the same honesty discipline as `docs/USERNAMES.md`.

## How it works

- **Crypto: real MLS.** Every group is an MLS group (OpenMLS, audited RustCrypto provider,
  ciphersuite `MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519`). One group ciphertext per message;
  membership changes rotate the epoch, so a removed/departed member cryptographically loses access
  to everything sent afterwards (forward secrecy on membership change), and additions get access
  only from their join epoch forward.
- **Transport: the existing verified 1:1 channels.** Invitations, key packages, Welcomes,
  membership commits, and group messages all travel as control frames inside the SAME sealed,
  post-quantum 1:1 ratchet channels Mercury already uses. The relay is completely unchanged: it
  sees the same opaque 1:1 ciphertext it always did and gains no new surface to attack or trust.
- **Trust path: contacts only.** You can only be invited by someone you have already added (and
  can verify via safety numbers); your app ignores group invitations from strangers. Inside the
  group ciphertext, every message is MLS-attributed to its sender's credential, and that
  attribution is cross-checked against the authenticated 1:1 carrier — a member cannot forge
  another member's authorship. (Limited transitivity, by design: when you join a group, your
  fellow members' self-authenticating cards are fetched so messages can be delivered, which adds
  them to your contacts — so a co-member from a shared group can later invite you to a *new* group
  they create. They still cannot read or alter the original group, and you verify any new contact's
  safety number as always.)
- **Persistence.** MLS group secrets ride the same encrypted, keychain-sealed snapshot as
  everything else; groups survive restarts.

## Honest limits (v1, disclosed)

1. **Classical MLS, not post-quantum (yet).** Mercury's 1:1 chats are PQ-hybrid. For groups, the
   post-quantum X-Wing suite exists in `mercury-mls` but stays **feature-gated off**, because the
   only published provider release pins libcrux versions with open high-severity RUSTSEC
   advisories. Shipping known-vulnerable crypto to gain "PQ" labeling would be fake security; we
   ship audited classical MLS and flip the gate when upstream patches. (`mercury-mls/Cargo.toml`
   documents the exact advisories.)
2. **Creator-administered.** The creator invites members, admits their key packages, processes
   leaves (committing the removal so the epoch rotates), and is the only one who can close the
   group. If the creator is offline, joins/leaves queue until they return; if the creator closes
   the group (or leaves), it ends for everyone. Member-driven administration (proposals any member
   can commit) is a later increment.
3. **Fan-out bandwidth + metadata.** A group message is one MLS ciphertext sent N−1 times (once
   per member's 1:1 channel). The relay cannot read anything, but it can observe the fan-out
   *timing pattern*, just as it already observes 1:1 traffic timing. Group size is capped at
   **16 members** to keep this honest; a relay-side group route would lift the cap later.
4. **Offline joins.** Adding a member requires their app to answer with a key package, so an
   offline invitee joins when they next come online (the invitation waits in their relay mailbox).
5. **Text only in groups (v1).** Attachments and delivery receipts remain 1:1 features for now
   (receipts in groups would be an N×N traffic storm; a quieter design comes later). Disappearing
   timers apply locally per group but are not yet propagated to other members.
6. **Auto-admission.** Invitees from a known contact are admitted automatically (like most
   messengers' group adds). An explicit accept/decline step is a later refinement.
7. **Message-loss window on membership change.** A message encrypted at epoch N can briefly race a
   commit that moves the group to epoch N+1; the late frame fails to decrypt and is dropped (never
   shown wrong, never crashes). Senders' messages converge after the commit lands.

The cryptographic engine (`mercury-mls/src/engine.rs`) and the full three-member lifecycle —
create, invite, join, attribution, leave-with-epoch-rotation, restart survival, close — are covered
by integration tests in `mercury-mls` and `mercury-app`.
