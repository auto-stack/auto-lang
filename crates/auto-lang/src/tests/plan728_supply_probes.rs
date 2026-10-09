// PLAN-728 supply probes (710 形态) — 供⑮ 文件后援分页 rope 的 E2E/契约面。
//
// 家族：
//   1. `paged_load_edit_save_e2e`   —— native 面装载→摘要→编辑→保存→
//                                      外部修改拒绝 E2E（AC-07 主证）
//   2. `small_file_arm_golden`      —— 小文件臂逐字节原样（frozen ②，
//                                      双轨选择面：is_paged == false）
//   3. `api_signature_pin`          —— Rope/RopeSnapshot 公共 API 签名
//                                      零 diff 编译期钉（frozen ①，AC-06）
//   4. `paged_find_and_jump`        —— 查找/行号跳转 overlay 意识（面⑥
//                                      E2E：窗口重物化+光标落点）
//   5. `paged_memory_contract`      —— 结构计量机制（AC-03 小档位；
//                                      1GB 实测数字在 T-08 基准谱）
//
// 生成式 fixture 不入库（013/016 先例）：51MB 跨阈值档（>50MB 装载臂）
// 与小档都在运行时生成到 temp dir。

#![cfg(feature = "code-editor")]

use cosmic_text::Edit as _;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};
use std::time::Instant;

use crate::ui::code_editor::core::rope::{PageConfig, Rope, RopeSnapshot};
use crate::ui::code_editor::core::{
    code_editor, code_editor_dispose, code_editor_edit, code_editor_load_file, code_editor_save,
    code_editor_with, set_font_system_call, storage_key, with_font_system, PAGED_LOAD_THRESHOLD,
};

/// Process-wide FontSystem behind a RwLock (mirrors the iced adapter's
/// install; same shape as the core tests' `test_font_system`).
fn probe_font_system(with: &mut dyn FnMut(&mut cosmic_text::FontSystem)) {
    static FS: OnceLock<RwLock<cosmic_text::FontSystem>> = OnceLock::new();
    let fs = FS.get_or_init(|| RwLock::new(cosmic_text::FontSystem::new()));
    let mut guard = fs.write().unwrap();
    with(&mut guard);
}

struct TempFile(PathBuf);

impl TempFile {
    fn new(tag: &str, bytes: &[u8]) -> Self {
        let mut p = std::env::temp_dir();
        p.push(format!("p728-probe-{}-{}.txt", tag, std::process::id()));
        let mut w = std::io::BufWriter::new(std::fs::File::create(&p).unwrap());
        w.write_all(bytes).unwrap();
        w.flush().unwrap();
        drop(w);
        TempFile(p)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Deterministic line mix (ASCII/CJK/CRLF) — the same family the rope
/// unit tests use, at a given approximate total size.
fn gen_lines(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 128);
    let mut i = 0usize;
    while out.len() < target_bytes {
        match i % 5 {
            0 => out.push_str(&format!("line {i} plain ascii padding text\n")),
            1 => out.push_str(&format!("line {i} 中文多字节内容🦀 with padding\n")),
            2 => out.push_str(&format!("line {i} crlf line\r\n")),
            3 => out.push_str(&format!("line {i} émoji 🎉 mixed content here\n")),
            _ => out.push_str(&format!("line {i} tab\tsep padding text to fill\n")),
        }
        i += 1;
    }
    out
}

// ── 家族 1：native 面 装载→摘要→编辑→保存→外部修改拒绝 ────────────────

#[test]
fn paged_load_edit_save_e2e() {
    let _guard = crate::ui::code_editor::core::REGISTRY_TEST_LOCK
        .lock()
        .unwrap();
    set_font_system_call(probe_font_system);

    // 51MB > 50MB threshold → paged arm (AC-07 装载臂选择).
    let model = gen_lines(PAGED_LOAD_THRESHOLD as usize + 1024 * 1024);
    let src = TempFile::new("e2e-src", model.as_bytes());
    assert!(std::fs::metadata(src.path()).unwrap().len() > PAGED_LOAD_THRESHOLD);

    let key = storage_key("test-p728-e2e");
    code_editor_dispose(&key);
    code_editor(&key, &Default::default());

    let t0 = Instant::now();
    let loaded = code_editor_load_file(&key, src.path().to_str().unwrap());
    let load_wall = t0.elapsed();
    assert_eq!(
        loaded,
        Some(model.len() as i64),
        "paged load returns the file size"
    );
    eprintln!("[p728-e2e] load {}B in {load_wall:?}", model.len());

    // Arm selection + instant summaries (AC-01: 行数免调入即答).
    let (is_paged, lines, digest) = code_editor_with(&key, |c| {
        (
            c.is_paged(),
            c.doc_snapshot().line_count(),
            c.doc_snapshot().content_hash(),
        )
    })
    .expect("core");
    assert!(is_paged, ">50MB load must take the file-backed arm");
    let want_lines = model.bytes().filter(|&b| b == b'\n').count() + 1;
    assert_eq!(lines, want_lines, "line_count answers from the page table");
    assert_eq!(
        digest,
        Rope::content_hash_of(&model),
        "root digest folds prescan page digests"
    );

    // Structured edit FAR into the document (AC-02 over the registry face)
    // — offsets are full-document coordinates.
    let far_line = want_lines.saturating_sub(10);
    let far = code_editor_with(&key, |c| c.doc_snapshot().line_start_byte(far_line)).unwrap();
    assert!(
        code_editor_edit(&key, far, far, "// p728 inserted\n"),
        "far edit applies"
    );
    let mut edited = model.clone();
    edited.insert_str(far, "// p728 inserted\n");

    // Save → byte-for-byte (AC-05 over the registry face).
    let out = TempFile::new("e2e-out", b"");
    let _ = std::fs::remove_file(out.path());
    assert!(
        code_editor_save(&key, out.path().to_str().unwrap()),
        "merge save"
    );
    assert_eq!(
        std::fs::read(out.path()).unwrap(),
        edited.as_bytes(),
        "save is byte-for-byte"
    );

    // External mutation behind the rope's back → refusal, 报错不静默.
    std::fs::write(src.path(), b"externally replaced").unwrap();
    assert!(
        !code_editor_save(&key, src.path().to_str().unwrap()),
        "refuses mutated base"
    );
    assert_eq!(
        std::fs::read(src.path()).unwrap(),
        b"externally replaced",
        "original untouched by refusal"
    );

    code_editor_dispose(&key);
}

// ── 家族 2：小文件臂 golden（frozen ②）────────────────────────────────

#[test]
fn small_file_arm_golden() {
    let _guard = crate::ui::code_editor::core::REGISTRY_TEST_LOCK
        .lock()
        .unwrap();
    set_font_system_call(probe_font_system);

    let text = gen_lines(300 * 1024); // 300KB « 50MB
    let src = TempFile::new("small-src", text.as_bytes());
    let key = storage_key("test-p728-small");
    code_editor_dispose(&key);
    code_editor(&key, &Default::default());

    assert_eq!(
        code_editor_load_file(&key, src.path().to_str().unwrap()),
        Some(text.len() as i64)
    );
    let (is_paged, content) = code_editor_with(&key, |c| (c.is_paged(), c.text())).expect("core");
    assert!(
        !is_paged,
        "small files stay on the in-memory arm (frozen ②)"
    );
    assert_eq!(content, text, "golden: content identical byte-for-byte");

    let out = TempFile::new("small-out", b"");
    let _ = std::fs::remove_file(out.path());
    assert!(code_editor_save(&key, out.path().to_str().unwrap()));
    assert_eq!(
        std::fs::read(out.path()).unwrap(),
        text.as_bytes(),
        "701 direct write unchanged"
    );

    code_editor_dispose(&key);
}

// ── 家族 3：公共 API 签名零 diff 编译期钉（frozen ①）──────────────────

#[test]
fn api_signature_pin() {
    // RopeSnapshot — the frozen consumer face (673 find / 703 diff+hash /
    // highlight warm consumers compile against exactly these shapes).
    let _: fn(&RopeSnapshot) -> usize = RopeSnapshot::len_bytes;
    let _: fn(&RopeSnapshot) -> usize = RopeSnapshot::len_chars;
    let _: fn(&RopeSnapshot) -> usize = RopeSnapshot::line_count;
    let _: fn(&RopeSnapshot, usize) -> (usize, usize) = RopeSnapshot::byte_to_point;
    let _: fn(&RopeSnapshot, usize, usize) -> Option<usize> = RopeSnapshot::point_to_byte;
    let _: fn(&RopeSnapshot, usize) -> usize = RopeSnapshot::line_start_byte;
    let _: fn(&RopeSnapshot, usize) -> usize = RopeSnapshot::line_end_byte;
    let _: fn(&RopeSnapshot, usize, usize) -> std::borrow::Cow<'_, str> = RopeSnapshot::slice_bytes;
    let _: fn(&RopeSnapshot, usize) -> std::borrow::Cow<'_, str> = RopeSnapshot::line;
    let _: fn(&RopeSnapshot) -> String = RopeSnapshot::to_string;
    let _: fn(&RopeSnapshot) -> u64 = RopeSnapshot::content_hash;
    let _: fn(&RopeSnapshot, usize, usize) -> u64 = RopeSnapshot::range_hash;
    let _: fn(&RopeSnapshot, &RopeSnapshot) -> bool = RopeSnapshot::subtree_equal;
    // Rope — the editor-kernel document store.
    let _: fn() -> Rope = Rope::new;
    let _: fn(&str) -> Rope = Rope::from_str;
    let _: fn(&Rope) -> RopeSnapshot = Rope::snapshot;
    let _: fn(&Rope) -> usize = Rope::len_bytes;
    let _: fn(&Rope) -> usize = Rope::len_chars;
    let _: fn(&Rope) -> usize = Rope::line_count;
    let _: fn(&Rope, usize) -> (usize, usize) = Rope::byte_to_point;
    let _: fn(&Rope, usize, usize) -> Option<usize> = Rope::point_to_byte;
    let _: fn(&Rope, usize) -> usize = Rope::line_start_byte;
    let _: fn(&Rope, usize) -> usize = Rope::line_end_byte;
    let _: fn(&Rope, usize, usize) -> std::borrow::Cow<'_, str> = Rope::slice_bytes;
    let _: fn(&Rope, usize) -> std::borrow::Cow<'_, str> = Rope::line;
    let _: fn(&Rope) -> String = Rope::to_string;
    let _: fn(&Rope) -> u64 = Rope::content_hash;
    let _: fn(&Rope, usize, usize) -> u64 = Rope::range_hash;
    let _: fn(&Rope, &Rope) -> bool = Rope::subtree_equal;
    let _: fn(&mut Rope, usize, &str) = Rope::insert_bytes;
    let _: fn(&mut Rope, usize, usize) = Rope::delete_bytes;
    let _: fn(&mut Rope, usize, usize, &str) = Rope::replace_bytes;
    // Send + Sync face (snapshot isolation across threads).
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Rope>();
    assert_send_sync::<RopeSnapshot>();
}

// ── 家族 4：查找/行号跳转 overlay 意识（面⑥ E2E）─────────────────────

#[test]
fn paged_find_and_jump() {
    let _guard = crate::ui::code_editor::core::REGISTRY_TEST_LOCK
        .lock()
        .unwrap();
    set_font_system_call(probe_font_system);

    // Small paged doc installed DIRECTLY through the kernel face (the
    // registry threshold only guards the LOAD arm; the core machinery is
    // size-agnostic) — keeps the daily tier fast.
    let text = gen_lines(3 * 1024 * 1024);
    let src = TempFile::new("find-src", text.as_bytes());
    let key = storage_key("test-p728-find");
    code_editor_dispose(&key);
    code_editor(&key, &Default::default());

    let (rope, store) = Rope::open_file_backed(src.path(), PageConfig::default()).expect("open");
    with_font_system(|fs| {
        crate::ui::code_editor::core::code_editor_with(&key, |c| {
            c.set_doc_file_backed(rope, store, fs)
        })
        .expect("core");
    });
    let is_paged = code_editor_with(&key, |c| c.is_paged()).unwrap();
    assert!(is_paged);

    // Line jump far outside the head window: rewindow + cursor lands on
    // the target line's content.
    let total = code_editor_with(&key, |c| c.doc_snapshot().line_count()).unwrap();
    let far_line = total - 8;
    let want: String = text.lines().nth(far_line).unwrap().to_string();
    crate::ui::code_editor::core::code_editor_set_cursor(&key, far_line, 0);
    let (cursor_line, cursor_col, _) = code_editor_with(&key, |c| c.cursor_info()).unwrap();
    assert_eq!(cursor_col, 0, "jump lands at column 0");
    let got: String = code_editor_with(&key, |c| {
        let mut editor = c.editor_lock();
        editor.with_buffer_mut(|b| {
            b.lines
                .get_mut(cursor_line)
                .map(|l| l.text().to_string())
                .unwrap_or_default()
        })
    })
    .unwrap();
    assert_eq!(got, want, "rewindow covers the jumped-to line");

    // Regex search over the paged doc: a unique needle a few lines past
    // the caret (find_next scans FORWARD from the caret; a far needle
    // would make the DAILY probe pay the whole-document scan that the
    // bench谱 measures instead).
    let needle_line = 3usize;
    let needle = format!("needle-{needle_line}-🦀");
    let mut with_needle = text.clone();
    let byte_of_needle = {
        let mut off = 0usize;
        for (i, l) in text.lines().enumerate() {
            if i == needle_line {
                break;
            }
            off += l.len() + 1;
        }
        off
    };
    with_needle.insert_str(byte_of_needle, &format!("{needle}\n"));
    let src2 = TempFile::new("find-src2", with_needle.as_bytes());
    let (rope2, store2) =
        Rope::open_file_backed(src2.path(), PageConfig::default()).expect("open2");
    with_font_system(|fs| {
        code_editor_with(&key, |c| c.set_doc_file_backed(rope2, store2, fs)).expect("core");
    });
    // Reset the caret to the document head first: find_next scans forward
    // FROM THE CARET and wraps — after the jump test the caret sits deep
    // in the document, and a wrap-around would pay the whole-document
    // line scan (that cost is the bench谱's line, not the daily tier's).
    crate::ui::code_editor::core::code_editor_set_cursor(&key, 0, 0);
    assert!(
        code_editor_with(&key, |c| c.set_search(&needle)).unwrap(),
        "pattern set"
    );
    let found = with_font_system(|fs| code_editor_with(&key, |c| c.find_next(fs)).unwrap_or(false));
    assert!(found, "find locates the needle");
    let (_, _, sel_len) = code_editor_with(&key, |c| c.cursor_info()).unwrap();
    assert_eq!(sel_len, needle.len(), "needle selected (byte length)");

    code_editor_with(&key, |c| c.doc_replace("")).is_some();
    code_editor_dispose(&key);
}

// ── 家族 5：结构计量机制（AC-03 mechanics；1GB 档在 T-08 谱）─────────

#[test]
fn paged_memory_contract() {
    let text = gen_lines(30 * 1024 * 1024); // 30MB, default 6MB cache budget
    let src = TempFile::new("mem-src", text.as_bytes());
    let (rope, store) = Rope::open_file_backed(src.path(), PageConfig::default()).expect("open");

    // Sweep the whole document in 256KB chunks (char-boundary aligned) —
    // every page faults through the LRU; a per-LINE walk would rescan a
    // 64KB page's newlines per line (debug-build slow, no extra coverage).
    let t0 = Instant::now();
    let mut pos = 0usize;
    while pos < rope.len_bytes() {
        let mut end = (pos + 256 * 1024).min(rope.len_bytes());
        while end > pos && !rope.is_char_boundary(end) {
            end -= 1;
        }
        let _ = rope.slice_bytes(pos, end);
        pos = end;
    }
    let scan_wall = t0.elapsed();
    let cache = store.cache_bytes();
    let structural = rope.structural_resident_estimate() + store.structural_resident_bytes();
    eprintln!(
        "[p728-mem] 30MB doc: full line scan {scan_wall:?}, cache {cache}B, structural {structural}B"
    );
    assert!(
        cache <= 6 * 1024 * 1024 + 64 * 1024,
        "cache within budget: {cache}"
    );
    // 30MB档 structural well under the 12MB contract constant (1GB档 lands
    // in the bench谱; the 30MB numbers scale as table+tree ~ linearly).
    assert!(
        structural <= 12 * 1024 * 1024,
        "structural estimate {structural} over 12MB at 30MB doc"
    );

    // Summaries still O(1)-instant after the mass eviction churn.
    assert_eq!(
        rope.line_count(),
        text.bytes().filter(|&b| b == b'\n').count() + 1
    );
}
