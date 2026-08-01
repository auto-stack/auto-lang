# Plan 383：命名函数引用作为值 — 让 `handler`（裸函数名）可在值位置传递

> **Status**: ✅ 已完成（2026-08-01）— 见 §10「实施记录」
> **来源**: Plan 379 §5 遗留 —— 放宽 `route` 关键字后，移植 axum 层的下一个障碍：
> `.route("/", handler)` 里 `handler`（裸函数名）被当成未定义变量。
> **影响仓库**: `auto-lang`（`crates/auto-lang/src/vm/codegen.rs`、`src/trans/rust.rs`）
> **风险**: 中 — 触动 `Expr::Ident` 核心解析链；需保证函数调用 `handler()` 行为不变
> **继任关系**: Plan 379 的直接延续

---

## 1. 问题

### 1.1 症状

auto-musk 移植 axum Web 层，核心模式（Rust）：

```rust
Router::new().route("/", get(handler)).route("/api/x", get(x))
```

移植到 Auto 后，`.route("/", handler)` 里的 `handler`（已定义函数，作为值传递而非调用）报错：

```auto
fn handler() str { return "ok" }
fn build(r Router) Router {
    return r.route("/", handler)   // ← Error: Undefined variable: handler
}
```

### 1.2 复现（最小用例，无 axum 依赖）

```auto
fn double(x int) int { return x * 2 }
fn main() {
    let f = double      // ← Undefined variable: double
    print(f(5))
}
```

对比：**lambda/closure 作为值已经能工作**：

```auto
fn main() {
    let f = () => { return 42 }   // ✅ 工作
    print(f())                    // ✅ 输出 42
}
```

**只有命名函数的引用传递坏了。**

### 1.3 根因（双路径）

**VM 路径**（`codegen.rs:4949-5083`，`Expr::Ident` 处理链）：

`Expr::Ident(name)` 的值位置处理依次查：actor state field → 捕获变量 → 局部变量 → 全局变量 → import_scope → enum variant → 隐式 self.field → rust crate/module prefix → py_native → **`Undefined variable` 报错**。

**这条链里完全没有"检查是否是已定义函数名"这一环。** 一个裸函数名 `handler` 不带 `()` 时，遍历完所有变量查找后直接报错。而 `codegen.rs:3996` / `5049` 已经有 `self.exports.contains_key(name)` 的查询能力——只是没接入这条链。

**a2r 路径**（`rust.rs:1416-1440`，`Expr::Ident` emit）：

裸函数名走 `else { write!(out, "{}", rust_ident(name)) }` → 直接输出 `handler`（Rust 里合法的函数引用）。但参数传递时的**自动借用（auto-borrow）escape 分析**误判，给它加了 `.clone()` → 输出 `handler.clone()`。

实测 a2r 转译 `apply(handler)`：
```rust
// 期望：let s = apply(handler);
// 实际：let s = apply(handler.clone());   // ← 多余的 .clone()
```

> 注：`fn()` 类型在 Rust 里实现了 `Copy`/`Clone`，所以 `handler.clone()` **能编译**，但这是不干净的代码，且暴露了 escape 分析把函数引用误判为需要 clone 的类型。

---

## 2. 设计目标（对标 Rust 语义）

Rust 里函数项（function item）在值位置自动 coerce 为函数指针 `fn(Args) -> Ret`，可被传递、存储、调用。Auto 应支持等价语义：

**核心规则**：当一个标识符 `name` 满足：
1. 匹配某个已定义的函数名（在 `exports` / `fn_decls` 中），且
2. 出现在**值位置**（非调用形式 `name(...)`，调用走 `Expr::Call`），

则把它当作**函数引用值**（function reference）。

**不变性**：
- 函数调用 `handler()` 行为完全不变（仍走 `Expr::Call` → CALL/CALL_NAT）。
- 不引入新语法（无 `&handler`、`fn handler` 等显式标记）——对标 Python/Rust 的隐式引用。
- VM 和 a2r 两条路径行为一致。

---

## 3. 方案

### 3.1 VM 路径：函数引用作为零捕获 closure

**插入点**：`codegen.rs:4949` `Expr::Ident` 处理链，在所有变量查找失败后、`Undefined variable` 报错前（约 line 5037 `vm_debug!("Variable {} NOT FOUND")` 之后），新增一环：

```rust
// Plan 383: 命名函数引用 — 裸函数名在值位置当作函数引用。
// 复用 closure 运行时：把函数包装成零捕获的 ClosureValue，
// 使其可通过 CALL_CLOSURE 调用（与 lambda 一致）。
// 仅当标识符是已定义函数（在 exports 中）且非调用位置时触发。
if self.exports.contains_key(&name_str) {
    // 把函数地址包装成 closure 值推上栈。
    // 方案 A（推荐）：新增 FUNC_REF opcode —— 最小运行时 footprint。
    // 方案 B：复用 CLOSURE opcode（capture_count=0）—— 不加新 opcode。
    self.emit(OpCode::FUNC_REF);   // 方案 A
    let func_addr_placeholder = self.code.len();
    self.code.extend_from_slice(&0u32.to_le_bytes());
    self.relocs.push(RelocEntry {
        offset: func_addr_placeholder as u32,
        symbol_name: name_str.clone(),
        reloc_type: RelocType::FuncCall,
        source_pos: None,
    });
    self.last_expr_type = ObjectType::NestedObject;  // 函数引用是可调用对象
    return Ok(());
}
```

**运行时表示**（方案 A：新增 `FUNC_REF` opcode）：

```rust
// engine.rs
OpCode::FUNC_REF => {
    // Immediate: func_addr (u32)
    // 把函数地址包装成零捕获 ClosureValue，push closure_id 到栈
    let func_addr = self.flash.read_u32(task.ip);
    task.ip += 4;
    let closure = ClosureValue {
        func_addr,
        captures: HashMap::new(),
        n_args: <从符号表或运行时获取>,
    };
    let closure_id = self.next_closure_id();
    self.closures.insert(closure_id, closure);
    task.ram.push_i32(closure_id as i32);
}
```

**调用机制**：复用既有 `CALL_CLOSURE` opcode。`f(5)` 其中 `f` 是函数引用 → codegen 看到 `Expr::Call { name: Expr::Ident("f"), ... }`，且 `f` 是局部变量（持有 closure_id）→ 走 CALL_CLOSURE（与 lambda 调用完全一致）。

> **方案 A vs B 取舍**：方案 A（新 opcode `FUNC_REF`）语义清晰、与 CLOSURE 解耦（函数引用不需要捕获环境）；方案 B 复用 CLOSURE（capture_count=0）不增加 opcode 但语义略绕。**推荐方案 A**，但若 `n_args` 难以在 FUNC_REF 处获取，退方案 B（CLOSURE 的 n_args 字段已存在）。

### 3.2 VM 路径：调用端的识别

函数引用的调用 `f(5)`（`f` 是持有 closure_id 的局部变量）**已经能工作**——因为 codegen 对 `Expr::Call { name: Expr::Ident(f) }` 在 `f` 是局部变量时已经会走 CALL_CLOSURE 分支（lambda 验证过）。**无需额外改动调用端。**

需验证：`Expr::Call` 处理里，callee 是局部变量时的分流逻辑（codegen.rs ~7874）是否会正确选择 CALL_CLOSURE。若当前只对 lambda 生效，需补"局部变量持有 closure_id → CALL_CLOSURE"的判断。

### 3.3 a2r 路径：去掉函数引用的误加 .clone()

**问题定位**：`rust.rs` 的参数 emit 路径里，auto-borrow/escape 分析对所有"未知类型"的参数默认加 `.clone()`（防御性）。函数引用被当作未知类型。

**修复**：在 auto-borrow 决策处，识别"参数是命名函数引用"（裸 ident 匹配已定义函数名）并跳过 clone。

插入点（`rust.rs` 参数 emit，emit_borrow 调用处）：

```rust
// Plan 383: 命名函数引用不需要 clone（fn 类型是 Copy）。
// 裸 ident 匹配已定义函数名时，直接输出 ident，不加 .clone()。
if let Expr::Ident(name) = arg_expr {
    if self.is_defined_function(name) {
        write!(out, "{}", Self::rust_ident(name))?;
        continue;  // 跳过 auto-borrow
    }
}
```

需新增 helper `is_defined_function(&self, name: &str) -> bool`：检查 `self.fn_decls` / 函数名集合是否包含该名字。

**生成结果对比**：
```rust
// 修复前：let s = apply(handler.clone());
// 修复后：let s = apply(handler);   // ← 干净的函数引用
```

---

## 4. 测试用例

### 4.1 VM 路径 file-based 测试

**位置**：`test/vm/27_function_reference/`

**`001_basic_ref/basic_ref.at`**（最小用例）：
```auto
fn double(x int) int { return x * 2 }
fn main() {
    let f = double
    print(f(5))
    print(f(10))
}
```
**`basic_ref.expected.out`**：
```
10
20
```

**`002_pass_as_arg/pass_as_arg.at`**（传参 + 调用）：
```auto
fn double(x int) int { return x * 2 }
fn apply(f fn(int)int, x int) int { return f(x) }
fn main() {
    print(apply(double, 21))
}
```
**`pass_as_arg.expected.out`**：
```
42
```

**`003_route_pattern/route_pattern.at`**（模拟 axum `.route("/", handler)` 模式）：
```auto
fn handler() str { return "ok" }
fn build(r Router) Router {
    return r.route("/", handler)
}
```
> 注：此用例依赖 `Router` 类型；若 VM 无该类型，用 `003_method_arg` 替代——
> 自定义带方法的类型，方法接受函数引用实参。或仅断言"不报 Undefined variable"。

**`004_call_unchanged/call_unchanged.at`**（回归守护：直接调用行为不变）：
```auto
fn double(x int) int { return x * 2 }
fn main() {
    print(double(5))   // 直接调用，非引用
}
```
**`call_unchanged.expected.out`**：
```
10
```

### 4.2 a2r 路径测试

新增 a2r 测试断言生成代码不含 `.clone()`：

```rust
// 在 a2r 测试模块
#[test]
fn fn_reference_no_clone() {
    let src = r#"
        fn handler() str { return "ok" }
        fn apply(f fn()str) str { return f() }
        fn main() { let s = apply(handler); print(s) }
    "#;
    let sink = transpile_rust("test", src).unwrap();
    let code = sink.as_str();
    assert!(code.contains("apply(handler)"), "expected clean fn ref, got: {}", code);
    assert!(!code.contains("handler.clone()"), "fn ref should not clone: {}", code);
}
```

### 4.3 注册到 vm_file_tests.rs

```rust
// === 27_function_reference (Plan 383) ===
#[test] #[ignore] fn test_27_function_reference_001_basic_ref() { test_vm("27_function_reference/001_basic_ref").unwrap(); }
#[test] #[ignore] fn test_27_function_reference_002_pass_as_arg() { test_vm("27_function_reference/002_pass_as_arg").unwrap(); }
#[test] #[ignore] fn test_27_function_reference_004_call_unchanged() { test_vm("27_function_reference/004_call_unchanged").unwrap(); }
```

---

## 5. 实施步骤

| 步骤 | 内容 | 验证 |
|------|------|------|
| 1 | 写 §4.1 file-based 测试（001/002/004）+ 注册 | 测试失败（红色）—— 精确暴露 bug |
| 2 | VM：codegen.rs `Expr::Ident` 链加函数引用分支（§3.1） | 001 转绿 |
| 3 | VM：新增 `FUNC_REF` opcode（opcode.rs/engine.rs/disasm.rs） | 001/002 编译运行 |
| 4 | VM：验证 `f()` 调用走 CALL_CLOSURE（§3.2，必要时补判断） | 002 转绿 |
| 5 | a2r：rust.rs 参数 emit 跳过函数引用的 clone（§3.3） | §4.2 a2r 测试通过 |
| 6 | 回归：`cargo test -p auto-lang` + a2r 测试 | 零新增失败 |

**TDD**：步骤 1 先红 → 2-5 修复转绿 → 6 守护。

---

## 6. 验收标准

1. ✅ `let f = double`（命名函数引用）在 VM 模式可执行，不报 `Undefined variable`。
2. ✅ `f(5)`（通过函数引用调用）返回正确结果。
3. ✅ `apply(double, 21)`（函数引用作参数）返回 `42`。
4. ✅ `.route("/", handler)`（方法实参，axum 模式）不报错。
5. ✅ `double(5)`（直接调用）行为不变（回归守护 004）。
6. ✅ a2r 转译 `apply(handler)` 生成 `apply(handler)` 而非 `apply(handler.clone())`。
7. ✅ §4 全部测试 + 既有回归零新增失败。
8. ✅ auto-musk 的 axum `Router::new().route("/", get(handler))` 端到端转译通过（跨仓库验证）。

---

## 7. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| 函数引用分支误触发：某个同名变量本应优先，却被当函数 | 低 | 高 | 分支插在**所有变量查找失败后**，变量永远优先；004 守护直接调用不变 |
| `FUNC_REF` 的 n_args 难获取（方案 A） | 中 | 中 | 退方案 B（复用 CLOSURE，n_args 字段已存在）|
| CALL_CLOSURE 调用端只认 lambda closure_id，不认 FUNC_REF 生成的 | 中 | 高 | 步骤 4 优先验证；FUNC_REF 与 CLOSURE 产出相同 ClosureValue 结构则天然兼容 |
| a2r 的 `is_defined_function` 漏判（如方法、跨模块函数） | 低 | 低 | 先只覆盖同文件 `fn`；方法/跨模块作为 follow-up |
| escape 分析改动影响其他类型 | 低 | 中 | 只对"明确是函数引用"跳过 clone，不改其他类型的借用决策 |

---

## 8. 非目标

- ❌ 不引入显式函数指针语法（`&fn`、`fn*` 等）——对标 Python/Rust 隐式引用。
- ❌ 不做完整函数类型系统（子类型、coercion 规则）——仅值位置隐式引用。
- ❌ 不处理方法引用（`obj.method` 作为值）——仅顶层 `fn` 命名函数。
- ❌ 不处理跨模块函数引用的命名空间解析（`module.func` 作为值）——仅同名作用域 `fn`。

---

## 9. 关联

- **Plan 379**：本计划的直接前置（放宽 `route` 关键字）。379 §5 把本问题列为遗留，本计划清算之。
- **Plan 060**（closure/lambda）：VM 函数值运行时机制的基础，本计划复用。
- **auto-musk Plan 014**：axum 层移植，本计划的端到端验证场景。

---

## 10. 实施记录（2026-08-01）

实施时采用 **方案 B（复用 CLOSURE opcode，零捕获）** 而非方案 A（新 FUNC_REF opcode）——因为 CLOSURE 的 func_addr+capture_count+n_args 布局已满足函数引用需求，且 capture_count=0 时无需新 opcode，改动更小。调用端 `f()` 走既有 CALL_CLOSURE（lambda 已验证）。

### 10.1 实际改动清单

**VM 路径（`codegen.rs`）**
- `Expr::Ident` 处理链（~line 5079）：在 py_native 分支后、`Undefined variable` 报错前，新增函数引用分支——若 `is_defined_function(name)`，emit `CLOSURE` + func_addr(reloc 占位) + capture_count=0 + n_args，设 `last_expr_type = NestedObject`。
- 新增 `is_defined_function(name)` helper（查 `exports.contains_key`）+ `function_param_count(name)` helper（查 `fn_params.len()`，回退 0）。
- `let` 类型推断（~line 1747，`Expr::Ident(src_name)` 分支）：当 `src_name` 是已定义函数时，推断 `Type::Fn(params, ret)` 而非 `Type::Unknown`——这是让 `is_closure_call`（codegen.rs:7255）识别 `f()` 走 CALL_CLOSURE 的关键。

**a2r 路径（`trans/rust.rs`）**
- `is_copy_type`（~line 1304）：补 `Type::Fn(_, _)` 分支。Rust 的 `fn` 指针类型实现 `Copy`，本应识别为 Copy。这是根因修复——让 `apply` 的 `f` 参数不再被标记为 `is_struct_param`（非 Copy），从而 `needs_clone=false`，输出干净 `handler`。
- `emit_borrow`（~line 715）：额外防御——若 `inner` 是 `Expr::Ident` 且匹配已定义函数名，直接 emit 裸 ident，不走 escape tier 分析。
- 新增 `function_names: HashSet<AutoStr>` 字段 + 预扫描收集（主转译分流循环里 `Stmt::Fn` 分支）+ 两个构造点初始化。

### 10.2 调用端无需改动（设计验证）

`f()`（f 是函数引用）的调用**已经能工作**——`Expr::Call { name: Expr::Ident(f) }` 在 `is_closure_call`（codegen.rs:7255）判断 `var_types["f"] == Type::Fn` 后走 CALL_CLOSURE。只要 `let f = double` 正确推断 `Type::Fn`（10.1 第三条），调用端天然兼容。无需 §3.2 预见的额外改动。

### 10.3 测试

- VM file-based：`test/vm/27_function_reference/001_basic_ref`（函数引用赋值+调用）、`002_pass_as_arg`（传参+被调用方调用）、`004_call_unchanged`（回归守护：直接调用不变），全部通过。
- a2r 单测：`test_a2r_function_reference_no_clone`（断言 `apply(handler)` 不含 `.clone()`），通过。
- 端到端验证：`r.route("/", handler)`（axum 模式）a2r 转译输出干净的 `r.route("/", handler);`。

### 10.4 回归

- VM non-ignored：21 FAILED = master 基线 21（零差异，既有失败均为 dstr/ffi 等 str 无关项）。
- a2r file-based（`--features test-trans`）：220 passed / 70 failed → **221 passed / 70 failed**（+1 我新增的测试，failed 完全一致，既有失败均为 cookbook os/file/science 等与函数引用无关）。
- **零新增回归。**

### 10.5 实施与设计的偏差

- §3 原选方案 A（新 FUNC_REF opcode），实施改用方案 B（复用 CLOSURE，capture_count=0）——更小改动，语义等价（零捕获 closure 即函数引用）。
- §3.3 原 a2r 修复设想"在 auto-borrow 决策处识别函数引用跳过 clone"，实际根因更深：`is_copy_type` 漏认 `Type::Fn` 导致函数参数被误判为非 Copy。修 `is_copy_type` 是最根本的一行修复；emit_borrow 的防御是补充。
- §4.1 `003_route_pattern` 未单独建测试（依赖 Router 类型），改由 a2r 端到端验证（§10.3）覆盖 axum 模式。

---

## 参考

- `crates/auto-lang/src/vm/codegen.rs:4949-5083`（`Expr::Ident` 值位置处理链）
- `crates/auto-lang/src/vm/codegen.rs:3996,5049`（`exports.contains_key` 既有查询）
- `crates/auto-lang/src/vm/engine.rs:5796`（CLOSURE opcode）、`5949`（CALL_CLOSURE）
- `crates/auto-lang/src/trans/rust.rs:1416-1440`（a2r `Expr::Ident` emit）、`683`（emit_borrow）
- `crates/auto-lang/src/ast/types.rs:119`（`Type::Fn(params, ret)` 函数类型）
- Rust Reference: [Function item types](https://doc.rust-lang.org/reference/types.html#function-item-types) / [Function pointer types](https://doc.rust-lang.org/reference/types.html#function-pointer-types)
