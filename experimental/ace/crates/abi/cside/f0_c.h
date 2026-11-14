// f0_c.h — PLAN-967 T-10 frozen C provider header (the annotations ARE
// the import metadata; binding/shim/manifest derive from this file).
#ifndef F0_C_H
#define F0_C_H
#include <stdint.h>
#include <stdbool.h>
// A macro in DECLARATION position: only the REAL preprocessor folds it
// (text scanning cannot see this).
#define F0C_API extern
typedef struct f0c_point { int32_t x; int32_t y; } f0c_point;
typedef enum { F0C_OK = 0, F0C_EMPTY = -1 } f0c_status;
typedef int32_t (*f0c_binop)(int32_t a, int32_t b);
// auto-ffi-c fn f0c_scaled_len form=pure ret=i32
F0C_API int32_t f0c_scaled_len(const uint8_t* data, uint64_t len);
// auto-ffi-c fn f0c_point_sum form=pure ret=i32
F0C_API int32_t f0c_point_sum(const f0c_point* p);
// auto-ffi-c fn f0c_status_of form=fallible errors=empty:-1
F0C_API int32_t f0c_status_of(const uint8_t* data, uint64_t len, int32_t* out);
#endif
