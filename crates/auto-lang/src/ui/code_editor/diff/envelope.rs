// PLAN-703 T-06 — envelope builders for the VM/back consumption endpoints
// (supply pack 供⑤). The JSON shapes are FIELD-IDENTICAL to the downstream
// replacement-seam contracts (auto-edit diff-view.md SD-01, PLAN-011/012):
// the downstream side swaps its back implementation body for these natives
// with zero envelope/view/matrix changes.
//
// diff_files:  {hunks:[{a1,a2,b1,b2}], rows:[{lo,ro,ln,rn,lk,rk,lpre,lmid,
//              lpost,rpre,rmid,rpost}], adds, dels, truncated, degraded, err}
//              rows: lo/ro 1-based (absent side 0); lk/rk ∈ ctx/del/add/"";
//              three-segment marking (paired rows common prefix/suffix trim;
//              ctx rows carry the full text in lpre; unpaired rows whole
//              line in mid — Q-4 form). CR tolerated (universal newlines).
//              truncated v1 恒 false; degraded 恒 false (engine era has no
//              degradation semantics — the field survives for downstream
//              compatibility). Missing file → err form, hunks/rows empty.
// diff_dirs:   {entries:[{rel,status,size_a,size_b,is_dir,note}],
//              counts:{same,added,deleted,modified,binary}, truncated, err}
//              entry cap 5000 → truncated=true (counts stay same-domain).
// diff_snapshots: hunk net form (rows are a file-face rendering projection;
//              the buffer face feeds hunk navigation), same counts/err.
//
// Errors are VALUES (the err field), never raises — the 669/687 endpoint
// family convention.

use std::path::Path;

use super::dirs::{DirDiffError, DirDiffIter, DirDiffOptions};
use super::{engine_changes, group_hunks_annotated, intern_lines, split_lines_universal, Change, DiffOpts};

/// Entry cap (downstream T-00 定参).
const DIR_ENTRY_CAP: usize = 5000;

/// Minimal JSON string escaper (hand-built small envelopes — no value-model
/// round trip needed).
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn err_envelope(message: &str) -> String {
    format!(
        "{{\"hunks\":[],\"rows\":[],\"adds\":0,\"dels\":0,\"truncated\":false,\"degraded\":false,\"err\":{}}}",
        json_str(message)
    )
}

fn hunks_json(out: &mut String, hunks: &[super::Hunk]) {
    out.push_str("{\"hunks\":[");
    for (i, h) in hunks.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("{{\"a1\":{},\"a2\":{},\"b1\":{},\"b2\":{}}}", h.a1, h.a2, h.b1, h.b2));
    }
    out.push(']');
}

/// Build the diff_files envelope for two texts (the core both the path and
/// snapshot faces share for row semantics).
pub fn diff_files_envelope(a_text: &str, b_text: &str, ctx: usize) -> String {
    let a_lines = split_lines_universal(a_text);
    let b_lines = split_lines_universal(b_text);
    let inp = intern_lines(&a_lines, &b_lines);
    let changes = engine_changes(&inp, false);
    let adds = changes.iter().filter(|c| !c.del).count();
    let dels = changes.iter().filter(|c| c.del).count();
    let grouped = group_hunks_annotated(&changes, a_lines.len(), b_lines.len(), ctx);
    let rows = build_rows(&changes, &grouped, &a_lines, &b_lines);

    let mut out = String::with_capacity(1024 + rows.len() * 96);
    hunks_json(&mut out, &grouped.iter().map(|g| g.hunk).collect::<Vec<_>>());
    out.push_str(",\"rows\":[");
    for (i, r) in rows.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&r.to_json());
    }
    out.push_str(&format!(
        "],\"adds\":{adds},\"dels\":{dels},\"truncated\":false,\"degraded\":false,\"err\":\"\"}}"
    ));
    out
}

/// PLAN-716 组C: rows 窗口投影（diff-engine-supply §6 want 升格——021 L2
/// 直拉 armed FAIL 5183.2ms 的清偿路径）。窗口调用形增 `rows_total` 字段
/// （全量行计数）+`truncated` 激活语义（=offset+返回行数 < rows_total）；
/// **默认三参调用（上函数）逐字节等价**——不增字段（016 消费面 frozen ③
/// 最严形）。hunks 全量保持（导航域）；rows 窗外行不物化 Row 结构
/// （100MB 散点对主耗=rows 投影+双层 JSON——跳过即收益）。
///
/// `rows_offset` 越界（≥rows_total）→ `rows:[]`（diff_snapshots 净形复用）
/// + truncated = rows_total > 0；limit 0 同形。
#[allow(clippy::too_many_arguments)]
pub fn diff_files_envelope_window(
    a_text: &str,
    b_text: &str,
    ctx: usize,
    rows_offset: usize,
    rows_limit: usize,
) -> String {
    let a_lines = split_lines_universal(a_text);
    let b_lines = split_lines_universal(b_text);
    let inp = intern_lines(&a_lines, &b_lines);
    let changes = engine_changes(&inp, false);
    let adds = changes.iter().filter(|c| !c.del).count();
    let dels = changes.iter().filter(|c| c.del).count();
    let grouped = group_hunks_annotated(&changes, a_lines.len(), b_lines.len(), ctx);
    let (rows, rows_total) =
        build_rows_windowed(&changes, &grouped, &a_lines, &b_lines, rows_offset, rows_limit);
    // truncated 语义（011 保留字段激活）：有行被省略即置位——头部省略
    // （offset>0）与尾部省略（offset+len<total）都算。
    let truncated = rows_offset > 0 || rows_offset + rows.len() < rows_total;

    let mut out = String::with_capacity(1024 + rows.len() * 96);
    hunks_json(&mut out, &grouped.iter().map(|g| g.hunk).collect::<Vec<_>>());
    out.push_str(",\"rows\":[");
    for (i, r) in rows.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&r.to_json());
    }
    out.push_str(&format!(
        "],\"adds\":{adds},\"dels\":{dels},\"rows_total\":{rows_total},\"truncated\":{truncated},\"degraded\":false,\"err\":\"\"}}"
    ));
    out
}

/// Endpoint face: read both files and envelope them. Missing files → err
/// form (empty hunks/rows), no raise.
pub fn diff_files_envelope_from_paths(path_a: &str, path_b: &str, ctx: usize) -> String {
    let ctx = if ctx == 0 { 3 } else { ctx };
    if !Path::new(path_a).is_file() {
        return err_envelope(&format!("文件不存在: {path_a}"));
    }
    if !Path::new(path_b).is_file() {
        return err_envelope(&format!("文件不存在: {path_b}"));
    }
    match (std::fs::read_to_string(path_a), std::fs::read_to_string(path_b)) {
        (Ok(a), Ok(b)) => diff_files_envelope(&a, &b, ctx),
        (Err(e), _) | (_, Err(e)) => err_envelope(&format!("读取失败: {e}")),
    }
}

/// 窗口形 endpoint face（语义同上——窗口参数缺省语义由调用方钳定：
/// offset≥total → 净形，limit 0 → 空）。
pub fn diff_files_envelope_from_paths_window(
    path_a: &str,
    path_b: &str,
    ctx: usize,
    rows_offset: usize,
    rows_limit: usize,
) -> String {
    let ctx = if ctx == 0 { 3 } else { ctx };
    if !Path::new(path_a).is_file() {
        return err_envelope(&format!("文件不存在: {path_a}"));
    }
    if !Path::new(path_b).is_file() {
        return err_envelope(&format!("文件不存在: {path_b}"));
    }
    match (std::fs::read_to_string(path_a), std::fs::read_to_string(path_b)) {
        (Ok(a), Ok(b)) => diff_files_envelope_window(&a, &b, ctx, rows_offset, rows_limit),
        (Err(e), _) | (_, Err(e)) => err_envelope(&format!("读取失败: {e}")),
    }
}

/// One render-ready row (the 12-field envelope shape).
struct Row {
    lo: usize,
    ro: usize,
    ln: String,
    rn: String,
    lk: &'static str,
    rk: &'static str,
    lpre: String,
    lmid: String,
    lpost: String,
    rpre: String,
    rmid: String,
    rpost: String,
}

impl Row {
    fn to_json(&self) -> String {
        format!(
            "{{\"lo\":{},\"ro\":{},\"ln\":{},\"rn\":{},\"lk\":{},\"rk\":{},\"lpre\":{},\"lmid\":{},\"lpost\":{},\"rpre\":{},\"rmid\":{},\"rpost\":{}}}",
            self.lo,
            self.ro,
            json_str(&self.ln),
            json_str(&self.rn),
            json_str(self.lk),
            json_str(self.rk),
            json_str(&self.lpre),
            json_str(&self.lmid),
            json_str(&self.lpost),
            json_str(&self.rpre),
            json_str(&self.rmid),
            json_str(&self.rpost),
        )
    }
}

fn ctx_row(lo: usize, ro: usize, text: &str) -> Row {
    Row {
        lo,
        ro,
        ln: text.to_string(),
        rn: text.to_string(),
        lk: "ctx",
        rk: "ctx",
        lpre: text.to_string(),
        lmid: String::new(),
        lpost: String::new(),
        rpre: text.to_string(),
        rmid: String::new(),
        rpost: String::new(),
    }
}

/// Char-indexed three-segment split: `[0, mid_start)` + `[mid_start,
/// mid_end)` + `[mid_end, len)`.
fn slice_three(s: &str, mid_start: usize, mid_end: usize) -> (String, String, String) {
    let chars: Vec<char> = s.chars().collect();
    let at = |from: usize, to: usize| chars.get(from..to).map(|c| c.iter().collect()).unwrap_or_default();
    (at(0, mid_start), at(mid_start, mid_end), at(mid_end, chars.len()))
}

/// Expand the change script + annotated hunks into render-ready rows
/// (downstream ⑤: per-hunk stream slices; contiguous change blocks get
/// index-aligned del/add pairing with three-segment marking).
fn build_rows(
    changes: &[Change],
    grouped: &[super::GroupedHunk],
    a_lines: &[&str],
    b_lines: &[&str],
) -> Vec<Row> {
    build_rows_windowed(changes, grouped, a_lines, b_lines, 0, usize::MAX).0
}

/// 单趟流窗口投影（PLAN-716 组C——与 build_rows 同一走线，计数全量行数，
/// 仅物化 [offset, offset+limit) 窗内的 Row 结构。走线单源：窗口形与全量形
/// 行序/行数恒一致（count 断言测试钉住）。
fn build_rows_windowed(
    changes: &[Change],
    grouped: &[super::GroupedHunk],
    a_lines: &[&str],
    b_lines: &[&str],
    rows_offset: usize,
    rows_limit: usize,
) -> (Vec<Row>, usize) {
    // Full stream (keeps + changes) with absolute coordinates — the slice
    // arithmetic below indexes it. Kind is tri-state (0 keep / 1 del /
    // 2 add): a bool would render adds as ctx rows.
    const KEEP: u8 = 0;
    const DEL: u8 = 1;
    const ADD: u8 = 2;
    let mut stream: Vec<(u8, usize, usize)> = Vec::with_capacity(a_lines.len() + b_lines.len());
    let (mut i, mut j) = (0usize, 0usize);
    // PLAN-704 D-1: per-change stream positions. `GroupedHunk.fc/lc` are
    // indices into `changes` (mod.rs grouping), while the slice arithmetic
    // below walks the keep+change stream — the two index spaces diverge as
    // soon as keeps precede/intersperse changes (multi-hunk and pure
    // add/del families), which used to drop change rows and duplicate
    // leading context. Record each change's stream slot at push time.
    let mut chg_stream: Vec<usize> = Vec::with_capacity(changes.len());
    for c in changes {
        // Keeps fill only while BOTH cursors can advance together (paired
        // lines). A `||` here would fabricate a mismatched keep between a
        // del and a following add (they are adjacent in the script).
        while i < c.i && j < c.j {
            stream.push((KEEP, i, j));
            i += 1;
            j += 1;
        }
        chg_stream.push(stream.len());
        stream.push((if c.del { DEL } else { ADD }, c.i, c.j));
        if c.del {
            i += 1;
        } else {
            j += 1;
        }
    }
    while i < a_lines.len() && j < b_lines.len() {
        stream.push((KEEP, i, j)); // tail keeps (paired)
        i += 1;
        j += 1;
    }

    let window_end = rows_offset.saturating_add(rows_limit);
    let mut rows = Vec::new();
    let mut row_idx = 0usize;
    // 单 Row 物化闭包：窗口判定+计数单源。
    let mut push_row = |rows: &mut Vec<Row>, row_idx: &mut usize, row: Row| {
        let idx = *row_idx;
        *row_idx += 1;
        if idx >= rows_offset && idx < window_end {
            rows.push(row);
        }
    };
    for g in grouped {
        let h = &g.hunk;
        // Stream slice: leading keeps from the hunk start to the first
        // change, and trailing keeps up to the hunk bounds (semantic walk —
        // the closed-form arithmetic loses the trailing ctx rows).
        // fc/lc are changes-indices; chg_stream maps them to stream slots
        // (PLAN-704 D-1 single-sourcing — the downstream 011 contract reads
        // these as stream positions).
        let lo = chg_stream[g.fc].saturating_sub(g.fi.saturating_sub(h.a1));
        const KEEP: u8 = 0;
        let mut hi = chg_stream[g.lc] + 1;
        // The change block that lc closes (lc is its LAST change — already
        // inside; the walk covers a mid-block change when grouping merged
        // runs), then the trailing ctx keeps within the hunk bounds.
        while hi < stream.len() && stream[hi].0 != KEEP {
            hi += 1;
        }
        while hi < stream.len() && stream[hi].0 == KEEP && stream[hi].1 < h.a2 && stream[hi].2 < h.b2 {
            hi += 1;
        }
        let mut k = lo;
        while k < hi {
            let (kind, si, sj) = stream[k];
            if kind == KEEP {
                let text = a_lines[si];
                push_row(&mut rows, &mut row_idx, ctx_row(si + 1, sj + 1, text));
                k += 1;
                continue;
            }
            // Contiguous change block [k, block_end).
            let mut block_end = k;
            while block_end < hi && stream[block_end].0 != KEEP {
                block_end += 1;
            }
            let mut dels: Vec<usize> = Vec::new();
            let mut adds: Vec<usize> = Vec::new();
            for &(kind, si, sj) in &stream[k..block_end] {
                if kind == DEL {
                    dels.push(si);
                } else {
                    adds.push(sj);
                }
            }
            k = block_end;
            // Index-aligned pairing with three-segment marking (engine era:
            // refine always — the downstream 100k char budget was a VM step
            // guard that has no Rust-side reason to exist).
            let n = dels.len().max(adds.len());
            for idx in 0..n {
                match (dels.get(idx), adds.get(idx)) {
                    (Some(&di), Some(&aj)) => {
                        let (x, y) = (a_lines[di], b_lines[aj]);
                        let r = super::refine_inline(x, y);
                        let (xc, yc) = (x.chars().count(), y.chars().count());
                        let (lpre, lmid, lpost) = slice_three(x, r.pre, xc - r.post);
                        let (rpre, rmid, rpost) = slice_three(y, r.pre, yc - r.post);
                        push_row(
                            &mut rows,
                            &mut row_idx,
                            Row {
                                lo: di + 1,
                                ro: aj + 1,
                                ln: x.to_string(),
                                rn: y.to_string(),
                                lk: "del",
                                rk: "add",
                                lpre,
                                lmid,
                                lpost,
                                rpre,
                                rmid,
                                rpost,
                            },
                        );
                    }
                    (Some(&di), None) => {
                        let x = a_lines[di];
                        push_row(
                            &mut rows,
                            &mut row_idx,
                            Row {
                                lo: di + 1,
                                ro: 0,
                                ln: x.to_string(),
                                rn: String::new(),
                                lk: "del",
                                rk: "",
                                lpre: String::new(),
                                lmid: x.to_string(),
                                lpost: String::new(),
                                rpre: String::new(),
                                rmid: String::new(),
                                rpost: String::new(),
                            },
                        );
                    }
                    (None, Some(&aj)) => {
                        let y = b_lines[aj];
                        push_row(
                            &mut rows,
                            &mut row_idx,
                            Row {
                                lo: 0,
                                ro: aj + 1,
                                ln: String::new(),
                                rn: y.to_string(),
                                lk: "",
                                rk: "add",
                                lpre: String::new(),
                                lmid: String::new(),
                                lpost: String::new(),
                                rpre: String::new(),
                                rmid: y.to_string(),
                                rpost: String::new(),
                            },
                        );
                    }
                    (None, None) => unreachable!(),
                }
            }
        }
    }
    (rows, row_idx)
}

/// Build the diff_dirs envelope (cap 5000, counts same-domain, err not
/// silent on missing roots).
pub fn diff_dirs_envelope(path_a: &str, path_b: &str) -> String {
    match DirDiffIter::new(Path::new(path_a), Path::new(path_b), DirDiffOptions::default()) {
        Err(DirDiffError::RootMissing(p)) => format!(
            "{{\"entries\":[],\"counts\":{{\"same\":0,\"added\":0,\"deleted\":0,\"modified\":0,\"binary\":0}},\"truncated\":false,\"err\":{}}}",
            json_str(&format!("目录不存在: {p}"))
        ),
        Ok(mut iter) => {
            let mut out = String::with_capacity(4096);
            out.push_str("{\"entries\":[");
            let mut total = 0usize;
            let mut truncated = false;
            for e in &mut iter {
                if total >= DIR_ENTRY_CAP {
                    truncated = true;
                    break;
                }
                if total > 0 {
                    out.push(',');
                }
                out.push_str(&format!(
                    "{{\"rel\":{},\"status\":{},\"size_a\":{},\"size_b\":{},\"is_dir\":{},\"note\":{}}}",
                    json_str(&e.rel),
                    json_str(e.status.as_str()),
                    e.size_a,
                    e.size_b,
                    e.is_dir,
                    json_str(&e.note),
                ));
                total += 1;
            }
            let c = iter.counts();
            out.push_str(&format!(
                "],\"counts\":{{\"same\":{},\"added\":{},\"deleted\":{},\"modified\":{},\"binary\":{}}},\"truncated\":{},\"err\":\"\"}}",
                c.same, c.added, c.deleted, c.modified, c.binary, truncated
            ));
            out
        }
    }
}

/// `diff_snapshots(key_a, key_b)` — direct buffer-registry read (editing
/// buffer comparison; zero full-text VM transit). Missing keys → err form.
pub fn diff_snapshots_envelope(key_a: &str, key_b: &str) -> String {
    let sa = editor_snapshot(key_a);
    let sb = editor_snapshot(key_b);
    let (Some(sa), Some(sb)) = (sa, sb) else {
        let missing = if editor_snapshot(key_a).is_none() { key_a } else { key_b };
        return err_envelope(&format!("编辑器不存在: {missing}"));
    };
    let out = super::diff_snapshots(&sa, &sb, DiffOpts::default());
    let mut s = String::with_capacity(512);
    hunks_json(&mut s, &out.hunks);
    s.push_str(&format!(
        ",\"rows\":[],\"adds\":{},\"dels\":{},\"truncated\":false,\"degraded\":false,\"err\":\"\"}}",
        out.adds, out.dels
    ));
    s
}

fn editor_snapshot(key: &str) -> Option<crate::ui::code_editor::core::rope::RopeSnapshot> {
    crate::ui::code_editor::code_editor_with(key, |core| core.doc_snapshot())
}

// ── PLAN-704: D-1 defect-fix regression (rows slice single-sourcing) ───────
//
// `build_rows` used to consume `GroupedHunk.fc/lc` (changes-indices) as
// stream slots — shapes whose changes follow keeps lost their change rows
// entirely (pure add: "adds=3, zero add rows") and multi-hunk shapes
// duplicated leading context (scattered: 41 rows vs 21). The per-change
// stream-position map restores the downstream 011 reference semantics;
// these tests pin the four drifted families.

#[cfg(test)]
mod plan704_rows {
    use super::super::Hunk;
    use super::*;

    fn numbered(tag: &str, n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("{tag}{i}")).collect()
    }

    fn strings(v: &[String]) -> Vec<&str> {
        v.iter().map(|s| s.as_str()).collect()
    }

    fn envelope_rows(
        a: &[&str],
        b: &[&str],
        ctx: usize,
    ) -> (Vec<Hunk>, Vec<Row>, usize, usize) {
        let inp = super::super::intern_lines(a, b);
        let changes = super::super::engine_changes(&inp, false);
        let adds = changes.iter().filter(|c| !c.del).count();
        let dels = changes.iter().filter(|c| c.del).count();
        let grouped =
            super::super::group_hunks_annotated(&changes, a.len(), b.len(), ctx);
        let rows = build_rows(&changes, &grouped, a, b);
        let hunks = grouped.iter().map(|g| g.hunk).collect();
        (hunks, rows, adds, dels)
    }

    #[test]
    fn d1_multihunk_rows_match_downstream_reference() {
        // downstream scattered fixture: 40 lines, far-apart single-line
        // changes at 0-based 4/19/34. Reference semantics: per change, 3
        // leading ctx + pair + 3 trailing ctx (21 rows, zero duplication).
        let mut a = numbered("L", 40);
        let mut b = a.clone();
        b[4] = "L4-changed".into();
        b[19] = "L19-changed".into();
        b[34] = "L34-changed".into();
        let (hunks, rows, adds, dels) =
            envelope_rows(&strings(&a), &strings(&b), 3);
        assert_eq!(adds, 3);
        assert_eq!(dels, 3);
        assert_eq!(
            hunks,
            vec![
                Hunk { a1: 1, a2: 8, b1: 1, b2: 8 },
                Hunk { a1: 16, a2: 23, b1: 16, b2: 23 },
                Hunk { a1: 31, a2: 38, b1: 31, b2: 38 },
            ]
        );
        assert_eq!(rows.len(), 21, "no duplicated leading context");
        let mut want: Vec<(usize, usize, &str)> = Vec::new();
        for c in [4usize, 19, 34] {
            for i in c - 3..c {
                want.push((i + 1, i + 1, "ctx"));
            }
            want.push((c + 1, c + 1, "pair"));
            for i in c + 1..c + 4 {
                want.push((i + 1, i + 1, "ctx"));
            }
        }
        for (r, w) in rows.iter().zip(want.iter()) {
            let kind = if r.lk == "ctx" { "ctx" } else { "pair" };
            assert_eq!((r.lo, r.ro, kind), *w, "row walk diverged at {}", r.lo);
        }
        // three-segment marking on the first pair: "L5" vs "L4-changed"
        let p = &rows[3];
        assert_eq!((p.lpre.as_str(), p.lmid.as_str(), p.lpost.as_str()), ("L", "5", ""));
        assert_eq!(
            (p.rpre.as_str(), p.rmid.as_str(), p.rpost.as_str()),
            ("L", "4-changed", "")
        );
    }

    #[test]
    fn d1_unbalanced_rows_reference() {
        // downstream unbalanced fixture: 3 dels + 1 add → 4 ctx + 1 pair +
        // 2 unpaired del rows (7). Pre-fix: pair/del rows dropped (6 ctx).
        let a = numbered("L", 10);
        let mut b: Vec<String> = a[..6].to_vec();
        b.push("NEW".into());
        b.extend(a[9..].to_vec());
        let (_, rows, adds, dels) =
            envelope_rows(&strings(&a), &strings(&b), 3);
        assert_eq!(adds, 1);
        assert_eq!(dels, 3);
        assert_eq!(rows.len(), 7, "pair + unpaired dels present");
        let seq: Vec<(usize, usize, &str, &str)> =
            rows.iter().map(|r| (r.lo, r.ro, r.lk, r.rk)).collect();
        assert_eq!(seq[0], (4, 4, "ctx", "ctx"));
        assert_eq!(seq[1], (5, 5, "ctx", "ctx"));
        assert_eq!(seq[2], (6, 6, "ctx", "ctx"));
        assert_eq!(seq[3], (7, 7, "del", "add"));
        assert_eq!(seq[4], (8, 0, "del", ""));
        assert_eq!(seq[5], (9, 0, "del", ""));
        assert_eq!(seq[6], (10, 8, "ctx", "ctx"));
    }

    #[test]
    fn d1_pure_add_and_del_change_rows_present() {
        // minimal negative assertions of the pre-fix symptom: "adds=3 while
        // rows carry zero add rows".
        let a = numbered("L", 10);
        let mut b: Vec<String> = a[..5].to_vec();
        b.extend(["E1".to_string(), "E2".to_string(), "E3".to_string()]);
        b.extend(a[5..].to_vec());
        let (_, rows, adds, dels) =
            envelope_rows(&strings(&a), &strings(&b), 3);
        assert_eq!(adds, 3);
        assert_eq!(dels, 0);
        assert_eq!(rows.len(), 9);
        assert_eq!(rows.iter().filter(|r| r.rk == "add").count(), 3, "add rows present");
        // mirror: pure delete
        let (_, rows2, adds2, dels2) =
            envelope_rows(&strings(&b), &strings(&a), 3);
        assert_eq!(adds2, 0);
        assert_eq!(dels2, 3);
        assert_eq!(rows2.iter().filter(|r| r.lk == "del").count(), 3, "del rows present");
    }
}

// ── PLAN-716 组C: rows 窗口投影（T-10——默认全量零扰动+窗口语义）────────

#[cfg(test)]
mod plan716_window {
    use super::*;

    /// 散点 fixture（多 hunk 远距改——021 FAIL 形态微缩）。
    fn scattered_pair(hunks: usize) -> (String, String) {
        let mut a = String::new();
        let mut b = String::new();
        for i in 0..hunks * 40 {
            a.push_str(&format!("L{}
", i));
            if i % 40 == 20 {
                b.push_str(&format!("L{}-changed
", i));
            } else {
                b.push_str(&format!("L{}
", i));
            }
        }
        (a, b)
    }

    /// frozen ③ 最严形：默认三参调用逐字节等价（golden 钉——不增任何字段）。
    #[test]
    fn default_envelope_byte_identical_golden() {
        let (a, b) = scattered_pair(3);
        let env = diff_files_envelope(&a, &b, 3);
        assert!(
            !env.contains("rows_total"),
            "默认形不得携带 rows_total 字段（frozen ③）"
        );
        assert!(env.contains("\"truncated\":false"), "默认形 truncated 恒 false");
        // 结构黄金钉：字段顺序与形态。
        assert!(env.starts_with("{\"hunks\":["));
        assert!(env.contains("\"rows\":[{\"lo\":"));
        // 尾形态（不猜转义——直接断言末段含 degraded/err 键序）。
        let tail = &env[env.len().saturating_sub(48)..];
        assert!(
            tail.contains("\"truncated\":false,\"degraded\":false,\"err\":\"\"}"),
            "默认形尾键序: {tail}"
        );
    }

    /// 窗口全开形 = 默认形 rows 逐行等价 + rows_total = 全量计数。
    #[test]
    fn window_full_matches_default_rows() {
        let (a, b) = scattered_pair(3);
        let def = diff_files_envelope(&a, &b, 3);
        let win = diff_files_envelope_window(&a, &b, 3, 0, usize::MAX);
        // rows_total = 默认形 rows 计数。
        let def_rows = def.match_indices("\"lo\":").count();
        assert!(
            win.contains(&format!("\"rows_total\":{def_rows}")),
            "rows_total 必须等于全量行数 {def_rows}: {}",
            &win[win.find("rows_total").unwrap()..win.find("rows_total").unwrap() + 30]
        );
        assert!(win.contains("\"truncated\":false"), "全窗不截断");
        // rows 内容逐行等价（serde 解析比较——字符串切片边界免疫）。
        let dv: serde_json::Value = serde_json::from_str(&def).unwrap();
        let wv: serde_json::Value = serde_json::from_str(&win).unwrap();
        assert_eq!(dv["rows"], wv["rows"], "全窗 rows 与默认形逐行等价");
    }

    /// 窗口切片语义：任意 (offset, limit) 的 rows == 全量 rows[offset..offset+limit]；
    /// truncated = offset+返回 < total；hunks 全量保持。
    #[test]
    fn window_slicing_semantics() {
        let (a, b) = scattered_pair(5);
        let win_full = diff_files_envelope_window(&a, &b, 3, 0, usize::MAX);
        let total: usize = win_full
            [win_full.find("rows_total").unwrap()..]
            .split(":")
            .nth(1)
            .unwrap()
            .split(",")
            .next()
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let rows_of = |s: &str| -> Vec<serde_json::Value> {
            let v: serde_json::Value = serde_json::from_str(s).unwrap();
            v["rows"].as_array().unwrap().clone()
        };
        let full_rows = rows_of(&win_full);

        for (off, lim) in [(0usize, 5usize), (3, 4), (total - 2, 10), (2, 0)] {
            let w = diff_files_envelope_window(&a, &b, 3, off, lim);
            let got = rows_of(&w);
            let want: Vec<serde_json::Value> =
                full_rows.iter().skip(off).take(lim).cloned().collect();
            assert_eq!(got, want, "窗口 ({off},{lim}) 切片语义");
            let expected_trunc = off > 0 || off + got.len() < total;
            assert!(
                w.contains(&format!("\"truncated\":{}", expected_trunc)),
                "truncated 语义 ({off},{lim})"
            );
            // hunks 全量保持（导航域）。
            let hunk_count_full = win_full.match_indices("\"a1\"").count();
            let hunk_count_win = w.match_indices("\"a1\"").count();
            assert_eq!(hunk_count_win, hunk_count_full, "hunks 不受窗口影响");
        }
    }

    /// 边界语义：offset 越界（≥total）→ rows:[]（diff_snapshots 净形复用）
    /// + truncated = total>0；limit 0 → 空+truncated 同语义。
    #[test]
    fn window_edge_semantics() {
        let (a, b) = scattered_pair(3);
        let win_full = diff_files_envelope_window(&a, &b, 3, 0, usize::MAX);
        let total: usize = win_full
            [win_full.find("rows_total").unwrap()..]
            .split(":")
            .nth(1)
            .unwrap()
            .split(",")
            .next()
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        // offset 越界 → 净形。
        let w = diff_files_envelope_window(&a, &b, 3, total + 10, 5);
        assert!(w.contains("\"rows\":[]"), "越界 offset → 净形 rows:[]");
        assert!(w.contains("\"truncated\":true"), "越界且 total>0 → truncated");
        assert!(w.contains(&format!("\"rows_total\":{total}")));

        // limit 0 → 空 + truncated（有行被省略）。
        let w0 = diff_files_envelope_window(&a, &b, 3, 0, 0);
        assert!(w0.contains("\"rows\":[]"));
        assert!(w0.contains("\"truncated\":true"));

        // 空净形形态（offset=0, limit=0 且无差异对——err 形除外）。
        let (ca, cb) = ("same
same
", "same
same
");
        let ident = diff_files_envelope_window(ca, cb, 3, 0, 10);
        assert!(ident.contains("\"rows\":[]"), "零差异 → 空 rows");
        assert!(ident.contains("\"truncated\":false"), "零差异无截断");
        assert!(ident.contains("\"rows_total\":0"));
    }

    /// 逐字散点窗口正确性：窗口跨 hunk（切片起止在不同 hunk 的行域）。
    #[test]
    fn window_across_hunks() {
        let (a, b) = scattered_pair(4);
        // 全量行序上取跨 hunk 窗（第 10..25 行——落在 hunk0 尾+ctx+hunk1 头域）。
        let full = diff_files_envelope_window(&a, &b, 3, 0, usize::MAX);
        let rows_of = |s: &str| -> Vec<serde_json::Value> {
            let v: serde_json::Value = serde_json::from_str(s).unwrap();
            v["rows"].as_array().unwrap().clone()
        };
        let all = rows_of(&full);
        let w = diff_files_envelope_window(&a, &b, 3, 10, 15);
        let got = rows_of(&w);
        let want: Vec<serde_json::Value> =
            all.iter().skip(10).take(15).cloned().collect();
        assert_eq!(got, want, "跨 hunk 窗口切片");
    }
}
