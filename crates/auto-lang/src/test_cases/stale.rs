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

/// One row of the staleness summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowVerdict {
    Fresh { case: String, check: String },
    Stale { case: String, check: String, drifted_axes: Vec<String> },
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
    m.insert("auto-lang".to_string(), env!("CARGO_PKG_VERSION").to_string());
    m.insert("limits".to_string(), fingerprint(&limits));
    m
}

/// Recompute the current identity of every declared check through the real
/// pipeline. Checks whose references no longer resolve get an identity with
/// an empty oracle hash AND an extra marker axis so they are unambiguously
/// stale rather than silently equal.
pub fn current_identities(
    suite_path: &Path,
    limits: &Limits,
) -> Result<BTreeMap<String, IdentityHashes>, String> {
    let loaded = load_suite(suite_path, limits).map_err(|e| format!("suite load: {}", e))?;
    let mut out = BTreeMap::new();
    for file in &loaded.cases {
        for spec in &file.specs {
            let resolved = resolve_case(spec.clone(), &file.doc, &loaded.fixture_base, limits).ok();
            for check in &spec.checks {
                let oracle = resolve_oracle(&spec.id, check, &loaded.fixture_base, limits).ok();
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
                    contract: fingerprint(&spec),
                };
                out.insert(format!("{}:{}", spec.id, check.id), identity);
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
    let limits = Limits::defaults();
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
        let case = r.get("case").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        let check = r.get("check").and_then(|v| v.as_str()).unwrap_or("?").to_string();
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
