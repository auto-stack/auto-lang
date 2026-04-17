//! Tauri project generation and build utilities
//!
//! This module provides the complete Tauri + Vue + shadcn-vue project workflow:
//! 1. Generate Vue project structure
//! 2. Generate Tauri backend structure
//! 3. Package install (bun/npm/pnpm)
//! 4. Install shadcn-vue components
//! 5. Run tauri dev (which runs both Vue dev server and Tauri backend)
//!
//! Plan 151: Tauri IPC Mode - When backend: { front: "tauri", back: "rust" },
//! generates a complete Rust backend crate from AutoLang source files.

use std::path::Path;

use colored::Colorize;

use crate::AutoResult;

/// Run Tauri dev server (full workflow: generate Vue, install deps, run)
///
/// Steps:
/// 1. Generate Vue project structure if not exists
/// 2. Generate API client code (if api.at exists)
/// 3. Generate Rust backend crate (Plan 151: if backend: { front: "tauri", back: "rust" })
/// 4. npm install
/// 5. Install shadcn-vue components
/// 6. Initialize Tauri if not exists
/// 7. npm run tauri dev (no build needed for dev mode)
pub fn run_tauri_project(root_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    println!("{}", "Running Tauri dev server (backend: tauri)".bright_cyan());

    // Load Vue project context
    let project = crate::vue::VueProject::from_workspace(root_dir)?;

    // Check if Tauri is already initialized
    let vue_dir = root_dir.join("gen").join("vue");
    let tauri_dir = vue_dir.join("src-tauri");
    let tauri_exists = tauri_dir.exists();

    // Step 1: Generate project structure if not exists
    let total_steps = if project.exists() {
        if tauri_exists { 5 } else { 6 }
    } else {
        if tauri_exists { 6 } else { 7 }
    };
    let mut current_step = 0;

    if !project.exists() {
        current_step += 1;
        println!();
        println!("▶ Step {}/{}: Generating Vue project...", current_step, total_steps);
        project.generate()?;
    }

    // Step 2: Generate API client code (if api.at exists)
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Generating API client...", current_step, total_steps);
    if let Err(e) = crate::api_gen::generate_api(root_dir, "tauri") {
        // API generation is optional - only warn on failure
        println!("  ⚠ API generation skipped: {}", e);
    }

    // Plan 151: Step 2.5 - Generate Rust backend crate (if backend: { front: "tauri", back: "rust" })
    let rust_dir = root_dir.join("gen").join("rust");
    if rust_dir.exists() {
        current_step += 1;
        println!();
        println!("▶ Step {}/{}: Generating Rust backend crate (Plan 151)...", current_step, total_steps);
        if let Err(e) = crate::tauri_backend::generate_tauri_backend(root_dir) {
            println!("  ⚠ Rust backend generation skipped: {}", e);
        } else {
            // Update src-tauri/Cargo.toml to depend on ../../rust
            if tauri_dir.exists() {
                update_tauri_cargo_toml(&tauri_dir)?;
            }
        }
    } else {
        // No rust/ directory, skip backend generation
    }

    // Step 3: npm install
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing dependencies...", current_step, total_steps);
    project.npm_install()?;

    // Step 4: Install shadcn-vue components
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Installing shadcn-vue components...", current_step, total_steps);
    project.install_shadcn_components()?;

    // Step 5: Initialize Tauri if not exists
    if !tauri_exists {
        current_step += 1;
        println!();
        println!("▶ Step {}/{}: Initializing Tauri...", current_step, total_steps);
        init_tauri(&vue_dir)?;

        // Plan 151: Update src-tauri/Cargo.toml to depend on ../../rust
        let rust_dir = root_dir.join("gen").join("rust");
        if rust_dir.exists() {
            update_tauri_cargo_toml(&tauri_dir)?;
        }
    }

    // Step 6: Run Tauri dev (no build needed - tauri dev handles it)
    current_step += 1;
    println!();
    println!("▶ Step {}/{}: Starting Tauri dev server...", current_step, total_steps);
    run_tauri_dev(root_dir)?;

    Ok(())
}

/// Plan 151: Update src-tauri/Cargo.toml to depend on ../../rust
///
/// This modifies the Tauri project's Cargo.toml to include the generated
/// Rust backend crate as a dependency.
fn update_tauri_cargo_toml(tauri_dir: &Path) -> AutoResult<()> {
    let cargo_toml_path = tauri_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&cargo_toml_path)
        .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

    // Check if already has the dependency
    if content.contains("../../rust") {
        return Ok(());
    }

    // Read the rust/Cargo.toml to get the actual package name
    let rust_cargo_path = tauri_dir.join("../../rust/Cargo.toml");
    let package_name = if rust_cargo_path.exists() {
        let rust_content = std::fs::read_to_string(&rust_cargo_path)
            .map_err(|e| format!("Failed to read rust/Cargo.toml: {}", e))?;
        // Extract package name from Cargo.toml
        rust_content
            .lines()
            .find(|line| line.trim().starts_with("name = "))
            .and_then(|line| {
                line.split('=')
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').to_string())
            })
            .unwrap_or_else(|| "api-server".to_string())
    } else {
        "api-server".to_string()
    };

    // Add the dependency to [dependencies] section
    let updated = if content.contains("[dependencies]") {
        content.replace(
            "[dependencies]",
            &format!("[dependencies]\n{} = {{ path = \"../../rust\" }}", package_name)
        )
    } else {
        format!(r#"[dependencies]
{} = {{ path = "../../rust" }}"#, package_name)
    };

    std::fs::write(&cargo_toml_path, updated)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    println!("  ✓ Updated src-tauri/Cargo.toml to depend on ../../rust (package: {})", package_name);
    Ok(())
}

/// Initialize Tauri in the Vue project
fn init_tauri(vue_dir: &Path) -> AutoResult<()> {
    // Step 1: Install Tauri CLI as dev dependency
    println!("  Installing @tauri-apps/cli...");
    crate::pkg::add_packages(&["@tauri-apps/cli@^2"], true, vue_dir)
        .map_err(|e| format!("Failed to install @tauri-apps/cli: {}", e))?;
    println!("  ✓ @tauri-apps/cli installed");

    // Step 2: Install Tauri API as dependency
    println!("  Installing @tauri-apps/api...");
    crate::pkg::add_packages(&["@tauri-apps/api@^2"], false, vue_dir)
        .map_err(|e| format!("Failed to install @tauri-apps/api: {}", e))?;
    println!("  ✓ @tauri-apps/api installed");

    // Step 3: Install cross-env for cross-platform environment variables
    println!("  Installing cross-env...");
    crate::pkg::add_packages(&["cross-env"], true, vue_dir)
        .map_err(|e| format!("Failed to install cross-env: {}", e))?;
    println!("  ✓ cross-env installed");

    // Step 4: Add dev:tauri script to package.json if not present
    let package_json_path = vue_dir.join("package.json");
    if package_json_path.exists() {
        let content = std::fs::read_to_string(&package_json_path)
            .map_err(|e| format!("Failed to read package.json: {}", e))?;

        // Check if dev:tauri script exists
        if !content.contains("\"dev:tauri\"") {
            // Add dev:tauri script by finding the scripts section and adding it
            // This is a simple string replacement approach
            if let Some(pos) = content.find("\"dev\":") {
                // Find the end of the dev script line
                let before = &content[..pos];
                let after = &content[pos..];

                // Find where to insert (after the "dev" script line ends)
                if let Some(line_end) = after.find('\n') {
                    let dev_line = &after[..line_end];
                    let rest = &after[line_end..];

                    let updated = format!(
                        "{}{}\n    \"dev:tauri\": \"cross-env TAURI_ENV=1 vite\",{}",
                        before, dev_line, rest
                    );

                    std::fs::write(&package_json_path, updated)
                        .map_err(|e| format!("Failed to write package.json: {}", e))?;
                    println!("  ✓ Added dev:tauri script to package.json");
                }
            }
        }
    }

    // Step 5: Initialize Tauri using the package manager's exec command
    // Note: Vite dev server runs on port 3000 (configured in vue.rs)
    // Use --ci flag for non-interactive mode
    // Use --force to overwrite existing configuration
    let pm = crate::pkg::display_name();
    let before_dev_cmd = format!("{} run dev:tauri", pm);
    let init_args: Vec<&str> = vec![
        "init",
        "--ci",
        "--force",
        "--app-name", "App",
        "--window-title", "App",
        "--dev-url", "http://localhost:3000",
        "--before-dev-command", &before_dev_cmd,
        "--frontend-dist", "../dist",
    ];

    println!("  Running: {} tauri {}", crate::pkg::exec_cmd(), init_args.join(" "));

    crate::pkg::exec("tauri", &init_args, vue_dir)
        .map_err(|e| format!("Tauri init failed: {}", e))?;

    // Step 4.5: Update tauri.conf.json to use port 3000 (tauri init defaults to 5173)
    let tauri_conf = vue_dir.join("src-tauri").join("tauri.conf.json");
    if tauri_conf.exists() {
        let content = std::fs::read_to_string(&tauri_conf)
            .map_err(|e| format!("Failed to read tauri.conf.json: {}", e))?;
        // Replace default port 5173 with 3000
        let updated = content.replace("\"http://localhost:5173\"", "\"http://localhost:3000\"");
        std::fs::write(&tauri_conf, updated)
            .map_err(|e| format!("Failed to write tauri.conf.json: {}", e))?;
        println!("  ✓ Updated tauri.conf.json to use port 3000");
    }

    // Step 5: Add empty [workspace] to src-tauri/Cargo.toml to exclude from root workspace
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

/// Run tauri dev
fn run_tauri_dev(root_dir: &Path) -> AutoResult<()> {
    let vue_dir = root_dir.join("gen").join("vue");

    if !vue_dir.exists() {
        return Err("Vue project directory not found. Please run 'auto gen' first.".into());
    }

    println!();
    println!("{} {}", "Starting Tauri dev...".bright_green(), "(this may take a while for first build)".bright_black());
    println!();

    // Use npx tauri dev instead of npm run tauri dev
    // (tauri CLI is installed as dev dependency, exec can run it directly)
    crate::pkg::exec("tauri", &["dev"], &vue_dir)
        .map_err(|e| format!("Tauri dev failed: {}", e))?;

    Ok(())
}
