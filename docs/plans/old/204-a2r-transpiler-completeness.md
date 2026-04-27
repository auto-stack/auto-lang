# Plan 204: a2r 转译器完整性

## Status: ✅ COMPLETE (Phase 4 was already implemented)

Verified 2026-04-28:
- ✅ Result<T> transpilation: Type::Result mapped to Rust Result<T, Box<dyn Err>>, Ok/Err pattern branches
- ✅ Spec transpilation: Type::Spec mapped to Box<dyn Trait>, spec_decls caching
- ✅ Phase 1 basic fixes (assert! macro, &str vs String, ! operator, loop body, array type inference)
- ✅ Phase 2 type -> struct, ext -> impl, enum -> Rust enum
- ✅ Phase 3 Result mapping: !T → Result<T, Box<dyn Err>>, auto-emit Err trait, Err values boxed
- ✅ Phase 4 spec → trait mapping: spec→trait, ext for spec→impl Trait for Type, Box<dyn Trait> (already done)
- ✅ Phase 5 stdlib method mapping (24 methods including char_at, to_hex, find, sub, slice)
- ✅ Phase 6 safe output (.a2r.rs suffix, bracket validation, f-string {{}} escapes)
- ✅ VM fix: CREATE_OK supports all value types via type tag operand (not just i32)

> 基于 ac-examples 01~13 的 `.at` → a2r → `.a2r.rs` 与原始 `.rs` 对比分析。
> 配合 Plan 201 补齐 Auto 语言能力后，a2r 转译器仍需大量实现工作才能输出可编译的 Rust。
> 本计划解决 a2r 转译器自身的实现缺陷。

## 动机

对比结果：**12/13 转译成功，0/12 输出可编译**。a2r 的核心问题是"只转译表达式，不转译声明"——`type`、`ext`、`enum` 块被跳过，只有函数体和 `main()` 被输出。

a2r 输出的典型问题：
- `assert_eq(x, y)` 无 `!` 宏标记
- `Usage::new(100, 50)` 但 `Usage` struct 从未定义
- `hash.to_hex(16)` 无 Rust std 等价方法
- `let s: &str = usage.to_string()` 类型不匹配
- `[/* unknown */; 3]` 数组元素类型推断失败

## 与 Plan 201 的关系

Plan 201 补齐 Auto 语言能力（enum 多字段、闭包 HOF、Result、spec vtable），使 Auto 代码能表达与 Rust 对等的语义。

Plan 204 补齐 a2r 转译器，使已有 Auto 语义能正确映射到可编译的 Rust。

**依赖关系**：Plan 204 的部分 Phase 依赖 Plan 201，但有大量工作可立即推进。

```
Plan 201 Phase 1 (enum 多字段) ──> Plan 204 Phase 2 (enum → Rust enum)
Plan 201 Phase 3 (!T + *Err)    ──> Plan 204 Phase 3 (Result → Rust Result)
Plan 201 Phase 4 (spec vtable)  ──> Plan 204 Phase 4 (spec → Rust trait)

Plan 204 Phase 1 (基础修复)     ── 独立，可立即开始
Plan 204 Phase 5 (stdlib 映射)  ── 独立，可立即开始
Plan 204 Phase 6 (安全输出)     ── 独立，可立即开始
```

## 问题分类

### A. a2r 转译器实现缺陷

| # | 缺陷 | 严重程度 | 影响范围 |
|---|---|---|---|
| A1 | `assert`/`assert_eq`/`assert_ne` 无 `!` 宏标记 | 高 | 所有断言（12/12 示例） |
| A2 | `type` 块不生成 Rust `struct` 定义 | 高 | 所有使用 struct 的代码（5/12） |
| A3 | `ext` 块不生成 Rust `impl` 定义 | 高 | 所有使用 ext 的代码（5/12） |
| A4 | `enum` 不生成 Rust `enum` 定义 | 高 | 所有使用 enum 的代码（5/12） |
| A5 | 数组类型推断输出 `[/* unknown */; N]` | 中 | 数组字面量（4/12） |
| A6 | `&str` vs `String` 返回类型错误 | 中 | 函数返回字符串时（5/12） |
| A7 | Auto stdlib 方法无 Rust 映射 | 中 | `to_hex()`, `char_at()` 等（4/12） |
| A8 | 循环体转译为空（语句丢失） | 高 | `06_line_formatter`（1/12） |
| A9 | `!` 运算符优先级：`!x <= N` → 按位取反 | 中 | `07_glob_match`（1/12） |
| A10 | f-string 嵌套引号使 lexer 崩溃 | 高 | `11_tool_result_serde`（1/12 转译失败） |

### B. Auto 语言设计缺陷（Plan 201 范围，此处仅记录映射策略）

| # | 缺陷 | Plan 201 Phase | a2r 映射策略 |
|---|---|---|---|
| B1 | 扁平 `type` + `kind: str` 替代 enum | Phase 1 | enum 多字段实现后，直接映射为 Rust `enum` |
| B2 | `ok: bool` 字段替代 `Result<T, E>` | Phase 3 | `!T` 实现后，映射为 `Result<T, Box<dyn Err>>` |
| B3 | 字符串分派替代 trait | Phase 4 | spec 实现后，映射为 Rust `trait` + `impl` |
| B4 | 无闭包高阶函数 | Phase 2 | 闭包实现后，映射为 `.map()`/`.filter()` |

---

## Phase 1：基础修复（独立，可立即开始）

### 1A：宏标记 `!`

**问题**：`assert(expr)` → 应输出 `assert!(expr)`，`assert_eq(a, b)` → `assert_eq!(a, b)`

**文件**：`crates/auto-lang/src/trans/rust.rs`

**实现**：
- 在 `fn_call_to_rust()` 或语句输出逻辑中，检测函数名为 `assert`/`assert_eq`/`assert_ne`/`assert!(...)`/`assert_eq!(...)`/`assert_ne!(...)`
- 自动追加 `!` 后缀
- 保持参数格式不变（Rust 宏参数语法与函数调用语法一致）

### 1B：`&str` vs `String` 返回类型

**问题**：Auto 函数 `fn foo() str` 被转译为 `fn foo() -> &str`，但函数体可能返回 `String`

**文件**：`crates/auto-lang/src/trans/rust.rs`

**实现**：
- 当函数返回类型为 `str` 时，分析函数体：
  - 如果只有 `return "literal"` → `-> &'static str`
  - 如果包含 `format!(...)`/`.to_string()`/拼接 → `-> String`
  - 如果无法确定 → `-> String`（更安全的选择）
- 参数类型 `str` 保持 `&str`（借用语义）

### 1C：`!` 运算符优先级

**问题**：Auto 的 `!expr <= val`（逻辑非）被原样输出，Rust 解析为 `(!expr) <= val`（按位取反）

**文件**：`crates/auto-lang/src/trans/rust.rs`

**实现**：
- 当 `!` 应用于比较表达式的左操作数时，包裹括号：`!(expr <= val)`
- AST 中应已保留运算符优先级信息，在输出时检查

### 1D：循环体语句丢失

**问题**：`06_line_formatter` 中 `count_lines`/`get_line` 的 `while` 循环体为空

**文件**：`crates/auto-lang/src/trans/rust.rs`

**实现**：
- 调试 `while`/`for` 循环的转译逻辑，找出语句丢失原因
- 可能是 `Block` 或 `Body` 的转译递归在某些情况下跳过了子语句
- 添加针对循环体为空的测试用例

### 1E：数组类型推断

**问题**：`[expr1, expr2, ...]` 被输出为 `[/* unknown */; N]`

**文件**：`crates/auto-lang/src/trans/rust.rs`

**实现**：
- 从数组元素表达式中推断元素类型
- 如果元素类型一致（如都是 `int`）→ `[i32; N]`
- 如果元素是用户类型（如 `ContentBlock`）→ 使用类型名
- 如果无法推断 → 使用 `Vec<_>` 替代（Rust 可以推断）

---

## Phase 2：`type` → `struct` + `ext` → `impl` 转译

### 2A：`type` 块生成 Rust `struct` 定义

**问题**：`type Usage { input_tokens uint, output_tokens uint }` 被完全跳过

**文件**：`crates/auto-lang/src/trans/rust.rs`

**实现**：
- 在 `transpile_stmt()` 中处理 `Stmt::TypeDecl(type_decl)`
- 输出格式：
  ```rust
  #[derive(Clone, Debug, PartialEq)]
  struct Usage {
      input_tokens: u32,
      output_tokens: u32,
  }
  ```
- 根据类型中引用的 specs 添加 derive（如果有 `==` 比较 → `PartialEq`，如果有 `to(str)` → `Debug`）

### 2B：`ext` 块生成 Rust `impl` 定义

**问题**：`ext Usage { fn total_tokens() uint { ... } }` 被完全跳过

**文件**：`crates/auto-lang/src/trans/rust.rs`

**实现**：
- 在 `transpile_stmt()` 中处理 `Stmt::Ext(ext_block)`
- 区分两种形式：
  - `ext TypeName { ... }` → `impl TypeName { ... }`（固有实现）
  - `ext TypeName for SpecName { ... }` → `impl SpecName for TypeName { ... }`（trait 实现）
- 方法签名映射：
  - `static fn new(...) Self` → `pub fn new(...) -> Self`
  - `fn method(self) Type` → `pub fn method(&self) -> Type`（注意 `&self` 引用）
  - `fn method(self, arg Type) Type` → `pub fn method(&self, arg: Type) -> Type`

### 2C：`enum` 块生成 Rust `enum` 定义

**问题**：`enum PermissionMode { Allow = 1, Ask = 2 }` 不生成 Rust enum

**文件**：`crates/auto-lang/src/trans/rust.rs`

**实现**：
- 根据 `EnumKind` 分派：
  - `Scalar` → Rust C-like enum：
    ```rust
    #[derive(Clone, Debug, PartialEq)]
    enum PermissionMode { Allow = 1, Ask = 2, ReadOnly = 3 }
    ```
  - `Heterogeneous`（当前单字段）→ Rust 带数据 enum：
    ```rust
    enum Atom { Int(i64), Str(String), None }
    ```
  - `Heterogeneous`（Plan 201 Phase 1 后多字段）→ Rust struct-variant enum：
    ```rust
    enum ApiError {
        Http(String),
        Api { status: u32, message: String, retryable: bool },
    }
    ```
- 自动添加 `#[derive(Clone, Debug, PartialEq)]`

---

## Phase 3：`!T` + `*Err` → Rust `Result<T, E>` 映射（依赖 Plan 201 Phase 3）

**前置条件**：Plan 201 Phase 3 完成后，Auto 有 `!T` + `*Err` 运行时方案

### 映射策略

```auto
// Auto
fn execute(input str) !str {
    if input == "" {
        return Err(ParseError.UnexpectedEnd)
    }
    Ok(input.to_upper())
}
```

↓ a2r 转译为 ↓

```rust
// Rust
fn execute(input: &str) -> Result<String, Box<dyn Err>> {
    if input.is_empty() {
        return Err(Box::new(ParseError::UnexpectedEnd));
    }
    Ok(input.to_upper())
}
```

### 实现步骤

- `!T` 返回类型 → `Result<T, Box<dyn Err>>`
- `Ok(expr)` → `Ok(expr)`（直译）
- `Err(enum_variant)` → `Err(Box::new(EnumName::Variant))`（装箱为 trait 对象）
- `.?` 错误传播 → `?` 操作符（直译，Rust 语法一致）
- `is result { Ok(x) -> ... Err(e) -> ... }` → `match result { Ok(x) => ... Err(e) => ... }`

---

## Phase 4：`spec` → Rust `trait` 映射（依赖 Plan 201 Phase 4）

**前置条件**：Plan 201 Phase 4 完成后，Auto 有 spec + vtable 动态分派

### 映射策略

```auto
// Auto
spec Tool {
    fn name() str
    fn execute(input str) !str
}

type EchoTool
ext EchoTool for Tool {
    fn name() str { "Echo" }
    fn execute(input str) !str { Ok(f"echo: $input") }
}

let tools List<Tool> = [EchoTool, UpperTool]
```

↓ a2r 转译为 ↓

```rust
// Rust
trait Tool {
    fn name(&self) -> String;
    fn execute(&self, input: &str) -> Result<String, Box<dyn Err>>;
}

struct EchoTool;
impl Tool for EchoTool {
    fn name(&self) -> String { "Echo".to_string() }
    fn execute(&self, input: &str) -> Result<String, Box<dyn Err>> {
        Ok(format!("echo: {}", input))
    }
}

let tools: Vec<Box<dyn Tool>> = vec![Box::new(EchoTool), Box::new(UpperTool)];
```

### 实现步骤

- `spec Name { methods }` → `trait Name { method_signatures }`
- `ext TypeName for SpecName { methods }` → `impl SpecName for TypeName { methods }`
- `List<SpecType>` → `Vec<Box<dyn SpecType>>`
- `SpecType` 参数 → `&dyn SpecType` 或 `Box<dyn SpecType>`
- dyn 对象构造 `ConcreteType` → `Box::new(ConcreteType) as Box<dyn SpecType>`

---

## Phase 5：Auto stdlib → Rust std 映射（独立）

### 方法映射表

| Auto 方法 | Rust 等价 | 说明 |
|---|---|---|
| `s.len()` / `s.length()` | `s.len()` | 直接映射 |
| `s.to_hex(n)` | `format!("{:0>width$x}", val, width = n)` | 十六进制格式化 |
| `s.char_at(i)` | `s.chars().nth(i).unwrap()` | 字符索引 |
| `s.sub(start, end)` | `&s[start..end]` | 切片 |
| `s.slice(n)` | `&s[n..]` | 后缀切片 |
| `s.starts_with(p)` | `s.starts_with(p)` | 直接映射 |
| `s.ends_with(p)` | `s.ends_with(p)` | 直接映射 |
| `s.contains(p)` | `s.contains(p)` | 直接映射 |
| `s.find(p)` | `s.find(p).map(|i| i as i32).unwrap_or(-1)` | 返回索引或 -1 |
| `s.repeat(n)` | `"x".repeat(n)` | 返回 `String` |
| `s.to_upper()` | `s.to_uppercase()` | 大写 |
| `s.to(str)` / `.to_string()` | `format!("{:?}", val)` 或 `val.to_string()` | 类型转换 |
| `now_ms()` | `SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()` | 时间戳 |
| `min(a, b)` | `std::cmp::min(a, b)` 或 `a.min(b)` | 最小值 |
| `print(expr)` | `println!("{}", expr)` | 输出 |
| `n.to(str)` | `n.to_string()` | 数字转字符串 |

### 实现步骤

- 文件：`crates/auto-lang/src/trans/rust.rs`
- 在方法调用转译逻辑中（`method_call_to_rust()` 或等效位置）
- 添加映射表查找：接收者类型 + 方法名 → Rust 等价表达式
- 需要考虑方法链：`s.find(marker).slice(start)` 的映射顺序

---

## Phase 6：安全输出机制（独立）

### 6A：输出路径安全

**问题**：`trans_rust()` 硬编码 `path.replace(".at", ".rs")`，直接覆盖 `.rs` 源文件

**文件**：`crates/auto-lang/src/lib.rs` `trans_rust_with_session()`

**实现**：
- 修改输出路径逻辑：`.at` → `.a2r.rs`（而非 `.rs`）
- 或添加 `output_path` 参数，允许 CLI 指定输出路径
- CLI 侧：`auto rust main.at -o main.a2r.rs`

### 6B：输出验证

**实现**：
- 转译完成后，运行 `rustfmt --check` 验证语法
- 或至少做基本语法检查（括号匹配、分号存在性）
- 如果验证失败，输出警告但不阻止写入

### 6C：f-string 嵌套引号修复

**问题**：`f'[{{"type":"text","text":"${text}"}}]'` 使 lexer 崩溃

**文件**：`crates/auto-lang/src/lexer.rs`

**实现**：
- f-string 中 `{{` 和 `}}` 应作为转义花括号处理，不应干扰字符串边界检测
- 当 lexer 遇到 `f'...` 或 `f"..."` 时，应正确处理内部的 `{{`/`}}` 和 `${...}` 嵌套

---

## 实施优先级

```
Phase 1 (基础修复) ──── 立即开始，无外部依赖
Phase 5 (stdlib 映射) ── 立即开始，无外部依赖
Phase 6 (安全输出) ──── 立即开始，无外部依赖
Phase 2 (type/ext/enum) ─ 立即开始，可先做单字段版本
Phase 3 (Result 映射) ─── 等 Plan 201 Phase 3 完成
Phase 4 (spec 映射) ───── 等 Plan 201 Phase 4 完成
```

建议实施顺序：
1. **Phase 6C**（f-string 修复）— 修复 11 的转译失败
2. **Phase 1A**（宏标记 `!`）— 影响面最广（12/12）
3. **Phase 6A**（安全输出路径）— 防止覆盖源文件
4. **Phase 2**（type/ext/enum 转译）— 最核心的结构性改进
5. **Phase 5**（stdlib 映射）— 减少运行时错误
6. **Phase 1B/1C/1D/1E**（类型系统修复）— 边角问题
7. **Phase 3/4**（等 Plan 201 完成）

## 验证标准

每个 Phase 完成后，用 ac-examples 01~13 作为回归测试：

| Phase | 目标 |
|---|---|
| Phase 1A | 所有 `assert`/`assert_eq` 输出带 `!` |
| Phase 2A+B | `08_usage_struct` 的 a2r 输出包含 `struct Usage` 和 `impl Usage` |
| Phase 2C | `02_retryable_status` 的 a2r 输出包含 `enum PermissionMode` |
| Phase 5 | `01_djb2_hash` 的 `to_hex(16)` 正确映射为 `format!` |
| Phase 6A | a2r 不再覆盖原始 `.rs` 文件 |
| Phase 6C | `11_tool_result_serde` 转译不再崩溃 |
| 全部完成 | 01~13 的 a2r 输出通过 `rustc --check` |
