#ifndef MERCURY_FFI_H
#define MERCURY_FFI_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MERCURY_FFI_ABI_VERSION 1u

#define MERCURY_FFI_OK 0
#define MERCURY_FFI_OUTPUT_POINTER_NULL 1
#define MERCURY_FFI_INPUT_POINTER_NULL 2
#define MERCURY_FFI_INVALID_UTF8 3
#define MERCURY_FFI_INVALID_JSON 4

typedef struct MercuryFfiBuffer {
    uint8_t *ptr;
    size_t len;
} MercuryFfiBuffer;

uint32_t mercury_ffi_abi_version(void);
const char *mercury_ffi_status_label(int32_t status);
int32_t mercury_ffi_handle_bridge_request(
    const uint8_t *input_ptr,
    size_t input_len,
    MercuryFfiBuffer *output
);
void mercury_ffi_free_buffer(MercuryFfiBuffer buffer);

#ifdef __cplusplus
}
#endif

#endif
