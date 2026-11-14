//! Host metadata directory for the M0 experiment (PLAN-1021 §5): a TOML
//! manifest describing the two registered app identities, their local
//! version, ACE package name, source root and declared capabilities.
//! This manifest is an experiment description, NOT a public authoring
//! syntax. Structural validity is checked here; identity/version/
//! capability admission and root containment are enforced at resolve and
//! read time (`uri.rs`, `source.rs`, `run.rs`).

use crate::error::{AseError, Kind, Phase};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct RegistryFile {
    app: Vec<AppFile>,
}

#[derive(Deserialize)]
struct AppFile {
    identity: String,
    version: String,
    package: String,
    root: String,
    entry: String,
    sources: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
}

/// One registered app identity in the host metadata.
#[derive(Clone, Debug)]
pub struct RegistryApp {
    pub identity: String,
    pub version: String,
    pub package: String,
    /// Source root, relative to the registry directory.
    pub root: PathBuf,
    /// Entry module id (an identifier; file convention `<id>.at`).
    pub entry: String,
    /// Registered source file names (single-segment, `<module>.at`).
    pub sources: Vec<String>,
    /// Declared host capabilities. The M0 domain supports none; a non-empty
    /// list is refused at admission time (Kind::Authz), not silently run.
    pub capabilities: Vec<String>,
}

/// The loaded host metadata directory.
#[derive(Clone, Debug)]
pub struct Registry {
    pub dir: PathBuf,
    pub apps: Vec<RegistryApp>,
}

fn is_version_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '_' | '-')
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn registry_err(message: impl Into<String>) -> AseError {
    AseError::new(Phase::Resolve, Kind::Registry, message)
}

pub fn load_registry(dir: &Path) -> Result<Registry, AseError> {
    let manifest = dir.join("registry.toml");
    let text = std::fs::read_to_string(&manifest)
        .map_err(|e| registry_err(format!("read {}: {e}", manifest.display())))?;
    let file: RegistryFile = toml::from_str(&text)
        .map_err(|e| registry_err(format!("parse {}: {e}", manifest.display())))?;
    let mut apps = Vec::with_capacity(file.app.len());
    for app in file.app {
        if app.identity.is_empty()
            || !app
                .identity
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
        {
            return Err(registry_err(format!(
                "identity {:?} must be non-empty lowercase [a-z0-9.-]",
                app.identity
            )));
        }
        if app.version.is_empty() || !app.version.chars().all(is_version_char) {
            return Err(registry_err(format!(
                "version {:?} for {} must be non-empty [A-Za-z0-9.+_-]",
                app.version, app.identity
            )));
        }
        let mut chars = app.package.chars();
        let package_ok = matches!(chars.next(), Some(c) if c.is_ascii_lowercase())
            && app
                .package
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
        if !package_ok {
            return Err(registry_err(format!(
                "package {:?} for {} must match [a-z][a-z0-9_-]*",
                app.package, app.identity
            )));
        }
        if !is_ident(&app.entry) {
            return Err(registry_err(format!(
                "entry {:?} for {} must be an identifier",
                app.entry, app.identity
            )));
        }
        if app.sources.is_empty() {
            return Err(registry_err(format!(
                "sources for {} must list at least the entry file",
                app.identity
            )));
        }
        let entry_file = format!("{}.at", app.entry);
        if !app.sources.iter().any(|s| s == &entry_file) {
            return Err(registry_err(format!(
                "entry file {entry_file:?} for {} is not listed in sources",
                app.identity
            )));
        }
        if apps
            .iter()
            .any(|a: &RegistryApp| a.identity == app.identity)
        {
            return Err(registry_err(format!(
                "duplicate identity {:?}",
                app.identity
            )));
        }
        apps.push(RegistryApp {
            identity: app.identity,
            version: app.version,
            package: app.package,
            root: PathBuf::from(&app.root),
            entry: app.entry,
            sources: app.sources,
            capabilities: app.capabilities,
        });
    }
    Ok(Registry {
        dir: dir.to_path_buf(),
        apps,
    })
}

impl Registry {
    pub fn lookup(&self, identity: &str) -> Option<&RegistryApp> {
        self.apps.iter().find(|a| a.identity == identity)
    }
}
