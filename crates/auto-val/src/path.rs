use crate::AutoStr;
use normalize_path::NormalizePath;
use std::path::{Path, PathBuf};
use glob_match::glob_match;
pub struct AutoPath {
    pub path: PathBuf,
}

pub trait PathBufExt {
    fn unified(&self) -> AutoStr;
}

impl PathBufExt for PathBuf {
    fn unified(&self) -> AutoStr {
        let res = self.as_path().normalize().to_string_lossy().replace("\\", "/").into();
        if res == "" { ".".into() } else { res }
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
}
