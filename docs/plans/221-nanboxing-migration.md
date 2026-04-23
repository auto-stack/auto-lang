# Plan 221: AutoVM NaN-boxing Migration

> **Status: DESIGN APPROVED**
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace AutoVM's `Vec<i32>` stack with NaN-boxed `Vec<u64>` (NanoValue), providing unified 8-byte value representation for all types — int, float, string, bool, null, object, list.

**Motivation:** Current i32-only stack requires hacky string tagging (`-(idx+1)`), multi-slot f64/i64, and prevents runtime type information at FFI boundaries. NaN-boxing solves all three with a single representation change.

**Architecture:** Mirror JSC/SpiderMonkey NaN-boxing — pack type tag into unused NaN bit patterns of IEEE 754 doubles. f64 operations zero overhead (direct bit reinterpret), all other types use tagged NaN-boxed encoding.

---

## Section 1: NaN-boxing Encoding Scheme

```
bit 63      62-52          51-48   47-32        31-0
┌────┬───────────────┬───────┬────────┬──────────┐
│ 1  │ 11111111111   │ tag   │ 0000   │ payload  │
│    │ (NaN exponent)│ 4bit  │        │ 32 bit   │
└────┴───────────────┴───────┴────────┴──────────┘

Tag values:
  0000 = f64 (direct bit pattern, NOT NaN-boxed — checked by "high 12 bits != 0xFFF")
  0001 = i32 (payload = integer value)
  0010 = string (payload = string pool index)
  0011 = bool  (payload = 0 or 1)
  0100 = null  (payload = 0)
  0101 = object (payload = object id)
  0110 = list   (payload = list id)
  0111 = f32    (payload = f32 bit pattern)
  1000-1111 = reserved for future types

Detection:
  High 12 bits == 0xFFF → NaN-boxed tagged value
  High 12 bits != 0xFFF → normal f64, use directly
```

**Key property:** Normal f64 values have exponent bits that are NOT all 1s, so they never collide with NaN-boxed values. f64 operations are truly zero-overhead (just reinterpret bits).

---

## Section 2: Value Type API

Location: `crates/auto-val/src/nano_value.rs`

```rust
pub type NanoValue = u64;

// Tag constants (placed at bits 51-48)
const TAG_F64:    u64 = 0x0000_0000_0000_0000;
const TAG_I32:    u64 = 0x0001_0000_0000_0000;
const TAG_STRING: u64 = 0x0002_0000_0000_0000;
const TAG_BOOL:   u64 = 0x0003_0000_0000_0000;
const TAG_NULL:   u64 = 0x0004_0000_0000_0000;
const TAG_OBJECT: u64 = 0x0005_0000_0000_0000;
const TAG_LIST:   u64 = 0x0006_0000_0000_0000;
const TAG_F32:    u64 = 0x0007_0000_0000_0000;

const NANBOX_BASE: u64 = 0xFFF0_0000_0000_0000;

// Detection
#[inline] pub fn is_nanboxed(v: NanoValue) -> bool { (v >> 52) == 0xFFF }

// Encode (all #[inline])
pub fn encode_f64(f: f64) -> NanoValue    { f.to_bits() }
pub fn encode_i32(i: i32) -> NanoValue    { NANBOX_BASE | TAG_I32 | ((i as u32) as u64) }
pub fn encode_string(idx: u32) -> NanoValue { NANBOX_BASE | TAG_STRING | (idx as u64) }
pub fn encode_bool(b: bool) -> NanoValue  { NANBOX_BASE | TAG_BOOL | (b as u64) }
pub fn encode_null() -> NanoValue         { NANBOX_BASE | TAG_NULL }
pub fn encode_object(id: u32) -> NanoValue { NANBOX_BASE | TAG_OBJECT | (id as u64) }
pub fn encode_list(id: u32) -> NanoValue  { NANBOX_BASE | TAG_LIST | (id as u64) }
pub fn encode_f32(f: f32) -> NanoValue    { NANBOX_BASE | TAG_F32 | (f.to_bits() as u64) }

// Decode
pub fn decode_f64(v: NanoValue) -> f64    { f64::from_bits(v) }
pub fn decode_i32(v: NanoValue) -> i32    { (v & 0xFFFF_FFFF) as i32 }
pub fn decode_string(v: NanoValue) -> u32 { (v & 0xFFFF_FFFF) as u32 }
pub fn decode_bool(v: NanoValue) -> bool  { (v & 0xFFFF_FFFF) != 0 }
pub fn decode_object(v: NanoValue) -> u32 { (v & 0xFFFF_FFFF) as u32 }
pub fn decode_list(v: NanoValue) -> u32   { (v & 0xFFFF_FFFF) as u32 }
pub fn decode_f32(v: NanoValue) -> f32    { f32::from_bits((v & 0xFFFF_FFFF) as u32) }

// Type query
pub fn tag_of(v: NanoValue) -> u64 {
    if is_nanboxed(v) { (v >> 48) & 0xF } else { 0 } // 0 = TAG_F64
}
pub fn is_f64(v: NanoValue) -> bool    { !is_nanboxed(v) }
pub fn is_i32(v: NanoValue) -> bool    { tag_of(v) == 1 }
pub fn is_string(v: NanoValue) -> bool { tag_of(v) == 2 }
pub fn is_bool(v: NanoValue) -> bool   { tag_of(v) == 3 }
pub fn is_null(v: NanoValue) -> bool   { tag_of(v) == 4 }
pub fn is_object(v: NanoValue) -> bool { tag_of(v) == 5 }
pub fn is_list(v: NanoValue) -> bool   { tag_of(v) == 6 }
pub fn is_f32(v: NanoValue) -> bool    { tag_of(v) == 7 }
```

---

## Section 3: VirtualRAM Migration

Current → Target:

```
raw: Vec<i32>                    → raw: Vec<NanoValue>
push_i32(i32)                    → push_i32(i32) [thin wrapper: push(encode_i32(i32))]
pop_i32() → i32                  → pop_i32() → i32 [thin wrapper: decode_i32(pop())]
push_f64(f64) [2 slots!]         → push_f64(f64) [1 slot!]
pop_f64() → f64 [2 slots!]       → pop_f64() → f64 [1 slot!]
push string as -(idx+1)          → push_string(u32) [thin wrapper: push(encode_string(idx))]
pop -(idx+1) → decode            → pop_string() → u32 [thin wrapper: decode_string(pop())]
```

**Key semantic change:** f64 and i64 go from 2 slots to 1 slot. This affects:
- Opcode handlers that hardcode `pop × 2` for doubles
- FN_PROLOG frame size calculations
- Stack depth tracking in codegen

**Migration approach:** Retain convenience methods (push_i32, pop_i32, etc.) as thin wrappers. Calling code changes minimized — only sites that manually handle string tags or 2-slot doubles need rewrite.

---

## Section 4: Opcode Migration

### A: Simple stack ops (majority, ~80 opcodes)

Handler logic unchanged — just calling push_i32/pop_i32 which now wrap NanoValue.

ADD, SUB, MUL, DIV, MOD, AND, OR, XOR, NOT, SHL, SHR, NEG, INC, DEC, EQ, NE, LT, GT, LE, GE, etc.

### B: Semantic changes needed

| Opcode | Current | After |
|--------|---------|-------|
| CONST_F64 | Push 2 slots | Push 1 slot (encode_f64) |
| CONST_I64 | Push 2 slots | Push 1 slot (encode_i64) |
| PROMOTE_F64 | Pop f32 (1 slot) → push f64 (2 slots) | Pop f32 → push f64 (1 slot) |
| LOAD_STR | Push -(idx+1) | push_string(idx) |
| ADD_D..DIV_D | Pop 4 slots, push 2 | Pop 2 slots, push 1 |
| EQ_D..GE_D | Pop 4 slots, push 1 | Pop 2 slots, push 1 |

### C: Frame/pointer ops

FN_PROLOG, RESERVE_STACK: frame size calculation changes from `n_locals * 4` to `n_locals * 8`.

### Bytecode format: UNCHANGED

Opcode numbers and parameter encoding stay the same. Only runtime interpreter dispatch changes.

---

## Section 5: Codegen Adaptation

Minimal impact — codegen emits bytecode, not stack values.

### Changed
- String reference: `emit_i32(-(idx as i32) - 1)` → `emit_i32(idx as i32)` (remove negation)
- Internal stack depth tracking: `stack_depth += 2` for f64 → `stack_depth += 1`

### Unchanged
- emit_op, emit_i32, emit_byte — bytecode format unchanged
- Control flow (JMP/JMP_IF offsets) — byte offsets, not slot counts
- Type inference (last_expr_type) — compile-time only, not stored on stack

---

## Section 6: FFI Shim Adaptation

~65 sites, all mechanical replacement.

### Pattern: String tag removal

```rust
// Before
let raw = task.ram.pop_i32();
let str_idx = if raw < 0 { (-(raw) - 1) as usize } else { raw as usize };
// After
let str_idx = task.ram.pop_string() as usize;
```

### Pattern: f64 2-slot → 1-slot

```rust
// Before (C FFI shim)
let lo = task.ram.pop_i32() as u32;
let hi = task.ram.pop_i32() as u32;
let val = f64::from_bits(((hi as u64) << 32) | (lo as u64));
// After
let val = task.ram.pop_f64();
```

### Affected files

| File | Sites | Type |
|------|-------|------|
| ffi.rs (Rust FFI) | ~10 | string tag |
| py_ffi.rs (Python FFI) | ~5 | string tag |
| c_ffi.rs (C FFI) | ~20 | string tag + double |
| stdlib.rs (builtins) | ~30 | string tag |

---

## Section 7: Incremental Migration Strategy

### Phase 1: Value type + VirtualRAM foundation

- Add `nano_value.rs` to auto-val
- Add `raw2: Vec<NanoValue>` to VirtualRAM (coexists with `raw: Vec<i32>`)
- Add `push_nv()`/`pop_nv()` methods
- All existing tests pass (old stack untouched)
- **Estimated: ~300 lines new code, low risk**

### Phase 2: Dual-track runtime verification

- Dispatch loop maintains both stacks simultaneously
- `push_i32(val)` does: `self.raw.push(val)` + `self.raw2.push(encode_i32(val))`
- Add `verify_sync()` to compare stacks after each opcode
- Migrate a few opcodes (ADD, SUB, CONST_I32, PRINT) for validation
- **Estimated: ~200 lines glue code, medium risk**

### Phase 3: Switch primary stack (feature-gated)

- Add `#[cfg(feature = "nanbox")]` feature flag
- Under flag: push/pop methods operate on `raw2` only
- Migrate opcode handlers in batches:
  - Batch 1: Arithmetic (ADD/SUB/MUL/DIV/MOD + float variants)
  - Batch 2: Comparison + logic (EQ/NE/LT/GT/AND/OR)
  - Batch 3: Control flow (CALL/RET/JMP/FN_PROLOG)
  - Batch 4: Strings (LOAD_STR + all string tag sites)
  - Batch 5: FFI shims
- Run full test suite after each batch
- **Estimated: ~2,000 lines, high risk (main work)**

### Phase 4: Cleanup

- Remove `raw: Vec<i32>`, rename `raw2` → `raw`
- Remove `-(idx+1)` string tag remnants
- Remove dual-track verification code
- Remove `nanbox` feature flag (becomes only implementation)
- **Estimated: ~600 lines deleted, medium risk**

### Rollback safety

- Phase 1-2: Old stack untouched, `git revert` safe
- Phase 3: Feature flag controls, default off
- Phase 4: Flag removed only after full validation

---

## Future Optimization (out of scope)

After NaN-boxing is stable, add typed instructions (I32_ADD, F64_MUL) that bypass tag checks for statically-typed regions. This is Phase 5 — performance optimization, not part of this plan.

---

## Reference

- NaN-boxing in JavaScriptCore: <https://wingolog.org/archives/2011/05/18/value-representation-in-javascript-implementations>
- QuickJS value representation (tagged union default, NaN-boxing optional): <https://www.infoq.cn/article/e8CdMSWKcDJSk3JhrGus>
- IEEE 754 NaN bit patterns: <https://arxiv.org/html/2411.16544v3>
