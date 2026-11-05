//! Versioned semantic contracts consumed by the real inventory scanner.
//! Inventory evidence never promotes a provider candidate to executed support.
use std::{collections::BTreeSet, path::Path};

use serde::{Deserialize, Serialize};

use super::model::{AssemblyDiagnostic, LayerKind, ModuleInventory, ParseStatus, SymbolKind};

pub const OPERATIONS: &str = include_str!("../../../../stdlib/contracts/operations.toml");
pub const COMPATIBILITY: &str = include_str!("../../../../stdlib/contracts/compatibility.toml");
const MAX_BYTES: usize = 256 * 1024;
pub const INVALID: &str = "STDASSEMBLY.CONTRACT_INVALID";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationCatalog {
    pub schema_version: u32,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub id: String,
    pub semantic_version: u32,
    pub module: String,
    pub symbol: String,
    pub layer: LayerKind,
    pub spelling: String,
    pub caller: String,
    pub native: Option<String>,
    pub visibility: String,
    pub parameters: Vec<String>,
    pub modes: Vec<String>,
    pub returns: String,
    pub unit: String,
    pub dispatch: String,
    pub errors: String,
    pub ownership: String,
    pub requirements: Vec<String>,
    pub profile: String,
    pub oracle: String,
    pub source: String,
    pub followup: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityCatalog {
    pub schema_version: u32,
    pub entries: Vec<Compatibility>,
}

/// Independent compatibility policy: changing an operation's interpretation
/// must also explicitly revise the policy and its source evidence.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    pub operation: String,
    pub unit: String,
    pub dispatch: String,
    pub parameters: Vec<String>,
    pub modes: Vec<String>,
    pub returns: String,
    pub native: Option<String>,
    pub legacy: String,
    pub evidence: String,
    pub ace_av: String,
    pub ace_native: String,
    pub hir_rust: String,
    pub hir_c: String,
    pub host: String,
    pub owner: String,
    pub gap: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ContractInventory {
    pub schema_version: u32,
    pub operations_hash: u64,
    pub compatibility_hash: u64,
    pub rows: Vec<ContractRow>,
    pub providers: Vec<super::providers::ProviderClaim>,
    pub operations: Vec<Operation>,
    pub compatibility: Vec<Compatibility>,
    pub native_bindings: Vec<NativeBinding>,
    pub operation_proofs: Vec<OperationProof>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperationProof {
    pub operation: String,
    /// Policy agreement is not independent implementation evidence.
    pub signature_status: String,
    pub source_file: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NativeBinding {
    pub name: String,
    pub id: Option<u16>,
    pub bound: bool,
    pub producer: Option<String>,
    pub verification: String,
    pub consumer: String,
}

/// Audit the supplied, final production interface, rather than re-creating an
/// approximation of its registration order. Alias names retain separate rows.
pub fn audit_providers(
    inventory: &mut super::model::StdlibInventory,
    interface: &crate::vm::native::NativeInterface,
) {
    let mut names: BTreeSet<String> = crate::vm::native_registry::BIGVM_NATIVES
        .lock()
        .unwrap()
        .get_function_names()
        .into_iter()
        .collect();
    names.extend(interface.id_to_name().into_values());
    names.extend(
        crate::vm::native_catalog::NATIVE_ID_ENTRIES
            .iter()
            .map(|(name, _)| name.to_string()),
    );
    for module in &inventory.modules {
        for layer in &module.layers {
            for symbol in &layer.symbols {
                if matches!(symbol.kind, SymbolKind::Fn | SymbolKind::Method) {
                    names.insert(super::validate::public_native_name(&module.module, symbol));
                }
            }
        }
    }
    inventory.contracts.native_bindings = names
        .into_iter()
        .map(|name| {
            let id = interface
                .resolve(&name)
                .or_else(|| {
                    crate::vm::native_registry::BIGVM_NATIVES
                        .lock()
                        .unwrap()
                        .get_id(&name)
                })
                .or_else(|| {
                    crate::vm::native_catalog::NATIVE_ID_ENTRIES
                        .iter()
                        .find(|(n, _)| *n == name)
                        .map(|(_, id)| *id)
                });
            let bound = id.is_some_and(|id| interface.get(id).is_some());
            let contract = id.and_then(|id| interface.contract(id));
            NativeBinding {
                name,
                id,
                bound,
                producer: contract.map(|c| c.producer.clone()),
                verification: if bound {
                    "bound_semantics_unverified"
                } else if id.is_some() {
                    "resolved_unbound"
                } else {
                    "unknown"
                }
                .into(),
                consumer: "legacy_vm_final_interface".into(),
            }
        })
        .collect();
    for op in &inventory.contracts.operations {
        if let Some(name) = &op.native {
            let expected = inventory
                .contracts
                .compatibility
                .iter()
                .find(|p| p.operation == op.id)
                .and_then(|p| p.native.as_deref());
            let expected_id = expected.and_then(|n| interface.resolve(n));
            if expected_id.is_none() || interface.resolve(name) != expected_id {
                inventory.diagnostics.push(bad(
                    &op.module,
                    format!(
                        "{}: native {name} conflicts with compatibility binding {expected:?}",
                        op.id
                    ),
                ));
            }
            match interface
                .resolve(name)
                .and_then(|id| interface.contract(id))
            {
                Some(contract)
                    if contract
                        .parameters
                        .iter()
                        .map(|s| strict_host_type(s))
                        .collect::<Vec<_>>()
                        != op.parameters
                        || strict_host_type(&contract.returns) != op.returns
                        || contract.parameter_modes != op.modes =>
                {
                    inventory.diagnostics.push(bad(
                        &op.module,
                        format!(
                            "{}: signature/mode conflicts with native {name} ({})",
                            op.id, contract.producer
                        ),
                    ));
                }
                Some(_) => {
                    if !interface
                        .resolve(name)
                        .is_some_and(|id| interface.get(id).is_some())
                    {
                        inventory.diagnostics.push(bad(
                            &op.module,
                            format!("{}: native {name} is not bound", op.id),
                        ));
                    }
                    if let Some(proof) = inventory
                        .contracts
                        .operation_proofs
                        .iter_mut()
                        .find(|p| p.operation == op.id)
                    {
                        proof.signature_status =
                            "producer_signature_checked_semantics_unverified".into();
                    }
                }
                None => inventory.diagnostics.push(bad(
                    &op.module,
                    format!(
                        "{}: native {name} has no independent producer signature",
                        op.id
                    ),
                )),
            }
        }
    }
}

// Only the two representations used by the byte-length adapter are aliases.
// In particular i64/u64/usize must never collapse into source int/i32 here.
fn strict_host_type(name: &str) -> String {
    match name {
        "String" | "&str" => "str".into(),
        "i32" => "int".into(),
        "()" => "void".into(),
        _ => name.into(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContractRow {
    pub module: String,
    pub file: String,
    pub layer: LayerKind,
    /// None retains one failed layer in the denominator; its API count is unknown.
    pub symbol: Option<String>,
    pub native_name: Option<String>,
    pub visibility: String,
    pub operation: Option<String>,
    pub status: String,
    pub consumer: String,
}

fn bad(module: &str, message: impl Into<String>) -> AssemblyDiagnostic {
    AssemblyDiagnostic {
        code: INVALID.into(),
        module: module.into(),
        file: Some("stdlib/contracts".into()),
        message: message.into(),
    }
}

pub fn decode(
    operations: &str,
    compatibility: &str,
) -> Result<(OperationCatalog, CompatibilityCatalog), String> {
    if operations.len() > MAX_BYTES || compatibility.len() > MAX_BYTES {
        return Err("contract exceeds 256 KiB input budget".into());
    }
    let ops: OperationCatalog =
        toml::from_str(operations).map_err(|e| format!("operations: {e}"))?;
    let compat: CompatibilityCatalog =
        toml::from_str(compatibility).map_err(|e| format!("compatibility: {e}"))?;
    if ops.schema_version != 1 || compat.schema_version != 1 {
        return Err("unsupported semantic contract schema".into());
    }
    if ops.operations.len() > 1024 || compat.entries.len() > 1024 {
        return Err("contract exceeds 1024 entry budget".into());
    }
    Ok((ops, compat))
}

pub fn validate(
    catalog: &OperationCatalog,
    compatibility: &CompatibilityCatalog,
    modules: &[ModuleInventory],
) -> Vec<AssemblyDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut ids = BTreeSet::new();
    let mut bindings = BTreeSet::new();
    for op in &catalog.operations {
        let label = format!("{} ({}.{})", op.id, op.module, op.symbol);
        if op.id.is_empty() || !ids.insert(op.id.as_str()) {
            diagnostics.push(bad(
                &op.module,
                format!("{label}: missing/duplicate operation ID"),
            ));
        }
        if !bindings.insert((
            &op.module,
            &op.symbol,
            format!("{:?}", op.layer),
            &op.dispatch,
        )) {
            diagnostics.push(bad(&op.module, format!("{label}: duplicate binding")));
        }
        if op.semantic_version == 0
            || op.parameters.len() != op.modes.len()
            || op
                .modes
                .iter()
                .any(|m| !matches!(m.as_str(), "View" | "Mut" | "Move" | "Take"))
            || [
                &op.module,
                &op.symbol,
                &op.spelling,
                &op.unit,
                &op.dispatch,
                &op.errors,
                &op.ownership,
                &op.profile,
                &op.oracle,
                &op.source,
                &op.followup,
                &op.caller,
                &op.returns,
            ]
            .iter()
            .any(|s| s.trim().is_empty())
            || !matches!(op.visibility.as_str(), "public" | "private" | "native_only")
        {
            diagnostics.push(bad(
                &op.module,
                format!("{label}: invalid operation shape/mode/visibility"),
            ));
        }
        let policies: Vec<_> = compatibility
            .entries
            .iter()
            .filter(|c| c.operation == op.id)
            .collect();
        if policies.len() != 1 {
            diagnostics.push(bad(
                &op.module,
                format!("{label}: expected exactly one compatibility policy"),
            ));
        } else {
            let policy = policies[0];
            if policy.unit != op.unit || policy.dispatch != op.dispatch {
                diagnostics.push(bad(
                    &op.module,
                    format!(
                        "{label}: unit/dispatch conflicts with compatibility policy ({})",
                        policy.evidence
                    ),
                ));
            }
            if policy.parameters != op.parameters
                || policy.modes != op.modes
                || policy.returns != op.returns
            {
                diagnostics.push(bad(
                    &op.module,
                    format!(
                        "{label}: logical signature/mode conflicts with compatibility policy ({})",
                        policy.evidence
                    ),
                ));
            }
            if policy.native.is_some() != op.native.is_some() {
                diagnostics.push(bad(&op.module, format!("{label}: missing/unexpected native evidence required by compatibility policy")));
            }
        }
        // An unparsable layer is a reported gap, never fabricated declaration proof.
        if let Some(layer) = modules
            .iter()
            .find(|m| m.module == op.module)
            .and_then(|m| m.layers.iter().find(|l| l.kind == op.layer))
        {
            if matches!(layer.parse, ParseStatus::Parsed { .. }) {
                match layer.symbols.iter().find(|s| s.name == op.symbol) {
                    Some(sym) => {
                        if let Some(sig) = &sym.signature {
                            if sig.parameters != op.parameters
                                || sig.parameter_modes != op.modes
                                || sig.returns != op.returns
                            {
                                diagnostics.push(bad(
                                    &op.module,
                                    format!(
                                        "{label}: signature/mode conflicts with {}",
                                        layer.file
                                    ),
                                ));
                            }
                        }
                        if (op.visibility == "public" && !sym.is_pub)
                            || (op.visibility == "private" && sym.is_pub)
                        {
                            diagnostics.push(bad(
                                &op.module,
                                format!("{label}: visibility conflicts with {}", layer.file),
                            ));
                        }
                    }
                    None if op.visibility == "native_only" => {}
                    None => diagnostics.push(bad(
                        &op.module,
                        format!("{label}: binding missing from {}", layer.file),
                    )),
                }
            }
        } else {
            diagnostics.push(bad(
                &op.module,
                format!("{label}: declaration layer missing"),
            ));
        }
    }
    let mut policy_ids = BTreeSet::new();
    for policy in &compatibility.entries {
        if !ids.contains(policy.operation.as_str()) || !policy_ids.insert(policy.operation.as_str())
        {
            diagnostics.push(bad(
                "",
                format!(
                    "{}: missing operation ID or duplicate compatibility policy",
                    policy.operation
                ),
            ));
        }
        if [
            &policy.legacy,
            &policy.evidence,
            &policy.owner,
            &policy.gap,
            &policy.host,
        ]
        .iter()
        .any(|v| v.trim().is_empty())
            || [
                &policy.ace_av,
                &policy.ace_native,
                &policy.hir_rust,
                &policy.hir_c,
            ]
            .iter()
            .any(|s| {
                !matches!(
                    s.as_str(),
                    "supported" | "missing_capability" | "unwired" | "unknown"
                )
            })
        {
            diagnostics.push(bad(
                "",
                format!(
                    "{}: incomplete compatibility/owner/status",
                    policy.operation
                ),
            ));
        }
    }
    diagnostics
}

/// Called from scan_inventory, including SDK-selected roots. A standalone
/// fixture with no contracts is explicitly uncontracted; half a pair is invalid.
pub fn scan(
    root: &Path,
    modules: &[ModuleInventory],
) -> (ContractInventory, Vec<AssemblyDiagnostic>) {
    let contracts = root.parent().unwrap_or(root).join("contracts");
    let paths = [
        contracts.join("operations.toml"),
        contracts.join("compatibility.toml"),
    ];
    let mut report = ContractInventory {
        schema_version: 1,
        ..Default::default()
    };
    let mut diagnostics = Vec::new();
    if paths.iter().any(|p| p.exists()) {
        let read = |p: &Path| -> Result<String, String> {
            use std::io::Read;
            let mut text = String::new();
            std::fs::File::open(p)
                .map_err(|e| e.to_string())?
                .take((MAX_BYTES + 1) as u64)
                .read_to_string(&mut text)
                .map_err(|e| e.to_string())?;
            Ok(text)
        };
        match read(&paths[0]).and_then(|ops| read(&paths[1]).map(|compat| (ops, compat))) {
            Ok((ops, compat)) => {
                report.operations_hash = super::model::fnv1a64(&ops);
                report.compatibility_hash = super::model::fnv1a64(&compat);
                match decode(&ops, &compat) {
                    Ok((ops, compat)) => {
                        diagnostics.extend(validate(&ops, &compat, modules));
                        report.operations = ops.operations;
                        report.compatibility = compat.entries;
                    }
                    Err(e) => diagnostics.push(bad("", e)),
                }
            }
            Err(e) => diagnostics.push(bad("", format!("contract read failed: {e}"))),
        }
    }
    report.providers = super::providers::load_catalog()
        .map(|c| c.providers)
        .unwrap_or_default();
    report.operation_proofs = report
        .operations
        .iter()
        .map(|op| {
            let layer = modules
                .iter()
                .find(|m| m.module == op.module)
                .and_then(|m| m.layers.iter().find(|l| l.kind == op.layer));
            let checked = layer
                .and_then(|l| l.symbols.iter().find(|s| s.name == op.symbol))
                .and_then(|s| s.signature.as_ref())
                .is_some_and(|sig| {
                    sig.parameters == op.parameters
                        && sig.parameter_modes == op.modes
                        && sig.returns == op.returns
                });
            OperationProof {
                operation: op.id.clone(),
                signature_status: if checked {
                    "source_signature_checked_semantics_unverified"
                } else {
                    "policy_only_signature_unverified"
                }
                .into(),
                source_file: layer.map(|l| l.file.clone()),
                native: op.native.clone(),
            }
        })
        .collect();
    for module in modules {
        for layer in &module.layers {
            if matches!(layer.parse, ParseStatus::Failed { .. }) {
                report.rows.push(ContractRow {
                    module: module.module.clone(),
                    file: layer.file.clone(),
                    layer: layer.kind,
                    symbol: None,
                    native_name: None,
                    visibility: "unknown".into(),
                    operation: None,
                    status: "parse_failed_api_unknown".into(),
                    consumer: "inventory_only".into(),
                });
            }
            for sym in &layer.symbols {
                let op = report.operations.iter().find(|op| {
                    op.module == module.module && op.layer == layer.kind && op.symbol == sym.name
                });
                report.rows.push(ContractRow {
                    module: module.module.clone(),
                    file: layer.file.clone(),
                    layer: layer.kind,
                    symbol: Some(sym.name.clone()),
                    native_name: matches!(sym.kind, SymbolKind::Fn | SymbolKind::Method)
                        .then(|| super::validate::public_native_name(&module.module, sym)),
                    visibility: if sym.is_pub {
                        "public"
                    } else {
                        "non_public_or_implicit_method"
                    }
                    .into(),
                    operation: op.map(|o| o.id.clone()),
                    status: if op.is_some() {
                        "contract_declared"
                    } else {
                        "missing_contract"
                    }
                    .into(),
                    consumer: "inventory_only".into(),
                });
            }
        }
    }
    (report, diagnostics)
}

#[cfg(test)]
mod tests;
