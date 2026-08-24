// Plan 432 S2 / M2 闸门:AAVM v2 parser 与 Rust parser 的 AST dump 一致性。
//
// 语料:test/vm/aavm2/corpus_m2/*.at(parser 构造语料,99_bootstrap 009-021 同源)
//       + test/vm/aavm2/corpus_m1/*.at(全程序,M2 语料同源复用);
// 判据:两侧 dump(Code 的 Display S-expr,见 docs/specs/aavm/m2-ast-dump-format.md)
//       逐字符相等。Rust 侧:Parser::from(code).parse() 后 format!("{}", code);
//       AAVM 侧:auto/lib/{token,lexer,parser}.at 的 parse_dump(source)。

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

/// Rust 参考侧 AST dump:parse(含 parser 内联推断,如 let 无注解时的
/// infer_type_expr)后按 Code 的 Display 格式化 —— 与 M2 规范逐字一致。
fn rust_parse_dump(code: &str) -> AutoResult<String> {
    let mut parser = crate::parser::Parser::from(code);
    let ast = parser.parse()?;
    Ok(format!("{}", ast))
}

fn corpus_dirs() -> Vec<PathBuf> {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2");
    vec![base.join("corpus_m2"), base.join("corpus_m1")]
}

fn test_m2_corpus_file(path: &std::path::Path) -> AutoResult<()> {
    let code = std::fs::read_to_string(path)?;
    let expected = rust_parse_dump(&code)?;
    // 前置拼接 AAVM v2 lib(AUTO_LIB_FILES_V2,单一事实源)
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let mut lib_code = String::new();
    for f in crate::AUTO_LIB_FILES_V2 {
        lib_code.push_str(&std::fs::read_to_string(root.join(f))?);
        lib_code.push('\n');
    }
    let program = format!(
        "{}\nfn main() {{\n    print(parse_dump(\"{}\"))\n}}\n",
        lib_code,
        escape_for_at_literal(&code)
    );
    let (_r, stdout) = run_with_capture(&program)?;
    assert_eq!(
        stdout.trim_end(),
        expected.trim_end(),
        "M2 AST-dump mismatch for {}\n--- rust ---\n{}\n--- aavm ---\n{}",
        path.display(),
        expected,
        stdout
    );
    Ok(())
}

#[test]
fn test_aavm2_m2_parser_corpus() {
    let mut checked = 0;
    for dir in corpus_dirs() {
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("corpus dir {}: {e}", dir.display()))
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
            .collect();
        entries.sort();
        assert!(!entries.is_empty(), "no corpus files under {}", dir.display());
        for p in entries {
            test_m2_corpus_file(&p).unwrap();
            checked += 1;
        }
    }
    eprintln!("M2 corpus: {checked} files, AST dumps identical");
}

/// 诊断用:打印 Rust 参考侧对语料的 dump(--nocapture)。
#[test]
fn test_aavm2_m2_rust_dump_print() {
    for dir in corpus_dirs() {
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .expect("corpus dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
            .collect();
        entries.sort();
        for p in entries {
            let code = std::fs::read_to_string(&p).unwrap();
            match rust_parse_dump(&code) {
                Ok(dump) => eprintln!("=== {} ===\n{}\n", p.display(), dump),
                Err(e) => eprintln!("=== {} === PARSE ERROR: {}\n", p.display(), e),
            }
        }
    }
}
