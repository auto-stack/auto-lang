//! PLAN-892 T-02/T-05 A2X-HIR worker: the PREBUILT tool the registered
//! recipe `ac-a2r/hir-scalar-emit-v1` executes against (the 796 trusted
//! worker identity contract — a case can never name an executable).
//!
//! Protocol (`a2xworker/1`, JSON request on argv[1] or stdin → JSON
//! response on stdout, exit 0 on success / nonzero + {"status":"error"}
//! on controlled failure):
//!
//! * `{"op":"identity"}` → tool/protocol/recipe/backend identity.
//! * `{"op":"daily-emit","source_path","profile":"core|scalar","entry",
//!    "caps":[...],"out_dir","nonce"}` → COLD emit of the full actual file
//!   set (main.rs / a2x_runtime.rs / generated.rs / manifest.json) into
//!   `out_dir`. NO rustc/cargo/target-program spawn (the daily spawn-zero
//!   contract, plan §5.5); nothing is read from any golden — goldens are
//!   judged parent-side.
//! * `{"op":"certify",...}` → the SAME on-the-spot actual set PLUS a real
//!   rustc compile+run of it and of the frozen i32-form reference, each
//!   compared to the caller-supplied closed-form oracle value/trap. The
//!   EXPLICIT certification leg only — never the daily case path.
//!
//! The worker never receives expected values on the daily path; the
//! certify op takes the oracle explicitly because it IS the explicit leg.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use auto_a2x_core::sha256::sha256_hex;
use auto_a2x_core::SemanticView;
use auto_a2x_rust::{assemble_file_set, lower_program};
use auto_ac_prototype::source::{compile_scalar_source, compile_source};

const PROTOCOL: &str = "a2xworker/1";
const RECIPE: &str = "ac-a2r/hir-scalar-emit-v1";

fn main() {
    let mut req = String::new();
    if let Some(arg) = std::env::args().nth(1) {
        req = arg;
    } else {
        std::io::stdin()
            .read_to_string(&mut req)
            .expect("read request");
    }
    let v: serde_json::Value = match serde_json::from_str(&req) {
        Ok(v) => v,
        Err(e) => fail(&format!("bad request json: {e}")),
    };
    let out = match v["op"].as_str().unwrap_or("") {
        "identity" => identity(),
        "daily-emit" => daily_emit(&v),
        "certify" => certify(&v),
        other => fail(&format!("unknown op `{}`", other)),
    };
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

fn fail(detail: &str) -> ! {
    println!(
        "{}",
        serde_json::json!({ "status": "error", "detail": detail })
    );
    std::process::exit(2);
}

fn ok_base() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "tool": "a2x-worker",
        "protocol": PROTOCOL,
        "recipe": RECIPE,
    })
}

fn identity() -> serde_json::Value {
    let t = auto_a2x_rust::TargetProfile::v1();
    serde_json::json!({
        "status": "ok",
        "tool": "a2x-worker",
        "protocol": PROTOCOL,
        "recipe": RECIPE,
        "backend": t.backend,
        "backend_version": t.backend_version,
        "edition": t.edition,
    })
}

struct EmitOutcome {
    files: Vec<(String, String)>,
    manifest: String,
    input_hash: String,
    entry_fn: String,
    profile: String,
    source_sha256: String,
}

fn read_source(v: &serde_json::Value) -> (String, String) {
    if let Some(p) = v["source_path"].as_str() {
        let text = match std::fs::read_to_string(p) {
            Ok(t) => t,
            Err(e) => fail(&format!("read source {p}: {e}")),
        };
        (
            text,
            sha256_hex(std::fs::read(p).unwrap_or_default().as_slice()),
        )
    } else if let Some(t) = v["source_text"].as_str() {
        let h = sha256_hex(t.as_bytes());
        (t.to_string(), h)
    } else {
        fail("request needs source_path or source_text");
    }
}

fn emit(v: &serde_json::Value) -> EmitOutcome {
    let (src, source_sha256) = read_source(v);
    let profile = v["profile"].as_str().unwrap_or("core").to_string();
    let entry = v["entry"].as_str().unwrap_or("main").to_string();
    let caps: Vec<String> = v["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let scalar = match profile.as_str() {
        "core" => false,
        "scalar" => true,
        other => fail(&format!("unknown source profile `{}`", other)),
    };
    let compiled = if scalar {
        compile_scalar_source(&src, "a2x-worker", &caps)
    } else {
        compile_source(&src, "a2x-worker", &caps)
    };
    let compiled = match compiled {
        Ok(c) => c,
        Err(ds) => {
            let detail = ds
                .iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ");
            fail(&format!("frontend diagnostics: {}", detail));
        }
    };
    let view = match SemanticView::from_checked(&compiled.checked) {
        Ok(view) => view,
        Err(r) => fail(&format!("view admission: {}", r)),
    };
    let program = match lower_program(&view, &entry) {
        Ok(p) => p,
        Err(r) => fail(&format!("lowering: {}", r)),
    };
    let entry_fn = program.entry.clone();
    let fs = assemble_file_set(&view, &program, &compiled.input_hash, &entry);
    EmitOutcome {
        files: fs.files,
        manifest: fs.manifest,
        input_hash: compiled.input_hash,
        entry_fn,
        profile: view.profile.to_string(),
        source_sha256,
    }
}

fn file_records(files: &[(String, String)]) -> Vec<serde_json::Value> {
    files
        .iter()
        .map(|(p, c)| {
            serde_json::json!({ "path": p, "sha256": sha256_hex(c.as_bytes()), "bytes": c.len() })
        })
        .collect()
}

fn write_out_dir(dir: &str, out: &EmitOutcome) {
    let _ = dir; // callers pass the canonicalized dir already
    let dir = PathBuf::from(dir);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        fail(&format!("create out_dir: {e}"));
    }
    for (p, c) in &out.files {
        if let Err(e) = std::fs::write(dir.join(p), c) {
            fail(&format!("write {p}: {e}"));
        }
    }
    if let Err(e) = std::fs::write(dir.join("manifest.json"), &out.manifest) {
        fail(&format!("write manifest: {e}"));
    }
}

fn daily_emit(v: &serde_json::Value) -> serde_json::Value {
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let nonce = v["nonce"].as_str().unwrap_or("");
    let out = emit(v);
    write_out_dir(out_dir, &out);
    let mut r = ok_base();
    r["mode"] = "daily-emit".into();
    r["nonce"] = nonce.into();
    r["profile"] = out.profile.clone().into();
    r["input_hash"] = out.input_hash.clone().into();
    r["source_sha256"] = out.source_sha256.clone().into();
    r["entry_fn"] = out.entry_fn.clone().into();
    r["manifest_sha256"] = sha256_hex(out.manifest.as_bytes()).into();
    r["files"] = file_records(&out.files).into();
    r["spawned_rustc"] = 0.into();
    r
}

fn certify(v: &serde_json::Value) -> serde_json::Value {
    let out_dir = v["out_dir"]
        .as_str()
        .unwrap_or_else(|| fail("out_dir required"));
    let reference_path = {
        let raw = v["reference_path"]
            .as_str()
            .unwrap_or_else(|| fail("reference_path required (the case's hand-written reference)"));
        // F-3: canonicalize at entry — relative paths break nothing
        std::fs::canonicalize(raw)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|e| fail(&format!("canonicalize reference: {e}")))
    };
    let oracle_kind = v["oracle_kind"].as_str().unwrap_or("value");
    let oracle_line = match oracle_kind {
        "value" => format!(
            "value:{}",
            v["oracle_value"]
                .as_i64()
                .unwrap_or_else(|| fail("oracle_value required"))
        ),
        "trap" => format!(
            "trap:{}",
            v["oracle_trap"]
                .as_str()
                .unwrap_or_else(|| fail("oracle_trap required"))
        ),
        other => fail(&format!("oracle_kind must be value|trap, got {other}")),
    };
    let case_id = v["case_id"].as_str().unwrap_or("a2r-hir-892/unnamed");
    let out = emit(v);
    write_out_dir(out_dir, &out);
    let dir = PathBuf::from(out_dir);

    // actual program: real rustc + run (stdout value/trap + stderr marks);
    // walls are OBSERVED per the 852 budget lines (compile <=120s, run
    // <=30s, whole certify <=360s) — the caller asserts the caps
    let t0 = std::time::Instant::now();
    let (actual_out, actual_err) = compile_and_run(&dir, "main.rs", "a2x_actual.exe");
    let actual_wall_ms = t0.elapsed().as_millis() as u64;
    // independent hand-written reference: compile + run separately
    let reference_src = std::fs::read_to_string(&reference_path)
        .unwrap_or_else(|e| fail(&format!("read reference {reference_path}: {e}")));
    std::fs::write(dir.join("reference.rs"), &reference_src)
        .unwrap_or_else(|e| fail(&format!("write reference: {e}")));
    let t1 = std::time::Instant::now();
    let (reference_out, reference_err) = compile_and_run(&dir, "reference.rs", "a2x_reference.exe");
    let reference_wall_ms = t1.elapsed().as_millis() as u64;

    let mut r = ok_base();
    r["mode"] = "certify".into();
    r["case"] = case_id.into();
    r["actual_observed"] = actual_out.clone().into();
    r["reference_observed"] = reference_out.clone().into();
    r["oracle"] = oracle_line.clone().into();
    let stdout_ok = actual_out == oracle_line && reference_out == oracle_line;
    let marks_ok = actual_err == reference_err;
    r["marks_match_reference"] = marks_ok.into();
    let verdict = if stdout_ok && marks_ok {
        "PASS"
    } else {
        "REJECTED"
    };
    r["verdict"] = verdict.into();
    if verdict == "REJECTED" {
        r["detail"] = format!(
            "actual={:?} reference={:?} oracle={:?} marks_actual={:?} marks_ref={:?}",
            actual_out, reference_out, oracle_line, actual_err, reference_err
        )
        .into();
    }

    // the strict certificate draft binds the full actual file set, the
    // reference bytes and the OBSERVED toolchain identity
    let vv = toolchain_vv();
    let cert = auto_a2x_rust::CertDraft {
        schema: auto_a2x_rust::CERT_SCHEMA,
        recipe: auto_a2x_rust::RECIPE,
        case: case_id.to_string(),
        profile: out.profile.clone(),
        hir_domain: {
            let t = auto_a2x_rust::TargetProfile::v1();
            let _ = t;
            domain_of(&out.profile)
        },
        entry: v["entry"].as_str().unwrap_or("main").to_string(),
        verdict: verdict.to_string(),
        oracle_kind: if oracle_kind == "trap" {
            "trap"
        } else {
            "value"
        },
        oracle_value: if oracle_kind == "trap" {
            v["oracle_trap"].as_str().unwrap_or("").to_string()
        } else {
            v["oracle_value"].as_i64().unwrap_or(0).to_string()
        },
        input_hash: out.input_hash.clone(),
        source_sha256: out.source_sha256.clone(),
        actual_files: out
            .files
            .iter()
            .map(|(p, c)| auto_a2x_rust::BoundFile {
                path: p.clone(),
                sha256: sha256_hex(c.as_bytes()),
            })
            .collect(),
        golden_sha256: None,
        wrapper_sha256: sha256_hex(
            out.files
                .iter()
                .find(|(p, _)| p == "main.rs")
                .map(|(_, c)| c.as_bytes())
                .unwrap_or(&[]),
        ),
        reference_sha256: sha256_hex(reference_src.as_bytes()),
        backend: "auto-a2x-rust",
        backend_version: "0.1.0",
        shared_view_version: "0.1.0",
        protocol: "a2x-observation/1",
        toolchain: auto_a2x_rust::ToolchainIdentity::observed(
            vv,
            vec!["--edition".into(), "2021".into()],
        ),
        capabilities: v["caps"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|c| c.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
    };
    if let Err(e) = cert.validate() {
        fail(&format!("certificate draft invalid: {e}"));
    }
    let cert_json = cert.to_json();
    std::fs::write(dir.join("case.cert.json"), &cert_json)
        .unwrap_or_else(|e| fail(&format!("write cert: {e}")));
    r["cert_sha256"] = sha256_hex(cert_json.as_bytes()).into();
    r["actual_wall_ms"] = actual_wall_ms.into();
    r["reference_wall_ms"] = reference_wall_ms.into();
    r["files"] = file_records(&out.files).into();
    r
}

fn domain_of(profile: &str) -> String {
    if profile.contains("scalar") {
        "scalar-i32-draft".to_string()
    } else {
        "core-i32-draft".to_string()
    }
}

fn toolchain_vv() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let out = Command::new(&rustc)
        .arg("-vV")
        .output()
        .unwrap_or_else(|e| fail(&format!("rustc -vV: {e}")));
    if !out.status.success() {
        fail("rustc -vV failed");
    }
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// 852 budget lines (plan §5.6): compile <=120s, run <=30s; streams are
/// capped at 1MiB each (overflow terminates the tree — never an unbounded
/// reader). F-1 (review): the ac-core contained executor provides the
/// CREATE_SUSPENDED + Job-subtree + single-deadline + full-collection
/// contract; a timeout kills the WHOLE owned tree and is a controlled
/// refusal, never a hang.
const COMPILE_DEADLINE_SECS: u64 = 120;
const RUN_DEADLINE_SECS: u64 = 30;
const STREAM_CAP: usize = 1024 * 1024;

fn contained(
    cmd: &mut Command,
    deadline: std::time::Duration,
) -> auto_ac_prototype::link::CapturedRun {
    use auto_ac_prototype::link::{run_deadlined_captured, CaptureCaps, SubprocError};
    match run_deadlined_captured(
        cmd,
        deadline,
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    ) {
        Ok(r) => r,
        Err(SubprocError::Deadline) => fail("deadline: the owned tree was killed before finishing"),
        Err(SubprocError::Spawn(e)) => fail(&format!("spawn: {e}")),
        Err(SubprocError::ContainmentUnavailable(d)) => {
            fail(&format!("containment unavailable: {d}"))
        }
        Err(other) => fail(&format!("contained run failed: {other:?}")),
    }
}

fn compile_and_run(dir: &Path, src: &str, exe_name: &str) -> (String, String) {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let exe = dir.join(exe_name);
    let mut compile = Command::new(&rustc);
    compile
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg(src)
        .current_dir(dir);
    let out = contained(
        &mut compile,
        std::time::Duration::from_secs(COMPILE_DEADLINE_SECS),
    );
    if out.exit_code != Some(0) {
        fail(&format!(
            "{src} compile failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut run = Command::new(&exe);
    run.current_dir(dir);
    let run = contained(&mut run, std::time::Duration::from_secs(RUN_DEADLINE_SECS));
    if run.exit_code != Some(0) {
        fail(&format!("{exe_name} exit non-zero"));
    }
    (
        String::from_utf8_lossy(&run.stdout).trim().to_string(),
        String::from_utf8_lossy(&run.stderr).trim().to_string(),
    )
}
