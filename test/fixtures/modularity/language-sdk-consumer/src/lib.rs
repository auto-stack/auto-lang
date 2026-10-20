//! PLAN-814 M814-A 纯 SDK 消费面（V-01）。
//!
//! 本 crate 刻意保持空实现体——被测面全部在 `tests/` 集成测试中经
//! `auto_lang_core::` 公开路径驱动（真实外部消费者形态，非 crate 内
//! cfg(test) 白盒）。

pub const PROBE: &str = "language-sdk-consumer";
