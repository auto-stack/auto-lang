//! Selected-target readiness and the only durable current-pointer switch.
//! No current source, declarations or body artifacts change during linking,
//! execution, persistence, cancellation or failed pointer replacement.
use super::publication_scratch::Scratch;
use super::*;
use ace_foreign_contract::manual::HostAuthority;
use std::{fs::File, io::Read, path::PathBuf, sync::atomic::AtomicBool};

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
    work: Vec<Work>,
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
    pub fn publication_maintenance(&self) -> &PublicationMaintenance {
        &self.publication_maintenance
    }
    /// Retry only owned root retirement. A crash on either side of the pointer
    /// switch leaves at most the old and prepared generation. The actual
    /// pointer hash chooses the survivor without decoding a caller's wrapper.
    pub fn retry_publication_maintenance(
        &mut self,
        store: &ManagedStore,
        pointer: &Path,
    ) -> Result<usize, String> {
        let _lock = pointer_lock(pointer)?;
        let observed = read_pointer_identity(pointer)?;
        let project = publication_root_project(pointer)?;
        let result: Result<(usize, super::publication_scratch::Swept), String> = (|| {
            let roots = store
                .root_artifacts(&project, observed.as_deref().unwrap_or(""))
                .map_err(|e| e.to_string())?;
            if observed.is_some() && roots.is_empty() {
                return Err("foreign.publication.unrecognized-current-root".into());
            }
            let turn = store.staging_turn().map_err(|e| e.to_string())?;
            let scratch = Scratch::open(store, &project, self.front.resources(), &turn)?;
            let swept = scratch.sweep()?;
            let retired = store
                .retain_root_kind(&project, observed.as_deref())
                .map_err(|e| e.to_string())?;
            Ok((retired, swept))
        })();
        self.publication_maintenance = match &result {
            Ok((roots_retired, swept)) => PublicationMaintenance {
                roots_retired: *roots_retired,
                temp_files_removed: swept.files,
                temp_bytes_removed: swept.bytes,
                cleanup_error: None,
            },
            Err(error) => PublicationMaintenance {
                roots_retired: 0,
                cleanup_error: Some(error.clone()),
                ..PublicationMaintenance::default()
            },
        };
        result.map(|(retired, _)| retired)
    }
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
        let mut work = Vec::new();
        let exe = link_artifacts(
            directory,
            self.target,
            source,
            environment,
            provider,
            &self.runtime,
            &artifacts,
            &mut self.meter,
            &self.front.work(),
            &mut work,
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
            work,
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
    /// Production recovery accepts current HOST authority, not a freshly
    /// projected environment. All logical/physical/declaration/binding data
    /// comes from the accepted required closure and remains untrusted until
    /// deep validation and the final fresh qualification below.
    pub fn restore_stored_current(
        store: &ManagedStore,
        stored: &Stored,
        target: Target,
        capabilities: &[String],
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
    ) -> Result<Self, String> {
        Self::restore_stored_current_with_resources(
            store,
            stored,
            target,
            capabilities,
            current,
            Resources::default(),
        )
    }
    pub fn restore_stored_current_with_resources(
        store: &ManagedStore,
        stored: &Stored,
        target: Target,
        capabilities: &[String],
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
        resources: Resources,
    ) -> Result<Self, String> {
        let authority = current()?;
        let restored = Self::restore_stored_host_with_resources(
            store,
            stored,
            target,
            capabilities,
            authority.foreign,
            resources,
        )?;
        qualify(
            restored.front.foreign(),
            &restored.runtime_identity,
            target,
            current,
        )?;
        Ok(restored)
    }
    pub fn restore_current(
        root: &[u8],
        expected: &str,
        target: Target,
        capabilities: &[String],
        read: impl FnMut(&str) -> Option<Arc<Vec<u8>>>,
        current: &mut impl FnMut() -> Result<CurrentAuthority, String>,
    ) -> Result<Self, String> {
        let authority = current()?;
        let restored = Self::restore_host(
            root,
            expected,
            target,
            capabilities,
            authority.foreign,
            read,
        )?;
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
        let project = publication_root_project(pointer)?;
        if let Some(kind) = &observed {
            if store
                .root_artifacts(&project, kind)
                .map_err(|e| e.to_string())?
                .is_empty()
            {
                return Err("foreign.publication.unrecognized-current-root".into());
            }
        }
        let (retired_before, swept) = {
            let turn = store.staging_turn().map_err(|e| e.to_string())?;
            let scratch = Scratch::open(store, &project, self.front.resources(), &turn)?;
            let swept = scratch.sweep()?;
            let retired = store
                .retain_root_kind(&project, observed.as_deref())
                .map_err(|e| format!("foreign.publication.recovery: {e}"))?;
            (retired, swept)
        };
        let (stored, lease) = prepared.snapshot.persist_leased(store, previous)?;
        let bytes = encode_pointer(&stored)?;
        if bytes.len() > MAX_ROOT {
            return Err("foreign.resource.pointer".into());
        }
        let turn = store.staging_turn().map_err(|e| e.to_string())?;
        let scratch = Scratch::open(store, &project, self.front.resources(), &turn)?;
        let kind = hash(&bytes);
        let temp = scratch.write(bytes, &turn)?;
        let mut introduced = false;
        // Last gates are after every fallible producer and before the sole
        // current-pointer switch. No source/HIR/target is installed on failure.
        let switch = (|| {
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
            let existing = store
                .root_artifacts(&project, &kind)
                .map_err(|e| e.to_string())?;
            if existing.iter().any(|id| id != &stored.artifact_id) {
                return Err("foreign.publication.pointer-content-conflict".into());
            }
            store
                .register_root(&project, &stored.artifact_id, &kind)
                .map_err(|e| e.to_string())?;
            introduced = existing.is_empty();
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
            atomic_replace(&temp.path, pointer)
        })();
        if let Err(error) = switch {
            temp.remove()
                .map_err(|cleanup| format!("{error}; {cleanup}"))?;
            if introduced {
                store
                    .remove_root(&project, &kind, &stored.artifact_id)
                    .map_err(|cleanup| {
                        format!("{error}; foreign.publication.rollback-pending: {cleanup}")
                    })?;
            }
            return Err(error);
        }
        temp.moved();
        match prepared.edit {
            Edit::Current => {}
            Edit::Body(body) => self.commit(body)?,
            Edit::Foreign(foreign) => self.commit_foreign(foreign)?,
        }
        for output in prepared.work {
            output.accept();
        }
        self.root_lease = Some(lease);
        // The candidate is now accepted. Retirement is a separate maintenance
        // result; reporting it as an ordinary rejected candidate would lie
        // about the already committed pointer/source/target. Next publication
        // retries this before persisting a new candidate under the lock.
        self.publication_maintenance = match store.retain_root_kind(&project, Some(&kind)) {
            Ok(retired) => PublicationMaintenance {
                roots_retired: retired_before + retired,
                temp_files_removed: swept.files,
                temp_bytes_removed: swept.bytes,
                cleanup_error: None,
            },
            Err(error) => PublicationMaintenance {
                roots_retired: retired_before,
                temp_files_removed: swept.files,
                temp_bytes_removed: swept.bytes,
                cleanup_error: Some(error.to_string()),
            },
        };
        Ok(stored)
    }
}

/// Namespace owned by one accepted pointer. Other projects' roots, delivered
/// artifacts and active reader leases are outside this retirement operation.
pub fn publication_root_project(pointer: &Path) -> Result<String, String> {
    let parent = pointer
        .parent()
        .ok_or("foreign.publication.pointer-parent")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let path = parent.join(
        pointer
            .file_name()
            .ok_or("foreign.publication.pointer-name")?,
    );
    Ok(format!(
        "source938-pointer-{}",
        hash(
            path.to_str()
                .ok_or("foreign.publication.non-utf8-pointer")?
                .as_bytes()
        )
    ))
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
