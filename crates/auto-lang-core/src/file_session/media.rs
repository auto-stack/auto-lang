use super::{mutation, DirectoryGrant, FileAccessError, FileSession};
use auto_media_core::image_pipeline::{
    supported_media_path, MediaAssetError, MediaAssetId, MediaHttpResponse, MediaReadSource, MediaRuntime,
    MediaSurfaceStatus,
    MAX_IMAGE_FILE_BYTES, MAX_SESSION_ENTRIES,
};
use cap_fs_ext::DirExt;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

impl FileSession {
    /// Give a hosted viewer its authored source directory for project-relative
    /// sample paths. File access still requires a separate directory grant.
    pub fn for_media_source(source_dir: &Path) -> Result<Self, FileAccessError> {
        Ok(Self {
            media_source_dir: Some(std::path::absolute(source_dir)?),
            ..Self::default()
        })
    }

    pub(crate) fn open_media(&self, path: &Path) -> Result<String, FileAccessError> {
        self.with_grant(|grant| {
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else if let Some(source) = self.media_source_dir.as_ref()
                .filter(|_| path.starts_with(".."))
            {
                authored_media_path(source, path)?
            } else {
                grant.root.join(path)
            };
            let paths = media_paths(grant, &path)?;
            let mut sources = Vec::new();
            for path in paths.into_iter().take(MAX_SESSION_ENTRIES) {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or(FileAccessError::InvalidEncoding)?;
                let ticket = self.ticket(grant, path.clone());
                sources.push(MediaReadSource::new(name.to_owned(), move || {
                    ticket
                        .read(MAX_IMAGE_FILE_BYTES)
                        .map(Arc::from)
                        .map_err(|error| MediaAssetError::Rejected(error.to_string()))
                }));
            }
            let mut media = self.media.lock().map_err(|_| FileAccessError::Closed)?;
            let runtime = media.get_or_insert_with(|| MediaRuntime::new(Duration::from_secs(30)));
            Ok(runtime.open_read_session(sources))
        })
    }

    pub(crate) fn with_media_runtime<T>(
        &self,
        use_runtime: impl FnOnce(&MediaRuntime) -> T,
    ) -> Option<T> {
        self.with_grant(|_| {
            let media = self.media.lock().map_err(|_| FileAccessError::Closed)?;
            Ok(media.as_ref().map(use_runtime))
        })
        .ok()
        .flatten()
    }

    /// The private transport can serve only bytes from this file session.
    pub fn media_response(
        &self,
        method: &str,
        path: &str,
        if_none_match: Option<&str>,
    ) -> Option<MediaHttpResponse> {
        self.with_media_runtime(|runtime| {
            runtime.session_media_response(method, path, if_none_match)
        })
        .flatten()
    }

    pub fn media_status(&self, path: &str) -> Option<MediaSurfaceStatus> {
        self.with_media_runtime(|runtime| runtime.session_media_status(path)).flatten()
    }

    pub fn media_render(&self, path: &str) -> Option<(MediaAssetId, u32, u32, Arc<[u8]>)> {
        self.with_media_runtime(|runtime| runtime.session_media_render(path)).flatten()
    }
}

fn authored_media_path(source: &Path, relative: &Path) -> Result<PathBuf, FileAccessError> {
    let mut resolved = PathBuf::new();
    for part in source.components().chain(relative.components()) {
        match part {
            Component::ParentDir => {
                if !resolved.pop() {
                    return Err(FileAccessError::OutsideRoot);
                }
            }
            Component::CurDir => {}
            _ => resolved.push(part.as_os_str()),
        }
    }
    Ok(resolved)
}

fn media_paths(grant: &DirectoryGrant, path: &Path) -> Result<Vec<PathBuf>, FileAccessError> {
    let relative = grant.relative(path)?;
    let directory = relative
        .components()
        .try_fold(grant.directory.try_clone()?, |parent, part| {
            parent.open_dir_nofollow(part.as_os_str())
        });
    match directory {
        Ok(directory) => {
            let mut files = Vec::new();
            for entry in directory.entries()?.flatten() {
                let file = path.join(entry.file_name());
                if entry.file_type().is_ok_and(|kind| kind.is_file()) && supported_media_path(&file)
                {
                    files.push(file);
                }
            }
            files.sort();
            Ok(files)
        }
        Err(_) if supported_media_path(path) => {
            mutation::open_read_file(grant, path)?;
            Ok(vec![path.to_path_buf()])
        }
        Err(error) => Err(error.into()),
    }
}
