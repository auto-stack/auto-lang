//! PLAN-966 M2-A 编译期执行产品腿：语句级 `#if/#for/#is` 与宏调用
//! `#debug(x)` 经公共装配/编译前门在真实各端执行，独立手定 oracle：
//! AV（进程内值）、AC/native（mn exe 退出码）、A2R（rustc exe stdout）、
//! A2C（cl exe stdout）、A2PY（OS Python 真跑）。
//!
//! oracle（目标轴固定）：Windows = 10+2+100 = 112；Linux = 20+2+200 = 222。
//! 宏腿的口径：证明 `#debug(x)` 在四端可投影/绑定/发射/运行到完成（纯函数、
//! 返回值丢弃）；"宏被绑定为同名前门调用"由 core `plan966_tests` 的 AST/引用
//! 断言承载（AC 原生 v1 面不支持模块全局读写，故不用全局计数做观测）。
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_c::CTargetProvider;
use ace_a2x_python::PythonTargetProvider;
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

const MAIN: &str = "fn debug(v int) int { return v + 1000 }
fn main() int {
    var x int = 0
#if sys.current == sys.Windows { x = 10 } else { x = 20 }
#for i in 0..2 { x = x + 1 }
#is sys.current { Windows -> { x = x + 100 } Linux -> { x = x + 200 } }
#debug(x)
    return x
}";

fn scratch(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/pkg966/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn fixture_dir(tag: &str, source: &str) -> PathBuf {
    let dir = scratch(tag);
    std::fs::write(dir.join("main.at"), source).unwrap();
    dir
}

fn assemble(
    dir: &Path,
    backend: AssemblyBackend,
    provider: ProviderChoice,
    target: TargetOs,
) -> PackageProduct {
    let files = discover_provider_sources(dir).unwrap();
    let mut ctx = PackageAssemblyContext::new(backend, provider, target);
    let inputs = assemble_package_inputs(&files, "app::main", &mut ctx)
        .unwrap_or_else(|e| panic!("assemble {backend:?}/{provider:?}/{target:?}: {e}"));
    compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
        .unwrap_or_else(|ds| {
            panic!(
                "compile {backend:?}/{provider:?}/{target:?}: {} {} {}",
                ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
            )
        })
}

fn run_rust_stdout(files: &[(String, String)], dir: &Path) -> String {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("pkg966.exe");
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
    let exe = dir.join("pkg966_c.exe");
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
        .build_checked(&product.compiled.checked, "d_main", dir, None, None, 966)
        .unwrap_or_else(|e| panic!("mn build: {e:?}"));
    Command::new(&built.exe_path)
        .current_dir(dir)
        .status()
        .expect("run mn exe")
        .code()
        .unwrap_or(-1)
}

/// T-03/T-09 产品腿：语句级 CT 折叠 + 宏调用在 AV/AC/A2R/A2C/A2PY 真实执行，
/// 目标轴（Windows/Linux）在每端独立成立。宏腿为纯函数、返回值丢弃：本腿证明
/// "可投影/绑定/发射/运行到完成"，"宏被绑定为同名前门调用"由 core
/// `plan966_tests` 的 AST/引用断言承载（AC 原生 v1 面不支持模块全局，故不用全局
/// 计数做可观测副作用）。
#[test]
fn p966_ct_statements_and_macro_run_on_av_native_rust_c_py() {
    let dir = fixture_dir("src", MAIN);

    // ---- AV（进程内值）----
    let av_win = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let av = PackageAvTarget::cold(&scratch("av-win"), &av_win).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(112)));
    let av_lnx = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Linux,
    );
    let av = PackageAvTarget::cold(&scratch("av-lnx"), &av_lnx).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(222)));

    // ---- AC/native（原生机器码退出码）----
    let ac_win = assemble(
        &dir,
        AssemblyBackend::Ac,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    assert_eq!(native_exit(&ac_win, &scratch("ac-win")), 112);
    let ac_lnx = assemble(
        &dir,
        AssemblyBackend::Ac,
        ProviderChoice::Auto,
        TargetOs::Linux,
    );
    assert_eq!(native_exit(&ac_lnx, &scratch("ac-lnx")), 222);

    // ---- A2R（共享 Checked HIR → Rust 工程真实编译运行）----
    let a2r = assemble(
        &dir,
        AssemblyBackend::A2R,
        ProviderChoice::Kind(ProviderKind::Rs),
        TargetOs::Windows,
    );
    let rust = PackageA2xTarget::cold(
        &scratch("a2r-store"),
        "pkg966",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &a2r,
    )
    .unwrap();
    assert_eq!(
        run_rust_stdout(&rust.current.files, &scratch("a2r-build")),
        "value:112"
    );

    // ---- A2C（cl 出版本）----
    let a2c = assemble(
        &dir,
        AssemblyBackend::A2C,
        ProviderChoice::Kind(ProviderKind::C),
        TargetOs::Windows,
    );
    let c = PackageA2xTarget::cold(
        &scratch("a2c-store"),
        "pkg966",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &a2c,
    )
    .unwrap();
    assert_eq!(
        run_c_stdout(&c.current.files, &c.current.manifest, &scratch("a2c-build")),
        "value:112"
    );

    // ---- A2PY（新 producer + 真实 OS Python）----
    let a2py = assemble(
        &dir,
        AssemblyBackend::A2Py,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Windows,
    );
    let py = PackageA2xTarget::cold(
        &scratch("a2py-store"),
        "pkg966py",
        "app",
        16 << 20,
        PythonTargetProvider,
        &a2py,
    )
    .unwrap();
    let py_dir = scratch("a2py-build");
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
    assert_eq!(
        run.stdout,
        b"value:112\n",
        "stderr={:?}",
        String::from_utf8_lossy(&run.stderr)
    );
}

// V-08：CT 生成的声明（项级 #is 选中臂）改变实际四端结果——目标轴不同
// 声明，AV/native/A2R/A2C/A2PY 都按各自目标取到 10 / 20。
const DECL: &str = "#is sys.current {
    Windows -> { pub fn plat() int { return 10 } }
    Linux -> { pub fn plat() int { return 20 } }
}
fn main() int { return plat() }";

#[test]
fn p966_ct_generated_declarations_change_end_results() {
    let dir = fixture_dir("decl", DECL);

    let av_win = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    let av = PackageAvTarget::cold(&scratch("decl-av-win"), &av_win).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(10)));
    let av_lnx = assemble(
        &dir,
        AssemblyBackend::Av,
        ProviderChoice::Auto,
        TargetOs::Linux,
    );
    let av = PackageAvTarget::cold(&scratch("decl-av-lnx"), &av_lnx).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(20)));

    let ac_win = assemble(
        &dir,
        AssemblyBackend::Ac,
        ProviderChoice::Auto,
        TargetOs::Windows,
    );
    assert_eq!(native_exit(&ac_win, &scratch("decl-ac-win")), 10);
    let ac_lnx = assemble(
        &dir,
        AssemblyBackend::Ac,
        ProviderChoice::Auto,
        TargetOs::Linux,
    );
    assert_eq!(native_exit(&ac_lnx, &scratch("decl-ac-lnx")), 20);

    let a2r = assemble(
        &dir,
        AssemblyBackend::A2R,
        ProviderChoice::Kind(ProviderKind::Rs),
        TargetOs::Windows,
    );
    let rust = PackageA2xTarget::cold(
        &scratch("decl-a2r-store"),
        "pkg966d",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &a2r,
    )
    .unwrap();
    assert_eq!(
        run_rust_stdout(&rust.current.files, &scratch("decl-a2r-build")),
        "value:10"
    );

    let a2c = assemble(
        &dir,
        AssemblyBackend::A2C,
        ProviderChoice::Kind(ProviderKind::C),
        TargetOs::Windows,
    );
    let c = PackageA2xTarget::cold(
        &scratch("decl-a2c-store"),
        "pkg966d",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &a2c,
    )
    .unwrap();
    assert_eq!(
        run_c_stdout(
            &c.current.files,
            &c.current.manifest,
            &scratch("decl-a2c-build")
        ),
        "value:10"
    );

    let a2py = assemble(
        &dir,
        AssemblyBackend::A2Py,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Windows,
    );
    let py = PackageA2xTarget::cold(
        &scratch("decl-a2py-store"),
        "pkg966dp",
        "app",
        16 << 20,
        PythonTargetProvider,
        &a2py,
    )
    .unwrap();
    let py_dir = scratch("decl-a2py-build");
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
    assert_eq!(run.stdout, b"value:10\n");
}
