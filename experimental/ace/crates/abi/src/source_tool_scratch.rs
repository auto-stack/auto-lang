//! Private certification scratch: a flat tool directory and its reference
//! child, owned under the original store's logs namespace. Persistent caller
//! outputs are never passed here. OS locks, not saved PIDs, decide liveness.
use ace_artifacts::store::{FileGuard, ManagedStore};
use ace_source_query::resources::{Charge, Reservation, Resources};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
};
const MARKER: &str = "source938-tools.owner";
static NEXT: AtomicU64 = AtomicU64::new(1);

enum Kind {
    Sweep,
    Run,
    Provider(u8),
}
fn owned_name(name: &str) -> bool {
    if matches!(name, "provider-0" | "provider-1") {
        return true;
    }
    let Some(numbers) = name.strip_prefix("run-") else {
        return false;
    };
    let mut parts = numbers.split('-');
    for _ in 0..3 {
        if !parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
        {
            return false;
        }
    }
    parts.next().is_none()
}

fn tool_path(path: &Path) -> Result<PathBuf, String> {
    let text = path.to_str().ok_or("non-UTF8 tool scratch path")?;
    if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        return Ok(PathBuf::from(format!(r"\\{unc}")));
    }
    if let Some(dos) = text.strip_prefix(r"\\?\") {
        if dos.as_bytes().get(1) != Some(&b':') {
            return Err("unsupported tool scratch device path".into());
        }
        return Ok(PathBuf::from(dos));
    }
    Ok(path.to_path_buf())
}

struct Pin {
    _file: File,
}
impl Pin {
    fn open(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .share_mode(3)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|e| format!("tool directory {}: {e}", path.display()))?;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(format!("tool directory boundary: {}", path.display()));
        }
        Ok(Self { _file: file })
    }
}
fn marker(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path);
    let mut file = match file {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file()
        || metadata.len() > 512
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(format!(
            "tool ownership marker boundary: {}",
            path.display()
        ));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    Ok(Some(bytes))
}
fn initialize(path: &Path, expected: &[u8]) -> Result<(), String> {
    let existing = marker(&path.join(MARKER))?;
    if existing.as_deref() == Some(expected) {
        return Ok(());
    }
    if existing
        .as_ref()
        .is_some_and(|bytes| !expected.starts_with(bytes))
    {
        return Err(format!(
            "unrecognized tool namespace owner: {}",
            path.display()
        ));
    }
    // Repair only empty initialization or the exact expected marker prefix.
    // All normal files are created after the marker is complete and durable.
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_name() != MARKER {
            return Err(format!(
                "unrecognized tool namespace contents: {}",
                path.display()
            ));
        }
    }
    let mut options = OpenOptions::new();
    options
        .write(true)
        .share_mode(1)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    if existing.is_some() {
        options.truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(path.join(MARKER)).map_err(|e| e.to_string())?;
    file.write_all(expected)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())
}
fn flat_files(path: &Path) -> Result<(), String> {
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(format!(
                "unrecognized tool output: {}",
                entry.path().display()
            ));
        }
    }
    Ok(())
}
fn remove_flat_files(path: &Path, keep_marker: bool) -> Result<(), String> {
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if keep_marker && entry.file_name() == MARKER {
            continue;
        }
        let metadata = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(format!(
                "tool output changed during cleanup: {}",
                entry.path().display()
            ));
        }
        std::fs::remove_file(entry.path())
            .map_err(|e| format!("tool cleanup {}: {e}", entry.path().display()))?;
    }
    Ok(())
}
fn remove_owned(path: &Path, expected: &[u8], held: Option<Pin>) -> Result<(), String> {
    let pin = match held.map(Ok).unwrap_or_else(|| Pin::open(path)) {
        Ok(pin) => pin,
        Err(error) => match std::fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            _ => return Err(error),
        },
    };
    initialize(path, expected)?;
    let reference = path.join("reference");
    let mut reference_pin = None;
    // Validate the complete, finite directory shape before deleting any file.
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(format!(
                "tool output reparse point: {}",
                entry.path().display()
            ));
        }
        if entry.file_name() == "reference" {
            if !metadata.is_dir() {
                return Err(format!(
                    "unrecognized reference directory kind: {}",
                    entry.path().display()
                ));
            }
            reference_pin = Some(Pin::open(&reference)?);
            flat_files(&reference)?;
        } else if !metadata.is_file() {
            return Err(format!(
                "unrecognized tool directory: {}",
                entry.path().display()
            ));
        }
    }
    if reference_pin.is_some() {
        remove_flat_files(&reference, false)?;
        drop(reference_pin);
        std::fs::remove_dir(&reference).map_err(|e| e.to_string())?;
    }
    remove_flat_files(path, true)?;
    std::fs::remove_file(path.join(MARKER)).map_err(|e| e.to_string())?;
    drop(pin);
    std::fs::remove_dir(path).map_err(|e| e.to_string())
}

pub(crate) struct ToolScratch {
    path: PathBuf,
    owner: Vec<u8>,
    _guard: FileGuard,
    _pins: [Pin; 3],
    child_pin: Option<Pin>,
    _index: Reservation,
    _io: Reservation,
    finished: bool,
}
impl ToolScratch {
    pub fn create(store: &ManagedStore, resources: &Resources) -> Result<Self, String> {
        Self::open(store, resources, Kind::Run)?.ok_or_else(|| "missing run scratch".into())
    }
    pub fn provider(store: &ManagedStore, resources: &Resources, bias: u8) -> Result<Self, String> {
        if bias > 1 {
            return Err("unsupported reviewed provider variant".into());
        }
        Self::open(store, resources, Kind::Provider(bias))?
            .ok_or_else(|| "missing provider scratch".into())
    }
    pub fn sweep(store: &ManagedStore, resources: &Resources) -> Result<(), String> {
        Self::open(store, resources, Kind::Sweep).map(|_| ())
    }
    fn open(
        store: &ManagedStore,
        resources: &Resources,
        kind: Kind,
    ) -> Result<Option<Self>, String> {
        if store.store_id().len() > 256 {
            return Err("tool store identity exceeds ownership marker budget".into());
        }
        // Includes cleanup scratch. Cleanup never needs a new quota grant on
        // an error exit; the two pinned directory shapes are fixed above.
        let bytes = store
            .canonical_root()
            .as_os_str()
            .len()
            .checked_mul(32)
            .and_then(|n| n.checked_add(16 << 10))
            .ok_or("tool path budget overflow")?;
        let index = resources
            .reserve(Charge::index(bytes))
            .map_err(|e| e.to_string())?;
        let io = resources
            .reserve(Charge::io(1024))
            .map_err(|e| e.to_string())?;
        let turn = store.staging_turn().map_err(|e| e.to_string())?;
        let root_pin = Pin::open(store.canonical_root())?;
        let logs = store.canonical_root().join("logs");
        let logs_pin = Pin::open(&logs)?;
        let namespace = logs.join("source938-tools");
        let _namespace = FileGuard::lock_timeout(
            &store.root.join("locks/source938-tools-namespace.lock"),
            true,
            Duration::from_secs(8),
        )
        .map_err(|e| e.to_string())?
        .ok_or("tool namespace busy")?;
        match std::fs::create_dir(&namespace) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.to_string()),
        }
        let namespace_pin = Pin::open(&namespace)?;
        let namespace_owner = format!("abi.source-tool-scratch/1\nstore={}\n", store.store_id());
        initialize(&namespace, namespace_owner.as_bytes())?;
        for entry in std::fs::read_dir(&namespace).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_name() == MARKER {
                continue;
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "non-UTF8 tool scratch name")?;
            if name.len() > 96 || !owned_name(&name) {
                return Err(format!("unrecognized tool scratch entry: {name}"));
            }
            let lock = store.root.join("locks").join(format!(
                "source938-tool-{}.lock",
                blake3::hash(name.as_bytes()).to_hex()
            ));
            if let Some(_dead) = FileGuard::try_lock(&lock, true).map_err(|e| e.to_string())? {
                let owner = format!("{namespace_owner}child={name}\n");
                remove_owned(&entry.path(), owner.as_bytes(), None)?;
            }
        }
        let name = match kind {
            Kind::Sweep => return Ok(None),
            // Recreate the same source/include paths before every cache probe.
            // All input bytes/listings are still independently checked by the
            // original C cache; cleanup never changes a compile's identity.
            Kind::Provider(bias) => format!("provider-{bias}"),
            Kind::Run => {
                let stamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_nanos();
                format!(
                    "run-{}-{stamp}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                )
            }
        };
        let lock = store.root.join("locks").join(format!(
            "source938-tool-{}.lock",
            blake3::hash(name.as_bytes()).to_hex()
        ));
        let guard = FileGuard::try_lock(&lock, true)
            .map_err(|e| e.to_string())?
            .ok_or("new tool scratch lock busy")?;
        // Tool argument parsers do not all accept the NT extended prefix.
        // Pins/owner checks still identify the same canonical directory.
        let path = tool_path(&namespace.join(&name))?;
        std::fs::create_dir(&path).map_err(|e| e.to_string())?;
        let child_pin = Pin::open(&path)?;
        let owner = format!("{namespace_owner}child={name}\n").into_bytes();
        initialize(&path, &owner)?;
        drop(turn);
        Ok(Some(Self {
            path,
            owner,
            _guard: guard,
            _pins: [root_pin, logs_pin, namespace_pin],
            child_pin: Some(child_pin),
            _index: index,
            _io: io,
            finished: false,
        }))
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn finish(mut self) -> Result<(), String> {
        self.finished = true;
        remove_owned(&self.path, &self.owner, self.child_pin.take())
    }
}
impl Drop for ToolScratch {
    fn drop(&mut self) {
        if !self.finished {
            let _ = remove_owned(&self.path, &self.owner, self.child_pin.take());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(tag: &str) -> ManagedStore {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("tool-scratch-{tag}-{}-{stamp}", std::process::id()));
        ManagedStore::open(&root).unwrap()
    }
    #[test]
    fn provider_paths_are_stable_exclusive_and_cleaned_after_output_lock_failure() {
        let store = store("provider");
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let first = ToolScratch::provider(&store, &resources, 0).unwrap();
        let path = first.path().to_path_buf();
        let source = path.join("provider.c");
        let object = path.join("provider.obj");
        std::fs::write(&source, b"provider source").unwrap();
        std::fs::write(&object, b"provider object").unwrap();
        ToolScratch::sweep(&store, &resources).unwrap();
        assert!(ToolScratch::provider(&store, &resources, 0).is_err());
        let run = ToolScratch::create(&store, &resources).unwrap();
        assert_eq!(std::fs::read(&source).unwrap(), b"provider source");
        run.finish().unwrap();
        let held = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&object)
            .unwrap();
        assert!(first.finish().unwrap_err().contains("provider.obj"));
        assert!(path.join(MARKER).is_file());
        assert!(ToolScratch::sweep(&store, &resources).is_err());
        drop(held);
        ToolScratch::sweep(&store, &resources).unwrap();
        assert!(!path.exists());
        let second = ToolScratch::provider(&store, &resources, 0).unwrap();
        assert_eq!(second.path(), path);
        second.finish().unwrap();
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn owned_tools_preserve_active_scopes_and_cleanup_even_when_shared_pools_are_full() {
        let store = store("lifetime");
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let first = ToolScratch::create(&store, &resources).unwrap();
        let first_path = first.path().to_path_buf();
        std::fs::write(first.path().join("caller.obj"), b"old reader").unwrap();
        let second = ToolScratch::create(&store, &resources).unwrap();
        assert!(
            first_path.join("caller.obj").exists(),
            "active scratch was swept"
        );
        second.finish().unwrap();
        let reference = first.path().join("reference");
        std::fs::create_dir(&reference).unwrap();
        std::fs::write(reference.join("reference.exe"), b"reference").unwrap();
        let current = resources.stats().current;
        let index_pressure = resources
            .reserve(Charge::index((8 << 20) - current.index))
            .unwrap();
        let io_pressure = resources
            .reserve(Charge::io((8 << 20) - current.io))
            .unwrap();
        first.finish().unwrap();
        assert!(!first_path.exists());
        drop(index_pressure);
        drop(io_pressure);
        assert_eq!(resources.stats().current, baseline);
        let panic_path = std::cell::RefCell::new(PathBuf::new());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let scratch = ToolScratch::create(&store, &resources).unwrap();
            *panic_path.borrow_mut() = scratch.path().to_path_buf();
            std::fs::write(scratch.path().join("partial.obj"), b"partial").unwrap();
            panic!("producer unwound");
        }))
        .is_err());
        assert!(!panic_path.into_inner().exists());
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn tool_cleanup_failure_stays_owned_and_retries_after_the_real_file_lock_is_released() {
        let store = store("locked");
        let resources = Resources::default();
        let scratch = ToolScratch::create(&store, &resources).unwrap();
        let path = scratch.path().to_path_buf();
        let output = path.join("held.obj");
        std::fs::write(&output, b"still held").unwrap();
        let lock = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&output)
            .unwrap();
        let error = scratch.finish().unwrap_err();
        assert!(error.contains("held.obj"), "{error}");
        assert!(path.join(MARKER).exists());
        assert!(ToolScratch::create(&store, &resources).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), b"still held");
        drop(lock);
        let next = ToolScratch::create(&store, &resources).unwrap();
        assert!(!path.exists());
        next.finish().unwrap();
    }

    #[test]
    fn tool_cleanup_rejects_unknown_directories_and_junctions_before_deleting_any_input() {
        let store = store("boundaries");
        let resources = Resources::default();
        let scratch = ToolScratch::create(&store, &resources).unwrap();
        let path = scratch.path().to_path_buf();
        std::fs::write(
            path.join("caller.obj"),
            b"preserve until complete validation",
        )
        .unwrap();
        std::fs::create_dir(path.join("unknown")).unwrap();
        assert!(scratch.finish().unwrap_err().contains("unrecognized"));
        assert!(path.join(MARKER).exists() && path.join("caller.obj").exists());
        std::fs::remove_dir(path.join("unknown")).unwrap();
        let scratch = ToolScratch::create(&store, &resources).unwrap();
        assert!(!path.exists());
        let path = scratch.path().to_path_buf();
        let outside = store.root.join("user-deliverable");
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("sentinel.txt"), b"USER").unwrap();
        std::fs::write(path.join("caller.obj"), b"preserve").unwrap();
        let junction = path.join("reference");
        let ordinary = |p: &Path| {
            p.to_str()
                .unwrap()
                .trim_start_matches(r"\\?\")
                .replace('/', "\\")
        };
        let mut command = std::process::Command::new("cmd.exe");
        command.args([
            "/c",
            "mklink",
            "/J",
            &ordinary(&junction),
            &ordinary(&outside),
        ]);
        let output = ace_core::link::run_deadlined_typed(
            &mut command,
            Duration::from_secs(10),
            ace_core::link::CaptureCaps {
                stdout_max: 4096,
                stderr_max: 4096,
            },
            Vec::new(),
        )
        .unwrap();
        assert_eq!(
            output.exit_code,
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(scratch.finish().unwrap_err().contains("reparse"));
        assert!(path.join(MARKER).exists() && path.join("caller.obj").exists());
        assert_eq!(
            std::fs::read(outside.join("sentinel.txt")).unwrap(),
            b"USER"
        );
        // Remove only the junction entry, never its target or target files.
        std::fs::remove_dir(&junction).unwrap();
        ToolScratch::create(&store, &resources)
            .unwrap()
            .finish()
            .unwrap();
        assert!(!path.exists());
        assert_eq!(
            std::fs::read(outside.join("sentinel.txt")).unwrap(),
            b"USER"
        );
    }

    #[test]
    fn tool_scratch_recovers_a_real_process_exit_before_any_successful_request() {
        const KEY: &str = "SOURCE938_TOOL_SCRATCH_CRASH_ROOT";
        if let Some(root) = std::env::var_os(KEY) {
            let store = ManagedStore::open_existing(Path::new(&root)).unwrap();
            let scratch = if std::env::var("SOURCE938_CRASH_PROVIDER").as_deref() == Ok("1") {
                ToolScratch::provider(&store, &Resources::default(), 0).unwrap()
            } else {
                ToolScratch::create(&store, &Resources::default()).unwrap()
            };
            std::fs::write(scratch.path().join("partial.exe"), b"child output").unwrap();
            std::fs::write(
                store.root.join("crashed-path.txt"),
                scratch.path().to_str().unwrap(),
            )
            .unwrap();
            std::process::exit(73);
        }
        for provider in [false, true] {
            let store = store("crashed");
            std::fs::write(store.root.join("delivered.exe"), b"USER").unwrap();
            let mut command = std::process::Command::new(std::env::current_exe().unwrap());
            command.args(["--exact", "source_tool_scratch::tests::tool_scratch_recovers_a_real_process_exit_before_any_successful_request", "--nocapture"])
            .env(KEY, &store.root)
            .env("SOURCE938_CRASH_PROVIDER", if provider { "1" } else { "0" });
            let output = ace_core::link::run_deadlined_typed(
                &mut command,
                Duration::from_secs(30),
                ace_core::link::CaptureCaps {
                    stdout_max: 4096,
                    stderr_max: 4096,
                },
                Vec::new(),
            )
            .unwrap();
            assert_eq!(
                output.exit_code,
                Some(73),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let old = PathBuf::from(
                std::fs::read_to_string(store.root.join("crashed-path.txt")).unwrap(),
            );
            assert!(old.join("partial.exe").exists());
            ToolScratch::sweep(&store, &Resources::default()).unwrap();
            assert!(!old.exists());
            assert_eq!(
                std::fs::read(store.root.join("delivered.exe")).unwrap(),
                b"USER"
            );
        }
    }
}
