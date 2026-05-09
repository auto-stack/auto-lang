//! Package manager abstraction: npm, pnpm, or bun with config-based selection.
//!
//! Resolution order:
//! 1. Per-project: `pkg: "bun"` in `pac.at`
//! 2. Global: `pkg: "pnpm"` in `~/.auto/auto-man/am.at`
//! 3. Auto-detect: prefers bun > pnpm > npm (cached process-wide)

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// Supported package managers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkgManagerKind {
    Npm,
    Pnpm,
    Bun,
}

impl PkgManagerKind {
    /// Parse from a config string (e.g. `"bun"`, `"npm"`, `"pnpm"`).
    /// Returns None for unrecognized values.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "npm" => Some(PkgManagerKind::Npm),
            "pnpm" => Some(PkgManagerKind::Pnpm),
            "bun" => Some(PkgManagerKind::Bun),
            _ => None,
        }
    }
}

/// Process-wide cached package manager.
static CACHED_PM: OnceLock<PkgManagerKind> = OnceLock::new();

fn auto_detect() -> PkgManagerKind {
    if command_exists("bun") {
        PkgManagerKind::Bun
    } else if command_exists("pnpm") {
        PkgManagerKind::Pnpm
    } else {
        PkgManagerKind::Npm
    }
}

/// Check if a command exists on the system PATH.
pub fn command_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    let check = Command::new("where").arg(cmd).output();
    #[cfg(not(windows))]
    let check = Command::new("which").arg(cmd).output();
    check.map(|o| o.status.success()).unwrap_or(false)
}

/// Resolve the package manager. Checks project config, then global config,
/// then auto-detects. Result is cached process-wide.
pub fn resolve() -> PkgManagerKind {
    *CACHED_PM.get_or_init(resolve_impl)
}

fn resolve_impl() -> PkgManagerKind {
    // 1. Try project-level config: pac.at `pkg: "bun"`
    if let Ok(content) = std::fs::read_to_string("pac.at") {
        if let Some(pm) = parse_pkg_from_at(&content) {
            return pm;
        }
    }

    // 2. Try global config: ~/.auto/auto-man/am.at `pkg: "bun"`
    if let Some(home) = dirs::home_dir() {
        let global_path = home.join(".auto").join("auto-man").join("am.at");
        if let Ok(content) = std::fs::read_to_string(&global_path) {
            if let Some(pm) = parse_pkg_from_at(&content) {
                return pm;
            }
        }
    }

    // 3. Auto-detect
    auto_detect()
}

/// Try to extract `pkg: "bun"` from .at file content.
fn parse_pkg_from_at(content: &str) -> Option<PkgManagerKind> {
    for line in content.lines() {
        let trimmed = line.trim();
        // Match patterns like: pkg: "bun" or pkg: "pnpm"
        if let Some(rest) = trimmed.strip_prefix("pkg:") {
            let rest = rest.trim();
            if let Some(value) = rest.strip_prefix('"') {
                if let Some(value) = value.strip_suffix('"') {
                    if let Some(pm) = PkgManagerKind::from_str(value) {
                        return Some(pm);
                    }
                }
            }
        }
    }
    None
}

/// Run a command with live output (inherits stdout/stderr).
/// On Windows, uses `cmd /C` to properly resolve commands from PATH.
pub fn run_command_live(cmd: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let status = {
        let mut full_args = vec!["/C", cmd];
        full_args.extend(args);
        Command::new("cmd")
            .args(&full_args)
            .current_dir(cwd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to run {}: {}", cmd, e))?
    };

    #[cfg(not(windows))]
    let status = {
        Command::new(cmd)
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to run {}: {}", cmd, e))?
    };

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with code {:?}", cmd, status.code()))
    }
}

/// The install command name: `"bun"`, `"pnpm"`, or `"npm"`.
pub fn install_cmd() -> &'static str {
    match resolve() {
        PkgManagerKind::Bun => "bun",
        PkgManagerKind::Pnpm => "pnpm",
        PkgManagerKind::Npm => "npm",
    }
}

/// The one-off exec command name: `"pnpm"` (dlx) or `"npx"`.
///
/// Prefers `pnpm dlx` over `bunx` because bunx re-resolves `@latest` from the
/// registry on every invocation, while pnpm's content-addressable cache skips
/// re-downloads for already-resolved versions.
pub fn exec_cmd() -> &'static str {
    if command_exists("pnpm") {
        "pnpm"
    } else {
        "npx"
    }
}

/// Display label for log messages.
pub fn display_name() -> &'static str {
    install_cmd()
}

/// Run `bun install` / `pnpm install` / `npm install` in the given directory.
pub fn install(cwd: &Path) -> Result<(), String> {
    run_command_live(install_cmd(), &["install"], cwd)
}

/// Run `bun run <script>` / `pnpm run <script>` / `npm run <script>`.
pub fn run_script(script: &str, extra_args: &[&str], cwd: &Path) -> Result<(), String> {
    let mut args = vec!["run", script];
    args.extend(extra_args);
    run_command_live(install_cmd(), &args, cwd)
}

/// Run a one-off package via `pnpm dlx` or `npx --yes`.
///
/// Always uses pnpm or npx for exec (not bunx) because bunx re-downloads
/// `@latest` packages on every invocation instead of using the cache.
pub fn exec(package: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    let cmd = exec_cmd();
    let mut full_args: Vec<&str> = Vec::new();
    if cmd == "pnpm" {
        full_args.push("dlx");
    } else {
        full_args.push("--yes");
    }
    full_args.push(package);
    full_args.extend(args);
    run_command_live(cmd, &full_args, cwd)
}

/// Run a locally-installed package via `pnpm exec` or `npx`.
///
/// Use this when the package is already installed in node_modules (e.g.
/// `@tauri-apps/cli` added as a dev dependency).
pub fn exec_local(package: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    let cmd = exec_cmd();
    let mut full_args: Vec<&str> = Vec::new();
    if cmd == "pnpm" {
        full_args.push("exec");
    }
    full_args.push(package);
    full_args.extend(args);
    run_command_live(cmd, &full_args, cwd)
}

/// Install specific packages: `bun add [--dev]` / `pnpm add [--save-dev]` / `npm install [--save-dev]`.
pub fn add_packages(packages: &[&str], dev: bool, cwd: &Path) -> Result<(), String> {
    let cmd = install_cmd();
    let mut args: Vec<&str> = match resolve() {
        PkgManagerKind::Bun | PkgManagerKind::Pnpm => {
            let mut a = vec!["add"];
            if dev {
                a.push("--dev");
            }
            a
        }
        PkgManagerKind::Npm => {
            let mut a = vec!["install"];
            if dev {
                a.push("--save-dev");
            }
            a
        }
    };
    args.extend(packages);
    run_command_live(cmd, &args, cwd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(PkgManagerKind::from_str("bun"), Some(PkgManagerKind::Bun));
        assert_eq!(PkgManagerKind::from_str("pnpm"), Some(PkgManagerKind::Pnpm));
        assert_eq!(PkgManagerKind::from_str("npm"), Some(PkgManagerKind::Npm));
        assert_eq!(PkgManagerKind::from_str("BUN"), Some(PkgManagerKind::Bun));
        assert_eq!(PkgManagerKind::from_str("yarn"), None);
    }

    #[test]
    fn test_parse_pkg_from_at() {
        assert_eq!(
            parse_pkg_from_at("name: \"foo\"\npkg: \"bun\"\nversion: \"1.0\""),
            Some(PkgManagerKind::Bun)
        );
        assert_eq!(
            parse_pkg_from_at("pkg:\"pnpm\""),
            Some(PkgManagerKind::Pnpm)
        );
        assert_eq!(
            parse_pkg_from_at("name: \"foo\""),
            None
        );
    }

    #[test]
    fn test_resolve_is_cached() {
        let a = resolve();
        let b = resolve();
        assert_eq!(a, b);
    }

    #[test]
    fn test_command_names_are_consistent() {
        let pm = resolve();
        match pm {
            PkgManagerKind::Bun => {
                assert_eq!(install_cmd(), "bun");
                // exec uses pnpm/npx, not bunx (bunx re-downloads @latest)
            }
            PkgManagerKind::Pnpm => {
                assert_eq!(install_cmd(), "pnpm");
            }
            PkgManagerKind::Npm => {
                assert_eq!(install_cmd(), "npm");
            }
        }
        // exec_cmd is always pnpm or npx, never bunx
        assert!(exec_cmd() == "pnpm" || exec_cmd() == "npx");
    }

    #[test]
    fn test_command_exists_with_known_command() {
        #[cfg(windows)]
        assert!(command_exists("cmd"));
        #[cfg(not(windows))]
        assert!(command_exists("ls"));
    }

    #[test]
    fn test_command_exists_with_gibberish() {
        assert!(!command_exists("definitely_not_a_real_command_xyz123"));
    }
}
