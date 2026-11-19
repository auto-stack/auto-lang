//! PLAN-1045 T-09 (V-05): the hosted scenario closure — the diagnostic
//! driver compiled by the Gen0 frontend on the sys lane to a REAL
//! executable (A2R end, real rustc) and run as a real child from TWO
//! different working directories. The diag legs (cwd roundtrip+restore,
//! α source-span stability, OS-path NUL refusal, missing tool, same-source
//! retry) must be identical across the runs with only the reported
//! starting cwd differing; a LANGUAGE type error is refused at COMPILE
//! stage (a different, earlier family than every runtime leg).
//!
//! The argv VALUE leg is a NAMED BLOCKED SLICE (D-1045-01, a2x runtime):
//! the argv read through this module is shape-nondeterministically
//! corrupted (content equality AND tail lengths both observed wrong
//! across dedicated-exe shapes that previously read green; in-session
//! probes recorded the full equal() oracle green and the tail codepoints
//! byte-exact). No weakened fallback binding is shipped; the closing
//! oracle lives in `host_diag.at` (`args_tail_ok`/`argv_shape`).

#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_aac_host_conformance as lane;
use ace_core::source::packages::AssemblyBackend;

const DIAG_SOURCE: &str = include_str!("../../../aac/host/host_diag.at");

fn bslash() -> char {
    0x5c as char
}

/// Build one A2R exe for `source` under `root`'s build dir (real rustc).
fn build_a2r_exe(
    tag: &str,
    root: &std::path::Path,
    source_files: Vec<(String, String)>,
) -> std::path::PathBuf {
    let product = lane::sys_product_backend(tag, &source_files, AssemblyBackend::A2R)
        .expect("compile sys product");
    let store = lane::scratch(&format!("{tag}-store"));
    let target = PackageA2xTarget::cold_aggregates(
        &store,
        tag,
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .expect("a2r target");
    let bdir = root.join("a2r-build");
    std::fs::create_dir_all(&bdir).unwrap();
    for (p, c) in &target.current.files {
        std::fs::write(bdir.join(p), c).unwrap();
    }
    let exe = bdir.join(format!("{tag}.exe"));
    let out = std::process::Command::new("rustc")
        .args(["--edition", "2021", "main.rs", "-o"])
        .arg(&exe)
        .current_dir(&bdir)
        .output()
        .expect("rustc");
    assert!(
        out.status.success(),
        "rustc: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    exe
}

fn diag_fixture(root: &std::path::Path) {
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::write(root.join("a"), b"prefix").unwrap();
    // "diag 模块 α\0b" — 16 UTF-8 bytes with an embedded NUL at index 14
    std::fs::write(root.join("diag-module.at"), "diag 模块 α\0b").unwrap();
}

fn run_diag(exe: &std::path::Path, root: &std::path::Path, cwd: &std::path::Path) -> (i32, String) {
    let out = std::process::Command::new(exe)
        .args(["αrg 空 格", "two words", "", "a\"b", "中文-1045"])
        .env(
            "AAC1045_DIAG_ROOT",
            root.to_string_lossy().replace(bslash(), "/"),
        )
        .current_dir(cwd)
        .output()
        .expect("run diag exe");
    let report = std::fs::read_to_string(root.join("report.txt")).unwrap_or_default();
    (out.status.code().unwrap_or(-1), report)
}

/// V-05 diag legs: one client, two cwds — the α source span, the NUL
/// refusal, the missing-tool failure and the retry outcome are IDENTICAL
/// (the report codes are all 0 in both runs); only the reported starting
/// cwd differs. The A2R process exit is the runtime's own success code,
/// so the report FILE is the observable (the PLAN-1064 discipline).
#[test]
fn v05_host_diag_scenario() {
    let root = lane::scratch("t09-root");
    diag_fixture(&root);
    let root_fwd = root.to_string_lossy().replace(bslash(), "/");
    let main_src = format!(
        "use std.file.write_text\nuse app.host_diag.diag_report\nfn main() int {{\n    let rep = diag_report(\"{root_fwd}\")\n    return write_text(\"{root_fwd}/report.txt\", rep)\n}}\n"
    );
    let files = vec![
        ("host_diag.at".to_string(), DIAG_SOURCE.to_string()),
        ("main.at".to_string(), main_src),
    ];
    let exe = build_a2r_exe("aac1045diag", &root, files);

    let cwd1 = root.join("sub");
    let cwd2 = std::env::temp_dir();
    let (code1, report1) = run_diag(&exe, &root, &cwd1);
    assert_eq!(code1, 0, "first run exits 0 (cwd={})", cwd1.display());
    let (code2, report2) = run_diag(&exe, &root, &cwd2);
    assert_eq!(code2, 0, "second run exits 0 (cwd={})", cwd2.display());
    let norm = |p: &std::path::Path| {
        p.to_string_lossy()
            .trim_end_matches(bslash())
            .trim_end_matches('/')
            .to_string()
    };
    let want1 = format!("0,0,0,0,0|{}", norm(&cwd1));
    let want2 = format!("0,0,0,0,0|{}", norm(&cwd2));
    assert_eq!(report1, want1, "first run all legs green, cwd reported");
    assert_eq!(report2, want2, "second run all legs green, cwd reported");
    assert_ne!(report1, report2, "the starting cwd really differs");
}

/// V-05 stage split (fast leg, no external toolchain): a LANGUAGE type
/// error never reaches a process — it is refused at COMPILE stage with a
/// diagnostic, unlike every runtime failure family of the driver above
/// (missing tool / OS refusal / retry).
#[test]
fn v05_compile_stage_split() {
    let bad = vec![(
        "main.at".to_string(),
        "fn main() int {\n    return \"not-an-int\"\n}\n".to_string(),
    )];
    let err = match lane::sys_product_backend("t09-type-error", &bad, AssemblyBackend::Av) {
        Err(e) => e,
        Ok(_) => panic!("type error must be refused at compile stage"),
    };
    assert!(
        err.contains("compile sys") && !err.contains("av exec"),
        "a compile-stage diagnostic, got: {err}"
    );
}
