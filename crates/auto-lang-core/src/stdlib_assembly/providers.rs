//! PLAN-738：provider 声明目录装载与机器对照。
//!
//! `stdlib/assembly-providers.json` 声明「谁承担哪个模块哪个目标」。目录
//! 不是第二份"声称已实现"表——每个 supported 声明必须能被真实面机器核对：
//! - vm / native-shim → BIGVM_NATIVES 名 surface 存在 `locator` 前缀名；
//! - rust / a2r-std → a2r-std crate 存在 `pub mod <leaf>`；
//! - c / c-generated → locator 指向的 `.c.at`（或生成产物）在磁盘存在；
//! - unsupported/unverified → reason 必填。
//!
//! 对照失败产出 `validate::code::PROVIDER_CLAIM_NO_CALLEE` 诊断，不静默。

use serde::{Deserialize, Serialize};

use super::model::AssemblyDiagnostic;
use super::validate::code;

pub const ASSEMBLY_PROVIDERS_JSON: &str =
    include_str!("../../../../stdlib/assembly-providers.json");

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderCatalog {
    pub schema_version: u32,
    pub providers: Vec<ProviderClaim>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderClaim {
    pub module: String,
    /// "vm" | "rust" | "c"
    pub target: String,
    /// "native-shim" | "a2r-std" | "c-generated" | "none"
    pub kind: String,
    /// "supported" | "unsupported" | "unverified"
    pub status: String,
    pub locator: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

pub fn load_catalog() -> Result<ProviderCatalog, String> {
    let catalog: ProviderCatalog = serde_json::from_str(ASSEMBLY_PROVIDERS_JSON)
        .map_err(|e| format!("assembly-providers.json parse failed: {e}"))?;
    Ok(catalog)
}

/// PLAN-738 T-05：provider 目录 schema 版本（进程内缓存一次——目录经
/// include_str! 编译期内嵌，运行时不变）。缓存条目身份含此版本：目录
/// 声明变更（bump schema_version）→ 既有缓存条目全量失效。
pub fn catalog_schema_version() -> u32 {
    static SCHEMA: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *SCHEMA.get_or_init(|| load_catalog().map(|c| c.schema_version).unwrap_or(0))
}

impl ProviderClaim {
    pub fn is_supported(&self) -> bool {
        self.status == "supported"
    }
}

/// 目录自检（无外部依赖的形状约束）：
/// - unsupported/unverified 必须 reason 非空；
/// - target/kind/status 词表约束；
/// - supported 必须 locator 非空。
pub fn shape_checks(catalog: &ProviderCatalog) -> Vec<AssemblyDiagnostic> {
    let mut out = Vec::new();
    for c in &catalog.providers {
        let bad = |msg: String| AssemblyDiagnostic {
            code: code::PROVIDER_CLAIM_NO_CALLEE.to_string(),
            module: c.module.clone(),
            file: Some("stdlib/assembly-providers.json".to_string()),
            message: msg,
        };
        if !matches!(c.target.as_str(), "vm" | "rust" | "c") {
            out.push(bad(format!("unknown target '{}'", c.target)));
        }
        if !matches!(
            c.status.as_str(),
            "supported" | "unsupported" | "unverified"
        ) {
            out.push(bad(format!("unknown status '{}'", c.status)));
        }
        if matches!(c.status.as_str(), "unsupported" | "unverified")
            && c.reason.as_deref().map(str::is_empty).unwrap_or(true)
        {
            out.push(bad(format!("{} claim requires non-empty reason", c.status)));
        }
        if c.is_supported() && c.locator.is_none() {
            out.push(bad("supported claim requires locator".to_string()));
        }
    }
    out
}

/// 对照真实面（T-02 机器对照的核心）：
/// - rust/a2r-std supported：a2r-std lib.rs 存在 `pub mod <leaf>`；
/// - c/c-generated supported：locator 的 stdlib 相对文件存在；
/// - vm/native-shim supported：`vm_names`（真实注册名 surface）存在以
///   locator 前缀（剥 `*`）开头的名字。
pub fn cross_check_real_surfaces(
    catalog: &ProviderCatalog,
    vm_names: &[String],
    a2r_lib_rs: &str,
    stdlib_root: &std::path::Path,
) -> Vec<AssemblyDiagnostic> {
    let mut out = Vec::new();
    for c in &catalog.providers {
        if !c.is_supported() {
            continue;
        }
        let Some(locator) = &c.locator else { continue };
        let bad = |msg: String| AssemblyDiagnostic {
            code: code::PROVIDER_CLAIM_NO_CALLEE.to_string(),
            module: c.module.clone(),
            file: Some("stdlib/assembly-providers.json".to_string()),
            message: msg,
        };
        match (c.target.as_str(), c.kind.as_str()) {
            ("vm", "native-shim") => {
                let prefix = locator.trim_end_matches('*');
                let hit = vm_names.iter().any(|n| n.starts_with(prefix));
                if !hit {
                    out.push(bad(format!(
                        "vm locator '{locator}' matches no registered native name"
                    )));
                }
            }
            ("rust", "a2r-std") => {
                let leaf = locator.rsplit("::").next().unwrap_or(locator);
                let marker = format!("pub mod {leaf}");
                if !a2r_lib_rs.contains(&marker) {
                    out.push(bad(format!(
                        "a2r-std has no '{marker}' for locator '{locator}'"
                    )));
                }
            }
            ("c", "c-generated") => {
                // locator 允许仓根相对（stdlib/auto/...）或 stdlib 根相对（auto/...）
                let rel = locator
                    .strip_prefix("stdlib/auto/")
                    .unwrap_or(locator.trim_start_matches("stdlib/"));
                if !stdlib_root.join(rel).exists() {
                    out.push(bad(format!("c locator '{locator}' not on disk")));
                }
            }
            _ => {}
        }
    }
    out
}
pub fn catalog_content_fingerprint() -> u64 {
    super::model::fnv1a64(include_str!("../../../../stdlib/assembly-providers.json"))
}

pub fn unsupported_reason(module: &str, target: &str) -> Option<String> {
    if !super::validate::CORE_MODULES.contains(&module) {
        return None;
    }
    load_catalog()
        .ok()?
        .providers
        .into_iter()
        .find(|p| p.module == module && p.target == target && p.status == "unsupported")?
        .reason
}

pub fn require_reference_provider(
    module: &str,
    symbol: &str,
    target: &str,
) -> crate::AutoResult<()> {
    if let Some(reason) = unsupported_reason(module, target) {
        return Err(crate::AutoError::Msg(format!(
            "STDASSEMBLY.PROVIDER_UNSUPPORTED: {module}.{symbol}/{target}: {reason}"
        )));
    }
    Ok(())
}
