/*
 * resource939_reference.c — PLAN-939 independent oracle caller (the
 * REFERENCE leg). Hand-written expectations; exits non-zero on the first
 * divergence. Compiled and run separately from any generated code.
 *
 * scenario selected via -DREFERENCE_SCENARIO=<name>:
 *   s01   — text views (byte length + scalar count, NUL/multibyte)
 *   s02   — atomic mut record (success + failure lanes)
 *   s04   — handle lifecycle (create/get/destroy/ledger/live)
 *   s05   — atomic multi-out txn (success, seed failure, last-step
 *           failure with original-party reclamation)
 */
#include "resource939.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#ifndef REFERENCE_SCENARIO
#define REFERENCE_SCENARIO s01
#endif

#define CHECK(cond) do { if (!(cond)) { fprintf(stderr, "reference939 FAIL %s:%d\n", __FILE__, __LINE__); exit(70); } } while (0)

int main(void) {
#if REFERENCE_SCENARIO == s01
    {
        /* "A中😀" = 1 + 3 + 4 = 8 bytes, 3 scalars; NUL preserved as data */
        const uint8_t text[] = { 'A', 0xE4, 0xB8, 0xAD, 0xF0, 0x9F, 0x98, 0x80 };
        CHECK(t939_byte_length(text, 8) == 8);
        CHECK(t939_scalar_count(text, 8) == 3);
        const uint8_t with_nul[] = { 'a', 0x00, 'b' };
        CHECK(t939_byte_length(with_nul, 3) == 3);
        CHECK(t939_scalar_count(with_nul, 3) == 3);
        const uint8_t empty[] = { 0 };
        CHECK(t939_byte_length(empty, 0) == 0); /* legal len0 */
    }
#elif REFERENCE_SCENARIO == s02
    {
        Counter c;
        int32_t out = -1;
        c.value = 20;
        CHECK(t939_bump(&c, 1, &out) == 0);
        CHECK(c.value == 21 && out == 21);
        Counter d;
        d.value = 20;
        out = -1;
        CHECK(t939_bump(&d, -1, &out) == -2);
        CHECK(d.value == 20 && out == -1); /* every caller byte untouched */
    }
#elif REFERENCE_SCENARIO == s04
    {
        t939_reset();
        int64_t live = 0, allocs = 0, frees = 0;
        CHECK(t939_ledger(&live, &allocs, &frees) == 0 && live == 0);
        int64_t h = t939_create(4);
        CHECK(h > 0 && h <= 0x7fffffff);
        CHECK(t939_live() == 1);
        int32_t got = -1;
        CHECK(t939_get(h, &got) == 0 && got == 4);
        int32_t gen = -1;
        CHECK(t939_destroy(h, &gen) == 0 && gen >= 1);
        CHECK(t939_destroy(h, &gen) == -1); /* double destroy rejected */
        CHECK(t939_get(h, &got) == -1);     /* stale handle rejected */
        CHECK(t939_ledger(&live, &allocs, &frees) == 0);
        CHECK(live == 0 && allocs == 1 && frees == 1);
    }
#elif REFERENCE_SCENARIO == s05
    {
        t939_reset();
        TxnResult r;
        int32_t out = -1;
        r.value = 111;
        r.handle = 222;
        CHECK(t939_txn(7, &r, &out) == 0);
        CHECK(out == 7 && r.value == 14 && r.handle > 0);
        CHECK(t939_live() == 1);
        /* seed failure: BOTH slots byte-identical */
        int32_t out2 = -1;
        TxnResult keep = r;
        CHECK(t939_txn(3000, &r, &out2) == -2);
        CHECK(out2 == -1 && r.value == keep.value && r.handle == keep.handle);
        /* last-step failure: transient handle reclaimed by the provider */
        CHECK(t939_txn(1500, &r, &out2) == -3);
        CHECK(out2 == -1 && r.value == keep.value && r.handle == keep.handle);
        CHECK(t939_live() == 1); /* only the committed one survives */
        int64_t live = 0, allocs = 0, frees = 0;
        CHECK(t939_ledger(&live, &allocs, &frees) == 0);
        CHECK(allocs == 3 && frees == 2); /* original-party reclamation */
    }
#else
    fprintf(stderr, "reference939: unknown scenario\n");
    return 71;
#endif
    fprintf(stderr, "reference939: ok\n");
    return 0;
}
