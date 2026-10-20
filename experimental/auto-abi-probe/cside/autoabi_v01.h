/*
 * autoabi_v01.h — PLAN-797 T-01 C-side contract surface.
 *
 * This header is the INDEPENDENTLY AUDITED C view of the first probe
 * function. It is hand-maintained C, deliberately NOT generated from the
 * ABI descriptor: the descriptor-vs-header consistency check (T-02) must
 * have two independent sources to compare. Do not regenerate or sync it
 * mechanically from the Rust side.
 *
 * Platform: x86_64-pc-windows-msvc, Windows x64 calling convention.
 * Freestanding notes: objects compiled with -GS- -Zl (no CRT init, no
 * stack cookies); FP-using translation units define _fltused themselves.
 */

#ifndef AUTOABI_V01_H
#define AUTOABI_V01_H

/* Frozen T-01 probe inputs (hand-picked; asymmetric weights below make any
 * argument permutation observable in the result). */
#define P797_A 7
#define P797_B (-3)
#define P797_C 100
#define P797_D 5
#define P797_E 1000000007LL
#define P797_F 1.5
#define P797_G 11
#define P797_H (-2.25)

/* Frozen hand-computed oracle for the inputs above:
 *   e*2 + a*1 + b*3 + c*7 + d*11 + g*13 + (i64)f*1000 + (i64)h*17
 *   = 2000000014 + 7 - 9 + 700 + 55 + 143 + 1000 - 34 = 2000001876
 * (f64 -> i64 is truncation toward zero on this profile: 1.5 -> 1,
 * -2.25 -> -2; values are frozen in-range so no C UB and no Cranelift
 * fp->int trap can fire.) */
#define P797_MIX_EXPECTED 2000001876LL

/* Exit codes shared by the probe PEs: 0 = all in-PE checks passed;
 * 1 = value mismatch (ABI leg delivered a wrong result); 2 = reserved. */
#define P797_EXIT_PASS 0
#define P797_EXIT_VALUE_MISMATCH 1

/* The T-01 probe function: heterogeneous params (4 i32 in the integer
 * register banks, an i64 + i32 spilling to the stack, two f64 in XMM
 * banks), i64 return. Implemented once by the Cranelift emitter (AutoABI
 * callee) and once by callee_mix.c (C callee) — same frozen semantics,
 * independently written bodies. */
long long p797_mix(int a, int b, int c, int d, long long e, double f,
                   int g, double h);

#endif /* AUTOABI_V01_H */
