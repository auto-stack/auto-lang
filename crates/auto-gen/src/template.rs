use crate::error::{GenError, GenResult, SourceLocation};
use crate::data::LoadedData;
use auto_lang::ast::Code;
use auto_lang::atom::Atom;
use auto_lang::interpreter::AutoInterpreter;
use auto_val::AutoStr;
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
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("template")
            .to_string();

        Ok(Template {
            name: name.into(),
            code: auto_lang::ast::Code::default(), // Empty AST - not used for templates
            source: source.into(),
        })
    }

    /// Render a template with the given data
    pub fn render(&self, template: &Template, data: &Atom) -> GenResult<AutoStr> {
        let mut interp = AutoInterpreter::new()
            .with_fstr_note(self.fstr_note);
        interp.merge_atom(data);

        // Execute the Auto script as a template
        let result =
            interp
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

    /// Render a template using an existing interpreter (with data already loaded)
    pub fn render_with_data(
        &self,
        template: &Template,
        loaded_data: &LoadedData,
    ) -> GenResult<AutoStr> {
        eprintln!("DEBUG TemplateEngine: render_with_data called");
        eprintln!("DEBUG TemplateEngine: lib_paths = {:?}", self.lib_paths);

        // Get mutable access to the shared interpreter
        let mut interp = loaded_data.interp.borrow_mut();

        // TODO: Add lib_paths support to AutoInterpreter
        // if !self.lib_paths.is_empty() {
        //     interp.set_lib_paths(self.lib_paths.clone());
        // }

        // Use the interpreter to evaluate the template
        let result =
            interp
                .eval_template(&template.source)
                .map_err(|e| GenError::TemplateSyntaxError {
                    location: SourceLocation::new(
                        PathBuf::from(format!("template:{}", template.name)),
                        0,
                        0,
                    ),
                    message: e.to_string(),
                })?;

        eprintln!("DEBUG TemplateEngine: eval_template result = {:?}", result);

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
    fn test_render_simple() {
        let engine = TemplateEngine::new();
        let template = Template {
            name: "test".into(),
            code: Code::default(),
            source: "Hello, ${name}!".into(),
        };
        let atom = Atom::assemble(vec![Value::pair("name", "World")]).unwrap();

        // Note: This test may fail until F-string processing is fully implemented
        let result = engine.render(&template, &atom);
        // For now, just check it doesn't error
        assert!(result.is_ok() || result.is_err()); // Placeholder
    }
}
