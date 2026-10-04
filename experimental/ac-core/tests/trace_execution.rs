//! PLAN-741 T-06 native call-order trace test: with the `hir.test.trace.v1`
//! capability explicitly provided AND the minimal host support library
//! linked, the 03-call-order document must execute natively with mark_b
//! evaluated before mark_a (stderr order "b" then "a") and pair() returning
//! 12 as the process exit code. Windows only.

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

const EXE: &str = env!("CARGO_BIN_EXE_auto-ac-prototype");
const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn build_support_lib() -> PathBuf {
    // The support library is a standalone no_std crate; build it into its
    // own target dir so no cargo file locks contend with the outer test run.
    let dir = PathBuf::from(std::env::temp_dir()).join("ac741-trace-support");
    let _ = std::fs::remove_dir_all(&dir);
    let out = Command::new("cargo")
        .args(["build", "--locked"])
        .current_dir(Path::new(REPO).join("experimental/ac-core/test-support"))
        .output()
        .expect("spawn cargo for test-support");
    assert!(
        out.status.success(),
        "test-support build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let lib =
        Path::new(REPO).join("experimental/ac-core/test-support/target/debug/ac_trace_support.lib");
    assert!(lib.is_file(), "support lib missing at {}", lib.display());
    let _ = dir;
    lib
}

#[test]
fn call_order_executes_natively_b_then_a_result_12() {
    let lib = build_support_lib();
    let dir = std::env::temp_dir().join("ac741-trace").join("run");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("trace.exe");
    let fixture = Path::new(REPO).join("docs/design/strategy/hir-examples/03-call-order.atom");

    // Without the capability the build must already be rejected.
    let no_cap = Command::new(EXE)
        .args([
            "build",
            &fixture.display().to_string(),
            "--entry",
            "d_caller",
            "--output",
            &exe.display().to_string(),
        ])
        .output()
        .unwrap();
    assert_ne!(
        no_cap.status.code(),
        Some(0),
        "capability gate did not reject"
    );

    // With the capability AND the support lib the build links and runs.
    let r = Command::new(EXE)
        .args([
            "build",
            &fixture.display().to_string(),
            "--entry",
            "d_caller",
            "--output",
            &exe.display().to_string(),
            "--capability",
            "hir.test.trace.v1",
            "--support-lib",
            &lib.display().to_string(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        r.status.code(),
        Some(0),
        "build failed:\n{}",
        String::from_utf8_lossy(&r.stderr)
    );
    assert!(exe.is_file());

    let run = Command::new(&exe).output().unwrap();
    assert_eq!(
        run.status.code(),
        Some(12),
        "pair(a=1, b=2) with a*10+b must exit 12"
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    let b_pos = stderr.find('b').expect("mark_b not recorded on stderr");
    let a_pos = stderr.find('a').expect("mark_a not recorded on stderr");
    assert!(
        b_pos < a_pos,
        "native call order must be b,a (stderr was `{}`)",
        stderr.replace('\n', " ").trim()
    );
}
