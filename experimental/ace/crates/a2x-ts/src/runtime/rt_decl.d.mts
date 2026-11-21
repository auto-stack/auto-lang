// ---------------------------------------------------------------------------
// TypeScript declarations for the a2ts runtime body (PLAN-976).
//
// The runtime itself is plain ESM JavaScript (`a2x_rt.mjs`); this declaration
// is shipped next to it as `a2x_rt.d.mts` so the TypeScript leg can type-check
// generated programs, and so `tsc --noEmit` sees the same helper catalogue the
// JavaScript leg imports. Every runtime export is declared here exactly once.
//
// Representation rules the types encode:
//   * i32/i8/u8/u32 values are `number` (checked at every operation);
//   * i64/u64 values are `bigint` end to end (never `number`);
//   * borrowed text is `Uint8Array`, owned text is `A2xOwnedStr`.
// ---------------------------------------------------------------------------

/** Borrowed UTF-8 bytes (`str`). */
type A2xBorrowedText = Uint8Array;
/** Either text representation accepted by the runtime helpers. */
type A2xText = Uint8Array | A2xOwnedStr;

/** Version tag of the runtime module (part of every port contract). */
export declare const A2X_RUNTIME_VERSION: string;

/** Deterministic SHA-256 of the emitted runtime body. */
export declare const A2X_RUNTIME_BODY_SHA256: string;

/** `A2X_RUNTIME_VERSION + "@" + A2X_RUNTIME_BODY_SHA256`. */
export declare const A2X_PORT_DEP: string;

/** Script language of the emitted program (`"typescript"` / `"javascript"`). */
export declare const A2X_TARGET_LANG: string;

/** A deterministic Auto-semantics trap (never a host exception). */
export declare class A2xTrap extends Error {
  readonly code: string;
  readonly detail: string;
  readonly port_id: string | null;
  constructor(code: string, detail?: string, port_id?: string | null);
}

/** Owned `Str`: ledger-registered UTF-8 buffer with explicit release. */
export declare class A2xOwnedStr {
  readonly id: number;
  bytes: Uint8Array;
  freed: boolean;
  constructor(bytes: Uint8Array);
  /** Byte length (0 once released). */
  readonly length: number;
  /** Ownership brand (true for owned `Str`). */
  isOwnedStr(): boolean;
  /** Borrow the owned bytes; traps `ResourceClosed` after release. */
  as_bytes(): Uint8Array;
  /** Release the buffer; a second release traps `DoubleClose`. */
  explicit_free(): void;
}

/** Growable list carrying its checked element-kind tag. */
export declare class A2xList {
  items: unknown[];
  elemKind: string;
  constructor(elemKind?: string);
}

/** Structural variant value with one live arm. */
export declare class A2xVariant {
  disc: number;
  payload: unknown;
  constructor(disc: number, payload?: unknown);
  readonly tag: number;
}

/** Reset the whole runtime ledger (fresh program run). */
export declare function a2x_rt_reset(): void;

/** Plain, JSON-friendly snapshot of the runtime ledger. */
export declare function a2x_rt_snapshot(): {
  runtime_version: string;
  alloc_count: number;
  free_count: number;
  double_free_count: number;
  live_bytes: number;
  live_strs: number[];
  marks: string[];
  budget_left: number;
};

/** Charge `units` against the execution budget (traps `Budget`). */
export declare function a2x_bill(units: number): void;

// --- i32 family ------------------------------------------------------------

/** Validate/return a checked i32. */
export declare function a2x_check_i32(v: number): number;
/** Validate/return an integer inside `[lo, hi]`. */
export declare function a2x_check_range(v: number, lo: number, hi: number): number;
/** Checked i32 addition. */
export declare function a2x_add(a: number, b: number): number;
/** Checked i32 subtraction. */
export declare function a2x_sub(a: number, b: number): number;
/** Checked i32 multiplication. */
export declare function a2x_mul(a: number, b: number): number;
/** Checked i32 negation (INT_MIN traps). */
export declare function a2x_neg(a: number): number;
/** Truncating-toward-zero i32 division. */
export declare function a2x_div(a: number, b: number): number;
/** Truncating-toward-zero i32 remainder. */
export declare function a2x_rem(a: number, b: number): number;
/** Unchecked C truncating division. */
export declare function a2x_c_trunc_div(a: number, b: number): number;
/** Unchecked C truncating remainder. */
export declare function a2x_c_trunc_rem(a: number, b: number): number;
/** Shift-left inside a `width`-bit lane (amount masked). */
export declare function a2x_shl(a: number, b: number, width: number): number;
/** Arithmetic shift-right inside a `width`-bit lane. */
export declare function a2x_shr(a: number, b: number, width: number): number;

// --- i64 / u64 family (bigint end to end) ---------------------------------

/** Validate/return a bigint inside `[lo, hi]`. */
export declare function a2x_bcheck(v: bigint, lo: bigint, hi: bigint): bigint;
/** Checked bigint addition. */
export declare function a2x_badd(v: bigint, w: bigint, lo: bigint, hi: bigint): bigint;
/** Checked bigint subtraction. */
export declare function a2x_bsub(v: bigint, w: bigint, lo: bigint, hi: bigint): bigint;
/** Checked bigint multiplication. */
export declare function a2x_bmul(v: bigint, w: bigint, lo: bigint, hi: bigint): bigint;
/** Truncating bigint division (zero traps). */
export declare function a2x_bdiv(v: bigint, w: bigint, lo: bigint, hi: bigint): bigint;
/** Truncating bigint remainder (zero traps). */
export declare function a2x_brem(v: bigint, w: bigint, lo: bigint, hi: bigint): bigint;
/** Checked bigint negation. */
export declare function a2x_bneg(v: bigint, lo: bigint, hi: bigint): bigint;
/** Shift-left of a bigint lane (amount masked, then range-checked). */
export declare function a2x_bshl(
  v: bigint,
  shift: number | bigint,
  width: number,
  lo: bigint,
  hi: bigint,
): bigint;
/** Shift-right of a bigint lane (arithmetic when signed, logical when unsigned). */
export declare function a2x_bshr(
  v: bigint,
  shift: number | bigint,
  width: number,
  lo: bigint,
  hi: bigint,
): bigint;

// --- casts and floats ------------------------------------------------------

/** Materialise the f32 value of a u32 bit pattern. */
export declare function a2x_f32_bits(bits: number): number;
/** Materialise the f64 value of a u64 bit pattern. */
export declare function a2x_f64_bits(bits: bigint | number): number;
/** Round to f32 precision. */
export declare function a2x_f32_round(x: number): number;
/** Cast to f32. */
export declare function a2x_cast_f32(v: number | bigint): number;
/** Cast to f64. */
export declare function a2x_cast_f64(v: number | bigint): number;
/** IEEE float division (never traps). */
export declare function a2x_fdiv(a: number | bigint, b: number | bigint): number;
/** number -> bigint, truncating toward zero. */
export declare function a2x_cast_big(v: number | bigint): bigint;
/** bigint -> number after an explicit range check. */
export declare function a2x_cast_num_from_big(v: bigint, lo: bigint, hi: bigint): number;
/** number narrowing with an explicit range check. */
export declare function a2x_cast_num(v: number, lo: number, hi: number): number;
/** Validate a Unicode scalar. */
export declare function a2x_char_to_codepoint(c: number): number;
/** Validate a Unicode scalar (no surrogates). */
export declare function a2x_char_from_codepoint(n: number): number;

// --- text ------------------------------------------------------------------

/** Encode a JS string literal to a borrowed (immortal) byte view. */
export declare function a2x_str_lit(s: string | Uint8Array): A2xBorrowedText;
/** Decode a text value to a JS string (diagnostics / trap details). */
export declare function a2x_str_utf8(x: A2xText): string;
export declare function a2x_str_find(text: A2xText, pattern: A2xText, start: number): number;
/** Unicode scalar (code point) count. */
export declare function a2x_str_len(x: A2xText): number;
/** UTF-8 byte count. */
export declare function a2x_str_len_bytes(x: A2xText): number;
/** Unicode scalar at scalar index `i`. */
export declare function a2x_str_char_at(x: A2xText, i: number): number;
/** Borrowed byte slice of the scalar range `[start, end)`. */
export declare function a2x_str_slice(
  x: A2xText,
  start: number,
  end: number,
): A2xBorrowedText;
/** Concatenate two text values into a fresh owned `Str`. */
export declare function a2x_str_concat(l: A2xText, r: A2xText): A2xOwnedStr;
/** Fresh owned copy of any text value. */
export declare function a2x_str_clone(x: A2xText): A2xOwnedStr;
/** Fresh owned copy of any text value (ownership conversion). */
export declare function a2x_str_to_owned(x: A2xText): A2xOwnedStr;
/** Borrow any text value without copying or ledger change. */
export declare function a2x_str_to_borrowed(x: A2xText): A2xBorrowedText;
/** Owned `Str` holding exactly one Unicode scalar. */
export declare function a2x_str_from_char(c: number): A2xOwnedStr;
/** Release an owned `Str`; `null`/`undefined` are a no-op. */
export declare function a2x_str_free(x: A2xText | null | undefined): void;
/** Byte equality of two text values. */
export declare function a2x_str_eq(l: A2xText, r: A2xText): boolean;
/** Byte inequality of two text values. */
export declare function a2x_str_ne(l: A2xText, r: A2xText): boolean;
/** Owned `Str` of the decimal representation of a checked i32. */
export declare function a2x_str_display_i32(v: number): A2xOwnedStr;
export declare function a2x_str_display_f32(v: number): A2xOwnedStr;

// --- collections -----------------------------------------------------------

/** New list (`elemKind` defaults to `"any"`). */
export declare function a2x_list_new(elemKind?: string): A2xList;
/** Current list length. */
export declare function a2x_list_len(l: A2xList): number;
/** Append one element. */
export declare function a2x_list_push(l: A2xList, v: unknown): void;
/** Insert one element at `i` (`0..len`). */
export declare function a2x_list_insert(l: A2xList, i: number, v: unknown): void;
/** Copy one element out at `i`. */
export declare function a2x_list_get(l: A2xList, i: number): any;
/** Overwrite one element at `i`. */
export declare function a2x_list_set(l: A2xList, i: number, v: unknown): void;
/** Remove and return one element at `i`. */
export declare function a2x_list_remove(l: A2xList, i: number): any;
/** Drop every element. */
export declare function a2x_list_clear(l: A2xList): void;
/** Move the payload of `disc` out of a variant (destructive projection). */
export declare function a2x_variant_project(v: A2xVariant, disc: number): any;
/** Cooperative poll latch (no scheduler in this target: a no-op). */
export declare function a2x_coll_poll(): void;

// --- trace marks -----------------------------------------------------------

/** Trace mark A (pushes `mark_a`, writes stderr, returns 1). */
export declare function a2x_mark_a(): number;
/** Trace mark B (pushes `mark_b`, writes stderr, returns 2). */
export declare function a2x_mark_b(): number;

// --- ports -----------------------------------------------------------------

/** Dispatch a standard operation after validating its port contract. */
export declare function a2x_port_call(
  op: string,
  semver: number,
  modes: readonly string[],
  dep: string,
  ...args: any[]
): any;

// --- entry driver ----------------------------------------------------------

/** Run the program entry and emit `value:<n>` / `trap:<code>`; 0 or 1. */
export declare function a2x_run_main(entry: () => unknown): number;
