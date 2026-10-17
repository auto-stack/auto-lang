//! PLAN-681: native open/save file dialogs — a2r host of the VM builtins
//! `dialog_open` / `dialog_save` (natives 2927/2928, `auto.dialog.open/save`).
//!
//! The ui_gen builtin-call mapping table emits `auto_lang::ui::dialog::…`
//! for those names in generated apps.
//!
//! PLAN-804: 本模块收敛为**薄门面**——过滤解析、默认文件名种子、取消
//! 空串约定、rfd 构建的单一 OS 实现迁 `host_adapters::dialog`（VM shim
//! provider 与本门面共用，不再两份构建/过滤/父窗算法）。本门面只保留
//! a2r 旧 **unparented** 父窗策略（PLAN-681 §9 F1 缝：a2r generated apps
//! call this host directly and the dialog is unparented in v1 — modal
//! parenting for the a2r shell is a separate wiring concern, not a
//! semantic fork）；VM 默认 adapter 用 parented 策略（Plan 428 债修保持），
//! 策略差异诚实登记于 shell-a2r-seams。

/// `dialog_open(filter) -> String` — pick one file; `""` on cancel.
pub fn open(filter: &str) -> String {
    crate::host_adapters::dialog::unparented_open(filter)
}

/// `dialog_save(default_name) -> String` — pick a save target; `""` on cancel.
pub fn save(default_name: &str) -> String {
    crate::host_adapters::dialog::unparented_save(default_name)
}
