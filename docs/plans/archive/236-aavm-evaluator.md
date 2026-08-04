# Plan 235: AAVM Tree-Walking Evaluator + Type Inference

## Context

Plan 234 完成了 AAVM parser P1（10 个高优先级特性 + 29 个共享测试）。AAVM 能将源码 tokenize → parse 成 AST，但 AST 节点只存储 S-expression 字符串（如 `"(+ 1 2)"`），无法被执行。

**核心问题**：AST 没有「结构化数据」——子节点没有被保存为引用，全部被拼接成了字符串。evaluator 无法从字符串重建程序结构。

**目标**：让 AAVM 能执行简单的 AutoLang 程序（算术、变量、函数、条件、循环）。

**策略**：Tree-Walking Evaluator（树遍历解释器），不生成 bytecode，直接遍历 AST 执行。这是最简路径，避免了在 AutoLang 中实现 bytecode emitter 的巨大复杂度。

## 实施范围

| Phase | 内容 | 估计代码量 |
|-------|------|-----------|
| 0 | AST 结构化重构 | ast.at ~60 行, parser.at ~200 行 |
| 1 | 合并到 Phase 2（不单独实现） | — |
| 2 | Tree-Walking Evaluator（含基础类型跟踪） | eval.at ~600 行 (新文件) |
| 3 | 集成测试 | 10 个新测试目录 |

## Phase 0: AST 结构化

### 问题

当前 `ASTNode` 的构造函数把所有子节点信息拼接成字符串存入 `value`：

```auto
fn bin_node(op str, left ASTNode, right ASTNode) ASTNode {
    var s = "(" + op + " " + left.value + " " + right.value + ")"
    ASTNode(NodeKind.BinExpr, s, ...)  // value = "(+ 1 2)", 子节点丢失
}
```

evaluator 无法从 `"(+ 1 2)"` 重建程序结构。

### 解决方案

**保留 `value` 字段**（向后兼容，用于 `ast_to_string`），同时在 `left`/`right`/`children` 等列表字段中存储实际子节点引用。

#### ast.at 改动

添加 `name` 字段到 `ASTNode`：

```auto
type ASTNode {
    kind NodeKind
    value str       // 保留：调试用 S-expression
    name str        // 新增：标识符名称/字面量文本
    children List   // 通用子节点列表
    left List       // 左操作数/初始化表达式
    right List      // 右操作数
    op str          // 运算符
    params List     // 参数列表 (List of Param)
    type_name str   // 类型标注
    cond List       // 条件表达式
    else_body List  // else 分支语句列表
}
```

修改所有构造函数，**同时**设置结构化字段和 value 字符串：

| 构造函数 | name | left | right | children | value (调试) |
|----------|------|------|-------|----------|-------------|
| `int_node(val)` | val | [] | [] | [] | val |
| `str_node(val)` | val | [] | [] | [] | `"val"` |
| `bool_node(val)` | val | [] | [] | [] | val |
| `ident_node(n)` | n | [] | [] | [] | n |
| `bin_node(op, l, r)` | "" | [l] | [r] | [] | "(op l r)" |
| `unary_node(op, o)` | "" | [o] | [] | [] | "(op o)" |
| `call_node(callee, args)` | "" | [] | [] | [callee, ...args] | "(callee ...)" |
| `dot_node(obj, field)` | field | [obj] | [] | [] | "obj.field" |
| `fn_node(name, params, ret, body)` | name | [] | [] | body_stmts | "(fn ...)" |
| `store_node(kind, n, t, expr)` | n | [expr] | [] | [] | "(let/var n ...)" |
| `return_node(expr)` | "" | [expr] | [] | [] | "(return ...)" |
| `if_node(cond, body, else)` | "" | [] | [] | body | "(if ...)" |
| `forin_node(n, range, body)` | n | [range] | [] | body | "(forin ...)" |
| `for_node(cond, body)` | "" | [] | [] | body | "(for ...)" |
| `expr_stmt_node(expr)` | "" | [expr] | [] | [] | expr.value |
| `closure_node(params, body)` | params | [body] | [] | [] | "(closure ...)" |
| `object_node(pairs_str)` | "" | [] | [] | [] | "(object ...)" |
| `pair_node(k, v)` | k | [v] | [] | [] | "(pair k v)" |
| `nil_node()` | "" | [] | [] | [] | "nil" |

#### parser.at 改动

`parse_body` 改为返回 `List`（语句节点列表），而不是 `str`：

```auto
// 旧：fn parse_body(p Parser) str
// 新：fn parse_body(p Parser) List
fn parse_body(p Parser) List {
    p.pos = p.pos + 1  // skip '{'
    parser_skip_nl_semi(p)
    var stmts = empty_list()
    for parser_kind(p) != TokenKind.RBrace && parser_kind(p) != TokenKind.EOF {
        var stmt = parse_stmt(p)
        if stmt.kind != NodeKind.NilNode {
            stmts.push(stmt)
        }
        parser_skip_nl_semi(p)
    }
    if parser_kind(p) == TokenKind.RBrace {
        p.pos = p.pos + 1
    }
    return stmts
}
```

`parse_fn_decl`：body 直接传 list 给 `fn_node`，不再拼字符串。

`parse_if_stmt`：cond 存 `cond`，body 存 `children`，else 存 `else_body`。

`parse_for_stmt` / `parse_forin_stmt`：类似改动。

**关键**：`ast_to_string()` 函数保持不变（仍然从 `value` 字段读取 S-expression），确保 29 个现有 parser 测试全部通过。

#### 验证

- 所有 29 个现有 parser 测试（009-037）必须继续通过
- 新测试：解析 `let x = 1 + 2`，验证 `node.left.get(0).kind == BinExpr`

## Phase 1: Type 推断（最小化版）

**目标**：只实现让 evaluator 能正确执行简单程序所需的类型信息。不做泛型、不做 unification、不做约束求解。

长期目标是用 Auto 版编译器替代 Rust 版，但初版只需要：
- 字面量类型识别（int/float/bool/str）
- 变量绑定的类型跟踪（`let x = 42` → x 是 int）
- 函数签名注册（`fn add(a int, b int) int` → 记录返回类型）
- 算术运算类型传播（int + float → float）

### 合并到 eval.at（不单独建文件）

Type inference 不单独建 type.at，直接集成在 evaluator 中。evaluator 在执行时自然获得类型信息：
- `let x = 42` → x 绑定 int 值 42，VM 自动知道类型
- `fn add(a int, b int) int` → 注册函数时记录返回类型
- 算术运算 → VM 自动处理 int/float 混合

**理由**：tree-walking evaluator 直接操作 VM 原生值（int、str、List），不需要显式的 type representation。VM 的动态类型系统已经能区分 int/float/bool/str。单独建 TypeInfo 体系增加了复杂度但没有带来实际收益——evaluator 拿到值就知道类型了。

### 唯一需要类型信息的场景

1. **函数返回值**：caller 需要知道函数的返回类型吗？不需要——evaluator 直接返回求值结果。
2. **变量赋值**：需要类型检查吗？初版不做——直接赋值。
3. **类型标注**：`let x int = 42` 中的 `int`？初版忽略——值已经是 int 了。

**结论**：Phase 1 合并到 Phase 2 evaluator 中，不单独实现 type inference。后续 Plan 236 再加完整类型推断。

## Phase 2: Tree-Walking Evaluator

### 新文件：`auto/lib/eval.at`

```auto
type EvalEnv {
    globals Map    // 全局变量: name -> value
    scopes List    // 作用域栈: List of Map
    fn_defs Map    // 函数定义: name -> ASTNode (fn_stmt node)
    output str     // 累积输出
}
```

### 核心函数

```
fn eval_new() EvalEnv
fn eval_program(env EvalEnv, stmts List)
fn eval_stmt(env EvalEnv, node ASTNode)    // 返回值 + return 标志
fn eval_expr(env EvalEnv, node ASTNode)    // 返回值
```

### 表达式求值

| NodeKind | 求值逻辑 |
|----------|---------|
| IntExpr | `node.name` 解析为 int |
| StrExpr | 返回 `node.name` |
| BoolExpr | `"true"` → 1, `"false"` → 0 |
| IdentExpr | `eval_lookup(env, node.name)` |
| BinExpr | 递归求值 left/right，按 op 运算 |
| UnaryExpr | 递归求值 operand，取反/取非 |
| CallExpr | 查找函数定义，push scope，绑定参数，执行 body，pop scope |
| DotExpr | 求值对象，访问字段 |
| FStrExpr | 拼接插值部分 |

### 语句求值

| NodeKind | 求值逻辑 |
|----------|---------|
| LetStmt/VarStmt | 求值初始化表达式，绑定变量 |
| FnStmt | 注册到 fn_defs（不立即执行） |
| ReturnStmt | 求值表达式，设置 return 标志 |
| IfStmt | 求值条件，执行对应分支 |
| ForStmt | 循环求值条件，执行 body |
| ForInStmt | 遍历 range/list，执行 body |
| ExprStmt | 求值表达式，丢弃结果 |

### 内置函数

evaluator 内置支持（不走 fn_defs）：
- `print(value)` → 累积到 output
- `List.new()` → 创建空 List
- `list.push(item)` → 追加元素
- `list.get(index)` → 取元素
- `list.len()` → 列表长度
- `str.len()` → 字符串长度
- `str.sub(start, end)` → 子字符串

### Return 处理

使用 VM 的 Map 来传递 return 信号：

```auto
fn eval_stmt(env EvalEnv, node ASTNode) Map {
    // 返回 Map: {"value": v, "is_return": 0 或 1}
    // 调用方检查 is_return，如果为 1 则向上传播
}
```

### 函数调用流程

```
1. 查找函数名 -> 获取 FnStmt ASTNode
2. eval_push_scope(env)
3. 逐个绑定参数: eval_bind(env, param_name, arg_value)
4. 逐个执行 body 中的 stmt
5. 如果某个 stmt 返回 is_return=1, 提取 value, 停止执行
6. eval_pop_scope(env)
7. 返回函数结果值
```

**前向引用**：`eval_program` 先做一次扫描，注册所有 `FnStmt`，再执行 main。

## Phase 3: 集成测试

### 新建 10 个测试目录

在 `crates/auto-lang/test/vm/99_bootstrap/` 下：

| # | 目录 | 测试内容 |
|---|------|---------|
| 038 | eval_arithmetic | `1 + 2 * 3` → 7 |
| 039 | eval_variable | `let x = 10; print(x + 5)` → 15 |
| 040 | eval_fn_call | `fn double(n) { n * 2 }; print(double(21))` → 42 |
| 041 | eval_if_else | `if 1 > 0 { print(1) } else { print(0) }` → 1 |
| 042 | eval_for_loop | `var s = 0; for i in 0..5 { s = s + i }; print(s)` → 10 |
| 043 | eval_recursion | `fn fib(n) { if n <= 1 { return n } return fib(n-1) + fib(n-2) }; print(fib(10))` → 55 |
| 044 | eval_string | `let s = "hello"; print(s.len())` → 5 |
| 045 | eval_list | `let list = List.new(); list.push(1); list.push(2); print(list.get(1))` → 2 |
| 046 | eval_closure | `let f = x => x + 1; print(f(41))` → 42 |
| 047 | eval_multi_fn | 多函数协作：`fn add(a, b) { a + b }; fn main() { print(add(20, 22)) }` → 42 |

### 测试模式

每个测试的 `.at` 文件调用 `run_eval(source)` 函数：

```auto
fn main() {
    let result = run_eval("fn main() { print(1 + 2) }")
    print(result)
}
```

`.expected.out` 中是预期输出（如 `3`）。

## 验证

```bash
# Phase 0 验证：现有测试不回归
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap
cargo test -p auto-lang --lib -- test_rust_99_bootstrap

# Phase 2 验证：evaluator 测试
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap_038

# 完整回归
cargo test -p auto-lang --lib -- vm_file_tests
```

## 文件改动总览

| 文件 | 变更类型 | 估计行数 |
|------|---------|---------|
| `auto/lib/ast.at` | 修改 | +60 行 |
| `auto/lib/parser.at` | 修改 | +200 行 |
| `auto/lib/eval.at` | 新建 | ~600 行 |
| `crates/auto-lang/src/tests/vm_file_tests.rs` | 修改 | +20 行 |
| `crates/auto-lang/test/vm/99_bootstrap/038-047_*/` | 新建 | 10 个目录 |

## 实施顺序

```
Phase 0: AST 结构化（先做，是后续所有工作的基础）
  ├── 修改 ast.at（加 name 字段 + 修改构造函数）
  ├── 修改 parser.at（parse_body 返回 List）
  └── 验证：29 个现有 parser 测试通过

Phase 2: Evaluator（Phase 1 合并到此）
  ├── 新建 eval.at（含 EvalEnv、eval_expr、eval_stmt、eval_program）
  └── 验证：evaluator 能执行简单表达式

Phase 3: 集成测试
  ├── 新建 10 个测试目录
  ├── 注册到 vm_file_tests.rs
  └── 验证：全部测试通过
```

## 风险与缓解

1. **Parser 重构风险**：parse_body 从返回 str 改为返回 List，影响面大。缓解：逐个 statement 类型修改，每步跑测试。
2. **值表示**：AutoLang 没有 union type，evaluator 的返回值依赖 VM 动态类型。缓解：用 Map 包装返回值 + return 标志。
3. **性能**：tree-walking 对递归函数（如 fib(10)）可能慢。缓解：Phase 2 只要求正确性，不要求性能。
4. **字符串拼接 bug**：Plan 234 中发现 VM 在长链字符串拼接中调用函数会丢失返回值。缓解：所有函数调用结果先赋值给临时变量。
