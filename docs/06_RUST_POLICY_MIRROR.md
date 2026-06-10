# Rust Policy Mirror

Generated: 2026-05-27

## Status

Mercury now has a Rust mirror crate for the Phase 1 envelope policy:

- `core/rust/mercury-policy/src/lib.rs`
- `core/rust/mercury-policy/tests/envelope_vectors.rs`
- root `Cargo.toml` workspace

The Rust library mirrors the staged Helix API:

```text
validate_identity_v1(...)
validate_order_v1(...)
validate_content_v1(...)
first_reject(...)
audit_class_for_reason(...)
validate_envelope(...)
```

## Verification Performed

Passed:

```powershell
cargo fmt
cargo check -p mercury-policy --lib
python .\tools\check_envelope_vectors.py
```

Blocked locally:

```powershell
cargo test
```

Reason: the installed Windows Rust toolchain targets `x86_64-pc-windows-msvc`, but this machine does not currently have the MSVC linker `link.exe` available. WSL also does not have `cargo` installed. The integration test is still present and should run on a Rust environment with a working linker.

GitHub CI now runs the Rust tests on Linux. See `docs/07_CI_AND_VERIFICATION.md`.

## Why Keep The Rust Mirror

The Helix policy is the assurance-oriented source for deterministic policy checks. The Rust mirror is the production integration target for the eventual client core. Keeping both tied to the same JSON vectors lets Mercury catch drift early.
