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
pub(crate) fn hash_file(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = match file.read(&mut buffer) {
            Ok(count) => count,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().to_hex().to_string())
}

pub(crate) fn source_identity(
    path: &std::path::Path,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CPathText, StoreError> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
    let size = metadata_size(&[name.len(), 1, 64])?;
    let mut allocation = reserve_c_metadata(resources, metadata_size(&[size, 64])?)?;
    let digest =
        hash_file(path).map_err(|e| StoreError::Internal(format!("{}: {e}", path.display())))?;
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
