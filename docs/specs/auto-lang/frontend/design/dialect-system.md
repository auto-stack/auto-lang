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
- 派发实现（parser.rs `try_dialect_stmt`/`try_token_dialect_stmt`）：方言表元素为
  Arc 共享句柄，派发时克隆 Arc 调用而非移出——`mem::take` 移表模式已废除
  （PLAN-778 R1：Err 提前退出会永久丢失整张方言表，且委托递归期间活跃方言
  不可见）。恢复不变量由派发器兜底：Ok(None) 后强制回滚游标+五旗标；Err 后
  恢复五旗标、游标留失败位置、注册不丢。新增方言派发点不得恢复移出式派发。
- 注册（`build_dialects`）：内建仅 `UiDialect`，`Scenario::UI` 时装配；
  外部方言经 `register_dialect`（旧 trait）/`register_external_dialect`
  （PLAN-801 新接口 `ExternalDialect`，经 `ExternalDialectAdapter` 协议转换）
  显式注册进同一张三段表（Override 前插/Append 后接），跨 `with_session` 保留。
- 新接口端口（PLAN-801，`crates/auto-parser-api`）：外部实现只见对象安全
  `ParseContext`（`&mut dyn` 形态的 778 桥面，由 `ParserBridge` 在上层实现）
  与不透明快照凭据 `ParseCheckpoint`（有作用域句柄表：一次性、跨上下文/
  过期受控拒绝、方言调用退出水位清表）。旧 trait + by-value
  `BridgeCheckpoint` 公开面保持不变。
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

> 来源: docs/design/dialect-extension-diagnosis.md §4/§6.1；代码核对 crates/auto-lang/src/dialect.rs、dialect/ui.rs、parser.rs（PLAN-801 增补：crates/auto-parser-api、dialect/external.rs、parser_bridge.rs 端口实现）
