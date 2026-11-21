use super::{DirectoryGrant, FileAccessError};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, File, OpenOptions};
use same_file::Handle;
use std::ffi::OsStr;
use std::io::{self, ErrorKind};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub struct FileWritePermissions {
    pub create: bool,
    pub copy: bool,
    pub rename: bool,
    pub delete: bool,
}

impl std::str::FromStr for FileWritePermissions {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut permissions = Self::default();
        for operation in value.split(',').map(str::trim).filter(|name| !name.is_empty()) {
            match operation {
                "create" => permissions.create = true,
                "copy" => permissions.copy = true,
                "rename" => permissions.rename = true,
                "delete" => permissions.delete = true,
                _ => return Err(format!("Unknown File Manager write permission: {operation}")),
            }
        }
        Ok(permissions)
    }
}

/// A trusted window host submits these commands after validating its event owner.
#[derive(Clone, Copy, Debug)]
pub enum FileMutation<'a> {
    CreateFile(&'a Path),
    CreateDirectory(&'a Path),
    Copy { from: &'a Path, to: &'a Path },
    Rename { from: &'a Path, to: &'a Path },
    DeleteFile(&'a Path),
    DeleteEmptyDirectory(&'a Path),
}

impl<'a> FileMutation<'a> {
    /// Decode the shared File Manager request before trying to access its paths.
    pub fn from_request(operation: &str, path: &'a str, target: &'a str) -> Option<Self> {
        let from = Path::new(path);
        let to = Path::new(target);
        match (operation, target.is_empty()) {
            ("create-file", true) => Some(Self::CreateFile(from)),
            ("create-directory", true) => Some(Self::CreateDirectory(from)),
            ("copy", false) => Some(Self::Copy { from, to }),
            ("rename", false) => Some(Self::Rename { from, to }),
            ("delete-file", true) => Some(Self::DeleteFile(from)),
            ("delete-directory", true) => Some(Self::DeleteEmptyDirectory(from)),
            _ => None,
        }
    }

    pub(super) fn execute(self, grant: &DirectoryGrant) -> Result<(), FileAccessError> {
        let allowed = match self {
            Self::CreateFile(_) | Self::CreateDirectory(_) => grant.writes.create,
            Self::Copy { .. } => grant.writes.copy,
            Self::Rename { .. } => grant.writes.rename,
            Self::DeleteFile(_) | Self::DeleteEmptyDirectory(_) => grant.writes.delete,
        };
        if !allowed {
            return Err(FileAccessError::OperationDenied);
        }

        let root = &grant.directory;
        let open_entry = |path| -> Result<_, FileAccessError> {
            Ok(entry_parts(grant, path)?.open_parent(root, None)?)
        };
        match self {
            Self::Copy { from, to } | Self::Rename { from, to } => {
                // Validate both names before opening either side of the operation.
                let from = entry_parts(grant, from)?;
                let to = entry_parts(grant, to)?;
                let source = from.open_parent(root, None)?;
                match self {
                    Self::Copy { .. } => copy_entry(root, source, to)?,
                    _ => {
                        let target = to.open_parent(root, None)?;
                        rename_without_replace(
                            &source.parent,
                            source.name,
                            &target.parent,
                            target.name,
                        )?;
                    }
                }
            }
            Self::CreateFile(path) => {
                let entry = open_entry(path)?;
                entry
                    .parent
                    .open_with(entry.name, OpenOptions::new().write(true).create_new(true))?;
            }
            Self::CreateDirectory(path) => {
                let entry = open_entry(path)?;
                entry.parent.create_dir(entry.name)?;
            }
            Self::DeleteFile(path) => {
                let entry = open_entry(path)?;
                entry.parent.remove_file(entry.name)?;
            }
            Self::DeleteEmptyDirectory(path) => {
                let entry = open_entry(path)?;
                entry.parent.remove_dir(entry.name)?;
            }
        }
        Ok(())
    }
}

struct EntryParts<'a> {
    parent: &'a Path,
    name: &'a OsStr,
}

struct Entry<'a> {
    parent: Dir,
    name: &'a OsStr,
}

fn entry_parts<'a>(
    grant: &DirectoryGrant,
    path: &'a Path,
) -> Result<EntryParts<'a>, FileAccessError> {
    let relative = grant.relative(path)?;
    let name = relative.file_name().ok_or(FileAccessError::OutsideRoot)?;
    // A Windows component containing a colon can name an alternate data stream.
    #[cfg(windows)]
    if relative.components().any(|component| {
        component
            .as_os_str()
            .as_encoded_bytes()
            .iter()
            .any(|&byte| byte == b':' || byte == 0)
    }) {
        return Err(FileAccessError::OutsideRoot);
    }
    Ok(EntryParts {
        parent: relative.parent().unwrap_or(Path::new("")),
        name,
    })
}

impl<'a> EntryParts<'a> {
    fn open_parent(self, root: &Dir, source: Option<&Handle>) -> io::Result<Entry<'a>> {
        let mut parent = root.try_clone()?;
        for part in self.parent.components() {
            reject_copy_into_self(&parent, source)?;
            parent = parent.open_dir_nofollow(part.as_os_str())?;
        }
        reject_copy_into_self(&parent, source)?;
        Ok(Entry {
            parent,
            name: self.name,
        })
    }
}

pub(super) fn with_open_target<T>(
    grant: &DirectoryGrant,
    path: &Path,
    launch: impl FnOnce(&Path) -> io::Result<T>,
) -> Result<T, FileAccessError> {
    let entry = entry_parts(grant, path)?.open_parent(&grant.directory, None)?;
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    #[cfg(unix)]
    cap_fs_ext::OpenOptionsSyncExt::nonblock(&mut options, true);
    #[cfg(windows)]
    {
        use cap_std::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};
        // Keep this name from being removed or replaced before the launcher
        // has consumed it. Other readers and writers can still open the file.
        options.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE);
    }
    if !entry.parent.symlink_metadata(entry.name)?.is_file() {
        return Err(io::Error::new(ErrorKind::InvalidInput, "open requires a regular file").into());
    }
    let file = entry.parent.open_with(entry.name, &options)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(ErrorKind::InvalidInput, "open requires a regular file").into());
    }
    let handle = Handle::from_file(file.into_std())?;
    let resolved = path.canonicalize()?;
    // A desktop association takes a path. Check that the ambient name still
    // names the object obtained through the held directory capability.
    if handle != Handle::from_path(&resolved)? {
        return Err(FileAccessError::OutsideRoot);
    }
    launch(&resolved).map_err(Into::into)
}

fn reject_copy_into_self(directory: &Dir, source: Option<&Handle>) -> io::Result<()> {
    if let Some(source) = source {
        if source == &directory_identity(directory)? {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "cannot copy a directory into itself",
            ));
        }
    }
    Ok(())
}

fn directory_identity(directory: &Dir) -> io::Result<Handle> {
    Handle::from_file(directory.try_clone()?.into_std_file())
}

fn copy_entry(root: &Dir, source: Entry<'_>, target: EntryParts<'_>) -> io::Result<()> {
    let source = CopySource::open(&source.parent, source.name)?;
    let source_handle = match &source {
        CopySource::Directory(directory) => Some(directory_identity(directory)?),
        CopySource::File(_) => None,
    };
    let target = target.open_parent(root, source_handle.as_ref())?;
    match target.parent.symlink_metadata(target.name) {
        Ok(_) => return Err(io::Error::from(ErrorKind::AlreadyExists)),
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    // Build the entire copy in a temporary sibling. The final rename publishes
    // it in one operation, with the destination required to be absent.
    let stage = copy_stage(&target.parent)?;
    let name = OsStr::new("entry");
    let result = directory_identity(&stage)
        .and_then(|stage_identity| source.copy_to(&stage, name, &stage_identity))
        .and_then(|()| rename_without_replace(&stage, name, &target.parent, target.name));
    if let Err(error) = stage.remove_open_dir_all() {
        log::warn!("could not remove a file copy staging directory: {error}");
    }
    result
}

fn copy_stage(parent: &Dir) -> io::Result<Dir> {
    loop {
        let name = format!(".auto-copy-{:032x}", fastrand::u128(..));
        match parent.create_dir(&name) {
            Ok(()) => {
                let stage = parent.open_dir_nofollow(&name);
                if stage.is_err() {
                    let _ = parent.remove_dir(&name);
                }
                return stage;
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

enum CopySource {
    Directory(Dir),
    File(File),
}

impl CopySource {
    fn open(parent: &Dir, name: &OsStr) -> io::Result<Self> {
        let kind = parent.symlink_metadata(name)?.file_type();
        if kind.is_dir() {
            return parent.open_dir_nofollow(name).map(Self::Directory);
        }
        if !kind.is_file() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "copy requires a regular file or directory",
            ));
        }
        let mut read = OpenOptions::new();
        read.read(true).follow(FollowSymlinks::No);
        #[cfg(unix)]
        cap_fs_ext::OpenOptionsSyncExt::nonblock(&mut read, true);
        let file = parent.open_with(name, &read)?;
        if !file.metadata()?.is_file() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "copy requires a regular file",
            ));
        }
        Ok(Self::File(file))
    }

    fn copy_to(self, target: &Dir, name: &OsStr, stage: &Handle) -> io::Result<()> {
        match self {
            Self::Directory(source) => {
                // A concurrent directory move must not turn staging into input.
                reject_copy_into_self(&source, Some(stage))?;
                target.create_dir(name)?;
                let target = target.open_dir_nofollow(name)?;
                for item in source.entries()? {
                    let name = item?.file_name();
                    Self::open(&source, &name)?.copy_to(&target, &name, stage)?;
                }
            }
            Self::File(mut source) => {
                let mut output =
                    target.open_with(name, OpenOptions::new().write(true).create_new(true))?;
                io::copy(&mut source, &mut output)?;
                output.set_permissions(source.metadata()?.permissions())?;
            }
        }
        Ok(())
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "redox"
))]
fn rename_without_replace(from: &Dir, name: &OsStr, to: &Dir, new_name: &OsStr) -> io::Result<()> {
    rustix::fs::renameat_with(from, name, to, new_name, rustix::fs::RenameFlags::NOREPLACE)?;
    Ok(())
}

#[cfg(windows)]
fn rename_without_replace(from: &Dir, name: &OsStr, to: &Dir, new_name: &OsStr) -> io::Result<()> {
    use cap_fs_ext::{OpenOptionsExt, OpenOptionsMaybeDirExt};
    use std::mem::{offset_of, size_of};
    use std::os::windows::{ffi::OsStrExt, io::AsRawHandle};
    use windows_sys::Wdk::Storage::FileSystem::{
        FileRenameInformation, NtSetInformationFile, FILE_RENAME_INFORMATION,
    };
    use windows_sys::Win32::{
        Foundation::RtlNtStatusToDosError, Storage::FileSystem::DELETE, System::IO::IO_STATUS_BLOCK,
    };

    let mut options = OpenOptions::new();
    options
        .access_mode(DELETE)
        .maybe_dir(true)
        .follow(FollowSymlinks::No);
    let file = from.open_with(name, &options)?;
    let name: Vec<_> = new_name.encode_wide().collect();
    let name_bytes = std::mem::size_of_val(name.as_slice());
    let size = name_bytes
        .checked_add(size_of::<FILE_RENAME_INFORMATION>())
        .and_then(|size| u32::try_from(size).ok())
        .ok_or_else(|| io::Error::from(ErrorKind::InvalidInput))?;
    let mut buffer = vec![0_usize; (size as usize).div_ceil(size_of::<usize>())];
    let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
    // The flexible array contains the destination leaf relative to the held
    // destination directory. The zero-initialized replacement flag is false.
    let mut completion = IO_STATUS_BLOCK::default();
    let status = unsafe {
        (*info).RootDirectory = to.as_raw_handle();
        (*info).FileNameLength = name_bytes as u32;
        let filename = info
            .cast::<u8>()
            .add(offset_of!(FILE_RENAME_INFORMATION, FileName))
            .cast::<u16>();
        std::ptr::copy_nonoverlapping(name.as_ptr(), filename, name.len());
        NtSetInformationFile(
            file.as_raw_handle(),
            &mut completion,
            info.cast(),
            size,
            FileRenameInformation,
        )
    };
    if status < 0 {
        Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(status) } as i32,
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(any(
    windows,
    target_os = "linux",
    target_os = "android",
    target_vendor = "apple",
    target_os = "redox"
)))]
fn rename_without_replace(_: &Dir, _: &OsStr, _: &Dir, _: &OsStr) -> io::Result<()> {
    Err(io::Error::from(ErrorKind::Unsupported))
}
