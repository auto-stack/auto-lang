// f0_c.h — PLAN-967 T-10 frozen C provider header (the annotations ARE
// the import metadata; binding/shim/manifest derive from this file).
// auto-ffi-c provider f0_c/1 target=x86_64-pc-windows-msvc
#ifndef F0_C_H
#define F0_C_H
#include <stdint.h>
#include <stdbool.h>
// A macro in DECLARATION position: only the REAL preprocessor folds it
// (text scanning cannot see this).
#define F0C_API extern
typedef struct f0c_point { int32_t x; int32_t y; } f0c_point;
typedef enum { F0C_OK = 0, F0C_EMPTY = -1, F0C_MAX = 41 } f0c_status;
typedef int32_t (*f0c_binop)(int32_t a, int32_t b);
// auto-ffi-c fn f0c_scaled_len form=pure ret=i32
F0C_API int32_t f0c_scaled_len(const uint8_t* data, uint64_t len);
// auto-ffi-c fn f0c_point_sum form=pure ret=i32
F0C_API int32_t f0c_point_sum(const f0c_point* p);
// auto-ffi-c fn f0c_status_of form=fallible errors=empty:-1
F0C_API int32_t f0c_status_of(const uint8_t* data, uint64_t len, int32_t* out);
// auto-ffi-c fn f0c_constants_probe form=pure ret=i32
F0C_API int32_t f0c_constants_probe(void);
// auto-ffi-c fn f0c_inline_twice form=pure ret=i32
static inline int32_t f0c_inline_twice(int32_t v) { return v + v; }
#endif
