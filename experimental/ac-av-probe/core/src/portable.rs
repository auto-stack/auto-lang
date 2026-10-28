//! PLAN-824 portable value freezing (T-10 module subset + r2 collections
//! expansion, contract-decision.md §4–§5/§7).
//!
//! Same-source note: NOT declared in core's `lib.rs`; compiled by the
//! `parallel/824` harness via `#[path]` (see dynamic.rs).
//!
//! Portable contract (frozen; r2 layout note below):
//! - The wire is a versioned DFS-preorder tree where EVERY node carries
//!   its full inline TYPE DESCRIPTOR (self-describing: empty lists keep
//!   their element identity, non-live variant arms keep their payload
//!   types — types are never guessed from values) followed by the payload.
//! - Nominal identity (record/variant owner + name + field/arm names in
//!   declaration order) is persisted and re-admitted against the CURRENT
//!   dependencies through [`RestoreContext`] (`portable.unknown-type` /
//!   `portable.layout-mismatch`, §5.1.5). The wire NEVER persists process
//!   addresses, allocation ids, arena hooks, positional table indices,
//!   frame/borrow tokens, list epochs/generations or any credential.
//! - `freeze` enforces the owned-tree boundary: every `Rc` payload
//!   (record, Str buffer, list buffer, variant payload) may be visited
//!   exactly once — a shared child (including an acyclic DAG), a cycle or
//!   a repeat reference is rejected `portable.unsupported(shared-graph)`
//!   and the SOURCE IS NOT CONSUMED (V-04/V-12/V-14d).
//! - `unfreeze` re-verifies in the frozen order: bounds/budgets BEFORE
//!   semantics, structure (tags/domains/UTF-8/duplicate names/declared
//!   vs observed types/truncation/trailing), current nominal admission,
//!   then the FNV-1a trailer. A good checksum never bypasses a structural
//!   check and vice versa.
//! - Restored Str/List handles are detached (id 0, no arena) and list
//!   epochs/generations restart at 0: module-level restore has no ledger —
//!   in-execution restore mints arena handles at T-02+ (contract §5).
//!   Half-built graphs on failure are reclaimed by ownership rollback.
//!
//! r2 layout note: v1's record-header stream became the inline-tydesc
//! stream (List/Variant payloads need self-describing types). The format
//! was never published outside this plan — both sides ship together, so
//! the version stays 1 (contract §12).

use crate::contract::{
    fnv1a64, ListHandle, RecordValue, StrHandle, Value, VariantValue, MAX_LIST_ELEMS,
    MAX_RECORD_FIELDS,
};
use crate::dynamic::{
    byte_cost_cell, byte_cost_list_extra, byte_cost_record_extra, leaf_cost, DynArm, DynField,
    DynType, DynValue, Traversal, TraversalOptions, TraversalStop, MAX_DYN_BYTES, MAX_DYN_DEPTH,
    MAX_DYN_NODES,
};
use std::collections::HashSet;
use std::rc::Rc;

/// Portable schema identity (contract §4).
pub const PORTABLE_SCHEMA: &str = "auto.portable.av-dynamic/1";
/// Wire magic family of the portable frozen payload (`AVPF`).
pub const PORTABLE_MAGIC: [u8; 4] = *b"AVPF";
/// Wire version (v1; the r2 collections expansion stays v1 — the format
/// was never published outside this plan, both sides ship together).
pub const PORTABLE_VERSION: u16 = 1;
/// Payload kind: a DynValue tree (the only kind of v1).
pub const PORTABLE_KIND_DYN_VALUE: u8 = 1;

const HEADER_LEN: usize = 16;
const TRAILER_LEN: usize = 8;

// Node payload tags (value kinds).
const TAG_I32: u8 = 0;
const TAG_BOOL: u8 = 1;
const TAG_I64: u8 = 2;
const TAG_U64: u8 = 3;
const TAG_F32: u8 = 4;
const TAG_F64: u8 = 5;
const TAG_CHAR: u8 = 6;
const TAG_STR: u8 = 7;
const TAG_RECORD: u8 = 8;
const TAG_LIST: u8 = 9;
const TAG_VARIANT: u8 = 10;

// Type-descriptor tags (inline before every node payload).
const TY_BOOL: u8 = 0;
const TY_I32: u8 = 1;
const TY_I64: u8 = 2;
const TY_U64: u8 = 3;
const TY_F32: u8 = 4;
const TY_F64: u8 = 5;
const TY_CHAR: u8 = 6;
const TY_STR: u8 = 7;
const TY_LIST: u8 = 8;
const TY_RECORD: u8 = 9;
const TY_VARIANT: u8 = 10;

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
    /// tag / tydesc / bool-domain / char-domain / utf8 / duplicate-field /
    /// duplicate-arm / name-length / field-count / arm-count / type /
    /// truncated / trailing / checksum.
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

/// One current nominal RECORD binding: the declaration this decode domain
/// accepts — owner + name + field names in declaration order WITH the
/// CURRENT declared field types (complete recursive identity: a names-only
/// shape cannot even express a changed field type — F-824-R1-02) — and
/// the positional type-table index restored `RecordValue`s carry. Field
/// types are `Rc` so a builder resolving a SHARED type table hands out one
/// node set per type index instead of re-expanding the DAG per field.
#[derive(Clone)]
pub struct RecordBinding<'a> {
    pub owner: &'a str,
    pub name: &'a str,
    pub fields: Vec<(&'a str, Rc<DynType>)>,
    pub ty_index: u32,
}

/// The arm signature of one current nominal VARIANT binding: name + the
/// CURRENT declared payload type (`None` = payload-less arm).
#[derive(Clone)]
pub struct ArmSig<'a> {
    pub name: &'a str,
    pub payload: Option<Rc<DynType>>,
}

/// One current nominal VARIANT binding (arms in declaration order).
#[derive(Clone)]
pub struct VariantBinding<'a> {
    pub owner: &'a str,
    pub name: &'a str,
    pub arms: Vec<ArmSig<'a>>,
    pub ty_index: u32,
}

/// The read-only current-dependency set of one decode domain. No
/// credentials: restoring never un-freezes trust, it re-verifies names.
pub struct RestoreContext<'a> {
    pub records: &'a [RecordBinding<'a>],
    pub variants: &'a [VariantBinding<'a>],
}

impl<'a> RestoreContext<'a> {
    pub fn new(records: &'a [RecordBinding<'a>], variants: &'a [VariantBinding<'a>]) -> Self {
        RestoreContext { records, variants }
    }

    fn record(
        &self,
        owner: &str,
        name: &str,
        t: &mut Traversal,
    ) -> Result<Option<&'a RecordBinding<'a>>, PortableError> {
        for binding in self.records {
            t.charge_steps(1)?;
            if binding.owner == owner && binding.name == name {
                return Ok(Some(binding));
            }
        }
        Ok(None)
    }

    fn variant(
        &self,
        owner: &str,
        name: &str,
        t: &mut Traversal,
    ) -> Result<Option<&'a VariantBinding<'a>>, PortableError> {
        for binding in self.variants {
            t.charge_steps(1)?;
            if binding.owner == owner && binding.name == name {
                return Ok(Some(binding));
            }
        }
        Ok(None)
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

    // Unique-in-edge ledger: every Rc payload (record / Str buffer / list
    // buffer / variant payload) may be emitted exactly once.
    let mut visited: HashSet<usize> = HashSet::new();
    // DFS preorder work stack of SHALLOW-owned values (handle bumps — a
    // ListBuf RefCell is read exactly once per pop, guards never stack).
    let mut stack: Vec<(Value, &DynType, u32)> = vec![(v.as_value().clone(), v.ty(), 1)];

    while let Some((val, ty, depth)) = stack.pop() {
        t.enter_node(depth)?;
        // `node` carries this node's full bytes except large payloads,
        // which are charged and appended separately (the byte budget is
        // checked BEFORE any sizeable payload write).
        let mut node = Vec::with_capacity(16);
        let mut payload: Option<(usize, &[u8])> = None; // (prefix_len, bytes)
        let mut children: Vec<(Value, &DynType)> = Vec::new();
        match &val {
            // PLAN-862: the unit value has no portable encoding — it is a
            // zero-width result channel value, never a Dyn payload. A
            // controlled rejection, not a silent zero.
            Value::Unit => {
                return Err(PortableError::Unsupported {
                    category: "unit-value",
                })
            }
            Value::I32(x) => {
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_I32);
                node.extend_from_slice(&x.to_le_bytes());
            }
            Value::Bool(b) => {
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_BOOL);
                node.push(*b as u8);
            }
            Value::I64(x) => {
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_I64);
                node.extend_from_slice(&x.to_le_bytes());
            }
            Value::U64(x) => {
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_U64);
                node.extend_from_slice(&x.to_le_bytes());
            }
            Value::F32(x) => {
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_F32);
                node.extend_from_slice(&x.to_bits().to_le_bytes());
            }
            Value::F64(x) => {
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_F64);
                node.extend_from_slice(&x.to_bits().to_le_bytes());
            }
            Value::Char(c) => {
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_CHAR);
                node.extend_from_slice(&(*c as u32).to_le_bytes());
            }
            Value::Str(h) => {
                if !visited.insert(Rc::as_ptr(&h.rc) as usize) {
                    return Err(PortableError::Unsupported {
                        category: "shared-graph",
                    });
                }
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_STR);
                node.extend_from_slice(&(h.bytes().len() as u32).to_le_bytes());
                payload = Some((node.len(), h.bytes()));
            }
            Value::Record(rec) => {
                let fields = match ty {
                    DynType::Record { fields, .. } => fields,
                    _ => return Err(PortableError::Invalid { reason: "type" }),
                };
                if rec.fields.len() != fields.len() {
                    // Unreachable for a valid DynValue (pack invariant).
                    return Err(PortableError::Invalid { reason: "type" });
                }
                if !visited.insert(Rc::as_ptr(rec) as usize) {
                    return Err(PortableError::Unsupported {
                        category: "shared-graph",
                    });
                }
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_RECORD);
                for (fv, ft) in rec.fields.iter().zip(fields.iter()) {
                    children.push((fv.clone(), &ft.ty));
                }
                // Push in reverse so the LIFO pops field 0 first
                // (declaration order on the wire).
            }
            Value::List(h) => {
                let elem = match ty {
                    DynType::List(elem) => elem,
                    _ => return Err(PortableError::Invalid { reason: "type" }),
                };
                if !visited.insert(Rc::as_ptr(&h.rc) as usize) {
                    return Err(PortableError::Unsupported {
                        category: "shared-graph",
                    });
                }
                let elems = h.rc.borrow().elems.clone();
                if elems.len() > MAX_LIST_ELEMS {
                    return Err(PortableError::Budget {
                        code: "av.resource.dyn-nodes",
                    });
                }
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_LIST);
                node.extend_from_slice(&(elems.len() as u32).to_le_bytes());
                for ev in elems.iter() {
                    children.push((ev.clone(), elem));
                }
            }
            Value::Variant(var) => {
                let arms = match ty {
                    DynType::Variant { arms, .. } => arms,
                    _ => return Err(PortableError::Invalid { reason: "type" }),
                };
                let arm = arms
                    .get(var.disc as usize)
                    .ok_or(PortableError::Invalid { reason: "type" })?;
                if let Some(p) = &var.payload {
                    if !visited.insert(Rc::as_ptr(p) as usize) {
                        return Err(PortableError::Unsupported {
                            category: "shared-graph",
                        });
                    }
                }
                write_tydesc(&mut node, ty, 1, &mut t)?;
                node.push(TAG_VARIANT);
                node.extend_from_slice(&var.disc.to_le_bytes());
                if let Some(p) = &var.payload {
                    if let Some(pt) = &arm.payload {
                        children.push(((**p).clone(), pt));
                    }
                }
            }
            // The pack whitelist makes these unreachable; kept explicit so
            // a future category cannot silently ride the wire.
            Value::StrView(_) | Value::MutRef(_) | Value::Place(_) => {
                return Err(PortableError::Unsupported { category: "borrow" });
            }
            Value::Closure(_) => {
                return Err(PortableError::Unsupported {
                    category: "closure",
                });
            }
            Value::Dyn(_) => {
                // A Dyn never nests inside a frozen tree in v1 (nested
                // erasure is not a whitelisted category).
                return Err(PortableError::Unsupported { category: "dyn" });
            }
        }
        match payload {
            Some((prefix, bytes)) => {
                t.charge_bytes(node.len() + bytes.len())?;
                out.extend_from_slice(&node[..prefix]);
                out.extend_from_slice(bytes);
            }
            None => {
                t.charge_bytes(node.len())?;
                out.extend_from_slice(&node);
            }
        }
        let d = depth + 1;
        for (cv, ct) in children.into_iter().rev() {
            stack.push((cv, ct, d));
        }
    }

    // Patch the real node count, then the checksum trailer.
    let nodes = t.node_count();
    out[8..12].copy_from_slice(&nodes.to_le_bytes());
    t.charge_bytes(TRAILER_LEN)?;
    let sum = fnv1a64(&out);
    out.extend_from_slice(&sum.to_le_bytes());
    Ok(out)
}

/// Write one inline type descriptor. Recursive with a hard depth bound
/// (`MAX_DYN_DEPTH`): a packed DynValue's type tree is already within it,
/// so the guard is pure defense.
fn write_tydesc(
    out: &mut Vec<u8>,
    ty: &DynType,
    depth: u32,
    t: &mut Traversal,
) -> Result<(), PortableError> {
    if depth > MAX_DYN_DEPTH {
        return Err(PortableError::Budget {
            code: "av.resource.dyn-depth",
        });
    }
    t.charge_steps(1)?;
    t.check_pending_bytes(out.len().saturating_add(1))?;
    match ty {
        DynType::Bool => out.push(TY_BOOL),
        DynType::I32 => out.push(TY_I32),
        DynType::I64 => out.push(TY_I64),
        DynType::U64 => out.push(TY_U64),
        DynType::F32 => out.push(TY_F32),
        DynType::F64 => out.push(TY_F64),
        DynType::Char => out.push(TY_CHAR),
        DynType::Str => out.push(TY_STR),
        DynType::List(elem) => {
            out.push(TY_LIST);
            write_tydesc(out, elem, depth + 1, t)?;
        }
        DynType::Record {
            owner,
            name,
            fields,
        } => {
            out.push(TY_RECORD);
            push_name(out, owner, t)?;
            push_name(out, name, t)?;
            t.check_pending_bytes(out.len().saturating_add(2))?;
            out.extend_from_slice(&(fields.len() as u16).to_le_bytes());
            for f in fields {
                t.charge_steps(1)?;
                push_name(out, &f.name, t)?;
                write_tydesc(out, &f.ty, depth + 1, t)?;
            }
        }
        DynType::Variant { owner, name, arms } => {
            out.push(TY_VARIANT);
            push_name(out, owner, t)?;
            push_name(out, name, t)?;
            t.check_pending_bytes(out.len().saturating_add(2))?;
            out.extend_from_slice(&(arms.len() as u16).to_le_bytes());
            for a in arms {
                t.charge_steps(1)?;
                push_name(out, &a.name, t)?;
                t.check_pending_bytes(out.len().saturating_add(1))?;
                match &a.payload {
                    Some(p) => {
                        out.push(1);
                        write_tydesc(out, p, depth + 1, t)?;
                    }
                    None => out.push(0),
                }
            }
        }
    }
    Ok(())
}

fn push_name(out: &mut Vec<u8>, s: &str, t: &Traversal) -> Result<(), PortableError> {
    if s.len() > u16::MAX as usize {
        return Err(PortableError::Invalid {
            reason: "name-length",
        });
    }
    t.check_pending_bytes(out.len().saturating_add(2).saturating_add(s.len()))?;
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
    // Retain exactly one root descriptor. A child descriptor is read,
    // compared to the parent's declared subtree, then dropped immediately;
    // open value frames borrow subtrees from the root rather than retaining
    // one complete owned descriptor per depth.
    t.enter_node(1)?;
    let root_ty = read_tydesc(&mut r, &mut t, 1, false)?;
    admit_tydesc(&root_ty, ctx, &mut t)?;
    struct Frame<'a> {
        ty: &'a DynType,
        want: usize,
        ty_index: u32,
        disc: Option<u32>,
        vals: Vec<Value>,
    }
    fn expected_child<'a>(f: &Frame<'a>) -> Result<&'a DynType, PortableError> {
        match (f.ty, f.vals.len(), f.disc) {
            (DynType::Record { fields, .. }, i, _) => fields.get(i).map(|f| &f.ty),
            (DynType::List(elem), _, _) => Some(elem.as_ref()),
            (DynType::Variant { arms, .. }, _, Some(disc)) => {
                arms.get(disc as usize).and_then(|a| a.payload.as_ref())
            }
            _ => None,
        }
        .ok_or(PortableError::Invalid { reason: "type" })
    }
    let mut stack: Vec<Frame<'_>> = Vec::new();
    let mut root: Option<Value> = None;
    let mut parsed_nodes: u32 = 0;

    loop {
        if r.pos == body_len {
            if stack.is_empty() {
                break;
            }
            return Err(PortableError::Invalid {
                reason: "truncated",
            });
        }
        if root.is_some() {
            return Err(PortableError::Invalid { reason: "trailing" });
        }
        let node_ty = if parsed_nodes == 0 {
            &root_ty
        } else {
            let expected = expected_child(
                stack
                    .last()
                    .ok_or(PortableError::Invalid { reason: "trailing" })?,
            )?;
            t.enter_node(stack.len() as u32 + 1)?;
            let before = t.bytes_used();
            let observed = read_tydesc(&mut r, &mut t, 1, false)?;
            if !ty_ident(&observed, expected, &mut t)? {
                return Err(PortableError::Invalid { reason: "type" });
            }
            drop(observed);
            let temporary = t.bytes_used().saturating_sub(before);
            t.release_bytes(temporary);
            expected
        };
        parsed_nodes += 1;
        let tag = r.u8()?;
        // F-824-R1-01: a scalar payload pairs with EXACTLY its own
        // descriptor kind — a tampered descriptor (even one with a
        // recomputed checksum) is a structural `type` rejection, never a
        // checked-type lie. Container tags carry their own kind checks in
        // the arms below.
        expect_leaf_kind(tag, node_ty)?;
        let mut item_v: Value = match tag {
            TAG_I32 => Value::I32(r.i32()?),
            TAG_BOOL => {
                let b = r.u8()?;
                if b > 1 {
                    return Err(PortableError::Invalid {
                        reason: "bool-domain",
                    });
                }
                Value::Bool(b == 1)
            }
            TAG_I64 => Value::I64(r.i64()?),
            TAG_U64 => Value::U64(r.u64()?),
            TAG_F32 => Value::F32(f32::from_bits(r.u32()?)),
            TAG_F64 => Value::F64(f64::from_bits(r.u64()?)),
            TAG_CHAR => {
                let c = char::from_u32(r.u32()?).ok_or(PortableError::Invalid {
                    reason: "char-domain",
                })?;
                Value::Char(c)
            }
            TAG_STR => {
                let len = r.u32()? as usize;
                let raw = r.take(len)?;
                let s = std::str::from_utf8(raw)
                    .map_err(|_| PortableError::Invalid { reason: "utf8" })?;
                t.check_pending_bytes(len)?;
                Value::Str(StrHandle::detached(
                    s.as_bytes().to_vec().into_boxed_slice(),
                ))
            }
            TAG_RECORD => {
                let (owner, name) = match node_ty {
                    DynType::Record { owner, name, .. } => (owner.as_str(), name.as_str()),
                    _ => return Err(PortableError::Invalid { reason: "type" }),
                };
                let nfields = match node_ty {
                    DynType::Record { fields, .. } => fields.len(),
                    _ => unreachable!(),
                };
                let binding = ctx
                    .record(owner, name, &mut t)?
                    .ok_or(PortableError::Admission {
                        code: "portable.unknown-type",
                    })?;
                let names_ok = binding.fields.len() == nfields && {
                    let fields = match node_ty {
                        DynType::Record { fields, .. } => fields,
                        _ => unreachable!(),
                    };
                    binding
                        .fields
                        .iter()
                        .zip(fields.iter())
                        .all(|((bn, _bt), f)| *bn == f.name.as_str())
                };
                if !names_ok {
                    return Err(PortableError::Admission {
                        code: "portable.layout-mismatch",
                    });
                }
                t.charge_bytes(byte_cost_cell() + byte_cost_record_extra())?;
                if nfields > 0 {
                    stack.push(Frame {
                        ty: node_ty,
                        want: nfields,
                        ty_index: binding.ty_index,
                        disc: None,
                        vals: Vec::new(),
                    });
                    continue;
                }
                // A 0-field record completes immediately — no children will
                // arrive, so it rolls up the attach cascade like a leaf.
                Value::Record(Rc::new(RecordValue {
                    ty: binding.ty_index,
                    fields: Vec::new(),
                }))
            }
            TAG_LIST => {
                if !matches!(node_ty, DynType::List(_)) {
                    return Err(PortableError::Invalid { reason: "type" });
                }
                let count = r.u32()? as usize;
                if count > MAX_LIST_ELEMS {
                    return Err(PortableError::Budget {
                        code: "av.resource.dyn-nodes",
                    });
                }
                t.charge_bytes(byte_cost_cell() + byte_cost_list_extra())?;
                if count > 0 {
                    stack.push(Frame {
                        ty: node_ty,
                        want: count,
                        ty_index: 0,
                        disc: None,
                        vals: Vec::new(),
                    });
                    continue;
                }
                // An empty list completes immediately (its element identity
                // lives in the inline tydesc, not in children).
                Value::List(ListHandle::detached(Vec::new()))
            }
            TAG_VARIANT => {
                let disc = r.u32()?;
                let (owner, name, arms_len, arm_has_payload) = match node_ty {
                    DynType::Variant { owner, name, arms } => (
                        owner.as_str(),
                        name.as_str(),
                        arms.len(),
                        arms.get(disc as usize).map(|a| a.payload.is_some()),
                    ),
                    _ => return Err(PortableError::Invalid { reason: "type" }),
                };
                if disc as usize >= arms_len {
                    return Err(PortableError::Invalid { reason: "type" });
                }
                let binding =
                    ctx.variant(owner, name, &mut t)?
                        .ok_or(PortableError::Admission {
                            code: "portable.unknown-type",
                        })?;
                let sigs_ok = binding.arms.len() == arms_len && {
                    let arms = match node_ty {
                        DynType::Variant { arms, .. } => arms,
                        _ => unreachable!(),
                    };
                    binding.arms.iter().zip(arms.iter()).all(|(sig, a)| {
                        sig.name == a.name.as_str() && sig.payload.is_some() == a.payload.is_some()
                    })
                };
                if !sigs_ok {
                    return Err(PortableError::Admission {
                        code: "portable.layout-mismatch",
                    });
                }
                t.charge_bytes(byte_cost_cell())?;
                if arm_has_payload.unwrap_or(false) {
                    stack.push(Frame {
                        ty: node_ty,
                        want: 1,
                        ty_index: binding.ty_index,
                        disc: Some(disc),
                        vals: Vec::new(),
                    });
                    continue;
                }
                // A payload-less arm completes immediately.
                Value::Variant(VariantValue {
                    ty: binding.ty_index,
                    disc,
                    payload: None,
                })
            }
            _ => return Err(PortableError::Invalid { reason: "tag" }),
        };
        // The descriptor has already been checked against its parent.
        t.charge_bytes(leaf_cost(&item_v))?;
        loop {
            match stack.last_mut() {
                None => {
                    root = Some(item_v);
                    break;
                }
                Some(f) => {
                    f.vals.push(item_v);
                    if f.vals.len() < f.want {
                        break;
                    }
                    let mut done = stack.pop().unwrap();
                    item_v = match (done.ty, done.disc) {
                        (DynType::Record { .. }, _) => Value::Record(Rc::new(RecordValue {
                            ty: done.ty_index,
                            fields: done.vals,
                        })),
                        (DynType::List(_), _) => Value::List(ListHandle::detached(done.vals)),
                        (DynType::Variant { .. }, Some(disc)) => Value::Variant(VariantValue {
                            ty: done.ty_index,
                            disc,
                            payload: done.vals.pop().map(Rc::new),
                        }),
                        _ => return Err(PortableError::Invalid { reason: "type" }),
                    };
                }
            }
        }
    }

    let root_v = root.ok_or(PortableError::Invalid {
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
    Ok(DynValue::from_parts(root_v, root_ty))
}

/// Read one inline type descriptor (see `write_tydesc`).
fn read_tydesc(
    r: &mut Reader,
    t: &mut Traversal,
    depth: u32,
    embedded: bool,
) -> Result<DynType, PortableError> {
    if depth > MAX_DYN_DEPTH {
        return Err(PortableError::Budget {
            code: "av.resource.dyn-depth",
        });
    }
    t.charge_steps(1)?;
    if !embedded {
        t.charge_bytes(std::mem::size_of::<DynType>())?;
    }
    let tag = r.u8()?;
    Ok(match tag {
        TY_BOOL => DynType::Bool,
        TY_I32 => DynType::I32,
        TY_I64 => DynType::I64,
        TY_U64 => DynType::U64,
        TY_F32 => DynType::F32,
        TY_F64 => DynType::F64,
        TY_CHAR => DynType::Char,
        TY_STR => DynType::Str,
        TY_LIST => DynType::List(Box::new(read_tydesc(r, t, depth + 1, false)?)),
        TY_RECORD => {
            let owner = r.str16(t, false)?;
            let name = r.str16(t, false)?;
            let nf = r.u16()? as usize;
            if nf > MAX_RECORD_FIELDS {
                return Err(PortableError::Invalid {
                    reason: "field-count",
                });
            }
            t.charge_steps(1)?;
            t.charge_bytes(nf.saturating_mul(
                std::mem::size_of::<DynField>().saturating_add(2 * std::mem::size_of::<String>()),
            ))?;
            let mut fields = Vec::with_capacity(nf);
            let mut uniq: HashSet<String> = HashSet::new();
            for _ in 0..nf {
                t.charge_steps(1)?;
                let fname = r.str16(t, true)?;
                if !uniq.insert(fname.clone()) {
                    return Err(PortableError::Invalid {
                        reason: "duplicate-field",
                    });
                }
                let fty = read_tydesc(r, t, depth + 1, true)?;
                fields.push(DynField {
                    name: fname,
                    ty: fty,
                });
            }
            DynType::Record {
                owner,
                name,
                fields,
            }
        }
        TY_VARIANT => {
            let owner = r.str16(t, false)?;
            let name = r.str16(t, false)?;
            let na = r.u16()? as usize;
            if na > MAX_RECORD_FIELDS {
                return Err(PortableError::Invalid {
                    reason: "arm-count",
                });
            }
            t.charge_steps(1)?;
            t.charge_bytes(na.saturating_mul(
                std::mem::size_of::<DynArm>().saturating_add(2 * std::mem::size_of::<String>()),
            ))?;
            let mut arms = Vec::with_capacity(na);
            let mut uniq: HashSet<String> = HashSet::new();
            for _ in 0..na {
                t.charge_steps(1)?;
                let aname = r.str16(t, true)?;
                if !uniq.insert(aname.clone()) {
                    return Err(PortableError::Invalid {
                        reason: "duplicate-arm",
                    });
                }
                let has = r.u8()?;
                let payload = if has == 1 {
                    Some(read_tydesc(r, t, depth + 1, true)?)
                } else if has == 0 {
                    None
                } else {
                    return Err(PortableError::Invalid { reason: "type" });
                };
                arms.push(DynArm {
                    name: aname,
                    payload,
                });
            }
            DynType::Variant { owner, name, arms }
        }
        _ => return Err(PortableError::Invalid { reason: "tydesc" }),
    })
}

/// Admit the ROOT type descriptor against the CURRENT dependencies
/// (F-824-R1-02): EVERY nominal occurrence in the tree — including ones
/// with no value node downstream (an empty List's element type, an
/// inactive arm's payload type, an unused nested parameter) — is looked
/// up (`portable.unknown-type`) and compared for COMPLETE recursive type
/// identity: same owner/name/field/arm names AND the same field/payload
/// TYPES at every depth (`portable.layout-mismatch`). Downstream node
/// descriptors are subtrees of this root (the declared-vs-observed
/// rollup rejects any disagreement structurally), so one walk here
/// covers the whole wire's type identity. Explicit worklist (the
/// module's no-host-stack-recursion discipline): every visited node and
/// every compared node pair charges one traversal step.
fn admit_tydesc(
    ty: &DynType,
    ctx: &RestoreContext,
    t: &mut Traversal,
) -> Result<(), PortableError> {
    let mut walk: Vec<&DynType> = vec![ty];
    while let Some(node) = walk.pop() {
        t.charge_steps(1)?;
        match node {
            DynType::Record {
                owner,
                name,
                fields,
            } => {
                admit_record(ctx, owner, name, fields, t)?;
                for field in fields {
                    t.charge_steps(1)?;
                    walk.push(&field.ty);
                }
            }
            DynType::Variant { owner, name, arms } => {
                admit_variant(ctx, owner, name, arms, t)?;
                for a in arms {
                    if let Some(p) = &a.payload {
                        walk.push(p);
                    }
                }
            }
            DynType::List(elem) => walk.push(elem),
            _ => {}
        }
    }
    Ok(())
}

/// Admit one decoded RECORD descriptor against the current bindings
/// (see [`admit_tydesc`]'s F-824-R1-02 note).
fn admit_record(
    ctx: &RestoreContext,
    owner: &str,
    name: &str,
    fields: &[DynField],
    t: &mut Traversal,
) -> Result<(), PortableError> {
    let binding = ctx
        .record(owner, name, t)?
        .ok_or(PortableError::Admission {
            code: "portable.unknown-type",
        })?;
    if binding.fields.len() != fields.len() {
        return Err(PortableError::Admission {
            code: "portable.layout-mismatch",
        });
    }
    for ((bn, bt), f) in binding.fields.iter().zip(fields.iter()) {
        t.charge_steps(1)?;
        if *bn != f.name.as_str() || !ty_ident(&f.ty, bt, t)? {
            return Err(PortableError::Admission {
                code: "portable.layout-mismatch",
            });
        }
    }
    Ok(())
}

/// Admit one decoded VARIANT descriptor against the current bindings.
fn admit_variant(
    ctx: &RestoreContext,
    owner: &str,
    name: &str,
    arms: &[DynArm],
    t: &mut Traversal,
) -> Result<(), PortableError> {
    let binding = ctx
        .variant(owner, name, t)?
        .ok_or(PortableError::Admission {
            code: "portable.unknown-type",
        })?;
    if binding.arms.len() != arms.len() {
        return Err(PortableError::Admission {
            code: "portable.layout-mismatch",
        });
    }
    for (sig, a) in binding.arms.iter().zip(arms.iter()) {
        t.charge_steps(1)?;
        let same = sig.name == a.name.as_str()
            && match (&sig.payload, &a.payload) {
                (Some(bt), Some(wire)) => ty_ident(wire, bt, t)?,
                (None, None) => true,
                _ => false,
            };
        if !same {
            return Err(PortableError::Admission {
                code: "portable.layout-mismatch",
            });
        }
    }
    Ok(())
}

/// Full recursive type identity between the WIRE descriptor and the
/// CURRENT declared binding type. Same owner/name/field-names is NOT
/// enough (F-824-R1-02): a changed field or payload type at any depth is
/// a mismatch. Explicit pair worklist (no host-stack recursion); every
/// compared node pair charges one traversal step, so a wide comparison
/// storm trips the steps budget instead of running unbilled.
fn ty_ident(wire: &DynType, current: &DynType, t: &mut Traversal) -> Result<bool, PortableError> {
    let mut stack: Vec<(&DynType, &DynType)> = vec![(wire, current)];
    while let Some((w, c)) = stack.pop() {
        t.charge_steps(1)?;
        match (w, c) {
            (DynType::List(a), DynType::List(b)) => stack.push((a, b)),
            (
                DynType::Record {
                    owner: ao,
                    name: an,
                    fields: af,
                },
                DynType::Record {
                    owner: bo,
                    name: bn,
                    fields: bf,
                },
            ) => {
                if ao != bo || an != bn || af.len() != bf.len() {
                    return Ok(false);
                }
                for (x, y) in af.iter().zip(bf.iter()) {
                    t.charge_steps(1)?;
                    if x.name != y.name {
                        return Ok(false);
                    }
                    stack.push((&x.ty, &y.ty));
                }
            }
            (
                DynType::Variant {
                    owner: ao,
                    name: an,
                    arms: aa,
                },
                DynType::Variant {
                    owner: bo,
                    name: bn,
                    arms: ba,
                },
            ) => {
                if ao != bo || an != bn || aa.len() != ba.len() {
                    return Ok(false);
                }
                for (x, y) in aa.iter().zip(ba.iter()) {
                    t.charge_steps(1)?;
                    if x.name != y.name {
                        return Ok(false);
                    }
                    match (&x.payload, &y.payload) {
                        (Some(xp), Some(yp)) => stack.push((xp, yp)),
                        (None, None) => {}
                        _ => return Ok(false),
                    }
                }
            }
            (DynType::Bool, DynType::Bool)
            | (DynType::I32, DynType::I32)
            | (DynType::I64, DynType::I64)
            | (DynType::U64, DynType::U64)
            | (DynType::F32, DynType::F32)
            | (DynType::F64, DynType::F64)
            | (DynType::Char, DynType::Char)
            | (DynType::Str, DynType::Str) => {}
            _ => return Ok(false),
        }
    }
    Ok(true)
}

/// Scalar leaf payload tags pair with EXACTLY the equal scalar descriptor
/// kind (F-824-R1-01); container tags do their own structural checks in
/// the unfreeze arms.
fn expect_leaf_kind(tag: u8, ty: &DynType) -> Result<(), PortableError> {
    use DynType as T;
    let ok = match tag {
        TAG_I32 => matches!(ty, T::I32),
        TAG_BOOL => matches!(ty, T::Bool),
        TAG_I64 => matches!(ty, T::I64),
        TAG_U64 => matches!(ty, T::U64),
        TAG_F32 => matches!(ty, T::F32),
        TAG_F64 => matches!(ty, T::F64),
        TAG_CHAR => matches!(ty, T::Char),
        TAG_STR => matches!(ty, T::Str),
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        Err(PortableError::Invalid { reason: "type" })
    }
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
    /// A `u16`-length UTF-8 name (owner / record / field / arm names).
    fn str16(&mut self, t: &mut Traversal, duplicate: bool) -> Result<String, PortableError> {
        let len = self.u16()? as usize;
        let raw = self.take(len)?;
        t.charge_bytes(len.saturating_mul(if duplicate { 2 } else { 1 }))?;
        std::str::from_utf8(raw)
            .map(|s| s.to_string())
            .map_err(|_| PortableError::Invalid { reason: "utf8" })
    }
}

#[cfg(test)]
mod plan823_824_cross_profile_tests {
    use super::*;
    use crate::contract::{ClosureRefV, RecordValue};
    use crate::dynamic::{DynPackError, DynType};
    use std::rc::Rc;

    fn closure() -> Value {
        Value::Closure(ClosureRefV {
            func: 1,
            sig: 2,
            env: 3,
            gen: 4,
        })
    }

    #[test]
    fn plan823_824_closure_rejected_at_pack_and_freeze_at_every_depth() {
        let ty = DynType::record("plan823", "Box", vec![("inner", DynType::I32)]);
        let nested = Value::Record(Rc::new(RecordValue {
            ty: 0,
            fields: vec![closure()],
        }));
        assert!(matches!(
            DynValue::pack(closure(), DynType::I32),
            Err(DynPackError::Unsupported {
                category: "closure",
                depth: 1
            })
        ));
        assert!(matches!(
            DynValue::pack(nested.clone(), ty.clone()),
            Err(DynPackError::Unsupported {
                category: "closure",
                depth: 2
            })
        ));
        // from_parts is a crate-private defensive test seam: normal callers cannot mint an invalid Dyn.
        let forged = DynValue::from_parts(nested, ty);
        assert!(matches!(
            freeze(&forged, &TraversalOptions::default()),
            Err(PortableError::Unsupported {
                category: "closure"
            })
        ));
        let valid = DynValue::pack(Value::I32(42), DynType::I32).expect("824 legal scalar");
        let bytes = freeze(&valid, &TraversalOptions::default()).expect("824 legal freeze");
        let recovered = unfreeze(
            &bytes,
            &RestoreContext::new(&[], &[]),
            &TraversalOptions::default(),
        )
        .expect("824 legal restore");
        assert!(matches!(recovered.as_value(), Value::I32(42)));
    }
}
