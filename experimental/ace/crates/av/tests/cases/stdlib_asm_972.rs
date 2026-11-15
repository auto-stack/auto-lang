//! PLAN-972 T-02 assembled stdlib conformance: the frozen assembled std set
//! (`std.list` — the 946 slice superset + closure-HOF family
//! reduce/find/any/all/filter/map) through the packages-aggregates-ct lane
//! (`assemble_stdlib_aggregates_inputs` + `compile_assembled_aggregates_package`)
//! observed on the AV end with independent hand-computed oracles; std identity
//! reservation (tampered source / provider layers) rejects pre-run; closure
//! params stay rejected on the plain aggregates lane (lane isolation).
//! Four-end producer legs, store roundtrip and fault paths are T-03/T-04.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::Value;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, PackageImplLayer, PackageModuleInput, ProviderChoice,
    ProviderKind, TargetOs, STDLIB_PACKAGE, STDLIB_VERSION,
};
use ace_source_query::modules::packages::{
    assemble_stdlib_aggregates_inputs, compile_assembled_aggregates_inputs,
    compile_package_product_aggregates, stdlib_aggregates_inputs,
};

/// The V-DETAIL-01 program shape over the HOF family:
/// reduce=15, find(>3)=Some(4), all(>0)=true, any(<0)=false,
/// filter(>1)=[2,3,4,5] (len=4 sum=14), map(*10) sum=140
/// → 15*10^5 + 4*10^4 + 4*10^3 + 14*10 + 140 + 1 = 1544281.
const HOF_MAIN: &str = r#"
use std.list.reduce
use std.list.find
use std.list.any
use std.list.all
use std.list.filter
use std.list.map
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let s = reduce(xs, 0, (acc int, x int) => { return acc + x })
    let f = find(xs, (x int) => { return x > 3 })
    var got = 0
    is f {
        Some(v) -> { got = v }
        None -> { got = 0 }
    }
    let ys = filter(xs, (x int) => { return x > 1 })
    let zs = map(ys, (x int) => { return x * 10 })
    var ys_sum = 0
    for y in ys {
        ys_sum = ys_sum + y
    }
    var zs_sum = 0
    for z in zs {
        zs_sum = zs_sum + z
    }
    var code = s * 100000 + got * 10000 + ys.len() * 1000 + ys_sum * 10 + zs_sum
    if all(xs, (x int) => { return x > 0 }) {
        code = code + 1
    }
    if any(xs, (x int) => { return x < 0 }) {
        code = code + 2
    }
    return code
}
"#;

const SLICE_MAIN: &str = r#"
use std.list.sum
use std.list.find_first_gt
use std.list.pop
use std.list.total_len
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let s = sum(xs)
    let f = find_first_gt(xs, 3)
    var got = 0
    is f {
        Some(v) -> { got = v }
        None -> { got = 0 }
    }
    let p = pop(xs)
    var popped = 0
    is p {
        Some(v) -> { popped = v }
        None -> { popped = 0 }
    }
    var ss = List<Str>.new()
    ss.push("ab")
    ss.push("cde")
    return s * 1000 + got * 100 + popped * 10 + total_len(ss)
}
"#;

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/asm972/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn asm_lane_product(
    tag: &str,
    main_src: &str,
) -> ace_source_query::modules::packages::PackageProduct {
    let mut ctx = PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = assemble_stdlib_aggregates_inputs(
        &[("main.at".to_string(), main_src.to_string())],
        "app::main",
        &mut ctx,
    )
    .unwrap_or_else(|e| panic!("{tag} assemble: {e}"));
    compile_assembled_aggregates_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
        .unwrap_or_else(|ds| {
            panic!(
                "{tag} compile: {} {} {}",
                ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
            )
        })
}

fn run_av(tag: &str, product: &ace_source_query::modules::packages::PackageProduct) -> Value {
    let av =
        PackageAvTarget::cold(&scratch(&format!("av-{tag}")), product).unwrap_or_else(|e| panic!("av {tag}: {e}"));
    av.run_value().unwrap_or_else(|e| panic!("run {tag}: {e}"))
}

/// The HOF family + slice superset on AV×Auto through the assembled
/// aggregates lane, with hand-set oracles (1544281 / 15455).
#[test]
fn stdlib_asm_972_hof_and_slice_av() {
    let p = asm_lane_product("hof", HOF_MAIN);
    assert_eq!(run_av("hof", &p), Value::I32(1544281));

    let p = asm_lane_product("slice", SLICE_MAIN);
    // sum=15, find_first_gt(3)=4, pop=5, total_len(["ab","cde"])=5
    // → 15*1000 + 4*100 + 5*10 + 5 = 15455.
    assert_eq!(run_av("slice", &p), Value::I32(15455));
}

/// The S-01 i64 family + int square through the assembled lane on AV:
/// abs(-i64::MAX)=MAX, min=3, max=5, square(7)=49
/// → 9223372036854775807 - 3*10 - 5*100 - 49 = 9223372036854775228.
const MATH_MAIN: &str = r#"
use std.math.abs
use std.math.min
use std.math.max
use std.math.square
fn main() i64 {
    let a = abs(-9223372036854775807i64)
    let lo = min(3i64, 5i64)
    let hi = max(3i64, 5i64)
    return a - lo * 10i64 - hi * 100i64 - square(7).to(i64)
}
"#;

#[test]
fn stdlib_asm_972_math_i64_av() {
    let p = asm_lane_product("math", MATH_MAIN);
    assert_eq!(run_av("math", &p), Value::I64(9223372036854775228));
}

/// V-02 boundary: abs(i64::MIN) negation traps IntOverflow under the
/// checked-arithmetic contract (PLAN-961) — independently asserted, never
/// wrapped silently.
#[test]
fn stdlib_asm_972_math_abs_min_boundary_traps() {
    const ABS_MIN: &str = "use std.math.abs\nfn main() i64 { return abs(-9223372036854775808i64) }";
    let p = asm_lane_product("abs-min", ABS_MIN);
    let dir = scratch("av-abs-min");
    let av = PackageAvTarget::cold(&dir, &p).unwrap();
    let e = av.run_value().unwrap_err();
    assert!(
        e.to_lowercase().contains("overflow") || e.to_lowercase().contains("trap"),
        "abs(i64::MIN) must trap (checked arithmetic), got: {e}"
    );
}

/// The S-02 text/char slice through the assembled lane on AV, with the V-02
/// Unicode unit oracles: A中😀 = 8 bytes / 3 scalars; e+U+0301 = 3 bytes;
/// NUL does not truncate (explicit lengths); equal/starts_with/ends_with/
/// contains/repeat + char predicates. Scored bits:
/// 8×10^7 + 3×10^6 + 3×10^5 + equal 4×10^4 + starts 2×10^3 + ends 10^3
/// + contains 100 + repeat("ab",3).len_bytes()=6 ×10 + digit 4 + alpha 0
/// + whitespace 1 = 83343165.
const TEXT_MAIN: &str = r#"
use std.text.byte_len
use std.text.scalar_len
use std.text.equal
use std.text.starts_with
use std.text.ends_with
use std.text.contains
use std.text.repeat
use std.char.is_digit
use std.char.is_alpha_ascii
use std.char.is_whitespace
fn main() int {
    let a Str = "A中😀"
    let e Str = "é"
    var code = byte_len(a.clone()) * 10000000
    code = code + scalar_len(a.clone()) * 1000000
    code = code + byte_len(e.clone()) * 100000
    if equal(a.clone(), a.clone()) {
        code = code + 40000
    }
    if starts_with(a.clone(), "A") {
        code = code + 2000
    }
    if ends_with(a.clone(), "😀") {
        code = code + 1000
    }
    if contains(a.clone(), "中") {
        code = code + 100
    }
    code = code + repeat("ab", 3).len_bytes() * 10
    if is_digit('7') {
        code = code + 4
    }
    if is_alpha_ascii('中') {
        code = code + 2
    }
    if is_whitespace(' ') {
        code = code + 1
    }
    return code
}
"#;

/// V-02 NUL discipline: an embedded NUL never truncates — byte_len counts
/// past it and equal distinguishes the tail ("a\0b" = 3 bytes ≠ "a" = 1).
const NUL_MAIN: &str = r#"
use std.text.byte_len
use std.text.equal
fn main() int {
    let s Str = "a\0b"
    var code = byte_len(s.clone()) * 10
    if equal(s.clone(), "a") {
        code = code + 5
    }
    if equal(s.clone(), "a\0b") {
        code = code + 1
    }
    return code
}
"#;

#[test]
fn stdlib_asm_972_text_char_av() {
    let p = asm_lane_product("text", TEXT_MAIN);
    assert_eq!(run_av("text", &p), Value::I32(83343165));
    let p = asm_lane_product("nul", NUL_MAIN);
    assert_eq!(run_av("nul", &p), Value::I32(31));
}

/// T-03 consumer legs: the SAME assembled-lane product on the real external
/// ends — AC/native (mn exe exit, collections bridge) and A2R (rustc exe
/// stdout, aggregates family). Non-closure slice first (sum=15, gt3=4 →
/// 1504); closure HOF legs ride the A2R end below (the mn collections
/// bridge carries no closure VALUES yet — named gap, PLAN-972 T-03/T-09).
const CONSUMER_MAIN: &str = r#"
use std.list.sum
use std.list.find_first_gt
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let s = sum(xs)
    let f = find_first_gt(xs, 3)
    is f {
        Some(v) -> { return s * 100 + v }
        None -> { return -1 }
    }
}
"#;

#[test]
fn stdlib_asm_972_cross_end_gaps_and_a2r_math() {
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::packages::PackageA2xTarget;
    use ace_source_query::modules::packages::assemble_stdlib_aggregates_inputs;
    use std::process::Command;

    let compile_asm = |src: &str| {
        let mut ctx = PackageAssemblyContext::new(
            AssemblyBackend::Av,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs = assemble_stdlib_aggregates_inputs(
            &[("main.at".to_string(), src.to_string())],
            "app::main",
            &mut ctx,
        )
        .unwrap();
        compile_assembled_aggregates_inputs(
            &inputs,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx,
        )
        .unwrap_or_else(|ds| {
            panic!(
                "consumer compile: {} {} {}",
                ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
            )
        })
    };
    // ---- AC / native (mn) WHOLE-SET leg (PLAN-972 T-03: capture-free
    // closure VALUES on the collections path — the callee binds through
    // its unit import; the indirect call rides the cells ABI): the HOF
    // program over the FULL assembled std set — filter([1..5], >1) len=4,
    // reduce sum=14 → (14%10)*10 + 4 = 44 exit on real machine code. ----
    let hof_main = "use std.list.reduce
use std.list.filter
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let ys = filter(xs, (x int) => { return x > 1 })
    let s = reduce(ys, 0, (acc int, x int) => { return acc + x })
    return (s % 10) * 10 + ys.len()
}";
    let whole_hof = compile_asm(hof_main);
    let dir = scratch("cons-mn-whole");
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_collections_checked(
            &whole_hof.compiled.checked,
            "d_main",
            &dir,
            &whole_hof.bound.input_hash,
            972,
        )
        .unwrap_or_else(|e| panic!("mn whole-set build: {e:?}"));
    let code = Command::new(&built.exe_path)
        .current_dir(&dir)
        .status()
        .expect("run mn whole-set exe")
        .code()
        .unwrap_or(-1);
    assert_eq!(code, 44, "AC/native whole-set exit (HOF closures)");

    // ---- A2C / cl WHOLE-SET HOF leg (PLAN-972 T-03: closure values as
    // C function pointers + numeric + text/char + LIST RESULTS on the C
    // collections family): the HOF program over the FULL assembled std
    // set → value:144 via real cl. ----
    {
        use ace_a2x_c::CTargetProvider;
        let src_c = "use std.list.reduce
use std.list.filter
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let ys = filter(xs, (x int) => { return x > 1 })
    let s = reduce(ys, 0, (acc int, x int) => { return acc + x })
    return s * 10 + ys.len()
}";
        let mut ctx_c0 = PackageAssemblyContext::new(
            AssemblyBackend::A2C,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs_c0 = assemble_stdlib_aggregates_inputs(
            &[("main.at".to_string(), src_c.to_string())],
            "app::main",
            &mut ctx_c0,
        )
        .unwrap();
        let whole_c = compile_assembled_aggregates_inputs(
            &inputs_c0,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx_c0,
        )
        .unwrap_or_else(|ds| panic!("a2c whole compile: {ds:?}"));
        let store = scratch("cons-a2c-whole");
        let mut c_target = PackageA2xTarget::cold_aggregates(
            &store,
            "asm972c",
            "app",
            16 << 20,
            CTargetProvider::v1(),
            &whole_c,
        )
        .unwrap_or_else(|e| panic!("a2c cold (whole): {e}"));
        let build_dir = scratch("cons-a2c-whole-build");
        std::fs::create_dir_all(&build_dir).unwrap();
        for (path, content) in &c_target.current.files {
            std::fs::write(build_dir.join(path), content).unwrap();
        }
        std::fs::write(build_dir.join("manifest.json"), &c_target.current.manifest).unwrap();
        let exe = build_dir.join("asm972c.exe");
        ace_a2x_worker::crun::compile_c_fileset(&build_dir, &exe)
            .unwrap_or_else(|e| panic!("cl failed: {e}"));
        let run = Command::new(&exe).current_dir(&build_dir).output().unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:144"), "a2c whole stdout: {out}");
    }

    // ---- A2PY / real OS Python WHOLE-SET HOF leg (PLAN-972 T-03: the
    // Python producer with closure lambdas over the FULL assembled std
    // set → value:144 via PythonTargetProvider + pyrun). ----
    {
        use ace_a2x_python::PythonTargetProvider;
        let mut ctx_py = PackageAssemblyContext::new(
            AssemblyBackend::A2Py,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let src_py = "use std.list.reduce
use std.list.filter
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let ys = filter(xs, (x int) => { return x > 1 })
    let s = reduce(ys, 0, (acc int, x int) => { return acc + x })
    return s * 10 + ys.len()
}";
        let inputs_py = assemble_stdlib_aggregates_inputs(
            &[("main.at".to_string(), src_py.to_string())],
            "app::main",
            &mut ctx_py,
        )
        .unwrap();
        let whole_py = compile_assembled_aggregates_inputs(
            &inputs_py,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx_py,
        )
        .unwrap_or_else(|ds| panic!("a2py whole compile: {ds:?}"));
        let store = scratch("cons-a2py-whole");
        let mut py = PackageA2xTarget::cold_aggregates(
            &store,
            "asm972py",
            "app",
            16 << 20,
            PythonTargetProvider,
            &whole_py,
        )
        .unwrap_or_else(|e| panic!("a2py cold (whole): {e}"));
        let py_dir = scratch("cons-a2py-whole-build");
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
        assert!(out.ends_with("value:144"), "a2py whole stdout: {out}");
    }

    // ---- A2R / rustc WHOLE-SET leg (PLAN-972 T-03: capture-free closure
    // values + the text/char/numeric surface ride the collections family):
    // the HOF program — filter([1..5], >1)=[2,3,4,5] sum=14 len=4 →
    // 14*10+4 = 144 through real rustc over the FULL assembled std set. ----
    {
        let mut ctx_h = PackageAssemblyContext::new(
            AssemblyBackend::A2R,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let hof_inputs = assemble_stdlib_aggregates_inputs(
            &[(
                "main.at".to_string(),
                "use std.list.reduce
use std.list.filter
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    let ys = filter(xs, (x int) => { return x > 1 })
    let s = reduce(ys, 0, (acc int, x int) => { return acc + x })
    return s * 10 + ys.len()
}".to_string(),
            )],
            "app::main",
            &mut ctx_h,
        )
        .unwrap();
        let hof_product = compile_assembled_aggregates_inputs(
            &hof_inputs,
            "app::main",
            FrontBudget::WORK_OWNED_64MIB,
            &ctx_h,
        )
        .unwrap_or_else(|ds| panic!("hof compile: {ds:?}"));
        let store = scratch("cons-a2r-hof");
        let mut rust = PackageA2xTarget::cold_aggregates(
            &store,
            "asm972hof",
            "app",
            16 << 20,
            RustTargetProvider::v1(),
            &hof_product,
        )
        .unwrap_or_else(|e| panic!("a2r cold (hof): {e}"));
        let build_dir = scratch("cons-a2r-hof-build");
        std::fs::create_dir_all(&build_dir).unwrap();
        for (path, content) in &rust.current.files {
            std::fs::write(build_dir.join(path), content).unwrap();
        }
        let exe = build_dir.join("asm972hof.exe");
        let mut cmd =
            Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
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
        let run = Command::new(&exe).current_dir(&build_dir).output().unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
        assert!(out.ends_with("value:144"), "a2r hof stdout: {out}");
    }

    // ---- A2R / rustc POSITIVE leg on the closure-free subset (std.math
    // + app): square(7)*10+3 = 493 through real rustc. The 961 numeric
    // literal forms now ride the collections family (PLAN-972 T-03);
    // the closure/char whole-set gaps above stay frozen until the
    // closure/char domain extensions land. ----
    let math_src = ace_core::source::packages::stdlib_assembled_files()
        .into_iter()
        .find(|(n, _)| n == "math.at")
        .map(|(_, c)| c)
        .unwrap();
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::A2R,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let inputs = vec![
        ace_core::source::packages::PackageModuleInput {
            package: "std".into(),
            version: "1.0.0".into(),
            id: "math".into(),
            source: math_src,
            provider: None,
        },
        ace_core::source::packages::PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: "use std.math.square
use std.math.min
fn main() int { return square(7) * 10 + min(3i64, 5i64).to(i32) }".into(),
            provider: None,
        },
    ];
    let math_product = compile_assembled_aggregates_inputs(
        &inputs,
        "app::main",
        FrontBudget::WORK_OWNED_64MIB,
        &ctx,
    )
    .unwrap_or_else(|ds| panic!("math subset compile: {ds:?}"));
    let store = scratch("cons-a2r-math");
    let mut rust = PackageA2xTarget::cold_aggregates(
        &store,
        "asm972math",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &math_product,
    )
    .unwrap_or_else(|e| panic!("a2r cold (math): {e}"));
    let build_dir = scratch("cons-a2r-math-build");
    std::fs::create_dir_all(&build_dir).unwrap();
    for (path, content) in &rust.current.files {
        std::fs::write(build_dir.join(path), content).unwrap();
    }
    let exe = build_dir.join("asm972math.exe");
    let mut cmd = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
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
    let run = Command::new(&exe).current_dir(&build_dir).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
    assert!(out.ends_with("value:493"), "a2r math stdout: {out}");

    // ---- AC / native (mn) POSITIVE leg on the closure-free subset
    // (std.math + std.text + std.char + app): byte_len("A中😀")=8,
    // scalar_len=3 → exit 83 — the Unicode unit oracle on real machine
    // code through the PLAN-972 T-03 text/char/numeric emission arms
    // (closure values stay the named whole-set gap). ----
    let subset: Vec<ace_core::source::packages::PackageModuleInput> =
        ace_core::source::packages::stdlib_assembled_files()
            .into_iter()
            .filter(|(n, _)| matches!(n.as_str(), "math.at" | "text.at" | "char.at"))
            .map(|(n, c)| {
                let id = n.trim_end_matches(".at").to_string();
                ace_core::source::packages::PackageModuleInput {
                    package: "std".into(),
                    version: "1.0.0".into(),
                    id,
                    source: c,
                    provider: None,
                }
            })
            .chain(std::iter::once(
                ace_core::source::packages::PackageModuleInput {
                    package: "app".into(),
                    version: "1.0.0".into(),
                    id: "main".into(),
                    source: "use std.text.byte_len
use std.text.scalar_len
fn main() int {
    let a Str = \"A中😀\"
    return byte_len(a.clone()) * 10 + scalar_len(a.clone())
}".into(),
                    provider: None,
                },
            ))
            .collect();
    let mut ctx2 = PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let subset_product = compile_assembled_aggregates_inputs(
        &subset,
        "app::main",
        FrontBudget::WORK_OWNED_64MIB,
        &ctx2,
    )
    .unwrap_or_else(|ds| panic!("mn subset compile: {ds:?}"));
    let dir = scratch("cons-mn-subset");
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_collections_checked(
            &subset_product.compiled.checked,
            "d_main",
            &dir,
            &subset_product.bound.input_hash,
            972,
        )
        .unwrap_or_else(|e| panic!("mn subset build: {e:?}"));
    let code = Command::new(&built.exe_path)
        .current_dir(&dir)
        .status()
        .expect("run mn subset exe")
        .code()
        .unwrap_or(-1);
    assert_eq!(code, 83, "AC/native subset exit (Unicode units)");
}

/// S-05 parse family with the V-DETAIL-02 error paths independently
/// asserted: "42"→42, "-2147483648"→MIN (the boundary parses), "007"→7,
/// "2147483648"→None (range), "12a"→None (invalid digit), ""→None,
/// "-"→None (lone sign); is_uint negatives.
const CONV_MAIN: &str = r#"
use std.conv.to_int
use std.conv.is_uint
use std.conv.from_uint
fn pick(s Str) int {
    let r = to_int(s)
    is r {
        Some(v) -> { return v }
        None -> { return 999999 }
    }
}
fn main() int {
    var code = pick("42".clone()) * 10000000
    code = code + (pick("-2147483648".clone()).to(i64) + 2147483648i64).to(i32) * 1000000
    code = code + pick("007".clone()) * 100000
    if (pick("2147483648".clone()) == 999999) {
        code = code + 40000
    }
    if (pick("12a".clone()) == 999999) {
        code = code + 2000
    }
    if (pick("".clone()) == 999999) {
        code = code + 1000
    }
    if (pick("-".clone()) == 999999) {
        code = code + 400
    }
    if (is_uint("123".clone())) {
        code = code + 20
    }
    if (is_uint("12a".clone())) {
        code = code + 2
    }
    if (is_uint("".clone())) {
        code = code + 1
    }
    code = code + from_uint(0).clone().len() * 10
    code = code + from_uint(421).clone().len()
    if (from_uint(-5).clone().len() == 0) {
        code = code + 5
    }
    return code
}
"#;

#[test]
fn stdlib_asm_972_conv_parse_av() {
    let p = asm_lane_product("conv", CONV_MAIN);
    assert_eq!(run_av("conv", &p), Value::I32(420743438));
}

/// V-DETAIL-03 ownership negatives: the lane's move discipline surfaces as
/// NAMED rejections (never silent reuse). A moved owned argument read again
/// → `mir.use-after-move`; a move-in argument evaluated while an argument
/// expression still borrows it → `mir.mut-conflict`. Both reject before any
/// observable output.
#[test]
fn stdlib_asm_972_ownership_move_negatives() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "use-after-move",
            "fn eat(s Str) int { return s.len() }
fn main() int {
    let a Str = \"ab\"
    let n = eat(a)
    return n + a.len()
}",
            "use-after-move",
        ),
        (
            "mut-conflict",
            "fn both(a Str, b Str) int {
    return a.len() + b.len()
}
fn main() int {
    let p Str = \"abc\"
    return both(p.slice(0, p.len()).clone(), p)
}",
            "mut-conflict",
        ),
    ];
    for (name, src, want) in cases {
        let mut ctx = PackageAssemblyContext::new(
            AssemblyBackend::Av,
            ProviderChoice::Auto,
            TargetOs::Windows,
        );
        let inputs = assemble_stdlib_aggregates_inputs(
            &[("main.at".to_string(), src.to_string())],
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
        .unwrap_or_else(|ds| panic!("{name} compile: {ds:?}"));
        let dir = scratch(&format!("neg-{name}"));
        // the named rejection may surface at cold target construction (MIR
        // verify) or at run — both are pre-output rejections.
        let err = match PackageAvTarget::cold(&dir, &product) {
            Err(e) => e,
            Ok(av) => av.run_value().unwrap_err(),
        };
        assert!(
            err.to_lowercase().contains(want),
            "{name}: expected named rejection `{want}`, got: {err}"
        );
    }
}

/// `std` identity reservation on the assembled lane: a tampered `std.list`
/// source, and a provider layer riding `std.list`, both reject BEFORE any
/// compile/run with the named reservation code.
#[test]
fn stdlib_asm_972_std_identity_reserved() {
    use ace_core::source::packages::bind_packages_assembled_aggregates;
    use ace_source_query::modules::packages::assembled_aggregates_caps;
    let ctx = PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let caps = assembled_aggregates_caps();
    let files = ace_core::source::packages::stdlib_assembled_files();
    let frozen_list = files
        .iter()
        .find(|(n, _)| n == "list.at")
        .map(|(_, c)| c.clone())
        .expect("frozen assembled list module");
    let app = PackageModuleInput {
        package: "app".into(),
        version: STDLIB_VERSION.into(),
        id: "main".into(),
        source: "fn main() int { return 0 }".into(),
        provider: None,
    };

    // tampered std.list source
    let tampered = PackageModuleInput {
        package: STDLIB_PACKAGE.into(),
        version: STDLIB_VERSION.into(),
        id: "list".into(),
        source: format!("{frozen_list}\npub fn evil(x int) int {{ return x }}"),
        provider: None,
    };
    let e = bind_packages_assembled_aggregates(
        &[app.clone(), tampered],
        "app::main",
        &caps,
        FrontBudget::WORK_OWNED_64MIB,
        &ctx,
    )
    .unwrap_err();
    assert_eq!(e.diagnostic.code, "ace.stdlib-reserved", "{e:?}");

    // provider layer riding std.list (frozen public source)
    let layered = PackageModuleInput {
        package: STDLIB_PACKAGE.into(),
        version: STDLIB_VERSION.into(),
        id: "list".into(),
        source: frozen_list,
        provider: Some(PackageImplLayer {
            kind: ProviderKind::At,
            source: "fn never(x int) int { return x }".into(),
        }),
    };
    let e = bind_packages_assembled_aggregates(
        &[app, layered],
        "app::main",
        &caps,
        FrontBudget::WORK_OWNED_64MIB,
        &ctx,
    )
    .unwrap_err();
    assert_eq!(e.diagnostic.code, "ace.stdlib-reserved", "{e:?}");
}

/// Lane isolation: closure-typed params stay a named syntax rejection on the
/// plain aggregates lane (packages-aggregates without the ct/asm face; the
/// composed packages-aggregates-ct lane admits them — see the HOF test above).
#[test]
fn stdlib_asm_972_closure_params_still_rejected_on_plain_lanes() {
    use ace_core::source::packages::PACKAGES_CAPABILITY;
    const CLOSURE_SRC: &str = "fn apply(f fn(int) -> int, v int) int { return f(v) }\nfn main() int { return apply((n int) => { return n + 1 }, 41) }";
    let mut inputs = stdlib_aggregates_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: CLOSURE_SRC.into(),
        provider: None,
    });
    let ds = match compile_package_product_aggregates(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("closure-typed params unexpectedly admitted on the plain aggregates lane"),
        Err(ds) => ds,
    };
    assert_eq!(ds[0].diagnostic.code, "source.syntax", "{ds:?}");
}
