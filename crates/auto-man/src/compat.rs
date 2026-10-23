//! UI seam 的 compat 实现（PLAN-816 T-03）：把 runtime 服务编排的 UI 臂
//! 接回 auto-ui-tools（生成/构建/run 算法单源，run 家族随 815-T05 迁入）+
//! 外层 auto-lang 的 `run_file`（语言执行兼容面）。纯委托，零算法复制。
//!
//! CLI 入口（`auto` main）在启动时调用一次 [`install_default_ui_ops`]；
//! runtime 未安装时 UI 臂具名失败，非 UI 路径不受影响。

use std::path::Path;
use std::sync::Arc;

use auto_man_runtime::ui_ops::UiOps;
use auto_project_model::GenerationRequest;
use auto_val::AutoResult;

/// 安装默认（compat）UI 操作集——写入**默认兜底槽**，不抢占调用方
/// 已有的显式 [`auto_man_runtime::ui_ops::install`]；幂等。CLI 入口与
/// 链接自举均走此入口。
pub fn install_default_ui_ops() {
    auto_man_runtime::ui_ops::install_default(Arc::new(CompatUiOps));
}

struct CompatUiOps;

impl UiOps for CompatUiOps {
    fn vue_build(&self, root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        crate::vue::build_vue_project(root_dir)
    }

    fn vue_generate(&self, root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        crate::vue::gen_vue_project(root_dir)
    }

    fn vue_generate_workspace(
        &self,
        root_dir: &Path,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        let ctx = crate::vue::VueProject::from_workspace_with_request(root_dir, request)?;
        ctx.generate()
    }

    fn vue_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::vue::run_vue_project_with_request(root_dir, args, request)
    }

    fn rust_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        crate::rust_ui::build_rust_ui_with_request(root_dir, request)
    }

    fn rust_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::rust_ui::generate_rust_ui_with_request(root_dir, output, project, request)
    }

    fn rust_generate_delivery(
        &self,
        root_dir: &Path,
        delivery_root: &Path,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::rust_ui::generate_rust_ui_delivery_with_request(root_dir, delivery_root, request)
    }

    fn rust_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::rust_ui::run_rust_ui_with_request(root_dir, args, request)
    }

    fn vm_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::rust_ui::run_vm_ui_with_request(root_dir, args, request)
    }

    fn jet_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::jet::generate_jet_project_with_request(root_dir, output, project, request)
    }

    fn jet_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        crate::jet::build_jet_project_with_request(root_dir, request)
    }

    fn jet_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::jet::run_jet_project_with_request(root_dir, args, request)
    }

    fn ark_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::ark::generate_ark_project_with_request(root_dir, output, project, request)
    }

    fn ark_build(&self, root_dir: &Path, request: &GenerationRequest) -> AutoResult<()> {
        crate::ark::build_ark_project_with_request(root_dir, request)
    }

    fn ark_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::ark::run_ark_project_with_request(root_dir, args, request)
    }

    fn vscode_generate(
        &self,
        root_dir: &Path,
        output: Option<&Path>,
        project: bool,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::vscode::generate_vscode_project(root_dir, output, project)
    }

    fn vscode_build(&self, root_dir: &Path, _request: &GenerationRequest) -> AutoResult<()> {
        crate::vscode::build_vscode_project(root_dir)
    }

    fn vscode_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        _request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::vscode::run_vscode_project(root_dir, args)
    }

    fn tauri_run(
        &self,
        root_dir: &Path,
        args: Vec<String>,
        request: &GenerationRequest,
    ) -> AutoResult<()> {
        crate::tauri::run_tauri_project_with_request(root_dir, args, request)
    }

    fn create_jet_project(&self, name: &str) -> AutoResult<()> {
        // 原 Automan::create_jet 身体（JetProjectConfig 全流程;UI 脚手架
        // owner=auto-lang ui_gen::jet 兼容面——本 compat 层职责)。
        use auto_lang::ui_gen::jet::{JetProjectConfig, ProjectGenerator};

        let path = Path::new(name);
        if path.is_file() {
            return Err(format!("A file named with {} already exists", name).into());
        }
        if path.is_dir()
            && path
                .read_dir()
                .map(|mut d| d.next().is_some())
                .unwrap_or(false)
        {
            return Err(format!("A non-empty directory named with {} already exists", name).into());
        }
        let pac_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("MyApp");
        let config = JetProjectConfig::new(pac_name);
        let mut generator = ProjectGenerator::with_config(config);
        let files = generator.generate();

        std::fs::create_dir_all(path)?;
        for (file_path, content) in files {
            let full_path = path.join(&file_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, content)?;
            log::info!("Created {}", full_path.display());
        }
        println!(
            "Jetpack Compose project '{}' created successfully!",
            pac_name
        );
        println!("Open with: auto open");
        Ok(())
    }

    fn pkg_display_name(&self) -> &'static str {
        crate::pkg::display_name()
    }

    fn pkg_run_script(&self, script: &str, extra_args: &[&str], cwd: &Path) -> Result<(), String> {
        crate::pkg::run_script(script, extra_args, cwd)
    }
}

/// PLAN-816 T-03 F-01/R2-01：旧公开入口默认合同——链接本 crate 的进程在
/// 启动时登记 CompatUiOps 为**默认兜底**，`auto_man::Automan::create_jet`
/// 等无需调用方显式安装；不占用显式注入槽，调用方首次 `ui_ops::install`
/// 仍被采用。runtime-only 消费者不链本 crate，UI seam 仍具名失败。
#[ctor::ctor]
fn auto_install_compat_ui_ops() {
    install_default_ui_ops();
}
