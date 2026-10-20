/*
 * interop_record_mutout_negctl.c — F-797-R4-02 (R8) negative control:
 * the SAME mut-view entry with a FLIPPED expectation (asserting the
 * typed-failure contract against a VALID record) must exit NONZERO —
 * proving the mut/out checks are wired to the real commit/rollback
 * behavior, not constants.
 */

#include "interop_resource.h"

int _fltused = 0;

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

static int bad = 0;

static int neg_main(void)
{
    RecMix r;
    long long sum;

    r.flag = 1u; r.count = 10; r.tag = 0x61u; r.big = 100LL;
    sum = -1;
    /* VALID record: the provider must SUCCEED — expecting the failure
     * contract here makes this consumer fail (exit != 0). */
    if (p797_crec_mutate(&r, 7LL, &sum) != R797_ERR_INVALID) {
        bad = 1;
    }
    if (p797_crec_make(500, &r) != R797_ERR_INVALID) {
        bad = 1;
    }
    return bad;
}

void p797_rec_mo_neg_entry(void)
{
    ExitProcess(neg_main());
}
