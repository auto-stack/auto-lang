//! AutoDown Cell Directive Parser
//!
//! Extracts `/// cell:` metadata directives from raw AutoDown source.
//! Standard AutoDown treats these as comments (ignored by the lexer).
//! AutoLab uses this module to build the cell model from `.ad` files.
//!
//! # Directive Format
//!
//! ```text
//! /// cell:<id> type:<code|markdown|ai|chart> [depends_on:<id1,id2>]
//! ```
//!
//! # Example
//!
//! ```autodown
//! /// cell:c1 type:code
//! # Data Loading
//!
//! $var data = load_csv("data.csv")
//! $print(f"Loaded ${data.len} rows")
//!
//! /// cell:c2 type:markdown depends_on:c1
//! The dataset contains ${data.len} rows.
//! ```

use std::collections::HashSet;

// ============================================================================
// Cell Directive Types
// ============================================================================

/// Cell type discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellType {
    /// Executable Auto code
    Code,
    /// Markdown/text content
    Markdown,
    /// AI-generated content
    Ai,
    /// Chart/visualization
    Chart,
}

impl std::str::FromStr for CellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(CellType::Code),
            "markdown" => Ok(CellType::Markdown),
            "ai" => Ok(CellType::Ai),
            "chart" => Ok(CellType::Chart),
            _ => Err(format!("unknown cell type: {}", s)),
        }
    }
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellType::Code => write!(f, "code"),
            CellType::Markdown => write!(f, "markdown"),
            CellType::Ai => write!(f, "ai"),
            CellType::Chart => write!(f, "chart"),
        }
    }
}

/// Parsed cell directive extracted from a `/// cell:` comment line
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellDirective {
    /// Cell identifier (e.g., "c1")
    pub id: String,

    /// Cell type
    pub cell_type: CellType,

    /// IDs of cells this cell depends on (for execution order)
    pub depends_on: Vec<String>,

    /// 1-based line number where the directive appears
    pub line: usize,
}

impl CellDirective {
    /// Create a new cell directive
    pub fn new(id: impl Into<String>, cell_type: CellType, line: usize) -> Self {
        Self {
            id: id.into(),
            cell_type,
            depends_on: Vec::new(),
            line,
        }
    }

    /// Add a dependency
    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.depends_on.push(id.into());
        self
    }
}

// ============================================================================
// Extraction
// ============================================================================

/// Extract all `/// cell:` directives from raw AutoDown source.
///
/// Returns directives in source order. Each directive records the line
/// number where it appears so downstream tools can split the document
/// into cell regions.
///
/// Lines that start with `/// cell:` but have invalid syntax are silently
/// skipped. Use [`try_extract_cell_directives`] if you need error details.
pub fn extract_cell_directives(source: &str) -> Vec<CellDirective> {
    let mut directives = Vec::new();
    let mut seen_ids = HashSet::new();

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim_start();

        if let Some(rest) = trimmed.strip_prefix("/// cell:") {
            if let Some(directive) = parse_directive_body(rest.trim(), line_num) {
                // Skip duplicate IDs (keep first occurrence)
                if seen_ids.insert(directive.id.clone()) {
                    directives.push(directive);
                }
            }
        }
    }

    directives
}

/// Parse the body of a `/// cell:` directive (the part after the prefix).
///
/// Expected format: `<id> type:<type> [depends_on:<id1,id2>]`
fn parse_directive_body(body: &str, line: usize) -> Option<CellDirective> {
    let mut parts = body.split_whitespace();

    // First token is the cell ID
    let id = parts.next()?.to_string();
    if id.is_empty() {
        return None;
    }

    let mut cell_type = None;
    let mut depends_on = Vec::new();

    for part in parts {
        if let Some(type_val) = part.strip_prefix("type:") {
            cell_type = type_val.parse::<CellType>().ok();
        } else if let Some(dep_val) = part.strip_prefix("depends_on:") {
            for dep in dep_val.split(',') {
                let dep = dep.trim();
                if !dep.is_empty() {
                    depends_on.push(dep.to_string());
                }
            }
        }
        // Unknown fields are silently ignored for forward compatibility
    }

    let cell_type = cell_type?;

    Some(CellDirective {
        id,
        cell_type,
        depends_on,
        line,
    })
}

/// Extract cell directives with detailed error reporting.
///
/// Returns a tuple of `(directives, errors)` where `errors` contains
/// descriptions of lines that looked like directives but failed to parse.
pub fn try_extract_cell_directives(source: &str) -> (Vec<CellDirective>, Vec<CellDirectiveError>) {
    let mut directives = Vec::new();
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::new();

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim_start();

        if let Some(rest) = trimmed.strip_prefix("/// cell:") {
            match try_parse_directive_body(rest.trim(), line_num) {
                Ok(directive) => {
                    if seen_ids.insert(directive.id.clone()) {
                        directives.push(directive);
                    }
                }
                Err(e) => errors.push(e),
            }
        }
    }

    (directives, errors)
}

/// Error describing a malformed cell directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellDirectiveError {
    /// 1-based line number
    pub line: usize,
    /// Error description
    pub message: String,
}

fn try_parse_directive_body(body: &str, line: usize) -> Result<CellDirective, CellDirectiveError> {
    let mut parts = body.split_whitespace();

    let id = parts
        .next()
        .ok_or_else(|| CellDirectiveError {
            line,
            message: "missing cell id".to_string(),
        })?
        .to_string();

    if id.is_empty() {
        return Err(CellDirectiveError {
            line,
            message: "empty cell id".to_string(),
        });
    }

    let mut cell_type = None;
    let mut depends_on = Vec::new();

    for part in parts {
        if let Some(type_val) = part.strip_prefix("type:") {
            cell_type = Some(type_val.parse::<CellType>().map_err(|e| CellDirectiveError {
                line,
                message: e,
            })?);
        } else if let Some(dep_val) = part.strip_prefix("depends_on:") {
            for dep in dep_val.split(',') {
                let dep = dep.trim();
                if !dep.is_empty() {
                    depends_on.push(dep.to_string());
                }
            }
        }
    }

    let cell_type = cell_type.ok_or_else(|| CellDirectiveError {
        line,
        message: "missing type:<code|markdown|ai|chart>".to_string(),
    })?;

    Ok(CellDirective {
        id,
        cell_type,
        depends_on,
        line,
    })
}

// ============================================================================
// Cell-aware document splitting
// ============================================================================

/// A region of an AutoDown document between two cell directives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRegion {
    /// Cell metadata (None for preamble before first directive)
    pub directive: Option<CellDirective>,
    /// Raw source text belonging to this cell (excluding the directive line)
    pub source: String,
    /// 1-based start line (inclusive)
    pub start_line: usize,
    /// 1-based end line (inclusive)
    pub end_line: usize,
}

/// Split an AutoDown document into cell regions using `/// cell:` directives.
///
/// Content before the first directive becomes a preamble region with
/// `directive: None`. Each subsequent region starts at the line after
/// a directive and ends at the line before the next directive (or EOF).
pub fn split_into_cells(source: &str) -> Vec<CellRegion> {
    let directives = extract_cell_directives(source);
    let mut regions = Vec::new();

    if directives.is_empty() {
        // No directives — entire document is one preamble region
        let line_count = source.lines().count().max(1);
        regions.push(CellRegion {
            directive: None,
            source: source.to_string(),
            start_line: 1,
            end_line: line_count,
        });
        return regions;
    }

    let lines: Vec<&str> = source.lines().collect();
    let total_lines = lines.len();

    // Preamble: from line 1 up to (but not including) the first directive line
    let first_dir_line = directives[0].line;
    if first_dir_line > 1 {
        let preamble: Vec<&str> = lines[..first_dir_line - 1].to_vec();
        regions.push(CellRegion {
            directive: None,
            source: preamble.join("\n"),
            start_line: 1,
            end_line: first_dir_line - 1,
        });
    }

    // Each directive defines a cell region
    for (i, directive) in directives.iter().enumerate() {
        let start_idx = directive.line; // 0-based index = line - 1
        let end_idx = if i + 1 < directives.len() {
            directives[i + 1].line - 1
        } else {
            total_lines
        };

        let cell_lines: Vec<&str> = lines[start_idx..end_idx].to_vec();
        regions.push(CellRegion {
            directive: Some(directive.clone()),
            source: cell_lines.join("\n"),
            start_line: start_idx + 1,
            end_line: end_idx,
        });
    }

    regions
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_directive() {
        let source = r#"/// cell:c1 type:code
# Hello
$print("world")
"#;
        let dirs = extract_cell_directives(source);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].id, "c1");
        assert_eq!(dirs[0].cell_type, CellType::Code);
        assert!(dirs[0].depends_on.is_empty());
        assert_eq!(dirs[0].line, 1);
    }

    #[test]
    fn test_extract_with_dependency() {
        let source = r#"/// cell:c1 type:code
$var x = 1
/// cell:c2 type:markdown depends_on:c1
The value is ${x}.
"#;
        let dirs = extract_cell_directives(source);
        assert_eq!(dirs.len(), 2);
        assert_eq!(dirs[1].id, "c2");
        assert_eq!(dirs[1].cell_type, CellType::Markdown);
        assert_eq!(dirs[1].depends_on, vec!["c1"]);
        assert_eq!(dirs[1].line, 3);
    }

    #[test]
    fn test_extract_multiple_dependencies() {
        let source = "/// cell:c3 type:chart depends_on:c1,c2";
        let dirs = extract_cell_directives(source);
        assert_eq!(dirs[0].depends_on, vec!["c1", "c2"]);
    }

    #[test]
    fn test_extract_all_types() {
        let source = r#"/// cell:a type:code
/// cell:b type:markdown
/// cell:c type:ai
/// cell:d type:chart
"#;
        let dirs = extract_cell_directives(source);
        assert_eq!(dirs.len(), 4);
        assert_eq!(dirs[0].cell_type, CellType::Code);
        assert_eq!(dirs[1].cell_type, CellType::Markdown);
        assert_eq!(dirs[2].cell_type, CellType::Ai);
        assert_eq!(dirs[3].cell_type, CellType::Chart);
    }

    #[test]
    fn test_ignore_invalid_directives() {
        let source = r#"/// cell: type:code
/// cell:c1 unknown_type
/// cell:c2 type:code
"#;
        let dirs = extract_cell_directives(source);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].id, "c2");
    }

    #[test]
    fn test_ignore_duplicate_ids() {
        let source = r#"/// cell:c1 type:code
/// cell:c1 type:markdown
"#;
        let dirs = extract_cell_directives(source);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].cell_type, CellType::Code);
    }

    #[test]
    fn test_try_extract_with_errors() {
        let source = r#"/// cell: type:code
/// cell:c1 type:code
/// cell:c2 unknown_type
"#;
        let (dirs, errs) = try_extract_cell_directives(source);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].id, "c1");
        assert_eq!(errs.len(), 2);
    }

    #[test]
    fn test_split_into_cells_with_preamble() {
        let source = r#"# Document Title

/// cell:c1 type:code
$var x = 1

/// cell:c2 type:markdown depends_on:c1
Value: ${x}
"#;
        let regions = split_into_cells(source);
        assert_eq!(regions.len(), 3);

        // Preamble
        assert!(regions[0].directive.is_none());
        assert!(regions[0].source.contains("Document Title"));

        // Cell c1
        assert_eq!(regions[1].directive.as_ref().unwrap().id, "c1");
        assert!(regions[1].source.contains("$var x = 1"));

        // Cell c2
        assert_eq!(regions[2].directive.as_ref().unwrap().id, "c2");
        assert!(regions[2].source.contains("Value: ${x}"));
    }

    #[test]
    fn test_split_no_directives() {
        let source = "# Just a document\n\nSome text.";
        let regions = split_into_cells(source);
        assert_eq!(regions.len(), 1);
        assert!(regions[0].directive.is_none());
        assert_eq!(regions[0].source, source);
    }

    #[test]
    fn test_split_no_preamble() {
        let source = r#"/// cell:c1 type:code
$var x = 1
"#;
        let regions = split_into_cells(source);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].directive.as_ref().unwrap().id, "c1");
    }

    #[test]
    fn test_lexer_skips_comments() {
        use crate::autodown::lexer::{AdocLexer, AdTokenKind};

        let source = "/// cell:c1 type:code\n# Title\n";
        let mut lexer = AdocLexer::new(source);

        // After skipping comment line, we get the trailing newline, then Header
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Newline);

        let token = lexer.next_token().unwrap();
        assert!(matches!(token.kind, AdTokenKind::Header { level: 1 }));

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert_eq!(token.text, "Title");

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Newline);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::EOF);
    }

    #[test]
    fn test_lexer_inline_comment() {
        use crate::autodown::lexer::{AdocLexer, AdTokenKind};

        let source = "Hello // this is a comment\nWorld";
        let mut lexer = AdocLexer::new(source);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert!(token.text.contains("Hello"));

        // Skip whitespace, then comment line, then get World
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Newline);

        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, AdTokenKind::Text);
        assert!(token.text.contains("World"));
    }
}
