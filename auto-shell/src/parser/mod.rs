//! Command parsing utilities
//!
//! This module provides parsers for various shell syntax elements.

pub mod pipeline;
pub mod quote;
pub mod redirect;
pub mod history;

pub use quote::parse_args;
pub use quote::parse_args_preserve_quotes;
pub use history::{expand_history, History};
