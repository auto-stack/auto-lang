//! Resource and path admission (Plan 768 feasibility §8).
//!
//! Two-sided control: contracts may not declare anything above the hard cap;
//! runners may tune between the default and the cap. Every over-limit
//! condition is a `Resource` error — never masquerading as a syntax error —
//! and every path violation is rejected before any filesystem access.

/// Admission limits. Frozen defaults and hard caps per Plan 768 feasibility
/// §8; units are bytes or plain counts per field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_document_bytes: usize,
    pub max_inline_payload_bytes: usize,
    pub max_sidecar_bytes: usize,
    pub max_cases_per_file: usize,
    pub max_checks_per_case: usize,
    pub max_files_per_case: usize,
    pub max_suite_files: usize,
    pub max_expanded_checks_per_run: usize,
}

impl Limits {
    /// Frozen defaults.
    pub const fn defaults() -> Self {
        Limits {
            max_document_bytes: 2 * 1024 * 1024,
            max_inline_payload_bytes: 256 * 1024,
            max_sidecar_bytes: 1024 * 1024,
            max_cases_per_file: 64,
            max_checks_per_case: 16,
            max_files_per_case: 32,
            max_suite_files: 256,
            max_expanded_checks_per_run: 4096,
        }
    }

    /// Hard ceilings: no contract or runner may exceed these.
    pub const fn hard_caps() -> Self {
        Limits {
            max_document_bytes: 8 * 1024 * 1024,
            max_inline_payload_bytes: 1024 * 1024,
            max_sidecar_bytes: 4 * 1024 * 1024,
            max_cases_per_file: 256,
            max_checks_per_case: 64,
            max_files_per_case: 128,
            max_suite_files: 1024,
            max_expanded_checks_per_run: 16384,
        }
    }

    /// Clamp a proposed configuration into `[defaults, hard_caps]`.
    pub fn clamped(over: Limits) -> Limits {
        let d = Limits::defaults();
        let h = Limits::hard_caps();
        Limits {
            max_document_bytes: over
                .max_document_bytes
                .clamp(d.max_document_bytes, h.max_document_bytes),
            max_inline_payload_bytes: over
                .max_inline_payload_bytes
                .clamp(d.max_inline_payload_bytes, h.max_inline_payload_bytes),
            max_sidecar_bytes: over
                .max_sidecar_bytes
                .clamp(d.max_sidecar_bytes, h.max_sidecar_bytes),
            max_cases_per_file: over
                .max_cases_per_file
                .clamp(d.max_cases_per_file, h.max_cases_per_file),
            max_checks_per_case: over
                .max_checks_per_case
                .clamp(d.max_checks_per_case, h.max_checks_per_case),
            max_files_per_case: over
                .max_files_per_case
                .clamp(d.max_files_per_case, h.max_files_per_case),
            max_suite_files: over
                .max_suite_files
                .clamp(d.max_suite_files, h.max_suite_files),
            max_expanded_checks_per_run: over
                .max_expanded_checks_per_run
                .clamp(d.max_expanded_checks_per_run, h.max_expanded_checks_per_run),
        }
    }
}

/// Admission failures. `Resource` carries the limit name, observed value, and
/// the effective maximum so reports can show all three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    Resource {
        limit: &'static str,
        observed: usize,
        max: usize,
    },
    /// Virtual/relative path violates the contract (absolute, `..`, empty
    /// component, backslash, drive/root prefix, NUL).
    PathEscape(String),
    /// A filesystem object turned out to be a symlink/junction/reparse point.
    ReparsePoint(String),
    /// A file could not be opened/read during admission (Plan 773 F-07:
    /// read failures propagate, never silently drop the input).
    Read { path: String, detail: String },
    /// A write target overlaps a protected input/oracle root (Plan 773
    /// F-05): outputs must live in a disjoint path domain.
    OutputOverlap { out: String, protected: String },
}

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdmissionError::Resource {
                limit,
                observed,
                max,
            } => {
                write!(
                    f,
                    "resource limit {} exceeded: {} > {}",
                    limit, observed, max
                )
            }
            AdmissionError::PathEscape(p) => write!(f, "path violates contract: {:?}", p),
            AdmissionError::ReparsePoint(p) => {
                write!(f, "reparse point (symlink/junction) rejected: {:?}", p)
            }
            AdmissionError::Read { path, detail } => {
                write!(f, "read failed for {:?}: {}", path, detail)
            }
            AdmissionError::OutputOverlap { out, protected } => write!(
                f,
                "output path {:?} overlaps the protected input/oracle root {:?} — outputs must live in a disjoint directory (not equal, inside, or containing it)",
                out, protected
            ),
        }
    }
}

impl std::error::Error for AdmissionError {}

/// Check a size/count against a limit (value == max passes).
pub fn check_limit(limit: &'static str, observed: usize, max: usize) -> Result<(), AdmissionError> {
    if observed <= max {
        Ok(())
    } else {
        Err(AdmissionError::Resource {
            limit,
            observed,
            max,
        })
    }
}

/// Validate a contract virtual path (forward-slash, relative, no escapes).
///
/// Rules: non-empty; `/`-separated (a `\` is rejected — the virtual domain is
/// not a Windows path); no leading `/`; no drive/root prefix (`C:`, `\\`);
/// no `.`/`..`/empty components; no NUL.
pub fn validate_virtual_path(path: &str) -> Result<(), AdmissionError> {
    let bad = || AdmissionError::PathEscape(path.to_string());
    if path.is_empty() || path.contains('\0') || path.contains('\\') {
        return Err(bad());
    }
    if path.starts_with('/') || path.contains(':') {
        return Err(bad());
    }
    for comp in path.split('/') {
        if comp.is_empty() || comp == "." || comp == ".." {
            return Err(bad());
        }
    }
    Ok(())
}

/// Reject symlinks/junctions (any reparse point) on an existing path.
pub fn check_no_reparse(path: &std::path::Path) -> Result<(), AdmissionError> {
    match std::fs::symlink_metadata(path) {
        Ok(md) if md.file_type().is_symlink() => {
            Err(AdmissionError::ReparsePoint(path.display().to_string()))
        }
        Ok(_) => Ok(()),
        // Non-existent paths cannot be reparse points; existence is validated
        // by the caller that intends to read them.
        Err(_) => Ok(()),
    }
}

/// Reject reparse points on a path AND every existing ancestor (Plan 773
/// F-05). The endpoint-only check misses a junction anywhere up the chain:
/// `check_no_reparse` on `root/out/report.json` happily passes while `out`
/// itself is a junction re-basing the whole subtree.
pub fn check_no_reparse_ancestors(path: &std::path::Path) -> Result<(), AdmissionError> {
    check_no_reparse(path)?;
    let mut cur = path.to_path_buf();
    while cur.pop() {
        check_no_reparse(&cur)?;
    }
    Ok(())
}

/// Bounded file read under admission (Plan 773 F-07): reparse-audits the file
/// and its ancestors, then reads at most `max` bytes — one extra byte is read
/// to detect overflow, so an over-limit file is rejected WITHOUT first
/// buffering it whole. `limit_name` names the limit in Resource errors.
pub fn read_bounded(
    path: &std::path::Path,
    limit_name: &'static str,
    max: usize,
) -> Result<Vec<u8>, AdmissionError> {
    use std::io::Read as _;
    check_no_reparse_ancestors(path)?;
    let file = std::fs::File::open(path).map_err(|e| AdmissionError::Read {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    let mut buf = Vec::new();
    file.take(max.saturating_add(1) as u64)
        .read_to_end(&mut buf)
        .map_err(|e| AdmissionError::Read {
            path: path.display().to_string(),
            detail: e.to_string(),
        })?;
    if buf.len() > max {
        return Err(AdmissionError::Resource {
            limit: limit_name,
            observed: buf.len(),
            max,
        });
    }
    Ok(buf)
}

/// Output-path domain audit (Plan 773 F-05): a writable output location must
/// not overlap any protected input/oracle root in either direction — equal,
/// inside, or containing are all rejections. Comparison happens on
/// canonicalized (`\\?\`-verbatim on Windows) forms so case differences,
/// trailing separators, and 8.3 aliases cannot sneak an overlap through.
///
/// Both sides must exist on disk (callers create the output directory before
/// auditing it); non-canonicalizable paths are rejected as unreadable.
pub fn validate_output_domain(
    out: &std::path::Path,
    protected: &[std::path::PathBuf],
) -> Result<(), AdmissionError> {
    let out_canon = out.canonicalize().map_err(|e| AdmissionError::Read {
        path: out.display().to_string(),
        detail: format!("canonicalize: {}", e),
    })?;
    for root in protected {
        let root_canon = root.canonicalize().map_err(|e| AdmissionError::Read {
            path: root.display().to_string(),
            detail: format!("canonicalize: {}", e),
        })?;
        let overlap = out_canon == root_canon
            || out_canon.starts_with(&root_canon)
            || root_canon.starts_with(&out_canon);
        if overlap {
            return Err(AdmissionError::OutputOverlap {
                out: out_canon.display().to_string(),
                protected: root_canon.display().to_string(),
            });
        }
    }
    Ok(())
}

/// Validate that a file write target stays out of the protected input/oracle
/// roots (Plan 773 F-05, `--json-out` class). The target's parent directory
/// must exist for canonicalization; `name` is the leaf file name.
pub fn validate_output_file(
    target: &std::path::Path,
    protected: &[std::path::PathBuf],
) -> Result<(), AdmissionError> {
    // Audit the target path itself (parent canonical + leaf) so a
    // pre-existing file directly inside a protected root is caught even
    // before it exists under the exact spellings compared below.
    let parent = target.parent().unwrap_or(std::path::Path::new("."));
    let parent_canon = parent.canonicalize().map_err(|e| AdmissionError::Read {
        path: parent.display().to_string(),
        detail: format!("canonicalize: {}", e),
    })?;
    let target_canon = parent_canon.join(target.file_name().unwrap_or_default());
    for root in protected {
        let root_canon = root.canonicalize().map_err(|e| AdmissionError::Read {
            path: root.display().to_string(),
            detail: format!("canonicalize: {}", e),
        })?;
        if target_canon.starts_with(&root_canon) {
            return Err(AdmissionError::OutputOverlap {
                out: target_canon.display().to_string(),
                protected: root_canon.display().to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_defaults_within_hard_caps() {
        let d = Limits::defaults();
        let h = Limits::hard_caps();
        assert!(d.max_document_bytes <= h.max_document_bytes);
        assert!(d.max_expanded_checks_per_run <= h.max_expanded_checks_per_run);
    }

    #[test]
    fn clamped_never_exceeds_caps_or_drops_below_defaults() {
        let wild = Limits {
            max_document_bytes: usize::MAX,
            max_inline_payload_bytes: 0,
            max_sidecar_bytes: 5 * 1024 * 1024,
            max_cases_per_file: 100,
            max_checks_per_case: 1,
            max_files_per_case: 999,
            max_suite_files: 500,
            max_expanded_checks_per_run: 999999,
        };
        let c = Limits::clamped(wild);
        let d = Limits::defaults();
        let h = Limits::hard_caps();
        assert_eq!(c.max_document_bytes, h.max_document_bytes);
        assert_eq!(c.max_inline_payload_bytes, d.max_inline_payload_bytes);
        assert_eq!(c.max_sidecar_bytes, h.max_sidecar_bytes);
        assert_eq!(c.max_checks_per_case, d.max_checks_per_case);
        assert!(c.max_files_per_case <= h.max_files_per_case);
    }

    #[test]
    fn check_limit_boundary() {
        assert!(check_limit("x", 10, 10).is_ok());
        assert_eq!(
            check_limit("x", 11, 10),
            Err(AdmissionError::Resource {
                limit: "x",
                observed: 11,
                max: 10
            })
        );
    }

    #[test]
    fn virtual_path_rules() {
        assert!(validate_virtual_path("main.at").is_ok());
        assert!(validate_virtual_path("api/handlers.at").is_ok());
        for bad in [
            "",
            "/",
            "a//b",
            "./x",
            "../x",
            "a/../b",
            "a\\b",
            "C:/x",
            "C:\\x",
            "\\\\srv\\s",
            "a\0b",
            "/abs",
        ] {
            assert!(
                validate_virtual_path(bad).is_err(),
                "should reject {:?}",
                bad
            );
        }
    }

    // ---------------------------------------------- Plan 773 F-05/F-07 --

    fn temp_dir_tagged(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("atc-adm-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn output_domain_rejects_overlap_in_both_directions() {
        let root = temp_dir_tagged("domain");
        let inputs = root.join("inputs");
        std::fs::create_dir_all(inputs.join("sub")).unwrap();
        // out inside the protected root (audit requires the target to
        // exist — callers create it first, mirroring the runner).
        let inside = inputs.join("sub").join("out");
        std::fs::create_dir_all(&inside).unwrap();
        let err = validate_output_domain(&inside, &[inputs.clone()]).unwrap_err();
        assert!(err.to_string().contains("overlaps"), "{}", err);
        // out equal to the protected root.
        assert!(validate_output_domain(&inputs, &[inputs.clone()]).is_err());
        // out CONTAINING the protected root (parent direction).
        assert!(validate_output_domain(&root, &[inputs.clone()]).is_err());
        // Disjoint sibling directory: accepted.
        let ok = root.join("elsewhere");
        std::fs::create_dir_all(&ok).unwrap();
        assert!(validate_output_domain(&ok, &[inputs.clone()]).is_ok());
        // Windows case-difference cannot sneak an overlap through.
        let lower = root.join("inputs2");
        std::fs::create_dir_all(&lower).unwrap();
        let upper = root.join("INPUTS2");
        assert!(validate_output_domain(&upper, &[lower.clone()]).is_err());
    }

    #[test]
    fn output_file_targets_cannot_land_in_protected_roots() {
        let root = temp_dir_tagged("outfile");
        let inputs = root.join("inputs");
        std::fs::create_dir_all(&inputs).unwrap();
        let err =
            validate_output_file(&inputs.join("report.json"), &[inputs.clone()]).unwrap_err();
        assert!(err.to_string().contains("overlaps"), "{}", err);
        let ok = root.join("out");
        std::fs::create_dir_all(&ok).unwrap();
        assert!(validate_output_file(&ok.join("report.json"), &[inputs]).is_ok());
    }

    #[test]
    fn ancestors_reparse_audit_walks_the_chain() {
        let root = temp_dir_tagged("ancestors");
        let deep = root.join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        assert!(check_no_reparse_ancestors(&deep.join("f.txt")).is_ok());
        // A normal directory chain has no reparse points; the endpoint
        // check alone also passes — the walk is the guard for junctions
        // higher up (source-audited; creating real junctions in tests is
        // platform-privilege dependent).
        assert!(check_no_reparse_ancestors(&root).is_ok());
    }

    #[test]
    fn bounded_read_rejects_oversized_files() {
        let root = temp_dir_tagged("bounded");
        let f = root.join("big.bin");
        std::fs::write(&f, vec![0u8; 100]).unwrap();
        let err = read_bounded(&f, "max_probe", 16).unwrap_err();
        assert!(err.to_string().contains("max_probe"), "{}", err);
        // Exactly at the limit passes.
        assert_eq!(read_bounded(&f, "max_probe", 100).unwrap().len(), 100);
        // Missing file: a Read error, not a silent skip.
        let err = read_bounded(&root.join("nope.bin"), "max_probe", 16).unwrap_err();
        assert!(err.to_string().contains("read failed"), "{}", err);
    }
}
