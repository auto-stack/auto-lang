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
}

impl Builder for CargoBuilder {
    fn build(&mut self, _pac: &mut Pac) -> AutoResult<()> {
        log::info!("Building with Cargo: {}", self.path);

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
        let build_dir = self.path.parent();

        let mut members = Vec::new();

        // Setup each target as a Cargo package
        for target in &pac.targets {
            if target.lang.as_str() != "rust" {
                continue;
            }

            let target_name = target.name.as_str();
            members.push(target_name.to_string());

            let target_dir = build_dir.join(target_name);
            let target_cargo_toml = target_dir.join("Cargo.toml");
            let src_dir = target_dir.join("src");

            // Generate Cargo.toml for this target
            let mut cargo_toml = format!(
                r#"[package]
name = "{}"
version = "{}"
edition = "2021"

[dependencies]
"#,
                target.name.as_str(),
                target.version.as_str()
            );

            // Map target dependencies to Cargo path dependencies
            for dep in &target.deps {
                if dep.lang.as_str() == "rust" {
                    cargo_toml.push_str(&format!(
                        r#"{} = {{ path = "../{}" }}"#,
                        dep.name.as_str(),
                        dep.name.as_str()
                    ));
                    cargo_toml.push('\n');
                }
            }

            self.write_file(target_cargo_toml.path(), &cargo_toml)?;

            // Generate root source file based on target kind
            let is_app = target.kind == TargetKind::App;
            let root_file = if is_app {
                src_dir.join("main.rs")
            } else {
                src_dir.join("lib.rs")
            };

            let mut root_content = String::new();

            // Find transpiled .rs files belonging to this target and declare them as modules
            for src in &target.srcs {
                let src_path = AutoPath::new(src.as_str());
                if let Some(ext) = src_path.path().extension() {
                    if ext == "rs" {
                        if let Some(stem) = src_path.path().file_stem() {
                            let module_name = stem.to_string_lossy();

                            // Don't declare main.rs or lib.rs inside themselves
                            if module_name != "main" && module_name != "lib" {
                                root_content.push_str(&format!("pub mod {};\n", module_name));
                            }
                        }

                        // Copy or move the transpiled file into the src directory
                        let dest_path = src_dir.join(src_path.filename());

                        if !self.memory_output_enabled {
                            let parent_path = dest_path.parent();
                            if !parent_path.path().as_os_str().is_empty() {
                                fs::create_dir_all(parent_path.path())?;
                            }
                            fs::copy(src_path.path(), dest_path.path())?;
                        }
                    }
                }
            }

            // If App, ensure we have a main function if one wasn't somehow copied
            if is_app
                && !root_content.contains("fn main")
                && !target
                    .srcs
                    .iter()
                    .any(|s| AutoPath::new(s.as_str()).filename() == "main.rs")
            {
                root_content.push_str("\nfn main() {\n    // Application entry point\n}\n");
            }

            self.write_file(root_file.path(), &root_content)?;
        }

        // Generate workspace Cargo.toml
        if !members.is_empty() {
            let mut workspace_toml = String::from("[workspace]\nmembers = [\n");
            for member in members {
                workspace_toml.push_str(&format!("    \"{}\",\n", member));
            }
            workspace_toml.push_str("]\n");
            // Setup a default resolver
            workspace_toml.push_str("resolver = \"2\"\n");

            let path = self.path.clone();
            self.write_file(path.path(), &workspace_toml)?;
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
