/*
 * bridge_wide_caller.c — PLAN-939 S07 wide typed-resource leg:
 * independent C CALLER PE (leg C -> native-typed). Its oracle is
 * computed with this file's OWN arithmetic — the frozen wide constants
 * live in bridge_typed.h (hand-maintained), and the expectations below
 * are hand-computed from them, NOT derived from any generator. Exits
 * with the number of FAILED checks (0 = pass).
 *
 * Freestanding: -GS- -Zl, no CRT, integer-only (no _fltneeded).
 */

#include "bridge_typed.h"

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

/* this file's OWN frozen callback (structurally independent spelling of
 * the frozen body (ctx ^ item) * 3 + item) */
static long long t939w_caller_cb(long long ctx, long long item)
{
    unsigned long long x = (unsigned long long)ctx ^ (unsigned long long)item;
    unsigned long long m = x * 3ULL;
    return (long long)(m + (unsigned long long)item);
}

static unsigned long long t939w_caller_sum(const long long *p,
                                           unsigned long long n)
{
    unsigned long long i, acc = 0;
    for (i = 0; i < n; i++) {
        acc += (unsigned long long)p[i];
    }
    return acc;
}

static int t939w_c_main(void)
{
    int failed = 0;
    static const long long items[8] = {
        T939W_I0, T939W_I1, T939W_I2, T939W_I3,
        T939W_I4, T939W_I5, T939W_I6, T939W_I7,
    };

    /* 1) RUNTIME layout probe — independent offsetof evidence (V-01):
     * the S07 record closure's field offsets measured by the real C
     * compiler, at run time, in the caller PE. (size/align are pinned
     * compile-time by the header's negative-array typedefs — comparing
     * them here would be C4127 constant conditions under -W4 -WX.) */
    {
        RecMixT probe;
        unsigned long long o_flag, o_count, o_tag, o_big;
        probe.flag = 1u;
        probe.count = 2;
        probe.tag = 3u;
        probe.big = 4;
        o_flag = (unsigned long long)((char *)&probe.flag - (char *)&probe);
        o_count = (unsigned long long)((char *)&probe.count - (char *)&probe);
        o_tag = (unsigned long long)((char *)&probe.tag - (char *)&probe);
        o_big = (unsigned long long)((char *)&probe.big - (char *)&probe);
        if (o_flag != 0u || o_count != 4u || o_tag != 8u || o_big != 16u) {
            failed++;
        }
    }

    /* 2) wide_sum: frozen 8-element fold + ELEMENT-COUNT prefixes + the
     * empty view, against BOTH the frozen constants and this file's own
     * element walk. A byte-unit misread of n observably fails. */
    {
        long long got8 = t939_wide_sum(items, 8u);
        long long got3 = t939_wide_sum(items, 3u);
        long long got1 = t939_wide_sum(items, 1u);
        long long got0 = t939_wide_sum(items, 0u);
        if (got8 != T939W_SUM_8 || got8 != (long long)t939w_caller_sum(items, 8u)) {
            failed++;
        }
        if (got3 != T939W_SUM_3 || got3 != (long long)t939w_caller_sum(items, 3u)) {
            failed++;
        }
        if (got1 != T939W_SUM_1) {
            failed++;
        }
        if (got0 != 0) {
            failed++; /* empty view: legal control, total 0 */
        }
    }

    /* 3) crec_sum: the frozen record (U+10FFFF tag boundary, wide big
     * near MIN) + a second all-zero-tag-'A' record, against the frozen
     * constant and this file's own fold. */
    {
        RecMixT r;
        long long got;
        r.flag = (unsigned char)T939W_REC_FLAG;
        r.count = T939W_REC_COUNT;
        r.tag = T939W_REC_TAG;
        r.big = T939W_REC_BIG;
        got = t939_crec_sum(&r);
        if (got != T939W_REC) {
            failed++;
        }
        if (got != (long long)(((unsigned long long)(long long)r.flag * 1000ULL)
                               + ((unsigned long long)(long long)r.count * 3ULL)
                               + ((unsigned long long)r.tag * 5ULL)
                               + (unsigned long long)r.big)) {
            failed++;
        }
        r.flag = 0u;
        r.count = 0;
        r.tag = 0x41u;
        r.big = 0;
        if (t939_crec_sum(&r) != 0x41LL * 5LL) {
            failed++;
        }
    }

    /* 4) wide_fold: THIS file's callback travels as the fnptr (the
     * native callee invokes back through it); frozen wide ctx, full
     * fold + prefix + the empty fold (callback invoked zero times). */
    {
        long long got8 = t939_wide_fold(t939w_caller_cb, T939W_FOLD_CTX, items, 8u);
        long long got3 = t939_wide_fold(t939w_caller_cb, T939W_FOLD_CTX, items, 3u);
        long long got0 = t939_wide_fold(t939w_caller_cb, T939W_FOLD_CTX, items, 0u);
        if (got8 != T939W_FOLD_8) {
            failed++;
        }
        if (got3 != T939W_FOLD_3) {
            failed++;
        }
        if (got0 != 0) {
            failed++;
        }
        /* the callback body itself: cb(ctx, 0) against the frozen
         * constant (the ctx-only term, observable on its own). */
        if (t939w_caller_cb(T939W_FOLD_CTX, 0LL) != T939W_CB_AT_0) {
            failed++;
        }
    }

    return failed;
}

void t939w_c_entry(void)
{
    ExitProcess((unsigned int)t939w_c_main());
}
