# Plan 206: Closure HOF — call_closure API + List 高阶函数

> **Status: ✅ COMPLETE** — call_closure API + List.map/filter/reduce/for_each/find/any/all all implemented and tested
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 AutoVM 添加 native→closure 回调 API，然后基于此实现 `List.map/filter/reduce/for_each/find/any/all` 六个高阶函数。

**Architecture:** 从 `execute_task` 提取核心调度循环为 `run_one_instruction`，新增 `AutoVM::call_closure` 公共方法。Native shim 通过该方法回调 Auto 闭包。List 高阶函数的 Rust 实现在循环中调用 `call_closure` 处理每个元素。

**Tech Stack:** Rust, AutoVM engine refactoring, DashMap interior mutability

**Depends on:** Plan 201 Phase 1A-C (enum multi-field, ✅ completed), Plan 200 Phase 1-2 (✅ completed)

---

## Task 1: 提取 `run_one_instruction` 并重构 `execute_task`

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:779-4079`

**Step 1: 在 `execute_task` 之前添加 `run_one_instruction` 方法**

在 `execute_task` 方法之前（约 line 778），添加新方法：

```rust
/// Execute a single instruction. Returns Ok(true) if should continue,
/// Ok(false) if task should terminate, Err on VM error.
fn run_one_instruction(&self, task: &mut AutoTask) -> Result<bool, VMError> {
    if task.ip >= self.flash.memory.len() {
        return Ok(false);
    }

    let op_byte = self.flash.read_u8(task.ip);
    task.ip += 1;
    let op: OpCode = op_byte.into();

    match op {
        // ... 将 execute_task 中 match op { ... } 的全部内容搬入此处 ...
        // 但将以下终止条件改为 return Ok(false):
        // - HALT opcode → return Ok(false)
        // - RET/RET_D when bp == 0 → return Ok(false)
        // 其他所有分支保持不变
    }

    Ok(true)
}
```

**注意**：将 `execute_task` 中整个 `match op { ... }` 块（约 line 797-4073）搬到 `run_one_instruction` 中。`execute_task` 改为调用 `run_one_instruction` 的循环。

**Step 2: 重构 `execute_task` 为调用者**

```rust
fn execute_task(&self, task: &mut AutoTask) -> Result<TaskStatus, VMError> {
    let budget = 100;
    for _ in 0..budget {
        match self.run_one_instruction(task)? {
            true => continue,
            false => return Ok(TaskStatus::Terminated),
        }
    }
    Ok(TaskStatus::Ready)
}
```

**Step 3: 验证重构未破坏现有测试**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: 所有测试通过（与重构前相同）

**Step 4: Commit**

```
refactor(vm): extract run_one_instruction from execute_task
```

---

## Task 2: 实现 `call_closure` 公共 API

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs`（新增方法）

**Step 1: 在 AutoVM impl 中添加 `call_closure` 方法**

在 `impl AutoVM` 块中（`pub fn get_string` 之后，约 line 530），添加：

```rust
/// Call an Auto closure from native code.
///
/// Stack effect: pops closure_id + args, pushes result.
///
/// # Arguments
/// * `task` - Current task (mutable)
/// * `closure_id` - ID of closure to call
/// * `arg_count` - Number of arguments already on stack (below closure_id)
///
/// # Returns
/// Ok(()) with result on stack, or Err on execution failure.
pub fn call_closure(
    &self,
    task: &mut AutoTask,
    closure_id: u32,
    arg_count: usize,
) -> Result<(), VMError> {
    let closure = self.closures.get(&closure_id).cloned()
        .ok_or_else(|| VMError::RuntimeError(format!("Invalid closure ID: {}", closure_id)))?;

    // Save state for restoration after closure returns
    let saved_ip = task.ip;
    let saved_bp = task.bp;
    let saved_closure_id = task.current_closure_id;
    let saved_fn_n_args = task.current_fn_n_args;

    // Setup closure context
    task.current_closure_id = Some(closure_id);
    task.current_fn_n_args = closure.n_args;
    task.saved_closure_id = saved_closure_id;

    // Setup stack frame (same as CALL_CLOSURE opcode)
    task.ram.push_i32(saved_ip as i32);  // Return address
    task.ram.push_i32(saved_bp as i32);  // Old BP
    task.bp = task.ram.sp - 1;

    // Jump to closure body
    task.ip = closure.func_addr as usize;

    // Execute until we return to saved_bp
    let budget = 1_000_000;
    for _ in 0..budget {
        match self.run_one_instruction(task)? {
            true => {
                // Check if RET has restored us to the calling frame
                if task.bp == saved_bp {
                    // Result is already on stack from RET
                    // But RET puts result at new_sp-1, and restored BP/IP.
                    // The result should be at the right place on stack.
                    break;
                }
                continue;
            }
            false => {
                return Err(VMError::RuntimeError(
                    "Closure execution terminated unexpectedly".into()
                ));
            }
        }
    }

    // Restore non-stack state
    task.current_closure_id = saved_closure_id;
    task.current_fn_n_args = saved_fn_n_args;

    Ok(())
}
```

**注意**：需要 `clone()` 闭包（或仅复制 `func_addr` + `n_args`），因为 `DashMap::get` 返回引用_guard，不能跨 `yield` 点持有。由于 `Closure` 只有 `u32 + HashMap + usize`，clone 成本可接受。

**Step 2: 验证编译通过**

Run: `cargo build -p auto-lang`
Expected: 编译成功

**Step 3: Commit**

```
feat(vm): add call_closure API for native→AutoVM closure callback
```

---

## Task 3: 添加 List 高阶函数常量和注册

**Files:**
- Modify: `crates/auto-lang/src/vm/native.rs`（常量 + shim 声明 + 注册）
- Modify: `crates/auto-lang/src/vm/native_registry.rs`（名称注册）

**Step 1: 添加常量**

在 `native.rs` 的常量区（约 line 470 之后）添加：

```rust
// List higher-order functions (Plan 206)
pub const NATIVE_LIST_MAP: u16 = 2060;
pub const NATIVE_LIST_FILTER: u16 = 2061;
pub const NATIVE_LIST_FOREACH: u16 = 2062;
pub const NATIVE_LIST_FIND: u16 = 2063;
pub const NATIVE_LIST_ANY: u16 = 2064;
pub const NATIVE_LIST_ALL: u16 = 2065;
pub const NATIVE_LIST_REDUCE: u16 = 2066;
```

**Step 2: 在 `register_std_shims` 中注册**

在 List 函数注册区（约 line 161 之后）添加：

```rust
// List higher-order functions (Plan 206)
self.register(NATIVE_LIST_MAP, shim_list_map);
self.register(NATIVE_LIST_FILTER, shim_list_filter);
self.register(NATIVE_LIST_FOREACH, shim_list_for_each);
self.register(NATIVE_LIST_FIND, shim_list_find);
self.register(NATIVE_LIST_ANY, shim_list_any);
self.register(NATIVE_LIST_ALL, shim_list_all);
self.register(NATIVE_LIST_REDUCE, shim_list_reduce);
```

**Step 3: 在 `native_registry.rs` 中注册名称**

在 `register_builtin_natives()` 函数的 List 部分（约 line 170 之后）添加：

```rust
// List higher-order functions (Plan 206)
registry.register_with_id("List.map", 2060);
registry.register_with_id("List.filter", 2061);
registry.register_with_id("List.for_each", 2062);
registry.register_with_id("List.find", 2063);
registry.register_with_id("List.any", 2064);
registry.register_with_id("List.all", 2065);
registry.register_with_id("List.reduce", 2066);
```

**Step 4: Commit**

```
feat(vm): register List higher-order function IDs and names
```

---

## Task 4: 实现 List.map 和 List.filter shim

**Files:**
- Modify: `crates/auto-lang/src/vm/native.rs`

**Step 1: 实现辅助函数 `get_list_elements`**

在 shim 函数区域之前添加辅助函数：

```rust
/// Get a clone of list elements from a list heap object ID.
fn get_list_elements(vm: &AutoVM, list_id: u64) -> Result<Vec<Value>, VMError> {
    let obj = vm.heap_objects.get(&list_id)
        .ok_or_else(|| VMError::RuntimeError(format!("Invalid list ID: {}", list_id)))?;
    let guard = obj.read().map_err(|_| VMError::RuntimeError("List lock poisoned".into()))?;
    // Try to downcast to ListData
    let list_data = guard.as_any().downcast_ref::<ListData<Value>>()
        .ok_or_else(|| VMError::RuntimeError("Not a ListData".into()))?;
    Ok(list_data.elems.clone())
}

/// Create a new list heap object from elements, return heap ID.
fn create_list_from_elements(vm: &AutoVM, elems: Vec<Value>) -> u64 {
    let list_data = ListData { elems, storage: None };
    let id = vm.heap_object_id_gen.fetch_add(1, Ordering::Relaxed);
    vm.heap_objects.insert(id, Arc::new(RwLock::new(list_data)));
    id
}
```

**注意**：需要确认 `ListData` 实现了 `HeapObject` trait。如果没有，需要改用 `VmRefData::List` 路径或实现该 trait。编译时确认。

**Step 2: 实现 `shim_list_map`**

```rust
/// List.map(closure) → new List
/// Stack: closure_id, list_id → result_list_id
pub fn shim_list_map(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_elements(vm, list_id)?;
    let mut results = Vec::with_capacity(elements.len());

    for elem in elements {
        // Push element as arg, then closure_id
        task.ram.push_i32(elem.as_i32().unwrap_or(0));
        vm.call_closure(task, closure_id, 1)?;
        let mapped = task.ram.pop_i32();
        results.push(Value::Int(mapped));
    }

    let new_id = create_list_from_elements(vm, results);
    task.ram.push_i32(new_id as i32);
    Ok(())
}
```

**Step 3: 实现 `shim_list_filter`**

```rust
/// List.filter(closure) → new List
/// Stack: closure_id, list_id → result_list_id
pub fn shim_list_filter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_elements(vm, list_id)?;
    let mut results = Vec::new();

    for elem in elements {
        let val = elem.as_i32().unwrap_or(0);
        task.ram.push_i32(val);
        vm.call_closure(task, closure_id, 1)?;
        let predicate = task.ram.pop_i32();
        if predicate != 0 {
            results.push(Value::Int(val));
        }
    }

    let new_id = create_list_from_elements(vm, results);
    task.ram.push_i32(new_id as i32);
    Ok(())
}
```

**Step 4: 验证编译通过**

Run: `cargo build -p auto-lang`
Expected: 编译成功（可能有类型调整，见注意事项）

**Step 5: Commit**

```
feat(vm): implement List.map and List.filter native shims
```

---

## Task 5: 实现 List.reduce, List.for_each, List.find, List.any, List.all shim

**Files:**
- Modify: `crates/auto-lang/src/vm/native.rs`

**Step 1: 实现 `shim_list_reduce`**

```rust
/// List.reduce(init, closure) → accumulated value
/// Stack: closure_id, init_val, list_id → result
pub fn shim_list_reduce(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let init_val = task.ram.pop_i32();
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_elements(vm, list_id)?;
    let mut acc = init_val;

    for elem in elements {
        let val = elem.as_i32().unwrap_or(0);
        // Push acc and elem as args (2 args)
        task.ram.push_i32(acc);
        task.ram.push_i32(val);
        vm.call_closure(task, closure_id, 2)?;
        acc = task.ram.pop_i32();
    }

    task.ram.push_i32(acc);
    Ok(())
}
```

**Step 2: 实现 `shim_list_for_each`**

```rust
/// List.for_each(closure) → void
/// Stack: closure_id, list_id → (no result, push 0)
pub fn shim_list_for_each(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_elements(vm, list_id)?;

    for elem in elements {
        let val = elem.as_i32().unwrap_or(0);
        task.ram.push_i32(val);
        vm.call_closure(task, closure_id, 1)?;
        task.ram.pop_i32(); // Discard result
    }

    task.ram.push_i32(0); // void
    Ok(())
}
```

**Step 3: 实现 `shim_list_find`**

```rust
/// List.find(closure) → ?T (found value or -1 for None)
/// Stack: closure_id, list_id → result
pub fn shim_list_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_elements(vm, list_id)?;

    for elem in elements {
        let val = elem.as_i32().unwrap_or(0);
        task.ram.push_i32(val);
        vm.call_closure(task, closure_id, 1)?;
        let found = task.ram.pop_i32();
        if found != 0 {
            task.ram.push_i32(val);
            return Ok(());
        }
    }

    task.ram.push_i32(-1); // None
    Ok(())
}
```

**Step 4: 实现 `shim_list_any` 和 `shim_list_all`**

```rust
/// List.any(closure) → bool
/// Stack: closure_id, list_id → bool (1/0)
pub fn shim_list_any(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_elements(vm, list_id)?;

    for elem in elements {
        let val = elem.as_i32().unwrap_or(0);
        task.ram.push_i32(val);
        vm.call_closure(task, closure_id, 1)?;
        let result = task.ram.pop_i32();
        if result != 0 {
            task.ram.push_i32(1);
            return Ok(());
        }
    }

    task.ram.push_i32(0);
    Ok(())
}

/// List.all(closure) → bool
/// Stack: closure_id, list_id → bool (1/0)
pub fn shim_list_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    let elements = get_list_elements(vm, list_id)?;

    for elem in elements {
        let val = elem.as_i32().unwrap_or(0);
        task.ram.push_i32(val);
        vm.call_closure(task, closure_id, 1)?;
        let result = task.ram.pop_i32();
        if result == 0 {
            task.ram.push_i32(0);
            return Ok(());
        }
    }

    task.ram.push_i32(1);
    Ok(())
}
```

**Step 5: 验证编译通过**

Run: `cargo build -p auto-lang`
Expected: 编译成功

**Step 6: Commit**

```
feat(vm): implement List.reduce/for_each/find/any/all native shims
```

---

## Task 6: Codegen — 闭包作为 CALL_NAT 参数

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs`

当前 codegen 在编译 `list.map(x => x * 2)` 时，需要：
1. 先编译闭包表达式（emit CLOSURE → closure_id on stack）
2. 编译 list 表达式（list_id on stack）
3. Emit CALL_NAT with `List.map` native ID

**问题**：需要确认 codegen 在编译方法调用 `expr.method(closure)` 时，是否正确地将闭包表达式编译为 closure_id。

**Step 1: 在 codegen.rs 的方法调用路径中确认闭包参数处理**

在编译 `call_expr` 时（约 line 4800-4900），当解析到 `list.map(x => x * 2)`：
- `list` 编译为 LOAD → list_id
- `x => x * 2` 编译为 CLOSURE → closure_id
- `List.map` 解析为 native ID 2060
- Emit CALL_NAT

**检查**：当前 codegen 对 `Expr::Closure` 在函数参数位置的处理是否正确。如果 `compile_expr` 已经能处理 `Expr::Closure`（emit CLOSURE opcode），那么参数位置无需特殊处理。

**Step 2: 如有需要，添加 Fn 类型参数的 codegen 支持**

如果 codegen 对 native 调用参数中的 `Expr::Closure` 已能正确编译（通过 `compile_expr` → `Expr::Closure` 分支），则无需额外代码。

否则，在编译 CALL_NAT 参数时，检测参数是否为 `Expr::Closure`，如果是则正常编译（会 emit LOAD捕获 + CLOSURE + JMP + body + RET）。

**Step 3: 验证**

Run: `cargo build -p auto-lang`
Expected: 编译成功

**Step 4: Commit（如有改动）**

```
feat(vm): support closure expressions as native function arguments
```

---

## Task 7: 测试用例 — call_closure 基础功能

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/010_closure_callback/closure_callback.at`
- Create: `crates/auto-lang/test/vm/09_functions/010_closure_callback/closure_callback.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: 创建测试 — 基础闭包回调**

`closure_callback.at`:
```auto
// Test: native function calls an Auto closure
// List.map with closure
let nums = [1, 2, 3, 4, 5]
let doubled = nums.map(x => x * 2)
print(doubled)
```

**注意**：这个测试依赖 codegen 能正确编译 `nums.map(x => x * 2)`。如果 codegen 路径尚未支持，先写一个手动版本测试 call_closure API：

```auto
// Minimal test: closure creation and direct call
let double = (x int) => x * 2
let result = double(21)
print(result)
```

`closure_callback.expected.out`:
```
42
```

**Step 2: 注册测试函数**

在 `vm_file_tests.rs` 中添加：
```rust
#[test] fn test_09_functions_010_closure_callback() { test_vm("09_functions/010_closure_callback").unwrap(); }
```

**Step 3: 运行测试**

Run: `cargo test -p auto-lang -- test_09_functions_010_closure_callback`
Expected: PASS

**Step 4: Commit**

```
test(vm): add closure callback base test
```

---

## Task 8: 测试用例 — List.map 链式调用

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/011_list_map_chain/list_map_chain.at`
- Create: `crates/auto-lang/test/vm/09_functions/011_list_map_chain/list_map_chain.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: 创建测试**

`list_map_chain.at`:
```auto
// Test: List.map with chained operations
let nums = [1, 2, 3, 4, 5]
let evens = nums.filter(x => x % 2 == 0)
let doubled = nums.map(x => x * 2)
print(evens.len())
print(doubled.len())
```

`list_map_chain.expected.out`:
```
2
5
```

**Step 2: 注册测试函数**

```rust
#[test] fn test_09_functions_011_list_map_chain() { test_vm("09_functions/011_list_map_chain").unwrap(); }
```

**Step 3: 运行测试**

Run: `cargo test -p auto-lang -- test_09_functions_011_list_map_chain`
Expected: PASS

**Step 4: Commit**

```
test(vm): add List.map/filter chain test
```

---

## Task 9: 测试用例 — List.reduce 和 List.find

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/012_list_reduce_find/list_reduce_find.at`
- Create: `crates/auto-lang/test/vm/09_functions/012_list_reduce_find/list_reduce_find.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: 创建测试**

`list_reduce_find.at`:
```auto
// Test: List.reduce — sum of elements
let nums = [1, 2, 3, 4, 5]
let sum = nums.reduce(0, (acc, x) => acc + x)
print(sum)

// Test: List.find — first element > 3
let found = nums.find(x => x > 3)
print(found)

// Test: List.any / List.all
let has_neg = nums.any(x => x < 0)
let all_pos = nums.all(x => x > 0)
print(has_neg)
print(all_pos)
```

`list_reduce_find.expected.out`:
```
15
4
0
1
```

**Step 2: 注册测试函数**

```rust
#[test] fn test_09_functions_012_list_reduce_find() { test_vm("09_functions/012_list_reduce_find").unwrap(); }
```

**Step 3: 运行测试**

Run: `cargo test -p auto-lang -- test_09_functions_012_list_reduce_find`
Expected: PASS

**Step 4: Commit**

```
test(vm): add List.reduce/find/any/all tests
```

---

## Task 10: 测试用例 — 闭包捕获变量

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.at`
- Create: `crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: 创建测试 — 闭包捕获外层变量 + HOF**

`closure_capture_hof.at`:
```auto
// Test: closure captures outer variable and used in HOF
let factor = 3
let nums = [1, 2, 3]
let scaled = nums.map(x => x * factor)
// scaled should be [3, 6, 9]
print(scaled.len())

// Test: nested closure capture
let threshold = 10
let big = nums.filter(x => x * factor > threshold)
print(big.len())
```

`closure_capture_hof.expected.out`:
```
3
2
```

**注意**：这个测试验证 `call_closure` 能正确恢复闭包的捕获环境（`LOAD_CAPTURED` opcode）。如果捕获环境在 `call_closure` 中未正确设置，此测试会失败。

**Step 2: 注册测试函数**

```rust
#[test] fn test_09_functions_013_closure_capture_hof() { test_vm("09_functions/013_closure_capture_hof").unwrap(); }
```

**Step 3: 运行测试**

Run: `cargo test -p auto-lang -- test_09_functions_013_closure_capture_hof`
Expected: PASS

**Step 4: Commit**

```
test(vm): add closure capture + HOF test
```

---

## Task 11: 测试用例 — for_each 副作用 + 边界情况

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/014_list_for_each_edge/list_for_each_edge.at`
- Create: `crates/auto-lang/test/vm/09_functions/014_list_for_each_edge/list_for_each_edge.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: 创建测试**

`list_for_each_edge.at`:
```auto
// Test: for_each with side effects
let nums = [10, 20, 30]
nums.for_each(x => print(x))

// Test: empty list operations
let empty = []
let mapped = empty.map(x => x * 2)
print(mapped.len())

let filtered = empty.filter(x => x > 0)
print(filtered.len())

let sum = empty.reduce(0, (acc, x) => acc + x)
print(sum)

// Test: find on empty list
let found = empty.find(x => x > 0)
print(found)
```

`list_for_each_edge.expected.out`:
```
10
20
30
0
0
0
-1
```

**Step 2: 注册测试函数**

```rust
#[test] fn test_09_functions_014_list_for_each_edge() { test_vm("09_functions/014_list_for_each_edge").unwrap(); }
```

**Step 3: 运行测试**

Run: `cargo test -p auto-lang -- test_09_functions_014_list_for_each_edge`
Expected: PASS

**Step 4: Commit**

```
test(vm): add for_each and edge case tests for List HOF
```

---

## Task 12: 测试用例 — map→filter 链式管道

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.at`
- Create: `crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: 创建测试 — 链式管道（最接近真实场景）**

`list_chain_pipeline.at`:
```auto
// Test: realistic pipeline — filter, map, reduce chain
let nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

// Get even numbers, double them, sum
let evens = nums.filter(x => x % 2 == 0)
let doubled = evens.map(x => x * 2)
let total = doubled.reduce(0, (acc, x) => acc + x)
print(total)

// Verify intermediate lengths
print(evens.len())
print(doubled.len())
```

`list_chain_pipeline.expected.out`:
```
60
5
5
```

**说明**：`evens` = [2, 4, 6, 8, 10], `doubled` = [4, 8, 12, 16, 20], sum = 60

**Step 2: 注册测试函数**

```rust
#[test] fn test_09_functions_015_list_chain_pipeline() { test_vm("09_functions/015_list_chain_pipeline").unwrap(); }
```

**Step 3: 运行全部测试**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: 所有测试通过

**Step 4: Commit**

```
test(vm): add chained pipeline test for List HOF
```

---

## 实施顺序和依赖关系

```
Task 1 (extract run_one_instruction) ──> Task 2 (call_closure API)
                                                │
                                                ├──> Task 3 (常量+注册)
                                                │       │
                                                │       └──> Task 4 (map/filter)
                                                │               │
                                                │               └──> Task 5 (reduce/find/any/all)
                                                │
                                                └──> Task 6 (codegen 闭包参数)
                                                        │
                                                        ├──> Task 7  (基础测试)
                                                        ├──> Task 8  (map/filter 测试)
                                                        ├──> Task 9  (reduce/find 测试)
                                                        ├──> Task 10 (捕获变量测试)
                                                        ├──> Task 11 (for_each+边界测试)
                                                        └──> Task 12 (链式管道测试)
```

**关键路径**：Task 1 → 2 → 3 → 4 → 5 → 6 → 7-12（可并行）

**风险点**：
1. `ListData` 可能未实现 `HeapObject` trait — 需要改用现有 list 存储路径（`shim_list_push` 中的模式）
2. 闭包参数类型可能不是 `i32` — 需要处理 `Value` 枚举的所有变体
3. `call_closure` 中的 `bp == saved_bp` 检测 — 需要确认 RET 正确恢复了 BP

## 后续工作（本 Plan 不包含）

- Plan 201 Phase 2D：字符串 `.chars().map()` — 需要 `str.chars()` native 返回 `List<str>`
- Plan 201 Phase 2E：Map 的 `.map()` / `.filter()` / `.keys()` / `.values()`
- Plan 200 Phase 3.3：`.map_err()` — 基于 `call_closure` 的 Result 系列方法
