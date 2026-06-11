//! Ash Core — AutoShell engine, pure logic with zero terminal dependencies
//!
//! This crate contains all shell logic that is independent of the terminal
//! (no reedline, crossterm, nu-ansi-term, or ratatui).
//!
//! ## Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | `parser` | Pipeline, quote, redirect, history parsing |
//! | `data` | ShellValue, AshFileEntry, data conversion |
//! | `bookmarks` | Bookmark data + file persistence |
//! | `shell` | Shell variable management |
//! | `completions` | auto, command, file completion + Completion type + get_completions() |
//! | `cmd` | data operations, value helpers, external command execution |
//! | `pipeline` | Atom type system, AtomStream, AtomPipeline for typed data flow |

pub mod bookmarks;
pub mod cmd;
pub mod completions;
pub mod data;
pub mod pipeline;
pub mod parser;
pub mod shell;
