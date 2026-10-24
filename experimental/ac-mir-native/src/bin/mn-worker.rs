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

        // Cross-process tool-discovery memo (T-07 turnaround): the lld/SDK
        // PATHS are memoized next to the store (short-lived worker
        // processes of one session share it); tool CONTENT identity is
        // still re-hashed in-process per request.
        {
            let memo = PathBuf::from(&store_root).join("..").join("mn-tool-memo.json");
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
            let rev: i64 = flag("--rev")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
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
