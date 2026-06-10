//! Core module - pure logic layer with zero terminal dependencies
//!
//! This module contains all shell logic that is independent of the terminal
//! (no reedline, crossterm, or nu-ansi-term). Eventually this will become
//! the `ash-core` crate.
//!
//! ## Migration Status
//!
//! | Module | Status | Notes |
//! |--------|--------|-------|
//! | parser | ✅ migrated | Pipeline, quote, redirect, history parsing |
//! | data | ✅ migrated | ShellValue, AshFileEntry, convert (table stays in frontend) |
//! | bookmarks | ✅ migrated | Bookmark data + file persistence |
//! | shell/vars | ✅ migrated | Shell variable management |
//! | completions | ✅ partial | auto, command, file (reedline adapter stays in frontend) |
//! | cmd | ✅ partial | data, value_helpers, external (builtin, fs stay in frontend) |

pub mod bookmarks;
pub mod cmd;
pub mod completions;
pub mod data;
pub mod parser;
pub mod shell;
