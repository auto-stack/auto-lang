//! Managed cache units for the ac-av probe (PLAN-798 §5.4).
//!
//! Reuses the PLAN-783 `ManagedStore` through its public API ONLY, as an
//! isolated producer namespace: this plan's OWN `LayerSpec`s describe the
//! A08 (MIR) / A09 (ABC + manifest) seams it actually implements, payloads
//! ride the generic `ChunkKind::LayoutData` carrier (never tagged Hir or
//! Object), and every write/read validates the inner envelope's producer,
//! schema, owner and stage. Nothing here modifies 783's registry, ChunkKind
//! set or GC internals.
//!
//! Unit model: MIR/ABC persist PER FUNCTION; a module link manifest maps
//! unit names to artifact ids. Callee bodies are addressed through unit
//! imports (`fn:<name>`), so a body edit re-keys only its own unit while
//! callers' code units stay reusable; a SIGNATURE edit changes every
//! caller's fingerprint (the caller's canonical HIR dump embeds callee
//! signature content). Cache keys always include owner/profile, body+sig
//! content, algorithm/schema and MIR/ABC/AV semantic versions plus the
//! capability-affecting configuration; the source-map sidecar is keyed
//! separately (position changes share code content, sidecar re-binds).
//!
//! Restore is never trusted: every restored payload re-decodes, re-enters
//! the real verifiers, and re-checks the envelope against the caller's
//! expected identity. A hit still executes AV.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{LoadedArtifact, ManagedStore, PutRequest, StoreError};
use auto_av_core::contract::{fnv1a64_parts, ABC_SCHEMA, AV_PROFILE, MIR_SCHEMA, PRODUCER_ID};
use auto_av_core::mir::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// This plan's layer seams (independent of 783's registry)
// ---------------------------------------------------------------------------

pub const MIR_LAYER: LayerSpec = LayerSpec {
    id: "A08",
    name: "semantic-lowering-passes (plan798 MIR units)",
    schema_id: "ac798/a08-mir/v1",
    producer: PRODUCER_ID,
    consumers: &["av probe ABC emission", "diagnostics"],
    key_inputs: &[
        "producer", "owner-module", "unit", "profile", "body-sig-content", "callee-sigs",
        "mir-schema", "mir-semantics", "av-semantics", "capabilities-config",
    ],
    ref_kinds: &["provenance"],
    storage: "one binary unit per function inside the shared managed store (LayoutData carrier)",
    residency: "per-function units; module manifest references units by artifact id",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

pub const ABC_LAYER: LayerSpec = LayerSpec {
    id: "A09",
    name: "backend-object (plan798 ABC units + module manifest)",
    schema_id: "ac798/a09-abc/v1",
    producer: PRODUCER_ID,
    consumers: &["av executor closure assembly"],
    key_inputs: &[
        "producer", "owner-module", "unit", "profile", "body-sig-content", "callee-sigs",
        "abc-codec", "abc-semantics", "av-semantics", "capabilities-config",
    ],
    ref_kinds: &["required-data", "provenance"],
    storage: "one ABC unit per function + module link manifest (LayoutData carrier)",
    residency: "restored units re-verify before execution; never trusted flags",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};

// ---------------------------------------------------------------------------
// Source sidecar codec + merge
// ---------------------------------------------------------------------------

/// Encode one function's span table: count, then (instr, func, start, end).
pub fn encode_sidecar(spans: &[(u32, auto_av_core::contract::SpanInfo)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + spans.len() * 16);
    out.extend_from_slice(&(spans.len() as u32).to_le_bytes());
    for (instr, sp) in spans {
        out.extend_from_slice(&instr.to_le_bytes());
        out.extend_from_slice(&sp.func.to_le_bytes());
        out.extend_from_slice(&sp.start.to_le_bytes());
        out.extend_from_slice(&sp.end.to_le_bytes());
    }
    out
}

pub fn decode_sidecar(bytes: &[u8]) -> Result<Vec<(u32, auto_av_core::contract::SpanInfo)>, CacheError> {
    if bytes.len() < 4 {
        return Err(CacheError::Codec("sidecar truncated".into()));
    }
    let n = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    if bytes.len() != 4 + n * 16 {
        return Err(CacheError::Codec(format!(
            "sidecar length {} does not match {} entries",
            bytes.len(),
            n
        )));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let b = &bytes[4 + i * 16..4 + (i + 1) * 16];
        out.push((
            u32::from_le_bytes(b[0..4].try_into().unwrap()),
            auto_av_core::contract::SpanInfo {
                func: u32::from_le_bytes(b[4..8].try_into().unwrap()),
                start: u32::from_le_bytes(b[8..12].try_into().unwrap()),
                end: u32::from_le_bytes(b[12..16].try_into().unwrap()),
            },
        ));
    }
    Ok(out)
}

/// Merge a decoded sidecar into a span-less unit (diagnostics track only).
pub fn merge_spans(unit: &mut MirModule, spans: &[(u32, auto_av_core::contract::SpanInfo)]) {
    if let Some(f) = unit.funcs.first_mut() {
        f.spans = spans.to_vec();
    }
}

pub fn sidecar_key(owner: &str, unit: &str, snapshot: u64) -> QueryKey {
    QueryKey::new(
        &MIR_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("kind".into(), "source-sidecar".into()),
            ("unit".into(), unit.into()),
            ("snapshot".into(), format!("{:016x}", snapshot)),
        ],
    )
}

// ---------------------------------------------------------------------------
// Inner envelope
// ---------------------------------------------------------------------------

pub const ENVELOPE_MAGIC: &[u8; 4] = b"AVEN";
pub const ENVELOPE_VERSION: u16 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvelopeKind {
    MirUnit = 1,
    AbcUnit = 2,
    ModuleManifest = 3,
    /// Diagnostics-only source-map track: spans + snapshot per unit. The
    /// SEPARATE mapping track — code-content artifacts never carry spans, so
    /// position changes share code units and only re-bind the sidecar
    /// (PLAN-798 §5.4).
    SourceSidecar = 4,
}

#[derive(Clone, Debug)]
pub struct EnvelopeIdentity<'a> {
    pub kind: EnvelopeKind,
    pub producer: &'a str,
    pub schema: &'a str,
    pub owner_module: &'a str,
    /// 1 = MIR stage, 2 = ABC stage (manifests live at the ABC stage).
    pub stage: u8,
    pub mir_sem: u32,
    pub abc_sem: u32,
    pub av_sem: u32,
    pub snapshot: u64,
}

impl EnvelopeIdentity<'_> {
    pub fn mir(owner: &str, snapshot: u64) -> EnvelopeIdentity<'_> {
        EnvelopeIdentity {
            kind: EnvelopeKind::MirUnit,
            producer: PRODUCER_ID,
            schema: MIR_SCHEMA,
            owner_module: owner,
            stage: 1,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot,
        }
    }

    pub fn abc_raw(owner: &str, snapshot: u64) -> EnvelopeIdentity<'_> {
        EnvelopeIdentity {
            kind: EnvelopeKind::AbcUnit,
            producer: PRODUCER_ID,
            schema: ABC_SCHEMA,
            owner_module: owner,
            stage: 2,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot,
        }
    }

    /// Code-content identities carry NO snapshot: position changes share
    /// code units (only the sidecar re-binds). Callers pass 0.
    pub fn abc(owner: &str) -> EnvelopeIdentity<'_> {
        Self::abc_raw(owner, 0)
    }

    pub fn manifest(owner: &str, snapshot: u64) -> EnvelopeIdentity<'_> {
        EnvelopeIdentity {
            kind: EnvelopeKind::ModuleManifest,
            producer: PRODUCER_ID,
            schema: ABC_SCHEMA,
            owner_module: owner,
            stage: 2,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot,
        }
    }

    /// The diagnostics-only source-map track identity.
    pub fn sidecar(owner: &str, snapshot: u64) -> EnvelopeIdentity<'_> {
        EnvelopeIdentity {
            kind: EnvelopeKind::SourceSidecar,
            producer: PRODUCER_ID,
            schema: MIR_SCHEMA,
            owner_module: owner,
            stage: 1,
            mir_sem: 1,
            abc_sem: 1,
            av_sem: 1,
            snapshot,
        }
    }
}

#[derive(Debug)]
pub enum CacheError {
    Store(StoreError),
    Envelope(String),
    Codec(String),
    Verify(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Store(e) => write!(f, "store: {}", e),
            CacheError::Envelope(m) => write!(f, "envelope: {}", m),
            CacheError::Codec(m) => write!(f, "codec: {}", m),
            CacheError::Verify(m) => write!(f, "verify: {}", m),
        }
    }
}

impl From<StoreError> for CacheError {
    fn from(e: StoreError) -> Self {
        CacheError::Store(e)
    }
}

pub fn envelope_wrap(payload: &[u8], id: &EnvelopeIdentity) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 96);
    out.extend_from_slice(ENVELOPE_MAGIC);
    out.extend_from_slice(&ENVELOPE_VERSION.to_le_bytes());
    out.push(id.kind as u8);
    out.push(id.stage);
    out.extend_from_slice(&id.mir_sem.to_le_bytes());
    out.extend_from_slice(&id.abc_sem.to_le_bytes());
    out.extend_from_slice(&id.av_sem.to_le_bytes());
    out.extend_from_slice(&id.snapshot.to_le_bytes());
    for s in [id.producer, id.schema, id.owner_module] {
        out.extend_from_slice(&(s.len() as u16).to_le_bytes());
        out.extend_from_slice(s.as_bytes());
    }
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(payload);
    let sum = fnv1a64_parts(&[&out]);
    out.extend_from_slice(&sum.to_le_bytes());
    out
}

/// Unwrap + validate an envelope against the EXPECTED identity. Every field
/// must match: a self-consistent envelope from another producer, schema,
/// owner or stage is a controlled rejection, not a hit.
pub fn envelope_unwrap(bytes: &[u8], expect: &EnvelopeIdentity) -> Result<Vec<u8>, CacheError> {
    let bad = |m: String| Err(CacheError::Envelope(m));
    if bytes.len() < 4 + 2 + 1 + 1 + 12 + 8 + 8 {
        return bad("envelope truncated".to_string());
    }
    if &bytes[0..4] != ENVELOPE_MAGIC {
        return bad("not an AVEN envelope".to_string());
    }
    let ver = u16::from_le_bytes([bytes[4], bytes[5]]);
    if ver != ENVELOPE_VERSION {
        return bad(format!("unknown envelope version {}", ver));
    }
    let body_end = bytes.len() - 8;
    let want = u64::from_le_bytes(bytes[body_end..].try_into().unwrap());
    let got = fnv1a64_parts(&[&bytes[..body_end]]);
    if want != got {
        return bad(format!(
            "envelope checksum mismatch: stored {:016x}, computed {:016x}",
            want, got
        ));
    }
    let mut p = 6usize;
    let kind = bytes[p];
    p += 1;
    let stage = bytes[p];
    p += 1;
    let mir_sem = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap());
    p += 4;
    let abc_sem = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap());
    p += 4;
    let av_sem = u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap());
    p += 4;
    let snapshot = u64::from_le_bytes(bytes[p..p + 8].try_into().unwrap());
    p += 8;
    let mut strs = Vec::with_capacity(3);
    for _ in 0..3 {
        let n = u16::from_le_bytes(bytes[p..p + 2].try_into().unwrap()) as usize;
        p += 2;
        if p + n > body_end {
            return bad("envelope string overruns body".to_string());
        }
        strs.push(String::from_utf8_lossy(&bytes[p..p + n]).to_string());
        p += n;
    }
    let plen = u64::from_le_bytes(bytes[p..p + 8].try_into().unwrap()) as usize;
    p += 8;
    if p + plen != body_end {
        return bad(format!(
            "envelope payload length {} does not close the body at {}",
            plen, body_end
        ));
    }
    if kind != expect.kind as u8 {
        return bad(format!("envelope kind {} is not the expected {:?}", kind, expect.kind));
    }
    if stage != expect.stage {
        return bad(format!("envelope stage {} is not the expected {}", stage, expect.stage));
    }
    if strs[0] != expect.producer {
        return bad(format!("envelope producer `{}` is not `{}`", strs[0], expect.producer));
    }
    if strs[1] != expect.schema {
        return bad(format!("envelope schema `{}` is not `{}`", strs[1], expect.schema));
    }
    if strs[2] != expect.owner_module {
        return bad(format!("envelope owner `{}` is not `{}`", strs[2], expect.owner_module));
    }
    if (mir_sem, abc_sem, av_sem) != (expect.mir_sem, expect.abc_sem, expect.av_sem) {
        return bad(format!(
            "envelope semantics ({},{},{}) do not match expected ({},{},{})",
            mir_sem, abc_sem, av_sem, expect.mir_sem, expect.abc_sem, expect.av_sem
        ));
    }
    if snapshot != expect.snapshot {
        return bad(format!("envelope snapshot {} is not {}", snapshot, expect.snapshot));
    }
    Ok(bytes[p..p + plen].to_vec())
}

// ---------------------------------------------------------------------------
// Canonical HIR body fingerprint (pre-MIR, per unit)
// ---------------------------------------------------------------------------

use auto_ac_prototype::hir;

/// Deterministic canonical dump of one function's verified HIR content:
/// locals, places, expressions, blocks/stmts and loop order, plus the
/// SIGNATURE content of every distinct callee (so a callee signature change
/// re-keys its callers while a body-only change does not). Spans and
/// cosmetic fields are excluded.
pub fn canonical_hir_function(module: &hir::Module, body_id: hir::BodyId) -> Vec<u8> {
    let body = &module.bodies[body_id.0 as usize];
    let mut c = CanonBuf { out: Vec::with_capacity(1024) };
    // Locals.
    c.w("[loc");
    for l in &body.locals {
        c.w(" ");
        c.w(match l.role {
            hir::LocalRole::Param => "p",
            hir::LocalRole::Binding => "b",
        });
        c.b(l.mutable);
        c.w(":");
        c.w(&shape_of_use(&l.ty, module));
    }
    c.w("]");
    // Places.
    c.w("[pl");
    for p in &body.places {
        c.w(" ");
        c.w(&shape_of_use(&p.ty, module));
        c.b(matches!(p.access, hir::Access::Mutable));
    }
    c.w("]");
    // Exprs (table order; graph shape is verify-guaranteed).
    c.w("[ex");
    for e in &body.exprs {
        c.w(" (");
        c.w(&shape_of_use(&e.ty, module));
        c.w(" ");
        match &e.kind {
            hir::ExprKind::ReadLocal { local } => {
                c.w("rl ");
                c.w(&local.0.to_string());
            }
            hir::ExprKind::Constant { value } => {
                c.w("k ");
                c.w(&value.to_string());
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                c.w("b ");
                c.w(op.as_str());
                c.w(" ");
                c.w(&lhs.0.to_string());
                c.w(" ");
                c.w(&rhs.0.to_string());
            }
            hir::ExprKind::Unary { op, operand } => {
                c.w("u ");
                c.w(op.as_str());
                c.w(" ");
                c.w(&operand.0.to_string());
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                c.w("l ");
                c.w(op.as_str());
                c.w(" ");
                c.w(&lhs.0.to_string());
                c.w(" ");
                c.w(&rhs.0.to_string());
            }
            hir::ExprKind::Call { callee, eval_args, bindings } => {
                c.w("c ");
                let callee_name = module
                    .defs
                    .get(callee.0 as usize)
                    .map(|d| d.name_for_diagnostic().to_string())
                    .unwrap_or_else(|| "?".to_string());
                c.w(&callee_name);
                // Embed the callee SIGNATURE content (dependency edge).
                if let Some(d) = module.defs.get(callee.0 as usize) {
                    if let Some(ty) = d.kind.fn_type() {
                        c.w("<");
                        c.w(&sig_content(module, ty));
                        c.w(">");
                    }
                }
                for a in eval_args {
                    c.w(" ");
                    c.w(&a.0.to_string());
                }
                c.w(" |");
                let mut bs: Vec<_> = bindings.iter().collect();
                bs.sort_by_key(|b| b.param);
                for b in bs {
                    c.w(" ");
                    c.w(&b.param.to_string());
                    c.w(":");
                    c.w(&b.arg.to_string());
                }
            }
            other => {
                c.w("?unsupported:");
                c.w(&format!("{:?}", std::mem::discriminant(other)));
            }
        }
        c.w(")");
    }
    c.w("]");
    // Blocks/stmts.
    c.w("[bl");
    for b in &body.blocks {
        c.w(" {");
        for s in &b.stmts {
            match s {
                hir::Stmt::Let { local, value } => {
                    c.w(" let ");
                    c.w(&local.0.to_string());
                    c.w(" ");
                    c.w(&value.0.to_string());
                }
                hir::Stmt::Return { value } => {
                    c.w(" ret ");
                    c.w(&value.0.to_string());
                }
                hir::Stmt::If { cond, then, els } => {
                    c.w(" if ");
                    c.w(&cond.0.to_string());
                    c.w(" ");
                    c.w(&then.0.to_string());
                    if let Some(e) = els {
                        c.w(" ");
                        c.w(&e.0.to_string());
                    }
                }
                hir::Stmt::Loop { id, body } => {
                    c.w(" loop ");
                    c.w(&id.0.to_string());
                    c.w(" ");
                    c.w(&body.0.to_string());
                }
                hir::Stmt::Assign { place, value } => {
                    c.w(" as ");
                    c.w(&place.0.to_string());
                    c.w(" ");
                    c.w(&value.0.to_string());
                }
                hir::Stmt::Break { target } => {
                    c.w(" brk ");
                    c.w(&target.0.to_string());
                }
                hir::Stmt::Continue { target } => {
                    c.w(" cont ");
                    c.w(&target.0.to_string());
                }
                hir::Stmt::Drop { local } => {
                    c.w(" drop ");
                    c.w(&local.0.to_string());
                }
            }
        }
        c.w(" }");
    }
    c.w("]");
    // Loop registration order.
    c.w("[lp");
    for l in &body.loops {
        c.w(" ");
        c.w(l);
    }
    c.w("]");
    c.out
}

/// Small write helper keeping the canonical dump allocation-light.
struct CanonBuf {
    out: Vec<u8>,
}

impl CanonBuf {
    fn w(&mut self, s: &str) {
        self.out.extend_from_slice(s.as_bytes());
    }
    fn b(&mut self, v: bool) {
        self.out.push(v as u8);
    }
}

fn shape_of_use(u: &hir::TypeUse, m: &hir::Module) -> String {
    let s = match u {
        hir::TypeUse::Builtin(b) => b.as_str(),
        hir::TypeUse::Ref(tid) => match m.types.get(tid.0 as usize).map(|t| &t.kind) {
            Some(hir::TypeKind::I32) => "i32",
            Some(hir::TypeKind::Bool) => "bool",
            _ => "other",
        },
    };
    s.to_string()
}

fn sig_content(m: &hir::Module, ty: hir::TypeId) -> String {
    match m.types.get(ty.0 as usize).map(|t| &t.kind) {
        Some(hir::TypeKind::Function(sig)) => {
            let mut s = String::from("(");
            for p in &sig.params {
                s.push_str(&shape_of_use(p, m));
                s.push(',');
            }
            s.push_str(")->");
            s.push_str(&shape_of_use(&sig.result, m));
            s
        }
        _ => "?".to_string(),
    }
}

/// Body+signature fingerprint for unit keys.
pub fn unit_body_fingerprint(module: &hir::Module, body_id: hir::BodyId) -> String {
    let dump = canonical_hir_function(module, body_id);
    format!("{:016x}", fnv1a64_parts(&[&dump]))
}

/// Capability-affecting configuration fingerprint (sorted, deduped).
pub fn caps_fingerprint(caps: &[String]) -> String {
    let mut v: Vec<&str> = caps.iter().map(|s| s.as_str()).collect();
    v.sort_unstable();
    v.dedup();
    format!("{:016x}", fnv1a64_parts(&[v.join(",").as_bytes()]))
}

// ---------------------------------------------------------------------------
// Unit split / closure assembly
// ---------------------------------------------------------------------------

/// Split a lowered module into per-function self-contained units. Calls to
/// other module functions become unit imports (`fn:<name>`) carrying the
/// callee signature; test-mark imports are copied verbatim.
pub fn split_units(m: &MirModule) -> Result<Vec<MirModule>, String> {
    let mut units = Vec::with_capacity(m.funcs.len());
    for f in m.funcs.iter() {
        let mut unit = MirModule::unit(m.owner.clone());
        unit.capabilities = m.capabilities.clone();
        let mut ty_map: Vec<TypeIdx> = vec![u32::MAX; m.types.len()];
        let mut c_map: Vec<u32> = vec![u32::MAX; m.consts.len()];
        // Needed types: own sig + slots + callee sigs + referenced import sigs.
        let mut needed: Vec<TypeIdx> = vec![f.sig];
        for s in &f.slots {
            needed.push(s.ty);
        }
        for b in &f.blocks {
            for i in &b.instrs {
                if let MirInstr::Call { callee, .. } = i {
                    match callee {
                        CalleeRef::Func(t) => needed.push(m.funcs[*t as usize].sig),
                        CalleeRef::Import(ii) => needed.push(m.imports[*ii as usize].sig),
                    }
                }
            }
        }
        // Transitive closure over Fn-sig param/result types: scalars intern
        // immediately; Fn types intern once their children are remapped
        // (v1 sigs have scalar children only, but keep the fixpoint general).
        loop {
            let mut progress = false;
            let mut scan = 0usize;
            while scan < needed.len() {
                let t = needed[scan];
                scan += 1;
                if ty_map[t as usize] != u32::MAX {
                    continue;
                }
                match &m.types[t as usize] {
                    MirType::Fn(sig) => {
                        let mut children_pending = false;
                        for p in sig.params.iter().chain(std::iter::once(&sig.result)) {
                            if ty_map[*p as usize] == u32::MAX {
                                children_pending = true;
                                if !needed.contains(p) {
                                    needed.push(*p);
                                }
                            }
                        }
                        if !children_pending {
                            let mut ns =
                                FnSig { params: Vec::with_capacity(sig.params.len()), result: 0 };
                            for p in &sig.params {
                                ns.params.push(ty_map[*p as usize]);
                            }
                            ns.result = ty_map[sig.result as usize];
                            let idx = unit.intern_type(MirType::Fn(ns));
                            ty_map[t as usize] = idx;
                            progress = true;
                        }
                    }
                    _ => {
                        let idx = unit.intern_type(m.types[t as usize].clone());
                        ty_map[t as usize] = idx;
                        progress = true;
                    }
                }
            }
            if !progress {
                break;
            }
        }
        if needed.iter().any(|t| ty_map[*t as usize] == u32::MAX) {
            return Err(format!("unit `{}`: unresolved type closure", f.name));
        }

        // Rewrite instructions into unit-local indices.
        let mut blocks = f.blocks.clone();
        for b in blocks.iter_mut() {
            for instr in b.instrs.iter_mut() {
                match instr {
                    MirInstr::Const { value, .. } => {
                        if c_map[*value as usize] == u32::MAX {
                            let idx = unit.intern_const(m.consts[*value as usize]);
                            c_map[*value as usize] = idx;
                        }
                        *value = c_map[*value as usize];
                    }
                    MirInstr::Call { callee, .. } => match *callee {
                        CalleeRef::Func(target) => {
                            let tdef = &m.funcs[target as usize];
                            let local_sig = ty_map[tdef.sig as usize];
                            let idx = unit.intern_import(MirImport {
                                symbol: format!("fn:{}", tdef.name),
                                effect: Effect::Pure,
                                sig: local_sig,
                            });
                            *callee = CalleeRef::Import(idx);
                        }
                        CalleeRef::Import(ii) => {
                            let src = &m.imports[ii as usize];
                            let idx = unit.intern_import(MirImport {
                                symbol: src.symbol.clone(),
                                effect: src.effect,
                                sig: ty_map[src.sig as usize],
                            });
                            *callee = CalleeRef::Import(idx);
                        }
                    },
                    _ => {}
                }
            }
        }
        let slots = f
            .slots
            .iter()
            .map(|s| MirSlot { kind: s.kind, ty: ty_map[s.ty as usize], name: s.name.clone() })
            .collect();
        unit.funcs.push(MirFunction {
            owner: f.owner.clone(),
            name: f.name.clone(),
            sig: ty_map[f.sig as usize],
            slots,
            blocks,
            entry: f.entry,
            source_snapshot: f.source_snapshot,
            spans: f.spans.clone(),
        });
        unit.entry = 0;
        units.push(unit);
    }
    Ok(units)
}

/// Assemble an executable module closure from verified single-function ABC
/// units. Every `fn:<name>` import must resolve to another unit with a
/// structurally equal signature (interned-type equality in the merged
/// table); unresolved names or signature mismatches are controlled
/// rejections. The assembled module still passes `verify_mir`/`verify_abc`
/// before execution — this is the complete-consistency gate, not a trust
/// grant.
pub fn assemble_closure(units: &[MirModule], owner: &str, entry: &str) -> Result<MirModule, String> {
    let mut merged = MirModule::new(OwnerScope {
        schema: MIR_SCHEMA.to_string(),
        module: owner.to_string(),
    });
    merged.capabilities = units
        .iter()
        .flat_map(|u| u.capabilities.iter().cloned())
        .collect::<Vec<_>>();
    merged.capabilities.sort();
    merged.capabilities.dedup();

    struct Fused {
        name: String,
        idx: FuncIdx,
        sig: TypeIdx,
    }
    let mut fused: Vec<Fused> = Vec::new();
    // (function index in merged, block, instr, target name, merged-space sig)
    let mut patches: Vec<(usize, usize, usize, String, TypeIdx)> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for u in units {
        let ty_map = merge_types(&mut merged, u);
        let c_map = merge_consts(&mut merged, u);
        let mut blocks = u.funcs[0].blocks.clone();
        for (bi, b) in blocks.iter_mut().enumerate() {
            for (ii, instr) in b.instrs.iter_mut().enumerate() {
                match instr {
                    MirInstr::Const { value, .. } => *value = c_map[*value as usize],
                    MirInstr::Call { callee, .. } => {
                        if let CalleeRef::Import(impo) = *callee {
                            let imp = &u.imports[impo as usize];
                            let sig_merged = ty_map[imp.sig as usize];
                            if let Some(fname) = imp.symbol.strip_prefix("fn:") {
                                // Deferred resolution: unit calls may be
                                // recursive/mutual, so targets merge later.
                                patches.push((
                                    merged.funcs.len(),
                                    bi,
                                    ii,
                                    fname.to_string(),
                                    sig_merged,
                                ));
                                *callee = CalleeRef::Func(u32::MAX);
                            } else {
                                let idx = merged.intern_import(MirImport {
                                    symbol: imp.symbol.clone(),
                                    effect: imp.effect,
                                    sig: sig_merged,
                                });
                                *callee = CalleeRef::Import(idx);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let slots = u.funcs[0]
            .slots
            .iter()
            .map(|s| MirSlot { kind: s.kind, ty: ty_map[s.ty as usize], name: s.name.clone() })
            .collect();
        let sig = ty_map[u.funcs[0].sig as usize];
        if !matches!(merged.types.get(sig as usize), Some(MirType::Fn(_))) {
            return Err(format!("unit `{}` sig is not a function type", u.funcs[0].name));
        }
        fused.push(Fused { name: u.funcs[0].name.clone(), idx: merged.funcs.len() as FuncIdx, sig });
        names.push(u.funcs[0].name.clone());
        merged.funcs.push(MirFunction {
            owner: merged.owner.clone(),
            name: u.funcs[0].name.clone(),
            sig,
            slots,
            blocks,
            entry: u.funcs[0].entry,
            source_snapshot: u.funcs[0].source_snapshot,
            spans: u.funcs[0].spans.clone(),
        });
    }
    // Duplicate unit names are a closure inconsistency.
    {
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        if sorted.len() != names.len() {
            return Err("duplicate unit names in closure".to_string());
        }
    }
    // Patch deferred callees; interned-type equality is content equality.
    for (fi, bi, ii, target, sig_merged) in patches {
        let t = fused
            .iter()
            .find(|f| f.name == target)
            .ok_or_else(|| format!("closure import `fn:{}` does not resolve to any unit", target))?;
        if t.sig != sig_merged {
            return Err(format!(
                "closure signature mismatch for `fn:{}`: call site type #{} != unit type #{}",
                target, sig_merged, t.sig
            ));
        }
        merged.funcs[fi].blocks[bi].instrs[ii] = match &merged.funcs[fi].blocks[bi].instrs[ii] {
            MirInstr::Call { dst, args, .. } => MirInstr::Call {
                callee: CalleeRef::Func(t.idx),
                dst: *dst,
                args: args.clone(),
            },
            other => unreachable!("patch site is a call: {:?}", other),
        };
    }
    let entry_idx = merged
        .funcs
        .iter()
        .position(|f| f.name == entry)
        .ok_or_else(|| format!("entry function `{}` not among units", entry))?;
    merged.entry = entry_idx as u32;
    Ok(merged)
}

fn merge_types(merged: &mut MirModule, u: &MirModule) -> Vec<TypeIdx> {
    let mut map = vec![u32::MAX; u.types.len()];
    // Scalars intern immediately; Fn sigs intern once their children are
    // mapped (fixpoint loop, general over nested sigs).
    loop {
        let mut progress = false;
        for (i, t) in u.types.iter().enumerate() {
            if map[i] != u32::MAX {
                continue;
            }
            match t {
                MirType::I32 | MirType::Bool => {
                    map[i] = merged.intern_type(t.clone());
                    progress = true;
                }
                MirType::Fn(sig) => {
                    let pending = sig
                        .params
                        .iter()
                        .chain(std::iter::once(&sig.result))
                        .any(|p| map[*p as usize] == u32::MAX);
                    if !pending {
                        let mut ns = FnSig { params: Vec::with_capacity(sig.params.len()), result: 0 };
                        for p in &sig.params {
                            ns.params.push(map[*p as usize]);
                        }
                        ns.result = map[sig.result as usize];
                        let idx = merged.intern_type(MirType::Fn(ns));
                        map[i] = idx;
                        progress = true;
                    }
                }
            }
        }
        if !progress {
            break;
        }
    }
    map
}

fn merge_consts(merged: &mut MirModule, u: &MirModule) -> Vec<u32> {
    let mut map = vec![0u32; u.consts.len()];
    for (i, c) in u.consts.iter().enumerate() {
        map[i] = merged.intern_const(*c);
    }
    map
}

// ---------------------------------------------------------------------------
// Module link manifest
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnitLink {
    pub name: String,
    pub artifact: String,
    /// Canonical signature content `(t,t)->t` for closure cross-checks.
    pub sig: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModuleManifest {
    pub owner: String,
    pub entry: String,
    pub profile: String,
    pub av_sem: u32,
    pub units: Vec<UnitLink>,
}

impl ModuleManifest {
    pub fn fingerprint(&self) -> String {
        let mut parts: Vec<String> = self
            .units
            .iter()
            .map(|u| format!("{}={}:{}", u.name, u.artifact, u.sig))
            .collect();
        parts.sort();
        let joined = format!(
            "{}|{}|{}|{}|{}",
            self.owner,
            self.entry,
            self.profile,
            self.av_sem,
            parts.join(";")
        );
        format!("{:016x}", fnv1a64_parts(&[joined.as_bytes()]))
    }
}

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

pub fn mir_unit_key(owner: &str, unit: &str, body_fp: &str, caps_fp: &str) -> QueryKey {
    QueryKey::new(
        &MIR_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("unit".into(), unit.into()),
            ("profile".into(), AV_PROFILE.into()),
            ("body-sig".into(), body_fp.into()),
            ("mir-schema".into(), MIR_SCHEMA.into()),
            ("mir-sem".into(), "1".into()),
            ("av-sem".into(), "1".into()),
            ("caps".into(), caps_fp.into()),
        ],
    )
}

pub fn abc_unit_key(owner: &str, unit: &str, body_fp: &str, caps_fp: &str) -> QueryKey {
    QueryKey::new(
        &ABC_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("unit".into(), unit.into()),
            ("profile".into(), AV_PROFILE.into()),
            ("body-sig".into(), body_fp.into()),
            ("abc-codec".into(), ABC_SCHEMA.into()),
            ("abc-sem".into(), "1".into()),
            ("av-sem".into(), "1".into()),
            ("caps".into(), caps_fp.into()),
        ],
    )
}

pub fn manifest_key(owner: &str, entry: &str, manifest_fp: &str) -> QueryKey {
    QueryKey::new(
        &ABC_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("kind".into(), "module-manifest".into()),
            ("entry".into(), entry.into()),
            ("profile".into(), AV_PROFILE.into()),
            ("units-fp".into(), manifest_fp.into()),
            ("av-sem".into(), "1".into()),
        ],
    )
}

// ---------------------------------------------------------------------------
// Store operations
// ---------------------------------------------------------------------------

/// Put one enveloped payload as a LayoutData-carried artifact.
pub fn put_enveloped(
    store: &ManagedStore,
    key: &QueryKey,
    layer: &'static str,
    id: &EnvelopeIdentity,
    payload: &[u8],
    edges: Vec<(String, ac_artifact_probe::identity::RefKind)>,
    project: Option<&str>,
) -> Result<String, CacheError> {
    let env = envelope_wrap(payload, id);
    let art = store.put_artifact(PutRequest {
        layer,
        key: key.clone(),
        kind: ChunkKind::LayoutData, // generic carrier; NEVER Hir/Object
        payload: env,
        edges,
        project: project.map(|s| s.to_string()),
    })?;
    Ok(art)
}

/// Get + validate one enveloped payload; `None` on a clean miss.
pub fn get_enveloped_opt(
    store: &ManagedStore,
    key: &QueryKey,
    layer: &str,
    expect: &EnvelopeIdentity,
) -> Result<Option<Vec<u8>>, CacheError> {
    match store.get_artifact(layer, key)? {
        None => Ok(None),
        Some(loaded) => {
            if loaded.kind != ChunkKind::LayoutData {
                return Err(CacheError::Envelope(format!(
                    "artifact {} carries kind {:?}, expected the LayoutData carrier",
                    loaded.artifact_id, loaded.kind
                )));
            }
            let payload = envelope_unwrap(&loaded.payload, expect)?;
            Ok(Some(payload))
        }
    }
}

/// Load an artifact payload by ARTIFACT ID (restore path: the manifest holds
/// unit artifact ids; the store's address-admission still guards the read).
pub fn get_by_artifact_id(
    store: &ManagedStore,
    artifact_id: &str,
    expect: &EnvelopeIdentity,
) -> Result<Vec<u8>, CacheError> {
    #[allow(unused_imports)]
    use rusqlite::OptionalExtension;
    let db = store.db();
    let chunk: String = db
        .query_row(
            "SELECT payload_chunk FROM artifacts WHERE artifact_id = ?1",
            [artifact_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| CacheError::Store(StoreError::Io(e.to_string())))?
        .ok_or_else(|| CacheError::Envelope(format!("artifact {} not present", artifact_id)))?;
    drop(db);
    let framed = store
        .read_chunk_gated(chunk.as_str())?
        .ok_or_else(|| CacheError::Envelope(format!("artifact {} chunk missing", artifact_id)))?;
    let (kind, payload) = ac_artifact_probe::codec::unframe_chunk(&framed)
        .map_err(|e| CacheError::Codec(e.to_string()))?;
    if kind != ChunkKind::LayoutData {
        return Err(CacheError::Envelope(format!(
            "artifact {} carries kind {:?}, expected the LayoutData carrier",
            artifact_id, kind
        )));
    }
    envelope_unwrap(&payload, expect)
}

/// Convenience for `LoadedArtifact` payload access.
pub fn loaded_payload(loaded: &LoadedArtifact) -> &[u8] {
    &loaded.payload
}

/// Driver counter map type.
pub type Counters = BTreeMap<String, u64>;

/// Open (creating when absent) the probe's managed store.
pub fn open_store(root: &Path) -> Result<ManagedStore, CacheError> {
    Ok(ManagedStore::open(root)?)
}
