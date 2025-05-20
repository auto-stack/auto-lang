use crate::{AutoResult, AutoStr};
use glob_match::glob_match;
use normalize_path::NormalizePath;
use std::fmt;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AutoPath {
    pub path: PathBuf,
}

pub trait PathBufExt {
    fn unified(&self) -> AutoStr;
}

impl PathBufExt for PathBuf {
    fn unified(&self) -> AutoStr {
        let res = self
            .as_path()
            .normalize()
            .to_string_lossy()
            .replace("\\", "/")
            .into();
        if res == "" {
            ".".into()
        } else {
            res
        }
    }
}

impl AutoPath {
    pub fn crate_root() -> Self {
        let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        Self::new(path)
    }
}

impl AutoPath {
    pub fn new(path: impl Into<AutoStr>) -> Self {
        let s = path.into();
        let path = Path::new(s.as_str()).to_path_buf();
        Self { path }
    }

    pub fn is_file(&self) -> bool {
        self.path.is_file()
    }

    pub fn is_dir(&self) -> bool {
        self.path.is_dir()
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn unified(&self) -> AutoStr {
        self.path().to_path_buf().unified()
    }

    pub fn filename(&self) -> AutoStr {
        let n = self.path.file_name();
        if let Some(n) = n {
            if let Some(n) = n.to_str() {
                return n.into();
            }
        }
        AutoStr::default()
    }

    pub fn join(&self, path: impl Into<AutoStr>) -> Self {
        let s = path.into();
        let path = self.path.join(s.as_str());
        Self { path }
    }

    pub fn to_astr(&self) -> AutoStr {
        self.path.unified()
    }

    pub fn matches_suffix(&self, suffix: &str) -> bool {
        let s = self.path.file_name();
        if let Some(s) = s {
            if let Some(s) = s.to_str() {
                // use regex to match the suffix with the pattern
                return glob_match(suffix, s);
            }
        }
        false
    }

    pub fn ext(&self) -> AutoStr {
        let s = self.path.extension();
        if let Some(s) = s {
            if let Some(s) = s.to_str() {
                return s.into();
            }
        }
        AutoStr::default()
    }

    pub fn exts(&self, n: usize) -> Vec<AutoStr> {
        let mut res = Vec::new();
        let s = self.path.file_name();
        if let Some(s) = s {
            let s = s.to_str();
            if let Some(s) = s {
                let mut s = s.split(".").collect::<Vec<&str>>();
                // reverse the vector
                s.reverse();
                for i in 0..n {
                    // skip the last one as it is the file name
                    if i < s.len() - 1 {
                        res.push(s[i].into());
                    } else {
                        break;
                    }
                }
            }
        }
        res
    }

    pub fn normalized(&self) -> AutoPath {
        let path = self.path.unified();
        AutoPath::new(path)
    }

    pub fn parent(&self) -> AutoPath {
        let path = self.path.parent();
        if let Some(path) = path {
            AutoPath::from(path.to_path_buf())
        } else {
            self.clone()
        }
    }

    pub fn head(&self) -> AutoPath {
        let c = self.path.components().next();
        match c {
            Some(c) => match c {
                Component::Normal(s) => AutoPath::from(s.to_str().unwrap()),
                Component::CurDir => AutoPath::from("."),
                Component::ParentDir => AutoPath::from(".."),
                Component::RootDir => AutoPath::from("/"),
                _ => self.clone(),
            },
            None => self.clone(),
        }
    }

    pub fn tail(&self, n: usize) -> AutoPath {
        let c: Vec<Component> = self.path.components().rev().take(n).collect();
        let p = c
            .iter()
            .rev()
            .map(|c| c.as_os_str().to_str().unwrap())
            .collect::<Vec<&str>>()
            .join("/");
        AutoPath::new(p)
    }

    pub fn reverse_relative(&self) -> AutoStr {
        // check number of parts in this unified path
        let level = self.unified().split("/").count();
        "../".repeat(level).into()
    }

    /// Get the depth of this path
    pub fn depth(&self) -> usize {
        self.unified().split("/").count()
    }

    /// Check if this path has children
    pub fn is_empty(&self) -> AutoResult<bool> {
        Ok(self.path().read_dir()?.next().is_none())
    }

    pub fn abs(&self) -> AutoStr {
        if self.path.is_absolute() {
            return self.unified();
        } else {
            let abs_path = std::path::absolute(&self.path);
            if abs_path.is_err() {
                return self.unified();
            }
            Self::from(abs_path.unwrap()).to_astr()
        }
    }
}

/// File system operations
impl AutoPath {
    pub fn clean_with_parents(&mut self) -> AutoResult<()> {
        // 0. check wether the directory is sub folder of working directory
        let cwd = AutoPath::from(std::env::current_dir()?).abs();
        let abs_path = self.abs();
        if !abs_path.starts_with(cwd.as_str()) {
            return Err(format!(
                "Path {} is not a subdirectory of working directory {}",
                abs_path, cwd
            )
            .into());
        }
        // 1. remove directory at the path
        println!("removing ... {}", self.path().display());
        std::fs::remove_dir_all(self.path())?;
        // 2. recursively check parents: if empty, remove directory
        let mut parent = self.parent();
        while parent.to_astr() != "." && parent.unified() != cwd {
            if parent.is_empty()? {
                println!("removing ... {}", parent.path().display());
                std::fs::remove_dir_all(parent.path())?;
                parent = parent.parent();
            } else {
                break;
            }
        }
        Ok(())
    }
}

impl From<PathBuf> for AutoPath {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl From<&str> for AutoPath {
    fn from(path: &str) -> Self {
        Self::new(path)
    }
}

impl fmt::Display for AutoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_astr())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_astr() {
        let path = "test.txt";
        let ap = AutoPath::new(path);
        assert_eq!(ap.to_astr(), "test.txt");
    }

    #[test]
    fn test_matches_suffix() {
        let path = "test.at.txt";
        let pat = "*.at.*";
        let ap = AutoPath::new(path);
        assert_eq!(ap.matches_suffix(pat), true);

        let path = "test.as.txt";
        let ap = AutoPath::new(path);
        assert_eq!(ap.matches_suffix(pat), false);

        let path = "test.txt";
        let ap = AutoPath::new(path);
        assert_eq!(ap.matches_suffix(pat), false);

        let path = "test.at.txt.back";
        let ap = AutoPath::new(path);
        assert_eq!(ap.matches_suffix(pat), true);
    }

    #[test]
    fn test_exts() {
        let path = "test.at.txt";
        let ap = AutoPath::new(path);
        assert_eq!(ap.exts(1), vec!["txt"]);
        assert_eq!(ap.exts(2), vec!["txt", "at"]);
        assert_eq!(ap.exts(3), vec!["txt", "at"]);
        assert_eq!(ap.exts(4), vec!["txt", "at"]);
    }

    #[test]
    fn test_workspace_root() {
        let path = AutoPath::crate_root();
        assert_eq!(path.filename(), "auto-val");
    }

    #[test]
    fn test_head() {
        let path = AutoPath::new("lib/servcie/test.txt");
        let head = path.head();
        assert_eq!(head.to_astr(), "lib");
    }

    #[test]
    fn test_relative_location() {
        let path = AutoPath::new("crates/auto-lang");
        let relative_loc = path.reverse_relative();
        assert_eq!(relative_loc, "../../");
    }

    #[test]
    fn test_parents() {
        let path = AutoPath::new("build/lanshan/iar");
        let mut parent = path.parent();
        for _ in 0..4 {
            println!("{}", parent.to_astr());
            parent = parent.parent();
        }
    }

    #[test]
    fn test_abs() {
        let path = "./Cargo.toml";
        println!("{}", AutoPath::new(path).abs());
    }
}
