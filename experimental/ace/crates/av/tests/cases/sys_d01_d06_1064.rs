//! PLAN-1064 T-03 (`V-02`): permanent conformance & cross-backend parity suite for
//! `D-01..D-06` deferred `std.{file,path,env,process}` surfaces across AV, A2R, and A2Py.
//! Verifies exact 058_read_text_range (10-line) and 046_env_get_set oracles,
//! `canonical_w16` missing-target rejection, `env_get_w16` unset-vs-empty distinction,
//! Win32 quoted/empty/Unicode `process.args()` roundtrip, and `spawn_with_output` cap/kill cleanup.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_python::PythonTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::ExecOutcome;
use ace_av_core::platform_io::Win32PlatformIo;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_sys_inputs, compile_assembled_aggregates_inputs, PackageProduct,
};

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
        .join(format!("../../target/v02_1064/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn compile_sys(backend: AssemblyBackend, main: &str) -> PackageProduct {
    let app_files = vec![("main.at".to_string(), main.to_string())];
    let mut ctx = PackageAssemblyContext::new(backend, ProviderChoice::Auto, TargetOs::Windows);
    let inputs =
        assemble_stdlib_sys_inputs(&app_files, "app::main", &mut ctx).expect("assemble sys inputs");
    compile_assembled_aggregates_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
        .unwrap_or_else(|diags| {
            panic!(
                "compile failed: {:?}",
                diags
                    .iter()
                    .map(|d| format!(
                        "{:?} @{}..{}: {}",
                        d.module, d.diagnostic.span.start, d.diagnostic.span.end, d.diagnostic.message
                    ))
                    .collect::<Vec<_>>()
            )
        })
}

fn run_av_check(tag: &str, main: &str) -> (ExecOutcome, std::path::PathBuf) {
    let store = scratch(&format!("{tag}_store"));
    let product = compile_sys(AssemblyBackend::Av, main);
    let target = PackageAvTarget::cold(&store, &product).expect("cold link");
    let mut backend = Win32PlatformIo::default();
    let (out, _stats) = target
        .run_platform_value(&sys_grants(), &mut backend)
        .expect("av exec");
    (out, store)
}

#[test]
fn v02_d01_d02_file_text_range_and_dir_names_av() {
    let tmp = scratch("d01_d02_av");
    let tp = tmp.to_string_lossy().replace('\\', "/");

    let sub = tmp.join("sub");
    let empty_sub = tmp.join("empty_sub");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::create_dir_all(&empty_sub).unwrap();
    std::fs::write(tmp.join("ascii.txt"), "abcdefghij").unwrap();
    std::fs::write(tmp.join("cjk.txt"), "你好世界").unwrap();
    std::fs::write(tmp.join("empty.txt"), "").unwrap();
    std::fs::write(tmp.join("bad.bin"), &[0xFF, 0xFE, 0xFD]).unwrap();
    std::fs::write(tmp.join("nul.bin"), b"a\x00b\r\nc").unwrap();
    std::fs::write(tmp.join("esc.txt"), "a\"b\\c\nd\te").unwrap();
    std::fs::write(tmp.join(".hidden"), "h").unwrap();
    std::fs::write(tmp.join("中文.txt"), "ok").unwrap();

    let report_path = tmp.join("report.txt");
    let rp = report_path.to_string_lossy().replace('\\', "/");

    let src = format!(
        r#"
use std.file.read_text
use std.file.read_text_range
use std.file.write_text
use std.file.delete
use std.file.dir_names

fn check_names_sum(xs List<Str>) int {{
    if (xs.len() == 0) {{ return 0 }}
    let item Str = xs.remove(0)
    if (item == "." || item == "..") {{ return -10000 }}
    var w = 0
    if (item == "ascii.txt") {{ w = 1 }}
    if (item == ".hidden") {{ w = 10 }}
    if (item == "中文.txt") {{ w = 100 }}
    if (item == "sub") {{ w = 1000 }}
    let rest = check_names_sum(xs)
    if (rest < 0) {{ return rest }}
    return w + rest
}}

fn main() int {{
    let d = "{tp}"
    let t_asc = read_text(d + "/ascii.txt")
    if (t_asc != "abcdefghij") {{ return 101 }}
    let t_cjk = read_text(d + "/cjk.txt")
    if (t_cjk != "你好世界") {{ return 102 }}
    if (read_text(d + "/empty.txt") != "") {{ return 103 }}
    if (read_text(d + "/no_such.txt") != "") {{ return 104 }}
    if (read_text(d + "/bad.bin") != "") {{ return 105 }}
    let t_nul = read_text(d + "/nul.bin")
    if (t_nul.len() != 6) {{ return 106 }}

    // 058_read_text_range exact 10-line expected.out matrix
    let l1 = read_text_range(d + "/ascii.txt", 2, 4)
    let l2 = read_text_range(d + "/ascii.txt", 6, 10)
    let l3 = read_text_range(d + "/ascii.txt", 100, 4)
    let l4 = read_text_range(d + "/ascii.txt", 0, 0)
    let l5 = read_text_range(d + "/no_such.txt", 0, 4)
    let l6 = read_text_range(d + "/cjk.txt", 2, 4)
    let l7 = read_text_range(d + "/cjk.txt", 0, 4)
    let l8 = read_text_range(d + "/cjk.txt", 3, 3)
    let l9 = read_text_range(d + "/cjk.txt", 9, 10)
    let l10 = read_text_range(d + "/empty.txt", 0, 10)

    let l_bad = read_text_range(d + "/bad.bin", 0, 3)
    let l_neg = read_text_range(d + "/ascii.txt", -1, 4)
    let l_esc = read_text_range(d + "/esc.txt", 0, 16)

    // Delete files from guest after reading to prove zero leaked file handles
    if (delete(d + "/esc.txt") != 0) {{ return 107 }}

    // D-02 dir_names: excludes "." and "..", preserves Unicode/dotfile/subdir
    let names = dir_names(d.clone())
    if (names.len() != 9) {{ return 108 }}
    let sum = check_names_sum(names)
    if (sum != 1111) {{ return 109 }}

    let names_e = dir_names(d + "/empty_sub")
    if (names_e.len() != 0) {{ return 114 }}
    let names_m = dir_names(d + "/missing_dir")
    if (names_m.len() != 0) {{ return 115 }}
    let names_f = dir_names(d + "/ascii.txt")
    if (names_f.len() != 0) {{ return 116 }}

    let out = l1 + "\n" + l2 + "\n" + l3 + "\n" + l4 + "\n" + l5 + "\n" + l6 + "\n" + l7 + "\n" + l8 + "\n" + l9 + "\n" + l10 + "\n" + l_bad + "\n" + l_neg + "\n" + l_esc
    if (write_text("{rp}", out) != 0) {{ return 117 }}
    return 0
}}
"#
    );

    let (out, store) = run_av_check("d01_d02", &src);
    assert!(
        matches!(out, ExecOutcome::Success { value: ace_av_core::contract::Value::I32(0), .. }),
        "av D-01/D-02 status: {out:?}"
    );
    let report = std::fs::read_to_string(&report_path).unwrap();
    let lines: Vec<&str> = report.lines().collect();
    let expected_058 = [
        r#"{"text":"cdef","total":10,"next_offset":6}"#,
        r#"{"text":"ghij","total":10,"next_offset":null}"#,
        r#"{"text":"","total":10,"next_offset":null}"#,
        r#"{"text":"","total":-1,"next_offset":null}"#,
        r#"{"text":"","total":-1,"next_offset":null}"#,
        r#"{"text":"你","total":12,"next_offset":3}"#,
        r#"{"text":"你","total":12,"next_offset":3}"#,
        r#"{"text":"好","total":12,"next_offset":6}"#,
        r#"{"text":"界","total":12,"next_offset":null}"#,
        r#"{"text":"","total":0,"next_offset":null}"#,
    ];
    assert_eq!(lines.len(), 13, "report lines: {report}");
    for (i, exp) in expected_058.iter().enumerate() {
        assert_eq!(lines[i], *exp, "058 line {} mismatch", i + 1);
    }
    assert_eq!(lines[10], r#"{"text":"","total":-1,"next_offset":null}"#, "bad utf8");
    assert_eq!(lines[11], r#"{"text":"","total":-1,"next_offset":null}"#, "negative offset");
    assert_eq!(lines[12], r#"{"text":"a\"b\\c\nd\te","total":9,"next_offset":null}"#, "json escape");

    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn v02_d03_d06_env_path_process_av() {
    struct RestoreCwd(std::path::PathBuf);
    impl Drop for RestoreCwd {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }
    let _cwd_guard = RestoreCwd(std::env::current_dir().unwrap());

    let tmp = scratch("d03_d06_av");
    let tp = tmp.to_string_lossy().replace('\\', "/");
    let sub = tmp.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    let target_file = tmp.join("canon_target.txt");
    std::fs::write(&target_file, "ok").unwrap();

    let home_exp = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
    let appdata_exp = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    let home_lit = home_exp.replace('\\', "\\\\");
    let appdata_lit = appdata_exp.replace('\\', "\\\\");

    let src = format!(
        r#"
use std.env.get
use std.env.get_or
use std.env.set
use std.env.remove
use std.env.home_dir
use std.env.local_data_dir
use std.path.canonicalize
use std.path.canonical
use std.path.filename
use std.process.args
use std.process.current_dir
use std.process.set_current_dir
use std.process.spawn_with_output
use std.file.write_text
use std.file.delete

fn main() int {{
    let d = "{tp}"

    // D-03: 046_env_get_set + unset vs empty + Unicode + home/local_data
    let rs1 = set("AUTO_TEST_VAR", "hello")
    if (rs1 != 0) {{ return 201 }}
    if (get("AUTO_TEST_VAR") != "hello") {{ return 202 }}
    let rr1 = remove("AUTO_TEST_VAR")
    if (get("AUTO_TEST_VAR") != "") {{ return 203 }}

    let rs2 = set("AUTO_1064_EMPTY", "")
    if (rs2 != 0) {{ return 204 }}
    if (get("AUTO_1064_EMPTY") != "") {{ return 205 }}
    if (get_or("AUTO_1064_EMPTY", "FALLBACK") != "") {{ return 206 }}
    let rr2 = remove("AUTO_1064_EMPTY")

    let rr3 = remove("AUTO_1064_UNSET")
    if (get("AUTO_1064_UNSET") != "") {{ return 207 }}
    if (get_or("AUTO_1064_UNSET", "FALLBACK") != "FALLBACK") {{ return 208 }}

    let rs3 = set("AUTO_1064_UNI", "你好-1064")
    if (get("AUTO_1064_UNI") != "你好-1064") {{ return 209 }}
    let rr4 = remove("AUTO_1064_UNI")

    if (home_dir() != "{home_lit}") {{ return 210 }}
    if (local_data_dir() != "{appdata_lit}") {{ return 211 }}

    // D-04: canonicalize / canonical positive + missing-path negative + handle release
    let c1 = canonicalize(d + "/sub/../canon_target.txt")
    let c2 = canonical(d + "/canon_target.txt")
    if (c1.len() == 0 || c1 != c2) {{ return 220 }}
    if (filename(c1) != "canon_target.txt") {{ return 221 }}
    if (canonicalize(d + "/definitely_missing_1064.txt") != "") {{ return 222 }}
    if (canonical(d + "/definitely_missing_1064.txt") != "") {{ return 223 }}
    if (canonicalize("") != "") {{ return 224 }}
    if (delete(d + "/canon_target.txt") != 0) {{ return 225 }}

    // D-05: process.args + current_dir/set_current_dir roundtrip + restore
    var argv = args()
    if (argv.len() < 1) {{ return 230 }}
    let a0 Str = argv.remove(0)
    if (a0.len() == 0) {{ return 231 }}
    let orig_cwd = current_dir()
    if (orig_cwd.len() == 0) {{ return 232 }}
    if (set_current_dir(d + "/sub") != 0) {{ return 233 }}
    let sub_cwd = current_dir()
    if (set_current_dir(orig_cwd.clone()) != 0) {{ return 234 }}
    if (current_dir() != orig_cwd) {{ return 235 }}
    if (filename(sub_cwd) != "sub") {{ return 236 }}
    return 0
}}
"#
    );

    let (out, store) = run_av_check("d03_d06", &src);
    assert!(
        matches!(out, ExecOutcome::Success { value: ace_av_core::contract::Value::I32(0), .. }),
        "av D-03..D-05 status: {out:?}"
    );
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn v02_d06_spawn_with_output_av() {
    let tmp = scratch("d06_av");
    let report_path = tmp.join("report.txt");
    let rp = report_path.to_string_lossy().replace('\\', "/");

    let src = format!(
        r#"
use std.process.spawn_with_output
use std.file.write_text

fn main() int {{
    let sw1 = spawn_with_output("[\"python\", \"-c\", \"import sys; sys.stdout.write('ok\\\\n\\\\\\\"q\\\\\\\"'); sys.stderr.write('ew'); sys.exit(7)\"]")
    if (write_text("{rp}", sw1) != 0) {{ return 241 }}
    let sw_bad_json = spawn_with_output("not json")
    let sw_empty = spawn_with_output("[]")
    let sw_missing = spawn_with_output("[\"no_such_cmd_1064_xyz.exe\"]")
    let sw_cap = spawn_with_output("[\"python\", \"-c\", \"import sys; sys.stdout.write('a'*70000)\"]")
    if (sw_bad_json != "" || sw_empty != "" || sw_missing != "" || sw_cap != "") {{ return 240 }}
    return 0
}}
"#
    );

    let store = scratch("d06_av_store");
    let product = compile_sys(AssemblyBackend::Av, &src);
    let target = PackageAvTarget::cold(&store, &product).expect("cold link");
    let mut backend = Win32PlatformIo::default();
    let limits = ace_av_core::av::ExecLimits {
        max_steps: 5_000_000,
        ..Default::default()
    };
    let (out, _trace, stats) = ace_av_core::execute_program_platform(
        &target.current,
        &limits,
        &ace_av_core::CancelFlag::default(),
        &sys_grants(),
        None,
        &mut backend,
    )
    .expect("av exec");
    assert!(
        matches!(out, ExecOutcome::Success { value: ace_av_core::contract::Value::I32(0), .. }),
        "av D-06 status: {out:?}, stats={stats:?}"
    );
    let sw1 = std::fs::read_to_string(&report_path).unwrap();
    assert!(
        sw1.contains(r#""exit_code":7"#) && sw1.contains(r#""stderr":"ew""#) && sw1.contains(r#"\"q\""#),
        "unexpected spawn_with_output JSON: {sw1}"
    );

    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&tmp);
}

fn shared_cross_end_source(tp: &str, rp: &str) -> String {
    format!(
        r#"
use std.file.read_text
use std.file.read_text_range
use std.file.write_text
use std.file.delete
use std.file.dir_names
use std.env.get
use std.env.get_or
use std.env.set
use std.env.remove
use std.env.home_dir
use std.env.local_data_dir
use std.path.canonicalize
use std.path.canonical
use std.path.filename
use std.process.args
use std.process.current_dir
use std.process.set_current_dir
use std.process.spawn_with_output

fn check_cross_names(xs List<Str>) int {{
    if (xs.len() == 0) {{ return 0 }}
    let item Str = xs.remove(0)
    if (item == "." || item == "..") {{ return -10000 }}
    var w = 0
    if (item == "cjk.txt") {{ w = 1 }}
    let rest = check_cross_names(xs)
    if (rest < 0) {{ return rest }}
    return w + rest
}}

fn main() int {{
    let d = "{tp}"
    // D-01
    if (read_text(d + "/cjk.txt") != "你好世界") {{ return 101 }}
    if (read_text(d + "/bad.bin") != "") {{ return 102 }}
    if (read_text_range(d + "/cjk.txt", 2, 4) != "{{\"text\":\"你\",\"total\":12,\"next_offset\":3}}") {{ return 103 }}
    if (read_text_range(d + "/no_such.txt", 0, 4) != "{{\"text\":\"\",\"total\":-1,\"next_offset\":null}}") {{ return 104 }}
    // D-02
    let dns = dir_names(d.clone())
    if (check_cross_names(dns) != 1) {{ return 105 }}
    let dns_m = dir_names(d + "/missing")
    if (dns_m.len() != 0) {{ return 107 }}
    // D-03
    let s1 = set("AUTO_TEST_VAR", "hello")
    if (get("AUTO_TEST_VAR") != "hello") {{ return 108 }}
    let r1 = remove("AUTO_TEST_VAR")
    let s2 = set("AUTO_X_EMPTY", "")
    if (get_or("AUTO_X_EMPTY", "FB") != "") {{ return 109 }}
    let r2 = remove("AUTO_X_UNSET")
    if (get_or("AUTO_X_UNSET", "FB") != "FB") {{ return 110 }}
    if (home_dir().len() == 0 || local_data_dir().len() == 0) {{ return 111 }}
    // D-04
    let c1 = canonicalize(d + "/sub/../cjk.txt")
    let c2 = canonical(d + "/cjk.txt")
    if (c1.len() == 0 || filename(c1) != "cjk.txt" || filename(c2) != "cjk.txt") {{ return 112 }}
    if (canonicalize(d + "/missing.txt") != "" || canonical(d + "/missing.txt") != "") {{ return 113 }}
    // D-05: real CLI args with quoted spaces, empty arg, quote, Unicode + cwd roundtrip
    var argv = args()
    let argc = argv.len()
    if (argc < 6) {{ return 114 }}
    let tail4 Str = argv.remove(argc - 1)
    let tail3 Str = argv.remove(argc - 2)
    let tail2 Str = argv.remove(argc - 3)
    let tail1 Str = argv.remove(argc - 4)
    let tail0 Str = argv.remove(argc - 5)
    if (tail0 != "alpha" || tail1 != "two words" || tail2 != "" || tail3 != "a\"b" || tail4 != "中文-1064") {{ return 115 }}
    let orig_cwd = current_dir()
    if (set_current_dir(d + "/sub") != 0) {{ return 116 }}
    let sc = current_dir()
    let rs = set_current_dir(orig_cwd.clone())
    if (filename(sc) != "sub" || current_dir() != orig_cwd) {{ return 117 }}
    // D-06
    let sw = spawn_with_output("[\"python\", \"-c\", \"import sys; sys.stdout.write(sys.argv[1]); sys.exit(3)\", \"two words\"]")
    if (spawn_with_output("[]") != "" || spawn_with_output("[\"missing_cmd_1064.exe\"]") != "") {{ return 118 }}
    if (spawn_with_output("[\"python\", \"-c\", \"import sys; sys.stdout.write('a'*70000)\"]") != "") {{ return 119 }}
    if (write_text("{rp}", sw) != 0) {{ return 120 }}
    return 0
}}
"#
    )
}

#[cfg(feature = "test-toolchain")]
#[test]
fn v02_d01_d06_a2r_cross_end() {
    let tmp = scratch("d01_d06_a2r_disk");
    std::fs::create_dir_all(tmp.join("sub")).unwrap();
    std::fs::write(tmp.join("cjk.txt"), "你好世界").unwrap();
    std::fs::write(tmp.join("bad.bin"), &[0xFF, 0xFE]).unwrap();
    let report = tmp.join("report.txt");

    let tp = tmp.to_string_lossy().replace('\\', "/");
    let rp = report.to_string_lossy().replace('\\', "/");
    let src = shared_cross_end_source(&tp, &rp);

    let prod = compile_sys(AssemblyBackend::A2R, &src);
    let store = scratch("d01_d06_a2r_store");
    let r_t = PackageA2xTarget::cold_aggregates(&store, "d16r", "app", 16 << 20, RustTargetProvider::v1(), &prod).unwrap();
    let bdir = scratch("d01_d06_a2r_build");
    for (p, c) in &r_t.current.files {
        std::fs::write(bdir.join(p), c).unwrap();
    }
    let exe = bdir.join("d16r.exe");
    let mut cmd = std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"]).arg(&exe);
    cmd.current_dir(&bdir);
    let compile = auto_tool_execution::run_command_capture(cmd, auto_tool_execution::TreeScope::ProcessTree, None).unwrap();
    assert_eq!(compile.kind, auto_tool_execution::ExitKind::Success, "{}", String::from_utf8_lossy(compile.stderr.as_deref().unwrap_or(&[])));

    let run = std::process::Command::new(&exe)
        .current_dir(&bdir)
        .args(["alpha", "two words", "", "a\"b", "中文-1064"])
        .output()
        .unwrap();
    let sw = std::fs::read_to_string(&report).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&bdir);
    let _ = std::fs::remove_dir_all(&tmp);
    assert!(run.status.success(), "a2r exit: {:?}", run.status.code());
    assert_eq!(sw, r#"{"exit_code":3,"stdout":"two words","stderr":""}"#);
}

#[cfg(feature = "test-toolchain")]
#[test]
fn v02_d01_d06_a2py_cross_end() {
    let tmp = scratch("d01_d06_a2py_disk");
    std::fs::create_dir_all(tmp.join("sub")).unwrap();
    std::fs::write(tmp.join("cjk.txt"), "你好世界").unwrap();
    std::fs::write(tmp.join("bad.bin"), &[0xFF, 0xFE]).unwrap();
    let report = tmp.join("report.txt");

    let tp = tmp.to_string_lossy().replace('\\', "/");
    let rp = report.to_string_lossy().replace('\\', "/");
    let src = shared_cross_end_source(&tp, &rp);

    let prod = compile_sys(AssemblyBackend::A2Py, &src);
    let store = scratch("d01_d06_a2py_store");
    let py_t = PackageA2xTarget::cold_aggregates(&store, "d16py", "app", 16 << 20, PythonTargetProvider, &prod).unwrap();
    let pdir = scratch("d01_d06_a2py_build");
    for (p, c) in &py_t.current.files {
        std::fs::write(pdir.join(p), c).unwrap();
    }
    let rt = ace_a2x_python::python_runtime_source(&[]);
    let rt_hash = ace_core::runtime_contract::sha256_hex(rt.as_bytes());
    let tool = ace_a2x_worker::pyrun::resolve_python(&rt_hash).expect("python");
    let run = std::process::Command::new(&tool.program)
        .args(&tool.pre_args)
        .args(["-I", "-B", "a2x_main.py"])
        .args(["alpha", "two words", "", "a\"b", "中文-1064"])
        .current_dir(&pdir)
        .output()
        .unwrap();
    let sw = std::fs::read_to_string(&report).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&store);
    let _ = std::fs::remove_dir_all(&tmp);
    assert!(
        run.status.success() && sw == r#"{"exit_code":3,"stdout":"two words","stderr":""}"#,
        "a2py exit={:?} sw={:?} pdir={} stderr={} stdout={}",
        run.status.code(),
        sw,
        pdir.display(),
        String::from_utf8_lossy(&run.stderr),
        String::from_utf8_lossy(&run.stdout)
    );
    let _ = std::fs::remove_dir_all(&pdir);
}
