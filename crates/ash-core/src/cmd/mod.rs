//! Core command infrastructure - pure logic, zero terminal dependencies
//!
//! Data manipulation helpers, value formatting, and external process execution.
//! These have no dependency on reedline, crossterm, or nu-ansi-term.

pub mod data;
pub mod external;
pub mod value_helpers;
