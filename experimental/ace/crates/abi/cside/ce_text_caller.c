/*
 * ce_text_caller.c - PLAN-939 T-11 (D-939-CE-1 bridge arm, S01 CE):
 * the INDEPENDENT C caller for the text-view source export. Its
 * expectations are HAND-COMPUTED here; nothing is derived from the
 * generator.
 *
 * Frozen oracle values (hand-computed; `.len()` on the source str view
 * counts UNICODE SCALARS - the frozen 925 semantics - while the caller
 * passes the exact BYTE span; NUL is DATA, never a terminator):
 *   t939ce_bytes("A")        -> status 0, out 1  (1 byte, 1 scalar)
 *   t939ce_bytes("")         -> status 0, out 0   (len0 is legal)
 *   t939ce_bytes("A\0B")     -> status 0, out 3   (NUL is a counted scalar)
 *   t939ce_bytes("A\xe4\xb8\xad\xf0\x9f\x98\x80") -> status 0, out 3
 *       (8 bytes / 3 scalars - scalar-counting over the full multibyte
 *        span proves the whole BYTE span crossed, un-truncated)
 *   t939ce_bytes("e\xcc\x81") -> status 0, out 2  (3 bytes / 2 scalars)
 *
 * Exit code = number of FAILED checks (0 = pass).
 */

#include "abi_export.h"

#include <stdint.h>
#include <stddef.h>
#include <stdio.h>

int main(void)
{
    int failed = 0;
    int32_t out = 0;
    int32_t reason = 77;
    int32_t st;

    /* 1) one ASCII byte. */
    out = -1;
    reason = 77;
    st = t939ce_bytes(0, (const uint8_t*)"A", 1, &out, &reason);
    if (st != 0 || out != 1 || reason != 0) {
        printf("chk1 st=%d out=%d reason=%d\n", (int)st, (int)out, (int)reason);
        failed++;
    }

    /* 2) the legal empty view (len 0). */
    out = -1;
    reason = 77;
    st = t939ce_bytes(0, (const uint8_t*)"", 0, &out, &reason);
    if (st != 0 || out != 0 || reason != 0) {
        printf("chk2 st=%d out=%d reason=%d\n", (int)st, (int)out, (int)reason);
        failed++;
    }

    /* 3) an EMBEDDED NUL: three data bytes, none a terminator. */
    out = -1;
    reason = 77;
    st = t939ce_bytes(0, (const uint8_t*)"A\0B", 3, &out, &reason);
    if (st != 0 || out != 3 || reason != 0) {
        printf("chk3 st=%d out=%d reason=%d\n", (int)st, (int)out, (int)reason);
        failed++;
    }

    /* 4) multibyte: 'A' + U+4E2D + U+1F600 = 8 bytes / 3 scalars. */
    out = -1;
    reason = 77;
    st = t939ce_bytes(
        0,
        (const uint8_t*)"A\xe4\xb8\xad\xf0\x9f\x98\x80",
        8,
        &out,
        &reason);
    if (st != 0 || out != 3 || reason != 0) {
        printf("chk4 st=%d out=%d reason=%d\n", (int)st, (int)out, (int)reason);
        failed++;
    }

    /* 5) decomposed 'e' + U+0301 = 3 bytes / 2 scalars. */
    out = -1;
    reason = 77;
    st = t939ce_bytes(0, (const uint8_t*)"e\xcc\x81", 3, &out, &reason);
    if (st != 0 || out != 2 || reason != 0) {
        printf("chk5 st=%d out=%d reason=%d\n", (int)st, (int)out, (int)reason);
        failed++;
    }

    return failed;
}
