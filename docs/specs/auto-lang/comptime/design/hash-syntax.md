# `#` 语法与 AST 节点

## 范围

comptime 的语法表面（四种 `#` 构造）、lexer/parser 落点、AST 数据结构，以及设计文档规定的作用域规则。
不含求值语义（见 `comptime-eval.md`）与管线集成（见 `ctee-pipeline.md`）。

## 原则

- **显式优于隐式**：`#` 前缀即编译期标记，程序员一眼区分 Meta/Runtime（设计文档 §1）。
- **语句级提升**：`#` 作用于整个 `if-elif-else` / `for` / `is` 结构，后续分支自动继承编译期属性，
  无需重复加 `#`；块内代码默认"发射"（Emit）进运行时 AST（设计文档 §1.3、§2.1）。
- **不再单造关键字**：`const`/`type` 复用现有关键字承载编译期声明，不引入 `#let`（设计文档 §2.2）。

## 四种构造

| 语法 | 用途 | AST 节点 |
|---|---|---|
| `#if cond { } elif … else { }` | 条件编译，整结构裁剪 | `Stmt::HashIf(HashIf)` |
| `#for i in 0..N { }` | 编译期循环展开 | `Stmt::HashFor(HashFor)` |
| `#is expr { pat => { } else => { } }` | 编译期匹配 | `Stmt::HashIs(HashIs)` |
| `#{ expr }` | 编译期求值，结果替换为字面量 | 语句位 `Stmt::HashBrace`；表达式位 `Expr::Comptime(Box<HashBrace>)` |

## 数据结构（ast/comptime.rs）

- `HashIf { cond, then_block, else_block: Option<HashIfElse> }`；`HashIfElse = Block(Body) | ElseIf(Box<HashIf>)`
  ——`elif` 表示为嵌套 `HashIf`，变换时递归处理。
- `HashFor { var, iter, body }`——迭代源是任意表达式，求值后由 `value_to_iter` 解释（目前只认 `Int`）。
- `HashIs { target, branches: Vec<HashIsBranch> }`；分支三种：`EqBranch(模式表达式, Body)`、
  `IfBranch(条件表达式, Body)`、`ElseBranch(Body)`。注意：实现是**值相等匹配**，不是类型模式匹配
  （`mod.rs` 注释中的 "Type pattern matching" 说法与设计/实现均不符）。
- `HashBrace { expr }`——单一表达式，求值后回写为字面量。

## 解析落点

- lexer.rs 对 `#` 前瞻消歧：`#if`/`#is`/`#for`/`#{` 各成独立 token，`#[` 仍归 `Hash` 注解（plan-095 Phase 1）。
- parser.rs:1807 在 primary 表达式处产出 `Expr::Comptime`；parser.rs:6285 附近在语句位产出 `Stmt::HashBrace`。

## 作用域规则（设计规定）

- **向下可见**：`#` 块内可读取外部 `const` 常量。
- **隔离性**：`#{ ... }` 块内 `let/var` 为局部，计算结束即销毁，不污染全局命名空间。
- 实现现状：隔离性靠"表达式转源码交独立 VM 实例执行"天然成立；`const` 向下可见未做专门机制
  （VM 环境不含外层符号表），属未完全兑现的设计条款。

## 显式非目标

- 不做文本宏替换：`#` 是 AST 结构化操作，不操作 Token 流。
- 不做编译期类型反射：`type_of(val).fields` 之类属设计 Phase 4 的 `std.meta` 规划，未实现。
- `#{ }` 设计为多语句求值块（"最后一行为返回值"），实现只支持单表达式。

> 来源: docs/design/raw/compile-time-execution.md、docs/plans/old/095-compile-time-execution-engine.md、crates/auto-lang/src/ast/comptime.rs、crates/auto-lang/src/parser.rs、crates/auto-lang/src/lexer.rs
