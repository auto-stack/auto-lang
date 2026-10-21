//! PLAN-816 T-04：I-TOOL 工具发现（窄能力，M815-A 面）。
//!
//! 发现序（contract-baseline §5）：
//! 1. **env 覆盖**（`AUTO_UI_TOOL_BIN` 等具名键）——显式路径，必须存在；
//! 2. **已配置/已安装匹配 tool**：候选二进制经真实 `describe` 核验
//!    协议版本与所需能力；
//! 3. **开发/安装解析**：workspace `target/{debug,release}` →
//!    与当前可执行文件邻接的安装目录。
//!
//! 禁止：`build.rs` 偷编译 UI、搜索任意兄弟仓、隐式下载。完整安装
//! profile / 开发 bootstrap 构建归 M815-B；本模块只解析**已存在**产物
//! 并用 815 describe/版本做身份核验。探测收尾（shutdown 流/退出）失败
//! 必须具名传播，不得当作已发现成功。

use std::path::{Path, PathBuf};

use crate::{ClientError, ExitKind, ToolClient, ToolClientSpec};
use auto_tool_protocol::{ErrorKind, ToolDescriptor, PROTOCOL_VERSION};

/// 发现配置：环境键、期望工具名、协议版本与（可选）必需能力。
#[derive(Debug, Clone)]
pub struct DiscoverySpec {
    /// env 覆盖键（如 `AUTO_UI_TOOL_BIN`）。
    pub env_key: String,
    /// 期望的 `tool_name`（describe 核验）。
    pub tool_name: String,
    /// 期望协议版本。
    pub protocol_version: u32,
    /// 必需能力操作名（describe 核验；空 = 只验身份/版本）。
    pub required_operations: Vec<String>,
    /// 开发解析用的 workspace 根（`CARGO_MANIFEST_DIR` 向上到 target 的锚）。
    /// None = 从当前可执行文件/环境推断。
    pub workspace_root: Option<PathBuf>,
}

impl DiscoverySpec {
    pub fn auto_ui_tool() -> Self {
        Self {
            env_key: "AUTO_UI_TOOL_BIN".into(),
            tool_name: "auto-ui-tool".into(),
            protocol_version: PROTOCOL_VERSION,
            required_operations: vec!["describe".into(), "validate_project".into()],
            workspace_root: None,
        }
    }
}

/// 发现失败——具名分类，不静默跳过、不隐式 fallback。
#[derive(Debug)]
pub enum DiscoveryError {
    /// env 指定的路径不存在。
    EnvPathMissing { env_key: String, path: PathBuf },
    /// 所有候选均不存在。
    NotFound {
        tool_name: String,
        candidates: Vec<PathBuf>,
    },
    /// 候选存在但 describe 失败（不可执行/非协议宿主）。
    DescribeFailed { path: PathBuf, source: ClientError },
    /// 协议/工具版本不匹配。
    VersionMismatch {
        path: PathBuf,
        expected_protocol: u32,
        actual_protocol: u32,
        expected_tool: String,
        actual_tool: String,
    },
    /// 缺少必需能力（缺 UI/缺操作）——**无隐式 fallback**。
    CapabilityMissing {
        path: PathBuf,
        operation: String,
        offered: Vec<String>,
    },
    /// describe 已成功，但探测进程**收尾失败**（流错误/收尾客户端错误，
    /// 或非 Success 退出）——不得当作已发现成功，也不回落到其他候选。
    TeardownFailed { path: PathBuf, reason: String },
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiscoveryError::EnvPathMissing { env_key, path } => write!(
                f,
                "tool discovery: {env_key} points to missing path {}",
                path.display()
            ),
            DiscoveryError::NotFound {
                tool_name,
                candidates,
            } => write!(
                f,
                "tool discovery: `{tool_name}` not found; candidates tried: {:?}",
                candidates
            ),
            DiscoveryError::DescribeFailed { path, source } => write!(
                f,
                "tool discovery: describe failed for {}: {source}",
                path.display()
            ),
            DiscoveryError::VersionMismatch {
                path,
                expected_protocol,
                actual_protocol,
                expected_tool,
                actual_tool,
            } => write!(
                f,
                "tool discovery: version mismatch at {} — tool `{actual_tool}` protocol {actual_protocol} (expected `{expected_tool}` / protocol {expected_protocol})",
                path.display()
            ),
            DiscoveryError::CapabilityMissing {
                path,
                operation,
                offered,
            } => write!(
                f,
                "tool discovery: `{operation}` not offered by {} (offered: {offered:?}) — no fallback",
                path.display()
            ),
            DiscoveryError::TeardownFailed { path, reason } => write!(
                f,
                "tool discovery: probe teardown failed for {}: {reason}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for DiscoveryError {}

/// 发现结果：真实二进制路径 + describe 身份。
#[derive(Debug, Clone)]
pub struct DiscoveredTool {
    pub path: PathBuf,
    pub descriptor: ToolDescriptor,
}

/// 按发现序解析工具二进制，并以真实 describe 核验身份/版本/能力。
///
/// **无隐式 UI fallback**：缺件、版本错、缺能力均具名失败。
pub fn discover(spec: &DiscoverySpec) -> Result<DiscoveredTool, DiscoveryError> {
    let candidates = candidate_paths(spec)?;
    let mut last_not_found: Vec<PathBuf> = Vec::new();
    for path in &candidates {
        if !path.exists() {
            last_not_found.push(path.clone());
            continue;
        }
        return verify_tool(spec, path);
    }
    Err(DiscoveryError::NotFound {
        tool_name: spec.tool_name.clone(),
        candidates: last_not_found,
    })
}

/// 仅解析路径（不 spawn describe）——供已显式给路径的调用方做存在性检查。
pub fn resolve_existing(spec: &DiscoverySpec) -> Result<PathBuf, DiscoveryError> {
    let candidates = candidate_paths(spec)?;
    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    Err(DiscoveryError::NotFound {
        tool_name: spec.tool_name.clone(),
        candidates,
    })
}

fn candidate_paths(spec: &DiscoverySpec) -> Result<Vec<PathBuf>, DiscoveryError> {
    let mut out = Vec::new();
    // ① env 覆盖（最高优先；指定了就必须存在，不回落）
    if let Some(raw) = std::env::var_os(&spec.env_key) {
        let p = PathBuf::from(raw);
        if !p.exists() {
            return Err(DiscoveryError::EnvPathMissing {
                env_key: spec.env_key.clone(),
                path: p,
            });
        }
        return Ok(vec![p]);
    }
    // ② workspace target/{debug,release}（开发已构建产物）
    let exe = exe_name(&spec.tool_name);
    if let Some(root) = workspace_root(spec) {
        for profile in ["debug", "release"] {
            out.push(root.join("target").join(profile).join(&exe));
        }
    }
    // ③ 邻接当前可执行文件（独立安装 profile 形态：tool 与宿主同目录）
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            out.push(dir.join(&exe));
        }
    }
    Ok(out)
}

fn verify_tool(spec: &DiscoverySpec, path: &Path) -> Result<DiscoveredTool, DiscoveryError> {
    let mut client = ToolClient::spawn(ToolClientSpec::new(path)).map_err(|source| {
        DiscoveryError::DescribeFailed {
            path: path.to_path_buf(),
            source,
        }
    })?;
    let descriptor = client
        .describe()
        .map_err(|source| DiscoveryError::DescribeFailed {
            path: path.to_path_buf(),
            source,
        })?;
    // 收尾结果不得丢弃：流错误与非 Success 退出都是探测失败
    // （F-816-T04-R1-01），且不回落到其他候选。
    let shutdown = client
        .shutdown()
        .map_err(|source| DiscoveryError::TeardownFailed {
            path: path.to_path_buf(),
            reason: format!("shutdown client error: {source}"),
        })?;
    match shutdown.kind {
        ExitKind::Success => {}
        other => {
            return Err(DiscoveryError::TeardownFailed {
                path: path.to_path_buf(),
                reason: format!("probe host exit: {other:?}"),
            })
        }
    }

    if descriptor.tool_name != spec.tool_name
        || descriptor.protocol_version != spec.protocol_version
    {
        return Err(DiscoveryError::VersionMismatch {
            path: path.to_path_buf(),
            expected_protocol: spec.protocol_version,
            actual_protocol: descriptor.protocol_version,
            expected_tool: spec.tool_name.clone(),
            actual_tool: descriptor.tool_name.clone(),
        });
    }
    for op in &spec.required_operations {
        if !descriptor.offers(op) {
            return Err(DiscoveryError::CapabilityMissing {
                path: path.to_path_buf(),
                operation: op.clone(),
                offered: descriptor
                    .capabilities
                    .iter()
                    .map(|c| c.operation.clone())
                    .collect(),
            });
        }
    }
    Ok(DiscoveredTool {
        path: path.to_path_buf(),
        descriptor,
    })
}

fn exe_name(tool_name: &str) -> String {
    if cfg!(windows) {
        format!("{tool_name}.exe")
    } else {
        tool_name.to_string()
    }
}

fn workspace_root(spec: &DiscoverySpec) -> Option<PathBuf> {
    if let Some(r) = &spec.workspace_root {
        return Some(r.clone());
    }
    // 从当前源码包锚点推断（集成测试的 CARGO_MANIFEST_DIR 在 fixture 下）。
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut p = PathBuf::from(manifest);
        for _ in 0..6 {
            if p.join("Cargo.toml").exists() && p.join("crates").is_dir() {
                return Some(p);
            }
            if !p.pop() {
                break;
            }
        }
    }
    None
}

/// 缺 UI 能力负控辅助：对已发现工具请求表外操作应由宿主 `UnknownOperation`
/// 拒绝且零副作用（M815-A 窄工具无 generate/build/run）。
pub fn assert_no_ui_fallback_error(err: &DiscoveryError) -> bool {
    matches!(err, DiscoveryError::CapabilityMissing { .. })
}

/// 版本错负控辅助。
pub fn assert_version_error(err: &DiscoveryError) -> bool {
    matches!(err, DiscoveryError::VersionMismatch { .. })
}

/// 缺件负控辅助。
pub fn assert_missing_tool(err: &DiscoveryError) -> bool {
    matches!(
        err,
        DiscoveryError::NotFound { .. } | DiscoveryError::EnvPathMissing { .. }
    )
}

/// 收尾失败负控辅助（R1-01）。
pub fn assert_teardown_failed(err: &DiscoveryError) -> bool {
    matches!(err, DiscoveryError::TeardownFailed { .. })
}

// 编译期钉住 ErrorKind 路径可被 discovery 消费方引用（版本/能力拒绝语义）。
#[allow(dead_code)]
fn _kind_anchor() -> ErrorKind {
    ErrorKind::UnknownOperation {
        operation: "generate".into(),
    }
}
