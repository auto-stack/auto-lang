# Plan 109: AutoDown Document Format Implementation

## Overview

AutoDown is a text-first document DSL that transpiles to multiple backends (Typst/PDF, DOCX/Word, React/Vue/Web). It uses three core symbols (`#`, `$`, `%{}`) and a Flip mechanism to switch between text and code modes.

**Note**: The math delimiter is `%{ ... }` (opening `%{`, closing `}`), with **no trailing `%`**.

## Design Reference

See `docs/design/auto-down.md` for the full design specification.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   AutoDown Source (.ad)                     │
│                                                              │
│  # Title                                                     │
│  This is ${variable} text.                                   │
│  %{ a + b } is math.                                         │
│  $if cond { content }                                        │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              AutoDown Lexer (Mode-Aware)                     │
│  - Text Mode: paragraphs, lists, inline markup              │
│  - Flip to Code Mode on `$` or `%{`                         │
│  - Flip to Math Mode on `%{`                                │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              AutoDown Parser (Flip Preprocessor)             │
│  - Builds ADOC (Auto Document Object Code) AST              │
│  - Reuses existing Auto parser for code blocks              │
│  - Wraps text blocks as `view { ... }` nodes                │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                  ADOC AST (Document IR)                      │
│  - AdocDocument { title, metadata, sections, ... }          │
│  - AdocSection { level, title, content[] }                  │
│  - AdocContent { Text, Math, Code, ControlFlow }            │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              Backend Transpilers (Multi-Target)              │
│  ├─ TypstTrans → .typ (PDF via typst library)               │
│  ├─ DocxTrans → .docx (Word via docx-rs)                    │
│  └─ HtmlTrans → .html/.vue (Web via React/Vue)              │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1: Core Infrastructure (Week 1-2)

### 1.1 AutoDown Lexer (`autodown/lexer.rs`)

**Purpose**: Tokenize AutoDown source with mode awareness.

**Key Types**:
```rust
/// AutoDown lexer modes
pub enum LexerMode {
    Text,       // Default: paragraphs, lists, inline markup
    Code,       // After `$`: Auto code mode
    Math,       // Inside `%{ ... }`: Math expressions (note: closing with just `}`)
    Interpolate,// Inside `${ ... }`: Expression interpolation
}

/// AutoDown tokens
pub enum AdToken {
    // Text tokens
    Text(String),           // Plain text content
    Newline,                // Line break
    BlankLine,              // Paragraph separator
    
    // Header tokens
    Header { level: u8 },   // # ## ### etc.
    
    // Logic tokens
    Dollar,                 // $ - Logic domain entry
    CodeBlock(String),      // Auto code (parsed by existing parser)
    Interpolate(String),    // ${ expr }
    
    // Math tokens
    MathStart,              // %{
    MathEnd,                // }  (note: NO trailing %)
    MathBlock(String),      // Math content
    
    // Control flow tokens (parsed from $ prefix)
    If, Else, For, In,
    
    // Inline markup (Markdown-compatible)
    BoldStart, BoldEnd,
    ItalicStart, ItalicEnd,
    CodeStart, CodeEnd,     // `code`
    Link { text: String, url: String },
    Image { alt: String, url: String },
    
    // List tokens
    ListItem { depth: u8, marker: ListMarker },
    OrderedListStart,
    
    // Block tokens
    CodeFence { lang: Option<String> },
    Blockquote,
    HorizontalRule,
}
```

**Implementation**:
- [ ] Create `crates/auto-lang/src/autodown/` directory
- [ ] Implement `AdocLexer` with mode switching
- [ ] Handle Flip transitions: Text → Code/Math → Text
- [ ] Support inline markup within text mode
- [ ] Unit tests for each token type

### 1.2 AutoDown AST (`autodown/ast.rs`)

**Purpose**: Define document AST nodes.

**Key Types**:
```rust
/// AutoDown Document (root node)
pub struct AdocDocument {
    pub title: Option<String>,
    pub metadata: AdocMetadata,
    pub sections: Vec<AdocSection>,
    pub preamble: Vec<AdocBlock>,  // Content before first header
}

/// Document metadata (YAML-like front matter)
pub struct AdocMetadata {
    pub author: Option<String>,
    pub date: Option<String>,
    pub custom: HashMap<String, String>,
}

/// Section (header + content)
pub struct AdocSection {
    pub level: u8,           // 1-6
    pub title: String,
    pub id: Option<String>,  // Auto-generated or explicit
    pub content: Vec<AdocBlock>,
    pub subsections: Vec<AdocSection>,
}

/// Block-level content
pub enum AdocBlock {
    Paragraph(Vec<AdocInline>),
    List { items: Vec<AdocListItem>, ordered: bool },
    CodeBlock { lang: Option<String>, code: String },
    Blockquote(Vec<AdocBlock>),
    MathBlock(AdocMath),
    Table { headers: Vec<String>, rows: Vec<Vec<AdocInline>> },
    HorizontalRule,
    
    // Logic blocks (Flip to code mode)
    If { condition: String, then_body: Vec<AdocBlock>, else_body: Option<Vec<AdocBlock>> },
    For { var: String, iterable: String, body: Vec<AdocBlock> },
    
    // Component calls
    Component { name: String, props: HashMap<String, AdocExpr>, children: Vec<AdocBlock> },
    
    // Raw Auto code (style definitions, etc.)
    RawCode(String),
}

/// Inline content
pub enum AdocInline {
    Text(String),
    Bold(Vec<AdocInline>),
    Italic(Vec<AdocInline>),
    Code(String),
    Link { text: String, url: String },
    Image { alt: String, url: String },
    
    // Math (inline)
    Math(AdocMath),
    
    // Interpolation
    Interpolate(AdocExpr),
}

/// Math expression
pub struct AdocMath {
    pub content: String,     // Raw math content
    pub parsed: Option<AdocExpr>,  // Parsed expression (if valid)
}

/// Expression (simplified from AuraExpr)
pub enum AdocExpr {
    Literal(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Var(String),             // Variable reference
    Binary { op: AdocBinOp, left: Box<AdocExpr>, right: Box<AdocExpr> },
    Call { name: String, args: Vec<AdocExpr> },
    FieldAccess { object: Box<AdocExpr>, field: String },
}
```

**Implementation**:
- [ ] Create `autodown/ast.rs` with all AST types
- [ ] Implement `Display` for debugging
- [ ] Add serialization support (serde) for tooling
- [ ] Unit tests for AST construction

### 1.3 AutoDown Parser (`autodown/parser.rs`)

**Purpose**: Parse tokens into ADOC AST.

**Key Features**:
- Recursive descent parser
- Mode-aware parsing (Text → Code → Text)
- Reuse existing Auto parser for code blocks
- Support trailing closures for components

**Implementation**:
```rust
pub struct AdocParser {
    lexer: AdocLexer,
    current: AdToken,
    peek: AdToken,
    mode: ParserMode,
}

impl AdocParser {
    pub fn parse(source: &str) -> Result<AdocDocument, AdocError> {
        let mut parser = Self::new(source);
        parser.parse_document()
    }
    
    fn parse_document(&mut self) -> Result<AdocDocument, AdocError> {
        // 1. Parse front matter (optional)
        // 2. Parse preamble (content before first header)
        // 3. Parse sections recursively
    }
    
    fn parse_section(&mut self, level: u8) -> Result<AdocSection, AdocError> {
        // Parse section title, content, and subsections
    }
    
    fn parse_block(&mut self) -> Result<Option<AdocBlock>, AdocError> {
        // Dispatch based on token type
    }
    
    fn parse_paragraph(&mut self) -> Result<AdocBlock, AdocError> {
        // Parse inline content until blank line
    }
    
    fn parse_if_block(&mut self) -> Result<AdocBlock, AdocError> {
        // $if condition { ... } $else { ... }
    }
    
    fn parse_for_block(&mut self) -> Result<AdocBlock, AdocError> {
        // $for item in .list { ... }
    }
    
    fn parse_component(&mut self) -> Result<AdocBlock, AdocError> {
        // $component_name(prop: value) { children }
    }
    
    fn parse_math(&mut self) -> Result<AdocMath, AdocError> {
        // %{ ... }  (note: closing with just }, no trailing %)
    }
}
```

**Implementation**:
- [ ] Create `autodown/parser.rs`
- [ ] Implement recursive descent parser
- [ ] Handle mode transitions (Flip mechanism)
- [ ] Integrate with existing Auto parser for code blocks
- [ ] Comprehensive error messages with miette
- [ ] Unit tests for each grammar rule

---

## Phase 2: Backend Transpilers (Week 3-4)

### 2.1 Transpiler Trait (`autodown/trans/mod.rs`)

**Purpose**: Define common interface for all transpilers.

```rust
/// Common trait for AutoDown transpilers
pub trait AdocTranspiler {
    /// Transpile a complete document
    fn transpile(&mut self, doc: &AdocDocument) -> Result<String, AdocError>;
    
    /// Transpile a section
    fn transpile_section(&mut self, section: &AdocSection) -> Result<String, AdocError>;
    
    /// Transpile a block
    fn transpile_block(&mut self, block: &AdocBlock) -> Result<String, AdocError>;
    
    /// Transpile inline content
    fn transpile_inline(&mut self, inline: &AdocInline) -> Result<String, AdocError>;
    
    /// Transpile math expression
    fn transpile_math(&mut self, math: &AdocMath) -> Result<String, AdocError>;
    
    /// Transpile expression (for interpolation)
    fn transpile_expr(&mut self, expr: &AdocExpr) -> Result<String, AdocError>;
}

/// Output sink for transpiled code
pub struct AdocSink {
    pub main: String,
    pub styles: String,
    pub front_matter: String,
}
```

### 2.2 Typst Transpiler (`autodown/trans/typst.rs`)

**Purpose**: Generate Typst code for PDF output.

**Key Mappings**:
| AutoDown | Typst |
|----------|-------|
| `# Title` | `= Title` |
| `## Section` | `== Section` |
| `**bold**` | `*bold*` |
| `*italic*` | `_italic_` |
| `%{ math }` | `$ math $` |
| `$if cond { }` | `#if cond { }` |
| `$for x in list { }` | `#for x in list { }` |

**Implementation**:
```rust
pub struct TypstTranspiler {
    indent: usize,
    in_math: bool,
}

impl AdocTranspiler for TypstTranspiler {
    fn transpile(&mut self, doc: &AdocDocument) -> Result<String, AdocError> {
        let mut output = String::new();
        
        // Front matter
        if let Some(title) = &doc.title {
            output.push_str(&format!("#set document(title: \"{}\")\n\n", title));
        }
        
        // Metadata
        self.transpile_metadata(&doc.metadata, &mut output)?;
        
        // Preamble
        for block in &doc.preamble {
            output.push_str(&self.transpile_block(block)?);
            output.push_str("\n\n");
        }
        
        // Sections
        for section in &doc.sections {
            output.push_str(&self.transpile_section(section)?);
        }
        
        Ok(output)
    }
    
    fn transpile_math(&mut self, math: &AdocMath) -> Result<String, AdocError> {
        // Convert function-style math to Typst notation
        // sum(i=0..n, f(i)) → $ sum_i^n f(i) $
        Ok(format!("$ {} $", math.content))
    }
}
```

**Tasks**:
- [ ] Create `autodown/trans/typst.rs`
- [ ] Implement all trait methods
- [ ] Math notation conversion (AutoMath → Typst)
- [ ] Handle control flow (#if, #for)
- [ ] Test cases with expected Typst output

### 2.3 DOCX Transpiler (`autodown/trans/docx.rs`)

**Purpose**: Generate Word documents using `docx-rs` crate.

**Dependencies**: Add to Cargo.toml
```toml
docx-rs = "0.4"
```

**Implementation**:
```rust
use docx_rs::*;

pub struct DocxTranspiler;

impl DocxTranspiler {
    pub fn transpile(&self, doc: &AdocDocument) -> Result<Vec<u8>, AdocError> {
        let mut document = Document::new();
        
        // Add title
        if let Some(title) = &doc.title {
            document = document.add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(title).size(28))
            );
        }
        
        // Add sections
        for section in &doc.sections {
            document = self.add_section(document, section)?;
        }
        
        // Serialize to bytes
        let buf = Vec::new();
        document.pack(buf)?;
        Ok(buf)
    }
    
    fn add_paragraph(&self, block: &AdocBlock) -> Result<Paragraph, AdocError> {
        match block {
            AdocBlock::Paragraph(inlines) => {
                let mut para = Paragraph::new();
                for inline in inlines {
                    para = self.add_inline(para, inline)?;
                }
                Ok(para)
            }
            // ... other block types
        }
    }
    
    fn transpile_math(&self, math: &AdocMath) -> Result<Math, AdocError> {
        // Convert to MathML for Word
        // Use docx-rs Math element
    }
}
```

**Tasks**:
- [ ] Add `docx-rs` dependency
- [ ] Create `autodown/trans/docx.rs`
- [ ] Implement paragraph and text formatting
- [ ] MathML generation for math
- [ ] Table, list, and image support
- [ ] Test with Word document comparison

### 2.4 HTML Transpiler (`autodown/trans/html.rs`)

**Purpose**: Generate HTML for web publishing.

**Key Mappings**:
| AutoDown | HTML |
|----------|------|
| `# Title` | `<h1>Title</h1>` |
| `**bold**` | `<strong>bold</strong>` |
| `%{ math }` | `<span class="math">...</span>` (with MathJax/KaTeX) |
| `$if cond { }` | `{#if cond}...{/if}` (Svelte) or template literal |

**Implementation**:
```rust
pub struct HtmlTranspiler {
    options: HtmlOptions,
}

pub struct HtmlOptions {
    pub math_renderer: MathRenderer,  // MathJax, KaTeX, or plain
    pub framework: Framework,         // Plain, Vue, React, Svelte
}

impl AdocTranspiler for HtmlTranspiler {
    fn transpile(&mut self, doc: &AdocDocument) -> Result<String, AdocError> {
        let mut html = String::new();
        
        // HTML5 document wrapper
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        
        // Title
        if let Some(title) = &doc.title {
            html.push_str(&format!("<title>{}</title>\n", title));
        }
        
        // Math renderer script
        if self.options.math_renderer == MathRenderer::MathJax {
            html.push_str("<script src=\"https://polyfill.io/v3/polyfill.min.js?features=es6\"></script>\n");
            html.push_str("<script id=\"MathJax-script\" async src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\"></script>\n");
        }
        
        html.push_str("</head>\n<body>\n");
        
        // Document content
        for section in &doc.sections {
            html.push_str(&self.transpile_section(section)?);
        }
        
        html.push_str("</body>\n</html>");
        Ok(html)
    }
}
```

**Tasks**:
- [ ] Create `autodown/trans/html.rs`
- [ ] Implement HTML5 generation
- [ ] Math rendering options (MathJax/KaTeX)
- [ ] CSS class injection for styling
- [ ] Framework-specific output (Vue SFC, React component)
- [ ] Test cases

---

## Phase 3: CLI and API Integration (Week 5)

### 3.1 CLI Commands

**Add to `src/main.rs`**:
```rust
// New commands for AutoDown
enum AdocCommand {
    /// Transpile AutoDown to Typst
    Typst { input: String, output: Option<String> },
    /// Transpile AutoDown to DOCX
    Docx { input: String, output: Option<String> },
    /// Transpile AutoDown to HTML
    Html { input: String, output: Option<String> },
    /// Preview AutoDown (launch local server)
    Preview { input: String },
}
```

**Tasks**:
- [ ] Add CLI subcommands for AutoDown
- [ ] File extension: `.ad` or `.autodown`
- [ ] Output file handling
- [ ] Error reporting with miette

### 3.2 Library API

**Add to `src/lib.rs`**:
```rust
pub mod autodown;

/// Parse AutoDown document
pub fn parse_autodown(source: &str) -> Result<AdocDocument, AdocError>;

/// Transpile AutoDown to Typst
pub fn transpile_autodown_typst(doc: &AdocDocument) -> Result<String, AdocError>;

/// Transpile AutoDown to DOCX
pub fn transpile_autodown_docx(doc: &AdocDocument) -> Result<Vec<u8>, AdocError>;

/// Transpile AutoDown to HTML
pub fn transpile_autodown_html(doc: &AdocDocument) -> Result<String, AdocError>;
```

**Tasks**:
- [ ] Public API functions
- [ ] Integration with existing compile session
- [ ] Documentation and examples

---

## Phase 4: Advanced Features (Week 6-7)

### 4.1 AutoMath Parser (`autodown/math.rs`)

**Purpose**: Parse function-style math expressions.

**AutoMath Syntax**:
```
%{
    Z = sum(i=0..infinity, exp(-E_i / (k * T)))
}
```

**Conversion Rules**:
| AutoMath | Target |
|----------|--------|
| `sum(i=a..b, expr)` | `sum_i=a^b expr` (LaTeX/Typst) |
| `prod(i=a..b, expr)` | `prod_i=a^b expr` |
| `integral(a, b, expr)` | `int_a^b expr` |
| `sqrt(x)` | `sqrt(x)` |
| `^` | `^` (superscript) |
| `*` | `·` or implicit |
| `/` | `/` or fraction |

**Implementation**:
```rust
pub struct AutoMathParser;

impl AutoMathParser {
    pub fn parse(input: &str) -> Result<AdocMath, MathError> {
        // Parse function-style math into AST
        // Convert to target notation in transpiler
    }
    
    pub fn to_latex(math: &AdocMath) -> String {
        // Convert to LaTeX notation
    }
    
    pub fn to_typst(math: &AdocMath) -> String {
        // Convert to Typst notation
    }
    
    pub fn to_mathml(math: &AdocMath) -> String {
        // Convert to MathML for Word
    }
}
```

**Tasks**:
- [ ] Create `autodown/math.rs`
- [ ] Parse function-style math
- [ ] Convert to LaTeX, Typst, MathML
- [ ] Handle common functions (sin, cos, log, etc.)
- [ ] Test cases for math conversion

### 4.2 Component System

**Purpose**: Allow reusable document components.

**Syntax**:
```markdown
$callout(type: "warning") {
    **Warning**: This is a dangerous operation.
}

$code_example(lang: "rust") {
    fn main() { println!("Hello"); }
}
```

**Implementation**:
- [ ] Built-in components (callout, code_example, figure, etc.)
- [ ] Custom component definitions
- [ ] Component registry
- [ ] Prop validation

### 4.3 Template Variables

**Purpose**: Support external data injection.

**Syntax**:
```markdown
---
variables:
  name: "World"
---

# Hello, ${name}!
```

**Implementation**:
- [ ] Variable resolution from front matter
- [ ] External JSON/YAML data loading
- [ ] Runtime variable binding (for dynamic documents)

---

## Phase 5: Testing and Documentation (Week 8)

### 5.1 Test Infrastructure

**Test Directory Structure**:
```
crates/auto-lang/test/autodown/
├── 001_basic/
│   ├── basic.ad
│   ├── basic.expected.typ
│   ├── basic.expected.html
│   └── basic.expected.docx
├── 002_headers/
├── 003_lists/
├── 004_math/
├── 005_control_flow/
├── 006_components/
└── ...
```

**Tasks**:
- [ ] Create test framework similar to a2c tests
- [ ] Add test cases for each feature
- [ ] Snapshot testing for transpiler output
- [ ] Integration tests with Typst compiler

### 5.2 Documentation

**Tasks**:
- [ ] Write user guide (docs/autodown-guide.md)
- [ ] Syntax reference
- [ ] Backend comparison table
- [ ] Migration guide from Markdown
- [ ] API documentation (rustdoc)

---

## File Structure Summary

```
crates/auto-lang/src/
├── autodown/
│   ├── mod.rs           # Module exports
│   ├── lexer.rs         # Mode-aware lexer
│   ├── ast.rs           # Document AST types
│   ├── parser.rs        # Recursive descent parser
│   ├── math.rs          # AutoMath parser
│   ├── error.rs         # Error types
│   └── trans/
│       ├── mod.rs       # Transpiler trait
│       ├── typst.rs     # Typst transpiler
│       ├── docx.rs      # DOCX transpiler
│       └── html.rs      # HTML transpiler

crates/auto-lang/test/autodown/
├── 001_basic/
├── 002_headers/
└── ...

docs/
├── autodown-guide.md    # User guide
└── design/auto-down.md  # Design spec (exists)
```

---

## Dependencies

Add to `crates/auto-lang/Cargo.toml`:
```toml
[dependencies]
# Existing dependencies...

# New for AutoDown
docx-rs = { version = "0.4", optional = true }
katex = { version = "0.4", optional = true }  # For HTML math

[features]
default = []
autodown = ["docx-rs"]
autodown-all = ["autodown", "katex"]
```

---

## Success Criteria

1. **Lexer**: Correctly tokenizes AutoDown with mode switching
2. **Parser**: Parses all design doc examples without errors
3. **Typst Backend**: Generates valid Typst that compiles to PDF
4. **DOCX Backend**: Generates valid Word documents with math
5. **HTML Backend**: Generates accessible HTML with math rendering
6. **Tests**: 80%+ coverage, all test cases pass
7. **Documentation**: Complete user guide with examples

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Math parsing complexity | Start with subset, expand incrementally |
| DOCX format changes | Use stable docx-rs library |
| Performance with large docs | Lazy evaluation, streaming output |
| Markdown compatibility | Prioritize CommonMark subset |

---

## Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1-2 | Phase 1 | Lexer, AST, Parser |
| 3-4 | Phase 2 | Typst, DOCX, HTML transpilers |
| 5 | Phase 3 | CLI, API integration |
| 6-7 | Phase 4 | AutoMath, Components, Templates |
| 8 | Phase 5 | Testing, Documentation |

**Total Duration**: 8 weeks (can be parallelized)

---

## Next Steps

1. Create `autodown/` directory structure
2. Implement Phase 1.1: Lexer with mode switching
3. Create initial test cases for validation
4. Iterate on parser design based on test feedback
