// Plan 447 部分② Phase 7 / AA2R-is 闸门:a2r.at 自身的 is-match/or-臂/
// 枚举载荷发射与主 a2r(transpile_rust,基线 = 已修 H4/H5)逐字符一致。
//
// 语料:test/vm/aavm2/corpus_a2r/*.at(枚举载荷 is/标量 or-臂/字面量与
// 卫语句/字符串模式)。
// 判据:Rust 侧 transpile_rust(name, src).done() 与 AAVM 侧 auto/lib
// 全七文件前置后 ar_run(src, 0) 的输出逐字符相等(M2 式 live 对齐,
// 无落盘 golden——主 a2r 行为即唯一基准)。

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
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_a2r")
}

fn test_a2r_corpus_file(path: &std::path::Path) -> AutoResult<()> {
    let code = std::fs::read_to_string(path)?;
    let stem = path.file_stem().unwrap().to_string_lossy().to_string();
    let name = stem.splitn(2, '_').nth(1).unwrap_or(&stem).to_string();
    let mut sink = crate::trans::rust::transpile_rust(&name, &code)?;
    let expected = String::from_utf8_lossy(sink.done()?).to_string();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let mut lib_code = String::new();
    for f in crate::AUTO_LIB_FILES_V2 {
        lib_code.push_str(&std::fs::read_to_string(root.join(f))?);
        lib_code.push('\n');
    }
    let program = format!(
        "{}\nfn main() {{\n    print(ar_run(\"{}\", 0))\n}}\n",
        lib_code,
        escape_for_at_literal(&code)
    );
    let (_r, stdout) = run_with_capture(&program)?;
    if std::env::var("AA2R_DUMP").is_ok() {
        eprintln!("DUMP-FILE {}
DUMP-HOST<<<{}>>>
DUMP-AA2R<<<{}>>>", path.display(), expected, stdout);
    }
    assert_eq!(
        stdout.trim_end(),
        expected.trim_end(),
        "AA2R mismatch for {}\n--- main a2r ---\n{}\n--- aa2r ---\n{}",
        path.display(),
        expected,
        stdout
    );
    Ok(())
}

#[test]
fn test_aavm2_a2r_is_corpus() {
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
        test_a2r_corpus_file(&p).unwrap();
        checked += 1;
    }
    eprintln!("AA2R is corpus: {checked} files, transpiled text identical to main a2r");
}

/// 诊断用:打印主 a2r 对语料的转译产物(--nocapture)。
#[test]
fn test_aavm2_a2r_main_dump_print() {
    let dir = corpus_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    entries.sort();
    for p in entries {
        let code = std::fs::read_to_string(&p).unwrap();
        let stem = p.file_stem().unwrap().to_string_lossy().to_string();
        let name = stem.splitn(2, '_').nth(1).unwrap_or(&stem).to_string();
        match crate::trans::rust::transpile_rust(&name, &code) {
            Ok(mut sink) => {
                let rs = String::from_utf8_lossy(sink.done().unwrap()).to_string();
                eprintln!("=== {} ===\n{}\n", p.display(), rs)
            }
            Err(e) => eprintln!("=== {} === TRANSPILE ERROR: {}\n", p.display(), e),
        }
    }
}

/// Plan 447 7.5 探针冒烟:99_idiom_probe 的 p01/p02b/p04/p05/p12 经 AA2R
/// (a2r.at 自身)转译后 rustc 零错。`#[ignore]`:需 rustc,按需跑:
/// cargo test -p auto-lang --lib --features test-vm-files a2r_probe_smoke -- --ignored
#[test]
#[ignore = "shells out to rustc; on-demand AA2R compile-level guard (Plan 447)"]
fn test_aavm2_a2r_probe_smoke() {
    let probes = [
        "p01_is_string",
        "p02b_enum_or_arm",
        "p04_runtime_concat_payload",
        "p05_double_match",
        "p12_is_binding_types",
    ];
    let lib_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let mut lib_code = String::new();
    for f in crate::AUTO_LIB_FILES_V2 {
        lib_code.push_str(&std::fs::read_to_string(lib_dir.join(f)).unwrap());
        lib_code.push('\n');
    }
    let probe_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/99_idiom_probe");
    for p in probes {
        let case_dir = probe_root.join(p);
        let stem = p.splitn(2, '_').nth(1).unwrap_or(p);
        let src = std::fs::read_to_string(case_dir.join(format!("{}.at", stem))).unwrap();
        let program = format!(
            "{}
fn main() {{
    print(ar_run(\"{}\", 0))
}}
",
            lib_code,
            escape_for_at_literal(&src)
        );
        let (_r, stdout) = run_with_capture(&program)
            .unwrap_or_else(|e| panic!("AA2R transpile failed for {}: {}", p, e));
        let rs = stdout.trim_end().to_string();
        assert!(!rs.starts_with("TRANSPILE-ERROR") && !rs.is_empty(), "AA2R error for {}: {}", p, rs);
        let out = std::env::temp_dir().join(format!("aa2r_probe_{}.rmeta", stem));
        let status = std::process::Command::new("rustc")
            .args(["--crate-type=bin", "--edition", "2021", "--emit=metadata", "-o"])
            .arg(&out)
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(rs.as_bytes()).unwrap();
                child.wait_with_output()
            })
            .expect("rustc spawn");
        assert!(
            status.status.success(),
            "AA2R product for {} failed rustc:
{}
--- product ---
{}",
            p,
            String::from_utf8_lossy(&status.stderr),
            rs
        );
    }
}
