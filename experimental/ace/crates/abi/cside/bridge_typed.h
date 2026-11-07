/*
 * bridge_typed.h — PLAN-937 CA-D1 typed-lane C-side contract surface.
 *
 * Hand-maintained, independently audited C — deliberately NOT generated
 * from the foreign-contract facts/plan. The Rust side (typed_decl.rs +
 * foreign-contract) and this file are two independent sources; the
 * bridge legs compare them by EXECUTION, never by syncing text.
 *
 * Platform: x86_64-pc-windows-msvc, Windows x64 calling convention
 * (POSITIONAL slots: positions 0-3 in RCX/RDX/R8/R9 or XMM0..3 by lane
 * class, positions 4+ on the stack — same rule the foreign-contract
 * plan spells and both compilers implement).
 *
 * Freestanding lane spellings (compiled -GS- -Zl, no CRT headers):
 *   model lane    model c_spell   this header spells
 *   i32           int32_t         int
 *   u32           uint32_t        unsigned int
 *   i64           int64_t         long long
 *   u64           uint64_t        unsigned long long
 *   f64           double          double
 *   bool (u8)     bool            unsigned char   (values exactly 0/1)
 *   char (u32)    char32_t        unsigned int    (Unicode scalar value)
 */

#ifndef BRIDGE_TYPED_H
#define BRIDGE_TYPED_H

/* Frozen mix constants (bitwise-identical to typed_decl.rs T937_MIX_C
 * and the Cranelift body — three independent carriers of one table). */
#define T937_C0 0x9E3779B97F4A7C15ULL
#define T937_C1 0xC2B2AE3D27D4EB4FULL
#define T937_C2 0x165667B19E3779F9ULL
#define T937_C3 0x27D4EB2F165667C5ULL
#define T937_C4 0x85EBCA77C2B2AE63ULL
#define T937_C5 0x2545F4914F6DDC1DULL
#define T937_C6 0x9E3779B185EBCA87ULL
#define T937_C7 0xF39CC0605CEDC834ULL

/* Frozen typed inputs (asymmetric extremes; f64 lane carries a quiet
 * NaN with payload 0xDEADBEEF — the fold must see the RAW BITS). */
#define T937_A (-1234567)
#define T937_B 0xDEADBEEFU
#define T937_C ((long long)(-9223372036854775685LL))
#define T937_D 0xFFFFFFFFFFFFFFF8ULL
#define T937_F_BITS 0x7FF80000DEADBEEFULL
#define T937_G 1u
#define T937_H 0x0010FFFFu
#define T937_I 0x7FFFFFFFFFFFF000LL

/* Hand-computed fold of the frozen inputs (independent Python fold):
 * 5791786002463203090 == 0x50608EAEDBE61F12. */
#define T937_MIX_EXPECTED 5791786002463203090LL

/* Fallible-div statuses (the explicit table the facts carry). */
#define T937_DIV_OK 0
#define T937_DIV_ZERO (-1)

/* PE exit codes: 0 = all in-PE checks passed; the exit value is the
 * number of FAILED checks. */
#define T937_EXIT_PASS 0

/* The typed public surface (physical forms of the neutral plan):
 * pure heterogeneous checksum + fallible div with explicit nullable
 * ctx, i64 out slot, i32 status. */
long long t937_mix(int p0, unsigned int p1, long long p2,
                   unsigned long long p3, double p4, unsigned char p5,
                   unsigned int p6, long long p7);
int t937_div(void *ctx, long long n, long long d, long long *out);

/* ------------------------------------------------------------------
 * PLAN-939 S07 — the WIDE typed-resource families. Frozen shapes:
 *  - BufViewI64Sum  (const long long *p, u64 n) -> i64, n counts
 *    ELEMENTS (never bytes);
 *  - RecMixSum      (const RecMixT *r) -> i64 over the frozen wide
 *    field closure flag/count/tag/big under natural layout;
 *  - WideFold       the ORIGINAL wide synchronous callback
 *    long long (*)(long long ctx, long long item) invoked per element
 *    (never stored — strictly per-call).
 * This header's RecMixT is an INDEPENDENT hand-maintained mirror of
 * the frozen closure; the negative-array typedefs pin size/align at
 * every compile (layout drift fails -W4 -WX loudly).
 * ------------------------------------------------------------------ */

typedef struct RecMixT {
    unsigned char flag; /* 0/1            @0  */
    int           count; /* i32            @4  */
    unsigned int  tag; /* char scalar value @8 */
    long long     big; /* i64            @16 */
} RecMixT;

typedef char t939w_recmix_size[(sizeof(RecMixT) == 24) ? 1 : -1];
typedef char t939w_recmix_align[(__alignof(RecMixT) == 8) ? 1 : -1];
typedef char t939w_lanes[(sizeof(long long) == 8 && sizeof(int) == 4) ? 1 : -1];

/* The frozen i64 element array (asymmetric extremes) and prefixes. */
#define T939W_I0 0x7FFFFFFFFFFFF001LL
#define T939W_I1 (-1234567890123LL)
#define T939W_I2 0LL
#define T939W_I3 (-1LL)
#define T939W_I4 (-9223372036854775801LL) /* i64::MIN + 7 */
#define T939W_I5 42LL
#define T939W_I6 (-9223372036854775807LL - 1) /* i64::MIN */
#define T939W_I7 9223372036854775807LL         /* i64::MAX */

/* The wide fold's non-zero WIDE ctx seed (beyond the i32 domain). */
#define T939W_FOLD_CTX 0x000BAD1DEA600DLL

/* The frozen RecMix field values (flag 0/1, tag U+10FFFF, big near MIN). */
#define T939W_REC_FLAG 1u
#define T939W_REC_COUNT (-123456)
#define T939W_REC_TAG 0x0010FFFFu
#define T939W_REC_BIG (-9223372036854775801LL)

/* Hand-computed expectations (independent Python folds of the frozen
 * inputs — NOT derived from any generator; guard the C bodies). */
#define T939W_SUM_8 (-1234567894171LL)
#define T939W_SUM_3 9223370802286881590LL
#define T939W_SUM_1 9223372036854771713LL
#define T939W_REC (-9223372036849574614LL)
#define T939W_FOLD_8 1995608927678LL
#define T939W_FOLD_3 (-36518868513495LL)
#define T939W_CB_AT_0 38514477441063LL /* cb(ctx, 0) */

/* The frozen wide callback body: (ctx ^ item) * 3 + item on the
 * two's-complement bit patterns (machine wrapping). */
long long t939_wide_sum(const long long *p, unsigned long long n);
long long t939_crec_sum(const RecMixT *r);
long long t939_wide_fold(long long (*cb)(long long ctx, long long item),
                         long long ctx, const long long *items,
                         unsigned long long n);

#endif /* BRIDGE_TYPED_H */
