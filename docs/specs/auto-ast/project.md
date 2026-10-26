# auto-ast

> **Status**: active
> 路径：`crates/auto-ast`  | 技术栈：Rust（依赖仅 auto-val + auto-lex + serde）
>
> Core AST 与序列化模型的单一来源；PLAN-841 起不再承载 UI 领域节点。

## 模块与依赖

`crates/auto-ast` 实现通用 `Code`/`Stmt`/`Expr`/`Type`/`Name`、Godot
`SceneDecl`、任务/事件/导入路径、模式及 `ToAtom`/`AtomWriter`/`ToAtomStr`/
`ToNode` 投影。normal 闭包仅 auto-val、auto-lex 与 serde，不含 Parser、
TypeStore、VM、UI 文法/compiler/runtime 或 Man。独立消费门为
`test/fixtures/modularity/ast-model-consumer/`（唯一依赖 auto-ast）。

`WidgetDecl`/`StoreDecl`/`ActionsDecl`/`ViewFragmentDecl`/`StyleRecipeDecl`/
`MsgDecl`/`ModelBlock`/`ViewBlock`/`UseWeb`、`ViewNode` 与 `ExtImport` 不在
本 crate；其唯一 owner 是 auto-ui-dialect 的 `UiStmt`/UI AST。`SceneDecl`
有真实 Godot 非 UI 消费者，保留在 Core。`Expr::View`、Core `OnEvents` 与
普通 `msg` 标识符也不按名称误删。

auto-lang 的 `crate::ast` 仅为同类型 re-export 门面；旧 `ast::ui` 和 Core
`Stmt` UI 兼容臂已退出。Parser 耦合的 `ParserExt` 与 markdown 测试仍归上层，
不在底层复制实现。

## 模型与序列化契约

- `AutoStr`、`Node`、`Op`、`Shared`、`Value` 与 `Pos` 由底层同源
  re-export；`AtomWriter` 返回 `auto_val::AutoResult`，不同于诊断层的
  `auto_diagnostics::AutoResult`。
- `Shared<T>` 只经显式 serde adapter 持久化，不以全局对象表、裸指针或
  side-table 模拟 owned AST。
- PLAN-799 的 36-case 冻结 golden 仍逐段逐字节使用，不重生成：Core/Scene
  段由 `crates/auto-ast/tests/v02_projection_replay.rs` 和
  `ast-model-consumer` 重放；9 个 UI 段由
  `test/fixtures/modularity/ui-ast-model-consumer/` 经 auto-ui-dialect
  重放。`UseWeb` 的 Display 单复数与 `ToAtom = "use.web"` 差额已修正并
  由同一 golden 锁定。
- 源注解等元数据是否进入 Display/ToNode/ToAtom 仍按原投影合同，不因
  领域拆分改变 Core 输出。

## 边界与已知限制

- `Type`（源码类型表达）与 `auto_val::Type`（运行时值类型）是不同类型；
  后者不在本模块根重复导出。
- Core 不提供 UI 领域 enum 的兼容臂或全局类型擦除载荷。需要 UI 时由上层
  显式组合 `Parser::parse_domain(&UiDialect)` 并取得 `UiCode`；未 lowering
  的领域产物不能进入 Core compile/VM。

## PLAN-841 来源与验证

Core AST 仅保通用节点与 Godot Scene；UI 专属 Stmt/模型归 auto-ui-dialect，投影 golden 保持。 规范与验收来源：[PLAN-841](../../plans/archive/841-ui-domain-ast-token-lowering-finalization.md)；复审及统一门证据见其归档记录。
