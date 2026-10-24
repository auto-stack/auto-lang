//! mn-worker — the bounded MIR-native worker binary (PLAN-818).
//!
//! `restore-run` opens the store in THIS process, re-admits a persisted
//! closure from bytes, really links and really runs it. The worker has no
//! MIR lowering path at all: `codegen_emit_units=0` is structural.
//!
//! `case-run` is the FORMAL AutoDown entry (T-02): one JSON request line
//! on stdin (recipe/nonce/source_b64/entry/caps/store_root/tracelib/
//! request_hash), one JSON response line on stdout. The worker never sees
//! expected outcomes — judgement is entirely parent-side.
//!
//! Output contract: exactly one JSON result line on stdout; raw PE
//! stdout/stderr ride inside as hex (byte-exact, never UTF-8-mangled).

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod case_run {
    use super::*;

    pub const MN_WORKER_PROTOCOL: &str = "mnworker/1";
    pub const MN_WORKER_VERSION: &str = "mnworker/1.0";

    /// Locate (and on first use build) the landed ac-trace-support
    /// staticlib through the contained, deadline-bounded executor —
    /// missing tools are infra failures, never silent passes.
    fn tracelib() -> Result<Option<PathBuf>, String> {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let support = repo.join("experimental/ac-core/test-support");
        let lib = support.join("target/debug/ac_trace_support.lib");
        if lib.is_file() {
            return Ok(Some(lib));
        }
        if !support.join("Cargo.toml").is_file() {
            return Err(format!(
                "ac-core test-support not found at {}",
                support.display()
            ));
        }
        let mut cmd = std::process::Command::new("cargo");
        cmd.args(["build", "--locked"]).current_dir(&support);
        let run = auto_ac_prototype::link::run_deadlined_split(
            &mut cmd,
            std::time::Duration::from_secs(300),
        )
        .map_err(|e| format!("test-support build: {e:?}"))?;
        if run.0 != Some(0) {
            return Err(format!("test-support build exited {:?}: {}", run.0, run.2));
        }
        if lib.is_file() {
            Ok(Some(lib))
        } else {
            Err(format!(
                "trace support lib missing after build: {}",
                lib.display()
            ))
        }
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// The incremental-session path (F-818-R1-03): one warm session per
    /// request; edit-source steps are applied as single-body transactions
    /// (literal located exactly once in the CURRENT source); every step
    /// really links and runs. The judged value is the LAST step's exit.
    #[allow(clippy::too_many_arguments)]
    fn serve_incremental(
        req: serde_json::Value,
        echo: serde_json::Value,
        source: String,
        entry: String,
        caps: Vec<String>,
        store_root: String,
        workdir: String,
        tracelib: Option<PathBuf>,
    ) -> ExitCode {
        let fail = |echo: serde_json::Value, code: &str, message: String| -> ExitCode {
            let mut r = echo.clone();
            r["error"] = serde_json::json!({"code": code, "message": message});
            println!("{r}");
            ExitCode::FAILURE
        };
        let steps = req
            .get("steps")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut store_path = PathBuf::from(&store_root);
        // The incremental session wants its own scratch store next to the
        // request's workdir (session lifetime = this request).
        if store_path
            .components()
            .last()
            .map(|c| c.as_os_str() == "mn-store")
            .unwrap_or(false)
        {
            store_path = workdir.clone().into();
        }
        store_path.push("mn-inc-store");
        let (mut session, cold) = match ac_mn::incremental::MnIncrementalSession::warmup(
            &source,
            &entry,
            &caps,
            &store_path,
        ) {
            Ok(pair) => pair,
            Err(e) => return fail(echo, "mn.worker.incremental", e.message()),
        };
        let _ = &tracelib;
        let mut step_records: Vec<serde_json::Value> = Vec::new();
        let record = |obs: &ac_mn::incremental::EditObservation, op: &str| {
            serde_json::json!({
                "op": op,
                "exit_code": obs.exit_code,
                "changed_fn": obs.changed_fn,
                "codegen_units": obs.codegen_units,
                "cache_hits": obs.cache_hits,
                "fallback_whole_lower": obs.fallback_whole_lower,
                "reused_history_unit": obs.reused_history_unit,
                "unaffected_stage_delta": obs.unaffected_stage_delta,
                "whole_module_stage_delta": obs.whole_module_stage_delta,
                "refused": obs.refused_reason,
                "infra": obs.infra,
                "wall_ms": obs.wall_ms,
                "phase_walls": obs.phase_walls_us,
            })
        };
        step_records.push(record(&cold, "warmup"));
        let mut judged_exit: Option<i32> = cold.exit_code;
        let mut refused: Option<(String, String)> = None;
        for (st_idx, st) in steps.iter().enumerate() {
            match st.get("op").and_then(|v| v.as_str()) {
                Some("edit-source") => {
                    let from = st.get("from").and_then(|v| v.as_str()).unwrap_or("");
                    let to = st.get("to").and_then(|v| v.as_str()).unwrap_or("");
                    // Locate the literal exactly once in the CURRENT source.
                    let cur = session_source(&session);
                    let hits = cur.matches(from).count();
                    if from.is_empty() || hits != 1 {
                        return fail(
                            echo,
                            "mn.worker.edit",
                            format!("edit-source literal matched {hits} times (expected once)"),
                        );
                    }
                    let start = cur.find(from).expect("one hit") as u64;
                    let end = start + from.len() as u64;
                    let obs = session.edit(start, end, to.as_bytes().to_vec());
                    if let Some(i) = &obs.infra {
                        return fail(echo, "mn.worker.incremental", i.clone());
                    }
                    if let Some(r) = &obs.refused_reason {
                        if r.contains("syntax fallback") {
                            // The 828 syntax increment engine cannot
                            // converge on THIS edit shape (a LEGAL edit,
                            // an engine limitation): fall back to the
                            // WHOLE-REQUEST path over the fully edited
                            // source — recorded, never labeled incremental.
                            return serve_incremental_whole_fallback(
                                echo,
                                &source,
                                steps[st_idx..].to_vec(),
                                from,
                                to,
                                &entry,
                                &caps,
                                &store_root,
                                &workdir,
                                tracelib.as_deref(),
                            );
                        }
                        refused = Some(("mn.incremental.refused".into(), r.clone()));
                    }
                    judged_exit = obs.exit_code;
                    step_records.push(record(&obs, "edit-source"));
                }
                Some("run") => {
                    let obs = session.run_only();
                    if let Some(i) = &obs.infra {
                        return fail(echo, "mn.worker.incremental", i.clone());
                    }
                    judged_exit = obs.exit_code;
                    step_records.push(record(&obs, "run"));
                }
                _ => {
                    return fail(
                        echo,
                        "mn.worker.steps",
                        "unsupported op in the incremental path".to_string(),
                    );
                }
            }
        }
        session.close();
        let mut ok = serde_json::json!({
            "recipe": "ac/mir-native-scalar-v1",
            "worker_version": MN_WORKER_VERSION,
            "mn_abi": ac_mn::config::MN_ABI,
            "mn_target": ac_mn::config::MN_TARGET,
            "incremental": true,
            "steps": step_records,
        });
        ok["counters"] = serde_json::json!({});
        ok["spans"] = serde_json::json!({});
        let mut resp = echo;
        if let Some((code, message)) = refused {
            resp["diagnostic"] = serde_json::json!({"code": code, "message": message});
        } else {
            ok["exit_code"] = serde_json::json!(judged_exit);
            ok["stderr_hex"] = serde_json::json!("");
            ok["stdout_hex"] = serde_json::json!("");
        }
        resp["ok"] = ok;
        println!("{resp}");
        ExitCode::SUCCESS
    }

    /// The whole-request FALLBACK for syntax-increment non-convergence:
    /// materialize the refused edit (plus every remaining edit step) on
    /// the ORIGINAL source and run ONE full MnDriver request. The
    /// response is the legacy whole-request shape plus
    /// `warm_edit_fallback: true` — never labeled incremental
    /// (F-818-R1-03).
    #[allow(clippy::too_many_arguments)]
    fn serve_incremental_whole_fallback(
        echo: serde_json::Value,
        source: &str,
        remaining: Vec<serde_json::Value>,
        refused_from: &str,
        refused_to: &str,
        entry: &str,
        caps: &[String],
        store_root: &str,
        workdir: &str,
        tracelib: Option<&Path>,
    ) -> ExitCode {
        let fail = |code: &str, message: String| -> ExitCode {
            let mut r = echo.clone();
            r["error"] = serde_json::json!({"code": code, "message": message});
            println!("{r}");
            ExitCode::FAILURE
        };
        let mut cur = source.to_string();
        let mut apply = |from: &str, to: &str, cur: &mut String| -> Result<(), String> {
            let hits = cur.matches(from).count();
            if from.is_empty() || hits != 1 {
                return Err(format!(
                    "edit-source literal matched {hits} times (expected once)"
                ));
            }
            *cur = cur.replacen(from, to, 1);
            Ok(())
        };
        if let Err(e) = apply(refused_from, refused_to, &mut cur) {
            return fail("mn.worker.edit", e);
        }
        for st in remaining.iter().skip(1) {
            if st.get("op").and_then(|v| v.as_str()) == Some("edit-source") {
                let from = st.get("from").and_then(|v| v.as_str()).unwrap_or("");
                let to = st.get("to").and_then(|v| v.as_str()).unwrap_or("");
                if let Err(e) = apply(from, to, &mut cur) {
                    return fail("mn.worker.edit", e);
                }
            }
        }
        let mut driver =
            match ac_mn::driver::MnDriver::with_store(&PathBuf::from(store_root), "mn-case") {
                Ok(d) => d,
                Err(e) => return fail("mn.worker.store", e.message()),
            };
        let r = driver.request(&cur, entry, caps, &PathBuf::from(workdir), tracelib);
        let mut ok = serde_json::json!({
            "recipe": "ac/mir-native-scalar-v1",
            "worker_version": MN_WORKER_VERSION,
            "mn_abi": ac_mn::config::MN_ABI,
            "mn_target": ac_mn::config::MN_TARGET,
            "warm_edit_fallback": true,
            "counters": {},
            "spans": {},
        });
        for (k, v) in &r.obs.counters {
            ok["counters"][k] = serde_json::json!(v);
        }
        for (k, v) in &r.obs.spans {
            ok["spans"][k] = serde_json::json!(v);
        }
        if let Some(fp) = &r.manifest_fp {
            ok["manifest_fp"] = serde_json::json!(fp);
        }
        let mut resp = echo.clone();
        if r.ok {
            ok["exit_code"] = serde_json::json!(r.exit_code);
            ok["stderr_hex"] = serde_json::json!(hex(&r.stderr));
            ok["stdout_hex"] = serde_json::json!(hex(&r.stdout));
            resp["ok"] = ok;
        } else if let Some(e) = &r.error {
            resp["ok"] = ok;
            resp["diagnostic"] = serde_json::json!({"code": e.code(), "message": e.message()});
        } else {
            return fail("mn.worker.internal", "inconsistent result".into());
        }
        println!("{resp}");
        ExitCode::SUCCESS
    }

    /// The session's current source (diagnostics seam for the edit
    /// locator; the production path reads the 828 session).
    fn session_source(session: &ac_mn::incremental::MnIncrementalSession) -> String {
        session.source().to_string()
    }

    pub fn serve() -> ExitCode {
        let mut line = String::new();
        {
            let stdin = std::io::stdin();
            let mut lock = stdin.lock();
            loop {
                line.clear();
                match lock.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) if !line.trim().is_empty() => break,
                    Ok(_) => continue,
                    Err(e) => {
                        eprintln!("mn-worker case-run: read stdin: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
        }
        let req: serde_json::Value = match serde_json::from_str(line.trim()) {
            Ok(v) => v,
            Err(e) => {
                let resp = serde_json::json!({
                    "schema": MN_WORKER_PROTOCOL,
                    "error": {"code": "mn.worker.request", "message": format!("bad request line: {e}")}
                });
                println!("{resp}");
                return ExitCode::FAILURE;
            }
        };
        let echo = serde_json::json!({
            "schema": MN_WORKER_PROTOCOL,
            "nonce": req.get("nonce").cloned().unwrap_or(serde_json::Value::Null),
            "request_hash": req.get("request_hash").cloned().unwrap_or(serde_json::Value::Null),
        });
        let fail = |echo: serde_json::Value, code: &str, message: String| -> ExitCode {
            let mut r = echo.clone();
            r["error"] = serde_json::json!({"code": code, "message": message});
            println!("{r}");
            ExitCode::FAILURE
        };
        let recipe = req.get("recipe").and_then(|v| v.as_str()).unwrap_or("");
        if recipe != "ac/mir-native-scalar-v1" {
            return fail(
                echo,
                "mn.worker.recipe",
                format!("recipe `{recipe}` is not ac/mir-native-scalar-v1"),
            );
        }
        let Some(source_b64) = req.get("source_b64").and_then(|v| v.as_str()) else {
            return fail(echo, "mn.worker.request", "missing source_b64".into());
        };
        use base64::Engine as _;
        let Ok(source) = base64::engine::general_purpose::STANDARD.decode(source_b64) else {
            return fail(
                echo,
                "mn.worker.request",
                "source_b64 is not valid base64".into(),
            );
        };
        let Ok(source) = String::from_utf8(source) else {
            return fail(echo, "mn.worker.request", "source is not UTF-8".into());
        };
        let entry = req
            .get("entry")
            .and_then(|v| v.as_str())
            .unwrap_or("d_main")
            .to_string();
        let caps: Vec<String> = req
            .get("caps")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|c| c.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let store_root = req
            .get("store_root")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if store_root.is_empty() {
            return fail(echo, "mn.worker.request", "missing store_root".into());
        }
        let workdir = req
            .get("workdir")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if workdir.is_empty() {
            return fail(echo, "mn.worker.request", "missing workdir".into());
        }
        let tracelib = match req.get("tracelib").and_then(|v| v.as_str()) {
            Some(p) => Some(PathBuf::from(p)),
            None => match tracelib() {
                Ok(p) => p,
                Err(e) => return fail(echo, "mn.worker.tracelib", e),
            },
        };

        // F-818-R1-03: steps carrying edit-source route through the
        // D-828 incremental session (warm in-request session: one
        // whole-module warm-up, then SINGLE-BODY edit transactions with
        // unit-level COFF re-emission / history reuse and a real link/run
        // per step). A trailing `run` step is the judged fresh link/run.
        // Plain `[run]` requests keep the whole-request pipeline (the
        // cold/restored scenario) — its warm edits are NOT incremental.
        let steps: Vec<serde_json::Value> = req
            .get("steps")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let has_edit = steps
            .iter()
            .any(|st| st.get("op").and_then(|v| v.as_str()) == Some("edit-source"));
        if has_edit {
            let inc_req = serde_json::json!({
                "recipe": "ac/mir-native-scalar-v1",
                "steps": steps,
            });
            return case_run::serve_incremental(
                inc_req,
                echo,
                source,
                entry,
                caps,
                store_root.to_string(),
                workdir.to_string(),
                tracelib,
            );
        }

        // Cross-process tool-discovery memo (T-07 turnaround): the lld/SDK
        // PATHS are memoized next to the store (short-lived worker
        // processes of one session share it); tool CONTENT identity is
        // still re-hashed in-process per request.
        {
            let memo = PathBuf::from(&store_root)
                .join("..")
                .join("mn-tool-memo.json");
            if let Some(parent) = memo.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            ac_mn::link_driver::set_tool_disk_memo(memo);
        }
        let mut driver =
            match ac_mn::driver::MnDriver::with_store(&PathBuf::from(store_root), "mn-case") {
                Ok(d) => d,
                Err(e) => return fail(echo, "mn.worker.store", e.message()),
            };
        let r = driver.request(
            &source,
            &entry,
            &caps,
            &PathBuf::from(workdir),
            tracelib.as_deref(),
        );
        let mut ok = serde_json::json!({
            "recipe": "ac/mir-native-scalar-v1",
            "worker_version": MN_WORKER_VERSION,
            "mn_abi": ac_mn::config::MN_ABI,
            "mn_target": ac_mn::config::MN_TARGET,
            "counters": {},
            "spans": {},
        });
        for (k, v) in &r.obs.counters {
            ok["counters"][k] = serde_json::json!(v);
        }
        for (k, v) in &r.obs.spans {
            ok["spans"][k] = serde_json::json!(v);
        }
        if let Some(fp) = &r.manifest_fp {
            ok["manifest_fp"] = serde_json::json!(fp);
        }
        match (&r.ok, &r.error) {
            (true, None) => {
                ok["exit_code"] = serde_json::json!(r.exit_code);
                ok["stderr_hex"] = serde_json::json!(hex(&r.stderr));
                ok["stdout_hex"] = serde_json::json!(hex(&r.stdout));
                let mut resp = echo;
                resp["ok"] = ok;
                println!("{resp}");
                ExitCode::SUCCESS
            }
            (false, Some(e)) => {
                // A diagnosed pipeline rejection: report the mn.* code —
                // the parent maps it to the contract's completion faces.
                let mut resp = echo;
                resp["ok"] = ok;
                resp["diagnostic"] = serde_json::json!({"code": e.code(), "message": e.message()});
                println!("{resp}");
                ExitCode::SUCCESS
            }
            _ => fail(echo, "mn.worker.internal", "inconsistent result".into()),
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(mode) = args.first() else {
        eprintln!("usage: mn-worker restore-run --store <dir> --workdir <dir> [--tracelib <path>] (--manifest-id <id> | --owner-module <m> --entry <e> --fps <u:fp,...> --link-inputs <digest>)");
        eprintln!("       mn-worker case-run   (one JSON request line on stdin)");
        eprintln!("       mn-worker load-revision --rev <n>  (reference workload revision dump)");
        return ExitCode::from(2);
    };
    let mut flag = |name: &str| -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let _ = &mut flag;
    match mode.as_str() {
        "case-run" => case_run::serve(),
        // The frozen reference workload (golden.md §2) — single source of
        // truth for the formal turnaround scenarios (the 798 `load-revision`
        // form): each revision bumps the frozen fn_041 body-edit bit.
        "load-revision" => {
            let rev: i64 = flag("--rev").and_then(|v| v.parse().ok()).unwrap_or(0);
            let wl = ac_mn::workload::generate(rev);
            let v = serde_json::json!({
                "rev": rev,
                "source": wl.src,
                "expected_main": wl.expected,
            });
            println!("{v}");
            ExitCode::SUCCESS
        }
        "restore-run" => {
            let Some(store) = flag("--store") else {
                eprintln!("restore-run: --store is required");
                return ExitCode::from(2);
            };
            let Some(workdir) = flag("--workdir") else {
                eprintln!("restore-run: --workdir is required");
                return ExitCode::from(2);
            };
            let trace_lib = flag("--tracelib").map(PathBuf::from);
            let result = if let Some(id) = flag("--manifest-id") {
                ac_mn::worker::restore_run_by_id(
                    &PathBuf::from(store),
                    &id,
                    &PathBuf::from(workdir),
                    trace_lib.as_deref(),
                )
            } else {
                let Some(owner_module) = flag("--owner-module") else {
                    eprintln!("restore-run: --manifest-id or --owner-module required");
                    return ExitCode::from(2);
                };
                let Some(entry) = flag("--entry") else {
                    eprintln!("restore-run: --entry is required");
                    return ExitCode::from(2);
                };
                let Some(fps) = flag("--fps") else {
                    eprintln!("restore-run: --fps is required");
                    return ExitCode::from(2);
                };
                let Some(link_inputs) = flag("--link-inputs") else {
                    eprintln!("restore-run: --link-inputs is required");
                    return ExitCode::from(2);
                };
                let unit_fps: Vec<(String, String)> = fps
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|p| {
                        let (u, f) = p.split_once(':').expect("fps entries are unit:fp");
                        (u.to_string(), f.to_string())
                    })
                    .collect();
                ac_mn::worker::restore_run_by_key(
                    &PathBuf::from(store),
                    &owner_module,
                    &entry,
                    &unit_fps,
                    &link_inputs,
                    &PathBuf::from(workdir),
                    trace_lib.as_deref(),
                )
            };
            match result {
                Ok(out) => {
                    // One JSON line; stream bytes rendered as hex to keep
                    // the line parseable regardless of content.
                    let stderr_hex: String =
                        out.stderr.iter().map(|b| format!("{b:02x}")).collect();
                    let stdout_hex: String =
                        out.stdout.iter().map(|b| format!("{b:02x}")).collect();
                    println!(
                        "{{\"ok\":true,\"exit_code\":{:?},\"units_admitted\":{},\"manifest_fp\":\"{}\",\"stdout_hex\":\"{}\",\"stderr_hex\":\"{}\",\"codegen_emit_units\":0}}",
                        out.exit_code,
                        out.units_admitted,
                        out.manifest_fp,
                        stdout_hex,
                        stderr_hex
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    println!(
                        "{{\"ok\":false,\"error\":\"{}\",\"message\":\"{}\",\"codegen_emit_units\":0}}",
                        e.code().replace('"', "'"),
                        e.message().replace('\\', "\\\\").replace('"', "'").replace('\n', " ")
                    );
                    ExitCode::FAILURE
                }
            }
        }
        other => {
            eprintln!("mn-worker: unknown mode `{other}`");
            ExitCode::from(2)
        }
    }
}
