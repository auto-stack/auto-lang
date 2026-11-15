//! PLAN-1062 T-14 bridge: the trans-pipeline legacy helpers read their
//! migration documents through the shared auto-test-case reader.
use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorCase};
use auto_test_case::contract::{ArtifactPayload, PayloadSpec};
use auto_test_case::document::CaseDocument;

pub struct LegacyDoc {
    pub case: AuthorCase,
}

pub fn load_case(dir: &Path, stem: &str) -> Result<LegacyDoc, String> {
    let doc_path = dir.join(format!("{stem}.case.ad"));
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
        dir.file_name().map(|d| d.to_string_lossy().to_string()).unwrap_or_default(),
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
    Ok(LegacyDoc { case: cases.remove(0) })
}

impl LegacyDoc {
    pub fn source_text(&self) -> &str {
        match &self.case.spec.source.payload {
            PayloadSpec::Text(t) => t,
            other => unreachable!("legacy docs inline their source: {other:?}"),
        }
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
