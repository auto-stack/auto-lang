use crate::data::{DataLoader, DataSource, LoadedData};
use crate::error::{GenError, GenResult};
use crate::guard::GuardProcessor;
use crate::template::{Template, TemplateEngine};
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
    /// Library files (.at files) to load before processing templates
    /// These files can contain utility functions used by templates
    pub lib_files: Vec<PathBuf>,
    /// Library search paths for `use` statements
    /// When a template uses `use util: check_on`, the interpreter will
    /// search for `util.at` in these directories
    pub lib_paths: Vec<PathBuf>,
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

        // Set library search paths for data loader (for use statements in data files)
        self.data_loader.set_lib_paths(spec.lib_paths.clone());

        // Load data
        let mut loaded_data = self.data_loader.load(spec.data_source.clone())?;

        // Set library search paths for template engine (for use statements in templates)
        self.template_engine.set_lib_paths(spec.lib_paths.clone());

        // Load library files (.at files) and merge their definitions into the universe
        for lib_path in &spec.lib_files {
            self.load_lib_file(lib_path, &mut loaded_data)?;
        }

        let mut files_generated = Vec::new();
        let mut errors = Vec::new();

        // Process each template
        for template_spec in &spec.templates {
            match self.generate_one(&loaded_data, template_spec) {
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

    fn generate_one(
        &mut self,
        data: &LoadedData,
        template_spec: &TemplateSpec,
    ) -> GenResult<PathBuf> {
        // Load template
        let template = self.template_engine.load(&template_spec.template_path)?;

        // Render template using the interpreter from loaded data
        let rendered = self
            .template_engine
            .render_with_data(&template, data)?;

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

    /// Load a library file (.at) and merge its definitions into the interpreter
    fn load_lib_file(&self, lib_path: &PathBuf, loaded_data: &mut LoadedData) -> GenResult<()> {
        // Read the library file
        let lib_code = std::fs::read_to_string(lib_path).map_err(|e| GenError::TemplateLoadError {
            path: lib_path.clone(),
            reason: e.to_string(),
        })?;

        // Get mutable access to the shared interpreter
        let mut interp = loaded_data.interp.borrow_mut();

        // Evaluate the library code to load function definitions
        let _result = interp.eval(&lib_code);

        // The functions defined in the library are now in the interpreter

        Ok(())
    }
}

/// Builder for CodeGenerator
pub struct CodeGeneratorBuilder {
    config: GeneratorConfig,
    data_source: Option<DataSource>,
    templates: Vec<TemplateSpec>,
    lib_paths: Vec<PathBuf>,
}

impl CodeGeneratorBuilder {
    pub fn new() -> Self {
        Self {
            config: GeneratorConfig::default(),
            data_source: None,
            templates: Vec::new(),
            lib_paths: Vec::new(),
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

    /// Set library search paths for `use` statements
    ///
    /// # Example
    /// ```
    /// use auto_gen::CodeGenerator;
    /// use std::path::PathBuf;
    /// let builder = CodeGenerator::builder()
    ///     .lib_paths(vec![
    ///         PathBuf::from("./generator/utils"),
    ///         PathBuf::from("./shared/templates")
    ///     ]);
    /// ```
    pub fn lib_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.lib_paths = paths;
        self
    }

    /// Add a single library search path for `use` statements
    ///
    /// # Example
    /// ```
    /// use auto_gen::CodeGenerator;
    /// let builder = CodeGenerator::builder()
    ///     .add_lib_path("./generator/utils")
    ///     .add_lib_path("/usr/local/my_modules");
    /// ```
    pub fn add_lib_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.lib_paths.push(path.into());
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

    /// Create a GenerationSpec from the builder configuration
    ///
    /// This is a convenience method that combines all the builder settings
    /// into a GenerationSpec that can be passed to CodeGenerator::generate().
    ///
    /// # Example
    /// ```
    /// use auto_gen::{CodeGenerator, DataSource, GeneratorConfig};
    /// let spec = CodeGenerator::builder()
    ///     .data_source(DataSource::AutoCode("...".to_string()))
    ///     .add_template("template.at", "output.txt")
    ///     .add_lib_path("./utils")
    ///     .create_spec()
    ///     .unwrap();
    ///
    /// let mut generator = CodeGenerator::new(GeneratorConfig::default());
    /// let _ = generator.generate(&spec);
    /// ```
    pub fn create_spec(self) -> GenResult<GenerationSpec> {
        let data_source = self.data_source.ok_or_else(|| {
            GenError::Other("data_source is required to create a GenerationSpec".to_string())
        })?;

        Ok(GenerationSpec {
            data_source,
            templates: self.templates,
            lib_files: Vec::new(),
            lib_paths: self.lib_paths,
        })
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
