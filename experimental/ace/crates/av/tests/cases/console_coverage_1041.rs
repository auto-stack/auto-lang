//! PLAN-1041 V-COVERAGE (a2x legs): the REAL io provider-suffix package
//! (`stdlib/auto/platform/` — io.at + at/c/rs/py candidates) assembled
//! per explicit 1043 context and REALLY run on A2R (rustc exe), A2C (cl
//! exe) and A2PY (real OS Python). The SAME consumer source proves, per
//! leg: the selected provider's identity fn really executed (at=1 c=2
//! rs=3 py=4 — a config-name switch or native masquerade returns the
//! wrong identity), the classification contract (pipe ⇒ kind 2), and
//! byte-exact UTF-8 output with NUL content. AV legs run the same source
//! through the console lane (console_out_1041.rs); AV/C and AV/Rust legs
//! are pending PLAN-967's package-lane foreign admission (recorded in the
//! plan §5/§0 freeze — never faked here).
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_c::CTargetProvider;
use ace_a2x_python::PythonTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, ProviderKind, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_package_inputs, compile_assembled_inputs, discover_provider_sources,
};
use std::path::{Path, PathBuf};
use std::process::Command;

const PLATFORM_DIR: &str = "../../../../stdlib/auto/platform";

/// The consumer: provider identity + classification + byte-exact output
/// (UTF-8, NUL, surrogate-pair emoji). The return folds identity, kind and
/// confirmed bytes so a wrong runtime cannot fake one axis.
const MAIN: &str = "use io.out_kind
use io.out_stdout
use io.out_text
use io.out_bytes
fn main() int {
    let h = out_stdout()
    let k = out_kind(h)
    let a = out_text(h, \"V中\0😀\n\")
    let b = out_bytes(h, \"tail\")
    return k * 100000 + a + b
}";
/// The exact bytes the program itself writes (before the wrapper's
/// `value:N` line): 1+3+1+4+1 + 4 = 14.
const EXPECT_OUT: &str = "V中\0😀\ntail";
const EXPECT_BYTES_CONFIRMED: i32 = 14;

fn scratch(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/pkg1041cov/{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn fixture_dir(tag: &str) -> PathBuf {
    let dir = scratch(tag);
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PLATFORM_DIR);
    for name in ["io.at", "io.at.at", "io.c.at", "io.rs.at", "io.py.at"] {
        std::fs::copy(src.join(name), dir.join(name))
            .unwrap_or_else(|e| panic!("copy {name} from stdlib/auto/platform: {e}"));
    }
    std::fs::write(dir.join("main.at"), MAIN).unwrap();
    dir
}

fn assemble(
    dir: &Path,
    backend: AssemblyBackend,
    provider: ProviderChoice,
    target: TargetOs,
) -> ace_source_query::modules::packages::PackageProduct {
    let files = discover_provider_sources(dir).unwrap();
    let mut ctx = PackageAssemblyContext::new(backend, provider, target);
    let inputs = assemble_package_inputs(&files, "app::main", &mut ctx).unwrap();
    compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
        .unwrap_or_else(|ds| {
            for d in &ds {
                eprintln!(
                    "DIAG {} {} {:?} span={:?}",
                    d.module, d.diagnostic.code, d.diagnostic.message, d.diagnostic.span
                );
            }
            panic!(
                "compile {backend:?}/{provider:?}: {} {}",
                ds[0].module, ds[0].diagnostic.message
            )
        })
}

fn expected_value() -> String {
    format!("value:{}", 2 * 100_000 + EXPECT_BYTES_CONFIRMED)
}

fn run_rust_stdout(files: &[(String, String)], dir: &Path) -> String {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("p1041.exe");
    let mut cmd = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"]).arg(&exe);
    cmd.current_dir(dir);
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
    String::from_utf8_lossy(&run.stdout).to_string()
}

fn run_c_stdout(files: &[(String, String)], manifest: &str, dir: &Path) -> String {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), manifest).unwrap();
    let exe = dir.join("p1041_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(dir, &exe).unwrap_or_else(|e| panic!("cl failed: {e}"));
    let run = Command::new(&exe).current_dir(dir).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).to_string()
}

#[test]
fn vcoverage_1041_same_source_runs_on_a2r_c_and_a2py_with_provider_identity() {
    let dir = fixture_dir("src");

    // ---- A2R（Rs provider：真实 rustc 工程运行）----
    let a2r = assemble(
        &dir,
        AssemblyBackend::A2R,
        ProviderChoice::Kind(ProviderKind::Rs),
        TargetOs::Windows,
    );
    let rust = PackageA2xTarget::cold(
        &scratch("a2r-store"),
        "pkg1041cov",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &a2r,
    )
    .unwrap();
    let stdout = run_rust_stdout(&rust.current.files, &scratch("a2r-build"));
    assert!(stdout.starts_with(EXPECT_OUT), "A2R bytes: {stdout:?}");
    assert!(
        stdout.ends_with(&format!("{}\n", expected_value())),
        "A2R identity/value: {stdout:?}"
    );

    // ---- A2C（C provider：真实 cl 工程运行；AV/C 与其共享本受检 C 来源）----
    let a2c = assemble(
        &dir,
        AssemblyBackend::A2C,
        ProviderChoice::Kind(ProviderKind::C),
        TargetOs::Windows,
    );
    let c = PackageA2xTarget::cold(
        &scratch("a2c-store"),
        "pkg1041cov",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &a2c,
    )
    .unwrap();
    let stdout = run_c_stdout(&c.current.files, &c.current.manifest, &scratch("a2c-build"));
    assert!(stdout.starts_with(EXPECT_OUT), "A2C bytes: {stdout:?}");
    assert!(
        stdout.ends_with(&format!("{}\n", expected_value())),
        "A2C identity/value: {stdout:?}"
    );

    // ---- A2PY（Py provider：真实 OS Python 运行，PLAN-1042 producer）----
    let a2py = assemble(
        &dir,
        AssemblyBackend::A2Py,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Windows,
    );
    let py = PackageA2xTarget::cold(
        &scratch("a2py-store"),
        "pkg1041covpy",
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
    assert!(
        run.stdout.starts_with(EXPECT_OUT.as_bytes()),
        "A2PY bytes: {:?}",
        String::from_utf8_lossy(&run.stdout)
    );
    assert!(
        run.stdout
            .ends_with(format!("{}\n", expected_value()).as_bytes()),
        "A2PY identity/value: {:?}",
        String::from_utf8_lossy(&run.stdout)
    );

    // ---- 负控：Py provider 在 A2C 后端不可选（py 来源只服务 A2PY），
    // 具名拒绝先于任何运行（装配负控，非“改配置名”静默通过）。----
    let files = discover_provider_sources(&dir).unwrap();
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::A2C,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Windows,
    );
    if let Ok(inputs) = assemble_package_inputs(&files, "app::main", &mut ctx) {
        if compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx)
            .is_ok()
        {
            panic!("A2C + Py provider must reject (py source serves A2PY only)");
        }
    }
}
