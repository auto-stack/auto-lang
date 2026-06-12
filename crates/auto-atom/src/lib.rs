//! # Auto-Atom
//!
//! Static Atom data structures for the AutoLang ecosystem.
//!
//! Atom (Auto Object Markup) is a JSON/XML-like data format used by AutoLang.
//! This crate provides the core data structures (`Atom`, `AtomBuilder`) and
//! error types without pulling in the full AutoLang compiler or interpreter.
//!
//! ## Quick start
//!
//! ```rust
//! use auto_atom::Atom;
//! use auto_val::Value;
//!
//! let atom = Atom::assemble(vec![
//!     Value::pair("name", "Alice"),
//!     Value::pair("age", 30),
//! ]).unwrap();
//!
//! println!("{}", atom.to_astr());
//! ```

pub mod atom;
pub mod error;
pub mod parser;

pub use atom::{Atom, AtomBuilder, EMPTY};
pub use error::{AtomError, AtomResult};
pub use parser::AtomParser;
