//! Symbol types extracted from universe.rs
//! Plan 091: Part of Universe removal

use auto_val::AutoStr;
use crate::ast::Type;

/// Location of a symbol in source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolLocation {
    pub line: usize,
    pub character: usize,
    pub pos: usize,
}

impl SymbolLocation {
    pub fn new(line: usize, character: usize, pos: usize) -> Self {
        Self { line, character, pos }
    }
}

/// Code pack for compiled code info
#[derive(Debug, Clone)]
pub struct CodePak {
    pub name: AutoStr,
    pub return_type: Type,
    pub local_vars: usize,
    pub capture_count: usize,
}
