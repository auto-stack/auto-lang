//! Selected-target readiness and the only durable current-pointer switch.
//! No current source, declarations or body artifacts change during linking,
//! execution, persistence, cancellation or failed pointer replacement.
use super::*;
use ace_foreign_contract::manual::HostAuthority;
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::atomic::AtomicBool,
};

/// Supplied afresh by the host, independent of serialized candidate pages.
#[derive(Clone, Debug)]
pub struct CurrentAuthority {
    pub foreign: HostAuthority,
    pub runtime_digest: String,
    pub emitter: String,
}

pub enum Edit {
    Current,
    Body(Candidate),
    Foreign(ForeignCandidate),
}

pub struct PreparedPublication {
    edit: Edit,
    snapshot: Snapshot,
    environment: AdmittedForeign,
    runtime_digest: String,
    target: Target,
    exe: PathBuf,
    exe_digest: String,
    owner: u64,
    base: u64,
}
fn cancelled(cancel: Option<&AtomicBool>) -> Result<(), String> {
    if cancel.is_some_and(|c| c.load(Ordering::Acquire)) {
        Err("foreign.publication.cancelled".into())
    } else {
        Ok(())
    }
}
fn qualify(
    environment: &AdmittedForeign,
    runtime_digest: &str,
    target: Target,
    current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
) -> Result<(), String> {
    let authority = current()?;
    environment.requalify(&authority.foreign)?;
    if authority.runtime_digest != runtime_digest || authority.emitter != target.identity() {
        return Err("foreign.authority.runtime-or-emitter".into());
    }
    Ok(())
}
impl PreparedPublication {
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub fn executable(&self) -> &Path {
        &self.exe
    }
    fn verify_executable(&self) -> Result<File, String> {
        let (digest, file) = executable_identity(&self.exe)?;
        if digest != self.exe_digest {
            return Err("foreign.ready.executable-changed".into());
        }
        Ok(file)
    }
    /// Requalify again immediately before executing the already-linked target.
    /// A revoked capability/runtime/binding never reaches the provider.
    pub fn run(
        &self,
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
        cancel: Option<&AtomicBool>,
    ) -> Result<ace_native::link_driver::PeRun, String> {
        cancelled(cancel)?;
        qualify(
            &self.environment,
            &self.runtime_digest,
            self.target,
            current,
        )?;
        let _executable = self.verify_executable()?;
        cancelled(cancel)?;
        ace_native::link_driver::run_pe(
            &self.exe,
            crate::bridge::BRIDGE_RUN_DEADLINE,
            ace_core::link::CaptureCaps {
                stdout_max: 1 << 20,
                stderr_max: 1 << 20,
            },
        )
        .map_err(|e| e.to_string())
    }
}
impl Session {
    fn validate_edit(&self, edit: &Edit) -> Result<(), String> {
        match edit {
            Edit::Current => Ok(()),
            Edit::Body(body) => self.validate(body),
            Edit::Foreign(foreign) => self.validate_foreign(foreign),
        }
    }
    pub fn prepare_publication(
        &mut self,
        edit: Edit,
        directory: &Path,
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
        cancel: Option<&AtomicBool>,
    ) -> Result<PreparedPublication, String> {
        cancelled(cancel)?;
        self.validate_edit(&edit)?;
        let (front, environment, provider, provider_id, overrides) = match &edit {
            Edit::Current => (
                self.front.checkpoint(),
                self.front.foreign(),
                &self.provider,
                self.provider_page_id.clone(),
                BTreeMap::new(),
            ),
            Edit::Body(body) => (
                self.front.checkpoint_candidate(&body.source),
                self.front.foreign(),
                &self.provider,
                self.provider_page_id.clone(),
                [(body.source.owner().to_string(), body.artifact.clone())]
                    .into_iter()
                    .collect(),
            ),
            Edit::Foreign(foreign) => (
                self.front.checkpoint_foreign(&foreign.source),
                foreign.source.foreign(),
                &foreign.provider,
                hash(&foreign.provider),
                foreign.artifacts.clone(),
            ),
        };
        qualify(environment, &self.runtime_identity, self.target, current)?;
        cancelled(cancel)?;
        let snapshot = self.snapshot_view(
            front.map_err(|e| e.to_string())?,
            environment,
            provider,
            &provider_id,
            &overrides,
        )?;
        let artifacts = self
            .artifacts
            .iter()
            .map(|(name, artifact)| {
                (
                    name.clone(),
                    overrides.get(name).unwrap_or(artifact).as_ref(),
                )
            })
            .collect();
        let source = match &edit {
            Edit::Body(body) => body.source.source(),
            Edit::Foreign(foreign) => foreign.source.source().unwrap_or(self.front.source()),
            _ => self.front.source(),
        };
        let exe = link_artifacts(
            directory,
            self.target,
            source,
            environment,
            provider,
            &self.runtime,
            &artifacts,
            &mut self.meter,
        )?;
        cancelled(cancel)?;
        qualify(environment, &self.runtime_identity, self.target, current)?;
        cancelled(cancel)?;
        let (exe_digest, _executable) = executable_identity(&exe)?;
        let environment = environment.clone();
        Ok(PreparedPublication {
            edit,
            snapshot,
            environment,
            runtime_digest: self.runtime_identity.clone(),
            target: self.target,
            exe,
            exe_digest,
            owner: self.id,
            base: self.generation,
        })
    }
    pub fn restore_authorized(
        root: &[u8],
        expected: &str,
        target: Target,
        capabilities: &[String],
        environment: AdmittedForeign,
        read: impl FnMut(&str) -> Option<Arc<Vec<u8>>>,
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
    ) -> Result<Self, String> {
        let restored = Self::restore(root, expected, target, capabilities, environment, read)?;
        qualify(
            restored.front.foreign(),
            &restored.runtime_identity,
            target,
            current,
        )?;
        Ok(restored)
    }
    pub fn restore_stored_authorized(
        store: &ManagedStore,
        stored: &Stored,
        target: Target,
        capabilities: &[String],
        environment: AdmittedForeign,
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
    ) -> Result<Self, String> {
        let restored = Self::restore_stored(store, stored, target, capabilities, environment)?;
        qualify(
            restored.front.foreign(),
            &restored.runtime_identity,
            target,
            current,
        )?;
        Ok(restored)
    }
    pub fn run_authorized(
        &mut self,
        directory: &Path,
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
        cancel: Option<&AtomicBool>,
    ) -> Result<ace_native::link_driver::PeRun, String> {
        self.prepare_publication(Edit::Current, directory, current, cancel)?
            .run(current, cancel)
    }
    /// Persist required pages first, then exclusively compare and atomically
    /// replace the host's accepted pointer. Only then install the same candidate
    /// in memory. The exclusive session borrow makes final core checks stable.
    pub fn publish_prepared(
        &mut self,
        prepared: PreparedPublication,
        store: &ManagedStore,
        pointer: &Path,
        expected_pointer: Option<&str>,
        previous: Option<&Stored>,
        encode_pointer: impl FnOnce(&Stored) -> Result<Vec<u8>, String>,
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
        cancel: Option<&AtomicBool>,
    ) -> Result<Stored, String> {
        if prepared.owner != self.id || prepared.base != self.generation {
            return Err("foreign.publication.stale".into());
        }
        self.validate_edit(&prepared.edit)?;
        let _executable = prepared.verify_executable()?;
        cancelled(cancel)?;
        qualify(
            &prepared.environment,
            &prepared.runtime_digest,
            self.target,
            current,
        )?;
        let _lock = pointer_lock(pointer)?;
        let observed = read_pointer_identity(pointer)?;
        if observed.as_deref() != expected_pointer {
            return Err("foreign.publication.late-pointer".into());
        }
        let stored = prepared
            .snapshot
            .publish(store, "source938-current", previous)?;
        let _lease = store
            .acquire_lease("source938-publishing", 120, Some(&stored.artifact_id))
            .map_err(|e| e.to_string())?;
        let bytes = encode_pointer(&stored)?;
        if bytes.len() > MAX_ROOT {
            return Err("foreign.resource.pointer".into());
        }
        static SERIAL: AtomicU64 = AtomicU64::new(1);
        let temp = pointer.with_extension(format!(
            "tmp-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        struct Temp(PathBuf);
        impl Drop for Temp {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }
        let temp = Temp(temp);
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp.0)
            .map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        // Last gates are after every fallible producer and before the sole
        // current-pointer switch. No source/HIR/target is installed on failure.
        cancelled(cancel)?;
        qualify(
            &prepared.environment,
            &prepared.runtime_digest,
            self.target,
            current,
        )?;
        if read_pointer_identity(pointer)? != observed {
            return Err("foreign.publication.late-pointer".into());
        }
        cancelled(cancel)?;
        atomic_replace(&temp.0, pointer)?;
        match prepared.edit {
            Edit::Current => {}
            Edit::Body(body) => self.commit(body)?,
            Edit::Foreign(foreign) => self.commit_foreign(foreign)?,
        }
        Ok(stored)
    }
}

/// The identity and decoded host record must come from these SAME bytes.
/// Reading a record and then separately hashing a newer pointer admits a lost
/// update; callers retain this read's digest as their publication base.
pub fn read_pointer_bytes(pointer: &Path) -> Result<Option<Vec<u8>>, String> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0x1 | 0x4);
    }
    let mut file = match options.open(pointer) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_ROOT as u64 {
        return Err("foreign.publication.pointer-budget-or-kind".into());
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(metadata.len() as usize)
        .map_err(|_| "foreign.resource.pointer")?;
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        if bytes.len().checked_add(count).is_none_or(|n| n > MAX_ROOT) {
            return Err("foreign.resource.pointer".into());
        }
        if bytes.len() + count > bytes.capacity() {
            bytes
                .try_reserve_exact(count)
                .map_err(|_| "foreign.resource.pointer")?;
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    Ok(Some(bytes))
}
pub fn read_pointer_identity(pointer: &Path) -> Result<Option<String>, String> {
    read_pointer_bytes(pointer).map(|bytes| bytes.map(|b| hash(&b)))
}
#[cfg(windows)]
fn pointer_lock(pointer: &Path) -> Result<File, String> {
    use std::os::windows::fs::OpenOptionsExt;
    // Kernel sharing ownership, not a stale create_new sentinel. A crashed
    // publisher releases it automatically; keep the lock inode between writers.
    std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .share_mode(0)
        .open(pointer.with_extension("publication-lock"))
        .map_err(|e| format!("foreign.publication.busy: {e}"))
}
#[cfg(not(windows))]
fn pointer_lock(_: &Path) -> Result<File, String> {
    Err("foreign.publication.target-unsupported".into())
}
#[cfg(windows)]
fn atomic_replace(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    }
    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    // REPLACE_EXISTING | WRITE_THROUGH; both paths are on the same volume.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 0x1 | 0x8) } == 0 {
        Err(format!(
            "foreign.publication.replace: {}",
            std::io::Error::last_os_error()
        ))
    } else {
        Ok(())
    }
}
#[cfg(not(windows))]
fn atomic_replace(_: &Path, _: &Path) -> Result<(), String> {
    Err("foreign.publication.target-unsupported".into())
}

fn executable_identity(path: &Path) -> Result<(String, File), String> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0x1);
    }
    let mut file = options.open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_ARTIFACT as u64 {
        return Err("foreign.resource.executable".into());
    }
    let mut hasher = blake3::Hasher::new();
    let mut total = 0usize;
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        total = total.checked_add(n).ok_or("foreign.resource.executable")?;
        if total > MAX_ARTIFACT {
            return Err("foreign.resource.executable".into());
        }
        hasher.update(&buffer[..n]);
    }
    Ok((hasher.finalize().to_hex().to_string(), file))
}
