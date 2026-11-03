//! PLAN-925 CA-B: the C11 RECORD-TEXT runtime header text (a versioned
//! text asset of the provider, marks-parameterized like the scalar one).
//!
//! Semantics mirror the Rust record-text runtime (a2x-rust/src/rt.rs)
//! exactly: the scalar checked-i32 core plus the text/char/resource layer —
//! * owned `Str` is `a2x_str` (one buffer); cleanup is EXPLICIT
//!   (`a2x_str_free` at the verifier-scheduled points; no hidden release);
//! * allocation budgets are checked BEFORE every allocation (checked
//!   length arithmetic; host `malloc` failure is the typed `Resource`
//!   trap, never a crash);
//! * `len` counts Unicode scalars, `len_bytes` UTF-8 bytes, `eq/ne`
//!   compare full bytes with no normalization (NUL included — nothing
//!   ever calls `strlen` on guest text);
//! * `char_at`/`slice` are scalar-indexed; negative/out-of-range/invalid
//!   positions are the typed `TextIndex` trap; `char.from_codepoint`
//!   rejects non-scalars as `InvalidScalar`;
//! * a TEST-ONLY, default-off fault/observation configuration is read
//!   lazily from the environment by the runtime itself — the emitted file
//!   set is identical in daily and certified runs:
//!   `A2X_RT_FAULT_ALLOC=<n>` fails the n-th allocation (typed `Resource`,
//!   no partial commit), `A2X_RT_EVENTS=1` mirrors alloc/free events to
//!   stderr as `rt:alloc:<bytes>` / `rt:free:<bytes>` lines.
//!
//! Everything lives in ONE header (`a2x_c_rt_runtime.h`) included by
//! `main.c` and `generated.c` — all `static` (per-TU state; only
//! `generated.c`'s copy is ever exercised because every allocation, free
//! and mark happens in generated code).

/// The record-text runtime header for a view: checked-i32 core + the
/// text/char/resource layer + the marks the view actually uses
/// (content-hashed into every AXC2 fragment key, so a runtime change can
/// never hit stale fragments).
pub fn record_text_runtime_source(marks: &[&'static str]) -> String {
    let mut s = String::new();
    s.push_str("// AUTO-GENERATED typed resource runtime (PLAN-925 CA-B) — DO NOT EDIT.\n");
    s.push_str("// Checked i32 core + owned-Str resource layer with explicit cleanup,\n");
    s.push_str("// pre-allocation budget checks and default-off fault/observation hooks.\n");
    s.push_str("#ifndef A2X_C_RT_RUNTIME_H\n#define A2X_C_RT_RUNTIME_H\n\n");
    s.push_str("#include <stdbool.h>\n#include <stddef.h>\n#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\n");
    s.push_str(RT_CORE);
    for &mark in marks {
        let (suffix, value) = if mark == "hir.test.mark_a" {
            ("a", "1")
        } else {
            ("b", "2")
        };
        s.push_str(&format!(
            "\nstatic int32_t a2x_mark_{suf}(void) {{\n\
             \x20   fprintf(stderr, \"mark_{suf}\\n\");\n\
             \x20   return {val};\n\
             }}\n",
            suf = suffix,
            val = value
        ));
    }
    s.push_str("\n#endif /* A2X_C_RT_RUNTIME_H */\n");
    s
}

/// Frozen runtime core (byte-stable for a crate version).
const RT_CORE: &str = r#"typedef enum {
    A2X_TRAP_NONE = 0,
    A2X_TRAP_OVERFLOW,
    A2X_TRAP_DIV_BY_ZERO,
    A2X_TRAP_DIV_OVERFLOW,
    A2X_TRAP_REM_BY_ZERO,
    A2X_TRAP_REM_OVERFLOW,
    A2X_TRAP_INVALID_SCALAR,
    A2X_TRAP_TEXT_INDEX,
    A2X_TRAP_RESOURCE
} a2x_trap_code;

typedef struct { int32_t value; a2x_trap_code trap; } a2x_result;
typedef struct { uint32_t value; a2x_trap_code trap; } a2x_char_result;

/* A borrowed text view: (data, byte length). Never NUL-terminated; every
 * operation carries the explicit length — strlen is never called. */
typedef struct { const uint8_t* data; size_t len; } a2x_str_view;
/* An owned text buffer: exactly one allocation, released ONLY by the
 * explicit a2x_str_free the lowering emits at verifier-scheduled points. */
typedef struct { uint8_t* data; size_t len; } a2x_str;

typedef struct { a2x_str_view value; a2x_trap_code trap; } a2x_strv_result;
typedef struct { a2x_str value; a2x_trap_code trap; } a2x_stro_result;

static inline a2x_result a2x_value(int32_t v) {
    a2x_result r; r.value = v; r.trap = A2X_TRAP_NONE; return r;
}

/* value constructors + trap forwarders of the non-i32 result kinds (the
 * per-record ones are generated into generated.c next to their structs). */
static inline a2x_char_result a2x_char_value(uint32_t c) {
    a2x_char_result r; r.value = c; r.trap = A2X_TRAP_NONE; return r;
}
static inline a2x_strv_result a2x_strv_value(a2x_str_view v) {
    a2x_strv_result r; r.value = v; r.trap = A2X_TRAP_NONE; return r;
}
static inline a2x_stro_result a2x_stro_value(a2x_str v) {
    a2x_stro_result r; r.value = v; r.trap = A2X_TRAP_NONE; return r;
}
static inline a2x_result a2x_fwd_i32(a2x_trap_code t) {
    a2x_result r; r.value = 0; r.trap = t; return r;
}
static inline a2x_char_result a2x_fwd_char(a2x_trap_code t) {
    a2x_char_result r; r.value = 0; r.trap = t; return r;
}
static inline a2x_strv_result a2x_fwd_strv(a2x_trap_code t) {
    a2x_strv_result r; r.value.data = NULL; r.value.len = 0; r.trap = t; return r;
}
static inline a2x_stro_result a2x_fwd_stro(a2x_trap_code t) {
    a2x_stro_result r; r.value.data = NULL; r.value.len = 0; r.trap = t; return r;
}

static const char* a2x_trap_name(a2x_trap_code t) {
    static const char* const names[9] = {
        "None", "Overflow", "DivByZero", "DivOverflow",
        "RemByZero", "RemOverflow", "InvalidScalar", "TextIndex", "Resource"
    };
    return names[(int)t];
}

/* Checked i32: compute in int64_t, range-check, THEN narrow — C signed
 * overflow is never executed (PLAN-921 §5.1, unchanged here). */
static a2x_result a2x_add(int32_t a, int32_t b) {
    int64_t s = (int64_t)a + (int64_t)b;
    if (s < INT32_MIN || s > INT32_MAX) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r; }
    return a2x_value((int32_t)s);
}
static a2x_result a2x_sub(int32_t a, int32_t b) {
    int64_t s = (int64_t)a - (int64_t)b;
    if (s < INT32_MIN || s > INT32_MAX) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r; }
    return a2x_value((int32_t)s);
}
static a2x_result a2x_mul(int32_t a, int32_t b) {
    int64_t s = (int64_t)a * (int64_t)b;
    if (s < INT32_MIN || s > INT32_MAX) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r; }
    return a2x_value((int32_t)s);
}
static a2x_result a2x_neg(int32_t a) {
    if (a == INT32_MIN) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_OVERFLOW; return r; }
    return a2x_value((int32_t)(-(int64_t)a));
}
static a2x_result a2x_div(int32_t a, int32_t b) {
    a2x_result r;
    if (b == 0) { r.value = 0; r.trap = A2X_TRAP_DIV_BY_ZERO; return r; }
    if (a == INT32_MIN && b == -1) { r.value = 0; r.trap = A2X_TRAP_DIV_OVERFLOW; return r; }
    return a2x_value(a / b);
}
static a2x_result a2x_rem(int32_t a, int32_t b) {
    a2x_result r;
    if (b == 0) { r.value = 0; r.trap = A2X_TRAP_REM_BY_ZERO; return r; }
    if (a == INT32_MIN && b == -1) { r.value = 0; r.trap = A2X_TRAP_REM_OVERFLOW; return r; }
    return a2x_value(a % b);
}

/* ---- allocation ledger (default-off fault/observation hooks) --------
 * A faulted attempt is NOT an allocation: no buffer exists, no event. */

static long a2x_rt_cfg_state = 0; /* 0 = uninitialized, 1 = read */
static long a2x_rt_cfg_fault = 0; /* 1-based ordinal to fail, <=0 off */
static int a2x_rt_cfg_events = 0;

static void a2x_rt_config(void) {
    if (a2x_rt_cfg_state == 0) {
        const char* fault = getenv("A2X_RT_FAULT_ALLOC");
        const char* events = getenv("A2X_RT_EVENTS");
        if (fault != NULL) {
            long n = 0; int ok = 0;
            while (*fault >= '0' && *fault <= '9') { n = n * 10 + (long)(*fault - '0'); ok = 1; fault++; }
            if (ok && *fault == '\0' && n > 0) a2x_rt_cfg_fault = n;
        }
        if (events != NULL && events[0] == '1' && events[1] == '\0') a2x_rt_cfg_events = 1;
        a2x_rt_cfg_state = 1;
    }
}

static long a2x_rt_alloc_ordinal = 0;
static long a2x_rt_free_count = 0;

/* Returns nonzero when this allocation is the faulted one (typed
 * Resource; the caller must commit nothing). */
static int a2x_rt_reserve(size_t bytes) {
    long n;
    a2x_rt_config();
    n = ++a2x_rt_alloc_ordinal;
    if (a2x_rt_cfg_fault == n) return 1;
    if (a2x_rt_cfg_events) fprintf(stderr, "rt:alloc:%llu\n", (unsigned long long)bytes);
    return 0;
}

static void a2x_rt_note_free(size_t bytes) {
    a2x_rt_free_count++;
    a2x_rt_config();
    if (a2x_rt_cfg_events) fprintf(stderr, "rt:free:%llu\n", (unsigned long long)bytes);
}

/* ---- text layer ------------------------------------------------------ */

static inline a2x_str_view a2x_str_view_of(const a2x_str* s) {
    a2x_str_view v; v.data = s->data; v.len = s->len; return v;
}

static a2x_stro_result a2x_str_alloc(size_t len, const uint8_t* fill) {
    a2x_stro_result r;
    if (a2x_rt_reserve(len)) { r.value.data = NULL; r.value.len = 0; r.trap = A2X_TRAP_RESOURCE; return r; }
    r.value.data = (uint8_t*)malloc(len == 0 ? 1 : len);
    if (r.value.data == NULL) { r.value.len = 0; r.trap = A2X_TRAP_RESOURCE; return r; }
    r.value.len = len;
    if (len > 0) memcpy(r.value.data, fill, len);
    r.trap = A2X_TRAP_NONE;
    return r;
}

/* Literal promotion (AP-4): an owned copy of a static literal. */
static a2x_stro_result a2x_str_from_lit(a2x_str_view lit) {
    return a2x_str_alloc(lit.len, lit.data);
}
/* Independent owned copy (AP-2, s.clone()). */
static a2x_stro_result a2x_str_clone(a2x_str_view s) {
    return a2x_str_alloc(s.len, s.data);
}
/* Promotion of a borrowed view (s.to(Str)). */
static a2x_stro_result a2x_str_to_owned(a2x_str_view s) {
    return a2x_str_alloc(s.len, s.data);
}
/* Concatenation into a fresh owned buffer (AP-3) — checked length
 * arithmetic BEFORE the allocation. */
static a2x_stro_result a2x_str_concat(a2x_str_view l, a2x_str_view r) {
    a2x_stro_result out;
    uint8_t* buf;
    if (l.len > SIZE_MAX - r.len) { out.value.data = NULL; out.value.len = 0; out.trap = A2X_TRAP_RESOURCE; return out; }
    if (a2x_rt_reserve(l.len + r.len)) { out.value.data = NULL; out.value.len = 0; out.trap = A2X_TRAP_RESOURCE; return out; }
    buf = (uint8_t*)malloc(l.len + r.len == 0 ? 1 : l.len + r.len);
    if (buf == NULL) { out.value.data = NULL; out.value.len = 0; out.trap = A2X_TRAP_RESOURCE; return out; }
    if (l.len > 0) memcpy(buf, l.data, l.len);
    if (r.len > 0) memcpy(buf + l.len, r.data, r.len);
    out.value.data = buf; out.value.len = l.len + r.len; out.trap = A2X_TRAP_NONE;
    return out;
}

/* Explicit free of an owned buffer — the ONLY release path. */
static void a2x_str_free(a2x_str* s) {
    if (s != NULL && s->data != NULL) {
        a2x_rt_note_free(s->len);
        free(s->data);
        s->data = NULL;
        s->len = 0;
    }
}

/* UTF-8 lead-byte length (1..4); 0 on a continuation byte. */
static inline size_t a2x_utf8_step(uint8_t b) {
    if ((b & 0x80u) == 0x00u) return 1;
    if ((b & 0xE0u) == 0xC0u) return 2;
    if ((b & 0xF0u) == 0xE0u) return 3;
    if ((b & 0xF8u) == 0xF0u) return 4;
    return 0;
}

/* Unicode scalar count. */
static a2x_result a2x_str_len(a2x_str_view s) {
    size_t scalars = 0, i = 0;
    int64_t n;
    while (i < s.len) {
        size_t step = a2x_utf8_step(s.data[i]);
        if (step == 0 || i + step > s.len) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_TEXT_INDEX; return r; }
        i += step; scalars++;
    }
    n = (int64_t)scalars;
    if (n > INT32_MAX) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_RESOURCE; return r; }
    return a2x_value((int32_t)n);
}

/* UTF-8 byte length. */
static a2x_result a2x_str_len_bytes(a2x_str_view s) {
    int64_t n = (int64_t)s.len;
    if (n > INT32_MAX) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_RESOURCE; return r; }
    return a2x_value((int32_t)n);
}

/* Full-byte equality (no normalization; NUL included). */
static bool a2x_str_eq(a2x_str_view l, a2x_str_view r) {
    return l.len == r.len && (l.len == 0 || memcmp(l.data, r.data, l.len) == 0);
}
static bool a2x_str_ne(a2x_str_view l, a2x_str_view r) {
    return !a2x_str_eq(l, r);
}

/* Decode the scalar starting at byte offset i (well-formed by
 * construction; a malformed lead returns TextIndex defensively). */
static a2x_char_result a2x_utf8_decode(a2x_str_view s, size_t i) {
    a2x_char_result bad; bad.value = 0; bad.trap = A2X_TRAP_TEXT_INDEX;
    uint8_t b = s.data[i];
    uint32_t cp; size_t step;
    if ((b & 0x80u) == 0x00u) { cp = b; step = 1; }
    else if ((b & 0xE0u) == 0xC0u) {
        if (i + 1 >= s.len || (s.data[i+1] & 0xC0u) != 0x80u) return bad;
        cp = ((uint32_t)(b & 0x1Fu) << 6) | (uint32_t)(s.data[i+1] & 0x3Fu); step = 2;
    } else if ((b & 0xF0u) == 0xE0u) {
        if (i + 2 >= s.len || (s.data[i+1] & 0xC0u) != 0x80u || (s.data[i+2] & 0xC0u) != 0x80u) return bad;
        cp = ((uint32_t)(b & 0x0Fu) << 12) | ((uint32_t)(s.data[i+1] & 0x3Fu) << 6) | (uint32_t)(s.data[i+2] & 0x3Fu); step = 3;
    } else if ((b & 0xF8u) == 0xF0u) {
        if (i + 3 >= s.len || (s.data[i+1] & 0xC0u) != 0x80u || (s.data[i+2] & 0xC0u) != 0x80u || (s.data[i+3] & 0xC0u) != 0x80u) return bad;
        cp = ((uint32_t)(b & 0x07u) << 18) | ((uint32_t)(s.data[i+1] & 0x3Fu) << 12) | ((uint32_t)(s.data[i+2] & 0x3Fu) << 6) | (uint32_t)(s.data[i+3] & 0x3Fu); step = 4;
    } else {
        return bad;
    }
    (void)step;
    bad.value = cp; bad.trap = A2X_TRAP_NONE;
    return bad;
}

/* Scalar-indexed char access; out-of-bounds (incl. negative) is the typed
 * TextIndex trap, never a byte-level misread. */
static a2x_char_result a2x_str_char_at(a2x_str_view s, int32_t idx) {
    a2x_char_result bad; bad.value = 0; bad.trap = A2X_TRAP_TEXT_INDEX;
    size_t scalars, i;
    if (idx < 0) return bad;
    scalars = 0; i = 0;
    while (i < s.len) {
        size_t step;
        if (scalars == (size_t)idx) return a2x_utf8_decode(s, i);
        step = a2x_utf8_step(s.data[i]);
        if (step == 0 || i + step > s.len) return bad;
        i += step; scalars++;
    }
    return bad;
}

/* Scalar-indexed [start, end) slice — a view of the operand (no
 * allocation). Invalid ranges are TextIndex. */
static a2x_strv_result a2x_str_slice(a2x_str_view s, int32_t start, int32_t end) {
    a2x_strv_result bad; bad.value.data = NULL; bad.value.len = 0; bad.trap = A2X_TRAP_TEXT_INDEX;
    a2x_strv_result ok;
    size_t scalars, i, byte_start = 0, byte_end = 0;
    int have_start = 0, have_end = 0;
    if (start < 0 || end < start) return bad;
    scalars = 0; i = 0;
    while (i < s.len) {
        size_t step;
        if (scalars == (size_t)start) { byte_start = i; have_start = 1; }
        if (scalars == (size_t)end) { byte_end = i; have_end = 1; }
        step = a2x_utf8_step(s.data[i]);
        if (step == 0 || i + step > s.len) return bad;
        i += step; scalars++;
    }
    if (scalars == (size_t)start) { byte_start = i; have_start = 1; }
    if (scalars == (size_t)end) { byte_end = i; have_end = 1; }
    if (!have_start || !have_end) return bad;
    ok.value.data = s.data + byte_start;
    ok.value.len = byte_end - byte_start;
    ok.trap = A2X_TRAP_NONE;
    return ok;
}

/* char code point -> i32 (pure widening). */
static inline int32_t a2x_char_to_codepoint(uint32_t c) {
    return (int32_t)c;
}
/* i32 -> char; non-scalars (surrogates, > U+10FFFF, negative) are the
 * typed InvalidScalar trap. */
static a2x_char_result a2x_char_from_codepoint(int32_t n) {
    a2x_char_result bad; bad.value = 0; bad.trap = A2X_TRAP_INVALID_SCALAR;
    a2x_char_result ok;
    uint32_t u = (uint32_t)n;
    if (n < 0 || u > 0x10FFFFu || (u >= 0xD800u && u <= 0xDFFFu)) return bad;
    ok.value = u; ok.trap = A2X_TRAP_NONE;
    return ok;
}
"#;
