/*
 * ce_record_caller.c — PLAN-939 T-11 (D-939-CE-1 bridge arm, S02 CE):
 * the INDEPENDENT C caller for the record source export. Its
 * expectations are HAND-COMPUTED here; nothing is derived from the
 * generator.
 *
 * Static layout checks (compile-time, C11): the public record closure
 * is one naturally aligned int32 field.
 *   sizeof(Counter) == 4, alignof(Counter) == 4,
 *   offsetof(Counter, value) == 0.
 *
 * Frozen oracle values (hand-computed):
 *   t939ce_bump({20}, 1)  -> status 0, out 21, record {21} (writeback)
 *   t939ce_bump({21}, 5)  -> status 0, out 26, record {26}
 *   t939ce_bump({INT32_MAX}, 1) -> overflow trap: negative status,
 *       out SENTINEL untouched, record byte-identical {INT32_MAX}
 *       (§5.3 failure atomicity — failure never writes caller bytes).
 *
 * Exit code = number of FAILED checks (0 = pass).
 */

#include "abi_export.h"

#include <stdint.h>
#include <stddef.h>
#include <limits.h>

/* The closed public layout is checked HERE, against the caller's own
 * understanding — a layout drift fails the compile, never a wrong run. */
_Static_assert(sizeof(Counter) == 4, "Counter is one int32 field");
_Static_assert(_Alignof(Counter) == 4, "Counter aligns naturally");
_Static_assert(offsetof(Counter, value) == 0, "value is the first field");

#define T939CE_SENTINEL 0x5EC7C0DE

int main(void)
{
    int failed = 0;
    int32_t out = 0;
    int32_t reason = 99;
    int32_t st;

    /* 1) mut writeback: 20 + 1 == 21, the caller's record follows. */
    Counter c;
    c.value = 20;
    out = T939CE_SENTINEL;
    reason = 99;
    st = t939ce_bump(0, &c, 1, &out, &reason);
    if (st != 0 || out != 21 || reason != 0 || c.value != 21) {
        failed++;
    }

    /* 2) a second bump through the SAME record: 21 + 5 == 26. */
    out = T939CE_SENTINEL;
    reason = 99;
    st = t939ce_bump(0, &c, 5, &out, &reason);
    if (st != 0 || out != 26 || reason != 0 || c.value != 26) {
        failed++;
    }

    /* 3) failure atomicity: INT32_MAX + 1 traps; the caller's record
     * keeps EVERY byte and the out sentinel is untouched. */
    c.value = INT32_MAX;
    out = T939CE_SENTINEL;
    reason = 0;
    st = t939ce_bump(0, &c, 1, &out, &reason);
    if (st == 0 || out != T939CE_SENTINEL || c.value != INT32_MAX || reason == 0) {
        failed++;
    }

    return failed;
}
