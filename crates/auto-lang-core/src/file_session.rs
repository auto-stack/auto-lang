//! Host-owned authorization for a file browser session.

use cap_std::{ambient_authority, fs::Dir};
use serde::Serialize;
use std::cell::RefCell;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

mod mutation;
pub use mutation::{FileMutation, FileWritePermissions};
#[cfg(test)]
mod tests;

thread_local! {
    static REQUEST_SESSION: RefCell<Option<Arc<FileSession>>> = const { RefCell::new(None) };
}

/// Bind an already-authorized host session to this synchronous VM request.
/// Nested invocations and unwinding restore the previous request.
pub fn with_file_session<T>(session: Arc<FileSession>, invoke: impl FnOnce() -> T) -> T {
    struct Restore(Option<Arc<FileSession>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            REQUEST_SESSION.with(|slot| slot.replace(self.0.take()));
        }
    }
    let _restore = Restore(REQUEST_SESSION.with(|slot| slot.replace(Some(session))));
    invoke()
}

pub(crate) fn current_session() -> Option<Arc<FileSession>> {
    REQUEST_SESSION.with(|slot| slot.borrow().clone())
}

#[derive(Debug, thiserror::Error)]
pub enum FileAccessError {
    #[error("file session is closed")]
    Closed,
    #[error("file session has no directory read grant")]
    NoReadGrant,
    #[error("file session has no grant for this operation")]
    OperationDenied,
    #[error("path is outside the authorized root")]
    OutsideRoot,
    #[error("file path is not UTF-8")]
    InvalidEncoding,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub mtime: i64,
    pub extension: String,
}

#[derive(Debug, Serialize)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<DirectoryEntry>,
}

#[derive(Default)]
pub struct FileSession {
    // A revoke or close completes after an in-flight operation exits the lock.
    access: Mutex<Access>,
}

#[derive(Default)]
enum Access {
    #[default]
    Unassigned,
    Granted(DirectoryGrant),
    Closed,
}

struct DirectoryGrant {
    root: PathBuf,
    directory: Dir,
    writes: FileWritePermissions,
}

impl FileSession {
    /// Only the host can grant a directory. Authorization captures its handle
    /// once, and every subsequent access is relative to that handle.
    pub fn grant_read(&self, root: &Path) -> Result<(), FileAccessError> {
        self.grant(root, FileWritePermissions::default())
    }

    /// Replace the host grant, including the allowed file mutations.
    pub fn grant(&self, root: &Path, writes: FileWritePermissions) -> Result<(), FileAccessError> {
        let mut access = self.access.lock().map_err(|_| FileAccessError::Closed)?;
        if matches!(*access, Access::Closed) {
            return Err(FileAccessError::Closed);
        }
        let root = std::path::absolute(root)?;
        display_path(&root)?;
        let directory = Dir::open_ambient_dir(&root, ambient_authority())?;
        *access = Access::Granted(DirectoryGrant {
            root,
            directory,
            writes,
        });
        Ok(())
    }

    pub fn revoke(&self) {
        if let Ok(mut access) = self.access.lock() {
            if matches!(*access, Access::Granted(_)) {
                *access = Access::Unassigned;
            }
        }
    }

    pub fn close(&self) {
        if let Ok(mut access) = self.access.lock() {
            *access = Access::Closed;
        }
    }

    pub fn read_root(&self) -> Result<String, FileAccessError> {
        self.with_grant(|grant| display_path(&grant.root))
    }

    /// Start an independent read-only session on the same held directory.
    pub fn fork_read(&self) -> Result<Self, FileAccessError> {
        self.fork_grant_with(|_| FileWritePermissions::default())
    }

    /// Start an independent session with the same directory handle and permissions.
    pub fn fork_grant(&self) -> Result<Self, FileAccessError> {
        self.fork_grant_with(|writes| writes)
    }

    fn fork_grant_with(
        &self,
        writes: impl FnOnce(FileWritePermissions) -> FileWritePermissions,
    ) -> Result<Self, FileAccessError> {
        self.with_grant(|grant| {
            Ok(Self {
                access: Mutex::new(Access::Granted(DirectoryGrant {
                    root: grant.root.clone(),
                    directory: grant.directory.try_clone()?,
                    writes: writes(grant.writes),
                })),
            })
        })
    }

    pub fn list(&self, path: &Path) -> Result<DirectoryListing, FileAccessError> {
        self.with_grant(|grant| grant.list(path))
    }

    /// Read the currently granted root while holding the same authorization as the listing.
    pub fn list_root(&self) -> Result<DirectoryListing, FileAccessError> {
        self.with_grant(|grant| grant.list(&grant.root))
    }

    /// Execute one authorized request. A revoked or closed session cannot start it.
    pub fn mutate(&self, operation: FileMutation<'_>) -> Result<(), FileAccessError> {
        self.with_grant(|grant| operation.execute(grant))
    }

    /// Supply a checked regular-file path for a synchronous desktop launch.
    /// The grant and the opened file remain held until the launcher returns.
    pub fn with_open_target<T>(
        &self,
        path: &Path,
        launch: impl FnOnce(&Path) -> std::io::Result<T>,
    ) -> Result<T, FileAccessError> {
        self.with_grant(|grant| mutation::with_open_target(grant, path, launch))
    }

    fn with_grant<T>(
        &self,
        execute: impl FnOnce(&DirectoryGrant) -> Result<T, FileAccessError>,
    ) -> Result<T, FileAccessError> {
        match &*self.access.lock().map_err(|_| FileAccessError::Closed)? {
            Access::Granted(grant) => execute(grant),
            Access::Unassigned => Err(FileAccessError::NoReadGrant),
            Access::Closed => Err(FileAccessError::Closed),
        }
    }
}

impl DirectoryGrant {
    fn relative<'a>(&self, path: &'a Path) -> Result<&'a Path, FileAccessError> {
        let suffix = path
            .strip_prefix(&self.root)
            .map_err(|_| FileAccessError::OutsideRoot)?;
        if !suffix
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err(FileAccessError::OutsideRoot);
        }
        Ok(suffix)
    }

    fn list(&self, path: &Path) -> Result<DirectoryListing, FileAccessError> {
        let relative: PathBuf = self.relative(path)?.components().collect();
        let directory = self
            .directory
            .open_dir(if relative.as_os_str().is_empty() {
                Path::new(".")
            } else {
                &relative
            })?;
        let display = if relative.as_os_str().is_empty() {
            self.root.clone()
        } else {
            self.root.join(relative)
        };
        let mut entries = Vec::new();
        for result in directory.entries()? {
            let entry = result?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| FileAccessError::InvalidEncoding)?;
            // Inspect the directory entry itself; following a link here would
            // reveal metadata about a different entry.
            let metadata = directory.symlink_metadata(&name)?;
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|time| time.into_std().duration_since(std::time::UNIX_EPOCH).ok())
                .and_then(|duration| i64::try_from(duration.as_secs()).ok())
                .unwrap_or(-1);
            let extension = Path::new(&name)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(str::to_lowercase)
                .unwrap_or_default();
            entries.push(DirectoryEntry {
                path: display_path(&display.join(&name))?,
                name,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                mtime,
                extension,
            });
        }
        Ok(DirectoryListing {
            path: display_path(&display)?,
            entries,
        })
    }
}

fn display_path(path: &Path) -> Result<String, FileAccessError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or(FileAccessError::InvalidEncoding)
}
