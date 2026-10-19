/*
 * interop_resource.h — PLAN-797 T-04 resource/ownership/callback
 * provider contract (hand-audited C, NOT generated).
 *
 * The provider owns every allocation: a private arena + a REAL ledger
 * (per-slot live/generation/size records — not just counters), handles
 * carry an index AND a generation so stale/double-destroy/forged
 * handles are rejected, allocation can be made to fail at a chosen Nth
 * slot to prove rollback, and callbacks are strictly synchronous
 * per-call (never stored — p797_stored_cb() is observably 0).
 *
 * Frozen status codes:
 *   0 OK / -1 ERR_HANDLE / -2 ERR_CAP / -3 ERR_BUF_NULL / -4 ERR_BOUNDS
 */

#ifndef AUTOABI_INTEROP_RESOURCE_H
#define AUTOABI_INTEROP_RESOURCE_H

#include <stdint.h>

#include "autoabi_layout_v01.h" /* RecMix wire type for record views */

#define R797_OK            0
#define R797_ERR_HANDLE  (-1)
#define R797_ERR_CAP     (-2)
#define R797_ERR_BUFNULL (-3)
#define R797_ERR_UTF8    (-7)   /* F-797-R4-02: invalid UTF-8 in a text view */
#define R797_ERR_INVALID (-8)   /* F-797-R4-02: illegal bool/char scalar value */
#define R797_ERR_BOUNDS  (-4)

/* owned-handle lifecycle */
long long p797_h_create(int size);          /* 0 = failure (injected/full) */
int p797_h_set(long long h, int idx, long long v);
int p797_h_get(long long h, int idx, long long *out);
int p797_h_destroy(long long h);            /* double/stale/forged -> -1 */

/* REAL ledger facts (derived from the slot table, not shadow counters) */
int p797_ledger(long long *live, long long *allocs, long long *frees);

/* borrowed-buffer views */
int p797_buf_sum(const unsigned char *p, uint64_t len, long long *out);
int p797_buf_copy(unsigned char *dst, uint64_t cap,
                  const unsigned char *src, uint64_t len);

/* failure injection: the Nth next allocation fails (1-based; <1 = off) */
void p797_fail_nth_alloc(int n);

/* composite op proving rollback: alloc transient A + result B; on any
 * failure A is rolled back, *out untouched, status -1; on success A is
 * released (transient), *out = handle B, status 0. */
int p797_two_step(int x, long long *out);

/* synchronous per-call callback with explicit ctx; never stored */
long long p797_fold(long long (*cb)(long long ctx, long long item),
                    long long ctx, const long long *items, uint64_t n);
long long p797_stored_cb(void); /* always 0 — persistence is observable */

/* F-797-R4-02 promised-surface entries (each has positive AND negative
 * native controls): borrowed UTF-8 view validation + record-view value
 * validation. Both are provider-owned: rejection is a typed status, the
 * out slot stays untouched on failure. */
int p797_cbuf_sum_utf8(const unsigned char *p, uint64_t len, long long *chars);
int p797_crec_valid(const RecMix *r, long long *sum);

/* F-797-R4-02 (R8): the record MUT-VIEW and CALLER-OWNED OUT-RECORD
 * lanes the profile promises — native commit/rollback controls, not
 * just spellings.
 *  - crec_mutate: MUT view. On success: count += 1, big ^= seed (flag
 *    and tag preserved) and *sum = the new weighted sum. A value
 *    failure (illegal bool/char, null view) returns a typed status and
 *    leaves EVERY caller byte — record and out slot — untouched.
 *  - crec_make: caller-owned OUT record. seed in [0,1000] commits
 *    {flag=1, count=seed, tag=0x41, big=seed*7}; any other seed is a
 *    typed failure with the caller slot untouched. */
int p797_crec_mutate(RecMix *r, long long seed, long long *sum);
int p797_crec_make(int seed, RecMix *out);

#endif /* AUTOABI_INTEROP_RESOURCE_H */
