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

#define R797_OK            0
#define R797_ERR_HANDLE  (-1)
#define R797_ERR_CAP     (-2)
#define R797_ERR_BUFNULL (-3)
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

#endif /* AUTOABI_INTEROP_RESOURCE_H */
