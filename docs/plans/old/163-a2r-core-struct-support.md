# Plan 163: a2r 转译器 — 5 项核心结构支持

**日期**: 2026-04-13
**状态**: DONE

## Context

Plan 159 Phase 6B-4 差距分析识别出 10 个严重阻碍。本计划实现其中最容易的 5 个，
合计约 70 行改动，可解锁 ~70% 的 auto-code-rs 代码转译。

## 实施步骤

### Step 1: 关联函数（无 self）— `is_static` 支持

**问题**: `fn_decl()` (rust.rs:2129) 在 `is_method` 时强制输出 `&self`，
不检查 `fn_decl.is_static`。Auto 的 `static fn` 已被 parser 解析并存储在 `Fn.is_static`。

**改动**:

文件: `crates/auto-lang/src/trans/rust.rs`
- line 2129: `if is_method {` → `if is_method && !fn_decl.is_static {`

仅 1 行修改。

**代码示例**:

```auto
// Auto 输入
type Foo {
    value int

    static fn new(v int) Foo {
        return Foo(v)
    }

    fn get_value() int {
        return .value
    }
}
```

```rust
// Rust 输出
impl Foo {
    fn new(v: i32) -> Foo {       // ✅ 无 &self（static）
        return Foo { value: v };
    }
    fn get_value(&self) -> i32 {  // ✅ 有 &self（实例方法）
        return self.value;
    }
}
```

**测试**: `test/a2r/148_static_fn/static_fn.at`

---

### Step 2: `pub` 可见性

**问题**: Parser 已解析 `#[pub]`（`has_pub`），但所有调用点用 `_has_pub` 忽略。
AST 节点（Fn、TypeDecl、EnumDecl、SpecDecl）没有 `is_pub` 字段。

**改动**:

1. `crates/auto-lang/src/ast/fun.rs` — `Fn` struct 加 `pub is_pub: bool`
2. `crates/auto-lang/src/ast/types.rs` — `TypeDecl` struct 加 `pub is_pub: bool`
3. `crates/auto-lang/src/ast/enums.rs` — `EnumDecl` struct 加 `pub is_pub: bool`
4. `crates/auto-lang/src/ast/spec.rs` — `SpecDecl` struct 加 `pub is_pub: bool`
5. `crates/auto-lang/src/parser.rs` — `_has_pub` → `has_pub`，传递到 AST
6. `crates/auto-lang/src/trans/rust.rs` — `fn_decl()`、`type_decl()`、`enum_decl()`、`spec_decl()` 输出 `pub ` 前缀

**代码示例**:

```auto
// Auto 输入
#[pub]
type Point {
    x int
    y int

    #[pub]
    fn distance() int {
        return .x * .x + .y * .y
    }
}

#[pub]
fn get_origin() Point {
    return Point(0, 0)
}
```

```rust
// Rust 输出
pub struct Point {                // ✅ pub struct
    x: i32,
    y: i32,
}

impl Point {
    pub fn distance(&self) -> i32 { // ✅ pub fn
        return self.x * self.x + self.y * self.y;
    }
}

pub fn get_origin() -> Point {     // ✅ pub fn
    return Point { x: 0, y: 0 };
}
```

**测试**: `test/a2r/149_pub_visibility/pub_visibility.at`

---

### Step 3: `#[tokio::main]` + async main

**问题**: 当前 `fn main()` 硬编码为同步函数。
如果 main 函数体中有 await 调用，需要生成 `#[tokio::main]` 和 `async fn main()`。

**方案**: 检测 main 中的语句是否包含 `Expr::Await`。

**改动**:

文件: `crates/auto-lang/src/trans/rust.rs`
- 新增 `fn has_await(stmts: &[Stmt]) -> bool` 和 `fn expr_has_await(expr: &Expr) -> bool`
- `fn_decl()` 中检测用户定义的 `fn main()` body 是否含 await
- `Trans::trans()` Phase 4: top-level stmts 的 main 也检测 await

**代码示例**:

```auto
// Auto 输入
fn main() {
    let result = fetch_data().await
    print(result)
}
```

```rust
// Rust 输出
#[tokio::main]                    // ✅ 自动检测 .await 并添加
async fn main() {                 // ✅ async fn
    let result = fetch_data().await;
    println!("{}", result);
}
```

**测试**: `test/a2r/150_tokio_main/tokio_main.at`

---

### Step 4: `&mut self` 方法

**问题**: `fn_decl()` 硬编码 `&self`。需要支持 `&mut self`。

**Auto 语法设计**: 使用 `mut fn method()` 表示可变方法（与 `static fn` 对称）。

**改动**:

1. `crates/auto-lang/src/ast/fun.rs` — `Fn` struct 加 `pub is_mut: bool`
2. `crates/auto-lang/src/parser.rs` — 检测 `TokenKind::Mut` 在 `TokenKind::Fn` 之前
3. `crates/auto-lang/src/trans/rust.rs` — `fn_decl()` 输出 `&mut self`

**代码示例**:

```auto
// Auto 输入
type Counter {
    count int

    fn new() Counter {
        return Counter(0)
    }

    mut fn increment() void {    // ✅ mut 关键字
        .count = .count + 1
    }
}
```

```rust
// Rust 输出
impl Counter {
    fn new(&self) -> Counter {
        return Counter { count: 0 };
    }
    fn increment(&mut self) {    // ✅ &mut self
        self.count() = self.count() + 1;
    }
}
```

**测试**: `test/a2r/151_mut_self/mut_self.at`

---

### Step 5: per-field serde 属性

**问题**: `TypeDecl.attrs` 已支持 type 级属性透传。`Member` struct 没有属性字段。

**改动**:

1. `crates/auto-lang/src/ast/types.rs` — `Member` struct 加 `pub attrs: Vec<AutoStr>`
2. `crates/auto-lang/src/parser.rs` — body loop 中从 `raw_attrs` 收集成员属性
3. `crates/auto-lang/src/trans/rust.rs` — `type_decl()` 在字段前输出 `#[...]`

**代码示例**:

```auto
// Auto 输入
type Role {
    #[serde(rename = "role_id")]
    id int
    #[serde(rename = "role_name")]
    name str
}
```

```rust
// Rust 输出
struct Role {
    #[serde(rename = "role_id")]    // ✅ 属性透传到字段
    id: i32,
    #[serde(rename = "role_name")]  // ✅ 引号和空格正确保留
    name: &str,
}
```

**测试**: `test/a2r/152_field_attrs/field_attrs.at`

---

## 额外修复（实施中发现）

### 注解解析器字符串引号丢失

`parse_fn_annotations()` 收集 `#[serde(rename = "role_id")]` 时，lexer 已剥离字符串引号，
导致输出 `#[serde(rename=role_id)]`。修复：添加 `TokenKind::Str` 分支恢复引号。

### `=` 号无空格

`#[serde(rename = "role_id")]` 中的 `=` 被拼接为 `rename=role_id`。
修复：添加 `TokenKind::Asn` 分支输出 ` = `。

### `is_mut` 未传递到 `fn_decl_stmt_with_annotations`

`type_decl_stmt_with_annotation()` 中解析 `mut fn` 后，`is_mut` 变量未设置到 Fn 上。
修复：在返回后手动设置 `fn_expr.is_mut = true`。

### `#[tokio::main]` 仅检测 top-level main

原始实现只在 `Trans::trans()` Phase 4 检测 top-level `main` vec 中的 await。
用户定义的 `fn main() { ... }` 走 `fn_decl()` 路径，不会被检测。
修复：在 `fn_decl()` 中也检测 `fn main()` body 是否含 await。

### test 138 expected output 更新

`138_list_as_cast` 的 `.at` 源文件有 `#[pub]`，但 `.expected.rs` 是在 pub 功能
实现之前创建的。更新 expected output 为正确的 `pub fn get_value`。

---

## 测试

新增 5 个 a2r 测试：

| 测试 | 覆盖特性 | 关键验证点 |
|------|---------|-----------|
| `148_static_fn` | Step 1 | `static fn new() -> Self` → 不带 `&self` |
| `149_pub_visibility` | Step 2 | `#[pub]` → `pub struct`, `pub fn` |
| `150_tokio_main` | Step 3 | main 中有 `.await` → `#[tokio::main] async fn main()` |
| `151_mut_self` | Step 4 | `mut fn push(...)` → `fn push(&mut self, ...)` |
| `152_field_attrs` | Step 5 | `#[serde(rename = "role")]` → 透传到字段 |

## 验证

```bash
rtk cargo test -p auto-lang test_148_static_fn
rtk cargo test -p auto-lang test_149_pub_visibility
rtk cargo test -p auto-lang test_150_tokio_main
rtk cargo test -p auto-lang test_151_mut_self
rtk cargo test -p auto-lang test_152_field_attrs
# 回归测试
rtk cargo test -p auto-lang -- a2r
```

**结果**: 74 个 a2r 测试全部通过，2778 个总测试 0 失败。
