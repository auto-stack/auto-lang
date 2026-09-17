//! Backend-neutral local media service (Plan 617 T-05).
//!
//! Supplies the two primitives a real video player needs from its backend:
//!
//! * a **recursive index** of a media root, and
//! * **HTTP byte-range** semantics so `<video>` can seek a multi-GB file
//!   without the server ever reading it whole.
//!
//! Deliberately dependency-light and synchronous: it does no I/O on behalf of a
//! UI thread, spawns nothing, and knows nothing about axum/tokio. The generated
//! backend turns a [`StreamPlan`] into an actual streamed response; tests can
//! cover the whole decision surface without a socket.
//!
//! Shape is mirrored from `crates/auto-man/src/image_viewer.rs` (the image
//! viewer's proven index): recursive collect, relative paths with `/`
//! separators, natural ordering, a blake3 token instead of an exposed absolute
//! path. Only the extension whitelist and the byte-range part are new.

use std::cmp::Ordering;
use std::io;
use std::path::{Path, PathBuf};

/// Extensions offered as *candidates*. Whether a given file actually decodes is
/// the browser's (or the native decoder's) call — the index never promises
/// playability. `.mkv` is included on purpose: Matroska *demuxing* works in
/// Chromium, and excluding it would hide the very files the user cares about.
/// Audio extensions (`.mp3`, `.flac`, `.wav`, etc.) are also supported for music players.
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "webm", "mkv", "mov", "avi",
    "mp3", "flac", "wav", "ogg", "m4a", "aac",
];

/// Hard cap on directory depth, so a pathological tree cannot spin forever even
/// if a symlink loop somehow slips past the symlink guard.
const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaEntry {
    /// blake3 of `relative_path` — the only identifier that leaves the backend.
    /// Absolute paths are never serialised (mirrors `image_viewer::redact_path`).
    pub id: String,
    pub name: String,
    /// Parent directory relative to the root, `/`-separated, `""` at the top.
    pub rel_dir: String,
    pub relative_path: String,
    pub extension: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct MediaIndex {
    /// Redacted root label (the root itself, not a per-file path).
    pub root: String,
    pub entries: Vec<MediaEntry>,
}

/// Resolve the media root. Order: explicit argument (pac.at) -> `AUTO_MEDIA_ROOT`
/// -> the platform default. Returns `None` when nothing usable is configured, so
/// callers can render an honest empty state instead of guessing.
pub fn resolve_root(explicit: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = explicit.map(str::trim).filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    if let Ok(p) = std::env::var("AUTO_MEDIA_ROOT") {
        let p = p.trim().to_string();
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    None
}

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Recursive collect. **Never follows symlinks** — a junction pointing back up
/// the tree would otherwise recurse forever, and `git worktree remove` has been
/// burned by links-through before (Plan 529).
fn collect_files(root: &Path, dir: &Path, depth: usize, out: &mut Vec<(PathBuf, u64)>) -> io::Result<()> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    for item in std::fs::read_dir(dir)? {
        let item = match item {
            Ok(i) => i,
            Err(_) => continue, // unreadable entry: skip, do not fail the whole scan
        };
        let path = item.path();
        // symlink_metadata does not traverse the link, so we can detect and skip it.
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            let _ = collect_files(root, &path, depth + 1, out);
        } else if meta.is_file() && is_supported(&path) && path.strip_prefix(root).is_ok() {
            out.push((path, meta.len()));
        }
    }
    Ok(())
}

/// Natural ordering: `S01E02` before `S01E10`. Same intent as
/// `image_viewer::natural_sort_key`, kept local so this module stays standalone.
fn natural_key(s: &str) -> Vec<NaturalPart> {
    let mut parts = Vec::new();
    let mut num = String::new();
    let mut txt = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            if !txt.is_empty() {
                parts.push(NaturalPart::Text(txt.to_lowercase()));
                txt.clear();
            }
            num.push(ch);
        } else {
            if !num.is_empty() {
                parts.push(NaturalPart::Number(num.parse::<u128>().unwrap_or(u128::MAX)));
                num.clear();
            }
            txt.push(ch);
        }
    }
    if !num.is_empty() {
        parts.push(NaturalPart::Number(num.parse::<u128>().unwrap_or(u128::MAX)));
    }
    if !txt.is_empty() {
        parts.push(NaturalPart::Text(txt.to_lowercase()));
    }
    parts
}

#[derive(Debug, PartialEq, Eq)]
enum NaturalPart {
    Number(u128),
    Text(String),
}

impl Ord for NaturalPart {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (NaturalPart::Number(a), NaturalPart::Number(b)) => a.cmp(b),
            (NaturalPart::Text(a), NaturalPart::Text(b)) => a.cmp(b),
            // A number sorts before trailing text ("E02" < "E02v2").
            (NaturalPart::Number(_), NaturalPart::Text(_)) => Ordering::Less,
            (NaturalPart::Text(_), NaturalPart::Number(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for NaturalPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Recursively index `root` into entries sorted naturally by relative path.
pub fn index_directory(root: impl AsRef<Path>) -> io::Result<MediaIndex> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_files(root, root, 0, &mut files)?;
    files.sort_by(|(a, _), (b, _)| {
        let a_rel = a.strip_prefix(root).unwrap_or(a).to_string_lossy().replace('\\', "/");
        let b_rel = b.strip_prefix(root).unwrap_or(b).to_string_lossy().replace('\\', "/");
        natural_key(&a_rel).cmp(&natural_key(&b_rel)).then_with(|| a_rel.cmp(&b_rel))
    });
    let entries = files
        .into_iter()
        .map(|(path, bytes)| {
            let relative_path = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            let rel_dir = relative_path
                .rsplit_once('/')
                .map(|(d, _)| d.to_string())
                .unwrap_or_default();
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let id = blake3::hash(relative_path.as_bytes()).to_hex().to_string();
            MediaEntry { id, name, rel_dir, relative_path, extension, bytes }
        })
        .collect();
    Ok(MediaIndex { root: root.to_string_lossy().to_string(), entries })
}

/// Reverse lookup by token. The request never carries a path, so a client
/// cannot ask for a file outside the indexed set.
pub fn find<'a>(index: &'a MediaIndex, id: &str) -> Option<&'a MediaEntry> {
    index.entries.iter().find(|e| e.id == id)
}

/// Absolute path of an entry, re-joined under the index root.
pub fn entry_path(root: &Path, entry: &MediaEntry) -> PathBuf {
    let mut p = root.to_path_buf();
    for part in entry.relative_path.split('/') {
        p.push(part);
    }
    p
}

pub fn content_type(extension: &str) -> &'static str {
    match extension {
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        _ => "application/octet-stream",
    }
}

/// Outcome of interpreting a `Range` header against a known file length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamPlan {
    /// No (or unusable) Range header: send the whole file as 200.
    Full,
    /// Inclusively bounded window: 206 with `Content-Range`.
    Partial { start: u64, end: u64 },
    /// Syntactically valid but unsatisfiable: 416 with `bytes */len`.
    Unsatisfiable,
}

impl StreamPlan {
    pub fn status(&self) -> u16 {
        match self {
            StreamPlan::Full => 200,
            StreamPlan::Partial { .. } => 206,
            StreamPlan::Unsatisfiable => 416,
        }
    }
    pub fn content_length(&self, file_len: u64) -> u64 {
        match self {
            StreamPlan::Full => file_len,
            StreamPlan::Partial { start, end } => end - start + 1,
            StreamPlan::Unsatisfiable => 0,
        }
    }
}

/// Parse a single-range `Range` header. Multi-range is deliberately treated as
/// unusable (full response) rather than mis-served — `<video>` only asks for one.
pub fn parse_range(header: Option<&str>, file_len: u64) -> StreamPlan {
    let raw = match header.map(str::trim).filter(|h| !h.is_empty()) {
        Some(h) => h,
        None => return StreamPlan::Full,
    };
    let spec = match raw.strip_prefix("bytes=") {
        Some(s) => s.trim(),
        None => return StreamPlan::Full, // unknown unit: RFC allows ignoring it
    };
    if spec.contains(',') {
        return StreamPlan::Full;
    }
    let (a, b) = match spec.split_once('-') {
        Some(p) => p,
        None => return StreamPlan::Full,
    };
    if file_len == 0 {
        return StreamPlan::Unsatisfiable;
    }
    match (a.trim(), b.trim()) {
        // bytes=N-
        ("", "") => StreamPlan::Full,
        (start, "") => match start.parse::<u64>() {
            Ok(s) if s < file_len => StreamPlan::Partial { start: s, end: file_len - 1 },
            Ok(_) => StreamPlan::Unsatisfiable,
            Err(_) => StreamPlan::Full,
        },
        // bytes=-N (suffix)
        ("", suffix) => match suffix.parse::<u64>() {
            Ok(0) => StreamPlan::Unsatisfiable,
            Ok(n) => {
                let start = file_len.saturating_sub(n);
                StreamPlan::Partial { start, end: file_len - 1 }
            }
            Err(_) => StreamPlan::Full,
        },
        (start, end) => match (start.parse::<u64>(), end.parse::<u64>()) {
            (Ok(s), Ok(e)) if s <= e && s < file_len => {
                StreamPlan::Partial { start: s, end: e.min(file_len - 1) }
            }
            (Ok(_), Ok(_)) => StreamPlan::Unsatisfiable,
            _ => StreamPlan::Full,
        },
    }
}

/// `Content-Range` value for a partial response.
pub fn content_range(start: u64, end: u64, file_len: u64) -> String {
    format!("bytes {}-{}/{}", start, end, file_len)
}

/// Human-readable size for display ("7.3 MB", "5.4 GB").
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", v, UNITS[i])
    }
}

/// Display title: the file name without its extension.
pub fn display_title(name: &str) -> &str {
    match name.rfind('.') {
        Some(i) if i > 0 => &name[..i],
        _ => name,
    }
}

/// Parse artist and song title from a media filename (e.g. "阿杜 - 撕夜.flac").
pub fn parse_artist_and_title(filename: &str) -> (String, String) {
    let raw = display_title(filename).trim();
    if let Some((artist, title)) = raw.split_once(" - ") {
        let a = artist.trim();
        let t = title.trim();
        if !a.is_empty() && !t.is_empty() {
            return (a.to_string(), t.to_string());
        }
    }
    if let Some((artist, title)) = raw.split_once('-') {
        let a = artist.trim();
        let t = title.trim();
        if !a.is_empty() && !t.is_empty() && !a.chars().all(|c| c.is_ascii_digit()) {
            return (a.to_string(), t.to_string());
        }
    }
    // Check leading track numbers like "01 粉雪"
    let trimmed = raw.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == '-' || c == ' ');
    if !trimmed.is_empty() && trimmed.len() < raw.len() {
        return ("精选音乐".to_string(), trimmed.trim().to_string());
    }
    ("本地音乐".to_string(), raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_tree() -> PathBuf {
        let base = std::env::temp_dir().join(format!("p617_media_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        // mirrors the real E:\Video shape: a top-level file plus a nested set
        // whose natural order differs from lexicographic order
        std::fs::create_dir_all(base.join("TV").join("Loki")).unwrap();
        for f in [
            "caelestia.mp4",
            "TV/龙猫.mp4",
            "TV/Loki/Loki.S01E10.mp4",
            "TV/Loki/Loki.S01E02.mp4",
            "TV/Loki/Loki.S02E01.mkv",
            "TV/Loki/notes.txt",           // whitelist must reject
            "TV/Loki/.hidden.md",          // whitelist must reject
        ] {
            let p = base.join(f);
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d).unwrap();
            }
            std::fs::write(&p, b"x").unwrap();
        }
        base
    }

    #[test]
    fn index_is_recursive_and_filtered() {
        let base = tmp_tree();
        let idx = index_directory(&base).unwrap();
        let names: Vec<_> = idx.entries.iter().map(|e| e.relative_path.clone()).collect();
        // recursion: nested files are present (a flat scan would see only 1)
        assert!(names.iter().any(|n| n == "TV/Loki/Loki.S01E02.mp4"), "{names:?}");
        assert!(names.iter().any(|n| n == "TV/龙猫.mp4"), "{names:?}");
        // whitelist
        assert!(!names.iter().any(|n| n.ends_with(".txt")), "{names:?}");
        assert!(!names.iter().any(|n| n.ends_with(".md")), "{names:?}");
        assert_eq!(idx.entries.len(), 5, "{names:?}");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn index_sorts_naturally_and_fills_rel_dir() {
        let base = tmp_tree();
        let idx = index_directory(&base).unwrap();
        let names: Vec<_> = idx.entries.iter().map(|e| e.name.clone()).collect();
        let e02 = names.iter().position(|n| n.contains("S01E02")).unwrap();
        let e10 = names.iter().position(|n| n.contains("S01E10")).unwrap();
        assert!(e02 < e10, "natural order broken (lexicographic would put E10 first): {names:?}");
        let loki = idx.entries.iter().find(|e| e.name.contains("S01E02")).unwrap();
        assert_eq!(loki.rel_dir, "TV/Loki");
        let top = idx.entries.iter().find(|e| e.name == "caelestia.mp4").unwrap();
        assert_eq!(top.rel_dir, "");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn ids_are_tokens_and_never_leak_absolute_paths() {
        let base = tmp_tree();
        let idx = index_directory(&base).unwrap();
        for e in &idx.entries {
            assert_eq!(e.id.len(), 64, "blake3 hex");
            assert!(!e.id.contains(':'), "no drive letters in the token");
            assert!(!e.relative_path.contains(base.to_string_lossy().as_ref()));
        }
        // lookup by token works and unknown tokens miss
        let first = idx.entries[0].id.clone();
        assert!(find(&idx, &first).is_some());
        assert!(find(&idx, "deadbeef").is_none());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn missing_root_is_an_error_not_a_panic() {
        let nope = std::env::temp_dir().join("p617_definitely_missing_root");
        let _ = std::fs::remove_dir_all(&nope);
        assert!(index_directory(&nope).is_err());
    }

    #[test]
    fn parse_range_covers_the_http_surface() {
        let len = 1000u64;
        assert_eq!(parse_range(None, len), StreamPlan::Full);
        assert_eq!(parse_range(Some(""), len), StreamPlan::Full);
        assert_eq!(parse_range(Some("bytes=0-1023"), len), StreamPlan::Partial { start: 0, end: 999 });
        assert_eq!(parse_range(Some("bytes=0-99"), len), StreamPlan::Partial { start: 0, end: 99 });
        assert_eq!(parse_range(Some("bytes=100-"), len), StreamPlan::Partial { start: 100, end: 999 });
        assert_eq!(parse_range(Some("bytes=-100"), len), StreamPlan::Partial { start: 900, end: 999 });
        // past the end -> 416, not a silent empty 206
        assert_eq!(parse_range(Some("bytes=1000-1200"), len), StreamPlan::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=-0"), len), StreamPlan::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=0-0"), 0), StreamPlan::Unsatisfiable);
        // multi-range and junk fall back to a full response rather than mis-serving
        assert_eq!(parse_range(Some("bytes=0-10,20-30"), len), StreamPlan::Full);
        assert_eq!(parse_range(Some("items=0-10"), len), StreamPlan::Full);
        assert_eq!(parse_range(Some("bytes=abc-def"), len), StreamPlan::Full);
        // end beyond EOF is clamped, which is what browsers rely on
        assert_eq!(parse_range(Some("bytes=0-99999"), len), StreamPlan::Partial { start: 0, end: 999 });
    }

    #[test]
    fn stream_plan_status_and_length() {
        assert_eq!(StreamPlan::Full.status(), 200);
        assert_eq!(StreamPlan::Partial { start: 0, end: 99 }.status(), 206);
        assert_eq!(StreamPlan::Unsatisfiable.status(), 416);
        assert_eq!(StreamPlan::Full.content_length(500), 500);
        assert_eq!(StreamPlan::Partial { start: 100, end: 199 }.content_length(500), 100);
        assert_eq!(content_range(0, 1023, 2198645361), "bytes 0-1023/2198645361");
    }

    #[test]
    fn content_type_map() {
        assert_eq!(content_type("mp4"), "video/mp4");
        assert_eq!(content_type("mkv"), "video/x-matroska");
        assert_eq!(content_type("mp3"), "audio/mpeg");
        assert_eq!(content_type("flac"), "audio/flac");
        assert_eq!(content_type("wav"), "audio/wav");
        assert_eq!(content_type("MP4"), "application/octet-stream"); // extension is pre-lowercased
        assert_eq!(content_type("bin"), "application/octet-stream");
    }
}
