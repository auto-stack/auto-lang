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
    let mut lib_code = String::new();
    for f in crate::AUTO_LIB_FILES_V2 {
        lib_code.push_str(&std::fs::read_to_string(root.join(f))?);
        lib_code.push('\n');
    }
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
fn test_aavm2_m5_engine_corpus() {
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

/// M3 主里程碑演示:一条命令在 AutoVM 内经 AAVM 编译运行 helloworld 与
/// fib,输出正确(fib(10) = 55)。
#[test]
fn test_aavm2_m3_milestone_fib() {
    let dir = corpus_dir();
    let hello = std::fs::read_to_string(dir.join("b01_hello.at")).unwrap();
    let fib = std::fs::read_to_string(dir.join("b07_fib.at")).unwrap();
    for (name, code) in [("helloworld", &hello), ("fib", &fib)] {
        let (_r, expected) = run_with_capture(code).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let mut lib_code = String::new();
        for f in crate::AUTO_LIB_FILES_V2 {
            lib_code.push_str(&std::fs::read_to_string(root.join(f)).unwrap());
            lib_code.push('\n');
        }
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
