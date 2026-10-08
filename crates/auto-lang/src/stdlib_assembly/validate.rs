//! PLAN-738：覆盖检查（T-02 范围）。核心符号/provider/native 绑定门在
//! T-03/T-04 充实；本文件先固化诊断码表与纯结构检查。

use super::model::{AssemblyDiagnostic, LayerKind, ModuleInventory};

/// 诊断码表（machine-readable，CLI `--check` 输出消费）。
pub mod code {
    pub const PARSE_FAIL: &str = "STDINV.PARSE_FAIL";
    pub const READ_FAIL: &str = "STDINV.READ_FAIL";
    pub const NO_PUBLIC_LAYER: &str = "STDINV.NO_PUBLIC_LAYER";
    pub const PROVIDER_UNSUPPORTED: &str = "STDASSEMBLY.PROVIDER_UNSUPPORTED";
    pub const PROVIDER_CLAIM_NO_CALLEE: &str = "STDASSEMBLY.PROVIDER_CLAIM_NO_CALLEE";
    pub const SIGNATURE_DRIFT: &str = "STDASSEMBLY.SIGNATURE_DRIFT";
    pub const NATIVE_ID_CONFLICT: &str = "STDASSEMBLY.NATIVE_ID_CONFLICT";
    pub const TARGET_MISMATCH: &str = "STDASSEMBLY.TARGET_MISMATCH";
}

/// 结构覆盖检查：有目标层却无公共层的模块（§5.1 缺口——目标层可以是
/// candidate，但公共声明缺失使缝合无锚）。
pub fn coverage_checks(modules: &[ModuleInventory]) -> Vec<AssemblyDiagnostic> {
    let mut out = Vec::new();
    for m in modules {
        let has_public = m.layers.iter().any(|l| l.kind == LayerKind::Public);
        if !has_public {
            let files: Vec<String> = m.layers.iter().map(|l| l.file.clone()).collect();
            out.push(AssemblyDiagnostic {
                code: code::NO_PUBLIC_LAYER.to_string(),
                module: m.module.clone(),
                file: None,
                message: format!(
                    "target layer(s) without a public .at: {}",
                    files.join(", ")
                ),
            });
        }
    }
    out
}
