// Plan 091: SymbolLocation and CodePak extracted from universe.rs

use crate::ast;
use crate::scope::Sid;
use auto_val::AutoStr;

/// Location information for a symbol definition
#[derive(Debug, Clone)]
pub struct SymbolLocation {
    pub line: usize,
    pub character: usize,
    pub pos: usize,
}

impl SymbolLocation {
    pub fn new(line: usize, character: usize, pos: usize) -> Self {
        Self {
            line,
            character,
            pos,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodePak {
    pub sid: Sid,
    pub text: AutoStr,
    pub ast: ast::Code,
    pub file: AutoStr,
    pub cfile: AutoStr,
    pub header: AutoStr,
}
