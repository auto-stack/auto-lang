//! PLAN-1044 T-04: the AAC baseline admission checker
//! (`autolang.aac-baseline/1`) — the frozen CS/CL/subject/bridge/supply/
//! stage contracts plus the A-receipt and source identity pins.
//!
//! The contract assets live under
//! `crates/auto-lang/test/cases/ace-m2/aac-baseline-1044/`:
//! - `contract-profile.tsv` — identity pins (A receipt + source/spec
//!   hashes), support profile (host/target/CT/ABI/schema), and the
//!   frozen M2-A resource budgets;
//! - `subject-map.tsv` — which module is Auto subject authority vs. the
//!   bounded Rust retention;
//! - `bridge-contracts.tsv` — trust-boundary seams (checked-flag, source
//!   rebinding, bad cache, legacy fallback ...);
//! - `cs-requirements.tsv` / `cl-requirements.tsv` — the CS source-demand
//!   rows and the full CL denominator (final M2 AV/AC/A2R required +
//!   A-deferred increments; the CL matrix is the compiled-language
//!   support target and never shrinks because CS does not use a row);
//! - `supply-matrix.tsv` — the Design59 §9 F01..F10 supply responsibilities
//!   bound to actual closing tasks/owners;
//! - `stage-test-contract.tsv` — stage-driver / cross-repo handoff rules.
//!
//! Admission families (frozen by PLAN-1044 §6, V-01..V-06):
//! - identity: every pinned file must exist at its sha16 prefix; a stale
//!   or forged receipt is RED;
//! - completeness: dropping any CS/CL/supply/stage/subject/bridge row is
//!   RED with the missing row id named — the denominator cannot shrink;
//! - vocabulary: disposition/phase/kind/authority values are frozen; a
//!   required row flipped to NA (fake platform/NA exemption) is RED;
//! - ownership: an unowned row or a deferred row without a named owner is
//!   RED;
//! - subject/trust: a semantic-authority module listed as a Rust backend
//!   helper, or a serialized `checked=true` credential, is RED; legal
//!   MIR-machine backend retention still admits;
//! - evidence honesty: a pending/inflight row citing a receipt is RED.

use std::path::{Path, PathBuf};

/// Contract schema identity (wire/format version).
pub const AAC_SCHEMA: &str = "autolang.aac-baseline/1";

/// Repo-relative location of the frozen contract assets.
pub const DATA_DIR_REL: &str = "crates/auto-lang/test/cases/ace-m2/aac-baseline-1044";

pub const PROFILE_HEADER: &str = "row_id\tkind\tkey\tvalue\tunit\tnote";
pub const SUBJECT_HEADER: &str = "row_id\tmodule_path\tsubject_authority\trust_retention\tnote";
pub const BRIDGE_HEADER: &str = "row_id\tseam\tdimension\tvalue\tnote";
pub const REQ_HEADER: &str = "row_id\tkind\tfamily\trequirement\tproducer_chain\tconsumer\tphase\tdisposition\towner\tsource_ref\tevidence_state\tevidence_ref";
pub const SUPPLY_HEADER: &str =
    "row_id\tfinding\tsupply_gap\tclosing_owner\tdelivery_task\tinput\toracle\tclosing_condition";
pub const STAGE_HEADER: &str = "row_id\tdimension\tvalue\tnote";

pub const PROFILE_KINDS: [&str; 3] = ["identity", "profile", "resource"];
pub const REQ_KINDS: [&str; 2] = ["CS", "CL"];
pub const PHASES: [&str; 2] = ["stage", "final"];
/// `NA` is deliberately absent: a required row flipped to NA (fake
/// platform/unsupported exemption) must be rejected.
pub const DISPOSITIONS: [&str; 2] = ["required", "deferred"];
pub const EVIDENCE_STATES: [&str; 3] = ["pending", "inflight", "landed"];
pub const SUBJECT_AUTHORITIES: [&str; 4] =
    ["auto", "auto-driver", "rust-retained", "rust-reference"];
pub const RECEIPT_ROOTS: [&str; 2] = ["docs/specs/", "docs/plans/archive/"];

/// The frozen identity row set (V-01/V-02: deleting an A-merged identity
/// row is RED).
pub fn identity_rows() -> Vec<String> {
    [
        "profile.identity.a-receipt",
        "profile.identity.design59",
        "profile.identity.roadmap",
        "profile.identity.source-mod",
        "profile.identity.source-hir",
        "profile.identity.mir",
        "profile.identity.source-query",
        "profile.identity.grammar",
        "profile.identity.spec-ac-project",
        "profile.identity.spec-hir-stage",
        "profile.identity.spec-abi",
    ]
    .map(String::from)
    .to_vec()
}

pub fn profile_rows() -> Vec<String> {
    [
        "profile.host",
        "profile.target",
        "profile.abi",
        "profile.ct-domain",
        "profile.execution-mode",
        "profile.schema",
        "profile.checked-serialization",
        "profile.receipt-roots",
        "profile.driver-policy",
    ]
    .map(String::from)
    .to_vec()
}

pub fn resource_rows() -> Vec<String> {
    [
        "resource.front-work-bytes",
        "resource.parse-depth-max",
        "resource.semantic-steps-max",
        "resource.semantic-wall-ms",
        "resource.ct-root-wall-ms",
    ]
    .map(String::from)
    .to_vec()
}

/// Frozen per-row expected values (a deviation is a trust/identity
/// violation, not an editor's choice).
pub fn frozen_profile_values() -> Vec<(&'static str, &'static str)> {
    vec![
        ("profile.host", "windows-x64"),
        ("profile.target", "windows-x64"),
        ("profile.abi", "autoabi-windows-x64"),
        ("profile.ct-domain", "hosted-av"),
        ("profile.execution-mode", "single-request-serial"),
        ("profile.schema", AAC_SCHEMA),
        ("profile.checked-serialization", "never-trusted"),
    ]
}

pub fn frozen_subject_authorities() -> Vec<(&'static str, &'static str)> {
    vec![
        ("subject.syntax", "auto"),
        ("subject.bind", "auto"),
        ("subject.semantics", "auto"),
        ("subject.cte", "auto"),
        ("subject.lower", "auto"),
        ("subject.incremental", "auto"),
        ("subject.native", "auto-driver"),
    ]
}

pub fn frozen_bridge_values() -> Vec<(&'static str, &'static str)> {
    vec![
        ("bridge.syntax-hir.authority", "auto-only"),
        ("bridge.mir-backend.checked-flag", "reject"),
        ("bridge.ct-av.source-input", "reject"),
        ("bridge.incremental.cache-upgrade", "reject"),
        ("bridge.ffi-os.link-fallback", "reject"),
        ("bridge.runtime.execution", "rust-retained"),
        (
            "bridge.incremental.publish",
            "hash-schema-target-stdlib-grammar-profile-pass-bridge",
        ),
    ]
}

/// The frozen CS denominator (source-demand rows, Design59 §3).
pub fn required_cs_rows() -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    for i in 101..=107 {
        rows.push(format!("CS-{i}"));
    }
    for i in 201..=209 {
        rows.push(format!("CS-{i}"));
    }
    rows
}

/// The frozen CL denominator (compiled-language support, Design59 §3
/// family mapping; never shrinks because CS does not use a row).
pub fn required_cl_rows() -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    let ranges: &[(&str, u32, u32)] = &[
        ("CL-1", 1, 6),
        ("CL-2", 1, 5),
        ("CL-3", 1, 4),
        ("CL-4", 1, 4),
        ("CL-5", 1, 5),
        ("CL-6", 1, 2),
        ("CL-7", 1, 5),
        ("CL-8", 1, 5),
        ("CL-9", 1, 6),
    ];
    for (prefix, from, to) in ranges {
        for i in *from..=*to {
            rows.push(format!("{prefix}{i:02}"));
        }
    }
    rows
}

pub fn required_subject_rows() -> Vec<String> {
    [
        "subject.syntax",
        "subject.bind",
        "subject.semantics",
        "subject.cte",
        "subject.lower",
        "subject.incremental",
        "subject.native",
    ]
    .map(String::from)
    .to_vec()
}

pub fn required_bridge_rows() -> Vec<String> {
    [
        "bridge.syntax-hir.authority",
        "bridge.mir-backend.checked-flag",
        "bridge.ct-av.source-input",
        "bridge.incremental.cache-upgrade",
        "bridge.ffi-os.link-fallback",
        "bridge.runtime.execution",
        "bridge.incremental.publish",
    ]
    .map(String::from)
    .to_vec()
}

pub fn required_supply_rows() -> Vec<String> {
    [
        "supply.f01.ct-pipeline",
        "supply.f02.parser-boundary",
        "supply.f03.hash-codec",
        "supply.f04.persistence",
        "supply.f05.driver-mir",
        "supply.f06.demand-matrix",
        "supply.f06.package-lock",
        "supply.f07.platform-axes",
        "supply.f08.test-owner",
        "supply.f09.rust-entry-window",
        "supply.f10.generations",
    ]
    .map(String::from)
    .to_vec()
}

pub fn required_stage_rows() -> Vec<String> {
    [
        "stage-test.case-format-owner",
        "stage-test.case-producer",
        "stage-test.worker",
        "stage-test.driver-policy",
        "stage-test.evidence-policy",
        "stage-test.cross-repo",
        "stage-test.gen0-window",
    ]
    .map(String::from)
    .to_vec()
}

/// Frozen stage-contract values (V-06: auto-down must NOT be the language
/// TestCase producer; a rust-groups-only report must NOT pass).
pub fn frozen_stage_values() -> Vec<(&'static str, &'static str)> {
    vec![
        ("stage-test.case-format-owner", "auto-down"),
        ("stage-test.case-producer", "auto-lang"),
        ("stage-test.worker", "experimental/ace/crates/case-worker"),
        ("stage-test.driver-policy", "first-user-work-task"),
        ("stage-test.evidence-policy", "auto-stage-run-required"),
        ("stage-test.cross-repo", "provenance-pinned"),
        ("stage-test.gen0-window", "gen0-legal"),
    ]
}

/// Admission summary of a well-formed contract set.
#[derive(Debug, Clone, Default)]
pub struct AdmissionReport {
    pub identity_rows: usize,
    pub profile_rows: usize,
    pub resource_rows: usize,
    pub subject_rows: usize,
    pub bridge_rows: usize,
    pub cs_rows: usize,
    pub cl_rows: usize,
    pub supply_rows: usize,
    pub stage_rows: usize,
}

fn sha16_hex(bytes: &[u8]) -> String {
    auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(bytes))
}

/// A `path:sha16` pin whose file must exist at the pinned hash prefix.
fn check_path_pin(
    value: &str,
    workspace_root: &Path,
    row_id: &str,
    allowed_roots: &[&str],
) -> Result<(), String> {
    let Some((path, pinned)) = value.rsplit_once(':') else {
        return Err(format!("{row_id}: identity `{value}` is not `path:sha16`"));
    };
    if !allowed_roots.iter().any(|root| path.starts_with(root)) {
        return Err(format!(
            "{row_id}: identity `{path}` outside receipt roots {allowed_roots:?}"
        ));
    }
    let full = workspace_root.join(path);
    let bytes =
        std::fs::read(&full).map_err(|e| format!("{row_id}: identity `{path}` unreadable: {e}"))?;
    let actual = sha16_hex(&bytes);
    // Hex pins are case-insensitive (PowerShell/lowercase tooling differ).
    if !actual
        .to_ascii_lowercase()
        .starts_with(&pinned.to_ascii_lowercase())
    {
        return Err(format!(
            "{row_id}: identity `{path}` sha256 mismatch: pinned `{pinned}` actual `{actual}` (stale or forged identity)"
        ));
    }
    Ok(())
}

fn validate_profile(text: &str, root: &Path) -> Result<AdmissionReport, String> {
    let mut lines = text.lines();
    if lines.next() != Some(PROFILE_HEADER) {
        return Err(format!(
            "contract-profile schema header mismatch: expected `{PROFILE_HEADER}`"
        ));
    }
    let mut seen_identity = Vec::new();
    let mut seen_profile = Vec::new();
    let mut seen_resource = Vec::new();
    let mut report = AdmissionReport::default();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            return Err(format!("contract-profile line {}: empty row", index + 2));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 6 {
            return Err(format!(
                "contract-profile line {}: expected 6 tab-separated fields, got {}",
                index + 2,
                fields.len()
            ));
        }
        let row_id = fields[0].to_string();
        let kind = fields[1];
        let key = fields[2];
        let value = fields[3];
        let unit = fields[4];
        let note = fields[5];
        if !PROFILE_KINDS.contains(&kind) {
            return Err(format!(
                "contract-profile line {}: row `{row_id}` kind `{kind}` not in {PROFILE_KINDS:?}",
                index + 2
            ));
        }
        if key.trim().is_empty() || value.trim().is_empty() {
            return Err(format!(
                "contract-profile line {}: row `{row_id}` key/value empty",
                index + 2
            ));
        }
        if unit.trim().is_empty() || note.trim().is_empty() {
            return Err(format!(
                "contract-profile line {}: row `{row_id}` unit/note empty",
                index + 2
            ));
        }
        match kind {
            "identity" => {
                if !identity_rows().contains(&row_id) {
                    return Err(format!(
                        "contract-profile line {}: unknown identity row `{row_id}`",
                        index + 2
                    ));
                }
                if seen_identity.contains(&row_id) {
                    return Err(format!(
                        "contract-profile line {}: duplicate identity row `{row_id}`",
                        index + 2
                    ));
                }
                seen_identity.push(row_id.clone());
                check_path_pin(
                    &value,
                    root,
                    &row_id,
                    &["docs/specs/", "docs/design/", "experimental/"],
                )?;
                report.identity_rows += 1;
            }
            "profile" => {
                if !profile_rows().contains(&row_id) {
                    return Err(format!(
                        "contract-profile line {}: unknown profile row `{row_id}`",
                        index + 2
                    ));
                }
                if seen_profile.contains(&row_id) {
                    return Err(format!(
                        "contract-profile line {}: duplicate profile row `{row_id}`",
                        index + 2
                    ));
                }
                seen_profile.push(row_id.clone());
                report.profile_rows += 1;
            }
            _ => {
                if !resource_rows().contains(&row_id) {
                    return Err(format!(
                        "contract-profile line {}: unknown resource row `{row_id}`",
                        index + 2
                    ));
                }
                if seen_resource.contains(&row_id) {
                    return Err(format!(
                        "contract-profile line {}: duplicate resource row `{row_id}`",
                        index + 2
                    ));
                }
                let n: u64 = value.parse().map_err(|_| {
                    format!("contract-profile line {}: resource `{row_id}` value `{value}` is not a positive u64", index + 2)
                })?;
                if n == 0 {
                    return Err(format!(
                        "contract-profile line {}: resource `{row_id}` value must be > 0",
                        index + 2
                    ));
                }
                seen_resource.push(row_id.clone());
                report.resource_rows += 1;
            }
        }
    }
    let missing_identity: Vec<String> = identity_rows()
        .into_iter()
        .filter(|id| !seen_identity.contains(id))
        .collect();
    let missing_profile: Vec<String> = profile_rows()
        .into_iter()
        .filter(|id| !seen_profile.contains(id))
        .collect();
    let missing_resource: Vec<String> = resource_rows()
        .into_iter()
        .filter(|id| !seen_resource.contains(id))
        .collect();
    if !missing_identity.is_empty() {
        return Err(format!(
            "contract-profile missing identity row(s): {} — 删除A已合入身份不能缩小分母",
            missing_identity.join(", ")
        ));
    }
    if !missing_profile.is_empty() {
        return Err(format!(
            "contract-profile missing profile row(s): {}",
            missing_profile.join(", ")
        ));
    }
    if !missing_resource.is_empty() {
        return Err(format!(
            "contract-profile missing resource row(s): {}",
            missing_resource.join(", ")
        ));
    }
    for (row_id, expected) in frozen_profile_values() {
        let Some(line) = text.lines().find(|l| l.starts_with(&format!("{row_id}\t"))) else {
            return Err(format!(
                "contract-profile: frozen profile row `{row_id}` absent"
            ));
        };
        let fields: Vec<&str> = line.split('\t').collect();
        if fields[3] != expected {
            return Err(format!(
                "contract-profile: profile `{row_id}` value `{}` != frozen `{expected}`",
                fields[3]
            ));
        }
    }
    Ok(report)
}

fn validate_requirements(text: &str, root: &Path, kind: &str) -> Result<usize, String> {
    let label = if kind == "CS" {
        "cs-requirements"
    } else {
        "cl-requirements"
    };
    let mut lines = text.lines();
    if lines.next() != Some(REQ_HEADER) {
        return Err(format!(
            "{label} schema header mismatch: expected `{REQ_HEADER}`"
        ));
    }
    let required: Vec<String> = if kind == "CS" {
        required_cs_rows()
    } else {
        required_cl_rows()
    };
    let mut seen = Vec::new();
    let mut count = 0usize;
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            return Err(format!("{label} line {}: empty row", index + 2));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 12 {
            return Err(format!(
                "{label} line {}: expected 12 tab-separated fields, got {}",
                index + 2,
                fields.len()
            ));
        }
        let row_id = fields[0].to_string();
        let row_kind = fields[1];
        let family = fields[2];
        let requirement = fields[3];
        let producer_chain = fields[4];
        let consumer = fields[5];
        let phase = fields[6];
        let disposition = fields[7];
        let owner = fields[8];
        let source_ref = fields[9];
        let evidence_state = fields[10];
        let evidence_ref = fields[11];
        if !required.contains(&row_id) {
            return Err(format!(
                "{label} line {}: unknown row `{row_id}` — not part of the frozen denominator",
                index + 2
            ));
        }
        if seen.contains(&row_id) {
            return Err(format!(
                "{label} line {}: duplicate row `{row_id}`",
                index + 2
            ));
        }
        if row_kind != kind {
            return Err(format!(
                "{label} line {}: row `{row_id}` kind `{row_kind}` != expected `{kind}`",
                index + 2
            ));
        }
        if family.trim().is_empty() || requirement.trim().is_empty() {
            return Err(format!(
                "{label} line {}: row `{row_id}` family/requirement empty",
                index + 2
            ));
        }
        if producer_chain.trim().is_empty() || consumer.trim().is_empty() {
            return Err(format!(
                "{label} line {}: row `{row_id}` producer_chain/consumer empty — 每行必须标语法→Auto语义→MIR→runtime→consumer",
                index + 2
            ));
        }
        if !PHASES.contains(&phase) {
            return Err(format!(
                "{label} line {}: row `{row_id}` phase `{phase}` not in {PHASES:?}",
                index + 2
            ));
        }
        if !DISPOSITIONS.contains(&disposition) {
            return Err(format!(
                "{label} line {}: row `{row_id}` disposition `{disposition}` 非法（required/NA 豁免必须拒绝，NA 不在词表）",
                index + 2
            ));
        }
        if owner.trim().is_empty() {
            return Err(format!(
                "{label} line {}: row `{row_id}` has no owner（无补齐owner须拒绝）",
                index + 2
            ));
        }
        if source_ref.trim().is_empty() {
            return Err(format!(
                "{label} line {}: row `{row_id}` has no source_ref",
                index + 2
            ));
        }
        if disposition == "deferred" && phase != "stage" {
            return Err(format!(
                "{label} line {}: row `{row_id}` deferred only allowed at stage phase（完整支持不能以pending代通过）",
                index + 2
            ));
        }
        if !EVIDENCE_STATES.contains(&evidence_state) {
            return Err(format!(
                "{label} line {}: row `{row_id}` evidence_state `{evidence_state}` not in {EVIDENCE_STATES:?}",
                index + 2
            ));
        }
        match evidence_state {
            "landed" => {
                check_path_pin(&evidence_ref, root, &row_id, RECEIPT_ROOTS.as_slice())?;
            }
            "inflight" | "pending" => {
                if !evidence_ref.is_empty() {
                    return Err(format!(
                        "{label} line {}: row `{row_id}` is {evidence_state} but cites a receipt `{evidence_ref}` (evidence inflation)",
                        index + 2
                    ));
                }
            }
            _ => unreachable!(),
        }
        seen.push(row_id);
        count += 1;
    }
    let missing: Vec<String> = required
        .into_iter()
        .filter(|id| !seen.contains(id))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{label} missing {} frozen denominator row(s): {} — 漏行不能缩小分母",
            missing.len(),
            missing.join(", ")
        ));
    }
    Ok(count)
}

fn validate_subject_map(text: &str) -> Result<usize, String> {
    let mut lines = text.lines();
    if lines.next() != Some(SUBJECT_HEADER) {
        return Err(format!(
            "subject-map schema header mismatch: expected `{SUBJECT_HEADER}`"
        ));
    }
    let required = required_subject_rows();
    let mut seen = Vec::new();
    let mut count = 0usize;
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            return Err(format!("subject-map line {}: empty row", index + 2));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!(
                "subject-map line {}: expected 5 tab-separated fields, got {}",
                index + 2,
                fields.len()
            ));
        }
        let row_id = fields[0].to_string();
        let module_path = fields[1];
        let authority = fields[2];
        let retention = fields[3];
        let note = fields[4];
        if !required.contains(&row_id) {
            return Err(format!(
                "subject-map line {}: unknown row `{row_id}`",
                index + 2
            ));
        }
        if seen.contains(&row_id) {
            return Err(format!(
                "subject-map line {}: duplicate row `{row_id}`",
                index + 2
            ));
        }
        if module_path.trim().is_empty() || note.trim().is_empty() {
            return Err(format!(
                "subject-map line {}: row `{row_id}` module_path/note empty",
                index + 2
            ));
        }
        if !SUBJECT_AUTHORITIES.contains(&authority) {
            return Err(format!(
                "subject-map line {}: row `{row_id}` subject_authority `{authority}` not in {SUBJECT_AUTHORITIES:?}",
                index + 2
            ));
        }
        if retention.trim().is_empty() {
            return Err(format!(
                "subject-map line {}: row `{row_id}` rust_retention empty",
                index + 2
            ));
        }
        seen.push(row_id);
        count += 1;
    }
    for (row_id, expected) in frozen_subject_authorities() {
        let Some(line) = text.lines().find(|l| l.starts_with(&format!("{row_id}\t"))) else {
            return Err(format!("subject-map: frozen row `{row_id}` absent"));
        };
        let fields: Vec<&str> = line.split('\t').collect();
        if fields[2] != expected {
            return Err(format!(
                "subject-map: `{row_id}` subject_authority `{}` != frozen `{expected}` — 把主体源码 solver 列成 backend helper 必须拒绝",
                fields[2]
            ));
        }
    }
    let missing: Vec<String> = required
        .into_iter()
        .filter(|id| !seen.contains(id))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "subject-map missing row(s): {}",
            missing.join(", ")
        ));
    }
    Ok(count)
}

fn validate_bridge_contracts(text: &str) -> Result<usize, String> {
    let mut lines = text.lines();
    if lines.next() != Some(BRIDGE_HEADER) {
        return Err(format!(
            "bridge-contracts schema header mismatch: expected `{BRIDGE_HEADER}`"
        ));
    }
    let required = required_bridge_rows();
    let mut seen = Vec::new();
    let mut count = 0usize;
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            return Err(format!("bridge-contracts line {}: empty row", index + 2));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!(
                "bridge-contracts line {}: expected 5 tab-separated fields, got {}",
                index + 2,
                fields.len()
            ));
        }
        let row_id = fields[0].to_string();
        let seam = fields[1];
        let dimension = fields[2];
        let note = fields[4];
        if !required.contains(&row_id) {
            return Err(format!(
                "bridge-contracts line {}: unknown row `{row_id}`",
                index + 2
            ));
        }
        if seen.contains(&row_id) {
            return Err(format!(
                "bridge-contracts line {}: duplicate row `{row_id}`",
                index + 2
            ));
        }
        if seam.trim().is_empty() || dimension.trim().is_empty() || note.trim().is_empty() {
            return Err(format!(
                "bridge-contracts line {}: row `{row_id}` seam/dimension/note empty",
                index + 2
            ));
        }
        seen.push(row_id);
        count += 1;
    }
    for (row_id, expected) in frozen_bridge_values() {
        let Some(line) = text.lines().find(|l| l.starts_with(&format!("{row_id}\t"))) else {
            return Err(format!("bridge-contracts: frozen row `{row_id}` absent"));
        };
        let fields: Vec<&str> = line.split('\t').collect();
        if fields[3] != expected {
            return Err(format!(
                "bridge-contracts: `{row_id}` value `{}` != frozen `{expected}` — 信任边界声明被篡改必须拒绝",
                fields[3]
            ));
        }
    }
    let missing: Vec<String> = required
        .into_iter()
        .filter(|id| !seen.contains(id))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "bridge-contracts missing row(s): {}",
            missing.join(", ")
        ));
    }
    Ok(count)
}

fn validate_supply_matrix(text: &str) -> Result<usize, String> {
    let mut lines = text.lines();
    if lines.next() != Some(SUPPLY_HEADER) {
        return Err(format!(
            "supply-matrix schema header mismatch: expected `{SUPPLY_HEADER}`"
        ));
    }
    let required = required_supply_rows();
    let mut seen = Vec::new();
    let mut count = 0usize;
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            return Err(format!("supply-matrix line {}: empty row", index + 2));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 8 {
            return Err(format!(
                "supply-matrix line {}: expected 8 tab-separated fields, got {}",
                index + 2,
                fields.len()
            ));
        }
        let row_id = fields[0].to_string();
        if !required.contains(&row_id) {
            return Err(format!(
                "supply-matrix line {}: unknown row `{row_id}`",
                index + 2
            ));
        }
        if seen.contains(&row_id) {
            return Err(format!(
                "supply-matrix line {}: duplicate row `{row_id}`",
                index + 2
            ));
        }
        // V-05: 不允许凭 owner 名字空置交付条件——input/oracle/
        // closing_condition 必须非空。
        if fields.iter().skip(1).any(|f| f.trim().is_empty()) {
            return Err(format!(
                "supply-matrix line {}: row `{row_id}` 存在空列 — 必须落实际实现任务/输入/oracle/正式交付条件，不能仅列 owner 名字",
                index + 2
            ));
        }
        seen.push(row_id);
        count += 1;
    }
    let missing: Vec<String> = required
        .into_iter()
        .filter(|id| !seen.contains(id))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "supply-matrix missing {} row(s): {} — 删除 BLAKE3/模块包锁/供应行不能缩小分母",
            missing.len(),
            missing.join(", ")
        ));
    }
    Ok(count)
}

fn validate_stage_contract(text: &str) -> Result<usize, String> {
    let mut lines = text.lines();
    if lines.next() != Some(STAGE_HEADER) {
        return Err(format!(
            "stage-test-contract schema header mismatch: expected `{STAGE_HEADER}`"
        ));
    }
    let required = required_stage_rows();
    let mut seen = Vec::new();
    let mut count = 0usize;
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            return Err(format!("stage-test-contract line {}: empty row", index + 2));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err(format!(
                "stage-test-contract line {}: expected 4 tab-separated fields, got {}",
                index + 2,
                fields.len()
            ));
        }
        let row_id = fields[0].to_string();
        let dimension = fields[1];
        let note = fields[3];
        if !required.contains(&row_id) {
            return Err(format!(
                "stage-test-contract line {}: unknown row `{row_id}`",
                index + 2
            ));
        }
        if seen.contains(&row_id) {
            return Err(format!(
                "stage-test-contract line {}: duplicate row `{row_id}`",
                index + 2
            ));
        }
        if dimension.trim().is_empty() || note.trim().is_empty() {
            return Err(format!(
                "stage-test-contract line {}: row `{row_id}` dimension/note empty",
                index + 2
            ));
        }
        seen.push(row_id);
        count += 1;
    }
    for (row_id, expected) in frozen_stage_values() {
        let Some(line) = text.lines().find(|l| l.starts_with(&format!("{row_id}\t"))) else {
            return Err(format!("stage-test-contract: frozen row `{row_id}` absent"));
        };
        let fields: Vec<&str> = line.split('\t').collect();
        if fields[2] != expected {
            return Err(format!(
                "stage-test-contract: `{row_id}` value `{}` != frozen `{expected}` — 阶段/跨仓交接合同被篡改必须拒绝",
                fields[2]
            ));
        }
    }
    let missing: Vec<String> = required
        .into_iter()
        .filter(|id| !seen.contains(id))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "stage-test-contract missing row(s): {}",
            missing.join(", ")
        ));
    }
    Ok(count)
}

fn read_contract(root: &Path, file: &str) -> Result<String, String> {
    let path = root.join(DATA_DIR_REL).join(file);
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Validate the whole frozen contract set. All identity pins must resolve
/// at their pinned sha16; every denominator row must be present exactly
/// once; vocabulary/ownership/trust rules must hold. This is contract
/// admission (V-01..V-06), not a bootstrap PASS.
pub fn validate_all(workspace_root: &Path) -> Result<AdmissionReport, String> {
    let profile = validate_profile(
        &read_contract(workspace_root, "contract-profile.tsv")?,
        workspace_root,
    )?;
    let cs = validate_requirements(
        &read_contract(workspace_root, "cs-requirements.tsv")?,
        workspace_root,
        "CS",
    )?;
    let cl = validate_requirements(
        &read_contract(workspace_root, "cl-requirements.tsv")?,
        workspace_root,
        "CL",
    )?;
    let subject = validate_subject_map(&read_contract(workspace_root, "subject-map.tsv")?)?;
    let bridge =
        validate_bridge_contracts(&read_contract(workspace_root, "bridge-contracts.tsv")?)?;
    let supply = validate_supply_matrix(&read_contract(workspace_root, "supply-matrix.tsv")?)?;
    let stage =
        validate_stage_contract(&read_contract(workspace_root, "stage-test-contract.tsv")?)?;
    Ok(AdmissionReport {
        identity_rows: profile.identity_rows,
        profile_rows: profile.profile_rows,
        resource_rows: profile.resource_rows,
        subject_rows: subject,
        bridge_rows: bridge,
        cs_rows: cs,
        cl_rows: cl,
        supply_rows: supply,
        stage_rows: stage,
    })
}

/// Locate the workspace root by walking up from this crate's manifest
/// directory until the contract data dir resolves.
pub fn workspace_root() -> Result<PathBuf, String> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join(DATA_DIR_REL).is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(format!(
                "workspace root not found above {}: data dir `{DATA_DIR_REL}` missing",
                env!("CARGO_MANIFEST_DIR")
            ));
        }
    }
}

/// Load and validate the tracked contract set (the production entry).
pub fn validate_tracked_contracts() -> Result<(AdmissionReport, PathBuf), String> {
    let root = workspace_root()?;
    let report = validate_all(&root)?;
    Ok((report, root))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn real_text(file: &str) -> String {
        let root = workspace_root().expect("workspace root");
        std::fs::read_to_string(root.join(DATA_DIR_REL).join(file))
            .unwrap_or_else(|e| panic!("{file} unreadable: {e}"))
    }

    fn root() -> PathBuf {
        workspace_root().expect("workspace root")
    }

    fn all_text() -> Vec<(String, String)> {
        let files = [
            "contract-profile.tsv",
            "cs-requirements.tsv",
            "cl-requirements.tsv",
            "subject-map.tsv",
            "bridge-contracts.tsv",
            "supply-matrix.tsv",
            "stage-test-contract.tsv",
        ];
        files
            .iter()
            .map(|f| (f.to_string(), real_text(f)))
            .collect()
    }

    /// V-01: the complete legal contract set admits — every identity pin
    /// resolves at its sha16, every denominator row is present. This is
    /// contract admission, not a bootstrap PASS.
    #[test]
    fn v01_complete_legal_contract_set_admits() {
        let (report, path) = validate_tracked_contracts()
            .unwrap_or_else(|e| panic!("legal contract set invalid: {e}"));
        assert_eq!(report.identity_rows, identity_rows().len());
        assert_eq!(report.profile_rows, profile_rows().len());
        assert_eq!(report.resource_rows, resource_rows().len());
        assert_eq!(report.cs_rows, required_cs_rows().len());
        assert_eq!(report.cl_rows, required_cl_rows().len());
        assert_eq!(report.subject_rows, required_subject_rows().len());
        assert_eq!(report.bridge_rows, required_bridge_rows().len());
        assert_eq!(report.supply_rows, required_supply_rows().len());
        assert_eq!(report.stage_rows, required_stage_rows().len());
        assert!(path.is_file() || path.is_dir());
    }

    /// V-02: deleting an A-merged identity row, dropping one CL line, or
    /// flipping a required row to NA must each be rejected with the row
    /// named.
    #[test]
    fn v02_missing_identity_missing_cl_row_and_fake_na_are_red() {
        // 删除 A 已合入身份（a-receipt 行）。
        let (file, text) = all_text()
            .into_iter()
            .find(|(f, _)| f == "contract-profile.tsv")
            .expect("profile file");
        let mutated = text
            .lines()
            .filter(|l| !l.starts_with("profile.identity.a-receipt\t"))
            .collect::<Vec<_>>()
            .join("\n");
        let err = validate_profile(&mutated, &root()).expect_err("missing identity must be RED");
        assert!(err.contains("profile.identity.a-receipt"), "{err}");
        assert!(err.contains("删除A已合入身份"), "{err}");

        // 漏一条 CL 行。
        let (_, cl_text) = all_text()
            .into_iter()
            .find(|(f, _)| f == "cl-requirements.tsv")
            .expect("cl file");
        let mutated = cl_text
            .lines()
            .filter(|l| !l.starts_with("CL-403\t"))
            .collect::<Vec<_>>()
            .join("\n");
        let err =
            validate_requirements(&mutated, &root(), "CL").expect_err("missing CL row must be RED");
        assert!(err.contains("CL-403"), "{err}");

        // 将 required 改 NA（fake NA）。
        let fake_na = cl_text.replacen("\trequired\t", "\tNA\t", 1);
        assert_ne!(&fake_na, &cl_text, "mutation must hit a disposition");
        let err = validate_requirements(&fake_na, &root(), "CL").expect_err("NA must be RED");
        assert!(err.contains("NA"), "{err}");
    }

    /// V-03: listing a Rust source solver as a backend helper, or a schema
    /// carrying checked=true, must be rejected; the legal MIR-machine
    /// backend retention still admits.
    #[test]
    fn v03_subject_authority_flip_and_checked_flag_are_red() {
        // 把 subject.semantics（auto 语义权威）flip 成 rust-retained。
        let (_, text) = all_text()
            .into_iter()
            .find(|(f, _)| f == "subject-map.tsv")
            .expect("subject file");
        // 直接改 authority 列（第 3 列）：定位 subject.semantics 行。
        let Some(semantics_line) = text.lines().find(|l| l.starts_with("subject.semantics\t"))
        else {
            panic!("subject.semantics exists");
        };
        let flipped_authority = semantics_line.replacen("\tauto\t", "\trust-retained\t", 1);
        assert_ne!(&flipped_authority, semantics_line);
        let mutated = text.replace(semantics_line, &flipped_authority);
        let err = validate_subject_map(&mutated).expect_err("authority flip must be RED");
        assert!(err.contains("subject.semantics"), "{err}");
        assert!(err.contains("backend helper"), "{err}");

        // schema 带 checked=true（bridge checked-flag 从 reject 改成 trust）。
        let (_, bridge) = all_text()
            .into_iter()
            .find(|(f, _)| f == "bridge-contracts.tsv")
            .expect("bridge file");
        let mutated = bridge.replace(
            "bridge.mir-backend.checked-flag\tmir-backend\tserialized-checked\treject",
            "bridge.mir-backend.checked-flag\tmir-backend\tserialized-checked\ttrust",
        );
        assert_ne!(&mutated, &bridge);
        let err = validate_bridge_contracts(&mutated).expect_err("checked=true must be RED");
        assert!(err.contains("bridge.mir-backend.checked-flag"), "{err}");

        // 合法 MIR 机器后端保留项（subject.native = auto-driver +
        // cranelift-linker-ffi）仍 admit。
        validate_all(&root()).expect("legal retention still admits");
    }

    /// V-04: 删除一条 CL-required 行 → 拒绝并返回该行 ID；恢复同行为
    /// 伪 NA → 拒绝；最后恢复 required 与实际证据 → 仅当全部身份有效才 admit。
    #[test]
    fn v04_restore_required_with_real_evidence_admits() {
        let (_, cl_text) = all_text()
            .into_iter()
            .find(|(f, _)| f == "cl-requirements.tsv")
            .expect("cl file");
        let Some(line) = cl_text.lines().find(|l| l.starts_with("CL-304\t")) else {
            panic!("CL-304 exists");
        };
        // 1) 删除恰好一行。
        let deleted = cl_text
            .lines()
            .filter(|l| *l != line)
            .collect::<Vec<_>>()
            .join("\n");
        let err =
            validate_requirements(&deleted, &root(), "CL").expect_err("deleted row must be RED");
        assert!(err.contains("CL-304"), "{err}");
        // 2) 恢复该行并伪置 NA（原位替换，不制造重复行）。
        let restored_na = cl_text.replace(line, &line.replace("\trequired\t", "\tNA\t"));
        assert_ne!(&restored_na, &cl_text, "mutation must flip the disposition");
        let err = validate_requirements(&restored_na, &root(), "CL").expect_err("NA must be RED");
        assert!(err.contains("NA"), "{err}");
        // 3) 恢复 required 与实际证据：真实矩阵（全部 pending、无伪证）admit。
        validate_all(&root()).expect("restored required with honest evidence admits");
    }

    /// V-05 (T-09): 删除 BLAKE3 供应行、删除模块包锁行、删除一个 AC
    /// 适用 CL 行、交换 host/target 声明——准入必须列具体缺口拒绝；完整
    /// 合法合同通过。
    #[test]
    fn v05_supply_and_platform_mutations_are_red() {
        let (_, supply) = all_text()
            .into_iter()
            .find(|(f, _)| f == "supply-matrix.tsv")
            .expect("supply file");
        // 删除 BLAKE3（supply.f03.hash-codec）。
        let mutated = supply
            .lines()
            .filter(|l| !l.starts_with("supply.f03.hash-codec\t"))
            .collect::<Vec<_>>()
            .join("\n");
        let err = validate_supply_matrix(&mutated).expect_err("missing BLAKE3 supply must be RED");
        assert!(err.contains("supply.f03.hash-codec"), "{err}");
        // 删除模块包锁（supply.f06.package-lock）。
        let mutated = supply
            .lines()
            .filter(|l| !l.starts_with("supply.f06.package-lock\t"))
            .collect::<Vec<_>>()
            .join("\n");
        let err =
            validate_supply_matrix(&mutated).expect_err("missing package-lock supply must be RED");
        assert!(err.contains("supply.f06.package-lock"), "{err}");
        // 删除一个 AC 适用 CL 行。
        let (_, cl_text) = all_text()
            .into_iter()
            .find(|(f, _)| f == "cl-requirements.tsv")
            .expect("cl file");
        let mutated = cl_text
            .lines()
            .filter(|l| !l.starts_with("CL-501\t"))
            .collect::<Vec<_>>()
            .join("\n");
        let err =
            validate_requirements(&mutated, &root(), "CL").expect_err("missing CL row must be RED");
        assert!(err.contains("CL-501"), "{err}");
        // 交换 host/target 声明。
        let (_, profile) = all_text()
            .into_iter()
            .find(|(f, _)| f == "contract-profile.tsv")
            .expect("profile file");
        let swapped = profile.replace(
            "profile.host\tprofile\thost\twindows-x64",
            "profile.host\tprofile\thost\twindows-x64-swapped",
        );
        assert_ne!(&swapped, &profile);
        let err = validate_profile(&swapped, &root()).expect_err("host swap must be RED");
        assert!(err.contains("profile.host"), "{err}");
        // 完整合法合同通过。
        validate_all(&root()).expect("complete legal contract passes");
    }

    /// V-06 (T-10): 把 auto-down 目录当语言 TestCase 生产者，或仅旧 Rust
    /// 组绿未跑 Auto 阶段 driver 的报告——必须拒绝；阶段与整机来源齐才接受。
    #[test]
    fn v06_stage_test_producer_and_evidence_policy_are_red() {
        let (_, stage) = all_text()
            .into_iter()
            .find(|(f, _)| f == "stage-test-contract.tsv")
            .expect("stage file");
        // auto-down 当 TestCase 生产者。
        let mutated = stage.replace(
            "stage-test.case-producer\tlanguage-testcase-owner\tauto-lang",
            "stage-test.case-producer\tlanguage-testcase-owner\tauto-down",
        );
        assert_ne!(&mutated, &stage);
        let err = validate_stage_contract(&mutated).expect_err("auto-down as producer must be RED");
        assert!(err.contains("stage-test.case-producer"), "{err}");
        // 仅旧 Rust 组绿充 Auto 阶段已测。
        let mutated = stage.replace(
            "stage-test.evidence-policy\tevidence\tauto-stage-run-required",
            "stage-test.evidence-policy\tevidence\trust-rust-groups-only",
        );
        assert_ne!(&mutated, &stage);
        let err =
            validate_stage_contract(&mutated).expect_err("rust-groups-only report must be RED");
        assert!(err.contains("stage-test.evidence-policy"), "{err}");
        // 阶段与整机来源齐才接受。
        validate_all(&root()).expect("complete stage contract accepts");
    }
}
