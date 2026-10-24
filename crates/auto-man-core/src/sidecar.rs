//! Sidecar 声明解析旧入口再导出（PLAN-831 T-05 C2）。
//!
//! 实现单源已迁 `auto-project-model::sidecar`（逐字）；本模块仅保持
//! `auto_man_core::sidecar::*` 与 lib 级 `auto_man_core::{sidecar_decl_from_root,
//! SidecarDecl}` 旧路径可用——Man 域既有消费者零改动。

pub use auto_project_model::sidecar::*;
