//! Project materialization and proposal-only baseline generation.
//!
//! `materialize_project` writes a case's virtual project into a scratch
//! directory (unique per run/case/check — never the fixture tree).
//!
//! `write_proposals` is the ONLY oracle-adjacent writer, and it writes
//! exclusively into a caller-provided proposal directory: candidates,
//! diffs, and affected-check manifests. Accepting a baseline is a separate
//! human/PR action — normal runs never touch expected files.
//!
//! Plan 773 F-11: proposals retain BOTH sides' identity — the current
//! (old) oracle hash and the candidate hash, a real content diff, the
//! other checks sharing the same oracle sidecar, and the run report the
//! candidate came from. Read failures PROPAGATE (a proposal that silently
//! skips an unreadable actual is not evidence).
//!
//! Plan 773 F-09: `lint_resolve` is the full admission (payloads AND
//! oracles) that lint/inspect share with the runner — references are
//! validated before anything executes.

use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::compare::unified_diff_hunk;
use auto_test_case::contract::{ArtifactExpect, CheckSpec};
use auto_test_case::discovery::{load_suite, LoadedSuite};
use auto_test_case::ir::{resolve_case, resolve_oracle, ResolvedCase};
use auto_test_case::report::{CheckOutcome, RunReport, Status};

/// Write the case's virtual project into `dir` (creating parents).
pub fn materialize_project(case: &ResolvedCase, dir: &Path) -> Result<(), String> {
    for file in &case.files {
        let target = dir.join(
            file.virtual_path
                .replace('/', std::path::MAIN_SEPARATOR_STR),
        );
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("scratch mkdir: {}", e))?;
        }
        std::fs::write(&target, &file.bytes)
            .map_err(|e| format!("scratch write {}: {}", target.display(), e))?;
    }
    Ok(())
}

/// One candidate baseline entry inside the proposal directory.
#[derive(Debug, Clone)]
pub struct Proposal {
    pub case_id: String,
    pub check_id: String,
    pub logical_path: String,
    pub candidate_bytes: Vec<u8>,
    pub candidate_sha256: String,
    /// Oracle sidecar reference (fixture-root relative) as declared.
    pub oracle_file: String,
    /// Current (old) oracle content — the bytes that would be replaced.
    pub old_bytes: Vec<u8>,
    pub old_sha256: String,
    /// Other case/check pairs whose oracle declares the SAME sidecar file —
    /// accepting this candidate affects them too.
    pub shared_consumers: Vec<String>,
    /// Report artifact path this candidate was generated from, when known.
    pub report_ref: Option<String>,
    pub suite: String,
}

/// Collect proposal candidates for FAILED artifact assertions of a report
/// (Plan 773 F-11). Candidate bytes come from the check's persisted
/// `actual/` scratch files; the old side and shared-consumer context are
/// re-resolved from the suite contract so both identities are bound to the
/// same revision. Every read failure propagates.
pub fn collect_proposals(
    report: &RunReport,
    suite_path: &Path,
    limits: &Limits,
) -> Result<Vec<Proposal>, String> {
    let loaded = load_suite(suite_path, limits).map_err(|e| format!("reload suite: {}", e))?;
    let mut out = Vec::new();
    for r in &report.results {
        if r.status != Status::Fail {
            continue;
        }
        if !r
            .mismatches
            .iter()
            .any(|m| m.observation.starts_with("artifact"))
        {
            continue;
        }
        let check = declaring_check(&loaded, r)?;
        for a in &r.artifacts {
            if a.logical_path.starts_with('<') {
                continue; // pseudo-evidence refs are not baselines
            }
            let want: &ArtifactExpect = check
                .expect
                .artifacts
                .iter()
                .find(|w| w.path == a.logical_path)
                .ok_or_else(|| {
                    format!(
                        "proposal: artifact {:?} of {}/{} is not declared by the contract",
                        a.logical_path, r.case_id, r.check_id
                    )
                })?;
            // Old side: re-resolve the oracle sidecar through full admission.
            let oracle =
                resolve_oracle(&r.case_id, check, &loaded.fixture_base, limits).map_err(|e| {
                    format!(
                        "proposal: oracle resolution for {}/{}: {}",
                        r.case_id, r.check_id, e
                    )
                })?;
            let resolved_art = oracle
                .artifacts
                .iter()
                .find(|x| x.path == a.logical_path)
                .ok_or_else(|| format!("proposal: resolved oracle lacks {:?}", a.logical_path))?;
            // Candidate side: the persisted actual. Read failures propagate
            // (F-11: never silently skip).
            let candidate_bytes = std::fs::read(&a.stored_path).map_err(|e| {
                format!(
                    "proposal: reading actual {} of {}/{}: {}",
                    a.stored_path, r.case_id, r.check_id, e
                )
            })?;
            // Shared consumers: every OTHER check declaring the same sidecar.
            let mut shared = Vec::new();
            for file in &loaded.cases {
                for spec in &file.specs {
                    for other in &spec.checks {
                        if spec.id == r.case_id && other.id == r.check_id {
                            continue;
                        }
                        if other.expect.artifacts.iter().any(|w| w.file == want.file) {
                            shared.push(format!("{}:{}", spec.id, other.id));
                        }
                    }
                }
            }
            out.push(Proposal {
                case_id: r.case_id.clone(),
                check_id: r.check_id.clone(),
                logical_path: a.logical_path.clone(),
                candidate_bytes,
                candidate_sha256: a.sha256.clone(),
                oracle_file: want.file.clone(),
                old_bytes: resolved_art.bytes.clone(),
                old_sha256: resolved_art.sha256.clone(),
                shared_consumers: shared,
                report_ref: None,
                suite: report.suite_id.clone(),
            });
        }
    }
    Ok(out)
}

/// Fetch the CheckSpec behind one report row (helper for proposal collection).
fn declaring_check<'a>(loaded: &'a LoadedSuite, r: &CheckOutcome) -> Result<&'a CheckSpec, String> {
    for file in &loaded.cases {
        for spec in &file.specs {
            if spec.id != r.case_id {
                continue;
            }
            for check in &spec.checks {
                if check.id == r.check_id {
                    return Ok(check);
                }
            }
        }
    }
    Err(format!(
        "proposal: case {:?} check {:?} not found in the reloaded suite",
        r.case_id, r.check_id
    ))
}

/// Write proposal candidates under `proposal_dir/<case>/<check>/<logical>`;
/// returns the number written. Never writes anywhere else. Each candidate
/// gets a sibling `.diff` file with the real content diff; the manifest
/// records old/new hashes, shared consumers, and the source report.
pub fn write_proposals(
    proposals: &[Proposal],
    proposal_dir: &Path,
    report_ref: Option<&Path>,
) -> Result<usize, String> {
    use std::io::Write as _;
    let mut count = 0usize;
    let manifest_path = proposal_dir.join("PROPOSALS.md");
    let mut manifest = String::from(
        "# Baseline proposals (candidate only)\n\n\
         Generated by a normal run; NOT applied. Accepting a baseline is a\n\
         separate, reviewable action — copy candidates into the case tree\n\
         deliberately and update the contract hashes.\n\n",
    );
    for p in proposals {
        let rel = format!(
            "{}/{}/{}",
            sanitize(&p.case_id),
            sanitize(&p.check_id),
            sanitize(&p.logical_path)
        );
        let target = proposal_dir.join(&rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("proposal mkdir: {}", e))?;
        }
        std::fs::write(&target, &p.candidate_bytes)
            .map_err(|e| format!("proposal write {}: {}", target.display(), e))?;
        // Real content diff old → candidate (F-11).
        let diff = unified_diff_hunk(&p.old_bytes, &p.candidate_bytes, 3);
        let diff_rel = format!("{}.diff", rel);
        if let Some(diff_text) = &diff {
            let diff_path = proposal_dir.join(&diff_rel);
            std::fs::write(&diff_path, diff_text)
                .map_err(|e| format!("proposal diff {}: {}", diff_path.display(), e))?;
        }
        manifest.push_str(&format!(
            "- `{}` — case `{}`, check `{}` (suite `{}`)\n  - oracle sidecar: `{}` (old sha256 `{}`, {} bytes)\n  - candidate sha256 `{}`, {} bytes; diff: {}\n  - shared consumers of the same sidecar: {}\n  - source report: {}\n",
            rel,
            p.case_id,
            p.check_id,
            p.suite,
            p.oracle_file,
            p.old_sha256,
            p.old_bytes.len(),
            p.candidate_sha256,
            p.candidate_bytes.len(),
            if diff.is_some() {
                format!("`{}`", diff_rel)
            } else {
                "(identical)".to_string()
            },
            if p.shared_consumers.is_empty() {
                "none".to_string()
            } else {
                p.shared_consumers.join(", ")
            },
            p.report_ref
                .as_deref()
                .or(report_ref.and_then(|r| r.to_str()))
                .unwrap_or("(not recorded)"),
        ));
        count += 1;
    }
    if count > 0 {
        let mut f =
            std::fs::File::create(&manifest_path).map_err(|e| format!("manifest: {}", e))?;
        f.write_all(manifest.as_bytes())
            .map_err(|e| format!("manifest: {}", e))?;
    }
    Ok(count)
}

/// Full admission shared by lint/inspect (Plan 773 F-09): resolve every
/// case's payloads AND every check's oracle references (existence,
/// readability, bounds, reparse-cleanliness). Returns the located error
/// list; empty means the whole suite resolves.
pub fn lint_resolve(suite_path: &Path, limits: &Limits) -> Result<Vec<String>, String> {
    let loaded = load_suite(suite_path, limits).map_err(|e| e.to_string())?;
    let mut errors: Vec<String> = loaded.errors.iter().map(|e| e.to_string()).collect();
    for file in &loaded.cases {
        for spec in &file.specs {
            if let Err(e) = resolve_case(spec.clone(), &file.doc, &loaded.fixture_base, limits) {
                errors.push(format!("{}: {}", file.path.display(), e));
                continue;
            }
            for check in &spec.checks {
                if let Err(e) = resolve_oracle(&spec.id, check, &loaded.fixture_base, limits) {
                    errors.push(format!("{}: {}", file.path.display(), e));
                }
            }
        }
    }
    Ok(errors)
}

fn sanitize(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Read-only guard evidence helper: hashes of a check outcome's identity.
pub fn outcome_identity(r: &CheckOutcome) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        r.identity.suite,
        r.identity.case_file,
        r.identity.source,
        r.identity.oracle,
        r.identity.contract
    )
}
