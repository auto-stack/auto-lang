//! AutoDown Document Format - Text-First Document DSL
//!
//! AutoDown is a text-first document DSL that transpiles to multiple backends
//! (Typst/PDF, DOCX/Word, HTML/Web). It uses three core symbols:
//! - `#` for headers
//! - `$` for logic/code domain
//! - `%{ ... }` for math expressions
//!
//! # Architecture
//!
//! ```text
//! AutoDown Source (.ad)
//!         ↓
//! Lexer (mode-aware: Text/Code/Math)
//!         ↓
//! Parser (Flip mechanism)
//!         ↓
//! ADOC AST (Document IR)
//!         ↓
//! Transpilers (Typst/DOCX/HTML)
//! ```
//!
//! # Example
//!
//! ```autodown
//! # Document Title
//!
//! This is a paragraph with ${variable} interpolation.
//!
//! Math formula: %{ E = m * c^2 }
//!
//! $for item in .items {
//!     - Item: ${item.name}
//! }
//! ```

pub mod ast;
pub mod cell;
pub mod error;
pub mod lexer;
pub mod parser;

pub mod trans;
pub mod math;

// Re-export main types for convenience
pub use ast::{AdocBlock, AdocDocument, AdocExpr, AdocInline, AdocMath, AdocSection};
pub use cell::{CellDirective, CellDirectiveError, CellRegion, CellType};
pub use cell::{extract_cell_directives, split_into_cells, try_extract_cell_directives};
pub use error::{AdocError, AdocResult};
pub use lexer::{AdocLexer, LexerMode};
pub use parser::AdocParser;

// Re-export transpilers
pub use trans::{AdocSink, AdocTranspiler};
