//! PLAN-852 T-04: the EXPLICIT independent a2r certification entry
//! (`ace-certify`). One certification, daily snapshot (T-03) — this module
//! is the only place that compiles and runs Rust for the ace-m1 corpus, and
//! it is never invoked by the suite runner or any daily scope.
//!
//! Contract (frozen in docs/plans/attachments/852/contract-decision.md §3,
//! as repaired for F-852-R1-01/03):
//! 1. Emit the ACTUAL Rust fresh from the corpus source through the
//!    production transpiler (never read the golden as the actual).
//! 2. Require actual bytes == frozen golden bytes (a drift FAILs; goldens
//!    are only updated through explicit review — G04).
//! 3. Compile AND run (a) the actual bytes under the frozen host wrapper
//!    (byte-identical `generated.rs` + bridging `mod`, hashed identity) and
//!    (b) the hand-written `reference/<id>.rs`; BOTH must print the fixed
//!    behavior oracle `i32:<decimal>\n` exactly.
//! 4. Persist a version-controlled certificate (schema `ace.m1.cert/1`)
//!    binding every identity axis, and REUSE an existing receipt only after
//!    STRICT validation: structure (schema/case/module/verdict/oracle text/
//!    observed runs/frozen flags/deps) must be internally valid — any
//!    structural break is a hard error, never a reuse and never a silent
//!    re-certification — while file-content drift (source/golden/wrapper/
//!    reference/toolchain) marks the receipt STALE and re-certifies.
//!
//! Stream governance (F-852-R1-03): stdout and stderr are drained by
//! INDEPENDENT reader threads (a child filling one pipe can never deadlock
//! the other); exceeding a stream cap kills the whole job tree and surfaces
//! a parent-visible overflow error (nothing is silently truncated); the
//! compile face counts stdout too. Tool identity comes from the BUILD-TIME
//! captured `$RUSTC -vV` (zero spawns on the default path; an explicit
//! `--rustc` override is captured through the same contained, bounded run).
//!
//! Contained-tool governance (contract-decision §3 budgets): rustc and the
//! certified binaries start SUSPENDED, join a kill-on-close Job Object
//! before their first instruction, and run under a 120s compile wall / 30s
//! run wall / 1MB stream caps; the whole scratch tree is removed on every
//! exit path (explicit `--scratch`, else an isolated temp dir).

use crate::test_cases::sema::containment;
use auto_test_case::sha256::{hex, sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub const CERT_SCHEMA: &str = "ace.m1.cert/1";
pub const COMPILE_WALL: Duration = Duration::from_secs(120);
pub const RUN_WALL: Duration = Duration::from_secs(30);
pub const STREAM_CAP: usize = 1 << 20;
/// Emit needs more than a default thread stack (mirrors the a2r adapter).
const EMIT_STACK_BYTES: usize = 128 * 1024 * 1024;

/// The frozen certification wrapper (contract-decision §3). Single source:
/// the versioned corpus file; its hash is an identity axis of every
/// certificate.
pub const WRAPPER_MAIN_RS: &str =
    include_str!("../../../../crates/auto-lang/test/cases/ace-m1/oracle/wrapper-main-v1.rs");

/// Build-time toolchain identity (`$RUSTC -vV`: rustc version, release,
/// host triple, commit hash) captured by build.rs. Used as the certificate
/// toolchain identity on the default path so neither the certifier nor the
/// daily validity check ever spawns rustc for identity.
pub const BUILD_TOOLCHAIN_IDENTITY: &str =
    include_str!(concat!(env!("OUT_DIR"), "/build_toolchain.txt"));

/// The frozen effective flag TOKENS handed to rustc (identity axis; the
/// crate name, input and `-o` are per-invocation paths, not identity).
pub const RUSTC_FLAG_TOKENS: [&str; 2] = ["--edition=2021", "-O"];

/// The frozen dependency closure marker for the scalar set (std only).
pub const DEPS_CLOSURE: &str = "std-only";

pub struct CertifyArgs {
    pub corpus: PathBuf,
    pub certs_dir: PathBuf,
    pub only: Vec<String>,
    pub force: bool,
    /// None: default rustc (identity = build-time capture, zero spawns).
    /// Some(path): explicit override (identity captured contained).
    pub rustc: Option<String>,
    /// Explicit scratch root (default: isolated temp dir; always removed).
    pub scratch: Option<PathBuf>,
}

pub struct CaseSummary {
    pub case: String,
    pub outcome: String,
    pub detail: String,
}

/// Parse binding.tsv rows for the certifiable set: corpus value rows whose
/// a2r cell is exactly `ok-emit` (snapshot-only rows — composite pseudo-Rust
/// that cannot compile — are deliberately excluded and stay registered gaps).
fn certifiable_rows(binding: &Path) -> Result<Vec<(String, String)>, String> {
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
        if f[1] == "corpus" && f[3] == "value" && f[8] == "ok-emit" {
            rows.push((f[0].to_string(), f[4].to_string()));
        }
    }
    if rows.is_empty() {
        return Err("no certifiable rows (expected the scalar ok-emit set)".to_string());
    }
    Ok(rows)
}

/// One certification pass over the corpus. Returns per-case summaries; the
/// caller decides the exit code from `outcome` (`certified`/`reused`/`fail`).
pub fn certify(args: &CertifyArgs) -> Result<Vec<CaseSummary>, String> {
    let rows = certifiable_rows(&args.corpus.join("binding.tsv"))?;
    let selected: Vec<&(String, String)> = rows
        .iter()
        .filter(|(id, _)| {
            args.only.is_empty() || args.only.iter().any(|o| o.eq_ignore_ascii_case(id))
        })
        .collect();
    if selected.is_empty() {
        return Err("--only matched no certifiable case".to_string());
    }
    std::fs::create_dir_all(&args.certs_dir).map_err(|e| format!("certs dir: {e}"))?;

    let scratch_root = match &args.scratch {
        Some(p) => p.clone(),
        None => std::env::temp_dir().join(format!("ace-m1-certify-{}", std::process::id())),
    };
    let _ = std::fs::remove_dir_all(&scratch_root);
    std::fs::create_dir_all(&scratch_root).map_err(|e| format!("scratch: {e}"))?;
    let rustc = args
        .rustc
        .clone()
        .unwrap_or_else(|| std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
    let toolchain_identity = match &args.rustc {
        None => BUILD_TOOLCHAIN_IDENTITY.trim().to_string(),
        Some(path) => capture_toolchain_contained(path)?,
    };
    let result = (|| {
        let mut summaries = Vec::new();
        for (id, expected) in selected {
            let summary = certify_one(
                &id,
                expected,
                args,
                &scratch_root,
                &rustc,
                &toolchain_identity,
            );
            summaries.push(summary);
        }
        Ok(summaries)
    })();
    let _ = std::fs::remove_dir_all(&scratch_root);
    result
}

/// How an existing receipt relates to the current context.
pub enum ReceiptProblem {
    /// Structurally invalid / internally inconsistent — possible tampering.
    /// Hard error: never reused, never silently overwritten (even with
    /// --force); the file must be manually deleted or repaired first.
    Invalid(String),
    /// Valid receipt whose file-derived axes drifted — the honest
    /// re-certification path (G05).
    Stale(String),
}

/// Current file-derived identity axes for one case.
pub struct CurrentAxes {
    pub source_sha: String,
    pub golden_sha: String,
    pub wrapper_sha: String,
    pub oracle_sha: String,
    pub reference_sha: String,
}

/// STRICT receipt validation (F-852-R1-01). Every structural field is
/// checked against the frozen contract; every file axis against the current
/// tree. `Ok(())` means the receipt is valid AND current (reusable).
pub fn validate_receipt(
    cert: &serde_json::Value,
    case_id: &str,
    expected_oracle: &str,
    axes: &CurrentAxes,
    toolchain_identity: &str,
) -> Result<(), ReceiptProblem> {
    let invalid = |what: String| ReceiptProblem::Invalid(what);
    if cert.get("schema").and_then(|v| v.as_str()) != Some(CERT_SCHEMA) {
        return Err(invalid(format!("schema is not {CERT_SCHEMA}")));
    }
    if cert.get("case").and_then(|v| v.as_str()) != Some(case_id) {
        return Err(invalid(format!("case is not {case_id}")));
    }
    let module = format!("m1_{}", case_id.to_lowercase());
    if cert.get("module").and_then(|v| v.as_str()) != Some(&module) {
        return Err(invalid(format!("module is not {module}")));
    }
    if cert.get("verdict").and_then(|v| v.as_str()) != Some("certified") {
        return Err(invalid("verdict is not certified".to_string()));
    }
    if cert.get("expected_oracle").and_then(|v| v.as_str()) != Some(expected_oracle) {
        return Err(invalid(format!(
            "expected_oracle is not the frozen registry value {expected_oracle:?}"
        )));
    }
    // Observed runs must equal the oracle text (self-consistent evidence).
    for side in ["actual_run", "reference_run"] {
        let stdout = cert
            .pointer(&format!("/observed/{side}/stdout"))
            .and_then(|v| v.as_str());
        if stdout != Some(expected_oracle) {
            return Err(invalid(format!(
                "observed.{side}.stdout does not equal the fixed oracle"
            )));
        }
        let exit = cert
            .pointer(&format!("/observed/{side}/exit"))
            .and_then(|v| v.as_i64());
        if exit != Some(0) {
            return Err(invalid(format!("observed.{side}.exit is not 0")));
        }
    }
    // Frozen contract constants recorded verbatim.
    let flags: Vec<&str> = cert
        .get("toolchain")
        .and_then(|t| t.get("flags"))
        .and_then(|f| f.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    if flags != RUSTC_FLAG_TOKENS {
        return Err(invalid(format!(
            "toolchain.flags is not the frozen set {:?} (recorded {:?})",
            RUSTC_FLAG_TOKENS, flags
        )));
    }
    let deps = cert
        .pointer("/deps/closure")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if deps != DEPS_CLOSURE {
        return Err(invalid(format!(
            "deps.closure is not {DEPS_CLOSURE:?} (recorded {deps:?})"
        )));
    }
    let external: Vec<&str> = cert
        .pointer("/deps/external")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    if !external.is_empty() {
        return Err(invalid("deps.external is not empty".to_string()));
    }
    let id = match cert.get("identity") {
        Some(i) => i,
        None => return Err(invalid("identity object missing".to_string())),
    };
    let actual = id
        .get("actual_sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let golden = id
        .get("golden_sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if actual != golden {
        return Err(invalid(
            "identity.actual_sha256 != golden_sha256 (self-inconsistent)".to_string(),
        ));
    }
    // File-derived axes: drift here is STALE (the legitimate re-certify path).
    let stale = |what: String| ReceiptProblem::Stale(what);
    for (key, current, label) in [
        ("source_sha256", &axes.source_sha, "source"),
        ("golden_sha256", &axes.golden_sha, "golden"),
        ("wrapper_sha256", &axes.wrapper_sha, "wrapper"),
        ("oracle_sha256", &axes.oracle_sha, "oracle"),
        ("reference_sha256", &axes.reference_sha, "reference"),
    ] {
        let recorded = id.get(key).and_then(|v| v.as_str()).unwrap_or("");
        if recorded != *current {
            return Err(stale(format!("{label} drifted (receipt axis {key})")));
        }
    }
    let recorded_tc = cert
        .pointer("/toolchain/identity")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if recorded_tc != toolchain_identity {
        return Err(stale("toolchain identity drifted".to_string()));
    }
    Ok(())
}

/// Compute the current file-derived axes for one case.
pub fn current_axes(corpus: &Path, id: &str, expected: &str) -> Result<CurrentAxes, String> {
    let module = format!("m1_{}", id.to_lowercase());
    let read = |p: PathBuf, what: &str| -> Result<Vec<u8>, String> {
        std::fs::read(&p).map_err(|e| format!("read {what}: {e}"))
    };
    let source = read(corpus.join("src").join(format!("{id}.at")), "source")?;
    let golden = read(corpus.join("oracle").join(format!("{module}.rs")), "golden")?;
    let reference = read(
        corpus.join("reference").join(format!("{id}.rs")),
        "reference",
    )?;
    Ok(CurrentAxes {
        source_sha: hex(&sha256(&source)),
        golden_sha: hex(&sha256(&golden)),
        wrapper_sha: hex(&sha256(WRAPPER_MAIN_RS.as_bytes())),
        oracle_sha: hex(&sha256(format!("i32:{expected}\n").as_bytes())),
        reference_sha: hex(&sha256(&reference)),
    })
}

#[allow(clippy::too_many_lines)]
fn certify_one(
    id: &str,
    expected: &str,
    args: &CertifyArgs,
    scratch_root: &Path,
    rustc: &str,
    toolchain_identity: &str,
) -> CaseSummary {
    let module = format!("m1_{}", id.to_lowercase());
    let oracle_text = format!("i32:{expected}\n");
    let case_scratch = scratch_root.join(&module);
    let _ = std::fs::create_dir_all(&case_scratch);

    let fail = |detail: String| CaseSummary {
        case: id.to_string(),
        outcome: "fail".to_string(),
        detail,
    };

    // ---- 1. fresh actual through the production transpiler ----------------
    let source_path = args.corpus.join("src").join(format!("{id}.at"));
    let source = match std::fs::read_to_string(&source_path) {
        Ok(s) => s,
        Err(e) => return fail(format!("read source: {e}")),
    };
    let emit = std::thread::Builder::new()
        .stack_size(EMIT_STACK_BYTES)
        .spawn({
            let source_dir = args.corpus.join("src");
            let module = module.clone();
            let source = source.clone();
            move || {
                let mut sink = crate::trans::rust::transpile_rust_with_source_dir(
                    &source_dir,
                    &module,
                    &source,
                )?;
                let bytes = sink.done()?.clone();
                Ok::<_, crate::error::AutoError>(bytes)
            }
        })
        .and_then(|h| Ok(h.join()));
    let actual: Vec<u8> = match emit {
        Ok(Ok(Ok(bytes))) => bytes,
        Ok(Ok(Err(e))) => return fail(format!("a2r emit failed: {e}")),
        Ok(Err(payload)) => {
            return fail(format!(
                "emit thread panicked: {}",
                super::avm::panic_message(&payload)
            ))
        }
        Err(e) => return fail(format!("spawn emit thread: {e}")),
    };

    // ---- 2. actual == frozen golden (drift never re-certifies) -------------
    let golden_path = args.corpus.join("oracle").join(format!("{module}.rs"));
    let golden = match std::fs::read(&golden_path) {
        Ok(g) => g,
        Err(e) => return fail(format!("read golden: {e}")),
    };
    if actual != golden {
        return fail(format!(
            "actual != golden for {id} ({} vs {} bytes) — explicit review required, no auto-update",
            actual.len(),
            golden.len()
        ));
    }

    // ---- identity axes ------------------------------------------------------
    let axes = match current_axes(&args.corpus, id, expected) {
        Ok(a) => a,
        Err(e) => return fail(e),
    };

    // ---- reuse (G03/G08): STRICT validation; tampering is a hard error ----
    // Structural validation runs REGARDLESS of --force: a broken receipt is
    // never silently overwritten, only manually repaired or deleted.
    // --force skips REUSE of a valid receipt (fresh certification) but not
    // the rejection of an invalid one.
    let cert_path = args.certs_dir.join(format!("{module}.cert.json"));
    if let Ok(old_text) = std::fs::read_to_string(&cert_path) {
        match serde_json::from_str::<serde_json::Value>(&old_text) {
            Ok(old) => match validate_receipt(&old, id, &oracle_text, &axes, toolchain_identity) {
                Ok(()) => {
                    if !args.force {
                        return CaseSummary {
                            case: id.to_string(),
                            outcome: "reused".to_string(),
                            detail: "strict receipt validation passed — axes unchanged, nothing compiled (G03/G08)".to_string(),
                        };
                    }
                    // --force: proceed to a fresh certification below.
                }
                Err(ReceiptProblem::Invalid(why)) => {
                    return fail(format!(
                        "REJECTED receipt {module}.cert.json: {why} — structurally invalid (possible tampering); manually delete or repair it, then certify again. Never reused, never silently overwritten (not even with --force)."
                    ));
                }
                Err(ReceiptProblem::Stale(_why)) => {
                    // Valid receipt, drifted context: re-certify below.
                }
            },
            Err(e) => {
                return fail(format!(
                    "REJECTED receipt {module}.cert.json: not valid JSON ({e}) — manually delete or repair it, then certify again"
                ));
            }
        }
    }

    // ---- 3. compile & run actual (wrapper) and reference --------------------
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
    if !actual_run.success || actual_run.stdout != oracle_text.as_bytes() {
        return fail(format!(
            "actual-wrapper run mismatched the fixed oracle: exit={} stdout={:?} (expected {:?})",
            actual_run.exit,
            String::from_utf8_lossy(&actual_run.stdout),
            oracle_text
        ));
    }

    let ref_exe = case_scratch.join(format!("{module}_ref.exe"));
    if let Err(e) = compile(
        rustc,
        &args.corpus.join("reference").join(format!("{id}.rs")),
        &ref_exe,
        &crate_name,
    ) {
        return fail(format!("compile reference: {e}"));
    }
    let reference_run = match run_capped(&ref_exe) {
        Ok(r) => r,
        Err(e) => return fail(format!("run reference: {e}")),
    };
    if !reference_run.success || reference_run.stdout != oracle_text.as_bytes() {
        return fail(format!(
            "reference run mismatched the fixed oracle: exit={} stdout={:?} (expected {:?})",
            reference_run.exit,
            String::from_utf8_lossy(&reference_run.stdout),
            oracle_text
        ));
    }

    // ---- 4. certificate ------------------------------------------------------
    let cert = serde_json::json!({
        "schema": CERT_SCHEMA,
        "case": id,
        "module": module,
        "wrapper_file": "oracle/wrapper-main-v1.rs",
        "expected_oracle": oracle_text,
        "identity": {
            "source_sha256": axes.source_sha,
            "golden_sha256": axes.golden_sha,
            "actual_sha256": axes.golden_sha,
            "wrapper_sha256": axes.wrapper_sha,
            "oracle_sha256": axes.oracle_sha,
            "reference_sha256": axes.reference_sha,
        },
        "toolchain": {
            "identity": toolchain_identity,
            "flags": RUSTC_FLAG_TOKENS,
        },
        "deps": { "external": [], "closure": DEPS_CLOSURE },
        "observed": {
            "actual_run": { "exit": actual_run.exit, "stdout": oracle_text },
            "reference_run": { "exit": reference_run.exit, "stdout": oracle_text },
        },
        "verdict": "certified",
        "produced_by": { "tool": "ace-certify", "schema_note": "transpiler identity is audit-only, not a staleness axis" },
        "created_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    });
    let written = serde_json::to_string_pretty(&cert)
        .map_err(|e| e.to_string())
        .and_then(|p| std::fs::write(&cert_path, p).map_err(|e| e.to_string()));
    match written {
        Ok(()) => CaseSummary {
            case: id.to_string(),
            outcome: "certified".to_string(),
            detail: format!(
                "actual==golden; wrapper+reference both printed the fixed oracle; cert {module}.cert.json"
            ),
        },
        Err(e) => fail(format!("write certificate: {e}")),
    }
}

/// Capture a tool identity for an EXPLICIT --rustc override through the
/// same contained, bounded execution as every other tool invocation.
fn capture_toolchain_contained(rustc: &str) -> Result<String, String> {
    let mut cmd = Command::new(rustc);
    cmd.arg("-vV").stdout(Stdio::piped()).stderr(Stdio::piped());
    let out = run_bounded(cmd, COMPILE_WALL, "rustc -vV (identity capture)")?;
    if !out.success || out.stdout.is_empty() {
        return Err(format!(
            "rustc -vV failed (exit {}): {}",
            out.exit,
            String::from_utf8_lossy(&out.stderr[..out.stderr.len().min(400)])
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[derive(Debug)]
pub struct RunOutcome {
    pub success: bool,
    pub exit: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Run a certified binary under containment with the run wall and caps.
fn run_capped(exe: &Path) -> Result<RunOutcome, String> {
    let mut cmd = Command::new(exe);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let out = run_bounded(cmd, RUN_WALL, &format!("run {}", exe.display()))?;
    Ok(RunOutcome {
        success: out.success,
        exit: out.exit,
        stdout: out.stdout,
        stderr: out.stderr,
    })
}

/// Compile one file under containment with the frozen flags and wall.
fn compile(rustc: &str, input: &Path, out_exe: &Path, crate_name: &str) -> Result<(), String> {
    let mut cmd = Command::new(rustc);
    cmd.args(RUSTC_FLAG_TOKENS)
        .arg("--crate-name")
        .arg(crate_name)
        .arg(input.as_os_str().to_string_lossy().as_ref())
        .arg("-o")
        .arg(out_exe.as_os_str().to_string_lossy().as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    run_bounded(cmd, COMPILE_WALL, "rustc compile").map(|_| ())
}

/// Spawn a command SUSPENDED, place it in a kill-on-close Job Object before
/// its first instruction, resume it, and wait inside `wall`. stdout and
/// stderr are drained by INDEPENDENT capped reader threads (F-852-R1-03: a
/// child flooding one pipe can never block the other); when either stream
/// exceeds STREAM_CAP the WHOLE job tree is killed and a parent-visible
/// overflow error is returned — output is never silently truncated. The
/// wall overrun kills the tree the same way.
fn run_bounded(mut cmd: Command, wall: Duration, what: &str) -> Result<RunOutcome, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let started = Instant::now();
    let mut child = cmd.spawn().map_err(|e| format!("spawn {what}: {e}"))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "containment unavailable: refusing an uncontained {what}"
            ));
        }
    };
    #[cfg(windows)]
    containment::resume_suspended_process(&child).map_err(|e| format!("resume {what}: {e}"))?;
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;

    // Independent per-stream capped readers; the overflow latches are
    // observed by the parent loop, which owns the kill.
    let stdout_overflow = std::sync::Arc::new(AtomicBool::new(false));
    let stderr_overflow = std::sync::Arc::new(AtomicBool::new(false));
    fn spawn_reader<R: Read + Send + 'static>(
        pipe: Option<R>,
        latch: std::sync::Arc<AtomicBool>,
    ) -> std::thread::JoinHandle<Vec<u8>> {
        std::thread::spawn(move || {
            let mut pipe = match pipe {
                Some(p) => p,
                None => return Vec::new(),
            };
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            loop {
                match pipe.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.len() > STREAM_CAP {
                            latch.store(true, Ordering::SeqCst);
                            break; // parent kills the tree; we stop draining
                        }
                    }
                }
            }
            buf
        })
    }
    let out_handle = spawn_reader(child.stdout.take(), stdout_overflow.clone());
    let err_handle = spawn_reader(child.stderr.take(), stderr_overflow.clone());

    let kill_tree = |child: &mut std::process::Child, job: &mut Option<containment::JobGuard>| {
        let _ = child.kill();
        drop(job.take()); // kill-on-close job reaps the whole descendant tree
        let _ = child.wait();
    };

    let status = loop {
        if stdout_overflow.load(Ordering::SeqCst) || stderr_overflow.load(Ordering::SeqCst) {
            kill_tree(&mut child, &mut job);
            let _ = out_handle.join();
            let _ = err_handle.join();
            let stream = if stdout_overflow.load(Ordering::SeqCst) {
                "stdout"
            } else {
                "stderr"
            };
            return Err(format!(
                "{what}: {stream} exceeded the {} byte cap — job tree killed, output never silently truncated",
                STREAM_CAP
            ));
        }
        if started.elapsed() > wall {
            kill_tree(&mut child, &mut job);
            let _ = out_handle.join();
            let _ = err_handle.join();
            return Err(format!("{what} exceeded the {}s wall", wall.as_secs()));
        }
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => {
                kill_tree(&mut child, &mut job);
                let _ = out_handle.join();
                let _ = err_handle.join();
                return Err(format!("poll {what}: {e}"));
            }
        }
    };
    drop(job.take()); // normal exit: close the job, reaping any descendants
    let stdout = out_handle
        .join()
        .map_err(|_| format!("{what}: stdout reader join"))?;
    let stderr = err_handle
        .join()
        .map_err(|_| format!("{what}: stderr reader join"))?;
    // A reader may have latched overflow right as the child exited.
    if stdout_overflow.load(Ordering::SeqCst) || stderr_overflow.load(Ordering::SeqCst) {
        let stream = if stdout_overflow.load(Ordering::SeqCst) {
            "stdout"
        } else {
            "stderr"
        };
        return Err(format!(
            "{what}: {stream} exceeded the {} byte cap — job tree killed, output never silently truncated",
            STREAM_CAP
        ));
    }
    Ok(RunOutcome {
        success: status.success(),
        exit: status.code().unwrap_or(-1),
        stdout,
        stderr,
    })
}

/// Convenience wrapper used by the bin/tests: run certification and reduce to
/// an exit code (`0` all certified/reused, `1` any failure, `2` contract
/// error).
pub fn certify_and_report(args: &CertifyArgs) -> i32 {
    match certify(args) {
        Ok(summaries) => {
            let mut fails = 0;
            for s in &summaries {
                println!("{} {} — {}", s.outcome, s.case, s.detail);
                if s.outcome == "fail" {
                    fails += 1;
                }
            }
            let certified = summaries
                .iter()
                .filter(|s| s.outcome == "certified")
                .count();
            let reused = summaries.iter().filter(|s| s.outcome == "reused").count();
            println!(
                "ace-certify: {} certified, {} reused, {} failed",
                certified, reused, fails
            );
            if fails > 0 {
                1
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("ace-certify: {e}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_tmp(rustc: &str, name: &str, body: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ace-cert-stream-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("prog.rs");
        std::fs::write(&src, body).unwrap();
        let exe = dir.join(format!("{name}.exe"));
        compile(rustc, &src, &exe, name).expect("compile stream-governance probe");
        exe
    }

    /// F-852-R1-03 replays: stdout-only, stderr-FIRST 128KiB (the review's
    /// exact reproducer shape), interleaved bounded streams — all must
    /// complete WITHOUT the wall (the old sequential reader deadlocked the
    /// stderr-first shape until the 30s wall). Explicit-only entry: these
    /// compile and run real Rust.
    #[test]
    #[ignore = "PLAN-852 F-852-R1-02/03: compiles+runs real Rust — invoke explicitly with --ignored"]
    fn stream_governance_no_deadlock_on_any_pipe_order() {
        let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
        let stderr_first_128k = compile_tmp(
            &rustc,
            "stderr_first",
            r#"use std::io::Write;
fn main() {
    std::io::stderr().write_all(&vec![b'x'; 131072]).unwrap();
    println!("i32:14");
}
"#,
        );
        let started = Instant::now();
        let out = run_capped(&stderr_first_128k).expect("stderr-first 128KiB must not deadlock");
        assert_eq!(out.stdout, b"i32:14\n");
        assert_eq!(out.stderr.len(), 131072);
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "must finish well before the 30s wall (took {:?})",
            started.elapsed()
        );

        let stdout_only = compile_tmp(&rustc, "stdout_only", r#"fn main() { println!("i32:1"); }"#);
        let out = run_capped(&stdout_only).expect("stdout-only");
        assert_eq!(out.stdout, b"i32:1\n");

        let interleaved = compile_tmp(
            &rustc,
            "interleaved",
            r#"use std::io::Write;
fn main() {
    for i in 0..2048 {
        println!("out{}", i);
        let _ = std::io::stderr().write_all(format!("err{}\n", i).as_bytes());
    }
}
"#,
        );
        let out = run_capped(&interleaved).expect("interleaved bounded streams");
        assert!(out.stdout.len() > 10_000 && out.stderr.len() > 10_000);
    }

    /// F-852-R1-03: each stream overflowing its cap must KILL the tree and
    /// surface a parent-visible overflow error — never a silent truncate.
    #[test]
    #[ignore = "PLAN-852 F-852-R1-02/03: compiles+runs real Rust — invoke explicitly with --ignored"]
    fn stream_overflow_kills_and_surfaces() {
        let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
        for (name, body, stream) in [
            (
                "stdout_flood",
                r#"fn main() { for i in 0.. { println!("x{}", i); } }"#,
                "stdout",
            ),
            (
                "stderr_flood",
                r#"use std::io::Write;
fn main() { loop { std::io::stderr().write_all(&vec![b'y'; 4096]).unwrap(); } }"#,
                "stderr",
            ),
        ] {
            let exe = compile_tmp(&rustc, name, body);
            let started = Instant::now();
            let err = run_capped(&exe).expect_err("overflow must be an error");
            assert!(
                err.contains(stream) && err.contains("cap"),
                "{name}: unexpected error: {err}"
            );
            assert!(
                started.elapsed() < Duration::from_secs(20),
                "{name}: kill must be timely (took {:?})",
                started.elapsed()
            );
        }
    }

    /// The certify kill control (AC-07): a deliberately hanging binary is
    /// killed by the bounded run inside the run wall — the deadline, not a
    /// human, reaps the tree (Job containment closes on every exit path).
    #[test]
    #[ignore = "PLAN-852 F-852-R1-02: compiles+runs real Rust — invoke explicitly with --ignored"]
    fn wall_kills_a_hanging_binary() {
        let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
        let exe = compile_tmp(
            &rustc,
            "hang",
            "fn main() { loop { std::thread::sleep(std::time::Duration::from_millis(50)); } }\n",
        );
        let started = Instant::now();
        let err = run_capped(&exe).expect_err("the wall must fire");
        assert!(err.contains("wall"), "wall error: {err}");
        assert!(
            started.elapsed() < Duration::from_secs(35),
            "the kill must be timely (took {:?})",
            started.elapsed()
        );
    }

    /// F-852-R1-01: structurally invalid receipts are REJECTED — never
    /// reused, never silently overwritten. Pure-JSON unit faces (no spawn),
    /// replaying the reviewer's tamper variants field by field.
    #[test]
    fn invalid_receipts_are_rejected_in_memory() {
        let axes = CurrentAxes {
            source_sha: "a".repeat(64),
            golden_sha: "b".repeat(64),
            wrapper_sha: "c".repeat(64),
            oracle_sha: "d".repeat(64),
            reference_sha: "e".repeat(64),
        };
        let oracle = "i32:14\n";
        let tc = "rustc test identity";
        let base = serde_json::json!({
            "schema": CERT_SCHEMA,
            "case": "B01",
            "module": "m1_b01",
            "verdict": "certified",
            "expected_oracle": oracle,
            "identity": {
                "source_sha256": "a".repeat(64),
                "golden_sha256": "b".repeat(64),
                "actual_sha256": "b".repeat(64),
                "wrapper_sha256": "c".repeat(64),
                "oracle_sha256": "d".repeat(64),
                "reference_sha256": "e".repeat(64),
            },
            "toolchain": { "identity": tc, "flags": RUSTC_FLAG_TOKENS },
            "deps": { "external": [], "closure": DEPS_CLOSURE },
            "observed": {
                "actual_run": { "exit": 0, "stdout": oracle },
                "reference_run": { "exit": 0, "stdout": oracle },
            },
        });
        assert!(matches!(
            validate_receipt(&base, "B01", oracle, &axes, tc),
            Ok(())
        ));
        for (field, value) in [
            ("schema", serde_json::json!("invalid/999")),
            ("verdict", serde_json::json!("maybe")),
            ("expected_oracle", serde_json::json!("i32:999\n")),
        ] {
            let mut v = base.clone();
            v[field] = value;
            assert!(
                matches!(
                    validate_receipt(&v, "B01", oracle, &axes, tc),
                    Err(ReceiptProblem::Invalid(_))
                ),
                "{field} tamper must be Invalid"
            );
        }
        let mut v = base.clone();
        v["toolchain"]["flags"] = serde_json::json!(["--wrong-flag"]);
        assert!(matches!(
            validate_receipt(&v, "B01", oracle, &axes, tc),
            Err(ReceiptProblem::Invalid(_))
        ));
        let mut v = base.clone();
        v["deps"]["closure"] = serde_json::json!("changed-dependency");
        assert!(matches!(
            validate_receipt(&v, "B01", oracle, &axes, tc),
            Err(ReceiptProblem::Invalid(_))
        ));
        let mut v = base.clone();
        v["observed"]["actual_run"]["stdout"] = serde_json::json!("i32:999");
        assert!(matches!(
            validate_receipt(&v, "B01", oracle, &axes, tc),
            Err(ReceiptProblem::Invalid(_))
        ));
        let mut v = base.clone();
        v["identity"]["actual_sha256"] = serde_json::json!("0".repeat(64));
        assert!(matches!(
            validate_receipt(&v, "B01", oracle, &axes, tc),
            Err(ReceiptProblem::Invalid(_))
        ));
        // File-axis drift is STALE (the honest re-certify path), not Invalid.
        let mut v = base.clone();
        v["identity"]["golden_sha256"] = serde_json::json!("f".repeat(64));
        v["identity"]["actual_sha256"] = serde_json::json!("f".repeat(64));
        assert!(matches!(
            validate_receipt(&v, "B01", oracle, &axes, tc),
            Err(ReceiptProblem::Stale(_))
        ));
        let mut v = base.clone();
        v["toolchain"]["identity"] = serde_json::json!("rustc other identity");
        assert!(matches!(
            validate_receipt(&v, "B01", oracle, &axes, tc),
            Err(ReceiptProblem::Stale(_))
        ));
        // Wrapper hash comes from the CURRENT frozen wrapper file.
        let live = hex(&sha256(WRAPPER_MAIN_RS.as_bytes()));
        let mut v = base.clone();
        v["identity"]["wrapper_sha256"] = serde_json::json!(live);
        assert!(matches!(
            validate_receipt(&v, "B01", oracle, &axes, tc),
            Err(ReceiptProblem::Stale(_))
        ));
    }
}
