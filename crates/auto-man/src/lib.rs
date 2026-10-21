// PLAN-816 T-03：本 crate 自本起是 **auto-man-runtime 的纯 re-export 门面 +
// UI 门面**（单源分层）：
// - Pac/Target/Automan/所有非 UI 模块**单源在 auto-man-runtime**
//   （语言面 auto-lang-core、进程面 auto-tool-execution、闭包无 UI 实现）；
// - UI 公开门面（vue/rust_ui/jet/ark/tauri/vscode/gallery/...）经
//   auto-ui-tools 转发（815-T04 已迁；run 家族随 815-T05 迁
//   auto-ui-tools::run_vue / run_vm，M814-B 门解除，历史路径保持）；
// - [`compat`] 安装 UiOps 实现（runtime 服务编排的 UI 臂经 seam 回调本层）。
//
// Plan 078-079 迁移史注释保留于此（原 lib.rs 头）。

// ---- 非 UI 单源 re-export（模块与根符号与旧路径全兼容）----
pub use auto_man_runtime::*;

// ---- UI 门面（815-T04/T-05 迁移后的历史路径转发）----
pub mod vue {
    pub use auto_ui_tools::run_vue::{
        run_vue_project, run_vue_project_with_request, start_gallery_back_proxy,
    };
    pub use auto_ui_tools::vue::*;
}

pub use auto_ui_tools::ark;
pub use auto_ui_tools::gallery_assets;
pub use auto_ui_tools::gallery_cache;
pub use auto_ui_tools::image_viewer;
pub use auto_ui_tools::jet;
pub use auto_ui_tools::pkg;
pub use auto_ui_tools::tauri;
pub use auto_ui_tools::tauri_backend;
pub use auto_ui_tools::vscode;
pub use auto_ui_tools::vue_shadcn;
pub use auto_ui_tools::wm_assets;

/// UI 工具 HTTP/端口约定（815-T04 迁 auto-ui-tools;本 crate 残留/
/// 历史路径消费——`crate::util::http_port` 等）。
pub mod ui_port_util {
    pub use auto_ui_tools::util::{
        hash_string, http_base_url, http_port, kill_process_on_port, parse_tasklist_name,
        should_skip_kill,
    };
}

/// 历史 `auto_man::util` 模块路径（PLAN-816 T-03 F-03）：runtime 非 UI
/// util + UI 端口约定六函数单源转发。显式 `mod util` 遮蔽 glob 再导出的
/// `auto_man_runtime::util`，保持
/// `auto_man::util::{hash_string,http_base_url,http_port,kill_process_on_port,parse_tasklist_name,should_skip_kill}`。
pub mod util {
    pub use auto_man_runtime::util::*;
    pub use auto_ui_tools::util::{
        hash_string, http_base_url, http_port, kill_process_on_port, parse_tasklist_name,
        should_skip_kill,
    };
}

pub mod rust_ui {
    pub use auto_ui_tools::run_vm::{run_vm_ui, run_vm_ui_with_request, start_vm_server};
    pub use auto_ui_tools::rust_ui::*;
}

// PLAN-786: 展示字段只读入口（815-T04 迁 auto-ui-tools;本 crate 测试消费）。
pub(crate) use auto_ui_tools::project_display_reader;

// PLAN-815 T-04: I-PROJECT 请求缝对拍测试(构造侧=adapter 属 Man)。
#[cfg(test)]
mod vue_request_seam_tests;

// ---- UI seam 注入（runtime 服务编排的 UI 臂经此回调用本层实现）----
pub mod compat;

// Re-exports (Phase 5+: UI 根符号历史路径)
pub use ark::*;
pub use image_viewer::*;
pub use jet::*;
pub use tauri::*;
pub use ui_port_util::*;
pub use vscode::*;
pub use vue::*;

/// AutoMan 版本（与 runtime 同值;历史常量路径保持）。
pub const AUTOMAN_VERSION: &str = auto_man_runtime::AUTOMAN_VERSION;

/// 兼容 stub（历史类型;功能面在 runtime::automan::Automan）。
pub struct AutoMan {
    root: std::path::PathBuf,
}

impl AutoMan {
    pub fn new(root: std::path::PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub fn init(&self) -> Result<(), AutoManError> {
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
