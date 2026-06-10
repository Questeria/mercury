# Group Chat Command Envelopes

Generated: 2026-05-28

## Status

Group chat readiness now has backend command envelopes in addition to checked prototype fixtures.

```text
run_group_chat_mls_ready
run_group_chat_mls_setup_required
run_group_chat_membership_sync_required
run_group_chat_plaintext_metadata_forbidden
run_group_chat_high_security_mls_required
run_group_chat_high_security_pq_required
run_group_chat_mls_provider_security_required
```

These commands use the same `PrototypeBackendCommand` gate as the anonymous relay and storage diagnostics. Human-owned commands with 32-byte command IDs and zero plaintext payload can inspect group-readiness decisions; remote AI and local AI actors cannot run these human-owned readiness checks.

## Security Value

The UI no longer needs to treat group readiness as fixture-only. Desktop, mobile, and platform bridge code can ask for group readiness through the command path and receive both:

```text
command
result
```

The command view proves the request passed command authorization. The result remains the checked `group_chat` decision payload.

## Verification

Run:

```powershell
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_group_chat_mls_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_group_chat_mls_provider_security_required
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```
