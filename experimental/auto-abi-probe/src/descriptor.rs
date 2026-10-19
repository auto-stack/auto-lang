//! Machine-readable module descriptor — the pre-call public boundary
//! (plan §5.1, RFC §3.2/§3.3).
//!
//! A descriptor is what a consumer READS BEFORE any native call: ABI
//! domain/version, target/profile, packing, the semantic-ref snapshot it
//! was built against, its record types, public symbols with exact
//! physical signature forms, and the runtime services it requires.
//! Unknown versions / wrong target / packing / symbols / services are
//! rejected at admission ([`crate::validate`]) with native call count 0
//! — this module only defines and fingerprints; it never links or runs.
//!
//! Canonical form: stable JSON (sorted symbol/record vectors). The
//! interface fingerprint is blake3 over the canonical form; it is a fast
//! comparison key, while the canonical form stays available to explain
//! conflicts (RFC §3.2).

use crate::layout::{RecordDefOwned, SemScalar};
use crate::semref;
use serde::{Deserialize, Serialize};

pub const DESCRIPTOR_FORMAT: &str = "autoabi.descriptor/2";
pub const ABI_DOMAIN: &str = "autoabi";
pub const ABI_VERSION: &str = "0.1";
/// v0.1 supports exactly one packing rule; anything else is rejected.
pub const PACKING: &str = "natural-align";

/// Parameter kinds legal in public signatures (contract-decision.md §2).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ParamKind {
    I32,
    U32,
    I64,
    U64,
    F64,
    Bool,
    Char,
    /// Explicit context pointer (fallible form's first parameter).
    CtxPtr {
        nullable: bool,
    },
    /// Borrowed byte buffer view `{ptr, len}`.
    BufViewBytes,
    /// Borrowed UTF-8 text view — providers VALIDATE the encoding.
    BufViewUtf8,
    /// Borrowed view of a named record (layout via the record table).
    RecView {
        rec: String,
    },
    /// Opaque provider-owned handle.
    Handle,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum RetKind {
    Unit,
    I32,
    U32,
    I64,
    U64,
    F64,
    Bool,
    Char,
}

impl ParamKind {
    pub fn spell(&self) -> String {
        match self {
            ParamKind::I32 => "i32".into(),
            ParamKind::U32 => "u32".into(),
            ParamKind::I64 => "i64".into(),
            ParamKind::U64 => "u64".into(),
            ParamKind::F64 => "f64".into(),
            ParamKind::Bool => "bool".into(),
            ParamKind::Char => "char".into(),
            ParamKind::CtxPtr { nullable } => {
                format!("ctx:{}", if *nullable { "nullable" } else { "required" })
            }
            ParamKind::BufViewBytes => "buf:bytes".into(),
            ParamKind::BufViewUtf8 => "buf:utf8".into(),
            ParamKind::RecView { rec } => format!("rec:{rec}"),
            ParamKind::Handle => "handle".into(),
        }
    }
}

impl RetKind {
    pub fn spell(&self) -> &'static str {
        match self {
            RetKind::Unit => "unit",
            RetKind::I32 => "i32",
            RetKind::U32 => "u32",
            RetKind::I64 => "i64",
            RetKind::U64 => "u64",
            RetKind::F64 => "f64",
            RetKind::Bool => "bool",
            RetKind::Char => "char",
        }
    }
}

/// Physical signature form of one public symbol (contract-decision §2).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SymbolForm {
    /// Platform call convention, scalar params/return.
    Pure {
        params: Vec<ParamKind>,
        ret: RetKind,
    },
    /// `i32 status f(params..., out*)` — success commits out, failure
    /// leaves caller output untouched.
    Fallible {
        /// F-797-R4-02: the EXPLICIT CONTEXT first parameter is part of
        /// the form (contract §2); `None` spells a context-free entry.
        ctx: Option<bool>, // Some(nullable)
        params: Vec<ParamKind>,
        out: ParamKind,
    },
}

impl SymbolForm {
    /// Canonical signature spelling — the exact comparison key between
    /// consumer expectation and descriptor.
    pub fn signature(&self) -> String {
        match self {
            SymbolForm::Pure { params, ret } => format!(
                "pure({})->{}",
                params
                    .iter()
                    .map(|p| p.spell())
                    .collect::<Vec<_>>()
                    .join(","),
                ret.spell()
            ),
            SymbolForm::Fallible { ctx, params, out } => {
                let ctx_s = match ctx {
                    Some(nullable) => {
                        format!("ctx:{},", if *nullable { "nullable" } else { "required" })
                    }
                    None => String::new(),
                };
                format!(
                    "fallible({}{})->status,out {}",
                    ctx_s,
                    params
                        .iter()
                        .map(|p| p.spell())
                        .collect::<Vec<_>>()
                        .join(","),
                    out.spell()
                )
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub name: String,
    pub form: SymbolForm,
}

/// Runtime service requirements (RFC §7.2 service-table naming, v0.1
/// minimal set; versions are small integers).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct RuntimeReqs {
    pub services: Vec<(String, u32)>,
}

/// A semantic reference the module binds to: the DECL's logical id and
/// CONTENT VERSION minted through the real semantic contract. Admission
/// compares these against the consumer's own binding (F-797-R4-01): a
/// same-width re-spelling of a record mints a DIFFERENT content version
/// and is rejected even when the physical layout coincides.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SemanticRef {
    pub logical: String,
    pub content_version: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ModuleDescriptor {
    pub format: String,
    pub abi_domain: String,
    pub abi_version: String,
    pub target: String,
    pub profile: String,
    pub packing: String,
    pub interface_revision: u32,
    /// Semantic binding hash this module was built against.
    pub semref_hash: String,
    /// Per-decl semantic refs (F-797-R4-01): nominal + content-version
    /// of every record the module's types reference.
    #[serde(default)]
    pub semantic_refs: Vec<SemanticRef>,
    pub records: Vec<RecordDefOwned>,
    pub symbols: Vec<SymbolEntry>,
    pub runtime: RuntimeReqs,
}

#[derive(Debug)]
pub struct DescriptorError(pub String);

impl std::fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "abi.descriptor: {}", self.0)
    }
}

impl ModuleDescriptor {
    /// The T-02/T-03 probe module descriptor (p797_mix + RecMix), built
    /// against the frozen semantic-ref snapshot.
    pub fn probe_module() -> ModuleDescriptor {
        ModuleDescriptor {
            format: DESCRIPTOR_FORMAT.into(),
            abi_domain: ABI_DOMAIN.into(),
            abi_version: ABI_VERSION.into(),
            target: crate::layout::TARGET.into(),
            profile: crate::layout::ABI_PROFILE.into(),
            packing: PACKING.into(),
            interface_revision: 1,
            semref_hash: semref::binding_hash().into(),
            semantic_refs: vec![SemanticRef {
                logical: semref::rec_mix_ref()
                    .expect("record ref mints")
                    .logical
                    .as_str()
                    .to_string(),
                content_version: semref::rec_mix_ref()
                    .expect("record ref mints")
                    .content
                    .as_str()
                    .to_string(),
            }],
            records: vec![crate::layout::RecordDefOwned::from(&crate::layout::REC_MIX)],
            symbols: vec![SymbolEntry {
                name: "p797_mix".into(),
                form: SymbolForm::Pure {
                    params: vec![
                        ParamKind::I32,
                        ParamKind::I32,
                        ParamKind::I32,
                        ParamKind::I32,
                        ParamKind::I64,
                        ParamKind::F64,
                        ParamKind::I32,
                        ParamKind::F64,
                    ],
                    ret: RetKind::I64,
                },
            }],
            runtime: RuntimeReqs {
                services: vec![
                    ("alloc".into(), 1),
                    ("free".into(), 1),
                    ("handle-registry".into(), 1),
                ],
            },
        }
    }

    /// Canonical JSON: symbol and record vectors sorted by name so the
    /// fingerprint is a function of CONTENT, not declaration order.
    pub fn canonical(&self) -> Result<String, DescriptorError> {
        let mut sorted = self.clone();
        sorted.symbols.sort_by(|a, b| a.name.cmp(&b.name));
        sorted.records.sort_by(|a, b| a.name.cmp(&b.name));
        sorted.runtime.services.sort_by(|a, b| a.0.cmp(&b.0));
        serde_json::to_string(&sorted).map_err(|e| DescriptorError(e.to_string()))
    }

    /// Interface fingerprint: blake3 over the canonical form.
    pub fn fingerprint(&self) -> Result<String, DescriptorError> {
        Ok(crate::semref::blake3_hex(self.canonical()?.as_bytes()))
    }

    pub fn encode(&self) -> Result<Vec<u8>, DescriptorError> {
        serde_json::to_vec(self).map_err(|e| DescriptorError(e.to_string()))
    }

    pub fn decode(bytes: &[u8]) -> Result<ModuleDescriptor, DescriptorError> {
        let d: ModuleDescriptor =
            serde_json::from_slice(bytes).map_err(|e| DescriptorError(format!("decode: {e}")))?;
        if d.format != DESCRIPTOR_FORMAT {
            return Err(DescriptorError(format!(
                "unknown descriptor format '{}' (known: '{}')",
                d.format, DESCRIPTOR_FORMAT
            )));
        }
        Ok(d)
    }
}

/// F-797-R4-02: a CONSUMABLE descriptor for the C interop provider's
/// promised-surface entries (the validation lanes the support matrix
/// claims) — the same descriptor/2 language as AutoABI modules, so
/// consumers admit C providers by the same contract.
pub fn interop_c_provider_descriptor() -> ModuleDescriptor {
    let mut d = ModuleDescriptor::probe_module();
    d.symbols = vec![
        SymbolEntry {
            name: "p797_cbuf_sum_utf8".into(),
            form: SymbolForm::Fallible {
                ctx: None,
                params: vec![ParamKind::BufViewUtf8, ParamKind::U64],
                out: ParamKind::I64,
            },
        },
        SymbolEntry {
            name: "p797_crec_valid".into(),
            form: SymbolForm::Fallible {
                ctx: None,
                params: vec![ParamKind::RecView {
                    rec: "RecMix".into(),
                }],
                out: ParamKind::I64,
            },
        },
    ];
    d
}

/// Scalar bridge for record fields shared with layout.
pub fn scalar_of(s: SemScalar) -> ParamKind {
    match s {
        SemScalar::Bool => ParamKind::Bool,
        SemScalar::U8 => ParamKind::I32, // u8 crosses as i32 lane at T-02; no public u8 params
        SemScalar::I32 => ParamKind::I32,
        SemScalar::U32 => ParamKind::U32,
        SemScalar::I64 => ParamKind::I64,
        SemScalar::U64 => ParamKind::U64,
        SemScalar::F64 => ParamKind::F64,
        SemScalar::Char => ParamKind::Char,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_roundtrip_and_fingerprint_stability() {
        let d = ModuleDescriptor::probe_module();
        let bytes = d.encode().unwrap();
        let back = ModuleDescriptor::decode(&bytes).unwrap();
        assert_eq!(d, back);
        let fp1 = d.fingerprint().unwrap();
        let fp2 = ModuleDescriptor::probe_module().fingerprint().unwrap();
        assert_eq!(fp1, fp2, "fingerprint must be content-stable");
    }

    #[test]
    fn fingerprint_tracks_signature_change() {
        let mut d = ModuleDescriptor::probe_module();
        let fp_before = d.fingerprint().unwrap();
        if let SymbolForm::Pure { params, .. } = &mut d.symbols[0].form {
            params[0] = ParamKind::U32;
        }
        assert_ne!(fp_before, d.fingerprint().unwrap());
    }

    #[test]
    fn unknown_format_rejected_at_decode() {
        let mut d = ModuleDescriptor::probe_module();
        d.format = "autoabi.descriptor/999".into();
        let bytes = serde_json::to_vec(&d).unwrap();
        let err = ModuleDescriptor::decode(&bytes).unwrap_err();
        assert!(err.0.contains("unknown descriptor format"));
    }
}
