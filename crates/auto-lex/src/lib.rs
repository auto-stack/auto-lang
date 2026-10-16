//! auto-lex —— AutoLang 词法基础 crate（PLAN-789 自 auto-lang 逐字抽取）。
//!
//! 单一来源：`Token`/`TokenKind`/`Pos` 与 `Lexer`/`LexerState` 只在此定义，
//! auto-lang 经 `crate::token` / `crate::lexer` 兼容出口 re-export 同一类型
//! （类型身份跨 crate 互传，不保留第二份实现）。
//!
//! 依赖边界（module-boundaries 域 `lang-core`）：仅依赖 auto-val（AutoStr）
//! 与 strum/miette/thiserror 诊断基建；**禁止**依赖 auto-lang/auto-man/UI
//! renderer（反向边即架构违规）。
//!
//! 行为冻结：token 序列、`keyword_kind`/`all_keywords`、Display、f-string
//! note（`set_fstr_note`）、Unicode 位置计数、BOM/`\r` 跳过、peek/buffer/
//! checkpoint（`save_state`/`restore_state`）与 `capture_raw_block` 语义
//! 均以 PLAN-789 V02 冻结 golden（`docs/plans/attachments/789/
//! v02-golden-frozen.json`，基面 e4ac72a62 旧实现生成）为对拍基线。

pub mod error;
pub mod lexer;
pub mod token;
