/*
 * interop_callee.c — PLAN-797 T-03 leg B C callee suite.
 *
 * Every function is hand-written C implementing the frozen suite
 * semantics (interop_v02.h header comment). Compiled by the real MSVC
 * cl.exe into an independent object; an AutoABI caller object links
 * against it. No code is shared with the Cranelift emitter or the Rust
 * oracles.
 */

#include "interop_v02.h"

int _fltused = 0; /* this TU holds FP operations */

long long p797_cmix(int a, int b, int c, int d, long long e, double f,
                    int g, double h)
{
    return e * 2LL
         + (long long)a * 1LL
         + (long long)b * 3LL
         + (long long)c * 7LL
         + (long long)d * 11LL
         + (long long)g * 13LL
         + (long long)f * 1000LL
         + (long long)h * 17LL;
}

double p797_cfacc(double a, double b)
{
    return a * 2.5 + b * 1.25;
}

int p797_cdiv(long long n, long long d, long long *q)
{
    if (d == 0) {
        return P797_DIV_ZERO;
    }
    *q = n / d;
    return 0;
}

long long p797_crec_sum(const RecMix *r)
{
    return (long long)r->flag * 1000LL
         + (long long)r->count * 3LL
         + (long long)r->tag * 5LL
         + r->big;
}

static long long g_c_counter = 0;

long long p797_cbump(int x)
{
    g_c_counter += (long long)x;
    return g_c_counter;
}
