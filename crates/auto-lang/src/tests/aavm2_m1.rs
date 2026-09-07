// Plan 432 S1 / M1 闸门:AAVM v2 lexer 与 Rust lexer 的 token 流一致性。
//
// 语料:test/vm/aavm2/corpus_m1/*.at(原始源码文件,非用例);
// 判据:两侧 dump(kind|esc_text|line|at|len)逐行相等。
// Rust 侧:crate::lexer 逐 token,kind 用 Debug 名,文本转义(\\ \n \t \r);
// AAVM 侧:auto/lib/{token,lexer}.at 的 lex_dump(同一格式约定,见 lexer.at 头)。
// Plan 565 L1:语料走 once-compiled runner(编译一次+File.read_text 注入,
//       见 aavm2_corpus_runner.rs);判据断言原样保留在本闸门。

use std::path::PathBuf;

use crate::tests::aavm2_corpus_runner::{run_corpus_once_compiled, CorpusCase};

fn rust_lex_dump(code: &str) -> String {
    let mut lexer = crate::lexer::Lexer::new(code);
    let mut out = String::new();
    loop {
        let t = match lexer.next() {
            Ok(t) => t,
            Err(_) => break,
        };
        let text = t.text.to_string();
        let esc = text
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\t', "\\t")
            .replace('\r', "\\r");
        out.push_str(&format!(
            "{:?}|{}|{}|{}|{}\n",
            t.kind, esc, t.pos.line, t.pos.at, t.pos.len
        ));
        if t.kind == crate::token::TokenKind::EOF {
            break;
        }
    }
    out
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_m1")
}

#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_m1_lexer_corpus() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_m1_lexer_corpus") {
        return;
    }
    let dir = corpus_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("corpus_m1 dir")
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
    let outs = run_corpus_once_compiled("m1", "lex_dump", &cases)
        .unwrap_or_else(|e| panic!("M1 corpus runner: {e}"));
    let mut checked = 0;
    for (case, stdout) in cases.iter().zip(&outs) {
        let expected = rust_lex_dump(&case.code);
        assert_eq!(
            stdout.trim_end(),
            expected.trim_end(),
            "M1 token-stream mismatch for {}",
            case.path.display()
        );
        checked += 1;
    }
    eprintln!("M1 corpus: {checked} files, token streams identical (once-compiled runner)");
}
