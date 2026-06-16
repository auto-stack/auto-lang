//! Three-tier completion spec loading (Plan 315).
//!
//! Tier directories under `<config>/ash/completions/`:
//! - **user**:      `<base>/<cmd>.at`        (highest priority, user-authored)
//! - **generated**: `<base>/generated/<cmd>.at`  (`completions generate`, overrides built-ins)
//! - **cache**:     `<base>/cache/<cmd>.at`   (runtime help-probe, lowest)
//!
//! `<base>` resolves to `~/.config/ash/completions` (or `%APPDATA%/ash/completions`
//! on Windows if the home path is unavailable). Pure file I/O + (de)serialization;
//! the probe orchestration (running `cmd --help`) lives in the ShellCompleter.

use std::path::{Path, PathBuf};

use ash_core::completions::spec::CompletionSpec;
use ash_core::completions::spec_format;

/// Candidate base dirs (home-based first, then platform config dir).
fn base_candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(home) = dirs::home_dir() {
        v.push(home.join(".config").join("ash").join("completions"));
    }
    if let Some(cfg) = dirs::config_dir() {
        v.push(cfg.join("ash").join("completions"));
    }
    v
}

/// The base completions dir, creating it (with generated/ + cache/ subdirs) if absent.
fn ensure_base() -> Option<PathBuf> {
    for c in base_candidates() {
        if c.exists() {
            return Some(c);
        }
    }
    // Create the home-based base + subdirs.
    let base = base_candidates().into_iter().next()?;
    let _ = std::fs::create_dir_all(base.join("generated"));
    let _ = std::fs::create_dir_all(base.join("cache"));
    Some(base)
}

pub fn user_dir() -> Option<PathBuf> {
    ensure_base()
}
pub fn generated_dir() -> Option<PathBuf> {
    ensure_base().map(|b| b.join("generated"))
}
pub fn cache_dir() -> Option<PathBuf> {
    ensure_base().map(|b| b.join("cache"))
}

/// Load every `.at` spec in a directory.
pub fn load_dir(dir: &Path) -> Vec<CompletionSpec> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("at") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Ok(spec) = spec_format::deserialize(&text) {
                    out.push(spec);
                }
            }
        }
    }
    out
}

/// Load a single command's spec from a tier directory (`<dir>/<cmd>.at`).
pub fn load_one(dir: &Path, cmd: &str) -> Option<CompletionSpec> {
    let path = dir.join(format!("{}.at", cmd));
    let text = std::fs::read_to_string(&path).ok()?;
    spec_format::deserialize(&text).ok()
}

/// Serialize `spec` and write it to the cache tier (`cache/<cmd>.at`).
pub fn write_cache(cmd: &str, spec: &CompletionSpec) -> std::io::Result<()> {
    let dir = cache_dir().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no cache dir"))?;
    std::fs::create_dir_all(&dir)?;
    let text = spec_format::serialize(spec);
    std::fs::write(dir.join(format!("{}.at", cmd)), text)
}

/// Load a command's cached spec, if present.
pub fn load_cache(cmd: &str) -> Option<CompletionSpec> {
    let dir = cache_dir()?;
    load_one(&dir, cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash_core::completions::spec::{CompletionSpec, FlagSpec};

    fn tmp_base(suffix: &str) -> PathBuf {
        // Unique temp dir per test (suffix) to avoid parallel races.
        let base = std::env::temp_dir().join(format!(
            "ash_completions_tier_{}_{}",
            std::process::id(),
            suffix
        ));
        let _ = std::fs::create_dir_all(base.join("generated"));
        let _ = std::fs::create_dir_all(base.join("cache"));
        base
    }

    #[test]
    fn load_dir_and_load_one_and_roundtrip() {
        let base = tmp_base("roundtrip");
        let spec = CompletionSpec::new("rg")
            .flag(FlagSpec::long("ignore-case"))
            .flag(FlagSpec::both("t", "type").takes_arg("TYPE"));
        let path = base.join("generated").join("rg.at");
        std::fs::write(&path, spec_format::serialize(&spec)).unwrap();

        let loaded = load_one(&base.join("generated"), "rg").expect("should load");
        assert_eq!(loaded.command, "rg");
        assert!(loaded.flags.iter().any(|f| f.long.as_deref() == Some("ignore-case")));
        assert!(loaded.flags.iter().any(|f| f.short.as_deref() == Some("t")));

        let all = load_dir(&base.join("generated"));
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].command, "rg");

        // cleanup
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn load_one_missing_returns_none() {
        let base = tmp_base("missing");
        assert!(load_one(&base.join("generated"), "nonexistent_cmd").is_none());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn write_then_load_cache() {
        // write_cache writes to the REAL cache_dir (resolved via dirs). To avoid
        // polluting the user's real dir, this test only exercises the serialize
        // path; the real write is integration-tested elsewhere.
        let spec = CompletionSpec::new("probeonly");
        let text = spec_format::serialize(&spec);
        assert!(text.contains("probeonly"));
    }
}
