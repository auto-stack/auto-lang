// Plan 432 S1 / M1 闸门:AAVM v2 lexer 与 Rust lexer 的 token 流一致性。
//
// 语料:test/vm/aavm2/corpus_m1/*.at(原始源码文件,非用例);
// 判据:两侧 dump(kind|esc_text|line|at|len)逐行相等。
// Rust 侧:crate::lexer 逐 token,kind 用 Debug 名,文本转义(\\ \n \t \r);
// AAVM 侧:auto/lib/{token,lexer}.at 的 lex_dump(同一格式约定,见 lexer.at 头)。

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

fn test_m1_corpus_file(path: &std::path::Path) -> AutoResult<()> {
    let code = std::fs::read_to_string(path)?;
    let expected = rust_lex_dump(&code);
    // 前置拼接 AAVM v2 lib(AUTO_LIB_FILES_V2,单一事实源)
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let mut lib_code = String::new();
    for f in crate::AUTO_LIB_FILES_V2 {
        lib_code.push_str(&std::fs::read_to_string(root.join(f))?);
        lib_code.push('\n');
    }
    let program = format!(
        "{}\nfn main() {{\n    print(lex_dump(\"{}\"))\n}}\n",
        lib_code,
        escape_for_at_literal(&code)
    );
    let (_r, stdout) = run_with_capture(&program)?;
    assert_eq!(
        stdout.trim_end(),
        expected.trim_end(),
        "M1 token-stream mismatch for {}",
        path.display()
    );
    Ok(())
}

#[test]
fn test_aavm2_m1_lexer_corpus() {
    let dir = corpus_dir();
    let mut checked = 0;
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("corpus_m1 dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "no corpus files under {}", dir.display());
    for p in entries {
        test_m1_corpus_file(&p).unwrap();
        checked += 1;
    }
    eprintln!("M1 corpus: {checked} files, token streams identical");
}
