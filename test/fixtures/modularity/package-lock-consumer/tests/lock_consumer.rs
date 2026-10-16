//! PLAN-795 V01 — an EXTERNAL consumer using only `auto-package-lock`.
//!
//! Everything here must hold without auto-lang/auto-man/auto-gen/auto-ui/
//! auto-vm anywhere in the build (enforced structurally by this crate's
//! manifest and by `verify-deps.sh`). It proves an external, VM/compiler/
//! renderer-free consumer can read, write and verify the existing `pac.lock`
//! format with frozen bytes and error stages.

use auto_package_lock::{LockEntry, LockFile, ResolvedDependency};
use auto_val::AutoPath;

fn ap(path: &std::path::Path) -> AutoPath {
    AutoPath::new(path.to_string_lossy().to_string().as_str())
}

fn sample_entries() -> Vec<LockEntry> {
    vec![
        LockEntry {
            name: "zeta-lib".to_string(),
            version: "2.0.0".to_string(),
            url: "https://example.com/zeta".to_string(),
            commit: "c0ffee".to_string(),
            path: "deps\\zeta-lib".to_string(),
        },
        LockEntry {
            name: "alpha-依赖".to_string(),
            version: "1.2.3".to_string(),
            url: String::new(),
            commit: String::new(),
            path: "D:/tmp dir/依赖库".to_string(),
        },
    ]
}

#[test]
fn external_consumer_writes_and_reads_frozen_toml() {
    let dir =
        std::env::temp_dir().join(format!("package_lock_consumer_{}_toml", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let lock_path = dir.join("pac.lock");

    let lock = LockFile::from_dependencies("0.1.0", sample_entries());
    lock.save(&ap(&lock_path)).unwrap();

    // Frozen bytes from the historical auto-man implementation (pre
    // extraction, 579f6dec0): an external consumer writes the exact same
    // pac.lock an AutoMan build wrote. from_dependencies applies the
    // canonical name ordering, so the golden is in sorted order.
    let text = std::fs::read_to_string(&lock_path).unwrap();
    assert_eq!(
        text,
        "automan_version = \"0.1.0\"\n\
         \n\
         [[dependencies]]\n\
         name = \"alpha-依赖\"\n\
         version = \"1.2.3\"\n\
         url = \"\"\n\
         commit = \"\"\n\
         path = \"D:/tmp dir/依赖库\"\n\
         \n\
         [[dependencies]]\n\
         name = \"zeta-lib\"\n\
         version = \"2.0.0\"\n\
         url = \"https://example.com/zeta\"\n\
         commit = \"c0ffee\"\n\
         path = 'deps\\zeta-lib'\n"
    );

    let loaded = LockFile::load(&ap(&lock_path)).unwrap();
    assert_eq!(loaded.automan_version, "0.1.0");
    assert_eq!(loaded.dependencies[0].name, "alpha-依赖");
    assert_eq!(loaded.dependencies[1].name, "zeta-lib");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn external_consumer_writes_and_reads_frozen_json() {
    let dir =
        std::env::temp_dir().join(format!("package_lock_consumer_{}_json", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let lock_path = dir.join("pac.lock.json");

    let lock = LockFile::from_dependencies("0.1.0", sample_entries());
    lock.save(&ap(&lock_path)).unwrap();

    let text = std::fs::read_to_string(&lock_path).unwrap();
    assert_eq!(
        text,
        "{\n  \"automan_version\": \"0.1.0\",\n  \"dependencies\": [\n    {\n      \"name\": \"alpha-依赖\",\n      \"version\": \"1.2.3\",\n      \"url\": \"\",\n      \"commit\": \"\",\n      \"path\": \"D:/tmp dir/依赖库\"\n    },\n    {\n      \"name\": \"zeta-lib\",\n      \"version\": \"2.0.0\",\n      \"url\": \"https://example.com/zeta\",\n      \"commit\": \"c0ffee\",\n      \"path\": \"deps\\\\zeta-lib\"\n    }\n  ]\n}"
    );

    let loaded = LockFile::load(&ap(&lock_path)).unwrap();
    assert_eq!(loaded.dependencies[1].path, "deps\\zeta-lib");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn external_consumer_verifies_dependencies() {
    let dir = std::env::temp_dir().join(format!(
        "package_lock_consumer_{}_verify",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let materialized = dir.join("materialized");
    std::fs::create_dir_all(&materialized).unwrap();

    let lock = LockFile::from_dependencies(
        "0.1.0",
        vec![
            LockEntry {
                name: "git-dep".to_string(),
                version: "1.0.0".to_string(),
                url: "https://example.com/git-dep".to_string(),
                commit: "abc123".to_string(),
                path: String::new(),
            },
            LockEntry {
                name: "local-dep".to_string(),
                version: "0.5.0".to_string(),
                url: String::new(),
                commit: String::new(),
                path: materialized.to_string_lossy().to_string(),
            },
        ],
    );

    // Positive control: everything matches and the local path is materialized.
    assert_eq!(
        auto_package_lock::verify_dependencies(
            &lock.dependencies,
            &[
                ResolvedDependency::new("git-dep", "1.0.0"),
                ResolvedDependency::new("local-dep", "0.5.0"),
            ]
        ),
        Ok(())
    );

    // Negative controls with frozen error text and aggregation order.
    let err = auto_package_lock::verify_dependencies(
        &lock.dependencies,
        &[
            ResolvedDependency::new("local-dep", "9.9.9"),
            ResolvedDependency::new("ghost", "1.0.0"),
        ],
    )
    .unwrap_err();
    assert_eq!(
        err,
        "Dependency 'local-dep' version mismatch: lockfile has 0.5.0, but resolved 9.9.9\n\
         Dependency 'ghost' not found in lock file"
    );

    // Unmaterialized local path dep (empty commit → path existence check).
    std::fs::remove_dir_all(&materialized).unwrap();
    let err = auto_package_lock::verify_dependencies(
        &lock.dependencies,
        &[ResolvedDependency::new("local-dep", "0.5.0")],
    )
    .unwrap_err();
    assert_eq!(
        err,
        format!(
            "Dependency 'local-dep' path '{}' is not materialized",
            materialized.to_string_lossy()
        )
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn external_consumer_error_stages_are_frozen() {
    let err = LockFile::load(&AutoPath::new("Z:/definitely/not/here/pac.lock")).unwrap_err();
    assert!(
        err.starts_with("Failed to read lock file 'Z:/definitely/not/here/pac.lock': "),
        "got: {err}"
    );

    // Lookup: first-match semantics with a caller-supplied tool version.
    let lock = LockFile::new("3.2.1-tool");
    assert_eq!(lock.automan_version, "3.2.1-tool");
    assert!(lock.get("anything").is_none());
    assert!(auto_package_lock::verify_dependencies(&lock.dependencies, &[]).is_ok());
}
