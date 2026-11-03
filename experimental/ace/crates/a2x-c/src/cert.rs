//! PLAN-921 T-10: the C scalar channel's certificate draft (strict).
//!
//! Mirrors the Rust channel's `CertDraft` discipline exactly (its own
//! schema/recipe literals; exact-field-set policy; verdict/oracle are
//! observations whose identity must be self-consistent; unknown keys
//! cannot enter because the struct is typed) with the C toolchain
//! identity: the resolved cl.exe banner + binary hash, the derived
//! vcvars64 path hash and the frozen flags. The old `ace.m1.cert/1` and
//! every `ace.a2r-hir.*` schema are mutually untrusted with this one —
//! cross-schema impersonation is a REJECTED tamper control.

use crate::C_RECIPE;

/// The C scalar channel's certificate schema.
pub const C_CERT_SCHEMA: &str = "ace.a2c-hir.scalar.cert/1";

/// One bound actual file (path + content sha256).
#[derive(Clone, Debug)]
pub struct BoundFile {
    pub path: String,
    pub sha256: String,
}

/// The C toolchain identity bound into certs (frozen locator output: cl
/// banner + binary hash + the vcvars64 path content hash + the frozen
/// flags). Populated by the worker leg (`crun::toolchain_identity`),
/// which owns the PLAN-797 locator.
#[derive(Clone, Debug)]
pub struct CToolchainIdentity {
    pub cl_version: String,
    pub cl_hash: String,
    pub vcvars_sha256: String,
    pub flags: Vec<String>,
}

/// The C scalar certification draft (canonical serialization order).
#[derive(Clone, Debug)]
pub struct CCertDraft {
    pub schema: String,
    pub recipe: String,
    pub case: String,
    pub profile: String,
    pub hir_domain: String,
    pub entry: String,
    pub verdict: String,
    pub oracle_kind: String,
    pub oracle_value: String,
    pub input_hash: String,
    pub source_sha256: String,
    pub actual_files: Vec<BoundFile>,
    pub golden_sha256: Option<String>,
    pub wrapper_sha256: String,
    pub reference_sha256: String,
    pub backend: String,
    pub backend_version: String,
    pub standard: String,
    pub shared_view_version: String,
    pub protocol: String,
    pub toolchain: CToolchainIdentity,
    pub capabilities: Vec<String>,
}

fn kv(out: &mut String, k: &str, v: &str) {
    out.push_str(&format!("  \"{}\": \"{}\",\n", k, v));
}

fn sha64_ok(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

impl CCertDraft {
    /// Byte-stable JSON (fixed key order).
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\n");
        kv(&mut out, "schema", &self.schema);
        kv(&mut out, "recipe", &self.recipe);
        kv(&mut out, "case", &self.case);
        kv(&mut out, "profile", &self.profile);
        kv(&mut out, "hir_domain", &self.hir_domain);
        kv(&mut out, "entry", &self.entry);
        kv(&mut out, "verdict", &self.verdict);
        kv(&mut out, "oracle_kind", &self.oracle_kind);
        kv(&mut out, "oracle_value", &self.oracle_value);
        kv(&mut out, "input_hash", &self.input_hash);
        kv(&mut out, "source_sha256", &self.source_sha256);
        if let Some(g) = &self.golden_sha256 {
            kv(&mut out, "golden_sha256", g);
        }
        kv(&mut out, "wrapper_sha256", &self.wrapper_sha256);
        kv(&mut out, "reference_sha256", &self.reference_sha256);
        kv(&mut out, "backend", &self.backend);
        kv(&mut out, "backend_version", &self.backend_version);
        kv(&mut out, "standard", &self.standard);
        kv(&mut out, "shared_view_version", &self.shared_view_version);
        kv(&mut out, "protocol", &self.protocol);
        out.push_str("  \"toolchain\": {\n");
        kv(&mut out, "kind", "cl");
        kv(&mut out, "version", &self.toolchain.cl_version);
        kv(&mut out, "version_sha256", &self.toolchain.cl_hash);
        kv(&mut out, "vcvars_sha256", &self.toolchain.vcvars_sha256);
        out.push_str(&format!("    \"flags\": {:?}\n", self.toolchain.flags));
        out.push_str("  },\n");
        out.push_str("  \"capabilities\": [\n");
        for (i, c) in self.capabilities.iter().enumerate() {
            let comma = if i + 1 < self.capabilities.len() {
                ","
            } else {
                ""
            };
            out.push_str(&format!("    \"{}\"{}\n", c, comma));
        }
        out.push_str("  ],\n");
        out.push_str("  \"actual_files\": [\n");
        for (i, f) in self.actual_files.iter().enumerate() {
            let comma = if i + 1 < self.actual_files.len() {
                ","
            } else {
                ""
            };
            out.push_str(&format!(
                "    {{ \"path\": \"{}\", \"sha256\": \"{}\" }}{}\n",
                f.path, f.sha256, comma
            ));
        }
        out.push_str("  ]\n}\n");
        out
    }

    /// STRICT validation: exact schema/recipe literals, complete
    /// identities, well-formed files. `None` on any deviation.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != C_CERT_SCHEMA {
            return Err(format!("schema `{}` is not {}", self.schema, C_CERT_SCHEMA));
        }
        if self.recipe != C_RECIPE {
            return Err(format!("recipe `{}` is not {}", self.recipe, C_RECIPE));
        }
        for f in [
            &self.case,
            &self.profile,
            &self.hir_domain,
            &self.entry,
            &self.input_hash,
            &self.source_sha256,
            &self.wrapper_sha256,
            &self.reference_sha256,
            &self.backend_version,
            &self.shared_view_version,
            &self.protocol,
        ] {
            if f.is_empty() {
                return Err("an identity field is empty".into());
            }
        }
        if !matches!(self.verdict.as_str(), "PASS" | "REJECTED") {
            return Err(format!("verdict `{}` is not PASS/REJECTED", self.verdict));
        }
        if !matches!(self.oracle_kind.as_str(), "value" | "trap") {
            return Err(format!(
                "oracle_kind `{}` is not value/trap",
                self.oracle_kind
            ));
        }
        if self.oracle_value.is_empty() {
            return Err("oracle_value is empty".into());
        }
        if self.actual_files.is_empty() {
            return Err("actual_files is empty (compile-only or trimmed evidence)".into());
        }
        for f in &self.actual_files {
            if f.path.is_empty() || !sha64_ok(&f.sha256) {
                return Err(format!("actual file `{}` has a bad binding", f.path));
            }
        }
        if self.backend != "auto-a2x-c" {
            return Err(format!("backend `{}` is not auto-a2x-c", self.backend));
        }
        if self.standard != "c11" {
            return Err(format!("standard `{}` is not c11", self.standard));
        }
        if self.toolchain.cl_version.is_empty()
            || !sha64_ok(&self.toolchain.cl_hash)
            || !sha64_ok(&self.toolchain.vcvars_sha256)
            || self.toolchain.flags.is_empty()
        {
            return Err("toolchain identity is incomplete".into());
        }
        if let Some(g) = &self.golden_sha256 {
            if !sha64_ok(g) {
                return Err("golden_sha256 is malformed".into());
            }
        }
        Ok(())
    }
}
