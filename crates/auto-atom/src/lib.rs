//! # Auto-Atom: Auto Object Markup Language
//!
//! Auto-Atom is a data interchange format designed as a modern alternative to JSON/XML/YAML.
//! It combines the best features of these formats while adding powerful capabilities specific
//! to the AutoLang ecosystem.
//!
//! ## Features
//!
//! - **Tree-based structure**: Hierarchical data with nodes and properties
//! - **Type-safe**: Rust's type system ensures correctness at compile time
//! - **Ergonomic API**: Convenient builders and converters
//! - **Format-agnostic**: Serialize to JSON, XML, YAML, and more
//!
//! ## Quick Start
//!
//! ```rust
//! use auto_atom::{Atom, AtomResult};
//! use auto_val::Value;
//!
//! fn main() -> AtomResult<()> {
//!     // Create an atom from values
//!     let atom = Atom::assemble(vec![
//!         Value::pair("name", "AutoLang"),
//!         Value::pair("version", "0.1.0"),
//!     ])?;
//!
//!     // Convert to string
//!     println!("{}", atom.to_astr());
//!     Ok(())
//! }
//! ```
//!
//! ## Data Model
//!
//! Atoms consist of:
//! - **Node**: Hierarchical structure with properties and children
//! - **Array**: Ordered list of values
//! - **Empty**: Null/empty value
//!
//! ## See Also
//!
//! - [`Atom`] - Main data structure
//! - [`Root`] - Root content variants
//! - [`AtomError`] - Error types
//! - [`AtomResult`] - Result type alias

mod atom;
mod error;

pub use atom::*;
pub use error::{AtomError, AtomResult};
