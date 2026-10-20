//! Old-report staleness check (Plan 773 F-06).
//!
//! A persisted report's PASS is only trustworthy against the tree it was
//! produced from. `check` re-runs the real load+resolve pipeline for the
//! CURRENT suite and compares every identity axis per result — suite file
//! bytes, case-file bytes, resolved source bytes, oracle CONTENT (sidecar
//! bytes included), the case contract, and the toolchain/config identity.
//! Any drift marks the result STALE; an old PASS never carries over.

use std::collections::BTreeMap;
use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::discovery::load_suite;
use auto_test_case::ir::{resolve_case, resolve_oracle};
use auto_test_case::report::IdentityHashes;
use auto_test_case::sha256::{hex, sha256};
use auto_test_case::suite::apply_defaults;

/// One row of the staleness summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowVerdict {
    Fresh {
        case: String,
        check: String,
    },
    Stale {
        case: String,
        check: String,
        drifted_axes: Vec<String>,
    },
}

/// R3 (F05): which native identity axes APPLY to one row, derived from the
/// check's ACTUAL consumption path — not a single global set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NativeApplicability {
    /// "ac" | "syntax" (only these carry native records).
    pub adapter: String,
    /// ac/build|run from source/hir: the lld/sdk content axes apply IFF the
    /// row actually linked (obj/exe produced or a run observed). Product
    /// consumers and check rows never link.
    pub links_conditionally: bool,
    /// The contract declares the trace-marks support library — its CONTENT
    /// hash is an actual link input and binds the row.
    pub support_lib: bool,
}

fn fingerprint(v: &dyn std::fmt::Debug) -> String {
    hex(&sha256(format!("{:?}", v).as_bytes()))
}

/// Current toolchain/config identity, axis by axis (mirrors what the runner
/// binds into fresh reports).
pub fn toolchain_axes(limits: &Limits) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert(
        "auto-test-case".to_string(),
        auto_test_case::CORE_VERSION.to_string(),
    );
    m.insert(
        "vendored-parser".to_string(),
        auto_test_case::document::vendored_parser_fingerprint(),
    );
    m.insert(
        "auto-lang".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    m.insert("limits".to_string(), fingerprint(&limits));
    // PLAN-788: the ac worker IPC identity — a protocol revision invalidates
    // every prior native result row (per-check worker/toolchain builds also
    // travel in each row's native record).
    m.insert("ac-case-worker".to_string(), "acworker/1".to_string());
    m
}

/// Recompute the current identity of every declared check through the real
/// pipeline. Checks whose references no longer resolve get an identity with
/// an empty oracle hash AND an extra marker axis so they are unambiguously
/// stale rather than silently equal. R3 (F05): each entry also carries the
/// row's native-axis APPLICABILITY (adapter / conditional linking / support
/// lib) so the comparison uses the axes the row actually consumed.
pub fn current_identities(
    suite_path: &Path,
    limits: &Limits,
) -> Result<BTreeMap<String, (IdentityHashes, NativeApplicability)>, String> {
    let loaded = load_suite(suite_path, limits).map_err(|e| format!("suite load: {}", e))?;
    let mut out = BTreeMap::new();
    for file in &loaded.cases {
        for spec in &file.specs {
            let resolved = resolve_case(spec.clone(), &file.doc, &loaded.fixture_base, limits).ok();
            for check in &spec.checks {
                let (check, _) = apply_defaults(check, &loaded.suite.defaults);
                let oracle = resolve_oracle(&spec.id, &check, &loaded.fixture_base, limits).ok();
                let identity = IdentityHashes {
                    suite: loaded.suite_sha256.clone(),
                    case_file: file.sha256.clone(),
                    source: resolved
                        .as_ref()
                        .map(|r| r.source_sha256.clone())
                        .unwrap_or_default(),
                    oracle: match &oracle {
                        Some(o) => o.oracle_sha256.clone(),
                        // Unresolvable oracle: hash a marker so any old
                        // value (or another unresolvable state) is stale.
                        None => hex(&sha256(b"<unresolvable-oracle>")),
                    },
                    contract: fingerprint(spec),
                };
                let applicability = NativeApplicability {
                    adapter: check.adapter.as_str().to_string(),
                    links_conditionally: check.adapter == auto_test_case::contract::Adapter::Ac
                        && check.product_from.is_none()
                        && matches!(
                            check.action,
                            auto_test_case::contract::Action::Build
                                | auto_test_case::contract::Action::Run
                        ),
                    support_lib: check.options.effective_support_lib().is_some(),
                };
                out.insert(
                    format!("{}:{}", spec.id, check.id),
                    (identity, applicability),
                );
            }
        }
    }
    Ok(out)
}

/// Compare a persisted report (already parsed JSON) against the current
/// tree. Returns one verdict per result row.
pub fn check(
    suite_path: &Path,
    report_json: &serde_json::Value,
) -> Result<Vec<RowVerdict>, String> {
    check_with_native(suite_path, report_json, None)
}

/// [`check`] with an injectable CURRENT native toolchain identity (R1
/// F07): production computes the real axes (`None`); tests can pin them so
/// worker/toolchain drift is deterministic without mutating the process
/// environment.
pub fn check_with_native(
    suite_path: &Path,
    report_json: &serde_json::Value,
    native_axes: Option<BTreeMap<String, String>>,
) -> Result<Vec<RowVerdict>, String> {
    check_with_families(suite_path, report_json, native_axes, None)
}

/// F-796-R6-04: ac and probe rows carry DIFFERENT worker identities.
/// One shared lazy map let the first family's identity answer the
/// other's axes; each family now resolves its own current axes.
pub fn check_with_families(
    suite_path: &Path,
    report_json: &serde_json::Value,
    ac_axes: Option<BTreeMap<String, String>>,
    probe_axes: Option<BTreeMap<String, String>>,
) -> Result<Vec<RowVerdict>, String> {
    let limits = Limits::defaults();
    // The ACTUAL worker/tool identity is computed lazily PER FAMILY:
    // only reports carrying native rows pay for the discovery.
    let mut current_ac: Option<BTreeMap<String, String>> = ac_axes;
    let mut current_probe_abi: Option<std::collections::BTreeMap<String, String>> = None;
    let mut current_probe_av: Option<BTreeMap<String, String>> = None;
    let mut current_probe: Option<BTreeMap<String, String>> = probe_axes;
    let current = current_identities(suite_path, &limits)?;
    let new_toolchain = toolchain_axes(&limits);
    let old_toolchain: BTreeMap<String, String> = report_json
        .get("toolchain")
        .and_then(|t| t.as_object())
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let results = report_json
        .get("results")
        .and_then(|r| r.as_array())
        .ok_or_else(|| "report has no results array".to_string())?;
    if results.is_empty() {
        return Err("report contains zero dynamic checks — nothing to trust".to_string());
    }
    let mut verdicts = Vec::new();
    for r in results {
        let case = r
            .get("case")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let check = r
            .get("check")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string();
        let key = format!("{}:{}", case, check);
        let get = |axis: &str| {
            r.get("identity")
                .and_then(|i| i.get(axis))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        };
        let cur = match current.get(&key) {
            Some(c) => c,
            None => {
                verdicts.push(RowVerdict::Stale {
                    case,
                    check,
                    drifted_axes: vec!["no longer declared in the current suite".to_string()],
                });
                continue;
            }
        };
        let cur = &cur.0;
        let applicability = &current.get(&key).unwrap().1;
        // R3 (F05): the support-lib content axis recomputes lazily — only
        // rows whose contract actually declares the library pay for it.
        if applicability.support_lib {
            if let Some((k, v)) = crate::test_cases::ac::support_lib_identity() {
                current_ac
                    .get_or_insert_with(crate::test_cases::ac::current_toolchain_identity)
                    .insert(k, v);
            }
        }
        let short = |h: &str| h.chars().take(8).collect::<String>();
        let mut drifted = Vec::new();
        for (axis, old_val, now) in [
            ("suite", get("suite"), cur.suite.clone()),
            ("case_file", get("case_file"), cur.case_file.clone()),
            ("source", get("source"), cur.source.clone()),
            ("oracle", get("oracle"), cur.oracle.clone()),
            ("contract", get("contract"), cur.contract.clone()),
        ] {
            if !old_val.is_empty() && old_val != now {
                drifted.push(format!("{} {}→{}", axis, short(&old_val), short(&now)));
            }
        }
        for (k, new_v) in &new_toolchain {
            if let Some(old_v) = old_toolchain.get(k) {
                if old_v != new_v {
                    drifted.push(format!("toolchain.{} {}→{}", k, short(old_v), short(new_v)));
                }
            }
        }
        // R1 (F07) + R3 (F05): native rows compare the worker/tool axes
        // bound at execution time AGAINST the row's own applicability set:
        // worker-exe + protocol on every native row (ac AND syntax); the
        // lld/sdk content axes only on ac build/run rows that actually
        // linked; the support-lib content axis when the contract declares
        // it. Missing, extra or contradicting axes are drift — never
        // silent consistency. Lazy discovery keeps non-native suites cheap.
        if matches!(
            r.get("adapter").and_then(|a| a.as_str()),
            Some("ac") | Some("syntax") | Some("probe")
        ) {
            let is_probe = r.get("adapter").and_then(|a| a.as_str()) == Some("probe");
            // PLAN-797 T-06: probe current identity is RECIPE-SPECIFIC —
            // an ABI row recomputes against the abi worker's axes.
            let av_probe_row = is_probe
                && r.get("native")
                    .and_then(|n| n.get("toolchain"))
                    .and_then(|t| t.as_array())
                    .map(|arr| {
                        arr.iter().any(|p| {
                            p.get(0).and_then(|v| v.as_str()) == Some("recipe")
                                && p.get(1).and_then(|v| v.as_str())
                                    == Some(crate::test_cases::ac_av::AV_RECIPE)
                        })
                    })
                    .unwrap_or(false);
            let abi_probe_row = is_probe
                && r.get("native")
                    .and_then(|n| n.get("toolchain"))
                    .and_then(|t| t.as_array())
                    .map(|arr| {
                        arr.iter().any(|p| {
                            p[0].as_str() == Some("recipe")
                                && p[1].as_str() == Some("abi/interop-v1")
                        })
                    })
                    .unwrap_or(false);
            // F-797-R4-11: the two probe recipes have DIFFERENT identity
            // maps; caching them in ONE slot made mixed suites
            // order-dependent (first recipe's map served the other's
            // rows). Separate lazies keep each family exact.
            let current = if abi_probe_row {
                current_probe_abi
                    .get_or_insert_with(crate::test_cases::abi::current_toolchain_identity)
            } else if av_probe_row {
                current_probe_av
                    .get_or_insert_with(crate::test_cases::ac_av::current_toolchain_identity)
            } else if is_probe {
                current_probe
                    .get_or_insert_with(crate::test_cases::sema::current_toolchain_identity)
            } else {
                current_ac.get_or_insert_with(crate::test_cases::ac::current_toolchain_identity)
            };
            let row_axes: BTreeMap<String, String> = r
                .get("native")
                .and_then(|n| n.get("toolchain"))
                .and_then(|t| t.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            Some((
                                p[0].as_str()?.to_string(),
                                p[1].as_str().unwrap_or("").to_string(),
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default();
            if row_axes.is_empty() {
                drifted.push("native.toolchain absent — cannot confirm the worker identity".into());
            } else {
                let linked = r["native"]["exit_code"].is_i64()
                    || r.get("artifacts")
                        .and_then(|a| a.as_array())
                        .map(|arr| {
                            arr.iter().any(|a| {
                                // R4 (F02): the report serializes ArtifactRef
                                // under `path` (not logical_path) — reading
                                // the wrong key made genuinely-linked BUILD
                                // rows look unlinked and their lld/sdk axes
                                // "not applicable", flagging unchanged
                                // reports STALE.
                                a.get("path")
                                    .and_then(|l| l.as_str())
                                    .map(|l| l == "app.obj" || l == "app.exe")
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false);
                // PLAN-797 T-06: probe rows carry a RECIPE-SPECIFIC axis
                // family — the ABI recipe binds to the abi worker's
                // identity axes, everything else to the sema worker's.
                let abi_row = row_axes.get("recipe").map(|r| r == "abi/interop-v1");
                let mut applicable: Vec<&str> = if is_probe {
                    match abi_row {
                        Some(true) => vec![
                            "abi-worker-exe-sha256",
                            "abi-worker-version",
                            "abi-protocol",
                            "recipe",
                        ],
                        Some(false) if av_probe_row => vec![
                            "av-worker-exe-sha256",
                            "av-worker-version",
                            "av-protocol",
                            "recipe",
                        ],
                        _ => {
                            // F-796-R4-04 + R6-04: probe rows bind to the sema
                            // worker's CONTENT identity, the mirrored version AND
                            // the protocol/recipe axes the report actually writes
                            vec![
                                "sema-worker-exe-sha256",
                                "sema-worker-version",
                                "sema-protocol",
                                "recipe",
                            ]
                        }
                    }
                } else {
                    vec!["worker-exe-sha256", "ac-worker-protocol"]
                };
                if applicability.links_conditionally && linked {
                    applicable.extend(["lld-sha256", "sdk-um-sha256"]);
                }
                if applicability.support_lib {
                    applicable.push("support-lib-trace-marks-sha256");
                }
                for k in &applicable {
                    match current.get(*k) {
                        None => drifted.push(format!(
                            "native.{} not recomputable now — cannot confirm the row",
                            k
                        )),
                        Some(now) => match row_axes.get(*k) {
                            Some(old_v) if old_v == now => {}
                            Some(old_v) => {
                                drifted.push(format!(
                                    "native.{} {}→{}",
                                    k,
                                    short(old_v),
                                    short(now)
                                ));
                            }
                            None => {
                                drifted.push(format!("native.{} missing (now {})", k, short(now)));
                            }
                        },
                    }
                }
                // Extra axes the row cannot justify are protocol drift —
                // a check row carrying linker axes, or a foreign axis name,
                // never passes as consistent.
                for k in row_axes.keys() {
                    if !applicable.contains(&k.as_str()) {
                        drifted.push(format!(
                            "native.{} present but not applicable to this row's consumption path",
                            k
                        ));
                    }
                }
            }
        }
        verdicts.push(if drifted.is_empty() {
            RowVerdict::Fresh { case, check }
        } else {
            RowVerdict::Stale {
                case,
                check,
                drifted_axes: drifted,
            }
        });
    }
    Ok(verdicts)
}
