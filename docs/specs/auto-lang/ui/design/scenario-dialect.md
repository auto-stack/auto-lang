# Scenario 与 UI 方言（条件关键字）

## 范围

编译器行为由项目场景（scenario）驱动，而非全局语言特性。本主题覆盖 `pac.at` 配置、`CompilerSession` 传递与 UI 关键字的条件激活机制。

## 机制

- **配置单一真源**：`pac.at` 声明 `scenario`（`ui`/`core`/`shell`）、后端目标与构建设置；LSP 初始化时读取它配置解析模式，诊断/hover/补全都尊重场景。
- **会话传递**：`session.rs:CompilerSession` 携带 `Scenario` 贯穿管线。
- **dialect 注册**：`dialect/ui.rs:UiDialect` 实现 `Dialect` trait，`matches()` 在 `scenario == Scenario::UI` 时生效。

## 关键字接管细节（以代码为准）

- `widget`/`msg`/`model` 是**普通标识符**（`TokenKind::Ident`），UI 场景下经 `Dialect::try_parse_stmt` 接管为声明语句。
- `view`/`on` 是**真实 TokenKind**，走 `try_parse_token_stmt`；`view` 与 core 语言的参数模式关键字（`fn foo(view x int)`）复用同一 token，靠语句位置区分。
- `view fn` 前缀为 view fragment（plan-367 P2-3）。

## 不变量

- core 场景下 `let widget = create_window()` 必须合法——UI 关键字零命名空间污染（ADR-03）。
- 场景判定只读 `CompilerSession`，解析器本身不硬编码 UI 规则。

## 演进记录

docs/design/08 描述的是"parser 直接检查 session 提升上下文关键字"；现实现已重构为 dialect 机制（docs/design/dialect-extension-diagnosis.md §6.1，`dialect/ui.rs` 头注）。以代码为准。

## 显式非目标

- 不为每个场景发明新语法——dialect 只接管既有标识符的语句位解析。
- LSP 的 scenario 同步细节不在本主题（属 LSP 模块）。

> 来源: docs/design/08-ui-systems.md §Scenario-Based Programming；crates/auto-lang/src/dialect/ui.rs；docs/design/dialect-extension-diagnosis.md §6.1
