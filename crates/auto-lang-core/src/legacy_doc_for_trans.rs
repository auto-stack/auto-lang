//! PLAN-1062 T-14 bridge: the trans-pipeline legacy helpers read their
//! migration documents through the shared auto-test-case reader.
//!
//! PLAN-1066 T-03[B]: the migration document is also the AUTHORITY for
//! discovery and execution of the N corpus families (vm/a2r/a2c/a2ts/
//! cookbook): the selected set comes from `<stem>.case.ad` presence, the
//! virtual source path and the oracle channels come from the decoded
//! document, and the original `.at`/`.as`/`.expected.*` sidecars are frozen
//! history — never read on the normal path and never a fallback.
use std::path::{Path, PathBuf};

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorCase};
use auto_test_case::contract::{ArtifactPayload, PayloadSpec};
use auto_test_case::document::CaseDocument;

pub struct LegacyDoc {
    pub case: AuthorCase,
}

/// The migration document path of one corpus entry.
pub fn doc_path(dir: &Path, stem: &str) -> PathBuf {
    dir.join(format!("{stem}.case.ad"))
}

/// The canonical entry stem of a case directory (`001_hello` → `hello`).
pub fn case_stem(dir: &Path) -> Option<String> {
    dir.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.split_once('_'))
        .map(|(_, stem)| stem.to_string())
}

pub fn load_case(dir: &Path, stem: &str) -> Result<LegacyDoc, String> {
    let doc_path = doc_path(dir, stem);
    let bytes = std::fs::read(&doc_path).map_err(|e| {
        format!(
            "legacy doc {} missing/unreadable: {e} (migration doc required; sidecar fallback is forbidden)",
            doc_path.display()
        )
    })?;
    let text = std::str::from_utf8(&bytes).map_err(|e| format!("{}: {e}", doc_path.display()))?;
    let doc =
        CaseDocument::parse(text.as_bytes()).map_err(|e| format!("{}: {e}", doc_path.display()))?;
    let rel = format!(
        "{}/{}.case.ad",
        dir.file_name()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_default(),
        stem
    );
    let mut cases = decode_document(&doc, &rel, &Limits::defaults())
        .map_err(|e| format!("{}: {}", doc_path.display(), e.message))?;
    if cases.len() != 1 {
        return Err(format!(
            "{}: expected exactly one case, found {}",
            doc_path.display(),
            cases.len()
        ));
    }
    Ok(LegacyDoc {
        case: cases.remove(0),
    })
}

impl LegacyDoc {
    pub fn source_text(&self) -> &str {
        match &self.case.spec.source.payload {
            PayloadSpec::Text(t) => t,
            other => unreachable!("legacy docs inline their source: {other:?}"),
        }
    }

    /// The declared virtual source path (`hello.at` / `script.as`).
    pub fn source_path(&self) -> &str {
        &self.case.spec.source.path
    }

    /// Is the declared source the `.as` script form (pre-lowered by s2s)?
    pub fn source_is_script(&self) -> bool {
        self.source_path().to_ascii_lowercase().ends_with(".as")
    }

    /// Every check id declared by the document, in document order.
    pub fn check_ids(&self) -> Vec<&str> {
        self.case
            .spec
            .checks
            .iter()
            .map(|c| c.id.as_str())
            .collect()
    }

    pub fn has_check(&self, id: &str) -> bool {
        self.case.spec.checks.iter().any(|c| c.id == id)
    }

    /// One check's `legacy-display` result text (the `VM result` / stdout
    /// payload), byte-exact as authored.
    pub fn result_text(&self, id: &str) -> Option<String> {
        self.case
            .spec
            .checks
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.expect.result.as_ref())
            .map(|r| r.text.clone())
    }

    /// Every declared artifact path, in document order.
    pub fn artifact_paths(&self) -> Vec<&str> {
        self.case
            .spec
            .checks
            .iter()
            .flat_map(|c| c.expect.artifacts.iter().map(|a| a.path.as_str()))
            .collect()
    }

    pub fn artifact(&self, path_suffix: &str) -> Option<&[u8]> {
        for check in &self.case.spec.checks {
            for a in &check.expect.artifacts {
                if a.path.ends_with(path_suffix) {
                    if let ArtifactPayload::Inline { bytes, .. } = &a.payload {
                        return Some(bytes);
                    }
                }
            }
        }
        None
    }
}

/// Cached document facts used by discovery (no full payload copy).
pub struct DocFacts {
    pub source_path: String,
    pub is_script: bool,
    pub checks: Vec<String>,
    pub artifacts: Vec<String>,
}

impl DocFacts {
    pub fn from_doc(doc: &LegacyDoc) -> Self {
        DocFacts {
            source_path: doc.source_path().to_string(),
            is_script: doc.source_is_script(),
            checks: doc.check_ids().into_iter().map(|s| s.to_string()).collect(),
            artifacts: doc
                .artifact_paths()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn has_check(&self, id: &str) -> bool {
        self.checks.iter().any(|c| c == id)
    }
}

/// Decode the entry document's facts, or `None` when no document exists.
///
/// A present-but-malformed document is a contract breach: the error names the
/// document so the consumer fails loudly instead of silently skipping an
/// obligation or falling back to frozen sidecars.
pub fn doc_facts(dir: &Path, stem: &str) -> Result<Option<DocFacts>, String> {
    if !doc_path(dir, stem).is_file() {
        return Ok(None);
    }
    let doc = load_case(dir, stem)?;
    Ok(Some(DocFacts::from_doc(&doc)))
}
