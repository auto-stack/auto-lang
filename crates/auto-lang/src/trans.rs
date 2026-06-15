use super::ast::*;
use crate::AutoResult;
use auto_val::AutoStr;
use std::io::{self, Write};

pub mod c;
pub mod rust;
pub mod python;
pub mod gdscript;
pub mod javascript;
pub mod typescript;
pub mod r2a;
pub mod tscn;
pub mod escape;

/// A single entry in the source map, mapping a source line to an output line.
/// Both line numbers are 1-based.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SourceMapEntry {
    pub source_line: usize,
    pub output_line: usize,
}

pub struct Sink {
    pub name: AutoStr,
    pub includes: Vec<u8>,
    pub body: Vec<u8>,
    pub header: Vec<u8>,
    pub source: Vec<u8>,
    /// Source map entries: maps source line -> output line (both 1-based)
    pub source_map: Vec<SourceMapEntry>,
    /// Currently active source line (set by transpilers before emitting a statement)
    current_source_line: Option<usize>,
    /// Current output line number (1-based), incremented on each newline in body
    current_output_line: usize,
}

impl Sink {
    pub fn new(name: AutoStr) -> Self {
        Self {
            name,
            includes: Vec::new(),
            body: Vec::new(),
            header: Vec::new(),
            source: Vec::new(),
            source_map: Vec::new(),
            current_source_line: None,
            current_output_line: 1,
        }
    }

    /// Create a dummy sink for temporary statement processing
    pub fn dummy() -> Self {
        Self {
            name: "".into(),
            includes: Vec::new(),
            body: Vec::new(),
            header: Vec::new(),
            source: Vec::new(),
            source_map: Vec::new(),
            current_source_line: None,
            current_output_line: 1,
        }
    }

    pub fn print(&mut self, data: &[u8]) -> AutoResult<()> {
        self.track_newlines(data);
        self.body.write(data)?;
        Ok(())
    }

    pub fn println(&mut self, data: &[u8]) -> AutoResult<()> {
        self.track_newlines(data);
        self.body.write(data)?;
        self.track_newlines(b"\n");
        self.body.write(b"\n")?;
        Ok(())
    }

    /// Set the current source line. Transpilers call this before emitting each statement.
    pub fn set_source_line(&mut self, line: usize) {
        self.current_source_line = Some(line);
    }

    /// Clear the current source line (e.g., between top-level items).
    pub fn clear_source_line(&mut self) {
        self.current_source_line = None;
    }

    /// Track newlines in data and push source map entries when a source line is active.
    pub(crate) fn track_newlines(&mut self, data: &[u8]) {
        for &byte in data {
            if byte == b'\n' {
                if let Some(sl) = self.current_source_line {
                    self.source_map.push(SourceMapEntry {
                        source_line: sl,
                        output_line: self.current_output_line,
                    });
                }
                self.current_output_line += 1;
            }
        }
    }

    pub fn done(&mut self) -> AutoResult<&Vec<u8>> {
        // add include to self.h
        // println!("Sink Name: {}", self.name); // LSP: disabled
        let mut prefix_lines = 0usize;
        if !self.header.is_empty() && !self.body.is_empty() {
            self.source.write(b"#include \"")?;
            self.source.write(self.name.as_bytes())?;
            self.source.write(b".h\"\n\n")?;
            prefix_lines = 2; // the include line + blank line
        }
        // Adjust all source map output lines by any prefix lines added above
        if prefix_lines > 0 {
            for entry in &mut self.source_map {
                entry.output_line += prefix_lines;
            }
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

/// Plan 168: Escape a string for embedding in a double-quoted string literal.
/// Handles newlines, tabs, carriage returns, backslashes, and double quotes.
pub fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str(r"\\"),
            '"' => out.push_str(r#"\""#),
            '\n' => out.push_str(r"\n"),
            '\r' => out.push_str(r"\r"),
            '\t' => out.push_str(r"\t"),
            '\0' => out.push_str(r"\0"),
            _ => out.push(c),
        }
    }
    out
}

/// Plan 167: Multi-file output sink for project-level transpilation
pub struct MultiSink {
    pub files: Vec<(String, Sink)>,
}

impl MultiSink {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn add(&mut self, name: &str) -> &mut Sink {
        self.files.push((name.to_string(), Sink::new(AutoStr::from(name))));
        &mut self.files.last_mut().unwrap().1
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Sink> {
        self.files.iter_mut().find(|(n, _)| n == name).map(|(_, s)| s)
    }

    pub fn get(&self, name: &str) -> Option<&Sink> {
        self.files.iter().find(|(n, _)| n == name).map(|(_, s)| s)
    }

    /// Get all files as (name, content) pairs
    pub fn done(self) -> Vec<(String, Vec<u8>)> {
        self.files
            .into_iter()
            .map(|(name, mut sink)| {
                let content = sink.done().unwrap().clone();
                (name, content)
            })
            .collect()
    }
}
