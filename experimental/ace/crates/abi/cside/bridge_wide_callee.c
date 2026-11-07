/*
 * bridge_wide_callee.c — PLAN-939 S07 wide typed-resource leg:
 * independent C CALLEE implementing the frozen wide families
 * (BufViewI64Sum / RecMixSum / WideFold). Compiled by the real MSVC
 * cl.exe into an independent object; a Cranelift-emitted wide CALLER
 * links against it (and vice versa in bridge_wide_caller.c). No code
 * shared with the Cranelift emitter or the Rust oracles.
 *
 * Freestanding: -GS- -Zl, no CRT, integer-only (no _fltneeded).
 */

#include "bridge_typed.h"

long long t939_wide_sum(const long long *p, unsigned long long n)
{
    unsigned long long i;
    unsigned long long acc = 0; /* C wraps; the family is total */
    for (i = 0; i < n; i++) {
        acc += (unsigned long long)p[i]; /* n counts ELEMENTS */
    }
    return (long long)acc;
}

long long t939_crec_sum(const RecMixT *r)
{
    /* struct field reads: the C compiler places flag/count/tag/big —
     * the weighted fold observes every field AT ITS LAYOUT OFFSET. */
    unsigned long long s;
    s = (unsigned long long)(long long)r->flag * 1000ULL;
    s += (unsigned long long)(long long)r->count * 3ULL;
    s += (unsigned long long)r->tag * 5ULL;
    s += (unsigned long long)r->big;
    return (long long)s;
}

long long t939_wide_fold(long long (*cb)(long long ctx, long long item),
                         long long ctx, const long long *items,
                         unsigned long long n)
{
    /* strictly synchronous per-call: the fnptr is used only inside this
     * frame and never stored anywhere */
    unsigned long long i;
    unsigned long long acc = 0;
    for (i = 0; i < n; i++) {
        acc += (unsigned long long)cb(ctx, items[i]);
    }
    return (long long)acc;
}
