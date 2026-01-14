use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub mod backend;
pub mod completion;
pub mod diagnostics;
pub mod hover_info;
pub mod goto_def;

pub use backend::Backend;
