//! Auto source-subset frontends for the `auto.source.core-i32.draft`
//! (PLAN-745) and `auto.source.scalar-i32.draft` (PLAN-767) profiles.
//!
//! Pipeline position (plan §2): `.at` source → S0 spanned tokens/AST
//! (this module: `lexer`/`ast`/`parser`) → signature pre-registration,
//! lexical name resolution and exact type checking (`resolve`, `typecheck`)
//! → Atom text emission with a generated-token → source-span map
//! (`adapter`) → the existing Schema binder (`crate::descriptor`) and
//! semantic verifier (`crate::verify`).
//!
//! The frontend is purely syntactic here: it accepts the statement-oriented
//! grammar of the plan §5.2 with byte spans into the original source and
//! rejects excluded-surface constructs with `source.*` diagnostics at stage
//! `text`. Which operator/loop surface a parse accepts is selected by the
//! explicit [`SourceProfile`] — the core profile's accept/reject sets are
//! unchanged by the scalar profile's wider surface.

pub mod adapter;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod resolve;
pub mod typecheck;

pub use adapter::{
    compile_parts, compile_parts_with, compile_scalar_source, compile_source, CompiledSource,
    SourceParts,
};
pub use parser::{parse_program, parse_program_with};
pub use resolve::resolve_with;
pub use typecheck::typecheck_with;

/// The core source profile (PLAN-745): `+ * <`, negative literals only as
/// `-` before digits, `while` conditional loops.
pub const SOURCE_PROFILE: &str = "auto.source.core-i32.draft";

/// The scalar source profile (PLAN-767): the core surface plus
/// `- / % > <= >= == !=` (binary), unary `- !`, `&& ||` short-circuit, and
/// conditional `for <bool-expr> { ... }` loops (`while` is rejected here).
pub const SCALAR_SOURCE_PROFILE: &str = "auto.source.scalar-i32.draft";

/// The explicitly selected source profile of a compilation. Never inferred
/// from file extensions or content.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceProfile {
    Core,
    Scalar,
}

impl SourceProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceProfile::Core => SOURCE_PROFILE,
            SourceProfile::Scalar => SCALAR_SOURCE_PROFILE,
        }
    }

    /// The HIR identity domain documents of this profile compile into.
    pub fn hir_domain(&self) -> crate::hir::HirDomain {
        match self {
            SourceProfile::Core => crate::hir::HirDomain::Core,
            SourceProfile::Scalar => crate::hir::HirDomain::Scalar,
        }
    }

    /// Whether the scalar-only operator surface (new binary operators,
    /// unary ops, logical short-circuit) and the conditional `for` loop
    /// are accepted.
    pub fn is_scalar(&self) -> bool {
        matches!(self, SourceProfile::Scalar)
    }

    pub fn of(s: &str) -> Option<SourceProfile> {
        match s {
            SOURCE_PROFILE => Some(SourceProfile::Core),
            SCALAR_SOURCE_PROFILE => Some(SourceProfile::Scalar),
            _ => None,
        }
    }
}

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
