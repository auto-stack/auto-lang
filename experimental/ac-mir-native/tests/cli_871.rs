//! PLAN-871 T-04 `ac-native` CLI counterexamples (V-02): the five frozen
//! profiles build and RUN through the shared-MIR backend from one CLI, the
//! switch state is observable, and a per-profile rollback is an EXPLICIT
//! refusal instead of a silent different-backend build.
//!
//! Every invocation is a child process, so the env switch is per case and
//! never races with a sibling test.

#![cfg(windows)]

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ac-native")
}

fn scratch(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ac871-cli-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

struct Out {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_cli(env_value: Option<&str>, args: &[&str]) -> Out {
    let mut cmd = Command::new(bin());
    cmd.args(args).env_remove("AC_NATIVE_BACKEND");
    if let Some(v) = env_value {
        cmd.env("AC_NATIVE_BACKEND", v);
    }
    let out = cmd.output().expect("spawn ac-native");
    Out {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

fn run_exe(path: &Path) -> Option<i32> {
    Command::new(path).status().ok().and_then(|s| s.code())
}

fn corpus(name: &str) -> String {
    common::repo_root()
        .join(format!("crates/auto-lang/test/cases/ace-m1/src/{name}.at"))
        .display()
        .to_string()
}

fn fixture(rel: &str) -> String {
    common::repo_root().join(rel).display().to_string()
}

fn build_and_check(profile: &str, src: &str, want: i32, caps: &[&str], tag: &str) {
    let dir = scratch(tag);
    let exe = dir.join("app.exe");
    let mut args: Vec<&str> = vec![
        "build-source",
        src,
        "--source-profile",
        profile,
        "--entry",
        "main",
        "--output",
        exe.to_str().unwrap(),
    ];
    for c in caps {
        args.push("--capability");
        args.push(c);
    }
    let out = run_cli(None, &args);
    assert_eq!(
        out.code,
        Some(0),
        "{profile} build failed:\n{}\n{}",
        out.stdout,
        out.stderr
    );
    assert_eq!(run_exe(&exe), Some(want), "{profile} run exit");
    let stem = exe
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap();
    let receipt = std::fs::read_to_string(dir.join(format!("{stem}.ac-link.txt")))
        .expect("receipt written next to the exe");
    assert!(
        receipt.contains(&format!("backend: {}", ac_mn::backend::Backend::Mn.tag())),
        "{receipt}"
    );
    assert!(receipt.contains("mir-lowering: "), "{receipt}");
    assert!(receipt.contains("units: "), "{receipt}");
    assert!(
        receipt.contains("unit: "),
        "per-unit identity missing:\n{receipt}"
    );
    assert!(receipt.contains("manifest-fp: "), "{receipt}");
    assert!(receipt.contains("source-profile: "), "{receipt}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli871_default_table_is_mn() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let out = run_cli(None, &["backend", "--json"]);
    assert_eq!(out.code, Some(0), "{}", out.stderr);
    for profile in ["core", "scalar", "text", "record", "text-record"] {
        assert!(
            out.stdout
                .contains(&format!("\"profile\":\"{profile}\",\"backend\":\"mn\"")),
            "profile {profile} not switched:\n{}",
            out.stdout
        );
    }
    // Per-profile rollback is visible on the SAME surface.
    let out = run_cli(Some("record=legacy"), &["backend"]);
    assert_eq!(out.code, Some(0), "{}", out.stderr);
    assert!(
        out.stdout.contains("env override: record=legacy"),
        "{}",
        out.stdout
    );
    let rows: Vec<(String, String)> = out
        .stdout
        .lines()
        .filter(|l| l.starts_with("  "))
        .filter_map(|l| {
            let mut p = l.split_whitespace();
            Some((p.next()?.to_string(), p.next()?.to_string()))
        })
        .collect();
    assert!(
        rows.contains(&("record".to_string(), "legacy".to_string())),
        "record rollback not visible: {rows:?}"
    );
    assert!(
        rows.contains(&("core".to_string(), "mn".to_string())),
        "record rollback must not touch core: {rows:?}"
    );
}

#[test]
fn cli871_five_profiles_build_and_run() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let text_cap = ["ac.runtime.text.v1"];
    // core: 745 P01 fixture, exit 14.
    build_and_check(
        "core",
        &fixture("experimental/ac-core/fixtures/source/valid/p01-precedence.at"),
        14,
        &[],
        "core",
    );
    // scalar: 767 source fixture (a - -1 == 0? the frozen fixture returns 0).
    build_and_check(
        "scalar",
        &fixture("experimental/ac-core/fixtures/scalar-source/valid/p-a-minus-1.at"),
        0,
        &[],
        "scalar",
    );
    // text: Unicode codepoint row.
    build_and_check("text", &corpus("T01"), 38, &text_cap, "text");
    // record.
    build_and_check("record", &corpus("R01"), 19, &[], "record");
    // text-record (owned): the exe owns Str allocations at runtime.
    build_and_check("text-record", &corpus("R04"), 51, &text_cap, "text-record");
}

#[test]
fn cli871_hir_build_keeps_the_retiring_receipt_needles() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let dir = scratch("hir");
    let exe = dir.join("add-2-3.exe");
    let out = run_cli(
        None,
        &[
            "build",
            &fixture("experimental/ac-core/fixtures/native/add-2-3.atom"),
            "--entry",
            "d_entry",
            "--output",
            exe.to_str().unwrap(),
        ],
    );
    assert_eq!(out.code, Some(0), "{}", out.stderr);
    assert_eq!(run_exe(&exe), Some(5));
    let receipt =
        std::fs::read_to_string(dir.join("add-2-3.ac-link.txt")).expect("receipt written");
    assert!(
        receipt.contains("hir-schema: auto.hir.core.draft"),
        "{receipt}"
    );
    assert!(receipt.contains("hir-revision: 1"), "{receipt}");
    assert!(receipt.contains("backend: mn:"), "{receipt}");
    assert!(receipt.contains("entry: d_entry"), "{receipt}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli871_legacy_resolved_profile_is_an_explicit_refusal() {
    let dir = scratch("legacy-refusal");
    let exe = dir.join("app.exe");
    let out = run_cli(
        Some("record=legacy"),
        &[
            "build-source",
            &corpus("R01"),
            "--source-profile",
            "record",
            "--entry",
            "main",
            "--output",
            exe.to_str().unwrap(),
        ],
    );
    assert_eq!(
        out.code,
        Some(2),
        "stdout {}\nstderr {}",
        out.stdout,
        out.stderr
    );
    assert!(
        out.stderr.contains("RETIRED") && out.stderr.contains("mn"),
        "the refusal must name the rollback and the legacy face:\n{}",
        out.stderr
    );
    assert!(!exe.exists(), "a refused build must not publish an exe");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli871_malformed_switch_is_an_explicit_refusal() {
    let dir = scratch("malformed");
    let exe = dir.join("app.exe");
    let out = run_cli(
        Some("core=clif"),
        &[
            "build-source",
            &fixture("experimental/ac-core/fixtures/source/valid/p01-precedence.at"),
            "--source-profile",
            "core",
            "--entry",
            "main",
            "--output",
            exe.to_str().unwrap(),
        ],
    );
    assert_eq!(out.code, Some(2), "stderr {}", out.stderr);
    assert!(out.stderr.contains("unknown backend"), "{}", out.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}
