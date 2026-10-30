/*
 * interop_record_mutout.c — F-797-R4-02 (R8) record mut-view /
 * caller-owned out-record consumer, hand-written C with its OWN
 * byte-level oracle: success commits are observed by value AND the
 * failure paths are proven untouched by comparing a full saved copy of
 * the caller memory (padding included). Exit 0 = every check passed.
 */

#include "interop_resource.h"

int _fltused = 0;

__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);

static int flags = 0;
#define CHECK(cond) do { if (!(cond)) { flags++; } } while (0)

static int rec_eq(const RecMix *a, const RecMix *b)
{
    const unsigned char *pa = (const unsigned char *)a;
    const unsigned char *pb = (const unsigned char *)b;
    size_t i;
    for (i = 0; i < sizeof(RecMix); i++) {
        if (pa[i] != pb[i]) {
            return 0;
        }
    }
    return 1;
}

static int mo_main(void)
{
    RecMix r, keep, made;
    long long sum, live, allocs, frees;

    /* --- mut view: success commits the declared mutation ----------- */
    r.flag = 1u; r.count = 10; r.tag = 0x61u; r.big = 100LL;
    keep = r;
    sum = -1;
    CHECK(p797_crec_mutate(&r, 0x5A5A5A5ALL, &sum) == R797_OK);
    CHECK(r.count == 11);                      /* count += 1 */
    CHECK(r.big == (100LL ^ 0x5A5A5A5ALL));    /* big ^= seed */
    CHECK(r.flag == keep.flag && r.tag == keep.tag); /* preserved */
    CHECK(sum == 1LL * 1000LL + 11LL * 3LL + 0x61LL * 5LL
              + (100LL ^ 0x5A5A5A5ALL));       /* weighted, mutated */
    CHECK(!rec_eq(&r, &keep));                 /* really mutated */

    /* --- mut view: value failure leaves EVERY byte untouched -------- */
    r.flag = 2u; /* illegal bool */
    keep = r;
    sum = -1;
    CHECK(p797_crec_mutate(&r, 7LL, &sum) == R797_ERR_INVALID);
    CHECK(rec_eq(&r, &keep));                  /* record untouched */
    CHECK(sum == -1);                          /* out slot untouched */
    r.tag = 0xD800u; /* surrogate char */
    keep = r;
    CHECK(p797_crec_mutate(&r, 7LL, &sum) == R797_ERR_INVALID);
    CHECK(rec_eq(&r, &keep));
    CHECK(sum == -1);
    CHECK(p797_crec_mutate(0, 7LL, &sum) == R797_ERR_BUFNULL);

    /* --- out record: success commits the derived record ------------- */
    made.flag = 9u; made.count = -9; made.tag = 9u; made.big = -9LL;
    CHECK(p797_crec_make(500, &made) == R797_OK);
    CHECK(made.flag == 1u && made.count == 500 && made.tag == 0x41u
          && made.big == 3500LL);

    /* --- out record: rejected seed leaves the slot untouched -------- */
    made.flag = 9u; made.count = -9; made.tag = 9u; made.big = -9LL;
    keep = made;
    CHECK(p797_crec_make(-1, &made) == R797_ERR_INVALID);
    CHECK(rec_eq(&made, &keep));
    CHECK(p797_crec_make(1001, &made) == R797_ERR_INVALID);
    CHECK(rec_eq(&made, &keep));
    CHECK(p797_crec_make(0, 0) == R797_ERR_BUFNULL);

    /* --- the ledger is untouched by record-lane failures ------------ */
    CHECK(p797_ledger(&live, &allocs, &frees) == R797_OK);
    CHECK(live == 0 && allocs == 0 && frees == 0);

    return flags;
}

void p797_rec_mo_entry(void)
{
    ExitProcess(mo_main());
}
