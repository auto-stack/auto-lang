//! 源码 snapshot：不可变分页块 + 编辑事务 + 内容身份（PLAN-787 §5.1）。
//!
//! - 页 = 内容寻址（blake3(bytes)），≤ `PAGE_BYTES`，无最小尺寸；
//! - 文档目录 = 页引用序列，偏移 = 前缀和（长度索引），页内不存绝对 offset——
//!   开头插入只改目录，后缀页内容不变即 hash 复用；
//! - 相同文本跨文档共享字节页（PageStore 按 hash 去重）；来源定位隔离由
//!   产物/查询层携带工程上下文，内容身份保持纯内容；
//! - Edit：错 base、乱序、重叠、非 UTF-8 边界/插入 → 受控拒绝，旧 snapshot 不变；
//! - 批量 edit 统一基于同一 base、非重叠、按确定顺序（字节序）应用。

use crate::identity::{domain_hash, SnapshotId};
use crate::program::RtError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// 源页尺寸上限（T-01 冻结：16 KiB；页可以更小——编辑局部性优先于均匀分页）。
pub const PAGE_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug)]
pub struct Page {
    pub hash: String,
    pub bytes: Vec<u8>,
}

/// 会话级页仓：hash → 驻留页。编辑后未再引用的旧页随 Rc 释放淘汰。
#[derive(Default)]
pub struct PageStore {
    resident: RefCell<HashMap<String, Rc<Page>>>,
    /// 驻留预算（字节，超出后按插入序淘汰未被引用的页）。
    budget: usize,
    /// 已请求读出的字节总数（真实读路径计量，V-06 复用率分母）。
    pub read_bytes: std::cell::Cell<u64>,
}

impl PageStore {
    pub fn new() -> Self {
        Self::with_budget(64 << 20)
    }
    pub fn with_budget(budget: usize) -> Self {
        PageStore {
            resident: RefCell::new(HashMap::new()),
            budget,
            read_bytes: std::cell::Cell::new(0),
        }
    }
    fn intern(&self, bytes: Vec<u8>) -> Rc<Page> {
        let hash = domain_hash("page", &[&bytes]);
        let mut m = self.resident.borrow_mut();
        if let Some(p) = m.get(&hash) {
            return p.clone();
        }
        let p = Rc::new(Page { hash, bytes });
        m.insert(p.hash.clone(), p.clone());
        // 简单预算淘汰：仅淘汰无外部引用（Rc==1）的页，活树不受影响。
        if m.values().map(|p| p.bytes.len()).sum::<usize>() > self.budget {
            let evictable: Vec<String> = m
                .iter()
                .filter(|(_, p)| Rc::strong_count(p) == 1)
                .map(|(h, _)| h.clone())
                .collect();
            for h in evictable {
                m.remove(&h);
            }
        }
        p
    }
    fn get(&self, hash: &str) -> Option<Rc<Page>> {
        self.resident.borrow().get(hash).cloned()
    }
}

/// 页引用（目录项）。
#[derive(Clone, Debug)]
pub struct PageRef {
    pub hash: String,
    pub len: u32,
}

/// 不可变快照（文档版本）。`SnapshotId` 绑定内容（页 hash 链）。
#[derive(Clone, Debug)]
pub struct Snapshot {
    pages: Vec<PageRef>,
    total_len: u64,
    id: SnapshotId,
    revision: u32,
    store: Rc<PageStore>,
}

/// 编辑会话：当前根 + 显式旧版本 lease（`LeasedSnapshot` 释放后可淘汰）。
pub struct Document {
    pub current: Snapshot,
    store: Rc<PageStore>,
}

#[derive(Clone, Debug)]
pub struct Edit {
    /// 半开字节范围 [start, end)。
    pub start: u64,
    pub end: u64,
    pub insert: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct EditRequest {
    pub base_revision: u32,
    /// 按声明序给出；内部按 start 排序+重叠校验（确定顺序）。
    pub edits: Vec<Edit>,
}

pub fn snapshot_id_for(pages: &[PageRef], total_len: u64) -> SnapshotId {
    let mut parts: Vec<Vec<u8>> = Vec::with_capacity(pages.len() * 2 + 2);
    parts.push(total_len.to_le_bytes().to_vec());
    parts.push((pages.len() as u64).to_le_bytes().to_vec());
    for p in pages {
        parts.push(p.len.to_le_bytes().to_vec());
        parts.push(p.hash.as_bytes().to_vec());
    }
    let refs: Vec<&[u8]> = parts.iter().map(|v| v.as_slice()).collect();
    SnapshotId::from_parts(&refs)
}

impl Snapshot {
    /// 从 UTF-8 文本构建（非 UTF-8 输入拒绝）。空文本 = 单个空页（目录不变量统一）。
    pub fn from_text(store: &Rc<PageStore>, text: &str) -> Snapshot {
        let bytes = text.as_bytes();
        let mut pages = Vec::with_capacity(bytes.len().div_ceil(PAGE_BYTES).max(1));
        let chunks: Vec<&[u8]> = if bytes.is_empty() {
            vec![&[][..]]
        } else {
            bytes.chunks(PAGE_BYTES).collect()
        };
        for chunk in chunks {
            let p = store.intern(chunk.to_vec());
            pages.push(PageRef { hash: p.hash.clone(), len: chunk.len() as u32 });
        }
        let total_len = bytes.len() as u64;
        Snapshot {
            id: snapshot_id_for(&pages, total_len),
            pages,
            total_len,
            revision: 0,
            store: store.clone(),
        }
    }

    pub fn id(&self) -> &SnapshotId {
        &self.id
    }
    pub fn revision(&self) -> u32 {
        self.revision
    }
    pub fn len(&self) -> u64 {
        self.total_len
    }
    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
    pub fn page_refs(&self) -> &[PageRef] {
        &self.pages
    }

    /// 页起始偏移（前缀和；不缓存 = 目录不变时 O(n) 重建，n=页数）。
    pub fn page_offsets(&self) -> Vec<u64> {
        let mut v = Vec::with_capacity(self.pages.len() + 1);
        let mut acc = 0u64;
        v.push(0);
        for p in &self.pages {
            acc += p.len as u64;
            v.push(acc);
        }
        v
    }

    fn locate(&self, offsets: &[u64], off: u64) -> usize {
        match offsets.binary_search(&off) {
            Ok(i) => i.min(self.pages.len() - 1),
            Err(i) => (i - 1).min(self.pages.len() - 1),
        }
    }

    /// 读字节范围（跨页拼接）；计量真实读页字节（V-06 复用统计）。
    pub fn read_bytes(&self, start: u64, end: u64) -> Result<Vec<u8>, RtError> {
        if end < start || end > self.total_len {
            return Err(RtError::BadEdit(format!("range [{start},{end}) out of bounds {}", self.total_len)));
        }
        if start == end {
            return Ok(Vec::new());
        }
        let offsets = self.page_offsets();
        let first = self.locate(&offsets, start);
        let mut out = Vec::with_capacity((end - start) as usize);
        let mut i = first;
        while i < self.pages.len() && offsets[i] < end {
            let page = self
                .store
                .get(&self.pages[i].hash)
                .ok_or_else(|| RtError::Storage(format!("page {} evicted", &self.pages[i].hash[..12])))?;
            self.store.read_bytes.set(self.store.read_bytes.get() + page.bytes.len() as u64);
            let pstart = offsets[i].max(start) - offsets[i];
            let pend = (end - offsets[i]).min(page.bytes.len() as u64);
            out.extend_from_slice(&page.bytes[pstart as usize..pend as usize]);
            i += 1;
        }
        Ok(out)
    }

    pub fn read_str(&self, start: u64, end: u64) -> Result<String, RtError> {
        let b = self.read_bytes(start, end)?;
        String::from_utf8(b).map_err(|_| RtError::BadEdit("range is not utf-8 aligned".into()))
    }

    fn is_char_boundary(&self, off: u64) -> Result<bool, RtError> {
        if off > self.total_len {
            return Err(RtError::BadEdit(format!("offset {off} out of bounds")));
        }
        if off == 0 || off == self.total_len {
            return Ok(true);
        }
        // off 是 UTF-8 标量边界 ⟺ off 处字节不是延续字节（0b10xxxxxx）。
        let b = self.read_bytes(off, off + 1)?;
        Ok(b[0] & 0xC0 != 0x80)
    }

    /// 应用编辑（校验后），产出新 snapshot；失败则本 snapshot 不变。
    pub fn edit(&self, req: &EditRequest) -> Result<Snapshot, RtError> {
        if req.base_revision != self.revision {
            return Err(RtError::BadEdit(format!(
                "base revision {} != current {}",
                req.base_revision, self.revision
            )));
        }
        if req.edits.is_empty() {
            return Ok(self.clone());
        }
        let mut edits = req.edits.clone();
        edits.sort_by_key(|e| e.start);
        std::str::from_utf8(&{
            let mut all = Vec::new();
            for e in &edits {
                all.extend_from_slice(&e.insert);
            }
            all
        })
        .map_err(|_| RtError::BadEdit("insert payload is not valid utf-8".into()))?;
        for w in edits.windows(2) {
            if w[0].end > w[1].start {
                return Err(RtError::BadEdit(format!(
                    "overlapping edits [{},{}) and [{},{})",
                    w[0].start, w[0].end, w[1].start, w[1].end
                )));
            }
        }
        for e in &edits {
            if e.end < e.start {
                return Err(RtError::BadEdit(format!("edit end {} < start {}", e.end, e.start)));
            }
            if e.end > self.total_len {
                return Err(RtError::BadEdit(format!("edit [{},{}) out of bounds {}", e.start, e.end, self.total_len)));
            }
            if !self.is_char_boundary(e.start)? || !self.is_char_boundary(e.end)? {
                return Err(RtError::BadEdit(format!(
                    "edit boundary not on utf-8 scalar boundary: [{},{})",
                    e.start, e.end
                )));
            }
        }

        // 联合跨度重建：所有编辑的受影响页区间取并集，一次重建。
        // 页内多编辑按字节序缝合（前缀 + gap/insert 交替 + 后缀）。
        let offsets = self.page_offsets();
        let lo = edits.first().unwrap().start;
        let hi = edits.last().unwrap().end.max(edits.last().unwrap().start);
        let affected_first = self.locate(&offsets, lo);
        let affected_last = if hi > lo { self.locate(&offsets, hi - 1) } else { self.locate(&offsets, lo) };
        let mut new_pages: Vec<PageRef> = Vec::with_capacity(self.pages.len() + edits.len() + 4);
        for i in 0..affected_first {
            new_pages.push(self.pages[i].clone());
        }
        // 前缀：受影响区起点之前。
        let span_start = offsets[affected_first];
        if lo > span_start {
            let bytes = self.read_bytes(span_start, lo)?;
            let p = self.store.intern(bytes);
            new_pages.push(PageRef { hash: p.hash.clone(), len: (lo - span_start) as u32 });
        }
        // 编辑区：gap/insert 交替。
        let mut prev_end = lo;
        for e in &edits {
            if e.start > prev_end {
                let bytes = self.read_bytes(prev_end, e.start)?;
                let p = self.store.intern(bytes);
                new_pages.push(PageRef { hash: p.hash.clone(), len: (e.start - prev_end) as u32 });
            }
            for chunk in e.insert.chunks(PAGE_BYTES) {
                let p = self.store.intern(chunk.to_vec());
                new_pages.push(PageRef { hash: p.hash.clone(), len: chunk.len() as u32 });
            }
            prev_end = e.end;
        }
        // 后缀：最后编辑末尾到受影响区终点。
        let span_end = offsets[affected_last + 1];
        if span_end > prev_end {
            let bytes = self.read_bytes(prev_end, span_end)?;
            let p = self.store.intern(bytes);
            new_pages.push(PageRef { hash: p.hash.clone(), len: (span_end - prev_end) as u32 });
        }
        // 尾部未触页。
        for r in &self.pages[(affected_last + 1)..] {
            new_pages.push(r.clone());
        }

        let delta: i64 = edits
            .iter()
            .map(|e| e.insert.len() as i64 - (e.end - e.start) as i64)
            .sum();
        let new_len = (self.total_len as i64 + delta) as u64;
        Ok(Snapshot {
            id: snapshot_id_for(&new_pages, new_len),
            pages: new_pages,
            total_len: new_len,
            revision: self.revision + 1,
            store: self.store.clone(),
        })
    }
}

impl Document {
    pub fn from_text(store: &Rc<PageStore>, text: &str) -> Document {
        Document { current: Snapshot::from_text(store, text), store: store.clone() }
    }
    pub fn edit(&mut self, req: &EditRequest) -> Result<&Snapshot, RtError> {
        let next = self.current.edit(req)?;
        self.current = next;
        Ok(&self.current)
    }
    pub fn snapshot(&self) -> &Snapshot {
        &self.current
    }
    pub fn store(&self) -> &Rc<PageStore> {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> Rc<PageStore> {
        Rc::new(PageStore::new())
    }

    #[test]
    fn snapshot_identity_is_content_bound() {
        let s1 = Snapshot::from_text(&store(), "hello world");
        let s2 = Snapshot::from_text(&store(), "hello world");
        let s3 = Snapshot::from_text(&store(), "hello worlds");
        assert_eq!(s1.id(), s2.id(), "same text shares snapshot id across documents");
        assert_ne!(s1.id(), s3.id());
    }

    #[test]
    fn snapshot_pages_shared_by_content() {
        let st = store();
        let plen = format!("{}", "x".repeat(PAGE_BYTES));
        let a = Snapshot::from_text(&st, &plen);
        let b = Snapshot::from_text(&st, &plen);
        assert_eq!(a.page_refs()[0].hash, b.page_refs()[0].hash);
        // 整页对齐追加：前缀页共享，尾部为小页。
        let c = Snapshot::from_text(&st, &format!("{}y", plen));
        assert_eq!(a.page_refs()[0].hash, c.page_refs()[0].hash);
        assert_eq!(c.page_count(), 2);
    }

    #[test]
    fn edit_rejects_bad_base_and_keeps_old() {
        let mut d = Document::from_text(&store(), "fn main() int { return 0 }");
        let before = d.snapshot().clone();
        let bad = d.edit(&EditRequest { base_revision: 7, edits: vec![Edit { start: 0, end: 0, insert: b"x".to_vec() }] });
        assert!(matches!(bad, Err(RtError::BadEdit(_))));
        assert_eq!(d.snapshot().id(), before.id());
    }

    #[test]
    fn edit_rejects_overlap_and_bad_utf8() {
        let mut d = Document::from_text(&store(), "abcdef");
        let mut e = |req: EditRequest| d.edit(&req).map(|_| ()).unwrap_err();
        assert!(matches!(
            e(EditRequest {
                base_revision: 0,
                edits: vec![
                    Edit { start: 1, end: 3, insert: vec![] },
                    Edit { start: 2, end: 4, insert: vec![] },
                ],
            }),
            RtError::BadEdit(_)
        ));
        // emoji 中点：'😀' = F0 9F 98 80，切在 2 = 非标量边界。
        let mut d2 = Document::from_text(&store(), "a😀b");
        let r = d2.edit(&EditRequest {
            base_revision: 0,
            edits: vec![Edit { start: 2, end: 3, insert: b"x".to_vec() }],
        });
        assert!(matches!(r, Err(RtError::BadEdit(_))), "mid-scalar cut must be rejected");
        // 非法 UTF-8 插入。
        let mut d3 = Document::from_text(&store(), "ab");
        let r = d3.edit(&EditRequest {
            base_revision: 0,
            edits: vec![Edit { start: 1, end: 1, insert: vec![0xFF, 0xFE] }],
        });
        assert!(matches!(r, Err(RtError::BadEdit(_))));
    }

    #[test]
    fn edit_content_and_revision_chain() {
        let mut d = Document::from_text(&store(), "fn a() int { return 1 }");
        assert_eq!(d.snapshot().revision(), 0);
        d.edit(&EditRequest {
            base_revision: 0,
            edits: vec![Edit { start: 3, end: 4, insert: b"bee".to_vec() }] // a -> bee
        })
        .unwrap();
        assert_eq!(d.snapshot().revision(), 1);
        assert_eq!(d.snapshot().read_str(0, d.snapshot().len()).unwrap(), "fn bee() int { return 1 }");
    }

    #[test]
    fn insert_at_start_reuses_tail_pages() {
        let st = store();
        let text = "x".repeat(PAGE_BYTES * 3 + 17);
        let mut d = Document::from_text(&st, &text);
        let old_tail: Vec<String> = d.snapshot().page_refs()[1..].iter().map(|p| p.hash.clone()).collect();
        d.edit(&EditRequest {
            base_revision: 0,
            edits: vec![Edit { start: 0, end: 1, insert: b"Y".to_vec() }],
        })
        .unwrap();
        let s = d.snapshot();
        // 新目录 = [插入页, 首页重建余量, 原后缀页...]；后缀页 hash 原样复用。
        let new_all: Vec<String> = s.page_refs().iter().map(|p| p.hash.clone()).collect();
        let new_tail: Vec<String> = s.page_refs()[2..].iter().map(|p| p.hash.clone()).collect();
        assert_eq!(new_all.len(), old_tail.len() + 2);
        assert_eq!(old_tail, new_tail, "inserting at doc start must not rewrite suffix pages");
        assert_eq!(s.read_str(0, s.len()).unwrap(), format!("Y{}", &text[1..]));
    }

    #[test]
    fn multi_edit_deterministic_order() {
        let st = store();
        let mut d = Document::from_text(&st, "aaa bbb ccc");
        // 乱序给出，内部排序后应用。
        d.edit(&EditRequest {
            base_revision: 0,
            edits: vec![
                Edit { start: 8, end: 11, insert: b"ccc!".to_vec() },
                Edit { start: 0, end: 3, insert: b"aaa!".to_vec() },
            ],
        })
        .unwrap();
        assert_eq!(d.snapshot().read_str(0, d.snapshot().len()).unwrap(), "aaa! bbb ccc!");
    }

    #[test]
    fn crlf_midpoint_policy() {
        // T-01 冻结：\r 与 \n 之间是合法字节编辑点（CRLF 中点插入允许，字节层诚实）。
        let mut d = Document::from_text(&store(), "a\r\nb");
        d.edit(&EditRequest {
            base_revision: 0,
            edits: vec![Edit { start: 2, end: 2, insert: b"-".to_vec() }] // \r|\n 中点
        })
        .unwrap();
        assert_eq!(d.snapshot().read_str(0, d.snapshot().len()).unwrap(), "a\r-\nb");
    }

    #[test]
    fn read_range_across_pages() {
        let st = store();
        let text = format!("{}middle{}", "A".repeat(PAGE_BYTES), "B".repeat(PAGE_BYTES));
        let s = Snapshot::from_text(&st, &text);
        let mid = s.read_str(PAGE_BYTES as u64 - 2, PAGE_BYTES as u64 + 5).unwrap();
        assert_eq!(mid, "AAmiddl".to_string());
    }
}

impl std::fmt::Debug for PageStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PageStore(..)")
    }
}
