/*
 * interop_resource.c — PLAN-797 T-04 provider implementation.
 *
 * Real allocation, real ledger, provider-owned destroy, generation
 * handles, failure injection with rollback, borrowed-buffer rules, and
 * strictly synchronous callbacks. The ledger is the slot table itself;
 * p797_ledger derives its facts from it, so "live==0 after rollback"
 * means the table has no live slot — not a replayed counter.
 */

#include "interop_resource.h"

/* Freestanding notes: /O2 would fold byte loops into CRT memset calls
 * (an unresolved symbol under -Zl; memset is also an MSVC intrinsic
 * that user code may not define), so scrub loops write through
 * volatile to stay byte loops. */

#define R_ARENA_BYTES  (1u << 20)
#define R_SLOT_COUNT   256
#define R_MAX_SLOT     16384

static unsigned char g_arena[R_ARENA_BYTES];
static struct {
    int live;
    uint64_t gen;   /* bumps on each reuse; 0 = never used */
    size_t size;
    size_t off;
} g_slots[R_SLOT_COUNT];
static size_t g_bump = 0;          /* arena bump pointer */
static uint64_t g_allocs = 0;      /* ledger facts (derived use only) */
static uint64_t g_frees = 0;
static int g_fail_nth = -1;        /* 1-based; -1 = off */

#define R_IDX(h)  ((uint32_t)((uint64_t)(h) & 0xFFFFFFFFu))
#define R_GEN(h)  ((uint32_t)((uint64_t)(h) >> 32))

static long long slot_handle(int i)
{
    return ((long long)(uint32_t)g_slots[i].gen << 32) | (uint32_t)i;
}

long long p797_h_create(int size)
{
    int i;
    size_t need;

    if (g_fail_nth >= 0) {
        if (g_fail_nth == 1) {
            g_fail_nth = -1;
            return 0;
        }
        g_fail_nth--;
    }
    if (size <= 0 || (size_t)size > R_MAX_SLOT) {
        return 0;
    }
    need = (size_t)size;
    for (i = 0; i < R_SLOT_COUNT; i++) {
        if (!g_slots[i].live) {
            size_t off = (g_bump + 7u) & ~(size_t)7u;
            if (off + need > R_ARENA_BYTES) {
                return 0;
            }
            g_slots[i].live = 1;
            g_slots[i].gen += 1u;
            g_slots[i].size = need;
            g_slots[i].off = off;
            g_bump = off + need;
            g_allocs += 1u;
            return slot_handle(i);
        }
    }
    return 0;
}

static int slot_valid(long long h)
{
    uint32_t i = R_IDX(h);
    if (i >= R_SLOT_COUNT || !g_slots[i].live) {
        return 0;
    }
    return R_GEN(h) == (uint32_t)g_slots[i].gen;
}

int p797_h_set(long long h, int idx, long long v)
{
    size_t byte;
    if (!slot_valid(h)) {
        return R797_ERR_HANDLE;
    }
    byte = (size_t)idx * 8u;
    if (byte + 8u > g_slots[R_IDX(h)].size) {
        return R797_ERR_BOUNDS;
    }
    *(long long *)(g_arena + g_slots[R_IDX(h)].off + byte) = v;
    return R797_OK;
}

int p797_h_get(long long h, int idx, long long *out)
{
    size_t byte;
    if (!slot_valid(h)) {
        return R797_ERR_HANDLE;
    }
    byte = (size_t)idx * 8u;
    if (byte + 8u > g_slots[R_IDX(h)].size) {
        return R797_ERR_BOUNDS;
    }
    *out = *(long long *)(g_arena + g_slots[R_IDX(h)].off + byte);
    return R797_OK;
}

int p797_h_destroy(long long h)
{
    uint32_t i = R_IDX(h);
    if (!slot_valid(h)) {
        return R797_ERR_HANDLE;
    }
    /* scrub the payload: freed provider memory never leaks old values */
    for (size_t k = 0; k < g_slots[i].size; k++) {
        *(volatile unsigned char *)(g_arena + g_slots[i].off + k) = 0;
    }
    g_slots[i].live = 0;
    g_frees += 1u;
    return R797_OK;
}

int p797_ledger(long long *live, long long *allocs, long long *frees)
{
    long long n = 0;
    int i;
    for (i = 0; i < R_SLOT_COUNT; i++) {
        if (g_slots[i].live) {
            n++;
        }
    }
    *live = n;
    *allocs = (long long)g_allocs;
    *frees = (long long)g_frees;
    return R797_OK;
}

int p797_buf_sum(const unsigned char *p, uint64_t len, long long *out)
{
    uint64_t i;
    long long s = 0;
    if (len > 0u && p == 0) {
        return R797_ERR_BUFNULL;
    }
    for (i = 0; i < len; i++) {
        s += (long long)p[i];
    }
    *out = s;
    return R797_OK;
}

int p797_buf_copy(unsigned char *dst, uint64_t cap,
                  const unsigned char *src, uint64_t len)
{
    uint64_t i;
    if (len > cap) {
        return R797_ERR_CAP;
    }
    if (len > 0u && (dst == 0 || src == 0)) {
        return R797_ERR_BUFNULL;
    }
    for (i = 0; i < len; i++) {
        dst[i] = src[i];
    }
    return R797_OK;
}

void p797_fail_nth_alloc(int n)
{
    g_fail_nth = n;
}

int p797_two_step(int x, long long *out)
{
    long long a = p797_h_create(16);
    long long b;
    if (a == 0) {
        return R797_ERR_HANDLE; /* injected failure before any success */
    }
    if (p797_h_set(a, 0, (long long)x * 3LL) != R797_OK) {
        p797_h_destroy(a);
        return R797_ERR_HANDLE;
    }
    b = p797_h_create(16);
    if (b == 0) {
        p797_h_destroy(a); /* rollback the partial allocation */
        return R797_ERR_HANDLE;
    }
    if (p797_h_set(b, 0, (long long)x * 5LL) != R797_OK) {
        p797_h_destroy(a);
        p797_h_destroy(b);
        return R797_ERR_HANDLE;
    }
    p797_h_destroy(a); /* A is transient on the success path too */
    *out = b;
    return R797_OK;
}

long long p797_fold(long long (*cb)(long long ctx, long long item),
                    long long ctx, const long long *items, uint64_t n)
{
    long long sum = 0;
    uint64_t i;
    /* strictly synchronous per-call: the pointer is used only inside
     * this frame and never stored anywhere */
    for (i = 0; i < n; i++) {
        sum += cb(ctx, items[i]);
    }
    return sum;
}

long long p797_stored_cb(void)
{
    return 0;
}
