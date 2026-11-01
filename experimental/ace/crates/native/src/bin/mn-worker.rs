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
    use ace_native::source_diagnostics as sd;

    pub const MN_WORKER_PROTOCOL: &str = "mnworker/1";
    pub const MN_WORKER_VERSION: &str = "mnworker/1.0";

    /// TEST-ONLY fault-injection seam (F-818-R3-01/R3-02 closing oracles):
    /// the REAL worker binary misbehaves in one protocol-aware way so the
    /// parent adapter's deadline/cap machinery is exercised against a
    /// worker that still computes valid echoes. Read once per process;
    /// production paths never set it. Modes:
    /// - `no-stdin-read`          never reads stdin (blocks the write path)
    /// - `stdout-oversize-noeol`  consumes the request, floods a
    ///                            newline-free stream past the IPC cap
    /// - `stdout-oversize-line`   consumes the request, floods one
    ///                            newline-terminated line past the cap
    /// - `stdout-flood-lines`     consumes the request, floods unsolicited
    ///                            small lines forever (bounded-queue probe)
    /// - `stop-after-warmup`      rounds: answers warmup, then never reads
    /// - `close-without-exit`     rounds: valid close reply, never exits
    /// - `ignore-close`           rounds: never answers close
    /// - `stderr-overcap`         rounds: valid protocol throughout, stderr
    ///                            floods past the parent's stderr cap
    /// - `stderr-under-cap`       rounds: same, but just under the cap
    ///                            (valid control)
    /// - `stdout-line-under-cap`  rounds: warmup line padded to just under
    ///                            the parent's line cap (valid control)
    fn fixture_fault() -> Option<String> {
        std::env::var("AC_MN_WORKER_FAULT")
            .ok()
            .filter(|s| !s.is_empty())
    }

    /// Writes this fixture process's PID so the test can verify the tree
    /// is really gone after the parent's teardown.
    fn fixture_pidfile() {
        if let Ok(path) = std::env::var("AC_MN_WORKER_FIXTURE_PIDFILE") {
            if !path.is_empty() {
                let _ = std::fs::write(&path, format!("{}", std::process::id()));
            }
        }
    }

    fn fixture_sleep() {
        std::thread::sleep(std::time::Duration::from_secs(600));
    }

    /// The pre-protocol fault modes (they never reach the real serving
    /// logic; each consumes the request line where the parent's write
    /// must complete first).
    fn serve_fault(mode: &str) -> ExitCode {
        use std::io::Write as _;
        let consume_request = || {
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
        };
        match mode {
            "no-stdin-read" => {
                fixture_sleep();
                ExitCode::FAILURE
            }
            "stdout-oversize-noeol" => {
                consume_request();
                let chunk = "x".repeat(8192);
                let mut out = std::io::stdout();
                for _ in 0..780 {
                    let _ = out.write_all(chunk.as_bytes());
                }
                let _ = out.flush();
                fixture_sleep();
                ExitCode::FAILURE
            }
            "stdout-oversize-line" => {
                consume_request();
                let chunk = "x".repeat(8192);
                let mut out = std::io::stdout();
                for _ in 0..780 {
                    let _ = out.write_all(chunk.as_bytes());
                }
                let _ = out.write_all(b"\n");
                let _ = out.flush();
                fixture_sleep();
                ExitCode::FAILURE
            }
            "stdout-flood-lines" => {
                consume_request();
                let mut n: u64 = 0;
                loop {
                    n += 1;
                    println!("{{\"unsolicited\":{n}}}");
                }
            }
            other => {
                eprintln!("mn-worker: unknown AC_MN_WORKER_FAULT `{other}`");
                ExitCode::from(2)
            }
        }
    }

    /// Locate (and on first use build) the landed ac-trace-support
    /// staticlib through the contained, deadline-bounded executor —
    /// missing tools are infra failures, never silent passes.
    fn tracelib() -> Result<Option<PathBuf>, String> {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
        let support = repo.join("experimental/ace/crates/trace-runtime");
        let lib = support.join("../../target/runtime-dev/ace_trace_runtime.lib");
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
        cmd.args(["build", "--locked", "--profile", "runtime-dev"])
            .current_dir(&support);
        let run =
            ace_core::link::run_deadlined_split(&mut cmd, std::time::Duration::from_secs(300))
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

    /// One incremental step's record (byte-exact streams included —
    /// F-818-R2-01: trace bytes ride every record, never dropped).
    /// PLAN-911: in `source-v1` mode the record also carries the typed
    /// refusal (kind/phase/snapshot) and the producer's body-work window;
    /// the default mode keeps the exact pre-911 key set.
    fn step_json(
        obs: &ace_native::incremental::EditObservation,
        op: &str,
        mode: DiagMode,
        step: Option<u64>,
    ) -> serde_json::Value {
        let mut v = serde_json::json!({
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
            "stderr_hex": hex(&obs.stderr),
            "stdout_hex": hex(&obs.stdout),
        });
        if mode == DiagMode::SourceV1 {
            if let Some(refusal) = &obs.refusal {
                let mut r = refusal.clone();
                if let (Some(se), Some(step)) = (r.source_error.as_mut(), step) {
                    se.operation.step = Some(step);
                }
                v["refusal"] = serde_json::to_value(&r).expect("refusal serializes");
            }
            v["body_work"] = body_work_json(&obs.body_work);
        }
        v
    }

    fn body_work_json(
        bw: &std::collections::BTreeMap<String, ace_core::body_work::BodyWork>,
    ) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        for (name, w) in bw {
            m.insert(
                name.clone(),
                serde_json::json!({
                    "resolve": w.resolve, "typecheck": w.typecheck,
                    "plan": w.plan, "bind": w.bind,
                }),
            );
        }
        serde_json::Value::Object(m)
    }

    /// PLAN-911: the explicit diagnostics observation mode of ONE request.
    /// `source-v1` turns on the typed fact channel; omitted keeps the
    /// pre-911 behavior bit-for-bit.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum DiagMode {
        Default,
        SourceV1,
    }

    fn parse_diag_mode(req: &serde_json::Value) -> Result<DiagMode, String> {
        match req.get("diagnostics") {
            None => Ok(DiagMode::Default),
            Some(v) if v.as_str() == Some(sd::SD_MODE) => Ok(DiagMode::SourceV1),
            Some(v) => Err(format!("unknown diagnostics mode {v}")),
        }
    }

    /// Strict-mode echo: EVERY ok container names the enabled mode/schema —
    /// legal programs included — so a worker that silently ignores the new
    /// field cannot pass as compatible (the parent rejects a missing echo).
    fn echo_mode(ok: &mut serde_json::Value, mode: DiagMode) {
        if mode == DiagMode::SourceV1 {
            ok["diagnostics_mode"] = serde_json::json!(sd::SD_MODE);
            ok["diagnostics_schema"] = serde_json::json!(sd::SD_SCHEMA);
        }
    }

    /// Shape ② — a REAL source rejection: the typed snapshot at the top
    /// level, the legacy projection kept as `diagnostic`, and no execution
    /// result anywhere in the response.
    fn attach_source_error(resp: &mut serde_json::Value, snap: &sd::SourceDiagnosticSnapshot) {
        resp["source_error"] = snap.to_json();
        resp["diagnostic"] = serde_json::json!({
            "code": "mn.unsupported.frontend",
            "message": snap.legacy_message(),
        });
    }

    /// Exactly-once literal replacement on `cur` (the shared edit oracle:
    /// ambiguity is an error, never a silent first-hit edit).
    fn replacen_once(cur: &mut String, from: &str, to: &str) -> Result<(), String> {
        let hits = cur.matches(from).count();
        if from.is_empty() || hits != 1 {
            return Err(format!(
                "edit-source literal matched {hits} times (expected once)"
            ));
        }
        *cur = cur.replacen(from, to, 1);
        Ok(())
    }

    /// The incremental-session path (F-818-R1-03): one warm session per
    /// request; edit-source steps are applied as single-body transactions
    /// (literal located exactly once in the CURRENT source); every step
    /// really links and runs. The judged value is the LAST step's exit.
    ///
    /// PLAN-911 `source-v1`: a warm-up frontend rejection is the request's
    /// typed snapshot (shape ②); a typed edit refusal STOPS the sequence at
    /// the first failure (the accepted prefix is reported, later steps are
    /// never executed); only `syntax-not-converged` still falls back — and
    /// the fallback gates every materialized candidate's semantics one
    /// edit at a time instead of applying the whole tail first.
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
        mode: DiagMode,
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
        let (mut session, cold) = match ace_native::incremental::MnIncrementalSession::warmup(
            &source,
            &entry,
            &caps,
            &store_path,
            tracelib.as_deref(),
        ) {
            Ok(pair) => pair,
            Err(e) => {
                // PLAN-911: a warm-up frontend rejection is the typed fact
                // (shape ②), not an incremental-infra error.
                if let (DiagMode::SourceV1, Some(snap)) = (mode, e.source_diagnostic()) {
                    let mut ok = serde_json::json!({
                        "recipe": "ac/mir-native-scalar-v1",
                        "worker_version": MN_WORKER_VERSION,
                        "mn_abi": ace_native::config::MN_ABI,
                        "mn_target": ace_native::config::MN_TARGET,
                        "incremental": true,
                        "steps": [step_json(&cold_stub(), "warmup", mode, None)],
                        "counters": {},
                        "spans": {},
                    });
                    echo_mode(&mut ok, mode);
                    let mut resp = echo;
                    resp["ok"] = ok;
                    attach_source_error(&mut resp, snap);
                    println!("{resp}");
                    return ExitCode::SUCCESS;
                }
                return fail(echo, "mn.worker.incremental", e.message());
            }
        };
        let mut step_records: Vec<serde_json::Value> = Vec::new();
        step_records.push(step_json(&cold, "warmup", mode, None));
        // F-818-R2-01: the judged streams are the judged round's REAL PE
        // bytes — authorized trace bytes survive the incremental path.
        let mut judged_streams = (cold.stdout.clone(), cold.stderr.clone());
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
                    if mode == DiagMode::SourceV1 {
                        if let Some(refusal) = &obs.refusal {
                            if refusal.kind == sd::RefusalKind::SyntaxNotConverged {
                                // A LEGAL edit the engine cannot converge on:
                                // the explicit fallback, now per-candidate.
                                return serve_incremental_strict_fallback(
                                    echo,
                                    &session_source(&session),
                                    steps[st_idx..].to_vec(),
                                    from,
                                    to,
                                    &entry,
                                    &caps,
                                    &store_root,
                                    &workdir,
                                    tracelib.as_deref(),
                                    st_idx,
                                );
                            }
                            // First failure: STOP. Later steps are not
                            // executed; the accepted prefix is this
                            // session's current source.
                            let accepted = session_source(&session);
                            step_records.push(step_json(
                                &obs,
                                "edit-source",
                                mode,
                                Some(st_idx as u64),
                            ));
                            session.close();
                            let mut ok = serde_json::json!({
                                "recipe": "ac/mir-native-scalar-v1",
                                "worker_version": MN_WORKER_VERSION,
                                "mn_abi": ace_native::config::MN_ABI,
                                "mn_target": ace_native::config::MN_TARGET,
                                "incremental": true,
                                "steps": step_records,
                                "accepted_prefix": {
                                    "steps_applied": st_idx,
                                    "source_sha256": sd::sha256_hex(accepted.as_bytes()),
                                },
                                "counters": {},
                                "spans": {},
                            });
                            echo_mode(&mut ok, mode);
                            let mut resp = echo;
                            match &refusal.source_error {
                                Some(snap) => attach_source_error(&mut resp, snap),
                                None => {
                                    resp["diagnostic"] = serde_json::json!({
                                        "code": "mn.incremental.refused",
                                        "message": refusal.reason,
                                    });
                                }
                            }
                            resp["ok"] = ok;
                            println!("{resp}");
                            return ExitCode::SUCCESS;
                        }
                    } else if let Some(r) = &obs.refused_reason {
                        if r.contains("syntax fallback") {
                            // The 828 syntax increment engine cannot
                            // converge on THIS edit shape (a LEGAL edit,
                            // an engine limitation): fall back to the
                            // WHOLE-REQUEST path over the fully edited
                            // source — recorded, never labeled incremental.
                            // F-818-R2-02: the accepted prefix rides in the
                            // SESSION'S CURRENT source (every edit committed
                            // so far), never the original request source.
                            return serve_incremental_whole_fallback(
                                echo,
                                &session_source(&session),
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
                    judged_streams = (obs.stdout.clone(), obs.stderr.clone());
                    step_records.push(step_json(&obs, "edit-source", mode, Some(st_idx as u64)));
                }
                Some("run") => {
                    let obs = session.run_only();
                    if let Some(i) = &obs.infra {
                        return fail(echo, "mn.worker.incremental", i.clone());
                    }
                    judged_exit = obs.exit_code;
                    judged_streams = (obs.stdout.clone(), obs.stderr.clone());
                    step_records.push(step_json(&obs, "run", mode, Some(st_idx as u64)));
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
            "mn_abi": ace_native::config::MN_ABI,
            "mn_target": ace_native::config::MN_TARGET,
            "incremental": true,
            "steps": step_records,
        });
        echo_mode(&mut ok, mode);
        ok["counters"] = serde_json::json!({});
        ok["spans"] = serde_json::json!({});
        let mut resp = echo;
        if let Some((code, message)) = refused {
            resp["diagnostic"] = serde_json::json!({"code": code, "message": message});
        } else {
            ok["exit_code"] = serde_json::json!(judged_exit);
            ok["stderr_hex"] = serde_json::json!(hex(&judged_streams.1));
            ok["stdout_hex"] = serde_json::json!(hex(&judged_streams.0));
        }
        resp["ok"] = ok;
        println!("{resp}");
        ExitCode::SUCCESS
    }

    /// A warm-up that never produced an observation (its source rejected):
    /// the step record still names the op — with no execution result.
    fn cold_stub() -> ace_native::incremental::EditObservation {
        Default::default()
    }

    /// PLAN-911 strict fallback (syntax-not-converged on a LEGAL edit):
    /// materialize the remaining edits ONE candidate at a time; every
    /// candidate except the last is gated by ONE real frontend compile
    /// (`check_source` — zero codegen/link/run); the first semantic
    /// failure stops the sequence at THAT candidate (shape ②, operation
    /// `fallback-cold`, step index, base + candidate SHAs). Only when
    /// every candidate passes does ONE whole request over the fully
    /// materialized source link and run (the judged result). The tail is
    /// never applied past a rejected candidate.
    #[allow(clippy::too_many_arguments)]
    fn serve_incremental_strict_fallback(
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
        start_step: usize,
    ) -> ExitCode {
        let fail = |echo: serde_json::Value, code: &str, message: String| -> ExitCode {
            let mut r = echo;
            r["error"] = serde_json::json!({"code": code, "message": message});
            println!("{r}");
            ExitCode::FAILURE
        };
        let mut driver =
            match ace_native::driver::MnDriver::with_store(&PathBuf::from(store_root), "mn-case") {
                Ok(d) => d,
                Err(e) => return fail(echo.clone(), "mn.worker.store", e.message()),
            };
        let mut cur = source.to_string();
        let mut last_edit_idx = start_step;
        let mut materialized = 0usize;
        let mut gate_records: Vec<serde_json::Value> = Vec::new();
        for (i, st) in remaining.iter().enumerate() {
            let step_idx = start_step + i;
            match st.get("op").and_then(|v| v.as_str()) {
                Some("edit-source") => {
                    let from = if i == 0 {
                        refused_from
                    } else {
                        st.get("from").and_then(|v| v.as_str()).unwrap_or("")
                    };
                    let to = if i == 0 {
                        refused_to
                    } else {
                        st.get("to").and_then(|v| v.as_str()).unwrap_or("")
                    };
                    let base = cur.clone();
                    if let Err(e) = replacen_once(&mut cur, from, to) {
                        return fail(echo.clone(), "mn.worker.edit", e);
                    }
                    // The last edit's semantics are proven by the final
                    // whole request that runs it; earlier candidates get
                    // their own one-compile gate.
                    let more_edits = remaining[i + 1..]
                        .iter()
                        .any(|s| s.get("op").and_then(|v| v.as_str()) == Some("edit-source"));
                    if more_edits {
                        if let Err(e) = driver.check_source(&cur, caps) {
                            if let Some(snap) = e.source_diagnostic() {
                                let mut snap = snap.clone();
                                snap.operation = sd::Operation::fallback_cold(&base, &cur)
                                    .with_step(step_idx as u64);
                                let mut ok = serde_json::json!({
                                    "recipe": "ac/mir-native-scalar-v1",
                                    "worker_version": MN_WORKER_VERSION,
                                    "mn_abi": ace_native::config::MN_ABI,
                                    "mn_target": ace_native::config::MN_TARGET,
                                    "warm_edit_fallback": true,
                                    "steps": gate_records,
                                    "accepted_prefix": {
                                        "steps_applied": step_idx,
                                        "source_sha256": sd::sha256_hex(base.as_bytes()),
                                    },
                                    "counters": {},
                                    "spans": {},
                                });
                                echo_mode(&mut ok, DiagMode::SourceV1);
                                let mut resp = echo;
                                attach_source_error(&mut resp, &snap);
                                resp["ok"] = ok;
                                println!("{resp}");
                                return ExitCode::SUCCESS;
                            }
                            return fail(echo.clone(), "mn.worker.incremental", e.message());
                        }
                        gate_records.push(serde_json::json!({
                            "op": "edit-source", "warm_edit_fallback": true,
                            "materialized": true, "semantic_gate": "pass",
                            "step": step_idx,
                        }));
                    } else {
                        gate_records.push(serde_json::json!({
                            "op": "edit-source", "warm_edit_fallback": true,
                            "materialized": true, "semantic_gate": "final",
                            "step": step_idx,
                        }));
                    }
                    materialized += 1;
                    last_edit_idx = step_idx;
                }
                Some("run") => {}
                _ => {
                    return fail(
                        echo.clone(),
                        "mn.worker.steps",
                        "unsupported op in the strict fallback path".to_string(),
                    );
                }
            }
        }
        let _ = materialized;
        // Every candidate passed: ONE whole request over the fully
        // materialized source is the judged execution.
        let r = driver.request(&cur, entry, caps, &PathBuf::from(workdir), tracelib);
        let mut ok = serde_json::json!({
            "recipe": "ac/mir-native-scalar-v1",
            "worker_version": MN_WORKER_VERSION,
            "mn_abi": ace_native::config::MN_ABI,
            "mn_target": ace_native::config::MN_TARGET,
            "warm_edit_fallback": true,
            "steps": gate_records,
            "counters": {},
            "spans": {},
        });
        echo_mode(&mut ok, DiagMode::SourceV1);
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
            if let Some(snap) = e.source_diagnostic() {
                let mut snap = snap.clone();
                snap.operation.step = Some(last_edit_idx as u64);
                resp["ok"] = ok;
                attach_source_error(&mut resp, &snap);
            } else {
                resp["ok"] = ok;
                resp["diagnostic"] = serde_json::json!({"code": e.code(), "message": e.message()});
            }
        } else {
            return fail(echo, "mn.worker.internal", "inconsistent result".into());
        }
        println!("{resp}");
        ExitCode::SUCCESS
    }

    /// The whole-request FALLBACK for syntax-increment non-convergence
    /// (default mode, unchanged pre-911 behavior): materialize the refused
    /// edit (plus every remaining edit step) on the ORIGINAL source and
    /// run ONE full MnDriver request. The response is the legacy
    /// whole-request shape plus `warm_edit_fallback: true` — never labeled
    /// incremental (F-818-R1-03).
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
        if let Err(e) = replacen_once(&mut cur, refused_from, refused_to) {
            return fail("mn.worker.edit", e);
        }
        for st in remaining.iter().skip(1) {
            if st.get("op").and_then(|v| v.as_str()) == Some("edit-source") {
                let from = st.get("from").and_then(|v| v.as_str()).unwrap_or("");
                let to = st.get("to").and_then(|v| v.as_str()).unwrap_or("");
                if let Err(e) = replacen_once(&mut cur, from, to) {
                    return fail("mn.worker.edit", e);
                }
            }
        }
        let mut driver =
            match ace_native::driver::MnDriver::with_store(&PathBuf::from(store_root), "mn-case") {
                Ok(d) => d,
                Err(e) => return fail("mn.worker.store", e.message()),
            };
        let r = driver.request(&cur, entry, caps, &PathBuf::from(workdir), tracelib);
        let mut ok = serde_json::json!({
            "recipe": "ac/mir-native-scalar-v1",
            "worker_version": MN_WORKER_VERSION,
            "mn_abi": ace_native::config::MN_ABI,
            "mn_target": ace_native::config::MN_TARGET,
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

    /// The ROUNDS mode (F-818-R1-01 parent-observed timing gate): ONE
    /// persistent worker process carries ONE warm incremental session;
    /// the parent drives ONE step per request line and reads ONE response
    /// line per step. The parent therefore observes the CONTINUOUS
    /// per-edit interval — its own request preparation (source copy +
    /// literal search/insert + payload serialization), IPC both ways,
    /// this side's locate/edit/link/run, response serialization, and
    /// parent admission. Protocol lines (single-line JSON each):
    ///   parent → worker: the initial request with `"mode":"rounds"` (no
    ///                     steps); the worker answers the warmup record;
    ///   per step: `{"op":"edit-source","from","to","nonce"}` or
    ///             `{"op":"run","nonce"}` or `{"op":"close"}` — one
    ///             response line echoing the round nonce with the record.
    /// A syntax-fallback edit switches to MATERIALITZATION mode: the
    /// accepted prefix is the session's CURRENT source (F-818-R2-02),
    /// edits materialize exactly once, and the trailing run executes ONE
    /// whole-request pipeline over the fully edited source (recorded as
    /// warm_edit_fallback, never labeled incremental).
    ///
    /// PLAN-911 `source-v1`: the warmup echo carries the mode/schema; a
    /// typed refusal answers its round with the refusal object (and the
    /// snapshot for language diagnostics, inside `ok`); the strict
    /// materialization mode gates each candidate's semantics with one
    /// real frontend compile — a rejected candidate is reported and the
    /// accepted prefix is retained (never silently materialized past).
    #[allow(clippy::too_many_arguments)]
    fn serve_rounds(
        echo: serde_json::Value,
        source: String,
        entry: String,
        caps: Vec<String>,
        store_root: String,
        workdir: String,
        tracelib: Option<PathBuf>,
        mode: DiagMode,
    ) -> ExitCode {
        let respond = |v: serde_json::Value| {
            println!("{v}");
            use std::io::Write as _;
            let _ = std::io::stdout().flush();
        };
        let fail_round = |nonce: &str, code: &str, message: String| -> ExitCode {
            let mut r = serde_json::json!({
                "schema": MN_WORKER_PROTOCOL,
                "nonce": nonce,
            });
            r["error"] = serde_json::json!({"code": code, "message": message});
            respond(r);
            ExitCode::FAILURE
        };
        let mut store_path = PathBuf::from(&store_root);
        if store_path
            .components()
            .last()
            .map(|c| c.as_os_str() == "mn-store")
            .unwrap_or(false)
        {
            store_path = workdir.clone().into();
        }
        store_path.push("mn-inc-store");
        let (mut session, cold) = match ace_native::incremental::MnIncrementalSession::warmup(
            &source,
            &entry,
            &caps,
            &store_path,
            tracelib.as_deref(),
        ) {
            Ok(pair) => pair,
            Err(e) => {
                if let (DiagMode::SourceV1, Some(snap)) = (mode, e.source_diagnostic()) {
                    let mut warm = serde_json::json!({});
                    echo_mode(&mut warm, mode);
                    let mut r = serde_json::json!({
                        "schema": MN_WORKER_PROTOCOL,
                        "nonce": "",
                    });
                    attach_source_error(&mut r, snap);
                    r["ok"] = warm;
                    respond(r);
                    return ExitCode::SUCCESS;
                }
                return fail_round("", "mn.worker.incremental", e.message());
            }
        };
        let mut warm_ok = step_json(&cold, "warmup", mode, None);
        warm_ok["recipe"] = serde_json::json!("ac/mir-native-scalar-v1");
        warm_ok["worker_version"] = serde_json::json!(MN_WORKER_VERSION);
        warm_ok["mn_abi"] = serde_json::json!(ace_native::config::MN_ABI);
        warm_ok["mn_target"] = serde_json::json!(ace_native::config::MN_TARGET);
        echo_mode(&mut warm_ok, mode);
        // Test-only fixture faults that need the REAL serving logic (the
        // echoes stay valid; only the faulted dimension misbehaves).
        match fixture_fault().as_deref() {
            // Valid boundary control: pad the warmup RESPONSE LINE to just
            // under the parent's line cap (budget covers the echo wrapper
            // too) — the parent must still admit it.
            Some("stdout-line-under-cap") => {
                let base = serde_json::to_string(&warm_ok).unwrap_or_default().len();
                let pad = 4_131_840usize.saturating_sub(base + 64);
                warm_ok["fixture_pad"] = serde_json::json!("p".repeat(pad));
            }
            _ => {}
        }
        let mut warm = echo;
        warm["ok"] = warm_ok;
        respond(warm);
        match fixture_fault().as_deref() {
            // Liveness fixture: answer warmup, then never read again —
            // the parent's step receive must hit its absolute wall.
            Some("stop-after-warmup") => {
                fixture_sleep();
                return ExitCode::FAILURE;
            }
            // Cap fixtures: the protocol stays valid throughout (stdout
            // answers every round); stderr floods on a BACKGROUND thread
            // so the main loop keeps serving — the parent must propagate
            // the stderr cap even while stdout looks perfectly healthy
            // (over-cap) / stay green just under the cap (valid control).
            Some("stderr-overcap") | Some("stderr-under-cap") => {
                let over = fixture_fault().as_deref() == Some("stderr-overcap");
                std::thread::spawn(move || {
                    let size = if over { 2 * 1024 * 1024 } else { 900 * 1024 };
                    eprint!("{}", "e".repeat(size));
                    let _ = std::io::Write::flush(&mut std::io::stderr());
                });
            }
            _ => {}
        }
        // Materialization state after a syntax fallback: the accepted
        // prefix is the session's CURRENT source, the refused edit (and
        // every later edit) materializes exactly once on it. In strict
        // mode each materialized candidate is gated by one real frontend
        // compile; a rejected candidate keeps the accepted prefix.
        let mut fallback_src: Option<String> = None;
        let mut gate_driver =
            ace_native::driver::MnDriver::with_store(&PathBuf::from(&store_root), "mn-case");
        let stdin = std::io::stdin();
        let mut lock = stdin.lock();
        let mut line = String::new();
        let mut round_no: u64 = 0;
        loop {
            line.clear();
            match lock.read_line(&mut line) {
                Ok(0) => break, // EOF = close
                Ok(_) if line.trim().is_empty() => continue,
                Ok(_) => {}
                Err(e) => return fail_round("", "mn.worker.rounds", format!("read step: {e}")),
            }
            let st: serde_json::Value = match serde_json::from_str(line.trim()) {
                Ok(v) => v,
                Err(e) => return fail_round("", "mn.worker.rounds", format!("bad step line: {e}")),
            };
            let nonce = st
                .get("nonce")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            round_no += 1;
            let mut resp = serde_json::json!({
                "schema": MN_WORKER_PROTOCOL,
                "nonce": nonce,
            });
            match st.get("op").and_then(|v| v.as_str()) {
                Some("close") => {
                    match fixture_fault().as_deref() {
                        // Liveness fixtures: a valid close reply without an
                        // exit, and an ignored close — the parent's reap
                        // and close-handshake reads must stay under its
                        // absolute wall.
                        Some("close-without-exit") => {
                            session.close();
                            resp["ok"] = serde_json::json!({"op": "close"});
                            respond(resp);
                            fixture_sleep();
                            return ExitCode::FAILURE;
                        }
                        Some("ignore-close") => {
                            fixture_sleep();
                            return ExitCode::FAILURE;
                        }
                        _ => {}
                    }
                    session.close();
                    resp["ok"] = serde_json::json!({"op": "close"});
                    respond(resp);
                    return ExitCode::SUCCESS;
                }
                Some("edit-source") => {
                    let from = st.get("from").and_then(|v| v.as_str()).unwrap_or("");
                    let to = st.get("to").and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(src) = fallback_src.as_mut() {
                        let base = src.clone();
                        if let Err(e) = replacen_once(src, from, to) {
                            return fail_round(&nonce, "mn.worker.edit", e);
                        }
                        if mode == DiagMode::SourceV1 {
                            // Strict gate: one real frontend compile of THIS
                            // candidate; a failure keeps the accepted prefix.
                            let driver = match gate_driver.as_mut() {
                                Ok(d) => d,
                                Err(e) => {
                                    return fail_round(&nonce, "mn.worker.store", e.message());
                                }
                            };
                            if let Err(e) = driver.check_source(src, &caps) {
                                if let Some(snap) = e.source_diagnostic() {
                                    let mut snap = snap.clone();
                                    snap.operation = sd::Operation::fallback_cold(&base, src)
                                        .with_step(round_no);
                                    let mut ok = serde_json::json!({
                                        "op": "edit-source",
                                        "warm_edit_fallback": true,
                                        "materialized": false,
                                    });
                                    echo_mode(&mut ok, mode);
                                    attach_source_error(&mut resp, &snap);
                                    resp["ok"] = ok;
                                    // Restore the accepted prefix.
                                    *src = base;
                                    respond(resp);
                                    continue;
                                }
                                return fail_round(&nonce, "mn.worker.incremental", e.message());
                            }
                        }
                        resp["ok"] = serde_json::json!({
                            "op": "edit-source",
                            "warm_edit_fallback": true,
                            "materialized": true,
                        });
                        respond(resp);
                        continue;
                    }
                    let cur = session_source(&session);
                    let hits = cur.matches(from).count();
                    if from.is_empty() || hits != 1 {
                        return fail_round(
                            &nonce,
                            "mn.worker.edit",
                            format!("edit-source literal matched {hits} times (expected once)"),
                        );
                    }
                    let start = cur.find(from).expect("one hit") as u64;
                    let end = start + from.len() as u64;
                    let obs = session.edit(start, end, to.as_bytes().to_vec());
                    if let Some(i) = &obs.infra {
                        return fail_round(&nonce, "mn.worker.incremental", i.clone());
                    }
                    if let Some(r) = &obs.refused_reason {
                        if r.contains("syntax fallback") {
                            let mut src = cur.clone();
                            if let Err(e) = replacen_once(&mut src, from, to) {
                                return fail_round(&nonce, "mn.worker.edit", e);
                            }
                            // Strict: the FIRST materialized candidate is
                            // gated too (a legal edit that cannot converge
                            // may still be semantically illegal).
                            if mode == DiagMode::SourceV1 {
                                let driver = match gate_driver.as_mut() {
                                    Ok(d) => d,
                                    Err(e) => {
                                        return fail_round(&nonce, "mn.worker.store", e.message());
                                    }
                                };
                                if let Err(e) = driver.check_source(&src, &caps) {
                                    if let Some(snap) = e.source_diagnostic() {
                                        let mut snap = snap.clone();
                                        snap.operation = sd::Operation::fallback_cold(&cur, &src)
                                            .with_step(round_no);
                                        let mut ok = serde_json::json!({
                                            "op": "edit-source",
                                            "warm_edit_fallback": true,
                                            "materialized": false,
                                            "refused": r,
                                        });
                                        echo_mode(&mut ok, mode);
                                        attach_source_error(&mut resp, &snap);
                                        resp["ok"] = ok;
                                        respond(resp);
                                        continue;
                                    }
                                    return fail_round(
                                        &nonce,
                                        "mn.worker.incremental",
                                        e.message(),
                                    );
                                }
                            }
                            fallback_src = Some(src);
                            resp["ok"] = serde_json::json!({
                                "op": "edit-source",
                                "warm_edit_fallback": true,
                                "materialized": true,
                                "refused": r,
                            });
                            respond(resp);
                            continue;
                        }
                        if mode == DiagMode::SourceV1 {
                            // A typed refusal answers its own round; the
                            // session keeps the accepted prefix (the
                            // parent stops the sequence — V-16).
                            let mut ok = step_json(&obs, "edit-source", mode, Some(round_no));
                            echo_mode(&mut ok, mode);
                            if let Some(snap) =
                                obs.refusal.as_ref().and_then(|f| f.source_error.as_ref())
                            {
                                let mut snap = snap.clone();
                                snap.operation.step = Some(round_no);
                                attach_source_error(&mut resp, &snap);
                            }
                            resp["ok"] = ok;
                            respond(resp);
                            continue;
                        }
                        resp["ok"] = step_json(&obs, "edit-source", mode, Some(round_no));
                        respond(resp);
                        continue;
                    }
                    round_no += 1;
                    resp["ok"] = step_json(&obs, "edit-source", mode, Some(round_no));
                    respond(resp);
                }
                Some("run") => {
                    if let Some(src) = &fallback_src {
                        // ONE whole-request pipeline over the fully edited
                        // source (prefix preserved, recorded as fallback).
                        let mut driver = match ace_native::driver::MnDriver::with_store(
                            &PathBuf::from(&store_root),
                            "mn-case",
                        ) {
                            Ok(d) => d,
                            Err(e) => return fail_round(&nonce, "mn.worker.store", e.message()),
                        };
                        let r = driver.request(
                            src,
                            &entry,
                            &caps,
                            &PathBuf::from(&workdir),
                            tracelib.as_deref(),
                        );
                        let mut ok = serde_json::json!({
                            "op": "run",
                            "warm_edit_fallback": true,
                            "recipe": "ac/mir-native-scalar-v1",
                            "worker_version": MN_WORKER_VERSION,
                            "mn_abi": ace_native::config::MN_ABI,
                            "mn_target": ace_native::config::MN_TARGET,
                            "counters": {},
                            "spans": {},
                        });
                        echo_mode(&mut ok, mode);
                        for (k, v) in &r.obs.counters {
                            ok["counters"][k] = serde_json::json!(v);
                        }
                        for (k, v) in &r.obs.spans {
                            ok["spans"][k] = serde_json::json!(v);
                        }
                        if let Some(fp) = &r.manifest_fp {
                            ok["manifest_fp"] = serde_json::json!(fp);
                        }
                        if r.ok {
                            ok["exit_code"] = serde_json::json!(r.exit_code);
                            ok["stderr_hex"] = serde_json::json!(hex(&r.stderr));
                            ok["stdout_hex"] = serde_json::json!(hex(&r.stdout));
                        } else if let Some(e) = &r.error {
                            if let (DiagMode::SourceV1, Some(snap)) = (mode, e.source_diagnostic())
                            {
                                let mut snap = snap.clone();
                                snap.operation.step = Some(round_no);
                                attach_source_error(&mut resp, &snap);
                            } else {
                                ok["diagnostic"] =
                                    serde_json::json!({"code": e.code(), "message": e.message()});
                            }
                        } else {
                            return fail_round(
                                &nonce,
                                "mn.worker.internal",
                                "inconsistent result".into(),
                            );
                        }
                        resp["ok"] = ok;
                        respond(resp);
                        continue;
                    }
                    let obs = session.run_only();
                    if let Some(i) = &obs.infra {
                        return fail_round(&nonce, "mn.worker.incremental", i.clone());
                    }
                    resp["ok"] = step_json(&obs, "run", mode, Some(round_no));
                    respond(resp);
                }
                _ => {
                    return fail_round(
                        &nonce,
                        "mn.worker.steps",
                        "unsupported op in rounds mode".to_string(),
                    )
                }
            }
        }
        session.close();
        ExitCode::SUCCESS
    }

    /// The session's current source (diagnostics seam for the edit
    /// locator; the production path reads the 828 session).
    fn session_source(session: &ace_native::incremental::MnIncrementalSession) -> String {
        session.source().to_string()
    }

    pub fn serve() -> ExitCode {
        fixture_pidfile();
        if let Some(mode) = fixture_fault() {
            // Only the PRE-PROTOCOL fixtures hijack serving; the
            // mid-protocol modes (stop-after-warmup / close faults /
            // stderr caps / the under-cap line pad) ride the REAL serving
            // path (serve_rounds injects them at their points).
            if matches!(
                mode.as_str(),
                "no-stdin-read"
                    | "stdout-oversize-noeol"
                    | "stdout-oversize-line"
                    | "stdout-flood-lines"
            ) {
                return serve_fault(&mode);
            }
        }
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
        if recipe == "ac/mir-native-typed-v1" {
            return typed_case_run::serve_typed(req, echo);
        }
        if recipe != "ac/mir-native-scalar-v1" {
            return fail(
                echo,
                "mn.worker.recipe",
                format!("recipe `{recipe}` is not ac/mir-native-scalar-v1"),
            );
        }
        // PLAN-911: the explicit diagnostics mode belongs to the scalar
        // recipe only; an unknown value is a named request error.
        let diag_mode = match parse_diag_mode(&req) {
            Ok(m) => m,
            Err(e) => return fail(echo, "mn.worker.request", e),
        };
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

        // F-818-R1-01: the ROUNDS protocol — one persistent session, one
        // step per request/response line — so the PARENT observes the
        // continuous per-edit interval (its preparation through its
        // admission of each round's result).
        if req.get("mode").and_then(|v| v.as_str()) == Some("rounds") {
            return case_run::serve_rounds(
                echo,
                source,
                entry,
                caps,
                store_root.to_string(),
                workdir.to_string(),
                tracelib,
                diag_mode,
            );
        }

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
                diag_mode,
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
            ace_native::link_driver::set_tool_disk_memo(memo);
        }
        let mut driver =
            match ace_native::driver::MnDriver::with_store(&PathBuf::from(store_root), "mn-case") {
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
            "mn_abi": ace_native::config::MN_ABI,
            "mn_target": ace_native::config::MN_TARGET,
            "counters": {},
            "spans": {},
        });
        echo_mode(&mut ok, diag_mode);
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
                // PLAN-911: in strict mode a REAL source rejection is the
                // typed snapshot (shape ②); the legacy projection rides
                // `diagnostic` for default consumers.
                let mut resp = echo;
                resp["ok"] = ok;
                if let (DiagMode::SourceV1, Some(snap)) = (diag_mode, e.source_diagnostic()) {
                    attach_source_error(&mut resp, snap);
                } else {
                    resp["diagnostic"] =
                        serde_json::json!({"code": e.code(), "message": e.message()});
                }
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
        "collections-restore-run" => {
            let (Some(store), Some(workdir), Some(id)) =
                (flag("--store"), flag("--workdir"), flag("--manifest-id"))
            else {
                eprintln!("collections-restore-run requires --store, --workdir and --manifest-id");
                return ExitCode::from(2);
            };
            let project = flag("--project").unwrap_or_else(|| "mn.collections".into());
            let store = match ace_artifacts::store::ManagedStore::open_existing(Path::new(&store)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("collections restore store: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let mut driver = ace_native::driver::MnDriver::with_managed_store(store, &project);
            driver.forbid_codegen = true;
            if args.iter().any(|s| s == "--read-only") {
                driver.cache_mode = ace_native::driver::CacheMode::ReadOnly;
            }
            let support = flag("--support-lib").map(PathBuf::from);
            let result =
                driver.request_restored_collections(&id, Path::new(&workdir), support.as_deref());
            let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
            println!(
                "{}",
                serde_json::json!({"protocol":"mncollections-restore/1","ok":result.ok,
                "error":result.error.as_ref().map(|e|e.code()),"message":result.error.as_ref().map(|e|e.message()),
                "exit_code":result.exit_code,"stdout_hex":hex(&result.stdout),"stderr_hex":hex(&result.stderr),
                "manifest_fp":result.manifest_fp,"manifest_artifact_id":result.manifest_artifact_id,
                "counters":result.obs.counters,"spans":result.obs.spans})
            );
            if result.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        // The frozen reference workload (golden.md §2) — single source of
        // truth for the formal turnaround scenarios (the 798 `load-revision`
        // form): each revision bumps the frozen fn_041 body-edit bit.
        "load-revision" => {
            let rev: i64 = flag("--rev").and_then(|v| v.parse().ok()).unwrap_or(0);
            let wl = ace_native::workload::generate(rev);
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
                ace_native::worker::restore_run_by_id(
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
                let text_lib_sha = flag("--textlibsha");
                ace_native::worker::restore_run_by_key(
                    &PathBuf::from(store),
                    &owner_module,
                    &entry,
                    &unit_fps,
                    &link_inputs,
                    &PathBuf::from(workdir),
                    trace_lib.as_deref(),
                    text_lib_sha.as_deref(),
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

// ---- PLAN-891 typed case-run (ac/mir-native-typed-v1) --------------------
//
// The formal typed recipe: the parent sends the file-ized fixture content
// (mn-typed-fixture/1), the entry name and the actual run arguments as
// canonical (kind, bits) pairs; the worker runs the real admission chain
// (fixture parse -> shared verify_mir -> split -> typed admission -> unit
// COFFs -> cached wrapper -> one real PE per run) and reports ONLY what
// the machine returned (value kind + raw bits, or a named trap/resource/
// error frame). Expected values never enter the request; judgement is
// entirely parent-side.
mod typed_case_run {
    use super::case_run::{MN_WORKER_PROTOCOL, MN_WORKER_VERSION};

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
    use ace_artifacts::store::ManagedStore;
    use ace_native::typed_entry::{self, TypedArg, TypedKind, TypedOutcome};
    use ace_native::typed_fixture::{self, FixtureDoc};
    use ace_native::typed_pipeline::{self, TypedCacheMode};
    use ace_native::typed_session::TypedSession;
    use std::io::{BufRead, Write};
    use std::path::PathBuf;
    use std::process::ExitCode;

    pub const TYPED_RECIPE: &str = "ac/mir-native-typed-v1";

    fn typed_json(run: &ace_native::typed_pipeline::TypedRun) -> serde_json::Value {
        let mut t = serde_json::json!({
            "status": match &run.outcome {
                TypedOutcome::Value { .. } => "value",
                TypedOutcome::Trap { .. } => "trap",
                TypedOutcome::Resource { .. } => "resource",
                TypedOutcome::Error { .. } => "error",
            },
            "nonce": run.echoed_nonce,
            "sig_id": run.echoed_sig_id,
        });
        match &run.outcome {
            TypedOutcome::Value { kind_code, bits } => {
                t["kind"] =
                    serde_json::json!(typed_entry::kind_label_of(*kind_code).unwrap_or("?"));
                t["bits"] = serde_json::json!(typed_entry::canonical_hex(*kind_code, *bits));
            }
            TypedOutcome::Trap { reason }
            | TypedOutcome::Resource { reason }
            | TypedOutcome::Error { reason } => {
                t["reason"] = serde_json::json!(reason);
            }
        }
        t
    }

    fn counters_json(c: &ace_native::typed_pipeline::TypedCounters) -> serde_json::Value {
        serde_json::json!({
            "verify_mir": c.verify_mir,
            "unit_prepare": c.unit_prepare,
            "unit_fp_render": c.unit_fp_render,
            "native_admit": c.native_admit,
            "emit_units": c.emit_units,
            "persist_units": c.persist_units,
            "cache_hits": c.cache_hits,
            "wrapper_builds": c.wrapper_builds,
            "wrapper_hits": c.wrapper_hits,
            "link_invocations": c.link_invocations,
            "run_invocations": c.run_invocations,
            "whole_fallback": c.whole_fallback,
        })
    }

    fn parse_args(v: Option<&serde_json::Value>) -> Result<Vec<TypedArg>, String> {
        let mut out = Vec::new();
        let Some(list) = v.and_then(|a| a.as_array()) else {
            return Ok(out);
        };
        for pair in list {
            let arr = pair
                .as_array()
                .ok_or("arg entry is not a [kind,bits] pair")?;
            if arr.len() != 2 {
                return Err("arg entry is not a [kind,bits] pair".into());
            }
            let kind = arr[0].as_str().ok_or("arg kind is not a string")?;
            let bits = arr[1].as_str().ok_or("arg bits is not a string")?;
            let k = TypedKind::from_label(kind).ok_or(format!("unknown arg kind `{kind}`"))?;
            out.push(typed_entry::parse_arg(k, bits).map_err(|e| e.message())?);
        }
        Ok(out)
    }

    pub fn serve_typed(req: serde_json::Value, echo: serde_json::Value) -> ExitCode {
        let fail = |echo: serde_json::Value, code: &str, message: String| -> ExitCode {
            let mut r = echo;
            r["error"] = serde_json::json!({"code": code, "message": message});
            println!("{r}");
            ExitCode::FAILURE
        };
        use base64::Engine as _;
        let Some(fixture_b64) = req.get("typed_fixture_b64").and_then(|v| v.as_str()) else {
            return fail(
                echo,
                "mn.worker.request",
                "missing typed_fixture_b64".into(),
            );
        };
        let Ok(fixture_bytes) = base64::engine::general_purpose::STANDARD.decode(fixture_b64)
        else {
            return fail(
                echo,
                "mn.worker.request",
                "typed_fixture_b64 is not valid base64".into(),
            );
        };
        let doc = match typed_fixture::parse_fixture(&fixture_bytes) {
            Ok(d) => d,
            Err(e) => return fail(echo, "mn.worker.fixture", e.message()),
        };
        let entry = req
            .get("entry")
            .and_then(|v| v.as_str())
            .unwrap_or("main")
            .to_string();
        let args = match parse_args(req.get("args")) {
            Ok(a) => a,
            Err(m) => return fail(echo, "mn.worker.request", m),
        };
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
            None => None,
        };
        let nonce = req.get("nonce").and_then(|v| v.as_u64()).unwrap_or(0);

        if req.get("mode").and_then(|v| v.as_str()) == Some("rounds") {
            return serve_rounds(doc, entry, store_root.into(), workdir.into(), tracelib);
        }

        // One-shot: build once, run once, report the machine frame.
        let module = match typed_fixture::build_verified_module(&doc) {
            Ok(m) => m,
            Err(e) => return fail(echo, "mn.worker.fixture", e.message()),
        };
        let store = match ManagedStore::open(std::path::Path::new(store_root)) {
            Ok(s) => s,
            Err(e) => return fail(echo, "mn.worker.store", format!("{e:?}")),
        };
        match typed_pipeline::run_typed(
            &module,
            &entry,
            &args,
            nonce,
            std::path::Path::new(workdir),
            Some(&store),
            TypedCacheMode::Reuse,
            false,
            tracelib.as_deref(),
        ) {
            Ok(run) => {
                let ok = serde_json::json!({
                    "recipe": TYPED_RECIPE,
                    "worker_version": MN_WORKER_VERSION,
                    "mn_abi": ace_native::config::TYPED_ABI,
                    "mn_target": ace_native::config::MN_TARGET,
                    "counters": counters_json(&run.counters),
                    "typed": typed_json(&run),
                    "manifest_fp": run.manifest_fp,
                    "exit_code": run.exit_code,
                    "stderr_hex": hex(&run.stderr),
                    "stdout_hex": hex(&run.stdout_frame),
                });
                let mut resp = echo;
                resp["ok"] = ok;
                println!("{resp}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                let mut resp = echo;
                resp["ok"] = serde_json::json!({
                    "recipe": TYPED_RECIPE,
                    "worker_version": MN_WORKER_VERSION,
                    "mn_abi": ace_native::config::TYPED_ABI,
                    "mn_target": ace_native::config::MN_TARGET,
                });
                resp["diagnostic"] = serde_json::json!({"code": e.code(), "message": e.message()});
                println!("{resp}");
                ExitCode::SUCCESS
            }
        }
    }

    fn serve_rounds(
        doc: FixtureDoc,
        entry: String,
        store_root: PathBuf,
        workdir: PathBuf,
        tracelib: Option<PathBuf>,
    ) -> ExitCode {
        let respond = |v: serde_json::Value| {
            println!("{v}");
            let _ = std::io::stdout().flush();
        };
        let mut store_path = store_root;
        if store_path
            .components()
            .last()
            .map(|c| c.as_os_str() == "mn-store")
            .unwrap_or(false)
        {
            store_path = workdir.clone();
        }
        store_path.push("mn-typed-store");
        let store = match ManagedStore::open(&store_path) {
            Ok(s) => s,
            Err(e) => {
                let mut r = serde_json::json!({"schema": MN_WORKER_PROTOCOL});
                r["error"] =
                    serde_json::json!({"code": "mn.worker.store", "message": format!("{e:?}")});
                respond(r);
                return ExitCode::FAILURE;
            }
        };
        let _ = entry;
        let mut session = match TypedSession::start(
            doc,
            &workdir,
            Some(store),
            TypedCacheMode::Reuse,
        ) {
            Ok(s) => s,
            Err(e) => {
                let mut r = serde_json::json!({"schema": MN_WORKER_PROTOCOL});
                r["error"] =
                    serde_json::json!({"code": "mn.worker.typed-session", "message": e.message()});
                respond(r);
                return ExitCode::FAILURE;
            }
        };
        let warm = serde_json::json!({
            "schema": MN_WORKER_PROTOCOL,
            "ok": {
                "recipe": TYPED_RECIPE,
                "worker_version": MN_WORKER_VERSION,
                "mn_abi": ace_native::config::TYPED_ABI,
                "mn_target": ace_native::config::MN_TARGET,
                "step": "warmup",
                "counters": counters_json(session.counters()),
            }
        });
        respond(warm);
        let stdin = std::io::stdin();
        let mut lines = stdin.lock().lines();
        while let Some(Ok(line)) = lines.next() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let op: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(e) => {
                    let mut r = serde_json::json!({"schema": MN_WORKER_PROTOCOL});
                    r["error"] = serde_json::json!({"code": "mn.worker.request", "message": format!("bad op line: {e}")});
                    respond(r);
                    return ExitCode::FAILURE;
                }
            };
            let nonce = op.get("nonce").and_then(|v| v.as_u64()).unwrap_or(0);
            match op.get("op").and_then(|v| v.as_str()).unwrap_or("") {
                "run" => {
                    let args = match parse_args(op.get("args")) {
                        Ok(a) => a,
                        Err(m) => {
                            let mut r =
                                serde_json::json!({"schema": MN_WORKER_PROTOCOL, "nonce": nonce});
                            r["error"] =
                                serde_json::json!({"code": "mn.worker.request", "message": m});
                            respond(r);
                            return ExitCode::FAILURE;
                        }
                    };
                    match session.run(&args, nonce, tracelib.as_deref()) {
                        Ok(run) => {
                            respond(serde_json::json!({
                                "schema": MN_WORKER_PROTOCOL,
                                "nonce": nonce,
                                "ok": {
                                    "recipe": TYPED_RECIPE,
                                    "worker_version": MN_WORKER_VERSION,
                                    "mn_abi": ace_native::config::TYPED_ABI,
                                    "mn_target": ace_native::config::MN_TARGET,
                                    "step": "run",
                                    "typed": typed_json(&run),
                                    "counters": counters_json(&run.counters),
                                    "exit_code": run.exit_code,
                                    "stderr_hex": hex(&run.stderr),
                                }
                            }));
                        }
                        Err(e) => {
                            respond(serde_json::json!({
                                "schema": MN_WORKER_PROTOCOL,
                                "nonce": nonce,
                                "error": {"code": e.code(), "message": e.message()},
                            }));
                            return ExitCode::FAILURE;
                        }
                    }
                }
                "edit-unit" => {
                    let unit = op.get("unit").cloned().unwrap_or(serde_json::Value::Null);
                    let decl: Result<typed_fixture::UnitDecl, String> =
                        serde_json::from_value(unit).map_err(|e| format!("edit-unit payload: {e}"));
                    match decl {
                        Ok(decl) => match session.edit_unit(decl) {
                            Ok(obs) => {
                                respond(serde_json::json!({
                                    "schema": MN_WORKER_PROTOCOL,
                                    "nonce": nonce,
                                    "ok": {
                                        "recipe": TYPED_RECIPE,
                                        "worker_version": MN_WORKER_VERSION,
                                        "mn_abi": ace_native::config::TYPED_ABI,
                                        "mn_target": ace_native::config::MN_TARGET,
                                        "step": "edit-unit",
                                        "edit": {
                                            "edited_unit": obs.edited_unit,
                                            "reprepared_units": obs.reprepared_units,
                                            "verify_mir": obs.verify_mir,
                                            "unit_prepare": obs.unit_prepare,
                                            "unit_fp_render": obs.unit_fp_render,
                                            "native_admit": obs.native_admit,
                                            "emit_units": obs.emit_units,
                                            "persist_units": obs.persist_units,
                                            "cache_hits": obs.cache_hits,
                                            "whole_fallback": obs.whole_fallback,
                                        },
                                        "counters": counters_json(session.counters()),
                                    }
                                }));
                            }
                            Err(e) => {
                                respond(serde_json::json!({
                                    "schema": MN_WORKER_PROTOCOL,
                                    "nonce": nonce,
                                    "error": {"code": e.code(), "message": e.message()},
                                }));
                                return ExitCode::FAILURE;
                            }
                        },
                        Err(m) => {
                            let mut r =
                                serde_json::json!({"schema": MN_WORKER_PROTOCOL, "nonce": nonce});
                            r["error"] =
                                serde_json::json!({"code": "mn.worker.request", "message": m});
                            respond(r);
                            return ExitCode::FAILURE;
                        }
                    }
                }
                "close" => {
                    session.close();
                    respond(serde_json::json!({
                        "schema": MN_WORKER_PROTOCOL,
                        "nonce": nonce,
                        "ok": {"step": "closed"}
                    }));
                    return ExitCode::SUCCESS;
                }
                other => {
                    let mut r = serde_json::json!({"schema": MN_WORKER_PROTOCOL, "nonce": nonce});
                    r["error"] = serde_json::json!({
                        "code": "mn.worker.request",
                        "message": format!("unknown typed op `{other}`")
                    });
                    respond(r);
                    return ExitCode::FAILURE;
                }
            }
        }
        ExitCode::SUCCESS
    }
}
