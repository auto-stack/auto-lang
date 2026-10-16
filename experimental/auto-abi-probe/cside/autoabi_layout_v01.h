/*
 * autoabi_layout_v01.h — PLAN-797 T-02 independently audited C layout
 * mirror of the descriptor record set.
 *
 * Hand-maintained C, deliberately NOT generated from the descriptor: the
 * layout conformance check compares this file's compiler-measured
 * sizeof / __alignof / offsetof against the Rust layout engine, so the
 * two sides must stay independent sources. Field types are fixed-width;
 * no #pragma pack — MSVC natural alignment is the frozen v0.1 packing.
 *
 * bool  -> unsigned char (0/1)     char -> uint32_t (scalar value)
 * i32/u32 -> int32_t/uint32_t      i64/u64/f64 -> int64_t/uint64_t/double
 */

#ifndef AUTOABI_LAYOUT_V01_H
#define AUTOABI_LAYOUT_V01_H

#include <stdint.h>   /* fixed widths; header-only, no CRT linkage */

/* RecMix: trailing-scalar padding case (flag@0, count@4, tag@8, big@16). */
typedef struct RecMix {
    unsigned char flag;
    int32_t       count;
    uint32_t      tag;
    int64_t       big;
} RecMix;

/* RecPad: interior + tail padding (a@0, b@8, c@16, size 24 align 8). */
typedef struct RecPad {
    unsigned char a;
    int64_t       b;
    unsigned char c;
} RecPad;

/* RecNested: nested record by value inside a record
 * (x@0, m@8..32, d@32; size 40 align 8). */
typedef struct RecNested {
    int32_t  x;
    RecMix   m;
    double   d;
} RecNested;

/* RecOrderB: same field set as RecMix in a DIFFERENT order — proves
 * declaration order is load-bearing (big@0, flag@8, count@12, tag@16). */
typedef struct RecOrderB {
    int64_t       big;
    unsigned char flag;
    int32_t       count;
    uint32_t      tag;
} RecOrderB;

#endif /* AUTOABI_LAYOUT_V01_H */
