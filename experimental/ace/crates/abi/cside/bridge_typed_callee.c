/*
 * bridge_typed_callee.c — PLAN-937 CA-D1 typed leg: independent C
 * CALLEE implementing the frozen typed families. Compiled by the real
 * MSVC cl.exe into an independent object; a Cranelift-emitted typed
 * CALLER links against it (and vice versa in bridge_typed_caller.c).
 * No code shared with the Cranelift emitter or the Rust oracles.
 *
 * Freestanding: -GS- -Zl, no CRT. This TU holds FP ops -> _fltused.
 */

#include "bridge_typed.h"

int _fltused = 0;

/* The fold is computed entirely in unsigned 64-bit (defined wrapping in
 * C); f64 folds by its raw bit pattern via memcpy-free union — the
 * ONLY reinterpretation, bit-exact, no value conversion. */
static unsigned long long t937_f64_bits(double v)
{
    union { double f; unsigned long long u; } cvt;
    cvt.f = v;
    return cvt.u;
}

long long t937_mix(int p0, unsigned int p1, long long p2,
                   unsigned long long p3, double p4, unsigned char p5,
                   unsigned int p6, long long p7)
{
    unsigned long long wa = (unsigned long long)(long long)p0 * T937_C0;
    unsigned long long wb = (unsigned long long)p1 * T937_C1;
    unsigned long long wc = (unsigned long long)p2 * T937_C2;
    unsigned long long wd = p3 * T937_C3;
    unsigned long long wf = t937_f64_bits(p4) * T937_C4;
    unsigned long long wg = (unsigned long long)p5 * T937_C5;
    unsigned long long wh = (unsigned long long)p6 * T937_C6;
    unsigned long long wi = (unsigned long long)p7 * T937_C7;

    unsigned long long acc = wa ^ wb;
    acc += wc;
    acc ^= wd;
    acc += wf;
    acc ^= wg;
    acc += wh;
    acc ^= wi;
    return (long long)acc;
}

int t937_div(void *ctx, long long n, long long d, long long *out)
{
    (void)ctx; /* nullable ctx — never dereferenced */
    if (d == 0) {
        return T937_DIV_ZERO; /* out untouched on failure */
    }
    /* wrapping semantics for INT64_MIN / -1 (no C UB, no trap): */
    if (d == -1) {
        *out = 0ULL - (unsigned long long)n;
        return T937_DIV_OK;
    }
    *out = n / d;
    return T937_DIV_OK;
}
