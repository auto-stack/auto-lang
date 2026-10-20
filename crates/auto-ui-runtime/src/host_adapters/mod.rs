//! PLAN-804: 宿主 OS 能力适配层。
//!
//! crate 内 vm core 与 ui renderer 门之外的第三落点：承载"OS 实现但不必
//! 要求 ui renderer"的宿主能力——首员 = 文件对话框（`dialog`，ui-dialog
//! 单门控）。vm core（`vm/native.rs` shim）经
//! [`auto_lang_core::vm::host_capabilities`] provider 接口消费本层实现；ui 侧
//! 薄门面（`ui::dialog`）同层委托。ui 宿主专属的 adapter（如 dnd，需
//! ui 输入线程/事件面）仍留 `ui::` 下，不入本层。

#[cfg(feature = "ui-dialog")]
pub mod dialog;
