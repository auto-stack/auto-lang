# auto-ast

> **Status**: active
> 路径：`crates/auto-ast`  | 技术栈：Rust（依赖仅 auto-val + auto-lex + serde）
>
> PLAN-799 交付：AST 模型单一来源 crate——`Code`/`Stmt`/`Expr`/`Type`/
> UI 兼容节点与 `ToNode`/`ToAtom`/`AtomWriter` 序列化行为自 auto-lang
> 逐字抽取（基面 143aff0a2），auto-lang 保留薄门面。本 spec 为该冻结增量的
> canonical 沉淀。

## 模块与依赖

`crates/auto-ast` 是 AutoLang AST 数据与类型表达的唯一实现：模块根
`src/ast.rs`（`Code`/`Stmt`/`Expr`/`Name`=`AutoStr`、traits `ToAtom`/
`AtomWriter`/`ToAtomStr`（blanket）/`ToNode`）+ 32 个子模块（`types`/`fun`/
`task`/`ui`/`cover`/`module_path`/`on` 等——声明、类型表达、UI 兼容节点、
模式、任务、事件、导入路径）。normal 依赖闭包仅 auto-val（轻值/Node/Op/
Value/Shared——模型字段与投影的原语）、auto-lex（`token::Pos`，call/task
源位置）、serde（`fun.rs` 手工 `Serialize for Fn`）——无 auto-lang/Parser/
TypeStore/推断/VM/renderer/Man 反向边（module-boundaries 域 `lang-core`，
规则 `lang-core-no-monolith-no-man-no-tools`）。外部消费门：
`test/fixtures/modularity/ast-model-consumer/` 独立 manifest（唯一依赖
auto-ast）构建+测试即独立可消费证据（V01）。

auto-lang 侧兼容出口：`crate::ast` 模块为单行 glob 门面
（`pub use auto_ast::ast::*`，同一类型身份，`pub mod ui/cover/fstr` 路径与
`auto_lang::ast::*` 旧路径全部保持，跨 crate 互传无转换副本）。解析便利层
不进底层：`ParserExt` trait 留 `auto_lang::parser`（旧导入路径不变），其
对 `Code`/`Name`/`Expr`/`Is`/`Arrow`/`OnEvents` 的六类 impl（含 OnEvents
`skip_check` 旧语义）与 Parser 耦合测试（`test/ast/*.test.md` markdown
oracle）归上层 `auto_lang::ast_parse_ext`（私有模块；旧 `ast::parsers` 本
就非 pub，无外部路径消费）。不保留第二份 AST 实现。

## 模型与序列化契约

- 轻值原语同源 re-export：`ast` 模块根 `pub use auto_val::{AutoStr,
  Node as AutoNode, Op, Shared, Value}` 与 `pub use auto_lex::token::Pos`
  ——独立消费方仅依赖 auto-ast 即可构造 AST 值（`Op`/`AutoStr`）与消费
  `to_node()` 产物（`Node`/`Value`）。**导出面以"不与其他常用 glob 面
  产生不同 item 撞名"为限**（F-799-R1-01：`auto_val::AutoResult` 曾公开
  再导出，与 `auto_lang::error::AutoResult` 是不同 item，破坏旧门面
  `ast::* + error::*` glob 兼容——E0659；已回退为模型内部私有，永久回归
  `crates/auto-lang/tests/ast_facade_glob_compat.rs` 钉住）。语法级
  `Type`/`Node`/`Arg` 保持模型原名；`auto_val::Arg`/`auto_val::Type` 因
  模型已有同名项不再导出。
- 两个 `AutoResult` 是不同类型且各自绑定不变：模型 `AtomWriter` 返回
  `auto_val::AutoResult`（`Result<T, Box<dyn Error>>`）；`ParserExt::parse`
  返回 auto-diagnostics 的富 `AutoResult`（`Result<T, AutoError>`）。
- 投影保持基线：PLAN-799 V02 冻结 golden（36 case——core 语句族/6 表达式
  变体/泛型与三声明/5 模式/task+on 事件/3 导入/10 UI 兼容节点/嵌套+
  comptime，每 case Display+ToAtom+ToNode 三投影；旧实现生成，SHA256
  1e74a8df…，`docs/plans/attachments/799/v02-golden.txt`）由
  `crates/auto-ast/tests/v02_projection_replay.rs` 永久重放（日常档），
  独立消费方形态由 ast-model-consumer fixture 同源重放；Parser→AST→投影
  链路的语义 oracle 由上层 `ast_parse_ext::markdown_tests`
  （`test/ast/*.test.md`）钉住。
- 序列化不改变的元数据字段（`Code.file_attrs`/`Store.attrs`/`is_pub` 等）
  保持原语义：源码级注解仅转译器消费，Display/ToNode/ToAtom 不输出。

## 边界与已知限制

- 本 crate 包含原 UI 兼容 AST 变体（`Stmt::WidgetDecl`/`StoreDecl`/
  `ViewFragmentDecl`/`StyleRecipeDecl`/`ActionsDecl`/`MsgDecl`/`ModelBlock`/
  `ViewBlock`/`SceneDecl`/`UseWeb` 与 `ast::ui` 视图树模型）——属无
  renderer 依赖的共享兼容模型，**不宣称纯 Core AST**；其终态 UiAst 分离
  是另立合同（777 焊点表 `weld-ast-ui-variants`，R8；Rust 下游 crate 无法
  给上游 enum 增变体是闭合 `Stmt` enum 的结构性原因）。
- 剩余前端边（未迁事实，owner 见 frontend/overview）：`Parser` 仍持有
  Lexer/TypeStore/InferenceContext/方言表（含 UI 文法），`crate::ui` 五目录
  与 ui_gen 经 `crate::ast` 门面消费模型——Parser/type-store/推断/UI 文法
  外置不在本 spec 范围。
- `Type`（源码级类型表达，本 crate）与 `auto_val::Type`（运行时值类型）
  是两个不同类型，模型代码内以限定路径区分；本 crate 不再导出
  `auto_val::Type`（与模型 `Type` 撞名）。
