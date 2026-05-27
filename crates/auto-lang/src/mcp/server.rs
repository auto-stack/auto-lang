// MCP Server — JSON-RPC router + tool dispatch
//
// Reads JSON-RPC requests from stdin, dispatches to tool handlers,
// writes responses to stdout. Implements MCP protocol handshake.

use super::protocol::*;
use super::session_manager::SessionManager;
use serde_json::json;

pub struct McpServer {
    sessions: SessionManager,
    initialized: bool,
}

impl McpServer {
    pub fn new() -> Self {
        Self { sessions: SessionManager::new(), initialized: false }
    }

    /// Run the MCP server loop. Reads from stdin, writes to stdout.
    pub fn run(&mut self) {
        eprintln!("MCP: AutoVM MCP server starting (stdio transport)");
        while let Some(req) = read_message() {
            let response = self.handle_request(req);
            write_response(&response);
            // Flush stdout to ensure the client receives the response
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
        eprintln!("MCP: server shutting down (stdin closed)");
    }

    fn handle_request(&mut self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            // ── MCP Protocol Lifecycle ──
            "initialize" => {
                self.initialized = true;
                JsonRpcResponse::success(req.id, serde_json::to_value(InitializeResult {
                    protocol_version: "2024-11-05".into(),
                    capabilities: ServerCapabilities {
                        tools: ToolCapabilities {},
                    },
                    server_info: ServerInfo {
                        name: "autovm".into(),
                        version: "0.1.0".into(),
                    },
                }).unwrap())
            }
            "notifications/initialized" => {
                // Client confirms initialization — no response needed for notifications
                JsonRpcResponse { jsonrpc: "2.0".into(), id: None, result: Some(json!({})), error: None }
            }
            "ping" => {
                JsonRpcResponse::success(req.id, json!({}))
            }

            // ── Tool Discovery ──
            "tools/list" => {
                JsonRpcResponse::success(req.id, json!({
                    "tools": self.tool_definitions()
                }))
            }

            // ── Tool Execution ──
            "tools/call" => {
                let params: ToolCallParams = match serde_json::from_value(
                    req.params.unwrap_or(json!({}))
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        return JsonRpcResponse::success(req.id, serde_json::to_value(
                            ToolResult::error(format!("Invalid tool call params: {}", e))
                        ).unwrap());
                    }
                };
                let result = self.dispatch_tool(&params.name, params.arguments);
                JsonRpcResponse::success(req.id, serde_json::to_value(result).unwrap())
            }

            // ── Unknown Method ──
            _ => JsonRpcResponse::error(req.id, -32601, format!("Method not found: {}", req.method)),
        }
    }

    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "auto_session_create".into(),
                description: "Create a new AutoVM session with isolated execution state. Returns session_id for use in subsequent calls.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "sandbox": { "type": "boolean", "default": false, "description": "Enable sandbox mode (no file I/O, no network)" }
                    }
                }),
            },
            ToolDefinition {
                name: "auto_evaluate".into(),
                description: "Execute Auto code in the session. Can define functions/types, evaluate expressions, or run statements. Returns the result value, type, and any diagnostics.".into(),
                input_schema: json!({
                    "type": "object",
                    "required": ["session_id", "code"],
                    "properties": {
                        "session_id": { "type": "string" },
                        "code": { "type": "string", "description": "Auto code to execute" }
                    }
                }),
            },
            ToolDefinition {
                name: "auto_session_reset".into(),
                description: "Reset an AutoVM session to clean state, or delete it entirely.".into(),
                input_schema: json!({
                    "type": "object",
                    "required": ["session_id"],
                    "properties": {
                        "session_id": { "type": "string" },
                        "action": { "type": "string", "enum": ["reset", "delete"], "default": "reset" }
                    }
                }),
            },
            ToolDefinition {
                name: "auto_inspect".into(),
                description: "Query the current state of an AutoVM session. Returns defined functions, types, variables, and their signatures/values.".into(),
                input_schema: json!({
                    "type": "object",
                    "required": ["session_id"],
                    "properties": {
                        "session_id": { "type": "string" },
                        "kind": { "type": "string", "enum": ["functions", "variables", "all"], "default": "all" }
                    }
                }),
            },
        ]
    }

    pub fn dispatch_tool(&mut self, name: &str, args: serde_json::Value) -> ToolResult {
        match name {
            "auto_session_create" => self.tool_session_create(args),
            "auto_evaluate" => self.tool_evaluate(args),
            "auto_session_reset" => self.tool_session_reset(args),
            "auto_inspect" => self.tool_inspect(args),
            _ => ToolResult::error(format!("Unknown tool: {}", name)),
        }
    }

    // ── Tool Implementations ──

    fn tool_session_create(&mut self, args: serde_json::Value) -> ToolResult {
        let sandbox = args.get("sandbox").and_then(|v| v.as_bool()).unwrap_or(false);
        let session_id = self.sessions.create(sandbox);
        ToolResult::text(serde_json::to_string(&json!({
            "session_id": session_id,
            "status": "created"
        })).unwrap())
    }

    fn tool_evaluate(&mut self, args: serde_json::Value) -> ToolResult {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return ToolResult::error("Missing required parameter: session_id"),
        };
        let code = match args.get("code").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => return ToolResult::error("Missing required parameter: code"),
        };

        let session = match self.sessions.get(&session_id) {
            Some(s) => s,
            None => return ToolResult::error(format!("Session not found: {}", session_id)),
        };

        match session.run(&code) {
            Ok(output) => {
                let result_value = session.format_last_result()
                    .or_else(|| session.get_last_result().map(|v| v.to_string()));

                ToolResult::text(serde_json::to_string(&json!({
                    "status": "ok",
                    "output": output,
                    "value": result_value,
                    "diagnostics": []
                })).unwrap())
            }
            Err(e) => {
                let err_str = format!("{}", e);
                ToolResult::text(serde_json::to_string(&json!({
                    "status": "error",
                    "output": null,
                    "value": null,
                    "diagnostics": [{
                        "severity": "error",
                        "message": err_str
                    }]
                })).unwrap())
            }
        }
    }

    fn tool_session_reset(&mut self, args: serde_json::Value) -> ToolResult {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return ToolResult::error("Missing required parameter: session_id"),
        };
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("reset");

        match action {
            "delete" => {
                if self.sessions.delete(&session_id) {
                    ToolResult::text(serde_json::to_string(&json!({
                        "status": "deleted",
                        "session_id": session_id
                    })).unwrap())
                } else {
                    ToolResult::error(format!("Session not found: {}", session_id))
                }
            }
            _ => { // "reset"
                if self.sessions.reset(&session_id) {
                    ToolResult::text(serde_json::to_string(&json!({
                        "status": "reset",
                        "session_id": session_id
                    })).unwrap())
                } else {
                    ToolResult::error(format!("Session not found: {}", session_id))
                }
            }
        }
    }

    fn tool_inspect(&mut self, args: serde_json::Value) -> ToolResult {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return ToolResult::error("Missing required parameter: session_id"),
        };
        let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("all");

        let session = match self.sessions.get(&session_id) {
            Some(s) => s,
            None => return ToolResult::error(format!("Session not found: {}", session_id)),
        };

        let stats = session.stats();
        let mut result = json!({
            "session_id": session_id,
            "stats": {
                "bytecode_size": stats.bytecode_size,
                "heap_objects": stats.heap_objects,
                "arrays": stats.arrays
            }
        });

        if kind == "functions" || kind == "all" {
            result["functions"] = serde_json::to_value(
                session.functions().into_iter().map(|f| json!({"name": f})).collect::<Vec<_>>()
            ).unwrap();
        }
        if kind == "variables" || kind == "all" {
            result["variables"] = serde_json::to_value(
                session.locals().into_iter().map(|v| json!({"name": v})).collect::<Vec<_>>()
            ).unwrap();
        }

        ToolResult::text(serde_json::to_string(&result).unwrap())
    }
}
