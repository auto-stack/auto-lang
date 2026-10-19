//! Dir 数据结构与纯路径方法（PLAN-809 T-03 自 auto-man/src/dir.rs 迁入
//! 数据半；`scan` 族 FS 枚举方法留在 auto-man 的 `DirEffects` 扩展 trait，
//! 选择规则本体在 T-05 迁入）。

use crate::target::TargetKind;
use auto_val::{Array, AutoPath, AutoStr, Node, PathBufExt};
use log::debug;
use std::hash::{Hash, Hasher};
use std::{collections::HashSet, fmt};

#[derive(Clone)]
pub struct Dir {
    /// Name of this directory. TODO: should not be a path?
    pub name: AutoStr,
    /// Root directory of this directory. This is the real path in local file system .
    // TODO: This should replace the local name of the directory?
    pub at: AutoStr,
    /// Logical path of this directory. Logical path are made of {target_name}/{dir_name}/{dir_name}s
    pub lpath: AutoStr,
    /// Relative path of this directory.
    pub rpath: AutoStr,
    /// Whether to scan regular source directories like `src`, `include` and `inc`.
    pub scan: bool,
    /// Whether to recursively scan all subdirectories.
    pub recurse: bool,
    /// Whether to recognize header files as source files.
    ///  - if set `true`, header files are put into `srcs`
    ///  - if set `false`, only directory names are put into `incs`
    pub show_headers: bool,
    pub target_kind: TargetKind,
    // TODO: add sub dirs
    pub dirs: Vec<Dir>,
    pub srcs: Vec<AutoStr>,
    pub incs: Vec<AutoStr>,
    pub skips: HashSet<AutoStr>,
}

impl Dir {
    pub fn from_str(name: AutoStr, parent: AutoStr, target_kind: TargetKind) -> Self {
        let name = name;
        let root = std::path::Path::new(parent.as_str())
            .join(name.as_str())
            .unified();
        debug!("making Dir {}::{} with root: {}", parent, name, root);
        Self {
            name: name.clone(),
            at: root,
            lpath: AutoStr::new(),
            rpath: name,
            scan: true,
            recurse: false,
            show_headers: false,
            target_kind,
            dirs: Vec::new(),
            srcs: Vec::new(),
            incs: Vec::new(),
            skips: HashSet::new(),
        }
    }

    pub fn from_node(node: &Node, parent: AutoStr, target_kind: TargetKind) -> Self {
        use crate::node_ext::NodeExt;

        // Get directory name from main argument (works for both `dir("name")` and `dir name`)
        let name = node.main_arg().to_astr();
        let parent_path = std::path::Path::new(parent.as_str());
        let rel_path = node.get_str_or("at", name.as_str());
        let at = parent_path.join(rel_path.as_str()).unified();

        let scan = node.get_bool_or("scan", true);
        let recurse = node.get_bool_or("recurse", false);
        let headers = node.get_bool_or("headers", false);

        // TODO read srcs, incs, skips from node
        let mut dirs = node
            .get_prop("dirs")
            .to_str_vec()
            .into_iter()
            .map(|s| Dir::from_str(s, at.clone(), target_kind.clone()))
            .collect::<Vec<Dir>>();
        // read sub dirs by dir() node
        let dir_nodes = node.nodes(&"dir");
        for dir_node in dir_nodes {
            let dir = Dir::from_node(dir_node, at.clone(), target_kind.clone());
            dirs.push(dir);
        }

        let srcs = node.get_str_vec_or("srcs");
        let incs = node.get_str_vec_or("incs");
        let skips = node
            .get_str_vec_or("skips")
            .into_iter()
            .collect::<HashSet<AutoStr>>();
        Self {
            name,
            at,
            lpath: AutoStr::new(),
            rpath: rel_path,
            scan,
            recurse,
            show_headers: headers,
            target_kind,
            dirs,
            srcs,
            incs,
            skips,
        }
    }

    pub fn set_lpath(&mut self, parent_lpath: AutoStr) {
        self.lpath = if parent_lpath.is_empty() {
            self.name.clone()
        } else {
            format!("{}/{}", parent_lpath, self.name).into()
        };

        for d in &mut self.dirs {
            d.set_lpath(self.lpath.clone());
        }
    }

    pub fn logical_depths(&self) -> usize {
        AutoPath::new(self.lpath.clone()).depth()
    }

    pub fn update_root(&mut self, root: AutoStr) {
        // set dir's root
        let old_root = self.at.clone();
        self.at = AutoPath::new(root).join(old_root).to_astr();
        // recursively set root of subdirs
        for dir in self.dirs.iter_mut() {
            dir.update_root(self.at.clone());
        }
    }

    pub fn to_node(&self) -> Node {
        let mut node = Node::new("dir");
        node.args.add_pos(self.name.clone());
        node.set_prop("root", self.at.clone());
        node.set_prop("srcs", self.srcs.clone());
        node.set_prop("incs", self.incs.clone());
        node.set_prop("lpath", self.lpath.clone());
        // relative path
        node.set_prop("rpath", self.rpath.clone());
        node.set_prop(
            "skips",
            self.skips.clone().into_iter().collect::<Vec<AutoStr>>(),
        );

        // add sub dirs
        if self.dirs.is_empty() {
            node.set_prop("dirs", Array::new());
        }
        for dir in &self.dirs {
            let n = dir.to_node();
            node.add_kid(n);
        }

        node
    }

    pub fn append_root(&mut self, root: AutoStr) {
        self.at = AutoPath::new(root.as_str())
            .join(self.at.as_str())
            .unified();

        for dir in self.dirs.iter_mut() {
            dir.append_root(root.clone());
        }
    }
}

impl PartialEq for Dir {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.at == other.at
    }
}

impl Eq for Dir {}

impl Hash for Dir {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.at.hash(state);
    }
}

impl fmt::Debug for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = AutoPath::new(self.at.as_str());
        write!(f, "{}", path.unified())
    }
}
