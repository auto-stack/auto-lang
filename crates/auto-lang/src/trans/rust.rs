use super::{Trans, Sink};
use crate::ast::Code;
use crate::AutoResult;

pub struct RustTrans {
}

impl Trans for RustTrans {
    fn trans(&mut self, _ast: Code, _out: &mut Sink) -> AutoResult<()> {
        Ok(())
    }
}
