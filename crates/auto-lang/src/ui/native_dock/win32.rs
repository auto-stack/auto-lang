//! Win32 适配层（Plan 473）：发现 / 几何 / 样式 / 层级 / WinEventHook。
//!
//! `windows` crate 调用只允许出现在本文件与 `tools/native-fixture/`；
//! 其余模块一律经由 `native_dock::mod` 的纯逻辑层与本层的薄封装。
