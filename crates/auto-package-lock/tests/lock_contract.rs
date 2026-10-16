//! PLAN-795 V02/V03 — format-freeze and verification contract, exercised
//! against the core directly (no auto-man in scope). Golden bytes were
//! captured from the pre-extraction `auto-man` implementation at 579f6dec0;
//! these tests pin the extraction to bit-for-bit format compatibility.

use auto_package_lock::{find_entry, LockEntry, LockFile, ResolvedDependency};
use auto_val::AutoPath;

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

fn ap(path: &std::path::Path) -> AutoPath {
    AutoPath::new(path.to_string_lossy().to_string().as_str())
}

#[test]
fn v02_toml_bytes_are_frozen() {
    // Unsorted entry list serialized directly (mirrors the historical
    // direct-serde path over a hand-built LockFile).
    let lock = LockFile {
        automan_version: "0.1.0".to_string(),
        dependencies: sample_entries(),
    };
    let toml_text = toml::to_string_pretty(&lock).unwrap();
    assert_eq!(
        toml_text,
        "automan_version = \"0.1.0\"\n\
         \n\
         [[dependencies]]\n\
         name = \"zeta-lib\"\n\
         version = \"2.0.0\"\n\
         url = \"https://example.com/zeta\"\n\
         commit = \"c0ffee\"\n\
         path = 'deps\\zeta-lib'\n\
         \n\
         [[dependencies]]\n\
         name = \"alpha-依赖\"\n\
         version = \"1.2.3\"\n\
         url = \"\"\n\
         commit = \"\"\n\
         path = \"D:/tmp dir/依赖库\"\n"
    );
}

#[test]
fn v02_json_bytes_are_frozen() {
    let lock = LockFile {
        automan_version: "0.1.0".to_string(),
        dependencies: sample_entries(),
    };
    let json_text = serde_json::to_string_pretty(&lock).unwrap();
    assert_eq!(
        json_text,
        "{\n  \"automan_version\": \"0.1.0\",\n  \"dependencies\": [\n    {\n      \"name\": \"zeta-lib\",\n      \"version\": \"2.0.0\",\n      \"url\": \"https://example.com/zeta\",\n      \"commit\": \"c0ffee\",\n      \"path\": \"deps\\\\zeta-lib\"\n    },\n    {\n      \"name\": \"alpha-依赖\",\n      \"version\": \"1.2.3\",\n      \"url\": \"\",\n      \"commit\": \"\",\n      \"path\": \"D:/tmp dir/依赖库\"\n    }\n  ]\n}"
    );
    // No trailing newline (frozen serde_json::to_string_pretty behavior).
    assert!(!json_text.ends_with('\n'));
}

#[test]
fn v02_empty_lock_frozen_in_both_formats() {
    let lock = LockFile::new("0.1.0");
    assert_eq!(
        toml::to_string_pretty(&lock).unwrap(),
        "automan_version = \"0.1.0\"\ndependencies = []\n"
    );
    assert_eq!(
        serde_json::to_string_pretty(&lock).unwrap(),
        "{\n  \"automan_version\": \"0.1.0\",\n  \"dependencies\": []\n}"
    );
}

#[test]
fn v02_new_stamps_caller_version_never_core_version() {
    let lock = LockFile::new("7.7.7-man");
    assert_eq!(lock.automan_version, "7.7.7-man");
    assert_ne!(lock.automan_version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn v02_save_load_roundtrip_preserves_unicode_and_windows_paths() {
    let dir = tempfile::tempdir().unwrap();
    let lock = LockFile::from_dependencies("0.1.0", sample_entries());

    for file in ["pac.lock", "pac.lock.json"] {
        let path = dir.path().join(file);
        lock.save(&ap(&path)).unwrap();
        let loaded = LockFile::load(&ap(&path)).unwrap();
        assert_eq!(loaded.automan_version, "0.1.0");
        assert_eq!(loaded.dependencies.len(), 2);
        // Sorted by name on construction; round-trip preserves order.
        assert_eq!(loaded.dependencies[0].name, "alpha-依赖");
        assert_eq!(loaded.dependencies[0].path, "D:/tmp dir/依赖库");
        assert_eq!(loaded.dependencies[1].name, "zeta-lib");
        assert_eq!(loaded.dependencies[1].path, "deps\\zeta-lib");
    }
}

#[test]
fn v02_json_selection_on_unified_path_string() {
    // Backslash input unifies to '/' before the suffix check → JSON format.
    let dir = tempfile::tempdir().unwrap();
    let backslash_json = dir.path().join("sub");
    std::fs::create_dir_all(&backslash_json).unwrap();
    let path = AutoPath::new(
        backslash_json
            .join("pac.lock.json")
            .to_string_lossy()
            .to_string()
            .as_str(),
    );
    assert_eq!(
        path.to_string(),
        backslash_json
            .join("pac.lock.json")
            .to_string_lossy()
            .replace('\\', "/")
    );
    let lock = LockFile::new("0.1.0");
    lock.save(&path).unwrap();
    let text = std::fs::read_to_string(backslash_json.join("pac.lock.json")).unwrap();
    assert!(text.starts_with('{'), "expected JSON, got: {text}");
}

#[test]
fn v03_load_error_stages_are_frozen() {
    // Read stage: unified path in the message, OS error text follows.
    let missing = AutoPath::new("Z:/definitely/missing/pac.lock");
    let err = LockFile::load(&missing).unwrap_err();
    assert!(
        err.starts_with("Failed to read lock file 'Z:/definitely/missing/pac.lock': "),
        "got: {err}"
    );

    let dir = tempfile::tempdir().unwrap();

    // TOML parse stage (non-.json extension).
    let bad_toml = dir.path().join("bad.lock");
    std::fs::write(&bad_toml, "not [valid toml").unwrap();
    let err = LockFile::load(&ap(&bad_toml)).unwrap_err();
    assert!(
        err.starts_with("Failed to parse lock file TOML: "),
        "got: {err}"
    );

    // JSON parse stage (.json extension).
    let bad_json = dir.path().join("bad.lock.json");
    std::fs::write(&bad_json, "{ nope").unwrap();
    let err = LockFile::load(&ap(&bad_json)).unwrap_err();
    assert!(
        err.starts_with("Failed to parse lock file JSON: "),
        "got: {err}"
    );
}

#[test]
fn v03_verify_path_existence_negative_and_positive_controls() {
    let dir = tempfile::tempdir().unwrap();
    let real_dir = dir.path().join("real");
    std::fs::create_dir_all(&real_dir).unwrap();
    let not_a_dir = dir.path().join("file.txt");
    std::fs::write(&not_a_dir, "x").unwrap();
    let never = dir.path().join("never");

    let lock = LockFile::from_dependencies(
        "0.1.0",
        vec![
            LockEntry {
                name: "real-dep".to_string(),
                version: "1.0.0".to_string(),
                url: String::new(),
                commit: String::new(),
                path: real_dir.to_string_lossy().to_string(),
            },
            LockEntry {
                name: "file-dep".to_string(),
                version: "1.0.0".to_string(),
                url: String::new(),
                commit: String::new(),
                path: not_a_dir.to_string_lossy().to_string(),
            },
            LockEntry {
                name: "gone-dep".to_string(),
                version: "1.0.0".to_string(),
                url: String::new(),
                commit: String::new(),
                path: never.to_string_lossy().to_string(),
            },
        ],
    );

    let ok = ResolvedDependency::new("real-dep", "1.0.0");
    assert_eq!(
        auto_package_lock::verify_dependencies(&lock.dependencies, &[ok]),
        Ok(())
    );

    // A file (not a directory) is not a materialized dependency path.
    let err = auto_package_lock::verify_dependencies(
        &lock.dependencies,
        &[ResolvedDependency::new("file-dep", "1.0.0")],
    )
    .unwrap_err();
    assert_eq!(
        err,
        format!(
            "Dependency 'file-dep' path '{}' is not materialized",
            not_a_dir.to_string_lossy()
        )
    );

    // Missing path: same arm (existence check is directory-based).
    let err = auto_package_lock::verify_dependencies(
        &lock.dependencies,
        &[ResolvedDependency::new("gone-dep", "1.0.0")],
    )
    .unwrap_err();
    assert_eq!(
        err,
        format!(
            "Dependency 'gone-dep' path '{}' is not materialized",
            never.to_string_lossy()
        )
    );

    // Path existence is NOT a commit verification: a pinned commit skips
    // the path check entirely, even for a missing path.
    let pinned = LockEntry {
        name: "pinned".to_string(),
        version: "1.0.0".to_string(),
        url: String::new(),
        commit: "abc123".to_string(),
        path: never.to_string_lossy().to_string(),
    };
    assert_eq!(
        auto_package_lock::verify_dependencies(
            &[pinned],
            &[ResolvedDependency::new("pinned", "1.0.0")]
        ),
        Ok(())
    );
}

#[test]
fn v03_verify_aggregates_errors_in_resolved_order() {
    let lock = LockFile::from_dependencies(
        "0.1.0",
        vec![LockEntry {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            url: String::new(),
            commit: "c1".to_string(),
            path: String::new(),
        }],
    );
    let err = auto_package_lock::verify_dependencies(
        &lock.dependencies,
        &[
            ResolvedDependency::new("a", "9.9.9"),
            ResolvedDependency::new("ghost", "1.0.0"),
        ],
    )
    .unwrap_err();
    assert_eq!(
        err,
        "Dependency 'a' version mismatch: lockfile has 1.0.0, but resolved 9.9.9\n\
         Dependency 'ghost' not found in lock file"
    );
}

#[test]
fn v02_duplicate_names_keep_first_match_semantics() {
    let lock = LockFile::from_dependencies(
        "0.1.0",
        vec![
            LockEntry {
                name: "dup".to_string(),
                version: "1.0.0".to_string(),
                url: String::new(),
                commit: "c1".to_string(),
                path: String::new(),
            },
            LockEntry {
                name: "dup".to_string(),
                version: "2.0.0".to_string(),
                url: String::new(),
                commit: "c2".to_string(),
                path: String::new(),
            },
        ],
    );
    // Both entries are kept (historical behavior), stable order among
    // equals, and lookup/verification see the first one.
    assert_eq!(lock.dependencies.len(), 2);
    assert_eq!(lock.get("dup").unwrap().version, "1.0.0");
    assert_eq!(
        find_entry(&lock.dependencies, "dup").unwrap().version,
        "1.0.0"
    );
    assert!(auto_package_lock::verify_dependencies(
        &lock.dependencies,
        &[ResolvedDependency::new("dup", "1.0.0")]
    )
    .is_ok());
    assert!(auto_package_lock::verify_dependencies(
        &lock.dependencies,
        &[ResolvedDependency::new("dup", "2.0.0")]
    )
    .is_err());
}

#[test]
fn v03_save_failure_does_not_write_new_data() {
    // Serialization happens before any directory creation or write: pointing
    // the lock at a path whose PARENT is a regular file makes the later
    // create_dir_all fail — the file content written by a prior successful
    // save must remain untouched by the failed attempt, and no partial new
    // file may appear. (The serialize-first order itself is structural; this
    // test pins the observable no-partial-write/no-clobber behavior.)
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, "keep me").unwrap();
    let target = blocker.join("pac.lock");

    let lock = LockFile::new("0.1.0");
    let err = lock.save(&ap(&target)).unwrap_err();
    assert!(
        err.starts_with("Failed to create lock file directory: "),
        "got: {err}"
    );
    assert_eq!(std::fs::read(&blocker).unwrap(), b"keep me");
    assert!(!target.exists());
}
