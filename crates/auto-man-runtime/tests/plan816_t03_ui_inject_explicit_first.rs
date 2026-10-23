//! PLAN-816 T-03 R2-01：显式注入不得被默认 ctor/compat 兜底抢占。
//!
//! 本文件为独立集成测（独立进程），覆盖「先显式后默认」合同。

use std::path::Path;
use std::sync::Arc;

use auto_man_runtime::ui_ops::{self, UiOps};
use auto_man_runtime::AutoResult;
use auto_project_model::GenerationRequest;

struct MarkerUiOps(&'static str);

impl MarkerUiOps {
    fn unavailable() -> AutoResult<()> {
        Err("marker should not run real UI".into())
    }
}

impl UiOps for MarkerUiOps {
    fn vue_build(&self, _: &Path, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vue_generate(&self, _: &Path, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vue_generate_workspace(&self, _: &Path, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vue_run(&self, _: &Path, _: Vec<String>, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn rust_build(&self, _: &Path, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn rust_generate(
        &self,
        _: &Path,
        _: Option<&Path>,
        _: bool,
        _: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn rust_generate_delivery(&self, _: &Path, _: &Path, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn rust_run(&self, _: &Path, _: Vec<String>, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vm_run(&self, _: &Path, _: Vec<String>, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn jet_generate(
        &self,
        _: &Path,
        _: Option<&Path>,
        _: bool,
        _: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn jet_build(&self, _: &Path, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn jet_run(&self, _: &Path, _: Vec<String>, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn ark_generate(
        &self,
        _: &Path,
        _: Option<&Path>,
        _: bool,
        _: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn ark_build(&self, _: &Path, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn ark_run(&self, _: &Path, _: Vec<String>, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vscode_generate(
        &self,
        _: &Path,
        _: Option<&Path>,
        _: bool,
        _: &GenerationRequest,
    ) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vscode_build(&self, _: &Path, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn vscode_run(&self, _: &Path, _: Vec<String>, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn tauri_run(&self, _: &Path, _: Vec<String>, _: &GenerationRequest) -> AutoResult<()> {
        Self::unavailable()
    }
    fn create_jet_project(&self, _: &str) -> AutoResult<()> {
        Self::unavailable()
    }
    fn pkg_display_name(&self) -> &'static str {
        self.0
    }
    fn pkg_run_script(&self, _: &str, _: &[&str], _: &Path) -> Result<(), String> {
        Err("marker".into())
    }
}

/// R2-01 主序（单测串行，避免 OnceLock 交叉）：首次显式注入被采用，
/// 其后默认兜底不覆盖，后装显式不覆盖首个显式。
#[test]
fn explicit_first_install_is_adopted_and_default_preserves_it() {
    let desired: Arc<dyn UiOps> = Arc::new(MarkerUiOps("explicit-marker"));
    ui_ops::install(desired.clone());
    let current = ui_ops::current();
    assert!(Arc::ptr_eq(&desired, &current), "首次显式注入必须被采用");
    assert_eq!(current.pkg_display_name(), "explicit-marker");

    ui_ops::install_default(Arc::new(MarkerUiOps("default-marker")));
    assert!(
        Arc::ptr_eq(&desired, &ui_ops::current()),
        "默认兜底不得覆盖已有显式 provider"
    );
    assert_eq!(ui_ops::current().pkg_display_name(), "explicit-marker");

    let second: Arc<dyn UiOps> = Arc::new(MarkerUiOps("second"));
    ui_ops::install(second);
    assert_eq!(
        ui_ops::current().pkg_display_name(),
        "explicit-marker",
        "后装显式不覆盖已有显式"
    );
}
