//! V-01 坐标/快照（PLAN-787 T-07 矩阵 filter 族）：emoji/中文、LF/CRLF、跨块
//! 位置、错 base/UTF 边界/重叠 edit。语义断言与 runtime::snapshot 单测同源，
//! 此处为矩阵可发现的集成入口（含 position 往返）。

use ac_syntax_rt::position::LineIndex;
use ac_syntax_rt::program::RtError;
use ac_syntax_rt::snapshot::{Edit, EditRequest, PageStore, Snapshot};
use std::rc::Rc;

fn snap(text: &str) -> Snapshot {
    Snapshot::from_text(&Rc::new(PageStore::new()), text)
}

#[test]
fn snapshot_utf8_roundtrip_and_identity() {
    let a = snap("fn 中文(emoji 😀 x) {}");
    let b = snap("fn 中文(emoji 😀 x) {}");
    assert_eq!(a.id(), b.id());
    let s = a.read_str(0, a.len()).unwrap();
    assert_eq!(s, "fn 中文(emoji 😀 x) {}");
    // 非 UTF-8 插入拒绝。
    let bad = a.edit(&EditRequest { base_revision: 0, edits: vec![Edit { start: 0, end: 0, insert: vec![0xFF] }] });
    assert!(matches!(bad, Err(RtError::BadEdit(_))));
}

#[test]
fn snapshot_position_roundtrip_lf_crlf_utf16() {
    let text = "a😀b\r\nc中文\r\n尾行";
    let s = snap(text);
    let li = LineIndex::build(&s);
    // 行数：\n 计数 2 → 3 行。
    assert_eq!(li.line_count(), 3);
    // emoji 后偏移 5（a+😀4B+b）的 UTF-16 col：a(1)+😀(2)=3 → col 4。
    assert_eq!(li.line_col_utf16(&s, 5), (1, 4));
    // 代理对中点 floor：col 3（emoji 中间）→ 字节 1（emoji 起点）。
    assert_eq!(li.offset_of_utf16(&s, 1, 3), 1);
    // CRLF：\r 属行尾内容、\n 终行；第三行起点 = 第二个 \n 后。
    let (l3, _) = li.line_col_utf8(&s, text.find('尾').unwrap() as u64);
    assert_eq!(l3, 3);
}

#[test]
fn snapshot_cross_page_position_and_read() {
    let pb = ac_syntax_rt::snapshot::PAGE_BYTES;
    let text = format!("{}分页边界second", "头".repeat(pb / 2 + 10));
    let s = snap(&text);
    assert!(s.page_count() >= 2, "multi-page doc");
    let li = LineIndex::build(&s);
    let second_at = text.find("second").unwrap() as u64;
    let (l, _) = li.line_col_utf8(&s, second_at);
    assert_eq!(l, 1);
    assert_eq!(li.offset_of_utf8(1, 1), 0);
    // 跨页读取逐字节一致。
    let mid = s.read_bytes(pb as u64 / 2, pb as u64 / 2 + 40).unwrap();
    assert_eq!(mid.as_slice(), &text.as_bytes()[pb / 2..pb / 2 + 40]);
}

#[test]
fn snapshot_edit_rejections_keep_old_snapshot() {
    let s = snap("fn a() int { return 1 }");
    let before = s.id().clone();
    // 错 base。
    let r = s.edit(&EditRequest { base_revision: 9, edits: vec![Edit { start: 0, end: 0, insert: b"x".to_vec() }] });
    assert!(matches!(r, Err(RtError::BadEdit(_))));
    // 重叠。
    let r = s.edit(&EditRequest {
        base_revision: 0,
        edits: vec![
            Edit { start: 1, end: 4, insert: vec![] },
            Edit { start: 3, end: 6, insert: vec![] },
        ],
    });
    assert!(matches!(r, Err(RtError::BadEdit(_))));
    // emoji 标量中点。
    let s2 = snap("a😀b");
    let r = s2.edit(&EditRequest { base_revision: 0, edits: vec![Edit { start: 2, end: 3, insert: b"x".to_vec() }] });
    assert!(matches!(r, Err(RtError::BadEdit(_))));
    assert_eq!(*s.id(), before, "rejected edits must not mutate the old snapshot");
}
