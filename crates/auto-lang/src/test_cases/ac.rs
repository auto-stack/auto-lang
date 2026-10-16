//! `ac` adapter executor (PLAN-788 T-04): the parent-side half of the
//! native test path. auto-lang NEVER links the experimental compiler —
//! this module spawns the `ac-case-worker` process (env override
//! `AC_CASE_WORKER`, else located from the repo layout and built release
//! once per process) and JUDGES its observations against the case oracle.
//!
//! All comparisons live HERE (worker PASS is never trusted): typed stage/
//! code/message on error completions, native exit code, raw stream bytes,
//! canonical-HIR byte goldens, and the optional declared perf assertion.
//! Products (hir/obj/exe) are persisted under the check's artifact dir
//! with parent-verified manifest hashes and register for `product_from`
//! handoff; an `ac/run` consumer of an upstream build RUNS the consumed
//! exe (never recompiles source to replace a bad product).

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use auto_test_case::compare::{compare_bytes, Mismatch};
use auto_test_case::contract::{BudgetSpec, CheckSpec, Completion, MessageMatcher};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{ArtifactRef, NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};

use super::avm::Verdict;

/// Locate (and on first use build) the worker binary. Env override first;
/// otherwise find `experimental/ac-case-worker` from this crate's source
/// root and `cargo build --release` it (a warm no-op rebuild is ~1s).
pub fn worker_binary() -> Result<PathBuf, String> {
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| locate_worker().map_err(|e| e)).clone()
}

fn locate_worker() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("AC_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "AC_CASE_WORKER points at a missing file: {}",
            env_path
        ));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "ac-case-worker not located: no experimental/ac-case-worker above the crate and no AC_CASE_WORKER override".to_string(),
            );
        };
        let m = d
            .join("experimental")
            .join("ac-case-worker")
            .join("Cargo.toml");
        if m.is_file() {
            break m;
        }
        dir = d.parent();
    };
    let status = Command::new("cargo")
        .args(["build", "--release", "--manifest-path"])
        .arg(&manifest)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("spawn cargo for worker build: {}", e))?;
    if !status.status.success() {
        let tail = String::from_utf8_lossy(&status.stderr);
        let tail: String = tail
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!("ac-case-worker build failed:\n{}", tail));
    }
    let exe = manifest
        .parent()
        .unwrap()
        .join("target")
        .join("release")
        .join("ac-case-worker.exe");
    if exe.is_file() {
        Ok(exe)
    } else {
        Err(format!(
            "worker build produced no binary at {}",
            exe.display()
        ))
    }
}

struct WorkerIpc {
    nonce: String,
    request_hash: String,
}

/// One single-shot worker session: begin → request → end. The parent
/// enforces its OWN wall deadline on top of the worker's self-limits —
/// on expiry the worker process is killed (its Job-contained children die
/// with it), which surfaces as an infrastructure error, never a hang.
fn call_worker(
    request_json: &str,
    nonce: &str,
    request_hash: &str,
    kind: &str,
    deadline: Duration,
) -> Result<serde_json::Value, String> {
    let exe = worker_binary()?;
    let started = Instant::now();
    let mut child = Command::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn ac-case-worker ({}): {}", exe.display(), e))?;
    {
        let mut stdin = child.stdin.take().expect("piped stdin");
        let _ = writeln!(stdin, r#"{{"op":"begin","schema":"acworker/1"}}"#);
        let _ = writeln!(stdin, r#"{{"op":"request","request":{} }}"#, request_json);
        let _ = writeln!(stdin, r#"{{"op":"end"}}"#);
    }
    // R1 (F03): stdout is read with a HARD bound (the IPC metadata cap plus
    // slack) — an oversized or runaway response is a controlled failure,
    // not unbounded buffering; stderr is drained CONCURRENTLY so a chatty
    // worker can never wedge on a full pipe.
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");
    const READ_CAP: usize = 8 << 20; // 2× the frozen IPC metadata cap
    let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 65536];
        let mut exceeded = false;
        loop {
            match stdout_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() > READ_CAP {
                        exceeded = true;
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = tx.send(if exceeded { Err(buf.len()) } else { Ok(buf) });
    });
    let err_reader = std::thread::spawn(move || {
        // R2 (F05): the stderr drain is BOUNDED — a chatty worker cannot
        // wedge the parent on an unbounded allocation either.
        const STDERR_CAP: usize = 1 << 20;
        let mut sink = Vec::new();
        let mut chunk = [0u8; 65536];
        loop {
            match stderr_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    sink.extend_from_slice(&chunk[..n]);
                    if sink.len() > STDERR_CAP {
                        sink.truncate(STDERR_CAP);
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        sink
    });
    let response = match rx.recv_timeout(deadline.saturating_sub(started.elapsed())) {
        Ok(Ok(bytes)) => String::from_utf8_lossy(&bytes).to_string(),
        Ok(Err(observed)) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = out_reader.join();
            let _ = err_reader.join();
            return Err(format!(
                "worker response exceeded the {} byte IPC bound (observed {}); the worker was killed",
                READ_CAP, observed
            ));
        }
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = out_reader.join();
            let _ = err_reader.join();
            return Err(format!(
                "ac-case-worker did not answer within {:?}; the worker tree was killed",
                deadline
            ));
        }
    };
    let _ = out_reader.join();
    // R2 (F05): the SAME absolute deadline bounds the post-response wait —
    // a worker that answered and then hung (or slept past its budget) is
    // killed at the deadline, never accepted late. The worker's leaked
    // self-containment job reaps its whole tree on that kill.
    let wait_left = deadline.saturating_sub(started.elapsed());
    let wait_deadline = std::time::Instant::now() + wait_left;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if std::time::Instant::now() >= wait_deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = err_reader.join();
                    return Err(format!(
                        "ac-case-worker outlived its {:?} budget after answering; the tree was killed",
                        deadline
                    ));
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(e) => {
                let _ = err_reader.join();
                return Err(format!("wait ac-case-worker: {}", e));
            }
        }
    };
    let _ = err_reader.join();
    if !status.success() {
        return Err(format!(
            "ac-case-worker exited abnormally ({:?}); no response is trusted from a dead worker",
            status.code()
        ));
    }
    // R2 (F05): metadata budget — everything EXCEPT the declared byte
    // payloads is bounded by the frozen 4MiB IPC metadata cap.
    {
        let parsed: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| format!("worker response undecodable (pre-line): {}", e))?;
        let payload_len: usize = ["stdout_b64", "stderr_b64", "hir_b64"]
            .iter()
            .filter_map(|k| parsed["ok"]["run"][k].as_str().map(|s| s.len()))
            .chain(parsed["ok"]["hir_b64"].as_str().map(|s| s.len()))
            .sum();
        let metadata = response.len().saturating_sub(payload_len);
        if metadata > 4 << 20 {
            return Err(format!(
                "worker response metadata {} bytes exceeds the 4MiB IPC metadata cap",
                metadata
            ));
        }
    }
    let line = response
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .ok_or_else(|| "ac-case-worker produced no response line".to_string())?;
    let v: serde_json::Value =
        serde_json::from_str(line).map_err(|e| format!("worker response undecodable: {}", e))?;
    validate_response(&v, nonce, request_hash, kind)
}

/// R1 (F03): FULL identity + shape admission over a parsed response. The
/// nonce/hash echo binds the response to THIS request; the schema,
/// ok/error exclusivity and the action's required result shape are checked
/// parent-side — the worker's own admission is never trusted. Extracted so
/// the admission table itself is unit-testable without a process.
pub(crate) fn validate_response(
    v: &serde_json::Value,
    nonce: &str,
    request_hash: &str,
    kind: &str,
) -> Result<serde_json::Value, String> {
    let got_schema = v["schema"].as_str().unwrap_or("");
    if got_schema != "acworker/1" {
        return Err(format!(
            "worker response schema {:?} is not acworker/1 — a foreign or stale protocol is never accepted",
            got_schema
        ));
    }
    let got_nonce = v["nonce"].as_str().unwrap_or("");
    let got_hash = v["request_hash"].as_str().unwrap_or("");
    if got_nonce != nonce || got_hash != request_hash {
        return Err(format!(
            "worker response identity mismatch (nonce {got_nonce:?}/hash {got_hash:?} — a stale or foreign response is never trusted)"
        ));
    }
    let has_ok = v.get("ok").map(|o| !o.is_null()).unwrap_or(false);
    let has_err = v.get("error").map(|o| !o.is_null()).unwrap_or(false);
    if has_ok == has_err {
        return Err(format!(
            "worker response must carry exactly one of ok/error (ok={has_ok}, error={has_err})"
        ));
    }
    if has_ok && kind == "run" && v["ok"]["run"].is_null() {
        // R2 F04: a LANGUAGE failure (ok.diagnostics non-empty) legally
        // carries no run observation; only a diagnostics-free ok must.
        let diags = v["ok"]["diagnostics"]
            .as_array()
            .map(|d| !d.is_empty())
            .unwrap_or(false);
        if !diags {
            return Err(format!(
                "run request answered without a run observation — the response shape is refused"
            ));
        }
    }
    // R2 F06: typed admission — the whole response must deserialize into
    // the versioned protocol types (deny_unknown_fields + field types +
    // enums). Malformed shapes (e.g. run as an integer) are refused here.
    let typed: crate::test_cases::ac_protocol::WorkerResponse =
        serde_json::from_value(v.clone())
            .map_err(|e| format!("response is not a typed acworker/1 value: {}", e))?;
    if let Some(ok) = &typed.ok {
        for d in &ok.diagnostics {
            if d.code.is_empty() || d.message.is_empty() {
                return Err("typed diagnostics must carry code and message".to_string());
            }
        }
        // Strict stream encodings even when nothing compares them.
        if let Some(run) = &ok.run {
            for (name, raw) in [("stdout", &run.stdout_b64), ("stderr", &run.stderr_b64)] {
                b64d_strict(raw).map_err(|e| format!("{}: {}", name, e))?;
            }
        }
    }
    Ok(v.clone())
}

/// Run one `ac/check|build|run` check. `Err` = infrastructure failure
/// before/around the observation; language diagnostics are observations
/// judged against the oracle (a negative case PASSes on a matching typed
/// rejection).
pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "ac adapter needs the resolved entry input".to_string())?;
    let scratch = artifact_dir.join("work");
    std::fs::create_dir_all(&scratch).map_err(|e| format!("worker scratch: {}", e))?;

    let budget = check.options.effective_budget();
    let budget = effective_budget(&budget);
    let (build_ms, run_ms, total_ms) = (
        budget.build_ms.unwrap_or(60_000),
        budget.run_ms.unwrap_or(5_000),
        budget.total_ms.unwrap_or(70_000),
    );
    let nonce = hex(&sha256(
        format!(
            "{}|{:?}|{}|{:?}",
            std::process::id(),
            started,
            check.id,
            oracle.oracle_sha256
        )
        .as_bytes(),
    ));
    let input_sha = hex(&sha256(&input_bytes));
    let request_hash = hex(&sha256(
        format!(
            "acworker/1|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            check.action.as_str(),
            input_sha,
            check.options.effective_profile().as_str(),
            check.options.effective_entry(),
            build_ms,
            run_ms,
            total_ms,
            scratch.display(),
            check.options.effective_cache().as_str(),
            run_dir.join("cache-store").display().to_string()
        )
        .as_bytes(),
    ));
    let input = if case.spec.source.syntax == auto_test_case::contract::Syntax::HirAtom {
        serde_json::json!({ "kind": "hir", "hir_b64": b64(&input_bytes) })
    } else {
        serde_json::json!({ "kind": "source", "source_b64": b64(&input_bytes) })
    };
    let request = serde_json::json!({
        "schema": "acworker/1",
        "kind": check.action.as_str(),
        "nonce": nonce,
        "request_hash": request_hash,
        "input": input,
        "profile": check.options.effective_profile().as_str(),
        "entry": check.options.effective_entry(),
        "budget": {
            "build_ms": budget.build_ms,
            "run_ms": budget.run_ms,
            "total_ms": budget.total_ms,
        },
        // Cache participation (T-05): one store per run by default
        // (cross-check reuse inside the run); AC_CASE_STORE pins a
        // persistent root for the restart/disk-hot scenarios. The cache
        // mode travels from the check's own options; a hir.atom golden
        // asks the worker for the canonical bytes.
        "store_root": if check.adapter == auto_test_case::contract::Adapter::Syntax {
            serde_json::Value::Null
        } else {
            serde_json::json!(std::env::var("AC_CASE_STORE").ok().unwrap_or_else(|| {
                run_dir.join("cache-store").display().to_string()
            }))
        },
        "grammar": if check.adapter == auto_test_case::contract::Adapter::Syntax {
            serde_json::json!(check.options.effective_grammar())
        } else {
            serde_json::Value::Null
        },
        "capabilities": check.options.effective_capabilities(),
        "support_lib": match check.options.effective_support_lib() {
            Some("trace-marks") => serde_json::json!(trace_marks_lib()
                .map_err(|e| e)?
                .display()
                .to_string()),
            _ => serde_json::Value::Null,
        },
        "edits": match (&check.options.syntax_edits, check.action) {
            (Some(steps), auto_test_case::contract::Action::Edit) => serde_json::json!(steps
                .iter()
                .map(|s| serde_json::json!({
                    "start": s.start,
                    "end": s.end,
                    "insert_b64": b64(s.insert.as_bytes()),
                }))
                .collect::<Vec<_>>()),
            _ => serde_json::Value::Null,
        },
        "cache": check.options.effective_cache().as_str(),
        "project": "ac-case",
        "include_hir": !oracle.artifacts.is_empty(),
        "scratch": scratch.display().to_string(),
    });
    let ipc = WorkerIpc {
        nonce: nonce.clone(),
        request_hash: request_hash.clone(),
    };
    // R1 (F05): the parent wall clock IS the request total budget — no
    // grace margin may smuggle extra time past a declared deadline; spawn
    // and IPC happen inside it.
    let wall = Duration::from_millis(total_ms);
    let response = call_worker(
        &request.to_string(),
        &ipc.nonce,
        &ipc.request_hash,
        check.action.as_str(),
        wall,
    )?;

    if let Some(err) = response.get("error") {
        let code = err["code"].as_str().unwrap_or("?").to_string();
        let message = err["message"].as_str().unwrap_or("").to_string();
        return Ok(Verdict::error(format!(
            "ac-case-worker infrastructure failure [{}]: {}",
            code, message
        )));
    }
    let ok = response
        .get("ok")
        .ok_or_else(|| "worker response carries neither ok nor error".to_string())?;

    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut native = NativeRecord::default();

    // Diagnostics vs completion oracle.
    let diags = ok["diagnostics"].as_array().cloned().unwrap_or_default();
    match &oracle.completion {
        Completion::Success => {
            if !diags.is_empty() {
                let d = &diags[0];
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "diagnostics",
                    expected: "success".into(),
                    actual: format!(
                        "error [{}/{}]: {}",
                        d["stage"].as_str().unwrap_or("?"),
                        d["code"].as_str().unwrap_or("?"),
                        d["message"].as_str().unwrap_or("")
                    ),
                    first_diff: None,
                });
            }
        }
        Completion::Error {
            stage,
            code,
            message,
        } => {
            if diags.is_empty() {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "diagnostics",
                    expected: format!(
                        "error [{}]{}",
                        stage.as_str(),
                        code.as_deref()
                            .map(|c| format!(" code {}", c))
                            .unwrap_or_default()
                    ),
                    actual: "success (no diagnostics)".into(),
                    first_diff: None,
                });
            } else {
                let d = &diags[0];
                let got_stage = d["stage"].as_str().unwrap_or("?");
                let got_code = d["code"].as_str().unwrap_or("?");
                if got_stage != stage.as_str() {
                    mismatches.push(Mismatch {
                        observation: "completion.stage".into(),
                        kind: "stage",
                        expected: stage.as_str().into(),
                        actual: got_stage.into(),
                        first_diff: None,
                    });
                }
                if let Some(want) = code {
                    if got_code != want {
                        mismatches.push(Mismatch {
                            observation: "completion.code".into(),
                            kind: "code",
                            expected: want.clone(),
                            actual: got_code.into(),
                            first_diff: None,
                        });
                    }
                }
                if let Some(matcher) = message {
                    let got = d["message"].as_str().unwrap_or("");
                    let hit = match matcher {
                        MessageMatcher::Exact(t) => got == t,
                        MessageMatcher::Contains(t) => got.contains(t.as_str()),
                    };
                    if !hit {
                        mismatches.push(Mismatch {
                            observation: "completion.message".into(),
                            kind: "message",
                            expected: format!("{:?}", matcher),
                            actual: escape_tail(got),
                            first_diff: None,
                        });
                    }
                }
            }
        }
    }

    // Artifacts: FULL parent-side manifest admission (R1 F04) before any
    // byte is buffered: fixed kind/logical sets, uniqueness, in-scratch
    // location with an ancestor reparse audit, declared-length match via
    // metadata, and the single/total caps — then read, hash-verify and
    // persist. Oracle byte-comparison happens only over admitted products.
    let mut artifacts: Vec<ArtifactRef> = Vec::new();
    let raw_artifacts = ok["artifacts"].as_array().cloned().unwrap_or_default();
    let mut seen_logical = std::collections::BTreeSet::new();
    let mut declared_total = 0u64;
    for a in &raw_artifacts {
        let logical = a["logical"].as_str().unwrap_or("");
        let kind = a["kind"].as_str().unwrap_or("");
        let declared_bytes = a["bytes"].as_u64().unwrap_or(u64::MAX);
        let path_text = a["path"].as_str().unwrap_or("");
        let legal: &[(&str, &str)] = if check.adapter == auto_test_case::contract::Adapter::Syntax {
            &[
                ("syntax", "reconstructed.at"),
                ("syntax", "outline.json"),
                ("syntax", "cst.canonical"),
            ]
        } else {
            &[("hir", "hir.atom"), ("obj", "app.obj"), ("exe", "app.exe")]
        };
        if !legal.iter().any(|(k, l)| *k == kind && *l == logical) {
            return Ok(Verdict::error(format!(
                "worker artifact {:?}/{} is outside the fixed manifest set",
                kind, logical
            )));
        }
        if !seen_logical.insert(logical.to_string()) {
            return Ok(Verdict::error(format!(
                "worker manifest declares {:?} twice",
                logical
            )));
        }
        if declared_bytes > (64 << 20) as u64 {
            return Ok(Verdict::error(format!(
                "worker artifact {:?} declares {} bytes — over the 64MiB single-artifact cap",
                logical, declared_bytes
            )));
        }
        declared_total += declared_bytes;
        if declared_total > (128 << 20) as u64 {
            return Ok(Verdict::error(format!(
                "check artifact total {} exceeds the 128MiB cap (declared, pre-read)",
                declared_total
            )));
        }
        let path = Path::new(path_text);
        // In-scratch + reparse-clean BEFORE reading a byte.
        let scratch_canon =
            std::fs::canonicalize(&scratch).map_err(|e| format!("canonicalize scratch: {}", e))?;
        let path_canon = std::fs::canonicalize(path)
            .map_err(|e| format!("worker artifact {:?} path unusable: {}", logical, e))?;
        if !path_canon.starts_with(&scratch_canon) {
            return Ok(Verdict::error(format!(
                "worker artifact {:?} path {:?} is outside this request's scratch — never accepted",
                logical, path_text
            )));
        }
        if path.file_name().and_then(|f| f.to_str()) != Some(logical) {
            return Ok(Verdict::error(format!(
                "worker artifact {:?} path file name does not match its logical identity",
                logical
            )));
        }
        let meta = std::fs::symlink_metadata(&path_canon)
            .map_err(|e| format!("worker artifact {:?}: {}", logical, e))?;
        if meta.file_type().is_symlink() {
            return Ok(Verdict::error(format!(
                "worker artifact {:?} is a reparse point",
                logical
            )));
        }
        if meta.len() != declared_bytes {
            return Ok(Verdict::error(format!(
                "worker artifact {:?} declares {} bytes but the file is {} — manifest mismatch",
                logical,
                declared_bytes,
                meta.len()
            )));
        }
    }
    // R2 F04: a LANGUAGE failure legally carries NO products — the
    // required-set admission applies only to diagnostics-free successes
    // (a failure with forged success products is refused below).
    let diags = ok["diagnostics"]
        .as_array()
        .map(|d| !d.is_empty())
        .unwrap_or(false);
    let artifacts_declared = ok["artifacts"]
        .as_array()
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    let is_ac = check.adapter == auto_test_case::contract::Adapter::Ac;
    if is_ac && diags && artifacts_declared {
        // AC success artifacts (hir/obj/exe) on a language failure are
        // forged products. SYNTAX failures legitimately carry their
        // projections (reconstruction is lossless even for broken trees).
        return Ok(Verdict::error(
            "language failure carrying success artifacts — refused".to_string(),
        ));
    }
    if !diags {
        if let Err(e) = require_products(check, &seen_logical) {
            return Ok(Verdict::error(e));
        }
    }
    let has = |l: &str| seen_logical.contains(l);
    let _ = has;
    let is_syntax = check.adapter == auto_test_case::contract::Adapter::Syntax;
    let _ = is_syntax;
    for a in &raw_artifacts {
        let logical = a["logical"].as_str().unwrap_or("");
        let path_text = a["path"].as_str().unwrap_or("");
        let want_sha = a["sha256"].as_str().unwrap_or("");
        let bytes = std::fs::read(path_text)
            .map_err(|e| format!("read worker artifact {}: {}", logical, e))?;
        let got_sha = hex(&sha256(&bytes));
        if got_sha != want_sha {
            return Ok(Verdict::error(format!(
                "worker manifest mismatch for {}: hashes {} but manifest says {}",
                logical, got_sha, want_sha
            )));
        }
        let stored = artifact_dir.join(sanitize(logical));
        std::fs::write(&stored, &bytes).map_err(|e| format!("persist {}: {}", logical, e))?;
        artifacts.push(ArtifactRef {
            logical_path: logical.to_string(),
            stored_path: stored.display().to_string(),
            sha256: got_sha,
            bytes: Some(bytes.len()),
        });
    }

    // Canonical-HIR golden (and any other declared byte artifacts).
    for want in &oracle.artifacts {
        let produced = artifacts
            .iter()
            .find(|a| a.logical_path == want.path)
            .map(|a| std::fs::read(Path::new(&a.stored_path)).unwrap_or_default());
        match produced {
            None => mismatches.push(Mismatch {
                observation: format!("artifact {}", want.path),
                kind: "missing",
                expected: format!("{} bytes", want.bytes.len()),
                actual: "not produced".into(),
                first_diff: None,
            }),
            Some(actual) => {
                if let Some(m) = compare_bytes(
                    &want.bytes,
                    &actual,
                    want.compare,
                    &format!("artifact {}", want.path),
                ) {
                    mismatches.push(m);
                }
            }
        }
    }

    // Native run observation: exit + raw stream bytes.
    if let Some(run) = ok.get("run") {
        let got_exit = run["exit_code"].as_i64();
        native.exit_code = got_exit.map(|c| c as i32);
        if let Some(want) = oracle.exit {
            if got_exit != Some(want as i64) {
                mismatches.push(Mismatch {
                    observation: "exit".into(),
                    kind: "exit",
                    expected: format!("{}", want),
                    actual: format!("{:?}", got_exit),
                    first_diff: None,
                });
            }
        }
        // R2 F03: the wire field is <key>_b64; a missing/invalid field is
        // a protocol refusal, never a silent empty stream.
        let mut stream_err: Option<String> = None;
        let mut compare_stream = |key: &str,
                                  want: Option<&auto_test_case::contract::StreamExpect>,
                                  out: &mut Vec<Mismatch>| {
            let Some(w) = want else {
                // Undeclared streams are not read here at all (their
                // encoding was still strictly validated in
                // validate_response — F06).
                return;
            };
            let field = format!("{}_b64", key);
            let raw = match run[&field].as_str() {
                Some(r) => r,
                None => {
                    stream_err = Some(format!(
                        "worker run response missing {} — never an empty stream",
                        &field
                    ));
                    return;
                }
            };
            let actual_b = match b64d_strict(raw) {
                Ok(b) => b,
                Err(e) => {
                    stream_err = Some(format!("worker stream {}: {}", &field, e));
                    return;
                }
            };
            if let Some(m) = compare_bytes(w.text.as_bytes(), &actual_b, w.compare, key) {
                out.push(m);
            }
        };
        compare_stream("stdout", oracle.stdout.as_ref(), &mut mismatches);
        compare_stream("stderr", oracle.stderr.as_ref(), &mut mismatches);
        if let Some(e) = stream_err {
            return Err(e);
        }
    } else if oracle.exit.is_some() || oracle.stdout.is_some() || oracle.stderr.is_some() {
        mismatches.push(Mismatch {
            observation: "run".into(),
            kind: "missing",
            expected: "a native run observation".into(),
            actual: format!("{} produced no run", check.action.as_str()),
            first_diff: None,
        });
    }

    // Trace/toolchain into the v2 report record.
    native.stage_spans = ok["trace"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|s| {
            Some((
                s["stage"].as_str()?.to_string(),
                s["wall_ms"].as_u64().unwrap_or(0),
            ))
        })
        .collect();
    if let Some(tc) = ok.get("toolchain") {
        // R1 (F07): native rows carry exactly the axes the stale checker
        // can RE-COMPUTE (the worker exe content hash embeds compiler and
        // dependency versions; the protocol version pins the IPC face).
        // Display-only fields (lld path, sdk dir, version strings) stay in
        // the worker response, out of the identity.
        // R2 (F09): the FULL real tool axes — worker exe hash (embeds the
        // compiler) + the actual linker content hash + the SDK um dir —
        // every axis the stale checker can independently recompute.
        for (axis, key) in [
            ("worker-exe-sha256", "worker_exe_sha256"),
            ("lld-sha256", "lld_sha256"),
            ("sdk-um", "sdk_um"),
        ] {
            if let Some(v) = tc[key].as_str() {
                if !v.is_empty() {
                    native.toolchain.push((axis.to_string(), v.to_string()));
                }
            }
        }
        native
            .toolchain
            .push(("ac-worker-protocol".to_string(), "acworker/1".to_string()));
    }
    native.cache_mode = Some(check.options.effective_cache().as_str().to_string());
    if let Some(cache) = ok.get("cache") {
        native.cache_hits = cache["hits"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|h| Some((h[0].as_str()?.to_string(), h[1].as_str()?.to_string())))
            .collect();
        native.counters = cache["counters"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|c| Some((c[0].as_str()?.to_string(), c[1].as_u64()? as u32)))
            .collect();
    }

    // Declared perf assertion (absent = record-only; the reference recipe
    // owns the P95 gates — ordinary cases never carry wall clocks).
    if let Some(perf) = oracle.perf {
        if let Some(max) = perf.total_ms_max {
            let took = started.elapsed().as_millis() as u64;
            if took > max {
                mismatches.push(Mismatch {
                    observation: "perf.total_ms".into(),
                    kind: "perf",
                    expected: format!("<= {}ms", max),
                    actual: format!("{}ms", took),
                    first_diff: None,
                });
            }
        }
    }

    let mut verdict = if mismatches.is_empty() {
        Verdict::pass()
    } else {
        Verdict::fail(mismatches)
    };
    verdict.artifacts = artifacts;
    verdict.native = Some(native);
    Ok(verdict)
}

/// `ac/run` consuming an upstream build's products: the RUN goes through
/// the ac-case-worker process under the SAME contained, budgeted, capped
/// executor (R1 F01/F02) — the parent never spawns product executables
/// itself, never recompiles source to replace a bad product, and an
/// execution FAILURE is an ERROR verdict, not a swallowed PASS.
#[allow(clippy::too_many_arguments)]
pub fn consume_and_run(
    exe_path: &Path,
    check: &CheckSpec,
    expect_exit: Option<i32>,
    expect_stdout: Option<&auto_test_case::contract::StreamExpect>,
    expect_stderr: Option<&auto_test_case::contract::StreamExpect>,
    expect_perf_total_ms_max: Option<u64>,
) -> Result<Verdict, String> {
    let started = Instant::now();
    let scratch = exe_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "consumer-work-{}",
            hex(&sha256(
                format!("{}|{}", std::process::id(), check.id).as_bytes()
            ))
        ));
    std::fs::create_dir_all(&scratch).map_err(|e| format!("consumer scratch: {}", e))?;
    let budget = check.options.effective_budget();
    let (build_ms, run_ms, total_ms) = (
        budget.build_ms.unwrap_or(60_000),
        budget.run_ms.unwrap_or(5_000),
        budget.total_ms.unwrap_or(70_000),
    );
    let nonce = hex(&sha256(
        format!(
            "consumer|{}|{}",
            std::process::id(),
            started.elapsed().as_nanos()
        )
        .as_bytes(),
    ));
    let request_hash = hex(&sha256(
        format!(
            "acworker/1|product|{}|{}|{}",
            exe_path.display(),
            check.id,
            total_ms
        )
        .as_bytes(),
    ));
    let request = serde_json::json!({
        "schema": "acworker/1",
        "kind": "run",
        "nonce": nonce,
        "request_hash": request_hash,
        "input": { "kind": "product", "exe_path": exe_path.display().to_string() },
        "profile": check.options.effective_profile().as_str(),
        "entry": check.options.effective_entry(),
        "budget": { "build_ms": build_ms, "run_ms": run_ms, "total_ms": total_ms },
        "scratch": scratch.display().to_string(),
    });
    let response = call_worker(
        &request.to_string(),
        &nonce,
        &request_hash,
        "run",
        Duration::from_millis(total_ms),
    )?;
    if let Some(err) = response.get("error") {
        // Infrastructure failure of the consumed run — an ERROR verdict the
        // runner must propagate (R1 F02), never a PASS.
        return Ok(Verdict::error(format!(
            "ac consumer run failed [{}]: {}",
            err["code"].as_str().unwrap_or("?"),
            err["message"].as_str().unwrap_or("")
        )));
    }
    let ok = response
        .get("ok")
        .ok_or_else(|| "consumer response carries neither ok nor error".to_string())?;
    let run = ok
        .get("run")
        .ok_or_else(|| "consumer run response missing its run observation".to_string())?;
    let mut mismatches = Vec::new();
    let got_exit = run["exit_code"].as_i64();
    if let Some(want) = expect_exit {
        if got_exit != Some(want as i64) {
            mismatches.push(Mismatch {
                observation: "exit".into(),
                kind: "exit",
                expected: format!("{}", want),
                actual: format!("{:?}", got_exit),
                first_diff: None,
            });
        }
    }
    for (key, want) in [("stdout", expect_stdout), ("stderr", expect_stderr)] {
        if let Some(w) = want {
            let field = format!("{}_b64", key);
            let raw = run[&field].as_str().ok_or_else(|| {
                format!(
                    "consumer run response missing {} — never an empty stream",
                    &field
                )
            })?;
            let actual =
                b64d_strict(raw).map_err(|e| format!("consumer stream {}: {}", &field, e))?;
            if let Some(m) = compare_bytes(w.text.as_bytes(), &actual, w.compare, key) {
                mismatches.push(m);
            }
        }
    }
    if let Some(max) = expect_perf_total_ms_max {
        let took = started.elapsed().as_millis() as u64;
        if took > max {
            mismatches.push(Mismatch {
                observation: "perf.total_ms".into(),
                kind: "perf",
                expected: format!("<= {}ms", max),
                actual: format!("{}ms", took),
                first_diff: None,
            });
        }
    }
    let mut native = NativeRecord {
        exit_code: got_exit.map(|c| c as i32),
        cache_mode: None,
        ..Default::default()
    };
    native.stage_spans = ok["trace"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|s| {
            Some((
                s["stage"].as_str()?.to_string(),
                s["wall_ms"].as_u64().unwrap_or(0),
            ))
        })
        .collect();
    if let Some(tc) = ok.get("toolchain") {
        // R1 (F07): native rows carry exactly the axes the stale checker
        // can RE-COMPUTE (the worker exe content hash embeds compiler and
        // dependency versions; the protocol version pins the IPC face).
        // Display-only fields (lld path, sdk dir, version strings) stay in
        // the worker response, out of the identity.
        // R2 (F09): the FULL real tool axes — worker exe hash (embeds the
        // compiler) + the actual linker content hash + the SDK um dir —
        // every axis the stale checker can independently recompute.
        for (axis, key) in [
            ("worker-exe-sha256", "worker_exe_sha256"),
            ("lld-sha256", "lld_sha256"),
            ("sdk-um", "sdk_um"),
        ] {
            if let Some(v) = tc[key].as_str() {
                if !v.is_empty() {
                    native.toolchain.push((axis.to_string(), v.to_string()));
                }
            }
        }
        native
            .toolchain
            .push(("ac-worker-protocol".to_string(), "acworker/1".to_string()));
    }
    let _ = std::fs::remove_dir_all(&scratch);
    let mut verdict = if mismatches.is_empty() {
        Verdict::pass()
    } else {
        Verdict::fail(mismatches)
    };
    verdict.native = Some(native);
    Ok(verdict)
}

/// Resolve the NAMED support library "trace-marks" (R2 F01): the PLAN-741
/// trace-marks runtime, built once per process from the ac-core
/// test-support crate (a warm no-op rebuild is ~1s). Named-lib admission
/// keeps arbitrary paths out of the contract; the build is the same one
/// ac-core's own trace tests use.
fn trace_marks_lib() -> Result<PathBuf, String> {
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../experimental/ac-core/test-support");
            let out = Command::new("cargo")
                .args(["build", "--locked"])
                .current_dir(&dir)
                .output()
                .map_err(|e| format!("spawn cargo for trace-marks: {}", e))?;
            if !out.status.success() {
                let tail = String::from_utf8_lossy(&out.stderr);
                return Err(format!("trace-marks build failed:\n{}", tail));
            }
            let lib = dir.join("target/debug/ac_trace_support.lib");
            if lib.is_file() {
                Ok(lib)
            } else {
                Err(format!("trace-marks library missing at {}", lib.display()))
            }
        })
        .clone()
}

/// The ACTUAL current toolchain identity for stale comparison (R1 F07):
/// the worker binary's content hash (it statically embeds the compiler),
/// the resolved linker's file hash and the SDK um dir — the same axes the
/// executor binds into native records. Rows missing or contradicting any
/// axis are stale, never silently consistent.
pub fn current_toolchain_identity() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    if let Ok(exe) = worker_binary() {
        if let Ok(bytes) = std::fs::read(&exe) {
            m.insert("worker-exe-sha256".into(), hex(&sha256(&bytes)));
        }
    }
    // R2 (F09): the EXTERNAL tools are recomputed here with the same
    // resolution order ac-core uses (env override -> rustc sysroot ->
    // registry roots), keeping the stale axes equal to the execution axes.
    if let Some(lld) = resolve_rust_lld() {
        if let Ok(bytes) = std::fs::read(&lld) {
            m.insert("lld-sha256".into(), hex(&sha256(&bytes)));
        }
    }
    if let Some(sdk) = resolve_sdk_um_dir() {
        m.insert("sdk-um".into(), sdk.display().to_string());
    }
    m.insert("ac-worker-protocol".into(), "acworker/1".into());
    m
}

/// Mirror of ac-core's `find_rust_lld` resolution (displayed identity only;
/// the linker itself is never invoked here).
fn resolve_rust_lld() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("AC741_RUST_LLD") {
        let p = PathBuf::from(env_path);
        if p.is_file() {
            return Some(p);
        }
        return None;
    }
    let out = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sysroot = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let candidate =
        PathBuf::from(sysroot).join("lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe");
    candidate.is_file().then_some(candidate)
}

/// Mirror of ac-core's `find_sdk_um_dir` resolution.
fn resolve_sdk_um_dir() -> Option<PathBuf> {
    if let Ok(env_dir) = std::env::var("AC741_SDK_UM_LIB") {
        let p = PathBuf::from(env_dir);
        return p.join("kernel32.lib").is_file().then_some(p);
    }
    let out = std::process::Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Windows Kits\Installed Roots",
            "/v",
            "KitsRoot10",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut roots = Vec::new();
    if let Some(pos) = text.find("KitsRoot10") {
        if let Some(rp) = text[pos..].find("REG_SZ") {
            let line = &text[pos + rp + "REG_SZ".len()..];
            let val = line.lines().next().unwrap_or("").trim();
            if !val.is_empty() {
                roots.push(PathBuf::from(val));
            }
        }
    }
    roots.push(PathBuf::from(r"C:\Program Files (x86)\Windows Kits"));
    for root in roots {
        let Ok(entries) = std::fs::read_dir(root.join("Lib")) else {
            continue;
        };
        let mut versions: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("um/x64/kernel32.lib").is_file())
            .collect();
        versions.sort();
        if let Some(latest) = versions.pop() {
            return Some(latest.join("um/x64"));
        }
    }
    None
}

/// R2 F04: required products are a SUCCESS-shape contract — only a
/// diagnostics-free response must carry them; language failures carry none.
fn require_products(
    check: &CheckSpec,
    seen_logical: &std::collections::BTreeSet<String>,
) -> Result<(), String> {
    let has = |l: &str| seen_logical.contains(l);
    match (check.adapter, check.action) {
        (_, auto_test_case::contract::Action::Build) => {
            if !has("app.obj") || !has("app.exe") {
                return Err("build response must carry both app.obj and app.exe".to_string());
            }
        }
        (auto_test_case::contract::Adapter::Ac, auto_test_case::contract::Action::Run) => {
            if !has("app.exe") {
                return Err("run response must carry app.exe".to_string());
            }
        }
        // T-06: syntax parse/edit must carry the lossless reconstruction.
        (auto_test_case::contract::Adapter::Syntax, _) => {
            if !has("reconstructed.at") {
                return Err("syntax response must carry reconstructed.at".to_string());
            }
        }
        _ => {}
    }
    Ok(())
}

fn effective_budget(b: &BudgetSpec) -> BudgetSpec {
    BudgetSpec {
        build_ms: Some(b.build_ms.unwrap_or(60_000)),
        run_ms: Some(b.run_ms.unwrap_or(5_000)),
        total_ms: Some(b.total_ms.unwrap_or(70_000)),
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_tail(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars().take(120) {
        out.push_str(&format!("{:?}", c).trim_matches('"'));
    }
    out
}

fn b64(bytes: &[u8]) -> String {
    // Local base64 (std): auto-lang must not grow a base64 dependency for
    // one protocol field; this is the standard alphabet, padded.
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Strict standard-alphabet base64 decode (R2 F03): malformed padding or
/// characters are an error — the forgiving decoder silently turned
/// protocol damage into empty streams.
fn b64d_strict(s: &str) -> Result<Vec<u8>, String> {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = s.as_bytes();
    if bytes.len() % 4 != 0 {
        return Err(format!("length {} is not a multiple of 4", bytes.len()));
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for (ci, chunk) in bytes.chunks(4).enumerate() {
        let mut vals = [0u32; 4];
        let mut pad = 0usize;
        for (i, &c) in chunk.iter().enumerate() {
            if c == b'=' {
                if i < 2 || (i == 2 && chunk[3] != b'=') || ci * 4 + 4 != bytes.len() {
                    return Err("misplaced '=' padding".to_string());
                }
                pad += 1;
            } else {
                let v = T.iter().position(|&t| t == c).ok_or_else(|| {
                    format!("invalid base64 byte {:?} at chunk {}", c as char, ci)
                })?;
                if pad > 0 {
                    return Err("data after '=' padding".to_string());
                }
                vals[i] = v as u32;
            }
        }
        let n = (vals[0] << 18) | (vals[1] << 12) | (vals[2] << 6) | vals[3];
        out.push((n >> 16) as u8);
        if pad < 2 {
            out.push((n >> 8) as u8);
        }
        if pad < 1 {
            out.push(n as u8);
        }
    }
    Ok(out)
}

fn b64d(s: &str) -> Vec<u8> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let mut acc = 0u32;
    let mut bits = 0u32;
    for &c in s.as_bytes() {
        if c == b'=' {
            break;
        }
        let Some(v) = val(c) else { continue };
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}
