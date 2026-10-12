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
                message: format!("target layer(s) without a public .at: {}", files.join(", ")),
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
    pub source_file: String,
    pub public_symbol: bool,
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
        let Some(m) = inventory.module(module) else {
            continue;
        };
        for layer in &m.layers {
            if !matches!(layer.kind, LayerKind::Public | LayerKind::Vm) {
                continue;
            }
            for sym in &layer.symbols {
                if !matches!(sym.kind, SymbolKind::Fn | SymbolKind::Method)
                    || !(sym.is_vm_decl || (layer.kind == LayerKind::Public && sym.is_pub))
                {
                    continue;
                }
                let native_name = public_native_name(module, sym);
                let id = registry
                    .get_id(&native_name)
                    .or_else(|| shims.resolve(&native_name));
                let resolved = id.is_some();
                let bound = id.is_some_and(|i| shims.get(i).is_some());
                let io_contract = if *module == "io" && sym.kind == SymbolKind::Method {
                    crate::vm::init_io_module();
                    let method = sym.name.strip_prefix("File.").unwrap_or("");
                    let registry = crate::vm::VM_REGISTRY.lock().unwrap();
                    if method == "open" {
                        registry
                            .get_function("auto.io", "File.open")
                            .filter(|entry| {
                                std::ptr::fn_addr_eq(
                                    entry.func,
                                    crate::vm::io::open as crate::vm::VmFunction,
                                )
                            })
                            .and_then(|_| crate::vm::io::method_contract(method))
                    } else {
                        registry.get_method("File", method).and_then(|selected| {
                            crate::vm::io::selected_method_contract(method, *selected)
                        })
                    }
                } else {
                    None
                };
                let bound = bound || io_contract.is_some();
                let resolved = resolved || io_contract.is_some();
                let mut verification = if bound {
                    VerificationLevel::Bound
                } else if resolved {
                    VerificationLevel::Resolved
                } else {
                    VerificationLevel::Declared
                };
                let (status, reason) = if environment == Environment::Browser {
                    (
                        CoreSymbolStatus::Unsupported,
                        Some("no browser provider/adapter is registered for this symbol".into()),
                    )
                } else if *module == "io" && sym.name == "File.read_buf" {
                    (
                        CoreSymbolStatus::DeclaredStub,
                        Some(
                            "VmModule read_buf_method is an explicit immutable-buffer stub".into(),
                        ),
                    )
                } else if sym.has_body && !sym.is_vm_decl {
                    (
                        CoreSymbolStatus::Supported,
                        Some("Auto body selected; host signature not claimed".into()),
                    )
                } else if let Some(contract) = io_contract
                    .as_ref()
                    .or_else(|| id.and_then(|i| shims.contract(i)))
                {
                    match signature_matches(module, sym, contract, layer.kind) {
                        Ok(()) if bound => {
                            verification = VerificationLevel::SignatureChecked;
                            (CoreSymbolStatus::Supported, None)
                        }
                        Ok(()) => (
                            CoreSymbolStatus::Unverified,
                            Some("producer contract has no callable binding".into()),
                        ),
                        Err(reason) => (CoreSymbolStatus::Unverified, Some(reason)),
                    }
                } else if bound {
                    (
                        CoreSymbolStatus::Unverified,
                        Some(
                            "callee bound; independent logical producer signature unavailable"
                                .into(),
                        ),
                    )
                } else if resolved {
                    (
                        CoreSymbolStatus::Unverified,
                        Some("native name registered but no bound callee".into()),
                    )
                } else {
                    (
                        CoreSymbolStatus::DeclaredStub,
                        Some("declaration has no selected callee".into()),
                    )
                };
                out.push(CoreSymbolValidation {
                    module: (*module).into(),
                    source_file: layer.file.clone(),
                    public_symbol: layer.kind == LayerKind::Public,
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
    out.sort_by(|a, b| (&a.module, &a.symbol).cmp(&(&b.module, &b.symbol)));
    out
}

/// CLI and production consumers use the same target/environment/closure gate.
/// Parse failures in requested modules must not disappear behind an empty
/// symbol list. Unsupported host providers never pass through the VM checker.
pub fn requested_diagnostics(
    inventory: &StdlibInventory,
    target: super::model::AssemblyTarget,
    environment: Environment,
    modules: &[&str],
    validations: &[CoreSymbolValidation],
) -> Vec<AssemblyDiagnostic> {
    use super::model::AssemblyTarget;
    let mut out = Vec::new();
    let catalog = super::providers::load_catalog().ok();
    for module in modules {
        let Some(mi) = inventory.module(module) else {
            out.push(AssemblyDiagnostic {
                code: code::NO_PUBLIC_LAYER.into(),
                module: (*module).into(),
                file: None,
                message: "requested module is absent from the inventory".into(),
            });
            continue;
        };
        let selected_kind = match target {
            AssemblyTarget::Vm => LayerKind::Vm,
            AssemblyTarget::Rust => LayerKind::Rust,
            AssemblyTarget::C => LayerKind::C,
        };
        for layer in mi
            .layers
            .iter()
            .filter(|l| l.kind == LayerKind::Public || l.kind == selected_kind)
        {
            if let super::model::ParseStatus::Failed { error } = &layer.parse {
                out.push(AssemblyDiagnostic {
                    code: code::PARSE_FAIL.into(),
                    module: (*module).into(),
                    file: Some(layer.file.clone()),
                    message: error.clone(),
                });
            }
        }
        if !CORE_MODULES.contains(module) {
            continue;
        }
        if target == AssemblyTarget::Vm {
            out.extend(core_status_diagnostics(validations, Some(&[*module])));
        } else {
            let target_name = match target {
                AssemblyTarget::Rust => "rust",
                AssemblyTarget::C => "c",
                _ => unreachable!(),
            };
            let claim = catalog.as_ref().and_then(|c| {
                c.providers
                    .iter()
                    .find(|p| p.module == *module && p.target == target_name)
            });
            let (code, reason) = if environment == Environment::Browser {
                (
                    code::PROVIDER_UNSUPPORTED,
                    "no browser host adapter is registered".to_string(),
                )
            } else if let Some(claim) = claim {
                (
                    if claim.status == "unsupported" {
                        code::PROVIDER_UNSUPPORTED
                    } else {
                        code::PROVIDER_CLAIM_NO_CALLEE
                    },
                    claim.reason.clone().unwrap_or_else(|| {
                        "host claim lacks independent per-symbol producer verification".into()
                    }),
                )
            } else {
                (
                    code::PROVIDER_CLAIM_NO_CALLEE,
                    "no provider for requested target".into(),
                )
            };
            out.push(AssemblyDiagnostic {
                code: code.into(),
                module: (*module).into(),
                file: None,
                message: reason,
            });
        }
    }
    out
}

/// The name table is the same public dispatch identity used by VM codegen.
/// Layer helper spelling must not create a second, unbound public API.
pub fn public_native_name(module: &str, sym: &SymbolEntry) -> String {
    if module == "http" && sym.kind == SymbolKind::Method {
        if let Some((owner, method)) = sym.name.split_once('.') {
            let prefix = match owner {
                "Server" => "server",
                "Request" => "request",
                "Response" => "response",
                "RequestBuilder" => "request_builder",
                "HTTPStream" => "stream",
                _ => return canonical_native_name(module, sym),
            };
            let module = if owner == "HTTPStream" {
                "http_stream"
            } else {
                "http"
            };
            return format!("auto.{module}.{prefix}_{method}");
        }
    }
    if module == "io" && sym.kind == SymbolKind::Method {
        return canonical_native_name(module, sym);
    }
    if module == "net" && sym.kind == SymbolKind::Method {
        let (owner, method) = sym.name.split_once('.').unwrap_or(("", &sym.name));
        let owner = match owner {
            "TcpListener" => "tcp_listener",
            "TcpStream" => "tcp_stream",
            _ => owner,
        };
        return format!("auto.net.{owner}_{method}");
    }
    if module == "json" {
        let name = sym.name.strip_prefix("JsonValue.").unwrap_or(&sym.name);
        let name = match name {
            "json_get" => "get",
            "json_get_at" => "get_at",
            "json_has_key" => "has_key",
            "json_len" => "len",
            "json_keys" => "keys",
            "json_as_int" => "as_int",
            "json_as_bool" => "as_bool",
            "json_as_string" => "as_string",
            "json_as_number" => "as_number",
            "json_as_array" => "as_array",
            "json_is_null" => "is_null",
            "json_value_type" | "type" => "type",
            _ => name,
        };
        return format!("auto.json.{name}");
    }
    if sym.kind == SymbolKind::Method {
        if let Some((owner, method)) = sym.name.split_once('.') {
            if let Some((_, prefix)) = crate::vm::native_registry::TYPE_CANONICAL_MAP
                .iter()
                .find(|(short, _)| *short == owner)
            {
                return format!("{prefix}.{method}");
            }
        }
    }
    canonical_native_name(module, sym)
}

pub(crate) fn normalize_host_type(ty: &str) -> String {
    let ty = ty.replace(' ', "");
    match ty.as_str() {
        "String" | "&str" | "str" | "Str" => "str".into(),
        "()" | "void" => "void".into(),
        "i32" | "i64" | "u32" | "u64" | "usize" | "int" => "int".into(),
        "f32" | "f64" | "float" => "float".into(),
        _ => {
            if let Some(inner) = ty.strip_prefix("Option<").and_then(|x| x.strip_suffix('>')) {
                return format!("?{}", normalize_host_type(inner));
            }
            if let Some(inner) = ty.strip_prefix("Vec<").and_then(|x| x.strip_suffix('>')) {
                return format!("[]{}", normalize_host_type(inner));
            }
            if let Some(inner) = ty.strip_suffix('?') {
                return format!("?{inner}");
            }
            ty
        }
    }
}

pub(crate) fn signature_matches(
    module: &str,
    sym: &SymbolEntry,
    contract: &crate::vm::native::NativeContract,
    kind: LayerKind,
) -> Result<(), String> {
    let signature = sym
        .signature
        .as_ref()
        .ok_or("source logical signature missing")?;
    if signature.generics != contract.generics
        || signature.parameter_modes != contract.parameter_modes
        || signature.attributes.iter().any(|attr| attr != "vm")
    {
        return Err(
            "producer metadata does not prove generic, ownership or attribute constraints".into(),
        );
    }
    if sym.kind == SymbolKind::Method {
        let owner = sym.name.split('.').next().unwrap_or("");
        if signature.is_static != contract.is_static
            || (!signature.is_static && contract.receiver.as_deref() != Some(owner))
        {
            return Err("independent receiver/static contract unavailable".into());
        }
    }
    if sym.arity != contract.parameters.len() {
        return Err(format!(
            "arity drift: declaration {} / producer {} ({})",
            sym.arity,
            contract.parameters.len(),
            contract.producer
        ));
    }
    for (index, (source, host)) in signature
        .parameters
        .iter()
        .zip(&contract.parameters)
        .enumerate()
    {
        let source = normalize_host_type(source);
        let host = normalize_host_type(host);
        // Net opaque handles cross the host stack as i32. This explicit
        // adapter is independently documented by the TCP shim producer.
        let opaque_adapter = module == "net"
            && index == 0
            && host == "int"
            && matches!(source.as_str(), "TcpListener" | "TcpStream");
        if source != host && !opaque_adapter {
            return Err(format!(
                "parameter {index} drift: {source} / {host} ({})",
                contract.producer
            ));
        }
    }
    let source_ret = normalize_host_type(&signature.returns);
    let host_ret = normalize_host_type(&contract.returns);
    // Legacy VM helpers expose the nullable handle's raw payload. Public
    // declarations retain the option; do not erase option from public checks.
    let layer_adapter = kind == LayerKind::Vm
        && module == "net"
        && host_ret.strip_prefix('?') == Some(source_ret.as_str());
    if source_ret != host_ret && !layer_adapter {
        return Err(format!(
            "return drift: {source_ret} / {host_ret} ({})",
            contract.producer
        ));
    }
    Ok(())
}

/// ID 别名冲突检测（AC-03：ID 复用只限明确别名）。合法别名两形：
/// ① NATIVE_ID_ENTRIES 声明组——catalog (name,id) 表是手工维护的声明面，
///    同 id 全部名字都在该 id 的声明组内（如 file/fs 族裁决②保留的别名）；
/// ② 短别名——某名字恰为另一名字的末段。
/// 两形皆非的共 id = 两 callee 争 ID，报 conflict 而非 last-writer-wins。
///
/// PLAN-738 T-06：结构化组（`IdConflictGroup`）供 CLI 按模块前缀分诊；
/// `id_alias_conflicts` 保持诊断视图。
#[derive(Debug, Clone, serde::Serialize)]
pub struct IdConflictGroup {
    pub id: u16,
    pub names: Vec<String>,
}

pub fn id_alias_conflict_groups(
    registry: &crate::vm::native_registry::AutoVMNativeRegistry,
) -> Vec<IdConflictGroup> {
    use crate::vm::native_catalog::NATIVE_ID_ENTRIES;
    use std::collections::{HashMap, HashSet};

    // 声明组：id → NATIVE_ID_ENTRIES 中该 id 的名字集
    let mut declared: HashMap<u16, HashSet<&str>> = HashMap::new();
    for (name, id) in NATIVE_ID_ENTRIES {
        declared.entry(*id).or_default().insert(name);
    }

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
        // 合法形①：全部名字都在声明组内
        let all_declared = names.iter().all(|n| {
            declared
                .get(&id)
                .map(|s| s.contains(n.as_str()))
                .unwrap_or(false)
        });
        // 合法形②：排序后相邻名字互为末段别名
        let short_aliased = names.windows(2).all(|w| {
            let (a, b) = (w[0].as_str(), w[1].as_str());
            a.rsplit('.').next() == Some(b) || b.rsplit('.').next() == Some(a) || a == b
        });
        let canonical_alias = names
            .iter()
            .map(|name| {
                if !name.starts_with("auto.") {
                    if let Some(canonical) = crate::vm::native::NativeInterface::to_canonical(name)
                    {
                        return canonical;
                    }
                }
                name.clone()
            })
            .collect::<HashSet<_>>()
            .len()
            == 1;
        if !all_declared && !short_aliased && !canonical_alias {
            out.push(IdConflictGroup { id, names });
        }
    }
    out
}

pub fn id_alias_conflicts(
    registry: &crate::vm::native_registry::AutoVMNativeRegistry,
) -> Vec<AssemblyDiagnostic> {
    id_alias_conflict_groups(registry)
        .into_iter()
        .map(|g| AssemblyDiagnostic {
            code: code::NATIVE_ID_CONFLICT.to_string(),
            module: format!("id:{}", g.id),
            file: None,
            message: format!(
                "multiple unrelated names share native id {}: {:?}",
                g.id, g.names
            ),
        })
        .collect()
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

/// PLAN-738 T-07（AC-05/AC-01）：六核心「公开符号 × target × environment」
/// 能力矩阵（§5.6 最终报告的机器面）。
///
/// - vm 腿：`validate_core_vm_bindings` 符号级分类（native/browser 两环境）；
/// - rust / c 腿：provider 目录逐模块 claim（当前目录粒度=模块级；符号级
///   provider 元数据属公共面重写/D3b）+ 该模块公共层公开符号名册。
/// 矩阵完整性契约：六模块 × 四格（vm.native/vm.browser/rust/c）全部在册；
/// 非 Supported 格必有原因（reason/claim.reason 非空）。
pub fn core_target_env_matrix(
    inventory: &StdlibInventory,
    registry: &crate::vm::native_registry::AutoVMNativeRegistry,
    shims: &crate::vm::native::NativeInterface,
) -> serde_json::Value {
    let v_native = validate_core_vm_bindings(inventory, registry, shims, Environment::Native);
    let v_browser = validate_core_vm_bindings(inventory, registry, shims, Environment::Browser);
    let catalog = super::providers::load_catalog().ok();

    let symbol_json = |v: &CoreSymbolValidation| {
        serde_json::json!({
            "symbol": v.symbol,
            "native_name": v.native_name,
            "status": v.status,
            "verification": v.verification,
            "reason": v.reason,
        })
    };

    let modules: serde_json::Map<String, serde_json::Value> = CORE_MODULES
        .iter()
        .map(|m| {
            let mut native_cell: Vec<_> = v_native
                .iter()
                .filter(|v| v.module == *m && v.public_symbol)
                .map(symbol_json)
                .collect();
            let mut browser_cell: Vec<_> = v_browser
                .iter()
                .filter(|v| v.module == *m && v.public_symbol)
                .map(symbol_json)
                .collect();
            let public_symbols: Vec<String> = inventory
                .module(m)
                .map(|mi| {
                    mi.layers
                        .iter()
                        .filter(|l| l.kind == LayerKind::Public)
                        .flat_map(|l| l.symbols.iter().filter(|s| s.is_pub))
                        .map(|s| s.name.clone())
                        .collect()
                })
                .unwrap_or_default();
            // Types and physical fields share the public denominator with callables.
            // They carry declaration evidence only until a target layout is proven.
            for symbol in &public_symbols {
                for (cell, environment) in [(&mut native_cell, Environment::Native),
                    (&mut browser_cell, Environment::Browser)] {
                    if !cell.iter().any(|entry| entry["symbol"] == symbol.as_str()) {
                        cell.push(serde_json::json!({
                            "symbol": symbol, "native_name": null,
                            "status": if environment == Environment::Browser { "unsupported" } else { "unverified" },
                            "verification": "declared",
                            "reason": if environment == Environment::Browser { "no browser layout adapter is registered" }
                                else { "physical type/field representation has declaration evidence only" },
                        }));
                    }
                }
            }
            let claim = |target: &str| {
                catalog
                    .as_ref()
                    .and_then(|c| {
                        c.providers
                            .iter()
                            .find(|p| p.module == *m && p.target == target)
                    })
                    .map(|p| {
                        serde_json::json!({
                            "status": p.status,
                            "kind": p.kind,
                            "locator": p.locator,
                            "reason": p.reason,
                        })
                    })
                    .unwrap_or(serde_json::json!({
                        "status": "unverified",
                        "kind": "none",
                        "locator": null,
                        "reason": "no catalog claim for this target",
                    }))
            };
            (
                m.to_string(),
                serde_json::json!({
                    "vm": { "native": native_cell, "browser": browser_cell },
                    "rust": { "claim": claim("rust"), "public_symbols": public_symbols,
                        "native": host_cell(inventory, m, "rust", Environment::Native),
                        "browser": host_cell(inventory, m, "rust", Environment::Browser) },
                    "c": { "claim": claim("c"), "public_symbols": public_symbols,
                        "native": host_cell(inventory, m, "c", Environment::Native),
                        "browser": host_cell(inventory, m, "c", Environment::Browser) },
                }),
            )
        })
        .collect();

    serde_json::json!({
        "schema_version": 1,
        "core_modules": CORE_MODULES,
        "modules": modules,
    })
}

fn host_cell(
    inventory: &StdlibInventory,
    module: &str,
    target: &str,
    environment: Environment,
) -> Vec<serde_json::Value> {
    let catalog = super::providers::load_catalog().ok();
    let claim = catalog.as_ref().and_then(|c| {
        c.providers
            .iter()
            .find(|p| p.module == module && p.target == target)
    });
    inventory.module(module).into_iter().flat_map(|m| &m.layers)
        .filter(|l| l.kind == LayerKind::Public).flat_map(|l| &l.symbols)
        .filter(|s| s.is_pub).map(|symbol| {
            let unsupported = environment == Environment::Browser || claim.is_some_and(|c| c.status == "unsupported");
            serde_json::json!({
                "symbol": symbol.name, "source_file": format!("stdlib/auto/{module}.at"),
                "status": if unsupported { "unsupported" } else { "unverified" },
                "verification": "declared",
                "reason": if environment == Environment::Browser { "no browser host adapter is registered" }
                    else { claim.and_then(|c| c.reason.as_deref()).unwrap_or("independent host producer signature and routing require verification") },
            })
        }).collect()
}
