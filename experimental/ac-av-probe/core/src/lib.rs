//! auto-av-core — PLAN-798 correctness layer for the limited AV execution
//! profile (`av.core-i32/1`).
//!
//! Layers, in dependency order:
//! - [`contract`]: frozen identities, budgets and the outcome taxonomy.
//! - [`mir`]: the Typed MIR data model (explicit basic blocks, typed mutable
//!   local/temp slots, unique terminators) plus per-function units.
//! - [`mir_verify`]: the MIR verifier — the ONLY constructor of `CheckedMir`.
//! - [`abc`]: the versioned ABC wire codec (little-endian, paged).
//! - [`abc_verify`]: shape→CFG→init→signature admission producing the
//!   `LoadedModule`; AV executes nothing unverified.
//! - [`av`]: the AV executor with an explicit frame/slot stack (guest
//!   recursion never rides the host stack), fuel, frames, wall, slot-byte and
//!   cancellation budgets.
//!
//! This crate has zero external dependencies; the HIR adapter, cache and
//! native cross-check live in `auto-av-probe`.

pub mod abc;
pub mod abc_verify;
pub mod av;
pub mod contract;
pub mod function_code;
pub mod mir;
pub mod mir_verify;
pub mod program_snapshot;

pub use abc::{decode_abc, encode_abc, AbcDecodeError, DecodedAbc};
pub use abc_verify::{verify_abc, verify_abc_bytes, AbcAdmissionError, LoadedModule};
pub use av::{
    execute, execute_entry, execute_program, execute_program_steps, resume_program,
    CancelCheck, CancelFlag, ExecLimits, ExecTrace, PausedRun, SteppedOutcome,
};
pub use contract::{
    AvSemanticsVersion, ExecOutcome, MirSemanticsVersion, ResourceKind, SpanInfo, TrapKind, Value,
    ABC_SCHEMA, AV_PROFILE, MIR_SCHEMA, PRODUCER_ID,
};
pub use function_code::{FunctionCode, SharedFunctionCode};
pub use mir::{MirModule, MirModuleUncheckedMarker, TraceEvent};
pub use mir_verify::{verify_mir, verify_mir_with_deadline, CheckedMir, MirVerifyError};
pub use program_snapshot::ProgramSnapshot;
