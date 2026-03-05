//! AutoDown Transpilers
//!
//! Backend generators for different output formats.

mod typst;
mod html;

// pub mod docx; // Optional - requires docx-rs dependency

use super::ast::*;
use super::error::{AdocError, AdocResult};
use std::collections::HashMap;

pub use typst::TypstTranspiler;
pub use html::HtmlTranspiler;
// pub use docx::DocxTranspiler;

/// Output sink for transpiled code
#[derive(Debug, Default)]
pub struct AdocSink {
    /// Main output content
    pub main: String,
    
    /// Style/header content (CSS, includes)
    pub styles: String,
    
    /// Front matter (metadata, YAML, etc.)
    pub front_matter: String,
}

impl AdocSink {
    /// Create a new sink
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get final output
    pub fn output(&self) -> String {
        let mut output = String::new();
        
        if !self.front_matter.is_empty() {
            output.push_str(&self.front_matter);
            output.push_str("\n\n");
        }
        
        if !self.styles.is_empty() {
            output.push_str(&self.styles);
            output.push_str("\n\n");
        }
        
        output.push_str(&self.main);
        
        output
    }
}

/// Common trait for AutoDown transpilers
pub trait AdocTranspiler {
    /// Transpile a complete document
    fn transpile(&mut self, doc: &AdocDocument) -> AdocResult<String>;
    
    /// Get the file extension for this backend
    fn extension(&self) -> &'static str;
    
    /// Transpile a section
    fn transpile_section(&mut self, section: &AdocSection) -> AdocResult<String>;
    
    /// Transpile a block
    fn transpile_block(&mut self, block: &AdocBlock) -> AdocResult<String>;
    
    /// Transpile inline content
    fn transpile_inline(&mut self, inline: &AdocInline) -> AdocResult<String>;
    
    /// Transpile math expression
    fn transpile_math(&mut self, math: &AdocMath) -> AdocResult<String>;
    
    /// Transpile expression (for interpolation)
    fn transpile_expr(&mut self, expr: &AdocExpr) -> AdocResult<String>;
}

/// Common helpers for transpilers
pub mod helpers {
    use super::*;
    
    /// Escape HTML special characters
    pub fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
    
    /// Escape text for Typst
    pub fn escape_typst(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('@', "\\@")
            .replace('#', "\\#")
            .replace('$', "\\$")
    }
    
    /// Escape text for LaTeX
    pub fn escape_latex(s: &str) -> String {
        s.replace('\\', "\\textbackslash{}")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('$', "\\$")
            .replace('&', "\\&")
            .replace('#', "\\#")
            .replace('%', "\\%")
            .replace('_', "\\_")
            .replace('~', "\\textasciitilde{}")
            .replace('^', "\\textasciicircum{}")
    }
    
    /// Generate section ID from title
    pub fn section_id(title: &str) -> String {
        title
            .to_lowercase()
            .replace(' ', "-")
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
            .trim_matches('-')
            .to_string()
    }
    
    /// Format metadata as YAML
    pub fn format_yaml_metadata(metadata: &AdocMetadata) -> String {
        let mut yaml = String::from("---\n");
        
        if let Some(author) = &metadata.author {
            yaml.push_str(&format!("author: \"{}\"\n", author));
        }
        
        if let Some(date) = &metadata.date {
            yaml.push_str(&format!("date: \"{}\"\n", date));
        }
        
        for (key, value) in &metadata.custom {
            yaml.push_str(&format!("{}: \"{}\"\n", key, value));
        }
        
        yaml.push_str("---\n");
        yaml
    }
}
