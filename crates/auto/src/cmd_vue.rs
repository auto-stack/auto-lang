//! `auto vue` command - Generate complete Vue + shadcn-vue project from AURA file
//!
//! Usage:
//!   auto vue input.at -o ./my-app
//!   auto vue input.at -o ./my-app --name MyApp
//!   auto vue                          # If pac.at exists in current directory
//!
//! This command:
//! 1. Checks for pac.at in current directory (workspace mode)
//! 2. If pac.at exists: compiles source/front and source/back
//! 3. If no pac.at: transpiles single .at file (legacy mode)
//! 4. Generates a complete Vite + Vue + TypeScript project
//! 5. Runs npm install
//! 6. Runs npx shadcn-vue add to add components

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use colored::Colorize;
use auto_lang::aura::AuraRoute;
use auto_lang::ui_gen::VueGenerator;
use auto_lang::route::{RouteDiscovery, RouteMerger, RouteDef, RouteSource};

/// Recursively copy a directory and all its contents
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Check if a command exists
fn command_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    let check = Command::new("where").arg(cmd).output();
    #[cfg(not(windows))]
    let check = Command::new("which").arg(cmd).output();

    check.map(|o| o.status.success()).unwrap_or(false)
}

/// Run a command with live output (inherits stdout/stderr)
/// On Windows, uses cmd.exe /C to properly resolve commands in PATH
fn run_command_live(cmd: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    use std::process::Stdio;

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

/// Check if pac.at exists in the current directory
fn find_pac_at() -> Option<std::path::PathBuf> {
    let pac_path = Path::new("pac.at");
    if pac_path.exists() {
        Some(pac_path.to_path_buf())
    } else {
        None
    }
}

/// Generate Vue project from workspace (pac.at mode)
fn generate_workspace_project(
    pac_path: &Path,
    output_dir: Option<&str>,
    no_install: bool,
    yes: bool,
) -> Result<(), String> {
    println!("{}", "─────────────────────────────────".bright_yellow().bold());
    println!("{}", "  AURA Workspace → Vue + shadcn-vue".bright_yellow().bold());
    println!("{}", "─────────────────────────────────".bright_yellow().bold());
    println!();

    // Read pac.at to get workspace structure
    let pac_content = fs::read_to_string(pac_path)
        .map_err(|e| format!("Failed to read pac.at: {}", e))?;

    // Get the directory containing pac.at
    let pac_dir = pac_path.parent()
        .ok_or_else(|| "Cannot determine pac.at directory".to_string())?;

    // Parse workspace paths from pac.at
    let front_rel_path = parse_workspace_path(&pac_content, "front")
        .unwrap_or_else(|| "source/front".to_string());
    let back_rel_path = parse_workspace_path(&pac_content, "back")
        .unwrap_or_else(|| "source/back".to_string());

    // Resolve paths relative to pac.at directory
    let front_dir = pac_dir.join(&front_rel_path);
    let back_dir = pac_dir.join(&back_rel_path);

    println!("{} {}", "Workspace:".bright_cyan(), pac_path.display());
    println!("{} {}", "Front:".bright_cyan(), front_rel_path);
    println!("{} {}", "Back:".bright_cyan(), back_rel_path);
    println!();

    // Check if front directory exists
    if !front_dir.exists() {
        return Err(format!("Front directory '{}' not found", front_dir.display()));
    }

    // Find app.at in front directory
    let app_at = front_dir.join("app.at");
    if !app_at.exists() {
        return Err(format!("Entry file '{}' not found", app_at.display()));
    }

    // Get project name from pac.at
    let project_name = parse_pac_name(&pac_content)
        .unwrap_or_else(|| "aura-app".to_string());

    // Determine output directory (relative to pac_dir/workspace root)
    // Plan 129: Use backend-specific output directory (vue/ instead of dist/)
    let output = output_dir.unwrap_or("vue");
    let output_path = pac_dir.join(output);

    println!("{} {}", "Output:".bright_cyan(), output);
    println!("{} {}", "Name:".bright_cyan(), project_name);
    println!();

    // Create output directory
    fs::create_dir_all(&output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Create src directory structure
    let src_dir = output_path.join("src");
    let components_dir = src_dir.join("components");
    let lib_dir = src_dir.join("lib");
    let assets_dir = src_dir.join("assets");

    fs::create_dir_all(&components_dir)
        .map_err(|e| format!("Failed to create src/components: {}", e))?;
    fs::create_dir_all(&lib_dir)
        .map_err(|e| format!("Failed to create src/lib: {}", e))?;
    fs::create_dir_all(&assets_dir)
        .map_err(|e| format!("Failed to create src/assets: {}", e))?;

    println!("{}", "✓ Created directory structure".bright_green());

    // Note: public folder will be copied in Step 3 during install steps
    let source_public = front_dir.join("public");

    // Compile .at files in front directory
    // Structure: (relative_path, file_stem, vue_code, widget_name)
    // e.g., ("pages/button", "button", "<template>...", "ButtonPage")
    let mut all_components: Vec<(String, String, String, String)> = Vec::new();
    let mut all_shadcn_components = HashSet::new();
    let mut all_routes: Vec<AuraRoute> = Vec::new();

    // Process app.at first
    let app_at = front_dir.join("app.at");
    if app_at.exists() {
        println!("{} {}", "  Compiling:".bright_black(), app_at.display());

        match auto_lang::ui_build_shadcn_with_widgets(app_at.to_str().unwrap(), None) {
            Ok((vue_code, widgets)) => {
                let components = detect_shadcn_components(&vue_code);
                for comp in &components {
                    all_shadcn_components.insert(comp.clone());
                }
                // Extract routes from widgets
                for widget in &widgets {
                    if let Some(ref routes) = widget.routes {
                        all_routes.extend(routes.routes.clone());
                    }
                }
                // Get widget name from first widget (or use "App" as default)
                let widget_name = widgets.first().map(|w| w.name.as_str()).unwrap_or("App");
                all_components.push(("".to_string(), "app".to_string(), vue_code, widget_name.to_string()));
            }
            Err(e) => {
                println!("{} {}", "  Warning: Failed to compile app.at:".bright_yellow(), e);
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
                let file_stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("page");

                println!("{} {}", "  Compiling:".bright_black(), path.display());

                match auto_lang::ui_build_shadcn_with_widgets(path.to_str().unwrap(), None) {
                    Ok((vue_code, widgets)) => {
                        let components = detect_shadcn_components(&vue_code);
                        for comp in &components {
                            all_shadcn_components.insert(comp.clone());
                        }
                        // Extract routes from widgets
                        for widget in &widgets {
                            if let Some(ref routes) = widget.routes {
                                all_routes.extend(routes.routes.clone());
                            }
                        }
                        // Get widget name from first widget (or use file_stem as default)
                        let widget_name = widgets.first().map(|w| w.name.as_str()).unwrap_or(file_stem);
                        all_components.push(("pages".to_string(), file_stem.to_string(), vue_code, widget_name.to_string()));
                    }
                    Err(e) => {
                        println!("{} Failed to compile {}: {}", "  Warning:".bright_yellow(), path.display(), e);
                    }
                }
            }
        }
    }

    let shadcn_components: Vec<String> = all_shadcn_components.into_iter().collect();
    println!("{} {}", "✓ Detected shadcn-vue components:".bright_green(), shadcn_components.join(", "));

    // =================================================================
    // Plan 114: Hybrid Routing - Convention + Config
    // =================================================================

    // Collect config routes from widgets
    let config_routes: Vec<RouteDef> = all_routes
        .iter()
        .map(|r| RouteDef::new(&r.path, &r.module).with_source(RouteSource::Config))
        .collect();

    // Discover convention-based routes from routes/ folder
    let routes_dir = front_dir.join("routes");
    let discovered_routes = if routes_dir.exists() {
        let discovery = RouteDiscovery::new(routes_dir.clone());
        match discovery.discover() {
            Ok(routes) => {
                println!("{} {}", "✓ Discovered routes from routes/ folder:".bright_green(), routes.len());
                for route in &routes {
                    println!("    {} -> {} ({})", route.path.bright_cyan(), route.module, "file".dimmed());
                }
                routes
            }
            Err(e) => {
                println!("{} {}", "  Warning: Failed to discover routes:".bright_yellow(), e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Merge routes (config routes override convention routes)
    let merged_route_defs = RouteMerger::merge(discovered_routes, config_routes);

    // Convert merged RouteDefs back to AuraRoutes for compatibility
    let all_routes: Vec<AuraRoute> = merged_route_defs
        .iter()
        .map(|r| r.to_aura_route())
        .collect();

    // Check if any routes were detected (after merge)
    let has_routes = !all_routes.is_empty();
    if has_routes {
        println!("{} {}", "✓ Final routes (after merge):".bright_green(), all_routes.len());
        for route in &all_routes {
            let source_marker = if merged_route_defs.iter().any(|r| r.path == route.path && r.source == RouteSource::Config) {
                "(config)"
            } else {
                "(file)"
            };
            println!("    {} -> {} {}", route.path.bright_cyan(), route.module, source_marker.dimmed());
        }
    }

    // Generate App.vue from app.at (relative_dir should be empty for app.at)
    let app_vue_code = all_components.iter()
        .find(|(_, name, _, _)| name == "app")
        .map(|(_, _, code, _)| code.clone())
        .ok_or_else(|| "app.at not found or failed to compile".to_string())?;

    // Write project files
    write_project_files(&output_path, &project_name, &app_vue_code, &shadcn_components, has_routes)?;

    // Generate router files if routes detected
    if has_routes {
        let router_dir = output_path.join("src/router");
        fs::create_dir_all(&router_dir)
            .map_err(|e| format!("Failed to create src/router: {}", e))?;

        let router_content = VueGenerator::generate_router_file(&all_routes);
        fs::write(router_dir.join("index.ts"), router_content)
            .map_err(|e| format!("Failed to write router/index.ts: {}", e))?;

        println!("{}", "  Generated src/router/index.ts".bright_green());
    }

    // Write all components to mirror source directory structure
    // components/ -> src/components/
    // pages/ -> src/pages/
    for (relative_dir, name, code, widget_name) in &all_components {
        if name != "app" {
            // Determine output subdirectory
            let output_subdir = if relative_dir.is_empty() {
                components_dir.clone()
            } else if relative_dir == "components" {
                components_dir.clone()
            } else if relative_dir == "pages" {
                let pages_dir = src_dir.join("pages");
                fs::create_dir_all(&pages_dir)
                    .map_err(|e| format!("Failed to create src/pages: {}", e))?;
                pages_dir
            } else if relative_dir.starts_with("components/") {
                // Handle nested paths like components/ui/button
                let sub_path = relative_dir.strip_prefix("components/").unwrap_or(relative_dir);
                let nested_dir = components_dir.join(sub_path);
                fs::create_dir_all(&nested_dir)
                    .map_err(|e| format!("Failed to create {}: {}", nested_dir.display(), e))?;
                nested_dir
            } else if relative_dir.starts_with("pages/") {
                let sub_path = relative_dir.strip_prefix("pages/").unwrap_or(relative_dir);
                let pages_dir = src_dir.join("pages");
                let nested_dir = pages_dir.join(sub_path);
                fs::create_dir_all(&nested_dir)
                    .map_err(|e| format!("Failed to create {}: {}", nested_dir.display(), e))?;
                nested_dir
            } else {
                // Other directories go to components
                let nested_dir = components_dir.join(relative_dir);
                fs::create_dir_all(&nested_dir)
                    .map_err(|e| format!("Failed to create {}: {}", nested_dir.display(), e))?;
                nested_dir
            };

            // Use lowercase file name for pages (Plan 106), widget_name for others
            // pages/card.at -> pages/card.vue (not CardPage.vue)
            let vue_file_name = if relative_dir == "pages" || relative_dir.starts_with("pages/") {
                name  // lowercase file stem (e.g., "card")
            } else {
                widget_name  // widget name (e.g., "CardPage")
            };
            let component_file = output_subdir.join(format!("{}.vue", vue_file_name));
            fs::write(&component_file, code)
                .map_err(|e| format!("Failed to write {}: {}", component_file.display(), e))?;
        }
    }

    println!("{}", "✓ Generated project files".bright_green());

    // Install dependencies if requested
    if !no_install {
        // Pass public folder source to run_install_steps for Step 3 copy
        run_install_steps(&output_path, &shadcn_components, yes, Some(&source_public))?;
    } else {
        // For no-install mode, copy public folder directly
        let dest_public = output_path.join("public");
        if source_public.exists() && source_public.is_dir() {
            copy_dir_all(&source_public, &dest_public)
                .map_err(|e| format!("Failed to copy public folder: {}", e))?;
            println!("{}", "✓ Copied public assets".bright_green());
        }

        println!();
        println!();
        println!("{}", "Project created successfully!".bright_green().bold());
        println!();
        println!("Next steps:");
        println!("  cd {}", output);
        println!("  npm install");
        if !shadcn_components.is_empty() {
            println!("  npx shadcn-vue@latest add {} --yes", shadcn_components.join(" "));
        }
        println!("  npm run dev");
    }

    Ok(())
}

/// Parse workspace path from pac.at content
fn parse_workspace_path(content: &str, key: &str) -> Option<String> {
    // Look for: front: "./source/front" or workspace: { front: "./source/front" }
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{}:", key)) {
            // Extract path from: front: "./source/front"
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                // Remove quotes
                let value = value.trim_matches('"').trim_matches('\'');
                // Remove trailing comma
                let value = value.trim_end_matches(',');
                return Some(value.to_string());
            }
        }
    }
    None
}

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

/// Check if npm dependencies are already installed
fn is_npm_installed(output_path: &Path) -> bool {
    output_path.join("node_modules").exists()
}

/// Check if shadcn-vue components are already installed
fn are_shadcn_components_installed(output_path: &Path, components: &[String]) -> bool {
    // Check if components.json exists (shadcn-vue config file)
    let components_json = output_path.join("components.json");
    if !components_json.exists() {
        return false;
    }

    // Check if all required component files exist
    // shadcn-vue uses folder structure: src/components/ui/{component}/Component.vue
    // or: src/components/ui/{component}.ts for primitives
    for component in components {
        let ui_dir = output_path.join("src/components/ui");

        // Check for folder-based component (e.g., button/Button.vue)
        let component_folder = ui_dir.join(component);
        let pascal_name = component
            .split('-')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        let folder_vue = component_folder.join(format!("{}.vue", pascal_name));
        let folder_index = component_folder.join("index.ts");
        let primitive_ts = ui_dir.join(format!("{}.ts", component));

        if !folder_vue.exists() && !folder_index.exists() && !primitive_ts.exists() {
            return false;
        }
    }
    true
}

/// Run npm install and shadcn-vue add
fn run_install_steps(
    output_path: &Path,
    components: &[String],
    yes: bool,
    public_folder_source: Option<&Path>,
) -> Result<(), String> {
    if !command_exists("npm") {
        println!();
        println!("{}", "⚠ npm not found".bright_yellow());
        println!("Please install Node.js from https://nodejs.org/");
        return Ok(());
    }

    // Detect which steps need to be run
    // Always run npm install to pick up new dependencies (it's fast if already installed)
    let shadcn_needed = !components.is_empty() && !are_shadcn_components_installed(output_path, components);

    // Calculate step numbers dynamically
    let total_steps = 4; // Always 4 steps, but some may be skipped
    let mut current_step = 0;

    // Step 1: npm install (always run to pick up new dependencies)
    println!();
    current_step += 1;
    println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Installing dependencies...", current_step, total_steps).bright_white());

    let npm_install_args = if yes {
        println!("{}", "  Running: npm install -y".bright_black());
        vec!["install", "-y"]
    } else {
        println!("{}", "  Running: npm install".bright_black());
        vec!["install"]
    };

    match run_command_live("npm", &npm_install_args, output_path) {
        Ok(_) => println!("{}", "  ✓ Dependencies installed".bright_green()),
        Err(e) => {
            println!("{} {}", "  ✗ Failed:".bright_red(), e);
            println!("  You may need to run 'npm install' manually.");
        }
    }

    // Step 2: shadcn-vue add (or skip if already installed or not needed)
    current_step += 1;
    if !components.is_empty() {
        if shadcn_needed {
            println!();
            println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Adding shadcn-vue components ({})...", current_step, total_steps, components.join(", ")).bright_white());

            let mut args = if yes {
                println!("{}", format!("  Running: npx --yes shadcn-vue@latest add {} --yes", components.join(" ")).bright_black());
                vec!["--yes", "shadcn-vue@latest", "add"]
            } else {
                println!("{}", format!("  Running: npx shadcn-vue@latest add {} --yes", components.join(" ")).bright_black());
                vec!["shadcn-vue@latest", "add"]
            };
            args.extend(components.iter().map(|s| s.as_str()));
            args.push("--yes");

            match run_command_live("npx", &args, output_path) {
                Ok(_) => println!("{}", "  ✓ shadcn-vue components added".bright_green()),
                Err(e) => {
                    println!("{} {}", "  ✗ Failed:".bright_red(), e);
                    println!("  You may need to run 'npx shadcn-vue@latest add {} --yes' manually.", components.join(" "));
                }
            }
        } else {
            println!();
            println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: shadcn-vue components already installed (skipping)", current_step, total_steps).bright_white());
        }
    } else {
        println!();
        println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: No shadcn-vue components needed", current_step, total_steps).bright_white());
    }

    // Step 3: Copy public assets
    current_step += 1;
    println!();
    if let Some(source_public) = public_folder_source {
        let dest_public = output_path.join("public");
        if source_public.exists() && source_public.is_dir() {
            // Check if public assets are already copied (destination exists and has same files)
            if dest_public.exists() && dest_public.is_dir() {
                println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Public assets already copied (skipping)", current_step, total_steps).bright_white());
            } else {
                println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Copying public assets...", current_step, total_steps).bright_white());
                copy_dir_all(source_public, &dest_public)
                    .map_err(|e| format!("Failed to copy public folder: {}", e))?;
                println!("{}", "  ✓ Public assets copied".bright_green());
            }
        } else {
            println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: No public assets to copy", current_step, total_steps).bright_white());
        }
    } else {
        println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: No public assets to copy", current_step, total_steps).bright_white());
    }

    // Step 4: Run dev server
    current_step += 1;
    println!();
    println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Ready to start dev server", current_step, total_steps).bright_white());
    println!();
    println!("{}", "═════════════════════════════════".bright_green().bold());
    println!("{}", "  Project created successfully!".bright_green().bold());
    println!("{}", "═════════════════════════════════".bright_green().bold());
    println!();
    println!("Starting dev server...");
    println!();

    let _ = run_command_live("npm", &["run", "dev"], output_path);

    Ok(())
}

/// Generate Vue project from AURA file or workspace
pub fn generate_vue_project(
    input_path: Option<&str>,
    output_dir: Option<&str>,
    project_name: Option<&str>,
    no_install: bool,
    yes: bool,
) -> Result<(), String> {
    // Check if we're in workspace mode (pac.at exists)
    if input_path.is_none() {
        if let Some(pac_path) = find_pac_at() {
            return generate_workspace_project(&pac_path, output_dir, no_install, yes);
        } else {
            return Err("No pac.at found in current directory. Please specify an input file: auto vue <input.at>".to_string());
        }
    }

    let input = input_path.unwrap();
    let input_path_buf = Path::new(input);

    // Check if input is a directory (workspace mode)
    if input_path_buf.is_dir() {
        // Look for pac.at in the directory
        let pac_path = input_path_buf.join("pac.at");
        if pac_path.exists() {
            return generate_workspace_project(&pac_path, output_dir, no_install, yes);
        } else {
            return Err(format!("No pac.at found in directory '{}'", input));
        }
    }

    // Legacy mode: transpile single .at file
    generate_single_file_project(input, output_dir, project_name, no_install, yes)
}

/// Generate Vue project from single AURA file (legacy mode)
fn generate_single_file_project(
    input_path: &str,
    output_dir: Option<&str>,
    project_name: Option<&str>,
    no_install: bool,
    yes: bool,
) -> Result<(), String> {
    // Determine output directory
    let input = Path::new(input_path);
    let input_stem = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("aura-app");

    let output = output_dir.unwrap_or(input_stem);
    let output_path = Path::new(output);

    // Determine project name
    let name = project_name.unwrap_or_else(|| {
        output_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(input_stem)
    });

    println!("{}", "─────────────────────────────────".bright_yellow().bold());
    println!("{}", "  AURA → Vue + shadcn-vue".bright_yellow().bold());
    println!("{}", "─────────────────────────────────".bright_yellow().bold());
    println!();

    // Check prerequisites
    if !command_exists("npm") {
        return Err("npm not found. Please install Node.js from https://nodejs.org/".to_string());
    }

    println!("{} {}", "Input:".bright_cyan(), input_path);
    println!("{} {}", "Output:".bright_cyan(), output);
    println!("{} {}", "Name:".bright_cyan(), name);
    println!();

    // Create output directory
    if output_path.exists() {
        return Err(format!("Output directory '{}' already exists", output));
    }

    fs::create_dir_all(output_path)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    // Create src directory structure
    let src_dir = output_path.join("src");
    let components_dir = src_dir.join("components");
    let lib_dir = src_dir.join("lib");
    let assets_dir = src_dir.join("assets");

    fs::create_dir_all(&components_dir)
        .map_err(|e| format!("Failed to create src/components: {}", e))?;
    fs::create_dir_all(&lib_dir)
        .map_err(|e| format!("Failed to create src/lib: {}", e))?;
    fs::create_dir_all(&assets_dir)
        .map_err(|e| format!("Failed to create src/assets: {}", e))?;

    // Note: public folder will be copied in Step 3 during install steps
    let input_dir = input.parent().unwrap_or(Path::new("."));
    let source_public = input_dir.join("public");

    println!("{}", "✓ Created directory structure".bright_green());

    // Parse AURA and generate Vue component with shadcn-vue mode
    let (vue_code, widgets) = auto_lang::ui_build_shadcn_with_widgets(input_path, None)
        .map_err(|e| format!("Failed to generate Vue code: {:?}", e))?;

    // Detect required shadcn components
    let components = detect_shadcn_components(&vue_code);
    println!("{} {}", "✓ Detected shadcn-vue components:".bright_green(), components.join(", "));

    // Detect routes from widgets
    let mut all_routes: Vec<AuraRoute> = Vec::new();
    for widget in &widgets {
        if let Some(ref routes) = widget.routes {
            all_routes.extend(routes.routes.clone());
        }
    }

    let has_routes = !all_routes.is_empty();
    if has_routes {
        println!("{} {}", "✓ Detected routes:".bright_green(), all_routes.len());
        for route in &all_routes {
            println!("    {} -> {}", route.path.bright_cyan(), route.module);
        }
    }

    // Write project files
    write_project_files(output_path, name, &vue_code, &components, has_routes)?;

    // Generate router files if routes detected
    if has_routes {
        let router_dir = output_path.join("src/router");
        fs::create_dir_all(&router_dir)
            .map_err(|e| format!("Failed to create src/router: {}", e))?;

        let router_content = VueGenerator::generate_router_file(&all_routes);
        fs::write(router_dir.join("index.ts"), router_content)
            .map_err(|e| format!("Failed to write router/index.ts: {}", e))?;

        println!("{}", "  Generated src/router/index.ts".bright_green());
    }

    println!("{}", "✓ Generated project files".bright_green());

    if no_install {
        // For no-install mode, copy public folder directly
        if source_public.exists() && source_public.is_dir() {
            let dest_public = output_path.join("public");
            copy_dir_all(&source_public, &dest_public)
                .map_err(|e| format!("Failed to copy public folder: {}", e))?;
            println!("{}", "✓ Copied public assets".bright_green());
        }

        println!();
        println!("{}", "Project created successfully!".bright_green().bold());
        println!();
        println!("Next steps:");
        println!("  cd {}", output);
        println!("  npm install");
        println!("  npx shadcn-vue@latest add {} --yes", components.join(" "));
        println!("  npm run dev");
    } else {
        // Check if npm exists
        if !command_exists("npm") {
            println!();
            println!("{}", "⚠ npm not found".bright_yellow());
            println!("Please install Node.js from https://nodejs.org/");
            println!();
            println!("Then run:");
            println!("  cd {}", output);
            println!("  npm install");
            println!("  npx shadcn-vue@latest add {} --yes", components.join(" "));
            println!("  npm run dev");
            return Ok(());
        }

        // Detect which steps need to be run
        // Always run npm install to pick up new dependencies (it's fast if already installed)
        let shadcn_needed = !components.is_empty() && !are_shadcn_components_installed(output_path, &components);

        let total_steps = 4;
        let mut current_step = 0;

        // Step 1: npm install (always run to pick up new dependencies)
        println!();
        current_step += 1;
        println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Installing dependencies...", current_step, total_steps).bright_white());

        let npm_install_args = if yes {
            println!("{}", "  Running: npm install -y".bright_black());
            vec!["install", "-y"]
        } else {
            println!("{}", "  Running: npm install".bright_black());
            vec!["install"]
        };

        match run_command_live("npm", &npm_install_args, output_path) {
            Ok(_) => println!("{}", "  ✓ Dependencies installed".bright_green()),
            Err(e) => {
                println!("{} {}", "  ✗ Failed:".bright_red(), e);
                println!("  You may need to run 'npm install' manually.");
            }
        }

        // Step 2: shadcn-vue add (or skip if already installed or not needed)
        current_step += 1;
        if !components.is_empty() {
            if shadcn_needed {
                println!();
                println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Adding shadcn-vue components ({})...", current_step, total_steps, components.join(", ")).bright_white());

                let mut args = if yes {
                    println!("{}", format!("  Running: npx --yes shadcn-vue@latest add {} --yes", components.join(" ")).bright_black());
                    vec!["--yes", "shadcn-vue@latest", "add"]
                } else {
                    println!("{}", format!("  Running: npx shadcn-vue@latest add {} --yes", components.join(" ")).bright_black());
                    vec!["shadcn-vue@latest", "add"]
                };
                args.extend(components.iter().map(|s| s.as_str()));
                args.push("--yes");

                match run_command_live("npx", &args, output_path) {
                    Ok(_) => println!("{}", "  ✓ shadcn-vue components added".bright_green()),
                    Err(e) => {
                        println!("{} {}", "  ✗ Failed:".bright_red(), e);
                        println!("  You may need to run 'npx shadcn-vue@latest add {} --yes' manually.", components.join(" "));
                    }
                }
            } else {
                println!();
                println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: shadcn-vue components already installed (skipping)", current_step, total_steps).bright_white());
            }
        } else {
            println!();
            println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: No shadcn-vue components needed", current_step, total_steps).bright_white());
        }

        // Step 3: Copy public assets
        current_step += 1;
        println!();
        if source_public.exists() && source_public.is_dir() {
            let dest_public = output_path.join("public");
            if dest_public.exists() && dest_public.is_dir() {
                println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Public assets already copied (skipping)", current_step, total_steps).bright_white());
            } else {
                println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Copying public assets...", current_step, total_steps).bright_white());
                copy_dir_all(&source_public, &dest_public)
                    .map_err(|e| format!("Failed to copy public folder: {}", e))?;
                println!("{}", "  ✓ Public assets copied".bright_green());
            }
        } else {
            println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: No public assets to copy", current_step, total_steps).bright_white());
        }

        // Step 4: Run dev server
        current_step += 1;
        println!();
        println!("{} {}", "▶".bright_cyan(), format!("Step {}/{}: Ready to start dev server", current_step, total_steps).bright_white());
        println!();
        println!("{}", "═════════════════════════════════".bright_green().bold());
        println!("{}", "  Project created successfully!".bright_green().bold());
        println!("{}", "═════════════════════════════════".bright_green().bold());
        println!();
        println!("Starting dev server...");
        println!();

        // Run npm run dev
        let _ = run_command_live("npm", &["run", "dev"], output_path);
    }

    Ok(())
}

/// Detect which shadcn-vue components are needed from generated Vue code
fn detect_shadcn_components(vue_code: &str) -> Vec<String> {
    let mut components = HashSet::new();

    // Map import patterns to component names
    let component_patterns = [
        ("@/components/ui/button", "button"),
        ("@/components/ui/input", "input"),
        ("@/components/ui/textarea", "textarea"),
        ("@/components/ui/checkbox", "checkbox"),
        ("@/components/ui/switch", "switch"),
        ("@/components/ui/select", "select"),
        ("@/components/ui/tabs", "tabs"),
        ("@/components/ui/dialog", "dialog"),
        ("@/components/ui/tooltip", "tooltip"),
        ("@/components/ui/slider", "slider"),
        ("@/components/ui/radio-group", "radio-group"),
        ("@/components/ui/progress", "progress"),
        ("@/components/ui/badge", "badge"),
        ("@/components/ui/skeleton", "skeleton"),
        ("@/components/ui/card", "card"),
        ("@/components/ui/avatar", "avatar"),
        ("@/components/ui/table", "table"),
        ("@/components/ui/separator", "separator"),
        ("@/components/ui/scroll-area", "scroll-area"),
        ("@/components/ui/label", "label"),
        // Feedback & Overlay
        ("@/components/ui/alert", "alert"),
        ("@/components/ui/sonner", "sonner"),
        ("@/components/ui/dropdown-menu", "dropdown-menu"),
        ("@/components/ui/popover", "popover"),
        ("@/components/ui/sheet", "sheet"),
        ("@/components/ui/breadcrumb", "breadcrumb"),
        // High Priority Components
        ("@/components/ui/accordion", "accordion"),
        ("@/components/ui/alert-dialog", "alert-dialog"),
        ("@/components/ui/command", "command"),
        ("@/components/ui/form", "form"),
        ("@/components/ui/navigation-menu", "navigation-menu"),
        ("@/components/ui/sidebar", "sidebar"),
        ("@/components/ui/stepper", "stepper"),
        // Medium Priority Components
        ("@/components/ui/calendar", "calendar"),
        ("@/components/ui/carousel", "carousel"),
        ("@/components/ui/combobox", "combobox"),
        ("@/components/ui/context-menu", "context-menu"),
        ("@/components/ui/drawer", "drawer"),
        ("@/components/ui/hover-card", "hover-card"),
        ("@/components/ui/number-field", "number-field"),
        ("@/components/ui/pagination", "pagination"),
        ("@/components/ui/pin-input", "pin-input"),
        ("@/components/ui/tags-input", "tags-input"),
        ("@/components/ui/toggle-group", "toggle-group"),
        // Low Priority Components
        ("@/components/ui/aspect-ratio", "aspect-ratio"),
        ("@/components/ui/button-group", "button-group"),
        ("@/components/ui/chart", "chart"),
        ("@/components/ui/collapsible", "collapsible"),
        ("@/components/ui/input-group", "input-group"),
        ("@/components/ui/input-otp", "input-otp"),
        ("@/components/ui/kbd", "kbd"),
        ("@/components/ui/menubar", "menubar"),
        ("@/components/ui/native-select", "native-select"),
        ("@/components/ui/range-calendar", "range-calendar"),
        ("@/components/ui/resizable", "resizable"),
        ("@/components/ui/auto-complete", "auto-complete"),
    ];

    for (pattern, component) in component_patterns {
        if vue_code.contains(pattern) {
            components.insert(component.to_string());
        }
    }

    // Sort for consistent output
    let mut result: Vec<String> = components.into_iter().collect();
    result.sort();
    result
}

/// Write all project files
fn write_project_files(
    output_path: &Path,
    name: &str,
    vue_code: &str,
    components: &[String],
    has_routes: bool,
) -> Result<(), String> {
    // package.json
    let package_json = generate_package_json(name, has_routes);
    fs::write(output_path.join("package.json"), package_json)
        .map_err(|e| format!("Failed to write package.json: {}", e))?;

    // vite.config.ts
    let vite_config = generate_vite_config();
    fs::write(output_path.join("vite.config.ts"), vite_config)
        .map_err(|e| format!("Failed to write vite.config.ts: {}", e))?;

    // tsconfig.json
    let tsconfig = generate_tsconfig();
    fs::write(output_path.join("tsconfig.json"), tsconfig)
        .map_err(|e| format!("Failed to write tsconfig.json: {}", e))?;

    // tsconfig.node.json
    let tsconfig_node = generate_tsconfig_node();
    fs::write(output_path.join("tsconfig.node.json"), tsconfig_node)
        .map_err(|e| format!("Failed to write tsconfig.node.json: {}", e))?;

    // tailwind.config.cjs (use .cjs for ES module compatibility)
    let tailwind_config = generate_tailwind_config();
    fs::write(output_path.join("tailwind.config.cjs"), tailwind_config)
        .map_err(|e| format!("Failed to write tailwind.config.cjs: {}", e))?;

    // postcss.config.cjs
    let postcss_config = generate_postcss_config();
    fs::write(output_path.join("postcss.config.cjs"), postcss_config)
        .map_err(|e| format!("Failed to write postcss.config.cjs: {}", e))?;

    // index.html
    let index_html = generate_index_html(name);
    fs::write(output_path.join("index.html"), index_html)
        .map_err(|e| format!("Failed to write index.html: {}", e))?;

    // src/main.ts
    let main_ts = generate_main_ts(has_routes);
    fs::write(output_path.join("src/main.ts"), main_ts)
        .map_err(|e| format!("Failed to write src/main.ts: {}", e))?;

    // src/App.vue
    let app_vue = generate_app_vue(vue_code);
    fs::write(output_path.join("src/App.vue"), app_vue)
        .map_err(|e| format!("Failed to write src/App.vue: {}", e))?;

    // src/assets/index.css
    let index_css = generate_index_css();
    fs::write(output_path.join("src/assets/index.css"), index_css)
        .map_err(|e| format!("Failed to write src/assets/index.css: {}", e))?;

    // src/lib/utils.ts
    let utils_ts = generate_utils_ts();
    fs::write(output_path.join("src/lib/utils.ts"), utils_ts)
        .map_err(|e| format!("Failed to write src/lib/utils.ts: {}", e))?;

    // Write Vue component(s)
    write_vue_components(&output_path.join("src/components"), vue_code)?;

    Ok(())
}

/// Write Vue component files
fn write_vue_components(components_dir: &Path, vue_code: &str) -> Result<(), String> {
    // For now, we write the entire generated code as a single component
    // The vue generator already produces proper component code

    // Extract widget name from the generated code
    let widget_name = extract_widget_name(vue_code).unwrap_or_else(|| "Widget".to_string());

    // Write the component file
    fs::write(components_dir.join(format!("{}.vue", widget_name)), vue_code)
        .map_err(|e| format!("Failed to write component: {}", e))?;

    Ok(())
}

/// Extract widget name from generated Vue code
fn extract_widget_name(vue_code: &str) -> Option<String> {
    // Look for <!-- WidgetName component --> comment
    for line in vue_code.lines() {
        if line.starts_with("<!--") && line.contains("component") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
    }
    None
}

// Template generators

fn generate_package_json(name: &str, has_routes: bool) -> String {
    let router_dep = if has_routes {
        r#"    "vue-router": "^4.2.0",
"#
    } else {
        ""
    };

    format!(r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vue-tsc && vite build",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "vue": "^3.4.0",
{}    "@vueuse/core": "^10.7.0",
    "reka-ui": "^2.0.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.2.0",
    "lucide-vue-next": "^0.312.0",
    "prismjs": "^1.29.0",
    "embla-carousel-vue": "^8.5.1",
    "vee-validate": "^4.15.1",
    "@vee-validate/zod": "^4.15.1",
    "zod": "^3.25.76"
  }},
  "devDependencies": {{
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.3.0",
    "vue-tsc": "^1.8.0",
    "tailwindcss": "^3.4.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "tailwindcss-animate": "^1.0.7"
  }}
}}
"#, name, router_dep)
}

fn generate_vite_config() -> String {
    r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    open: true
  }
})
"#.to_string()
}

fn generate_tsconfig() -> String {
    r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
"#.to_string()
}

fn generate_tsconfig_node() -> String {
    r#"{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
"#.to_string()
}

fn generate_tailwind_config() -> String {
    r#"/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: 0 },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: 0 },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
}
"#.to_string()
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

fn generate_index_html(name: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/main.ts"></script>
</body>
</html>
"#, name)
}

fn generate_main_ts(has_routes: bool) -> String {
    if has_routes {
        r#"import { createApp, nextTick } from 'vue'
import App from './App.vue'
import router from './router'
import './assets/index.css'

// Prism.js setup
import Prism from 'prismjs'
import 'prismjs/components/prism-bash'
import 'prismjs/components/prism-typescript'
import 'prismjs/components/prism-javascript'
import 'prismjs/components/prism-css'
import 'prismjs/components/prism-markup' // HTML/XML
import 'prismjs/themes/prism-tomorrow.min.css'

// Define custom 'auto' language (similar to rust/swift)
Prism.languages.auto = Prism.languages.extend('clike', {
  'keyword': /\b(widget|fn|let|mut|const|var|if|else|for|in|loop|while|break|continue|return|use|import|export|type|struct|enum|impl|trait|pub|private|static|async|await|try|catch|throw|new|true|false|null|nil|self|super)\b/,
  'string': /(["'`])(?:\\[\s\S]|(?!\1)[^\\])*\1/,
  'number': /\b\d+\.?\d*\b/,
  'operator': /[+\-*/%=<>!&|^~?:]+/,
  'punctuation': /[{}[\]();,.]/,
  'function': /\b[a-zA-Z_]\w*(?=\s*\()/,
  'comment': /\/\/.*|\/\*[\s\S]*?\*\//,
});

// Define custom 'vue' language (HTML-based with Vue directives)
Prism.languages.vue = Prism.languages.extend('markup', {
  // Vue directives: v-if, v-for, v-model, @click, :class, etc.
  'directive': {
    pattern: /(?:^|\s)(?:v-[a-z]+|@[a-z]+(?::[a-z]+)?|:[a-z-]+)(?:=\s*(?:"[^"]*"|'[^']*'|[^\s"'=<>`]+))?/i,
    lookbehind: true,
    inside: {
      'punctuation': /^=|[:=]/,
      'attribute-name': /[a-z-]+/i,
    }
  },
  // Mustache templates: {{ ... }}
  'mustache': {
    pattern: /\{\{[\s\S]*?\}\}/,
    inside: {
      'delimiter': /^\{\{|\}\}$/,
      'expression': /[\s\S]+/,
    }
  },
});

// Highlight function that works with Vue's render cycle
function highlightCode() {
  nextTick(() => {
    Prism.highlightAll()
  })
}

const app = createApp(App)
app.use(router)
app.mount('#app')

// Initial highlight
highlightCode()

// Re-highlight on route change
router.afterEach(() => {
  highlightCode()
})
"#.to_string()
    } else {
        r#"import { createApp, nextTick } from 'vue'
import App from './App.vue'
import './assets/index.css'

// Prism.js setup
import Prism from 'prismjs'
import 'prismjs/components/prism-bash'
import 'prismjs/components/prism-typescript'
import 'prismjs/components/prism-javascript'
import 'prismjs/components/prism-css'
import 'prismjs/components/prism-markup' // HTML/XML
import 'prismjs/themes/prism-tomorrow.min.css'

// Define custom 'auto' language (similar to rust/swift)
Prism.languages.auto = Prism.languages.extend('clike', {
  'keyword': /\b(widget|fn|let|mut|const|var|if|else|for|in|loop|while|break|continue|return|use|import|export|type|struct|enum|impl|trait|pub|private|static|async|await|try|catch|throw|new|true|false|null|nil|self|super)\b/,
  'string': /(["'`])(?:\\[\s\S]|(?!\1)[^\\])*\1/,
  'number': /\b\d+\.?\d*\b/,
  'operator': /[+\-*/%=<>!&|^~?:]+/,
  'punctuation': /[{}[\]();,.]/,
  'function': /\b[a-zA-Z_]\w*(?=\s*\()/,
  'comment': /\/\/.*|\/\*[\s\S]*?\*\//,
});

// Define custom 'vue' language (HTML-based with Vue directives)
Prism.languages.vue = Prism.languages.extend('markup', {
  // Vue directives: v-if, v-for, v-model, @click, :class, etc.
  'directive': {
    pattern: /(?:^|\s)(?:v-[a-z]+|@[a-z]+(?::[a-z]+)?|:[a-z-]+)(?:=\s*(?:"[^"]*"|'[^']*'|[^\s"'=<>`]+))?/i,
    lookbehind: true,
    inside: {
      'punctuation': /^=|[:=]/,
      'attribute-name': /[a-z-]+/i,
    }
  },
  // Mustache templates: {{ ... }}
  'mustache': {
    pattern: /\{\{[\s\S]*?\}\}/,
    inside: {
      'delimiter': /^\{\{|\}\}$/,
      'expression': /[\s\S]+/,
    }
  },
});

createApp(App).mount('#app')

// Highlight after mount
nextTick(() => {
  Prism.highlightAll()
})
"#.to_string()
    }
}

fn generate_app_vue(vue_code: &str) -> String {
    // Extract the widget name to create the import
    let widget_name = extract_widget_name(vue_code).unwrap_or_else(|| "Widget".to_string());

    format!(r#"<script setup lang="ts">
import {0} from './components/{0}.vue'
</script>

<template>
  <div class="min-h-screen bg-background">
    <{0} />
  </div>
</template>
"#, widget_name)
}

fn generate_index_css() -> String {
    r#"@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 0 0% 100%;
    --foreground: 222.2 84% 4.9%;
    --card: 0 0% 100%;
    --card-foreground: 222.2 84% 4.9%;
    --popover: 0 0% 100%;
    --popover-foreground: 222.2 84% 4.9%;
    --primary: 222.2 47.4% 11.2%;
    --primary-foreground: 210 40% 98%;
    --secondary: 210 40% 96.1%;
    --secondary-foreground: 222.2 47.4% 11.2%;
    --muted: 210 40% 96.1%;
    --muted-foreground: 215.4 16.3% 46.9%;
    --accent: 210 40% 96.1%;
    --accent-foreground: 222.2 47.4% 11.2%;
    --destructive: 0 84.2% 60.2%;
    --destructive-foreground: 210 40% 98%;
    --border: 214.3 31.8% 91.4%;
    --input: 214.3 31.8% 91.4%;
    --ring: 222.2 84% 4.9%;
    --radius: 0.5rem;
  }

  .dark {
    --background: 222.2 84% 4.9%;
    --foreground: 210 40% 98%;
    --card: 222.2 84% 4.9%;
    --card-foreground: 210 40% 98%;
    --popover: 222.2 84% 4.9%;
    --popover-foreground: 210 40% 98%;
    --primary: 210 40% 98%;
    --primary-foreground: 222.2 47.4% 11.2%;
    --secondary: 217.2 32.6% 17.5%;
    --secondary-foreground: 210 40% 98%;
    --muted: 217.2 32.6% 17.5%;
    --muted-foreground: 215 20.2% 65.1%;
    --accent: 217.2 32.6% 17.5%;
    --accent-foreground: 210 40% 98%;
    --destructive: 0 62.8% 30.6%;
    --destructive-foreground: 210 40% 98%;
    --border: 217.2 32.6% 17.5%;
    --input: 217.2 32.6% 17.5%;
    --ring: 212.7 26.8% 83.9%;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
    font-feature-settings: "rlig" 1, "calt" 1;
  }
}

@layer components {
  /* Header styles */
  header {
    @apply w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60;
  }

  /* Sidebar styles */
  aside {
    @apply w-64 border-r bg-background;
  }

  /* Navigation link styles */
  nav a,
  a.nav-link {
    @apply text-sm font-medium text-muted-foreground hover:text-foreground transition-colors;
  }

  /* Sidebar link styles */
  aside a {
    @apply block px-3 py-2 rounded-md text-sm hover:bg-muted transition-colors;
  }

  /* Main content area */
  main {
    @apply flex-1;
  }
}
"#.to_string()
}

fn generate_utils_ts() -> String {
    r#"import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
"#.to_string()
}
