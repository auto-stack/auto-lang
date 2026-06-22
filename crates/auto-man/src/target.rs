use crate::dir::{check_incs, scan_specific_srcs};
use crate::git::check_changes;
use crate::group::Group;
use crate::Dir;
use crate::{AutoResult, Port};
use auto_lang::trans::c::transpile_c;
use auto_lang::util::*;
use auto_lang::Atom;
use auto_val::{Array, Obj};
use auto_val::{AutoPath, AutoStr, Node, Value};
use log::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::path::Path;
use tabled::Tabled;

// Plan 082: AutoCache integration
use auto_cache::CTranspilationCache;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TargetKind {
    App,    // 应用程序
    Lib,    // 静态代码库
    Bag,    // 普通代码包，作为拼图的一部分，而不少单独的编译目标
    Dep,    // 依赖库
    Device, // 设备资源
    Test,   // 测试代码
}

impl TargetKind {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "app" => Ok(TargetKind::App),
            "lib" => Ok(TargetKind::Lib),
            "bag" => Ok(TargetKind::Bag),
            "dep" => Ok(TargetKind::Dep),
            "device" => Ok(TargetKind::Device),
            "test" => Ok(TargetKind::Test),
            _ => Err(format!(
                "Invalid target kind: {}. Valid options are: app, lib, bag, dep, device, test",
                s
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TargetOrigin {
    Local,
    Git,
    Index,
}

impl fmt::Display for TargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetKind::App => write!(f, "App"),
            TargetKind::Lib => write!(f, "Lib"),
            TargetKind::Dep => write!(f, "Dep"),
            TargetKind::Device => write!(f, "Device"),
            TargetKind::Test => write!(f, "Test"),
            TargetKind::Bag => write!(f, "Bag"),
        }
    }
}

#[derive(Clone)]
pub struct Target {
    pub name: AutoStr,
    pub version: AutoStr,
    pub kind: TargetKind,
    pub lang: AutoStr,
    pub origin: TargetOrigin,
    pub from: AutoStr, // where the target is downloaded from
    pub at: AutoStr,   // root dir of the target, including its name
    pub rename: AutoStr,
    pub is_scan: bool,
    pub recurse: bool,
    pub show_headers: bool,
    pub dirs: HashMap<AutoStr, Dir>,
    // Auto Source files
    pub autos: Vec<AutoStr>,
    // C Source files
    pub srcs: HashSet<AutoStr>,
    // C header files/folders
    pub incs: HashSet<AutoStr>,
    pub selects: HashSet<AutoStr>,
    pub skips: HashSet<AutoStr>,
    pub links: Vec<Node>,
    pub is_resolved: bool,
    pub is_main: bool,
    pub props: Obj,
    pub deps: Vec<Target>,
    pub dep_names: Vec<AutoStr>,
    pub all_links: Vec<Node>,
    pub all_deps: Vec<Target>,
    pub rel: AutoStr,
    pub defines: Vec<AutoStr>,
    pub port: Port,
}

pub fn set_to_vec<T: Ord>(set: HashSet<T>) -> Vec<T> {
    let mut vec: Vec<T> = set.into_iter().collect();
    vec.sort();
    vec
}

trait VecExt<T> {
    fn to_set(self) -> HashSet<T>;
}

impl<T: Hash + Eq> VecExt<T> for Vec<T> {
    fn to_set(self) -> HashSet<T> {
        self.into_iter().collect()
    }
}

pub fn get_root_dir(name: &AutoStr, kind: &TargetKind, pac: &AutoStr) -> AutoStr {
    // 如果target的名称和pac名称相同，说明是主target，使用当前目录
    if name == pac {
        return AutoStr::from(".");
    }

    // 对于 app 类型，如果名称是 "main" 或与 pac 相同，
    // 先尝试 apps/<name>，如果不存在则回退到当前目录 "."
    if *kind == TargetKind::App && (name.as_str() == "main" || name == pac) {
        let app_dir = format!("apps/{}", name);
        if Path::new(&app_dir).is_dir() {
            return AutoStr::from(app_dir);
        }
        return AutoStr::from(".");
    }

    // 否则，使用libs/apps/deps中的目录
    let target_dir = match kind {
        TargetKind::App => "apps",
        TargetKind::Lib => "libs",
        TargetKind::Bag => "bags",
        TargetKind::Dep => "deps",
        TargetKind::Device => "devices",
        TargetKind::Test => "tests",
    };
    AutoStr::from(format!("{}/{}", target_dir, name))
}

fn dir_filter(name: &AutoStr, selects: &HashSet<AutoStr>, skips: &HashSet<AutoStr>) -> bool {
    if !selects.is_empty() && !selects.contains(name) {
        return false;
    }
    if !skips.is_empty() && skips.contains(name) {
        return false;
    }
    return true;
}

impl Target {
    pub fn new(name: impl Into<AutoStr>, kind: TargetKind) -> Self {
        Self {
            name: name.into(),
            version: AutoStr::new(),
            kind,
            lang: "c".into(),
            origin: TargetOrigin::Index,
            from: AutoStr::new(),
            at: AutoStr::new(),
            rename: AutoStr::default(),
            is_scan: true,
            recurse: false,
            show_headers: false,
            dirs: HashMap::new(),
            autos: Vec::new(),
            srcs: HashSet::new(),
            incs: HashSet::new(),
            selects: HashSet::new(),
            skips: HashSet::new(),
            links: Vec::new(),
            is_resolved: false,
            is_main: false,
            props: Obj::new(),
            deps: Vec::new(),
            dep_names: Vec::new(),
            all_links: Vec::new(),
            all_deps: Vec::new(),
            rel: AutoStr::new(),
            defines: Vec::new(),
            port: Port::default(),
        }
    }

    pub fn from(node: Node, pac: AutoStr) -> Self {
        use crate::node_ext::NodeExt;

        // Extract basic information
        let (name, version) = Self::extract_name_and_version(&node);
        let kind = Self::extract_kind(&node);
        let origin = Self::extract_origin(&node);
        let from = node.get_str_or("from", "");

        // Extract flags using NodeExt
        let is_scan = node.get_bool_or("scan", true);
        let recurse = node.get_bool_or("recurse", false);

        // Calculate location
        let (at, rename) = Self::calculate_location(&node, &name, &kind, &pac);

        // Extract directory information
        let dirs = Self::extract_directories(&node, &at, &rename, &kind);

        // Extract source files and includes using NodeExt
        let srcs = node.get_str_vec_or("srcs").into_iter().collect();
        let incs = node.get_str_vec_or("incs").into_iter().collect();

        // Extract links and other properties
        let links = node.get_kids("link");
        let props = node.props_clone();
        let selects = Self::extract_selects(&node);
        let skips = node.get_str_vec_or("skips").into_iter().collect();

        let is_main = name == pac;

        // Extract language property, defaulting to "c"
        let lang = node.get_str_or("lang", "c").into();

        Self {
            name,
            version,
            kind,
            lang,
            origin,
            from,
            at,
            rename,
            is_scan,
            recurse,
            show_headers: false,
            dirs,
            autos: Vec::new(),
            srcs,
            incs,
            selects,
            skips,
            links,
            is_resolved: false,
            is_main,
            props,
            deps: Vec::new(),
            dep_names: Vec::new(),
            all_links: Vec::new(),
            all_deps: Vec::new(),
            rel: AutoStr::new(),
            defines: Vec::new(),
            port: Port::default(),
        }
    }

    /// Extract name and version from node
    fn extract_name_and_version(node: &Node) -> (AutoStr, AutoStr) {
        let mut name = AutoStr::new();
        let mut version = AutoStr::new();

        if !node.id.is_empty() {
            name = node.id.clone();
        } else if !node.args.is_empty() {
            name = node.args.args[0].to_astr();
            if node.args.args.len() > 1 {
                version = node.args.args[1].to_astr();
            }
        }

        if version.is_empty() {
            version = if node.has_prop("version") {
                node.get_prop("version").to_astr()
            } else {
                "latest".into()
            };
        }

        (name, version)
    }

    /// Extract target kind from node name
    fn extract_kind(node: &Node) -> TargetKind {
        TargetKind::from_str(&node.name).unwrap_or_else(|e| {
            panic!("Failed to parse target kind in node '{}': {}", node.name, e)
        })
    }

    /// Extract origin (local/git/index) from node
    fn extract_origin(node: &Node) -> TargetOrigin {
        let mut origin = TargetOrigin::Index;

        if let Some(arg) = &node.args.get_arg(&"kind".into()) {
            let mode = arg.get_val().to_astr();
            match mode.as_str() {
                "local" => origin = TargetOrigin::Local,
                "git" => origin = TargetOrigin::Git,
                _ => {}
            }
        }

        origin
    }

    /// Calculate the target location and rename
    fn calculate_location(
        node: &Node,
        name: &AutoStr,
        kind: &TargetKind,
        pac: &AutoStr,
    ) -> (AutoStr, AutoStr) {
        // Get parent property if exists
        let parent = if node.has_prop("parent") {
            node.get_prop("parent").to_astr()
        } else {
            AutoStr::default()
        };

        // Get at property or calculate default
        let mut at = if node.has_prop("at") {
            node.get_prop("at").to_astr()
        } else {
            get_root_dir(name, kind, pac)
        };

        // Apply parent if specified
        if !parent.is_empty() {
            at = AutoPath::new(parent.clone()).join(name.clone()).to_astr();
        }

        // Calculate rename from filename
        let mut rename = AutoPath::new(at.clone()).filename();
        if rename.is_empty() {
            rename = name.clone();
        }

        (at, rename)
    }

    /// Extract and process directory information from node
    fn extract_directories(
        node: &Node,
        at: &AutoStr,
        rename: &AutoStr,
        kind: &TargetKind,
    ) -> HashMap<AutoStr, Dir> {
        let selects = Self::extract_selects(node);
        let skips = node.get_prop("skips").to_str_vec().to_set();

        // Process dirs property (array of directory names)
        let dirs_prop = node.get_prop("dirs").to_str_vec();
        let mut dirs: Vec<Dir> = dirs_prop
            .into_iter()
            .filter(|d| dir_filter(d, &selects, &skips))
            .map(|d| Dir::from_str(d, at.clone(), kind.clone()))
            .collect();

        // Process dir nodes
        let dir_nodes = node.nodes(&"dir");
        let dirs_from_nodes: Vec<Dir> = dir_nodes
            .into_iter()
            .filter(|d| dir_filter(&d.main_arg().to_astr(), &selects, &skips))
            .map(|n| Dir::from_node(n, at.clone(), kind.clone()))
            .collect();
        dirs.extend(dirs_from_nodes);

        // Set logical path for all directories
        for d in &mut dirs {
            d.set_lpath(rename.clone());
        }

        // Convert to hashmap
        dirs.into_iter().map(|d| (d.name.clone(), d)).collect()
    }

    /// Extract selects and defaults from node
    fn extract_selects(node: &Node) -> HashSet<AutoStr> {
        let mut selects = node.get_prop("selects").to_str_vec().to_set();
        let defaults = node.get_prop("defaults").to_str_vec().to_set();

        if !defaults.is_empty() {
            selects.extend(defaults);
        }

        selects
    }

    pub fn location(&self) -> AutoPath {
        AutoPath::new(self.at.clone())
    }

    pub fn local_name(&self) -> AutoStr {
        self.rename.clone()
    }

    pub fn config_name(&self) -> AutoStr {
        match self.kind {
            TargetKind::Device => "device.at".into(),
            _ => "pac.at".into(),
        }
    }

    pub fn set_parent(&mut self, parent: impl Into<AutoStr>) {
        self.at = AutoPath::new(parent.into()).join(&self.rename).to_astr();
        // update root of each sub dirs
        for dir in self.dirs.iter_mut() {
            dir.1.update_root(self.at.clone());
        }
    }

    pub fn parent(&self) -> AutoStr {
        AutoPath::new(self.at.clone()).parent().to_astr()
    }

    pub fn all_incs(&self) -> Vec<AutoStr> {
        let mut all_incs = Vec::new();
        all_incs.extend(self.incs.clone());
        for target in &self.deps {
            all_incs.extend(target.incs.clone());
        }
        all_incs.sort();
        all_incs
    }

    fn get_extra_output(&self, extra_output: &str) -> Obj {
        let mut o = Obj::new();
        match extra_output {
            "srec" => {
                o.set("name", format!("{}.srec", self.local_name()));
                o.set("format", 0);
                o.set("enable", 1);
            }
            "hex" => {
                o.set("name", format!("{}.hex", self.local_name()));
                o.set("format", 1);
                o.set("enable", 1);
            }
            "txt" => {
                o.set("name", format!("{}.txt", self.local_name()));
                o.set("format", 2);
                o.set("enable", 1);
            }
            "bin" => {
                o.set("name", format!("{}.bin", self.local_name()));
                o.set("format", 3);
                o.set("enable", 1);
            }
            "sim" => {
                o.set("name", format!("{}.sim", self.local_name()));
                o.set("format", 4);
                o.set("enable", 1);
            }
            _ => {
                o.set("name", format!("{}.srec", self.local_name()));
                o.set("format", 0);
                o.set("enable", 0);
            }
        };
        o
    }

    pub fn is_major(&self) -> bool {
        self.kind == TargetKind::App || self.kind == TargetKind::Lib
    }

    pub fn to_atom(&self) -> Atom {
        Atom::node(self.to_node())
    }

    pub fn to_node(&self) -> Node {
        let node_type = match self.kind {
            TargetKind::App => "app",
            TargetKind::Lib => "lib",
            TargetKind::Bag => "bag",
            TargetKind::Dep => "dep",
            TargetKind::Device => "device",
            TargetKind::Test => "test",
        };
        let mut node = Node::new(node_type);
        node.args.add_pos(self.name.clone());
        node.id = self.name.clone();

        // Set properties directly on the node (replacing the deprecated body field)
        node.set_prop("kind", node_type);
        node.set_prop("relative_path", self.rel.clone());
        node.set_prop("at", self.at.clone());
        node.set_prop("name", self.name.clone());
        node.set_prop("id", self.name.clone());
        node.set_prop("root", self.at.clone());
        node.set_prop("srcs", set_to_vec(self.srcs.clone()));
        node.set_prop("incs", set_to_vec(self.incs.clone()));
        node.set_prop("all_incs", self.all_incs());
        node.set_prop("skips", set_to_vec(self.skips.clone()));

        // Add custom properties
        for (p, v) in self.props.iter() {
            match v {
                Value::Str(s) => {
                    node.set_prop(p.clone(), s.clone());
                }
                Value::Node(n) => {
                    if n.name == "file" {
                        // for file node, add target's root to the file's path
                        let path = n.main_arg();
                        let path = Path::new(self.at.as_str()).join(path.as_str());
                        node.set_prop(p.clone(), path.unified());
                    } else {
                        node.set_prop(p.clone(), n.to_astr());
                    }
                }
                _ => {
                    node.set_prop(p.clone(), v.clone());
                }
            }
        }

        // Add dirs as property
        let mut dirs = Array::new();
        for dir in self.dirs.values() {
            let array_val: Vec<Value> = dir.srcs.iter().map(|s| Value::Str(s.clone())).collect();
            let mut obj = Obj::new();
            obj.set("name", Value::Str(dir.name.clone()));
            obj.set("srcs", array_val);
            obj.set("lpath", dir.lpath.clone());
            obj.set("rpath", dir.rpath.clone());
            dirs.push(obj);
        }
        node.set_prop("dirs", dirs);

        // Add deps as kids
        for dep in &self.deps {
            let n = dep.to_node();
            node.add_kid(n);
        }

        if !self.deps.iter().any(|t| t.kind == TargetKind::Device) {
            node.set_prop("devices", Value::empty_array());
        }

        if !self.deps.iter().any(|t| t.kind == TargetKind::Dep) {
            node.set_prop("deps", Value::empty_array());
        }

        if !self.deps.iter().any(|t| t.kind == TargetKind::Bag) {
            node.set_prop("bags", Value::empty_array());
        }

        // Add defines property
        node.set_prop("defines", Value::str_array(self.defines.clone()));

        // For IAR builder
        if self.is_major() && self.port.builder == "iar" {
            node.set_prop("groups", self.groups().to_xml());

            let mut xo = "none".into();
            if self.props.has("extra_output") {
                xo = self.props.get_str_or("extra_output", xo);
            };
            node.set_prop("extout", self.get_extra_output(&xo));
        }

        node
    }

    pub fn get_dir(&self, name: impl Into<AutoStr>) -> Option<&Dir> {
        let name = name.into();
        self.dirs.get(&name)
    }

    pub fn has_auto(&self) -> bool {
        !self.autos.is_empty()
    }

    pub fn transpile_auto(&mut self) -> AutoResult<()> {
        use indicatif::{ProgressBar, ProgressStyle};

        if self.autos.is_empty() {
            return Ok(());
        }

        // Multi-file Rust project: use transpile_rust_project() for cross-module support
        if self.lang.as_str() == "rust" && self.autos.len() > 1 {
            return self.transpile_rust_project_multi();
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
                let mut rust_code = auto_lang::trans::rust::transpile_rust(fname.clone(), &content)
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
                let result = auto_lang::trans_typescript(path.as_str());
                match result {
                    Ok(msg) => info!("{}", msg),
                    Err(e) => return Err(format!("Failed to transpile '{}' to typescript: {}", path, e).into()),
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

            // TODO: Plan 092 - Add code paks support with new AutoVM
            // Previously: uni.borrow().code_paks
            // for (_sid, pak) in uni.borrow().code_paks.iter() {
            //     let inc = AutoPath::new(pak.header.clone()).parent().parent();
            //     if inc.is_dir() {
            //         self.incs.insert(inc.to_astr());
            //     }
            //     self.srcs.insert(pak.cfile.clone());
            // }

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
            if let Err(e) = self.generate_api_server() {
                eprintln!("[a2r] API server generation failed: {}", e);
            }
        }

        Ok(())
    }

    /// Plan 328: Generate Axum HTTP server code from #[api] endpoints.
    ///
    /// Parses all .at source files, extracts #[api] endpoints via ApiExtractor,
    /// then uses AxumGenerator to produce router.rs (handlers + router) and
    /// main.rs (server entry point). These are written to rust/src/ alongside
    /// the a2r-generated business logic files.
    fn generate_api_server(&self) -> Result<(), String> {
        use auto_lang::api::{ApiExtractor, ApiModule};
        use auto_lang::api::targets::AxumGenerator;
        use auto_lang::api::targets::TauriGenerator; // Plan 328 IPC
        use auto_lang::api::TargetGenerator;

        // Collect #[api] endpoints from all .at source files
        let mut combined_module = ApiModule::new("api".to_string());
        let extractor = ApiExtractor::new();

        for path in self.autos.iter() {
            let content = std::fs::read_to_string(path.as_str())
                .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
            // Parse the source file to get AST
            let mut parser = auto_lang::parser::Parser::new(&content);
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
        let at_str = self.at.to_string();
        let project_root = std::path::Path::new(&at_str)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let is_tauri = project_root.join("tauri.conf.json").exists()
            || project_root.join("src-tauri").exists()
            || project_root.join("tauri/tauri.conf.json").exists();

        if is_tauri {
            eprintln!("[a2r] Generating Tauri IPC commands with {} endpoint(s)", combined_module.endpoints.len());
            let gen = TauriGenerator::new();
            let commands_code = gen.generate_full(&combined_module);
            std::fs::write(format!("{}/commands.rs", rs_dir), commands_code)
                .map_err(|e| format!("Failed to write commands.rs: {}", e))?;
            let main_code = gen.generate_server_main(&combined_module.endpoints);
            std::fs::write(format!("{}/main.rs", rs_dir), main_code)
                .map_err(|e| format!("Failed to write main.rs: {}", e))?;
            eprintln!("[a2r] Generated rust/src/commands.rs and rust/src/main.rs (Tauri IPC)");
        } else {
            eprintln!("[a2r] Generating Axum HTTP server with {} endpoint(s)", combined_module.endpoints.len());
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
    fn transpile_rust_project_multi(&mut self) -> AutoResult<()> {
        info!("Transpiling multi-file Rust project ({} files)", self.autos.len());

        // Find entry file: prefer main.at, otherwise use first .at file
        let entry = self.autos.iter()
            .find(|p| p.as_str().ends_with("main.at"))
            .cloned()
            .unwrap_or_else(|| self.autos[0].clone());

        info!("Entry file: {}", entry.as_str());

        // Call transpile_rust_project() which discovers all modules
        let files = auto_lang::trans::rust::transpile_rust_project(entry.as_str())
            .map_err(|e| format!("transpile_rust_project failed: {}", e))?;

        info!("Generated {} files", files.len());

        // Write all generated files to rust/src/, skipping Cargo.toml
        let rs_dir = "rust/src";
        std::fs::create_dir_all(rs_dir)
            .map_err(|e| format!("Failed to create dir '{}': {}", rs_dir, e))?;

        for (name, content) in &files {
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
            self.srcs.insert(AutoStr::from(full_path.clone()));
            info!("Generated {}", full_path);
        }

        // Update lib.rs to ensure all subdirectory modules have pub mod declarations.
        // The transpiler may generate subdirectory modules (tools/, forge/, relay/, etc.)
        // that aren't listed in the hand-written lib.rs.
        let lib_rs_path = format!("{}/lib.rs", rs_dir);
        if Path::new(&lib_rs_path).exists() {
            let mut lib_content = std::fs::read_to_string(&lib_rs_path)
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
                            let has_rs_files = path.read_dir()
                                .map(|mut entries| entries.any(|e| e.map(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false)).unwrap_or(false)))
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
                info!("Updated lib.rs with {} missing module declarations: {:?}", missing_mods.len(), missing_mods);
            }
        }

        Ok(())
    }

    /// Transpile Auto files with AutoCache support (Plan 082)
    ///
    /// This method integrates with AutoCache to cache transpilation artifacts.
    /// Cache is controlled by the AUTO_CACHE_ENABLED environment variable.
    pub fn transpile_auto_with_cache(&mut self) -> AutoResult<()> {
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

                    // TODO: Plan 092 - Add code paks support with new AutoVM
                    // Previously: uni.borrow().code_paks
                    // for (_sid, pak) in uni.borrow().code_paks.iter() {
                    //     let inc = AutoPath::new(pak.header.clone()).parent().parent();
                    //     if inc.is_dir() {
                    //         self.incs.insert(inc.to_astr());
                    //     }
                    //     self.srcs.insert(pak.cfile.clone());
                    // }
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

                    // TODO: Plan 092 - Add code paks support with new AutoVM
                    // Previously: uni.borrow().code_paks
                    // for (_sid, pak) in uni.borrow().code_paks.iter() {
                    //     let inc = AutoPath::new(pak.header.clone()).parent().parent();
                    //     if inc.is_dir() {
                    //         self.incs.insert(inc.to_astr());
                    //     }
                    //     self.srcs.insert(pak.cfile.clone());
                    // }
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

    pub fn exists(&self) -> bool {
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

    pub fn downloaded(&self) -> bool {
        let loc = self.location();
        return loc.is_dir();
    }

    // TODO: only allow args to be overrided
    pub fn merge_link(&mut self, link: &Node) {
        // currently only support normal props override
        for (k, v) in link.props_iter() {
            self.props.set(k.clone(), v.clone());
        }
    }

    pub fn merge(&mut self, other: &Target) {
        self.props.merge(&other.props);
        self.autos.extend(other.autos.clone());
        // TODO: merge dirs
        for (k, v) in other.dirs.iter() {
            let mut dir = v.clone();
            // check new root dir
            let path = Path::new(self.at.as_str()).join(dir.at.as_str());
            if path.is_dir() {
                dir.append_root(self.at.clone());
            }
            self.dirs.insert(k.clone(), dir);
        }
        self.srcs.extend(other.srcs.clone());
        self.incs.extend(other.incs.clone());
        self.links.extend(other.links.clone());
        self.skips.extend(other.skips.clone());
    }

    pub fn set_rel(&mut self, rel: AutoStr) {
        self.rel = rel;
    }

    pub fn set_port(&mut self, port: Port) {
        self.port = port;
    }

    pub fn set_defines(&mut self, defines: Vec<AutoStr>) {
        self.defines = defines;
    }

    /// If the target has links, copy link's target info
    pub fn check_links(&mut self) -> AutoResult<Vec<Node>> {
        // 2. If the targets links to a dep target, recursively resolve the linked target
        let links = std::mem::take(&mut self.links);
        Ok(links)
    }

    pub fn apply_link(&mut self, _link: &Node) {}

    // fn check_root(&mut self) -> AutoResult<()> {
    //     let root = self.get_root_dir();
    //     info!("{} root: {}", self.name, root.as_str());
    //     self.root = root;
    //     Ok(())
    // }

    fn check_dirs(&mut self) -> AutoResult<()> {
        // 1. check prop `dirs` for existing dirs
        for dir in self.dirs.values() {
            let path = dir.to_string();
            let path = Path::new(path.as_str());
            if !path.is_dir() {
                return Err(format!("checking dir: {} is not a directory", path.display()).into());
            }
        }
        Ok(())
    }

    pub fn scan(&mut self) -> AutoResult<()> {
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
            if let Value::Node(n) = v {
                if n.name == "file" {
                    // TODO: for file node, check it's existence
                }
            }
        }
        // info!("scanning sources for {}", self.name);
        // TODO: why do we need to clone dirs?
        // 1. check dirs for existence
        self.check_dirs()?;

        let mut succ_srcs = HashSet::new();
        let mut succ_incs = HashSet::new();

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

        self.scan_auto()?;

        Ok(())
    }

    fn scan_auto(&mut self) -> AutoResult<()> {
        let mut auto_files = Vec::new();
        let root = Path::new(self.at.as_str());
        self.scan_auto_dir(root, &mut auto_files);
        self.autos.extend(auto_files);
        Ok(())
    }

    fn scan_auto_dir(&self, dir: &Path, files: &mut Vec<AutoStr>) {
        if let Ok(entries) = dir.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("at")) {
                    if path.file_name() == Some(std::ffi::OsStr::new("pac.at"))
                        || path.file_name() == Some(std::ffi::OsStr::new("device.at"))
                        || path.file_name() == Some(std::ffi::OsStr::new("os.at"))
                    {
                        continue;
                    }
                    files.push(path.unified());
                } else if path.is_dir() && self.recurse {
                    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if dir_name.starts_with('.') || dir_name == "rust" {
                        continue;
                    }
                    self.scan_auto_dir(&path, files);
                }
            }
        }
    }

    pub fn clean(&self) -> AutoResult<()> {
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

    pub fn print_info(&self) -> AutoResult<()> {
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

    fn group_target(&self, g: &mut Group, t: &Target) {
        // set files
        for src in &t.srcs {
            let fpath = format!("$PROJ_DIR${}\\{}", &self.rel, src);
            g.files.push(crate::group::File { name: fpath.into() })
        }
        // set dirs
        for (_n, d) in &t.dirs {
            let dg = g.mut_kid(&d.name);
            self.group_dir(dg, d);
        }
        if self.name != t.name {
            // set deps
            for dep in &t.deps {
                let dg = g.mut_kid(&dep.name);
                self.group_target(dg, dep);
            }
        }
    }

    fn group_dir(&self, dg: &mut Group, d: &Dir) {
        for s in &d.srcs {
            let fpath = format!("$PROJ_DIR${}\\{}", &self.rel, s);
            dg.files.push(crate::group::File { name: fpath.into() })
        }
        for s in &d.dirs {
            let sg = dg.mut_kid(&s.name);
            self.group_dir(sg, s);
        }
    }

    fn groups(&self) -> Group {
        let mut group = Group::new(self.name.as_str());
        let tg = group.mut_kid_path(&self.location());
        self.group_target(tg, self);
        for t in &self.deps {
            let loc = t.location();
            let g = group.mut_kid_path(&loc);
            self.group_target(g, t);
        }
        group
    }

    pub fn libname(&self) -> AutoStr {
        match self.port.platform.as_str() {
            "windows" => format!("{}.lib", self.name).into(),
            "linux" => format!("lib{}.a", self.name).into(),
            _ => format!("lib{}.a", self.name).into(),
        }
    }

    pub(crate) fn collect_srcs(&mut self) -> AutoResult<()> {
        // warn!("Colelecting srcs: [{}]", self.name);
        let mut sub_srcs = HashSet::new();
        for d in self.dirs.values() {
            let dir_srcs = d.collect_srcs()?;
            // warn!("collected from [{}]: {:?}", d.name, dir_srcs);
            sub_srcs.extend(dir_srcs);
        }
        self.srcs.extend(sub_srcs);

        if !self.deps.is_empty() {
            for dep in self.deps.iter_mut() {
                dep.collect_srcs()?;
            }
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pac;
    use auto_lang::config::AutoConfig;

    #[test]
    fn test_dirs() {
        let dir = Dir::from_str("main".into(), ".".into(), TargetKind::App);
        assert_eq!(dir.to_string(), "main");
    }

    #[test]
    #[ignore = "VM codegen skips child nodes — kids_id always -1 for generic nodes"]
    fn test_dir_from_node_with_main() {
        let code = r#"
        app("main") {
            dir("src") {
                src("main.at")
            }
            dir("util") {
                src("util.at")
            }
        }
        "#;
        let config = AutoConfig::new(code).unwrap();
        let nodes = config.root.nodes("app");
        let app_node = nodes.first().unwrap();
        let target = Target::from((*app_node).clone(), "main".into());
        let at = target.at.clone();
        assert_eq!(at, ".");

        let src_dir = target.get_dir("src");
        if let Some(dir) = src_dir {
            assert_eq!(dir.to_string(), "src");
            assert_eq!(dir.at, "src");
            assert_eq!(dir.name, "src");
        } else {
            panic!("src_dir is None");
        }

        let util_dir = target.get_dir("util");
        if let Some(dir) = util_dir {
            assert_eq!(dir.to_string(), "util");
            assert_eq!(dir.at, "util");
            assert_eq!(dir.name, "util");
        } else {
            panic!("util_dir is None");
        }
    }

    #[test]
    fn test_dir_from_node_with_demo() {
        let code = r#"
        name: "math"

        app("demo") {
        }
        "#;
        let config = AutoConfig::new(code).unwrap();
        let pac = Pac::new(config);
        let target = pac.get_target("demo").unwrap();
        let root = target.at.clone();
        assert_eq!(root, "apps/demo");
    }

    #[test]
    fn test_target() {
        let code = r#"
        app("main") {}
        "#;
        let config = AutoConfig::new(code).unwrap();
        let nodes = config.root.nodes("app");
        let node = nodes.first().unwrap();
        let target = Target::from((*node).clone(), "main".into());
        let root_dir = target.at.clone();
        assert_eq!(root_dir, ".");
        assert_eq!(target.name, "main");
        assert_eq!(target.kind, TargetKind::App);

        // target.scan().unwrap();
        // assert_eq!(target.autos, vec!["main.at"]);
        // assert_eq!(target.srcs, vec!["main.c"]);
        // assert_eq!(target.incs, vec!["main.h"]);
    }

    #[test]
    fn test_current_dir() {
        use auto_lang::util::*;
        let root = "apps/demo";
        let cur = ".";
        let path = Path::new(root).join(cur);
        let path = path.unified();
        assert_eq!(path, "apps/demo");
    }

    #[test]
    fn test_filter_dirs() {
        let selects: HashSet<AutoStr> = vec!["a".into(), "c".into()].to_set();
        let skips: HashSet<AutoStr> = vec![].to_set();
        let name = "a".into();
        let succ = dir_filter(&name, &selects, &skips);
        assert_eq!(succ, true);

        let name = "b".into();
        assert_eq!(dir_filter(&name, &selects, &skips), false);
    }

    // Tests for refactored target creation functions

    #[test]
    fn test_extract_name_and_version_with_id() {
        use auto_val::AutoStr;

        // Create a simple node with id
        let code = r#"name: "test"; app("myapp") {}"#;
        let config = auto_lang::config::AutoConfig::new(code).unwrap();
        let node = config.root.nodes("app").first().unwrap().clone();

        let (name, version) = Target::extract_name_and_version(&node);

        assert_eq!(name, AutoStr::from("myapp"));
        assert_eq!(version, AutoStr::from("latest"));
    }

    #[test]
    #[ignore = "dep is now a keyword — dep() call syntax no longer parses as node"]
    fn test_extract_name_and_version_with_args() {
        use auto_val::AutoStr;

        let code = r#"name: "test"; dep("mylib", "1.0.0") {}"#;
        let config = auto_lang::config::AutoConfig::new(code).unwrap();
        let node = config.root.nodes("dep").first().unwrap().clone();

        let (name, version) = Target::extract_name_and_version(&node);

        assert_eq!(name, AutoStr::from("mylib"));
        assert_eq!(version, AutoStr::from("1.0.0"));
    }

    #[test]
    fn test_extract_kind() {
        let code = r#"name: "test"; app("test") {}"#;
        let config = auto_lang::config::AutoConfig::new(code).unwrap();
        let node = config.root.nodes("app").first().unwrap().clone();

        let kind = Target::extract_kind(&node);

        assert_eq!(kind, TargetKind::App);
    }

    #[test]
    #[ignore = "VM codegen drops named arg keys — kind: 'local' becomes positional arg"]
    fn test_extract_origin_local() {
        let code = r#"name: "test"; app("test", kind: "local") {}"#;
        let config = auto_lang::config::AutoConfig::new(code).unwrap();
        let node = config.root.nodes("app").first().unwrap().clone();

        let origin = Target::extract_origin(&node);

        assert_eq!(origin, TargetOrigin::Local);
    }

    #[test]
    fn test_extract_scan_flag() {
        use crate::node_ext::NodeExt;

        // Test with scan flag set to false
        let code = r#"name: "test"; app("test") { scan: false }"#;
        let config = auto_lang::config::AutoConfig::new(code).unwrap();
        let node = config.root.nodes("app").first().unwrap().clone();

        let is_scan = node.get_bool_or("scan", true);

        assert_eq!(is_scan, false);
    }

    #[test]
    fn test_extract_scan_flag_default() {
        use crate::node_ext::NodeExt;

        // Test without scan flag (should default to true)
        let code = r#"name: "test"; app("test") {}"#;
        let config = auto_lang::config::AutoConfig::new(code).unwrap();
        let node = config.root.nodes("app").first().unwrap().clone();

        let is_scan = node.get_bool_or("scan", true);

        assert_eq!(is_scan, true);
    }

    #[test]
    fn test_extract_recurse_flag() {
        use crate::node_ext::NodeExt;

        // Test with recurse flag set to true
        let code = r#"name: "test"; app("test") { recurse: true }"#;
        let config = auto_lang::config::AutoConfig::new(code).unwrap();
        let node = config.root.nodes("app").first().unwrap().clone();

        let recurse = node.get_bool_or("recurse", false);

        assert_eq!(recurse, true);
    }

    #[test]
    fn test_extract_sources_and_includes() {
        use crate::node_ext::NodeExt;
        use auto_val::AutoStr;

        let code =
            r#"name: "test"; app("test") { srcs: ["main.c", "utils.c"], incs: ["header.h"] }"#;
        let config = auto_lang::config::AutoConfig::new(code).unwrap();
        let node = config.root.nodes("app").first().unwrap().clone();

        // 使用 NodeExt 直接获取属性
        let srcs: Vec<AutoStr> = node.get_str_vec_or("srcs");
        let incs: Vec<AutoStr> = node.get_str_vec_or("incs");

        assert_eq!(srcs.len(), 2);
        assert!(srcs.contains(&AutoStr::from("main.c")));
        assert!(srcs.contains(&AutoStr::from("utils.c")));
        assert_eq!(incs.len(), 1);
        assert!(incs.contains(&AutoStr::from("header.h")));
    }
}
