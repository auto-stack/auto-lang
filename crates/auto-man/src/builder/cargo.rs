use crate::builder::Builder;
use crate::{AutoResult, Pac, Target, TargetKind};
use auto_val::AutoPath;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct CargoBuilder {
    path: AutoPath,
    memory_output_enabled: bool,
    memory_files: HashMap<String, Vec<u8>>,
}

impl CargoBuilder {
    pub fn new(path: AutoPath) -> Self {
        Self {
            path,
            memory_output_enabled: false,
            memory_files: HashMap::new(),
        }
    }

    fn write_file(&mut self, path: &Path, content: &str) -> AutoResult<()> {
        if self.memory_output_enabled {
            self.memory_files.insert(
                path.to_string_lossy().to_string(),
                content.as_bytes().to_vec(),
            );
            Ok(())
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, content)?;
            Ok(())
        }
    }

    /// Find auto-lang crate path relative to the build directory.
    ///
    /// Strategy:
    /// 1. If the project is inside the auto-lang monorepo (has crates/auto-lang ancestor),
    ///    compute a relative path from the build dir.
    /// 2. Otherwise, use the compile-time known location of auto-lang via CARGO_MANIFEST_DIR.
    fn find_auto_lang_path(&self) -> Option<String> {
        let cwd = std::env::current_dir().ok()?;
        let build_dir = cwd.join(self.path.parent().path());
        let canonical_build = build_dir.canonicalize().ok()?;

        // Strategy 1: walk up from build dir looking for crates/auto-lang
        let mut dir = canonical_build.clone();
        let mut ups = 0;
        loop {
            let candidate = dir.join("crates/auto-lang");
            if candidate.is_dir() {
                let mut rel = "../".repeat(ups);
                rel.push_str("crates/auto-lang");
                return Some(rel);
            }
            match dir.parent() {
                Some(p) => {
                    dir = p.to_path_buf();
                    ups += 1;
                }
                None => break,
            }
        }

        // Strategy 2: use compile-time auto-man crate dir to compute relative path
        // auto-man is at <repo>/crates/auto-man, auto-lang is at <repo>/crates/auto-lang
        let auto_man_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let auto_lang_abs = auto_man_dir.parent()?.join("auto-lang");
        let canonical_auto_lang = auto_lang_abs.canonicalize().ok()?;
        if canonical_auto_lang.is_dir() {
            // Compute relative path from build_dir to canonical_auto_lang
            let mut ups = 0;
            let mut d = canonical_build.as_path();
            // Walk up from build_dir until we find a common ancestor with canonical_auto_lang
            loop {
                if let Ok(stripped) = canonical_auto_lang.strip_prefix(d) {
                    let mut rel = "../".repeat(ups);
                    for (i, component) in stripped.components().enumerate() {
                        if i > 0 {
                            rel.push('/');
                        }
                        rel.push_str(&component.as_os_str().to_string_lossy());
                    }
                    return Some(rel.replace('\\', "/"));
                }
                d = d.parent()?;
                ups += 1;
            }
        }

        None
    }
}

impl Builder for CargoBuilder {
    fn build(&mut self, pac: &mut Pac) -> AutoResult<()> {
        log::info!("Building with Cargo: {}", self.path);

        // Generate Cargo.toml pointing to already-transpiled rust/src/
        self.setup(pac)?;

        if self.memory_output_enabled {
            log::info!("Memory output enabled, skipping physical cargo execution");
            return Ok(());
        }

        let dir = self.path.parent();
        let status = std::process::Command::new("cargo")
            .arg("build")
            .current_dir(dir.path())
            .status()?;

        if !status.success() {
            return Err(format!("Cargo build failed with status: {}", status).into());
        }

        Ok(())
    }

    fn setup(&mut self, pac: &mut Pac) -> AutoResult<()> {
        log::info!("Setting up Cargo builder: {}", self.path);

        // Find the first rust app/lib target to use as the Cargo package
        let target = pac.targets.iter().find(|t| {
            t.lang.as_str() == "rust"
                && (t.kind == TargetKind::App || t.kind == TargetKind::Lib)
        });

        let Some(target) = target else {
            log::warn!("No rust target found, skipping Cargo setup");
            return Ok(());
        };

        // Use "0.1.0" as fallback version (cargo doesn't accept "latest")
        let version = target.version.as_str();
        let version = if version.is_empty() || version == "latest" {
            "0.1.0"
        } else {
            version
        };

        // Determine crate type based on target kind
        let crate_type = if target.kind == TargetKind::App {
            ""
        } else {
            "\n[lib]"
        };

        // Cargo package names can't start with a digit. Sanitize by prepending
        // "app-" if the name starts with a digit (e.g. 015-notes → app-015-notes).
        let pkg_name = if target.name.as_str().chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            format!("app-{}", target.name.as_str())
        } else {
            target.name.as_str().to_string()
        };

        let mut cargo_toml = format!(
            r#"[package]
name = "{}"
version = "{}"
edition = "2021"
{}"#,
            pkg_name,
            version,
            crate_type
        );

        // Add dependencies
        cargo_toml.push_str("\n\n[dependencies]\n");
        // a2r-generated code imports auto_lang::a2r_std, so always add it
        // Try to find auto-lang relative to current workspace
        let auto_lang_path = self.find_auto_lang_path();
        if let Some(path) = auto_lang_path {
            cargo_toml.push_str(&format!("auto-lang = {{ path = \"{}\" }}\n", path));
        } else {
            // Fallback: assume it's available via cargo registry or workspace
            cargo_toml.push_str("auto-lang = \"*\"\n");
        }
        for dep in &target.deps {
            if dep.lang.as_str() == "rust" {
                cargo_toml.push_str(&format!(
                    "{} = {{ path = \"../{}\" }}\n",
                    dep.name.as_str(),
                    dep.name.as_str()
                ));
            }
        }

        // Scan existing .rs files in src/ for external crate usage
        // (e.g. hand-written files using `use serde::{Serialize, Deserialize}`)
        let src_dir_path = self.path.parent().join("src");
        if src_dir_path.path().exists() {
            let mut extra_deps: Vec<String> = Vec::new();
            let built_in = ["std", "core", "alloc", "proc_macro", "crate", "super", "self",
                "a2r_std", "auto_lang"];
            scan_rs_for_crates(src_dir_path.path(), &mut extra_deps, &built_in);
            for dep in &extra_deps {
                // Check it's not already in deps (from target.deps or auto-lang)
                let already = dep == "auto-lang"
                    || target.deps.iter().any(|d| d.name.as_str() == dep);
                if !already {
                    cargo_toml.push_str(&format!("{} = \"*\"\n", dep));
                }
            }
        }

        // Plan 328: If router.rs exists (a2r generated Axum server), inject
        // fixed-version dependencies for axum/tokio/serde. These are required
        // for the generated server code but won't appear via scan_rs_for_crates
        // because they're in auto-generated use statements.
        let router_rs = self.path.parent().join("src").join("router.rs");
        let commands_rs = self.path.parent().join("src").join("commands.rs");
        if router_rs.path().exists() {
            // Plan 328: Axum HTTP server dependencies
            cargo_toml.push_str("axum = \"0.7\"\n");
            cargo_toml.push_str("tokio = { version = \"1\", features = [\"full\"] }\n");
            cargo_toml.push_str("serde = { version = \"1\", features = [\"derive\"] }\n");
            cargo_toml.push_str("serde_json = \"1\"\n");
            cargo_toml.push_str("futures = \"0.3\"\n");
            eprintln!("[a2r] Injected axum/tokio/serde/futures dependencies");
        } else if commands_rs.path().exists() {
            // Plan 328 IPC: Tauri IPC dependencies
            cargo_toml.push_str("tauri = { version = \"2\", features = [\"protocol-asset\"] }\n");
            cargo_toml.push_str("serde = { version = \"1\", features = [\"derive\"] }\n");
            cargo_toml.push_str("serde_json = \"1\"\n");
            eprintln!("[a2r] Injected tauri/serde dependencies (IPC mode)");
        }

        // Prevent cargo from detecting parent workspace
        cargo_toml.push_str("\n\n[workspace]\n");

        // Write Cargo.toml at self.path (e.g., rust/Cargo.toml)
        let cargo_toml_path = self.path.path().to_path_buf();
        self.write_file(&cargo_toml_path, &cargo_toml)?;

        // Ensure rust/src/ directory exists (transpile should have already created it)
        let src_dir = self.path.parent().join("src");
        if !self.memory_output_enabled && !src_dir.path().exists() {
            fs::create_dir_all(src_dir.path())?;
        }

        // Check if main.rs/lib.rs exists; if not, generate a stub
        let is_app = target.kind == TargetKind::App;
        let root_file = if is_app {
            src_dir.join("main.rs")
        } else {
            src_dir.join("lib.rs")
        };

        if !self.memory_output_enabled && !root_file.path().exists() {
            let stub = if is_app {
                "\nfn main() {\n    // Application entry point\n}\n"
            } else {
                ""
            };
            self.write_file(root_file.path(), stub)?;
        }

        Ok(())
    }

    fn finish(&mut self, _pac: &Pac) -> AutoResult<()> {
        Ok(())
    }

    fn target(&mut self, _target: &Target, _pac: &Pac) -> AutoResult<()> {
        Ok(())
    }

    fn clean(&mut self) -> AutoResult<()> {
        log::info!("Cleaning with Cargo: {}", self.path);
        if !self.memory_output_enabled {
            let dir = self.path.parent();
            std::process::Command::new("cargo")
                .arg("clean")
                .current_dir(dir.path())
                .status()?;
        }
        Ok(())
    }

    fn run(&mut self, _pac: &Pac, args: Vec<String>) -> AutoResult<()> {
        log::info!("Running with Cargo: {}", self.path);
        if !self.memory_output_enabled {
            let dir = self.path.parent();
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("run").current_dir(dir.path());
            for arg in args {
                cmd.arg(arg);
            }
            let status = cmd.status()?;
            if !status.success() {
                return Err(format!("Cargo run failed with status: {}", status).into());
            }
        }
        Ok(())
    }

    fn enable_memory_output(&mut self) -> AutoResult<()> {
        self.memory_output_enabled = true;
        Ok(())
    }

    fn get_memory_output(&self) -> HashMap<String, Vec<u8>> {
        self.memory_files.clone()
    }
}

/// Recursively scan .rs files for external crate usage (e.g. `use serde::{...}`).
/// Only detects `use X::...` patterns (must contain `::` to be an external crate).
fn scan_rs_for_crates(dir: &Path, deps: &mut Vec<String>, exclude: &[&str]) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_rs_for_crates(&path, deps, exclude);
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(rest) = trimmed.strip_prefix("use ") {
                            let rest = rest.trim_start();
                            // Must contain :: to be an external crate path
                            if !rest.contains("::") {
                                continue;
                            }
                            let first = rest.split("::").next().unwrap_or("").trim();
                            if !first.is_empty()
                                && !exclude.contains(&first)
                                && !deps.contains(&first.to_string())
                            {
                                deps.push(first.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}
