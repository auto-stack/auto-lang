# Plan 233: AAVM 自托管 Parser（P0 + P1）

## Context

AAVM 已完成自举 Lexer（Phase 1.1 Token + Phase 1.2 Lexer），能正确 tokenize `"let x = 42"` → `<let>x=42`。本计划实现 **Phase 1.3 AST + Phase 1.4 Parser**，让 VM 能 parse AutoLang 程序。

**参考实现**：Rust parser（`crates/auto-lang/src/parser.rs`，~11,000 行）使用 Pratt parsing 处理表达式优先级，recursive descent 处理语句。

**VM 能力确认**：
- `enum` 比较：`kind == TokenKind.Fn` ✓
- `is` 模式匹配：`is kind { TokenKind.Fn -> ... }` ✓
- `List.push()` / `List.get()` 含 type 实例 ✓
- `type` 字段可变（`p.pos = p.pos + 1`）✓

## 文件改动总览

| 文件 | 说明 |
|------|------|
| `auto/lib/lexer.at` | 添加 `tokenize_list()` + F-string/C-string/Char 词法规则 |
| `auto/lib/ast.at` | **新建** — AST 节点类型 + 构造函数 + S-expression 输出 |
| `auto/lib/parser.at` | **新建** — Pratt 表达式解析器 + 语句解析器（P0+P1 共 20 个特性） |
| `crates/auto-lang/src/tests/vm_file_tests.rs` | 注册新测试 |
| `crates/auto-lang/test/vm/99_bootstrap/008-037_*/` | 新建 30 个测试目录 |

---

## Part A: P0 — 基础 Parser

### A.1 lexer.at — 添加 `tokenize_list()`

返回 `List<Token>` 而非字符串，复用所有 `lex_*` 函数。

```auto
fn tokenize_list(source str) List {
    var p uint = 0
    var list = List.new()
    var len = source.len().as(uint)

    for p < len {
        p = skip_ws(source, p)
        if p >= len { break }

        var c = cur_char(source, p)

        if c == '\n' {
            list.push(Token(TokenKind.Newline, Pos(1, p, 1), "<nl>"))
            p = p + 1
        } else if is_digit(c) {
            var tok = lex_number(source, p)
            list.push(tok)
            p = p + tok.pos.total
        } else if is_alpha(c) {
            var tok = lex_ident(source, p)
            list.push(tok)
            p = p + tok.pos.total
        } else if c == '"' {
            var tok = lex_string(source, p)
            list.push(tok)
            p = p + tok.pos.total
        } else {
            var tok = lex_operator(source, p)
            if tok.kind != TokenKind.EOF {
                list.push(tok)
                p = p + tok.pos.total
            } else {
                p = p + 1  // skip unknown char
            }
        }
    }

    // Append EOF sentinel
    list.push(Token(TokenKind.EOF, Pos(0, p, 0), ""))
    return list
}
```

### A.2 ast.at — AST 节点类型

**核心设计**：Auto VM 没有 Rust 风格的 data-carrying enum，用 `type struct` + `kind` 鉴别器。

#### NodeKind 枚举

```auto
enum NodeKind {
    // Expressions
    IntExpr
    StrExpr
    BoolExpr
    IdentExpr
    BinExpr
    UnaryExpr
    CallExpr
    DotExpr
    // Statements
    FnStmt
    LetStmt
    VarStmt
    ReturnStmt
    IfStmt
    ForStmt
    ForInStmt
    ExprStmt
    BlockStmt
    TypeStmt
    NilNode
    // P1 additions
    ClosureExpr
    FStrExpr
    IsStmt
    EnumStmt
    ExtStmt
    SpecStmt
    AliasStmt
    ObjectExpr
    PairExpr
}
```

#### ASTNode 类型

```auto
type ASTNode {
    kind NodeKind    // 节点类型
    value str        // 字面量文本 / 标识符名 / 函数名
    children List    // 子节点列表（body语句、call参数）
    left List        // 单子节点（let的expr、binary的左操作数、dot的对象）
    right List       // 单子节点（binary的右操作数、index的索引）
    op str           // 操作符文本（+, -, *, /, =, ==, 等）
    params List      // 函数参数列表（List of Param）
    type_name str    // 类型标注 / 返回类型
    cond List        // if/for 的条件表达式
    else_body List   // else 分支
}
```

#### Param 类型

```auto
type Param {
    name str
    type_name str
}
```

#### 构造辅助函数

```
int_node(val) → ASTNode(IntExpr, val, ...)
str_node(val) → ASTNode(StrExpr, val, ...)
bool_node(val) → ASTNode(BoolExpr, val, ...)
ident_node(name) → ASTNode(IdentExpr, name, ...)
nil_node() → ASTNode(NilNode, "", ...)
bin_node(op, left, right) → ASTNode(BinExpr, op=op, left=[left], right=[right])
unary_node(op, operand) → ASTNode(UnaryExpr, op=op, left=[operand])
call_node(callee, args) → ASTNode(CallExpr, children=args, left=[callee])
dot_node(obj, field) → ASTNode(DotExpr, value=field, left=[obj])
fn_node(name, params, ret, body) → ASTNode(FnStmt, value=name, params=params, type_name=ret, children=body)
store_node(kind, name, type_name, expr) → ASTNode(kind, value=name, type_name=type_name, left=[expr])
return_node(expr) → ASTNode(ReturnStmt, left=[expr] or empty)
if_node(cond, body, else_body) → ASTNode(IfStmt, cond=[cond], children=body, else_body=else_body)
forin_node(name, range, body) → ASTNode(ForInStmt, value=name, cond=[range], children=body)
for_node(cond, body) → ASTNode(ForStmt, cond=[cond], children=body)
// P1 additions
closure_node(params, body) → ASTNode(ClosureExpr, params=params, children=body)
fstr_node(parts) → ASTNode(FStrExpr, children=parts)
is_node(expr, branches, else_body) → ASTNode(IsStmt, left=[expr], children=branches, else_body=else_body)
enum_node(name, variants) → ASTNode(EnumStmt, value=name, children=variants)
ext_node(name, methods) → ASTNode(ExtStmt, value=name, children=methods)
spec_node(name, methods) → ASTNode(SpecStmt, value=name, children=methods)
alias_node(name, target) → ASTNode(AliasStmt, value=name, type_name=target)
object_node(pairs) → ASTNode(ObjectExpr, children=pairs)
pair_node(key, value) → ASTNode(PairExpr, value=key, left=[value])
```

#### `ast_to_string()` — S-expression 输出

递归将 AST 转为可打印字符串：
- `IntExpr` → `"42"`
- `StrExpr` → `"\"hello\""`
- `IdentExpr` → `"x"`
- `BinExpr` → `"(+ 1 2)"`
- `CallExpr` → `"(print \"hello\")"`
- `DotExpr` → `"obj.field"`
- `FnStmt` → `"(fn main (params) (body ...))"`
- `LetStmt` → `"(let x 42)"`
- `IfStmt` → `"(if cond (body ...) (else ...))"`

### A.3 parser.at — 核心解析器

#### Parser 类型

```auto
type Parser {
    tokens List    // List<Token>
    pos uint       // 当前位置
    len uint       // token 总数
}
```

#### Token 访问函数

```
parser_new(tokens) → Parser
parser_cur(p) → Token
parser_kind(p) → TokenKind
parser_is(p, kind) → bool
parser_skip_newlines(p) → 跳过 Newline tokens
parser_skip_semi(p) → 跳过 Semi tokens
```

#### 入口点

```
parse_program(p) → List<ASTNode>
parse_top_stmt(p) → ASTNode  (dispatch to fn/let/var/type/expr)
parse_body(p) → List<ASTNode> (expect { ... })
parse_stmt(p) → ASTNode (dispatch to fn/let/var/return/if/for/expr)
```

#### P0 语句解析器

| 函数 | 处理 | 关键逻辑 |
|------|------|----------|
| `parse_fn_decl(p)` | `fn name(params) ret { body }` | 跳过 fn → 读 name → 解析 params → 可选返回类型 → parse_body |
| `parse_let_stmt(p)` | `let name [type] = expr` | 跳过 let → 读 name → peek 检查是否有 type → expect = → parse_expr |
| `parse_var_stmt(p)` | `var name [type] = expr` | 同 let |
| `parse_return_stmt(p)` | `return [expr]` | 跳过 return → 如果下一个不是 Newline/RBrace/Semi/EOF 则 parse_expr |
| `parse_if_stmt(p)` | `if cond { body } [else { body }]` | 跳过 if → parse_expr → parse_body → 可选 else |
| `parse_for_stmt(p)` | `for ... { body }` | 分支：for {body} / for name in range {body} / for cond {body} |
| `parse_type_decl(p)` | `type Name { fields }` | 简化版：只解析字段名+类型 |

#### Pratt 表达式解析器

**优先级表**（8 级）：

| 级别 | 操作符 | 说明 |
|------|--------|------|
| 1 | `=` `+=` `-=` `*=` `/=` | 赋值（右结合） |
| 2 | `||` | 逻辑或 |
| 3 | `&&` | 逻辑与 |
| 4 | `==` `!=` | 相等 |
| 5 | `<` `>` `<=` `>=` `..` `..=` | 比较/Range |
| 6 | `+` `-` | 加减 |
| 7 | `*` `/` `%` | 乘除 |
| 8 | `.` `(` `[` | 后缀（dot/call/index） |

```
parse_expr(p) → parse_expr_prec(p, 1)
parse_expr_prec(p, min_prec) → ASTNode
    lhs = parse_atom(p)
    loop:
        kind = parser_kind(p)
        if kind is stop token: break
        prec = infix_prec(kind)
        if prec < min_prec: break
        handle operator (consume token, parse rhs with prec+1, build node)
    return lhs
```

赋值处理：右结合，`parse_expr_prec(p, prec)`（不变 prec，不是 +1）。

#### 原子表达式 `parse_atom(p)`

| Token | 生成节点 |
|-------|----------|
| `Int/Uint/Float` | `int_node(text)` / `float_node(text)` |
| `Str` | `str_node(text)` |
| `True/False` | `bool_node("true"/"false")` |
| `Nil` | `nil_node()` |
| `Ident` | `ident_node(text)` |
| `LParen` | 括号表达式 / P1: `(params) =>` 闭包检测 |
| `LSquare` | 数组 `[expr, ...]` |
| `Sub` | 一元负 `unary_node("-", operand)` |
| `Not` | 一元非 `unary_node("!", operand)` |

#### `infix_prec(kind)` 优先级查表

```auto
fn infix_prec(kind TokenKind) uint {
    is kind {
        TokenKind.Asn -> 1
        TokenKind.AddEq -> 1
        TokenKind.SubEq -> 1
        TokenKind.Or -> 2
        TokenKind.And -> 3
        TokenKind.Eq -> 4
        TokenKind.Neq -> 4
        TokenKind.Lt -> 5
        TokenKind.Gt -> 5
        TokenKind.Le -> 5
        TokenKind.Ge -> 5
        TokenKind.Range -> 5
        TokenKind.RangeEq -> 5
        TokenKind.Add -> 6
        TokenKind.Sub -> 6
        TokenKind.Star -> 7
        TokenKind.Div -> 7
        TokenKind.Mod -> 7
        TokenKind.Dot -> 8
        TokenKind.LParen -> 8
        TokenKind.LSquare -> 8
        else -> 0
    }
}
```

---

## Part B: P1 — 高优先级扩展特性

### B.1 Lexer 扩展

- **F-string 词法**：只支持 `$ident` 形式的插值，产出 token 序列：`FStrStart`, `FStrNote`, `Ident`, `FStrEnd`
- **C-string** (`c"..."`)：读取原始字符串（不处理转义），产出 `CStr` token
- **Char 字面量** (`'a'`)：读取单个字符（含转义），产出 `Char` token

### B.2 P1 Parser 特性（10 个）

| # | 特性 | 示例 | 说明 |
|---|------|------|------|
| 1 | Closure | `x => x + 1`, `(a, b) => a + b` | parse_atom 中 peek `=>` 检测 |
| 2 | F-string | `` `Hello ${name}` `` | FStrStart/FStrPart/FStrEnd token 组合 |
| 3 | is/match | `is x { Some(v) -> v }` | P1 仅字面量+else 分支 |
| 4 | enum | `enum Color { Red, Green }` | 含可选 payload |
| 5 | tag | `tag Atom { Int int }` | 复用 enum 解析 |
| 6 | use/import | `use math::add`, `use.c <stdio.h>` | 模块路径 + items |
| 7 | ext 块 | `ext Point { fn dist() {} }` | 含方法 body |
| 8 | spec 声明 | `spec Flyer { fn fly() }` | 方法签名无 body |
| 9 | alias | `alias IntList = List` | 最简单 |
| 10 | Object | `{x: 1, y: 2}` | vs `{stmt}` 歧义消解 |

### B.3 实施顺序（由简到难）

1. **alias** — `alias Name = Target`
2. **enum/tag** — `enum Name { variants }`
3. **use/import** — 模块路径 + items
4. **spec** — `spec Name { fn sig() }`
5. **ext** — `ext Type { fn method() {} }`
6. **closure** — `x => expr` / `(a, b) => expr`，需 lookahead
7. **f-string** — FStrStart/FStrPart/FStrEnd 组合
8. **is/match** — 分支模式匹配
9. **object** — `{key: value}` vs `{stmt}` 歧义消解

### B.4 关键技术点

- **Closure 检测**：`parse_atom()` 的 `Ident` case 中 peek `DoubleArrow`；`LParen` case 中检查 `(params) =>` 形式
- **Object vs Block**：`parse_atom()` 中 `LBrace` 时 lookahead 检查 `Ident + Colon` 模式
- **is 分支**：使用 `->` (Arrow) 分隔 pattern 和 body，`else` 处理默认分支

---

## 测试

### P0 测试（008-027）

| # | 目录 | 说明 |
|---|------|------|
| 008 | parser_hello | `fn main() { print("hello") }` |
| 009-027 | parser_* | 基本表达式、if/else、for、fn、let/var、type 等 |

### P1 测试（028-037）

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

### 验证

```bash
cargo test -p auto-lang --lib -- test_rust_99_bootstrap
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap
cargo test -p auto-lang --lib -- vm_file_tests
```

---

## 实施顺序

1. **A.1**: `lexer.at` — 添加 `tokenize_list()` (~30 行)
2. **A.2**: `ast.at` — 节点类型 + 构造函数 + `ast_to_string()` (~200 行)
3. **A.3 (3.1-3.2)**: `parser.at` — Parser 类型 + token 访问 (~40 行)
4. **A.3 (3.5-3.6)**: `parser.at` — Pratt 表达式解析器 + atom (~150 行)
5. **A.3 (3.3-3.4)**: `parser.at` — P0 语句解析器 (~150 行)
6. P0 测试 + 调试
7. **B.1**: `lexer.at` — F-string/C-string/Char 词法 (~80 行)
8. **B.2-B.3**: `parser.at` — P1 特性按顺序实现 (~400 行)
9. P1 测试 + 调试

预估代码量：`ast.at` ~270 行 + `parser.at` ~750 行 + `lexer.at` ~110 行 ≈ 1130 行。

---

## 不在范围内

- 泛型类型参数 `<T>`
- 类型检查 / 类型推导
- 语义分析
- 代码生成
- async/await 运行时

## 附录 A：中优先级特性（后续 Plan 候选）

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

## 附录 B：低优先级特性

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
