# VM Enum & Ext Codegen 实现计划

## Status: ✅ COMPLETE

Verified 2026-04-23: All features implemented.
- `Stmt::EnumDecl` codegen registers enum variants in `enum_values` and `generic_registry`
- `Stmt::Ext` codegen compiles ext methods with `TypeName.method` mangling
- `IS_VARIANT` opcode (0xB9) for `is` match on enum variants in `engine.rs`
- Scalar and data-carrying enum variants both supported

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 使 `05_permission_check/main.at` 能在 AutoVM 中正确解析、编译和运行。

**Architecture:** 分 5 个阶段逐步实现：先让 `enum` 替代 `type` 用于枚举声明（文件需修改），再实现 ext 方法的 VM codegen，然后让 `is` 匹配枚举变体，接着添加数据变体支持，最后完善 assert_eq 对对象/枚举的比较。

**Tech Stack:** Rust, AutoVM bytecode, miette diagnostics

---

## 背景分析

`05_permission_check/main.at` 涉及 5 个未实现的语言特性：

| # | 特性 | 当前状态 |
|---|------|----------|
| A | `type Name { Variant }` 当枚举用 | 解析失败 — `type` 只支持 `name Type` 字段 |
| B | `ext Type { fn method(...) ... }` 编译到 VM | 解析 OK，codegen 无 `Stmt::Ext` 分支 |
| C | `is expr { Type.Variant -> ... }` 枚举模式匹配 | 仅支持 integer EQ，不支持带 tag 的变体匹配 |
| D | 数据变体 `Deny { reason str }` 构造 | 解析/AST/codegen 全部不支持 |
| E | `assert_eq` 复杂类型比较 | VM 只支持 i32 和 string 比较 |

**策略：** 先修改 `.at` 文件使用正确的 Auto 语法（`enum` 替代 `type`），然后逐步实现 VM 支持。因为 scalar enum（`PermissionMode`）已能工作，可以分阶段验证。

---

## Task 1: 修改测试文件使用 `enum` 语法

**问题：** `05_permission_check/main.at` 使用 `type` 关键字声明枚举，但 Auto 的 `type` 只支持 struct 字段声明。正确语法是 `enum`。

**Files:**
- Modify: `d:\autostack\auto-code-rs\crates\ac-examples\src\05_permission_check\main.at`

**Step 1: 将 `type PermissionMode` 和 `type PermissionDecision` 改为 `enum`**

将 `type PermissionMode { Allow, Ask, ReadOnly }` 改为 `enum PermissionMode { Allow, Ask, ReadOnly }`。
将 `type PermissionDecision { Allow, Deny { reason str } }` 暂时简化为 `enum PermissionDecision { Allow, Deny }`（数据变体后阶段实现）。

**Step 2: 暂时移除依赖 Deny payload 的 assert_eq 调用**

`assert_eq(ro.check("Bash", false), PermissionDecision.Deny { reason: "..." })` 暂时注释掉，后续 Task 恢复。

**Step 3: 确认解析不再报错**

Run: `auto d:\autostack\auto-code-rs\crates\ac-examples\src\05_permission_check\main.at`
Expected: 解析成功（可能运行时有其他错误，但不报 `Expected term, got RBrace`）

**Step 4: Commit**

```bash
git add d:/autostack/auto-code-rs/crates/ac-examples/src/05_permission_check/main.at
git commit -m "fix(examples): use enum keyword for PermissionMode in 05_permission_check"
```

---

## Task 2: 实现 `Stmt::Ext` 的 VM codegen

**问题：** `ext PermissionPolicy { fn new(...) ... fn check(...) ... }` 被 parser 解析但 codegen 的 `compile_stmt()` 没有 `Stmt::Ext` 匹配分支，静默跳过。

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs`（约 line 1374，`Stmt::TypeDecl` 之后）

**Step 1: 写失败的测试**

创建 `crates/auto-lang/test/vm/10_types/010_ext_method/ext_method.at`:

```auto
type Point { x int }

ext Point {
    fn new(x int) Point {
        Point { x: x }
    }

    fn get_x(self) int {
        self.x
    }
}

let p = Point.new(42)
print(p.get_x())
```

创建 `crates/auto-lang/test/vm/10_types/010_ext_method/ext_method.expected.out`:

```
42
```

在 `crates/auto-lang/src/tests/vm_file_tests.rs` 的 10_types section 添加：

```rust
#[test] fn test_10_types_010_ext_method() { test_vm("10_types/010_ext_method").unwrap(); }
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p auto-lang --lib test_10_types_010_ext_method`
Expected: FAIL — ext 方法未被编译，`Point.new` 调用失败

**Step 3: 实现 ext codegen**

在 `codegen.rs` 的 `compile_stmt()` 中，在 `Stmt::TypeDecl(...)` 之后（约 line 1374），添加 `Stmt::Ext(ext_block)` 分支：

```rust
Stmt::Ext(ext_block) => {
    // Compile ext methods identically to TypeDecl methods
    let type_name = ext_block.target.to_string();
    for method in &ext_block.methods {
        let mangled_name = format!("{}.{}", type_name, method.name);
        let mut method_fn = method.clone();
        method_fn.name = crate::ast::Name::from(mangled_name.as_str());
        method_fn.parent = Some(crate::ast::Name::from(type_name.as_str()));

        if !method.is_static {
            let has_self = method_fn.params.first().map(|p| p.name.to_string() == "self").unwrap_or(false);
            if !has_self {
                method_fn.params.insert(0, crate::ast::Param {
                    name: crate::ast::Name::from("self"),
                    ty: Type::Unknown,
                    default: None,
                    mode: crate::ast::ParamMode::View,
                });
            }
        }

        self.compile_stmt(&Stmt::Fn(method_fn))?;
    }
}
```

注意：逻辑与 `Stmt::TypeDecl` 中的方法编译完全一致（复用 same pattern）。

**Step 4: 运行测试确认通过**

Run: `cargo test -p auto-lang --lib test_10_types_010_ext_method`
Expected: PASS

**Step 5: 运行全量测试确认无回归**

Run: `cargo test -p auto-lang --lib`
Expected: 全部通过

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs crates/auto-lang/test/vm/10_types/010_ext_method/ crates/auto-lang/src/tests/vm_file_tests.rs
git commit -m "feat(vm): compile Stmt::Ext methods to VM bytecode"
```

---

## Task 3: 实现 `enum` 类型注册（让 `is` 能匹配枚举变体值）

**问题：** `Stmt::EnumDecl` 在 codegen 中被跳过（不做任何事），枚举变体值通过 TypeStore 的 `get_enum_variant_value` 注册。但 `is` 匹配需要枚举变体在编译期可用。需要确认 enum 注册流程是否完整。

**Files:**
- 可能修改: `crates/auto-lang/src/vm/codegen.rs`（`Stmt::EnumDecl` 分支）
- 创建: `crates/auto-lang/test/vm/10_types/011_enum_is_match/`

**Step 1: 写失败的测试**

创建 `crates/auto-lang/test/vm/10_types/011_enum_is_match/enum_is_match.at`:

```auto
enum Color { Red = 1, Green = 2, Blue = 3 }

fn check(c int) str {
    is c {
        1 -> "red"
        2 -> "green"
        3 -> "blue"
        else -> "unknown"
    }
}

print(check(1))
print(check(3))
```

创建 `enum_is_match.expected.out`:

```
red
blue
```

添加测试函数到 `vm_file_tests.rs`。

**Step 2: 运行确认是否已能工作**

Run: `cargo test -p auto-lang --lib test_10_types_011_enum_is_match`
如果 PASS — scalar enum 的 `is` 匹配已经工作（通过 integer EQ）。进入 Step 6。
如果 FAIL — 需要修复。

**Step 3: 写使用 `Color.Red` 的变体测试**

创建 `crates/auto-lang/test/vm/10_types/012_enum_dot_match/enum_dot_match.at`:

```auto
enum Color { Red = 1, Green = 2, Blue = 3 }

fn check(c int) str {
    is c {
        Color.Red -> "red"
        Color.Green -> "green"
        Color.Blue -> "blue"
        else -> "unknown"
    }
}

print(check(Color.Red))
print(check(Color.Blue))
```

Expected out: `red\nblue`

**Step 4: 如果 `Color.Red` 在 is branch 中不被编译为 CONST_I32，修复 codegen**

在 `is` 分支的 codegen 中，`Cover::Tag` 模式需要正确编译 `PermissionMode.Allow` 为其整数值。检查 `is` codegen 中 tag cover 分支是否调用了和普通 `Dot` 表达式相同的枚举值查找逻辑。

**Step 5: 运行全量测试**

Run: `cargo test -p auto-lang --lib`
Expected: 全部通过

**Step 6: Commit**

```bash
git add crates/auto-lang/test/vm/10_types/011_enum_is_match/ crates/auto-lang/test/vm/10_types/012_enum_dot_match/ crates/auto-lang/src/vm/codegen.rs
git commit -m "test(vm): add enum is-match tests and fix tag cover in is branches"
```

---

## Task 4: 添加 `PermissionMode` 类型的 scalar enum 运行测试

**目的：** 验证 Task 1-3 的组合：scalar enum + ext 方法 + is 匹配。

**Files:**
- 创建: `crates/auto-lang/test/vm/14_permission/001_scalar_mode/scalar_mode.at`

**Step 1: 写测试**

创建 `scalar_mode.at`（简化版 permission_check，只用 scalar enum）:

```auto
enum PermissionMode { Allow = 1, Ask = 2, ReadOnly = 3 }

type PermissionPolicy {
    mode int
}

ext PermissionPolicy {
    fn new(mode int) PermissionPolicy {
        PermissionPolicy { mode: mode }
    }

    fn check(self, tool_name str, is_read_only bool) int {
        is self.mode {
            1 -> 1
            3 -> {
                if is_read_only { 1 } else { 0 }
            }
            2 -> 1
            else -> 0
        }
    }
}

let allow = PermissionPolicy.new(1)
assert_eq(allow.check("Bash", false), 1)

let ro = PermissionPolicy.new(3)
assert_eq(ro.check("Read", true), 1)
assert_eq(ro.check("Bash", false), 0)

print("permission scalar mode: all assertions passed")
```

Expected out: `permission scalar mode: all assertions passed`

**Step 2: 运行确认**

Run: `cargo test -p auto-lang --lib test_14_permission`
Expected: PASS

**Step 3: Commit**

```bash
git add crates/auto-lang/test/vm/14_permission/
git commit -m "test(vm): add scalar permission mode integration test"
```

---

## Task 5: 实现数据变体（Enum with struct payload）

**问题：** `Deny { reason str }` 这种内联 struct 变体不被支持。需要：
1. Parser 支持 `enum Name { Variant { field Type, ... } }`
2. AST 扩展 `EnumItem` 支持 fields
3. Codegen 生成构造代码（创建带 type tag 的 object）
4. 运行时支持匹配数据变体

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`（`parse_enum_body`，约 line 3834）
- Modify: `crates/auto-lang/src/ast/enums.rs`（`EnumItem` struct，约 line 53）
- Modify: `crates/auto-lang/src/vm/codegen.rs`（枚举变体构造、is 匹配）
- Modify: `crates/auto-lang/src/vm/engine.rs`（可能需要新 opcode）

**Step 1: 扩展 AST — 给 EnumItem 添加 fields**

在 `crates/auto-lang/src/ast/enums.rs` line 53 的 `EnumItem` 中添加：

```rust
pub struct EnumItem {
    pub name: AutoStr,
    pub scalar_value: Option<i32>,
    pub payload_type: Option<Type>,
    pub fields: Vec<crate::ast::Member>,  // NEW: inline struct fields for data variants
}
```

所有构造 `EnumItem` 的地方需要添加 `fields: vec![]` 默认值。

**Step 2: 扩展 Parser — 解析 `{ field Type }` 变体**

在 `parse_enum_body()` 中（约 line 3849），当看到 `LBrace` 时，检查是否为 inline struct。如果变体名后紧跟 `{` 且下一个 token 是 identifier（不是 `}`），则解析为 inline struct fields：

```rust
// After reading variant name, before parse_type():
if self.is_kind(TokenKind::LBrace) {
    // Check if this is inline struct fields or a block
    // Peek ahead: if next is Ident, it's struct fields
    let saved = self.cur.clone();
    self.next(); // skip {
    if self.is_kind(TokenKind::Ident) || self.is_kind(TokenKind::RBrace) {
        // Inline struct: { field1 Type, field2 Type, ... }
        let mut fields = Vec::new();
        while !self.is_kind(TokenKind::RBrace) {
            let member = self.type_member()?;
            fields.push(member);
            if self.is_kind(TokenKind::Comma) { self.next(); }
        }
        self.expect(TokenKind::RBrace)?;
        return Ok(EnumItem { name, scalar_value: None, payload_type: None, fields });
    } else {
        // Not struct fields, restore
        self.cur = saved;
    }
}
```

**Step 3: 写失败的测试**

创建 `crates/auto-lang/test/vm/10_types/013_data_variant/data_variant.at`:

```auto
enum Decision {
    Allow = 1
    Deny = 2
}

fn make_deny(reason str) int {
    2
}

let d = make_deny("blocked")
assert_eq(d, 2)
print("data variant: ok")
```

Expected: `data variant: ok`（先用 scalar 值验证基础流程）

**Step 4: 运行确认基础 enum 工作正常后，扩展测试**

创建 `014_data_variant_struct/data_variant_struct.at`:

```auto
enum Decision {
    Allow
    Deny
}

let d = Decision.Deny
print(f"deny=${d}")
```

Expected: `deny=2`（或类似，取决于自动赋值）

**Step 5: 扩展 codegen — 数据变体构造**

当 `Type.Variant { field: value }` 出现时：
1. 创建一个 VM Object，包含 `__tag` 字段（变体名）和各 payload 字段
2. 将 object_id push 到栈上

当 `is expr { Type.Variant -> ... }` 匹配时：
1. 检查 object 的 `__tag` 字段
2. 如果匹配，解构 payload 字段到局部变量

这需要新 opcode 或复用现有 object 机制。具体实现根据探索结果确定。

**Step 6: 运行全量测试确认无回归**

Run: `cargo test -p auto-lang --lib`
Expected: 全部通过

**Step 7: Commit**

```bash
git add crates/auto-lang/src/ast/enums.rs crates/auto-lang/src/parser.rs crates/auto-lang/src/vm/codegen.rs crates/auto-lang/test/vm/10_types/013_data_variant/ crates/auto-lang/test/vm/10_types/014_data_variant_struct/
git commit -m "feat(vm): support enum data variants with inline struct fields"
```

---

## Task 6: 扩展 assert_eq 支持对象比较

**问题：** VM 的 `shim_assert_eq` 只比较 i32 和 string。当比较两个 object（如 `PermissionDecision` 实例）时需要结构化比较。

**Files:**
- Modify: `crates/auto-lang/src/vm/native.rs`（`shim_assert_eq`，约 line 721）

**Step 1: 写失败的测试**

创建 `crates/auto-lang/test/vm/15_assert/001_assert_eq_obj/assert_eq_obj.at`:

```auto
type Pair { x int }

let a = Pair { x: 42 }
let b = Pair { x: 42 }
assert_eq(a, b)
print("assert_eq objects: ok")
```

Expected: `assert_eq objects: ok`

**Step 2: 运行确认失败**

Run: `cargo test -p auto-lang --lib test_15_assert`
Expected: FAIL — object IDs 不同

**Step 3: 实现 object 结构化比较**

在 `shim_assert_eq` 中，当两个值都是正整数时（可能是 object ID），尝试比较 heap object 的内容：

```rust
// After integer comparison fails:
let left_handle = left as u64;
let right_handle = right as u64;
if let (Some(left_obj), Some(right_obj)) = (vm.get_heap_object(left_handle), vm.get_heap_object(right_handle)) {
    let left_guard = left_obj.read().unwrap();
    let right_guard = right_obj.read().unwrap();
    // Compare as AutoObj (field-by-field)
    if let (Some(left_auto), Some(right_auto)) = (left_guard.as_any().downcast_ref::<AutoObj>(), right_guard.as_any().downcast_ref::<AutoObj>()) {
        if left_auto.len() == right_auto.len() {
            let equal = left_auto.iter().zip(right_auto.iter()).all(|((k1, v1), (k2, v2))| {
                k1 == k2 && values_equal(v1, v2)
            });
            if equal { /* success */ return Ok(()); }
        }
    }
}
```

注意：需要 `values_equal` 递归比较函数，处理 Int/String/Bool/Nested objects。

**Step 4: 运行确认通过**

Run: `cargo test -p auto-lang --lib test_15_assert`
Expected: PASS

**Step 5: 运行全量测试**

Run: `cargo test -p auto-lang --lib`
Expected: 全部通过

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/native.rs crates/auto-lang/test/vm/15_assert/
git commit -m "feat(vm): assert_eq supports structural object comparison"
```

---

## Task 7: 最终集成 — 运行 05_permission_check

**目的：** 恢复 `05_permission_check/main.at` 中被注释的内容，运行完整测试。

**Files:**
- Modify: `d:\autostack\auto-code-rs\crates\ac-examples\src\05_permission_check\main.at`

**Step 1: 恢复 Deny 数据变体**

将 `enum PermissionDecision { Allow, Deny }` 改回 `enum PermissionDecision { Allow, Deny { reason str } }`。

**Step 2: 恢复 assert_eq 调用**

取消注释 `assert_eq(ro.check("Bash", false), PermissionDecision.Deny { reason: "..." })`。

**Step 3: 运行完整脚本**

Run: `auto d:\autostack\auto-code-rs\crates\ac-examples\src\05_permission_check\main.at`
Expected: `05_permission_check: all assertions passed`

**Step 4: Commit**

```bash
git add d:/autostack/auto-code-rs/crates/ac-examples/src/05_permission_check/main.at
git commit -m "feat(examples): restore full 05_permission_check with data variants"
```

---

## 依赖关系

```
Task 1 (修改 .at 文件)
  ↓
Task 2 (ext codegen) ← 独立，但 Task 4 依赖它
  ↓
Task 3 (enum is-match) ← 可能已工作
  ↓
Task 4 (集成测试) ← 依赖 Task 1+2+3
  ↓
Task 5 (数据变体) ← 独立但复杂
  ↓
Task 6 (assert_eq 对象比较) ← Task 5 之后
  ↓
Task 7 (最终集成) ← 依赖所有前置
```

Task 2 是最高优先级（解锁所有 ext 方法）。Task 5 最复杂（涉及 parser+AST+codegen+engine 全面修改）。
