// Plan 568: AAVM/AA2R runner 测试——自 vm_file_tests.rs 整体迁入(原
// test-vm-files 门内嵌 aavm 腿;现独立 test-aavm 档,见 tests.rs 注册与
// AGENTS.md 触发条件/作用域映射)。内容:AAVM v1 runner(lib-legacy)+
// AAVM v2 基建(runner/compile 腿)+ AAVM-Rust 转译构建(build_aavm_rust_bin,
// 含现场 cargo build,产物内容寻址缓存)+ 99_bootstrap v1 ignored 一行测试。
// 共享设施(语料缓存 get_cached_test/VmTestData)仍居 vm_file_tests
// (test-vm-files 门;test-aavm implies 之,经 pub(crate) 复用)。

use crate::error::AutoResult;
use crate::tests::vm_file_tests::get_cached_test;
use crate::{run, run_with_capture};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::OnceLock;

// =============================================================================
// Plan 229b Phase 1.2: AAVM (Auto AutoVM) Test Runner
// =============================================================================
// Tests the self-hosted compiler code (auto/lib-legacy/*.at for v1; auto/lib/*.at for v2)
// cases and running through the AutoVM. Future: transpile via a2r → compile → run.
//
// Test cases reuse the same format as VM file tests (.at + .expected.out).
// The AAVM runner prepends lib code before the test case code.

/// Auto library files to prepend for AAVM tests (order matters: dependencies first).
/// Single source of truth lives in lib.rs (plan-429 A3: the two lists had drifted
/// — this one included generics.at, lib.rs's did not).
const AUTO_LIB_FILES: &[&str] = crate::AUTO_LIB_FILES;

/// Cached auto/lib code. Loaded once per test process.
static AUTO_LIB_CACHE: OnceLock<String> = OnceLock::new();

/// Read and concatenate all auto/lib-legacy/*.at files (cached)
fn read_auto_lib() -> AutoResult<&'static str> {
    let lib_code = AUTO_LIB_CACHE.get_or_init(|| {
        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let mut code = String::new();
        for file in AUTO_LIB_FILES {
            let path = d.join(file);
            if path.exists() {
                if let Ok(content) = read_to_string(&path) {
                    code.push_str(&content);
                    code.push('\n');
                }
            }
        }
        code
    });
    Ok(lib_code)
}

/// AAVM test runner: merges auto/lib code with test case, runs through VM
fn test_aavm(case: &str) -> AutoResult<()> {
    // Use cached test data for the source code
    let data = get_cached_test(case)
        .expect(&format!("AAVM test case '{}' not found in cache. Check test/vm/ directory.", case));

    // Merge: auto/lib code first, then test case (which may override main())
    let lib_code = read_auto_lib()?;
    let merged = format!("{}\n{}", lib_code, data.source);

    // Check .expected.error — expect runtime error
    if data.expected_error {
        let result = run(&merged);
        assert!(
            result.is_err(),
            "Expected error but got: {:?}",
            result
        );
        return Ok(());
    }

    // Execute with stdout capture
    let (_result, stdout) = run_with_capture(&merged)?;

    // Check .expected.out — stdout output
    if let Some(ref expected_out) = data.expected_out {
        if stdout != *expected_out {
            let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let dir_name = case.rsplit('/').next().unwrap_or(case);
            let parts: Vec<&str> = dir_name.splitn(2, '_').collect();
            let name = parts[1..].join("_");
            let wrong_path = d.join(format!("test/vm/{}/{}.wrong.out", case, name));
            std::fs::write(&wrong_path, &stdout)?;
        }
        assert_eq!(stdout, *expected_out);
    }

    // Check .expected.result — return value
    if let Some(ref expected_res) = data.expected_result {
        let result = _result;
        if result != *expected_res {
            let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let dir_name = case.rsplit('/').next().unwrap_or(case);
            let parts: Vec<&str> = dir_name.splitn(2, '_').collect();
            let name = parts[1..].join("_");
            let wrong_path = d.join(format!("test/vm/{}/{}.wrong.result", case, name));
            std::fs::write(&wrong_path, &result)?;
        }
        assert_eq!(result, *expected_res);
    }

    Ok(())
}

// =============================================================================
// Plan 431 E2/E3: AAVM v2 测试基建(test/vm/aavm2/)
//
// E2 runner(test_aavm2):前置拼接 auto/lib v2(AUTO_LIB_FILES_V2,432 逐个登记)
// 后走 VM 执行,与 .expected.out 比对——v2 目录尚空时等价于裸跑用例。
// E3 骨架(test_aavm2_compile,#[ignore]):transpile_rust → 临时 cargo 工程 →
// cargo build → 运行 → diff stdout。完整四向对比是 plan-433 的任务。

const AUTO_LIB_FILES_V2: &[&str] = crate::AUTO_LIB_FILES_V2;

static AUTO_LIB_V2_CACHE: OnceLock<String> = OnceLock::new();

fn read_auto_lib_v2() -> AutoResult<&'static str> {
    Ok(AUTO_LIB_V2_CACHE.get_or_init(|| {
        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        // Plan 517 W2 双轨剥离:经 aavm2_lib_source 剥 use auto.lib.* 行
        crate::aavm2_lib_source(&d).unwrap_or_default()
    }))
}

/// AAVM v2 runner:合并 v2 lib 后走 VM。
fn test_aavm2(case: &str) -> AutoResult<()> {
    let data = get_cached_test(case)
        .expect(&format!("aavm2 test case '{}' not found", case));
    let lib_code = read_auto_lib_v2()?;
    let merged = format!("{}
{}", lib_code, data.source);
    if data.expected_error {
        assert!(run(&merged).is_err(), "expected error for {}", case);
        return Ok(());
    }
    let (_result, stdout) = run_with_capture(&merged)?;
    if let Some(ref expected_out) = data.expected_out {
        assert_eq!(stdout, *expected_out, "aavm2 output mismatch for {}", case);
    }
    Ok(())
}

/// Plan 431 E3 骨架:a2r 编译对比(transpile → cargo build → run → diff)。
/// 需要 cargo 工具链,#[ignore];语料在 test/vm/aavm2/*/,预期输出为
/// `main.expected.out`。433 扩为全量 corpus runner 时沿用此管线。
fn test_aavm2_compile(case: &str) -> AutoResult<()> {
    use std::process::Command;
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let case_dir = d.join(format!("test/vm/aavm2/{}", case));
    // 目录名 002_hello_compile → 语料名 hello_compile(与 vm_file_tests 全库约定一致)
    let dir_name = case.rsplit('/').next().unwrap_or(case);
    let name = dir_name.splitn(2, '_').nth(1).unwrap_or(dir_name).to_string();
    let src = read_to_string(case_dir.join(format!("{}.at", name)))?;
    let expected = read_to_string(case_dir.join(format!("{}.expected.out", name)))?;

    let mut sink = crate::trans::rust::transpile_rust(&name, &src)?;
    let rs_code = sink.done()?;

    // 临时 cargo 工程(bin)
    let proj = std::env::temp_dir().join(format!("aavm2-{}", name));
    let src_dir = proj.join("src");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(proj.join("Cargo.toml"), format!(
        "[package]
name = \"aavm2_{name}\"
version = \"0.1.0\"
edition = \"2021\"

[workspace]

[dependencies]
"
    ))?;
    std::fs::write(src_dir.join("main.rs"), rs_code)?;

    let build = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&proj)
        .output()
        .expect("cargo spawn");
    assert!(
        build.status.success(),
        "cargo build failed:
{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let exe = proj.join("target/release").join(format!("aavm2_{}", name.replace('-', "_")));
    let run_out = Command::new(&exe).output().expect("run compiled bin");
    assert!(run_out.status.success(), "compiled bin failed");
    let stdout = String::from_utf8_lossy(&run_out.stdout).to_string();
    assert_eq!(stdout, expected, "aavm2 compile-compare mismatch for {}", case);
    Ok(())
}

#[test]
fn test_aavm2_001_smoke() { test_aavm2("aavm2/001_smoke").unwrap(); }

#[test] #[ignore]
fn test_aavm2_002_hello_compile() { test_aavm2_compile("002_hello_compile").unwrap(); }

// =============================================================================
// Plan 433 B: AAVM-Rust(②)— auto/lib v2 经 a2r --merge 转译编译出的
// 编译器+VM 二进制,对 corpus 执行层语料跑行为,与 ① Rust 参考
// (run_with_capture live oracle)对齐。管线沿用 431-E3 骨架:
// transpile_rust_project_merged → 临时 cargo bin(内容寻址缓存)→ 运行 → diff。

/// Plan 433 B1: 组装 AAVM-Rust 二进制(merge 转译 + main harness)。
/// 返回 exe 路径。产物按转译内容 hash 缓存,重跑不重建。
fn build_aavm_rust_bin() -> PathBuf {
    use std::process::Command;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    // Plan 434 追记:242 #18 修复后 ② 回归整目录(含 a2r.at)。
    // Plan 517 W2 双轨剥离:merge 输入为剥除 use 行的临时目录副本
    // (use 发射的 crate:: 路径在单文件平铺 merge 产物中不可解析,剥离后
    // 与模块化前行为逐字节等价)。
    let lib_dir = {
        let src_dir = &root;
        // Plan 523 复审修复:剥离目录按进程隔离——固定共享路径在两 corpus 腿
        // 并发首建时互删(remove_dir_all 打架,199KB/400KB 残缺 merge 实证)。
        let stripped = std::env::temp_dir().join(format!(
            "aavm2-merge-lib-stripped-p{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&stripped);
        std::fs::create_dir_all(&stripped).expect("mkdir merge-lib-stripped");
        for file in AUTO_LIB_FILES_V2 {
            let content = std::fs::read_to_string(src_dir.join(file)).expect("read lib file");
            let mut out = String::new();
            for line in content.lines() {
                if line.trim_start().starts_with("use auto.lib.") {
                    continue;
                }
                out.push_str(line);
                out.push('\n');
            }
            let name = std::path::Path::new(file)
                .file_name().unwrap().to_str().unwrap().to_string();
            std::fs::write(stripped.join(name), out).expect("write stripped lib");
        }
        stripped
    };
    let merged = crate::trans::rust::transpile_rust_project_merged(
        lib_dir.to_str().expect("utf8 path")).expect("a2r merge transpile auto/lib");

    // 内容寻址缓存:同产物不重复 cargo build
    // Plan 511 W3 前导 shim:aavm lib 的 File.read_text(W3 模块解析原语)
    // 在 a2r 转译侧无原生映射——bin 层提供同名直通实现(leg 语料不触达,
    // 仅为可编译;divergence 登记 D-AA2R-struct 条目内)
    let prelude = r#"
#[allow(dead_code)]
pub struct File;
#[allow(dead_code)]
impl File {
    pub fn read_text(p: impl AsRef<std::path::Path>) -> String {
        std::fs::read_to_string(p).unwrap_or_default()
    }
}
"#;
    // harness main:读 .at → ev_run → stdout
    let harness = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Plan 523 W2:--files <main.at> → ev_run_files 多文件主入口
    // (corpus_use 编译腿;524 aavm.at 位置参数形态同款路由)
    if args.len() >= 3 && &args[1] == "--files" {
        let out = ev_run_files(&args[2]);
        print!("{}", out);
        return;
    }
    // Plan 523 W3:path4(AA2R self-bin)译文模式——读 .at → ar_run → stdout
    if args.len() >= 3 && &args[1] == "--trans" {
        let source = match std::fs::read_to_string(&args[2]) {
            Ok(s) => s,
            Err(e) => { eprintln!("read error: {}", e); std::process::exit(2); }
        };
        let out = ar_run(&source, 0);
        print!("{}", out);
        return;
    }
    if args.len() < 2 {
        eprintln!("usage: aavm2 <file.at>");
        std::process::exit(2);
    }
    let source = match std::fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => { eprintln!("read error: {}", e); std::process::exit(2); }
    };
    let out = ev_run(&source);
    print!("{}", out);
}
"#;

    let hash = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        merged.hash(&mut h);
        prelude.hash(&mut h);
        format!("{:016x}", h.finish())
    };
    // Plan 523 W2:harness 文本入键(--files 模式加入后,harness 变更必须
    // 换新缓存——此前仅 merged+prelude,harness 演进走旧 exe)。
    let harness_hash = {
        use std::hash::{Hash, Hasher};
        let mut hh = std::collections::hash_map::DefaultHasher::new();
        harness.hash(&mut hh);
        format!("{:016x}", hh.finish())
    };
    let proj = std::env::temp_dir().join(format!("aavm2-bin-{}-{}", hash, harness_hash));
    // Plan 523 W3:每进程独立 staging 构建 + 完工原子改名发布——规避
    // nextest 多进程对同一新 hash 目录的首建竞态(写/读竞态在 cargo 自身
    // 锁之外;前期 mkdir 锁方案在进程异常退出时残留锁目录致等待死等)。
    let staging = std::env::temp_dir().join(format!(
        "aavm2-bin-{}-{}-p{}",
        hash,
        harness_hash,
        std::process::id()
    ));
    let exe = proj.join("target/release/aavm2_bin.exe");
    if exe.exists() {
        return exe;
    }
    let src_dir = staging.join("src");
    std::fs::create_dir_all(&src_dir).expect("create src dir");
    std::fs::write(staging.join("Cargo.toml"), "[package]\nname = \"aavm2_bin\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n\n[dependencies]\n").unwrap();
    let mut full = merged.clone();
    full.extend_from_slice(prelude.as_bytes());
    full.extend_from_slice(harness.as_bytes());
    std::fs::write(src_dir.join("main.rs"), &full).unwrap();

    let build = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&staging)
        .output()
        .expect("cargo spawn");
    assert!(
        build.status.success(),
        "AAVM-Rust cargo build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    // Plan 523 W3:完工发布——赢家改名;输家(发布位已被赢家占位)清理
    // 自身 staging 用已发布产物;改名失败且发布位仍缺时退回自身产物。
    if exe.exists() {
        let _ = std::fs::remove_dir_all(&staging);
        return exe;
    }
    match std::fs::rename(&staging, &proj) {
        Ok(()) => exe,
        Err(_) => {
            if exe.exists() {
                let _ = std::fs::remove_dir_all(&staging);
                exe
            } else {
                staging.join("target/release/aavm2_bin.exe")
            }
        }
    }
}

/// Plan 433 B2: corpus 执行层语料上 ②(AAVM-Rust) vs ①(Rust 参考)
/// 行为对齐。语料 = corpus_m4 全量(= 99_bootstrap 038-052 回收 +
/// 数组四件套,M4/M5 双绿集)。#[ignore]]:需 cargo 工具链,按需跑:
/// Plan 514 W2（P511-1/2 清偿）：corpus 37/37 全绿（CJK 注释词法哨兵 +
/// 主 a2r 二元优先级括号双根因修复），去 ignore 纳入常规门禁——
/// `-- test_aavm2` 默认即覆盖本腿。
#[test]
fn test_aavm2_compile_corpus() {
    let exe = build_aavm_rust_bin();
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/vm/aavm2/corpus_m4");
    // Plan 511 待澄清①缺省处置的欠账(b34–b43 十前缀跳过)于 Plan 523 W2
    // 摘除——中阶发射面已补全(主 a2r H1–H5/H7 洞清偿 + AA2R 镜像 +
    // H6 T3 通道),a2r 运行闸全量覆盖中阶语料。留痕:原
    // a2r_skip_prefixes = ["b34","b35","b36","b37","b38","b39","b40",
    // "b41","b42","b43"]。
    let mut entries: Vec<_> = std::fs::read_dir(&corpus)
        .expect("corpus_m4 dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    entries.sort();
    assert!(!entries.is_empty());
    let mut mismatches: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for path in &entries {
        let code = std::fs::read_to_string(path).unwrap();
        // ① Rust 参考(live oracle,与 M5 闸门同源)
        let (_r, expected) = run_with_capture(&code).expect("rust reference run");
        // ② AAVM-Rust
        let out = std::process::Command::new(&exe)
            .arg(path)
            .output()
            .expect("run aavm2 bin");
        assert!(out.status.success(), "aavm2 bin failed on {}", path.display());
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        checked += 1;
        if stdout.trim_end() != expected.trim_end() {
            mismatches.push(format!(
                "{}\n--- rust ---\n{}\n--- aavm-rust ---\n{}",
                path.display(), expected, stdout
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "AAVM-Rust corpus mismatches ({} of {}):\n{}",
        mismatches.len(), checked, mismatches.join("\n====\n")
    );
    eprintln!("AAVM-Rust corpus: {checked} files, outputs identical to Rust reference");
}

/// Plan 523 W2:corpus_use 多文件用例入 a2r 编译腿(转译版 ev_run_files
/// 首次实测)——① Rust 参考(run_with_capture_and_path 模块解析 oracle,
/// 与 M5 use-corpus 同源)vs ② aavm2_bin --files(merge 转译编译产物,
/// 内容寻址缓存)。
#[test]
fn test_aavm2_compile_use_corpus() {
    let exe = build_aavm_rust_bin();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/vm/aavm2/corpus_use");
    let mut cases: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("corpus_use dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).map(|n| n != "errors").unwrap_or(true))
        .collect();
    cases.sort();
    assert!(!cases.is_empty(), "no corpus_use cases");
    let mut mismatches: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for case in &cases {
        let main_path = case.join("main.at");
        let code = std::fs::read_to_string(&main_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", main_path.display()));
        let (_r, expected) = crate::run_with_capture_and_path(&code, &main_path.display().to_string())
            .unwrap_or_else(|e| panic!("rust reference failed on {}: {e}", case.display()));
        let out = std::process::Command::new(&exe)
            .arg("--files")
            .arg(&main_path)
            .output()
            .unwrap_or_else(|e| panic!("run aavm2 bin on {}", case.display()));
        assert!(out.status.success(), "aavm2 bin --files failed on {}", case.display());
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        checked += 1;
        if stdout.trim_end() != expected.trim_end() {
            mismatches.push(format!(
                "{}
--- rust ---
{}
--- aavm-rust ---
{}",
                case.display(), expected, stdout
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "AAVM-Rust use-corpus mismatches ({} of {}):
{}",
        mismatches.len(), checked, mismatches.join("
====
")
    );
    eprintln!("AAVM-Rust use corpus: {checked} cases, outputs identical to Rust reference");
}

// Plan 233: AAVM Parser tests
#[test] #[ignore] fn test_99_bootstrap_008_parser_hello() { test_aavm("99_bootstrap/008_parser_hello").unwrap(); }

// Plan 233: AAVM v1 Parser 测试(ignored,自 vm_file_tests 配对区迁入)
#[test] #[ignore] fn test_aavm_99_bootstrap_009_parser_arithmetic() { test_aavm("99_bootstrap/009_parser_arithmetic").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_010_parser_precedence() { test_aavm("99_bootstrap/010_parser_precedence").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_011_parser_unary() { test_aavm("99_bootstrap/011_parser_unary").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_012_parser_not() { test_aavm("99_bootstrap/012_parser_not").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_013_parser_comparison() { test_aavm("99_bootstrap/013_parser_comparison").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_014_parser_equality() { test_aavm("99_bootstrap/014_parser_equality").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_015_parser_logical() { test_aavm("99_bootstrap/015_parser_logical").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_016_parser_let() { test_aavm("99_bootstrap/016_parser_let").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_017_parser_var() { test_aavm("99_bootstrap/017_parser_var").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_018_parser_fn_decl() { test_aavm("99_bootstrap/018_parser_fn_decl").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_019_parser_fn_call() { test_aavm("99_bootstrap/019_parser_fn_call").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_020_parser_if_else() { test_aavm("99_bootstrap/020_parser_if_else").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_021_parser_for_in() { test_aavm("99_bootstrap/021_parser_for_in").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_022_parser_return() { test_aavm("99_bootstrap/022_parser_return").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_023_parser_dot() { test_aavm("99_bootstrap/023_parser_dot").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_024_parser_assign() { test_aavm("99_bootstrap/024_parser_assign").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_025_parser_range() { test_aavm("99_bootstrap/025_parser_range").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_026_parser_string() { test_aavm("99_bootstrap/026_parser_string").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_027_parser_multi() { test_aavm("99_bootstrap/027_parser_multi").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_028_parser_alias() { test_aavm("99_bootstrap/028_parser_alias").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_029_parser_enum() { test_aavm("99_bootstrap/029_parser_enum").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_030_parser_use() { test_aavm("99_bootstrap/030_parser_use").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_031_parser_spec() { test_aavm("99_bootstrap/031_parser_spec").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_032_parser_ext() { test_aavm("99_bootstrap/032_parser_ext").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_033_parser_closure() { test_aavm("99_bootstrap/033_parser_closure").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_034_parser_closure_multi() { test_aavm("99_bootstrap/034_parser_closure_multi").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_035_parser_fstr() { test_aavm("99_bootstrap/035_parser_fstr").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_036_parser_is() { test_aavm("99_bootstrap/036_parser_is").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_037_parser_object() { test_aavm("99_bootstrap/037_parser_object").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_038_eval_arithmetic() { test_aavm("99_bootstrap/038_eval_arithmetic").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_039_eval_variable() { test_aavm("99_bootstrap/039_eval_variable").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_040_eval_fn_call() { test_aavm("99_bootstrap/040_eval_fn_call").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_041_eval_if_else() { test_aavm("99_bootstrap/041_eval_if_else").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_042_eval_for_loop() { test_aavm("99_bootstrap/042_eval_for_loop").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_043_eval_recursion() { test_aavm("99_bootstrap/043_eval_recursion").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_044_eval_string() { test_aavm("99_bootstrap/044_eval_string").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_045_eval_list() { test_aavm("99_bootstrap/045_eval_list").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_046_eval_closure() { test_aavm("99_bootstrap/046_eval_closure").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_047_eval_multi_fn() { test_aavm("99_bootstrap/047_eval_multi_fn").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_048_eval_str_print() { test_aavm("99_bootstrap/048_eval_str_print").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_049_eval_str_var() { test_aavm("99_bootstrap/049_eval_str_var").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_050_eval_str_concat() { test_aavm("99_bootstrap/050_eval_str_concat").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_051_eval_str_fn_arg() { test_aavm("99_bootstrap/051_eval_str_fn_arg").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_051a_eval_str_fn_arg_simple() { test_aavm("99_bootstrap/051a_eval_str_fn_arg_simple").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_052_eval_str_return() { test_aavm("99_bootstrap/052_eval_str_return").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_053_eval_str_literal() { test_aavm("99_bootstrap/053_eval_str_literal").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_054_type_infer_basic() { test_aavm("99_bootstrap/054_type_infer_basic").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_055_type_infer_fn() { test_aavm("99_bootstrap/055_type_infer_fn").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_060_bytecode_int() { test_aavm("99_bootstrap/060_bytecode_int").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_061_bytecode_var() { test_aavm("99_bootstrap/061_bytecode_var").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_062_bytecode_fn() { test_aavm("99_bootstrap/062_bytecode_fn").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_063_bytecode_if() { test_aavm("99_bootstrap/063_bytecode_if").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_064_bytecode_for() { test_aavm("99_bootstrap/064_bytecode_for").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_065_bytecode_str() { test_aavm("99_bootstrap/065_bytecode_str").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_066_bytecode_list() { test_aavm("99_bootstrap/066_bytecode_list").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_067_bytecode_map() { test_aavm("99_bootstrap/067_bytecode_map").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_068_bytecode_list_fn() { test_aavm("99_bootstrap/068_bytecode_list_fn").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_069_bvm_str_len() { test_aavm("99_bootstrap/069_bvm_str_len").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_070_bvm_str_methods() { test_aavm("99_bootstrap/070_bvm_str_methods").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_071_bvm_map_str() { test_aavm("99_bootstrap/071_bvm_map_str").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_072_bvm_list_set() { test_aavm("99_bootstrap/072_bvm_list_set").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_073_bvm_list_pop() { test_aavm("99_bootstrap/073_bvm_list_pop").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_074_bvm_str_ops() { test_aavm("99_bootstrap/074_bvm_str_ops").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_075_list_str_push_get() { test_aavm("99_bootstrap/075_list_str_push_get").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_076_list_str_set() { test_aavm("99_bootstrap/076_list_str_set").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_077_list_str_pop() { test_aavm("99_bootstrap/077_list_str_pop").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_078_list_mixed_types() { test_aavm("99_bootstrap/078_list_mixed_types").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_079_list_str_fn_arg() { test_aavm("99_bootstrap/079_list_str_fn_arg").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_080_list_str_loop() { test_aavm("99_bootstrap/080_list_str_loop").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_081_a2r_hello() { test_aavm("99_bootstrap/081_a2r_hello").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_082_a2r_fn() { test_aavm("99_bootstrap/082_a2r_fn").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_083_a2r_var() { test_aavm("99_bootstrap/083_a2r_var").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_084_a2r_if() { test_aavm("99_bootstrap/084_a2r_if").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_085_a2r_for() { test_aavm("99_bootstrap/085_a2r_for").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_086_a2r_str() { test_aavm("99_bootstrap/086_a2r_str").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_087_a2r_use() { test_aavm("99_bootstrap/087_a2r_use").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_088_a2r_type() { test_aavm("99_bootstrap/088_a2r_type").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_089_a2r_enum() { test_aavm("99_bootstrap/089_a2r_enum").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_090_a2r_is() { test_aavm("99_bootstrap/090_a2r_is").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_091_a2r_ext() { test_aavm("99_bootstrap/091_a2r_ext").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_092_a2r_fstr() { test_aavm("99_bootstrap/092_a2r_fstr").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_093_a2r_spec() { test_aavm("99_bootstrap/093_a2r_spec").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_094_a2r_closure() { test_aavm("99_bootstrap/094_a2r_closure").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_095_a2r_alias() { test_aavm("99_bootstrap/095_a2r_alias").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_096_a2r_object() { test_aavm("99_bootstrap/096_a2r_object").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_097_a2r_array() { test_aavm("99_bootstrap/097_a2r_array").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_098_a2r_error_prop() { test_aavm("99_bootstrap/098_a2r_error_prop").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_099_a2r_self_field() { test_aavm("99_bootstrap/099_a2r_self_field").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_100_a2r_use_ffi() { test_aavm("99_bootstrap/100_a2r_use_ffi").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_101_a2r_is_multi() { test_aavm("99_bootstrap/101_a2r_is_multi").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_102_a2r_struct_ctor() { test_aavm("99_bootstrap/102_a2r_struct_ctor").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_103_a2r_generic_type() { test_aavm("99_bootstrap/103_a2r_generic_type").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_104_a2r_generic_vec() { test_aavm("99_bootstrap/104_a2r_generic_vec").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_105_a2r_generic_map() { test_aavm("99_bootstrap/105_a2r_generic_map").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_106_a2r_generic_enum() { test_aavm("99_bootstrap/106_a2r_generic_enum").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_107_a2r_option_match() { test_aavm("99_bootstrap/107_a2r_option_match").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_108_a2r_ownership() { test_aavm("99_bootstrap/108_a2r_ownership").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_110_a2r_phase3() { test_aavm("99_bootstrap/110_a2r_phase3").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_111_a2r_real_source() { test_aavm("99_bootstrap/111_a2r_real_source").unwrap(); }
#[test] #[ignore] fn test_aavm_99_bootstrap_112_a2r_generic_registry() { test_aavm("99_bootstrap/112_a2r_generic_registry").unwrap(); }

/// Plan 523 W3:build_aavm_rust_bin 的 four-path runner 复用入口(内容寻址
/// 缓存同源;aavm2_a2r::test_aavm2_fourpath_runner 消费)。
pub(crate) fn build_aavm_rust_bin_pub() -> PathBuf {
    build_aavm_rust_bin()
}
