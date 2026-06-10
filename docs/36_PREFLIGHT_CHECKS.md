# Preflight Checks

Generated: 2026-05-28

## Status

Mercury now has a repo-local preflight script:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The script runs the current non-UI safety bundle:

- Rust formatting
- Rust workspace tests
- Rust serde decision-view test
- Python policy contract checker
- Python vector checkers
- Helix policy checks and generated test binaries
- `git diff --check`
- ASCII scan over source, docs, fixtures, tools, policy, and vectors

It uses Visual Studio Build Tools automatically when the installed `VsDevCmd.bat` path exists. Pass `-SkipHelix` when Helix is not available locally.

## Intended Use

Run preflight before pushing changes that touch:

- Helix policy or vectors
- Rust core policy
- platform bindings
- store, relay, receive, bootstrap, or AI backend prototypes
- docs that should stay ASCII and whitespace-clean

CI still runs the portable subset on GitHub. The local preflight is broader because it can call the local Helix compiler checkout.

## Next Step

The crypto provider scaffolding is documented in `docs/37_CRYPTO_PROVIDER_SCAFFOLDING.md`. The next parallel increment should add fixture coverage for the store, relay, AI, and crypto prototypes so UI and platform agents can consume stable JSON states beyond the initial decision-view fixtures.
