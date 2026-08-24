//! Plan 435 P3 —— widgets-gallery Vue 生成 golden 基线。
//!
//! 数据流翻转(render_support / vue import 映射改 schema 派生)的零回归闸门:
//! 对 gallery 全部 .at 源文件逐一生成 Vue SFC,输出全文做稳定哈希(blake3 不引入,
//! 用 FxHash 简化 —— 这里用 std 的 DefaultHasher 即可,golden 只在本机对比)
//! 与基线文件逐字节对比。
//!
//! 记录/更新基线(翻转前采样;更新必须人工复核 diff 并写明理由):
//! ```text
//! GALLERY_GOLDEN_UPDATE=1 cargo test -p auto-lang --test gallery_golden
//! ```

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

fn gallery_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/widgets-gallery/src")
}

fn collect_at_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_at_files(&p, out);
            } else if p.extension().map_or(false, |x| x == "at") {
                out.push(p);
            }
        }
    }
}

/// 单文件生成全文(与 `auto build` 同路径:generate_component_from_file)。
/// 仓库根的绝对路径(worktree 与主仓长度不同,输出偶有内嵌,必须归一)。
fn repo_root_display() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
        .display()
        .to_string()
}

/// 单文件生成全文(与 `auto build` 同路径)。输出做路径归一(仓库根 → <ROOT>):
/// golden 必须跨 worktree 可移植,否则内嵌绝对路径'换 worktree 必红'
/// (ui_snapshots 预存债同病根)。
fn generate(at: &Path) -> String {
    let opts = auto_lang::ui_gen::ComponentGenOptions::default();
    let root = repo_root_display();
    let root_fwd = root.replace('\\', "/");
    let raw = match auto_lang::ui_gen::generate_component_from_file(at, opts) {
        Ok(result) => {
            let mut out = String::new();
            // 稳定排序:widget 名 → (文件名, SFC 全文)
            let mut codes: Vec<_> = result.all_widget_codes.clone();
            codes.sort_by(|a, b| a.0.cmp(&b.0));
            for (name, code) in codes {
                out.push_str(&format!("==== {} ====\n", name));
                out.push_str(&code);
                out.push('\n');
            }
            out
        }
        Err(e) => format!("!!GENERATION ERROR!! {}", e),
    };
    // 错误信息里路径是 MANIFEST_DIR 拼接的非规范化形态(crates/x/../../..),
    // canonicalize 盖不住 —— 按原始前缀整体归一
    let manifest = env!("CARGO_MANIFEST_DIR").to_string();
    raw.replace(&manifest, "<CRATE>")
        .replace(&root, "<ROOT>")
        .replace(&root_fwd, "<ROOT>")
}

#[test]
fn gallery_vue_golden() {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_at_files(&gallery_dir(), &mut files);
    files.sort();
    assert!(!files.is_empty(), "gallery .at 源未找到");

    let baseline_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/gallery_vue_golden.txt");
    // 逐文件:相对路径 + 输出长度 + 稳定哈希(避免基线文件爆炸;全文对比在
    // 失败时通过 REGEN 提示做)。哈希碰撞风险对本用途可忽略。
    let mut report = String::new();
    let mut full_text = String::new();
    for f in &files {
        let rel = f
            .strip_prefix(gallery_dir())
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let code = generate(f);
        let mut h = DefaultHasher::new();
        code.hash(&mut h);
        report.push_str(&format!("{}\t{}\t{:016x}\n", rel, code.len(), h.finish()));
        full_text.push_str(&code);
        full_text.push('\n');
    }
    let mut fh = DefaultHasher::new();
    full_text.hash(&mut fh);
    report.push_str(&format!("TOTAL\t{}\t{:016x}\n", full_text.len(), fh.finish()));

    // 调试:GALLERY_GOLDEN_DUMP=<path> 导出全文,供两次运行 diff 定位非确定性
    if let Ok(dump) = std::env::var("GALLERY_GOLDEN_DUMP") {
        fs::write(&dump, &full_text).expect("write dump");
    }

    if std::env::var("GALLERY_GOLDEN_UPDATE").is_ok() {
        fs::write(&baseline_path, &report).expect("write golden baseline");
        panic!(
            "gallery golden 已重写({} 文件)—— 复核 diff 后重跑(不带环境变量)确认绿",
            files.len()
        );
    }
    let baseline = fs::read_to_string(&baseline_path).unwrap_or_else(|_| {
        panic!(
            "golden 基线缺失: {} —— 先采样:\n\
             GALLERY_GOLDEN_UPDATE=1 cargo test -p auto-lang --test gallery_golden",
            baseline_path.display()
        )
    });
    assert_eq!(
        baseline, report,
        "Plan 435 P3 golden 回归:widgets-gallery Vue 输出与基线不一致。\n\
         若属预期变更,复核后更新基线并写明理由:\n\
         GALLERY_GOLDEN_UPDATE=1 cargo test -p auto-lang --test gallery_golden"
    );
}
