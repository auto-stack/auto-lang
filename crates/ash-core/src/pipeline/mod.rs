//! Atom Pipeline — typed data flow between shell commands
//!
//! This module provides the Atom type system for semantic pipeline data:
//!
//! - [`Atom`] — single typed value with a semantic tag
//! - [`AtomStream`] — cursor-based iteration over multiple Atoms
//! - [`ExternalStream`] — streaming output from an external child process
//! - [`AtomPipeline`] — the pipeline enum (Atom | Stream | ExternalStream | Text | Empty)
//! - [`convert`] — type inference and conversion helpers
//! - [`batom`] — high-performance binary encoding (Batom format)

pub mod atom;
pub mod atom_stream;
pub mod external_stream;
pub mod atom_pipeline;
pub mod convert;
pub mod batom;
pub mod operators;

pub use atom::{Atom, AtomType};
pub use atom_stream::AtomStream;
pub use external_stream::ExternalStream;
pub use atom_pipeline::AtomPipeline;
