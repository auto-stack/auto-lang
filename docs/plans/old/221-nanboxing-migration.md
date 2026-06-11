# Plan 221: AutoVM NaN-boxing Migration

> **Status: ✅ COMPLETE — nanbox is now the default feature (362/365 tests pass, 3 baseline failures)**
>
> **Migration complete:** nanbox feature flag enabled by default in Cargo.toml.
> All nanbox-specific bugs fixed: f64 2-slot handling, f32→f64 FFI promotion, heap object ID comparison in EQ/NE.

**Goal:** Replace AutoVM's `Vec<i32>` stack with NaN-boxed `Vec<u64>` (NanoValue), providing unified 8-byte value representation for all types — int, float, string, bool, null, object, list.

**Architecture:** Mirror JSC/SpiderMonkey NaN-boxing — pack type tag into unused NaN bit patterns of IEEE 754 doubles. f64 operations zero overhead (direct bit reinterpret), all other types use tagged NaN-boxed encoding.

**Tech Stack:** auto-val NanoValue type, VirtualRAM refactor, feature-gated migration

---

## NaN-boxing Encoding Scheme

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

---

## Task 1: Create NanoValue type in auto-val

**Files:**
- Create: `crates/auto-val/src/nano_value.rs`
- Modify: `crates/auto-val/src/lib.rs:1`

**Step 1: Create nano_value.rs with full encode/decode API**

```rust
//! Plan 221: NaN-boxed value representation for AutoVM
//!
//! Packs type tag + payload into a single u64 using IEEE 754 NaN bit patterns.
//! Normal f64 values are stored directly (zero overhead).
//! All other types use the NaN-boxed encoding with a 4-bit tag.

/// A NaN-boxed value — 64 bits that can hold any Auto type.
pub type NanoValue = u64;

// Tag constants (placed at bits 51-48 within NaN-boxed values)
const TAG_F64:    u64 = 0x0000_0000_0000_0000;
const TAG_I32:    u64 = 0x0001_0000_0000_0000;
const TAG_STRING: u64 = 0x0002_0000_0000_0000;
const TAG_BOOL:   u64 = 0x0003_0000_0000_0000;
const TAG_NULL:   u64 = 0x0004_0000_0000_0000;
const TAG_OBJECT: u64 = 0x0005_0000_0000_0000;
const TAG_LIST:   u64 = 0x0006_0000_0000_0000;
const TAG_F32:    u64 = 0x0007_0000_0000_0000;

// NaN-box base: sign=1, exponent=0x7FF (all 1s), tag=0, payload=0
// This creates a quiet NaN that encodes type information.
const NANBOX_BASE: u64 = 0xFFF0_0000_0000_0000;

// Tag shift for extraction
const TAG_SHIFT: u64 = 48;
const TAG_MASK: u64 = 0xF;
const PAYLOAD_MASK: u64 = 0xFFFF_FFFF;

// ---- Detection ----

/// Returns true if the value is NaN-boxed (not a normal f64).
#[inline(always)]
pub fn is_nanboxed(v: NanoValue) -> bool {
    (v >> 52) == 0xFFF
}

// ---- Encode ----

/// Encode an f64 directly — zero overhead (just bit reinterpret).
#[inline(always)]
pub fn encode_f64(f: f64) -> NanoValue {
    f.to_bits()
}

/// Encode an i32 into a NaN-boxed value.
#[inline(always)]
pub fn encode_i32(i: i32) -> NanoValue {
    NANBOX_BASE | TAG_I32 | ((i as u32) as u64)
}

/// Encode a string pool index into a NaN-boxed value.
#[inline(always)]
pub fn encode_string(idx: u32) -> NanoValue {
    NANBOX_BASE | TAG_STRING | (idx as u64)
}

/// Encode a bool into a NaN-boxed value.
#[inline(always)]
pub fn encode_bool(b: bool) -> NanoValue {
    NANBOX_BASE | TAG_BOOL | (b as u64)
}

/// Encode null/void into a NaN-boxed value.
#[inline(always)]
pub fn encode_null() -> NanoValue {
    NANBOX_BASE | TAG_NULL
}

/// Encode an object id into a NaN-boxed value.
#[inline(always)]
pub fn encode_object(id: u32) -> NanoValue {
    NANBOX_BASE | TAG_OBJECT | (id as u64)
}

/// Encode a list id into a NaN-boxed value.
#[inline(always)]
pub fn encode_list(id: u32) -> NanoValue {
    NANBOX_BASE | TAG_LIST | (id as u64)
}

/// Encode an f32 into a NaN-boxed value.
#[inline(always)]
pub fn encode_f32(f: f32) -> NanoValue {
    NANBOX_BASE | TAG_F32 | (f.to_bits() as u64)
}

// ---- Decode ----

/// Decode an f64 — zero overhead (just bit reinterpret).
#[inline(always)]
pub fn decode_f64(v: NanoValue) -> f64 {
    f64::from_bits(v)
}

/// Decode an i32 from a NaN-boxed value.
#[inline(always)]
pub fn decode_i32(v: NanoValue) -> i32 {
    (v & PAYLOAD_MASK) as i32
}

/// Decode a string pool index from a NaN-boxed value.
#[inline(always)]
pub fn decode_string(v: NanoValue) -> u32 {
    (v & PAYLOAD_MASK) as u32
}

/// Decode a bool from a NaN-boxed value.
#[inline(always)]
pub fn decode_bool(v: NanoValue) -> bool {
    (v & PAYLOAD_MASK) != 0
}

/// Decode an object id from a NaN-boxed value.
#[inline(always)]
pub fn decode_object(v: NanoValue) -> u32 {
    (v & PAYLOAD_MASK) as u32
}

/// Decode a list id from a NaN-boxed value.
#[inline(always)]
pub fn decode_list(v: NanoValue) -> u32 {
    (v & PAYLOAD_MASK) as u32
}

/// Decode an f32 from a NaN-boxed value.
#[inline(always)]
pub fn decode_f32(v: NanoValue) -> f32 {
    f32::from_bits((v & PAYLOAD_MASK) as u32)
}

// ---- Type query ----

/// Get the type tag of a NanoValue. Returns 0 for f64 (unboxed).
#[inline(always)]
pub fn tag_of(v: NanoValue) -> u64 {
    if is_nanboxed(v) {
        (v >> TAG_SHIFT) & TAG_MASK
    } else {
        0 // f64
    }
}

/// True if the value is a normal f64 (not NaN-boxed).
#[inline(always)]
pub fn is_f64(v: NanoValue) -> bool {
    !is_nanboxed(v)
}

#[inline(always)]
pub fn is_i32(v: NanoValue) -> bool { tag_of(v) == 1 }

#[inline(always)]
pub fn is_string(v: NanoValue) -> bool { tag_of(v) == 2 }

#[inline(always)]
pub fn is_bool(v: NanoValue) -> bool { tag_of(v) == 3 }

#[inline(always)]
pub fn is_null(v: NanoValue) -> bool { tag_of(v) == 4 }

#[inline(always)]
pub fn is_object(v: NanoValue) -> bool { tag_of(v) == 5 }

#[inline(always)]
pub fn is_list(v: NanoValue) -> bool { tag_of(v) == 6 }

#[inline(always)]
pub fn is_f32(v: NanoValue) -> bool { tag_of(v) == 7 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_roundtrip() {
        let vals = [0.0, -0.0, 1.0, -1.0, 3.14, f64::MAX, f64::MIN, f64::EPSILON];
        for v in vals {
            assert_eq!(decode_f64(encode_f64(v)), v, "f64 roundtrip failed for {}", v);
        }
        assert!(!is_nanboxed(encode_f64(1.0)));
        assert!(is_f64(encode_f64(1.0)));
    }

    #[test]
    fn test_i32_roundtrip() {
        let vals = [0, 1, -1, i32::MAX, i32::MIN, 42, -100];
        for v in vals {
            assert_eq!(decode_i32(encode_i32(v)), v, "i32 roundtrip failed for {}", v);
        }
        assert!(is_nanboxed(encode_i32(42)));
        assert!(is_i32(encode_i32(42)));
        assert!(!is_i32(encode_string(0)));
    }

    #[test]
    fn test_string_roundtrip() {
        for idx in [0u32, 1, 100, u32::MAX] {
            assert_eq!(decode_string(encode_string(idx)), idx);
        }
        assert!(is_string(encode_string(0)));
        assert!(!is_nanboxed(encode_string(0)) == false); // always NaN-boxed
    }

    #[test]
    fn test_bool_roundtrip() {
        assert_eq!(decode_bool(encode_bool(true)), true);
        assert_eq!(decode_bool(encode_bool(false)), false);
        assert!(is_bool(encode_bool(true)));
    }

    #[test]
    fn test_null() {
        let n = encode_null();
        assert!(is_null(n));
        assert!(is_nanboxed(n));
        assert!(!is_i32(n));
    }

    #[test]
    fn test_object_list_roundtrip() {
        assert_eq!(decode_object(encode_object(42)), 42);
        assert_eq!(decode_list(encode_list(7)), 7);
        assert!(is_object(encode_object(0)));
        assert!(is_list(encode_list(0)));
    }

    #[test]
    fn test_f32_roundtrip() {
        let vals = [0.0f32, 1.0, -1.0, 3.14];
        for v in vals {
            assert_eq!(decode_f32(encode_f32(v)), v);
        }
        assert!(is_f32(encode_f32(1.0)));
    }

    #[test]
    fn test_no_collision_between_types() {
        let values = [
            encode_f64(1.0),
            encode_i32(1),
            encode_string(1),
            encode_bool(true),
            encode_null(),
            encode_object(1),
            encode_list(1),
            encode_f32(1.0),
        ];
        // All should be distinct bit patterns
        for i in 0..values.len() {
            for j in (i+1)..values.len() {
                assert_ne!(values[i], values[j], "Collision between types {} and {}", i, j);
            }
        }
        // Each should be uniquely identified
        assert!(is_f64(values[0]) && !is_nanboxed(values[0]));
        assert!(is_i32(values[1]));
        assert!(is_string(values[2]));
        assert!(is_bool(values[3]));
        assert!(is_null(values[4]));
        assert!(is_object(values[5]));
        assert!(is_list(values[6]));
        assert!(is_f32(values[7]));
    }
}
```

**Step 2: Register module in lib.rs**

Add at line 2 of `crates/auto-val/src/lib.rs`:

```rust
mod nano_value;
pub use nano_value::*;
```

**Step 3: Run tests**

Run: `cargo test -p auto-val nano_value`
Expected: All 8 tests PASS

**Step 4: Commit**

```bash
git add crates/auto-val/src/nano_value.rs crates/auto-val/src/lib.rs
git commit -m "feat(auto-val): add NanoValue NaN-boxed type with encode/decode API (Plan 221 Task 1)"
```

---

## Task 2: Add NanoValue stack to VirtualRAM (dual-track)

**Files:**
- Modify: `crates/auto-lang/src/vm/virt_memory.rs:220-340`

This task adds a parallel `raw_nv: Vec<NanoValue>` field alongside the existing `raw: Vec<i32>`. The old stack is untouched — dual-track for verification only.

**Step 1: Add NanoValue import and raw_nv field**

At the top of `virt_memory.rs`, add:
```rust
use auto_val::nano_value::{NanoValue, encode_i32, decode_i32, encode_f64, decode_f64,
    encode_f32, decode_f32, encode_string, decode_string};
```

In the `VirtualRAM` struct (line ~220), add after `raw`:
```rust
pub struct VirtualRAM {
    pub raw: Vec<i32>,
    /// Plan 221: NaN-boxed stack (dual-track with raw for migration verification)
    pub raw_nv: Vec<NanoValue>,
    pub sp: usize,
    pub bp: usize,
    pub ranges: Vec<(i32, i32, bool)>,
}
```

**Step 2: Initialize raw_nv in `new()`**

```rust
pub fn new(size: usize) -> Self {
    Self {
        raw: vec![0; size],
        raw_nv: vec![0u64; size],
        sp: 0,
        bp: 0,
        ranges: Vec::new(),
    }
}
```

**Step 3: Add NanoValue convenience methods**

Add these methods to `impl VirtualRAM` (after existing push/pop methods, around line ~355):

```rust
    // ---- Plan 221: NanoValue stack operations (dual-track) ----

    /// Push a NanoValue onto the NaN-boxed stack
    #[inline(always)]
    pub fn push_nv(&mut self, val: NanoValue) {
        if self.sp >= self.raw_nv.len() {
            panic!("Stack Overflow (nanbox)");
        }
        self.raw_nv[self.sp] = val;
    }

    /// Pop a NanoValue from the NaN-boxed stack
    #[inline(always)]
    pub fn pop_nv(&mut self) -> NanoValue {
        if self.sp == 0 {
            panic!("Stack Underflow (nanbox)");
        }
        self.sp -= 1; // Note: sp is shared with old stack
        self.raw_nv[self.sp]
    }

    /// Push string index as NaN-boxed value (replaces -(idx+1) convention)
    #[inline(always)]
    pub fn push_string(&mut self, idx: u32) {
        self.push_nv(encode_string(idx));
    }

    /// Pop string index from NaN-boxed value
    #[inline(always)]
    pub fn pop_string(&mut self) -> u32 {
        decode_string(self.pop_nv())
    }
```

**Step 4: Verify compilation**

Run: `cargo build -p auto-lang`
Expected: Compiles with no errors (raw_nv is added but not yet used by any opcode)

**Step 5: Run existing tests to confirm no regression**

Run: `cargo test -p auto-lang --lib 2>&1 | grep "test result"`
Expected: All existing tests PASS

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/virt_memory.rs
git commit -m "feat(vm): add NanoValue dual-track stack to VirtualRAM (Plan 221 Task 2)"
```

---

## Task 3: Migrate LOAD_STR and string tagging in engine.rs

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs` (LOAD_STR at ~line 954, string decode sites)
- Modify: `crates/auto-lang/src/vm/native.rs` (decode_str_idx at ~line 15)

This is the highest-impact single change — replacing the `-(idx+1)` string tagging convention with NaN-boxed strings.

**Step 1: Migrate LOAD_STR opcode handler**

In `engine.rs` at ~line 954, change:

```rust
// Before:
OpCode::LOAD_STR => {
    let str_idx = self.flash.read_u16(task.ip);
    task.ip += 2;
    task.ram.push_i32(-(str_idx as i32) - 1);
    task.last_result_type = ResultType::default();
}

// After:
OpCode::LOAD_STR => {
    let str_idx = self.flash.read_u16(task.ip);
    task.ip += 2;
    task.ram.push_string(str_idx as u32);
    task.ram.push_i32(-(str_idx as i32) - 1); // keep old stack in sync for now
    task.last_result_type = ResultType::default();
}
```

Wait — this needs the dual-track approach. For Phase 3, we'll do the full switch. For now in Task 3, we just validate the encode/decode works by adding the NaN-boxed push alongside the old push.

Actually, the cleaner approach: **switch LOAD_STR to NanoValue-only and update all string decode sites simultaneously**. This is a batch change.

**Revised Step 1: Update `decode_str_idx` in native.rs**

In `crates/auto-lang/src/vm/native.rs` at ~line 15, the helper function:

```rust
// Before:
pub fn decode_str_idx(bits: i32) -> usize {
    (-bits - 1) as usize
}

// After:
pub fn decode_str_idx(bits: i32) -> usize {
    if bits < 0 {
        (-bits - 1) as usize
    } else {
        bits as usize
    }
}
```

This is a defensive change — makes decode_str_idx handle both old and new encoding. We'll fully remove it later.

**Step 2: Run tests to see current baseline**

Run: `cargo test -p auto-lang --lib 2>&1 | grep "test result"`
Expected: All pass — no behavior change yet.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/native.rs
git commit -m "fix(vm): make decode_str_idx handle both positive and negative indices (Plan 221 Task 3)"
```

---

## Task 4: Add `nanbox` feature flag and switch VirtualRAM methods

**Files:**
- Modify: `crates/auto-lang/Cargo.toml` (add nanbox feature)
- Modify: `crates/auto-lang/src/vm/virt_memory.rs` (feature-gated push/pop)

This is the core switch — under `#[cfg(feature = "nanbox")]`, VirtualRAM methods operate on `raw_nv: Vec<NanoValue>` instead of `raw: Vec<i32>`.

**Step 1: Add nanbox feature to Cargo.toml**

In `crates/auto-lang/Cargo.toml` features section, add:

```toml
# Plan 221: NaN-boxed value representation (migrating from i32 stack)
nanbox = []
```

**Step 2: Feature-gate VirtualRAM push/pop methods**

Replace the existing `push_i32`/`pop_i32`/`push_f64`/`pop_f64` implementations with feature-gated versions:

```rust
#[cfg(not(feature = "nanbox"))]
#[inline(always)]
pub fn push_i32(&mut self, val: i32) {
    if self.sp >= self.raw.len() {
        panic!("Stack Overflow");
    }
    self.raw[self.sp] = val;
    self.sp += 1;
}

#[cfg(feature = "nanbox")]
#[inline(always)]
pub fn push_i32(&mut self, val: i32) {
    if self.sp >= self.raw_nv.len() {
        panic!("Stack Overflow");
    }
    self.raw_nv[self.sp] = encode_i32(val);
    self.sp += 1;
}

#[cfg(not(feature = "nanbox"))]
#[inline(always)]
pub fn pop_i32(&mut self) -> i32 {
    if self.sp == 0 {
        panic!("Stack Underflow");
    }
    self.sp -= 1;
    self.raw[self.sp]
}

#[cfg(feature = "nanbox")]
#[inline(always)]
pub fn pop_i32(&mut self) -> i32 {
    if self.sp == 0 {
        panic!("Stack Underflow");
    }
    self.sp -= 1;
    decode_i32(self.raw_nv[self.sp])
}
```

Do the same for `push_f64`/`pop_f64` — under `nanbox` feature, f64 becomes single-slot:

```rust
#[cfg(feature = "nanbox")]
#[inline(always)]
pub fn push_f64(&mut self, val: f64) {
    if self.sp >= self.raw_nv.len() {
        panic!("Stack Overflow");
    }
    self.raw_nv[self.sp] = encode_f64(val);
    self.sp += 1;
}

#[cfg(feature = "nanbox")]
#[inline(always)]
pub fn pop_f64(&mut self) -> f64 {
    if self.sp == 0 {
        panic!("Stack Underflow");
    }
    self.sp -= 1;
    decode_f64(self.raw_nv[self.sp])
}
```

Also add `push_string`/`pop_string` under `nanbox`:

```rust
#[cfg(feature = "nanbox")]
#[inline(always)]
pub fn push_string(&mut self, idx: u32) {
    if self.sp >= self.raw_nv.len() {
        panic!("Stack Overflow");
    }
    self.raw_nv[self.sp] = encode_string(idx);
    self.sp += 1;
}

#[cfg(feature = "nanbox")]
#[inline(always)]
pub fn pop_string(&mut self) -> u32 {
    if self.sp == 0 {
        panic!("Stack Underflow");
    }
    self.sp -= 1;
    decode_string(self.raw_nv[self.sp])
}
```

**Step 3: Feature-gate `top()`, `read_i32()`, `write_i32()`**

```rust
#[cfg(feature = "nanbox")]
pub fn top(&self) -> Option<i32> {
    if self.sp == 0 { None } else { Some(decode_i32(self.raw_nv[self.sp - 1])) }
}

#[cfg(feature = "nanbox")]
pub fn read_i32(&self, addr: usize) -> i32 {
    decode_i32(self.raw_nv[addr])
}

#[cfg(feature = "nanbox")]
pub fn write_i32(&mut self, addr: usize, val: i32) {
    self.raw_nv[addr] = encode_i32(val);
}
```

**Step 4: Verify both modes compile**

```bash
cargo build -p auto-lang                        # Without nanbox
cargo build -p auto-lang --features nanbox      # With nanbox
```

Expected: Both compile. Tests pass without nanbox.

**Step 5: Commit**

```bash
git add crates/auto-lang/Cargo.toml crates/auto-lang/src/vm/virt_memory.rs
git commit -m "feat(vm): add nanbox feature flag and dual-track VirtualRAM methods (Plan 221 Task 4)"
```

---

## Task 5: Migrate LOAD_STR and string encode sites (nanbox feature)

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs` (~25 string tagging sites)

Under `#[cfg(feature = "nanbox")]`, switch LOAD_STR and all `push_i32(-(idx as i32) - 1)` to `push_string(idx as u32)`.

**Step 1: Migrate LOAD_STR**

```rust
#[cfg(feature = "nanbox")]
OpCode::LOAD_STR => {
    let str_idx = self.flash.read_u16(task.ip);
    task.ip += 2;
    task.ram.push_string(str_idx as u32);
    task.last_result_type = ResultType::default();
}

#[cfg(not(feature = "nanbox"))]
OpCode::LOAD_STR => {
    let str_idx = self.flash.read_u16(task.ip);
    task.ip += 2;
    task.ram.push_i32(-(str_idx as i32) - 1);
    task.last_result_type = ResultType::default();
}
```

**Step 2: Migrate string decode sites**

All sites that decode `(-bits - 1) as usize` need to switch to `pop_string() as usize` under nanbox. Key sites in engine.rs:

- Line ~381: `ram.push_i32(-(idx as i32) - 1)` → `ram.push_string(idx as u32)`
- Line ~1089: `(-tagged - 1) as usize` → `task.ram.pop_string() as usize`
- Line ~1323: `task.ram.push_i32(-(result_idx as i32) - 1)` → `task.ram.push_string(result_idx as u32)`
- Line ~1562, 1569: TO_STR → push_string
- Line ~1610, 1620, 1630, 1640, 1671: TYPE_*_TO_STR → push_string
- Line ~1747, 1758: TYPE_CAST → push_string
- Line ~1782, 1792: String concat decode → pop_string
- Line ~1811: String result → push_string
- Line ~2073: String indexing decode → pop_string
- Line ~2464: String slice decode → pop_string
- Line ~2508: String element decode → pop_string
- Line ~2589: String field decode → pop_string
- Line ~2743: Field name decode → pop_string
- Line ~2870: Value conversion → push_string

Use `cfg` blocks at each site:

```rust
#[cfg(feature = "nanbox")]
{
    task.ram.push_string(result_idx as u32);
}
#[cfg(not(feature = "nanbox"))]
{
    task.ram.push_i32(-(result_idx as i32) - 1);
}
```

For decode sites:

```rust
#[cfg(feature = "nanbox")]
let idx = task.ram.pop_string() as usize;
#[cfg(not(feature = "nanbox"))]
let idx = (-tagged - 1) as usize;
```

**Step 3: Run tests with nanbox**

Run: `cargo test -p auto-lang --features nanbox --lib 2>&1 | grep "test result"`
Expected: Some tests may fail — fix string-related opcode handlers.

**Step 4: Fix failing tests iteratively**

Each failure will point to a string tagging site that wasn't migrated. Add the cfg block and re-run.

**Step 5: Run tests without nanbox to confirm no regression**

Run: `cargo test -p auto-lang --lib 2>&1 | grep "test result"`
Expected: All pass.

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs
git commit -m "feat(vm): migrate LOAD_STR and string tagging to NanoValue under nanbox feature (Plan 221 Task 5)"
```

---

## Task 6: Migrate f64 opcodes from 2-slot to 1-slot (nanbox feature)

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs` (f64 arithmetic/comparison opcodes)

Under nanbox, f64 values occupy 1 slot instead of 2. This affects ~20 opcodes.

**Step 1: Migrate f64 arithmetic (ADD_D, SUB_D, MUL_D, DIV_D, MOD_D, NEG_D)**

```rust
#[cfg(feature = "nanbox")]
{
    let b = task.ram.pop_f64();
    let a = task.ram.pop_f64();
    task.ram.push_f64(a + b); // or -, *, /, etc.
}
#[cfg(not(feature = "nanbox"))]
{
    // existing 2-slot code
    let b = task.ram.pop_f64();
    let a = task.ram.pop_f64();
    task.ram.push_f64(a + b);
}
```

Note: The existing code already uses `pop_f64()`/`push_f64()` which internally does 2-slot. Under nanbox, these same method calls will do 1-slot. So **many f64 opcode handlers don't need code changes** — only the VirtualRAM methods change (already done in Task 4).

Check sites that manually pop 2 × i32 for f64:

- CONST_F64 handler (~line 931): Already uses `task.ram.push_f64(val)` — no change needed
- ADD_D through DIV_D (~line 2962): Already uses `pop_f64()`/`push_f64()` — no change needed
- PROMOTE_F64 (~line 1495): `pop_f32()` → `push_f64()` — no change needed

**This task may be mostly a no-op** if the existing handlers already use `pop_f64()`/`push_f64()`. Verify by running tests.

**Step 2: Check for manual 2-slot f64 manipulation**

Search for any site that does `pop_i32() × 2` to reconstruct f64 manually (not via pop_f64). These need migration.

Run: `grep -n "pop_i32.*pop_i32\|push_i32.*push_i32" engine.rs`

**Step 3: Run tests with nanbox**

Run: `cargo test -p auto-lang --features nanbox --lib 2>&1 | grep "test result"`
Expected: f64 arithmetic tests pass.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs
git commit -m "feat(vm): verify f64 opcodes work with 1-slot NanoValue under nanbox (Plan 221 Task 6)"
```

---

## Task 7: Migrate native.rs string operations

**Files:**
- Modify: `crates/auto-lang/src/vm/native.rs` (~30 decode_str_idx call sites)

**Step 1: Add NanoValue imports**

```rust
#[cfg(feature = "nanbox")]
use auto_val::nano_value::{decode_string, encode_string};
```

**Step 2: Feature-gate decode_str_idx**

```rust
#[cfg(not(feature = "nanbox"))]
pub fn decode_str_idx(bits: i32) -> usize {
    (-bits - 1) as usize
}

#[cfg(feature = "nanbox")]
pub fn decode_str_idx_from_nv(nv: NanoValue) -> usize {
    decode_string(nv) as usize
}
```

**Step 3: Update call sites**

At each site calling `decode_str_idx(task.ram.pop_i32())`:
- Under nanbox: use `task.ram.pop_string() as usize` directly
- Under old: keep existing code

This is the largest mechanical change (~30 sites). Use sed-like replacement:

```rust
// Pattern at each site:
#[cfg(feature = "nanbox")]
let idx = task.ram.pop_string() as usize;
#[cfg(not(feature = "nanbox"))]
let idx = decode_str_idx(task.ram.pop_i32());
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang --features nanbox --lib 2>&1 | grep "test result"`
Expected: String operation tests pass under nanbox.

**Step 5: Commit**

```bash
git add crates/auto-lang/src/vm/native.rs
git commit -m "feat(vm): migrate native.rs string operations to NanoValue under nanbox (Plan 221 Task 7)"
```

---

## Task 8: Migrate FFI shims

**Files:**
- Modify: `crates/auto-lang/src/ffi.rs` (~10 sites)
- Modify: `crates/auto-lang/src/py_ffi.rs` (~5 sites)
- Modify: `crates/auto-lang/src/vm/ffi/c_ffi.rs` (~20 sites)
- Modify: `crates/auto-lang/src/vm/ffi/stdlib.rs` (~30 sites)

**Step 1: Migrate ffi.rs string tagging**

Replace all `-(idx as i32) - 1` patterns with cfg-gated `push_string`/`pop_string`.

**Step 2: Migrate py_ffi.rs string tagging**

Same pattern — replace negative tagging with `pop_string()`/`push_string()`.

**Step 3: Migrate c_ffi.rs**

Most complex — handles multiple types. Under nanbox, `pop_f64()` is 1-slot instead of 2-slot. Check for manual 2-slot f64 reconstruction.

**Step 4: Migrate stdlib.rs builtins**

~30 sites with string tag decode. Mechanical replacement:

```rust
#[cfg(feature = "nanbox")]
let idx = task.ram.pop_string() as usize;
#[cfg(not(feature = "nanbox"))]
let raw = task.ram.pop_i32();
let idx = if raw < 0 { (-(raw) - 1) as usize } else { raw as usize };
```

**Step 5: Run tests with and without nanbox**

```bash
cargo test -p auto-lang --lib                        # Without nanbox
cargo test -p auto-lang --features nanbox --lib      # With nanbox
```

Expected: Both pass.

**Step 6: Commit**

```bash
git add crates/auto-lang/src/ffi.rs crates/auto-lang/src/py_ffi.rs crates/auto-lang/src/vm/ffi/
git commit -m "feat(vm): migrate FFI shims to NanoValue string encoding under nanbox (Plan 221 Task 8)"
```

---

## Task 9: Migrate codegen string references

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (LOAD_STR emit)

Under nanbox, codegen should emit positive string indices (runtime handles encoding).

**Step 1: Feature-gate string index emission**

Find the LOAD_STR emit site in codegen. Currently emits `-(idx as i32) - 1`. Under nanbox, emit positive idx:

```rust
#[cfg(feature = "nanbox")]
self.emit_i32(idx as i32);  // positive index
#[cfg(not(feature = "nanbox"))]
self.emit_i32(-(idx as i32) - 1);  // negative tagging
```

**Step 2: Check for stack_depth += 2 for f64**

Search codegen for any `stack_depth += 2` or equivalent related to f64/i64 values. Under nanbox these should be `+= 1`.

**Step 3: Run full test suite**

```bash
cargo test -p auto-lang --features nanbox
```

Expected: All tests pass.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(codegen): migrate string emission for NanoValue under nanbox (Plan 221 Task 9)"
```

---

## Task 10: Full test suite validation and cleanup

**Files:**
- Modify: `crates/auto-lang/src/vm/virt_memory.rs` (remove dead code)
- Modify: `docs/plans/221-nanboxing-migration.md` (update status)

**Step 1: Run comprehensive tests**

```bash
cargo test -p auto-lang --features nanbox --lib
cargo test -p auto-lang --features nanbox,python --lib py_ffi   # if Python available
cargo test -p auto-lang -- trans                                  # transpiler tests
```

Expected: All pass.

**Step 2: Run without nanbox to confirm no regression**

```bash
cargo test -p auto-lang --lib
```

Expected: All pass.

**Step 3: Update plan status**

Change status in `docs/plans/221-nanboxing-migration.md` to `✅ COMPLETE (nanbox feature)`

**Step 4: Commit**

```bash
git add docs/plans/221-nanboxing-migration.md
git commit -m "docs: update Plan 221 status — NaN-boxing migration complete (nanbox feature)"
```

---

## Future Work (out of scope for this plan)

- **~~Phase 4 cleanup~~**: ✅ Done in Plan 298 — removed `raw: Vec<i32>`, removed `#[cfg(not(feature = "nanbox"))]` blocks, NaN-boxing is now the only implementation
- **Phase 5 typed instructions**: Add I32_ADD, F64_MUL etc. that bypass tag checks for statically-typed regions
- **i64/u64 encoding**: Add TAG_I64/TAG_U64 with full 64-bit payload (requires different encoding — currently reserved)

---

## Reference

- NaN-boxing in JavaScriptCore: <https://wingolog.org/archives/2011/05/18/value-representation-in-javascript-implementations>
- QuickJS value representation (tagged union default, NaN-boxing optional): <https://www.infoq.cn/article/e8CdMSWKcDJSk3JhrGus>
- IEEE 754 NaN bit patterns: <https://arxiv.org/html/2411.16544v3>
