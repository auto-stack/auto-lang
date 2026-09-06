// Plan 432 S5 / M3 主里程碑闸门:AAVM 全管线(token→lexer→parser→
// typeinfo→codegen→engine)在 AutoVM 内编译并运行语料,输出与 Rust 参考
// 实现一致。
//
// 语料:复用 corpus_m4/*.at(全部为可执行程序:hello/let/assign/ifelse/
// while/for-range/fib/strcat/logic/multilet)。
// 判据:Rust 侧 run_with_capture 的 stdout 与 AAVM 侧 ev_run(source)
// 逐行相等。

use crate::error::AutoResult;
use crate::run_with_capture;
use std::path::PathBuf;

fn escape_for_at_literal(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_m4")
}

fn test_m5_corpus_file(path: &std::path::Path) -> AutoResult<()> {
    let code = std::fs::read_to_string(path)?;
    let (_r, expected) = run_with_capture(&code)?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let lib_code = crate::aavm2_lib_source(&root)?;
    let program = format!(
        "{}\nfn main() {{\n    print(ev_run(\"{}\"))\n}}\n",
        lib_code,
        escape_for_at_literal(&code)
    );
    let (_r, stdout) = run_with_capture(&program)?;
    assert_eq!(
        stdout.trim_end(),
        expected.trim_end(),
        "M5 behavior mismatch for {}\n--- rust ---\n{}\n--- aavm ---\n{}",
        path.display(),
        expected,
        stdout
    );
    Ok(())
}

#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_m5_engine_corpus() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_m5_engine_corpus") {
        return;
    }
    let dir = corpus_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("corpus dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "no corpus files under {}", dir.display());
    let mut checked = 0;
    for p in entries {
        test_m5_corpus_file(&p).unwrap();
        checked += 1;
    }
    eprintln!("M5 corpus: {checked} files, outputs identical");
}


// ── Plan 511 W3:corpus_use 多文件行为腿 + 错误用例通道 ──

fn corpus_use_dir_m5() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_use")
}

fn aavm_lib_program_m5(call: &str) -> AutoResult<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let lib_code = crate::aavm2_lib_source(&root)?;
    Ok(format!("{}
fn main() {{
    print({})
}}
", lib_code, call))
}

/// corpus_use 成功用例:Rust 侧 execute 管线(带路径,镜像 resolve_uses+
/// Linker 全链)输出 vs aavm ev_run_files。
#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_m5_use_corpus() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_m5_use_corpus") {
        return;
    }
    let dir = corpus_use_dir_m5();
    let mut cases: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("corpus_use dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).map(|n| n != "errors").unwrap_or(true))
        .collect();
    cases.sort();
    assert!(!cases.is_empty(), "no corpus_use cases");
    for case in &cases {
        let main_path = case.join("main.at");
        let code = std::fs::read_to_string(&main_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", main_path.display()));
        let (_r, expected) = crate::run_with_capture_and_path(&code, &main_path.display().to_string())
            .unwrap_or_else(|e| panic!("rust reference failed on {}: {e}", case.display()));
        let program = aavm_lib_program_m5(&format!(
            "ev_run_files(\"{}\")",
            escape_for_at_literal(&main_path.display().to_string())
        ))
        .unwrap();
        let (_r2, stdout) = crate::run_with_capture(&program)
            .unwrap_or_else(|e| panic!("aavm run failed on {}: {e}", case.display()));
        assert_eq!(
            stdout.trim_end(),
            expected.trim_end(),
            "M5 use-corpus mismatch for {}
--- rust ---
{}
--- aavm ---
{}",
            case.display(),
            expected,
            stdout
        );
    }
    eprintln!("M5 use corpus: outputs identical");
}

/// corpus_use 错误用例通道:aavm2 闸门的 .expected.error 等价物——两侧错误
/// 信息一致(错误文本以宿主为规范;use.rs 拒绝/未声明模块/use.py)。
#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_m5_use_errors() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_m5_use_errors") {
        return;
    }
    let dir = corpus_use_dir_m5().join("errors");
    let mut cases: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("errors dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    cases.sort();
    assert!(!cases.is_empty(), "no error cases");
    for case in &cases {
        let main_path = case.join("main.at");
        let code = std::fs::read_to_string(&main_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", main_path.display()));
        let rust_err = crate::run_with_capture_and_path(&code, &main_path.display().to_string())
            .expect_err(&format!("host should error on {}", case.display()))
            .to_string();
        let program = aavm_lib_program_m5(&format!(
            "ev_run_files(\"{}\")",
            escape_for_at_literal(&main_path.display().to_string())
        ))
        .unwrap();
        let (_r2, stdout) = crate::run_with_capture(&program)
            .unwrap_or_else(|e| panic!("aavm run failed on {}: {e}", case.display()));
        assert_eq!(
            stdout.trim_end(),
            rust_err.trim_end(),
            "use-error mismatch for {}
--- rust ---
{}
--- aavm ---
{}",
            case.display(),
            rust_err,
            stdout
        );
    }
    eprintln!("use-error channel: messages identical");
}

/// M3 主里程碑演示:一条命令在 AutoVM 内经 AAVM 编译运行 helloworld 与
/// fib,输出正确(fib(10) = 55)。
#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_m3_milestone_fib() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_m3_milestone_fib") {
        return;
    }
    let dir = corpus_dir();
    let hello = std::fs::read_to_string(dir.join("b01_hello.at")).unwrap();
    let fib = std::fs::read_to_string(dir.join("b07_fib.at")).unwrap();
    for (name, code) in [("helloworld", &hello), ("fib", &fib)] {
        let (_r, expected) = run_with_capture(code).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let lib_code = crate::aavm2_lib_source(&root).unwrap();
        let program = format!(
            "{}\nfn main() {{\n    print(ev_run(\"{}\"))\n}}\n",
            lib_code,
            escape_for_at_literal(code)
        );
        let (_r, stdout) = run_with_capture(&program).unwrap();
        assert_eq!(
            stdout.trim_end(),
            expected.trim_end(),
            "M3 milestone mismatch for {}",
            name
        );
        eprintln!("M3 milestone [{}] => {}", name, stdout.trim_end());
    }
}
