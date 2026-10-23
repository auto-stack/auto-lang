//! Tool identity, capabilities, and lifecycle (PLAN-815 §5.3/§5.5).

use serde::{Deserialize, Serialize};

/// Exit code: stdin reached EOF after a clean shutdown.
pub const EXIT_CODE_OK: i32 = 0;
/// Exit code: host-internal failure.
pub const EXIT_CODE_INTERNAL: i32 = 1;
/// Exit code: stream-level fatal failure (bad/oversized frame) — framing
/// sync is lost, the host must stop reading.
pub const EXIT_CODE_FATAL_STREAM: i32 = 2;

/// One operation the tool offers, with its lifecycle semantics declared up
/// front — clients never have to probe for them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    /// Operation name as it appears in
    /// [`ClientBody`](crate::message::ClientBody).
    pub operation: String,
    /// Whether the operation honors `cancel` while in flight.
    pub cancellable: bool,
    /// Whether the operation may read interactive input from stdin beyond
    /// framed protocol messages.
    pub interactive: bool,
    /// PLAN-816 T-05 R2（F-816-T05-R2-04）：该操作**兑现**的 op 级执行
    /// 选项名（与 [`crate::message::ClientBody`] 各操作的
    /// `#[serde(default)]` 字段名一致，见 [`op_options`]）。缺席/空 =
    /// v1 基线（无执行选项语义）。消费方在携带这些选项发请求前按本
    /// 广播核对：未广播即在**任何副作用前**具名拒绝——serde 对未知
    /// 字段的静默忽略是解码兼容，不等于语义兑现（旧 v1 工具会丢掉
    /// `output`/`strict` 等选项继续生成）。
    #[serde(default)]
    pub options: Vec<String>,
}

/// op 级执行选项名单源（F-816-T05-R2-04 协商载体；客户端与工具共用）。
pub mod op_options {
    /// `generate_ui`：显式输出目录（旧 `gen -o`；rust/jet/ark/vscode）。
    pub const OUTPUT: &str = "output";
    /// `generate_ui`：工程脚手架开关（旧 `gen -p`）。
    pub const PROJECT: &str = "project";
    /// `generate_ui`：792 显式交付模式根（仅 rust）。
    pub const DELIVERY_ROOT: &str = "delivery_root";
    /// `generate_ui`/`build_ui`：严格校验信号（旧 build 默认严格）。
    pub const STRICT: &str = "strict";
    /// `run_ui`：命令行透传参数（旧 `auto run -- <args>`）。
    pub const ARGS: &str = "args";
}

impl Capability {
    pub fn non_interactive(operation: impl Into<String>, cancellable: bool) -> Self {
        Self {
            operation: operation.into(),
            cancellable,
            interactive: false,
            options: Vec::new(),
        }
    }

    /// 带兑现的 op 级执行选项的能力项（F-816-T05-R2-04 广播面）。
    pub fn with_options(
        operation: impl Into<String>,
        cancellable: bool,
        interactive: bool,
        options: &[&str],
    ) -> Self {
        Self {
            operation: operation.into(),
            cancellable,
            interactive,
            options: options.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// True when this operation honors the named op-level execution option
    /// ([`op_options`] names; absent = v1 baseline without the semantics).
    pub fn honors(&self, option: &str) -> bool {
        self.options.iter().any(|o| o == option)
    }
}

/// Lifecycle policy of the tool host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecyclePolicy {
    /// The host shuts down cleanly (exit 0) when stdin reaches EOF.
    pub shutdown_on_eof: bool,
    /// Logs (free-form diagnostics) are written to stderr only; the control
    /// stream (stdout) carries framed protocol messages exclusively.
    pub logs_on_stderr_only: bool,
    /// Exit codes as [code, meaning] pairs, so clients can render them.
    pub exit_codes: Vec<(i32, String)>,
}

impl LifecyclePolicy {
    pub fn contract() -> Self {
        Self {
            shutdown_on_eof: true,
            logs_on_stderr_only: true,
            exit_codes: vec![
                (EXIT_CODE_OK, "clean EOF shutdown".into()),
                (EXIT_CODE_INTERNAL, "internal failure".into()),
                (
                    EXIT_CODE_FATAL_STREAM,
                    "fatal stream/protocol violation".into(),
                ),
            ],
        }
    }
}

/// The `describe` answer: identity + version + capability/lifecycle truth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDescriptor {
    /// Tool identity, e.g. `auto-ui-tool`.
    pub tool_name: String,
    /// Tool implementation version.
    pub tool_version: String,
    /// Protocol version this host speaks.
    pub protocol_version: u32,
    /// Offered operations with lifecycle semantics.
    pub capabilities: Vec<Capability>,
    /// Host lifecycle contract.
    pub lifecycle: LifecyclePolicy,
}

impl ToolDescriptor {
    /// True when `operation` is listed in the capabilities.
    pub fn offers(&self, operation: &str) -> bool {
        self.capabilities.iter().any(|c| c.operation == operation)
    }

    /// The capability entry for `operation`（F-816-T05-R2-04 选项协商
    /// 查询面；None = 未提供该操作）。
    pub fn capability(&self, operation: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.operation == operation)
    }
}
