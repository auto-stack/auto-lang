//! PLAN-978 T-01/T-02 (AC-01/AC-ID-BUILD): the project-level engineering
//! build identity. Every axis is a REAL read (source bytes hashed at
//! build time, lane/tool identities read from the lanes themselves) —
//! no labels, no caller-supplied trust.
//!
//! The key layering contract (plan §5 ID-01): physical provider build
//! information NEVER leaks into per-function semantic keys (those stay
//! with the AXS* layers in a2x-store); this identity binds one whole
//! (project, target) BUILD NODE and is what the precise artifact cache
//! and the SDK provenance section key on.

use serde::{Deserialize, Serialize};

use crate::compat::TargetAxis;
use crate::driver::ProjectInputs;

/// Probe-owned layer (the 967 S967B pattern: a new layer id registered by
/// its owner, zero modification of the 783/787 layer registry).
pub const M978B_LAYER: &str = "M978B";

pub const PROJECT_BUILD_SCHEMA: &str = "ace-m2-build/project-build/1";
pub const BUILD_RECEIPT_SCHEMA: &str = "ace-m2-build/build-receipt/1";

/// The build/run targets of the engineering build. One AV executor with
/// explicit native/C/Rust provider sources plus the A2R/A2C/A2PY emit
/// targets (plan §5 coverage contract — the legacy "four ends" remain,
/// A2C included; nothing is dropped).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Av,
    Native,
    A2r,
    A2c,
    A2py,
}

impl TargetKind {
    pub const ALL: [TargetKind; 5] = [
        TargetKind::Av,
        TargetKind::Native,
        TargetKind::A2r,
        TargetKind::A2c,
        TargetKind::A2py,
    ];

    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "av" => Ok(TargetKind::Av),
            "native" => Ok(TargetKind::Native),
            "a2r" => Ok(TargetKind::A2r),
            "a2c" => Ok(TargetKind::A2c),
            "a2py" => Ok(TargetKind::A2py),
            other => Err(format!(
                "unknown target `{other}` (expected av|native|a2r|a2c|a2py)"
            )),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TargetKind::Av => "av",
            TargetKind::Native => "native",
            TargetKind::A2r => "a2r",
            TargetKind::A2c => "a2c",
            TargetKind::A2py => "a2py",
        }
    }

    /// The suffix-source layer this target selects (PLAN-1043 suffix
    /// assembly; `None` = the public layer alone).
    pub fn source_suffix(&self) -> Option<&'static str> {
        match self {
            TargetKind::Av => None,
            TargetKind::Native => Some("c"),
            TargetKind::A2r => Some("rs"),
            TargetKind::A2c => Some("c"),
            // The python lane consumes the C ABI foreign surface (1042).
            TargetKind::A2py => Some("c"),
        }
    }
}

/// One source file in the declared closure. Roles mirror the assembly
/// manifest roles (`public` / `selected_target` / `candidate` / `stdlib`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub role: String,
    pub sha256: String,
}

/// The engineering build identity of one (project, target) build node.
///
/// Tool axes are RECORDED for provenance and precise cache invalidation
/// of the BUILD node; they are NEVER runtime-admission axes (that is
/// `crate::compat`, where rustc identity is explicitly not compared).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectBuildIdentity {
    pub schema: String,
    pub project: String,
    pub target: TargetKind,
    pub entry: String,
    pub caps: Vec<String>,
    /// Sorted by (path, role) — content-addressed closure.
    pub sources: Vec<SourceFile>,
    /// Content identity of the consumed stdlib closure (0 entries = none).
    pub stdlib: Vec<SourceFile>,
    /// Grammar/pipeline identity of the lane (a REAL read from the lane).
    pub grammar_identity: String,
    /// Sorted tool/generator axes (build-time provenance only).
    pub tool_axes: Vec<(String, String)>,
    /// The 967 S967B provider build identity when a provider artifact
    /// participated in this build node (consumed as-is, never re-derived).
    pub provider_build: Option<ace_abi::provider_build_cache::BuildIdentity>,
}

impl ProjectBuildIdentity {
    /// Derive the identity from REAL reads of the project inputs and the
    /// lane's own identity needles.
    pub fn of(
        inputs: &ProjectInputs,
        target: TargetKind,
        grammar_identity: &str,
        tool_axes: Vec<(String, String)>,
    ) -> Self {
        let mut sources: Vec<SourceFile> = inputs
            .sources
            .iter()
            .map(|(path, text)| SourceFile {
                path: path.clone(),
                role: "public".into(),
                sha256: ace_a2x_core::sha256::sha256_hex(text.as_bytes()),
            })
            .collect();
        sources.sort_by(|a, b| (&a.path, &a.role).cmp(&(&b.path, &b.role)));
        let mut stdlib = inputs
            .stdlib
            .iter()
            .map(|(path, text)| SourceFile {
                path: path.clone(),
                role: "stdlib".into(),
                sha256: ace_a2x_core::sha256::sha256_hex(text.as_bytes()),
            })
            .collect::<Vec<_>>();
        stdlib.sort_by(|a, b| a.path.cmp(&b.path));
        let mut caps = inputs.caps.clone();
        caps.sort();
        Self {
            schema: PROJECT_BUILD_SCHEMA.into(),
            project: inputs.name.clone(),
            target,
            entry: inputs.entry.clone(),
            caps,
            sources,
            stdlib,
            grammar_identity: grammar_identity.to_string(),
            tool_axes: {
                let mut axes = tool_axes;
                axes.sort();
                axes
            },
            provider_build: None,
        }
    }

    /// The canonical store key of this identity (sorted axes, version
    /// tagged — the precise-cache hit/miss decision).
    pub fn key(&self) -> String {
        let sources = self
            .sources
            .iter()
            .map(|s| format!("{}:{}:{}", s.path, s.role, &s.sha256[..16]))
            .collect::<Vec<_>>()
            .join(",");
        let stdlib = self
            .stdlib
            .iter()
            .map(|s| format!("{}:{}", s.path, &s.sha256[..16]))
            .collect::<Vec<_>>()
            .join(",");
        let tools = self
            .tool_axes
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(";");
        let provider = self
            .provider_build
            .as_ref()
            .map(|b| b.key())
            .unwrap_or_else(|| "none".into());
        format!(
            "M978B/v1 project={} target={} entry={} caps={} sources=[{}] stdlib=[{}] grammar={} tools=[{}] provider={}",
            self.project,
            self.target.name(),
            self.entry,
            self.caps.join("+"),
            sources,
            stdlib,
            self.grammar_identity,
            tools,
            provider
        )
    }

    /// The target axis this build node produced for (runtime-compat seed).
    pub fn target_axis(&self) -> TargetAxis {
        TargetAxis::host()
    }
}

/// One persisted build receipt (what a cache HIT restores): the identity
/// plus the concrete products (SDK-relative paths + bytes).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildReceipt {
    pub schema: String,
    pub identity: ProjectBuildIdentity,
    /// (relative path, base64-free raw bytes are NOT stored here — the
    /// products ride as hex to keep the receipt one JSON document)
    pub products: Vec<(String, String)>,
}
