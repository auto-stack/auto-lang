//! VSCode Extension project generation utilities
//!
//! This module generates a complete VSCode extension project from AURA widgets.
//! The extension renders the app widget in a sidebar webview panel using
//! a2vue-generated Vue 3 content.
//!
//! Architecture:
//! - pac.at config -> vscode.rs -> package.json, extension.ts, AppPanel.ts
//! - AURA widgets -> VueGenerator -> webview-ui/ (Vue app)
//!
//! The generated extension uses the VSCode Webview API to display the Vue app
//! in a sidebar or editor panel.

use std::fs;
use std::path::Path;

use colored::Colorize;
use auto_lang::ui_gen::{BackendGenerator, VueGenerator};

use crate::AutoResult;

// ---------------------------------------------------------------------------
// VscodeConfig — parsed from pac.at `vscode { }` block
// ---------------------------------------------------------------------------

/// Configuration for the VSCode extension, parsed from pac.at.
#[derive(Debug, Clone)]
pub struct VscodeConfig {
    /// Where to display the panel: "sidebar" or "editor". Default: "sidebar".
    pub panel: String,
    /// VSCode command ID. Default: "<project-name>.open".
    pub command: String,
    /// Panel display title. Default: project name.
    pub title: String,
    /// Optional icon path relative to project root.
    pub icon: Option<String>,
}

impl Default for VscodeConfig {
    fn default() -> Self {
        Self {
            panel: "sidebar".to_string(),
            command: String::new(), // will be derived from project name
            title: String::new(),   // will be derived from project name
            icon: None,
        }
    }
}

impl VscodeConfig {
    /// Build a VscodeConfig with sensible defaults derived from `project_name`.
    pub fn with_defaults(project_name: &str) -> Self {
        let kebab = to_kebab_case(project_name);
        Self {
            panel: "sidebar".to_string(),
            command: format!("{}.open", kebab),
            title: project_name.to_string(),
            icon: None,
        }
    }

    /// Parse the `vscode { }` block from pac.at content.
    /// Returns defaults if no block is found.
    pub fn parse_from_pac(pac_content: &str, project_name: &str) -> Self {
        let mut config = Self::with_defaults(project_name);

        // Look for a `vscode { ... }` block
        let mut in_block = false;
        for line in pac_content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("vscode") && trimmed.contains('{') {
                in_block = true;
                continue;
            }
            if in_block && trimmed == "}" {
                break;
            }
            if !in_block {
                continue;
            }

            // Parse key: value pairs inside the block
            if let Some((key, value)) = parse_kv(trimmed) {
                match key {
                    "panel" => config.panel = value,
                    "command" => config.command = value,
                    "title" => config.title = value,
                    "icon" => config.icon = Some(value),
                    _ => {}
                }
            }
        }

        config
    }
}

// ---------------------------------------------------------------------------
// VscodeProject — project generation context
// ---------------------------------------------------------------------------

/// VSCode extension project generation context.
pub struct VscodeProject {
    /// Project root directory (where pac.at is).
    pub root_dir: std::path::PathBuf,
    /// Output directory (<root>/vscode).
    pub output_dir: std::path::PathBuf,
    /// Project name.
    pub name: String,
    /// Front source directory.
    pub front_dir: std::path::PathBuf,
    /// VSCode-specific configuration.
    pub config: VscodeConfig,
    /// Generated Vue app code (App.vue content).
    pub app_vue_code: String,
    /// All sub-widget components (relative_dir, name, code, widget_name).
    pub components: Vec<(String, String, String, String)>,
}

impl VscodeProject {
    /// Create a new VscodeProject from a workspace directory.
    pub fn from_workspace(root_dir: &Path) -> AutoResult<Self> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }

        let pac_content = fs::read_to_string(&pac_path)
            .map_err(|e| format!("Failed to read pac.at: {}", e))?;

        // Parse project name
        let name = parse_pac_name(&pac_content)
            .unwrap_or_else(|| "my-extension".to_string());

        // Parse vscode config block
        let config = VscodeConfig::parse_from_pac(&pac_content, &name);

        // Determine front directory
        let front_dir = if root_dir.join("source").join("front").exists() {
            root_dir.join("source").join("front")
        } else if root_dir.join("front").exists() {
            root_dir.join("front")
        } else {
            root_dir.join("front")
        };

        let output_dir = root_dir.join("vscode");

        // Compile .at files to Vue using the existing VueGenerator
        let mut all_components: Vec<(String, String, String, String)> = Vec::new();

        // Process app.at
        let app_at = front_dir.join("app.at");
        let root_app_at = root_dir.join("app.at");
        let app_at_path = if app_at.exists() {
            app_at
        } else if root_app_at.exists() {
            root_app_at
        } else {
            app_at
        };

        if app_at_path.exists() {
            match compile_at_to_vue(&app_at_path) {
                Ok((vue_code, widget_names)) => {
                    let widget_name = widget_names.first()
                        .map(|s| s.as_str())
                        .unwrap_or("App");
                    all_components.push((
                        "".to_string(),
                        "app".to_string(),
                        vue_code,
                        widget_name.to_string(),
                    ));
                }
                Err(e) => {
                    println!(
                        "{} Failed to compile {}: {}",
                        "Warning:".bright_yellow(),
                        app_at_path.display(),
                        e
                    );
                }
            }
        }

        // Process widgets/ directory
        let widgets_dir = front_dir.join("widgets");
        if widgets_dir.exists() {
            if let Ok(entries) = fs::read_dir(&widgets_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        match compile_at_to_vue(&path) {
                            Ok((vue_code, widget_names)) => {
                                let widget_name = widget_names.first()
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        path.file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("Widget")
                                            .to_string()
                                    });
                                all_components.push((
                                    "components".to_string(),
                                    widget_name.to_string(),
                                    vue_code,
                                    widget_name.clone(),
                                ));
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
        }

        // Process pages/ directory
        let pages_dir = front_dir.join("pages");
        if pages_dir.exists() {
            if let Ok(entries) = fs::read_dir(&pages_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        let file_stem = path.file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("page");
                        match compile_at_to_vue(&path) {
                            Ok((vue_code, widget_names)) => {
                                let widget_name = widget_names.first()
                                    .cloned()
                                    .unwrap_or_else(|| file_stem.to_string());
                                all_components.push((
                                    "pages".to_string(),
                                    file_stem.to_string(),
                                    vue_code,
                                    widget_name,
                                ));
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
        }

        // Get App.vue code
        let app_vue_code = all_components.iter()
            .find(|(_, name, _, _)| name == "app")
            .map(|(_, _, code, _)| code.clone())
            .unwrap_or_default();

        Ok(Self {
            root_dir: root_dir.to_path_buf(),
            output_dir,
            name,
            front_dir,
            config,
            app_vue_code,
            components: all_components,
        })
    }

    /// Check if the generated project already exists.
    pub fn exists(&self) -> bool {
        self.output_dir.exists() && self.output_dir.join("package.json").exists()
    }

    /// Generate the complete VSCode extension project.
    pub fn generate(&self) -> AutoResult<()> {
        println!(
            "{}",
            "---------------------------------".bright_yellow().bold()
        );
        println!(
            "{}",
            "  AURA Workspace -> VSCode Extension".bright_yellow().bold()
        );
        println!(
            "{}",
            "---------------------------------".bright_yellow().bold()
        );
        println!();

        println!("{} {}", "Output:".bright_cyan(), self.output_dir.display());
        println!("{} {}", "Name:".bright_cyan(), self.name);
        println!("{} {}", "Panel:".bright_cyan(), self.config.panel);
        println!("{} {}", "Command:".bright_cyan(), self.config.command);
        println!();

        // Create output directories
        let src_dir = self.output_dir.join("src").join("panels");
        let webview_src_dir = self.output_dir.join("webview-ui").join("src");
        let webview_components_dir = webview_src_dir.join("components");
        let media_dir = self.output_dir.join("media");
        let vscode_dir = self.output_dir.join(".vscode");

        fs::create_dir_all(&src_dir)
            .map_err(|e| format!("Failed to create src/panels: {}", e))?;
        fs::create_dir_all(&webview_components_dir)
            .map_err(|e| format!("Failed to create webview-ui/src/components: {}", e))?;
        fs::create_dir_all(&media_dir)
            .map_err(|e| format!("Failed to create media: {}", e))?;
        fs::create_dir_all(&vscode_dir)
            .map_err(|e| format!("Failed to create .vscode: {}", e))?;

        println!("{}", "  Created directory structure".bright_green());

        // Generate all files
        self.write_package_json()?;
        self.write_extension_ts()?;
        self.write_app_panel_ts()?;
        self.write_webview_index_html()?;
        self.write_webview_main_ts()?;
        self.write_webview_app_vue()?;
        self.write_webview_package_json()?;
        self.write_webview_vite_config()?;
        self.write_webview_tsconfig()?;
        self.write_webview_env_dts()?;
        self.write_tailwind_config()?;
        self.write_postcss_config()?;
        self.write_base_css()?;
        self.write_tsconfig()?;
        self.write_webpack_config()?;
        self.write_vscodeignore()?;
        self.write_launch_json()?;
        self.write_tasks_json()?;

        // Write sub-widget components
        for (relative_dir, name, code, widget_name) in &self.components {
            if name != "app" {
                let comp_dir = if relative_dir == "components" || relative_dir.is_empty() {
                    webview_components_dir.clone()
                } else if relative_dir == "pages" {
                    webview_src_dir.join("pages")
                } else {
                    webview_components_dir.join(relative_dir)
                };

                fs::create_dir_all(&comp_dir)
                    .map_err(|e| format!("Failed to create {}: {}", comp_dir.display(), e))?;

                let file_name = format!("{}.vue", widget_name);
                let file_path = comp_dir.join(&file_name);
                fs::write(&file_path, code)
                    .map_err(|e| format!("Failed to write {}: {}", file_path.display(), e))?;
                println!("  {} {}", "Generated".bright_green(), file_name);
            }
        }

        // Copy icon if specified
        if let Some(ref icon_rel) = self.config.icon {
            let icon_src = self.root_dir.join(icon_rel);
            if icon_src.exists() {
                let icon_dest = media_dir.join(
                    Path::new(icon_rel).file_name().unwrap_or_default(),
                );
                fs::copy(&icon_src, &icon_dest)
                    .map_err(|e| format!("Failed to copy icon: {}", e))?;
                println!("  {} icon from {}", "Copied".bright_green(), icon_rel);
            }
        }

        println!();
        println!(
            "{}",
            "  VSCode extension project generated!".bright_green().bold()
        );

        Ok(())
    }

    // -- Individual file writers -------------------------------------------

    fn write_package_json(&self) -> AutoResult<()> {
        let content = generate_package_json(&self.name, &self.config);
        let path = self.output_dir.join("package.json");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write package.json: {}", e))?;
        println!("  {} package.json", "Generated".bright_green());
        Ok(())
    }

    fn write_extension_ts(&self) -> AutoResult<()> {
        let content = generate_extension_ts(&self.config);
        let path = self.output_dir.join("src").join("extension.ts");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write extension.ts: {}", e))?;
        println!("  {} src/extension.ts", "Generated".bright_green());
        Ok(())
    }

    fn write_app_panel_ts(&self) -> AutoResult<()> {
        let content = generate_app_panel_ts(&self.config);
        let path = self.output_dir.join("src").join("panels").join("AppPanel.ts");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write AppPanel.ts: {}", e))?;
        println!("  {} src/panels/AppPanel.ts", "Generated".bright_green());
        Ok(())
    }

    fn write_webview_index_html(&self) -> AutoResult<()> {
        let content = generate_webview_index_html(&self.config);
        let path = self.output_dir.join("webview-ui").join("index.html");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/index.html: {}", e))?;
        println!("  {} webview-ui/index.html", "Generated".bright_green());
        Ok(())
    }

    fn write_webview_main_ts(&self) -> AutoResult<()> {
        let content = generate_webview_main_ts();
        let path = self.output_dir.join("webview-ui").join("src").join("main.ts");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/src/main.ts: {}", e))?;
        println!("  {} webview-ui/src/main.ts", "Generated".bright_green());
        Ok(())
    }

    fn write_webview_app_vue(&self) -> AutoResult<()> {
        let path = self.output_dir.join("webview-ui").join("src").join("App.vue");
        fs::write(&path, &self.app_vue_code)
            .map_err(|e| format!("Failed to write App.vue: {}", e))?;
        println!("  {} webview-ui/src/App.vue", "Generated".bright_green());
        Ok(())
    }

    fn write_webview_package_json(&self) -> AutoResult<()> {
        let content = generate_webview_package_json(&self.name);
        let path = self.output_dir.join("webview-ui").join("package.json");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/package.json: {}", e))?;
        println!("  {} webview-ui/package.json", "Generated".bright_green());
        Ok(())
    }

    fn write_tsconfig(&self) -> AutoResult<()> {
        let content = generate_tsconfig();
        let path = self.output_dir.join("tsconfig.json");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;
        println!("  {} tsconfig.json", "Generated".bright_green());
        Ok(())
    }

    fn write_webpack_config(&self) -> AutoResult<()> {
        let content = generate_webpack_config();
        let path = self.output_dir.join("webpack.config.js");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webpack.config.js: {}", e))?;
        println!("  {} webpack.config.js", "Generated".bright_green());
        Ok(())
    }

    fn write_vscodeignore(&self) -> AutoResult<()> {
        let content = generate_vscodeignore();
        let path = self.output_dir.join(".vscodeignore");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write .vscodeignore: {}", e))?;
        println!("  {} .vscodeignore", "Generated".bright_green());
        Ok(())
    }

    fn write_webview_vite_config(&self) -> AutoResult<()> {
        let content = generate_webview_vite_config();
        let path = self.output_dir.join("webview-ui").join("vite.config.ts");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/vite.config.ts: {}", e))?;
        println!("  {} webview-ui/vite.config.ts", "Generated".bright_green());
        Ok(())
    }

    fn write_webview_tsconfig(&self) -> AutoResult<()> {
        let content = generate_webview_tsconfig();
        let path = self.output_dir.join("webview-ui").join("tsconfig.json");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/tsconfig.json: {}", e))?;
        println!("  {} webview-ui/tsconfig.json", "Generated".bright_green());
        Ok(())
    }

    fn write_webview_env_dts(&self) -> AutoResult<()> {
        let content = generate_webview_env_dts();
        let path = self.output_dir.join("webview-ui").join("src").join("env.d.ts");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/src/env.d.ts: {}", e))?;
        println!("  {} webview-ui/src/env.d.ts", "Generated".bright_green());
        Ok(())
    }

    fn write_tailwind_config(&self) -> AutoResult<()> {
        let content = generate_tailwind_config();
        let path = self.output_dir.join("webview-ui").join("tailwind.config.cjs");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/tailwind.config.cjs: {}", e))?;
        println!("  {} webview-ui/tailwind.config.cjs", "Generated".bright_green());
        Ok(())
    }

    fn write_postcss_config(&self) -> AutoResult<()> {
        let content = generate_postcss_config();
        let path = self.output_dir.join("webview-ui").join("postcss.config.cjs");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/postcss.config.cjs: {}", e))?;
        println!("  {} webview-ui/postcss.config.cjs", "Generated".bright_green());
        Ok(())
    }

    fn write_base_css(&self) -> AutoResult<()> {
        let content = generate_base_css();
        let assets_dir = self.output_dir.join("webview-ui").join("src").join("assets");
        fs::create_dir_all(&assets_dir)
            .map_err(|e| format!("Failed to create webview-ui/src/assets: {}", e))?;
        let path = assets_dir.join("index.css");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write webview-ui/src/assets/index.css: {}", e))?;
        println!("  {} webview-ui/src/assets/index.css", "Generated".bright_green());
        Ok(())
    }

    fn write_launch_json(&self) -> AutoResult<()> {
        let content = generate_launch_json();
        let path = self.output_dir.join(".vscode").join("launch.json");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write .vscode/launch.json: {}", e))?;
        println!("  {} .vscode/launch.json", "Generated".bright_green());
        Ok(())
    }

    fn write_tasks_json(&self) -> AutoResult<()> {
        let content = generate_tasks_json();
        let path = self.output_dir.join(".vscode").join("tasks.json");
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write .vscode/tasks.json: {}", e))?;
        println!("  {} .vscode/tasks.json", "Generated".bright_green());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Public API functions (called from automan.rs)
// ---------------------------------------------------------------------------

/// Generate the VSCode extension project (auto gen command).
pub fn generate_vscode_project(
    root_dir: &Path,
    output_dir: Option<&Path>,
    project: bool,
) -> AutoResult<()> {
    println!("{}", "Generating VSCode extension project".bright_cyan());

    let project_ctx = VscodeProject::from_workspace(root_dir)?;

    let output = output_dir
        .map(|p| p.to_path_buf())
        .unwrap_or(project_ctx.output_dir.clone());

    // If caller gave a different output dir, temporarily override
    let actual_output = project_ctx.output_dir.clone();
    let mut proj = project_ctx;
    if output != actual_output {
        proj.output_dir = output;
    }

    if project || !proj.exists() {
        proj.generate()?;
    } else {
        proj.generate()?;
    }

    Ok(())
}

/// Build the VSCode extension project (auto build command).
pub fn build_vscode_project(root_dir: &Path) -> AutoResult<()> {
    println!("{}", "Building VSCode extension project".bright_cyan());

    // Step 1: Generate code
    println!();
    println!(
        "{}",
        "  Step 1: Generating VSCode extension code...".bright_cyan()
    );
    generate_vscode_project(root_dir, None, false)?;

    let vscode_dir = root_dir.join("vscode");

    // Step 2: Check for npm
    println!();
    println!(
        "{}",
        "  Step 2: Checking build tools...".bright_cyan()
    );

    let has_npm = {
        #[cfg(windows)]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "where", "npm"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(windows))]
        {
            std::process::Command::new("which")
                .arg("npm")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    };

    if !has_npm {
        println!(
            "  {} npm not found. Cannot build automatically.",
            "Warning:".bright_yellow()
        );
        println!("  Please install Node.js from https://nodejs.org/");
        println!("  Project location: {}", vscode_dir.display());
        return Ok(());
    }

    // Step 3: Install webview dependencies
    println!("  {} npm found", "OK".bright_green());
    println!();
    println!(
        "{}",
        "  Step 3: Installing dependencies...".bright_cyan()
    );

    let webview_dir = vscode_dir.join("webview-ui");

    // Install webview-ui dependencies if needed
    if webview_dir.exists() && !webview_dir.join("node_modules").exists() {
        #[cfg(windows)]
        let webview_install = std::process::Command::new("cmd")
            .args(&["/C", "npm", "install"])
            .current_dir(&webview_dir)
            .status();

        #[cfg(not(windows))]
        let webview_install = std::process::Command::new("npm")
            .args(&["install"])
            .current_dir(&webview_dir)
            .status();

        match webview_install {
            Ok(status) if status.success() => {
                println!("  {} webview-ui dependencies installed", "OK".bright_green());
            }
            _ => {
                println!(
                    "  {} Failed to install webview-ui dependencies",
                    "Warning:".bright_yellow()
                );
            }
        }
    }

    // Install root dependencies if needed
    if !vscode_dir.join("node_modules").exists() {
        #[cfg(windows)]
        let root_install = std::process::Command::new("cmd")
            .args(&["/C", "npm", "install"])
            .current_dir(&vscode_dir)
            .status();

        #[cfg(not(windows))]
        let root_install = std::process::Command::new("npm")
            .args(&["install"])
            .current_dir(&vscode_dir)
            .status();

        match root_install {
            Ok(status) if status.success() => {
                println!("  {} Root dependencies installed", "OK".bright_green());
            }
            _ => {
                println!(
                    "  {} Failed to install root dependencies",
                    "Warning:".bright_yellow()
                );
            }
        }
    }

    // Step 4: Build (webview + extension)
    println!();
    println!(
        "{}",
        "  Step 4: Building project...".bright_cyan()
    );

    #[cfg(windows)]
    let npm_result = std::process::Command::new("cmd")
        .args(&["/C", "npm", "run", "build"])
        .current_dir(&vscode_dir)
        .status();

    #[cfg(not(windows))]
    let npm_result = std::process::Command::new("npm")
        .args(&["run", "build"])
        .current_dir(&vscode_dir)
        .status();

    match npm_result {
        Ok(status) if status.success() => {
            println!();
            println!(
                "{}",
                "  VSCode extension built successfully!".bright_green().bold()
            );
        }
        Ok(status) => {
            println!();
            println!(
                "  {} Build exited with code {:?}",
                "Warning:".bright_yellow(),
                status.code()
            );
            println!("  Try running manually:");
            println!("    cd {} && npm install && npm run build", vscode_dir.display());
        }
        Err(e) => {
            println!("  {} Build failed: {}", "Error:".bright_red(), e);
        }
    }

    Ok(())
}

/// Run the VSCode extension in development mode (auto run command).
pub fn run_vscode_project(root_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    println!(
        "{}",
        "Running VSCode extension project".bright_cyan()
    );

    // Step 1: Generate code
    println!();
    println!(
        "{}",
        "  Step 1: Generating VSCode extension code...".bright_cyan()
    );
    generate_vscode_project(root_dir, None, false)?;

    let vscode_dir = root_dir.join("vscode");

    println!();
    println!(
        "{}",
        "===================================".bright_green()
    );
    println!(
        "{}",
        "  VSCode Extension ready!".bright_green().bold()
    );
    println!(
        "{}",
        "===================================".bright_green()
    );
    println!();
    println!("To run the extension in development mode:");
    println!();
    println!(
        "  {}",
        format!(
            "code --extensionDevelopmentPath={}",
            vscode_dir.display()
        )
        .bright_cyan()
    );
    println!();
    println!("Or open VSCode and use F5 to launch the Extension Development Host.");
    println!("  Project location: {}", vscode_dir.display());

    Ok(())
}

// ---------------------------------------------------------------------------
// Template generators
// ---------------------------------------------------------------------------

fn generate_package_json(name: &str, config: &VscodeConfig) -> String {
    let command = &config.command;
    let title = &config.title;
    let kebab = to_kebab_case(name);

    let icon_field = if config.icon.is_some() {
        r#",
    "icon": "media/icon.png""#
        .to_string()
    } else {
        "".to_string()
    };

    format!(
        r#"{{
    "name": "{kebab}",
    "displayName": "{title}",
    "description": "Auto-generated VSCode extension from AURA widgets",
    "version": "0.1.0",
    "publisher": "auto-lang",
    "engines": {{
        "vscode": "^1.85.0"
    }},
    "categories": [
        "Other"
    ],
    "main": "./dist/extension.js",
    "contributes": {{
        "commands": [
            {{
                "command": "{command}",
                "title": "Open {title}"
            }}
        ]
    }},
    "scripts": {{
        "vscode:prepublish": "npm run compile",
        "build": "npm run webview:install && npm run webview:build && npm install && npm run compile",
        "compile": "webpack --mode production",
        "watch": "webpack --mode development --watch",
        "webview:install": "cd webview-ui && npm install",
        "webview:build": "cd webview-ui && npm run build"
    }},
    "devDependencies": {{
        "@types/vscode": "^1.85.0",
        "@types/node": "^20.0.0",
        "typescript": "^5.3.0",
        "ts-loader": "^9.5.0",
        "webpack": "^5.90.0",
        "webpack-cli": "^5.1.0"
    }}{icon_field}
}}
"#
    )
}

fn generate_extension_ts(config: &VscodeConfig) -> String {
    let command = &config.command;
    let title = &config.title;

    format!(
        r#"// Auto-generated by a2vscode — VSCode Extension entry point

import * as vscode from 'vscode';
import {{ AppPanel }} from './panels/AppPanel';

export function activate(context: vscode.ExtensionContext) {{
    console.log(`{title} extension activated`);

    // Register command to open the panel
    context.subscriptions.push(
        vscode.commands.registerCommand('{command}', () => {{
            AppPanel.createOrShow(context.extensionUri, '{title}');
        }})
    );

    // Add status bar icon (bottom-right)
    const statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Right,
        100
    );
    statusBarItem.text = '$(globe) {title}';
    statusBarItem.tooltip = 'Open {title}';
    statusBarItem.command = '{command}';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);
}}

export function deactivate() {{
    AppPanel.dispose();
}}
"#
    )
}

fn generate_app_panel_ts(config: &VscodeConfig) -> String {
    let command = &config.command;
    let title = &config.title;

    format!(
        r#"// Auto-generated by a2vscode — Webview panel for the AURA app

import * as vscode from 'vscode';

export class AppPanel {{
    public static currentPanel: AppPanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private readonly _extensionUri: vscode.Uri;
    private _disposables: vscode.Disposable[] = [];

    public static createOrShow(extensionUri: vscode.Uri, title: string) {{
        // If a panel already exists, show it
        if (AppPanel.currentPanel) {{
            AppPanel.currentPanel._panel.reveal(vscode.ViewColumn.Beside);
            return;
        }}

        // Create a new panel in the right side (beside the active editor)
        const panel = vscode.window.createWebviewPanel(
            '{command}',
            title,
            {{ viewColumn: vscode.ViewColumn.Beside, preserveFocus: true }},
            getWebviewOptions(extensionUri)
        );

        AppPanel.currentPanel = new AppPanel(panel, extensionUri);
    }}

    public static dispose() {{
        if (AppPanel.currentPanel) {{
            AppPanel.currentPanel.dispose();
        }}
    }}

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {{
        this._panel = panel;
        this._extensionUri = extensionUri;

        // Set the webview's HTML content
        this._update();

        // Handle panel disposal
        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        // Handle messages from the webview
        this._panel.webview.onDidReceiveMessage(
            (message: {{ type: string; data?: any }}) => {{
                switch (message.type) {{
                    default:
                        console.log('Received message from webview:', message);
                }}
            }},
            null,
            this._disposables
        );
    }}

    public dispose() {{
        AppPanel.currentPanel = undefined;

        this._panel.dispose();

        while (this._disposables.length) {{
            const disposable = this._disposables.pop();
            if (disposable) {{
                disposable.dispose();
            }}
        }}
    }}

    private _update() {{
        const webview = this._panel.webview;
        this._panel.webview.html = this._getHtmlForWebview(webview);
    }}

    private _getHtmlForWebview(webview: vscode.Webview): string {{
        // Get the local path to the webview-ui build output
        const scriptUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._extensionUri, 'webview-ui', 'dist', 'assets', 'index.js')
        );
        const styleUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._extensionUri, 'webview-ui', 'dist', 'assets', 'index.css')
        );

        // Use a nonce to only allow specific scripts
        const nonce = getNonce();

        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="Content-Security-Policy"
          content="default-src 'none';
                   style-src ${{webview.cspSource}} 'unsafe-inline';
                   script-src 'nonce-${{nonce}}';
                   img-src ${{webview.cspSource}} https:;">
    <link href="${{styleUri}}" rel="stylesheet">
    <title>{title}</title>
    <style>
        html, body {{ margin: 0; padding: 0; height: 100%; overflow: hidden; }}
        #app {{ height: 100%; display: flex; flex-direction: column; }}
        #app > div {{ flex: 1; min-height: 0; }}
    </style>
</head>
<body>
    <div id="app"></div>
    <script nonce="${{nonce}}">
        // VSCode API stub for the webview
        const vscode = acquireVsCodeApi();

        // Bridge for AURA messaging
        window.auraPostMessage = function(type, data) {{
            vscode.postMessage({{ type, data }});
        }};

        window.auraOnMessage = function(handler) {{
            window.addEventListener('message', (event) => {{
                const message = event.data;
                handler(message.type, message.data);
            }});
        }};
    </script>
    <script type="module" nonce="${{nonce}}" src="${{scriptUri}}"></script>
</body>
</html>`;
    }}
}}

function getWebviewOptions(extensionUri: vscode.Uri): vscode.WebviewOptions {{
    return {{
        enableScripts: true,
        localResourceRoots: [
            vscode.Uri.joinPath(extensionUri, 'webview-ui'),
            vscode.Uri.joinPath(extensionUri, 'media'),
        ],
    }};
}}

function getNonce(): string {{
    let text = '';
    const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    for (let i = 0; i < 32; i++) {{
        text += possible.charAt(Math.floor(Math.random() * possible.length));
    }}
    return text;
}}
"#
    )
}

fn generate_webview_index_html(config: &VscodeConfig) -> String {
    let title = &config.title;
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
</head>
<body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
</body>
</html>
"#
    )
}

fn generate_webview_main_ts() -> String {
    r#"import { createApp } from 'vue';
import App from './App.vue';
import './assets/index.css';

// VSCode webview API bridge
declare global {
    interface Window {
        auraPostMessage(type: string, data?: any): void;
        auraOnMessage(handler: (type: string, data: any) => void): void;
    }
}

const app = createApp(App);
app.mount('#app');
"#.to_string()
}

fn generate_webview_package_json(name: &str) -> String {
    let kebab = to_kebab_case(name);
    format!(
        r#"{{
    "name": "{kebab}-webview",
    "version": "0.1.0",
    "private": true,
    "type": "module",
    "scripts": {{
        "dev": "vite",
        "build": "vite build",
        "preview": "vite preview"
    }},
    "dependencies": {{
        "vue": "^3.4.0",
        "clsx": "^2.1.0",
        "tailwind-merge": "^2.2.0",
        "class-variance-authority": "^0.7.0"
    }},
    "devDependencies": {{
        "@vitejs/plugin-vue": "^5.0.0",
        "vite": "^5.0.0",
        "typescript": "^5.3.0",
        "vue-tsc": "^2.0.0",
        "tailwindcss": "^3.4.0",
        "tailwindcss-animate": "^1.0.7",
        "autoprefixer": "^10.4.0",
        "postcss": "^8.4.0"
    }}
}}
"#
    )
}

fn generate_tsconfig() -> String {
    r#"{
    "compilerOptions": {
        "module": "commonjs",
        "target": "ES2020",
        "outDir": "dist",
        "lib": ["ES2020"],
        "sourceMap": true,
        "rootDir": "src",
        "strict": true,
        "esModuleInterop": true,
        "skipLibCheck": true,
        "forceConsistentCasingInFileNames": true
    },
    "exclude": ["node_modules", ".vscode-test", "webview-ui"]
}
"#.to_string()
}

fn generate_webpack_config() -> String {
    r#"//@ts-check
'use strict';

const path = require('path');

/** @type {import('webpack').Configuration} */
const config = {
    target: 'node',
    mode: 'none',
    entry: './src/extension.ts',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'extension.js',
        libraryTarget: 'commonjs2',
    },
    externals: {
        vscode: 'commonjs vscode',
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    module: {
        rules: [
            {
                test: /\.ts$/,
                exclude: /node_modules/,
                use: [
                    {
                        loader: 'ts-loader',
                    },
                ],
            },
        ],
    },
    devtool: 'nosources-source-map',
    infrastructureLogging: {
        level: 'log',
    },
};

module.exports = config;
"#.to_string()
}

fn generate_vscodeignore() -> String {
    r#".vscode/**
.vscode-test/**
src/**
webview-ui/src/**
webview-ui/node_modules/**
webview-ui/index.html**
node_modules/**
.gitignore
tsconfig.json
webpack.config.js
**/*.map
**/*.ts
"#
    .to_string()
}

fn generate_webview_vite_config() -> String {
    r#"import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
    plugins: [vue()],
    build: {
        outDir: 'dist',
        assetsDir: 'assets',
        rollupOptions: {
            output: {
                entryFileNames: 'assets/index.js',
                chunkFileNames: 'assets/[name].js',
                assetFileNames: 'assets/[name].[ext]',
            },
        },
    },
});
"#.to_string()
}

fn generate_webview_tsconfig() -> String {
    r#"{
    "compilerOptions": {
        "target": "ES2020",
        "module": "ESNext",
        "moduleResolution": "bundler",
        "strict": true,
        "jsx": "preserve",
        "resolveJsonModule": true,
        "isolatedModules": true,
        "esModuleInterop": true,
        "lib": ["ES2020", "DOM"],
        "skipLibCheck": true,
        "noEmit": true,
        "paths": {
            "@/*": ["./src/*"]
        }
    },
    "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.vue"],
    "exclude": ["node_modules"]
}
"#.to_string()
}

fn generate_webview_env_dts() -> String {
    r#"/// <reference types="vite/client" />

declare module '*.vue' {
    import type { DefineComponent } from 'vue';
    const component: DefineComponent<{}, {}, any>;
    export default component;
}
"#.to_string()
}

fn generate_tailwind_config() -> String {
    VueGenerator::generate_tailwind_config()
}

fn generate_postcss_config() -> String {
    r#"module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
"#.to_string()
}

fn generate_base_css() -> String {
    VueGenerator::generate_base_css()
}

fn generate_launch_json() -> String {
    r#"{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Run Extension",
            "type": "extensionHost",
            "request": "launch",
            "args": [
                "--extensionDevelopmentPath=${workspaceFolder}"
            ],
            "outFiles": [
                "${workspaceFolder}/dist/**/*.js"
            ],
            "preLaunchTask": "${defaultBuildTask}"
        }
    ]
}
"#.to_string()
}

fn generate_tasks_json() -> String {
    r#"{
    "version": "2.0.0",
    "tasks": [
        {
            "type": "npm",
            "script": "build",
            "group": {
                "kind": "build",
                "isDefault": true
            }
        }
    ]
}
"#.to_string()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compile a .at file to Vue component code.
/// Returns (vue_code, widget_names).
fn compile_at_to_vue(at_path: &Path) -> AutoResult<(String, Vec<String>)> {
    use auto_lang::Parser;
    use auto_lang::aura::extract_widget_from_decl;
    use auto_lang::session::CompilerSession;

    let code = fs::read_to_string(at_path)
        .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;

    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(code.as_str());
    parser = parser.with_session(session);

    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let mut widgets = Vec::new();
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widgets found".to_string().into());
    }

    let mut generator = VueGenerator::new()
        .with_mode(auto_lang::ui_gen::VueMode::Shadcn);
    let vue_code = generator
        .generate(&widgets[0])
        .map_err(|e| e.to_string())?;

    let names: Vec<String> = widgets.iter().map(|w| w.name.clone()).collect();
    Ok((vue_code, names))
}

/// Parse project name from pac.at content.
fn parse_pac_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_matches('"').trim_matches('\'');
                let value = value.trim_end_matches(',');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

/// Parse a "key: value" or "key value" line, stripping quotes and commas.
fn parse_kv(line: &str) -> Option<(&str, String)> {
    // Try "key: value" first
    if let Some(colon_pos) = line.find(':') {
        let key = line[..colon_pos].trim();
        let value = line[colon_pos + 1..].trim();
        let value = value.trim_end_matches(',');
        // Strip surrounding quotes
        let value = if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            &value[1..value.len() - 1]
        } else {
            value
        };
        if !key.is_empty() && !value.is_empty() {
            return Some((key, value.to_string()));
        }
    }
    None
}

/// Convert CamelCase or space-separated name to kebab-case.
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '_' {
            result.push('-');
        } else {
            result.push(c);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("MyApp"), "my-app");
        assert_eq!(to_kebab_case("hello_world"), "hello-world");
        assert_eq!(to_kebab_case("Hello World"), "hello--world");
        assert_eq!(to_kebab_case("app"), "app");
        assert_eq!(to_kebab_case("MyVSCodeTool"), "my-v-s-code-tool");
    }

    #[test]
    fn test_parse_kv() {
        assert_eq!(
            parse_kv("panel: sidebar"),
            Some(("panel", "sidebar".to_string()))
        );
        assert_eq!(
            parse_kv(r#"command: "myTool.open""#),
            Some(("command", "myTool.open".to_string()))
        );
        assert_eq!(
            parse_kv(r#"title: "My Tool","#),
            Some(("title", "My Tool".to_string()))
        );
    }

    #[test]
    fn test_vscode_config_defaults() {
        let config = VscodeConfig::with_defaults("MyApp");
        assert_eq!(config.panel, "sidebar");
        assert_eq!(config.command, "my-app.open");
        assert_eq!(config.title, "MyApp");
        assert!(config.icon.is_none());
    }

    #[test]
    fn test_vscode_config_parse() {
        let pac = r#"
name: "TestProject"
backend: ["vue", "vscode"]

vscode {
    panel: editor
    command: "test.open"
    title: "Test Tool"
    icon: "icon.png"
}
"#;
        let config = VscodeConfig::parse_from_pac(pac, "TestProject");
        assert_eq!(config.panel, "editor");
        assert_eq!(config.command, "test.open");
        assert_eq!(config.title, "Test Tool");
        assert_eq!(config.icon, Some("icon.png".to_string()));
    }

    #[test]
    fn test_vscode_config_no_block() {
        let pac = r#"
name: "TestProject"
backend: ["vscode"]
"#;
        let config = VscodeConfig::parse_from_pac(pac, "TestProject");
        assert_eq!(config.panel, "sidebar");
        assert_eq!(config.command, "test-project.open");
        assert_eq!(config.title, "TestProject");
    }

    #[test]
    fn test_parse_pac_name() {
        let content = r#"name: "MyExtension"
backend: ["vscode"]
"#;
        assert_eq!(parse_pac_name(content), Some("MyExtension".to_string()));
    }

    #[test]
    fn test_generate_package_json_basic() {
        let config = VscodeConfig::with_defaults("MyTool");
        let json = generate_package_json("MyTool", &config);
        assert!(json.contains(r#""command": "my-tool.open""#));
        assert!(json.contains(r#""build""#));
        assert!(json.contains(r#"webview:build"#));
        assert!(!json.contains(r#""dependencies""#)); // no vscode npm dep
    }
}
