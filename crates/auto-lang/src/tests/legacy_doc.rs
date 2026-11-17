//! PLAN-1062 T-14: document-payload reader for the legacy B-layer corpus.
//!
//! The legacy wrappers (a2r/cookbook/vm/a2c/a2ts/python/conformance) read
//! their inputs and goldens from `<stem>.case.ad` documents sitting next to
//! the original corpus entries. The ORIGINAL backends keep executing; only
//! the payload source changes (no `.at`/`.expected.*` sidecar reads on the
//! normal path). Missing docs are hard errors — never a fallback to the old
//! sidecars.

use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorCase};
use auto_test_case::contract::{ArtifactPayload, PayloadSpec};
use auto_test_case::document::CaseDocument;

/// One migrated corpus entry: the inline source bytes plus the oracle
/// artifacts declared by its document.
pub struct LegacyDoc {
    pub case: AuthorCase,
}

/// Read `<dir>/<stem>.case.ad` and return its single decoded case.
pub fn load_case(dir: &Path, stem: &str) -> Result<LegacyDoc, String> {
    let doc_path = dir.join(format!("{stem}.case.ad"));
    let bytes = std::fs::read(&doc_path)
        .map_err(|e| format!("legacy doc {} missing/unreadable: {e} (migration doc required; sidecar fallback is forbidden)", doc_path.display()))?;
    let text = std::str::from_utf8(&bytes).map_err(|e| format!("{}: {e}", doc_path.display()))?;
    let doc = CaseDocument::parse(text.as_bytes()).map_err(|e| format!("{}: {e}", doc_path.display()))?;
    let rel = format!("{}/{}.case.ad",
        dir.file_name().map(|d| d.to_string_lossy().to_string()).unwrap_or_default(), stem);
    let mut cases = decode_document(&doc, &rel, &Limits::defaults())
        .map_err(|e| format!("{}: {}", doc_path.display(), e.message))?;
    if cases.len() != 1 {
        return Err(format!(
            "{}: expected exactly one case, found {}",
            doc_path.display(),
            cases.len()
        ));
    }
    Ok(LegacyDoc { case: cases.remove(0) })
}

impl LegacyDoc {
    /// The inline source text (the old `<name>.at` bytes).
    pub fn source_text(&self) -> &str {
        match &self.case.spec.source.payload {
            PayloadSpec::Text(t) => t,
            other => unreachable!("legacy docs inline their source: {other:?}"),
        }
    }

    /// The virtual source path (`<name>.at`).
    pub fn source_path(&self) -> &str {
        &self.case.spec.source.path
    }

    /// Artifact oracle bytes by logical path (inline payloads only).
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

    /// First text-channel expectation (VM result stream).
    pub fn result_text(&self) -> Option<&str> {
        self.case.spec.checks.first().and_then(|c| c.expect.result.as_ref().map(|r| r.text.as_str()))
    }

    /// PLAN-1066 T-03[B]: one named check's result text, when declared.
    pub fn result_text_for(&self, check_id: &str) -> Option<&str> {
        self.case
            .spec
            .checks
            .iter()
            .find(|c| c.id == check_id)
            .and_then(|c| c.expect.result.as_ref())
            .map(|r| r.text.as_str())
    }

    /// Does the document declare a check with this id?
    pub fn has_check(&self, check_id: &str) -> bool {
        self.case.spec.checks.iter().any(|c| c.id == check_id)
    }

    /// The document's declared virtual source path (`sort_int.at`).
    pub fn source_is_script(&self) -> bool {
        self.case
            .spec
            .source
            .path
            .to_ascii_lowercase()
            .ends_with(".as")
    }
}
