//! auto-diagnostics —— AutoLang 诊断基础 crate（PLAN-793 自 auto-lang 抽取）。
//!
//! 单一来源：`AutoError` 聚合器与语法/类型/名称/运行时/comptime/警告各族
//! 诊断、source-attach 包装（`SyntaxErrorWithSource`/`LexerErrorWithSource`/
//! `MsgWithSource`）、投影与建议助手（`attach_source`/`print_error`/
//! `format_error`/`find_best_match`）只在此定义；auto-lang 经
//! `crate::error` 兼容门面 re-export 同一类型（类型身份跨 crate 互传，
//! 不保留第二份实现）。
//!
//! 依赖边界（module-boundaries 域 `lang-core`）：仅依赖 auto-lex
//! （`LexerError`/`LexError` 单一定义，PLAN-789）与 auto-val（`AutoStr`，
//! 即 auto-lang `ast::Name` 的同一底层类型）及 miette/thiserror 诊断基建；
//! **禁止**依赖 auto-lang/auto-man/UI，也不携带 AST/Parser/推断类型
//! （反向边即架构违规，闭包由 test/fixtures/modularity/diagnostics-consumer
//! 钉住）。
//!
//! 行为冻结：诊断 code/severity/help/labels/span/Display/带源渲染以
//! PLAN-793 V02 冻结 golden（`docs/plans/attachments/793/
//! v02-golden-frozen.json`，基面 579f6dec0 旧实现生成，SHA256
//! `e97b9450…2a09d`）为对拍基线。

pub mod error;
