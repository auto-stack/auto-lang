# Plan: ABT (Auto Byte Text) Implementation

## Background

The AutoVM has a binary bytecode format (ABC) and a debugger-only disassembler (`disasm.rs`). The Playground has a "Bytecode" tab that only shows content when a Debug WebSocket session is active. There is no standalone Auto → ABT transpiler, no ABT → ABC assembler, and no formal ABT text format.

## Goals

1. Define a formal ABT text format that is human-readable and round-trippable.
2. Implement ABC → ABT (enhanced disassembler).
3. Implement ABT → ABC (assembler) so ABT code can be directly compiled and run.
4. Add Auto → ABT as a transpile target in the Playground.
5. Evaluate whether a direct Auto → ABT codegen path is worthwhile.

---

## Current State Analysis

### ABC Format (Binary Bytecode)
- Flat `Vec<u8>` in `VirtualFlash.memory`, little-endian.
- ~110 opcodes, variable operand sizes (0/1/2/4/8 bytes).
- Metadata pools stored separately: `strings`, `object_keys`, `object_types`, `exports_by_name`, `addr_to_name`, `symbol_map`.
- `SOURCE_LINE` (0xFE) opcode embedded inline for debug line mapping.
- Jump offsets are relative (i16/i32), `CALL` targets are absolute u32 addresses.
- `FN_PROLOG` inserted retroactively; function bodies preceded by a `JMP` skip.

### Existing Disassembler (`disasm.rs`)
- Outputs `Vec<DisasmLine>` with `{offset, mnemonic, operands, line}`.
- **Known bugs**: `CONST_I32`/`CONST_F32` decode 2 bytes instead of 4; `CONST_I64`/`CONST_U64`/`CONST_F64` decode 4 bytes instead of 8.
- No metadata output (no string pool, exports, etc.).
- No labels — jumps shown as raw hex offsets.

---

## Proposed ABT Text Format

ABT is an assembly-style text format with sections. Design principles:
- **Readable**: mnemonics match opcodes, operands use decimal by default.
- **Round-trippable**: assembler can reconstruct identical ABC + metadata.
- **Label-based**: symbolic labels for jumps/calls instead of raw offsets.

### Syntax Overview

```abt
; ─── Metadata Sections ───
.strings
  "Hello, World!"
  "add"

.exports
  main
  add

.object_keys
  0: ["x", "y"]

.object_types
  0: [Int, Int]

; ─── Code Section ───
.code

main:
  .line 1
  fn.prolog 0, 1      ; args=0, locals=1
  reserve 1
  const.i32 42
  store_local 0
  load_str 0          ; "Hello, World!"
  call_nat 0          ; print
  ret

add:
  .line 5
  fn.prolog 2, 1      ; args=2, locals=1
  reserve 1
  load_local 0
  load_local 1
  add
  store_local 2
  load_local 2
  ret
```

### Label Resolution Rules
- Labels are alphanumeric + `_`, ending with `:`.
- Jump targets: `jmp @label`, `jmp_if_z @label`, `jmp_l @label`.
- Call targets: `call @label`, `closure @label`.
- The assembler does a two-pass layout: collect label offsets → resolve → emit.

### Instruction Encoding in ABT
- Most operands are decimal integers.
- Hex literals allowed with `0x` prefix (e.g., `call_nat 0x10`).
- String references by pool index: `load_str 5`.
- Field references by string pool index: `get_field 3`.
- `.line N` pseudo-op corresponds to `SOURCE_LINE` opcode.

---

## Implementation Plan

### Phase 1: Fix Disassembler & Formalize ABT Format

**Files to modify/create:**
- `crates/auto-lang/src/vm/disasm.rs` — fix operand size bugs
- `crates/auto-lang/src/vm/abt/` — new module (or `crates/auto-lang/src/vm/abt.rs`)

**Tasks:**
1. **Fix disassembler operand decoding**:
   - `CONST_I32`: read 4 bytes (i32 LE), not 2.
   - `CONST_F32`: read 4 bytes (f32 LE), not 2.
   - `CONST_I64`: read 8 bytes (i64 LE), not 4.
   - `CONST_U64`: read 8 bytes (u64 LE), not 4.
   - `CONST_F64`: read 8 bytes (f64 LE), not 4.
   - Verify all other opcodes match engine decoder in `engine.rs`.

2. **Design `AbtProgram` struct** — in-memory representation of ABT:
   ```rust
   pub struct AbtProgram {
       pub strings: Vec<String>,
       pub exports: Vec<(String, usize)>,  // name → label offset
       pub object_keys: Vec<Vec<String>>,
       pub object_types: Vec<Vec<ObjectType>>,
       pub code: Vec<AbtInstruction>,
       pub labels: HashMap<String, usize>, // label → byte offset
   }

   pub struct AbtInstruction {
       pub offset: usize,
       pub opcode: OpCode,
       pub operands: Vec<AbtOperand>,
       pub source_line: Option<u32>,
   }

   pub enum AbtOperand {
       ImmI32(i32),
       ImmI64(i64),
       ImmU64(u64),
       ImmF32(f32),
       ImmF64(f64),
       ImmU8(u8),
       ImmU16(u16),
       Label(String),       // for jumps/calls
       StringIdx(usize),    // for load_str
       FieldIdx(usize),     // for get/set_field
       NatIdx(u16),         // for call_nat
   }
   ```

3. **Implement `abc_to_abt`** — enhanced disassembler that produces `AbtProgram`:
   - Disassemble `VirtualFlash.memory` into `AbtInstruction` sequence.
   - Resolve `CALL`/`CLOSURE` absolute addresses to labels using `addr_to_name` / `exports_by_name`.
   - Generate synthetic labels for unnamed targets (e.g., `L_0x0042`).
   - Extract metadata from `VirtualFlash` and `AutoVM` into ABT sections.
   - Emit `AbtProgram` which can be formatted to text.

4. **Implement ABT text formatter** (`AbtProgram::to_string()`):
   - Output sections in order: `.strings`, `.exports`, `.object_keys`, `.object_types`, `.code`.
   - Print labels on their own lines.
   - Print instructions as: `{mnemonic} {operands}`.
   - Keep `.line` pseudo-ops for `SOURCE_LINE`.

### Phase 2: Assembler (ABT → ABC)

**Files to create:**
- `crates/auto-lang/src/vm/abt/asm.rs` — assembler

**Tasks:**
1. **ABT text parser**:
   - Parse sections (`# comment`, `.section`, labels, instructions).
   - Handle operand parsing (decimal, hex, floats, string/field indices).
   - Validate opcodes and operand counts/types against `OpCode` enum.

2. **Two-pass assembly**:
   - **Pass 1**: Tokenize, collect label positions (byte offsets), layout code size.
   - **Pass 2**: Emit opcodes + operands, resolve labels to offsets/addresses.
   - Jump instructions: compute relative offsets from label positions.
   - `CALL`/`CLOSURE`: emit absolute addresses from label positions.

3. **Metadata assembly**:
   - Build string pool from `.strings` section.
   - Build exports map from `.exports` section.
   - Build object_keys / object_types from their sections.
   - Handle `CALL_NAT` indices (map to native function names or accept numeric indices).

4. **Produce runnable output**:
   - Return `CompiledPackage` (bytecode + pools + exports) or directly `VirtualFlash`.
   - Integrate with existing `Linker` if multi-module support is needed.

5. **Add assembler entry point to `auto-lang` crate**:
   ```rust
   pub fn assemble_abt(source: &str) -> Result<CompiledPackage, CompileError>
   ```

### Phase 3: Auto → ABT Transpile Target

**Files to modify:**
- `crates/auto-playground/src/routes/trans.rs` — add `"abt"` / `"bytecode"` target
- `crates/auto-playground/frontend/src/composables/usePlayground.ts` — add `'abt'` to `OutputTab`
- `crates/auto-playground/frontend/src/components/OutputPanel.vue` — add ABT tab

**Tasks:**
1. **Backend transpiler**:
   ```rust
   fn transpile_abt(source: &str) -> Result<(String, Vec<SourceMapEntry>), AppError> {
       // 1. Compile Auto source to ABC (existing codegen)
       let (vm, _, _) = auto_lang::create_vm_from_source(source)?;
       // 2. Disassemble ABC to AbtProgram
       let abt = auto_lang::vm::abt::disassemble_to_abt(&vm.flash, &vm)?;
       // 3. Format to text
       Ok((abt.to_string(), vec![]))
   }
   ```
   Or use `CompiledPackage` directly if we don't need a full VM.

2. **Frontend updates**:
   - Add `'abt'` or `'bytecode'` to `OutputTab` type and `tabs` array in `OutputPanel.vue`.
   - Add to `transpileAll()` targets list.
   - The CodePreview component can render ABT text without syntax highlighting (or use a simple custom highlighter).

### Phase 4: Direct Auto → ABT Evaluation

**Question**: Should we implement direct Auto → ABT codegen instead of Auto → ABC → ABT?

**Analysis:**

| Aspect | Auto → ABC → ABT | Direct Auto → ABT |
|--------|------------------|-------------------|
| **Implementation effort** | Low (reuse codegen + new disassembler) | High (new text emitter in codegen, or parallel codegen) |
| **Code complexity** | Clean separation: codegen is binary-only, ABT is a view layer | codegen becomes dual-output, harder to maintain |
| **Performance** | One extra pass through disassembler — negligible for playground | Slightly faster (no binary intermediate) |
| **Use case fit** | Debugging / inspection — perfect | Hand-written ABT optimization — irrelevant |
| **Round-trip correctness** | ABC is canonical; ABT is derived | ABT might diverge from canonical ABC |
| **Maintenance** | Single codegen path to maintain | Every codegen change must update both binary and text paths |

**Conclusion**: Direct Auto → ABT is **not worth implementing** at this stage. The indirect path (Auto → ABC → ABT) is:
- Simpler and less error-prone.
- Performance difference is negligible for the intended use case (Playground transpilation).
- Keeps codegen as a single source of truth.

**When direct Auto → ABT might become valuable:**
- If ABT becomes a first-class hand-written target (like Rust/C/Python), and we want the compiler to emit ABT directly for readability/optimization purposes.
- If the disassembler pass becomes a measurable bottleneck (unlikely).

---

## File Inventory

### New Files
- `crates/auto-lang/src/vm/abt/mod.rs` — ABT data structures + formatter
- `crates/auto-lang/src/vm/abt/disasm.rs` — ABC → ABT (enhanced disassembler)
- `crates/auto-lang/src/vm/abt/asm.rs` — ABT → ABC (assembler)
- `crates/auto-lang/src/vm/abt/parser.rs` — ABT text parser

### Modified Files
- `crates/auto-lang/src/vm/disasm.rs` — fix operand size bugs
- `crates/auto-lang/src/vm/mod.rs` — add `pub mod abt;`
- `crates/auto-playground/src/routes/trans.rs` — add `"abt"` target
- `crates/auto-playground/frontend/src/types.ts` — add `'abt'` to `OutputTab`
- `crates/auto-playground/frontend/src/composables/usePlayground.ts` — add `'abt'` to targets
- `crates/auto-playground/frontend/src/components/OutputPanel.vue` — add ABT tab button

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Disassembler operand bugs are symptoms of codegen/engine mismatch | Audit ALL opcodes against engine decoder; write unit tests for each opcode |
| Label resolution in assembler is complex (jumps, calls, closures) | Write comprehensive assembler unit tests with forward/backward references |
| ABT text format may need to evolve | Version the format (e.g., `.version 1.0` pseudo-op) |
| String/object metadata may not round-trip perfectly | Ensure assembler populates same `VirtualFlash` fields as codegen |
| Performance of disassembler on large programs | Disassembler is O(n) in bytecode size; acceptable for Playground |

---

## Recommended Approach

**Single Option (Recommended):**

1. **Phase 1** (can start immediately): Fix `disasm.rs` operand bugs, design `AbtProgram`, implement ABC → ABT.
2. **Phase 2**: Implement ABT parser and assembler (ABT → ABC).
3. **Phase 3**: Add `transpile_abt` to playground backend + frontend tab.
4. **Defer Phase 4**: Direct Auto → ABT is not justified by current requirements.

The playground Bytecode tab will then work without requiring a Debug session: selecting "ABT" will transpile Auto → ABC → ABT and display the text.
