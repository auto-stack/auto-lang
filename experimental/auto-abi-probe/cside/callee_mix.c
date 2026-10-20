/*
 * callee_mix.c — PLAN-797 T-01 leg B: AutoABI caller -> independent C
 * callee. Compiled by a real, separately located MSVC cl.exe (never the
 * Cranelift emitter and never a Rust extern "C" shim). The body below is
 * written by hand from the frozen contract; it shares no code generator
 * with the AutoABI side.
 */

#include "autoabi_v01.h"

long long p797_mix(int a, int b, int c, int d, long long e, double f,
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
