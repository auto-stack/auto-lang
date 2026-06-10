//! Core data types - pure logic, zero terminal dependencies
//!
//! Shell value types, file metadata types, and conversion functions.
//! These have no dependency on reedline, crossterm, or nu-ansi-term.

pub mod convert;
pub mod types;
pub mod value;

pub use value::ShellValue;
pub use types::{
    AshFileEntry, AshProcessEntry, AshDiskEntry,
    AshCpuInfo, AshMemoryInfo, FileType,
};
pub use convert::{metadata_to_entry, file_entry_to_value, file_entries_to_value};
