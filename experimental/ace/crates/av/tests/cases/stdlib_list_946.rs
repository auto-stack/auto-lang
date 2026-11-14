//! PLAN-946 T-02 stdlib list conformance: the frozen public slice
//! (`std.list.sum/find_first_gt/pop/total_len` from the real
//! `stdlib/ace/std/list.at` source) through the packages producer chain
//! under the AGGREGATES profile (PLAN-962 admission — List-typed public
//! parameters are the T-01-frozen home for public List algorithms),
//! observed on the AV end with independent hand-computed oracles.
//! Four-end producers, fault paths and resource ledgers are T-03/T-05
//! (T-03 wires the a2x producer ends — Rust/C exes, native mn — and the
//! fault legs; T-05 assembles the six-path coverage evidence).
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_c::CTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_av::modules::packages::PackageAvTarget;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::{
    compile_package_product_aggregates, stdlib_aggregates_inputs, stdlib_inputs,
};

fn app(source: &str) -> Vec<PackageModuleInput> {
    // The aggregates lane hosts aggregates-family modules: std.list only
    // (stdlib_aggregates_inputs). std.math/std.text are TextRecord-domain
    // modules and stay with the plain packages lane (pinned boundary below).
    let mut inputs = stdlib_aggregates_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: source.into(),
        provider: None,
    });
    inputs
}
fn compile(
    inputs: &[PackageModuleInput],
) -> Result<
    ace_source_query::modules::packages::PackageProduct,
    Vec<ace_core::source::packages::PackageDiagnostic>,
> {
    compile_package_product_aggregates(
        inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
}
fn run_av(tag: &str, source: &str) -> i32 {
    let product = compile(&app(source)).unwrap_or_else(|ds| {
        panic!(
            "compile {tag}: {} {} {}",
            ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
        )
    });
    let dir = std::env::temp_dir().join(format!("pkg946-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let av = PackageAvTarget::cold(&dir, &product).unwrap_or_else(|e| panic!("av {tag}: {e}"));
    let out = av.run_value().unwrap_or_else(|e| panic!("run {tag}: {e}"));
    std::fs::remove_dir_all(&dir).ok();
    match out {
        ace_av_core::contract::Value::I32(v) => v,
        other => panic!("{tag}: non-i32 entry result: {other:?}"),
    }
}

/// The reduction + query slice: sum([1..5])=15 and find_first_gt(...,3)=4
/// on the SAME list (the parameter is a borrow-shaped read — two public
/// calls over one owner), 15*100+4 = 1504.
#[test]
fn stdlib_list_946_sum_and_find_av() {
    let src = r#"
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
    assert_eq!(run_av("sum-find", src), 1504);
}

/// find_first_gt with no match takes the None branch (-1 oracle).
#[test]
fn stdlib_list_946_find_none_av() {
    let src = r#"
use std.list.find_first_gt
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    let f = find_first_gt(xs, 9)
    is f {
        Some(v) -> { return v }
        None -> { return -1 }
    }
}
"#;
    assert_eq!(run_av("find-none", src), -1);
}

/// Option-return/is-match isolation (same module, no stdlib call): does
/// the lane carry Some payloads at all?
#[test]
fn stdlib_list_946_option_roundtrip_local_av() {
    let src = r#"
fn mk(x int) ?int {
    if (x > 0) {
        return Some(x)
    }
    return None
}
fn main() int {
    let f = mk(5)
    is f {
        Some(v) -> { return 100 + v }
        None -> { return -1 }
    }
}
"#;
    assert_eq!(run_av("opt-local", src), 105);
}

/// Early-return-inside-for-in isolation (same module): the find shape
/// without the cross-module call.
#[test]
fn stdlib_list_946_find_shape_local_av() {
    let src = r#"
fn find_first_gt(xs List<int>, threshold int) ?int {
    for x in xs {
        if (x > threshold) {
            return Some(x)
        }
    }
    return None
}
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(4)
    let f = find_first_gt(xs, 3)
    is f {
        Some(v) -> { return 100 + v }
        None -> { return -1 }
    }
}
"#;
    assert_eq!(run_av("find-local", src), 104);
}

/// Argument-convention isolation (packages lane, SAME module): does a
/// same-module List param borrow like the single-module face (22)?
#[test]
fn stdlib_list_946_arg_convention_same_module_av() {
    let src = r#"
fn len_of(xs List<int>) int {
    return xs.len()
}
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    let a = len_of(xs)
    return a * 10 + xs.len()
}
"#;
    // Divergence dump for the fix record: the atom text around the call
    // in both lanes (single-module vs packages).
    let single = ace_core::source::compile_aggregates_source(src, "probe.at", &[]);
    match &single {
        Ok(cs) => {
            for line in cs.atom_text.lines() {
                if line.contains("len_of") || line.contains("main") || line.contains("list") {
                    println!("single| {line}");
                }
            }
        }
        Err(ds) => println!(
            "single compile err: {:?}",
            ds.first().map(|d| d.message.clone())
        ),
    }
    let product = compile(&app(src)).expect("packages compile");
    for line in product.compiled.atom_text.lines() {
        if line.contains("len_of") || line.contains("main") || line.contains("list") {
            println!("pkgs  | {line}");
        }
    }
    assert_eq!(run_av("arg-same-module", src), 22);
}

/// Argument-convention isolation (packages lane, CROSS module): the same
/// shape through the imported std.list function — the old 021_list_push_pop
/// public obligation (push 10/20/30, pop -> 30) on the typed ACE form:
/// left at len 2 (30*100+2 = 3002).
#[test]
fn stdlib_list_946_pop_hit_av() {
    let src = r#"
use std.list.pop
fn main() int {
    var xs = List<int>.new()
    xs.push(10)
    xs.push(20)
    xs.push(30)
    let a = pop(xs)
    is a {
        Some(v) -> { return v * 100 + xs.len() }
        None -> { return 0 }
    }
}
"#;
    assert_eq!(run_av("pop-hit", src), 3002);
}

/// The ORIGINAL empty contract (old VM ListStorage::pop returned None):
/// pop on an empty list is Some/None-selectable, never a trap — -7 oracle.
#[test]
fn stdlib_list_946_pop_empty_none_av() {
    let src = r#"
use std.list.pop
fn main() int {
    var xs = List<int>.new()
    let a = pop(xs)
    is a {
        Some(v) -> { return v }
        None -> { return -7 }
    }
}
"#;
    assert_eq!(run_av("pop-empty", src), -7);
}

/// The owned-Str fold: total_len(["hello","world"]) = 10.
#[test]
fn stdlib_list_946_owned_str_total_len_av() {
    let src = r#"
use std.list.total_len
fn main() int {
    var xs = List<Str>.new()
    xs.push("hello")
    xs.push("world")
    return total_len(xs)
}
"#;
    assert_eq!(run_av("owned-str", src), 10);
}

/// Lane boundary (pinned): TextRecord-domain stdlib modules (std.text's
/// `str` family) are NOT part of the aggregates packages lane — the mixed
/// graph rejects with the named diagnostic, never silently.
#[test]
fn stdlib_list_946_text_modules_not_in_aggregates_lane() {
    let mut inputs = stdlib_inputs();
    inputs.extend(stdlib_aggregates_inputs());
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: "use std.list.sum\nfn main() int { return 0 }".into(),
        provider: None,
    });
    let err = match compile(&inputs) {
        Ok(_) => panic!("mixed text+aggregates graph must reject"),
        Err(ds) => ds,
    };
    assert!(
        err.iter().any(|d| {
            d.module.contains("text")
                && d.diagnostic.code == "source.unsupported"
                && d.diagnostic.message.contains("str")
        }),
        "expected the named str reject on std::text, got: {:?}",
        err.iter()
            .map(|d| (&d.module, d.diagnostic.code.as_str()))
            .collect::<Vec<_>>()
    );
}

/// std identity negative: a tampered `std.list` module is rejected before
/// binding (ace.stdlib-reserved — no user package can shadow the frozen
/// slice).
#[test]
fn stdlib_list_946_tampered_std_list_rejected() {
    let mut inputs = stdlib_aggregates_inputs();
    for input in inputs.iter_mut() {
        input.source = "pub fn sum(xs List<int>) int { return 1 }".into();
    }
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: "use std.list.sum\nfn main() int { return 0 }".into(),
        provider: None,
    });
    let err = match compile(&inputs) {
        Ok(_) => panic!("tampered std.list must reject"),
        Err(ds) => ds,
    };
    assert!(
        err.iter()
            .any(|d| d.diagnostic.code == "ace.stdlib-reserved"),
        "expected ace.stdlib-reserved, got: {:?}",
        err.iter()
            .map(|d| d.diagnostic.code.as_str())
            .collect::<Vec<_>>()
    );
}


/// T-03 ends programs: each runs on AV (value), Rust (rustc exe stdout),
/// C (cl exe stdout) and native/mn (exit code) with ONE independent
/// oracle per program.
const END_SUM_FIND: &str = r#"
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
const END_POP: &str = r#"
use std.list.pop
fn main() int {
    var xs = List<int>.new()
    xs.push(10)
    xs.push(20)
    xs.push(30)
    let a = pop(xs)
    is a {
        Some(v) -> { return v * 100 + xs.len() }
        None -> { return 0 }
    }
}
"#;
const END_OWNED_STR: &str = r#"
use std.list.total_len
fn main() int {
    var xs = List<Str>.new()
    xs.push("hello")
    xs.push("world")
    return total_len(xs)
}
"#;

/// T-03 Rust end: real rustc compile + exe run per program, the stdout
/// oracle `value:<n>` (the wrapper prints the entry's i32 result).
fn rust_end(tag: &str, src: &str, want: i32) {
    let product = compile(&app(src)).expect("aggregates product");
    let store = std::env::temp_dir().join(format!("pkg946-rs-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    let mut rust = PackageA2xTarget::cold_aggregates(
        &store,
        "pkg946rs",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .unwrap_or_else(|e| panic!("rust a2x target: {e}"));
    let dir = store.join("build");
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &rust.current.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("pkg.exe");
    let mut cmd = std::process::Command::new(
        std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()),
    );
    cmd.args(["--edition", "2021", "main.rs", "-o"]).arg(&exe).current_dir(&dir);
    let built = cmd.output().expect("rustc run");
    assert!(
        built.status.success(),
        "rustc failed: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = std::process::Command::new(&exe).current_dir(&dir).output().unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
    std::fs::remove_dir_all(&store).ok();
    assert_eq!(out, format!("value:{want}"), "rust exe stdout oracle ({tag})");
}

#[test]
fn stdlib_list_946_rust_end_sum_find() {
    rust_end("rs-sf", END_SUM_FIND, 1504);
}

#[test]
fn stdlib_list_946_rust_end_pop() {
    rust_end("rs-pop", END_POP, 3002);
}

#[test]
fn stdlib_list_946_rust_end_owned_str() {
    rust_end("rs-str", END_OWNED_STR, 10);
}

/// T-03 C end: real cl compile + exe run per program.
fn c_end(tag: &str, src: &str, want: i32) {
    let product = compile(&app(src)).expect("aggregates product");
    let store = std::env::temp_dir().join(format!("pkg946-c-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    let mut c = PackageA2xTarget::cold_aggregates(
        &store,
        "pkg946c",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &product,
    )
    .unwrap_or_else(|e| panic!("c a2x target: {e}"));
    let dir = store.join("build");
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &c.current.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &c.current.manifest).unwrap();
    let exe = dir.join("pkg_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(&dir, &exe)
        .unwrap_or_else(|e| panic!("cl failed: {e}"));
    let run = std::process::Command::new(&exe).current_dir(&dir).output().unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let out = String::from_utf8_lossy(&run.stdout).trim().to_string();
    std::fs::remove_dir_all(&store).ok();
    assert_eq!(out, format!("value:{want}"), "c exe stdout oracle ({tag})");
}

#[test]
fn stdlib_list_946_c_end_sum_find() {
    c_end("c-sf", END_SUM_FIND, 1504);
}

#[test]
fn stdlib_list_946_c_end_pop() {
    c_end("c-pop", END_POP, 3002);
}

#[test]
fn stdlib_list_946_c_end_owned_str() {
    c_end("c-str", END_OWNED_STR, 10);
}

/// T-03 native/mn end: the collections lane bridge — real machine-code
/// build + exe run per program; the exit code carries the oracle.
fn native_end(tag: &str, src: &str, want: i32) {
    let product = compile(&app(src)).expect("aggregates product");
    let dir = std::env::temp_dir().join(format!("pkg946-mn-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_collections_checked(
            &product.compiled.checked,
            "d_main",
            &dir,
            &product.bound.input_hash,
            946,
        )
        .unwrap_or_else(|e| panic!("mn build: {e:?}"));
    let run = std::process::Command::new(&built.exe_path)
        .current_dir(&dir)
        .output()
        .expect("run mn exe");
    // the exit code IS the entry's i32 result (the collections wrapper
    // contract) — no success() gate; a trap would print MN-COLL-TRAP and
    // miss the oracle below.
    let code = run.status.code().unwrap_or(-1);
    if code != want {
        println!(
            "mn exe stderr tail ({tag}): {:?}",
            String::from_utf8_lossy(&run.stderr)
                .lines()
                .rev()
                .take(6)
                .collect::<Vec<_>>()
        );
    }
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(code, want, "native mn exit-code oracle ({tag})");
}

#[test]
fn stdlib_list_946_native_end_sum_find() {
    native_end("mn-sf", END_SUM_FIND, 1504);
}

#[test]
fn stdlib_list_946_native_end_pop() {
    native_end("mn-pop", END_POP, 3002);
}

#[test]
fn stdlib_list_946_native_end_owned_str() {
    native_end("mn-str", END_OWNED_STR, 10);
}

/// Native bisect: the same computation WITHOUT any cross-function call
/// (everything inline in main) — isolates the container emission from the
/// cross-unit call convention.
#[test]
fn stdlib_list_946_native_no_call_inline() {
    let src = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    var total = 0
    for x in xs {
        total = total + x
    }
    var found = 0
    for x in xs {
        if (x > 3) {
            if (found == 0) {
                found = x
            }
        }
    }
    return total * 100 + found
}
"#;
    let product = compile(&app(src)).expect("aggregates product");
    let dir = std::env::temp_dir().join(format!("pkg946-mnnc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_collections_checked(
            &product.compiled.checked,
            "d_main",
            &dir,
            &product.bound.input_hash,
            946,
        )
        .unwrap_or_else(|e| panic!("mn build: {e:?}"));
    let run = std::process::Command::new(&built.exe_path)
        .current_dir(&dir)
        .output()
        .expect("run mn exe");
    println!(
        "mn-nc exe: code={:?} stderr-tail: {:?}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr).lines().rev().take(4).collect::<Vec<_>>()
    );
    let code = run.status.code().unwrap_or(-1);
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(code, 1504, "native inline oracle");
}

/// T-08 / V-08: a REAL cross-module public List call returning OWNED text
/// — the app's `strings` module takes the caller's list, moves the first
/// element out, and returns the owned Str; the caller RELEASES the list
/// (drop) and the surviving owned text still reads (5 for "hello").
#[test]
fn stdlib_list_946_cross_module_owned_text_survives_release_av() {
    let mut inputs = stdlib_aggregates_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "strings".into(),
        source: "pub fn take_first(xs List<Str>) Str {
    return xs.remove(0)
}".into(),
        provider: None,
    });
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: r#"
use app.strings.take_first
fn main() int {
    var xs = List<Str>.new()
    xs.push("hello")
    xs.push("world")
    let first Str = take_first(xs)
    xs.clear()
    return first.len()
}
"#
        .into(),
        provider: None,
    });
    let product = compile(&inputs).expect("cross-module owned-text product");
    let dir = std::env::temp_dir().join(format!("pkg946-xmod-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let av = PackageAvTarget::cold(&dir, &product).unwrap_or_else(|e| panic!("av: {e}"));
    let out = av.run_value().unwrap_or_else(|e| panic!("run: {e}"));
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(out, ace_av_core::contract::Value::I32(5));
}

/// T-08 / V-08 negative: a public function trying to RETURN a borrowed
/// element view rejects BEFORE any backend (`source.view-escape` — the
/// 962 frozen materialization rule).
#[test]
fn stdlib_list_946_cross_module_view_escape_rejected() {
    let mut inputs = stdlib_aggregates_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "strings".into(),
        source: "pub fn peek(xs List<Str>) Str {
    return xs.get(0)
}".into(),
        provider: None,
    });
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: "use app.strings.peek
fn main() int { return 0 }".into(),
        provider: None,
    });
    let err = match compile(&inputs) {
        Ok(_) => panic!("view-escaping public return must reject"),
        Err(ds) => ds,
    };
    assert!(
        err.iter().any(|d| d.diagnostic.code.contains("view-escape")),
        "expected the view-escape rejection, got: {:?}",
        err.iter()
            .map(|d| (d.module.as_str(), d.diagnostic.code.as_str()))
            .collect::<Vec<_>>()
    );
}

/// Grammar face (packages-aggregates): the admitted constructs parse —
/// List<int>/List<Str> params, for-in bodies, `?int` returns, the pop
/// body, if-inside-for with a PARENTHESIZED comparison — and the whole
/// frozen list.at parses. The unparenthesized `if a > t {` shape (var RHS
/// + same-line brace) stays a reject under BOTH packages faces: the
/// record-ctor `t { .. }` shape commits first — a pre-existing packages-
/// face ambiguity (PLAN-946 T-02 record; 945's own stdlib sources use
/// the parenthesized form).
#[test]
fn stdlib_list_946_grammar_face() {
    let admitted = [
        "pub fn f(xs List<int>) int {\n    return 0\n}",
        "pub fn f(xs List<Str>) int {\n    var t = 0\n    for x in xs {\n        t = t + 1\n    }\n    return t\n}",
        "pub fn find_first_gt(xs List<int>, threshold int) ?int {\n    for x in xs {\n        if (x > threshold) {\n            return Some(x)\n        }\n    }\n    return None\n}",
        "pub fn pop(xs List<int>) ?int {\n    if xs.len() == 0 {\n        return None\n    }\n    return Some(xs.remove(xs.len() - 1))\n}",
        "pub fn f(a int, t int) int {\n    if (a > t) {\n        return 1\n    }\n    return 0\n}",
    ];
    for src in admitted {
        ace_core::source::packages::parse_package_source_aggregates(
            src,
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .unwrap_or_else(|ds| panic!("must parse under packages-aggregates: {ds:?} — {src}"));
    }
    let ambiguous = "pub fn f(a int, t int) int {\n    if a > t {\n    }\n    return 0\n}";
    for (face, parse) in [
        (
            "plain",
            ace_core::source::packages::parse_package_source(
                ambiguous,
                &[PACKAGES_CAPABILITY.into()],
                FrontBudget::WORK_OWNED_64MIB,
            )
            .is_err(),
        ),
        (
            "aggregates",
            ace_core::source::packages::parse_package_source_aggregates(
                ambiguous,
                &[PACKAGES_CAPABILITY.into()],
                FrontBudget::WORK_OWNED_64MIB,
            )
            .is_err(),
        ),
    ] {
        assert!(
            parse,
            "unparenthesized var-RHS condition must reject under the {face} face"
        );
    }
    // and the plain face still rejects for-in (collections-family stmt).
    assert!(
        ace_core::source::packages::parse_package_source(
            "pub fn f(xs List<int>) int {\n    for x in xs {\n    }\n    return 0\n}",
            &[PACKAGES_CAPABILITY.into()],
            FrontBudget::WORK_OWNED_64MIB,
        )
        .is_err(),
        "plain packages face must keep rejecting for-in"
    );
}

/// The frozen base face stays intact: a List-typed public function in an
/// app module still rejects under the plain (text-record) packages path —
/// the aggregates lane is the only public home for List parameters
/// (T-01 freeze). App-only inputs: std.list itself does not parse under
/// the plain grammar (for-in is collections-family).
#[test]
fn stdlib_list_946_base_packages_path_still_rejects_list_params() {
    let inputs = vec![PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: "fn id(xs List<int>) int { return 0 }\nfn main() int { return 0 }".into(),
        provider: None,
    }];
    let err = match ace_source_query::modules::packages::compile_package_product(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("base packages path must keep rejecting List parameters"),
        Err(ds) => ds,
    };
    // The plain packages face never admitted parameterized types: the
    // binder rejects `List` as an unknown module type (source.module-
    // symbol) BEFORE resolve's list-value-context could fire — the
    // single-module collections face's own freeze is pinned separately
    // in p946_t01_probe.
    assert!(
        err.iter()
            .any(|d| d.diagnostic.code == "source.module-symbol"),
        "expected the binder's unknown-type reject, got: {:?}",
        err.iter()
            .map(|d| d.diagnostic.code.as_str())
            .collect::<Vec<_>>()
    );
}
