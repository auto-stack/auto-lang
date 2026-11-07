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
#include <stdio.h>
#include <stdlib.h>

#ifndef R939_S_check
#define R939_S_check(cond) do { if (!(cond)) { fprintf(stderr, "reference939 FAIL %s:%d\n", __FILE__, __LINE__); exit(70); } } while (0)
#endif

int main(void) {
#ifdef R939_S_utf8_bytes
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
#else
    fprintf(stderr, "reference939: unknown scenario\n");
    return 71;
#endif
    /* exactly one scenario is compiled in and every branch returns */
}
