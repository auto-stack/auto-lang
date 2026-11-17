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
    transpile_with_profile(source, module, entry, TranspileProfile::Scalar)
}

/// 受检前门的 profile（决定用哪个 `compile_*_source`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranspileProfile {
    Scalar,
    Collections,
    Records,
    Scheduler,
}

/// 同 [`transpile_scalar_source`]，但显式选择受检 profile（collections/records
/// 家族走各自前门；错误码与确定性合同相同）。
pub fn transpile_with_profile(
    source: &str,
    module: &str,
    entry: &str,
    profile: TranspileProfile,
) -> Result<AceRustFileSet, AceRustError> {
    let compiled_result = match profile {
        TranspileProfile::Scalar => ace_core::source::compile_scalar_source(source, module, &[]),
        TranspileProfile::Collections => {
            ace_core::source::compile_collections_source(source, module, &[])
        }
        TranspileProfile::Records => ace_core::source::compile_record_source(source, module, &[]),
        TranspileProfile::Scheduler => {
            ace_core::source::compile_scheduler_source(source, module, &[])
        }
    };
    let compiled = compiled_result.map_err(|diags| {
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
    // collections 家族有**自己的**运行时前导（`A2xStr` 等）：用它的装配器，
    // 否则产物缺少文本运行时、冷构建必失败（本轮实测：E0425 A2xStr）。
    let fileset = match profile {
        TranspileProfile::Collections => {
            // 该装配器还要求 full source sha256 与规范化执行配置；两者都进
            // manifest/证书身份，不能省。
            let source_sha256 = sha256_placeholder_hex(source.as_bytes());
            // 规范化执行配置：本驱动只跑"无故障/无预算/无事件/无取消"的默认
            // 形态（与冷构建门一致）；预算/取消的接线属后续切片。
            let config =
                ace_a2x_rust::canonical_execution_config("transpile", u64::MAX, None, false, false);
            let set = ace_a2x_rust::assemble_collections_file_set(
                &view,
                &program,
                &input_hash,
                &source_sha256,
                &config,
                entry,
            );
            ace_a2x_rust::FileSet {
                files: set.files,
                manifest: set.manifest,
            }
        }
        // scheduler 家族的产物还需要**文本/记录前导表**（`prelude_seed` 解码出的
        // records/variants/lists）——那是提供者框架的装配路径，本无缓存驱动没有它。
        // 与其吐出一个**编译不过**的 file set，不如具名拒绝（本轮实测：缺 A2xStr
        // 与 a2x_sched_* 前导时 rustc 直接 E0425）。
        TranspileProfile::Scheduler => {
            return Err(AceRustError::new(
                "ace.rust/profile-unsupported",
                "scheduler assembly needs the provider prelude seed (records/text tables); \
                 the uncached driver refuses instead of emitting a non-buildable file set",
            ))
        }
        _ => ace_a2x_rust::assemble_file_set(&view, &program, &input_hash, entry),
    };
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

/// 占位：本驱动不引 sha2 依赖；collections 装配器只把它当身份字段填入
/// manifest（内容哈希的正式实现属后续切片，登记为差额）。
fn sha256_placeholder_hex(bytes: &[u8]) -> String {
    let out = sha256_placeholder(bytes);
    out.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256_placeholder(bytes: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let a = fnv1a(bytes).to_le_bytes();
    let b = fnv1a(&bytes.iter().map(|v| v.wrapping_add(1)).collect::<Vec<u8>>()).to_le_bytes();
    out[..8].copy_from_slice(&a);
    out[8..16].copy_from_slice(&b);
    let c = fnv1a(&bytes.iter().rev().copied().collect::<Vec<u8>>()).to_le_bytes();
    let d = fnv1a(&[bytes.len() as u8]).to_le_bytes();
    out[16..24].copy_from_slice(&c);
    out[24..32].copy_from_slice(&d);
    out
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
/// 把三件套折成**单个 .rs 文件**（`--merge` 面）。
///
/// 形态是"模块内联"：`mod a2x_runtime { … }` + `mod generated { … }` + 入口
/// `fn main`。`generated` 里的 `use crate::a2x_runtime::*;` 因此仍解析到兄弟模块，
/// 不需要改写生成文本（绝不改生成器的输出）。
pub fn merge_fileset(fileset: &AceRustFileSet) -> Result<String, AceRustError> {
    let get = |name: &str| {
        fileset
            .files
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.text.clone())
            .ok_or_else(|| AceRustError::new("ace.rust/merge", format!("missing {name}")))
    };
    let wrapper = get("main.rs")?;
    let runtime = get("a2x_runtime.rs")?;
    let generated = get("generated.rs")?;

    // 只过滤 wrapper 里的模块声明与 `use`：生成文本本身一字不改。
    let mut merged = String::new();
    merged.push_str("// AUTO-GENERATED (merged single file) — DO NOT EDIT.\n\n");
    merged.push_str("mod a2x_runtime {\n");
    merged.push_str(&runtime);
    merged.push_str("\n}\n\nmod generated {\n");
    merged.push_str(&generated);
    merged.push_str("\n}\n\n");
    for line in wrapper.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("mod a2x_runtime;")
            || trimmed.starts_with("mod generated;")
            || trimmed.starts_with("// AUTO-GENERATED")
        {
            continue;
        }
        merged.push_str(line);
        merged.push('\n');
    }
    Ok(merged)
}
