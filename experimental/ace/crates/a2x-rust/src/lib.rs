//! PLAN-892 first real A2X target provider: Rust (Design51 AX-A), widened
//! by PLAN-902 (AX-B) to the record/text/text-record source surfaces.
//!
//! Consumes ONLY the shared semantic view (`ace_a2x_core::SemanticView`)
//! and lowers it to a typed Rust program — explicit checked arithmetic with
//! `Result`-based trap propagation (`AutoTrap`), source-order argument
//! evaluation pinned by temporaries, short-circuit preserved as lazy
//! `if`-expressions, nominal records with mode-legalized borrows, owned
//! text with explicit verifier-scheduled cleanup — then assembles the full
//! emit file set (`generated.rs` + `a2x_runtime.rs` + `main.rs` wrapper +
//! manifest).
//!
//! Per plan §2: the Rust-facing representation is `Result<T, AutoTrap>`-
//! shaped (explicit checked ops); it is NOT the mn private ABI and does NOT
//! use the old a2r i64 wrapper. Target-agnostic preparation lives in
//! a2x-core; Rust legalization/type mapping/printing lives here. The
//! scalar channel (`auto.a2x.rust.fileset/1`, `ac-a2r/hir-scalar-emit-v1`)
//! and the record-text channel (`auto.a2x.rust.record-text.fileset/1`,
//! `ac-a2r/hir-record-text-emit-v1`) are parallel and mutually untrusted.

pub mod artifacts;
pub mod cert;
pub mod coll;
pub mod lower;
pub mod lower_coll;
pub mod print;
/// PLAN-921: the Rust [`ace_a2x_core::target::TargetProvider`]
/// implementation (byte-identical relocation of the session's former
/// Rust-coupled produce/fileset logic).
pub mod provider;
pub mod rt;
pub mod target;

pub use artifacts::{assemble_file_set, runtime_source, FileSet};
pub use cert::{
    BoundFile, CertDraft, ToolchainIdentity, CERT_SCHEMA, RECIPE, RT_CERT_SCHEMA, RT_RECIPE,
};
pub use coll::{
    assemble_collections_file_set, canonical_execution_config, collections_runtime_source,
    CollFileSet, COLL_CERT_SCHEMA, COLL_FILESET_SCHEMA, COLL_RECIPE,
};
pub use lower::{
    lower_functions, lower_program, CmpOp, RBlock, RExpr, RStmt, RustFn, RustParam, RustProgram,
    RustRecord,
};
pub use lower_coll::{lower_collections_program, rust_ty_str};
pub use print::print_program;
pub use provider::{RustTargetProvider, RUST_PRINT_VERSION};
pub use rt::{
    assemble_record_text_file_set, record_text_runtime_source, RtFileSet, RT_FILESET_SCHEMA,
};
pub use target::TargetProfile;
