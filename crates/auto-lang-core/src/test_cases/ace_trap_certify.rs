//! PLAN-997: the EXPLICIT independent a2r TRAP certification sub-entry
//! (`ace-certify --trap`). Domain-isolated from the value certifier
//! (`ace.m1.cert/1`): this module certifies corpus rows whose outcome is
//! `trap` under the EXPLICIT versioned source-int profile
//! `legacy-a2r.checked-i32/1`.
//!
//! Domain contract (frozen with the schema id `ace.m1.a2r-trap/1`):
//! 1. Emit the ACTUAL Rust fresh from the corpus source through the
//!    production transpiler WITH the checked-i32 profile selected
//!    explicitly (never the default carrier for these cells).
//! 2. Require actual bytes == frozen trap-golden bytes
//!    (`oracle/trap/m1_<id>.rs`) — drift FAILs, goldens move only through
//!    explicit review (G04).
//! 3. The trap oracle is the declarative registry
//!    `oracle/trap/trap-oracle.json` (`ace.m1.a2r-trap-oracle/1`): per
//!    cell a typed {kind, op, line} plus the shared exit code 70. The
//!    registry `kind` MUST equal the binding row's `expected` column —
//!    the original outcome/expectation stays authoritative.
//! 4. Compile AND run (a) the actual bytes under the frozen host wrapper
//!    and (b) the hand-written independent reference
//!    (`reference/trap/<ID>.rs`, std `checked_*` on an i32 carrier — no
//!    shared code with the emission). BOTH sides must observe the SAME
//!    typed event: exit 70, stderr exactly the oracle line, stdout EMPTY.
//!    A Rust panic (exit 101/abort), a value result, a compiler error,
//!    or a stream/wall overrun never satisfies the oracle.
//! 5. Persist the certificate at `oracle/trap-certs/m1_<id>.trap-cert.json`
//!    (a path the value reader never enumerates) binding every identity
//!    axis; REUSE only after STRICT validation with the same Invalid vs
//!    Stale discipline as the value domain (`ace_certify`).
//!
//! Contained-tool governance mirrors the value certifier exactly: rustc
//! and the certified binaries start suspended in kill-on-close Job
//! Objects under the same 120s compile / 30s run / 1MB stream caps, and
//! the scratch tree is removed on every exit path.

use super::ace_certify::{
    capture_toolchain_contained, compile, run_capped, BUILD_TOOLCHAIN_IDENTITY, COMPILE_WALL,
    DEPS_CLOSURE, RUN_WALL, RUSTC_FLAG_TOKENS, WRAPPER_MAIN_RS,
};
use crate::trans::rust::{transpile_rust_with_source_dir_profile, IntProfile};
use auto_test_case::sha256::{hex, sha256};
use std::path::{Path, PathBuf};

pub const TRAP_CERT_SCHEMA: &str = "ace.m1.a2r-trap/1";
pub const TRAP_ORACLE_SCHEMA: &str = "ace.m1.a2r-trap-oracle/1";
pub const TRAP_PROFILE_ID: &str = "legacy-a2r.checked-i32/1";
/// The frozen trap terminal state (mirrors the mn engine's frozen trap
/// exit code recorded in the B14 binding note).
pub const TRAP_EXIT_CODE: i64 = 70;
/// Emit needs more than a default thread stack (mirrors the a2r adapter).
const EMIT_STACK_BYTES: usize = 128 * 1024 * 1024;

pub struct TrapCertifyArgs {
    pub corpus: PathBuf,
    pub certs_dir: PathBuf,
    pub only: Vec<String>,
    pub force: bool,
    pub rustc: Option<String>,
    pub scratch: Option<PathBuf>,
}

pub fn trap_certify_and_report(args: &TrapCertifyArgs) -> i32 {
    match trap_certify(args) {
        Ok(summaries) => {
            let mut certified = 0;
            let mut reused = 0;
            let mut failed = 0;
            for s in &summaries {
                match s.outcome.as_str() {
                    "certified" => certified += 1,
                    "reused" => reused += 1,
                    _ => failed += 1,
                }
                println!("{}: {} — {}", s.outcome, s.case, s.detail);
            }
            println!(
                "trap certification: {} certified, {} reused, {} failed",
                certified, reused, failed
            );
            if failed == 0 {
                0
            } else {
                1
            }
        }
        Err(e) => {
            eprintln!("ace-certify --trap: {e}");
            1
        }
    }
}

/// One typed trap-oracle cell from the declarative registry.
struct TrapOracleCell {
    kind: String,
    op: String,
    line: String,
}

/// Load `oracle/trap/trap-oracle.json` strictly: exact keys, declared
/// types, and per-cell {kind, op, line} consistency (line is exactly
/// `auto.trap:<kind>:<op>`).
fn load_trap_oracle(
    corpus: &Path,
) -> Result<std::collections::BTreeMap<String, TrapOracleCell>, String> {
    let path = corpus.join("oracle").join("trap").join("trap-oracle.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("read trap oracle: {e}"))?;
    let v: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("trap oracle is not valid JSON: {e}"))?;
    let root = v
        .as_object()
        .ok_or_else(|| "trap oracle root is not an object".to_string())?;
    let exact = |keys: &[&str]| -> Result<(), String> {
        for k in root.keys() {
            if !keys.contains(&k.as_str()) {
                return Err(format!("unknown trap-oracle field {k:?} (strict schema)"));
            }
        }
        for k in keys {
            if !root.contains_key(*k) {
                return Err(format!("missing trap-oracle field {k:?}"));
            }
        }
        Ok(())
    };
    exact(&["schema", "profile", "exit_code", "stderr_prefix", "cells"])?;
    if root.get("schema").and_then(|s| s.as_str()) != Some(TRAP_ORACLE_SCHEMA) {
        return Err(format!("trap oracle schema is not {TRAP_ORACLE_SCHEMA}"));
    }
    if root.get("profile").and_then(|s| s.as_str()) != Some(TRAP_PROFILE_ID) {
        return Err(format!("trap oracle profile is not {TRAP_PROFILE_ID}"));
    }
    if root.get("exit_code").and_then(|c| c.as_i64()) != Some(TRAP_EXIT_CODE) {
        return Err(format!("trap oracle exit_code is not {TRAP_EXIT_CODE}"));
    }
    let prefix = root
        .get("stderr_prefix")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "trap oracle stderr_prefix is not a string".to_string())?;
    if prefix != "auto.trap:" {
        return Err("trap oracle stderr_prefix is not the frozen auto.trap:".to_string());
    }
    let cells = root
        .get("cells")
        .and_then(|c| c.as_object())
        .ok_or_else(|| "trap oracle cells is not an object".to_string())?;
    let mut out = std::collections::BTreeMap::new();
    for (id, cell) in cells {
        let c = cell
            .as_object()
            .ok_or_else(|| format!("trap oracle cell {id} is not an object"))?;
        if c.len() != 3
            || !c.contains_key("kind")
            || !c.contains_key("op")
            || !c.contains_key("line")
        {
            return Err(format!(
                "trap oracle cell {id} must have exactly {{kind, op, line}}"
            ));
        }
        let kind = c.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        let op = c.get("op").and_then(|v| v.as_str()).unwrap_or("");
        let line = c.get("line").and_then(|v| v.as_str()).unwrap_or("");
        if kind.is_empty() || op.is_empty() {
            return Err(format!("trap oracle cell {id} has empty kind/op"));
        }
        let want_line = format!("{prefix}{kind}:{op}");
        if line != want_line {
            return Err(format!(
                "trap oracle cell {id} line {line:?} != {want_line:?} (prefix+kind+op)"
            ));
        }
        out.insert(
            id.clone(),
            TrapOracleCell {
                kind: kind.to_string(),
                op: op.to_string(),
                line: line.to_string(),
            },
        );
    }
    if out.is_empty() {
        return Err("trap oracle has no cells".to_string());
    }
    Ok(out)
}

/// Parse binding.tsv for the trap-certifiable set: corpus rows whose
/// outcome is exactly `trap`. Rows already migrated to the trap-domain
/// token (`ok-emit-trap`) are always eligible; rows still carrying the
/// pre-997 gap token (`cert-diff`) certify ONLY when `--only` names them
/// (evidence first, token flip afterwards — never a bulk green sweep).
fn trap_certifiable_rows(binding: &Path, only: &[String]) -> Result<Vec<(String, String)>, String> {
    let text = std::fs::read_to_string(binding).map_err(|e| format!("read binding: {e}"))?;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 12 {
            return Err(format!(
                "binding row has {} columns (frozen schema = 12)",
                f.len()
            ));
        }
        if f[1] != "corpus" || f[3] != "trap" {
            continue;
        }
        let named = only.iter().any(|o| o.eq_ignore_ascii_case(f[0]));
        if f[8] == "ok-emit-trap" || (f[8] == "cert-diff" && named) {
            rows.push((f[0].to_string(), f[4].to_string()));
        }
    }
    if rows.is_empty() {
        return Err(
            "no trap-certifiable rows (outcome=trap with ok-emit-trap, or cert-diff named by --only)"
                .to_string(),
        );
    }
    Ok(rows)
}

pub fn trap_certify(
    args: &TrapCertifyArgs,
) -> Result<Vec<super::ace_certify::CaseSummary>, String> {
    let oracle = load_trap_oracle(&args.corpus)?;
    let rows = trap_certifiable_rows(&args.corpus.join("binding.tsv"), &args.only)?;
    let selected: Vec<&(String, String)> = rows
        .iter()
        .filter(|(id, _)| {
            args.only.is_empty() || args.only.iter().any(|o| o.eq_ignore_ascii_case(id))
        })
        .collect();
    if selected.is_empty() {
        return Err("--only matched no trap-certifiable case".to_string());
    }
    for (id, expected) in &selected {
        let cell = oracle
            .get(id)
            .ok_or_else(|| format!("trap oracle has no cell for {id}"))?;
        if cell.kind != *expected {
            return Err(format!(
                "trap oracle kind {:?} != binding expected {:?} for {id} — the binding stays authoritative",
                cell.kind, expected
            ));
        }
    }
    std::fs::create_dir_all(&args.certs_dir).map_err(|e| format!("certs dir: {e}"))?;

    let scratch_root = match &args.scratch {
        Some(p) => p.clone(),
        None => std::env::temp_dir().join(format!("ace-m1-trap-certify-{}", std::process::id())),
    };
    let _ = std::fs::remove_dir_all(&scratch_root);
    std::fs::create_dir_all(&scratch_root).map_err(|e| format!("scratch: {e}"))?;
    let result = (|| {
        let rustc = args
            .rustc
            .clone()
            .unwrap_or_else(|| std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
        let identity_source = if args.rustc.is_some() {
            "--rustc override"
        } else if std::env::var("RUSTC").is_ok() {
            "env RUSTC"
        } else {
            "PATH"
        };
        let toolchain_identity = capture_toolchain_contained(&rustc)?;
        let mut summaries = Vec::new();
        for (id, expected) in selected {
            let cell = &oracle[id];
            let summary = trap_certify_one(
                id,
                expected,
                cell,
                args,
                &scratch_root,
                &rustc,
                &toolchain_identity,
                identity_source,
            );
            summaries.push(summary);
        }
        Ok(summaries)
    })();
    let _ = std::fs::remove_dir_all(&scratch_root);
    result
}

/// Current file-derived identity axes for one trap cell.
struct TrapAxes {
    source_sha: String,
    golden_sha: String,
    wrapper_sha: String,
    reference_sha: String,
    oracle_sha: String,
}

fn trap_axes(corpus: &Path, id: &str) -> Result<TrapAxes, String> {
    let module = format!("m1_{}", id.to_lowercase());
    let read = |p: PathBuf, what: &str| -> Result<Vec<u8>, String> {
        std::fs::read(&p).map_err(|e| format!("read {what}: {e}"))
    };
    let source = read(corpus.join("src").join(format!("{id}.at")), "trap source")?;
    let golden = read(
        corpus
            .join("oracle")
            .join("trap")
            .join(format!("{module}.rs")),
        "trap golden",
    )?;
    let reference = read(
        corpus
            .join("reference")
            .join("trap")
            .join(format!("{id}.rs")),
        "trap reference",
    )?;
    let oracle = read(
        corpus.join("oracle").join("trap").join("trap-oracle.json"),
        "trap oracle",
    )?;
    Ok(TrapAxes {
        source_sha: hex(&sha256(&source)),
        golden_sha: hex(&sha256(&golden)),
        wrapper_sha: hex(&sha256(WRAPPER_MAIN_RS.as_bytes())),
        reference_sha: hex(&sha256(&reference)),
        oracle_sha: hex(&sha256(&oracle)),
    })
}

/// Strict exact-key sets for `ace.m1.a2r-trap/1` at every level.
const TRAP_ROOT_KEYS: &[&str] = &[
    "schema",
    "case",
    "module",
    "profile",
    "wrapper_file",
    "expected_trap",
    "identity",
    "toolchain",
    "deps",
    "observed",
    "verdict",
    "produced_by",
    "created_at",
];
const TRAP_EXPECTED_KEYS: &[&str] = &["kind", "op", "line", "exit"];
const TRAP_IDENTITY_KEYS: &[&str] = &[
    "source_sha256",
    "golden_sha256",
    "actual_sha256",
    "wrapper_sha256",
    "reference_sha256",
    "trap_oracle_sha256",
];
const TRAP_RUN_KEYS: &[&str] = &["exit", "stdout", "stderr"];

/// STRICT trap receipt validation (mirrors `ace_certify::validate_receipt`).
/// `Ok(())` = valid AND current; `Invalid` = structural break (never reused,
/// never overwritten); `Stale` = honest re-certification path.
///
/// F-997-R1-02: the declarative registry cell (`cell`) is authoritative —
/// the receipt's `expected_trap` must equal the cell's {kind, op, line}, so a
/// self-consistently forged typed event can never be reused (the PS reader
/// enforces the same cross-check on the daily face).
fn validate_trap_receipt(
    cert: &serde_json::Value,
    case_id: &str,
    cell: &TrapOracleCell,
    axes: &TrapAxes,
    toolchain_identity: &str,
) -> Result<(), super::ace_certify::ReceiptProblem> {
    use super::ace_certify::ReceiptProblem;
    let invalid = |what: String| ReceiptProblem::Invalid(what);
    let stale = |what: String| ReceiptProblem::Stale(what);
    let exact_keys = |obj: &serde_json::Map<String, serde_json::Value>,
                      expected: &[&str],
                      where_: &str|
     -> Result<(), ReceiptProblem> {
        for k in obj.keys() {
            if !expected.contains(&k.as_str()) {
                return Err(invalid(format!(
                    "unknown field {k:?} in {where_} (strict schema: exact keys only)"
                )));
            }
        }
        for e in expected {
            if !obj.contains_key(*e) {
                return Err(invalid(format!("missing field {e:?} in {where_}")));
            }
        }
        Ok(())
    };
    let is_hash64 = |v: &serde_json::Value| matches!(v.as_str(), Some(s) if s.len() == 64 && s.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)));

    let root = cert
        .as_object()
        .ok_or_else(|| invalid("trap receipt root is not an object".to_string()))?;
    exact_keys(root, TRAP_ROOT_KEYS, "trap receipt root")?;
    if cert.get("schema").and_then(|v| v.as_str()) != Some(TRAP_CERT_SCHEMA) {
        return Err(invalid(format!("schema is not {TRAP_CERT_SCHEMA}")));
    }
    if cert.get("case").and_then(|v| v.as_str()) != Some(case_id) {
        return Err(invalid(format!("case is not {case_id}")));
    }
    let module = format!("m1_{}", case_id.to_lowercase());
    if cert.get("module").and_then(|v| v.as_str()) != Some(&module) {
        return Err(invalid(format!("module is not {module}")));
    }
    if cert.get("profile").and_then(|v| v.as_str()) != Some(TRAP_PROFILE_ID) {
        return Err(invalid(format!("profile is not {TRAP_PROFILE_ID}")));
    }
    if cert.get("wrapper_file").and_then(|v| v.as_str()) != Some("oracle/wrapper-main-v1.rs") {
        return Err(invalid(
            "wrapper_file is not the frozen oracle/wrapper-main-v1.rs".to_string(),
        ));
    }
    if cert.get("verdict").and_then(|v| v.as_str()) != Some("certified") {
        return Err(invalid("verdict is not certified".to_string()));
    }
    if cert.get("created_at").and_then(|v| v.as_u64()).is_none() {
        return Err(invalid("created_at is not an unsigned integer".to_string()));
    }
    // expected_trap: the typed event from the declarative registry.
    let expected = cert
        .get("expected_trap")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("expected_trap is not an object".to_string()))?;
    exact_keys(expected, TRAP_EXPECTED_KEYS, "expected_trap")?;
    let kind = expected.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let op = expected.get("op").and_then(|v| v.as_str()).unwrap_or("");
    let line = expected.get("line").and_then(|v| v.as_str()).unwrap_or("");
    if kind.is_empty() || op.is_empty() || line != format!("auto.trap:{kind}:{op}") {
        return Err(invalid(
            "expected_trap kind/op/line are inconsistent with the auto.trap:<kind>:<op> contract"
                .to_string(),
        ));
    }
    if expected.get("exit").and_then(|v| v.as_i64()) != Some(TRAP_EXIT_CODE) {
        return Err(invalid(format!(
            "expected_trap.exit is not the frozen {TRAP_EXIT_CODE}"
        )));
    }
    // F-997-R1-02: the registry cell is authoritative — a self-consistent
    // but non-registry typed event is a structural forgery, never a reuse.
    if kind != cell.kind || op != cell.op || line != cell.line {
        return Err(invalid(format!(
            "expected_trap does not equal the trap-oracle registry cell for {case_id} (receipt {kind}:{op}, registry {}:{})",
            cell.kind, cell.op
        )));
    }
    // observed: BOTH sides must show the identical typed event with clean
    // stdout — a value result, a panic exit, or empty evidence never passes.
    let observed = cert
        .get("observed")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("observed is not an object".to_string()))?;
    exact_keys(observed, &["actual_run", "reference_run"], "observed")?;
    for side in ["actual_run", "reference_run"] {
        let run = observed
            .get(side)
            .and_then(|v| v.as_object())
            .ok_or_else(|| invalid(format!("observed.{side} is not an object")))?;
        exact_keys(run, TRAP_RUN_KEYS, &format!("observed.{side}"))?;
        if run.get("exit").and_then(|v| v.as_i64()) != Some(TRAP_EXIT_CODE) {
            return Err(invalid(format!(
                "observed.{side}.exit is not the integer {TRAP_EXIT_CODE}"
            )));
        }
        if run.get("stdout").and_then(|v| v.as_str()) != Some("") {
            return Err(invalid(format!(
                "observed.{side}.stdout is not the empty string (clean stdout is part of the trap contract)"
            )));
        }
        // The raw observation is the newline-terminated event line exactly
        // as eprintln! emitted it on stderr.
        if run.get("stderr").and_then(|v| v.as_str()) != Some(&format!("{line}\n")) {
            return Err(invalid(format!(
                "observed.{side}.stderr does not equal the oracle line {line:?} (newline-terminated)"
            )));
        }
    }
    // toolchain: same frozen shape as the value domain.
    let toolchain = cert
        .get("toolchain")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("toolchain is not an object".to_string()))?;
    exact_keys(
        toolchain,
        &[
            "identity",
            "identity_source",
            "default_identity_at_certify",
            "flags",
        ],
        "toolchain",
    )?;
    let flags = toolchain
        .get("flags")
        .and_then(|v| v.as_array())
        .ok_or_else(|| invalid("toolchain.flags is not an array".to_string()))?;
    if flags.len() != RUSTC_FLAG_TOKENS.len()
        || flags
            .iter()
            .zip(RUSTC_FLAG_TOKENS.iter())
            .any(|(v, want)| v.as_str() != Some(*want))
    {
        return Err(invalid(format!(
            "toolchain.flags is not the frozen set {:?}",
            RUSTC_FLAG_TOKENS
        )));
    }
    for k in ["identity", "identity_source", "default_identity_at_certify"] {
        let v = toolchain.get(k).and_then(|v| v.as_str()).unwrap_or("");
        if v.trim().is_empty() {
            return Err(invalid(format!("toolchain.{k} is not a non-empty string")));
        }
    }
    if !matches!(
        toolchain.get("identity_source").and_then(|v| v.as_str()),
        Some("--rustc override") | Some("env RUSTC") | Some("PATH")
    ) {
        return Err(invalid(
            "toolchain.identity_source is not one of the frozen selection rules".to_string(),
        ));
    }
    // deps: std-only, empty external (the trap helpers are self-contained).
    let deps_obj = cert
        .get("deps")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("deps is not an object".to_string()))?;
    exact_keys(deps_obj, &["external", "closure"], "deps")?;
    let external = deps_obj
        .get("external")
        .and_then(|v| v.as_array())
        .ok_or_else(|| invalid("deps.external is not an array".to_string()))?;
    if !external.is_empty() {
        return Err(invalid("deps.external is not empty".to_string()));
    }
    if deps_obj.get("closure").and_then(|v| v.as_str()) != Some(DEPS_CLOSURE) {
        return Err(invalid(format!("deps.closure is not {DEPS_CLOSURE:?}")));
    }
    // identity: every axis a 64-char lowercase hex; actual == golden.
    let id = cert
        .get("identity")
        .and_then(|v| v.as_object())
        .ok_or_else(|| invalid("identity is not an object".to_string()))?;
    exact_keys(id, TRAP_IDENTITY_KEYS, "identity")?;
    for k in TRAP_IDENTITY_KEYS {
        if !is_hash64(id.get(*k).unwrap_or(&serde_json::Value::Null)) {
            return Err(invalid(format!(
                "identity.{k} is not a 64-char lowercase hex string"
            )));
        }
    }
    if id.get("actual_sha256").and_then(|v| v.as_str())
        != id.get("golden_sha256").and_then(|v| v.as_str())
    {
        return Err(invalid(
            "identity.actual_sha256 != golden_sha256 (self-inconsistent)".to_string(),
        ));
    }
    // File-derived axes: drift is STALE.
    for (key, current, label) in [
        ("source_sha256", &axes.source_sha, "trap source"),
        ("golden_sha256", &axes.golden_sha, "trap golden"),
        ("wrapper_sha256", &axes.wrapper_sha, "wrapper"),
        ("reference_sha256", &axes.reference_sha, "trap reference"),
        ("trap_oracle_sha256", &axes.oracle_sha, "trap oracle"),
    ] {
        let recorded = id.get(key).and_then(|v| v.as_str()).unwrap_or("");
        if recorded != *current {
            return Err(stale(format!("{label} drifted (receipt axis {key})")));
        }
    }
    if toolchain
        .get("identity")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        != toolchain_identity
    {
        return Err(stale("toolchain identity drifted".to_string()));
    }
    if toolchain
        .get("default_identity_at_certify")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        != BUILD_TOOLCHAIN_IDENTITY.trim()
    {
        return Err(stale(
            "default toolchain drifted since certification (re-certify to re-bind)".to_string(),
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn trap_certify_one(
    id: &str,
    expected_kind: &str,
    cell: &TrapOracleCell,
    args: &TrapCertifyArgs,
    scratch_root: &Path,
    rustc: &str,
    toolchain_identity: &str,
    identity_source: &str,
) -> super::ace_certify::CaseSummary {
    let module = format!("m1_{}", id.to_lowercase());
    let case_scratch = scratch_root.join(&module);
    let _ = std::fs::create_dir_all(&case_scratch);

    let fail = |detail: String| super::ace_certify::CaseSummary {
        case: id.to_string(),
        outcome: "fail".to_string(),
        detail,
    };

    // ---- 1. fresh actual through the production transpiler, checked profile
    let source_path = args.corpus.join("src").join(format!("{id}.at"));
    let source = match std::fs::read_to_string(&source_path) {
        Ok(s) => s,
        Err(e) => return fail(format!("read source: {e}")),
    };
    if IntProfile::from_id(TRAP_PROFILE_ID) != Some(IntProfile::CheckedI32) {
        return fail("profile id no longer admits — frozen mismatch".to_string());
    }
    let emit = std::thread::Builder::new()
        .stack_size(EMIT_STACK_BYTES)
        .spawn({
            let source_dir = args.corpus.join("src");
            let module = module.clone();
            let source = source.clone();
            move || {
                let mut sink = transpile_rust_with_source_dir_profile(
                    &source_dir,
                    &module,
                    &source,
                    IntProfile::CheckedI32,
                )?;
                let bytes = sink.done()?.clone();
                Ok::<_, crate::error::AutoError>(bytes)
            }
        })
        .and_then(|h| Ok(h.join()));
    let actual: Vec<u8> = match emit {
        Ok(Ok(Ok(bytes))) => bytes,
        Ok(Ok(Err(e))) => return fail(format!("a2r emit (checked-i32) failed: {e}")),
        Ok(Err(payload)) => {
            return fail(format!(
                "emit thread panicked: {}",
                super::avm::panic_message(&payload)
            ))
        }
        Err(e) => return fail(format!("spawn emit thread: {e}")),
    };

    // ---- 2. actual == frozen trap golden (drift never re-certifies)
    let golden_path = args
        .corpus
        .join("oracle")
        .join("trap")
        .join(format!("{module}.rs"));
    let golden = match std::fs::read(&golden_path) {
        Ok(g) => g,
        Err(e) => return fail(format!("read trap golden: {e}")),
    };
    if actual != golden {
        return fail(format!(
            "actual != trap golden for {id} ({} vs {} bytes) — explicit review required, no auto-update",
            actual.len(),
            golden.len()
        ));
    }

    // ---- 3. reuse: STRICT validation, Invalid never reused/overwritten
    let axes = match trap_axes(&args.corpus, id) {
        Ok(a) => a,
        Err(e) => return fail(e),
    };
    let cert_path = args.certs_dir.join(format!("{module}.trap-cert.json"));
    if let Ok(old_text) = std::fs::read_to_string(&cert_path) {
        match serde_json::from_str::<serde_json::Value>(&old_text) {
            Ok(old) => match validate_trap_receipt(&old, id, cell, &axes, toolchain_identity) {
                Ok(()) => {
                    if !args.force {
                        return super::ace_certify::CaseSummary {
                            case: id.to_string(),
                            outcome: "reused".to_string(),
                            detail: "strict trap receipt validation passed — axes unchanged, nothing compiled".to_string(),
                        };
                    }
                }
                Err(super::ace_certify::ReceiptProblem::Invalid(why)) => {
                    return fail(format!(
                        "REJECTED trap receipt {module}.trap-cert.json: {why} — structurally invalid; manually delete or repair it, then certify again"
                    ));
                }
                Err(super::ace_certify::ReceiptProblem::Stale(_why)) => {
                    // valid, drifted context: re-certify below
                }
            },
            Err(e) => {
                return fail(format!(
                    "REJECTED trap receipt {module}.trap-cert.json: not valid JSON ({e}) — manually delete or repair it, then certify again"
                ));
            }
        }
    }

    // ---- 4. compile & run the actual under the frozen wrapper; the typed
    // event must be observed EXACTLY: exit 70, stderr == oracle line,
    // stdout empty. A panic exit (101/abnormal), a value result, or a
    // compile failure never satisfies the oracle.
    let wrapper_dir = case_scratch.join("wrap");
    let _ = std::fs::create_dir_all(wrapper_dir.join("src"));
    if let Err(e) = std::fs::write(wrapper_dir.join("src").join("generated.rs"), &golden) {
        return fail(format!("write generated.rs: {e}"));
    }
    if let Err(e) = std::fs::write(wrapper_dir.join("src").join("main.rs"), WRAPPER_MAIN_RS) {
        return fail(format!("write wrapper main.rs: {e}"));
    }
    let wrap_exe = wrapper_dir.join(format!("{module}_wrap.exe"));
    let crate_name = module.trim_start_matches("m1_").to_string();
    if let Err(e) = compile(
        rustc,
        &wrapper_dir.join("src").join("main.rs"),
        &wrap_exe,
        &crate_name,
    ) {
        return fail(format!("compile actual-wrapper: {e}"));
    }
    let actual_run = match run_capped(&wrap_exe) {
        Ok(r) => r,
        Err(e) => return fail(format!("run actual-wrapper: {e}")),
    };
    let oracle_line = format!("{}\n", cell.line);
    let obs_err = |side: &str, run: &super::ace_certify::RunOutcome| -> String {
        format!(
            "{side} mismatched the trap oracle: exit={} stdout={} bytes stderr={:?} (expected exit={TRAP_EXIT_CODE}, empty stdout, stderr {oracle_line:?})",
            run.exit,
            run.stdout.len(),
            String::from_utf8_lossy(&run.stderr)
        )
    };
    if actual_run.exit as i64 != TRAP_EXIT_CODE
        || !actual_run.stdout.is_empty()
        || actual_run.stderr != oracle_line.as_bytes()
    {
        return fail(obs_err("actual-wrapper run", &actual_run));
    }

    // ---- 5. compile & run the INDEPENDENT reference (standalone, i32
    // checked_* carrier): the same typed event must be observed.
    let ref_exe = case_scratch.join(format!("{module}_ref.exe"));
    if let Err(e) = compile(
        rustc,
        &args
            .corpus
            .join("reference")
            .join("trap")
            .join(format!("{id}.rs")),
        &ref_exe,
        &crate_name,
    ) {
        return fail(format!("compile trap reference: {e}"));
    }
    let reference_run = match run_capped(&ref_exe) {
        Ok(r) => r,
        Err(e) => return fail(format!("run trap reference: {e}")),
    };
    if reference_run.exit as i64 != TRAP_EXIT_CODE
        || !reference_run.stdout.is_empty()
        || reference_run.stderr != oracle_line.as_bytes()
    {
        return fail(obs_err("trap reference run", &reference_run));
    }

    // ---- 6. certificate (strict schema `ace.m1.a2r-trap/1`)
    let cert = serde_json::json!({
        "schema": TRAP_CERT_SCHEMA,
        "case": id,
        "module": module,
        "profile": TRAP_PROFILE_ID,
        "wrapper_file": "oracle/wrapper-main-v1.rs",
        "expected_trap": {
            "kind": expected_kind,
            "op": cell.op,
            "line": cell.line,
            "exit": TRAP_EXIT_CODE,
        },
        "identity": {
            "source_sha256": axes.source_sha,
            "golden_sha256": axes.golden_sha,
            "actual_sha256": axes.golden_sha,
            "wrapper_sha256": axes.wrapper_sha,
            "reference_sha256": axes.reference_sha,
            "trap_oracle_sha256": axes.oracle_sha,
        },
        "toolchain": {
            "identity": toolchain_identity,
            "identity_source": identity_source,
            "default_identity_at_certify": BUILD_TOOLCHAIN_IDENTITY.trim(),
            "flags": RUSTC_FLAG_TOKENS,
        },
        "deps": { "external": [], "closure": DEPS_CLOSURE },
        "observed": {
            "actual_run": {
                "exit": TRAP_EXIT_CODE,
                "stdout": "",
                "stderr": oracle_line,
            },
            "reference_run": {
                "exit": TRAP_EXIT_CODE,
                "stdout": "",
                "stderr": oracle_line,
            },
        },
        "verdict": "certified",
        "produced_by": { "tool": "ace-certify --trap", "schema_note": "trap domain is independent of ace.m1.cert/1; receipts never mix domains" },
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    });
    let written = serde_json::to_string_pretty(&cert)
        .map_err(|e| e.to_string())
        .and_then(|p| std::fs::write(&cert_path, p).map_err(|e| e.to_string()));
    match written {
        Ok(()) => super::ace_certify::CaseSummary {
            case: id.to_string(),
            outcome: "certified".to_string(),
            detail: format!(
                "actual==trap golden; wrapper+reference both observed {} (exit {TRAP_EXIT_CODE}, clean stdout); cert {module}.trap-cert.json",
                cell.line
            ),
        },
        Err(e) => fail(format!("write trap certificate: {e}")),
    }
}

/// Wall constants re-exported for readers that report budgets.
pub fn trap_budgets() -> (std::time::Duration, std::time::Duration, usize) {
    (COMPILE_WALL, RUN_WALL, 1 << 20)
}
