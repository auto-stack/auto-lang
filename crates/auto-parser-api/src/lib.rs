//! PLAN-801：方言接口独立与 Parser 依赖倒置。
//!
//! 本 crate 是外部文法实现与语言核心之间的**轻端口合同**：
//!
//! - [`session`]：轻 `CompilerSession`/`Scenario` 单一定义（自 auto-lang
//!   迁入；`auto_lang::session` 为薄 re-export 门面，类型身份不变）。
//! - [`dialect`]：[`ExternalDialect`] trait（外部方言实现面）与
//!   [`DialectPriority`] 注册优先级（Override 前插 / Append 后接，PLAN-778
//!   冻结语义）。
//! - [`context`]：对象安全端口 [`ParseContext`]——PLAN-778 `ParserBridge`
//!   桥面的 dyn 形态（游标 / checkpoint / 核心表达式与类型解析委托 /
//!   场景与语句上下文查询 / 语法错误构造），外加不透明快照凭据
//!   [`ParseCheckpoint`]（有作用域句柄表后端，T-01 D3）。
//! - [`ui_context`]：完整 UI 文法端口 [`UiParseContext`]（PLAN-810 冻结
//!   面；PLAN-814 R3-03b 自 auto-ui-dialect 迁入——语言核心 SDK 函数表
//!   派发需双侧共同可见，孤儿规则下唯一合法位置）与 UI 文法函数表
//!   [`UiGrammar`]（SDK 内置 UI 方言的装配注入端口）。
//!
//! 依赖方向（T-01 D1）：外部方言 crate → 本 crate → auto-ast/auto-lex/
//! auto-diagnostics（lang-core 轻层）。**不依赖** auto-lang 或任何 Parser/
//! VM/UI/Man 实现；解析端口由 auto-lang 上层实现（`ParserBridge` 实现
//! `ParseContext`），生产派发经 auto-lang `Parser::register_external_dialect`
//! 进入与旧 trait 同一张三段有序派发表（Override→builtin→Append）。
//!
//! AST/词法/诊断沿用单一定义（PLAN-799/789/793 抽取物），无镜像 DTO；
//! 跨 crate 边界只传 owned 文本与现有 AST 值，不返回 Parser/lexer/scope
//! 引用。旧 [`auto_lang::dialect::Dialect`] + [`ParserBridge`] 公开面在
//! auto-lang 保持不变（PLAN-778 冻结面零漂移）。

pub mod context;
pub mod dialect;
pub mod session;
pub mod ui_context;

pub use context::{ParseCheckpoint, ParseContext};
pub use dialect::{DialectPriority, ExternalDialect};
pub use session::{CompilerSession, Scenario};
pub use ui_context::{UiGrammar, UiParseContext};

// 轻层门面 re-export：端口方法签名所暴露的类型（AST 值 / token / 诊断）
// 的所属模块，模块名与旧 `auto_lang::{ast,token,error}` 对齐——外部方言
// 从 auto_lang 迁到本 crate 近似路径改名。类型身份即 auto-ast/auto-lex/
// auto-diagnostics 单一实现（PLAN-799/789/793），无镜像 DTO。
pub use auto_ast::ast;
pub use auto_diagnostics::error;
pub use auto_lex::token;
