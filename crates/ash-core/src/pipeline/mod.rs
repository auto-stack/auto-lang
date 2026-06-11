//! Atom Pipeline — typed data flow between shell commands
//!
//! This module provides the Atom type system for semantic pipeline data:
//!
//! - [`Atom`] — single typed value with a semantic tag
//! - [`AtomStream`] — cursor-based iteration over multiple Atoms
//! - [`AtomPipeline`] — the pipeline enum (Atom | Stream | Text | Empty)
//! - [`convert`] — type inference and conversion helpers
//! - [`batom`] — high-performance binary encoding (Batom format)

pub mod atom;
pub mod atom_stream;
pub mod atom_pipeline;
pub mod convert;
pub mod batom;

pub use atom::{Atom, AtomType};
pub use atom_stream::AtomStream;
pub use atom_pipeline::AtomPipeline;
