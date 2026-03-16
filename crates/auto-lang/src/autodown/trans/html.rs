//! HTML Transpiler for AutoDown - Converts AutoDown AST to HTML for web publishing.

use super::super::ast::*;
use super::super::error::AdocResult;
use super::super::math::AutoMathParser;
use super::{helpers, AdocSink, AdocTranspiler};

/// Math rendering mode for HTML output
#[derive(Debug, Clone, Copy, Default)]
pub enum MathRenderer {
    /// Use MathML (native browser support)
    #[default]
    MathML,
    /// Use KaTeX (requires KaTeX library)
    KaTeX,
    /// Use MathJax (requires MathJax library)
    MathJax,
}

/// HTML generation options
#[derive(Debug, Clone)]
pub struct HtmlOptions {
    /// Math rendering mode
    pub math_renderer: MathRenderer,

    /// Generate standalone HTML document (with <html>, <head>, <body>)
    pub standalone: bool,

    /// CSS class prefix
    pub class_prefix: String,

    /// Include default CSS styles
    pub include_styles: bool,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            math_renderer: MathRenderer::default(),
            standalone: true,
            class_prefix: "adoc-".to_string(),
            include_styles: true,
        }
    }
}

/// HTML transpiler implementation
#[derive(Debug)]
#[allow(dead_code)]
pub struct HtmlTranspiler {
    /// Output sink
    sink: AdocSink,

    /// Generation options
    options: HtmlOptions,

    /// Current section ID counter
    section_counter: usize,
}

impl Default for HtmlTranspiler {
    fn default() -> Self {
        Self {
            sink: AdocSink::new(),
            options: HtmlOptions::default(),
            section_counter: 0,
        }
    }
}

impl HtmlTranspiler {
    /// Create a new HTML transpiler
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom options
    pub fn with_options(options: HtmlOptions) -> Self {
        Self {
            sink: AdocSink::new(),
            options,
            section_counter: 0,
        }
    }

    /// Generate CSS class name
    fn class(&self, name: &str) -> String {
        format!("{}{}", self.options.class_prefix, name)
    }

    /// Convert AutoMath to HTML
    fn convert_math(&self, math: &AdocMath) -> String {
        match self.options.math_renderer {
            MathRenderer::MathML => AutoMathParser::to_mathml(math),
            MathRenderer::KaTeX => format!("${}$", AutoMathParser::to_latex(math)),
            MathRenderer::MathJax => format!("\\({}\\)", AutoMathParser::to_latex(math)),
        }
    }

    /// Generate inline content
    fn transpile_inline_inner(&self, inline: &AdocInline) -> AdocResult<String> {
        let output = match inline {
            AdocInline::Text(text) => helpers::escape_html(text),

            AdocInline::Bold(content) => {
                let inner = self.transpile_inlines_inner(content)?;
                format!(
                    "<strong class=\"{}\">{}</strong>",
                    self.class("bold"),
                    inner
                )
            }

            AdocInline::Italic(content) => {
                let inner = self.transpile_inlines_inner(content)?;
                format!("<em class=\"{}\">{}</em>", self.class("italic"), inner)
            }

            AdocInline::Strikethrough(content) => {
                let inner = self.transpile_inlines_inner(content)?;
                format!("<s class=\"{}\">{}</s>", self.class("strike"), inner)
            }

            AdocInline::Code(code) => {
                format!(
                    "<code class=\"{}\">{}</code>",
                    self.class("code"),
                    helpers::escape_html(code)
                )
            }

            AdocInline::Math(math) => self.convert_math(math),

            AdocInline::Link { text, url } => {
                format!(
                    "<a href=\"{}\" class=\"{}\">{}</a>",
                    helpers::escape_html(url),
                    self.class("link"),
                    helpers::escape_html(text)
                )
            }

            AdocInline::Image { alt, url } => {
                format!(
                    "<img src=\"{}\" alt=\"{}\" class=\"{}\" />",
                    helpers::escape_html(url),
                    helpers::escape_html(alt),
                    self.class("image")
                )
            }

            AdocInline::Interpolate(expr) => {
                // For HTML, we just show the expression as-is
                format!(
                    "<span class=\"{}\">${{{}}}</span>",
                    self.class("interp"),
                    helpers::escape_html(&format!("{:?}", expr))
                )
            }
        };

        Ok(output)
    }

    /// Generate multiple inline elements
    fn transpile_inlines_inner(&self, inlines: &[AdocInline]) -> AdocResult<String> {
        let mut output = String::new();
        for inline in inlines {
            output.push_str(&self.transpile_inline_inner(inline)?);
        }
        Ok(output)
    }

    /// Generate HTML block content
    fn transpile_block_content(&self, block: &AdocBlock) -> AdocResult<String> {
        match block {
            AdocBlock::Paragraph(inlines) => {
                let content = self.transpile_inlines_inner(inlines)?;
                Ok(format!(
                    "<p class=\"{}\">{}</p>",
                    self.class("para"),
                    content
                ))
            }

            AdocBlock::CodeBlock { lang, code } => {
                let lang_class = match lang {
                    Some(l) if !l.is_empty() => format!(" language-{}", l),
                    _ => String::new(),
                };
                Ok(format!(
                    "<pre class=\"{}{}\"><code>{}</code></pre>",
                    self.class("code-block"),
                    lang_class,
                    helpers::escape_html(code)
                ))
            }

            AdocBlock::MathBlock(math) => Ok(format!(
                "<div class=\"{}\">{}</div>",
                self.class("math-block"),
                self.convert_math(math)
            )),

            AdocBlock::List { items, ordered } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let mut output = format!("<{} class=\"{}\">\n", tag, self.class("list"));

                for item in items {
                    let content = self.transpile_inlines_inner(&item.content)?;
                    output.push_str(&format!(
                        "  <li class=\"{}\">{}</li>\n",
                        self.class("list-item"),
                        content
                    ));

                    // Nested list
                    if let Some(nested_list) = &item.nested {
                        let nested = self.transpile_block_content(&AdocBlock::List {
                            items: nested_list.items.clone(),
                            ordered: nested_list.ordered,
                        })?;
                        output.push_str(&format!("  {}\n", nested));
                    }
                }

                output.push_str(&format!("</{}>", tag));
                Ok(output)
            }

            AdocBlock::Blockquote(blocks) => {
                let mut output = format!("<blockquote class=\"{}\">\n", self.class("quote"));

                for block in blocks {
                    output.push_str(&self.transpile_block_content(block)?);
                    output.push('\n');
                }

                output.push_str("</blockquote>");
                Ok(output)
            }

            AdocBlock::Table { headers, rows } => {
                let mut output = format!("<table class=\"{}\">\n", self.class("table"));

                // Headers
                if !headers.is_empty() {
                    output.push_str(&format!("  <thead>\n    <tr>\n"));
                    for header in headers {
                        let content = self.transpile_inline_inner(header)?;
                        output.push_str(&format!("      <th>{}</th>\n", content));
                    }
                    output.push_str("    </tr>\n  </thead>\n");
                }

                // Rows
                output.push_str("  <tbody>\n");
                for row in rows {
                    output.push_str("    <tr>\n");
                    for cell in row {
                        let content = self.transpile_inline_inner(cell)?;
                        output.push_str(&format!("      <td>{}</td>\n", content));
                    }
                    output.push_str("    </tr>\n");
                }
                output.push_str("  </tbody>\n</table>");

                Ok(output)
            }

            AdocBlock::HorizontalRule => Ok(format!("<hr class=\"{}\" />", self.class("break"))),

            AdocBlock::Image { alt, url } => {
                Ok(format!(
                    "<img src=\"{}\" alt=\"{}\" class=\"{}\" />",
                    helpers::escape_html(url),
                    helpers::escape_html(alt),
                    self.class("image")
                ))
            }

            AdocBlock::If { condition, then_body, else_body } => {
                let mut output = format!("<!-- if {} -->\n", condition);
                for block in then_body {
                    output.push_str(&self.transpile_block_content(block)?);
                    output.push('\n');
                }
                if let Some(else_blocks) = else_body {
                    output.push_str("<!-- else -->\n");
                    for block in else_blocks {
                        output.push_str(&self.transpile_block_content(block)?);
                        output.push('\n');
                    }
                }
                output.push_str("<!-- endif -->");
                Ok(output)
            }

            AdocBlock::For { var, index, iterable, body } => {
                let mut output = if let Some(idx) = index {
                    format!("<!-- for ({}, {}) in {} -->\n", var, idx, iterable)
                } else {
                    format!("<!-- for {} in {} -->\n", var, iterable)
                };
                for block in body {
                    output.push_str(&self.transpile_block_content(block)?);
                    output.push('\n');
                }
                output.push_str("<!-- endfor -->");
                Ok(output)
            }

            AdocBlock::Component { name, props, children } => {
                let mut output = format!("<div class=\"{}\" ", name);
                for (key, value) in props {
                    output.push_str(&format!("{}={:?} ", key, value));
                }
                output.push_str(">\n");
                for block in children {
                    output.push_str(&self.transpile_block_content(block)?);
                    output.push('\n');
                }
                output.push_str("</div>");
                Ok(output)
            }

            AdocBlock::RawCode(code) => {
                Ok(format!("<script>\n{}\n</script>", helpers::escape_html(code)))
            }

            AdocBlock::Include(path) => {
                Ok(format!("<!-- include: {} -->", path))
            }
        }
    }

    /// Generate HTML head section
    fn generate_head(&self, doc: &AdocDocument) -> String {
        let mut head = String::new();

        head.push_str("<head>\n");
        head.push_str("  <meta charset=\"UTF-8\" />\n");
        head.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n",
        );

        // Title
        if let Some(title) = &doc.title {
            head.push_str(&format!(
                "  <title>{}</title>\n",
                helpers::escape_html(title)
            ));
        }

        // Math rendering library
        match self.options.math_renderer {
            MathRenderer::KaTeX => {
                head.push_str("  <link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css\" />\n");
                head.push_str("  <script defer src=\"https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js\"></script>\n");
            }
            MathRenderer::MathJax => {
                head.push_str("  <script src=\"https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js\"></script>\n");
            }
            MathRenderer::MathML => {
                // MathML is native, no library needed
            }
        }

        // Default styles
        if self.options.include_styles {
            head.push_str(&self.generate_default_css());
        }

        head.push_str("</head>\n");
        head
    }

    /// Generate default CSS styles
    fn generate_default_css(&self) -> String {
        let prefix = &self.options.class_prefix;
        format!(
            r#"  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 2rem; }}
    .{}h1, .{}h2, .{}h3 {{ margin-top: 1.5em; margin-bottom: 0.5em; }}
    .{}para {{ margin: 1em 0; }}
    .{}code {{ background: #f5f5f5; padding: 0.2em 0.4em; border-radius: 3px; font-family: "Courier New", monospace; }}
    .{}code-block {{ background: #f5f5f5; padding: 1em; overflow-x: auto; border-radius: 4px; }}
    .{}quote {{ border-left: 4px solid #ddd; padding-left: 1em; margin-left: 0; color: #666; }}
    .{}table {{ border-collapse: collapse; width: 100%; }}
    .{}table th, .{}table td {{ border: 1px solid #ddd; padding: 0.5em; text-align: left; }}
    .{}table th {{ background: #f5f5f5; }}
    .{}math-block {{ text-align: center; margin: 1em 0; }}
    .{}link {{ color: #0066cc; text-decoration: none; }}
    .{}link:hover {{ text-decoration: underline; }}
  </style>
"#,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix,
            prefix
        )
    }
}

impl AdocTranspiler for HtmlTranspiler {
    fn extension(&self) -> &'static str {
        "html"
    }

    /// Transpile a complete document
    fn transpile(&mut self, doc: &AdocDocument) -> AdocResult<String> {
        let mut output = String::new();

        // Standalone HTML document
        if self.options.standalone {
            output.push_str("<!DOCTYPE html>\n");
            output.push_str("<html lang=\"en\">\n");
            output.push_str(&self.generate_head(doc));
            output.push_str("<body>\n");

            // Document title
            if let Some(title) = &doc.title {
                output.push_str(&format!(
                    "<h1 class=\"{}\">{}</h1>\n",
                    self.class("title"),
                    helpers::escape_html(title)
                ));
            }

            // Metadata
            if let Some(author) = &doc.metadata.author {
                output.push_str(&format!(
                    "<p class=\"{}\"><strong>Author:</strong> {}</p>\n",
                    self.class("meta"),
                    helpers::escape_html(author)
                ));
            }

            if let Some(date) = &doc.metadata.date {
                output.push_str(&format!(
                    "<p class=\"{}\"><strong>Date:</strong> {}</p>\n",
                    self.class("meta"),
                    helpers::escape_html(date)
                ));
            }
        }

        // Preamble
        for block in &doc.preamble {
            output.push_str(&self.transpile_block_content(block)?);
            output.push_str("\n\n");
        }

        // Sections
        for section in &doc.sections {
            output.push_str(&self.transpile_section(section)?);
            output.push('\n');
        }

        // Close standalone document
        if self.options.standalone {
            output.push_str("</body>\n");
            output.push_str("</html>\n");
        }

        Ok(output.trim_end().to_string())
    }

    /// Transpile a section
    fn transpile_section(&mut self, section: &AdocSection) -> AdocResult<String> {
        self.section_counter += 1;
        let section_id = format!("section-{}", self.section_counter);

        let mut output = String::new();

        // Section heading
        let tag = format!("h{}", section.level.min(6));
        output.push_str(&format!(
            "<{} id=\"{}\" class=\"{}\">{}</{}>\n",
            tag,
            section_id,
            self.class(&format!("h{}", section.level)),
            helpers::escape_html(&section.title),
            tag
        ));

        // Section content
        for block in &section.content {
            output.push_str(&self.transpile_block_content(block)?);
            output.push('\n');
        }

        // Subsections
        for subsection in &section.subsections {
            output.push_str(&self.transpile_section(subsection)?);
        }

        Ok(output)
    }

    /// Transpile a block
    fn transpile_block(&mut self, block: &AdocBlock) -> AdocResult<String> {
        self.transpile_block_content(block)
    }

    /// Transpile inline content
    fn transpile_inline(&mut self, inline: &AdocInline) -> AdocResult<String> {
        self.transpile_inline_inner(inline)
    }

    /// Transpile math expression
    fn transpile_math(&mut self, math: &AdocMath) -> AdocResult<String> {
        Ok(self.convert_math(math))
    }

    /// Transpile expression (for interpolation)
    fn transpile_expr(&mut self, expr: &AdocExpr) -> AdocResult<String> {
        // For HTML output, we can't execute expressions, so we just format them
        Ok(format!("{:?}", expr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_transpiler_basic() {
        let mut transpiler = HtmlTranspiler::new();
        let doc = AdocDocument {
            title: Some("Test Document".to_string()),
            metadata: AdocMetadata::default(),
            preamble: vec![],
            sections: vec![],
        };

        let result = transpiler.transpile(&doc).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>Test Document</title>"));
    }

    #[test]
    fn test_html_inline_text() {
        let mut transpiler = HtmlTranspiler::new();
        let inline = AdocInline::Text("Hello, world!".to_string());
        let result = transpiler.transpile_inline(&inline).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_html_inline_bold() {
        let mut transpiler = HtmlTranspiler::new();
        let inline = AdocInline::Bold(vec![AdocInline::Text("bold".to_string())]);
        let result = transpiler.transpile_inline(&inline).unwrap();
        assert!(result.contains("<strong"));
        assert!(result.contains("bold"));
    }

    #[test]
    fn test_html_escape() {
        let mut transpiler = HtmlTranspiler::new();
        let inline = AdocInline::Text("<script>alert('XSS')</script>".to_string());
        let result = transpiler.transpile_inline(&inline).unwrap();
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
    }
}
