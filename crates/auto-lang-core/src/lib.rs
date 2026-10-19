//! auto-lang-core — 纯语言 SDK（PLAN-814）。
//!
//! 从 auto-lang 巨型 crate 逐模块迁入的非 UI 语言栈单一来源：use 扫描、
//! 解析/AST/类型、编译会话、AutoVM、多目标转译与基础服务。
//!
//! 边界合同（PLAN-814 §1/AC-01）：本 crate 的 normal/build/dev 闭包不得
//! 引入 UI 实现（ui/ui_gen 生成器、renderer、桌面宿主、Man 管理实现）；
//! 宿主能力一律经 `vm::host_capabilities` provider 端口，不反向链接 UI。
//! 旧 `auto_lang::*` 公开路径由上层兼容包（auto-lang）经再导出保身份。

pub mod use_scanner;
