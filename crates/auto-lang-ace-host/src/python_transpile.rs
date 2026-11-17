//! PLAN-982（982/PY，第一步）：**A2Py 的宿主驱动面**。
//!
//! 与 RUST 的 `rust_transpile` 同层、同纪律，但 PY 的 provider **没有**便捷
//! 装配器：必须按 `a2x-worker` 的参考管线走完整流程——
//! 受检前门 → `SemanticView` → provider 的 `runtime_source`/`produce_fragments`/
//! `semantic_fingerprint`/`capture_prelude_seed` → `assemble_fileset(FilesetRequest)`。
//!
//! 合同：
//! - 失败**具名**（`ace.py/compile`、`ace.py/view`、`ace.py/produce`、`ace.py/assemble`）；
//! - 同输入**逐字节确定**（file set + manifest）；
//! - 不隐瞒 profile：本驱动只服务 `Family::Scalar`（其它家族需要各自的种子/前导，
//!   未验证前不假装支持）。

use ace_a2x_core::target::{Family, FilesetRequest, FragmentText, TargetProvider};
use ace_a2x_core::SemanticView;

/// 一个产出文件（相对路径 + 文本）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcePyFile {
    pub name: String,
    pub text: String,
}

/// 一次 A2Py 渲染的完整结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcePyFileSet {
    pub files: Vec<AcePyFile>,
    /// 冻结 schema 的 manifest（`auto.a2x.python.fileset/…`）。
    pub manifest: String,
    /// 降级后的 Python 入口名。
    pub entry: String,
    /// 内容 sha256（manifest 的身份字段）。
    pub source_sha256: String,
}

/// A2Py 驱动的具名失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcePyError {
    pub code: &'static str,
    pub message: String,
}

impl AcePyError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AcePyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// 把一段真实 Auto 源码渲染成 A2Py file set（`Family::Scalar`）。
pub fn transpile_scalar_source(
    source: &str,
    module: &str,
    entry: &str,
) -> Result<AcePyFileSet, AcePyError> {
    let compiled =
        ace_core::source::compile_scalar_source(source, module, &[]).map_err(|diags| {
            AcePyError::new(
                "ace.py/compile",
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
        .map_err(|reject| AcePyError::new("ace.py/view", format!("{reject:?}")))?;

    let provider = ace_a2x_python::PythonTargetProvider;
    let family = Family::Scalar;
    let runtime = provider
        .runtime_source(&view, family)
        .map_err(|reject| AcePyError::new("ace.py/produce", format!("{reject:?}")))?;
    let functions: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let texts = provider
        .produce_fragments(&view, &functions, family)
        .map_err(|reject| AcePyError::new("ace.py/produce", format!("{reject:?}")))?;
    let mut fingerprints = Vec::with_capacity(functions.len());
    for function in &functions {
        let fp = ace_a2x_core::semantic_fingerprint(&view, function)
            .map_err(|reject| AcePyError::new("ace.py/produce", format!("{reject:?}")))?;
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
        .map_err(|reject| AcePyError::new("ace.py/produce", format!("{reject:?}")))?;
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
        .map_err(|reject| AcePyError::new("ace.py/assemble", format!("{reject:?}")))?;
    Ok(AcePyFileSet {
        files: fileset
            .files
            .into_iter()
            .map(|(name, text)| AcePyFile { name, text })
            .collect(),
        manifest: fileset.manifest,
        entry: ace_a2x_python::lower::py_fn_name(entry),
        source_sha256,
    })
}
