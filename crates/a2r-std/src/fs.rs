/// File system operations
/// Transpiled from auto-lang/stdlib/auto/file.at + file.rs.at
use std::io::Write as IoWrite;
use std::path::Path;

// ═══════════════════════════════════════════════════════════
// File Read/Write
// ═══════════════════════════════════════════════════════════

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
pub fn read_text_range(path: impl AsRef<Path>, offset: usize, limit: usize) -> String {
    #[derive(serde::Serialize)]
    struct ReadTextRangeOut {
        text: String,
        total: i64,
        next_offset: Option<u64>,
    }
    let err = || {
        serde_json::to_string(&ReadTextRangeOut { text: String::new(), total: -1, next_offset: None })
            .expect("read_text_range envelope serialization cannot fail")
    };
    if limit == 0 {
        return err();
    }
    let bytes = match std::fs::read(path.as_ref()) {
        Ok(b) => b,
        Err(_) => return err(),
    };
    let total = bytes.len() as i64;
    let text = match String::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => return err(),
    };
    let len = text.len();
    let mut start = offset.min(len);
    while !text.is_char_boundary(start) {
        start -= 1;
    }
    if start >= len {
        // offset past end (empty file included): EOF shape, real total.
        return serde_json::to_string(&ReadTextRangeOut { text: String::new(), total, next_offset: None })
            .expect("read_text_range envelope serialization cannot fail");
    }
    let mut end = start.saturating_add(limit).min(len);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let chunk = text[start..end].to_string();
    let next_offset = if end >= len { None } else { Some(end as u64) };
    serde_json::to_string(&ReadTextRangeOut { text: chunk, total, next_offset })
        .expect("read_text_range envelope serialization cannot fail")
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
