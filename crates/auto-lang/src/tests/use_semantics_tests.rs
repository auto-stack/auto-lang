//! Plan 545: `use` namespace-semantics tests.
//!
//! Target semantics (Rust-2018 style):
//! - bare `use db`      → namespace only (`db.X` qualified access; bare `X` is an error)
//! - `use db: *`        → explicit flat import (old bare behavior)
//! - `use db: a, b`     → named import (unchanged)
//!
//! Spike (task 1) anchors the CURRENT resolution paths for qualified and flat
//! access through the full VM pipeline (path-anchored module resolution, no
//! chdir — safe under parallel test runners).

use std::path::PathBuf;

/// Temp workspace with a `db.at` module next to the entry source; returns the
/// entry file path (caller keeps the TempDir alive until after the run).
struct ModuleFixture {
    _dir: tempfile::TempDir,
    main_path: PathBuf,
}

fn fixture(db_src: &str, main_src: &str) -> ModuleFixture {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("db.at"), db_src).unwrap();
    let main_path = dir.path().join("main.at");
    std::fs::write(&main_path, main_src).unwrap();
    ModuleFixture { _dir: dir, main_path }
}

fn run(fixture: &ModuleFixture, code: &str) -> Result<(String, String), String> {
    let path = fixture.main_path.to_string_lossy().to_string();
    crate::run_with_capture_and_path(code, &path).map_err(|e| format!("{e:?}"))
}

const DB: &str = "pub fn add(a int, b int) int {\n    a + b\n}\n";

/// Spike A: qualified access `db.add(2,3)` after bare `use db` — the target
/// canonical form. Must keep working before AND after the semantic change.
#[test]
fn a_bare_use_qualified_call() {
    let fx = fixture(DB, "");
    let (_, out) = run(
        &fx,
        "use db\nfn main() {\n    print(db.add(2, 3))\n}\n",
    )
    .expect("qualified call should work today");
    assert_eq!(out, "5\n");
}

/// B: flat bare-name access `add(2,3)` after bare `use db` must be a compile
/// error under Plan 545 semantics (bare use = namespace-only), with the D6
/// hint naming the owning module and the two opt-in forms.
#[test]
fn b_bare_use_flat_call_is_error_with_hint() {
    let fx = fixture(DB, "");
    let err = run(
        &fx,
        "use db\nfn main() {\n    print(add(2, 3))\n}\n",
    )
    .expect_err("flat call must not resolve after bare use (Plan 545)");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("db.add") && msg.contains("use db: *"),
        "D6 hint missing owner/fix suggestion: {msg}"
    );
}

/// Spike C: named import — unchanged semantics.
#[test]
fn c_named_import() {
    let fx = fixture(DB, "");
    let (_, out) = run(
        &fx,
        "use db: add\nfn main() {\n    print(add(2, 3))\n}\n",
    )
    .expect("named import works");
    assert_eq!(out, "5\n");
}

/// Spike D: explicit wildcard — the post-change opt-in for flat access.
#[test]
fn d_wildcard_import() {
    let fx = fixture(DB, "");
    let (_, out) = run(
        &fx,
        "use db: *\nfn main() {\n    print(add(2, 3))\n}\n",
    )
    .expect("wildcard import works");
    assert_eq!(out, "5\n");
}

/// Plan 545 test-design #4: same-name fn from two modules pulled in via two
/// wildcards with DIFFERENT definitions → compile error naming both modules.
#[test]
fn e_wildcard_conflict_same_name_different_def_is_error() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("db.at"),
        "pub fn load() int {\n    1\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("helpers.at"),
        "pub fn load() int {\n    2\n}\n",
    )
    .unwrap();
    let main_path = dir.path().join("main.at");
    std::fs::write(&main_path, "").unwrap();
    let path = main_path.to_string_lossy().to_string();
    let err = crate::run_with_capture_and_path(
        "use db: *\nuse helpers: *\nfn main() {\n    print(load())\n}\n",
        &path,
    )
    .expect_err("conflicting wildcard imports must be a compile error");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("db") && msg.contains("helpers") && msg.contains("load"),
        "conflict error must name both modules and the symbol: {msg}"
    );
}

/// Plan 545 test-design #4 (re-export leg): same name + same definition from
/// two modules (re-export shape) → NOT an error.
#[test]
fn f_wildcard_same_def_reexport_is_not_error() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("db.at"),
        "pub fn load() int {\n    1\n}\n",
    )
    .unwrap();
    // helpers re-exports the identical signature+body shape of `load`
    std::fs::write(
        dir.path().join("helpers.at"),
        "pub fn load() int {\n    1\n}\n",
    )
    .unwrap();
    let main_path = dir.path().join("main.at");
    std::fs::write(&main_path, "").unwrap();
    let path = main_path.to_string_lossy().to_string();
    let (_, out) = crate::run_with_capture_and_path(
        "use db: *\nuse helpers: *\nfn main() {\n    print(load())\n}\n",
        &path,
    )
    .expect("identical definitions (re-export shape) must not conflict");
    assert_eq!(out, "1\n");
}

/// Plan 545 test-design #6 (transitive isolation): `db.at` itself does
/// `use tools` — the importing module must not gain `tools` symbols in its
/// bare namespace through a bare `use db` (module form).
#[test]
fn g_bare_use_does_not_leak_transitive_bare_symbols() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("tools.at"),
        "pub fn magic() int {\n    42\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("db.at"),
        "use tools\npub fn add(a int, b int) int {\n    a + b\n}\n",
    )
    .unwrap();
    let main_path = dir.path().join("main.at");
    std::fs::write(&main_path, "").unwrap();
    let path = main_path.to_string_lossy().to_string();
    // `magic()` (from db's own bare use of tools) must NOT resolve in main.
    let err = crate::run_with_capture_and_path(
        "use db\nfn main() {\n    print(magic())\n}\n",
        &path,
    )
    .expect_err("transitive bare symbols must not leak into importer");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("magic"),
        "error should name the unresolved symbol: {msg}"
    );
}
