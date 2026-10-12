// PLAN-728 T-08 基准谱 — 文件后援分页 rope 的阶梯基准（生成式 fixture 不入库）。
//
// 运行形态（显式，不进日常档）：
//   AUTO_LANG_P728_BENCH=512m,1g cargo test -p auto-lang --lib plan728_bench -- --ignored --nocapture
//   （缺省档位 = 50m,512m,1g；AUTO_LANG_P728_BENCH=off 跳过生成大档）
//
// 四线谱（每档位一行 JSONL，落 docs/reports/p728-bench.jsonl）：
//   load   —— open_file_backed 墙钟（预扫+页表+树构建）
//   answer —— 打开即答面（line_count/content_hash O(1) 记帐 + 远跳 line_start_byte
//             + 窗口行读×101 —— 滚动形状）
//   edit   —— 远端 insert×50（fault+split+concat 带宽）
//   save   —— write_backed 合并写墙钟（未改区段磁盘到磁盘照抄）
// 另两线：
//   resident —— 结构计量（树节点+页表+页缓存）@答案/编辑后（RSS 实测由
//               scripts/measure_test_mem.py 进程峰值轮询补，见谱注）
//   typing   —— 小文件（5MB 内存 rope）逐键带宽 ×200 + 同操作在 1GB 后援形
//               上的对照（725 后 1-4ms 带的内核侧不劣化证）

#![cfg(feature = "code-editor")]

use std::io::Write as _;
use std::path::PathBuf;
use std::time::Instant;

use crate::ui::code_editor::core::rope::{PageConfig, Rope};

struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// ~46B 行混合语料（ASCII/中文/emoji），target 按字节计。
fn gen_fixture(path: &std::path::Path, target_bytes: usize) -> usize {
    let f = std::fs::File::create(path).unwrap();
    let mut w = std::io::BufWriter::with_capacity(4 * 1024 * 1024, f);
    let mut written = 0usize;
    let mut i = 0usize;
    let mut buf = String::new();
    while written < target_bytes {
        buf.clear();
        match i % 5 {
            0 => buf.push_str(&format!("line {i} plain ascii padding to fill the page\n")),
            1 => buf.push_str(&format!("行 {i} 中文多字节内容带填充文本🦀\n")),
            2 => buf.push_str(&format!("line {i} crlf padding line\r\n")),
            3 => buf.push_str(&format!("line {i} émoji 🎉 mixed content padding\n")),
            _ => buf.push_str(&format!("line {i} tab\tsep padding text filler line\n")),
        }
        w.write_all(buf.as_bytes()).unwrap();
        written += buf.len();
        i += 1;
    }
    w.flush().unwrap();
    written
}

fn now_ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn parse_tier(spec: &str) -> usize {
    let spec = spec.trim().to_ascii_lowercase();
    let (num, unit) = spec.split_at(spec.len().saturating_sub(1));
    let n: usize = num.parse().unwrap_or(0);
    match unit {
        "g" => n * 1024 * 1024 * 1024,
        "m" => n * 1024 * 1024,
        _ => 0,
    }
}

fn report(line: String) {
    println!("[p728-bench] {line}");
    // CARGO_MANIFEST_DIR = <repo>/crates/auto-lang → repo/docs/reports.
    let mut p: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    p.push("../../docs/reports");
    let _ = std::fs::create_dir_all(&p);
    p.push("p728-bench.jsonl");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&p)
    {
        let _ = writeln!(f, "{line}");
    }
}

#[test]
#[ignore = "bench 谱（显式跑）: AUTO_LANG_P728_BENCH=1g cargo test -p auto-lang --lib plan728_bench -- --ignored --nocapture"]
fn p728_bench_ladder() {
    let tiers = std::env::var("AUTO_LANG_P728_BENCH").unwrap_or_else(|_| "50m,512m,1g".into());
    for tier in tiers.split(',').filter(|s| !s.is_empty()) {
        let target = parse_tier(tier);
        if target == 0 {
            continue;
        }
        let mut src: PathBuf = std::env::temp_dir().into();
        src.push(format!("p728-bench-{}-{}.txt", tier, std::process::id()));
        let _src_tmp = TempFile(src.clone());

        let t = Instant::now();
        let written = gen_fixture(&src, target);
        let gen_ms = now_ms(t);

        // ── load line ──
        let t = Instant::now();
        let (mut rope, store) = Rope::open_file_backed(&src, PageConfig::default()).expect("open");
        let load_ms = now_ms(t);

        // ── answer line (打开即答 + 远跳 + 滚动窗) ──
        let t = Instant::now();
        let lines = rope.line_count();
        let digest = rope.content_hash();
        let far = rope.line_start_byte(lines.saturating_sub(10));
        let far_line = rope.byte_to_point(far).0;
        let mut window_bytes = 0usize;
        for i in far_line.saturating_sub(50)..(far_line + 51).min(lines) {
            window_bytes += rope.line(i).len();
        }
        let answer_ms = now_ms(t);
        assert!(window_bytes > 0);
        assert_eq!(rope.len_bytes(), written);

        // ── edit line (远端插入×50：fault+split+concat) ──
        let mut edit_max = 0.0f64;
        let t_all = Instant::now();
        for k in 0..50u32 {
            let t1 = Instant::now();
            let at = rope.line_start_byte(lines.saturating_sub(10 + k as usize));
            rope.insert_bytes(at, &format!("// p728 bench edit {k}\n"));
            edit_max = edit_max.max(now_ms(t1));
        }
        let edit_ms = now_ms(t_all);

        // ── resident line (结构计量；RSS 实测=进程峰值轮询另补) ──
        let structural = rope.structural_resident_estimate() + store.structural_resident_bytes();

        // ── save line (合并写：未改区段照抄+增量写回+原子改名) ──
        let mut dst: PathBuf = std::env::temp_dir().into();
        dst.push(format!(
            "p728-bench-out-{}-{}.txt",
            tier,
            std::process::id()
        ));
        let _dst_tmp = TempFile(dst.clone());
        let t = Instant::now();
        let saved = rope.write_backed(&dst).expect("save");
        let save_ms = now_ms(t);
        assert_eq!(saved as usize, rope.len_bytes());

        report(format!(
            "{{\"tier\":\"{tier}\",\"bytes\":{written},\"lines\":{lines},\"gen_ms\":{gen_ms:.1},\
             \"load_ms\":{load_ms:.1},\"answer_ms\":{answer_ms:.3},\"digest\":\"{digest:#x}\",\
             \"edit_50_ms\":{edit_ms:.1},\"edit_max_ms\":{edit_max:.3},\
             \"structural_resident_bytes\":{structural},\"save_ms\":{save_ms:.1},\
             \"cache_bytes\":{}}}",
            store.cache_bytes()
        ));
    }
}

/// 小文件键入带（725 后 1-4ms 带的内核侧不劣化证）：同一操作序列在
/// （a）5MB 内存 rope（frozen ② 小文件臂）与（b）阶梯最大档的后援形上
/// 的逐键分布对照。
#[test]
#[ignore = "bench 谱（显式跑）: AUTO_LANG_P728_BENCH=1g cargo test -p auto-lang --lib plan728_bench -- --ignored --nocapture"]
fn p728_bench_typing_band() {
    let tiers = std::env::var("AUTO_LANG_P728_BENCH").unwrap_or_else(|_| "1g".into());
    let big = tiers
        .split(',')
        .map(parse_tier)
        .filter(|&n| n > 0)
        .max()
        .unwrap_or(0);

    // (a) small in-memory arm.
    let text = {
        let mut s = String::with_capacity(5 * 1024 * 1024);
        let mut i = 0;
        while s.len() < 5 * 1024 * 1024 {
            s.push_str(&format!("line {i} typing band fixture content padding\n"));
            i += 1;
        }
        s
    };
    let mut small = Rope::from_str(&text);
    let mut at = small.len_bytes() / 2;
    let mut times: Vec<f64> = Vec::with_capacity(200);
    for k in 0..200u32 {
        let t = Instant::now();
        let ins = format!("edit {k} ");
        small.insert_bytes(at, &ins);
        times.push(now_ms(t));
        at += ins.len();
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (p50, p95, mx) = (times[100], times[190], times[199]);
    report(format!(
        "{{\"tier\":\"typing-small-5m\",\"edits\":200,\"p50_ms\":{p50:.4},\"p95_ms\":{p95:.4},\"max_ms\":{mx:.4}}}"
    ));

    // (b) paged arm at the ladder's top tier (skip when off).
    if big >= 50 * 1024 * 1024 {
        let mut src: PathBuf = std::env::temp_dir().into();
        src.push(format!("p728-bench-type-{}.txt", std::process::id()));
        let _tmp = TempFile(src.clone());
        gen_fixture(&src, big);
        let (mut rope, _store) = Rope::open_file_backed(&src, PageConfig::default()).expect("open");
        let lines = rope.line_count();
        let mut at = rope.line_start_byte(lines / 2);
        let mut times: Vec<f64> = Vec::with_capacity(200);
        for k in 0..200u32 {
            let t = Instant::now();
            let ins = format!("edit {k} ");
            rope.insert_bytes(at, &ins);
            times.push(now_ms(t));
            at += ins.len();
        }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (p50, p95, mx) = (times[100], times[190], times[199]);
        report(format!(
            "{{\"tier\":\"typing-paged-{}\",\"edits\":200,\"p50_ms\":{p50:.4},\"p95_ms\":{p95:.4},\"max_ms\":{mx:.4}}}",
            std::env::var("AUTO_LANG_P728_BENCH").unwrap_or_default()
        ));
    }
}
