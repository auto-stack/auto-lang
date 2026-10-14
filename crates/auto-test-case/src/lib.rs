//! # auto-test-case
//!
//! Protocol core for the AutoDown TestCase profile (`autolang.case/1` /
//! `autolang.suite/1`, Design 35 / Plan 768): strict document loading with
//! byte-accurate fence payloads, admission limits, the case/suite contract
//! model, oracle comparison, and report projection.
//!
//! Dependency rules (Design 35 §8): this crate must not depend on auto-lang,
//! UI crates, or any graphical runtime. The only runtime dependency is the
//! pinned [`autodown_test_parser`] snapshot; adapters that execute real
//! backends live in `auto-lang` and depend on THIS crate, never the reverse.
//!
//! Layering: [`document`]/[`raw_ranges`]/[`admission`] load and guard raw
//! bytes; [`atom`]/[`contract`] decode the strict case schema; [`ir`]
//! materializes payloads into owned bytes; [`suite`]/[`discovery`] bind
//! cases into a suite with explicit defaults; [`compare`]/[`report`] judge
//! observations and project results.

pub mod admission;
pub mod atom;
pub mod compare;
pub mod contract;
pub mod discovery;
pub mod document;
pub mod ir;
pub mod raw_ranges;
pub mod report;
pub mod sha256;
pub mod suite;

/// Crate version, for report toolchain identity (Plan 773 F-06).
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
