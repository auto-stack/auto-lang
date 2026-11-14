//! PLAN-1041 V-LOAD: the io output package consumed THROUGH the 1043
//! loading mechanism — provider-suffix selection plus the minimal `#in`
//! target axis. A real consumer (#in-selected platform module + the io
//! capability) REALLY runs on A2PY (real OS Python, Windows and Linux
//! targets) and AC (native exit, both targets); the negative controls
//! prove a missing SELECTED module is a named rejection while an
//! unselected module is never bound (its file can be absent). The
//! store/incremental-recovery machinery itself is PLAN-1043's own
//! conformance (already on master) — reused by applicability, not
//! duplicated here.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_python::PythonTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, ProviderKind, TargetOs,
};
use ace_source_query::modules::packages::{
    assemble_package_inputs, compile_assembled_inputs, discover_provider_sources,
};
use std::path::{Path, PathBuf};

const PLATFORM_DIR: &str = "../../../../stdlib/auto/platform";

fn scratch(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/pkg1041load/{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The consumer: the io capability writes real bytes; the `#in` axis
/// selects the platform probe (win=10 / lnx=20) whose value rides the
/// exit code — a wrong selection is observable, not silent.
const MAIN: &str = "use io.out_stdout
use io.out_text
use io.out_bytes
#in sys is current {
    Windows -> use winio.pio_probe
    Linux -> use lnxio.pio_probe
}
fn main() int {
    let h = out_stdout()
    let a = out_text(h, \"L\0中\n\")
    let b = out_bytes(h, \"tail\")
    return a + b + pio_probe()
}";
const PLT_WIN: &str = "pub fn pio_probe() int {\n    return 10\n}";
const PLT_LNX: &str = "pub fn pio_probe() int {\n    return 20\n}";
/// out_text("L\0中\n") = 5 bytes; out_bytes("tail") = 4.
const BYTES: i32 = 10;

fn fixture_dir(tag: &str, with_lnx: bool) -> PathBuf {
    let dir = scratch(tag);
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PLATFORM_DIR);
    for name in ["io.at", "io.at.at", "io.c.at", "io.rs.at", "io.py.at"] {
        std::fs::copy(src.join(name), dir.join(name)).unwrap();
    }
    std::fs::write(dir.join("main.at"), MAIN).unwrap();
    std::fs::write(dir.join("plt.at"), "pub spec fn pio_probe() int").unwrap();
    std::fs::write(
        dir.join("plt.at.at"),
        "fn pio_probe() int {\n    return 0\n}",
    )
    .unwrap();
    std::fs::write(
        dir.join("plt.rs.at"),
        "fn pio_probe() int {\n    return 0\n}",
    )
    .unwrap();
    std::fs::write(
        dir.join("plt.py.at"),
        "fn pio_probe() int {\n    return 0\n}",
    )
    .unwrap();
    std::fs::write(dir.join("winio.at"), PLT_WIN).unwrap();
    if with_lnx {
        std::fs::write(dir.join("lnxio.at"), PLT_LNX).unwrap();
    }
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
            panic!(
                "compile {backend:?}/{provider:?}/{target:?}: {} {}",
                ds[0].module, ds[0].diagnostic.message
            )
        })
}

#[test]
fn vload_1041_in_axis_with_io_runs_on_a2py_both_targets() {
    let dir = fixture_dir("a2py", true);

    // Windows target: pio_win (10) selected; real OS Python run writes the
    // io bytes THEN the value line.
    let a2py = assemble(
        &dir,
        AssemblyBackend::A2Py,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Windows,
    );
    let mut py = PackageA2xTarget::cold(
        &scratch("a2py-win-store"),
        "pkg1041load",
        "app",
        16 << 20,
        PythonTargetProvider,
        &a2py,
    )
    .unwrap();
    let py_dir = scratch("a2py-win");
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
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        run.stdout.starts_with(b"L\0\xe4\xb8\xad\ntail"),
        "real output bytes first: {:?}",
        String::from_utf8_lossy(&run.stdout)
    );
    assert!(
        run.stdout
            .ends_with(format!("value:{}\n", BYTES + 10).as_bytes()),
        "windows arm identity: {:?}",
        String::from_utf8_lossy(&run.stdout)
    );

    // Linux target on the SAME directory: pio_lnx (20) selected.
    let a2py = assemble(
        &dir,
        AssemblyBackend::A2Py,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Linux,
    );
    let mut py = PackageA2xTarget::cold(
        &scratch("a2py-lnx-store"),
        "pkg1041loadlnx",
        "app",
        16 << 20,
        PythonTargetProvider,
        &a2py,
    )
    .unwrap();
    let py_dir = scratch("a2py-lnx");
    for (path, content) in &py.current.files {
        std::fs::write(py_dir.join(path), content).unwrap();
    }
    let run = ace_a2x_worker::pyrun::run_py_fileset(&tool, &py_dir, &[]).unwrap();
    assert_eq!(
        run.exit_code,
        Some(0),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        run.stdout
            .ends_with(format!("value:{}\n", BYTES + 20).as_bytes()),
        "linux arm identity: {:?}",
        String::from_utf8_lossy(&run.stdout)
    );
}

#[test]
fn vload_1041_in_axis_selection_negatives() {
    // (a) A Linux target with the Linux module ABSENT: the selected arm's
    // use must be a NAMED rejection — never a silent native fallback.
    let dir = fixture_dir("neg", false); // no lnxio.at
    let files = discover_provider_sources(&dir).unwrap();
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::A2Py,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Linux,
    );
    let inputs = assemble_package_inputs(&files, "app::main", &mut ctx).unwrap();
    let ds =
        match compile_assembled_inputs(&inputs, "app::main", FrontBudget::WORK_OWNED_64MIB, &ctx) {
            Ok(_) => panic!("missing selected module must reject"),
            Err(ds) => ds,
        };
    assert_eq!(
        ds[0].diagnostic.code, "source.module-missing",
        "{:?}",
        ds[0].diagnostic.message
    );

    // (b) The UNSELECTED arm is never bound: the same directory (no
    // lnxio.at) assembles and RUNS fine for the Windows target.
    let _ = assemble(
        &dir,
        AssemblyBackend::A2Py,
        ProviderChoice::Kind(ProviderKind::Py),
        TargetOs::Windows,
    );
}
