//! PLAN-917 T-04: the EXPLICIT certification entry for the legacy-A2R
//! mut-record (R07) and text-slice (T08) evidence, plus their legal
//! neighbors — domain `ace.m1.a2r-emit917.cert/1`.
//!
//! This module is the ONLY place that compiles and runs Rust for the
//! a2r-emit-917 case set, and it is never invoked by the suite runner or any
//! daily scope (double axis per plan §5.2/§5.3: daily = emit+bytes vs golden,
//! zero Rust spawns; certification = this entry, explicitly invoked).
//!
//! Contract:
//! 1. Emit the ACTUAL Rust fresh from the case source through the production
//!    transpiler entries (`transpile_rust_with_source_dir` single /
//!    `transpile_rust_project` project) — never read the golden as actual.
//! 2. Require actual bytes == frozen golden bytes for every declared
//!    artifact; drift is a FAIL that only produces a proposal (never an
//!    automatic golden update).
//! 3. Compile AND run, under CREATE_SUSPENDED + kill-on-close Job containment
//!    (120s compile wall / 30s run wall / 1 MiB per-stream caps, streams
//!    drained by independent reader threads — reused from the 852 harness):
//!    (a) each runnable actual artifact under the frozen host wrapper, and
//!    (b) the hand-written `reference/<id>.rs`; both must satisfy the frozen
//!    oracle — a value oracle (`i32:<n>\n` on stdout, exit 0) or a trap
//!    oracle (non-zero exit with the frozen marker on stderr). The trap
//!    oracle is a BEHAVIOR OBSERVATION of the frozen slice-oob semantics; it
//!    is NOT an Auto trap certification (that channel is a separate gap).
//!    Behavior certification runs the single-mode artifact (a complete
//!    actual, `fn main` present); project-mode artifacts carry the crate-root
//!    `#![allow]` pragma, which cannot be include!-wrapped inside a `mod`, so
//!    they are certified on the emit/bytes axis only (same Rust program
//!    modulo the pragma header and the std-only Cargo.toml).
//! 4. Persist a version-controlled certificate binding every identity axis
//!    (source / each golden / wrapper / reference SHA-256, toolchain identity
//!    captured CONTAINED from the exact launched compiler, frozen flag
//!    tokens, deps closure). Receipt reuse is STRICT: a structurally invalid
//!    receipt is a hard error (never reused, never silently overwritten);
//!    file-content drift marks it STALE and re-certifies honestly.
//!
//! The domain is strictly separated from the 852 scalar domain: receipts
//! live under `a2r-emit-917/certs/`, carry schema `ace.m1.a2r-emit917.cert/1`,
//! and the 852 reader (which scans `oracle/certs/` for `ace.m1.cert/1`)
//! never sees them.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::ace_certify::{
    capture_toolchain_contained, compile, run_capped, RunOutcome, BUILD_TOOLCHAIN_IDENTITY,
    RUSTC_FLAG_TOKENS,
};
use auto_test_case::sha256::{hex, sha256};

pub const CERT_SCHEMA: &str = "ace.m1.a2r-emit917.cert/1";
/// Emit needs more than a default thread stack (mirrors the a2r adapter).
const EMIT_STACK_BYTES: usize = 128 * 1024 * 1024;
/// The frozen dependency closure for this set (verified: no Cargo deps).
pub const DEPS_CLOSURE: &str = "std-only";

/// Frozen behavior oracle for one case.
pub enum Oracle {
    /// The run must exit 0 and print exactly this text on stdout.
    Value(&'static str),
    /// The run must exit non-zero with this marker on stderr (frozen
    /// slice-oob observation; not an Auto trap certification).
    Trap(&'static str),
}

/// One certifiable case: source (relative to the case dir), the golden
/// artifacts (relative path + project mode? + runnable under wrapper?),
/// and the hand-written reference. Golden layout convention: single-mode
/// goldens are named `<id>.single.rs` (logical artifact `<id>.rs`);
/// project-mode goldens live under `golden/<id>.project/` and their file
/// names ARE the project artifact logical names (e.g. `R07.rs`,
/// `Cargo.toml`).
pub struct CertCase {
    pub id: &'static str,
    pub source_rel: &'static str,
    /// (golden rel path, project mode emission?, runnable under wrapper?)
    pub goldens: &'static [(&'static str, bool, bool)],
    pub reference_rel: &'static str,
    pub oracle: Oracle,
}

/// The frozen case table (single source; the case dir layout is identity).
pub const CASES: &[CertCase] = &[
    CertCase {
        id: "R07",
        source_rel: "../src/R07.at",
        goldens: &[
            ("golden/r07.single.rs", false, true),
            ("golden/r07.project/R07.rs", true, false),
            ("golden/r07.project/Cargo.toml", true, false),
        ],
        reference_rel: "reference/r07.rs",
        oracle: Oracle::Value("i32:42\n"),
    },
    CertCase {
        id: "T08",
        source_rel: "../src/T08.at",
        goldens: &[
            ("golden/t08.single.rs", false, true),
            ("golden/t08.project/T08.rs", true, false),
            ("golden/t08.project/Cargo.toml", true, false),
        ],
        reference_rel: "reference/t08.rs",
        oracle: Oracle::Trap("auto str slice out of range"),
    },
    CertCase {
        id: "nb_now21",
        source_rel: "src/nb_now21.at",
        goldens: &[("golden/nb_now21.single.rs", false, true)],
        reference_rel: "reference/nb_now21.rs",
        oracle: Oracle::Value("i32:21\n"),
    },
    CertCase {
        id: "nb_caller21",
        source_rel: "src/nb_caller21.at",
        goldens: &[("golden/nb_caller21.single.rs", false, true)],
        reference_rel: "reference/nb_caller21.rs",
        oracle: Oracle::Value("i32:21\n"),
    },
    CertCase {
        id: "nb_view",
        source_rel: "src/nb_view.at",
        goldens: &[("golden/nb_view.single.rs", false, true)],
        reference_rel: "reference/nb_view.rs",
        oracle: Oracle::Value("i32:40\n"),
    },
    CertCase {
        id: "nb_move",
        source_rel: "src/nb_move.at",
        goldens: &[("golden/nb_move.single.rs", false, true)],
        reference_rel: "reference/nb_move.rs",
        oracle: Oracle::Value("i32:20\n"),
    },
    CertCase {
        id: "nb_two_roots",
        source_rel: "src/nb_two_roots.at",
        goldens: &[("golden/nb_two_roots.single.rs", false, true)],
        reference_rel: "reference/nb_two_roots.rs",
        oracle: Oracle::Value("i32:53\n"),
    },
    CertCase {
        id: "nb_nested",
        source_rel: "src/nb_nested.at",
        goldens: &[("golden/nb_nested.single.rs", false, true)],
        reference_rel: "reference/nb_nested.rs",
        oracle: Oracle::Value("i32:121\n"),
    },
    CertCase {
        id: "nb_slice_ok",
        source_rel: "src/nb_slice_ok.at",
        goldens: &[("golden/nb_slice_ok.single.rs", false, true)],
        reference_rel: "reference/nb_slice_ok.rs",
        oracle: Oracle::Value("i32:2\n"),
    },
];

pub struct CertifyArgs {
    pub case_dir: PathBuf,
    pub certs_dir: PathBuf,
    pub only: Vec<String>,
    /// None: default rustc (env RUSTC > PATH); Some(path): explicit override.
    pub rustc: Option<String>,
    /// Explicit scratch root (default: isolated temp dir; always removed).
    pub scratch: Option<PathBuf>,
}

pub struct CaseSummary {
    pub case: String,
    pub outcome: String,
    pub detail: String,
}

fn sha_file(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(hex(&sha256(&bytes)))
}

/// Emit the fresh actual artifact map for one case through the production
/// entries. `project == false` yields the single-file map `{<id>.rs: bytes}`.
fn emit_actuals(
    source: &Path,
    id: &str,
    project: bool,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let src = std::fs::read_to_string(source)
        .map_err(|e| format!("read source {}: {e}", source.display()))?;
    let handle = std::thread::Builder::new()
        .stack_size(EMIT_STACK_BYTES)
        .spawn({
            let source = source.to_path_buf();
            let id = id.to_string();
            let src = src.clone();
            move || -> Result<BTreeMap<String, Vec<u8>>, String> {
                if project {
                    let map =
                        crate::trans::rust::transpile_rust_project(source.to_str().unwrap_or(""))
                            .map_err(|e| format!("project emit: {e}"))?;
                    Ok(map
                        .into_iter()
                        .map(|(k, (bytes, _maps))| (format!("project:{k}"), bytes))
                        .collect())
                } else {
                    let mut sink = crate::trans::rust::transpile_rust_with_source_dir(
                        source.parent().unwrap_or(Path::new(".")),
                        &id,
                        &src,
                    )
                    .map_err(|e| format!("single emit: {e}"))?;
                    let bytes = sink.done().map_err(|e| format!("sink done: {e}"))?;
                    Ok(BTreeMap::from([(format!("single:{id}.rs"), bytes.clone())]))
                }
            }
        })
        .map_err(|e| format!("spawn emit thread: {e}"))?;
    handle
        .join()
        .map_err(|_| "emit thread panicked".to_string())?
}

fn judge_run(outcome: &RunOutcome, oracle: &Oracle) -> Result<String, String> {
    match oracle {
        Oracle::Value(want_stdout) => {
            if !outcome.success {
                return Err(format!(
                    "run exited {} (expected 0); stderr: {}",
                    outcome.exit,
                    String::from_utf8_lossy(&outcome.stderr[..outcome.stderr.len().min(300)])
                ));
            }
            let stdout = String::from_utf8_lossy(&outcome.stdout).to_string();
            if stdout != *want_stdout {
                return Err(format!("stdout {:?} != oracle {:?}", stdout, want_stdout));
            }
            Ok(stdout.trim().to_string())
        }
        Oracle::Trap(marker) => {
            if outcome.success {
                return Err(format!(
                    "run unexpectedly succeeded (exit 0): stdout {:?}",
                    String::from_utf8_lossy(&outcome.stdout[..outcome.stdout.len().min(200)])
                ));
            }
            let stderr = String::from_utf8_lossy(&outcome.stderr).to_string();
            if !stderr.contains(marker) {
                return Err(format!(
                    "stderr does not contain trap marker {marker:?}: {:?}",
                    &stderr[..stderr.len().min(300)]
                ));
            }
            Ok(format!("exit {} with marker", outcome.exit))
        }
    }
}

/// Compile `generated.rs` under the frozen wrapper and judge the run.
/// `compile` returning without an executable (rustc failure with a non-zero
/// status) is surfaced here — the 852 helper itself does not check success.
fn compile_run_wrapped(
    rustc: &str,
    scratch_case: &Path,
    crate_name: &str,
    oracle: &Oracle,
) -> Result<String, String> {
    let wrapper = scratch_case.join("wrapper-main.rs");
    std::fs::write(&wrapper, wrapper_source()).map_err(|e| format!("write wrapper: {e}"))?;
    let exe = scratch_case.join(format!("{crate_name}.exe"));
    compile(rustc, &wrapper, &exe, crate_name)?;
    if !exe.is_file() {
        return Err(format!("compile produced no executable for {crate_name}"));
    }
    let outcome = run_capped(&exe)?;
    judge_run(&outcome, oracle)
}

/// How an existing receipt relates to the current context.
/// Strict validation: ANY structural break is an Invalid reason (hard error —
/// the receipt is never reused and never silently overwritten). Content drift
/// on otherwise-valid receipts is handled by the caller as an honest
/// re-certification.
pub type ReceiptProblem = String;

fn is_hash64(v: &serde_json::Value) -> bool {
    v.as_str()
        .map(|s| s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or(false)
}

fn exact_keys(v: &serde_json::Value, want: &[&str], ctx: &str) -> Result<(), String> {
    let obj = v
        .as_object()
        .ok_or_else(|| format!("{ctx}: not an object"))?;
    let got: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
    if got.len() != want.len() || !want.iter().all(|w| obj.contains_key(*w)) {
        return Err(format!("{ctx}: keys {got:?} != expected {want:?}"));
    }
    Ok(())
}

/// STRICT receipt structural validation. Any structural break returns
/// Err(reason) — never reuse, never silently overwrite.
pub fn validate_receipt(receipt: &serde_json::Value) -> Result<(), ReceiptProblem> {
    let bad = |m: String| Err(m);
    if receipt.get("schema").and_then(|v| v.as_str()) != Some(CERT_SCHEMA) {
        return bad(format!("schema is not {CERT_SCHEMA}"));
    }
    exact_keys(
        receipt,
        &[
            "schema",
            "case",
            "verdict",
            "oracle_kind",
            "oracle_text",
            "observed",
            "axes",
            "identity",
            "deps_closure",
        ],
        "receipt",
    )?;
    if receipt.get("verdict").and_then(|v| v.as_str()) != Some("certified") {
        return bad("verdict is not certified".to_string());
    }
    let oracle_kind = receipt
        .get("oracle_kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "oracle_kind".to_string())?;
    if !matches!(oracle_kind, "value" | "trap") {
        return bad(format!("oracle_kind {oracle_kind:?} unknown"));
    }
    let observed = receipt
        .get("observed")
        .ok_or_else(|| "observed".to_string())?;
    exact_keys(observed, &["actual", "reference"], "observed")?;
    let axes = receipt.get("axes").ok_or_else(|| "axes".to_string())?;
    exact_keys(axes, &["source", "goldens", "wrapper", "reference"], "axes")?;
    if !is_hash64(&axes["source"]) {
        return bad("axes.source is not a sha256 hex".to_string());
    }
    let goldens = axes["goldens"]
        .as_object()
        .ok_or_else(|| "axes.goldens".to_string())?;
    if goldens.is_empty() {
        return bad("axes.goldens is empty".to_string());
    }
    for (k, v) in goldens {
        if k.is_empty() || !is_hash64(v) {
            return bad(format!("axes.goldens[{k:?}] is not a sha256 hex"));
        }
    }
    if !is_hash64(&axes["wrapper"]) || !is_hash64(&axes["reference"]) {
        return bad("axes.wrapper / axes.reference is not a sha256 hex".to_string());
    }
    let identity = receipt
        .get("identity")
        .ok_or_else(|| "identity".to_string())?;
    exact_keys(
        identity,
        &[
            "toolchain",
            "identity_source",
            "rustc_flags",
            "default_identity_at_certify",
        ],
        "identity",
    )?;
    Ok(())
}

/// The frozen wrapper: byte-identical to the corpus wrapper
/// `oracle/wrapper-main-v1.rs` (its SHA is an identity axis). Single source
/// of truth stays the corpus file; this accessor exists so the certifier and
/// its tests share one read path.
pub fn wrapper_source() -> &'static str {
    super::ace_certify::WRAPPER_MAIN_RS
}

fn certify_one(
    case: &CertCase,
    args: &CertifyArgs,
    scratch_root: &Path,
    rustc: &str,
    toolchain_identity: &str,
    identity_source: &str,
) -> CaseSummary {
    let fail = |detail: String| CaseSummary {
        case: case.id.to_string(),
        outcome: "fail".to_string(),
        detail,
    };
    let case_dir = &args.case_dir;
    let source = case_dir.join(case.source_rel);

    // --- 1. Fresh actuals from the production entries.
    let mut actuals: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let has_project = case.goldens.iter().any(|(_, project, _)| *project);
    match emit_actuals(&source, case.id, false) {
        Ok(m) => actuals.extend(m),
        Err(e) => return fail(format!("single emit: {e}")),
    }
    if has_project {
        match emit_actuals(&source, case.id, true) {
            Ok(m) => actuals.extend(m),
            Err(e) => return fail(format!("project emit: {e}")),
        }
    }

    // --- 2. Byte drift vs frozen goldens (never auto-update).
    let mut golden_shas: BTreeMap<String, String> = BTreeMap::new();
    for (golden_rel, _project, _runnable) in case.goldens {
        let golden_path = case_dir.join(golden_rel);
        let want = match std::fs::read(&golden_path) {
            Ok(b) => b,
            Err(e) => return fail(format!("read golden {golden_rel}: {e}")),
        };
        // Logical artifact name in the production emission map (mode-prefixed
        // to keep the single/project artifacts apart).
        let logical = if golden_rel.ends_with(".single.rs") {
            format!("single:{}.rs", case.id)
        } else {
            format!(
                "project:{}",
                golden_rel.rsplit('/').next().unwrap_or(golden_rel)
            )
        };
        let actual = match actuals.get(&logical) {
            Some(b) => b,
            None => return fail(format!("actual artifact {logical} missing")),
        };
        if actual.as_slice() != want.as_slice() {
            if let Ok(dump) = std::env::var("A2R917_DEBUG_DUMP") {
                let _ = std::fs::create_dir_all(&dump);
                let _ = std::fs::write(
                    std::path::Path::new(&dump).join(format!("actual-{}.rs", case.id)),
                    actual,
                );
                let _ = std::fs::write(
                    std::path::Path::new(&dump).join(format!("golden-{}.rs", case.id)),
                    &want,
                );
            }
            return fail(format!(
                "golden drift on {logical}: actual {} != golden {} ({golden_rel}) — \
                 review and update the golden explicitly; this tool never overwrites it",
                hex(&sha256(actual)),
                hex(&sha256(&want))
            ));
        }
        golden_shas.insert(golden_rel.to_string(), hex(&sha256(&want)));
    }

    // --- 3. Strict receipt reuse: Invalid = hard stop; Stale = re-certify.
    let cert_path = args.certs_dir.join(format!("{}.cert.json", case.id));
    let (oracle_kind, oracle_text) = match &case.oracle {
        Oracle::Value(t) => ("value", t.to_string()),
        Oracle::Trap(m) => ("trap", m.to_string()),
    };
    let current_axes = || -> Result<(String, String, String, String), String> {
        Ok((
            sha_file(&source)?,
            sha_file(&case_dir.join(case.reference_rel))?,
            sha_file(&case_dir.join("../oracle/wrapper-main-v1.rs"))?,
            hex(&sha256(wrapper_source().as_bytes())),
        ))
    };
    if let Ok(existing_text) = std::fs::read_to_string(&cert_path) {
        let existing: serde_json::Value = match serde_json::from_str(&existing_text) {
            Ok(v) => v,
            Err(e) => {
                return fail(format!(
                    "existing receipt is structurally invalid ({e}); refusing to reuse or \
                     overwrite it — repair or delete the file first"
                ))
            }
        };
        if let Err(reason) = validate_receipt(&existing) {
            return fail(format!(
                "existing receipt structurally invalid ({reason}); refusing to reuse or \
                 overwrite it — repair or delete the file first"
            ));
        }
        // Structural drift check on the file-derived axes.
        let axes = &existing["axes"];
        let stale = (|| -> Option<String> {
            let (source_sha, reference_sha, wrapper_file_sha, _wrapper_const_sha) =
                current_axes().ok()?;
            if axes["source"].as_str()? != source_sha {
                return Some("source".to_string());
            }
            if axes["reference"].as_str()? != reference_sha {
                return Some("reference".to_string());
            }
            if axes["wrapper"].as_str()? != wrapper_file_sha {
                return Some("wrapper".to_string());
            }
            let goldens = axes["goldens"].as_object()?;
            for (k, v) in &golden_shas {
                if goldens.get(k)?.as_str()? != v {
                    return Some(format!("golden {k}"));
                }
            }
            if existing["oracle_kind"].as_str()? != oracle_kind
                || existing["oracle_text"].as_str()? != oracle_text
            {
                return Some("oracle".to_string());
            }
            None
        })();
        if stale.is_none() {
            return CaseSummary {
                case: case.id.to_string(),
                outcome: "reused".to_string(),
                detail: "receipt valid, all identity axes current".to_string(),
            };
        }
        // Stale: fall through to honest re-certification.
    }
    let (source_sha, reference_sha, wrapper_file_sha, _) = match current_axes() {
        Ok(a) => a,
        Err(e) => return fail(e),
    };

    // --- 4. Compile + run, contained: each runnable actual and the reference.
    let scratch_case = scratch_root.join(case.id);
    if let Err(e) = std::fs::create_dir_all(&scratch_case) {
        return fail(format!("scratch: {e}"));
    }
    let mut observed_actual: Vec<String> = Vec::new();
    for (golden_rel, _project, runnable) in case.goldens {
        if !runnable {
            continue;
        }
        let logical = if golden_rel.ends_with(".single.rs") {
            format!("single:{}.rs", case.id)
        } else {
            format!(
                "project:{}",
                golden_rel.rsplit('/').next().unwrap_or(golden_rel)
            )
        };
        let bytes = match actuals.get(&logical) {
            Some(b) => b.clone(),
            None => return fail(format!("actual artifact {logical} missing for run")),
        };
        let sub = scratch_case.join(
            golden_rel
                .trim_end_matches(".rs")
                .replace(['/', '.', ':', '-'], "_"),
        );
        if let Err(e) = std::fs::create_dir_all(&sub) {
            return fail(format!("scratch: {e}"));
        }
        if let Err(e) = std::fs::write(sub.join("generated.rs"), &bytes) {
            return fail(format!("write generated.rs: {e}"));
        }
        let crate_name = format!(
            "p917_{}",
            golden_rel
                .trim_end_matches(".rs")
                .replace(['/', '.', ':', '-'], "_")
        );
        match compile_run_wrapped(rustc, &sub, &crate_name, &case.oracle) {
            Ok(observed) => observed_actual.push(format!("{logical}: {observed}")),
            Err(e) => return fail(format!("actual {logical} run: {e}")),
        }
    }
    // Reference: hand-written, same wrapper + oracle.
    let reference_bytes = match std::fs::read(case_dir.join(case.reference_rel)) {
        Ok(b) => b,
        Err(e) => return fail(format!("read reference: {e}")),
    };
    let ref_sub = scratch_case.join("reference");
    if let Err(e) = std::fs::create_dir_all(&ref_sub) {
        return fail(format!("scratch: {e}"));
    }
    if let Err(e) = std::fs::write(ref_sub.join("generated.rs"), &reference_bytes) {
        return fail(format!("write reference generated.rs: {e}"));
    }
    let observed_reference =
        match compile_run_wrapped(rustc, &ref_sub, "p917_reference", &case.oracle) {
            Ok(observed) => observed,
            Err(e) => return fail(format!("reference run: {e}")),
        };

    // --- 5. Persist the receipt.
    let receipt = serde_json::json!({
        "schema": CERT_SCHEMA,
        "case": case.id,
        "verdict": "certified",
        "oracle_kind": oracle_kind,
        "oracle_text": oracle_text,
        "observed": {
            "actual": observed_actual,
            "reference": format!("reference: {observed_reference}"),
        },
        "axes": {
            "source": source_sha,
            "goldens": golden_shas,
            "wrapper": wrapper_file_sha,
            "reference": reference_sha,
        },
        "identity": {
            "toolchain": toolchain_identity,
            "identity_source": identity_source,
            "rustc_flags": RUSTC_FLAG_TOKENS.join(" "),
            "default_identity_at_certify": BUILD_TOOLCHAIN_IDENTITY.trim(),
        },
        "deps_closure": DEPS_CLOSURE,
    });
    if let Err(e) = std::fs::write(&cert_path, serde_json::to_string_pretty(&receipt).unwrap()) {
        return fail(format!("write receipt: {e}"));
    }
    CaseSummary {
        case: case.id.to_string(),
        outcome: "certified".to_string(),
        detail: format!(
            "compiled+ran {} actual artifact(s) and the reference under the frozen wrapper",
            observed_actual.len()
        ),
    }
}

/// One certification pass over the case table.
pub fn certify(args: &CertifyArgs) -> Result<Vec<CaseSummary>, String> {
    let selected: Vec<&CertCase> = CASES
        .iter()
        .filter(|c| args.only.is_empty() || args.only.iter().any(|o| o.eq_ignore_ascii_case(c.id)))
        .collect();
    if selected.is_empty() {
        return Err("--only matched no case".to_string());
    }
    std::fs::create_dir_all(&args.certs_dir).map_err(|e| format!("certs dir: {e}"))?;
    let scratch_root = match &args.scratch {
        Some(p) => p.clone(),
        None => std::env::temp_dir().join(format!("a2r-emit-917-certify-{}", std::process::id())),
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
        for case in selected {
            summaries.push(certify_one(
                case,
                args,
                &scratch_root,
                &rustc,
                &toolchain_identity,
                identity_source,
            ));
        }
        Ok(summaries)
    })();
    let _ = std::fs::remove_dir_all(&scratch_root);
    result
}

/// Run certification and reduce to an exit code
/// (0 = all certified/reused, 1 = any failure, 2 = contract error).
pub fn certify_and_report(args: &CertifyArgs) -> i32 {
    match certify(args) {
        Ok(summaries) => {
            let mut fails = 0;
            for s in &summaries {
                println!("{}\t{}\t{}", s.case, s.outcome, s.detail);
                if s.outcome == "fail" {
                    fails += 1;
                }
            }
            if fails > 0 {
                1
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("a2r-emit-917-cert: {e}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_receipt() -> serde_json::Value {
        serde_json::json!({
            "schema": CERT_SCHEMA,
            "case": "R07",
            "verdict": "certified",
            "oracle_kind": "value",
            "oracle_text": "i32:42\n",
            "observed": { "actual": ["x"], "reference": "y" },
            "axes": {
                "source": "a".repeat(64),
                "goldens": { "golden/r07.single.rs": "b".repeat(64) },
                "wrapper": "c".repeat(64),
                "reference": "d".repeat(64),
            },
            "identity": {
                "toolchain": "rustc 1.98.0",
                "identity_source": "PATH",
                "rustc_flags": "--edition=2021 -O",
                "default_identity_at_certify": "rustc 1.98.0",
            },
            "deps_closure": "std-only",
        })
    }

    #[test]
    fn strict_receipt_validation() {
        let ok = valid_receipt();
        assert!(validate_receipt(&ok).is_ok());
        // Wrong schema: invalid.
        let mut bad = ok.clone();
        bad["schema"] = serde_json::json!("ace.m1.cert/1");
        assert!(validate_receipt(&bad).is_err());
        // Extra key: invalid.
        let mut bad = ok.clone();
        bad["extra"] = serde_json::json!(1);
        assert!(validate_receipt(&bad).is_err());
        // Non-hex source hash: invalid.
        let mut bad = ok.clone();
        bad["axes"]["source"] = serde_json::json!("not-a-hash");
        assert!(validate_receipt(&bad).is_err());
        // Bad oracle kind: invalid.
        let mut bad = ok.clone();
        bad["oracle_kind"] = serde_json::json!("other");
        assert!(validate_receipt(&bad).is_err());
    }

    #[test]
    fn case_table_is_well_formed() {
        let mut seen = std::collections::BTreeSet::new();
        for case in CASES {
            assert!(seen.insert(case.id), "duplicate case id {}", case.id);
            assert!(!case.goldens.is_empty(), "{} has no goldens", case.id);
            assert!(
                case.goldens.iter().any(|(_, _, runnable)| *runnable),
                "{} has no runnable artifact",
                case.id
            );
        }
    }
}
