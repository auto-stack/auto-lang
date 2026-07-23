# 方言体系（Dialect）

## 范围

轴 A（语法子集扩展）的方言机制：哪些关键字在哪个场景下生效、命中后如何解析。
不管执行/转译（轴 B）与程序形态（轴 C）——那是诊断文档另外两轴的事。

## 原则

- **场景驱动**：方言以 `matches(&CompilerSession)` 判定生效与否（如 `Scenario::UI`），
  同一关键字在不同场景可有不同解析。
- **产物不变式**：方言解析出的节点仍是基础 `Stmt` 的合法变体，下游消费者
  （typeck/trans/vm）类型签名不变。
- **核心 parser 零改动**：新方言实现 trait 后在 `Parser::build_dialects` 按 session
  注册即可，派发逻辑不散落。

## 细节

- trait 形态（dialect.rs）：
  - `keywords()` 列出接管的**语句起始**关键字（仅 `TokenKind::Ident` 路径，如
    `widget`/`msg`/`model`）；这些关键字在所属场景视为"上下文关键字"而非普通标识符。
  - `try_parse_stmt(parser, keyword) -> AutoResult<Option<Stmt>>`：`Ok(Some)` 已处理、
    `Ok(None)` 让位给下一个方言/默认路径、`Err` 报错。
  - `try_parse_token_stmt(parser, kind)`：接管真实 TokenKind（如 `TokenKind::View`/
    `TokenKind::On`），默认不接管。
- 派发实现（parser.rs:437/458）：`try_dialect_stmt`/`try_token_dialect_stmt` 用
  `mem::take` 把方言表移出 `self`，遍历局部副本后放回——规避"方言方法借 `&mut Parser`
  而方言表借 `&self.dialects`"的自引用冲突。新增方言派发点须沿用此模式。
- 注册（parser.rs:419 `build_dialects`）：当前仅 `UiDialect`，`Scenario::UI` 时装配。
- `view` 的两义性：Core 语言里是参数模式关键字（`fn foo(view x int)`），UI 场景在
  语句位置是 view 块/fragment——由 `try_parse_token_stmt` 按场景分派
  （dialect/ui.rs 头注）。

## 不变量

- 方言只在**语句位置**被查询，表达式中的同名标识符不受影响。
- 方言表按注册顺序即优先级查询，先返回 `Ok(Some)` 者胜出。
- 方言之间不共享状态；所有可变状态经 `&mut Parser` 传递。

## 显式非目标

- 不做方言间的语法组合/继承机制——每个方言独立声明关键字全集。
- 不管轴 B（执行后端统一）与轴 C（AURA IR 程序形态），见诊断文档 §3/§4.3。
- 不为方言引入独立 AST 类型层——产物必须落回基础 `Stmt` 变体。

> 来源: docs/design/dialect-extension-diagnosis.md §4/§6.1；代码核对 crates/auto-lang/src/dialect.rs、dialect/ui.rs、parser.rs:174-470
