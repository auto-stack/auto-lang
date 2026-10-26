# auto-parser-api

> **Status**: active
> 路径：`crates/auto-parser-api`  | 技术栈：Rust（依赖仅 auto-ast + auto-lex + auto-diagnostics）
>
> 中立 Parser 端口、会话与 Core 方言契约的单一来源。

## 模块与依赖

`src/session.rs` 定义 `CompilerSession`/`Scenario`；`src/dialect.rs` 定义
Core `ExternalDialect`/`DialectPriority`；`src/context.rs` 定义对象安全
`ParseContext` 与 `ParseCheckpoint`；`src/domain.rs` 定义
`DomainParseContext`、`DomainDialect`、`DomainCode<D>` 与
`DomainStmt<D>::{Core,Domain}`。normal 闭包不含 Parser/lexer 实现、
TypeStore、VM、UI 文法/compiler/runtime 或 Man。

`DomainParseContext` 只提供通用 token/checkpoint/raw cursor/scope 与 Core
expr/type/stmt 委托。关联产出由方言自身拥有；端口不包含 UI 字段、
`Any`、裸指针、全局 side-table 或 UI codec。旧 `UiGrammar` 函数表、
`UiParseContext` 与 `ui_context` 模块已删除，不再以“轻 UI 端口”名义
把 UI 类型塞回 Core。

外部消费门包括 `dialect-api-consumer`（中立端口与显式 UI 领域组合）、
`ui-dialect-consumer`（真 Parser 驱动）、`ui-grammar-consumer`（轻 toy
端口驱动真实 UI 算法）以及 `language-sdk-consumer`（纯 Core）。

## Core 方言与快照契约

- Core `ExternalDialect` 保留 `matches`/`keywords`/Ident/token 三态语义；
  `Ok(Some)` 接管、`Ok(None)` 回滚让位、`Err` 报错且注册不丢。非 UI 旧
  `Dialect` + `ParserBridge` 公开面也继续有独立兼容控制。
- `ParseCheckpoint` 绑定 Parser 实例、槽位与单调 gen；按签发调用作用域
  失效。跨实例、过期或重复兑换在 restore 处受控拒绝且不动游标；外层
  快照在内层调用退出后仍有效。没有全局快照池或 unsafe 延寿。
- `probe_next_kind` 成功路径零副作用/幂等；失败路径如实遵循 lexer 的
  失败消费协议，不伪造游标不变或补记诊断。
- 跨界数据为 owned 文本与通用 AST 值；诊断单调追加，checkpoint 不承诺
  符号表等所有副作用的事务回滚。

## 领域组合契约

`Parser::parse_domain(&D)` 直接返回 `DomainCode<D::Output>`，不经过 Core
`Stmt` 的 UI 兼容臂。UI 生产组合为 `D=UiDialect`、`Output=UiStmt`，再由
UI owner 校验/lowering 后进入类型、执行与生成器。未组合 UI 方言的裸
`parse()` 对排他 UI 声明确定性拒绝；Core 同名语法保持合法。

外部 model 子集以 `DomainDialect<Output=UiStmt>` 实现；显式
`ExternalUiDialect` 先咨询外部 model，再委托 `UiDialect`。它不再把
`ModelBlock` 塞入 Core `Stmt`，也不依赖进程级注册表或 Override/Append
优先级。Core 闭包/裸块不自动承接 UI 领域语句，widget 内部结构仍由 UI
文法自己的结构解析器处理。

## 边界与发行

旧三段派发表只承担 Core/非 UI 兼容，不是 UI 类型回流传输层。过渡期的
旧注册/优先级的测试差额列入 PLAN-842 适配器退役清单。auto-parser-api
仍在 792 受管交付闭包内，fresh/locked/重定位与缺源负控继续适用。
