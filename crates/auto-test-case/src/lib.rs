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
//! Current scope is the T-02 foundation:
//! - [`document`]: pinned block tree + raw-fence correspondence, anchor
//!   section resolution.
//! - [`raw_ranges`]: byte-accurate fence locator over original bytes.
//! - [`admission`]: resource/path admission limits and checks.

pub mod admission;
pub mod document;
pub mod raw_ranges;
pub mod sha256;
