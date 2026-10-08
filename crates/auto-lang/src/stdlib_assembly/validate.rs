//! PLAN-738：装配校验。T-02 固化诊断码表与纯结构检查；T-04 增加核心
//! 符号/native 绑定校验、ID 别名冲突检测与 Browser 环境能力分类。

use serde::Serialize;

use super::model::{
    AssemblyDiagnostic, Environment, LayerKind, ModuleInventory, StdlibInventory, SymbolEntry,
    SymbolKind, VerificationLevel,
};

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

// ============================================================================
// T-04：核心符号/native 绑定校验（AC-03/05）
// ============================================================================

/// strict 首期六核心模块（Design 33 §6.9）。
pub const CORE_MODULES: &[&str] = &["io", "net", "async", "http", "json", "sse"];

/// 核心符号状态（§5.3）。`Unsupported` 与 `DeclaredStub` 是诚实覆盖结果，
/// 不是失败；失败面 = 被引用闭包内的 Missing/Conflict/Unverified（T-06 CLI
/// --check 按 requested-closure 出非零）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreSymbolStatus {
    /// 名面解析 + shim 实际绑定（签名等级见 verification）
    Supported,
    /// 声明存在（含 #[vm]）但无任何 native 注册——现存占位（如 sse.parse_sse）
    DeclaredStub,
    /// 名面解析但无绑定 shim——"name 存在即绿"正是本门禁消灭的类别
    Unverified,
    /// 环境/目标不适配（Browser 监听/本地 FS/native socket 三族），reason 必填
    Unsupported,
}

/// 六核心单个 #[vm] 符号的绑定校验结果。
#[derive(Debug, Clone, Serialize)]
pub struct CoreSymbolValidation {
    pub module: String,
    /// 归一符号身份（Type.name / fn）
    pub symbol: String,
    /// canonical native 名（auto.<stem>[.<target>].<fn>，与注册扫描同形状）
    pub native_name: String,
    pub resolved: bool,
    pub bound: bool,
    pub arity: usize,
    /// resolved+bound 且返回类别已知 → SignatureChecked；类别未知 → Bound
    ///（§5.3：变参/self/async 深 ABI 差异不得以 Unknown 假称 signature_checked）
    pub verification: VerificationLevel,
    pub status: CoreSymbolStatus,
    pub reason: Option<String>,
}

/// canonical native 名（与 register_vm_declarations / persistent 对齐：
/// 顶层 = auto.<stem>.<fn>；方法/并入型 = auto.<stem>.<owner_lower>.<fn>）。
pub fn canonical_native_name(module: &str, sym: &SymbolEntry) -> String {
    let stem = module.rsplit('.').next().unwrap_or(module);
    match sym.kind {
        SymbolKind::Method => {
            let (owner, fname) = sym.name.split_once('.').unwrap_or(("", &sym.name));
            format!("auto.{}.{}.{}", stem, owner.to_lowercase(), fname)
        }
        _ => format!("auto.{}.{}", stem, sym.name),
    }
}

/// Browser 环境下无适配、必须显式 Unsupported 的三族（§5.3：监听/本地
/// FS/native sockets；json/async/sse 解析面与 http 客户端面不在此列）。
fn browser_unsupported_reason(module: &str, native_name: &str) -> Option<String> {
    match module {
        "io" => Some("Browser 无本地文件系统适配（io 全族 Unsupported）".to_string()),
        "net" => Some("Browser 无 native socket（net 全族 Unsupported）".to_string()),
        "http" => {
            if native_name.contains("server") || native_name.contains("listen") {
                Some("Browser 不可监听服务端口（http server 族 Unsupported）".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 六核心 #[vm] 声明 × 实际注册/绑定面校验（AC-03：name→ID resolution 与
/// NativeInterface 真实 shim 绑定同时成立才 Supported；避免 name 存在即绿）。
///
/// `registry`/`shims` 应为生产 init 同款构造（register_builtin_natives 的
/// 磁盘扫描面 + register_std_shims/register_stdlib_ffi 手工面）。
pub fn validate_core_vm_bindings(
    inventory: &StdlibInventory,
    registry: &crate::vm::native_registry::AutoVMNativeRegistry,
    shims: &crate::vm::native::NativeInterface,
    environment: Environment,
) -> Vec<CoreSymbolValidation> {
    let mut out = Vec::new();
    for module in CORE_MODULES {
        let Some(m) = inventory.module(module) else { continue };
        for layer in &m.layers {
            // VM provider 面 = #[vm] 声明（.vm.at 层全量 + 公共层 legacy 属性）
            if !matches!(layer.kind, LayerKind::Public | LayerKind::Vm) {
                continue;
            }
            for sym in &layer.symbols {
                if !sym.is_vm_decl {
                    continue;
                }
                let native_name = canonical_native_name(module, sym);
                let id = registry.get_id(&native_name);
                let resolved = id.is_some();
                let bound = id.map(|i| shims.get(i).is_some()).unwrap_or(false);

                if environment == Environment::Browser {
                    if let Some(reason) = browser_unsupported_reason(module, &native_name) {
                        out.push(CoreSymbolValidation {
                            module: (*module).to_string(),
                            symbol: sym.name.clone(),
                            native_name,
                            resolved,
                            bound,
                            arity: sym.arity,
                            verification: VerificationLevel::Declared,
                            status: CoreSymbolStatus::Unsupported,
                            reason: Some(reason),
                        });
                        continue;
                    }
                }

                let (verification, status, reason) = if resolved && bound {
                    // 签名等级：返回类别已知才算 SignatureChecked（§5.3 诚实
                    // 等级；深 ABI 槽宽/生命周期属 D3b，不在此自称）
                    let ret_known = registry.get_return_type(&native_name).is_some();
                    let v = if ret_known {
                        VerificationLevel::SignatureChecked
                    } else {
                        VerificationLevel::Bound
                    };
                    (v, CoreSymbolStatus::Supported, None)
                } else if resolved {
                    (
                        VerificationLevel::Resolved,
                        CoreSymbolStatus::Unverified,
                        Some("native name registered but no bound shim callable".to_string()),
                    )
                } else {
                    (
                        VerificationLevel::Declared,
                        CoreSymbolStatus::DeclaredStub,
                        Some("declared #[vm] but no native registration exists".to_string()),
                    )
                };
                out.push(CoreSymbolValidation {
                    module: (*module).to_string(),
                    symbol: sym.name.clone(),
                    native_name,
                    resolved,
                    bound,
                    arity: sym.arity,
                    verification,
                    status,
                    reason,
                });
            }
        }
    }
    out
}

/// ID 别名冲突检测（AC-03：ID 复用只限明确别名——短名 = canonical 末段；
/// 同 ID 的两个名字若互不为对方末段，即两 callee 争 ID，报 conflict 而非
/// last-writer-wins）。
pub fn id_alias_conflicts(
    registry: &crate::vm::native_registry::AutoVMNativeRegistry,
) -> Vec<AssemblyDiagnostic> {
    use std::collections::HashMap;
    let mut by_id: HashMap<u16, Vec<String>> = HashMap::new();
    for name in registry.get_function_names() {
        if let Some(id) = registry.get_id(&name) {
            by_id.entry(id).or_default().push(name);
        }
    }
    let mut out = Vec::new();
    let mut ids: Vec<_> = by_id.keys().copied().collect();
    ids.sort();
    for id in ids {
        let mut names = by_id[&id].clone();
        names.sort();
        if names.len() < 2 {
            continue;
        }
        let aliased = names.windows(2).all(|w| {
            let (a, b) = (w[0].as_str(), w[1].as_str());
            a.rsplit('.').next() == Some(b)
                || b.rsplit('.').next() == Some(a)
                || a == b
        });
        if !aliased {
            out.push(AssemblyDiagnostic {
                code: code::NATIVE_ID_CONFLICT.to_string(),
                module: format!("id:{id}"),
                file: None,
                message: format!("multiple unrelated names share native id {id}: {names:?}"),
            });
        }
    }
    out
}

/// 六核心符号状态 → 诊断视图（供 CLI --check / manifest 消费；Supported
/// 不出诊断，Unsupported/DeclaredStub/Unverified 各自带原因）。
pub fn core_status_diagnostics(
    validations: &[CoreSymbolValidation],
    only_modules: Option<&[&str]>,
) -> Vec<AssemblyDiagnostic> {
    let mut out = Vec::new();
    for v in validations {
        if let Some(mods) = only_modules {
            if !mods.contains(&v.module.as_str()) {
                continue;
            }
        }
        let code_str = match v.status {
            CoreSymbolStatus::Supported => continue,
            CoreSymbolStatus::Unsupported => code::PROVIDER_UNSUPPORTED,
            CoreSymbolStatus::DeclaredStub => code::PROVIDER_CLAIM_NO_CALLEE,
            CoreSymbolStatus::Unverified => code::SIGNATURE_DRIFT,
        };
        out.push(AssemblyDiagnostic {
            code: code_str.to_string(),
            module: v.module.clone(),
            file: None,
            message: format!(
                "{} ({}): {}",
                v.native_name,
                v.symbol,
                v.reason.as_deref().unwrap_or("")
            ),
        });
    }
    out
}
