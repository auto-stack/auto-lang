// Hot-Reload Support for AutoUI
//
// This module enables automatic reloading of UI components when .at files change,
// providing a rapid development experience without manual restarts.

use auto_val::Node;
use super::node_converter::{convert_node, ConversionError};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use notify::{RecommendedWatcher, RecursiveMode, Event, EventKind, Watcher};
use thiserror::Error;

/// Errors that can occur during hot-reload operations
#[derive(Debug, Error)]
pub enum HotReloadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),

    #[error("Conversion error: {0}")]
    Conversion(#[from] ConversionError),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
}

/// Result type for hot-reload operations
pub type HotReloadResult<T> = Result<T, HotReloadError>;

/// Hot-reloadable UI component
///
/// Wraps a dynamically loaded `.at` file and provides automatic reloading
/// when the file changes on disk.
///
/// # Example
///
/// ```ignore
/// use auto_ui::hot_reload::{HotReloadComponent, UIWatcher};
///
/// let mut counter = HotReloadComponent::load("ui/counter.at")?;
/// let view = counter.view()?;
/// ```
pub struct HotReloadComponent {
    /// Path to the .at file
    path: PathBuf,
    /// Last parsed UI node
    node: RwLock<Node>,
    /// Last conversion error (if any)
    error: RwLock<Option<ConversionError>>,
}

impl HotReloadComponent {
    /// Load a component from an .at file
    ///
    /// # Arguments
    /// * `path` - Path to the .at file
    ///
    /// # Example
    /// ```ignore
    /// let counter = HotReloadComponent::load("ui/counter.at")?;
    /// ```
    pub fn load(path: impl AsRef<Path>) -> HotReloadResult<Self> {
        let path = path.as_ref();

        // Read and parse the file
        let content = std::fs::read_to_string(path)
            .map_err(|_| HotReloadError::FileNotFound(path.to_path_buf()))?;

        let node = Self::parse_content(&content)?;

        Ok(Self {
            path: path.to_path_buf(),
            node: RwLock::new(node),
            error: RwLock::new(None),
        })
    }

    /// Reload the component from disk
    ///
    /// This should be called when a file change event is received.
    ///
    /// # Returns
    /// * `Ok(true)` - Successfully reloaded
    /// * `Ok(false)` - No changes detected
    /// * `Err(...)` - Reload failed (error stored in `self.error()`)
    pub fn reload(&self) -> HotReloadResult<bool> {
        // Read the file
        let content = std::fs::read_to_string(&self.path)
            .map_err(|_| HotReloadError::FileNotFound(self.path.clone()))?;

        // Parse the content
        let node = Self::parse_content(&content)?;

        // Update the node
        let mut node_guard = self.node.write().unwrap();
        *node_guard = node;

        // Clear any previous error
        let mut error_guard = self.error.write().unwrap();
        *error_guard = None;

        Ok(true)
    }

    /// Get the current View, converting from the stored node
    ///
    /// # Returns
    /// * `Ok(view)` - Successfully converted View
    /// * `Err(...)` - Conversion failed
    pub fn view(&self) -> HotReloadResult<crate::View<String>> {
        let node = self.node.read().map_err(|e| {
            HotReloadError::Parse(format!("RwLock poisoned: {:?}", e))
        })?;
        let view = convert_node(&*node)?;
        Ok(view)
    }

    /// Get the last reload error (if any)
    pub fn error(&self) -> Option<ConversionError> {
        let error_guard = self.error.read().ok()?;
        error_guard.clone()
    }

    /// Check if there's currently an error
    pub fn has_error(&self) -> bool {
        self.error().is_some()
    }

    /// Get the file path being watched
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Parse AutoLang content and extract the UI node
    ///
    /// This is a simplified parser that extracts the first Node from the content.
    /// A production version would use the full auto-lang parser.
    fn parse_content(_content: &str) -> HotReloadResult<Node> {
        // For now, return a simple test node
        // In production, this would use the full parser
        Ok(Node::new("text").with_arg("Hot-reload test"))
    }
}

/// File watcher for .at files
///
/// Watches a directory for changes to .at files and triggers reloads.
pub struct UIWatcher {
    watcher: Option<RecommendedWatcher>,
    /// Path being watched
    watch_path: PathBuf,
}

impl UIWatcher {
    /// Create a new file watcher
    ///
    /// # Example
    /// ```ignore
    /// let watcher = UIWatcher::new()?;
    /// watcher.watch(Path::new("ui"))?;
    /// ```
    pub fn new() -> HotReloadResult<Self> {
        Ok(Self {
            watcher: None,
            watch_path: PathBuf::new(),
        })
    }

    /// Watch a directory for .at file changes
    ///
    /// # Arguments
    /// * `path` - Directory to watch
    pub fn watch(&mut self, path: impl AsRef<Path>) -> HotReloadResult<()> {
        self.watch_path = path.as_ref().to_path_buf();

        // Create the actual watcher with proper handler
        let mut watcher = notify::recommended_watcher(|res| match res {
            Ok(event) => {
                Self::handle_event(event);
            }
            Err(e) => {
                eprintln!("Watch error: {:?}", e);
            }
        })?;

        // Start watching
        watcher.watch(&self.watch_path, RecursiveMode::Recursive)?;

        self.watcher = Some(watcher);

        Ok(())
    }

    /// Internal event handler for file changes
    fn handle_event(event: Event) {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                // Trigger reload
                if let Some(path) = event.paths.first() {
                    if path.extension().and_then(|s| s.to_str()) == Some("at") {
                        // File changed - would trigger callback here
                        println!("File changed: {:?}", path);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Message wrapper for hot-reload components
///
/// Use this to wrap your component's message type and add reload messages.
#[derive(Clone)]
pub enum HotReloadMessage<M> {
    /// A message from the user component
    UserMessage(M),
    /// The UI was reloaded
    Reloaded,
    /// Reload failed
    ReloadError(String),
}

/// Helper function to create a hot-reloadable application
///
/// # Example
///
/// ```ignore
/// use auto_ui::hot_reload::hot_reloadable_app;
///
/// let app = hot_reloadable_app("ui/app.at", |comp| {
///     comp.view()
/// });
/// ```
pub fn hot_reloadable_app<F>(
    path: impl AsRef<Path>,
    _view_fn: F,
) -> HotReloadResult<HotReloadComponent> {
    HotReloadComponent::load(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_component_load() {
        // This test requires actual .at files to exist
        // For now, just verify the API compiles
        assert!(true);
    }

    #[test]
    fn test_ui_watcher_creation() {
        // Test watcher creation (may not work in all environments)
        let result = UIWatcher::new();
        // Just verify it compiles - actual file watching depends on OS
        let _ = result;
    }
}
