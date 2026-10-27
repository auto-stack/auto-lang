//! PLAN-799: AST 模型单一来源（`auto_ast::ast`）。
//!
//! Auto 语言的 Core AST 数据与类型表达（`Code`/`Stmt`/`Expr`/`Type`/
//! `SceneDecl`/`ModulePath` 等）及其序列化行为（`ToNode`/`ToAtom`/`AtomWriter`）。
//! 自 auto-lang 逐字抽取（见 docs/plans/799-frontend-ast-model-extraction.md），
//! 无 Parser/TypeStore/类型推断/VM/转译/renderer/Man 依赖；解析便利层
//! （`ParserExt` 六 impl）位于 `auto_lang_core::ast_parse_ext`。
//! 自 auto-lang 0.2 起直接使用 `auto_ast::ast` 或 SDK 同源入口。
//!
//! UI 领域节点由 auto-ui-dialect 拥有；本 crate 不承载 UI 兼容变体。

pub mod ast;
