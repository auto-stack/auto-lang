//! PLAN-970 (T-03): the explicit-ACE session **MCP server** — JSON-RPC
//! 2.0 over stdio on the neutral session contract (`crate::ace_session`).
//!
//! Protocol discipline (frozen by the PLAN-970 T-01 freeze record):
//! - `initialize` answers with THIS server's supported protocol version
//!   (`2024-11-05`) — the client disconnects if incompatible (the MCP
//!   negotiation the legacy server ignored).
//! - The legacy seven tool names are kept (`auto_session_create`,
//!   `auto_evaluate`, `auto_session_reset`, `auto_inspect`,
//!   `auto_typecheck`, `auto_patch`, `auto_snapshot`); the session
//!   capability adds `auto_edit` (same-PID online update) and
//!   `auto_vars` (the carry table).
//! - No legacy engine is reachable from here: every tool dispatches the
//!   neutral ACE session contract; an unregistered host is a named tool
//!   error, never a fallback.

use serde_json::{json, Value};

use super::protocol::{
    ContentBlock, Id, JsonRpcError, JsonRpcRequest, JsonRpcResponse, ToolResult,
};
use crate::ace_session::{
    ace_session_create, ace_session_op, AceSessionOp, AceSessionOutcome, AceSessionReport,
    AceSessionValue, SESSION_PROTOCOL_NAME, SESSION_PROTOCOL_VERSION,
};

/// The MCP protocol version this server speaks.
pub const ACE_MCP_PROTOCOL_VERSION: &str = "2024-11-05";

pub struct AceMcpServer {
    initialized: bool,
}

impl Default for AceMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl AceMcpServer {
    pub fn new() -> Self {
        AceMcpServer { initialized: false }
    }

    /// Serve stdin until EOF (each line one JSON-RPC message).
    pub fn run(&mut self) {
        while let Some(req) = super::protocol::read_message() {
            if let Some(resp) = self.handle(&req) {
                super::protocol::write_response(&resp);
            }
        }
    }

    /// Handle one request; `None` for notifications (no response).
    pub fn handle(&mut self, req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone();
        match req.method.as_str() {
            "initialize" => {
                self.initialized = true;
                Some(JsonRpcResponse::success(
                    id,
                    json!({
                        "protocolVersion": ACE_MCP_PROTOCOL_VERSION,
                        "capabilities": {
                            "tools": { "listChanged": false }
                        },
                        "serverInfo": {
                            "name": "ace-session",
                            "version": "0.1.0",
                        },
                        "sessionProtocol": {
                            "name": SESSION_PROTOCOL_NAME,
                            "version": SESSION_PROTOCOL_VERSION,
                        }
                    }),
                ))
            }
            "notifications/initialized" | "notifications/cancelled" => None,
            "ping" => Some(JsonRpcResponse::success(id, json!({}))),
            "tools/list" => Some(JsonRpcResponse::success(id, json!({ "tools": tools() }))),
            "tools/call" => {
                let params = req.params.clone().unwrap_or(Value::Null);
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                let result = self.call_tool(name, &arguments);
                Some(JsonRpcResponse::success(
                    id,
                    serde_json::to_value(&result).unwrap_or(Value::Null),
                ))
            }
            other => Some(JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {other}"),
                    data: None,
                }),
            }),
        }
    }

    fn call_tool(&mut self, name: &str, args: &Value) -> ToolResult {
        match name {
            "auto_session_create" => {
                let report = ace_session_create(args.get("code").and_then(Value::as_str), None);
                report_tool_result(report, true)
            }
            "auto_evaluate" => {
                let (session, code) = match str_field(args, "session").zip(str_field(args, "code"))
                {
                    Some(pair) => pair,
                    None => return ToolResult::error("auto_evaluate needs session and code"),
                };
                let report = ace_session_op(
                    &session,
                    AceSessionOp::Eval {
                        input: &code,
                        wall: None,
                        cancel: None,
                    },
                );
                report_tool_result(report, false)
            }
            "auto_edit" => {
                let session = match str_field(args, "session") {
                    Some(s) => s,
                    None => return ToolResult::error("auto_edit needs session"),
                };
                let (Some(start), Some(end), Some(insert)) = (
                    args.get("start").and_then(Value::as_u64),
                    args.get("end").and_then(Value::as_u64),
                    str_field(args, "insert"),
                ) else {
                    return ToolResult::error("auto_edit needs start/end/insert");
                };
                let report = ace_session_op(
                    &session,
                    AceSessionOp::Edit {
                        start,
                        end,
                        insert: insert.as_bytes(),
                    },
                );
                report_tool_result(report, false)
            }
            "auto_inspect" => session_only_tool(args, AceSessionOp::Inspect),
            "auto_vars" => session_only_tool(args, AceSessionOp::Vars),
            "auto_snapshot" => session_only_tool(args, AceSessionOp::Snapshot),
            "auto_session_reset" => session_only_tool(args, AceSessionOp::Reset),
            "auto_typecheck" => {
                // Compile-only probe: a THROWAWAY session compiles the
                // source; diagnostics come back, no persistent state.
                let code = match str_field(args, "code") {
                    Some(c) => c,
                    None => return ToolResult::error("auto_typecheck needs code"),
                };
                let report = ace_session_create(Some(&code), None);
                let text = match &report.outcome {
                    AceSessionOutcome::Ok => format!("ok (revision {})", report.revision),
                    AceSessionOutcome::Diagnostic { code, diagnostics } => format!(
                        "{code}: {}",
                        diagnostics
                            .first()
                            .map(|d| d.message.clone())
                            .unwrap_or_default()
                    ),
                    AceSessionOutcome::Denied { code, message }
                    | AceSessionOutcome::HostFailure { code, message } => {
                        format!("{code}: {message}")
                    }
                    other => format!("{other:?}"),
                };
                // The probe session is closed immediately.
                if let Some(session) = report.session.clone() {
                    let _ = ace_session_op(&session, AceSessionOp::Close);
                }
                ToolResult::text(text)
            }
            "auto_patch" => {
                // The legacy text-replace patch on the ACE session: the
                // from-string must occur exactly once in the current
                // source; the patch is a RAW same-PID edit.
                let (session, from, to) = match str_field(args, "session")
                    .zip(str_field(args, "from"))
                    .zip(str_field(args, "to"))
                {
                    Some(((s, f), t)) => (s, f, t),
                    None => return ToolResult::error("auto_patch needs session/from/to"),
                };
                let inspect = ace_session_op(&session, AceSessionOp::Inspect);
                let Some(source) = inspect.snapshot.clone() else {
                    return ToolResult::error("auto_patch: no session source");
                };
                let hits = source.matches(from.as_str()).count();
                if hits != 1 {
                    return ToolResult::error(format!(
                        "auto_patch: `from` must occur exactly once (found {hits})"
                    ));
                }
                let at = source.find(&from).expect("checked") as u64;
                let report = ace_session_op(
                    &session,
                    AceSessionOp::Edit {
                        start: at,
                        end: at + from.len() as u64,
                        insert: to.as_bytes(),
                    },
                );
                report_tool_result(report, false)
            }
            other => ToolResult::error(format!("Unknown tool: {other}")),
        }
    }
}

fn str_field(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(|s| s.to_string())
}

fn session_only_tool(args: &Value, op: AceSessionOp<'_>) -> ToolResult {
    let Some(session) = str_field(args, "session") else {
        return ToolResult::error("this tool needs a session");
    };
    let report = ace_session_op(&session, op);
    report_tool_result(report, false)
}

fn report_tool_result(report: AceSessionReport, is_create: bool) -> ToolResult {
    let ok = matches!(report.outcome, AceSessionOutcome::Ok);
    let mut text = String::new();
    if is_create {
        text.push_str(&format!(
            "session {} (revision {})",
            report.session.clone().unwrap_or_default(),
            report.revision
        ));
    } else {
        text.push_str(&format!("revision {}", report.revision));
    }
    if let Some(display) = report.value.display() {
        text.push_str(&format!("\n{display}"));
    }
    if let Some(snapshot) = &report.snapshot {
        if report.eval_id.is_none() && snapshot != "running" {
            text.push_str(&format!("\nsource {snapshot}"));
        }
    }
    if !report.vars.is_empty() {
        let vars: Vec<String> = report
            .vars
            .iter()
            .map(|v| {
                format!(
                    "{} = {}",
                    v.name,
                    match v.value {
                        AceSessionValue::Int(n) => n.to_string(),
                        AceSessionValue::Bool(b) => b.to_string(),
                        AceSessionValue::None => "-".to_string(),
                    }
                )
            })
            .collect();
        text.push_str(&format!("\nvars: {}", vars.join(", ")));
    }
    if !ok {
        text.push_str(&format!("\n{:?}", report.outcome));
    }
    if ok {
        ToolResult::text(text)
    } else {
        ToolResult::error(text)
    }
}

fn tools() -> Value {
    json!([
        {"name": "auto_session_create", "description": "Create an explicit-ACE REPL session (optional seed source compiled as the session program)."},
        {"name": "auto_evaluate", "description": "Evaluate one REPL input (statements or one expression) in the session; failed evaluations keep the last accepted generation."},
        {"name": "auto_edit", "description": "Apply a byte-range same-PID incremental edit to the session program."},
        {"name": "auto_inspect", "description": "Inspect the session: revision, canonical source, variables."},
        {"name": "auto_vars", "description": "List the session's carried variables (int/bool scalar carry domain)."},
        {"name": "auto_typecheck", "description": "Compile-only probe of a source (throwaway session; no persistent state)."},
        {"name": "auto_patch", "description": "Text-replace patch on the session source (single occurrence), applied as a same-PID edit."},
        {"name": "auto_session_reset", "description": "Reset the session (fresh program, empty carry table)."},
        {"name": "auto_snapshot", "description": "Export the session's current user-visible program source."},
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: &str, params: Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(Id::Number(1)),
            method: method.into(),
            params: Some(params),
        }
    }

    /// p970: initialize answers THIS server's protocol version (the MCP
    /// negotiation the legacy server ignored).
    #[test]
    fn p970_mcp_initialize_negotiates() {
        let mut s = AceMcpServer::new();
        let resp = s
            .handle(&request(
                "initialize",
                json!({"protocolVersion": "2025-01-01"}),
            ))
            .expect("response");
        let version = resp
            .result
            .as_ref()
            .and_then(|v| v.get("protocolVersion"))
            .and_then(Value::as_str)
            .unwrap()
            .to_string();
        assert_eq!(version, "2024-11-05");
    }

    /// p970: the tool list keeps the legacy seven names and adds the
    /// session capability tools.
    #[test]
    fn p970_mcp_tool_list() {
        let mut s = AceMcpServer::new();
        let resp = s.handle(&request("tools/list", Value::Null)).expect("resp");
        let names: Vec<String> = resp
            .result
            .unwrap()
            .get("tools")
            .and_then(Value::as_array)
            .expect("tools array")
            .iter()
            .filter_map(|t| t.get("name").and_then(Value::as_str))
            .map(String::from)
            .collect();
        for legacy in [
            "auto_session_create",
            "auto_evaluate",
            "auto_session_reset",
            "auto_inspect",
            "auto_typecheck",
            "auto_patch",
            "auto_snapshot",
        ] {
            assert!(names.iter().any(|n| n == legacy), "missing {legacy}");
        }
        assert!(names.iter().any(|n| n == "auto_edit"));
        assert!(names.iter().any(|n| n == "auto_vars"));
    }

    /// p970: with no host installed the tools answer the NAMED host
    /// failure (never a panic, never a legacy route).
    use crate::ace_session::test_registry::REGISTRY_LOCK;

    #[test]
    fn p970_mcp_unregistered_host_is_tool_error() {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        crate::ace_session::shutdown_ace_session_host();
        let mut s = AceMcpServer::new();
        let resp = s
            .handle(&request(
                "tools/call",
                json!({"name": "auto_evaluate", "arguments": {"session": "ses_x", "code": "1"}}),
            ))
            .expect("response");
        let text = serde_json::to_string(&resp).unwrap();
        assert!(text.contains("host-unregistered"), "{text}");
    }

    /// p970: unknown methods keep the JSON-RPC -32601 discipline.
    #[test]
    fn p970_mcp_unknown_method() {
        let mut s = AceMcpServer::new();
        let resp = s
            .handle(&request("no/such", Value::Null))
            .expect("response");
        assert_eq!(resp.error.expect("error").code, -32601);
    }
}
