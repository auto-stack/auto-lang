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
    deadline: Duration,
) -> Result<serde_json::Value, String> {
    let exe = worker_binary()?;
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
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let reader = std::thread::spawn(move || {
        let mut out = String::new();
        let _ = stdout_pipe.read_to_string(&mut out);
        let _ = tx.send(out);
    });
    let response = match rx.recv_timeout(deadline) {
        Ok(out) => out,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err(format!(
                "ac-case-worker did not answer within {:?}; the worker tree was killed",
                deadline
            ));
        }
    };
    let _ = reader.join();
    let _ = child.wait();
    let line = response
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .ok_or_else(|| "ac-case-worker produced no response line".to_string())?;
    let v: serde_json::Value =
        serde_json::from_str(line).map_err(|e| format!("worker response undecodable: {}", e))?;
    // Identity re-check: the response must carry OUR nonce and hash back.
    let got_nonce = v["nonce"].as_str().unwrap_or("");
    let got_hash = v["request_hash"].as_str().unwrap_or("");
    if got_nonce != nonce || got_hash != request_hash {
        return Err(format!(
            "worker response identity mismatch (nonce {got_nonce:?}/hash {got_hash:?} — a stale or foreign response is never trusted)"
        ));
    }
    Ok(v)
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
        "store_root": std::env::var("AC_CASE_STORE").ok().unwrap_or_else(|| {
            run_dir.join("cache-store").display().to_string()
        }),
        "cache": check.options.effective_cache().as_str(),
        "project": "ac-case",
        "include_hir": !oracle.artifacts.is_empty(),
        "scratch": scratch.display().to_string(),
    });
    let ipc = WorkerIpc {
        nonce: nonce.clone(),
        request_hash: request_hash.clone(),
    };
    // Parent-side wall clock: the request total budget plus a fixed margin
    // for process spawn/IPC — the worker's internal stages are bounded by
    // the same budget, so this only catches a wedged worker.
    let wall = Duration::from_millis(total_ms + 15_000);
    let response = call_worker(&request.to_string(), &ipc.nonce, &ipc.request_hash, wall)?;

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

    // Artifacts: persist + parent-side manifest verification + oracle
    // byte-comparison for declared goldens (canonical HIR only by contract).
    let mut artifacts: Vec<ArtifactRef> = Vec::new();
    let mut artifact_total = 0usize;
    for a in ok["artifacts"].as_array().cloned().unwrap_or_default() {
        let logical = a["logical"].as_str().unwrap_or("").to_string();
        let path = a["path"].as_str().unwrap_or("").to_string();
        let want_sha = a["sha256"].as_str().unwrap_or("").to_string();
        let bytes =
            std::fs::read(&path).map_err(|e| format!("read worker artifact {}: {}", logical, e))?;
        let got_sha = hex(&sha256(&bytes));
        if got_sha != want_sha {
            return Ok(Verdict::error(format!(
                "worker manifest mismatch for {}: hashes {} but manifest says {}",
                logical, got_sha, want_sha
            )));
        }
        artifact_total += bytes.len();
        if artifact_total > 128 << 20 {
            return Ok(Verdict::error(format!(
                "check artifact total {} exceeds the 128MiB cap",
                artifact_total
            )));
        }
        let stored = artifact_dir.join(sanitize(&logical));
        std::fs::write(&stored, &bytes).map_err(|e| format!("persist {}: {}", logical, e))?;
        artifacts.push(ArtifactRef {
            logical_path: logical,
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
        let compare_stream = |key: &str, want: Option<&auto_test_case::contract::StreamExpect>| {
            let mut out = Vec::new();
            let actual_b = b64d(run[key].as_str().unwrap_or(""));
            if let Some(w) = want {
                if let Some(m) = compare_bytes(w.text.as_bytes(), &actual_b, w.compare, key) {
                    out.push(m);
                }
            }
            out
        };
        mismatches.extend(compare_stream("stdout", oracle.stdout.as_ref()));
        mismatches.extend(compare_stream("stderr", oracle.stderr.as_ref()));
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
        for (k, key) in [
            ("lld", "lld_path"),
            ("lld-sha256", "lld_sha256"),
            ("sdk-um", "sdk_um"),
            ("ac-core", "ac_core_version"),
            ("ac-case-worker", "worker_version"),
        ] {
            if let Some(v) = tc[key].as_str() {
                if !v.is_empty() {
                    native.toolchain.push((k.to_string(), v.to_string()));
                }
            }
        }
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

/// `ac/run` consuming an upstream build's products: RUNS the manifest-
/// verified exe (never recompiles source to replace a bad product) and
/// judges the consumer's own native exit oracle. Streams compare as raw
/// bytes like the direct path.
pub fn consume_and_run(
    exe_bytes: &[u8],
    check: &CheckSpec,
    expect_exit: Option<i32>,
    expect_stdout: Option<&auto_test_case::contract::StreamExpect>,
    expect_stderr: Option<&auto_test_case::contract::StreamExpect>,
) -> Result<Verdict, String> {
    let dir = std::env::temp_dir().join(format!(
        "ac788-consume-{}-{}",
        std::process::id(),
        sanitize(&check.id)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("consumer scratch: {}", e))?;
    let exe = dir.join("app.exe");
    std::fs::write(&exe, exe_bytes).map_err(|e| format!("stage consumed exe: {}", e))?;
    let t = Instant::now();
    let out = Command::new(&exe)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    let mut mismatches = Vec::new();
    let mut native = NativeRecord {
        toolchain: vec![("ac-case-consumer".into(), "direct-run".into())],
        ..Default::default()
    };
    match out {
        Ok(o) => {
            let code = o.status.code();
            native.exit_code = code;
            if let Some(want) = expect_exit {
                if code != Some(want) {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "exit",
                        expected: format!("{}", want),
                        actual: format!("{:?}", code),
                        first_diff: None,
                    });
                }
            }
            for (key, bytes, want) in [
                ("stdout", o.stdout.as_slice(), expect_stdout),
                ("stderr", o.stderr.as_slice(), expect_stderr),
            ] {
                if let Some(w) = want {
                    if let Some(m) = compare_bytes(w.text.as_bytes(), bytes, w.compare, key) {
                        mismatches.push(m);
                    }
                }
            }
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Ok(Verdict::error(format!("run consumed exe: {}", e)));
        }
    }
    native.stage_spans = vec![("run".into(), t.elapsed().as_millis() as u64)];
    let _ = std::fs::remove_dir_all(&dir);
    let mut verdict = if mismatches.is_empty() {
        Verdict::pass()
    } else {
        Verdict::fail(mismatches)
    };
    verdict.native = Some(native);
    Ok(verdict)
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
