//! Plan 433 C1: AAVM 四向对比矩阵(①②③④)。
//!
//! | 方 | backend | 说明 |
//! |---|---|---|
//! | ① | reference | auto-lang 原生实现(`auto <case.at>` 的求值 stdout,oracle) |
//! | ② | aavm_rust | auto/lib v2 经 a2r --merge 转译编译出的编译器+VM 二进制 |
//! | ③ | aavm_vm | AutoVM 解释执行 AAVM .at(lib 前置拼接 + ev_run 包装) |
//! | ④ | golden | corpus 的 `.expected.out`;缺失时回落 ①(矩阵注明) |
//!
//! 一条命令可复现(Verification #2):
//! `cargo run -- --root . --auto-binary ../../target/debug/auto.exe aavm`

use std::path::{Path, PathBuf};
use std::process::Command;

/// AAVM v2 库文件(依赖序;与 crates/auto-lang/src/lib.rs 的
/// AUTO_LIB_FILES_V2 单一事实源保持同步)。
pub const AUTO_LIB_FILES_V2: &[&str] = &[
    "auto/lib/token.at",
    "auto/lib/lexer.at",
    "auto/lib/parser.at",
    "auto/lib/typeinfo.at",
    "auto/lib/codegen.at",
    "auto/lib/engine.at",
];

/// corpus 执行层语料目录(相对 repo root)。
pub const CORPUS_DIR: &str = "crates/auto-lang/test/vm/aavm2/corpus_m4";

/// 单个语料的四向结果。
#[derive(Debug, Clone)]
pub struct MatrixCase {
    pub name: String,
    /// ① 参考 stdout(trim_end 后,与 M5 闸门同判据)
    pub reference: String,
    /// ② AAVM-Rust stdout
    pub aavm_rust: String,
    /// ③ AAVM-VM stdout
    pub aavm_vm: String,
    /// ④ golden 文本(无 golden 文件时 = ①,`golden_is_file` 记 false)
    pub golden: String,
    pub golden_is_file: bool,
    /// 后端运行错误(非空 = 该后端没能产出输出)
    pub errors: Vec<String>,
}

impl MatrixCase {
    pub fn all_agree(&self) -> bool {
        self.errors.is_empty()
            && self.reference.trim() == self.aavm_rust.trim()
            && self.reference.trim() == self.aavm_vm.trim()
            && self.reference.trim() == self.golden.trim()
    }
}

/// 矩阵报告。
pub struct MatrixReport {
    pub cases: Vec<MatrixCase>,
    /// ② 二进制备注(路径/缓存命中)
    pub bin_note: String,
}

impl MatrixReport {
    pub fn all_green(&self) -> bool {
        self.cases.iter().all(|c| c.all_agree())
    }
}

/// 运行配置(repo root 从 parity root 推导)。
pub struct AavmConfig {
    pub repo_root: PathBuf,
    pub auto_binary: String,
}

/// 收集 corpus .at 语料(按文件名排序)。
fn corpus_files(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = repo_root.join(CORPUS_DIR);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("corpus dir {}: {}", dir.display(), e))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    files.sort();
    Ok(files)
}

/// `auto <file>` 的 stdout 含启动横幅;截取最后一行 `----` 之后的内容。
fn strip_auto_banner(stdout: &str) -> String {
    if let Some(pos) = stdout.rfind("----------------------") {
        stdout[pos + "----------------------".len()..].trim_start_matches('\r').to_string()
    } else {
        stdout.to_string()
    }
}

fn run_auto(config: &AavmConfig, program_path: &Path) -> Result<String, String> {
    let out = Command::new(&config.auto_binary)
        .arg(program_path)
        .output()
        .map_err(|e| format!("spawn {}: {}", config.auto_binary, e))?;
    if !out.status.success() {
        return Err(format!(
            "auto exited {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(strip_auto_banner(&String::from_utf8_lossy(&out.stdout)))
}

/// 把语料源转义为 .at 字符串字面量载荷(镜像 aavm2_m5.rs 的 escape)。
fn escape_for_at_literal(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// ③ AAVM-VM:lib 前置拼接 + ev_run 包装,经 AutoVM 解释执行。
fn run_aavm_vm(config: &AavmConfig, case: &Path) -> Result<String, String> {
    let mut program = String::new();
    for f in AUTO_LIB_FILES_V2 {
        let path = config.repo_root.join(f);
        program.push_str(&std::fs::read_to_string(&path).map_err(|e| format!("{}: {}", path.display(), e))?);
        program.push('\n');
    }
    let source = std::fs::read_to_string(case).map_err(|e| e.to_string())?;
    program.push_str(&format!(
        "\nfn main() {{\n    print(ev_run(\"{}\"))\n}}\n",
        escape_for_at_literal(&source)
    ));
    let tmp = std::env::temp_dir().join(format!("aavm-parity-vm-{}.at", std::process::id()));
    std::fs::write(&tmp, &program).map_err(|e| e.to_string())?;
    run_auto(config, &tmp)
}

/// ② AAVM-Rust:auto trans --merge → main harness → cargo bin(内容寻址缓存)。
fn build_aavm_rust_bin(config: &AavmConfig) -> Result<(PathBuf, String), String> {
    let lib_dir = config.repo_root.join("auto/lib");
    // 经临时文件取 merge 产物(CLI 的 stdout 会带 [trans] 横幅,文件内容干净)
    let tmp = std::env::temp_dir().join(format!("aavm-parity-merged-{}.rs", std::process::id()));
    let out = Command::new(&config.auto_binary)
        .args([
            "trans", "-p", lib_dir.to_str().ok_or("utf8 path")?, "rust", "--merge",
            "-o", tmp.to_str().ok_or("utf8 path")?,
        ])
        .output()
        .map_err(|e| format!("spawn auto trans: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "auto trans --merge failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let merged = std::fs::read(&tmp).map_err(|e| e.to_string())?;

    let hash = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        merged.hash(&mut h);
        format!("{:016x}", h.finish())
    };
    let proj = std::env::temp_dir().join(format!("aavm-parity-bin-{}", hash));
    let exe = proj.join("target/release/aavm2_bin.exe");
    if exe.exists() {
        return Ok((exe, format!("cache hit ({})", hash)));
    }

    let src_dir = proj.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    std::fs::write(
        proj.join("Cargo.toml"),
        "[package]\nname = \"aavm2_bin\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n\n[dependencies]\n",
    )
    .map_err(|e| e.to_string())?;
    let harness = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
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
    let mut full = merged.clone();
    full.extend_from_slice(harness.as_bytes());
    std::fs::write(src_dir.join("main.rs"), &full).map_err(|e| e.to_string())?;

    let build = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&proj)
        .output()
        .map_err(|e| format!("spawn cargo: {}", e))?;
    if !build.status.success() {
        return Err(format!(
            "AAVM-Rust cargo build failed:\n{}",
            String::from_utf8_lossy(&build.stderr)
        ));
    }
    Ok((exe, format!("built ({})", hash)))
}

fn run_aavm_rust(exe: &Path, case: &Path) -> Result<String, String> {
    let out = Command::new(exe)
        .arg(case)
        .output()
        .map_err(|e| format!("spawn aavm2 bin: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "aavm2 bin exited {:?}: {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// 跑四向矩阵。语料 = corpus_m4 全量(执行层稳定集)。
pub fn run_matrix(config: &AavmConfig) -> Result<MatrixReport, String> {
    let files = corpus_files(&config.repo_root)?;
    if files.is_empty() {
        return Err("no corpus files found".into());
    }
    let (bin, bin_note) = build_aavm_rust_bin(config)?;

    let mut cases = Vec::new();
    for path in &files {
        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let mut errors = Vec::new();

        let reference = match run_auto(config, path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("① reference: {}", e));
                String::new()
            }
        };
        let aavm_rust = match run_aavm_rust(&bin, path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("② aavm_rust: {}", e));
                String::new()
            }
        };
        let aavm_vm = match run_aavm_vm(config, path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("③ aavm_vm: {}", e));
                String::new()
            }
        };
        let golden_path = path.with_extension("expected.out");
        let (golden, golden_is_file) = if golden_path.is_file() {
            (std::fs::read_to_string(&golden_path).unwrap_or_default(), true)
        } else {
            (reference.clone(), false)
        };

        cases.push(MatrixCase {
            name,
            reference,
            aavm_rust,
            aavm_vm,
            golden,
            golden_is_file,
            errors,
        });
    }
    Ok(MatrixReport { cases, bin_note: format!("{} ({})", bin.display(), bin_note) })
}

/// 文本矩阵(终端输出)。
pub fn format_matrix(report: &MatrixReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "AAVM 四向对比矩阵(①reference ②aavm_rust ③aavm_vm ④golden)  [{}]\n",
        report.bin_note
    ));
    out.push_str(&format!(
        "{:<24} {:>9} {:>11} {:>9} {:>9}\n",
        "case", "①ref", "②aavm-rust", "③aavm-vm", "④golden"
    ));
    out.push_str(&format!("{}\n", "-".repeat(70)));
    for c in &report.cases {
        let mark = |eq: bool| if eq { "ok" } else { "DIFF" };
        out.push_str(&format!(
            "{:<24} {:>9} {:>11} {:>9} {:>9}\n",
            c.name,
            mark(c.reference.trim_end() == c.reference.trim_end()),
            mark(c.aavm_rust.trim() == c.reference.trim()),
            mark(c.aavm_vm.trim() == c.reference.trim()),
            mark(c.golden.trim() == c.reference.trim()),
        ));
    }
    let green = report.cases.iter().filter(|c| c.all_agree()).count();
    out.push_str(&format!(
        "\n自举稳定集: {}/{} 全绿{}\n",
        green,
        report.cases.len(),
        if report.all_green() { "" } else { "(存在差异,见上表 DIFF 行)" }
    ));
    // 差异详情
    for c in &report.cases {
        if c.all_agree() {
            continue;
        }
        out.push_str(&format!("\n==== {} ====\n", c.name));
        for e in &c.errors {
            out.push_str(&format!("  ERROR {}\n", e));
        }
        if c.aavm_rust.trim() != c.reference.trim() {
            out.push_str(&format!("--- ① reference ---\n{}\n--- ② aavm_rust ---\n{}\n", c.reference, c.aavm_rust));
        }
        if c.aavm_vm.trim() != c.reference.trim() {
            out.push_str(&format!("--- ① reference ---\n{}\n--- ③ aavm_vm ---\n{}\n", c.reference, c.aavm_vm));
        }
    }
    out
}

/// 独立 HTML 页(仪表盘的 aavm 一节,Plan 433 C1 "独立页"选项)。
pub fn write_html(report: &MatrixReport, path: &Path) -> Result<(), String> {
    let mut rows = String::new();
    for c in &report.cases {
        let cell = |eq: bool| {
            if eq { "<td class=\"ok\">ok</td>" } else { "<td class=\"diff\">DIFF</td>" }.to_string()
        };
        rows.push_str(&format!(
            "<tr><td>{}</td>{}{}{}{}</tr>\n",
            c.name,
            cell(true),
            cell(c.aavm_rust.trim() == c.reference.trim()),
            cell(c.aavm_vm.trim() == c.reference.trim()),
            cell(c.golden.trim() == c.reference.trim()),
        ));
    }
    let green = report.cases.iter().filter(|c| c.all_agree()).count();
    let total = report.cases.len();
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh"><head><meta charset="utf-8">
<title>AAVM 四向对比矩阵(Plan 433)</title>
<style>
body {{ font-family: system-ui, sans-serif; margin: 2rem; }}
table {{ border-collapse: collapse; }}
td, th {{ border: 1px solid #ccc; padding: 4px 12px; text-align: center; }}
td:first-child {{ text-align: left; font-family: monospace; }}
.ok {{ background: #e6f4e6; color: #1a5c1a; }}
.diff {{ background: #fde8e8; color: #a11; font-weight: bold; }}
.summary {{ margin: 1rem 0; font-size: 1.1rem; }}
</style></head><body>
<h1>AAVM 四向对比矩阵(Plan 433)</h1>
<p>① reference = auto-lang 原生实现(oracle) · ② aavm_rust = AAVM .at 经 a2r 转译编译后的
编译器+VM · ③ aavm_vm = AutoVM 解释执行 AAVM .at · ④ golden = corpus .expected.out
(无 golden 文件时回落 ①)。④golden 列在 golden 缺失时与 ① 同源,仅作占位。</p>
<p class="summary">自举稳定集:{green}/{total} 全绿({bin_note})</p>
<table>
<tr><th>case</th><th>① reference</th><th>② aavm_rust</th><th>③ aavm_vm</th><th>④ golden</th></tr>
{rows}
</table>
</body></html>"#,
        green = green,
        total = total,
        bin_note = report.bin_note,
        rows = rows,
    );
    std::fs::write(path, html).map_err(|e| e.to_string())
}
