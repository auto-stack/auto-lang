//! PLAN-982（982/WEBTARGET，第一步）：**A2TS/A2JS 的宿主驱动面**。
//!
//! 与 [`crate::python_transpile`] 同一形状（provider 无便捷装配器，走 `a2x-worker`
//! 的参考管线）。本驱动用 **JavaScript** provider：产物可直接由 `node` 执行，
//! 因此能在本机拿到**真实运行**证据（无需 `tsc`）。
//!
//! 合同：失败具名；同输入逐字节确定；只服务 `Family::Scalar`（其它家族的前导
//! 种子未验证，不假装支持）。

use ace_a2x_core::target::{Family, FilesetRequest, FragmentText, TargetProvider};
use ace_a2x_core::SemanticView;

/// 一个产出文件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AceScriptFile {
    pub name: String,
    pub text: String,
}

/// 一次 A2JS 渲染的结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AceScriptFileSet {
    pub files: Vec<AceScriptFile>,
    pub manifest: String,
    pub source_sha256: String,
}

/// 具名失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AceScriptError {
    pub code: &'static str,
    pub message: String,
}

impl AceScriptError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AceScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// 把一段真实 Auto 源码渲染成 A2JS file set（`Family::Scalar`）。
pub fn transpile_scalar_source(
    source: &str,
    module: &str,
    entry: &str,
) -> Result<AceScriptFileSet, AceScriptError> {
    let compiled =
        ace_core::source::compile_scalar_source(source, module, &[]).map_err(|diags| {
            AceScriptError::new(
                "ace.js/compile",
                format!(
                    "frontend refused the source: {}",
                    diags
                        .iter()
                        .map(|d| format!("{:?}/{}", d.stage, d.code))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        })?;
    let view = SemanticView::from_checked(&compiled.checked)
        .map_err(|reject| AceScriptError::new("ace.js/view", format!("{reject:?}")))?;

    let provider = ace_a2x_ts::JavaScriptTargetProvider::v1();
    let family = Family::Scalar;
    let runtime = provider
        .runtime_source(&view, family)
        .map_err(|reject| AceScriptError::new("ace.js/produce", format!("{reject:?}")))?;
    let functions: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let texts = provider
        .produce_fragments(&view, &functions, family)
        .map_err(|reject| AceScriptError::new("ace.js/produce", format!("{reject:?}")))?;
    let mut fingerprints = Vec::with_capacity(functions.len());
    for function in &functions {
        let fp = ace_a2x_core::semantic_fingerprint(&view, function)
            .map_err(|reject| AceScriptError::new("ace.js/produce", format!("{reject:?}")))?;
        fingerprints.push(fp);
    }
    let fragments: Vec<FragmentText<'_>> = functions
        .iter()
        .zip(texts.iter())
        .zip(fingerprints.iter())
        .map(|((function, text), fp)| FragmentText {
            def_index: function.def_index,
            source_name: function.source_name,
            sem_fp: fp.as_str(),
            text: text.as_str(),
        })
        .collect();
    let prelude_seed = provider
        .capture_prelude_seed(&view, family)
        .map_err(|reject| AceScriptError::new("ace.js/produce", format!("{reject:?}")))?;
    let source_sha256 = crate::rust_transpile::source_sha256_hex(source.as_bytes());
    let fileset = provider
        .assemble_fileset(FilesetRequest {
            entry,
            input_hash: &compiled.input_hash,
            family,
            fragments: &fragments,
            view: Some(&view),
            runtime: &runtime,
            prelude_seed: prelude_seed.as_deref(),
            origins: None,
            source_sha256: Some(&source_sha256),
        })
        .map_err(|reject| AceScriptError::new("ace.js/assemble", format!("{reject:?}")))?;
    Ok(AceScriptFileSet {
        files: fileset
            .files
            .into_iter()
            .map(|(name, text)| AceScriptFile { name, text })
            .collect(),
        manifest: fileset.manifest,
        source_sha256,
    })
}
