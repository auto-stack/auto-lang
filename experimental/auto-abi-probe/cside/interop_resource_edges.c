/*
 * interop_resource_edges.c — PLAN-797 R4 permanent counterexample
 * consumer (F-797-R4-04/05/06): independent C oracle driving BOTH
 * providers (A = interop_resource.c exports; B = the renamed-shim
 * compilation). Exit 0 = every edge rule holds.
 *
 *  1. negative index on h_set/h_get is BOUNDS-rejected and cannot
 *     touch the neighbouring allocation (F04);
 *  2. a handle minted by provider A is HANDLE-rejected by provider B
 *     (and vice versa) with B's payload unchanged (F05);
 *  3. create/destroy cycles RECLAIM capacity: 80 rounds of a 16KiB
 *     object with live==0 between rounds never exhaust the arena (F06);
 *  4. valid first/last slot elements still round-trip (boundary
 *     control for the F04 fix).
 */

#include "interop_resource.h"

/* Provider B surface (renamed exports of the independent shim TU). */
long long b_h_create(int size);
int b_h_set(long long h, int idx, long long v);
int b_h_get(long long h, int idx, long long *out);
int b_h_destroy(long long h);
int b_ledger(long long *live, long long *allocs, long long *frees);

int _fltused = 0; /* this TU compares doubles nowhere but keeps the
                    freestanding set uniform for the link */

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

static int flags = 0;
#define CHECK(cond) do { if (!(cond)) { flags++; } } while (0)

static int edges_main(void)
{
    long long a, b, out, live, allocs, frees;
    int i;

    /* --- F04: negative index cannot cross into a neighbour --------- */
    a = p797_h_create(16);           /* slot-local 2-element object */
    b = p797_h_create(16);
    CHECK(a != 0 && b != 0);
    CHECK(p797_h_set(a, 1, 222) == R797_OK);
    CHECK(p797_h_set(b, -1, 999) == R797_ERR_BOUNDS);  /* the wrap case */
    CHECK(p797_h_get(a, 1, &out) == R797_OK && out == 222); /* intact */
    out = 555;
    CHECK(p797_h_get(b, -1, &out) == R797_ERR_BOUNDS);
    CHECK(out == 555);               /* old out untouched on rejection */
    /* far-negative and idx-past-end also rejected */
    CHECK(p797_h_set(b, -2000000000, 1) == R797_ERR_BOUNDS);
    CHECK(p797_h_set(b, 2, 1) == R797_ERR_BOUNDS);
    /* valid first/last elements still round-trip */
    CHECK(p797_h_set(b, 0, 11) == R797_OK && p797_h_set(b, 1, 22) == R797_OK);
    CHECK(p797_h_get(b, 0, &out) == R797_OK && out == 11);
    CHECK(p797_h_get(b, 1, &out) == R797_OK && out == 22);

    /* --- F05: cross-provider handles are rejected ------------------- */
    {
        long long hb = b_h_create(16);
        CHECK(hb != 0);
        CHECK(b_h_set(hb, 0, 777) == 0); /* B self-access control */
        CHECK(p797_h_set(hb, 0, 999) == R797_ERR_HANDLE); /* A rejects B */
        CHECK(b_h_set(a, 0, 999) == R797_ERR_HANDLE);     /* B rejects A */
        /* both payloads unchanged after the rejected foreign writes */
        CHECK(p797_h_get(a, 0, &out) == R797_OK && out == 0);
        CHECK(b_h_get(hb, 0, &out) == 0 && out == 777);
        CHECK(b_h_destroy(hb) == 0);
    }

    CHECK(p797_h_destroy(a) == R797_OK);
    CHECK(p797_h_destroy(b) == R797_OK);

    /* --- F06: capacity is really reclaimed -------------------------- */
    for (i = 0; i < 80; i++) {
        long long h = p797_h_create(16384);
        CHECK(h != 0); /* without reclamation the arena exhausts ~64 */
        CHECK(p797_h_set(h, 0, (long long)i) == R797_OK);
        CHECK(p797_h_get(h, 0, &out) == R797_OK && out == (long long)i);
        CHECK(p797_h_destroy(h) == R797_OK);
        CHECK(p797_ledger(&live, &allocs, &frees) == R797_OK);
        CHECK(live == 0);
    }
    CHECK(allocs == 82 && frees == 82); /* 2 earlier + 80 cycles */

    return flags;
}

void p797_res_edges_entry(void)
{
    ExitProcess(edges_main());
}
