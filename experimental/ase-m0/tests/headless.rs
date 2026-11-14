//! PLAN-1021 §6 headless suite (V-01..V-05), all through the public
//! library surface on copies of the checked-in fixtures (edits, junctions
//! and store dirs never touch the worktree). Oracles are hand-computed
//! expected values; the consumed-source identity (sha256/input_hash) is
//! asserted so a stub returning a constant cannot pass.

use ace_core::source::cst::FrontBudget;
use ase_m0::{load_registry, run_app, AseError, Kind, RunOptions, RunReport};
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ase-m0-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let target = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A scratch registry directory with the fixtures copied in.
fn scratch_registry(tag: &str) -> PathBuf {
    let dir = scratch(tag);
    copy_dir(&fixtures(), &dir);
    dir
}

fn app_block(identity: &str, package: &str, root: &str, capabilities: &str) -> String {
    format!(
        "[[app]]\nidentity = \"{identity}\"\nversion = \"1.0.0\"\npackage = \"{package}\"\n\
root = \"{root}\"\nentry = \"main\"\nsources = [\"main.at\"]\ncapabilities = {capabilities}\n\n"
    )
}

fn registry_text(a_root: &str, b_root: &str) -> String {
    app_block("auto.demo.a", "auto_demo_a", a_root, "[]")
        + &app_block("auto.demo.b", "auto_demo_b", b_root, "[]")
}

fn write_registry(dir: &Path, text: String) {
    std::fs::write(dir.join("registry.toml"), text).unwrap();
}

fn run(registry_dir: &Path, uri: &str, store: &Path) -> Result<RunReport, AseError> {
    let registry = load_registry(registry_dir).unwrap();
    run_app(&registry, uri, &RunOptions::new(store.to_path_buf()))
}

fn main_source(dir: &Path, identity: &str) -> PathBuf {
    dir.join("apps")
        .join(identity)
        .join("1.0.0")
        .join("main.at")
}

/// V-01 (AC-01): strict supported domain — the near misses of the two
/// registered root routes are refused by name BEFORE any source read or
/// dispatch (dispatch count observed at the real adapter boundary is 0).
#[test]
fn v01_strict_domain_authorization_and_zero_dispatch_rejections() {
    let dir = scratch_registry("v01");
    let store = dir.join("store");
    for uri in [
        "http://auto.demo.a/",
        "app:auto.demo.a/",
        "app:///",
        "app://Auto.Demo.A/",
        "app://user@auto.demo.a/",
        "app://auto.demo.a:8080/",
        "app://auto.demo.a",
        "app://auto.demo.a/ui",
        "app://auto.demo.a/?q=1",
        "app://auto.demo.a/#f",
        "app://unknown.app/",
    ] {
        let err = run(&dir, uri, &store).unwrap_err();
        assert_eq!(err.dispatches, 0, "{uri}: {err}");
        assert!(
            err.kind == Kind::Uri || err.kind == Kind::Authz,
            "{uri}: {err}"
        );
    }
    // Unregistered version: named refusal before dispatch; the registered
    // version passes admission (its real run is asserted in v03).
    let registry = load_registry(&dir).unwrap();
    let err = run_app(
        &registry,
        "app://auto.demo.a/",
        &RunOptions {
            version: Some("2.0.0".to_string()),
            budget: None,
            store_dir: store.clone(),
        },
    )
    .unwrap_err();
    assert_eq!(err.kind, Kind::Authz, "{err}");
    assert_eq!(err.dispatches, 0);
    run_app(
        &registry,
        "app://auto.demo.a/",
        &RunOptions {
            version: Some("1.0.0".to_string()),
            budget: None,
            store_dir: store,
        },
    )
    .unwrap();
    // Declared extra host capability: refused before dispatch (the M0
    // domain carries no extra capabilities).
    write_registry(
        &dir,
        app_block(
            "auto.demo.a",
            "auto_demo_a",
            "apps/auto.demo.a/1.0.0",
            "[\"gpu\"]",
        ) + &app_block("auto.demo.b", "auto_demo_b", "apps/auto.demo.b/1.0.0", "[]"),
    );
    let err = run(&dir, "app://auto.demo.a/", &dir.join("store-caps")).unwrap_err();
    assert_eq!(err.kind, Kind::Authz, "{err}");
    assert_eq!(err.dispatches, 0);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// V-02 (AC-02): reads stay inside the allowed root — root escapes,
/// reparse points and missing pieces are refused by name with zero
/// dispatches; a host-private path outside the root is never read.
#[test]
fn v02_root_containment_reparse_and_missing() {
    let dir = scratch_registry("v02");
    let store = dir.join("store");

    // Root escape via parent references (host-private target exists but
    // must not be read — refusal fires before any content read).
    let private = dir.join("private-root");
    std::fs::create_dir_all(&private).unwrap();
    std::fs::write(private.join("main.at"), "fn main() int { return 11 }\n").unwrap();
    write_registry(
        &dir,
        registry_text("../../private-root", "apps/auto.demo.b/1.0.0"),
    );
    let err = run(&dir, "app://auto.demo.a/", &store).unwrap_err();
    assert_eq!(err.kind, Kind::SourceEscape, "{err}");
    assert_eq!(err.dispatches, 0);

    // Absolute root refused the same way.
    let absolute = if cfg!(windows) {
        "C:/private-root"
    } else {
        "/private-root"
    };
    write_registry(&dir, registry_text(absolute, "apps/auto.demo.b/1.0.0"));
    let err = run(&dir, "app://auto.demo.a/", &store).unwrap_err();
    assert_eq!(err.kind, Kind::SourceEscape, "{err}");

    // Multi-segment source file names are refused (registered closure is
    // flat: single names under the app root).
    write_registry(
        &dir,
        "[[app]]\nidentity = \"auto.demo.a\"\nversion = \"1.0.0\"\npackage = \"auto_demo_a\"\n\
root = \"apps/auto.demo.a/1.0.0\"\nentry = \"main\"\n\
sources = [\"main.at\", \"../../private-root/main.at\"]\ncapabilities = []\n\n"
            .to_owned()
            + &app_block("auto.demo.b", "auto_demo_b", "apps/auto.demo.b/1.0.0", "[]"),
    );
    let err = run(&dir, "app://auto.demo.a/", &store).unwrap_err();
    assert_eq!(err.kind, Kind::SourceEscape, "{err}");
    assert_eq!(err.dispatches, 0);

    // Missing registered file.
    std::fs::remove_file(main_source(&dir, "auto.demo.b")).unwrap();
    let err = run(&dir, "app://auto.demo.b/", &store).unwrap_err();
    assert_eq!(err.kind, Kind::SourceMissing, "{err}");
    assert_eq!(err.dispatches, 0);

    // Reparse point inside the allowed root (junction over the version
    // dir): refused by name, nothing read through the link.
    #[cfg(windows)]
    {
        let a_root = dir.join("apps").join("auto.demo.a").join("1.0.0");
        let foreign = dir.parent().unwrap().join("ase-m0-v02-foreign");
        let _ = std::fs::remove_dir_all(&foreign);
        std::fs::create_dir_all(&foreign).unwrap();
        std::fs::write(foreign.join("main.at"), "fn main() int { return 11 }\n").unwrap();
        std::fs::remove_dir_all(&a_root).unwrap();
        let output = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&a_root)
            .arg(&foreign)
            .output()
            .unwrap();
        assert!(output.status.success(), "mklink /J failed: {output:?}");
        let err = run(&dir, "app://auto.demo.a/", &store).unwrap_err();
        assert_eq!(err.kind, Kind::SourceReparse, "{err}");
        assert_eq!(err.dispatches, 0);
        // Remove the LINK itself only (never traverse), then clean up the
        // foreign target dir.
        std::fs::remove_dir(&a_root).unwrap();
        std::fs::remove_dir_all(&foreign).unwrap();
    }

    std::fs::remove_dir_all(&dir).unwrap();
}

/// V-03 (AC-03): both identities execute for real through the plain-source
/// front door and the public AV chain (11/22), the consumed-source identity
/// is real (differs per closure, changes with the source), and renaming /
/// re-ordering registrations does not corrupt resolution — the value
/// follows the registered closure, not the name or the order.
#[test]
fn v03_real_av_execution_and_source_identity() {
    let dir = scratch_registry("v03");
    let ra = run(&dir, "app://auto.demo.a/", &dir.join("store-a")).unwrap();
    assert_eq!(ra.value, 11);
    assert_eq!(ra.dispatches, 1);
    assert_eq!(ra.entry, "auto_demo_a::main");
    assert_eq!(ra.sources.len(), 1);
    assert_eq!(ra.sources[0].0, "main");
    let rb = run(&dir, "app://auto.demo.b/", &dir.join("store-b")).unwrap();
    assert_eq!(rb.value, 22);
    assert_eq!(rb.entry, "auto_demo_b::main");
    assert_ne!(ra.input_hash, rb.input_hash);
    assert!(ra.lowered >= 1 && rb.lowered >= 1);

    // Swapped registration: the a-identity now runs the closure registered
    // to it (b's sources) and returns b's value — resolution is by
    // registration. The consumed-source digest matches b's file, while the
    // package input_hash stays a distinct identity (the package name is
    // part of the binding hash).
    write_registry(
        &dir,
        registry_text("apps/auto.demo.b/1.0.0", "apps/auto.demo.a/1.0.0"),
    );
    let swapped = run(&dir, "app://auto.demo.a/", &dir.join("store-swap")).unwrap();
    assert_eq!(swapped.value, 22);
    assert_eq!(swapped.sources[0].2, rb.sources[0].2);
    assert_ne!(swapped.input_hash, ra.input_hash);
    assert_ne!(swapped.input_hash, rb.input_hash);

    // Re-ordered registrations (b first): same independent results.
    write_registry(
        &dir,
        app_block("auto.demo.b", "auto_demo_b", "apps/auto.demo.b/1.0.0", "[]")
            + &app_block("auto.demo.a", "auto_demo_a", "apps/auto.demo.a/1.0.0", "[]"),
    );
    assert_eq!(
        run(&dir, "app://auto.demo.a/", &dir.join("store-a2"))
            .unwrap()
            .value,
        11
    );
    assert_eq!(
        run(&dir, "app://auto.demo.b/", &dir.join("store-b2"))
            .unwrap()
            .value,
        22
    );

    // Editing the source changes the consumed identity (a stub returning a
    // constant could not reproduce the hash).
    std::fs::write(
        main_source(&dir, "auto.demo.a"),
        "fn main() int { return 12 }\n",
    )
    .unwrap();
    let edited = run(&dir, "app://auto.demo.a/", &dir.join("store-a3")).unwrap();
    assert_eq!(edited.value, 12);
    assert_ne!(edited.input_hash, ra.input_hash);
    assert_ne!(edited.sources[0].2, ra.sources[0].2);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// V-04 (AC-04): editing A to 12 leaves B at 22; a broken A source fails
/// in the compile phase with one dispatch; repairing it makes a real retry
/// succeed.
#[test]
fn v04_edit_isolation_bad_source_and_repair() {
    let dir = scratch_registry("v04");
    let first = run(&dir, "app://auto.demo.a/", &dir.join("store-a")).unwrap();
    assert_eq!(first.value, 11);

    std::fs::write(
        main_source(&dir, "auto.demo.a"),
        "fn main() int { return 12 }\n",
    )
    .unwrap();
    let edited = run(&dir, "app://auto.demo.a/", &dir.join("store-a2")).unwrap();
    assert_eq!(edited.value, 12);
    // B is untouched by A's edit.
    assert_eq!(
        run(&dir, "app://auto.demo.b/", &dir.join("store-b"))
            .unwrap()
            .value,
        22
    );

    // Broken source: refused in the compile phase (the dispatch happened;
    // the chain itself rejected the source).
    std::fs::write(
        main_source(&dir, "auto.demo.a"),
        "fn main() int { return }\n",
    )
    .unwrap();
    let err = run(&dir, "app://auto.demo.a/", &dir.join("store-a3")).unwrap_err();
    assert_eq!(err.kind, Kind::Compile, "{err}");
    assert_eq!(err.phase.as_str(), "compile");
    assert_eq!(err.dispatches, 1);

    // Repair: a real retry succeeds.
    std::fs::write(
        main_source(&dir, "auto.demo.a"),
        "fn main() int { return 12 }\n",
    )
    .unwrap();
    let repaired = run(&dir, "app://auto.demo.a/", &dir.join("store-a4")).unwrap();
    assert_eq!(repaired.value, 12);
    std::fs::remove_dir_all(&dir).unwrap();
}

/// V-05 (AC-05): a tiny public-entry budget is a resource refusal (never a
/// syntax error, never success); repeated one-shot runs are cold and agree;
/// the AV store's file ownership is released after the run (the store dir
/// can be removed — on Windows this fails while sqlite holds the file).
#[test]
fn v05_resource_budget_repeat_and_store_release() {
    let dir = scratch_registry("v05");
    let registry = load_registry(&dir).unwrap();

    // Tiny parse depth through the public front budget: resource refusal.
    let err = run_app(
        &registry,
        "app://auto.demo.a/",
        &RunOptions {
            version: None,
            budget: Some(FrontBudget {
                work_bytes: 64 << 20,
                max_parse_depth: 1,
            }),
            store_dir: dir.join("store-budget"),
        },
    )
    .unwrap_err();
    assert_eq!(err.kind, Kind::Resource, "{err}");
    assert_eq!(err.dispatches, 1);
    assert!(err.message.contains("source.resource"), "{err}");

    // Repeated one-shot execution: each run is its own cold dispatch with
    // the same consumed identity; no warm cache or session is claimed.
    let r1 = run(&dir, "app://auto.demo.a/", &dir.join("store-r1")).unwrap();
    let r2 = run(&dir, "app://auto.demo.a/", &dir.join("store-r2")).unwrap();
    assert_eq!((r1.value, r2.value), (11, 11));
    assert_eq!(r1.input_hash, r2.input_hash);
    assert_eq!(r1.dispatches, 1);
    assert_eq!(r2.dispatches, 1);
    assert_ne!(r1.sources[0].2, "");

    // Store ownership released: the run's target is dropped inside
    // run_app, so the store tree is deletable (file locks would fail).
    std::fs::remove_dir_all(dir.join("store-r1")).unwrap();
    std::fs::remove_dir_all(dir.join("store-r2")).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
}
