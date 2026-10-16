//! PLAN-800 V01 — an EXTERNAL consumer using only `auto-package-resolver`.
//!
//! Everything here must hold without auto-lang/auto-man/auto-gen/auto-ui/
//! auto-vm/auto-val/auto-ast anywhere in the build (enforced structurally
//! by this crate's manifest and by `verify-deps.sh`). It proves an
//! external, language-free consumer can drive the extracted dependency
//! view and path rules on real plain directories (no junctions/symlinks):
//! fresh-manifest resolution, ambiguity, missing-module errors, opaque
//! mode labels, and same-name replacement visibility.

use auto_package_resolver::{DependencyEntry, PackageResolverCore};
use std::fs;
use std::path::{Path, PathBuf};

/// Consumer-side mode label — the core must accept ANY tag type without
/// knowing what it means (ExecutionMode stays in the auto-man adapter).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConsumerMode {
    Native,
    Wasm,
}

impl Default for ConsumerMode {
    fn default() -> Self {
        ConsumerMode::Native
    }
}

/// Fresh plain directory per test (pid-suffixed to survive parallel runs).
fn fresh_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "package_resolver_consumer_{}_{}",
        std::process::id(),
        tag
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn external_consumer_resolves_file_and_directory_modules() {
    let root = fresh_dir("resolve");
    let dep = root.join("vendor").join("database");
    write(&dep.join("connection.at"), "// connection");
    write(&dep.join("models").join("mod.at"), "// models");

    let core = PackageResolverCore::<ConsumerMode>::new(root.join("app"), root.join("stdlib"));
    // dependency view is required before in-dependency resolution works
    let mut core = core;
    core.add_dependency("database".to_string(), dep.clone(), ConsumerMode::Wasm);

    assert_eq!(
        core.resolve_in_dependency("database", &["connection"])
            .unwrap(),
        dep.join("connection.at")
    );
    assert_eq!(
        core.resolve_in_dependency("database", &["models"]).unwrap(),
        dep.join("models").join("mod.at")
    );
    // deep segment path
    write(
        &dep.join("api").join("v1").join("handlers.at"),
        "// handlers",
    );
    assert_eq!(
        core.resolve_in_dependency("database", &["api", "v1", "handlers"])
            .unwrap(),
        dep.join("api").join("v1").join("handlers.at")
    );
}

#[test]
fn external_consumer_ambiguity_must_not_succeed() {
    let root = fresh_dir("ambiguity");
    let dep = root.join("dep");
    // both file module and directory module present
    write(&dep.join("models.at"), "// file form");
    write(&dep.join("models").join("mod.at"), "// dir form");

    let mut core = PackageResolverCore::<ConsumerMode>::new(root.join("app"), root.join("stdlib"));
    core.add_dependency("dep".to_string(), dep.clone(), ConsumerMode::default());

    let err = core.resolve_in_dependency("dep", &["models"]).unwrap_err();
    assert!(err.starts_with("Ambiguous module 'models' in dependency - both '"));
    assert!(err.contains("' and '"));
    assert!(err.ends_with("' exist"));
}

#[test]
fn external_consumer_missing_module_error_projection() {
    let root = fresh_dir("missing");
    let dep = root.join("dep");
    write(&dep.join("keep.at"), "// keep the dir non-empty");

    let mut core = PackageResolverCore::<ConsumerMode>::new(root.join("app"), root.join("stdlib"));
    core.add_dependency("dep".to_string(), dep.clone(), ConsumerMode::default());

    let err = core
        .resolve_in_dependency("dep", &["ghost", "module"])
        .unwrap_err();
    assert!(err.contains("Module 'ghost.module' not found in dependency."));
    assert!(err.contains("Searched locations:"));
    assert!(err.contains(&format!(
        "- {} (file module)",
        dep.join("ghost").join("module.at").display()
    )));
    assert!(err.contains(&format!(
        "- {} (directory module)",
        dep.join("ghost").join("module").join("mod.at").display()
    )));
    assert!(err.contains(&format!("Dependency root: {}", dep.display())));

    // undeclared dependency name → frozen not-declared text
    let err = core.resolve_in_dependency("nope", &["x"]).unwrap_err();
    assert!(err.contains("Dependency 'nope' not declared in pac.at."));
    assert!(err.contains("Declared dependencies: dep"));
}

#[test]
fn external_consumer_find_package_path_candidates() {
    let root = fresh_dir("pkgpath");
    let project = root.join("proj");
    let std_root = root.join("stdlib");
    write(
        &project
            .join("packages")
            .join("local_pkg")
            .join("package.at"),
        "// local",
    );
    write(&std_root.join("io.at"), "// std");

    let core = PackageResolverCore::<ConsumerMode>::new(project.clone(), std_root.clone());
    assert_eq!(
        core.find_package_path("local_pkg").unwrap(),
        project.join("packages").join("local_pkg")
    );
    assert_eq!(
        core.find_package_path("std.io").unwrap(),
        std_root.join("io.at")
    );
    let err = core.find_package_path("missing").unwrap_err();
    assert_eq!(
        err,
        auto_package_resolver::PackagePathNotFound {
            searched_local: project.join("packages").join("missing")
        }
    );
}

#[test]
fn external_consumer_mode_labels_are_opaque() {
    let root = fresh_dir("mode");
    let mut core = PackageResolverCore::<ConsumerMode>::new(root.join("app"), root.join("stdlib"));

    // entry constructors work for any mode type; default comes from the
    // mode type itself (adapter's ExecutionMode::default = AutoVM analog)
    let entry: DependencyEntry<ConsumerMode> =
        DependencyEntry::with_default_mode("p".into(), root.join("p"));
    assert_eq!(entry.mode, ConsumerMode::Native);
    core.add_dependency("p".into(), root.join("p"), ConsumerMode::Wasm);

    let deps = core.dependencies();
    assert_eq!(deps.get("p").unwrap().mode, ConsumerMode::Wasm);
    assert_eq!(deps.len(), 1);
}

#[test]
fn external_consumer_same_name_replace_is_immediately_visible() {
    let root = fresh_dir("replace");
    let v1 = root.join("v1");
    let v2 = root.join("v2");
    write(&v1.join("old_mod.at"), "// old");
    write(&v2.join("new_mod.at"), "// new");

    let mut core = PackageResolverCore::<ConsumerMode>::new(root.join("app"), root.join("stdlib"));
    core.add_dependency("pkg".into(), v1.clone(), ConsumerMode::Native);
    assert_eq!(
        core.resolve_in_dependency("pkg", &["old_mod"]).unwrap(),
        v1.join("old_mod.at")
    );

    // same-name replace: single authoritative map — new path/mode win at
    // once, the old path is never consulted again (no stale shadow map)
    core.add_dependency("pkg".into(), v2.clone(), ConsumerMode::Wasm);
    assert_eq!(core.resolve("pkg").unwrap(), v2.join("package.at"));
    assert_eq!(
        core.resolve_in_dependency("pkg", &["new_mod"]).unwrap(),
        v2.join("new_mod.at")
    );
    assert!(core.resolve_in_dependency("pkg", &["old_mod"]).is_err());
    assert_eq!(
        core.dependencies().get("pkg").unwrap().mode,
        ConsumerMode::Wasm
    );
    assert_eq!(core.dependencies().len(), 1);
}

#[test]
fn external_consumer_resolve_and_existence_rules() {
    let root = fresh_dir("rules");
    let std_root = root.join("stdlib");
    write(&std_root.join("io.at"), "// std io");
    let libs = root.join("libs");
    write(&libs.join("afile.at"), "// file module");
    write(&libs.join("adir").join("package.at"), "// package dir");

    let mut core = PackageResolverCore::<ConsumerMode>::new(root.join("app"), std_root.clone());
    core.add_search_path(libs.clone());
    core.add_dependency(
        "ghost".into(),
        root.join("nowhere"),
        ConsumerMode::default(),
    );

    // std arm constructs without disk check; exists() requires the disk
    assert_eq!(core.resolve("std.io").unwrap(), std_root.join("io.at"));
    assert!(core.exists("std.io"));
    assert!(!core.exists("std.missing"));
    // declared key → exists without path presence; resolve appends package.at
    assert!(core.exists("ghost"));
    assert_eq!(
        core.resolve("ghost").unwrap(),
        root.join("nowhere").join("package.at")
    );
    // search-path arm: file module and package directory
    assert_eq!(core.resolve("afile").unwrap(), libs.join("afile.at"));
    assert_eq!(
        core.resolve("adir").unwrap(),
        libs.join("adir").join("package.at")
    );
    // frozen asymmetry: exists() has no package.at arm
    assert!(core.exists("afile"));
    assert!(!core.exists("adir"));
    // not-found error text
    let err = core.resolve("nothing").unwrap_err();
    assert!(err.starts_with("Module 'nothing' not found. Available dependencies:"));
    // composed search paths: [extra..., std, dep roots...]
    let sp = core.module_search_paths();
    assert_eq!(sp.first(), Some(&libs));
    assert!(sp.contains(&std_root));
    assert!(sp.contains(&root.join("nowhere")));
}
