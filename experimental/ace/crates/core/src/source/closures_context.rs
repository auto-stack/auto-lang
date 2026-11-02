//! PLAN-913 T-03/T-05: the portable DECLARATION CONTEXT of a retained
//! closures frontend.
//!
//! The 912 warm path ([`ClosuresFunctionFrontend::edit_body`]) needs the
//! retained `CheckedEnv` only for its DECLARATIONS: the resolver's
//! signature/record/list/variant/function-type tables, each function's
//! signature, the unit catalog (spans, logical ids, creation-site facts) and
//! the verified closures registry. It never reads another function's resolved
//! BODY — `resolve_lambda_in_context` builds its own locals from the site
//! facts and the single re-check grafts one body into the environment.
//!
//! That is what makes a source-bound restore possible at all: the context is
//! portable data. This module dumps it ([`dump_context`]), and rebuilds a
//! SAFE context from untrusted decoded data ([`restore_context`]) only after
//! cross-checking it against the units a real decoder+verifier admitted.
//!
//! Boundaries (declared, not hidden):
//!
//! - unchanged functions are restored with SIGNATURE-ONLY entries (empty
//!   bodies/locals) and only ever act as document scaffolds. An edit that
//!   needs a retained BODY (the owner route's lambda reuse) is therefore a
//!   named non-fast on a restored context — it takes the whole-module truth
//!   path and reports its production honestly.
//! - the registry's `cell`/`mutable` facts are not carried by MIR; they are
//!   admitted only when the decoded creator sites agree on the callee, the
//!   capture-mode sequence and the environment arity. A mismatch is a
//!   controlled rejection (`restored.context-mismatch`), never a silent
//!   widen.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::hir::{BuiltinType, CaptureMode, TypeId, TypeUse};
use crate::source::ast::Receiver;
use crate::source::closures_fn::{ClosuresFunctionFrontend, UnitEntry, UnitKind};
use crate::source::resolve::{
    checked_env_from_parts, fn_sig_from_parts, ClosureFnType, LambdaSiteFacts, LocalRef, RBlock,
    ResolvedField, ResolvedFn, ResolvedMethod, ResolvedProgram, ResolvedRecord, ResolvedVariant, VT,
};
use crate::verify::{ClosureEnvField, ClosuresVerifyContext, LambdaDecl};

/// The context dump's schema identity. A different id is a controlled STALE.
pub const CONTEXT_DUMP_SCHEMA: &str = "ace.closures.context-dump/1";

pub const MAX_DUMP_UNITS: usize = 4096;
pub const MAX_DUMP_FNS: usize = 4096;
pub const MAX_DUMP_TABLE: usize = 65536;
pub const MAX_DUMP_STRING: usize = 8192;

// ── untrusted mirror types (the serialized form) ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VtDump {
    I32,
    Bool,
    Str,
    StrOwned,
    Char,
    Rec(usize),
    Unit,
    List(usize),
    Variant(usize),
    Function(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltinDump {
    I32,
    Bool,
    StrSlice,
    StrOwned,
    Char,
    Unit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeUseDump {
    Builtin(BuiltinDump),
    Ref(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiverDump {
    Free,
    Static,
    View,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamModeDump {
    Default,
    Mut,
    Move,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureModeDump {
    Move,
    Borrow,
    Shared,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SigDump {
    pub name: String,
    pub param_names: Vec<String>,
    pub params: Vec<VtDump>,
    pub result: VtDump,
    pub index: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FnDump {
    pub name: String,
    pub name_span: (usize, usize),
    pub item_span: (usize, usize),
    pub param_names: Vec<String>,
    pub params: Vec<VtDump>,
    pub param_modes: Vec<bool>,
    pub result: VtDump,
    pub receiver: ReceiverDump,
    pub owner_rec: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDump {
    pub name: String,
    pub vt: VtDump,
    pub span: (usize, usize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MethodDump {
    pub name: String,
    pub is_static: bool,
    pub fn_index: usize,
    pub span: (usize, usize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordDump {
    pub name: String,
    pub name_span: (usize, usize),
    pub fields: Vec<FieldDump>,
    pub methods: Vec<MethodDump>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariantDump {
    pub name: String,
    pub name_span: (usize, usize),
    pub arms: Vec<(String, Option<VtDump>)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FnTypeDump {
    pub params: Vec<VtDump>,
    pub result: VtDump,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteDump {
    pub owner: String,
    pub ordinal: usize,
    pub name: String,
    pub expr_span: (usize, usize),
    pub move_capture: bool,
    pub params: Vec<(String, VtDump)>,
    pub capture_uses: Vec<(String, bool)>,
    pub capture_decls: Vec<(String, VtDump, CaptureModeDump, bool, bool)>,
    pub capture_sources: Vec<usize>,
    pub outer_names: Vec<String>,
    pub result: VtDump,
    pub type_index: usize,
    pub nested: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UnitKindDump {
    Owner { lambdas: Vec<usize> },
    Lambda { logical_id: u64, owner: usize, site: SiteDump },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitDump {
    pub name: String,
    pub kind: UnitKindDump,
    pub span: (usize, usize),
    pub body_span: (usize, usize),
    pub body_digest: String,
    pub mint_span: (usize, usize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvFieldDump {
    pub ty: TypeUseDump,
    pub mutable: bool,
    pub cell: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegistryDump {
    /// The lambda's def id text (`d_<name>`).
    pub callee: String,
    pub creator: String,
    pub site_modes: Vec<CaptureModeDump>,
    pub env: Vec<EnvFieldDump>,
}

/// The whole portable declaration context.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextDump {
    pub schema: String,
    pub source_sha: String,
    pub module_path: String,
    pub capabilities: Vec<String>,
    pub generation: u64,
    pub context_revision: String,
    pub next_logical: u64,
    pub trace_enabled: bool,
    pub units: Vec<UnitDump>,
    pub fns: Vec<FnDump>,
    pub records: Vec<RecordDump>,
    pub lists: Vec<VtDump>,
    pub variants: Vec<VariantDump>,
    pub function_types: Vec<FnTypeDump>,
    pub sigs: Vec<SigDump>,
    pub rec_index: Vec<(String, usize)>,
    pub registry: Vec<RegistryDump>,
    pub sites_by_creator: Vec<(String, Vec<String>)>,
}

/// Unit evidence from a REAL decoder+verifier (av builds it from the admitted
/// MIR): a lambda's environment arity and a creator's creation sites.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitEvidence {
    pub unit: String,
    pub env_arity: Option<usize>,
    pub sites: Vec<(String, Vec<CaptureModeDump>)>,
}

fn diag(code: &str, message: impl Into<String>) -> Vec<Diagnostic> {
    vec![Diagnostic::new(
        Stage::Verify,
        code,
        message,
        Span { start: 0, end: 0 },
    )]
}

fn span(p: (usize, usize)) -> Span {
    Span {
        start: p.0,
        end: p.1,
    }
}

fn pair(s: Span) -> (usize, usize) {
    (s.start, s.end)
}

// ── conversions: core → dump ────────────────────────────────────────────────

fn vt_dump(v: VT) -> VtDump {
    match v {
        VT::I32 => VtDump::I32,
        VT::Bool => VtDump::Bool,
        VT::Str => VtDump::Str,
        VT::StrOwned => VtDump::StrOwned,
        VT::Char => VtDump::Char,
        VT::Rec(i) => VtDump::Rec(i),
        VT::Unit => VtDump::Unit,
        VT::List(i) => VtDump::List(i),
        VT::Variant(i) => VtDump::Variant(i),
        VT::Function(i) => VtDump::Function(i),
    }
}

fn builtin_dump(b: BuiltinType) -> BuiltinDump {
    match b {
        BuiltinType::I32 => BuiltinDump::I32,
        BuiltinType::Bool => BuiltinDump::Bool,
        BuiltinType::StrSlice => BuiltinDump::StrSlice,
        BuiltinType::StrOwned => BuiltinDump::StrOwned,
        BuiltinType::Char => BuiltinDump::Char,
        BuiltinType::Unit => BuiltinDump::Unit,
    }
}

fn type_use_dump(t: TypeUse) -> TypeUseDump {
    match t {
        TypeUse::Builtin(b) => TypeUseDump::Builtin(builtin_dump(b)),
        TypeUse::Ref(id) => TypeUseDump::Ref(id.0),
    }
}

fn capture_dump(m: CaptureMode) -> CaptureModeDump {
    match m {
        CaptureMode::Move => CaptureModeDump::Move,
        CaptureMode::Borrow => CaptureModeDump::Borrow,
        CaptureMode::Shared => CaptureModeDump::Shared,
    }
}

fn receiver_dump(r: Receiver) -> ReceiverDump {
    match r {
        Receiver::Free => ReceiverDump::Free,
        Receiver::Static => ReceiverDump::Static,
        Receiver::View => ReceiverDump::View,
    }
}

fn site_dump(s: &LambdaSiteFacts) -> SiteDump {
    SiteDump {
        owner: s.owner.clone(),
        ordinal: s.ordinal,
        name: s.name.clone(),
        expr_span: pair(s.expr_span),
        move_capture: s.move_capture,
        params: s.params.iter().map(|(n, v)| (n.clone(), vt_dump(*v))).collect(),
        capture_uses: s.capture_uses.clone(),
        capture_decls: s
            .capture_decls
            .iter()
            .map(|(n, v, m, w, c)| (n.clone(), vt_dump(*v), capture_dump(*m), *w, *c))
            .collect(),
        capture_sources: s.capture_sources.iter().map(|r| r.0).collect(),
        outer_names: s.outer_names.iter().cloned().collect(),
        result: vt_dump(s.result),
        type_index: s.type_index,
        nested: s.nested,
    }
}

/// Dump this frontend's portable declaration context. Bodies are NOT
/// included: the context is declarations + catalog + registry by design.
pub fn dump_context(front: &ClosuresFunctionFrontend) -> ContextDump {
    let env = front.env();
    let units = front
        .units()
        .iter()
        .map(|u| UnitDump {
            name: format!("d_{}", u.name),            kind: match &u.kind {
                UnitKind::Owner { lambdas } => UnitKindDump::Owner {
                    lambdas: lambdas.clone(),
                },
                UnitKind::Lambda {
                    logical_id,
                    owner,
                    site,
                } => UnitKindDump::Lambda {
                    logical_id: *logical_id,
                    owner: *owner,
                    site: site_dump(site),
                },
            },
            span: pair(u.span),
            body_span: pair(u.body_span),
            body_digest: u.body_digest.clone(),
            mint_span: pair(u.mint_span),
        })
        .collect();
    let fns = env
        .program
        .fns
        .iter()
        .map(|f| FnDump {
            name: f.name.clone(),
            name_span: pair(f.name_span),
            item_span: pair(f.item_span),
            param_names: f.param_names.clone(),
            params: f.params.iter().map(|v| vt_dump(*v)).collect(),
            param_modes: f.param_modes.clone(),
            result: vt_dump(f.result),
            receiver: receiver_dump(f.receiver),
            owner_rec: f.owner_rec,
        })
        .collect();
    let records = env
        .program
        .records
        .iter()
        .map(|r| RecordDump {
            name: r.name.clone(),
            name_span: pair(r.name_span),
            fields: r
                .fields
                .iter()
                .map(|f| FieldDump {
                    name: f.name.clone(),
                    vt: vt_dump(f.vt),
                    span: pair(f.span),
                })
                .collect(),
            methods: r
                .methods
                .iter()
                .map(|m| MethodDump {
                    name: m.name.clone(),
                    is_static: m.is_static,
                    fn_index: m.fn_index,
                    span: pair(m.span),
                })
                .collect(),
        })
        .collect();
    let variants = env
        .program
        .variants
        .iter()
        .map(|v| VariantDump {
            name: v.name.clone(),
            name_span: pair(v.name_span),
            arms: v
                .arms
                .iter()
                .map(|(n, p)| (n.clone(), p.map(vt_dump)))
                .collect(),
        })
        .collect();
    let function_types = env
        .program
        .function_types
        .iter()
        .map(|f| FnTypeDump {
            params: f.params.iter().map(|v| vt_dump(*v)).collect(),
            result: vt_dump(f.result),
        })
        .collect();
    let sigs = env
        .sigs
        .iter()
        .map(|(name, s)| {
            let (param_names, params, result, index) = s.dump();
            SigDump {
                name: name.clone(),
                param_names,
                params: params.iter().map(|v| vt_dump(*v)).collect(),
                result: vt_dump(result),
                index,
            }
        })
        .collect();
    let rec_index = env.rec_index.iter().map(|(k, v)| (k.clone(), *v)).collect();
    let (lambdas, sites_index) = front.registry().declarations();
    let registry: Vec<RegistryDump> = lambdas
        .iter()
        .map(|(callee, decl)| RegistryDump {
            callee: callee.clone(),
            creator: decl.creator.clone(),
            site_modes: decl.site_modes.iter().map(|m| capture_dump(*m)).collect(),
            env: decl
                .env
                .iter()
                .map(|f| EnvFieldDump {
                    ty: type_use_dump(f.ty),
                    mutable: f.mutable,
                    cell: f.cell,
                })
                .collect(),
        })
        .collect();
    let sites_by_creator: Vec<(String, Vec<String>)> = sites_index
        .iter()
        .map(|(creator, sites)| (creator.clone(), sites.clone()))
        .collect();
    ContextDump {
        schema: CONTEXT_DUMP_SCHEMA.to_string(),
        source_sha: front.source_sha().to_string(),
        module_path: front.source_path().to_string(),
        capabilities: front.capabilities().to_vec(),
        generation: front.generation(),
        context_revision: front.context_revision().to_string(),
        next_logical: front.next_logical_id(),
        trace_enabled: env.trace_enabled,
        units,
        fns,
        records,
        lists: env.program.lists.iter().map(|v| vt_dump(*v)).collect(),
        variants,
        function_types,
        sigs,
        rec_index,
        registry,
        sites_by_creator,
    }
}

// ── conversions: dump → core (validated) ────────────────────────────────────

fn vt_in(d: VtDump) -> VT {
    match d {
        VtDump::I32 => VT::I32,
        VtDump::Bool => VT::Bool,
        VtDump::Str => VT::Str,
        VtDump::StrOwned => VT::StrOwned,
        VtDump::Char => VT::Char,
        VtDump::Rec(i) => VT::Rec(i),
        VtDump::Unit => VT::Unit,
        VtDump::List(i) => VT::List(i),
        VtDump::Variant(i) => VT::Variant(i),
        VtDump::Function(i) => VT::Function(i),
    }
}

fn builtin_in(d: BuiltinDump) -> BuiltinType {
    match d {
        BuiltinDump::I32 => BuiltinType::I32,
        BuiltinDump::Bool => BuiltinType::Bool,
        BuiltinDump::StrSlice => BuiltinType::StrSlice,
        BuiltinDump::StrOwned => BuiltinType::StrOwned,
        BuiltinDump::Char => BuiltinType::Char,
        BuiltinDump::Unit => BuiltinType::Unit,
    }
}

fn type_use_in(d: TypeUseDump) -> TypeUse {
    match d {
        TypeUseDump::Builtin(b) => TypeUse::Builtin(builtin_in(b)),
        TypeUseDump::Ref(id) => TypeUse::Ref(TypeId(id)),
    }
}

fn capture_in(d: CaptureModeDump) -> CaptureMode {
    match d {
        CaptureModeDump::Move => CaptureMode::Move,
        CaptureModeDump::Borrow => CaptureMode::Borrow,
        CaptureModeDump::Shared => CaptureMode::Shared,
    }
}

fn receiver_in(d: ReceiverDump) -> Receiver {
    match d {
        ReceiverDump::Free => Receiver::Free,
        ReceiverDump::Static => Receiver::Static,
        ReceiverDump::View => Receiver::View,
    }
}

fn site_in(d: &SiteDump) -> LambdaSiteFacts {
    LambdaSiteFacts {
        owner: d.owner.clone(),
        ordinal: d.ordinal,
        name: d.name.clone(),
        expr_span: span(d.expr_span),
        move_capture: d.move_capture,
        params: d.params.iter().map(|(n, v)| (n.clone(), vt_in(*v))).collect(),
        capture_uses: d.capture_uses.clone(),
        capture_decls: d
            .capture_decls
            .iter()
            .map(|(n, v, m, w, c)| (n.clone(), vt_in(*v), capture_in(*m), *w, *c))
            .collect(),
        capture_sources: d.capture_sources.iter().map(|i| LocalRef(*i)).collect(),
        outer_names: d.outer_names.iter().cloned().collect(),
        result: vt_in(d.result),
        type_index: d.type_index,
        nested: d.nested,
    }
}

fn check_dump_bounds(d: &ContextDump) -> Result<(), Vec<Diagnostic>> {
    if d.schema != CONTEXT_DUMP_SCHEMA {
        return Err(diag(
            "restored.stale",
            format!("context dump schema {} is not {CONTEXT_DUMP_SCHEMA}", d.schema),
        ));
    }
    let bounded = |n: usize, cap: usize, what: &str| -> Result<(), Vec<Diagnostic>> {
        if n > cap {
            return Err(diag(
                "restored.bounds",
                format!("{what} has {n} entries (cap {cap})"),
            ));
        }
        Ok(())
    };
    bounded(d.units.len(), MAX_DUMP_UNITS, "units")?;
    bounded(d.fns.len(), MAX_DUMP_FNS, "functions")?;
    bounded(d.records.len(), MAX_DUMP_TABLE, "records")?;
    bounded(d.lists.len(), MAX_DUMP_TABLE, "lists")?;
    bounded(d.variants.len(), MAX_DUMP_TABLE, "variants")?;
    bounded(d.function_types.len(), MAX_DUMP_TABLE, "function types")?;
    bounded(d.sigs.len(), MAX_DUMP_FNS, "sigs")?;
    bounded(d.registry.len(), MAX_DUMP_TABLE, "registry")?;
    bounded(d.rec_index.len(), MAX_DUMP_TABLE, "record index")?;
    let strings = d
        .units
        .iter()
        .map(|u| u.name.len() + u.body_digest.len())
        .chain(d.fns.iter().map(|f| f.name.len() + f.param_names.len() * 8))
        .chain(std::iter::once(d.source_sha.len()))
        .chain(std::iter::once(d.module_path.len()));
    if strings.clone().any(|n| n > MAX_DUMP_STRING) {
        return Err(diag(
            "restored.bounds",
            format!("a context string exceeds {MAX_DUMP_STRING} bytes"),
        ));
    }
    // A dump with no units or no functions is not a context.
    if d.units.is_empty() || d.fns.is_empty() {
        return Err(diag(
            "restored.empty",
            "a context dump needs at least one unit and one function",
        ));
    }
    Ok(())
}

fn check_evidence(d: &ContextDump, evidence: &[UnitEvidence]) -> Result<(), Vec<Diagnostic>> {
    if evidence.len() != d.units.len() {
        return Err(diag(
            "restored.evidence",
            format!(
                "decoded evidence covers {} units, the context declares {}",
                evidence.len(),
                d.units.len()
            ),
        ));
    }
    let by_unit: BTreeMap<&str, &UnitEvidence> =
        evidence.iter().map(|e| (e.unit.as_str(), e)).collect();
    for u in &d.units {
        let Some(ev) = by_unit.get(u.name.as_str()) else {
            return Err(diag(
                "restored.evidence",
                format!("unit {} has no decoded evidence", u.name),
            ));
        };
        match &u.kind {
            UnitKindDump::Lambda { .. } => {
                let env_len = ev.env_arity.ok_or_else(|| {
                    diag(
                        "restored.context-mismatch",
                        format!("unit {} is a lambda but its MIR declares no environment", u.name),
                    )
                })?;
                let reg = d.registry.iter().find(|r| r.callee == u.name).ok_or_else(|| {
                    diag(
                        "restored.context-mismatch",
                        format!("lambda unit {} has no registry declaration", u.name),
                    )
                })?;
                if reg.env.len() != env_len {
                    return Err(diag(
                        "restored.context-mismatch",
                        format!(
                            "lambda {} environment arity {} != decoded {}",
                            u.name,
                            reg.env.len(),
                            env_len
                        ),
                    ));
                }
            }
            UnitKindDump::Owner { .. } => {
                if ev.env_arity.is_some() {
                    return Err(diag(
                        "restored.context-mismatch",
                        format!("owner unit {} carries a closure environment", u.name),
                    ));
                }
                // The creator's decoded sites must agree with the registry:
                // same callees, same capture-mode sequence, same order.
                let decl_sites = d
                    .sites_by_creator
                    .iter()
                    .find(|(c, _)| *c == u.name)
                    .map(|(_, s)| s.clone())
                    .unwrap_or_default();
                if decl_sites.len() != ev.sites.len() {
                    return Err(diag(
                        "restored.context-mismatch",
                        format!(
                            "creator {} declares {} sites, decoded MIR has {}",
                            u.name,
                            decl_sites.len(),
                            ev.sites.len()
                        ),
                    ));
                }
                for (declared, (callee, modes)) in decl_sites.iter().zip(ev.sites.iter()) {
                    if declared != callee {
                        return Err(diag(
                            "restored.context-mismatch",
                            format!(
                                "creator {} site order differs: declared {declared}, decoded {callee}",
                                u.name
                            ),
                        ));
                    }
                    let reg = d.registry.iter().find(|r| r.callee == *callee).ok_or_else(|| {
                        diag(
                            "restored.context-mismatch",
                            format!("site callee {callee} has no registry declaration"),
                        )
                    })?;
                    let declared_modes: Vec<CaptureModeDump> = reg.site_modes.clone();
                    if declared_modes != *modes {
                        return Err(diag(
                            "restored.context-mismatch",
                            format!(
                                "site {callee} capture modes differ: declared {declared_modes:?}, decoded {modes:?}"
                            ),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Rebuild a SAFE frontend context from a dump that a REAL decoder+verifier
/// cross-checked. `source` is the caller's source text: it must be the exact
/// source the dump was formed from.
pub fn restore_context(
    dump: ContextDump,
    source: &str,
    module_path: &str,
    capabilities: &[String],
    evidence: &[UnitEvidence],
) -> Result<ClosuresFunctionFrontend, Vec<Diagnostic>> {
    check_dump_bounds(&dump)?;
    let source_sha = crate::runtime_contract::sha256_hex(source.as_bytes());
    if dump.source_sha != source_sha {
        return Err(diag(
            "restored.source-mismatch",
            "the context dump binds a different source SHA-256",
        ));
    }
    if dump.module_path != module_path {
        return Err(diag(
            "restored.context-mismatch",
            format!(
                "the context dump binds module `{}`, the request carries `{module_path}`",
                dump.module_path
            ),
        ));
    }
    if dump.capabilities != capabilities {
        return Err(diag(
            "restored.capability",
            "the context dump's capabilities differ from the current host grants",
        ));
    }
    check_evidence(&dump, evidence)?;
    // The catalog and the signature table must describe the SAME functions.
    if dump.units.len() != dump.fns.len() {
        return Err(diag(
            "restored.context-mismatch",
            format!(
                "catalog has {} units but the environment has {} functions",
                dump.units.len(),
                dump.fns.len()
            ),
        ));
    }
    for (u, f) in dump.units.iter().zip(dump.fns.iter()) {
        let wire = format!("d_{}", f.name);
        if u.name != wire {
            return Err(diag(
                "restored.context-mismatch",
                format!(
                    "catalog unit {} != environment function {} (wire {wire})",
                    u.name, f.name
                ),
            ));
        }
    }
    let mut fns: Vec<ResolvedFn> = Vec::with_capacity(dump.fns.len());
    for f in &dump.fns {
        fns.push(ResolvedFn {
            name: f.name.clone(),
            name_span: span(f.name_span),
            item_span: span(f.item_span),
            param_names: f.param_names.clone(),
            params: f.params.iter().map(|v| vt_in(*v)).collect(),
            param_modes: f.param_modes.clone(),
            result: vt_in(f.result),
            locals: Vec::new(),
            body: RBlock {
                stmts: Vec::new(),
                span: Span { start: 0, end: 0 },
            },
            loop_count: 0,
            used_intrinsics: BTreeSet::new(),
            receiver: receiver_in(f.receiver),
            owner_rec: f.owner_rec,
        });
    }
    let records = dump
        .records
        .iter()
        .map(|r| ResolvedRecord {
            name: r.name.clone(),
            name_span: span(r.name_span),
            fields: r
                .fields
                .iter()
                .map(|f| ResolvedField {
                    name: f.name.clone(),
                    vt: vt_in(f.vt),
                    span: span(f.span),
                })
                .collect(),
            methods: r
                .methods
                .iter()
                .map(|m| ResolvedMethod {
                    name: m.name.clone(),
                    is_static: m.is_static,
                    fn_index: m.fn_index,
                    span: span(m.span),
                })
                .collect(),
        })
        .collect();
    let variants = dump
        .variants
        .iter()
        .map(|v| ResolvedVariant {
            name: v.name.clone(),
            name_span: span(v.name_span),
            arms: v.arms.iter().map(|(n, p)| (n.clone(), p.map(vt_in))).collect(),
        })
        .collect();
    let function_types = dump
        .function_types
        .iter()
        .map(|f| ClosureFnType {
            params: f.params.iter().map(|v| vt_in(*v)).collect(),
            result: vt_in(f.result),
        })
        .collect();
    let mut sigs = BTreeMap::new();
    for s in &dump.sigs {
        let sig = fn_sig_from_parts(
            s.param_names.clone(),
            s.params.iter().map(|v| vt_in(*v)).collect(),
            vt_in(s.result),
            s.index,
        );
        if sigs.insert(s.name.clone(), sig).is_some() {
            return Err(diag(
                "restored.duplicate",
                format!("signature `{}` appears twice", s.name),
            ));
        }
    }
    let rec_index: BTreeMap<String, usize> = dump.rec_index.iter().cloned().collect();
    if rec_index.len() != dump.rec_index.len() {
        return Err(diag(
            "restored.duplicate",
            "a record index key appears twice",
        ));
    }
    // The registry: each lambda exactly one creator/site; creators agree.
    let mut lambdas: BTreeMap<String, LambdaDecl> = BTreeMap::new();
    for r in &dump.registry {
        let decl = LambdaDecl {
            creator: r.creator.clone(),
            site_modes: r.site_modes.iter().map(|m| capture_in(*m)).collect(),
            env: r
                .env
                .iter()
                .map(|f| ClosureEnvField {
                    ty: type_use_in(f.ty.clone()),
                    mutable: f.mutable,
                    cell: f.cell,
                })
                .collect(),
        };
        if lambdas.insert(r.callee.clone(), decl).is_some() {
            return Err(diag(
                "restored.duplicate",
                format!("registry entry `{}` appears twice", r.callee),
            ));
        }
    }
    let mut sites_by_creator: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (creator, sites) in &dump.sites_by_creator {
        if sites_by_creator
            .insert(creator.clone(), sites.clone())
            .is_some()
        {
            return Err(diag(
                "restored.duplicate",
                format!("creator `{creator}` appears twice in the site index"),
            ));
        }
    }
    let registry =
        ClosuresVerifyContext::from_declarations(lambdas, sites_by_creator).map_err(|e| {
            diag(
                "restored.registry",
                format!("the restored registry is inconsistent: {e}"),
            )
        })?;
    let env = checked_env_from_parts(
        ResolvedProgram {
            fns,
            records,
            lists: dump.lists.iter().map(|v| vt_in(*v)).collect(),
            variants,
            function_types,
        },
        rec_index,
        sigs,
        dump.trace_enabled,
    );
    let units: Vec<UnitEntry> = dump
        .units
        .iter()
        .map(|u| UnitEntry {
            // The dump keys units by WIRE identity (`d_<name>`), the catalog
            // stores the display name.
            name: u
                .name
                .strip_prefix("d_")
                .unwrap_or(u.name.as_str())
                .to_string(),
            kind: match &u.kind {
                UnitKindDump::Owner { lambdas } => UnitKind::Owner {
                    lambdas: lambdas.clone(),
                },
                UnitKindDump::Lambda {
                    logical_id,
                    owner,
                    site,
                } => UnitKind::Lambda {
                    logical_id: *logical_id,
                    owner: *owner,
                    site: site_in(site),
                },
            },
            span: span(u.span),
            body_span: span(u.body_span),
            body_digest: u.body_digest.clone(),
            mint_span: span(u.mint_span),
        })
        .collect();
    Ok(ClosuresFunctionFrontend::from_restored(
        env,
        source.to_string(),
        source_sha,
        module_path.to_string(),
        capabilities.to_vec(),
        units,
        registry,
        dump.next_logical,
        dump.generation,
        dump.context_revision,
    ))
}
