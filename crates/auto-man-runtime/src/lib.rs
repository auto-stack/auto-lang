//! # auto-man-runtime — 非 UI 管理服务/执行实现（PLAN-816 T-03）
//!
//! 自 auto-man 析出的**非 UI**管理核心：包管理（pac/lock/resolver）、
//! 构建编排（builder/ninja/cargo）、目标生成（target/exporter）、Git
//! adapter、服务入口（http_service）与 Man 编排（automan）。语言面经
//! [`auto_lang_core`]（814 语言 SDK adapter）；进程操作经
//! [`auto_tool_execution`]（本 Plan 统一 owned 设施）。
//!
//! **闭包无 UI 实现**：不依赖 auto-lang 兼容大包/auto-ui-compiler/
//! auto-ui-runtime/auto-ui-tools/iced。UI 操作（vue/rust_ui/jet/ark/
//! tauri/vscode 生成构建运行、VM UI 运行、共享 workspace 重写）经
//! [`ui_ops::UiOps`] seam 由上层 compat（auto-man）注入实现；
//! [`ui_ops`] 未安装时 UI 臂具名失败，非 UI 路径不受影响。
//!
//! 旧 `auto-man` crate 是本包的纯 re-export 门面（Pac/Target/Automan
//! 类型**单源**在本包）+ UI 门面转发（run 家族随 815-T05 迁
//! auto-ui-tools::run_vue/run_vm，M814-B 门解除）。业务算法不复制。

pub mod error;
pub mod resolver;

pub mod asset;
pub mod file_types;
pub mod fs;
pub mod group;
pub mod node_ext;
pub mod port;
pub mod pull;
pub mod util;
pub mod version;

pub mod git;
pub mod index;
pub mod language;
pub mod lock;

pub mod builder;

pub mod cache;
pub mod dir;
pub mod scanner;

pub use auto_api_gen::api_gen;
pub use auto_api_gen::sidecar;
pub mod target;

pub mod automan;
pub mod generation_request_adapter;
pub mod pac;
pub mod project_model_adapter;

pub mod http_service;

pub mod exporter;

// ---- UI seam（F-816 T-03：服务闭包无 UI 的依赖倒置点）----
pub mod ui_ops;

pub use asset::*;
pub use error::*;
pub use file_types::*;
pub use group::*;
pub use node_ext::*;
pub use port::*;
pub use resolver::AutoManResolver;
pub use util::*;
pub use version::*;

pub use git::*;
pub use index::*;
pub use lock::*;

pub use builder::*;

pub use cache::*;
pub use dir::*;
pub use scanner::*;
pub use target::*;

pub use builder::ninja::config::CompilerConfig;

pub use automan::*;
pub use pac::*;

// AutoVal re-exports（模块内以裸路径消费）
pub use auto_val::AutoError;
pub use auto_val::AutoResult;

// 语言 adapter 单源转发（旧 pac 等模块以 auto_lang_core::config 消费的类型）
pub use auto_lang_core::config::BackendType;

/// AutoMan 版本（compat 门面同值转发）。
pub const AUTOMAN_VERSION: &str = env!("CARGO_PKG_VERSION");
