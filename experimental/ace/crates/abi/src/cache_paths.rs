//! Private C cache path metadata: shared ownership and pre-allocation bounds.
use ace_artifacts::store::StoreError;

pub(crate) fn reserve_c_metadata(
    resources: Option<&ace_source_query::resources::Resources>,
    bytes: usize,
) -> Result<Option<ace_source_query::resources::Reservation>, StoreError> {
    resources
        .map(|r| r.reserve(ace_source_query::resources::Charge::index(bytes)))
        .transpose()
        .map_err(|e| StoreError::ResourceExhausted(format!("C cache metadata: {e}")))
}
pub(crate) fn resize_c_metadata(
    allocation: &mut Option<ace_source_query::resources::Reservation>,
    bytes: usize,
) -> Result<(), StoreError> {
    if let Some(allocation) = allocation {
        allocation
            .resize(ace_source_query::resources::Charge::index(bytes))
            .map_err(|e| StoreError::ResourceExhausted(format!("C cache metadata: {e}")))?;
    }
    Ok(())
}
pub(crate) fn metadata_size(parts: &[usize]) -> Result<usize, StoreError> {
    parts
        .iter()
        .try_fold(0usize, |n, p| n.checked_add(*p))
        .ok_or_else(|| StoreError::ResourceExhausted("C cache metadata size overflow".into()))
}
pub(crate) fn metadata_array<T>(count: usize) -> Result<usize, StoreError> {
    count
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| StoreError::ResourceExhausted("C cache metadata array overflow".into()))
}
pub(crate) fn metadata_vec<T>(count: usize) -> Result<Vec<T>, StoreError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|e| StoreError::ResourceExhausted(format!("C cache metadata allocation: {e}")))?;
    Ok(values)
}

pub(crate) struct CPathText {
    pub(crate) value: String,
    pub(crate) _allocation: Option<ace_source_query::resources::Reservation>,
}

/// Lexical relative path FROM a directory TO a file (leading `..`
/// segments allowed; same-drive only). The common-prefix walk borrows both
/// inputs, and only its final relative spelling requires owned storage.
pub(crate) fn rel_spec(
    from_dir: &str,
    to_file: &str,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<Option<CPathText>, StoreError> {
    let mut from = split_norm(from_dir).peekable();
    let mut to = split_norm(to_file).peekable();
    let (Some(first_from), Some(first_to)) = (from.next(), to.next()) else {
        return Ok(None);
    };
    if first_from.len() < 2 || first_to.len() < 2 || !eq_ci(first_from, first_to) {
        return Ok(None);
    }
    while from.peek().zip(to.peek()).is_some_and(|(a, b)| eq_ci(a, b)) {
        from.next();
        to.next();
    }
    let parents = from.count();
    let count = metadata_size(&[parents, to.clone().count()])?;
    if count == 0 {
        return Ok(None);
    }
    let suffix = to
        .clone()
        .try_fold(0usize, |n, s| metadata_size(&[n, s.len()]))?;
    let parent_bytes = parents
        .checked_mul(2)
        .ok_or_else(|| StoreError::ResourceExhausted("C cache relative path overflow".into()))?;
    let size = metadata_size(&[parent_bytes, suffix, count - 1])?;
    let allocation = reserve_c_metadata(resources, size)?;
    let mut value = String::with_capacity(size);
    for part in std::iter::repeat_n("..", parents).chain(to) {
        if !value.is_empty() {
            value.push('/');
        }
        value.push_str(part);
    }
    Ok(Some(CPathText {
        value,
        _allocation: allocation,
    }))
}

/// Join and normalize with a bounded borrowed segment stack. `..` never
/// removes the drive root, matching the original lexical algorithm.
pub(crate) fn join_lex(
    dir: &str,
    spec: &str,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CPathText, StoreError> {
    let extra = spec
        .split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".");
    let capacity = metadata_size(&[split_norm(dir).count(), extra.clone().count()])?;
    let array = metadata_array::<&str>(capacity)?;
    let mut allocation = reserve_c_metadata(resources, array)?;
    let mut parts = metadata_vec(capacity)?;
    parts.extend(split_norm(dir));
    for part in extra {
        if part == ".." {
            if parts.len() > 1 {
                parts.pop();
            }
        } else {
            parts.push(part);
        }
    }
    let text = parts
        .iter()
        .try_fold(0usize, |n, s| metadata_size(&[n, s.len()]))?;
    let size = metadata_size(&[text, parts.len().saturating_sub(1)])?;
    resize_c_metadata(&mut allocation, metadata_size(&[array, size])?)?;
    let mut value = String::with_capacity(size);
    for (i, part) in parts.iter().enumerate() {
        if i != 0 {
            value.push('\\');
        }
        value.push_str(part);
    }
    drop(parts);
    resize_c_metadata(&mut allocation, size)?;
    Ok(CPathText {
        value,
        _allocation: allocation,
    })
}

pub(crate) fn parent_of(path: &str) -> &str {
    // A parent of a UTF-8 lexical String is another UTF-8 slice. Borrow it
    // until the destination directory vector admits its own copy.
    std::path::Path::new(path)
        .parent()
        .and_then(std::path::Path::to_str)
        .unwrap_or(path)
}

pub(crate) fn split_norm(p: &str) -> impl Iterator<Item = &str> + Clone {
    p.strip_prefix("\\\\?\\")
        .unwrap_or(p)
        .split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
}

pub(crate) struct CPathList {
    pub(crate) values: Vec<String>,
    allocation: Option<ace_source_query::resources::Reservation>,
    text: usize,
}
impl CPathList {
    pub(crate) fn new(
        resources: Option<&ace_source_query::resources::Resources>,
    ) -> Result<Self, StoreError> {
        Ok(Self {
            values: Vec::new(),
            allocation: reserve_c_metadata(resources, 0)?,
            text: 0,
        })
    }
    pub(crate) fn contains(&self, value: &str) -> bool {
        self.values.iter().any(|s| s == value)
    }
    pub(crate) fn push(&mut self, value: &str) -> Result<(), StoreError> {
        let old = self.values.capacity();
        let capacity = if self.values.len() == old {
            old.checked_mul(2)
                .map(|n| n.max(4))
                .ok_or_else(|| StoreError::ResourceExhausted("C cache path list overflow".into()))?
        } else {
            old
        };
        let text = metadata_size(&[self.text, value.len()])?;
        let array = metadata_array::<String>(capacity)?;
        let transient = if capacity != old {
            metadata_array::<String>(old)?
        } else {
            0
        };
        resize_c_metadata(
            &mut self.allocation,
            metadata_size(&[text, array, transient])?,
        )?;
        if capacity != old {
            self.values
                .try_reserve_exact(capacity - self.values.len())
                .map_err(|e| {
                    StoreError::ResourceExhausted(format!("C cache path list allocation: {e}"))
                })?;
        }
        self.values.push(value.to_owned());
        self.text = text;
        resize_c_metadata(&mut self.allocation, metadata_size(&[text, array])?)?;
        Ok(())
    }
    pub(crate) fn sort_dedup(&mut self) -> Result<(), StoreError> {
        self.values.sort_unstable();
        self.values.dedup();
        self.text = self
            .values
            .iter()
            .try_fold(0usize, |n, s| metadata_size(&[n, s.capacity()]))?;
        resize_c_metadata(
            &mut self.allocation,
            metadata_size(&[self.text, metadata_array::<String>(self.values.capacity())?])?,
        )
    }
}

fn eq_ci(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Streamed source/header hashing has one owned 64-byte result, never a
/// whole input copy. The caller admits that result in its actual live owner.
pub(crate) fn hash_file(
    path: &std::path::Path,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<String, StoreError> {
    let io_error = |e: std::io::Error| StoreError::Internal(e.to_string());
    let file = {
        let _native = reserve_native_path(path, resources)?;
        std::fs::File::open(path).map_err(io_error)?
    };
    crate::semref::blake3_reader(file).map_err(io_error)
}

pub(crate) fn source_identity(
    path: &std::path::Path,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CPathText, StoreError> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let size = metadata_size(&[name.len(), 1, 64])?;
    let mut allocation = reserve_c_metadata(resources, metadata_size(&[size, 64])?)?;
    let digest = hash_file(path, resources).map_err(|e| match e {
        StoreError::Internal(message) => {
            StoreError::Internal(format!("{}: {message}", path.display()))
        }
        other => other,
    })?;
    let mut value = String::with_capacity(size);
    value.push_str(name);
    value.push('|');
    value.push_str(&digest);
    drop(digest);
    resize_c_metadata(&mut allocation, size)?;
    Ok(CPathText {
        value,
        _allocation: allocation,
    })
}

pub(crate) fn toolchain_text(
    executable: &std::path::Path,
    version: &str,
    digest: &str,
    flags: &[&str],
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CPathText, StoreError> {
    let flags_size = flags
        .iter()
        .try_fold(flags.len().saturating_sub(1), |n, f| {
            metadata_size(&[n, f.len()])
        })?;
    let size = metadata_size(&[
        "cl:".len(),
        executable.as_os_str().len(),
        "|ver:".len(),
        version.len(),
        "|hash:".len(),
        digest.len(),
        "|flags:".len(),
        flags_size,
    ])?;
    let allocation = reserve_c_metadata(resources, size)?;
    let mut value = String::with_capacity(size);
    use std::fmt::Write as _;
    write!(
        &mut value,
        "cl:{}|ver:{version}|hash:{digest}|flags:",
        executable.display()
    )
    .map_err(|e| StoreError::Internal(format!("C tool identity formatting: {e}")))?;
    for (i, flag) in flags.iter().enumerate() {
        if i != 0 {
            value.push(' ');
        }
        value.push_str(flag);
    }
    Ok(CPathText {
        value,
        _allocation: allocation,
    })
}

pub(crate) fn discovery_needle(
    source: &str,
    target: &str,
    toolchain: &str,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CPathText, StoreError> {
    let inputs = [
        ("%source=", source),
        (",target=", target),
        (",toolchain=", toolchain),
    ];
    let size = inputs.iter().try_fold(1usize, |n, (label, value)| {
        let escapes = value
            .bytes()
            .filter(|b| matches!(b, b'\\' | b'%' | b'_'))
            .count();
        metadata_size(&[n, label.len(), value.len(), escapes])
    })?;
    let allocation = reserve_c_metadata(resources, size)?;
    let mut value = String::with_capacity(size);
    for (label, input) in inputs {
        value.push_str(label);
        for ch in input.chars() {
            if matches!(ch, '\\' | '%' | '_') {
                value.push('\\');
            }
            value.push(ch);
        }
    }
    value.push('%');
    Ok(CPathText {
        value,
        _allocation: allocation,
    })
}

/// Bound std's Windows native-path conversion before calling it. Verbatim
/// paths and short absolute paths use only to_u16s' exact input buffer;
/// longer absolute paths also normalize with bounded old/new UTF-16 copies.
/// Relative spellings can expand against a long CWD, so reserve its Windows
/// namespace bound as temporary scratch, not as a cap on the returned path.
pub(crate) fn reserve_native_path(
    path: &std::path::Path,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<Option<ace_source_query::resources::Reservation>, StoreError> {
    let bytes = path.as_os_str().as_encoded_bytes();
    let short_absolute = bytes.len() < 247
        && (path.is_absolute()
            || (bytes.len() == 2
                && bytes[1] == b':'
                && bytes[0] != std::path::MAIN_SEPARATOR as u8
                && bytes[0] != b'/'));
    let size = if bytes.starts_with(br"\\?\") || bytes.starts_with(br"\??\") || short_absolute {
        metadata_array::<u16>(metadata_size(&[bytes.len(), 1])?)?
    } else {
        let input = bytes
            .len()
            .checked_mul(8)
            .ok_or_else(|| StoreError::ResourceExhausted("C cache native path overflow".into()))?;
        metadata_size(&[input, if path.is_absolute() { 4096 } else { 16 * 32768 }])?
    };
    reserve_c_metadata(resources, size)
}

pub(crate) fn display_path(
    path: &std::path::Path,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CPathText, StoreError> {
    let size = path.as_os_str().len();
    let allocation = reserve_c_metadata(resources, size)?;
    let mut value = String::with_capacity(size);
    use std::fmt::Write as _;
    write!(&mut value, "{}", path.display())
        .map_err(|e| StoreError::Internal(format!("C path display: {e}")))?;
    Ok(CPathText {
        value,
        _allocation: allocation,
    })
}

/// Same access/share/normalization semantics as Windows std::canonicalize.
/// Query the handle's required final-path length first so a short junction
/// spelling never authorizes an uncharged, much longer target allocation.
pub(crate) fn canonical_text(
    path: &std::path::Path,
    resources: Option<&ace_source_query::resources::Resources>,
    context: &str,
) -> Result<CPathText, StoreError> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        GetFinalPathNameByHandleW, FILE_FLAG_BACKUP_SEMANTICS, VOLUME_NAME_DOS,
    };
    let io_error = |e| StoreError::Internal(format!("{context}: {e}"));
    let file = {
        let _native = reserve_native_path(path, resources)?;
        std::fs::OpenOptions::new()
            .access_mode(0)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
            .map_err(io_error)?
    };
    let mut required = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle(),
            std::ptr::null_mut(),
            0,
            VOLUME_NAME_DOS,
        )
    };
    if required == 0 {
        return Err(io_error(std::io::Error::last_os_error()));
    }
    let mut allocation = reserve_c_metadata(resources, 0)?;
    let mut wide: Vec<u16> = Vec::new();
    loop {
        let count = required as usize;
        let old = metadata_array::<u16>(wide.capacity())?;
        let next = metadata_array::<u16>(count)?;
        resize_c_metadata(&mut allocation, metadata_size(&[old, next])?)?;
        if count > wide.capacity() {
            wide.try_reserve_exact(count - wide.len()).map_err(|e| {
                StoreError::ResourceExhausted(format!("C final path allocation: {e}"))
            })?;
        }
        wide.resize(count, 0);
        resize_c_metadata(&mut allocation, metadata_array::<u16>(wide.capacity())?)?;
        let length = unsafe {
            GetFinalPathNameByHandleW(
                file.as_raw_handle(),
                wide.as_mut_ptr(),
                required,
                VOLUME_NAME_DOS,
            )
        };
        if length == 0 {
            return Err(io_error(std::io::Error::last_os_error()));
        }
        if length < required {
            wide.truncate(length as usize);
            break;
        }
        required = length
            .checked_add(1)
            .ok_or_else(|| StoreError::ResourceExhausted("C final path length overflow".into()))?;
    }
    let size = char::decode_utf16(wide.iter().copied()).try_fold(0usize, |n, c| {
        metadata_size(&[n, c.unwrap_or(char::REPLACEMENT_CHARACTER).len_utf8()])
    })?;
    resize_c_metadata(
        &mut allocation,
        metadata_size(&[metadata_array::<u16>(wide.capacity())?, size])?,
    )?;
    let mut value = String::with_capacity(size);
    for c in char::decode_utf16(wide.iter().copied()) {
        value.push(c.unwrap_or(char::REPLACEMENT_CHARACTER));
    }
    drop(wide);
    drop(file);
    resize_c_metadata(&mut allocation, size)?;
    Ok(CPathText {
        value,
        _allocation: allocation,
    })
}

pub(crate) fn artifact_path(
    path: &std::path::Path,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CPathText, StoreError> {
    let parent = path
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let directory = match canonical_text(parent, resources, "canonicalize artifact dir") {
        Ok(value) => value,
        Err(StoreError::ResourceExhausted(message)) => {
            return Err(StoreError::ResourceExhausted(message))
        }
        Err(_) => display_path(parent, resources)?,
    };
    let dir = directory
        .value
        .strip_prefix("\\\\?\\")
        .unwrap_or(&directory.value);
    let name = path.file_name().unwrap_or_default();
    let size = metadata_size(&[dir.len(), 1, name.len()])?;
    let allocation = reserve_c_metadata(resources, size)?;
    let mut value = String::with_capacity(size);
    for ch in dir.chars() {
        value.push(if ch == '/' { '\\' } else { ch });
    }
    value.push('\\');
    use std::fmt::Write as _;
    write!(&mut value, "{}", name.display())
        .map_err(|e| StoreError::Internal(format!("C artifact path: {e}")))?;
    Ok(CPathText {
        value,
        _allocation: allocation,
    })
}
