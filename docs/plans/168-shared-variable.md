# shared 变量 + pub 关键字迁移实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** (1) 实现 `shared` 关键字作为静态存储修饰符，a2r 转译为 Rust `static`；(2) 将 `#[pub]` 注解语法迁移为 `pub` 关键字前缀（与 Rust 一致）。

**状态:** ✅ 已完成 — commit `fb0fbc9e`

**Architecture:** 两个独立但相关的改动。`shared` 新增 Token + AST 变体 + parser + a2r 转译。`pub` 迁移是在 `parse_stmt()` 中统一处理 `pub` 前缀关键字（类似已有的 `pub use`），分发到 `fn`/`type`/`enum`/`spec`/`ext` 各解析路径，同时移除 `#[pub]` 在 `parse_fn_annotations()` 中的处理。AST 层 `is_pub` 字段不变，转译器不变。

**Tech Stack:** Rust, AutoLang lexer/parser/AST/a2r transpiler

---

## Part A: `shared` 变量声明

### Task 1: 添加 `Shared` Token

**Files:**
- Modify: `crates/auto-lang/src/token.rs`

**Step 1: 在 TokenKind 枚举中添加 `Shared`**

在 `token.rs` 第 121 行 `Static` 之后添加：

```rust
    Shared, // ADDED: shared keyword for static storage (Plan 6B-4.19)
```

**Step 2: 在 `keyword_kind()` 中注册 `"shared"`**

在 `token.rs` 的 `keyword_kind()` 函数中（第 369 行 `"static"` 之后）添加：

```rust
            "shared" => Some(TokenKind::Shared),
```

**Step 3: 验证编译**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功

---

### Task 2: 添加 `StoreKind::Shared` AST 变体

**Files:**
- Modify: `crates/auto-lang/src/ast/store.rs`

**Step 1: 在 StoreKind 枚举中添加 `Shared`**

在 `store.rs` 第 9 行 `Const` 之后添加：

```rust
    Shared, // shared = static storage (Plan 6B-4.19)
```

**Step 2: 在所有 Display/AtomWriter/ToNode 实现中添加 `Shared` 分支**

在 5 个 match 块中（Store::fmt, StoreKind::fmt, AtomWriter, ToNode, Atom），每个 `Const` 分支后添加对应的 `Shared` 分支：

```rust
StoreKind::Shared => write!(f, "(shared (name {}) {})", self.name, self.expr),  // Display for Store
StoreKind::Shared => write!(f, "shared"),  // Display for StoreKind
StoreKind::Shared => "shared",  // AtomWriter
StoreKind::Shared => "shared",  // ToNode
```

**Step 3: 验证编译**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功

---

### Task 3: Parser 解析 `shared` 关键字

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:5172` (parse_store_stmt)
- Modify: `crates/auto-lang/src/parser.rs:3095` (parse_stmt match arms)

**Step 1: 在 `parse_store_stmt()` 开头处理 `shared` 前缀**

在 `parser.rs` 的 `parse_store_stmt()` 方法开头（第 5174 行之前），添加 `shared` 检查：

```rust
    pub fn parse_store_stmt(&mut self) -> AutoResult<Stmt> {
        // Plan 6B-4.19: Check for 'shared' modifier
        let is_shared = self.is_kind(TokenKind::Shared);
        if is_shared {
            self.next(); // skip 'shared'
        }

        // store kind: var/let (mut keyword is now aliased to var)
        let mut store_kind = self.store_kind()?;
        self.next(); // skip var/let/mut

        // Plan 6B-4.19: shared var/let → StoreKind::Shared
        if is_shared {
            store_kind = StoreKind::Shared;
        }
```

**Step 2: 在 `parse_stmt()` 的 match 中添加 `Shared` arm**

在 `parser.rs` 第 3112 行 `TokenKind::Const => self.parse_store_stmt()?` 之后添加：

```rust
            TokenKind::Shared => self.parse_store_stmt()?,
```

**Step 3: 验证编译**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功

---

### Task 4: a2r 转译 — `shared` → Rust `static`

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs`

**Step 1: 在全局变量注册中识别 `Shared`**

在 `rust.rs` 约第 3841 行，将：
```rust
                    if matches!(store.kind, StoreKind::Var) {
```
改为：
```rust
                    if matches!(store.kind, StoreKind::Var) || matches!(store.kind, StoreKind::Shared) {
```

**Step 2: 添加 `shared` Store 转译分支**

在 `rust.rs` 约第 2035 行（`const` 转译之前），添加：

```rust
        // Plan 6B-4.19: shared var → static NAME: Lazy<Mutex<T>> = Lazy::new(|| Mutex::new(...));
        if matches!(store.kind, StoreKind::Shared) {
            let static_name = self.global_var_static_name(&store.name);
            let ty = self.rust_type_name(&store.ty);
            write!(out, "static {}: Lazy<Mutex<{}>> = Lazy::new(|| Mutex::new(",
                   static_name, ty)?;
            self.expr(&store.expr, out)?;
            write!(out, "));")?;
            return Ok(());
        }
```

**Step 3: 验证编译**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功

---

### Task 5: 测试 — test 162_shared_var

**Files:**
- Create: `crates/auto-lang/test/a2r/162_shared_var/shared_var.at`
- Create: `crates/auto-lang/test/a2r/162_shared_var/shared_var.expected.rs`
- Modify: `crates/auto-lang/src/tests/a2r_tests.rs`

**Step 1: 创建 `.at` 源文件**

```auto
// Plan 6B-4.19: shared variable declaration

shared var COUNTER int = 0
shared var APP_NAME str = "AutoLang"

fn main() {
    COUNTER += 1
    say(APP_NAME)
    say(COUNTER)
}
```

**Step 2: 创建 `.expected.rs`** — 先运行测试生成 `.wrong.rs`，确认输出正确后重命名为 `.expected.rs`。

**Step 3: 添加测试函数**

在 `a2r_tests.rs` 第 500 行之前添加：

```rust
// Plan 6B-4.19: shared variable declaration (static storage)
#[test]
fn test_162_shared_var() {
    test_a2r("162_shared_var").unwrap();
}
```

**Step 4: 运行测试，根据 `.wrong.rs` 调整 `.expected.rs`**

Run: `rtk cargo test -p auto-lang test_162_shared_var`

**Step 5: 确认测试通过**

---

## Part B: `#[pub]` → `pub` 关键字迁移

> **核心策略**: `pub` 已经作为 ident 存在（`pub use` 已支持）。我们将其提升为统一的前缀关键字，在 `parse_stmt()` 中拦截，分发到 `fn`/`type`/`enum`/`spec` 等解析路径。

### Task 6: `parse_stmt()` 统一 `pub` 前缀分发

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:3075-3093` (parse_stmt)

**Step 1: 扩展现有 `pub` 检查逻辑**

当前 `parse_stmt()` 中（第 3075-3093 行）只处理 `pub use`。扩展为处理所有声明类型：

将现有的 `pub` 检查代码（第 3076-3093 行）替换为：

```rust
    pub fn parse_stmt(&mut self) -> AutoResult<Stmt> {
        // Plan 6B-4.19: pub keyword prefix — unified visibility handling
        if self.cur.text.as_str() == "pub" && self.cur.kind == TokenKind::Ident {
            let saved_cur = self.cur.clone();
            let saved_prev = self.prev.clone();
            self.next(); // consume "pub"

            // Check what follows "pub"
            let stmt = match self.kind() {
                TokenKind::Use => {
                    let mut stmt = self.use_stmt()?;
                    if let Stmt::Use(ref mut u) = stmt {
                        u.is_pub = true;
                    }
                    stmt
                }
                TokenKind::Fn => {
                    self.fn_decl_stmt_with_annotations("", false, false, false, false, true, Vec::new())?
                }
                TokenKind::Static => {
                    // pub static fn ...
                    self.next(); // skip static
                    self.fn_decl_stmt_with_annotations("", false, false, false, true, true, Vec::new())?
                }
                TokenKind::Type => {
                    self.type_decl_stmt_with_annotation(false, true)?
                }
                TokenKind::Enum | TokenKind::Tag => {
                    // Parse the enum normally, then set is_pub
                    let mut stmt = self.enum_stmt()?;
                    if let Stmt::EnumDecl(ref mut e) = stmt {
                        e.is_pub = true;
                    }
                    stmt
                }
                TokenKind::Spec => {
                    // Parse the spec normally, then set is_pub
                    let mut stmt = self.spec_decl_stmt()?;
                    if let Stmt::SpecDecl(ref mut s) = stmt {
                        s.is_pub = true;
                    }
                    stmt
                }
                TokenKind::Ext => {
                    // pub ext — ext itself doesn't carry pub, methods inside may have pub
                    // Just parse ext normally
                    self.ext_stmt()?
                }
                _ => {
                    // Not a recognized pub declaration — put the token back
                    self.lexer.push_token(self.cur.clone());
                    self.cur = saved_cur;
                    self.prev = saved_prev;
                    return self.parse_stmt_inner();
                }
            };
            return Ok(stmt);
        }

        self.parse_stmt_inner()
    }
```

**Step 2: 将当前 `parse_stmt()` 的 match 体重构为 `parse_stmt_inner()`**

将当前 `parse_stmt()` 中第 3095 行 `let stmt = match self.kind() { ... }` 到函数末尾的代码提取到新方法 `parse_stmt_inner()`：

```rust
    fn parse_stmt_inner(&mut self) -> AutoResult<Stmt> {
        let stmt = match self.kind() {
            TokenKind::Break => self.break_stmt()?,
            // ... (保持所有现有 arm 不变)
        };
        Ok(stmt)
    }
```

> **重要**: `parse_stmt_inner()` 保留所有现有的 match arm，包括 `TokenKind::Hash =>`（`#[...]` 注解路径）。`#[pub]` 仍然在注解路径中被解析（向后兼容），但新的 `pub` 关键字路径是首选。

**Step 3: 验证编译**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功

---

### Task 7: 从 `parse_fn_annotations()` 中移除 `"pub"` 处理

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:5442` (parse_fn_annotations)

**Step 1: 移除 `"pub" => has_pub = true` 分支**

在 `parse_fn_annotations()` 函数中（第 5442 行），删除：

```rust
                        "pub" => has_pub = true,
```

> **向后兼容**: `#[pub]` 注解仍然被 `parse_fn_annotations()` 的 `while` 循环消费（`pub` 作为 ident 被跳过），但不再设置 `has_pub = true`。这意味着旧的 `#[pub]` 语法将被静默忽略（不再生效），新的 `pub` 关键字语法是唯一有效的方式。

**Step 2: 验证编译**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功

---

### Task 8: 移除 `enum_stmt()` 和 `spec_decl_stmt()` 中的 `#[pub]` 预解析

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:3573-3580` (enum_stmt)
- Modify: `crates/auto-lang/src/parser.rs:6201-6208` (spec_decl_stmt)

**Step 1: 简化 `enum_stmt()`**

将 `enum_stmt()` 开头（第 3573-3580 行）的 `#[pub]` 预解析删除：

```rust
    fn enum_stmt(&mut self) -> AutoResult<Stmt> {
        // Support both 'enum' and 'tag' keywords (tag is deprecated)
        if self.is_kind(TokenKind::Tag) || self.is_kind(TokenKind::Enum) {
            self.next(); // skip 'enum' or 'tag'
        }
        // ... (rest unchanged)
```

**Step 2: 简化 `spec_decl_stmt()`**

将 `spec_decl_stmt()` 开头（第 6201-6208 行）的 `#[pub]` 预解析删除：

```rust
    pub fn spec_decl_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `spec` keyword
        // ... (rest unchanged)
```

**Step 3: 验证编译**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功

---

### Task 9: 更新测试 — test 149_pub_visibility

**Files:**
- Modify: `crates/auto-lang/test/a2r/149_pub_visibility/pub_visibility.at`
- Modify: `crates/auto-lang/test/a2r/149_pub_visibility/pub_visibility.expected.rs` (可能不需要改)
- Modify: `crates/auto-lang/test/a2r/138_list_as_cast/list_as_cast.at` (如有 `#[pub]`)

**Step 1: 将 `pub_visibility.at` 中的 `#[pub]` 改为 `pub` 前缀**

将：
```auto
#[pub]
type Point { x int, y int, #[pub] fn distance() int { ... } }
#[pub]
fn get_origin() Point { ... }
```

改为：
```auto
pub type Point { x int, pub fn distance() int { ... } }
pub fn get_origin() Point { ... }
```

**Step 2: 更新 `list_as_cast.at` 中如有 `#[pub]`**

将 `#[pub]` 改为 `pub` 前缀。

**Step 3: 运行受影响的测试**

Run: `rtk cargo test -p auto-lang test_149_pub_visibility`
Run: `rtk cargo test -p auto-lang test_138_list_as_cast`

**Step 4: 如果 `.expected.rs` 不匹配，根据实际输出更新**

先运行测试生成 `.wrong.rs`，确认输出正确。

---

### Task 10: 更新 stdlib 中的 `#[pub]`

**Files:**
- Modify: 25+ files in `stdlib/auto/` (全部 `#[pub]` → `pub` 前缀)

**Step 1: 批量替换**

对 `stdlib/auto/` 下所有 `.at` 文件，将 `#[pub]` 替换为 `pub`：

对于独立声明（函数、类型）：
- `#[pub]\nfn foo()` → `pub fn foo()`
- `#[pub]\ntype Foo` → `pub type Foo`
- `#[pub]\nenum Color` → `pub enum Color`
- `#[pub]\nspec Foo` → `pub spec Foo`

对于 type body 内的方法：
- `#[pub]\n    fn foo()` → `    pub fn foo()`
- `#[pub]\n    static fn foo()` → `    pub static fn foo()`

对于组合注解：
- `#[pub, vm]\nfn foo()` → `#[vm]\npub fn foo()`
- `#[pub, c]\nfn foo()` → `#[c]\npub fn foo()`
- `#[c, vm, pub]\nfn foo()` → `#[c, vm]\npub fn foo()`

> **注意**: 替换时需要小心保持缩进和换行。建议逐文件检查。

**Step 2: 验证 stdlib 解析正常**

Run: `rtk cargo build -p auto-lang`
Expected: 编译成功（stdlib 文件在编译时被解析）

---

### Task 11: 更新 CLAUDE.md 文档

**Files:**
- Modify: `d:\autostack\auto-lang\CLAUDE.md` (Function Annotations 部分)

**Step 1: 更新注解语法文档**

将 CLAUDE.md 中关于 `#[pub]` 的描述更新为 `pub` 关键字前缀。关键改动：

1. 删除 `#[pub]` 作为注解的说明
2. 添加 `pub` 关键字前缀的说明
3. 更新示例代码

示例改动：
```
旧: #[pub] fn public_function() int;
新: pub fn public_function() int;

旧: #[pub, vm] fn hybrid_function(data []byte) void;
新: #[vm] pub fn hybrid_function(data []byte) void;
```

---

### Task 12: 全量回归测试

**Files:** None (testing only)

**Step 1: 运行所有 a2r 测试**

Run: `rtk cargo test -p auto-lang -- a2r`
Expected: 所有测试 PASS

**Step 2: 运行所有 a2c 测试**

Run: `rtk cargo test -p auto-lang -- a2c`
Expected: 所有测试 PASS（C transpiler 不受 pub 影响）

**Step 3: 运行全量编译检查**

Run: `rtk cargo check -p auto-lang`
Expected: 编译成功，无 warning

**Step 4: 运行完整测试套件**

Run: `rtk cargo test -p auto-lang`
Expected: 所有测试 PASS

---

### Task 13: 更新 Plan 159 状态

**Files:**
- Modify: `docs/plans/159-autocode-coding-agent.md`

**Step 1: 标记 6B-4.19 完成**

将 6B-4.19 行的状态从 `P2（部分已有 global_vars）` 改为 `✅ **已完成** (test 162) — shared 关键字替代 static`。

**Step 2: 标记 pub 关键字迁移完成**

在计划中添加 pub 关键字迁移的记录。
