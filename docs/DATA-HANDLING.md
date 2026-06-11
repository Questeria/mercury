# Mercury — data handling, persistence & deletion

This document states plainly **what Mercury stores, where, and what happens to it** when you install,
update, uninstall, or ask to delete your data. It is written to be checkable against the code, not to
reassure — where something is a best-effort or a known residual, it says so.

## TL;DR

| You... | Your data... |
|---|---|
| **install** | a new account (identity keys, profile) is created on your device |
| **update** the app | survives — nothing is lost |
| **uninstall + reinstall** (same machine) | survives by default and is restored on next launch |
| move to a **new machine** | restore from an encrypted backup you exported |
| ask to **delete your data** | the account + keys are erased from your device, irreversibly; optionally your messages are wiped from the people you messaged too |

Nothing recoverable ever leaves your device unless **you** export a backup. The relay only ever holds
opaque ciphertext it cannot read.

## What is stored, and where

**On your device (everything that matters):**
- An **encrypted snapshot** — `mercury-snapshot.bin` in the OS application-data directory
  (`%APPDATA%/com.mercury.messaging` on Windows; the platform app-data dir elsewhere). It contains
  your identity keys, contacts, message sessions, and message history.
- It is sealed with **XChaCha20-Poly1305** under a **device key** that lives in the **OS keychain**
  (Windows Credential Manager / macOS Keychain / Linux Secret Service) — never in a file, never
  exported. The snapshot on disk is useless without that key.
- The app **refuses to start fresh** if the keychain is present but unreadable (locked vault, denied
  access), specifically so a transient error can never silently overwrite your account.

**On the relay (it can read none of it):**
- **Queued ciphertext** — messages addressed to you that you haven't fetched yet. Opaque bytes with a
  **time-to-live**; a background sweeper clears them after expiry, and they're deleted on delivery.
- Your **published contact card** — a prekey bundle others use to start a conversation with you. No
  message content; it cannot decrypt anything.
- If you claimed a **username**, a `username → account-id` entry in a **key-transparency (KT) log**.
  This log is **append-only by design** — that is the anti-impersonation property that lets your
  contacts detect if anyone ever tried to swap the key behind your name.

## Across an update

Your data **survives updates**. The snapshot path and the keychain entry are keyed to stable
identifiers (`com.mercury.messaging`), which do not change between versions, so an update reattaches
to the same account. The app's quit/restart flush and crash-consistent (temp-then-rename) writes mean
an interrupted update cannot corrupt or reset the snapshot.

## Across uninstall / reinstall

**Same machine:** by default your data **survives**. The uninstaller does not erase the app-data
directory, and the OS keychain entry persists independently of the app, so a reinstall reattaches to
your existing account and restores it.

**New machine, or after a full wipe:** restore from an **encrypted backup**. In the Recovery panel,
*Export an encrypted backup* writes a passphrase-sealed `.mercbak` file (memory-hard **Argon2id** key
derivation). On a fresh install, the first-run screen offers **"Already have an account? Restore from
a backup"**, which restores everything from that file + your passphrase.

> The passphrase **is** the security of the backup. It is never stored; a lost passphrase makes the
> file permanently unreadable. Keep the file offline and the passphrase somewhere safe.

## Deleting your data

The Recovery panel's **Delete everything on this device** is a real, irreversible erase:

1. It removes the **device key** from the OS keychain **first**. If that step fails (e.g. a locked
   keychain), it stops and deletes **nothing** — your account stays intact and you can retry. There
   is no half-deleted state.
2. Once the key is gone (which makes the snapshot ciphertext permanently unopenable), it deletes the
   snapshot file and restarts the app into a clean first run.

There is an optional checkbox, **"Also delete my messages from the people I've messaged."** When
ticked, before erasing locally the app sends a *delete-for-everyone* to every conversation and
ends/leaves every group, so reachable peers drop their copies too.

**Honest limits of deletion** (we will not pretend otherwise):
- *Local erase is complete and irreversible* for this device. If you have no backup, the account is
  gone for good.
- *"Also wipe from peers" is best-effort.* It reaches contacts who are reachable; someone offline
  forever, or who already kept a copy of a message, is beyond our reach. We cannot guarantee deletion
  on hardware we don't control.
- *Server-side traces are minor and mostly self-clearing.* Undelivered queued messages **auto-expire**
  via the relay's TTL. Your published contact card becomes useless the moment your account is gone (a
  handshake to a deleted account simply fails). A claimed **username stays in the append-only
  transparency log** — that record is a public handle→random-id mapping, not message content, and the
  log is append-only on purpose; a future version may append a revocation marker.

## What is NOT collected

No phone number, no email, no contact-list upload, no analytics or telemetry, no server-side message
storage in the clear, no key escrow. The relay authenticates the two write paths cryptographically and
learns nothing it could decrypt by doing so.

## Roadmap (not yet shipped — listed so this page never overstates)

- **Automatic local backups** to a durable folder, so survival doesn't depend on remembering to export.
- An **uninstall-time prompt**: *Keep your encrypted data for a reinstall, or erase everything now?*
- **Opt-in end-to-end-encrypted cloud backup** (the relay would store only an opaque, passphrase-sealed
  blob it cannot read) for effortless restore on any machine.
- **Server-side revocation** of the contact card and username on account delete.

Each of these will be added to the table above only once it actually ships.
