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
use crate::ui::vnode::{VNode, VNodeProps, VTree, VNodeId};
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
    /// EDGE-01: key bindings (including element-attribute onkeydown.*) for
    /// tool_keyboard lookup. Set by renderer each view() via update().
    key_bindings: HashMap<String, String>,
}

/// Screenshot request stored in SharedState for the iced thread to pick up (Plan 285).
///
/// Plan 371 Task 20: optional visual-regression options. When `diff` is set,
/// the iced thread compares the captured PNG against a baseline (looked up by
/// `name` under `tests/screenshots/`) and returns a structured diff result
/// instead of just the file path.
pub struct ScreenshotRequest {
    pub reply_tx: std::sync::mpsc::Sender<Result<String, String>>,
    /// Stable identifier for the screenshot (used as the baseline filename).
    /// Empty → legacy timestamped behavior.
    pub name: String,
    /// If true, write the capture to `tests/screenshots/<name>.png` (overwrite).
    pub baseline: bool,
    /// If true, compare the capture against `tests/screenshots/<name>.png`.
    pub diff: bool,
    /// Allowed fraction of differing pixels (0.0–1.0). Above this → "DIFFERS".
    pub threshold: f64,
}

impl Default for ScreenshotRequest {
    fn default() -> Self {
        Self {
            reply_tx: std::sync::mpsc::channel().0,
            name: String::new(),
            baseline: false,
            diff: false,
            threshold: 0.01,
        }
    }
}

/// Plan 371 Task 20: parsed screenshot options passed from `tool_screenshot`
/// to `SharedState::request_screenshot`. Pure data (no `image` types) so it
/// can live in this non-ui-iced-gated module.
pub struct ScreenshotOptions {
    pub name: String,
    pub baseline: bool,
    pub diff: bool,
    pub threshold: f64,
}

/// A message sent from MCP thread to iced event loop to simulate user actions.
///
/// Self-describing (Plan 371 Task 19): `target` selects the addressing mode.
#[derive(Debug, Clone)]
pub struct ActionMessage {
    pub target: ActionTarget,
    pub action: UiActionType,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ActionTarget {
    Event { widget: String, event: String },
    Path { path: Vec<u16> },
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
            key_bindings: HashMap::new(),
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
    pub fn request_screenshot(
        &mut self,
        opts: ScreenshotOptions,
    ) -> std::sync::mpsc::Receiver<Result<String, String>> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.screenshot_request = Some(ScreenshotRequest {
            reply_tx: tx,
            name: opts.name,
            baseline: opts.baseline,
            diff: opts.diff,
            threshold: opts.threshold,
        });
        rx
    }

    /// Take and clear the pending screenshot request (called by iced thread) (Plan 285).
    pub fn take_screenshot_request(&mut self) -> Option<ScreenshotRequest> {
        self.screenshot_request.take()
    }

    /// Replace the state map (Plan 371 Task 21). Used by rust mode, where the
    /// DevTools layer pushes a `Component::state_snapshot()` each frame (there
    /// is no VM heap to read from). VM mode uses [`SharedState::update`].
    pub fn set_state(&mut self, state: HashMap<String, auto_val::Value>) {
        self.state = state;
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
        key_bindings: HashMap<String, String>,
    ) {
        self.view = Some(view);
        self.id_map = Some(id_map);
        self.state = state;
        self.input_state_map = input_state_map;
        self.key_bindings = key_bindings;
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
                        "enum": ["press", "type_text", "submit", "toggle", "select_option", "set_value", "clear"],
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
            "description": "Capture a PNG screenshot of the current UI window, optionally saving a named baseline or comparing against one (Plan 371 Task 20).\n\n## Modes\n- Default: save a timestamped PNG to tmp/ and return its path.\n- baseline=true (requires 'name'): save to tests/screenshots/<name>.png (overwrite).\n- diff=true (requires 'name'): compare against tests/screenshots/<name>.png and return matches/DIFFERS with the diff percentage.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Baseline/screenshot identifier (e.g. '015-notes/initial'). Required when baseline or diff is true."
                    },
                    "baseline": {
                        "type": "boolean",
                        "default": false,
                        "description": "Save the capture as the baseline tests/screenshots/<name>.png (overwrites)"
                    },
                    "diff": {
                        "type": "boolean",
                        "default": false,
                        "description": "Compare the capture against the baseline tests/screenshots/<name>.png"
                    },
                    "threshold": {
                        "type": "number",
                        "default": 0.01,
                        "description": "Allowed fraction of differing pixels (0.0-1.0). Above this the diff reports DIFFERS."
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
            "title": "Wait for Change",
            "description": "Wait for a state field change OR an element to appear/disappear. Polls at intervals until the condition is met or timeout.\n\n## Modes\n1. State wait: pass `field` — blocks until the state field changes value.\n2. Element wait (Plan 371): pass `kind` + `label` + `condition` — blocks until a matching element appears or disappears.\n\n## Element wait examples\n- Wait for Save button to appear after clicking Edit:\n  {kind: 'button', label: 'Save', condition: 'appears'}\n- Wait for a loading spinner to disappear:\n  {kind: 'text', label: 'Loading', condition: 'disappears'}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "field": {
                        "type": "string",
                        "description": "State field name to watch for changes (state wait mode)"
                    },
                    "kind": {
                        "type": "string",
                        "description": "Element kind to wait for (element wait mode): button, input, text, textarea..."
                    },
                    "label": {
                        "type": "string",
                        "description": "Case-insensitive substring to match on element label/content (element wait mode)"
                    },
                    "condition": {
                        "type": "string",
                        "enum": ["appears", "disappears"],
                        "default": "appears",
                        "description": "Whether to wait for the element to appear or disappear"
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
                        "enum": ["Enter", "Tab", "Escape", "Backspace", "Delete", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown", "F12"],
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
        // Plan 371 Task 7: search VTree for matching nodes
        json!({
            "name": "autoui_find",
            "title": "Find Widgets in VTree",
            "description": "Search the live rendered VTree for nodes matching given criteria. Returns each match as an Atom-format ancestor-chain subtree (root → ... → matched node), showing the matched node's position in the UI hierarchy. Use autoui_exists for a faster concise summary when you only need 'does it exist?'.\n\n## When to use\n- Understand WHERE a widget sits in the UI hierarchy\n- Find all buttons/inputs matching a label, with structural context\n- Debug UI structure after an action\n\n## Search criteria (all optional, combined with AND)\n- kind: node type (button, input, text, textarea, col, row, checkbox...)\n- label: substring match on label/content/value/placeholder\n- limit: max results (default 20)\n\n## Output\nAtom subtrees showing the ancestor chain for each match, e.g.:\ncol vnode_0 { ... col vnode_5 { ... row vnode_8 { button vnode_10 {label: \"Edit\"} } } }",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "kind": {
                        "type": "string",
                        "description": "Filter by node kind (button, input, text, textarea, col, row, checkbox, slider, image...)"
                    },
                    "label": {
                        "type": "string",
                        "description": "Substring match on the node's label, content, value, or placeholder text (case-insensitive)"
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "default": 20,
                        "description": "Maximum number of matching nodes to return"
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
        // Plan 371: quick existence check (concise summary, no Atom subtree)
        json!({
            "name": "autoui_exists",
            "title": "Check Widget Exists",
            "description": "Quick existence check — search the VTree and return a concise FOUND/NOT FOUND summary with match count and IDs. Faster than autoui_find when you only need to verify 'does this element exist?'\n\n## When to use\n- After an action, verify a widget appeared (e.g. 'is there a Save button now?')\n- Before an action, verify a widget is present\n- Assert-based test validation\n\n## Output\n'FOUND N match(es): kind=X, label~=Y\\n  button \"Save\"; button \"Cancel\"' or 'NOT FOUND (0 matches): ...'",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "kind": {
                        "type": "string",
                        "description": "Filter by node kind (button, input, text, textarea, col, row...)"
                    },
                    "label": {
                        "type": "string",
                        "description": "Case-insensitive substring match on label/content/value/placeholder"
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
        // Plan 403: press a sequence of buttons by label, return final state.
        json!({
            "name": "autoui_press_sequence",
            "title": "Press Button Sequence",
            "description": "Press a sequence of calculator/UI buttons by their label text, then return the resulting application state. Each key is matched to a rendered button by exact label (e.g. \"2\", \"+\", \"=\"). This is the expression-evaluation-via-MCP interface: send [\"2\",\"+\",\"3\",\"=\"] and read the computed result from the returned state — the math happens through real button presses, not direct calculation.\n\n## When to use\n- Drive a calculator: [\"5\",\"*\",\"3\",\"=\"] → state shows display=15\n- Automate any keypad/button-sequence interaction\n- End-to-end verification that button wiring + handlers work\n\n## Parameters\n- keys (required): array of button label strings\n- delay_ms (optional): ms to wait between presses (default 50)\n- state_fields (optional): array of field names to filter the returned state to\n\n## Output\n'Pressed: [2, +, 3, =]\\n\\nState:\\n  display: 5 (int)\\n  ...'",
            "inputSchema": {
                "type": "object",
                "required": ["keys"],
                "properties": {
                    "keys": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Button labels to press in order (e.g. [\"2\",\"+\",\"3\",\"=\"])"
                    },
                    "delay_ms": {
                        "type": "integer",
                        "default": 50,
                        "description": "Milliseconds to wait between presses"
                    },
                    "state_fields": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "If given, filter the returned state to only these fields"
                    }
                }
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": false,
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
        "autoui_find" => tool_find(shared, args),
        "autoui_exists" => tool_exists(shared, args),
        "autoui_press_sequence" => tool_press_sequence(shared, args),
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

    // Plan: prefer the RENDERED VTree snapshot (styled_vtree) — it reflects
    // the actual on-screen tree with child widgets inlined and `for` loops
    // expanded. Fall back to the raw view_template only when no rendered
    // frame is available yet (e.g. before first paint).
    if let Some(snap) = shared.clone_styled_vtree() {
        let layout_bounds = if include_bounds { shared.get_layout_bounds().clone() } else { HashMap::new() };
        let output = build_aura_from_styled_vtree(&snap, include_status, include_bounds, &layout_bounds);
        return text_result(output);
    }

    // Fallback: raw view_template (pre-render AURA source-style snapshot).
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

    let element_id = match parse_element_id(element_id_str) {
        Some(id) => id,
        None => return error_result(format!("Invalid element_id format: '{}' — expected 'aura_N' or 'vnode_N'", element_id_str)),
    };

    let shared = shared.lock().unwrap();

    // Plan 371: vnode_N path — inspect from styled VTree + View tree
    if let ElementId::Vnode(vnode_id) = element_id {
        let snap = match &shared.styled_vtree {
            Some(s) => s,
            None => return error_result("No styled VTree available yet"),
        };
        let vnode = match snap.vtree.get(vnode_id) {
            Some(n) => n,
            None => return error_result(format!("Element not found: vnode_{}", vnode_id.as_u64())),
        };
        let mut out = format!("Inspect vnode_{}\n", vnode_id.as_u64());
        out.push_str(&format!("  kind: {}\n", vnode.kind));
        // Show label/content from props
        match &vnode.props {
            crate::ui::vnode::VNodeProps::Text { content } => {
                out.push_str(&format!("  text: {}\n", content));
            }
            crate::ui::vnode::VNodeProps::Button { label } => {
                out.push_str(&format!("  label: {}\n", label));
            }
            crate::ui::vnode::VNodeProps::Input { value, placeholder, .. } => {
                out.push_str(&format!("  value: {}\n", value));
                out.push_str(&format!("  placeholder: {}\n", placeholder));
            }
            _ => {}
        }
        // Show events from the View tree
        if let Some(view) = &shared.view {
            if let Some(target) = find_view_by_path(view, &vnode.path) {
                let events = collect_view_events(target);
                if !events.is_empty() {
                    out.push_str("  events:\n");
                    for (ev, handler) in &events {
                        out.push_str(&format!("    {} -> {}\n", ev, handler));
                    }
                }
            }
        }
        return text_result(out);
    }

    // Legacy aura_N path
    let element_id = match element_id {
        ElementId::Aura(id) => id,
        _ => unreachable!(),
    };

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

/// Collect event handlers from a View as (event_type, handler_string) pairs.
fn collect_view_events(view: &View<DynamicMessage>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    match view {
        View::Button { onclick, .. } => {
            if let Some((w, e)) = extract_dyn_msg(onclick) {
                out.push(("onclick".into(), format!("{}.{}", w, e)));
            }
        }
        View::Input { on_change, on_submit, .. } => {
            if let Some(m) = on_change {
                if let Some((w, e)) = extract_dyn_msg(m) {
                    out.push(("onchange".into(), format!("{}.{}", w, e)));
                }
            }
            if let Some(m) = on_submit {
                if let Some((w, e)) = extract_dyn_msg(m) {
                    out.push(("onsubmit".into(), format!("{}.{}", w, e)));
                }
            }
        }
        View::Textarea { on_change, on_submit, .. } => {
            if let Some(m) = on_change {
                if let Some((w, e)) = extract_dyn_msg(m) {
                    out.push(("onchange".into(), format!("{}.{}", w, e)));
                }
            }
            // Plan 053 M4: textarea Enter → onsubmit (mirrors Input).
            if let Some(m) = on_submit {
                if let Some((w, e)) = extract_dyn_msg(m) {
                    out.push(("onsubmit".into(), format!("{}.{}", w, e)));
                }
            }
        }
        View::Checkbox { on_toggle, .. } => {
            if let Some(m) = on_toggle {
                if let Some((w, e)) = extract_dyn_msg(m) {
                    out.push(("ontoggle".into(), format!("{}.{}", w, e)));
                }
            }
        }
        _ => {}
    }
    out
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

    let element_id = match parse_element_id(element_id_str) {
        Some(id) => id,
        None => return error_result(format!("Invalid element_id format: '{}' — expected 'aura_N' or 'vnode_N'", element_id_str)),
    };

    let action_type = match action_str {
        "press" => UiActionType::Press,
        "type_text" => UiActionType::TypeText,
        "submit" => UiActionType::Submit,
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

        // Plan 371: dual-path dispatch — vnode_N covers all elements,
        // aura_N covers root-only (backward compat).
        let result = match element_id {
            ElementId::Vnode(vnode_id) => {
                execute_action_vnode(&shared, vnode_id, action_type, value)
            }
            ElementId::Aura(aura_id) => {
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
                execute_action_on_shared(&shared, &snapshot.tree, aura_id, action_type, value)
            }
        };
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

// ── Helpers for autoui_press_sequence (Plan 403) ─────────────────────────

/// Find buttons whose label matches `label` (case-insensitive substring),
/// returning up to `limit` VNodeIds. Reuses the same styled-VTree + label
/// matching as `tool_find`.
fn find_buttons_by_label(shared: &SharedState, label: &str, limit: usize) -> Vec<VNodeId> {
    let label_lower = label.to_lowercase();
    let snap = match shared.styled_vtree.as_ref() {
        Some(s) => s,
        None => return Vec::new(),
    };
    snap.vtree.nodes.iter()
        .filter(|vnode| {
            format!("{}", vnode.kind).to_lowercase() == "button"
                && vnode_searchable_text(&vnode.props).to_lowercase() == label_lower
        })
        .map(|n| n.id)
        .take(limit)
        .collect()
}

/// Format the state map as human-readable text (mirrors `tool_state` output).
fn format_state(state: &std::collections::HashMap<String, auto_val::Value>) -> String {
    if state.is_empty() {
        return "State: (empty)".to_string();
    }
    let mut out = String::from("State:\n");
    let mut entries: Vec<_> = state.iter().collect();
    entries.sort_by_key(|(k, _)| k.to_string());
    for (name, value) in &entries {
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
        out.push_str(&format!("  {}: {} ({})\n", name, value, type_str));
    }
    out
}

// ── Tool: autoui_press_sequence (Plan 403) ──────────────────────────────
// Press a sequence of buttons by label, then return the final state. Each key
// is matched to a rendered button by its label text (e.g. "2", "+", "="). This
// is the "expression evaluation via MCP" interface: send ["2","+","3","="] and
// read the result from the returned state — the computation happens through
// real UI button presses, not direct math.
fn tool_press_sequence(shared_handle: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let keys = match args.get("keys").and_then(|v| v.as_array()) {
        Some(arr) if !arr.is_empty() => arr.clone(),
        _ => return error_result("Missing or empty required parameter: keys (array of button labels)"),
    };
    let delay_ms = args.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(50);

    let mut pressed: Vec<String> = Vec::new();
    let mut last_error: Option<String> = None;

    for key_val in &keys {
        let label = match key_val.as_str() {
            Some(s) => s,
            None => { last_error = Some(format!("Non-string key: {:?}", key_val)); break; }
        };
        // Find the button by label.
        let find_result = {
            let shared = shared_handle.lock().unwrap();
            find_buttons_by_label(&shared, label, 1)
        };
        let vnode_id = match find_result.first() {
            Some(id) => *id,
            None => { last_error = Some(format!("No button found with label '{}'", label)); break; }
        };
        // Press it.
        let press_result = {
            let shared = shared_handle.lock().unwrap();
            execute_action_vnode(&shared, vnode_id, UiActionType::Press, None)
        };
        match press_result {
            Ok(_) => {
                pressed.push(label.to_string());
                if delay_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                }
            }
            Err(e) => { last_error = Some(format!("Press '{}' failed: {}", label, e)); break; }
        }
    }

    // Read final state.
    let state_text = {
        let shared = shared_handle.lock().unwrap();
        format_state(&shared.state)
    };

    if let Some(err) = last_error {
        error_result(format!("Sequence aborted after [{}]: {}", pressed.join(", "), err))
    } else {
        let mut msg = format!("Pressed: [{}]\n\n{}", pressed.join(", "), state_text);
        // If a "state_fields" param was given, filter to just those fields.
        if let Some(fields) = args.get("state_fields").and_then(|v| v.as_array()) {
            let want: Vec<String> = fields.iter().filter_map(|f| f.as_str().map(String::from)).collect();
            msg = format!("Pressed: [{}]\n\n", pressed.join(", "));
            for line in state_text.lines() {
                if want.iter().any(|w| line.contains(w.as_str())) {
                    msg.push_str(line);
                    msg.push('\n');
                }
            }
        }
        text_result(msg)
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

fn tool_screenshot(shared: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    // Plan 371 Task 20: parse visual-regression options.
    let opts = ScreenshotOptions {
        name: args.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        baseline: args.get("baseline").and_then(|v| v.as_bool()).unwrap_or(false),
        diff: args.get("diff").and_then(|v| v.as_bool()).unwrap_or(false),
        threshold: args.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.01),
    };
    // diff/baseline require a name.
    if (opts.baseline || opts.diff) && opts.name.is_empty() {
        return error_result("'name' is required when 'baseline' or 'diff' is true");
    }

    let rx = {
        let mut shared = shared.lock().unwrap();
        shared.request_screenshot(opts)
    };

    match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(Ok(msg)) => text_result(msg),
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
            // Match a field if it's requested exactly OR ends with the requested
            // name as a path suffix (Plan 371 Task 22b). Rust mode exposes
            // nested-component state with a prefix (e.g. `store.dark_mode`), so
            // querying `dark_mode` should still surface it; VM mode has the
            // bare name (`dark_mode`) and matches exactly.
            let matches = fields.iter().any(|f| {
                name.as_str() == f.as_str()
                    || name.ends_with(&format!(".{}", f))
            });
            if !matches {
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
    let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(5000);
    let interval_ms = args.get("interval_ms").and_then(|v| v.as_u64()).unwrap_or(100);

    // Plan 371 Task 9: element-level wait — poll autoui_exists until element
    // appears (condition="appears") or disappears (condition="disappears").
    let kind_filter = args.get("kind").and_then(|v| v.as_str());
    let label_filter = args.get("label").and_then(|v| v.as_str());
    let condition = args.get("condition").and_then(|v| v.as_str()).unwrap_or("appears");

    if kind_filter.is_some() || label_filter.is_some() {
        let want_found = condition == "appears";
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            let shared = shared_handle.lock().unwrap();
            let snap = match shared.clone_styled_vtree() {
                Some(s) => s,
                None => {
                    drop(shared);
                    std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                    if std::time::Instant::now() >= deadline {
                        return error_result("Timeout: no VTree snapshot available");
                    }
                    continue;
                }
            };
            let found = snap.vtree.nodes.iter().any(|vnode| {
                if let Some(kf) = kind_filter {
                    if format!("{}", vnode.kind).to_lowercase() != kf.to_lowercase() {
                        return false;
                    }
                }
                if let Some(lf) = label_filter {
                    if !vnode_searchable_text(&vnode.props).to_lowercase().contains(&lf.to_lowercase()) {
                        return false;
                    }
                }
                true
            });
            drop(shared);

            if found == want_found {
                let state_str = if want_found { "appeared" } else { "disappeared" };
                let crit: Vec<&str> = [kind_filter, label_filter].iter()
                    .filter_map(|v| *v).collect();
                return text_result(format!("Element {} ({}): {}", state_str, crit.join("/"), state_str));
            }

            if std::time::Instant::now() >= deadline {
                let state_str = if want_found { "appear" } else { "disappear" };
                return error_result(format!("Timeout waiting for element to {} (waited {}ms)", state_str, timeout_ms));
            }
            std::thread::sleep(std::time::Duration::from_millis(interval_ms));
        }
    }

    // Legacy: state field wait
    let field = match args.get("field").and_then(|v| v.as_str()) {
        Some(f) => f.to_string(),
        None => return error_result("Missing required parameter: 'field' (for state wait) or 'kind'+'label' (for element wait)"),
    };

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

    // Plan 371 续篇 / ash-gui M1:统一用 parse_element_id,支持 vnode_ 和 aura_。
    // vnode_ 覆盖所有渲染元素(含子组件内部,如 PromptBar 的 input),是 vm 模式
    // 定位子组件元素的唯一可靠方式 —— view_template 不展开 Component 节点,
    // find_first_input 找不到子组件内的 input。无 element_id 时优先从 styled_vtree
    // 找首个 Input vnode;回退到旧的 view_template 路径(向后兼容)。
    let element_id = match element_id_opt {
        Some(id_str) => match parse_element_id(id_str) {
            Some(id) => id,
            None => return error_result(format!("Invalid element_id format: '{}' — expected 'aura_N' or 'vnode_N'", id_str)),
        },
        None => {
            // 优先:从 styled_vtree(渲染后,展开 component)找首个 Input/Textarea vnode。
            let shared = shared_handle.lock().unwrap();
            if let Some(vnode_id) = find_first_input_vnode(&shared) {
                ElementId::Vnode(vnode_id)
            } else {
                // 回退:旧的 view_template 路径(只覆盖根 widget 直接子元素)。
                match &shared.view_template {
                    Some(t) => match find_first_input(&t.0) {
                        Some(id) => ElementId::Aura(id),
                        None => return error_result("No input element found — specify element_id (vnode_N from autoui_find)"),
                    },
                    None => return error_result("No UI available yet"),
                }
            }
        }
    };

    // clear_first:type_text 前先清空(vnode_ 走 execute_action_vnode,aura_ 走旧路径)。
    if clear_first {
        let clear_result = {
            let shared = shared_handle.lock().unwrap();
            match element_id {
                ElementId::Vnode(vid) => {
                    execute_action_vnode(&shared, vid, UiActionType::Clear, None)
                }
                ElementId::Aura(aura_id) => {
                    let (view, id_map) = match (&shared.view, &shared.id_map) {
                        (Some(v), Some(m)) => (v, m),
                        _ => return error_result("No UI available yet"),
                    };
                    let snapshot = SnapshotBuilder::build(&shared.widget_name, &shared.state, view, id_map);
                    execute_action_on_shared(&shared, &snapshot.tree, aura_id, UiActionType::Clear, None)
                }
            }
        };
        if let Err(e) = clear_result {
            // Clear may not be supported on all elements, that's OK
            eprintln!("AutoUI MCP: clear before type failed: {}", e);
        }
    }

    // Send type_text action
    let result = {
        let shared = shared_handle.lock().unwrap();
        match element_id {
            ElementId::Vnode(vid) => {
                execute_action_vnode(&shared, vid, UiActionType::TypeText, Some(auto_val::Value::str(&text)))
            }
            ElementId::Aura(aura_id) => {
                let (view, id_map) = match (&shared.view, &shared.id_map) {
                    (Some(v), Some(m)) => (v, m),
                    _ => return error_result("No UI available yet"),
                };
                let snapshot = SnapshotBuilder::build(&shared.widget_name, &shared.state, view, id_map);
                execute_action_on_shared(&shared, &snapshot.tree, aura_id, UiActionType::TypeText, Some(auto_val::Value::str(&text)))
            }
        }
    };

    match result {
        Ok(action_result) => text_result(action_result.to_aura_string()),
        Err(e) => error_result(e.to_string()),
    }
}

// ── Tool: autoui_keyboard (Plan 299 Phase 3) ──

fn tool_keyboard(shared_handle: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let key = match args.get("key").and_then(|v| v.as_str()) {
        Some(k) => k,
        None => return error_result("Missing required parameter: key"),
    };
    let _modifiers: Vec<String> = args.get("modifiers")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let shared = shared_handle.lock().unwrap();
    let widget_name = shared.widget_name.clone();

    // Plan 371: F12 toggles DevTools directly (bypasses key_binding lookup).
    let msg = if key == "F12" {
        ActionMessage {
            target: ActionTarget::Event {
                widget: String::new(),
                event: "__toggle_debug".to_string(),
            },
            action: UiActionType::Press,
            value: None,
        }
    } else {
        // EDGE-01: Build the key_str the same way keyboard_subscription does
        // (renderer.rs:2621-2653) and look up the handler in key_bindings
        // (which now includes element-attribute onkeydown.* bindings collected
        // by collect_onkeydown_bindings_with_registry). If found, dispatch the
        // handler directly; otherwise fall back to the legacy key_<lower>.
        let has_ctrl = _modifiers.iter().any(|m| m.eq_ignore_ascii_case("ctrl"));
        let has_alt = _modifiers.iter().any(|m| m.eq_ignore_ascii_case("alt"));
        let key_str = if has_ctrl || has_alt {
            let mut prefix = String::new();
            if has_ctrl { prefix.push_str("Ctrl+"); }
            if has_alt { prefix.push_str("Alt+"); }
            // For ctrl+r, key is "r"; first char lowercased.
            let c = key.chars().next().unwrap_or(' ').to_ascii_lowercase();
            format!("{}{}", prefix, c)
        } else {
            key.to_string()
        };
        // Look up in key_bindings (ArrowUp / Ctrl+r / Tab / etc).
        if let Some(handler_entry) = shared.key_bindings.get(&key_str) {
            // handler_entry is "WidgetName.HandlerName" (EDGE-01 format) or
            // plain "HandlerName" (bind-block format). Split on '.' to get
            // (widget, event). If no '.', dispatch to root widget.
            let (kb_widget, kb_event) = if let Some(dot) = handler_entry.find('.') {
                (&handler_entry[..dot], &handler_entry[dot + 1..])
            } else {
                (widget_name.as_str(), handler_entry.as_str())
            };
            ActionMessage {
                target: ActionTarget::Event {
                    widget: kb_widget.to_string(),
                    event: kb_event.to_string(),
                },
                action: UiActionType::Press,
                value: None,
            }
        } else {
            // Fallback: legacy key_<lower> handler name.
            let handler = format!("key_{}", key.to_lowercase());
            ActionMessage {
                target: ActionTarget::Event {
                    widget: widget_name,
                    event: handler,
                },
                action: UiActionType::Press,
                value: Some(format!("{}{}", _modifiers.iter().map(|m| format!("{}+", m)).collect::<Vec<_>>().join(""), key)),
            }
        }
    };

    match shared.send_action(msg) {
        Ok(()) => text_result(format!("Key sent: {}{}", _modifiers.iter().map(|m| format!("{}+", m)).collect::<Vec<_>>().join(""), key)),
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

/// Find the first Input/Textarea VNode in the styled VTree (rendered, component-
/// expanded). This is the vm-mode-correct way to locate inputs that live inside
/// child widgets (e.g. PromptBar's input) — view_template does not expand
/// Component nodes, so find_first_input misses them. Returns the VNodeId.
fn find_first_input_vnode(shared: &SharedState) -> Option<VNodeId> {
    use crate::ui::vnode::VNodeKind;
    let snap = shared.styled_vtree.as_ref()?;
    // VTree stores nodes flat in `nodes`; iterate in id order to find the first
    // Input/Textarea. DFS order via children would be more "first visible", but
    // flat iteration matches the snapshot's top-to-bottom render order closely
    // enough for the "type into the input" use case.
    snap.vtree.nodes.iter()
        .find(|n| matches!(n.kind, VNodeKind::Input | VNodeKind::Textarea))
        .map(|n| n.id)
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
        UiActionType::Submit => {
            if target.kind != "Input" && target.kind != "Textarea" {
                return Err(format!("Action 'submit' not valid for component type '{}'", target.kind));
            }
        }
    }

    // Find handler from actions list
    let action_name = match &action {
        UiActionType::Press => "press",
        UiActionType::TypeText => "type",
        UiActionType::Submit => "submit",
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
        target: ActionTarget::Event {
            widget: shared.widget_name.clone(),
            event: handler.clone(),
        },
        action: action.clone(),
        value: input_value,
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

// ============================================================================
// Plan 371: vnode_N support for action/inspect (unified ID system)
// ============================================================================

/// A unified element identifier — supports both legacy `aura_N` (root-only)
/// and `vnode_N` (covers all rendered elements including child widget internals).
enum ElementId {
    Aura(AuraNodeId),
    Vnode(VNodeId),
}

/// Parse either `aura_N` or `vnode_N` from a string.
fn parse_element_id(s: &str) -> Option<ElementId> {
    if let Some(n) = s.strip_prefix("aura_") {
        return n.parse::<u32>().ok().map(|n| ElementId::Aura(AuraNodeId(n)));
    }
    if let Some(n) = s.strip_prefix("vnode_") {
        return n.parse::<u64>().ok().map(|n| ElementId::Vnode(VNodeId::new(n)));
    }
    None
}

/// Navigate the View tree by a VNode's path (child-index sequence) to find the
/// corresponding rendered View. Mirrors `extract_children` from vnode_converter.
fn find_view_by_path<'a>(
    view: &'a View<DynamicMessage>,
    path: &[u16],
) -> Option<&'a View<DynamicMessage>> {
    let mut current = view;
    for &idx in path {
        let children: Vec<&View<DynamicMessage>> = match current {
            View::Column { children, .. } | View::Row { children, .. } => {
                children.iter().collect()
            }
            View::Grid { cells, .. } => cells.iter().collect(),
            View::Container { child, .. } | View::Scrollable { child, .. } => vec![child.as_ref()],
            View::List { items, .. } => items.iter().collect(),
            _ => return None,
        };
        current = children.get(idx as usize)?;
    }
    Some(current)
}

/// Extract (widget_name, event_name) from a DynamicMessage, encoding any
/// payload args into the event string via `encode_payload` so multi-arg
/// handlers (`.Reveal(cell.x, cell.y)`, `.SetDifficulty("beginner")`,
/// `.Digit(7)`) carry their args through the MCP action path (Plan 402 bug 4/5;
/// Plan 403 re-confirmed for `.Digit(n)` single-arg).
fn extract_dyn_msg(msg: &DynamicMessage) -> Option<(String, String)> {
    match msg {
        DynamicMessage::Typed { widget_name, event_name, args } => {
            let encoded = crate::ui::iced::encode_payload(event_name, args);
            Some((widget_name.clone(), encoded))
        }
        DynamicMessage::String(name) => Some((String::new(), name.clone())),
    }
}

/// Extract the handler (widget_name, event_name) from a View for a given action.
/// `action_name`: "press" / "type" / "submit" / "toggle" / "select" / "set_value".
fn extract_action_from_view(
    view: &View<DynamicMessage>,
    action_name: &str,
) -> Option<(String, String)> {
    match view {
        View::Button { onclick, .. } if action_name == "press" => extract_dyn_msg(onclick),
        View::Input { on_change, .. } | View::Textarea { on_change, .. }
            if action_name == "type" =>
        {
            on_change.as_ref().and_then(|m| extract_dyn_msg(m))
        }
        // submit → on_submit(Enter 键)。ash-gui M1:命令输入栏回车执行。
        // Plan 053 M4: textarea 也有 on_submit(onenter → OnEnter)。
        View::Input { on_submit, .. } | View::Textarea { on_submit, .. }
            if action_name == "submit" =>
        {
            on_submit.as_ref().and_then(|m| extract_dyn_msg(m))
        }
        View::Checkbox { on_toggle, .. } if action_name == "toggle" => {
            on_toggle.as_ref().and_then(|m| extract_dyn_msg(m))
        }
        _ => None,
    }
}

/// Get the VNodeKind string for a View (for action validation).
fn view_kind_str(view: &View<DynamicMessage>) -> &'static str {
    match view {
        View::Button { .. } => "Button",
        View::Input { .. } => "Input",
        View::Textarea { .. } => "Textarea",
        View::Checkbox { .. } => "Checkbox",
        View::Slider { .. } => "Slider",
        _ => "Other",
    }
}

/// Execute an action on a vnode_N element: look up the VNode, find the View by
/// path, extract the handler from the DynamicMessage, and dispatch.
fn execute_action_vnode(
    shared: &SharedState,
    vnode_id: VNodeId,
    action: UiActionType,
    value: Option<auto_val::Value>,
) -> Result<ActionResult, String> {
    let snap = shared.styled_vtree.as_ref()
        .ok_or_else(|| "No styled VTree available yet".to_string())?;
    let vnode = snap.vtree.get(vnode_id)
        .ok_or_else(|| format!("VNode not found: vnode_{}", vnode_id.as_u64()))?;

    // Validate action type by VNode kind (works for both VM and Rust mode)
    let action_name = match &action {
        UiActionType::Press => "press",
        UiActionType::TypeText | UiActionType::Clear => "type",
        UiActionType::Submit => "submit",
        UiActionType::Toggle => "toggle",
        UiActionType::SelectOption => "select",
        UiActionType::SetValue => "set_value",
    };
    let vnode_kind_str = format!("{}", vnode.kind);
    match &action {
        UiActionType::Press if vnode_kind_str != "Button" =>
            return Err(format!("Action 'press' not valid for component type '{}'", vnode_kind_str)),
        UiActionType::TypeText if vnode_kind_str != "Input" && vnode_kind_str != "Textarea" =>
            return Err(format!("Action 'type_text' not valid for component type '{}'", vnode_kind_str)),
        UiActionType::Submit if vnode_kind_str != "Input" && vnode_kind_str != "Textarea" =>
            return Err(format!("Action 'submit' not valid for component type '{}'", vnode_kind_str)),
        UiActionType::Toggle if vnode_kind_str != "Checkbox" =>
            return Err(format!("Action 'toggle' not valid for component type '{}'", vnode_kind_str)),
        _ => {}
    }

    // Build input_value for type_text/clear
    let mut input_value = match &action {
        UiActionType::TypeText => Some(
            value.as_ref()
                .map(|v| match v {
                    auto_val::Value::Str(s) => s.to_string(),
                    other => other.to_string(),
                })
                .ok_or_else(|| "Action 'type_text' requires a value parameter".to_string())?
        ),
        UiActionType::Clear => Some(String::new()),
        _ => None,
    };

    // Plan 371 Task 19: addressing mode depends on whether the typed View tree
    // is available. VM mode carries a `View<DynamicMessage>` whose handlers are
    // named (widget.event) — route by name. Rust mode has NO typed view in
    // SharedState, so send the VNode `path` and let the iced side walk its own
    // typed `View<C::Msg>` to the exact node and extract its handler (replacing
    // the old Debug-substring heuristic that silently failed on many labels).
    let msg = if let Some(view) = shared.view.as_ref() {
        // VM mode: extract named handler from the DynamicMessage.
        let target_view = find_view_by_path(view, &vnode.path)
            .ok_or_else(|| format!("View not found at path {:?}", vnode.path))?;
        let (widget_name, event_name) = extract_action_from_view(target_view, action_name)
            .ok_or_else(|| format!("No '{}' handler found on vnode_{}", action_name, vnode_id.as_u64()))?;
        // ash-gui M1:submit 模拟 onenter(如 PromptBar 的 `onenter: .OnEnter`)。
        // handler 带 .input 参数,但 submit 不自带 value —— 从 target_view
        // (Input/Textarea)读当前 value 字段作为 handler 参数,使 Run(cmd) 收到
        // 命令文本。
        if action == UiActionType::Submit && input_value.is_none() {
            if let View::Input { value: v, .. } = target_view {
                input_value = Some(v.clone());
            } else if let View::Textarea { value: v, .. } = target_view {
                input_value = Some(v.clone());
            }
        }
        let widget = if widget_name.is_empty() { shared.widget_name.clone() } else { widget_name };
        ActionMessage {
            target: ActionTarget::Event { widget, event: event_name.clone() },
            action: action.clone(),
            value: input_value.clone(),
        }
    } else {
        // Rust mode: address by VNode path — the iced side resolves it exactly.
        ActionMessage {
            target: ActionTarget::Path { path: vnode.path.clone() },
            action: action.clone(),
            value: input_value.clone(),
        }
    };

    // Build a human-readable handler label for the result report.
    let handler_label = match &msg.target {
        ActionTarget::Event { widget, event } => format!("{}.{}", widget, event),
        ActionTarget::Path { path } => format!("<path {:?}>", path),
    };
    shared.send_action(msg)?;

    Ok(ActionResult {
        status: "ok".to_string(),
        element_id: AuraNodeId(0),
        action: action.to_string(),
        handler: Some(format!(".{}", handler_label)),
        state_changes: vec![],
    })
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

// ── Tool: autoui_find (Plan 371 Task 7) ──

fn tool_find(shared: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let kind_filter = args.get("kind").and_then(|v| v.as_str());
    let label_filter = args.get("label").and_then(|v| v.as_str());
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as usize;

    let snap = shared.lock().unwrap().clone_styled_vtree();
    let snap = match snap {
        Some(s) => s,
        None => return error_result("No live VTree snapshot yet — retry after the window has painted."),
    };

    let opts = VTreeAtomOptions {
        scope: None,
        depth: None,
        include_box: false,
        include_style: false,
        include_events: true,
        include_source: false,
        include_props: true,
    };

    let kind_lower = kind_filter.map(|s| s.to_lowercase());
    let label_lower = label_filter.map(|s: &str| s.to_lowercase());

    // Find matching nodes
    let matched_ids: Vec<VNodeId> = snap.vtree.nodes.iter()
        .filter(|vnode| {
            if let Some(ref kf) = kind_lower {
                if format!("{}", vnode.kind).to_lowercase() != *kf {
                    return false;
                }
            }
            if let Some(ref lf) = label_lower {
                if !vnode_searchable_text(&vnode.props).to_lowercase().contains(lf) {
                    return false;
                }
            }
            true
        })
        .map(|n| n.id)
        .take(limit)
        .collect();

    if matched_ids.is_empty() {
        let mut criteria = Vec::new();
        if let Some(k) = kind_filter { criteria.push(format!("kind={}", k)); }
        if let Some(l) = label_filter { criteria.push(format!("label~={}", l)); }
        return text_result(format!("No nodes found matching: {}", criteria.join(", ")));
    }

    // Build ancestor-chain Atom subtrees for each match
    let mut out = format!("Found {} node(s):\n\n", matched_ids.len());
    for match_id in &matched_ids {
        // Build the ancestor chain: root → ... → matched node
        // Each ancestor shows its kind + vnode_N, with non-matched siblings collapsed.
        let subtree = build_ancestor_subtree(&snap, *match_id, &opts);
        out.push_str(&subtree);
        out.push('\n');
    }
    text_result(out)
}

/// Build an Atom-format ancestor-chain subtree for a matched node.
/// Shows the path from root to the matched node, with each ancestor's
/// siblings collapsed to a count (only the ancestor on the path is expanded).
fn build_ancestor_subtree(
    snap: &StyledNodeSnapshot,
    target_id: VNodeId,
    opts: &VTreeAtomOptions,
) -> String {
    let target = match snap.vtree.get(target_id) {
        Some(n) => n,
        None => return format!("(node vnode_{} not found)\n", target_id.as_u64()),
    };
    let path = &target.path;

    // Walk from root, following the path, building nested Atom text
    let mut out = String::new();
    let mut current_children: &[VNodeId] = match snap.vtree.root() {
        Some(root) => {
            // Root node itself
            let root_atom = VTreeAtomBuilder::build_node_only(root, &snap.computed, opts);
            out.push_str(&format!("{}", root_atom));
            &root.children
        }
        None => return out,
    };

    for (depth, &idx) in path.iter().enumerate() {
        let is_last = depth == path.len() - 1;
        let child_id = match current_children.get(idx as usize) {
            Some(id) => *id,
            None => break,
        };
        let child = match snap.vtree.get(child_id) {
            Some(n) => n,
            None => break,
        };

        // Count siblings (for collapse annotation)
        let sibling_count = current_children.len();

        out.push_str(" {\n");
        // Annotate collapsed siblings
        if sibling_count > 1 {
            out.push_str(&format!("  // {} sibling(s) collapsed\n", sibling_count - 1));
        }

        let child_atom = VTreeAtomBuilder::build_node_only(child, &snap.computed, opts);
        out.push_str(&format!("  {}", child_atom));

        if is_last {
            // Show the matched node's direct children too (1 level)
            if !child.children.is_empty() {
                out.push_str(" {\n");
                for cid in &child.children {
                    if let Some(gc) = snap.vtree.get(*cid) {
                        let gc_atom = VTreeAtomBuilder::build_node_only(gc, &snap.computed, opts);
                        out.push_str(&format!("    {}\n", gc_atom));
                    }
                }
                out.push_str("  }");
            }
            out.push('\n');
            // Close all open braces
            for _ in 0..=depth {
                out.push_str(&"  ".repeat(depth - (path.len() - 1 - depth) + 1));
                out.push_str("}\n");
            }
            break;
        }

        current_children = &child.children;
    }

    // Simplify: just return a clean ancestor chain
    build_ancestor_chain(snap, target_id, opts)
}

/// Simpler approach: build a linear ancestor chain as nested Atom.
fn build_ancestor_chain(
    snap: &StyledNodeSnapshot,
    target_id: VNodeId,
    opts: &VTreeAtomOptions,
) -> String {
    let target = match snap.vtree.get(target_id) {
        Some(n) => n,
        None => return String::new(),
    };
    let path = &target.path;

    // Collect ancestor nodes by walking the path from root
    let mut chain: Vec<&VNode> = Vec::new();
    let mut current = snap.vtree.root();
    for &idx in path {
        if let Some(node) = current {
            chain.push(node);
            current = node.children.get(idx as usize).and_then(|id| snap.vtree.get(*id));
        }
    }
    // Add the target itself
    chain.push(target);

    // Build nested Atom text
    let mut out = String::new();
    for (i, node) in chain.iter().enumerate() {
        let indent = "  ".repeat(i);
        let atom = VTreeAtomBuilder::build_node_only(node, &snap.computed, opts);
        if i > 0 {
            out.push_str(" {\n");
        }
        out.push_str(&format!("{}{}", indent, atom));
    }
    // Close braces
    for i in (0..chain.len()).rev() {
        if i > 0 {
            let indent = "  ".repeat(i - 1);
            out.push_str(&format!("\n{}}}", indent));
        }
    }
    out.push('\n');
    out
}

/// Plan 371: Quick existence check — returns a concise summary (count + IDs),
/// without the full Atom subtree. Faster than autoui_find for simple
/// "does this element exist?" validation.
fn tool_exists(shared: &SharedStateHandle, args: serde_json::Value) -> serde_json::Value {
    let kind_filter = args.get("kind").and_then(|v| v.as_str());
    let label_filter = args.get("label").and_then(|v| v.as_str());

    let snap = shared.lock().unwrap().clone_styled_vtree();
    let snap = match snap {
        Some(s) => s,
        None => return error_result("No live VTree snapshot yet."),
    };

    let kind_lower = kind_filter.map(|s| s.to_lowercase());
    let label_lower = label_filter.map(|s: &str| s.to_lowercase());

    let matches: Vec<&VNode> = snap.vtree.nodes.iter()
        .filter(|vnode| {
            if let Some(ref kf) = kind_lower {
                if format!("{}", vnode.kind).to_lowercase() != *kf {
                    return false;
                }
            }
            if let Some(ref lf) = label_lower {
                if !vnode_searchable_text(&vnode.props).to_lowercase().contains(lf) {
                    return false;
                }
            }
            true
        })
        .collect();

    let mut criteria = Vec::new();
    if let Some(k) = kind_filter { criteria.push(format!("kind={}", k)); }
    if let Some(l) = label_filter { criteria.push(format!("label~={}", l)); }

    if matches.is_empty() {
        text_result(format!("NOT FOUND (0 matches): {}", criteria.join(", ")))
    } else {
        let ids: Vec<String> = matches.iter()
            .map(|n| format!("vnode_{}", n.id.as_u64()))
            .collect();
        let labels: Vec<String> = matches.iter()
            .map(|n| {
                let txt = vnode_searchable_text(&n.props);
                if txt.is_empty() { format!("{}", n.kind) } else { format!("{} \"{}\"", n.kind, txt) }
            })
            .collect();
        text_result(format!(
            "FOUND {} match(es): {}\n  {}",
            matches.len(),
            criteria.join(", "),
            labels.join("; ")
        ))
    }
}

/// Extract searchable text from VNodeProps (label, content, value, placeholder).
fn vnode_searchable_text(props: &crate::ui::vnode::VNodeProps) -> String {
    use crate::ui::vnode::VNodeProps;
    match props {
        VNodeProps::Text { content } => content.clone(),
        VNodeProps::Button { label } => label.clone(),
        VNodeProps::Input { placeholder, value, .. } => format!("{} {}", placeholder, value),
        VNodeProps::Textarea { placeholder, value } => format!("{} {}", placeholder, value),
        VNodeProps::Checkbox { label, .. } => label.clone(),
        VNodeProps::Radio { label, .. } => label.clone(),
        _ => String::new(),
    }
}

/// Build an AURA-style snapshot string from the RENDERED VTree (styled_vtree),
/// which reflects the actual on-screen component tree — including inlined
/// child widgets and expanded `for` loops. This is the source of truth for
/// what the window actually shows, unlike the raw view_template which leaves
/// child-widget calls unexpanded.
///
/// Format mirrors AuraSnapshotBuilder's output (tag #id @rect [for] { props
/// events children }) so AI agents see a familiar structure, but every node
/// comes from the live render.
fn build_aura_from_styled_vtree(
    snap: &StyledNodeSnapshot,
    include_status: bool,
    include_bounds: bool,
    layout_bounds: &HashMap<String, (f32, f32, f32, f32)>,
) -> String {
    let mut out = String::new();
    out.push_str("AURA Snapshot v2 (rendered)\n");
    out.push_str(&format!("widget: \"{}\"\n", snap.widget_name));
    out.push_str("\ntree:\n");
    if let Some(root) = snap.vtree.root() {
        aura_vtree_node(root, &snap.vtree, &snap.computed, include_status, include_bounds, layout_bounds, 0, &mut out);
    }
    out
}

/// Recursive helper: emit one VNode as AURA-style text.
fn aura_vtree_node(
    node: &VNode,
    vtree: &VTree,
    computed: &HashMap<VNodeId, ComputedNodeLite>,
    include_status: bool,
    include_bounds: bool,
    layout_bounds: &HashMap<String, (f32, f32, f32, f32)>,
    indent: usize,
    out: &mut String,
) {
    use crate::ui::vnode::kind_keyword;
    let pad = "  ".repeat(indent);
    let tag = kind_keyword(node.kind);
    let id_str = format!(" #vnode_{}", node.id.as_u64());

    // Extract label from props (text content / button label).
    let label = match &node.props {
        VNodeProps::Text { content } => Some(content.clone()),
        VNodeProps::Button { label } => Some(label.clone()),
        VNodeProps::Checkbox { label, .. } => Some(label.clone()),
        VNodeProps::Radio { label, .. } => Some(label.clone()),
        _ => None,
    };

    // Computed metadata (class, events, bounds, for-context).
    let comp = computed.get(&node.id);
    let raw_class = comp.and_then(|c| c.raw_class.as_deref());
    let events = comp.map(|c| c.events.as_slice()).unwrap_or(&[]);
    let for_ctx = comp.and_then(|c| c.for_context.as_ref());

    // Suffix: @rect + [for: ...]
    let mut suffix = String::new();
    if include_bounds {
        if let Some(c) = comp {
            if let Some((x, y, w, h)) = c.bounds {
                suffix.push_str(&format!(" @rect({},{},{},{})", x.round() as i32, y.round() as i32, w.round() as i32, h.round() as i32));
            }
        }
    }
    if let Some((var, idx, val)) = for_ctx {
        let idx_str = idx.map(|i| i.to_string()).unwrap_or_else(|| "_".to_string());
        suffix.push_str(&format!(" [for: {}, {} = {}]", var, idx_str, val));
    }

    let has_body = !node.children.is_empty() || !events.is_empty() || raw_class.map_or(false, |c| !c.is_empty());

    // Opening line
    if let Some(lbl) = &label {
        if has_body {
            out.push_str(&format!("{}{}{} \"{}\"{} {{\n", pad, tag, id_str, lbl, suffix));
        } else {
            out.push_str(&format!("{}{}{} \"{}\"{}\n", pad, tag, id_str, lbl, suffix));
            return;
        }
    } else if has_body {
        out.push_str(&format!("{}{}{}{} {{\n", pad, tag, id_str, suffix));
    } else {
        out.push_str(&format!("{}{}{}\n", pad, tag, id_str));
        return;
    }

    // style (raw class string)
    if let Some(cls) = raw_class {
        if !cls.is_empty() {
            out.push_str(&format!("{}style: \"{}\"\n", "  ".repeat(indent + 1), cls));
        }
    }

    // Special props for inputs etc.
    match &node.props {
        VNodeProps::Input { placeholder, value, .. } => {
            if !placeholder.is_empty() {
                out.push_str(&format!("{}placeholder: \"{}\"\n", "  ".repeat(indent + 1), placeholder));
            }
            out.push_str(&format!("{}value: \"{}\"\n", "  ".repeat(indent + 1), value));
        }
        VNodeProps::Textarea { placeholder, value, .. } => {
            out.push_str(&format!("{}value: \"{}\"\n", "  ".repeat(indent + 1), value));
            if !placeholder.is_empty() {
                out.push_str(&format!("{}placeholder: \"{}\"\n", "  ".repeat(indent + 1), placeholder));
            }
        }
        VNodeProps::Checkbox { is_checked, .. } => {
            out.push_str(&format!("{}checked: {}\n", "  ".repeat(indent + 1), is_checked));
        }
        _ => {}
    }

    // events (sorted for determinism)
    let mut ev_sorted: Vec<&(String, String)> = events.iter().collect();
    ev_sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (ev, handler) in ev_sorted {
        out.push_str(&format!("{}{}: {}\n", "  ".repeat(indent + 1), ev, handler));
    }

    // children
    for cid in &node.children {
        if let Some(child) = vtree.get(*cid) {
            aura_vtree_node(child, vtree, computed, include_status, include_bounds, layout_bounds, indent + 1, out);
        }
    }

    out.push_str(&format!("{}}}\n", pad));
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
