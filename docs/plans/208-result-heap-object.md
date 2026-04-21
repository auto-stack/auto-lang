# Plan 208: Result Heap Object — `!T` with Rich Error Values

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Change Result from sentinel integers to heap objects so `Err` can carry any value (enum variants, strings, objects), enabling `is result { Ok(x) -> ... Err(e) -> is e { ParseError.InvalidChar(ch, pos) -> ... } }`.

**Architecture:** Reuse `GenericInstanceData` + `IS_VARIANT` + `GET_GENERIC_FIELD` infrastructure. `Ok(val)` becomes a heap object with `mono_name = "Result.Ok"`, `Err(val)` becomes `"Result.Err"`. The `is` pattern match already handles `IS_VARIANT` for enum variants — we add the same for Result variants. No new opcodes.

**Tech Stack:** Rust, AutoVM engine, codegen

**Depends on:** Plan 201 Phase 1 (✅), Plan 207 (✅)

---

## Task 1: Engine — Refactor CREATE_OK to create heap object

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1741-1747`

**Current code** (line 1741):
```rust
OpCode::CREATE_OK => {
    // Value is already on stack, just tag it as Ok
    // TODO: Implement proper Result<T> type tracking in VM
}
```

**New code:**
```rust
OpCode::CREATE_OK => {
    // Value is on stack. Wrap it in a Result.Ok heap object.
    use crate::vm::generic_registry::GenericInstanceData;
    let val = task.ram.pop_i32();
    let instance = GenericInstanceData::new("Result.Ok".to_string(), vec![auto_val::Value::Int(val)]);
    let instance_id = self.insert_heap_object(instance);
    task.ram.push_i32(instance_id as i32);
}
```

**Verify**: `cargo build -p auto-lang`

---

## Task 2: Engine — Refactor CREATE_ERR to create heap object

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1749-1758`

**Current code** (line 1749):
```rust
OpCode::CREATE_ERR => {
    let _msg_bits = task.ram.pop_i32();
    task.ram.push_i32(-2);
}
```

**New code:**
```rust
OpCode::CREATE_ERR => {
    // Error value is on stack (string index, heap ID, or i32).
    // Wrap it in a Result.Err heap object.
    use crate::vm::generic_registry::GenericInstanceData;
    let err_val = task.ram.pop_i32();
    let instance = GenericInstanceData::new("Result.Err".to_string(), vec![auto_val::Value::Int(err_val)]);
    let instance_id = self.insert_heap_object(instance);
    task.ram.push_i32(instance_id as i32);
}
```

**Verify**: `cargo build -p auto-lang`

---

## Task 3: Engine — Refactor IS_OK to check heap object mono_name

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1767-1772`

**Current code** (line 1767):
```rust
OpCode::IS_OK => {
    let value = task.ram.pop_i32();
    let is_ok = if value >= 0 { 1 } else { 0 };
    task.ram.push_i32(is_ok);
}
```

**New code:**
```rust
OpCode::IS_OK => {
    use crate::vm::generic_registry::GenericInstanceData;
    let value = task.ram.pop_i32();
    // Check if it's a heap object with mono_name "Result.Ok"
    let is_ok = if value > 0 {
        if let Some(obj) = self.get_heap_object(value as u64) {
            let guard = obj.read().unwrap();
            if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
                inst.mono_name == "Result.Ok"
            } else {
                // Legacy: plain positive value = Ok
                true
            }
        } else {
            // Legacy: plain positive value = Ok
            true
        }
    } else {
        false
    };
    task.ram.push_i32(if is_ok { -2147483648 } else { -2147483647 });
}
```

**Note**: Uses the VM boolean convention (`i32::MIN` = true, `i32::MIN+1` = false) to match `IS_VARIANT`.

**Verify**: `cargo build -p auto-lang`

---

## Task 4: Engine — Refactor UNWRAP_OK and UNWRAP_ERR for heap objects

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1784-1800`

**Current UNWRAP_OK** (line 1784):
```rust
OpCode::UNWRAP_OK => {
    let value = task.ram.pop_i32();
    if value < 0 {
        return Err(VMError::RuntimeError("called unwrap on Err".to_string()));
    }
    task.ram.push_i32(value);
}
```

**New UNWRAP_OK:**
```rust
OpCode::UNWRAP_OK => {
    use crate::vm::generic_registry::GenericInstanceData;
    let value = task.ram.pop_i32();
    if value <= 0 {
        return Err(VMError::RuntimeError("called unwrap on Err".to_string()));
    }
    if let Some(obj) = self.get_heap_object(value as u64) {
        let guard = obj.read().unwrap();
        if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
            if inst.mono_name == "Result.Ok" {
                if let Some(field) = inst.fields.first() {
                    task.ram.push_i32(field.as_i32().unwrap_or(0));
                    return Ok(StepResult::Continue);
                }
            }
        }
    }
    // Legacy fallback: plain positive value
    task.ram.push_i32(value);
}
```

**Current UNWRAP_ERR** (line 1794):
```rust
OpCode::UNWRAP_ERR => {
    let value = task.ram.pop_i32();
    if value >= 0 {
        return Err(VMError::RuntimeError("called unwrap_err on Ok".to_string()));
    }
    // Push the error message index back
    task.ram.push_i32(value);
}
```

**New UNWRAP_ERR:**
```rust
OpCode::UNWRAP_ERR => {
    use crate::vm::generic_registry::GenericInstanceData;
    let value = task.ram.pop_i32();
    if value <= 0 {
        return Err(VMError::RuntimeError("called unwrap_err on non-heap value".to_string()));
    }
    if let Some(obj) = self.get_heap_object(value as u64) {
        let guard = obj.read().unwrap();
        if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
            if inst.mono_name == "Result.Err" {
                if let Some(field) = inst.fields.first() {
                    Self::push_value(&mut task.ram, &field, &self.strings);
                    return Ok(StepResult::Continue);
                }
            }
        }
    }
    task.ram.push_i32(value);
}
```

**Verify**: `cargo build -p auto-lang`

---

## Task 5: Engine — Refactor ERROR_PROPAGATE for heap Result objects

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1340-1370`

The current `ERROR_PROPAGATE` handles Option.None (heap object or sentinel -1). It needs to also handle `Result.Err` heap objects.

**Logic**: After the existing Option.None check, add a check for `Result.Err`:

```rust
// Check if it's a Result.Err heap object — propagate error
if value > 0 {
    if let Some(obj) = self.get_heap_object(value as u64) {
        let guard = obj.read().unwrap();
        if let Some(inst) = guard.as_any().downcast_ref::<GenericInstanceData>() {
            if inst.mono_name == "Result.Err" {
                // Error case: propagate the Result.Err object to caller
                task.ram.push_i32(value);
                // Early return
                let old_bp = task.ram.read_i32(task.bp) as usize;
                let ret_ip = task.ram.read_i32(task.bp - 1) as usize;
                let n_args = self.flash.read_u8(task.ip) as usize;
                task.ip += 1; // consume n_args byte
                // ... standard RET logic ...
            } else if inst.mono_name == "Result.Ok" {
                // Ok case: unwrap the inner value
                if let Some(field) = inst.fields.first() {
                    Self::push_value(&mut task.ram, &field, &self.strings);
                } else {
                    task.ram.push_i32(value);
                }
                return Ok(StepResult::Continue);
            }
        }
    }
}
```

**IMPORTANT**: Read the existing ERROR_PROPAGATE code carefully. It already has complex logic for Option.None/Some. Add Result.Err handling in the right place without breaking Option propagation.

**Verify**: `cargo build -p auto-lang` and `cargo test -p auto-lang -- vm_file_tests`

---

## Task 6: Codegen — Compile Result is-match via IS_VARIANT

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs:2413-2489`

**Current**: The `ResultPattern` (Ok/Err) branch uses `IS_OK` + `UNWRAP_OK`/`UNWRAP_ERR`. 

**New**: Use `IS_VARIANT("Result.Ok")` / `IS_VARIANT("Result.Err")` + `GET_GENERIC_FIELD(0)` instead. This makes Result matching work exactly like enum variant matching, enabling the `Err(e) -> is e { ParseError.InvalidChar(ch, pos) -> ... }` nested match.

For **Ok(x)** branch:
```rust
crate::ast::ResultVariant::Ok => {
    self.emit(OpCode::DUP);
    // Use IS_VARIANT instead of IS_OK
    self.emit(OpCode::IS_VARIANT);
    let name = "Result.Ok";
    self.emit_u16(name.len() as u16);
    for &byte in name.as_bytes() { self.code.push(byte); }

    self.emit(OpCode::JMP_IF_Z);
    let jump_to_next = self.emit_placeholder_i16();

    if let Some(binding) = &res_cover.binding {
        // GET_GENERIC_FIELD(0) extracts inner value
        self.emit(OpCode::GET_GENERIC_FIELD);
        self.emit_u32(0);
        let var_idx = self.add_var(binding.as_str());
        self.emit_store_loc(var_idx);
    } else {
        self.emit(OpCode::POP);
    }

    self.compile_stmt(&Stmt::Block(body.clone()))?;
    self.emit(OpCode::JMP);
    let jump_to_end = self.emit_placeholder_i16();
    end_jumps.push(jump_to_end);
    self.patch_jump(jump_to_next);
    continue;
}
```

For **Err(e)** branch:
```rust
crate::ast::ResultVariant::Err => {
    self.emit(OpCode::DUP);
    // Use IS_VARIANT instead of IS_OK + XOR
    self.emit(OpCode::IS_VARIANT);
    let name = "Result.Err";
    self.emit_u16(name.len() as u16);
    for &byte in name.as_bytes() { self.code.push(byte); }

    self.emit(OpCode::JMP_IF_Z);
    let jump_to_next = self.emit_placeholder_i16();

    if let Some(binding) = &res_cover.binding {
        // GET_GENERIC_FIELD(0) extracts error value (heap ID of error object)
        self.emit(OpCode::GET_GENERIC_FIELD);
        self.emit_u32(0);
        let var_idx = self.add_var(binding.as_str());
        self.emit_store_loc(var_idx);
    } else {
        self.emit(OpCode::POP);
    }

    self.compile_stmt(&Stmt::Block(body.clone()))?;
    self.emit(OpCode::JMP);
    let jump_to_end = self.emit_placeholder_i16();
    end_jumps.push(jump_to_end);
    self.patch_jump(jump_to_next);
    continue;
}
```

**Key insight**: After `GET_GENERIC_FIELD(0)`, the binding `e` holds a heap ID pointing to the error value (e.g., a `ParseError.InvalidChar` instance). Then `is e { ParseError.InvalidChar(ch, pos) -> ... }` uses `IS_VARIANT("ParseError.InvalidChar")` which already works.

**Verify**: `cargo test -p auto-lang -- vm_file_tests`

---

## Task 7: Commit engine + codegen changes

Commit all Tasks 1-6 together:
```
feat(vm): change Result from sentinel integers to heap objects (Plan 208)
```

---

## Task 8: Test — basic Ok/Err with heap objects

**Files:**
- Create: `crates/auto-lang/test/vm/16_option_result/003_result_heap/result_heap.at`
- Create: `crates/auto-lang/test/vm/16_option_result/003_result_heap/result_heap.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

`result_heap.at`:
```auto
// Test: Result as heap object — Ok/Err matching
fn try_divide(a int, b int) !int {
    if b == 0 {
        return Err("division by zero")
    }
    Ok(a / b)
}

let r1 = try_divide(10, 3)
is r1 {
    Ok(n) -> print(n)
    Err(e) -> print("error")
}

let r2 = try_divide(10, 0)
is r2 {
    Ok(n) -> print(n)
    Err(e) -> print("error")
}
```

`result_heap.expected.out`:
```
3
error
```

**Register**: `#[test] fn test_16_option_result_003_result_heap() { test_vm("16_option_result/003_result_heap").unwrap(); }`

**Run**: `cargo test -p auto-lang -- test_16_option_result_003_result_heap`

---

## Task 9: Test — error propagation with `?` operator on heap Results

**Files:**
- Create: `crates/auto-lang/test/vm/16_option_result/004_result_propagate/result_propagate.at`
- Create: `crates/auto-lang/test/vm/16_option_result/004_result_propagate/result_propagate.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

`result_propagate.at`:
```auto
// Test: error propagation with .? on heap Result
fn try_parse(s str) !int {
    if s == "" {
        return Err("empty string")
    }
    Ok(42)
}

fn chain() !int {
    let a = try_parse("hello").?
    let b = try_parse("").?
    Ok(a + b)
}

let result = chain()
is result {
    Ok(n) -> print(n)
    Err(e) -> print("propagated error")
}
```

`result_propagate.expected.out`:
```
propagated error
```

---

## Task 10: Test — Err carries enum variant (nested is-match)

**Files:**
- Create: `crates/auto-lang/test/vm/16_option_result/005_result_enum_error/result_enum_error.at`
- Create: `crates/auto-lang/test/vm/16_option_result/005_result_enum_error/result_enum_error.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

`result_enum_error.at`:
```auto
// Test: Err carries enum variant — nested is-match
enum ParseError {
    InvalidChar { ch str, pos int }
    UnexpectedEnd
}

fn parse(s str) !int {
    if s == "" {
        return Err(ParseError.UnexpectedEnd)
    }
    Ok(42)
}

let r = parse("")
is r {
    Ok(n) -> print(n)
    Err(e) -> is e {
        ParseError.UnexpectedEnd -> print("unexpected end")
        _ -> print("other error")
    }
}

let r2 = parse("hello")
is r2 {
    Ok(n) -> print(n)
    Err(e) -> print("error")
}
```

`result_enum_error.expected.out`:
```
unexpected end
42
```

**Note**: `Err(ParseError.UnexpectedEnd)` passes the enum variant's heap ID as the error value. `UNWRAP_ERR` extracts it. `is e { ParseError.UnexpectedEnd -> ... }` uses `IS_VARIANT("ParseError.UnexpectedEnd")`.

---

## Task 11: Test — Err carries multi-field enum variant

**Files:**
- Create: `crates/auto-lang/test/vm/16_option_result/006_result_multi_error/result_multi_error.at`
- Create: `crates/auto-lang/test/vm/16_option_result/006_result_multi_error/result_multi_error.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

`result_multi_error.at`:
```auto
// Test: Err carries multi-field enum variant with destructuring
enum ApiError {
    Http { code int, msg str }
    Timeout
}

fn fetch(url str) !str {
    if url == "" {
        return Err(ApiError.Http(code: 404, msg: "not found"))
    }
    if url == "slow" {
        return Err(ApiError.Timeout)
    }
    Ok("response")
}

let r1 = fetch("")
is r1 {
    Ok(data) -> print(data)
    Err(e) -> is e {
        ApiError.Http(code, msg) -> print(code)
        ApiError.Timeout -> print("timeout")
    }
}

let r2 = fetch("slow")
is r2 {
    Ok(data) -> print(data)
    Err(e) -> is e {
        ApiError.Http(code, msg) -> print(code)
        ApiError.Timeout -> print("timeout")
    }
}

let r3 = fetch("ok")
is r3 {
    Ok(data) -> print(data)
    Err(e) -> print("error")
}
```

`result_multi_error.expected.out`:
```
404
timeout
response
```

---

## Task 12: Commit tests and verify

Commit Tasks 8-11:
```
test(vm): add Result heap object tests (Plan 208 Tasks 8-11)
```

Run full suite: `cargo test -p auto-lang -- vm_file_tests`

---

## Dependency Graph

```
Task 1 (CREATE_OK) ──┐
Task 2 (CREATE_ERR) ──┤──> Task 5 (ERROR_PROPAGATE) ──> Task 6 (Codegen IS_VARIANT)
Task 3 (IS_OK) ───────┤                                         │
Task 4 (UNWRAP) ──────┘                                         ├──> Task 7 (Commit)
                                                                │
                                            Task 8-11 (Tests) ──┘
```

**Critical path**: Tasks 1-6 are sequential engine/codegen changes → Task 7 commit → Tasks 8-11 tests

## Risks

1. **ERROR_PROPAGATE complexity**: The `?` operator has complex logic for both Option and Result. Must not break Option propagation when adding Result propagation.
2. **Heap ID range**: Existing code may assume positive values are plain integers, not heap IDs. The `value > 0` / `value >= 4000000` checks need auditing.
3. **`GET_GENERIC_FIELD` stack side effect**: It reads instance_id from stack WITHOUT popping (line 2080: `read_i32(sp-1)`). After extracting the field, the instance_id is still on stack. The codegen needs to POP after extraction, or the stack will be corrupted.
4. **Backward compat**: Some tests may use `Ok(42)` and expect `42` on stack (plain integer). After heapification, they get a heap ID instead. Tests using `is result { Ok(x) -> print(x) }` should work because `GET_GENERIC_FIELD` extracts the inner value.
