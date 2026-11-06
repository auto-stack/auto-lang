//! The original compiler-reported dependency walk with pre-allocation admission.
use crate::cache_paths::{metadata_array, metadata_size, native_path_size};
use std::path::{Path, PathBuf};

pub(crate) fn parse(
    stdout: &str,
    src: &Path,
    mut admit: impl FnMut(usize) -> Result<(), String>,
) -> Result<Vec<PathBuf>, String> {
    let size_error = |e: ace_artifacts::store::StoreError| e.to_string();
    let mut deps: Vec<PathBuf> = Vec::new();
    let mut text = 0usize;
    for line in stdout.lines() {
        let Some(path) = extract_drive_path(line) else {
            continue;
        };
        let p = Path::new(path);
        let array = metadata_array::<PathBuf>(deps.capacity()).map_err(size_error)?;
        let steady = metadata_size(&[array, text]).map_err(size_error)?;
        // The borrowed filesystem probe also creates native UTF-16 scratch.
        admit(
            metadata_size(&[steady, native_path_size(p).map_err(size_error)?])
                .map_err(size_error)?,
        )?;
        let exists = p.is_file();
        admit(steady)?;
        if !exists || p == src || deps.iter().any(|seen| seen == p) {
            continue;
        }
        let old = deps.capacity();
        let capacity = if deps.len() == old {
            old.checked_mul(2)
                .map(|n| n.max(4))
                .ok_or_else(|| "C dependency capacity overflow".to_owned())?
        } else {
            old
        };
        let next_text = metadata_size(&[text, p.as_os_str().len()]).map_err(size_error)?;
        let next_array = metadata_array::<PathBuf>(capacity).map_err(size_error)?;
        let old_array = if capacity != old { array } else { 0 };
        admit(metadata_size(&[next_text, next_array, old_array]).map_err(size_error)?)?;
        if capacity != old {
            deps.try_reserve_exact(capacity - deps.len())
                .map_err(|e| format!("C dependency allocation: {e}"))?;
        }
        deps.push(p.to_path_buf());
        text = next_text;
        admit(metadata_size(&[text, next_array]).map_err(size_error)?)?;
    }
    Ok(deps)
}

pub(crate) fn extract_drive_path(line: &str) -> Option<&str> {
    let bytes = line.as_bytes();
    for i in 0..bytes.len().saturating_sub(2) {
        if bytes[i].is_ascii_alphabetic()
            && bytes[i + 1] == b':'
            && (bytes[i + 2] == b'\\' || bytes[i + 2] == b'/')
        {
            let candidate = line[i..].trim().trim_matches('"');
            if candidate.contains('\\') || candidate.contains('/') {
                return Some(candidate);
            }
        }
    }
    None
}
