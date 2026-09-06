// Plan 432 S3 / M3 闸门:AAVM v2 typeinfo 与 Rust 类型层(.type 行为通道)一致性。
//
// 语料:test/vm/aavm2/corpus_m3/*.at —— 可执行程序,`print(EXPR.type)` 查询。
// 判据:Rust 侧 run_with_capture 的 stdout(真 VM + infer_expr_type)与
// AAVM 侧 auto/lib/{token,lexer,parser,typeinfo}.at 的 typecheck_dump(source)
// 逐行相等。格式规格:docs/specs/aavm/m3-typecheck-format.md。

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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_m3")
}

fn test_m3_corpus_file(path: &std::path::Path) -> AutoResult<()> {
    let code = std::fs::read_to_string(path)?;
    let (_r, expected) = crate::run_with_capture(&code)?;
    // 前置拼接 AAVM v2 lib(AUTO_LIB_FILES_V2,单一事实源)
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let lib_code = crate::aavm2_lib_source(&root)?;
    let program = format!(
        "{}\nfn main() {{\n    print(typecheck_dump(\"{}\"))\n}}\n",
        lib_code,
        escape_for_at_literal(&code)
    );
    let (_r, stdout) = run_with_capture(&program)?;
    assert_eq!(
        stdout.trim_end(),
        expected.trim_end(),
        "M3 type-inference mismatch for {}\n--- rust(vm) ---\n{}\n--- aavm ---\n{}",
        path.display(),
        expected,
        stdout
    );
    Ok(())
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
    let mut checked = 0;
    for p in entries {
        test_m3_corpus_file(&p).unwrap();
        checked += 1;
    }
    eprintln!("M3 corpus: {checked} files, type tables identical");
}
