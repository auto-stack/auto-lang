//! `auto watch` — incremental .at → SFC regeneration with live HMR.
//!
//! Watches `src/front/**/*.at` for changes and regenerates only the affected
//! Vue SFC files. Vite's dev server picks up the changed .vue files via HMR,
//! delivering <1s feedback.
//!
//! Plan 362:
//!   Phase 1: MVP — notify watcher + incremental SFC generation.
//!   Phase 2: caching — content-hash skip + GENERATOR_VERSION.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use notify::{Event, EventKind, RecursiveMode, Watcher};

/// Bump when generated SFC format has a breaking change.
/// Embedded in generated .vue comments so `auto watch` can detect
/// generator-code changes and trigger full rebuild.
const GENERATOR_VERSION: &str = "1.0";

/// Cache of .at file content hashes, persisted to `.auto/build/cache.json`.
/// Skips regeneration when content hasn't changed (e.g., file was just saved
/// without edits, or formatting-only change).
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct WatchCache {
    /// Map: absolute .at path → hex content hash
    entries: HashMap<String, String>,
    /// Generator version when cache was written.
    /// If current GENERATOR_VERSION differs, all entries are invalidated.
    generator_version: String,
}

/// Run the `auto watch` command.
///
/// `project_dir` is the workspace root (where pac.at lives).
pub fn run_watch(project_dir: &Path) -> Result<(), String> {
    let front_dir = find_front_dir(project_dir)?;
    let output_dir = project_dir.join("gen").join("front").join("vue").join("src").join("components");
    let cache_path = project_dir.join(".auto").join("build").join("cache.json");

    // Load or init cache
    let mut cache = load_cache(&cache_path);

    // If generator version changed, invalidate all entries
    if cache.generator_version != GENERATOR_VERSION {
        println!(
            "{} Generator version changed ({} → {}), full rebuild needed",
            "⟳".bright_yellow(),
            cache.generator_version,
            GENERATOR_VERSION
        );
        cache.entries.clear();
        cache.generator_version = GENERATOR_VERSION.to_string();
        save_cache(&cache_path, &cache)?;
    }

    // Collect initial set of watched .at files
    let at_files = collect_at_files(&front_dir)?;
    if at_files.is_empty() {
        return Err(format!(
            "No .at files found in {}. Is this an AutoUI project?",
            front_dir.display()
        ));
    }

    println!(
        "{} Watching {} .at files in {}",
        "▸".bright_cyan(),
        at_files.len(),
        front_dir.display()
    );
    println!(
        "{} Output → {}",
        "▸".bright_cyan(),
        output_dir.display()
    );
    println!("{} Vite HMR will reload on .vue changes\n", "▸".bright_cyan());

    // Channel for file events
    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })
    .map_err(|e| format!("Failed to create file watcher: {}", e))?;

    // Watch the front directory recursively
    watcher
        .watch(&front_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch {}: {}", front_dir.display(), e))?;

    // Debounce: collect events over 100ms, then process the batch
    let debounce = Duration::from_millis(100);
    let mut pending: HashMap<PathBuf, EventKind> = HashMap::new();

    println!("{} Ready. Edit an .at file to trigger rebuild.", "✓".bright_green());

    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(event) => {
                // Collect changed .at paths
                for path in &event.paths {
                    if path.extension().map(|e| e == "at").unwrap_or(false) {
                        pending.insert(path.clone(), event.kind.clone());
                    }
                }

                // If we have pending changes, wait the debounce period for more
                if !pending.is_empty() {
                    // Drain any additional events arriving during debounce
                    let deadline = std::time::Instant::now() + debounce;
                    while std::time::Instant::now() < deadline {
                        match rx.recv_timeout(
                            deadline.saturating_duration_since(std::time::Instant::now()),
                        ) {
                            Ok(e) => {
                                for p in &e.paths {
                                    if p.extension().map(|e| e == "at").unwrap_or(false) {
                                        pending.insert(p.clone(), e.kind.clone());
                                    }
                                }
                            }
                            Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => {
                                break;
                            }
                        }
                    }

                    // Process the batch
                    for (path, _kind) in pending.drain() {
                        if let Err(e) = rebuild_single_file(&path, &output_dir, &mut cache) {
                            eprintln!(
                                "{} {}: {}",
                                "✗".bright_red(),
                                path.display(),
                                e
                            );
                        } else {
                            // Save cache after each successful rebuild
                            let _ = save_cache(&cache_path, &cache);
                        }
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // No events — just loop (allows periodic checks)
            }
            Err(RecvTimeoutError::Disconnected) => {
                println!("\n{} Watcher disconnected. Exiting.", "✗".bright_yellow());
                break;
            }
        }
    }

    Ok(())
}

/// Find the front-end directory from the project root.
fn find_front_dir(root: &Path) -> Result<PathBuf, String> {
    // Try standard locations
    let candidates = [
        root.join("src").join("front"),
        root.join("source").join("front"),
        root.join("front"),
    ];
    for c in &candidates {
        if c.exists() && c.is_dir() {
            return Ok(c.clone());
        }
    }
    // Also check pac.at for workspace path config (Plan 129)
    let pac_at = root.join("pac.at");
    if pac_at.exists() {
        if let Ok(content) = std::fs::read_to_string(&pac_at) {
            if let Some(rel) = parse_workspace_path(&content, "front") {
                let resolved = root.join(&rel);
                if resolved.exists() {
                    return Ok(resolved);
                }
            }
        }
    }
    Err(format!("Front directory not found in {}", root.display()))
}

/// Collect all .at files in the front directory (recursively).
fn collect_at_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file()
            && entry.path().extension().map(|e| e == "at").unwrap_or(false)
        {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

/// Rebuild a single .at file: parse → generate → write .vue SFC.
///
/// Returns Ok(true) if regeneration happened, Ok(false) if skipped (cache hit).
fn rebuild_single_file(at_path: &Path, output_dir: &Path, cache: &mut WatchCache) -> Result<bool, String> {
    let path_key = at_path.to_string_lossy().to_string();

    // Compute content hash
    let content = std::fs::read_to_string(at_path)
        .map_err(|e| format!("Failed to read {}: {}", at_path.display(), e))?;
    let content_hash = hash_string(&content);

    // Check cache — skip if content hasn't changed
    if let Some(cached_hash) = cache.entries.get(&path_key) {
        if *cached_hash == content_hash {
            return Ok(false); // No change, skip
        }
    }

    let start = std::time::Instant::now();
    let rel_path = at_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.at".to_string());

    // Use the unified generate_component_from_file from Plan 361
    let opts = auto_lang::ui_gen::ComponentGenOptions::default();
    let result = auto_lang::ui_gen::generate_component_from_file(at_path, opts)
        .map_err(|e| format!("{}", e))?;

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output dir: {}", e))?;

    // Write each widget's SFC with generator version comment
    let version_comment = format!(
        "<!-- Auto-generated from {} (gen v{}, Plan 362) -->\n",
        rel_path, GENERATOR_VERSION
    );
    let mut written = 0usize;
    for (widget_name, code) in &result.all_widget_codes {
        let out_path = output_dir.join(format!("{}.vue", widget_name));
        let code_with_version = format!("{}{}", version_comment, code);
        std::fs::write(&out_path, &code_with_version)
            .map_err(|e| format!("Failed to write {}: {}", out_path.display(), e))?;
        written += 1;
    }

    // Write store composables
    for (filename, code) in &result.store_composables {
        let out_path = output_dir.join("..").join(filename);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let code_with_version = format!("{}{}", version_comment, code);
        std::fs::write(&out_path, &code_with_version).ok();
    }

    // Update cache
    cache.entries.insert(path_key, content_hash);

    // Print validation warnings if any
    let warn_count = result.validation_warnings.len();
    if warn_count > 0 {
        let warn_text = auto_lang::ui_gen::validators::format_warnings(&result.validation_warnings);
        eprintln!("{}", warn_text);
    }

    let elapsed = start.elapsed();
    println!(
        "{} {} → {} SFC{} ({:.0}ms){}",
        "✓".bright_green(),
        rel_path,
        written,
        if written != 1 { "s" } else { "" },
        elapsed.as_secs_f64() * 1000.0,
        if warn_count > 0 {
            format!(" — {} warning(s)", warn_count)
        } else {
            String::new()
        }
    );

    Ok(true)
}

/// Compute a stable hex hash of a string using std's DefaultHasher.
fn hash_string(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Load cache from disk, or return default empty cache.
fn load_cache(path: &Path) -> WatchCache {
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            if let Ok(cache) = serde_json::from_str::<WatchCache>(&content) {
                return cache;
            }
        }
    }
    WatchCache {
        generator_version: GENERATOR_VERSION.to_string(),
        ..Default::default()
    }
}

/// Save cache to disk.
fn save_cache(path: &Path, cache: &WatchCache) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("Failed to serialize cache: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Failed to write cache: {}", e))?;
    Ok(())
}

/// Very simple parser for pac.at `app("front")` path declarations.
fn parse_workspace_path(pac_content: &str, app_name: &str) -> Option<String> {
    // Look for `front("src/front")` or `app("front"): "src/front"`
    for line in pac_content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(app_name) && trimmed.contains('"') {
            // Extract the quoted path
            if let Some(start) = trimmed.find('"') {
                let after = &trimmed[start + 1..];
                if let Some(end) = after.find('"') {
                    let path = &after[..end];
                    if !path.is_empty() {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }
    None
}

use colored::Colorize;
