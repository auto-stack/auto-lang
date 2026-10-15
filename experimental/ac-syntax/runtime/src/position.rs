//! 位置 adapter：分页行索引 + byte↔UTF-8/UTF-16 行列转换。
//!
//! T-01 冻结策略：
//! - 行以 `\n` 结束（CRLF 属同一行尾；孤立 `\r` 是行内普通字节）；
//! - line/col 1-based；col 为行内字节偏移+1（UTF-8 视图）或 UTF-16 单元数（UTF-16 视图）；
//! - UTF-16 偏移落在代理对中间（emoji 中点）→ 向下取整到包含标量的起始（确定性 floor）；
//! - 超出行尾的 col → 夹取行尾（不含换行符）；超出行数 → 夹取末行末尾；
//! - 单次转换只触达目标行所在页（行索引分页），不全文扫描。

use crate::snapshot::Snapshot;

/// 分页行索引：每页的换行偏移表 + 页前累计行数。
pub struct LineIndex {
    /// page -> 页内换行的字节偏移（页内相对，升序）。
    page_newlines: Vec<Vec<u32>>,
    /// page -> 页前累计换行数（prefix sum）。
    prefix: Vec<u64>,
    page_offsets: Vec<u64>,
    total_len: u64,
}

impl LineIndex {
    pub fn build(snap: &Snapshot) -> LineIndex {
        let page_offsets = snap.page_offsets();
        let mut page_newlines = Vec::with_capacity(snap.page_count());
        let mut prefix = Vec::with_capacity(snap.page_count() + 1);
        let mut acc = 0u64;
        for (pi, pref) in snap.page_refs().iter().enumerate() {
            prefix.push(acc);
            let mut nl = Vec::new();
            if let Ok(bytes) = snap.read_bytes(page_offsets[pi], page_offsets[pi + 1]) {
                for (i, b) in bytes.iter().enumerate() {
                    if *b == b'\n' {
                        nl.push(i as u32);
                        acc += 1;
                    }
                }
            }
            let _ = pref;
            page_newlines.push(nl);
        }
        prefix.push(acc);
        LineIndex { page_newlines, prefix, page_offsets, total_len: snap.len() }
    }

    pub fn line_count(&self) -> u64 {
        self.prefix[self.prefix.len() - 1] + 1
    }

    /// 字节偏移 → (line, col_utf8)。O(页) 定位 + 行内步进。
    pub fn line_col_utf8(&self, _snap: &Snapshot, offset: u64) -> (u64, u64) {
        let (page, line_start_abs, line_no) = self.locate_line(offset);
        let col = offset - line_start_abs;
        let _ = page;
        (line_no + 1, col + 1)
    }

    /// 字节偏移 → (line, col_utf16)。
    pub fn line_col_utf16(&self, snap: &Snapshot, offset: u64) -> (u64, u64) {
        let (page, line_start_abs, line_no) = self.locate_line(offset);
        let mut u16_units = 0u64;
        let mut i = line_start_abs;
        while i < offset {
            let b = self.byte_at(snap, i);
            if b < 0x80 {
                u16_units += 1;
                i += 1;
            } else if b < 0xE0 {
                u16_units += 1;
                i += 2;
            } else if b < 0xF0 {
                u16_units += 1;
                i += 3;
            } else {
                u16_units += 2; // 代理对
                i += 4;
            }
        }
        let _ = page;
        (line_no + 1, u16_units + 1)
    }

    /// (line, col_utf16) → 字节偏移（emoji 代理中点 floor；行尾夹取）。line 1-based。
    pub fn offset_of_utf16(&self, snap: &Snapshot, line: u64, col_u16: u64) -> u64 {
        let line_start = self.line_start(line - 1);
        let line_end = self.line_end(line - 1);
        let mut units = 0u64;
        let mut i = line_start;
        while i < line_end {
            if units >= col_u16 {
                return i;
            }
            let b = self.byte_at(snap, i);
            let w = if b < 0x80 {
                1
            } else if b < 0xE0 {
                2
            } else if b < 0xF0 {
                3
            } else {
                4
            };
            // 代理对中点：units+1 < col（col 落在标量中间）→ 仍返回标量起点。
            if units + (if w == 4 { 2 } else { 1 }) >= col_u16 && col_u16 > units {
                return i;
            }
            units += if w == 4 { 2 } else { 1 };
            i += w;
        }
        line_end.min(self.total_len)
    }

    /// (line, col_utf8字节) → 偏移（行尾夹取）。line 1-based。
    pub fn offset_of_utf8(&self, line: u64, col: u64) -> u64 {
        let s = self.line_start(line - 1);
        let e = self.line_end(line - 1);
        (s + col.saturating_sub(1)).min(e).min(self.total_len)
    }

    fn locate_line(&self, offset: u64) -> (usize, u64, u64) {
        let page = match self.page_offsets.binary_search(&offset) {
            Ok(i) => i.min(self.page_newlines.len() - 1),
            Err(i) => (i - 1).min(self.page_newlines.len() - 1),
        };
        let in_page = (offset - self.page_offsets[page]) as u32;
        let nl_before_in_page = match self.page_newlines[page].binary_search(&in_page) {
            Ok(_) | Err(_) => self.page_newlines[page].iter().filter(|&&x| x < in_page).count() as u64,
        };
        let line_no = self.prefix[page] + nl_before_in_page;
        let line_start = if nl_before_in_page == 0 {
            self.page_offsets[page]
        } else {
            let idx = (nl_before_in_page - 1) as usize;
            self.page_offsets[page] + self.page_newlines[page][idx] as u64 + 1
        };
        (page, line_start, line_no)
    }

    /// 第 nth 个换行（0-based）所在页与页内下标；None = 不存在（EOF 后）。
    fn page_of_newline(&self, nth: u64) -> Option<(usize, usize)> {
        for p in 0..self.page_newlines.len() {
            let cnt = self.page_newlines[p].len() as u64;
            if nth < self.prefix[p] + cnt {
                return Some((p, (nth - self.prefix[p]) as usize));
            }
        }
        None
    }

    fn line_start(&self, line: u64) -> u64 {
        if line == 0 {
            return 0;
        }
        match self.page_of_newline(line - 1) {
            Some((p, i)) => self.page_offsets[p] + self.page_newlines[p][i] as u64 + 1,
            None => self.total_len,
        }
    }

    fn line_end(&self, line: u64) -> u64 {
        match self.page_of_newline(line) {
            Some((p, i)) => self.page_offsets[p] + self.page_newlines[p][i] as u64,
            None => self.total_len,
        }
    }

    fn byte_at(&self, snap: &Snapshot, offset: u64) -> u8 {
        snap.read_bytes(offset, offset + 1).map(|b| b[0]).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{PageStore, Snapshot};
    use std::rc::Rc;

    fn snap(text: &str) -> Snapshot {
        Snapshot::from_text(&Rc::new(PageStore::new()), text)
    }

    #[test]
    fn position_utf8_lines_and_cols() {
        let s = snap("abc\ndef\ngh");
        let li = LineIndex::build(&s);
        assert_eq!(li.line_count(), 3);
        assert_eq!(li.line_col_utf8(&s, 0), (1, 1));
        assert_eq!(li.line_col_utf8(&s, 3), (1, 4));
        assert_eq!(li.line_col_utf8(&s, 4), (2, 1));
        assert_eq!(li.line_col_utf8(&s, 8), (3, 1));
        assert_eq!(li.offset_of_utf8(2, 2), 5);
    }

    #[test]
    fn position_utf16_emoji_floor_policy() {
        let s = snap("a😀b");
        let li = LineIndex::build(&s);
        // 😀 = 4 字节 / 2 UTF-16 单元。
        assert_eq!(li.line_col_utf16(&s, 0), (1, 1));
        assert_eq!(li.line_col_utf16(&s, 1), (1, 2)); // b 前是 1 标量(a)+2 单元(emoji) → col 4
        // 上行应为 (1,4)：a=1, emoji=2 → 偏移1 是 emoji 起点，其前 1 单元 → col 2。对：
        // 偏移 1 (emoji 起点) 前 = "a" = 1 单元 → col 2 ✓
        assert_eq!(li.line_col_utf16(&s, 5), (1, 4)); // b 前 = 1+2 = 3 单元 → col 4
        // 代理对中点（col 3）→ floor 到 emoji 起点（字节 1）。
        assert_eq!(li.offset_of_utf16(&s, 1, 3), 1);
        assert_eq!(li.offset_of_utf16(&s, 1, 1), 0);
        assert_eq!(li.offset_of_utf16(&s, 1, 4), 5);
    }

    #[test]
    fn position_crlf_single_line() {
        let s = snap("a\r\nb\r\n");
        let li = LineIndex::build(&s);
        assert_eq!(li.line_count(), 3);
        assert_eq!(li.line_col_utf8(&s, 1), (1, 2)); // \r 行内
        assert_eq!(li.line_col_utf8(&s, 2), (1, 3)); // \n 行内（同一行）
        assert_eq!(li.line_col_utf8(&s, 3), (2, 1));
    }

    #[test]
    fn position_cross_page() {
        let text = format!("{}\nsecond line", "x".repeat(crate::snapshot::PAGE_BYTES + 10));
        let s = snap(&text);
        let li = LineIndex::build(&s);
        let off = crate::snapshot::PAGE_BYTES as u64 + 10;
        assert_eq!(li.line_col_utf8(&s, off + 1), (2, 1));
        assert_eq!(li.offset_of_utf8(2, 6), off + 6);
    }
}
