//! PLAN-799: AST 模型单一来源（`auto_ast::ast`）。
//!
//! Auto 语言的 AST 数据与类型表达（`Code`/`Stmt`/`Expr`/`Type`/UI 兼容
//! 节点/`ModulePath` 等）及其序列化行为（`ToNode`/`ToAtom`/`AtomWriter`）。
//! 自 auto-lang 逐字抽取（见 docs/plans/799-frontend-ast-model-extraction.md），
//! 无 Parser/TypeStore/类型推断/VM/转译/renderer/Man 依赖；解析便利层
//! （`ParserExt` 六 impl）留在 auto-lang 上层 `ast_parse_ext`，旧
//! `auto_lang::ast::*` 路径经薄门面保持同一类型身份。
//!
//! 本 crate 当前包含原 UI 兼容 AST 变体（`ast::ui` 等）——属无 renderer
//! 依赖的共享兼容模型，不宣称纯 Core AST；其终态 UiAst 分离另立合同
//!（777 焊点表 weld-ast-ui-variants，R8）。

pub mod ast;
