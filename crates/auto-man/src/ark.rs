//! ArkTS (HarmonyOS) project generation utilities
//!
//! This module provides the complete ArkTS/HarmonyOS project workflow:
//! 1. Generate ArkTS code from .at files
//! 2. Generate full project structure (optional)
//! 3. Copy to output directory

use std::fs;
use std::path::Path;

use colored::Colorize;
use auto_lang::ui_gen::ark::{ArkGenerator, ArkProjectGenerator};
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

/// Check if ark is in the backend list
fn has_ark_backend(content: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("backend:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                // Check if it's an array format
                if value.starts_with('[') {
                    // Extract all backends from array
                    let backends: Vec<&str> = value
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .split(',')
                        .filter_map(|s| {
                            let s = s.trim().trim_matches('"').trim_matches('\'');
                            if !s.is_empty() { Some(s) } else { None }
                        })
                        .collect();
                    // Check for both "ark" and "arkts" variants
                    return backends.iter().any(|&b| b == "ark" || b == "arkts");
                } else {
                    let value = value.trim_matches('"').trim_matches('\'');
                    let value = value.trim_end_matches(',');
                    return value == "ark" || value == "arkts";
                }
            }
        }
    }
    false
}

/// ArkTS project generation context
pub struct ArkProject {
    /// Project root directory (where pac.at is)
    pub root_dir: std::path::PathBuf,
    /// Output directory
    pub output_dir: std::path::PathBuf,
    /// Project name
    pub name: String,
    /// Front source directory
    pub front_dir: std::path::PathBuf,
    /// Generated ArkTS files (relative_path, content)
    pub arkts_files: Vec<(String, String)>,
    /// Widget names discovered from .at files
    pub widget_names: Vec<String>,
}

impl ArkProject {
    /// Create a new Ark project context from a workspace directory
    pub fn from_workspace(root_dir: &Path) -> AutoResult<Self> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }

        let pac_content = fs::read_to_string(&pac_path)
            .map_err(|e| format!("Failed to read pac.at: {}", e))?;

        // Check if ark is in the backend list
        if !has_ark_backend(&pac_content) {
            return Err("Backend 'ark' not found in pac.at. Add 'ark' to backend list.".into());
        }

        // Get project name
        let name = parse_pac_name(&pac_content)
            .unwrap_or_else(|| "MyApp".to_string());

        // Determine front directory
        let front_dir = if root_dir.join("source").join("front").exists() {
            root_dir.join("source").join("front")
        } else if root_dir.join("front").exists() {
            root_dir.join("front")
        } else {
            root_dir.join("source").join("front")
        };

        // Output directory
        let output_dir = root_dir.join("ark");

        // Compile .at files to ArkTS
        let mut arkts_files: Vec<(String, String)> = Vec::new();
        let mut widget_names: Vec<String> = Vec::new();

        // Process app.at if exists
        let app_at = front_dir.join("app.at");
        if app_at.exists() {
            match Self::compile_at_file(&app_at, &name) {
                Ok((files, names)) => {
                    arkts_files.extend(files);
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
                    match Self::compile_at_file(&path, &name) {
                        Ok((files, names)) => {
                            arkts_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                        }
                    }
                }
            }
        }

        // Process pages/ directory
        let pages_dir = front_dir.join("pages");
        if pages_dir.exists() {
            for entry in fs::read_dir(&pages_dir)
                .map_err(|e| format!("Failed to read pages directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    match Self::compile_at_file(&path, &name) {
                        Ok((files, names)) => {
                            arkts_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), path.display(), e);
                        }
                    }
                }
            }
        }

        // Process components/ directory
        let components_dir = front_dir.join("components");
        if components_dir.exists() {
            for entry in fs::read_dir(&components_dir)
                .map_err(|e| format!("Failed to read components directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();

                if path.extension().map(|e| e == "at").unwrap_or(false) {
                    match Self::compile_at_file(&path, &name) {
                        Ok((files, names)) => {
                            arkts_files.extend(files);
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
            arkts_files,
            widget_names,
        })
    }

    /// Compile a single .at file to ArkTS code
    /// Returns (arkts_files, widget_names)
    fn compile_at_file(at_path: &Path, _project_name: &str) -> Result<(Vec<(String, String)>, Vec<String>), String> {
        let code = fs::read_to_string(at_path)
            .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

        // Parse with UI scenario
        use auto_lang::session::CompilerSession;
        let session = CompilerSession::ui().with_backend("ark");
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
            return Ok((Vec::new(), Vec::new()));
        }

        // Generate ArkTS code for each widget
        let mut generator = ArkGenerator::new();
        let mut files = Vec::new();
        let mut names = Vec::new();

        for widget in &widgets {
            let arkts_code = generator.generate(widget)
                .map_err(|e| e.to_string())?;

            // Collect widget name
            names.push(widget.name.clone());

            // Generate file path: entry/src/main/ets/pages/{WidgetName}.ets
            let widget_name = &widget.name;
            let file_name = format!("{}.ets", widget_name);
            let relative_path = format!("entry/src/main/ets/pages/{}", file_name);
            files.push((relative_path, arkts_code));
        }

        Ok((files, names))
    }

    /// Check if the project structure already exists
    pub fn exists(&self) -> bool {
        self.output_dir.exists() && self.output_dir.join("build-profile.json5").exists()
    }

    /// Generate the ArkTS project structure
    pub fn generate(&self) -> AutoResult<()> {
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!("{}", "  AURA Workspace → ArkTS/HarmonyOS".bright_yellow().bold());
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!();

        println!("{} {}", "Output:".bright_cyan(), self.output_dir.display());
        println!("{} {}", "Name:".bright_cyan(), self.name);
        println!("{} {}", "Widgets:".bright_cyan(), self.widget_names.join(", "));
        println!();

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Generate full project structure using ArkProjectGenerator
        let project_gen = ArkProjectGenerator::new(&self.name);
        let project_files = project_gen.generate();

        // Write all project files
        for (file_path, content) in project_files {
            let full_path = self.output_dir.join(&file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
            // Handle binary files (PNG) which are base64 encoded
            if file_path.ends_with(".png") {
                use base64::{Engine as _, engine::general_purpose};
                let bytes = general_purpose::STANDARD
                    .decode(&content)
                    .map_err(|e| format!("Failed to decode base64 for {}: {}", file_path, e))?;
                fs::write(&full_path, bytes)
                    .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
            } else {
                fs::write(&full_path, content)
                    .map_err(|e| format!("Failed to write {}: {}", full_path.display(), e))?;
            }
        }

        println!("{}", "✓ Created project structure".bright_green());

        // Write generated ArkTS widget files
        for (relative_path, content) in &self.arkts_files {
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
        println!("{}", "  ArkTS/HarmonyOS project generated!".bright_green().bold());
        println!("═════════════════════════════════");
        println!();
        println!("{} {}", "Next steps:".bright_cyan(), "");
        println!("  cd {}", self.output_dir.display());
        println!("  # Open in DevEco Studio or run hvigorw assembleHap");

        Ok(())
    }

    /// Generate only ArkTS files (no full project structure)
    pub fn generate_arkts_only(&self, output_dir: &Path) -> AutoResult<()> {
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!("{}", "  AURA → ArkTS Code Generation".bright_yellow().bold());
        println!("{}", "─────────────────────────────────".bright_yellow().bold());
        println!();

        println!("{} {}", "Output:".bright_cyan(), output_dir.display());
        println!("{} {}", "Files:".bright_cyan(), self.arkts_files.len());
        println!();

        // Create output directory
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Write generated ArkTS files
        for (relative_path, content) in &self.arkts_files {
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
        println!("{} ArkTS files generated.", "✓".bright_green());

        Ok(())
    }
}

/// Generate ArkTS code from .at files (auto gen command for ark backend)
pub fn generate_ark_project(root_dir: &Path, output_dir: Option<&Path>, full_project: bool) -> AutoResult<()> {
    println!("{}", "Generating ArkTS/HarmonyOS project".bright_cyan());

    // Load project context
    let project = ArkProject::from_workspace(root_dir)?;

    // Determine output directory
    let output = output_dir.unwrap_or(&project.output_dir);

    if full_project || !project.exists() {
        // Generate full project structure
        project.generate()?;
    } else {
        // Generate only ArkTS files
        project.generate_arkts_only(output)?;
    }

    Ok(())
}

/// Build ArkTS project (auto build command for ark backend)
pub fn build_ark_project(root_dir: &Path) -> AutoResult<()> {
    println!("{}", "Building ArkTS/HarmonyOS project (backend: ark)".bright_cyan());

    // Step 1: Generate code
    println!();
    println!("{}", "▶ Step 1: Generating ArkTS code...".bright_cyan());
    generate_ark_project(root_dir, None, false)?;

    let ark_dir = root_dir.join("ark");

    // Step 2: Check for hvigorw
    println!();
    println!("{}", "▶ Step 2: Checking Hvigor wrapper...".bright_cyan());

    let hvigorw = if cfg!(windows) {
        ark_dir.join("hvigorw.bat")
    } else {
        ark_dir.join("hvigorw")
    };

    if !hvigorw.exists() {
        println!("  ⚠ Hvigor wrapper not found.");
        println!("  Please open the project in DevEco Studio to build.");
        println!("  Project location: {}", ark_dir.display());
        return Ok(());
    }

    println!("  ✓ Hvigor wrapper found");

    // Step 3: Build the project
    println!();
    println!("{}", "▶ Step 3: Building HAP...".bright_cyan());

    let build_result = std::process::Command::new(&hvigorw)
        .args(&["assembleHap"])
        .current_dir(&ark_dir)
        .status()
        .map_err(|e| format!("Failed to build: {}", e))?;

    if build_result.success() {
        println!();
        println!("{}", "✓ Build successful!".bright_green());
        println!();
        println!("HAP location:");
        println!("  {}", ark_dir.join("entry/build/default/outputs/default/entry-default-signed.hap").display().to_string().bright_cyan());
    } else {
        println!();
        println!("  ⚠ Build failed. Try running manually:");
        println!("    cd {} && ./hvigorw assembleHap", ark_dir.display());
    }

    Ok(())
}

/// Run ArkTS project (auto run command for ark backend)
pub fn run_ark_project(root_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    println!("{}", "Running ArkTS/HarmonyOS project (backend: ark)".bright_cyan());

    // Step 1: Generate code
    println!();
    println!("{}", "▶ Step 1: Generating ArkTS code...".bright_cyan());
    generate_ark_project(root_dir, None, false)?;

    let ark_dir = root_dir.join("ark");

    println!();
    println!("{}", "═══════════════════════════════════".bright_green());
    println!("{}", "  ArkTS/HarmonyOS project ready!".bright_green());
    println!("{}", "═══════════════════════════════════".bright_green());
    println!();
    println!("To run the app:");
    println!("  1. Open DevEco Studio");
    println!("  2. Open the project at: {}", ark_dir.display().to_string().bright_cyan());
    println!("  3. Connect a HarmonyOS device or start an emulator");
    println!("  4. Click Run or use: ./hvigorw installHap");

    Ok(())
}
