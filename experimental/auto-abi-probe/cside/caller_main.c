/*
 * caller_main.c — PLAN-797 T-01 leg C: independent C caller -> AutoABI
 * callee (Cranelift-emitted object). The oracle below is computed with
 * this file's own arithmetic; it is not derived from the AutoABI module
 * or from any shared generator. Exit 0 = delivered value matched the
 * C-side oracle; exit 1 = mismatch.
 *
 * Freestanding: no CRT. _fltused is defined here because this translation
 * unit holds FP literals (each link carries at most one C object, so the
 * definition cannot collide).
 */

#include "autoabi_v01.h"

int _fltused = 0;

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

static int p797_c_main(void)
{
    long long got = p797_mix(P797_A, P797_B, P797_C, P797_D, P797_E,
                             P797_F, P797_G, P797_H);
    long long want = P797_E * 2LL
                   + (long long)P797_A * 1LL
                   + (long long)P797_B * 3LL
                   + (long long)P797_C * 7LL
                   + (long long)P797_D * 11LL
                   + (long long)P797_G * 13LL
                   + (long long)P797_F * 1000LL
                   + (long long)P797_H * 17LL;

    if (got != want) {
        return P797_EXIT_VALUE_MISMATCH;
    }
    return P797_EXIT_PASS;
}

void p797_c_entry(void)
{
    ExitProcess(p797_c_main());
}
