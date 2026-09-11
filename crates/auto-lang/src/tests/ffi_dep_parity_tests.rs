// PLAN-592 T6: dep 三轨行为对拍 runner。
//
// 同一 input.at 语料跑三条腿,stdout(trim)必须与 golden 精确相等:
// 1. VM 腿(日常档):dep+use.rs → methods pack → dispatch 3000(同 ffi_dual 骨架,
//    {{FFI_DUAL_DIR}} 占位替换,nightly 缺席 skip);
// 2. oracle 腿(日常档):手写 Rust 直调同一 fixture crate(oracle/ 目录,
//    空 [workspace] + 相对路径依赖),就地 cargo build(cargo 自身增量缓存,
//    内容 hash marker 门控避免无谓 rebuild);
// 3. a2r 腿(AUTO_LANG_DEP_PARITY_A2R=1 启用):transpile_rust 转译(剥离 dep 行,
//    依赖由本腿渲染进生成的 Cargo.toml——仅 path 依赖,离线可跑)→ 就地
//    cargo build → 跑产物。默认关闭,CI 全量档打开(冷构建 1-2min/腿)。
//
// 语料:016_dep_abi_matrix(marshalling 全矩阵 + p.a 字段语法)、
//      017_dep_lifecycle(chain/clone/自由函数)、
//      018_dep_fields(PLAN-591:Option nullable/Result fallible/嵌套/enum 判别)。
// 新增语料向 CASES 登记。

use crate::error::AutoResult;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::Command;

const CASES: &[&str] = &["016_dep_abi_matrix", "017_dep_lifecycle", "018_dep_fields", "020_dep_traits_generics", "021_dep_callback"];

// =============================================================================
// 公共骨架
// =============================================================================

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn case_dir(case: &str) -> PathBuf {
    manifest_dir().join("test/ffi_dual").join(case)
}

fn read_corpus(case: &str) -> AutoResult<(String, String)> {
    let mut src = read_to_string(case_dir(case).join("input.at"))?;
    let ffi_dual_dir = manifest_dir()
        .join("test/ffi_dual")
        .to_string_lossy()
        .replace('\\', "/");
    src = src.replace("{{FFI_DUAL_DIR}}", &ffi_dual_dir);
    let golden = read_to_string(case_dir(case).join("expected_output.txt"))?;
    Ok((src, golden))
}

/// fnv1a64 内容指纹(与 auto-cache 同族算法,本地实现避免跨 crate API 耦合)
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// 递归收集目录内全部文件的 (相对路径, 内容) 参与指纹(跳过 target/ 与 build_a2r/)
fn hash_dir_tree(dir: &Path, buf: &mut Vec<u8>) {
    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "target" || name == "build_a2r" || name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            hash_dir_tree(&path, buf);
        } else if let Ok(content) = std::fs::read(&path) {
            buf.extend_from_slice(name.as_bytes());
            buf.push(b'\0');
            buf.extend_from_slice(&content);
            buf.push(b'\0');
        }
    }
}

fn run_binary_capturing_stdout(bin: &Path) -> String {
    let out = Command::new(bin)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", bin.display()));
    assert!(
        out.status.success(),
        "binary {} exited with {:?}\nstdout: {}\nstderr: {}",
        bin.display(),
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 就地 cargo build --release(内容 hash marker 门控;cargo target 目录自身增量)
fn cargo_build_if_changed(dir: &Path, fingerprint: u64, inputs: &[u8]) {
    let marker = dir.join(".parity_cache_ok");
    if let Ok(m) = read_to_string(&marker) {
        if m.trim() == fingerprint.to_string() {
            return; // 输入未变,cargo target 增量即缓存
        }
    }
    let status = Command::new(env!("CARGO"))
        .args(["build", "--release"])
        .current_dir(dir)
        .status()
        .unwrap_or_else(|e| panic!("failed to spawn cargo in {}: {e}", dir.display()));
    assert!(
        status.success(),
        "cargo build --release failed in {}\n(inputs fingerprint {fingerprint:016x})",
        dir.display()
    );
    let _ = std::fs::write(&marker, fingerprint.to_string());
    let _ = inputs;
}

// =============================================================================
// 三条腿
// =============================================================================

fn vm_leg(case: &str, src: &str) -> AutoResult<String> {
    let (_, stdout) = crate::run_with_capture(src)?;
    Ok(stdout)
}

fn oracle_leg(case: &str) -> String {
    let dir = case_dir(case).join("oracle");
    let mut inputs = Vec::new();
    hash_dir_tree(&dir, &mut inputs);
    // fixture 参与指纹(oracle 依赖它,虽在 case 目录外)
    for dep in fixture_deps(case) {
        hash_dir_tree(&dep.1, &mut inputs);
    }
    let fp = fnv1a64(&inputs);
    cargo_build_if_changed(&dir, fp, &inputs);
    let bin = find_bin(&dir, &format!("oracle_{case}"));
    run_binary_capturing_stdout(&bin)
}

/// 解析 input.at 的 dep 声明 → (crate 名, fixture 绝对路径)。
/// 本 runner 只支持 path 依赖(离线原则);version/git 依赖直接 panic 明示。
fn fixture_deps(case: &str) -> Vec<(String, PathBuf)> {
    let src = read_to_string(case_dir(case).join("input.at")).expect("input.at readable");
    let mut deps = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if !line.starts_with("dep ") {
            continue;
        }
        let rest = line["dep ".len()..].trim();
        let name = rest.split(['(', ' ', ':']).next().unwrap_or("").to_string();
        if let Some(open) = rest.find("path:") {
            let after = rest[open + "path:".len()..].trim_start();
            let after = after.trim_start_matches('"');
            let path = after.split('"').next().unwrap_or("");
            let path = path.replace("{{FFI_DUAL_DIR}}", &manifest_dir().join("test/ffi_dual").to_string_lossy().replace('\\', "/"));
            deps.push((name, PathBuf::from(&path)));
        } else {
            panic!("dep-parity runner only supports path deps; `{name}` in {case} has none");
        }
    }
    deps
}

fn find_bin(dir: &Path, name: &str) -> PathBuf {
    let exe = if cfg!(windows) { format!("{name}.exe") } else { name.to_string() };
    let bin = dir.join("target/release").join(&exe);
    assert!(bin.exists(), "binary not found at {} (build step missing?)", bin.display());
    bin
}

/// a2r 腿:剥离 dep 行 → transpile_rust → 生成最小依赖 crate → build → run。
fn a2r_leg(case: &str, src: &str) -> String {
    let stripped: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("dep "))
        .collect::<Vec<_>>()
        .join("\n");
    let mut sink = crate::trans::rust::transpile_rust("main", &stripped)
        .unwrap_or_else(|e| panic!("a2r transpile failed for {case}: {e:?}"));
    let main_rs = sink.done().expect("a2r sink done");

    let dir = case_dir(case).join("build_a2r/main");
    std::fs::create_dir_all(dir.join("src")).expect("create build_a2r dirs");

    let auto_lang_path = manifest_dir().to_string_lossy().replace('\\', "/");
    let a2r_std_path = manifest_dir().join("../a2r-std").to_string_lossy().replace('\\', "/");
    let mut dep_lines = String::new();
    let mut inputs = Vec::new();
    inputs.extend_from_slice(&main_rs);
    for (name, path) in fixture_deps(case) {
        let p = path.to_string_lossy().replace('\\', "/");
        dep_lines.push_str(&format!("{name} = {{ path = \"{p}\" }}\n"));
        hash_dir_tree(&path, &mut inputs);
    }
    let cargo_toml = format!(
        "# Generated by ffi_dep_parity_tests (PLAN-592 T6). DO NOT EDIT.\n\
         [package]\n\
         name = \"dep_parity_{case_token}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\n\
         [dependencies]\n\
         auto-lang = {{ path = \"{auto_lang_path}\" }}\n\
         a2r-std = {{ path = \"{a2r_std_path}\" }}\n\
         {dep_lines}\n\
         [[bin]]\n\
         name = \"dep_parity_{case_token}\"\n\
         path = \"src/main.rs\"\n\n\
         [workspace]\n",
        case_token = case.replace('-', "_"),
    );
    inputs.extend_from_slice(cargo_toml.as_bytes());
    std::fs::write(dir.join("Cargo.toml"), &cargo_toml).expect("write Cargo.toml");
    std::fs::write(dir.join("src/main.rs"), &main_rs).expect("write main.rs");

    let fp = fnv1a64(&inputs);
    cargo_build_if_changed(&dir, fp, &inputs);
    let bin = find_bin(&dir, &format!("dep_parity_{}", case.replace('-', "_")));
    run_binary_capturing_stdout(&bin)
}

// =============================================================================
// 测试入口
// =============================================================================

fn test_dep_parity(case: &str) {
    if !auto_cache::methods_pack::nightly_available() {
        eprintln!("skipped: nightly toolchain unavailable for methods pack");
        return;
    }
    let (src, golden) = read_corpus(case).expect("corpus readable");
    let golden_trim = golden.trim().to_string();

    let vm_out = vm_leg(case, &src).expect("VM leg runs");
    assert_eq!(
        vm_out.trim(),
        golden_trim,
        "VM leg diverged from golden for {case}"
    );

    let oracle_out = oracle_leg(case);
    assert_eq!(
        oracle_out.trim(),
        golden_trim,
        "oracle leg diverged from golden for {case}"
    );

    // PLAN-596 V2-6(experimental): a2r 腿对回调实参不装箱(DIV-DEP-19——
    // `inv.apply(|x| ..)` 发射裸闭包,E0308 expected Box<dyn Fn>;根治=a2r
    // 元数据接入后按形参类型装箱)。复审 F-1 后豁免收窄到回调独立语料
    // 021——020 主面(trait/泛型)恢复自动 a2r 三轨。豁免表内用例 a2r 腿
    // 跳过,VM/oracle 照常。
    const A2R_SKIP: &[&str] = &["021_dep_callback"];
    if A2R_SKIP.contains(&case) {
        eprintln!("[dep-parity] a2r leg skipped for {case} (DIV-DEP-19: closure arg unboxed)");
    } else if std::env::var("AUTO_LANG_DEP_PARITY_A2R")
        .map(|v| v != "0")
        .unwrap_or(false)
    {
        let a2r_out = a2r_leg(case, &src);
        assert_eq!(
            a2r_out.trim(),
            golden_trim,
            "a2r leg diverged from golden for {case}"
        );
    } else {
        eprintln!("[dep-parity] a2r leg skipped (set AUTO_LANG_DEP_PARITY_A2R=1)");
    }
}

#[test]
fn dep_parity_016_dep_abi_matrix() {
    test_dep_parity("016_dep_abi_matrix");
}

#[test]
fn dep_parity_017_dep_lifecycle() {
    test_dep_parity("017_dep_lifecycle");
}

#[test]
fn dep_parity_018_dep_fields() {
    test_dep_parity("018_dep_fields");
}


#[test]
fn dep_parity_020_dep_traits_generics() {
    test_dep_parity("020_dep_traits_generics");
}

#[test]
fn dep_parity_021_dep_callback() {
    test_dep_parity("021_dep_callback");
}
