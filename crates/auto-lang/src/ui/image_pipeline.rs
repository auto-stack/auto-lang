//! Shared, renderer-neutral image asset pipeline — facade + runtime half.
//!
//! PLAN-811: the data plane (public media types, registry, caches, scheduler
//! primitives, metadata/limits, HTTP rules) moved verbatim into the
//! `auto-media-core` crate; the re-export below preserves the historical
//! in-`ui` path with UNCHANGED type identity — every re-exported name here
//! IS the `auto_media_core::image_pipeline` item. The worker pool /
//! viewer-session runtime migrates in the following task of the same plan
//! and temporarily remains below, consuming the core types through this
//! module exactly as external consumers do.

pub use auto_media_core::image_pipeline::{
    checked_rgba_bytes, decode_rendition, global_media_registry, inspect_image_metadata,
    media_change_generation, media_http_response, media_surface_status, pending_media_count,
    resolve_media_uri, validate_image_limits, MediaAssetError, MediaAssetId, MediaAssetKey,
    MediaAssetRegistry, MediaAssetState, MediaAssetTicket, DecodedPixelCache, EncodedByteLru,
    MediaHttpResponse, MediaLatestWins, MediaLookup, MediaMetadata, MediaOrientation, MediaPin,
    MediaPipelineStats, MediaPriority, MediaPriorityQueue, MediaSurfaceStatus, MediaWork,
    RenditionSpec, RenderedImage, MAX_IMAGE_FILE_BYTES, MAX_IMAGE_PIXELS,
};

// Private helpers that moved with the data plane; unqualified references in
// the runtime half below keep the migrated code verbatim and the historical
// public surface of this module unchanged.
use auto_media_core::image_pipeline::{media_uri, parse_media_path, supported_media_path};

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

#[derive(Clone, Debug)]
enum MediaSource {
    Path(PathBuf),
    Bytes(Arc<[u8]>),
}

#[derive(Clone, Debug)]
struct MediaWorkItem {
    id: u64,
    ticket: MediaAssetTicket,
    source: MediaSource,
    spec: RenditionSpec,
    priority: MediaPriority,
    generation: u64,
    view_revision: u64,
}

#[derive(Debug)]
struct DecodedWork {
    item: MediaWorkItem,
    encoded: Arc<[u8]>,
    metadata: MediaMetadata,
    image: image::DynamicImage,
}

enum ResizeMessage {
    Work(DecodedWork),
    Stop,
}

struct PipelineCaches {
    encoded: EncodedByteLru,
    decoded: DecodedPixelCache,
}

struct WorkerState {
    stopping: bool,
    queue: MediaPriorityQueue,
    jobs: HashMap<u64, MediaWorkItem>,
}

struct WorkerInner {
    state: Mutex<WorkerState>,
    wake: Condvar,
    registry: MediaAssetRegistry,
    resize_tx: SyncSender<ResizeMessage>,
    caches: Mutex<PipelineCaches>,
    latest: Mutex<HashMap<MediaAssetId, MediaLatestWins>>,
}

static NEXT_MEDIA_WORK_ID: AtomicU64 = AtomicU64::new(1);

/// Bounded media executor: two decode/read workers feed one resize/publish
/// lane. Workers sleep on condition variables or channel receives while idle;
/// no UI call performs filesystem I/O or image decoding.
pub struct MediaWorkerPool {
    inner: Arc<WorkerInner>,
    workers: Vec<JoinHandle<()>>,
}

impl MediaWorkerPool {
    pub fn new() -> Self {
        Self::with_registry(MediaAssetRegistry::default())
    }

    pub fn with_registry(registry: MediaAssetRegistry) -> Self {
        let (resize_tx, resize_rx) = sync_channel(8);
        let inner = Arc::new(WorkerInner {
            state: Mutex::new(WorkerState {
                stopping: false,
                queue: MediaPriorityQueue::new(),
                jobs: HashMap::new(),
            }),
            wake: Condvar::new(),
            registry,
            resize_tx,
            caches: Mutex::new(PipelineCaches {
                encoded: EncodedByteLru::default(),
                decoded: DecodedPixelCache::default(),
            }),
            latest: Mutex::new(HashMap::new()),
        });
        let resize_inner = inner.clone();
        let resize_thread = thread::Builder::new()
            .name("media-resize".into())
            .spawn(move || {
                run_resize_worker(resize_inner, resize_rx);
            })
            .expect("spawn media resize worker");
        let mut workers = vec![resize_thread];
        for name in ["media-decode-1", "media-decode-2"] {
            let worker_inner = inner.clone();
            workers.push(
                thread::Builder::new()
                    .name(name.into())
                    .spawn(move || {
                        run_decode_worker(worker_inner);
                    })
                    .expect("spawn media decode worker"),
            );
        }
        Self { inner, workers }
    }

    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    pub fn submit_path(
        &self,
        ticket: MediaAssetTicket,
        path: PathBuf,
        spec: RenditionSpec,
        priority: MediaPriority,
        generation: u64,
        view_revision: u64,
    ) -> bool {
        self.submit(MediaWorkItem {
            id: NEXT_MEDIA_WORK_ID.fetch_add(1, Ordering::Relaxed),
            ticket,
            source: MediaSource::Path(path),
            spec,
            priority,
            generation,
            view_revision,
        })
    }

    pub fn submit_bytes(
        &self,
        ticket: MediaAssetTicket,
        bytes: Arc<[u8]>,
        spec: RenditionSpec,
        priority: MediaPriority,
        generation: u64,
        view_revision: u64,
    ) -> bool {
        self.submit(MediaWorkItem {
            id: NEXT_MEDIA_WORK_ID.fetch_add(1, Ordering::Relaxed),
            ticket,
            source: MediaSource::Bytes(bytes),
            spec,
            priority,
            generation,
            view_revision,
        })
    }

    fn submit(&self, item: MediaWorkItem) -> bool {
        if let Ok(mut latest) = self.inner.latest.lock() {
            if let Some(gate) = latest.get_mut(&item.ticket.id) {
                gate.advance(item.generation, item.view_revision);
            } else {
                latest.insert(
                    item.ticket.id,
                    MediaLatestWins::new(item.generation, item.view_revision),
                );
            }
        }
        let mut state = self.inner.state.lock().expect("media worker lock poisoned");
        if state.stopping || !state.queue.push(MediaWork::new(item.id, item.priority)) {
            return false;
        }
        state.jobs.insert(item.id, item);
        self.inner.wake.notify_one();
        true
    }

    pub fn decoded_pixels(&self, id: MediaAssetId) -> Option<(u32, u32, Arc<[u8]>)> {
        self.inner.caches.lock().ok()?.decoded.get_rendered(id)
    }

    pub fn shutdown(&mut self) {
        {
            let mut state = self.inner.state.lock().expect("media worker lock poisoned");
            state.stopping = true;
            state.queue.clear();
            state.jobs.clear();
            self.inner.wake.notify_all();
        }
        let _ = self.inner.resize_tx.send(ResizeMessage::Stop);
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

fn take_media_work(inner: &Arc<WorkerInner>) -> Option<MediaWorkItem> {
    let mut state = inner.state.lock().expect("media worker lock poisoned");
    loop {
        if state.stopping {
            return None;
        }
        if let Some(work) = state.queue.pop() {
            return state.jobs.remove(&work.id);
        }
        state = inner.wake.wait(state).expect("media worker lock poisoned");
    }
}

fn run_decode_worker(inner: Arc<WorkerInner>) {
    while let Some(item) = take_media_work(&inner) {
        let item_id = item.ticket.id;
        if let Ok(mut latest) = inner.latest.lock() {
            if !latest.get_mut(&item.ticket.id).is_some_and(|gate| {
                gate.accept_decode_complete(item.generation, item.view_revision)
            }) {
                let _ = inner
                    .registry
                    .transition(item.ticket.id, MediaAssetState::Stale);
                continue;
            }
        }
        let result = (|| {
            inner
                .registry
                .transition(item.ticket.id, MediaAssetState::Reading)?;
            let encoded: Arc<[u8]> = match &item.source {
                MediaSource::Path(path) => Arc::from(
                    std::fs::read(path)
                        .map_err(|_| MediaAssetError::Rejected("image read failed".into()))?,
                ),
                MediaSource::Bytes(bytes) => bytes.clone(),
            };
            let metadata = inspect_image_metadata(&encoded)?;
            inner
                .registry
                .update_metadata(item.ticket.id, metadata.clone());
            inner
                .registry
                .transition(item.ticket.id, MediaAssetState::Decoding)?;
            let image = image::load_from_memory(&encoded).map_err(|_| MediaAssetError::Corrupt)?;
            Ok::<_, MediaAssetError>(DecodedWork {
                item,
                encoded,
                metadata,
                image,
            })
        })();
        match result {
            Ok(decoded) => {
                if inner.resize_tx.send(ResizeMessage::Work(decoded)).is_err() {
                    return;
                }
            }
            Err(error) => {
                let _ = inner.registry.fail(item_id, error);
            }
        }
    }
}

fn oriented_image(
    image: image::DynamicImage,
    orientation: MediaOrientation,
) -> image::DynamicImage {
    match orientation {
        MediaOrientation::Normal => image,
        MediaOrientation::FlipHorizontal => image.fliph(),
        MediaOrientation::Rotate180 => image.rotate180(),
        MediaOrientation::FlipVertical => image.flipv(),
        MediaOrientation::Transpose => image.rotate90().fliph(),
        MediaOrientation::Rotate90 => image.rotate90(),
        MediaOrientation::Transverse => image.rotate270().fliph(),
        MediaOrientation::Rotate270 => image.rotate270(),
    }
}

fn render_decoded(
    image: image::DynamicImage,
    orientation: MediaOrientation,
    spec: &RenditionSpec,
) -> RenderedImage {
    let oriented = oriented_image(image, orientation);
    let rendered = if spec.original_pixels || spec.width == 0 || spec.height == 0 {
        oriented
    } else {
        oriented.resize(
            spec.width,
            spec.height,
            image::imageops::FilterType::Lanczos3,
        )
    };
    let rgba = rendered.to_rgba8();
    let (width, height) = rgba.dimensions();
    RenderedImage {
        width,
        height,
        rgba: Arc::from(rgba.into_raw()),
    }
}

fn run_resize_worker(inner: Arc<WorkerInner>, resize_rx: Receiver<ResizeMessage>) {
    while let Ok(message) = resize_rx.recv() {
        let ResizeMessage::Work(work) = message else {
            break;
        };
        if let Ok(mut latest) = inner.latest.lock() {
            if !latest.get_mut(&work.item.ticket.id).is_some_and(|gate| {
                gate.accept_publish(work.item.generation, work.item.view_revision)
            }) {
                let _ = inner
                    .registry
                    .transition(work.item.ticket.id, MediaAssetState::Stale);
                continue;
            }
        }
        if inner
            .registry
            .transition(work.item.ticket.id, MediaAssetState::Transforming)
            .is_err()
        {
            continue;
        }
        let rendered = render_decoded(work.image, work.metadata.orientation, &work.item.spec);
        let pin = match work.item.priority {
            MediaPriority::Current | MediaPriority::SettledCurrent => MediaPin::Hard,
            MediaPriority::Neighbor => MediaPin::Soft,
            MediaPriority::Thumbnail => MediaPin::None,
        };
        if let Ok(mut caches) = inner.caches.lock() {
            caches
                .encoded
                .insert(work.item.ticket.id, work.encoded.clone());
            caches.decoded.insert_rendered(
                work.item.ticket.id,
                rendered.width,
                rendered.height,
                rendered.rgba,
                pin,
            );
        }
        let _ = inner.registry.publish_ready(
            work.item.ticket.id,
            work.item.ticket.revision,
            work.encoded,
        );
    }
}

pub fn global_media_worker_pool() -> &'static MediaWorkerPool {
    static POOL: std::sync::LazyLock<MediaWorkerPool> = std::sync::LazyLock::new(|| {
        MediaWorkerPool::with_registry(global_media_registry().clone())
    });
    &POOL
}

pub fn queue_media_path(path: impl Into<PathBuf>, priority: MediaPriority) -> MediaAssetTicket {
    let path = path.into();
    let key = MediaAssetKey {
        source_fingerprint: path.to_string_lossy().into_owned(),
        orientation: MediaOrientation::Normal,
        rendition: RenditionSpec::original(),
        revision: 1,
    };
    let ticket = global_media_registry().queue(key, MediaMetadata::default());
    if !global_media_worker_pool().submit_path(
        ticket,
        path,
        RenditionSpec::original(),
        priority,
        0,
        1,
    ) {
        let _ = global_media_registry().fail(
            ticket.id,
            MediaAssetError::Rejected("media queue is full".into()),
        );
    }
    ticket
}

/// Starts a directory scan away from VM/UI handler threads. Each discovered
/// supported file is submitted to the bounded media executor; the call itself
/// returns immediately and never exposes source paths to the control plane.
pub fn scan_media_directory(path: impl Into<PathBuf>) {
    let root = path.into();
    thread::Builder::new()
        .name("media-directory-scan".into())
        .spawn(move || {
            let Ok(entries) = std::fs::read_dir(root) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = queue_media_path(path, MediaPriority::Thumbnail);
                }
            }
        })
        .ok();
}

// ---------------------------------------------------------------------------
// Metadata-only viewer sessions
// ---------------------------------------------------------------------------

/// A viewer session is deliberately host-owned.  The public value is only the
/// stable session id; paths and tickets remain in this bounded table and are
/// never serialized into Auto state or media URLs.
const MAX_MEDIA_SESSIONS: usize = 32;
const MAX_SESSION_ENTRIES: usize = 256;

#[derive(Debug)]
struct MediaSessionEntry {
    path: PathBuf,
    ticket: MediaAssetTicket,
    name: String,
}

#[derive(Debug)]
struct MediaSession {
    id: String,
    root: PathBuf,
    entries: Vec<MediaSessionEntry>,
    selected: usize,
    generation: u64,
}

static MEDIA_SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);
static MEDIA_SESSIONS: std::sync::LazyLock<Mutex<VecDeque<MediaSession>>> =
    std::sync::LazyLock::new(|| Mutex::new(VecDeque::new()));

fn append_session_entry(session_id: &str, path: PathBuf) {
    if !supported_media_path(&path) {
        return;
    }
    let ticket = queue_media_path(path.clone(), MediaPriority::Thumbnail);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("image")
        .to_owned();
    let mut sessions = match MEDIA_SESSIONS.lock() {
        Ok(sessions) => sessions,
        Err(_) => {
            global_media_registry().release(ticket.id);
            return;
        }
    };
    let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) else {
        global_media_registry().release(ticket.id);
        return;
    };
    if session.entries.len() >= MAX_SESSION_ENTRIES {
        global_media_registry().release(ticket.id);
        return;
    }
    session
        .entries
        .push(MediaSessionEntry { path, ticket, name });
}

fn populate_media_session(session_id: String, root: PathBuf) {
    // All filesystem work happens on this detached worker, never on a VM/UI
    // handler thread.  Sorting makes directory navigation deterministic.
    let mut paths = Vec::new();
    if root.is_file() {
        paths.push(root);
    } else if root.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&root) {
            paths.extend(
                entries
                    .flatten()
                    .map(|entry| entry.path())
                    .filter(|path| path.is_file()),
            );
            paths.sort();
        }
    }
    for path in paths {
        append_session_entry(&session_id, path);
    }
}

/// Open a file or directory without blocking the caller on filesystem I/O.
/// The returned id is safe to keep in Auto model state.
pub fn open_media_session(root: impl Into<PathBuf>) -> String {
    let root = root.into();
    let id = format!(
        "image-session-{}",
        MEDIA_SESSION_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    if let Ok(mut sessions) = MEDIA_SESSIONS.lock() {
        if sessions.len() >= MAX_MEDIA_SESSIONS {
            if let Some(evicted) = sessions.pop_front() {
                for entry in evicted.entries {
                    global_media_registry().release(entry.ticket.id);
                }
            }
        }
        sessions.push_back(MediaSession {
            id: id.clone(),
            root: root.clone(),
            entries: Vec::new(),
            selected: 0,
            generation: 1,
        });
    }
    let worker_id = id.clone();
    thread::Builder::new()
        .name("media-session-open".into())
        .spawn(move || populate_media_session(worker_id, root))
        .ok();
    id
}

fn session_snapshot_locked(session: &MediaSession) -> String {
    let uri = session
        .entries
        .get(session.selected)
        .map(|entry| media_uri(entry.ticket))
        .unwrap_or_default();
    let name = session
        .entries
        .get(session.selected)
        .map(|entry| entry.name.as_str())
        .unwrap_or("");
    // `id`, `uri`, and `name` are generated/filename values.  Escape the two
    // user-visible strings so this metadata response remains valid JSON.
    let escape = |value: &str| value.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "{{\"session\":\"{}\",\"selected\":{},\"count\":{},\"generation\":{},\"name\":\"{}\",\"uri\":\"{}\"}}",
        escape(&session.id), session.selected, session.entries.len(), session.generation, escape(name), escape(&uri)
    )
}

/// Return a metadata-only snapshot for a session.
pub fn media_session_snapshot(session_id: &str) -> String {
    MEDIA_SESSIONS
        .lock()
        .ok()
        .and_then(|sessions| {
            sessions
                .iter()
                .find(|session| session.id == session_id)
                .map(session_snapshot_locked)
        })
        .unwrap_or_else(|| "{\"error\":\"unknown_session\"}".into())
}

/// Return only the currently selected opaque media URI. This lets a front
/// model bind `ImageSurface.src` without parsing JSON or receiving paths.
pub fn media_session_uri(session_id: &str) -> String {
    MEDIA_SESSIONS
        .lock()
        .ok()
        .and_then(|sessions| {
            sessions
                .iter()
                .find(|session| session.id == session_id)
                .and_then(|session| {
                    session
                        .entries
                        .get(session.selected)
                        .map(|entry| media_uri(entry.ticket))
                })
        })
        .unwrap_or_default()
}

/// Return bounded display names for the current directory session. Names are
/// metadata only; the corresponding media URI is still resolved by index.
pub fn media_session_names(session_id: &str) -> Vec<String> {
    MEDIA_SESSIONS
        .lock()
        .ok()
        .and_then(|sessions| {
            sessions
                .iter()
                .find(|session| session.id == session_id)
                .map(|session| {
                    session
                        .entries
                        .iter()
                        .map(|entry| entry.name.clone())
                        .collect()
                })
        })
        .unwrap_or_default()
}

/// Move the selection while preserving a bounded, host-side session table.
pub fn navigate_media_session(session_id: &str, delta: i32) -> String {
    let Ok(mut sessions) = MEDIA_SESSIONS.lock() else {
        return "{\"error\":\"session_lock\"}".into();
    };
    let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) else {
        return "{\"error\":\"unknown_session\"}".into();
    };
    if !session.entries.is_empty() {
        let len = session.entries.len() as i32;
        session.selected = (session.selected as i32 + delta).rem_euclid(len) as usize;
        session.generation = session.generation.saturating_add(1);
    }
    session_snapshot_locked(session)
}

/// Queue a viewport rendition for the selected asset.  The original ticket is
/// retained until the replacement is published, so a stale request cannot
/// invalidate the currently displayed media.
pub fn request_media_view(
    session_id: &str,
    index: i32,
    width: i32,
    height: i32,
    quality: i32,
) -> String {
    let (path, generation) = {
        let Ok(mut sessions) = MEDIA_SESSIONS.lock() else {
            return "{\"error\":\"session_lock\"}".into();
        };
        let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) else {
            return "{\"error\":\"unknown_session\"}".into();
        };
        if session.entries.is_empty() {
            return session_snapshot_locked(session);
        }
        session.selected = index.clamp(0, session.entries.len() as i32 - 1) as usize;
        session.generation = session.generation.saturating_add(1);
        (
            session.entries[session.selected].path.clone(),
            session.generation,
        )
    };
    let spec = RenditionSpec {
        width: width.max(0) as u32,
        height: height.max(0) as u32,
        rotation_degrees: 0,
        quality: quality.clamp(1, 100) as u8,
        original_pixels: false,
    };
    let ticket = queue_media_rendition(path, spec, MediaPriority::SettledCurrent, generation);
    if let Ok(mut sessions) = MEDIA_SESSIONS.lock() {
        if let Some(session) = sessions.iter_mut().find(|session| session.id == session_id) {
            let old_ticket = session.entries[session.selected].ticket;
            session.entries[session.selected].ticket = ticket;
            // The session owns exactly one reference per entry. Release the
            // superseded ticket after swapping it so repeated view requests
            // cannot grow the registry indefinitely while queued work settles.
            if old_ticket.id != ticket.id {
                global_media_registry().release(old_ticket.id);
            }
            return session_snapshot_locked(session);
        }
    }
    global_media_registry().release(ticket.id);
    "{\"error\":\"unknown_session\"}".into()
}

/// Release all tickets held by a session.  Returns false for an already closed
/// or unknown id, allowing generated APIs to report deterministic close state.
pub fn close_media_session(session_id: &str) -> bool {
    let Some(session) = MEDIA_SESSIONS.lock().ok().and_then(|mut sessions| {
        sessions
            .iter()
            .position(|session| session.id == session_id)
            .and_then(|index| sessions.remove(index))
    }) else {
        return false;
    };
    for entry in session.entries {
        global_media_registry().release(entry.ticket.id);
    }
    true
}

pub fn media_session_stats(_session_id: &str) -> String {
    let stats = global_media_registry().stats();
    format!(
        "{{\"queued\":{},\"running\":{},\"completed\":{},\"dropped\":{},\"encoded_bytes\":{},\"decoded_bytes\":{},\"cache_hits\":{},\"cache_misses\":{}}}",
        stats.queued, stats.running, stats.completed, stats.dropped, stats.encoded_bytes, stats.decoded_bytes, stats.cache_hits, stats.cache_misses
    )
}

fn queue_media_rendition(
    path: PathBuf,
    spec: RenditionSpec,
    priority: MediaPriority,
    revision: u64,
) -> MediaAssetTicket {
    let key = MediaAssetKey {
        source_fingerprint: path.to_string_lossy().into_owned(),
        orientation: MediaOrientation::Normal,
        rendition: spec.clone(),
        revision,
    };
    let ticket = global_media_registry().queue(key, MediaMetadata::default());
    if !global_media_worker_pool().submit_path(ticket, path, spec, priority, revision, revision) {
        let _ = global_media_registry().fail(
            ticket.id,
            MediaAssetError::Rejected("media queue is full".into()),
        );
    }
    ticket
}

/// Queues a small square thumbnail rendition for one image file (PLAN-023
/// `auto.image.thumb` runtime). Returns the opaque media URI for the
/// rendition, or "" when the path is missing, unreadable, or has an
/// unsupported extension — callers render their icon fallback off "".
///
/// The registry queue takes a reference on dedup hit, and file-manager-style
/// callers never hold tickets, so the reference is released immediately: the
/// entry rides its 30s expiry grace (decode finishes in milliseconds) and
/// stays evictable afterwards, while the decoded-pixels LRU keeps the URI
/// rendering until budget pressure evicts it (self-healing on re-navigation).
pub fn queue_media_thumbnail(path: impl Into<PathBuf>, size: i32) -> String {
    let path = path.into();
    if !supported_media_path(&path) {
        return String::new();
    }
    if !path.is_file() {
        return String::new();
    }
    let edge = size.clamp(16, 1024) as u32;
    let spec = RenditionSpec {
        width: edge,
        height: edge,
        rotation_degrees: 0,
        quality: 85,
        original_pixels: false,
    };
    let ticket = queue_media_rendition(path, spec, MediaPriority::Thumbnail, 1);
    global_media_registry().release(ticket.id);
    media_uri(ticket)
}

/// Resolves already-decoded pixels for the native renderer. The lookup never
/// opens a file or decodes encoded bytes; only the resize lane populates this
/// bounded cache.
pub fn resolve_media_pixels(uri: &str) -> Option<(u32, u32, Arc<[u8]>)> {
    if !uri.starts_with("/api/__auto/media/") {
        return None;
    }
    let (id, _) = parse_media_path(uri)?;
    global_media_worker_pool().decoded_pixels(id)
}

/// Resolve a media URI for rendering, also returning the asset id (the
/// renderer keys its texture-Handle cache on it — see renderer.rs).
pub fn resolve_media_render(uri: &str) -> Option<(MediaAssetId, u32, u32, Arc<[u8]>)> {
    if !uri.starts_with("/api/__auto/media/") {
        return None;
    }
    let (id, _) = parse_media_path(uri)?;
    global_media_worker_pool()
        .decoded_pixels(id)
        .map(|(w, h, px)| (id, w, h, px))
}

impl Drop for MediaWorkerPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        MediaAssetKey, MediaAssetRegistry, MediaAssetState, MediaMetadata, MediaOrientation,
        MediaPriority, RenditionSpec,
    };

    #[test]
    fn worker_shutdown_joins_two_decode_workers_and_resize_lane() {
        let mut workers = super::MediaWorkerPool::new();
        assert_eq!(workers.worker_count(), 3);
        workers.shutdown();
        assert_eq!(workers.worker_count(), 0);
    }

    #[test]
    fn worker_pipeline_reads_decodes_resizes_and_publishes() {
        let registry = MediaAssetRegistry::new(Duration::from_secs(1));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::new(2, 3))
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let key = MediaAssetKey {
            source_fingerprint: "worker-fixture".into(),
            orientation: MediaOrientation::Normal,
            rendition: RenditionSpec::viewport(1, 1),
            revision: 1,
        };
        let ticket = registry.queue(key, MediaMetadata::default());
        let mut workers = super::MediaWorkerPool::with_registry(registry.clone());
        assert!(workers.submit_bytes(
            ticket,
            std::sync::Arc::from(bytes.into_inner()),
            RenditionSpec::viewport(1, 1),
            MediaPriority::Current,
            1,
            1
        ));
        assert_eq!(
            registry.wait_for_terminal(ticket.id, Duration::from_secs(2)),
            Some(MediaAssetState::Ready)
        );
        assert!(matches!(
            registry.lookup(ticket.id, ticket.revision),
            super::MediaLookup::Ready(_)
        ));
        workers.shutdown();
    }

    #[test]
    fn viewer_session_opens_directory_and_exposes_only_metadata() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures");
        let session = super::open_media_session(root);
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while super::media_session_names(&session).is_empty()
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        let names = super::media_session_names(&session);
        assert!(names.iter().any(|name| name == "rgb-1x1.png"));
        let snapshot = super::media_session_snapshot(&session);
        assert!(snapshot.contains("\"count\":"));
        assert!(
            !snapshot.contains("fixtures"),
            "snapshot must not leak source paths"
        );
        let _ = super::request_media_view(&session, 0, 32, 32, 90);
        let uri = super::media_session_uri(&session);
        assert!(uri.is_empty() || uri.starts_with("/api/__auto/media/"));
        assert!(super::close_media_session(&session));
        assert!(!super::close_media_session(&session));
    }

    #[test]
    fn media_change_generation_advances_on_queue_and_terminal() {
        use super::{media_change_generation, queue_media_path, MediaPriority};
        let before = media_change_generation();
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures/rgb-1x1.png");
        let ticket = queue_media_path(&fixture, MediaPriority::Current);
        assert_ne!(
            media_change_generation(),
            before,
            "queue must bump generation"
        );
        let mid = media_change_generation();
        let _ = super::global_media_registry().fail(ticket.id, super::MediaAssetError::Corrupt);
        assert_ne!(
            media_change_generation(),
            mid,
            "terminal failure must bump generation"
        );
    }

    #[test]
    fn thumbnail_queue_rejects_unsupported_and_queues_supported_paths() {
        // Missing path (supported ext) and unsupported ext both degrade to ""
        // so callers can fall back to their icon branch.
        assert_eq!(super::queue_media_thumbnail("not-a-file.png", 64), "");
        assert_eq!(super::queue_media_thumbnail("fixture.txt", 64), "");

        // Supported fixture queues a Thumbnail-priority rendition and returns
        // an opaque media URI that never leaks the source path.
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/ui/031-image-viewer/tests/fixtures/rgb-1x1.png");
        let uri = super::queue_media_thumbnail(&fixture, 64);
        assert!(uri.starts_with("/api/__auto/media/"));
        assert!(!uri.contains("fixtures"));
        let route = uri.strip_prefix("/api/__auto/media/").unwrap();
        let (id, revision) = route.split_once('/').unwrap();
        assert_eq!(id.len(), 32);
        assert_eq!(revision, "1");
    }
}
