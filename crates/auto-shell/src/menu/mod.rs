//! AshMenu — adaptive completion menu for AutoShell
//!
//! Replaces reedline's ColumnarMenu with a smarter menu that:
//! - Auto-selects layout (compact grid vs descriptive list) based on data
//! - Colors completions by type (command=blue, file=white, dir=cyan, etc.)
//! - Pages through results at half-screen height
//! - Supports search filtering (future)

pub mod ash_menu;
pub mod layout;
pub mod render;
pub mod style;

pub use ash_menu::{AshMenu, AshMenuConfig};
