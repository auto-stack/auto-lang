//! VSCode Extension project generation utilities
//!
//! This module generates a VSCode extension scaffold from AURA widgets.
//! The extension loads its webview UI from the sibling `gen/vue/dist/` directory,
//! reusing the Vue build output instead of maintaining a separate webview project.
//!
//! Architecture:
//! - pac.at config -> vscode.rs -> package.json, extension.ts, AppPanel.ts
//! - gen/vue/dist/ -> shared webview assets (built by vue.rs)
//!
//! The generated extension uses the VSCode Webview API to display the Vue app
//! in a sidebar or editor panel.

use std::fs;
use std::path::Path;

use colored::Colorize;

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
        let front_dir = if root_dir.join("src").join("front").exists() {
            root_dir.join("src").join("front")
        } else if root_dir.join("source").join("front").exists() {
            root_dir.join("source").join("front")
        } else if root_dir.join("front").exists() {
            root_dir.join("front")
        } else {
            root_dir.join("src").join("front")
        };

        let output_dir = root_dir.join("gen").join("vscode");

        Ok(Self {
            root_dir: root_dir.to_path_buf(),
            output_dir,
            name,
            front_dir,
            config,
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
        let media_dir = self.output_dir.join("media");
        let vscode_dir = self.output_dir.join(".vscode");

        fs::create_dir_all(&src_dir)
            .map_err(|e| format!("Failed to create src/panels: {}", e))?;
        fs::create_dir_all(&media_dir)
            .map_err(|e| format!("Failed to create media: {}", e))?;
        fs::create_dir_all(&vscode_dir)
            .map_err(|e| format!("Failed to create .vscode: {}", e))?;

        println!("{}", "  Created directory structure".bright_green());

        // Generate all files
        self.write_package_json()?;
        self.write_extension_ts()?;
        self.write_app_panel_ts()?;
        self.write_tsconfig()?;
        self.write_webpack_config()?;
        self.write_vscodeignore()?;
        self.write_launch_json()?;
        self.write_tasks_json()?;

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

    // Step 1: Ensure gen/vue is built (webview prerequisite)
    let vue_dir = root_dir.join("gen").join("vue");
    let vue_dist = vue_dir.join("dist");

    if !vue_dist.join("assets").join("index.js").exists() {
        println!();
        println!(
            "{}",
            "  Step 1: Building Vue project (webview dependency)...".bright_cyan()
        );

        // Generate Vue project first
        crate::vue::build_vue_project(root_dir)?;

        // Install Vue dependencies
        #[cfg(windows)]
        let npm_install = std::process::Command::new("cmd")
            .args(&["/C", "npm", "install"])
            .current_dir(&vue_dir)
            .status();

        #[cfg(not(windows))]
        let npm_install = std::process::Command::new("npm")
            .args(&["install"])
            .current_dir(&vue_dir)
            .status();

        match npm_install {
            Ok(status) if status.success() => {
                println!("  {} Vue dependencies installed", "OK".bright_green());
            }
            _ => {
                println!(
                    "  {} Failed to install Vue dependencies",
                    "Warning:".bright_yellow()
                );
            }
        }

        // Build Vue project
        #[cfg(windows)]
        let npm_build = std::process::Command::new("cmd")
            .args(&["/C", "npm", "run", "build"])
            .current_dir(&vue_dir)
            .status();

        #[cfg(not(windows))]
        let npm_build = std::process::Command::new("npm")
            .args(&["run", "build"])
            .current_dir(&vue_dir)
            .status();

        match npm_build {
            Ok(status) if status.success() => {
                println!("  {} Vue project built", "OK".bright_green());
            }
            _ => {
                return Err("Vue build failed — cannot generate VSCode extension without webview assets".into());
            }
        }
    } else {
        println!();
        println!(
            "  {} Vue dist already exists, skipping Vue build",
            "OK".bright_green()
        );
    }

    // Step 2: Generate extension scaffold
    println!();
    println!(
        "{}",
        "  Step 2: Generating VSCode extension code...".bright_cyan()
    );
    generate_vscode_project(root_dir, None, false)?;

    let vscode_dir = root_dir.join("gen").join("vscode");

    // Step 3: Check for npm
    println!();
    println!(
        "{}",
        "  Step 3: Checking build tools...".bright_cyan()
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

    // Step 4: Install extension dependencies
    println!("  {} npm found", "OK".bright_green());
    println!();
    println!(
        "{}",
        "  Step 4: Installing dependencies...".bright_cyan()
    );

    // Install extension dependencies if needed
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

    // Step 5: Compile extension
    println!();
    println!(
        "{}",
        "  Step 5: Compiling extension...".bright_cyan()
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

    // Step 6: Copy Vue dist into extension directory for webview access
    // VSCode webviews cannot load resources from outside the extension dir,
    // so we copy gen/vue/dist/ → gen/vscode/webview/ at build time.
    // Symlinks/junctions don't work — the service worker resolves the real
    // path and rejects it as outside the extension.
    let vue_dist_src = root_dir.join("gen").join("vue").join("dist");
    let vscode_webview_dst = vscode_dir.join("webview");

    if vue_dist_src.exists() {
        // Remove existing copy if present
        if vscode_webview_dst.exists() {
            let _ = fs::remove_dir_all(&vscode_webview_dst);
        }
        copy_dir_recursive(&vue_dist_src, &vscode_webview_dst)?;
        println!("  {} Vue dist copied into extension", "OK".bright_green());
    }

    Ok(())
}

/// Run the VSCode extension in development mode (auto run command).
pub fn run_vscode_project(root_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    println!(
        "{}",
        "Running VSCode extension project".bright_cyan()
    );

    // Step 1: Build first (generates + installs deps + compiles)
    build_vscode_project(root_dir)?;

    let vscode_dir = root_dir.join("gen").join("vscode");

    // Step 2: Open VSCode with extension loaded
    println!();
    println!(
        "{}",
        "  Opening VSCode with extension loaded...".bright_cyan()
    );

    let vscode_path = vscode_dir.to_string_lossy().to_string();

    // On Windows, `code` is a .cmd script so must be invoked via cmd /C
    #[cfg(windows)]
    let result = std::process::Command::new("cmd")
        .args(&["/C", "code", "--extensionDevelopmentPath", &vscode_path])
        .status();

    #[cfg(not(windows))]
    let result = std::process::Command::new("code")
        .args(&["--extensionDevelopmentPath", &vscode_path])
        .status();

    match result {
        Ok(status) if status.success() => {
            println!(
                "  {} VSCode opened. Click the globe icon or run command to open the panel.",
                "OK".bright_green()
            );
        }
        _ => {
            println!(
                "  {} Could not auto-open VSCode. Run manually:",
                "Warning:".bright_yellow()
            );
            println!(
                "  {}",
                format!("code --extensionDevelopmentPath={}", vscode_dir.display())
                    .bright_cyan()
            );
        }
    }

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
                "title": "Open {title}",
                "icon": "$(globe)"
            }}
        ],
        "menus": {{
            "editor/title": [
                {{
                    "command": "{command}",
                    "group": "navigation"
                }}
            ]
        }}
    }},
    "scripts": {{
        "vscode:prepublish": "npm run compile",
        "build": "npm install && npm run compile",
        "compile": "webpack --mode production",
        "watch": "webpack --mode development --watch"
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
        // Get the local path to the shared Vue build output (linked as webview/)
        const scriptUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._extensionUri, 'webview', 'assets', 'index.js')
        );
        const styleUri = webview.asWebviewUri(
            vscode.Uri.joinPath(this._extensionUri, 'webview', 'assets', 'index.css')
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
        #app {{ height: 100%; }}
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
            vscode.Uri.joinPath(extensionUri, 'webview'),
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
    "exclude": ["node_modules", ".vscode-test"]
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
node_modules/**
.gitignore
tsconfig.json
webpack.config.js
**/*.map
**/*.ts
"#
    .to_string()
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

/// Recursively copy a directory. Used as fallback when symlink fails.
fn copy_dir_recursive(src: &Path, dst: &Path) -> AutoResult<()> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create {}: {}", dst.display(), e))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
        }
    }
    Ok(())
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
        assert!(json.contains(r#"npm install && npm run compile"#));
        assert!(!json.contains(r#"webview"#)); // no webview scripts
        assert!(!json.contains(r#""dependencies""#)); // no vscode npm dep
    }
}
