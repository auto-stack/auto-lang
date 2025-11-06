use super::ast::*;
use crate::AutoResult;
use std::io;

pub mod c;
pub mod rust;

pub struct Sink {
    pub includes: Vec<u8>,
    pub body: Vec<u8>,
    pub header: Vec<u8>,
    pub source: Vec<u8>,
}

impl Sink {
    pub fn new() -> Self {
        Self {
            includes: Vec::new(),
            body: Vec::new(),
            header: Vec::new(),
            source: Vec::new(),
        }
    }

    pub fn done(&mut self) -> &Vec<u8> {
        if self.includes.len() > 0 {
            self.source.append(&mut self.includes);
            self.source.push('\n' as u8);
        }
        self.source.append(&mut self.body);
        &self.source
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
