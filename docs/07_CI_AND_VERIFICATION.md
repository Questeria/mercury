# CI And Verification

Generated: 2026-05-27

## Status

Mercury now has a GitHub Actions workflow at `.github/workflows/ci.yml`.

The first CI job runs on `ubuntu-latest` and verifies:

- Rust formatting with `cargo fmt --check`.
- Rust policy and client-core integration tests with `cargo test --workspace`.
- Feature-gated decision-view serialization with `cargo test -p mercury-core --features serde decision_view_serializes_compact_fields`.
- Host-side JSON vector consistency with `python3 tools/check_envelope_vectors.py`.
- Host-side AI grant vector consistency with `python3 tools/check_ai_grant_vectors.py`.
- Host-side AI grant lifecycle vector consistency with `python3 tools/check_ai_grant_lifecycle_vectors.py`.
- Host-side room epoch vector consistency with `python3 tools/check_room_epoch_vectors.py`.
- Host-side policy pipeline vector consistency with `python3 tools/check_policy_pipeline_vectors.py`.
- Host-side relay submission vector consistency with `python3 tools/check_relay_submit_vectors.py`.
- Host-side platform decision vector consistency with `python3 tools/check_platform_decision_vectors.py`.
- Host-side outbound decision vector consistency with `python3 tools/check_outbound_decide_vectors.py`.
- Host-side receive decision vector consistency with `python3 tools/check_receive_decide_vectors.py`.
- Host-side bootstrap decision vector consistency with `python3 tools/check_bootstrap_decide_vectors.py`.
- Host-side inbound sync vector consistency with `python3 tools/check_inbound_sync_vectors.py`.
- Host-side account recovery vector consistency with `python3 tools/check_account_recovery_vectors.py`.
- Policy contract drift checks with `python3 tools/check_policy_contract.py`.
- Helix policy attestation drift checks with `python3 tools/gen_policy_attestations.py --check`.

## Local Windows Toolchain

On Windows, run full Rust tests from a Visual Studio Build Tools developer environment so `link.exe` is on PATH:

```powershell
cmd.exe /d /s /c '"C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64 >NUL && cargo test --workspace'
```

GitHub's Linux runner still provides the reproducible CI path for pushes and pull requests.

## Helix Checks

The pinned Helix compiler submodule is now the normal local and CI Helix lane:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_helix_checks.ps1
```

The Linux CI equivalent is:

```bash
bash tools/run_helix_checks.sh third_party/helix
```

That lane typechecks each policy, emits function hashes, emits proof obligations, compiles each Helix test to an ELF, and requires exit 42.

CI can only clone the private Helix submodule when `HELIX_CHECKOUT_TOKEN` has read access to both Mercury and Helix. Without that token, the Rust/Python policy job still gates the code and the Helix job reports the missing private dependency.

## Policy Attestation

Mercury also commits an attestation manifest:

```text
helix/policy/attestation.json
ui/app/src/mercury/policyAttestation.generated.ts
```

Regenerate it after changing Helix policy sources, Helix tests, JSON vectors, policy JSON files, or Rust mirror tests:

```powershell
python .\tools\gen_policy_attestations.py --write
```

The manifest records each policy's source hash, test hash, vector corpus hash, Rust mirror tests, required gates, optional from-raw gate, and production runtime boundary. The UI inspector consumes the generated TypeScript artifact so decision cards can show their Helix provenance.

## From-Raw Cross-Check

`C:\Projects\Helix` remains read-only context only. When present, run the optional from-raw K1 cross-check with:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_native_helix_checks.ps1
```

The wrapper invokes `tools/run_fromraw_helix_check.sh` through WSL. It copies `seed.bin` and `k1src.hx` into `/tmp`, builds K1 there, compiles the Mercury policy tests there, and writes nothing to Helix.
