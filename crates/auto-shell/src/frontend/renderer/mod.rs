//! Renderer module — ratatui Buffer to terminal output bridges
//!
//! This module provides the conversion layer between ratatui's in-memory
//! `Buffer` (rendered by widgets) and ANSI strings that can be displayed
//! in the terminal via reedline or direct stdout.

pub mod buffer_to_ansi;

pub use buffer_to_ansi::{buffer_to_ansi, buffer_to_plain};
