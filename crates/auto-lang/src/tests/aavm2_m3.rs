// Plan 432 S3 / M3 闸门:AAVM v2 typeinfo 与 Rust 类型层(.type 行为通道)一致性。
//
// 语料:test/vm/aavm2/corpus_m3/*.at —— 可执行程序,`print(EXPR.type)` 查询。
// 判据:Rust 侧 run_with_capture 的 stdout(真 VM + infer_expr_type)与
// AAVM 侧 auto/lib/{token,lexer,parser,typeinfo}.at 的 typecheck_dump(source)
// 逐行相等。格式规格:docs/specs/aavm/m3-typecheck-format.md。
// Plan 565 L1:AAVM 侧语料走 once-compiled runner(编译一次+File.read_text
//       注入,见 aavm2_corpus_runner.rs);判据断言原样保留在本闸门
//       (Rust 参考侧仍逐语料 run_with_capture,量级可忽略)。

use std::path::PathBuf;

use crate::tests::aavm2_corpus_runner::{run_corpus_once_compiled, CorpusCase};

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_m3")
}

#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_m3_typeinfo_corpus() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_m3_typeinfo_corpus") {
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
    let cases: Vec<CorpusCase> = entries
        .into_iter()
        .map(|p| CorpusCase {
            code: std::fs::read_to_string(&p).unwrap(),
            path: p,
        })
        .collect();
    let outs = run_corpus_once_compiled("m3", "typecheck_dump", &cases)
        .unwrap_or_else(|e| panic!("M3 corpus runner: {e}"));
    let mut checked = 0;
    for (case, stdout) in cases.iter().zip(&outs) {
        let (_r, expected) = crate::run_with_capture(&case.code).unwrap();
        assert_eq!(
            stdout.trim_end(),
            expected.trim_end(),
            "M3 type-inference mismatch for {}\n--- rust(vm) ---\n{}\n--- aavm ---\n{}",
            case.path.display(),
            expected,
            stdout
        );
        checked += 1;
    }
    eprintln!("M3 corpus: {checked} files, type tables identical (once-compiled runner)");
}
