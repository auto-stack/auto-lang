//! # AutoUI MCP Server — Embedded in iced desktop process (Plan 278, Plan 299)
//!
//! Runs as a background thread inside the iced GUI process, providing MCP tools
//! for AI agents to inspect and manipulate the UI.
//!
//! ## Communication
//!
//! ```text
//! AI Agent (Claude Code)
//!     | HTTP POST /mcp (localhost:9247)
//!     v
//! McpUiServer (background thread in iced process)
//!     | via SharedState
//!     v
//! DynamicState → DynamicComponent → VmBridge
//! ```
//!
//! ## Transport
//!
//! Uses Streamable HTTP (Plan 299): axum HTTP server accepting JSON-RPC 2.0
//! over POST /mcp. Compatible with all standard MCP clients (Claude Code, Cursor, etc.).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

use serde_json::json;

use crate::aura::{AuraNode, AuraNodeId};
use crate::ui::debug::{BoxModel, ComputedNode, InspectorCache};
use crate::ui::debug_id_map::DebugIdMap;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::mcp_types::{ActionResult, UiActionType};
use crate::ui::snapshot_builder::SnapshotBuilder;
use crate::ui::vnode::{VTree, VNodeId};
use crate::ui::view::View;
use crate::ui::vtree_atom::{VTreeAtomBuilder, VTreeAtomOptions};

// ============================================================================
// Real-time styled VTree snapshot (Plan 314)
// ============================================================================

/// 序列化友好的单节点 computed 子集。
///
/// 由 F12 的 [`ComputedNode`] 提炼。全字段 `Option`/`Vec`——缺失即省略对应
/// Atom prop，node 仍输出（不变量：永不因缺数据 panic）。
#[derive(Debug, Clone, Default)]
pub struct ComputedNodeLite {
    /// 测量 border-box `(x, y, w, h)`——旧 `rect` 的超集/等价。
    pub bounds: Option<(f32, f32, f32, f32)>,
    /// 完整盒模型（content + padding + border + margin）。
    pub box_model: Option<BoxModel>,
    /// computed 样式 k/v（class 解析后）。
    pub computed_style: Vec<(String, String)>,
    /// 原始 `class` 字符串（便于 AI 对照源码）。
    pub raw_class: Option<String>,
    /// 事件绑定 `(event, handler)`。
    pub events: Vec<(String, String)>,
    /// 源码位置 `"app.at:42"`。
    pub source: Option<String>,
    /// for 循环上下文 `(var, index, value_repr)`。
    pub for_context: Option<(String, Option<usize>, String)>,
}

impl ComputedNodeLite {
    /// 从 F12 的 [`ComputedNode`] 提炼。
    pub fn from_computed(c: &ComputedNode) -> Self {
        Self {
            bounds: c.bounds.map(|r| (r.x, r.y, r.width, r.height)),
            box_model: c.box_model.clone(),
            computed_style: c.computed_style.clone(),
            raw_class: c.raw_class.clone(),
            events: c
                .events
                .iter()
                .map(|e| (e.event.clone(), e.handler.clone()))
                .collect(),
            source: c.source.clone(),
            for_context: c
                .for_context
                .as_ref()
                .map(|f| (f.var.clone(), f.index, f.value_repr.clone())),
        }
    }
}

/// 一帧的实时 VTree 快照（Plan 314）。
///
/// 由 iced 渲染器每帧（F12 开 或 MCP 激活 时）拷进 [`SharedState`]，供
/// `autoui_vtree` 工具序列化成 Atom。`VTree` + `InspectorCache` 是 VM 与 rust
/// 模式共有的数据形状（renderer.rs 的 `live_vtree`/`live_cache`），因此用
/// 一个自由组装函数即可复用，无需 trait 抽象。
#[derive(Debug, Clone)]
pub struct StyledNodeSnapshot {
    /// 顶层 widget 名（如 "NotesApp"）。
    pub widget_name: String,
    /// 实例级 VTree（path-based `VNodeId`，for 循环每次展开唯一）。
    pub vtree: VTree,
    /// 按 `VNodeId` 索引的 computed 子集。
    pub computed: HashMap<VNodeId, ComputedNodeLite>,
}

impl StyledNodeSnapshot {
    /// 从 live `VTree` + `InspectorCache` 组装快照。
    pub fn from_live(widget_name: &str, vtree: &VTree, cache: &InspectorCache) -> Self {
        let mut computed = HashMap::new();
        for id in cache.ids() {
            if let Some(c) = cache.get(id) {
                computed.insert(id, ComputedNodeLite::from_computed(c));
            }
        }
        Self {
            widget_name: widget_name.to_string(),
            vtree: vtree.clone(),
            computed,
        }
    }
}

// ============================================================================
// Shared State — Bridge between iced and MCP threads
// ============================================================================

/// Wrapper around `AuraNode` that is `Send` so it can live in the cross-thread
/// `SharedState` (axum state requires `Send + Sync`).
///
/// # Safety
/// `AuraNode` is not automatically `Send` because `ast::Expr` may contain
/// `ast::Node` whose type info uses `Rc<RefCell<_>>`. In practice the view
/// template stored here is built once by the parser, only ever **read** (to
/// render a text snapshot) while the `Mutex` is held, and replaced/dropped
/// exclusively from the iced thread via `SharedState::update`. The `Rc` handles
/// are never cloned or mutated across threads, so moving the owning value
/// between threads is sound.
struct SendViewTemplate(AuraNode);
unsafe impl Send for SendViewTemplate {}
unsafe impl Sync for SendViewTemplate {}

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
    view_template: Option<SendViewTemplate>,
    /// Window size (width, height) in logical pixels (Plan 281).
    window_size: Option<(f32, f32)>,
    /// Actual layout bounds from iced renderer (Plan 282).
    /// Key: widget ID like "aura_0", Value: (x, y, width, height)
    layout_bounds: HashMap<String, (f32, f32, f32, f32)>,
    /// Real-time styled VTree snapshot (Plan 314). Copied each frame by the
    /// iced renderer when F12 is open or MCP is active.
    styled_vtree: Option<StyledNodeSnapshot>,
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
            styled_vtree: None,
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

    /// Set the real-time styled VTree snapshot (Plan 314). Called each frame by
    /// the iced renderer when F12 is open or MCP is active.
    pub fn set_styled_vtree(&mut self, snap: StyledNodeSnapshot) {
        self.styled_vtree = Some(snap);
    }

    /// Take (move out) the latest styled VTree snapshot, if any (Plan 314).
    /// Leaves `None` behind so a stale frame is never served twice.
    pub fn take_styled_vtree(&mut self) -> Option<StyledNodeSnapshot> {
        self.styled_vtree.take()
    }

    /// Peek (clone) the latest styled VTree snapshot, if any (Plan 314).
    pub fn clone_styled_vtree(&self) -> Option<StyledNodeSnapshot> {
        self.styled_vtree.clone()
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
            self.view_template = view_template.map(SendViewTemplate);
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
/// Listens on HTTP port and serves MCP tool calls for UI inspection and manipulation.
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
    ///
    /// Uses Streamable HTTP transport (Plan 299): axum HTTP server
    /// accepting JSON-RPC 2.0 over POST /mcp.
    pub fn run(&self) {
        let shared = self.shared.clone();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            let app = axum::Router::new()
                .route("/mcp", axum::routing::post(mcp_http_handler))
                .with_state(shared);
            let addr = format!("127.0.0.1:{}", self.port);
            let listener = match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => {
                    eprintln!("AutoUI MCP: listening on http://{}", addr);
                    l
                }
                Err(e) => {
                    eprintln!("AutoUI MCP: failed to bind {}: {}", addr, e);
                    return;
                }
            };
            axum::serve(listener, app).await.unwrap();
        });
    }
}

/// Axum handler for POST /mcp — processes a single JSON-RPC request.
async fn mcp_http_handler(
    axum::extract::State(shared): axum::extract::State<SharedStateHandle>,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    let response = handle_request_static(&shared, request);
    axum::Json(response)
}

/// Static version of handle_request that takes SharedStateHandle directly.
fn handle_request_static(shared: &SharedStateHandle, req: serde_json::Value) -> serde_json::Value {
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
                        "version": "0.2.0"
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
                    "tools": tool_definitions()
                }
            })
        }
        "tools/call" => {
            let params = req.get("params").cloned().unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            let result = dispatch_tool_static(shared, tool_name, arguments);
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

// ============================================================================
// Tool Definitions (Plan 299: enhanced descriptions with workflow guidance)
// ============================================================================

fn tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "autoui_snapshot",
            "title": "UI Snapshot",
            "description": "Capture a structured snapshot of the current AutoUI page.\n\n## Workflow\n1. Call this tool first to understand what's on screen\n2. Identify element IDs (e.g., #aura_3) and their available actions\n3. Use autoui_action or autoui_type to interact with elements\n4. Call again to verify changes\n\nReturns the complete component hierarchy in AURA text format with widget states, element properties, and available interactions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "include_styles": {
                        "type": "boolean",
                        "default": false,
                        "description": "Include style/Tailwind class information for each element"
                    },
                    "include_state": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include full widget state (all state variable values)"
                    },
                    "include_status": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include render status annotations (FALLBACK/PARTIAL warnings)"
                    },
                    "include_bounds": {
                        "type": "boolean",
                        "default": false,
                        "description": "Include layout bounds (@rect x,y,w,h) for each element"
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
            "title": "Inspect Element",
            "description": "Inspect a specific UI element by its ID. Returns type, properties, current value, available actions, and source location.\n\n## Workflow\n1. Use autoui_snapshot to discover element IDs\n2. Call this with a specific element_id for detailed info",
            "inputSchema": {
                "type": "object",
                "required": ["element_id"],
                "properties": {
                    "element_id": {
                        "type": "string",
                        "description": "The element ID to inspect (e.g., 'aura_3')"
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
            "title": "Perform Action",
            "description": "Perform an action on a UI element.\n\n## Workflow\n1. Use autoui_snapshot to find element IDs and available actions\n2. Call this with element_id, action type, and optional value\n3. Use autoui_snapshot again to verify the result\n\n## Actions\n- press: Click a button\n- type_text: Type into an input/textarea (requires 'value')\n- toggle: Toggle a checkbox\n- select_option: Select from dropdown/radio (requires 'value')\n- set_value: Adjust a slider (requires numeric 'value')\n- clear: Clear an input/textarea",
            "inputSchema": {
                "type": "object",
                "required": ["element_id", "action"],
                "properties": {
                    "element_id": {
                        "type": "string",
                        "description": "Target element ID (e.g., 'aura_3')"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["press", "type_text", "toggle", "select_option", "set_value", "clear"],
                        "description": "Action to perform"
                    },
                    "value": {
                        "description": "Action parameter. For type_text: text string. For select_option: index or label. For set_value: number.",
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
            "title": "Render Check",
            "description": "Run a diagnostic check on the current UI. Detects rendering issues by comparing AURA source intent against iced backend capabilities.\n\n## When to use\n- When layout looks wrong\n- To verify all UI elements render correctly\n- After making changes to AURA source code",
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
            "title": "Take Screenshot",
            "description": "Capture a PNG screenshot of the current UI window. Returns the file path of the saved image.\n\n## When to use\n- To visually verify layouts and colors\n- To debug visual rendering issues\n- To confirm UI changes look correct",
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
        json!({
            "name": "autoui_state",
            "title": "Query State",
            "description": "Query the current widget state values. Returns all state variables with their types and current values.\n\n## Workflow\n1. Call without arguments to see all state fields\n2. Call with specific 'fields' to query only certain values\n3. Use after autoui_action to verify state changes",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "fields": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Specific state field names to query. If omitted, returns all fields."
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
            "name": "autoui_wait",
            "title": "Wait for State Change",
            "description": "Wait for a state field to change value. Polls at intervals until a change is detected or timeout.\n\n## Workflow\n1. Call with a state field name to watch\n2. The tool blocks until the field changes or timeout\n3. Returns the change details (before → after)",
            "inputSchema": {
                "type": "object",
                "required": ["field"],
                "properties": {
                    "field": {
                        "type": "string",
                        "description": "State field name to watch for changes"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "default": 5000,
                        "description": "Maximum time to wait in milliseconds"
                    },
                    "interval_ms": {
                        "type": "integer",
                        "default": 100,
                        "description": "Polling interval in milliseconds"
                    }
                }
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        json!({
            "name": "autoui_type",
            "title": "Type Text",
            "description": "Type text into an input element. Optionally clear existing text first.\n\n## Workflow\n1. Use autoui_snapshot to find the input element ID\n2. Call autoui_type with element_id and text\n3. Optionally set clear_first=true to erase existing content\n\nMore convenient than autoui_action type_text for form input.",
            "inputSchema": {
                "type": "object",
                "required": ["text"],
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to type"
                    },
                    "element_id": {
                        "type": "string",
                        "description": "Target input element ID. If omitted, uses the first focused input."
                    },
                    "clear_first": {
                        "type": "boolean",
                        "default": true,
                        "description": "Clear existing text before typing"
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
            "name": "autoui_keyboard",
            "title": "Send Key",
            "description": "Send a keyboard event (Enter, Tab, Escape, arrow keys, shortcuts).\n\n## When to use\n- Press Enter to submit a form\n- Press Tab to move focus\n- Press Escape to dismiss\n- Combine with modifiers for shortcuts (Ctrl+S, etc.)",
            "inputSchema": {
                "type": "object",
                "required": ["key"],
                "properties": {
                    "key": {
                        "type": "string",
                        "enum": ["Enter", "Tab", "Escape", "Backspace", "Delete", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"],
                        "description": "The key to press"
                    },
                    "modifiers": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["ctrl", "shift", "alt"] },
                        "description": "Modifier keys to hold"
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
            "name": "autoui_vtree",
            "title": "Live Styled VTree (Atom)",
            "description": "Return the live, post-render VTree serialized as Atom text. Each Atom node maps 1:1 to a rendered VNode: its name is the source widget keyword (col/row/button/center/text...), its id is the instance-level vnode_<n>, and its props carry the full box model (bbox + content/padding/border/margin insets), computed style, raw class, events, and source location.\n\n## When to use\nThis is the PRIMARY structural/perceptual channel for AutoUI — it shows the actually-rendered tree (for-loops expanded, geometry measured per-frame), NOT source code. Use it instead of a screenshot to perceive layout, structure, and style precisely. Pair with autoui_screenshot only for pixel-level verification.\n\n## Output\nAtom text: `col vnode_0 { bbox: {...}; style: {...}; class: \"...\"; button vnode_3 { label: \"OK\"; bbox: {...}; events: {...} } }`. Any field not measured yet (e.g. bounds before first layout) is omitted, never an error.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "scope": {
                        "type": "string",
                        "description": "Return only the subtree rooted at this node id (e.g. 'vnode_3'). Default: the whole tree."
                    },
                    "depth": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Maximum render depth relative to scope root. Deeper children collapse to a count. Default: unlimited."
                    },
                    "include_box": { "type": "boolean", "default": true, "description": "Include bbox + box model props" },
                    "include_style": { "type": "boolean", "default": true, "description": "Include computed style + class props" },
                    "include_events": { "type": "boolean", "default": true, "description": "Include events prop" },
                    "include_source": { "type": "boolean", "default": true, "description": "Include source + for_iter props" },
                    "include_props": { "type": "boolean", "default": true, "description": "Include widget props (content/label/value...)" }
                }
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
    ]
}

// ============================================================================
// Tool Dispatch (Plan 299: all tools as top-level functions)
// ============================================================================

fn dispatch_tool_static(shared: &SharedStateHandle, name: &str, args: serde_json::Value) -> serde_json::Value {
    match name {
        "autoui_snapshot" => tool_snapshot(shared, args),
        "autoui_inspect" => tool_inspect(shared, args),
        "autoui_action" => tool_action(shared, args),
        "autoui_check" => tool_check(shared, args),
        "autoui_screenshot" => tool_screenshot(shared, args),
        "autoui_state" => tool_state(shared, args),
        "autoui_wait" => tool_wait(shared, args),
        "autoui_type" => tool_type(shared, args),
        "autoui_keyboard" => tool_keyboard(shared, args),
        "autoui_vtree" => tool_vtree(shared, args),
        _ => error_result(format!("Unknown tool: {}", name)),
    }
}

// ── Tool: autoui_snapshot ──

fn tool_snapshot(shared: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let include_status = args
        .get("include_status")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let include_bounds = args
        .get("include_bounds")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let shared = shared.lock().unwrap();

    match &shared.view_template {
        Some(t) => {
            let template = &t.0;
            use crate::ui::aura_snapshot_builder::AuraSnapshotBuilder;
            let mut builder = AuraSnapshotBuilder::new(&shared.state).with_status(include_status);
            if let Some((w, h)) = shared.window_size {
                builder = builder.with_viewport(w, h);
            }
            if include_bounds {
                builder = builder.with_layout_bounds(shared.get_layout_bounds().clone());
            }
            let output = builder.build(&shared.widget_name, template);
            text_result(output)
        }
        None => error_result("No UI available yet — the application may not have rendered"),
    }
}

// ── Tool: autoui_inspect ──

fn tool_inspect(shared: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let element_id_str = match args.get("element_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return error_result("Missing required parameter: element_id"),
    };

    let element_id = match parse_aura_id(element_id_str) {
        Some(id) => id,
        None => return error_result(format!("Invalid element_id format: '{}' — expected 'aura_N'", element_id_str)),
    };

    let shared = shared.lock().unwrap();

    match &shared.view_template {
        Some(t) => {
            let template = &t.0;
            match find_aura_node(template, element_id) {
                Some((tag, props, events)) => {
                    use crate::ui::aura_snapshot_builder::AuraSnapshotBuilder;
                    let builder = AuraSnapshotBuilder::new(&shared.state);

                    let mut out = format!("Inspect #{}\n", element_id);
                    out.push_str(&format!("  tag: {}\n", tag));

                    out.push_str("  properties:\n");
                    for (key, prop_val) in props {
                        let val = builder.eval_prop_value(prop_val);
                        out.push_str(&format!("    {}: {}\n", key, val));
                    }

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

fn tool_action(shared_handle: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
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
        "clear" => UiActionType::Clear,
        _ => return error_result(format!("Unknown action: '{}'", action_str)),
    };

    let value = args.get("value").and_then(|v| json_value_to_auto_val(v));

    // Capture before-state and execute action
    let (before_state, result) = {
        let shared = shared_handle.lock().unwrap();

        let before_state = shared.state.clone();

        let (view, id_map) = match (&shared.view, &shared.id_map) {
            (Some(v), Some(m)) => (v, m),
            _ => return error_result("No UI available yet"),
        };

        let snapshot = SnapshotBuilder::build(
            &shared.widget_name,
            &shared.state,
            view,
            id_map,
        );

        let result = execute_action_on_shared(&shared, &snapshot.tree, element_id, action_type, value);
        (before_state, result)
    };

    match result {
        Ok(mut action_result) => {
            // Wait for state changes (Plan 299 Phase 3.4)
            let state_changes = wait_for_state_changes(shared_handle, &before_state, 500);
            action_result.state_changes = state_changes;
            text_result(action_result.to_aura_string())
        }
        Err(e) => error_result(e.to_string()),
    }
}

// ── Tool: autoui_check ──

fn tool_check(shared: &SharedStateHandle, _args: serde_json::Value) -> serde_json::Value {
    use crate::aura::{AuraNode, AuraNodeId};
    use crate::ui::render_support::{self, SupportLevel};

    let shared = shared.lock().unwrap();

    let template = match &shared.view_template {
        Some(t) => &t.0,
        None => return error_result("No UI available yet — the application may not have rendered"),
    };

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
                            if support.level == SupportLevel::Fallback || support.level == SupportLevel::Unsupported {
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

// ── Tool: autoui_screenshot ──

fn tool_screenshot(shared: &SharedStateHandle, _args: serde_json::Value) -> serde_json::Value {
    let rx = {
        let mut shared = shared.lock().unwrap();
        shared.request_screenshot()
    };

    match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(Ok(path)) => text_result(format!("Screenshot saved to: {}", path)),
        Ok(Err(e)) => error_result(format!("Screenshot failed: {}", e)),
        Err(_) => error_result("Screenshot timed out — iced thread may not be responding"),
    }
}

// ── Tool: autoui_state (Plan 299 Phase 2) ──

fn tool_state(shared: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let filter_fields: Option<Vec<String>> = args.get("fields")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

    let shared = shared.lock().unwrap();

    if shared.state.is_empty() {
        return text_result("No state available yet — the application may not have rendered".to_string());
    }

    let mut out = String::from("State:\n");
    let mut entries: Vec<_> = shared.state.iter().collect();
    entries.sort_by_key(|(k, _)| k.to_string());

    for (name, value) in &entries {
        if let Some(ref fields) = filter_fields {
            if !fields.contains(name) {
                continue;
            }
        }
        let type_str = match value {
            auto_val::Value::Int(_) => "int",
            auto_val::Value::Float(_) => "float",
            auto_val::Value::Bool(_) => "bool",
            auto_val::Value::Str(_) => "str",
            auto_val::Value::Null => "null",
            auto_val::Value::Array(_) => "list",
            auto_val::Value::Obj(_) => "object",
            _ => "unknown",
        };
        let val_str = match value {
            auto_val::Value::Str(s) => format!("{:?}", s),
            auto_val::Value::Float(f) => format!("{:.2}", f),
            other => other.to_string(),
        };
        out.push_str(&format!("  {}: {} ({})\n", name, val_str, type_str));
    }

    text_result(out)
}

// ── Tool: autoui_wait (Plan 299 Phase 2) ──

fn tool_wait(shared_handle: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let field = match args.get("field").and_then(|v| v.as_str()) {
        Some(f) => f.to_string(),
        None => return error_result("Missing required parameter: field"),
    };
    let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(5000);
    let interval_ms = args.get("interval_ms").and_then(|v| v.as_u64()).unwrap_or(100);

    // Capture initial value
    let before_val = {
        let shared = shared_handle.lock().unwrap();
        shared.state.get(&field).map(|v| format_auto_val(v))
    };

    let before_str = match &before_val {
        Some(v) => v.clone(),
        None => return error_result(format!("State field '{}' not found", field)),
    };

    // Poll until change or timeout
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    loop {
        std::thread::sleep(std::time::Duration::from_millis(interval_ms));

        let after_val = {
            let shared = shared_handle.lock().unwrap();
            shared.state.get(&field).map(|v| format_auto_val(v))
        };

        let after_str = match &after_val {
            Some(v) => v.clone(),
            None => return error_result(format!("State field '{}' disappeared", field)),
        };

        if after_str != before_str {
            return text_result(format!("State changed: {}.{} = {} -> {}", field, "", before_str, after_str));
        }

        if std::time::Instant::now() >= deadline {
            return error_result(format!("Timeout waiting for state change on '{}' (waited {}ms)", field, timeout_ms));
        }
    }
}

// ── Tool: autoui_type (Plan 299 Phase 3) ──

fn tool_type(shared_handle: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let text = match args.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return error_result("Missing required parameter: text"),
    };
    let element_id_opt = args.get("element_id").and_then(|v| v.as_str());
    let clear_first = args.get("clear_first").and_then(|v| v.as_bool()).unwrap_or(true);

    // Find the target input element
    let element_id = match element_id_opt {
        Some(id_str) => match parse_aura_id(id_str) {
            Some(id) => id,
            None => return error_result(format!("Invalid element_id format: '{}'", id_str)),
        },
        None => {
            // Find the first input element with a handler
            let shared = shared_handle.lock().unwrap();
            match &shared.view_template {
                Some(t) => match find_first_input(&t.0) {
                    Some(id) => id,
                    None => return error_result("No input element found — specify element_id"),
                },
                None => return error_result("No UI available yet"),
            }
        }
    };

    // If clear_first, send a clear action
    if clear_first {
        let clear_result = {
            let shared = shared_handle.lock().unwrap();
            let (view, id_map) = match (&shared.view, &shared.id_map) {
                (Some(v), Some(m)) => (v, m),
                _ => return error_result("No UI available yet"),
            };
            let snapshot = SnapshotBuilder::build(&shared.widget_name, &shared.state, view, id_map);
            execute_action_on_shared(&shared, &snapshot.tree, element_id, UiActionType::Clear, None)
        };
        if let Err(e) = clear_result {
            // Clear may not be supported on all elements, that's OK
            eprintln!("AutoUI MCP: clear before type failed: {}", e);
        }
    }

    // Send type_text action
    let result = {
        let shared = shared_handle.lock().unwrap();
        let (view, id_map) = match (&shared.view, &shared.id_map) {
            (Some(v), Some(m)) => (v, m),
            _ => return error_result("No UI available yet"),
        };
        let snapshot = SnapshotBuilder::build(&shared.widget_name, &shared.state, view, id_map);
        execute_action_on_shared(&shared, &snapshot.tree, element_id, UiActionType::TypeText, Some(auto_val::Value::str(&text)))
    };

    match result {
        Ok(action_result) => text_result(action_result.to_aura_string()),
        Err(e) => error_result(e.to_string()),
    }
}

// ── Tool: autoui_keyboard (Plan 299 Phase 3) ──

fn tool_keyboard(shared_handle: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let _key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return error_result("Missing required parameter: key"),
    };
    let _modifiers: Vec<String> = args.get("modifiers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    // For now, keyboard events are forwarded as an action message.
    // The iced renderer can intercept these via key_bindings in DynamicComponent.
    let shared = shared_handle.lock().unwrap();
    let widget_name = shared.widget_name.clone();

    // Map common keys to handler names
    let handler = format!("key_{}", _key.to_lowercase());

    let msg = ActionMessage {
        widget: widget_name,
        event: handler,
        input_value: Some(format!("{}{}", _modifiers.iter().map(|m| format!("{}+", m)).collect::<Vec<_>>().join(""), _key)),
    };

    match shared.send_action(msg) {
        Ok(()) => text_result(format!("Key sent: {}{}", _modifiers.iter().map(|m| format!("{}+", m)).collect::<Vec<_>>().join(""), _key)),
        Err(e) => error_result(format!("Failed to send key event: {}", e)),
    }
}

/// Find the first input element in the view template.
fn find_first_input(node: &crate::aura::AuraNode) -> Option<AuraNodeId> {
    match node {
        crate::aura::AuraNode::Element { tag, debug_id, children, .. } => {
            if tag == "input" || tag == "textarea" {
                return *debug_id;
            }
            for child in children {
                if let Some(id) = find_first_input(child) {
                    return Some(id);
                }
            }
            None
        }
        crate::aura::AuraNode::ForLoop { body, .. } => {
            for child in body {
                if let Some(id) = find_first_input(child) {
                    return Some(id);
                }
            }
            None
        }
        crate::aura::AuraNode::Conditional { then_body, else_body, .. } => {
            for child in then_body {
                if let Some(id) = find_first_input(child) {
                    return Some(id);
                }
            }
            if let Some(else_nodes) = else_body {
                for child in else_nodes {
                    if let Some(id) = find_first_input(child) {
                        return Some(id);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// Format an auto_val::Value for display.
fn format_auto_val(v: &auto_val::Value) -> String {
    match v {
        auto_val::Value::Str(s) => format!("{:?}", s),
        auto_val::Value::Float(f) => format!("{:.2}", f),
        other => other.to_string(),
    }
}

/// Wait for state changes after an action (Plan 299 Phase 3.4).
/// Polls SharedState for up to `timeout_ms`, comparing against before_state.
fn wait_for_state_changes(
    shared: &SharedStateHandle,
    before_state: &HashMap<String, auto_val::Value>,
    timeout_ms: u64,
) -> Vec<(String, String, String)> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let interval = std::time::Duration::from_millis(50);

    loop {
        std::thread::sleep(interval);
        let after_state = {
            let shared = shared.lock().unwrap();
            shared.state.clone()
        };

        let changes = compute_state_diff_static(before_state, &after_state);
        if !changes.is_empty() {
            return changes;
        }
        if std::time::Instant::now() >= deadline {
            return vec![];
        }
    }
}

/// Compute state diff between two state maps.
fn compute_state_diff_static(
    before: &HashMap<String, auto_val::Value>,
    after: &HashMap<String, auto_val::Value>,
) -> Vec<(String, String, String)> {
    let mut changes = Vec::new();
    for (key, after_val) in after {
        let before_val = before.get(key);
        let before_str = before_val.map_or("null".to_string(), |v| format_auto_val(v));
        let after_str = format_auto_val(after_val);
        if before_str != after_str {
            changes.push((key.clone(), before_str, after_str));
        }
    }
    changes
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
        UiActionType::Clear => {
            if target.kind != "Input" && target.kind != "Textarea" {
                return Err(format!("Action 'clear' not valid for component type '{}'", target.kind));
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
        UiActionType::Clear => "type", // Clear uses the same handler as type_text
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
// ── Tool: autoui_vtree (Plan 314) ──

/// Parse a `scope` argument into a `VNodeId`.
///
/// Accepts either `"vnode_<n>"` (the Atom id form) or a bare integer string.
/// Returns `None` if it cannot be parsed (the whole tree is returned instead).
fn parse_scope(raw: &str) -> Option<VNodeId> {
    let digits = raw.strip_prefix("vnode_").unwrap_or(raw);
    digits.parse::<u64>().ok().map(VNodeId::new)
}

fn tool_vtree(shared: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let opts = VTreeAtomOptions {
        scope: args
            .get("scope")
            .and_then(|v| v.as_str())
            .and_then(parse_scope),
        depth: args
            .get("depth")
            .and_then(|v| v.as_i64())
            .map(|n| n.max(0) as usize),
        include_box: args.get("include_box").and_then(|v| v.as_bool()).unwrap_or(true),
        include_style: args
            .get("include_style")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        include_events: args
            .get("include_events")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        include_source: args
            .get("include_source")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        include_props: args
            .get("include_props")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    };

    // Peek (clone) the latest frame so repeated calls and concurrent readers
    // both work; never consume the snapshot.
    let snap = shared.lock().unwrap().clone_styled_vtree();
    match snap {
        Some(snap) => {
            let atom = VTreeAtomBuilder::build(&snap, &opts).to_string();
            text_result(atom)
        }
        None => error_result(
            "No live VTree snapshot yet — the UI has not rendered a frame with \
             DevTools/MCP capture active. Retry after the window has painted.",
        ),
    }
}

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

#[cfg(test)]
mod tests_314 {
    use super::*;
    use crate::ui::debug::{ComputedNode, InspectorCache, Rect};
    use crate::ui::vnode::{VNode, VNodeKind, VNodeProps};

    fn build_sample_tree() -> (VTree, [VNodeId; 3]) {
        let mut tree = VTree::new();
        // root: Column (id 0)
        let root = VNode::new(VNodeId::new(0), VNodeKind::Column, VNodeProps::Layout { spacing: 8, padding: 4 });
        tree.set_root(root);
        // child: Text (id 1)
        let text = VNode::new(VNodeId::new(1), VNodeKind::Text, VNodeProps::Text { content: "Hello".into() });
        tree.add_node(text);
        tree.get_mut(VNodeId::new(0)).unwrap().add_child(VNodeId::new(1));
        // child: Button (id 2)
        let btn = VNode::new(VNodeId::new(2), VNodeKind::Button, VNodeProps::Button { label: "OK".into() });
        tree.add_node(btn);
        tree.get_mut(VNodeId::new(0)).unwrap().add_child(VNodeId::new(2));
        (tree, [VNodeId::new(0), VNodeId::new(1), VNodeId::new(2)])
    }

    fn fill_cache(ids: [VNodeId; 3]) -> InspectorCache {
        let mut cache = InspectorCache::new();
        // root: bounds only
        let r = cache.get_mut_or_default(ids[0]);
        r.bounds = Some(Rect { x: 0.0, y: 0.0, width: 100.0, height: 50.0 });
        // button: bounds + style + event
        let b = cache.get_mut_or_default(ids[2]);
        b.bounds = Some(Rect { x: 40.0, y: 10.0, width: 60.0, height: 30.0 });
        b.computed_style.push(("color".into(), "#ffffff".into()));
        b.events.push(crate::ui::debug::EventHandlerInfo { event: "press".into(), handler: ".Ok".into() });
        b.raw_class = Some("btn".into());
        cache
    }

    #[test]
    fn styled_snapshot_from_live_copies_computed_subset() {
        let (tree, ids) = build_sample_tree();
        let cache = fill_cache(ids);
        let snap = StyledNodeSnapshot::from_live("Demo", &tree, &cache);

        assert_eq!(snap.widget_name, "Demo");
        assert_eq!(snap.vtree.node_count(), 3);

        // root: bounds copied, no style/event
        let r = snap.computed.get(&ids[0]).expect("root computed present");
        assert_eq!(r.bounds, Some((0.0, 0.0, 100.0, 50.0)));
        assert!(r.computed_style.is_empty() && r.events.is_empty());

        // text (id 1): no entry in cache → absent from map (degrades gracefully)
        assert!(!snap.computed.contains_key(&ids[1]));

        // button: full subset
        let b = snap.computed.get(&ids[2]).expect("button computed present");
        assert_eq!(b.bounds, Some((40.0, 10.0, 60.0, 30.0)));
        assert_eq!(b.computed_style, vec![("color".to_string(), "#ffffff".to_string())]);
        assert_eq!(b.events, vec![("press".to_string(), ".Ok".to_string())]);
        assert_eq!(b.raw_class.as_deref(), Some("btn"));
    }

    #[test]
    fn computed_lite_from_empty_computed_is_all_none() {
        let empty = ComputedNode::default();
        let lite = ComputedNodeLite::from_computed(&empty);
        assert!(lite.bounds.is_none() && lite.box_model.is_none());
        assert!(lite.computed_style.is_empty() && lite.events.is_empty());
    }

    /// Build a SharedStateHandle carrying a sample styled snapshot.
    fn shared_with_snapshot() -> SharedStateHandle {
        let (tree, ids) = build_sample_tree();
        let cache = fill_cache(ids);
        let snap = StyledNodeSnapshot::from_live("Demo", &tree, &cache);
        let mut state = SharedState::new("Demo".into());
        state.set_styled_vtree(snap);
        Arc::new(Mutex::new(state))
    }

    #[test]
    fn tool_vtree_returns_atom_text_for_full_tree() {
        let shared = shared_with_snapshot();
        let res = dispatch_tool_static(&shared, "autoui_vtree", json!({}));
        let text = res["content"][0]["text"].as_str().expect("text content");
        // widget keyword names + vnode ids
        assert!(text.contains("col vnode_0"), "root: {text}");
        assert!(text.contains("text vnode_1"), "text child: {text}");
        assert!(text.contains("button vnode_2"), "button child: {text}");
        // widget props + computed props present by default
        assert!(text.contains("content:") && text.contains("label:"), "props: {text}");
        assert!(text.contains("bbox:") && text.contains("style:"), "computed: {text}");
        assert!(!res["isError"].as_bool().unwrap_or(true), "not an error: {text}");
    }

    #[test]
    fn tool_vtree_scope_returns_subtree_only() {
        let shared = shared_with_snapshot();
        let res = dispatch_tool_static(&shared, "autoui_vtree", json!({ "scope": "vnode_2" }));
        let text = res["content"][0]["text"].as_str().expect("text content");
        assert!(text.contains("button vnode_2"), "rooted at button: {text}");
        assert!(!text.contains("col vnode_0"), "root excluded: {text}");
    }

    #[test]
    fn tool_vtree_respects_include_flags() {
        let shared = shared_with_snapshot();
        let res = dispatch_tool_static(
            &shared,
            "autoui_vtree",
            json!({ "include_props": false, "include_box": false, "include_style": false }),
        );
        let text = res["content"][0]["text"].as_str().expect("text content");
        assert!(!text.contains("label:"), "no widget props: {text}");
        assert!(!text.contains("bbox:"), "no bbox: {text}");
        assert!(!text.contains("style:"), "no style: {text}");
    }

    #[test]
    fn tool_vtree_errors_when_no_snapshot() {
        let shared: SharedStateHandle = Arc::new(Mutex::new(SharedState::new("Demo".into())));
        let res = dispatch_tool_static(&shared, "autoui_vtree", json!({}));
        assert!(res["isError"].as_bool().unwrap_or(false), "should error: {res}");
    }
}
