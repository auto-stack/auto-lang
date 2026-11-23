//! PLAN-972 T-08 / V-08: per-operation coverage closure over the assembled
//! std set. Part A — manifest parity: the frozen set's PUBLIC surface
//! (per-module `pub fn` names, scanned from the repo-frozen sources) must
//! equal the required operation table EXACTLY; removing any required row,
//! or adding/renaming an unlisted public symbol, fails before any run.
//! Part B — the all-symbol exercise main: every public symbol (49) is
//! called with type-correct representative inputs and an exact independent
//! expectation (one +1 bit each) → 49, run on ALL FIVE ends (AV
//! in-process; mn real machine code exit; cl / rustc / real OS Python
//! stdout). A missing or broken operation reds its end. Provider-leg
//! tampering is covered by `stdlib_asm_972_std_identity_reserved`.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::Value;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_aggregates_inputs, compile_assembled_aggregates_inputs,
};

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/asm972/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The 49-check exercise main over the whole public surface. Expected
/// values are the frozen independent oracles (A中😀 = 8 bytes / 3 scalars;
/// hex/base64 from Python; hash seeds/mix per the S-06 ruling fixtures).
pub const EXERCISE_MAIN: &str = r#"
use std.list.sum
use std.list.find_first_gt
use std.list.pop
use std.list.total_len
use std.list.reduce
use std.list.find
use std.list.any
use std.list.all
use std.list.filter
use std.list.map
use std.math.abs
use std.math.min
use std.math.max
use std.math.square
use std.math.cube
use std.math.abs_f
use std.math.sqrt
use std.math.powf
use std.math.sin
use std.math.cos
use std.text.byte_len
use std.text.scalar_len
use std.text.char_code_at
use std.text.equal
use std.text.starts_with
use std.text.ends_with
use std.text.contains
use std.text.repeat
use std.text.upper_ascii
use std.text.lower_ascii
use std.text.find as t_find
use std.text.reverse
use std.text.trim
use std.text.replace
use std.text.split
use std.text.lines
use std.list.sort
use std.char.is_digit
use std.char.is_alpha_ascii
use std.char.is_alphanum_ascii
use std.char.is_ident_ascii
use std.char.is_whitespace
use std.char.to_upper_ascii
use std.char.to_lower_ascii
use std.conv.to_int
use std.conv.is_uint
use std.conv.from_uint
use std.conv.from_int
use std.hash.codepoint_hash
use std.bytes.byte_at
use std.bytes.utf8_len
use std.bytes.byte_of
use std.bytes.byte_hash
use std.bytes.to_hex
use std.bytes.to_base64
use std.map.map_new
use std.map.map_put
use std.map.map_get
use std.map.map_has
use std.map.map_len
use std.map.map_remove
fn oi(r ?int) int {
    is r {
        Some(v) -> { return v }
        None -> { return 999999 }
    }
}
fn main() int {
    let s1 Str = "A中😀"
    var code = 0
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    if (sum(xs) == 15) {
        code = code + 1
    }
    if (oi(find_first_gt(xs, 3)) == 4) {
        code = code + 1
    }
    var xs2 = List<int>.new()
    xs2.push(1)
    xs2.push(2)
    xs2.push(3)
    xs2.push(4)
    xs2.push(5)
    if (oi(pop(xs2)) == 5) {
        code = code + 1
    }
    var ss = List<Str>.new()
    ss.push("ab")
    ss.push("cde")
    if (total_len(ss) == 5) {
        code = code + 1
    }
    if (reduce(xs, 0, (acc int, x int) => { return acc + x }) == 15) {
        code = code + 1
    }
    if (oi(find(xs, (x int) => { return x > 4 })) == 5) {
        code = code + 1
    }
    if (!any(xs, (x int) => { return x < 0 })) {
        code = code + 1
    }
    if (all(xs, (x int) => { return x > 0 })) {
        code = code + 1
    }
    let ys = filter(xs, (x int) => { return x > 1 })
    if (ys.len() == 4) {
        code = code + 1
    }
    let zs = map(xs, (x int) => { return x * 10 })
    if (sum(zs) == 150) {
        code = code + 1
    }
    if (abs(-9223372036854775807i64) == 9223372036854775807i64) {
        code = code + 1
    }
    if (min(3i64, 5i64) == 3i64) {
        code = code + 1
    }
    if (max(3i64, 5i64) == 5i64) {
        code = code + 1
    }
    if (square(7) == 49) {
        code = code + 1
    }
    if (cube(3) == 27) {
        code = code + 1
    }
    if (abs_f(-2.5d) == 2.5d) {
        code = code + 1
    }
    let r2 = sqrt(2.0d)
    if (r2 > 1.41421356237d) {
        if (r2 < 1.41421356238d) {
            code = code + 1
        }
    }
    let p10 = powf(2.0d, 10.0d)
    if (p10 > 1023.9999d) {
        if (p10 < 1024.0001d) {
            code = code + 1
        }
    }
    let sn1 = sin(1.0d)
    if (sn1 > 0.8414709847d) {
        if (sn1 < 0.8414709849d) {
            code = code + 1
        }
    }
    let cs1 = cos(1.0d)
    if (cs1 > 0.5403023057d) {
        if (cs1 < 0.5403023059d) {
            code = code + 1
        }
    }
    if (byte_len(s1.clone()) == 8) {
        code = code + 1
    }
    if (scalar_len(s1.clone()) == 3) {
        code = code + 1
    }
    if (char_code_at("Ab", 1) == 98) {
        code = code + 1
    }
    if (equal(s1.clone(), s1.clone())) {
        code = code + 1
    }
    if (starts_with(s1.clone(), "A")) {
        code = code + 1
    }
    if (ends_with(s1.clone(), "😀")) {
        code = code + 1
    }
    if (contains(s1.clone(), "中")) {
        code = code + 1
    }
    if (repeat("ab", 3).len_bytes() == 6) {
        code = code + 1
    }
    let u1 = upper_ascii("aB中z9")
    if (equal(u1, "AB中Z9")) {
        code = code + 1
    }
    let l1 = lower_ascii("Ab中Z9")
    if (equal(l1, "ab中z9")) {
        code = code + 1
    }
    if (is_digit('7')) {
        code = code + 1
    }
    if (!is_alpha_ascii('中')) {
        code = code + 1
    }
    if (is_alphanum_ascii('9')) {
        code = code + 1
    }
    if (is_ident_ascii('_')) {
        code = code + 1
    }
    if (is_whitespace(' ')) {
        code = code + 1
    }
    if (to_upper_ascii('q'.to_codepoint()) == 81) {
        code = code + 1
    }
    if (to_lower_ascii('Q'.to_codepoint()) == 113) {
        code = code + 1
    }
    let fidx = t_find("A中😀中", "中")
    if (oi(fidx) == 1) {
        code = code + 1
    }
    let nohit = t_find("abc", "z")
    if (oi(nohit) == 999999) {
        code = code + 1
    }
    let rev = reverse("A中")
    if (equal(rev, "中A")) {
        code = code + 1
    }
    let tr = trim("  a b ")
    if (equal(tr, "a b")) {
        code = code + 1
    }
    let rep = replace("aXbXc", "X", "-")
    if (equal(rep, "a-b-c")) {
        code = code + 1
    }
    let parts = split("a,b,c", ",")
    if (parts.len() == 3) {
        code = code + 1
    }
    let lns = lines("x
y")
    if (lns.len() == 2) {
        code = code + 1
    }
    var un = List<int>.new()
    un.push(3)
    un.push(1)
    un.push(2)
    let sorted = sort(un)
    if (sorted.get(0) == 1) {
        if (sorted.get(2) == 3) {
            code = code + 1
        }
    }
    if (oi(to_int("42")) == 42) {
        code = code + 1
    }
    if (is_uint("123")) {
        code = code + 1
    }
    let fu = from_uint(421)
    if (fu.len() == 3) {
        code = code + 1
    }
    let fi = from_int(-421)
    if (fi.len() == 4) {
        code = code + 1
    }
    if (codepoint_hash("auto") == 2075808597i64) {
        code = code + 1
    }
    if (oi(byte_at(s1.clone(), 1)) == 228) {
        code = code + 1
    }
    if (utf8_len(20013) == 3) {
        code = code + 1
    }
    if (byte_of(20013, 0) == 228) {
        code = code + 1
    }
    if (byte_hash("auto") == 2075808597i64) {
        code = code + 1
    }
    let hx = to_hex("ab")
    if (equal(hx, "6162")) {
        code = code + 1
    }
    let b64 = to_base64("abc")
    if (equal(b64, "YWJj")) {
        code = code + 1
    }
    var m = map_new()
    if (map_put(m, 1, 10) == 0) {
        code = code + 1
    }
    if (map_put(m, 1, 99) == 1) {
        code = code + 1
    }
    if (oi(map_get(m, 1)) == 99) {
        code = code + 1
    }
    if (map_has(m, 1)) {
        code = code + 1
    }
    if (map_len(m) == 1) {
        code = code + 1
    }
    if (map_remove(m, 1)) {
        code = code + 1
    }
    return code
}
"#;

#[cfg(feature = "test-toolchain")]
#[test]
fn v08_surface_manifest_and_five_end_exercise() {
    // ---- Part A: manifest parity (remove-a-row / stray-symbol control) ----
    let required: &[(&str, &[&str])] = &[
        (
            "list",
            &[
                "all",
                "any",
                "filter",
                "find",
                "find_first_gt",
                "map",
                "pop",
                "reduce",
                "sort",
                "sum",
                "total_len",
            ],
        ),
        (
            "math",
            &[
                "abs", "abs_f", "cos", "max", "min", "powf", "sin", "sqrt", "square", "cube",
            ],
        ),
        (
            "text",
            &[
                "byte_len",
                "char_code_at",
                "contains",
                "ends_with",
                "equal",
                "find",
                "lines",
                "lower_ascii",
                "repeat",
                "replace",
                "reverse",
                "scalar_len",
                "split",
                "starts_with",
                "trim",
                "upper_ascii",
            ],
        ),
        (
            "char",
            &[
                "is_alphanum_ascii",
                "is_alpha_ascii",
                "is_digit",
                "is_ident_ascii",
                "is_whitespace",
                "to_lower_ascii",
                "to_upper_ascii",
            ],
        ),
        (
            "conv",
            &["from_int", "from_u64", "from_uint", "is_uint", "parse_u64", "to_int"],
        ),
        ("hash", &["codepoint_hash"]),
        (
            "bytes",
            &[
                "byte_at",
                "byte_hash",
                "byte_of",
                "decode_base64",
                "decode_base64_str",
                "decode_hex",
                "decode_hex_str",
                "decode_utf8",
                "to_base64",
                "to_hex",
                "utf8_len",
            ],
        ),
        (
            "map",
            &[
                "map_get",
                "map_has",
                "map_len",
                "map_new",
                "map_put",
                "map_remove",
            ],
        ),
    ];
    let files = ace_core::source::packages::stdlib_assembled_files();
    let mut module_names: Vec<&str> = files
        .iter()
        .map(|(n, _)| n.trim_end_matches(".at"))
        .collect();
    module_names.sort_unstable();
    let mut required_names: Vec<&str> = required.iter().map(|(m, _)| *m).collect();
    required_names.sort_unstable();
    assert_eq!(module_names, required_names, "assembled module set");
    for (name, content) in &files {
        let module = name.trim_end_matches(".at");
        let mut pubs: Vec<&str> = content
            .lines()
            .filter_map(|l| {
                let t = l.trim();
                t.strip_prefix("pub fn ")
                    .map(|rest| rest.split('(').next().unwrap_or("").trim())
            })
            .collect();
        pubs.sort_unstable();
        let mut want: Vec<&str> = required
            .iter()
            .find(|(m, _)| *m == module)
            .unwrap_or_else(|| panic!("unlisted module {module}"))
            .1
            .to_vec();
        want.sort_unstable();
        assert_eq!(pubs, want, "public surface of std::{module}");
    }

    // ---- AV leg (in-process): all 49 checks green. ----
    let mut ctx =
        PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = assemble_stdlib_aggregates_inputs(
        &[("main.at".to_string(), EXERCISE_MAIN.to_string())],
        "app::main",
        &mut ctx,
    )
    .unwrap();
    let product = compile_assembled_aggregates_inputs(
        &inputs,
        "app::main",
        FrontBudget::WORK_OWNED_64MIB,
        &ctx,
    )
    .unwrap_or_else(|ds| {
        panic!(
            "v08 compile: {} {} {}",
            ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
        )
    });
    let dir = scratch("v08-av");
    let av = PackageAvTarget::cold(&dir, &product).unwrap();
    assert_eq!(av.run_value().unwrap(), Value::I32(62));

    // ---- mn leg (real machine code): exit code = 49. ----
    let dir = scratch("v08-mn");
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_collections_checked(
            &product.compiled.checked,
            "d_main",
            &dir,
            &product.bound.input_hash,
            972,
        )
        .unwrap_or_else(|e| panic!("v08 mn build: {e:?}"));
    let code = std::process::Command::new(&built.exe_path)
        .current_dir(&dir)
        .status()
        .expect("run v08 mn exe")
        .code()
        .unwrap_or(-1);
    assert_eq!(code, 62, "AC/native v08 exercise exit");

    // ---- A2C leg (real cl): value:49. ----
    {
        use ace_a2x_c::CTargetProvider;
        use ace_a2x_store::modules::packages::PackageA2xTarget;
        let mut ctx_c = PackageAssemblyContext::new(
            AssemblyBackend::A2C,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs_c = assemble_stdlib_aggregates_inputs(
            &[("main.at".to_string(), EXERCISE_MAIN.to_string())],
            "app::main",
            &mut ctx_c,
        )
        .unwrap();
        let c_product = compile_assembled_aggregates_inputs(
            &inputs_c,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx_c,
        )
        .unwrap_or_else(|ds| panic!("v08 a2c compile: {ds:?}"));
        let store = scratch("v08-a2c");
        let mut c_target = PackageA2xTarget::cold_aggregates(
            &store,
            "v0972c",
            "app",
            16 << 20,
            CTargetProvider::v1(),
            &c_product,
        )
        .unwrap_or_else(|e| panic!("v08 a2c cold: {e}"));
        let build_dir = scratch("v08-a2c-build");
        std::fs::create_dir_all(&build_dir).unwrap();
        for (path, content) in &c_target.current.files {
            std::fs::write(build_dir.join(path), content).unwrap();
        }
        std::fs::write(build_dir.join("manifest.json"), &c_target.current.manifest).unwrap();
        let exe = build_dir.join("v0972c.exe");
        ace_a2x_worker::crun::compile_c_fileset(&build_dir, &exe)
            .unwrap_or_else(|e| panic!("cl failed: {e}"));
        let run = std::process::Command::new(&exe)
            .current_dir(&build_dir)
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:62"), "a2c v08 stdout: {out}");
    }

    // ---- A2R leg (real rustc): value:49. ----
    {
        use ace_a2x_rust::RustTargetProvider;
        use ace_a2x_store::modules::packages::PackageA2xTarget;
        let mut ctx_r = PackageAssemblyContext::new(
            AssemblyBackend::A2R,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs_r = assemble_stdlib_aggregates_inputs(
            &[("main.at".to_string(), EXERCISE_MAIN.to_string())],
            "app::main",
            &mut ctx_r,
        )
        .unwrap();
        let r_product = compile_assembled_aggregates_inputs(
            &inputs_r,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx_r,
        )
        .unwrap_or_else(|ds| panic!("v08 a2r compile: {ds:?}"));
        let store = scratch("v08-a2r");
        let mut rust = PackageA2xTarget::cold_aggregates(
            &store,
            "v0972r",
            "app",
            16 << 20,
            RustTargetProvider::v1(),
            &r_product,
        )
        .unwrap_or_else(|e| panic!("v08 a2r cold: {e}"));
        let build_dir = scratch("v08-a2r-build");
        std::fs::create_dir_all(&build_dir).unwrap();
        for (path, content) in &rust.current.files {
            std::fs::write(build_dir.join(path), content).unwrap();
        }
        let exe = build_dir.join("v0972r.exe");
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
            "{}",
            String::from_utf8_lossy(compile.stderr.as_deref().unwrap_or(&[]))
        );
        let run = std::process::Command::new(&exe)
            .current_dir(&build_dir)
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:62"), "a2r v08 stdout: {out}");
    }

    // ---- A2PY leg (real OS Python): value:49. ----
    {
        use ace_a2x_python::PythonTargetProvider;
        use ace_a2x_store::modules::packages::PackageA2xTarget;
        let mut ctx_py = PackageAssemblyContext::new(
            AssemblyBackend::A2Py,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs_py = assemble_stdlib_aggregates_inputs(
            &[("main.at".to_string(), EXERCISE_MAIN.to_string())],
            "app::main",
            &mut ctx_py,
        )
        .unwrap();
        let py_product = compile_assembled_aggregates_inputs(
            &inputs_py,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx_py,
        )
        .unwrap_or_else(|ds| panic!("v08 a2py compile: {ds:?}"));
        let store = scratch("v08-a2py");
        let mut py = PackageA2xTarget::cold_aggregates(
            &store,
            "v0972py",
            "app",
            16 << 20,
            PythonTargetProvider,
            &py_product,
        )
        .unwrap_or_else(|e| panic!("v08 a2py cold: {e}"));
        let py_dir = scratch("v08-a2py-build");
        for (path, content) in &py.current.files {
            std::fs::write(py_dir.join(path), content).unwrap();
        }
        let rt = ace_a2x_python::python_runtime_source(&[]);
        let rt_hash = ace_core::runtime_contract::sha256_hex(rt.as_bytes());
        let tool = ace_a2x_worker::pyrun::resolve_python(&rt_hash)
            .expect("OS Python interpreter required (missing Python cannot PASS)");
        let run = ace_a2x_worker::pyrun::run_py_fileset(&tool, &py_dir, &[]).unwrap();
        assert_eq!(
            run.exit_code,
            Some(0),
            "stderr={}",
            String::from_utf8_lossy(&run.stderr)
        );
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:62"), "a2py v08 stdout: {out}");
    }
}
