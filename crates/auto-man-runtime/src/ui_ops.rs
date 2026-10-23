//! UI seam（PLAN-816 T-03）：服务闭包无 UI 的依赖倒置点。
//!
//! 服务层（automan/http_service/builder）的 UI 操作臂不直接链接任何 UI
//! 实现——经 [`UiOps`] trait 由上层 compat（auto-man）注入：compat 用
//! auto-ui-tools 公开面（run 家族随 815-T05 迁入）实现本 trait，并在
//! CLI 入口安装（[`install`]）。未安装时 [`current`]
//! 返回 [`NotInstalledUiOps`]——UI 臂具名失败（"UI ops not installed"），
//! 非路径不受影响；服务包的依赖闭包因此不含 UI 实现。

use std::path::Path;
use std::sync::Arc;

use auto_project_model::GenerationRequest;
use auto_val::AutoResult;

/// UI 操作集：vue、rust_ui、jet、ark、tauri、vscode 的生成、构建与
/// 运行，VM UI 运行，以及服务侧 workspace 重写。签名与
/// auto-ui-tools/compat 公开面一一对应，不复制算法（compat 侧纯委托）。
pub trait UiOps: Send + Sync {
    // ---- vue ----
    /// PLAN-816 T-05 R1（F-04）：vue/vscode 臂与 jet/ark 同型消费**本次
    /// 有效请求**（render 覆盖/动态求值在场即权威——不再让实现侧回读
    /// 主 pac 形成第二真源）。
    fn vue_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()>;
    fn vue_generate(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()>;
    /// `VueProject::from_workspace_with_request(..)?.generate()` 的收敛形态
    /// （vue/tauri 前端臂共用）。
    fn vue_generate_workspace(
        &self,
        root_dir: &Path,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn vue_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()>;

    // ---- rust UI ----
    fn rust_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()>;
    fn rust_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn rust_generate_delivery(
        &self,
        root_dir: &Path,
        delivery_root: &Path,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn rust_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn vm_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()>;

    // ---- jet / ark / vscode / tauri ----
    fn jet_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn jet_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()>;
    fn jet_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn ark_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn ark_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()>;
    fn ark_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn vscode_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn vscode_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()>;
    fn vscode_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    fn tauri_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()>;
    /// `auto new --jet` 的 Jetpack 工程脚手架（JetProjectConfig 全流程）。
    fn create_jet_project(&self, name: &str) -> AutoResult<()>;

    // ---- pkg/npm 脚本编排（算法单源 auto-ui-tools::pkg;builder Vue 臂）----
    fn pkg_display_name(&self) -> &'static str;
    fn pkg_run_script(&self, script: &str, extra_args: &[&str], cwd: &Path) -> Result<(), String>;
}

/// 未安装时的具名失败实现（非 UI 路径不触达）。
struct NotInstalledUiOps;

impl NotInstalledUiOps {
    fn unavailable() -> AutoResult<()> {
        Err(
            "UI ops not installed: construct via the auto-man compat entry \
             (auto_man::compat::install_default_ui_ops) before UI-backed operations"
                .into(),
        )
    }
}

impl UiOps for NotInstalledUiOps {
    fn vue_build(&self, _root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vue_generate(&self, _root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vue_generate_workspace(
        &self,
        _root_dir: &Path,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vue_run(
        &self,
        _root_dir: &Path,
        _args: Vec<String>,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn rust_build(&self, _root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn rust_generate(
        &self,
        _root_dir: &Path,
        _output: Option<&Path>,
        _project: bool,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn rust_generate_delivery(
        &self,
        _root_dir: &Path,
        _delivery_root: &Path,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn rust_run(
        &self,
        _root_dir: &Path,
        _args: Vec<String>,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vm_run(
        &self,
        _root_dir: &Path,
        _args: Vec<String>,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn jet_generate(
        &self,
        _root_dir: &Path,
        _output: Option<&Path>,
        _project: bool,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn jet_build(&self, _root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn jet_run(
        &self,
        _root_dir: &Path,
        _args: Vec<String>,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn ark_generate(
        &self,
        _root_dir: &Path,
        _output: Option<&Path>,
        _project: bool,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn ark_build(&self, _root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn ark_run(
        &self,
        _root_dir: &Path,
        _args: Vec<String>,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vscode_generate(
        &self,
        _root_dir: &Path,
        _output: Option<&Path>,
        _project: bool,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vscode_build(&self, _root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vscode_run(
        &self,
        _root_dir: &Path,
        _args: Vec<String>,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn tauri_run(
        &self,
        _root_dir: &Path,
        _args: Vec<String>,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn create_jet_project(&self, _name: &str) -> AutoResult<()> {
        Self::unavailable()
    }
    fn pkg_display_name(&self) -> &'static str {
        "auto"
    }
    fn pkg_run_script(
        &self,
        _script: &str,
        _extra_args: &[&str],
        _cwd: &Path,
    ) -> Result<(), String> {
        Err("UI ops not installed (pkg)".into())
    }
}

static EXPLICIT: std::sync::OnceLock<Arc<dyn UiOps>> = std::sync::OnceLock::new();
static DEFAULT_OPS: std::sync::OnceLock<Arc<dyn UiOps>> = std::sync::OnceLock::new();

/// 调用方**显式**注入 UI 操作集：先装先得；重复安装不覆盖已有显式 provider。
///
/// 这是公开依赖倒置点——测试替身/自定义实现必须能先于任何默认兜底被采用。
pub fn install(ops: Arc<dyn UiOps>) {
    let _ = EXPLICIT.set(ops);
}

/// 默认兜底实现（compat）。不抢占显式注入；[`current`] 仅在无显式 provider 时采用。
pub fn install_default(ops: Arc<dyn UiOps>) {
    let _ = DEFAULT_OPS.set(ops);
}

/// 当前 UI 操作集：显式注入优先，其次默认兜底，否则 [`NotInstalledUiOps`]。
pub fn current() -> Arc<dyn UiOps> {
    EXPLICIT
        .get()
        .or_else(|| DEFAULT_OPS.get())
        .cloned()
        .unwrap_or_else(|| Arc::new(NotInstalledUiOps))
}
