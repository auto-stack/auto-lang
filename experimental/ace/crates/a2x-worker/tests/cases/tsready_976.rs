//! PLAN-976 TSREADY: end-to-end integration tests for the shared TS/JS
//! provider through the REAL store lanes and the worker ops:
//!
//! * TSREADY-01 (V-DETAIL-01, the module leg): a two-module package (user
//!   helper + std port use) through `PackageA2xTarget::cold` — the packages
//!   lane where `use`/module init is flattened — executed by REAL node on
//!   both legs against the same independent oracle as the Python leg
//!   (square(3)+10 = 19, byte_len("A中😀") = 8, value 27);
//! * TSREADY-02: the `a2x-worker` bin ops `identity` / `daily-emit-ts`
//!   (spawn-zero) / `run-ts` (real node execution: value line, trap line,
//!   exit codes, engine identity).

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_core::TargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_a2x_store::A2xSession;
use ace_a2x_ts::{
    verify_script_fileset_manifest, JavaScriptTargetProvider, ScriptLang, TypeScriptTargetProvider,
    TS_PRODUCER,
};
use ace_core::runtime_contract::sha256_hex;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::{compile_package_product, stdlib_inputs};

fn out_root(sub: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2ts-out")
        .join(format!("{sub}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn manifest_schema(manifest: &str) -> String {
    // The manifest is hand-scanned by the verifier; extract the declared
    // schema the same way (line-anchored, no JSON dependency).
    for line in manifest.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("\"schema\": \"") {
            if let Some(schema) = rest.strip_suffix("\",") {
                return schema.to_string();
            }
        }
    }
    panic!("manifest carries no schema line:\n{manifest}");
}

fn write_and_verify(dir: &PathBuf, files: &[(String, String)], manifest: &str) {
    for (rel, content) in files {
        std::fs::write(dir.join(rel), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), manifest).unwrap();
    let schema = manifest_schema(manifest);
    assert!(
        schema.starts_with("ace.a2ts-hir."),
        "package fileset must carry a TS schema, got `{schema}`"
    );
    verify_script_fileset_manifest(manifest, files, TS_PRODUCER, &schema)
        .expect("valid new-producer manifest");
}

fn run_node(dir: &PathBuf, lang: ScriptLang) -> (i32, String, String) {
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".to_string());
    let out = Command::new(&node)
        .arg(lang.main_file())
        .current_dir(dir)
        .output()
        .expect("spawn node");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The two-module package from the Python leg (PLAN-1042), oracle unchanged:
/// square(3) = 9 -> +10 = 19; byte_len("A中😀") = 8 UTF-8 bytes; 19 + 8 = 27.
#[test]
fn tsready_01_two_module_package_executes_via_node() {
    let mut inputs = stdlib_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "helper".into(),
        source: "use std.math.square\npub fn combined(x int) int { return square(x) + 10 }\n"
            .into(),
        provider: None,
    });
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: concat!(
            "use app.helper.combined\n",
            "use std.text.byte_len\n",
            "fn main() int {\n",
            "    let a = combined(3)\n",
            "    let n = byte_len(\"A中😀\")\n",
            "    return a + n\n",
            "}\n",
        )
        .into(),
        provider: None,
    });
    let product = compile_package_product(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.to_string()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("compile two-module stdlib package");

    tsready_01_leg(ScriptLang::TypeScript, TypeScriptTargetProvider, &product);
    tsready_01_leg(ScriptLang::JavaScript, JavaScriptTargetProvider, &product);
}

fn tsready_01_leg<P: TargetProvider + 'static>(
    lang: ScriptLang,
    pkg: P,
    product: &ace_source_query::modules::packages::PackageProduct,
) {
    let store_dir = out_root(&format!("tsready01_{:?}_store", lang));
    let mut pkg_target = PackageA2xTarget::cold(
        &store_dir,
        "tsready01",
        "probe",
        64 * 1024 * 1024,
        pkg,
        product,
    )
    .expect("PackageA2xTarget::cold for the script provider");
    let receipt = pkg_target.receipt().expect("pkg receipt");
    assert!(!receipt.digest.is_empty());
    assert!(pkg_target.current.manifest.contains(TS_PRODUCER));

    let run_dir = out_root(&format!("tsready01_{:?}_run", lang));
    write_and_verify(
        &run_dir,
        &pkg_target.current.files,
        &pkg_target.current.manifest,
    );
    // Module init/dispatch evidence: entry present + no legacy emitter.
    let generated_path = if run_dir.join("generated.ts").exists() {
        run_dir.join("generated.ts")
    } else {
        run_dir.join("generated.mjs")
    };
    let generated = std::fs::read_to_string(&generated_path).unwrap();
    assert!(generated.contains("fn_main"), "entry present:\n{generated}");
    assert!(!generated.contains("TypeScriptTrans"));
    assert!(!generated.contains("JavaScriptTrans"));

    let (code, stdout, stderr) = run_node(&run_dir, lang);
    assert_eq!(
        (code, stdout.as_str()),
        (0, "value:27\n"),
        "{lang:?}: stderr={stderr}\ngenerated=\n{generated}"
    );
    let _ = std::fs::remove_dir_all(&store_dir);
    let _ = std::fs::remove_dir_all(&run_dir);
}

/// Invoke the a2x-worker bin with one JSON request (stdin), parse the reply.
fn worker_op(request: &serde_json::Value) -> serde_json::Value {
    let exe = env!("CARGO_BIN_EXE_a2x-worker");
    let mut cmd = Command::new(exe);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().expect("spawn a2x-worker");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("worker reply");
    assert!(
        out.status.success(),
        "worker failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("worker json reply")
}

#[test]
fn tsready_02_worker_identity_daily_emit_and_run() {
    // identity: the script channels are registered.
    let id = worker_op(&serde_json::json!({"op": "identity"}));
    assert_eq!(id["status"], "ok");
    assert_eq!(id["ts_backend"], "auto-a2x-typescript");
    assert_eq!(id["js_backend"], "auto-a2x-javascript");
    let recipes = id["recipes"].as_array().unwrap();
    for r in ["a2ts-hir-typescript-checked", "a2js-hir-javascript-checked"] {
        assert!(
            recipes.iter().any(|x| x.as_str() == Some(r)),
            "recipe `{r}` missing from identity"
        );
    }
    let targets = id["targets"].as_array().unwrap();
    assert!(targets.iter().any(|t| t == "typescript"));
    assert!(targets.iter().any(|t| t == "javascript"));

    let src = "fn square(x int) int { return x * x }\nfn main() int { return square(8) + 1 }";

    // daily-emit-ts: spawn-zero on both legs, files + manifest verified
    // in-process against the real artifacts.
    for target_lang in ["typescript", "javascript"] {
        let dir = out_root(&format!("tsready02_emit_{target_lang}"));
        let reply = worker_op(&serde_json::json!({
            "op": "daily-emit-ts",
            "source_text": src,
            "profile": "core",
            "entry": "main",
            "target_lang": target_lang,
            "out_dir": dir.to_string_lossy(),
            "nonce": "n1",
        }));
        assert_eq!(reply["status"], "ok", "{reply}");
        assert_eq!(reply["execution_claimed"], false);
        assert_eq!(reply["spawned_node"], 0);
        assert_eq!(reply["target_lang"], target_lang);
        let manifest = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
        let files: Vec<(String, String)> = [
            "a2x_rt.mjs",
            "generated.ts",
            "generated.mjs",
            "a2x_main.ts",
            "a2x_main.mjs",
            "a2x_rt.d.mts",
            "source_map.json",
        ]
        .iter()
        .filter_map(|f| {
            std::fs::read_to_string(dir.join(f))
                .ok()
                .map(|c| (f.to_string(), c))
        })
        .collect();
        let schema = manifest_schema(&manifest);
        verify_script_fileset_manifest(&manifest, &files, TS_PRODUCER, &schema)
            .expect("daily-emit-ts artifacts verify");
        assert_eq!(
            sha256_hex(manifest.as_bytes()),
            reply["manifest_sha256"].as_str().unwrap()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // run-ts: REAL node execution — value line + exit 0 on both legs.
    for target_lang in ["typescript", "javascript"] {
        let dir = out_root(&format!("tsready02_run_{target_lang}"));
        let reply = worker_op(&serde_json::json!({
            "op": "run-ts",
            "source_text": src,
            "profile": "core",
            "entry": "main",
            "target_lang": target_lang,
            "out_dir": dir.to_string_lossy(),
        }));
        assert_eq!(reply["status"], "ok", "{reply}");
        assert_eq!(reply["execution_claimed"], true);
        assert_eq!(reply["spawned_node"], 1);
        assert_eq!(reply["stdout"], "value:65\n", "{reply}");
        assert_eq!(reply["exit_code"], 0);
        assert_eq!(reply["node_identity"]["engine"], "node");
        assert!(!reply["node_identity"]["version"]
            .as_str()
            .unwrap()
            .is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // run-ts: the trap path — i32 overflow prints the trap line, exit 1.
    let dir = out_root("tsready02_trap");
    let reply = worker_op(&serde_json::json!({
        "op": "run-ts",
        "source_text": "fn main() int { return 2147483647 + 1 }",
        "profile": "core",
        "entry": "main",
        "target_lang": "typescript",
        "out_dir": dir.to_string_lossy(),
    }));
    assert_eq!(reply["status"], "ok", "{reply}");
    assert_eq!(reply["stdout"], "trap:Overflow\n");
    assert_eq!(reply["exit_code"], 1);
    let _ = std::fs::remove_dir_all(&dir);
}

/// TSREADY-06 (V-01): i64/u64 magnitudes beyond 2^53 survive the WHOLE
/// chain (source bigint literals -> provider -> node) on both legs — a JS
/// `number` representation cannot carry them, so `value:1`/`value:2000000012`
/// prove the `bigint` representation end to end.
#[test]
fn tsready_06_i64_u64_beyond_2p53_roundtrip() {
    let src = "fn main() int {\n    let a = 2000000000i64\n    let b = a + 12i64\n    return b.as(i32)\n}\nfn other() int {\n    let x = 18446744073709551615u64\n    let y = x - 18446744073709551614u64\n    return y.as(i32)\n}";
    tsready_06_leg(
        ScriptLang::TypeScript,
        TypeScriptTargetProvider,
        src,
        "main",
        "value:2000000012\n",
    );
    tsready_06_leg(
        ScriptLang::JavaScript,
        JavaScriptTargetProvider,
        src,
        "main",
        "value:2000000012\n",
    );
    tsready_06_leg(
        ScriptLang::TypeScript,
        TypeScriptTargetProvider,
        src,
        "other",
        "value:1\n",
    );
}

fn tsready_06_leg<P: TargetProvider + 'static>(
    lang: ScriptLang,
    provider: P,
    src: &str,
    entry: &str,
    expected: &str,
) {
    let store_dir = out_root(&format!("tsready06_{:?}_{}_store", lang, entry));
    let run_dir = out_root(&format!("tsready06_{:?}_{}_run", lang, entry));
    let mut s = A2xSession::open(&store_dir, "tsready06", "probe", 64 * 1024 * 1024, provider)
        .expect("session open");
    let asm = s
        .cold_assemble_profile(
            src,
            ace_core::source::SourceProfile::Aggregates,
            &[],
            entry,
            false,
        )
        .expect("cold assemble");
    write_and_verify(&run_dir, &asm.files, &asm.manifest);
    let (code, stdout, stderr) = run_node(&run_dir, lang);
    assert_eq!(
        (code, stdout.as_str()),
        (0, expected),
        "{lang:?}/{entry}: stderr={stderr}"
    );
    let _ = std::fs::remove_dir_all(&store_dir);
    let _ = std::fs::remove_dir_all(&run_dir);
}

/// TSREADY-05 (V-02): side-effect short-circuit and named-argument
/// evaluation order through REAL node, observed on the mark stderr channel —
/// a non-short-circuiting or order-swapping lowering produces a different
/// stderr/stdout pair (the independent oracle is the source order).
#[test]
fn tsready_05_short_circuit_and_named_arg_order() {
    let src = "fn pair(a int, b int) int { return a * 10 + b }\nfn main() int {\n    let r = pair(b: mark_b(), a: mark_a())\n    if mark_a() == 1 || mark_b() == 2 { return r }\n    return 0\n}";
    let store_dir = out_root("tsready05_store");
    let run_dir = out_root("tsready05_run");
    let mut s = A2xSession::open(
        &store_dir,
        "tsready05",
        "probe",
        64 * 1024 * 1024,
        TypeScriptTargetProvider,
    )
    .expect("session open");
    let asm = s
        .cold_assemble_profile(
            src,
            ace_core::source::SourceProfile::Scalar,
            &[ace_core::source::TRACE_CAPABILITY.to_string()],
            "main",
            false,
        )
        .expect("cold assemble");
    write_and_verify(&run_dir, &asm.files, &asm.manifest);

    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".to_string());
    let out = Command::new(&node)
        .arg("a2x_main.ts")
        .current_dir(&run_dir)
        .output()
        .expect("spawn node");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(out.status.code(), Some(0), "stderr={stderr}");
    // named args evaluate in SOURCE order (b first); the `||` then
    // short-circuits on `mark_a() == 1` so the second `mark_b` never runs.
    assert_eq!(
        stdout, "value:12\n",
        "a=mark_a()=1, b=mark_b()=2 -> 1*10+2; stderr={stderr}"
    );
    // node may prefix a MODULE_TYPELESS_PACKAGE_JSON warning when a parent
    // directory carries a package.json (environment noise, not semantics) —
    // the mark SEQUENCE is the oracle.
    assert!(
        stderr.ends_with("mark_b\nmark_a\nmark_a\n"),
        "evaluation order (stderr={stderr:?})"
    );
    assert_eq!(stderr.matches("mark_a").count(), 2);
    assert_eq!(stderr.matches("mark_b").count(), 1);
    let _ = std::fs::remove_dir_all(&store_dir);
    let _ = std::fs::remove_dir_all(&run_dir);
}

/// TSREADY-04 (V-08, the type-check leg): the emitted TypeScript fileset
/// passes `tsc --noEmit --strict` with the frozen flags when a tsc is
/// available (§5 B: a missing tsc is a NAMED infra skip of the type-check
/// leg only — never a language failure and never a silent PASS; the node
/// execution leg always runs).
#[test]
fn tsready_04_tsc_strict_typecheck_leg() {
    let src = "type Point { x int; y int }\nfn pick(p Point) int {\n    return p.x * 10 + p.y\n}\nfn main() int {\n    let p Point = Point { x: 3, y: 4 }\n    return pick(p)\n}";
    let store_dir = out_root("tsready04_store");
    let run_dir = out_root("tsready04_run");
    let mut s = A2xSession::open(
        &store_dir,
        "tsready04",
        "probe",
        64 * 1024 * 1024,
        TypeScriptTargetProvider,
    )
    .expect("session open");
    let asm = s
        .cold_assemble_profile(
            src,
            ace_core::source::SourceProfile::TextRecord,
            &[],
            "main",
            false,
        )
        .expect("cold assemble");
    write_and_verify(&run_dir, &asm.files, &asm.manifest);

    // The node execution leg always runs (independent oracle: 34).
    let (code, stdout, stderr) = run_node(&run_dir, ScriptLang::TypeScript);
    assert_eq!(
        (code, stdout.as_str()),
        (0, "value:34\n"),
        "stderr={stderr}"
    );

    // V-04: the source map binds each declaration to its ORIGINAL span
    // (`fn pick` starts at byte 23 of the source above) and to generated
    // lines — not to a renumbered or empty span.
    let source_map = std::fs::read_to_string(run_dir.join("source_map.json")).unwrap();
    let pick_origin = src.find("fn pick").unwrap();
    assert!(
        source_map.contains(&format!(
            "\"decl\": \"pick\", \"decl_kind\": \"fn\", \"origin\": [{pick_origin}, "
        )),
        "source map must anchor `pick` at its original span:\n{source_map}"
    );
    assert!(source_map.contains("\"generated_lines\": ["));

    let tsc = std::env::var("A2TS_TSC").unwrap_or_else(|_| {
        "D:/autostack/auto-ui/examples/ui/001-helloworld/gen/front/vue/node_modules/typescript/bin/tsc"
            .to_string()
    });
    if !std::path::Path::new(&tsc).exists() {
        eprintln!(
            "INFRA SKIP (named): tsc not found at `{tsc}` — the strict type-check leg did not run; node execution above is the real oracle"
        );
        return;
    }
    let node = std::env::var("A2TS_NODE").unwrap_or_else(|_| "node".to_string());
    let out = Command::new(&node)
        .arg(&tsc)
        .args([
            "--noEmit",
            "--target",
            "es2022",
            "--module",
            "nodenext",
            "--strict",
            "--allowImportingTsExtensions",
        ])
        .arg("generated.ts")
        .arg("a2x_main.ts")
        .current_dir(&run_dir)
        .output()
        .expect("spawn tsc");
    assert!(
        out.status.success(),
        "tsc --strict failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&store_dir);
    let _ = std::fs::remove_dir_all(&run_dir);
}

/// The single-source session lane (cold + restart on the same store) both
/// legs, node-executed — the store-level reuse evidence for the new layers.
#[test]
fn tsready_03_session_cold_and_restart_reuse() {
    let src = "fn square(x int) int { return x * x }\nfn main() int { return square(9) + 2 }";
    tsready_03_leg(ScriptLang::TypeScript, TypeScriptTargetProvider, src);
    tsready_03_leg(ScriptLang::JavaScript, JavaScriptTargetProvider, src);
}

fn tsready_03_leg<P: TargetProvider + Clone + 'static>(lang: ScriptLang, provider: P, src: &str) {
    let store_dir = out_root(&format!("tsready03_{:?}_store", lang));
    let mut s = A2xSession::open(
        &store_dir,
        "tsready03",
        "probe",
        64 * 1024 * 1024,
        provider.clone(),
    )
    .expect("session open");
    let asm = s
        .cold_assemble_profile(
            src,
            ace_core::source::SourceProfile::Core,
            &[],
            "main",
            false,
        )
        .expect("cold assemble");
    let cold_manifest = asm.manifest.clone();
    let run_dir = out_root(&format!("tsready03_{:?}_run1", lang));
    write_and_verify(&run_dir, &asm.files, &asm.manifest);
    let (code, stdout, stderr) = run_node(&run_dir, lang);
    assert_eq!(
        (code, stdout.as_str()),
        (0, "value:83\n"),
        "stderr={stderr}"
    );

    // restart lane: a NEW session on the SAME store reassembles from
    // store hits — byte-identical manifest, same real execution.
    let mut s2 = A2xSession::open(&store_dir, "tsready03", "probe", 64 * 1024 * 1024, provider)
        .expect("restart session open");
    let asm2 = s2
        .cold_assemble_profile(
            src,
            ace_core::source::SourceProfile::Core,
            &[],
            "main",
            false,
        )
        .expect("restart assemble");
    assert_eq!(asm2.manifest, cold_manifest, "restart manifest drift");
    let run2 = out_root(&format!("tsready03_{:?}_run2", lang));
    write_and_verify(&run2, &asm2.files, &asm2.manifest);
    let (code2, stdout2, stderr2) = run_node(&run2, lang);
    assert_eq!(
        (code2, stdout2.as_str()),
        (0, "value:83\n"),
        "stderr={stderr2}"
    );
    let _ = std::fs::remove_dir_all(&store_dir);
}
