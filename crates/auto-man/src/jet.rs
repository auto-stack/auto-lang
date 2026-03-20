//! Jetpack Compose project generation utilities
//!
//! This module provides the complete Jetpack Compose Android project workflow:
//! 1. Generate Kotlin code from .at files
//! 2. Generate full project structure (optional)
//! 3. Copy to output directory

use std::fs;
use std::path::{Path, PathBuf};

use colored::Colorize;
use auto_lang::database::{UIArtifact, UIBackend, UICache};
use auto_lang::ui_gen::jet::{JetGenerator, JetProjectConfig, ProjectGenerator};
use auto_lang::ui_gen::BackendGenerator;
use auto_lang::Parser;

use crate::util::hash_string;
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

/// Parse backend from pac.at content (supports array format)
#[allow(dead_code)]
fn parse_pac_backend(content: &str) -> Option<String> {
    // First, try to parse as array: backend: ["vue", "jet"]
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
                    // Return first backend if jet is in the list
                    if backends.iter().any(|&b| b == "jet") {
                        return Some("jet".to_string());
                    }
                    return backends.first().map(|s| s.to_string());
                } else {
                    // Single backend
                    let value = value.trim_matches('"').trim_matches('\'');
                    let value = value.trim_end_matches(',');
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Check if jet is in the backend list
fn has_jet_backend(content: &str) -> bool {
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
                    return backends.iter().any(|&b| b == "jet");
                } else {
                    let value = value.trim_matches('"').trim_matches('\'');
                    let value = value.trim_end_matches(',');
                    return value == "jet";
                }
            }
        }
    }
    false
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

        // Check if jet is in the backend list (supports multi-backend configuration)
        if !has_jet_backend(&pac_content) {
            return Err("Backend 'jet' not found in pac.at. Add 'jet' to backend list.".into());
        }

        // Get project name
        let name = parse_pac_name(&pac_content)
            .unwrap_or_else(|| "MyApp".to_string());

        // Determine front directory - check multiple locations
        let front_dir = if root_dir.join("source").join("front").exists() {
            root_dir.join("source").join("front")
        } else if root_dir.join("front").exists() {
            root_dir.join("front")
        } else {
            // Default to source/front (will be created if needed)
            root_dir.join("source").join("front")
        };

        // Output directory (Plan 129: jet/ instead of dist/)
        let output_dir = root_dir.join("jet");

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
                    match Self::compile_at_file(&path, &name) {
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

        // Process pages/ directory (Plan 136)
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

        // Process components/ directory (Plan 136)
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

    /// Generate Kotlin files with incremental support
    /// Returns (project, changed_files)
    pub fn from_workspace_incremental(root_dir: &Path) -> AutoResult<(Self, Vec<String>)> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }

        let pac_content = fs::read_to_string(&pac_path)
            .map_err(|e| format!("Failed to read pac.at: {}", e))?;

        if !has_jet_backend(&pac_content) {
            return Err("Backend 'jet' not found in pac.at".into());
        }

        let name = parse_pac_name(&pac_content).unwrap_or_else(|| "MyApp".to_string());
        let front_dir = root_dir.join("source").join("front");
        let output_dir = root_dir.join("jet");

        // Load cache
        let mut cache = UICache::load(root_dir);
        let mut changed_files = Vec::new();

        // Process app.at
        let mut kotlin_files: Vec<(String, String)> = Vec::new();
        let mut widget_names: Vec<String> = Vec::new();

        let app_at = front_dir.join("app.at");
        if app_at.exists() {
            let content = fs::read_to_string(&app_at)
                .map_err(|e| format!("Failed to read app.at: {}", e))?;
            let hash = hash_string(&content);

            if cache.is_dirty(&app_at, hash) {
                println!("  {} (changed)", "app.at".bright_yellow());
                match Self::compile_at_file(&app_at, &name) {
                    Ok((files, names)) => {
                        // Create artifacts for tracking
                        let artifacts: Vec<UIArtifact> = files.iter().zip(names.iter()).map(|((path, content), widget_name)| {
                            UIArtifact {
                                source_path: app_at.clone(),
                                widget_name: widget_name.clone(),
                                output_path: PathBuf::from(path),
                                source_hash: hash,
                                content_hash: hash_string(content),
                                backend: UIBackend::Jet,
                            }
                        }).collect();

                        cache.update(app_at.clone(), hash, artifacts);
                        kotlin_files.extend(files);
                        widget_names.extend(names);
                        changed_files.push("app.at".to_string());
                    }
                    Err(e) => {
                        println!("{} {}", "Warning: Failed to compile app.at:".bright_yellow(), e);
                    }
                }
            } else {
                println!("  {} (cached)", "app.at".bright_green());
            }
        }

        // Process widgets/ directory
        let widgets_dir = front_dir.join("widgets");
        if widgets_dir.exists() {
            if let Ok(entries) = fs::read_dir(&widgets_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                        if let Ok(content) = fs::read_to_string(&path) {
                            let hash = hash_string(&content);

                            if cache.is_dirty(&path, hash) {
                                println!("  {} (changed)", file_name.bright_yellow());

                                match Self::compile_at_file(&path, &name) {
                                    Ok((files, names)) => {
                                        let artifacts: Vec<UIArtifact> = files.iter().zip(names.iter()).map(|((p, c), widget_name)| {
                                            UIArtifact {
                                                source_path: path.clone(),
                                                widget_name: widget_name.clone(),
                                                output_path: PathBuf::from(p),
                                                source_hash: hash,
                                                content_hash: hash_string(c),
                                                backend: UIBackend::Jet,
                                            }
                                        }).collect();

                                        cache.update(path.clone(), hash, artifacts);
                                        kotlin_files.extend(files);
                                        widget_names.extend(names);
                                        changed_files.push(file_name);
                                    }
                                    Err(e) => {
                                        println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), file_name, e);
                                    }
                                }
                            } else {
                                println!("  {} (cached)", file_name.bright_green());
                            }
                        }
                    }
                }
            }
        }

        // Process pages/ directory (Plan 136)
        let pages_dir = front_dir.join("pages");
        if pages_dir.exists() {
            if let Ok(entries) = fs::read_dir(&pages_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                        if let Ok(content) = fs::read_to_string(&path) {
                            let hash = hash_string(&content);

                            if cache.is_dirty(&path, hash) {
                                println!("  {} (changed)", file_name.bright_yellow());

                                match Self::compile_at_file(&path, &name) {
                                    Ok((files, names)) => {
                                        let artifacts: Vec<UIArtifact> = files.iter().zip(names.iter()).map(|((p, c), widget_name)| {
                                            UIArtifact {
                                                source_path: path.clone(),
                                                widget_name: widget_name.clone(),
                                                output_path: PathBuf::from(p),
                                                source_hash: hash,
                                                content_hash: hash_string(c),
                                                backend: UIBackend::Jet,
                                            }
                                        }).collect();

                                        cache.update(path.clone(), hash, artifacts);
                                        kotlin_files.extend(files);
                                        widget_names.extend(names);
                                        changed_files.push(file_name);
                                    }
                                    Err(e) => {
                                        println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), file_name, e);
                                    }
                                }
                            } else {
                                println!("  {} (cached)", file_name.bright_green());
                            }
                        }
                    }
                }
            }
        }

        // Process components/ directory (Plan 136)
        let components_dir = front_dir.join("components");
        if components_dir.exists() {
            if let Ok(entries) = fs::read_dir(&components_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                        if let Ok(content) = fs::read_to_string(&path) {
                            let hash = hash_string(&content);

                            if cache.is_dirty(&path, hash) {
                                println!("  {} (changed)", file_name.bright_yellow());

                                match Self::compile_at_file(&path, &name) {
                                    Ok((files, names)) => {
                                        let artifacts: Vec<UIArtifact> = files.iter().zip(names.iter()).map(|((p, c), widget_name)| {
                                            UIArtifact {
                                                source_path: path.clone(),
                                                widget_name: widget_name.clone(),
                                                output_path: PathBuf::from(p),
                                                source_hash: hash,
                                                content_hash: hash_string(c),
                                                backend: UIBackend::Jet,
                                            }
                                        }).collect();

                                        cache.update(path.clone(), hash, artifacts);
                                        kotlin_files.extend(files);
                                        widget_names.extend(names);
                                        changed_files.push(file_name);
                                    }
                                    Err(e) => {
                                        println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), file_name, e);
                                    }
                                }
                            } else {
                                println!("  {} (cached)", file_name.bright_green());
                            }
                        }
                    }
                }
            }
        }

        // Save cache
        cache.save(root_dir).ok();

        Ok((Self {
            root_dir: root_dir.to_path_buf(),
            output_dir,
            name,
            front_dir,
            kotlin_files,
            widget_names,
        }, changed_files))
    }

    /// Compile a single .at file to Kotlin code
    /// Returns (kotlin_files, widget_names)
    ///
    /// # Arguments
    /// * `at_path` - Path to the .at file
    /// * `project_name` - Project name (used for package name)
    fn compile_at_file(at_path: &Path, project_name: &str) -> Result<(Vec<(String, String)>, Vec<String>), String> {
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
        // Use project name for package - all widgets go to the same package
        let safe_project_name = project_name.to_lowercase().replace('-', "_");
        let package_name = format!("com.example.{}.ui.widgets", safe_project_name);
        let mut generator = JetGenerator::new().with_package(&package_name);
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
            let relative_path = format!("app/src/main/java/com/example/{}/ui/widgets/{}", safe_project_name, file_name);
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

        // Check if any kotlin files use navigation (to add dependency)
        let uses_navigation = self.kotlin_files.iter().any(|(_, content)| {
            content.contains("NavHostController") ||
            content.contains("NavHost(") ||
            content.contains("rememberNavController")
        });

        // Generate full project structure using ProjectGenerator
        // Configure with widget names so MainActivity.kt can import them
        let mut config = JetProjectConfig::new(&self.name);
        for widget_name in &self.widget_names {
            config = config.with_widget(widget_name);
        }

        // Add navigation dependency if needed
        if uses_navigation {
            config = config.with_dependency("navigation", "2.7.6");
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

/// Run Jetpack Compose project (auto run command for jet backend)
///
/// Steps:
/// 1. Generate code (incremental - only changed files)
/// 2. Generate project structure if not exists
/// 3. Build the project
/// 4. Install and run on connected device/emulator
pub fn run_jet_project(root_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    println!("{}", "Running Jetpack Compose project (backend: jet)".bright_cyan());

    // Step 1: Generate code (always, incremental)
    println!();
    println!("{}", "▶ Step 1: Generating Kotlin code...".bright_cyan());
    generate_jet_project(root_dir, None, false)?;

    let jet_dir = root_dir.join("jet");

    // Step 2: Check for gradlew
    println!();
    println!("{}", "▶ Step 2: Checking Gradle wrapper...".bright_cyan());

    let gradlew = if cfg!(windows) {
        jet_dir.join("gradlew.bat")
    } else {
        jet_dir.join("gradlew")
    };

    if !gradlew.exists() {
        println!("  ⚠ Gradle wrapper not found, generating...");
        // Generate gradle wrapper if needed
        std::process::Command::new("gradle")
            .args(&["wrapper"])
            .current_dir(&jet_dir)
            .output()
            .map_err(|e| format!("Failed to generate gradle wrapper: {}. Please install Gradle or Android Studio.", e))?;
    } else {
        println!("  ✓ Gradle wrapper found");
    }

    // Step 3: Build the project
    println!();
    println!("{}", "▶ Step 3: Building Android project...".bright_cyan());

    let build_result = std::process::Command::new(&gradlew)
        .args(&["assembleDebug"])
        .current_dir(&jet_dir)
        .status()
        .map_err(|e| format!("Failed to run gradlew assembleDebug: {}", e))?;

    if !build_result.success() {
        return Err("Build failed. Check the error messages above.".into());
    }
    println!("  ✓ Build successful");

    // Step 4: Install and run on device/emulator
    println!();
    println!("{}", "▶ Step 4: Installing on device/emulator...".bright_cyan());

    // Check for connected devices
    let adb_devices = std::process::Command::new("adb")
        .args(&["devices"])
        .output();

    let has_device = match adb_devices {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Check if there's at least one device (excluding header line)
            stdout.lines().any(|line| line.contains("\tdevice"))
        }
        Err(_) => false
    };

    if has_device {
        // Install the APK
        let install_result = std::process::Command::new(&gradlew)
            .args(&["installDebug"])
            .current_dir(&jet_dir)
            .status()
            .map_err(|e| format!("Failed to install: {}", e))?;

        if install_result.success() {
            // Read project name from pac.at
            let pac_path = root_dir.join("pac.at");
            let project_name = if pac_path.exists() {
                if let Ok(content) = fs::read_to_string(&pac_path) {
                    parse_pac_name(&content).unwrap_or_else(|| "MyApp".to_string())
                } else {
                    "MyApp".to_string()
                }
            } else {
                "MyApp".to_string()
            };

            println!("  ✓ App installed successfully!");
            println!();
            println!("{}", "App is now running on your device/emulator.".bright_green());
            println!("Package: com.example.{}", project_name.to_lowercase().replace("-", "_"));
        } else {
            println!("  ⚠ Install failed. Try running manually:");
            println!("    cd {} && ./gradlew installDebug", jet_dir.display());
        }
    } else {
        println!("  ⚠ No Android device or emulator found.");
        println!();
        println!("To run the app:");
        println!("  1. Connect an Android device (with USB debugging enabled), or");
        println!("  2. Start an Android emulator, or");
        println!("  3. Open the project in Android Studio:");
        println!();
        println!("     {}", jet_dir.display().to_string().bright_cyan());
        println!();
        println!("Then run: ./gradlew installDebug");
    }

    println!();
    println!("{}", "═══════════════════════════════════".bright_green());
    println!("{}", "  Jetpack Compose project ready!".bright_green());
    println!("{}", "═══════════════════════════════════".bright_green());

    Ok(())
}

/// Build Jetpack Compose project (auto build command for jet backend)
///
/// Steps:
/// 1. Generate code (incremental - only changed files)
/// 2. Generate project structure if not exists
/// 3. Run gradlew assembleDebug
pub fn build_jet_project(root_dir: &Path) -> AutoResult<()> {
    println!("{}", "Building Jetpack Compose project (backend: jet)".bright_cyan());

    // Step 1: Generate code (always, incremental)
    println!();
    println!("{}", "▶ Step 1: Generating Kotlin code...".bright_cyan());
    generate_jet_project(root_dir, None, false)?;

    let jet_dir = root_dir.join("jet");

    // Step 2: Check for gradlew
    println!();
    println!("{}", "▶ Step 2: Checking Gradle wrapper...".bright_cyan());

    let gradlew = if cfg!(windows) {
        jet_dir.join("gradlew.bat")
    } else {
        jet_dir.join("gradlew")
    };

    if !gradlew.exists() {
        println!("  ⚠ Gradle wrapper not found, generating...");

        // Try to find gradle in common locations
        let gradle_cmd = find_gradle();

        match gradle_cmd {
            Some(gradle_path) => {
                println!("  Using Gradle from: {}", gradle_path.display());
                let result = std::process::Command::new(&gradle_path)
                    .args(&["wrapper"])
                    .current_dir(&jet_dir)
                    .status()
                    .map_err(|e| format!("Failed to generate gradle wrapper: {}", e))?;

                if !result.success() {
                    return Err("Failed to generate gradle wrapper. Please run 'gradle wrapper' manually in the jet directory.".into());
                }
                println!("  ✓ Gradle wrapper generated");
            }
            None => {
                return Err("Gradle not found. Please install Gradle or Android Studio, or run the project in Android Studio.".into());
            }
        }
    } else {
        println!("  ✓ Gradle wrapper found");
    }

    // Step 3: Build the project
    println!();
    println!("{}", "▶ Step 3: Building APK...".bright_cyan());

    let build_result = std::process::Command::new(&gradlew)
        .args(&["assembleDebug"])
        .current_dir(&jet_dir)
        .status()
        .map_err(|e| format!("Failed to build: {}", e))?;

    if build_result.success() {
        println!();
        println!("{}", "✓ Build successful!".bright_green());
        println!();
        println!("APK location:");
        println!("  {}", jet_dir.join("app/build/outputs/apk/debug/app-debug.apk").display().to_string().bright_cyan());
    } else {
        println!();
        println!("  ⚠ Build failed. Try running manually:");
        println!("    cd {} && ./gradlew assembleDebug", jet_dir.display());
    }

    Ok(())
}

/// Find gradle executable in common locations
fn find_gradle() -> Option<std::path::PathBuf> {
    // 1. Check PATH using std::env::split_paths
    if let Ok(path_env) = std::env::var("PATH") {
        for path in std::env::split_paths(&path_env) {
            let gradle_exe = if cfg!(windows) {
                path.join("gradle.bat")
            } else {
                path.join("gradle")
            };
            if gradle_exe.exists() {
                return Some(gradle_exe);
            }
        }
    }

    // 2. Check user's .gradle/wrapper/dists directory
    // Structure: .gradle/wrapper/dists/gradle-X.X.X-bin/<hash>/gradle-X.X.X/bin/gradle
    let home_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    if let Ok(home) = std::env::var(home_var) {
        let gradle_wrapper_dists = std::path::PathBuf::from(&home)
            .join(".gradle/wrapper/dists");

        if gradle_wrapper_dists.exists() {
            // Look for gradle-*-bin directories (Level 1)
            if let Ok(entries) = std::fs::read_dir(&gradle_wrapper_dists) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("gradle-") && name_str.ends_with("-bin") {
                        // Look inside for hash directories (Level 2)
                        let hash_dir_level = entry.path();
                        if let Ok(hash_entries) = std::fs::read_dir(&hash_dir_level) {
                            for hash_entry in hash_entries.flatten() {
                                let hash_dir = hash_entry.path();
                                if hash_dir.is_dir() {
                                    // Look for gradle-X.X.X directory inside hash dir (Level 3)
                                    if let Ok(gradle_home_entries) = std::fs::read_dir(&hash_dir) {
                                        for gradle_home_entry in gradle_home_entries.flatten() {
                                            let gradle_home = gradle_home_entry.path();
                                            if gradle_home.is_dir() {
                                                let gradle_bin = if cfg!(windows) {
                                                    gradle_home.join("bin/gradle.bat")
                                                } else {
                                                    gradle_home.join("bin/gradle")
                                                };
                                                if gradle_bin.exists() {
                                                    return Some(gradle_bin);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
