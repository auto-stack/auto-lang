//! PLAN-926 CA-C: the C11 COLLECTIONS runtime header text (a versioned
//! text asset of the provider, marks-parameterized like the scalar and
//! record-text ones).
//!
//! Semantics mirror the Rust collections runtime (a2x-rust/src/coll.rs,
//! PLAN-904 T-02 freeze) exactly — validated against the real MSVC
//! toolchain by the PLAN-926 T-02 spike before implementation:
//! * traps add `ListIndex` / `VariantBranch` / `Budget` / `Cancelled` to
//!   the record-text set; the four failure classes stay strictly
//!   separated (a parent kill never masquerades as cooperative cleanup);
//! * the work budget is charged BEFORE the work it pays for (frozen
//!   table: 2 WU per list element slot, 1 WU per 8 copied Str bytes,
//!   1 WU per record/variant ctor, 4 WU per call frame, 1 WU per loop
//!   back-edge poll); default budget 2^24;
//! * `A2X_RT_FAULT_ALLOC=<n>` fails the n-th REAL allocation request
//!   (typed `Resource`, no partial commit) BEFORE the request — a faulted
//!   attempt is not an allocation (no buffer, no event, no bytes);
//! * `A2X_RT_CANCEL=1` arms cooperative cancellation, observed ONLY at
//!   `a2x_coll_poll` (loop back-edges);
//! * `A2X_RT_EVENTS=1` mirrors logical events and real backing requests
//!   to stderr (`coll:*` lines) — listed separately, never a fake
//!   alloc/free;
//! * owned `Str` is `a2x_str` (one buffer); the ONLY release path is the
//!   explicit `a2x_str_free` the lowering emits at verifier-scheduled
//!   points; `len` counts Unicode scalars, `len_bytes` UTF-8 bytes;
//! * cleanup is ZERO-charge / zero-poll / zero-allocation: free paths
//!   never bill, never cancel, never allocate;
//! * live-accounting observers (`a2x_coll_live_*`) are test observation
//!   only — helper probes verify cleanup by these counters, not by logs;
//! * the deep-link witness chain (`a2x_deep_link`) proves iterative
//!   (no-recursion) deep push/free/clone/len with mid-fault rollback —
//!   the same generated runtime a certified probe compiles against.
//!
//! The typed list/variant/record helpers are NOT here: they are nominal,
//! seed-derived, and live in `generated.c` (see
//! [`crate::provider::coll_type_table_block`]). Everything in this header
//! is family-agnostic and `static` (single-TU discipline: only
//! `generated.c`'s copy is ever exercised).

/// The collections runtime header for a view: the frozen collections
/// core + the marks the view actually uses (content-hashed into every
/// AXC3 fragment key, so a runtime change can never hit stale fragments).
pub fn collections_runtime_source(marks: &[&'static str]) -> String {
    let mut s = String::new();
    s.push_str("// AUTO-GENERATED typed collections runtime (PLAN-926 CA-C) — DO NOT EDIT.\n");
    s.push_str("// Checked i32 core + owned-Str/list/billing layer with explicit cleanup,\n");
    s.push_str("// pre-allocation budget checks and default-off fault/observation hooks.\n");
    s.push_str("#ifndef A2X_C_COLL_RUNTIME_H\n#define A2X_C_COLL_RUNTIME_H\n\n");
    s.push_str("#include <stdbool.h>\n#include <stddef.h>\n#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\n");
    s.push_str(COLL_CORE);
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
    s.push_str("\n#endif /* A2X_C_COLL_RUNTIME_H */\n");
    s
}

/// Frozen runtime core (byte-stable for a crate version).
const COLL_CORE: &str = r#"typedef enum {
    A2X_TRAP_NONE = 0,
    A2X_TRAP_OVERFLOW,
    A2X_TRAP_DIV_BY_ZERO,
    A2X_TRAP_DIV_OVERFLOW,
    A2X_TRAP_REM_BY_ZERO,
    A2X_TRAP_REM_OVERFLOW,
    A2X_TRAP_INVALID_SCALAR,
    A2X_TRAP_TEXT_INDEX,
    A2X_TRAP_RESOURCE,
    A2X_TRAP_LIST_INDEX,
    A2X_TRAP_VARIANT_BRANCH,
    A2X_TRAP_BUDGET,
    A2X_TRAP_CANCELLED
} a2x_trap_code;

static const char* a2x_trap_name(a2x_trap_code t) {
    static const char* const names[13] = {
        "None", "Overflow", "DivByZero", "DivOverflow", "RemByZero",
        "RemOverflow", "InvalidScalar", "TextIndex", "Resource",
        "ListIndex", "VariantBranch", "Budget", "Cancelled"
    };
    return names[(int)t];
}

typedef struct { int32_t value; a2x_trap_code trap; } a2x_result;
typedef struct { bool value; a2x_trap_code trap; } a2x_bool_result;

static inline a2x_result a2x_value(int32_t v) {
    a2x_result r; r.value = v; r.trap = A2X_TRAP_NONE; return r;
}
static inline a2x_result a2x_fwd_i32(a2x_trap_code t) {
    a2x_result r; r.value = 0; r.trap = t; return r;
}
static inline a2x_bool_result a2x_fwd_bool(a2x_trap_code t) {
    a2x_bool_result r; r.value = false; r.trap = t; return r;
}
static inline a2x_bool_result a2x_bool_value(bool v) {
    a2x_bool_result r; r.value = v; r.trap = A2X_TRAP_NONE; return r;
}

/* ---- checked i32 (compute in int64_t, range-check, THEN narrow) ------ */

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

/* ---- work budget / cooperative cancellation / fault hooks ------------
 * Frozen billing table (PLAN-904 T-02, C freeze PLAN-926 T-02): 2 WU per
 * list element slot, 1 WU per 8 copied Str bytes, 1 WU per ctor, 4 WU per
 * call frame, 1 WU per loop back-edge poll. Budget default 2^24;
 * A2X_RT_BUDGET overrides; A2X_RT_CANCEL=1 arms cancellation (observed at
 * polling points only). Cleanup paths never bill, never poll, never
 * allocate. Knobs are read lazily from the environment exactly once — the
 * emitted file set is identical in daily and certified runs. */

#define A2X_COLL_DEFAULT_BUDGET 16777216 /* 2^24 */

static long a2x_coll_cfg_state = 0; /* 0 = uninitialized, 1 = read */
static long a2x_coll_cfg_fault = 0; /* 1-based ordinal to fail, <=0 off */
static int a2x_coll_cfg_events = 0;
static int a2x_coll_cfg_cancel = 0;
static long a2x_coll_work = 0; /* negative remaining, pre-deducted */

static void a2x_coll_config(void) {
    if (a2x_coll_cfg_state == 0) {
        const char* fault = getenv("A2X_RT_FAULT_ALLOC");
        const char* events = getenv("A2X_RT_EVENTS");
        const char* budget = getenv("A2X_RT_BUDGET");
        const char* cancel = getenv("A2X_RT_CANCEL");
        if (fault != NULL) {
            long n = 0; int ok = 0;
            while (*fault >= '0' && *fault <= '9') { n = n * 10 + (long)(*fault - '0'); ok = 1; fault++; }
            if (ok && *fault == '\0' && n > 0) a2x_coll_cfg_fault = n;
        }
        if (events != NULL && events[0] == '1' && events[1] == '\0') a2x_coll_cfg_events = 1;
        if (budget != NULL && budget[0] != '\0') {
            long n = 0; int ok = 0;
            while (*budget >= '0' && *budget <= '9') { n = n * 10 + (long)(*budget - '0'); ok = 1; budget++; }
            if (ok && *budget == '\0' && n > 0) a2x_coll_work = -n;
        }
        if (a2x_coll_work == 0) a2x_coll_work = -A2X_COLL_DEFAULT_BUDGET;
        if (cancel != NULL && cancel[0] == '1' && cancel[1] == '\0') a2x_coll_cfg_cancel = 1;
        a2x_coll_cfg_state = 1;
    }
}

/* Deduct work units at a real work site, BEFORE the allocation/commit it
 * pays for; exhaustion is the typed Budget trap. */
static a2x_trap_code a2x_coll_charge(long units) {
    a2x_coll_config();
    a2x_coll_work += units;
    if (a2x_coll_cfg_events) fprintf(stderr, "coll:charge:%ld\n", units);
    if (a2x_coll_work > 0) return A2X_TRAP_BUDGET;
    return A2X_TRAP_NONE;
}

/* Cooperative cancellation point (loop back-edges; 1 WU each). */
static a2x_trap_code a2x_coll_poll(void) {
    a2x_trap_code t = a2x_coll_charge(1);
    if (t != A2X_TRAP_NONE) return t;
    if (a2x_coll_cfg_cancel) return A2X_TRAP_CANCELLED;
    return A2X_TRAP_NONE;
}

/* Call-frame charge (4 WU; callee prologue). */
static a2x_trap_code a2x_coll_charge_call(void) {
    return a2x_coll_charge(4);
}

/* Record/variant construction charge (1 WU). */
static a2x_trap_code a2x_coll_charge_ctor(void) {
    return a2x_coll_charge(1);
}

/* Str copy charge (1 WU per 8 bytes, rounded up, at least 1). */
static a2x_trap_code a2x_coll_charge_str_bytes(size_t bytes) {
    long units = ((long)bytes + 7) / 8;
    if (units < 1) units = 1;
    return a2x_coll_charge(units);
}

/* ---- live accounting (test observation only) ------------------------- */

static long a2x_coll_st_alloc_ordinal = 0;
static long a2x_coll_st_free_count = 0;
static long a2x_coll_st_live_nodes = 0;
static long a2x_coll_st_live_str_bytes = 0;
static long a2x_coll_st_live_backing_bytes = 0;

static long a2x_coll_alloc_count(void) {
    a2x_coll_config();
    return a2x_coll_st_alloc_ordinal;
}
static long a2x_coll_live_nodes(void) { return a2x_coll_st_live_nodes; }
static long a2x_coll_live_str_bytes(void) { return a2x_coll_st_live_str_bytes; }
static long a2x_coll_live_backing_bytes(void) { return a2x_coll_st_live_backing_bytes; }
static long a2x_coll_free_total(void) { return a2x_coll_st_free_count; }

/* ---- allocation ledger (default-off fault/observation hooks) ----------
 * A faulted attempt is NOT an allocation: no buffer exists, no event. */

static a2x_trap_code a2x_coll_alloc_gate(size_t bytes) {
    long n;
    a2x_coll_config();
    n = ++a2x_coll_st_alloc_ordinal;
    if (a2x_coll_cfg_fault == n) {
        if (a2x_coll_cfg_events) fprintf(stderr, "coll:fault:%llu\n", (unsigned long long)bytes);
        return A2X_TRAP_RESOURCE;
    }
    if (a2x_coll_cfg_events) fprintf(stderr, "coll:alloc:%llu\n", (unsigned long long)bytes);
    a2x_coll_st_live_backing_bytes += (long)bytes;
    return A2X_TRAP_NONE;
}

static void a2x_coll_backing_release(size_t bytes) {
    a2x_coll_config();
    a2x_coll_st_free_count++;
    a2x_coll_st_live_backing_bytes -= (long)bytes;
    if (a2x_coll_cfg_events) fprintf(stderr, "coll:free-backing:%llu\n", (unsigned long long)bytes);
}

/* ---- owned Str layer (billing + fault before the real request) -------- */

typedef struct { uint8_t* data; size_t len; } a2x_str;

/* result wrappers of the owned-Str channel (the per-record/variant ones
 * are generated into generated.c next to their structs). */
typedef struct { a2x_str value; a2x_trap_code trap; } a2x_stro_result;
static inline a2x_stro_result a2x_stro_value(a2x_str v) {
    a2x_stro_result r; r.value = v; r.trap = A2X_TRAP_NONE; return r;
}
static inline a2x_stro_result a2x_fwd_stro(a2x_trap_code t) {
    a2x_stro_result r; r.value.data = NULL; r.value.len = 0; r.trap = t; return r;
}

static a2x_trap_code a2x_str_alloc(size_t len, const uint8_t* fill, a2x_str* out) {
    a2x_trap_code t;
    uint8_t* p;
    out->data = NULL; out->len = 0;
    t = a2x_coll_charge_str_bytes(len);
    if (t != A2X_TRAP_NONE) return t;
    t = a2x_coll_alloc_gate(len);
    if (t != A2X_TRAP_NONE) return t;
    p = (uint8_t*)malloc(len == 0 ? 1 : len);
    if (p == NULL) { a2x_coll_st_live_backing_bytes -= (long)len; return A2X_TRAP_RESOURCE; }
    if (len > 0 && fill != NULL) memcpy(p, fill, len);
    out->data = p; out->len = len;
    a2x_coll_st_live_str_bytes += (long)len;
    return A2X_TRAP_NONE;
}

/* Literal promotion: an owned copy of a static literal. */
static a2x_trap_code a2x_str_from_lit(const uint8_t* lit, size_t len, a2x_str* out) {
    return a2x_str_alloc(len, lit, out);
}
/* Independent owned copy (s.clone()). */
static a2x_trap_code a2x_str_clone(const uint8_t* data, size_t len, a2x_str* out) {
    return a2x_str_alloc(len, data, out);
}

/* PLAN-964 (L-22/FStr): concatenation into a fresh owned buffer. */
static a2x_trap_code a2x_str_concat(const a2x_str* l, const a2x_str* r, a2x_stro_result* out) {
    a2x_trap_code t;
    size_t total;
    if (l->len > SIZE_MAX - r->len) { out->trap = A2X_TRAP_RESOURCE; out->value.data = NULL; out->value.len = 0; return A2X_TRAP_RESOURCE; }
    total = l->len + r->len;
    t = a2x_str_alloc(total, NULL, &out->value);
    if (t != A2X_TRAP_NONE) { out->trap = t; return t; }
    if (l->len > 0) memcpy(out->value.data, l->data, l->len);
    if (r->len > 0) memcpy(out->value.data + l->len, r->data, r->len);
    out->trap = A2X_TRAP_NONE;
    return A2X_TRAP_NONE;
}

/* PLAN-964 (L-22/FStr): render an i32 as a decimal owned Str. */
static a2x_trap_code a2x_str_display_i32(int32_t v, a2x_stro_result* out) {
    char tmp[12];
    int32_t n = v;
    size_t len = 0;
    if (n == INT32_MIN) {
        const char* lit = "-2147483648";
        while (lit[len] != 0) { tmp[len] = lit[len]; len++; }
    } else {
        int neg = n < 0 ? 1 : 0;
        uint32_t m = neg ? (uint32_t)(-(int64_t)n) : (uint32_t)n;
        char digits[11];
        int nd = 0;
        do { digits[nd++] = (char)('0' + (m % 10)); m /= 10; } while (m != 0);
        if (neg) tmp[len++] = '-';
        while (nd > 0) tmp[len++] = digits[--nd];
    }
    out->trap = a2x_str_alloc(len, (const uint8_t*)tmp, &out->value);
    return out->trap;
}

/* Explicit free of an owned buffer — the ONLY release path. Zero-charge,
 * zero-poll, zero-allocation. */
static void a2x_str_free(a2x_str* s) {
    if (s != NULL && s->data != NULL) {
        a2x_coll_st_live_str_bytes -= (long)s->len;
        a2x_coll_backing_release(s->len);
        free(s->data);
        s->data = NULL; s->len = 0;
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

/* Unicode scalar count (the `len` semantics of the collections domain). */
static a2x_result a2x_str_len(const a2x_str* s) {
    size_t scalars = 0, i = 0;
    int64_t n;
    while (i < s->len) {
        size_t step = a2x_utf8_step(s->data[i]);
        if (step == 0 || i + step > s->len) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_TEXT_INDEX; return r; }
        i += step; scalars++;
    }
    n = (int64_t)scalars;
    if (n > INT32_MAX) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_RESOURCE; return r; }
    return a2x_value((int32_t)n);
}

/* UTF-8 byte length. */
static a2x_result a2x_str_len_bytes(const a2x_str* s) {
    int64_t n = (int64_t)s->len;
    if (n > INT32_MAX) { a2x_result r; r.value = 0; r.trap = A2X_TRAP_RESOURCE; return r; }
    return a2x_value((int32_t)n);
}

/* Full-byte equality (no normalization; NUL included). */
static bool a2x_str_eq(const a2x_str* l, const a2x_str* r) {
    return l->len == r->len && (l->len == 0 || memcmp(l->data, r->data, l->len) == 0);
}
static bool a2x_str_ne(const a2x_str* l, const a2x_str* r) {
    return !a2x_str_eq(l, r);
}

/* ---- iterative deep-link witness chain (helper probes) ----------------
 * No recursion anywhere: push/free/clone/len are explicit work loops; the
 * n-th payload failure frees the partially cloned prefix (rollback); free
 * works under any stack depth (the certified 3072-level / 256 KiB probe
 * compiles against THIS generated runtime). */

typedef struct a2x_deep_link {
    a2x_str payload;
    struct a2x_deep_link* next;
} a2x_deep_link;

static a2x_trap_code a2x_deep_push(a2x_deep_link** head, const uint8_t* lit, size_t len) {
    a2x_deep_link* node = (a2x_deep_link*)malloc(sizeof *node);
    a2x_trap_code t;
    if (node == NULL) return A2X_TRAP_RESOURCE;
    node->next = *head;
    node->payload.data = NULL; node->payload.len = 0;
    t = a2x_str_from_lit(lit, len, &node->payload);
    if (t != A2X_TRAP_NONE) { free(node); return t; }
    *head = node;
    return A2X_TRAP_NONE;
}

static void a2x_deep_free(a2x_deep_link** head) {
    while (*head != NULL) {
        a2x_deep_link* n = *head;
        *head = n->next;
        a2x_str_free(&n->payload);
        free(n);
    }
}

static a2x_trap_code a2x_deep_clone(const a2x_deep_link* head, a2x_deep_link** out) {
    const a2x_deep_link* cur;
    a2x_deep_link* acc = NULL;
    *out = NULL;
    for (cur = head; cur != NULL; cur = cur->next) {
        a2x_deep_link* node = (a2x_deep_link*)malloc(sizeof *node);
        a2x_trap_code t;
        if (node == NULL) { a2x_deep_free(&acc); return A2X_TRAP_RESOURCE; }
        node->payload.data = NULL; node->payload.len = 0;
        t = a2x_str_clone(cur->payload.data, cur->payload.len, &node->payload);
        if (t != A2X_TRAP_NONE) { free(node); a2x_deep_free(&acc); return t; }
        node->next = acc;
        acc = node;
    }
    *out = acc;
    return A2X_TRAP_NONE;
}

static long a2x_deep_len(const a2x_deep_link* head) {
    long n = 0;
    const a2x_deep_link* cur;
    for (cur = head; cur != NULL; cur = cur->next) n++;
    return n;
}
"#;
