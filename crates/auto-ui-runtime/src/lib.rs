//! auto-ui-runtime — UI 运行/renderer/宿主域（PLAN-814）。
//!
//! 从 auto-lang 迁入的 VM 渲染轨与桌面宿主栈：iced renderer、desktop
//! session/protocol、编辑器/终端/媒体 widget、host adapter 与宿主能力
//! provider 实现（T-04 迁移面）。
//!
//! 边界合同：依赖 SDK（auto-lang-core）与 UI compiler（auto-ui-compiler）；
//! 不被任一反向依赖。宿主能力装配（`install_ui_host_capabilities` 族）
//! 收敛到本 crate，SDK 零隐式 UI 装配。
