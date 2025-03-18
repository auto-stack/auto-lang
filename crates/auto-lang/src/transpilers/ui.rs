use auto_val::AutoStr;
use super::Transpiler;
use crate::ast::Code;
use std::io::Write;

pub struct UITranspiler {
    indent: usize,
    includes: Vec<u8>,
    header: Vec<u8>,
    name: AutoStr,
}

impl Transpiler for UITranspiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> Result<(), String> {
        Ok(())
    }
}
