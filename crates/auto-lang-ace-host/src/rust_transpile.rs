//! PLAN-982（982/RUST I，第二步）：**A2R 的宿主驱动面**。
//!
//! 产品侧（`auto trans -t rust`、`auto-man-runtime::target`）目前仍走旧
//! `transpile_rust*`。本模块是两者之间的**唯一边界**（与配置桥同层）：
//! 源码 → 受检 HIR（`ace_core::source` 前门）→ `SemanticView` 准入 →
//! `lower_program` → `assemble_file_set`，产出一个**无缓存、确定性**的
//! `AceRustFileSet`（冻结的 wrapper/runtime/generated 三件套 + manifest）。
//!
//! PLAN-1117 T-02：`AUTO_ACE_RUST=ace` 绑定的候选路由（兄弟 CLI 注册的
//! `transpile_scalar_source`）改走**普通语言通道**——`ac.source.language.av/v1`
//! 准入 + 实例化/CT 准备钩子 + `lower_language_program` + 语言 fileset schema
//! （PLAN-1115/1116 交付的 v0.5 替换面）。
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
///
/// PLAN-1117 T-02：候选路由改走**普通语言通道**（PLAN-1115/1116 的
/// `ac.source.language.av/v1` 面 + 语言 fileset schema）——v0.5 替换分母覆盖的
/// 就是这个面。标量前门仍可用（`transpile_with_profile(.., Scalar)`），但用户
/// 入口绑定的本函数不再以它为准。
pub fn transpile_scalar_source(
    source: &str,
    module: &str,
    entry: &str,
) -> Result<AceRustFileSet, AceRustError> {
    transpile_with_profile(source, module, entry, TranspileProfile::Language)
}

/// 受检前门的 profile（决定用哪个 `compile_*_source`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranspileProfile {
    Scalar,
    Collections,
    Records,
    Scheduler,
    /// PLAN-1117：普通语言面（`ac.source.language.av/v1` 准入 + 实例化/CT
    /// 准备钩子 + `lower_language_program` + 语言 fileset schema）。
    Language,
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
        // PLAN-1117 T-02：普通语言前门——准入键 `ac.source.language.av/v1`
        // （源含 `#{...}` 时补 CT 键，同 a2x-worker language 车道的 caps 形），
        // 准备钩子与 AV/worker 车道同一对共享 pass：泛型实例化
        // （`instantiate_auto_program`）+ CT 单门（`prepare_auto_ct`）。
        TranspileProfile::Language => {
            let mut caps = vec![ace_core::source::AUTO_SOURCE_ADMISSION_CAP.to_string()];
            if source.contains('#') {
                caps.push(ace_core::source::cst::CT_CAPABILITY.to_string());
            }
            ace_core::source::adapter::compile_language_source_prepared(
                source,
                module,
                &caps,
                |program, projected_source, has_instances| {
                    if has_instances {
                        ace_source_query::instantiate_auto_program(program, module, &caps)?;
                    }
                    if source.contains('#') {
                        let mut service =
                            ace_source_cte::CtService::new(ace_source_cte::MemoryEnvelopeSink::new());
                        let _counters = service.counters();
                        ace_source_cte::product::prepare_auto_ct(
                            &mut service,
                            program,
                            projected_source,
                            module,
                            &caps,
                        )?;
                    }
                    Ok(())
                },
            )
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
    // PLAN-1117：语言通道用自己的整模块 lowerer（跨帧错误槽 + 语言陷阱面）；
    // 其余 profile 保持原冻结 lowerer。
    let program = match profile {
        TranspileProfile::Language => ace_a2x_rust::lower_language_program(&view, entry),
        _ => ace_a2x_rust::lower_program(&view, entry),
    }
    .map_err(|reject| AceRustError::new("ace.rust/lower", format!("{reject:?}")))?;
    let input_hash = format!("{:016x}", fnv1a(source.as_bytes()));
    // collections 家族有**自己的**运行时前导（`A2xStr` 等）：用它的装配器，
    // 否则产物缺少文本运行时、冷构建必失败（本轮实测：E0425 A2xStr）。
    let fileset = match profile {
        TranspileProfile::Collections => {
            // 该装配器还要求 full source sha256 与规范化执行配置；两者都进
            // manifest/证书身份，不能省。
            let source_sha256 = source_sha256_hex(source.as_bytes());
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
        // PLAN-1117 T-02：语言通道装配器——同 collections 的身份纪律（真实
        // full-source sha256 + 规范化执行配置），落在语言通道自己的 fileset
        // schema（`auto.a2x.rust.language.fileset/1`）上。
        TranspileProfile::Language => {
            let source_sha256 = source_sha256_hex(source.as_bytes());
            let config =
                ace_a2x_rust::canonical_execution_config("transpile", u64::MAX, None, false, false);
            let set = ace_a2x_rust::assemble_language_file_set(
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

/// 真实的内容 sha256（collections 装配器的 `full_source_sha256` 身份字段）。
///
/// round-28 该位置是**占位**（fnv1a 派生），会污染 manifest/证书身份；本轮换成
/// 真实摘要，并用 NIST 向量在测试里钉住。
pub(crate) fn source_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// PLAN-982（982/RUST · round 200）：**包级 A2R 渲染**——把整个多模块工程
/// 一次编译成一个完整 file set（镜像 A2C 的 `transpile_package_c`）。
///
/// `modules` = `(模块名, 源码)` 列表（工程全部 `.at`），`entry` = 入口模块名
/// （如 `"main"`）。标准库（`std.math`/`std.text`）由本驱动按冻结输入注入，
/// 调用方只给用户模块；跨模块 `use` 由 packages 绑定图承载。链路：packages
/// 前门（`compile_package_product`）→ `PackageA2xTarget::cold` ＋
/// `RustTargetProvider`（HIR→Rust，RecordText 家族）→ fileset（`generated.rs`
/// ＋`a2x_runtime.rs`＋`main.rs` 包装＋manifest）。非 UI 面——Iced 工程另留
/// 1074 联合验收。
pub fn transpile_package_rust(
    modules: &[(String, String)],
    entry: &str,
) -> Result<AceRustFileSet, (&'static str, String)> {
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::packages::PackageA2xTarget;
    use ace_core::source::cst::FrontBudget;
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
    use ace_source_query::modules::packages::{compile_package_product, stdlib_inputs};

    let mut inputs = stdlib_inputs();
    for (id, source) in modules {
        inputs.push(PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: id.clone(),
            source: source.clone(),
            provider: None,
        });
    }
    let entry_id = format!("app::{entry}");
    let product = compile_package_product(
        &inputs,
        &entry_id,
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .map_err(|diags| {
        let first = diags.first();
        let message = match first {
            Some(d) => format!(
                "package '{}': {} {}",
                d.module, d.diagnostic.code, d.diagnostic.message
            ),
            None => "the package compile failed without a diagnostic".to_string(),
        };
        ("ace.rust/compile", message)
    })?;

    // store 冷启动：同输入 ⇒ 同 fileset（与 A2C 驱动同一纪律）。
    let store = std::env::temp_dir().join(format!("ace-a2r-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    std::fs::create_dir_all(&store).map_err(|e| {
        (
            "ace.rust/emit",
            format!("scratch store dir {}: {e}", store.display()),
        )
    })?;
    let target = PackageA2xTarget::cold(
        &store,
        "auto-a2r",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .map_err(|e| ("ace.rust/emit", e))?;
    let files: Vec<AceRustFile> = target
        .current
        .files
        .iter()
        .map(|(name, text)| AceRustFile {
            name: name.clone(),
            text: text.clone(),
        })
        .collect();
    let fileset = AceRustFileSet {
        files,
        manifest: target.current.manifest.clone(),
        entry: entry.to_string(),
    };
    let _ = std::fs::remove_dir_all(&store);
    Ok(fileset)
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
