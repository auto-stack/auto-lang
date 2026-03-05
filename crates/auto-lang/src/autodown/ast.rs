//! AutoDown AST Types
//!
//! Defines the document AST for AutoDown format.

use std::collections::HashMap;
use std::fmt;

// ============================================================================
// Document Root
// ============================================================================

/// AutoDown Document (root node)
#[derive(Debug, Clone)]
pub struct AdocDocument {
    /// Document title (from first # header or front matter)
    pub title: Option<String>,

    /// Document metadata (YAML-like front matter)
    pub metadata: AdocMetadata,

    /// Content before first header
    pub preamble: Vec<AdocBlock>,

    /// Document sections (from # headers)
    pub sections: Vec<AdocSection>,
}

impl Default for AdocDocument {
    fn default() -> Self {
        Self {
            title: None,
            metadata: AdocMetadata::default(),
            preamble: Vec::new(),
            sections: Vec::new(),
        }
    }
}

// ============================================================================
// Metadata
// ============================================================================

/// Document metadata (YAML-like front matter)
#[derive(Debug, Clone, Default)]
pub struct AdocMetadata {
    /// Author name
    pub author: Option<String>,

    /// Document date
    pub date: Option<String>,

    /// Custom metadata fields
    pub custom: HashMap<String, String>,
}

// ============================================================================
// Sections
// ============================================================================

/// Section (header + content)
#[derive(Debug, Clone)]
pub struct AdocSection {
    /// Header level (1-6)
    pub level: u8,

    /// Section title text
    pub title: String,

    /// Auto-generated or explicit ID (for anchors)
    pub id: Option<String>,

    /// Section content blocks
    pub content: Vec<AdocBlock>,

    /// Nested subsections
    pub subsections: Vec<AdocSection>,
}

impl AdocSection {
    /// Create a new section
    pub fn new(level: u8, title: impl Into<String>) -> Self {
        Self {
            level,
            title: title.into(),
            id: None,
            content: Vec::new(),
            subsections: Vec::new(),
        }
    }

    /// Generate an ID from the title
    pub fn generate_id(&mut self) {
        self.id = Some(
            self.title
                .to_lowercase()
                .replace(' ', "-")
                .replace(|c: char| !c.is_alphanumeric() && c != '-', ""),
        );
    }
}

// ============================================================================
// Block Content
// ============================================================================

/// Block-level content
#[derive(Debug, Clone)]
pub enum AdocBlock {
    /// Paragraph with inline content
    Paragraph(Vec<AdocInline>),

    /// List (ordered or unordered)
    List {
        items: Vec<AdocListItem>,
        ordered: bool,
    },

    /// Code block with optional language
    CodeBlock { lang: Option<String>, code: String },

    /// Blockquote
    Blockquote(Vec<AdocBlock>),

    /// Math block (display mode)
    MathBlock(AdocMath),

    /// Table
    Table {
        headers: Vec<AdocInline>,
        rows: Vec<Vec<AdocInline>>,
    },

    /// Horizontal rule (thematic break)
    HorizontalRule,

    /// Image block
    Image { alt: String, url: String },

    /// Conditional block (if)
    If {
        condition: String,
        then_body: Vec<AdocBlock>,
        else_body: Option<Vec<AdocBlock>>,
    },

    /// Loop block (for)
    For {
        var: String,
        index: Option<String>,
        iterable: String,
        body: Vec<AdocBlock>,
    },

    /// Component call
    Component {
        name: String,
        props: HashMap<String, AdocExpr>,
        children: Vec<AdocBlock>,
    },

    /// Raw Auto code (for style definitions, etc.)
    RawCode(String),

    /// Section reference (for including other documents)
    Include(String),
}

// ============================================================================
// Inline Content
// ============================================================================

/// Inline content
#[derive(Debug, Clone)]
pub enum AdocInline {
    /// Plain text
    Text(String),

    /// Bold text
    Bold(Vec<AdocInline>),

    /// Italic text
    Italic(Vec<AdocInline>),

    /// Inline code
    Code(String),

    /// Strikethrough text
    Strikethrough(Vec<AdocInline>),

    /// Link with text and URL
    Link { text: String, url: String },

    /// Image with alt text and URL
    Image { alt: String, url: String },

    /// Inline math
    Math(AdocMath),

    /// Variable/expression interpolation
    Interpolate(AdocExpr),
}

// ============================================================================
// List Items
// ============================================================================

/// List item with optional nested list
#[derive(Debug, Clone)]
pub struct AdocListItem {
    /// Item content
    pub content: Vec<AdocInline>,

    /// Nested list (if any)
    pub nested: Option<Box<AdocList>>,
}

impl AdocListItem {
    /// Create a simple list item
    pub fn simple(content: Vec<AdocInline>) -> Self {
        Self {
            content,
            nested: None,
        }
    }
}

/// List container
#[derive(Debug, Clone)]
pub struct AdocList {
    /// List items
    pub items: Vec<AdocListItem>,

    /// Whether ordered (numbered) or unordered (bullets)
    pub ordered: bool,
}

// ============================================================================
// Math
// ============================================================================

/// Math expression
#[derive(Debug, Clone)]
pub struct AdocMath {
    /// Raw math content (as written in source)
    pub content: String,

    /// Parsed expression (if valid)
    pub parsed: Option<AdocExpr>,

    /// Whether this is display mode (block) or inline
    pub display: bool,
}

impl AdocMath {
    /// Create inline math
    pub fn inline(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            parsed: None,
            display: false,
        }
    }

    /// Create display (block) math
    pub fn display(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            parsed: None,
            display: true,
        }
    }
}

// ============================================================================
// Expressions
// ============================================================================

/// Expression (for interpolation and logic)
#[derive(Debug, Clone)]
pub enum AdocExpr {
    /// Literal string
    Literal(String),

    /// Integer literal
    Int(i64),

    /// Float literal
    Float(f64),

    /// Boolean literal
    Bool(bool),

    /// Variable reference (e.g., "name" from ${name})
    Var(String),

    /// Property access (e.g., "user.name" from ${user.name})
    Property {
        object: Box<AdocExpr>,
        property: String,
    },

    /// Binary operation
    Binary {
        left: Box<AdocExpr>,
        op: AdocBinOp,
        right: Box<AdocExpr>,
    },

    /// Unary operation
    Unary {
        op: AdocUnaryOp,
        operand: Box<AdocExpr>,
    },

    /// Function/method call
    Call {
        function: String,
        args: Vec<AdocExpr>,
    },

    /// Method call on object
    MethodCall {
        object: Box<AdocExpr>,
        method: String,
        args: Vec<AdocExpr>,
    },

    /// Array literal
    Array(Vec<AdocExpr>),

    /// Object literal
    Object(Vec<(String, AdocExpr)>),

    /// Ternary conditional
    Ternary {
        condition: Box<AdocExpr>,
        then_expr: Box<AdocExpr>,
        else_expr: Box<AdocExpr>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdocBinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // String
    Concat,

    // Range
    Range,
    RangeInclusive,
}

impl fmt::Display for AdocBinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdocBinOp::Add => write!(f, "+"),
            AdocBinOp::Sub => write!(f, "-"),
            AdocBinOp::Mul => write!(f, "*"),
            AdocBinOp::Div => write!(f, "/"),
            AdocBinOp::Mod => write!(f, "%"),
            AdocBinOp::Pow => write!(f, "^"),
            AdocBinOp::Eq => write!(f, "=="),
            AdocBinOp::Ne => write!(f, "!="),
            AdocBinOp::Lt => write!(f, "<"),
            AdocBinOp::Le => write!(f, "<="),
            AdocBinOp::Gt => write!(f, ">"),
            AdocBinOp::Ge => write!(f, ">="),
            AdocBinOp::And => write!(f, "&&"),
            AdocBinOp::Or => write!(f, "||"),
            AdocBinOp::Concat => write!(f, "++"),
            AdocBinOp::Range => write!(f, ".."),
            AdocBinOp::RangeInclusive => write!(f, "..="),
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdocUnaryOp {
    /// Negation (-x)
    Neg,
    /// Logical not (!x)
    Not,
}

impl fmt::Display for AdocUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdocUnaryOp::Neg => write!(f, "-"),
            AdocUnaryOp::Not => write!(f, "!"),
        }
    }
}

// ============================================================================
// Helper implementations
// ============================================================================

impl AdocInline {
    /// Create a text inline
    pub fn text(content: impl Into<String>) -> Self {
        AdocInline::Text(content.into())
    }

    /// Check if this is empty text
    pub fn is_empty(&self) -> bool {
        match self {
            AdocInline::Text(s) => s.is_empty(),
            _ => false,
        }
    }
}

impl AdocBlock {
    /// Create a paragraph from text
    pub fn paragraph(text: impl Into<String>) -> Self {
        AdocBlock::Paragraph(vec![AdocInline::text(text)])
    }

    /// Create a code block
    pub fn code(code: impl Into<String>) -> Self {
        AdocBlock::CodeBlock {
            lang: None,
            code: code.into(),
        }
    }

    /// Create a code block with language
    pub fn code_with_lang(lang: impl Into<String>, code: impl Into<String>) -> Self {
        AdocBlock::CodeBlock {
            lang: Some(lang.into()),
            code: code.into(),
        }
    }
}

impl AdocDocument {
    /// Create a new empty document
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a document with title
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Self::default()
        }
    }

    /// Add a section to the document
    pub fn add_section(&mut self, section: AdocSection) {
        self.sections.push(section);
    }

    /// Add a block to the preamble
    pub fn add_preamble(&mut self, block: AdocBlock) {
        self.preamble.push(block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = AdocDocument::with_title("Test Document");
        assert_eq!(doc.title, Some("Test Document".to_string()));
        assert!(doc.sections.is_empty());
    }

    #[test]
    fn test_section_creation() {
        let mut section = AdocSection::new(1, "Introduction");
        section.generate_id();

        assert_eq!(section.level, 1);
        assert_eq!(section.title, "Introduction");
        assert_eq!(section.id, Some("introduction".to_string()));
    }

    #[test]
    fn test_paragraph_creation() {
        let para = AdocBlock::paragraph("Hello, world!");
        match para {
            AdocBlock::Paragraph(inlines) => {
                assert_eq!(inlines.len(), 1);
                match &inlines[0] {
                    AdocInline::Text(s) => assert_eq!(s, "Hello, world!"),
                    _ => panic!("Expected Text inline"),
                }
            }
            _ => panic!("Expected Paragraph block"),
        }
    }

    #[test]
    fn test_math_creation() {
        let inline_math = AdocMath::inline("E = mc^2");
        assert!(!inline_math.display);

        let display_math = AdocMath::display("E = mc^2");
        assert!(display_math.display);
    }

    #[test]
    fn test_expression() {
        let expr = AdocExpr::Binary {
            left: Box::new(AdocExpr::Int(1)),
            op: AdocBinOp::Add,
            right: Box::new(AdocExpr::Int(2)),
        };

        match expr {
            AdocExpr::Binary { op, .. } => assert_eq!(op, AdocBinOp::Add),
            _ => panic!("Expected Binary expression"),
        }
    }
}
