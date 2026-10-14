//! # auto-project-model
//!
//! Light static project-model for AutoLang `pac.at` declarations
//! (PLAN-780 pilot). Parses the *frozen static subset* (T-01 report,
//! `test/fixtures/modularity/project-model/survey/T01-REPORT.md`) without
//! the compiler/VM/UI stack: the crate depends only on `auto-atom` /
//! `auto-val`, holds no `Builder`/`IndexStore`/`AutoVM` types, and its read
//! entrypoints have no effects (no network, no writes, no builds).
//!
//! Two invariants carry the pilot's honesty contract:
//!
//! - **No silent field loss.** Unknown top-level keys and unmodeled nodes
//!   (targets, `theme{}` blocks, `npm_deps`, …) are catalogued verbatim on
//!   [`ProjectDescriptor::raw_props`] / [`ProjectDescriptor::raw_nodes`]
//!   with source lines.
//! - **No fake static equivalence.** Constructs only the config VM can
//!   produce (`${}` interpolation, bare variable references, assignments,
//!   calls, f-strings) are classified on
//!   [`ProjectDescriptor::unsupported`]; a descriptor with any entry there
//!   is not static-equivalent ([`ProjectDescriptor::is_static_equivalent`]).
//!   Evaluation-needing inputs keep flowing through the explicit legacy
//!   adapter (`AutoConfig::from_code`), which this crate never replaces.
//!
//! Relative paths declared in the document (`dep path`, `back.project`,
//! `members`) resolve against the pac.at's own directory — see
//! [`SourceInfo`].

pub mod descriptor;
mod from_scan;
pub mod scanner;

pub use descriptor::{
    DependencyDecl, DepNameForm, DisplayFields, DuplicateKey, Identity, ProjectDescriptor,
    RawNodeDecl, RawProp, RenderDecl, RenderSpec, RuntimeFields, SceneKind, SourceInfo,
    ToggleFields, ToggleValue, UnsupportedConstruct, UnsupportedKind, WindowSpec,
};

use std::fmt;
use std::path::{Path, PathBuf};

/// Which scanner stage produced an error (V-05 diagnosis surface).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorStage {
    /// File-level read failure.
    Io,
    /// Top-level document structure.
    Document,
    /// Entry head (prop key / node head / name / args).
    Entry,
    /// A prop value.
    Value,
    /// Inside a node body.
    NodeBody,
}

impl fmt::Display for ErrorStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorStage::Io => "io",
            ErrorStage::Document => "document",
            ErrorStage::Entry => "entry",
            ErrorStage::Value => "value",
            ErrorStage::NodeBody => "node-body",
        };
        f.write_str(s)
    }
}

/// Parse failure with stage/position/source-path diagnosis. Never a silent
/// fallback — callers decide between retry, user-facing diagnostics, or the
/// legacy adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectModelError {
    pub stage: ErrorStage,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub source_path: Option<PathBuf>,
}

impl fmt::Display for ProjectModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.source_path {
            Some(p) => write!(
                f,
                "[{}] {}:{}:{}: {}",
                self.stage,
                p.display(),
                self.line,
                self.column,
                self.message
            ),
            None => write!(
                f,
                "[{}] {}:{}: {}",
                self.stage, self.line, self.column, self.message
            ),
        }
    }
}

impl std::error::Error for ProjectModelError {}

pub type ProjectModelResult<T> = Result<T, ProjectModelError>;

/// Parse pac.at text into a [`ProjectDescriptor`].
///
/// Classification (unsupported constructs) is carried on the descriptor;
/// only malformed static input is an `Err`.
pub fn parse_document(text: &str, source: SourceInfo) -> ProjectModelResult<ProjectDescriptor> {
    let scan = scanner::Scanner::new(text, &source).scan_document()?;
    Ok(from_scan::build(source, scan))
}

/// Read and parse a pac.at file (read-only; no side effects).
pub fn parse_file(path: &Path) -> ProjectModelResult<ProjectDescriptor> {
    let text = std::fs::read_to_string(path).map_err(|e| ProjectModelError {
        stage: ErrorStage::Io,
        line: 0,
        column: 0,
        message: format!("failed to read {}: {e}", path.display()),
        source_path: Some(path.to_path_buf()),
    })?;
    parse_document(&text, SourceInfo::from_path(path))
}

#[cfg(test)]
mod tests;
