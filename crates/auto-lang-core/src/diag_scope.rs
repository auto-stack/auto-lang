//! PLAN-1066 T-07[A] (A-NOWRITE): diagnostic outputs on normal test paths
//! must never write into the author tree.
//!
//! The legacy runners used to drop `.wrong.*` siblings next to the corpus
//! oracle files. Those writes are author-tree mutations: a red verdict does
//! not need a sidecar to be red. Writers on the paths PLAN-1066/A actually
//! executes route through [`write_scoped`]:
//!
//! * default: the payload is dropped (the verdict itself is the diagnostic);
//! * with `AUTO_TEST_DIAG_SCOPE` pointing at an existing directory, the
//!   payload is mirrored under `<scope>/plan-1066-diag/<mirror_rel>` so
//!   debugging a failure never depends on polluting the corpus.
//!
//! Author-side generation is an explicit author action owned by the
//! T-07[B] proposal flow; nothing here ever writes into the repo.

use std::io;
use std::path::PathBuf;

/// Write a diagnostic payload without touching the author tree.
///
/// `mirror_rel` is the repo-relative path the legacy writer would have used
/// (forward slashes). Returns the write target when the scope is configured,
/// `None` when the payload was intentionally dropped.
pub fn write_scoped(mirror_rel: &str, bytes: &[u8]) -> io::Result<Option<PathBuf>> {
    let Ok(scope) = std::env::var("AUTO_TEST_DIAG_SCOPE") else {
        return Ok(None);
    };
    if scope.trim().is_empty() {
        return Ok(None);
    }
    let scope = PathBuf::from(scope);
    if !scope.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!(
                "AUTO_TEST_DIAG_SCOPE {:?} is not an existing directory \
                 (author-tree writes are forbidden; create the scope first)",
                scope
            ),
        ));
    }
    let target = scope.join("plan-1066-diag").join(mirror_rel);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&target, bytes)?;
    Ok(Some(target))
}
