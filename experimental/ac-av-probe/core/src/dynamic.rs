//! PLAN-824 `av.dynamic/1` — the DynValue container (T-10 module subset,
//! contract-decision.md §2–§3/§7).
//!
//! Same-source note: this file is NOT declared in core's `lib.rs` (the
//! T-10 private module). The `parallel/824` harness compiles it via
//! `#[path]`; `crate::contract` resolves through the harness's re-export of
//! the delivered public core modules — and against core's own module
//! unchanged after the T-02/T-03 core window registers `mod dynamic;`.
//!
//! Contract surface (frozen in docs/plans/attachments/824/contract-decision.md):
//! - `pack` is the ONLY constructor of `DynValue`: explicit type erasure of
//!   the 821-delivered whitelist (bool/scalars/char/owned Str/pure owned
//!   records). `StrView`/`MutRef` are rejected at ANY depth (§5.1.1);
//!   Fn/closure/List/Variant have no category here at all (D822 pending).
//! - Type equivalence is the 820 frozen rule: derived `PartialEq` over the
//!   nominal record identity (owner + name + fields in declaration order).
//!   Same-width different-name records never compare equal; no cross-kind
//!   coercion anywhere.
//! - `borrow_as` grants a read-only share (no drop/copy right); `take_as`
//!   on a wrong type is a RUNTIME CHECKED failure that returns the source
//!   unconsumed (§5.5); copying must go through the budgeted `deep_clone`.
//! - Every traversal in this module is ITERATIVE and budgeted (nodes /
//!   steps / bytes / depth, §7 "有界显式遍历"): no host-stack recursion,
//!   so no value tree can stack-bomb a later freeze/clone/drop.

use crate::contract::{RecordValue, StrHandle, Value, CANCEL_CHECK_INTERVAL, MAX_RECORD_FIELDS};
use std::collections::HashSet;
use std::rc::Rc;

/// Value-tree node ceiling shared by freeze/unfreeze/deep_clone (§7).
pub const MAX_DYN_NODES: u32 = 65_536;
/// Traversal work-step ceiling (every node/element counts one step).
pub const MAX_DYN_STEPS: u64 = 2_000_000;
/// Data-byte ceiling: frozen payload size, unfreeze input size and rebuilt
/// value estimate all share this one 8 MiB line (§5.2 值数据合计口径).
pub const MAX_DYN_BYTES: usize = 8 * 1024 * 1024;
/// Value-tree depth ceiling. Source-derived values are frontend-nesting
/// bounded (≤128); 256 covers that with margin while keeping every later
/// recursive `Rc` drop trivially stack-safe.
pub const MAX_DYN_DEPTH: u32 = 256;

// ---------------------------------------------------------------------------
// Budgeted traversal core (shared with portable.rs)
// ---------------------------------------------------------------------------

/// Options carried by every budgeted operation. `fail_after_nodes` is the
/// deterministic fault seam (V-06/V-07 anchor): the Nth node fails with the
/// SAME resource classification as the node budget — injected faults are
/// resource-boundary equivalents, never a separate error state.
pub struct TraversalOptions<'a> {
    pub fail_after_nodes: Option<u32>,
    pub cancel: Option<&'a dyn Fn() -> bool>,
}

impl Default for TraversalOptions<'_> {
    fn default() -> Self {
        TraversalOptions {
            fail_after_nodes: None,
            cancel: None,
        }
    }
}

/// The resource/cancel stop states of a budgeted traversal (distinct from
/// the language-checked failures — §5.2 分态).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TraversalStop {
    Nodes,
    Steps,
    Bytes,
    Depth,
    Cancelled,
}

impl TraversalStop {
    pub fn code(&self) -> &'static str {
        match self {
            TraversalStop::Nodes => "av.resource.dyn-nodes",
            TraversalStop::Steps => "av.resource.dyn-steps",
            TraversalStop::Bytes => "av.resource.dyn-bytes",
            TraversalStop::Depth => "av.resource.dyn-depth",
            TraversalStop::Cancelled => "av.dyn.cancelled",
        }
    }
}

pub(crate) struct Traversal<'a> {
    opts: &'a TraversalOptions<'a>,
    nodes: u32,
    steps: u64,
    bytes: usize,
    since_cancel: u32,
    max_bytes: usize,
}

impl<'a> Traversal<'a> {
    pub(crate) fn new(opts: &'a TraversalOptions<'a>, max_bytes: usize) -> Self {
        Traversal {
            opts,
            nodes: 0,
            steps: 0,
            bytes: 0,
            since_cancel: 0,
            max_bytes,
        }
    }

    /// Enter (count) one node at `depth`. Depth first (structural safety),
    /// then the node ceiling, the fault seam (same class as the ceiling),
    /// one step per node, and the cancellation probe every
    /// `CANCEL_CHECK_INTERVAL` nodes (same interval as the executor).
    pub(crate) fn enter_node(&mut self, depth: u32) -> Result<(), TraversalStop> {
        if depth > MAX_DYN_DEPTH {
            return Err(TraversalStop::Depth);
        }
        self.nodes += 1;
        if self.nodes > MAX_DYN_NODES {
            return Err(TraversalStop::Nodes);
        }
        if let Some(n) = self.opts.fail_after_nodes {
            if n == self.nodes {
                return Err(TraversalStop::Nodes);
            }
        }
        self.steps += 1;
        if self.steps > MAX_DYN_STEPS {
            return Err(TraversalStop::Steps);
        }
        self.since_cancel += 1;
        if self.since_cancel >= CANCEL_CHECK_INTERVAL as u32 {
            self.since_cancel = 0;
            if let Some(cancel) = self.opts.cancel {
                if cancel() {
                    return Err(TraversalStop::Cancelled);
                }
            }
        }
        Ok(())
    }

    /// Charge `n` work steps (per-element costs: names, lengths, fields).
    pub(crate) fn charge_steps(&mut self, n: u64) -> Result<(), TraversalStop> {
        self.steps += n;
        if self.steps > MAX_DYN_STEPS {
            return Err(TraversalStop::Steps);
        }
        Ok(())
    }

    /// Charge `n` data bytes against the shared 8 MiB line (emitted bytes
    /// on freeze, rebuilt value estimate on unfreeze/clone).
    pub(crate) fn charge_bytes(&mut self, n: usize) -> Result<(), TraversalStop> {
        self.bytes += n;
        if self.bytes > self.max_bytes {
            return Err(TraversalStop::Bytes);
        }
        Ok(())
    }

    /// Nodes entered so far (freeze patches the wire header with this).
    pub(crate) fn node_count(&self) -> u32 {
        self.nodes
    }
}

// ---------------------------------------------------------------------------
// DynType — the self-contained whitelisted type tree
// ---------------------------------------------------------------------------

/// The whitelisted dynamic type tree (§2). Records carry their NOMINAL
/// identity (`owner` + `name` + declaration-order fields); derived
/// `PartialEq` IS the frozen 820 type-equivalence applied at `is` /
/// `borrow_as` / `take_as`. No Fn/closure/borrow/List category exists here.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DynType {
    Bool,
    I32,
    I64,
    U64,
    F32,
    F64,
    Char,
    Str,
    Record {
        owner: String,
        name: String,
        fields: Vec<DynField>,
    },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DynField {
    pub name: String,
    pub ty: DynType,
}

impl DynType {
    /// Stable human-readable descriptor used in mismatch diagnostics (the
    /// same text a wrong-type `take_as` reports).
    pub fn describe(&self) -> String {
        match self {
            DynType::Bool => "bool".into(),
            DynType::I32 => "i32".into(),
            DynType::I64 => "i64".into(),
            DynType::U64 => "u64".into(),
            DynType::F32 => "f32".into(),
            DynType::F64 => "f64".into(),
            DynType::Char => "char".into(),
            DynType::Str => "Str".into(),
            DynType::Record {
                owner,
                name,
                fields,
            } => {
                format!("record {}/{}[{}]", owner, name, fields.len())
            }
        }
    }

    /// Convenience constructor from `(&str, DynType)` field pairs.
    pub fn record(owner: &str, name: &str, fields: Vec<(&str, DynType)>) -> DynType {
        DynType::Record {
            owner: owner.to_string(),
            name: name.to_string(),
            fields: fields
                .into_iter()
                .map(|(n, ty)| DynField {
                    name: n.to_string(),
                    ty,
                })
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Pack admission
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DynPackError {
    /// Recursive whitelist rejection: `str-view` / `mut-ref` at any depth.
    Unsupported { category: &'static str, depth: u32 },
    /// Value shape does not match the declared type (kind or arity).
    TypeMismatch {
        expected: String,
        actual: String,
        depth: u32,
    },
    /// The declared type itself is malformed (duplicate field names,
    /// record wider than the core `MAX_RECORD_FIELDS` guard).
    InvalidType { reason: &'static str },
    /// Traversal budget rejection (depth — §7).
    Budget { code: &'static str },
}

impl DynPackError {
    pub fn code(&self) -> &'static str {
        match self {
            DynPackError::Unsupported { .. } => "av.dyn.unsupported",
            DynPackError::TypeMismatch { .. } => "av.dyn.pack-type",
            DynPackError::InvalidType { .. } => "av.dyn.invalid-type",
            DynPackError::Budget { .. } => "av.resource.dyn-depth",
        }
    }
}

/// The type-erased whitelisted value. Constructed ONLY by `pack` (and the
/// portable decoder through the crate-private `from_parts`); the
/// (value, type) pair invariant — whitelist-clean and structurally
/// consistent — holds by construction for every `DynValue` that exists.
#[derive(Debug, PartialEq, Eq)]
pub struct DynValue {
    pub(crate) value: Value,
    pub(crate) ty: DynType,
}

impl DynValue {
    /// Explicit type erasure with full recursive admission (§2): the value
    /// must contain only whitelisted categories at every depth AND match
    /// `ty` structurally. Consumes `value` (move semantics — §5.1.2);
    /// rejected values are dropped with normal cleanup before returning.
    pub fn pack(value: Value, ty: DynType) -> Result<DynValue, DynPackError> {
        validate_type(&ty)?;
        check_shape(&value, &ty)?;
        Ok(DynValue { value, ty })
    }

    /// Crate-private trust boundary for the portable decoder: the wire
    /// parse has structurally enforced the whitelist (only whitelisted
    /// tags decode), UTF-8 and char domains; this skips re-validation.
    pub(crate) fn from_parts(value: Value, ty: DynType) -> DynValue {
        DynValue { value, ty }
    }

    pub fn ty(&self) -> &DynType {
        &self.ty
    }

    /// Read-only untyped view (same sharing as `borrow_as`; a shared
    /// reference grants no release or copy right).
    pub fn as_value(&self) -> &Value {
        &self.value
    }

    pub fn is(&self, ty: &DynType) -> bool {
        self.ty == *ty
    }

    /// Checked typed borrow: `Some(&Value)` only when `ty` equals this
    /// value's type under the frozen equivalence; a share grants no
    /// release right and never clones.
    pub fn borrow_as(&self, ty: &DynType) -> Option<&Value> {
        if self.is(ty) {
            Some(&self.value)
        } else {
            None
        }
    }

    /// Checked recovery. A wrong type is a RUNTIME CHECKED failure (§5.5),
    /// not bad input: `Err` carries the original `DynValue` back — the
    /// source stays readable, take-able and owned (V-01/V-12/V-14b).
    pub fn take_as(self, ty: &DynType) -> Result<Value, DynTakeError> {
        if self.ty == *ty {
            Ok(self.value)
        } else {
            Err(DynTakeError {
                expected: ty.describe(),
                actual: self.ty.describe(),
                source: self,
            })
        }
    }

    /// Explicit independent deep copy (§3): fresh `Rc`s and fresh Str
    /// buffers — a shared DAG is EXPANDED into a tree (expanded nodes are
    /// what the node budget counts). Budgeted and cancellable like every
    /// traversal; module-level copies mint detached Str handles (id 0, no
    /// arena) — in-execution clones route through the real arena at T-02+.
    pub fn deep_clone(&self, opts: &TraversalOptions) -> Result<DynValue, TraversalStop> {
        let mut t = Traversal::new(opts, MAX_DYN_BYTES);
        let mut root: Option<(Value, DynType)> = None;

        match (&self.value, &self.ty) {
            // The root itself is a record: seed the build stack.
            (Value::Record(_), DynType::Record { .. }) => {}
            // Inconsistent pairing is unreachable for a packed DynValue;
            // refuse rather than clone across kinds.
            (Value::Record(_), _) | (_, DynType::Record { .. }) => {
                return Err(TraversalStop::Nodes);
            }
            // The root is a leaf: one node, done.
            (leaf, ty) => {
                t.enter_node(1)?;
                t.charge_bytes(leaf_cost(leaf))?;
                root = Some((clone_leaf(leaf), ty.clone()));
            }
        }

        // Explicit post-order build stack: a frame accumulates its finished
        // children; record children push a frame, leaves complete in place,
        // a full frame pops and rebuilds one level with a FRESH Rc.
        struct Frame<'a> {
            src: &'a Value,
            ty: &'a DynType,
            vals: Vec<Value>,
            tys: Vec<DynType>,
            next: usize,
            depth: u32,
            entered: bool,
        }
        let mut stack: Vec<Frame<'_>> = Vec::new();
        if root.is_none() {
            stack.push(Frame {
                src: &self.value,
                ty: &self.ty,
                vals: Vec::new(),
                tys: Vec::new(),
                next: 0,
                depth: 1,
                entered: false,
            });
        }

        while !stack.is_empty() {
            let idx = stack.len() - 1;
            if !stack[idx].entered {
                stack[idx].entered = true;
                let depth = stack[idx].depth;
                t.enter_node(depth)?;
                t.charge_bytes(byte_cost_record_extra() + byte_cost_cell())?;
            }
            let nfields = match stack[idx].ty {
                DynType::Record { fields, .. } => fields.len(),
                _ => unreachable!("only record frames are pushed"),
            };
            if stack[idx].next < nfields {
                let i = stack[idx].next;
                stack[idx].next += 1;
                let (cv, ct) = match (stack[idx].src, stack[idx].ty) {
                    (Value::Record(rec), DynType::Record { fields, .. }) => {
                        (&rec.fields[i], &fields[i].ty)
                    }
                    _ => unreachable!("frame sources are records"),
                };
                if let Value::Record(_) = cv {
                    let depth = stack[idx].depth + 1;
                    let src = cv;
                    let ty = ct;
                    stack.push(Frame {
                        src,
                        ty,
                        vals: Vec::new(),
                        tys: Vec::new(),
                        next: 0,
                        depth,
                        entered: false,
                    });
                } else {
                    let d = stack[idx].depth + 1;
                    t.enter_node(d)?;
                    t.charge_bytes(leaf_cost(cv))?;
                    stack[idx].vals.push(clone_leaf(cv));
                    stack[idx].tys.push(ct.clone());
                }
            } else {
                // All children finished: rebuild with a FRESH Rc — sharing
                // is deliberately broken (deep copy expands shared DAGs).
                let done = stack.pop().unwrap();
                let (owner, name, src_index) = match (&done.ty, done.src) {
                    (DynType::Record { owner, name, .. }, Value::Record(rec)) => {
                        (owner.clone(), name.clone(), rec.ty)
                    }
                    _ => unreachable!("frame sources are records"),
                };
                let field_names: Vec<String> = match &done.ty {
                    DynType::Record { fields, .. } => {
                        fields.iter().map(|f| f.name.clone()).collect()
                    }
                    _ => unreachable!(),
                };
                let value = Value::Record(Rc::new(RecordValue {
                    // DynType never carries the positional index (§5.1.5:
                    // the wire must not persist it) — the module-level copy
                    // reuses the source node's index verbatim; real index
                    // resolution belongs to the T-02+ RestoreContext.
                    ty: src_index,
                    fields: done.vals,
                }));
                let ty = DynType::Record {
                    owner,
                    name,
                    fields: field_names
                        .into_iter()
                        .zip(done.tys)
                        .map(|(name, ty)| DynField { name, ty })
                        .collect(),
                };
                match stack.last_mut() {
                    Some(parent) => {
                        parent.vals.push(value);
                        parent.tys.push(ty);
                    }
                    None => root = Some((value, ty)),
                }
            }
        }

        root.map(|(value, ty)| DynValue::from_parts(value, ty))
            .ok_or(TraversalStop::Nodes)
    }
}

/// Wrong-type `take_as` failure: `source` is the unconsumed original.
/// (Deliberately NOT `Clone`: a container copy would silently share the
/// payload — independent copies go through `deep_clone`.)
#[derive(Debug)]
pub struct DynTakeError {
    pub expected: String,
    pub actual: String,
    pub source: DynValue,
}

impl DynTakeError {
    pub fn code(&self) -> &'static str {
        "av.dyn.take-type"
    }
    /// Recover the untouched original value container.
    pub fn into_source(self) -> DynValue {
        self.source
    }
}

// ---------------------------------------------------------------------------
// Pack helpers (iterative — no host-stack recursion anywhere)
// ---------------------------------------------------------------------------

/// Validate the TYPE tree itself: unique field names per record, the core
/// `MAX_RECORD_FIELDS` width guard, per-node depth ceiling.
fn validate_type(ty: &DynType) -> Result<(), DynPackError> {
    let mut stack: Vec<(&DynType, u32)> = vec![(ty, 1)];
    while let Some((t, depth)) = stack.pop() {
        if depth > MAX_DYN_DEPTH {
            return Err(DynPackError::Budget {
                code: "av.resource.dyn-depth",
            });
        }
        if let DynType::Record { fields, .. } = t {
            if fields.len() > MAX_RECORD_FIELDS {
                return Err(DynPackError::InvalidType {
                    reason: "field-count",
                });
            }
            let mut names: HashSet<&str> = HashSet::new();
            for f in fields {
                if !names.insert(f.name.as_str()) {
                    return Err(DynPackError::InvalidType {
                        reason: "duplicate-field",
                    });
                }
                stack.push((&f.ty, depth + 1));
            }
        }
    }
    Ok(())
}

/// Recursive-shape admission of (value, type): whitelist at every depth +
/// structural match, with the same per-node depth rule freeze applies
/// (a packable tree is exactly a freezable one).
fn check_shape(value: &Value, ty: &DynType) -> Result<(), DynPackError> {
    let mut stack: Vec<(&Value, &DynType, u32)> = vec![(value, ty, 1)];
    while let Some((v, t, depth)) = stack.pop() {
        if depth > MAX_DYN_DEPTH {
            return Err(DynPackError::Budget {
                code: "av.resource.dyn-depth",
            });
        }
        use DynType as T;
        let ok = match (v, t) {
            (Value::Bool(_), T::Bool)
            | (Value::I32(_), T::I32)
            | (Value::I64(_), T::I64)
            | (Value::U64(_), T::U64)
            | (Value::F32(_), T::F32)
            | (Value::F64(_), T::F64)
            | (Value::Char(_), T::Char)
            | (Value::Str(_), T::Str) => true,
            (Value::Record(rec), T::Record { fields, .. }) => {
                if rec.fields.len() != fields.len() {
                    return Err(DynPackError::TypeMismatch {
                        expected: t.describe(),
                        actual: format!("record with {} fields", rec.fields.len()),
                        depth,
                    });
                }
                for (fv, ft) in rec.fields.iter().zip(fields.iter()) {
                    stack.push((fv, &ft.ty, depth + 1));
                }
                true
            }
            // Whitelist rejections fire BEFORE the generic mismatch arm: a
            // borrow category is unsupported at any depth even when the
            // declared type would otherwise disagree (§5.1.1, V-12).
            (Value::StrView(_), _) => {
                return Err(DynPackError::Unsupported {
                    category: "str-view",
                    depth,
                })
            }
            (Value::MutRef(_), _) => {
                return Err(DynPackError::Unsupported {
                    category: "mut-ref",
                    depth,
                })
            }
            _ => false,
        };
        if !ok {
            return Err(DynPackError::TypeMismatch {
                expected: t.describe(),
                actual: v.type_name().to_string(),
                depth,
            });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared size accounting (same cost model as Value::deep_size, computed
// iteratively by the callers so every node is charged exactly ONCE: one
// `Value` cell per node, one Vec header per record level, Str payload
// bytes at their leaf)
// ---------------------------------------------------------------------------

pub(crate) fn byte_cost_cell() -> usize {
    std::mem::size_of::<Value>()
}

pub(crate) fn byte_cost_record_extra() -> usize {
    std::mem::size_of::<Vec<Value>>()
}

pub(crate) fn leaf_cost(v: &Value) -> usize {
    byte_cost_cell()
        + match v {
            Value::Str(s) => s.bytes().len(),
            _ => 0,
        }
}

fn clone_leaf(v: &Value) -> Value {
    match v {
        Value::Str(h) => Value::Str(StrHandle::detached(h.bytes().to_vec().into_boxed_slice())),
        other => other.clone(),
    }
}
