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
static long long ref_cb_reentrant(long long ctx, long long item) {
    static const uint8_t one[1] = { 'A' };
    uint64_t unit = t939_byte_length(one, 1);
    return ctx + item + (long long)unit;
}
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
#ifdef R939_S_callback_reentrant
    {
        /* PLAN-939 T-08 (V-07): a LIMITED LEGAL reentrant call — the
         * callback itself makes a foreign view call per item, mirroring
         * the deck exactly (nested foreign call DURING a foreign call,
         * same request, no stored callback). */
        t939_reset();
        int32_t out = -1;
        /* the mirror is EXACT: only the fold — its own callback makes
         * the nested byte_length calls (one per item). */
        R939_S_check(t939_fold(ref_cb_reentrant, 5LL, 3, &out) == 0);
        R939_S_check(out == 6 + 7 + 8);
        return 21;
    }
#elif defined(R939_S_txn3_commit)
    {
        /* PLAN-939 T-07 (V-04): the THREE-out transaction commits ALL
         * slots once in declaration order; both fresh handles read back
         * their payloads and each owner destroys exactly once. */
        t939_reset();
        Txn3OutA a;
        TxnResult b;
        int32_t out = -1, g1 = -1, g2 = -1, d1 = -1, d2 = -1, live = -1;
        a.mark = 0; a.handle = 0; b.value = 0; b.handle = 0;
        R939_S_check(t939_txn3(500, &a, &b, &out) == 0);
        R939_S_check(out == 500 && a.mark == 501 && b.value == 1000);
        R939_S_check(a.handle > 0 && b.handle > 0 && a.handle != b.handle);
        R939_S_check(t939_get(a.handle, &g1) == 0 && g1 == 501);
        R939_S_check(t939_get(b.handle, &g2) == 0 && g2 == 1000);
        R939_S_check(t939_destroy(a.handle, &d1) == 0);
        R939_S_check(t939_destroy(b.handle, &d2) == 0);
        live = t939_live();
        R939_S_check(live == 0);
        return (int)(out + a.mark + b.value + g1 + g2 + d1 + d2 + live); /* 3504 */
    }
#elif defined(R939_S_txn3_lastfail)
    {
        /* PLAN-939 T-07 (V-04): the contracted LAST-STEP failure — H1 is
         * created and destroyed by the ORIGINAL provider before -3;
         * EVERY caller byte stays untouched and live stays 0. */
        t939_reset();
        Txn3OutA a;
        TxnResult b;
        int32_t out = -1;
        a.mark = 7; a.handle = 7; b.value = 7; b.handle = 7;
        int32_t st = t939_txn3(1500, &a, &b, &out);
        R939_S_check(st == -3);
        R939_S_check(out == -1 && a.mark == 7 && a.handle == 7);
        R939_S_check(b.value == 7 && b.handle == 7);
        return 70; /* the frozen failure transport */
    }
#elif defined(R939_S_clone_owners)
    {
        /* PLAN-939 V-03 (the DECLARED clone hook): clone makes an
         * independent slot; BOTH owners get the payload and each
         * destroys exactly once; the ledger ends at zero live. */
        int64_t h = t939_create(4);
        int64_t g;
        int32_t a = -1, b = -1, d1 = -1, d2 = -1, live = -1;
        R939_S_check(h != 0);
        g = t939_clone(h);
        R939_S_check(g != 0 && g != h);
        R939_S_check(t939_get(h, &a) == 0 && a == 4);
        R939_S_check(t939_get(g, &b) == 0 && b == 4);
        R939_S_check(t939_destroy(h, &d1) == 0);
        R939_S_check(t939_destroy(g, &d2) == 0);
        live = t939_live();
        R939_S_check(live == 0);
        return (int)(a + b + d1 + d2 + live); /* 10 */
    }
#elif defined(R939_S_forged_handle)
    {
        /* PLAN-939 V-06: a well-formed encoding of a NEVER-allocated
         * slot — the provider rejects with ZERO state change. */
        int32_t out = 77;
        int32_t st = t939_get(65536, &out);
        R939_S_check(st == -1);
        R939_S_check(out == 77); /* untouched — zero provider state change */
        return 70; /* the frozen failure transport, like every failing deck */
    }
#elif defined(R939_S_stored_cb)
    {
        /* PLAN-939 V-14: the noescape observable — after a REAL fold
         * (mirrored exactly: the same provider call with the reference's
         * own callback), nothing was stored. */
        t939_reset();
        int32_t out = -1;
        R939_S_check(t939_fold(ref_cb_add, 5LL, 3, &out) == 0);
        R939_S_check(out == 18);
        R939_S_check(t939_stored_cb() == 0);
        return (int)(out * 10 + 0); /* 180 */
    }
#elif defined(R939_S_pairv_temp)
    {
        /* PLAN-939 V-02 (ctor temp as the CONST view lane): the call-site
         * temp Counter(5) is materialized once, viewed, consumed by the
         * provider, and dies with the call. Mirror: 20->21, out 21+5. */
        Counter x;
        Counter v;
        int32_t out = -1;
        x.value = 20;
        v.value = 5;
        int32_t st = t939_pairv(&x, &v, &out);
        R939_S_check(st == 0);
        R939_S_check(out == 26 && x.value == 21);
        return (int)(out + x.value); /* 47 */
    }
#elif defined(R939_S_utf8_owned)
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
        /* the deck's owned bystander (A + U+4E2D + U+1F600) stays live
         * across BOTH foreign calls and adds its scalar count (3). */
        return (int)n + 3;
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
