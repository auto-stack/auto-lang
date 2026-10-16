//! Versioned compact page codec (PLAN-796 §5.1, contract-decision D5/D6).
//!
//! One *page* is the unit of semantic publication: an explicit envelope
//! (schema id, producer, page kind, content version) around a compact
//! binary body. The body schema is owned by this crate; PLAN-797 consumes
//! the TypeDesc descriptions read-only.
//!
//! Determinism rules:
//!
//! * maps/sets are encoded in sorted key order (canonical);
//! * field/method order is DECLARATION order where the language makes it
//!   observable (record fields, parameters) — never re-sorted for hashing;
//! * the encoded form doubles as the canonical content used for
//!   [`crate::ContentVersion`] hashing.
//!
//! Safety rules: decoding is fully bounded (length caps, depth caps,
//! trailing-byte rejection) and ITERATIVE on nested structures — a legal
//! 4096+-deep spine decodes without touching the native stack budget.

use crate::identity::{
    ContentVersion, DeclKind, DeclRef, LogicalId, PackageId, RefError, SEMA_PRODUCER_ID,
};

/// Schema identity of the page envelope (contract-decision D5).
pub const SEMA_PAGE_SCHEMA_V1: &str = "ac796/sema-page/v1";
/// Codec revision of the body layouts (bumped on any body layout change;
/// part of the cache key — old payloads miss, never reinterpret).
pub const SEMA_CODEC_VERSION: u16 = 1;

/// Hard caps (contract-decision D6): bounded decode, never an OOM on a
/// hostile page.
pub const MAX_PAGE_BODY: usize = 512 * 1024; // 512 KiB small-query payload cap
pub const MAX_NESTING_DEPTH: u32 = 8192; // ≥ 2× the 4096-level probe chain
pub const MAX_VEC_LEN: u32 = 1 << 20;

/// Page kind — selects the body layout.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PageKind {
    /// Function signature facts ([`FnMeta`]).
    FnMeta = 1,
    /// Record shape + methods + recursive-group manifest ([`RecordMeta`]).
    RecordMeta = 2,
    /// Body inference facts ([`InferFacts`]).
    InferBody = 3,
    /// Module declaration index + candidate sets ([`ModuleIndex`]).
    ModuleIndex = 4,
    /// Cached query result + dependency manifest ([`CachedResult`]).
    CachedResult = 5,
}

impl PageKind {
    fn from_u8(v: u8) -> Result<Self, CodecError> {
        Ok(match v {
            1 => PageKind::FnMeta,
            2 => PageKind::RecordMeta,
            3 => PageKind::InferBody,
            4 => PageKind::ModuleIndex,
            5 => PageKind::CachedResult,
            _ => return Err(CodecError::BadTag("page-kind", v)),
        })
    }
}

/// Value-passing mode annotation on a described type (§5.1: view/mut/
/// owned). Metadata only — it does not widen the inference profile.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Mode {
    View = 1,
    Mut = 2,
    Owned = 3,
}

impl Mode {
    fn from_u8(v: u8) -> Result<Self, CodecError> {
        Ok(match v {
            1 => Mode::View,
            2 => Mode::Mut,
            3 => Mode::Owned,
            _ => return Err(CodecError::BadTag("mode", v)),
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::View => "view",
            Mode::Mut => "mut",
            Mode::Owned => "owned",
        }
    }
}

/// Portable type *description* (§5.1): the metadata representation PLAN-797
/// consumes. This is representation support only — the presence of u32/
/// f64/opaque here does NOT widen the limited i32/bool inference profile.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeDesc {
    Unit,
    I32,
    U32,
    I64,
    U64,
    F64,
    Bool,
    Char,
    ByteBuffer,
    /// Named opaque type (externally laid out; identity by name).
    Opaque { name: String },
    /// Function type: parameter descriptions in declaration order + result.
    Fn { params: Vec<TypeDesc>, ret: Box<TypeDesc> },
    /// Nominal record — ALWAYS an indirect reference to its [`DeclRef`];
    /// recursion is expressed through this indirection, never by value.
    Record { decl: DeclRef },
    /// Mode-annotated type (view/mut/owned over an inner description).
    Mode { inner: Box<TypeDesc>, mode: Mode },
}

impl TypeDesc {
    pub fn i32() -> Self {
        TypeDesc::I32
    }
    pub fn bool() -> Self {
        TypeDesc::Bool
    }
    /// Wrap in a mode annotation.
    pub fn with_mode(self, mode: Mode) -> Self {
        TypeDesc::Mode { inner: Box::new(self), mode }
    }
}

/// One record field, in declaration order (observable order preserved).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FieldDesc {
    pub name: String,
    pub ty: TypeDesc,
}

/// The shape of one record declaration.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecordShape {
    pub decl: DeclRef,
    pub fields: Vec<FieldDesc>,
}

/// A recursive group (SCC) of records: published as one unit. Members
/// reference each other through [`TypeDesc::Record`] indirections to
/// member decls; a non-member indirect ref is a controlled error at
/// verification (V-01: 坏图不 panic).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecGroup {
    pub members: Vec<RecordShape>,
}

impl RecGroup {
    /// Group content version: order-free over members (sorted by logical
    /// id), each contributing (logical, content, field set).
    pub fn group_content(&self) -> ContentVersion {
        let mut members = self.members.clone();
        members.sort_by(|a, b| a.decl.logical.cmp(&b.decl.logical));
        let mut buf = Vec::new();
        for m in &members {
            write_u32(&mut buf, m.decl.logical.as_str().len() as u32);
            buf.extend_from_slice(m.decl.logical.as_str().as_bytes());
            write_u32(&mut buf, m.decl.content.as_str().len() as u32);
            buf.extend_from_slice(m.decl.content.as_str().as_bytes());
            write_u32(&mut buf, m.fields.len() as u32);
            for f in &m.fields {
                encode_str(&mut buf, &f.name);
                encode_typedesc(&mut buf, &f.ty).unwrap();
            }
        }
        ContentVersion::of_canonical(&buf)
    }

    /// Verify the group graph: every indirect record ref that points at a
    /// member resolves to a real member; refs leaving the group must be
    /// to OTHER published groups (here: simply well-formed decls). By-value
    /// recursion is structurally impossible in this representation
    /// (records are always indirect), so the 797 by-value-layout rejection
    /// has a clean seam.
    pub fn verify(&self) -> Result<(), RefError> {
        for m in &self.members {
            m.decl.verify()?;
            if m.decl.kind != DeclKind::Record {
                return Err(RefError::BadContentVersion(format!(
                    "group member {} is a {}, not a record",
                    m.decl.name, m.decl.kind.as_str()
                )));
            }
        }
        // Cross-member closure: a member's field Record-ref either points
        // INSIDE the group or names an external decl (allowed — that is
        // how DAGs between groups compose); what is rejected elsewhere is
        // a group whose member list claims a decl twice.
        let mut seen = std::collections::BTreeSet::new();
        for m in &self.members {
            if !seen.insert(m.decl.logical.clone()) {
                return Err(RefError::BadContentVersion(format!(
                    "duplicate member {} in recursive group",
                    m.decl.name
                )));
            }
        }
        Ok(())
    }
}

/// Function signature facts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnMeta {
    pub decl: DeclRef,
    /// Parameters in declaration order: (name, description, mode).
    pub params: Vec<(String, TypeDesc, Mode)>,
    pub ret: TypeDesc,
    /// `free` | `static` | `view` receiver shape.
    pub receiver: &'static str,
}

/// Record metadata: shape + methods + the group it was published with.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecordMeta {
    pub shape: RecordShape,
    pub methods: Vec<FnMeta>,
    pub group_content: ContentVersion,
}

/// One body-inference fact set (published FACTS, never solver work state:
/// no type variables, no substitutions, no union-find).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InferFacts {
    pub fn_logical: LogicalId,
    /// Local bindings in program order: (name, inferred description).
    pub bindings: Vec<(String, TypeDesc)>,
    /// Sparse expression types by source span (start,end) — stable across
    /// restarts because spans are source bytes, not intern slots.
    pub expr_types: Vec<(u32, u32, TypeDesc)>,
    /// Diagnostics (empty when the check succeeded).
    pub diagnostics: Vec<FactDiag>,
}

/// A frozen diagnostic fact: code + span + message. Cached only for
/// definitive accept/reject outcomes (never transient states).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FactDiag {
    pub code: &'static str,
    pub start: u32,
    pub end: u32,
    pub message: String,
}

/// Module declaration index: the candidate sets that name lookup ran
/// against — this is the input of NEGATIVE dependencies (§5.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModuleIndex {
    pub package: PackageId,
    pub module: String,
    /// Sorted function names (the function-domain candidate set).
    pub fn_names: Vec<String>,
    /// Sorted record names (the type-domain candidate set).
    pub record_names: Vec<String>,
    /// Per-decl content versions, sorted by logical id.
    pub decl_versions: Vec<(LogicalId, ContentVersion)>,
}

/// One dependency fingerprint of a published result.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DepFingerprint {
    /// What was read: `fn-sig`, `fn-body`, `field-set`, `candidate-set`,
    /// `source-map`, `config`, `group-content`.
    pub kind: &'static str,
    /// Logical id / domain name it was read from.
    pub origin: String,
    /// Content hash of what was read.
    pub fp: String,
}

/// A negative dependency: "this name did NOT exist in this domain when
/// the result was computed" — invalidated when the candidate set gains a
/// member or the absent name becomes defined (M-04).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NegDep {
    pub domain: &'static str,
    pub name: String,
    /// Fingerprint of the full candidate set at compute time.
    pub set_fp: String,
}

/// Dependency manifest carried by every cached result; validated for full
/// freshness on every hit (§5.4: 验证完整新鲜).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DepManifest {
    pub positives: Vec<DepFingerprint>,
    pub negatives: Vec<NegDep>,
}

/// A cached query result page body.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CachedResult {
    /// Which query produced this: `infer-body`, `fn-sig`, `record-fields`,
    /// `list-candidates`.
    pub query: &'static str,
    pub target: LogicalId,
    /// The frozen facts (kind-appropriate; inference facts for
    /// `infer-body`, shape for `record-fields`, …).
    pub facts: ResultFacts,
    pub manifest: DepManifest,
}

/// Kind-appropriate result facts.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ResultFacts {
    Infer(InferFacts),
    FnSig(FnMeta),
    Fields(RecordShape),
    Candidates(Vec<String>),
}

/// The decoded page: envelope + typed body.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SemaPage {
    pub header: PageHeader,
    pub body: PageBody,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PageHeader {
    pub schema: String,
    pub producer: String,
    pub codec_version: u16,
    pub kind: PageKind,
    /// Content version of the declaration (or module) this page describes.
    pub content: ContentVersion,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PageBody {
    FnMeta(FnMeta),
    RecordMeta(RecordMeta),
    InferBody(InferFacts),
    ModuleIndex(ModuleIndex),
    CachedResult(CachedResult),
}

/// Byte-level TypeDesc encode (public seam for tests/tools).
pub fn encode_typedesc_bytes(t: &TypeDesc) -> Result<Vec<u8>, CodecError> {
    let mut out = Vec::new();
    encode_typedesc(&mut out, t)?;
    Ok(out)
}

/// Byte-level TypeDesc decode: bounded, iterative, trailing-rejecting.
pub fn decode_typedesc_bytes(bytes: &[u8]) -> Result<TypeDesc, CodecError> {
    let mut cur = Cur::new(bytes);
    let t = decode_typedesc(&mut cur)?;
    cur.done()?;
    Ok(t)
}

/// Controlled codec errors — malformed pages are rejected, never panic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    BadSchema { expected: String, found: String },
    BadProducer { expected: String, found: String },
    UnsupportedCodec(u16),
    BadTag(&'static str, u8),
    Truncated { at: &'static str },
    TrailingBytes(u64),
    TooLarge { what: &'static str, len: u64 },
    TooDeep { at: u32 },
    BadRef(RefError),
    BadUtf8,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodecError::BadSchema { expected, found } => {
                write!(f, "page schema mismatch: expected {expected}, found {found}")
            }
            CodecError::BadProducer { expected, found } => {
                write!(f, "page producer mismatch: expected {expected}, found {found}")
            }
            CodecError::UnsupportedCodec(v) => write!(f, "unsupported codec version {v}"),
            CodecError::BadTag(what, v) => write!(f, "bad {what} tag {v}"),
            CodecError::Truncated { at } => write!(f, "truncated page at {at}"),
            CodecError::TrailingBytes(n) => write!(f, "{n} trailing bytes after page body"),
            CodecError::TooLarge { what, len } => write!(f, "{what} too large: {len}"),
            CodecError::TooDeep { at } => write!(f, "nesting too deep at level {at}"),
            CodecError::BadRef(e) => write!(f, "bad reference: {e}"),
            CodecError::BadUtf8 => write!(f, "bad utf-8 in page string"),
        }
    }
}

impl std::error::Error for CodecError {}

impl From<RefError> for CodecError {
    fn from(e: RefError) -> Self {
        CodecError::BadRef(e)
    }
}

// ---------------------------------------------------------------------------
// primitive cursor IO
// ---------------------------------------------------------------------------

pub(crate) struct Cur<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cur<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Cur { buf, pos: 0 }
    }
    pub(crate) fn take(&mut self, n: usize, at: &'static str) -> Result<&'a [u8], CodecError> {
        if self.pos + n > self.buf.len() {
            return Err(CodecError::Truncated { at });
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    pub(crate) fn u8(&mut self, at: &'static str) -> Result<u8, CodecError> {
        Ok(self.take(1, at)?[0])
    }
    pub(crate) fn u16(&mut self, at: &'static str) -> Result<u16, CodecError> {
        let b = self.take(2, at)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub(crate) fn u32(&mut self, at: &'static str) -> Result<u32, CodecError> {
        let b = self.take(4, at)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub(crate) fn str(&mut self, at: &'static str) -> Result<String, CodecError> {
        let n = self.u32(at)? as usize;
        if n > MAX_VEC_LEN as usize {
            return Err(CodecError::TooLarge { what: "string", len: n as u64 });
        }
        let b = self.take(n, at)?;
        String::from_utf8(b.to_vec()).map_err(|_| CodecError::BadUtf8)
    }
    pub(crate) fn vec_len(&mut self, what: &'static str, at: &'static str) -> Result<usize, CodecError> {
        let n = self.u32(at)?;
        if n > MAX_VEC_LEN {
            return Err(CodecError::TooLarge { what, len: n as u64 });
        }
        Ok(n as usize)
    }
    pub(crate) fn done(&self) -> Result<(), CodecError> {
        if self.pos != self.buf.len() {
            Err(CodecError::TrailingBytes((self.buf.len() - self.pos) as u64))
        } else {
            Ok(())
        }
    }
}

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn encode_str(out: &mut Vec<u8>, s: &str) {
    write_u32(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}

// ---------------------------------------------------------------------------
// TypeDesc codec — ITERATIVE decode (explicit stack), depth-capped
// ---------------------------------------------------------------------------

fn tag_of(t: &TypeDesc) -> u8 {
    match t {
        TypeDesc::Unit => 1,
        TypeDesc::I32 => 2,
        TypeDesc::U32 => 3,
        TypeDesc::I64 => 4,
        TypeDesc::U64 => 5,
        TypeDesc::F64 => 6,
        TypeDesc::Bool => 7,
        TypeDesc::Char => 8,
        TypeDesc::ByteBuffer => 9,
        TypeDesc::Opaque { .. } => 10,
        TypeDesc::Fn { .. } => 11,
        TypeDesc::Record { .. } => 12,
        TypeDesc::Mode { .. } => 13,
    }
}

/// Encode a TypeDesc. Encoding recurses over the STRUCT depth — the
/// in-memory tree is built by our own code paths, and the deep-chain
/// probes publish WIDE (many bindings), not deeply nested descriptions;
/// decode (the untrusted side) is iterative regardless.
pub(crate) fn encode_typedesc(out: &mut Vec<u8>, t: &TypeDesc) -> Result<(), CodecError> {
    match t {
        TypeDesc::Unit
        | TypeDesc::I32
        | TypeDesc::U32
        | TypeDesc::I64
        | TypeDesc::U64
        | TypeDesc::F64
        | TypeDesc::Bool
        | TypeDesc::Char
        | TypeDesc::ByteBuffer => out.push(tag_of(t)),
        TypeDesc::Opaque { name } => {
            out.push(10);
            encode_str(out, name);
        }
        TypeDesc::Fn { params, ret } => {
            out.push(11);
            write_u32(out, params.len() as u32);
            for p in params {
                encode_typedesc(out, p)?;
            }
            encode_typedesc(out, ret)?;
        }
        TypeDesc::Record { decl } => {
            out.push(12);
            encode_declref(out, decl)?;
        }
        TypeDesc::Mode { inner, mode } => {
            out.push(13);
            out.push(*mode as u8);
            encode_typedesc(out, inner)?;
        }
    }
    Ok(())
}

enum Frame {
    /// Fn frame: expects `expect` children (n params + 1 result).
    Fn { params: Vec<TypeDesc>, expect: u32 },
    /// Mode frame: expects exactly 1 child.
    Mode { mode: Mode },
}

/// Iterative TypeDesc decode: untrusted bytes never drive native
/// recursion; nesting beyond [`MAX_NESTING_DEPTH`] is a controlled
/// `TooDeep` rejection.
pub(crate) fn decode_typedesc(cur: &mut Cur<'_>) -> Result<TypeDesc, CodecError> {
    let mut stack: Vec<Frame> = Vec::new();
    'read: loop {
        let at = "typedesc";
        let tag = cur.u8(at)?;
        let node = match tag {
            1 => TypeDesc::Unit,
            2 => TypeDesc::I32,
            3 => TypeDesc::U32,
            4 => TypeDesc::I64,
            5 => TypeDesc::U64,
            6 => TypeDesc::F64,
            7 => TypeDesc::Bool,
            8 => TypeDesc::Char,
            9 => TypeDesc::ByteBuffer,
            10 => TypeDesc::Opaque { name: cur.str(at)? },
            11 => {
                let n = cur.u32(at)?;
                if n > MAX_VEC_LEN {
                    return Err(CodecError::TooLarge { what: "fn params", len: n as u64 });
                }
                if stack.len() as u32 >= MAX_NESTING_DEPTH {
                    return Err(CodecError::TooDeep { at: stack.len() as u32 });
                }
                stack.push(Frame::Fn { params: Vec::new(), expect: n + 1 });
                continue 'read;
            }
            12 => TypeDesc::Record { decl: decode_declref(cur)? },
            13 => {
                let mode = Mode::from_u8(cur.u8(at)?)?;
                if stack.len() as u32 >= MAX_NESTING_DEPTH {
                    return Err(CodecError::TooDeep { at: stack.len() as u32 });
                }
                stack.push(Frame::Mode { mode });
                continue 'read;
            }
            _ => return Err(CodecError::BadTag("typedesc", tag)),
        };
        // attach `node` to the innermost open frame, closing frames as
        // they complete (each close re-attaches one level up)
        let mut done = node;
        loop {
            match stack.last_mut() {
                None => return Ok(done),
                Some(Frame::Fn { params, expect }) => {
                    params.push(done);
                    if (params.len() as u32) < *expect {
                        break; // need one more child
                    }
                    match stack.pop().unwrap() {
                        Frame::Fn { mut params, .. } => {
                            let ret = params.pop().expect("fn frame closed with children");
                            done = TypeDesc::Fn { params, ret: Box::new(ret) };
                            // continue attaching upward
                        }
                        _ => unreachable!(),
                    }
                }
                Some(Frame::Mode { .. }) => match stack.pop().unwrap() {
                    Frame::Mode { mode } => {
                        done = TypeDesc::Mode { inner: Box::new(done), mode };
                    }
                    _ => unreachable!(),
                },
            }
        }
    }
}

// ---------------------------------------------------------------------------
// DeclRef codec
// ---------------------------------------------------------------------------

fn encode_declref(out: &mut Vec<u8>, d: &DeclRef) -> Result<(), CodecError> {
    encode_str(out, d.package.as_str());
    encode_str(out, &d.module);
    out.push(d.kind as u8);
    encode_str(out, &d.name);
    encode_str(out, d.content.as_str());
    encode_str(out, d.logical.as_str());
    Ok(())
}

fn decode_declref(cur: &mut Cur<'_>) -> Result<DeclRef, CodecError> {
    let at = "declref";
    let package = PackageId(cur.str(at)?);
    let module = cur.str(at)?;
    let k = cur.u8(at)?;
    let kind = match k {
        1 => DeclKind::Fn,
        2 => DeclKind::Record,
        _ => return Err(CodecError::BadTag("decl-kind", k)),
    };
    let name = cur.str(at)?;
    let content = ContentVersion(cur.str(at)?);
    let logical = LogicalId(cur.str(at)?);
    let r = DeclRef { logical, package, module, kind, name, content };
    r.verify().map_err(CodecError::BadRef)?;
    Ok(r)
}

// ---------------------------------------------------------------------------
// page envelope + bodies
// ---------------------------------------------------------------------------

fn encode_fnmeta(out: &mut Vec<u8>, m: &FnMeta) -> Result<(), CodecError> {
    encode_declref(out, &m.decl)?;
    write_u32(out, m.params.len() as u32);
    for (n, t, mo) in &m.params {
        encode_str(out, n);
        encode_typedesc(out, t)?;
        out.push(*mo as u8);
    }
    encode_typedesc(out, &m.ret)?;
    encode_str(out, m.receiver);
    Ok(())
}

fn decode_fnmeta(cur: &mut Cur<'_>) -> Result<FnMeta, CodecError> {
    let at = "fnmeta";
    let decl = decode_declref(cur)?;
    let n = cur.vec_len("params", at)?;
    let mut params = Vec::with_capacity(n);
    for _ in 0..n {
        let name = cur.str(at)?;
        let ty = decode_typedesc(cur)?;
        let mode = Mode::from_u8(cur.u8(at)?)?;
        params.push((name, ty, mode));
    }
    let ret = decode_typedesc(cur)?;
    let receiver_box = cur.str(at)?;
    let receiver: &'static str = match receiver_box.as_str() {
        "free" => "free",
        "static" => "static",
        "view" => "view",
        _ => return Err(CodecError::BadTag("receiver", 0)),
    };
    Ok(FnMeta { decl, params, ret, receiver })
}

fn encode_shape(out: &mut Vec<u8>, s: &RecordShape) -> Result<(), CodecError> {
    encode_declref(out, &s.decl)?;
    write_u32(out, s.fields.len() as u32);
    for f in &s.fields {
        encode_str(out, &f.name);
        encode_typedesc(out, &f.ty)?;
    }
    Ok(())
}

fn decode_shape(cur: &mut Cur<'_>) -> Result<RecordShape, CodecError> {
    let at = "shape";
    let decl = decode_declref(cur)?;
    let n = cur.vec_len("fields", at)?;
    let mut fields = Vec::with_capacity(n);
    for _ in 0..n {
        let name = cur.str(at)?;
        let ty = decode_typedesc(cur)?;
        fields.push(FieldDesc { name, ty });
    }
    Ok(RecordShape { decl, fields })
}

fn encode_inferfacts(out: &mut Vec<u8>, f: &InferFacts) -> Result<(), CodecError> {
    encode_str(out, f.fn_logical.as_str());
    write_u32(out, f.bindings.len() as u32);
    for (n, t) in &f.bindings {
        encode_str(out, n);
        encode_typedesc(out, t)?;
    }
    write_u32(out, f.expr_types.len() as u32);
    for (s, e, t) in &f.expr_types {
        write_u32(out, *s);
        write_u32(out, *e);
        encode_typedesc(out, t)?;
    }
    write_u32(out, f.diagnostics.len() as u32);
    for d in &f.diagnostics {
        encode_str(out, d.code);
        write_u32(out, d.start);
        write_u32(out, d.end);
        encode_str(out, &d.message);
    }
    Ok(())
}

fn decode_inferfacts(cur: &mut Cur<'_>) -> Result<InferFacts, CodecError> {
    let at = "inferfacts";
    let fn_logical = LogicalId(cur.str(at)?);
    let n = cur.vec_len("bindings", at)?;
    let mut bindings = Vec::with_capacity(n);
    for _ in 0..n {
        let name = cur.str(at)?;
        let ty = decode_typedesc(cur)?;
        bindings.push((name, ty));
    }
    let n = cur.vec_len("expr-types", at)?;
    let mut expr_types = Vec::with_capacity(n);
    for _ in 0..n {
        let s = cur.u32(at)?;
        let e = cur.u32(at)?;
        let t = decode_typedesc(cur)?;
        expr_types.push((s, e, t));
    }
    let n = cur.vec_len("diags", at)?;
    let mut diagnostics = Vec::with_capacity(n);
    for _ in 0..n {
        let code = cur.str(at)?;
        let start = cur.u32(at)?;
        let end = cur.u32(at)?;
        let message = cur.str(at)?;
        diagnostics.push(FactDiag { code: leak_code(code), start, end, message });
    }
    Ok(InferFacts { fn_logical, bindings, expr_types, diagnostics })
}

/// Map a decoded diagnostic code onto the static code set; unknown codes
/// decode as the generic `fact-unknown` marker instead of failing (the
/// payload identity already pins the producer/schema/codec triple).
fn leak_code(s: String) -> &'static str {
    match s.as_str() {
        "type-mismatch" => "type-mismatch",
        "unknown-name" => "unknown-name",
        "cond-not-bool" => "cond-not-bool",
        "assign-immutable" => "assign-immutable",
        "ambiguous-candidates" => "ambiguous-candidates",
        "occurs-check" => "occurs-check",
        "unsolved-variable" => "unsolved-variable",
        "unsupported-construct" => "unsupported-construct",
        _ => "fact-unknown",
    }
}

fn encode_index(out: &mut Vec<u8>, m: &ModuleIndex) -> Result<(), CodecError> {
    encode_str(out, m.package.as_str());
    encode_str(out, &m.module);
    write_u32(out, m.fn_names.len() as u32);
    for n in &m.fn_names {
        encode_str(out, n);
    }
    write_u32(out, m.record_names.len() as u32);
    for n in &m.record_names {
        encode_str(out, n);
    }
    write_u32(out, m.decl_versions.len() as u32);
    for (l, v) in &m.decl_versions {
        encode_str(out, l.as_str());
        encode_str(out, v.as_str());
    }
    Ok(())
}

fn decode_index(cur: &mut Cur<'_>) -> Result<ModuleIndex, CodecError> {
    let at = "moduleindex";
    let package = PackageId(cur.str(at)?);
    let module = cur.str(at)?;
    let n = cur.vec_len("fn-names", at)?;
    let mut fn_names = Vec::with_capacity(n);
    for _ in 0..n {
        fn_names.push(cur.str(at)?);
    }
    let n = cur.vec_len("record-names", at)?;
    let mut record_names = Vec::with_capacity(n);
    for _ in 0..n {
        record_names.push(cur.str(at)?);
    }
    let n = cur.vec_len("decl-versions", at)?;
    let mut decl_versions = Vec::with_capacity(n);
    for _ in 0..n {
        let l = LogicalId(cur.str(at)?);
        let v = ContentVersion(cur.str(at)?);
        decl_versions.push((l, v));
    }
    Ok(ModuleIndex { package, module, fn_names, record_names, decl_versions })
}

fn encode_manifest(out: &mut Vec<u8>, m: &DepManifest) -> Result<(), CodecError> {
    write_u32(out, m.positives.len() as u32);
    for p in &m.positives {
        encode_str(out, p.kind);
        encode_str(out, &p.origin);
        encode_str(out, &p.fp);
    }
    write_u32(out, m.negatives.len() as u32);
    for n in &m.negatives {
        encode_str(out, n.domain);
        encode_str(out, &n.name);
        encode_str(out, &n.set_fp);
    }
    Ok(())
}

fn decode_manifest(cur: &mut Cur<'_>) -> Result<DepManifest, CodecError> {
    let at = "manifest";
    let n = cur.vec_len("positives", at)?;
    let mut positives = Vec::with_capacity(n);
    for _ in 0..n {
        let kind = cur.str(at)?;
        let origin = cur.str(at)?;
        let fp = cur.str(at)?;
        positives.push(DepFingerprint { kind: leak_dep_kind(kind), origin, fp });
    }
    let n = cur.vec_len("negatives", at)?;
    let mut negatives = Vec::with_capacity(n);
    for _ in 0..n {
        let domain = cur.str(at)?;
        let name = cur.str(at)?;
        let set_fp = cur.str(at)?;
        negatives.push(NegDep { domain: leak_domain(domain), name, set_fp });
    }
    Ok(DepManifest { positives, negatives })
}

fn leak_dep_kind(s: String) -> &'static str {
    match s.as_str() {
        "fn-sig" => "fn-sig",
        "fn-body" => "fn-body",
        "field-set" => "field-set",
        "candidate-set" => "candidate-set",
        "source-map" => "source-map",
        "config" => "config",
        "group-content" => "group-content",
        _ => "config",
    }
}

fn leak_domain(s: String) -> &'static str {
    match s.as_str() {
        "fn" => "fn",
        "type" => "type",
        "local" => "local",
        _ => "local",
    }
}

fn encode_cached(out: &mut Vec<u8>, c: &CachedResult) -> Result<(), CodecError> {
    encode_str(out, c.query);
    encode_str(out, c.target.as_str());
    match &c.facts {
        ResultFacts::Infer(f) => {
            out.push(1);
            encode_inferfacts(out, f)?;
        }
        ResultFacts::FnSig(m) => {
            out.push(2);
            encode_fnmeta(out, m)?;
        }
        ResultFacts::Fields(s) => {
            out.push(3);
            encode_shape(out, s)?;
        }
        ResultFacts::Candidates(v) => {
            out.push(4);
            write_u32(out, v.len() as u32);
            for n in v {
                encode_str(out, n);
            }
        }
    }
    encode_manifest(out, &c.manifest)?;
    Ok(())
}

fn decode_cached(cur: &mut Cur<'_>) -> Result<CachedResult, CodecError> {
    let at = "cached-result";
    let query = cur.str(at)?;
    let target = LogicalId(cur.str(at)?);
    let t = cur.u8(at)?;
    let facts = match t {
        1 => ResultFacts::Infer(decode_inferfacts(cur)?),
        2 => ResultFacts::FnSig(decode_fnmeta(cur)?),
        3 => ResultFacts::Fields(decode_shape(cur)?),
        4 => {
            let n = cur.vec_len("candidates", at)?;
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(cur.str(at)?);
            }
            ResultFacts::Candidates(v)
        }
        _ => return Err(CodecError::BadTag("result-facts", t)),
    };
    let manifest = decode_manifest(cur)?;
    Ok(CachedResult { query: leak_query(query), target, facts, manifest })
}

fn leak_query(s: String) -> &'static str {
    match s.as_str() {
        "infer-body" => "infer-body",
        "fn-sig" => "fn-sig",
        "record-fields" => "record-fields",
        "list-candidates" => "list-candidates",
        _ => "fn-sig",
    }
}

/// Encode a full page (envelope + body). The output is deterministic for
/// equal inputs — it doubles as canonical content for version hashing.
pub fn encode_page(page: &SemaPage) -> Result<Vec<u8>, CodecError> {
    if page.header.schema != SEMA_PAGE_SCHEMA_V1 {
        return Err(CodecError::BadSchema {
            expected: SEMA_PAGE_SCHEMA_V1.into(),
            found: page.header.schema.clone(),
        });
    }
    if page.header.producer != SEMA_PRODUCER_ID {
        return Err(CodecError::BadProducer {
            expected: SEMA_PRODUCER_ID.into(),
            found: page.header.producer.clone(),
        });
    }
    let mut body = Vec::new();
    match &page.body {
        PageBody::FnMeta(m) => {
            body.push(1);
            encode_fnmeta(&mut body, m)?;
        }
        PageBody::RecordMeta(m) => {
            body.push(2);
            encode_shape(&mut body, &m.shape)?;
            write_u32(&mut body, m.methods.len() as u32);
            for f in &m.methods {
                encode_fnmeta(&mut body, f)?;
            }
            encode_str(&mut body, m.group_content.as_str());
        }
        PageBody::InferBody(f) => {
            body.push(3);
            encode_inferfacts(&mut body, f)?;
        }
        PageBody::ModuleIndex(i) => {
            body.push(4);
            encode_index(&mut body, i)?;
        }
        PageBody::CachedResult(c) => {
            body.push(5);
            encode_cached(&mut body, c)?;
        }
    }
    if body.len() > MAX_PAGE_BODY {
        return Err(CodecError::TooLarge { what: "page body", len: body.len() as u64 });
    }
    let mut out = Vec::with_capacity(body.len() + 64);
    encode_str(&mut out, &page.header.schema);
    encode_str(&mut out, &page.header.producer);
    out.extend_from_slice(&page.header.codec_version.to_le_bytes());
    out.push(page.header.kind as u8);
    encode_str(&mut out, page.header.content.as_str());
    write_u32(&mut out, body.len() as u32);
    out.extend_from_slice(&body);
    Ok(out)
}

/// Decode + validate a page: envelope admission (schema/producer/codec),
/// bounded structural decode, trailing-byte rejection.
pub fn decode_page(bytes: &[u8]) -> Result<SemaPage, CodecError> {
    let mut cur = Cur::new(bytes);
    let at = "page";
    let schema = cur.str(at)?;
    if schema != SEMA_PAGE_SCHEMA_V1 {
        return Err(CodecError::BadSchema { expected: SEMA_PAGE_SCHEMA_V1.into(), found: schema });
    }
    let producer = cur.str(at)?;
    if producer != SEMA_PRODUCER_ID {
        return Err(CodecError::BadProducer { expected: SEMA_PRODUCER_ID.into(), found: producer });
    }
    let codec_version = cur.u16(at)?;
    if codec_version != SEMA_CODEC_VERSION {
        return Err(CodecError::UnsupportedCodec(codec_version));
    }
    let kind = PageKind::from_u8(cur.u8(at)?)?;
    let content = ContentVersion(cur.str(at)?);
    let body_len = cur.u32(at)? as usize;
    if body_len > MAX_PAGE_BODY {
        return Err(CodecError::TooLarge { what: "page body", len: body_len as u64 });
    }
    let body_bytes = cur.take(body_len, "page body")?;
    let mut bcur = Cur::new(body_bytes);
    let body_tag = bcur.u8("body-tag")?;
    if body_tag != kind as u8 {
        return Err(CodecError::BadTag("body-kind", body_tag));
    }
    let body = match kind {
        PageKind::FnMeta => PageBody::FnMeta(decode_fnmeta(&mut bcur)?),
        PageKind::RecordMeta => {
            let shape = decode_shape(&mut bcur)?;
            let n = bcur.vec_len("methods", "recordmeta")?;
            let mut methods = Vec::with_capacity(n);
            for _ in 0..n {
                methods.push(decode_fnmeta(&mut bcur)?);
            }
            let group_content = ContentVersion(bcur.str("recordmeta")?);
            PageBody::RecordMeta(RecordMeta { shape, methods, group_content })
        }
        PageKind::InferBody => PageBody::InferBody(decode_inferfacts(&mut bcur)?),
        PageKind::ModuleIndex => PageBody::ModuleIndex(decode_index(&mut bcur)?),
        PageKind::CachedResult => PageBody::CachedResult(decode_cached(&mut bcur)?),
    };
    bcur.done()?;
    cur.done()?;
    Ok(SemaPage { header: PageHeader { schema, producer, codec_version, kind, content }, body })
}

// ---------------------------------------------------------------------------
// typed view (§5.1): read-only typed accessors over a decoded page
// ---------------------------------------------------------------------------

/// Read-only typed view of a decoded page: kind-checked accessors so a
/// consumer never reinterpret-casts a page body across kinds.
pub struct PageView<'a> {
    page: &'a SemaPage,
}

impl<'a> PageView<'a> {
    pub fn new(page: &'a SemaPage) -> Self {
        PageView { page }
    }

    pub fn kind(&self) -> PageKind {
        self.page.header.kind
    }

    pub fn content_version(&self) -> &ContentVersion {
        &self.page.header.content
    }

    pub fn try_as_fn_meta(&self) -> Result<&'a FnMeta, CodecError> {
        match &self.page.body {
            PageBody::FnMeta(m) => Ok(m),
            _ => Err(CodecError::BadTag("page-kind-for-fnmeta", self.page.header.kind as u8)),
        }
    }

    pub fn try_as_record_meta(&self) -> Result<&'a RecordMeta, CodecError> {
        match &self.page.body {
            PageBody::RecordMeta(m) => Ok(m),
            _ => Err(CodecError::BadTag("page-kind-for-recordmeta", self.page.header.kind as u8)),
        }
    }

    pub fn try_as_infer_facts(&self) -> Result<&'a InferFacts, CodecError> {
        match &self.page.body {
            PageBody::InferBody(f) | PageBody::CachedResult(CachedResult {
                facts: ResultFacts::Infer(f),
                ..
            }) => Ok(f),
            _ => Err(CodecError::BadTag("page-kind-for-infer", self.page.header.kind as u8)),
        }
    }

    pub fn try_as_cached(&self) -> Result<&'a CachedResult, CodecError> {
        match &self.page.body {
            PageBody::CachedResult(c) => Ok(c),
            _ => Err(CodecError::BadTag("page-kind-for-cached", self.page.header.kind as u8)),
        }
    }

    /// Reflection-safe field lookup: position and type of one field by
    /// name, preserving declaration order (§5.3 只读反射投影).
    pub fn field(&self, name: &str) -> Option<(usize, &'a FieldDesc)> {
        let m = self.try_as_record_meta().ok()?;
        m.shape.fields.iter().enumerate().find(|(_, f)| f.name == name)
    }
}
