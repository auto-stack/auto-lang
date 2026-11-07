/*
 * resource939_fault_sweep.c — PLAN-939 T-07 (V-05): the allocation-
 * domain fault sweep over the THREE-out transaction t939_txn3.
 *
 * Hand-audited independent C driver (compiled by the real cl, linked
 * against the provider, EXECUTED). The sweep is exhaustive over the
 * MEASURED allocation domain: every k in 1..N fails in turn, N+1 is a
 * no-op, and clearing the fault retries FRESH in the same process.
 * Checked per failure: EVERY caller byte survives untouched, the
 * partial half-tree is reclaimed by the ORIGINAL provider (ledger
 * balanced: allocs == frees, live == 0), and a stale copy of a
 * reclaimed handle rejects (never an idempotent free masking a double
 * destroy). Exits with the FAILED-check count (0 = pass).
 */

#include "resource939.h"

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

/* freestanding: failures are reported ONLY through the exit code (the
 * check count) — no CRT headers inside the pinned cside include set. */
static int g_failed = 0;

#define SW_CHECK(cond) \
    do { \
        if (!(cond)) { \
            g_failed++; \
            return g_failed; \
        } \
    } while (0)

static int sweep(void)
{
    int64_t live0 = -1, allocs0 = -1, frees0 = -1;
    int64_t live1 = -1, allocs1 = -1, frees1 = -1;
    Txn3OutA a;
    TxnResult b;
    int32_t out;

    /* 1) MEASURE the allocation domain: a clean run's allocs delta. */
    SW_CHECK(t939_ledger(&live0, &allocs0, &frees0) == 0);
    a.mark = 0; a.handle = 0; b.value = 0; b.handle = 0; out = 0;
    SW_CHECK(t939_txn3(500, &a, &b, &out) == 0);
    SW_CHECK(t939_ledger(&live1, &allocs1, &frees1) == 0);
    long long domain = allocs1 - allocs0;
    SW_CHECK(domain == 2); /* the domain is measured, never assumed */
    SW_CHECK(frees1 - frees0 == 0);
    SW_CHECK(live1 - live0 == 2); /* both committed handles stay live */
    {
        int32_t g = -1;
        SW_CHECK(t939_destroy(a.handle, &g) == 0);
        SW_CHECK(t939_destroy(b.handle, &g) == 0);
    }

    /* 2) the sweep: every k in 1..domain fails IN TURN; N+1 no-op. */
    for (int32_t k = 1; k <= (int32_t)domain + 1; ++k) {
        int64_t lb = -1, ab = -1, fb = -1;
        int64_t la = -1, aa = -1, fa = -1;
        Txn3OutA sa;
        TxnResult sb;
        int32_t sout;
        sa.mark = 0x5EC7; sa.handle = 0x5EC7;
        sb.value = 0x5EC7; sb.handle = 0x5EC7;
        sout = 0x5EC7;
        SW_CHECK(t939_ledger(&lb, &ab, &fb) == 0);
        t939_fail_nth_alloc(k);
        int32_t st = t939_txn3(500, &sa, &sb, &sout);
        SW_CHECK(t939_ledger(&la, &aa, &fa) == 0);
        if (k <= domain) {
            /* the injected failure fires INSIDE the domain: typed -3,
             * every caller byte untouched, every PARTIAL allocation
             * reclaimed by the original provider (balance), live
             * unchanged. */
            SW_CHECK(st == -3);
            SW_CHECK(sa.mark == 0x5EC7 && sa.handle == 0x5EC7);
            SW_CHECK(sb.value == 0x5EC7 && sb.handle == 0x5EC7);
            SW_CHECK(sout == 0x5EC7);
            long long grew = aa - ab; /* allocations that really happened */
            SW_CHECK(grew == (long long)(k - 1));
            SW_CHECK(fa - fb == grew); /* each one freed by the provider */
            SW_CHECK(la - lb == 0);
        } else {
            /* k = N+1: past the domain — the fault NEVER fires; the
             * transaction commits fresh. */
            SW_CHECK(st == 0);
            SW_CHECK(sa.mark == 501 && sb.value == 1000 && sout == 500);
            SW_CHECK(sa.handle > 0 && sb.handle > 0);
            SW_CHECK(aa - ab == domain);
            SW_CHECK(la - lb == domain);
            {
                int32_t g = -1;
                SW_CHECK(t939_destroy(sa.handle, &g) == 0);
                SW_CHECK(t939_destroy(sb.handle, &g) == 0);
            }
        }
    }

    /* 3) clear the fault and RETRY in the same process: fresh success. */
    {
        Txn3OutA sa;
        TxnResult sb;
        int32_t sout;
        sa.mark = 0; sa.handle = 0; sb.value = 0; sb.handle = 0; sout = 0;
        t939_fail_nth_alloc(0);
        SW_CHECK(t939_txn3(500, &sa, &sb, &sout) == 0);
        SW_CHECK(sa.mark == 501 && sb.value == 1000 && sout == 500);
        {
            int32_t g = -1;
            SW_CHECK(t939_destroy(sa.handle, &g) == 0);
            SW_CHECK(t939_destroy(sb.handle, &g) == 0);
        }
        /* retry = fresh: a SECOND clear-fault run succeeds too. */
        sa.mark = 0; sa.handle = 0; sb.value = 0; sb.handle = 0; sout = 0;
        SW_CHECK(t939_txn3(500, &sa, &sb, &sout) == 0);
        {
            int32_t g = -1;
            SW_CHECK(t939_destroy(sa.handle, &g) == 0);
            SW_CHECK(t939_destroy(sb.handle, &g) == 0);
        }
    }

    /* 4) the double-destroy guard: a STALE COPY of a handle the provider
     * already reclaimed rejects — the cleanup is exactly-once per handle,
     * never an idempotent free (a forged duplicate of a live handle's
     * ENCODING is likewise rejected after its one destroy). */
    {
        Txn3OutA sa;
        TxnResult sb;
        int32_t sout, g = -1;
        sa.mark = 0; sa.handle = 0; sb.value = 0; sb.handle = 0; sout = 0;
        SW_CHECK(t939_txn3(500, &sa, &sb, &sout) == 0);
        int32_t stale = sa.handle; /* a stale copy kept by the caller */
        SW_CHECK(t939_destroy(sa.handle, &g) == 0);
        g = -1;
        SW_CHECK(t939_destroy(stale, &g) == -1); /* the copy rejects */
        SW_CHECK(g == -1); /* untouched on the rejection */
        SW_CHECK(t939_destroy(sb.handle, &g) == 0); /* reclaim the second */
    }

    /* 5) final balance: every allocation in this process was freed. */
    {
        int64_t lf = -1, af = -1, ff = -1;
        SW_CHECK(t939_ledger(&lf, &af, &ff) == 0);
        SW_CHECK(lf == 0);
        SW_CHECK(af == ff);
    }
    return 0;
}

void r939_sweep_entry(void)
{
    int r = sweep();
    if (r == 0) {
        r = g_failed; /* a check that returned without failing still reports */
    }
    ExitProcess((unsigned int)r);
}
