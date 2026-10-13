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
}
