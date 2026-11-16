//! PLAN-1064 V-03 conformance for `std.storage` (`ACE-KV/1`, D-07 / T-KV)
//! and `std.zip` (D-08 / T-04) across AV, A2R, and A2Py.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_python::PythonTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::platform_io::Win32PlatformIo;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs};
use ace_source_query::modules::packages::compile_assembled_aggregates_inputs;

fn sys_grants() -> Vec<String> {
    vec![
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.stdin".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
        "platform.file".into(),
        "platform.file.path".into(),
        "platform.env".into(),
        "platform.clock".into(),
        "platform.entropy".into(),
        "platform.process".into(),
    ]
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/kvzip1064/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn slash(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn compile_sys(
    backend: AssemblyBackend,
    main_src: &str,
) -> ace_source_query::modules::packages::PackageProduct {
    let app_files = vec![("main.at".to_string(), main_src.to_string())];
    let mut ctx = PackageAssemblyContext::new(backend, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = ace_source_query::modules::packages::assemble_stdlib_sys_inputs(
        &app_files,
        "app::main",
        &mut ctx,
    )
    .expect("assemble sys inputs");
    compile_assembled_aggregates_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
        .unwrap_or_else(|diags| {
            panic!(
                "compile failed: {:?}",
                diags
                    .iter()
                    .map(|d| format!(
                        "{:?} {:?} {} @{}..{}",
                        d.module,
                        d.diagnostic.stage,
                        d.diagnostic.message,
                        d.diagnostic.span.start,
                        d.diagnostic.span.end
                    ))
                    .collect::<Vec<_>>()
            )
        })
}

fn run_av_i32(tag: &str, main_src: &str) -> i32 {
    let product = compile_sys(AssemblyBackend::Av, main_src);
    let store = scratch(tag);
    let target = PackageAvTarget::cold(&store, &product).expect("AV cold link");
    let mut backend = Win32PlatformIo::default();
    let (out, _summary) = target
        .run_platform_value(&sys_grants(), &mut backend)
        .expect("AV platform run");
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("{tag}: unexpected AV outcome {other:?}"),
    }
}

fn run_a2r_i32(tag: &str, main_src: &str) -> i32 {
    let product = compile_sys(AssemblyBackend::A2R, main_src);
    let store = scratch(&format!("{tag}-store"));
    let rust = PackageA2xTarget::cold_aggregates(
        &store,
        tag,
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .expect("A2R cold");
    let build_dir = scratch(&format!("{tag}-build"));
    for (path, content) in &rust.current.files {
        std::fs::write(build_dir.join(path), content).unwrap();
    }
    let exe = build_dir.join(format!("{tag}.exe"));
    let mut cmd =
        std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"]).arg(&exe);
    cmd.current_dir(&build_dir);
    let compile = auto_tool_execution::run_command_capture(
        cmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc failed: {}",
        String::from_utf8_lossy(compile.stderr.as_deref().unwrap_or(&[]))
    );
    let run = std::process::Command::new(&exe)
        .current_dir(&build_dir)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "a2r exe failed: exit={:?} stderr={}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );
    let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
    let suffix = out
        .rsplit("value:")
        .next()
        .unwrap_or_else(|| panic!("missing value: in a2r stdout {out:?}"));
    suffix
        .trim()
        .parse::<i32>()
        .unwrap_or_else(|e| panic!("bad value {suffix:?} in stdout {out:?}: {e}"))
}

fn run_a2py_i32(tag: &str, main_src: &str) -> i32 {
    let product = compile_sys(AssemblyBackend::A2Py, main_src);
    let store = scratch(&format!("{tag}-store"));
    let py = PackageA2xTarget::cold_aggregates(
        &store,
        tag,
        "app",
        16 << 20,
        PythonTargetProvider,
        &product,
    )
    .expect("A2Py cold");
    let py_dir = scratch(&format!("{tag}-build"));
    for (path, content) in &py.current.files {
        std::fs::write(py_dir.join(path), content).unwrap();
    }
    let rt = ace_a2x_python::python_runtime_source(&[]);
    let rt_hash = ace_core::runtime_contract::sha256_hex(rt.as_bytes());
    let tool = ace_a2x_worker::pyrun::resolve_python(&rt_hash)
        .expect("OS Python interpreter required");
    let run = ace_a2x_worker::pyrun::run_py_fileset(&tool, &py_dir, &[]).unwrap();
    assert_eq!(
        run.exit_code,
        Some(0),
        "python stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
    let suffix = out
        .rsplit("value:")
        .next()
        .unwrap_or_else(|| panic!("missing value: in python stdout {out:?}"));
    suffix
        .trim()
        .parse::<i32>()
        .unwrap_or_else(|e| panic!("bad value {suffix:?} in stdout {out:?}: {e}"))
}

fn kv_record(key: &[u8], val: &[u8]) -> Vec<u8> {
    let mut out = format!("{} {}\n", key.len(), val.len()).into_bytes();
    out.extend_from_slice(key);
    out.extend_from_slice(val);
    out.push(b'\n');
    out
}

#[test]
fn v03_kv_wire_oracle_and_reopen_av() {
    let dir = scratch("kv-av-oracle");
    let oracle_db = dir.join("oracle.kv");
    let fresh_db = dir.join("fresh.kv");
    let os = slash(&oracle_db);
    let fs = slash(&fresh_db);

    // Independent Rust hand-crafted ACE-KV/1 wire bytes:
    //   1) empty key + empty value ("0 0\n\n")
    //   2) duplicate key "dup": first "old", second "new"
    //   3) NUL / newline / space / Unicode key + value
    //   4) two keys differing only after NUL ("k\0a" vs "k\0b")
    let mut raw = b"ACE-KV/1\n".to_vec();
    raw.extend(kv_record(b"", b""));
    raw.extend(kv_record(b"dup", b"old"));
    raw.extend(kv_record(b"dup", b"new"));
    raw.extend(kv_record(b"k\x00\n 1", "v\x00\n A中😀".as_bytes()));
    raw.extend(kv_record(b"k\x00a", b"va"));
    raw.extend(kv_record(b"k\x00b", b"vb"));
    std::fs::write(&oracle_db, &raw).unwrap();

    let prog_read = format!(
        r#"
use std.storage.KVRead
use std.storage.kv_get
use std.storage.get
use std.storage.kv_set
use std.storage.set
use std.storage.kv_remove
use std.storage.remove

fn main() int {{
    let r_empty KVRead = kv_get("{os}", "")
    if (r_empty.status != 0 || !r_empty.found || r_empty.value != "") {{ return -1 }}
    let r_miss KVRead = get("{os}", "absent")
    if (r_miss.status != 0 || r_miss.found || r_miss.value != "") {{ return -2 }}
    let r_dup KVRead = kv_get("{os}", "dup")
    if (r_dup.status != 0 || !r_dup.found || r_dup.value != "new") {{ return -3 }}
    let r_bin KVRead = kv_get("{os}", "k\0\n 1")
    if (r_bin.status != 0 || !r_bin.found || r_bin.value != "v\0\n A中😀") {{ return -4 }}
    let r_ka KVRead = kv_get("{os}", "k\0a")
    let r_kb KVRead = kv_get("{os}", "k\0b")
    if (!r_ka.found || r_ka.value != "va" || !r_kb.found || r_kb.value != "vb") {{ return -5 }}

    // Write fresh DB with Unicode + LF + NUL
    if (kv_set("{fs}", "alpha", "A中😀\n\0z") != 0) {{ return -6 }}
    if (set("{fs}", "beta", "v1") != 0) {{ return -7 }}
    if (kv_set("{fs}", "beta", "v2") != 0) {{ return -8 }}
    if (remove("{fs}", "absent") != 0) {{ return -9 }}
    return 100
}}
"#
    );
    assert_eq!(run_av_i32("kv-av-1", &prog_read), 100);

    // Independent Rust wire-byte verification of fresh.kv after first AV session
    let disk_bytes = std::fs::read(&fresh_db).unwrap();
    let mut want = b"ACE-KV/1\n".to_vec();
    want.extend(kv_record(b"alpha", "A中😀\n\0z".as_bytes()));
    want.extend(kv_record(b"beta", b"v2"));
    assert_eq!(disk_bytes, want, "disk ACE-KV/1 wire mismatch");
    assert!(!fresh_db.with_file_name("fresh.kv.acekv.tmp").exists());

    // Reopen in a second cold AV instance and test remove
    let prog_reopen = format!(
        r#"
use std.storage.KVRead
use std.storage.kv_get
use std.storage.kv_remove

fn main() int {{
    let a KVRead = kv_get("{fs}", "alpha")
    if (a.status != 0 || !a.found || a.value != "A中😀\n\0z") {{ return -1 }}
    let b KVRead = kv_get("{fs}", "beta")
    if (b.status != 0 || !b.found || b.value != "v2") {{ return -2 }}
    if (kv_remove("{fs}", "alpha") != 1) {{ return -3 }}
    if (kv_remove("{fs}", "alpha") != 0) {{ return -4 }}
    let a2 KVRead = kv_get("{fs}", "alpha")
    if (a2.status != 0 || a2.found || a2.value != "") {{ return -5 }}
    if (kv_remove("{fs}", "beta") != 1) {{ return -6 }}
    return 200
}}
"#
    );
    assert_eq!(run_av_i32("kv-av-reopen", &prog_reopen), 200);
    assert_eq!(std::fs::read(&fresh_db).unwrap(), b"ACE-KV/1\n");
}

#[test]
fn v03_kv_corrupt_io_budget_and_atomic_preserve_av() {
    let dir = scratch("kv-av-neg");
    let sub = dir.join("subdir");
    std::fs::create_dir_all(&sub).unwrap();
    let missing = dir.join("missing.kv");

    let mut bad_files: Vec<(String, Vec<u8>)> = vec![
        ("empty.kv".into(), b"".to_vec()),
        ("badmagic.kv".into(), b"ACE-KV/2\n0 0\n\n".to_vec()),
        ("neg.kv".into(), b"ACE-KV/1\n-1 0\n\n".to_vec()),
        ("leadzero.kv".into(), b"ACE-KV/1\n01 0\na\n".to_vec()),
        ("overflow.kv".into(), b"ACE-KV/1\n9999999 0\n\n".to_vec()),
        ("trunc.kv".into(), b"ACE-KV/1\n5 1\nab".to_vec()),
        ("notraillf.kv".into(), b"ACE-KV/1\n1 1\nab\r".to_vec()),
    ];
    let mut bad_utf8 = b"ACE-KV/1\n".to_vec();
    bad_utf8.extend(kv_record(b"ok", b"yes"));
    bad_utf8.extend(kv_record(b"bad", &[0xFF]));
    bad_files.push(("badutf8.kv".into(), bad_utf8));

    let mut bad_hashes = Vec::new();
    let mut bad_checks = String::new();
    for (idx, (name, bytes)) in bad_files.iter().enumerate() {
        let p = dir.join(name);
        std::fs::write(&p, bytes).unwrap();
        bad_hashes.push((name.clone(), p.clone(), ace_core::runtime_contract::sha256_hex(bytes)));
        let ps = slash(&p);
        let err_base = -(100 + (idx as i32) * 3);
        bad_checks.push_str(&format!(
            "    let g{idx} KVRead = kv_get(\"{ps}\", \"ok\")\n    if (g{idx}.status != -2 || g{idx}.found || g{idx}.value != \"\") {{ return {err_base} }}\n    if (kv_set(\"{ps}\", \"ok\", \"x\") != -2) {{ return {} }}\n    if (kv_remove(\"{ps}\", \"ok\") != -2) {{ return {} }}\n",
            err_base - 1,
            err_base - 2
        ));
    }

    // Over-budget size (> 262,144 B) -> -3 cleanly in O(1) before reading
    let big = dir.join("big.kv");
    let mut big_bytes = b"ACE-KV/1\n".to_vec();
    big_bytes.resize(262_145, b'0');
    std::fs::write(&big, &big_bytes).unwrap();
    let big_s = slash(&big);

    // Nonexistent parent write -> -1 IO error, no temp file leak
    let bad_parent = dir.join("no_such_dir").join("x.kv");
    let bp_s = slash(&bad_parent);
    let ms = slash(&missing);
    let ss = slash(&sub);

    // Atomic preservation when target is read-only (ReplaceFileW / MoveFileExW fails with access denied)
    let ro = dir.join("readonly.kv");
    let mut ro_bytes = b"ACE-KV/1\n".to_vec();
    ro_bytes.extend(kv_record(b"keep", b"orig"));
    std::fs::write(&ro, &ro_bytes).unwrap();
    let ro_hash = ace_core::runtime_contract::sha256_hex(&ro_bytes);
    let mut perms = std::fs::metadata(&ro).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&ro, perms).unwrap();
    let ro_s = slash(&ro);

    let misc_src = format!(
        r#"
use std.storage.KVRead
use std.storage.kv_get
use std.storage.kv_set
use std.storage.kv_remove

fn main() int {{
    let gm KVRead = kv_get("{ms}", "a")
    if (gm.status != 0 || gm.found || gm.value != "") {{ return -1 }}
    if (kv_remove("{ms}", "a") != 0) {{ return -2 }}

    let gd KVRead = kv_get("{ss}", "a")
    if (gd.status != -1 || gd.found) {{ return -3 }}
    if (kv_set("{ss}", "a", "b") != -1) {{ return -4 }}
    if (kv_remove("{ss}", "a") != -1) {{ return -5 }}

    let gb KVRead = kv_get("{big_s}", "a")
    if (gb.status != -3 || gb.found) {{ return -6 }}
    if (kv_set("{big_s}", "a", "b") != -3) {{ return -7 }}

{bad_checks}    if (kv_set("{bp_s}", "a", "b") != -1) {{ return -8 }}
    if (kv_set("{ro_s}", "keep", "new") != -1) {{ return -9 }}
    return 42
}}
"#
    );
    let rc = run_av_i32("kv-av-misc", &misc_src);
    // Restore write bit so scratch cleanup always succeeds cleanly
    let mut perms2 = std::fs::metadata(&ro).unwrap().permissions();
    perms2.set_readonly(false);
    std::fs::set_permissions(&ro, perms2).unwrap();

    assert_eq!(rc, 42);
    for (name, p, before_hash) in &bad_hashes {
        let after = std::fs::read(p).unwrap();
        assert_eq!(
            ace_core::runtime_contract::sha256_hex(&after),
            *before_hash,
            "corrupt file modified on failed op: {name}"
        );
    }
    assert!(!missing.exists(), "get/remove must not create missing.kv");
    assert!(!ro.with_file_name("readonly.kv.acekv.tmp").exists(), "temp leaked");
    assert_eq!(
        ace_core::runtime_contract::sha256_hex(&std::fs::read(&ro).unwrap()),
        ro_hash,
        "readonly.kv altered on failed atomic replace"
    );
}

fn run_kv_cross_backend_leg(tag: &str, is_rust: bool) {
    let dir = scratch(tag);

    let mut full_4096 = b"ACE-KV/1\n".to_vec();
    for i in 0..4096 {
        let k = format!("k{i}");
        full_4096.extend(kv_record(k.as_bytes(), b"v"));
    }
    let full_p = dir.join("full4096.kv");
    std::fs::write(&full_p, &full_4096).unwrap();

    let mut over_4097 = full_4096.clone();
    over_4097.extend(kv_record(b"extra", b"v"));
    let over_p = dir.join("over4097.kv");
    std::fs::write(&over_p, &over_4097).unwrap();

    let bad_p = dir.join("bad.kv");
    std::fs::write(&bad_p, b"ACE-KV/1\n1 1\na\xFF\n").unwrap();

    let db_p = dir.join(format!("{tag}.kv"));
    let mut seed = b"ACE-KV/1\n".to_vec();
    seed.extend(kv_record(b"dup", b"1"));
    seed.extend(kv_record(b"dup", b"2"));
    seed.extend(kv_record(b"u\x00key", "值😀\n\0".as_bytes()));
    std::fs::write(&db_p, &seed).unwrap();

    let dbs = slash(&db_p);
    let fulls = slash(&full_p);
    let overs = slash(&over_p);
    let bads = slash(&bad_p);

    let src = format!(
        r#"
use std.storage.KVRead
use std.storage.kv_get
use std.storage.kv_set
use std.storage.kv_remove

fn main() int {{
    let d KVRead = kv_get("{dbs}", "dup")
    if (d.status != 0 || !d.found || d.value != "2") {{ return -1 }}
    let u KVRead = kv_get("{dbs}", "u\0key")
    if (u.status != 0 || !u.found || u.value != "值😀\n\0") {{ return -2 }}

    let sr = kv_set("{dbs}", "added", "A\0B")
    if (sr != 0) {{ return -300 + sr }}
    let rr = kv_remove("{dbs}", "dup")
    if (rr != 1) {{ return -400 + rr }}

    let badr KVRead = kv_get("{bads}", "x")
    if (badr.status != -2 || badr.found) {{ return -5 }}

    let ov KVRead = kv_get("{overs}", "k0")
    if (ov.status != -3 || ov.found) {{ return -6 }}

    if (kv_set("{fulls}", "brand_new", "x") != -3) {{ return -7 }}
    return 77
}}
"#
    );
    let got = if is_rust {
        run_a2r_i32(tag, &src)
    } else {
        run_a2py_i32(tag, &src)
    };
    assert_eq!(got, 77, "{tag}");

    let mut want = b"ACE-KV/1\n".to_vec();
    want.extend(kv_record(b"u\x00key", "值😀\n\0".as_bytes()));
    want.extend(kv_record(b"added", b"A\0B"));
    assert_eq!(std::fs::read(&db_p).unwrap(), want, "{tag} final disk bytes");
}

#[test]
fn v03_kv_a2r_roundtrip_and_budget_4096() {
    run_kv_cross_backend_leg("kv-a2r", true);
}

#[test]
fn v03_kv_a2py_roundtrip_and_budget_4096() {
    run_kv_cross_backend_leg("kv-a2py", false);
}
