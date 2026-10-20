//! Error types for Atom operations
//!
//! The canonical definitions now live in the `auto-atom` crate. This module
//! re-exports them under `auto_lang::atom_error` for backward compatibility.

pub use auto_atom::{AtomError, AtomResult};
