use auto_gen::{CodeGenerator, DataSource, GenerationSpec, GeneratorConfig, TemplateSpec};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "autogen")]
#[command(about = "AutoLang Code Generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Config file (Auto format)
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate code from data and templates
    Generate {
        /// Data source file (Auto format)
        #[arg(short = 'd', long)]
        data: PathBuf,

        /// Template directory
        #[arg(long = "template-dir")]
        template_dir: Option<PathBuf>,

        /// Template files (can specify multiple)
        #[arg(short = 't', long = "template")]
        template_files: Vec<PathBuf>,

        /// Output directory
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// F-string note character (default: $)
        #[arg(short = 'n', long, default_value_t = '$')]
        note: char,

        /// Dry run (don't write files)
        #[arg(long)]
        dry_run: bool,

        /// Overwrite guarded sections
        #[arg(long)]
        overwrite_guarded: bool,
    },

    /// Validate template syntax
    Validate {
        /// Template file or directory
        templates: PathBuf,
    },

    /// Watch mode (future feature)
    Watch {
        /// Data source file
        #[arg(short = 'd', long)]
        data: PathBuf,

        /// Template directory
        #[arg(long = "template-dir")]
        template_dir: PathBuf,

        /// Output directory
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            data,
            template_dir,
            template_files,
            output,
            note,
            dry_run,
            overwrite_guarded,
        } => {
            // Load config if provided
            let mut config = if let Some(config_path) = cli.config {
                load_config(&config_path)?
            } else {
                GeneratorConfig::default()
            };

            // Override with CLI args
            if let Some(out) = output {
                config.output_dir = out;
            }
            config.dry_run = dry_run;
            config.fstr_note = note;
            config.overwrite_guarded = overwrite_guarded;

            let mut generator = CodeGenerator::new(config);

            // Build generation spec
            let templates = if !template_files.is_empty() {
                template_files
                    .into_iter()
                    .map(|p| TemplateSpec {
                        template_path: p,
                        output_name: None,
                        rename: false,
                    })
                    .collect()
            } else if let Some(dir) = template_dir {
                discover_templates(&dir)?
            } else {
                eprintln!("Error: Must specify --template or --template-dir");
                std::process::exit(1);
            };

            let spec = GenerationSpec {
                data_source: DataSource::AutoFile(data),
                templates,
            };

            match generator.generate(&spec) {
                Ok(report) => {
                    println!(
                        "Generated {} files in {:?}",
                        report.files_generated.len(),
                        report.duration
                    );
                    for file in &report.files_generated {
                        println!("  {}", file.display());
                    }
                    if !report.errors.is_empty() {
                        eprintln!("\nErrors occurred:");
                        for error in &report.errors {
                            eprintln!("  {}", error);
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Validate { templates } => {
            validate_templates(&templates)?;
        }

        Commands::Watch { .. } => {
            eprintln!("Error: Watch mode is not yet implemented");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn load_config(path: &PathBuf) -> Result<GeneratorConfig, Box<dyn std::error::Error>> {
    // For now, return default config
    // TODO: Implement Auto-format config file parsing
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

    // Parse Auto config file
    // This is a simplified version - full implementation would parse the Auto file
    // and extract config values
    let mut config = GeneratorConfig::default();

    // Simple key-value parsing for common config options
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            match key {
                "output_dir" => config.output_dir = PathBuf::from(value),
                "fstr_note" => {
                    if value.len() == 1 {
                        config.fstr_note = value.chars().next().unwrap();
                    }
                }
                "overwrite_guarded" => config.overwrite_guarded = value == "true",
                _ => {}
            }
        }
    }

    Ok(config)
}

fn discover_templates(dir: &PathBuf) -> Result<Vec<TemplateSpec>, Box<dyn std::error::Error>> {
    let mut templates = Vec::new();

    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Failed to read template directory: {}", e))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("at") {
            templates.push(TemplateSpec {
                template_path: path,
                output_name: None,
                rename: false,
            });
        }
    }

    if templates.is_empty() {
        return Err(format!("No template files (.at) found in {}", dir.display()).into());
    }

    Ok(templates)
}

fn validate_templates(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_file() {
        // Validate single template file
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read template file: {}", e))?;

        // Try to parse as template
        match auto_lang::interp::Interpreter::new().eval_template(&source) {
            Ok(_) => {
                println!("✓ {}", path.display());
            }
            Err(e) => {
                eprintln!("✗ {}: {}", path.display(), e);
                std::process::exit(1);
            }
        }
    } else if path.is_dir() {
        // Validate all template files in directory
        let entries =
            std::fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

        let mut valid_count = 0;
        let mut error_count = 0;

        for entry in entries {
            let entry = entry?;
            let template_path = entry.path();

            if template_path.extension().and_then(|s| s.to_str()) == Some("at") {
                match std::fs::read_to_string(&template_path) {
                    Ok(source) => {
                        match auto_lang::interp::Interpreter::new().eval_template(&source) {
                            Ok(_) => {
                                println!("✓ {}", template_path.display());
                                valid_count += 1;
                            }
                            Err(e) => {
                                eprintln!("✗ {}: {}", template_path.display(), e);
                                error_count += 1;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ {}: Failed to read: {}", template_path.display(), e);
                        error_count += 1;
                    }
                }
            }
        }

        println!(
            "\nValidation complete: {} valid, {} errors",
            valid_count, error_count
        );

        if error_count > 0 {
            std::process::exit(1);
        }
    } else {
        return Err("Path is neither a file nor a directory".into());
    }

    Ok(())
}
