//! Target 数据半与语言/FS 效果方法（PLAN-809 T-03 起）：
//! - `Target`/`TargetKind`/`TargetOrigin`/`get_root_dir`/`set_to_vec` 单一
//!   实现于 `auto-man-core/src/target.rs`（声明归一/选择/投影纯规则），
//!   本模块 re-export 保持旧公开路径与类型身份；
//! - transpile/scan/clean 等**效果方法**以 `TargetEffects` 扩展 trait 承接
//!   （Rust 不允许跨 crate inherent impl）——`use auto_man::target::*` 或
//!   `use auto_man::*` 即可继续 `t.scan()`/`t.transpile_auto()` 方法语法；
//! - `TargetStatus`（tabled 展示）留本模块。

use crate::dir::Dir;
use crate::git::check_changes;
use crate::AutoResult;
use auto_lang_core::trans::c::transpile_c;
use auto_lang_core::util::*;
use auto_val::{AutoPath, AutoStr};
use log::*;
use std::path::Path;
use tabled::Tabled;

// Plan 082: AutoCache integration
use auto_cache::CTranspilationCache;

pub use auto_man_core::target::{
    get_root_dir, get_root_dir_with_probe, set_to_vec, Target, TargetKind, TargetOrigin,
};

/// Target 的语言转译/FS 扫描/清理效果方法（原 inherent impl 的逐字承接；
/// 纯规则半见 auto-man-core::target）。
pub trait TargetEffects {
    fn transpile_auto(&mut self) -> AutoResult<()>;
    fn transpile_auto_with_cache(&mut self) -> AutoResult<()>;
    fn exists(&self) -> bool;
    fn downloaded(&self) -> bool;
    fn clean(&self) -> AutoResult<()>;
    fn print_info(&self) -> AutoResult<()>;
    fn scan(&mut self) -> AutoResult<()>;
}

/// collect_srcs 的自由函数形态（原 pub(crate) 方法；避免 trait 可见面扩宽）。
pub fn collect_target_srcs(t: &mut Target) -> AutoResult<()> {
    use crate::dir::DirEffects;
    let mut sub_srcs = std::collections::HashSet::new();
    for d in t.dirs.values() {
        let dir_srcs = d.collect_srcs()?;
        sub_srcs.extend(dir_srcs);
    }
    t.srcs.extend(sub_srcs);

    if !t.deps.is_empty() {
        for dep in t.deps.iter_mut() {
            collect_target_srcs(dep)?;
        }
    }
    Ok(())
}

impl TargetEffects for Target {
    fn transpile_auto(&mut self) -> AutoResult<()> {
        use indicatif::{ProgressBar, ProgressStyle};

        if self.autos.is_empty() {
            return Ok(());
        }

        // Multi-file Rust project: use transpile_rust_project() for cross-module support
        if self.lang.as_str() == "rust" && self.autos.len() > 1 {
            return transpile_rust_project_multi(self);
        }

        // Create progress bar for transpilation
        let pb = ProgressBar::new(self.autos.len() as u64);
        pb.set_message("Transpiling Auto files");
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("=> "),
        );

        for path in self.autos.iter() {
            pb.set_message(format!("Transpiling {}", path.as_str()));
            info!("Transpiling {}", path.as_str());

            // Read the Auto source file
            let content = std::fs::read_to_string(path.as_str())
                .map_err(|e| format!("Failed to read Auto file '{}': {}", path, e))?;

            // Transpile based on target language
            let fname = AutoPath::new(path).basename();

            if self.lang.as_str() == "rust" {
                let mut rust_code =
                    auto_lang_core::trans::rust::transpile_rust(fname.clone(), &content)
                        .map_err(|e| format!("Failed to transpile '{}' to rust: {}", path, e))?;

                let rs_content = rust_code.done()?;
                if !rs_content.is_empty() {
                    // Output to rust/src/ directory
                    // basename() returns file stem (e.g. "main" from "main.at"), so append ".rs"
                    let rs_filename = format!("{}.rs", fname.as_str());
                    let rs_dir = "rust/src";
                    std::fs::create_dir_all(rs_dir)
                        .map_err(|e| format!("Failed to create dir '{}': {}", rs_dir, e))?;
                    let rs_path = format!("{}/{}", rs_dir, rs_filename);
                    std::fs::write(Path::new(&rs_path), rs_content)
                        .map_err(|e| format!("Failed to write Rust file '{}': {}", rs_path, e))?;
                    self.srcs.insert(AutoStr::from(rs_path.clone()));
                    info!("Generated {}", rs_path);
                }
            } else if self.lang.as_str() == "ts" || self.lang.as_str() == "typescript" {
                // TypeScript transpilation via a2ts
                let ts_path = path.as_str().replace(".at", ".ts");
                let result = crate::trans_typescript(path.as_str());
                match result {
                    Ok(msg) => info!("{}", msg),
                    Err(e) => {
                        return Err(
                            format!("Failed to transpile '{}' to typescript: {}", path, e).into(),
                        )
                    }
                }
                self.srcs.insert(AutoStr::from(ts_path.clone()));
            } else {
                // Default to C transpilation
                let mut c_code = transpile_c(fname, &content)
                    .map_err(|e| format!("Failed to transpile '{}' to c: {}", path, e))?;

                // Generate output file paths
                let c_path = path.as_str().replace(".at", ".c");
                let h_path = path.as_str().replace(".at", ".h");

                // Write C file
                let c_content = c_code.done()?;
                if !c_content.is_empty() {
                    std::fs::write(Path::new(c_path.as_str()), c_content)
                        .map_err(|e| format!("Failed to write C file '{}': {}", c_path, e))?;
                    self.srcs.insert(AutoStr::from(c_path.clone()));
                    info!("Generated {}", c_path);
                }

                // Write header file
                if !c_code.header.is_empty() {
                    std::fs::write(Path::new(h_path.as_str()), c_code.header)
                        .map_err(|e| format!("Failed to write header file '{}': {}", h_path, e))?;

                    if let Some(d) = Path::new(h_path.as_str()).parent() {
                        let d: AutoStr = d.to_str().unwrap().into();
                        if !d.is_empty() {
                            self.incs.insert(d);
                        }
                    }
                    info!("Generated {}", h_path);
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message("Transpilation complete");

        // Recursively transpile dependencies
        for dep in self.deps.iter_mut() {
            dep.transpile_auto()?;
        }

        // Plan 328: If this is a Rust target with #[api] endpoints, generate
        // the Axum server code (router.rs + main.rs) after a2r transpilation.
        if self.lang.as_str() == "rust" {
            if let Err(e) = generate_api_server(self) {
                eprintln!("[a2r] API server generation failed: {}", e);
            }
        }

        Ok(())
    }

    /// Transpile Auto files with AutoCache support (Plan 082)
    ///
    /// This method integrates with AutoCache to cache transpilation artifacts.
    /// Cache is controlled by the AUTO_CACHE_ENABLED environment variable.
    fn transpile_auto_with_cache(&mut self) -> AutoResult<()> {
        use indicatif::{ProgressBar, ProgressStyle};

        if self.autos.is_empty() {
            return Ok(());
        }

        // Check if caching is enabled (enabled by default, can be disabled with AUTO_CACHE_ENABLED=false)
        let cache_enabled = std::env::var("AUTO_CACHE_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true); // Default: enabled

        if !cache_enabled {
            // Fallback to non-cached transpilation
            return self.transpile_auto();
        }

        // Initialize cache
        let project_name = self.name.to_string();
        let cache = CTranspilationCache::new(project_name)
            .map_err(|e| format!("Failed to initialize cache: {}", e))?;

        info!("AutoCache enabled for transpilation");

        // Create progress bar for transpilation
        let pb = ProgressBar::new(self.autos.len() as u64);
        pb.set_message("Transpiling Auto files (with cache)");
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("=> "),
        );

        for path in self.autos.iter() {
            pb.set_message(format!("Transpiling {}", path.as_str()));
            info!("Transpiling {}", path.as_str());

            // Read the Auto source file
            let content = std::fs::read_to_string(path.as_str())
                .map_err(|e| format!("Failed to read Auto file '{}': {}", path, e))?;

            // Generate output file paths
            let c_path_str = path.as_str().replace(".at", ".c");
            let h_path_str = path.as_str().replace(".at", ".h");
            let c_path = Path::new(&c_path_str);
            let h_path = Path::new(&h_path_str);

            // Generate module name from path
            let module_name = path.as_str().replace("\\", "/").replace(".at", "");
            let module_name = module_name.replace("/", ":");

            // Check cache
            let cached = cache.get_or_link(&module_name, &content, c_path, Some(h_path));

            match cached {
                Ok(true) => {
                    // Cache hit - files already linked
                    info!("[Cache Hit] {} -> {}", module_name, c_path_str);
                    self.srcs.insert(AutoStr::from(c_path_str.clone()));

                    // Add header directory to includes if header exists
                    if h_path.exists() {
                        if let Some(d) = h_path.parent() {
                            let d: AutoStr = d.to_str().unwrap().into();
                            if !d.is_empty() {
                                self.incs.insert(d);
                            }
                        }
                    }
                }
                Ok(false) => {
                    // Cache miss - transpile normally
                    info!("[Cache Miss] {}", module_name);

                    let fname = AutoPath::new(path).basename();
                    let mut c_code = transpile_c(fname, &content)
                        .map_err(|e| format!("Failed to transpile '{}': {}", path, e))?;

                    // Write C file
                    let c_content = c_code.done()?;
                    if !c_content.is_empty() {
                        std::fs::write(c_path, c_content).map_err(|e| {
                            format!("Failed to write C file '{}': {}", c_path_str, e)
                        })?;
                        self.srcs.insert(AutoStr::from(c_path_str.clone()));
                        info!("Generated {}", c_path_str);
                    }

                    // Write header file
                    let h_content = c_code.header;
                    if !h_content.is_empty() {
                        std::fs::write(h_path, h_content).map_err(|e| {
                            format!("Failed to write header file '{}': {}", h_path_str, e)
                        })?;

                        if let Some(d) = h_path.parent() {
                            let d: AutoStr = d.to_str().unwrap().into();
                            if !d.is_empty() {
                                self.incs.insert(d);
                            }
                        }
                        info!("Generated {}", h_path_str);

                        // Store in cache (only if both .c and .h were generated)
                        if c_path.exists() && h_path.exists() {
                            if let Err(e) =
                                cache.store(&module_name, &content, c_path, Some(h_path))
                            {
                                warn!("Failed to cache artifact '{}': {}", module_name, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    // Cache error - fallback to normal transpilation
                    warn!("Cache error: {}, falling back to transpilation", e);
                    let fname = AutoPath::new(path).basename();
                    let mut c_code = transpile_c(fname, &content)
                        .map_err(|e| format!("Failed to transpile '{}': {}", path, e))?;

                    let c_content = c_code.done()?;
                    if !c_content.is_empty() {
                        std::fs::write(c_path, c_content).map_err(|e| {
                            format!("Failed to write C file '{}': {}", c_path_str, e)
                        })?;
                        self.srcs.insert(AutoStr::from(c_path_str.clone()));
                    }

                    if !c_code.header.is_empty() {
                        std::fs::write(h_path, c_code.header).map_err(|e| {
                            format!("Failed to write header file '{}': {}", h_path_str, e)
                        })?;
                        if let Some(d) = h_path.parent() {
                            let d: AutoStr = d.to_str().unwrap().into();
                            if !d.is_empty() {
                                self.incs.insert(d);
                            }
                        }
                    }
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message("Transpilation complete (with cache)");

        // Recursively transpile dependencies
        for dep in self.deps.iter_mut() {
            dep.transpile_auto_with_cache()?;
        }

        Ok(())
    }

    fn exists(&self) -> bool {
        if self.is_main {
            return true;
        }
        for dir in self.dirs.values() {
            if dir.name != "." && Path::new(dir.to_string().as_str()).is_dir() {
                return true;
            }
        }
        false
    }

    fn downloaded(&self) -> bool {
        let loc = self.location();
        return loc.is_dir();
    }

    fn clean(&self) -> AutoResult<()> {
        if !(self.kind == TargetKind::Dep || self.kind == TargetKind::Device) {
            return Err(format!(
                "Target {} is not a dependency or device, no need to clean",
                self.name
            )
            .into());
        }
        // check if dir is a git directory
        let dir = AutoPath::new(&self.at);
        check_changes(&dir)?;
        println!("target {} is ok to clean!", self.name);
        std::fs::remove_dir_all(&dir.path())?;
        Ok(())
    }

    fn print_info(&self) -> AutoResult<()> {
        println!("Target: {}", self.name);
        println!("Kind: {}", self.kind);
        println!("Root: {}", self.at);
        println!(
            "Dirs: {}",
            self.dirs
                .values()
                .map(|d| d.name.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        Ok(())
    }

    fn scan(&mut self) -> AutoResult<()> {
        use crate::dir::{check_incs, scan_specific_srcs, DirEffects};
        println!(
            "scanning target: {}:{} at {}",
            self.kind, self.name, self.at
        );
        if !self.props.has("defines") {
            let defines = Vec::<AutoStr>::new();
            self.props.set("defines", defines);
        }

        // 0. check for properties that needs to be scanned
        for (_, v) in self.props.iter() {
            if let auto_val::Value::Node(n) = v {
                if n.name == "file" {
                    // TODO: for file node, check it's existence
                }
            }
        }
        // info!("scanning sources for {}", self.name);
        // TODO: why do we need to clone dirs?
        // 1. check dirs for existence
        check_dirs(self)?;

        let mut succ_srcs = std::collections::HashSet::new();
        let mut succ_incs = std::collections::HashSet::new();

        if self.is_scan {
            let mut target_dir = Dir::from_str(".".into(), self.at.clone(), self.kind.clone());
            target_dir.show_headers = self.show_headers;
            target_dir.scan()?;
            succ_srcs.extend(target_dir.srcs.clone());
            succ_incs.extend(target_dir.incs.clone());
        }

        // 2. check for target level specified srcs/incs
        let srcs = self
            .srcs
            .iter()
            .map(|s| s.clone())
            .collect::<Vec<AutoStr>>();
        let scanned_srcs = scan_specific_srcs(&srcs, &self.at)?;
        succ_srcs.extend(scanned_srcs);

        let incs = self
            .incs
            .iter()
            .map(|s| s.clone())
            .collect::<Vec<AutoStr>>();
        let scanned_incs = check_incs(&incs, &self.at)?;
        succ_incs.extend(scanned_incs);

        // 3. for each dir, scan for source files, include dirs
        let mut dirs = std::mem::take(&mut self.dirs);
        for dir in dirs.values_mut() {
            // info!("scanning dir: {}", dir.to_string());
            dir.show_headers = self.show_headers;
            dir.scan()?;
            // NOTE: srcs in dirs should be included into the target in needed builder
            // succ_srcs.extend(dir.srcs.clone());
            succ_incs.extend(dir.incs.clone());
        }
        self.dirs = dirs;

        self.srcs = succ_srcs.into_iter().collect();
        self.incs = succ_incs.into_iter().collect();
        // dirs are moved out and scanned, with all infomation merged into the target
        // so we don't need to store dirs anymore
        // self.dirs is empty now

        // 4. scan deps of this target

        for dep in &mut self.deps {
            dep.scan()?;
        }

        scan_auto(self)?;

        Ok(())
    }
}

/// scan 的私有辅助（原 Target 私有方法，效果半留此）。

/// 原 Target 私有效果辅助（自由函数形态，供 TargetEffects 调用）。
fn check_dirs(t: &Target) -> AutoResult<()> {
    // 1. check prop `dirs` for existing dirs
    for dir in t.dirs.values() {
        let path = dir.to_string();
        let path = Path::new(path.as_str());
        if !path.is_dir() {
            return Err(format!("checking dir: {} is not a directory", path.display()).into());
        }
    }
    Ok(())
}

fn scan_auto(t: &mut Target) -> AutoResult<()> {
    let mut auto_files = Vec::new();
    let root = Path::new(t.at.as_str());
    scan_auto_dir(t, root, &mut auto_files);
    t.autos.extend(auto_files);
    Ok(())
}

fn scan_auto_dir(t: &Target, dir: &Path, files: &mut Vec<AutoStr>) {
    // PLAN-809 T-05：排除表/下潜过滤规则单源在 auto_man_core::selection。
    if let Ok(entries) = dir.read_dir() {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("at")) {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if auto_man_core::is_excluded_auto_config(name) {
                    continue;
                }
                files.push(path.unified());
            } else if path.is_dir() && t.recurse {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if !auto_man_core::auto_recurse_into(&dir_name) {
                    continue;
                }
                scan_auto_dir(t, &path, files);
            }
        }
    }
}

/// Plan 328: Generate Axum HTTP server code from #[api] endpoints.
fn generate_api_server(t: &Target) -> Result<(), String> {
    use auto_lang_core::api::targets::AxumGenerator;
    use auto_lang_core::api::targets::TauriGenerator;
    use auto_lang_core::api::ApiExtractor;
    use auto_lang_core::api::ApiModule; // Plan 328 IPC

    // Collect #[api] endpoints from all .at source files
    let mut combined_module = ApiModule::new("api".to_string());
    let extractor = ApiExtractor::new();

    for path in t.autos.iter() {
        let content = std::fs::read_to_string(path.as_str())
            .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
        // Parse the source file to get AST
        let mut parser = auto_lang_core::parser::Parser::new(&content);
        let code = match parser.parse() {
            Ok(c) => c,
            Err(_) => continue, // skip unparseable files
        };
        let api_module = extractor.extract("api", &code.stmts);
        for endpoint in api_module.endpoints {
            combined_module.add_endpoint(endpoint);
        }
        for typ in api_module.types {
            combined_module.types.push(typ);
        }
    }

    if combined_module.endpoints.is_empty() {
        return Ok(()); // No #[api] endpoints, skip server generation
    }

    let rs_dir = "rust/src";
    std::fs::create_dir_all(rs_dir)
        .map_err(|e| format!("Failed to create dir '{}': {}", rs_dir, e))?;

    // Plan 328 IPC: detect Tauri frontend by checking for tauri.conf.json
    // in the project root. If present → IPC commands (invoke_handler).
    // Otherwise → Axum HTTP server.
    let at_str = t.at.to_string();
    let project_root = std::path::Path::new(&at_str)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let is_tauri = project_root.join("tauri.conf.json").exists()
        || project_root.join("src-tauri").exists()
        || project_root.join("tauri/tauri.conf.json").exists();

    if is_tauri {
        eprintln!(
            "[a2r] Generating Tauri IPC commands with {} endpoint(s)",
            combined_module.endpoints.len()
        );
        let gen = TauriGenerator::new();
        let commands_code = gen.generate_full(&combined_module);
        std::fs::write(format!("{}/commands.rs", rs_dir), commands_code)
            .map_err(|e| format!("Failed to write commands.rs: {}", e))?;
        let main_code = gen.generate_server_main(&combined_module.endpoints);
        std::fs::write(format!("{}/main.rs", rs_dir), main_code)
            .map_err(|e| format!("Failed to write main.rs: {}", e))?;
        eprintln!("[a2r] Generated rust/src/commands.rs and rust/src/main.rs (Tauri IPC)");
    } else {
        eprintln!(
            "[a2r] Generating Axum HTTP server with {} endpoint(s)",
            combined_module.endpoints.len()
        );
        let gen = AxumGenerator::new();
        let router_code = gen.generate_full(&combined_module);
        std::fs::write(format!("{}/router.rs", rs_dir), router_code)
            .map_err(|e| format!("Failed to write router.rs: {}", e))?;
        let main_code = gen.generate_server_main();
        std::fs::write(format!("{}/main.rs", rs_dir), main_code)
            .map_err(|e| format!("Failed to write main.rs: {}", e))?;
        eprintln!("[a2r] Generated rust/src/router.rs and rust/src/main.rs (Axum HTTP)");
    }
    Ok(())
}

/// Transpile a multi-file Rust project using transpile_rust_project().
///
/// This handles cross-file module discovery, shared TypeStore, and proper
/// mod declarations — unlike single-file transpile_rust().
fn transpile_rust_project_multi(t: &mut Target) -> AutoResult<()> {
    info!(
        "Transpiling multi-file Rust project ({} files)",
        t.autos.len()
    );

    // Find entry file: prefer main.at, otherwise use first .at file
    let entry = t
        .autos
        .iter()
        .find(|p| p.as_str().ends_with("main.at"))
        .cloned()
        .unwrap_or_else(|| t.autos[0].clone());

    info!("Entry file: {}", entry.as_str());

    // Call transpile_rust_project() which discovers all modules
    let files = auto_lang_core::trans::rust::transpile_rust_project(entry.as_str())
        .map_err(|e| format!("transpile_rust_project failed: {}", e))?;

    info!("Generated {} files", files.len());

    // Write all generated files to rust/src/, skipping Cargo.toml
    let rs_dir = "rust/src";
    std::fs::create_dir_all(rs_dir)
        .map_err(|e| format!("Failed to create dir '{}': {}", rs_dir, e))?;

    for (name, (content, _source_map)) in &files {
        if name == "Cargo.toml" {
            info!("Skipping Cargo.toml (CargoBuilder will generate it)");
            continue;
        }

        // transpile_rust_project returns paths like "main.rs", "runtime/mod.rs", "api/anthropic.rs"
        let full_path = format!("{}/{}", rs_dir, name);

        // Create parent directories for subdirectory modules
        if let Some(parent) = Path::new(&full_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir '{}': {}", parent.display(), e))?;
        }

        std::fs::write(Path::new(&full_path), content)
            .map_err(|e| format!("Failed to write Rust file '{}': {}", full_path, e))?;
        t.srcs.insert(AutoStr::from(full_path.clone()));
        info!("Generated {}", full_path);
    }

    // Update lib.rs to ensure all subdirectory modules have pub mod declarations.
    // The transpiler may generate subdirectory modules (tools/, forge/, relay/, etc.)
    // that aren't listed in the hand-written lib.rs.
    let lib_rs_path = format!("{}/lib.rs", rs_dir);
    if Path::new(&lib_rs_path).exists() {
        let lib_content = std::fs::read_to_string(&lib_rs_path)
            .map_err(|e| format!("Failed to read lib.rs: {}", e))?;

        // Find all subdirectories in rust/src/ that contain mod.rs
        let mut declared: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Extract existing pub mod declarations
        for line in lib_content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("pub mod ") {
                if let Some(name) = rest.strip_suffix(';') {
                    declared.insert(name.trim().to_string());
                }
            } else if let Some(rest) = trimmed.strip_prefix("mod ") {
                if let Some(name) = rest.strip_suffix(';') {
                    declared.insert(name.trim().to_string());
                }
            }
        }

        // Scan for directories with mod.rs that aren't declared
        let src_path = Path::new(rs_dir);
        let mut missing_mods: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(src_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        // Check if directory has a mod.rs or is a valid module
                        let has_mod_rs = path.join("mod.rs").exists();
                        let has_rs_files = path
                            .read_dir()
                            .map(|mut entries| {
                                entries.any(|e| {
                                    e.map(|e| {
                                        e.path().extension().map(|ext| ext == "rs").unwrap_or(false)
                                    })
                                    .unwrap_or(false)
                                })
                            })
                            .unwrap_or(false);
                        if (has_mod_rs || has_rs_files) && !declared.contains(name) {
                            missing_mods.push(name.to_string());
                        }
                    }
                }
            }
        }

        // Also check for standalone .rs files not declared
        if let Ok(entries) = std::fs::read_dir(src_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "rs").unwrap_or(false) {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        if name != "lib" && name != "main" && !declared.contains(name) {
                            missing_mods.push(name.to_string());
                        }
                    }
                }
            }
        }

        if !missing_mods.is_empty() {
            missing_mods.sort();
            // Insert pub mod declarations at the beginning of lib.rs
            let mut new_lines: Vec<String> = Vec::new();
            for m in &missing_mods {
                new_lines.push(format!("pub mod {};", m));
            }
            new_lines.push(String::new());
            let updated = format!("{}\n{}", new_lines.join("\n"), lib_content);
            std::fs::write(&lib_rs_path, &updated)
                .map_err(|e| format!("Failed to update lib.rs: {}", e))?;
            info!(
                "Updated lib.rs with {} missing module declarations: {:?}",
                missing_mods.len(),
                missing_mods
            );
        }
    }

    Ok(())
}

#[derive(Tabled)]
pub struct TargetStatus {
    pub name: AutoStr,
    pub kind: AutoStr,
    pub at: AutoStr,
    pub dirs: AutoStr,
}

impl TargetStatus {
    pub fn from(target: &Target) -> Self {
        Self {
            name: target.name.clone(),
            kind: target.kind.to_string().into(),
            at: target.at.clone(),
            dirs: target
                .dirs
                .values()
                .map(|d| d.name.to_string())
                .collect::<Vec<String>>()
                .join(", ")
                .into(),
        }
    }
}
