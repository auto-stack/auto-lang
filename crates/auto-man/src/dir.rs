//! Dir 数据半与扫描效果（PLAN-809 T-03 起）：
//! - `Dir` 结构体与纯路径方法单一实现于 `auto-man-core/src/dir.rs`，
//!   本模块 re-export 保持旧公开路径与类型身份；
//! - `scan` 族（FS 枚举驱动的目录扫描）是效果方法，Rust 不允许跨 crate
//!   inherent impl，故以 `DirEffects` 扩展 trait 承接——`use auto_man::dir::*`
//!   或 `use auto_man::*` 即可继续 `dir.scan()` 方法语法；
//! - `check_incs`/`scan_specific_srcs`/`scan_dir_flat` 自由函数保留（T-05
//!   随源选择规则迁 core）。

use crate::AutoResult;
use crate::TargetKind;
use auto_val::{AutoStr, PathBufExt};
use log::*;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub use auto_man_core::dir::Dir;

/// Dir 的 FS 扫描效果方法（原 inherent impl 的逐字承接）。
pub trait DirEffects {
    fn scan(&mut self) -> AutoResult<()>;
    fn scan_recursive(&mut self) -> AutoResult<()>;
    fn scan_flat(&mut self) -> AutoResult<()>;
    fn scan_dir(&mut self) -> AutoResult<()>;
    fn collect_srcs(&self) -> AutoResult<Vec<AutoStr>>;
}

impl DirEffects for Dir {
    fn scan(&mut self) -> AutoResult<()> {
        if self.recurse {
            self.scan_recursive()
        } else {
            self.scan_dir()
        }
    }

    fn scan_recursive(&mut self) -> AutoResult<()> {
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

    fn scan_flat(&mut self) -> AutoResult<()> {
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

    fn scan_dir(&mut self) -> AutoResult<()> {
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

    fn collect_srcs(&self) -> AutoResult<Vec<AutoStr>> {
        let mut all_srcs = Vec::new();

        all_srcs.extend(self.srcs.clone());

        for dir in self.dirs.iter() {
            // info!("scanning sub dir: {}", dir.root);
            all_srcs.extend(dir.collect_srcs()?)
        }

        Ok(all_srcs)
    }
}

// TODO: scan recursively?
pub fn check_incs(incs: &Vec<AutoStr>, root: &AutoStr) -> AutoResult<HashSet<AutoStr>> {
    let mut succ_incs = HashSet::new();
    for inc in incs.iter() {
        let path = auto_val::AutoPath::new(inc.as_str());
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
            let rel_path = auto_val::AutoPath::new(root.as_str()).join(inc.as_str());
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
        let path = auto_val::AutoPath::new(root.as_str()).join(src.as_str());
        // info!("scanning absolute path src: {}", path);
        // if the path itself is a file, add it to the succ_srcs
        if path.is_file() {
            succ_srcs.insert(path.unified());
        } else {
            // try to find the path relative to the root dir
            let rel_path = auto_val::AutoPath::new(root.as_str()).join(src.as_str());
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
