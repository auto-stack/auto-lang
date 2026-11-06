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
///
/// PLAN-938: `write_runtime` is the ONE formatter; the legacy entry keeps
/// its exact historical bytes, and the admitted entry reserves the exact
/// output size with the host before allocating the payload.
pub fn runtime_source(marks: &[&'static str]) -> String {
    let mut s = String::new();
    write_runtime(marks, &mut s).expect("formatting into a String cannot fail");
    s
}

/// Same bytes as [`runtime_source`], with the exact size admitted by `admit`
/// BEFORE the single exactly-sized allocation.
pub fn runtime_source_admitted(
    marks: &[&'static str],
    admit: impl FnOnce(usize) -> Result<(), String>,
) -> Result<String, String> {
    crate::admitted_text::render_admitted(|out| write_runtime(marks, out), admit)
}

/// The single formatter behind both entries (counting pass allocates
/// nothing; the writer is bounded by the admitted size).
pub fn write_runtime(marks: &[&'static str], out: &mut dyn std::fmt::Write) -> std::fmt::Result {
    out.write_str("// AUTO-GENERATED minimal typed runtime (PLAN-921 CA-A) — DO NOT EDIT.\n")?;
    out.write_str("// Checked i32 primitives with named traps + authorized trace marks.\n")?;
    out.write_str("#ifndef A2X_C_RUNTIME_H\n#define A2X_C_RUNTIME_H\n\n")?;
    out.write_str("#include <stdbool.h>\n#include <stdint.h>\n#include <stdio.h>\n\n")?;
    out.write_str("typedef enum {\n")?;
    out.write_str("    A2X_TRAP_NONE = 0,\n")?;
    out.write_str("    A2X_TRAP_OVERFLOW,\n")?;
    out.write_str("    A2X_TRAP_DIV_BY_ZERO,\n")?;
    out.write_str("    A2X_TRAP_DIV_OVERFLOW,\n")?;
    out.write_str("    A2X_TRAP_REM_BY_ZERO,\n")?;
    out.write_str("    A2X_TRAP_REM_OVERFLOW\n")?;
    out.write_str("} a2x_trap_code;\n\n")?;
    out.write_str("typedef struct {\n")?;
    out.write_str("    int32_t value;\n")?;
    out.write_str("    a2x_trap_code trap;\n")?;
    out.write_str("} a2x_result;\n\n")?;
    out.write_str("static inline a2x_result a2x_value(int32_t v) {\n")?;
    out.write_str("    a2x_result r;\n")?;
    out.write_str("    r.value = v;\n")?;
    out.write_str("    r.trap = A2X_TRAP_NONE;\n")?;
    out.write_str("    return r;\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline const char *a2x_trap_name(a2x_trap_code t) {\n")?;
    out.write_str("    static const char *const names[6] = {\n")?;
    out.write_str("        \"None\", \"Overflow\", \"DivByZero\", \"DivOverflow\",\n")?;
    out.write_str("        \"RemByZero\", \"RemOverflow\"\n")?;
    out.write_str("    };\n")?;
    out.write_str("    return names[(int)t];\n")?;
    out.write_str("}\n\n")?;
    out.write_str("/* Checked i32: compute in int64_t, range-check, THEN narrow — C signed\n")?;
    out.write_str(" * overflow is never executed (PLAN-921 §5.1). */\n")?;
    out.write_str("static inline a2x_result a2x_add(int32_t a, int32_t b) {\n")?;
    out.write_str("    int64_t s = (int64_t)a + (int64_t)b;\n")?;
    out.write_str("    if (s < INT32_MIN || s > INT32_MAX) {\n")?;
    out.write_str("        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;\n")?;
    out.write_str("    }\n")?;
    out.write_str("    return a2x_value((int32_t)s);\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline a2x_result a2x_sub(int32_t a, int32_t b) {\n")?;
    out.write_str("    int64_t s = (int64_t)a - (int64_t)b;\n")?;
    out.write_str("    if (s < INT32_MIN || s > INT32_MAX) {\n")?;
    out.write_str("        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;\n")?;
    out.write_str("    }\n")?;
    out.write_str("    return a2x_value((int32_t)s);\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline a2x_result a2x_mul(int32_t a, int32_t b) {\n")?;
    out.write_str("    int64_t s = (int64_t)a * (int64_t)b;\n")?;
    out.write_str("    if (s < INT32_MIN || s > INT32_MAX) {\n")?;
    out.write_str("        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;\n")?;
    out.write_str("    }\n")?;
    out.write_str("    return a2x_value((int32_t)s);\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline a2x_result a2x_neg(int32_t a) {\n")?;
    out.write_str("    if (a == INT32_MIN) {\n")?;
    out.write_str("        a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r;\n")?;
    out.write_str("    }\n")?;
    out.write_str("    return a2x_value((int32_t)(-(int64_t)a));\n")?;
    out.write_str("}\n\n")?;
    out.write_str("/* div/rem: reject zero first, then INT32_MIN / -1; C truncation toward\n")?;
    out.write_str(" * zero preserves -7/3 == -2 and -7%3 == -1. */\n")?;
    out.write_str("static inline a2x_result a2x_div(int32_t a, int32_t b) {\n")?;
    out.write_str("    a2x_result r;\n")?;
    out.write_str("    if (b == 0) { r.value = 0; r.trap = A2X_TRAP_DIV_BY_ZERO; return r; }\n")?;
    out.write_str(
        "    if (a == INT32_MIN && b == -1) { r.value = 0; r.trap = A2X_TRAP_DIV_OVERFLOW; return r; }\n",
    )?;
    out.write_str("    return a2x_value(a / b);\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline a2x_result a2x_rem(int32_t a, int32_t b) {\n")?;
    out.write_str("    a2x_result r;\n")?;
    out.write_str("    if (b == 0) { r.value = 0; r.trap = A2X_TRAP_REM_BY_ZERO; return r; }\n")?;
    out.write_str(
        "    if (a == INT32_MIN && b == -1) { r.value = 0; r.trap = A2X_TRAP_REM_OVERFLOW; return r; }\n",
    )?;
    out.write_str("    return a2x_value(a % b);\n")?;
    out.write_str("}\n")?;
    for &mark in marks {
        let (suffix, value) = if mark == "hir.test.mark_a" {
            ("a", "1")
        } else {
            ("b", "2")
        };
        write!(
            out,
            "\nstatic inline int32_t a2x_mark_{suf}(void) {{\n    fprintf(stderr, \"mark_{suf}\\n\");\n    return {val};\n}}\n",
            suf = suffix,
            val = value
        )?;
    }
    out.write_str("\n#endif /* A2X_C_RUNTIME_H */\n")
}
