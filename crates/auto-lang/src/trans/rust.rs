use super::Trans;
use crate::ast::Code;
use std::io::Write;
use crate::AutoResult;

pub struct RustTrans {
}

impl Trans for RustTrans {
    fn trans(&mut self, _ast: Code, _out: &mut impl Write) -> AutoResult<()> {
        Ok(())
    }
}
