//! Suite schema (`autolang.suite/1`), explicit defaults, and selection.
//!
//! Defaults are the ONLY suite-injected context (Design 35 §4.4): `tier` and
//! per-adapter/action `options`. Merging is object-recursive with whole-array
//! replacement; a case's local value always wins; missing required fields
//! remain errors at the case. Suites never copy semantic expectations from
//! one backend to another and never declare new checks.

use std::collections::BTreeMap;

use crate::admission::validate_virtual_path;
use crate::atom::CVal;
use crate::contract::{
    Adapter, BudgetSpec, CheckSpec, ContractError, ContractVersion, Options, AC_TARGET,
    BUDGET_HARD_CAP_BUILD_MS, BUDGET_HARD_CAP_RUN_MS, BUDGET_HARD_CAP_TOTAL_MS,
};

pub const SUITE_SCHEMA: &str = "autolang.suite/1";
pub const SUITE_SCHEMA_V2: &str = "autolang.suite/2";

#[derive(Debug, Clone, PartialEq)]
pub struct SuiteSpec {
    pub schema: String,
    pub id: String,
    /// Case files relative to the suite file (no globbing in v1).
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub defaults: Defaults,
    /// Adapters every selected case must declare a (non-skipped) check for;
    /// enforced by the gate reducer as a required-coverage contract error.
    pub required_adapters: Vec<Adapter>,
    /// Explicit fixture root for `file:` payload and expected-artifact
    /// references, relative to the suite file (default `.`). Referenced
    /// paths themselves stay root-relative with no `..` — the root is the
    /// single, suite-declared pointer (imported corpora keep their files in
    /// place; nothing is copied).
    pub fixture_root: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Defaults {
    pub tier: Option<String>,
    /// adapter -> action -> options object (raw contract value; deep-merged
    /// under the case's own options).
    pub options: BTreeMap<String, BTreeMap<String, CVal>>,
}

/// One defaulted field provenance entry for `inspect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldOrigin {
    pub field: String,
    pub source: &'static str, // "case" | "suite-default" | "implicit"
}

fn err(path: &str, message: impl Into<String>) -> ContractError {
    ContractError {
        path: path.to_string(),
        message: message.into(),
    }
}

fn as_obj<'v>(v: &'v CVal, path: &str) -> Result<&'v [(String, CVal)], ContractError> {
    match v {
        CVal::Obj(pairs) => Ok(pairs),
        other => Err(err(
            path,
            format!("expected object, got {}", other.type_name()),
        )),
    }
}

fn req_str<'v>(
    pairs: &'v [(String, CVal)],
    key: &str,
    path: &str,
) -> Result<String, ContractError> {
    let v = pairs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| err(&format!("{}.{}", path, key), "required field is missing"))?;
    match v {
        CVal::Str(s) => Ok(s.clone()),
        other => Err(err(
            &format!("{}.{}", path, key),
            format!("expected string, got {}", other.type_name()),
        )),
    }
}

fn str_list(pairs: &[(String, CVal)], key: &str, path: &str) -> Result<Vec<String>, ContractError> {
    let v = pairs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| err(&format!("{}.{}", path, key), "required field is missing"))?;
    match v {
        CVal::List(items) => items
            .iter()
            .enumerate()
            .map(|(i, item)| match item {
                CVal::Str(s) => Ok(s.clone()),
                other => Err(err(
                    &format!("{}[{}]", path, i),
                    format!("expected string, got {}", other.type_name()),
                )),
            })
            .collect(),
        other => Err(err(
            &format!("{}.{}", path, key),
            format!("expected array, got {}", other.type_name()),
        )),
    }
}

fn check_known(pairs: &[(String, CVal)], known: &[&str], path: &str) -> Result<(), ContractError> {
    for (k, _) in pairs {
        if !known.contains(&k.as_str()) {
            return Err(err(
                &format!("{}.{}", path, k),
                format!("unknown field {:?} (known fields: {})", k, known.join(", ")),
            ));
        }
    }
    Ok(())
}

/// Decode a `suite { … }` node.
pub fn decode_suite(v: &CVal) -> Result<SuiteSpec, ContractError> {
    let (name, props) = v.as_node().ok_or_else(|| {
        err(
            "suite",
            format!("expected `suite {{ … }}` node, got {}", v.type_name()),
        )
    })?;
    if name != "suite" {
        return Err(err(
            "suite",
            format!("expected node `suite`, got {:?}", name),
        ));
    }
    check_known(
        props,
        &[
            "schema",
            "id",
            "include",
            "exclude",
            "defaults",
            "required",
            "fixture_root",
        ],
        "suite",
    )?;

    let schema = req_str(props, "schema", "suite")?;
    let version = if schema == SUITE_SCHEMA {
        ContractVersion::V1
    } else if schema == SUITE_SCHEMA_V2 {
        ContractVersion::V2
    } else {
        return Err(err(
            "suite.schema",
            format!(
                "unsupported schema {:?} (this pilot implements {}, {})",
                schema, SUITE_SCHEMA, SUITE_SCHEMA_V2
            ),
        ));
    };
    let id = req_str(props, "id", "suite")?;
    if id.is_empty()
        || id.len() > 64
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
    {
        return Err(err(
            "suite.id",
            "invalid suite id (alphanumerics, '_', '-')",
        ));
    }

    let include = str_list(props, "include", "suite.include")?;
    if include.is_empty() {
        return Err(err(
            "suite.include",
            "empty include set — a suite with no case files is a contract error",
        ));
    }
    for inc in &include {
        validate_virtual_path(inc)
            .map_err(|e| err(&format!("suite.include[{:?}]", inc), e.to_string()))?;
    }
    let exclude = match props.iter().find(|(k, _)| k == "exclude") {
        None => Vec::new(),
        Some((_, CVal::List(_))) => str_list(props, "exclude", "suite.exclude")?,
        Some((_, other)) => {
            return Err(err(
                "suite.exclude",
                format!("expected array, got {}", other.type_name()),
            ))
        }
    };

    let mut defaults = Defaults::default();
    if let Some((_, dv)) = props.iter().find(|(k, _)| k == "defaults") {
        let dpairs = as_obj(dv, "suite.defaults")?;
        check_known(dpairs, &["tier", "options"], "suite.defaults")?;
        if let Some((_, tv)) = dpairs.iter().find(|(k, _)| k == "tier") {
            match tv {
                CVal::Str(s) => defaults.tier = Some(s.clone()),
                other => {
                    return Err(err(
                        "suite.defaults.tier",
                        format!("expected string, got {}", other.type_name()),
                    ))
                }
            }
        }
        if let Some((_, ov)) = dpairs.iter().find(|(k, _)| k == "options") {
            let opairs = as_obj(ov, "suite.defaults.options")?;
            let adapter_keys: &[&str] = match version {
                ContractVersion::V1 => &["avm", "a2r"],
                ContractVersion::V2 => &["avm", "a2r", "ac", "syntax"],
            };
            check_known(opairs, adapter_keys, "suite.defaults.options")?;
            for (adapter, av) in opairs {
                let apath = format!("suite.defaults.options.{}", adapter);
                let apairs = as_obj(av, &apath)?;
                let action_keys: &[&str] = match adapter.as_str() {
                    "avm" => &["run"],
                    "a2r" => &["emit"],
                    "ac" => &["check", "build", "run"],
                    // syntax options freeze at T-06 — none are mergeable yet.
                    "syntax" => &[],
                    _ => unreachable!("check_known gates the adapter keys"),
                };
                check_known(apairs, action_keys, &apath)?;
                for (action, val) in apairs {
                    let as_obj_val = as_obj(val, &format!("{}.{}", apath, action))?;
                    // Validate that defaults decode as options for this combo.
                    validate_options_shape(as_obj_val, adapter, action)?;
                    defaults
                        .options
                        .entry(adapter.clone())
                        .or_default()
                        .insert(action.clone(), CVal::Obj(as_obj_val.to_vec()));
                }
            }
        }
    }

    let fixture_root = match props.iter().find(|(k, _)| k == "fixture_root") {
        None => ".".to_string(),
        Some((_, CVal::Str(s))) => s.clone(),
        Some((_, other)) => {
            return Err(err(
                "suite.fixture_root",
                format!("expected string, got {}", other.type_name()),
            ))
        }
    };
    if fixture_root.is_empty()
        || fixture_root.contains('\\')
        || fixture_root.contains(':')
        || fixture_root.starts_with('/')
    {
        return Err(err(
            "suite.fixture_root",
            "must be a non-empty relative path (no drive/absolute/backslash)",
        ));
    }

    let mut required_adapters = Vec::new();
    if let Some((_, rv)) = props.iter().find(|(k, _)| k == "required") {
        let rpairs = as_obj(rv, "suite.required")?;
        check_known(rpairs, &["adapters"], "suite.required")?;
        if let Some((_, av)) = rpairs.iter().find(|(k, _)| k == "adapters") {
            match av {
                CVal::List(items) => {
                    for (i, item) in items.iter().enumerate() {
                        match item {
                            CVal::Str(s) if s == "avm" => required_adapters.push(Adapter::Avm),
                            CVal::Str(s) if s == "a2r" => required_adapters.push(Adapter::A2r),
                            CVal::Str(s) if s == "ac" && version == ContractVersion::V2 => {
                                required_adapters.push(Adapter::Ac)
                            }
                            CVal::Str(s) if s == "syntax" && version == ContractVersion::V2 => {
                                required_adapters.push(Adapter::Syntax)
                            }
                            CVal::Str(other) => {
                                return Err(err(
                                    &format!("suite.required.adapters[{}]", i),
                                    format!(
                                        "unknown adapter {:?} ({} implements avm, a2r{})",
                                        other,
                                        if version == ContractVersion::V2 {
                                            "v2"
                                        } else {
                                            "v1"
                                        },
                                        if version == ContractVersion::V2 {
                                            ", ac, syntax"
                                        } else {
                                            ""
                                        }
                                    ),
                                ))
                            }
                            other => {
                                return Err(err(
                                    &format!("suite.required.adapters[{}]", i),
                                    format!("expected string, got {}", other.type_name()),
                                ))
                            }
                        }
                    }
                }
                other => {
                    return Err(err(
                        "suite.required.adapters",
                        format!("expected array, got {}", other.type_name()),
                    ))
                }
            }
        }
    }

    Ok(SuiteSpec {
        schema,
        id,
        include,
        exclude,
        defaults,
        required_adapters,
        fixture_root,
    })
}

fn validate_options_shape(
    pairs: &[(String, CVal)],
    adapter: &str,
    action: &str,
) -> Result<(), ContractError> {
    let known: &[&str] = match adapter {
        "avm" => &[],
        "ac" => &["profile", "target", "entry", "cache", "budget"],
        "syntax" => &[],
        _ => &["module_name"],
    };
    for (k, v) in pairs {
        if !known.contains(&k.as_str()) {
            return Err(err(
                &format!("suite.defaults.options.{}.{}", adapter, k),
                format!(
                    "unknown option {:?} for {} ({} fields: {})",
                    k,
                    adapter,
                    action,
                    if known.is_empty() {
                        "none".to_string()
                    } else {
                        known.join(", ")
                    }
                ),
            ));
        }
        if k == "module_name" && !matches!(v, CVal::Str(_)) {
            return Err(err(
                &format!("suite.defaults.options.{}.module_name", adapter),
                format!("expected string, got {}", v.type_name()),
            ));
        }
        // ac defaults are fully value-validated HERE (suite level) so that
        // apply_defaults' later infallible reconstruction can never see an
        // illegal merged raw value — case-declared values were already
        // validated by `decode_options`.
        if adapter == "ac" {
            validate_ac_option_value(k, v)?;
        }
    }
    Ok(())
}

fn validate_ac_option_value(k: &str, v: &CVal) -> Result<(), ContractError> {
    let path = format!("suite.defaults.options.ac.{}", k);
    match k {
        "profile" => match v {
            CVal::Str(s) if s == "core" || s == "scalar" => Ok(()),
            CVal::Str(s) => Err(err(
                &path,
                format!(
                    "profile {:?} is not imported in this batch (supported: core, scalar)",
                    s
                ),
            )),
            other => Err(err(
                &path,
                format!("expected string, got {}", other.type_name()),
            )),
        },
        "target" => match v {
            CVal::Str(s) if s == AC_TARGET => Ok(()),
            CVal::Str(s) => Err(err(
                &path,
                format!(
                    "target {:?} is not supported (this batch imports {} only)",
                    s, AC_TARGET
                ),
            )),
            other => Err(err(
                &path,
                format!("expected string, got {}", other.type_name()),
            )),
        },
        "entry" => match v {
            CVal::Str(s)
                if !s.is_empty()
                    && s.len() <= 64
                    && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') =>
            {
                Ok(())
            }
            CVal::Str(_) => Err(err(
                &path,
                "entry must be a function-name identifier ([A-Za-z0-9_], non-empty, <=64 chars)",
            )),
            other => Err(err(
                &path,
                format!("expected string, got {}", other.type_name()),
            )),
        },
        "cache" => match v {
            CVal::Str(s) if s == "fresh" || s == "reuse" || s == "read-only" => Ok(()),
            CVal::Str(s) => Err(err(
                &path,
                format!(
                    "unknown cache mode {:?} (supported: fresh, reuse, read-only)",
                    s
                ),
            )),
            other => Err(err(
                &path,
                format!("expected string, got {}", other.type_name()),
            )),
        },
        "budget" => {
            let bpairs = as_obj(v, &path)?;
            for (bk, bv) in bpairs {
                let cap = match bk.as_str() {
                    "build_ms" => BUDGET_HARD_CAP_BUILD_MS,
                    "run_ms" => BUDGET_HARD_CAP_RUN_MS,
                    "total_ms" => BUDGET_HARD_CAP_TOTAL_MS,
                    other => {
                        return Err(err(
                            &format!("suite.defaults.options.ac.budget.{}", other),
                            format!(
                                "unknown budget field {:?} (known: build_ms, run_ms, total_ms)",
                                other
                            ),
                        ))
                    }
                };
                match bv {
                    CVal::Int(n) if *n > 0 && (*n as u64) <= cap => {}
                    CVal::Int(n) => {
                        return Err(err(
                            &format!("suite.defaults.options.ac.budget.{}", bk),
                            format!(
                                "{} must be in (0, {}] ms — frozen hard cap (got {})",
                                bk, cap, n
                            ),
                        ))
                    }
                    other => {
                        return Err(err(
                            &format!("suite.defaults.options.ac.budget.{}", bk),
                            format!("expected integer, got {}", other.type_name()),
                        ))
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Deep-merge objects (recursive), replace arrays wholesale; local wins.
pub fn deep_merge(base: &CVal, over: &CVal) -> CVal {
    match (base, over) {
        (CVal::Obj(b), CVal::Obj(o)) => {
            let mut merged = b.clone();
            for (k, v) in o {
                if let Some(slot) = merged.iter_mut().find(|(bk, _)| bk == k) {
                    slot.1 = deep_merge(&slot.1.clone(), v);
                } else {
                    merged.push((k.clone(), v.clone()));
                }
            }
            CVal::Obj(merged)
        }
        (_, over) => over.clone(),
    }
}

/// A check with suite defaults applied (tier fallback + options deep-merge),
/// plus per-field origins for `inspect`.
pub fn apply_defaults(check: &CheckSpec, defaults: &Defaults) -> (CheckSpec, Vec<FieldOrigin>) {
    let mut merged = check.clone();
    let mut origins = Vec::new();
    if merged.tier.is_none() {
        if let Some(t) = &defaults.tier {
            merged.tier = Some(t.clone());
            origins.push(FieldOrigin {
                field: "tier".into(),
                source: "suite-default",
            });
        }
    } else {
        origins.push(FieldOrigin {
            field: "tier".into(),
            source: "case",
        });
    }
    let adapter = merged.adapter.as_str();
    let action = merged.action.as_str();
    if let Some(actions) = defaults.options.get(adapter) {
        if let Some(base) = actions.get(action) {
            // Rebuild the case-local options as raw values so every
            // adapter's fields participate in the deep-merge (v2 ac has
            // profile/target/entry/cache/budget; `key: null` marks unset
            // locals and is dropped before merging — suite defaults fill
            // the gaps, the frozen implicit defaults cover the rest at
            // decode time).
            let opt = |v: &Option<String>| v.clone().map(CVal::Str).unwrap_or(CVal::Null);
            let budget_val = |b: &Option<BudgetSpec>| match b {
                None => CVal::Null,
                Some(b) => CVal::Obj(vec![
                    (
                        "build_ms".to_string(),
                        b.build_ms
                            .map(|n| CVal::Int(n as i64))
                            .unwrap_or(CVal::Null),
                    ),
                    (
                        "run_ms".to_string(),
                        b.run_ms.map(|n| CVal::Int(n as i64)).unwrap_or(CVal::Null),
                    ),
                    (
                        "total_ms".to_string(),
                        b.total_ms
                            .map(|n| CVal::Int(n as i64))
                            .unwrap_or(CVal::Null),
                    ),
                ]),
            };
            let local = CVal::Obj(vec![
                (
                    "module_name".to_string(),
                    merged
                        .options
                        .module_name
                        .clone()
                        .map(CVal::Str)
                        .unwrap_or(CVal::Null),
                ),
                (
                    "profile".to_string(),
                    opt(&merged.options.ac_profile.map(|p| p.as_str().to_string())),
                ),
                ("target".to_string(), opt(&merged.options.ac_target)),
                ("entry".to_string(), opt(&merged.options.ac_entry)),
                (
                    "cache".to_string(),
                    opt(&merged.options.ac_cache.map(|c| c.as_str().to_string())),
                ),
                ("budget".to_string(), budget_val(&merged.options.ac_budget)),
            ])
            .filter_nulls();
            let combined = deep_merge(base, &local);
            let pairs = combined.as_obj().unwrap_or(&[]);
            let mut module_name = None;
            let mut from_default_module = false;
            let mut ac_profile = None;
            let mut ac_target = None;
            let mut ac_entry = None;
            let mut ac_cache = None;
            let mut ac_budget = None;
            for (k, v) in pairs {
                match k.as_str() {
                    "module_name" => {
                        if let CVal::Str(s) = v {
                            if merged.options.module_name.is_none() {
                                from_default_module = true;
                            }
                            module_name = Some(s.clone());
                        }
                    }
                    "profile" => {
                        if let CVal::Str(s) = v {
                            ac_profile = Some(s.clone());
                        }
                    }
                    "target" => {
                        if let CVal::Str(s) = v {
                            ac_target = Some(s.clone());
                        }
                    }
                    "entry" => {
                        if let CVal::Str(s) = v {
                            ac_entry = Some(s.clone());
                        }
                    }
                    "cache" => {
                        if let CVal::Str(s) = v {
                            ac_cache = Some(s.clone());
                        }
                    }
                    "budget" => {
                        if let CVal::Obj(bp) = v {
                            let mut b = BudgetSpec::default();
                            for (bk, bv) in bp {
                                let n = match bv {
                                    CVal::Int(n) => Some(*n as u64),
                                    _ => None,
                                };
                                match bk.as_str() {
                                    "build_ms" => b.build_ms = n,
                                    "run_ms" => b.run_ms = n,
                                    "total_ms" => b.total_ms = n,
                                    _ => {}
                                }
                            }
                            ac_budget = Some(b);
                        }
                    }
                    _ => {}
                }
            }
            let had_module = module_name.is_some();
            if merged.adapter == Adapter::Ac {
                // Merged raw values are legal by construction (case values
                // validated at decode_options, suite values at decode_suite),
                // so reconstruction is infallible; absent fields keep the
                // frozen implicit defaults for execution-side `effective_*`.
                merged.options = Options {
                    module_name,
                    ac_profile: None,
                    ac_target: None,
                    ac_entry: None,
                    ac_cache: None,
                    ac_budget: None,
                    ac_capabilities: None,
                    ac_support_lib: None,
                    syntax_grammar: None,
                    syntax_edits: None,
                    probe_recipe: None,
                    probe_steps: None,
                    probe_sema: None,
                    probe_abi: None,
                    probe_av: None,
                };
                // Preserve the DECLARED-vs-default distinction: only
                // store what the merged object actually carries; the rest
                // resolves through effective_* at execution.
                if let Some(p) = ac_profile {
                    merged.options.ac_profile = match p.as_str() {
                        "scalar" => Some(crate::contract::AcProfile::Scalar),
                        _ => Some(crate::contract::AcProfile::Core),
                    };
                }
                if let Some(t) = ac_target {
                    merged.options.ac_target = Some(t);
                }
                if let Some(e) = ac_entry {
                    merged.options.ac_entry = Some(e);
                }
                if let Some(c) = ac_cache {
                    merged.options.ac_cache = match c.as_str() {
                        "fresh" => Some(crate::contract::CacheMode::Fresh),
                        "read-only" => Some(crate::contract::CacheMode::ReadOnly),
                        _ => Some(crate::contract::CacheMode::Reuse),
                    };
                }
                if let Some(b) = ac_budget {
                    merged.options.ac_budget = Some(b);
                }
            } else {
                merged.options = Options {
                    module_name,
                    ac_profile: None,
                    ac_target: None,
                    ac_entry: None,
                    ac_cache: None,
                    ac_budget: None,
                    ac_capabilities: None,
                    ac_support_lib: None,
                    syntax_grammar: None,
                    syntax_edits: None,
                    probe_recipe: None,
                    probe_steps: None,
                    probe_sema: None,
                    probe_abi: None,
                    probe_av: None,
                };
            }
            if had_module {
                origins.push(FieldOrigin {
                    field: "options.module_name".to_string(),
                    source: if from_default_module {
                        "suite-default"
                    } else {
                        "case"
                    },
                });
            }
        }
    }
    (merged, origins)
}

trait FilterNulls {
    fn filter_nulls(self) -> CVal;
}

impl FilterNulls for CVal {
    /// Drop `key: null` placeholders used to represent "unset" locals.
    fn filter_nulls(self) -> CVal {
        match self {
            CVal::Obj(pairs) => CVal::Obj(
                pairs
                    .into_iter()
                    .filter(|(_, v)| !matches!(v, CVal::Null))
                    .map(|(k, v)| (k, v.filter_nulls()))
                    .collect(),
            ),
            CVal::List(items) => {
                CVal::List(items.into_iter().map(FilterNulls::filter_nulls).collect())
            }
            other => other,
        }
    }
}

/// Effective include list after exclusion (order preserved).
pub fn selected_files(suite: &SuiteSpec) -> Vec<String> {
    suite
        .include
        .iter()
        .filter(|inc| !suite.exclude.contains(inc))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suite_text(include: &str, defaults: &str, required: &str) -> String {
        format!(
            "suite {{ schema: \"autolang.suite/1\", id: \"t\", include: {}, exclude: [], defaults: {}, required: {} }}",
            include, defaults, required
        )
    }

    #[test]
    fn decodes_and_selects() {
        let v = crate::atom::parse_contract(&suite_text(
            "[\"a.case.ad\", \"b.case.ad\"]",
            "{ tier: \"vm-corpus\" }",
            "{ adapters: [\"avm\"] }",
        ))
        .unwrap();
        let s = decode_suite(&v).unwrap();
        assert_eq!(s.defaults.tier.as_deref(), Some("vm-corpus"));
        assert_eq!(s.required_adapters, vec![Adapter::Avm]);
        assert_eq!(selected_files(&s), vec!["a.case.ad", "b.case.ad"]);

        let v = crate::atom::parse_contract(&suite_text("[\"a\", \"b\"]", "{}", "{}")).unwrap();
        let mut s = decode_suite(&v).unwrap();
        s.exclude = vec!["a".to_string()];
        assert_eq!(selected_files(&s), vec!["b".to_string()]);
    }

    #[test]
    fn empty_or_unknown_things_error() {
        let v = crate::atom::parse_contract(&suite_text("[]", "{}", "{}")).unwrap();
        assert!(decode_suite(&v)
            .unwrap_err()
            .message
            .contains("empty include"));

        let v = crate::atom::parse_contract(&suite_text(
            "[\"a\"]",
            "{ options: { a2r: { emit: { module_nam: \"x\" } } } }",
            "{}",
        ))
        .unwrap();
        assert!(decode_suite(&v)
            .unwrap_err()
            .message
            .contains("unknown option"));

        let v = crate::atom::parse_contract(&suite_text("[\"../escape.case.ad\"]", "{}", "{}"))
            .unwrap();
        assert!(decode_suite(&v).unwrap_err().message.contains("path"));
    }

    #[test]
    fn deep_merge_objects_recursive_arrays_replace() {
        let base = crate::atom::parse_contract("{ a: { x: 1, y: [1, 2] }, b: \"keep\" }").unwrap();
        let over = crate::atom::parse_contract("{ a: { y: [9], z: 3 } }").unwrap();
        let merged = deep_merge(&base, &over);
        let pairs = merged.as_obj().unwrap();
        let a = pairs
            .iter()
            .find(|(k, _)| k == "a")
            .unwrap()
            .1
            .as_obj()
            .unwrap();
        let get = |k: &str| a.iter().find(|(bk, _)| bk == k).unwrap().1.clone();
        assert_eq!(get("x"), CVal::Int(1)); // kept from base (recursive)
        assert_eq!(get("y"), CVal::List(vec![CVal::Int(9)])); // array replaced wholesale
        assert_eq!(get("z"), CVal::Int(3)); // added
        assert_eq!(
            pairs.iter().find(|(k, _)| k == "b").unwrap().1,
            CVal::Str("keep".into())
        );
    }
}
