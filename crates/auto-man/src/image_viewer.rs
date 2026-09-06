//! Deterministic directory/index primitives for Plan 547's image viewer.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageEntry {
    pub id: String,
    pub name: String,
    pub relative_path: String,
    pub extension: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageAsset {
    pub entry: ImageEntry,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewerSnapshot {
    pub root: String,
    pub entries: Vec<ImageEntry>,
    pub selected: usize,
    pub generation: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewRequest {
    pub index: usize,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub quality: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewTicket {
    pub session_id: String,
    pub entry_id: String,
    pub generation: u64,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageViewerApiResponse {
    pub session_id: Option<String>,
    pub snapshot: Option<ViewerSnapshot>,
    pub ticket: Option<ViewTicket>,
    pub stats: ImageViewerStats,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImageViewerStats {
    pub open_sessions: usize,
    pub requests: u64,
    pub settled: u64,
    pub stale_dropped: u64,
}

#[derive(Debug, Clone)]
pub struct ImageViewerSession {
    pub id: String,
    pub snapshot: ViewerSnapshot,
    pub generation: u64,
    pub request_revision: u64,
    pub accepted_revision: u64,
}

impl ImageViewerSession {
    pub fn selected_entry(&self) -> Option<&ImageEntry> {
        self.snapshot.entries.get(self.snapshot.selected)
    }

    pub fn keep_indices(&self) -> Vec<usize> {
        let len = self.snapshot.entries.len();
        let Some(current) = cycle_index(len, self.snapshot.selected, 0) else {
            return Vec::new();
        };
        let mut keep = vec![current];
        for delta in [-1, 1] {
            if let Some(index) = cycle_index(len, current, delta) {
                if !keep.contains(&index) {
                    keep.push(index);
                }
            }
        }
        keep
    }
}

#[derive(Debug, Default)]
pub struct ImageViewerService {
    next_session: u64,
    sessions: HashMap<String, ImageViewerSession>,
    stats: ImageViewerStats,
}

impl ImageViewerService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, root: impl AsRef<Path>) -> io::Result<String> {
        let snapshot = index_directory(root)?;
        self.next_session = self.next_session.saturating_add(1);
        let id = format!("session-{}", self.next_session);
        self.sessions.insert(
            id.clone(),
            ImageViewerSession {
                id: id.clone(),
                generation: snapshot.generation,
                request_revision: 0,
                accepted_revision: 0,
                snapshot,
            },
        );
        self.stats.open_sessions = self.sessions.len();
        Ok(id)
    }

    pub fn session(&self, id: &str) -> Option<&ImageViewerSession> {
        self.sessions.get(id)
    }

    pub fn navigate(&mut self, id: &str, delta: isize) -> Option<&ImageEntry> {
        let session = self.sessions.get_mut(id)?;
        let next = cycle_index(session.snapshot.entries.len(), session.snapshot.selected, delta)?;
        session.snapshot.selected = next;
        session.generation = session.generation.saturating_add(1);
        session.snapshot.generation = session.generation;
        session.selected_entry()
    }

    pub fn request_view(
        &mut self,
        id: &str,
        viewport_width: u32,
        viewport_height: u32,
        quality: u8,
    ) -> Option<ViewTicket> {
        let session = self.sessions.get_mut(id)?;
        let entry = session.selected_entry()?.id.clone();
        session.request_revision = session.request_revision.saturating_add(1);
        self.stats.requests = self.stats.requests.saturating_add(1);
        Some(ViewTicket {
            session_id: id.to_string(),
            entry_id: entry,
            generation: session.generation,
            revision: session.request_revision,
        })
    }

    pub fn accept_settled(&mut self, id: &str, revision: u64, elapsed: Duration) -> bool {
        let Some(session) = self.sessions.get_mut(id) else {
            return false;
        };
        if revision != session.request_revision || elapsed < Duration::from_millis(80) {
            self.stats.stale_dropped = self.stats.stale_dropped.saturating_add(1);
            return false;
        }
        session.accepted_revision = revision;
        self.stats.settled = self.stats.settled.saturating_add(1);
        true
    }

    pub fn close(&mut self, id: &str) -> bool {
        let removed = self.sessions.remove(id).is_some();
        self.stats.open_sessions = self.sessions.len();
        removed
    }

    pub fn stats(&self) -> ImageViewerStats {
        let mut stats = self.stats.clone();
        stats.open_sessions = self.sessions.len();
        stats
    }
}

pub fn canonical_root(root: impl AsRef<Path>) -> io::Result<PathBuf> {
    let canonical = fs::canonicalize(root.as_ref())?;
    if !canonical.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotADirectory, "image viewer root is not a directory"));
    }
    Ok(canonical)
}

pub fn is_supported_format(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NaturalPart {
    Text(String),
    Number(u64),
}

impl Ord for NaturalPart {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.cmp(b),
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            (Self::Number(_), Self::Text(_)) => Ordering::Less,
            (Self::Text(_), Self::Number(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for NaturalPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn natural_sort_key(name: &str) -> Vec<NaturalPart> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut numeric = None;
    for ch in name.chars() {
        let is_digit = ch.is_ascii_digit();
        if numeric != Some(is_digit) && !current.is_empty() {
            parts.push(if numeric == Some(true) {
                NaturalPart::Number(current.parse::<u64>().unwrap_or(u64::MAX))
            } else {
                NaturalPart::Text(current.to_ascii_lowercase())
            });
            current.clear();
        }
        numeric = Some(is_digit);
        current.push(ch);
    }
    if !current.is_empty() {
        parts.push(if numeric == Some(true) {
            NaturalPart::Number(current.parse::<u64>().unwrap_or(u64::MAX))
        } else {
            NaturalPart::Text(current.to_ascii_lowercase())
        });
    }
    parts
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, fs::Metadata)>) -> io::Result<()> {
    for item in fs::read_dir(dir)? {
        let item = item?;
        let path = item.path();
        let metadata = item.metadata()?;
        if metadata.is_dir() {
            collect_files(root, &path, out)?;
        } else if metadata.is_file() && is_supported_format(&path) && path.strip_prefix(root).is_ok() {
            out.push((path, metadata));
        }
    }
    Ok(())
}

pub fn index_directory(root: impl AsRef<Path>) -> io::Result<ViewerSnapshot> {
    let canonical = canonical_root(root)?;
    let mut files = Vec::new();
    collect_files(&canonical, &canonical, &mut files)?;
    files.sort_by(|(a, _), (b, _)| {
        let a_rel = a.strip_prefix(&canonical).unwrap().to_string_lossy();
        let b_rel = b.strip_prefix(&canonical).unwrap().to_string_lossy();
        natural_sort_key(&a_rel).cmp(&natural_sort_key(&b_rel)).then_with(|| a_rel.cmp(&b_rel))
    });
    let entries = files.into_iter().map(|(path, metadata)| {
        let relative = path.strip_prefix(&canonical).unwrap().to_string_lossy().replace('\\', "/");
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        let id = blake3::hash(relative.as_bytes()).to_hex().to_string();
        ImageEntry { id, name, relative_path: relative, extension, bytes: metadata.len() }
    }).collect();
    Ok(ViewerSnapshot { root: redact_path(&canonical, &canonical), entries, selected: 0, generation: 1 })
}

pub fn redact_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| "<outside-root>".to_string())
}

pub fn cycle_index(len: usize, current: usize, delta: isize) -> Option<usize> {
    if len == 0 { None } else { Some((current as isize + delta).rem_euclid(len as isize) as usize) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn image_viewer_directory_index_filters_naturally_and_redacts_paths() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["image10.JPG", "image2.png", "image01.webp", "notes.txt"] {
            let mut file = fs::File::create(dir.path().join(name)).unwrap();
            writeln!(file, "fixture").unwrap();
        }
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("image3.jpeg"), b"nested").unwrap();
        let snapshot = index_directory(dir.path()).unwrap();
        let names: Vec<_> = snapshot.entries.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, ["image01.webp", "image2.png", "image10.JPG", "image3.jpeg"]);
        assert!(snapshot.entries.iter().all(|entry| !entry.relative_path.contains(':')));
        assert!(snapshot.entries.iter().all(|entry| !entry.relative_path.contains('\\')));
        assert_eq!(snapshot.root, "");
        assert_eq!(cycle_index(names.len(), 0, -1), Some(3));
        assert_eq!(cycle_index(names.len(), 3, 1), Some(0));
    }

    #[test]
    fn image_viewer_directory_rejects_non_directories() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("one.png");
        fs::write(&file, b"x").unwrap();
        assert_eq!(canonical_root(&file).unwrap_err().kind(), io::ErrorKind::NotADirectory);
    }

    #[test]
    fn image_viewer_service_is_current_first_and_revision_gated() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["1.png", "2.png", "3.png"] {
            fs::write(dir.path().join(name), b"x").unwrap();
        }
        let mut service = ImageViewerService::new();
        let session = service.open(dir.path()).unwrap();
        assert_eq!(service.session(&session).unwrap().keep_indices(), vec![0, 2, 1]);

        service.navigate(&session, 1).unwrap();
        assert_eq!(service.session(&session).unwrap().keep_indices(), vec![1, 0, 2]);
        let first = service.request_view(&session, 800, 600, 90).unwrap();
        let second = service.request_view(&session, 800, 600, 90).unwrap();
        assert!(!service.accept_settled(&session, first.revision, Duration::from_millis(80)));
        assert!(!service.accept_settled(&session, second.revision, Duration::from_millis(79)));
        assert!(service.accept_settled(&session, second.revision, Duration::from_millis(80)));
        assert_eq!(service.stats().settled, 1);
        assert!(service.close(&session));
        assert_eq!(service.stats().open_sessions, 0);
    }

    #[test]
    fn image_viewer_api_returns_metadata_and_opaque_tickets_only() {
        let response = ImageViewerApiResponse {
            session_id: Some("session-1".to_string()),
            snapshot: Some(ViewerSnapshot {
                root: String::new(),
                entries: vec![ImageEntry {
                    id: "asset-1".to_string(),
                    name: "one.png".to_string(),
                    relative_path: "one.png".to_string(),
                    extension: "png".to_string(),
                    bytes: 12,
                }],
                selected: 0,
                generation: 1,
            }),
            ticket: Some(ViewTicket {
                session_id: "session-1".to_string(),
                entry_id: "asset-1".to_string(),
                generation: 1,
                revision: 1,
            }),
            stats: ImageViewerStats::default(),
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["session_id"], "session-1");
        assert_eq!(json["snapshot"]["entries"][0]["bytes"], 12);
        assert_eq!(json["ticket"]["entry_id"], "asset-1");
        assert!(json.get("pixels").is_none());
        assert!(json.get("data").is_none());
        assert!(json.get("encoded").is_none());
        assert!(json.to_string().find("89504e47").is_none());

        let api_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/src/back/api.at");
        let api = std::fs::read_to_string(api_path).unwrap();
        for endpoint in [
            "open_file", "open_directory", "open_path", "snapshot",
            "navigate", "request_view", "close_session", "image_stats",
        ] {
            assert!(api.contains(&format!("fn {endpoint}")), "missing API endpoint {endpoint}");
        }
        assert!(api.contains("auto.image"), "API must delegate media work to auto.image");
        assert!(!api.contains("pixels"), "API contract must not carry pixel payloads");
    }
}
