//! PLAN-814 R3-03b：UI 文法依赖回归测试组——SDK（auto-lang-core）清单
//! 消除 auto-ui-dialect 边后，原 `ui-dialect` feature 门控的 86 项测试
//! （parser 60 / design_tokens 9 / tscn 11 / gdscript 6）随文法 owner 迁入
//! 本 crate（dev-dep auto-lang-core 驱动真 Parser；文法表经 `setup()`
//! 进程级装配）。选择面经 cargo t/tf/tt 的 `-p auto-ui-dialect` 保持
//! （V-05 映射表见 docs/plans/attachments/814/）。

#[cfg(test)]
pub mod design_tokens_recipe;
#[cfg(test)]
pub mod parser_ui_grammar;
#[cfg(test)]
pub mod trans_gdscript;
#[cfg(test)]
pub mod trans_tscn;
