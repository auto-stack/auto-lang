//! PLAN-806：UI msg 与 model 声明文法正式外置。
//!
//! 本 crate 是完整 msg/model 声明算法的**单一份实现**（自 auto-lang
//! `Parser::parse_msg_decl_inner` / `parse_model_block_inner` /
//! `parse_model_field_annotations` 逐字迁出，行为冻结见 PLAN-806 T-01
//! D1–D7）。只依赖 [`auto_parser_api`] 轻端口（其闭包为 auto-ast/
//! auto-lex/auto-diagnostics + auto-val 轻层），**不依赖** auto-lang，
//! 不复制 Parser/lexer/Pratt/类型文法——核心表达式与类型解析经端口
//! 委托（`ParseContext::parse_expr` / `parse_type`）。
//!
//! 生产接线（auto-lang 上层，薄委托）：
//! - 顶层：`UiDialect` builtin 派发段 `"msg"`/`"model"` 分支 →
//!   `Parser::parse_msg_decl` / `parse_model_block` → 本 crate；
//! - 结构路径：`parse_widget_decl` / `parse_store_decl` /
//!   `parse_component_fn_decl` 体内直连 inner → 本 crate（同一实现，
//!   无第二份算法）；
//! - import：新 Parser 经 `with_session` 默认装配后同源。
//!
//! 两个 model 合同面共用一份块循环与字段算法（T-01 D4）：
//! - [`ModelGrammar::Builtin`]：内建完整文法（注解 + `var` + name +
//!   [type] [= expr]，**无裸字段**）——生产默认；
//! - [`ModelGrammar::ExternalSubset`]：PLAN-778 冻结的外部子集（裸字段
//!   允许、注解让行 [`ModelSubset::NotClaimed`]）——仅在显式 pilot
//!   策略下启用（默认 false，不构成默认文法漂移）。
//!
//! 行为冻结要点（迁出时未收紧/未扩展）：
//! - msg：旧 `msg Name {` 名字读弃（Plan 448 A）；quoted variant 合同名；
//!   多 payload（Plan 043 M5 #1）；named payload 经端口零副作用原语
//!   `probe_next_kind` 做原始单 token 探测（Plan 435 挂账②；成功路径
//!   零副作用/幂等、失败路径遵循旧回推前瞻失败消费协议——与旧 lexer
//!   单 token 回推逐字同构，F806-R2-01/R3-01 冻结形态）；
//! - model：`#[primary]`/`#[Consume]`/`#[Provide]`/`#[NavParam]` 注解
//!   （Plan 119/05-Nav），未知注解错误文本与 span 逐字保持；`var`
//!   可变旗标（Plan 130）；空行/逗号宽容；
//! - 错误：关键字消费复刻 `Parser::expect_ident`（"Expected 'x', got
//!   'y'"、纯文本比较、无 kind 检查——桥面 `expect_keyword` 的文本与
//!   检查语义不同，T-01 D3）；`expect` 族经桥面走 `Parser::expect`
//!   同一实现；未知注解经 `syntax_error`（`SyntaxError::Generic` 同文
//!   消息 + `pos_to_span(cur.pos)` 同位）。

pub mod declarations;
pub mod grammar;
// PLAN-814 R3-03b：UI 文法依赖回归测试组（随迁自 auto-lang-core 门控测试；
// dev-dep 驱动，仅测试编译）。
#[cfg(test)]
mod regression;

pub use declarations::{
    parse_model_block, parse_model_block_subset, parse_msg_decl, ModelGrammar, ModelSubset,
};
pub use grammar::UiParseContext;
// PLAN-814 R3-03b：SDK 函数表端口单源构造器（crate 根路径）。
pub use grammar::ui_grammar;
