# Plan 134: UI Incremental Compilation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement incremental compilation for UI code generation (Vue/Jet) so that only changed `.at` files are regenerated, using existing AIE infrastructure.

**Architecture:**
- Reuse AIE's `Database` for file hashing and dirty tracking
- New `UICache` module wraps Database with UI-specific methods
- Persistent cache stored in `.auto/ui-cache.json`
- `auto run` detects changes and only regenerates dirty widgets

**Tech Stack:** Rust, blake3, serde_json, existing AIE infrastructure

---

## Overview

Currently, `auto run` regenerates ALL `.vue`/`.kt` files every time, even if source `.at` files haven't changed. This plan adds incremental compilation:

```
source/front/app.at  →  hash: abc123
                        ↓
                   .auto/ui-cache.json
                        {
                          "app.at": { "hash": "abc123", "widgets": ["App"] }
                        }
                        ↓
                   Compare hashes, only regenerate changed files
```

---

## Task 1: Extend ArtifactType for UI Backends

**Files:**
- Modify: `crates/auto-lang/src/database.rs:118-125`

**Step 1: Check current ArtifactType definition**

Run: `grep -n "pub enum ArtifactType" crates/auto-lang/src/database.rs`
Expected: Shows line 118 with CSource, CHeader, RustSource variants

**Step 2: Add UI artifact types**

In `crates/auto-lang/src/database.rs`, extend `ArtifactType`:

```rust
/// Type of transpilation artifact
#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactType {
    CSource,
    CHeader,
    RustSource,
    // UI backends (Plan 134)
    VueComponent,
    KotlinFile,
}
```

**Step 3: Verify compilation**

Run: `cargo check -p auto-lang`
Expected: No errors

**Step 4: Commit**

```bash
git add crates/auto-lang/src/database.rs
git commit -m "feat(database): add VueComponent and KotlinFile artifact types"
```

---

## Task 2: Create UIArtifact Structure

**Files:**
- Create: `crates/auto-lang/src/database/ui_artifact.rs`
- Modify: `crates/auto-lang/src/database.rs` (add mod declaration)

**Step 1: Create ui_artifact.rs**

Create file `crates/auto-lang/src/database/ui_artifact.rs`:

```rust
//! UI Artifact for incremental code generation (Plan 134)
//!
//! Tracks generated UI files (.vue, .kt) for incremental compilation.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// A generated UI artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIArtifact {
    /// Source .at file path (relative to project root)
    pub source_path: PathBuf,
    /// Widget name extracted from source
    pub widget_name: String,
    /// Generated output file path (relative to output directory)
    pub output_path: PathBuf,
    /// Hash of source file content (BLAKE3 truncated to u64)
    pub source_hash: u64,
    /// Hash of generated content
    pub content_hash: u64,
    /// Target backend
    pub backend: UIBackend,
}

/// UI backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UIBackend {
    Vue,
    Jet,
    Tauri,
}

impl std::fmt::Display for UIBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UIBackend::Vue => write!(f, "vue"),
            UIBackend::Jet => write!(f, "jet"),
            UIBackend::Tauri => write!(f, "tauri"),
        }
    }
}

impl std::str::FromStr for UIBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vue" => Ok(UIBackend::Vue),
            "jet" => Ok(UIBackend::Jet),
            "tauri" => Ok(UIBackend::Tauri),
            _ => Err(format!("Unknown UI backend: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_backend_display() {
        assert_eq!(UIBackend::Vue.to_string(), "vue");
        assert_eq!(UIBackend::Jet.to_string(), "jet");
        assert_eq!(UIBackend::Tauri.to_string(), "tauri");
    }

    #[test]
    fn test_ui_backend_from_str() {
        assert_eq!("vue".parse::<UIBackend>().unwrap(), UIBackend::Vue);
        assert_eq!("JET".parse::<UIBackend>().unwrap(), UIBackend::Jet);
    }
}
```

**Step 2: Add mod declaration in database.rs**

In `crates/auto-lang/src/database.rs`, add at the top after imports:

```rust
// Plan 134: UI Artifact support
mod ui_artifact;
pub use ui_artifact::{UIArtifact, UIBackend};
```

**Step 3: Verify compilation**

Run: `cargo check -p auto-lang`
Expected: No errors

**Step 4: Run tests**

Run: `cargo test -p auto-lang ui_artifact`
Expected: 2 tests pass

**Step 5: Commit**

```bash
git add crates/auto-lang/src/database/ui_artifact.rs crates/auto-lang/src/database.rs
git commit -m "feat(database): add UIArtifact and UIBackend types for incremental UI compilation"
```

---

## Task 3: Create UICache Module

**Files:**
- Create: `crates/auto-lang/src/database/ui_cache.rs`
- Modify: `crates/auto-lang/src/database.rs` (add mod declaration)

**Step 1: Create ui_cache.rs**

Create file `crates/auto-lang/src/database/ui_cache.rs`:

```rust
//! UI Cache for incremental code generation (Plan 134)
//!
//! Manages persistent cache of generated UI files for incremental compilation.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::ui_artifact::UIArtifact;

/// Persistent cache for UI incremental compilation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UICache {
    /// File path → content hash
    file_hashes: HashMap<PathBuf, u64>,
    /// File path → generated artifacts
    artifacts: HashMap<PathBuf, Vec<UIArtifact>>,
    /// Cache version for migration
    version: u32,
}

impl UICache {
    const VERSION: u32 = 1;

    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            file_hashes: HashMap::new(),
            artifacts: HashMap::new(),
            version: Self::VERSION,
        }
    }

    /// Get cache file path for a project
    pub fn cache_path(project_root: &Path) -> PathBuf {
        project_root.join(".auto").join("ui-cache.json")
    }

    /// Load cache from project root
    pub fn load(project_root: &Path) -> Self {
        let path = Self::cache_path(project_root);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<Self>(&content) {
                        Ok(cache) => {
                            // Version check - invalidate if version mismatch
                            if cache.version == Self::VERSION {
                                return cache;
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse UI cache: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read UI cache: {}", e);
                }
            }
        }
        Self::new()
    }

    /// Save cache to project root
    pub fn save(&self, project_root: &Path) -> std::io::Result<()> {
        let path = Self::cache_path(project_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&path, content)
    }

    /// Check if a file needs regeneration
    pub fn is_dirty(&self, source_path: &Path, current_hash: u64) -> bool {
        match self.file_hashes.get(source_path) {
            Some(&cached_hash) => cached_hash != current_hash,
            None => true,
        }
    }

    /// Get artifacts for a source file
    pub fn get_artifacts(&self, source_path: &Path) -> Option<&[UIArtifact]> {
        self.artifacts.get(source_path).map(|v| v.as_slice())
    }

    /// Update cache entry for a source file
    pub fn update(&mut self, source_path: PathBuf, hash: u64, artifacts: Vec<UIArtifact>) {
        self.file_hashes.insert(source_path.clone(), hash);
        self.artifacts.insert(source_path, artifacts);
    }

    /// Remove a file from cache
    pub fn remove(&mut self, source_path: &Path) {
        self.file_hashes.remove(source_path);
        self.artifacts.remove(source_path);
    }

    /// Get all tracked source files
    pub fn tracked_files(&self) -> impl Iterator<Item = &PathBuf> {
        self.file_hashes.keys()
    }

    /// Get number of tracked files
    pub fn file_count(&self) -> usize {
        self.file_hashes.len()
    }

    /// Get total number of artifacts
    pub fn artifact_count(&self) -> usize {
        self.artifacts.values().map(|v| v.len()).sum()
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.file_hashes.clear();
        self.artifacts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_new() {
        let cache = UICache::new();
        assert_eq!(cache.file_count(), 0);
        assert_eq!(cache.artifact_count(), 0);
    }

    #[test]
    fn test_is_dirty_new_file() {
        let cache = UICache::new();
        let path = PathBuf::from("app.at");
        assert!(cache.is_dirty(&path, 12345));
    }

    #[test]
    fn test_is_dirty_unchanged_file() {
        let mut cache = UICache::new();
        let path = PathBuf::from("app.at");
        cache.update(path.clone(), 12345, vec![]);
        assert!(!cache.is_dirty(&path, 12345));
    }

    #[test]
    fn test_is_dirty_changed_file() {
        let mut cache = UICache::new();
        let path = PathBuf::from("app.at");
        cache.update(path.clone(), 12345, vec![]);
        assert!(cache.is_dirty(&path, 99999));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = UICache::new();

        let path = PathBuf::from("app.at");
        let artifact = UIArtifact {
            source_path: path.clone(),
            widget_name: "App".to_string(),
            output_path: PathBuf::from("src/components/App.vue"),
            source_hash: 12345,
            content_hash: 67890,
            backend: super::UIBackend::Vue,
        };

        cache.update(path.clone(), 12345, vec![artifact]);
        cache.save(temp_dir.path()).unwrap();

        let loaded = UICache::load(temp_dir.path());
        assert_eq!(loaded.file_count(), 1);
        assert!(!loaded.is_dirty(&path, 12345));
    }
}
```

**Step 2: Add mod declaration in database.rs**

In `crates/auto-lang/src/database.rs`, add after `mod ui_artifact;`:

```rust
mod ui_cache;
pub use ui_cache::UICache;
```

**Step 3: Verify compilation**

Run: `cargo check -p auto-lang`
Expected: No errors

**Step 4: Run tests**

Run: `cargo test -p auto-lang ui_cache`
Expected: 5 tests pass

**Step 5: Commit**

```bash
git add crates/auto-lang/src/database/ui_cache.rs crates/auto-lang/src/database.rs
git commit -m "feat(database): add UICache for persistent incremental UI compilation"
```

---

## Task 4: Add Hash Utility Function

**Files:**
- Modify: `crates/auto-man/src/util.rs` (or create if needed)

**Step 1: Check if util.rs exists**

Run: `ls crates/auto-man/src/util.rs`
Expected: File exists

**Step 2: Add hash_file function**

In `crates/auto-man/src/util.rs`, add:

```rust
use std::fs;
use std::path::Path;

/// Compute BLAKE3 hash of a file's contents
/// Returns the first 64 bits as u64 for compact storage
pub fn hash_file(path: &Path) -> std::io::Result<u64> {
    let content = fs::read(path)?;
    let hash = blake3::hash(&content);
    Ok(u64::from_be_bytes(hash.as_bytes()[0..8].try_into().unwrap()))
}

/// Compute BLAKE3 hash of a string
/// Returns the first 64 bits as u64 for compact storage
pub fn hash_string(content: &str) -> u64 {
    let hash = blake3::hash(content.as_bytes());
    u64::from_be_bytes(hash.as_bytes()[0..8].try_into().unwrap())
}
```

**Step 3: Verify blake3 dependency**

Run: `grep -n "blake3" crates/auto-man/Cargo.toml`
Expected: blake3 is listed. If not, add it:

```toml
[dependencies]
blake3 = "1.5"
```

**Step 4: Verify compilation**

Run: `cargo check -p auto-man`
Expected: No errors

**Step 5: Commit**

```bash
git add crates/auto-man/src/util.rs crates/auto-man/Cargo.toml
git commit -m "feat(auto-man): add hash_file and hash_string utilities"
```

---

## Task 5: Add Incremental Support to JetProject

**Files:**
- Modify: `crates/auto-man/src/jet.rs`

**Step 1: Add imports**

At the top of `crates/auto-man/src/jet.rs`, add:

```rust
use std::collections::HashSet;

use crate::util::hash_string;
use auto_lang::database::{UIArtifact, UIBackend, UICache};
```

**Step 2: Add incremental generation method to JetProject**

In `impl JetProject`, add new method:

```rust
impl JetProject {
    // ... existing methods ...

    /// Generate Kotlin files with incremental support
    /// Returns (kotlin_files, widget_names, changed_files)
    pub fn from_workspace_incremental(root_dir: &Path) -> AutoResult<(Self, Vec<String>)> {
        let pac_path = root_dir.join("pac.at");
        if !pac_path.exists() {
            return Err("pac.at not found in workspace".into());
        }

        let pac_content = fs::read_to_string(&pac_path)
            .map_err(|e| format!("Failed to read pac.at: {}", e))?;

        if !has_jet_backend(&pac_content) {
            return Err("Backend 'jet' not found in pac.at".into());
        }

        let name = parse_pac_name(&pac_content).unwrap_or_else(|| "MyApp".to_string());
        let front_dir = root_dir.join("source").join("front");
        let output_dir = root_dir.join("jet");

        // Load cache
        let mut cache = UICache::load(root_dir);
        let mut changed_files = Vec::new();

        // Process app.at
        let mut kotlin_files: Vec<(String, String)> = Vec::new();
        let mut widget_names: Vec<String> = Vec::new();

        let app_at = front_dir.join("app.at");
        if app_at.exists() {
            let content = fs::read_to_string(&app_at)
                .map_err(|e| format!("Failed to read app.at: {}", e))?;
            let hash = hash_string(&content);

            if cache.is_dirty(&app_at, hash) {
                println!("  {} (changed)", "app.at".bright_yellow());
                match Self::compile_at_file(&app_at, &name) {
                    Ok((files, names)) => {
                        let artifacts: Vec<UIArtifact> = files.iter().map(|(path, content)| {
                            UIArtifact {
                                source_path: PathBuf::from("source/front/app.at"),
                                widget_name: names.iter().next().cloned().unwrap_or_default(),
                                output_path: PathBuf::from(path),
                                source_hash: hash,
                                content_hash: hash_string(content),
                                backend: UIBackend::Jet,
                            }
                        }).collect();

                        cache.update(app_at.clone(), hash, artifacts);
                        kotlin_files.extend(files);
                        widget_names.extend(names);
                        changed_files.push("app.at".to_string());
                    }
                    Err(e) => {
                        println!("{} {}", "Warning: Failed to compile app.at:".bright_yellow(), e);
                    }
                }
            } else {
                println!("  {} (cached)", "app.at".bright_green());
            }
        }

        // Process widgets/ directory
        let widgets_dir = front_dir.join("widgets");
        if widgets_dir.exists() {
            if let Ok(entries) = fs::read_dir(&widgets_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                        if let Ok(content) = fs::read_to_string(&path) {
                            let hash = hash_string(&content);

                            if cache.is_dirty(&path, hash) {
                                println!("  {} (changed)", file_name.bright_yellow());
                                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("widget");

                                match Self::compile_at_file(&path, stem) {
                                    Ok((files, names)) => {
                                        let artifacts: Vec<UIArtifact> = files.iter().map(|(p, c)| {
                                            UIArtifact {
                                                source_path: path.clone(),
                                                widget_name: names.iter().next().cloned().unwrap_or_default(),
                                                output_path: PathBuf::from(p),
                                                source_hash: hash,
                                                content_hash: hash_string(c),
                                                backend: UIBackend::Jet,
                                            }
                                        }).collect();

                                        cache.update(path.clone(), hash, artifacts);
                                        kotlin_files.extend(files);
                                        widget_names.extend(names);
                                        changed_files.push(file_name);
                                    }
                                    Err(e) => {
                                        println!("{} Failed to compile {}: {}", "Warning:".bright_yellow(), file_name, e);
                                    }
                                }
                            } else {
                                println!("  {} (cached)", file_name.bright_green());
                            }
                        }
                    }
                }
            }
        }

        // Save cache
        cache.save(root_dir).ok();

        Ok((Self {
            root_dir: root_dir.to_path_buf(),
            output_dir,
            name,
            front_dir,
            kotlin_files,
            widget_names,
        }, changed_files))
    }
}
```

**Step 3: Verify compilation**

Run: `cargo check -p auto-man`
Expected: No errors

**Step 4: Commit**

```bash
git add crates/auto-man/src/jet.rs
git commit -m "feat(jet): add incremental compilation support with UICache"
```

---

## Task 6: Add Incremental Support to VueProject

**Files:**
- Modify: `crates/auto-man/src/vue.rs`

**Step 1: Add imports**

At the top of `crates/auto-man/src/vue.rs`, add:

```rust
use crate::util::hash_string;
use auto_lang::database::{UIArtifact, UIBackend, UICache};
```

**Step 2: Add incremental check to run_vue_project**

Find the `run_vue_project` function and modify it to use incremental compilation:

```rust
/// Run Vue dev server with incremental compilation
pub fn run_vue_project(root_dir: &Path, args: Vec<String>) -> AutoResult<()> {
    println!("{}", "Running Vue dev server (backend: vue)".bright_cyan());

    // Load cache
    let mut cache = UICache::load(root_dir);
    let front_dir = root_dir.join("source").join("front");
    let mut changed_count = 0;

    // Check for changes
    let app_at = front_dir.join("app.at");
    if app_at.exists() {
        if let Ok(content) = fs::read_to_string(&app_at) {
            let hash = hash_string(&content);
            if cache.is_dirty(&app_at, hash) {
                println!("  {} (changed)", "app.at".bright_yellow());
                // Generate Vue component
                if let Ok((vue_code, widgets)) = compile_at_to_vue(&app_at, &content) {
                    let artifacts: Vec<UIArtifact> = widgets.iter().map(|w| {
                        UIArtifact {
                            source_path: PathBuf::from("source/front/app.at"),
                            widget_name: w.clone(),
                            output_path: PathBuf::from(format!("src/components/{}.vue", w)),
                            source_hash: hash,
                            content_hash: hash_string(&vue_code),
                            backend: UIBackend::Vue,
                        }
                    }).collect();
                    cache.update(app_at.clone(), hash, artifacts);
                    changed_count += 1;
                }
            } else {
                println!("  {} (cached)", "app.at".bright_green());
            }
        }
    }

    // Save cache
    cache.save(root_dir).ok();

    if changed_count > 0 {
        println!("{} files changed, regenerated", changed_count.to_string().bright_yellow());
    } else {
        println!("{}", "No changes detected, using cached files".bright_green());
    }

    // ... rest of existing run_vue_project code ...
}
```

**Step 3: Add helper function**

```rust
/// Compile a .at file to Vue component
fn compile_at_to_vue(at_path: &Path, content: &str) -> Result<(String, Vec<String>), String> {
    use auto_lang::Parser;
    use auto_lang::session::CompilerSession;
    use auto_lang::ui_gen::{BackendGenerator, VueGenerator};
    use auto_lang::aura::extract_widget_from_decl;

    let session = CompilerSession::ui().with_backend("vue");
    let mut parser = Parser::from(content);
    parser = parser.with_session(session);

    let ast = parser.parse().map_err(|e| format!("Parse error: {:?}", e))?;

    let mut widgets = Vec::new();
    for stmt in &ast.stmts {
        if let auto_lang::ast::Stmt::WidgetDecl(widget_decl) = stmt {
            let aura_widget = extract_widget_from_decl(widget_decl)
                .map_err(|e| e.to_string())?;
            widgets.push(aura_widget);
        }
    }

    if widgets.is_empty() {
        return Err("No widgets found".to_string());
    }

    let mut generator = VueGenerator::new();
    let vue_code = generator.generate(&widgets[0])
        .map_err(|e| e.to_string())?;

    let names: Vec<String> = widgets.iter().map(|w| w.name.clone()).collect();
    Ok((vue_code, names))
}
```

**Step 4: Verify compilation**

Run: `cargo check -p auto-man`
Expected: No errors

**Step 5: Commit**

```bash
git add crates/auto-man/src/vue.rs
git commit -m "feat(vue): add incremental compilation support with UICache"
```

---

## Task 7: Integrate Incremental Compilation into auto run

**Files:**
- Modify: `crates/auto-man/src/automan.rs`

**Step 1: Update run_backend to show incremental stats**

In `run_backend` method, add cache stats output:

```rust
fn run_backend(&mut self, backend: &auto_lang::config::BackendType, args: Vec<String>) -> AutoResult<()> {
    use auto_lang::database::UICache;

    let root_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Show cache status
    let cache = UICache::load(&root_dir);
    println!("{}", "─────────────────────────────────".bright_cyan());
    println!("{} {} files, {} artifacts cached",
        "Cache:".bright_cyan(),
        cache.file_count(),
        cache.artifact_count()
    );
    println!("{}", "─────────────────────────────────".bright_cyan());

    match backend {
        auto_lang::config::BackendType::Vue => {
            println!("Running Vue dev server (backend: vue)");
            self.run_vue(args)
        }
        auto_lang::config::BackendType::Tauri => {
            println!("Running Tauri dev server (backend: tauri)");
            self.run_tauri(args)
        }
        auto_lang::config::BackendType::Jet => {
            println!("Running Jetpack project (backend: jet)");
            let root_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            crate::jet::run_jet_project(&root_dir, args)
        }
        _ => Err("Unknown backend type".into()),
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p auto-man`
Expected: No errors

**Step 3: Commit**

```bash
git add crates/auto-man/src/automan.rs
git commit -m "feat(automan): show cache stats in auto run"
```

---

## Task 8: Build and Test End-to-End

**Files:**
- Test: `examples/unified-example/`

**Step 1: Build the project**

Run: `cargo build --release`
Expected: Build succeeds

**Step 2: Test with unified-example**

```bash
cd examples/unified-example
# First run - should generate all files
../../target/release/auto.exe run
# Select vue

# Check cache was created
ls -la .auto/ui-cache.json

# Second run - should use cache
../../target/release/auto.exe run
# Should show "(cached)" for files
```

**Step 3: Test cache invalidation**

```bash
# Modify app.at (add a comment)
echo "// test comment" >> source/front/app.at

# Run again - should detect change
../../target/release/auto.exe run
# Should show "app.at (changed)"
```

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: complete Plan 134 - UI incremental compilation"
```

---

## Success Criteria

1. ✅ `auto run` creates `.auto/ui-cache.json` on first run
2. ✅ Second `auto run` shows "(cached)" for unchanged files
3. ✅ Modifying `.at` files triggers regeneration
4. ✅ Cache persists across sessions
5. ✅ Works for both Vue and Jet backends

---

## Future Enhancements (Not in Scope)

- Widget-level dependency tracking
- Fine-grained fragment hashing
- Cache invalidation on pac.at changes
- Integration with file watcher for hot reload
