//! Compile and execute unmodified production Rust, not a helper-only oracle.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn newest_rlib(deps: &Path, prefix: &str) -> PathBuf {
    fs::read_dir(deps)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(prefix)
                && path
                    .extension()
                    .is_some_and(|extension| extension == "rlib")
        })
        .max_by_key(|path| fs::metadata(path).unwrap().modified().unwrap())
        .expect("Cargo must build the current core dependency before this consumer test")
}

fn rustc_flags() -> Vec<String> {
    let deps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let mut flags = vec![
        "--edition=2021".into(),
        "-L".into(),
        format!("dependency={}", deps.display()),
        "--extern".into(),
        format!(
            "auto_lang_core={}",
            newest_rlib(&deps, "libauto_lang_core-").display()
        ),
    ];
    for entry in fs::read_dir(deps.parent().unwrap().join("build"))
        .unwrap()
        .flatten()
    {
        let native = entry.path().join("out");
        if native.is_dir() {
            flags.extend(["-L".into(), format!("native={}", native.display())]);
        }
    }
    if cfg!(target_env = "msvc") {
        let cargo_home = std::env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("USERPROFILE").unwrap()).join(".cargo")
            });
        for registry in fs::read_dir(cargo_home.join("registry/src"))
            .unwrap()
            .flatten()
        {
            for entry in fs::read_dir(registry.path()).unwrap().flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("windows_x86_64_msvc-")
                {
                    flags.extend([
                        "-L".into(),
                        format!("native={}", entry.path().join("lib").display()),
                    ]);
                }
            }
        }
    }
    flags
}

#[test]
fn plan988_source_bytes_compile_run_and_preserve_rejected_targets() {
    let temp = if let Some(parent) = std::env::var_os("PLAN988_TEST_OUT") {
        fs::create_dir_all(&parent).unwrap();
        tempfile::tempdir_in(parent).unwrap()
    } else {
        tempfile::tempdir().unwrap()
    };
    let flags = rustc_flags();
    let bytes = [0, 127, 128, 255];
    let sentinel = [42, 128, 255];
    // (name, source body / whole program, expected bytes, VM must reject).
    let mut cases: Vec<(&str, String, Vec<u8>, bool)> = vec![
        ("typed", "let bs = [0,127,128,255]\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)".into(), bytes.to_vec(), false),
        ("empty_direct", "let rc = File.write_bytes(PATH, [])\nprint(rc)".into(), vec![], false),
        ("empty_local", "let bs = []\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)".into(), vec![], false),
        ("empty_branch", "var bs = []\nif true { bs = [] }\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)".into(), vec![], false),
        ("empty_return", "fn bytes() []int { return [] }\nfn main() { let rc = File.write_bytes(PATH, bytes())\nprint(rc) }".into(), vec![], false),
        ("parameter", "fn save(p str, bs []int) int { return File.write_bytes(p, bs) }\nfn main() { let bs = [0,127,128,255]\nlet rc = save(PATH, bs)\nprint(rc) }".into(), bytes.to_vec(), false),
        ("empty_parameter", "fn save(p str, bs []int) int { return File.write_bytes(p, bs) }\nfn main() { let bs = []\nlet rc = save(PATH, bs)\nprint(rc) }".into(), vec![], false),
        ("forwarded_parameter", "fn save(p str, bs []int) int { return File.write_bytes(p, bs) }\nfn relay(p str, bs []int) int { return save(p, bs) }\nfn main() { let rc = relay(PATH, [0,127,128,255])\nprint(rc) }".into(), bytes.to_vec(), false),
        ("read_control", "let bs = File.read_bytes(INPUT)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)".into(), bytes.to_vec(), false),
        ("json_inline", "let rc = File.write_bytes(PATH, json.parse(\"[0,127,128,255]\"))\nprint(rc)".into(), bytes.to_vec(), false),
        ("json_nested", "let obj = json.parse(\"{\\\"payload\\\":[0,127,128,255]}\")\nlet rc = File.write_bytes(PATH, obj.payload)\nprint(rc)".into(), bytes.to_vec(), false),
        ("json_bytes_field", "let obj = json.parse(\"{\\\"bytes\\\":[0,127,128,255]}\")\nlet rc = File.write_bytes(PATH, obj.bytes)\nprint(rc)".into(), bytes.to_vec(), false),
        ("typed_reused", "let p = PATH\nlet bs = [0,127,128,255]\nlet first = File.write_bytes(p, bs)\nlet rc = File.write_bytes(p, bs)\nprint(rc)".into(), bytes.to_vec(), false),
    ];
    cases.push(("empty_reused", "let p = PATH\nlet bs = []\nlet first = File.write_bytes(p, bs)\nlet rc = File.write_bytes(p, bs)\nprint(rc)".into(), vec![], false));
    cases.push(("json_reused", "let p = PATH\nlet bs = json.parse(\"[0,127,128,255]\")\nlet first = File.write_bytes(p, bs)\nlet rc = File.write_bytes(p, bs)\nprint(rc)".into(), bytes.to_vec(), false));
    for (name, json, expected, reject) in [
        ("json_list", "[0,127,128,255]", bytes.to_vec(), false),
        ("json_empty", "[]", vec![], false),
        ("json_bool", "[0,true,255]", sentinel.to_vec(), true),
        ("json_float", "[0,1.5,255]", sentinel.to_vec(), true),
        ("json_string", "[0,\"128\",255]", sentinel.to_vec(), true),
        ("json_nested_list", "[0,[128],255]", sentinel.to_vec(), true),
        ("json_null", "null", sentinel.to_vec(), true),
        ("json_scalar", "12", sentinel.to_vec(), true),
        ("json_object", "{}", sentinel.to_vec(), true),
    ] {
        cases.push((
            name,
            format!(
                "let bs = json.parse({})\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)",
                serde_json::to_string(json).unwrap()
            ),
            expected,
            reject,
        ));
    }
    for (name, source, expected, reject) in cases {
        let dir = temp.path().join(name);
        fs::create_dir(&dir).unwrap();
        let target = dir.join("bytes.bin");
        let input = dir.join("seed.bin");
        fs::write(&input, bytes).unwrap();
        let source = source
            .replace(
                "PATH",
                &serde_json::to_string(&target.to_string_lossy().replace('\\', "/")).unwrap(),
            )
            .replace(
                "INPUT",
                &serde_json::to_string(&input.to_string_lossy().replace('\\', "/")).unwrap(),
            );
        let source = if source.starts_with("fn ") {
            source
        } else {
            format!("fn main() {{\n{source}\n}}")
        };
        fs::write(dir.join("input.at"), &source).unwrap();
        fs::write(&target, sentinel).unwrap();
        let vm = auto_lang_core::run_autovm_capture(&source);
        assert_eq!(vm.is_err(), reject, "{name}: {vm:?}");
        if let Ok(output) = vm {
            assert_eq!(output.1.trim(), "0", "{name}: VM status");
        }
        assert_eq!(fs::read(&target).unwrap(), expected, "{name}: VM bytes");

        let mut sink = auto_lang_core::trans::rust::transpile_rust(name, &source).unwrap();
        let generated = String::from_utf8(sink.done().unwrap().clone()).unwrap();
        let generated = auto_generation_core::shared_generation::qualify_a2r_std(generated);
        let rust = dir.join("main.rs");
        fs::write(&rust, generated).unwrap();
        let executable = dir.join(if cfg!(windows) {
            "consumer.exe"
        } else {
            "consumer"
        });
        let compilation = Command::new("rustc")
            .arg(&rust)
            .args(&flags)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        fs::write(dir.join("compile.log"), &compilation.stderr).unwrap();
        assert!(
            compilation.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        fs::write(&target, sentinel).unwrap();
        let result = Command::new(&executable).output().unwrap();
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&result.stdout).trim(),
            if reject { "-1" } else { "0" },
            "{name}: Rust status"
        );
        assert_eq!(fs::read(&target).unwrap(), expected, "{name}: Rust bytes");
        eprintln!("PASS {name}: VM + generated Rust, status and bytes");
    }
}
