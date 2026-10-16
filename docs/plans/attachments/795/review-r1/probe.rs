use auto_package_lock::{LockEntry, LockFile, ResolvedDependency, verify_dependencies};
use auto_val::AutoPath;

fn main() {
    let root = std::env::temp_dir().join(format!("review795-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let missing = root.join("missing").to_string_lossy().into_owned();
    let entry = LockEntry { name: "dup".into(), version: "1".into(), url: "".into(), commit: "".into(), path: missing.clone() };
    let mut second = entry.clone(); second.version = "2".into(); second.commit = "pinned".into();
    let lock = LockFile::from_dependencies("caller", vec![entry, second]);
    let resolved = [ResolvedDependency::new("dup", "2"), ResolvedDependency::new("ghost", "1")];
    let expected = format!("Dependency 'dup' version mismatch: lockfile has 1, but resolved 2\nDependency 'dup' path '{}' is not materialized\nDependency 'ghost' not found in lock file", missing);
    assert_eq!(verify_dependencies(&lock.dependencies, &resolved), Err(expected));
    std::fs::create_dir_all(&missing).unwrap();
    assert_eq!(verify_dependencies(&lock.dependencies, &[ResolvedDependency::new("dup", "1")]), Ok(()));
    println!("R01 PASS: duplicate first entry + version/path/missing combined ordering + materialized positive control");
    for suffix in ["pac.lock", "pac.lock.json", "pac.lock.JSON"] {
        let file = root.join(suffix);
        let path = AutoPath::new(file.to_string_lossy().as_ref());
        lock.save(&path).unwrap();
        assert_eq!(LockFile::load(&path).unwrap(), lock);
        let text = std::fs::read_to_string(&file).unwrap();
        assert_eq!(text.starts_with('{'), suffix.ends_with(".json"));
    }
    println!("R02 PASS: duplicate entries roundtrip TOML/JSON/uppercase fallback, caller version retained");
    let directory = AutoPath::new(root.to_string_lossy().as_ref());
    let err = lock.save(&directory).unwrap_err();
    assert!(err.starts_with("Failed to write lock file '"), "{err}");
    assert!(root.join("pac.lock").is_file());
    println!("R03 PASS: write-stage failure distinguished from directory-creation failure, existing output retained");
    for (name, text, prefix) in [
        ("bad.json", "{\"automan_version\":\"v\",\"dependencies\":[{\"name\":\"a\"}]}", "Failed to parse lock file JSON: "),
        ("bad.toml", "automan_version = 'v'\n[[dependencies]]\nname = 'a'\n", "Failed to parse lock file TOML: "),
    ] {
        let file = root.join(name);
        std::fs::write(&file, text).unwrap();
        let err = LockFile::load(&AutoPath::new(file.to_string_lossy().as_ref())).unwrap_err();
        assert!(err.starts_with(prefix), "{err}");
        assert!(err.contains("missing field"), "{err}");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), text);
    }
    println!("R04 PASS: incomplete serde shape rejected at parse stage without mutating source");
}
