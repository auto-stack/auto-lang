use crate::data::{DataLoader, DataSource, LoadedData};
use crate::template::TemplateEngine;
use auto_gen_core::{
    GenError, GenReport, OutputConfig, RenderedTemplate, TemplateRenderer, TemplateSpec,
};
use std::path::PathBuf;

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

/// Specification for a generation task (Auto adapter side: the data source is
/// interpreter-bound; templates are handed to auto-gen-core as-is).
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

/// Auto executable-template renderer: loads a template file and renders it
/// with the shared interpreter from `LoadedData`. This is the adapter side of
/// the auto-gen-core boundary — the core receives owned text and never sees
/// the interpreter, AST or template types.
struct AutoTemplateRenderer<'a> {
    engine: &'a TemplateEngine,
    data: &'a LoadedData,
}

impl TemplateRenderer for AutoTemplateRenderer<'_> {
    fn render(&mut self, spec: &TemplateSpec) -> Result<RenderedTemplate, GenError> {
        let template = self.engine.load(&spec.template_path)?;
        let rendered = self.engine.render_with_data(&template, self.data)?;
        Ok(RenderedTemplate {
            template_name: template.name.clone(),
            content: rendered.to_string(),
        })
    }
}

/// Main code generator: Auto data/template adapter over auto-gen-core.
pub struct CodeGenerator {
    data_loader: DataLoader,
    template_engine: TemplateEngine,
    config: GeneratorConfig,
}

impl CodeGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            data_loader: DataLoader::new(),
            template_engine: TemplateEngine::new().with_fstr_note(config.fstr_note),
            config,
        }
    }

    /// Builder pattern: start building a generator
    pub fn builder() -> CodeGeneratorBuilder {
        CodeGeneratorBuilder::new()
    }

    /// Generate code from a specification.
    ///
    /// Phase boundary (frozen at extraction, PLAN-791): data and library
    /// loading failures abort the whole call with a top-level error; each
    /// template failure is collected into the report and the batch continues.
    /// Output naming, guard preservation, dry-run and write semantics are
    /// owned by [`auto_gen_core::generate`].
    pub fn generate(&mut self, spec: &GenerationSpec) -> Result<GenReport, GenError> {
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

        let mut renderer = AutoTemplateRenderer {
            engine: &self.template_engine,
            data: &loaded_data,
        };
        let mut report = auto_gen_core::generate(
            &spec.templates,
            &OutputConfig {
                output_dir: self.config.output_dir.clone(),
                dry_run: self.config.dry_run,
                overwrite_guarded: self.config.overwrite_guarded,
            },
            &mut renderer,
        );
        // Keep the historical duration semantics: the clock covers the whole
        // generate() call including the data/library load stage, not just the
        // core's template loop.
        report.duration = start.elapsed();
        Ok(report)
    }

    /// Load a library file (.at) and merge its definitions into the interpreter
    fn load_lib_file(
        &self,
        lib_path: &PathBuf,
        loaded_data: &mut LoadedData,
    ) -> Result<(), GenError> {
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
        output_name: impl Into<auto_val::AutoStr>,
    ) -> Self {
        self.templates.push(TemplateSpec {
            template_path: template_path.into(),
            output_name: Some(output_name.into()),
            rename: false,
        });
        self
    }

    pub fn build(self) -> Result<CodeGenerator, GenError> {
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
    pub fn create_spec(self) -> Result<GenerationSpec, GenError> {
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
}
