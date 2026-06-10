# Production Store Session Prototype

Generated: 2026-05-28

## Status

Mercury now has a composed production-store session prototype in `mercury-core`:

```text
PrototypeProductionStoreSessionInput
PrototypeProductionStoreSessionOutcome
PrototypeProductionStoreSessionReason
run_prototype_production_store_session(...)
```

This is a non-UI integration harness for the local secure-storage chain. It does not implement a production database, but it proves the backend order of operations before a platform adapter exists.

## Session Flow

The prototype composes:

```text
keychain/keystore adapter decision
  -> local-store unlock decision
  -> production-open manifest decision
  -> explicit WAL replay branch when required
  -> accepted database open
  -> accepted sealed/hash/public write through storage policy
  -> owned sealed/hash/public read
```

No plaintext payload variant is introduced. The outcome carries `plaintext_exposed = false`.

## Stop Points

Stable session reason labels:

```text
ACCEPTED
KEYCHAIN_REJECTED
UNLOCK_REJECTED
PRODUCTION_OPEN_REJECTED
WAL_REPLAY_REQUIRED
STORE_WRITE_REJECTED
STORE_READ_MISSING
```

Rejected branches stop before later side effects:

- keychain rejection stops before unlock/open/write/read side effects
- unlock rejection stops before production open
- WAL replay required replays WAL but does not open records or write
- production-open rejection stops before adapter open
- store-write rejection stops before read

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_production_store_session
cargo test -p mercury-bindings --test prototype_fixtures
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype production_store_session_happy_path
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_production_store_session_happy_path
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused test covers a complete accepted store session, keychain rejection, WAL replay branch, rejected plaintext write, and stable session reason codes/labels.

Checked prototype fixtures cover happy path, keychain rejection, WAL replay required, and plaintext write rejection states.

The same branches are now reachable through backend command envelopes:

```text
run_production_store_session_happy_path
run_production_store_session_keychain_rejected
run_production_store_session_wal_replay_required
run_production_store_session_write_rejected
```

## Next Backend Step

The platform local-store adapter gate is documented in `docs/52_PLATFORM_LOCAL_STORE_ADAPTER_GATE.md`.
