//! Pointer copies are original-store staging, never filename-selected files
//! in a user's source/output directory. The original staging/liveness turn
//! owns quota and GC exclusion. The pointer lock excludes active publishers.
use super::*;
use ace_artifacts::store::{is_reparse, StagingTurn};
use std::io::{Read, Write};

pub(super) struct Scratch {
    root: std::path::PathBuf,
    resources: Resources,
    _index: Reservation,
}
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Swept {
    pub files: usize,
    pub bytes: u64,
}
pub(super) struct Temp {
    pub path: std::path::PathBuf,
    bytes: Vec<u8>,
    file: std::fs::File,
    _io: Reservation,
    armed: bool,
}
impl Temp {
    pub fn moved(mut self) {
        self.armed = false;
    }
    pub fn remove(mut self) -> Result<(), String> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("foreign.publication.temp-cleanup: {e}")),
        }
        self.armed = false;
        Ok(())
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        if self.armed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
impl Scratch {
    pub fn open(
        store: &ManagedStore,
        project: &str,
        resources: Resources,
        turn: &StagingTurn<'_>,
    ) -> Result<Self, String> {
        let index = resources
            .reserve(Charge::index(
                store
                    .root
                    .as_os_str()
                    .len()
                    .saturating_mul(8)
                    .saturating_add(4096),
            ))
            .map_err(|e| e.to_string())?;
        let root = store.root.join("staging").join(project);
        if is_reparse(&root) {
            return Err("foreign.publication.staging-reparse".into());
        }
        let marker = root.join("source938-publication.marker");
        const OWNER: &[u8] = b"abi.source-publication-staging/1\noriginal-managed-store-owned\n";
        if !root.exists() {
            turn.admit_bytes(OWNER.len() as u64)
                .map_err(|e| e.to_string())?;
            std::fs::create_dir(&root).map_err(|e| e.to_string())?;
        }
        if !root.is_dir() {
            return Err("foreign.publication.staging-kind".into());
        }
        if is_reparse(&marker) {
            return Err("foreign.publication.staging-marker-reparse".into());
        }
        if marker.exists() {
            let meta = marker.metadata().map_err(|e| e.to_string())?;
            if !meta.is_file() || meta.len() > OWNER.len() as u64 {
                return Err("foreign.publication.staging-owner".into());
            }
            let _io = resources
                .reserve(Charge::io(OWNER.len() * 2))
                .map_err(|e| e.to_string())?;
            let mut file = std::fs::File::open(&marker).map_err(|e| e.to_string())?;
            let mut content = [0u8; OWNER.len()];
            let len = meta.len() as usize;
            file.read_exact(&mut content[..len])
                .map_err(|e| e.to_string())?;
            let mut extra = [0u8; 1];
            if file.read(&mut extra).map_err(|e| e.to_string())? != 0
                || content[..len] != OWNER[..len]
            {
                return Err("foreign.publication.staging-owner".into());
            }
            drop(file);
            if len < OWNER.len() {
                // Repair only the exact expected initialization prefix in
                // this original-store-owned subdirectory with no other files.
                for entry in std::fs::read_dir(&root).map_err(|e| e.to_string())? {
                    if entry.map_err(|e| e.to_string())?.file_name()
                        != "source938-publication.marker"
                    {
                        return Err("foreign.publication.staging-unowned".into());
                    }
                }
                turn.admit_bytes((OWNER.len() - len) as u64)
                    .map_err(|e| e.to_string())?;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&marker)
                    .map_err(|e| e.to_string())?;
                file.write_all(OWNER)
                    .and_then(|_| file.sync_all())
                    .map_err(|e| e.to_string())?;
            }
        } else {
            // The namespace is under the independently admitted original
            // store, not under a candidate-selected user directory. Repair
            // only an EMPTY initialization; never claim existing user files.
            if std::fs::read_dir(&root)
                .map_err(|e| e.to_string())?
                .next()
                .is_some()
            {
                return Err("foreign.publication.staging-unowned".into());
            }
            turn.admit_bytes(OWNER.len() as u64)
                .map_err(|e| e.to_string())?;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&marker)
                .map_err(|e| e.to_string())?;
            file.write_all(OWNER)
                .and_then(|_| file.sync_all())
                .map_err(|e| e.to_string())?;
        }
        Ok(Self {
            root,
            resources,
            _index: index,
        })
    }
    pub fn sweep(&self) -> Result<Swept, String> {
        let mut result = Swept::default();
        for entry in std::fs::read_dir(&self.root).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some(numbers) = name
                .strip_prefix("pointer-")
                .and_then(|n| n.strip_suffix(".tmp"))
            else {
                continue;
            };
            let Some((pid, serial)) = numbers.split_once('-') else {
                continue;
            };
            if pid.is_empty()
                || serial.is_empty()
                || !pid.bytes().all(|b| b.is_ascii_digit())
                || !serial.bytes().all(|b| b.is_ascii_digit())
            {
                continue;
            }
            let path = entry.path();
            if is_reparse(&path) {
                return Err("foreign.publication.staging-reparse".into());
            }
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            if !metadata.is_file() {
                return Err("foreign.publication.staging-kind".into());
            }
            std::fs::remove_file(&path)
                .map_err(|e| format!("foreign.publication.temp-cleanup: {e}"))?;
            result.files += 1;
            result.bytes = result
                .bytes
                .checked_add(metadata.len())
                .ok_or("foreign.resource.staging-overflow")?;
        }
        Ok(result)
    }
    pub fn write(&self, bytes: Vec<u8>, turn: &StagingTurn<'_>) -> Result<Temp, String> {
        turn.admit_bytes(bytes.len() as u64)
            .map_err(|e| e.to_string())?;
        let io = self
            .resources
            .reserve(Charge::io(bytes.capacity()))
            .map_err(|e| e.to_string())?;
        static SERIAL: AtomicU64 = AtomicU64::new(1);
        let path = self.root.join(format!(
            "pointer-{}-{}.tmp",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = std::fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // The exact bytes remain immutable through final qualification.
            // DELETE sharing permits the atomic move; WRITE sharing is denied.
            options.share_mode(0x1 | 0x4);
        }
        let file = options.open(&path).map_err(|e| e.to_string())?;
        let mut temp = Temp {
            path,
            bytes,
            file,
            _io: io,
            armed: true,
        };
        if let Err(error) = temp
            .file
            .write_all(&temp.bytes)
            .and_then(|_| temp.file.sync_all())
        {
            temp.remove()
                .map_err(|cleanup| format!("{error}; {cleanup}"))?;
            return Err(error.to_string());
        }
        Ok(temp)
    }
}
