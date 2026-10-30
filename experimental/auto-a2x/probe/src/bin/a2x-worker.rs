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

use auto_ac_prototype::source::{compile_scalar_source, compile_source};
use auto_a2x_core::sha256::sha256_hex;
use auto_a2x_core::SemanticView;
use auto_a2x_rust::{assemble_file_set, lower_program};

const PROTOCOL: &str = "a2xworker/1";
const RECIPE: &str = "ac-a2r/hir-scalar-emit-v1";

/// The frozen independent i32-form reference for the certified entry (the
/// NEW wrapper form — never the old a2r i64 golden).
const REFERENCE_SRC: &str = r#"// PLAN-892 independent reference (hand-written, i32/trap form)
fn ref_main() -> i32 { 2 + 3 * 4 }
fn main() { println!("value:{}", ref_main()); }
"#;

fn main() {
    let mut req = String::new();
    if let Some(arg) = std::env::args().nth(1) {
        req = arg;
    } else {
        std::io::stdin().read_to_string(&mut req).expect("read request");
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
        (text, sha256_hex(std::fs::read(p).unwrap_or_default().as_slice()))
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
        .map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect())
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
    let out_dir = v["out_dir"].as_str().unwrap_or_else(|| fail("out_dir required"));
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
    let out_dir = v["out_dir"].as_str().unwrap_or_else(|| fail("out_dir required"));
    let oracle_value = v["oracle_value"].as_i64().unwrap_or_else(|| fail("oracle_value required"));
    let out = emit(v);
    write_out_dir(out_dir, &out);
    let dir = PathBuf::from(out_dir);

    // actual program: real rustc + run
    let actual_line = compile_and_run(&dir, "main.rs", "a2x_actual.exe");
    // independent reference: compile + run separately
    if let Err(e) = std::fs::write(dir.join("reference.rs"), REFERENCE_SRC) {
        fail(&format!("write reference: {e}"));
    }
    let reference_line = compile_and_run(&dir, "reference.rs", "a2x_reference.exe");
    let expect = format!("value:{}", oracle_value);
    let mut r = ok_base();
    r["mode"] = "certify".into();
    r["actual_observed"] = actual_line.clone().into();
    r["reference_observed"] = reference_line.clone().into();
    r["oracle"] = expect.clone().into();
    if actual_line == expect && reference_line == expect {
        r["verdict"] = "PASS".into();
    } else {
        r["verdict"] = "REJECTED".into();
        r["detail"] = format!(
            "actual={:?} reference={:?} oracle={:?}",
            actual_line, reference_line, expect
        )
        .into();
    }
    r["files"] = file_records(&out.files).into();
    r
}

fn compile_and_run(dir: &Path, src: &str, exe_name: &str) -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let exe = dir.join(exe_name);
    let out = match Command::new(&rustc)
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg(src)
        .current_dir(dir)
        .output()
    {
        Ok(o) => o,
        Err(e) => fail(&format!("spawn {rustc}: {e}")),
    };
    if !out.status.success() {
        fail(&format!(
            "{src} compile failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let run = match Command::new(&exe).current_dir(dir).output() {
        Ok(o) => o,
        Err(e) => fail(&format!("run {exe_name}: {e}")),
    };
    if !run.status.success() {
        fail(&format!("{exe_name} exit non-zero"));
    }
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}
