/*
 * interop_caller.c — PLAN-797 T-03 leg C C caller suite.
 *
 * Independent C entry + oracle: this TU DEFINES p797_inner (the AutoABI
 * callee's nested import lands back in C), calls every AutoABI suite
 * export with the frozen inputs, checks each result against its OWN
 * hand-written arithmetic, accumulates violation flags, and exits with
 * the flag count. Exit 0 = every interop check passed on the C side.
 *
 * Freestanding: kernel32 only; _fltused defined here (FP literals).
 */

#include "interop_v02.h"

int _fltused = 0;

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

/* The nested-call target the AutoABI callee imports. */
long long p797_inner(long long x)
{
    return x * 3LL + 11LL;
}

static int flags = 0;
#define CHECK(cond) do { if (!(cond)) { flags++; } } while (0)

static long long c_oracle_mix(void)
{
    return P797_E * 2LL + (long long)P797_A * 1LL + (long long)P797_B * 3LL
         + (long long)P797_C * 7LL + (long long)P797_D * 11LL
         + (long long)P797_G * 13LL + (long long)P797_F * 1000LL
         + (long long)P797_H * 17LL;
}

static int p797_c_suite_main(void)
{
    long long got_i;
    double got_d;
    int got_s;
    long long out_q;
    RecMix rec;

    /* heterogeneous mix */
    got_i = p797_mix(P797_A, P797_B, P797_C, P797_D, P797_E, P797_F,
                     P797_G, P797_H);
    CHECK(got_i == c_oracle_mix());

    /* f64 return path (exact-representable, == is the oracle) */
    got_d = p797_facc(1.5, -0.75);
    CHECK(got_d == 1.5 * 2.5 + (-0.75) * 1.25);

    /* nested: AutoABI outer -> p797_inner (defined HERE, in C) */
    got_i = p797_outer(13, 2.25);
    CHECK(got_i == p797_inner(13LL * 7LL + 2LL * 100LL) + 13LL);

    /* fallible: success commits out */
    out_q = 0;
    got_s = p797_div(0, -100LL, 7LL, &out_q);
    CHECK(got_s == 0);
    CHECK(out_q == -100LL / 7LL);

    /* fallible: div-by-zero keeps the sentinel AND reports status -1 */
    out_q = 47000LL;
    got_s = p797_div(0, 12345LL, 0LL, &out_q);
    CHECK(got_s == P797_DIV_ZERO);
    CHECK(out_q == 47000LL);

    /* record view: build the wire record in C, offsets matter */
    rec.flag = 1;
    rec.count = -7;
    rec.tag = 1001;
    rec.big = 99999999999LL;
    got_i = p797_rec_sum(&rec);
    CHECK(got_i == 1LL * 1000LL + (-7LL) * 3LL + 1001LL * 5LL
                        + 99999999999LL);

    /* observable counter: five calls, last returns the cumulative */
    p797_bump(3);
    p797_bump(-1);
    p797_bump(10);
    p797_bump(0);
    got_i = p797_bump(7);
    CHECK(got_i == 19LL);

    return flags;
}

void p797_c_suite_entry(void)
{
    ExitProcess(p797_c_suite_main());
}
