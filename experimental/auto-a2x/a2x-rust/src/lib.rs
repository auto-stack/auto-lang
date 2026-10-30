//! PLAN-892 first real A2X target provider: Rust (Design51 AX-A).
//!
//! Consumes ONLY the shared semantic view (`auto_a2x_core::SemanticView`)
//! and lowers it to a typed Rust program — explicit checked arithmetic with
//! `Result`-based trap propagation (`AutoTrap`), source-order argument
//! evaluation pinned by temporaries, short-circuit preserved as lazy
//! `if`-expressions — then assembles the full emit file set
//! (`generated.rs` + `a2x_runtime.rs` + `main.rs` wrapper + manifest).
//!
//! Per plan §2: the Rust-facing representation is `Result<i32/bool,
//! AutoTrap>`-shaped (explicit checked ops); it is NOT the mn private ABI
//! and does NOT use the old a2r i64 wrapper. Target-agnostic preparation
//! lives in a2x-core; Rust legalization/type mapping/printing lives here.

pub mod artifacts;
pub mod lower;
pub mod print;
pub mod target;

pub use artifacts::{assemble_file_set, FileSet};
pub use lower::{lower_program, CmpOp, RBlock, RExpr, RStmt, RustFn, RustProgram};
pub use print::print_program;
pub use target::TargetProfile;
