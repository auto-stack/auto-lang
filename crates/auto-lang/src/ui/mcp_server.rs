//! # AutoUI MCP Server — Embedded in iced desktop process (Plan 278)
//!
//! Runs as a background thread inside the iced GUI process, providing MCP tools
//! for AI agents to inspect and manipulate the UI.
//!
//! ## Communication
//!
//! ```text
//! AI Agent (Claude Code)
//!     | TCP (localhost:9247)
//!     v
//! McpUiServer (background thread in iced process)
//!     | via SharedState
//!     v
//! DynamicState → DynamicComponent → VmBridge
//! ```
//!
//! ## Transport
//!
//! Uses a simple TCP server accepting JSON-RPC 2.0 over line-delimited JSON.
//! This is a lightweight alternative to full Streamable HTTP — sufficient for
//! a local single-client scenario.

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

use serde_json::json;

use crate::aura::{AuraNode, AuraNodeId};
use crate::ui::debug_id_map::DebugIdMap;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::mcp_types::{ActionResult, UiActionType};
use crate::ui::snapshot_builder::SnapshotBuilder;
use crate::ui::view::View;

// ============================================================================
// Shared State — Bridge between iced and MCP threads
// ============================================================================

/// Shared state that the iced main thread updates and the MCP thread reads.
///
/// The iced thread holds `SharedState` and updates the snapshot after each
/// render. The MCP thread reads from it and sends action requests back.
pub struct SharedState {
    /// Latest view tree from iced render (used for action routing).
    view: Option<View<DynamicMessage>>,
    /// Latest DebugIdMap from iced render (used for action routing).
    id_map: Option<DebugIdMap>,
    /// Current state values.
    state: HashMap<String, auto_val::Value>,
    /// Widget name.
    widget_name: String,
    /// Input-to-state field mapping.
    input_state_map: HashMap<String, String>,
    /// Channel to inject IcedMessages into the iced event loop.
    /// MCP thread sends, iced subscription receives.
    action_tx: Option<mpsc::Sender<ActionMessage>>,
    /// Original AuraNode view template (Plan 279).
    /// Used for AURA source-style snapshots with full original info.
    view_template: Option<AuraNode>,
    /// Window size (width, height) in logical pixels (Plan 281).
    window_size: Option<(f32, f32)>,
    /// Actual layout bounds from iced renderer (Plan 282).
    /// Key: widget ID like "aura_0", Value: (x, y, width, height)
    layout_bounds: HashMap<String, (f32, f32, f32, f32)>,
    /// Pending screenshot request from MCP thread (Plan 285).
    screenshot_request: Option<ScreenshotRequest>,
}

/// Screenshot request stored in SharedState for the iced thread to pick up (Plan 285).
pub struct ScreenshotRequest {
    pub reply_tx: std::sync::mpsc::Sender<Result<String, String>>,
}

/// A message sent from MCP thread to iced event loop to simulate user actions.
#[derive(Debug)]
pub struct ActionMessage {
    /// Widget name (e.g., "App")
    pub widget: String,
    /// Event name (e.g., "InputChanged", "AddTodo")
    pub event: String,
    /// Input text value (for type_text actions)
    pub input_value: Option<String>,
}

impl SharedState {
    pub fn new(widget_name: String) -> Self {
        Self {
            view: None,
            id_map: None,
            state: HashMap::new(),
            widget_name,
            input_state_map: HashMap::new(),
            action_tx: None,
            view_template: None,
            window_size: None,
            layout_bounds: HashMap::new(),
            screenshot_request: None,
        }
    }

    /// Check whether a view has been pushed yet.
    pub fn has_view(&self) -> bool {
        self.view.is_some()
    }

    /// Set the action sender channel (called once at startup).
    pub fn set_action_tx(&mut self, tx: mpsc::Sender<ActionMessage>) {
        self.action_tx = Some(tx);
    }

    /// Try to send an action message to the iced event loop.
    pub fn send_action(&self, msg: ActionMessage) -> Result<(), String> {
        match &self.action_tx {
            Some(tx) => tx.send(msg).map_err(|e| format!("Channel send error: {}", e)),
            None => Err("No action channel available".to_string()),
        }
    }

    /// Set the window size (Plan 281).
    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_size = Some((width, height));
    }

    /// Set layout bounds from iced renderer (Plan 282).
    pub fn set_layout_bounds(&mut self, bounds: HashMap<String, (f32, f32, f32, f32)>) {
        self.layout_bounds = bounds;
    }

    /// Get layout bounds (Plan 282).
    pub fn get_layout_bounds(&self) -> &HashMap<String, (f32, f32, f32, f32)> {
        &self.layout_bounds
    }

    /// Request a screenshot capture. Returns a Receiver that will receive the
    /// file path once the iced thread processes the request (Plan 285).
    pub fn request_screenshot(&mut self) -> std::sync::mpsc::Receiver<Result<String, String>> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.screenshot_request = Some(ScreenshotRequest { reply_tx: tx });
        rx
    }

    /// Take and clear the pending screenshot request (called by iced thread) (Plan 285).
    pub fn take_screenshot_request(&mut self) -> Option<ScreenshotRequest> {
        self.screenshot_request.take()
    }

    /// Update the shared state with a new view tree and state values.
    /// Called by the iced thread after each render.
    pub fn update(
        &mut self,
        view: View<DynamicMessage>,
        id_map: DebugIdMap,
        state: HashMap<String, auto_val::Value>,
        input_state_map: HashMap<String, String>,
        view_template: Option<AuraNode>,
    ) {
        self.view = Some(view);
        self.id_map = Some(id_map);
        self.state = state;
        self.input_state_map = input_state_map;
        if view_template.is_some() {
            self.view_template = view_template;
        }
    }
}

/// Thread-safe handle to the shared state.
pub type SharedStateHandle = Arc<Mutex<SharedState>>;

/// Action request sent from MCP thread to iced thread.
pub enum ActionRequest {
    /// Call a handler by event name.
    CallHandler {
        event_name: String,
        args: Vec<auto_val::Value>,
    },
    /// Write a state field.
    WriteState {
        field: String,
        value: auto_val::Value,
    },
}

// ============================================================================
// MCP UI Server
// ============================================================================

/// MCP server that runs inside the iced process.
///
/// Listens on a TCP port and serves MCP tool calls for UI inspection and manipulation.
pub struct McpUiServer {
    shared: SharedStateHandle,
    port: u16,
}

impl McpUiServer {
    /// Create a new MCP UI server.
    pub fn new(shared: SharedStateHandle, port: u16) -> Self {
        Self { shared, port }
    }

    /// Start the MCP server in the current thread (blocking).
    /// Intended to be called from a spawned background thread.
    pub fn run(&self) {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => {
                eprintln!("AutoUI MCP: listening on {}", addr);
                l
            }
            Err(e) => {
                eprintln!("AutoUI MCP: failed to bind {}: {}", addr, e);
                return;
            }
        };

        // Accept a single client (one AI agent at a time)
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    eprintln!("AutoUI MCP: client connected");
                    self.handle_client(&mut stream);
                    eprintln!("AutoUI MCP: client disconnected");
                }
                Err(e) => {
                    eprintln!("AutoUI MCP: accept error: {}", e);
                }
            }
        }
    }

    fn handle_client(&self, stream: &mut std::net::TcpStream) {
        let reader = std::io::BufReader::new(stream.try_clone().unwrap());
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let request: serde_json::Value = match serde_json::from_str(trimmed) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("AutoUI MCP: parse error: {}", e);
                            continue;
                        }
                    };

                    let response = self.handle_request(request);
                    let response_json = match serde_json::to_string(&response) {
                        Ok(j) => j,
                        Err(e) => {
                            eprintln!("AutoUI MCP: serialize error: {}", e);
                            continue;
                        }
                    };

                    if let Err(e) = stream.write_all(format!("{}\n", response_json).as_bytes()) {
                        eprintln!("AutoUI MCP: write error: {}", e);
                        break;
                    }
                    let _ = stream.flush();
                }
                Err(e) => {
                    eprintln!("AutoUI MCP: read error: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_request(&self, req: serde_json::Value) -> serde_json::Value {
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = req.get("id").cloned();

        match method {
            "initialize" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": { "tools": {} },
                        "serverInfo": {
                            "name": "autoui",
                            "version": "0.1.0"
                        }
                    }
                })
            }
            "notifications/initialized" => {
                json!({ "jsonrpc": "2.0", "id": null, "result": {} })
            }
            "ping" => {
                json!({ "jsonrpc": "2.0", "id": id, "result": {} })
            }
            "tools/list" => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": self.tool_definitions()
                    }
                })
            }
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(json!({}));
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                let result = self.dispatch_tool(tool_name, arguments);
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result
                })
            }
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": format!("Method not found: {}", method) }
                })
            }
        }
    }

    fn tool_definitions(&self) -> Vec<serde_json::Value> {
        vec![
            json!({
                "name": "autoui_snapshot",
                "description": "Capture a structured snapshot of the current AutoUI page. Returns the complete component hierarchy with all widget states, element properties, and available interactions in AURA text format. Use this to understand what is on screen before performing actions. Render status annotations (FALLBACK/PARTIAL) show which features the iced backend does not fully support.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "include_styles": {
                            "type": "boolean",
                            "default": false,
                            "description": "Whether to include style information for each element"
                        },
                        "include_state": {
                            "type": "boolean",
                            "default": true,
                            "description": "Whether to include the full widget state (all state variable values)"
                        },
                        "include_status": {
                            "type": "boolean",
                            "default": true,
                            "description": "Whether to include render status annotations (FALLBACK/PARTIAL warnings for unsupported features)"
                        }
                    }
                },
                "annotations": {
                    "readOnlyHint": true,
                    "destructiveHint": false,
                    "idempotentHint": true,
                    "openWorldHint": false
                }
            }),
            json!({
                "name": "autoui_inspect",
                "description": "Inspect a specific UI element by its AuraNodeId. Returns detailed information about that element: its type, properties, current value, available actions, and source location. Use autoui_snapshot first to discover element IDs.",
                "inputSchema": {
                    "type": "object",
                    "required": ["element_id"],
                    "properties": {
                        "element_id": {
                            "type": "string",
                            "description": "The AuraNodeId of the element to inspect (e.g., 'aura_3'). Obtain IDs from autoui_snapshot."
                        }
                    }
                },
                "annotations": {
                    "readOnlyHint": true,
                    "destructiveHint": false,
                    "idempotentHint": true,
                    "openWorldHint": false
                }
            }),
            json!({
                "name": "autoui_action",
                "description": "Perform an action on a UI element. Actions include pressing buttons, typing text into inputs, toggling checkboxes, selecting options, and adjusting sliders. Use autoui_snapshot first to discover element IDs and their available actions.",
                "inputSchema": {
                    "type": "object",
                    "required": ["element_id", "action"],
                    "properties": {
                        "element_id": {
                            "type": "string",
                            "description": "The AuraNodeId of the target element (e.g., 'aura_3')."
                        },
                        "action": {
                            "type": "string",
                            "enum": ["press", "type_text", "toggle", "select_option", "set_value"],
                            "description": "The action to perform. 'press' for buttons. 'type_text' for input/textarea. 'toggle' for checkboxes. 'select_option' for select dropdowns. 'set_value' for sliders."
                        },
                        "value": {
                            "description": "Action parameter. For type_text: the text string. For select_option: the option index (integer) or option label (string). For set_value: a number.",
                            "type": ["string", "number", "integer", "null"]
                        }
                    }
                },
                "annotations": {
                    "readOnlyHint": false,
                    "destructiveHint": false,
                    "idempotentHint": false,
                    "openWorldHint": false
                }
            }),
            json!({
                "name": "autoui_check",
                "description": "Run a diagnostic check on the current UI to detect rendering issues. Compares the AURA source intent against what the iced backend actually supports. Reports unsupported tags (e.g., grid), partial support (e.g., missing props), and suggests fixes. Use this to validate that the UI renders correctly.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                },
                "annotations": {
                    "readOnlyHint": true,
                    "destructiveHint": false,
                    "idempotentHint": true,
                    "openWorldHint": false
                }
            }),
            json!({
                "name": "autoui_screenshot",
                "description": "Capture a PNG screenshot of the current UI window. Returns the absolute file path of the saved PNG file. Use this to visually inspect the rendered UI, verify layouts, check colors, and debug visual issues. The screenshot is saved in the tmp/ directory.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                },
                "annotations": {
                    "readOnlyHint": true,
                    "destructiveHint": false,
                    "idempotentHint": false,
                    "openWorldHint": false
                }
            }),
        ]
    }

    fn dispatch_tool(&self, name: &str, args: serde_json::Value) -> serde_json::Value {
        match name {
            "autoui_snapshot" => self.tool_snapshot(args),
            "autoui_inspect" => self.tool_inspect(args),
            "autoui_action" => self.tool_action(args),
            "autoui_check" => self.tool_check(args),
            "autoui_screenshot" => self.tool_screenshot(args),
            _ => error_result(format!("Unknown tool: {}", name)),
        }
    }

    // ── Tool: autoui_snapshot ──

    fn tool_snapshot(&self, args: serde_json::Value) -> serde_json::Value {
        let include_status = args
            .get("include_status")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let shared = self.shared.lock().unwrap();

        match &shared.view_template {
            Some(template) => {
                use crate::ui::aura_snapshot_builder::AuraSnapshotBuilder;
                let mut builder = AuraSnapshotBuilder::new(&shared.state).with_status(include_status);
                if let Some((w, h)) = shared.window_size {
                    builder = builder.with_viewport(w, h);
                }
                builder = builder.with_layout_bounds(shared.get_layout_bounds().clone());
                let output = builder.build(&shared.widget_name, template);
                text_result(output)
            }
            None => error_result("No UI available yet — the application may not have rendered"),
        }
    }

    // ── Tool: autoui_inspect ──

    fn tool_inspect(&self, args: serde_json::Value) -> serde_json::Value {
        let element_id_str = match args.get("element_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return error_result("Missing required parameter: element_id"),
        };

        let element_id = match parse_aura_id(element_id_str) {
            Some(id) => id,
            None => return error_result(format!("Invalid element_id format: '{}' — expected 'aura_N'", element_id_str)),
        };

        let shared = self.shared.lock().unwrap();

        match &shared.view_template {
            Some(template) => {
                // Find the AuraNode by debug_id
                match find_aura_node(template, element_id) {
                    Some((tag, props, events)) => {
                        use crate::ui::aura_snapshot_builder::AuraSnapshotBuilder;
                        let builder = AuraSnapshotBuilder::new(&shared.state);

                        let mut out = format!("Inspect #{}\n", element_id);
                        out.push_str(&format!("  tag: {}\n", tag));

                        // Props
                        out.push_str("  properties:\n");
                        for (key, prop_val) in props {
                            let val = builder.eval_prop_value(prop_val);
                            out.push_str(&format!("    {}: {}\n", key, val));
                        }

                        // Events
                        if !events.is_empty() {
                            out.push_str("  events:\n");
                            for (event_name, aura_event) in events {
                                out.push_str(&format!("    {} -> {}\n", event_name, aura_event.handler));
                            }
                        }

                        text_result(out)
                    }
                    None => error_result(format!("Element not found: #{}", element_id)),
                }
            }
            None => error_result("No UI available yet"),
        }
    }

    // ── Tool: autoui_action ──

    fn tool_action(&self, args: serde_json::Value) -> serde_json::Value {
        let element_id_str = match args.get("element_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return error_result("Missing required parameter: element_id"),
        };

        let action_str = match args.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return error_result("Missing required parameter: action"),
        };

        let element_id = match parse_aura_id(element_id_str) {
            Some(id) => id,
            None => return error_result(format!("Invalid element_id format: '{}'", element_id_str)),
        };

        let action_type = match action_str {
            "press" => UiActionType::Press,
            "type_text" => UiActionType::TypeText,
            "toggle" => UiActionType::Toggle,
            "select_option" => UiActionType::SelectOption,
            "set_value" => UiActionType::SetValue,
            _ => return error_result(format!("Unknown action: '{}'", action_str)),
        };

        let value = args.get("value").and_then(|v| json_value_to_auto_val(v));

        // Execute the action with shared state (read-only — action goes through channel)
        let shared = self.shared.lock().unwrap();

        let (view, id_map) = match (&shared.view, &shared.id_map) {
            (Some(v), Some(m)) => (v, m),
            _ => return error_result("No UI available yet"),
        };

        // Build snapshot for node lookup
        let snapshot = SnapshotBuilder::build(
            &shared.widget_name,
            &shared.state,
            view,
            id_map,
        );

        // Execute the action — sends ActionMessage through channel to iced
        let result = execute_action_on_shared(&shared, &snapshot.tree, element_id, action_type, value);

        match result {
            Ok(action_result) => text_result(action_result.to_aura_string()),
            Err(e) => error_result(e.to_string()),
        }
    }

    // ── Tool: autoui_check (Plan 280) ──

    fn tool_check(&self, _args: serde_json::Value) -> serde_json::Value {
        use crate::aura::{AuraNode, AuraNodeId};
        use crate::ui::render_support::{self, SupportLevel};

        let shared = self.shared.lock().unwrap();

        let template = match &shared.view_template {
            Some(t) => t,
            None => return error_result("No UI available yet — the application may not have rendered"),
        };

        // Walk the AuraNode tree and collect issues
        struct Issue {
            id: Option<AuraNodeId>,
            tag: String,
            level: SupportLevel,
            note: String,
            ignored_props: Vec<String>,
        }

        fn collect_issues(node: &AuraNode, issues: &mut Vec<Issue>) {
            match node {
                AuraNode::Element { tag, props, children, debug_id, .. } => {
                    let support = render_support::get_support(tag);
                    if support.level != SupportLevel::Full {
                        let ignored: Vec<String> = props.keys()
                            .filter(|k| {
                                // For fallback/unsupported tags, all props are ignored
                                if support.level == SupportLevel::Fallback || support.level == SupportLevel::Unsupported {
                                    // style and class are always relevant
                                    !matches!(k.as_str(), "style" | "class")
                                        || support.ignored_props.contains(&k.as_str())
                                } else {
                                    support.ignored_props.contains(&k.as_str())
                                }
                            })
                            .cloned()
                            .collect();

                        issues.push(Issue {
                            id: *debug_id,
                            tag: tag.clone(),
                            level: support.level,
                            note: support.note.to_string(),
                            ignored_props: ignored,
                        });
                    }
                    for child in children {
                        collect_issues(child, issues);
                    }
                }
                AuraNode::ForLoop { body, .. } => {
                    for child in body {
                        collect_issues(child, issues);
                    }
                }
                AuraNode::Conditional { then_body, else_body, .. } => {
                    for child in then_body {
                        collect_issues(child, issues);
                    }
                    if let Some(else_nodes) = else_body {
                        for child in else_nodes {
                            collect_issues(child, issues);
                        }
                    }
                }
                _ => {}
            }
        }

        let mut issues: Vec<Issue> = Vec::new();
        collect_issues(template, &mut issues);

        // Count total elements for summary
        fn count_elements(node: &AuraNode) -> usize {
            match node {
                AuraNode::Element { children, .. } => {
                    1 + children.iter().map(count_elements).sum::<usize>()
                }
                AuraNode::ForLoop { body, .. } => {
                    body.iter().map(count_elements).sum()
                }
                AuraNode::Conditional { then_body, else_body, .. } => {
                    let mut count: usize = then_body.iter().map(count_elements).sum();
                    if let Some(else_nodes) = else_body {
                        count += else_nodes.iter().map(count_elements).sum::<usize>();
                    }
                    count
                }
                _ => 1,
            }
        }

        let total_elements = count_elements(template);
        let error_count = issues.iter().filter(|i| i.level == SupportLevel::Fallback || i.level == SupportLevel::Unsupported).count();
        let warn_count = issues.iter().filter(|i| i.level == SupportLevel::Partial).count();
        let ok_count = total_elements - issues.len();

        // Format output
        let mut out = String::new();
        out.push_str("AutoUI Render Check\n");
        out.push_str(&format!("widget: \"{}\"\n\n", shared.widget_name));

        if issues.is_empty() {
            out.push_str("No issues found — all elements fully supported.\n");
        } else {
            out.push_str(&format!("Issues found: {} errors, {} warnings\n\n", error_count, warn_count));

            for issue in &issues {
                let id_str = issue.id.map(|id| format!("#{}", id)).unwrap_or_default();
                let level_str = match issue.level {
                    SupportLevel::Fallback | SupportLevel::Unsupported => "ERROR",
                    SupportLevel::Partial => "WARN",
                    SupportLevel::Full => unreachable!(),
                };
                out.push_str(&format!("[{}] {} {} — {:?}\n", level_str, id_str, issue.tag, issue.level));
                out.push_str(&format!("  {}\n", issue.note));
                if !issue.ignored_props.is_empty() {
                    out.push_str(&format!("  Ignored props: {}\n", issue.ignored_props.join(", ")));
                }
                out.push('\n');
            }
        }

        out.push_str(&format!("Summary: {} errors, {} warnings, {} OK elements ({} total)\n",
            error_count, warn_count, ok_count, total_elements));

        text_result(out)
    }

    // ── Tool: autoui_screenshot (Plan 285) ──

    fn tool_screenshot(&self, _args: serde_json::Value) -> serde_json::Value {
        // Store a screenshot request in SharedState and wait for the iced thread
        // to pick it up, capture the window, and reply with the file path.
        let rx = {
            let mut shared = self.shared.lock().unwrap();
            shared.request_screenshot()
        };
        // Lock released — iced thread can now pick up the request.

        match rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(Ok(path)) => text_result(format!("Screenshot saved to: {}", path)),
            Ok(Err(e)) => error_result(format!("Screenshot failed: {}", e)),
            Err(_) => error_result("Screenshot timed out — iced thread may not be responding"),
        }
    }
}

// ============================================================================
// Action Execution on SharedState
// ============================================================================

/// Execute an action by sending an ActionMessage through the channel to the
/// iced event loop. This simulates real user interaction — the iced update
/// handler runs with full state mutation, animations, and UI refresh.
fn execute_action_on_shared(
    shared: &SharedState,
    tree: &crate::ui::mcp_types::UiNode,
    element_id: AuraNodeId,
    action: UiActionType,
    value: Option<auto_val::Value>,
) -> Result<ActionResult, String> {
    // Find target node
    let target = SnapshotBuilder::find_node(tree, element_id)
        .ok_or_else(|| format!("Element not found: #{}", element_id))?;

    // Validate action type
    match &action {
        UiActionType::Press => {
            if target.kind != "Button" {
                return Err(format!("Action 'press' not valid for component type '{}'", target.kind));
            }
        }
        UiActionType::TypeText => {
            if target.kind != "Input" && target.kind != "Textarea" {
                return Err(format!("Action 'type_text' not valid for component type '{}'", target.kind));
            }
        }
        UiActionType::Toggle => {
            if target.kind != "Checkbox" {
                return Err(format!("Action 'toggle' not valid for component type '{}'", target.kind));
            }
        }
        UiActionType::SelectOption => {
            if target.kind != "Select" && target.kind != "Radio" {
                return Err(format!("Action 'select_option' not valid for component type '{}'", target.kind));
            }
        }
        UiActionType::SetValue => {
            if target.kind != "Slider" {
                return Err(format!("Action 'set_value' not valid for component type '{}'", target.kind));
            }
        }
    }

    // Find handler from actions list
    let action_name = match &action {
        UiActionType::Press => "press",
        UiActionType::TypeText => "type",
        UiActionType::Toggle => "toggle",
        UiActionType::SelectOption => "select",
        UiActionType::SetValue => "set_value",
    };

    let handler = target.actions.iter()
        .find(|a| a.name == action_name)
        .map(|a| a.handler.trim_start_matches('.').to_string())
        .ok_or_else(|| format!("No '{}' handler found on element #{}", action_name, element_id))?;

    // Build the ActionMessage to inject into iced event loop
    let input_value = match &action {
        UiActionType::TypeText => {
            Some(value.as_ref()
                .map(|v| match v {
                    auto_val::Value::Str(s) => s.to_string(),
                    other => other.to_string(),
                })
                .ok_or_else(|| "Action 'type_text' requires a value parameter".to_string())?)
        }
        _ => None,
    };

    let msg = ActionMessage {
        widget: shared.widget_name.clone(),
        event: handler.clone(),
        input_value,
    };

    // Send through the channel — iced subscription will pick it up
    shared.send_action(msg)?;

    Ok(ActionResult {
        status: "ok".to_string(),
        element_id,
        action: action.to_string(),
        handler: Some(format!(".{}", handler)),
        state_changes: vec![], // Real state changes happen in iced update, visible on next snapshot
    })
}

// ============================================================================
// Helpers
// ============================================================================

/// Parse an AuraNodeId from string format "aura_N".
fn parse_aura_id(s: &str) -> Option<AuraNodeId> {
    s.strip_prefix("aura_")
        .and_then(|n| n.parse::<u32>().ok())
        .map(AuraNodeId)
}

/// Convert a JSON value to an Auto Value.
fn json_value_to_auto_val(v: &serde_json::Value) -> Option<auto_val::Value> {
    match v {
        serde_json::Value::String(s) => Some(auto_val::Value::str(s)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(auto_val::Value::Int(i as i32))
            } else if let Some(f) = n.as_f64() {
                Some(auto_val::Value::Float(f))
            } else {
                None
            }
        }
        serde_json::Value::Bool(b) => Some(auto_val::Value::Bool(*b)),
        serde_json::Value::Null => Some(auto_val::Value::Null),
        _ => None,
    }
}

/// Create a MCP tool result with text content.
fn text_result(text: String) -> serde_json::Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": false
    })
}

/// Create a MCP tool error result.
fn error_result(msg: impl Into<String>) -> serde_json::Value {
    let msg = msg.into();
    json!({
        "content": [{ "type": "text", "text": format!("Error: {}", msg) }],
        "isError": true
    })
}

// ============================================================================
// Server Startup
// ============================================================================

/// Start the MCP UI server in a background thread.
///
/// Returns a `SharedStateHandle` that the iced main thread should use
/// to update the view tree and state after each render.
///
/// # Arguments
///
/// * `widget_name` — The name of the main widget
/// * `port` — TCP port to listen on (default: 9247)
pub fn start_mcp_server(widget_name: String, port: u16) -> (SharedStateHandle, mpsc::Receiver<ActionMessage>) {
    let (action_tx, action_rx) = mpsc::channel::<ActionMessage>();

    let mut shared_state = SharedState::new(widget_name);
    shared_state.set_action_tx(action_tx);
    let shared = Arc::new(Mutex::new(shared_state));

    let server = McpUiServer::new(shared.clone(), port);

    std::thread::spawn(move || {
        server.run();
    });

    (shared, action_rx)
}

/// Get the MCP port from environment variable or use default.
pub fn mcp_port() -> u16 {
    std::env::var("AUTOUI_MCP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9247)
}

/// Find an AuraNode by AuraNodeId, returning its tag, props, and events.
fn find_aura_node<'a>(
    node: &'a crate::aura::AuraNode,
    target_id: AuraNodeId,
) -> Option<(&'a str, &'a std::collections::HashMap<String, crate::aura::AuraPropValue>, &'a std::collections::HashMap<String, crate::aura::AuraEvent>)> {
    match node {
        crate::aura::AuraNode::Element { tag, props, events, children, debug_id, .. } => {
            if let Some(id) = debug_id {
                if *id == target_id {
                    return Some((tag.as_str(), props, events));
                }
            }
            for child in children {
                if let Some(result) = find_aura_node(child, target_id) {
                    return Some(result);
                }
            }
            None
        }
        crate::aura::AuraNode::ForLoop { body, debug_id, .. } => {
            if let Some(id) = debug_id {
                if *id == target_id {
                    return None; // ForLoop itself is not inspectable as an element
                }
            }
            for child in body {
                if let Some(result) = find_aura_node(child, target_id) {
                    return Some(result);
                }
            }
            None
        }
        crate::aura::AuraNode::Conditional { then_body, else_body, .. } => {
            for child in then_body {
                if let Some(result) = find_aura_node(child, target_id) {
                    return Some(result);
                }
            }
            if let Some(else_nodes) = else_body {
                for child in else_nodes {
                    if let Some(result) = find_aura_node(child, target_id) {
                        return Some(result);
                    }
                }
            }
            None
        }
        _ => None,
    }
}
