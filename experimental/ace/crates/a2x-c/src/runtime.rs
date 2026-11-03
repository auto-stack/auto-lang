//! PLAN-921 CA-A: the C11 runtime header text (a versioned text asset of
//! the provider, marks-parameterized like the Rust runtime).
//!
//! Semantics mirror the Rust runtime exactly (artifacts.rs): named trap
//! taxonomy `Overflow / DivByZero / DivOverflow / RemByZero /
//! RemOverflow`; checked add/sub/mul/neg compute in `int64_t` and
//! range-check BEFORE the narrowing conversion (C signed overflow is
//! never executed); div/rem reject zero first, then `INT32_MIN / -1`;
//! C's truncation toward zero preserves `-7/3 == -2` and `-7%3 == -1`.
//! Trace marks (`mark_a` → 1, `mark_b` → 2) print to stderr at mark time
//! — marks before a trap stay observable, mirroring the eprintln! order.
//!
//! Everything is `static inline` in ONE header (`a2x_c_runtime.h`)
//! included by both `main.c` and `generated.c` — no separate runtime
//! translation unit, no link-time surface.

/// The runtime header text for a view: base checked-i32 core + the mark
/// functions the view actually uses (content-hashed into every C fragment
/// key, so a runtime change can never hit stale fragments).
pub fn runtime_source(marks: &[&'static str]) -> String {
    let mut s = String::new();
    s.push_str("// AUTO-GENERATED minimal typed runtime (PLAN-921 CA-A) — DO NOT EDIT.\n");
    s.push_str("// Checked i32 primitives with named traps + authorized trace marks.\n");
    s.push_str("#ifndef A2X_C_RUNTIME_H\n#define A2X_C_RUNTIME_H\n\n");
    s.push_str("#include <stdbool.h>\n#include <stdint.h>\n#include <stdio.h>\n\n");
    s.push_str("typedef enum {\n");
    s.push_str("    A2X_TRAP_NONE = 0,\n");
    s.push_str("    A2X_TRAP_OVERFLOW,\n");
    s.push_str("    A2X_TRAP_DIV_BY_ZERO,\n");
    s.push_str("    A2X_TRAP_DIV_OVERFLOW,\n");
    s.push_str("    A2X_TRAP_REM_BY_ZERO,\n");
    s.push_str("    A2X_TRAP_REM_OVERFLOW\n");
    s.push_str("} a2x_trap_code;\n\n");
    s.push_str("typedef struct {\n");
    s.push_str("    int32_t value;\n");
    s.push_str("    a2x_trap_code trap;\n");
    s.push_str("} a2x_result;\n\n");
    s.push_str("static inline a2x_result a2x_value(int32_t v) {\n");
    s.push_str("    a2x_result r;\n");
    s.push_str("    r.value = v;\n");
    s.push_str("    r.trap = A2X_TRAP_NONE;\n");
    s.push_str("    return r;\n");
    s.push_str("}\n\n");
    s.push_str("static inline const char *a2x_trap_name(a2x_trap_code t) {\n");
    s.push_str("    static const char *const names[6] = {\n");
    s.push_str("        \"None\", \"Overflow\", \"DivByZero\", \"DivOverflow\",\n");
    s.push_str("        \"RemByZero\", \"RemOverflow\"\n");
    s.push_str("    };\n");
    s.push_str("    return names[(int)t];\n");
    s.push_str("}\n\n");
    s.push_str("/* Checked i32: compute in int64_t, range-check, THEN narrow — C signed\n");
    s.push_str(" * overflow is never executed (PLAN-921 §5.1). */\n");
    s.push_str("static inline a2x_result a2x_add(int32_t a, int32_t b) {\n");
    s.push_str("    int64_t s = (int64_t)a + (int64_t)b;\n");
    s.push_str("    if (s < INT32_MIN || s > INT32_MAX) {\n");
    s.push_str("        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;\n");
    s.push_str("    }\n");
    s.push_str("    return a2x_value((int32_t)s);\n");
    s.push_str("}\n\n");
    s.push_str("static inline a2x_result a2x_sub(int32_t a, int32_t b) {\n");
    s.push_str("    int64_t s = (int64_t)a - (int64_t)b;\n");
    s.push_str("    if (s < INT32_MIN || s > INT32_MAX) {\n");
    s.push_str("        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;\n");
    s.push_str("    }\n");
    s.push_str("    return a2x_value((int32_t)s);\n");
    s.push_str("}\n\n");
    s.push_str("static inline a2x_result a2x_mul(int32_t a, int32_t b) {\n");
    s.push_str("    int64_t s = (int64_t)a * (int64_t)b;\n");
    s.push_str("    if (s < INT32_MIN || s > INT32_MAX) {\n");
    s.push_str("        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;\n");
    s.push_str("    }\n");
    s.push_str("    return a2x_value((int32_t)s);\n");
    s.push_str("}\n\n");
    s.push_str("static inline a2x_result a2x_neg(int32_t a) {\n");
    s.push_str("    if (a == INT32_MIN) {\n");
    s.push_str("        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;\n");
    s.push_str("    }\n");
    s.push_str("    return a2x_value((int32_t)(-(int64_t)a));\n");
    s.push_str("}\n\n");
    s.push_str("/* div/rem: reject zero first, then INT32_MIN / -1; C truncation toward\n");
    s.push_str(" * zero preserves -7/3 == -2 and -7%3 == -1. */\n");
    s.push_str("static inline a2x_result a2x_div(int32_t a, int32_t b) {\n");
    s.push_str("    a2x_result r;\n");
    s.push_str("    if (b == 0) { r.value = 0; r.trap = A2X_TRAP_DIV_BY_ZERO; return r; }\n");
    s.push_str("    if (a == INT32_MIN && b == -1) { r.value = 0; r.trap = A2X_TRAP_DIV_OVERFLOW; return r; }\n");
    s.push_str("    return a2x_value(a / b);\n");
    s.push_str("}\n\n");
    s.push_str("static inline a2x_result a2x_rem(int32_t a, int32_t b) {\n");
    s.push_str("    a2x_result r;\n");
    s.push_str("    if (b == 0) { r.value = 0; r.trap = A2X_TRAP_REM_BY_ZERO; return r; }\n");
    s.push_str("    if (a == INT32_MIN && b == -1) { r.value = 0; r.trap = A2X_TRAP_REM_OVERFLOW; return r; }\n");
    s.push_str("    return a2x_value(a % b);\n");
    s.push_str("}\n");
    for &mark in marks {
        let (suffix, value) = if mark == "hir.test.mark_a" {
            ("a", "1")
        } else {
            ("b", "2")
        };
        s.push_str(&format!(
            "\nstatic inline int32_t a2x_mark_{suf}(void) {{\n\
             \x20   fprintf(stderr, \"mark_{suf}\\n\");\n\
             \x20   return {val};\n\
             }}\n",
            suf = suffix,
            val = value
        ));
    }
    s.push_str("\n#endif /* A2X_C_RUNTIME_H */\n");
    s
}
