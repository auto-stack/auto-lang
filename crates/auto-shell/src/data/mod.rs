//! Data module for shell
//!
//! Provides structured data types and table rendering.

pub mod table;
pub mod convert;
pub mod value;
pub mod types;

pub use table::{Table, Column, Align, FileEntry};
pub use value::ShellValue;
pub use types::{
    AshFileEntry, AshProcessEntry, AshDiskEntry,
    AshCpuInfo, AshMemoryInfo, FileType,
};
