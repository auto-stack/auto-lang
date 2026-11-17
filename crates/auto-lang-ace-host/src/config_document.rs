//! PLAN-982（B-CONFIG-I，E6）：配置文档的**宿主桥**。
//!
//! 设计（`docs/design/ace-config-document-evaluation.md` §3.2/§6）：公开面是
//! `auto_lang_core::ace_config` 的中立合同；本模块把一次
//! [`AceConfigRequest`] 端到端落到 ACE 的宿主车道：
//!
//! `compile_config_document`（ACE 源前门：文档行→单顺序入口的 `cfg.*` 调用）
//! → `lower_checked_owned`（共享 Checked HIR→MIR）→ `verify_mir` → ABC
//! 编码/准入 → 打开配置会话 → AV 执行（表达式/`if`/模板都是真实指令）→
//! 关闭会话取 owned 快照 → 投影成中立 [`ConfigDocument`]。
//!
//! 纪律：
//! - **无 legacy 回退**：任何失败都是具名终态（`Diagnostic`/`Denied`/
//!   `ResourceDenied`/`Cancelled`/`HostFailure`），绝不调旧
//!   `eval_config_with_vm`；
//! - 会话在**所有**路径上关闭或丢弃（失败不留全局状态）；
//! - 预算逐轴映射（steps/wall → AV 执行预算；nodes/bytes → 会话预算），
//!   超出是 `ResourceDenied` 而不是截断；
//! - 结果身份由 SDK 侧 `ConfigIdentity::of` 生成（engine=`av`），代价来自真实
//!   观测（AV steps、导出节点/字节/深度、原语 dispatch 数）。

use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use ace_av_core::contract::ExecOutcome;
use ace_av_core::{encode_abc, verify_abc_bytes, verify_mir, CancelFlag, ExecLimits};
use auto_lang_core::ace_config::{
    AceConfigOutcome, AceConfigReport, AceConfigRequest, ConfigCost, ConfigDocument,
    ConfigIdentity, ConfigNode, ConfigSpan, ConfigValue, CONFIG_ENGINE_ID,
};

/// 一次配置求值的墙钟上限（请求未给时用 SDK 默认；这里只兜底防跑飞）。
const DEFAULT_WALL: Duration = Duration::from_secs(5);
/// 单次配置文档的 AV 帧上限（配置文档只有单入口 + 内联表达式调用）。
const MAX_FRAMES: usize = 4096;

/// 把请求上下文里的 args 转成注入值（legacy 语义 + 内建 `on`/`off`）。
///
/// 只接受宿主车道能精确承载的种类：bool / i32 范围内的整数 / char / text。
/// 浮点、null、对象、数组与超范围整数都是**具名拒绝**（不静默丢弃、不截断）。
fn inject_from_context(
    context: &auto_lang_core::ace_config::ConfigContext,
) -> Result<Vec<(String, ace_core::source::InjectValue)>, (String, String)> {
    use ace_core::source::InjectValue;
    use auto_lang_core::ace_config::ConfigValue;
    let mut out: Vec<(String, InjectValue)> = Vec::with_capacity(context.args.len() + 2);
    for (name, value) in &context.args {
        let injected = match value {
            ConfigValue::Bool(b) => InjectValue::Bool(*b),
            ConfigValue::Char(c) => InjectValue::Char(*c),
            ConfigValue::Text(t) => InjectValue::Text(t.clone()),
            ConfigValue::I64(n) => match i32::try_from(*n) {
                Ok(v) => InjectValue::I32(v),
                Err(_) => {
                    return Err((
                        "ace.config/inject-range".to_string(),
                        format!(
                            "context argument `{name}` = {n} is outside the document's i32 value \
                             range; the configuration lane refuses instead of truncating"
                        ),
                    ))
                }
            },
            ConfigValue::U64(n) => match i32::try_from(*n) {
                Ok(v) => InjectValue::I32(v),
                Err(_) => {
                    return Err((
                        "ace.config/inject-range".to_string(),
                        format!(
                            "context argument `{name}` = {n} is outside the document's i32 value \
                             range; the configuration lane refuses instead of truncating"
                        ),
                    ))
                }
            },
            other => {
                return Err((
                    "ace.config/inject-unsupported".to_string(),
                    format!(
                        "context argument `{name}` has kind `{}`, which the document value lane \
                         cannot carry yet (bool/i32/char/text only)",
                        other.kind()
                    ),
                ))
            }
        };
        out.push((name.clone(), injected));
    }
    // legacy：`on`/`off` 是内建布尔；请求显式给了同名值则以请求为准。
    for (builtin, value) in [("on", true), ("off", false)] {
        if !out.iter().any(|(name, _)| name == builtin) {
            out.push((builtin.to_string(), InjectValue::Bool(value)));
        }
    }
    Ok(out)
}
fn now_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn report(outcome: AceConfigOutcome, start: Instant) -> AceConfigReport {
    AceConfigReport {
        outcome,
        wall_ms: now_ms(start),
    }
}

fn denied(code: &str, message: impl Into<String>, start: Instant) -> AceConfigReport {
    report(
        AceConfigOutcome::Denied {
            code: code.to_string(),
            message: message.into(),
        },
        start,
    )
}

fn host_failure(code: &str, message: impl Into<String>, start: Instant) -> AceConfigReport {
    report(
        AceConfigOutcome::HostFailure {
            code: code.to_string(),
            message: message.into(),
        },
        start,
    )
}

fn diagnostic(
    stage: &str,
    code: &str,
    message: impl Into<String>,
    span: Option<ConfigSpan>,
    start: Instant,
) -> AceConfigReport {
    report(
        AceConfigOutcome::Diagnostic {
            stage: stage.to_string(),
            code: code.to_string(),
            message: message.into(),
            span,
        },
        start,
    )
}

/// Serve one configuration-document request through the ACE AV spine.
pub fn eval_request(request: &AceConfigRequest<'_>) -> AceConfigReport {
    let start = Instant::now();
    // 1) 能力：配置求值自身的能力键由桥持有（它是本引擎的语义，不是宿主资源）；
    //    其它任何 `requires` 都必须由请求 grants 显式覆盖，否则具名拒绝。
    let mut grants: Vec<String> = request.context.grants.clone();
    if !grants
        .iter()
        .any(|g| g == ace_av_core::config_document::CAPABILITY)
    {
        grants.push(ace_av_core::config_document::CAPABILITY.to_string());
    }
    // 2) 上下文注入：legacy 语义——请求 args 在文档里按名可见，`on`/`off`
    //    是内建（请求已给则请求优先）。不能精确承载的值类型具名拒绝。
    let injected = match inject_from_context(request.context) {
        Ok(list) => list,
        Err((code, message)) => return denied(&code, message, start),
    };
    // 3) 源前门：文档行 → 单顺序入口的受检 `cfg.*` 调用。
    let front = match ace_core::source::compile_config_document_with_injection(
        request.source,
        request.virtual_path,
        &grants,
        &injected,
    ) {
        Ok(f) => f,
        Err(diags) => {
            let first = match diags.first() {
                Some(d) => d,
                None => {
                    return diagnostic(
                        "text",
                        "ace.config/empty-diagnostics",
                        "the configuration front end returned no diagnostic",
                        None,
                        start,
                    )
                }
            };
            return diagnostic(
                first.stage.as_str(),
                &first.code,
                &first.message,
                Some(ConfigSpan {
                    start: first.span.start as u64,
                    end: first.span.end as u64,
                }),
                start,
            );
        }
    };
    let entry = format!("d_{}", ace_core::source::CONFIG_DOCUMENT_ENTRY_NAME);
    // 3) 共享降级 → MIR 验证 → ABC 编码/准入。
    let mir = match ace_lowering::lower_checked_owned(&front.compiled.checked, &entry, 0x982) {
        Ok(m) => m,
        Err(e) => {
            return diagnostic(
                "lower",
                "ace.config/lower",
                format!("configuration lowering failed: {e:?}"),
                None,
                start,
            )
        }
    };
    if let Err(e) = verify_mir(&mir) {
        return diagnostic(
            "verify",
            "ace.config/verify",
            format!("configuration MIR failed verification: {e:?}"),
            None,
            start,
        );
    }
    let owner = ace_av_core::mir::OwnerScope {
        schema: mir.owner.schema.clone(),
        module: mir.owner.module.clone(),
    };
    let wire = match encode_abc(&mir) {
        Ok(w) => w,
        Err(e) => {
            return diagnostic(
                "encode",
                "ace.config/encode",
                format!("configuration ABC encoding failed: {e:?}"),
                None,
                start,
            )
        }
    };
    let loaded = match verify_abc_bytes(&wire.to_flat(), &owner) {
        Ok(l) => l,
        Err(e) => {
            return diagnostic(
                "admit",
                "ace.config/admit",
                format!("configuration ABC admission failed: {e:?}"),
                None,
                start,
            )
        }
    };
    // 4) 预算映射：nodes/bytes 进会话，steps/wall 进 AV。
    let wall = request.limits.wall.unwrap_or(DEFAULT_WALL);
    let limits = ExecLimits {
        max_steps: request.limits.steps.max(1),
        max_frames: MAX_FRAMES,
        wall,
        ..ExecLimits::default()
    };
    let session_limits = ace_av_core::config_document::DocumentLimits {
        value_nodes: request.limits.max_nodes.max(1) as usize,
        owned_bytes: request.limits.max_bytes.max(1) as usize,
    };
    if let Err(e) = ace_av_core::config_document::open_session(session_limits) {
        return host_failure(e.code, e.message, start);
    }
    let local_cancel = AtomicBool::new(false);
    let cancel_ref: &CancelFlag = match request.cancel {
        Some(flag) => flag,
        None => &local_cancel,
    };
    let (outcome, _trace) = ace_av_core::execute(&loaded, &limits, cancel_ref);
    let steps = match &outcome {
        ExecOutcome::Success { steps, .. } => *steps,
        _ => 0,
    };
    // 5) 无论成败都收会话：成功取快照，失败丢弃（不留全局状态）。
    if !matches!(
        outcome,
        ExecOutcome::Success { .. } | ExecOutcome::Exit { .. }
    ) {
        ace_av_core::config_document::abandon_session();
        return match outcome {
            ExecOutcome::Cancelled => report(AceConfigOutcome::Cancelled, start),
            ExecOutcome::Resource { kind, message } => report(
                AceConfigOutcome::ResourceDenied {
                    kind: format!("{kind:?}"),
                    message,
                },
                start,
            ),
            ExecOutcome::Trap { kind, .. } => diagnostic(
                "av",
                "ace.config/trap",
                format!("the configuration document trapped: {kind:?}"),
                None,
                start,
            ),
            ExecOutcome::Error { message, span } => diagnostic(
                "av",
                "ace.config/error",
                message,
                span.map(|s| ConfigSpan {
                    start: s.start as u64,
                    end: s.end as u64,
                }),
                start,
            ),
            ExecOutcome::Infra(message) => host_failure("ace.config/infra", message, start),
            other => host_failure(
                "ace.config/outcome",
                format!("unexpected execution outcome {other:?}"),
                start,
            ),
        };
    }
    let owned = match ace_av_core::config_document::close_session() {
        Ok(doc) => doc,
        Err(e) => return host_failure(e.code, e.message, start),
    };
    // 6) 投影成中立文档（形状/顺序/重复/种类保真；不做任何求值）。
    let document = ConfigDocument {
        root: project_node(owned.root),
    };
    let stats = document.stats();
    if stats.nodes > request.limits.max_nodes.max(1) {
        return report(
            AceConfigOutcome::ResourceDenied {
                kind: "nodes".to_string(),
                message: format!(
                    "the evaluated document holds {} nodes, over the {} node budget",
                    stats.nodes, request.limits.max_nodes
                ),
            },
            start,
        );
    }
    if stats.bytes > request.limits.max_bytes.max(1) {
        return report(
            AceConfigOutcome::ResourceDenied {
                kind: "bytes".to_string(),
                message: format!(
                    "the evaluated document retains {} bytes, over the {} byte budget",
                    stats.bytes, request.limits.max_bytes
                ),
            },
            start,
        );
    }
    let cost = ConfigCost {
        steps,
        nodes: stats.nodes,
        bytes: stats.bytes,
        depth: stats.depth,
        callee_count: Some(owned.cost.callees),
    };
    report(
        AceConfigOutcome::Success {
            document,
            identity: ConfigIdentity::of(request, CONFIG_ENGINE_ID),
            cost,
        },
        start,
    )
}

fn project_node(node: ace_av_core::config_document::OwnedNode) -> ConfigNode {
    let id = node.id.map(|id| match id {
        ace_av_core::config_document::OwnedId::Name(name) => name,
        ace_av_core::config_document::OwnedId::Text(text) => text,
    });
    ConfigNode {
        name: node.name,
        id,
        args: node.args.into_iter().map(project_value).collect(),
        props: node
            .props
            .into_iter()
            .map(|(k, v)| (k, project_value(v)))
            .collect(),
        children: node.children.into_iter().map(project_node).collect(),
    }
}

fn project_value(value: ace_av_core::config_document::OwnedValue) -> ConfigValue {
    use ace_av_core::config_document::OwnedValue as V;
    match value {
        V::I32(n) => ConfigValue::I64(n as i64),
        V::Bool(b) => ConfigValue::Bool(b),
        V::Char(c) => ConfigValue::Char(c),
        V::Text(t) => ConfigValue::Text(t),
        V::Node(n) => ConfigValue::Node(Box::new(project_node(*n))),
        V::Obj(fields) => ConfigValue::Object(
            fields
                .into_iter()
                .map(|(k, v)| (k, project_value(v)))
                .collect(),
        ),
        V::List(items) => ConfigValue::List(items.into_iter().map(project_value).collect()),
    }
}

/// 供测试/调试：当前线程是否有打开的配置会话（桥在每次请求后必须为 false）。
pub fn session_open() -> bool {
    ace_av_core::config_document::session_is_open()
}
