//! Block tier (Plan 342, Design 17).
//!
//! A *block* is a Skill-like UI unit sitting between widgets and apps: a
//! natural-language spec + structured contract that AI assembles from widgets.
//! Concretely each block is a *package* on disk (`blocks/<kind>/<name>/`) with:
//! - `spec.md` (TOML frontmatter + NL body)
//! - `reference/<variant>.at` (one or more reference implementations)
//! - `gotchas.md` (anti-examples)
//!
//! [`BlockRegistry`] scans and indexes them; [`BlockSpec`] is the parsed spec.

pub mod registry;
pub mod spec;

pub use registry::{BlockPackage, BlockRegistry};
pub use spec::{BlockSpec, DataSourceSignature};
