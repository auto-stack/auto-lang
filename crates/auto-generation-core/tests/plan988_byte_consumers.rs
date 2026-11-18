//! Compile and execute unmodified production Rust, not a helper-only oracle.
//!
//! PLAN-983 r23（2026-11-19，用户指令「四仓门禁修复」）：原单测
//! `plan988_source_bytes_compile_run_and_preserve_rejected_targets` 以一个
//! 循环串行执行全部 50 场景（每场景一次真实 `rustc` 编译+运行），整机
//! 105–131s，压过四仓门 `.config/nextest.toml` 的挂起保护上限（60s 警告/
//! 120s 终止）——本文件按**语义族**拆成五个独立 `#[test]`（写路径/push
//! 变异/替换与赋值/typed 上下文/json 值形态），场景表与执行体**逐字保留**
//! （名/源/期望字节/VM 拒绝位/VM+生成 Rust 双腿+字节对照+非法输入保旧），
//! 并保留 `PLAN988_BYTE_CASE` 子串过滤钩子做定向重放。完整性守卫测试保证
//! 族并集==全部场景（新场景漏挂族会红）。

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
    let core = newest_rlib(&deps, "libauto_lang_core-");
    // Match serde_json's Cargo fingerprint to the core being linked. Workspace
    // feature combinations may leave several incompatible crate identities.
    let core_hash = core
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .rsplit('-')
        .next()
        .unwrap();
    let fingerprints = deps.parent().unwrap().join(".fingerprint");
    let core_fp: serde_json::Value = serde_json::from_slice(
        &fs::read(fingerprints.join(format!(
            "auto-lang-core-{core_hash}/lib-auto_lang_core.json"
        )))
        .unwrap(),
    )
    .unwrap();
    let serde_fp = core_fp["deps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|dep| dep[1] == "serde_json")
        .unwrap()[3]
        .as_u64()
        .unwrap();
    let serde_fp = serde_fp
        .to_le_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let serde_hash = fs::read_dir(&fingerprints)
        .unwrap()
        .flatten()
        .find_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let hash = name.strip_prefix("serde_json-")?;
            (fs::read_to_string(entry.path().join("lib-serde_json"))
                .ok()?
                .trim()
                == serde_fp)
                .then(|| hash.to_owned())
        })
        .expect("core serde_json dependency fingerprint");
    let mut flags = vec![
        "--edition=2021".into(),
        "-L".into(),
        format!("dependency={}", deps.display()),
        "--extern".into(),
        format!("auto_lang_core={}", core.display()),
    ];
    flags.extend([
        "--extern".into(),
        format!(
            "serde_json={}",
            deps.join(format!("libserde_json-{serde_hash}.rlib"))
                .display()
        ),
    ]);
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

/// 全部场景（原单测的场景构造逐字搬移：名/源/期望字节/VM 拒绝位）。
fn byte_cases() -> Vec<(&'static str, String, Vec<u8>, bool)> {
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
    // Mutation is observed in both orders; FIRST preserves the initial write
    // as a separate file so the final result cannot hide a bad first phase.
    for (name, source, expected, reject) in [
        ("push_after_write", "var bs = []\nlet first = File.write_bytes(FIRST, bs)\nbs.push(255)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![255], false),
        ("push_before_write", "var bs = []\nbs.push(255)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![255], false),
        ("push_branch_loop", "var bs = []\nlet first = File.write_bytes(FIRST, bs)\nif true { for byte in [0,128,255] { bs.push(byte) } }\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![0,128,255], false),
        ("push_alias", "var bs = []\nvar copied = bs\ncopied.push(255)\nlet rc = File.write_bytes(PATH, copied)\nprint(rc)", vec![255], false),
        ("push_bad_value", "var bs = []\nlet first = File.write_bytes(FIRST, bs)\nlet value = json.parse(\"true\")\nbs.push(value)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", sentinel.to_vec(), true),
        ("push_typed_empty", "var bs []int = []\nlet first = File.write_bytes(FIRST, bs)\nbs.push(255)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![255], false),
        ("push_nonempty", "var bs = [0]\nbs.push(255)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![0,255], false),
        ("push_nested_local", "fn typed_count(items []int) int { return items.len() }\nfn main() { var bs = []\nlet first = File.write_bytes(FIRST, bs)\nif true { let other = []\nother.push(2)\nlet n = typed_count(other) }\nbs.push(255)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc) }", vec![255], false),
    ] { cases.push((name,source.into(),expected,reject)); }
    cases.push(("push_parameter", "fn save(p str, bs []int) int { return File.write_bytes(p, bs) }\nfn main() { var bs = []\nlet first = File.write_bytes(FIRST, bs)\nbs.push(255)\nlet rc = save(PATH, bs)\nprint(rc) }".into(), vec![255], false));
    cases.push((
        "push_condition",
        "var bs = []\nbs.push(255)\nif File.write_bytes(PATH, bs) == 0 { print(0) }".into(),
        vec![255],
        false,
    ));
    cases.push(("replace_array", "var bs = []\nlet first = File.write_bytes(FIRST, bs)\nbs = [0,128,255]\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)".into(), vec![0,128,255], false));
    cases.push(("replace_return", "fn make() []int { return [0,128,255] }\nfn main() { var bs = []\nlet first = File.write_bytes(FIRST, bs)\nbs = make()\nlet rc = File.write_bytes(PATH, bs)\nprint(rc) }".into(), vec![0,128,255], false));
    for (name, source, expected) in [
        ("assignment_empty", "var bs = []\nlet blank = []\nbs = blank\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![]),
        ("assignment_empty_branch", "var bs = []\nlet blank = []\nif true { bs = blank }\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![]),
        ("assignment_empty_chain", "var bs = []\nvar middle = []\nlet blank = []\nbs = middle\nmiddle = blank\nlet first = File.write_bytes(FIRST, bs)\nlet rc = File.write_bytes(PATH, middle)\nprint(rc)", vec![]),
        ("assignment_empty_cycle", "var bs = []\nvar blank = []\nbs = blank\nblank = bs\nbs = []\nlet first = File.write_bytes(FIRST, bs)\nlet rc = File.write_bytes(PATH, blank)\nprint(rc)", vec![]),
        ("assignment_filled_alias", "var bs = []\nlet filled = [0,255]\nbs = filled\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![0,255]),
        ("assignment_typed_empty", "var bs = []\nvar blank []int = []\nbs = blank\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![]),
        ("assignment_then_push", "var bs = []\nlet blank = []\nbs = blank\nlet first = File.write_bytes(FIRST, bs)\nbs.push(255)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", vec![255]),
        ("assignment_propagated_fill", "var bs = []\nvar middle = []\nlet filled = [255]\nbs = middle\nmiddle = filled\nlet first = File.write_bytes(FIRST, bs)\nlet rc = File.write_bytes(PATH, middle)\nprint(rc)", vec![255]),
    ] { cases.push((name,source.into(),expected,false)); }
    let save = "fn save(p str, bs []int) int { return File.write_bytes(p, bs) }";
    for (name, body, later) in [
        ("typed_context_after", "var bs = []\nlet first = File.write_bytes(FIRST, bs)\nlet rc = save(PATH, bs)\nprint(rc)", false),
        ("typed_context_before", "var bs = []\nlet first = save(FIRST, bs)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", false),
        ("typed_context_after_later", "var bs = []\nlet first = File.write_bytes(FIRST, bs)\nlet rc = save(PATH, bs)\nprint(rc)", true),
        ("typed_context_before_later", "var bs = []\nlet first = save(FIRST, bs)\nlet rc = File.write_bytes(PATH, bs)\nprint(rc)", true),
        ("typed_context_assignment", "var bs = []\nlet blank = []\nbs = blank\nlet first = File.write_bytes(FIRST, bs)\nlet rc = save(PATH, bs)\nprint(rc)", false),
        ("typed_context_alias", "var bs = []\nlet first = File.write_bytes(FIRST, bs)\nlet forwarded = bs\nlet rc = save(PATH, forwarded)\nprint(rc)", false),
    ] {
        let main = format!("fn main() {{\n{body}\n}}");
        let source = if later { format!("{main}\n{save}") } else { format!("{save}\n{main}") };
        cases.push((name, source, vec![], false));
    }
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
    cases
}

/// 单场景执行体（原单测循环体逐字搬移：VM 断言 + 生成 Rust 真编译运行 +
/// 双腿字节对照 + FIRST 保旧 + 非法输入 sentinel 保旧）。
fn run_byte_case(
    temp: &Path,
    flags: &[String],
    bytes: [u8; 4],
    sentinel: [u8; 3],
    name: &str,
    source: &str,
    expected: &[u8],
    reject: bool,
) {
    let dir = temp.join(name);
    fs::create_dir(&dir).unwrap();
    let target = dir.join("bytes.bin");
    let input = dir.join("seed.bin");
    let first = dir.join("first.bin");
    let check_first = source.contains("FIRST");
    fs::write(&input, bytes).unwrap();
    let source = source
        .replace(
            "FIRST",
            &serde_json::to_string(&first.to_string_lossy().replace('\\', "/")).unwrap(),
        )
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

    if check_first {
        assert_eq!(
            fs::read(&first).unwrap(),
            Vec::<u8>::new(),
            "{name}: first VM write"
        );
        fs::remove_file(&first).unwrap();
    }

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
        .args(flags)
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
    if check_first {
        assert_eq!(
            fs::read(&first).unwrap(),
            Vec::<u8>::new(),
            "{name}: first Rust write"
        );
    }
    eprintln!("PASS {name}: VM + generated Rust, status and bytes");
}

/// 语义族执行（PLAN-983 r23 拆分）：按精确名选场景；保留
/// `PLAN988_BYTE_CASE` 子串过滤钩子做族内定向重放。
fn run_family(family_names: &[&str]) {
    let temp = if let Some(parent) = std::env::var_os("PLAN988_TEST_OUT") {
        fs::create_dir_all(&parent).unwrap();
        tempfile::tempdir_in(parent).unwrap()
    } else {
        tempfile::tempdir().unwrap()
    };
    let flags = rustc_flags();
    let cases = byte_cases();
    for wanted in family_names {
        assert!(
            cases.iter().any(|(name, _, _, _)| name == wanted),
            "unknown byte case {wanted:?} (family list typo)"
        );
    }
    let filter = std::env::var("PLAN988_BYTE_CASE").unwrap_or_default();
    let mut selected = 0;
    for (name, source, expected, reject) in cases
        .into_iter()
        .filter(|(name, _, _, _)| family_names.contains(name) && name.contains(&filter))
    {
        run_byte_case(
            temp.path(),
            &flags,
            [0, 127, 128, 255],
            [42, 128, 255],
            name,
            &source,
            &expected,
            reject,
        );
        selected += 1;
    }
    assert!(selected > 0, "zero selected byte cases in this family");
}

const WRITE_PATHS: &[&str] = &[
    "typed",
    "empty_direct",
    "empty_local",
    "empty_branch",
    "empty_return",
    "parameter",
    "empty_parameter",
    "forwarded_parameter",
    "read_control",
    "json_inline",
    "json_nested",
    "json_bytes_field",
    "typed_reused",
    "empty_reused",
    "json_reused",
];

const PUSH_MUTATION: &[&str] = &[
    "push_after_write",
    "push_before_write",
    "push_branch_loop",
    "push_alias",
    "push_bad_value",
    "push_typed_empty",
    "push_nonempty",
    "push_nested_local",
    "push_parameter",
    "push_condition",
];

const REPLACE_ASSIGNMENT: &[&str] = &[
    "replace_array",
    "replace_return",
    "assignment_empty",
    "assignment_empty_branch",
    "assignment_empty_chain",
    "assignment_empty_cycle",
    "assignment_filled_alias",
    "assignment_typed_empty",
    "assignment_then_push",
    "assignment_propagated_fill",
];

const TYPED_CONTEXT: &[&str] = &[
    "typed_context_after",
    "typed_context_before",
    "typed_context_after_later",
    "typed_context_before_later",
    "typed_context_assignment",
    "typed_context_alias",
];

const JSON_VALUES: &[&str] = &[
    "json_list",
    "json_empty",
    "json_bool",
    "json_float",
    "json_string",
    "json_nested_list",
    "json_null",
    "json_scalar",
    "json_object",
];

/// 完整性守卫：族并集必须等于全部场景（新场景漏挂族=红；重复挂=红）。
#[test]
fn plan988_bytes_family_partition_is_complete() {
    let families: &[&[&str]] = &[
        WRITE_PATHS,
        PUSH_MUTATION,
        REPLACE_ASSIGNMENT,
        TYPED_CONTEXT,
        JSON_VALUES,
    ];
    let mut listed: Vec<&str> = families.iter().flat_map(|f| f.iter().copied()).collect();
    let total = listed.len();
    listed.sort_unstable();
    listed.dedup();
    assert_eq!(listed.len(), total, "a byte case is listed in two families");
    let mut all: Vec<&str> = byte_cases()
        .into_iter()
        .map(|(name, _, _, _)| name)
        .collect();
    all.sort_unstable();
    let mut listed_sorted = listed;
    assert_eq!(
        listed_sorted, all,
        "every byte case must belong to exactly one semantic family"
    );
}

#[test]
fn plan988_bytes_write_paths() {
    run_family(WRITE_PATHS);
}

#[test]
fn plan988_bytes_push_mutation() {
    run_family(PUSH_MUTATION);
}

#[test]
fn plan988_bytes_replace_assignment() {
    run_family(REPLACE_ASSIGNMENT);
}

#[test]
fn plan988_bytes_typed_context() {
    run_family(TYPED_CONTEXT);
}

#[test]
fn plan988_bytes_json_values() {
    run_family(JSON_VALUES);
}
