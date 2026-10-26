//! PLAN-852 T-04: the EXPLICIT independent a2r certification entry
//! (`ace-certify`). One certification, daily snapshot (T-03) — this module
//! is the only place that compiles and runs Rust for the ace-m1 corpus, and
//! it is never invoked by the suite runner or any daily scope.
//!
//! Contract (frozen in docs/plans/attachments/852/contract-decision.md §3):
//! 1. Emit the ACTUAL Rust fresh from the corpus source through the
//!    production transpiler (never read the golden as the actual).
//! 2. Require actual bytes == frozen golden bytes (a drift FAILs; goldens
//!    are only updated through explicit review — G04).
//! 3. Compile AND run (a) the actual bytes under the frozen host wrapper
//!    (byte-identical `generated.rs` + bridging `mod`, hashed identity) and
//!    (b) the hand-written `reference/<id>.rs`; BOTH must print the fixed
//!    behavior oracle `i32:<decimal>\n` exactly.
//! 4. Persist a version-controlled certificate (schema `ace.m1.cert/1`)
//!    binding every identity axis: source, golden/actual bytes, wrapper,
//!    oracle text, reference file, rustc identity + effective flags, the
//!    dependency closure (std-only for the scalar set), and both observed
//!    runs. Transpiler identity is audit-only — identical output under an
//!    unchanged context REUSES the certificate (G03/G08).
//!
//! Contained-tool governance (contract-decision §3 budgets): rustc and the
//! certified binaries run under Windows Job-Object containment with a 120s
//! compile wall / 30s run wall / 1MB stream caps, and the whole scratch
//! tree is removed on every exit path.

use crate::test_cases::sema::containment;
use auto_test_case::sha256::{hex, sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const CERT_SCHEMA: &str = "ace.m1.cert/1";
pub const COMPILE_WALL: Duration = Duration::from_secs(120);
pub const RUN_WALL: Duration = Duration::from_secs(30);
pub const STREAM_CAP: usize = 1 << 20;
/// Emit needs more than a default thread stack (mirrors the a2r adapter).
const EMIT_STACK_BYTES: usize = 128 * 1024 * 1024;

/// The frozen certification wrapper (contract-decision §3). Its full text is
/// an identity axis of every certificate; changing it re-certifies everything.
pub const WRAPPER_MAIN_RS: &str = concat!(
    "mod generated {\n",
    "    include!(\"generated.rs\");\n",
    "    pub fn entry() -> i64 { main() }\n",
    "}\n",
    "fn main() {\n",
    "    let observed = generated::entry() as i32;\n",
    "    println!(\"i32:{}\", observed);\n",
    "}\n"
);

/// The frozen effective flags handed to rustc (identity axis).
pub const RUSTC_FLAGS: [&str; 4] = ["--edition", "2021", "-O", "--crate-name"];

pub struct CertifyArgs {
    pub corpus: PathBuf,
    pub certs_dir: PathBuf,
    pub only: Vec<String>,
    pub force: bool,
    pub rustc: String,
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
            return Err(format!("binding row has {} columns (frozen schema = 12)", f.len()));
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
        .filter(|(id, _)| args.only.is_empty() || args.only.iter().any(|o| o.eq_ignore_ascii_case(id)))
        .collect();
    if selected.is_empty() {
        return Err("--only matched no certifiable case".to_string());
    }
    std::fs::create_dir_all(&args.certs_dir).map_err(|e| format!("certs dir: {e}"))?;

    let scratch_root = std::env::temp_dir().join(format!("ace-m1-certify-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch_root);
    std::fs::create_dir_all(&scratch_root).map_err(|e| format!("scratch: {e}"))?;
    let result = (|| {
        let mut summaries = Vec::new();
        for (id, expected) in selected {
            let summary = certify_one(&id, expected, args, &scratch_root);
            summaries.push(summary);
        }
        Ok(summaries)
    })();
    let _ = std::fs::remove_dir_all(&scratch_root);
    result
}

#[allow(clippy::too_many_lines)]
fn certify_one(
    id: &str,
    expected: &str,
    args: &CertifyArgs,
    scratch_root: &Path,
) -> CaseSummary {
    let module = format!("m1_{}", id.to_lowercase());
    let oracle_text = format!("i32:{}\n", expected);
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
                    &source_dir, &module, &source,
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
            return fail(format!("emit thread panicked: {}", super::avm::panic_message(&payload)))
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
    let golden_sha = hex(&sha256(&golden));
    let wrapper_sha = hex(&sha256(WRAPPER_MAIN_RS.as_bytes()));
    let oracle_sha = hex(&sha256(oracle_text.as_bytes()));
    let source_sha = hex(&sha256(source.as_bytes()));
    let reference_path = args.corpus.join("reference").join(format!("{id}.rs"));
    let reference = match std::fs::read(&reference_path) {
        Ok(r) => r,
        Err(e) => return fail(format!("read reference: {e}")),
    };
    let reference_sha = hex(&sha256(&reference));

    let rustc_identity = match rustc_version(&args.rustc) {
        Ok(v) => v,
        Err(e) => return fail(format!("rustc identity: {e}")),
    };

    // ---- reuse (G03/G08): identical axes -> reuse, spawn nothing -----------
    let cert_path = args.certs_dir.join(format!("{module}.cert.json"));
    if !args.force {
        if let Ok(old) = std::fs::read_to_string(&cert_path) {
            if let Ok(old) = serde_json::from_str::<serde_json::Value>(&old) {
                let axes_match = old.get("identity").and_then(|i| {
                    Some(
                        i.get("golden_sha256")?.as_str()? == golden_sha
                            && i.get("wrapper_sha256")?.as_str()? == wrapper_sha
                            && i.get("oracle_sha256")?.as_str()? == oracle_sha
                            && i.get("reference_sha256")?.as_str()? == reference_sha
                            && i.get("source_sha256")?.as_str()? == source_sha
                            && old.get("toolchain")?.get("identity")?.as_str()? == rustc_identity
                    )
                }) == Some(true)
                    && old.get("verdict").and_then(|v| v.as_str()) == Some("certified");
                if axes_match {
                    return CaseSummary {
                        case: id.to_string(),
                        outcome: "reused".to_string(),
                        detail: "identity axes unchanged — certificate reused, nothing compiled (G03/G08)".to_string(),
                    };
                }
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
        &args.rustc,
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
    if let Err(e) = compile(&args.rustc, &reference_path, &ref_exe, &crate_name) {
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
        "expected_oracle": oracle_text,
        "identity": {
            "source_sha256": source_sha,
            "golden_sha256": golden_sha,
            "actual_sha256": golden_sha,
            "wrapper_sha256": wrapper_sha,
            "oracle_sha256": oracle_sha,
            "reference_sha256": reference_sha,
        },
        "toolchain": {
            "identity": rustc_identity,
            "flags": RUSTC_FLAGS,
            "target_note": "host default (recorded verbatim in identity)",
        },
        "deps": { "external": [], "closure": "std-only" },
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
            detail: format!("actual==golden; wrapper+reference both printed the fixed oracle; cert {module}.cert.json"),
        },
        Err(e) => fail(format!("write certificate: {e}")),
    }
}

fn rustc_version(rustc: &str) -> Result<String, String> {
    let out = Command::new(rustc)
        .args(["--version"])
        .output()
        .map_err(|e| format!("spawn rustc --version: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        return Err("empty rustc identity".to_string());
    }
    Ok(text)
}

/// Compile one file under containment with the frozen flags and wall.
fn compile(rustc: &str, input: &Path, out_exe: &Path, crate_name: &str) -> Result<(), String> {
    let mut cmd = Command::new(rustc);
    cmd.args([
        "--edition",
        "2021",
        "-O",
        "--crate-name",
        crate_name,
        input.as_os_str().to_string_lossy().as_ref(),
        "-o",
        out_exe.as_os_str().to_string_lossy().as_ref(),
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    run_contained(cmd, COMPILE_WALL, "rustc compile")
}

pub struct RunOutcome {
    pub success: bool,
    pub exit: i32,
    pub stdout: Vec<u8>,
}

/// Run a certified binary under containment with the run wall and caps.
fn run_capped(exe: &Path) -> Result<RunOutcome, String> {
    let mut cmd = Command::new(exe);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let started = Instant::now();
    let mut child = cmd.spawn().map_err(|e| format!("spawn {}: {e}", exe.display()))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "containment unavailable: refusing an uncontained run of {}",
                exe.display()
            ));
        }
    };
    #[cfg(windows)]
    containment::resume_suspended_process(&child).map_err(|e| format!("resume {}: {e}", exe.display()))?;
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;

    let mut out_pipe = child.stdout.take();
    let mut err_pipe = child.stderr.take();
    let reader = std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut out = Vec::new();
        let mut err = Vec::new();
        loop {
            let mut progressed = false;
            if let Some(o) = out_pipe.as_mut() {
                match o.read(&mut buf) {
                    Ok(0) => out_pipe = None,
                    Ok(n) => {
                        out.extend_from_slice(&buf[..n]);
                        progressed = true;
                        if out.len() > STREAM_CAP { break; }
                    }
                    Err(_) => out_pipe = None,
                }
            }
            if let Some(e) = err_pipe.as_mut() {
                match e.read(&mut buf) {
                    Ok(0) => err_pipe = None,
                    Ok(n) => {
                        err.extend_from_slice(&buf[..n]);
                        progressed = true;
                        if err.len() > STREAM_CAP { break; }
                    }
                    Err(_) => err_pipe = None,
                }
            }
            if out_pipe.is_none() && err_pipe.is_none() { break; }
            if !progressed { std::thread::sleep(Duration::from_millis(5)); }
        }
        (out, err)
    });
    let status = loop {
        if started.elapsed() > RUN_WALL {
            child.kill();
            drop(job.take());
            let _ = child.wait();
            let _ = reader.join();
            return Err(format!("run exceeded the {}s wall", RUN_WALL.as_secs()));
        }
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => { drop(job.take()); return Err(format!("poll run: {e}")); }
        }
    };
    drop(job.take()); // close the job: reaps any descendant tree
    let (out, _err) = reader.join().map_err(|_| "reader join".to_string())?;
    let mut stdout = out;
    if stdout.len() > STREAM_CAP { stdout.truncate(STREAM_CAP); }
    Ok(RunOutcome { success: status.success(), exit: status.code().unwrap_or(-1), stdout })
}

/// Spawn a command under the Windows job containment and wait inside the
/// compile wall (kill the tree on overrun). Pipes are drained capped.
fn run_contained(mut cmd: Command, wall: Duration, what: &str) -> Result<(), String> {
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
            return Err(format!("containment unavailable: refusing an uncontained {what}"));
        }
    };
    #[cfg(windows)]
    containment::resume_suspended_process(&child).map_err(|e| format!("resume {what}: {e}"))?;
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;

    let mut out_pipe = child.stdout.take();
    let mut err_pipe = child.stderr.take();
    let reader = std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        let mut err = Vec::new();
        loop {
            let mut progressed = false;
            if let Some(o) = out_pipe.as_mut() {
                match o.read(&mut buf) {
                    Ok(0) => out_pipe = None,
                    Ok(_) => { progressed = true; }
                    Err(_) => out_pipe = None,
                }
            }
            if let Some(e) = err_pipe.as_mut() {
                match e.read(&mut buf) {
                    Ok(0) => err_pipe = None,
                    Ok(n) => {
                        err.extend_from_slice(&buf[..n]);
                        progressed = true;
                        if err.len() > STREAM_CAP { break; }
                    }
                    Err(_) => err_pipe = None,
                }
            }
            if out_pipe.is_none() && err_pipe.is_none() { break; }
            if !progressed { std::thread::sleep(Duration::from_millis(5)); }
        }
        err
    });
    let status = loop {
        if started.elapsed() > wall {
            child.kill();
            drop(job.take());
            let _ = child.wait();
            let _ = reader.join();
            return Err(format!("{what} exceeded the {}s wall", wall.as_secs()));
        }
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => { drop(job.take()); return Err(format!("poll {what}: {e}")); }
        }
    };
    drop(job.take());
    let err = reader.join().unwrap_or_default();
    if !status.success() {
        let tail = String::from_utf8_lossy(&err[..err.len().min(2000)]).to_string();
        return Err(format!("{what} exited {status:?}; stderr tail: {tail}"));
    }
    Ok(())
}

/// Convenience wrapper used by the bin/tests: run certification and reduce to
/// an exit code (`0` all certified/reused, `1` any failure).
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
            let certified = summaries.iter().filter(|s| s.outcome == "certified").count();
            let reused = summaries.iter().filter(|s| s.outcome == "reused").count();
            println!(
                "ace-certify: {} certified, {} reused, {} failed",
                certified,
                reused,
                fails
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
