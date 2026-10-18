//! Target 数据结构与纯声明/选择/投影规则（PLAN-809 T-03 自
//! auto-man/src/target.rs 迁入纯半；transpile/scan/clean 等语言与 FS
//! 效果方法留在 auto-man 的 `TargetEffects` 扩展 trait）。

use crate::dir::Dir;
use crate::group::Group;
use crate::node_ext::NodeExt;
use crate::port::Port;
use auto_atom::Atom;
use auto_val::{Array, AutoPath, AutoStr, Node, Obj, PathBufExt, Value};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::path::Path;

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

/// 目标根目录规则。`dir_exists` 是唯一的观测注入点（原臂为
/// `Path::new(&app_dir).is_dir()` 的直接 FS 探针；PLAN-809 合同 §1.3：
/// 2 参 `get_root_dir`/`Target::from` 兼容入口在 core 内保留同一直接
/// 探针以冻结公开签名与行为，纯测试/适配路径用 `*_with_probe` 注入）。
pub fn get_root_dir(name: &AutoStr, kind: &TargetKind, pac: &AutoStr) -> AutoStr {
    get_root_dir_with_probe(name, kind, pac, |dir| Path::new(dir).is_dir())
}

pub fn get_root_dir_with_probe(
    name: &AutoStr,
    kind: &TargetKind,
    pac: &AutoStr,
    dir_exists: impl Fn(&str) -> bool,
) -> AutoStr {
    // 如果target的名称和pac名称相同，说明是主target，使用当前目录
    if name == pac {
        return AutoStr::from(".");
    }

    // 对于 app 类型，如果名称是 "main" 或与 pac 相同，
    // 先尝试 apps/<name>，如果不存在则回退到当前目录 "."
    if *kind == TargetKind::App && (name.as_str() == "main" || name == pac) {
        let app_dir = format!("apps/{}", name);
        if dir_exists(&app_dir) {
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

    /// 声明归一（公开签名冻结：与旧臂一致，含 `apps/<name>` 直接 FS 探针）。
    pub fn from(node: Node, pac: AutoStr) -> Self {
        Self::from_with_probe(node, pac, |dir| Path::new(dir).is_dir())
    }

    /// 声明归一的纯入口（观测注入；测试与适配路径用）。
    pub fn from_with_probe(node: Node, pac: AutoStr, dir_exists: impl Fn(&str) -> bool) -> Self {
        // Extract basic information
        let (mut name, version) = Self::extract_name_and_version(&node);
        let kind = Self::extract_kind(&node);
        let origin = Self::extract_origin(&node);
        let from: AutoStr = if node.has_prop("path") {
            node.get_str_or("path", "").into()
        } else if node.has_prop("git") {
            node.get_str_or("git", "").into()
        } else {
            node.get_str_or("from", "").into()
        };

        if name.is_empty() && !from.is_empty() {
            if let Some(file_name) = std::path::Path::new(from.as_str()).file_name() {
                name = file_name.to_string_lossy().to_string().into();
            }
        }

        // Extract flags using NodeExt
        let is_scan = node.get_bool_or("scan", true);
        let recurse = node.get_bool_or("recurse", false);

        // Calculate location
        let (at, rename) =
            Self::calculate_location_with_probe(&node, &name, &kind, &pac, &dir_exists);

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

        if name.is_empty() {
            let main_val = node.main_arg();
            if !main_val.is_nil() {
                name = main_val.to_astr();
            }
        }

        if name.is_empty() && node.has_prop("name") {
            name = node.get_prop("name").to_astr();
        }

        if version.is_empty() {
            if node.has_prop("_arg1") {
                version = node.get_prop("_arg1").to_astr();
            } else if node.has_prop("version") {
                version = node.get_prop("version").to_astr();
            } else {
                version = "latest".into();
            }
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
        if let Some(arg) = &node.args.get_arg(&"kind".into()) {
            let mode = arg.get_val().to_astr();
            match mode.as_str() {
                "local" => return TargetOrigin::Local,
                "git" => return TargetOrigin::Git,
                _ => {}
            }
        }

        if node.has_prop("path") {
            return TargetOrigin::Local;
        }

        if node.has_prop("git") {
            return TargetOrigin::Git;
        }

        let from = node.get_str_or("from", "");
        if !from.is_empty() {
            if from.starts_with("git@")
                || from.starts_with("http://")
                || from.starts_with("https://")
            {
                return TargetOrigin::Git;
            } else {
                return TargetOrigin::Local;
            }
        }

        TargetOrigin::Index
    }

    /// Calculate the target location and rename
    fn calculate_location_with_probe(
        node: &Node,
        name: &AutoStr,
        kind: &TargetKind,
        pac: &AutoStr,
        dir_exists: &impl Fn(&str) -> bool,
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
            get_root_dir_with_probe(name, kind, pac, dir_exists)
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

        // Plan 375: expose deps grouped by kind as array props so mold templates
        // can iterate them (e.g. IAR `.ewp` does `@ for dev in devices { ... }`
        // and reads `dev.icf`). Previously these were only set to *empty* arrays
        // as a fallback when the kind was absent — so an app whose deps include
        // a Device ended up with NO `devices` prop at all, yielding
        // `Undefined variable: devices` at template render time.
        let mut dep_devices = Vec::new();
        let mut dep_deps = Vec::new();
        let mut dep_bags = Vec::new();
        for dep in &self.deps {
            let dep_node = Value::node(dep.to_node());
            match dep.kind {
                TargetKind::Device => dep_devices.push(dep_node.clone()),
                TargetKind::Dep => dep_deps.push(dep_node.clone()),
                TargetKind::Bag => dep_bags.push(dep_node.clone()),
                _ => {}
            }
        }
        node.set_prop("devices", Value::Array(Array::from_vec(dep_devices)));
        node.set_prop("deps", Value::Array(Array::from_vec(dep_deps)));
        node.set_prop("bags", Value::Array(Array::from_vec(dep_bags)));

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

    // TODO: only allow args to be overrided
    pub fn merge_link(&mut self, link: &Node) {
        // currently only support normal props override
        for (k, v) in link.props_iter() {
            self.props.set(k.clone(), v.clone());
        }
    }

    pub fn merge(&mut self, other: &Target) {
        self.merge_with_probe(other, |dir| Path::new(dir).is_dir())
    }

    /// merge 的纯入口（观测注入；原臂在 `self.at` join 下探测 `is_dir`
    /// 决定 dep dirs 是否 append_root）。
    pub fn merge_with_probe(&mut self, other: &Target, dir_exists: impl Fn(&str) -> bool) {
        self.props.merge(&other.props);
        self.autos.extend(other.autos.clone());
        // TODO: merge dirs
        for (k, v) in other.dirs.iter() {
            let mut dir = v.clone();
            // check new root dir
            let path = Path::new(self.at.as_str()).join(dir.at.as_str());
            if dir_exists(&path.to_string_lossy()) {
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
    pub fn check_links(&mut self) -> auto_val::AutoResult<Vec<Node>> {
        // 2. If the targets links to a dep target, recursively resolve the linked target
        let links = std::mem::take(&mut self.links);
        Ok(links)
    }

    pub fn apply_link(&mut self, _link: &Node) {}

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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_node(name: &str, pac: &str) -> Node {
        let mut node = Node::new("app");
        node.args.add_pos(Value::Str(name.into()));
        node.id = "".into();
        let _ = pac;
        node
    }

    #[test]
    fn target_from_node_basic() {
        let node = app_node("main", "test");
        let target = Target::from_with_probe(node, "test".into(), |_| false);
        assert_eq!(target.name, "main");
        assert_eq!(target.kind, TargetKind::App);
        // app "main" ≠ pac 名，probe=false → 回落 "."
        assert_eq!(target.at, ".");
        assert_eq!(target.is_main, false);
        assert_eq!(target.version, "latest");
        assert_eq!(target.lang, "c");
    }

    #[test]
    fn root_dir_apps_main_probe_branch() {
        // app "main" ≠ pac 名：probe 决定 apps/main vs "."
        let at = get_root_dir_with_probe(&"main".into(), &TargetKind::App, &"demo".into(), |d| {
            d == "apps/main"
        });
        assert_eq!(at, "apps/main");
        let at =
            get_root_dir_with_probe(&"main".into(), &TargetKind::App, &"demo".into(), |_| false);
        assert_eq!(at, ".");
        // 其它名字按 kind 目录
        let at = get_root_dir_with_probe(&"demo".into(), &TargetKind::App, &"x".into(), |_| true);
        assert_eq!(at, "apps/demo");
        let at = get_root_dir_with_probe(&"demo".into(), &TargetKind::Dep, &"x".into(), |_| true);
        assert_eq!(at, "deps/demo");
        // 名字同 pac → "."（app 规则不触发）
        let at = get_root_dir_with_probe(&"x".into(), &TargetKind::Lib, &"x".into(), |_| true);
        assert_eq!(at, ".");
    }

    #[test]
    fn libname_by_platform() {
        let mut t = Target::new("foo", TargetKind::Lib);
        t.port.platform = "windows".into();
        assert_eq!(t.libname(), "foo.lib");
        t.port.platform = "linux".into();
        assert_eq!(t.libname(), "libfoo.a");
    }

    #[test]
    fn selects_and_defaults_union() {
        let mut node = Node::new("lib");
        node.args.add_pos(Value::Str("l".into()));
        node.set_prop(
            "selects",
            Value::str_array(vec![AutoStr::from("a"), AutoStr::from("c")]),
        );
        node.set_prop("defaults", Value::str_array(vec![AutoStr::from("d")]));
        let t = Target::from_with_probe(node, "pac".into(), |_| false);
        assert!(t.selects.contains(&AutoStr::from("a")));
        assert!(t.selects.contains(&AutoStr::from("c")));
        assert!(t.selects.contains(&AutoStr::from("d")));
        assert_eq!(t.kind, TargetKind::Lib);
    }

    #[test]
    fn merge_link_overrides_props() {
        let mut t = Target::new("x", TargetKind::App);
        t.props.set("opt", Value::Int(1));
        let mut link = Node::new("link");
        link.args.add_pos(Value::Str("dep1".into()));
        link.set_prop("opt", Value::Int(2));
        t.merge_link(&link);
        assert_eq!(
            t.props.get("opt").map(|v| v.to_astr()),
            Some(AutoStr::from("2"))
        );
    }
}
