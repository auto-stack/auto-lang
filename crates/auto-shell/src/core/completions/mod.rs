//! Core completion logic - pure logic, zero terminal dependencies
//!
//! File path completion, command name completion, and Auto variable completion.
//! These have no dependency on reedline or any terminal library.

pub mod auto;
pub mod command;
pub mod file;
