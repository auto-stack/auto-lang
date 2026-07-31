# Plan 378：方法调用（Expr::Dot）的 I64/U64 返回类型识别 — 修 `.to_uint()` 等的栈错位

> **For Claude:** `"42".to_uint()` 返回垃圾值（如 `42-2147483647`），根因是 codegen 的 64-bit 检测函数 `contains_u64`/`is_u64_expr` 只识别 `Expr::Ident`（函数名调用），**不识别 `Expr::Dot`（方法调用）**。于是 `s.to_uint()` 被当作 I32（1 slot），而 native 实际返回 I64（2 slot），栈对齐错乱。本计划给这两个函数补 `Expr::Dot` 分支，并配覆盖全部 4 类影响场景的 file-based 测试。

> **Status**: 待实施
> **来源**: auto-shell Plan 034 附录 B Bug 1（2026-07-23 发现，2026-07-31 复核确认根因与行号）
> **影响仓库**: `auto-lang`（`crates/auto-lang/src/vm/codegen.rs`）
> **风险**: 中高 — 触动 codegen 类型推断，影响所有返回 I64/U64 的方法调用在算术/赋值/f-string 中的栈布局

---

## 1. 问题

### 1.1 症状

```auto
let s = "42"
let n = s.to_uint()      // 期望 42,实际 0 或垃圾值
print(n)                 // 输出 "42-2147483647" 之类
print(s.to_uint() + 8)   // 期望 50,实际错误
```

- `"42".to_uint()` 返回 `0-2147483647` 形式的垃圾值。
- `s.to_uint() + 8` 算术结果错误。
- 但 `var x = 0; x = x + 1`（纯 I32）正常 —— 说明只有「方法调用返回 64-bit」这条路径坏了。

### 1.2 根因（2026-07-31 复核，行号已更新）

`crates/auto-lang/src/vm/codegen.rs` 的两个 64-bit 检测函数，在 `Expr::Call` 分支只匹配 `Expr::Ident`，**不匹配 `Expr::Dot`（方法调用）**：

**`is_u64_expr`**（codegen.rs:9281-9291）：
```rust
fn is_u64_expr(&self, expr: &Expr) -> bool {
    match expr {
        Expr::U64(_) => true,
        Expr::Ident(name) => self.var_types.get(name.as_ref())
            .map(|t| matches!(t, Type::U64)).unwrap_or(false),
        _ => false,   // ← Expr::Dot（方法调用）落到这里,返回 false
    }
}
```

**`contains_u64`**（codegen.rs:9310-9330）：
```rust
fn contains_u64(&self, expr: &Expr) -> bool {
    match expr {
        // ...
        Expr::Call(call) => {
            if let Expr::Ident(fn_name) = call.name.as_ref() {   // ← 只处理 Ident
                self.fn_return_types.get(fn_name.as_ref())
                    .map(|t| matches!(t, Type::U64 | Type::I64 | Type::USize | Type::Uint))
                    .unwrap_or(false)
            } else {
                false   // ← Expr::Dot(call.name) 走这里,返回 false
            }
        }
        // ...
    }
}
```

`Expr::Dot(Box<Expr>, Name)` 定义于 `ast.rs:350`（receiver + 方法名）。方法调用 `s.to_uint()` 在 AST 里是 `Expr::Call { name: Expr::Dot(receiver, "to_uint"), .. }`。`call.name` 是 `Expr::Dot`，不匹配 `Expr::Ident` → 返回 false → 上游误判为 I32（1 slot）→ 漏发 64-bit 提升指令 → 栈错位 → 垃圾值。

### 1.3 为什么 `to_uint` 确实返回 I64

| 位置 | 声明 |
|------|------|
| `stdlib/auto/str.at:116` | `fn to_uint(s str) i64` |
| `vm/native_catalog.rs:1320/1371` | `("Str.to_uint", 1523, I64)` / `("str.to_uint", 1523, I64)` |

返回 `i64`（2 slot）。`contains_u64` 的 match 已包含 `I64` —— **所以只要补上 `Expr::Dot` 分支，就能正确命中，无需改 native 注册。**

### 1.4 `fn_return_types` 的查表 key 格式（修复必须对齐）

方法返回类型查表有多套 key（codegen 内部已有 fallback 链，但 `contains_u64`/`is_u64_expr` 根本没进这条链）：

| key 形式 | 示例 | 来源 |
|----------|------|------|
| `{type_name}.{method}` | `str.to_uint` | `enrich_fn_return_types_from_type_store`（L10962）+ native_catalog |
| `{Type}.{method}` | `Str.to_uint` | 同上 TitleCase 版（L10970）+ native_catalog |
| 裸 `{method}` | `to_uint` | 无 parent 时（L10976） |
| `auto.{type}.{method}` | `auto.str.to_uint` | codegen L1689 |

**结论**：新加的 `Expr::Dot` 分支查表时，要依次试这几个 key（最稳：`{type_name}.{method}` → 裸 `{method}`，与 codegen L1637-1642 既有 fallback 一致）。

---

## 2. 影响面分析（修复一处，波及 8 个调用点 / 4 类场景）

`contains_u64` / `is_u64_expr` 的**全部调用点**都在 `codegen.rs` 单文件内（grep 全 crate 无其他文件调用）。修复（给两个函数加 `Expr::Dot` 分支）会同时改变以下 8 处的决策：

| 行号 | 调用 | 影响场景 | 修复后变化 |
|------|------|---------|-----------|
| L1863 | `!contains_u64(&store.expr)` | ① 局部变量声明 `let x u64 = s.to_uint()` | 正确 emit `TYPE_CAST_U64` |
| L5549 | `!contains_u64(rhs)` | ② 赋值 `x = s.to_uint()`（x 是 u64） | 正确 emit 提升 |
| L5896 | `is_u64_operation(lhs,rhs)` | ③ 二元算术选 64-bit 操作码 | `s.to_uint() + 1` 用 `ADD_U64` |
| L5912 | `is_u64_expr(lhs)` | ③ double 提升 lhs | `s.to_uint() + 1.5` 用 `U64_TO_F64` |
| L5917 | `!contains_u64(lhs)` | ③ i32→u64 提升 lhs | 不再误提升 |
| L5928 | `is_u64_expr(rhs)` | ③ double 提升 rhs | 同 L5912 rhs 版 |
| L5933 | `!contains_u64(rhs)` | ③ i32→u64 提升 rhs | 同 L5917 rhs 版 |
| L9360 | `is_u64_operation(lhs,rhs)` | ④ f-string slot 推断 | `f"{s.to_uint()}"` 正确 slot 数 |

**4 类影响场景**（测试必须全覆盖）：
1. **局部变量声明**：`let x u64 = s.to_uint()`
2. **赋值**：`var x = 0; x = s.to_uint()`（x 为 u64）
3. **二元算术**：`s.to_uint() + N`（int 相加）、`s.to_uint() + 1.5`（double 提升）、`s.to_uint() - s2.to_uint()`（两方法相减）
4. **f-string 格式化**：`f"value: {s.to_uint()}"`

---

## 3. 方案

### 3.1 修复 `contains_u64`（codegen.rs:9310）

在 `Expr::Call` 分支后，增加对 `Expr::Dot` 的顶层处理（也作为独立 arm，因为 `Expr::Dot` 可单独出现，如字段访问）：

```rust
fn contains_u64(&self, expr: &Expr) -> bool {
    match expr {
        Expr::U64(_) | Expr::I64(_) => true,
        Expr::Cast { target_type, .. } => matches!(target_type,
            Type::U64 | Type::I64 | Type::USize | Type::Uint),
        Expr::Ident(name) => self.var_types.get(name.as_ref())
            .map(|t| matches!(t, Type::U64 | Type::I64)).unwrap_or(false),
        // ── Plan 378: 方法调用 ──────────────────────────────
        Expr::Dot(receiver, method) => {
            // 先查 fn_return_types(key 格式见 §1.4)
            if self.dot_method_returns_64(receiver, method.as_ref()) {
                return true;
            }
            // receiver 本身可能是 64-bit 表达式(如 (a.to_u64()).field)
            self.contains_u64(receiver.as_ref())
        }
        Expr::Call(call) => {
            // 既有: 函数名调用
            if let Expr::Ident(fn_name) = call.name.as_ref() {
                self.fn_return_types.get(fn_name.as_ref())
                    .map(|t| matches!(t, Type::U64 | Type::I64 | Type::USize | Type::Uint))
                    .unwrap_or(false)
            } else if let Expr::Dot(receiver, method) = call.name.as_ref() {
                // Plan 378: 方法调用形式 Expr::Call{ name: Expr::Dot(..) }
                self.dot_method_returns_64(receiver, method.as_ref())
            } else {
                false
            }
        }
        Expr::Bina(lhs, _, rhs) => self.contains_u64(lhs) || self.contains_u64(rhs),
        Expr::Unary(_, inner) => self.contains_u64(inner),
        _ => false,
    }
}
```

### 3.2 新增 helper `dot_method_returns_64`

抽出方法调用的 64-bit 返回判定，复用 codegen 既有的 fallback key 链（对齐 L1637-1642）：

```rust
/// Plan 378: 判定 `receiver.method()` 是否返回 I64/U64。
/// key fallback 顺序与 codegen L1637-1642 的方法返回类型查表一致。
fn dot_method_returns_64(&self, receiver: &Expr, method: &str) -> bool {
    // 1. 按 receiver 变量名查:{varname}.{method}(如 s.to_uint)
    if let Expr::Ident(var_name) = receiver {
        let k = format!("{}.{}", var_name.as_ref(), method);
        if let Some(t) = self.fn_return_types.get(&k) {
            return matches!(t, Type::U64 | Type::I64 | Type::USize | Type::Uint);
        }
    }
    // 2. 按类型名查:{type}.{method}(str.to_uint)+ TitleCase(Str.to_uint)+ auto 前缀
    let type_name = self.infer_receiver_type_name(receiver); // 复用既有 ObjectType→"str"/"int" 映射
    for prefix in [type_name.as_str(), &type_name_titlecase, &format!("auto.{}", type_name)] {
        let k = format!("{}.{}", prefix, method);
        if let Some(t) = self.fn_return_types.get(&k) {
            return matches!(t, Type::U64 | Type::I64 | Type::USize | Type::Uint);
        }
    }
    // 3. 裸方法名 fallback
    if let Some(t) = self.fn_return_types.get(method) {
        return matches!(t, Type::U64 | Type::I64 | Type::USize | Type::Uint);
    }
    false
}
```

> 注：`infer_receiver_type_name` 复用 codegen 既有的 `infer_object_type`（L6680）+ ObjectType→type_name 映射（L6694），不新写类型推断逻辑。

### 3.3 修复 `is_u64_expr`（codegen.rs:9281）

同样加 `Expr::Dot` 分支（`is_u64_expr` 比 `contains_u64` 更严格——只认 U64，不认 I64，用于选 `U64_TO_F64` vs `I64_TO_F64`）：

```rust
fn is_u64_expr(&self, expr: &Expr) -> bool {
    match expr {
        Expr::U64(_) => true,
        Expr::Ident(name) => self.var_types.get(name.as_ref())
            .map(|t| matches!(t, Type::U64)).unwrap_or(false),
        // ── Plan 378: 方法调用 ──
        Expr::Dot(receiver, method) => {
            if let Expr::Ident(_) = receiver {
                if let Some(t) = self.lookup_dot_method_type(receiver, method.as_ref()) {
                    return matches!(t, Type::U64);
                }
            }
            false
        }
        _ => false,
    }
}
```

（`lookup_dot_method_type` 是 `dot_method_returns_64` 的「返回 Type」版本，供两个函数共用，避免 key fallback 逻辑重复。）

### 3.4 实施顺序

1. 先抽 `lookup_dot_method_type(receiver, method) -> Option<&Type>`（返回查到的类型，供 3.1/3.2/3.3 共用）。
2. `contains_u64` 加 `Expr::Dot` + `Expr::Call{Dot}` 分支。
3. `is_u64_expr` 加 `Expr::Dot` 分支。
4. 跑 §4 全部测试 + 既有回归。

---

## 4. 测试用例（file-based VM tests，全覆盖 4 类场景）

**位置**：`crates/auto-lang/test/vm/25_method_u64/`（新建 category 25；当前最大 category 是 24_generics）。
**约定**：每个用例一个目录 `NNN_name/`，含 `name.at` + `name.expected.out`（print 输出）或 `name.expected.result`（返回值）。
**harness 注册**：在 `crates/auto-lang/src/tests/vm_file_tests.rs` 末尾加对应 `#[test] #[ignore]` 行（见既有 L143+ 模式）。
**跑法**：`cargo test -p auto-lang test_25_method_u64 -- --ignored`

### 4.1 核心：to_uint 基础算术（场景 ③）

**`001_to_uint_basic/to_uint_basic.at`**：
```auto
let s = "42"
let n = s.to_uint()
print(n)
print(s.to_uint() + 8)
print(s.to_uint() - 2)
```
**`to_uint_basic.expected.out`**：
```
42
50
40
```

**`002_to_uint_double_promote/to_uint_double_promote.at`**（场景 ③ double 提升）：
```auto
let s = "10"
print(s.to_uint() + 1.5)
print(s.to_uint() * 2.0)
```
**`to_uint_double_promote.expected.out`**：
```
11.5
20
```

### 4.2 场景 ①：局部变量声明（显式 u64 类型）

**`003_let_u64_from_method/let_u64_from_method.at`**：
```auto
let s = "100"
let x u64 = s.to_uint()
print(x)
print(x + 23)
```
**`let_u64_from_method.expected.out`**：
```
100
123
```

### 4.3 场景 ②：赋值给 u64 变量

**`004_assign_u64/assign_u64.at`**：
```auto
let s = "7"
var x u64 = 0
x = s.to_uint()
print(x)
x = x + s.to_uint()
print(x)
```
**`assign_u64.expected.out`**：
```
7
14
```

### 4.4 场景 ③ 变体：两方法结果相运算

**`005_two_methods_arith/two_methods_arith.at`**：
```auto
let a = "100"
let b = "45"
print(a.to_uint() + b.to_uint())
print(a.to_uint() - b.to_uint())
print(a.to_uint() > b.to_uint())
```
**`two_methods_arith.expected.out`**：
```
145
55
true
```

### 4.5 场景 ④：f-string 格式化（slot 推断）

**`006_fstring_u64/fstring_u64.at`**：
```auto
let s = "2024"
print(f"year: {s.to_uint()}")
print(f"next: {s.to_uint() + 1}")
```
**`fstring_u64.expected.out`**：
```
year: 2024
next: 2025
```

### 4.6 回归守护：其他 I64 方法（确保不只 to_uint）

**`007_other_i64_methods/other_i64_methods.at`**（覆盖 `.len()` 等其他可能受影响的方法；若某些方法返回 I32 则作为「不应被误判为 64-bit」的负向守护）：
```auto
let s = "hello"
print(s.len())
let arr = [1, 2, 3]
print(arr.len())
print(s.len() + 10)
```
**`other_i64_methods.expected.out`**：
```
5
3
15
```
> 注：`.len()` 的实际返回类型（I32 vs I64）需在实施时确认；若为 I32，此用例验证「I32 方法不受修复影响」（回归守护）。若为 I64，则验证「修复覆盖不止 to_uint」。

### 4.7 回归守护：纯 I32 不受影响（负向）

**`008_i32_unaffected/i32_unaffected.at`**（确认修复没把 I32 方法误判为 64-bit）：
```auto
var x = 0
x = x + 1
print(x)
print(2 + 3)
print("abc".find("b"))
```
**`i32_unaffected.expected.out`**：
```
1
5
1
```

### 4.8 codegen 单元测试（直接断言函数返回值）

在 `codegen.rs` 的 `#[cfg(test)] mod tests`（L11396 起）加：

```rust
#[test]
fn contains_u64_recognizes_dot_method_call() {
    // s.to_uint() 应被识别为 64-bit
    let expr = parse_expr(r#"s.to_uint()"#); // 用既有的测试 parse helper
    let cg = TestCodeGen::new_with_fn_return("str.to_uint", Type::I64);
    assert!(cg.contains_u64(&expr));
}

#[test]
fn is_u64_expr_dot_method_u64() {
    // 返回 u64 的方法
    let expr = parse_expr(r#"s.to_u64()"#);
    let cg = TestCodeGen::new_with_fn_return("str.to_u64", Type::U64);
    assert!(cg.is_u64_expr(&expr));
}

#[test]
fn contains_u64_i32_method_not_64bit() {
    // I32 方法不应被误判（负向）
    let expr = parse_expr(r#""abc".find("b")"#);
    let cg = TestCodeGen::new_with_fn_return("str.find", Type::Int);
    assert!(!cg.contains_u64(&expr));
}
```

> 注：`TestCodeGen` 的构造方式需对齐既有 codegen 测试的 fixture（看 L11396+ 既有测试怎么建 Codegen 实例）。

---

## 5. 实施步骤

| 步骤 | 内容 | 验证 |
|------|------|------|
| 1 | 写 §4 全部 file-based 测试用例 + 注册到 `vm_file_tests.rs` | 用例能被发现（`cargo test ... --list` 含 `test_25_*`）；此时应**失败**（bug 未修） |
| 2 | 抽 `lookup_dot_method_type` helper（§3.4） | 编译过 |
| 3 | `contains_u64` 加 `Expr::Dot` + `Expr::Call{Dot}` 分支 | §4.1-4.6 通过 |
| 4 | `is_u64_expr` 加 `Expr::Dot` 分支 | §4.2(double)、§4.5 通过 |
| 5 | 加 §4.8 codegen 单测 | 通过 |
| 6 | 全量回归 `cargo test -p auto-lang` | 既有测试（434+ file-based + 单测）全过 |

**TDD 顺序**：步骤 1 先写测试（红色）→ 步骤 2-4 修复（转绿）→ 步骤 5-6 守护。

---

## 6. 验收标准

1. ✅ `"42".to_uint()` 返回 `42`（非垃圾值）。
2. ✅ `s.to_uint() + 8` == `50`；`s.to_uint() + 1.5` == `11.5`（double 提升）。
3. ✅ `let x u64 = s.to_uint()` + `x = s.to_uint()` 正确。
4. ✅ `f"{s.to_uint()}"` 输出正确（slot 推断）。
5. ✅ §4 全部 file-based 测试 + §4.8 单测通过。
6. ✅ `cargo test -p auto-lang` 既有全量测试零回归。
7. ✅ auto-shell 的 filestats/loccount/csvsum 脚本能产出正确统计（跨仓库验证，见 §7）。

---

## 7. 跨仓库验证（auto-shell Plan 034）

修复合并后，在 auto-shell 仓库验证 example 脚本：
- `examples/filestats/filestats.ash src` → 正确统计扩展名分布（修复前恒为 0）。
- `examples/loccount/loccount.ash src` → 正确统计代码行数。
- `examples/csvsum/csvsum.ash sales.csv region amount` → 正确分组求和。

这解锁 auto-shell Plan 034 的 M2（bash 等价校验）。

---

## 8. 风险与缓解

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| `lookup_dot_method_type` 的 key fallback 与 codegen 既有链不完全一致 → 漏判 | 中 | 中 | §3.2 严格对齐 L1637-1642 的 fallback 顺序；§4.8 单测直接断言 |
| 修复波及 8 个调用点，某场景漏测 → 引入新 bug | 中 | 高 | §4 覆盖全部 4 类场景 + 负向守护（§4.7）；步骤 6 全量回归 |
| 某个既有 native 方法的返回类型注册与实际不符 → 修复后暴露 | 低 | 中 | 步骤 6 既有回归会捕获；如有，作为单独 bug 修 |
| `infer_object_type` 对某些 receiver 推断不准 → key 拼错 | 低 | 低 | fallback 到裸方法名（§3.2 step 3）兜底 |

---

## 9. 非目标

- ❌ 不改 native 注册表（`to_uint` 已正确注册为 I64）。
- ❌ 不改 `infer_object_type` / 类型推断主逻辑（只复用）。
- ❌ 不修 auto-shell 的脚本（脚本逻辑本身正确，是 VM 栈错位导致结果错）。
- ❌ 不处理 `to_int`（返回 I32，不在 64-bit 范畴；§4.7 守护它不受影响）。

---

## 参考

- auto-shell `designs/034-script-examples.md` 附录 B Bug 1（症状 + 原始诊断）
- `crates/auto-lang/src/vm/codegen.rs`:9281（`is_u64_expr`）、9310（`contains_u64`）、1637-1642（方法返回类型查表 fallback）、6680（`infer_object_type`）、10903（`build_fn_return_types`）
- `crates/auto-lang/src/vm/native_catalog.rs`:1320/1371（`to_uint` 注册为 I64）
- `crates/auto-lang/src/tests/vm_file_tests.rs`（file-based 测试 harness）
