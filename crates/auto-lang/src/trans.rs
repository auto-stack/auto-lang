use super::ast::*;
use crate::AutoResult;
use std::io;
use std::io::Write;

pub mod c;
pub mod rust;
pub trait Trans {
    fn trans(&mut self, ast: Code, out: &mut impl Write) -> AutoResult<()>;
}

pub trait ToStrError {
    fn to(self) -> AutoResult<()>;
}

impl ToStrError for Result<(), io::Error> {
    fn to(self) -> AutoResult<()> {
        self.map_err(|e| e.to_string().into())
    }
}

impl ToStrError for Result<usize, io::Error> {
    fn to(self) -> AutoResult<()> {
        match self {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string().into()),
        }
    }
}
