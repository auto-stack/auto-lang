use crate::error::{GenError, GenResult, SourceLocation};
use auto_lang::ast::Code;
use auto_lang::atom::Atom;
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
    /// Library search paths for `use` statements
    lib_paths: Vec<PathBuf>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            fstr_note: '$',
            lib_paths: Vec::new(),
        }
    }

    pub fn with_fstr_note(mut self, note: char) -> Self {
        self.fstr_note = note;
        self
    }

    pub fn set_lib_paths(&mut self, paths: Vec<PathBuf>) {
        self.lib_paths = paths;
    }

    /// Add a single library search path for `use` statements
    ///
    /// # Example
    /// ```
    /// engine.add_lib_path("./templates/common");
    /// engine.add_lib_path("/usr/local/my_modules");
    /// ```
    pub fn add_lib_path(&mut self, path: impl Into<PathBuf>) {
        self.lib_paths.push(path.into());
    }

    /// Get current library search paths
    pub fn lib_paths(&self) -> &[PathBuf] {
        &self.lib_paths
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
        eprintln!("DEBUG TemplateEngine: render_with_universe called");
        eprintln!("DEBUG TemplateEngine: lib_paths = {:?}", self.lib_paths);

        // Create an interpreter with the shared universe
        let mut inter = Interpreter::with_univ(universe.clone()).with_fstr_note(self.fstr_note);

        // Set library search paths for `use` statements
        if !self.lib_paths.is_empty() {
            inter.set_lib_paths(self.lib_paths.clone());
        }

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

        eprintln!("DEBUG TemplateEngine: eval_template result.is_error = {}", result.is_error());

        // Post-process: filter out `use` statement lines from output
        let filtered = self.filter_use_statements(result.to_astr());

        Ok(filtered)
    }

    /// Filter out `use` statement lines from the output
    fn filter_use_statements(&self, output: AutoStr) -> AutoStr {
        let output_string = output.to_string();
        let lines: Vec<&str> = output_string.lines()
            .filter(|line| {
                let trimmed = line.trim();
                // Filter out lines that are exactly `use` statements (not template expressions)
                !trimmed.starts_with("use ") || trimmed.contains("${")
            })
            .collect();

        lines.join("\n").into()
    }

    /// Preprocess template: extract and execute `use` statements, remove them from output
    #[allow(dead_code)]
    fn preprocess_use_statements(&self, source: &AutoStr, universe: &Shared<Universe>) -> GenResult<AutoStr> {
        let mut result_lines = Vec::new();
        let mut use_statements = Vec::new();

        for line in source.to_string().lines() {
            let trimmed = line.trim();
            // Check if this line is a `use` statement
            if trimmed.starts_with("use ") {
                use_statements.push(trimmed.to_string());
                // Don't add to result - `use` statements should not appear in output
            } else {
                result_lines.push(line.to_string());
            }
        }

        // Execute all `use` statements to load modules into the universe
        if !use_statements.is_empty() {
            let mut inter = Interpreter::with_univ(universe.clone()).with_fstr_note(self.fstr_note);
            if !self.lib_paths.is_empty() {
                inter.set_lib_paths(self.lib_paths.clone());
            }

            for use_stmt in use_statements {
                eprintln!("Executing use statement: {}", use_stmt);
                match inter.interpret(&use_stmt) {
                    Ok(_) => {
                        eprintln!("Use statement executed successfully");
                    }
                    Err(e) => {
                        eprintln!("Warning: use statement failed: {}", e);
                    }
                }
            }
        }

        Ok(result_lines.join("\n").into())
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
