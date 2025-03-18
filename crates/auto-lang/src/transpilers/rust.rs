use super::Transpiler;
use auto_val::AutoStr;
use crate::ast::Code;
use std::io::Write;



pub struct RustTranspiler {
}

impl Transpiler for RustTranspiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> Result<(), String> {
        Ok(())
    }
}
