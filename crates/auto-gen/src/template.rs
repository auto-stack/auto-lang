use crate::error::{GenError, GenResult, SourceLocation};
use auto_atom::Atom;
use auto_lang::ast::Code;
use auto_lang::interp::Interpreter;
use auto_lang::Universe;
use auto_val::{AutoStr, Shared};
use std::path::PathBuf;

/// A loaded template
#[derive(Clone)]
pub struct Template {
    pub name: AutoStr,
    pub code: Code,
    pub source: AutoStr,
}

/// Template engine that renders Auto scripts as templates
pub struct TemplateEngine {
    fstr_note: char,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self { fstr_note: '$' }
    }

    pub fn with_fstr_note(mut self, note: char) -> Self {
        self.fstr_note = note;
        self
    }

    /// Load a template from a file
    pub fn load(&self, path: &PathBuf) -> GenResult<Template> {
        let source = std::fs::read_to_string(path).map_err(|e| GenError::TemplateLoadError {
            path: path.clone(),
            reason: e.to_string(),
        })?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .into();

        // Don't parse the template as Auto code - store it as-is
        // The template will be transformed by flip_template during rendering
        Ok(Template {
            name,
            code: auto_lang::ast::Code::default(), // Empty AST - not used for templates
            source: source.into(),
        })
    }

    /// Load a template from a string
    pub fn load_from_string(&self, name: impl Into<AutoStr>, source: &str) -> GenResult<Template> {
        let name = name.into();

        // Don't parse the template as Auto code - store it as-is
        // The template will be transformed by flip_template during rendering
        Ok(Template {
            name,
            code: auto_lang::ast::Code::default(), // Empty AST - not used for templates
            source: source.into(),
        })
    }

    /// Render a template with the given data
    pub fn render(&self, template: &Template, data: &Atom) -> GenResult<AutoStr> {
        let mut universe = auto_lang::Universe::new();
        universe.merge_atom(data);

        let mut inter =
            auto_lang::interp::Interpreter::with_scope(universe).with_fstr_note(self.fstr_note);

        // Execute the Auto script as a template
        let result =
            inter
                .eval_template(&template.source)
                .map_err(|e| GenError::TemplateSyntaxError {
                    location: SourceLocation::new(
                        PathBuf::from(format!("template:{}", template.name)),
                        0,
                        0,
                    ),
                    message: e.to_string(),
                })?;

        Ok(result.to_astr())
    }

    /// Render a template using an existing universe (with data already loaded)
    pub fn render_with_universe(
        &self,
        template: &Template,
        universe: &Shared<Universe>,
    ) -> GenResult<AutoStr> {
        // Create an interpreter with the shared universe
        let mut inter = Interpreter::with_univ(universe.clone()).with_fstr_note(self.fstr_note);

        // Use the interpreter to evaluate the template
        let result =
            inter
                .eval_template(&template.source)
                .map_err(|e| GenError::TemplateSyntaxError {
                    location: SourceLocation::new(
                        PathBuf::from(format!("template:{}", template.name)),
                        0,
                        0,
                    ),
                    message: e.to_string(),
                })?;

        Ok(result.to_astr())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Value;

    #[test]
    fn test_load_from_string() {
        let engine = TemplateEngine::new();
        let source = r#"
let x = 42
x
"#;
        let template = engine.load_from_string("test", source).unwrap();
        assert_eq!(template.name, "test");
    }

    #[test]
    fn test_render_simple() {
        let engine = TemplateEngine::new();
        let source = "42";
        let template = engine.load_from_string("test", source).unwrap();
        let atom = Atom::default();
        let result = engine.render(&template, &atom).unwrap();
        assert_eq!(result.trim(), "42");
    }

    #[test]
    fn test_render_with_data() {
        let engine = TemplateEngine::new();
        let source = "42"; // Simple constant
        let template = engine.load_from_string("test", source).unwrap();
        let atom = Atom::default();
        let result = engine.render(&template, &atom).unwrap();
        assert_eq!(result.trim(), "42");
    }
}
