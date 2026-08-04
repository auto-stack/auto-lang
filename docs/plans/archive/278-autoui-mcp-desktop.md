# Plan 278: AutoUI MCP 桌面工具设计

**状态: ✅ 已完成（Phase 1: 核心框架）** — 2026-06-02

已完成所有 4 个核心模块的实现并编译通过。下一步是集成到 iced renderer 的 `run_dynamic_iced()` 入口函数中。

## Context

当前 AutoLang 的 MCP server（`mcp/`）仅提供 VM 会话管理工具（`auto_evaluate`, `auto_inspect` 等），无法让 AI agent 直接与 AutoUI 构建的桌面界面交互。本方案设计一套新的 MCP 工具，让 AI（如 Claude Code）能够"看到"UI 界面并操纵其中的组件。

**设计目标：** AI agent 可以像人类用户一样读取界面信息并执行操作，实现"AI 驱动的 UI 自动化"。

## 部署架构：内嵌模式

iced 桌面进程内嵌一个 MCP Server，通过 TCP 端口对外提供服务。AI agent 的 MCP 配置中添加该服务器。

```
AI Agent (Claude Code)
    |
    | MCP (TCP JSON-RPC, e.g., localhost:9247)
    v
AutoUI Desktop App (iced 进程)
    ├── MCP Server Thread (autoui_snapshot, autoui_inspect, autoui_action)
    ├── DynamicState (组件状态)
    │   ├── DynamicComponent
    │   │   ├── VmBridge (read_state, call_handler)
    │   │   └── AuraNode (视图模板)
    │   └── iced Renderer
    └── iced Event Loop
```

## 工具接口设计（3 个工具）

### 工具 1: `autoui_snapshot` — 页面快照

返回当前页面的 AURA 风格结构化快照，包含所有组件的层级、属性、可读信息和可操作内容。相当于对页面的"结构化 OCR"。

**输入 Schema:**
```json
{
  "type": "object",
  "properties": {
    "include_styles": {
      "type": "boolean",
      "default": false,
      "description": "是否包含样式信息"
    },
    "include_state": {
      "type": "boolean",
      "default": true,
      "description": "是否包含完整的 widget 状态变量值"
    }
  }
}
```

**输出（AURA 格式示例）：**
```
AuraUI Snapshot v1
widget: "TodoApp"
state:
  input: "" (str)
  todo_count: 2 (int)
  active_count: 1 (int)

tree:
  Center #aura_1 {
    Column #aura_2 {
      Row #aura_3 {
        Input #aura_4 {
          placeholder: "Add todo"
          value: ""
          actions: [type -> .InputChanged]
        }
        Button #aura_5 {
          label: "Add"
          actions: [press -> .AddTodo]
        }
      }
      Text #aura_6 { content: "Active: 1" }
      Column #aura_7 {
        Checkbox #aura_8 {
          checked: false
          label: "Buy milk"
          actions: [toggle -> .ToggleTodo]
        }
        Checkbox #aura_9 {
          checked: true
          label: "Walk dog"
          actions: [toggle -> .ToggleTodo]
        }
      }
    }
  }
```

**Annotations:** `readOnlyHint: true`

### 工具 2: `autoui_inspect` — 元素检查

查询单个组件的详细信息，包括类型、属性、可用操作、源码位置。

**输入 Schema:**
```json
{
  "type": "object",
  "required": ["element_id"],
  "properties": {
    "element_id": {
      "type": "string",
      "description": "目标元素的 AuraNodeId，如 'aura_4'。可从 autoui_snapshot 获取。"
    }
  }
}
```

**输出（AURA 格式示例）：**
```
Inspect #aura_4
  type: Input
  properties:
    placeholder: "Add todo"
    value: ""
    password: false
  actions:
    type -> .InputChanged
  source: line 15, col 8
```

**Annotations:** `readOnlyHint: true`

### 工具 3: `autoui_action` — 执行操作

对指定组件执行操作。

**输入 Schema:**
```json
{
  "type": "object",
  "required": ["element_id", "action"],
  "properties": {
    "element_id": {
      "type": "string",
      "description": "目标元素的 AuraNodeId，如 'aura_5'"
    },
    "action": {
      "type": "string",
      "enum": ["press", "type_text", "toggle", "select_option", "set_value"],
      "description": "操作类型。press=按钮点击; type_text=输入文本; toggle=复选框切换; select_option=下拉选择; set_value=滑块设值"
    },
    "value": {
      "description": "操作参数。type_text 时为字符串; select_option 时为选项索引(整数)或标签(字符串); set_value 时为数值。",
      "oneOf": [{"type": "string"}, {"type": "number"}, {"type": "integer"}]
    }
  }
}
```

**输出（AURA 格式示例）：**
```
ActionResult
  status: ok
  element: #aura_5
  action: press
  handler: .AddTodo
  state_changes:
    input: "" -> ""
    todo_count: 2 -> 3
```

**Annotations:** `destructiveHint: false`, `idempotentHint: false`

### 操作类型与组件映射

| Action | 适用组件 | 内部实现 |
|--------|----------|----------|
| `press` | Button | 调用 `VmBridge::call_handler(event_name, [])` |
| `type_text` | Input, Textarea | 调用 `VmBridge::write_state(field, value)` + `call_handler(on_change, [value])` |
| `toggle` | Checkbox | 调用 `VmBridge::call_handler(on_toggle, [])` |
| `select_option` | Select, Radio | 调用 `VmBridge::call_handler(on_select, [index, label])` |
| `set_value` | Slider | 调用 `VmBridge::call_handler(on_change, [value])` |

## 组件 ID 方案

使用现有的 **AuraNodeId**（`aura/types.rs:19`），格式为 `aura_{u32}`。

**选择理由：**
- AuraNodeId 在 AuraNode 提取时分配，跨重渲染稳定
- 已有 `DebugIdMap`（`ui/debug_id_map.rs`）映射路径 → AuraNodeId
- 已有 `aura_to_id_cache`（`DynamicState`）映射 AuraNodeId → 调试元素 ID
- 路径式 ID（如 `root.col[0].button`）在条件渲染/列表渲染时不稳定

## 实现步骤

### Step 1: 定义 UI 快照类型
**新建文件:** `crates/auto-lang/src/ui/mcp_types.rs`

定义以下结构体：
- `UiSnapshot { widget_name, state: Vec<(String, Value, String)>, tree: UiNode }`
- `UiNode { id: AuraNodeId, kind: String, props: Vec<(String, String)>, actions: Vec<UiAction>, children: Vec<UiNode> }`
- `UiAction { name: String, handler: String }`
- `ElementInfo { id, kind, props, actions, source_location }`
- `ActionResult { status, element_id, action, handler, state_changes }`
- 实现 `to_aura_string()` 格式化方法（参照 `aura/atom.rs` 的 Atom 格式风格）

### Step 2: 实现 View 树快照遍历
**新建文件:** `crates/auto-lang/src/ui/snapshot_builder.rs`

- 遍历 `View<DynamicMessage>` 树，提取每个节点的信息
- 对每个 View 变体，提取属性和可操作事件
- 将 AuraNodeId 关联到节点（通过 `DebugIdMap` 反查）
- 读取当前状态值（通过 `VmBridge::read_all_state()`）
- 构建 `UiSnapshot` 结构体

**关键复用：**
- `VmBridge::read_all_state()` — 读取所有状态
- `View` 枚举的各个变体 — 提取组件类型和属性
- `DebugIdMap` — 反查 AuraNodeId
- `aura/atom.rs` 的序列化风格 — AURA 文本格式

### Step 3: 实现操作映射器
**新建文件:** `crates/auto-lang/src/ui/action_mapper.rs`

- 接收 `element_id` + `action` + `value`
- 在 View 树中查找目标 AuraNodeId 对应的节点
- 验证操作与组件类型匹配（如 Button 只接受 press）
- 提取事件处理器名称
- 调用 `VmBridge` 的相应方法：
  - `press` → `call_handler(handler_name, [])`
  - `type_text` → `write_state(bound_field, value)` + `call_handler(on_change, [value])`
  - `toggle` → `call_handler(on_toggle, [])`
  - `select_option` → `call_handler(on_select, [index, label])`
  - `set_value` → `call_handler(on_change, [value])`
- 记录操作前后的状态变化

### Step 4: 在 iced 进程中嵌入 MCP Server
**修改文件:** `crates/auto-lang/src/ui/iced/renderer.rs`

- 在 `run_dynamic_iced()` 中，启动一个后台线程运行 MCP Server
- MCP Server 使用 TCP 传输（如 `localhost:9247`），端口可通过环境变量 `AUTOUI_MCP_PORT` 配置
- 后台线程持有 `Arc<Mutex<DynamicComponent>>` 引用
- 处理 JSON-RPC 请求，路由到 Step 2/3 的实现

### Step 5: 注册 MCP 工具定义
**修改文件:** `crates/auto-lang/src/mcp/server.rs`

- 添加 `autoui_snapshot`、`autoui_inspect`、`autoui_action` 三个工具定义
- 实现 dispatch 逻辑
- 复用现有的 JSON-RPC 框架

### Step 6: 端口与集成配置

- 默认端口: `9247`（可通过 `AUTOUI_MCP_PORT` 环境变量覆盖）
- AI agent 配置示例（`.claude/settings.json` 或 `.mcp.json`）：
```json
{
  "mcpServers": {
    "autoui": {
      "url": "http://localhost:9247/mcp",
      "transport": "streamable-http"
    }
  }
}
```

## 关键文件清单

| 文件 | 操作 | 说明 | 状态 |
|------|------|------|------|
| `crates/auto-lang/src/ui/mcp_types.rs` | 新建 | 快照、检查、操作的类型定义 + AURA 格式化 | ✅ 完成 |
| `crates/auto-lang/src/ui/snapshot_builder.rs` | 新建 | View 树 → UiSnapshot 遍历 | ✅ 完成 |
| `crates/auto-lang/src/ui/action_mapper.rs` | 新建 | 操作 → VmBridge 调用映射 | ✅ 完成 |
| `crates/auto-lang/src/ui/mcp_server.rs` | 新建 | 内嵌 TCP MCP Server + 3 个工具 + SharedState | ✅ 完成 |
| `crates/auto-lang/src/ui/mod.rs` | 修改 | 导出 4 个新模块 | ✅ 完成 |
| `crates/auto-lang/src/ui/iced/renderer.rs` | 待修改 | 在 `run_dynamic_iced()` 中启动 MCP 后台线程 + 更新共享状态 | ⏳ 下一步 |

## 验证方案

1. **编译验证:** ✅ `cargo build -p auto-lang --features ui-interpreter` — 0 errors
2. **单元测试:** 为 `snapshot_builder` 和 `action_mapper` 编写测试，使用 mock View 树（待做）
3. **集成测试:** 在 `run_dynamic_iced()` 接入 MCP 后，运行 `examples/ui/013-todo/`：
   - 调用 `autoui_snapshot`，验证返回正确的 AURA 格式快照
   - 调用 `autoui_action` 对 "Add" 按钮执行 `press`
   - 再次 `autoui_snapshot` 验证状态变化（待做）
4. **手动验证:** 用 `npx @modelcontextprotocol/inspector` 连接到 MCP Server，测试所有工具（待做）
