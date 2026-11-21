// auto-ffi-c provider parity1093_c/1 target=x86_64-pc-windows-msvc
//
// PLAN-1093 canonical C provider: `c_strlen` — the AV side passes the
// borrowed view as the (data, len) pair; the REAL `<string.h>` strlen
// lives in the SDK-include unit (`parity1093_c_impl.c`) linked beside
// this freestanding provider object.

// The frozen ViewText pair types (freestanding provider — no system
// headers).
typedef unsigned char uint8_t;
typedef unsigned long long uint64_t;

// auto-ffi-c fn c_strlen form=pure ret=i32
int c_strlen(const uint8_t* data, uint64_t len);

int __c_strlen_impl(const uint8_t* data, uint64_t len);
