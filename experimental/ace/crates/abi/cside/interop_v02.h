/*
 * interop_v02.h — PLAN-797 T-03 interop suite contract surface.
 *
 * Hand-maintained, independently audited C (NOT generated from the
 * descriptor). Two families live here:
 *  - p797_c* — the C CALLEE implementations (leg B: an AutoABI caller
 *    object links against these);
 *  - the AutoABI suite prototypes used by the C CALLER (leg C:
 *    interop_caller.c), which also DEFINES p797_inner so the AutoABI
 *    callee's nested call (C -> AutoABI -> C) lands back in C.
 *
 * Frozen semantics (mirrors of decl.rs oracles / emit.rs bodies):
 *   mix    e*2 + a*1 + b*3 + c*7 + d*11 + g*13 + (i64)f*1000 + (i64)h*17
 *   inner  x*3 + 11
 *   facc   a*2.5 + b*1.25            (binary-exact; == is the oracle)
 *   outer  inner((i64)a*7 + (i64)b*100) + a
 *   div    d==0 -> status -1, out untouched; else *q = n/d (trunc), 0
 *   rec    flag*1000 + count*3 + tag*5 + big  (offsets 0/4/8/16)
 *   bump   module-global += x, returns new total
 */

#ifndef AUTOABI_INTEROP_V02_H
#define AUTOABI_INTEROP_V02_H

#include "autoabi_v01.h"      /* p797_mix + frozen mix inputs */
#include "autoabi_layout_v01.h" /* RecMix wire type */

#define P797_DIV_ZERO (-1)

/* ---- C callee implementations (leg B) ---- */
long long p797_cmix(int a, int b, int c, int d, long long e, double f,
                    int g, double h);
double p797_cfacc(double a, double b);
int p797_cdiv(void *ctx, long long n, long long d, long long *q);
long long p797_crec_sum(const RecMix *r);
long long p797_cbump(int x);

/* ---- AutoABI suite surface (called by the leg C C caller) ---- */
double p797_facc(double a, double b);
long long p797_outer(int a, double b); /* imports p797_inner (C side) */
int p797_div(void *ctx, long long n, long long d, long long *q);
long long p797_rec_sum(const RecMix *r);
long long p797_bump(int x);
long long p797_inner(long long x); /* defined by the C caller object */

#endif /* AUTOABI_INTEROP_V02_H */
