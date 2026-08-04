# Plan 223: a2r 转译器关键缺陷修复

## Status: ✅ COMPLETE

> 基于 step-00-api-minimal（545 行 Auto 代码 → Rust）的翻译对比分析。
> 发现 3 类关键问题：lexer 偏移量漂移导致大文件崩溃、parser 不支持多参数 enum 变体和 `is` 语句表达式、a2r 运行时映射缺失。
> 本计划修复这些问题，使 a2r 能翻译 step-00 级别的实际项目。

## 2026-04-24 验证结果（最终确认）

用隔离测试用例逐一验证所有 5 个问题：

| # | 问题 | 验证结果 | 状态 |
|---|------|---------|------|
| A | Lexer `pos` 漂移 >3KB 崩溃 | ✅ 已修复（8KB 文件含 200 个转义字符串，201 fragments 成功翻译） | 已解决 |
| B | 多参数 enum 变体 | ✅ 已修复（`ToolUse str str str` → `ToolUse(&str, &str, &str)`，模式匹配解构正确） | 已解决 |
| C | `is` 无法作为表达式赋值 | ✅ 已修复（`let result = is x { ... }` 正确翻译为 `match`） | 已解决 |
| D | `is` 单行分支不支持 `return` | ✅ 已修复（`None -> return None` 正确翻译为 `None => return None`） | 已解决 |
| E | 运行时映射 | ✅ 已修复（`env.get`/`fs.read_to_string`/`sleep_ms` 在 a2r_std.rs，`http_post` 在 Plan 195 中实现） | 已解决 |

**结论**：Plan 223 全部 5 个问题均已修复。

## 与 Plan 204 的关系

Plan 204 解决 a2r 转译器的基础完整性（assert!、type→struct、enum→enum 等）。
本计划聚焦 Plan 204 未覆盖的 3 个阻断性问题，且提供了精确的根因和修复方案。

## 问题分类

| # | 问题 | 严重程度 | 影响 | 修复难度 |
|---|------|---------|------|---------|
| A | Lexer `pos` 漂移导致 >3KB 文件崩溃 | 🔴 阻断 | 所有含转义字符串的文件 | 低 |
| B | Parser 不支持多参数 enum 变体 | 🔴 阻断 | tagged union 核心语法 | 中 |
| C | `is` 语句无法作为表达式赋值 | 🟡 限制 | `let x = is y { ... }` | 中 |
| D | `is` 单行分支不支持 `return`/`break` | 🟡 限制 | 简洁模式匹配 | 低 |
| E | a2r 运行时映射缺失 | 🟠 功能 | HTTP/env/fs 等外部调用 | 中 |

---

## Phase 1: 修复 Lexer `pos` 漂移（问题 A）

### 根因

`lexer.rs` 的 `pos()` 方法用 **解码后的 token 文本长度** 推进 `self.pos`，
而非用 **源码中实际消耗的字节数**。

对于含转义序列的字符串，如 `"hello\nworld"`：
- 源码消耗 14 字节：`"hello\nworld"`
- 解码后文本 11 字符：`hello` + 换行 + `world`
- `self.pos` 只推进 11，产生 3 字节累积漂移

每个 `\n` 漂移 2 字节，`\"` 漂移 1 字节，`\\` 漂移 1 字节。
当文件 >3KB 且含多个转义序列时，漂移导致 `SourceSpan` 越界 → miette panic。

### 修复方案

**文件**: `crates/auto-lang/src/lexer.rs`

**核心思路**: 在 token 消费前后记录 `self.pos`，用差值作为实际长度。

```rust
// 修改前 (line 36-46):
pub fn pos(&mut self, len: usize) -> Pos {
    let p = Pos { line: self.line, at: self.at, pos: self.pos, len };
    self.pos += len;
    self.at += len;
    p
}

// 修改后:
pub fn pos(&mut self, len: usize) -> Pos {
    let p = Pos { line: self.line, at: self.at, pos: self.pos, len };
    self.pos += len;
    self.at += len;
    p
}

// 新增：基于源码字节偏移量的 pos 方法
pub fn pos_from_source(&mut self, source_start: usize) -> Pos {
    let consumed = self.source_pos - source_start; // 实际消耗的字节数
    let p = Pos { line: self.line, at: self.at, pos: source_start, len: consumed };
    self.pos = self.source_pos;
    self.at += consumed;
    p
}
```

**需要修改的 tokenizer 方法**（均使用 `self.pos(text.len())` → 改为记录源码偏移量）：

| 方法 | 行号 | 问题 |
|------|------|------|
| `str()` | 284 | `self.pos(text.len())` 转义序列缩短文本 |
| `multi_str()` | 367 | 同上 + `"""` 开头 3 字节未计入 |
| `char()` | 184-251 | `'\n'` 消耗 4 字节但 pos 推进 1 |
| `fstr()` | 2834+ | f-string 中的 `${}` 转义 |

**具体修复（以 `str()` 为例）**:

```rust
pub fn str(&mut self) -> Token {
    let mut text = String::new();
    let start_pos = self.pos; // 记录源码起始位置
    self.chars.next(); // skip opening "
    while let Some(&c) = self.chars.peek() {
        if c == '"' {
            self.chars.next();
            break;
        }
        if c == '\\' {
            self.chars.next();
            if let Some(&esc) = self.chars.peek() {
                match esc {
                    'n' => text.push('\n'),
                    't' => text.push('\t'),
                    'r' => text.push('\r'),
                    '0' => text.push('\0'),
                    '\\' => text.push('\\'),
                    '"' => text.push('"'),
                    _ => { text.push('\\'); text.push(esc); }
                }
                self.chars.next();
                continue;
            }
        }
        text.push(c);
        self.chars.next();
    }
    // 用源码偏移量差值而非文本长度
    let consumed = self.pos - start_pos + 1; // +1 for closing "
    Token::str(self.pos_from_source(start_pos), text.into())
}
```

**或者更简单的方案**: 在 `chars` 迭代器上维护一个独立的 `source_byte_pos` 计数器，
每次 `chars.next()` 时增加 `c.len_utf8()`，然后在 `pos()` 中使用这个值。

### 验证

```bash
# 修复前：崩溃
cargo run --bin auto -- rust test_3kb_file.at  # panic: range start index out of range

# 修复后：正常解析或给出正确的错误信息
cargo run --bin auto -- rust test_3kb_file.at  # 成功翻译或显示语法错误位置
```

### 预估工作量

- 代码修改：~50 行
- 测试：需添加 >3KB 的 .at 测试文件（含转义字符串）
- 风险：低（仅修改 pos 计算，不影响 token 内容）

---

## Phase 2: 支持多参数 Enum 变体（问题 B）

### 根因

`parser.rs:3998-4003` 的 `parse_enum_body()` 在解析变体时，只调用一次 `parse_type()`：

```rust
} else if self.is_kind(TokenKind::Ident) || self.is_kind(TokenKind::LParen) {
    payload_type = Some(self.parse_type()?);
    has_any_payload = true;
}
```

对于 `ToolUse str str str`，只消费第一个 `str`，剩余的 `str str` 被误认为后续变体名，
最终在 `}` 处产生 "Expected term, got RBrace" 错误。

### 修复方案

**文件**: `crates/auto-lang/src/parser.rs` + `crates/auto-lang/src/ast/enums.rs`

**方案 A（推荐）: 支持多参数元组变体**

1. **修改 AST**: `EnumItem` 新增 `payload_types: Vec<Type>` 字段

```rust
// ast/enums.rs 修改
pub struct EnumItem {
    pub name: AutoStr,
    pub scalar_value: Option<i32>,
    pub payload_type: Option<Type>,      // 保留向后兼容（单参数）
    pub payload_types: Vec<Type>,         // 新增：多参数元组
    pub fields: Vec<EnumField>,           // 命名字段
}
```

2. **修改 Parser**: 循环消费多个类型

```rust
// parser.rs:3998 修改
} else if self.is_kind(TokenKind::Ident) || self.is_kind(TokenKind::LParen) {
    let mut types = vec![];
    while self.is_kind(TokenKind::Ident) || self.is_kind(TokenKind::LParen) {
        types.push(self.parse_type()?);
    }
    if types.len() == 1 {
        payload_type = Some(types.into_iter().next().unwrap());
    } else {
        payload_types = types;
    }
    has_any_payload = true;
}
```

3. **修改 Transpiler**: 多参数变体 → Rust tuple enum

```rust
// trans/rust.rs 修改 enum_trans()
fn enum_variant_to_rust(item: &EnumItem) -> String {
    if !item.payload_types.is_empty() {
        // ToolUse str str str → ToolUse(String, String, String)
        let types: Vec<String> = item.payload_types.iter()
            .map(|t| self.rust_type_name(t))
            .collect();
        format!("    {}({}),", item.name, types.join(", "))
    } else if let Some(ref pt) = item.payload_type {
        format!("    {}({}),", item.name, self.rust_type_name(pt))
    } else if item.fields.is_empty() {
        format!("    {},", item.name)
    } else {
        // struct-like variant
        ...
    }
}
```

### 验证

```auto
// 修复前：Expected term, got RBrace
// 修复后：正确翻译
enum InputContentBlock {
    Text str
    ToolUse str str str
    ToolResult str str ?bool
}
```

```rust
// 期望输出
enum InputContentBlock {
    Text(&str),
    ToolUse(&str, &str, &str),
    ToolResult(&str, &str, Option<bool>),
}
```

### 预估工作量

- AST 修改：~10 行
- Parser 修改：~15 行
- Transpiler 修改：~20 行
- 测试：添加 `test/a2r/02_types/` 下的多参数变体测试
- 风险：中（需要确保不影响现有的单参数和 struct-like 变体）

---

## Phase 3: ~~`is` 语句支持表达式和 `return`~~（问题 C + D）— 已在之前修复

> **注意**：2026-04-24 验证确认问题 C 和 D 已在之前的提交中修复。
> `is` 已可作为表达式赋值，单行 `return` 在 `is` 分支中也可工作。
> 以下修复方案保留作为历史参考，无需实施。

### 问题 C 根因：`is` 只是 Stmt，不是 Expr

AST 中 `Is(Is)` 只存在于 `enum Stmt`（line 179），不在 `enum Expr`（line 292）中。
Parser 的 Pratt 表达式解析器（`expr_pratt()`）没有 `TokenKind::Is` 的 prefix case。
因此 `let x = is y { ... }` 无法解析。

### 问题 C 修复

**文件**: `crates/auto-lang/src/ast.rs` + `parser.rs` + `trans/rust.rs`

1. **AST**: 在 `enum Expr` 中添加 `Is(Is)` 变体

```rust
pub enum Expr {
    // ... existing variants ...
    Is(Box<Is>),  // 新增：is 作为表达式
}
```

2. **Parser**: 在 `expr_pratt()` 的 prefix cases 中添加 `TokenKind::Is`

```rust
// parser.rs expr_pratt() prefix cases (~line 1409)
TokenKind::Is => {
    let is = self.is_expr()?; // 复用 is 解析逻辑
    Ok(Expr::Is(Box::new(is)))
}
```

3. **Transpiler**: 在 `expr()` 方法中处理 `Expr::Is`

```rust
// trans/rust.rs expr()
Expr::Is(is) => {
    // match target { branches } 作为 Rust match 表达式
    let target = self.expr(&is.target)?;
    let arms = self.is_arms(&is.branches)?;
    write!(self.out, "match {} {{ {} }}", target, arms)
}
```

### 问题 D 根因：`parse_expr_or_body()` 不处理语句关键字

```rust
// parser.rs:5516
pub fn parse_expr_or_body(&mut self) -> AutoResult<Body> {
    if self.is_kind(TokenKind::LBrace) {
        self.body()
    } else {
        // return/break/continue 不是表达式，parse_expr() 会失败
        let mut body = Body::new();
        body.stmts.push(Stmt::Expr(self.parse_expr()?));
        Ok(body)
    }
}
```

### 问题 D 修复

**文件**: `crates/auto-lang/src/parser.rs`（仅此一个文件）

```rust
pub fn parse_expr_or_body(&mut self) -> AutoResult<Body> {
    if self.is_kind(TokenKind::LBrace) {
        self.body()
    } else if self.is_kind(TokenKind::Return) {
        // 单行 return: Some(v) -> return Some(v)
        let mut body = Body::new();
        body.stmts.push(self.return_stmt()?);
        Ok(body)
    } else if self.is_kind(TokenKind::Break) {
        let mut body = Body::new();
        body.stmts.push(Stmt::Break);
        self.next();
        Ok(body)
    } else if self.is_kind(TokenKind::Continue) {
        let mut body = Body::new();
        body.stmts.push(Stmt::Continue);
        self.next();
        Ok(body)
    } else {
        let mut body = Body::new();
        body.stmts.push(Stmt::Expr(self.parse_expr()?));
        Ok(body)
    }
}
```

**Transpiler 侧无需修改** — `write_match_arm_body()` 已经正确处理 `Stmt::Return`。

### 验证

```auto
// 问题 C: is 作为表达式
fn foo(x ?int) int {
    let result = is x {
        None -> 0,
        Some(n) -> n + 1
    }
    return result
}

// 问题 D: is 分支中的 return
fn bar(x ?int) ?int {
    is x {
        None -> return None,
        Some(n) -> return Some(n + 1)
    }
}
```

### 预估工作量

- 问题 C：AST 5 行 + Parser 10 行 + Transpiler 15 行 = ~30 行
- 问题 D：Parser 15 行
- 风险：低（问题 D 纯增强，不影响已有路径；问题 C 需要注意作用域）

---

## Phase 4: a2r 运行时映射扩展（问题 E）

### 根因

a2r 的 `trans/rust.rs` `call()` 方法只为少数函数做了映射（print/printf/assert 等）。
`a2r_std.rs` 只提供了 List/May/Json/str_* 的基础实现。

Auto 的外部调用（`env.get()`、`fs.read_to_string()`、`http_post()` 等）无对应 Rust 输出。

### 修复方案

**文件**: `crates/auto-lang/src/trans/rust.rs` + `crates/auto-lang/src/a2r_std.rs`

#### 4A: 添加函数名映射表

在 `trans/rust.rs` 的 `call()` 方法中扩展映射：

```rust
// env.get("KEY") → std::env::var("KEY").ok()
// env.set("KEY", "VAL") → std::env::set_var("KEY", "VAL")
// fs.read_to_string(path) → std::fs::read_to_string(path).ok()
// fs.write(path, content) → std::fs::write(path, content)
// sleep_ms(ms) → std::thread::sleep(std::time::Duration::from_millis(ms))
```

#### 4B: 扩展 a2r_std.rs

```rust
// a2r_std.rs 新增模块

pub mod env {
    pub fn get(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
    pub fn set(key: &str, val: &str) {
        std::env::set_var(key, val);
    }
}

pub mod fs {
    pub fn read_to_string(path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
    pub fn write(path: &str, content: &str) -> bool {
        std::fs::write(path, content).is_ok()
    }
    pub fn exists(path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

// http_post 需要外部 crate (reqwest)，通过 feature gate 控制
#[cfg(feature = "http")]
pub mod http {
    pub fn http_post(url: &str, body: &str, api_key: &str) -> reqwest::blocking::Response {
        reqwest::blocking::Client::new()
            .post(url)
            .header("x-api-key", api_key)
            .body(body.to_string())
            .send()
            .unwrap()
    }
}
```

#### 4C: 字符串方法映射

当前 transpiler 已映射部分字符串方法（`char_at`, `sub`, `slice` 等），
但缺少以下常用方法：

| Auto 方法 | 期望 Rust 输出 |
|-----------|---------------|
| `s.trim()` | `s.trim()` ✅ 已有 |
| `s.trim_left()` | `s.trim_start()` |
| `s.starts_with(prefix)` | `s.starts_with(prefix)` |
| `s.join(sep)` | `s.join(sep)` (对 Vec) ✅ 已有 |
| `s.to_string()` | `s.to_string()` ✅ 已有 |

### 预估工作量

- 函数映射表：~30 行
- a2r_std.rs 扩展：~80 行
- 字符串方法补齐：~15 行
- 风险：低（纯增量，不影响已有功能）

---

## 实施优先级

```
Phase 1 (Lexer pos)    → 🔴 必须修复，解除 3KB 限制（当前唯一阻断大文件翻译的问题）
Phase 2 (多参数 enum)  → 🔴 必须修复，tagged union 核心语法
Phase 3 (is 表达式)    → ✅ 已修复，无需实施
Phase 4 (运行时映射)   → 🟡 补充增强，env/fs 已有映射，http_post 待添加
```

## 修复后的预期

修复 Phase 1 + 2 后，`main.at` 的 545 行 Auto 代码应能完整通过 a2r 翻译（不再崩溃或报语法错误）。
输出的 Rust 代码仍需手动调整（添加 serde derive、reqwest 依赖等）才能编译运行。

## 预期效果

修复后，step-00 的 `main.at`（545 行, ~16KB）应该能够：
- Phase 1 后：不再崩溃，能给出正确的错误定位
- Phase 2 后：多参数 enum 变体正确翻译为 Rust tuple enum
- Phase 3 后：~~`is` 表达式赋值和单行 return 都能工作~~ ✅ 已支持
- Phase 4 后：HTTP/env/fs 等外部调用有对应的 Rust 实现

注意：修复后 a2r 输出的 Rust 代码仍然不能直接编译——需要手动添加
`Cargo.toml` 依赖（reqwest, tokio, serde 等）和 serde derive 标注。
这是 Plan 204 Phase 2-5 的范畴。
