//! Plan 387 W5b: behavior-parity tests for a2r actor translation.
//!
//! Each test transpiles an Auto actor `.at` to Rust, compiles+runs it as a
//! standalone crate (linking a2r-std + tokio), and asserts stdout matches the
//! VM actor golden output byte-for-byte. Slow (cargo build per case); #[ignore]d.

use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

/// The a2r/VM test root, relative to the workspace root (CARGO_MANIFEST_DIR is
/// crates/a2r-actor-tests, so workspace root is two levels up).
fn workspace_root() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.parent().and_then(|p| p.parent()).unwrap().to_path_buf()
}

/// Locate the `.at` and VM `.expected.out` for a case.
/// `case` is like "001_start_hook"; the file stem inside matches the last segment.
fn case_paths(case: &str) -> (PathBuf, PathBuf) {
    let root = workspace_root();
    let stem = case.rsplit('_').next().unwrap(); // "001_start_hook" → "start_hook"? No.
    // The directory is `NNN_<name>` and files are `<name>.at`. Derive <name>
    // by stripping the leading `NNN_`.
    let name = case.split_once('_').map(|(_, n)| n).unwrap_or(case);
    let at = root
        .join("crates/auto-lang/test/a2r/22_actors")
        .join(case)
        .join(format!("{}.at", name));
    let vm_out = root
        .join("crates/auto-lang/test/vm/23_actor")
        .join(case)
        .join(format!("{}.expected.out", name));
    let _ = stem; // silence unused
    (at, vm_out)
}

/// Transpile the given `.at` to Rust source. Runs on a 32MB-stack thread
/// (the Pratt parser overflows the default 2MB main thread stack).
fn transpile(at_path: &std::path::Path) -> String {
    let mut src = String::new();
    std::fs::File::open(at_path)
        .unwrap_or_else(|e| panic!("open {}: {}", at_path.display(), e))
        .read_to_string(&mut src)
        .unwrap();
    // auto_lang::trans::rust::transpile_rust is pub; call it on a big-stack thread.
    let src_clone = src.clone();
    let child = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || -> Vec<u8> {
            let mut rcode = auto_lang::trans::rust::transpile_rust("test", &src_clone).unwrap();
            let rs = rcode.done().unwrap();
            rs.to_vec()
        })
        .unwrap();
    let bytes = child.join().unwrap();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Build the Cargo.toml for the throwaway crate. Each case gets a UNIQUE
/// package name so cargo doesn't reuse a cached binary across cases when they
/// share a target dir. Uses path deps against the in-tree a2r-std; the empty
/// `[workspace]` table keeps it out of the host workspace.
fn throwaway_cargo_toml(case: &str) -> String {
    // Sanitize case name into a valid crate name (replace non-alnum with _).
    let pkg = format!("actor_{}", case.replace(|c: char| !c.is_alphanumeric(), "_"));
    format!(
        r#"[package]
name = "{pkg}"
version = "0.1.0"
edition = "2021"

[dependencies]
a2r-std = {{ path = "../../../crates/a2r-std" }}
tokio = {{ version = "1", features = ["full"] }}

[[bin]]
name = "{pkg}"
path = "src/main.rs"

[workspace]
"#,
        pkg = pkg
    )
}

/// Transpile, compile, and run an actor case; assert stdout == VM golden.
fn assert_actor_parity(case: &str) {
    let (at, vm_out) = case_paths(case);
    let expected = std::fs::read_to_string(&vm_out)
        .unwrap_or_else(|e| panic!("read VM golden {}: {}", vm_out.display(), e));

    let generated = transpile(&at);

    // Write a throwaway crate into a temp dir under the workspace so it reuses
    // the workspace target cache (and the a2r-std path dep resolves).
    let root = workspace_root();
    let tmp = root.join("target").join("a2r-actor-cases").join(case);
    let src_dir = tmp.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("main.rs"), &generated).unwrap();
    std::fs::write(tmp.join("Cargo.toml"), throwaway_cargo_toml(case)).unwrap();

    // Build + run via `cargo run` in the temp dir. All throwaway crates share a
    // single CARGO_TARGET_DIR so tokio (and deps) compile only once across cases.
    let shared_target = root.join("target").join("a2r-actor-cases-target");
    let output = Command::new("cargo")
        .arg("run")
        .env("CARGO_TARGET_DIR", &shared_target)
        .current_dir(&tmp)
        .output()
        .unwrap_or_else(|e| panic!("cargo run in {}: {}", tmp.display(), e));

    if !output.status.success() {
        panic!(
            "actor case {} failed to compile/run.\n--- stderr ---\n{}",
            case,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let actual = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        actual, expected,
        "actor case {} stdout mismatch.\n--- generated main.rs ---\n{}\n--- expected (VM) ---\n{}\n--- actual ---\n{}",
        case, generated, expected, actual
    );
}

#[test]
#[ignore] // slow: compiles a throwaway crate
fn actor_001_start_hook() {
    assert_actor_parity("001_start_hook");
}

#[test]
#[ignore]
fn actor_002_message_handler() {
    assert_actor_parity("002_message_handler");
}

#[test]
#[ignore]
fn actor_003_multi_message() {
    assert_actor_parity("003_multi_message");
}

#[test]
#[ignore]
fn actor_004_else_handler() {
    assert_actor_parity("004_else_handler");
}

#[test]
#[ignore]
fn actor_005_state_write() {
    assert_actor_parity("005_state_write");
}

#[test]
#[ignore]
fn actor_006_state_increment() {
    assert_actor_parity("006_state_increment");
}
