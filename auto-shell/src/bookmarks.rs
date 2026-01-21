use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Manages directory bookmarks ensuring persistence
pub struct BookmarkManager {
    bookmarks: HashMap<String, PathBuf>,
    file_path: PathBuf,
}

impl BookmarkManager {
    /// Create a new bookmark manager
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let file_path = home.join(".auto-shell-bookmarks");

        let mut manager = Self {
            bookmarks: HashMap::new(),
            file_path,
        };

        // Load existing bookmarks
        let _ = manager.load();

        manager
    }

    /// Load bookmarks from file
    pub fn load(&mut self) -> std::io::Result<()> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.file_path)?;

        self.bookmarks.clear();
        for line in content.lines() {
            if let Some((name, path)) = line.split_once('=') {
                self.bookmarks
                    .insert(name.trim().to_string(), PathBuf::from(path.trim()));
            }
        }

        Ok(())
    }

    /// Save bookmarks to file
    pub fn save(&self) -> std::io::Result<()> {
        let mut content = String::new();

        // Sort for stability
        let mut pairs: Vec<(&String, &PathBuf)> = self.bookmarks.iter().collect();
        pairs.sort_by_key(|(k, _)| *k);

        for (name, path) in pairs {
            content.push_str(&format!("{}={}\n", name, path.display()));
        }

        fs::write(&self.file_path, content)
    }

    /// Add a bookmark
    pub fn add(&mut self, name: String, path: PathBuf) -> std::io::Result<()> {
        self.bookmarks.insert(name, path);
        self.save()
    }

    /// Delete a bookmark
    pub fn del(&mut self, name: &str) -> std::io::Result<bool> {
        if self.bookmarks.remove(name).is_some() {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get a bookmark path
    pub fn get(&self, name: &str) -> Option<&PathBuf> {
        self.bookmarks.get(name)
    }

    /// List all bookmarks
    pub fn list(&self) -> Vec<(&String, &PathBuf)> {
        let mut list: Vec<_> = self.bookmarks.iter().collect();
        list.sort_by_key(|(k, _)| *k);
        list
    }
}
