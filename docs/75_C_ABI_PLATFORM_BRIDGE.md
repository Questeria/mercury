# C ABI Platform Bridge

Generated: 2026-05-28

## Status

Mercury now has a small C ABI crate for desktop and mobile shell integration:

```text
core/rust/mercury-ffi
core/rust/mercury-ffi/include/mercury_ffi.h
```

This does not choose a UI toolkit or platform package target. It exposes the existing JSON platform bridge through a pointer/length ABI so future Swift, Kotlin, C#, C++, Electron/Tauri, or native shells can call the same checked backend contract.

## Exported Functions

```c
uint32_t mercury_ffi_abi_version(void);
const char *mercury_ffi_status_label(int32_t status);
int32_t mercury_ffi_handle_bridge_request(
    const uint8_t *input_ptr,
    uintptr_t input_len,
    MercuryFfiBuffer *output
);
void mercury_ffi_free_buffer(MercuryFfiBuffer buffer);
```

`MercuryFfiBuffer` contains:

```c
uint8_t *ptr;
uintptr_t len;
```

The caller owns the returned buffer only after `MERCURY_FFI_OK` and must release it with `mercury_ffi_free_buffer`.

The checked C header exports the same constants, struct, and function symbols. Rust tests keep the header synchronized with the current ABI surface.

## Status Codes

```text
0 OK
1 OUTPUT_POINTER_NULL
2 INPUT_POINTER_NULL
3 INVALID_UTF8
4 INVALID_JSON
```

Transport and codec failures are FFI status codes. Security or routing decisions remain inside the JSON bridge contract as `bridge.accepted`, `reason_code`, and `reason_label`.

## Security Rules

- The ABI accepts UTF-8 JSON only.
- Raw plaintext payloads are not accepted by the bridge contract.
- The FFI layer does not expose secret pointers, decrypted message bytes, or mutable global state.
- UI shells should call `mercury_ffi_abi_version()` before dispatching requests.
- Every successful response buffer must be freed exactly once.
- Bridge decision booleans remain the authority for enabling UI actions.

## Verification

Run:

```powershell
cargo test -p mercury-ffi
cargo test --workspace
powershell -ExecutionPolicy Bypass -File .\tools\run_preflight.ps1
```

The focused tests cover ABI version/status labels, checked header exports, successful bridge dispatch, null pointer rejection, invalid UTF-8 rejection, invalid JSON rejection, owned-buffer release, and plaintext-payload rejection through the JSON bridge contract.

## Next Backend Step

Add platform package targets once the desktop/mobile client stack is selected.
