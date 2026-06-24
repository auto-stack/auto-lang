# Plan 299: AutoUI MCP V2 — 全面改进方案

> **状态: ✅ 已完成 (Phase 1-3)**

## 问题诊断

当前 AutoUI MCP 服务（Plan 278/279/280/285）已实现 5 个工具，但实际使用中存在四大问题：

### 问题 1: AI 不知道怎么用

**现状**: MCP 协议自带 `tools/list` 机制，每个工具都有 `name`、`description`、`inputSchema` 字段。AI agent（如 Claude Code）会自动读取这些字段来理解工具用法。**当前实现的 description 字段已经包含了说明文本**——所以理论上 AI 应该能使用。

**实际困难**:
- 没有 `.mcp.json` 配置文件 → AI agent 根本连不上 MCP server
- 没有 Claude Code Skill 来引导 AI 的使用流程（应该先 snapshot，再根据结果 action）
- 工具描述偏技术化，缺少工作流指引（"先调 snapshot，从返回的 ID 中选目标，再调 action"）

### 问题 2: 功能不完善

**缺失的查询能力**:
- ❌ 没有计算后的样式信息（layout bounds 有但未暴露在 snapshot 中）
- ❌ 没有 widget model 的完整值（只有顶层 state，没有 per-widget 的 model）
- ❌ 没有焦点状态、可见性、禁用状态
- ❌ 没有列表渲染的展开项信息

**缺失的交互能力**:
- ❌ 没有鼠标 hover 模拟
- ❌ 没有键盘事件（Enter、Tab、Escape 等）
- ❌ 没有滚动操作
- ❌ 没有拖拽操作
- ❌ 没有焦点管理
- ❌ 没有表单提交
- ❌ `autoui_action` 返回的 `state_changes` 始终为空（因为状态变化在 iced update 中发生）

### 问题 3: 架构选型不清晰

当前是 TCP JSON-RPC（非标准 MCP transport），需要回答：MCP vs RESTful API vs CLI？

### 问题 4: 跨应用通信

MCP 设计目标是 AI↔App 通信，不是 App↔App 通信。

---

## 问题 1 解答: MCP 协议的自说明能力

### MCP 协议的 Tool Discovery 机制

MCP 协议（最新版本 2025-11-25）**自带完整的工具描述机制**：

| 字段 | 作用 | 当前实现 |
|------|------|----------|
| `name` | 工具唯一标识 | ✅ `autoui_snapshot` 等 |
| `description` | 人/AI 可读的说明 | ✅ 已有，但不够好 |
| `inputSchema` | JSON Schema 参数定义 | ✅ 已有 |
| `annotations` | readOnlyHint, destructiveHint 等 | ✅ 已有 |
| `title` | 显示名称（2025-11-25 新增）| ❌ 未设置 |
| `outputSchema` | 返回值结构定义 | ❌ 未设置 |

**AI agent 使用流程**:
1. Agent 调用 `tools/list` → 获得所有工具定义
2. Agent 阅读 `description` + `inputSchema` → 理解每个工具
3. Agent 根据用户需求选择工具 → 调用 `tools/call`
4. Agent 阅读返回结果 → 决定下一步

### 结论: 三层改进策略

**第一层: 改善工具描述（必须做）**
- 在每个工具的 `description` 中添加使用工作流指引
- 添加 `title` 字段（友好名称）
- 添加 `outputSchema` 让 AI 知道返回什么

**第二层: 创建 `.mcp.json` 配置（必须做）**
- 没有这个文件，AI agent 根本找不到 MCP server
- 放在项目根目录或 `.claude/` 下

**第三层: 创建 Claude Code Skill（推荐做）**
- 一个 `autoui-interact` skill，封装"snapshot → 分析 → action → 验证"工作流
- 包含 AURA 语法速查和常见操作模式

---

## 问题 2 解答: 系统性完善 MCP 接口

### 当前工具 vs 需要的工具

| 功能域 | 当前工具 | 缺失能力 | 优先级 |
|--------|----------|----------|--------|
| **查询** | `autoui_snapshot` | 无计算样式、无焦点状态、无可见性 | P0 |
| **查询** | `autoui_inspect` | 无 layout bounds、无 computed style | P1 |
| **查询** | `autoui_check` | — | ✅ 够用 |
| **查询** | `autoui_screenshot` | — | ✅ 够用 |
| **交互** | `autoui_action` (5种) | hover、键盘、滚动、拖拽、焦点 | P0 |
| **状态** | — | 等待状态变更、轮询变化 | P1 |
| **导航** | — | 多页面/多窗口支持 | P2 |

### 改进方案: 分层工具集

#### Layer 1: 查询工具（Query Tools）

**`autoui_snapshot` 增强**:
```
当前输出:
  center #aura_0 { ... }

增强后输出:
  center #aura_0 {
    // layout: x=0, y=0, w=800, h=600
    // visible: true, focused: false
    col #aura_1 {
      ...
      input #aura_3 {
        placeholder: "Add todo"
        value: ""
        // focused: true
        // bounds: x=20, y=50, w=200, h=30
        oninput: .InputChanged
      }
    }
  }
```

新增参数:
- `include_bounds: bool` — 包含每个元素的布局位置和尺寸
- `include_focus: bool` — 包含焦点状态
- `include_visibility: bool` — 包含可见性

**新增 `autoui_state` 工具** — 独立查询状态:
```json
{
  "name": "autoui_state",
  "description": "Query the current widget state values. Returns all state variables with their types and current values.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "fields": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Specific state field names to query. If omitted, returns all fields."
      }
    }
  }
}
```

**新增 `autoui_wait` 工具** — 等待状态变更:
```json
{
  "name": "autoui_wait",
  "description": "Wait for a state change to occur. Polls state at intervals until a change is detected or timeout.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "field": { "type": "string", "description": "State field to watch" },
      "timeout_ms": { "type": "integer", "default": 5000 },
      "interval_ms": { "type": "integer", "default": 100 }
    }
  }
}
```

#### Layer 2: 交互工具（Interaction Tools）

**`autoui_action` 增强** — 新增操作类型:

| 新 Action | 适用组件 | 说明 |
|-----------|----------|------|
| `hover` | 任意 | 模拟鼠标悬停（触发 hover 样式变化） |
| `key_press` | 任意 | 发送键盘事件（Enter, Tab, Escape, 字母等） |
| `focus` | Input, Textarea, Button | 设置焦点到指定元素 |
| `blur` | Input, Textarea | 移除焦点 |
| `scroll` | Container, Column, Row | 滚动到指定位置或指定子元素 |
| `clear` | Input, Textarea | 清空输入内容 |
| `submit` | Form（如有） | 提交表单 |

**新增 `autoui_type` 工具** — 更自然的键盘输入:
```json
{
  "name": "autoui_type",
  "description": "Simulate typing text character by character into the currently focused input element. More realistic than autoui_action type_text.",
  "inputSchema": {
    "type": "object",
    "required": ["text"],
    "properties": {
      "text": { "type": "string", "description": "Text to type" },
      "delay_ms": { "type": "integer", "default": 50, "description": "Delay between keystrokes in ms" },
      "clear_first": { "type": "boolean", "default": true, "description": "Clear existing text before typing" }
    }
  }
}
```

**新增 `autoui_keyboard` 工具** — 发送特殊按键:
```json
{
  "name": "autoui_keyboard",
  "description": "Send keyboard events (Enter, Tab, Escape, arrow keys, shortcuts).",
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
  }
}
```

#### Layer 3: `autoui_action` 返回值改进

当前问题: `autoui_action` 返回时 `state_changes` 始终为空，因为状态变化在 iced 主线程的 `update()` 中异步发生。

**解决方案**: 在 action 执行后，等待一次 SharedState 更新（带超时），然后对比前后状态:

```rust
fn tool_action(&self, args: serde_json::Value) -> serde_json::Value {
    // 1. 记录 action 前的状态
    let before_state = shared.state.clone();

    // 2. 发送 ActionMessage
    shared.send_action(msg)?;

    // 3. 等待 SharedState 更新（iced re-render）
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        drop(shared_lock);
        thread::sleep(Duration::from_millis(50));
        shared_lock = self.shared.lock().unwrap();
        if shared_lock.state_changed_since(&before_state) || Instant::now() > deadline {
            break;
        }
    }

    // 4. 计算 state diff
    let after_state = &shared_lock.state;
    let state_changes = compute_state_diff(&before_state, after_state);

    ActionResult { state_changes, .. }
}
```

---

## 问题 3 解答: MCP vs RESTful API vs CLI

### 三种接口对比

| 维度 | MCP | RESTful API | CLI |
|------|-----|-------------|-----|
| **设计目标** | AI 自主发现和调用 | 开发者编码调用 | 用户命令行调用 |
| **发现机制** | `tools/list` 自动发现 | OpenAPI/Swagger 文档 | `--help` 文本 |
| **调用方** | AI 模型自主决定 | 开发者硬编码 | 用户手动输入 |
| **通信方式** | JSON-RPC 2.0（双向） | HTTP CRUD（单向） | stdin/stdout |
| **状态管理** | 有 session 概念 | 通常无状态 | 每次新进程 |
| **AI 友好度** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **跨应用通信** | 不适合 | ⭐⭐⭐⭐⭐ | 不适合 |
| **实现复杂度** | 中等 | 低 | 低 |

### 核心区别

**MCP**: AI agent 调用 `tools/list`，拿到工具描述后**自主决定**调用哪个工具、传什么参数。AI 是"驾驶员"。

**RESTful API**: 开发者阅读 API 文档，**硬编码** HTTP 请求。开发者是"驾驶员"。

**CLI**: AI 通过 Bash 执行命令，解析文本输出。容易出错，不结构化。

### 推荐策略: 三层架构

```
┌──────────────────────────────────────────────────┐
│  Layer 1: MCP (AI Agent 交互)                      │
│  - AI 自主发现和调用工具                             │
│  - 用于开发调试、AI 驱动的 UI 自动化                   │
│  - Transport: Streamable HTTP (标准 MCP)            │
├──────────────────────────────────────────────────┤
│  Layer 2: RESTful API (应用间通信)                   │
│  - 结构化的 HTTP 端点                                │
│  - 用于两个 AutoUI 应用之间互相调用                    │
│  - 用于 Web 前端 → 桌面后端的通信                     │
├──────────────────────────────────────────────────┤
│  Layer 3: CLI (脚本/CI)                            │
│  - `auto ui inspect`, `auto ui action`             │
│  - 用于 CI/CD 测试、脚本自动化                        │
└──────────────────────────────────────────────────┘
```

**当前阶段建议**: 专注 MCP（Layer 1），因为主要使用场景是 AI 开发调试。RESTful API（Layer 2）和 CLI（Layer 3）可以后续从 MCP 工具逻辑中提取复用。

### Transport 改进: 从 TCP JSON-RPC 迁移到标准 Streamable HTTP

当前实现使用自定义 TCP JSON-RPC，不符合 MCP 标准 transport 规范。应迁移到 **Streamable HTTP**：

```
当前:  AI → TCP socket → line-delimited JSON-RPC → MCP server
改为:  AI → HTTP POST /mcp → standard Streamable HTTP → MCP server
```

好处:
- 兼容所有标准 MCP 客户端（Claude Code、Cursor 等）
- 支持多客户端同时连接
- 支持会话管理
- 不需要自定义 transport 适配器

---

## 问题 4 解答: 跨应用通信

### MCP 不适合 App↔App 通信

MCP 的安全模型假设中间有一个 LLM 作为"决策者"和"人类监督者"。直接 App↔App 使用 MCP:
- 没有错误恢复机制
- 没有服务发现
- 没有负载均衡
- 安全模型不匹配

### 推荐方案: 双层接口

**短期（现在）**: MCP 用于 AI 调试，足够了。

**中期（需要跨应用时）**: 实现一套 RESTful API：

```
App A (AutoUI)
    │
    │ HTTP GET /api/state, POST /api/action
    │
    v
App B (AutoUI)
    │ 内嵌 HTTP server (Axum)
    │ /api/snapshot → JSON 格式的 UI 状态
    │ /api/action   → 执行操作并返回结果
    │ /api/state    → 查询特定状态字段
    │ /api/events   → SSE 事件流（实时通知）
    v
```

**核心**: MCP 工具的内部逻辑（query state, execute action）可以提取为独立的 service 层，MCP 和 REST 都调用同一个 service。这样实现 REST API 只是加一层 HTTP 路由。

### 未来架构

```
              ┌─────────────────┐
              │  Service Layer  │  ← 共享逻辑
              │  (query/action) │
              └───────┬─────────┘
                      │
          ┌───────────┼───────────┐
          │           │           │
    ┌─────┴─────┐ ┌───┴───┐ ┌───┴────┐
    │ MCP Server│ │ REST  │ │  CLI   │
    │ (AI 用)   │ │(App用)│ │(脚本用)│
    └───────────┘ └───────┘ └────────┘
```

---

## 实施计划

### Phase 1: 基础设施修复（让 MCP 可用）

**目标**: 让 AI agent 能发现并正确使用现有工具。

| 步骤 | 内容 | 文件 |
|------|------|------|
| 1.1 | 创建 `.mcp.json` 配置文件，连接到 localhost:9247 | `.mcp.json` (新建) |
| 1.2 | 改善所有工具的 `description`，添加使用工作流 | `mcp_server.rs` |
| 1.3 | 添加 `title` 字段到工具定义 | `mcp_server.rs` |
| 1.4 | 将 TCP transport 迁移到 Streamable HTTP | `mcp_server.rs` |
| 1.5 | 创建 `autoui-interact` Claude Code Skill | `.claude/skills/autoui-interact/SKILL.md` |

### Phase 2: 查询能力增强

**目标**: 通过 MCP 能获取 UI 的所有信息。

| 步骤 | 内容 | 文件 |
|------|------|------|
| 2.1 | `autoui_snapshot` 增加 bounds、focus、visibility 参数 | `mcp_server.rs`, `aura_snapshot_builder.rs` |
| 2.2 | 新增 `autoui_state` 工具 — 独立状态查询 | `mcp_server.rs` |
| 2.3 | 新增 `autoui_wait` 工具 — 等待状态变更 | `mcp_server.rs` |
| 2.4 | SharedState 存储焦点信息和可见性 | `mcp_server.rs`, `dynamic.rs`, `renderer.rs` |

### Phase 3: 交互能力增强

**目标**: 所有用户能做的操作，MCP 都能做。

| 步骤 | 内容 | 文件 |
|------|------|------|
| 3.1 | `autoui_action` 增加 hover、focus、blur、scroll、clear 操作 | `mcp_server.rs`, `action_mapper.rs` |
| 3.2 | 新增 `autoui_keyboard` 工具 — 特殊按键 | `mcp_server.rs` |
| 3.3 | 新增 `autoui_type` 工具 — 自然输入 | `mcp_server.rs` |
| 3.4 | `autoui_action` 返回真实 state_changes | `mcp_server.rs` |
| 3.5 | iced 端支持模拟事件（hover、keyboard、scroll） | `renderer.rs`, `dynamic.rs` |

### Phase 4: Service Layer 提取（未来）

**目标**: 为 REST API 和 CLI 复用做准备。

| 步骤 | 内容 | 文件 |
|------|------|------|
| 4.1 | 提取 UiService trait（query + action 接口）| `ui/service.rs` (新建) |
| 4.2 | MCP server 调用 UiService | `mcp_server.rs` 重构 |
| 4.3 | REST API server 调用 UiService | `ui/rest_server.rs` (新建) |
| 4.4 | CLI 命令调用 UiService | `auto/src/main.rs` |

---

## 工具全景图（Phase 1-3 完成后）

```
autoui_snapshot   — 页面快照（组件树 + 状态 + 布局 + 渲染状态）
autoui_inspect    — 元素检查（单个元素的详细信息）
autoui_state      — 状态查询（独立查询 widget 状态变量）
autoui_check      — 渲染诊断（检测 iced 后端的渲染问题）
autoui_screenshot — 截图（PNG 文件路径）
autoui_wait       — 等待状态变更

autoui_action     — 通用操作（press, type_text, toggle, select, set_value,
                    hover, focus, blur, scroll, clear）
autoui_type       — 自然键盘输入（逐字输入，支持延迟和清除）
autoui_keyboard   — 特殊按键（Enter, Tab, Escape, 方向键, 组合键）
```

10 个工具，覆盖 UI 的完整观测和操控能力。

---

## 典型使用场景

### 场景 1: AI 调试 Todo App

```
AI: 调用 autoui_snapshot
→ 看到完整的组件树，发现 input #aura_3 和 button #aura_4

AI: 调用 autoui_action { element_id: "aura_3", action: "type_text", value: "Buy milk" }
→ 文本输入完成

AI: 调用 autoui_action { element_id: "aura_4", action: "press" }
→ 按钮点击，返回 state_changes: { input: "" -> "Buy milk", todo_count: 2 -> 3 }

AI: 调用 autoui_snapshot
→ 确认新的 checkbox 已出现
```

### 场景 2: AI 修复 Layout 问题

```
AI: 调用 autoui_snapshot { include_bounds: true }
→ 看到 grid #aura_16 的 layout 显示异常（全部竖排）

AI: 调用 autoui_check
→ 确认 grid 是 FALLBACK（iced 不支持 grid 布局）

AI: 修改源码，用 row/col 替代 grid

AI: 调用 autoui_screenshot
→ 视觉确认 layout 已修复
```

### 场景 3: AI 驱动的端到端测试

```
AI: 调用 autoui_action { element_id: "aura_3", action: "focus" }
AI: 调用 autoui_type { text: "Test note", clear_first: true }
AI: 调用 autoui_keyboard { key: "Enter" }
AI: 调用 autoui_wait { field: "todo_count", timeout_ms: 1000 }
→ 确认状态已更新，测试通过
```

---

## 实施计划（Phases 1–3）

> Phase 4（Service Layer 提取）推迟。以下为 Phases 1–3 的具体执行步骤。

### Phase 1: 基础设施修复（让 MCP 可用）

#### 1.1 Streamable HTTP Transport

**Why**: Claude Code 的 `.mcp.json` 仅支持 `stdio` 和 SSE/Streamable HTTP transports。当前 TCP JSON-RPC 无法被发现。

**How**: 使用 `axum` 添加 HTTP 端点，替换 TCP listener。现有 `handle_request()` 保持不变——仅包装在 HTTP 中。

**Files**:
- `crates/auto-lang/Cargo.toml` — 添加 `axum` 依赖（`ui-interpreter` feature）
- `crates/auto-lang/src/ui/mcp_server.rs` — 用 axum HTTP server 替换 `run()` / `handle_client()`:
  - `POST /mcp` — JSON-RPC request → JSON-RPC response

```rust
pub fn run(&self) {
    let shared = self.shared.clone();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all().build().unwrap();
    rt.block_on(async {
        let app = axum::Router::new()
            .route("/mcp", axum::routing::post(mcp_http_handler))
            .with_state(shared);
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}
```

Remove: `use std::net::TcpListener`, `handle_client()`, line-delimited JSON parsing.

#### 1.2 创建 `.mcp.json`

**File**: `.mcp.json`（项目根目录，新建）

```json
{
  "mcpServers": {
    "autoui": {
      "url": "http://localhost:9247/mcp",
      "description": "AutoUI desktop MCP server — inspect and interact with AutoUI applications"
    }
  }
}
```

#### 1.3 改善工具描述

**File**: `crates/auto-lang/src/ui/mcp_server.rs` — `tool_definitions()` method

- 添加 `title` 字段
- 改善 `description` 添加使用工作流指引

#### 1.4 创建 Claude Code Skill

**File**: `.claude/skills/autoui-interact/SKILL.md`（新建）

工作流引导：snapshot → 分析 → action → 验证。包含 AURA 语法速查和常见操作模式。

---

### Phase 2: 查询能力增强

#### 2.1 新工具 `autoui_state`

**File**: `crates/auto-lang/src/ui/mcp_server.rs`

独立状态查询工具。读取 `shared.state`，可选按 `fields` 参数过滤，返回文本格式。

```json
{
  "name": "autoui_state",
  "title": "Query State",
  "inputSchema": {
    "properties": {
      "fields": { "type": "array", "items": { "type": "string" } }
    }
  }
}
```

#### 2.2 新工具 `autoui_wait`

**File**: `crates/auto-lang/src/ui/mcp_server.rs`

轮询 SharedState 直到状态字段变化或超时：
1. 捕获 before-state
2. `thread::sleep(interval)` 循环，重新读取 state，对比
3. 返回 diff 或 timeout error

#### 2.3 Snapshot 增强

**File**: `mcp_server.rs`, `aura_snapshot_builder.rs`

`autoui_snapshot` 新增参数：
- `include_bounds: bool` — 显示 layout 位置/尺寸（数据已在 SharedState，Plan 282）
- `include_focus: bool` — 焦点状态（stub: 暂时 "unknown"）
- `include_visibility: bool` — 可见性（stub: 暂时 true）

---

### Phase 3: 交互能力增强

#### 3.1 新 Action 类型

**File**: `mcp_types.rs` — `UiActionType` 新增 `Clear` variant

**File**: `mcp_server.rs` — `execute_action_on_shared()` 处理 Clear（写入空字符串 + 触发 handler）

> hover / focus / blur / scroll 需要 iced 端支持，暂缓。

#### 3.2 新工具 `autoui_type`

**File**: `mcp_server.rs`

便利工具：找到焦点输入框（或指定 element），可选清除后输入文本。参数：`text`, `element_id`（可选）, `clear_first`（默认 true）。

#### 3.3 新工具 `autoui_keyboard`

**File**: `mcp_server.rs`

发送特殊按键。`ActionMessage` 扩展 `key_event: Option<KeyEvent>`：

```rust
pub struct KeyEvent {
    pub key: String,
    pub modifiers: Vec<String>,
}
```

#### 3.4 Action 返回真实 state_changes

**File**: `mcp_server.rs` — `tool_action()`

发送 action 后，轮询 SharedState 最多 500ms 检测变化，对比 before/after 填充 `state_changes`。

---

## 文件变更总览

| File | Phase | Changes |
|------|-------|---------|
| `Cargo.toml` | 1 | 添加 `axum` 依赖 |
| `mcp_server.rs` | 1,2,3 | HTTP transport、新工具、描述改善、state tracking |
| `mcp_types.rs` | 3 | `UiActionType::Clear`、`KeyEvent` struct |
| `action_mapper.rs` | 3 | Handle Clear action |
| `aura_snapshot_builder.rs` | 2 | bounds/focus/visibility flags |
| `.mcp.json` | 1 | 新建 — MCP server config |
| `.claude/skills/autoui-interact/SKILL.md` | 1 | 新建 — Claude Code skill |

## 验证

每个 phase 完成后：
1. `cargo build -p auto-lang` — 必须编译通过
2. 运行 AutoUI 应用，验证 MCP HTTP server 在 port 9247 启动
3. 通过 Claude Code 测试：`.mcp.json` 自动连接，`tools/list` 正常

Phase 1: `curl -X POST http://localhost:9247/mcp -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'`
Phase 2: 测试 `autoui_state`、`autoui_wait`、snapshot `include_bounds`
Phase 3: 测试 `clear` action、`autoui_type`、`autoui_keyboard`、`state_changes` 非空

## 提交计划

1. `feat(autoui): MCP infrastructure — .mcp.json, HTTP transport, tool descriptions, skill`
2. `feat(autoui): query tools — autoui_state, autoui_wait, snapshot bounds`
3. `feat(autoui): interaction tools — clear action, autoui_type, autoui_keyboard, state tracking`
