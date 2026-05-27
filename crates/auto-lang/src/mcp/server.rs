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
            ToolDefinition {
                name: "auto_typecheck".into(),
                description: "Validate Auto code syntax without executing it. Returns parse errors, defined symbols, and import information. Useful for AI agents to verify code before execution.".into(),
                input_schema: json!({
                    "type": "object",
                    "required": ["code"],
                    "properties": {
                        "code": { "type": "string", "description": "Auto code to validate" }
                    }
                }),
            },
            ToolDefinition {
                name: "auto_patch".into(),
                description: "Replace a single definition (fn, type, enum) in an existing session. Rebuilds the session from accumulated source with the patched definition. Returns rebuild status.".into(),
                input_schema: json!({
                    "type": "object",
                    "required": ["session_id", "old_name", "new_code"],
                    "properties": {
                        "session_id": { "type": "string" },
                        "old_name": { "type": "string", "description": "Name of the definition to replace" },
                        "new_code": { "type": "string", "description": "Complete new definition code" }
                    }
                }),
            },
            ToolDefinition {
                name: "auto_snapshot".into(),
                description: "Export all accumulated source code in a session as a single .at file. Useful for persisting session state or sharing with others.".into(),
                input_schema: json!({
                    "type": "object",
                    "required": ["session_id"],
                    "properties": {
                        "session_id": { "type": "string" }
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
            "auto_typecheck" => self.tool_typecheck(args),
            "auto_patch" => self.tool_patch(args),
            "auto_snapshot" => self.tool_snapshot(args),
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

                // Record successful source for patch/snapshot
                self.sessions.append_source(&session_id, &code);

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

    // ── Phase 2: auto_typecheck ──

    fn tool_typecheck(&self, args: serde_json::Value) -> ToolResult {
        let code = match args.get("code").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => return ToolResult::error("Missing required parameter: code"),
        };

        match crate::parse_preserve_error(&code) {
            Ok(ast) => {
                let mut symbols = Vec::new();
                let mut imports = Vec::new();
                for stmt in &ast.stmts {
                    match stmt {
                        crate::ast::Stmt::Fn(f) => {
                            symbols.push(json!({
                                "kind": "function",
                                "name": f.name.to_string(),
                                "params": f.params.len(),
                                "return_type": f.ret.to_string()
                            }));
                        }
                        crate::ast::Stmt::TypeDecl(t) => {
                            symbols.push(json!({
                                "kind": "type",
                                "name": t.name.to_string(),
                                "fields": t.members.len()
                            }));
                        }
                        crate::ast::Stmt::EnumDecl(e) => {
                            symbols.push(json!({
                                "kind": "enum",
                                "name": e.name.to_string(),
                                "variants": e.items.len()
                            }));
                        }
                        crate::ast::Stmt::Use(u) => {
                            let module = if let Some(ref mp) = u.module_path {
                                mp.to_string()
                            } else {
                                u.paths.join("::")
                            };
                            imports.push(module);
                        }
                        _ => {}
                    }
                }

                ToolResult::text(serde_json::to_string(&json!({
                    "status": "ok",
                    "valid": true,
                    "symbols": symbols,
                    "imports": imports,
                    "diagnostics": []
                })).unwrap())
            }
            Err(e) => {
                ToolResult::text(serde_json::to_string(&json!({
                    "status": "error",
                    "valid": false,
                    "symbols": [],
                    "imports": [],
                    "diagnostics": [{
                        "severity": "error",
                        "message": format!("{}", e)
                    }]
                })).unwrap())
            }
        }
    }

    // ── Phase 3: auto_patch ──

    fn tool_patch(&mut self, args: serde_json::Value) -> ToolResult {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return ToolResult::error("Missing required parameter: session_id"),
        };
        let old_name = match args.get("old_name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => return ToolResult::error("Missing required parameter: old_name"),
        };
        let new_code = match args.get("new_code").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => return ToolResult::error("Missing required parameter: new_code"),
        };

        if !self.sessions.exists(&session_id) {
            return ToolResult::error(format!("Session not found: {}", session_id));
        }

        // Get current source
        let source = match self.sessions.get_source(&session_id) {
            Some(s) => s,
            None => return ToolResult::error("No source history for session"),
        };

        // Find and replace the definition block
        let patched = patch_replace_definition(&source, &old_name, &new_code);

        // Validate the patched source parses correctly
        if let Err(e) = crate::parse_preserve_error(&patched) {
            return ToolResult::text(serde_json::to_string(&json!({
                "status": "error",
                "message": "Patched code has syntax errors",
                "diagnostics": [{"severity": "error", "message": format!("{}", e)}]
            })).unwrap());
        }

        // Rebuild session with patched source
        self.sessions.rebuild_with_source(&session_id, &patched);

        // Re-execute the patched source
        let session = match self.sessions.get(&session_id) {
            Some(s) => s,
            None => return ToolResult::error("Session lost during rebuild"),
        };

        match session.run(&patched) {
            Ok(output) => ToolResult::text(serde_json::to_string(&json!({
                "status": "ok",
                "message": format!("Patched '{}' and rebuilt session", old_name),
                "output": output,
                "diagnostics": []
            })).unwrap()),
            Err(e) => ToolResult::text(serde_json::to_string(&json!({
                "status": "error",
                "message": "Patch parsed OK but execution failed",
                "output": null,
                "diagnostics": [{"severity": "error", "message": format!("{}", e)}]
            })).unwrap()),
        }
    }

    // ── Phase 3: auto_snapshot ──

    fn tool_snapshot(&self, args: serde_json::Value) -> ToolResult {
        let session_id = match args.get("session_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return ToolResult::error("Missing required parameter: session_id"),
        };

        match self.sessions.get_source(&session_id) {
            Some(source) => {
                if source.is_empty() {
                    return ToolResult::text(serde_json::to_string(&json!({
                        "status": "ok",
                        "source": "",
                        "lines": 0,
                        "message": "Session is empty (no code executed yet)"
                    })).unwrap());
                }

                let lines = source.lines().count();
                ToolResult::text(serde_json::to_string(&json!({
                    "status": "ok",
                    "source": source,
                    "lines": lines
                })).unwrap())
            }
            None => ToolResult::error(format!("Session not found: {}", session_id)),
        }
    }
}

/// Replace a top-level definition named `old_name` with `new_code` in source.
/// Finds the definition by matching `fn old_name`, `type old_name`, or `enum old_name`
/// at the start of a line, then replaces the entire block (up to matching braces).
fn patch_replace_definition(source: &str, old_name: &str, new_code: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let keywords = ["fn", "type", "enum", "spec", "ext"];

    let mut start_line = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        for kw in &keywords {
            let prefix = format!("{} {}", kw, old_name);
            if trimmed.starts_with(&prefix) {
                // Check that old_name is followed by a word boundary
                let after = &trimmed[prefix.len()..];
                if after.is_empty() || after.starts_with('(') || after.starts_with(' ') || after.starts_with('{') || after.starts_with('<') {
                    start_line = Some(i);
                    break;
                }
            }
        }
        if start_line.is_some() {
            break;
        }
    }

    let start = match start_line {
        Some(i) => i,
        None => {
            // Definition not found — append new code at end
            return format!("{}\n\n{}", source, new_code);
        }
    };

    // Find end of definition: count braces from start line
    let mut depth = 0i32;
    let mut end_line = start;
    let mut found_open = false;
    for i in start..lines.len() {
        for ch in lines[i].chars() {
            match ch {
                '{' => { depth += 1; found_open = true; }
                '}' => { depth -= 1; }
                _ => {}
            }
        }
        if found_open && depth <= 0 {
            end_line = i;
            break;
        }
        // Single-line definitions without braces (e.g., `type Alias = X`)
        if !found_open && i > start {
            // Next definition starts
            break;
        }
    }

    // Rebuild: lines before start + new_code + lines after end
    let mut result = String::new();
    for line in &lines[..start] {
        result.push_str(line);
        result.push('\n');
    }
    result.push_str(new_code);
    result.push('\n');
    for line in &lines[end_line + 1..] {
        result.push_str(line);
        result.push('\n');
    }
    result.trim_end().to_string()
}
