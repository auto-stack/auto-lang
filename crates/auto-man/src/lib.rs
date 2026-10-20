// Plan 078-079: AutoMan Migration - AutoLang Monorepo Edition
//
// This crate is being migrated from ../auto-man into the auto-lang workspace
// to provide package management and dependency resolution for AutoLang projects.

// Plan 078: ModuleResolver trait implementation
pub mod error;
pub mod resolver;

// Phase 1: Core modules (from ../auto-man)
pub mod asset;
pub mod file_types;
pub mod fs;
pub mod group;
pub mod node_ext;
pub mod port;
pub mod pull;
// pub mod up;  // NOTE: upgrade functionality disabled — zip dependency removed
pub mod util;
pub mod version;

// Phase 2: Package management (from ../auto-man)
pub mod git;
pub mod index;
pub mod lock;

// Phase 3: Build system (from ../auto-man)
pub mod builder;

// Phase 4: Target & Scanner (from ../auto-man)
pub mod cache;
pub mod dir;
pub mod scanner;
// PLAN-815 T-04: rust_sidecar effect adapter moved to auto-api-gen (declaration
// single source stays auto-man-core); module re-export keeps historic paths.
pub use auto_api_gen::sidecar;
pub mod target;

// Phase 2B: Package management (completed - depends on Phase 3 & 4)
pub mod automan;
/// PLAN-780: explicit old-vs-new probe adapter — not wired into any
/// production pac read path (pilot discipline: defaults unchanged).
pub mod generation_request_adapter;
pub mod pac;
/// PLAN-786: 展示字段（name/title/title_zh）统一只读入口——首个生产
/// project-model 消费者；兼容臂为 vue.rs 原行扫描的唯一冻结副本。
// PLAN-815 T-04: display-field reader moved to auto-ui-tools (vue closure).
pub(crate) use auto_ui_tools::project_display_reader;
pub mod project_model_adapter;
// PLAN-786 T-04: V01..V05 永久对拍族随 project_display_reader 迁
// auto-ui-tools（其 cfg(test) 模块在本 crate 测试构建运行）。

// Phase 5: Vue project support
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
// R1-F03: run 家族残留在 vue_run_compat（M814-B 门），经本门面转发回
// auto_man::vue::* 历史公开路径（无复制算法，纯转发）。
pub mod vue {
    pub use crate::vue_run_compat::{
        run_vue_project, run_vue_project_with_request, start_gallery_back_proxy,
    };
    pub use auto_ui_tools::vue::*;
}

// PLAN-662 T-02: 画廊行扫描磁盘缓存（画廊 registry/proxy 共用扫描面）
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::gallery_cache;

// PLAN-075: G-6 收口定向测——dep/components//bps 重生成臂同文件模块 fn 池
// （夹具 examples/capability-tests/048-bp-module-fn，首个真实消费方）。
// PLAN-815 T-04: plan075 bp 链测试随 vue 迁 auto-ui-tools（cfg(test) 模块）。

// PLAN-457: bundled shadcn-vue ui component snapshots (offline cold start)
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::gallery_assets;
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::vue_shadcn;
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::wm_assets;

// Phase 6: Tauri project support (Plan 130)
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::tauri;

// Phase 7: Jetpack Compose project support
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::jet;

// Phase 8: ArkTS/HarmonyOS project support
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::ark;

// Phase 9: Rust UI support (ICED/GPUI)
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
// R1-F03: run_vm_ui 家族/start_vm_server 残留在 rust_ui_vm_compat
// （M814-B 门），经本门面转发回 auto_man::rust_ui::* 历史公开路径。
pub mod rust_ui {
    pub use crate::rust_ui_vm_compat::{run_vm_ui, run_vm_ui_with_request, start_vm_server};
    pub use auto_ui_tools::rust_ui::*;
}

// Phase 9: API code generation (Plan 130)
// PLAN-815 T-04: api_gen moved to auto-api-gen (single algorithm; owner
// lang-service-adapter); module re-export keeps crate::api_gen paths alive.
pub use auto_api_gen::api_gen;

// PLAN-815 T-04: M814-B 门残留 —— vue run 面(画廊 back_proxy 依赖
// auto_lang::back_proxy/photo_service,814-B 迁 auto-ui-runtime 后归 T-05)
// 与 run_vm_ui 家族(iced 运行时 + vm::backend_abi 同门)。生成/构建主体
// 已迁 auto-ui-tools;残留经门面消费其公开面,行为冻结。
pub mod rust_ui_vm_compat;
pub mod vue_run_compat;
// PLAN-815 T-04: I-PROJECT 请求缝对拍测试(构造侧=adapter 属 Man)。
#[cfg(test)]
mod vue_request_seam_tests;

// PLAN-736: 工程级 HTTP 服务独立启动（`auto serve`；vm/rust 双轨）
pub mod http_service;

// Plan 547: deterministic image-viewer directory/index primitives.
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::image_viewer;

// Phase 10: Tauri backend generator (Plan 151)
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::tauri_backend;

// Phase 11: VSCode extension generator (Plan 181)
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::vscode;

// Phase 12: Package manager abstraction (bun/npm) (Plan 186)
// PLAN-815 T-04: moved to auto-ui-tools; facade keeps historic paths.
pub use auto_ui_tools::pkg;

// NOTE: stubs.rs removed in Phase 6 - all types migrated

// Re-exports (Plan 078 + Phase 1)
pub use asset::*;
pub use error::*;
pub use file_types::*;
pub use group::*;
pub use node_ext::*;
pub use port::*;
pub use resolver::AutoManResolver;

// pub use up::*;  // NOTE: upgrade functionality disabled
pub use util::*;
pub use version::*;

// Re-exports (Phase 2)
pub use git::*;
pub use index::*;
pub use lock::*;
// Re-exports (Phase 3)
pub use builder::*;
// Re-exports (Phase 4)
pub use cache::*;
pub use dir::*;
pub use scanner::*;
pub use target::*;
// Re-exports from Phase 3 (builder/ninja/config)
pub use builder::ninja::config::CompilerConfig;
// Re-exports (Phase 2B)
pub use automan::*;
pub use pac::*;

// Re-exports (Phase 5: Vue)
pub use vue::*;

pub use image_viewer::*;

// Re-exports (Phase 6: Tauri)
pub use tauri::*;

// Re-exports (Phase 7: Jet)
pub use jet::*;

// Re-exports (Phase 8: Ark)
pub use ark::*;

// Re-exports (Phase 11: VSCode)
pub use vscode::*;

// NOTE: All types migrated in Phases 1-5
// NOTE: stubs.rs removed in Phase 6 (cleanup)

// AutoVal re-exports
pub use auto_val::AutoError;
pub use auto_val::AutoResult;

// Re-export BackendType from auto_lang::config
pub use auto_lang::config::BackendType;

/// AutoMan version
pub const AUTOMAN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// AutoMan - Package manager for AutoLang projects
///
/// This is a stub during migration. Full functionality will be added
/// as modules are migrated from ../auto-man.
pub struct AutoMan {
    /// Project root directory
    root: std::path::PathBuf,
}

impl AutoMan {
    /// Create a new AutoMan instance for the given project root
    pub fn new(root: std::path::PathBuf) -> Self {
        Self { root }
    }

    /// Get the project root directory
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    /// Initialize AutoMan in a new project
    ///
    /// This creates the necessary configuration files and directory structure
    pub fn init(&self) -> Result<(), AutoManError> {
        // TODO: Implement pac.at creation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automan_creation() {
        let automan = AutoMan::new(std::path::PathBuf::from("."));
        assert_eq!(automan.root(), std::path::Path::new("."));
    }

    #[test]
    fn test_version() {
        assert!(!AUTOMAN_VERSION.is_empty());
    }
}

pub mod exporter;
