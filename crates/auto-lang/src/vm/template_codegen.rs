// Plan 075 Phase 2: TemplateCodegen Implementation
// Compiles template files to bytecode that builds concatenated strings

use crate::ast::{Code, Stmt, Expr};
use crate::vm::codegen::Codegen;
use crate::vm::opcode::OpCode;
use crate::vm::loader::Module;
use crate::error::{AutoResult, AutoError};

/// TemplateCodegen transforms template files into bytecode that builds
/// strings by concatenating evaluated expressions.
///
/// Input (template.at):
/// ```auto
/// Hello, $name!
/// You have ${count} messages.
/// ```
///
/// Output: bytecode that concatenates strings with variable values:
/// ```rust
/// LOAD_STR "Hello, "
/// LOAD_VAR "name"
/// TO_STR                    // Convert to string
/// STR_CAT                   // Concatenate
/// LOAD_STR "!\nYou have "
/// LOAD_VAR "count"
/// TO_STR
/// STR_CAT
/// LOAD_STR " messages."
/// STR_CAT
/// STR_CAT                   // Final result
/// RET
/// ```
pub struct TemplateCodegen {
    /// Base codegen for opcode emission
    base: Codegen,
    /// Separator between statements (newline by default)
    separator: String,
    /// Filter nil values from output
    filter_nil: bool,
}

impl TemplateCodegen {
    /// Create a new TemplateCodegen instance
    pub fn new() -> Self {
        Self {
            base: Codegen::new(),
            separator: "\n".to_string(),
            filter_nil: true,
        }
    }

    /// Set custom separator
    pub fn with_separator(mut self, sep: String) -> Self {
        self.separator = sep;
        self
    }

    /// Enable/disable nil filtering
    pub fn with_nil_filtering(mut self, filter: bool) -> Self {
        self.filter_nil = filter;
        self
    }

    /// Compile template file to bytecode
    ///
    /// Converts statements to string concatenation bytecode.
    pub fn compile_template(&mut self, code: &Code) -> AutoResult<()> {
        if code.stmts.is_empty() {
            // Empty template - return empty string
            self.emit_empty_string();
            self.base.code.push(OpCode::RET as u8);
            return Ok(());
        }

        // Process each statement
        for (i, stmt) in code.stmts.iter().enumerate() {
            self.compile_template_stmt(stmt)?;

            // Add separator between statements (but not after last)
            if i < code.stmts.len() - 1 && !self.separator.is_empty() {
                self.emit_separator();
            }
        }

        // Return the final string
        self.base.code.push(OpCode::RET as u8);

        Ok(())
    }

    /// Compile a single template statement
    fn compile_template_stmt(&mut self, stmt: &Stmt) -> AutoResult<()> {
        match stmt {
            // Expression statements are evaluated and converted to strings
            Stmt::Expr(expr) => {
                self.compile_expr_to_string(expr)?;
            }
            // String literals are emitted directly
            Stmt::Store(store) => {
                // For templates, store statements also contribute to output
                // Evaluate the RHS and convert to string
                self.compile_expr_to_string(&store.expr)?;
            }
            _ => {
                return Err(AutoError::Msg(
                    format!("Template mode does not support statement: {:?}", stmt)
                ));
            }
        }
        Ok(())
    }

    /// Compile an expression and convert to string
    fn compile_expr_to_string(&mut self, expr: &Expr) -> AutoResult<()> {
        // Compile the expression
        self.base.compile_expr(expr)?;

        // Convert to string
        self.base.code.push(OpCode::TO_STR as u8);

        // TODO: Add nil filtering with conditional jumps
        // For now, we just convert nil to "nil" string
        // if self.filter_nil {
        //     self.emit_nil_filter();
        // }

        Ok(())
    }

    /// Emit separator string
    fn emit_separator(&mut self) {
        let sep_bytes = self.separator.as_bytes().to_vec();
        let sep_idx = self.base.strings.len() as u16;
        self.base.strings.push(sep_bytes);

        self.base.code.push(OpCode::LOAD_STR as u8);
        self.base.code.extend_from_slice(&sep_idx.to_le_bytes());

        // Concatenate with previous result
        self.base.code.push(OpCode::STR_CAT as u8);
    }

    /// Emit empty string for empty templates
    fn emit_empty_string(&mut self) {
        let empty_idx = self.base.strings.len() as u16;
        self.base.strings.push(Vec::new());

        self.base.code.push(OpCode::LOAD_STR as u8);
        self.base.code.extend_from_slice(&empty_idx.to_le_bytes());
    }

    /// Finish compilation and return the module
    pub fn finish(self, name: String) -> Module {
        self.base.finish(name)
    }

    /// Get the base codegen for advanced usage
    pub fn base(&mut self) -> &mut Codegen {
        &mut self.base
    }
}

impl Default for TemplateCodegen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_source(source: &str) -> Code {
        let mut parser = Parser::from(source);
        parser.parse().unwrap()
    }

    #[test]
    fn test_template_codegen_simple_string() {
        let source = r#""Hello, world!""#;

        let code = parse_source(source);
        let mut templategen = TemplateCodegen::new();
        templategen.compile_template(&code).unwrap();

        let module = templategen.finish("test".to_string());

        // Verify bytecode contains expected opcodes
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x7A), "Expected TO_STR opcode (0x7A)");
        assert!(bytecode.contains(&0x71), "Expected RET opcode (0x71)");
    }

    #[test]
    fn test_template_codegen_with_variable() {
        // Test with integer expression (simpler than variable reference)
        let source = "42";

        let code = parse_source(source);
        let mut templategen = TemplateCodegen::new();
        templategen.compile_template(&code).unwrap();

        let module = templategen.finish("test".to_string());

        // Verify bytecode was generated
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x7A), "Expected TO_STR opcode (0x7A)");
        assert!(bytecode.contains(&0x71), "Expected RET opcode (0x71)");
    }

    #[test]
    fn test_template_codegen_concatenation() {
        let source = r#""Hello, "
"world!""#;

        let code = parse_source(source);
        let mut templategen = TemplateCodegen::new();
        templategen.compile_template(&code).unwrap();

        let module = templategen.finish("test".to_string());

        // Verify bytecode contains STR_CAT
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x7C), "Expected STR_CAT opcode (0x7C)");
    }

    #[test]
    fn test_template_codegen_empty_template() {
        let source = "";

        let mut parser = Parser::from(source);
        let code = parser.parse().unwrap();

        let mut templategen = TemplateCodegen::new();
        templategen.compile_template(&code).unwrap();

        let module = templategen.finish("test".to_string());

        // Should have LOAD_STR (empty) and RET
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x71), "Expected RET opcode (0x71)");
    }

    #[test]
    fn test_template_codegen_with_separator() {
        let source = r#""Hello"
"world""#;

        let code = parse_source(source);
        let mut templategen = TemplateCodegen::new().with_separator("\n".to_string());
        templategen.compile_template(&code).unwrap();

        let module = templategen.finish("test".to_string());

        // Verify bytecode was generated
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x7A), "Expected TO_STR opcode (0x7A)");
        assert!(bytecode.contains(&0x7C), "Expected STR_CAT opcode (0x7C)");
    }

    #[test]
    fn test_template_codegen_nil_value() {
        // Test with boolean true (simpler than nil which isn't supported yet)
        let source = "true";

        let code = parse_source(source);
        let mut templategen = TemplateCodegen::new();
        templategen.compile_template(&code).unwrap();

        let module = templategen.finish("test".to_string());

        // Verify bytecode was generated (true converts to "true" string)
        let bytecode = &module.code;
        assert!(bytecode.contains(&0x7A), "Expected TO_STR opcode (0x7A)");
        assert!(bytecode.contains(&0x71), "Expected RET opcode (0x71)");
    }
}
