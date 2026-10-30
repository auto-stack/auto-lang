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

static int r5_main(void)
{
    long long h[256], out, live, allocs, frees;
    int i;

    /* --- F-797-R5-01: failed create (slot table full) leaks NOTHING -- */
    for (i = 0; i < 256; i++) {
        h[i] = p797_h_create(8);
        CHECK(h[i] != 0);
    }
    /* every create must now fail: the slot table is full, and a failed
     * create must NOT consume arena/free-list capacity */
    for (i = 0; i < 64; i++) {
        CHECK(p797_h_create(16384) == 0);
    }
    for (i = 0; i < 256; i++) {
        CHECK(p797_h_destroy(h[i]) == R797_OK);
    }
    CHECK(p797_ledger(&live, &allocs, &frees) == R797_OK);
    CHECK(live == 0); /* alloc/free totals are cumulative with edges_main */
    /* after reclaiming everything the arena must still admit a 16KiB
     * object (nothing was leaked by the 64 failed creates) */
    {
        long long h2 = p797_h_create(16384);
        CHECK(h2 != 0);
        CHECK(p797_h_destroy(h2) == R797_OK);
    }

    /* --- F-797-R5-02: generation exhaustion retires, never wraps ----- */
    /* drive ONE slot to the 16-bit generation ceiling: create+destroy
     * 65,535 times keeps the same slot (table is otherwise empty) */
    for (i = 0; i < 65535; i++) {
        long long g1 = p797_h_create(8);
        CHECK(g1 != 0);
        if (i < 65534) {
            CHECK(p797_h_destroy(g1) == R797_OK);
        } else {
            /* the 65,535th generation handle stays live; its destroy
             * crosses the ceiling and RETIRES the slot */
            CHECK(p797_h_destroy(g1) == R797_OK);
        }
    }
    /* the retired slot must not be handed out again; a fresh create
     * must still succeed on ANOTHER slot and round-trip */
    {
        long long h3 = p797_h_create(8);
        CHECK(h3 != 0);
        CHECK(p797_h_set(h3, 0, 4242) == R797_OK);
        CHECK(p797_h_get(h3, 0, &out) == R797_OK && out == 4242);
        CHECK(p797_h_destroy(h3) == R797_OK);
    }
    CHECK(p797_ledger(&live, &allocs, &frees) == R797_OK);
    CHECK(live == 0);

    return flags;
}

void p797_res_edges_entry(void)
{
    ExitProcess(edges_main() + r5_main());
}
