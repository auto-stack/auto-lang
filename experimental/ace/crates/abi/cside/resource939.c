/*
 * resource939.c — PLAN-939 hand-audited provider implementation.
 * Independent of any generated code; compiled with the same header the
 * review table pins. Every observable fact (ledger, generation, failure
 * injection, commit-once discipline) is derived from the slot table,
 * never from shadow counters.
 */
#include "resource939.h"
/* No CRT headers: the ABI worker's provider compile path pins the MSVC
 * include set only (the 938 discipline); zeroing is hand-rolled. */

#ifndef BODY_BIAS
#define BODY_BIAS 0
#endif

#ifdef SOURCE939_EVENTS
extern void source939_event(const char *name);
#define SOURCE939_EVENT(name) source939_event(name)
#else
#define SOURCE939_EVENT(name) ((void)0)
#endif

#define R939_SLOTS 512
#define R939_GEN_MAX 255

struct r939_slot {
    int32_t live;   /* 0 free, 1 live */
    int32_t gen;    /* 1..R939_GEN_MAX */
    int32_t size;
    int32_t value;  /* the payload (set == size for get) */
};

static struct r939_slot slots[R939_SLOTS];
static int64_t total_allocs = 0;
static int64_t total_frees = 0;
static int32_t fail_nth = 0;   /* 1-based; <1 = off */
static int32_t alloc_ordinal = 0;

void t939_fail_nth_alloc(int32_t n) { fail_nth = n; alloc_ordinal = 0; }

void t939_reset(void) {
    for (int32_t i = 0; i < R939_SLOTS; ++i) {
        slots[i].live = 0;
        slots[i].gen = 0;
        slots[i].size = 0;
        slots[i].value = 0;
    }
    total_allocs = 0;
    total_frees = 0;
    fail_nth = 0;
    alloc_ordinal = 0;
}

/* Every handle allocation goes through here so failure injection and the
 * alloc ordinal are observable for EVERY live handle, including the
 * transient one a rolled-back transaction created. */
static int64_t alloc_slot(int32_t size, int32_t value) {
    ++alloc_ordinal;
    if (fail_nth > 0 && alloc_ordinal == fail_nth) {
        return 0; /* injected failure — nothing allocated */
    }
    for (int32_t i = 0; i < R939_SLOTS; ++i) {
        if (!slots[i].live) {
            if (slots[i].gen + 1 > R939_GEN_MAX) {
                return 0; /* generation exhausted — handle space retired */
            }
            slots[i].live = 1;
            slots[i].gen += 1;
            slots[i].size = size;
            slots[i].value = value;
            ++total_allocs;
            return (int64_t)slots[i].gen * 1024 + (int64_t)i;
        }
    }
    return 0;
}

static struct r939_slot *slot_of(int64_t h) {
    if (h <= 0) {
        return 0;
    }
    int64_t gen = h / 1024;
    int64_t idx = h % 1024;
    if (idx >= R939_SLOTS || gen < 1 || gen > R939_GEN_MAX) {
        return 0;
    }
    struct r939_slot *s = &slots[idx];
    if (!s->live || (int64_t)s->gen != gen) {
        return 0; /* stale or forged */
    }
    return s;
}

uint64_t t939_byte_length(const uint8_t *p, uint64_t len) {
    SOURCE939_EVENT("t939_byte_length");
    (void)p;
    return len;
}

uint64_t t939_scalar_count(const uint8_t *p, uint64_t len) {
    SOURCE939_EVENT("t939_scalar_count");
    /* count UTF-8 scalar starts (non-continuation bytes); the bridge
     * validated the encoding before the call */
    uint64_t n = 0;
    for (uint64_t i = 0; i < len; ++i) {
        if ((p[i] & 0xC0u) != 0x80u) {
            ++n;
        }
    }
    return n;
}

int32_t t939_bump(Counter *r, int32_t delta, int32_t *out) {
    SOURCE939_EVENT("t939_bump");
    if (delta < 0) {
        return R939_ERR_SEED; /* EVERY caller byte untouched */
    }
    /* commit once, after every possible failure point */
    r->value = r->value + delta;
    *out = r->value;
    return R939_OK;
}

int64_t t939_create(int32_t size) {
    SOURCE939_EVENT("t939_create");
    if (size < 0) {
        return 0;
    }
    return alloc_slot(size, size);
}

int64_t t939_clone(int64_t h) {
    SOURCE939_EVENT("t939_clone");
    struct r939_slot *s = slot_of(h);
    if (!s) {
        return 0;
    }
    /* independent backing: a fresh slot with the same payload — the two
     * owners' handles are distinct storage and distinct generations */
    return alloc_slot(s->size, s->value);
}

int32_t t939_stored_cb(void) {
    SOURCE939_EVENT("t939_stored_cb");
    return 0; /* noescape: nothing is ever stored */
}

int32_t t939_get(int64_t h, int32_t *out) {
    SOURCE939_EVENT("t939_get");
    struct r939_slot *s = slot_of(h);
    if (!s) {
        return R939_ERR_HANDLE;
    }
    *out = s->value;
    return R939_OK;
}

int32_t t939_destroy(int64_t h, int32_t *out) {
    SOURCE939_EVENT("t939_destroy");
    struct r939_slot *s = slot_of(h);
    if (!s) {
        return R939_ERR_HANDLE; /* *out untouched */
    }
    s->live = 0;
    ++total_frees;
    *out = s->gen; /* the destroyed generation — exactly-once observable */
    return R939_OK;
}

int32_t t939_live(void) {
    SOURCE939_EVENT("t939_live");
    int32_t n = 0;
    for (int32_t i = 0; i < R939_SLOTS; ++i) {
        if (slots[i].live) {
            ++n;
        }
    }
    return n;
}

int32_t t939_ledger(int64_t *live, int64_t *allocs, int64_t *frees) {
    SOURCE939_EVENT("t939_ledger");
    int64_t n = 0;
    for (int32_t i = 0; i < R939_SLOTS; ++i) {
        if (slots[i].live) {
            ++n;
        }
    }
    *live = n;
    *allocs = total_allocs;
    *frees = total_frees;
    return R939_OK;
}

int32_t t939_txn(int32_t seed, TxnResult *out2, int32_t *out) {
    SOURCE939_EVENT("t939_txn");
    if (seed < 0 || seed > 2000) {
        return R939_ERR_SEED;
    }
    /* prepare EVERY output first; nothing caller-visible commits until
     * all failure points are past */
    int64_t h = alloc_slot(4, seed);
    if (h == 0) {
        return R939_ERR_ALLOC; /* transient state: none — alloc failed clean */
    }
    if (seed > 1000) {
        /* LAST-STEP failure lane: the transient handle dies here, at the
         * ORIGINAL provider's hand — a half tree never escapes (V-04). */
        struct r939_slot *s = slot_of(h);
        s->live = 0;
        ++total_frees;
        return R939_ERR_ALLOC;
    }
    TxnResult prepared;
    prepared.value = seed * 2;
    prepared.handle = (int32_t)h; /* positive, inside i32 range */
    int32_t prepared_out = seed;
    /* single commit point: both slots, in declaration order */
    *out2 = prepared;
    *out = prepared_out;
    return R939_OK;
}

/* S06 — synchronous per-call callback with non-zero ctx (noescape: the
 * provider never stores cb; ctx passes through unchanged). The ctx seed
 * is the next ordinary argument after the callback, physically
 * long long. A negative callback result is the call's failure VERBATIM
 * (never a success sentinel); i32-domain violations fail with -2 with
 * *out untouched. */
int32_t t939_fold(long long (*cb)(long long ctx, long long item),
                  long long ctx_seed, int32_t n, int32_t *out) {
    SOURCE939_EVENT("t939_fold");
    if (n < 0 || n > 64) {
        return R939_ERR_SEED; /* domain guard BEFORE any callback runs */
    }
    long long sum = 0;
    for (int32_t i = 0; i < n; ++i) {
        long long r = cb(ctx_seed, (long long)i);
        if (r < INT32_MIN || r > INT32_MAX) {
            return R939_ERR_SEED; /* the callback domain is the checked bridge's */
        }
        if (r < 0) {
            return (int32_t)r; /* the callback's failure IS the call's failure */
        }
        sum += r;
        if (sum > INT32_MAX) {
            return R939_ERR_SEED; /* total stays inside the checked out lane */
        }
    }
    *out = (int32_t)sum;
    return R939_OK;
}

/* S03 — two record lanes in one call. The ALIAS rejection is a verifier
 * fact (same owner into two mutable lanes never reaches the provider);
 * these entries prove the distinct-owners positive and the byte-exact
 * failure discipline for the pair shape. */
int32_t t939_pair(Counter *a, Counter *b, int32_t *out) {
    SOURCE939_EVENT("t939_pair");
    if (a == NULL || b == NULL || out == NULL) {
        return R939_ERR_SEED; /* defensive; the bridge pre-validates */
    }
    a->value += 1;
    b->value += 2;
    *out = a->value + b->value;
    return R939_OK;
}

int32_t t939_pairv(Counter *r, const Counter *v, int32_t *out) {
    SOURCE939_EVENT("t939_pairv");
    if (r == NULL || v == NULL || out == NULL) {
        return R939_ERR_SEED;
    }
    r->value += 1;
    *out = r->value + v->value;
    return R939_OK;
}
