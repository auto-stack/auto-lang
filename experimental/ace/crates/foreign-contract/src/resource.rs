//! PLAN-939 CA-D3: the controlled RESOURCE projection of the foreign
//! contract (source declarations only — the typed peer keeps riding the
//! 937 typed bridge and the 797 descriptor3 directly).
//!
//! Scope frozen by T-01/T-02 (plan §5.1/§5.8): the resource forms a
//! CHECKED SOURCE declaration may spell are exactly
//!
//! * `ViewText`   — borrowed UTF-8 text view `{const uint8_t *ptr, u64 len}`
//!                  (source `str` view argument; len is a BYTE count);
//! * `Rec`        — view of a NOMINAL record whose public closure is
//!                  naturally aligned scalar fields, with role
//!                  Const/Mut/Out (source record argument; Mut needs the
//!                  `.mut` call-site borrow, Out is a caller slot the
//!                  provider commits only on success);
//! * `Handle`     — opaque provider-owned handle (physical `int64_t`;
//!                  source `int` via CHECKED i64<->i32 conversion — an
//!                  out-of-range value is a named rejection, never a
//!                  truncation);
//! * `Callback`   — synchronous per-call callback
//!                  `long long (*)(long long ctx, long long item)`
//!                  (source `fn(int,int)->int` thin trampoline; wide
//!                  typed callbacks stay on the 797 typed path);
//! * scalar `I32`/`Bool` lanes as in 938.
//!
//! Wide i64/u64/f64 PARAMS and public List/Variant/f32 shapes are NOT
//! spellable here (named rejection, plan §1 non-goals). The physical
//! projection below rides descriptor3-expressible `ParamKind` forms; the
//! 797 descriptor itself is never modified by this projection.
//!
//! This module is a pure model: validation and projection only.

use serde::{Deserialize, Serialize};

/// Resource-signature codec identity (strict, versioned).
pub const RESOURCE_SIG_FORMAT: &str = "autoabi.contract.resource-sig/1";

/// Capability required to declare/call any resource entry (938's
/// `foreign.scalar` stays required for the scalar lanes).
pub const RESOURCE_CAP: &str = "foreign.resource";

/// View/ownership role of a record parameter (mirrors descriptor3
/// `ViewRole`; spelled here so this crate stays backend-free).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewRole {
    Const,
    Mut,
    Out,
}

impl ViewRole {
    pub fn spell(self) -> &'static str {
        match self {
            ViewRole::Const => "const",
            ViewRole::Mut => "mut",
            ViewRole::Out => "out",
        }
    }
}

/// Public record-closure field lanes: the natural-align scalar set the
/// 797 profile admits in a record layout. SOURCE declarations are
/// restricted to `I32`/`Bool` fields (the checked source record domain);
/// wider fields are typed-peer-only (plan §5.1 record row).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldKind {
    I32,
    Bool,
    Char,
    U32,
    I64,
    U64,
    F64,
}

impl FieldKind {
    pub fn spell(self) -> &'static str {
        match self {
            FieldKind::I32 => "i32",
            FieldKind::Bool => "bool",
            FieldKind::Char => "char",
            FieldKind::U32 => "u32",
            FieldKind::I64 => "i64",
            FieldKind::U64 => "u64",
            FieldKind::F64 => "f64",
        }
    }
    /// Size/alignment (bytes) under the frozen natural-align packing.
    pub fn size_align(self) -> (u32, u32) {
        match self {
            FieldKind::Bool | FieldKind::I32 | FieldKind::U32 | FieldKind::Char => (4, 4),
            FieldKind::I64 | FieldKind::U64 | FieldKind::F64 => (8, 8),
        }
    }
    pub fn source_ok(self) -> bool {
        matches!(self, FieldKind::I32 | FieldKind::Bool)
    }
}

/// One resource-declaration parameter kind. The `spell()` forms are the
/// canonical signature tokens (comparison key between two sigs).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ResKind {
    I32,
    Bool,
    /// Borrowed UTF-8 text view (byte length; NUL is data, not a
    /// terminator; encoding validated before any call).
    ViewText,
    /// PLAN-939 S07 typed-peer-only lane: borrowed view of an i64
    /// ELEMENT array `{const int64_t *p, u64 n}` — n counts ELEMENTS,
    /// never bytes (descriptor3 `ParamKind::BufViewI64`). Element-size
    /// multiplication is checked before any access; existing i32 list
    /// memory is never reinterpreted as i64 elements.
    BufViewI64,
    /// PLAN-939 S07 typed-peer-only lane: bare wide i64 scalar (the
    /// original wide callback's ctx seed and friends). A checked SOURCE
    /// declaration cannot spell it (named rejection).
    I64,
    /// View of nominal record `rec` with role.
    Rec {
        rec: String,
        #[serde(default)]
        fields: Vec<(String, FieldKind)>,
        role: ViewRole,
    },
    /// Opaque handle owned by `provider` (physical i64).
    Handle {
        provider: String,
    },
    /// Synchronous per-call callback `(ctx, item) -> value`
    /// (physical `long long (*)(long long, long long)`); `ctx: false`
    /// spells the ctx-less callback form.
    Callback {
        #[serde(default)]
        ctx: bool,
    },
}

impl ResKind {
    pub fn spell(&self) -> String {
        match self {
            ResKind::I32 => "i32".into(),
            ResKind::I64 => "i64".into(),
            ResKind::Bool => "bool".into(),
            ResKind::ViewText => "view str".into(),
            ResKind::BufViewI64 => "view i64s".into(),
            ResKind::Rec { rec, role, .. } => format!("{rec}.{}", role.spell()),
            ResKind::Handle { provider } => format!("handle:{provider}"),
            ResKind::Callback { ctx } => {
                format!("callback:{}", if *ctx { "ctx" } else { "noctx" })
            }
        }
    }
    /// Number of PHYSICAL argument positions this kind occupies in the
    /// Windows-x64 call (view = ptr+len pair; the callback itself is one
    /// fnptr lane — its ctx seed travels as the NEXT i32 param, projected
    /// to the adjacent i64 ctx lane).
    pub fn phys_args(&self) -> usize {
        match self {
            ResKind::I32
            | ResKind::I64
            | ResKind::Bool
            | ResKind::Rec { .. }
            | ResKind::Handle { .. } => 1,
            ResKind::ViewText | ResKind::BufViewI64 => 2,
            ResKind::Callback { .. } => 1,
        }
    }
    pub fn is_rec_role(&self, role: ViewRole) -> bool {
        matches!(self, ResKind::Rec { role: r, .. } if *r == role)
    }
}

/// Value lanes a resource entry may RETURN (pure) or write to its
/// trailing out slots (fallible). `I64`/`U64`/`Handle` results reach the
/// source as a CHECKED i32 conversion (out-of-range is a named resource
/// rejection — never a silent truncation, plan §5.1).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ResOut {
    I32,
    Bool,
    I64,
    U64,
    Handle { provider: String },
}

impl ResOut {
    pub fn spell(&self) -> String {
        match self {
            ResOut::I32 => "i32".into(),
            ResOut::Bool => "bool".into(),
            ResOut::I64 => "i64".into(),
            ResOut::U64 => "u64".into(),
            ResOut::Handle { provider } => format!("handle:{provider}"),
        }
    }
}

/// Resource entry form: pure (direct return) or fallible (status return;
/// at most ONE value out for a source declaration — a second value out is
/// a named rejection and multi-out travels as `Rec{role: Out}` params,
/// plan §5.8 S05). The fallible ctx lane mirrors the 938 form: the
/// `context_symbol` binding supplies the ctx pointer.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ResourceForm {
    Pure {
        ret: ResOut,
    },
    Fallible {
        #[serde(default)]
        ctx: Option<bool>,
        #[serde(default)]
        out: Option<ResOut>,
    },
}

/// The resource signature carried by a resource `FnFact`
/// (`FnFact::resource`, manifest schema `ace.foreign.manual/2`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSig {
    pub format: String,
    pub params: Vec<ResKind>,
    pub form: ResourceForm,
    /// §5.3 failure-atomicity promise: REQUIRED true whenever a fallible
    /// entry takes Mut/Out record params (failure keeps every caller
    /// byte; the provider shadows and commits once).
    #[serde(default)]
    pub atomic: bool,
    /// §5.9 callback noescape promise: REQUIRED true whenever a callback
    /// param exists (the reviewed provider never stores the pointer).
    #[serde(default)]
    pub noescape: bool,
    /// PLAN-939 S07: this sig rides the TYPED-PEER domain (the original
    /// wide ABI shapes) rather than the checked-source domain. Typed-peer
    /// sigs may spell wide record fields (`i64`/`u64`/`f64`/`u32`) and
    /// the typed-only lanes [`ResKind::BufViewI64`] / [`ResKind::I64`];
    /// checked-source sigs cannot (the restrictions below are the
    /// source-domain contract and stay enforced when this is false).
    /// Absent = false on the wire (existing manifests decode and digest
    /// byte-identically).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub typed_peer: bool,
}

impl ResourceSig {
    pub fn validate(&self) -> Result<(), String> {
        if self.format != RESOURCE_SIG_FORMAT {
            return Err(format!(
                "unknown resource-sig format '{}' (known: '{RESOURCE_SIG_FORMAT}')",
                self.format
            ));
        }
        if self.params.len() > 16 {
            return Err("resource.params: at most 16 parameters".into());
        }
        let mut seen_rec = std::collections::BTreeMap::new();
        for (i, p) in self.params.iter().enumerate() {
            match p {
                ResKind::I32 | ResKind::Bool | ResKind::ViewText => {}
                ResKind::BufViewI64 | ResKind::I64 => {
                    if !self.typed_peer {
                        return Err(format!(
                            "resource.params[{i}]: {:?} is typed-peer only (a checked source \
                             declaration cannot spell the wide lane)",
                            p
                        ));
                    }
                }
                ResKind::Rec { rec, fields, role } => {
                    if !ident(rec) {
                        return Err(format!(
                            "resource.params[{i}]: record id must be an identifier"
                        ));
                    }
                    if fields.is_empty() {
                        return Err(format!(
                            "resource.params[{i}]: record '{rec}' must spell its field closure"
                        ));
                    }
                    // PLAN-939 S03: the SAME closure may appear in MULTIPLE
                    // lanes of one signature (the two-mut alias oracle);
                    // every occurrence must spell the IDENTICAL field
                    // closure (a different shape under one name is a
                    // rejection, not a silent merge).
                    match seen_rec.get(rec.as_str()) {
                        Some(known) => {
                            if known != fields.as_slice() {
                                return Err(format!(
                                    "resource.params[{i}]: record '{rec}' spells a different field closure than an earlier lane"
                                ));
                            }
                            let _ = role;
                        }
                        None => {
                            seen_rec.insert(rec.clone(), fields.clone());
                        }
                    }
                    let mut names = std::collections::BTreeSet::new();
                    for (name, kind) in fields {
                        if !ident(name) {
                            return Err(format!(
                                "resource.params[{i}]: record '{rec}' field name must be an identifier"
                            ));
                        }
                        if !names.insert(name.as_str()) {
                            return Err(format!(
                                "resource.params[{i}]: record '{rec}' duplicate field '{name}'"
                            ));
                        }
                        if !kind.source_ok() && !self.typed_peer {
                            return Err(format!(
                                "resource.params[{i}]: record '{rec}' field '{name}' kind {} is \
                                 not part of the checked source record domain (typed-peer only)",
                                kind.spell()
                            ));
                        }
                    }
                }
                ResKind::Handle { provider } => {
                    if !ident(provider) {
                        return Err(format!(
                            "resource.params[{i}]: handle provider must be an identifier"
                        ));
                    }
                }
                ResKind::Callback { ctx: has_ctx } => {
                    if !self.noescape {
                        return Err(format!(
                            "resource.params[{i}]: callback requires the noescape promise \
                             (storing callbacks are a named M2 successor, not this bridge)"
                        ));
                    }
                    // The ctx VALUE travels as an i32 seed argument
                    // immediately after the callback (sign-extended to the
                    // physical i64 ctx lane, e.g. p797_fold's `(cb, ctx,
                    // items, n)` shape). A ctx-ful callback without its
                    // seed — or a seed in a non-adjacent position — is a
                    // named rejection, never an implicit zero ctx. A
                    // typed-peer sig may spell the ORIGINAL wide seed
                    // (`i64`) the 797 fold physically takes.
                    if *has_ctx {
                        let next = self.params.get(i + 1);
                        let wide_seed_ok = self.typed_peer && matches!(next, Some(ResKind::I64));
                        if !matches!(next, Some(ResKind::I32)) && !wide_seed_ok {
                            return Err(format!(
                                "resource.params[{i}]: ctx-ful callback must be immediately \
                                 followed by an i32 ctx-seed parameter"
                            ));
                        }
                    }
                }
            }
        }
        match &self.form {
            ResourceForm::Pure { ret } => {
                if let ResOut::Handle { provider } = ret {
                    if !ident(provider) {
                        return Err("resource.ret: handle provider must be an identifier".into());
                    }
                }
                // A pure entry has no failure path to hang the atomic
                // commit on — mut/out record views require the fallible
                // form (plan §5.3).
                for (i, p) in self.params.iter().enumerate() {
                    if p.is_rec_role(ViewRole::Mut) || p.is_rec_role(ViewRole::Out) {
                        return Err(format!(
                            "resource.params[{i}]: mut/out record views require a fallible \
                             (atomic) form — a pure entry cannot spell the failure invariant"
                        ));
                    }
                }
            }
            ResourceForm::Fallible { out, .. } => {
                if let Some(ResOut::Handle { provider }) = out {
                    if !ident(provider) {
                        return Err("resource.out: handle provider must be an identifier".into());
                    }
                }
                let mut mut_or_out = false;
                for p in &self.params {
                    if p.is_rec_role(ViewRole::Mut) || p.is_rec_role(ViewRole::Out) {
                        mut_or_out = true;
                    }
                }
                if mut_or_out && !self.atomic {
                    return Err(
                        "resource.atomic: fallible entries with mut/out record params must \
                         promise failure atomicity (caller bytes unchanged on failure)"
                            .into(),
                    );
                }
            }
        }
        Ok(())
    }

    /// Canonical signature spelling — the comparison key.
    pub fn signature(&self) -> String {
        let params = self
            .params
            .iter()
            .map(|p| p.spell())
            .collect::<Vec<_>>()
            .join(",");
        match &self.form {
            ResourceForm::Pure { ret } => format!("pure({params})->{}", ret.spell()),
            ResourceForm::Fallible { out, .. } => match out {
                None => format!("fallible({params})->status"),
                Some(o) => format!("fallible({params})->status,out {}", o.spell()),
            },
        }
    }

    /// Total physical argument count (view/callback pairs count twice),
    /// excluding the fallible out slot.
    pub fn phys_param_count(&self) -> usize {
        self.params.iter().map(|p| p.phys_args()).sum()
    }
}

fn ident(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .next()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(params: Vec<ResKind>, form: ResourceForm) -> ResourceSig {
        ResourceSig {
            format: RESOURCE_SIG_FORMAT.into(),
            params,
            form,
            atomic: false,
            noescape: false,
            typed_peer: false,
        }
    }

    #[test]
    fn view_text_sig_validates() {
        let s = sig(
            vec![ResKind::ViewText],
            ResourceForm::Pure { ret: ResOut::U64 },
        );
        assert!(s.validate().is_ok());
        assert_eq!(s.signature(), "pure(view str)->u64");
        assert_eq!(s.phys_param_count(), 2);
    }

    #[test]
    fn rec_mut_fallible_requires_atomic() {
        let mut s = sig(
            vec![ResKind::Rec {
                rec: "Counter".into(),
                fields: vec![("value".into(), FieldKind::I32)],
                role: ViewRole::Mut,
            }],
            ResourceForm::Fallible {
                ctx: None,
                out: None,
            },
        );
        assert!(s.validate().unwrap_err().contains("atomic"));
        s.atomic = true;
        assert!(s.validate().is_ok());
    }

    #[test]
    fn callback_requires_noescape_and_ctx_seed() {
        let mut s = sig(
            vec![ResKind::Callback { ctx: true }],
            ResourceForm::Pure { ret: ResOut::I64 },
        );
        assert!(s.validate().unwrap_err().contains("noescape"));
        s.noescape = true;
        // ctx-ful callback without its adjacent i32 ctx seed -> reject
        assert!(s.validate().unwrap_err().contains("ctx-seed parameter"));
        s.params.push(ResKind::I32);
        assert!(s.validate().is_ok());
        assert_eq!(s.signature(), "pure(callback:ctx,i32)->i64");
        // ctx-less callback needs no seed
        let s2 = sig(
            vec![ResKind::Callback { ctx: false }],
            ResourceForm::Pure { ret: ResOut::I32 },
        );
        let s2 = ResourceSig {
            noescape: true,
            ..s2
        };
        assert!(s2.validate().is_ok());
    }

    #[test]
    fn wide_record_field_is_named_reject() {
        let s = sig(
            vec![ResKind::Rec {
                rec: "Wide".into(),
                fields: vec![("big".into(), FieldKind::I64)],
                role: ViewRole::Const,
            }],
            ResourceForm::Pure { ret: ResOut::I32 },
        );
        assert!(s.validate().unwrap_err().contains("typed-peer only"));
    }

    #[test]
    fn duplicate_rec_same_closure_admits_but_divergent_shape_rejects() {
        // PLAN-939 S03: the SAME closure may fill MULTIPLE lanes of one
        // signature (the two-mut alias oracle); only a DIVERGENT field
        // closure under one name is a rejection.
        let same = sig(
            vec![
                ResKind::Rec {
                    rec: "Counter".into(),
                    fields: vec![("value".into(), FieldKind::I32)],
                    role: ViewRole::Mut,
                },
                ResKind::Rec {
                    rec: "Counter".into(),
                    fields: vec![("value".into(), FieldKind::I32)],
                    role: ViewRole::Mut,
                },
            ],
            ResourceForm::Fallible {
                ctx: None,
                out: None,
            },
        );
        let mut same = same;
        same.atomic = true; // two-mut fallible promises caller-byte atomicity
        assert!(same.validate().is_ok(), "{:?}", same.validate());
        let divergent = sig(
            vec![
                ResKind::Rec {
                    rec: "Counter".into(),
                    fields: vec![("value".into(), FieldKind::I32)],
                    role: ViewRole::Mut,
                },
                ResKind::Rec {
                    rec: "Counter".into(),
                    fields: vec![("value".into(), FieldKind::Bool)],
                    role: ViewRole::Mut,
                },
            ],
            ResourceForm::Fallible {
                ctx: None,
                out: None,
            },
        );
        assert!(divergent
            .validate()
            .unwrap_err()
            .contains("different field closure"));
    }

    #[test]
    fn handle_provider_identity_travels() {
        let s = sig(
            vec![ResKind::Handle {
                provider: "r939".into(),
            }],
            ResourceForm::Fallible {
                ctx: None,
                out: Some(ResOut::Handle {
                    provider: "r939".into(),
                }),
            },
        );
        assert!(s.validate().is_ok());
        assert_eq!(
            s.signature(),
            "fallible(handle:r939)->status,out handle:r939"
        );
    }

    #[test]
    fn typed_peer_wide_lanes_and_fields_admit_but_source_rejects() {
        // The S07 WideFold shape: original wide callback + wide i64 seed +
        // element-count i64 view, wide record fields — typed-peer only.
        let wide = ResourceSig {
            typed_peer: true,
            noescape: true,
            ..sig(
                vec![
                    ResKind::Callback { ctx: true },
                    ResKind::I64,
                    ResKind::BufViewI64,
                ],
                ResourceForm::Pure { ret: ResOut::I64 },
            )
        };
        assert!(wide.validate().is_ok());
        assert_eq!(wide.signature(), "pure(callback:ctx,i64,view i64s)->i64");
        assert_eq!(wide.phys_param_count(), 4);
        // The wide seed/view lanes are NAMED rejections in the
        // checked-source domain (the adjacency rule fires on the wide
        // seed first for the full fold shape; the bare lanes reject
        // on their own).
        let mut source = wide.clone();
        source.typed_peer = false;
        let e = source.validate().unwrap_err();
        assert!(
            e.contains("ctx-seed parameter") || e.contains("typed-peer only"),
            "{e}"
        );
        let bare = ResourceSig {
            typed_peer: false,
            ..sig(
                vec![ResKind::BufViewI64],
                ResourceForm::Pure { ret: ResOut::I64 },
            )
        };
        assert!(bare.validate().unwrap_err().contains("typed-peer only"));
        let bare64 = ResourceSig {
            typed_peer: false,
            ..sig(vec![ResKind::I64], ResourceForm::Pure { ret: ResOut::I64 })
        };
        assert!(bare64.validate().unwrap_err().contains("typed-peer only"));
        // Wide record fields too.
        let mut rec = sig(
            vec![ResKind::Rec {
                rec: "RecMix".into(),
                fields: vec![
                    ("flag".into(), FieldKind::Bool),
                    ("count".into(), FieldKind::I32),
                    ("tag".into(), FieldKind::Char),
                    ("big".into(), FieldKind::I64),
                ],
                role: ViewRole::Const,
            }],
            ResourceForm::Pure { ret: ResOut::I64 },
        );
        assert!(rec.validate().unwrap_err().contains("typed-peer only"));
        rec.typed_peer = true;
        assert!(rec.validate().is_ok());
        assert_eq!(rec.signature(), "pure(RecMix.const)->i64");
    }

    #[test]
    fn typed_peer_wire_omits_the_flag_when_false() {
        // Existing manifests digest over bytes: `typed_peer: false` must
        // NOT appear on the wire (absent = false), and the field decodes
        // as false from old JSON.
        let s = sig(
            vec![ResKind::ViewText],
            ResourceForm::Pure { ret: ResOut::U64 },
        );
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("typed_peer"));
        let back: ResourceSig = serde_json::from_str(&json).unwrap();
        assert!(!back.typed_peer);
        let mut t = s.clone();
        t.typed_peer = true;
        assert!(serde_json::to_string(&t).unwrap().contains("typed_peer"));
    }
}
