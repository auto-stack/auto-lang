//! Package manager abstraction: npm, pnpm, or bun with config-based selection.
//!
//! Resolution order:
//! 1. Per-project: `pkg: "pnpm"` in `pac.at`
//! 2. Global: `pkg: "pnpm"` in `~/.auto/auto-man/am.at`
//! 3. Auto-detect: prefers pnpm > bun > npm (cached process-wide)
//!
//! pnpm is preferred because its content-addressable store uses hardlinks,
//! avoiding duplicate dependency copies across multiple Vue projects.

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

/// Process-wide cached package manager (LEGACY path only — see
/// [`set_tool_pm_for_request`]; the tool path resolves per-request and
/// must not cache across operations).
static CACHED_PM: OnceLock<PkgManagerKind> = OnceLock::new();

/// PLAN-815 R5（F-815-T05-R4-04）：工具协议路径的包管理器单源状态。
/// 外层 `None` = legacy 形态（未设置——CLI 直跑，全链含磁盘 pac.at 自扫，
/// 语义冻结）；`Some(inner)` = 工具工作操作在飞：`inner=Some(kind)` =
/// 当前请求声明了 `pkg:`（raw 保真字段，生产执行必须消费它）；
/// `inner=None` = 请求无该字段——**跳过磁盘主 pac.at**（无第二真源），
/// 只走外部全局配置/自动探测。每操作设置、guard 清理，不跨操作缓存。
static TOOL_PM_STATE: std::sync::Mutex<Option<Option<PkgManagerKind>>> =
    std::sync::Mutex::new(None);

/// Set the tool-path package-manager state from the CURRENT request and
/// return a guard that restores the legacy state on drop (covers early
/// error returns). `pkg` is taken from the request's verbatim `raw` props.
pub fn set_tool_pm_for_request(
    request: &auto_project_model::generation_request::GenerationRequest,
) -> ToolPmGuard {
    let declared = request.raw.iter().find_map(|prop| {
        if prop.key != "pkg" {
            return None;
        }
        prop.value
            .as_ref()
            .map(|v| v.to_astr().trim().to_string())
            .and_then(|s| PkgManagerKind::from_str(&s))
    });
    *TOOL_PM_STATE.lock().unwrap() = Some(declared);
    ToolPmGuard
}

/// Restores the legacy resolution state when the work operation ends.
pub struct ToolPmGuard;

impl Drop for ToolPmGuard {
    fn drop(&mut self) {
        *TOOL_PM_STATE.lock().unwrap() = None;
    }
}

fn auto_detect() -> PkgManagerKind {
    if command_exists("pnpm") {
        PkgManagerKind::Pnpm
    } else if command_exists("bun") {
        PkgManagerKind::Bun
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

/// Resolve the package manager. Tool path (per-request, never cached
/// across operations): request `pkg:` > global config > auto-detect —
/// the disk main pac.at is NEVER consulted (no second truth source,
/// PLAN-815 R5 / F-815-T05-R4-04). Legacy path (config-based selection,
/// cached process-wide): project pac.at > global config > auto-detect.
pub fn resolve() -> PkgManagerKind {
    // Tool path: honor the CURRENT request exactly (serial requests with
    // different `pkg:` declarations each get their own manager).
    if let Some(state) = TOOL_PM_STATE.lock().unwrap().clone() {
        return match state {
            Some(kind) => kind,
            None => resolve_global_then_detect(),
        };
    }
    *CACHED_PM.get_or_init(resolve_impl)
}

/// External-only chain (no disk main declaration): global config, then
/// auto-detect — the tool path's fallback when the request declares no
/// `pkg:` field.
fn resolve_global_then_detect() -> PkgManagerKind {
    if let Some(home) = dirs::home_dir() {
        let global_path = home.join(".auto").join("auto-man").join("am.at");
        if let Ok(content) = std::fs::read_to_string(&global_path) {
            if let Some(pm) = parse_pkg_from_at(&content) {
                return pm;
            }
        }
    }
    auto_detect()
}

fn resolve_impl() -> PkgManagerKind {
    // 1. Try project-level config: pac.at `pkg: "bun"`
    if let Ok(content) = std::fs::read_to_string("pac.at") {
        if let Some(pm) = parse_pkg_from_at(&content) {
            return pm;
        }
    }

    // 2. Try global config: ~/.auto/auto-man/am.at `pkg: "bun"`
    resolve_global_then_detect()
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
///
/// Always sets `CI=true` so child processes (notably pnpm v11's build-approval
/// prompt triggered inside `shadcn-vue add`) run non-interactively instead of
/// hanging on a TTY prompt that has no one to answer it.
///
/// PLAN-815 T-05 hosted mode: when a tool-host [`crate::proc_host::RunIo`] is
/// installed, the child is spawned tracked (piped stdout forwarded to the
/// host log stream, piped stdin, cancel-aware wait) instead of inheriting.
pub fn run_command_live(cmd: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let mut full_args = vec!["/C", cmd];
    #[cfg(windows)]
    full_args.extend(args);
    #[cfg(windows)]
    let mut command = Command::new("cmd");
    #[cfg(windows)]
    command.args(&full_args);
    #[cfg(not(windows))]
    let mut command = Command::new(cmd);
    #[cfg(not(windows))]
    command.args(args);
    command.current_dir(cwd).env("CI", "true");
    if crate::proc_host::current().is_some() {
        let mut spec =
            crate::proc_host::TrackedSpawn::new(cmd, args.iter().map(|a| a.to_string()).collect())
                .with_cwd(cwd)
                .with_env("CI", "true");
        #[cfg(windows)]
        {
            spec.windows_cmd_shim = true;
        }
        let tracked = crate::proc_host::spawn_tracked(spec, "install")
            .map_err(|e| format!("Failed to run {cmd}: {e}"))?;
        let status =
            crate::proc_host::wait_tracked(&tracked).map_err(|e| format!("Failed to wait: {e}"))?;
        return if status.success() {
            Ok(())
        } else {
            Err(format!("{cmd} exited with code {:?}", status.code()))
        };
    }
    let status = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to run {cmd}: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        // Plan 328: pnpm v9+ exits non-zero for "ignored builds" warnings
        // (ERR_PNPM_IGNORED_BUILDS), even though the install succeeded.
        // Check if node_modules exists (install actually completed).
        let node_modules = cwd.join("node_modules");
        if cmd == "pnpm" && node_modules.exists() {
            // Install succeeded despite non-zero exit code.
            return Ok(());
        }
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
    // Plan 346: dev server (vite) needs interactive mode — do NOT set CI=true
    // (CI=true disables Vite's TTY watcher, so `q` to quit stops working).
    // Also inherit stdin so the user can type `q` + Enter to stop Vite.
    run_script_live(install_cmd(), &args, cwd)
}

/// Run a package script (e.g., `pnpm run dev`) with full interactivity.
/// Unlike `run_command_live`, this does NOT set CI=true and DOES inherit
/// stdin, so interactive tools like Vite's dev server work properly
/// (press `q` + Enter to quit).
/// Plan 348: Wrapper that kills the child process when dropped, ensuring
/// Vite and other dev servers are cleaned up even if auto exits unexpectedly.
struct ChildGuard(Option<std::process::Child>);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn run_script_live(cmd: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let mut full_args = vec!["/C", cmd];
    #[cfg(windows)]
    full_args.extend(args);
    #[cfg(windows)]
    let mut command = Command::new("cmd");
    #[cfg(windows)]
    command.args(&full_args);
    #[cfg(not(windows))]
    let mut command = Command::new(cmd);
    #[cfg(not(windows))]
    command.args(args);
    command.current_dir(cwd);

    // PLAN-815 T-05 hosted mode: the dev server runs as a tracked child —
    // its stdout is forwarded to the host log stream (the sniffed URL
    // announces the launch spec), its stdin is fed by the protocol `Input`
    // channel (Vite really does quit on `q` + Enter), and cancel kills it.
    if crate::proc_host::current().is_some() {
        let mut spec =
            crate::proc_host::TrackedSpawn::new(cmd, args.iter().map(|a| a.to_string()).collect())
                .with_cwd(cwd);
        #[cfg(windows)]
        {
            spec.windows_cmd_shim = true;
        }
        let tracked = crate::proc_host::spawn_tracked(spec, "dev-server")
            .map_err(|e| format!("Failed to run {cmd}: {e}"))?;
        let status =
            crate::proc_host::wait_tracked(&tracked).map_err(|e| format!("Failed to wait: {e}"))?;
        return if status.success() {
            Ok(())
        } else {
            Err(format!("{cmd} exited with code {:?}", status.code()))
        };
    }

    // Legacy interactive path (behavior frozen): inherit stdio so the user
    // can type `q` + Enter to stop Vite.
    let child = command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;

    let mut guard = ChildGuard(Some(child));
    let status = guard
        .0
        .as_mut()
        .unwrap()
        .wait()
        .map_err(|e| format!("Failed to wait: {}", e))?;
    guard.0 = None; // Child exited normally, don't kill on drop

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with code {:?}", cmd, status.code()))
    }
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
                // Plan 465 T7: pnpm v11 rejects `add --dev` (bogus "Unknown
                // option" + non-zero exit, previously swallowed by the
                // node_modules heuristic below). `-D` works on pnpm and bun.
                a.push("-D");
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

    /// These tests mutate process-global state (cwd, TOOL_PM_STATE) —
    /// serialize them within this binary (single-process libtest runner).
    static PM_STATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// A request carrying `pkg: "x"` in its verbatim raw props.
    fn request_with_pkg(pkg: Option<&str>) -> auto_project_model::GenerationRequest {
        let pac = match pkg {
            Some(value) => format!(
                "name: \"pmx\"
render: \"vue\"
pkg: \"{value}\"
"
            ),
            None => "name: \"pmx\"
render: \"vue\"
"
            .to_string(),
        };
        let descriptor = auto_project_model::parse_document(
            &pac,
            auto_project_model::descriptor::SourceInfo::from_path("P:/pm-inline/pac.at"),
        )
        .expect("parse");
        auto_project_model::GenerationRequest::from_descriptor(&descriptor)
    }

    /// PLAN-815 R5（F-815-T05-R4-04）：工具路径的包管理器单源矩阵。
    /// - legacy（无工具状态）：磁盘 pac.at 仍是源（语义冻结）；
    /// - 工具+请求 pkg 字段：请求值赢（磁盘冲突负控）；
    /// - 工具+请求无字段：跳过磁盘（外部链兜底）；
    /// - guard 丢弃恢复 legacy（串行下一请求不残留）。
    #[test]
    fn tool_pm_state_matrix() {
        let _guard = PM_STATE_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("pac.at"),
            "name: \"diskx\"
pkg: \"bun\"
",
        )
        .unwrap();
        let original = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(dir.path()).expect("chdir");

        // legacy: disk wins (frozen behavior).
        assert_eq!(resolve(), PkgManagerKind::Bun, "legacy reads disk pac");

        // tool + request declares pnpm: request wins over disk bun.
        {
            let _tool = set_tool_pm_for_request(&request_with_pkg(Some("pnpm")));
            assert_eq!(
                resolve(),
                PkgManagerKind::Pnpm,
                "request pkg must beat the disk main declaration"
            );
        }

        // tool + request WITHOUT the field: the disk main pac.at is NOT a
        // source — resolution must not come from disk (bun); it falls to
        // the external chain (global config > auto-detect).
        {
            let _tool = set_tool_pm_for_request(&request_with_pkg(None));
            let resolved = resolve();
            assert_ne!(
                resolved,
                PkgManagerKind::Bun,
                "tool path must skip the disk main declaration"
            );
            if !command_exists("pnpm") {
                // On a machine without pnpm the external chain may land on
                // bun via auto-detect — the disk-independence claim is then
                // proven by the pnpm-present branch above/below.
                eprintln!("pnpm absent: auto-detect = {resolved:?}");
            }
        }

        // guard dropped: legacy state restored (serial next request sees
        // the frozen chain again).
        assert_eq!(
            resolve(),
            PkgManagerKind::Bun,
            "guard drop must restore the legacy resolution state"
        );

        std::env::set_current_dir(original).expect("restore cwd");
    }

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
        assert_eq!(parse_pkg_from_at("name: \"foo\""), None);
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
