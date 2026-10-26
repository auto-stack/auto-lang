//! PLAN-824 portable value freezing (T-10 module subset,
//! contract-decision.md §4–§5/§7).
//!
//! Same-source note: NOT declared in core's `lib.rs`; compiled by the
//! `parallel/824` harness via `#[path]` (see dynamic.rs).
//!
//! Portable contract (frozen):
//! - The wire is a versioned DFS-preorder tree of the whitelisted value
//!   with NOMINAL record headers (owner/name/field names in declaration
//!   order). It never persists process addresses, Str allocation ids or
//!   arena hooks, the positional `RecordValue::ty` index, frame/borrow
//!   tokens or any credential (§5.1.4).
//! - `freeze` enforces the owned-tree boundary: every `Rc` payload
//!   (record or Str buffer) may be visited exactly once — a shared child
//!   (including an acyclic DAG), a cycle, or a repeat reference is
//!   rejected with `portable.unsupported(shared-graph)` and the SOURCE IS
//!   NOT CONSUMED (`freeze` borrows; V-04/V-12/V-14d).
//! - `unfreeze` re-verifies in the frozen order: bounds/budgets BEFORE
//!   semantics (§5.1.5), structure (tags/domains/UTF-8/duplicate field
//!   names/truncation/trailing), current nominal dependencies through the
//!   [`RestoreContext`] (`portable.unknown-type` / `portable.layout-
//!   mismatch`), then the FNV-1a trailer. A good checksum never bypasses
//!   a structural check and vice versa.
//! - Restored Str handles are detached (id 0, no arena): module-level
//!   restore has no ledger — in-execution restore mints arena handles at
//!   T-02+ (contract §5). Half-built graphs on failure are reclaimed by
//!   ownership rollback (nothing partially installed, input bytes not
//!   consumed); the arena-observable free-event oracle belongs to the
//!   T-02+ execution integration.

use crate::contract::{fnv1a64, RecordValue, StrHandle, Value, MAX_RECORD_FIELDS};
use crate::dynamic::{
    byte_cost_cell, byte_cost_record_extra, DynField, DynType, DynValue, Traversal,
    TraversalOptions, TraversalStop, MAX_DYN_BYTES, MAX_DYN_NODES,
};
use std::collections::HashSet;
use std::rc::Rc;

/// Portable schema identity (contract §4).
pub const PORTABLE_SCHEMA: &str = "auto.portable.av-dynamic/1";
/// Wire magic family of the portable frozen payload (`AVPF`).
pub const PORTABLE_MAGIC: [u8; 4] = *b"AVPF";
/// Wire version (v1).
pub const PORTABLE_VERSION: u16 = 1;
/// Payload kind: a DynValue tree (the only kind of v1).
pub const PORTABLE_KIND_DYN_VALUE: u8 = 1;

const HEADER_LEN: usize = 16;
const TRAILER_LEN: usize = 8;

const TAG_I32: u8 = 0;
const TAG_BOOL: u8 = 1;
const TAG_I64: u8 = 2;
const TAG_U64: u8 = 3;
const TAG_F32: u8 = 4;
const TAG_F64: u8 = 5;
const TAG_CHAR: u8 = 6;
const TAG_STR: u8 = 7;
const TAG_RECORD: u8 = 8;

// ---------------------------------------------------------------------------
// Errors (§7 分态: unsupported / invalid / admission / budget / cancelled)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortableError {
    /// Category rejection at the portable boundary (shared graph / cycle).
    Unsupported {
        category: &'static str,
    },
    /// Bad wire: magic / version / kind / flags / reserved / node-count /
    /// tag / bool-domain / char-domain / utf8 / duplicate-field /
    /// name-length / truncated / trailing / checksum.
    Invalid {
        reason: &'static str,
    },
    /// Decode-time re-verification against the CURRENT nominal
    /// dependencies (§5.1.5 — layout/revision changes never reuse stale
    /// indices).
    Admission {
        code: &'static str,
    },
    Budget {
        code: &'static str,
    },
    Cancelled,
}

impl PortableError {
    pub fn code(&self) -> &'static str {
        match self {
            PortableError::Unsupported { .. } => "portable.unsupported",
            PortableError::Invalid { .. } => "portable.invalid",
            PortableError::Admission { code } => code,
            PortableError::Budget { code } => code,
            PortableError::Cancelled => "av.dyn.cancelled",
        }
    }
}

impl From<TraversalStop> for PortableError {
    fn from(stop: TraversalStop) -> Self {
        match stop {
            TraversalStop::Cancelled => PortableError::Cancelled,
            other => PortableError::Budget { code: other.code() },
        }
    }
}

// ---------------------------------------------------------------------------
// Restore context (current-dependency admission, §5)
// ---------------------------------------------------------------------------

/// One current nominal record binding: the declaration this decode domain
/// accepts (owner + name + field names in declaration order) and the
/// positional type-table index restored `RecordValue`s carry.
pub struct RecordBinding<'a> {
    pub owner: &'a str,
    pub name: &'a str,
    pub fields: &'a [&'a str],
    pub ty_index: u32,
}

/// The read-only current-dependency set of one decode domain. No
/// credentials: restoring never un-freezes trust, it re-verifies names.
pub struct RestoreContext<'a> {
    pub bindings: &'a [RecordBinding<'a>],
}

impl<'a> RestoreContext<'a> {
    pub fn new(bindings: &'a [RecordBinding<'a>]) -> Self {
        RestoreContext { bindings }
    }

    fn lookup(&self, owner: &str, name: &str) -> Option<&'a RecordBinding<'a>> {
        self.bindings
            .iter()
            .find(|b| b.owner == owner && b.name == name)
    }
}

// ---------------------------------------------------------------------------
// freeze
// ---------------------------------------------------------------------------

/// Freeze a DynValue into portable bytes (deterministic: equal content
/// freezes to equal bytes; V-14a byte-equivalence oracle).
pub fn freeze(v: &DynValue, opts: &TraversalOptions) -> Result<Vec<u8>, PortableError> {
    let mut t = Traversal::new(opts, MAX_DYN_BYTES);
    let mut out: Vec<u8> = Vec::with_capacity(64);
    t.charge_bytes(HEADER_LEN + TRAILER_LEN)?;
    out.extend_from_slice(&PORTABLE_MAGIC);
    out.extend_from_slice(&PORTABLE_VERSION.to_le_bytes());
    out.push(PORTABLE_KIND_DYN_VALUE);
    out.push(0); // flags (only 0 exists in v1)
    out.extend_from_slice(&0u32.to_le_bytes()); // node_count placeholder @8..12
    out.extend_from_slice(&0u32.to_le_bytes()); // reserved

    // Unique-in-edge ledger: every Rc payload may be emitted exactly once.
    let mut visited: HashSet<usize> = HashSet::new();
    // DFS preorder work stack (children pushed in REVERSE so declaration
    // order pops first).
    let mut stack: Vec<(&Value, &DynType, u32)> = vec![(v.as_value(), v.ty(), 1)];

    while let Some((val, ty, depth)) = stack.pop() {
        t.enter_node(depth)?;
        match val {
            Value::I32(x) => {
                t.charge_bytes(5)?;
                out.push(TAG_I32);
                out.extend_from_slice(&x.to_le_bytes());
            }
            Value::Bool(b) => {
                t.charge_bytes(2)?;
                out.push(TAG_BOOL);
                out.push(*b as u8);
            }
            Value::I64(x) => {
                t.charge_bytes(9)?;
                out.push(TAG_I64);
                out.extend_from_slice(&x.to_le_bytes());
            }
            Value::U64(x) => {
                t.charge_bytes(9)?;
                out.push(TAG_U64);
                out.extend_from_slice(&x.to_le_bytes());
            }
            Value::F32(x) => {
                t.charge_bytes(5)?;
                out.push(TAG_F32);
                out.extend_from_slice(&x.to_bits().to_le_bytes());
            }
            Value::F64(x) => {
                t.charge_bytes(9)?;
                out.push(TAG_F64);
                out.extend_from_slice(&x.to_bits().to_le_bytes());
            }
            Value::Char(c) => {
                t.charge_bytes(5)?;
                out.push(TAG_CHAR);
                out.extend_from_slice(&(*c as u32).to_le_bytes());
            }
            Value::Str(h) => {
                if !visited.insert(Rc::as_ptr(&h.rc) as usize) {
                    return Err(PortableError::Unsupported {
                        category: "shared-graph",
                    });
                }
                let bytes = h.bytes();
                t.charge_bytes(5 + bytes.len())?; // checked BEFORE the payload append
                out.push(TAG_STR);
                out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                out.extend_from_slice(bytes);
            }
            Value::Record(rec) => {
                let (owner, name, fields) = match ty {
                    DynType::Record {
                        owner,
                        name,
                        fields,
                    } => (owner, name, fields),
                    _ => {
                        return Err(PortableError::Invalid { reason: "arity" });
                    }
                };
                if rec.fields.len() != fields.len() {
                    // Unreachable for a valid DynValue (pack invariant);
                    // defense in depth.
                    return Err(PortableError::Invalid { reason: "arity" });
                }
                if !visited.insert(Rc::as_ptr(rec) as usize) {
                    return Err(PortableError::Unsupported {
                        category: "shared-graph",
                    });
                }
                t.charge_steps(2 + fields.len() as u64)?;
                let mut node_bytes = Vec::with_capacity(7 + owner.len() + name.len());
                node_bytes.push(TAG_RECORD);
                push_name(&mut node_bytes, owner)?;
                push_name(&mut node_bytes, name)?;
                node_bytes.extend_from_slice(&(fields.len() as u16).to_le_bytes());
                for f in fields {
                    push_name(&mut node_bytes, &f.name)?;
                }
                t.charge_bytes(node_bytes.len())?;
                out.extend_from_slice(&node_bytes);
                for (fv, ft) in rec.fields.iter().zip(fields.iter()).rev() {
                    stack.push((fv, &ft.ty, depth + 1));
                }
            }
            // The pack whitelist makes these unreachable; kept explicit so a
            // future category cannot silently ride the wire.
            Value::StrView(_) | Value::MutRef(_) => {
                return Err(PortableError::Unsupported { category: "borrow" });
            }
        }
    }

    // Patch the real node count, then the checksum trailer.
    let nodes = t_nodes(&t);
    out[8..12].copy_from_slice(&nodes.to_le_bytes());
    let sum = fnv1a64(&out);
    out.extend_from_slice(&sum.to_le_bytes());
    Ok(out)
}

fn t_nodes(t: &Traversal) -> u32 {
    t.node_count()
}

fn push_name(out: &mut Vec<u8>, s: &str) -> Result<(), PortableError> {
    if s.len() > u16::MAX as usize {
        return Err(PortableError::Invalid {
            reason: "name-length",
        });
    }
    out.extend_from_slice(&(s.len() as u16).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

// ---------------------------------------------------------------------------
// unfreeze
// ---------------------------------------------------------------------------

/// Restore a DynValue from portable bytes, re-verifying bounds/budgets,
/// structure, current nominal dependencies and the checksum in that order.
/// Failing consumes nothing: `bytes` is borrowed, partially built graphs
/// are reclaimed by ownership rollback.
pub fn unfreeze(
    bytes: &[u8],
    ctx: &RestoreContext,
    opts: &TraversalOptions,
) -> Result<DynValue, PortableError> {
    if bytes.len() > MAX_DYN_BYTES {
        return Err(PortableError::Budget {
            code: "av.resource.dyn-bytes",
        });
    }
    if bytes.len() < HEADER_LEN + TRAILER_LEN {
        return Err(PortableError::Invalid {
            reason: "truncated",
        });
    }
    if bytes[0..4] != PORTABLE_MAGIC {
        return Err(PortableError::Invalid { reason: "magic" });
    }
    if u16::from_le_bytes([bytes[4], bytes[5]]) != PORTABLE_VERSION {
        return Err(PortableError::Invalid { reason: "version" });
    }
    if bytes[6] != PORTABLE_KIND_DYN_VALUE {
        return Err(PortableError::Invalid { reason: "kind" });
    }
    if bytes[7] != 0 {
        return Err(PortableError::Invalid { reason: "flags" });
    }
    let node_count = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if node_count > MAX_DYN_NODES {
        return Err(PortableError::Budget {
            code: "av.resource.dyn-nodes",
        });
    }
    if u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) != 0 {
        return Err(PortableError::Invalid { reason: "reserved" });
    }

    let body_len = bytes.len() - TRAILER_LEN;
    let mut t = Traversal::new(opts, MAX_DYN_BYTES);
    let mut r = Reader {
        b: &bytes[..body_len],
        pos: HEADER_LEN,
    };

    struct Frame {
        owner: String,
        name: String,
        field_names: Vec<String>,
        ty_index: u32,
        vals: Vec<Value>,
        tys: Vec<DynType>,
    }
    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<(Value, DynType)> = None;
    let mut parsed_nodes: u32 = 0;

    loop {
        if r.pos == body_len {
            if stack.is_empty() {
                break;
            }
            // A frame still expects children: the tree is missing fields.
            return Err(PortableError::Invalid {
                reason: "truncated",
            });
        }
        if root.is_some() {
            return Err(PortableError::Invalid { reason: "trailing" });
        }
        let depth = stack.len() as u32 + 1;
        t.enter_node(depth)?;
        parsed_nodes += 1;
        let tag = r.u8()?;
        let (mut item_v, mut item_t): (Value, DynType) = match tag {
            TAG_I32 => (Value::I32(r.i32()?), DynType::I32),
            TAG_BOOL => {
                let b = r.u8()?;
                if b > 1 {
                    return Err(PortableError::Invalid {
                        reason: "bool-domain",
                    });
                }
                (Value::Bool(b == 1), DynType::Bool)
            }
            TAG_I64 => (Value::I64(r.i64()?), DynType::I64),
            TAG_U64 => (Value::U64(r.u64()?), DynType::U64),
            TAG_F32 => (Value::F32(f32::from_bits(r.u32()?)), DynType::F32),
            TAG_F64 => (Value::F64(f64::from_bits(r.u64()?)), DynType::F64),
            TAG_CHAR => {
                let c = char::from_u32(r.u32()?).ok_or(PortableError::Invalid {
                    reason: "char-domain",
                })?;
                (Value::Char(c), DynType::Char)
            }
            TAG_STR => {
                let len = r.u32()? as usize;
                let raw = r.take(len)?;
                let s = std::str::from_utf8(raw)
                    .map_err(|_| PortableError::Invalid { reason: "utf8" })?;
                (
                    Value::Str(StrHandle::detached(
                        s.as_bytes().to_vec().into_boxed_slice(),
                    )),
                    DynType::Str,
                )
            }
            TAG_RECORD => {
                let owner = r.str16()?;
                let name = r.str16()?;
                let nf = r.u16()? as usize;
                if nf > MAX_RECORD_FIELDS {
                    return Err(PortableError::Invalid {
                        reason: "field-count",
                    });
                }
                t.charge_steps(nf as u64 + 2)?;
                let mut names: Vec<String> = Vec::with_capacity(nf);
                let mut uniq: HashSet<String> = HashSet::new();
                for _ in 0..nf {
                    let n = r.str16()?;
                    if !uniq.insert(n.clone()) {
                        return Err(PortableError::Invalid {
                            reason: "duplicate-field",
                        });
                    }
                    names.push(n);
                }
                let binding = ctx.lookup(&owner, &name).ok_or(PortableError::Admission {
                    code: "portable.unknown-type",
                })?;
                if binding.fields.len() != names.len()
                    || binding
                        .fields
                        .iter()
                        .zip(names.iter())
                        .any(|(a, b)| *a != b.as_str())
                {
                    return Err(PortableError::Admission {
                        code: "portable.layout-mismatch",
                    });
                }
                t.charge_bytes(byte_cost_record_extra() + byte_cost_cell())?;
                if nf > 0 {
                    stack.push(Frame {
                        owner,
                        name,
                        field_names: names,
                        ty_index: binding.ty_index,
                        vals: Vec::new(),
                        tys: Vec::new(),
                    });
                    continue;
                }
                // A 0-field record completes immediately — no children will
                // arrive, so it rolls up the attach cascade like a leaf.
                (
                    Value::Record(Rc::new(RecordValue {
                        ty: binding.ty_index,
                        fields: Vec::new(),
                    })),
                    DynType::Record {
                        owner,
                        name,
                        fields: Vec::new(),
                    },
                )
            }
            _ => return Err(PortableError::Invalid { reason: "tag" }),
        };
        // A leaf completed — charge its rebuilt size, then roll it (and any
        // frames it completes) up the tree.
        t.charge_bytes(
            byte_cost_cell()
                + match &item_v {
                    Value::Str(h) => h.bytes().len(),
                    _ => 0,
                },
        )?;
        loop {
            match stack.last_mut() {
                None => {
                    root = Some((item_v, item_t));
                    break;
                }
                Some(f) => {
                    f.vals.push(item_v);
                    f.tys.push(item_t);
                    if f.vals.len() < f.field_names.len() {
                        break;
                    }
                    let done = stack.pop().unwrap();
                    item_v = Value::Record(Rc::new(RecordValue {
                        ty: done.ty_index,
                        fields: done.vals,
                    }));
                    item_t = DynType::Record {
                        owner: done.owner,
                        name: done.name,
                        fields: done
                            .field_names
                            .into_iter()
                            .zip(done.tys)
                            .map(|(name, ty)| DynField { name, ty })
                            .collect(),
                    };
                }
            }
        }
    }

    let (root_v, root_t) = root.ok_or(PortableError::Invalid {
        reason: "truncated",
    })?;
    if parsed_nodes != node_count {
        return Err(PortableError::Invalid {
            reason: "node-count",
        });
    }
    let sum = fnv1a64(&bytes[..body_len]);
    let want = u64::from_le_bytes(bytes[body_len..body_len + 8].try_into().unwrap());
    if sum != want {
        return Err(PortableError::Invalid { reason: "checksum" });
    }
    Ok(DynValue::from_parts(root_v, root_t))
}

struct Reader<'b> {
    b: &'b [u8],
    pos: usize,
}

impl<'b> Reader<'b> {
    fn u8(&mut self) -> Result<u8, PortableError> {
        let x = *self.b.get(self.pos).ok_or(PortableError::Invalid {
            reason: "truncated",
        })?;
        self.pos += 1;
        Ok(x)
    }
    fn u16(&mut self) -> Result<u16, PortableError> {
        Ok(u16::from_le_bytes(self.array::<2>()?))
    }
    fn u32(&mut self) -> Result<u32, PortableError> {
        Ok(u32::from_le_bytes(self.array::<4>()?))
    }
    fn i32(&mut self) -> Result<i32, PortableError> {
        Ok(i32::from_le_bytes(self.array::<4>()?))
    }
    fn i64(&mut self) -> Result<i64, PortableError> {
        Ok(i64::from_le_bytes(self.array::<8>()?))
    }
    fn u64(&mut self) -> Result<u64, PortableError> {
        Ok(u64::from_le_bytes(self.array::<8>()?))
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], PortableError> {
        let end = self.pos.checked_add(N).ok_or(PortableError::Invalid {
            reason: "truncated",
        })?;
        if end > self.b.len() {
            return Err(PortableError::Invalid {
                reason: "truncated",
            });
        }
        let a: [u8; N] = self.b[self.pos..end].try_into().unwrap();
        self.pos = end;
        Ok(a)
    }
    fn take(&mut self, n: usize) -> Result<&'b [u8], PortableError> {
        let end = self.pos.checked_add(n).ok_or(PortableError::Invalid {
            reason: "truncated",
        })?;
        if end > self.b.len() {
            return Err(PortableError::Invalid {
                reason: "truncated",
            });
        }
        let s = &self.b[self.pos..end];
        self.pos = end;
        Ok(s)
    }
    /// A `u16`-length UTF-8 name (owner / record / field names).
    fn str16(&mut self) -> Result<String, PortableError> {
        let len = self.u16()? as usize;
        let raw = self.take(len)?;
        std::str::from_utf8(raw)
            .map(|s| s.to_string())
            .map_err(|_| PortableError::Invalid { reason: "utf8" })
    }
}
