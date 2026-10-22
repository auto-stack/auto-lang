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

/// Where a resolution came from (PLAN-815 R6 / F-815-T05-R6-01: the
/// single-source oracle observes the SOURCE, not the value — bun is a
/// legal external fallback, so "not bun" can never prove disk
/// independence).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PmSource {
    /// The current request's `pkg:` field (tool path only).
    Request,
    /// The external global config (`~/.auto/auto-man/am.at`).
    Global,
    /// PATH auto-detection (pnpm > bun > npm).
    AutoDetect,
    /// The LEGACY chain reading the disk main pac.at (CLI direct runs).
    DiskLegacy,
}

/// Resolve the package manager WITH its source. Tool path (per-request,
/// never cached across operations): request `pkg:` > global config >
/// auto-detect — the disk main pac.at is NEVER consulted (no second
/// truth source, PLAN-815 R5 / F-815-T05-R4-04). Legacy path
/// (config-based selection, cached process-wide): project pac.at >
/// global config > auto-detect.
pub fn resolve_with_source() -> (PkgManagerKind, PmSource) {
    // Tool path: honor the CURRENT request exactly (serial requests with
    // different `pkg:` declarations each get their own manager).
    if let Some(state) = TOOL_PM_STATE.lock().unwrap().clone() {
        return match state {
            Some(kind) => (kind, PmSource::Request),
            None => resolve_global_then_detect_sourced(),
        };
    }
    (*CACHED_PM.get_or_init(resolve_impl), PmSource::DiskLegacy)
}

/// External-only chain (no disk main declaration) with source
/// classification: global config, then auto-detect — the tool path's
/// fallback when the request declares no `pkg:` field.
fn resolve_global_then_detect_sourced() -> (PkgManagerKind, PmSource) {
    if let Some(home) = dirs::home_dir() {
        let global_path = home.join(".auto").join("auto-man").join("am.at");
        if let Ok(content) = std::fs::read_to_string(&global_path) {
            if let Some(pm) = parse_pkg_from_at(&content) {
                return (pm, PmSource::Global);
            }
        }
    }
    (auto_detect(), PmSource::AutoDetect)
}

pub fn resolve() -> PkgManagerKind {
    resolve_with_source().0
}

fn resolve_impl() -> PkgManagerKind {
    // 1. Try project-level config: pac.at `pkg: "bun"`
    if let Ok(content) = std::fs::read_to_string("pac.at") {
        if let Some(pm) = parse_pkg_from_at(&content) {
            return pm;
        }
    }

    // 2. Try global config: ~/.auto/auto-man/am.at `pkg: "bun"`
    resolve_global_then_detect_sourced().0
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

    /// RAII restore for the process-global state this matrix mutates
    /// (cwd + PATH) — an assert failure must not poison the rest of the
    /// single-process libtest run (PLAN-815 R6-01).
    struct EnvGuard {
        saved_cwd: std::path::PathBuf,
        saved_path: Option<std::ffi::OsString>,
    }
    impl EnvGuard {
        fn capture() -> Self {
            Self {
                saved_cwd: std::env::current_dir().expect("cwd"),
                saved_path: std::env::var_os("PATH"),
            }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // SAFETY: tests run under PM_STATE_LOCK (single writer).
            unsafe {
                std::env::set_current_dir(&self.saved_cwd).ok();
                match self.saved_path.take() {
                    Some(v) => std::env::set_var("PATH", v),
                    None => std::env::remove_var("PATH"),
                }
            }
        }
    }

    /// A temp PATH dir containing ONLY the given manager stubs (+ the
    /// system dir so `where`/`which` itself still resolves) — the
    /// deterministic auto-detect isolation for the direct `auto_detect`
    /// oracle (the fallback chain's own priority semantics).
    fn stub_path(managers: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("stub path dir");
        for name in managers {
            #[cfg(windows)]
            std::fs::write(dir.path().join(format!("{name}.cmd")), b"").unwrap();
            #[cfg(not(windows))]
            {
                use std::os::unix::fs::PermissionsExt;
                let stub = dir.path().join(name);
                std::fs::write(&stub, b"").unwrap();
                let mut perms = std::fs::metadata(&stub).unwrap().permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&stub, perms).unwrap();
            }
        }
        dir
    }

    /// SAFETY: single writer under PM_STATE_LOCK.
    unsafe fn set_stub_path(stub: &std::path::Path) {
        let system = if cfg!(windows) {
            std::env::var_os("SystemRoot")
                .map(|root| std::path::PathBuf::from(root).join("System32"))
                .unwrap_or_default()
        } else {
            std::path::PathBuf::from("/usr/bin")
        };
        let mut combined = std::ffi::OsString::from(stub);
        combined.push(if cfg!(windows) { ";" } else { ":" });
        combined.push(system.as_os_str());
        std::env::set_var("PATH", combined);
    }

    /// PLAN-815 R6（F-815-T05-R6-01）重写：工具路径包管理器单源矩阵。
    /// 判定观察**来源**（`resolve_with_source`），不是值——bun 是外部链
    /// 的合法回退，"不等于 bun"永远证明不了磁盘独立性：
    /// - legacy（无工具状态）：来源=磁盘 pac（语义冻结）；
    /// - 工具+请求 pkg 字段：来源=请求（磁盘冲突负控）；
    /// - 工具+无字段：相同外部环境下改变/移除磁盘主 pac，(值,来源) 不变
    ///   且来源 ∈ {Global, AutoDetect}（磁盘独立性正反控——外部值随
    ///   机器合法变化，包括 bun）；
    /// - auto_detect 直接 oracle（stub PATH）：bun-only→Bun（合法回退）、
    ///   bun+pnpm→Pnpm（优先级）；
    /// - guard 丢弃恢复 legacy（串行下一请求不残留）。
    #[test]
    fn tool_pm_state_matrix() {
        let _guard = PM_STATE_LOCK.lock().unwrap();
        let _env = EnvGuard::capture();

        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("pac.at"), "name: \"diskx\"\npkg: \"bun\"\n").unwrap();
        std::env::set_current_dir(dir.path()).expect("chdir");

        // Legacy (frozen behavior): the source is the disk chain. The
        // VALUE is process-cached (OnceLock) once ANY test resolved
        // legacy — in-process we assert source + cache stability; the
        // disk-READ proof runs in a FRESH child process at the end
        // (PLAN-815 T-05 R7 / F-815-T05-R7-01: the old value-pinned
        // assert was order-dependent under the module test run).
        let legacy_baseline = resolve_with_source();
        assert_eq!(legacy_baseline.1, PmSource::DiskLegacy);
        assert_eq!(
            resolve_with_source(),
            legacy_baseline,
            "legacy resolution is frozen per process (cached)"
        );

        // tool + request declares pnpm: the request is the source and its
        // value wins over the disk bun.
        {
            let _tool = set_tool_pm_for_request(&request_with_pkg(Some("pnpm")));
            assert_eq!(
                resolve_with_source(),
                (PkgManagerKind::Pnpm, PmSource::Request),
                "request pkg must beat the disk main declaration"
            );
        }

        // tool + request WITHOUT the field: the external chain answers —
        // same external environment, disk flipped/removed: the (value,
        // source) pair must not move, and the source is never the disk.
        {
            let _tool = set_tool_pm_for_request(&request_with_pkg(None));
            let baseline = resolve_with_source();
            assert!(
                baseline.1 == PmSource::Global || baseline.1 == PmSource::AutoDetect,
                "no-field tool path resolves from the external chain: {baseline:?}"
            );
            for disk in ["pnpm", "bun"] {
                std::fs::write(
                    dir.path().join("pac.at"),
                    format!("name: \"diskx\"\npkg: \"{disk}\"\n"),
                )
                .unwrap();
                assert_eq!(
                    resolve_with_source(),
                    baseline,
                    "disk main pac change ({disk}) must not affect the tool path"
                );
            }
            std::fs::remove_file(dir.path().join("pac.at")).unwrap();
            assert_eq!(
                resolve_with_source(),
                baseline,
                "disk main pac removal must not affect the tool path"
            );
            // Restore the disk pac for the legacy checks below.
            std::fs::write(dir.path().join("pac.at"), "name: \"diskx\"\npkg: \"bun\"\n").unwrap();
        }

        // guard dropped: legacy state restored — source back to the disk
        // chain (value: the process-cached legacy baseline).
        assert_eq!(
            resolve_with_source(),
            legacy_baseline,
            "guard drop must restore the legacy (disk) resolution state"
        );

        // FRESH-PROCESS disk-read proof: a child test binary with cwd =
        // the disk pac dir resolves legacy from the disk (no prior cache
        // exists there) — deterministic on every machine and order
        // (PLAN-815 T-05 R7 / F-815-T05-R7-01).
        let disk = tempfile::tempdir().expect("disk dir");
        std::fs::write(
            disk.path().join("pac.at"),
            "name: \"diskfresh\"
pkg: \"bun\"
",
        )
        .unwrap();
        let out = std::process::Command::new(std::env::current_exe().expect("test exe"))
            .args([
                "pkg::tests::pkg_legacy_disk_probe".to_string(),
                "--exact".to_string(),
                "--nocapture".to_string(),
            ])
            .current_dir(disk.path())
            .env("PLAN815_LEGACY_DISK_PROBE", "1")
            .output()
            .expect("spawn probe");
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(
            stdout.contains("LEGACY_DISK_PROBE_OK(Bun, DiskLegacy)"),
            "fresh-process legacy disk read: rc={} stdout={stdout} stderr={}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// Fresh-process legacy disk-read probe: the parent spawns this single
    /// test with cwd = a disk pac dir; the child process has NO prior
    /// CACHED_PM, so the legacy chain genuinely reads the disk.
    #[test]
    fn pkg_legacy_disk_probe() {
        if std::env::var_os("PLAN815_LEGACY_DISK_PROBE").is_none() {
            return; // plain module run: no-op (only the child proves it)
        }
        let resolved = resolve_with_source();
        println!("LEGACY_DISK_PROBE_OK({:?}, {:?})", resolved.0, resolved.1);
        assert_eq!(resolved, (PkgManagerKind::Bun, PmSource::DiskLegacy));
    }

    /// The auto-detect chain's own oracle under a deterministic stubbed
    /// PATH: bun-only environments LEGITIMATELY resolve bun (R6-01 — the
    /// old matrix wrongly rejected this), and pnpm wins when both exist.
    #[test]
    fn auto_detect_fallback_priority_matrix() {
        let _guard = PM_STATE_LOCK.lock().unwrap();
        let _env = EnvGuard::capture();
        let bun_only = stub_path(&["bun"]);
        // SAFETY: single writer under PM_STATE_LOCK.
        unsafe { set_stub_path(bun_only.path()) };
        assert!(!command_exists("pnpm"), "stub isolation: no pnpm");
        assert!(command_exists("bun"), "stub isolation: bun present");
        assert_eq!(
            auto_detect(),
            PkgManagerKind::Bun,
            "bun-only fallback environment legitimately resolves bun"
        );
        let bun_pnpm = stub_path(&["bun", "pnpm"]);
        // SAFETY: single writer under PM_STATE_LOCK.
        unsafe { set_stub_path(bun_pnpm.path()) };
        assert_eq!(
            auto_detect(),
            PkgManagerKind::Pnpm,
            "pnpm is preferred over bun in auto-detect"
        );
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
        let _guard = PM_STATE_LOCK.lock().unwrap();
        let a = resolve();
        let b = resolve();
        assert_eq!(a, b);
    }

    #[test]
    fn test_command_names_are_consistent() {
        let _guard = PM_STATE_LOCK.lock().unwrap();
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
