use ace_artifacts::identity::QueryKey;
use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use ace_av_core::contract::{AV_PROFILE, PRODUCER_ID};
use ace_core::atom_text::Span;
use ace_core::hir;

// ---------------------------------------------------------------------------
// PLAN-828 T-03: function-level HIR artifacts — the A07 function-unit layer.
//
// Design 40 A07 target: versioned typed binary bodies + shared type/signature
// context + module manifest; a restored artifact is an UNTRUSTED description
// that must re-verify against the CURRENT context before adoption; the
// verification credential itself never persists. This codec is deliberately
// BOUNDED to the scalar/Core incremental surface (i32/bool free functions):
// any construct outside it is an encode/decode error, and both errors simply
// skip the artifact (the cold compile is the truth).
// ---------------------------------------------------------------------------

/// The function-HIR artifact layer (A07, function units). Distinct
/// versioned identity from the 783 module-bundle A07 (`ac783/a07-hir/v1`)
/// — no key or payload is shared with it.
///
/// Codec v2 (B1-C04): the persisted body's cross-table references (call
/// callees, `TypeUse::Ref`) are indices into the artifact's OWN
/// `callee_table`/`type_table` of stable id TEXTS — never the producing
/// module's array slots. v1 rows (raw producing-table slots) are not
/// readable under the v2 key and are never rewritten in place.
pub const HIR_LAYER: LayerSpec = LayerSpec {
    id: "A07",
    name: "public-hir function units (plan828 function-level HIR artifacts)",
    schema_id: "ac828/a07-fnhir/v2",
    producer: PRODUCER_ID,
    consumers: &["session warm restore (re-verified, never trusted)"],
    key_inputs: &[
        "producer",
        "owner-module",
        "unit",
        "profile",
        "hir-domain",
        "hir-schema-revision",
        "hir-codec",
        "capabilities-config",
        "body-content",
        "declaration-context",
    ],
    ref_kinds: &["required-data", "provenance"],
    storage: "one binary function-HIR artifact per function (LayoutData carrier)",
    residency: "restored only as an UNTRUSTED description; content, span-layout and declaration context must all re-verify against the current session before adoption",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

/// Hard codec bounds (bounded codec: any overshoot is an error, never a
/// panic, never an unbounded allocation).
const FN_HIR_MAX_PAYLOAD: usize = 8 * 1024 * 1024;
const FN_HIR_MAX_STRING: usize = 1 << 20;
const FN_HIR_MAX_COUNT: usize = 1 << 20;

/// One function's persisted HIR artifact: the typed binary body plus the
/// dependency facts the credential binds to (declaration-context
/// fingerprint, referenced intrinsic symbols, capabilities fingerprint).
/// The VERIFIER CONSTRUCTED credential is NOT part of the payload — on
/// restore the body re-verifies against the live context instead.
#[derive(Clone, Debug)]
pub struct FnHirArtifact {
    pub def_id: String,
    /// Span-free canonical content version (the content-history key axis).
    pub content_version: String,
    /// FNV-1a digest over the body's spans in declaration order — proves
    /// the artifact's span LAYOUT matches the current source before the
    /// body object may be adopted (the canonical dump alone is span-free).
    pub span_digest: u64,
    /// The declaration-context fingerprint this body was verified under.
    pub context_fingerprint: String,
    pub caps_fp: String,
    /// Intrinsic symbols this body actually reads (dependency facts);
    /// restore cross-checks the field against the REBOUND body's actual
    /// reads (B1-C01) — a forged list is a skipped artifact.
    pub intrinsics: Vec<String>,
    /// Stable cross-table identity (B1-C04): def id TEXTS the body's calls
    /// reference, first-use order. The persisted body's `Call.callee`
    /// DefIds are indices into THIS table, not any module's def array.
    pub callee_table: Vec<String>,
    /// Type id TEXTS for the body's `TypeUse::Ref`s, first-use order;
    /// persisted `TypeId`s index this table.
    pub type_table: Vec<String>,
    /// The body in ARTIFACT-LOCAL coordinates (see `callee_table`).
    pub body: hir::Body,
}

/// Normalize one body's cross-table references from the PRODUCING tables
/// (index → id text, e.g. a single-function candidate bundle's def table
/// or the cold module's) into artifact-local indices over stable id texts.
/// Array slots are never the cross-product identity — the id TEXT is
/// (B1-C04). `Err` = a reference outside the producing tables (no
/// artifact is deposited; the cold compile stays the truth).
pub fn normalize_fn_hir_body(
    producing_defs: &[String],
    producing_types: &[String],
    body: &hir::Body,
) -> Result<(hir::Body, Vec<String>, Vec<String>), String> {
    let mut callee_table: Vec<String> = Vec::new();
    let mut type_table: Vec<String> = Vec::new();
    let mut def_of = |raw: u32| -> Result<u32, String> {
        let name = producing_defs
            .get(raw as usize)
            .ok_or_else(|| format!("callee DefId({raw}) outside the producing def table"))?;
        let idx = callee_table
            .iter()
            .position(|n| n == name)
            .unwrap_or_else(|| {
                callee_table.push(name.clone());
                callee_table.len() - 1
            });
        Ok(idx as u32)
    };
    let mut ty_of = |u: &hir::TypeUse| -> Result<hir::TypeUse, String> {
        match u {
            hir::TypeUse::Builtin(_) => Ok(u.clone()),
            hir::TypeUse::Ref(t) => {
                let name = producing_types
                    .get(t.0 as usize)
                    .ok_or_else(|| format!("TypeId({}) outside the producing type table", t.0))?;
                let idx = type_table
                    .iter()
                    .position(|n| n == name)
                    .unwrap_or_else(|| {
                        type_table.push(name.clone());
                        type_table.len() - 1
                    });
                Ok(hir::TypeUse::Ref(hir::TypeId(idx as u32)))
            }
        }
    };
    let mut out = body.clone();
    for l in &mut out.locals {
        l.ty = ty_of(&l.ty)?;
    }
    for p in &mut out.places {
        p.ty = ty_of(&p.ty)?;
    }
    for e in &mut out.exprs {
        e.ty = ty_of(&e.ty)?;
        match &mut e.kind {
            hir::ExprKind::Call { callee, .. } | hir::ExprKind::CreateClosure { callee, .. } => {
                *callee = hir::DefId(def_of(callee.0)?);
            }
            _ => {}
        }
    }
    Ok((out, callee_table, type_table))
}

/// Re-bind a decoded artifact body from artifact-local table indices into
/// the CURRENT module's tables, by stable id text (B1-C04). Any name that
/// does not resolve in the current module fails — the artifact is then
/// skipped, never trusted.
pub fn rebind_fn_hir_body(a: &FnHirArtifact, current: &hir::Module) -> Result<hir::Body, String> {
    let mut out = a.body.clone();
    let def_of = |local: u32| -> Result<u32, String> {
        let name = a
            .callee_table
            .get(local as usize)
            .ok_or_else(|| format!("callee table index {local} out of bounds"))?;
        current
            .defs
            .iter()
            .position(|d| &d.id_text == name)
            .map(|i| i as u32)
            .ok_or_else(|| format!("callee `{name}` not present in the current module"))
    };
    let ty_of = |u: &hir::TypeUse| -> Result<hir::TypeUse, String> {
        match u {
            hir::TypeUse::Builtin(_) => Ok(u.clone()),
            hir::TypeUse::Ref(t) => {
                let name = a
                    .type_table
                    .get(t.0 as usize)
                    .ok_or_else(|| format!("type table index {} out of bounds", t.0))?;
                current
                    .types
                    .iter()
                    .position(|d| &d.id_text == name)
                    .map(|i| hir::TypeUse::Ref(hir::TypeId(i as u32)))
                    .ok_or_else(|| format!("type `{name}` not present in the current module"))
            }
        }
    };
    for l in &mut out.locals {
        l.ty = ty_of(&l.ty)?;
    }
    for p in &mut out.places {
        p.ty = ty_of(&p.ty)?;
    }
    for e in &mut out.exprs {
        e.ty = ty_of(&e.ty)?;
        match &mut e.kind {
            hir::ExprKind::Call { callee, .. } | hir::ExprKind::CreateClosure { callee, .. } => {
                *callee = hir::DefId(def_of(callee.0)?);
            }
            _ => {}
        }
    }
    Ok(out)
}

/// FNV-1a over every span in the body, in stable order.
pub fn fn_hir_span_digest(body: &hir::Body) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    let mix = |h: &mut u64, part: u64| {
        *h ^= part;
        *h = h.wrapping_mul(0x100000001b3);
    };
    mix(&mut h, body.span.start as u64);
    mix(&mut h, body.span.end as u64);
    for l in &body.locals {
        mix(&mut h, l.span.start as u64);
        mix(&mut h, l.span.end as u64);
    }
    for p in &body.places {
        mix(&mut h, p.span.start as u64);
        mix(&mut h, p.span.end as u64);
    }
    for e in &body.exprs {
        mix(&mut h, e.span.start as u64);
        mix(&mut h, e.span.end as u64);
    }
    for b in &body.blocks {
        mix(&mut h, b.span.start as u64);
        mix(&mut h, b.span.end as u64);
    }
    h
}

// -- encoding ---------------------------------------------------------------

struct FnHirEnc {
    out: Vec<u8>,
}

impl FnHirEnc {
    fn u32(&mut self, v: u32) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn b(&mut self, v: bool) {
        self.out.push(v as u8);
    }
    fn s(&mut self, v: &str) -> Result<(), String> {
        let b = v.as_bytes();
        if b.len() > FN_HIR_MAX_STRING {
            return Err(format!("string too long ({} bytes)", b.len()));
        }
        self.u32(b.len() as u32);
        self.out.extend_from_slice(b);
        Ok(())
    }
    fn count(&mut self, n: usize) -> Result<(), String> {
        if n > FN_HIR_MAX_COUNT {
            return Err(format!("table too large ({n})"));
        }
        self.u32(n as u32);
        Ok(())
    }
    fn span(&mut self, sp: &Span) {
        self.u32(sp.start as u32);
        self.u32(sp.end as u32);
    }
    fn ty(&mut self, u: &hir::TypeUse) -> Result<(), String> {
        match u {
            hir::TypeUse::Builtin(b) => {
                self.out.push(0);
                self.s(match b {
                    hir::BuiltinType::I32 => "i32",
                    hir::BuiltinType::Bool => "bool",
                    hir::BuiltinType::StrSlice | hir::BuiltinType::StrOwned => {
                        return Err("text builtin outside the scalar codec".into())
                    }
                    hir::BuiltinType::Char => return Err("char outside the scalar codec".into()),
                    // PLAN-862: the unit RESULT word (entry-unit profile).
                    hir::BuiltinType::Unit => "unit",
                })?;
            }
            hir::TypeUse::Ref(t) => {
                self.out.push(1);
                self.u32(t.0);
            }
        }
        Ok(())
    }
}

/// Encode one artifact (codec v2). `Err` = the body carries constructs
/// outside the bounded scalar surface (the artifact is simply not
/// deposited). The body must already be in ARTIFACT-LOCAL coordinates
/// (see [`normalize_fn_hir_body`]) — encode writes the reference tables
/// and the body's callee/type ids index into them.
pub fn encode_fn_hir(a: &FnHirArtifact) -> Result<Vec<u8>, String> {
    let mut e = FnHirEnc {
        out: Vec::with_capacity(1024),
    };
    e.out.extend_from_slice(b"FNH1");
    e.u32(2); // codec version (v2: artifact-local reference tables)
    e.s(&a.def_id)?;
    e.s(&a.content_version)?;
    e.u64(a.span_digest);
    e.s(&a.context_fingerprint)?;
    e.s(&a.caps_fp)?;
    e.count(a.intrinsics.len())?;
    for i in &a.intrinsics {
        e.s(i)?;
    }
    e.count(a.callee_table.len())?;
    for c in &a.callee_table {
        e.s(c)?;
    }
    e.count(a.type_table.len())?;
    for t in &a.type_table {
        e.s(t)?;
    }
    let b = &a.body;
    e.s(&b.id_text)?;
    e.u32(b.owner.0);
    e.u32(b.entry.0);
    e.span(&b.span);
    e.count(b.loops.len())?;
    for l in &b.loops {
        e.s(l)?;
    }
    e.count(b.locals.len())?;
    for l in &b.locals {
        e.s(&l.id_text)?;
        e.ty(&l.ty)?;
        e.s(&l.name)?;
        e.out.push(match l.role {
            hir::LocalRole::Param => 0,
            hir::LocalRole::Binding => 1,
            hir::LocalRole::Capture => 2,
        });
        match l.index {
            Some(i) => {
                e.out.push(1);
                e.u32(i);
            }
            None => e.out.push(0),
        }
        e.b(l.mutable);
        e.b(l.mutable_explicit);
        e.out.push(match l.param_mode {
            hir::ParamMode::View => 0,
            hir::ParamMode::MutBorrow => 1,
            hir::ParamMode::MoveOwn => 2,
        });
        e.span(&l.span);
    }
    e.count(b.places.len())?;
    for p in &b.places {
        e.s(&p.id_text)?;
        match &p.kind {
            hir::PlaceKind::Local => e.out.push(0),
            hir::PlaceKind::FieldPath(_) => {
                return Err("field-path place outside the scalar codec".into())
            }
        }
        e.u32(p.local.0);
        e.ty(&p.ty)?;
        e.out.push(match p.access {
            hir::Access::Mutable => 0,
            hir::Access::ReadOnly => 1,
        });
        e.span(&p.span);
    }
    e.count(b.exprs.len())?;
    for x in &b.exprs {
        e.s(&x.id_text)?;
        e.ty(&x.ty)?;
        match &x.kind {
            hir::ExprKind::ReadLocal { local } => {
                e.out.push(0);
                e.u32(local.0);
            }
            hir::ExprKind::Constant { value } => {
                e.out.push(1);
                e.i32(*value);
            }
            hir::ExprKind::Binary {
                op,
                overflow,
                lhs,
                rhs,
            } => {
                e.out.push(2);
                e.s(op.as_str())?;
                e.out.push(match overflow {
                    hir::OverflowPolicy::Trap => 0,
                });
                e.u32(lhs.0);
                e.u32(rhs.0);
            }
            hir::ExprKind::Unary { op, operand } => {
                e.out.push(3);
                e.s(op.as_str())?;
                e.u32(operand.0);
            }
            hir::ExprKind::Logical { op, lhs, rhs } => {
                e.out.push(4);
                e.s(op.as_str())?;
                e.u32(lhs.0);
                e.u32(rhs.0);
            }
            hir::ExprKind::Call {
                callee,
                eval_args,
                bindings,
            } => {
                e.out.push(5);
                e.u32(callee.0);
                e.count(eval_args.len())?;
                for a in eval_args {
                    e.u32(a.0);
                }
                e.count(bindings.len())?;
                for cb in bindings {
                    e.u32(cb.param);
                    e.u32(cb.arg);
                    e.b(cb.moved);
                }
            }
            hir::ExprKind::CallClosure { callee, eval_args } => {
                e.out.push(7);
                e.u32(callee.0);
                e.count(eval_args.len())?;
                for arg in eval_args {
                    e.u32(arg.0);
                }
            }
            hir::ExprKind::ClosureClone { operand } => {
                e.out.push(9);
                e.u32(operand.0);
            }
            hir::ExprKind::CreateClosure { callee, captures } => {
                e.out.push(8);
                e.u32(callee.0);
                e.count(captures.len())?;
                for capture in captures {
                    e.u32(capture.local.0);
                    e.out.push(match capture.mode {
                        hir::CaptureMode::Move => 0,
                        hir::CaptureMode::Borrow => 1,
                    });
                }
            }
            // PLAN-862: the unit value — tag 6, no payload (zero-width).
            hir::ExprKind::Unit => e.out.push(6),
            other => {
                return Err(format!(
                    "expr kind `{:?}` outside the scalar codec",
                    std::mem::discriminant(other)
                ))
            }
        }
        e.span(&x.span);
    }
    e.count(b.blocks.len())?;
    for blk in &b.blocks {
        e.s(&blk.id_text)?;
        e.span(&blk.span);
        e.count(blk.stmts.len())?;
        for st in &blk.stmts {
            match st {
                hir::Stmt::Let { local, value } => {
                    e.out.push(0);
                    e.u32(local.0);
                    e.u32(value.0);
                }
                hir::Stmt::Return { value } => {
                    e.out.push(1);
                    e.u32(value.0);
                }
                // PLAN-864: tag 11 = eval (unit-effect statement).
                hir::Stmt::Eval { value } => {
                    e.out.push(11);
                    e.u32(value.0);
                }
                hir::Stmt::If { cond, then, els } => {
                    e.out.push(2);
                    e.u32(cond.0);
                    e.u32(then.0);
                    match els {
                        Some(x) => {
                            e.out.push(1);
                            e.u32(x.0);
                        }
                        None => e.out.push(0),
                    }
                }
                hir::Stmt::Loop { id, body } => {
                    e.out.push(3);
                    e.u32(id.0);
                    e.u32(body.0);
                }
                hir::Stmt::Assign { place, value } => {
                    e.out.push(4);
                    e.u32(place.0);
                    e.u32(value.0);
                }
                hir::Stmt::Break { target } => {
                    e.out.push(5);
                    e.u32(target.0);
                }
                hir::Stmt::Continue { target } => {
                    e.out.push(6);
                    e.u32(target.0);
                }
                hir::Stmt::Drop { .. } => {
                    return Err("drop statement outside the scalar codec".into())
                }
            }
        }
    }
    if e.out.len() > FN_HIR_MAX_PAYLOAD {
        return Err(format!("payload too large ({} bytes)", e.out.len()));
    }
    Ok(e.out)
}

// -- decoding ---------------------------------------------------------------

struct FnHirDec<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> FnHirDec<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.pos + n > self.b.len() {
            return Err("truncated payload".into());
        }
        let s = &self.b[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, String> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn i32(&mut self) -> Result<i32, String> {
        let s = self.take(4)?;
        Ok(i32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn u64(&mut self) -> Result<u64, String> {
        let s = self.take(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(s);
        Ok(u64::from_le_bytes(a))
    }
    fn b(&mut self) -> Result<bool, String> {
        Ok(self.u8()? != 0)
    }
    fn s(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        if n > FN_HIR_MAX_STRING {
            return Err("string length out of bounds".into());
        }
        let s = self.take(n)?;
        String::from_utf8(s.to_vec()).map_err(|_| "invalid utf-8".to_string())
    }
    fn count(&mut self) -> Result<usize, String> {
        let n = self.u32()? as usize;
        if n > FN_HIR_MAX_COUNT {
            return Err("table count out of bounds".into());
        }
        Ok(n)
    }
    fn span(&mut self) -> Result<Span, String> {
        let start = self.u32()? as usize;
        let end = self.u32()? as usize;
        if end < start {
            return Err("span end before start".into());
        }
        Ok(Span { start, end })
    }
    fn ty(&mut self, type_table_len: usize) -> Result<hir::TypeUse, String> {
        match self.u8()? {
            0 => match self.s()?.as_str() {
                "i32" => Ok(hir::TypeUse::Builtin(hir::BuiltinType::I32)),
                "bool" => Ok(hir::TypeUse::Builtin(hir::BuiltinType::Bool)),
                // PLAN-862: the unit RESULT word.
                "unit" => Ok(hir::TypeUse::Builtin(hir::BuiltinType::Unit)),
                _ => Err("unknown builtin type".into()),
            },
            1 => Ok(hir::TypeUse::Ref(hir::TypeId(
                self.idx(type_table_len, "type-table")?,
            ))),
            _ => Err("unknown type-use tag".into()),
        }
    }
    fn idx(&mut self, n: usize, what: &str) -> Result<u32, String> {
        let i = self.u32()?;
        if i as usize >= n {
            return Err(format!("{what} index out of bounds"));
        }
        Ok(i)
    }
}

/// Decode one artifact from untrusted bytes (codec v2 only — a v1 payload
/// is a decode error and the row is skipped). Every length, count and
/// index is bounds-checked; any violation is an `Err` (the artifact is
/// skipped, never trusted). Trailing bytes are rejected.
pub fn decode_fn_hir(bytes: &[u8]) -> Result<FnHirArtifact, String> {
    if bytes.len() > FN_HIR_MAX_PAYLOAD {
        return Err("payload over codec bound".into());
    }
    let mut d = FnHirDec { b: bytes, pos: 0 };
    if d.take(4)? != b"FNH1" {
        return Err("bad magic".into());
    }
    if d.u32()? != 2 {
        return Err("unknown codec version".into());
    }
    let def_id = d.s()?;
    let content_version = d.s()?;
    let span_digest = d.u64()?;
    let context_fingerprint = d.s()?;
    let caps_fp = d.s()?;
    let mut intrinsics = Vec::new();
    for _ in 0..d.count()? {
        intrinsics.push(d.s()?);
    }
    let mut callee_table = Vec::new();
    for _ in 0..d.count()? {
        callee_table.push(d.s()?);
    }
    let type_table_len = d.count()?;
    let mut type_table = Vec::with_capacity(type_table_len);
    for _ in 0..type_table_len {
        type_table.push(d.s()?);
    }
    let id_text = d.s()?;
    let owner = d.u32()?;
    let entry = d.u32()?;
    let span = d.span()?;
    let mut loops = Vec::new();
    for _ in 0..d.count()? {
        loops.push(d.s()?);
    }
    let mut locals = Vec::new();
    let n_locals = d.count()?;
    for _ in 0..n_locals {
        let id_text = d.s()?;
        let ty = d.ty(type_table_len)?;
        let name = d.s()?;
        let role = match d.u8()? {
            0 => hir::LocalRole::Param,
            1 => hir::LocalRole::Binding,
            _ => return Err("unknown local role".into()),
        };
        let index = match d.u8()? {
            0 => None,
            1 => Some(d.u32()?),
            _ => return Err("unknown index tag".into()),
        };
        let mutable = d.b()?;
        let mutable_explicit = d.b()?;
        let param_mode = match d.u8()? {
            0 => hir::ParamMode::View,
            1 => hir::ParamMode::MutBorrow,
            2 => hir::ParamMode::MoveOwn,
            _ => return Err("unknown param mode".into()),
        };
        let lspan = d.span()?;
        locals.push(hir::LocalDef {
            id_text,
            ty,
            name,
            role,
            index,
            mutable,
            mutable_explicit,
            param_mode,
            span: lspan,
        });
    }
    let mut places = Vec::new();
    let n_places = d.count()?;
    for _ in 0..n_places {
        let id_text = d.s()?;
        let kind = match d.u8()? {
            0 => hir::PlaceKind::Local,
            _ => return Err("unknown place kind".into()),
        };
        let local = d.idx(n_locals, "local")?;
        let ty = d.ty(type_table_len)?;
        let access = match d.u8()? {
            0 => hir::Access::Mutable,
            1 => hir::Access::ReadOnly,
            _ => return Err("unknown access".into()),
        };
        let pspan = d.span()?;
        places.push(hir::Place {
            id_text,
            kind,
            local: hir::LocalId(local),
            ty,
            access,
            span: pspan,
        });
    }
    let mut exprs = Vec::new();
    let n_exprs = d.count()?;
    for _ in 0..n_exprs {
        let id_text = d.s()?;
        let ty = d.ty(type_table_len)?;
        let kind = match d.u8()? {
            0 => hir::ExprKind::ReadLocal {
                local: hir::LocalId(d.idx(n_locals, "local")?),
            },
            1 => hir::ExprKind::Constant { value: d.i32()? },
            2 => {
                let op = d.s()?;
                let op = bin_op_of(&op)?;
                let overflow = match d.u8()? {
                    0 => hir::OverflowPolicy::Trap,
                    _ => return Err("unknown overflow policy".into()),
                };
                let lhs = d.idx(n_exprs, "expr")?;
                let rhs = d.idx(n_exprs, "expr")?;
                hir::ExprKind::Binary {
                    op,
                    overflow,
                    lhs: hir::ExprId(lhs),
                    rhs: hir::ExprId(rhs),
                }
            }
            3 => {
                let op = unary_op_of(&d.s()?)?;
                hir::ExprKind::Unary {
                    op,
                    operand: hir::ExprId(d.idx(n_exprs, "expr")?),
                }
            }
            4 => {
                let op = match d.s()?.as_str() {
                    "and" => hir::LogicOp::And,
                    "or" => hir::LogicOp::Or,
                    _ => return Err("unknown logic op".into()),
                };
                let lhs = d.idx(n_exprs, "expr")?;
                let rhs = d.idx(n_exprs, "expr")?;
                hir::ExprKind::Logical {
                    op,
                    lhs: hir::ExprId(lhs),
                    rhs: hir::ExprId(rhs),
                }
            }
            5 => {
                let callee = d.idx(callee_table.len(), "callee-table")?;
                let mut eval_args = Vec::new();
                for _ in 0..d.count()? {
                    eval_args.push(hir::ExprId(d.idx(n_exprs, "expr")?));
                }
                let mut bindings = Vec::new();
                for _ in 0..d.count()? {
                    let param = d.u32()?;
                    let arg = d.u32()?;
                    let moved = d.b()?;
                    bindings.push(hir::CallBinding { param, arg, moved });
                }
                hir::ExprKind::Call {
                    callee: hir::DefId(callee),
                    eval_args,
                    bindings,
                }
            }
            // PLAN-862: the unit value (tag 6, no payload).
            6 => hir::ExprKind::Unit,
            7 => {
                let callee = d.idx(n_exprs, "expr")?;
                let mut eval_args = Vec::new();
                for _ in 0..d.count()? {
                    eval_args.push(hir::ExprId(d.idx(n_exprs, "expr")?));
                }
                hir::ExprKind::CallClosure {
                    callee: hir::ExprId(callee),
                    eval_args,
                }
            }
            8 => {
                let callee = d.idx(callee_table.len(), "callee-table")?;
                let mut captures = Vec::new();
                for _ in 0..d.count()? {
                    let local = d.idx(n_locals, "local")?;
                    let mode = match d.u8()? {
                        0 => hir::CaptureMode::Move,
                        1 => hir::CaptureMode::Borrow,
                        _ => return Err("unknown closure capture mode".into()),
                    };
                    captures.push(hir::CaptureBinding {
                        local: hir::LocalId(local),
                        mode,
                    });
                }
                hir::ExprKind::CreateClosure {
                    callee: hir::DefId(callee),
                    captures,
                }
            }
            9 => hir::ExprKind::ClosureClone {
                operand: hir::ExprId(d.idx(n_exprs, "expr")?),
            },
            _ => return Err("unknown expr kind".into()),
        };
        let espan = d.span()?;
        exprs.push(hir::Expr {
            id_text,
            ty,
            kind,
            span: espan,
        });
    }
    let mut blocks = Vec::new();
    let n_blocks = d.count()?;
    for _ in 0..n_blocks {
        let id_text = d.s()?;
        let bspan = d.span()?;
        let mut stmts = Vec::new();
        for _ in 0..d.count()? {
            let st = match d.u8()? {
                0 => hir::Stmt::Let {
                    local: hir::LocalId(d.idx(n_locals, "local")?),
                    value: hir::ExprId(d.idx(n_exprs, "expr")?),
                },
                1 => hir::Stmt::Return {
                    value: hir::ExprId(d.idx(n_exprs, "expr")?),
                },
                2 => {
                    let cond = d.idx(n_exprs, "expr")?;
                    let then = d.idx(n_blocks, "block")?;
                    let els = match d.u8()? {
                        0 => None,
                        1 => Some(hir::BlockId(d.idx(n_blocks, "block")?)),
                        _ => return Err("unknown else tag".into()),
                    };
                    hir::Stmt::If {
                        cond: hir::ExprId(cond),
                        then: hir::BlockId(then),
                        els,
                    }
                }
                3 => {
                    let id = d.idx(loops.len(), "loop")?;
                    let body = d.idx(n_blocks, "block")?;
                    hir::Stmt::Loop {
                        id: hir::LoopId(id),
                        body: hir::BlockId(body),
                    }
                }
                4 => hir::Stmt::Assign {
                    place: hir::PlaceId(d.idx(n_places, "place")?),
                    value: hir::ExprId(d.idx(n_exprs, "expr")?),
                },
                5 => hir::Stmt::Break {
                    target: hir::LoopId(d.idx(loops.len(), "loop")?),
                },
                6 => hir::Stmt::Continue {
                    target: hir::LoopId(d.idx(loops.len(), "loop")?),
                },
                _ => return Err("unknown statement tag".into()),
            };
            stmts.push(st);
        }
        blocks.push(hir::Block {
            id_text,
            stmts,
            span: bspan,
        });
    }
    if d.pos != bytes.len() {
        return Err("trailing bytes".into());
    }
    if entry as usize >= n_blocks {
        return Err("entry block out of bounds".into());
    }
    Ok(FnHirArtifact {
        def_id,
        content_version,
        span_digest,
        context_fingerprint,
        caps_fp,
        intrinsics,
        callee_table,
        type_table,
        body: hir::Body {
            id_text,
            owner: hir::DefId(owner),
            entry: hir::BlockId(entry),
            locals,
            places,
            exprs,
            blocks,
            loops,
            span,
        },
    })
}

fn bin_op_of(s: &str) -> Result<hir::BinOp, String> {
    Ok(match s {
        "add_i32" => hir::BinOp::AddI32,
        "sub_i32" => hir::BinOp::SubI32,
        "mul_i32" => hir::BinOp::MulI32,
        "div_i32" => hir::BinOp::DivI32,
        "rem_i32" => hir::BinOp::RemI32,
        "lt_i32" => hir::BinOp::LtI32,
        "le_i32" => hir::BinOp::LeI32,
        "gt_i32" => hir::BinOp::GtI32,
        "ge_i32" => hir::BinOp::GeI32,
        "eq" => hir::BinOp::Eq,
        "ne" => hir::BinOp::Ne,
        _ => return Err(format!("unknown bin op `{s}`")),
    })
}

fn unary_op_of(s: &str) -> Result<hir::UnaryOp, String> {
    Ok(match s {
        "neg_i32" => hir::UnaryOp::NegI32,
        "not_bool" => hir::UnaryOp::NotBool,
        _ => return Err(format!("unknown unary op `{s}`")),
    })
}

/// Store key of one function-HIR artifact (content-addressed like the ABC
/// unit rows: same body content under the same context shares a row).
pub fn fn_hir_key(owner: &str, unit: &str, body_cv: &str, caps_fp: &str) -> QueryKey {
    QueryKey::new(
        &HIR_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("owner".into(), owner.into()),
            ("unit".into(), unit.into()),
            ("profile".into(), AV_PROFILE.into()),
            ("hir-domain".into(), "scalar-i32-draft".into()),
            ("hir-schema".into(), "1".into()),
            ("hir-codec".into(), "ac828/a07-fnhir/v2".into()),
            ("caps".into(), caps_fp.into()),
            ("body-content".into(), body_cv.into()),
        ],
    )
}
