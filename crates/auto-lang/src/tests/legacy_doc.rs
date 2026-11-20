//! PLAN-1062 T-14: document-payload reader for the legacy B-layer corpus.
//!
//! The legacy wrappers (a2r/cookbook/vm/a2c/a2ts/python/conformance) read
//! their inputs and goldens from `<stem>.case.ad` documents sitting next to
//! the original corpus entries. The ORIGINAL backends keep executing; only
//! the payload source changes (no `.at`/`.expected.*` sidecar reads on the
//! normal path). Missing docs are hard errors — never a fallback to the old
//! sidecars.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorCase};
use auto_test_case::contract::{ArtifactPayload, PayloadSpec};
use auto_test_case::document::CaseDocument;

/// One migrated corpus entry: the inline source bytes plus the oracle
/// artifacts declared by its document.
pub struct LegacyDoc {
    pub case: AuthorCase,
    pub document_path: PathBuf,
}

/// Read `<dir>/<stem>.case.ad` and return its single decoded case.
pub fn load_case(dir: &Path, stem: &str) -> Result<LegacyDoc, String> {
    load_document(&dir.join(format!("{stem}.case.ad")))
}

/// PLAN-1066 T-05: read an explicitly named author document (e.g.
/// `main.project.case.ad`) and return its single decoded case.
pub fn load_document(doc_path: &Path) -> Result<LegacyDoc, String> {
    if let Some(canonical) = canonical_vm_case(doc_path)? {
        return Ok(canonical);
    }
    let bytes = std::fs::read(doc_path)
        .map_err(|e| format!("legacy doc {} missing/unreadable: {e} (migration doc required; sidecar fallback is forbidden)", doc_path.display()))?;
    let text = std::str::from_utf8(&bytes).map_err(|e| format!("{}: {e}", doc_path.display()))?;
    let doc =
        CaseDocument::parse(text.as_bytes()).map_err(|e| format!("{}: {e}", doc_path.display()))?;
    let rel = doc_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
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
        document_path: doc_path.to_path_buf(),
    })
}

/// Resolve exact migrations by their retained case identity. Other rows
/// record coverage relationships and must not redirect a legacy reader.
fn canonical_vm_case(path: &Path) -> Result<Option<LegacyDoc>, String> {
    use auto_test_case::discovery::{load_suite, LoadedSuite};

    static ALIASES: OnceLock<Result<BTreeMap<String, String>, String>> = OnceLock::new();
    static CORPUS: OnceLock<Result<LoadedSuite, String>> = OnceLock::new();
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    if !path.starts_with(manifest.join("test/vm")) {
        return Ok(None);
    }
    let Some(repo) = manifest.parent().and_then(Path::parent) else { return Ok(None); };
    let rel = path.strip_prefix(repo).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
    let cases = manifest.join("test/cases");
    let aliases = ALIASES.get_or_init(|| {
        let path = cases.join("language/migration/legacy-map.tsv");
        let contents = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut aliases = BTreeMap::new();
        for row in contents.lines().filter(|row| !row.is_empty() && !row.starts_with('#')) {
            let fields: Vec<_> = row.split('\t').collect();
            if let [source, old, _, _, _, _, _, canonical, _, _, _, "mapped"] = fields.as_slice() {
                if old == canonical {
                    let source = source.strip_prefix("../../../").unwrap_or(source);
                    if let Some(other) = aliases.insert(source.to_owned(), (*canonical).to_owned()) {
                        if other != *canonical {
                            return Err(format!("ambiguous canonical migration for {source}"));
                        }
                    }
                }
            }
        }
        Ok(aliases)
    }).as_ref().map_err(Clone::clone)?;
    let Some(id) = aliases.get(&rel) else { return Ok(None); };
    let suite = CORPUS.get_or_init(|| {
        let suite = load_suite(&cases.join("language.suite.ad"), &Limits::defaults())
            .map_err(|err| err.to_string())?;
        if let Some(first) = suite.errors.first() { return Err(first.to_string()); }
        Ok(suite)
    }).as_ref().map_err(Clone::clone)?;
    for file in &suite.cases {
        if let Some((index, spec)) = file.specs.iter().enumerate().find(|(_, spec)| &spec.id == id) {
            let span = file.doc_spans.get(index)
                .ok_or_else(|| format!("canonical case {id} lacks a document span"))?;
            return Ok(Some(LegacyDoc {
                document_path: file.path.clone(),
                case: AuthorCase {
                    spec: spec.clone(),
                    key: span.key.clone(),
                    heading_line: span.heading_line,
                    end_line: span.end_line,
                    check_spans: span.check_spans.clone(),
                },
            }));
        }
    }
    Err(format!("migrated legacy case {id} missing from the canonical language suite"))
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
        self.case
            .spec
            .checks
            .first()
            .and_then(|c| c.expect.result.as_ref().map(|r| r.text.as_str()))
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
