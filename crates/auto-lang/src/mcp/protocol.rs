// MCP Protocol Types — JSON-RPC 2.0 over stdio
//
// Minimal implementation of MCP (Model Context Protocol) for AI agent interaction.
// Only implements the subset needed: initialize, tools/list, tools/call, notifications.

use serde::{Deserialize, Serialize};

// ── JSON-RPC 2.0 ──

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String, // always "2.0"
    #[serde(default)]
    pub id: Option<Id>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Id {
    Number(i64),
    String(String),
}

impl JsonRpcResponse {
    pub fn success(id: Option<Id>, result: serde_json::Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }

    pub fn error(id: Option<Id>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(), id,
            result: None,
            error: Some(JsonRpcError { code, message: message.into(), data: None }),
        }
    }
}

// ── MCP Protocol Types ──

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub client_info: Option<ClientInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    pub tools: ToolCapabilities,
}

#[derive(Debug, Serialize)]
pub struct ToolCapabilities {
    // Empty for now — just signals that we support tools
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

impl ToolResult {
    pub fn text(text: impl Into<String>) -> Self {
        Self { content: vec![ContentBlock::Text { text: text.into() }], is_error: None }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self { content: vec![ContentBlock::Text { text: text.into() }], is_error: Some(true) }
    }
}

// ── Stdio Transport ──

/// Read one JSON-RPC message from stdin (line-delimited).
pub fn read_message() -> Option<JsonRpcRequest> {
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(0) => None, // EOF
        Ok(_) => {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return read_message(); // skip blank lines
            }
            match serde_json::from_str(trimmed) {
                Ok(req) => Some(req),
                Err(e) => {
                    eprintln!("MCP: failed to parse request: {} — input: {}", e, trimmed);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("MCP: stdin read error: {}", e);
            None
        }
    }
}

/// Write one JSON-RPC response to stdout.
pub fn write_response(resp: &JsonRpcResponse) {
    match serde_json::to_string(resp) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("MCP: failed to serialize response: {}", e),
    }
}
