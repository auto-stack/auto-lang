use crate::data::{DataLoader, DataSource};
use crate::error::{GenError, GenResult};
use crate::guard::GuardProcessor;
use crate::template::{Template, TemplateEngine};
use auto_atom::Atom;
use auto_val::AutoStr;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the code generator
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub output_dir: PathBuf,
    pub dry_run: bool,
    pub fstr_note: char,
    pub overwrite_guarded: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("."),
            dry_run: false,
            fstr_note: '$',
            overwrite_guarded: false,
        }
    }
}

/// Specification for a generation task
pub struct GenerationSpec {
    pub data_source: DataSource,
    pub templates: Vec<TemplateSpec>,
}

/// Specification for a single template
pub struct TemplateSpec {
    pub template_path: PathBuf,
    pub output_name: Option<AutoStr>,
    pub rename: bool,
}

/// Report from a generation task
pub struct GenReport {
    pub files_generated: Vec<PathBuf>,
    pub errors: Vec<GenError>,
    pub duration: Duration,
}

/// Main code generator
pub struct CodeGenerator {
    data_loader: DataLoader,
    template_engine: TemplateEngine,
    guard_processor: GuardProcessor,
    config: GeneratorConfig,
}

impl CodeGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            data_loader: DataLoader::new(),
            template_engine: TemplateEngine::new().with_fstr_note(config.fstr_note),
            guard_processor: GuardProcessor::new(),
            config,
        }
    }

    /// Builder pattern: start building a generator
    pub fn builder() -> CodeGeneratorBuilder {
        CodeGeneratorBuilder::new()
    }

    /// Generate code from a specification
    pub fn generate(&mut self, spec: &GenerationSpec) -> GenResult<GenReport> {
        let start = std::time::Instant::now();

        // Load data
        let data = self.data_loader.load(spec.data_source.clone())?;

        let mut files_generated = Vec::new();
        let mut errors = Vec::new();

        // Process each template
        for template_spec in &spec.templates {
            match self.generate_one(&data, template_spec) {
                Ok(output_path) => {
                    files_generated.push(output_path);
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        Ok(GenReport {
            files_generated,
            errors,
            duration: start.elapsed(),
        })
    }

    fn generate_one(&mut self, data: &Atom, template_spec: &TemplateSpec) -> GenResult<PathBuf> {
        // Load template
        let template = self.template_engine.load(&template_spec.template_path)?;

        // Render template
        let rendered = self.template_engine.render(&template, data)?;

        // Determine output path
        let output_path = self.resolve_output_path(&template_spec, &template)?;

        // Process guard blocks if file exists
        let final_output = if output_path.exists() && !self.config.overwrite_guarded {
            let existing = std::fs::read_to_string(&output_path)?;
            self.guard_processor
                .merge(&existing, &rendered.to_string())?
        } else {
            rendered.to_string()
        };

        // Write output
        if !self.config.dry_run {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&output_path, final_output.as_bytes())?;
        }

        Ok(output_path)
    }

    fn resolve_output_path(
        &self,
        template_spec: &TemplateSpec,
        template: &Template,
    ) -> GenResult<PathBuf> {
        let output_name = if let Some(name) = &template_spec.output_name {
            name.to_string()
        } else {
            // Use template name, remove trailing .at if present
            let name = template.name.to_string();
            if name.ends_with(".at") {
                name[..name.len() - 3].to_string()
            } else {
                name
            }
        };

        Ok(self.config.output_dir.join(&output_name))
    }
}

/// Builder for CodeGenerator
pub struct CodeGeneratorBuilder {
    config: GeneratorConfig,
    data_source: Option<DataSource>,
    templates: Vec<TemplateSpec>,
}

impl CodeGeneratorBuilder {
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
            data_source: None,
            templates: Vec::new(),
        }
    }

    pub fn output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.output_dir = dir.into();
        self
    }

    pub fn fstr_note(mut self, note: char) -> Self {
        self.config.fstr_note = note;
        self
    }

    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.config.dry_run = dry_run;
        self
    }

    pub fn overwrite_guarded(mut self, overwrite: bool) -> Self {
        self.config.overwrite_guarded = overwrite;
        self
    }

    pub fn data_source(mut self, source: DataSource) -> Self {
        self.data_source = Some(source);
        self
    }

    pub fn add_template(
        mut self,
        template_path: impl Into<PathBuf>,
        output_name: impl Into<AutoStr>,
    ) -> Self {
        self.templates.push(TemplateSpec {
            template_path: template_path.into(),
            output_name: Some(output_name.into()),
            rename: false,
        });
        self
    }

    pub fn build(self) -> GenResult<CodeGenerator> {
        Ok(CodeGenerator::new(self.config))
    }
}

impl Default for CodeGeneratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Value;

    #[test]
    fn test_generator_config_default() {
        let config = GeneratorConfig::default();
        assert_eq!(config.output_dir, PathBuf::from("."));
        assert_eq!(config.fstr_note, '$');
        assert!(!config.dry_run);
        assert!(!config.overwrite_guarded);
    }

    #[test]
    fn test_builder_pattern() {
        let builder = CodeGenerator::builder()
            .output_dir("./output")
            .fstr_note('@')
            .dry_run(true);

        assert_eq!(builder.config.output_dir, PathBuf::from("./output"));
        assert_eq!(builder.config.fstr_note, '@');
        assert!(builder.config.dry_run);
    }

    #[test]
    fn test_resolve_output_path() {
        let config = GeneratorConfig::default();
        let generator = CodeGenerator::new(config);

        let template_spec = TemplateSpec {
            template_path: PathBuf::from("test.txt.at"),
            output_name: None,
            rename: false,
        };

        let template = Template {
            name: "test.txt.at".into(),
            code: auto_lang::ast::Code::default(),
            source: "".into(),
        };

        let output_path = generator
            .resolve_output_path(&template_spec, &template)
            .unwrap();
        assert!(output_path.to_str().unwrap().ends_with("test.txt"));
    }
}
