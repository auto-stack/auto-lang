//! PLAN-1043 stdlib context assembly conformance: the flat provider-source
//! suffix selection (`demo.at` public + `demo.at.at`/`demo.c.at`/`demo.rs.at`
//! implementation candidates) and the minimal `#in sys is current` platform
//! arm, assembled with an EXPLICIT backend/provider/target context and
//! executed on the real ends with independent hand-set oracles:
//!   AV (in-process value), AC/native (mn exe exit), A2R (rustc exe stdout),
//!   A2C (cl exe stdout). The A2PY execution leg stays PENDING PLAN-1042
//! (no ACE Python producer exists yet; only selection/binding is asserted
//! there — never a fake run). Pure public bodies (`common`) are shared
//! single-source across every provider (+101 on all legs).
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_c::CTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::Value;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, ProviderKind, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_package_inputs, compile_assembled_inputs, discover_provider_sources, PackageProduct,
};
use std::path::{Path, PathBuf};
use std::process::Command;

const MAIN: &str = "use demo.provider_id\nuse demo.common\n#in sys is current {\n    Windows -> use winio.plat\n    Linux -> use lnxio.plat\n}\nfn main() int { return provider_id() + common(1) + plat() }";
const DEMO_PUBLIC: &str =
    "pub spec fn provider_id() int\npub fn common(x int) int { return x + 100 }";
const DEMO_AT: &str = "fn provider_id() int { return 1 }";
const DEMO_C: &str = "fn provider_id() int { return 2 }";
const DEMO_RS: &str = "fn provider_id() int { return 3 }";
const DEMO_PY: &str = "fn provider_id() int { return 4 }";
const WINIO: &str = "pub fn plat() int { return 10 }";
const LNXIO: &str = "pub fn plat() int { return 20 }";

fn scratch(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/pkg1043/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn fixture_dir(tag: &str) -> PathBuf {
    let dir = scratch(tag);
    let files: &[(&str, &str)] = &[
        ("main.at", MAIN),
        ("demo.at", DEMO_PUBLIC),
        ("demo.at.at", DEMO_AT),
        ("demo.c.at", DEMO_C),
        ("demo.rs.at", DEMO_RS),
        ("demo.py.at", DEMO_PY),
        ("winio.at", WINIO),
        ("lnxio.at", LNXIO),
    ];
    for (n, c) in files {
        std::fs::write(dir.join(n), c).unwrap();
    }
    dir
}

fn assemble(
    dir: &Path,
    backend: AssemblyBackend,
    provider: ProviderChoice,
    target: TargetOs,
) -> (
    Vec<ace_core::source::packages::PackageModuleInput>,
    PackageAssemblyContext,
    PackageProduct,
) {
    let files = discover_provider_sources(dir).unwrap();
    let mut ctx = PackageAssemblyContext::new(backend, provider, target);
    let inputs = assemble_package_inputs(&files, "app::main", &mut ctx)
        .unwrap_or_else(|e| panic!("assemble {backend:?}/{provider:?}/{target:?}: {e}"));
    let product =
        compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
            .unwrap_or_else(|ds| {
                panic!(
                    "compile {backend:?}/{provider:?}/{target:?}: {} {} {}",
                    ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
                )
            });
    (inputs, ctx, product)
}

fn run_rust_stdout(files: &[(String, String)], dir: &Path) -> String {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("pkg1043.exe");
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
    let run = Command::new(&exe).current_dir(dir).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

fn run_c_stdout(files: &[(String, String)], manifest: &str, dir: &Path) -> String {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), manifest).unwrap();
    let exe = dir.join("pkg1043_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(dir, &exe).unwrap_or_else(|e| panic!("cl failed: {e}"));
    let run = Command::new(&exe).current_dir(dir).output().unwrap();
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
        .build_checked(&product.compiled.checked, "d_main", dir, None, None, 1043)
        .unwrap_or_else(|e| panic!("mn build: {e:?}"));
    Command::new(&built.exe_path)
        .current_dir(dir)
        .status()
        .expect("run mn exe")
        .code()
        .unwrap_or(-1)
}

/// ASM-05: the SAME source fixture, assembled per explicit context, really
/// runs on AV / native(AC) / rustc(A2R) / cl(A2C) with hand-set oracles.
/// provider_id: at=1 c=2 rs=3 (py=4 pending 1042); common adds +101 on
/// every provider (pure public single source); plat: winio=10 lnxio=20 by
/// the `#in sys is current` target axis.
#[test]
fn stdlib_assembly_1043_selection_runs_on_av_native_rust_and_c() {
    let dir = fixture_dir("src");

    // ---- AV 三配置（同一 AV，三种实现来源；纯选择 fixture，不冒真实
    // C/Rust 外库 dispatch——实调归 967/库 owner）----
    let (_, _, av_at_win) = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let av = PackageAvTarget::cold(&scratch("av-at-win"), &av_at_win).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(112)));

    let (_, _, av_at_lnx) = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Linux,
    );
    let av = PackageAvTarget::cold(&scratch("av-at-lnx"), &av_at_lnx).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(122)));

    let (_, _, av_c) = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Kind(ProviderKind::C),
        TargetOs::Windows,
    );
    let av = PackageAvTarget::cold(&scratch("av-c"), &av_c).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(113)));

    let (_, _, av_rs) = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Kind(ProviderKind::Rs),
        TargetOs::Windows,
    );
    let av = PackageAvTarget::cold(&scratch("av-rs"), &av_rs).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(114)));

    // ---- AC（原生机器码；纯 Auto at 来源）----
    let (_, _, ac_win) = assemble(
        &dir,
        AssemblyBackend::Ac,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    assert_eq!(native_exit(&ac_win, &scratch("ac-win")), 112);
    let (_, _, ac_lnx) = assemble(
        &dir,
        AssemblyBackend::Ac,
        ProviderChoice::Auto,
        TargetOs::Linux,
    );
    assert_eq!(native_exit(&ac_lnx, &scratch("ac-lnx")), 122);

    // ---- A2R（共享 Checked HIR/A2X→Rust 工程真实编译运行）----
    let store = scratch("a2r-store");
    let (_, _, a2r) = assemble(
        &dir,
        AssemblyBackend::A2R,
        ProviderChoice::Kind(ProviderKind::Rs),
        TargetOs::Windows,
    );
    let mut rust = PackageA2xTarget::cold(
        &store,
        "pkg1043",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &a2r,
    )
    .unwrap();
    assert_eq!(
        run_rust_stdout(&rust.current.files, &scratch("a2r-build")),
        "value:114"
    );

    // ---- 原A2C（C provider 与 AV/C 共用同一受检 Auto C 适配源）----
    let store = scratch("a2c-store");
    let (_, _, a2c_win) = assemble(
        &dir,
        AssemblyBackend::A2C,
        ProviderChoice::Kind(ProviderKind::C),
        TargetOs::Windows,
    );
    let mut c = PackageA2xTarget::cold(
        &store,
        "pkg1043",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &a2c_win,
    )
    .unwrap();
    assert_eq!(
        run_c_stdout(&c.current.files, &c.current.manifest, &scratch("a2c-build")),
        "value:113"
    );
    let (_, _, a2c_lnx) = assemble(
        &dir,
        AssemblyBackend::A2C,
        ProviderChoice::Kind(ProviderKind::C),
        TargetOs::Linux,
    );
    let mut c = PackageA2xTarget::cold(
        &scratch("a2c-store2"),
        "pkg1043",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &a2c_lnx,
    )
    .unwrap();
    assert_eq!(
        run_c_stdout(
            &c.current.files,
            &c.current.manifest,
            &scratch("a2c-build2")
        ),
        "value:123"
    );

    // ---- A2PY：选择/绑定真实成立，执行腿 PENDING PLAN-1042（诚实缺口：
    // 本仓无 ACE Python producer；绝不以旧 emitter 或标签冒跑）----
    let (_, _, a2py) = assemble(
        &dir,
        AssemblyBackend::A2Py,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Windows,
    );
    assert!(
        a2py.compiled
            .checked
            .bundle()
            .modules
            .iter()
            .any(|m| m.defs.iter().any(|d| d.id_text == "d_main")),
        "a2py binding reaches checked HIR (execution leg pending PLAN-1042)"
    );
}

/// ASM-05/ASM-03 cross-check: Windows-host assembling a Linux target really
/// selects the Linux arm (V-09 target axis) on a real AV run — the unselected
/// `winio` module is never bound, so the fixture directory with `winio.at`
/// REMOVED still compiles and runs for the Linux target.
#[test]
fn stdlib_assembly_1043_linux_target_ignores_missing_windows_sdk() {
    let dir = fixture_dir("linux-only");
    std::fs::remove_file(dir.join("winio.at")).unwrap();
    let (_, _, av) = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Linux,
    );
    let av = PackageAvTarget::cold(&scratch("av-lnx-only"), &av).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(122)));
    // 同一目录按 Windows 目标装配 → 选中 use 的模块缺席 → 具名拒。
    let files = discover_provider_sources(&dir).unwrap();
    let mut ctx =
        PackageAssemblyContext::new(AssemblyBackend::Av, ProviderChoice::Auto, TargetOs::Windows);
    let inputs = assemble_package_inputs(&files, "app::main", &mut ctx).unwrap();
    let e =
        match compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx) {
            Ok(_) => panic!("expected the selected Windows arm to reject its missing module"),
            Err(ds) => ds,
        };
    assert_eq!(
        e[0].diagnostic.code, "source.module-missing",
        "{:?} {}",
        e[0].module, e[0].diagnostic.message
    );
}

/// ASM-06 boundary: assembled products restore through the /2 packages store
/// with the provider/context identity intact (see also
/// `packages_store::tests::plan1043_assembly`).
#[test]
fn stdlib_assembly_1043_store_roundtrip_keeps_assembly_identity() {
    use ace_source_query::modules::PackageStore;
    let dir = fixture_dir("store");
    let (inputs, ctx, product) = assemble(
        &dir,
        AssemblyBackend::A2C,
        ProviderChoice::Kind(ProviderKind::C),
        TargetOs::Linux,
    );
    let root = scratch("store-root");
    let receipt =
        PackageStore::prepare_with_context(&root, &product, &inputs, &[], Some(&ctx)).unwrap();
    let (restored, _) = PackageStore::restore(&root, &receipt).unwrap();
    assert_eq!(restored.bound.input_hash, product.bound.input_hash);
    let mut c = PackageA2xTarget::cold(
        &scratch("store-a2c"),
        "pkg1043",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &restored,
    )
    .unwrap();
    assert_eq!(
        run_c_stdout(
            &c.current.files,
            &c.current.manifest,
            &scratch("store-build")
        ),
        "value:123"
    );
}
