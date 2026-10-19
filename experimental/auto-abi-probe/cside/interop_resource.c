/*
 * interop_resource.c — PLAN-797 T-04 provider implementation.
 *
 * R5 repairs (F-797-R5-01/02):
 *  - R5-01: a create that cannot find a usable slot fails BEFORE any
 *    arena/free-list capacity is taken — failed allocations never leak.
 *  - R5-02: the handle generation is a consistent 16-bit field; a slot
 *    whose generation would wrap is RETIRED (never reused) instead of
 *    minting an invalid "success" handle.
 *
 * R4 repairs (F-797-R4-04/05/06):
 *  - F04: h_set/h_get reject NEGATIVE indices and use checked arithmetic
 *    (no unsigned wrap past the slot bounds).
 *  - F05: handles embed a PROVIDER IDENTITY (the address-derived token of
 *    THIS compilation's slot table), so a handle minted by a different
 *    provider — even with matching index/generation — is rejected.
 *  - F06: destroy RETURNS the block to a free list (first-fit, split,
 *    coalesce); create/destroy cycles reclaim real arena capacity.
 *
 * The ledger is still the slot table itself; p797_ledger derives its
 * facts from it. Real allocation, real reclamation, provider-owned
 * destroy, generation handles, failure injection with rollback,
 * borrowed-buffer rules, strictly synchronous callbacks.
 */

#include "interop_resource.h"

/* Freestanding notes: /O2 would fold byte loops into CRT memset calls
 * (an unresolved symbol under -Zl; memset is also an MSVC intrinsic
 * that user code may not define), so scrub loops write through
 * volatile to stay byte loops. No _fltused here: this TU has no FP
 * literals, and a second provider TU (interop_resource_b.c) links
 * beside this one. */

#define R_ARENA_BYTES  (1u << 20)
#define R_SLOT_COUNT   256
#define R_MAX_SLOT     16384
#define R_GEN_MAX      0xFFFFu /* R5-02: 16-bit generation domain; a
                                * slot at the ceiling is retired. */

static unsigned char g_arena[R_ARENA_BYTES];
static struct {
    int live;
    int retired;  /* R5-02: generation exhausted — never reused */
    uint32_t gen; /* bumps on each reuse; 0 = never used */
    size_t size;
    size_t off;
} g_slots[R_SLOT_COUNT];
static size_t g_bump = 0;          /* arena bump pointer */
static uint64_t g_allocs = 0;      /* ledger facts (derived use only) */
static uint64_t g_frees = 0;
static int g_fail_nth = -1;        /* 1-based; -1 = off */

/* Free list of reclaimed blocks (F06): first-fit with split; destroy
 * inserts back and coalesces with physically adjacent free blocks. */
#define R_FREE_MAX 512
static struct {
    size_t off;
    size_t size;
} g_free[R_FREE_MAX];
static int g_free_n = 0;

/* Provider identity (F05): a per-compilation token. Two independently
 * compiled providers (even with renamed exports in one PE) get
 * distinct tokens, so a foreign handle cannot validate here. The token
 * is WRITABLE static storage (never COMDAT/ICF-folded by the linker,
 * unlike identical read-only constants) and mixes in this provider's
 * arena address for the same reason. */
static char g_provider_token = 0x5A;

static uint32_t provider_id(void)
{
    uintptr_t a = (uintptr_t)(void *)&g_provider_token;
    uintptr_t b = (uintptr_t)(void *)&g_arena[0];
    uintptr_t m = a * 0x9E3779B97F4A7C15ULL ^ b * 0xC2B2AE3D27D4EB4FULL;
    m ^= m >> 29;
    m *= 0xFF51AFD7ED558CCDULL;
    m ^= m >> 32;
    return (uint32_t)(m >> 16);
}

#define R_IDX(h)   ((uint32_t)((uint64_t)(h) & 0xFFFFu))
#define R_GEN(h)   ((uint32_t)(((uint64_t)(h) >> 16) & 0xFFFFu))
#define R_PROV(h)  ((uint32_t)(((uint64_t)(h) >> 32) & 0xFFFFFFFFu))

static long long slot_handle(int i)
{
    /* R5-02: pack the generation as the SAME 16-bit field R_GEN reads
     * — provider id in the high 32 bits, generation the middle 16,
     * index the low 16. */
    return ((long long)(uint32_t)provider_id() << 32)
         | ((long long)(uint32_t)(g_slots[i].gen & R_GEN_MAX) << 16)
         | (long long)(uint32_t)i;
}

/* --- free-list management (F06) --------------------------------------- */

static void freelist_insert(size_t off, size_t size)
{
    /* coalesce with physically adjacent free blocks */
    for (int k = 0; k < g_free_n; k++) {
        if (g_free[k].off + g_free[k].size == off) {
            g_free[k].size += size;
            /* absorb a following neighbor too */
            for (int j = 0; j < g_free_n; j++) {
                if (g_free[k].off + g_free[k].size == g_free[j].off) {
                    g_free[k].size += g_free[j].size;
                    g_free[j] = g_free[--g_free_n];
                    break;
                }
            }
            return;
        }
        if (off + size == g_free[k].off) {
            g_free[k].off = off;
            g_free[k].size += size;
            return;
        }
    }
    if (g_free_n < R_FREE_MAX) {
        g_free[g_free_n].off = off;
        g_free[g_free_n].size = size;
        g_free_n++;
    }
    /* list full: capacity stays bump-resident (still correct, just less
     * reclaim until a slot frees); never a correctness failure */
}

static int freelist_take(size_t need, size_t *out_off)
{
    int best = -1;
    for (int k = 0; k < g_free_n; k++) {
        if (g_free[k].size >= need && (best < 0 || g_free[k].size < g_free[best].size)) {
            best = k;
        }
    }
    if (best < 0) {
        return 0;
    }
    *out_off = g_free[best].off;
    if (g_free[best].size - need >= 8) {
        /* split: keep the tail free */
        g_free[best].off += need;
        g_free[best].size -= need;
    } else {
        g_free[best] = g_free[--g_free_n];
    }
    return 1;
}

/* --- owned-handle lifecycle ------------------------------------------- */

long long p797_h_create(int size)
{
    int i;
    size_t need, off;
    int from_freelist;

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

    /* R5-01: find a usable slot FIRST — a full/retired table fails
     * before ANY arena or free-list capacity is taken, so a failed
     * create can never leak. R5-02: slots at the generation ceiling are
     * retired, never reused (a wrapped generation would mint a handle
     * whose very first access compares wrong). */
    for (i = 0; i < R_SLOT_COUNT; i++) {
        if (!g_slots[i].live) {
            if (g_slots[i].gen >= R_GEN_MAX) {
                g_slots[i].retired = 1;
                continue;
            }
            break;
        }
    }
    if (i >= R_SLOT_COUNT) {
        return 0; /* no capacity consumed */
    }

    /* capacity is taken only now (slot known usable); on a bump/arena
     * failure nothing was taken either (freelist_take returned 0). */
    from_freelist = freelist_take(need, &off);
    if (!from_freelist) {
        off = (g_bump + 7u) & ~(size_t)7u;
        if (off + need > R_ARENA_BYTES) {
            return 0;
        }
        g_bump = off + need;
    }
    /* slot could have been taken? single-threaded probe: no. */
    g_slots[i].live = 1;
    g_slots[i].gen += 1u;
    g_slots[i].size = need;
    g_slots[i].off = off;
    g_allocs += 1u;
    return slot_handle(i);
}

static int slot_valid(long long h)
{
    uint32_t i = R_IDX(h);
    if (i >= R_SLOT_COUNT || !g_slots[i].live) {
        return 0;
    }
    /* F05: the handle must carry THIS provider's identity — a foreign
     * provider's handle with a coincidental index/generation is
     * rejected before any memory access. */
    if (R_PROV(h) != provider_id()) {
        return 0;
    }
    return R_GEN(h) == (g_slots[i].gen & R_GEN_MAX);
}

/* F04: checked byte-range math for slot-local indexing. */
static int slot_bounds(size_t size, int idx)
{
    size_t byte;
    if (idx < 0) {
        return 0; /* negative index: never wraps into a legal window */
    }
    byte = (size_t)idx * 8u;
    if (byte > size || size - byte < 8u) {
        return 0;
    }
    return 1;
}

int p797_h_set(long long h, int idx, long long v)
{
    uint32_t i;
    if (!slot_valid(h)) {
        return R797_ERR_HANDLE;
    }
    i = R_IDX(h);
    if (!slot_bounds(g_slots[i].size, idx)) {
        return R797_ERR_BOUNDS;
    }
    *(long long *)(g_arena + g_slots[i].off + (size_t)idx * 8u) = v;
    return R797_OK;
}

int p797_h_get(long long h, int idx, long long *out)
{
    uint32_t i;
    if (!slot_valid(h)) {
        return R797_ERR_HANDLE;
    }
    i = R_IDX(h);
    if (!slot_bounds(g_slots[i].size, idx)) {
        return R797_ERR_BOUNDS;
    }
    *out = *(long long *)(g_arena + g_slots[i].off + (size_t)idx * 8u);
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
    freelist_insert(g_slots[i].off, g_slots[i].size); /* F06: real reclaim */
    g_slots[i].live = 0;
    /* R5-02: retire at the ceiling so the next create cannot wrap */
    if (g_slots[i].gen >= R_GEN_MAX) {
        g_slots[i].retired = 1;
    }
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
        p797_h_destroy(a);
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

/* F-797-R4-02: strict UTF-8 validation (overlongs, surrogates, >U+10FFFF
 * and truncated sequences are all rejected); on success *chars is the
 * SCALAR COUNT. The out slot is only written on success. */
int p797_cbuf_sum_utf8(const unsigned char *p, uint64_t len, long long *chars)
{
    uint64_t i = 0, count = 0;
    if (len > 0u && p == 0) {
        return R797_ERR_BUFNULL;
    }
    while (i < len) {
        unsigned char b = p[i];
        if (b < 0x80u) {
            i += 1u;
        } else if ((b & 0xE0u) == 0xC0u) {
            if (b < 0xC2u) {
                return R797_ERR_UTF8; /* overlong 2-byte */
            }
            if (i + 1u >= len || (p[i + 1u] & 0xC0u) != 0x80u) {
                return R797_ERR_UTF8;
            }
            i += 2u;
        } else if ((b & 0xF0u) == 0xE0u) {
            if (i + 2u >= len) {
                return R797_ERR_UTF8;
            }
            if ((p[i + 1u] & 0xC0u) != 0x80u || (p[i + 2u] & 0xC0u) != 0x80u) {
                return R797_ERR_UTF8;
            }
            if (b == 0xE0u && p[i + 1u] < 0xA0u) {
                return R797_ERR_UTF8; /* overlong 3-byte */
            }
            if (b == 0xEDu && p[i + 1u] >= 0xA0u) {
                return R797_ERR_UTF8; /* surrogate */
            }
            i += 3u;
        } else if ((b & 0xF8u) == 0xF0u) {
            if (b > 0xF4u) {
                return R797_ERR_UTF8; /* > U+10FFFF */
            }
            if (i + 3u >= len) {
                return R797_ERR_UTF8;
            }
            if ((p[i + 1u] & 0xC0u) != 0x80u || (p[i + 2u] & 0xC0u) != 0x80u
                || (p[i + 3u] & 0xC0u) != 0x80u) {
                return R797_ERR_UTF8;
            }
            if (b == 0xF0u && p[i + 1u] < 0x90u) {
                return R797_ERR_UTF8; /* overlong 4-byte */
            }
            if (b == 0xF4u && p[i + 1u] >= 0x90u) {
                return R797_ERR_UTF8; /* > U+10FFFF */
            }
            i += 4u;
        } else {
            return R797_ERR_UTF8; /* stray continuation / invalid lead */
        }
        count += 1u;
    }
    *chars = (long long)count;
    return R797_OK;
}

/* F-797-R4-02: record-view VALUE validation — bool must be 0/1, char
 * must be a scalar value (<= U+10FFFF, not a surrogate); a valid record
 * writes the weighted sum. The out slot is only written on success. */
int p797_crec_valid(const RecMix *r, long long *sum)
{
    if (r == 0) {
        return R797_ERR_BUFNULL;
    }
    if (r->flag > 1u) {
        return R797_ERR_INVALID;
    }
    if (r->tag > 0x10FFFFu || (r->tag >= 0xD800u && r->tag <= 0xDFFFu)) {
        return R797_ERR_INVALID;
    }
    *sum = (long long)r->flag * 1000LL + (long long)r->count * 3LL
         + (long long)r->tag * 5LL + r->big;
    return R797_OK;
}

/* F-797-R4-02 (R8): record MUT VIEW — the commit/rollback control the
 * profile promises. Validation happens BEFORE any store to the view:
 * a value failure returns with every caller byte untouched. Success
 * mutates in place and commits the weighted sum. */
int p797_crec_mutate(RecMix *r, long long seed, long long *sum)
{
    if (r == 0) {
        return R797_ERR_BUFNULL;
    }
    if (r->flag > 1u) {
        return R797_ERR_INVALID;
    }
    if (r->tag > 0x10FFFFu || (r->tag >= 0xD800u && r->tag <= 0xDFFFu)) {
        return R797_ERR_INVALID;
    }
    r->count += 1;
    r->big ^= seed;
    /* the committed sum reflects the MUTATED record */
    *sum = (long long)r->flag * 1000LL + (long long)r->count * 3LL
         + (long long)r->tag * 5LL + r->big;
    return R797_OK;
}

/* F-797-R4-02 (R8): caller-owned OUT record — committed only on
 * success; a rejected seed leaves the caller slot byte-identical. */
int p797_crec_make(int seed, RecMix *out)
{
    if (out == 0) {
        return R797_ERR_BUFNULL;
    }
    if (seed < 0 || seed > 1000) {
        return R797_ERR_INVALID;
    }
    out->flag = 1u;
    out->count = (int32_t)seed;
    out->tag = 0x41u;
    out->big = (long long)seed * 7LL;
    return R797_OK;
}
