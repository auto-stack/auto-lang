//! PLAN-043 Part 1: backend-neutral local photo service (029-photo-gallery).
//!
//! Mirrors `media_service.rs` (Plan 617) for still images: a **recursive
//! index** of a photo root plus **HTTP byte serving** of thumbnails and
//! originals. Thumbnails are rendered on demand (decode → EXIF orientation →
//! fit-in-box resize → JPEG) and disk-cached under the system temp dir, so a
//! directory with hundreds of photos pays the decode cost once per file.
//!
//! Deliberately dependency-light and synchronous like its sibling: no I/O on
//! behalf of a UI thread is done here beyond the explicit render call, and the
//! HTTP framing lives in the generated backend (auto-man api_gen) and the
//! desktop back-proxy — both of which already carry the `image-pipeline`
//! feature (which this module is gated on).
//!
//! Security boundary mirrors media_service: only blake3 tokens of relative
//! paths ever leave the backend; `find` + `entry_path` re-join under the
//! index root, so a request can never address a file outside the indexed set.

use std::cmp::Ordering;
use std::io;
use std::path::{Path, PathBuf};

/// Extensions offered as *candidates* (image crate decoder set). HEIC and
/// RAW formats are deliberately absent — the `image` crate cannot decode
/// them, and listing undecodable files would render broken tiles.
pub const PHOTO_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];

/// Hard cap on directory depth (symlink-loop guard, media_service precedent).
const MAX_DEPTH: usize = 32;

/// Decoder limits: photos from phones can be 100+MP, but a gallery thumbnail
/// never needs more than this to decode safely.
const MAX_IMAGE_DIM: u32 = 32_768;
const MAX_DECODE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoEntry {
    /// blake3 of `relative_path` — the only identifier that leaves the backend.
    pub id: String,
    pub name: String,
    /// Parent directory relative to the root, `/`-separated, `""` at the top.
    pub rel_dir: String,
    pub relative_path: String,
    pub extension: String,
    pub bytes: u64,
    /// Pixel dimensions from the container header (cheap; no full decode).
    /// Zero when the header could not be parsed — the frontend treats 0 as
    /// "dimensions unknown" and hides the meta segment.
    pub width: u32,
    pub height: u32,
    /// mtime seconds since epoch — the sort key ("date taken" proxy, same
    /// choice as the legacy bake pipeline) and part of the thumbnail cache
    /// key so edits invalidate cached renditions.
    pub mtime: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PhotoIndex {
    /// Redacted root label (the root itself, not a per-file path).
    pub root: String,
    pub entries: Vec<PhotoEntry>,
}

/// Resolve the photo root. Order: explicit argument (pac.at) ->
/// `AUTO_PHOTO_ROOT` env -> nothing (honest empty gallery).
pub fn resolve_root(explicit: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = explicit.map(str::trim).filter(|p| !p.is_empty()) {
        return Some(PathBuf::from(p));
    }
    if let Ok(p) = std::env::var("AUTO_PHOTO_ROOT") {
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
        .map(|e| PHOTO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Recursive collect. **Never follows symlinks** (media_service precedent —
/// a junction pointing back up the tree would recurse forever).
fn collect_files(root: &Path, dir: &Path, depth: usize, out: &mut Vec<(PathBuf, u64, i64)>) -> io::Result<()> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    for item in std::fs::read_dir(dir)? {
        let item = match item {
            Ok(i) => i,
            Err(_) => continue, // unreadable entry: skip, do not fail the whole scan
        };
        let path = item.path();
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
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            out.push((path, meta.len(), mtime));
        }
    }
    Ok(())
}

/// Natural ordering (S01E02 before S01E10) — media_service precedent, kept
/// local so this module stays standalone.
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

/// Pixel dimensions from the container header only — never decodes pixels.
fn header_dimensions(path: &Path) -> (u32, u32) {
    image::image_dimensions(path).unwrap_or((0, 0))
}

/// Recursively index `root` into entries sorted naturally by relative path.
pub fn index_directory(root: impl AsRef<Path>) -> io::Result<PhotoIndex> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_files(root, root, 0, &mut files)?;
    files.sort_by(|(a, _, _), (b, _, _)| {
        let a_rel = a.strip_prefix(root).unwrap_or(a).to_string_lossy().replace('\\', "/");
        let b_rel = b.strip_prefix(root).unwrap_or(b).to_string_lossy().replace('\\', "/");
        natural_key(&a_rel).cmp(&natural_key(&b_rel)).then_with(|| a_rel.cmp(&b_rel))
    });
    let entries = files
        .into_iter()
        .map(|(path, bytes, mtime)| {
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
            let (width, height) = header_dimensions(&path);
            let id = blake3::hash(relative_path.as_bytes()).to_hex().to_string();
            PhotoEntry { id, name, relative_path, rel_dir, extension, bytes, width, height, mtime }
        })
        .collect();
    Ok(PhotoIndex { root: root.to_string_lossy().to_string(), entries })
}

/// Reverse lookup by token. The request never carries a path, so a client
/// cannot ask for a file outside the indexed set.
pub fn find<'a>(index: &'a PhotoIndex, id: &str) -> Option<&'a PhotoEntry> {
    index.entries.iter().find(|e| e.id == id)
}

/// Absolute path of an entry, re-joined under the index root.
pub fn entry_path(root: &Path, entry: &PhotoEntry) -> PathBuf {
    let mut p = root.to_path_buf();
    for part in entry.relative_path.split('/') {
        p.push(part);
    }
    p
}

pub fn content_type(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}

/// Album label for an entry: first path segment of `rel_dir`, with root-level
/// files grouped under "photos" (matches the legacy gallery's top-level album).
pub fn album_of(entry: &PhotoEntry) -> &str {
    entry.rel_dir.split('/').next().filter(|s| !s.is_empty()).unwrap_or("photos")
}

/// Display title: the file name without its extension.
pub fn display_title(name: &str) -> &str {
    match name.rfind('.') {
        Some(i) if i > 0 => &name[..i],
        _ => name,
    }
}

// ---------------------------------------------------------------------------
// Thumbnails
// ---------------------------------------------------------------------------

/// `YYYY-MM-DD` from mtime — the sortable/displayable date column.
/// (Howard Hinnant's civil-from-days algorithm — no chrono.)
pub fn date_label(mtime: i64) -> String {
    let secs = if mtime < 0 { 0u64 } else { mtime as u64 };
    let days = (secs / 86_400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    // era 相对年（Hinnant 公式全程用相对值；era*400 只在最终年份上加）。
    let y_rel = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * y_rel + y_rel / 4 - y_rel / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = era * 400 + y_rel + if m <= 2 { 1 } else { 0 };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Human-readable size for display ("3.4 MB").
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

/// Disk cache directory for rendered thumbnails. Temp-dir based (per-user,
/// wiped by the OS on cleanup) — thumbnails are regenerable renditions, never
/// source data, so no durability requirement.
pub fn thumb_cache_dir() -> PathBuf {
    std::env::temp_dir().join("autoos-photo-thumbs")
}

/// Cache file name: token + mtime + width — an edit (mtime bump) or a width
/// change naturally misses the old entry; stale files are harmless leftovers
/// in a temp dir.
pub fn thumb_cache_name(id: &str, mtime: i64, width: u32) -> String {
    format!("{}-{}-{}.jpg", id, mtime, width)
}

/// Read EXIF orientation (1..=8) from a JPEG/TIFF container, if present.
/// (`kamadak-exif` keeps `exif` as its lib name — same path vocabulary as
/// `image_pipeline::inspect_image_metadata`.)
fn exif_orientation(path: &Path) -> Option<u8> {
    use exif::{In, Reader, Tag};
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exif = Reader::new().read_from_container(&mut bufreader).ok()?;
    exif.get_field(Tag::Orientation, In::PRIMARY)?
        .value
        .get_uint(0)
        .and_then(|v| u8::try_from(v).ok())
}

/// Apply an EXIF orientation (values 1..=8) to a decoded image. The
/// transform table mirrors `image_pipeline::oriented_image` (single source
/// of truth for the orientation vocabulary).
fn apply_orientation(img: image::DynamicImage, orientation: u8) -> image::DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        // Transpose/transverse are flips across the diagonal — compose from
        // the primitives the image crate offers (rotate + flip).
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

/// Render a `width`-px fit-in-box JPEG thumbnail. Serves from the disk cache
/// when a fresh entry exists; otherwise decodes (with limits), normalizes
/// EXIF orientation, resizes, encodes, and best-effort caches the result.
pub fn render_thumbnail(root: &Path, entry: &PhotoEntry, width: u32) -> Result<Vec<u8>, String> {
    let width = width.clamp(32, 1024);
    let cache = thumb_cache_dir().join(thumb_cache_name(&entry.id, entry.mtime, width));
    if let Ok(bytes) = std::fs::read(&cache) {
        if !bytes.is_empty() {
            return Ok(bytes);
        }
    }
    let path = entry_path(root, entry);
    let orientation = exif_orientation(&path).unwrap_or(1);
    let mut reader = image::ImageReader::open(&path)
        .and_then(|r| r.with_guessed_format())
        .map_err(|e| format!("open {}: {e}", entry.relative_path))?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIM);
    limits.max_image_height = Some(MAX_IMAGE_DIM);
    limits.max_alloc = Some(MAX_DECODE_BYTES);
    reader.limits(limits);
    let img = reader
        .decode()
        .map_err(|e| format!("decode {}: {e}", entry.relative_path))?;
    let img = apply_orientation(img, orientation);
    // Fit-in-box preserving aspect (PIL `thumbnail` semantics).
    let (w, h) = (img.width(), img.height());
    let scale = (width as f64 / w as f64).min(width as f64 / h as f64).min(1.0);
    let tw = ((w as f64 * scale).round() as u32).max(1);
    let th = ((h as f64 * scale).round() as u32).max(1);
    let thumb = img.thumbnail(tw, th);
    let mut out: Vec<u8> = Vec::new();
    thumb
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Jpeg)
        .map_err(|e| format!("encode: {e}"))?;
    if let Some(dir) = cache.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&cache, &out); // best-effort cache
    Ok(out)
}

// ================================ 测试 ================================

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_tree() -> PathBuf {
        let base = std::env::temp_dir().join(format!("p043_photo_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("Screenshots")).unwrap();
        std::fs::create_dir_all(base.join("Trip")).unwrap();
        // Tiny valid PNGs (1x1) so header dimension parsing has real input.
        let png_1px: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        for f in [
            "a.png", "b.PNG", "c.jpg", "notes.txt", "noext",
            "Screenshots/s2.png", "Screenshots/s10.png", "Trip/t1.webp",
        ] {
            let p = base.join(f);
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d).unwrap();
            }
            if f.ends_with(".png") || f.ends_with(".PNG") {
                std::fs::write(&p, png_1px).unwrap();
            } else {
                std::fs::write(&p, b"x").unwrap();
            }
        }
        base
    }

    #[test]
    fn index_is_recursive_filtered_and_dimensioned() {
        let base = tmp_tree();
        let idx = index_directory(&base).unwrap();
        let names: Vec<_> = idx.entries.iter().map(|e| e.relative_path.clone()).collect();
        assert!(names.iter().any(|n| n == "Trip/t1.webp"), "{names:?}");
        assert!(names.iter().any(|n| n == "Screenshots/s2.png"), "{names:?}");
        assert!(!names.iter().any(|n| n.ends_with(".txt")), "{names:?}");
        assert!(!names.iter().any(|n| n == "noext"), "{names:?}");
        assert_eq!(idx.entries.len(), 6, "{names:?}");
        // Header dimensions parsed for real PNGs (junk jpg header fails → 0, tolerated).
        let a = idx.entries.iter().find(|e| e.name == "a.png").unwrap();
        assert_eq!((a.width, a.height), (1, 1));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn natural_order_and_album_labels() {
        let base = tmp_tree();
        let idx = index_directory(&base).unwrap();
        let names: Vec<_> = idx.entries.iter().map(|e| e.name.clone()).collect();
        let s2 = names.iter().position(|n| n == "s2.png").unwrap();
        let s10 = names.iter().position(|n| n == "s10.png").unwrap();
        assert!(s2 < s10, "natural order broken: {names:?}");
        let root_file = idx.entries.iter().find(|e| e.name == "a.png").unwrap();
        assert_eq!(album_of(root_file), "photos");
        let shot = idx.entries.iter().find(|e| e.name == "s2.png").unwrap();
        assert_eq!(album_of(shot), "Screenshots");
        let trip = idx.entries.iter().find(|e| e.name == "t1.webp").unwrap();
        assert_eq!(album_of(trip), "Trip");
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
        let first = idx.entries[0].id.clone();
        assert!(find(&idx, &first).is_some());
        assert!(find(&idx, "deadbeef").is_none());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn content_type_map() {
        assert_eq!(content_type("jpg"), "image/jpeg");
        assert_eq!(content_type("jpeg"), "image/jpeg");
        assert_eq!(content_type("png"), "image/png");
        assert_eq!(content_type("webp"), "image/webp");
        assert_eq!(content_type("gif"), "image/gif");
        assert_eq!(content_type("bmp"), "image/bmp");
        assert_eq!(content_type("tiff"), "application/octet-stream");
    }

    #[test]
    fn date_label_converts_epoch_days() {
        assert_eq!(date_label(0), "1970-01-01");
        // 2026-09-23 00:00:00 UTC
        assert_eq!(date_label(1_790_121_600), "2026-09-23");
        assert_eq!(date_label(-5), "1970-01-01", "negative clamps");
    }

    #[test]
    fn thumb_cache_key_includes_mtime_and_width() {
        assert_eq!(
            thumb_cache_name("abc", 100, 260),
            thumb_cache_name("abc", 100, 260)
        );
        assert_ne!(
            thumb_cache_name("abc", 100, 260),
            thumb_cache_name("abc", 101, 260),
            "mtime bump must miss the cache"
        );
        assert_ne!(
            thumb_cache_name("abc", 100, 260),
            thumb_cache_name("abc", 100, 520),
            "width change must miss the cache"
        );
    }

    #[test]
    fn render_thumbnail_roundtrip_and_cache() {
        // Real decode path: build a 40x20 gradient PNG, render a 16px thumb.
        let base = std::env::temp_dir().join(format!("p043_thumb_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let img = image::RgbImage::from_fn(40, 20, |x, _| {
            image::Rgb([x as u8, 128, 64])
        });
        let src = base.join("grad.png");
        img.save(&src).unwrap();
        let idx = index_directory(&base).unwrap();
        let entry = &idx.entries[0];
        let out = render_thumbnail(&base, entry, 64).unwrap();
        assert!(!out.is_empty());
        assert!(out.len() > 100, "a real JPEG, not an empty file");
        // Cached: second call must hit the disk cache (same bytes).
        let cache = thumb_cache_dir().join(thumb_cache_name(&entry.id, entry.mtime, 64));
        assert!(cache.is_file(), "cache file written");
        let again = render_thumbnail(&base, entry, 64).unwrap();
        assert_eq!(out, again);
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::remove_file(&cache);
    }

    #[test]
    fn missing_root_is_an_error_not_a_panic() {
        let nope = std::env::temp_dir().join("p043_definitely_missing_photo_root");
        let _ = std::fs::remove_dir_all(&nope);
        assert!(index_directory(&nope).is_err());
    }
}
