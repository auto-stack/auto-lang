//! PLAN-823 owned closure environments (T-10 module subset; r2: integrated
//! clone delegation). Same-source module: the T-10 harness
//! (`experimental/ac-av-probe/parallel/823`) compiles THIS file via
//! `#[path]` against the delivered auto-av-core PUBLIC API; after the
//! core window registers `mod closures;` in core's lib.rs, the same file
//! compiles unchanged inside auto-av-core (single source, no copy).
//!
//! Capture law (§5.1/§5.5): escape capture is move or EXPLICIT clone only
//! — borrow kinds (StrView/MutRef, and PLAN-822's frame-rooted Place
//! tokens) are rejected at prepare BEFORE any allocation attempt
//! (attempt=0 law); a Closure value itself is not capturable in the first
//! case (nested closure capture is a T-03/T-04 surface with store-aware
//! cleanup). All fallible work (clone allocations, budget walks) happens
//! in `prepare`; `commit` is a pure validate-and-transfer (no allocation,
//! no callback, no cancellation observation). A prepare failure drops
//! already-staged clones in REVERSE staging order and never consumes a
//! to-move source (prepare only borrows; move sources transfer at
//! commit). Record/List/Variant interiors are treated as opaque owned
//! aggregates admitted upstream by `verify_mir`.
//!
//! Deep clone is DELEGATED (r2 convergence decision): the canonical
//! implementation is `av::clone_owned_value` (822-delivered public
//! explicit-stack visitor with cancel/wall/fuel discipline and
//! partial-output rollback). The T-10-era local walk is retired — one
//! implementation, pinned by the 822 clone-family replays plus the V-14
//! module cases.

use crate::av::{clone_owned_value, CancelFlag};
use crate::contract::{ExecOutcome, Value, MAX_AV_SLOT_BYTES};
use crate::owned_text::StrArena;

/// Re-export of the frozen env-field budget (contract.rs is the house of
/// budgets; the path `closures::MAX_ENV_FIELDS` stays stable).
pub use crate::contract::MAX_ENV_FIELDS;

/// Identity of this module subset — deliberately NOT a registered profile
/// string; `av.closures/1` registration lives with the core profile gate
/// (contract-decision r2 §2.2).
pub const MODULE_SUBSET: &str = "plan823/closures-module-subset/1";

/// Escape-capture mode (§5.1.3: borrow capture is not in this subset).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureMode {
    /// Whole-root transfer; happens at COMMIT (prepare only borrows).
    Move,
    /// Explicit deep clone: every reachable owned buffer gets a fresh
    /// independent allocation; staged at PREPARE (fallible through the
    /// arena fault seam); the source stays with its creator.
    Clone,
    /// BORROW capture (T-03, §5.1.3): the source must be an owned Str
    /// (the delivered borrow domain); the STAGED field is a share view
    /// minted by the CALLER over the creator's slot (the executor mints
    /// it; the module validates the source kind and consumes the staged
    /// value). Views hold no drop right and stage infallibly.
    Borrow,
}

/// One capture source handed to prepare (borrowed — never owned here).
#[derive(Clone, Copy)]
pub struct CaptureSpec<'a> {
    pub mode: CaptureMode,
    pub src: &'a Value,
}

/// The clone-walk budget/cancel context: the executor passes its real
/// dispatch discipline (cancel flag, remaining wall allowance, shared
/// cumulative fuel); the module subset passes a no-cancel default.
pub struct CloneContext<'a> {
    pub cancel: &'a CancelFlag,
    pub wall_left: std::time::Duration,
    pub fuel: &'a mut u64,
    pub max_fuel: u64,
}

/// Module-level budget slice of the frozen resource table (the executor
/// charges the full envelope; these bound one env construction).
#[derive(Clone, Copy, Debug)]
pub struct EnvLimits {
    pub max_fields: usize,
    pub max_env_bytes: usize,
}

impl Default for EnvLimits {
    fn default() -> Self {
        EnvLimits {
            max_fields: MAX_ENV_FIELDS,
            max_env_bytes: MAX_AV_SLOT_BYTES,
        }
    }
}

/// Mirror of the `Value` discriminant: binds prepare-time borrows to
/// commit-time owned sources (same kind, possibly different content —
/// content drift is re-budgeted at commit).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueKind {
    I32,
    Bool,
    I64,
    U64,
    F32,
    F64,
    Record,
    Char,
    Str,
    StrView,
    MutRef,
    List,
    Variant,
    Place,
    Closure,
    Dyn,
}

impl ValueKind {
    fn name(self) -> &'static str {
        match self {
            ValueKind::I32 => "i32",
            ValueKind::Bool => "bool",
            ValueKind::I64 => "i64",
            ValueKind::U64 => "u64",
            ValueKind::F32 => "f32",
            ValueKind::F64 => "f64",
            ValueKind::Record => "record",
            ValueKind::Char => "char",
            ValueKind::Str => "Str",
            ValueKind::StrView => "str",
            ValueKind::MutRef => "mut-ref",
            ValueKind::List => "list",
            ValueKind::Variant => "variant",
            ValueKind::Place => "place",
            ValueKind::Closure => "closure",
            ValueKind::Dyn => "dyn",
        }
    }
}

fn value_kind(v: &Value) -> ValueKind {
    match v {
        Value::I32(_) => ValueKind::I32,
        Value::Bool(_) => ValueKind::Bool,
        Value::I64(_) => ValueKind::I64,
        Value::U64(_) => ValueKind::U64,
        Value::F32(_) => ValueKind::F32,
        Value::F64(_) => ValueKind::F64,
        Value::Record(_) => ValueKind::Record,
        Value::Char(_) => ValueKind::Char,
        Value::Str(_) => ValueKind::Str,
        Value::StrView(_) => ValueKind::StrView,
        Value::MutRef(_) => ValueKind::MutRef,
        Value::List(_) => ValueKind::List,
        Value::Variant(_) => ValueKind::Variant,
        Value::Place(_) => ValueKind::Place,
        Value::Closure(_) => ValueKind::Closure,
        Value::Dyn(_) => ValueKind::Dyn,
    }
}

/// The kinds an escape capture NEVER admits: borrowed views and write
/// tokens (§5.1.3), PLAN-822's frame-rooted place tokens, and (first-case
/// boundary) closure values themselves.
fn escape_admissible(kind: ValueKind) -> bool {
    !matches!(
        kind,
        ValueKind::StrView
            | ValueKind::MutRef
            | ValueKind::Place
            | ValueKind::Closure
            | ValueKind::Dyn
    )
}

/// Prepare-phase rejection taxonomy (resource / unsupported / fault /
/// cancelled are distinct states; never a panic or timeout).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvPrepareError {
    TooManyFields {
        got: usize,
        max: usize,
    },
    EnvBytesBudget {
        would: usize,
        max: usize,
    },
    UnsupportedKind {
        kind: &'static str,
    },
    /// The arena fault seam fired during a capture clone. `attempt` is the
    /// arena's attempt counter at the fault (try_alloc increments before
    /// the fault check, so the counter IS the faulted attempt number).
    FaultedClone {
        attempt: u64,
        message: String,
    },
    /// A capture staging allocation could not be reserved.
    Allocation {
        what: &'static str,
    },
    /// The clone walk observed the cancellation flag (§5.5: no
    /// cancellation observation inside commit — this can only happen in
    /// prepare).
    Cancelled,
    /// The clone walk exceeded its fuel/wall allowance.
    CloneBudget {
        code: &'static str,
        message: String,
    },
    /// A Borrow capture arrived without its caller-minted staged view —
    /// an executor-side contract violation (Infra class), never a
    /// language behavior.
    BorrowStaging(String),
}

impl EnvPrepareError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::TooManyFields { .. } => "env.capture.too-many-fields",
            Self::EnvBytesBudget { .. } => "env.capture.bytes-budget",
            Self::UnsupportedKind { .. } => "env.capture.unsupported-kind",
            Self::FaultedClone { .. } => "env.capture.faulted-clone",
            Self::Allocation { .. } => "env.capture.allocation",
            Self::Cancelled => "env.capture.cancelled",
            Self::CloneBudget { .. } => "env.capture.clone-budget",
            Self::BorrowStaging(_) => "env.capture.borrow-staging",
        }
    }
}

/// Commit-phase rejection taxonomy (all pre-commit: sources are returned
/// UNCONSUMED and staged clones drop in reverse staging order).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvCommitError {
    SourceCountMismatch {
        expected: usize,
        got: usize,
    },
    SourceKind {
        index: usize,
        expected: &'static str,
        got: &'static str,
    },
    EnvBytesBudget {
        would: usize,
        max: usize,
    },
}

impl EnvCommitError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::SourceCountMismatch { .. } => "env.commit.source-count",
            Self::SourceKind { .. } => "env.commit.source-kind",
            Self::EnvBytesBudget { .. } => "env.commit.bytes-budget",
        }
    }
}

/// Fuel/byte accounting returned to the caller (the executor charges it;
/// `bytes` follows `Value::deep_size`, `fields` counts traversed nodes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnvAccounting {
    pub bytes: usize,
    pub fields: u64,
}

/// One staged capture: a validated move kind, or a clone whose deep copy
/// was already materialized at prepare.
#[derive(Debug)]
enum PlannedCapture {
    Move { kind: ValueKind },
    Clone { staged: Value },
}

/// The prepared-but-uncommitted environment (prepare output). Dropping a
/// plan — including every failure path below — releases staged clones in
/// REVERSE staging order (the rollback law).
#[derive(Debug)]
pub struct EnvPlan {
    limits: EnvLimits,
    planned: Vec<PlannedCapture>,
    // Storage is reserved during prepare, before any source is moved.
    fields: Vec<Value>,
    moved: Vec<bool>,
}

impl Drop for EnvPlan {
    fn drop(&mut self) {
        while let Some(p) = self.planned.pop() {
            drop(p);
        }
    }
}

impl EnvPlan {
    /// Phase 1 (fallible). Pass 1 is pure admission over ALL specs —
    /// field-count budget, per-spec kind gate, prospective total env size
    /// (a clone mirrors its source's deep size; a move transfers the
    /// source itself) — with ZERO arena attempts. Pass 2 stages clones in
    /// capture order through the shared clone visitor (fault/cancel/wall
    /// are its failure modes). Sources are borrowed throughout and never
    /// consumed.
    pub fn prepare(
        arena: &StrArena,
        specs: &[CaptureSpec<'_>],
        limits: &EnvLimits,
        ctx: &mut CloneContext<'_>,
        staged_borrows: Vec<Value>,
    ) -> Result<(EnvPlan, EnvAccounting), EnvPrepareError> {
        if specs.len() > limits.max_fields {
            return Err(EnvPrepareError::TooManyFields {
                got: specs.len(),
                max: limits.max_fields,
            });
        }
        let mut acc = EnvAccounting::default();
        for spec in specs {
            let kind = value_kind(spec.src);
            // BORROW captures check against the delivered borrow domain
            // (an owned Str source); escape admissibility applies to the
            // MOVE/CLONE domain.
            if spec.mode == CaptureMode::Borrow {
                if kind != ValueKind::Str {
                    return Err(EnvPrepareError::UnsupportedKind {
                        kind: spec.src.type_name(),
                    });
                }
            } else if !escape_admissible(kind) {
                return Err(EnvPrepareError::UnsupportedKind {
                    kind: spec.src.type_name(),
                });
            }
            acc.bytes += spec.src.deep_size(&mut acc.fields);
        }
        if acc.bytes > limits.max_env_bytes {
            return Err(EnvPrepareError::EnvBytesBudget {
                would: acc.bytes,
                max: limits.max_env_bytes,
            });
        }
        let mut planned = Vec::new();
        planned
            .try_reserve_exact(specs.len())
            .map_err(|_| EnvPrepareError::Allocation {
                what: "capture plan",
            })?;
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(specs.len())
            .map_err(|_| EnvPrepareError::Allocation {
                what: "environment fields",
            })?;
        let mut moved = Vec::new();
        moved
            .try_reserve_exact(specs.len())
            .map_err(|_| EnvPrepareError::Allocation {
                what: "environment field states",
            })?;
        moved.resize(specs.len(), false);
        let mut plan = EnvPlan {
            limits: *limits,
            planned,
            fields,
            moved,
        };
        let mut borrows = staged_borrows.into_iter();
        for spec in specs {
            let kind = value_kind(spec.src);
            match spec.mode {
                CaptureMode::Move => plan.planned.push(PlannedCapture::Move { kind }),
                CaptureMode::Clone => {
                    let staged = match deep_clone_value(spec.src, arena, ctx) {
                        Ok((v, a)) => {
                            acc.fields += a.fields;
                            v
                        }
                        Err(e) => {
                            // plan drops on this return: staged clones so
                            // far release in reverse staging order.
                            return Err(e);
                        }
                    };
                    plan.planned.push(PlannedCapture::Clone { staged });
                }
                CaptureMode::Borrow => {
                    // Infallible staging: the caller-minted share view
                    // carries no drop right; a missing staged value is a
                    // caller-side contract violation (never language
                    // behavior).
                    let staged = borrows.next().ok_or_else(|| {
                        EnvPrepareError::BorrowStaging(
                            "a Borrow capture arrived without a staged view".to_string(),
                        )
                    })?;
                    plan.planned.push(PlannedCapture::Clone { staged });
                }
            }
        }
        Ok((plan, acc))
    }

    /// Phase 2 (pure validate-and-transfer): the k-th source binds to the
    /// k-th Move spec; clone sources stay with their creator and are never
    /// handed in. No allocation, no callback, no cancellation observation.
    /// On any rejection the sources come back UNCONSUMED (test-readable,
    /// caller-owned) and the staged clones drop in reverse staging order.
    pub fn commit(
        mut self,
        move_sources: Vec<Value>,
    ) -> Result<(ClosureEnv, EnvAccounting), (EnvCommitError, Vec<Value>)> {
        let expected_moves = self
            .planned
            .iter()
            .filter(|p| matches!(p, PlannedCapture::Move { .. }))
            .count();
        if move_sources.len() != expected_moves {
            return Err((
                EnvCommitError::SourceCountMismatch {
                    expected: expected_moves,
                    got: move_sources.len(),
                },
                move_sources,
            ));
        }
        // Kind binding + final size: a same-kind source may carry different
        // content than the prepare-time borrow, so the total is recomputed.
        let mut acc = EnvAccounting::default();
        {
            let mut next_move = move_sources.iter();
            for (i, p) in self.planned.iter().enumerate() {
                match p {
                    PlannedCapture::Move { kind } => {
                        let src = next_move.next().expect("count matched above");
                        let got = value_kind(src);
                        if got != *kind {
                            return Err((
                                EnvCommitError::SourceKind {
                                    index: i,
                                    expected: kind.name(),
                                    got: src.type_name(),
                                },
                                move_sources,
                            ));
                        }
                        acc.bytes += src.deep_size(&mut acc.fields);
                    }
                    PlannedCapture::Clone { staged, .. } => {
                        acc.bytes += staged.deep_size(&mut acc.fields);
                    }
                }
            }
        }
        if acc.bytes > self.limits.max_env_bytes {
            return Err((
                EnvCommitError::EnvBytesBudget {
                    would: acc.bytes,
                    max: self.limits.max_env_bytes,
                },
                move_sources,
            ));
        }
        // Commit proper — nothing can fail past this point. `self` still
        // drops at scope end; taking the plan leaves its Drop a no-op.
        let planned = std::mem::take(&mut self.planned);
        let mut fields = std::mem::take(&mut self.fields);
        let mut next_move = move_sources.into_iter();
        for p in planned.into_iter() {
            match p {
                PlannedCapture::Move { .. } => {
                    fields.push(next_move.next().expect("count matched above"))
                }
                PlannedCapture::Clone { staged, .. } => fields.push(staged),
            }
        }
        let moved = std::mem::take(&mut self.moved);
        Ok((ClosureEnv { fields, moved }, acc))
    }
}

/// The owned closure environment: capture-order fields (§5.1.1 typed env;
/// the type identity binding is the executor's ClosureRef). The env is an
/// owned aggregate: fields release in REVERSE capture order (creation
/// LIFO, the 821/822 cleanup law); each buffer reports its free exactly
/// once through the delivered handle drops.
#[derive(Debug)]
pub struct ClosureEnv {
    fields: Vec<Value>,
    /// R1-F01: per-field moved-out flags — a MoveFromEnv capture TAKES the
    /// field (the slot holds a placeholder); reads of a moved field are
    /// rejected (verify gate first, runtime defense second).
    moved: Vec<bool>,
}

impl ClosureEnv {
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// The LIVE field value — `None` when out of range OR moved out.
    pub fn field(&self, i: usize) -> Option<&Value> {
        if self.moved.get(i).copied().unwrap_or(false) {
            return None;
        }
        self.fields.get(i)
    }

    /// R1-F01: take ONE field out for a MoveFromEnv capture — the value
    /// transfers to the child env; a placeholder + moved flag land here.
    /// `None` = already moved (the runtime defense arm of the verify
    /// gate).
    pub fn take_field(&mut self, i: usize) -> Option<Value> {
        if self.moved.get(i).copied().unwrap_or(false) || i >= self.fields.len() {
            return None;
        }
        self.moved[i] = true;
        Some(std::mem::replace(&mut self.fields[i], Value::I32(0)))
    }

    /// Deep data-size estimate over all fields — same rule as
    /// `Value::deep_size` (per-field accounting, conservative on shares).
    pub fn deep_size(&self, visited: &mut u64) -> usize {
        self.fields.iter().map(|f| f.deep_size(visited)).sum()
    }

    /// Surrender the fields to the integration owner (the executor's env
    /// release path feeds them to the explicit cleanup worklist); the env
    /// then drops empty — responsibility has left.
    pub fn into_fields(mut self) -> Vec<Value> {
        std::mem::take(&mut self.fields)
    }

    /// Adopt already-materialized fields as one env (capture order; the
    /// executor's closure-CLONE path — §5.1.2 deep clone — builds the
    /// copy's fields through the shared clone visitor and adopts them).
    pub fn from_fields(fields: Vec<Value>) -> Self {
        let moved = vec![false; fields.len()];
        ClosureEnv { fields, moved }
    }
}

impl Drop for ClosureEnv {
    fn drop(&mut self) {
        while let Some(v) = self.fields.pop() {
            drop(v);
        }
    }
}

/// Deep-clone one owned value for a capture — a DELEGATION to the
/// canonical visitor `av::clone_owned_value` (r2 convergence: one
/// implementation; explicit stack, cancel/wall/fuel discipline,
/// partial-output rollback, a fresh allocation per reachable owned
/// Str/List). Returns the clone plus its byte/field accounting.
pub fn deep_clone_value(
    val: &Value,
    arena: &StrArena,
    ctx: &mut CloneContext<'_>,
) -> Result<(Value, EnvAccounting), EnvPrepareError> {
    let cloned = match clone_owned_value(
        val,
        arena,
        ctx.cancel,
        ctx.wall_left,
        ctx.fuel,
        ctx.max_fuel,
    ) {
        Ok(v) => v,
        Err(ExecOutcome::Infra(message)) => {
            // The arena's attempt counter IS the faulted attempt number
            // (try_alloc increments before the fault check).
            return Err(EnvPrepareError::FaultedClone {
                attempt: arena.attempts(),
                message,
            });
        }
        Err(ExecOutcome::Cancelled) => return Err(EnvPrepareError::Cancelled),
        Err(ExecOutcome::Resource { kind, message }) => {
            return Err(EnvPrepareError::CloneBudget {
                code: kind.code(),
                message,
            })
        }
        Err(other) => {
            return Err(EnvPrepareError::CloneBudget {
                code: "av.infra",
                message: format!("{other:?}"),
            })
        }
    };
    let mut accounting = EnvAccounting::default();
    accounting.bytes = cloned.deep_size(&mut accounting.fields);
    Ok((cloned, accounting))
}
