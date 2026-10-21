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
    compile_parts, compile_parts_with, compile_program_parts, compile_record_source,
    compile_scalar_source, compile_source, compile_text_record_source, compile_text_source,
    emit_function_body, BodyEmitReceipt, CompiledSource, SourceParts, RUNTIME_TEXT_CAPABILITY,
};
pub use parser::{parse_program, parse_program_with};
pub use resolve::{
    env_fn_count, env_note_cold_walks, recheck_function, resolve_env, resolve_with, CheckedEnv,
};
pub use typecheck::{check_body_in, typecheck_with};

/// The core source profile (PLAN-745): `+ * <`, negative literals only as
/// `-` before digits, `while` conditional loops.
pub const SOURCE_PROFILE: &str = "auto.source.core-i32.draft";

/// The scalar source profile (PLAN-767): the core surface plus
/// `- / % > <= >= == !=` (binary), unary `- !`, `&& ||` short-circuit, and
/// conditional `for <bool-expr> { ... }` loops (`while` is rejected here).
pub const SCALAR_SOURCE_PROFILE: &str = "auto.source.scalar-i32.draft";

/// The text source profile (PLAN-770): the scalar surface plus str/Str/char
/// types, string/char literals (with the 774 escape whitelist and cross-line
/// string literals whose line breaks enter the value verbatim), the text
/// method set (`len/len_bytes/char_at/slice/to/clone/to_codepoint`,
/// `char.from_codepoint`), text `+` concatenation, text/char `==`/`!=`,
/// parameter mode words (`mut`/`move`) and call-site `.move`.
pub const TEXT_SOURCE_PROFILE: &str = "auto.source.text.draft";
/// The record source profile (PLAN-771): the scalar surface plus nominal
/// records — `type` declarations with fields and `static fn`/`fn` methods,
/// positional/named construction, nested field paths, readonly instance
/// receivers, and explicit `mut` free-function params (774 D-04).
pub const RECORD_SOURCE_PROFILE: &str = "auto.source.record.draft";
/// The text-record source profile (PLAN-782): the union of the text and
/// record surfaces — nominal records owning `Str` (and carrying `char`)
/// fields, aggregate explicit `.move`/`.clone()`, field reads/replacement,
/// plus the whole scalar/text operator and method surface.
pub const TEXT_RECORD_SOURCE_PROFILE: &str = "auto.source.text-record.draft";

/// The explicitly selected source profile of a compilation. Never inferred
/// from file extensions or content.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SourceProfile {
    Core,
    Scalar,
    Text,
    Record,
    TextRecord,
}

impl SourceProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceProfile::Core => SOURCE_PROFILE,
            SourceProfile::Scalar => SCALAR_SOURCE_PROFILE,
            SourceProfile::Text => TEXT_SOURCE_PROFILE,
            SourceProfile::Record => RECORD_SOURCE_PROFILE,
            SourceProfile::TextRecord => TEXT_RECORD_SOURCE_PROFILE,
        }
    }

    /// The HIR identity domain documents of this profile compile into.
    pub fn hir_domain(&self) -> crate::hir::HirDomain {
        match self {
            SourceProfile::Core => crate::hir::HirDomain::Core,
            SourceProfile::Scalar => crate::hir::HirDomain::Scalar,
            SourceProfile::Text => crate::hir::HirDomain::Text,
            SourceProfile::Record => crate::hir::HirDomain::Record,
            SourceProfile::TextRecord => crate::hir::HirDomain::TextRecord,
        }
    }

    /// Whether the scalar-only operator surface (new binary operators,
    /// unary ops, logical short-circuit) and the conditional `for` loop
    /// are accepted.
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            SourceProfile::Scalar | SourceProfile::Text | SourceProfile::TextRecord
        )
    }

    /// Whether the text surface (str/Str/char, literals, text methods,
    /// mode words) is accepted (PLAN-770).
    pub fn is_text(&self) -> bool {
        matches!(self, SourceProfile::Text | SourceProfile::TextRecord)
    }

    /// Whether the record surface (type declarations, construction, field
    /// paths, methods, `mut` params — PLAN-771) is accepted.
    pub fn is_record(&self) -> bool {
        matches!(self, SourceProfile::Record | SourceProfile::TextRecord)
    }

    /// The widest operator surface this profile accepts: record builds on
    /// the scalar surface.
    pub fn surface_is_scalar_or_wider(&self) -> bool {
        // every NON-CORE profile builds on the scalar operator surface:
        // scalar itself, text (PLAN-770) and record (PLAN-771)
        matches!(
            self,
            SourceProfile::Scalar
                | SourceProfile::Text
                | SourceProfile::Record
                | SourceProfile::TextRecord
        )
    }

    pub fn of(s: &str) -> Option<SourceProfile> {
        match s {
            SOURCE_PROFILE => Some(SourceProfile::Core),
            SCALAR_SOURCE_PROFILE => Some(SourceProfile::Scalar),
            TEXT_SOURCE_PROFILE => Some(SourceProfile::Text),
            RECORD_SOURCE_PROFILE => Some(SourceProfile::Record),
            TEXT_RECORD_SOURCE_PROFILE => Some(SourceProfile::TextRecord),
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
