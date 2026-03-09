use crate::asset::Templates;
use crate::AutoResult;
use crate::Pac;
use crate::Port;
use crate::TargetKind;
use crate::{Index, IndexStore};
use auto_lang::config::AutoConfig;
use auto_val::shared;
use auto_val::{AutoPath, AutoStr, Obj, Value};
use log::*;
use reqwest::blocking::get;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::path::Path;

// Plan 082: AutoCache integration
use auto_cache::{ArtifactType, AutoManCache};

pub struct Automan {
    pac: Pac,
    index_store: IndexStore,
    cache: Option<AutoManCache>, // Optional cache for builds
}

// Static API
impl Automan {
    pub fn create_app(name: &str) -> AutoResult<()> {
        Self::create_by_template(name, "app")
    }

    pub fn create_capp(name: &str) -> AutoResult<()> {
        Self::create_by_template(name, "capp")
    }

    pub fn create_lib(name: &str) -> AutoResult<()> {
        Self::create_by_template(name, "lib")
    }

    pub fn create_clib(name: &str) -> AutoResult<()> {
        Self::create_by_template(name, "clib")
    }

    /// Create a new Jetpack Compose Android project
    pub fn create_jet(name: &str) -> AutoResult<()> {
        use auto_lang::ui_gen::jet::{JetProjectConfig, ProjectGenerator};

        let path = Path::new(name);

        // Check if project already exists
        if path.is_file() {
            return Err(format!("A file named with {} already exists", name).into());
        }
        if path.is_dir() && path.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false) {
            return Err(format!("A non-empty directory named with {} already exists", name).into());
        }

        // Get project name from path
        let pac_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("MyApp");

        // Generate project with JetProjectConfig
        let config = JetProjectConfig::new(pac_name);
        let mut generator = ProjectGenerator::with_config(config);
        let files = generator.generate();

        // Create all files
        std::fs::create_dir_all(path)?;
        for (file_path, content) in files {
            let full_path = path.join(&file_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, content)?;
            info!("Created {}", file_path);
        }

        println!("Jetpack Compose project '{}' created successfully!", pac_name);
        println!("Open with: auto open");
        Ok(())
    }

    pub fn create_by_template(name: &str, template: &str) -> AutoResult<()> {
        // Special handling for jet template (dynamic generation)
        if template == "jet" {
            return Self::create_jet(name);
        }

        // Default: copy from static templates
        let path = Path::new(name);
        if path.is_file() {
            return Err(format!("A file named with {} already exists", name).into());
        }
        if path.is_dir() {
            return Err(format!("A directory named with {} already exists", name).into());
        }
        Templates::copy(template, name)?;
        Ok(())
    }

    pub fn reset_index() -> AutoResult<()> {
        // rewrite am.at
        let home = dirs::home_dir().ok_or("Can't open home dir")?;
        let auto_dir = home.join(".auto").join("auto-man");
        let am_at_file = auto_dir.join("am.at");
        let am_url =
            "https://gitee.com/auto-stack/auto-man/raw/master/crates/auto-man/assets/am.at";
        let page = get(am_url)?.text()?;
        std::fs::write(&am_at_file, page)?;
        info!("updated {}", am_at_file.display());
        // remove index dirs
        let index_dir = auto_dir.join("index");
        if index_dir.exists() {
            std::fs::remove_dir_all(&index_dir)?;
            info!("removed index dir {}", index_dir.display());
        }
        Ok(())
    }

    pub fn install_config(file: &str) -> AutoResult<()> {
        let home = dirs::home_dir().ok_or("Can't open home dir")?;
        let auto_dir = home.join(".auto").join("auto-man");
        let am_at_file = auto_dir.join("am.at");
        let path = Path::new(file);
        if !path.exists() {
            return Err(format!("am.at config file {} not found", file).into());
        }
        let content = std::fs::read_to_string(path)?;
        std::fs::write(&am_at_file, content)?;
        info!("updated {}", am_at_file.display());
        Ok(())
    }
}

// API
impl Automan {
    pub fn parse_pac(path: &str, am_config: &AmConfig) -> AutoResult<Self> {
        // 1. Find pac.at
        let config_path = Path::new(path).join("pac.at");
        if !config_path.is_file() {
            return Err(format!("No pac.at file found in {}", path).into());
        }

        // 2. try load saved port
        let port = Self::try_load_port()?;

        // let user select a port
        let config = AutoConfig::read(config_path.as_path())?;
        let ports = config.root.get_nodes("port");

        let mut port_names = Vec::new();
        for port in ports {
            port_names.push(port.main_arg().to_astr());
        }

        let dft_port: Port = Port::default();
        let port_name = match port {
            None => {
                if !port_names.contains(&dft_port.name) {
                    // default port is win32
                    port_names.push(dft_port.name);
                }
                Self::select_port(&port_names)?
            }
            Some(port) => {
                if !port_names.contains(&port) {
                    if !port_names.contains(&dft_port.name) {
                        // default port is win32
                        port_names.push(dft_port.name);
                    }
                    Self::select_port(&port_names)?
                } else {
                    port
                }
            }
        };

        println!("port NAME: {}", port_name);

        // Load config without port variable (port variable was used in old Universe-based code)
        let config = AutoConfig::from_file(config_path.as_path(), &Obj::new())?;
        let mut index_used: Vec<AutoStr> = Vec::new();
        if config.root.has_prop("index") {
            index_used = config
                .root
                .get_prop("index")
                .as_array()
                .iter()
                .map(|t| t.to_astr())
                .collect();
        }
        if index_used.is_empty() {
            index_used.push("default".into());
        }
        let mut pac = Pac::new(config);
        let index_store = Self::index_store(am_config, index_used)?;
        info!("loading index store from {}", index_store.path.to_astr());
        pac.index = shared(Index::load(index_store.path.join("index.at").to_astr())?);
        pac.device_index = shared(Index::load(index_store.path.join("devices.at").to_astr())?);

        pac.set_port(port_name.clone())?;
        pac.save_port(port_name.clone())?;

        Ok(Self {
            pac,
            index_store,
            cache: None,
        })
    }

    fn select_port(ports: &Vec<AutoStr>) -> AutoResult<AutoStr> {
        crate::util::select_or_default_port(None, ports, "Which port do you want to build?")
    }

    pub fn get_state_file() -> AutoResult<AutoPath> {
        // check state file exists? .am/state.at
        let am_dir = AutoPath::new(".am");

        if !am_dir.is_dir() {
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(&am_dir.path())?;
        }

        let state_file = am_dir.join("state.at");
        if !state_file.is_file() {
            // Create the state file if it doesn't exist
            std::fs::File::create(&state_file.path())?;
        }

        Ok(state_file)
    }

    pub fn try_load_port() -> AutoResult<Option<AutoStr>> {
        // load local state from .am/state.at
        let state_file = Self::get_state_file()?;
        let state = AutoConfig::read(&state_file.path())?;

        let port_name = state.root.get_prop("port").to_astr();
        println!("Loaded port: {}", port_name);
        if !port_name.is_empty() {
            Ok(Some(port_name))
        } else {
            Ok(None)
        }
    }

    pub fn new(path: &str, am: AmConfig) -> AutoResult<Self> {
        // Currently, only CMake builder is supported
        let mut am = Self::parse_pac(path, &am)?;

        // Initialize cache if enabled (enabled by default, can be disabled with AUTO_CACHE_ENABLED=false)
        let cache_enabled = env::var("AUTO_CACHE_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true); // Default: enabled

        if cache_enabled {
            match AutoManCache::in_home_dir(am.pac.name.to_string()) {
                Ok(cache) => {
                    info!("AutoCache initialized for project: {}", am.pac.name);
                    am.cache = Some(cache);
                }
                Err(e) => {
                    warn!("Failed to initialize AutoCache: {}", e);
                }
            }
        }

        Ok(am)
    }

    pub fn pull(&mut self) -> AutoResult<()> {
        self.index_store.pull()?;
        self.pac.pull()
    }

    pub fn set_port(&mut self, port: AutoStr) -> AutoResult<()> {
        self.pac.set_port(port.clone())?;
        self.pac.save_port(port)
    }

    pub fn open_ide(&mut self) -> AutoResult<()> {
        println!("build dir: {}", self.pac.build_location);
        println!("port: {}", self.pac.port.name);

        // Check backend first for project-type-specific IDEs
        let backend = self.pac.backend.as_str();
        if backend == "jet" {
            // Open Jetpack Compose project with Android Studio
            return self.open_jet_project();
        }

        // Fall back to port builder for embedded IDEs
        match self.pac.port.builder.as_str() {
            "iar" => {
                // open iar ide in the build_location
                let eww_path = AutoPath::new(self.pac.build_location.as_str())
                    .join(self.pac.name.to_string() + ".eww")
                    .to_astr()
                    .replace("/", "\\");
                println!("eww: {}", eww_path.clone());
                std::process::Command::new("explorer.exe")
                    .arg(eww_path.as_str())
                    .output()?;
            }
            "ghs" => {
                let ghs_path = AutoPath::new(self.pac.build_location.as_str())
                    .join("default.gpj")
                    .to_astr()
                    .replace("/", "\\");
                println!("ghs: {}", ghs_path.clone());
                std::process::Command::new("explorer.exe")
                    .arg(ghs_path.as_str())
                    .output()?;
            }
            "cmake" => {
                // in windows, there should be a .sln project in build dir
                let sln_path = AutoPath::new(self.pac.build_location.as_str())
                    .join("build")
                    .join(self.pac.name.to_string() + ".sln")
                    .to_astr()
                    .replace("/", "\\");
                println!("sln: {}", sln_path.clone());
                std::process::Command::new("explorer.exe")
                    .arg(sln_path.as_str())
                    .output()?;
            }
            _ => {
                println!("port builder: {}", self.pac.port.builder);
                todo!()
            }
        }
        Ok(())
    }

    /// Open Jetpack Compose project with Android Studio
    fn open_jet_project(&self) -> AutoResult<()> {
        let project_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        println!("Opening Jetpack Compose project with Android Studio...");
        println!("Project: {}", project_dir.display());

        // Try to find Android Studio installation
        let studio_path = self.find_android_studio();

        if let Some(studio) = studio_path {
            println!("Android Studio: {}", studio);
            std::process::Command::new(&studio)
                .arg(&project_dir)
                .spawn()
                .map_err(|e| format!("Failed to launch Android Studio: {}", e))?;
        } else {
            // Fallback: open with system default handler
            println!("Android Studio not found in default locations.");
            println!("Opening project folder with system default...");

            #[cfg(target_os = "windows")]
            {
                let path_str = project_dir.to_string_lossy().replace("/", "\\");
                std::process::Command::new("explorer.exe")
                    .arg(&path_str)
                    .output()
                    .map_err(|e| format!("Failed to open folder: {}", e))?;
            }

            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(&project_dir)
                    .output()
                    .map_err(|e| format!("Failed to open folder: {}", e))?;
            }

            #[cfg(target_os = "linux")]
            {
                std::process::Command::new("xdg-open")
                    .arg(&project_dir)
                    .output()
                    .map_err(|e| format!("Failed to open folder: {}", e))?;
            }
        }

        Ok(())
    }

    /// Find Android Studio installation path
    fn find_android_studio(&self) -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            // Common Android Studio installation paths on Windows
            let candidates = vec![
                // User-specific installation
                format!(
                    "{}\\AppData\\Local\\Programs\\Android Studio\\bin\\studio64.exe",
                    std::env::var("USERPROFILE").unwrap_or_default()
                ),
                // Program Files
                "C:\\Program Files\\Android\\Android Studio\\bin\\studio64.exe".to_string(),
                "C:\\Program Files (x86)\\Android\\Android Studio\\bin\\studio64.exe".to_string(),
                // Check PATH for studio64.exe
                "studio64.exe".to_string(),
            ];

            for path in candidates {
                if std::path::Path::new(&path).exists() || path == "studio64.exe" {
                    // For "studio64.exe", check if it's in PATH
                    if path == "studio64.exe" {
                        if std::process::Command::new(&path)
                            .arg("--version")
                            .output()
                            .is_ok()
                        {
                            return Some(path);
                        }
                    } else {
                        return Some(path);
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let app_path = "/Applications/Android Studio.app";
            if std::path::Path::new(app_path).exists() {
                return Some(app_path.to_string());
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Check common snap and flatpak installations
            let candidates = vec![
                "/snap/bin/android-studio",
                "/usr/local/android-studio/bin/studio.sh",
                "android-studio", // In PATH
            ];

            for path in candidates {
                if std::path::Path::new(path).exists() || path == "android-studio" {
                    return Some(path.to_string());
                }
            }
        }

        None
    }

    pub fn scan(&mut self) -> AutoResult<()> {
        // self.pac.print_targets();
        // 3. Resolve targets in the pac, including:
        // 3.1. Resolve dependencies
        // 3.2. Link targets with dependencies
        // 3.3. Scan targets for its local source files
        self.pac.resolve()?;
        self.pac.print_targets();
        self.pac.print_atom()
    }

    pub fn info(&self, _target: Option<String>) -> AutoResult<()> {
        // read content from pac.atom.at
        let config = AutoConfig::read(Path::new("pac.atom.at"))?;
        let pac = Pac::new(config);
        pac.print_targets();
        Ok(())
        // if let Some(target) = target {
        //     self.pac.print_target_info(&target)
        // } else {
        //     self.pac.print_targets();
        //     self.pac.print_atom()
        // }
    }

    pub fn build(&mut self) -> AutoResult<()> {
        let backend = self.pac.backend.as_str();

        match backend {
            "vue" => {
                // Vue backend: run npm run build in dist directory
                println!("Building Vue project (backend: vue)");
                self.build_vue()?;

                // Run garbage collection if needed
                if let Some(ref cache) = self.cache {
                    if cache.should_gc() {
                        println!("Running cache garbage collection...");
                        let freed_mb = cache.run_gc()? / (1024 * 1024);
                        println!("Cache GC: freed {} MB", freed_mb);
                    }
                }
            }
            _ => {
                // Default C backend
                println!("Transpiling auto code to c code");
                self.transpile_auto()?;

                // Build the project with a specific builder
                self.pac.build()?;

                // Run garbage collection if needed (Plan 082: AutoCache)
                if let Some(ref cache) = self.cache {
                    if cache.should_gc() {
                        println!("Running cache garbage collection...");
                        let freed_mb = cache.run_gc()? / (1024 * 1024);
                        println!("Cache GC: freed {} MB", freed_mb);
                    }
                }
            }
        }

        Ok(())
    }

    /// Build Vue project using npm (full workflow: generate, install, build)
    fn build_vue(&mut self) -> AutoResult<()> {
        let root_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        crate::vue::build_vue_project(&root_dir)
    }

    pub fn export(&mut self, port_name: String, format: String) -> AutoResult<()> {
        // 1. Set the port if different from current
        if self.pac.port.name.as_str() != port_name {
            info!("Switching to port: {}", port_name);
            self.set_port(port_name.into())?;
        }

        // 2. Transpile auto code
        // For now, exporters (CMake/IAR/GHS) are primarily for C backend
        println!("Transpiling auto code for export");
        self.transpile_auto()?;

        // 3. Resolve targets
        self.pac.resolve()?;

        // 4. Create exporter and export
        let build_path = AutoPath::new(&self.pac.build_location);
        if let Some(mut exporter) = crate::exporter::make_exporter(&format, build_path) {
            println!("Exporting project to {} format...", format);
            exporter.export(&mut self.pac)?;
            println!("Export completed successfully at {}", self.pac.build_location);
            Ok(())
        } else {
            Err(format!("Unknown export format: {}", format).into())
        }
    }

    pub fn transpile_auto(&mut self) -> AutoResult<()> {
        self.pac.transpile_autot()?;
        Ok(())
    }

    pub fn run(&mut self, args: Vec<String>) -> AutoResult<()> {
        let backend = self.pac.backend.as_str();

        match backend {
            "vue" => {
                // Vue backend: run npm run dev in dist directory
                println!("Running Vue dev server (backend: vue)");
                self.run_vue(args)
            }
            _ => {
                // Default: use pac.run()
                self.pac.run(args)
            }
        }
    }

    /// Run Vue dev server using npm run dev (full workflow: generate, install, run)
    fn run_vue(&mut self, args: Vec<String>) -> AutoResult<()> {
        let root_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        crate::vue::run_vue_project(&root_dir, args)
    }

    pub fn clean(&mut self) -> AutoResult<()> {
        // logs
        let files = glob::glob("*.log")?;
        for file in files {
            if let Ok(file) = file {
                info!("deleting file {}", file.display());
                std::fs::remove_file(file)?;
            }
        }

        // clean downloaded deps with pac.atom.at
        let atom_file = AutoPath::new(".am/pac.atom.at");
        if !atom_file.is_file() {
            return Err(format!(
                "{}\n{}\n{}",
                "Build info file .am/pac.atom.at not found!",
                "`am clean` depends on this info file to complete.",
                "Please run `am b` first to generate build info file."
            )
            .into());
        }
        let history_config = AutoConfig::read(atom_file.path())?;
        let mut history_pac = Pac::new(history_config);
        for t in history_pac
            .targets
            .iter_mut()
            .filter(|t| t.kind == TargetKind::Dep || t.kind == TargetKind::Device)
        {
            println!("target [{}] at {} ...", t.name, t.at);
            match t.clean() {
                Ok(_) => info!("Target [{}] at {} cleaned successfully", t.name, t.at),
                Err(e) => {
                    error!("Error: {}", e);
                    continue;
                }
            }
        }

        // clean .am folder
        std::fs::remove_dir_all(".am")?;
        // clean pac related files
        self.pac.clean()
    }

    // Plan 082: AutoCache management methods

    /// Display cache statistics
    pub fn cache_stats(&self) -> AutoResult<()> {
        let cache = self
            .cache
            .as_ref()
            .ok_or("AutoCache is not available. Set AUTO_CACHE_ENABLED=true to enable.")?;

        let stats = cache.get_statistics();

        println!("\n=== AutoCache Statistics ===");
        println!("Total Artifacts: {}", stats.count);
        println!(
            "Total Size: {:.2} GB / {} GB",
            stats.size_gb, stats.max_size_gb
        );
        println!("Hit Rate: {:.1}%", stats.hit_rate * 100.0);
        println!(
            "Status: {}",
            if stats.size_gb > stats.max_size_gb as f64 {
                "⚠️  Exceeds limit (GC needed)"
            } else if stats.size_gb > (stats.max_size_gb as f64 * 0.8) {
                "⚠️  Near limit"
            } else {
                "✓ Healthy"
            }
        );
        println!("============================\n");

        Ok(())
    }

    /// Run garbage collection manually
    pub fn cache_prune(&mut self) -> AutoResult<()> {
        let cache = self
            .cache
            .as_mut()
            .ok_or("AutoCache is not available. Set AUTO_CACHE_ENABLED=true to enable.")?;

        println!("Running cache garbage collection...");
        let freed_bytes = cache.run_gc().map_err(|e| format!("GC failed: {}", e))?;

        let freed_mb = freed_bytes / (1024 * 1024);
        let freed_gb = freed_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        println!(
            "Freed {} artifacts ({} MB, {:.2} GB)",
            freed_bytes > 0,
            freed_mb,
            freed_gb
        );

        // Show updated stats
        self.cache_stats()?;

        Ok(())
    }

    /// Clear all cached artifacts
    pub fn cache_clear(&mut self) -> AutoResult<()> {
        let cache = self
            .cache
            .as_mut()
            .ok_or("AutoCache is not available. Set AUTO_CACHE_ENABLED=true to enable.")?;

        print!("⚠️  This will clear ALL cached artifacts. Continue? [y/N] ");
        use std::io::Write;
        std::io::stdout().flush().ok();

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }

        println!("Clearing all cache...");
        cache
            .clear_all()
            .map_err(|e| format!("Failed to clear cache: {}", e))?;

        println!("Cache cleared successfully.");
        Ok(())
    }

    /// Inspect a specific cache entry by module name or hash key
    pub fn cache_inspect(&self, name_or_hash: &str) -> AutoResult<()> {
        let cache = self
            .cache
            .as_ref()
            .ok_or("AutoCache is not available. Set AUTO_CACHE_ENABLED=true to enable.")?;

        // Try to find artifact by hash key first
        if let Some(metadata) = cache.get_metadata(name_or_hash) {
            Self::display_metadata(&metadata);
            return Ok(());
        }

        // If not found by hash, search by module name
        // This requires listing artifacts and filtering by module name
        let artifacts = cache
            .list_artifacts(None, 1000) // Get up to 1000 artifacts
            .map_err(|e| format!("Failed to search artifacts: {}", e))?;

        let matching: Vec<_> = artifacts
            .iter()
            .filter(|a| a.module_name.contains(name_or_hash))
            .collect();

        if matching.is_empty() {
            println!("\nNo cache entry found for '{}'\n", name_or_hash);
            println!("Use `auto cache list` to see all cached artifacts.\n");
            return Ok(());
        }

        if matching.len() == 1 {
            println!("\n=== Cache Entry: {} ===\n", matching[0].module_name);
            Self::display_metadata(matching[0]);
        } else {
            println!(
                "\nFound {} cache entries matching '{}':\n",
                matching.len(),
                name_or_hash
            );
            for artifact in matching {
                println!(
                    "  [{}] {} - {} ({})",
                    &artifact.hash_key[..16],
                    artifact.module_name,
                    artifact.artifact_type,
                    Self::format_size(artifact.file_size)
                );
            }
            println!("\nUse specific hash key for full details.\n");
        }

        Ok(())
    }

    /// List all cached artifacts with optional filtering
    pub fn cache_list(&self, type_filter: Option<String>, limit: usize) -> AutoResult<()> {
        let cache = self
            .cache
            .as_ref()
            .ok_or("AutoCache is not available. Set AUTO_CACHE_ENABLED=true to enable.")?;

        let stats = cache.get_statistics();
        let count = stats.count as usize;

        println!(
            "\n=== Cached Artifacts (showing {} of {}) ===\n",
            limit.min(count),
            count
        );

        if stats.count == 0 {
            println!("Cache is empty.\n");
            return Ok(());
        }

        // Parse type filter
        let artifact_type = match type_filter.as_deref() {
            Some("c") => Some(ArtifactType::TranspiledC),
            Some("h") => Some(ArtifactType::TranspiledCHeader),
            Some("rust") => Some(ArtifactType::TranspiledRust),
            Some("bytecode") => Some(ArtifactType::Bytecode),
            Some("object") => Some(ArtifactType::CompiledObject),
            Some(_) => {
                println!("Unknown artifact type. Valid types: c, h, rust, bytecode, object\n");
                return Ok(());
            }
            None => None,
        };

        // List artifacts
        let artifacts = cache
            .list_artifacts(artifact_type, limit)
            .map_err(|e| format!("Failed to list artifacts: {}", e))?;

        if artifacts.is_empty() {
            println!("No artifacts found.\n");
            return Ok(());
        }

        // Display header
        println!(
            "{:<35} {:<12} {:>10} {:>12} {:>8}",
            "Module", "Type", "Size", "Last Used", "Access"
        );
        println!(
            "{:-<35} {:-<12} {:->10} {:->12} {:->8}",
            "-", "-", "-", "-", "-"
        );

        // Display artifacts
        for artifact in &artifacts {
            let module_name = artifact.module_name.clone();
            let size_str = Self::format_size(artifact.file_size);
            let last_used = Self::format_time_ago(artifact.last_used_at);
            let access_count = artifact.access_count;

            println!(
                "{:<35} {:<12} {:>10} {:>12} {:>8}",
                module_name, artifact.artifact_type, size_str, last_used, access_count
            );
        }

        println!("\n(Top {} artifacts shown)", artifacts.len());
        println!("Use `auto cache inspect <hash>` for details\n");

        Ok(())
    }

    /// Verify cache integrity
    pub fn cache_verify(&self) -> AutoResult<()> {
        let cache = self
            .cache
            .as_ref()
            .ok_or("AutoCache is not available. Set AUTO_CACHE_ENABLED=true to enable.")?;

        println!("\n=== Verifying Cache Integrity ===\n");

        println!("Checking metadata entries...");
        println!("Checking blob files...");
        println!("Verifying file integrity...");

        let report = cache
            .verify_integrity()
            .map_err(|e| format!("Failed to verify cache: {}", e))?;

        println!();

        if report.is_valid {
            println!("✓ Cache integrity verified");
        } else {
            println!("⚠ Cache integrity issues detected");
        }

        println!("  - {} metadata entries", report.metadata_entries);
        println!("  - {} blob files", report.blob_files);
        println!("  - {} orphaned files", report.orphaned_files);
        println!("  - {} corrupted entries", report.corrupted_entries);

        if !report.is_valid {
            println!("\nRecommendations:");
            if report.corrupted_entries > 0 {
                println!("  - Run `auto cache clear` to remove corrupted entries");
            }
            if report.orphaned_files > 0 {
                println!("  - Orphaned files will be cleaned up by garbage collection");
            }
        }

        println!();
        Ok(())
    }

    /// Format file size for display
    fn format_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Format timestamp as time ago
    fn format_time_ago(timestamp: u64) -> String {
        let now = chrono::Utc::now().timestamp() as u64;
        let diff = now.saturating_sub(timestamp);

        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;
        const DAY: u64 = 24 * HOUR;

        if diff < MINUTE {
            format!("{}s ago", diff)
        } else if diff < HOUR {
            format!("{}m ago", diff / MINUTE)
        } else if diff < DAY {
            format!("{}h ago", diff / HOUR)
        } else {
            format!("{}d ago", diff / DAY)
        }
    }

    /// Format timestamp as date/time string
    fn format_timestamp(timestamp: u64) -> String {
        let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0);
        match datetime {
            Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            None => format!("Invalid timestamp: {}", timestamp),
        }
    }

    /// Display artifact metadata
    fn display_metadata(metadata: &auto_cache::ArtifactMetadata) {
        println!("Hash Key:         {}", metadata.hash_key);
        println!("Module:           {}", metadata.module_name);
        println!("Type:             {}", metadata.artifact_type);
        println!(
            "Size:             {}",
            Self::format_size(metadata.file_size)
        );
        println!("Source Hash:      {}", metadata.source_hash);
        println!("Project:          {}", metadata.project_name);
        println!(
            "Created:          {}",
            Self::format_timestamp(metadata.created_at)
        );
        println!(
            "Last Used:        {}",
            Self::format_timestamp(metadata.last_used_at)
        );
        println!("Access Count:     {}", metadata.access_count);
        println!("Blob Path:        {}", metadata.blob_path.display());
        println!();
    }

    pub fn list_port_names(&self) -> Vec<AutoStr> {
        self.pac.list_port_names()
    }

    pub fn list_deps(config: &AmConfig) -> AutoResult<()> {
        let store = Self::index_store(config, vec![])?;
        store.list_deps()
    }

    pub fn index_store(config: &AmConfig, indexs: Vec<AutoStr>) -> AutoResult<IndexStore> {
        // try to get index location from amconfig
        let am_path = home_path().join(".auto/auto-man");
        if !am_path.is_dir() {
            info!("creating am dir: {}", am_path.to_astr());
            std::fs::create_dir_all(am_path.path())?;
        }
        // get index name and urls from config and indexs
        let all_index = &config.index;
        println!("all index: {:?}", all_index);
        let mut used_indexs = indexs;
        if used_indexs.is_empty() {
            // fill all index
            for k in all_index.keys() {
                used_indexs.push(k.clone());
            }
        }
        println!("used index: {:?}", used_indexs);

        let mut used_index_path = AutoPath::new(".");

        for index in used_indexs {
            let index_path = am_path.join("index").join(index.clone());
            used_index_path = index_path.clone();
            let repo = all_index.get(&index);
            let Some(repo) = repo else {
                error!(
                    "index base not found! {}, please check ~/.auto/auto-man/am.at",
                    index
                );
                continue;
            };
            if !index_path.is_dir() {
                // try to clone
                info!("cloning index dir: {}", index_path.to_astr());
                std::fs::create_dir_all(index_path.path())?;

                let result = std::process::Command::new("git")
                    .arg("clone")
                    .arg(repo.as_str())
                    .arg(index_path.path())
                    .output();
                match result {
                    Err(e) => {
                        error!(
                            "Failed to clone repository {} to {} with error {}",
                            repo, index_path, e
                        );
                    }
                    Ok(_) => {}
                }
            }
        }
        // else {
        // // try to update
        // info!("updating index dir: {}", index_path.to_astr());
        // let result = std::process::Command::new("git")
        //     .arg("pull")
        //     .arg("--rebase")
        //     .current_dir(index_path.path())
        //     .output();
        // match result {
        //     Err(e) => {
        //         let repo = config.index.as_str();
        //         error!(
        //             "Failed to update repository {} to {} with error {}",
        //             repo, index_path, e
        //         );
        //     }
        //     Ok(_) => {}
        // }
        // }
        return Ok(IndexStore::new(used_index_path));

        // let dir = Self::search_dirs_or_clone(
        //     config,
        //     ".am/index",
        //     vec!["D://".into(), home_path(), "C://".into(), "E://".into()],
        // );
        // println!("dir: {:?}", dir);
        // if let Some(dir) = dir {
        //     Ok(IndexStore::new(dir))
        // } else {
        //     Err(format!("No index found").into())
        // }
    }

    pub fn list_devices(config: &AmConfig) -> AutoResult<()> {
        let store = Self::index_store(config, vec![])?;
        store.list_devices()
    }
}

fn home_path() -> AutoPath {
    dirs::home_dir().unwrap().to_str().unwrap().into()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_search_dirs() {
        // let path = search_dirs_or_clone(
        //     ".am/index",
        //     vec!["D://".into(), home_path(), "C://".into(), "E://".into()],
        // );
        // assert!(path.is_some());
        // assert!(path.unwrap().is_dir());
    }

    #[test]
    fn test_env() {
        use std::env;
        println!("home: {}", env::var("HOME").unwrap());
    }
}

pub struct AmConfig {
    pub name: AutoStr,
    pub index: HashMap<AutoStr, AutoStr>,
    pub is_default: bool,
}

impl Default for AmConfig {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(
            "default".into(),
            "git@gitee.com:auto-stack/auto-index.git".into(),
        );
        Self {
            name: "default".into(),
            index: map,
            is_default: true,
        }
    }
}

impl Display for AmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "AmConfig {{")?;
        writeln!(f, "  name: {}", self.name)?;
        writeln!(f, "  index: {{")?;
        for (k, v) in self.index.iter() {
            writeln!(f, "    {}: {}", k, v)?;
        }
        writeln!(f, "  }}")?;
        writeln!(f, "}}\n")
    }
}

pub fn load_am_config() -> Option<AmConfig> {
    // try to look for am.at file in the working directory
    let mut am_path = None;

    let local_config_path = env::current_dir().unwrap().join("am.at");
    if local_config_path.is_file() {
        am_path = Some(local_config_path);
    }

    if am_path.is_none() {
        let user_home_config_path = dirs::home_dir()
            .unwrap()
            .join(".auto")
            .join("auto-man")
            .join("am.at");
        if !user_home_config_path.is_dir() {
            // create parent dir
            let rs = std::fs::create_dir_all(user_home_config_path.parent().unwrap());
            if rs.is_err() {
                error!("Failed to create parent directory for am.at file");
            }
        }
        if user_home_config_path.is_file() {
            am_path = Some(user_home_config_path);
        } else {
            info!("Automan Config file am.at not found in current or home directory, using the default one");
            // write default config to am.at file
            let default_config = AmConfig::default();
            let default_config_str = default_config.to_string();
            std::fs::write(user_home_config_path.clone(), default_config_str).unwrap();
            am_path = Some(user_home_config_path);
        }
    }

    let Some(am_path) = am_path else {
        return None;
    };
    let code = std::fs::read_to_string(am_path.clone());
    let Ok(code) = code else {
        return None;
    };
    let config = AutoConfig::new(code);
    let Ok(config) = config else {
        return None;
    };
    let mut am = AmConfig::default();
    if config.root.has_prop("index") {
        let index_prop = config.root.get_prop("index");
        if let Value::Obj(obj) = index_prop {
            am.index = obj.to_hashmap();
        }
    }
    am.is_default = false;
    println!("Loading AmConfig from {:?}...", am_path.to_str());
    println!("AmConfig: {}", am);
    Some(am)
}
