//! Shared Rust generation workspace layout (extracted from auto-man
//! rust_ui.rs in PLAN-815 T-04).
//!
//! Single source for the landing-point resolution order, member naming,
//! workspace Cargo.toml/.cargo assembly and lock identity helpers shared by
//! the API server generation (`api_gen`) and the Rust UI generation
//! (`auto-ui-tools::rust_ui`). Both migrated consumers plus auto-man's
//! `http_service` reach these through re-export facades; do not fork copies.

use auto_val::AutoResult;
use std::fs;
use std::path::{Path, PathBuf};

pub fn write_shared_cargo_config(project_dir: &Path, gen_subdir: &str) -> std::io::Result<()> {
    let cargo_dir = project_dir.join("gen").join(gen_subdir).join("rust");
    let config_dir = cargo_dir.join(".cargo");
    fs::create_dir_all(&config_dir)?;

    // Compute relative path from cargo_dir back to workspace root's target/
    let target_rel = find_workspace_target_path(&cargo_dir);

    let config = format!(
        "[build]\ntarget-dir = \"{}\"\n",
        target_rel.replace('\\', "/")
    );
    fs::write(config_dir.join("config.toml"), config)
}

/// Find relative path from a generated rust/ dir to the workspace root's target/ directory.
fn find_workspace_target_path(cargo_dir: &Path) -> String {
    // Walk up from cargo_dir to find workspace root (identified by crates/ directory)
    let mut ups = 0usize;
    let mut dir = cargo_dir.to_path_buf();
    for _ in 0..10 {
        if dir.join("crates").exists() {
            // Found workspace root — build relative path: ../../.../target
            let mut rel = (0..ups).map(|_| "..").collect::<Vec<_>>().join("/");
            if !rel.is_empty() {
                rel.push('/');
            }
            rel.push_str("target");
            return rel;
        }
        if !dir.pop() {
            break;
        }
        ups += 1;
    }
    // Fallback: absolute path to auto-lang/target
    let abs = std::env::current_dir().unwrap_or_default().join("target");
    abs.to_string_lossy().to_string().replace('\\', "/")
}

/// Get the shared Rust workspace directory for all generated UI projects.
///
/// Located inside the repo at `examples/rust-workspace/` and excluded from the
/// main workspace via `exclude` in the root `Cargo.toml`. All generated Rust
/// projects become members of this single workspace, enabling cross-project
/// compilation artifact reuse.
pub fn get_rust_workspace_dir() -> PathBuf {
    // Walk up from the CWD to find the auto-lang repo root (has
    // crates/auto-lang), also checking siblings (autostack/ layout). This
    // keeps the shared workspace at the repo root even when `auto` runs
    // inside a nested example dir; the legacy relative path remains the
    // fallback for standalone layouts.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some(root) = find_lang_repo_root(&cwd) {
        return root.join("examples").join("rust-workspace");
    }
    PathBuf::from("examples/rust-workspace")
}

/// Walk up from `start` looking for the auto-lang repo root: a directory
/// containing `crates/auto-lang`, either directly or as a sibling
/// (`../auto-lang/crates/auto-lang`).
pub fn find_lang_repo_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    for _ in 0..10 {
        if dir.join("crates").join("auto-lang").exists() {
            return Some(dir);
        }
        if let Some(parent) = dir.parent() {
            let sibling = parent.join("auto-lang");
            if sibling.join("crates").join("auto-lang").exists() {
                return Some(sibling);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// Stage B P-2（Design 01 §4-P2）：rust 产物落点解析序——
/// `AUTO_RUST_WORKSPACE`（env，**设置即权威**，可用于钉死落点/测试隔离）→
/// **框架仓内项目 = 共享工作区** `examples/rust-workspace/`（既有行为零变化，
/// P584-D1 语义保留）→ **仓外项目 = 项目仓内** `<project_dir>/rust-workspace/`
/// （产物 member 不再落框架仓，V4 零污染）。
pub fn resolve_rust_workspace_dir(project_dir: &Path) -> PathBuf {
    if let Some(env) = std::env::var_os("AUTO_RUST_WORKSPACE") {
        return PathBuf::from(env);
    }
    let fw_ws = get_rust_workspace_dir();
    if let Some(fw_root) = fw_ws.parent().and_then(|examples| examples.parent()) {
        if fw_root.join("crates").join("auto-lang").exists() && path_contains(fw_root, project_dir)
        {
            return fw_ws;
        }
    }
    project_dir.join("rust-workspace")
}

/// Windows 宽容的路径前缀判断（canonicalize 消化 `..`/盘符大小写/分隔符；
/// 归一小写比较——NTFS 大小写不敏感。双方同源比较：任一 canonicalize 失败
/// 则双方均退化原样，避免 verbatim `\\?\` 前缀与原始形态错配）。
fn path_contains(root: &Path, dir: &Path) -> bool {
    let norm_components = |p: &std::path::PathBuf| -> Vec<String> {
        p.components()
            .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
            .collect()
    };
    let prefix = |r: Vec<String>, d: Vec<String>| -> bool {
        !r.is_empty() && d.len() >= r.len() && d[..r.len()] == r[..]
    };
    match (std::fs::canonicalize(root), std::fs::canonicalize(dir)) {
        (Ok(r), Ok(d)) => prefix(norm_components(&r), norm_components(&d)),
        _ => prefix(
            norm_components(&root.to_path_buf()),
            norm_components(&dir.to_path_buf()),
        ),
    }
}

/// Compute the relative path from the workspace dir (ws landing point) to
/// the auto-lang crate.
fn validate_language_candidate(
    candidate: &Path,
    origin: &str,
    requirements: WorkspaceRequirements,
) -> AutoResult<()> {
    let crates = candidate.parent();
    let root = crates.and_then(Path::parent);
    let valid_workspace = root
        .and_then(|root| fs::read_to_string(root.join("Cargo.toml")).ok())
        .and_then(|text| text.parse::<toml::Value>().ok())
        .and_then(|manifest| {
            manifest
                .get("workspace")
                .and_then(toml::Value::as_table)
                .cloned()
        })
        .is_some();
    let valid = valid_workspace
        && (requirements == WorkspaceRequirements::Service
            || crate::source_owner::valid_source_crate(candidate, "auto-lang"))
        && crates.is_some_and(|dir| {
            ["auto-lang-core", "a2r-std", "auto-media-core"]
                .iter()
                .all(|name| crate::source_owner::valid_source_crate(&dir.join(name), name))
        });
    if !valid {
        return Err(format!(
            "producer {origin} 无效或缺少工作区依赖: {}",
            candidate.display()
        )
        .into());
    }
    if let Some(root) = root {
        if crate::source_owner::is_managed_sdk(root) {
            crate::source_owner::validate_installed_language_source(root)?;
        } else {
            // The facade has optional renderer path dependencies. Cargo resolves
            // them even when disabled, but a service never selects that facade.
            // Validate every actually selected language/media path instead.
            for name in ["auto-lang", "auto-lang-core", "a2r-std", "auto-media-core"] {
                if name == "auto-lang" && requirements == WorkspaceRequirements::Service {
                    continue;
                }
                let package = if name == "auto-lang" {
                    candidate.to_path_buf()
                } else {
                    crates.unwrap().join(name)
                };
                crate::source_owner::validate_development_dependencies(&package)
                    .map_err(|error| format!("producer {origin} 的本地依赖无效: {error}"))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
fn compute_auto_lang_rel_path(project_dir: &Path, ws_dir: &Path) -> AutoResult<String> {
    compute_auto_lang_rel_path_for(project_dir, ws_dir, WorkspaceRequirements::Ui)
}

fn compute_auto_lang_rel_path_for(
    project_dir: &Path,
    ws_dir: &Path,
    requirements: WorkspaceRequirements,
) -> AutoResult<String> {
    for key in [
        "AUTO_DELIVERY_SOURCE_ROOT",
        "AUTO_LANG_ROOT",
        "AUTO_LANG_CRATE",
    ] {
        if let Some(value) = std::env::var_os(key) {
            let path = PathBuf::from(value);
            let candidate = if key == "AUTO_LANG_CRATE" {
                path
            } else {
                path.join("crates/auto-lang")
            };
            validate_language_candidate(&candidate, &format!("显式 {key}"), requirements)?;
            return Ok(compute_relative_path(ws_dir, &candidate));
        }
    }
    // An adjacent managed install owns its SDK even if it is damaged or
    // incomplete; never borrow an ambient checkout or compiled-in path.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sdk = dir.join("sdk");
            if sdk.exists() || dir.join("install-manifest.json").exists() {
                if !dir.join("install-manifest.json").is_file() {
                    return Err("producer 安装清单缺失".into());
                }
                let candidate = sdk.join("crates/auto-lang");
                validate_language_candidate(&candidate, "安装 SDK", requirements)?;
                return Ok(compute_relative_path(ws_dir, &candidate));
            }
        }
    }
    let mut dir = project_dir.to_path_buf();
    for _ in 0..10 {
        let direct = dir.join("crates/auto-lang");
        if direct.exists() {
            validate_language_candidate(&direct, "发现的语言源", requirements)?;
            return Ok(compute_relative_path(ws_dir, &direct));
        }
        if let Some(parent) = dir.parent() {
            let sibling = parent.join("auto-lang/crates/auto-lang");
            if sibling.exists() {
                validate_language_candidate(&sibling, "发现的语言兄弟源", requirements)?;
                return Ok(compute_relative_path(ws_dir, &sibling));
            }
        }
        if !dir.pop() {
            break;
        }
    }
    let compiled_crate = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|parent| parent.join("auto-lang"));
    if let Some(candidate) = compiled_crate.filter(|path| path.exists()) {
        validate_language_candidate(&candidate, "编译期语言源", requirements)?;
        return Ok(compute_relative_path(ws_dir, &candidate));
    }
    Err("producer 语言源缺失: crates/auto-lang".into())
}

/// Compute relative path from `from` to `to` using only `..` and directory names.
fn compute_relative_path(from: &Path, to: &Path) -> String {
    // Canonicalize both paths for reliable comparison
    let from_abs = std::fs::canonicalize(from).unwrap_or_else(|_| from.to_path_buf());
    let to_abs = std::fs::canonicalize(to).unwrap_or_else(|_| to.to_path_buf());

    let from_parts: Vec<&std::ffi::OsStr> = from_abs.iter().collect();
    let to_parts: Vec<&std::ffi::OsStr> = to_abs.iter().collect();

    // Find common prefix length
    let common = from_parts
        .iter()
        .zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Windows drive roots cannot be traversed with `..`.
    if common == 0 {
        let path = to_abs.to_string_lossy();
        return path
            .strip_prefix(r"\\?\")
            .unwrap_or(&path)
            .replace('\\', "/");
    }

    // Go up from `from` to the common ancestor
    let ups = from_parts.len() - common;
    let mut result: Vec<String> = (0..ups).map(|_| "..".to_string()).collect();

    // Then descend to `to`
    for part in &to_parts[common..] {
        result.push(part.to_string_lossy().to_string());
    }

    result.join("/").replace('\\', "/")
}

/// Compute the relative path from the workspace dir (ws landing point) to the
/// auto-lang target/ directory.
/// Resolve the UI owner consistently for generated workspaces and delivery.
fn compute_auto_ui_runtime_rel_path(auto_lang_rel: &str, ws_dir: &Path) -> AutoResult<String> {
    let lang_crate = ws_dir.join(auto_lang_rel.replace(char::from(92), "/"));
    let producer = lang_crate
        .parent()
        .and_then(Path::parent)
        .ok_or("producer 语言源路径无效")?;
    let source = crate::source_owner::ui_crate_source(producer, "auto-ui-runtime")?;
    Ok(compute_relative_path(ws_dir, &source))
}

fn compute_target_rel_path(project_dir: &Path, ws_dir: &Path) -> String {
    let mut dir = project_dir.to_path_buf();
    for _ in 0..10 {
        if dir.join("crates").exists() {
            let target_abs = dir.join("target");
            return compute_relative_path(ws_dir, &target_abs);
        }
        if !dir.pop() {
            break;
        }
    }
    // PLAN-039 T-05 发现臂⑤（台账 M7-c「缺口即发现即修」）：原 fallback
    // "../../autostack/auto-lang/target" 为幻影路径——app 仓布局（auto-os
    // apps/<app>）祖先链永无 crates/，生成物 .cargo/config.toml 恒重定向
    // 到不存在布局（minesweeper 实测 exe 落 apps/autostack/auto-lang/
    // target 幽灵目录；tetris 在役 exe 实为旧默认位产物）。改落
    // workspace 本地默认 "target"（cargo 缺省）——desktop_exe 声明路径
    // 布局无关可循；框架内项目（祖先含 crates/）共享 target 行为不变。
    // 隔离需求由构建时 CARGO_TARGET_DIR env 覆盖（cargo 优先级高于
    // config build.target-dir），不烘焙进持久化配置。
    "target".to_string()
}

/// Absolute path of the repo-wide shared cargo target for a UI project:
/// walk up from the project dir to the auto-lang repo root (has
/// crates/auto-lang), also checking siblings (autostack/ layout).
pub fn shared_cargo_target_dir(project_dir: &Path) -> PathBuf {
    // Respect an explicit caller target. This is required for isolated app
    // worktrees and CI sandboxes where the framework checkout target is not
    // writable or must not be shared with another build.
    if let Ok(raw) = std::env::var("CARGO_TARGET_DIR") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    let mut dir = project_dir.to_path_buf();
    for _ in 0..10 {
        if dir.join("crates").join("auto-lang").exists() {
            return dir.join("target");
        }
        if let Some(parent) = dir.parent() {
            let sibling = parent.join("auto-lang");
            if sibling.join("crates").join("auto-lang").exists() {
                return sibling.join("target");
            }
        }
        if !dir.pop() {
            break;
        }
    }
    get_rust_workspace_dir().join("target")
}

/// Check whether a directory is a *complete* Cargo crate — i.e. it declares at
/// least one target, so cargo won't reject it with "no targets specified in the
/// manifest" when the workspace is resolved.
///
/// Mirrors cargo's own target discovery: a `src/main.rs` or `src/lib.rs`, or an
/// explicit `[lib]` / `[[bin]]` section in `Cargo.toml`.
///
/// An incomplete member (e.g. a half-generated frontend crate whose `src/` was
/// removed) would otherwise poison the whole workspace: cargo validates *every*
/// member when resolving the workspace manifest, so one bad member fails even an
/// unrelated `cargo run --manifest-path <other-member>`.
fn has_cargo_targets(dir: &Path) -> bool {
    if dir.join("src").join("main.rs").exists() || dir.join("src").join("lib.rs").exists() {
        return true;
    }
    // Fall back to explicit target declarations in Cargo.toml.
    if let Ok(content) = fs::read_to_string(dir.join("Cargo.toml")) {
        // A `[lib]` table or any `[[bin]]` array counts as a target. Match on a
        // line-starting header to avoid catching these strings in comments.
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("[lib]") || trimmed.starts_with("[[bin]]") {
                return true;
            }
        }
    }
    false
}

/// Ensure the shared Rust workspace exists and is configured.
///
/// Creates/updates:
/// - `examples/rust-workspace/Cargo.toml` (virtual manifest with all members)
/// - `examples/rust-workspace/.cargo/config.toml` (target-dir pointing to auto-lang/target/)
///
/// Returns the workspace directory path.
pub fn ensure_shared_workspace(project_dir: &Path) -> PathBuf {
    try_ensure_shared_workspace(project_dir).expect("shared workspace producer 预检失败")
}

/// Fallible form used by generators so invalid sources cannot overwrite artifacts.
pub fn try_ensure_shared_workspace(project_dir: &Path) -> AutoResult<PathBuf> {
    try_ensure_workspace_for(project_dir, WorkspaceRequirements::Ui)
}

/// Dependencies needed by the member being generated. Existing UI members
/// keep their shared dependencies when a service member is added later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceRequirements {
    Service,
    Ui,
}

fn manifest_needs_ui(value: &toml::Value) -> bool {
    match value {
        toml::Value::Table(table) => table.iter().any(|(key, entry)| {
            matches!(key.as_str(), "auto-ui-runtime" | "iced")
                || entry
                    .get("package")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|name| matches!(name, "auto-ui-runtime" | "iced"))
                || manifest_needs_ui(entry)
        }),
        _ => false,
    }
}

/// Service-only workspaces do not resolve or require an installed UI SDK.
/// Source validation still finishes before the previous manifest is replaced.
pub fn try_ensure_workspace_for(
    project_dir: &Path,
    requirements: WorkspaceRequirements,
) -> AutoResult<PathBuf> {
    // Stage B P-2: landing point by resolution order (env → in-framework
    // shared ws → project-local); the rel-path anchors below are computed
    // from this actual landing dir.
    let ws_dir = resolve_rust_workspace_dir(project_dir);
    fs::create_dir_all(&ws_dir)?;

    let ws_cargo = ws_dir.join("Cargo.toml");

    // Scan existing member directories (each subdirectory with a Cargo.toml).
    // Skip incomplete members (no src/main.rs, src/lib.rs, [lib], or [[bin]]):
    // cargo rejects them with "no targets specified" when resolving the
    // workspace, which would poison unrelated member runs. They get re-included
    // automatically once generate_rust_ui writes their src/.
    let mut members: Vec<String> = Vec::new();
    let mut needs_ui = requirements == WorkspaceRequirements::Ui;
    if let Ok(entries) = fs::read_dir(&ws_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("Cargo.toml").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip .cargo directory
                    if name == ".cargo" {
                        continue;
                    }
                    if !has_cargo_targets(&path) {
                        eprintln!(
                            "⚠ Skipping incomplete workspace member '{}' \
                             (no src/lib.rs, src/main.rs, [lib], or [[bin]] — \
                             re-included once generated)",
                            name
                        );
                        continue;
                    }
                    let member_manifest: toml::Value =
                        fs::read_to_string(path.join("Cargo.toml"))?.parse()?;
                    needs_ui |= manifest_needs_ui(&member_manifest);
                    members.push(name.to_string());
                }
            }
        }
    }
    members.sort();

    let language_requirements = if needs_ui {
        WorkspaceRequirements::Ui
    } else {
        WorkspaceRequirements::Service
    };
    let auto_lang_rel =
        compute_auto_lang_rel_path_for(project_dir, &ws_dir, language_requirements)?;
    // PLAN-668 R-24：a2r-std 与 auto-lang 同根（crates/a2r-std）。
    let a2r_std_rel = auto_lang_rel.replace("crates/auto-lang", "crates/a2r-std");
    // PLAN-811：媒体基础与 auto-lang 同根（crates/auto-media-core）——生成
    // 物的 viewer 桥/媒体 HTTP 直接消费 `auto_media_core::` 公开入口。
    let auto_media_core_rel = auto_lang_rel.replace("crates/auto-lang", "crates/auto-media-core");
    // The UI runtime has a distinct owner. An installed combined SDK is
    // accepted only with its managed ownership and file receipt; development
    // uses an explicit UI root or the sibling checkout, never a lang shadow.
    let ui_dependencies = if needs_ui {
        let auto_ui_runtime_rel = compute_auto_ui_runtime_rel_path(&auto_lang_rel, &ws_dir)?;
        format!(
            "auto-ui-runtime = {{ path = \"{auto_ui_runtime_rel}\" }}\n\
             iced = {{ version = \"0.14\", features = [\"tokio\", \"advanced\"] }}\n"
        )
    } else {
        String::new()
    };
    let auto_lang_core_rel = auto_lang_rel.replace("crates/auto-lang", "crates/auto-lang-core");
    let target_rel = compute_target_rel_path(project_dir, &ws_dir);

    let members_toml = members
        .iter()
        .map(|m| format!("    \"{}\"", m))
        .collect::<Vec<_>>()
        .join(",\n");

    let content = format!(
        r#"[workspace]
members = [
{members_toml}
]
resolver = "2"

[workspace.dependencies]
auto-lang = {{ path = "{auto_lang_rel}" }}
# PLAN-668 R-24：a2r_std（sys 原生面/StringBuilder 等）随 auto-lang 同源
# （仓库内 crates/a2r-std）——后端生成物 `a2r_std::sys::*` 调用需此 dep。
a2r-std = {{ path = "{a2r_std_rel}" }}
# PLAN-811：媒体基础（图像资产管线/媒体文件服务）——生成后端的
# `auto_media_core::` 公开入口调用需此 dep。
auto-media-core = {{ path = "{auto_media_core_rel}" }}
# PLAN-814 T-05 canonical 双直依赖（import_policy.rs）——rust 生成组件的
# `auto_ui_runtime::ui`/`auto_lang_core` import 需成员直依赖两包。
{ui_dependencies}auto-lang-core = {{ path = "{auto_lang_core_rel}" }}
serde_json = "1"
ureq = {{ version = "2", features = ["json"] }}
tokio = {{ version = "1", features = ["rt"] }}
axum = "0.7"
# PLAN-736 AC-03/06: generated-backend connection driving (accept loop + drain).
hyper = {{ version = "1", features = ["http1", "server"] }}
hyper-util = {{ version = "0.1", features = ["tokio", "server-auto", "service", "http1"] }}
serde = {{ version = "1", features = ["derive"] }}
tower-http = {{ version = "0.5", features = ["cors"] }}
"#
    );

    // Always rewrite to update members list
    fs::write(&ws_cargo, &content)?;

    // Write .cargo/config.toml with target-dir.
    // PLAN-039 T-04（批次 E 连带）：MSVC 主线程栈扩至 256MB——深视图
    // app（klondike 7 列×嵌套条件）的视图构造/drop 递归溢出默认栈
    // （minesweeper 浅视图无此面）；栈是上限非预留，浅 app 无扰。
    let config_dir = ws_dir.join(".cargo");
    fs::create_dir_all(&config_dir)?;
    let mut config = format!(
        "[build]\ntarget-dir = \"{}\"\n",
        target_rel.replace('\\', "/")
    );
    config.push_str(
        "\n[target.x86_64-pc-windows-msvc]\nrustflags = [\"-C\", \"link-arg=/STACK:268435456\"]\n",
    );
    fs::write(config_dir.join("config.toml"), config)?;

    Ok(ws_dir)
}

/// Get the member directory name for a frontend project in the shared workspace.
pub fn front_member_name(project_dir: &Path) -> String {
    project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("myapp")
        .to_string()
}

/// Get the member directory name for a backend project in the shared workspace.
pub fn back_member_name(project_dir: &Path) -> String {
    let base = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("myapp");
    format!("{}-back", base)
}

/// Cargo.lock 读取结果的身份映射：Ok(bytes)=Some(指纹)（含零字节——实际
/// 身份 811c9dc5，不与 absent 合并）；NotFound=None；其余 IO 错误=Err
/// （不可核验，不冒充 absent；P738-R9-01 fail-closed）。
pub fn workspace_lock_identity(
    result: std::io::Result<Vec<u8>>,
) -> Result<Option<String>, std::io::Error> {
    match result {
        Ok(bytes) => Ok(Some(format!("{:x}", crate::source_owner::fnv1a(&bytes)))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod plan842_tests {
    use super::*;

    #[test]
    fn workspace_explicit_roots_fail_before_overwriting_manifest() {
        const CHILD: &str = "PLAN842_WORKSPACE_ROOT_PROBE";
        if let Ok(mode) = std::env::var(CHILD) {
            let project = PathBuf::from(std::env::var_os("PLAN842_PROJECT").unwrap());
            let ws = resolve_rust_workspace_dir(&project);
            let result = try_ensure_shared_workspace(&project);
            if mode == "valid" {
                assert!(result.is_ok(), "{result:?}");
                let content = fs::read_to_string(ws.join("Cargo.toml")).unwrap();
                assert!(
                    content.contains("auto-ui/crates/auto-ui-runtime"),
                    "{content}"
                );
            } else {
                let error = result.unwrap_err().to_string();
                assert!(
                    error.contains(if mode.starts_with("bad-lang") {
                        "AUTO_LANG_ROOT"
                    } else {
                        "AUTO_UI_SOURCE_ROOT"
                    }),
                    "{error}"
                );
                assert_eq!(
                    fs::read(ws.join("Cargo.toml")).unwrap(),
                    b"previous manifest"
                );
            }
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        let ws = project.join("workspace");
        let lang = tmp.path().join("auto-lang");
        let ui = tmp.path().join("auto-ui");
        fs::create_dir_all(&lang).unwrap();
        fs::write(lang.join("Cargo.toml"), "[workspace]\n").unwrap();
        for name in ["auto-lang", "auto-lang-core", "a2r-std", "auto-media-core"] {
            let package = lang.join("crates").join(name);
            fs::create_dir_all(package.join("src")).unwrap();
            fs::write(
                package.join("Cargo.toml"),
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n"),
            )
            .unwrap();
            fs::write(package.join("src/lib.rs"), "").unwrap();
        }
        let broken_workspace = tmp.path().join("bad-lang-workspace");
        fs::create_dir_all(&broken_workspace).unwrap();
        fs::write(broken_workspace.join("Cargo.toml"), "[workspace").unwrap();
        for name in ["auto-lang", "auto-lang-core", "a2r-std", "auto-media-core"] {
            let source = lang.join("crates").join(name);
            let target = broken_workspace.join("crates").join(name);
            fs::create_dir_all(target.join("src")).unwrap();
            fs::copy(source.join("Cargo.toml"), target.join("Cargo.toml")).unwrap();
            fs::copy(source.join("src/lib.rs"), target.join("src/lib.rs")).unwrap();
        }
        let missing_transitive = tmp.path().join("bad-lang-transitive");
        fs::create_dir_all(&missing_transitive).unwrap();
        fs::write(missing_transitive.join("Cargo.toml"), "[workspace]\n").unwrap();
        for name in ["auto-lang", "auto-lang-core", "a2r-std", "auto-media-core"] {
            let source = lang.join("crates").join(name);
            let target = missing_transitive.join("crates").join(name);
            fs::create_dir_all(target.join("src")).unwrap();
            fs::copy(source.join("Cargo.toml"), target.join("Cargo.toml")).unwrap();
            fs::copy(source.join("src/lib.rs"), target.join("src/lib.rs")).unwrap();
        }
        fs::write(missing_transitive.join("crates/auto-lang/Cargo.toml"),
            "[package]\nname = \"auto-lang\"\nversion = \"0.1.0\"\n[dependencies]\nauto-val = { path = \"../auto-val\" }\n").unwrap();
        for (name, manifest) in [
            ("bad-lang-manifest", "[package"),
            (
                "bad-lang-sibling",
                "[package]\nname = \"auto-lang\"\nversion = \"0.1.0\"\n",
            ),
        ] {
            let root = tmp.path().join(name);
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
            let package = root.join("crates/auto-lang");
            fs::create_dir_all(package.join("src")).unwrap();
            fs::write(package.join("Cargo.toml"), manifest).unwrap();
            fs::write(package.join("src/lib.rs"), "").unwrap();
        }
        fs::create_dir_all(ui.join("crates/auto-ui-runtime")).unwrap();
        fs::write(
            ui.join("crates/auto-ui-runtime/Cargo.toml"),
            "[package]\nname = \"auto-ui-runtime\"\nversion = \"0.2.0\"\n",
        )
        .unwrap();
        fs::create_dir_all(ui.join("crates/auto-ui-runtime/src")).unwrap();
        fs::write(ui.join("crates/auto-ui-runtime/src/lib.rs"), "").unwrap();
        for (dir, content) in [
            ("partial-ui", None),
            ("malformed-ui", Some("[package")),
            (
                "wrong-ui",
                Some("[package]\nname = \"other\"\nversion = \"0.2.0\"\n"),
            ),
            (
                "missing-src",
                Some("[package]\nname = \"auto-ui-runtime\"\nversion = \"0.2.0\"\n"),
            ),
        ] {
            let path = tmp.path().join(dir).join("crates/auto-ui-runtime");
            fs::create_dir_all(&path).unwrap();
            if let Some(content) = content {
                fs::write(path.join("Cargo.toml"), content).unwrap();
            }
        }
        let bad_ui_transitive = tmp.path().join("bad-ui-transitive/crates/auto-ui-runtime");
        fs::create_dir_all(bad_ui_transitive.join("src")).unwrap();
        fs::write(bad_ui_transitive.join("Cargo.toml"), "[package]\nname = \"auto-ui-runtime\"\nversion = \"0.2.0\"\n[dependencies]\nauto-ui-compiler = { path = \"../auto-ui-compiler\" }\n").unwrap();
        fs::write(bad_ui_transitive.join("src/lib.rs"), "").unwrap();
        fs::create_dir_all(&ws).unwrap();
        for mode in [
            "bad-lang",
            "bad-lang-manifest",
            "bad-lang-sibling",
            "bad-lang-workspace",
            "bad-lang-transitive",
            "bad-ui",
            "partial-ui",
            "malformed-ui",
            "wrong-ui",
            "missing-src",
            "bad-ui-transitive",
            "valid",
        ] {
            fs::write(ws.join("Cargo.toml"), b"previous manifest").unwrap();
            let result = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "rust_workspace::plan842_tests::workspace_explicit_roots_fail_before_overwriting_manifest", "--nocapture"])
                .env(CHILD, mode)
                .env("PLAN842_PROJECT", &project)
                .env("AUTO_RUST_WORKSPACE", &ws)
                .env_remove("AUTO_DELIVERY_SOURCE_ROOT")
                .env_remove("AUTO_LANG_CRATE")
                .env("AUTO_LANG_ROOT", match mode { "bad-lang" => tmp.path().join("missing-lang"), "bad-lang-manifest" | "bad-lang-sibling" | "bad-lang-workspace" | "bad-lang-transitive" => tmp.path().join(mode), _ => lang.clone() })
                .env("AUTO_UI_SOURCE_ROOT", match mode { "bad-ui" => tmp.path().join("missing-ui"), "partial-ui" | "malformed-ui" | "wrong-ui" | "missing-src" | "bad-ui-transitive" => tmp.path().join(mode), _ => ui.clone() })
                .output().unwrap();
            assert!(
                result.status.success(),
                "{mode}: {} {}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }

    #[test]
    fn workspace_discovery_rejects_broken_selected_root() {
        const CHILD: &str = "PLAN842_DISCOVERY_PROBE";
        if let Ok(mode) = std::env::var(CHILD) {
            let base = PathBuf::from(std::env::var_os("PLAN842_DISCOVERY_DIR").unwrap());
            let project = if mode == "direct" {
                base.join("direct/project")
            } else {
                base.join("sibling/project")
            };
            let error = compute_auto_lang_rel_path(&project, &base.join("workspace"))
                .unwrap_err()
                .to_string();
            assert!(error.contains("producer"), "{error}");
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        for (root, project) in [
            (tmp.path().join("direct"), tmp.path().join("direct/project")),
            (
                tmp.path().join("sibling/auto-lang"),
                tmp.path().join("sibling/project"),
            ),
        ] {
            fs::create_dir_all(root.join("crates/auto-lang")).unwrap();
            fs::write(root.join("crates/auto-lang/Cargo.toml"), "[package").unwrap();
            fs::create_dir_all(project).unwrap();
        }
        for mode in ["direct", "sibling"] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "rust_workspace::plan842_tests::workspace_discovery_rejects_broken_selected_root", "--nocapture"])
                .env(CHILD, mode).env("PLAN842_DISCOVERY_DIR", tmp.path())
                .env_remove("AUTO_DELIVERY_SOURCE_ROOT").env_remove("AUTO_LANG_ROOT").env_remove("AUTO_LANG_CRATE")
                .output().unwrap();
            assert!(
                output.status.success(),
                "{mode}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn workspace_discovery_accepts_complete_development_roots() {
        const CHILD: &str = "PLAN842_DISCOVERY_VALID_PROBE";
        if let Ok(mode) = std::env::var(CHILD) {
            let base = PathBuf::from(std::env::var_os("PLAN842_DISCOVERY_DIR").unwrap());
            let (root, project) = if mode == "direct" {
                (base.join("direct"), base.join("direct/project"))
            } else {
                (base.join("sibling/auto-lang"), base.join("sibling/project"))
            };
            let ws = base.join("workspace");
            let selected = compute_auto_lang_rel_path(&project, &ws).unwrap();
            assert_eq!(
                fs::canonicalize(ws.join(selected)).unwrap(),
                fs::canonicalize(root.join("crates/auto-lang")).unwrap()
            );
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        for (root, project) in [
            (tmp.path().join("direct"), tmp.path().join("direct/project")),
            (
                tmp.path().join("sibling/auto-lang"),
                tmp.path().join("sibling/project"),
            ),
        ] {
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
            for name in ["auto-lang", "auto-lang-core", "a2r-std", "auto-media-core"] {
                let package = root.join("crates").join(name);
                fs::create_dir_all(package.join("src")).unwrap();
                fs::write(
                    package.join("Cargo.toml"),
                    format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n"),
                )
                .unwrap();
                fs::write(package.join("src/lib.rs"), "").unwrap();
            }
            fs::create_dir_all(project).unwrap();
        }
        for mode in ["direct", "sibling"] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "rust_workspace::plan842_tests::workspace_discovery_accepts_complete_development_roots", "--nocapture"])
                .env(CHILD, mode).env("PLAN842_DISCOVERY_DIR", tmp.path())
                .env_remove("AUTO_DELIVERY_SOURCE_ROOT").env_remove("AUTO_LANG_ROOT").env_remove("AUTO_LANG_CRATE")
                .output().unwrap();
            assert!(
                output.status.success(),
                "{mode}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    fn workspace_chooses_ui_owner_and_rejects_lang_shadow() {
        let tmp = tempfile::tempdir().unwrap();
        let lang = tmp.path().join("auto-lang");
        let ws = tmp.path().join("project/workspace");
        let shadow = lang.join("crates/auto-ui-runtime");
        let canonical = tmp.path().join("auto-ui/crates/auto-ui-runtime");
        fs::create_dir_all(lang.join("crates/auto-lang")).unwrap();
        fs::create_dir_all(&ws).unwrap();
        fs::create_dir_all(&shadow).unwrap();
        let lang_rel = compute_relative_path(&ws, &lang.join("crates/auto-lang"));
        assert!(compute_auto_ui_runtime_rel_path(&lang_rel, &ws).is_err());
        fs::create_dir_all(&canonical).unwrap();
        fs::write(
            canonical.join("Cargo.toml"),
            "[package]\nname = \"auto-ui-runtime\"\nversion = \"0.2.0\"\n",
        )
        .unwrap();
        fs::create_dir_all(canonical.join("src")).unwrap();
        fs::write(canonical.join("src/lib.rs"), "").unwrap();
        let chosen = compute_auto_ui_runtime_rel_path(&lang_rel, &ws).unwrap();
        assert_eq!(
            fs::canonicalize(ws.join(chosen)).unwrap(),
            fs::canonicalize(canonical).unwrap()
        );
    }
}
