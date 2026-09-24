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
//! paths ever leave the backend; tokens resolve through a per-root registry
//! populated by `list_directory` scans, so a request can never address a
//! file outside the scanned set, and `sanitize_rel_dir` keeps user-supplied
//! navigation segments inside the root (no `..`/absolute/drive-letter).
//!
//! Listing is **non-recursive** (user requirement, 2026-09-24): a scan of
//! `dir=X` returns only the images directly inside X plus X's immediate
//! subdirectories (each with a direct-image count for tab badges).
//! Subdirectory contents are fetched by a follow-up scan — the gallery
//! navigates like a file browser, it does not flat-render the whole tree.

use std::cmp::Ordering;
use std::io;
use std::path::{Path, PathBuf};

/// Extensions offered as *candidates* (image crate decoder set). HEIC and
/// RAW formats are deliberately absent — the `image` crate cannot decode
/// them, and listing undecodable files would render broken tiles.
pub const PHOTO_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "bmp"];

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

/// A subdirectory entry returned by a non-recursive listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoDir {
    /// Display name (the directory's file name).
    pub name: String,
    /// Root-relative path with `/` separators — the navigation token the
    /// frontend passes back as `?dir=`.
    pub rel_path: String,
    /// Number of supported images *directly* inside (one level; recursive
    /// counts would contradict the non-recursive browsing model).
    pub image_count: usize,
}

/// Result of one non-recursive directory listing.
#[derive(Debug, Clone, Default)]
pub struct PhotoListing {
    /// Root-relative path of the listed directory ("" = root).
    pub dir_rel: String,
    pub images: Vec<PhotoEntry>,
    pub dirs: Vec<PhotoDir>,
}

/// Validate a user-supplied navigation segment string (`?dir=`). Only
/// `a/b/c`-shaped relative paths are legal: no absolute paths, no drive
/// letters, no `..`, no backslashes, no quotes. Returns the normalized
/// `/`-separated relative path (`""` for root) or None.
pub fn sanitize_rel_dir(rel: &str) -> Option<String> {
    let rel = rel.trim().trim_matches('/');
    if rel.is_empty() {
        return Some(String::new());
    }
    let mut out: Vec<&str> = Vec::new();
    for seg in rel.split('/') {
        if seg.is_empty() || seg == "." {
            continue; // 空段（连续/首尾斜杠）与当前段无害，归一化跳过
        }
        if seg == ".." {
            return None;
        }
        // 拒绝反斜杠（Windows 分隔符的另一形态）、盘符冒号、引号——
        // 只允许干净的相对路径段（'/' 已在上层 split）。
        if seg.contains(['\\', ':', '"', '\'']) {
            return None;
        }
        out.push(seg);
    }
    Some(out.join("/"))
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

/// mtime seconds since epoch from file metadata (0 when unavailable).
fn mtime_secs(meta: &std::fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Count supported images *directly* inside `dir` (non-recursive; one
/// read_dir). Used for subdirectory tab badges.
fn count_direct_images(dir: &Path) -> usize {
    let Ok(rd) = std::fs::read_dir(dir) else { return 0 };
    rd.filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_file()).unwrap_or(false) && is_supported(&e.path())
        })
        .count()
}

/// One-level directory listing (non-recursive browsing model). Returns the
/// images directly inside `rel` plus its immediate subdirectories. Never
/// follows symlinks (media_service precedent). Errors when the directory
/// is unreadable/missing (caller maps to `root_missing` / 404 semantics).
pub fn list_directory(root: &Path, rel: &str) -> io::Result<PhotoListing> {
    let dir_rel = sanitize_rel_dir(rel)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "illegal dir segment"))?;
    let dir = if dir_rel.is_empty() {
        root.to_path_buf()
    } else {
        root.join(dir_rel.replace('/', std::path::MAIN_SEPARATOR_STR))
    };
    let rd = std::fs::read_dir(&dir)?;
    let mut images: Vec<(PathBuf, u64, i64)> = Vec::new();
    let mut dirs: Vec<PhotoDir> = Vec::new();
    for item in rd {
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
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let child_rel = if dir_rel.is_empty() {
                name.clone()
            } else {
                format!("{dir_rel}/{name}")
            };
            dirs.push(PhotoDir {
                name,
                image_count: count_direct_images(&path),
                rel_path: child_rel,
            });
        } else if meta.is_file() && is_supported(&path) {
            images.push((path, meta.len(), mtime_secs(&meta)));
        }
    }
    // 自然序（文件名数字感知）——浏览型图库的确定性排序。
    images.sort_by(|(a, _, _), (b, _, _)| {
        let a_rel = a.strip_prefix(root).unwrap_or(a).to_string_lossy().replace('\\', "/");
        let b_rel = b.strip_prefix(root).unwrap_or(b).to_string_lossy().replace('\\', "/");
        natural_key(&a_rel).cmp(&natural_key(&b_rel)).then_with(|| a_rel.cmp(&b_rel))
    });
    dirs.sort_by(|a, b| natural_key(&a.rel_path).cmp(&natural_key(&b.rel_path)));
    let mut listing = PhotoListing { dir_rel, images: Vec::new(), dirs };
    for (path, bytes, mtime) in images {
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
        listing.images.push(PhotoEntry { id, name, relative_path, rel_dir, extension, bytes, width, height, mtime });
    }
    // token → 相对路径注册表（thumb/full 反查用），按 root 分桶。
    if let Ok(mut reg) = token_registry().lock() {
        reg.insert(root.to_string_lossy().to_string(), listing.images.iter().map(|e| (e.id.clone(), e.relative_path.clone())).collect());
    }
    Ok(listing)
}

/// token → root-relative path registry. Bucketed by root so two photo apps
/// with identical relative paths never collide. Rebuilt wholesale per scan
/// (bounded by the scanned dir's image count).
fn token_registry() -> &'static std::sync::Mutex<std::collections::HashMap<String, std::collections::HashMap<String, String>>> {
    static REG: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, std::collections::HashMap<String, String>>>> = std::sync::OnceLock::new();
    &REG.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// Resolve a photo token against the registry for `root`. Returns the
/// root-relative path (`/` separators) only when the token was issued by a
/// scan of this same root — otherwise None (404 arm).
pub fn resolve_token(root: &Path, token: &str) -> Option<String> {
    let reg = token_registry().lock().ok()?;
    reg.get(&root.to_string_lossy().to_string())?.get(token).cloned()
}

/// Rebuild a `PhotoEntry` for a root-relative path (thumb/full 端点的
/// token 反查臂——只服务扫描过的文件，路径永不来自请求体）。
pub fn photo_from_rel(root: &Path, rel: &str) -> Option<PhotoEntry> {
    let path = rel_to_path(root, rel);
    let meta = std::fs::symlink_metadata(&path).ok()?;
    if !meta.is_file() || !is_supported(&path) {
        return None;
    }
    let name = path.file_name()?.to_string_lossy().to_string();
    let rel_dir = rel.rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_default();
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
    let (width, height) = header_dimensions(&path);
    Some(PhotoEntry {
        id: blake3::hash(rel.as_bytes()).to_hex().to_string(),
        name,
        relative_path: rel.to_string(),
        rel_dir,
        extension,
        bytes: meta.len(),
        width,
        height,
        mtime: mtime_secs(&meta),
    })
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

/// Build an absolute path from a root-relative (`/`-separated) path.
pub fn rel_to_path(root: &Path, relative_path: &str) -> PathBuf {
    let mut p = root.to_path_buf();
    for part in relative_path.split('/') {
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
    let path = rel_to_path(root, &entry.relative_path);
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
    fn root_listing_is_non_recursive_with_dirs() {
        let base = tmp_tree();
        let l = list_directory(&base, "").unwrap();
        // 根目录直属：3 张图（a.png b.PNG c.jpg），txt/noext 被白名单拒。
        let names: Vec<_> = l.images.iter().map(|e| e.name.clone()).collect();
        assert_eq!(names.len(), 3, "{names:?}");
        assert!(!names.iter().any(|n| n == "notes.txt"), "{names:?}");
        // 子目录作为导航条目（非内容平铺）。
        let dnames: Vec<_> = l.dirs.iter().map(|d| d.name.clone()).collect();
        assert!(dnames.iter().any(|n| n == "Screenshots"), "{dnames:?}");
        assert!(dnames.iter().any(|n| n == "Trip"), "{dnames:?}");
        // 直接图片计数（一层）。
        let shots = l.dirs.iter().find(|d| d.name == "Screenshots").unwrap();
        assert_eq!(shots.image_count, 2, "Screenshots 直属 2 张");
        assert_eq!(shots.rel_path, "Screenshots");
        // Header dimensions parsed for real PNGs.
        let a = l.images.iter().find(|e| e.name == "a.png").unwrap();
        assert_eq!((a.width, a.height), (1, 1));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn subdir_listing_navigates_one_level() {
        let base = tmp_tree();
        let l = list_directory(&base, "Screenshots").unwrap();
        assert_eq!(l.dir_rel, "Screenshots");
        let names: Vec<_> = l.images.iter().map(|e| name_only(e)).collect();
        assert!(names.iter().any(|n| n == "s2.png"), "{names:?}");
        assert!(names.iter().any(|n| n == "s10.png"), "{names:?}");
        // 自然序：s2 在 s10 前。
        let s2 = names.iter().position(|n| n == "s2.png").unwrap();
        let s10 = names.iter().position(|n| n == "s10.png").unwrap();
        assert!(s2 < s10, "natural order broken: {names:?}");
        // 该子目录下无更深子目录。
        assert!(l.dirs.is_empty(), "{:?}", l.dirs);
        let _ = std::fs::remove_dir_all(&base);
    }

    fn name_only(e: &PhotoEntry) -> String { e.name.clone() }

    #[test]
    fn sanitize_rel_dir_rejects_traversal() {
        assert_eq!(sanitize_rel_dir(""), Some(String::new()));
        assert_eq!(sanitize_rel_dir("Screenshots"), Some("Screenshots".into()));
        assert_eq!(sanitize_rel_dir("a/b/c"), Some("a/b/c".into()));
        assert_eq!(sanitize_rel_dir("/a//b/"), Some("a/b".into()));
        assert!(sanitize_rel_dir("..").is_none());
        assert!(sanitize_rel_dir("a/../b").is_none());
        assert!(sanitize_rel_dir("C:/Windows").is_none());
        assert!(sanitize_rel_dir("a\\b").is_none());
        assert!(sanitize_rel_dir("a:b").is_none());
    }

    #[test]
    fn tokens_resolve_per_root_and_never_leak_paths() {
        let base = tmp_tree();
        let l = list_directory(&base, "").unwrap();
        for e in &l.images {
            assert_eq!(e.id.len(), 64, "blake3 hex");
            assert!(!e.id.contains(':'), "no drive letters in the token");
            assert!(!e.relative_path.contains(base.to_string_lossy().as_ref()));
        }
        let first = l.images[0].id.clone();
        assert_eq!(resolve_token(&base, &first).as_deref(), Some(l.images[0].relative_path.as_str()));
        assert!(resolve_token(&base, "deadbeef").is_none());
        // 异 root 不串桶。
        let other = std::env::temp_dir().join("p043_other_root");
        let _ = std::fs::create_dir_all(&other);
        assert!(resolve_token(&other, &first).is_none());
        let _ = std::fs::remove_dir_all(&other);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn photo_from_rel_rebuilds_entry() {
        let base = tmp_tree();
        list_directory(&base, "").unwrap();
        let e = photo_from_rel(&base, "a.png").unwrap();
        assert_eq!(e.name, "a.png");
        assert_eq!((e.width, e.height), (1, 1));
        assert!(photo_from_rel(&base, "notes.txt").is_none(), "非图片不重建");
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
        list_directory(&base, "").unwrap();
        let entry = photo_from_rel(&base, "grad.png").unwrap();
        let out = render_thumbnail(&base, &entry, 64).unwrap();
        assert!(!out.is_empty());
        assert!(out.len() > 100, "a real JPEG, not an empty file");
        // Cached: second call must hit the disk cache (same bytes).
        let cache = thumb_cache_dir().join(thumb_cache_name(&entry.id, entry.mtime, 64));
        assert!(cache.is_file(), "cache file written");
        let again = render_thumbnail(&base, &entry, 64).unwrap();
        assert_eq!(out, again);
        let _ = std::fs::remove_dir_all(&base);
        let _ = std::fs::remove_file(&cache);
    }

    #[test]
    fn missing_root_is_an_error_not_a_panic() {
        let nope = std::env::temp_dir().join("p043_definitely_missing_photo_root");
        let _ = std::fs::remove_dir_all(&nope);
        assert!(list_directory(&nope, "").is_err());
        // 非法 dir 段（穿越尝试）同样是 Err 而非 panic。
        let base = tmp_tree();
        assert!(list_directory(&base, "..").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }
}
