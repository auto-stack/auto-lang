//! Auto source-subset frontend for the `auto.source.core-i32.draft`
//! profile (PLAN-745).
//!
//! Pipeline position (plan §2): `.at` source → S0 spanned tokens/AST
//! (this module: `lexer`/`ast`/`parser`) → signature pre-registration,
//! lexical name resolution and exact type checking (`resolve`, `typecheck`)
//! → Atom text emission with a generated-token → source-span map
//! (`adapter`) → the existing Schema binder (`crate::descriptor`) and
//! semantic verifier (`crate::verify`).
//!
//! The frontend is purely syntactic here: it accepts the statement-oriented
//! grammar of plan §5.2 with byte spans into the original source and rejects
//! excluded-surface constructs with `source.*` diagnostics at stage `text`.

pub mod adapter;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod resolve;
pub mod typecheck;

pub use adapter::{compile_source, CompiledSource};
pub use parser::parse_program;

/// The only source profile this frontend accepts. Callers must select it
/// explicitly; the mode is never inferred from file extensions.
pub const SOURCE_PROFILE: &str = "auto.source.core-i32.draft";

/// Capability that activates the closed set of test-only trace intrinsics.
pub const TRACE_CAPABILITY: &str = "hir.test.trace.v1";

/// Map a source-level callee name to its intrinsic symbol. These names
/// resolve only when the caller provided [`TRACE_CAPABILITY`], and user
/// functions and locals always win over them.
pub fn trace_intrinsic(callee: &str) -> Option<&'static str> {
    match callee {
        "mark_a" => Some("hir.test.mark_a"),
        "mark_b" => Some("hir.test.mark_b"),
        _ => None,
    }
}
