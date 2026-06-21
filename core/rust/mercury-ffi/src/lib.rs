use std::{ffi::c_char, str};

pub const MERCURY_FFI_ABI_VERSION: u32 = 1;
pub const MERCURY_FFI_OK: i32 = 0;
pub const MERCURY_FFI_OUTPUT_POINTER_NULL: i32 = 1;
pub const MERCURY_FFI_INPUT_POINTER_NULL: i32 = 2;
pub const MERCURY_FFI_INVALID_UTF8: i32 = 3;
pub const MERCURY_FFI_INVALID_JSON: i32 = 4;
/// An internal panic was CAUGHT at the boundary (it must never unwind into the host = UB). The call
/// produced no output; the host should treat it as a hard internal error.
pub const MERCURY_FFI_INTERNAL_PANIC: i32 = 5;

const STATUS_OK: &[u8] = b"OK\0";
const STATUS_OUTPUT_POINTER_NULL: &[u8] = b"OUTPUT_POINTER_NULL\0";
const STATUS_INPUT_POINTER_NULL: &[u8] = b"INPUT_POINTER_NULL\0";
const STATUS_INVALID_UTF8: &[u8] = b"INVALID_UTF8\0";
const STATUS_INVALID_JSON: &[u8] = b"INVALID_JSON\0";
const STATUS_INTERNAL_PANIC: &[u8] = b"INTERNAL_PANIC\0";
const STATUS_UNKNOWN: &[u8] = b"UNKNOWN_STATUS\0";

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MercuryFfiBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn mercury_ffi_abi_version() -> u32 {
    MERCURY_FFI_ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn mercury_ffi_status_label(status: i32) -> *const c_char {
    match status {
        MERCURY_FFI_OK => STATUS_OK.as_ptr(),
        MERCURY_FFI_OUTPUT_POINTER_NULL => STATUS_OUTPUT_POINTER_NULL.as_ptr(),
        MERCURY_FFI_INPUT_POINTER_NULL => STATUS_INPUT_POINTER_NULL.as_ptr(),
        MERCURY_FFI_INVALID_UTF8 => STATUS_INVALID_UTF8.as_ptr(),
        MERCURY_FFI_INVALID_JSON => STATUS_INVALID_JSON.as_ptr(),
        MERCURY_FFI_INTERNAL_PANIC => STATUS_INTERNAL_PANIC.as_ptr(),
        _ => STATUS_UNKNOWN.as_ptr(),
    }
    .cast()
}

/// Handles one JSON platform-bridge request and returns an owned JSON buffer.
///
/// The returned buffer must be released with `mercury_ffi_free_buffer`.
///
/// # Safety
///
/// - `input_ptr` must either be null (with `input_len` == 0) or valid for reads of `input_len` bytes.
/// - `output` must be non-null and valid for writing one [`MercuryFfiBuffer`].
/// - On `MERCURY_FFI_OK`, `output` receives a buffer the caller must release exactly once with
///   [`mercury_ffi_free_buffer`]. On any other status code, `output` is left untouched.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_ffi_handle_bridge_request(
    input_ptr: *const u8,
    input_len: usize,
    output: *mut MercuryFfiBuffer,
) -> i32 {
    if output.is_null() {
        return MERCURY_FFI_OUTPUT_POINTER_NULL;
    }

    let input = match ffi_input(input_ptr, input_len) {
        Ok(input) => input,
        Err(status) => return status,
    };
    let request = match str::from_utf8(input) {
        Ok(request) => request,
        Err(_) => return MERCURY_FFI_INVALID_UTF8,
    };
    // PANIC FIREWALL: a panic must NEVER unwind across this `extern "C"` boundary into the host
    // (Swift/Kotlin/Tauri) — that is undefined behavior. `guarded` catches it and converts it into a
    // defined status code instead.
    let response = match guarded(|| mercury_bindings::platform_bridge_handle_json(request)) {
        Ok(response) => response,
        Err(status) => return status,
    };

    unsafe {
        output.write(owned_buffer(response));
    }

    MERCURY_FFI_OK
}

/// Releases a buffer previously returned by [`mercury_ffi_handle_bridge_request`].
///
/// # Safety
///
/// - `buffer` must be a buffer produced by [`mercury_ffi_handle_bridge_request`] and not yet freed,
///   and must not be used after this call. A zeroed buffer (`{ ptr: null, len: 0 }`) is a safe no-op.
///   Passing any other forged or already-freed buffer is undefined behavior.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_ffi_free_buffer(buffer: MercuryFfiBuffer) {
    if buffer.ptr.is_null() || buffer.len == 0 {
        return;
    }

    unsafe {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
            buffer.ptr, buffer.len,
        )));
    }
}

fn ffi_input<'a>(input_ptr: *const u8, input_len: usize) -> Result<&'a [u8], i32> {
    if input_len == 0 {
        return Ok(&[]);
    }
    if input_ptr.is_null() {
        return Err(MERCURY_FFI_INPUT_POINTER_NULL);
    }

    Ok(unsafe { std::slice::from_raw_parts(input_ptr, input_len) })
}

fn owned_buffer(response: String) -> MercuryFfiBuffer {
    let mut boxed = response.into_bytes().into_boxed_slice();
    let ptr = boxed.as_mut_ptr();
    let len = boxed.len();
    std::mem::forget(boxed);
    MercuryFfiBuffer { ptr, len }
}

/// Runs the bridge handler behind a PANIC FIREWALL and maps the outcome to an FFI status.
///
/// A panic must never unwind across the `extern "C"` boundary into the host (Swift/Kotlin/Tauri) —
/// that is undefined behavior. `catch_unwind` converts a panic into `MERCURY_FFI_INTERNAL_PANIC`
/// instead. `AssertUnwindSafe` is sound here: each call is independent + stateless, so a caught panic
/// tears no shared state. Generic over the handler so the three arms are unit-testable with stubs —
/// the real handler currently never panics on attacker input, and this guards that invariant against
/// silently breaking during future development.
fn guarded<F, E>(handler: F) -> Result<String, i32>
where
    F: FnOnce() -> Result<String, E>,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(handler)) {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(_)) => Err(MERCURY_FFI_INVALID_JSON),
        Err(_) => Err(MERCURY_FFI_INTERNAL_PANIC),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::CStr, slice};

    #[test]
    fn ffi_reports_abi_version_and_status_labels() {
        assert_eq!(mercury_ffi_abi_version(), 1);

        let label = unsafe { CStr::from_ptr(mercury_ffi_status_label(MERCURY_FFI_INVALID_JSON)) };
        assert_eq!(label.to_str().unwrap(), "INVALID_JSON");

        let unknown = unsafe { CStr::from_ptr(mercury_ffi_status_label(999)) };
        assert_eq!(unknown.to_str().unwrap(), "UNKNOWN_STATUS");
    }

    #[test]
    fn ffi_header_exports_matching_symbols_and_constants() {
        let header = include_str!("../include/mercury_ffi.h");

        assert!(header.contains("#define MERCURY_FFI_ABI_VERSION 1u"));
        assert!(header.contains("#define MERCURY_FFI_INVALID_JSON 4"));
        assert!(header.contains("typedef struct MercuryFfiBuffer"));
        assert!(header.contains("uint32_t mercury_ffi_abi_version(void);"));
        assert!(header.contains("const char *mercury_ffi_status_label(int32_t status);"));
        assert!(header.contains("int32_t mercury_ffi_handle_bridge_request("));
        assert!(header.contains("void mercury_ffi_free_buffer(MercuryFfiBuffer buffer);"));
    }

    #[test]
    fn ffi_handles_platform_bridge_request() {
        let request = br#"{
            "request_id": "0123456789abcdef0123456789abcdef",
            "operation": "prototype_fixture",
            "target": "account_recovery_high_entropy_ready"
        }"#;
        let mut output = MercuryFfiBuffer::default();

        let status = unsafe {
            mercury_ffi_handle_bridge_request(request.as_ptr(), request.len(), &mut output)
        };

        assert_eq!(status, MERCURY_FFI_OK);
        assert!(!output.ptr.is_null());
        assert!(output.len > 0);

        let response = unsafe { slice::from_raw_parts(output.ptr, output.len) };
        let value: serde_json::Value = serde_json::from_slice(response).unwrap();

        assert_eq!(value["bridge"]["accepted"], true);
        assert_eq!(value["body"]["surface"], "account_recovery");
        assert_eq!(value["body"]["decision"]["can_start_recovery"], true);

        unsafe {
            mercury_ffi_free_buffer(output);
        }
    }

    #[test]
    fn ffi_rejects_transport_level_bad_inputs() {
        let request = br#"{"request_id":"0123456789abcdef0123456789abcdef"}"#;
        let mut output = MercuryFfiBuffer::default();

        let null_output_status = unsafe {
            mercury_ffi_handle_bridge_request(request.as_ptr(), request.len(), std::ptr::null_mut())
        };
        assert_eq!(null_output_status, MERCURY_FFI_OUTPUT_POINTER_NULL);

        let null_input_status = unsafe {
            mercury_ffi_handle_bridge_request(std::ptr::null(), request.len(), &mut output)
        };
        assert_eq!(null_input_status, MERCURY_FFI_INPUT_POINTER_NULL);

        let bad_utf8 = [0xff, 0xfe];
        let bad_utf8_status = unsafe {
            mercury_ffi_handle_bridge_request(bad_utf8.as_ptr(), bad_utf8.len(), &mut output)
        };
        assert_eq!(bad_utf8_status, MERCURY_FFI_INVALID_UTF8);

        let invalid_json = b"{";
        let bad_json_status = unsafe {
            mercury_ffi_handle_bridge_request(
                invalid_json.as_ptr(),
                invalid_json.len(),
                &mut output,
            )
        };
        assert_eq!(bad_json_status, MERCURY_FFI_INVALID_JSON);
    }

    #[test]
    fn ffi_keeps_bridge_policy_rejections_inside_json_contract() {
        let request = br#"{
            "request_id": "0123456789abcdef0123456789abcdef",
            "operation": "prototype_fixture",
            "target": "account_recovery_high_entropy_ready",
            "plaintext_payload_len": 1
        }"#;
        let mut output = MercuryFfiBuffer::default();

        let status = unsafe {
            mercury_ffi_handle_bridge_request(request.as_ptr(), request.len(), &mut output)
        };

        assert_eq!(status, MERCURY_FFI_OK);

        let response = unsafe { slice::from_raw_parts(output.ptr, output.len) };
        let value: serde_json::Value = serde_json::from_slice(response).unwrap();

        assert_eq!(value["bridge"]["accepted"], false);
        assert_eq!(
            value["bridge"]["reason_label"],
            "plaintext_payload_forbidden"
        );
        assert!(value["body"].is_null());

        unsafe {
            mercury_ffi_free_buffer(output);
        }
    }

    #[test]
    fn the_firewall_maps_a_handler_panic_to_a_defined_status_not_an_unwind() {
        // The whole point of the firewall: a panicking handler must yield INTERNAL_PANIC — caught and
        // converted, NEVER unwound across the C boundary (UB). The real handler can't be made to panic
        // on attacker input, so we drive the firewall directly with a panicking stub.
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {})); // silence the expected panic's default stderr print
        let status = guarded(|| -> Result<String, ()> { panic!("simulated handler panic") });
        std::panic::set_hook(prev);
        assert_eq!(status, Err(MERCURY_FFI_INTERNAL_PANIC));
    }

    #[test]
    fn the_firewall_passes_a_handler_error_through_as_invalid_json() {
        let status = guarded(|| -> Result<String, ()> { Err(()) });
        assert_eq!(status, Err(MERCURY_FFI_INVALID_JSON));
    }

    #[test]
    fn the_firewall_passes_a_handler_success_through_unchanged() {
        let status = guarded(|| Ok::<_, ()>("ok-body".to_string()));
        assert_eq!(status, Ok("ok-body".to_string()));
    }
}
