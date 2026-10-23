//! PLAN-816 T-03 R2-01：无显式注入时默认兜底生效（独立进程）。

use std::path::Path;
use std::sync::Arc;

use auto_man_runtime::ui_ops::{self, UiOps};
use auto_man_runtime::AutoResult;
use auto_project_model::GenerationRequest;

struct DefaultMarker;

impl DefaultMarker {
    fn unavailable() -> AutoResult<()> {
        Err("marker".into())
    }
}

impl UiOps for DefaultMarker {
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
        "default-marker"
    }
    fn pkg_run_script(&self, _: &str, _: &[&str], _: &Path) -> Result<(), String> {
        Err("marker".into())
    }
}

#[test]
fn default_fallback_used_when_no_explicit() {
    // 独立进程：本测不得先 install 显式槽。
    let ops: Arc<dyn UiOps> = Arc::new(DefaultMarker);
    ui_ops::install_default(ops.clone());
    assert!(
        Arc::ptr_eq(&ops, &ui_ops::current()),
        "无显式时默认兜底应生效"
    );
    assert_eq!(ui_ops::current().pkg_display_name(), "default-marker");
}
