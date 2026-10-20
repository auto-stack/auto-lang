//! Frozen contract identities, budgets and the outcome taxonomy
//! (PLAN-798 §3/§5.1, frozen by the T-01 `contract-decision.md` gate).
//!
//! These constants are EXPERIMENTAL probe policy for this plan, not a
//! permanent language-syntax ceiling: they bound the limited
//! `av.core-i32/1` profile and its verification/execution admissions.
//! Exceeding a table budget is a *resource* rejection, never an illegal-
//! syntax or unknown-opcode classification.

use std::fmt;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Identities
// ---------------------------------------------------------------------------

/// Logical schema identity of the Typed MIR produced by this probe.
pub const MIR_SCHEMA: &str = "auto.mir.av-probe/1";
/// Wire schema identity of the ABC format emitted/consumed by this probe.
/// /2 (F798-R12-03): the wire carries the unit/module KIND bit
/// (`FLAG_IS_UNIT`) as load-bearing semantics — kind-less /1 artifacts are
/// a different format generation and can never be read as /2 payloads.
pub const ABC_SCHEMA: &str = "auto.abc.av-probe/2";
/// The one execution profile this probe implements.
pub const AV_PROFILE: &str = "av.core-i32/1";
/// Producer identity used in store envelopes and keys (PLAN-798 §5.4).
pub const PRODUCER_ID: &str = "plan798/av-probe";

/// Semantic-version tuple carriers. A change in any of these values must
/// invalidate previously persisted artifacts of that stage (probe cache key
/// component; the numeric values are frozen by T-01).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MirSemanticsVersion(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AbcSemanticsVersion(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AvSemanticsVersion(pub u32);

pub const MIR_SEMANTICS: MirSemanticsVersion = MirSemanticsVersion(1);
/// 2 (F798-R12-03): ABC semantics distinguish unit vs complete-module
/// kind; 1 = the kind-less generation whose artifacts must miss/reject.
pub const ABC_SEMANTICS: AbcSemanticsVersion = AbcSemanticsVersion(2);
pub const AV_SEMANTICS: AvSemanticsVersion = AvSemanticsVersion(1);

/// Capability that admits the closed test-only trace intrinsic set
/// (same string as the HIR front door, `hir.test.trace.v1`).
pub const TRACE_CAPABILITY: &str = "hir.test.trace.v1";

// ---------------------------------------------------------------------------
// Frozen budgets (PLAN-798 §3; resource rejection on breach)
// ---------------------------------------------------------------------------

/// Maximum size of one source / HIR / ABC *input* presented to the probe.
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;
/// Maximum bytes in one codec page (encode flush / decode feed granularity).
pub const MAX_CODEC_PAGE_BYTES: usize = 256 * 1024;
/// Total table entries (types + consts + imports + functions + blocks +
/// slots + instructions) allowed in one MIR/ABC unit before allocation.
pub const MAX_TABLE_ENTRIES: usize = 65_536;
/// Nesting budget enforced at the OLD source/HIR text front door, before the
/// legacy recursive parser runs (probe admission, not a language rule).
pub const MAX_FRONTEND_NESTING: u32 = 128;
/// MIR/ABC verification work-step ceiling per request.
pub const MAX_VERIFY_WORK_STEPS: u64 = 2_000_000;
/// MIR/ABC verification wall ceiling per request.
pub const MAX_VERIFY_WALL: Duration = Duration::from_millis(5_000);
/// AV execution instruction-step ceiling per request.
pub const MAX_AV_INSTR_STEPS: u64 = 1_000_000;
/// AV guest frame ceiling (guest recursion never rides the host stack).
pub const MAX_AV_FRAMES: usize = 1024;
/// AV live slot-data ceiling across all frames.
pub const MAX_AV_SLOT_BYTES: usize = 8 * 1024 * 1024;
/// AV execution wall ceiling per request.
pub const MAX_AV_WALL: Duration = Duration::from_millis(2_000);
/// Instructions between cancellation-flag checks.
pub const CANCEL_CHECK_INTERVAL: u64 = 1024;
/// Test-trace byte ceiling for one execution.
pub const MAX_TRACE_BYTES: usize = 64 * 1024;

/// Slot cell size used for the live-slot-bytes budget accounting.
pub const SLOT_CELL_BYTES: usize = std::mem::size_of::<Value>();

/// Upper bound on the verifier's composite analysis allocations (the
/// definite-initialization bitsets over blocks × slot words, per function),
/// checked BEFORE those structures are allocated (F798-R3-08: a dense
/// 16384-slot/8192-block unit previously allocated ~51.7 MiB of matrices
/// before the work-step budget rejected it). Part of the 16 MiB staging
/// partition of the managed 32 MiB envelope.
pub const MAX_VERIFY_ANALYSIS_BYTES: usize = 16 * 1024 * 1024;
/// Upper bound on one decoded module's structural heap estimate, checked
/// at admission before deep verification (wire length is not a working-set
/// proxy; a tiny wire can declare large tables).
pub const MAX_DECODED_MODULE_BYTES: usize = 16 * 1024 * 1024;

// ---------------------------------------------------------------------------
// Values and traps
// ---------------------------------------------------------------------------

/// A scalar value of the limited profile. `bool` is not an arbitrary
/// integer: it keeps its own kind end to end.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Value {
    I32(i32),
    Bool(bool),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::I32(_) => "i32",
            Value::Bool(_) => "bool",
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::I32(v) => Some(*v),
            Value::Bool(_) => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            Value::I32(_) => None,
        }
    }
}

/// Language traps of the scalar contract (i32 arithmetic). These are
/// *runtime* outcomes; a trap on an unexecuted branch must stay latent —
/// it is never folded into a compile-time rejection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrapKind {
    /// add/sub/mul signed overflow (trap policy is implicit in the profile).
    IntOverflow,
    /// `x / 0`.
    DivByZero,
    /// `MIN / -1`.
    DivOverflow,
    /// `MIN % -1` (traps per the scalar contract, unlike host `%`).
    RemOverflow,
}

impl TrapKind {
    pub fn code(&self) -> &'static str {
        match self {
            TrapKind::IntOverflow => "av.trap.int-overflow",
            TrapKind::DivByZero => "av.trap.div-by-zero",
            TrapKind::DivOverflow => "av.trap.div-overflow",
            TrapKind::RemOverflow => "av.trap.rem-overflow",
        }
    }
}

/// Resource-limit classification (distinct from language traps and from
/// infrastructure failures). Host stack overflows and process kills are
/// never valid members of this taxonomy.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceKind {
    InputTooLarge,
    CodecPage,
    TableEntries,
    VerifySteps,
    VerifyWall,
    InstrSteps,
    Frames,
    SlotBytes,
    ExecutionWall,
    TraceBytes,
    /// Composite analysis allocations (init-dataflow bitsets etc.).
    VerifyMemory,
    /// Decoded module structural heap estimate.
    DecodeMemory,
}

impl ResourceKind {
    pub fn code(&self) -> &'static str {
        match self {
            ResourceKind::InputTooLarge => "av.resource.input-size",
            ResourceKind::CodecPage => "av.resource.codec-page",
            ResourceKind::TableEntries => "av.resource.table-entries",
            ResourceKind::VerifySteps => "av.resource.verify-steps",
            ResourceKind::VerifyWall => "av.resource.verify-wall",
            ResourceKind::InstrSteps => "av.resource.instr-steps",
            ResourceKind::Frames => "av.resource.frames",
            ResourceKind::SlotBytes => "av.resource.slot-bytes",
            ResourceKind::ExecutionWall => "av.resource.execution-wall",
            ResourceKind::TraceBytes => "av.resource.trace-bytes",
            ResourceKind::VerifyMemory => "av.resource.verify-memory",
            ResourceKind::DecodeMemory => "av.resource.decode-memory",
        }
    }
}

/// Outcome taxonomy of one AV execution (success value / language trap /
/// resource / cancellation / infra are distinct states; unsupported and the
/// two admission rejections happen before execution starts).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecOutcome {
    Success {
        value: Value,
        steps: u64,
    },
    Trap {
        kind: TrapKind,
        span: Option<SpanInfo>,
    },
    Resource {
        kind: ResourceKind,
        message: String,
    },
    Cancelled,
    /// Executor-internal failure (budget bookkeeping etc.) — never used to
    /// absorb a language trap or a resource classification.
    Infra(String),
}

/// Source-mapping record attached to traps and diagnostics: the owning
/// function plus a byte span into that function's source snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpanInfo {
    pub func: u32,
    pub start: u32,
    pub end: u32,
}

impl fmt::Display for SpanInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func#{}:{}..{}", self.func, self.start, self.end)
    }
}

/// FNV-1a 64-bit — the content-fingerprint primitive used by the codec
/// trailer, envelopes and unit keys. A change detector, not cryptography.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// FNV-1a 64-bit over several parts (domain-separated by hashing lengths).
pub fn fnv1a64_parts(parts: &[&[u8]]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        h ^= p.len() as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        for b in *p {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}
