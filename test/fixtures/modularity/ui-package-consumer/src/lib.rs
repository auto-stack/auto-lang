//! PLAN-814 M814-A UI 编译包消费面（V-03）。
//!
//! 本 crate 刻意保持空实现体——被测面全部在 `tests/` 集成测试中经
//! `auto_ui_compiler::` 公开路径驱动（真实外部消费者形态）。

pub const PROBE: &str = "ui-package-consumer";
