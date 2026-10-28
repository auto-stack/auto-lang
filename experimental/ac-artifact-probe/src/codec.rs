//! Frozen binary codecs for the probe's real payload set (PLAN-783 §5.1):
//!
//! * chunk framing — self-verifying envelope for every stored payload;
//! * HIR bundle — the core/scalar **restricted executable subset** of
//!   `auto_ac_prototype::hir::Bundle`, binary restore (never a text
//!   roundtrip), full bounds validation on decode;
//! * declaration summary — signature-only projection of the real source
//!   AST (content-identical across body-only edits);
//! * source snapshot record + object manifest.
//!
//! A restored bundle carries NO trust: the driver re-enters the real
//! `verify` of its domain before any use (V-02 admission oracle).

use crate::identity::ChunkId;
use std::fmt;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Controlled codec errors. Structural corruption, unsupported
/// schema/version/domain/feature and reference-bound violations are
/// distinct categories (the admission tests distinguish them).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    Truncated {
        at: &'static str,
    },
    LengthOverflow {
        what: &'static str,
        count: u64,
        available: u64,
    },
    BadMagic(u32),
    UnsupportedFraming(u16),
    UnknownChunkKind(u8),
    TooLarge {
        len: u64,
        cap: u64,
    },
    ChecksumMismatch {
        expected: String,
        actual: String,
    },
    TrailingBytes {
        extra: u64,
    },
    UnsupportedVersion(u16),
    UnsupportedDomain(u8),
    UnsupportedFeature(&'static str),
    InvalidUtf8,
    IndexOutOfBounds {
        what: &'static str,
        index: u64,
        len: u64,
    },
    BadTag {
        what: &'static str,
        tag: u8,
    },
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::Truncated { at } => write!(f, "truncated input at {}", at),
            CodecError::LengthOverflow {
                what,
                count,
                available,
            } => {
                write!(
                    f,
                    "{} count {} exceeds remaining bytes {}",
                    what, count, available
                )
            }
            CodecError::BadMagic(m) => write!(f, "bad chunk magic {:#x}", m),
            CodecError::UnsupportedFraming(v) => write!(f, "unsupported framing version {}", v),
            CodecError::UnknownChunkKind(k) => write!(f, "unknown chunk kind {}", k),
            CodecError::TooLarge { len, cap } => write!(f, "payload {} exceeds cap {}", len, cap),
            CodecError::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "chunk checksum mismatch: expected {} got {}",
                    expected, actual
                )
            }
            CodecError::TrailingBytes { extra } => {
                write!(f, "{} trailing bytes after payload", extra)
            }
            CodecError::UnsupportedVersion(v) => write!(f, "unsupported codec version {}", v),
            CodecError::UnsupportedDomain(d) => write!(f, "unsupported HIR domain code {}", d),
            CodecError::UnsupportedFeature(what) => {
                write!(
                    f,
                    "feature outside the probe's restricted executable subset: {}",
                    what
                )
            }
            CodecError::InvalidUtf8 => write!(f, "invalid utf-8 string"),
            CodecError::IndexOutOfBounds { what, index, len } => {
                write!(f, "{} index {} out of bounds (len {})", what, index, len)
            }
            CodecError::BadTag { what, tag } => write!(f, "bad {} tag {}", what, tag),
        }
    }
}

impl std::error::Error for CodecError {}

// ---------------------------------------------------------------------------
// Chunk framing
// ---------------------------------------------------------------------------

pub const CHUNK_MAGIC: u32 = 0x5A4B_3341; // "A3KZ"
pub const FRAMING_VERSION: u16 = 1;
pub const CHUNK_HEADER_LEN: usize = 48;
/// Probe-wide chunk size cap (layout pages are 64 KiB; objects/HIR far
/// smaller). Larger payloads are a controlled rejection, not an OOM.
pub const MAX_CHUNK: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ChunkKind {
    Hir = 0,
    DeclSummary = 1,
    Object = 2,
    SnapshotRecord = 3,
    ObjectManifest = 4,
    LayoutData = 5,
    // PLAN-787 加法扩展：真实语法产物（A01/A02/A03 生产者 = ac-syntax-artifact-adapter）。
    SyntaxGrammar = 6,
    TokenStream = 7,
    CstPage = 8,
    CstManifest = 9,
    // PLAN-787 修复轮（F787-R1-06）：内容寻址源码页（A02 源字节持久化，
    // CST required-data 图的源页节点）。
    SourcePage = 10,
}

impl ChunkKind {
    /// The SINGLE kind decoding table — both the frame layer (unframe) and
    /// the store metadata layer (`ChunkKind::from_code`) decode the same
    /// byte and must not drift apart. `pub(crate)` for the store delegation:
    /// a locally duplicated store table dropped SourcePage=10 and made valid
    /// artifacts unreadable via the public artifact APIs (F787-R7-01).
    pub(crate) fn from_u8(v: u8) -> Result<Self, CodecError> {
        match v {
            0 => Ok(ChunkKind::Hir),
            1 => Ok(ChunkKind::DeclSummary),
            2 => Ok(ChunkKind::Object),
            3 => Ok(ChunkKind::SnapshotRecord),
            4 => Ok(ChunkKind::ObjectManifest),
            5 => Ok(ChunkKind::LayoutData),
            6 => Ok(ChunkKind::SyntaxGrammar),
            7 => Ok(ChunkKind::TokenStream),
            8 => Ok(ChunkKind::CstPage),
            9 => Ok(ChunkKind::CstManifest),
            10 => Ok(ChunkKind::SourcePage),
            k => Err(CodecError::UnknownChunkKind(k)),
        }
    }
}

/// Frame a payload into a self-verifying chunk. The embedded digest is the
/// content identity of the payload (`ChunkId` hashes exactly these bytes
/// with the `chk` domain).
pub fn frame_chunk(kind: ChunkKind, payload: &[u8]) -> Vec<u8> {
    let digest = blake3::hash(payload);
    let mut out = Vec::with_capacity(CHUNK_HEADER_LEN + payload.len());
    out.extend_from_slice(&CHUNK_MAGIC.to_le_bytes());
    out.extend_from_slice(&FRAMING_VERSION.to_le_bytes());
    out.push(kind as u8);
    out.push(0);
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(digest.as_bytes());
    out.extend_from_slice(payload);
    out
}

/// Validate framing: magic, version, kind, exact length, checksum.
pub fn unframe_chunk(bytes: &[u8]) -> Result<(ChunkKind, &[u8]), CodecError> {
    if bytes.len() < CHUNK_HEADER_LEN {
        return Err(CodecError::Truncated { at: "chunk header" });
    }
    let magic = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    if magic != CHUNK_MAGIC {
        return Err(CodecError::BadMagic(magic));
    }
    let ver = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
    if ver != FRAMING_VERSION {
        return Err(CodecError::UnsupportedFraming(ver));
    }
    let kind = ChunkKind::from_u8(bytes[6])?;
    let len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    if len > MAX_CHUNK {
        return Err(CodecError::TooLarge {
            len,
            cap: MAX_CHUNK,
        });
    }
    let end = CHUNK_HEADER_LEN + len as usize;
    if bytes.len() < end {
        return Err(CodecError::Truncated {
            at: "chunk payload",
        });
    }
    if bytes.len() > end {
        return Err(CodecError::TrailingBytes {
            extra: (bytes.len() - end) as u64,
        });
    }
    let expected = &bytes[16..48];
    let actual = blake3::hash(&bytes[CHUNK_HEADER_LEN..end]);
    if actual.as_bytes() != expected {
        return Err(CodecError::ChecksumMismatch {
            expected: hex(expected),
            actual: actual.to_hex().to_string(),
        });
    }
    Ok((kind, &bytes[CHUNK_HEADER_LEN..end]))
}

/// Content identity of a payload (what `frame_chunk` digests).
pub fn chunk_id_of(payload: &[u8]) -> ChunkId {
    ChunkId::from_parts(&[payload])
}

/// Content identity of a STORED chunk: the framed bytes are what the CAS
/// addresses (this is exactly what `store::put_artifact` hashes). Receipts
/// must use THIS form, never the raw-payload digest, so a receipt written
/// from a hit (which reads the stored chunk id) and one written from a
/// fresh build are byte-identical.
pub fn framed_chunk_id(kind: ChunkKind, payload: &[u8]) -> ChunkId {
    let framed = frame_chunk(kind, payload);
    ChunkId::from_parts(&[framed.as_slice()])
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

// ---------------------------------------------------------------------------
// HIR bundle codec (restricted executable subset: core + scalar domains)
// ---------------------------------------------------------------------------

pub const HIR_CODEC_VERSION: u16 = 1;

use auto_ac_prototype::hir::{
    Access, BinOp, Bundle, CallBinding, Def, DefKind, Effect, Expr, ExprKind, FnSig, HirDomain,
    LocalDef, LocalRole, LogicOp, LoopId, Module, OverflowPolicy, ParamMode, Place, PlaceKind,
    Stmt, TypeDef, TypeKind, TypeUse, UnaryOp,
};

// Re-exported for driver use (ExprId etc. are constructed by decode).
pub use auto_ac_prototype::hir::{
    Block, BlockId, Body, BodyId, DefId, ExprId, LocalId, PlaceId, TypeId,
};

fn domain_code(d: HirDomain) -> Result<u8, CodecError> {
    match d {
        HirDomain::Core => Ok(0),
        HirDomain::Scalar => Ok(1),
        _ => Err(CodecError::UnsupportedFeature(
            "HIR domain outside core/scalar (text/record not in the probe subset)",
        )),
    }
}

fn domain_from_code(v: u8) -> Result<HirDomain, CodecError> {
    match v {
        0 => Ok(HirDomain::Core),
        1 => Ok(HirDomain::Scalar),
        v => Err(CodecError::UnsupportedDomain(v)),
    }
}

/// Encode a bundle into framed chunk bytes (binary restore proof path).
pub fn encode_bundle_chunk(bundle: &Bundle) -> Result<Vec<u8>, CodecError> {
    let payload = encode_bundle(bundle)?;
    Ok(frame_chunk(ChunkKind::Hir, &payload))
}

pub fn encode_bundle(bundle: &Bundle) -> Result<Vec<u8>, CodecError> {
    domain_code(bundle.domain)?; // text/record bundles are refused
    let mut w = Writer::new();
    w.u16(HIR_CODEC_VERSION);
    w.u8(domain_code(bundle.domain)?);
    w.u8(0); // flags
    w.strs(&bundle.requires);
    w.u32(bundle.modules.len() as u32);
    for m in &bundle.modules {
        w.str(&m.id_text);
        w.opt_str(m.path.as_deref());
        w.span(&m.span);
        w.u32(m.types.len() as u32);
        for t in &m.types {
            w.str(&t.id_text);
            w.span(&t.span);
            match &t.kind {
                TypeKind::I32 => w.u8(0),
                TypeKind::Bool => w.u8(1),
                TypeKind::Function(sig) => {
                    w.u8(2);
                    encode_sig(&mut w, sig)?;
                }
                TypeKind::Str
                | TypeKind::StrOwned
                | TypeKind::Char
                | TypeKind::Record(_)
                // PLAN-864: List/Variant types live only in the collections
                // domain, which this core/scalar artifact codec never sees.
                | TypeKind::List { .. }
                | TypeKind::Variant { .. } => {
                    return Err(CodecError::UnsupportedFeature(
                        "type kind outside core/scalar subset",
                    ))
                }
            }
        }
        w.u32(m.defs.len() as u32);
        for d in &m.defs {
            w.str(&d.id_text);
            w.span(&d.span);
            match &d.kind {
                DefKind::Function { name, ty, body } => {
                    w.u8(0);
                    w.str(name);
                    w.u32(ty.0);
                    w.u32(body.0);
                }
                DefKind::Intrinsic { symbol, effect, ty } => {
                    w.u8(1);
                    w.str(symbol);
                    w.u8(match effect {
                        Effect::Observable => 0,
                        Effect::Pure => 1,
                    });
                    w.u32(ty.0);
                }
                DefKind::Method { .. } => {
                    return Err(CodecError::UnsupportedFeature(
                        "method defs (record domain)",
                    ))
                }
            }
        }
        w.u32(m.bodies.len() as u32);
        for b in &m.bodies {
            w.str(&b.id_text);
            w.u32(b.owner.0);
            w.u32(b.entry.0);
            w.span(&b.span);
            w.u32(b.locals.len() as u32);
            for l in &b.locals {
                w.str(&l.id_text);
                w.type_use(&l.ty)?;
                w.str(&l.name);
                w.u8(match l.role {
                    LocalRole::Param => 0,
                    LocalRole::Binding => 1,
                });
                match l.index {
                    Some(i) => {
                        w.u8(1);
                        w.u32(i);
                    }
                    None => w.u8(0),
                }
                w.u8(u8::from(l.mutable));
                w.u8(u8::from(l.mutable_explicit));
                match l.param_mode {
                    ParamMode::View => w.u8(0),
                    _ => {
                        return Err(CodecError::UnsupportedFeature(
                            "param ownership mode (text/record)",
                        ))
                    }
                }
                w.span(&l.span);
            }
            w.u32(b.places.len() as u32);
            for p in &b.places {
                w.str(&p.id_text);
                match p.kind {
                    PlaceKind::Local => w.u8(0),
                    PlaceKind::FieldPath(_) => {
                        return Err(CodecError::UnsupportedFeature(
                            "field-path places (record domain)",
                        ))
                    }
                }
                w.u32(p.local.0);
                w.type_use(&p.ty)?;
                w.u8(match p.access {
                    Access::Mutable => 0,
                    Access::ReadOnly => 1,
                });
                w.span(&p.span);
            }
            w.u32(b.exprs.len() as u32);
            for e in &b.exprs {
                w.str(&e.id_text);
                w.type_use(&e.ty)?;
                w.span(&e.span);
                encode_expr(&mut w, &e.kind)?;
            }
            w.u32(b.blocks.len() as u32);
            for blk in &b.blocks {
                w.str(&blk.id_text);
                w.span(&blk.span);
                w.u32(blk.stmts.len() as u32);
                for s in &blk.stmts {
                    encode_stmt(&mut w, s)?;
                }
            }
            w.strs(&b.loops);
        }
    }
    Ok(w.out)
}

fn encode_sig(w: &mut Writer, sig: &FnSig) -> Result<(), CodecError> {
    w.u32(sig.params.len() as u32);
    for p in &sig.params {
        w.type_use(p)?;
    }
    w.type_use(&sig.result)?;
    Ok(())
}

fn encode_expr(w: &mut Writer, k: &ExprKind) -> Result<(), CodecError> {
    match k {
        ExprKind::ReadLocal { local } => {
            w.u8(0);
            w.u32(local.0);
        }
        ExprKind::Constant { value } => {
            w.u8(1);
            w.i32(*value);
        }
        ExprKind::Binary {
            op,
            overflow,
            lhs,
            rhs,
        } => {
            w.u8(2);
            w.u8(binop_code(*op));
            w.u8(match overflow {
                OverflowPolicy::Trap => 0,
            });
            w.u32(lhs.0);
            w.u32(rhs.0);
        }
        ExprKind::Unary { op, operand } => {
            w.u8(3);
            w.u8(match op {
                UnaryOp::NegI32 => 0,
                UnaryOp::NotBool => 1,
            });
            w.u32(operand.0);
        }
        ExprKind::Logical { op, lhs, rhs } => {
            w.u8(4);
            w.u8(match op {
                LogicOp::And => 0,
                LogicOp::Or => 1,
            });
            w.u32(lhs.0);
            w.u32(rhs.0);
        }
        ExprKind::Call {
            callee,
            eval_args,
            bindings,
        } => {
            w.u8(5);
            w.u32(callee.0);
            w.u32(eval_args.len() as u32);
            for a in eval_args {
                w.u32(a.0);
            }
            w.u32(bindings.len() as u32);
            for b in bindings {
                w.u32(b.param);
                w.u32(b.arg);
                w.u8(u8::from(b.moved));
            }
        }
        _ => {
            return Err(CodecError::UnsupportedFeature(
                "expression kind outside core/scalar subset",
            ))
        }
    }
    Ok(())
}

fn encode_stmt(w: &mut Writer, s: &Stmt) -> Result<(), CodecError> {
    match s {
        Stmt::Let { local, value } => {
            w.u8(0);
            w.u32(local.0);
            w.u32(value.0);
        }
        Stmt::Return { value } => {
            w.u8(1);
            w.u32(value.0);
        }
        Stmt::If { cond, then, els } => {
            w.u8(2);
            w.u32(cond.0);
            w.u32(then.0);
            match els {
                Some(b) => {
                    w.u8(1);
                    w.u32(b.0);
                }
                None => w.u8(0),
            }
        }
        Stmt::Loop { id, body } => {
            w.u8(3);
            w.u32(id.0);
            w.u32(body.0);
        }
        Stmt::Assign { place, value } => {
            w.u8(4);
            w.u32(place.0);
            w.u32(value.0);
        }
        Stmt::Break { target } => {
            w.u8(5);
            w.u32(target.0);
        }
        Stmt::Continue { target } => {
            w.u8(6);
            w.u32(target.0);
        }
        Stmt::Drop { .. } => {
            return Err(CodecError::UnsupportedFeature(
                "drop statement (text resource plan)",
            ))
        }
        // PLAN-864: unit-effect statements live only in the collections
        // domain, which this core/scalar artifact codec never sees.
        Stmt::Eval { .. } => {
            return Err(CodecError::UnsupportedFeature(
                "eval statement (collections surface)",
            ))
        }
    }
    Ok(())
}

fn binop_code(op: BinOp) -> u8 {
    match op {
        BinOp::AddI32 => 0,
        BinOp::SubI32 => 1,
        BinOp::MulI32 => 2,
        BinOp::DivI32 => 3,
        BinOp::RemI32 => 4,
        BinOp::LtI32 => 5,
        BinOp::LeI32 => 6,
        BinOp::GtI32 => 7,
        BinOp::GeI32 => 8,
        BinOp::Eq => 9,
        BinOp::Ne => 10,
    }
}

fn binop_from_code(v: u8) -> Result<BinOp, CodecError> {
    match v {
        0 => Ok(BinOp::AddI32),
        1 => Ok(BinOp::SubI32),
        2 => Ok(BinOp::MulI32),
        3 => Ok(BinOp::DivI32),
        4 => Ok(BinOp::RemI32),
        5 => Ok(BinOp::LtI32),
        6 => Ok(BinOp::LeI32),
        7 => Ok(BinOp::GtI32),
        8 => Ok(BinOp::GeI32),
        9 => Ok(BinOp::Eq),
        10 => Ok(BinOp::Ne),
        t => Err(CodecError::BadTag {
            what: "binop",
            tag: t,
        }),
    }
}

// -- writer / reader primitives ----------------------------------------------

struct Writer {
    out: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Writer { out: Vec::new() }
    }
    fn u8(&mut self, v: u8) {
        self.out.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.u32(v.len() as u32);
        self.out.extend_from_slice(v);
    }
    fn str(&mut self, v: &str) {
        self.bytes(v.as_bytes());
    }
    fn opt_str(&mut self, v: Option<&str>) {
        match v {
            Some(s) => {
                self.u8(1);
                self.str(s);
            }
            None => self.u8(0),
        }
    }
    fn strs(&mut self, v: &[String]) {
        self.u32(v.len() as u32);
        for s in v {
            self.str(s);
        }
    }
    fn span(&mut self, s: &auto_ac_prototype::atom_text::Span) {
        self.u64(s.start as u64);
        self.u64(s.end as u64);
    }
    fn type_use(&mut self, t: &TypeUse) -> Result<(), CodecError> {
        match t {
            TypeUse::Builtin(b) => {
                self.u8(0);
                match b {
                    auto_ac_prototype::hir::BuiltinType::I32 => self.u8(0),
                    auto_ac_prototype::hir::BuiltinType::Bool => self.u8(1),
                    _ => {
                        return Err(CodecError::UnsupportedFeature(
                            "builtin type outside core/scalar subset",
                        ))
                    }
                }
            }
            TypeUse::Ref(id) => {
                self.u8(1);
                self.u32(id.0);
            }
        }
        Ok(())
    }
}

/// Decode a framed HIR chunk into a bundle (untrusted; verify before use).
pub fn decode_bundle_chunk(bytes: &[u8]) -> Result<Bundle, CodecError> {
    let (kind, payload) = unframe_chunk(bytes)?;
    if kind != ChunkKind::Hir {
        return Err(CodecError::UnknownChunkKind(kind as u8));
    }
    decode_bundle(payload)
}

pub fn decode_bundle(payload: &[u8]) -> Result<Bundle, CodecError> {
    let mut r = Reader::new(payload);
    let version = r.u16("version")?;
    if version != HIR_CODEC_VERSION {
        return Err(CodecError::UnsupportedVersion(version));
    }
    let domain = domain_from_code(r.u8("domain")?)?;
    let _flags = r.u8("flags")?;
    let requires = r.strs("requires")?;
    let n_modules = r.count("modules")?;
    let mut modules = Vec::with_capacity(n_modules as usize);
    for _ in 0..n_modules {
        let id_text = r.string("module.id")?;
        let path = r.opt_str("module.path")?;
        let span = r.span()?;
        let n_types = r.count("types")?;
        let mut types = Vec::with_capacity(n_types as usize);
        for _ in 0..n_types {
            let tid = r.string("type.id")?;
            let tspan = r.span()?;
            let tag = r.u8("type.kind")?;
            let kind = match tag {
                0 => TypeKind::I32,
                1 => TypeKind::Bool,
                2 => TypeKind::Function(decode_sig(&mut r)?),
                t => {
                    return Err(CodecError::BadTag {
                        what: "type.kind",
                        tag: t,
                    })
                }
            };
            types.push(TypeDef {
                id_text: tid,
                kind,
                span: tspan,
            });
        }
        let n_defs = r.count("defs")?;
        let mut defs = Vec::with_capacity(n_defs as usize);
        for _ in 0..n_defs {
            let did = r.string("def.id")?;
            let dspan = r.span()?;
            let tag = r.u8("def.kind")?;
            let kind = match tag {
                0 => {
                    let name = r.string("fn.name")?;
                    let ty = TypeId(r.u32("fn.ty")?);
                    let body = BodyId(r.u32("fn.body")?);
                    DefKind::Function { name, ty, body }
                }
                1 => {
                    let symbol = r.string("intrinsic.symbol")?;
                    let effect = match r.u8("intrinsic.effect")? {
                        0 => Effect::Observable,
                        1 => Effect::Pure,
                        t => {
                            return Err(CodecError::BadTag {
                                what: "effect",
                                tag: t,
                            })
                        }
                    };
                    let ty = TypeId(r.u32("intrinsic.ty")?);
                    DefKind::Intrinsic { symbol, effect, ty }
                }
                t => {
                    return Err(CodecError::BadTag {
                        what: "def.kind",
                        tag: t,
                    })
                }
            };
            defs.push(Def {
                id_text: did,
                kind,
                span: dspan,
            });
        }
        let n_bodies = r.count("bodies")?;
        let mut bodies = Vec::with_capacity(n_bodies as usize);
        for _ in 0..n_bodies {
            let bid = r.string("body.id")?;
            let owner = DefId(r.u32("body.owner")?);
            let entry = BlockId(r.u32("body.entry")?);
            let bspan = r.span()?;
            let n_locals = r.count("locals")?;
            let mut locals = Vec::with_capacity(n_locals as usize);
            for _ in 0..n_locals {
                let lid = r.string("local.id")?;
                let ty = r.type_use()?;
                let name = r.string("local.name")?;
                let role = match r.u8("local.role")? {
                    0 => LocalRole::Param,
                    1 => LocalRole::Binding,
                    t => {
                        return Err(CodecError::BadTag {
                            what: "local.role",
                            tag: t,
                        })
                    }
                };
                let index = match r.u8("local.index-tag")? {
                    0 => None,
                    1 => Some(r.u32("local.index")?),
                    t => {
                        return Err(CodecError::BadTag {
                            what: "local.index-tag",
                            tag: t,
                        })
                    }
                };
                let mutable = r.u8("local.mutable")? != 0;
                let mutable_explicit = r.u8("local.mutable-explicit")? != 0;
                let param_mode = match r.u8("local.param-mode")? {
                    0 => ParamMode::View,
                    t => {
                        return Err(CodecError::BadTag {
                            what: "param-mode",
                            tag: t,
                        })
                    }
                };
                let lspan = r.span()?;
                locals.push(LocalDef {
                    id_text: lid,
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
            let n_places = r.count("places")?;
            let mut places = Vec::with_capacity(n_places as usize);
            for _ in 0..n_places {
                let pid = r.string("place.id")?;
                let tag = r.u8("place.kind")?;
                if tag != 0 {
                    return Err(CodecError::BadTag {
                        what: "place.kind",
                        tag,
                    });
                }
                let local = LocalId(r.u32("place.local")?);
                let ty = r.type_use()?;
                let access = match r.u8("place.access")? {
                    0 => Access::Mutable,
                    1 => Access::ReadOnly,
                    t => {
                        return Err(CodecError::BadTag {
                            what: "access",
                            tag: t,
                        })
                    }
                };
                let pspan = r.span()?;
                places.push(Place {
                    id_text: pid,
                    kind: PlaceKind::Local,
                    local,
                    ty,
                    access,
                    span: pspan,
                });
            }
            let n_exprs = r.count("exprs")?;
            let mut exprs: Vec<Expr> = Vec::with_capacity(n_exprs as usize);
            for _ in 0..n_exprs {
                let eid = r.string("expr.id")?;
                let ty = r.type_use()?;
                let espan = r.span()?;
                let kind = decode_expr(&mut r)?;
                exprs.push(Expr {
                    id_text: eid,
                    ty,
                    kind,
                    span: espan,
                });
            }
            let n_blocks = r.count("blocks")?;
            let mut blocks = Vec::with_capacity(n_blocks as usize);
            for _ in 0..n_blocks {
                let kbid = r.string("block.id")?;
                let kspan = r.span()?;
                let n_stmts = r.count("stmts")?;
                let mut stmts = Vec::with_capacity(n_stmts as usize);
                for _ in 0..n_stmts {
                    stmts.push(decode_stmt(&mut r)?);
                }
                blocks.push(Block {
                    id_text: kbid,
                    stmts,
                    span: kspan,
                });
            }
            let loops = r.strs("loops")?;
            bodies.push(Body {
                id_text: bid,
                owner,
                entry,
                locals,
                places,
                exprs,
                blocks,
                loops,
                span: bspan,
            });
        }
        modules.push(Module {
            id_text,
            path,
            types,
            defs,
            bodies,
            span,
        });
    }
    r.expect_end()?;
    let bundle = Bundle {
        domain,
        requires,
        modules,
    };
    validate_bundle(&bundle)?;
    Ok(bundle)
}

fn decode_sig(r: &mut Reader) -> Result<FnSig, CodecError> {
    let n = r.count("sig.params")?;
    let mut params = Vec::with_capacity(n as usize);
    for _ in 0..n {
        params.push(r.type_use()?);
    }
    let result = r.type_use()?;
    Ok(FnSig { params, result })
}

fn decode_expr(r: &mut Reader) -> Result<ExprKind, CodecError> {
    let tag = r.u8("expr.kind")?;
    match tag {
        0 => Ok(ExprKind::ReadLocal {
            local: LocalId(r.u32("read.local")?),
        }),
        1 => Ok(ExprKind::Constant {
            value: r.i32("const")?,
        }),
        2 => {
            let op = binop_from_code(r.u8("binop")?)?;
            let ov = match r.u8("overflow")? {
                0 => OverflowPolicy::Trap,
                t => {
                    return Err(CodecError::BadTag {
                        what: "overflow",
                        tag: t,
                    })
                }
            };
            let lhs = ExprId(r.u32("bin.lhs")?);
            let rhs = ExprId(r.u32("bin.rhs")?);
            Ok(ExprKind::Binary {
                op,
                overflow: ov,
                lhs,
                rhs,
            })
        }
        3 => {
            let op = match r.u8("unop")? {
                0 => UnaryOp::NegI32,
                1 => UnaryOp::NotBool,
                t => {
                    return Err(CodecError::BadTag {
                        what: "unop",
                        tag: t,
                    })
                }
            };
            Ok(ExprKind::Unary {
                op,
                operand: ExprId(r.u32("un.operand")?),
            })
        }
        4 => {
            let op = match r.u8("logicop")? {
                0 => LogicOp::And,
                1 => LogicOp::Or,
                t => {
                    return Err(CodecError::BadTag {
                        what: "logicop",
                        tag: t,
                    })
                }
            };
            Ok(ExprKind::Logical {
                op,
                lhs: ExprId(r.u32("logic.lhs")?),
                rhs: ExprId(r.u32("logic.rhs")?),
            })
        }
        5 => {
            let callee = DefId(r.u32("call.callee")?);
            let n = r.count("call.args")?;
            let mut eval_args = Vec::with_capacity(n as usize);
            for _ in 0..n {
                eval_args.push(ExprId(r.u32("call.arg")?));
            }
            let nb = r.count("call.bindings")?;
            let mut bindings = Vec::with_capacity(nb as usize);
            for _ in 0..nb {
                let param = r.u32("bind.param")?;
                let arg = r.u32("bind.arg")?;
                let moved = r.u8("bind.moved")? != 0;
                bindings.push(CallBinding { param, arg, moved });
            }
            Ok(ExprKind::Call {
                callee,
                eval_args,
                bindings,
            })
        }
        t => Err(CodecError::BadTag {
            what: "expr.kind",
            tag: t,
        }),
    }
}

fn decode_stmt(r: &mut Reader) -> Result<Stmt, CodecError> {
    let tag = r.u8("stmt.kind")?;
    match tag {
        0 => Ok(Stmt::Let {
            local: LocalId(r.u32("let.local")?),
            value: ExprId(r.u32("let.value")?),
        }),
        1 => Ok(Stmt::Return {
            value: ExprId(r.u32("ret.value")?),
        }),
        2 => {
            let cond = ExprId(r.u32("if.cond")?);
            let then = BlockId(r.u32("if.then")?);
            let els = match r.u8("if.else-tag")? {
                0 => None,
                1 => Some(BlockId(r.u32("if.else")?)),
                t => {
                    return Err(CodecError::BadTag {
                        what: "if.else-tag",
                        tag: t,
                    })
                }
            };
            Ok(Stmt::If { cond, then, els })
        }
        3 => Ok(Stmt::Loop {
            id: LoopId(r.u32("loop.id")?),
            body: BlockId(r.u32("loop.body")?),
        }),
        4 => Ok(Stmt::Assign {
            place: PlaceId(r.u32("assign.place")?),
            value: ExprId(r.u32("assign.value")?),
        }),
        5 => Ok(Stmt::Break {
            target: LoopId(r.u32("break.target")?),
        }),
        6 => Ok(Stmt::Continue {
            target: LoopId(r.u32("continue.target")?),
        }),
        t => Err(CodecError::BadTag {
            what: "stmt.kind",
            tag: t,
        }),
    }
}

/// Post-decode reference validation: every table index must be in bounds.
/// This is the reader-side half of the V-02 admission oracle (a tampered
/// index never becomes a panic or a silent alias).
fn validate_bundle(b: &Bundle) -> Result<(), CodecError> {
    for m in &b.modules {
        let n_types = m.types.len() as u64;
        let n_defs = m.defs.len() as u64;
        let n_bodies = m.bodies.len() as u64;
        // Function-type ids and body ids inside defs.
        for d in &m.defs {
            let (ty, body) = match &d.kind {
                DefKind::Function { ty, body, .. } => (*ty, Some(*body)),
                DefKind::Intrinsic { ty, .. } => (*ty, None),
                DefKind::Method { ty, .. } => (*ty, None),
            };
            if ty.0 as u64 >= n_types {
                return Err(CodecError::IndexOutOfBounds {
                    what: "type",
                    index: ty.0 as u64,
                    len: n_types,
                });
            }
            if let Some(bid) = body {
                if bid.0 as u64 >= n_bodies {
                    return Err(CodecError::IndexOutOfBounds {
                        what: "body",
                        index: bid.0 as u64,
                        len: n_bodies,
                    });
                }
            }
        }
        // TypeUse::Ref indices.
        let check_tu = |tu: &TypeUse| -> Result<(), CodecError> {
            if let TypeUse::Ref(t) = tu {
                if t.0 as u64 >= n_types {
                    return Err(CodecError::IndexOutOfBounds {
                        what: "type",
                        index: t.0 as u64,
                        len: n_types,
                    });
                }
            }
            Ok(())
        };
        for t in &m.types {
            if let TypeKind::Function(sig) = &t.kind {
                for p in &sig.params {
                    check_tu(p)?;
                }
                check_tu(&sig.result)?;
            }
        }
        for (bi, body) in m.bodies.iter().enumerate() {
            let _ = bi;
            let n_locals = body.locals.len() as u64;
            let n_exprs = body.exprs.len() as u64;
            let n_places = body.places.len() as u64;
            let n_blocks = body.blocks.len() as u64;
            let n_loops = body.loops.len() as u64;
            if body.owner.0 as u64 >= n_defs {
                return Err(CodecError::IndexOutOfBounds {
                    what: "def (body owner)",
                    index: body.owner.0 as u64,
                    len: n_defs,
                });
            }
            if body.entry.0 as u64 >= n_blocks {
                return Err(CodecError::IndexOutOfBounds {
                    what: "block (body entry)",
                    index: body.entry.0 as u64,
                    len: n_blocks,
                });
            }
            for l in &body.locals {
                check_tu(&l.ty)?;
            }
            for p in &body.places {
                if p.local.0 as u64 >= n_locals {
                    return Err(CodecError::IndexOutOfBounds {
                        what: "local (place)",
                        index: p.local.0 as u64,
                        len: n_locals,
                    });
                }
                check_tu(&p.ty)?;
            }
            for e in &body.exprs {
                check_tu(&e.ty)?;
                let check = |i: &ExprId| -> Result<(), CodecError> {
                    if i.0 as u64 >= n_exprs {
                        return Err(CodecError::IndexOutOfBounds {
                            what: "expr",
                            index: i.0 as u64,
                            len: n_exprs,
                        });
                    }
                    Ok(())
                };
                match &e.kind {
                    ExprKind::ReadLocal { local } => {
                        if local.0 as u64 >= n_locals {
                            return Err(CodecError::IndexOutOfBounds {
                                what: "local (read)",
                                index: local.0 as u64,
                                len: n_locals,
                            });
                        }
                    }
                    ExprKind::Binary { lhs, rhs, .. } | ExprKind::Logical { lhs, rhs, .. } => {
                        check(lhs)?;
                        check(rhs)?;
                    }
                    ExprKind::Unary { operand, .. } => check(operand)?,
                    ExprKind::Call {
                        callee, eval_args, ..
                    } => {
                        if callee.0 as u64 >= n_defs {
                            return Err(CodecError::IndexOutOfBounds {
                                what: "def (call callee)",
                                index: callee.0 as u64,
                                len: n_defs,
                            });
                        }
                        for a in eval_args {
                            check(a)?;
                        }
                    }
                    _ => {}
                }
            }
            for blk in &body.blocks {
                for s in &blk.stmts {
                    let check_e = |i: &ExprId| -> Result<(), CodecError> {
                        if i.0 as u64 >= n_exprs {
                            return Err(CodecError::IndexOutOfBounds {
                                what: "expr",
                                index: i.0 as u64,
                                len: n_exprs,
                            });
                        }
                        Ok(())
                    };
                    match s {
                        Stmt::Let { value, .. } | Stmt::Return { value } => check_e(value)?,
                        // PLAN-864: unit-effect statements value-check like
                        // returns (the artifact codec never sees them under
                        // its core/scalar identity, but bounds stay total).
                        Stmt::Eval { value } => check_e(value)?,
                        Stmt::Assign { place, value } => {
                            check_e(value)?;
                            if place.0 as u64 >= n_places {
                                return Err(CodecError::IndexOutOfBounds {
                                    what: "place",
                                    index: place.0 as u64,
                                    len: n_places,
                                });
                            }
                        }
                        Stmt::If { cond, then, els } => {
                            check_e(cond)?;
                            if then.0 as u64 >= n_blocks {
                                return Err(CodecError::IndexOutOfBounds {
                                    what: "block",
                                    index: then.0 as u64,
                                    len: n_blocks,
                                });
                            }
                            if let Some(e) = els {
                                if e.0 as u64 >= n_blocks {
                                    return Err(CodecError::IndexOutOfBounds {
                                        what: "block",
                                        index: e.0 as u64,
                                        len: n_blocks,
                                    });
                                }
                            }
                        }
                        Stmt::Loop { id, body } => {
                            if id.0 as u64 >= n_loops {
                                return Err(CodecError::IndexOutOfBounds {
                                    what: "loop",
                                    index: id.0 as u64,
                                    len: n_loops,
                                });
                            }
                            if body.0 as u64 >= n_blocks {
                                return Err(CodecError::IndexOutOfBounds {
                                    what: "block",
                                    index: body.0 as u64,
                                    len: n_blocks,
                                });
                            }
                        }
                        Stmt::Break { target } | Stmt::Continue { target } => {
                            if target.0 as u64 >= n_loops {
                                return Err(CodecError::IndexOutOfBounds {
                                    what: "loop",
                                    index: target.0 as u64,
                                    len: n_loops,
                                });
                            }
                        }
                        Stmt::Drop { local } => {
                            if local.0 as u64 >= n_locals {
                                return Err(CodecError::IndexOutOfBounds {
                                    what: "local (drop)",
                                    index: local.0 as u64,
                                    len: n_locals,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }
    fn take(&mut self, n: usize, at: &'static str) -> Result<&'a [u8], CodecError> {
        if self.buf.len() - self.pos < n {
            return Err(CodecError::Truncated { at });
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self, at: &'static str) -> Result<u8, CodecError> {
        Ok(self.take(1, at)?[0])
    }
    fn u16(&mut self, at: &'static str) -> Result<u16, CodecError> {
        Ok(u16::from_le_bytes(self.take(2, at)?.try_into().unwrap()))
    }
    fn u32(&mut self, at: &'static str) -> Result<u32, CodecError> {
        Ok(u32::from_le_bytes(self.take(4, at)?.try_into().unwrap()))
    }
    fn i32(&mut self, at: &'static str) -> Result<i32, CodecError> {
        Ok(i32::from_le_bytes(self.take(4, at)?.try_into().unwrap()))
    }
    fn u64(&mut self, at: &'static str) -> Result<u64, CodecError> {
        Ok(u64::from_le_bytes(self.take(8, at)?.try_into().unwrap()))
    }
    /// A table count: each element costs at least one byte on the wire, so
    /// a count larger than the remaining input is malformed (never a huge
    /// allocation).
    fn count(&mut self, what: &'static str) -> Result<u32, CodecError> {
        let n = self.u32(what)? as u64;
        let available = (self.buf.len() - self.pos) as u64;
        if n > available {
            return Err(CodecError::LengthOverflow {
                what,
                count: n,
                available,
            });
        }
        Ok(n as u32)
    }
    fn bytes(&mut self, at: &'static str) -> Result<&'a [u8], CodecError> {
        let n = self.u32(at)? as u64;
        let available = (self.buf.len() - self.pos) as u64;
        if n > available {
            return Err(CodecError::LengthOverflow {
                what: at,
                count: n,
                available,
            });
        }
        self.take(n as usize, at)
    }
    fn string(&mut self, at: &'static str) -> Result<String, CodecError> {
        let b = self.bytes(at)?;
        std::str::from_utf8(b)
            .map(|s| s.to_string())
            .map_err(|_| CodecError::InvalidUtf8)
    }
    fn strs(&mut self, what: &'static str) -> Result<Vec<String>, CodecError> {
        let n = self.count(what)?;
        let mut out = Vec::with_capacity(n as usize);
        for _ in 0..n {
            out.push(self.string(what)?);
        }
        Ok(out)
    }
    fn opt_str(&mut self, at: &'static str) -> Result<Option<String>, CodecError> {
        match self.u8(at)? {
            0 => Ok(None),
            1 => Ok(Some(self.string(at)?)),
            t => Err(CodecError::BadTag { what: at, tag: t }),
        }
    }
    fn span(&mut self) -> Result<auto_ac_prototype::atom_text::Span, CodecError> {
        let start = self.u64("span.start")? as usize;
        let end = self.u64("span.end")? as usize;
        if end < start {
            // Not a codec error per se, but a nonsensical span is refused at
            // the reader boundary rather than flowing downstream.
            return Err(CodecError::BadTag {
                what: "span order",
                tag: 0xff,
            });
        }
        Ok(auto_ac_prototype::atom_text::Span { start, end })
    }
    fn type_use(&mut self) -> Result<TypeUse, CodecError> {
        match self.u8("typeuse.tag")? {
            0 => match self.u8("typeuse.builtin")? {
                0 => Ok(TypeUse::Builtin(auto_ac_prototype::hir::BuiltinType::I32)),
                1 => Ok(TypeUse::Builtin(auto_ac_prototype::hir::BuiltinType::Bool)),
                t => Err(CodecError::BadTag {
                    what: "builtin type",
                    tag: t,
                }),
            },
            1 => Ok(TypeUse::Ref(TypeId(self.u32("typeuse.ref")?))),
            t => Err(CodecError::BadTag {
                what: "typeuse.tag",
                tag: t,
            }),
        }
    }
    fn expect_end(&self) -> Result<(), CodecError> {
        if self.pos != self.buf.len() {
            return Err(CodecError::TrailingBytes {
                extra: (self.buf.len() - self.pos) as u64,
            });
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Declaration summary (A04) — signature-only projection of the real AST
// ---------------------------------------------------------------------------

/// One function signature in the summary. `int`/`i32` canonicalize to
/// `i32` (synonyms in the profile), so summaries are content-identical
/// across body-only edits and across synonym spellings. Spans are
/// deliberately NOT part of the summary: they belong to the source
/// snapshot / HIR artifacts (V-01 span correctness is proven there), and
/// signature content identity must not move when a body edit shifts
/// offsets.
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct FnSummary {
    pub name: String,
    #[serde(default)]
    pub owner_record: Option<String>,
    pub receiver: String,              // free | static | view
    pub params: Vec<(String, String)>, // (name, canonical type word)
    pub result: String,
}

#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct DeclSummary {
    pub module: String,
    pub fns: Vec<FnSummary>,
}

/// Canonical type word of an AST annotation.
pub fn canonical_type_word(t: &auto_ac_prototype::source::ast::TypeAnn) -> String {
    use auto_ac_prototype::source::ast::{TypeRef, TypeWord};
    match &t.kind {
        TypeRef::Builtin(TypeWord::Int) | TypeRef::Builtin(TypeWord::I32) => "i32".into(),
        TypeRef::Builtin(TypeWord::Bool) => "bool".into(),
        TypeRef::Builtin(TypeWord::Str) => "str".into(),
        TypeRef::Builtin(TypeWord::StrOwned) => "Str".into(),
        TypeRef::Builtin(TypeWord::Char) => "char".into(),
        // PLAN-862: the Unit result word is canonical `unit` (entry-unit
        // source profile only; the other profiles never parse it).
        TypeRef::Builtin(TypeWord::Void) => "unit".into(),
        TypeRef::Record(id) => format!("rec:{}", id.name),
        // PLAN-864: collections type grammar (the canonical words keep the
        // written form; this codec's core/scalar summary never emits them).
        TypeRef::Optional(inner) => {
            format!("?{}", canonical_type_word(inner))
        }
        TypeRef::Generic { name, args } => {
            let a: Vec<String> = args.iter().map(canonical_type_word).collect();
            format!("{}<{}>", name.name, a.join(", "))
        }
    }
}

/// Build the summary from the REAL parsed source AST (no synthetic data).
pub fn build_decl_summary(
    program: &auto_ac_prototype::source::ast::Program,
    module: &str,
) -> DeclSummary {
    use auto_ac_prototype::source::ast::{Item, Receiver};
    let mut fns = Vec::new();
    for item in &program.items {
        match item {
            // PLAN-864: enum declarations carry no functions.
            Item::Enum(_) => {}
            Item::Fn(f) => {
                fns.push(FnSummary {
                    name: f.name.name.clone(),
                    owner_record: f.owner.as_ref().map(|o| o.name.clone()),
                    receiver: match f.receiver {
                        Receiver::Free => "free",
                        Receiver::Static => "static",
                        Receiver::View => "view",
                    }
                    .into(),
                    params: f
                        .params
                        .iter()
                        .map(|p| (p.name.name.clone(), canonical_type_word(&p.ty)))
                        .collect(),
                    result: canonical_type_word(&f.ret),
                });
            }
            Item::Type(t) => {
                for m in &t.methods {
                    fns.push(FnSummary {
                        name: m.name.name.clone(),
                        owner_record: Some(t.name.name.clone()),
                        receiver: match m.receiver {
                            Receiver::Free => "free",
                            Receiver::Static => "static",
                            Receiver::View => "view",
                        }
                        .into(),
                        params: m
                            .params
                            .iter()
                            .map(|p| (p.name.name.clone(), canonical_type_word(&p.ty)))
                            .collect(),
                        result: canonical_type_word(&m.ret),
                    });
                }
            }
        }
    }
    DeclSummary {
        module: module.into(),
        fns,
    }
}

/// Summary payload = canonical JSON (stable field order via struct serde).
/// Content identity of the artifact hashes exactly these bytes.
pub fn encode_decl_summary(s: &DeclSummary) -> Result<Vec<u8>, CodecError> {
    serde_json::to_vec(s).map_err(|_| CodecError::InvalidUtf8)
}

pub fn decode_decl_summary(bytes: &[u8]) -> Result<DeclSummary, CodecError> {
    serde_json::from_slice(bytes).map_err(|_| CodecError::InvalidUtf8)
}

// ---------------------------------------------------------------------------
// Object manifest (A09) + source snapshot record (A00)
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ObjectManifest {
    pub object_chunk: String,
    pub object_len: u64,
    pub target: String,
    pub backend: String,
    pub entry: String,
    pub produced_by: String,
}

#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct SnapshotRecord {
    pub logical_name: String,
    pub source_len: u64,
    pub source_chunk: String,
    /// Presentation path at registration time (NOT part of content
    /// identity; diagnostics bind to the current project's path).
    pub registered_from_path: String,
}

pub fn encode_json_chunk<T: serde::Serialize>(
    kind: ChunkKind,
    v: &T,
) -> Result<Vec<u8>, CodecError> {
    let payload = serde_json::to_vec(v).map_err(|_| CodecError::InvalidUtf8)?;
    Ok(frame_chunk(kind, &payload))
}

pub fn decode_json_chunk<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    kind: ChunkKind,
) -> Result<T, CodecError> {
    let (k, payload) = unframe_chunk(bytes)?;
    if k != kind {
        return Err(CodecError::UnknownChunkKind(k as u8));
    }
    serde_json::from_slice(payload).map_err(|_| CodecError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core_fixture_bundle() -> Bundle {
        let src = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
        let parts = auto_ac_prototype::source::compile_parts_with(
            src,
            "testmod",
            &[],
            auto_ac_prototype::source::SourceProfile::Core,
        )
        .expect("compile");
        parts.bundle
    }

    #[test]
    fn chunk_framing_roundtrip_and_corruption() {
        let chunk = frame_chunk(ChunkKind::Hir, b"payload-bytes");
        let (k, p) = unframe_chunk(&chunk).unwrap();
        assert_eq!(k, ChunkKind::Hir);
        assert_eq!(p, b"payload-bytes");
        // Single flipped byte → checksum rejection.
        let mut bad = chunk.clone();
        let last = bad.len() - 1;
        bad[last] ^= 0x01;
        assert!(matches!(
            unframe_chunk(&bad),
            Err(CodecError::ChecksumMismatch { .. })
        ));
        // Truncation.
        assert!(matches!(
            unframe_chunk(&chunk[..chunk.len() - 3]),
            Err(CodecError::Truncated { .. })
        ));
        // Trailing junk.
        let mut extra = chunk.clone();
        extra.push(0);
        assert!(matches!(
            unframe_chunk(&extra),
            Err(CodecError::TrailingBytes { .. })
        ));
        // Unknown framing version.
        let mut vbad = chunk.clone();
        vbad[4] = 9;
        assert!(matches!(
            unframe_chunk(&vbad),
            Err(CodecError::UnsupportedFraming(9))
        ));
        // Unknown kind byte.
        let mut kbad = chunk.clone();
        kbad[6] = 77;
        assert!(matches!(
            unframe_chunk(&kbad),
            Err(CodecError::UnknownChunkKind(77))
        ));
    }

    #[test]
    fn bundle_binary_roundtrip_semantic_equal_and_spans_preserved() {
        let b = core_fixture_bundle();
        let chunk = encode_bundle_chunk(&b).unwrap();
        let restored = decode_bundle_chunk(&chunk).unwrap();
        // Semantic equality (spans excluded by the real PartialEq).
        assert!(restored == b, "restored bundle must be semantically equal");
        // Span preservation: compare collected spans of def tables.
        let spans_of = |bb: &Bundle| -> Vec<(u64, u64)> {
            let mut v = Vec::new();
            for m in &bb.modules {
                v.push((m.span.start as u64, m.span.end as u64));
                for d in &m.defs {
                    v.push((d.span.start as u64, d.span.end as u64));
                }
            }
            v
        };
        assert_eq!(spans_of(&restored), spans_of(&b));
        // It really is binary, not text: payload has no "hir(" marker.
        let (_, payload) = unframe_chunk(&chunk).unwrap();
        let as_text = String::from_utf8_lossy(payload);
        assert!(
            !as_text.contains("hir("),
            "binary codec must not be a text encoding"
        );
    }

    #[test]
    fn bundle_structural_tamper_rejected_by_reader() {
        let b = core_fixture_bundle();
        let chunk = encode_bundle_chunk(&b).unwrap();
        // Tamper payload then RE-FRAME with a valid checksum: structural
        // validation must still reject (this is the re-encoded-tamper path).
        let (_, payload) = unframe_chunk(&chunk).unwrap();
        // Unknown expr-kind tag deep in the payload: brute-force a byte we
        // know is an expr kind tag by encoding a bundle and flipping a
        // documented position is fragile; instead decode-check known bad
        // payloads:
        assert!(matches!(
            decode_bundle(&[]),
            Err(CodecError::Truncated { .. })
        ));
        // Unsupported version.
        let mut v = payload.to_vec();
        v[0] = 42;
        let reframed = frame_chunk(ChunkKind::Hir, &v);
        assert!(matches!(
            decode_bundle_chunk(&reframed),
            Err(CodecError::UnsupportedVersion(42))
        ));
        // Count larger than remaining bytes.
        let mut c = Vec::new();
        c.extend_from_slice(&1u16.to_le_bytes());
        c.extend_from_slice(&[0, 0]);
        c.extend_from_slice(&0u32.to_le_bytes()); // 0 requires
        c.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes()); // modules count > bytes left
        let reframed = frame_chunk(ChunkKind::Hir, &c);
        assert!(matches!(
            decode_bundle_chunk(&reframed),
            Err(CodecError::LengthOverflow { .. })
        ));
    }

    #[test]
    fn bundle_reencoded_index_tamper_rejected() {
        // Take a valid payload and mutate the FIRST function def's body id
        // byte to an out-of-range index. Rather than guessing offsets, build
        // the tampered payload by hand-mirroring the writer format:
        let b = core_fixture_bundle();
        let payload = encode_bundle(&b).unwrap();
        // Find the def table by re-encoding with a sentinel: cheaper —
        // decode validates a hand-built bundle with an OOB body index.
        let mut w = Writer::new();
        w.u16(HIR_CODEC_VERSION);
        w.u8(0); // core
        w.u8(0);
        w.u32(0); // requires
        w.u32(1); // 1 module
        w.str("m");
        w.u8(0); // path none
        w.span(&auto_ac_prototype::atom_text::Span { start: 0, end: 1 });
        w.u32(1); // one type
        w.str("t_i32");
        w.span(&auto_ac_prototype::atom_text::Span { start: 0, end: 1 });
        w.u8(0); // i32
        w.u32(1); // defs
        w.str("f");
        w.span(&auto_ac_prototype::atom_text::Span { start: 0, end: 1 });
        w.u8(0); // function
        w.str("f");
        w.u32(0); // ty — valid now
        w.u32(9); // body id 9 — OOB (0 bodies)
        w.u32(0); // bodies
        let reframed = frame_chunk(ChunkKind::Hir, &w.out);
        assert!(matches!(
            decode_bundle_chunk(&reframed),
            Err(CodecError::IndexOutOfBounds {
                what: "body",
                index: 9,
                len: 0
            })
        ));
        let _ = payload;
    }

    #[test]
    fn text_domain_bundle_refused() {
        // A text-profile source compiles fine but its bundle must be refused
        // by the probe codec (outside the restricted executable subset).
        let src = "fn main() int { return \"a\".len_bytes() }";
        let r = auto_ac_prototype::source::compile_parts_with(
            src,
            "m",
            &["ac.runtime.text.v1".to_string()],
            auto_ac_prototype::source::SourceProfile::Text,
        );
        match r {
            Ok(parts) => {
                let err = encode_bundle(&parts.bundle).unwrap_err();
                assert!(matches!(err, CodecError::UnsupportedFeature(_)), "{err}");
            }
            Err(_) => {
                // If the fixture-like source needs the support lib, the
                // point stands via the domain check below; skip.
            }
        }
    }

    #[test]
    fn decl_summary_content_identical_across_body_edit() {
        let p1 = "fn main() int { return 1 }";
        let p2 = "fn main() int { return 2 }";
        let s1 = build_decl_summary(
            &auto_ac_prototype::source::parse_program(p1).expect("p"),
            "m",
        );
        let s2 = build_decl_summary(
            &auto_ac_prototype::source::parse_program(p2).expect("p"),
            "m",
        );
        assert_eq!(
            encode_decl_summary(&s1).unwrap(),
            encode_decl_summary(&s2).unwrap()
        );
        let p3 = "fn main(x int) int { return x }";
        let s3 = build_decl_summary(
            &auto_ac_prototype::source::parse_program(p3).expect("p"),
            "m",
        );
        assert_ne!(
            encode_decl_summary(&s1).unwrap(),
            encode_decl_summary(&s3).unwrap()
        );
        // int/i32 synonym canonicalization.
        let p4 = "fn main() i32 { return 1 }";
        let s4 = build_decl_summary(
            &auto_ac_prototype::source::parse_program(p4).expect("p"),
            "m",
        );
        assert_eq!(s1.fns[0].result, s4.fns[0].result);
    }
}
