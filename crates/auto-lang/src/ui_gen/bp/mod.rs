//! Blueprint tier (Plan 342, Design 17; renamed from Block, PLAN-639).
//!
//! A *blueprint* is a Skill-like UI unit sitting between widgets and apps: a
//! natural-language spec + structured contract that AI assembles from widgets.
//! Concretely each blueprint is a *package* on disk (`blueprints/<kind>/<name>/`) with:
//! - `spec.md` (TOML frontmatter + NL body)
//! - `reference/<variant>.at` (one or more reference implementations)
//! - `gotchas.md` (anti-examples)
//!
//! [`BlueprintRegistry`] scans and indexes them; [`BlueprintSpec`] is the parsed spec.
//! Renamed from Block tier (PLAN-639): `ui_gen/block` → `ui_gen/bp`, CLI `auto block` → `auto bp`.

pub mod registry;
pub mod spec;

pub use registry::{BlueprintPackage, BlueprintRegistry};
pub use spec::{BlueprintSpec, DataSourceSignature};
