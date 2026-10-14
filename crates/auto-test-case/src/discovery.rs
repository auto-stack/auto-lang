//! Suite loading pipeline: suite file → include list → case documents →
//! case specs, with per-file errors kept (never silently dropped) and
//! case-id uniqueness enforced across the whole selection set.

use std::path::{Path, PathBuf};

use crate::admission::{check_limit, read_bounded, Limits};
use crate::atom;
use crate::contract::{decode_case, CaseSpec};
use crate::document::CaseDocument;
use crate::sha256::{hex, sha256};
use crate::suite::{decode_suite, selected_files, SuiteSpec};

pub const CASE_FENCE_LANGUAGE: &str = "test-case";
pub const SUITE_FENCE_LANGUAGE: &str = "test-suite";

#[derive(Debug, Clone)]
pub struct CaseFile {
    pub path: PathBuf,
    pub sha256: String,
    pub doc: CaseDocument,
    pub specs: Vec<CaseSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    /// Originating file (suite or case path).
    pub file: PathBuf,
    pub detail: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file.display(), self.detail)
    }
}

impl std::error::Error for LoadError {}

#[derive(Debug, Clone)]
pub struct LoadedSuite {
    pub suite: SuiteSpec,
    pub suite_path: PathBuf,
    pub suite_sha256: String,
    /// Directory of the suite file — include base.
    pub root: PathBuf,
    /// Resolved fixture root (`suite.fixture_root` joined onto `root`):
    /// the single base for `file:` payload and expected-artifact references.
    pub fixture_base: PathBuf,
    pub cases: Vec<CaseFile>,
    /// Non-fatal per-file load errors (surfaced by lint/list; an empty
    /// selection plus errors is a hard failure for the caller).
    pub errors: Vec<LoadError>,
}

fn read_file_limited(path: &Path, limits: &Limits) -> Result<Vec<u8>, String> {
    // Bounded read (Plan 773 F-07): over-limit documents are rejected
    // without buffering them whole; reparse points on the file or any
    // ancestor are rejected before the open.
    read_bounded(path, "max_document_bytes", limits.max_document_bytes)
        .map_err(|e| e.to_string())
}

fn contract_from_fence(payload: &[u8], language: &str) -> Result<atom::CVal, String> {
    let text = std::str::from_utf8(payload)
        .map_err(|_| format!("{} fence payload is not valid UTF-8", language))?;
    atom::parse_contract(text).map_err(|e| format!("{} fence contract: {}", language, e))
}

/// Load and decode a suite plus every selected case file.
///
/// Hard errors (suite unreadable/undecodable, zero cases discovered) return
/// `Err`; per-case problems land in `errors` with the file attached so the
/// caller can report a complete discovery inventory.
pub fn load_suite(suite_path: &Path, limits: &Limits) -> Result<LoadedSuite, LoadError> {
    let root = suite_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let bytes = read_file_limited(suite_path, limits).map_err(|detail| LoadError {
        file: suite_path.to_path_buf(),
        detail,
    })?;
    let suite_sha = hex(&sha256(&bytes));
    let doc = CaseDocument::parse_with(&bytes, limits).map_err(|e| LoadError {
        file: suite_path.to_path_buf(),
        detail: format!("suite document: {}", e),
    })?;

    let suite_payloads: Vec<&crate::document::DocumentFence> = doc
        .fences()
        .iter()
        .filter(|f| f.language == SUITE_FENCE_LANGUAGE)
        .collect();
    if suite_payloads.len() != 1 {
        return Err(LoadError {
            file: suite_path.to_path_buf(),
            detail: format!(
                "expected exactly one `{}` fence, found {}",
                SUITE_FENCE_LANGUAGE,
                suite_payloads.len()
            ),
        });
    }
    let suite_value =
        contract_from_fence(doc.payload_bytes(suite_payloads[0]), SUITE_FENCE_LANGUAGE).map_err(
            |detail| LoadError {
                file: suite_path.to_path_buf(),
                detail,
            },
        )?;
    let suite = decode_suite(&suite_value).map_err(|e| LoadError {
        file: suite_path.to_path_buf(),
        detail: format!("suite contract: {}", e),
    })?;

    check_limit(
        "max_suite_files",
        selected_files(&suite).len(),
        limits.max_suite_files,
    )
    .map_err(|e| LoadError {
        file: suite_path.to_path_buf(),
        detail: e.to_string(),
    })?;

    let mut cases = Vec::new();
    let mut errors = Vec::new();
    let mut seen_case_ids: std::collections::BTreeMap<String, PathBuf> =
        std::collections::BTreeMap::new();

    for rel in selected_files(&suite) {
        let case_path = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        match read_file_limited(&case_path, limits) {
            Err(detail) => errors.push(LoadError {
                file: case_path,
                detail,
            }),
            Ok(case_bytes) => match CaseDocument::parse_with(&case_bytes, limits) {
                Err(e) => errors.push(LoadError {
                    file: case_path.clone(),
                    detail: format!("case document: {}", e),
                }),
                Ok(case_doc) => {
                    let sha = hex(&sha256(&case_bytes));
                    let mut specs = Vec::new();
                    let contract_fences: Vec<&crate::document::DocumentFence> = case_doc
                        .fences()
                        .iter()
                        .filter(|f| f.language == CASE_FENCE_LANGUAGE)
                        .collect();
                    if contract_fences.is_empty() {
                        errors.push(LoadError {
                            file: case_path,
                            detail: format!(
                                "no `{}` fence — nothing to import",
                                CASE_FENCE_LANGUAGE
                            ),
                        });
                        continue;
                    }
                    for fence in contract_fences {
                        let payload = case_doc.payload_bytes(fence);
                        match contract_from_fence(payload, CASE_FENCE_LANGUAGE)
                            .and_then(|v| decode_case(&v, limits).map_err(|e| e.to_string()))
                        {
                            Ok(spec) => specs.push(spec),
                            Err(detail) => errors.push(LoadError {
                                file: case_path.clone(),
                                detail: format!(
                                    "case fence at line {}: {}",
                                    fence.open_line, detail
                                ),
                            }),
                        }
                    }
                    if let Err(e) =
                        check_limit("max_cases_per_file", specs.len(), limits.max_cases_per_file)
                    {
                        errors.push(LoadError {
                            file: case_path.clone(),
                            detail: e.to_string(),
                        });
                    }
                    for spec in specs.iter() {
                        if let Some(prev) = seen_case_ids.get(&spec.id) {
                            errors.push(LoadError {
                                file: case_path.clone(),
                                detail: format!(
                                    "duplicate case id {:?} (also declared in {})",
                                    spec.id,
                                    prev.display()
                                ),
                            });
                        } else {
                            seen_case_ids.insert(spec.id.clone(), case_path.clone());
                        }
                    }
                    cases.push(CaseFile {
                        path: case_path,
                        sha256: sha,
                        doc: case_doc,
                        specs,
                    });
                }
            },
        }
    }

    let total_cases: usize = cases.iter().map(|c| c.specs.len()).sum();
    if total_cases == 0 {
        let mut detail = format!(
            "zero cases discovered from {} include(s)",
            suite.include.len()
        );
        if let Some(first) = errors.first() {
            detail.push_str(&format!(" (first error: {})", first));
        }
        return Err(LoadError {
            file: suite_path.to_path_buf(),
            detail,
        });
    }

    // Plan 773 F-07: the whole-run expansion limit is enforced at load —
    // every entry (lint, list, run) sees it, not just execution. Individual
    // file/case/check limits may all be legal while the total is not.
    let total_checks: usize = cases
        .iter()
        .map(|c| c.specs.iter().map(|s| s.checks.len()).sum::<usize>())
        .sum();
    check_limit(
        "max_expanded_checks_per_run",
        total_checks,
        limits.max_expanded_checks_per_run,
    )
    .map_err(|e| LoadError {
        file: suite_path.to_path_buf(),
        detail: e.to_string(),
    })?;

    let fixture_base = root.join(
        suite
            .fixture_root
            .replace('/', std::path::MAIN_SEPARATOR_STR),
    );
    let fixture_base = fixture_base.canonicalize().unwrap_or(fixture_base);
    Ok(LoadedSuite {
        suite,
        suite_path: suite_path.to_path_buf(),
        suite_sha256: suite_sha,
        root,
        fixture_base,
        cases,
        errors,
    })
}
