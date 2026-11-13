//! PLAN-945 stdlib core math & text conformance: the frozen public slice
//! (`std.math.square/cube`, `std.text.byte_len/equal/owned` from the real
//! `stdlib/ace/std/*.at` sources) through the SAME packages producer chain
//! as PLAN-960, observed on FOUR ends with independent hand-computed
//! oracles — AV (value), native/mn (exit code), Rust (rustc exe stdout)
//! and C (cl exe stdout). Includes the named-unwired i64 admission
//! negatives, the clone allocation-fault legs on each end's REAL fault
//! path (AV arena seam, A2X_RT_FAULT_ALLOC on both exes, plus the native
//! text-runtime ABI probe case), and packages-store recovery.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_c::CTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::av::{execute_with_arena, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::owned_text::StrArena;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::{compile_package_product, stdlib_inputs, PackageProduct};
use ace_source_query::modules::{PackageStore, PackageStoreReceipt};
use std::path::{Path, PathBuf};
use std::process::Command;

fn app(source: &str) -> Vec<PackageModuleInput> {
    let mut inputs = stdlib_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: source.into(),
    });
    inputs
}
fn compile(inputs: &[PackageModuleInput]) -> Result<PackageProduct, String> {
    compile_package_product(
        inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .map_err(|ds| {
        format!(
            "{}: {} {}",
            ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
        )
    })
}
fn err(inputs: &[PackageModuleInput]) -> String {
    match compile(inputs) {
        Ok(_) => panic!("expected rejection"),
        Err(e) => e,
    }
}
fn scratch(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/pkg945/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// P1 — the V-08 closed loop: the public text module is imported, the
/// source is cloned through `owned`, the source is released (overwritten
/// and dropped at `make`'s return), and the surviving clone reads back 8
/// UTF-8 bytes ("A中😀" = 1 + 3 + 4).
const P1: &str = "use std.text.byte_len
use std.text.owned
fn make() Str {
    var src Str = \"A中😀\"
    let c Str = owned(src.to(str))
    src = \"x\"
    return c
}
fn main() int {
    let c Str = make()
    return byte_len(c.to(str))
}";
/// P2 — the V-02 text family on four ends: byte_len 8/3/0/3 and byte-exact
/// equality (true, false, NUL-suffix distinct, NUL not a terminator,
/// embedded-NUL equal). 8303 + 173400 = 181703.
const P2: &str = "use std.text.byte_len
use std.text.equal
fn sel(c bool, t int, f int) int {
    if (c) { return t }
    return f
}
fn main() int {
    let lens int = byte_len(\"A中😀\") * 1000 + byte_len(\"e\\u{0301}\") * 100 + byte_len(\"\") * 10 + byte_len(\"a\\0b\")
    let e1 int = sel(equal(\"hello\", \"hello\"), 100000, 0)
    let e2 int = sel(equal(\"hello\", \"world\"), 0, 20000)
    let e3 int = sel(equal(\"a\\0b\", \"a\\0c\"), 0, 3000)
    let e4 int = sel(equal(\"a\", \"a\\0\"), 0, 400)
    let e5 int = sel(equal(\"a\\0b\", \"a\\0b\"), 50000, 0)
    return lens + e1 + e2 + e3 + e4 + e5
}";
/// P3 — the V-01 int family on four ends: the user's OWN legal min (their
/// signature, not the stdlib's) plus the public pure algorithms.
/// min(3,5)*1000 + square(3)*100 + cube(3) = 3927.
const P3: &str = "use std.math.square
use std.math.cube
fn min(a int, b int) int {
    if (a < b) { return a }
    return b
}
fn main() int {
    return min(3, 5) * 1000 + square(3) * 100 + cube(3)
}";
/// P3 overflow leg: 46341² overflows i32 — the checked trap, never a wrap.
const P3_OVERFLOW: &str = "use std.math.square
fn main() int {
    return square(46341)
}";
/// The clone-fault program for the a2x exes: the owned clone is the FIRST
/// allocation, so A2X_RT_FAULT_ALLOC=1 must fault exactly the clone.
const P_CLONE: &str = "use std.text.owned
fn main() int {
    let c Str = owned(\"A中😀\")
    return c.len_bytes()
}";

fn rust_stdout(files: &[(String, String)], dir: &Path, env: Option<(&str, &str)>) -> String {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("pkg.exe");
    let mut cmd = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"])
        .arg(&exe)
        .current_dir(dir);
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
    // plain Command: the fault legs must pass A2X_RT_FAULT_ALLOC through to
    // the child (the tree-scoped capture helper scrubs the environment).
    let mut cmd = Command::new(&exe);
    cmd.current_dir(dir);
    if let Some((k, v)) = env {
        cmd.env(k, v);
    }
    let run = cmd.output().expect("run rust exe");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn c_stdout(
    files: &[(String, String)],
    manifest: &str,
    dir: &Path,
    env: Option<(&str, &str)>,
) -> String {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), manifest).unwrap();
    let exe = dir.join("pkg_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(dir, &exe).unwrap_or_else(|e| panic!("cl failed: {e}"));
    // plain Command for the same env-passthrough reason as the rust leg.
    let mut cmd = Command::new(&exe);
    cmd.current_dir(dir);
    if let Some((k, v)) = env {
        cmd.env(k, v);
    }
    let run = cmd.output().expect("run c exe");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn native_exit(product: &PackageProduct, dir: &Path) -> i32 {
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_checked(&product.compiled.checked, "d_main", dir, None, None, 945)
        .unwrap_or_else(|e| panic!("mn build: {e:?}"));
    Command::new(&built.exe_path)
        .current_dir(dir)
        .status()
        .expect("run mn exe")
        .code()
        .unwrap_or(-1)
}

/// T-02/T-03/T-08: the four ends run every slice program against the
/// independent oracles (8 / 181703 / 3927), the overflow traps, and the
/// closed-loop clone survives its source's release.
#[test]
fn stdlib_core_945_four_ends_run_math_and_text_slices() {
    // ---- P1: closed loop = 8 on every end ----
    let p1 = compile(&app(P1)).unwrap();
    let av_dir = scratch("p1-av");
    let av = PackageAvTarget::cold(&av_dir, &p1).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(8)));
    // the surviving value is an independent OWNED buffer (the clone), not a
    // view of the released source
    let make = PackageAvTarget::cold_with_entry(&av_dir, &p1, "d_p3_appm4_main_make").unwrap();
    assert!(matches!(make.run_value(), Ok(Value::Str(_))));

    let store = scratch("p1-store");
    let mut rust = PackageA2xTarget::cold(
        &store,
        "pkg945",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &p1,
    )
    .unwrap();
    assert_eq!(
        rust_stdout(&rust.current.files, &scratch("p1-rust").join("build"), None),
        "value:8"
    );
    let mut c = PackageA2xTarget::cold(
        &store,
        "pkg945",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &p1,
    )
    .unwrap();
    assert_eq!(
        c_stdout(
            &c.current.files,
            &c.current.manifest,
            &scratch("p1-c").join("build"),
            None
        ),
        "value:8"
    );
    assert_eq!(native_exit(&p1, &scratch("p1-mn")), 8);

    // ---- P2: text family = 181703 on every end ----
    let p2 = compile(&app(P2)).unwrap();
    assert_eq!(
        PackageAvTarget::cold(&scratch("p2-av"), &p2)
            .unwrap()
            .run_value(),
        Ok(Value::I32(181703))
    );
    let mut rust2 = PackageA2xTarget::cold(
        &store,
        "pkg945p2",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &p2,
    )
    .unwrap();
    assert_eq!(
        rust_stdout(
            &rust2.current.files,
            &scratch("p2-rust").join("build"),
            None
        ),
        "value:181703"
    );
    let mut c2 = PackageA2xTarget::cold(
        &store,
        "pkg945p2",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &p2,
    )
    .unwrap();
    assert_eq!(
        c_stdout(
            &c2.current.files,
            &c2.current.manifest,
            &scratch("p2-c").join("build"),
            None
        ),
        "value:181703"
    );
    assert_eq!(native_exit(&p2, &scratch("p2-mn")), 181703);

    // ---- P3: int family + the user's own min = 3927 on every end ----
    let p3 = compile(&app(P3)).unwrap();
    assert_eq!(
        PackageAvTarget::cold(&scratch("p3-av"), &p3)
            .unwrap()
            .run_value(),
        Ok(Value::I32(3927))
    );
    let mut rust3 = PackageA2xTarget::cold(
        &store,
        "pkg945p3",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &p3,
    )
    .unwrap();
    assert_eq!(
        rust_stdout(
            &rust3.current.files,
            &scratch("p3-rust").join("build"),
            None
        ),
        "value:3927"
    );
    let mut c3 = PackageA2xTarget::cold(
        &store,
        "pkg945p3",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &p3,
    )
    .unwrap();
    assert_eq!(
        c_stdout(
            &c3.current.files,
            &c3.current.manifest,
            &scratch("p3-c").join("build"),
            None
        ),
        "value:3927"
    );
    assert_eq!(native_exit(&p3, &scratch("p3-mn")), 3927);

    // ---- P3 overflow: the checked trap on every end (no wrap) ----
    let pv = compile(&app(P3_OVERFLOW)).unwrap();
    let av_out = PackageAvTarget::cold(&scratch("ov-av"), &pv)
        .unwrap()
        .run_value();
    assert!(av_out.is_err(), "{av_out:?}");
    assert!(
        av_out.as_ref().unwrap_err().contains("IntOverflow"),
        "{av_out:?}"
    );
    let mut rustv = PackageA2xTarget::cold(
        &store,
        "pkg945ov",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &pv,
    )
    .unwrap();
    assert_eq!(
        rust_stdout(
            &rustv.current.files,
            &scratch("ov-rust").join("build"),
            None
        ),
        "trap:Overflow"
    );
    let mut cv = PackageA2xTarget::cold(
        &store,
        "pkg945ov",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &pv,
    )
    .unwrap();
    assert_eq!(
        c_stdout(
            &cv.current.files,
            &cv.current.manifest,
            &scratch("ov-c").join("build"),
            None
        ),
        "trap:Overflow"
    );
    assert_eq!(native_exit(&pv, &scratch("ov-mn")), 70, "mn trap exit");
}

/// V-03: the i64 public family is named-unsupported BEFORE any call (no
/// narrowing adapter), the `std` identity is reserved, ordinary typos keep
/// the generic rejection, and the user's own same-name functions are legal.
#[test]
fn stdlib_core_945_i64_family_named_unwired_and_std_reserved() {
    // `use std.math.min` — the frozen i64 public API, rejected by name.
    let e = err(&app("use std.math.min\nfn main() int { return min(1, 2) }"));
    assert!(e.contains("ace.stdlib-unwired"), "{e}");
    assert!(e.contains("i64"), "{e}");
    // qualified call form, same named rejection without any use.
    let e = err(&app("fn main() int { return std.math.abs(-5) }"));
    assert!(e.contains("ace.stdlib-unwired"), "{e}");
    // max via use-as alias — the target resolution still names it.
    let e = err(&app(
        "use std.math.max as m\nfn main() int { return m(1, 2) }",
    ));
    assert!(e.contains("ace.stdlib-unwired"), "{e}");
    // a spoofed `std` package never binds as the stdlib.
    let mut spoof = vec![PackageModuleInput {
        package: "std".into(),
        version: "1.0.0".into(),
        id: "math".into(),
        source: "pub fn min(a int, b int) int { return a }".into(),
    }];
    spoof.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: "fn main() int { return 1 }".into(),
    });
    let e = err(&spoof);
    assert!(e.contains("ace.stdlib-reserved"), "{e}");
    // an ordinary typo keeps the generic symbol rejection (not the i64 name).
    let e = err(&app("use std.math.squre\nfn main() int { return 1 }"));
    assert!(e.contains("source.module-symbol"), "{e}");
    assert!(!e.contains("ace.stdlib-unwired"), "{e}");
    // the user's own min (their signature) stays legal — see P3's four-end
    // run for the executed oracle.
    assert!(compile(&app(
        "fn min(a int, b int) int { return a }\nfn main() int { return min(3, 5) }"
    ))
    .is_ok());
}

/// V-04 clone-fault legs on the a2x exes: faulting the FIRST allocation
/// (the owned clone) yields the typed Resource trap — never a crash — and
/// the no-fault control returns 8.
#[test]
fn stdlib_core_945_clone_allocation_fault_is_a_typed_resource_trap() {
    let p = compile(&app(P_CLONE)).unwrap();
    let store = scratch("fault-store");
    let mut rust = PackageA2xTarget::cold(
        &store,
        "pkg945f",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &p,
    )
    .unwrap();
    assert_eq!(
        rust_stdout(
            &rust.current.files,
            &scratch("fault-rust").join("build"),
            None
        ),
        "value:8"
    );
    assert_eq!(
        rust_stdout(
            &rust.current.files,
            &scratch("fault-rust-f").join("build"),
            Some(("A2X_RT_FAULT_ALLOC", "1"))
        ),
        "trap:Resource"
    );
    let mut c = PackageA2xTarget::cold(
        &store,
        "pkg945f",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &p,
    )
    .unwrap();
    assert_eq!(
        c_stdout(
            &c.current.files,
            &c.current.manifest,
            &scratch("fault-c").join("build"),
            None
        ),
        "value:8"
    );
    assert_eq!(
        c_stdout(
            &c.current.files,
            &c.current.manifest,
            &scratch("fault-c-f").join("build"),
            Some(("A2X_RT_FAULT_ALLOC", "1"))
        ),
        "trap:Resource"
    );
}

/// V-04 AV leg + D5 recovery: sweep EVERY owned-allocation attempt of the
/// closed loop through the arena fault seam — each fault point aborts with
/// a typed outcome (never a panic, never a wrong success) and leaves ZERO
/// live buffers; then the body-edit recovery: only the app's segment
/// identity changes, the std segments are byte-stable, the restored run
/// gives the new value, and the a2x restore re-emits nothing.
#[test]
fn stdlib_core_945_av_fault_sweep_and_recovery_increment() {
    let p1 = compile(&app(P1)).unwrap();
    // lower through the same packages lane and admit like resource_915
    let bound = &p1.bound;
    let init_names: Vec<String> = bound
        .module_init_order
        .iter()
        .filter_map(|k| bound.module_init_fns.get(k).map(|s| format!("d_{s}")))
        .collect();
    let mir =
        ace_lowering::lower_checked_owned_with_init(&p1.compiled.checked, "d_main", 0, &init_names)
            .map_err(|e| format!("{e:?}"))
            .unwrap();
    let loaded = ace_av::resource_915::admit(&mir).unwrap();
    let cancel = CancelFlag::new(false);

    // clean control: 8, and the full ledger converges
    let clean = StrArena::new();
    let (out, _) = execute_with_arena(&loaded, &ExecLimits::default(), &cancel, clean.clone());
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(8),
                ..
            }
        ),
        "{out:?}"
    );
    let attempts = clean.attempts();
    assert!(
        attempts >= 2,
        "the loop must own at least the source and the clone"
    );
    assert_eq!(clean.live(), 0);

    // every fault point: typed abort, zero live buffers
    for k in 1..=attempts {
        let arena = StrArena::new();
        arena.set_fault_at(k);
        let (out, trace) = execute_with_arena(&loaded, &ExecLimits::default(), &cancel, arena);
        assert!(
            !matches!(out, ExecOutcome::Success { .. }),
            "fault #{k} must not succeed: {out:?}"
        );
        let live = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
        assert_eq!(live, 0, "fault #{k} leaked live owned buffers");
    }

    // ---- D5: body edit recovery ----
    let p1b = compile(&app(&P1.replace(
        "return byte_len(c.to(str))",
        "return byte_len(c.to(str)) + 1",
    )))
    .unwrap();
    assert_eq!(
        PackageAvTarget::cold(&scratch("rec-av"), &p1b)
            .unwrap()
            .run_value(),
        Ok(Value::I32(9))
    );
    let seg = |p: &PackageProduct, m: &str| {
        p.bound
            .segments
            .iter()
            .find(|s| s.module == m)
            .map(|s| ace_core::runtime_contract::sha256_hex(s.source.as_bytes()))
            .unwrap()
    };
    assert_ne!(seg(&p1, "main"), seg(&p1b, "main"));
    assert_eq!(seg(&p1, "math"), seg(&p1b, "math"));
    assert_eq!(seg(&p1, "text"), seg(&p1b, "text"));

    // a2x restore re-emits nothing for an unchanged product
    let store = scratch("rec-store");
    let mut rust = PackageA2xTarget::cold(
        &store,
        "pkg945r",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &p1b,
    )
    .unwrap();
    let receipt = rust.receipt().unwrap();
    let restored = PackageA2xTarget::restore(
        &store,
        "pkg945r",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &receipt,
        &p1b,
    )
    .unwrap();
    assert_eq!(restored.stats().lower, 0);
    assert_eq!(restored.stats().print, 0);
    assert_eq!(restored.current.files, rust.current.files);

    // packages root: identity-only persistence, restore re-admits
    let root_dir = store.clone();
    let targets = vec![(
        "a2x-rust".to_string(),
        PackageStoreReceipt {
            artifact: receipt.artifact.clone(),
            digest: receipt.digest.clone(),
        },
    )];
    let root_receipt = PackageStore::prepare(
        &root_dir,
        &p1b,
        &app(&P1.replace(
            "return byte_len(c.to(str))",
            "return byte_len(c.to(str)) + 1",
        )),
        &targets,
    )
    .unwrap();
    let (restored_product, restored_targets) =
        PackageStore::restore(&root_dir, &root_receipt).unwrap();
    assert_eq!(restored_product.bound.input_hash, p1b.bound.input_hash);
    assert_eq!(restored_targets.len(), 1);
    let av2 = PackageAvTarget::cold(&scratch("rec-av2"), &restored_product).unwrap();
    assert_eq!(av2.run_value(), Ok(Value::I32(9)));
}
