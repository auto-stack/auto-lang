//! PLAN-985 T-A01: the A/B obligation-matrix ledger
//! (`autolang.ab-obligations/1`) — the frozen 959 denominator rows
//! (L/S/A/B/C/D/E/F/K tables) plus the Design56 FFI/LIB delta rows,
//! each classified with phase (A/B), disposition
//! (inherited / named-replacement / internal-retire / product-B),
//! successor owner and the best current evidence binding.
//!
//! Validation families (V-A-MAP, frozen by PLAN-985 §6):
//! - **missing row**: every frozen denominator row must appear exactly
//!   once — dropping a row is a RED validation error, never a smaller
//!   denominator;
//! - **duplicate row**: the same obligation counted twice is RED;
//! - **wrong source**: a `landed` row must cite a real reviewed receipt
//!   (a file under `docs/specs/` or `docs/plans/archive/`) whose current
//!   sha256 matches the pinned prefix — a stale or forged receipt is
//!   RED, and a test binary's own green output is NOT a receipt;
//! - **vocabulary/ownership**: phase/disposition/evidence-state values
//!   and a non-empty successor owner are enforced — an unowned or
//!   ill-typed row is RED;
//! - **pending-honesty**: a `pending`/`inflight` row citing a receipt is
//!   RED (no evidence inflation).
//!
//! The admission-layer families named by V-A-MAP (stale binary, emit
//! masquerading as run, fake worker PASS, timeout masquerading as a
//! language negative) are owned by the case-admission spine — the
//! `ac` adapter's parent-side deadline/kill discipline and schema
//! admission, and the `ac_av`/probe parent-side judging where the
//! worker never sees expected outcomes; T-A02/T-A03 bind them per
//! capability when evidence is registered.

use std::path::{Path, PathBuf};

/// Ledger schema identity (wire/format version).
pub const LEDGER_SCHEMA: &str = "autolang.ab-obligations/1";

/// Repo-relative location of the tracked matrix.
pub const LEDGER_REL_PATH: &str =
    "crates/auto-lang/test/cases/ace-m2/m2-cert-985/ab-obligations.tsv";

/// The exact frozen header (column order is part of the schema).
pub const HEADER: &str =
    "row_id\tfamily\tobligation\tsuccessor\tphase\tdisposition\tevidence_state\tevidence_ref";

pub const PHASES: [&str; 3] = ["A", "B", "AB"];
pub const DISPOSITIONS: [&str; 4] = [
    "inherited",
    "named-replacement",
    "internal-retire",
    "product-B",
];
pub const EVIDENCE_STATES: [&str; 3] = ["landed", "inflight", "pending"];

/// Receipt roots: a `landed` evidence citation must live under one of
/// these (reviewed knowledge / archived plan receipts), never in a
/// source or build tree.
pub const RECEIPT_ROOTS: [&str; 2] = ["docs/specs/", "docs/plans/archive/"];

/// The frozen denominator row set (959 L-01..L-49, S-01..S-25,
/// A-01..A-12, B-01..B-11, C-01..C-11, D-01..D-09, E-01..E-06,
/// F-01..F-15, K-01..K-08) plus the Design56 delta rows
/// (FFI-M2-01..08, LIB-M2-01..08).
pub fn required_rows() -> Vec<String> {
    fn ids(prefix: &str, from: u32, to: u32) -> Vec<String> {
        (from..=to).map(|i| format!("{prefix}{i:02}")).collect()
    }
    let mut rows = Vec::new();
    rows.extend(ids("L-", 1, 49));
    rows.extend(ids("S-", 1, 25));
    rows.extend(ids("FFI-M2-", 1, 8));
    rows.extend(ids("LIB-M2-", 1, 8));
    rows.extend(ids("A-", 1, 12));
    rows.extend(ids("B-", 1, 11));
    rows.extend(ids("C-", 1, 11));
    rows.extend(ids("D-", 1, 9));
    rows.extend(ids("E-", 1, 6));
    rows.extend(ids("F-", 1, 15));
    rows.extend(ids("K-", 1, 8));
    rows
}

fn expected_family(row_id: &str) -> Result<&'static str, String> {
    if row_id.starts_with("L-") {
        Ok("L")
    } else if row_id.starts_with("S-") {
        Ok("S")
    } else if row_id.starts_with("FFI-M2-") {
        Ok("FFI")
    } else if row_id.starts_with("LIB-M2-") {
        Ok("LIB")
    } else if row_id.starts_with("A-") {
        Ok("AENTRY")
    } else if row_id.starts_with("B-") {
        Ok("BENTRY")
    } else if let Some(c) = row_id.chars().next() {
        match c {
            'C' | 'D' | 'E' | 'F' | 'K' => Ok("PRODUCT"),
            _ => Err(format!("unknown row family for `{row_id}`")),
        }
    } else {
        Err("empty row id".to_string())
    }
}

/// One parsed matrix row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerRow {
    pub row_id: String,
    pub family: String,
    pub obligation: String,
    pub successor: String,
    pub phase: String,
    pub disposition: String,
    pub evidence_state: String,
    pub evidence_ref: String,
}

/// Validation summary of a well-formed ledger.
#[derive(Debug, Clone, Default)]
pub struct LedgerReport {
    pub rows: usize,
    pub landed: usize,
    pub inflight: usize,
    pub pending: usize,
}

fn sha16_hex(bytes: &[u8]) -> String {
    // Local SHA-256 is overkill to reimplement; reuse the contract crate.
    auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(bytes))
}

/// Validate a ledger text against the frozen denominator. Structural,
/// completeness, ownership and evidence checks all apply; the first
/// violation wins with a named error (controls below prove each family
/// actually turns RED on a mutated copy).
pub fn validate_ledger(text: &str, workspace_root: &Path) -> Result<LedgerReport, String> {
    let mut lines = text.lines();
    let header = lines.next().ok_or("empty ledger: no header")?;
    if header != HEADER {
        return Err(format!("schema header mismatch: expected `{HEADER}`"));
    }
    let mut seen: Vec<String> = Vec::new();
    let mut report = LedgerReport::default();
    for (index, line) in lines.enumerate() {
        if line.trim().is_empty() {
            return Err(format!("line {}: empty row", index + 2));
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 8 {
            return Err(format!(
                "line {}: expected 8 tab-separated fields, got {}",
                index + 2,
                fields.len()
            ));
        }
        let row = LedgerRow {
            row_id: fields[0].to_string(),
            family: fields[1].to_string(),
            obligation: fields[2].to_string(),
            successor: fields[3].to_string(),
            phase: fields[4].to_string(),
            disposition: fields[5].to_string(),
            evidence_state: fields[6].to_string(),
            evidence_ref: fields[7].to_string(),
        };
        let required = required_rows();
        if !required.contains(&row.row_id) {
            return Err(format!(
                "line {}: unknown row `{}` — not part of the frozen denominator",
                index + 2,
                row.row_id
            ));
        }
        if seen.contains(&row.row_id) {
            return Err(format!(
                "line {}: duplicate row `{}` — an obligation counted twice",
                index + 2,
                row.row_id
            ));
        }
        let expected = expected_family(&row.row_id)?;
        if row.family != expected && expected != "PRODUCT" {
            return Err(format!(
                "line {}: row `{}` family `{}` != expected `{expected}`",
                index + 2,
                row.row_id,
                row.family
            ));
        }
        if row.obligation.trim().is_empty() {
            return Err(format!(
                "line {}: row `{}` has no obligation text",
                index + 2,
                row.row_id
            ));
        }
        if row.successor.trim().is_empty() {
            return Err(format!(
                "line {}: row `{}` has no successor owner (无owner row)",
                index + 2,
                row.row_id
            ));
        }
        if !PHASES.contains(&row.phase.as_str()) {
            return Err(format!(
                "line {}: row `{}` phase `{}` not in {PHASES:?}",
                index + 2,
                row.row_id,
                row.phase
            ));
        }
        if !DISPOSITIONS.contains(&row.disposition.as_str()) {
            return Err(format!(
                "line {}: row `{}` disposition `{}` not in {DISPOSITIONS:?}",
                index + 2,
                row.row_id,
                row.disposition
            ));
        }
        if !EVIDENCE_STATES.contains(&row.evidence_state.as_str()) {
            return Err(format!(
                "line {}: row `{}` evidence_state `{}` not in {EVIDENCE_STATES:?}",
                index + 2,
                row.row_id,
                row.evidence_state
            ));
        }
        match row.evidence_state.as_str() {
            "landed" => {
                let Some((path, pinned)) = row.evidence_ref.rsplit_once(':') else {
                    return Err(format!(
                        "line {}: row `{}` landed evidence `{}` is not `path:sha16`",
                        index + 2,
                        row.row_id,
                        row.evidence_ref
                    ));
                };
                if !RECEIPT_ROOTS.iter().any(|root| path.starts_with(root)) {
                    return Err(format!(
                        "line {}: row `{}` receipt `{path}` outside reviewed receipt roots {RECEIPT_ROOTS:?}",
                        index + 2,
                        row.row_id
                    ));
                }
                let full = workspace_root.join(path);
                let bytes = std::fs::read(&full).map_err(|e| {
                    format!(
                        "line {}: row `{}` receipt `{path}` unreadable: {e}",
                        index + 2,
                        row.row_id
                    )
                })?;
                let actual = sha16_hex(&bytes);
                if !actual.starts_with(pinned) {
                    return Err(format!(
                        "line {}: row `{}` receipt `{path}` sha256 mismatch: pinned `{pinned}` actual `{actual}` (stale or forged receipt)",
                        index + 2,
                        row.row_id
                    ));
                }
                report.landed += 1;
            }
            "inflight" => {
                if !row.evidence_ref.is_empty() {
                    return Err(format!(
                        "line {}: row `{}` is inflight but cites a receipt `{}` (evidence inflation)",
                        index + 2,
                        row.row_id,
                        row.evidence_ref
                    ));
                }
                report.inflight += 1;
            }
            _ => {
                if !row.evidence_ref.is_empty() {
                    return Err(format!(
                        "line {}: row `{}` is pending but cites a receipt `{}` (evidence inflation)",
                        index + 2,
                        row.row_id,
                        row.evidence_ref
                    ));
                }
                report.pending += 1;
            }
        }
        seen.push(row.row_id);
        report.rows += 1;
    }
    // Completeness: every frozen denominator row exactly once.
    let missing: Vec<String> = required_rows()
        .into_iter()
        .filter(|id| !seen.contains(id))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "missing {} frozen denominator row(s): {} — 删行不能缩小分母",
            missing.len(),
            missing.join(", ")
        ));
    }
    Ok(report)
}

/// Locate the workspace root by walking up from this crate's manifest
/// directory until the tracked ledger path resolves (same discipline as
/// the `ac` adapter's worker location).
pub fn workspace_root() -> Result<PathBuf, String> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join(LEDGER_REL_PATH).is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(format!(
                "workspace root not found above {}: ledger `{LEDGER_REL_PATH}` missing",
                env!("CARGO_MANIFEST_DIR")
            ));
        }
    }
}

/// Load and validate the tracked matrix (the production entry).
pub fn validate_tracked_ledger() -> Result<(LedgerReport, PathBuf), String> {
    let root = workspace_root()?;
    let path = root.join(LEDGER_REL_PATH);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let report = validate_ledger(&text, &root)?;
    Ok((report, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn real_text() -> String {
        let root = workspace_root().expect("workspace root");
        std::fs::read_to_string(root.join(LEDGER_REL_PATH))
            .unwrap_or_else(|e| panic!("ledger unreadable: {e}"))
    }

    fn root() -> PathBuf {
        workspace_root().expect("workspace root")
    }

    /// The REAL matrix validates: every frozen row present exactly once,
    /// all citations resolve at their pinned sha16.
    #[test]
    fn p985_real_matrix_validates() {
        let (report, path) =
            validate_tracked_ledger().unwrap_or_else(|e| panic!("real ledger invalid: {e}"));
        assert_eq!(report.rows, required_rows().len(), "frozen row total");
        assert!(report.landed > 0 && report.pending > 0);
        assert!(path.is_file());
    }

    /// Missing-row control: dropping a frozen row is RED (the
    /// denominator cannot shrink).
    #[test]
    fn p985_missing_row_is_red() {
        let text = real_text();
        let mutated = text
            .lines()
            .filter(|l| !l.starts_with("S-24\t"))
            .collect::<Vec<_>>()
            .join("\n");
        let err = validate_ledger(&mutated, &root()).expect_err("missing row must be RED");
        assert!(err.contains("missing"), "{err}");
        assert!(err.contains("S-24"), "{err}");
    }

    /// Duplicate control: the same obligation twice is RED.
    #[test]
    fn p985_duplicate_row_is_red() {
        let text = real_text();
        let dup = text
            .lines()
            .find(|l| l.starts_with("L-01\t"))
            .expect("L-01 row");
        let mutated = format!("{}\n{dup}", text.trim_end_matches('\n'));
        let err = validate_ledger(&mutated, &root()).expect_err("duplicate must be RED");
        assert!(err.contains("duplicate"), "{err}");
    }

    /// Wrong-source control: a corrupted sha16 pin is RED (stale or
    /// forged receipt).
    #[test]
    fn p985_wrong_source_hash_is_red() {
        let text = real_text();
        let mutated = text.replace(":8f10b476c76d8ac4", ":0000000000000000");
        // Guard: the mutation actually hit a pinned citation.
        assert_ne!(mutated, text, "mutation must change a pinned hash");
        let err = validate_ledger(&mutated, &root()).expect_err("wrong hash must be RED");
        assert!(err.contains("sha256 mismatch"), "{err}");
    }

    /// Wrong-source control: a receipt outside the reviewed roots (e.g.
    /// a source file or a test binary's own output) is RED — a worker's
    /// self-reported PASS is never a receipt.
    #[test]
    fn p985_non_receipt_evidence_path_is_red() {
        let text = real_text();
        let mutated = text.replace(
            "docs/specs/ace-migration/m2-numbers.md:",
            "crates/auto-lang-core/src/ace_entry.rs:",
        );
        assert_ne!(mutated, text);
        let err = validate_ledger(&mutated, &root()).expect_err("non-receipt path must be RED");
        assert!(err.contains("outside reviewed receipt roots"), "{err}");
    }

    /// Vocabulary + ownership controls: bad phase and empty successor
    /// are RED.
    #[test]
    fn p985_vocabulary_and_ownership_are_red() {
        let text = real_text();
        let bad_phase = text.replacen("\tA\t", "\tX\t", 1);
        assert_ne!(&bad_phase, &text);
        let err = validate_ledger(&bad_phase, &root()).expect_err("bad phase must be RED");
        assert!(err.contains("phase"), "{err}");

        let no_owner = text.replacen("\t961\t", "\t\t", 1);
        assert_ne!(&no_owner, &text);
        let err = validate_ledger(&no_owner, &root()).expect_err("unowned row must be RED");
        assert!(err.contains("successor owner"), "{err}");
    }

    /// Pending-honesty control: a pending row citing a receipt is RED
    /// (no evidence inflation).
    #[test]
    fn p985_pending_row_with_receipt_is_red() {
        let text = real_text();
        let Some(line) = text.lines().find(|l| l.ends_with("\tpending\t")) else {
            panic!("a pending row exists in the real matrix");
        };
        // `line` ends with the tab of the empty evidence_ref field — fill it.
        let inflated = format!("{line}docs/specs/ace-migration/m2-host.md:0000000000000000");
        let mutated = text.replace(line, &inflated);
        assert_ne!(&mutated, &text);
        let err = validate_ledger(&mutated, &root()).expect_err("pending+receipt must be RED");
        assert!(err.contains("evidence inflation"), "{err}");
    }
}
