/// File system operations
/// Transpiled from auto-lang/stdlib/auto/file.at + file.rs.at
use std::io::Write as IoWrite;
use std::path::Path;

// ═══════════════════════════════════════════════════════════
// File Read/Write
// ═══════════════════════════════════════════════════════════

/// Final path segment, treating both `/` and `\` as separators.
///
/// PLAN-681 (dual-track parity): mirrors the AutoVM `file_basename` native
/// (`shim_file_basename` — `path.rsplit(['/', '\\']).next()`), so tab titles
/// and derived names agree across VM/a2r for the same `.at` source.
pub fn basename(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or("").to_string()
}

/// Recursive directory tree as nested JSON aligned with the TreeView node
/// schema: `{id,label,children,kind,icon,is_leaf,badge}` per node, wrapped in
/// a top-level array. `id` is the path relative to `root` ('/'-separated).
///
/// PLAN-681 (dual-track parity): byte-identical with the AutoVM `fs.tree`
/// native (`shim_fs_tree` / `fs_tree_walk` in vm/native.rs) — same skip list
/// (`.git`/`target`/`build`/`node_modules`/`gen`/`dist`/`__pycache__` and all
/// dotfiles), same ordering (dirs first, then case-insensitive name), same
/// `max_depth` clamp (1..=8, where 1 = root entries only) and same empty
/// result (`[]`). The front side parses this JSON via `json.to_value`, so any
/// shape divergence between the two tracks would fork the app behavior.
pub fn tree(root: &str, max_depth: i32) -> String {
    fn skipped(name: &str) -> bool {
        matches!(
            name,
            ".git" | "target" | "build" | "node_modules" | "gen" | "dist" | "__pycache__"
        ) || name.starts_with('.')
    }

    fn walk(dir: &Path, root: &Path, depth: usize, out: &mut String) {
        let mut entries: Vec<std::fs::DirEntry> = match std::fs::read_dir(dir) {
            Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
            Err(_) => return,
        };
        entries.sort_by_key(|e| {
            let is_dir = e.path().is_dir();
            (!is_dir, e.file_name().to_string_lossy().to_lowercase())
        });
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if skipped(&name) {
                continue;
            }
            let path = entry.path();
            let is_dir = path.is_dir();
            let rel = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| name.clone());
            let id = serde_json::to_string(&rel).unwrap_or_else(|_| "\"\"".into());
            let label = serde_json::to_string(&name).unwrap_or_else(|_| "\"\"".into());
            if !out.is_empty() {
                out.push(',');
            }
            if is_dir && depth > 1 {
                let mut children = String::new();
                walk(&path, root, depth - 1, &mut children);
                out.push_str(&format!(
                    "{{\"id\":{},\"label\":{},\"children\":[{}],\"kind\":\"dir\",\"icon\":\"folder\",\"is_leaf\":false,\"badge\":\"\"}}",
                    id, label, children
                ));
            } else if is_dir {
                out.push_str(&format!(
                    "{{\"id\":{},\"label\":{},\"children\":[],\"kind\":\"dir\",\"icon\":\"folder\",\"is_leaf\":false,\"badge\":\"\"}}",
                    id, label
                ));
            } else {
                out.push_str(&format!(
                    "{{\"id\":{},\"label\":{},\"children\":[],\"kind\":\"file\",\"icon\":\"file-text\",\"is_leaf\":true,\"badge\":\"\"}}",
                    id, label
                ));
            }
        }
    }

    let depth = max_depth.clamp(1, 8) as usize;
    let root_path = Path::new(root);
    let mut buf = String::new();
    if root_path.is_dir() {
        walk(root_path, root_path, depth, &mut buf);
    }
    if buf.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", buf)
    }
}

/// Read text content from a file.
///
/// Returns the file contents on success, or an empty string on error —
/// matching the AutoVM `auto.fs.read_text` / `auto.file.read_text` native
/// (`shim_file_read_text` uses `read_to_string(...).unwrap_or_default()`).
/// Plan 368 (consumer-mode parity): aligning the a2r backend's error
/// convention with the VM's keeps three-way parity well-defined (the `.at`
/// source is written once and must behave identically across VM/a2r/Rust).
pub fn read_to_string(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path.as_ref()).unwrap_or_default()
}

/// Read text content from a file (alias).
///
/// See `read_to_string`: returns empty string on error (VM parity).
pub fn read_text(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path.as_ref()).unwrap_or_default()
}

/// Read at most `limit` bytes of UTF-8 text starting at byte `offset`,
/// returned as the Plan 673 §5 JSON envelope
/// (`{"text":...,"total":<file byte len>,"next_offset":<bytes>|null}`).
///
/// P670-D1 (dual-track parity): byte-identical with the AutoVM
/// `auto.file.read_text_range` native (`shim_file_read_text_range`) — same
/// serde shape and field order, same EOF/error rules (`next_offset` null
/// ONLY at true EOF so disk-tail short reads are not mistaken for EOF;
/// IO error or invalid UTF-8 → `total:-1` shape). Chunk edges never split
/// a char: a mid-char `offset` walks back to the preceding boundary and the
/// chunk end backs off likewise. Plan 368 parity note applies as well: the
/// `.at` source is written once and must behave identically across
/// VM/a2r/Rust.
///
/// PLAN-687 true chunked IO: the window actually READ from disk is
/// `[offset − 3, min(total, offset + limit + 1))` via `File::seek` +
/// exact-size read (metadata supplies `total` — no whole-file `fs::read`),
/// so a chunked scan costs O(limit) IO, not O(file) per call. Char-boundary
/// back-off walks the same window's continuation bytes (0b10xxxxxx) —
/// equivalent to the previous whole-text `is_char_boundary` walks.
/// **Streaming semantic note**: UTF-8 validation is per-chunk-region now;
/// invalid bytes outside the requested window surface as `total:-1` only
/// when their containing chunk is read (previously the whole file was
/// validated up front).
pub fn read_text_range(path: impl AsRef<Path>, offset: usize, limit: usize) -> String {
    #[derive(serde::Serialize)]
    struct ReadTextRangeOut {
        text: String,
        total: i64,
        next_offset: Option<u64>,
    }
    let ok = |text: String, total: i64, next_offset: Option<u64>| {
        serde_json::to_string(&ReadTextRangeOut { text, total, next_offset })
            .expect("read_text_range envelope serialization cannot fail")
    };
    let err = || ok(String::new(), -1, None);
    if limit == 0 {
        return err();
    }
    use std::io::{Read, Seek, SeekFrom};
    let mut file = match std::fs::File::open(path.as_ref()) {
        Ok(f) => f,
        Err(_) => return err(),
    };
    let total = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return err(),
    };
    let start = offset.min(total as usize) as u64;
    if start >= total {
        // offset past end (empty file included): EOF shape, real total.
        return ok(String::new(), total as i64, None);
    }
    let end = (start as usize).saturating_add(limit).min(total as usize) as u64;
    // Read window with boundary margins: 3 bytes before `start` (max UTF-8
    // continuation run) + 1 byte at `end` (to decide whether `end` sits
    // inside a multi-byte char).
    let win_start = start.saturating_sub(3);
    let win_end = (end + 1).min(total);
    let mut win = vec![0u8; (win_end - win_start) as usize];
    if file.seek(SeekFrom::Start(win_start)).is_err() {
        return err();
    }
    if file.read_exact(&mut win).is_err() {
        return err();
    }
    let is_cont = |b: u8| b & 0xC0 == 0x80;
    let mut s = (start - win_start) as usize;
    while s > 0 && is_cont(win[s]) {
        s -= 1;
    }
    let mut e = (end - win_start) as usize;
    if e < win.len() {
        // The byte AT `end` exists (end < total): back `end` off while it
        // points inside a multi-byte char.
        while e > s && is_cont(win[e]) {
            e -= 1;
        }
    }
    let text = match String::from_utf8(win[s..e].to_vec()) {
        Ok(t) => t,
        Err(_) => return err(),
    };
    let end_abs = (e + win_start as usize) as u64;
    let next_offset = if end_abs >= total { None } else { Some(end_abs) };
    ok(text, total as i64, next_offset)
}

/// Write text content to a file, returns true on success
pub fn write(path: &str, content: &str) -> bool {
    std::fs::write(path, content).is_ok()
}

/// Write text content to a file (alias), returns 0 on success, -1 on failure
pub fn write_text(path: &str, content: &str) -> i32 {
    if std::fs::write(path, content).is_ok() {
        0
    } else {
        -1
    }
}

/// Read file contents as bytes
pub fn read_bytes(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_default()
}

/// Write bytes to a file, returns 0 on success, -1 on failure
pub fn write_bytes(path: &str, bytes: &[u8]) -> i32 {
    if std::fs::write(path, bytes).is_ok() {
        0
    } else {
        -1
    }
}

// ═══════════════════════════════════════════════════════════
// File Management
// ═══════════════════════════════════════════════════════════

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn delete(path: &str) -> i32 {
    if std::fs::remove_file(path).is_ok() {
        0
    } else {
        -1
    }
}

pub fn copy(src: &str, dst: &str) -> i32 {
    if std::fs::copy(src, dst).is_ok() {
        0
    } else {
        -1
    }
}

pub fn size(path: &str) -> i64 {
    std::fs::metadata(path)
        .map(|m| m.len() as i64)
        .unwrap_or(-1)
}

// ═══════════════════════════════════════════════════════════
// Directory Operations
// ═══════════════════════════════════════════════════════════

pub fn create_dir(path: &str) -> i32 {
    if std::fs::create_dir_all(path).is_ok() {
        0
    } else {
        -1
    }
}

pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn append_text(path: &str, content: &str) -> i32 {
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        Ok(mut f) => {
            if f.write_all(content.as_bytes()).is_ok() {
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

/// Alias: mkdir_all → create_dir_all
pub fn mkdir_all(path: &str) -> i32 {
    create_dir(path)
}

#[cfg(test)]
mod read_text_range_687_tests {
    use super::read_text_range;

    fn envelope_text(env: &str) -> String {
        let v: serde_json::Value = serde_json::from_str(env).unwrap();
        v["text"].as_str().unwrap().to_string()
    }
    fn envelope_next(env: &str) -> Option<u64> {
        let v: serde_json::Value = serde_json::from_str(env).unwrap();
        v["next_offset"].as_u64()
    }

    /// PLAN-687 AC-2: true chunked IO — multi-block scan reassembles the
    /// original file exactly (8 MB temp file, 3 MB blocks).
    #[test]
    fn multi_block_scan_reassembles_file() {
        let dir = std::env::temp_dir().join("a2r_std_rtr_687");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("big.txt");
        let unit = "0123456789abcdefghijklmnopqrstuvwxyz你好世界\n"; // ASCII + CJK mix
        let target = 8 * 1024 * 1024;
        let mut content = String::with_capacity(target + unit.len());
        while content.len() < target {
            content.push_str(unit);
        }
        std::fs::write(&p, content.as_bytes()).unwrap();
        let mut got = String::new();
        let mut off: usize = 0;
        loop {
            let env = read_text_range(&p, off, 3 * 1024 * 1024);
            got.push_str(&envelope_text(&env));
            match envelope_next(&env) {
                None => break,
                Some(n) => off = n as usize,
            }
        }
        assert_eq!(got.len(), content.len());
        assert_eq!(got, content);
        std::fs::remove_file(&p).ok();
    }

    /// PLAN-687 AC-2 streaming note: UTF-8 validation is per-chunk-region —
    /// invalid bytes in block 2 don't poison block 1's read, and surface as
    /// the total:-1 error shape only when their containing chunk is read.
    #[test]
    fn invalid_utf8_validated_per_chunk_region() {
        let dir = std::env::temp_dir().join("a2r_std_rtr_687");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("bad.txt");
        // Block 1 (8 valid bytes) + invalid 0xFF at byte 8.
        std::fs::write(&p, b"abcdefgh\xFFij").unwrap();
        let first = read_text_range(&p, 0, 8);
        assert_eq!(envelope_text(&first), "abcdefgh");
        assert_eq!(envelope_next(&first), Some(8));
        let second = read_text_range(&p, 8, 8);
        let v: serde_json::Value = serde_json::from_str(&second).unwrap();
        assert_eq!(v["total"], serde_json::json!(-1));
        std::fs::remove_file(&p).ok();
    }
}
