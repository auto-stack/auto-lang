/*
 * bridge_source_caller.c — PLAN-937 CA-D1 source-export leg: the
 * INDEPENDENT C caller that EXECUTES the exported public surface of the
 * generated library (library.c + generated.c). Its expectations are
 * HAND-COMPUTED here; nothing is derived from the generator.
 *
 * Frozen oracle values (hand-computed):
 *   t937_sum(20, 22)            -> status 0,  out 42,  reason 0
 *   t937_sum body+1 edit        -> status 0,  out 43 (the edit MUST be
 *                                  observable through the SAME chain)
 *   t937_div(20, 2)             -> status 0,  out 10,  reason 0
 *   t937_div(INT32_MIN, -1)     -> status -103 (div-overflow), reason 3,
 *                                  out SENTINEL (untouched on failure)
 *   t937_div(42, 0)             -> status -102 (div-by-zero), reason 2,
 *                                  out SENTINEL
 *
 * Exit code = number of FAILED checks (0 = pass). Compiled with the
 * full CRT closure (/MT) — the a2x_c runtime header includes stdio.
 */

#include "abi_export.h"

#include <stdio.h>

#define T937_SRC_EXIT_OK 0
#define T937_SRC_SENTINEL 0x5EC7C0DE

int main(void)
{
    int failed = 0;
    int32_t out = 0;
    int32_t reason = 99;
    int32_t st;

    /* 1) sum: 20 + 22 == 42, success commits out+reason. */
    out = T937_SRC_SENTINEL;
    reason = 99;
    st = t937_sum(0, 20, 22, &out, &reason);
    if (st != 0 || out != 42 || reason != 0) {
        failed++;
    }

    /* 2) div success: 20 / 2 == 10. */
    out = T937_SRC_SENTINEL;
    reason = 99;
    st = t937_div(0, 20, 2, &out, &reason);
    if (st != 0 || out != 10 || reason != 0) {
        failed++;
    }

    /* 3) div by zero: status -102, reason carries the RAW trap code 2,
     * out UNTOUCHED (sentinel). */
    out = T937_SRC_SENTINEL;
    reason = 99;
    st = t937_div(0, 42, 0, &out, &reason);
    if (st != -102 || reason != 2 || out != T937_SRC_SENTINEL) {
        failed++;
    }

    /* 4) div overflow (INT32_MIN / -1): status -103, reason 3, out
     * untouched. */
    out = T937_SRC_SENTINEL;
    reason = 99;
    st = t937_div(0, (-2147483647 - 1), -1, &out, &reason);
    if (st != -103 || reason != 3 || out != T937_SRC_SENTINEL) {
        failed++;
    }

    /* 5) the ctx lane is nullable and never dereferenced: a non-null
     * dummy ctx must behave identically. */
    {
        int dummy_ctx = 1;
        out = T937_SRC_SENTINEL;
        reason = 99;
        st = t937_sum(&dummy_ctx, 2, 40, &out, &reason);
        if (st != 0 || out != 42 || reason != 0) {
            failed++;
        }
    }

    if (failed == 0) {
        printf("source-export: ok\n");
    } else {
        printf("source-export: %d checks FAILED\n", failed);
    }
    return failed;
}
