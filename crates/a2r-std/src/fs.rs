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
/// Plan 367 (consumer-mode parity): aligning the a2r backend's error
/// convention with the VM's keeps three-way parity well-defined (the `.at`
/// source is written once and must behave identically across VM/a2r/Rust).
pub fn read_to_string(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// Read text content from a file (alias).
///
/// See `read_to_string`: returns empty string on error (VM parity).
pub fn read_text(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// Write text content to a file, returns true on success
pub fn write(path: &str, content: &str) -> bool {
    std::fs::write(path, content).is_ok()
}

/// Write text content to a file (alias), returns 0 on success, -1 on failure
pub fn write_text(path: &str, content: &str) -> i32 {
    if std::fs::write(path, content).is_ok() { 0 } else { -1 }
}

/// Read file contents as bytes
pub fn read_bytes(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_default()
}

/// Write bytes to a file, returns 0 on success, -1 on failure
pub fn write_bytes(path: &str, bytes: &[u8]) -> i32 {
    if std::fs::write(path, bytes).is_ok() { 0 } else { -1 }
}

// ═══════════════════════════════════════════════════════════
// File Management
// ═══════════════════════════════════════════════════════════

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn delete(path: &str) -> i32 {
    if std::fs::remove_file(path).is_ok() { 0 } else { -1 }
}

pub fn copy(src: &str, dst: &str) -> i32 {
    if std::fs::copy(src, dst).is_ok() { 0 } else { -1 }
}

pub fn size(path: &str) -> i64 {
    std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(-1)
}

// ═══════════════════════════════════════════════════════════
// Directory Operations
// ═══════════════════════════════════════════════════════════

pub fn create_dir(path: &str) -> i32 {
    if std::fs::create_dir_all(path).is_ok() { 0 } else { -1 }
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
            if f.write_all(content.as_bytes()).is_ok() { 0 } else { -1 }
        }
        Err(_) => -1,
    }
}

/// Alias: mkdir_all → create_dir_all
pub fn mkdir_all(path: &str) -> i32 {
    create_dir(path)
}
