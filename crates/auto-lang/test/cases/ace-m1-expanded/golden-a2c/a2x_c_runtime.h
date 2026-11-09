// AUTO-GENERATED minimal typed runtime (PLAN-921 CA-A) — DO NOT EDIT.
// Checked i32 primitives with named traps + authorized trace marks.
#ifndef A2X_C_RUNTIME_H
#define A2X_C_RUNTIME_H

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

typedef enum {
    A2X_TRAP_NONE = 0,
    A2X_TRAP_OVERFLOW,
    A2X_TRAP_DIV_BY_ZERO,
    A2X_TRAP_DIV_OVERFLOW,
    A2X_TRAP_REM_BY_ZERO,
    A2X_TRAP_REM_OVERFLOW,
    A2X_TRAP_INVALID
} a2x_trap_code;

typedef struct {
    int32_t value;
    a2x_trap_code trap;
} a2x_result;

static inline a2x_result a2x_value(int32_t v) {
    a2x_result r;
    r.value = v;
    r.trap = A2X_TRAP_NONE;
    return r;
}

static inline const char *a2x_trap_name(a2x_trap_code t) {
    static const char *const names[7] = {
        "None", "Overflow", "DivByZero", "DivOverflow",
        "RemByZero", "RemOverflow", "InvalidScalar"
    };
    return names[(int)t];
}

/* Checked i32: compute in int64_t, range-check, THEN narrow — C signed
 * overflow is never executed (PLAN-921 §5.1). */
static inline a2x_result a2x_add(int32_t a, int32_t b) {
    int64_t s = (int64_t)a + (int64_t)b;
    if (s < INT32_MIN || s > INT32_MAX) {
        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;
    }
    return a2x_value((int32_t)s);
}

static inline a2x_result a2x_sub(int32_t a, int32_t b) {
    int64_t s = (int64_t)a - (int64_t)b;
    if (s < INT32_MIN || s > INT32_MAX) {
        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;
    }
    return a2x_value((int32_t)s);
}

static inline a2x_result a2x_mul(int32_t a, int32_t b) {
    int64_t s = (int64_t)a * (int64_t)b;
    if (s < INT32_MIN || s > INT32_MAX) {
        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;
    }
    return a2x_value((int32_t)s);
}

static inline a2x_result a2x_neg(int32_t a) {
    if (a == INT32_MIN) {
        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;
    }
    return a2x_value((int32_t)(-(int64_t)a));
}

/* div/rem: reject zero first, then INT32_MIN / -1; C truncation toward
 * zero preserves -7/3 == -2 and -7%3 == -1. */
static inline a2x_result a2x_div(int32_t a, int32_t b) {
    a2x_result r;
    if (b == 0) { r.value = 0; r.trap = A2X_TRAP_DIV_BY_ZERO; return r; }
    if (a == INT32_MIN && b == -1) { r.value = 0; r.trap = A2X_TRAP_DIV_OVERFLOW; return r; }
    return a2x_value(a / b);
}

static inline a2x_result a2x_rem(int32_t a, int32_t b) {
    a2x_result r;
    if (b == 0) { r.value = 0; r.trap = A2X_TRAP_REM_BY_ZERO; return r; }
    if (a == INT32_MIN && b == -1) { r.value = 0; r.trap = A2X_TRAP_REM_OVERFLOW; return r; }
    return a2x_value(a % b);
}

#include <intrin.h>
#include <string.h>
#include <math.h>
#include <float.h>

typedef struct { int64_t value; a2x_trap_code trap; } a2x_result_i64;
typedef struct { uint64_t value; a2x_trap_code trap; } a2x_result_u64;
typedef struct { float value; a2x_trap_code trap; } a2x_result_f32;
typedef struct { double value; a2x_trap_code trap; } a2x_result_f64;
#define A2X_OK_I(v) { (int64_t)(v), A2X_TRAP_NONE }
#define A2X_TRAP_I(t) { 0, (t) }
#define A2X_OK_U(v) { (uint64_t)(v), A2X_TRAP_NONE }
#define A2X_TRAP_U(t) { 0, (t) }
#define A2X_OK_F32(v) { (float)(v), A2X_TRAP_NONE }
#define A2X_OK_F64(v) { (double)(v), A2X_TRAP_NONE }

static inline a2x_result_i64 a2x_add_i8(int64_t a, int64_t b) {
    if ((b > 0 && a > INT64_MAX - b) || (b < 0 && a < INT64_MIN - b)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    int64_t v = (int64_t)(a + b);
    if (v < (-128LL) || v > (127LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_sub_i8(int64_t a, int64_t b) {
    if ((b < 0 && a > INT64_MAX + b) || (b > 0 && a < INT64_MIN + b)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    int64_t v = (int64_t)(a - b);
    if (v < (-128LL) || v > (127LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_mul_i8(int64_t a, int64_t b) {
    int64_t v = (int64_t)(a * b);
    if (v < (-128LL) || v > (127LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_div_i8(int64_t a, int64_t b) {
    if (b == 0) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_BY_ZERO); }
    if (a == (int64_t)(-128LL) && b == -1) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_OVERFLOW); }
    int64_t v = (int64_t)(a / b);
    if (v < (-128LL) || v > (127LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_rem_i8(int64_t a, int64_t b) {
    if (b == 0) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_BY_ZERO); }
    if (a == (int64_t)(-128LL) && b == -1) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_REM_OVERFLOW); }
    int64_t v = (int64_t)(a % b);
    if (v < (-128LL) || v > (127LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_neg_i8(int64_t a) {
    if (a == (int64_t)(-128LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(-a);
}

static inline a2x_result_i64 a2x_and_i8(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a & b);
}

static inline a2x_result_i64 a2x_or_i8(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a | b);
}

static inline a2x_result_i64 a2x_xor_i8(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a ^ b);
}

static inline a2x_result_i64 a2x_shl_i8(int64_t a, int64_t n) {
    if (n < 0 || n >= 8) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID); }
    return (a2x_result_i64)A2X_OK_I((int64_t)((uint64_t)a << (int)n));
}

static inline a2x_result_i64 a2x_shr_i8(int64_t a, int64_t n) {
    if (n < 0 || n >= 8) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID); }
    return (a2x_result_i64)A2X_OK_I(a >> (int)n);
}

static inline a2x_result_u64 a2x_add_u8(uint64_t a, uint64_t b) {
    if (b != 0 && a > UINT64_MAX - b) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    uint64_t v = (uint64_t)(a + b);
    if (v > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_sub_u8(uint64_t a, uint64_t b) {
    if (b != 0 && a < b) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    uint64_t v = (uint64_t)(a - b);
    if (v > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_mul_u8(uint64_t a, uint64_t b) {
    uint64_t v = (uint64_t)(a * b);
    if (v > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_div_u8(uint64_t a, uint64_t b) {
    if (b == 0) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_DIV_BY_ZERO); }
    uint64_t v = (uint64_t)(a / b);
    if (v > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_rem_u8(uint64_t a, uint64_t b) {
    if (b == 0) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_DIV_BY_ZERO); }
    uint64_t v = (uint64_t)(a % b);
    if (v > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_and_u8(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a & b);
}

static inline a2x_result_u64 a2x_or_u8(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a | b);
}

static inline a2x_result_u64 a2x_xor_u8(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a ^ b);
}

static inline a2x_result_u64 a2x_shl_u8(uint64_t a, uint64_t n) {
    if (n >= 8) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID); }
    return (a2x_result_u64)A2X_OK_U(a << (int)n);
}

static inline a2x_result_u64 a2x_shr_u8(uint64_t a, uint64_t n) {
    if (n >= 8) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID); }
    return (a2x_result_u64)A2X_OK_U(a >> (int)n);
}

static inline a2x_result_i64 a2x_add_i32(int64_t a, int64_t b) {
    if ((b > 0 && a > INT64_MAX - b) || (b < 0 && a < INT64_MIN - b)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    int64_t v = (int64_t)(a + b);
    if (v < (-2147483648LL) || v > (2147483647LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_sub_i32(int64_t a, int64_t b) {
    if ((b < 0 && a > INT64_MAX + b) || (b > 0 && a < INT64_MIN + b)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    int64_t v = (int64_t)(a - b);
    if (v < (-2147483648LL) || v > (2147483647LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_mul_i32(int64_t a, int64_t b) {
    int64_t v = (int64_t)(a * b);
    if (v < (-2147483648LL) || v > (2147483647LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_div_i32(int64_t a, int64_t b) {
    if (b == 0) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_BY_ZERO); }
    if (a == (int64_t)(-2147483648LL) && b == -1) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_OVERFLOW); }
    int64_t v = (int64_t)(a / b);
    if (v < (-2147483648LL) || v > (2147483647LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_rem_i32(int64_t a, int64_t b) {
    if (b == 0) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_BY_ZERO); }
    if (a == (int64_t)(-2147483648LL) && b == -1) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_REM_OVERFLOW); }
    int64_t v = (int64_t)(a % b);
    if (v < (-2147483648LL) || v > (2147483647LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_neg_i32(int64_t a) {
    if (a == (int64_t)(-2147483648LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(-a);
}

static inline a2x_result_i64 a2x_and_i32(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a & b);
}

static inline a2x_result_i64 a2x_or_i32(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a | b);
}

static inline a2x_result_i64 a2x_xor_i32(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a ^ b);
}

static inline a2x_result_i64 a2x_shl_i32(int64_t a, int64_t n) {
    if (n < 0 || n >= 32) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID); }
    return (a2x_result_i64)A2X_OK_I((int64_t)((uint64_t)a << (int)n));
}

static inline a2x_result_i64 a2x_shr_i32(int64_t a, int64_t n) {
    if (n < 0 || n >= 32) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID); }
    return (a2x_result_i64)A2X_OK_I(a >> (int)n);
}

static inline a2x_result_u64 a2x_add_u32(uint64_t a, uint64_t b) {
    if (b != 0 && a > UINT64_MAX - b) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    uint64_t v = (uint64_t)(a + b);
    if (v > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_sub_u32(uint64_t a, uint64_t b) {
    if (b != 0 && a < b) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    uint64_t v = (uint64_t)(a - b);
    if (v > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_mul_u32(uint64_t a, uint64_t b) {
    uint64_t v = (uint64_t)(a * b);
    if (v > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_div_u32(uint64_t a, uint64_t b) {
    if (b == 0) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_DIV_BY_ZERO); }
    uint64_t v = (uint64_t)(a / b);
    if (v > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_rem_u32(uint64_t a, uint64_t b) {
    if (b == 0) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_DIV_BY_ZERO); }
    uint64_t v = (uint64_t)(a % b);
    if (v > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_and_u32(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a & b);
}

static inline a2x_result_u64 a2x_or_u32(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a | b);
}

static inline a2x_result_u64 a2x_xor_u32(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a ^ b);
}

static inline a2x_result_u64 a2x_shl_u32(uint64_t a, uint64_t n) {
    if (n >= 32) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID); }
    return (a2x_result_u64)A2X_OK_U(a << (int)n);
}

static inline a2x_result_u64 a2x_shr_u32(uint64_t a, uint64_t n) {
    if (n >= 32) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID); }
    return (a2x_result_u64)A2X_OK_U(a >> (int)n);
}

static inline a2x_result_i64 a2x_add_i64(int64_t a, int64_t b) {
    if ((b > 0 && a > INT64_MAX - b) || (b < 0 && a < INT64_MIN - b)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    int64_t v = (int64_t)(a + b);
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_sub_i64(int64_t a, int64_t b) {
    if ((b < 0 && a > INT64_MAX + b) || (b > 0 && a < INT64_MIN + b)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    int64_t v = (int64_t)(a - b);
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_mul_i64(int64_t a, int64_t b) {
    int64_t lo64; int64_t hi64 = _mul128(a, b, &lo64);
    if (hi64 != lo64 >> 63) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(lo64);
}

static inline a2x_result_i64 a2x_div_i64(int64_t a, int64_t b) {
    if (b == 0) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_BY_ZERO); }
    if (a == (int64_t)(-9223372036854775808LL) && b == -1) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_OVERFLOW); }
    int64_t v = (int64_t)(a / b);
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_rem_i64(int64_t a, int64_t b) {
    if (b == 0) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_DIV_BY_ZERO); }
    if (a == (int64_t)(-9223372036854775808LL) && b == -1) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_REM_OVERFLOW); }
    int64_t v = (int64_t)(a % b);
    return (a2x_result_i64)A2X_OK_I(v);
}

static inline a2x_result_i64 a2x_neg_i64(int64_t a) {
    if (a == (int64_t)(-9223372036854775808LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I(-a);
}

static inline a2x_result_i64 a2x_and_i64(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a & b);
}

static inline a2x_result_i64 a2x_or_i64(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a | b);
}

static inline a2x_result_i64 a2x_xor_i64(int64_t a, int64_t b) {
    return (a2x_result_i64)A2X_OK_I(a ^ b);
}

static inline a2x_result_i64 a2x_shl_i64(int64_t a, int64_t n) {
    if (n < 0 || n >= 64) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID); }
    return (a2x_result_i64)A2X_OK_I((int64_t)((uint64_t)a << (int)n));
}

static inline a2x_result_i64 a2x_shr_i64(int64_t a, int64_t n) {
    if (n < 0 || n >= 64) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID); }
    return (a2x_result_i64)A2X_OK_I(a >> (int)n);
}

static inline a2x_result_u64 a2x_add_u64(uint64_t a, uint64_t b) {
    if (b != 0 && a > UINT64_MAX - b) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    uint64_t v = (uint64_t)(a + b);
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_sub_u64(uint64_t a, uint64_t b) {
    if (b != 0 && a < b) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    uint64_t v = (uint64_t)(a - b);
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_mul_u64(uint64_t a, uint64_t b) {
    uint64_t lo64; uint64_t hi64 = _umul128(a, b, &lo64);
    if (hi64 != 0) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U(lo64);
}

static inline a2x_result_u64 a2x_div_u64(uint64_t a, uint64_t b) {
    if (b == 0) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_DIV_BY_ZERO); }
    uint64_t v = (uint64_t)(a / b);
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_rem_u64(uint64_t a, uint64_t b) {
    if (b == 0) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_DIV_BY_ZERO); }
    uint64_t v = (uint64_t)(a % b);
    return (a2x_result_u64)A2X_OK_U(v);
}

static inline a2x_result_u64 a2x_and_u64(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a & b);
}

static inline a2x_result_u64 a2x_or_u64(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a | b);
}

static inline a2x_result_u64 a2x_xor_u64(uint64_t a, uint64_t b) {
    return (a2x_result_u64)A2X_OK_U(a ^ b);
}

static inline a2x_result_u64 a2x_shl_u64(uint64_t a, uint64_t n) {
    if (n >= 64) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID); }
    return (a2x_result_u64)A2X_OK_U(a << (int)n);
}

static inline a2x_result_u64 a2x_shr_u64(uint64_t a, uint64_t n) {
    if (n >= 64) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID); }
    return (a2x_result_u64)A2X_OK_U(a >> (int)n);
}

static inline a2x_result_f32 a2x_div_f32(float a, float b) {
    if (b == 0.0f) { return (a2x_result_f32){ 0, A2X_TRAP_DIV_BY_ZERO }; }
    return (a2x_result_f32)A2X_OK_F32(a / b);
}

static inline a2x_result_f64 a2x_div_f64(double a, double b) {
    if (b == 0.0) { return (a2x_result_f64){ 0, A2X_TRAP_DIV_BY_ZERO }; }
    return (a2x_result_f64)A2X_OK_F64(a / b);
}

static inline a2x_result_f32 a2x_rem_f32(float a, float b) {
    return (a2x_result_f32)A2X_OK_F32(fmodf(a, b));
}

static inline a2x_result_f64 a2x_rem_f64(double a, double b) {
    return (a2x_result_f64)A2X_OK_F64(fmod(a, b));
}

static inline a2x_result a2x_c_i8_i32(int64_t a) {
    if (a < (-2147483648LL) || a > (2147483647LL)) { return (a2x_result){ 0, A2X_TRAP_OVERFLOW }; }
    return a2x_value((int32_t)a);
}

static inline a2x_result_i64 a2x_c_i8_i64(int64_t a) {
    if (a < (-9223372036854775808LL) || a > (9223372036854775807LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result_u64 a2x_c_i8_u8(int64_t a) {
    if (a > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_i8_u32(int64_t a) {
    if (a > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_i8_u64(int64_t a) {
    if (a > (18446744073709551615ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_i64 a2x_c_i32_i8(int64_t a) {
    if (a < (-128LL) || a > (127LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result_i64 a2x_c_i32_i64(int64_t a) {
    if (a < (-9223372036854775808LL) || a > (9223372036854775807LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result_u64 a2x_c_i32_u8(int64_t a) {
    if (a > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_i32_u32(int64_t a) {
    if (a > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_i32_u64(int64_t a) {
    if (a > (18446744073709551615ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_i64 a2x_c_i64_i8(int64_t a) {
    if (a < (-128LL) || a > (127LL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result a2x_c_i64_i32(int64_t a) {
    if (a < (-2147483648LL) || a > (2147483647LL)) { return (a2x_result){ 0, A2X_TRAP_OVERFLOW }; }
    return a2x_value((int32_t)a);
}

static inline a2x_result_u64 a2x_c_i64_u8(int64_t a) {
    if (a > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_i64_u32(int64_t a) {
    if (a > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_i64_u64(int64_t a) {
    if (a > (18446744073709551615ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_i64 a2x_c_u8_i8(uint64_t a) {
    if (a > (127ULL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result a2x_c_u8_i32(uint64_t a) {
    if (a > (2147483647ULL)) { return (a2x_result){ 0, A2X_TRAP_OVERFLOW }; }
    return a2x_value((int32_t)a);
}

static inline a2x_result_i64 a2x_c_u8_i64(uint64_t a) {
    if (a > (9223372036854775807ULL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result_u64 a2x_c_u8_u32(uint64_t a) {
    if (a > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_u8_u64(uint64_t a) {
    if (a > (18446744073709551615ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_i64 a2x_c_u32_i8(uint64_t a) {
    if (a > (127ULL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result a2x_c_u32_i32(uint64_t a) {
    if (a > (2147483647ULL)) { return (a2x_result){ 0, A2X_TRAP_OVERFLOW }; }
    return a2x_value((int32_t)a);
}

static inline a2x_result_i64 a2x_c_u32_i64(uint64_t a) {
    if (a > (9223372036854775807ULL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result_u64 a2x_c_u32_u8(uint64_t a) {
    if (a > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_u32_u64(uint64_t a) {
    if (a > (18446744073709551615ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_i64 a2x_c_u64_i8(uint64_t a) {
    if (a > (127ULL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result a2x_c_u64_i32(uint64_t a) {
    if (a > (2147483647ULL)) { return (a2x_result){ 0, A2X_TRAP_OVERFLOW }; }
    return a2x_value((int32_t)a);
}

static inline a2x_result_i64 a2x_c_u64_i64(uint64_t a) {
    if (a > (9223372036854775807ULL)) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
}

static inline a2x_result_u64 a2x_c_u64_u8(uint64_t a) {
    if (a > (255ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_u64 a2x_c_u64_u32(uint64_t a) {
    if (a > (4294967295ULL)) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
}

static inline a2x_result_f32 a2x_c_i8_f32(int64_t a) {
    return (a2x_result_f32)A2X_OK_F32((float)a);
}

static inline a2x_result_f64 a2x_c_i8_f64(int64_t a) {
    return (a2x_result_f64)A2X_OK_F64((double)a);
}

static inline a2x_result_f32 a2x_c_i32_f32(int64_t a) {
    return (a2x_result_f32)A2X_OK_F32((float)a);
}

static inline a2x_result_f64 a2x_c_i32_f64(int64_t a) {
    return (a2x_result_f64)A2X_OK_F64((double)a);
}

static inline a2x_result_f32 a2x_c_i64_f32(int64_t a) {
    return (a2x_result_f32)A2X_OK_F32((float)a);
}

static inline a2x_result_f64 a2x_c_i64_f64(int64_t a) {
    return (a2x_result_f64)A2X_OK_F64((double)a);
}

static inline a2x_result_f32 a2x_c_u8_f32(uint64_t a) {
    return (a2x_result_f32)A2X_OK_F32((float)a);
}

static inline a2x_result_f64 a2x_c_u8_f64(uint64_t a) {
    return (a2x_result_f64)A2X_OK_F64((double)a);
}

static inline a2x_result_f32 a2x_c_u32_f32(uint64_t a) {
    return (a2x_result_f32)A2X_OK_F32((float)a);
}

static inline a2x_result_f64 a2x_c_u32_f64(uint64_t a) {
    return (a2x_result_f64)A2X_OK_F64((double)a);
}

static inline a2x_result_f32 a2x_c_u64_f32(uint64_t a) {
    return (a2x_result_f32)A2X_OK_F32((float)a);
}

static inline a2x_result_f64 a2x_c_u64_f64(uint64_t a) {
    return (a2x_result_f64)A2X_OK_F64((double)a);
}

static inline a2x_result_i64 a2x_c_f32_i8(float a) {
    if (!(a == a)) { goto bad; }
    if (a < -128.0 || a >= 128.0) { goto bad; }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
bad: return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID);
}

static inline a2x_result a2x_c_f32_i32(float a) {
    if (!(a == a)) { goto bad; }
    if (a < -2147483648.0 || a >= 2147483648.0) { goto bad; }
    return a2x_value((int32_t)a);
bad: return (a2x_result){ 0, A2X_TRAP_INVALID };
}

static inline a2x_result_i64 a2x_c_f32_i64(float a) {
    if (!(a == a)) { goto bad; }
    if (a < -9223372036854775808.0 || a >= 9223372036854775808.0) { goto bad; }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
bad: return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID);
}

static inline a2x_result_u64 a2x_c_f32_u8(float a) {
    if (!(a == a)) { goto bad; }
    if (a < 0.0 || a >= 255.0) { goto bad; }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
bad: return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID);
}

static inline a2x_result_u64 a2x_c_f32_u32(float a) {
    if (!(a == a)) { goto bad; }
    if (a < 0.0 || a >= 4294967295.0) { goto bad; }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
bad: return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID);
}

static inline a2x_result_u64 a2x_c_f32_u64(float a) {
    if (!(a == a)) { goto bad; }
    if (a < 0.0 || a >= 18446744073709551615.0) { goto bad; }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
bad: return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID);
}

static inline a2x_result_i64 a2x_c_f64_i8(double a) {
    if (!(a == a)) { goto bad; }
    if (a < -128.0 || a >= 128.0) { goto bad; }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
bad: return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID);
}

static inline a2x_result a2x_c_f64_i32(double a) {
    if (!(a == a)) { goto bad; }
    if (a < -2147483648.0 || a >= 2147483648.0) { goto bad; }
    return a2x_value((int32_t)a);
bad: return (a2x_result){ 0, A2X_TRAP_INVALID };
}

static inline a2x_result_i64 a2x_c_f64_i64(double a) {
    if (!(a == a)) { goto bad; }
    if (a < -9223372036854775808.0 || a >= 9223372036854775808.0) { goto bad; }
    return (a2x_result_i64)A2X_OK_I((int64_t)a);
bad: return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID);
}

static inline a2x_result_u64 a2x_c_f64_u8(double a) {
    if (!(a == a)) { goto bad; }
    if (a < 0.0 || a >= 255.0) { goto bad; }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
bad: return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID);
}

static inline a2x_result_u64 a2x_c_f64_u32(double a) {
    if (!(a == a)) { goto bad; }
    if (a < 0.0 || a >= 4294967295.0) { goto bad; }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
bad: return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID);
}

static inline a2x_result_u64 a2x_c_f64_u64(double a) {
    if (!(a == a)) { goto bad; }
    if (a < 0.0 || a >= 18446744073709551615.0) { goto bad; }
    return (a2x_result_u64)A2X_OK_U((uint64_t)a);
bad: return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID);
}

static inline a2x_result_f64 a2x_c_f32_f64(float a) {
    return (a2x_result_f64)A2X_OK_F64((double)a);
}

static inline a2x_result_f32 a2x_c_f64_f32(double a) {
    float r = (float)a;
    if ((r > FLT_MAX || r < -FLT_MAX)) {
        return (a2x_result_f32){ 0, A2X_TRAP_OVERFLOW };
    }
    return (a2x_result_f32)A2X_OK_F32(r);
}

static inline float a2x_f32_bits(uint32_t bits) {
    float f; memcpy(&f, &bits, sizeof f); return f;
}
static inline double a2x_f64_bits(uint64_t bits) {
    double d; memcpy(&d, &bits, sizeof d); return d;
}


#endif /* A2X_C_RUNTIME_H */
