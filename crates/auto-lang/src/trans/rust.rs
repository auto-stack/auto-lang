use super::Transpiler;
use auto_val::AutoStr;
use crate::ast::Code;
use std::io::Write;
use crate::AutoResult;

pub struct RustTranspiler {
}

impl Transpiler for RustTranspiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> AutoResult<()> {
        Ok(())
    }
}
