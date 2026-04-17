use crate::builder::Builder;
use crate::{AutoResult, Pac, Target};
use auto_val::AutoPath;
use std::collections::HashMap;
use std::path::Path;

pub struct VueBuilder {
    path: AutoPath,
    dist_dir: String,
    memory_output_enabled: bool,
    memory_files: HashMap<String, Vec<u8>>,
}

impl VueBuilder {
    pub fn new(path: AutoPath) -> Self {
        Self {
            path,
            dist_dir: "dist".to_string(),
            memory_output_enabled: false,
            memory_files: HashMap::new(),
        }
    }

    fn project_dir(&self) -> &Path {
        self.path.path()
    }
}

impl Builder for VueBuilder {
    fn build(&mut self, _pac: &mut Pac) -> AutoResult<()> {
        log::info!("Building Vue project: {}", self.path);

        if self.memory_output_enabled {
            log::info!("Memory output enabled, skipping physical build execution");
            return Ok(());
        }

        let project_dir = self.project_dir();
        let pm = crate::pkg::display_name();

        println!("Running {} run build in {}...", pm, project_dir.display());
        crate::pkg::run_script("build", &[], project_dir)
            .map_err(|e| format!("{} run build failed: {}", pm, e))?;

        println!("Vue project built successfully!");
        Ok(())
    }

    fn setup(&mut self, _pac: &mut Pac) -> AutoResult<()> {
        log::info!("Setting up Vue builder: {}", self.path);
        Ok(())
    }

    fn finish(&mut self, _pac: &Pac) -> AutoResult<()> {
        Ok(())
    }

    fn target(&mut self, _target: &Target, _pac: &Pac) -> AutoResult<()> {
        Ok(())
    }

    fn clean(&mut self) -> AutoResult<()> {
        log::info!("Cleaning Vue project: {}", self.path);
        if !self.memory_output_enabled {
            let project_dir = self.project_dir();
            let dist_path = project_dir.join(&self.dist_dir);
            if dist_path.exists() {
                std::fs::remove_dir_all(&dist_path)?;
                println!("Removed {} directory", self.dist_dir);
            }
        }
        Ok(())
    }

    fn run(&mut self, _pac: &Pac, args: Vec<String>) -> AutoResult<()> {
        log::info!("Running Vue dev server: {}", self.path);

        if !self.memory_output_enabled {
            let project_dir = self.project_dir();
            let pm = crate::pkg::display_name();

            println!("Starting Vue dev server...");
            let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            crate::pkg::run_script("dev", &args_str, project_dir)
                .map_err(|e| format!("{} run dev failed: {}", pm, e))?;
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
