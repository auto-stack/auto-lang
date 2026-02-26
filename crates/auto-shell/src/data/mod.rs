//! Data module for shell
//!
//! Provides structured data types and table rendering.

pub mod table;
pub mod convert;
pub mod value;

pub use table::{Table, Column, Align, FileEntry};
pub use value::ShellValue;
