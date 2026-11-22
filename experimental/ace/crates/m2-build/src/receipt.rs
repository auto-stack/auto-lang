//! PLAN-1117 T-04: the `IF-ACE-A2R-V05-REPLACEMENT/1` receiving gate — a
//! machine-checkable replacement receipt the 985/B / 986 / 987 chain
//! consumes. The gate REJECTS (each by name):
//!
//! * a missing v0.5-required row (`v05.required-row`);
//! * an emit-only row masquerading as a real run (`v05.stage`);
//! * an artifact hash that does not match the referenced SDK
//!   (`v05.artifact`), or an SDK whose a2r channel is not the language
//!   channel (`v05.channel`);
//! * a dispatch proof that claims ACE while legacy dispatches occurred
//!   (`v05.dispatch`) — the "labelled ACE, actually legacy" negative;
//! * schema/identity drift (`v05.schema`, `v05.identity`).

use crate::M2Reject;
use serde::{Deserialize, Serialize};

/// The frozen receipt schema identity.
pub const V05_RECEIPT_SCHEMA: &str = "auto.a2r.v05-replacement/1";

/// The language channel's fileset schema — the only channel a v0.5
/// candidate artifact may come from.
pub const V05_LANGUAGE_FILESET_SCHEMA: &str = "auto.a2x.rust.language.fileset/1";

/// The v0.5-required row set this interface signs. Row ids are frozen;
/// a row outside the set is allowed as extra evidence but never
/// substitutes a required one.
pub const V05_REQUIRED_ROWS: &[&str] = &[
    // E-02/E-01(merge): the CLI-bound renderer route (host bridge) — the
    // same emitted fileset, run for real by the build lane's consumer leg.
    "route.auto-rust",
    // E-03 single-file build: build lane -> precompiled exe -> SDK ->
    // relocated run.
    "route.build-lane",
    // The error face: the checked-trap oracle through the same lane.
    "route.trap",
    // V-02: the rollback replay (tamper -> named rejection -> restore ->
    // re-select candidate runs the new A2R).
    "route.rollback",
];

/// One replacement row: an entry, its route, its real-run evidence and
/// its dispatch proof, each hash-bound to the referenced SDK.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct V05Row {
    pub id: String,
    /// The row's candidate artifact tree (directory name under the
    /// validating root; each row's SDK is re-hashed from there).
    pub sdk: String,
    /// "ace" — a legacy row is not a replacement claim.
    pub route: String,
    /// "run" — emit-only evidence never satisfies a replacement row.
    pub stage: String,
    /// sha256 of the row's `targets/a2r/assembly-manifest.json`.
    pub manifest_sha256: String,
    /// sha256 of the row's precompiled `targets/a2r/<name>.exe`.
    pub artifact_sha256: String,
    /// The real run observation (exit / stdout / channel).
    pub exit: Option<i32>,
    pub stdout: String,
    pub channel: String,
    /// The dispatch proof (`AUTO_ACE_RUST_PROOF` shape).
    pub engine: String,
    pub a2r_dispatches: u64,
    pub legacy_dispatches: u64,
}

/// The receipt envelope.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct V05Receipt {
    pub schema: String,
    /// The candidate's source commit (worktree identity).
    pub commit: String,
    /// The rustc identity from the build receipt.
    pub toolchain: String,
    pub rows: Vec<V05Row>,
}

impl V05Receipt {
    /// Serialize (stable pretty JSON).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Parse + structurally validate a receipt against the frozen rules.
    /// `None` for `root` skips the artifact re-hash (used by negative
    /// controls that mutate hash fields); the receiving gate always
    /// supplies the candidate root holding each row's SDK directory.
    pub fn validate(json: &str, root: Option<&std::path::Path>) -> Result<Self, M2Reject> {
        let receipt: V05Receipt = serde_json::from_str(json)
            .map_err(|e| M2Reject::new("v05.schema", format!("receipt json: {e}")))?;
        if receipt.schema != V05_RECEIPT_SCHEMA {
            return Err(M2Reject::new(
                "v05.schema",
                format!("schema {:?} is not {V05_RECEIPT_SCHEMA}", receipt.schema),
            ));
        }
        if receipt.commit.trim().is_empty() || receipt.toolchain.trim().is_empty() {
            return Err(M2Reject::new(
                "v05.identity",
                "commit and toolchain must bind the candidate identity",
            ));
        }
        for required in V05_REQUIRED_ROWS {
            let Some(row) = receipt.rows.iter().find(|r| r.id == *required) else {
                return Err(M2Reject::new(
                    "v05.required-row",
                    format!("required row `{required}` is missing"),
                ));
            };
            if row.route != "ace" {
                return Err(M2Reject::new(
                    "v05.route",
                    format!("row `{required}` route {:?} is not a replacement claim", row.route),
                ));
            }
            if row.stage != "run" {
                return Err(M2Reject::new(
                    "v05.stage",
                    format!(
                        "row `{required}` stage {:?} — emit-only evidence never satisfies a replacement row",
                        row.stage
                    ),
                ));
            }
            if row.engine != "ace" || row.legacy_dispatches != 0 || row.a2r_dispatches == 0 {
                return Err(M2Reject::new(
                    "v05.dispatch",
                    format!(
                        "row `{required}` claims engine {:?} with a2r={} legacy={} — a legacy dispatch under an ACE claim is rejected",
                        row.engine, row.a2r_dispatches, row.legacy_dispatches
                    ),
                ));
            }
            if row.stdout.trim().is_empty() || row.channel.trim().is_empty() {
                return Err(M2Reject::new(
                    "v05.stage",
                    format!("row `{required}` carries no real run observation"),
                ));
            }
        }
        if let Some(root) = root {
            for row in &receipt.rows {
                let dir = root.join(&row.sdk);
                let manifest =
                    std::fs::read(dir.join("targets/a2r/assembly-manifest.json")).map_err(|e| {
                        M2Reject::new("v05.artifact", format!("read assembly manifest: {e}"))
                    })?;
                let manifest_sha = ace_a2x_core::sha256::sha256_hex(&manifest);
                if manifest_sha != row.manifest_sha256 {
                    return Err(M2Reject::new(
                        "v05.artifact",
                        format!(
                            "row `{}` manifest sha {manifest_sha} != claimed {}",
                            row.id, row.manifest_sha256
                        ),
                    ));
                }
                if !std::str::from_utf8(&manifest)
                    .map(|m| m.contains(V05_LANGUAGE_FILESET_SCHEMA))
                    .unwrap_or(false)
                {
                    return Err(M2Reject::new(
                        "v05.channel",
                        format!(
                            "row `{}` artifact is not on the language channel ({V05_LANGUAGE_FILESET_SCHEMA})",
                            row.id
                        ),
                    ));
                }
                let exe = find_a2r_exe(&dir).ok_or_else(|| {
                    M2Reject::new("v05.artifact", "the SDK carries no precompiled a2r exe")
                })?;
                let artifact_sha = ace_a2x_core::sha256::sha256_hex(&std::fs::read(&exe).map_err(
                    |e| M2Reject::new("v05.artifact", format!("read {}: {e}", exe.display())),
                )?);
                if artifact_sha != row.artifact_sha256 {
                    return Err(M2Reject::new(
                        "v05.artifact",
                        format!(
                            "row `{}` artifact sha {artifact_sha} != claimed {}",
                            row.id, row.artifact_sha256
                        ),
                    ));
                }
            }
        }
        Ok(receipt)
    }
}

fn find_a2r_exe(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let targets = dir.join("targets/a2r");
    std::fs::read_dir(targets).ok()?.find_map(|e| {
        let p = e.ok()?.path();
        (p.extension().and_then(|x| x.to_str()) == Some("exe")).then_some(p)
    })
}

/// Build a genuine row from a real SDK run: hash the SDK's actual
/// assembly manifest + precompiled exe and bind the run observation and
/// the dispatch proof.
pub fn v05_row_from_sdk(
    id: &'static str,
    sdk_label: &str,
    sdk_dir: &std::path::Path,
    exit: Option<i32>,
    stdout: &str,
    channel: &str,
    engine: &str,
    a2r_dispatches: u64,
    legacy_dispatches: u64,
) -> Result<V05Row, M2Reject> {
    let manifest = std::fs::read(sdk_dir.join("targets/a2r/assembly-manifest.json"))
        .map_err(|e| M2Reject::new("v05.artifact", format!("read assembly manifest: {e}")))?;
    let manifest_sha256 = ace_a2x_core::sha256::sha256_hex(&manifest);
    let exe = find_a2r_exe(sdk_dir)
        .ok_or_else(|| M2Reject::new("v05.artifact", "no precompiled a2r exe in the SDK"))?;
    let artifact_sha256 = ace_a2x_core::sha256::sha256_hex(
        &std::fs::read(&exe).map_err(|e| M2Reject::new("v05.artifact", format!("read exe: {e}")))?,
    );
    Ok(V05Row {
        id: id.to_string(),
        sdk: sdk_label.to_string(),
        route: "ace".into(),
        stage: "run".into(),
        manifest_sha256,
        artifact_sha256,
        exit,
        stdout: stdout.into(),
        channel: channel.into(),
        engine: engine.into(),
        a2r_dispatches,
        legacy_dispatches,
    })
}
