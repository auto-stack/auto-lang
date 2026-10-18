//! Case IR: resolve contract payload references into owned bytes.
//!
//! Resolution is the only place payloads materialize: anchored fences read
//! from the parsed [`CaseDocument`], sidecars read from disk under admission
//! checks (relative to the case file, no escape, no reparse points), inline
//! text used verbatim. Every file's bytes are hashed into the case identity.

use std::path::Path;

use crate::admission::{check_limit, read_bounded, AdmissionError, Limits};
use crate::contract::{CaseSpec, CheckSpec, Compare, Completion, PayloadSpec, StreamExpect};
use crate::document::CaseDocument;
use crate::sha256::{hex, sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFile {
    /// Virtual project path (validated at decode).
    pub virtual_path: String,
    pub bytes: Vec<u8>,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedCase {
    pub spec: CaseSpec,
    /// Entry source first, then `files` entries; virtual paths unique.
    pub files: Vec<ResolvedFile>,
    /// Virtual path of the entry source.
    pub entry_path: String,
    /// Combined hash over all files' `(virtual_path, sha256)` — the source
    /// identity an oracle PASS is bound to.
    pub source_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    pub case_id: String,
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "case {}: {}: {}", self.case_id, self.path, self.message)
    }
}

impl std::error::Error for ResolveError {}

fn fetch_payload(
    case_id: &str,
    payload: &PayloadSpec,
    field: &str,
    doc: &CaseDocument,
    case_root: &Path,
    limits: &Limits,
) -> Result<Vec<u8>, ResolveError> {
    let fail = |msg: String| ResolveError {
        case_id: case_id.to_string(),
        path: field.to_string(),
        message: msg,
    };
    match payload {
        PayloadSpec::Block(anchor) => {
            let fence = doc
                .anchor_fence(anchor)
                .map_err(|e| fail(format!("anchor {:?}: {}", anchor, e)))?;
            let bytes = doc.payload_bytes(fence).to_vec();
            check_limit(
                "max_inline_payload_bytes",
                bytes.len(),
                limits.max_inline_payload_bytes,
            )
            .map_err(|e| fail(e.to_string()))?;
            Ok(bytes)
        }
        PayloadSpec::File(rel) => {
            let full = case_root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
            read_bounded(&full, "max_sidecar_bytes", limits.max_sidecar_bytes)
                .map_err(|e: AdmissionError| fail(e.to_string()))
        }
        PayloadSpec::Text(text) => {
            let bytes = text.as_bytes().to_vec();
            check_limit(
                "max_inline_payload_bytes",
                bytes.len(),
                limits.max_inline_payload_bytes,
            )
            .map_err(|e| fail(e.to_string()))?;
            Ok(bytes)
        }
    }
}

/// Resolve all payloads of a case into owned bytes.
///
/// `case_root` is the directory holding the `.case.ad` file (sidecar base).
pub fn resolve_case(
    spec: CaseSpec,
    doc: &CaseDocument,
    case_root: &Path,
    limits: &Limits,
) -> Result<ResolvedCase, ResolveError> {
    let case_id = spec.id.clone();
    let fail = |path: String, msg: String| ResolveError {
        case_id: case_id.clone(),
        path,
        message: msg,
    };

    let mut files: Vec<ResolvedFile> = Vec::new();
    let entry_bytes = fetch_payload(
        &case_id,
        &spec.source.payload,
        "source",
        doc,
        case_root,
        limits,
    )?;
    let entry_path = spec.source.path.clone();
    files.push(ResolvedFile {
        virtual_path: entry_path.clone(),
        sha256: hex(&sha256(&entry_bytes)),
        bytes: entry_bytes,
    });
    for (i, fe) in spec.files.iter().enumerate() {
        let field = format!("files[{}]", i);
        let bytes = fetch_payload(&case_id, &fe.payload, &field, doc, case_root, limits)?;
        files.push(ResolvedFile {
            virtual_path: fe.path.clone(),
            sha256: hex(&sha256(&bytes)),
            bytes,
        });
    }

    let mut seen = std::collections::BTreeSet::new();
    for f in &files {
        if !seen.insert(f.virtual_path.clone()) {
            return Err(fail(
                "files".to_string(),
                format!("duplicate virtual path {:?}", f.virtual_path),
            ));
        }
    }

    let mut combined = String::new();
    let mut ordered: Vec<&ResolvedFile> = files.iter().collect();
    ordered.sort_by(|a, b| a.virtual_path.cmp(&b.virtual_path));
    for f in ordered {
        combined.push_str(&f.virtual_path);
        combined.push('\0');
        combined.push_str(&f.sha256);
        combined.push('\0');
    }
    let source_sha256 = hex(&sha256(combined.as_bytes()));

    Ok(ResolvedCase {
        spec,
        files,
        entry_path,
        source_sha256,
    })
}

/// One expected-artifact sidecar resolved to actual bytes (Plan 773 F-06:
/// oracle identity is over CONTENT, not just the declared reference).
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedArtifact {
    /// Logical output path (e.g. "main.rs").
    pub path: String,
    /// Sidecar reference exactly as declared (relative to the fixture base).
    pub file: String,
    pub compare: Compare,
    pub bytes: Vec<u8>,
    pub sha256: String,
}

/// A check's expectation with every `file:`-referenced sidecar read and
/// hashed. `oracle_sha256` covers completion, stream/result texts, and the
/// CONTENT hash of every expected artifact — mutating a sidecar's bytes
/// changes the identity and invalidates old PASSes.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedOracle {
    pub completion: Completion,
    pub result: Option<crate::contract::ResultExpect>,
    pub stdout: Option<StreamExpect>,
    pub stderr: Option<StreamExpect>,
    pub artifacts: Vec<ResolvedArtifact>,
    /// v2 ac/run native exit oracle (PLAN-788): part of the content-bound
    /// identity — changing an expected exit invalidates old PASSes.
    pub exit: Option<i32>,
    /// v2 ac optional perf assertion face (record-only unless declared).
    pub perf: Option<crate::contract::PerfExpect>,
    /// v2 probe (PLAN-796): per-step expectations with every
    /// `bindings_file` sidecar read and hashed (same content-bound
    /// admission as artifacts — mutating a bindings oracle changes the
    /// identity and invalidates old PASSes).
    pub probe: Option<Vec<ResolvedProbeStep>>,
    pub oracle_sha256: String,
}

/// One probe step expectation with its bindings sidecar resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProbeStep {
    pub cache: Option<String>,
    pub compute_calls: Option<u32>,
    pub bindings_file: Option<String>,
    pub bindings_bytes: Option<Vec<u8>>,
    pub bindings_sha256: Option<String>,
    pub stages_include: Vec<String>,
}

/// Resolve every `file:` reference of a check's expectation against
/// `ref_base` under admission (bounded read + reparse audit on the file and
/// its ancestors). Missing/unreadable/oversized sidecars are errors here —
/// not at comparison time — so lint and run share one full admission
/// (Plan 773 F-09).
pub fn resolve_oracle(
    case_id: &str,
    check: &CheckSpec,
    ref_base: &Path,
    limits: &Limits,
) -> Result<ResolvedOracle, ResolveError> {
    let fail = |msg: String| ResolveError {
        case_id: case_id.to_string(),
        path: "expect".to_string(),
        message: msg,
    };
    let expect = &check.expect;

    let mut artifacts = Vec::new();
    for want in &expect.artifacts {
        let full = ref_base.join(want.file.replace('/', std::path::MAIN_SEPARATOR_STR));
        let bytes =
            read_bounded(&full, "max_sidecar_bytes", limits.max_sidecar_bytes).map_err(|e| {
                fail(format!(
                    "expected artifact {} (check {:?}): {}",
                    want.file, check.id, e
                ))
            })?;
        let digest = hex(&sha256(&bytes));
        artifacts.push(ResolvedArtifact {
            path: want.path.clone(),
            file: want.file.clone(),
            compare: want.compare,
            bytes,
            sha256: digest,
        });
    }

    // PLAN-796: probe step expectations — resolve bindings sidecars under
    // the same bounded-read admission as artifacts.
    let mut probe = None;
    if let Some(steps) = &expect.probe {
        let mut out = Vec::new();
        for st in steps {
            let (bfile, bbytes, bsha) = match &st.bindings_file {
                None => (None, None, None),
                Some(f) => {
                    let full = ref_base.join(f.replace('/', std::path::MAIN_SEPARATOR_STR));
                    let bytes = read_bounded(&full, "max_sidecar_bytes", limits.max_sidecar_bytes)
                        .map_err(|e| {
                            fail(format!(
                                "probe bindings {} (check {:?}): {}",
                                f, check.id, e
                            ))
                        })?;
                    let digest = hex(&sha256(&bytes));
                    (Some(f.clone()), Some(bytes), Some(digest))
                }
            };
            out.push(ResolvedProbeStep {
                cache: st.cache.clone(),
                compute_calls: st.compute_calls,
                bindings_file: bfile,
                bindings_bytes: bbytes,
                bindings_sha256: bsha,
                stages_include: st.stages_include.clone(),
            });
        }
        probe = Some(out);
    }

    // Content-bound oracle identity: declarations AND sidecar bytes.
    let mut combined = String::new();
    combined.push_str(&format!("{:?}\0", expect.completion));
    if let Some(e) = &expect.exit {
        combined.push_str(&format!("exit\0{}\0", e));
    }
    if let Some(pf) = &expect.perf {
        combined.push_str(&format!("perf\0{:?}\0", pf.total_ms_max));
    }
    if let Some(r) = &expect.result {
        combined.push_str(&format!("result\0{}\0{}\0", r.format, r.text));
    }
    for (name, s) in [("stdout", &expect.stdout), ("stderr", &expect.stderr)] {
        if let Some(s) = s {
            combined.push_str(&format!("{}\0{}\0{:?}\0", name, s.text, s.compare));
        }
    }
    for a in &artifacts {
        combined.push_str(&format!(
            "artifact\0{}\0{}\0{:?}\0{}\0",
            a.path, a.file, a.compare, a.sha256
        ));
    }
    if let Some(steps) = &probe {
        for st in steps {
            combined.push_str(&format!(
                "probe\0{:?}\0{:?}\0{}\0{:?}\0",
                st.cache, st.compute_calls, st.bindings_sha256.clone().unwrap_or_default(), st.stages_include
            ));
        }
    }
    let oracle_sha256 = hex(&sha256(combined.as_bytes()));

    Ok(ResolvedOracle {
        completion: expect.completion.clone(),
        result: expect.result.clone(),
        stdout: expect.stdout.clone(),
        stderr: expect.stderr.clone(),
        artifacts,
        exit: expect.exit,
        perf: expect.perf,
        probe,
        oracle_sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{
        decode_case, CheckSpec, Completion, EntryMode, ExpectSpec, Options, PayloadSpec,
        SourceSpec, Syntax,
    };

    fn spec_with_files(paths: &[&str]) -> CaseSpec {
        CaseSpec {
            schema: crate::contract::CASE_SCHEMA.into(),
            id: "t.case".into(),
            tags: vec![],
            links: vec![],
            source: SourceSpec {
                payload: PayloadSpec::Text("fn main() {}".into()),
                syntax: Syntax::Auto,
                path: "main.at".into(),
            },
            files: paths
                .iter()
                .map(|p| crate::contract::FileEntry {
                    path: (*p).to_string(),
                    payload: PayloadSpec::Text("// x".into()),
                })
                .collect(),
            entry: EntryMode::Program,
            checks: vec![CheckSpec {
                id: "vm".into(),
                adapter: crate::contract::Adapter::Avm,
                action: crate::contract::Action::Run,
                tier: None,
                options: Options::default(),
                expect: ExpectSpec {
                    completion: Completion::Success,
                    result: None,
                    stdout: None,
                    stderr: None,
                    artifacts: vec![],
                    exit: None,
                    perf: None,
                    probe: None,
                },
                product_from: None,
                skip: None,
            }],
            status_skip: None,
        }
    }

    #[test]
    fn duplicate_virtual_paths_are_errors() {
        let spec = spec_with_files(&["db.at"]);
        let doc = CaseDocument::parse(b"# src ^s\n```auto\nfn main() {}\n```\n").unwrap();
        // source main.at + files db.at is fine; duplicate db.at must fail.
        let ok = resolve_case(
            spec.clone(),
            &doc,
            &std::env::temp_dir(),
            &Limits::defaults(),
        );
        assert!(ok.is_ok());
        let mut dup = spec_with_files(&["db.at", "db.at"]);
        dup.source.payload = PayloadSpec::Block("s".into());
        dup.files[1].path = "main.at".into(); // clash with entry path
        let _ = dup;
        let mut dup2 = spec_with_files(&["main.at"]); // files path == entry path
        dup2.source.payload = PayloadSpec::Block("s".into());
        let err = resolve_case(dup2, &doc, &std::env::temp_dir(), &Limits::defaults()).unwrap_err();
        assert!(err.message.contains("duplicate virtual path"), "{:?}", err);
    }

    #[test]
    fn decode_and_resolve_end_to_end() {
        let src = concat!(
            "case {\n",
            "  schema: \"autolang.case/1\"\n",
            "  id: \"lang.arith.basic\"\n",
            "  source: { block: \"s\", syntax: \"auto\", path: \"main.at\" }\n",
            "  entry: { mode: \"expression\" }\n",
            "  checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\",",
            " expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ]\n",
            "}\n",
        );
        let v = crate::atom::parse_contract(src).unwrap();
        let spec = decode_case(&v, &Limits::defaults()).unwrap();
        let doc = CaseDocument::parse(b"# T ^s\n```auto\n1 + 2\n```\n").unwrap();
        let rc = resolve_case(spec, &doc, &std::env::temp_dir(), &Limits::defaults()).unwrap();
        assert_eq!(rc.files.len(), 1);
        assert_eq!(rc.files[0].bytes, b"1 + 2\n".to_vec());
        assert_eq!(rc.entry_path, "main.at");
        assert_eq!(rc.source_sha256.len(), 64);
    }
}
