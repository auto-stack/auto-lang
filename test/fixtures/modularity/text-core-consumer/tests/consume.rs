//! 外部消费面（AC-01/V-01/V-06）：rope 双轨、快照 diff、窗口投影、目录
//! 五态、envelope 错误值形——全部经 auto-text-core 公开面，零语言/UI 引用。

use auto_text_core::diff::envelope::{
    diff_files_envelope, diff_files_envelope_from_paths, diff_snapshots_envelope_snapshots,
    DiffSnapshotsPair,
};
use auto_text_core::diff::{diff_lines, diff_lines_windowed, DiffOpts, Hunk};
use auto_text_core::{PageConfig, PageState, Rope};
use std::io::Write as _;

fn temp_file(tag: &str, bytes: &[u8]) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("p807-consumer-{tag}-{}.txt", std::process::id()));
    let mut f = std::fs::File::create(&p).unwrap();
    f.write_all(bytes).unwrap();
    p
}

/// V-01/V-02 双轨：同一操作序列在内存 rope 与分页 rope 上内容/摘要一致，
/// 旧快照冻结，分页观测面可用。
#[test]
fn paged_and_memory_rope_agree() {
    let mut text = String::new();
    for i in 0..1200 {
        match i % 3 {
            0 => text.push_str(&format!("line {i} ascii padding\n")),
            1 => text.push_str(&format!("行 {i} 中文多字节🦀\n")),
            _ => text.push_str(&format!("line {i} crlf\r\n")),
        }
    }
    let bytes = text.clone().into_bytes();
    let path = temp_file("paged", &bytes);

    let cfg = PageConfig {
        page_bytes: 512,
        cache_budget: 8 * 1024,
        prefetch_radius: 2,
    };
    let (mut paged, store) = Rope::open_file_backed(&path, cfg).unwrap();
    let mut mem = Rope::from_str(&text);

    // 打开即答面（零物化）。
    assert_eq!(paged.len_bytes(), mem.len_bytes());
    assert_eq!(paged.line_count(), mem.line_count());
    assert_eq!(paged.content_hash(), mem.content_hash());

    // 同编辑序列双轨一致。
    let at = paged.line_start_byte(500);
    paged.replace_bytes(at, at + 6, "替换\n");
    mem.replace_bytes(at, at + 6, "替换\n");
    assert_eq!(paged.to_string(), mem.to_string());
    assert_eq!(paged.content_hash(), mem.content_hash());

    // 快照冻结 + 页观测。
    let snap = paged.snapshot();
    paged.insert_bytes(0, "// head\n");
    assert_eq!(snap.to_string(), mem.to_string());
    let _ = paged.line(3);
    assert!(matches!(store.page_state(0), PageState::Resident));

    let _ = std::fs::remove_file(&path);
}

/// V-05 diff 语义：文本/快照/窗口三入口同源（serial 输出确定性；窗口=
/// 全量结果投影）。
#[test]
fn diff_three_entries_single_source() {
    let a = "alpha\nbeta\ngamma\ndelta\n";
    let b = "alpha\nBETA\ngamma\ndelta\nepsilon\n";

    let out = diff_lines(a, b, DiffOpts { ctx: 1 });
    assert_eq!(out.adds, 2);
    assert_eq!(out.dels, 1);

    // 窗口投影 = 全量结果按窗裁剪（不重新 diff 切片）。
    let full = diff_lines(a, b, DiffOpts::default());
    let win = diff_lines_windowed(a, b, DiffOpts::default(), 0..2, 0..2);
    let want: Vec<Hunk> = full
        .hunks
        .iter()
        .filter(|h| h.a1 < 2 && 0 < h.a2)
        .map(|h| Hunk {
            a1: h.a1.max(0),
            a2: h.a2.min(2),
            b1: h.b1.max(0),
            b2: h.b2.min(2),
        })
        .collect();
    assert_eq!(win.hunks, want);

    // 快照形：同源 rope 快照 diff 与文本 diff 计数一致（hunk 边界合法差异
    // 边界不在此断言——见 crate 内 oracle）。
    let mut ra = Rope::from_str(a);
    let sa = ra.snapshot();
    ra.replace_bytes(6, 10, "BETA");
    let sb = ra.snapshot();
    let env = diff_snapshots_envelope_snapshots(DiffSnapshotsPair::Both(sa, sb));
    assert!(env.contains("\"adds\":1"), "{env}");
    assert!(env.contains("\"err\":\"\""), "{env}");

    // Missing 形 = 值错误（编辑器不存在）。
    let miss = diff_snapshots_envelope_snapshots(DiffSnapshotsPair::Missing("k".into()));
    assert!(miss.contains("\"err\":\"编辑器不存在: k\""), "{miss}");
    assert!(miss.contains("\"hunks\":[]"), "{miss}");
}

/// V-06 envelope 错误值形：缺文件 = err 字段（非 panic/Result）。
#[test]
fn envelope_missing_file_is_err_value() {
    let ghost =
        std::env::temp_dir().join(format!("p807-consumer-ghost-{}.nope", std::process::id()));
    let env = diff_files_envelope_from_paths(ghost.to_str().unwrap(), ghost.to_str().unwrap(), 3);
    assert!(env.contains("\"err\":\"文件不存在"), "{env}");
    assert!(env.contains("\"hunks\":[]"), "{env}");

    // 成功形态字段序（冻结）：默认形无 rows_total。
    let env = diff_files_envelope("a\n", "b\n", 1);
    assert!(env.starts_with("{\"hunks\":["));
    assert!(!env.contains("rows_total"), "默认形不携带 rows_total");
    assert!(env.ends_with("\"truncated\":false,\"degraded\":false,\"err\":\"\"}"));
}
