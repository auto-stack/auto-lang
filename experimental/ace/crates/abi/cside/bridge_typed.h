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

#endif /* BRIDGE_TYPED_H */
