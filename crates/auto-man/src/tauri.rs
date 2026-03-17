//! Tauri project generation and build utilities
//!
//! This module provides the complete Tauri + Vue + shadcn-vue project workflow:
//! 1. Generate Vue project structure
//! 2. Generate Tauri backend structure
//! 3. npm install
//! 4. Install shadcn-vue components
//! 5. Run tauri dev (which runs both Vue dev server and Tauri backend)

use std::path::Path;
use std::process::{Command, Stdio};

use colored::Colorize;

use crate::AutoResult;

/// Run Tauri dev server (full workflow: generate Vue, install deps, run)
///
/// Steps:
/// 1. Generate Vue project structure if not exists
/// 2. npm install
/// 3. Install shadcn-vue components
/// 4. Initialize Tauri if not exists
/// 5. npm run tauri dev (no build needed for dev mode)
pub fn run_tauri_project(root_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    println!("{}", "Running Tauri dev server (backend: tauri)".bright_cyan());

    // Load Vue project context
    let project = crate::vue::VueProject::from_workspace(root_dir)?;

    // Check if Tauri is already initialized
    let vue_dir = root_dir.join("vue");
    let tauri_dir = vue_dir.join("src-tauri");
    let tauri_exists = tauri_dir.exists();

    // Step 1: Generate project structure if not exists
    let total_steps = if project.exists() {
        if tauri_exists { 4 } else { 5 }
    } else {
        if tauri_exists { 5 } else { 6 }
    };
    let mut current_step = 0;

    if !project.exists() {
        current_step += 1;
        println!();
        println!("▶ Step {}/{}: Generating Vue project...", current_step, total_steps);
        project.generate()?;
    }

    // Step 2: npm install
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing dependencies...", current_step, total_steps);
    project.npm_install()?;

    // Step 3: Install shadcn-vue components
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing shadcn-vue components...", current_step, total_steps);
    project.install_shadcn_components()?;

    // Step 4: Initialize Tauri if not exists
    if !tauri_exists {
        current_step += 1;
        println!();
        println!("▶ Step {}/{}: Initializing Tauri...", current_step, total_steps);
        init_tauri(&vue_dir)?;
    }

    // Step 5: Run Tauri dev (no build needed - tauri dev handles it)
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Starting Tauri dev server...", current_step, total_steps);
    run_tauri_dev(root_dir)?;

    Ok(())
}

/// Initialize Tauri in the Vue project
fn init_tauri(vue_dir: &Path) -> AutoResult<()> {
    // Step 1: Install Tauri CLI as dev dependency
    println!("  Installing @tauri-apps/cli...");
    let install_args = vec!["install", "--save-dev", "@tauri-apps/cli@^2"];
    run_command_live("npm", &install_args, vue_dir)?;
    println!("  ✓ @tauri-apps/cli installed");

    // Step 2: Install Tauri API as dependency
    println!("  Installing @tauri-apps/api...");
    let api_args = vec!["install", "@tauri-apps/api@^2"];
    run_command_live("npm", &api_args, vue_dir)?;
    println!("  ✓ @tauri-apps/api installed");

    // Step 3: Initialize Tauri using npx (directly runs the CLI)
    // Note: Vite dev server runs on port 3000 (configured in vue.rs)
    // Use --ci flag for non-interactive mode
    // On Windows, we need to be careful with argument quoting
    #[cfg(windows)]
    let init_args = vec![
        "tauri", "init",
        "--ci",
        "--app-name", "App",
        "--window-title", "App",
        "--dev-url", "http://localhost:3000",
        "--before-dev-command", "npm run dev",
        "--frontend-dist", "../dist",
    ];

    #[cfg(not(windows))]
    let init_args = vec![
        "tauri", "init",
        "--ci",
        "--app-name", "App",
        "--window-title", "App",
        "--dev-url", "http://localhost:3000",
        "--before-dev-command", "\"npm run dev\"",
        "--frontend-dist", "../dist",
    ];

    println!("  Running: npx {}", init_args.join(" "));

    run_command_live("npx", &init_args, vue_dir)?;

    // Step 4: Add empty [workspace] to src-tauri/Cargo.toml to exclude from root workspace
    let tauri_cargo_toml = vue_dir.join("src-tauri").join("Cargo.toml");
    if tauri_cargo_toml.exists() {
        let content = std::fs::read_to_string(&tauri_cargo_toml)
            .map_err(|e| format!("Failed to read src-tauri/Cargo.toml: {}", e))?;

        // Only add if not already present
        if !content.contains("[workspace]") {
            let updated = format!("{}\n\n# Exclude from root workspace\n[workspace]\n", content);
            std::fs::write(&tauri_cargo_toml, updated)
                .map_err(|e| format!("Failed to write src-tauri/Cargo.toml: {}", e))?;
            println!("  ✓ Added [workspace] to exclude from root workspace");
        }
    }

    println!("  ✓ Tauri initialized");

    Ok(())
}

/// Run a command with live output (inherits stdout/stderr)
fn run_command_live(cmd: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let status = {
        // On Windows, use cmd.exe /C to properly resolve npm/npx from PATH
        let mut full_args = vec!["/C", cmd];
        full_args.extend(args);
        Command::new("cmd")
            .args(&full_args)
            .current_dir(cwd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to run {}: {}", cmd, e))?
    };

    #[cfg(not(windows))]
    let status = {
        Command::new(cmd)
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to run {}: {}", cmd, e))?
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with code {:?}", cmd, status.code()))
    }
}

/// Run npx tauri dev
fn run_tauri_dev(root_dir: &Path) -> AutoResult<()> {
    let vue_dir = root_dir.join("vue");

    if !vue_dir.exists() {
        return Err("Vue project directory not found. Please run 'auto gen' first.".into());
    }

    println!();
    println!("{} {}", "Starting Tauri dev...".bright_green(), "(this may take a while for first build)".bright_black());
    println!();

    // Use npx tauri dev instead of npm run tauri dev
    // (tauri CLI is installed as dev dependency, npx can run it directly)
    let args = vec!["tauri", "dev"];
    run_command_live("npx", &args, &vue_dir)?;

    Ok(())
}
