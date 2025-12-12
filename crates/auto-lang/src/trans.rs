use super::ast::*;
use crate::AutoResult;
use auto_val::AutoStr;
use std::io::{self, Write};

pub mod c;
pub mod rust;

pub struct Sink {
    pub name: AutoStr,
    pub includes: Vec<u8>,
    pub body: Vec<u8>,
    pub header: Vec<u8>,
    pub source: Vec<u8>,
}

impl Sink {
    pub fn new() -> Self {
        Self {
            name: AutoStr::new(),
            includes: Vec::new(),
            body: Vec::new(),
            header: Vec::new(),
            source: Vec::new(),
        }
    }

    pub fn print(&mut self, data: &[u8]) -> AutoResult<()> {
        self.body.write(data)?;
        Ok(())
    }

    pub fn println(&mut self, data: &[u8]) -> AutoResult<()> {
        self.body.write(data)?;
        self.body.write(b"\n")?;
        Ok(())
    }

    pub fn done(&mut self) -> AutoResult<&Vec<u8>> {
        if !self.header.is_empty() && !self.body.is_empty() {
            self.source.write(b"#include \"")?;
            self.source.write(self.name.as_bytes())?;
            self.source.write(b".h\"\n\n")?;
        }
        self.source.append(&mut self.body);
        Ok(&self.source)
    }
}

pub trait Trans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()>;
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
