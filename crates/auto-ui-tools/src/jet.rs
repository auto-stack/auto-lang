//! Jetpack Compose project generation utilities
//!
//! This module provides the complete Jetpack Compose Android project workflow:
//! 1. Generate Kotlin code from .at files
//! 2. Generate full project structure (optional)
//! 3. Copy to output directory

use std::fs;
use std::path::{Path, PathBuf};

use auto_lang_core::database::{UIArtifact, UIBackend, UICache};
use auto_lang_core::Parser;
use auto_ui_compiler::ui_gen::jet::{JetGenerator, JetProjectConfig, ProjectGenerator};
use auto_ui_compiler::ui_gen::BackendGenerator;
use colored::Colorize;

use crate::util::hash_string;
use auto_val::AutoResult;

/// Parse project name from pac.at content
fn parse_pac_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parse render from pac.at content (supports array format, checks both "render:" and "backend:")
#[allow(dead_code)]
fn parse_pac_backend(content: &str) -> Option<String> {
    // First, try to parse as array: render: ["vue", "jet"]
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("render:") || line.starts_with("backend:") {
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
                            if !s.is_empty() {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .collect();
                    // Return first backend if jet is in the list
                    if backends.iter().any(|&b| b == "jet") {
                        return Some("jet".to_string());
                    }
                    return backends.first().map(|s| s.to_string());
                } else {
                    // Single backend
                    let value = value.trim_end_matches(',');
                    let value = value.trim_matches('"').trim_matches('\'');
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Check if jet is in the render list (supports both "render:" and legacy "backend:" fields)
fn has_jet_backend(content: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("render:") || line.starts_with("backend:") {
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
                            if !s.is_empty() {
                                Some(s)
                            } else {
                                None
                            }
                        })
                        .collect();
                    return backends.iter().any(|&b| b == "jet");
                } else {
                    let value = value.trim_end_matches(',');
                    let value = value.trim_matches('"').trim_matches('\'');
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

/// jet 项目读取缝（PLAN-809 T-06c / I-PROJECT）：pac.at 数据字段的
/// 两个来源共享同一构建算法。legacy 自扫路径保留给测试/旧调用；
/// 生产分派（Automan）走请求路径——生成器不再回读 pac.at/AutoConfig。
pub(crate) struct JetReads {
    pub name: String,
    pub gate_ok: bool,
}

impl JetReads {
    /// 旧臂自扫（行为冻结）。
    fn scan(root_dir: &Path) -> AutoResult<Self> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }
        let pac_content =
            fs::read_to_string(&pac_path).map_err(|e| format!("Failed to read pac.at: {}", e))?;
        Ok(Self {
            name: parse_pac_name(&pac_content).unwrap_or_else(|| "MyApp".to_string()),
            gate_ok: has_jet_backend(&pac_content),
        })
    }

    /// 请求路径（生产）：name 与 backend 门禁来自 GenerationRequest。
    fn from_request(req: &auto_project_model::generation_request::GenerationRequest) -> Self {
        let gate_ok = req
            .frontend_names()
            .iter()
            .any(|b| b == "jet" || b == "jetpack");
        Self {
            name: req
                .identity
                .name
                .clone()
                .unwrap_or_else(|| "MyApp".to_string()),
            gate_ok,
        }
    }
}

impl JetProject {
    /// Create a new Jet project context from a workspace directory
    /// （legacy 自扫路径——测试/旧调用；生产走 from_workspace_with_request。）
    pub fn from_workspace(root_dir: &Path) -> AutoResult<Self> {
        let reads = JetReads::scan(root_dir)?;
        Self::build(root_dir, reads)
    }

    /// 请求路径（PLAN-809 T-06c 生产入口）：pac.at 数据全部来自请求；
    /// 前端目录消费请求声明（T-06 R1/F-815-T06-R1-01——节点形式优先，
    /// 无声明回退布局探测链）。
    pub fn from_workspace_with_request(
        root_dir: &Path,
        req: &auto_project_model::generation_request::GenerationRequest,
    ) -> AutoResult<Self> {
        let front_override = crate::util::request_front_dir(root_dir, req);
        Self::build_front(
            root_dir,
            JetReads::from_request(req),
            front_override.as_deref(),
        )
    }

    fn build(root_dir: &Path, reads: JetReads) -> AutoResult<Self> {
        Self::build_front(root_dir, reads, None)
    }

    fn build_front(
        root_dir: &Path,
        reads: JetReads,
        front_override: Option<&Path>,
    ) -> AutoResult<Self> {
        if !reads.gate_ok {
            return Err("Backend 'jet' not found in pac.at. Add 'jet' to backend list.".into());
        }

        let name = reads.name;

        // Determine front directory: the request's declaration wins;
        // legacy keeps the multi-location probe chain.
        let front_dir = match front_override {
            Some(declared) => declared.to_path_buf(),
            None => {
                if root_dir.join("src").join("front").exists() {
                    root_dir.join("src").join("front")
                } else if root_dir.join("source").join("front").exists() {
                    root_dir.join("source").join("front")
                } else if root_dir.join("front").exists() {
                    root_dir.join("front")
                } else {
                    root_dir.join("src").join("front")
                }
            }
        };

        // Output directory
        let output_dir = root_dir.join("gen").join("front").join("jet");

        // Compile .at files to Kotlin
        let mut kotlin_files: Vec<(String, String)> = Vec::new();
        let mut widget_names: Vec<String> = Vec::new();

        // Process app.at if exists
        let app_at = front_dir.join("app.at");
        if app_at.exists() {
            match Self::compile_at_file(&app_at, &name, true) {
                Ok((files, names)) => {
                    kotlin_files.extend(files);
                    widget_names.extend(names);
                }
                Err(e) => {
                    println!(
                        "{} {}",
                        "Warning: Failed to compile app.at:".bright_yellow(),
                        e
                    );
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
                    match Self::compile_at_file(&path, &name, true) {
                        Ok((files, names)) => {
                            kotlin_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!(
                                "{} Failed to compile {}: {}",
                                "Warning:".bright_yellow(),
                                path.display(),
                                e
                            );
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
                    match Self::compile_at_file(&path, &name, true) {
                        Ok((files, names)) => {
                            kotlin_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!(
                                "{} Failed to compile {}: {}",
                                "Warning:".bright_yellow(),
                                path.display(),
                                e
                            );
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
                    match Self::compile_at_file(&path, &name, true) {
                        Ok((files, names)) => {
                            kotlin_files.extend(files);
                            widget_names.extend(names);
                        }
                        Err(e) => {
                            println!(
                                "{} Failed to compile {}: {}",
                                "Warning:".bright_yellow(),
                                path.display(),
                                e
                            );
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
        let reads = JetReads::scan(root_dir)?;
        Self::build_incremental(root_dir, reads)
    }

    /// 请求路径的增量形态（PLAN-809 T-06c）。
    pub fn from_workspace_incremental_with_request(
        root_dir: &Path,
        req: &auto_project_model::generation_request::GenerationRequest,
    ) -> AutoResult<(Self, Vec<String>)> {
        Self::build_incremental(root_dir, JetReads::from_request(req))
    }

    fn build_incremental(root_dir: &Path, reads: JetReads) -> AutoResult<(Self, Vec<String>)> {
        if !reads.gate_ok {
            return Err("Backend 'jet' not found in pac.at".into());
        }

        let name = reads.name;
        let front_dir = root_dir.join("source").join("front");
        let output_dir = root_dir.join("gen").join("front").join("jet");

        // Load cache
        // PLAN-814 R3-02：生成器指纹在 cache load/dirty 判定前注入（compiler
        // build.rs 单源经 SDK 槽；未注入则每次误判失效——Plan 698 SD-01 时序）。
        auto_lang_core::database::set_generator_fingerprint(
            auto_ui_compiler::generator_fingerprint(),
        );
        let mut cache = UICache::load(root_dir);
        let mut changed_files = Vec::new();

        // Process app.at
        let mut kotlin_files: Vec<(String, String)> = Vec::new();
        let mut widget_names: Vec<String> = Vec::new();

        let app_at = front_dir.join("app.at");
        if app_at.exists() {
            let content =
                fs::read_to_string(&app_at).map_err(|e| format!("Failed to read app.at: {}", e))?;
            let hash = hash_string(&content);

            if cache.is_dirty(&app_at, hash) {
                println!("  {} (changed)", "app.at".bright_yellow());
                match Self::compile_at_file(&app_at, &name, true) {
                    Ok((files, names)) => {
                        // Create artifacts for tracking
                        let artifacts: Vec<UIArtifact> = files
                            .iter()
                            .zip(names.iter())
                            .map(|((path, content), widget_name)| UIArtifact {
                                source_path: app_at.clone(),
                                widget_name: widget_name.clone(),
                                output_path: PathBuf::from(path),
                                source_hash: hash,
                                content_hash: hash_string(content),
                                backend: UIBackend::Jet,
                            })
                            .collect();

                        cache.update(app_at.clone(), hash, artifacts);
                        kotlin_files.extend(files);
                        widget_names.extend(names);
                        changed_files.push("app.at".to_string());
                    }
                    Err(e) => {
                        println!(
                            "{} {}",
                            "Warning: Failed to compile app.at:".bright_yellow(),
                            e
                        );
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

                                match Self::compile_at_file(&path, &name, false) {
                                    Ok((files, names)) => {
                                        let artifacts: Vec<UIArtifact> = files
                                            .iter()
                                            .zip(names.iter())
                                            .map(|((p, c), widget_name)| UIArtifact {
                                                source_path: path.clone(),
                                                widget_name: widget_name.clone(),
                                                output_path: PathBuf::from(p),
                                                source_hash: hash,
                                                content_hash: hash_string(c),
                                                backend: UIBackend::Jet,
                                            })
                                            .collect();

                                        cache.update(path.clone(), hash, artifacts);
                                        kotlin_files.extend(files);
                                        widget_names.extend(names);
                                        changed_files.push(file_name);
                                    }
                                    Err(e) => {
                                        println!(
                                            "{} Failed to compile {}: {}",
                                            "Warning:".bright_yellow(),
                                            file_name,
                                            e
                                        );
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

                                match Self::compile_at_file(&path, &name, false) {
                                    Ok((files, names)) => {
                                        let artifacts: Vec<UIArtifact> = files
                                            .iter()
                                            .zip(names.iter())
                                            .map(|((p, c), widget_name)| UIArtifact {
                                                source_path: path.clone(),
                                                widget_name: widget_name.clone(),
                                                output_path: PathBuf::from(p),
                                                source_hash: hash,
                                                content_hash: hash_string(c),
                                                backend: UIBackend::Jet,
                                            })
                                            .collect();

                                        cache.update(path.clone(), hash, artifacts);
                                        kotlin_files.extend(files);
                                        widget_names.extend(names);
                                        changed_files.push(file_name);
                                    }
                                    Err(e) => {
                                        println!(
                                            "{} Failed to compile {}: {}",
                                            "Warning:".bright_yellow(),
                                            file_name,
                                            e
                                        );
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

                                match Self::compile_at_file(&path, &name, false) {
                                    Ok((files, names)) => {
                                        let artifacts: Vec<UIArtifact> = files
                                            .iter()
                                            .zip(names.iter())
                                            .map(|((p, c), widget_name)| UIArtifact {
                                                source_path: path.clone(),
                                                widget_name: widget_name.clone(),
                                                output_path: PathBuf::from(p),
                                                source_hash: hash,
                                                content_hash: hash_string(c),
                                                backend: UIBackend::Jet,
                                            })
                                            .collect();

                                        cache.update(path.clone(), hash, artifacts);
                                        kotlin_files.extend(files);
                                        widget_names.extend(names);
                                        changed_files.push(file_name);
                                    }
                                    Err(e) => {
                                        println!(
                                            "{} Failed to compile {}: {}",
                                            "Warning:".bright_yellow(),
                                            file_name,
                                            e
                                        );
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

        Ok((
            Self {
                root_dir: root_dir.to_path_buf(),
                output_dir,
                name,
                front_dir,
                kotlin_files,
                widget_names,
            },
            changed_files,
        ))
    }

    /// Compile a single .at file to Kotlin code
    /// Returns (kotlin_files, widget_names)
    ///
    /// # Arguments
    /// * `at_path` - Path to the .at file
    /// * `project_name` - Project name (used for package name)
    /// * `verbose` - Print progress messages
    fn compile_at_file(
        at_path: &Path,
        project_name: &str,
        verbose: bool,
    ) -> Result<(Vec<(String, String)>, Vec<String>), String> {
        if verbose {
            let file_name = at_path.file_name().unwrap_or_default().to_string_lossy();
            println!("  {} {}", "Parsing".bright_cyan(), file_name);
            use std::io::Write;
            std::io::stdout().flush().ok();
        }

        let code = fs::read_to_string(at_path)
            .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

        // Parse with UI scenario
        use auto_lang_core::session::CompilerSession;
        let session = CompilerSession::ui().with_backend("jet");
        let mut parser = Parser::from(code.as_str());
        parser = parser.with_session(session);
        let ast = parser
            .parse()
            .map_err(|e| format!("Parse error: {:?}", e))?;

        // Extract AURA widgets from AST
        let mut widgets = Vec::new();
        for stmt in &ast.stmts {
            if let auto_lang_core::ast::Stmt::WidgetDecl(widget_decl) = stmt {
                let aura_widget = auto_ui_compiler::aura::extract_widget_from_decl(widget_decl)
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
            let kotlin_code = generator.generate(widget).map_err(|e| e.to_string())?;

            // Collect widget name
            names.push(widget.name.clone());

            // Generate file path: ui/widgets/{WidgetName}.kt
            let widget_name = &widget.name;
            let file_name = format!("{}.kt", widget_name);
            let relative_path = format!(
                "app/src/main/java/com/example/{}/ui/widgets/{}",
                safe_project_name, file_name
            );
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
        println!(
            "{}",
            "─────────────────────────────────".bright_yellow().bold()
        );
        println!(
            "{}",
            "  AURA Workspace → Jetpack Compose".bright_yellow().bold()
        );
        println!(
            "{}",
            "─────────────────────────────────".bright_yellow().bold()
        );
        println!();

        println!("{} {}", "Output:".bright_cyan(), self.output_dir.display());
        println!("{} {}", "Name:".bright_cyan(), self.name);
        println!(
            "{} {}",
            "Widgets:".bright_cyan(),
            self.widget_names.join(", ")
        );
        println!();

        // Create output directory
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        // Check if any kotlin files use navigation (to add dependency)
        let uses_navigation = self.kotlin_files.iter().any(|(_, content)| {
            content.contains("NavHostController")
                || content.contains("NavHost(")
                || content.contains("rememberNavController")
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
        println!(
            "{}",
            "  Jetpack Compose project generated!".bright_green().bold()
        );
        println!("═════════════════════════════════");
        println!();
        println!("{} {}", "Next steps:".bright_cyan(), "");
        println!("  cd {}", self.output_dir.display());
        println!("  ./gradlew assembleDebug");

        Ok(())
    }

    /// Generate only Kotlin files (no full project structure)
    pub fn generate_kotlin_only(&self, output_dir: &Path) -> AutoResult<()> {
        println!(
            "{}",
            "─────────────────────────────────".bright_yellow().bold()
        );
        println!(
            "{}",
            "  AURA → Kotlin Code Generation".bright_yellow().bold()
        );
        println!(
            "{}",
            "─────────────────────────────────".bright_yellow().bold()
        );
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
pub fn generate_jet_project(
    root_dir: &Path,
    output_dir: Option<&Path>,
    full_project: bool,
) -> AutoResult<()> {
    println!("{}", "Generating Jetpack Compose project".bright_cyan());

    // Load project context
    let project = JetProject::from_workspace(root_dir)?;
    generate_jet_project_with_project(root_dir, output_dir, full_project, project)
}

/// 请求路径（PLAN-809 T-06c 生产入口）。
pub fn generate_jet_project_with_request(
    root_dir: &Path,
    output_dir: Option<&Path>,
    full_project: bool,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    println!("{}", "Generating Jetpack Compose project".bright_cyan());
    let project = JetProject::from_workspace_with_request(root_dir, req)?;
    generate_jet_project_with_project(root_dir, output_dir, full_project, project)
}

fn generate_jet_project_with_project(
    _root_dir: &Path,
    output_dir: Option<&Path>,
    full_project: bool,
    project: JetProject,
) -> AutoResult<()> {
    // Determine output directory
    // PLAN-816 T-05 R2（F-R2-02/04）：显式 output 对全工程生成同样生效
    //（覆盖工程上下文的默认落点——镜像 vscode；此前 full_project 分支
    // 静默忽略 output_dir，选项被丢而产物落在默认布局）。
    let mut project = project;
    if let Some(out) = output_dir {
        project.output_dir = out.to_path_buf();
    }
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
pub fn run_jet_project(root_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    run_jet_project_inner(root_dir, args, None)
}

/// 请求路径（PLAN-809 R1-F02）。
pub fn run_jet_project_with_request(
    root_dir: &Path,
    args: Vec<String>,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    run_jet_project_inner(root_dir, args, Some(req))
}

fn run_jet_project_inner(
    root_dir: &Path,
    _args: Vec<String>,
    req: Option<&auto_project_model::generation_request::GenerationRequest>,
) -> AutoResult<()> {
    println!(
        "{}",
        "Running Jetpack Compose project (backend: jet)".bright_cyan()
    );

    // Step 1: Generate code (always, incremental)
    println!();
    println!("{}", "▶ Step 1: Generating Kotlin code...".bright_cyan());
    match req {
        Some(r) => generate_jet_project_with_request(root_dir, None, false, r)?,
        None => generate_jet_project(root_dir, None, false)?,
    }

    let jet_dir = root_dir.join("gen").join("front").join("jet");

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
        // PLAN-815 R4-05：托管形态经 owned 树设施（输出仅用于存在性判断，
        // 托管臂 stdout 转发日志流后以退出码裁决）；legacy 保持 output()。
        if crate::proc_host::current().is_some() {
            let status = crate::proc_host::run_external_status(
                std::path::Path::new("gradle"),
                &["wrapper"],
                &jet_dir,
                "jet-gradle-wrapper",
            )
            .map_err(|e| {
                format!("Failed to generate gradle wrapper: {e}. Please install Gradle or Android Studio.")
            })?;
            if !status.success() {
                return Err(
                    "Failed to generate gradle wrapper. Please install Gradle or Android Studio."
                        .into(),
                );
            }
        } else {
            std::process::Command::new("gradle")
                .args(&["wrapper"])
                .current_dir(&jet_dir)
                .output()
                .map_err(|e| format!("Failed to generate gradle wrapper: {}. Please install Gradle or Android Studio.", e))?;
        }
    } else {
        println!("  ✓ Gradle wrapper found");
    }

    // Step 3: Build the project
    println!();
    println!("{}", "▶ Step 3: Building Android project...".bright_cyan());

    // PLAN-815 R4-05（F-815-T05-R4-05）：构建是 owned 树——取消/EOF 真终止。
    let build_result =
        crate::proc_host::run_external_status(&gradlew, &["assembleDebug"], &jet_dir, "jet-build")
            .map_err(|e| format!("Failed to run gradlew assembleDebug: {}", e))?;

    if !build_result.success() {
        return Err("Build failed. Check the error messages above.".into());
    }
    println!("  ✓ Build successful");

    // Step 4: Install and run on device/emulator
    println!();
    println!(
        "{}",
        "▶ Step 4: Installing on device/emulator...".bright_cyan()
    );

    // Check for connected devices
    // PLAN-815 R4-05：托管形态=owned 树+捕获排水（取消真终止）；legacy
    // 保持 output()。
    let has_device = if crate::proc_host::current().is_some() {
        let spec = crate::proc_host::TrackedSpawn {
            capture_stdout: true,
            ..crate::proc_host::TrackedSpawn::new("adb", vec!["devices".to_string()])
        };
        match crate::proc_host::spawn_tracked(spec, "jet-adb") {
            Ok(tracked) => {
                let (stdout, end) = crate::proc_host::drain_captured_stdout(&tracked);
                // PLAN-815 R5（F-815-T05-R4-05-P1）：排水必须应答本操作的
                // 取消/EOF——被取消的操作在这里以取消哨兵收口（终态由
                // map_flow 映射 Cancelled），不再把取消后的 Ok 当 completed。
                if crate::proc_host::cancelled() {
                    return Err(crate::proc_host::CANCELLED_ERR.into());
                }
                // PLAN-815 R6（F-815-T05-R4-05-P2）：非自然捕获结束
                // （BoundOverrun/Truncated/IoError/无取消的 Stopped）与非零
                // 退出是**具名失败**——不完整的探测不是"无设备"，不得把
                // 分类抹成 bool 后误报 completed。探测树先终止+有界收尸
                // （界限触发后 owned ADB 仍活时，后续 wait 也有实际收尾
                // 边界），再带分类返回。
                let exited_ok = if end.is_natural_eof() {
                    match crate::proc_host::wait_tracked(&tracked) {
                        Ok(status) if status.success() => true,
                        Ok(status) => {
                            return Err(format!(
                                "adb devices probe failed (adb exited with {status}) — device detection unavailable"
                            )
                            .into());
                        }
                        Err(reason) => {
                            return Err(format!(
                                "adb devices probe wait failed: {reason} — device detection unavailable"
                            )
                            .into());
                        }
                    }
                } else {
                    tracked.kill();
                    let teardown = crate::proc_host::wait_tracked(&tracked);
                    return Err(format!(
                        "adb devices probe did not complete ({end:?}; owned tree teardown {teardown:?}) — device detection unavailable"
                    )
                    .into());
                };
                // 自然 EOF + 干净退出的完整输出才用于设备判定（空列表=
                // 合法"无设备"）。
                exited_ok
                    && String::from_utf8_lossy(&stdout)
                        .lines()
                        .any(|line| line.contains("\tdevice"))
            }
            Err(spawn_reason) => {
                // 探测进程未能启动同样是具名失败（此前 spawn Err 也被吞成
                // "无设备"→completed——R6 同因 sibling）。
                return Err(format!(
                    "adb devices probe failed to start: {spawn_reason} — device detection unavailable"
                )
                .into());
            }
        }
    } else {
        std::process::Command::new("adb")
            .args(&["devices"])
            .output()
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                Some(stdout.lines().any(|line| line.contains("\tdevice")))
            })
            .unwrap_or(false)
    };

    if has_device {
        // Install the APK
        // PLAN-815 R4-05：安装命令同为 owned 树。
        let install_result = crate::proc_host::run_external_status(
            &gradlew,
            &["installDebug"],
            &jet_dir,
            "jet-install",
        )
        .map_err(|e| format!("Failed to install: {}", e))?;

        if install_result.success() {
            // PLAN-815 R4-04：包名显示消费请求字段（req 形态）——不再
            // 回读主 pac.at；legacy 保持自扫。
            let project_name = match req {
                Some(r) => r
                    .identity
                    .name
                    .clone()
                    .unwrap_or_else(|| "MyApp".to_string()),
                None => {
                    let pac_path = root_dir.join("pac.at");
                    if pac_path.exists() {
                        if let Ok(content) = fs::read_to_string(&pac_path) {
                            parse_pac_name(&content).unwrap_or_else(|| "MyApp".to_string())
                        } else {
                            "MyApp".to_string()
                        }
                    } else {
                        "MyApp".to_string()
                    }
                }
            };

            println!("  ✓ App installed successfully!");
            println!();
            println!(
                "{}",
                "App is now running on your device/emulator.".bright_green()
            );
            println!(
                "Package: com.example.{}",
                project_name.to_lowercase().replace("-", "_")
            );
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
    build_jet_project_inner(root_dir, None)
}

/// 请求路径（PLAN-809 R2-F02：build 分派同样经合同）。
pub fn build_jet_project_with_request(
    root_dir: &Path,
    req: &auto_project_model::generation_request::GenerationRequest,
) -> AutoResult<()> {
    build_jet_project_inner(root_dir, Some(req))
}

fn build_jet_project_inner(
    root_dir: &Path,
    req: Option<&auto_project_model::generation_request::GenerationRequest>,
) -> AutoResult<()> {
    println!(
        "{}",
        "Building Jetpack Compose project (backend: jet)".bright_cyan()
    );

    // PLAN-815 R5（F-815-T05-R5-01）：协议严格失败语义只在**托管形态**
    // （工具宿主 RunIo 在装）生效；旧公开门面（CLI 直跑/Man 转发）保持
    // 基线返回语义——构建失败打印提示后 Ok，不静默升级为 Err 合同。
    let strict = crate::proc_host::current().is_some();

    // Step 1: Generate code (always, incremental)
    println!();
    println!("{}", "▶ Step 1: Generating Kotlin code...".bright_cyan());
    match req {
        Some(r) => generate_jet_project_with_request(root_dir, None, false, r)?,
        None => generate_jet_project(root_dir, None, false)?,
    }

    let jet_dir = root_dir.join("gen").join("front").join("jet");

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
                // PLAN-815 R4-05（F-815-T05-R4-05）：托管形态经 owned 树
                // 设施（取消/EOF 终止整树）；legacy 直跑保持原 Command 形态。
                let result = crate::proc_host::run_external_status(
                    &gradle_path,
                    &["wrapper"],
                    &jet_dir,
                    "jet-gradle-wrapper",
                )
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

    // PLAN-815 R4-05：同上——gradle 构建是 owned 树（取消真终止，不是
    // 只置标志等 status 返回）。
    let build_result =
        crate::proc_host::run_external_status(&gradlew, &["assembleDebug"], &jet_dir, "jet-build")
            .map_err(|e| format!("Failed to build: {}", e))?;

    let apk = jet_dir.join("app/build/outputs/apk/debug/app-debug.apk");
    if build_result.success() {
        // PLAN-815 R4-06（F-815-T05-R4-06）：产物 oracle——退出码 0 但产物
        // 缺失（假 wrapper/未真正执行）按具名失败回报，不冒称成功
        // （仅托管/协议形态；legacy 基线保持成功打印语义——R5-01）。
        if strict && !apk.is_file() {
            return Err(format!(
                "gradlew assembleDebug exited 0 but the APK artifact is missing at {} — the build did not really run",
                apk.display()
            ).into());
        }
        println!();
        println!("{}", "✓ Build successful!".bright_green());
        println!();
        println!("APK location:");
        println!("  {}", apk.display().to_string().bright_cyan());
    } else {
        // PLAN-815 R4-06/R5-01：托管/协议形态真实构建失败=具名 Err；
        // legacy 基线=打印提示后 Ok（旧公开入口返回语义不静默改）。
        if strict {
            return Err(format!(
                "jet build failed (gradlew assembleDebug exited with {build_result}); try manually: cd {} && ./gradlew assembleDebug",
                jet_dir.display()
            ).into());
        }
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
        let gradle_wrapper_dists = std::path::PathBuf::from(&home).join(".gradle/wrapper/dists");

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
