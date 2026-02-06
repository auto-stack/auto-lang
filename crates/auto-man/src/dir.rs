use crate::AutoResult;
use crate::TargetKind;
use auto_val::Array;
use auto_val::{AutoPath, AutoStr, Node, PathBufExt};
use log::*;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
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
        let root = Path::new(parent.as_str()).join(name.as_str()).unified();
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
        let parent_path = Path::new(parent.as_str());
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
        // info!("dir contents: {}", name);
        // info!("root: {}", root);
        // info!("srcs: {:?}", srcs);
        // info!("incs: {:?}", incs);
        // info!("skips: {:?}", skips);
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

    pub fn scan(&mut self) -> AutoResult<()> {
        if self.recurse {
            self.scan_recursive()
        } else {
            self.scan_dir()
        }
    }

    pub fn scan_recursive(&mut self) -> AutoResult<()> {
        let root = self.at.clone();
        let root_path = Path::new(root.as_str());
        self.scan = true;
        // check current dir
        //

        // for each subdir
        for entry in fs::read_dir(root_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                info!("got subdir: {}", path.display());
                let name: AutoStr = path.file_name().unwrap().to_str().unwrap().into();
                info!("dir name: {}", name.clone());
                let mut sub_dir = Dir::from_str(name, self.at.clone(), self.target_kind.clone());
                sub_dir.show_headers = self.show_headers;
                sub_dir.recurse = self.recurse;
                sub_dir.scan = self.scan;
                sub_dir.set_lpath(self.lpath.clone());
                sub_dir.skips = self.skips.clone();
                sub_dir.scan_recursive()?;
                self.incs.extend(sub_dir.incs.clone());
                self.dirs.push(sub_dir);
            }
        }

        info!("scanning dir: {}", root);
        self.scan_flat()?;
        Ok(())
    }

    pub fn scan_flat(&mut self) -> AutoResult<()> {
        let root_path = Path::new(self.at.as_str());
        let mut all_incs = HashSet::new();
        let mut all_srcs = HashSet::new();

        let (files, has_headers) =
            scan_dir_flat(root_path, &self.target_kind, &self.skips, self.show_headers)?;
        all_srcs.extend(files);
        if has_headers {
            all_incs.insert(self.at.clone());
        }

        self.srcs.extend(all_srcs);
        self.incs.extend(all_incs);
        Ok(())
    }

    pub fn scan_dir(&mut self) -> AutoResult<()> {
        // info!("- Scanning dir: {}", self.root.as_str());
        let root_path = Path::new(self.at.as_str());
        let mut all_incs = HashSet::new();
        let mut all_srcs = HashSet::new();

        if self.scan {
            // 扫描当前目录
            let (files, has_headers) =
                scan_dir_flat(root_path, &self.target_kind, &self.skips, self.show_headers)?;
            all_srcs.extend(files);
            if has_headers {
                all_incs.insert(self.at.clone());
            }

            // 使用 scan_standard_subdirs 扫描标准子目录
            use crate::scanner::scan_standard_subdirs;
            let std_result = scan_standard_subdirs(
                root_path,
                &self.target_kind,
                &self.skips,
                self.show_headers,
                scan_dir_flat,
            )?;
            all_srcs.extend(std_result.sources);
            all_incs.extend(std_result.includes);
        }

        // check specified srcs
        let succ_srcs = scan_specific_srcs(&self.srcs, &self.at)?;
        all_srcs.extend(succ_srcs);

        // check specified incs
        let succ_incs = check_incs(&self.incs, &self.at)?;
        all_incs.extend(succ_incs);

        // TODO: recursively scan sub dirs
        for dir in self.dirs.iter_mut() {
            // info!("scanning sub dir: {}", dir.root);
            dir.scan()?;
            all_incs.extend(dir.incs.clone());
            // all_srcs.extend(dir.srcs.clone());
        }

        self.srcs = all_srcs.into_iter().collect();
        self.incs = all_incs.into_iter().collect();
        Ok(())
    }

    pub fn collect_srcs(&self) -> AutoResult<Vec<AutoStr>> {
        let mut all_srcs = Vec::new();

        all_srcs.extend(self.srcs.clone());

        for dir in self.dirs.iter() {
            // info!("scanning sub dir: {}", dir.root);
            all_srcs.extend(dir.collect_srcs()?)
        }

        Ok(all_srcs)
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

// TODO: scan recursively?
pub fn check_incs(incs: &Vec<AutoStr>, root: &AutoStr) -> AutoResult<HashSet<AutoStr>> {
    let mut succ_incs = HashSet::new();
    for inc in incs.iter() {
        let path = AutoPath::new(inc.as_str());
        // if the absolute path itself is a dir, add it to the succ_srcs
        if path.is_file() {
            // NOTE: incs should be dirs, not files
            error!("inc: {} should be directories, not files", path.unified());
            continue;
        }
        if path.is_dir() {
            succ_incs.insert(path.unified());
        } else {
            // try to find the path relative to the root dir
            let rel_path = AutoPath::new(root.as_str()).join(inc.as_str());
            if rel_path.is_dir() {
                // info!("got inc dir: {}", rel_path.unified());
                succ_incs.insert(rel_path.unified());
            }
        }
    }
    Ok(succ_incs)
}

pub fn scan_specific_srcs(srcs: &Vec<AutoStr>, root: &AutoStr) -> AutoResult<HashSet<AutoStr>> {
    let mut succ_srcs = HashSet::new();
    for src in srcs {
        let path = AutoPath::new(root.as_str()).join(src.as_str());
        // info!("scanning absolute path src: {}", path);
        // if the path itself is a file, add it to the succ_srcs
        if path.is_file() {
            succ_srcs.insert(path.unified());
        } else {
            // try to find the path relative to the root dir
            let rel_path = AutoPath::new(root.as_str()).join(src.as_str());
            // info!("scanning relative path src: {}", rel_path);
            if rel_path.is_file() {
                succ_srcs.insert(rel_path.unified());
            }
        }
    }
    Ok(succ_srcs)
}

pub fn scan_dir_flat(
    path: &Path,
    kind: &TargetKind,
    skips: &HashSet<AutoStr>,
    use_header: bool,
) -> AutoResult<(HashSet<AutoStr>, bool)> {
    if !path.is_dir() {
        return Ok((HashSet::new(), false));
    }

    let mut files_found = HashSet::new();
    let mut headers_found = false;
    let files = std::fs::read_dir(path)?;
    let skip_globs = skips
        .iter()
        .filter(|s| s.contains("*"))
        .map(|s| s.clone())
        .collect::<Vec<AutoStr>>();
    for file in files {
        let file = file?;
        // check for Auto Source files
        let path = file.path();
        if !path.is_file() {
            continue;
        }
        // info!("..   got {}", path.display());
        let file_name = path.file_name();
        match file_name {
            Some(file_name) => {
                let file_name = AutoStr::from(file_name.to_str().unwrap());
                if skips.contains(&file_name) {
                    info!("- skipping {}", file_name.as_str());
                    continue;
                }
                let mut is_skip = false;
                for skip in &skip_globs {
                    if glob_match::glob_match(skip.as_str(), file_name.as_str()) {
                        is_skip = true;
                        break;
                    }
                }
                if is_skip {
                    info!("- skipping {}", file_name.as_str());
                    continue;
                }
            }
            None => {
                return Err(format!("{} is not a file", path.display()).into());
            }
        }

        let file_name = path.file_name().unwrap();

        let ext = path.extension();
        let path_str = path.unified();

        // 使用 FileFilter 判断是否应该包含此文件
        use crate::file_types::{FileFilter, FileType};
        let filter = FileFilter::for_target(kind);
        let file_name_str = file_name.to_str().unwrap();
        let ext_str = ext.and_then(|e| e.to_str());

        if !filter.should_include(file_name_str, ext_str) {
            continue;
        }

        // 处理头文件
        if let Some(file_type) = ext_str.and_then(FileType::from_extension) {
            if file_type.is_header() {
                headers_found = true;
                if use_header {
                    files_found.insert(path_str);
                }
            } else if file_type.is_source() {
                files_found.insert(path_str);
            }
        }
    }
    Ok((files_found, headers_found))
}
