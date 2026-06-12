//! Command parsing utilities
//!
//! This module provides parsers for various shell syntax elements.

pub mod lexer;
pub mod pipeline;
pub mod quote;
pub mod redirect;
pub mod history;

pub use lexer::{tokenize, tokens_to_string, ShellToken};
pub use pipeline::{parse_chain, parse_pipeline, group_pipe_segments, ChainOp, ChainSegment};
pub use quote::parse_args;
pub use quote::parse_args_preserve_quotes;
pub use redirect::{parse_redirect, Redirect, StderrRedirect};
pub use history::{expand_history, History};
