/*
 * interop_resource_caller.c — PLAN-797 T-04 leg-C-style consumer of the
 * resource provider, written in C with its OWN oracle checks: lifecycle,
 * generation/double-destroy rejection, ledger facts, buffer rules,
 * rollback under injected failure, and synchronous callbacks defined in
 * C. Exit 0 = every check passed on the C side.
 */

#include "interop_resource.h"

int _fltused = 0;

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

static int flags = 0;
#define CHECK(cond) do { if (!(cond)) { flags++; } } while (0)

static long long cb_a(long long ctx, long long item)
{
    return ctx * 1000LL + item;
}

static long long cb_b(long long ctx, long long item)
{
    return ctx * 2000LL + item * 3LL;
}

static int res_main(void)
{
    long long h1, out, live, allocs, frees;
    long long items[3];
    long long got;
    unsigned char dst[8];
    unsigned char src8[8];
    int i;

    /* lifecycle: create/set/get/destroy with ledger agreement */
    h1 = p797_h_create(64);
    CHECK(h1 != 0);
    CHECK(p797_h_set(h1, 0, 4242) == R797_OK);
    CHECK(p797_h_get(h1, 0, &out) == R797_OK);
    CHECK(out == 4242);
    CHECK(p797_h_destroy(h1) == R797_OK);
    CHECK(p797_ledger(&live, &allocs, &frees) == R797_OK);
    CHECK(live == 0 && allocs == 1 && frees == 1);

    /* double destroy / forged handle rejected */
    CHECK(p797_h_destroy(h1) == R797_ERR_HANDLE);
    CHECK(p797_h_destroy(0x1234ABCD00000099LL) == R797_ERR_HANDLE);

    /* generation: a recycled slot's stale handle is rejected */
    h1 = p797_h_create(64);
    CHECK(h1 != 0);
    CHECK(p797_h_destroy(h1) == R797_OK);
    {
        long long h2 = p797_h_create(64);
        CHECK(h2 != 0);
        CHECK(h2 != h1);            /* generation bumped */
        CHECK(p797_h_destroy(h1) == R797_ERR_HANDLE); /* stale */
        CHECK(p797_h_destroy(h2) == R797_OK);
    }

    /* buffers: empty/null rules and capacity rejection */
    CHECK(p797_buf_sum(0, 0, &out) == R797_OK && out == 0);
    CHECK(p797_buf_sum(0, 8, &out) == R797_ERR_BUFNULL);
    src8[0] = 1; src8[1] = 2; src8[2] = 3; src8[3] = 4;
    src8[4] = 5; src8[5] = 6; src8[6] = 7; src8[7] = 8;
    CHECK(p797_buf_sum(src8, 8, &out) == R797_OK && out == 36);
    for (i = 0; i < 8; i++) { dst[i] = 0xAA; }
    CHECK(p797_buf_copy(dst, 4, src8, 8) == R797_ERR_CAP);
    for (i = 0; i < 8; i++) { CHECK(dst[i] == 0xAA); }
    CHECK(p797_buf_copy(dst, 8, src8, 8) == R797_OK);
    for (i = 0; i < 8; i++) { CHECK(dst[i] == (unsigned char)(i + 1)); }

    /* rollback: injected Nth failure leaves no live slot and the caller
     * out slot untouched */
    out = 555666;
    p797_fail_nth_alloc(2); /* the SECOND alloc inside two_step fails */
    CHECK(p797_two_step(9, &out) == R797_ERR_HANDLE);
    CHECK(out == 555666);
    CHECK(p797_ledger(&live, &allocs, &frees) == R797_OK);
    CHECK(live == 0);

    /* success composite: transient A released, B flows out and is
     * destroyable by the ORIGINAL provider entry */
    CHECK(p797_two_step(9, &out) == R797_OK);
    CHECK(out != 0);
    CHECK(p797_h_destroy(out) == R797_OK);
    CHECK(p797_ledger(&live, &allocs, &frees) == R797_OK);
    CHECK(live == 0);

    /* callbacks: two DIFFERENT C callbacks with explicit ctx, in
     * sequence — the second result proves the first was not stored */
    items[0] = 3; items[1] = -1; items[2] = 10;
    got = p797_fold(cb_a, 7, items, 3);
    CHECK(got == (7LL * 1000LL + 3) + (7LL * 1000LL - 1) + (7LL * 1000LL + 10));
    got = p797_fold(cb_b, 7, items, 3);
    CHECK(got == (7LL * 2000LL + 3 * 3) + (7LL * 2000LL - 3) + (7LL * 2000LL + 30));
    CHECK(p797_stored_cb() == 0);

    return flags;
}

void p797_res_entry(void)
{
    ExitProcess(res_main());
}
