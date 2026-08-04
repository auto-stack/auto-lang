# Plan 233: AAVM 自托管 Parser P0 — 让 VM 能 parse hello world

## Context

AAVM 已完成自举 Lexer（Phase 1.1 Token + Phase 1.2 Lexer），能正确 tokenize `"let x = 42"` → `<let>x=42`。现在需要实现 **Phase 1.3 AST + Phase 1.4 Parser**，让 VM 能 parse 简单的 hello world 程序。

**目标**：在 VM 中运行 parser，将 `"fn main() { print(\"hello\") }"` 解析为 AST 并输出 S-expression `(fn main (params) (body (print "hello")))`。

**参考实现**：Rust parser（`crates/auto-lang/src/parser.rs`，~11,000 行）使用 Pratt parsing 处理表达式优先级，recursive descent 处理语句。

**VM 能力确认**：
- `enum` 比较：`kind == TokenKind.Fn` ✓
- `is` 模式匹配：`is kind { TokenKind.Fn -> ... }` ✓
- `List.push()` / `List.get()` 含 type 实例 ✓
- `type` 字段可变（`p.pos = p.pos + 1`）✓

## 新建/修改文件

| 文件 | 说明 |
|------|------|
| `auto/lib/lexer.at` | 添加 `tokenize_list()` 返回 `List<Token>` |
| `auto/lib/ast.at` | **新建** — AST 节点类型 + S-expression 输出 |
| `auto/lib/parser.at` | **新建** — Pratt 表达式解析器 + 语句解析器 |
| `crates/auto-lang/test/vm/99_bootstrap/008_parser_hello/` | **新建** — VM 测试 |
| `docs/plans/233-aavm-parser-p0.md` | 计划文档 |

## Step 1: lexer.at — 添加 `tokenize_list()`

在现有 `tokenize()` 函数后添加新函数，返回 `List<Token>` 而非字符串。复用所有 `lex_*` 函数。

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

## Step 2: ast.at — AST 节点类型

**核心设计**：Auto VM 没有 Rust 风格的 data-carrying enum，用 `type struct` + `kind` 鉴别器。

### 2.1 NodeKind 枚举

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
}
```

### 2.2 ASTNode 类型（统一节点）

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

### 2.3 Param 类型

```auto
type Param {
    name str
    type_name str
}
```

### 2.4 构造辅助函数

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
```

### 2.5 `ast_to_string()` — 输出 S-expression

递归将 AST 转为可打印字符串，用于验证：
- `IntExpr` → `"42"`
- `StrExpr` → `"\"hello\""`
- `IdentExpr` → `"x"`
- `BinExpr` → `"(+ 1 2)"`
- `CallExpr` → `"(print \"hello\")"`
- `DotExpr` → `"obj.field"`
- `FnStmt` → `"(fn main (params) (body ...))"`
- `LetStmt` → `"(let x 42)"`
- `IfStmt` → `"(if cond (body ...) (else ...))"`

## Step 3: parser.at — 核心解析器

### 3.1 Parser 类型

```auto
type Parser {
    tokens List    // List<Token>
    pos uint       // 当前位置
    len uint       // token 总数
}
```

### 3.2 Token 访问函数

```
parser_new(tokens) → Parser
parser_cur(p) → Token（当前位置的 token）
parser_kind(p) → TokenKind
parser_is(p, kind) → bool
parser_skip_newlines(p) → 跳过 Newline tokens
parser_skip_semi(p) → 跳过 Semi tokens
```

**注意**：`p.pos = p.pos + 1` 在 VM 中对 type 实例字段可变。每次 "consume" token 直接修改 `p.pos`。

### 3.3 入口点

```
parse_program(p) → List<ASTNode>
parse_top_stmt(p) → ASTNode  (dispatch to fn/let/var/type/expr)
parse_body(p) → List<ASTNode> (expect { ... })
parse_stmt(p) → ASTNode (dispatch to fn/let/var/return/if/for/expr)
```

### 3.4 语句解析器

| 函数 | 处理 | 关键逻辑 |
|------|------|----------|
| `parse_fn_decl(p)` | `fn name(params) ret { body }` | 跳过 fn → 读 name → 解析 params (name type, ...) → 可选返回类型 → parse_body |
| `parse_let_stmt(p)` | `let name [type] = expr` | 跳过 let → 读 name → peek 检查是否有 type → expect = → parse_expr |
| `parse_var_stmt(p)` | `var name [type] = expr` | 同 let |
| `parse_return_stmt(p)` | `return [expr]` | 跳过 return → 如果下一个不是 Newline/RBrace/Semi/EOF 则 parse_expr |
| `parse_if_stmt(p)` | `if cond { body } [else { body }]` | 跳过 if → parse_expr(cond) → parse_body → 可选 else（含 else if 递归） |
| `parse_for_stmt(p)` | `for ... { body }` | 跳过 for → 分支：for {body} / for name in range {body} / for cond {body} |
| `parse_type_decl(p)` | `type Name { fields }` | P0 简化版：只解析字段名+类型 |
| `parse_expr_stmt(p)` | `expr` | parse_expr → 包装为 ExprStmt |

### 3.5 Pratt 表达式解析器

**优先级表**（8 级，简化版 Rust parser 的 18 级）：

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

**赋值处理**：右结合，`parse_expr_prec(p, prec)`（不变 prec，不是 +1）。

### 3.6 原子表达式 `parse_atom(p)`

| Token | 生成节点 |
|-------|----------|
| `Int` | `int_node(text)` |
| `Uint` | `int_node(text)` |
| `Float` | `FloatExpr(text)` |
| `Str` | `str_node(text)` |
| `True` | `bool_node("true")` |
| `False` | `bool_node("false")` |
| `Nil` | `nil_node()` |
| `Ident` | `ident_node(text)` |
| `LParen` | 括号表达式：`parse_expr()` → expect `)` |
| `LSquare` | 数组：`parse_expr()` 列表 → expect `]` |
| `Sub` | 一元负：`parse_atom()` → `unary_node("-", operand)` |
| `Not` | 一元非：`parse_atom()` → `unary_node("!", operand)` |

### 3.7 `infix_prec(kind)` 优先级查表

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

## Step 4: VM 测试

### 测试文件 `test/vm/99_bootstrap/008_parser_hello/parser_hello.at`

测试解析 `"fn main() { print(\"hello\") }"` 并输出 S-expression。

由于 VM 测试框架会合并 `auto/lib/*.at` 文件（从 Plan 229 Phase 1.2 开始就如此），测试代码可以直接使用 `tokenize_list()`, `parser_new()`, `parse_program()`, `ast_to_string()` 等函数。

```auto
fn main() {
    let source = "fn main() {\n    print(\"hello\")\n}"
    let tokens = tokenize_list(source)
    let p = parser_new(tokens)
    let stmts = parse_program(p)
    var i = 0
    for i < stmts.len() {
        print(ast_to_string(stmts.get(i)))
        i = i + 1
    }
}
```

预期输出：
```
(fn main (params) (body (print "hello")))
```

### 验证顺序

1. `tokenize_list()` 正确返回 Token 列表（用 print 循环验证）
2. `parse_atom()` 处理字面量（`42`, `"hello"`, `true`, `x`）
3. `parse_expr()` 正确处理优先级（`1 + 2 * 3` → `(+ 1 (* 2 3))`）
4. `parse_stmt()` 处理 let/var/if/return
5. `parse_fn_decl()` 处理函数声明
6. `parse_program()` 处理完整程序
7. 完整的 hello world 测试

## 实施顺序

1. **Step 1**: `lexer.at` — 添加 `tokenize_list()` (~30 行)
2. **Step 2**: `ast.at` — 节点类型 + 构造函数 + `ast_to_string()` (~200 行)
3. **Step 3.1-3.2**: `parser.at` — Parser 类型 + token 访问函数 (~40 行)
4. **Step 3.5-3.6**: `parser.at` — Pratt 表达式解析器 + atom (~150 行)
5. **Step 3.3-3.4**: `parser.at` — 语句解析器 (~150 行)
6. **Step 4**: VM 测试文件 + 调试
7. 回归测试：`cargo test -p auto-lang --lib -- vm_file_tests`

预估代码量：`ast.at` ~200 行 + `parser.at` ~350 行 + `lexer.at` +35 行 ≈ 585 行 Auto 代码。

## 不在范围内

- 泛型类型参数 `<T>`
- `spec` / `ext` / `enum` / `tag` 声明
- 闭包 `(a, b) => a + b`
- F-string `f"..."`
- 模式匹配 `is x { ... }`（除了 token kind 分支）
- `use` 语句
- async/await
- 注解 `#[...]`
- a2r 编译（仅 VM 执行）
