//! PLAN-982（982/RUST I，第二步）：**A2R 的宿主驱动面**。
//!
//! 产品侧（`auto trans -t rust`、`auto-man-runtime::target`）目前仍走旧
//! `transpile_rust*`。本模块是两者之间的**唯一边界**（与配置桥同层）：
//! 源码 → 受检 HIR（`ace_core::source` 前门）→ `SemanticView` 准入 →
//! `lower_program` → `assemble_file_set`，产出一个**无缓存、确定性**的
//! `AceRustFileSet`（冻结的 wrapper/runtime/generated 三件套 + manifest）。
//!
//! 合同（与配置桥一致）：
//! - 每个失败都是**具名**的（`ace.rust/view`、`ace.rust/lower`、`ace.rust/entry`），
//!   绝不返回"空 file set"或静默降级到旧转译；
//! - 同输入**逐字节确定**（下游逐文件 diff 的前提）；
//! - 不隐瞒缓存：本驱动**不做** fragment 缓存（提供者框架的缓存键属后续切片），
//!   因此每次调用都真实 lower + print。

use ace_a2x_core::SemanticView;

/// 一个产出文件（相对路径 + 文本）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AceRustFile {
    pub name: String,
    pub text: String,
}

/// 一次 A2R 渲染的完整结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AceRustFileSet {
    pub files: Vec<AceRustFile>,
    /// 冻结 schema 的 manifest（`auto.a2x.rust.fileset/1`）。
    pub manifest: String,
    /// 降级后的 Rust 入口名（源名 `main` ⇒ `fn_main`）。
    pub entry: String,
}

/// A2R 驱动的具名失败。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AceRustError {
    pub code: &'static str,
    pub message: String,
}

impl AceRustError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AceRustError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// 把一段真实 Auto 源码渲染成 Rust file set。
///
/// `entry` 是**源名**（文档里的函数名，如 `main`）；`module` 只用于诊断与
/// input hash 的命名。产出是**确定**的：同输入两次调用逐字节一致。
pub fn transpile_scalar_source(
    source: &str,
    module: &str,
    entry: &str,
) -> Result<AceRustFileSet, AceRustError> {
    let compiled =
        ace_core::source::compile_scalar_source(source, module, &[]).map_err(|diags| {
            AceRustError::new(
                "ace.rust/compile",
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
        .map_err(|reject| AceRustError::new("ace.rust/view", format!("{reject:?}")))?;
    let program = ace_a2x_rust::lower_program(&view, entry)
        .map_err(|reject| AceRustError::new("ace.rust/lower", format!("{reject:?}")))?;
    let input_hash = format!("{:016x}", fnv1a(source.as_bytes()));
    let fileset = ace_a2x_rust::assemble_file_set(&view, &program, &input_hash, entry);
    Ok(AceRustFileSet {
        files: fileset
            .files
            .into_iter()
            .map(|(name, text)| AceRustFile { name, text })
            .collect(),
        manifest: fileset.manifest,
        entry: program.entry,
    })
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
