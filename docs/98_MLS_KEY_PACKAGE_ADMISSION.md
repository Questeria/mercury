# MLS KeyPackage Admission Gate

Generated: 2026-05-28

## Status

Mercury now has a backend MLS KeyPackage admission gate in `mercury-core`:

```text
MlsKeyPackageAdmissionInput
MlsKeyPackageAdmissionDecision
MlsKeyPackageAdmissionReason
evaluate_mls_key_package_admission(...)
```

The gate is a pre-production MLS membership-change contract. It decides whether a proposed member KeyPackage can be consumed to add a member and send a Welcome, without letting UI or platform code bypass group readiness, protocol/suite binding, signature, credential, lifetime, replay, or plaintext-identity checks.

## Research Basis

RFC 9420 defines KeyPackages as the external credential/leaf material used when adding clients to MLS groups. This gate tracks that shape by requiring:

- accepted group-chat readiness before membership changes
- matching MLS protocol version and group ciphersuite class
- valid leaf node, leaf-node signature, and KeyPackage signature
- valid and group-supported credential material
- required capabilities present before admission
- bounded, current KeyPackage lifetime
- leaf source bound to a KeyPackage
- supported extensions only
- distinct init and encryption keys
- fixed digest/key lengths for the current Mercury prototype contract
- one-time KeyPackage hash use
- zero plaintext identity fields

Source: <https://www.rfc-editor.org/rfc/rfc9420.html>

## Accepted Output

Accepted output enables:

```text
can_add_member = true
can_send_welcome = true
prevents_key_reuse = true
plaintext_bytes_exposed = false
```

Rejected output never enables add-member or Welcome sending.

## Checked Fixtures

Prototype fixtures:

```text
mls_key_package_admission_ready
mls_key_package_admission_group_rejected
mls_key_package_admission_lifetime_rejected
mls_key_package_admission_suite_mismatch
mls_key_package_admission_credential_rejected
mls_key_package_admission_replay_rejected
mls_key_package_admission_plaintext_rejected
```

Backend commands:

```text
run_mls_key_package_admission_ready
run_mls_key_package_admission_group_rejected
run_mls_key_package_admission_lifetime_rejected
run_mls_key_package_admission_suite_mismatch
run_mls_key_package_admission_credential_rejected
run_mls_key_package_admission_replay_rejected
run_mls_key_package_admission_plaintext_rejected
```

## UI Contract

UI and platform code must not treat a pending group invite or membership action as actionable unless the backend returns:

```text
accepted = true
can_add_member = true
can_send_welcome = true
```

Use `requires_sync`, `requires_mls_setup`, `requires_pq_upgrade`, and `requires_user_action` for routing. Do not infer readiness from a displayed member profile, a local room cache, a KeyPackage QR/deep link, or a server-provided invite.

## Verification

Run:

```powershell
cargo test -p mercury-core --test mls_key_package_admission
cargo test -p mercury-core --test prototype_backend_command
cargo test -p mercury-bindings --test prototype_fixtures
cargo test -p mercury-bindings --test backend_commands
cargo test -p mercury-bindings --test ui_sim_cli
cargo run -p mercury-bindings --bin mercury-ui-sim -- --prototype mls_key_package_admission_ready
cargo run -p mercury-bindings --bin mercury-ui-sim -- --command run_mls_key_package_admission_ready
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

## Next Backend Step

The follow-on `MlsKeyPackageConsumeStoreAdapter` now persists one-time KeyPackage consumption as an opaque digest-only record before Welcome sending. Production MLS provider integration should verify real MLS KeyPackages, feed accepted admissions into that consume store, then feed accepted consume-store output into the durable Welcome-send pipeline.
