//! PLAN-1042: end-to-end integration tests for PYREADY-01..04:
//! Checked HIR -> shared A2X (A2xSession / PackageA2xTarget) -> PythonTargetProvider -> real Python.

use std::path::PathBuf;

use ace_a2x_core::TargetProvider;
use ace_a2x_python::{
    default_stdlib_port_catalog, python_runtime_source, verify_python_fileset_manifest,
    PythonTargetProvider, PY_FILESET_SCHEMA, PY_PRODUCER, PY_RT_FILESET_SCHEMA, PY_RUNTIME_VERSION,
};
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_a2x_store::A2xSession;
use ace_a2x_worker::pyrun::{resolve_python, run_py_fileset, ResolvedPython};
use ace_core::runtime_contract::sha256_hex;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_core::source::cst::FrontBudget;
use ace_core::source::{SourceProfile, TRACE_CAPABILITY};
use ace_source_query::modules::packages::{compile_package_product, stdlib_inputs};

fn out_root(sub: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2x-out-py")
        .join(format!("{sub}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write_and_verify(
    dir: &PathBuf,
    files: &[(String, String)],
    manifest: &str,
    expected_schema: &str,
) {
    for (rel, content) in files {
        std::fs::write(dir.join(rel), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), manifest).unwrap();
    verify_python_fileset_manifest(manifest, files, PY_PRODUCER, expected_schema)
        .expect("valid new-producer manifest");
}

fn py_tool() -> ResolvedPython {
    let rt = python_runtime_source(&["hir.test.mark_a", "hir.test.mark_b"]);
    let rt_hash = sha256_hex(rt.as_bytes());
    resolve_python(&rt_hash).expect("OS Python interpreter required (missing Python cannot PASS)")
}

#[test]
fn pyready_01_two_module_stdlib_and_scalar_execution() {
    let py = py_tool();
    // 1) Two-module ordinary Auto program using std.math & user helper module
    let mut inputs = stdlib_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "helper".into(),
        source: "use std.math.square\npub fn combined(x int) int { return square(x) + 10 }\n".into(),
    });
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: concat!(
            "use app.helper.combined\n",
            "use std.text.byte_len\n",
            "fn main() int {\n",
            "    let a = combined(3)\n", // square(3) = 3*3 = 9 -> +10 = 19
            "    let n = byte_len(\"A中😀\")\n", // 8 UTF-8 bytes
            "    return a + n\n", // 19 + 8 = 27
            "}\n",
        )
        .into(),
    });
    let product = compile_package_product(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.to_string()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("compile two-module stdlib package");

    let store_dir = out_root("pyready01_pkg_store");
    let mut pkg_target = PackageA2xTarget::cold(
        &store_dir,
        "pyready01",
        "probe",
        64 * 1024 * 1024,
        PythonTargetProvider,
        &product,
    )
    .expect("PackageA2xTarget::cold for PythonTargetProvider");
    let receipt = pkg_target.receipt().expect("pkg receipt");
    assert!(!receipt.digest.is_empty());
    assert!(pkg_target.current.manifest.contains(PY_PRODUCER));

    let run_dir = out_root("pyready01_pkg_run");
    write_and_verify(
        &run_dir,
        &pkg_target.current.files,
        &pkg_target.current.manifest,
        PY_RT_FILESET_SCHEMA,
    );
    let source_map = std::fs::read_to_string(run_dir.join("source_map.json")).unwrap();
    assert!(source_map.contains("main"));
    let generated_py = std::fs::read_to_string(run_dir.join("generated.py")).unwrap();
    assert!(generated_py.contains("a2x_fn_"));
    assert!(!generated_py.contains("PythonTrans"));

    let run = run_py_fileset(&py, &run_dir, &[]).expect("run multi-module Python output");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}\ngenerated=\n{}",
        String::from_utf8_lossy(&run.stderr),
        std::fs::read_to_string(run_dir.join("generated.py")).unwrap_or_default()
    );
    assert_eq!(run.stdout, b"value:27\n", "stderr={:?}", String::from_utf8_lossy(&run.stderr));

    // 2) Scalar cases: C-truncation div/rem, overflow trap, short-circuit trace marks + effect count
    let cases: &[(&str, &str, bool, &[u8], &[&str])] = &[
        (
            "c_trunc_div_rem",
            "fn main() i32 { return (-7 / 2) * 10 + (-7 % 2) }",
            true,
            b"value:-31\n",
            &[],
        ),
        (
            "overflow_trap",
            "fn main() i32 { return 2147483647 + 1 }",
            false,
            b"trap:Overflow\n",
            &[],
        ),
        (
            "div_zero_trap",
            "fn main() i32 { return 42 / 0 }",
            false,
            b"trap:DivByZero\n",
            &[],
        ),
        (
            "short_circuit_and_or",
            "fn yes() bool { return mark_a() == 1 }\nfn no() bool { return mark_b() == 999 }\nfn main() int {\n    if no() && yes() { return 1 }\n    if yes() || no() { return 7 }\n    return 0\n}",
            true,
            b"value:7\n",
            &["mark_b", "mark_a"],
        ),
    ];

    for (name, src, trace, expected_stdout, expected_marks) in cases {
        let caps = if *trace {
            vec![TRACE_CAPABILITY.to_string()]
        } else {
            vec![]
        };
        let sdir = out_root(&format!("pyready01_sc_store_{name}"));
        let mut s = A2xSession::open(
            &sdir,
            "pyready01",
            "probe",
            64 * 1024 * 1024,
            PythonTargetProvider,
        )
        .unwrap();
        let asm = s.cold_assemble(src, true, &caps, "main", false).unwrap();
        let rdir = out_root(&format!("pyready01_sc_run_{name}"));
        write_and_verify(&rdir, &asm.files, &asm.manifest, PY_FILESET_SCHEMA);
        let out = run_py_fileset(&py, &rdir, &[]).unwrap();
        assert_eq!(out.exit_code, Some(0), "case {name}");
        assert_eq!(out.stdout.as_slice(), *expected_stdout, "case {name}");
        let got_marks: Vec<String> = String::from_utf8_lossy(&out.stderr)
            .lines()
            .map(ToString::to_string)
            .collect();
        let exp: Vec<String> = expected_marks.iter().map(|s| s.to_string()).collect();
        assert_eq!(got_marks, exp, "trace marks mismatch for {name}");
    }
}

#[test]
fn pyready_02_utf8_record_text_explicit_cleanup() {
    let py = py_tool();
    let src = concat!(
        "fn main() int {\n",
        "    var a Str = \"A中😀\"\n",
        "    let c Str = \"e\u{0301}\"\n",
        "    let o Str = a.clone()\n",
        "    a = a + \"xy\"\n",
        "    return a.len() + c.len() + o.len()\n",
        "}\n",
    );
    let sdir = out_root("pyready02_rt_store");
    let mut s = A2xSession::open(
        &sdir,
        "pyready02",
        "probe",
        64 * 1024 * 1024,
        PythonTargetProvider,
    )
    .unwrap();
    let asm = s
        .cold_assemble_profile(src, SourceProfile::TextRecord, &[], "main", false)
        .unwrap();
    let rdir = out_root("pyready02_rt_run");
    write_and_verify(&rdir, &asm.files, &asm.manifest, PY_RT_FILESET_SCHEMA);
    let trace_path = rdir.join("rt_trace.json");
    let run = run_py_fileset(
        &py,
        &rdir,
        &[
            ("A2X_PY_TRACE_JSON", trace_path.to_str().unwrap()),
            ("A2X_RT_EVENTS", "1"),
        ],
    )
    .unwrap();
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}\ngenerated=\n{}",
        String::from_utf8_lossy(&run.stderr),
        std::fs::read_to_string(rdir.join("generated.py")).unwrap_or_default()
    );
    // a = "A中😀xy" (5 scalars) + c = "e\u{0301}" (2 scalars) + o = "A中😀" (3 scalars) -> 10
    assert_eq!(run.stdout, b"value:10\n", "stderr={:?}", String::from_utf8_lossy(&run.stderr));
    let trace = std::fs::read_to_string(&trace_path).unwrap();
    assert!(trace.contains("\"live_strs\": []"), "expected zero leaked Str after explicit free: {trace}");
    assert!(!trace.contains("\"alloc_count\": 0"), "expected tracked allocation: {trace}");
}

#[test]
fn pyready_03_invalidation_and_failed_publish_isolation() {
    let py = py_tool();
    let v1_src = "fn main() i32 { return 11 + 31 }";
    let v2_src = "fn main() i32 { return 100 + 23 }";
    let bad_src = "fn main() i32 { return unknown_fn() }";

    let sdir = out_root("pyready03_store");
    let pub_dir = out_root("pyready03_publish");

    // Step 1: cold v1
    let mut s1 = A2xSession::open(&sdir, "pyready03", "probe", 64 * 1024 * 1024, PythonTargetProvider).unwrap();
    let asm1 = s1.cold_assemble(v1_src, true, &[], "main", false).unwrap();
    write_and_verify(&pub_dir, &asm1.files, &asm1.manifest, PY_FILESET_SCHEMA);
    let r1 = run_py_fileset(&py, &pub_dir, &[]).unwrap();
    assert_eq!(
        r1.exit_code,
        Some(0),
        "stderr={}\nmain=\n{}\ngenerated=\n{}",
        String::from_utf8_lossy(&r1.stderr),
        std::fs::read_to_string(pub_dir.join("a2x_main.py")).unwrap_or_default(),
        std::fs::read_to_string(pub_dir.join("generated.py")).unwrap_or_default()
    );
    assert_eq!(r1.stdout, b"value:42\n");
    let orig_manifest = asm1.manifest.clone();

    // Step 2: unchanged reopen restores cleanly via fragments
    let mut s_warm = A2xSession::open(&sdir, "pyready03", "probe", 64 * 1024 * 1024, PythonTargetProvider).unwrap();
    let asm_warm = s_warm.cold_assemble(v1_src, true, &[], "main", false).unwrap();
    assert_eq!(asm_warm.manifest, orig_manifest);

    // Step 3: failing build MUST NOT overwrite existing published fileset
    let mut s_bad = A2xSession::open(&sdir, "pyready03", "probe", 64 * 1024 * 1024, PythonTargetProvider).unwrap();
    let err = s_bad.cold_assemble(bad_src, true, &[], "main", false);
    assert!(err.is_err(), "invalid source must reject instead of emitting broken artifacts");
    let on_disk_manifest = std::fs::read_to_string(pub_dir.join("manifest.json")).unwrap();
    assert_eq!(on_disk_manifest, orig_manifest, "failed build must leave old manifest untouched");
    let r_old = run_py_fileset(&py, &pub_dir, &[]).unwrap();
    assert_eq!(r_old.stdout, b"value:42\n");

    // Step 4: repaired source invalidates fragments and emits/executes new output
    let mut s2 = A2xSession::open(&sdir, "pyready03", "probe", 64 * 1024 * 1024, PythonTargetProvider).unwrap();
    let asm2 = s2.cold_assemble(v2_src, true, &[], "main", false).unwrap();
    assert_ne!(asm2.manifest, orig_manifest);
    write_and_verify(&pub_dir, &asm2.files, &asm2.manifest, PY_FILESET_SCHEMA);
    let r2 = run_py_fileset(&py, &pub_dir, &[]).unwrap();
    assert_eq!(r2.stdout, b"value:123\n");

    // Step 5: helper / port catalog / interpreter identity key sensitivity
    let p = PythonTargetProvider;
    let k1 = p.fragment_key_inputs(&sha256_hex(b"rt-a"));
    let k2 = p.fragment_key_inputs(&sha256_hex(b"rt-b"));
    assert_ne!(k1, k2);
    let mut id_b = py.identity.clone();
    id_b.version = "3.99.0-changed".to_string();
    assert_ne!(py.identity.identity_key(), id_b.identity_key());
    let cat = default_stdlib_port_catalog(PY_RUNTIME_VERSION);
    assert!(!cat.canonical_fingerprint().is_empty());
}

#[test]
fn pyready_04_admission_negative_controls() {
    let py = py_tool();
    let sdir = out_root("pyready04_store");
    let mut s = A2xSession::open(&sdir, "pyready04", "probe", 64 * 1024 * 1024, PythonTargetProvider).unwrap();
    let asm = s
        .cold_assemble("fn main() i32 { return 19 + 23 }", true, &[], "main", false)
        .unwrap();

    // 1) Legacy PythonTrans manifest/producer explicitly rejected
    assert!(verify_python_fileset_manifest(
        &asm.manifest,
        &asm.files,
        "legacy/PythonTrans",
        PY_FILESET_SCHEMA,
    )
    .is_err());

    // 2) Manifest referencing legacy PythonTrans origin rejected even with right producer literal
    let tainted_files = vec![(
        "generated.py".to_string(),
        "# legacy PythonTrans\n".to_string(),
    )];
    let digest = sha256_hex(tainted_files[0].1.as_bytes());
    let tainted_manifest = format!(
        "{{\"schema\":\"{PY_FILESET_SCHEMA}\",\"producer\":\"{PY_PRODUCER}\",\"files\":[{{\"path\":\"generated.py\",\"sha256\":\"{digest}\"}}]}}"
    );
    assert!(verify_python_fileset_manifest(
        &tainted_manifest,
        &tainted_files,
        PY_PRODUCER,
        PY_FILESET_SCHEMA,
    )
    .is_err());

    // 3) Tampered file content rejected
    let mut tampered = asm.files.clone();
    tampered[0].1.push_str("\n# tampered");
    assert!(verify_python_fileset_manifest(
        &asm.manifest,
        &tampered,
        PY_PRODUCER,
        PY_FILESET_SCHEMA,
    )
    .is_err());

    // 4) Missing Python interpreter fails loudly (cannot PASS or fake)
    let orig_env = std::env::var("A2PY_PYTHON").ok();
    unsafe {
        std::env::set_var("A2PY_PYTHON", "Z:/nonexistent_python_binary_1042/python.exe");
    }
    let missing_res = resolve_python("abc");
    match orig_env {
        Some(v) => unsafe { std::env::set_var("A2PY_PYTHON", v) },
        None => unsafe { std::env::remove_var("A2PY_PYTHON") },
    }
    assert!(missing_res.is_err(), "missing Python interpreter must fail cleanly");

    // 5) Legal new-producer run accepted cleanly
    let rdir = out_root("pyready04_legal");
    write_and_verify(&rdir, &asm.files, &asm.manifest, PY_FILESET_SCHEMA);
    let out = run_py_fileset(&py, &rdir, &[]).unwrap();
    assert_eq!(out.stdout, b"value:42\n");
}
