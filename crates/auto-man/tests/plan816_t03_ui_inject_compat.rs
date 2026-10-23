//! PLAN-816 T-03 R2-01：链接 compat 后，调用方首次显式注入仍被采用，
//! 其后 `install_default_ui_ops`（含 ctor 自举）不得覆盖。

use std::path::Path;
use std::sync::Arc;

use auto_man_runtime::ui_ops::{self, UiOps};
use auto_man_runtime::AutoResult;
use auto_project_model::GenerationRequest;

struct InjectedUiOps;

impl InjectedUiOps {
    fn unavailable() -> AutoResult<()> {
        Err("injected marker".into())
    }
}

impl UiOps for InjectedUiOps {
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
        "injected"
    }
    fn pkg_run_script(&self, _: &str, _: &[&str], _: &Path) -> Result<(), String> {
        Err("injected".into())
    }
}

#[test]
fn explicit_injection_survives_compat_default_and_ctor() {
    let desired: Arc<dyn UiOps> = Arc::new(InjectedUiOps);
    ui_ops::install(desired.clone());
    assert!(
        Arc::ptr_eq(&desired, &ui_ops::current()),
        "首次显式注入必须被采用（ctor 自举不得抢占）"
    );
    assert_eq!(ui_ops::current().pkg_display_name(), "injected");

    auto_man::compat::install_default_ui_ops();
    assert!(
        Arc::ptr_eq(&desired, &ui_ops::current()),
        "默认安装须保留已有显式 provider"
    );
    assert_eq!(ui_ops::current().pkg_display_name(), "injected");
}
