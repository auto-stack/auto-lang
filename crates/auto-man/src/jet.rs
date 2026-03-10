//! Jetpack Compose project generation utilities
//!
//! This module provides the complete Jetpack Compose Android project workflow:
//! 1. Generate Kotlin code from .at files
//! 2. Generate full project structure (optional)
//! 3. Copy to output directory

use std::fs;
use std::path::Path;

use colored::Colorize;
use auto_lang::ui_gen::jet::{JetGenerator, JetProjectConfig, ProjectGenerator};
use auto_lang::ui_gen::BackendGenerator;
use auto_lang::Parser;

use crate::AutoResult;

/// Parse project name from pac.at content
fn parse_pac_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                let value = value.trim_end_matches(',');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parse backend from pac.at content
fn parse_pac_backend(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("backend:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                let value = value.trim_end_matches(',');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Jetpack Compose project generation context
pub struct JetProject {
    /// Project root directory (where pac.at is)
    pub root_dir: std::path::PathBuf,
    /// Output directory
    pub output_dir: std::path::PathBuf,
    /// Project name
    pub name: String,
    /// Front source directory
    pub front_dir: std::path::PathBuf,
    /// Generated Kotlin files (relative_path, content)
    pub kotlin_files: Vec<(String, String)>,
    /// Widget names discovered from .at files
    pub widget_names: Vec<String>,
}

impl JetProject {
    /// Create a new Jet project context from a workspace directory
    pub fn from_workspace(root_dir: &Path) -> AutoResult<Self> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }

        let pac_content = fs::read_to_string(&pac_path)
            .map_err(|e| format!("Failed to read pac.at: {}", e))?;

        // Verify backend is jet
        let backend = parse_pac_backend(&pac_content)
            .unwrap_or_else(|| "jet".to_string());
        if backend != "jet" {
            return Err(format!("Expected backend 'jet', found '{}'", backend).into());
        }

        // Get project name
        let name = parse_pac_name(&pac_content)
            .unwrap_or_else(|| "MyApp".to_string());

        // Default front directory
        let front_dir = root_dir.join("source").join("front");

        // Output directory
        let output_dir = root_dir.join("dist");

        // Compile .at files to Kotlin
        let mut kotlin_files: Vec<(String, String)> = Vec::new();
        let mut widget_names: Vec<String> = Vec::new();

        // Process app.at if exists
        let app_at = front_dir.join("app.at");
        if app_at.exists() {
            match Self::compile_at_file(&app_at, &name) {
                Ok((files, names)) => {
                    kotlin_files.extend(files);
                    widget_names.extend(names);
                }
                Err(e) => {
                    println!("{} {}", "Warning: Failed to compile app.at:".bright_yellow(), e);
                }
            }
        }

        // Process widgets/ directory
        let widgets_dir = front_dir.join("widgets");
        if widgets_dir.exists() {
            for entry in fs::read_dir(&widgets_dir)
                .map_err(|e| format!("Failed to read widgets directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    let file_stem = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("widget");

                    match Self::compile_at_file(&path, file_stem) {
                        Ok((files, names)) => {
                            kotlin_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(Self {
            root_dir: root_dir.to_path_buf(),
            output_dir,
            name,
            front_dir,
            kotlin_files,
            widget_names,
        })
    }

    /// Compile a single .at file to Kotlin code
    /// Returns (kotlin_files, widget_names)
    fn compile_at_file(at_path: &Path, default_name: &str) -> Result<(Vec<(String, String)>, Vec<String>), String> {
        let code = fs::read_to_string(at_path)
            .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

        // Parse with UI scenario
        use auto_lang::session::CompilerSession;
        let session = CompilerSession::ui().with_backend("jet");
        let mut parser = Parser::from(code.as_str());
        parser = parser.with_session(session);
        let ast = parser.parse().map_err(|e| {
            format!("Parse error: {:?}", e)
        })?;

        // Extract AURA widgets from AST
        let mut widgets = Vec::new();
        for stmt in &ast.stmts {
            if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
                let aura_widget = auto_lang::aura::extract_widget_from_decl(widget_decl)
                    .map_err(|e| e.to_string())?;
                widgets.push(aura_widget);
            }
        }

        if widgets.is_empty() {
            // No widgets found, skip this file
            return Ok((Vec::new(), Vec::new()));
        }

        // Generate Kotlin code for each widget
        let mut generator = JetGenerator::new();
        let mut files = Vec::new();
        let mut names = Vec::new();

        for widget in &widgets {
            let kotlin_code = generator.generate(widget)
                .map_err(|e| e.to_string())?;

            // Collect widget name
            names.push(widget.name.clone());

            // Generate file path: ui/widgets/{WidgetName}.kt
            let widget_name = &widget.name;
            let file_name = format!("{}.kt", widget_name);
            let relative_path = format!("app/src/main/java/com/example/{}/ui/widgets/{}", default_name.to_lowercase(), file_name);
            files.push((relative_path, kotlin_code));
        }

        Ok((files, names))
    }

    /// Check if the project structure already exists
    pub fn exists(&self) -> bool {
        self.output_dir.exists() && self.output_dir.join("settings.gradle.kts").exists()
    }

    /// Generate the Jetpack Compose project structure
    pub fn generate(&self) -> AutoResult<()> {
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!("{}", "  AURA Workspace → Jetpack Compose".bright_yellow().bold());
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!();

        println!("{} {}", "Output:".bright_cyan(), self.output_dir.display());
        println!("{} {}", "Name:".bright_cyan(), self.name);
        println!("{} {}", "Widgets:".bright_cyan(), self.widget_names.join(", "));
        println!();

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate full project structure using ProjectGenerator
        // Configure with widget names so MainActivity.kt can import them
        let mut config = JetProjectConfig::new(&self.name);
        for widget_name in &self.widget_names {
            config = config.with_widget(widget_name);
        }
        let mut generator = ProjectGenerator::with_config(config);
        let project_files = generator.generate();

        // Write all project files
        for (file_path, content) in project_files {
            let full_path = self.output_dir.join(&file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            fs::write(&full_path, content)
                .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
        }

        println!("{}", "✓ Created project structure".bright_green());

        // Write generated Kotlin widget files
        for (relative_path, content) in &self.kotlin_files {
            let full_path = self.output_dir.join(relative_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            fs::write(&full_path, content)
                .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
            println!("{} {}", "  Generated".bright_green(), relative_path);
        }

        println!();
        println!("═════════════════════════════════");
        println!("{}", "  Jetpack Compose project generated!".bright_green().bold());
        println!("═════════════════════════════════");
        println!();
        println!("{} {}", "Next steps:".bright_cyan(), "");
        println!("  cd {}", self.output_dir.display());
        println!("  ./gradlew assembleDebug");

        Ok(())
    }

    /// Generate only Kotlin files (no full project structure)
    pub fn generate_kotlin_only(&self, output_dir: &Path) -> AutoResult<()> {
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!("{}", "  AURA → Kotlin Code Generation".bright_yellow().bold());
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!();

        println!("{} {}", "Output:".bright_cyan(), output_dir.display());
        println!("{} {}", "Files:".bright_cyan(), self.kotlin_files.len());
        println!();

        // Create output directory
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Write generated Kotlin files
        for (relative_path, content) in &self.kotlin_files {
            let full_path = output_dir.join(relative_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            fs::write(&full_path, content)
                .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
            println!("{} {}", "  Generated".bright_green(), relative_path);
        }

        println!();
        println!("{} Kotlin files generated.", "✓".bright_green());

        Ok(())
    }
}

/// Generate Kotlin code from .at files (auto gen command for jet backend)
///
/// Steps:
/// 1. Parse pac.at to get project info
/// 2. Compile .at files to Kotlin
/// 3. Generate full project structure (if project flag)
/// 4. Copy to output directory
pub fn generate_jet_project(root_dir: &Path, output_dir: Option<&Path>, full_project: bool) -> AutoResult<()> {
    println!("{}", "Generating Jetpack Compose project".bright_cyan());

    // Load project context
    let project = JetProject::from_workspace(root_dir)?;

    // Determine output directory
    let output = output_dir.unwrap_or(&project.output_dir);

    if full_project || !project.exists() {
        // Generate full project structure
        project.generate()?;
    } else {
        // Generate only Kotlin files
        project.generate_kotlin_only(output)?;
    }

    Ok(())
}
