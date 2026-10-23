//! Shared Rust generation workspace layout (extracted from auto-man
//! rust_ui.rs in PLAN-815 T-04).
//!
//! Single source for the landing-point resolution order, member naming,
//! workspace Cargo.toml/.cargo assembly and lock identity helpers shared by
//! the API server generation (`api_gen`) and the Rust UI generation
//! (`auto-ui-tools::rust_ui`). Both migrated consumers plus auto-man's
//! `http_service` reach these through re-export facades; do not fork copies.

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
fn compute_auto_lang_rel_path(project_dir: &Path, ws_dir: &Path) -> String {
    // PLAN-025:AUTO_LANG_CRATE 环境覆盖(worktree 开发口子)——指向
    // auto-lang crate 目录(绝对或相对),跳过仓库漫步直用(双仓
    // worktree 联调惯例:主检出项目 + 依赖 worktree)。
    if let Ok(p) = std::env::var("AUTO_LANG_CRATE") {
        let t = p.trim().to_string();
        if !t.is_empty() {
            let cand = std::path::PathBuf::from(&t);
            if cand.exists() {
                return compute_relative_path(ws_dir, &cand);
            }
        }
    }
    // PLAN-816 B-R3-07：安装形态 SDK 闭包——CLI 可执行文件邻接 sdk/
    // 目录（install manifest 随附框架源闭包）优先于仓库漫步/主检出回退：
    // 安装机器无实施仓检出，路径不得借用开发树。env 覆盖仍是第一档
    //（开发口子）；sdk/crates/auto-lang 与 sdk/auto-lang 两种布局都认。
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for cand in [
                dir.join("sdk").join("crates").join("auto-lang"),
                dir.join("sdk").join("auto-lang"),
            ] {
                if cand.exists() {
                    return compute_relative_path(ws_dir, &cand);
                }
            }
        }
    }
    // Walk up from project_dir to find the workspace root (has crates/auto-lang).
    // Also check sibling directories at each level — auto-lang may live in a
    // sibling repo (e.g. auto-shell and auto-lang are both under autostack/).
    let mut dir = project_dir.to_path_buf();
    for _ in 0..10 {
        // Direct: this dir has crates/auto-lang
        if dir.join("crates").join("auto-lang").exists() {
            let auto_lang_abs = dir.join("crates").join("auto-lang");
            return compute_relative_path(ws_dir, &auto_lang_abs);
        }
        // Sibling: a sibling dir (e.g. ../auto-lang) has crates/auto-lang.
        // This covers the common layout where <project> and auto-lang are
        // sibling repos under a common parent (autostack/).
        if let Some(parent) = dir.parent() {
            for sibling in ["auto-lang"] {
                let candidate = parent.join(sibling).join("crates").join("auto-lang");
                if candidate.exists() {
                    return compute_relative_path(ws_dir, &candidate);
                }
            }
        }
        if !dir.pop() {
            break;
        }
    }
    // A project outside the repository still uses this generator's framework.
    // Resolve that real crate instead of inventing a project-relative checkout.
    let compiled_crate = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|parent| parent.join("auto-lang"));
    if let Some(crate_dir) = compiled_crate.filter(|path| path.join("Cargo.toml").is_file()) {
        return compute_relative_path(ws_dir, &crate_dir);
    }
    // Fallback
    "../../autostack/auto-lang/crates/auto-lang".to_string()
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
    // Stage B P-2: landing point by resolution order (env → in-framework
    // shared ws → project-local); the rel-path anchors below are computed
    // from this actual landing dir.
    let ws_dir = resolve_rust_workspace_dir(project_dir);
    fs::create_dir_all(&ws_dir).ok();

    let ws_cargo = ws_dir.join("Cargo.toml");

    // Scan existing member directories (each subdirectory with a Cargo.toml).
    // Skip incomplete members (no src/main.rs, src/lib.rs, [lib], or [[bin]]):
    // cargo rejects them with "no targets specified" when resolving the
    // workspace, which would poison unrelated member runs. They get re-included
    // automatically once generate_rust_ui writes their src/.
    let mut members: Vec<String> = Vec::new();
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
                    members.push(name.to_string());
                }
            }
        }
    }
    members.sort();

    let auto_lang_rel = compute_auto_lang_rel_path(project_dir, &ws_dir);
    // PLAN-668 R-24：a2r-std 与 auto-lang 同根（crates/a2r-std）。
    let a2r_std_rel = auto_lang_rel.replace("crates/auto-lang", "crates/a2r-std");
    // PLAN-811：媒体基础与 auto-lang 同根（crates/auto-media-core）——生成
    // 物的 viewer 桥/媒体 HTTP 直接消费 `auto_media_core::` 公开入口。
    let auto_media_core_rel = auto_lang_rel.replace("crates/auto-lang", "crates/auto-media-core");
    // PLAN-814 T-05 canonical 双直依赖（import_policy.rs；PLAN-815 T-07 子批2
    // 接线）：rust 生成组件只 import `auto_ui_runtime::ui`/`auto_lang_core`，
    // dev 成员经 workspace 继承直接依赖两包（同 auto-media-core 先例同根
    // 窄接线；缺此两行则生成工程 cargo build E0433）。
    let auto_ui_runtime_rel = auto_lang_rel.replace("crates/auto-lang", "crates/auto-ui-runtime");
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
auto-ui-runtime = {{ path = "{auto_ui_runtime_rel}" }}
auto-lang-core = {{ path = "{auto_lang_core_rel}" }}
serde_json = "1"
ureq = {{ version = "2", features = ["json"] }}
tokio = {{ version = "1", features = ["rt"] }}
# Plan 413 §5.2:放宽到 "0.14" 让未来补丁版自动流入
iced = {{ version = "0.14", features = ["tokio", "advanced"] }}
axum = "0.7"
# PLAN-736 AC-03/06: generated-backend connection driving (accept loop + drain).
hyper = {{ version = "1", features = ["http1", "server"] }}
hyper-util = {{ version = "0.1", features = ["tokio", "server-auto", "service", "http1"] }}
serde = {{ version = "1", features = ["derive"] }}
tower-http = {{ version = "0.5", features = ["cors"] }}
"#
    );

    // Always rewrite to update members list
    let _ = fs::write(&ws_cargo, &content);

    // Write .cargo/config.toml with target-dir.
    // PLAN-039 T-04（批次 E 连带）：MSVC 主线程栈扩至 256MB——深视图
    // app（klondike 7 列×嵌套条件）的视图构造/drop 递归溢出默认栈
    // （minesweeper 浅视图无此面）；栈是上限非预留，浅 app 无扰。
    let config_dir = ws_dir.join(".cargo");
    fs::create_dir_all(&config_dir).ok();
    let mut config = format!(
        "[build]\ntarget-dir = \"{}\"\n",
        target_rel.replace('\\', "/")
    );
    config.push_str(
        "\n[target.x86_64-pc-windows-msvc]\nrustflags = [\"-C\", \"link-arg=/STACK:268435456\"]\n",
    );
    let _ = fs::write(config_dir.join("config.toml"), config);

    ws_dir
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
        Ok(bytes) => Ok(Some(format!("{:x}", crate::api_gen::fnv1a(&bytes)))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
