//! Typst Transpiler for AutoDown
//!
//! Generates Typst markup code from ADOC AST.

use super::super::ast::*;
use super::super::error::AdocResult;
use super::{helpers, AdocSink, AdocTranspiler};

/// Typst transpiler
#[derive(Debug, Default)]
pub struct TypstTranspiler {
    /// Indentation level
    indent: usize,

    /// In math mode
    in_math: bool,

    /// Document metadata
    metadata: String,
}

impl TypstTranspiler {
    /// Create a new Typst transpiler
    pub fn new() -> Self {
        Self::default()
    }

    /// Transpile preamble (content before first section)
    fn transpile_preamble(&mut self, blocks: &[AdocBlock]) -> AdocResult<String> {
        let mut output = String::new();

        for block in blocks {
            output.push_str(&self.transpile_block(block)?);
            output.push_str("\n\n");
        }

        Ok(output.trim_end().to_string())
    }

    /// Transpile metadata as Typst header
    fn transpile_metadata(&mut self, metadata: &AdocMetadata) -> AdocResult<String> {
        let mut output = String::new();

        if let Some(author) = &metadata.author {
            output.push_str(&format!("#set text(author: \"{}\")\n", author));
        }

        if let Some(date) = &metadata.date {
            output.push_str(&format!("#set text(date: \"{}\")\n", date));
        }

        for (key, value) in &metadata.custom {
            // Custom metadata - store as variables
            output.push_str(&format!("#let {} = \"{}\"\n", key, value));
        }

        if !output.is_empty() {
            output.push_str("\n");
        }

        Ok(output)
    }

    /// Convert AutoMath to Typst notation
    fn convert_math(&self, content: &str) -> String {
        let mut result = content.to_string();

        // Convert function-style math to Typst notation
        // sum(i=0..n, f(i)) → sum_i^n f(i)
        result = self.convert_math_functions(&result);

        result
    }

    /// Convert function-style math
    fn convert_math_functions(&self, content: &str) -> String {
        // This is a simplified conversion
        // A full implementation would use proper parsing
        let mut result = content.to_string();

        // sum(i=a..b, expr) → sum_i^b expr
        result = self.replace_math_function(&result, "sum");

        // prod(i=a..b, expr) → product_i^b expr
        result = self.replace_math_function(&result, "prod");

        // integral(a, b, expr) → integral_a^b expr
        result = result.replace("integral(", "integral ");

        result
    }

    /// Replace a single math function pattern
    fn replace_math_function(&self, content: &str, func: &str) -> String {
        let pattern = format!("{}(", func);
        let mut result = String::new();
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            // Check for function pattern
            let rest: String = chars.clone().take(pattern.len()).collect();
            if c == pattern.chars().next().unwrap() && rest == pattern[1..] {
                // Consume the rest of the pattern
                for _ in 1..pattern.len() {
                    chars.next();
                }

                result.push_str(&format!("{}_", func));

                // Parse the arguments
                let mut paren_level = 1;
                let mut args = String::new();

                while paren_level > 0 {
                    if let Some(arg_c) = chars.next() {
                        if arg_c == '(' {
                            paren_level += 1;
                        } else if arg_c == ')' {
                            paren_level -= 1;
                            if paren_level == 0 {
                                break;
                            }
                        }
                        args.push(arg_c);
                    } else {
                        break;
                    }
                }

                // Parse i=a..b, expr
                if let Some(dot_pos) = args.find("..") {
                    let before_dot = &args[..dot_pos];
                    let after_dot = &args[dot_pos + 2..];

                    if let Some(comma_pos) = after_dot.find(',') {
                        let end_val = &after_dot[..comma_pos];
                        let expr = &after_dot[comma_pos + 1..];

                        // Extract variable and start value
                        if let Some(eq_pos) = before_dot.find('=') {
                            let var = &before_dot[..eq_pos];
                            result.push_str(var);
                            result.push('^');
                            result.push_str(end_val.trim());
                            result.push(' ');
                            result.push_str(expr.trim());
                        }
                    }
                }
            } else {
                result.push(c);
            }
        }

        result
    }
}

impl AdocTranspiler for TypstTranspiler {
    fn transpile(&mut self, doc: &AdocDocument) -> AdocResult<String> {
        let mut sink = AdocSink::new();

        // Document title
        if let Some(title) = &doc.title {
            sink.front_matter.push_str(&format!(
                "#set document(title: \"{}\")\n",
                helpers::escape_typst(title)
            ));
        }

        // Metadata
        sink.front_matter
            .push_str(&self.transpile_metadata(&doc.metadata)?);

        // Preamble
        if !doc.preamble.is_empty() {
            sink.main.push_str(&self.transpile_preamble(&doc.preamble)?);
            sink.main.push_str("\n\n");
        }

        // Sections
        for section in &doc.sections {
            sink.main.push_str(&self.transpile_section(section)?);
            sink.main.push('\n');
        }

        Ok(sink.output())
    }

    fn extension(&self) -> &'static str {
        "typ"
    }

    fn transpile_section(&mut self, section: &AdocSection) -> AdocResult<String> {
        let mut output = String::new();

        // Section header (Typst uses = for h1, == for h2, etc.)
        let equals = "=".repeat(section.level as usize);
        output.push_str(&format!(
            "{} {}\n\n",
            equals,
            helpers::escape_typst(&section.title)
        ));

        // Section content
        for block in &section.content {
            output.push_str(&self.transpile_block(block)?);
            output.push_str("\n\n");
        }

        // Subsections
        for subsection in &section.subsections {
            output.push_str(&self.transpile_section(subsection)?);
            output.push('\n');
        }

        Ok(output.trim_end().to_string())
    }

    fn transpile_block(&mut self, block: &AdocBlock) -> AdocResult<String> {
        match block {
            AdocBlock::Paragraph(inlines) => {
                let mut content = String::new();
                for inline in inlines {
                    content.push_str(&self.transpile_inline(inline)?);
                }
                Ok(content)
            }

            AdocBlock::List { items, ordered } => {
                let mut output = String::new();

                for item in items {
                    if *ordered {
                        output.push_str("+ ");
                    } else {
                        output.push_str("- ");
                    }

                    for inline in &item.content {
                        output.push_str(&self.transpile_inline(inline)?);
                    }
                    output.push('\n');
                }

                Ok(output.trim_end().to_string())
            }

            AdocBlock::CodeBlock { lang, code } => {
                let mut output = String::new();
                output.push_str("```");
                if let Some(l) = lang {
                    output.push_str(l);
                }
                output.push('\n');
                output.push_str(code);
                output.push_str("\n```\n");
                Ok(output)
            }

            AdocBlock::Blockquote(blocks) => {
                let mut output = String::new();
                output.push_str("#quote[\n");

                for block in blocks {
                    output.push_str(&self.transpile_block(block)?);
                    output.push('\n');
                }

                output.push_str("]\n");
                Ok(output)
            }

            AdocBlock::MathBlock(math) => self.transpile_math(math),

            AdocBlock::Table { headers, rows } => {
                let mut output = String::new();
                output.push_str("#table(\n");
                output.push_str("  columns: ");
                output.push_str(&headers.len().to_string());
                output.push_str(",\n");

                // Headers
                output.push_str("  [\n");
                for (i, header) in headers.iter().enumerate() {
                    output.push_str("    ");
                    output.push_str(&self.transpile_inline(header)?);
                    if i < headers.len() - 1 {
                        output.push_str("],\n    [");
                    }
                }
                output.push_str("],\n");

                // Rows
                for row in rows {
                    output.push_str("  [");
                    for (i, cell) in row.iter().enumerate() {
                        output.push_str(&self.transpile_inline(cell)?);
                        if i < row.len() - 1 {
                            output.push_str("], [");
                        }
                    }
                    output.push_str("],\n");
                }

                output.push_str(")\n");
                Ok(output)
            }

            AdocBlock::HorizontalRule => Ok("---".to_string()),

            AdocBlock::If {
                condition,
                then_body,
                else_body,
            } => {
                let mut output = String::new();
                output.push_str(&format!("#if {} {{\n", condition));

                self.indent += 1;
                for block in then_body {
                    output.push_str(&"  ".repeat(self.indent));
                    output.push_str(&self.transpile_block(block)?);
                    output.push('\n');
                }
                self.indent -= 1;

                if let Some(else_blocks) = else_body {
                    output.push_str("} else {\n");
                    self.indent += 1;
                    for block in else_blocks {
                        output.push_str(&"  ".repeat(self.indent));
                        output.push_str(&self.transpile_block(block)?);
                        output.push('\n');
                    }
                    self.indent -= 1;
                }

                output.push_str("}\n");
                Ok(output)
            }

            AdocBlock::For {
                var,
                index,
                iterable,
                body,
            } => {
                let mut output = String::new();
                if let Some(idx) = index {
                    output.push_str(&format!("#for ({}, {}) in {} {{\n", var, idx, iterable));
                } else {
                    output.push_str(&format!("#for {} in {} {{\n", var, iterable));
                }
                let mut output = String::new();
                output.push_str(&format!("#for {} in {} {{\n", var, iterable));

                self.indent += 1;
                for block in body {
                    output.push_str(&"  ".repeat(self.indent));
                    output.push_str(&self.transpile_block(block)?);
                    output.push('\n');
                }
                self.indent -= 1;

                output.push_str("}\n");
                Ok(output)
            }

            AdocBlock::Component {
                name,
                props,
                children,
            } => {
                let mut output = String::new();

                // Typst uses #for function calls
                output.push_str(&format!("#{}", name));

                if !props.is_empty() {
                    output.push('(');
                    let mut first = true;
                    for (key, value) in props {
                        if !first {
                            output.push_str(", ");
                        }
                        output.push_str(key);
                        output.push_str(": ");
                        output.push_str(&self.transpile_expr(value)?);
                        first = false;
                    }
                    output.push(')');
                }

                if !children.is_empty() {
                    output.push_str("[\n");
                    self.indent += 1;
                    for block in children {
                        output.push_str(&"  ".repeat(self.indent));
                        output.push_str(&self.transpile_block(block)?);
                        output.push('\n');
                    }
                    self.indent -= 1;
                    output.push_str("]\n");
                }

                Ok(output)
            }

            AdocBlock::RawCode(code) => {
                // Raw code - wrap in #[]
                Ok(format!("#{}\n", code))
            }

            AdocBlock::Image { alt, url } => {
                // Image block - Typst uses #image()
                Ok(format!("#image(\"{}\", alt: \"{}\")\n", url, alt))
            }

            AdocBlock::Include(path) => {
                // Include - Typst uses #include
                Ok(format!("#include \"{}\"\n", path))
            }
        }
    }
    fn transpile_inline(&mut self, inline: &AdocInline) -> AdocResult<String> {
        match inline {
            AdocInline::Text(s) => Ok(helpers::escape_typst(s)),

            AdocInline::Bold(content) => {
                let mut inner = String::new();
                for i in content {
                    inner.push_str(&self.transpile_inline(i)?);
                }
                Ok(format!("*{}*", inner))
            }

            AdocInline::Italic(content) => {
                let mut inner = String::new();
                for i in content {
                    inner.push_str(&self.transpile_inline(i)?);
                }
                Ok(format!("_{}_", inner))
            }

            AdocInline::Code(s) => Ok(format!("`{}`", s)),

            AdocInline::Strikethrough(content) => {
                // Typst doesn't have native strikethrough, use #strike
                let mut inner = String::new();
                for i in content {
                    inner.push_str(&self.transpile_inline(i)?);
                }
                Ok(format!("#strike[{}]", inner))
            }

            AdocInline::Link { text, url } => Ok(format!(
                "#link(\"{}\")[{}]",
                url,
                helpers::escape_typst(text)
            )),

            AdocInline::Image { alt, url } => Ok(format!(
                "#image(\"{}\", alt: \"{}\")",
                url,
                helpers::escape_typst(alt)
            )),

            AdocInline::Math(math) => self.transpile_math(math),

            AdocInline::Interpolate(expr) => {
                // In Typst, use #{...} for code interpolation
                Ok(format!("#{{{}}}", self.transpile_expr(expr)?))
            }
        }
    }

    fn transpile_math(&mut self, math: &AdocMath) -> AdocResult<String> {
        let content = self.convert_math(&math.content);

        if math.display {
            Ok(format!("$ {} $", content))
        } else {
            Ok(format!("${}$", content))
        }
    }

    fn transpile_expr(&mut self, expr: &AdocExpr) -> AdocResult<String> {
        match expr {
            AdocExpr::Literal(s) => Ok(format!("\"{}\"", helpers::escape_typst(s))),
            AdocExpr::Int(n) => Ok(n.to_string()),
            AdocExpr::Float(n) => Ok(n.to_string()),
            AdocExpr::Bool(b) => Ok(b.to_string()),
            AdocExpr::Var(name) => Ok(name.clone()),

            AdocExpr::Property { object, property } => {
                Ok(format!("{}.{}", self.transpile_expr(object)?, property))
            }

            AdocExpr::Binary { left, op, right } => Ok(format!(
                "{} {} {}",
                self.transpile_expr(left)?,
                op,
                self.transpile_expr(right)?
            )),

            AdocExpr::Unary { op, operand } => {
                Ok(format!("{}{}", op, self.transpile_expr(operand)?))
            }

            AdocExpr::Call { function, args } => {
                let mut output = format!("#{}(", function);
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| self.transpile_expr(a))
                    .collect::<AdocResult<Vec<_>>>()?;
                output.push_str(&arg_strs.join(", "));
                output.push(')');
                Ok(output)
            }

            AdocExpr::MethodCall {
                object,
                method,
                args,
            } => {
                let mut output = format!("{}.{}(", self.transpile_expr(object)?, method);
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| self.transpile_expr(a))
                    .collect::<AdocResult<Vec<_>>>()?;
                output.push_str(&arg_strs.join(", "));
                output.push(')');
                Ok(output)
            }

            AdocExpr::Array(elements) => {
                let mut output = String::from("(");
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|e| self.transpile_expr(e))
                    .collect::<AdocResult<Vec<_>>>()?;
                output.push_str(&elem_strs.join(", "));
                output.push(')');
                Ok(output)
            }

            AdocExpr::Object(fields) => {
                let mut output = String::from("(");
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| Ok(format!("{}: {}", k, self.transpile_expr(v)?)))
                    .collect::<AdocResult<Vec<_>>>()?;
                output.push_str(&field_strs.join(", "));
                output.push(')');
                Ok(output)
            }

            AdocExpr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => Ok(format!(
                "if {} {{ {} }} else {{ {} }}",
                self.transpile_expr(condition)?,
                self.transpile_expr(then_expr)?,
                self.transpile_expr(else_expr)?
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typst_simple_document() {
        let mut doc = AdocDocument::with_title("Test Document");
        doc.add_preamble(AdocBlock::paragraph("Hello, world!"));

        let mut transpiler = TypstTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();

        assert!(result.contains("Test Document"));
        assert!(result.contains("Hello, world!"));
    }

    #[test]
    fn test_typst_math() {
        let mut doc = AdocDocument::with_title("Math Test");
        doc.add_preamble(AdocBlock::MathBlock(AdocMath::inline("E = mc^2")));

        let mut transpiler = TypstTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();

        assert!(result.contains("$E = mc^2$"));
    }

    #[test]
    fn test_typst_list() {
        let mut doc = AdocDocument::with_title("List Test");
        doc.add_preamble(AdocBlock::List {
            items: vec![
                AdocListItem::simple(vec![AdocInline::text("Item 1")]),
                AdocListItem::simple(vec![AdocInline::text("Item 2")]),
            ],
            ordered: false,
        });

        let mut transpiler = TypstTranspiler::new();
        let result = transpiler.transpile(&doc).unwrap();

        assert!(result.contains("- Item 1"));
        assert!(result.contains("- Item 2"));
    }
}
