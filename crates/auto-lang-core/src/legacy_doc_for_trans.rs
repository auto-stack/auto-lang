//! PLAN-1062 T-14 bridge: the trans-pipeline legacy helpers read their
//! migration documents through the shared auto-test-case reader.
//!
//! PLAN-1066 T-03[B]: the migration document is also the AUTHORITY for
//! discovery and execution of the N corpus families (vm/a2r/a2c/a2ts/
//! cookbook): the selected set comes from `<stem>.case.ad` presence, the
//! virtual source path and the oracle channels come from the decoded
//! document, and the original `.at`/`.as`/`.expected.*` sidecars are frozen
//! history — never read on the normal path and never a fallback.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorCase};
use auto_test_case::contract::{ArtifactPayload, PayloadSpec};
use auto_test_case::discovery::{load_suite, LoadedSuite};
use auto_test_case::document::CaseDocument;

pub struct LegacyDoc {
    pub case: AuthorCase,
    pub document_path: PathBuf,
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

struct Migration {
    id: String,
    checks: BTreeSet<String>,
}

/// A discovery pass shares one view of the alias table and canonical suite.
/// Ordinary per-case reads create a fresh view, so later edits are visible.
pub struct CanonicalVmCases {
    root: PathBuf,
    migrations: OnceLock<Result<BTreeMap<String, Migration>, String>>,
    suite: OnceLock<Result<LoadedSuite, String>>,
}

impl CanonicalVmCases {
    pub fn new(test_root: &Path) -> Self {
        Self {
            root: test_root.to_path_buf(),
            migrations: OnceLock::new(),
            suite: OnceLock::new(),
        }
    }

    fn read_migrations(&self) -> Result<BTreeMap<String, Migration>, String> {
        let path = self.root.join("cases/language/migration/legacy-map.tsv");
        let map = match std::fs::read_to_string(&path) {
            Ok(map) => map,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(BTreeMap::new())
            }
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        let mut migrations = BTreeMap::new();
        for line in map.lines().filter(|line| !line.starts_with('#')) {
            let fields: Vec<_> = line.split('\t').collect();
            if let [old_path, old_id, old_check, _, _, _, _, new_id, _, _, _, "mapped"] =
                fields.as_slice()
            {
                if old_id != new_id {
                    continue;
                }
                let source = old_path.trim_start_matches("../../../");
                let migration = migrations
                    .entry(source.to_owned())
                    .or_insert_with(|| Migration {
                        id: (*new_id).to_owned(),
                        checks: BTreeSet::new(),
                    });
                if migration.id != *new_id {
                    return Err(format!("ambiguous canonical VM migration for {source}"));
                }
                migration.checks.insert((*old_check).to_owned());
            }
        }
        Ok(migrations)
    }

    /// Identity-preserving migrations expose only the original VM checks
    /// through the legacy reader. AV probes stay in the canonical suite.
    pub fn find(&self, path: &Path) -> Result<Option<LegacyDoc>, String> {
        let Ok(relative) = path.strip_prefix(&self.root) else {
            return Ok(None);
        };
        if !relative.starts_with("vm") {
            return Ok(None);
        }
        let source = format!(
            "crates/auto-lang/test/{}",
            relative.to_string_lossy().replace('\\', "/")
        );
        let migrations = self
            .migrations
            .get_or_init(|| self.read_migrations())
            .as_ref()
            .map_err(Clone::clone)?;
        let Some(migration) = migrations.get(&source) else {
            return Ok(None);
        };
        let suite = self
            .suite
            .get_or_init(|| {
                let suite = load_suite(
                    &self.root.join("cases/language.suite.ad"),
                    &Limits::defaults(),
                )
                .map_err(|error| error.to_string())?;
                if let Some(error) = suite.errors.first() {
                    return Err(error.to_string());
                }
                Ok(suite)
            })
            .as_ref()
            .map_err(Clone::clone)?;
        let mut matches = suite
            .cases
            .iter()
            .flat_map(|file| {
                file.specs
                    .iter()
                    .zip(&file.doc_spans)
                    .map(move |(spec, span)| (spec, span, &file.path))
            })
            .filter(|(spec, _, _)| spec.id == migration.id);
        let id = &migration.id;
        let (spec, span, document_path) = matches
            .next()
            .ok_or_else(|| format!("canonical VM case {id} is absent"))?;
        if matches.next().is_some() {
            return Err(format!("canonical VM case {id} is duplicated"));
        }
        let mut spec = spec.clone();
        spec.checks
            .retain(|check| migration.checks.contains(&check.id));
        if spec.checks.len() != migration.checks.len() {
            return Err(format!("canonical VM case {id} lost a legacy check"));
        }
        Ok(Some(LegacyDoc {
            document_path: document_path.clone(),
            case: AuthorCase {
                spec,
                key: span.key.clone(),
                heading_line: span.heading_line,
                end_line: span.end_line,
                check_spans: span.check_spans.clone(),
            },
        }))
    }
}

fn canonical_vm_cases(path: &Path) -> Option<CanonicalVmCases> {
    let category = path.parent()?.parent()?;
    let vm = category.parent()?;
    if !vm.ends_with("vm") {
        return None;
    }
    Some(CanonicalVmCases::new(vm.parent()?))
}

pub fn load_case(dir: &Path, stem: &str) -> Result<LegacyDoc, String> {
    let reader = canonical_vm_cases(&doc_path(dir, stem));
    read_case(dir, stem, reader.as_ref())
}

pub(crate) fn read_case(
    dir: &Path,
    stem: &str,
    vm: Option<&CanonicalVmCases>,
) -> Result<LegacyDoc, String> {
    let doc_path = doc_path(dir, stem);
    if let Some(reader) = vm {
        if let Some(doc) = reader.find(&doc_path)? {
            return Ok(doc);
        }
    }
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
        document_path: doc_path,
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
    let reader = canonical_vm_cases(&doc_path(dir, stem));
    read_doc_facts(dir, stem, reader.as_ref())
}

pub(crate) fn read_doc_facts(
    dir: &Path,
    stem: &str,
    vm: Option<&CanonicalVmCases>,
) -> Result<Option<DocFacts>, String> {
    let path = doc_path(dir, stem);
    if !path.is_file() {
        return match vm {
            Some(reader) => Ok(reader.find(&path)?.as_ref().map(DocFacts::from_doc)),
            None => Ok(None),
        };
    }
    let doc = read_case(dir, stem, vm)?;
    Ok(Some(DocFacts::from_doc(&doc)))
}
