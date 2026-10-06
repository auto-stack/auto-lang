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

pub mod ast;
pub mod lexer;
pub mod parser;

pub use parser::parse_program;
