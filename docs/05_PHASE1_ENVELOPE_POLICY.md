# Phase 1 Envelope Policy

Generated: 2026-05-27

## Status

Mercury now has its first Helix-backed policy module:

- `helix/policy/envelope.hx`
- `helix/tests/envelope_test.hx`
- `policy/envelope_policy_v1.json`
- `vectors/envelope/*.json`
- `tools/run_helix_checks.ps1`

The module is intentionally scalar-only. It validates policy facts about an already-parsed encrypted envelope and returns deterministic reason codes. It does not parse JSON, inspect ciphertext, verify signatures, or implement cryptography.

## Why The API Is Staged

The current Helix backend codegen path supports a small integer-parameter budget. A single all-fields function typechecks, but codegen fails once it exceeds that budget. Mercury therefore uses three small staged validators:

```text
mercury_validate_identity_v1(version, suite_id, min_suite_id, conversation_id_len, sender_account_id_len, sender_device_id_len)
mercury_validate_order_v1(epoch, sequence, expected_epoch, expected_sequence)
mercury_validate_content_v1(message_kind, payload_len, critical_flags, noncritical_flags, max_payload_len)
mercury_first_reject(identity_reason, order_reason, content_reason)
```

This keeps the policy executable today while remaining easy to mirror in Rust.

## Reason Codes

```text
0  ACCEPT
1  BAD_VERSION
2  UNSUPPORTED_SUITE
3  DOWNGRADED_SUITE
4  BAD_CONVERSATION_ID_LEN
5  BAD_SENDER_ACCOUNT_ID_LEN
6  BAD_SENDER_DEVICE_ID_LEN
7  BAD_EPOCH
8  BAD_SEQUENCE
9  BAD_MESSAGE_KIND
10 PAYLOAD_TOO_LARGE
11 UNKNOWN_CRITICAL_FLAG
12 RESERVED_AI_KIND
```

## Audit Classes

```text
1 ACCEPTED_MESSAGE
2 POLICY_REJECT
3 DOWNGRADE_ATTEMPT
4 SIZE_REJECT
```

## Golden Vector Set

- `valid_minimal_application_v1`
- `valid_empty_payload_control_v1`
- `valid_unknown_noncritical_flag_v1`
- `reject_bad_version_v0`
- `reject_downgrade_classical_below_floor`
- `reject_unknown_suite`
- `reject_oversize_payload`
- `reject_unknown_critical_flag`
- `reject_bad_epoch`
- `reject_bad_sequence`
- `reject_empty_sender_device_id`
- `reject_reserved_ai_context_kind_phase1`

## Verification

Run:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
```

Verified locally:

- Policy parse/typecheck/totality: OK.
- Policy structural hashes: emitted.
- Policy proof-obligation JSON: emitted with zero obligations and zero errors.
- Test parse/typecheck/totality: OK.
- Test ELF codegen: OK.
- WSL runtime execution: `envelope_test.bin` exits `42`.
- Python vector runner: `python .\tools\check_envelope_vectors.py` checks all 12 JSON vectors.
- Policy contract checker: `python .\tools\check_policy_contract.py` verifies manifest drift across Helix, Rust, Python, and vectors.

The runtime exit code matters: it proves the Helix `main()` assertions executed, rather than only passing the checker.

## Cross-Language Mirror

The first Rust mirror lives in `core/rust/mercury-policy`. See `docs/06_RUST_POLICY_MIRROR.md`.
