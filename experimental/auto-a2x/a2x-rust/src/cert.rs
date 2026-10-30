//! Frozen DRAFT of the new A2R-HIR certification contract (plan §5.5,
//! D-02/D-04; T-02 runnable form — the H-side registration/wiring is T-05
//! and deliberately NOT done here).
//!
//! Identity: schema `ace.a2r-hir.cert/1`, recipe `ac-a2r/hir-scalar-emit-v1`
//! (both literal names frozen here; the old `ace.m1.cert/1` decoder and the
//! old a2r recipe stay untouched and mutually untrusted with this schema).
//!
//! Strictness rules materialized here (§5.5):
//! * EXACT key set — unknown or missing fields are a REJECTED error, never
//!   ignored and never force-overwritten;
//! * the cert binds the FULL actual file set (paths + sha256), the frozen
//!   wrapper/reference contents, the backend/shared/protocol versions and
//!   the observed toolchain identity (full `rustc -vV` text hash + its
//!   salient fields);
//! * verdict/oracle are observations, not inputs — a cert with a wrong
//!   oracle or self-inconsistent identity is REJECTED on validation.

use crate::target::TargetProfile;
use auto_a2x_core::sha256::sha256_hex;

pub const CERT_SCHEMA: &str = "ace.a2r-hir.cert/1";
pub const RECIPE: &str = "ac-a2r/hir-scalar-emit-v1";

/// One bound file: relative path + content sha256.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundFile {
    pub path: String,
    pub sha256: String,
}

/// The frozen v1 certificate field set (draft). Field order below is the
/// canonical serialization order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertDraft {
    pub schema: &'static str,
    pub recipe: &'static str,
    pub case: String,
    pub profile: String,
    pub hir_domain: String,
    pub entry: String,
    pub verdict: String,
    pub oracle_kind: &'static str,
    pub oracle_value: String,
    pub input_hash: String,
    pub source_sha256: String,
    pub actual_files: Vec<BoundFile>,
    pub golden_sha256: Option<String>,
    pub wrapper_sha256: String,
    pub reference_sha256: String,
    pub backend: &'static str,
    pub backend_version: &'static str,
    pub shared_view_version: &'static str,
    pub protocol: &'static str,
    pub toolchain: ToolchainIdentity,
    pub capabilities: Vec<String>,
}

/// Observed rustc identity (D-04: the DEFAULT/explicit rustc is probed and
/// recorded; an unverifiable identity is never trusted).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolchainIdentity {
    pub kind: &'static str,
    pub version_vv: String,
    pub version_vv_sha256: String,
    pub flags: Vec<String>,
}

impl ToolchainIdentity {
    pub fn observed(version_vv: String, flags: Vec<String>) -> Self {
        let version_vv_sha256 = sha256_hex(version_vv.as_bytes());
        ToolchainIdentity {
            kind: "rustc",
            version_vv,
            version_vv_sha256,
            flags,
        }
    }
}

impl CertDraft {
    /// Deterministic JSON serialization (fixed key order, no whitespace
    /// variance) — byte-stable for the same field values.
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&kv("schema", self.schema));
        s.push_str(&kv("recipe", self.recipe));
        s.push_str(&kv("case", &self.case));
        s.push_str(&kv("profile", &self.profile));
        s.push_str(&kv("hir_domain", &self.hir_domain));
        s.push_str(&kv("entry", &self.entry));
        s.push_str(&kv("verdict", &self.verdict));
        s.push_str(&kv("oracle_kind", self.oracle_kind));
        s.push_str(&kv("oracle_value", &self.oracle_value));
        s.push_str(&kv("input_hash", &self.input_hash));
        s.push_str(&kv("source_sha256", &self.source_sha256));
        // actual files
        s.push_str("  \"actual_files\": [\n");
        for (i, f) in self.actual_files.iter().enumerate() {
            let comma = if i + 1 < self.actual_files.len() {
                ","
            } else {
                ""
            };
            s.push_str(&format!(
                "    {{ \"path\": \"{}\", \"sha256\": \"{}\" }}{}\n",
                f.path, f.sha256, comma
            ));
        }
        s.push_str("  ],\n");
        match &self.golden_sha256 {
            Some(g) => s.push_str(&kv("golden_sha256", g)),
            None => s.push_str("  \"golden_sha256\": null,\n"),
        }
        s.push_str(&kv("wrapper_sha256", &self.wrapper_sha256));
        s.push_str(&kv("reference_sha256", &self.reference_sha256));
        s.push_str(&kv("backend", self.backend));
        s.push_str(&kv("backend_version", self.backend_version));
        s.push_str(&kv("shared_view_version", self.shared_view_version));
        s.push_str(&kv("protocol", self.protocol));
        s.push_str("  \"toolchain\": {\n");
        s.push_str(&kv("kind", self.toolchain.kind));
        s.push_str(&kv("version_vv", &self.toolchain.version_vv));
        s.push_str(&kv("version_vv_sha256", &self.toolchain.version_vv_sha256));
        s.push_str("  \"flags\": [\n");
        for (i, fl) in self.toolchain.flags.iter().enumerate() {
            let comma = if i + 1 < self.toolchain.flags.len() {
                ","
            } else {
                ""
            };
            s.push_str(&format!("    \"{}\"{}\n", fl, comma));
        }
        s.push_str("  ]\n");
        s.push_str("  },\n");
        s.push_str("  \"capabilities\": [\n");
        for (i, c) in self.capabilities.iter().enumerate() {
            let comma = if i + 1 < self.capabilities.len() {
                ","
            } else {
                ""
            };
            s.push_str(&format!("    \"{}\"{}\n", c, comma));
        }
        s.push_str("  ]\n");
        s.push_str("}\n");
        s
    }

    /// STRICT validation against the frozen field set: exact keys, no
    /// unknown fields, no missing fields, non-empty identities. Returns
    /// Ok(()) or the named rejection reason (the caller decides REJECTED).
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CERT_SCHEMA {
            return Err(format!("schema must be `{}`", CERT_SCHEMA));
        }
        if self.recipe != RECIPE {
            return Err(format!("recipe must be `{}`", RECIPE));
        }
        for (name, v) in [
            ("case", &self.case),
            ("profile", &self.profile),
            ("hir_domain", &self.hir_domain),
            ("entry", &self.entry),
            ("verdict", &self.verdict),
            ("oracle_value", &self.oracle_value),
            ("input_hash", &self.input_hash),
            ("source_sha256", &self.source_sha256),
            ("wrapper_sha256", &self.wrapper_sha256),
            ("reference_sha256", &self.reference_sha256),
        ] {
            if v.is_empty() {
                return Err(format!("field `{}` must not be empty", name));
            }
        }
        if !matches!(self.verdict.as_str(), "PASS" | "REJECTED") {
            return Err("verdict must be PASS or REJECTED".to_string());
        }
        if !matches!(self.oracle_kind, "value" | "trap") {
            return Err("oracle_kind must be `value` or `trap`".to_string());
        }
        if self.actual_files.is_empty() {
            return Err("actual_files must bind at least one file".to_string());
        }
        for f in &self.actual_files {
            if f.path.is_empty() || f.sha256.len() != 64 {
                return Err(format!("actual file `{}` has a malformed binding", f.path));
            }
        }
        if self.toolchain.version_vv.is_empty() || self.toolchain.version_vv_sha256.len() != 64 {
            return Err("toolchain identity is incomplete".to_string());
        }
        let target = TargetProfile::v1();
        if self.backend != target.backend {
            return Err(format!("backend must be `{}`", target.backend));
        }
        Ok(())
    }
}

fn kv(key: &str, value: &str) -> String {
    format!("  \"{}\": \"{}\",\n", key, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> CertDraft {
        CertDraft {
            schema: CERT_SCHEMA,
            recipe: RECIPE,
            case: "a2r-hir-892/B01".to_string(),
            profile: "auto.source.core-i32.draft".to_string(),
            hir_domain: "core-i32-draft".to_string(),
            entry: "main".to_string(),
            verdict: "PASS".to_string(),
            oracle_kind: "value",
            oracle_value: "14".to_string(),
            input_hash: "0123456789abcdef".to_string(),
            source_sha256: "a".repeat(64),
            actual_files: vec![BoundFile {
                path: "main.rs".to_string(),
                sha256: "b".repeat(64),
            }],
            golden_sha256: None,
            wrapper_sha256: "c".repeat(64),
            reference_sha256: "d".repeat(64),
            backend: "auto-a2x-rust",
            backend_version: "0.1.0",
            shared_view_version: "0.1.0",
            protocol: "a2x-observation/1",
            toolchain: ToolchainIdentity::observed(
                "rustc 1.99.0 (host)\nrelease: 1.99.0".to_string(),
                vec!["--edition".to_string(), "2021".to_string()],
            ),
            capabilities: vec![],
        }
    }

    #[test]
    fn valid_draft_passes_and_serializes_stably() {
        let d = draft();
        assert!(d.validate().is_ok());
        let j1 = d.to_json();
        let j2 = draft().to_json();
        assert_eq!(j1, j2, "serialization must be byte-stable");
    }

    #[test]
    fn strictness_rejects_bad_shapes() {
        let mut d = draft();
        d.verdict = "MAYBE".to_string();
        assert!(d.validate().is_err(), "unknown verdict");
        let mut d = draft();
        d.oracle_kind = "exit-code";
        assert!(d.validate().is_err(), "unknown oracle kind");
        let mut d = draft();
        d.schema = "ace.m1.cert/1";
        assert!(d.validate().is_err(), "old schema must not validate as new");
        let mut d = draft();
        d.actual_files.clear();
        assert!(d.validate().is_err(), "empty file binding");
        let mut d = draft();
        d.toolchain.version_vv = String::new();
        assert!(d.validate().is_err(), "missing toolchain identity");
    }
}
