//! ABC wire codec v1 (PLAN-798 §5.2; layout frozen by `contract-decision.md`).
//!
//! A compact, self-describing little-endian byte format for the
//! `av.core-i32/1` profile — NOT a dump of the MIR Rust enums and NOT the
//! interchange format for any future public bytecode ABI. It carries
//! identity (version/profile flags), the module tables, per-function
//! slots/blocks/instructions/terminators, a diagnostics-only source-map
//! sidecar and an FNV-1a trailer.
//!
//! Decoding is pure structure: magic/version/profile/reserved checks,
//! checked length arithmetic, unknown opcode/sub-op rejection, bool
//! legality, index bounds and per-table quotas. Control-flow, initialization
//! and signature admission happen in [`crate::abc_verify`] AFTER decode —
//! an independently produced ABC input never skips verification because it
//! "came from this compiler".
//!
//! Paging: encode flushes ≤256 KiB pages; decode consumes pages. The
//! `AbcBytes` carrier keeps pages so large products stream instead of
//! double-buffering.

use crate::contract::{fnv1a64, ResourceKind};
use crate::mir::*;

// ---------------------------------------------------------------------------
// Wire identities (frozen)
// ---------------------------------------------------------------------------

pub const ABC_MAGIC: &[u8; 4] = b"AVAB";
/// 2 (F798-R12-03): the kind-bit generation. Version 1 artifacts carry no
/// `FLAG_IS_UNIT` and are a DIFFERENT format generation — decoding one as
/// the current wire would silently relabel a kind-less unit payload as a
/// complete module, so version 1 is an unsupported-version rejection, and
/// the cache keys/envelopes carry the /2 schema so old artifacts miss
/// deterministically instead of being read in place.
pub const ABC_VERSION: u16 = 2;
pub const ABC_PROFILE_CORE_I32: u16 = 1;

// opcodes
pub const OP_CONST: u8 = 0x01;
pub const OP_COPY: u8 = 0x02;
pub const OP_BIN: u8 = 0x03;
pub const OP_UN: u8 = 0x04;
pub const OP_CALL: u8 = 0x05;
pub const OP_TRACE: u8 = 0x06;
pub const TERM_JUMP: u8 = 0x10;
pub const TERM_BRANCH: u8 = 0x11;
pub const TERM_RETURN: u8 = 0x12;

// sub-ops
pub const BIN_ADD: u8 = 1;
pub const BIN_SUB: u8 = 2;
pub const BIN_MUL: u8 = 3;
pub const BIN_DIV: u8 = 4;
pub const BIN_REM: u8 = 5;
pub const BIN_LT: u8 = 6;
pub const BIN_LE: u8 = 7;
pub const BIN_GT: u8 = 8;
pub const BIN_GE: u8 = 9;
pub const BIN_EQ: u8 = 10;
pub const BIN_NE: u8 = 11;
pub const UN_NEG: u8 = 1;
pub const UN_NOT: u8 = 2;

// types
pub const TY_I32: u8 = 0;
pub const TY_BOOL: u8 = 1;
pub const TY_FN: u8 = 2;

// flags
pub const FLAG_SOURCE_SIDECAR: u16 = 1;
/// F798-R11-02: the artifact KIND travels on the wire — a single-function
/// WHOLE MODULE and a re-verifiable UNIT are different categories with
/// different admission contracts (a unit addresses callees through
/// imports; a module may call its own functions directly). Decoding must
/// not GUESS the kind from the function count.
pub const FLAG_IS_UNIT: u16 = 2;

/// Codec error taxonomy. `Resource` = frozen budget breach (e.g. page size,
/// table quota); `Format` = every structural violation (truncation, unknown
/// version/opcode, illegal bool, index out of range, checksum mismatch).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbcDecodeError {
    Format { code: String, message: String },
    Resource { kind: ResourceKind, message: String },
}

impl AbcDecodeError {
    fn fmt_err(code: &str, message: String) -> Self {
        AbcDecodeError::Format {
            code: code.to_string(),
            message,
        }
    }

    pub fn code(&self) -> String {
        match self {
            AbcDecodeError::Format { code, .. } => code.clone(),
            AbcDecodeError::Resource { kind, .. } => kind.code().to_string(),
        }
    }
}

/// Encoded ABC: identity + pages + total length.
#[derive(Clone, Debug)]
pub struct AbcBytes {
    pub version: u16,
    pub profile: u16,
    pub pages: Vec<Vec<u8>>,
    pub total_len: usize,
    /// FNV-1a over the full byte stream (trailer excluded; stored in the
    /// trailer itself).
    pub checksum: u64,
}

impl AbcBytes {
    pub fn to_flat(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.total_len);
        for p in &self.pages {
            out.extend_from_slice(p);
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Encode
// ---------------------------------------------------------------------------

struct PageWriter {
    pages: Vec<Vec<u8>>,
    cur: Vec<u8>,
    checksum: u64,
    total: usize,
}

impl PageWriter {
    fn new() -> Self {
        PageWriter {
            pages: Vec::new(),
            cur: Vec::new(),
            // Standard FNV-1a offset basis so decode_abc's fnv1a64 agrees.
            checksum: 0xcbf2_9ce4_8422_2325,
            total: 0,
        }
    }

    fn push(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.cur.push(*b);
            if self.cur.len() >= crate::contract::MAX_CODEC_PAGE_BYTES {
                let page = std::mem::take(&mut self.cur);
                self.absorb(&page);
                self.pages.push(page);
            }
        }
    }

    /// Update running checksum/total for a completed page.
    fn absorb(&mut self, page: &[u8]) {
        self.checksum = fnv1a64_cont(self.checksum, page);
        self.total += page.len();
    }

    fn finish(mut self) -> AbcBytes {
        if !self.cur.is_empty() {
            let page = std::mem::take(&mut self.cur);
            self.absorb(&page);
            self.pages.push(page);
        }
        AbcBytes {
            version: ABC_VERSION,
            profile: ABC_PROFILE_CORE_I32,
            pages: self.pages,
            total_len: self.total,
            checksum: self.checksum,
        }
    }
}

fn fnv1a64_cont(seed: u64, bytes: &[u8]) -> u64 {
    let mut h = seed;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn put_u16(w: &mut PageWriter, v: u16) {
    w.push(&v.to_le_bytes());
}
fn put_u32(w: &mut PageWriter, v: u32) {
    w.push(&v.to_le_bytes());
}
fn put_i32(w: &mut PageWriter, v: i32) {
    w.push(&v.to_le_bytes());
}
fn put_u8(w: &mut PageWriter, v: u8) {
    w.push(&[v]);
}

fn bin_op_code(op: BinOp) -> u8 {
    match op {
        BinOp::AddI32 => BIN_ADD,
        BinOp::SubI32 => BIN_SUB,
        BinOp::MulI32 => BIN_MUL,
        BinOp::DivI32 => BIN_DIV,
        BinOp::RemI32 => BIN_REM,
        BinOp::LtI32 => BIN_LT,
        BinOp::LeI32 => BIN_LE,
        BinOp::GtI32 => BIN_GT,
        BinOp::GeI32 => BIN_GE,
        BinOp::EqI32 => BIN_EQ,
        BinOp::NeI32 => BIN_NE,
    }
}

fn un_op_code(op: UnOp) -> u8 {
    match op {
        UnOp::NegI32 => UN_NEG,
        UnOp::NotBool => UN_NOT,
    }
}

/// Encode an unchecked MIR module to paged ABC bytes. The module must be
/// structurally encodable (slot/block/arg counts within wire field widths);
/// encoding performs width checks but NOT semantic verification.
pub fn encode_abc(m: &MirModule) -> Result<AbcBytes, AbcDecodeError> {
    let mut w = PageWriter::new();
    // Header.
    w.push(ABC_MAGIC);
    put_u16(&mut w, ABC_VERSION);
    put_u16(&mut w, ABC_PROFILE_CORE_I32);
    let has_sidecar = m
        .funcs
        .iter()
        .any(|f| !f.spans.is_empty() || f.source_snapshot != 0);
    let mut flags = 0u16;
    if has_sidecar {
        flags |= FLAG_SOURCE_SIDECAR;
    }
    if m.is_unit {
        flags |= FLAG_IS_UNIT;
    }
    put_u16(&mut w, flags);
    put_u16(&mut w, 0); // reserved
    put_u32(&mut w, m.types.len() as u32);
    put_u32(&mut w, m.consts.len() as u32);
    put_u32(&mut w, m.imports.len() as u32);
    put_u32(&mut w, m.funcs.len() as u32);
    put_u32(&mut w, m.entry);
    // Module capabilities (e.g. the test trace capability gating Trace
    // instructions) — admission data, carried in the header.
    if m.capabilities.len() > u16::MAX as usize {
        return Err(AbcDecodeError::fmt_err(
            "abc.encode.capabilities",
            "too many capabilities".to_string(),
        ));
    }
    put_u16(&mut w, m.capabilities.len() as u16);
    for c in &m.capabilities {
        let b = c.as_bytes();
        if b.len() > u16::MAX as usize {
            return Err(AbcDecodeError::fmt_err(
                "abc.encode.capabilities",
                format!("capability `{}` exceeds wire width", c),
            ));
        }
        put_u16(&mut w, b.len() as u16);
        w.push(b);
    }

    // Types.
    for t in &m.types {
        match t {
            MirType::I32 => put_u8(&mut w, TY_I32),
            MirType::Bool => put_u8(&mut w, TY_BOOL),
            MirType::Fn(sig) => {
                put_u8(&mut w, TY_FN);
                put_u32(&mut w, sig.params.len() as u32);
                for p in &sig.params {
                    put_u32(&mut w, *p);
                }
                put_u32(&mut w, sig.result);
            }
        }
    }
    // Consts.
    for c in &m.consts {
        match c {
            MirConst::I32(v) => {
                put_u8(&mut w, 0);
                put_i32(&mut w, *v);
            }
            MirConst::Bool(b) => {
                put_u8(&mut w, 1);
                put_u8(&mut w, *b as u8);
            }
        }
    }
    // Imports.
    for imp in &m.imports {
        let sym = imp.symbol.as_bytes();
        if sym.len() > u16::MAX as usize {
            return Err(AbcDecodeError::fmt_err(
                "abc.encode.symbol-length",
                format!("import symbol `{}` exceeds wire width", imp.symbol),
            ));
        }
        put_u16(&mut w, sym.len() as u16);
        w.push(sym);
        put_u8(
            &mut w,
            match imp.effect {
                Effect::Observable => 0,
                Effect::Pure => 1,
            },
        );
        put_u32(&mut w, imp.sig);
    }
    // Functions.
    for f in &m.funcs {
        let name = f.name.as_bytes();
        if name.len() > u16::MAX as usize {
            return Err(AbcDecodeError::fmt_err(
                "abc.encode.name-length",
                format!("function name `{}` exceeds wire width", f.name),
            ));
        }
        put_u32(&mut w, f.sig);
        put_u16(&mut w, name.len() as u16);
        w.push(name);
        put_u32(&mut w, f.slots.len() as u32);
        for s in &f.slots {
            put_u8(
                &mut w,
                match s.kind {
                    SlotKind::Param => 0,
                    SlotKind::Local => 1,
                    SlotKind::Temp => 2,
                },
            );
            put_u32(&mut w, s.ty);
        }
        put_u32(&mut w, f.entry);
        put_u32(&mut w, f.blocks.len() as u32);
        for b in &f.blocks {
            put_u32(&mut w, b.instrs.len() as u32);
            for instr in &b.instrs {
                match instr {
                    MirInstr::Const { dst, value } => {
                        put_u8(&mut w, OP_CONST);
                        put_u32(&mut w, *dst);
                        put_u32(&mut w, *value);
                    }
                    MirInstr::Copy { dst, src } => {
                        put_u8(&mut w, OP_COPY);
                        put_u32(&mut w, *dst);
                        put_u32(&mut w, *src);
                    }
                    MirInstr::Bin { op, dst, lhs, rhs } => {
                        put_u8(&mut w, OP_BIN);
                        put_u8(&mut w, bin_op_code(*op));
                        put_u32(&mut w, *dst);
                        put_u32(&mut w, *lhs);
                        put_u32(&mut w, *rhs);
                    }
                    MirInstr::Un { op, dst, operand } => {
                        put_u8(&mut w, OP_UN);
                        put_u8(&mut w, un_op_code(*op));
                        put_u32(&mut w, *dst);
                        put_u32(&mut w, *operand);
                    }
                    MirInstr::Call { callee, dst, args } => {
                        if args.len() > u8::MAX as usize {
                            return Err(AbcDecodeError::fmt_err(
                                "abc.encode.arity",
                                format!("call with {} args exceeds wire width", args.len()),
                            ));
                        }
                        put_u8(&mut w, OP_CALL);
                        match callee {
                            CalleeRef::Func(idx) => {
                                put_u8(&mut w, 0);
                                put_u32(&mut w, *idx);
                            }
                            CalleeRef::Import(idx) => {
                                put_u8(&mut w, 1);
                                put_u32(&mut w, *idx);
                            }
                        }
                        put_u32(&mut w, *dst);
                        put_u8(&mut w, args.len() as u8);
                        for a in args {
                            put_u32(&mut w, *a);
                        }
                    }
                    MirInstr::Trace { event } => {
                        put_u8(&mut w, OP_TRACE);
                        put_u8(
                            &mut w,
                            match event {
                                TraceEvent::MarkA => 1,
                                TraceEvent::MarkB => 2,
                            },
                        );
                    }
                }
            }
            match &b.term {
                Terminator::Jump { target } => {
                    put_u8(&mut w, TERM_JUMP);
                    put_u32(&mut w, *target);
                }
                Terminator::Branch { cond, then, else_ } => {
                    put_u8(&mut w, TERM_BRANCH);
                    put_u32(&mut w, *cond);
                    put_u32(&mut w, *then);
                    put_u32(&mut w, *else_);
                }
                Terminator::Return { value } => {
                    put_u8(&mut w, TERM_RETURN);
                    put_u32(&mut w, *value);
                }
            }
        }
    }
    // Source-map sidecar (diagnostics only; logical identity, no paths).
    if has_sidecar {
        let mut sc: Vec<u8> = Vec::new();
        for f in &m.funcs {
            sc.extend_from_slice(&(f.source_snapshot).to_le_bytes());
            sc.extend_from_slice(&(f.spans.len() as u32).to_le_bytes());
            for (instr, sp) in &f.spans {
                sc.extend_from_slice(&instr.to_le_bytes());
                sc.extend_from_slice(&sp.func.to_le_bytes());
                sc.extend_from_slice(&sp.start.to_le_bytes());
                sc.extend_from_slice(&sp.end.to_le_bytes());
            }
        }
        if sc.len() > u32::MAX as usize {
            return Err(AbcDecodeError::fmt_err(
                "abc.encode.sidecar-length",
                "source sidecar exceeds wire width".to_string(),
            ));
        }
        put_u32(&mut w, sc.len() as u32);
        w.push(&sc);
    }
    // Trailer checksum.
    let mut out = w.finish();
    let mut trailer = out.checksum.to_le_bytes().to_vec();
    // Trailer rides as one final page (≤ 8 bytes, page budget trivially ok).
    out.total_len += trailer.len();
    out.pages.push(std::mem::take(&mut trailer));
    Ok(out)
}

// ---------------------------------------------------------------------------
// Decode
// ---------------------------------------------------------------------------

/// Decoded ABC: the reconstructed (unchecked) MIR module plus identity.
#[derive(Debug)]
pub struct DecodedAbc {
    pub version: u16,
    pub profile: u16,
    pub module: MirModule,
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
    steps: u64,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], AbcDecodeError> {
        self.steps += 1;
        if self.steps > crate::contract::MAX_VERIFY_WORK_STEPS {
            return Err(AbcDecodeError::Resource {
                kind: ResourceKind::VerifySteps,
                message: "decode exceeded the work-step budget".to_string(),
            });
        }
        let end = self.pos.checked_add(n).ok_or_else(|| {
            AbcDecodeError::fmt_err(
                "abc.decode.length-overflow",
                "length arithmetic overflow".to_string(),
            )
        })?;
        if end > self.data.len() {
            return Err(AbcDecodeError::fmt_err(
                "abc.decode.truncated",
                format!(
                    "need {} bytes at offset {}, stream has {}",
                    n,
                    self.pos,
                    self.data.len()
                ),
            ));
        }
        let s = &self.data[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8, AbcDecodeError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, AbcDecodeError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Result<u32, AbcDecodeError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, AbcDecodeError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
    fn i32(&mut self) -> Result<i32, AbcDecodeError> {
        Ok(self.u32()? as i32)
    }
}

/// Decode flat ABC bytes (pages already concatenated) into an unchecked
/// module. Structure-only: identity checks, checked arithmetic, quotas,
/// unknown version/opcode rejection, bool legality, index bounds.
pub fn decode_abc(bytes: &[u8]) -> Result<DecodedAbc, AbcDecodeError> {
    if bytes.len() > crate::contract::MAX_INPUT_BYTES {
        return Err(AbcDecodeError::Resource {
            kind: ResourceKind::InputTooLarge,
            message: format!(
                "ABC input {} exceeds {}",
                bytes.len(),
                crate::contract::MAX_INPUT_BYTES
            ),
        });
    }
    if bytes.len() < 4 + 2 + 2 + 2 + 2 + 5 * 4 + 8 {
        return Err(AbcDecodeError::fmt_err(
            "abc.decode.truncated",
            "header shorter than its fixed layout".to_string(),
        ));
    }
    // Trailer checksum over everything before the last 8 bytes.
    let body_len = bytes.len() - 8;
    let want = u64::from_le_bytes(bytes[body_len..].try_into().expect("8 bytes"));
    let got = fnv1a64(&bytes[..body_len]);
    if want != got {
        return Err(AbcDecodeError::fmt_err(
            "abc.decode.checksum",
            format!(
                "trailer checksum mismatch: stored {:016x}, computed {:016x}",
                want, got
            ),
        ));
    }
    let mut c = Cursor {
        data: &bytes[..body_len],
        pos: 0,
        steps: 0,
    };
    let magic = c.take(4)?;
    if magic != ABC_MAGIC {
        return Err(AbcDecodeError::fmt_err(
            "abc.decode.magic",
            "not an AVAB stream".to_string(),
        ));
    }
    let version = c.u16()?;
    if version != ABC_VERSION {
        return Err(AbcDecodeError::fmt_err(
            "abc.decode.version",
            format!(
                "unknown ABC version {} (this build accepts {})",
                version, ABC_VERSION
            ),
        ));
    }
    let profile = c.u16()?;
    if profile != ABC_PROFILE_CORE_I32 {
        return Err(AbcDecodeError::fmt_err(
            "abc.decode.profile",
            format!(
                "unknown ABC profile id {} (this build accepts {})",
                profile, ABC_PROFILE_CORE_I32
            ),
        ));
    }
    let flags = c.u16()?;
    if flags & !(FLAG_SOURCE_SIDECAR | FLAG_IS_UNIT) != 0 {
        return Err(AbcDecodeError::fmt_err(
            "abc.decode.flags",
            format!("unknown flag bits {:x}", flags),
        ));
    }
    let reserved = c.u16()?;
    if reserved != 0 {
        return Err(AbcDecodeError::fmt_err(
            "abc.decode.reserved",
            "reserved field must be zero".to_string(),
        ));
    }
    let n_types = c.u32()? as usize;
    let n_consts = c.u32()? as usize;
    let n_imports = c.u32()? as usize;
    let n_funcs = c.u32()? as usize;
    let entry = c.u32()?;
    let n_caps = c.u16()? as usize;
    let mut capabilities = Vec::with_capacity(n_caps.min(16));
    for _ in 0..n_caps {
        let n = c.u16()? as usize;
        let cb = c.take(n)?.to_vec();
        capabilities.push(String::from_utf8(cb).map_err(|_| {
            AbcDecodeError::fmt_err(
                "abc.decode.capability-utf8",
                "capability is not UTF-8".to_string(),
            )
        })?);
    }
    // Quota: counts alone are bounded by the frozen table budget (checked
    // again post-assembly by the verifier; checked HERE before allocation).
    if n_types
        .saturating_add(n_consts)
        .saturating_add(n_imports)
        .saturating_add(n_funcs)
        > crate::contract::MAX_TABLE_ENTRIES
    {
        return Err(AbcDecodeError::Resource {
            kind: ResourceKind::TableEntries,
            message: format!(
                "header table counts exceed {}",
                crate::contract::MAX_TABLE_ENTRIES
            ),
        });
    }
    // F798-R6-05: cumulative decode-memory guard, checked BEFORE each
    // table's allocation. The whole-module heap estimate accumulates as
    // tables decode (module tables, per-function slot/block/instruction
    // tables, strings, spans); crossing the frozen decode partition budget
    // rejects as `av.resource.decode-memory` — decode itself must never
    // finish (let alone hand out) an over-budget object. Per-element
    // estimates mirror `MirModule::heap_estimate` semantics.
    let mut decoded_est: usize = 0usize;
    {
        let est_vec_overhead = std::mem::size_of::<usize>() * 3;
        decoded_est += std::mem::size_of::<MirModule>()
            + std::mem::size_of::<String>() * 3
            + capabilities
                .iter()
                .map(|c| c.len() + std::mem::size_of::<String>())
                .sum::<usize>()
            + est_vec_overhead * 5
            + n_types * (std::mem::size_of::<MirType>() + 4)
            + n_consts * std::mem::size_of::<MirConst>()
            + n_imports
                .saturating_mul(std::mem::size_of::<MirImport>() + std::mem::size_of::<String>())
            + n_funcs.saturating_mul(
                std::mem::size_of::<MirFunction>()
                    + std::mem::size_of::<String>()
                    + est_vec_overhead,
            );
    }
    if decoded_est > crate::contract::MAX_DECODED_MODULE_BYTES {
        return Err(AbcDecodeError::Resource {
            kind: ResourceKind::DecodeMemory,
            message: format!(
                "header alone estimates {} decoded bytes, above the frozen {} budget",
                decoded_est,
                crate::contract::MAX_DECODED_MODULE_BYTES
            ),
        });
    }
    macro_rules! decode_mem {
        ($add:expr, $what:expr) => {{
            decoded_est = decoded_est.saturating_add($add);
            if decoded_est > crate::contract::MAX_DECODED_MODULE_BYTES {
                return Err(AbcDecodeError::Resource {
                    kind: ResourceKind::DecodeMemory,
                    message: format!(
                        "decoding {} pushes the decoded-heap estimate to {} bytes, above the frozen {} budget",
                        $what,
                        decoded_est,
                        crate::contract::MAX_DECODED_MODULE_BYTES
                    ),
                });
            }
        }};
    }

    let mut module = MirModule {
        profile: crate::contract::AV_PROFILE.to_string(),
        owner: OwnerScope {
            schema: String::new(),
            module: String::new(),
        },
        // F798-R11-02: the kind is read from the WIRE flag, never
        // inferred from the function count — a single-function whole
        // module (direct self/mutual calls allowed) round-trips as a
        // module, and a split unit stays a unit.
        is_unit: flags & FLAG_IS_UNIT != 0,
        types: Vec::with_capacity(n_types.min(1024)),
        consts: Vec::with_capacity(n_consts.min(1024)),
        imports: Vec::with_capacity(n_imports.min(64)),
        funcs: Vec::with_capacity(n_funcs.min(1024)),
        entry,
        capabilities,
    };
    // NOTE: owner identity is envelope-level (probe cache); the wire carries
    // structural content. The probe re-stamps owner before verify.

    // Types.
    for _ in 0..n_types {
        let tag = c.u8()?;
        match tag {
            TY_I32 => module.types.push(MirType::I32),
            TY_BOOL => module.types.push(MirType::Bool),
            TY_FN => {
                let n = c.u32()? as usize;
                if n > u8::MAX as usize {
                    return Err(AbcDecodeError::fmt_err(
                        "abc.decode.sig-arity",
                        format!("signature arity {} exceeds wire width", n),
                    ));
                }
                decode_mem!(n * 4 + 24, "a signature parameter table");
                let mut params = Vec::with_capacity(n);
                for _ in 0..n {
                    params.push(c.u32()?);
                }
                let result = c.u32()?;
                module.types.push(MirType::Fn(FnSig { params, result }));
            }
            other => {
                return Err(AbcDecodeError::fmt_err(
                    "abc.decode.type-tag",
                    format!("unknown type tag {}", other),
                ));
            }
        }
    }
    // Consts.
    for _ in 0..n_consts {
        let tag = c.u8()?;
        match tag {
            0 => module.consts.push(MirConst::I32(c.i32()?)),
            1 => {
                let b = c.u8()?;
                if b > 1 {
                    return Err(AbcDecodeError::fmt_err(
                        "abc.decode.bool",
                        format!("boolean constant byte {} is not 0/1", b),
                    ));
                }
                module.consts.push(MirConst::Bool(b == 1));
            }
            other => {
                return Err(AbcDecodeError::fmt_err(
                    "abc.decode.const-tag",
                    format!("unknown const tag {}", other),
                ));
            }
        }
    }
    // Imports.
    for _ in 0..n_imports {
        let n = c.u16()? as usize;
        decode_mem!(n + 24, "an import symbol");
        let sym_bytes = c.take(n)?.to_vec();
        let symbol = String::from_utf8(sym_bytes).map_err(|_| {
            AbcDecodeError::fmt_err(
                "abc.decode.symbol-utf8",
                "import symbol is not UTF-8".to_string(),
            )
        })?;
        let eff = c.u8()?;
        let effect = match eff {
            0 => Effect::Observable,
            1 => Effect::Pure,
            other => {
                return Err(AbcDecodeError::fmt_err(
                    "abc.decode.effect",
                    format!("unknown effect byte {}", other),
                ));
            }
        };
        let sig = c.u32()?;
        module.imports.push(MirImport {
            symbol,
            effect,
            sig,
        });
    }
    // Functions.
    let mut sidecar_needed = flags & FLAG_SOURCE_SIDECAR != 0;
    let mut raw_funcs: Vec<(TypeIdx, String, Vec<MirSlot>, BlockId, Vec<MirBlock>)> =
        Vec::with_capacity(n_funcs);
    // Per-TABLE-FAMILY probe (plan §3 r2): module tables were checked in
    // the header; each FUNCTION's slot/block/instruction table is checked
    // at its own boundary — never accumulated across the whole module.
    let mut fn_probe: usize;
    for _ in 0..n_funcs {
        let sig = c.u32()?;
        let nlen = c.u16()? as usize;
        let name_bytes = c.take(nlen)?.to_vec();
        let name = String::from_utf8(name_bytes).map_err(|_| {
            AbcDecodeError::fmt_err(
                "abc.decode.name-utf8",
                "function name is not UTF-8".to_string(),
            )
        })?;
        decode_mem!(name.len() + 24, "a function name");
        let nslots = c.u32()? as usize;
        fn_probe = nslots;
        decode_mem!(nslots * std::mem::size_of::<MirSlot>() + 24, "a slot table");
        let mut slots = Vec::with_capacity(nslots.min(4096));
        for _ in 0..nslots {
            let k = c.u8()?;
            let kind = match k {
                0 => SlotKind::Param,
                1 => SlotKind::Local,
                2 => SlotKind::Temp,
                other => {
                    return Err(AbcDecodeError::fmt_err(
                        "abc.decode.slot-kind",
                        format!("unknown slot kind {}", other),
                    ));
                }
            };
            let ty = c.u32()?;
            slots.push(MirSlot {
                kind,
                ty,
                name: None,
            });
        }
        let entry_block = c.u32()?;
        let nblocks = c.u32()? as usize;
        fn_probe += nblocks;
        decode_mem!(
            nblocks * (std::mem::size_of::<MirBlock>() + 24),
            "a block table"
        );
        let mut blocks = Vec::with_capacity(nblocks.min(4096));
        for _ in 0..nblocks {
            let ninstrs = c.u32()? as usize;
            fn_probe += ninstrs;
            decode_mem!(
                ninstrs * std::mem::size_of::<MirInstr>() + 24,
                "an instruction table"
            );
            if fn_probe > crate::contract::MAX_TABLE_ENTRIES {
                return Err(AbcDecodeError::Resource {
                    kind: ResourceKind::TableEntries,
                    message: format!(
                        "function `{}` table entries {} exceed the frozen budget {}",
                        name,
                        fn_probe,
                        crate::contract::MAX_TABLE_ENTRIES
                    ),
                });
            }
            let mut instrs = Vec::with_capacity(ninstrs.min(4096));
            for _ in 0..ninstrs {
                let op = c.u8()?;
                let instr = match op {
                    OP_CONST => {
                        let dst = c.u32()?;
                        let value = c.u32()?;
                        MirInstr::Const { dst, value }
                    }
                    OP_COPY => {
                        let dst = c.u32()?;
                        let src = c.u32()?;
                        MirInstr::Copy { dst, src }
                    }
                    OP_BIN => {
                        let sub = c.u8()?;
                        let op = match sub {
                            BIN_ADD => BinOp::AddI32,
                            BIN_SUB => BinOp::SubI32,
                            BIN_MUL => BinOp::MulI32,
                            BIN_DIV => BinOp::DivI32,
                            BIN_REM => BinOp::RemI32,
                            BIN_LT => BinOp::LtI32,
                            BIN_LE => BinOp::LeI32,
                            BIN_GT => BinOp::GtI32,
                            BIN_GE => BinOp::GeI32,
                            BIN_EQ => BinOp::EqI32,
                            BIN_NE => BinOp::NeI32,
                            other => {
                                return Err(AbcDecodeError::fmt_err(
                                    "abc.decode.bin-subop",
                                    format!("unknown binary sub-opcode {}", other),
                                ));
                            }
                        };
                        let dst = c.u32()?;
                        let lhs = c.u32()?;
                        let rhs = c.u32()?;
                        MirInstr::Bin { op, dst, lhs, rhs }
                    }
                    OP_UN => {
                        let sub = c.u8()?;
                        let op = match sub {
                            UN_NEG => UnOp::NegI32,
                            UN_NOT => UnOp::NotBool,
                            other => {
                                return Err(AbcDecodeError::fmt_err(
                                    "abc.decode.un-subop",
                                    format!("unknown unary sub-opcode {}", other),
                                ));
                            }
                        };
                        let dst = c.u32()?;
                        let operand = c.u32()?;
                        MirInstr::Un { op, dst, operand }
                    }
                    OP_CALL => {
                        let ck = c.u8()?;
                        let callee = match ck {
                            0 => CalleeRef::Func(c.u32()?),
                            1 => CalleeRef::Import(c.u32()?),
                            other => {
                                return Err(AbcDecodeError::fmt_err(
                                    "abc.decode.callee-kind",
                                    format!("unknown callee kind {}", other),
                                ));
                            }
                        };
                        let dst = c.u32()?;
                        let nargs = c.u8()? as usize;
                        let mut args = Vec::with_capacity(nargs);
                        for _ in 0..nargs {
                            args.push(c.u32()?);
                        }
                        MirInstr::Call { callee, dst, args }
                    }
                    OP_TRACE => {
                        let ev = c.u8()?;
                        let event = match ev {
                            1 => TraceEvent::MarkA,
                            2 => TraceEvent::MarkB,
                            other => {
                                return Err(AbcDecodeError::fmt_err(
                                    "abc.decode.trace-event",
                                    format!("unknown trace event {}", other),
                                ));
                            }
                        };
                        MirInstr::Trace { event }
                    }
                    other => {
                        return Err(AbcDecodeError::fmt_err(
                            "abc.decode.opcode",
                            format!("unknown opcode {:#04x}", other),
                        ));
                    }
                };
                instrs.push(instr);
            }
            let t = c.u8()?;
            let term = match t {
                TERM_JUMP => Terminator::Jump { target: c.u32()? },
                TERM_BRANCH => {
                    let cond = c.u32()?;
                    let then = c.u32()?;
                    let else_ = c.u32()?;
                    Terminator::Branch { cond, then, else_ }
                }
                TERM_RETURN => Terminator::Return { value: c.u32()? },
                other => {
                    return Err(AbcDecodeError::fmt_err(
                        "abc.decode.term-opcode",
                        format!("unknown terminator opcode {:#04x}", other),
                    ));
                }
            };
            blocks.push(MirBlock { instrs, term });
        }
        raw_funcs.push((sig, name, slots, entry_block, blocks));
    }
    // Sidecar.
    let mut snapshots: Vec<u64> = Vec::with_capacity(n_funcs);
    let mut span_tables: Vec<Vec<(u32, crate::contract::SpanInfo)>> = Vec::with_capacity(n_funcs);
    if sidecar_needed {
        let sc_len = c.u32()? as usize;
        let sc = c.take(sc_len)?;
        let mut sc_cur = Cursor {
            data: sc,
            pos: 0,
            steps: 0,
        };
        for _ in 0..n_funcs {
            let snap = sc_cur.u64()?;
            let nspans = sc_cur.u32()? as usize;
            decode_mem!(
                nspans * std::mem::size_of::<(u32, crate::contract::SpanInfo)>() + 24,
                "a span table"
            );
            let mut spans = Vec::with_capacity(nspans.min(4096));
            for _ in 0..nspans {
                let instr = sc_cur.u32()?;
                let func = sc_cur.u32()?;
                let start = sc_cur.u32()?;
                let end = sc_cur.u32()?;
                spans.push((instr, crate::contract::SpanInfo { func, start, end }));
            }
            snapshots.push(snap);
            span_tables.push(spans);
        }
    } else {
        for _ in 0..n_funcs {
            snapshots.push(0);
            span_tables.push(Vec::new());
        }
    }
    if c.pos != c.data.len() {
        return Err(AbcDecodeError::fmt_err(
            "abc.decode.trailing-bytes",
            format!(
                "{} trailing bytes after the declared layout",
                c.data.len() - c.pos
            ),
        ));
    }
    sidecar_needed = false;
    let _ = sidecar_needed;

    let owner = OwnerScope {
        schema: crate::contract::ABC_SCHEMA.to_string(),
        module: String::new(),
    };
    module.owner = owner;
    for (fi, (sig, name, slots, entry_block, blocks)) in raw_funcs.into_iter().enumerate() {
        module.funcs.push(MirFunction {
            owner: module.owner.clone(),
            name,
            sig,
            slots,
            blocks,
            entry: entry_block,
            source_snapshot: snapshots[fi],
            spans: span_tables[fi].clone(),
        });
    }
    Ok(DecodedAbc {
        version,
        profile,
        module,
    })
}

#[cfg(test)]
mod decode_budget_tests {
    use super::*;
    use crate::contract::MIR_SCHEMA;

    fn owner() -> OwnerScope {
        OwnerScope {
            schema: MIR_SCHEMA.to_string(),
            module: "m".to_string(),
        }
    }

    /// F798-R6-05 replay: a compact wire (≪ 4 MiB) of 60 functions × 6000
    /// empty blocks decodes to >16 MiB of structure — decode itself must
    /// refuse (cumulative pre-allocation guard), never hand out the
    /// over-budget object.
    #[test]
    fn abc_decode_expansion_rejected_before_allocation() {
        let mut m = MirModule::new(owner());
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        for i in 0..60 {
            m.funcs.push(MirFunction {
                owner: owner(),
                name: format!("f{i}"),
                sig,
                slots: vec![],
                blocks: vec![
                    MirBlock {
                        instrs: vec![],
                        term: Terminator::Jump { target: 0 },
                    };
                    6000
                ],
                entry: 0,
                source_snapshot: 0,
                spans: vec![],
            });
        }
        m.entry = 0;
        let wire = encode_abc(&m).expect("encode");
        let flat = wire.to_flat();
        assert!(
            flat.len() < 4 * 1024 * 1024,
            "the adversarial wire stays under the input cap ({} bytes)",
            flat.len()
        );
        let err = decode_abc(&flat).expect_err("decode must refuse the over-budget object");
        assert_eq!(
            err.code(),
            "av.resource.decode-memory",
            "got {:?}",
            err.code()
        );
    }
}
