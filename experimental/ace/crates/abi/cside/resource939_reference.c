/*
 * resource939_reference.c — PLAN-939 independent oracle caller (the
 * REFERENCE leg). Hand-written expectations per scenario; exits non-zero
 * on the first divergence. Compiled and run separately from any
 * generated code. Each scenario mirrors ONE deck's exact provider call
 * sequence (the parent compares actual vs reference events).
 *
 * The scenario is selected by a single -DR939_S_<deck> define supplied by
 * the certification worker's policy.
 */
#include "resource939.h"
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#ifndef R939_S_check
#define R939_S_check(cond) do { if (!(cond)) { fprintf(stderr, "reference939 FAIL %s:%d\n", __FILE__, __LINE__); exit(70); } } while (0)
#endif

/* S06 oracle callbacks: hand-mirrors of the deck's source-level `cb`
 * functions (ctx passes through unchanged; the fail variant returns the
 * reviewed -7 status at item 2). */
static long long ref_cb_add(long long ctx, long long item) {
    return ctx + item;
}
static long long ref_cb_fail(long long ctx, long long item) {
    if (item == 2) {
        return -7;
    }
    return ctx + item;
}

/* PLAN-939 V-01 layout oracle (field ORDER/offset evidence): the header's
 * compile-time asserts pin size/alignment with builtin operators only;
 * this runtime probe completes the picture with the REAL offsetof — the
 * byte view of a typed store proves declaration-order offsets on the
 * wire (little-endian target, frozen by profile). Runs before every
 * scenario: no certified reference leg executes without it. */
static void layout_selfcheck(void) {
    /* auto variables keep the conditions non-constant (C4127 under /W4) */
    unsigned long long off_c_v = offsetof(Counter, value);
    unsigned long long off_t_v = offsetof(TxnResult, value);
    unsigned long long off_t_h = offsetof(TxnResult, handle);
    R939_S_check(off_c_v == 0);
    R939_S_check(off_t_v == 0 && off_t_h == 4);
    union {
        TxnResult t;
        unsigned char b[8];
    } u;
    u.t.value = 0x01020304;   /* little-endian bytes: 04 03 02 01 */
    u.t.handle = 0x05060708;  /* little-endian bytes: 08 07 06 05 */
    R939_S_check(u.b[0] == 0x04 && u.b[3] == 0x01);
    R939_S_check(u.b[4] == 0x08 && u.b[7] == 0x05);
}

int main(void) {
    layout_selfcheck();
#ifdef R939_S_utf8_owned
    {
        /* PLAN-939 V-02 (owned temp -> view): the deck's owned Str
         * local feeds BOTH views; the owner outlives every consumer and
         * is freed exactly once at exit (the text plan). Mirror: byte
         * length 9 (NUL is data), then scalar count 3. */
        const uint8_t text[] = { 'A', 0x00, 0xE4, 0xb8, 0xad, 0xf0, 0x9f, 0x98, 0x80 };
        uint64_t n = t939_byte_length(text, 9);
        R939_S_check(n == 9);
        /* NUL is DATA and a valid scalar: A, U+0000, U+4E2D, U+1F600. */
        n = t939_scalar_count(text, 9);
        R939_S_check(n == 4);
        return (int)n;
    }
#elif defined(R939_S_utf8_bytes)
    {
        const uint8_t text[] = { 'A', 0xE4, 0xb8, 0xad, 0xf0, 0x9f, 0x98, 0x80 };
        uint64_t n = t939_byte_length(text, 8);
        R939_S_check(n == 8);
        return (int)n;
    }
#elif defined(R939_S_utf8_scalars)
    {
        const uint8_t text[] = { 'A', 0xE4, 0xb8, 0xad, 0xf0, 0x9f, 0x98, 0x80 };
        uint64_t n = t939_scalar_count(text, 8);
        R939_S_check(n == 3);
        return (int)n;
    }
#elif defined(R939_S_record_mut)
    {
        Counter c;
        int32_t out = -1;
        c.value = 20;
        R939_S_check(t939_bump(&c, 1, &out) == 0);
        R939_S_check(c.value == 21 && out == 21);
        return out + c.value; /* 42 */
    }
#elif defined(R939_S_record_mut_fail)
    {
        Counter c;
        int32_t out = -1;
        c.value = 20;
        R939_S_check(t939_bump(&c, -1, &out) == -2);
        R939_S_check(c.value == 20 && out == -1);
        return 70; /* the frozen failure transport */
    }
#elif defined(R939_S_handle_lifecycle)
    {
        t939_reset();
        int64_t h = t939_create(4);
        R939_S_check(h > 0 && h <= 0x7fffffff);
        int32_t got = -1;
        R939_S_check(t939_get(h, &got) == 0 && got == 4);
        int32_t gen = -1;
        R939_S_check(t939_destroy(h, &gen) == 0 && gen >= 1);
        R939_S_check(t939_live() == 0);
        return got + gen; /* 4 + 1 */
    }
#elif defined(R939_S_handle_stale)
    {
        t939_reset();
        int64_t h = t939_create(4);
        int32_t gen = -1;
        R939_S_check(t939_destroy(h, &gen) == 0);
        int32_t got = -1;
        R939_S_check(t939_get(h, &got) == -1);
        return 70; /* stale handle: the frozen failure transport */
    }
#elif defined(R939_S_txn_commit)
    {
        t939_reset();
        TxnResult r;
        int32_t out = -1;
        r.value = 0;
        r.handle = 0;
        R939_S_check(t939_txn(7, &r, &out) == 0);
        R939_S_check(out == 7 && r.value == 14 && r.handle > 0);
        return out + r.value; /* 21 */
    }
#elif defined(R939_S_txn_seed_fail)
    {
        t939_reset();
        TxnResult r;
        int32_t out = -1;
        r.value = 5;
        r.handle = 6;
        R939_S_check(t939_txn(3000, &r, &out) == -2);
        R939_S_check(out == -1 && r.value == 5 && r.handle == 6);
        return 70;
    }
#elif defined(R939_S_txn_laststep_fail)
    {
        t939_reset();
        TxnResult r;
        int32_t out = -1;
        r.value = 5;
        r.handle = 6;
        R939_S_check(t939_txn(1500, &r, &out) == -3);
        R939_S_check(out == -1 && r.value == 5 && r.handle == 6);
        /* The oracle mirrors the deck's EXACT call sequence (the source
         * program dies in the failure transport) — the transient handle's
         * original-party reclamation is a provider-internal fact, proven
         * by the T-07 fault sweep's dedicated ledger probes, not by a
         * post-mortem probe the source cannot make. */
        return 70;
    }
#elif defined(R939_S_callback)
    {
        t939_reset();
        int32_t out = -1;
        R939_S_check(t939_fold(ref_cb_add, 5LL, 3, &out) == 0);
        R939_S_check(out == 5 + 6 + 7);
        return 18;
    }
#elif defined(R939_S_callback_fail)
    {
        t939_reset();
        int32_t out = -1;
        R939_S_check(t939_fold(ref_cb_fail, 5LL, 3, &out) == -7);
        R939_S_check(out == -1); /* the failure leaves the out slot untouched */
        return 70;
    }
#elif defined(R939_S_alias_pair_ok)
    {
        t939_reset();
        Counter a;
        Counter b;
        int32_t out = -1;
        a.value = 20;
        b.value = 30;
        R939_S_check(t939_pair(&a, &b, &out) == 0);
        R939_S_check(a.value == 21 && b.value == 32 && out == 53);
        return 53;
    }
#elif defined(R939_S_alias_pairv_ok)
    {
        t939_reset();
        Counter r;
        Counter v;
        int32_t out = -1;
        r.value = 10;
        v.value = 5;
        R939_S_check(t939_pairv(&r, &v, &out) == 0);
        R939_S_check(r.value == 11 && v.value == 5 && out == 16);
        return 16;
    }
#else
    fprintf(stderr, "reference939: unknown scenario\n");
    return 71;
#endif
    /* exactly one scenario is compiled in and every branch returns */
}
