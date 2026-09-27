// PLAN-703 T-05 — directory comparison v1 (supply pack 供④).
//
// Two-level model: existence/kind/metadata classification first, content
// level (the T-02 engine, on demand) second. The five-state classification
// mirrors the downstream `fsys.diff_dirs_json` contract field-for-field
// (auto-edit diff-view.md, PLAN-012 SD-01) so the replacement seam keeps
// matrices stable:
//   ① existence (single side → deleted/added; left=path_a=old, right=new)
//   ② kind conflict (dir↔file → modified)
//   ③ binary heuristic (size>0 and text read fails/empty on ≤2MB domain;
//      any side hit → binary)
//   ④ size difference → modified
//   ⑤ ≤2MB byte-equal → same; >2MB same-size → same + note="uncompared"
//      (downstream-visible "same (uncompared)" note state — the engine-era
//      byte compare for >2MB is a downstream-visible behaviour change and
//      stays out of the parity envelope).
//
// The Rust face is a lazy iterator: classification (and the file reads it
// needs) happens per `next()`, so large trees return incrementally and a
// consumer may stop early. Counts accumulate over the ITERATED domain (the
// envelope's counts/entries same-domain rule). Content hunks are lazy too —
// an entry carries its file pair and materializes them on demand.

use std::fs;
use std::path::{Path, PathBuf};

use super::{diff_lines, DiffOpts, DiffOut};

/// Skip-list segments — downstream `fif_skipped` semantics: any path
/// segment starting with '.' plus the six build-artifact directories.
fn segment_skipped(name: &str) -> bool {
    name.starts_with('.')
        || matches!(name, "target" | "node_modules" | "gen" | "dist" | "build" | "__pycache__")
}

/// ≤2MB content-compare domain (downstream `ccap`).
const CONTENT_COMPARE_CAP: u64 = 2 * 1024 * 1024;

/// Directory comparison options.
#[derive(Debug, Clone)]
pub struct DirDiffOptions {
    /// Same size + same mtime → `same` without reading content. Off by
    /// default: the downstream envelope never had an mtime fast path, and
    /// a same-size-same-mtime-different-content edge would classify
    /// differently (parity-first default).
    pub mtime_fast_path: bool,
}

impl Default for DirDiffOptions {
    fn default() -> Self {
        DirDiffOptions { mtime_fast_path: false }
    }
}

/// The five envelope states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirStatus {
    Same,
    Added,
    Deleted,
    Modified,
    Binary,
}

impl DirStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            DirStatus::Same => "same",
            DirStatus::Added => "added",
            DirStatus::Deleted => "deleted",
            DirStatus::Modified => "modified",
            DirStatus::Binary => "binary",
        }
    }
}

/// One classified relative path. `size_a`/`size_b` are 0 on the absent
/// side (downstream envelope shape); `pair` is set for file entries that
/// exist on at least one side and can be content-diffed on demand.
#[derive(Debug, Clone)]
pub struct DirEntryDiff {
    pub rel: String,
    pub status: DirStatus,
    pub size_a: u64,
    pub size_b: u64,
    pub is_dir: bool,
    pub note: String,
    pub pair: Option<(PathBuf, PathBuf)>,
}

impl DirEntryDiff {
    /// Lazy content-level hunks for a modified file entry (the two-level
    /// model's second level). `None` for dirs/non-modified/absent pairs.
    pub fn hunks(&self, opts: DiffOpts) -> Option<DiffOut> {
        let (pa, pb) = self.pair.as_ref()?;
        if self.is_dir || !matches!(self.status, DirStatus::Modified) {
            return None;
        }
        let ta = fs::read_to_string(pa).ok()?;
        let tb = fs::read_to_string(pb).ok()?;
        Some(diff_lines(&ta, &tb, opts))
    }
}

/// Same-domain counters over the iterated entries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DirCounts {
    pub same: usize,
    pub added: usize,
    pub deleted: usize,
    pub modified: usize,
    pub binary: usize,
}

/// Errors: missing roots (the endpoint maps these to the envelope `err`
/// form — entries empty, error not silent).
#[derive(Debug)]
pub enum DirDiffError {
    RootMissing(String),
}

struct MergedRel {
    rel: String,
    a: Option<(u64, bool, Option<std::time::SystemTime>)>,
    b: Option<(u64, bool, Option<std::time::SystemTime>)>,
    path_a: PathBuf,
    path_b: PathBuf,
}

/// Recursive listing into (rel → (size, is_dir, mtime)), rels '/'-separated,
/// skip-list filtered, per-directory sorted for determinism. Directory
/// entries are listed too (they participate in existence/kind checks); a
/// directory's size records 0 (downstream shape). Metadata races (delete
/// mid-walk) skip the entry instead of failing the walk.
fn walk_tree(root: &Path, out: &mut Vec<(String, u64, bool, Option<std::time::SystemTime>)>) -> std::io::Result<()> {
    let mut entries: Vec<_> = match fs::read_dir(root) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotADirectory || e.kind() == std::io::ErrorKind::PermissionDenied => return Ok(()),
        Err(e) => return Err(e),
    };
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if segment_skipped(&name) {
            continue;
        }
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue, // race delete — skip (downstream try-wrap)
        };
        let is_dir = meta.is_dir();
        let rel = name;
        out.push((rel.clone(), if is_dir { 0 } else { meta.len() }, is_dir, meta.modified().ok()));
        if is_dir {
            let sub = out.len();
            walk_tree(&path, out)?;
            // Prefix the subtree's rels with this segment.
            for (r, ..) in &mut out[sub..] {
                *r = format!("{rel}/{r}");
            }
        }
    }
    Ok(())
}

/// Streaming directory diff. Classification happens lazily per item; counts
/// accumulate over the iterated domain only.
pub struct DirDiffIter {
    merged: Vec<MergedRel>,
    idx: usize,
    opts: DirDiffOptions,
    counts: DirCounts,
}

impl DirDiffIter {
    /// Build the iterator: both trees walked, sorted, and merged by rel
    /// (existence on either side keeps the entry).
    pub fn new(a_root: &Path, b_root: &Path, opts: DirDiffOptions) -> Result<Self, DirDiffError> {
        if !a_root.is_dir() {
            return Err(DirDiffError::RootMissing(a_root.to_string_lossy().to_string()));
        }
        if !b_root.is_dir() {
            return Err(DirDiffError::RootMissing(b_root.to_string_lossy().to_string()));
        }
        let mut la = Vec::new();
        let mut lb = Vec::new();
        walk_tree(a_root, &mut la).map_err(|_| DirDiffError::RootMissing(a_root.to_string_lossy().to_string()))?;
        walk_tree(b_root, &mut lb).map_err(|_| DirDiffError::RootMissing(b_root.to_string_lossy().to_string()))?;
        let ma: std::collections::BTreeMap<String, (u64, bool, Option<std::time::SystemTime>)> =
            la.into_iter().map(|(r, s, d, m)| (r, (s, d, m))).collect();
        let mb: std::collections::BTreeMap<String, (u64, bool, Option<std::time::SystemTime>)> =
            lb.into_iter().map(|(r, s, d, m)| (r, (s, d, m))).collect();
        let mut rels: std::collections::BTreeSet<String> = ma.keys().cloned().collect();
        rels.extend(mb.keys().cloned());
        let merged = rels
            .into_iter()
            .map(|rel| MergedRel {
                path_a: a_root.join(&rel),
                path_b: b_root.join(&rel),
                a: ma.get(&rel).cloned(),
                b: mb.get(&rel).cloned(),
                rel,
            })
            .collect();
        Ok(DirDiffIter { merged, idx: 0, opts, counts: DirCounts::default() })
    }

    /// Counts over the entries classified so far (same domain as the
    /// consumed entries — AC-03 counting consistency).
    pub fn counts(&self) -> DirCounts {
        self.counts
    }

    pub fn remaining(&self) -> usize {
        self.merged.len() - self.idx
    }

    /// Binary heuristic (downstream ③): within the ≤2MB domain, a nonzero
    /// file whose text read fails or reads empty counts as binary.
    fn reads_binary(path: &Path, size: u64) -> bool {
        if size == 0 || size > CONTENT_COMPARE_CAP {
            return false;
        }
        match fs::read(path) {
            Ok(bytes) => bytes.is_empty() || std::str::from_utf8(&bytes).is_err(),
            Err(_) => true, // unreadable within the domain ≈ downstream read failure
        }
    }

    fn classify(&mut self, m: &MergedRel) -> DirEntryDiff {
        let mut counts = std::mem::take(&mut self.counts);
        let entry = match (m.a, m.b) {
            (None, None) => unreachable!("merged rels exist on at least one side"),
            (Some((sa, isd, _)), None) => {
                counts.deleted += 1;
                DirEntryDiff {
                    rel: m.rel.clone(),
                    status: DirStatus::Deleted,
                    size_a: sa,
                    size_b: 0,
                    is_dir: isd,
                    note: String::new(),
                    pair: None,
                }
            }
            (None, Some((sb, isd, _))) => {
                counts.added += 1;
                DirEntryDiff {
                    rel: m.rel.clone(),
                    status: DirStatus::Added,
                    size_a: 0,
                    size_b: sb,
                    is_dir: isd,
                    note: String::new(),
                    pair: None,
                }
            }
            (Some((sa, da, ma)), Some((sb, db, mb))) => {
                let base = DirEntryDiff {
                    rel: m.rel.clone(),
                    status: DirStatus::Same,
                    size_a: sa,
                    size_b: sb,
                    is_dir: da,
                    note: String::new(),
                    pair: Some((m.path_a.clone(), m.path_b.clone())),
                };
                if da != db {
                    // ② kind conflict.
                    counts.modified += 1;
                    DirEntryDiff { status: DirStatus::Modified, ..base }
                } else if da {
                    counts.same += 1;
                    base // ② same kind dir → same
                } else if self.opts.mtime_fast_path && sa == sb && ma.is_some() && ma == mb {
                    counts.same += 1;
                    DirEntryDiff { note: "uncompared".to_string(), ..base }
                } else if Self::reads_binary(&m.path_a, sa) || Self::reads_binary(&m.path_b, sb) {
                    // ③ binary heuristic (any side hit → whole entry binary).
                    counts.binary += 1;
                    DirEntryDiff { status: DirStatus::Binary, ..base }
                } else if sa != sb {
                    // ④ size difference.
                    counts.modified += 1;
                    DirEntryDiff { status: DirStatus::Modified, ..base }
                } else if sa > CONTENT_COMPARE_CAP {
                    // ⑤ >2MB same size: downstream-visible uncompared state.
                    counts.same += 1;
                    DirEntryDiff { note: "uncompared".to_string(), ..base }
                } else {
                    // ⑤ ≤2MB byte equality.
                    match (fs::read(&m.path_a), fs::read(&m.path_b)) {
                        (Ok(x), Ok(y)) if x == y => {
                            counts.same += 1;
                            base
                        }
                        _ => {
                            counts.modified += 1;
                            DirEntryDiff { status: DirStatus::Modified, ..base }
                        }
                    }
                }
            }
        };
        self.counts = counts;
        entry
    }
}

impl Iterator for DirDiffIter {
    type Item = DirEntryDiff;

    fn next(&mut self) -> Option<Self::Item> {
        let m = self.merged.get(self.idx)?;
        let m = MergedRel {
            rel: m.rel.clone(),
            a: m.a,
            b: m.b,
            path_a: m.path_a.clone(),
            path_b: m.path_b.clone(),
        };
        self.idx += 1;
        Some(self.classify(&m))
    }
}

// ---------------------------------------------------------------------------
// Tests (AC-06: nested / rename-similar / binary-mixed state classification)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct TempTree(PathBuf);
    impl TempTree {
        fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!("p703_dirdiff_{tag}_{}", std::process::id()));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            TempTree(p)
        }
        fn write(&self, rel: &str, content: &[u8]) {
            let path = self.0.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }
        fn mkdir(&self, rel: &str) {
            fs::create_dir_all(self.0.join(rel)).unwrap();
        }
    }
    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn collect(iter: &mut DirDiffIter) -> Vec<DirEntryDiff> {
        let mut v: Vec<DirEntryDiff> = iter.by_ref().collect();
        v.sort_by(|x, y| x.rel.cmp(&y.rel));
        v
    }

    #[test]
    fn existence_added_deleted_and_dirs() {
        let a = TempTree::new("exist_a");
        let b = TempTree::new("exist_b");
        a.write("only_a.txt", b"A");
        b.write("only_b.txt", b"B");
        a.write("same.txt", b"same");
        b.write("same.txt", b"same");
        a.mkdir("dir_a");
        b.mkdir("dir_both");
        b.write("dir_both/inner.txt", b"x");

        let mut it = DirDiffIter::new(&a.0, &b.0, DirDiffOptions::default()).unwrap();
        let entries = collect(&mut it);
        let counts = it.counts();
        let find = |rel: &str| entries.iter().find(|e| e.rel == rel).unwrap();
        assert_eq!(find("only_a.txt").status, DirStatus::Deleted);
        assert_eq!(find("only_b.txt").status, DirStatus::Added);
        assert_eq!(find("same.txt").status, DirStatus::Same);
        assert_eq!(find("dir_a").status, DirStatus::Deleted);
        assert!(find("dir_a").is_dir);
        assert_eq!(find("dir_both").status, DirStatus::Added, "dir exists only on b");
        assert_eq!(find("dir_both/inner.txt").status, DirStatus::Added);
        assert_eq!((counts.added, counts.deleted, counts.same), (3, 2, 1));
    }

    #[test]
    fn modified_size_and_content_and_binary() {
        let a = TempTree::new("mod_a");
        let b = TempTree::new("mod_b");
        a.write("size.txt", b"short");
        b.write("size.txt", b"shorter!");
        a.write("content.txt", b"one");
        b.write("content.txt", b"two");
        // Invalid UTF-8 → binary (within the ≤2MB domain).
        a.write("bin.dat", &[0xFF, 0xFE, 0x01]);
        b.write("bin.dat", &[0xFF, 0xFE, 0x02]);
        // Empty files: size 0 both → same (size>0 gate keeps empty from binary).
        a.write("empty.txt", b"");
        b.write("empty.txt", b"");

        let mut it = DirDiffIter::new(&a.0, &b.0, DirDiffOptions::default()).unwrap();
        let entries = collect(&mut it);
        let find = |rel: &str| entries.iter().find(|e| e.rel == rel).unwrap();
        assert_eq!(find("size.txt").status, DirStatus::Modified);
        assert_eq!(find("content.txt").status, DirStatus::Modified);
        assert_eq!(find("bin.dat").status, DirStatus::Binary);
        assert_eq!(find("empty.txt").status, DirStatus::Same);
    }

    #[test]
    fn uncompared_note_over_2mb() {
        let a = TempTree::new("big_a");
        let b = TempTree::new("big_b");
        let big = vec![b'x'; CONTENT_COMPARE_CAP as usize + 1];
        a.write("big.bin", &big);
        b.write("big.bin", &big);
        let mut it = DirDiffIter::new(&a.0, &b.0, DirDiffOptions::default()).unwrap();
        let entries = collect(&mut it);
        let e = entries.iter().find(|e| e.rel == "big.bin").unwrap();
        assert_eq!(e.status, DirStatus::Same);
        assert_eq!(e.note, "uncompared");
    }

    #[test]
    fn skiplist_hidden_and_build_dirs() {
        let a = TempTree::new("skip_a");
        let b = TempTree::new("skip_b");
        a.write("target/out.js", b"a");
        b.write("target/out.js", b"b");
        a.write(".hidden", b"a");
        b.write("node_modules/pkg.js", b"b");
        a.write("visible.txt", b"v");
        b.write("visible.txt", b"v");
        let mut it = DirDiffIter::new(&a.0, &b.0, DirDiffOptions::default()).unwrap();
        let entries = collect(&mut it);
        assert_eq!(entries.len(), 1, "only visible.txt survives the skip-list: {:?}", entries);
        assert_eq!(entries[0].rel, "visible.txt");
        assert_eq!(entries[0].status, DirStatus::Same);
    }

    #[test]
    fn lazy_iteration_can_stop_early() {
        let a = TempTree::new("lazy_a");
        let b = TempTree::new("lazy_b");
        for i in 0..50 {
            a.write(&format!("f{i:02}.txt"), format!("v{i}").as_bytes());
            b.write(&format!("f{i:02}.txt"), format!("v{i}").as_bytes());
        }
        let mut it = DirDiffIter::new(&a.0, &b.0, DirDiffOptions::default()).unwrap();
        let first = it.next().unwrap();
        assert_eq!(first.status, DirStatus::Same);
        assert_eq!(it.remaining(), 49, "iterator is lazy — 49 entries pending");
        // Counts accumulate only over classified entries.
        assert_eq!(it.counts().same, 1);
    }

    #[test]
    fn missing_root_is_an_error() {
        let a = TempTree::new("miss_a");
        a.write("x.txt", b"x");
        let ghost = a.0.join("nope");
        let err = match DirDiffIter::new(&a.0, &ghost, DirDiffOptions::default()) {
            Err(e) => e,
            Ok(_) => panic!("missing root must error"),
        };
        assert!(matches!(err, DirDiffError::RootMissing(_)));
    }

    #[test]
    fn lazy_hunks_on_modified_entry() {
        let a = TempTree::new("hunk_a");
        let b = TempTree::new("hunk_b");
        a.write("code.rs", b"fn a() {}\nfn b() {}\n");
        b.write("code.rs", b"fn a() {}\nfn c() {}\n");
        let mut it = DirDiffIter::new(&a.0, &b.0, DirDiffOptions::default()).unwrap();
        let e = it.next().unwrap();
        assert_eq!(e.status, DirStatus::Modified);
        let hunks = e.hunks(DiffOpts::default()).expect("modified file yields hunks");
        assert_eq!(hunks.dels, 1);
        assert_eq!(hunks.adds, 1);
    }
}
