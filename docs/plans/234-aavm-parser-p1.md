# Plan 234: AAVM Parser P1 — 高优先级缺失特性

## Context

Plan 233 完成了 AAVM 自托管 parser P0 + 文件化测试框架（19 个共享测试）。当前 AAVM parser 覆盖约 25% 的语言特性，能解析基本表达式、if/else、for/for-in、fn/type 声明、let/var/return。

本计划添加 **10 个高优先级特性**，使 AAVM parser 能解析自举编译器需要的核心语法。同时记录中低优先级特性供后续计划。

## 实施范围

### 高优先级（本计划实现）

| # | 特性 | 示例 | 改动文件 |
|---|------|------|----------|
| 1 | Closure | `x => x + 1`, `(a, b) => a + b` | ast.at, parser.at, lexer.at |
| 2 | F-string | `` `Hello ${name}` `` | ast.at, parser.at, lexer.at |
| 3 | is/match 模式匹配 | `is x { Some(v) -> v }` | ast.at, parser.at |
| 4 | enum 声明 | `enum Color { Red, Green }` | ast.at, parser.at |
| 5 | tag 声明 | `tag Atom { Int int }` | parser.at（复用 enum） |
| 6 | use/import | `use math::add`, `use.c <stdio.h>` | ast.at, parser.at |
| 7 | ext 块 | `ext Point { fn dist() {} }` | ast.at, parser.at |
| 8 | spec 声明 | `spec Flyer { fn fly() }` | ast.at, parser.at |
| 9 | alias 声明 | `alias IntList = List` | ast.at, parser.at |
| 10 | Object 字面量 | `{x: 1, y: 2}` | ast.at, parser.at |

## 文件改动总览

| 文件 | 变更 |
|------|------|
| `auto/lib/ast.at` | 添加 NodeKind 变体 + 构造函数 |
| `auto/lib/parser.at` | 添加 10 个解析函数 + dispatch |
| `auto/lib/lexer.at` | 添加 C-string、char、f-string 词法规则 |
| `crates/auto-lang/src/tests/vm_file_tests.rs` | 注册新测试 |
| `crates/auto-lang/test/vm/99_bootstrap/028-037_*/` | 新建 10 个测试目录 |

## Step 1: Lexer 扩展 (`auto/lib/lexer.at`)

### 1.1 F-string 词法（P1 简化版）

只支持 `$ident` 形式的插值，产出 token 序列：`FStrStart("Hello ")`, `FStrNote`, `Ident(name)`, `FStrEnd("")`。

### 1.2 C-string 词法 (`c"..."`)

当遇到 `c` 后跟 `"` 时，读取原始字符串（不处理转义），产出 `CStr` token。

### 1.3 Char 字面量 (`'a'`)

当遇到 `'` 时，读取单个字符（含转义），产出 `Char` token。

## Step 2: AST 节点扩展 (`auto/lib/ast.at`)

新增 9 个 NodeKind：`ClosureExpr`, `FStrExpr`, `IsStmt`, `EnumStmt`, `ExtStmt`, `SpecStmt`, `AliasStmt`, `ObjectExpr`, `PairExpr`。

每个新增对应构造函数，`value` 字段存储预计算 S-expression 字符串。

## Step 3: Parser 扩展 (`auto/lib/parser.at`)

### 实施顺序（由简到难）

1. **alias** — `alias Name = Target`，最简单
2. **enum/tag** — `enum Name { variants }`，含可选 payload
3. **use/import** — 模块路径 + items，多形式
4. **spec** — `spec Name { fn sig() }`，方法签名无 body
5. **ext** — `ext Type { fn method() {} }`，含方法 body
6. **closure** — `x => expr` / `(a, b) => expr`，需 lookahead
7. **f-string** — FStrStart/FStrPart/FStrEnd token 组合
8. **is/match** — 分支模式匹配，P1 仅字面量+else
9. **object** — `{key: value}` vs `{stmt}` 歧义消解

### 关键技术点

- **Closure 检测**：在 `parse_atom()` 的 `Ident` case 中 peek `DoubleArrow`；在 `LParen` case 中检查是否为 `(params) =>` 形式
- **Object vs Block**：`parse_atom()` 中 `LBrace` 时 lookahead 检查 `Ident + Colon` 模式
- **is 分支**：使用 `->` (Arrow) 分隔 pattern 和 body，`else` 处理默认分支

## Step 4: 测试用例

新建 10 个测试目录（028-037），每个含 `.at` + `.expected.out` + `.expected.rust_ast`。

| # | 目录 | Source | 说明 |
|---|------|--------|------|
| 028 | parser_closure | `x => x + 1` | 单参数闭包 |
| 029 | parser_closure_multi | `(a, b) => a + b` | 多参数闭包 |
| 030 | parser_fstr | `` `Hello ${name}` `` | F-string |
| 031 | parser_is | `is x { 1 -> true }` | 模式匹配 |
| 032 | parser_enum | `enum Color { Red, Green }` | enum 声明 |
| 033 | parser_use | `use math` | use 声明 |
| 034 | parser_ext | `ext str { fn len() int }` | ext 块 |
| 035 | parser_spec | `spec Flyer { fn fly() }` | spec 声明 |
| 036 | parser_alias | `alias CC = MyAdd` | alias 声明 |
| 037 | parser_object | `{x: 1, y: 2}` | Object 字面量 |

## 验证

```bash
cargo test -p auto-lang --lib -- test_rust_99_bootstrap
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap
cargo test -p auto-lang --lib -- vm_file_tests
```

预估代码量：ast.at +120 行 + parser.at +400 行 + lexer.at +80 行 ≈ 600 行。

---

## 附录 A：中优先级特性（Plan 235 候选）

| 特性 | 示例 | 难度 |
|------|------|------|
| 泛型类型参数 | `List<int>`, `fn foo<T>()` | 中 |
| Option/Result 构造 | `Some(42)`, `None`, `Ok(v)` | 易 |
| Named arguments | `foo(x: 1, y: 2)` | 易 |
| pub 可见性 | `pub fn foo()` | 易 |
| 注解/属性 | `#[vm]`, `#[c]` | 中 |
| const/shared | `const MAX = 100` | 易 |
| loop 语句 | `loop { break }` | 易 |
| reply 语句 | `reply result` | 易 |
| Tuple | `(1, "a", true)` | 中 |
| 类型转换 | `expr.as(int)` | 中 |
| 错误传播 | `expr.?`, `expr ?? default` | 中 |
| 属性访问器 | `expr.view`, `expr.mut` | 易 |
| pub use 重导出 | `pub use db: Connection` | 易 |
| dep 声明 | `dep "package"` | 易 |
| Slice/Range index | `arr[1..3]` | 中 |

## 附录 B：低优先级特性（Plan 236+ 候选）

| 特性 | 示例 | 难度 |
|------|------|------|
| Char 字面量 | `'a'` | 易 |
| C-string | `c"hello"` | 易 |
| Hex/Binary | `0xFF`, `0b1010` | 易 |
| Widget/Node | `button("OK") { border: 1 }` | 难 |
| Grid | `grid("a","b") { row }` | 难 |
| task/spawn | `task Counter { }` | 难 |
| async/await | `fn foo() ~int {}` | 难 |
| Comptime | `#if DEBUG { }` | 难 |
| on 事件块 | `on { Ev -> handler }` | 中 |
| 方法前缀 mut fn | `mut fn push() {}` | 易 |
| 静态方法 | `static fn new() {}` | 易 |
| 文档注释 | `/// doc comment` | 易 |

## 不在范围内

- 泛型类型解析（`<T>` 语法）
- 类型检查 / 类型推导
- 语义分析
- 代码生成
- async/await 运行时
