//! # auto-gen-core
//!
//! Interpreter-free generation core (PLAN-791): guarded-section processing,
//! output path/file orchestration, the generic generation error type and the
//! single-run generation report.
//!
//! The core has NO dependency on `auto-lang`/`auto-man`/any interpreter or UI
//! stack and never holds interpreter handles — adapters (like `auto-gen`,
//! the Auto executable-template adapter) hand it owned rendered text through
//! the [`TemplateRenderer`] callback and own every language-specific concern:
//! data loading, library paths, template parsing and execution-error mapping.
//!
//! Semantics frozen at extraction time (2026-10-16, PLAN-791 T-01):
//! - per-template errors are collected into [`GenReport`] and the batch
//!   continues; adapter-side load failures are a top-level error stage that
//!   happens BEFORE [`generate`] is invoked;
//! - an output file is only written after its render and guard merge
//!   succeeded (a failing template never touches an existing output);
//! - `dry_run` performs zero filesystem effects;
//! - [`guard::feed_guarded_legacy`] keeps the deprecated `AutoGen`
//!   byte-level behavior (join without trailing newline) distinct from
//!   [`guard::GuardProcessor::merge`].

pub mod error;
pub mod generate;
pub mod guard;

pub use error::{GenError, GenResult, SourceLocation};
pub use generate::{
    generate, GenReport, OutputConfig, RenderedTemplate, TemplateRenderer, TemplateSpec,
};
pub use guard::{feed_guarded_legacy, Conflict, GuardProcessor, GuardedSection};
