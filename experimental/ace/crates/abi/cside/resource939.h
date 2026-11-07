/*
 * resource939.h — PLAN-939 CA-D3 resource/callback provider contract
 * (hand-audited C, NOT generated).
 *
 * The provider owns every allocation: a REAL per-slot ledger (live /
 * generation / size), handles carry index AND generation so stale,
 * double-destroy and forged handles are rejected, allocation can be made
 * to fail at a chosen Nth slot to prove failure atomicity, and mut/out
 * record views are committed ONLY on success (a failure leaves every
 * caller byte untouched).
 *
 * Handle encoding: gen * 1024 + idx (gen in [1,255], idx in [0,511]) —
 * every live handle is a positive value inside the i32 range, so the
 * checked i64<->i32 bridge conversion is exact and reversible.
 *
 * Frozen status codes:
 *   0 OK / -1 ERR_HANDLE / -2 ERR_SEED / -3 ERR_ALLOC
 */
#ifndef PROVIDER_939_H
#define PROVIDER_939_H
#include <stdint.h>

#define R939_OK          0
#define R939_ERR_HANDLE (-1)
#define R939_ERR_SEED   (-2)
#define R939_ERR_ALLOC  (-3)

/* public record closures (natural align, i32 fields only) */
typedef struct { int32_t value; } Counter;
typedef struct { int32_t value; int32_t handle; } TxnResult;

/* S01 — text views (len is a BYTE count; encoding pre-validated by the
 * bridge; NUL is data). Both totals are pure. */
uint64_t t939_byte_length(const uint8_t *p, uint64_t len);
uint64_t t939_scalar_count(const uint8_t *p, uint64_t len);

/* S02 — atomic mut record view: success commits r->value += delta and
 * *out = the new value; a negative seed is a typed failure with EVERY
 * caller byte (record AND out slot) untouched. */
int32_t t939_bump(Counter *r, int32_t delta, int32_t *out);

/* S04 — owned opaque handles with a real ledger */
int64_t  t939_create(int32_t size);                /* 0 = injected failure */
int32_t  t939_get(int64_t h, int32_t *out);
int32_t  t939_destroy(int64_t h, int32_t *out);    /* *out = destroyed gen; stale/double -> -1 with *out untouched */
int32_t  t939_live(void);                          /* probe: current live-handle count (slot-table derived) */
int32_t  t939_ledger(int64_t *live, int64_t *allocs, int64_t *frees); /* reference leg only */
void     t939_fail_nth_alloc(int32_t n);           /* 1-based; <1 = off */

/* S05 — atomic multi-out transaction: prepares BOTH outputs (scalar out
 * AND record out), then commits once. seed in [0,1000] succeeds with
 * out2->value = seed*2, out2->handle = a fresh handle, *out = seed.
 * seed in [1001,2000] is the LAST-STEP failure lane: the transient
 * handle IS created and then destroyed by the ORIGINAL provider before
 * the typed failure returns. Any other seed fails before allocation.
 * EVERY failure returns with BOTH caller slots byte-identical (V-04). */
int32_t t939_txn(int32_t seed, TxnResult *out2, int32_t *out);

/* S06 — synchronous per-call CALLBACK with non-zero ctx: folds
 * cb(ctx_seed, item) for item in [0, n). A negative callback result is
 * returned VERBATIM as the status (the error channel — never a success
 * sentinel); n outside [0,64] fails with -2 before any call; a callback
 * result or the running sum outside the i32 range fails with -2 with
 * *out untouched. The provider never stores cb (noescape, synchronous
 * per-call); ctx is passed through unchanged. The ctx seed is the next
 * ordinary argument AFTER the callback, physically long long. */
int32_t t939_fold(long long (*cb)(long long ctx, long long item),
                  long long ctx_seed, int32_t n, int32_t *out);

/* provider reset (test isolation only; not part of the reviewed surface) */
void t939_reset(void);

#endif /* PROVIDER_939_H */
