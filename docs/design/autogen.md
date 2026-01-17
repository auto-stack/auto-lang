# Auto-Gen General Code Generator - Design Document

## Executive Summary

Design and implementation of a general-purpose code generator that:
- Takes data in **Auto (Atom) format** (Auto format is a superset of JSON)
- Uses **Auto scripts as templates** (templates are AutoLang code themselves)
- Generates code files with support for incremental updates through guard blocks
- Provides both **CLI tool** and **library API** for flexible integration

---

## Design Decisions Summary

Based on user requirements, the following design decisions have been made:

1. **Data Format**: Auto (Atom) format only - no separate JSON/YAML/TOML parsers
2. **Templates**: Auto scripts themselves - use `use` statement for includes, no special template directives
3. **Guard Blocks**: C-style only (`/// ---------- begin of guard: <id> ---`)
4. **Use Case**: Both CLI and library API equally important
5. **API Compatibility**: Flexible redesign - can break existing `AutoGen`/`Mold`/`OneGen` APIs
6. **Template Caching**: Not needed - small batch generation
7. **Watch Mode**: Important feature, implement in later phase
8. **Configuration**: CLI arguments + Auto-format config file
9. **Error Messages**: Full IDE-style with syntax highlighting and suggestions
10. **Performance**: Moderate - reasonably fast but no hard targets

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Data Input System](#2-data-input-system)
3. [Template System](#3-template-system)
4. [Code Generation Process](#4-code-generation-process)
5. [Guard Block System](#5-guard-block-system)
6. [API Design](#6-api-design)
7. [CLI Tool](#7-cli-tool)
8. [Configuration System](#8-configuration-system)
9. [Error Handling](#9-error-handling)
10. [Implementation Phases](#10-implementation-phases)
11. [Critical Files](#11-critical-files)

---

## 1. Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Code Generator                           │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────┐    ┌──────────┐    ┌──────────────────┐     │
│  │   Auto   │    │   Auto   │    │   Auto Scripts   │     │
│  │  Loader  │    │  Config  │    │   (.at templates)│     │
│  └────┬─────┘    └────┬─────┘    └────────┬─────────┘     │
│       │               │                    │               │
│       ▼               ▼                    ▼               │
│  ┌─────────────────────────────────────────────────┐      │
│  │           Data Normalization Layer              │      │
│  │  • Auto (Atom) format loading                  │      │
│  │  • Validation & schema checking                 │      │
│  └────────────────────┬────────────────────────────┘      │
│                       │                                     │
│                       ▼                                     │
│  ┌─────────────────────────────────────────────────┐      │
│  │         Template Engine (Auto Interpreter)       │      │
│  │  • Parse Auto scripts as templates              │      │
│  │  • Execute templates with bound data            │      │
│  │  • Support for `use` statements (includes)       │      │
│  └────────────────────┬────────────────────────────┘      │
│                       │                                     │
│                       ▼                                     │
│  ┌─────────────────────────────────────────────────┐      │
│  │          Guard Block Processor                  │      │
│  │  • Preserve hand-written code                   │      │
│  │  • C-style guard block format                   │      │
│  │  • Merge strategy                              │      │
│  └────────────────────┬────────────────────────────┘      │
│                       │                                     │
│                       ▼                                     │
│  ┌─────────────────────────────────────────────────┐      │
│  │           Output Generator                      │      │
│  │  • File writing                                 │      │
│  │  • Directory management                         │      │
│  │  • Dry-run mode                                 │      │
│  └─────────────────────────────────────────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **Auto-First**: Everything is Auto format - data, config, and templates
2. **Templates as Code**: Templates are Auto scripts, leveraging full language power
3. **Dual Interface**: First-class CLI tool and library API
4. **Flexible API**: Modern, clean API design without backward compatibility constraints
5. **Developer Experience**: IDE-style errors, clear messages, helpful suggestions

---

## 2. Data Input System

### Design

Since Auto format is a superset of JSON and is the primary data format, the data input system is straightforward:

```rust
use auto_atom::Atom;
use auto_val::AutoPath;
use std::path::PathBuf;

pub enum DataSource {
    AutoFile(PathBuf),        // .at files
    AutoCode(String),         // Auto code as string
    Atom(Atom),               // Direct Atom structure
}

pub struct DataLoader {
    // No cache needed for small batches
}

impl DataLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load(&self, source: DataSource) -> Result<Atom, GenError> {
        match source {
            DataSource::Atom(atom) => Ok(atom),
            DataSource::AutoFile(path) => {
                // Parse Auto file and convert to Atom
                let code = std::fs::read_to_string(&path)
                    .map_err(|e| GenError::DataLoadError {
                        path: path.clone(),
                        reason: e.to_string(),
                    })?;

                // Use AutoLang parser to create Code AST
                // Then extract data structures to create Atom
                self.parse_auto_to_atom(&code, path)
            },
            DataSource::AutoCode(code) => {
                self.parse_auto_to_atom(&code, PathBuf::from("<code>"))
            },
        }
    }

    fn parse_auto_to_atom(&self, code: &str, path: PathBuf) -> Result<Atom, GenError> {
        // Parse Auto code to extract data structures
        // Convert enums, structs, arrays to Atom format
        // This leverages existing auto-lang parser
        todo!("Implement Auto to Atom conversion")
    }
}
```

### Key Points

- **No JSON parser needed**: Auto format is the superset
- **Leverage existing parser**: Use `auto-lang` parser to read Auto files
- **Direct Atom support**: Can pass Atom structures directly in library API

---

## 3. Template System

### Design

Templates are **Auto scripts themselves**, not a separate template language:

```rust
use auto_lang::ast::Code;

pub struct Template {
    pub name: AutoStr,
    pub code: Code,              // Parsed Auto AST
    pub source: AutoStr,         // Original source code
}

pub struct TemplateEngine {
    fstr_note: char,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            fstr_note: '$',
        }
    }

    pub fn load(&self, path: &PathBuf) -> Result<Template, GenError> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| GenError::TemplateLoadError {
                path: path.clone(),
                reason: e.to_string(),
            })?;

        // Parse as Auto code
        let code = auto_lang::parse(&source)
            .map_err(|e| GenError::TemplateSyntaxError {
                location: SourceLocation {
                    file: path.clone(),
                    line: e.line,
                    column: e.column,
                },
                message: e.message,
            })?;

        Ok(Template {
            name: path.file_name().to_string_lossy().into(),
            code,
            source: source.into(),
        })
    }

    pub fn render(&self, template: &Template, data: &Atom) -> Result<AutoStr, GenError> {
        let mut universe = auto_lang::Universe::new();
        universe.merge_atom(data);

        let mut inter = auto_lang::interp::Interpreter::with_scope(universe)
            .with_fstr_note(self.fstr_note);

        // Execute the Auto script as a template
        let result = inter.eval_code(&template.code)?;
        Ok(result.to_astr())
    }
}
```

### Template Example

```auto
// template/service.txt.at

// This is an Auto script that generates code
// Data is bound from the Atom structure

use auto.io: say

// Access data using f-strings
fn $service_name() {
    say("Service: 0x$service_id")
}

// Generate enum for services
enum Service {
    $for service in $services {
        $service.name = 0x$service.id
    }
}
```

### Key Points

- **Templates are Auto scripts**: Full power of AutoLang available
- **`use` for includes**: Leverage Auto's existing `use` statement
- **No template directives**: All template features are Auto language features
- **No caching needed**: Small batches, parse templates each time

---

## 4. Code Generation Process

### Core Generator Structure

```rust
use std::path::PathBuf;
use auto_val::AutoStr;

pub struct CodeGenerator {
    data_loader: DataLoader,
    template_engine: TemplateEngine,
    guard_processor: GuardProcessor,
    config: GeneratorConfig,
}

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

impl CodeGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            data_loader: DataLoader::new(),
            template_engine: TemplateEngine::new(),
            guard_processor: GuardProcessor::new(),
            config,
        }
    }

    pub fn generate(&mut self, spec: &GenerationSpec) -> Result<GenReport, GenError> {
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
                },
                Err(e) => {
                    errors.push(e);
                },
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
        data: &Atom,
        template_spec: &TemplateSpec
    ) -> Result<PathBuf, GenError> {
        // Load template
        let template = self.template_engine.load(&template_spec.template_path)?;

        // Render template
        let rendered = self.template_engine.render(&template, data)?;

        // Determine output path
        let output_path = self.resolve_output_path(template_spec)?;

        // Process guard blocks if file exists
        let final_output = if output_path.exists() && !self.config.overwrite_guarded {
            self.guard_processor.merge(
                &std::fs::read_to_string(&output_path)?,
                &rendered
            )?
        } else {
            rendered
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
}

pub struct GenerationSpec {
    pub data_source: DataSource,
    pub templates: Vec<TemplateSpec>,
}

pub struct TemplateSpec {
    pub template_path: PathBuf,
    pub output_name: Option<AutoStr>,
    pub rename: bool,
}

pub struct GenReport {
    pub files_generated: Vec<PathBuf>,
    pub errors: Vec<GenError>,
    pub duration: std::time::Duration,
}
```

---

## 5. Guard Block System

### Design

C-style guard blocks only, with improved conflict detection:

```rust
use auto_val::AutoStr;
use regex::Regex;
use std::collections::HashMap;

pub struct GuardProcessor {
    start_pattern: Regex,
    end_pattern: Regex,
}

impl GuardProcessor {
    pub fn new() -> Self {
        Self {
            start_pattern: Regex::new(r#"///\s*----------\s+begin\s+of\s+guard:\s*<(\w+)>"#).unwrap(),
            end_pattern: Regex::new(r#"///\s*----------\s+end\s+of\s+guard:"#).unwrap(),
        }
    }

    pub fn extract_guards(&self, content: &str) -> HashMap<AutoStr, GuardedSection> {
        let mut guards = HashMap::new();
        let mut current_guard: Option<GuardedSection> = None;
        let mut line_number = 0;

        for line in content.lines() {
            line_number += 1;

            if let Some(caps) = self.start_pattern.captures(line) {
                let id = caps[1].to_string().into();
                current_guard = Some(GuardedSection {
                    id: id.clone(),
                    content: AutoStr::new(),
                    start_line: line_number,
                    end_line: 0,
                });
            } else if self.end_pattern.is_match(line) {
                if let Some(mut guard) = current_guard.take() {
                    guard.end_line = line_number;
                    guards.insert(guard.id.clone(), guard);
                }
            } else if let Some(ref mut guard) = current_guard {
                guard.content.push_str(line);
                guard.content.push('\n');
            }
        }

        guards
    }

    pub fn merge(&self, existing: &str, generated: &str) -> Result<String, MergeError> {
        let existing_guards = self.extract_guards(existing);
        let generated_guards = self.extract_guards(generated);

        let mut result = String::new();
        let mut in_guard = false;
        let mut current_guard_id: Option<AutoStr> = None;

        for line in generated.lines() {
            if let Some(caps) = self.start_pattern.captures(line) {
                let guard_id = caps[1].to_string().into();
                in_guard = true;
                current_guard_id = Some(guard_id.clone());
                result.push_str(line);
                result.push('\n');

                // Use existing guard content if available
                if let Some(existing_guard) = existing_guards.get(&guard_id) {
                    result.push_str(&existing_guard.content);
                }
            } else if in_guard && self.end_pattern.is_match(line) {
                in_guard = false;
                current_guard_id = None;
                result.push_str(line);
                result.push('\n');
            } else if !in_guard {
                result.push_str(line);
                result.push('\n');
            }
        }

        Ok(result)
    }

    pub fn detect_conflicts(&self, existing: &str, generated: &str) -> Vec<Conflict> {
        let existing_guards = self.extract_guards(existing);
        let generated_guards = self.extract_guards(generated);

        let mut conflicts = Vec::new();

        for (id, generated_guard) in generated_guards.iter() {
            if let Some(existing_guard) = existing_guards.get(id) {
                if existing_guard.content != generated_guard.content {
                    conflicts.push(Conflict {
                        guard_id: id.clone(),
                        existing_content: existing_guard.content.clone(),
                        generated_content: generated_guard.content.clone(),
                    });
                }
            }
        }

        conflicts
    }
}

pub struct GuardedSection {
    pub id: AutoStr,
    pub content: AutoStr,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct Conflict {
    pub guard_id: AutoStr,
    pub existing_content: AutoStr,
    pub generated_content: AutoStr,
}

#[derive(thiserror::Error, Debug)]
pub enum MergeError {
    #[error("Guard conflict in '{guard_id}'")]
    Conflict { guard_id: AutoStr },

    #[error("Invalid guard syntax at line {line}")]
    InvalidSyntax { line: usize },
}
```

---

## 6. API Design

### Library API

```rust
use auto_gen::{CodeGenerator, GenerationSpec, GeneratorConfig, DataSource, TemplateSpec};

// Simple usage
let config = GeneratorConfig {
    output_dir: PathBuf::from("./output"),
    dry_run: false,
    fstr_note: '$',
    overwrite_guarded: false,
};

let mut generator = CodeGenerator::new(config);

let spec = GenerationSpec {
    data_source: DataSource::AutoFile(PathBuf::from("./data/services.at")),
    templates: vec![
        TemplateSpec {
            template_path: PathBuf::from("./templates/service_header.txt.at"),
            output_name: Some("services.h".into()),
            rename: false,
        },
    ],
};

let report = generator.generate(&spec)?;
println!("Generated {} files in {:?}", report.files_generated.len(), report.duration);

// Builder pattern
let generator = CodeGenerator::builder()
    .output_dir("./output")
    .fstr_note('$')
    .data_source(DataSource::AutoFile(PathBuf::from("./data.at")))
    .add_template("./templates/header.txt.at", "output.h")
    .add_template("./templates/impl.txt.at", "output.c")
    .build()?;

let report = generator.generate()?;
```

### API Structure

```rust
// Core types
pub struct CodeGenerator { /* ... */ }
pub struct GeneratorConfig { /* ... */ }
pub struct GenerationSpec { /* ... */ }
pub struct TemplateSpec { /* ... */ }
pub struct GenReport { /* ... */ }

// Data source
pub enum DataSource {
    AutoFile(PathBuf),
    AutoCode(String),
    Atom(Atom),
}

// Errors
pub type GenResult<T> = Result<T, GenError>;
```

---

## 7. CLI Tool

### Design

**File**: `crates/auto-gen/src/bin/autogen.rs`

```rust
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
        #[arg(short = 'td', long = "template-dir")]
        template_dir: Option<PathBuf>,

        /// Template files (can specify multiple)
        #[arg(short = 't', long = "template-files")]
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
        #[arg(short = 'td', long = "template-dir")]
        template_dir: PathBuf,

        /// Output directory
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
}

fn main() -> Result<(), GenError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { data, template_dir, template_files, output, note, dry_run, overwrite_guarded } => {
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
                template_files.into_iter()
                    .map(|p| TemplateSpec {
                        template_path: p,
                        output_name: None,
                        rename: false,
                    })
                    .collect()
            } else if let Some(dir) = template_dir {
                discover_templates(&dir)?
            } else {
                return Err(GenError::Other("Must specify --template-files or --template-dir".into()));
            };

            let spec = GenerationSpec {
                data_source: DataSource::AutoFile(data),
                templates,
            };

            let report = generator.generate(&spec)?;
            print_report(&report);
        },
        Commands::Validate { templates } => {
            validate_templates(&templates)?;
        },
        Commands::Watch { .. } => {
            return Err(GenError::NotImplemented("Watch mode coming in future phase".into()));
        },
    }

    Ok(())
}
```

### CLI Arguments Summary

- **`-d, --data <FILE>`**: Data source file (Auto format)
- **`-td, --template-dir <DIR>`**: Template directory
- **`-t, --template-files <FILES>`**: Template files (multiple)
- **`-o, --output <DIR>`**: Output directory
- **`-n, --note <CHAR>`**: F-string note character (default: `$`)
- **`--dry-run`**: Don't write files
- **`--overwrite-guarded`**: Overwrite guarded sections
- **`--config <FILE>`**: Config file (Auto format)

---

## 8. Configuration System

### Design

Config files are in **Auto format**:

```auto
// autogen_config.at

// Output directory
let output_dir = "./generated"

// F-string note character
let fstr_note = '$'

// Overwrite guarded sections
let overwrite_guarded = false

// Template directories
let template_dirs = ["./templates", "./shared/templates"]

// Data sources
let data_sources = [
    "./data/services.at",
    "./data/config.at"
]
```

**Config Loader**:

```rust
pub fn load_config(path: &PathBuf) -> Result<GeneratorConfig, GenError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| GenError::ConfigLoadError {
            path: path.clone(),
            reason: e.to_string(),
        })?;

    // Parse Auto config
    let code = auto_lang::parse(&source)
        .map_err(|e| GenError::ConfigSyntaxError {
            path: path.clone(),
            message: e.message,
        })?;

    // Extract config values from parsed Auto code
    let mut config = GeneratorConfig::default();

    for stmt in code.stmts {
        match stmt {
            Stmt::Store(store) if store.name == "output_dir" => {
                if let Expr::Str(path) = store.expr {
                    config.output_dir = PathBuf::from(path.as_str());
                }
            },
            Stmt::Store(store) if store.name == "fstr_note" => {
                if let Expr::Char(c) = store.expr {
                    config.fstr_note = c;
                }
            },
            Stmt::Store(store) if store.name == "overwrite_guarded" => {
                if let Expr::Bool(b) = store.expr {
                    config.overwrite_guarded = b;
                }
            },
            // ... more config options
            _ => {},
        }
    }

    Ok(config)
}
```

### Key Points

- **Auto format config**: Leverage AutoLang for configuration
- **CLI overrides**: CLI args take precedence over config file
- **Optional config**: CLI args work without config file

---

## 9. Error Handling

### IDE-Style Error Messages

```rust
use thiserror::Error;
use auto_val::AutoStr;

#[derive(Error, Debug)]
pub enum GenError {
    #[error("Failed to load data from {path}: {reason}")]
    DataLoadError { path: std::path::PathBuf, reason: String },

    #[error("{location}: Template syntax error: {message}")]
    TemplateSyntaxError {
        location: SourceLocation,
        #[source] sug::AutoSuggestion
    },

    #[error("{location}: {message}")]
    ConfigSyntaxError {
        location: SourceLocation,
        message: String
    },

    #[error("Guard merge conflict in {file} at guard '{guard_id}'")]
    GuardConflict {
        file: std::path::PathBuf,
        guard_id: AutoStr,
        #[suggestion]
        suggestion: String,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: std::path::PathBuf,
    pub line: usize,
    pub column: usize,
    pub source_line: String,
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file.display(), self.line, self.column)
    }
}

// Error display with IDE-style formatting
impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GenError::TemplateSyntaxError { location, message, .. } => {
                writeln!(f, "error: {}", message)?;
                writeln!(f, "  --> {}:{}", location.file.display(), location.line)?;
                if !location.source_line.is_empty() {
                    writeln!(f, "{}", location.source_line)?;
                    writeln!(f, "{}^", " ".repeat(location.column))?;
                }
                Ok(())
            },
            // ... other error types
            _ => write!(f, "{:?}", self),
        }
    }
}
```

### Error Features

- **File location**: File path, line, column
- **Source snippet**: Show the problematic line
- **Visual indicator**: Point to error location with `^`
- **Suggestions**: "Did you mean..." hints
- **Fix suggestions**: Suggest how to fix the error

---

## 10. Implementation Phases

### Phase 1: Core Generator (Priority: HIGH)

**Files:**
- `crates/auto-gen/src/lib.rs` - Core generator API
- `crates/auto-gen/src/data/mod.rs` - Data loading
- `crates/auto-gen/src/template/mod.rs` - Template engine
- `crates/auto-gen/src/guard.rs` - Guard block processing
- `crates/auto-gen/src/error.rs` - Error types

**Tasks:**
1. Implement `CodeGenerator` with flexible API
2. Implement `DataLoader` for Auto/Atom format
3. Implement `TemplateEngine` using AutoLang interpreter
4. Implement `GuardProcessor` with C-style guards
5. Add comprehensive error types with IDE-style formatting
6. Add unit tests for all components

**Success Criteria:** Can generate files from Auto data and Auto templates

---

### Phase 2: CLI Tool (Priority: HIGH)

**Files:**
- `crates/auto-gen/src/bin/autogen.rs` - CLI implementation
- `crates/auto-gen/Cargo.toml` - Add `clap` dependency

**Tasks:**
1. Implement CLI structure with clap
2. Add `generate` command with all arguments
3. Add `validate` command
4. Add config file loading (Auto format)
5. Add help documentation and examples

**Success Criteria:** All operations possible via CLI

---

### Phase 3: Configuration System (Priority: MEDIUM)

**Files:**
- `crates/auto-gen/src/config.rs` - Config module

**Tasks:**
1. Implement Auto-format config parser
2. Add config file validation
3. Implement CLI override logic
4. Add config file examples

**Success Criteria:** Config files work in Auto format

---

### Phase 4: Error Enhancements (Priority: MEDIUM)

**Tasks:**
1. Add source line capture in errors
2. Implement visual error indicators
3. Add "did you mean?" suggestions
4. Add fix suggestions for common errors

**Success Criteria:** Errors provide IDE-style experience

---

### Phase 5: Watch Mode (Priority: LOW - Future)

**Tasks:**
1. Implement file watcher using `notify` crate
2. Add debouncing to avoid excessive regenerations
3. Add change detection for data and template files
4. Implement automatic regeneration on changes

**Success Criteria:** Changes trigger automatic regeneration

---

### Phase 6: Testing & Documentation (Priority: MEDIUM)

**Tasks:**
1. Create integration tests
2. Add example projects
3. Write API documentation
4. Create user guide with examples

**Test Structure:**
```
crates/auto-gen/tests/
├── integration/
│   ├── test_generation.rs
│   ├── test_guards.rs
│   ├── test_cli.rs
│   └── test_config.rs
├── fixtures/
│   ├── data/
│   │   └── services.at
│   └── templates/
│       └── service.txt.at
└── examples/
    ├── simple_codegen/
    └── diagnostic_gen/
```

---

## 11. Critical Files

### Files to Create

1. **`crates/auto-gen/src/lib.rs`**
   - Main library API
   - Reimplementation of `CodeGenerator`, `TemplateSpec`, etc.
   - Clean, flexible API design

2. **`crates/auto-gen/src/data/mod.rs`**
   - `DataSource` enum
   - `DataLoader` implementation
   - Auto file to Atom conversion

3. **`crates/auto-gen/src/template/mod.rs`**
   - `TemplateEngine` using AutoLang interpreter
   - Template loading and rendering
   - No caching (small batches)

4. **`crates/auto-gen/src/guard.rs`**
   - `GuardProcessor` with C-style guards
   - Guard extraction and merging
   - Conflict detection

5. **`crates/auto-gen/src/error.rs`**
   - Comprehensive error types
   - IDE-style error formatting
   - Source location tracking

6. **`crates/auto-gen/src/config.rs`**
   - Auto-format config loading
   - Config to `GeneratorConfig` conversion

7. **`crates/auto-gen/src/bin/autogen.rs`**
   - CLI implementation
   - Command definitions
   - Config file support

### Files to Modify

1. **`crates/auto-gen/Cargo.toml`**
   - Update dependencies
   - Add: `clap`, `thiserror`, `regex`, `notify` (for watch mode)

2. **`crates/auto-gen/src/lib.rs`**
   - Replace old implementation
   - New public API
   - Remove deprecated `AutoGen`, `Mold`, `OneGen`

---

## Success Criteria

### Functional Requirements
- ✅ Load data from Auto format files
- ✅ Generate code from Auto templates
- ✅ Preserve hand-written code in guard blocks
- ✅ CLI tool with full functionality
- ✅ Config file support (Auto format)
- ✅ Library API for embedding

### Non-Functional Requirements
- ✅ Moderate performance (no hard targets)
- ✅ IDE-style error messages
- ✅ Clear documentation
- ✅ Clean, flexible API design

### Quality Requirements
- ✅ Comprehensive error messages with locations
- ✅ Integration tests
- ✅ Example projects
- ✅ User guide with examples

---

*This design document reflects the user's requirements and preferences gathered through interactive questioning.*
