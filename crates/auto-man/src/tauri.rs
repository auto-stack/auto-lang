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

/// Run Tauri dev server (full workflow: generate Vue, generate Tauri, install, run)
///
/// Steps:
/// 1. Generate Vue project structure if not exists
/// 2. Generate Tauri backend if not exists
/// 3. npm install
/// 4. Install shadcn-vue components
/// 5. npm run tauri dev
pub fn run_tauri_project(root_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    println!("{}", "Running Tauri dev server (backend: tauri)".bright_cyan());

    // First, ensure Vue project is generated and ready
    println!();
    println!("{}", "▶ Step 1/2: Preparing Vue frontend...".bright_white());
    crate::vue::build_vue_project(root_dir)?;

    // Then run Tauri dev
    println!();
    println!("{}", "▶ Step 2/2: Starting Tauri dev server...".bright_white());
    run_tauri_dev(root_dir)?;

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

/// Run npm run tauri dev
fn run_tauri_dev(root_dir: &Path) -> AutoResult<()> {
    let vue_dir = root_dir.join("vue");

    if !vue_dir.exists() {
        return Err("Vue project directory not found. Please run 'auto gen' first.".into());
    }

    println!();
    println!("{} {}", "Starting Tauri dev...".bright_green(), "(this may take a while for first build)".bright_black());
    println!();

    // Run npm run tauri dev in the vue directory
    let args = vec!["run", "tauri", "dev"];
    run_command_live("npm", &args, &vue_dir)?;

    Ok(())
}
