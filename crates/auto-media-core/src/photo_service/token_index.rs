use super::{sanitize_rel_dir, PhotoListing, PhotoTokens};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub(super) fn directory(root: &Path) -> Option<PathBuf> {
    // Unit tests supply their own index directory and never write user state.
    if cfg!(test) {
        return None;
    }
    let root = root.canonicalize().ok()?;
    let identity = blake3::hash(root.as_os_str().as_encoded_bytes()).to_hex();
    Some(
        state_directory()?
            .join("auto-desktop")
            .join("photo-index-v1")
            .join(identity.as_str()),
    )
}

#[cfg(windows)]
fn state_directory() -> Option<PathBuf> {
    absolute_env_path("LOCALAPPDATA")
}

#[cfg(not(windows))]
fn state_directory() -> Option<PathBuf> {
    absolute_env_path("XDG_STATE_HOME")
        .or_else(|| absolute_env_path("HOME").map(|home| home.join(".local/state")))
}

fn absolute_env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

fn snapshot_name(relative_dir: &str) -> String {
    format!("{}.paths", blake3::hash(relative_dir.as_bytes()).to_hex())
}

pub(super) fn load(index: &Path) -> PhotoTokens {
    let mut tokens = PhotoTokens::default();
    let Ok(files) = fs::read_dir(index) else {
        return tokens;
    };
    for file in files.flatten() {
        let file_name = file.file_name();
        if !file.file_type().is_ok_and(|kind| kind.is_file())
            || Path::new(&file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                != Some("paths")
        {
            continue;
        }
        let Ok(snapshot) = fs::read_to_string(file.path()) else {
            continue;
        };
        if !snapshot.ends_with('\0') {
            continue;
        }
        let Some((relative_dir, records)) = snapshot.split_once('\0') else {
            continue;
        };
        if sanitize_rel_dir(relative_dir).as_deref() != Some(relative_dir)
            || Path::new(&file_name) != Path::new(&snapshot_name(relative_dir))
        {
            continue;
        }
        let entries = records.split_terminator('\0').filter_map(|relative_path| {
            let (parent, name) = relative_path
                .rsplit_once('/')
                .unwrap_or(("", relative_path));
            if parent != relative_dir
                || relative_path.starts_with('/')
                || Path::new(name).file_name().and_then(|part| part.to_str()) != Some(name)
            {
                return None;
            }
            Some((
                blake3::hash(relative_path.as_bytes()).to_hex().to_string(),
                relative_path.to_owned(),
            ))
        });
        tokens.replace(relative_dir, entries);
    }
    tokens
}

/// Directory snapshots are independent so two processes can update different albums.
pub(super) fn save(index: &Path, listing: &PhotoListing) -> io::Result<()> {
    let destination = index.join(snapshot_name(&listing.dir_rel));
    if listing.images.is_empty() {
        return match fs::remove_file(destination) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        };
    }

    // Filesystem paths cannot contain NUL, unlike newlines or other whitespace.
    let mut snapshot = listing.dir_rel.clone();
    snapshot.push('\0');
    for entry in &listing.images {
        snapshot.push_str(&entry.relative_path);
        snapshot.push('\0');
    }
    if fs::read_to_string(&destination).is_ok_and(|previous| previous == snapshot) {
        return Ok(());
    }

    fs::create_dir_all(index)?;
    let temporary = index.join(format!("{:016x}.tmp", fastrand::u64(..)));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let write = file
        .write_all(snapshot.as_bytes())
        .and_then(|()| file.sync_all());
    drop(file);
    let result = write.and_then(|()| fs::rename(&temporary, &destination));
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
