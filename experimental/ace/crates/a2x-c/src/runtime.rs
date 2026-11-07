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
    out.write_str("    A2X_TRAP_REM_OVERFLOW,\n")?;
    // PLAN-961: the numeric-only domain class (shift amount out of
    // range, float→int NaN/±Inf/out-of-range) — name "InvalidScalar".
    out.write_str("    A2X_TRAP_INVALID\n")?;
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
    out.write_str("    static const char *const names[7] = {\n")?;
    out.write_str("        \"None\", \"Overflow\", \"DivByZero\", \"DivOverflow\",\n")?;
    out.write_str("        \"RemByZero\", \"RemOverflow\", \"InvalidScalar\"\n")?;
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
    out.write_str("}\n\n")?;

    // ---- PLAN-961 numeric runtime (frozen T-01 §2/§3 matrix) ----------
    // Integer lanes compute in the int64/uint64 domain (MSVC has no
    // __int128; _mul128/_umul128 cover the 64-bit high half), then
    // range-check against the operand width. Float /±0 traps; % is
    // fmod; shift amounts are checked before any shift executes.
    out.write_str("#include <intrin.h>\n")?;
    out.write_str("#include <string.h>\n")?;
    out.write_str("#include <math.h>\n")?;
    out.write_str("#include <float.h>\n\n")?;
    out.write_str("typedef struct { int64_t value; a2x_trap_code trap; } a2x_result_i64;\n")?;
    out.write_str("typedef struct { uint64_t value; a2x_trap_code trap; } a2x_result_u64;\n")?;
    out.write_str("typedef struct { float value; a2x_trap_code trap; } a2x_result_f32;\n")?;
    out.write_str("typedef struct { double value; a2x_trap_code trap; } a2x_result_f64;\n")?;
    out.write_str("#define A2X_OK_I(v) { (int64_t)(v), A2X_TRAP_NONE }\n")?;
    out.write_str("#define A2X_TRAP_I(t) { 0, (t) }\n")?;
    out.write_str("#define A2X_OK_U(v) { (uint64_t)(v), A2X_TRAP_NONE }\n")?;
    out.write_str("#define A2X_TRAP_U(t) { 0, (t) }\n")?;
    out.write_str("#define A2X_OK_F32(v) { (float)(v), A2X_TRAP_NONE }\n")?;
    out.write_str("#define A2X_OK_F64(v) { (double)(v), A2X_TRAP_NONE }\n\n")?;

    // Per-width integer arithmetic: (name, cty, signed, min, max, width,
    // result channel i=I64/u=U64).
    let int_tys: &[(&str, &str, bool, i128, u128, u32, u8)] = &[
        (
            "i8",
            "int8_t",
            true,
            i8::MIN as i128,
            i8::MAX as u128,
            8,
            b'i',
        ),
        ("u8", "uint8_t", false, 0, u8::MAX as u128, 8, b'u'),
        (
            "i32",
            "int32_t",
            true,
            i32::MIN as i128,
            i32::MAX as u128,
            32,
            b'i',
        ),
        ("u32", "uint32_t", false, 0, u32::MAX as u128, 32, b'u'),
        (
            "i64",
            "int64_t",
            true,
            i64::MIN as i128,
            i64::MAX as u128,
            64,
            b'i',
        ),
        ("u64", "uint64_t", false, 0, u64::MAX as u128, 64, b'u'),
    ];
    for &(name, cty, sgn, lo, hi, width, chan) in int_tys {
        let rt = if chan == b'i' { "i64" } else { "u64" };
        let ok = if chan == b'i' { "A2X_OK_I" } else { "A2X_OK_U" };
        let trap = if chan == b'i' {
            "A2X_TRAP_I"
        } else {
            "A2X_TRAP_U"
        };
        let dom = if chan == b'i' { "int64_t" } else { "uint64_t" };
        let _ = cty;
        // add / sub / mul: domain arithmetic + width range check.
        for op in ["add", "sub", "mul"] {
            let sym = if op == "add" {
                "+"
            } else if op == "sub" {
                "-"
            } else {
                "*"
            };
            out.write_str(&format!(
                "static inline a2x_result_{rt} a2x_{op}_{name}({dom} a, {dom} b) {{\n"
            ))?;
            if name == "i64" && op == "mul" {
                out.write_str("    int64_t lo64; int64_t hi64 = _mul128(a, b, &lo64);\n")?;
                out.write_str("    if (hi64 != lo64 >> 63) { return (a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW); }\n")?;
                out.write_str("    return (a2x_result_i64)A2X_OK_I(lo64);\n")?;
            } else if name == "u64" && op == "mul" {
                out.write_str("    uint64_t lo64; uint64_t hi64 = _umul128(a, b, &lo64);\n")?;
                out.write_str("    if (hi64 != 0) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }\n")?;
                out.write_str("    return (a2x_result_u64)A2X_OK_U(lo64);\n")?;
            } else {
                if sgn {
                    if op == "add" {
                        out.write_str(&format!("    if ((b > 0 && a > INT64_MAX - b) || (b < 0 && a < INT64_MIN - b)) {{ return (a2x_result_{rt}){trap}(A2X_TRAP_OVERFLOW); }}\n"))?;
                    } else if op == "sub" {
                        out.write_str(&format!("    if ((b < 0 && a > INT64_MAX + b) || (b > 0 && a < INT64_MIN + b)) {{ return (a2x_result_{rt}){trap}(A2X_TRAP_OVERFLOW); }}\n"))?;
                    }
                } else if op == "add" {
                    out.write_str("    if (b != 0 && a > UINT64_MAX - b) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }\n")?;
                } else if op == "sub" {
                    out.write_str("    if (b != 0 && a < b) { return (a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW); }\n")?;
                }
                out.write_str(&format!(
                    "    {dom} v = ({dom})(a {sym} b);
"
                ))?;
                if width < 64 {
                    out.write_str(&format!(
                        "    if ({bounds}) {{ return (a2x_result_{rt}){trap}(A2X_TRAP_OVERFLOW); }}\n",
                        bounds = if sgn {
                            format!("v < ({lo}LL) || v > ({hi}LL)")
                        } else {
                            format!("v > ({hi}ULL)")
                        }
                    ))?;
                }
                out.write_str(&format!("    return (a2x_result_{rt}){ok}(v);\n"))?;
            }
            out.write_str("}\n\n")?;
        }
        // div / rem: zero, per-width MIN/-1, then domain op + range check.
        for op in ["div", "rem"] {
            let tcode = if op == "div" {
                "A2X_TRAP_DIV_OVERFLOW"
            } else {
                "A2X_TRAP_REM_OVERFLOW"
            };
            out.write_str(&format!(
                "static inline a2x_result_{rt} a2x_{op}_{name}({dom} a, {dom} b) {{\n"
            ))?;
            out.write_str(&format!(
                "    if (b == 0) {{ return (a2x_result_{rt}){trap}(A2X_TRAP_DIV_BY_ZERO); }}\n"
            ))?;
            if sgn {
                out.write_str(&format!(
                    "    if (a == ({dom})({lo}LL) && b == -1) {{ return (a2x_result_{rt}){trap}({tcode}); }}\n"
                ))?;
            }
            let sym = if op == "div" { "/" } else { "%" };
            out.write_str(&format!("    {dom} v = ({dom})(a {sym} b);\n"))?;
            if width < 64 {
                out.write_str(&format!(
                    "    if ({}) {{ return (a2x_result_{rt}){trap}(A2X_TRAP_OVERFLOW); }}\n",
                    if sgn {
                        format!("v < ({lo}LL) || v > ({hi}LL)")
                    } else {
                        format!("v > ({hi}ULL)")
                    }
                ))?;
            }
            out.write_str(&format!("    return (a2x_result_{rt}){ok}(v);\n"))?;
            out.write_str("}\n\n")?;
        }
        // neg: signed only; MIN traps.
        if sgn {
            out.write_str(&format!(
                "static inline a2x_result_{rt} a2x_neg_{name}({dom} a) {{\n"
            ))?;
            out.write_str(&format!(
                "    if (a == ({dom})({lo}LL)) {{ return (a2x_result_{rt}){trap}(A2X_TRAP_OVERFLOW); }}\n"
            ))?;
            out.write_str(&format!("    return (a2x_result_{rt}){ok}(-a);\n"))?;
            out.write_str("}\n\n")?;
        }
        // bit ops: native and in-range (no trap).
        for (op, sym) in [("and", "&"), ("or", "|"), ("xor", "^")] {
            out.write_str(&format!(
                "static inline a2x_result_{rt} a2x_{op}_{name}({dom} a, {dom} b) {{\n"
            ))?;
            out.write_str(&format!("    return (a2x_result_{rt}){ok}(a {sym} b);\n"))?;
            out.write_str("}\n\n")?;
        }
        // shifts: amount checked against the WIDTH; value wraps.
        for (op, cast) in [("shl", ""), ("shr", "")] {
            out.write_str(&format!(
                "static inline a2x_result_{rt} a2x_{op}_{name}({dom} a, {dom} n) {{\n"
            ))?;
            if sgn {
                out.write_str(&format!(
                    "    if (n < 0 || n >= {width}) {{ return (a2x_result_{rt}){trap}(A2X_TRAP_INVALID); }}\n"
                ))?;
            } else {
                out.write_str(&format!(
                    "    if (n >= {width}) {{ return (a2x_result_{rt}){trap}(A2X_TRAP_INVALID); }}\n"
                ))?;
            }
            let body = if op == "shl" {
                if sgn {
                    format!("({dom})((uint64_t)a << (int)n)")
                } else {
                    format!("a << (int)n")
                }
            } else if sgn {
                "a >> (int)n".to_string()
            } else {
                "a >> (int)n".to_string()
            };
            let _ = cast;
            out.write_str(&format!("    return (a2x_result_{rt}){ok}({body});\n"))?;
            out.write_str("}\n\n")?;
        }
    }

    // Float lanes: div-by-±0 traps; rem = fmod; arith is infallible.
    out.write_str("static inline a2x_result_f32 a2x_div_f32(float a, float b) {\n")?;
    out.write_str("    if (b == 0.0f) { return (a2x_result_f32){ 0, A2X_TRAP_DIV_BY_ZERO }; }\n")?;
    out.write_str("    return (a2x_result_f32)A2X_OK_F32(a / b);\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline a2x_result_f64 a2x_div_f64(double a, double b) {\n")?;
    out.write_str("    if (b == 0.0) { return (a2x_result_f64){ 0, A2X_TRAP_DIV_BY_ZERO }; }\n")?;
    out.write_str("    return (a2x_result_f64)A2X_OK_F64(a / b);\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline a2x_result_f32 a2x_rem_f32(float a, float b) {\n")?;
    out.write_str("    return (a2x_result_f32)A2X_OK_F32(fmodf(a, b));\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline a2x_result_f64 a2x_rem_f64(double a, double b) {\n")?;
    out.write_str("    return (a2x_result_f64)A2X_OK_F64(fmod(a, b));\n")?;
    out.write_str("}\n\n")?;

    // Conversion matrix: int→int checked narrowing/widening; int→float;
    // float→int checked; f32→f64 exact; f64→f32 checked.
    let all_tys: &[(&str, bool)] = &[
        ("i8", true),
        ("i32", true),
        ("i64", true),
        ("u8", false),
        ("u32", false),
        ("u64", false),
    ];
    let dom_ty = |n: &str| -> &'static str {
        match n {
            "i8" | "i32" | "i64" => "int64_t",
            _ => "uint64_t",
        }
    };
    // PLAN-961: the conversion channel mirrors the emission-side
    // c_result_ty (i32 keeps the frozen a2x_result int32 channel).
    let rt_chan = |n: &str| -> &'static str {
        match n {
            "i32" => "",
            "i8" | "i64" => "i64",
            _ => "u64",
        }
    };
    let lo_hi = |n: &str| -> (i128, u128) {
        match n {
            "i8" => (i8::MIN as i128, i8::MAX as u128),
            "i32" => (i32::MIN as i128, i32::MAX as u128),
            "i64" => (i64::MIN as i128, i64::MAX as u128),
            "u8" => (0, u8::MAX as u128),
            "u32" => (0, u32::MAX as u128),
            _ => (0, u64::MAX as u128),
        }
    };
    for &(f, _) in all_tys {
        for &(t, t_sgn) in all_tys {
            if f == t {
                continue;
            }
            let fdom = dom_ty(f);
            let frt = rt_chan(t);
            let chan_name: &str = if frt.is_empty() {
                "a2x_result"
            } else if frt == "i64" {
                "a2x_result_i64"
            } else {
                "a2x_result_u64"
            };
            out.write_str(&format!(
                "static inline {chan_name} a2x_c_{f}_{t}({fdom} a) {{\n"
            ))?;
            let (tlo, thi) = lo_hi(t);
            let _ = t_sgn;
            // An UNSIGNED source domain never has a lower-bound check
            // (an unsigned < negative literal comparison reinterprets the
            // literal as huge — always-true — MSVC trap).
            let src_unsigned = matches!(f, "u8" | "u32" | "u64");
            let check = if tlo == 0 || src_unsigned {
                format!("a > ({thi}ULL)")
            } else {
                format!("a < ({tlo}LL) || a > ({thi}LL)")
            };
            let trap_lit = if frt.is_empty() {
                "(a2x_result){ 0, A2X_TRAP_OVERFLOW }".to_string()
            } else if frt == "i64" {
                "(a2x_result_i64)A2X_TRAP_I(A2X_TRAP_OVERFLOW)".to_string()
            } else {
                "(a2x_result_u64)A2X_TRAP_U(A2X_TRAP_OVERFLOW)".to_string()
            };
            out.write_str(&format!("    if ({check}) {{ return {trap_lit}; }}\n"))?;
            let (ok_lit, out_dom): (String, &str) = if frt.is_empty() {
                ("a2x_value".to_string(), "int32_t")
            } else if frt == "i64" {
                ("(a2x_result_i64)A2X_OK_I".to_string(), "int64_t")
            } else {
                ("(a2x_result_u64)A2X_OK_U".to_string(), "uint64_t")
            };
            out.write_str(&format!(
                "    return {ok_lit}(({out_dom})a);
"
            ))?;
            out.write_str("}\n\n")?;
        }
    }
    // int → float (exact or round-to-nearest).
    for &(f, _) in all_tys {
        for (t, tcty) in [("f32", "float"), ("f64", "double")] {
            let fdom = dom_ty(f);
            out.write_str(&format!(
                "static inline a2x_result_{t} a2x_c_{f}_{t}({fdom} a) {{\n"
            ))?;
            let ok = if t == "f32" {
                "A2X_OK_F32"
            } else {
                "A2X_OK_F64"
            };
            out.write_str(&format!("    return (a2x_result_{t}){ok}(({tcty})a);\n"))?;
            out.write_str("}\n\n")?;
        }
    }
    // float → int: NaN/±Inf/range checked, truncation toward zero.
    for (f, fcty) in [("f32", "float"), ("f64", "double")] {
        for &(t, _) in all_tys {
            let (tlo, thi) = lo_hi(t);
            let frt = rt_chan(t);
            let chan_name2: &str = if frt.is_empty() {
                "a2x_result"
            } else if frt == "i64" {
                "a2x_result_i64"
            } else {
                "a2x_result_u64"
            };
            out.write_str(&format!(
                "static inline {chan_name2} a2x_c_{f}_{t}({fcty} a) {{\n"
            ))?;
            out.write_str("    if (!(a == a)) { goto bad; }\n")?;
            let lo_lit = if tlo == 0 {
                "0.0".to_string()
            } else {
                format!("{tlo}.0")
            };
            let hi_lit = if tlo == 0 {
                format!("{thi}.0")
            } else {
                format!("{}.0", thi.wrapping_add(1))
            };
            out.write_str(&format!(
                "    if (a < {lo_lit} || a >= {hi_lit}) {{ goto bad; }}\n"
            ))?;
            let (ok_lit, out_dom): (String, &str) = if frt.is_empty() {
                ("a2x_value".to_string(), "int32_t")
            } else if frt == "i64" {
                ("(a2x_result_i64)A2X_OK_I".to_string(), "int64_t")
            } else {
                ("(a2x_result_u64)A2X_OK_U".to_string(), "uint64_t")
            };
            out.write_str(&format!(
                "    return {ok_lit}(({out_dom})a);
"
            ))?;
            let bad_lit = if frt.is_empty() {
                "(a2x_result){ 0, A2X_TRAP_INVALID }".to_string()
            } else if frt == "i64" {
                "(a2x_result_i64)A2X_TRAP_I(A2X_TRAP_INVALID)".to_string()
            } else {
                "(a2x_result_u64)A2X_TRAP_U(A2X_TRAP_INVALID)".to_string()
            };
            out.write_str(&format!("bad: return {bad_lit};\n"))?;
            out.write_str("}\n\n")?;
        }
    }
    out.write_str("static inline a2x_result_f64 a2x_c_f32_f64(float a) {\n")?;
    out.write_str("    return (a2x_result_f64)A2X_OK_F64((double)a);\n")?;
    out.write_str("}\n\n")?;
    out.write_str("static inline a2x_result_f32 a2x_c_f64_f32(double a) {\n")?;
    out.write_str("    float r = (float)a;\n")?;
    out.write_str("    if ((r > FLT_MAX || r < -FLT_MAX)) {\n")?;
    out.write_str("        return (a2x_result_f32){ 0, A2X_TRAP_OVERFLOW };\n")?;
    out.write_str("    }\n")?;
    out.write_str("    return (a2x_result_f32)A2X_OK_F32(r);\n")?;
    out.write_str("}\n\n")?;
    // Bit-exact float literal constructors.
    out.write_str("static inline float a2x_f32_bits(uint32_t bits) {\n")?;
    out.write_str("    float f; memcpy(&f, &bits, sizeof f); return f;\n")?;
    out.write_str("}\n")?;
    out.write_str("static inline double a2x_f64_bits(uint64_t bits) {\n")?;
    out.write_str("    double d; memcpy(&d, &bits, sizeof d); return d;\n")?;
    out.write_str("}\n\n")?;

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
