/*
 * bridge_typed_caller.c — PLAN-937 CA-D1 typed leg: independent C
 * CALLER PE (leg C->AutoABI-typed). Its oracle is computed with this
 * file's own arithmetic — the fold constants live in bridge_typed.h
 * (hand-maintained), and the expectations below are hand-computed from
 * them, NOT derived from any generator. Exits with the number of
 * FAILED checks (0 = pass).
 *
 * Freestanding: no CRT. _fltused defined here (FP literals in this TU).
 */

#include "bridge_typed.h"

int _fltused = 0;

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

/* local fold — this file's OWN implementation, structurally different
 * (accumulating array walk) from bridge_typed_callee.c's expression. */
static long long t937_caller_fold(int p0, unsigned int p1, long long p2,
                                  unsigned long long p3, double p4,
                                  unsigned char p5, unsigned int p6,
                                  long long p7)
{
    unsigned long long lanes[8];
    lanes[0] = (unsigned long long)(long long)p0 * T937_C0;
    lanes[1] = (unsigned long long)p1 * T937_C1;
    lanes[2] = (unsigned long long)p2 * T937_C2;
    lanes[3] = p3 * T937_C3;
    {
        union { double f; unsigned long long u; } cvt;
        cvt.f = p4;
        lanes[4] = cvt.u * T937_C4;
    }
    lanes[5] = (unsigned long long)p5 * T937_C5;
    lanes[6] = (unsigned long long)p6 * T937_C6;
    lanes[7] = (unsigned long long)p7 * T937_C7;
    {
        /* the frozen fold sequence: xor the first pair, then alternate
         * add/xor — spelled EXACTLY as the family defines it. */
        unsigned long long acc = lanes[0] ^ lanes[1];
        acc += lanes[2];
        acc ^= lanes[3];
        acc += lanes[4];
        acc ^= lanes[5];
        acc += lanes[6];
        acc ^= lanes[7];
        return (long long)acc;
    }
}

static double t937_bits_f64(unsigned long long bits)
{
    union { double f; unsigned long long u; } cvt;
    cvt.u = bits;
    return cvt.f;
}

static int t937_c_main(void)
{
    int failed = 0;

    /* 1) frozen extremes: delivered value must equal BOTH the frozen
     * hand-computed constant and this file's own fold. */
    {
        long long got = t937_mix(T937_A, T937_B, T937_C, T937_D,
                                 t937_bits_f64(T937_F_BITS), (unsigned char)T937_G,
                                 T937_H, T937_I);
        long long want = t937_caller_fold(T937_A, T937_B, T937_C, T937_D,
                                          t937_bits_f64(T937_F_BITS),
                                          (unsigned char)T937_G, T937_H, T937_I);
        if (got != T937_MIX_EXPECTED) {
            failed++;
        }
        if (got != want) {
            failed++;
        }
    }

    /* 2) -0.0 (sign bit must be observable through the fold). */
    {
        long long got = t937_mix(T937_A, T937_B, T937_C, T937_D,
                                 t937_bits_f64(0x8000000000000000ULL), 0u, 0x41u,
                                 T937_I);
        long long want = t937_caller_fold(T937_A, T937_B, T937_C, T937_D,
                                          t937_bits_f64(0x8000000000000000ULL), 0u,
                                          0x41u, T937_I);
        if (got != want) {
            failed++;
        }
        /* -0.0 and +0.0 must fold DIFFERENTLY (bit-fold, not value). */
        {
            long long pz = t937_mix(T937_A, T937_B, T937_C, T937_D,
                                    t937_bits_f64(0x0ULL), 0u, 0x41u, T937_I);
            if (pz == got) {
                failed++;
            }
        }
    }

    /* 3) NaN payload bits must be observable. */
    {
        long long p1 = t937_mix(T937_A, T937_B, T937_C, T937_D,
                                t937_bits_f64(0x7FF80000DEADBEEFULL), 1u, T937_H,
                                T937_I);
        long long p2 = t937_mix(T937_A, T937_B, T937_C, T937_D,
                                t937_bits_f64(0x7FF80000DEADBEEFULL), 1u, T937_H,
                                T937_I);
        if (p1 != p2) {
            failed++; /* determinism */
        }
        {
            long long p3 = t937_mix(T937_A, T937_B, T937_C, T937_D,
                                    t937_bits_f64(0x7FF80000DEADBEF0ULL), 1u, T937_H,
                                    T937_I);
            if (p1 == p3) {
                failed++; /* payload bit flip must change the fold */
            }
        }
    }

    /* 4) fallible div: success, wrapping edge, controlled failure with
     * out UNTOUCHED (sentinel). */
    {
        long long out = 0x5EC7000000000001LL;
        int st = t937_div(0, 20, 2, &out);
        if (st != T937_DIV_OK || out != 10) {
            failed++;
        }
        out = 0x5EC7000000000001LL;
        st = t937_div(0, (-9223372036854775807LL - 1), -1, &out);
        if (st != T937_DIV_OK || out != (-9223372036854775807LL - 1)) {
            failed++;
        }
        out = 0x5EC7000000000001LL;
        st = t937_div(0, 42, 0, &out);
        if (st != T937_DIV_ZERO || out != 0x5EC7000000000001LL) {
            failed++;
        }
    }

    return failed;
}

void t937_c_entry(void)
{
    ExitProcess((unsigned int)t937_c_main());
}
