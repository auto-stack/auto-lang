use crate::builder::new_builder;
use crate::builder::Builder;
use crate::builder::BuilderKind;
use crate::git::check_changes;
use crate::git::check_detached;
use crate::git::pull;
use crate::git::switch_to_master;
use crate::make_builder;
use crate::AutoResult;
use crate::Port;
use crate::TargetOrigin;
use crate::{Index, PacInfo};
use crate::{Target, TargetKind, TargetStatus};
use auto_lang::config::AutoConfig;
use auto_lang::Atom;
use auto_val::{pretty, Arg, Array, AutoPath, AutoStr, Kid, Node, Obj, Value, ValueKey};
use auto_val::{shared, Shared};
use log::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;
use tabled::{
    settings::{object::Rows, style::Style, themes::Colorization, Color},
    Table,
};

pub struct Pac {
    pub name: AutoStr,
    pub port: Port,
    pub version: AutoStr,
    pub ports: Vec<Node>,
    pub builder: Option<Box<dyn Builder>>,
    pub targets: Vec<Target>,
    pub index: Shared<Index>,
    pub device_index: Shared<Index>,
    pub props: Obj,
    pub build_at: AutoStr,
    pub build_location: AutoStr,
    /// Whether to show header files in `srcs`
    pub show_headers: bool,
    pub defines: Vec<AutoStr>,
    is_update: bool,
}

fn select_port(input: Option<String>, ports: &Vec<AutoStr>) -> AutoResult<AutoStr> {
    crate::util::select_or_default_port(input, ports, "Which port do you want to build?")
}

impl Pac {
    pub fn new(config: AutoConfig) -> Self {
        use crate::node_ext::NodeExt;

        let pac_name = config.name();
        let version = config.version();
        let props = config.root.props_clone();

        // ports
        let ports = config.root.get_nodes("port");
        let builder = None;

        // defines
        let defines = config.root.get_str_vec_or("defines");

        // target related properties
        let target_props = vec!["at", "lang"];

        let mut default_builder: AutoStr = "ninja".into();
        if target_props.contains(&"lang") {
            let lang_val = config.args.get("lang");
            let lang_str = match lang_val {
                Some(v) => v.to_astr(),
                None => "c".into(),
            };
            if lang_str == "rust" {
                default_builder = "cargo".into();
            }
        }

        // targets, NOTE: ports are not targets
        let mut targets = vec![];
        for (_, kid) in config.root.kids_iter() {
            if let Kid::Node(mut n) = kid.clone() {
                if n.name != "port" {
                    // main target should receive target related args from the pac
                    if n.main_arg().to_astr() == pac_name && n.name == "device" {
                        for p in &target_props {
                            if config.args.has(*p) {
                                let value = config.args.get(*p).unwrap();
                                // println!("Setting prop: {}: {}", *p, value.clone());
                                // TODO: find a better way to transfer args to dependencies target
                                n.set_prop(*p, value);
                            }
                        }
                    }

                    let mut target = Target::from(n, pac_name.clone());
                    target.set_defines(defines.clone());
                    targets.push(target);
                }
            }
        }

        let mut port = Port::default();
        port.builder = default_builder;

        Self {
            name: pac_name,
            version,
            port,
            ports,
            builder,
            targets,
            props,
            index: shared(Index::default()),
            device_index: shared(Index::default()),
            build_at: ".".into(),
            build_location: "cmake".into(),
            show_headers: false,
            defines,
            is_update: false,
        }
    }

    pub fn transpile_autot(&mut self) -> AutoResult<()> {
        for target in &mut self.targets {
            if target.has_auto() {
                target.transpile_auto()?;
            }
        }
        Ok(())
    }

    pub fn print_target_info(&self, target: &str) -> AutoResult<()> {
        if let Some(target) = self.targets.iter().find(|t| t.name == target) {
            target.print_info()
        } else {
            return Err("Target not found".into());
        }
    }

    // print targets as a table
    pub fn print_targets(&self) {
        info!("--- pac: {} ---\n", self.name);
        let mut statuses = vec![];
        for target in &self.targets {
            statuses.push(TargetStatus::from(target));
        }

        let mut table = Table::new(&statuses);
        table
            .with(Style::modern_rounded())
            .with(Colorization::exact([Color::FG_BRIGHT_BLUE], Rows::first()));

        println!("{}", table);
    }

    pub fn has_device_prop(&self, prop: impl Into<AutoStr>) -> bool {
        let mut has_prop = false;
        // TODO: too much clone for key
        let key = ValueKey::Str(prop.into());
        if self.props.has(key.clone()) {
            return true;
        }
        for target in &self.targets {
            if target.kind == TargetKind::Device {
                if target.props.has(key.clone()) {
                    has_prop = true;
                    break;
                }
            }
        }
        has_prop
    }

    /// Print atom files to local directory
    pub fn print_atom(&self) -> AutoResult<()> {
        let atom = self.to_atom();
        // info!("ATOM packed for pac: <{}>", self.name);
        // println!("{}", atom);
        let atom_pretty = pretty(atom.to_string().as_str(), 3);
        // info!("ATOM for pac: <{}>", self.name);
        // write prettified atom to file
        // write prettified atom to file
        std::fs::write(".am/pac.atom.at", atom_pretty.as_bytes())?;

        for app in self.build_targets() {
            let name = app.name.clone();

            let app_atom = app.to_atom();
            let app_atom_pretty = pretty(app_atom.to_string().as_str(), 3);

            std::fs::write(format!(".am/{}.atom.at", name), app_atom_pretty.as_bytes())?;
        }

        Ok(())
    }

    pub fn get_target(&self, name: impl Into<AutoStr>) -> Option<&Target> {
        let name = name.into();
        self.targets.iter().find(|t| t.name == name)
    }

    pub fn deps(&self) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|t| t.kind == TargetKind::Dep)
            .collect()
    }

    pub fn libs(&self) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|t| t.kind == TargetKind::Lib)
            .collect()
    }

    pub fn apps(&self) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|t| t.kind == TargetKind::App)
            .collect()
    }

    pub fn build_targets_mut(&mut self) -> Vec<&mut Target> {
        self.targets
            .iter_mut()
            .filter(|t| t.kind == TargetKind::App || t.kind == TargetKind::Lib)
            .collect()
    }

    pub fn build_targets(&self) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|t| t.kind == TargetKind::App || t.kind == TargetKind::Lib)
            .collect()
    }

    pub fn list_port_names(&self) -> Vec<AutoStr> {
        self.ports.iter().map(|p| p.main_arg().to_astr()).collect()
    }

    pub fn get_state_file(&self) -> AutoResult<AutoPath> {
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

    pub fn try_load_port(&mut self) -> AutoResult<()> {
        // load local state from .am/state.at
        let state_file = self.get_state_file()?;
        let state = AutoConfig::read(&state_file.path())?;

        let port_name = state.root.get_prop("port").to_astr();
        if !port_name.is_empty() {
            self.set_port(port_name)?;
        } else {
            let ports = self.list_port_names();
            let port_name = select_port(None, &ports)?;
            self.set_port(port_name.clone())?;
            self.save_port(port_name)?;
        }

        Ok(())
    }

    pub fn save_port(&mut self, port: AutoStr) -> AutoResult<()> {
        let state_file = self.get_state_file()?;
        let mut state = AutoConfig::read(&state_file.path())?;
        state.root.set_prop("port", port.clone());
        state.save(&state_file)?;
        Ok(())
    }

    pub fn set_port(&mut self, port_name: AutoStr) -> AutoResult<()> {
        self.merge_port(port_name.clone())?;

        // 0. check port
        let mut at = self.port.at.clone();
        if at.ends_with("/") {
            at = at[0..at.len() - 1].into()
        }
        self.build_at = at.clone();
        self.build_location = AutoPath::new(at.clone()).to_astr();
        // set builder
        let build_path = AutoPath::new(self.build_location.clone());
        println!(
            "setting port builder: {} {}, at {}",
            port_name, self.port.builder, build_path
        );
        self.builder = make_builder(&self.port.builder, build_path);
        if self.port.builder == "ghs" || self.port.builder == "ninja" {
            self.set_show_headers();
        }
        // set tool

        Ok(())
    }

    pub fn set_show_headers(&mut self) {
        self.show_headers = true;
    }

    pub fn copy_target(&self, id: &AutoStr) -> Option<Target> {
        for t in &self.targets {
            if t.name == *id {
                return Some(t.clone());
            }
        }
        None
    }

    pub fn targets_map(&mut self) -> HashMap<AutoStr, Target> {
        let mut targets_map = HashMap::new();
        let targets = std::mem::take(&mut self.targets);
        for target in targets {
            targets_map.insert(target.name.clone(), target.clone());
        }
        targets_map
    }

    pub fn get_build_names(targets_map: &HashMap<AutoStr, Target>) -> Vec<AutoStr> {
        let mut build_names = Vec::new();
        for target in targets_map.values() {
            if target.kind == TargetKind::App || target.kind == TargetKind::Lib {
                build_names.push(target.name.clone());
            }
        }
        build_names
    }

    /// Resolve target dependencies by building transitive closure of links
    ///
    /// This function iteratively resolves dependencies until all targets have
    /// their complete dependency tree calculated. Returns true if all targets
    /// are resolved, false if some targets still have unresolved dependencies.
    fn resolve_target_deps(targets_map: &mut HashMap<AutoStr, Target>) -> bool {
        let all_resolved = Self::check_all_resolved(targets_map);

        // Build resolved links map for targets whose dependencies are all resolved
        let resolved_links = Self::build_resolved_links(targets_map);

        // Apply the resolved links to update target dependency information
        Self::apply_resolved_links(targets_map, resolved_links);

        all_resolved
    }

    /// Check if all targets are marked as resolved
    fn check_all_resolved(targets_map: &HashMap<AutoStr, Target>) -> bool {
        targets_map.values().all(|t| t.is_resolved)
    }

    /// Build a map of resolved links for targets whose dependencies are all resolved
    ///
    /// For each target, if all its direct and transitive dependencies are resolved,
    /// collect the complete set of links (transitive closure).
    fn build_resolved_links(targets_map: &HashMap<AutoStr, Target>) -> HashMap<AutoStr, Vec<Node>> {
        let mut resolved_links: HashMap<AutoStr, Vec<Node>> = HashMap::new();

        for target in targets_map.values() {
            if target.links.is_empty() {
                // Target with no dependencies is trivially resolved
                resolved_links.insert(target.name.clone(), vec![]);
                continue;
            }

            // Check if we can resolve this target's dependencies
            if let Some(links) = Self::try_resolve_target_links(target, targets_map) {
                resolved_links.insert(target.name.clone(), links);
            }
        }

        resolved_links
    }

    /// Try to resolve all links for a target
    ///
    /// Returns Some(links) if all dependencies are resolved, None otherwise.
    /// When successful, returns the transitive closure of all links.
    fn try_resolve_target_links(
        target: &Target,
        targets_map: &HashMap<AutoStr, Target>,
    ) -> Option<Vec<Node>> {
        let mut all_deps_resolved = true;
        let mut links = Vec::new();

        // Start with direct links
        links.extend(target.links.iter().cloned());

        // Transitively collect links from dependencies
        for link in &target.links {
            // println!("LINKS TO RESOLVE: {}", link);
            let Some(dep) = targets_map.get(&link.id()) else {
                continue;
            };

            if !dep.is_resolved {
                all_deps_resolved = false;
                break;
            }

            // Add transitive links from this dependency
            links.extend(dep.links.iter().cloned());
        }

        if all_deps_resolved {
            Some(links)
        } else {
            None
        }
    }

    /// Apply resolved links to update targets in the map
    ///
    /// For each target in resolved_links, mark it as resolved and update
    /// its dependency information (dep_names and all_links).
    fn apply_resolved_links(
        targets_map: &mut HashMap<AutoStr, Target>,
        resolved_links: HashMap<AutoStr, Vec<Node>>,
    ) {
        for (target_name, links) in resolved_links {
            // Extract dependency names from the links
            let dep_names: Vec<AutoStr> = links
                .iter()
                .filter_map(|link| {
                    let id = link.id();
                    targets_map.get(&id).map(|dep| dep.name.clone())
                })
                .collect();

            // Update the target with resolved information
            if let Some(target) = targets_map.get_mut(&target_name) {
                target.is_resolved = true;
                target.dep_names = dep_names;
                target.all_links = links;
            }
        }
    }

    pub fn pull(&mut self) -> AutoResult<()> {
        self.is_update = true;
        self.download()?;
        self.is_update = false;
        Ok(())
    }

    pub fn download(&mut self) -> AutoResult<()> {
        info!("Downloading deps");
        self.download_deps(&AutoStr::new(), &AutoStr::new())?;
        info!("download done!");
        Ok(())
    }

    pub fn resolve(&mut self) -> AutoResult<()> {
        // 0. check basic props
        if !self.props.has("defines") {
            let defines = Vec::<AutoStr>::new();
            // defines.push("DEBUG".into());
            // defines.push("USE_KERNEL".into());
            self.props.set("defines", defines);
        }

        // 1. download dependencies, recursively
        self.download()?;

        let rel = self.relative_path();
        for t in &mut self.targets {
            t.set_rel(rel.clone());
            t.set_defines(self.defines.clone());
            t.set_port(self.port.clone());
        }

        self.print_targets();

        // 2. link targets with dependencies
        info!("Linking targets with dependencies");

        // let devices: Vec<Target> = self
        //     .targets
        //     .iter()
        //     .filter(|t| t.kind == TargetKind::Device)
        //     .map(|t| t.clone())
        //     .collect();
        let mut targets_map = self.targets_map();
        let mut major_targets = Vec::new();
        // remained_targets.extend(devices);

        // resolve target's links, expanding indirect dependencies into target.dep_names
        let mut done = false;
        while !done {
            done = Self::resolve_target_deps(&mut targets_map);
        }

        let major_target_names = targets_map
            .values()
            .filter(|t| t.is_major())
            .map(|t| t.name.clone())
            .collect::<Vec<AutoStr>>();

        for n in major_target_names {
            let mut deps = Vec::new();
            let t = targets_map.get(&n).unwrap();
            for l in &t.all_links {
                let Some(dep) = targets_map.get(&l.id()) else {
                    continue;
                };
                let mut merge = dep.clone();
                merge.merge_link(l);
                deps.push(merge);
            }
            // for dep in &t.dep_names {
            //     let Some(dep) = targets_map.get(dep) else {
            //         continue;
            //     };
            //     deps.push(dep.clone());
            // }
            let Some(t) = targets_map.get_mut(&n) else {
                continue;
            };
            t.deps = deps;
        }

        // copy dep's target info into major targets
        targets_map
            .values()
            .filter(|t| {
                t.kind == TargetKind::Device
                    || t.kind == TargetKind::App
                    || t.kind == TargetKind::Lib
                    || t.kind == TargetKind::Test
            })
            .for_each(|t| major_targets.push(t.clone()));

        self.targets = major_targets;

        // 3. scan targets for its local source files
        info!("Scanning targets for srcs and incs, only major targets and their deps are scanned");
        let show_headers = self.show_headers;
        for target in &mut self.targets {
            target.show_headers = show_headers;
            target.scan()?;
        }
        Ok(())
    }

    fn merge_port(&mut self, port_name: AutoStr) -> AutoResult<()> {
        // reload config with said port
        let port_node = self
            .ports
            .iter()
            .find(|p| p.main_arg().to_astr() == port_name);

        let Some(port_node) = port_node else {
            self.port = Port::default();
            return Ok(());
        };

        // props
        let props = port_node.props_clone();
        self.props.merge(&props);

        // targets
        for (_, kid) in port_node.kids_iter() {
            if let Kid::Node(n) = kid {
                let target = Target::from(n.clone(), self.name.clone());
                self.targets.push(target);
            }
        }
        let mut port = Port::default();
        port.name = port_node.main_arg().to_astr();
        port.builder = port_node.get_prop("builder").to_astr();
        port.platform = port_node.get_prop("platform").to_astr();
        port.at = port_node.get_prop("at").to_astr();

        // 解析完整的编译器配置
        use crate::builder::ninja::config::{
            CompilerConfig, CompilerLocation, FlagFormat, FlagMappings,
        };
        use crate::node_ext::NodeExt;
        use auto_val::{Kid, ValueKey};

        if let Some(Kid::Node(compiler_node)) = port_node.get_kid(&ValueKey::Str("compiler".into()))
        {
            // 解析编译器类型
            let kind_str = compiler_node.get_prop("kind").to_astr();
            if let Some(kind) =
                crate::builder::ninja::config::CompilerKind::from_str(kind_str.as_str())
            {
                // 解析 location
                let location = if let Some(Kid::Node(location_node)) =
                    compiler_node.get_kid(&ValueKey::Str("location".into()))
                {
                    let location_type = location_node.main_arg().to_astr();
                    match location_type.as_str() {
                        "Env" => CompilerLocation::Env,
                        "Dir" => {
                            let dir_str = location_node.get_prop("path").to_astr();
                            CompilerLocation::Dir(auto_val::AutoPath::new(dir_str.as_str()))
                        }
                        "Executable" => {
                            let exec_type_str = location_node.get_prop("type").to_astr();
                            let exec_type = crate::builder::ninja::config::ExecutableType::from_str(
                                exec_type_str.as_str(),
                            );
                            if let Some(exec_type) = exec_type {
                                let path = auto_val::AutoPath::new(
                                    location_node.get_prop("path").to_astr().as_str(),
                                );
                                CompilerLocation::Executable(exec_type, path)
                            } else {
                                CompilerLocation::Env
                            }
                        }
                        _ => CompilerLocation::Env,
                    }
                } else {
                    CompilerLocation::Env
                };

                // 解析 executables
                let mut executables = std::collections::HashMap::new();
                if let Some(Kid::Node(execs_node)) =
                    compiler_node.get_kid(&ValueKey::Str("executables".into()))
                {
                    if execs_node.has_prop("cc") {
                        let cc =
                            auto_val::AutoPath::new(execs_node.get_prop("cc").to_astr().as_str());
                        executables
                            .insert(crate::builder::ninja::config::ExecutableType::Compiler, cc);
                    }
                    if execs_node.has_prop("ar") {
                        let ar =
                            auto_val::AutoPath::new(execs_node.get_prop("ar").to_astr().as_str());
                        executables
                            .insert(crate::builder::ninja::config::ExecutableType::Archiver, ar);
                    }
                    if execs_node.has_prop("link") {
                        let link =
                            auto_val::AutoPath::new(execs_node.get_prop("link").to_astr().as_str());
                        executables
                            .insert(crate::builder::ninja::config::ExecutableType::Linker, link);
                    }
                    if execs_node.has_prop("as") {
                        let as_cmd =
                            auto_val::AutoPath::new(execs_node.get_prop("as").to_astr().as_str());
                        executables.insert(
                            crate::builder::ninja::config::ExecutableType::Assembler,
                            as_cmd,
                        );
                    }
                }

                // 解析 flags
                let flags = if let Some(Kid::Node(flags_node)) =
                    compiler_node.get_kid(&ValueKey::Str("flags".into()))
                {
                    let parse_flag_format = |flag_name: &str| -> FlagFormat {
                        let flag_value = flags_node.get_str_or(flag_name, "-");
                        if flag_value.contains('(') {
                            let left_paren = flag_value.find('(').unwrap_or(0);
                            let right_paren = flag_value.find(')').unwrap_or(flag_value.len());
                            let format_type = &flag_value[0..left_paren];
                            let format_arg = &flag_value[left_paren + 1..right_paren];

                            match format_type {
                                "Prefix" => FlagFormat::Prefix(format_arg.to_string()),
                                "Postfix" => FlagFormat::Postfix(format_arg.to_string()),
                                "Both" => {
                                    let args: Vec<&str> = format_arg.split(", ").collect();
                                    if args.len() == 2 {
                                        FlagFormat::Both(args[0].to_string(), args[1].to_string())
                                    } else {
                                        FlagFormat::Prefix(args[0].to_string())
                                    }
                                }
                                _ => FlagFormat::Prefix(format_arg.to_string()),
                            }
                        } else {
                            FlagFormat::Prefix(flag_value.as_str().to_string())
                        }
                    };

                    FlagMappings {
                        include: parse_flag_format("include"),
                        define: parse_flag_format("define"),
                        library: parse_flag_format("library"),
                        library_path: parse_flag_format("library_path"),
                        output: parse_flag_format("output"),
                        compile_only: parse_flag_format("compile_only"),
                    }
                } else {
                    // 使用默认 flags
                    match kind {
                        crate::builder::ninja::config::CompilerKind::MSVC => {
                            FlagMappings::msvc_default()
                        }
                        crate::builder::ninja::config::CompilerKind::GCC => {
                            FlagMappings::gcc_default()
                        }
                        crate::builder::ninja::config::CompilerKind::Clang => {
                            FlagMappings::clang_default()
                        }
                        crate::builder::ninja::config::CompilerKind::IAR => {
                            FlagMappings::iar_default()
                        }
                        crate::builder::ninja::config::CompilerKind::GHS => {
                            FlagMappings::ghs_default()
                        }
                        crate::builder::ninja::config::CompilerKind::Targeting => {
                            FlagMappings::targeting_default()
                        }
                        crate::builder::ninja::config::CompilerKind::Hightec => {
                            FlagMappings::hightec_default()
                        }
                    }
                };

                // 解析 default_cflags
                let default_cflags = if let Some(Kid::Node(cflags_node)) =
                    compiler_node.get_kid(&ValueKey::Str("default_cflags".into()))
                {
                    cflags_node.get_str_vec_or("value")
                } else {
                    Vec::new()
                };

                // 创建编译器配置
                let compiler_config = CompilerConfig {
                    kind,
                    name: port.name.clone(),
                    executables,
                    flags,
                    location,
                    default_cflags,
                };

                port.compiler = Some(compiler_config);
            }
        }

        self.port = port;
        Ok(())
    }

    fn update_repo(&self, path: &AutoPath) -> AutoResult<()> {
        warn!("trying to update repo {}", path);

        check_changes(path)?;
        match check_detached(path) {
            Ok(_) => {}
            Err(_) => {
                // switch to master
                switch_to_master(path)?;
            }
        }

        pull(path)
    }

    pub fn contains(&self, name: &AutoStr) -> bool {
        self.targets.iter().any(|t| t.name == *name)
    }

    fn download_deps(&mut self, parent: &AutoStr, version: &AutoStr) -> AutoResult<()> {
        let mut targets = std::mem::take(&mut self.targets);
        let mut new_deps = HashMap::new();
        let mut target_names = HashMap::new();
        for t in targets.iter() {
            target_names.insert(t.name.clone(), true);
        }
        for target in targets.iter_mut() {
            // override target's at location
            if !parent.is_empty() {
                target.set_parent(parent);
            };
            let loc = target.location();

            match target.kind {
                TargetKind::Dep | TargetKind::Device => {
                    // 1. update if exists
                    if target.exists() {
                        continue;
                    }

                    if version == "latest" {
                        target.version = "latest".into();
                    }
                    // 2. download or copy the dep target
                    match target.origin {
                        TargetOrigin::Index => {
                            self.download_dep(&target)?;
                        }
                        TargetOrigin::Git => {
                            self.download_git(&target)?;
                        }
                        TargetOrigin::Local => {
                            self.copy_local(&target)?;
                        }
                    }

                    // 3. read pac.at from dep
                    // get args from dep node props
                    // read pac.at from dep and parse the lib target
                    let target_path = loc.join(target.config_name());
                    // info!(
                    // "reading pac {} with override props: {:?}",
                    // target_path, target.props
                    // );
                    let dep_config = AutoConfig::from_file(
                        target_path.path(),
                        // override dep props
                        &target.props,
                    )?;
                    let mut pac = Pac::new(dep_config);
                    pac.index = self.index.clone();
                    pac.device_index = self.device_index.clone();
                    pac.is_update = self.is_update;
                    // get the main target of the package
                    let dep_name = target.name.clone();
                    let lib_target = pac.get_target(dep_name);
                    if let Some(lib_target) = lib_target {
                        // merge the lib target into current dep target
                        target.merge(lib_target);
                    }
                    // get other lib targetsm
                    for lib in pac.libs() {
                        if lib.name == target.name {
                            continue;
                        }
                        if !target_names.contains_key(&lib.name) {
                            let mut lib = lib.clone();
                            // check new root dir
                            for (_, v) in lib.dirs.iter_mut() {
                                let path = Path::new(target.at.as_str()).join(v.at.as_str());
                                if path.is_dir() {
                                    v.append_root(target.at.clone());
                                }
                            }

                            new_deps.insert(lib.name.clone(), lib);
                        }
                    }

                    // 4. recursively download dep's deps
                    pac.download_deps(&target.parent(), &target.version.clone())?;
                    // merge dep's dep into current pac if not exists
                    for dep in pac.deps() {
                        // if !new_deps.contains_key(&dep.name)
                        // && !target_names.contains_key(&dep.name)
                        if !target_names.contains_key(&dep.name) {
                            new_deps.insert(dep.name.clone(), dep.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        self.targets = targets;
        for (_, dep) in new_deps {
            if !self.targets.iter().any(|t| t.name == dep.name) {
                self.targets.push(dep);
            }
        }
        Ok(())
    }

    pub fn download_git(&mut self, dep: &Target) -> AutoResult<()> {
        let info = PacInfo {
            name: dep.name.clone(),
            version: dep.version.clone(),
            repo: dep.from.clone(),
        };
        self.download_repo(&info, &dep.kind, &dep.at)
    }

    pub fn copy_local(&mut self, dep: &Target) -> AutoResult<()> {
        // println!("copy for dep {}", dep.name);
        let from = dep.from.clone();
        let from_path = AutoPath::new(from.clone());
        // check if from dir exists
        if !from_path.is_dir() {
            return Err(format!("Local dep dir {} does not exist", from).into());
        }
        // copy files from from to target
        let target = dep.at.clone();
        let target_path = AutoPath::new(target);
        if !target_path.exists() {
            std::fs::create_dir_all(target_path.path())?;
        }
        // copy recursively
        crate::fs::copy_dir_all(from_path.path(), target_path.path())?;
        Ok(())
    }

    pub fn download_dep(&mut self, dep: &Target) -> AutoResult<()> {
        // 查询索引
        let name = dep.name.clone();
        let at = dep.at.clone();

        let index = match dep.kind {
            TargetKind::Dep => self.index.borrow(),
            TargetKind::Device => self.device_index.borrow(),
            _ => return Err(format!("Invalid target kind to download: {}", dep.kind).into()),
        };
        let info = index.lookup(&name);
        match info {
            Some(info) => {
                let mut info = info.clone();
                // TODO: check if dep version is in index
                if dep.version == "latest" {
                    info.version = "master".into();
                } else {
                    info.version = dep.version.clone();
                }
                self.download_repo(&info, &dep.kind, &at)?;
                return Ok(());
            }
            None => {
                return Err(format!("Dependency {} not found", name).into());
            }
        }
    }

    fn starts_with_digit(&self, s: &str) -> bool {
        s.chars().next().map_or(false, |c| c.is_digit(10))
    }

    pub fn download_repo(&self, info: &PacInfo, kind: &TargetKind, at: &AutoStr) -> AutoResult<()> {
        use indicatif::{ProgressBar, ProgressStyle};

        // 1. make deps folder if not exists
        let mut deps_dir = match kind {
            TargetKind::Dep => "deps",
            TargetKind::Device => "devices",
            _ => return Err(format!("Invalid target kind: {}", kind).into()),
        };
        if !at.is_empty() {
            deps_dir = at.as_str();
        }
        // 2. use git to download the repo
        let url = format!("{}", info.repo);
        let path = deps_dir;
        info!("- {}: ({})", path, info.repo);

        if Path::new(&path).exists() {
            if self.is_update {
                let p = AutoPath::new(path);
                self.update_repo(&p)?;
            }
        } else {
            // Create progress bar for download
            let pb = ProgressBar::new_spinner();
            pb.set_message(format!("Cloning {} from {}", info.name, info.repo));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner} {msg}")
                    .unwrap(),
            );

            let output = std::process::Command::new("git")
                .arg("clone")
                .arg(url)
                .arg(path)
                .output()?;

            if output.status.success() {
                pb.finish_with_message(format!("Downloaded {}", info.name));
                info!("Downloaded dependency {} from {}", info.name, info.repo);
            } else {
                pb.abandon_with_message(format!("Failed to download {}", info.name));
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "Failed to download dependency {} from {}: {}",
                    info.name, info.repo, stderr
                )
                .into());
            }
        }
        // 3. checkout wanted version
        let mut version = info.version.clone();
        if !version.is_empty() {
            if version == "latest" {
                version = "master".into();
            } else if version == "master" {
                // no nothing
            } else {
                if self.starts_with_digit(version.as_str()) {
                    version = format!("v{}", version.as_str()).into();
                }
            }
            info!("    - checkout version {}", version);
            let mut cmd = std::process::Command::new("git");
            cmd.arg("-C")
                .arg(path)
                .arg("checkout")
                .arg(version.as_str());
            let output = cmd.output()?;
            if !output.status.success() {
                error!("Failed to checkout {} to version {}", info.name, version);
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Ok(())
    }

    pub fn exe_path(&self) -> String {
        // TODO: check Debug/Release, check exe name
        let mode = "Debug";
        // find first exe in targets
        for target in &self.targets {
            match target.kind {
                TargetKind::App => {
                    return format!("build/{}/{}", mode, target.name);
                }
                _ => {}
            }
        }
        format!("build/{}/{}", mode, self.name)
    }

    pub fn build(&mut self) -> AutoResult<()> {
        info!("Building port at ./{}", self.build_location);
        let builder = std::mem::take(&mut self.builder);
        if let Some(mut builder) = builder {
            builder.build(self)?;
            self.builder = Some(builder);
        }
        Ok(())
    }

    pub fn run(&mut self, args: Vec<String>) -> AutoResult<()> {
        let builder = std::mem::take(&mut self.builder);
        if let Some(mut builder) = builder {
            builder.run(self, args)?;
            self.builder = Some(builder);
        } else {
            let mut builder = new_builder(BuilderKind::CMake("CMakeLists.txt".to_string()));
            builder.run(self, args)?;
        }
        Ok(())
    }

    pub fn clean(&mut self) -> AutoResult<()> {
        // 1. remove build directory
        // TODO: what about other builders that are not choosed yet?
        let builder = std::mem::take(&mut self.builder);
        if let Some(mut builder) = builder {
            builder.clean()?;
            self.builder = Some(builder);
        }
        // 2. remove downloaded deps and devices

        // NOTE: if content is modified (yet to be committed), a warning should be displayed and the directory should be left
        // for t in self.downloaded() {
        // t.clean()?;
        // }
        Ok(())
    }

    pub(crate) fn collect_srcs(&mut self) -> AutoResult<()> {
        for t in self.targets.iter_mut() {
            t.collect_srcs()?;
        }
        Ok(())
    }
}

fn path_depth(path: &AutoStr) -> u32 {
    if path.is_empty() || path == "." {
        return 0;
    }
    path.split("/").count() as u32
}

impl Pac {
    fn has_target(&self, kind: TargetKind) -> bool {
        for t in &self.targets {
            if t.kind == kind {
                return true;
            }
        }
        false
    }

    pub fn all_incs(&self) -> Vec<AutoStr> {
        let mut all_incs = HashSet::new();
        for target in &self.targets {
            all_incs.extend(target.all_incs());
        }
        let mut all_incs = all_incs.into_iter().collect::<Vec<AutoStr>>();
        all_incs.sort();
        all_incs
    }

    pub fn to_node(&self) -> Node {
        let mut root = Node::new("root");
        // name prop is the first positional arg
        root.add_arg(Arg::Pos(Value::Str(self.name.clone())));
        root.set_prop("version", self.version.clone());
        let mut all_incs = self.all_incs();
        for target in &self.targets {
            root.add_kid(target.to_node());
        }
        all_incs.sort();
        root.set_prop("all_incs", Value::Array(Array::from_vec(all_incs)));

        if !self.has_target(TargetKind::Dep) {
            root.set_prop("deps", Value::Array(Array::new()));
        }

        if !self.has_target(TargetKind::Device) {
            root.set_prop("devices", Value::Array(Array::new()));
        }

        if !self.has_target(TargetKind::Lib) {
            root.set_prop("libs", Value::Array(Array::new()));
        }

        if !self.has_target(TargetKind::App) {
            root.set_prop("apps", Value::Array(Array::new()));
        }

        // root.set_prop("files", self.files_list());
        // root.set_prop("groups", self.group_list());
        root.set_prop("files", Array::new());
        // root.set_prop("groups", self.groups().to_xml());

        let props = self.props.clone();
        for (k, v) in props.iter() {
            root.set_prop(k.to_string().as_str(), v.clone());
        }

        // set builder folder
        root.set_prop("relative_path", self.relative_path());
        root
    }

    fn relative_path(&self) -> AutoStr {
        let depth = path_depth(&self.build_at) as usize;
        "/..".repeat(depth).into()
    }

    // fn group_target(&self, g: &mut Group, t: &Target) {
    //     // set files
    //     for src in &t.srcs {
    //         let fpath = format!("$PROJ_DIR${}\\{}", &self.relative_path(), src);
    //         g.files.push(crate::group::File { name: fpath.into() })
    //     }
    //     // set dirs
    //     for (_n, d) in &t.dirs {
    //         let dg = g.mut_kid(&d.name);
    //         self.group_dir(dg, d);
    //     }
    //     // set deps
    //     for dep in &t.deps {
    //         let dg = g.mut_kid(&dep.name);
    //         self.group_target(dg, dep);
    //     }
    // }

    // fn groups(&self) -> Group {
    //     let mut group = Group::new("root");
    //     for t in &self.targets {
    //         let loc = t.location();
    //         let g = group.mut_kid_path(&loc);
    //         self.group_target(g, t);
    //     }
    //     group
    // }

    #[allow(dead_code)]
    fn files_list(&self) -> Array {
        let mut total_list: HashMap<AutoStr, Vec<Obj>> = HashMap::new();

        for t in &self.targets {
            let loc = t.location();
            // get the first two parts of loc
            let mut head = loc.parent().filename();
            if head == t.name {
                head = ".".into();
            }
            if head.is_empty() {
                head = "main".into();
            }
            let mut target_dirs = Array::new();
            for (_, d) in t.dirs.iter() {
                let mut target_srcs = Obj::new();
                let array: Vec<Value> = d.srcs.iter().map(|x| Value::Str(x.clone())).collect();
                let array = Value::array(array);
                target_srcs.set("name", d.name.clone());
                target_srcs.set("srcs", array);
                target_dirs.push(target_srcs);
            }
            let mut target_obj = Obj::new();
            let array: Vec<Value> = t.srcs.iter().map(|x| Value::Str(x.clone())).collect();
            let array = Value::array(array);
            target_obj.set("srcs", array);
            target_obj.set("name", t.local_name());
            target_obj.set("dirs", target_dirs);
            if total_list.contains_key(&head) {
                let array = total_list.get_mut(&head);
                if let Some(array) = array {
                    array.push(target_obj);
                }
            } else {
                let mut array = vec![];
                array.push(target_obj);
                total_list.insert(head.clone(), array);
            }
        }

        // convert hashmap to Array
        let mut array = Array::new();
        for (k, v) in total_list {
            let mut obj = Obj::new();
            obj.set("name", k);
            obj.set("targets", v);
            array.push(obj);
        }
        array
    }

    pub fn to_atom(&self) -> Atom {
        Atom::node(self.to_node())
    }
}

impl fmt::Display for Pac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pac() {
        let code = r#"
        name: "test"
        version: "0.0.1"

        app("main") {}
        "#;
        let config = AutoConfig::new(code);
        match config {
            Ok(config) => {
                let pac = Pac::new(config);
                assert_eq!(pac.name, "test");
                assert_eq!(pac.version, "0.0.1");
                assert_eq!(pac.targets.len(), 1);
                assert_eq!(pac.targets[0].name, "main");
                assert_eq!(pac.targets[0].kind, TargetKind::App);
            }
            Err(e) => {
                assert!(false, "{}", e);
            }
        }
    }

    #[test]
    fn test_tabled() {
        let code = r#"
        name: "test"
        app("main") {}
        lib("lib") {}
        "#;
        let config = AutoConfig::new(code);
        let pac = Pac::new(config.unwrap());
        pac.print_targets();
        let atom = pac.to_atom();
        println!("{}", atom);
    }

    #[test]
    fn test_autostr_key() {
        use auto_val::AutoStr;
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let key = AutoStr::from("a");
        map.insert(key, "A");

        let key2: AutoStr = "a".into();
        assert!(map.contains_key(&key2));
    }

    #[test]
    fn test_path_depth() {
        use super::*;
        let path = "project/lse1480".into();
        assert_eq!(path_depth(&path), 2);

        let path = "project".into();
        assert_eq!(path_depth(&path), 1);

        let path = ".".into();
        assert_eq!(path_depth(&path), 0);
    }

    #[test]
    fn test_dir_groups() {
        use crate::group::*;
        let paths = vec![
            AutoPath::new("App"),
            AutoPath::new("App/motor"),
            AutoPath::new("App/com"),
            AutoPath::new("Bsp/mcal"),
            AutoPath::new("Bsw/xmen/can"),
            AutoPath::new("Bsw/xmen/config"),
        ];
        let group = dir_groups(paths);
        group.print_kids();
        assert_eq!(group.name, "root");
    }

    // Tests for refactored dependency resolution functions

    #[test]
    fn test_check_all_resolved() {
        use crate::target::{Target, TargetKind};
        use std::collections::HashMap;

        let mut targets = HashMap::new();

        // Create two targets, both resolved
        let mut target1: Target = Target::new("app1", TargetKind::App);
        target1.is_resolved = true;
        let mut target2: Target = Target::new("lib1", TargetKind::Lib);
        target2.is_resolved = true;

        targets.insert(target1.name.clone(), target1);
        targets.insert(target2.name.clone(), target2);

        // All targets should be resolved
        assert!(Pac::check_all_resolved(&targets));
    }

    #[test]
    fn test_check_all_resolved_unresolved() {
        use crate::target::{Target, TargetKind};
        use std::collections::HashMap;

        let mut targets = HashMap::new();

        // Create one resolved, one unresolved
        let mut target1: Target = Target::new("app1", TargetKind::App);
        target1.is_resolved = true;
        let mut target2: Target = Target::new("lib1", TargetKind::Lib);
        target2.is_resolved = false;

        targets.insert(target1.name.clone(), target1);
        targets.insert(target2.name.clone(), target2);

        // Not all targets should be resolved
        assert!(!Pac::check_all_resolved(&targets));
    }

    #[test]
    fn test_build_resolved_links_no_links() {
        use crate::target::{Target, TargetKind};
        use std::collections::HashMap;

        let mut targets = HashMap::new();

        // Create target with no links
        let target: Target = Target::new("app1", TargetKind::App);
        targets.insert(target.name.clone(), target);

        let resolved_links = Pac::build_resolved_links(&targets);

        // Should have one entry with empty links
        assert_eq!(resolved_links.len(), 1);
        let key = auto_val::AutoStr::from("app1");
        assert!(resolved_links.contains_key(&key));
        assert!(resolved_links.get(&key).unwrap().is_empty());
    }
}
