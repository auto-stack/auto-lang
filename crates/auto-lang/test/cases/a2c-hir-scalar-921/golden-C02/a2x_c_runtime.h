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
    A2X_TRAP_REM_OVERFLOW
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
    static const char *const names[6] = {
        "None", "Overflow", "DivByZero", "DivOverflow",
        "RemByZero", "RemOverflow"
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

#endif /* A2X_C_RUNTIME_H */
