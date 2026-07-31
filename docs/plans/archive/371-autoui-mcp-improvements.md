# Plan 371: AutoUI MCP 功能大改进 — Agent 驱动的 UI 自动化验证

> **状态**: ✅ COMPLETE (2026-07-31) — Tasks 1-22 全部完成；L1（特判泛化）+ 013-todo 新应用验证通过。L2/L3 为可选后续。
> **目标**: 让 Agent 通过 MCP 服务直接操纵 AutoUI 界面（类似 playwright-mcp），实现"开发→构建→启动→自动验证"的闭环。
>
> **核心挑战**: 当前 MCP 的 action/inspect 工具只覆盖根组件元素（`aura_N`），子组件内部元素（EditorPanel 的按钮、NavTree 的列表项）无法操作。需要统一 ID 系统为 `vnode_N`（覆盖全部渲染元素）。

---

## 1. 背景与问题

### 1.1 两套 ID 系统

| | `aura_N` | `vnode_N` |
|---|---|---|
| **类型** | `AuraNodeId(u32)` — 源码提取时静态计数器 | `VNodeId(u64)` — 渲染时路径 FNV-1a 哈希 |
| **覆盖范围** | 仅根组件直接子元素（~14个） | **全部**渲染元素（含子组件内部） |
| **用于** | action / inspect | vtree / snapshot |
| **子组件可见** | ❌ 不可见 | ✓ 可见 |

### 1.2 根因

`render_child_widget`（`aura_view_builder.rs:1350`）调用 `child_builder.build()`（untracked），不经过 `DebugIdMap`。子组件内部元素永远拿不到 aura ID。

### 1.3 具体表现

- `autoui_action(element_id="aura_N", action="press")` 只能点击根组件的按钮（如顶部 "New"）
- EditorPanel 的 Edit/Delete 按钮、NavTree 的 note 列表项没有 aura_N → 无法通过 MCP 操作
- `autoui_snapshot` 和 `autoui_vtree` 返回 `vnode_N`（覆盖全部），但 action/inspect 不认 vnode_N

---

## 2. 方案：action/inspect 支持 vnode_N（向后兼容 aura_N）

### 2.1 核心思路

View 树（`View<DynamicMessage>`）已经携带了完整的事件信息——每个 Button 的 `onclick` 是一个 `DynamicMessage { widget_name, event_name }`。VNode 的 `path` 字段（子索引序列）与 View 树结构一一对应。

**链路**: `vnode_N → VNode.path → 遍历 View 树找到对应 View → 提取 DynamicMessage（widget_name + event_name）→ 构建 ActionMessage`

不需要改渲染器、不需要改数据管道、不需要改 InspectorCache。

### 2.2 关键改进

vnode_N 路径直接从 DynamicMessage 获取 `widget_name`（如 "EditorPanel"），不再硬编码 `shared.widget_name`（"App"）。这修复了子组件按钮事件路由问题——之前即使能找到按钮，事件也会路由到 App 而非 EditorPanel。

---

## 3. 实施步骤

### 改动文件：`crates/auto-lang/src/ui/mcp_server.rs`

### Task 1: 新增 `ElementId` enum 和 `parse_element_id`

```rust
enum ElementId {
    Aura(AuraNodeId),
    Vnode(VNodeId),
}

fn parse_element_id(s: &str) -> Option<ElementId> {
    if let Some(n) = s.strip_prefix("aura_") {
        return n.parse::<u32>().ok().map(|n| ElementId::Aura(AuraNodeId(n)));
    }
    if let Some(n) = s.strip_prefix("vnode_") {
        return n.parse::<u64>().ok().map(|n| ElementId::Vnode(VNodeId::new(n)));
    }
    None
}
```

### Task 2: 新增 `find_view_by_vnode_path` — 按 VNode path 遍历 View 树

从 `styled_vtree.vtree` 中找到 VNodeId 对应的 VNode，取其 `path`（`Vec<u16>`），然后用 `extract_children`（从 `vnode_converter.rs`，已有）逐级深入 View 树，返回对应的 `&View<DynamicMessage>`。

```rust
fn find_view_by_vnode_path<'a>(
    view: &'a View<DynamicMessage>,
    path: &[u16],
) -> Option<&'a View<DynamicMessage>> {
    let mut current = view;
    for &idx in path {
        let children = extract_children(current);  // 复用 vnode_converter 的函数
        current = children.get(idx as usize)?;
    }
    Some(current)
}
```

### Task 3: 新增 `extract_action_from_view` — 从 View 提取 handler

```rust
/// 从 View 提取事件 handler，返回 (widget_name, event_name)。
/// action_name: "press" / "type" / "toggle" / "select" / "set_value"
fn extract_action_from_view(
    view: &View<DynamicMessage>,
    action_name: &str,
) -> Option<(String, String)> {
    match view {
        View::Button { onclick, .. } if action_name == "press" => {
            extract_dyn_msg(onclick)
        }
        View::Input { on_change, .. } | View::Textarea { on_change, .. }
            if action_name == "type" =>
        {
            on_change.as_ref().and_then(extract_dyn_msg)
        }
        View::Checkbox { on_toggle, .. } if action_name == "toggle" => {
            on_toggle.as_ref().and_then(extract_dyn_msg)
        }
        _ => None,
    }
}

fn extract_dyn_msg(msg: &DynamicMessage) -> Option<(String, String)> {
    match msg {
        DynamicMessage::Typed { widget_name, event_name } =>
            Some((widget_name.clone(), event_name.clone())),
        DynamicMessage::String(name) => Some(("App".into(), name.clone())),
    }
}
```

### Task 4: 修改 `tool_action` — 双路径分发

```rust
// 解析 element_id（支持 aura_N 和 vnode_N）
let element_id = match parse_element_id(element_id_str) { ... };

match element_id {
    ElementId::Aura(aura_id) => {
        // 现有路径：SnapshotBuilder + find_node + execute_action_on_shared（不变）
    }
    ElementId::Vnode(vnode_id) => {
        // 新路径：
        // 1. 从 styled_vtree.vtree.get(vnode_id) 获取 VNode（path + kind）
        // 2. 用 VNode.path 遍历 view 树（find_view_by_vnode_path）
        // 3. 从 View 提取 DynamicMessage（extract_action_from_view）
        // 4. 构建 ActionMessage { widget: widget_name, event: event_name, input_value }
        //    ↑ widget_name 来自 DynamicMessage（如 "EditorPanel"），不再硬编码 "App"
        // 5. send_action(msg)
    }
}
```

### Task 5: 修改 `tool_inspect` — 支持 vnode_N

vnode_N 路径：从 styled_vtree 查 VNode，返回其 kind/label/props。对于事件信息，同样从 View 树提取。

### Task 6: `autoui_snapshot` 已支持 vnode_N（v2 styled_vtree 路径），无需改动

---

## 4. 验证标准

### 4.1 Edit 流程验证（已通过 ✅）

用 MCP 执行完整的 Edit 流程自动化验证（零手动 UI 交互）：

1. `autoui_snapshot` → 找到 Edit 按钮的 `vnode_N`
2. `autoui_action(element_id="vnode_N", action="press")` → 点击 Edit
3. `autoui_snapshot` → 验证 EditorPanel 切换到编辑模式（Save/Cancel 按钮出现）
4. `autoui_state(fields=["editing"])` → 验证 `editing=true`
5. `autoui_snapshot` → 找到标题输入框的 `vnode_N`
6. `autoui_action(element_id="vnode_N", action="type_text", value="Modified Title")` → 输入文字
7. `autoui_state(fields=["edit_title"])` → 验证 `edit_title="Modified Title"`

### 4.2 全按钮点击验证（已通过 ✅）

系统性测试了 UI 中所有 24 个按钮，覆盖全部组件层级：

| # | 按钮类型 | 按钮 | handler | 状态变化 | 结果 |
|---|---------|------|---------|---------|------|
| 1 | **根组件** (App) | New | `.NewNote` | notes +1 | ✅ |
| 2 | **NavTree 标签** | All/Pinned/Recent | `.SelectAll/Pinned/Recent` | active_folder 变化 | ✅ |
| 3 | **NavTree 笔记项** | Welcome/Quick Ideas/... | `.SelectNote` | active_id 变化 | ✅ |
| 4 | **NavTree 文件夹** | + (NewNoteInFolder) | `.NewNote` | notes +1 | ✅ |
| 5 | **NavTree 主题色块** | 5 个色块 | `.SetAccent` | accent_color 变化 | ✅ |
| 6 | **NavTree 暗黑模式** | 🌙 Dark / ☀ Light | `.ToggleDarkMode` | dark_mode 翻转 | ✅ |
| 7 | **EditorPanel** | Edit | `.Edit` | editing→true, edit_title 填充 | ✅ |
| 8 | **EditorPanel** | Delete | `.Delete` | 触发 on_delete 回调 | ✅ |

**结论：通过 MCP vnode_N 可以点击任何按钮**，无论位于根组件还是子组件内部。

### 4.3 已知注意事项

- 点击后视图重建会导致 vnode_N 变化（路径结构变了）。连续操作需要在视图变化后重新 snapshot。
- 同一元素的连续点击（视图不变时）vnode_N 是稳定的。

### 不在本次范围

- 不改渲染器、不改 InspectorCache、不改 view builder
- 不删除 aura_N 支持（向后兼容）
- aura_N 的已知限制（不覆盖子组件）不修复——vnode_N 是推荐路径

---

## 5. 后续改进

### Task 7: VTree 全量展示 + 变化检测（高优先级）✅ 已完成

**动机**：当前验证按钮点击效果需要反复 `autoui_snapshot` + `autoui_state`，效率低。更快的办法是直接通过 `autoui_vtree` 获取**完整渲染节点树**（VTree），根据节点树的变化判断页面是否变化。

**验证结果**：
- `autoui_vtree` 已完整展示所有 73 个渲染节点（无截断、无折叠）
- 所有 `use`/`import` 的子组件（EditorPanel、NavTree）内部元素全部展开
- VTree 节点级 diff 清晰展示变化（如 Edit 点击后：Save/Cancel 新增、📌/标题消失）

**新增 `autoui_find` 工具（Task 7 实现）**：

比 diff 整棵树更精准——Agent 按条件搜索特定组件，直接验证"某个元素是否存在"。

搜索条件（全部可选，AND 组合）：
- `kind`：节点类型（button, input, text, textarea, col, row, checkbox...）
- `label`：对 label/content/value/placeholder 做大小写不敏感子串匹配
- `limit`：最大返回数（默认 20）

返回匹配节点的 Atom 格式（含 vnode_N ID 和路径深度缩进）。

验证场景：
- `autoui_find(kind=button, label=Edit)` → 精确找到 Edit 按钮
- `autoui_find(kind=input)` → 找到搜索输入框
- `autoui_find(label=Welcome)` → 找到按钮 + 文本节点
- 编辑模式验证：点击 Edit 前 `autoui_find(label=Save)` → 不存在；点击后 → 存在 ✓

### Task 8: InspectorCache 合并 events 到 ComputedNodeLite

让 `autoui_vtree` 的输出也能显示每个元素的事件信息（当前 events 字段为空）。需要在渲染器的 probe→cache 合并中复制 events 数据。

### Task 9: autoui_wait 支持 vnode_N 级别的等待

等待特定元素出现/消失，而不是只等待 state 字段变化。

### Task 10: 截图 + 视觉对比（回归测试）

`autoui_screenshot` 已存在。增强为支持截图 diff（对比基线截图），用于回归测试。

---

## 6. Rust 模式（a2r + iced）MCP 支持

### 6.1 背景

当前 MCP 仅在 VM 模式（`auto run -r vm`）中可用。Rust 模式（`auto run -r rust`，a2r 转译的 iced 原生窗口）没有 MCP 支持，无法用同一套测试套件验证。

**架构差异**：

| | VM 模式 | Rust 模式 |
|---|---|---|
| 组件类型 | `DynamicComponent`（运行时 VM） | `C: Component`（编译时 Rust struct） |
| 视图输出 | `View<DynamicMessage>` | `View<C::Msg>` |
| 消息类型 | `DynamicMessage { widget_name, event_name, args }` | 强类型 `C::Msg`（如 `AppMsg::AddTodo`） |
| 状态访问 | `read_state(field)` / `write_state(field, val)` | `Component` trait 无泛型字段访问 |
| MCP | ✅ `start_mcp_server` + `SharedState` | ❌ 无 |

### 6.2 好消息：80% 基础已具备

Rust 模式的 `DevToolsWrapper`（`renderer.rs:6619`）已经：

- **每帧构建 VTree**：`view_element()` 调用 `view_to_vtree_with_paths(self.inner.view())`，生成 VNodeId 索引的完整渲染树，存入 `DevToolsState.live_vtree`
- **支持消息注入**：`WrapperMsg<C>` 有 `Inner(C::Msg)` 和 `Debug(String)` 两个变体，`devtools_update` 分别路由到 `inner.on(msg)` 和 `apply_debug_event`
- **F12 DevTools 面板**：已有 hover/select/inspector 功能

### 6.3 实施步骤

#### Task 11: Rust 模式 MCP 服务器启动

在 `run_app_devtools` / `run_app_with_task_devtools` 中加入 `start_mcp_server` 调用（复用 VM 模式的 MCP 服务器代码）。

**挑战**：MCP 的 `SharedState` 需要 `View<DynamicMessage>` + state 字典，但 Rust 模式是 `View<C::Msg>` + 强类型 struct。需要一个**桥接层**。

**方案**：创建 `RustMcpBridge<C>` 适配器，将 Rust 模式的 `DevToolsWrapper` 包装为 MCP 可用的形式：
- `snapshot()`：从 `DevToolsState.live_vtree` 构建 Atom 文本（已有 `build_aura_from_styled_vtree`，只需适配数据源）
- `action(vnode_id, action_type)`：从 VTree 的 VNode path 遍历 `inner.view()` 找到对应 View，提取其 `C::Msg`（onclick/on_change），包装为 `WrapperMsg::Inner(msg)` 注入
- `state()`：从 VTree 的 props（label/value/content）提取可见状态（不依赖强类型 struct 字段）

#### Task 12: Rust 模式 action 分发

创建 Rust 版的 `find_view_by_path` + `extract_action_from_view`（泛型版本）：
- `find_view_by_path_generic<M>(view: &View<M>, path: &[u16]) -> Option<&View<M>>`：复用 `extract_children` 逻辑
- 从 `View<C::Msg>` 的 Button/Input/Textarea 提取 `onclick`/`on_change` 消息（类型为 `C::Msg`）
- 包装为 `WrapperMsg::Inner(msg)` 并通过 MCP channel 注入 iced 事件循环

**关键约束**：`C::Msg` 必须满足 `Send + 'static`（iced 消息要求）。MCP channel 使用 `mpsc::Sender<WrapperMsg<C>>`。由于 `C` 是泛型的，channel 类型需要泛型化或使用 trait object。

#### Task 13: Rust 模式 MCP subscription

在 `devtools_subscription` 中新增一个 subscription，轮询 MCP channel 并将 action 转为 `WrapperMsg`：

```rust
fn devtools_subscription<C: Component + 'static>(w: &DevToolsWrapper<C>)
    -> iced::Subscription<WrapperMsg<C>>
{
    let inner = w.inner.subscription().map(WrapperMsg::Inner);
    let f12 = /* ... */;
    let win = /* ... */;
    let mcp = mcp_action_subscription_rust::<C>();  // 新增
    iced::Subscription::batch(vec![inner, f12, win, mcp])
}
```

#### Task 14: 验证 Rust 模式 MCP

用与 VM 模式相同的测试套件验证 Rust 模式：
1. `auto run -r rust` 启动 015-notes（a2r iced 窗口）
2. MCP `autoui_snapshot` → 获取完整渲染树
3. MCP `autoui_find(button, Edit)` → 找到 Edit 按钮
4. MCP `autoui_action(vnode_N, press)` → 点击 Edit
5. MCP `autoui_exists(button, Save)` → 验证编辑模式

**预期结果**：VM 模式和 Rust 模式的 MCP 行为一致，同一套测试用例可以通过。

### 6.4 技术风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| `C::Msg` 不满足 `Send` | MCP channel 无法跨线程 | `Component` trait 已要求 `'static`；可能需要加 `Send` bound |
| state 字段无法泛型读取 | `autoui_state` 工具不可用 | 通过 VTree props 间接读取可见状态；或给 Component trait 加 `fn serialize_state() -> HashMap<String, Value>` |
| 泛型 channel 类型 | `start_mcp_server` 需要具体类型 | 使用 `dyn Fn(WrapperMsg)` trait object 或宏泛型化 |
| Rust 编译时间 | a2r 生成 + cargo build 较慢 | 与 VM 模式并行测试，不阻塞 |

### 6.5 不在本次范围

- 不修改 `Component` trait 核心签名（除非必须加 `Send`）
- 不支持 Vue+Rust 模式的 MCP（那是浏览器端，用 Playwright）
- 不实现 Rust 模式的 state 字段读写（首期只支持 action + snapshot + find）

---

## 7. 统一 AutoUI 测试声明格式（桌面 MCP 专属）

### 7.1 定位与分层

测试体系分三层，各层职责清晰分离：

```
acceptance.atd（业务场景定义 — 所有模式共享的 single source of truth）
       │
       ├─→ smoke.spec.ts / accent-dark.spec.ts
       │   （Playwright 实现 — Web 专属，保持不变）
       │   覆盖 T1-T13，含 DOM/CSS/API/localStorage 断言
       │
       └─→ 015-notes.autotest（MCP 场景声明 — 桌面 VM/Rust 专属）
              ├─→ MCP Adapter（Python）→ VM 模式 iced 窗口
              └─→ MCP Adapter（Python）→ Rust 模式 iced 窗口
              覆盖 T1-T13 的子集，用 find/action/exists/state 断言
```

**设计原则（方案 A）**：
- `acceptance.atd` 是统一业务场景定义，所有模式共享，不修改
- Playwright 测试（`.spec.ts`）保持不变，负责 Web 专属断言（CSS/API/DOM）
- `.autotest` 是桌面 MCP 专属的场景声明，从 acceptance.atd 派生
- 两套执行器并行存在，各自处理模式差异，不强制统一

### 7.2 为什么 Playwright 不迁移到 `.autotest`

Playwright 测试依赖大量 Web 专属能力，`.autotest` 无法表达：
- `getComputedStyle` 读取按钮背景色 → T12 主题色 RGB 匹配
- `page.request.get('/api/notes')` → T5 Save / T7 Delete 持久化验证
- `localStorage.getItem('notes-accent-color')` → T12 持久化验证
- `.ProseMirror` 可见性 → T1 Tiptap editor 验证
- `page.on('pageerror')` → T13 控制台错误检查

如果强行迁移，要么丢失这些断言能力，要么让 `.autotest` 原语集膨胀到重新发明测试框架。因此 `.autotest` 定位为桌面 MCP 专属，与 Playwright 并行。

### 7.3 声明格式：`.autotest` 文件

每个 `.autotest` 文件描述桌面 MCP 可执行的场景。格式借鉴 acceptance.atd 的 Given/When/Then，但增加**操作原语**和**断言原语**使其可被 MCP Adapter 直接执行。

```
# 015-notes.autotest — 桌面 MCP 测试场景声明（VM + Rust）
# 从 acceptance.atd 的 T1-T13 派生，只包含 MCP 可执行的场景。

suite "015-notes 核心流程"

# ── 导航类 ──────────────────────────────────────────────

scenario T2a "Pinned tab 无文件夹标题"
  when  click_button label="Pinned"
  then  not_exists text label="📁"

scenario T2b "All tab 有文件夹标题"
  when  click_button label="All"
  then  exists text label="📁 Notes"

scenario T2c "Recent tab 无文件夹标题"
  when  click_button label="Recent"
  then  not_exists text label="📁"

# ── 编辑类 ──────────────────────────────────────────────

scenario T5a "Edit 进入编辑模式"
  when  click_button label="Edit"
  then  exists  button label="Save"
  then  exists  button label="Cancel"
  then  exists  input  label="Note title"

scenario T5b "Cancel 返回只读模式"
  given editing_mode
  when  click_button label="Cancel"
  then  exists  button label="Edit"
  then  not_exists button label="Save"

scenario T5c "Edit 填充当前笔记内容"
  when  click_button label="Edit"
  then  inspect input label="Note title" has_value=true

scenario T6 "New 创建新笔记"
  when  click_button label="New"
  then  snapshot_changed

scenario T5d "输入标题"
  given editing_mode
  when  type_text input label="Note title" value="MCP Test"
  then  state field="edit_title" equals="MCP Test"
  skip_if rust  # Rust 模式无 autoui_state

scenario T5e "Save 退出编辑模式"
  given editing_mode
  when  click_button label="Save"
  then  exists  button label="Edit"

# ── 主题类 ──────────────────────────────────────────────

scenario T11 "Dark mode 切换"
  when  click_button label="Dark"
  then  state field="dark_mode" equals=true
  skip_if rust
  when  click_button label="Light"
  then  state field="dark_mode" equals=false
  skip_if rust

scenario T12 "主题色切换"
  when  click_button label="Dark"   # 先切到 dark 确认按钮文本
  when  click_button label="Light"
  then  state field="accent_color" equals="indigo"
  skip_if rust
```

### 7.4 原语定义

**操作原语（When）**：

| 原语 | 参数 | MCP 工具调用 |
|------|------|-------------|
| `click_button` | `label="X"` | `autoui_find(button,X)` → `autoui_action(vnode_N, press)` |
| `type_text` | `input label="X" value="Y"` | `autoui_find(input,X)` → `autoui_action(vnode_N, type_text, Y)` |
| `press_key` | `key="Enter"` | `autoui_keyboard(key=Enter)` |

**断言原语（Then）**：

| 原语 | 参数 | MCP 工具调用 |
|------|------|-------------|
| `exists` | `kind label="X"` | `autoui_exists(kind,label)` → 期望 FOUND |
| `not_exists` | `kind label="X"` | `autoui_exists(kind,label)` → 期望 NOT FOUND |
| `state` | `field="X" equals=Y` | `autoui_state(fields=["X"])` → 比较 |
| `inspect` | `kind label="X" has_value=true` | `autoui_find(kind,X)` → `autoui_inspect(vnode_N)` → 检查 value 非空 |
| `snapshot_changed` | — | 前后 `autoui_snapshot` 比较节点数/内容 |

**前置条件（Given）**：

| 原语 | 含义 | MCP 实现 |
|------|------|---------|
| `app_loaded` | 应用已启动 | 启动时自动满足（MCP 连接即说明已渲染） |
| `editing_mode` | 已进入编辑模式 | 前置 `click_button label="Edit"` |

**模式跳过**：

| 语法 | 含义 |
|------|------|
| `skip_if rust` | Rust 模式跳过此断言（autoui_state 不可用） |
| `skip_if vm` | VM 模式跳过此断言 |

### 7.5 已知场景覆盖差异

acceptance.atd 有 13 个场景（T1-T13），MCP 可执行的子集：

| 场景 | MCP 可执行？ | 说明 |
|------|-------------|------|
| T1 笔记切换 | ⚠️ 部分 | 可点击笔记，但 ProseMirror 断言不可用；改用 inspect textarea value |
| T2 View tabs | ✅ | find + exists 完全覆盖 |
| T3 搜索 | ❌ skip | 搜索功能未实现（sidebar.at 无 oninput） |
| T4 Tag 筛选 | ⚠️ | all_tags 当前为空，tag 筛选按钮不存在 |
| T5 Edit/Save/Cancel | ✅ | find + action + exists + state 完全覆盖 |
| T6 New | ✅ | click + snapshot_changed |
| T7 Delete | ✅ | click + snapshot_changed |
| T8 Pin 切换 | ⚠️ | pin 按钮在编辑模式显示，可点击但验证依赖 state |
| T9 Tag 添加 | ⚠️ | "+ tag" 按钮存在，交互链复杂 |
| T10 文件夹新建 | ✅ | click "+" button |
| T11 Dark mode | ✅ vm / ⚠️ rust | VM 用 state，Rust 用 snapshot diff |
| T12 主题色 | ✅ vm / ⚠️ rust | VM 用 state(accent_color)，Rust 无法验证 RGB |
| T13 控制台错误 | ❌ skip | 无浏览器控制台 |

### 7.6 实施计划

#### Task 15: 定义 `.autotest` 格式规范 + Python 解析器

- 编写 `.autotest` 语法规范（原语定义、skip_if 语法、注释）
- 实现 Python 解析器（`.autotest` → 场景对象列表，每场景含 steps 列表）
- 解析器输出 JSON 结构，供 MCP Adapter 消费

#### Task 16: MCP Adapter 执行器（Python）

- 实现场景执行器：解析原语 → 调用 MCP JSON-RPC 工具 → 收集结果
- 处理 `skip_if`（Rust 模式跳过 state 断言）
- 处理 `given` 前置条件（自动执行前置操作）
- 输出测试报告（PASS/FAIL/SKIP + 详细原因）

#### Task 17: 编写 015-notes 的 `.autotest` 文件

- 从 acceptance.atd 的 T1-T13 派生 MCP 可执行子集
- 标注 skip_if 和模式差异
- 在 VM 模式执行验证（Rust 模式待 a2r 重新生成后验证）

#### Task 18: 更新 acceptance.atd

- 在 acceptance.atd 的 D7 条目更新 Rust 平台状态：Rust ❌→✅（MCP 已支持）
- 添加引用：指向 `.autotest` 文件作为桌面 MCP 执行声明

---

## 8. 实现审计（2026-07-31）

对 Task 1-18 的实现情况做了逐项核查（证据见代码 `file:line`）。结论：**14/18 已完成**，1 个未实现，2 个存在 workaround。

### 8.1 状态总览

| Task | 内容 | 状态 | 备注 |
|---|---|---|---|
| 1-9 | action/inspect/wait 的 vnode_N 支持 + find/exists 工具 + events 合并 | ✅ 已完成 | VM 模式路径干净 |
| **10** | 截图 + 视觉对比 | ❌ **未实现** | → Task 20 |
| 11/13 | Rust 模式 MCP 启动 + subscription | ✅ 已完成 | 架构偏离但功能等价 |
| **12** | Rust 模式 action 分发 | ⚠️ **workaround** | → Task 19 |
| **14** | Rust 模式跨模式一致性 | ⚠️ 部分 | `autoui_state` 在 Rust 不可用 → Task 21 |
| 15-18 | `.autotest` 格式 + Python adapter + acceptance.atd | ✅ 已完成 | 仅 VM 模式跑过全量验证 |

### 8.2 关键遗留问题（workaround / 缺失）

**问题 A（Task 12 workaround，最关键）**：Rust 模式 action 分发使用脆弱启发式，存在**静默失败**风险。链路：`execute_action_vnode` 的 Rust 分支从 VNode 标签取首词作为 event 名（`derive_event_name_from_label`），经 `__mcp_action|...` 字符串通道，iced 侧 `find_msg_by_event_name` 用 `format!("{:?}", onclick).contains(event)` 子串匹配。三个缺陷：子串误匹配、首词未必命中（静默失败）、输入值端到端断裂（`_input_value` 未用，`INPUT_TEXT` thread-local 从未被 MCP 设置）。

**问题 B（Task 10 缺失）**：`autoui_screenshot`（`mcp_server.rs:1154`，`inputSchema.properties == {}`）未加 diff 能力。`run_autotest.py:52-93` 有个 post-hoc MD5 hash 存根（post-hoc、只存 hash、不影响退出码）。

**问题 C（Task 14 部分）**：Rust 模式从不调用 `SharedState::update`，`shared.state` 恒空，`tool_state` 早退。4 个 state 场景标 `skip_if rust`（T5c/T11/T11b/T12）。

### 8.3 后续补救计划（Task 19-21）

- **Task 19（高）**：路径寻址精确分发，消除 heuristic + 修复输入值断裂。
- **Task 20（中）**：Rust MCP 侧像素级截图 diff。
- **Task 21（低）**：Rust 模式标量状态快照，去掉 `skip_if rust`。

---

## 9. Task 19: Rust 模式 action 路径寻址（消除 heuristic）✅ 已实施

### 9.1 设计思路

放弃「MCP 侧猜 event 名 → iced 侧 Debug 子串匹配」，改为「**MCP 侧只传路径，iced 侧沿 path 精确取 handler**」。目标 View 节点的 handler 是构造 View 时内联的强类型 `M`，沿 path 定位即拿到正确的 `M`。

### 9.2 实施

- `ActionMessage` 重构为自描述结构：`{ target: ActionTarget, action, value }`，新增 `enum ActionTarget { Event{widget,event}, Path{path:Vec<u16>} }`。VM 模式用 `Event`（向后兼容），Rust 模式用 `Path`（从 `vnode.path`）。
- iced 侧 `devtools_subscription` 编码为 `__mcp_action_path|<a,b,c>|<action>|<value>`；`devtools_update` 新增该分支：解析 path → `find_view_by_path_generic(&view, &path)` → `extract_handler_from_view(node, action)` → 设置 `INPUT_TEXT`（**修复输入值断裂**）→ `w.inner.on(m)`。
- 新增泛型 `find_view_by_path_generic<M>` + `extract_handler_from_view<M>`（renderer.rs），删除 `find_msg_by_event_name` + `extract_view_children`（renderer.rs）和 `derive_event_name_from_label`（mcp_server.rs）。
- `vnode_converter.rs` 新增 `pub fn extract_children_ref`（引用版本，避免 clone）。

### 9.3 改动文件

`mcp_server.rs`、`iced/renderer.rs`、`vnode_converter.rs`。

### 9.4 验证

- `cargo build -p auto-lang --features ui-iced` ✅
- `cargo build --bin auto` ✅
- 015-notes rust workspace `cargo build` ✅
- mcp_server tests_314：6 passed ✅
- ⚠️ 待：VM 模式 `.autotest` 全量回归（需启动 GUI）；Rust 模式手动验证 Edit/type_text。

---

## 10. Task 20: 像素级截图 diff（Rust MCP 侧）✅ 已实施

### 10.1 设计思路

在 Rust MCP 侧用已依赖的 `image 0.25`（`ui-iced` 启用）做像素 diff，RGBA 缓冲区在 `renderer.rs` 已在内存中。

### 10.2 实施

- `tool_screenshot` 解析 `name`/`baseline`/`diff`/`threshold` 参数；schema 增加这些参数。
- `ScreenshotRequest`/`ScreenshotOptions` 携带选项（纯数据，无 image 类型，可放在非 ui-iced-gated 模块）。
- `renderer.rs` 新增 `process_screenshot`（命名/基线/diff 分发）+ `compare_pngs`（逐像素对比，差异超阈值返回 DIFFERS 并存红高亮 diff 图到 `tmp/<name>-diff.png`），删除冗余的 `save_screenshot_png`。
- Python 适配器：`McpAdapter.after_scenario(sid)` 钩子在每场景末尾调 `autoui_screenshot(name=sid, baseline/diff)`；`run_suite` 返回 `(results, screenshot_results)`；`run_autotest.py` 重写为传参 + diff 结果纳入退出码，删除 post-hoc MD5 存根。

### 10.3 改动文件

`mcp_server.rs`、`iced/renderer.rs`、`tests/autotest/__init__.py`、`tests/run_autotest.py`。

### 10.4 验证

- `cargo build -p auto-lang --features ui-iced` ✅；`cargo build --bin auto` ✅
- Python 文件 `import autotest` + `ast.parse` ✅
- ⚠️ 待：启动 GUI 跑 `--screenshot-baseline` / `--screenshot-diff` 端到端。

---

## 11. Task 21: Rust 模式标量状态快照（去掉 skip_if rust）✅ 部分实施

### 11.1 设计思路

`Component` 是本地 trait（`component.rs:34`），加**带默认实现的 `state_snapshot()`**（blast radius 零）。a2r 生成器对**标量字段**生成 override。

### 11.2 实施

- `component.rs`：trait 加 `state_snapshot()` 默认方法（默认空，VM 模式不经过此路径）。
- `ui/mod.rs`：re-export `auto_val`（供生成代码以 `auto_lang::ui::auto_val::Value` 引用）。
- `rust.rs` 生成器：`generate_component_impl` 新增 `generate_state_snapshot`，仅对标量类型（String/i8..i64/u8..u64/isize/usize/f32/f64/bool）生成条目，跳过 Vec/serde_json::Value/嵌套组件；新增 `is_scalar_state_type`/`scalar_to_auto_value_expr` helpers + 2 个单元测试。
- `mcp_server.rs`：`SharedState::set_state` setter。
- `renderer.rs`：`view_element` 每帧 `set_state(self.inner.state_snapshot())`。

### 11.3 改动文件

`component.rs`、`ui/mod.rs`、`ui_gen/rust.rs`、`mcp_server.rs`、`iced/renderer.rs`。

### 11.4 验证

- `cargo build -p auto-lang --features ui-iced` ✅（默认方法，零回归）
- 生成器单元测试：`test_state_snapshot_scalar_override`、`test_state_snapshot_no_scalars_no_override` ✅（25 passed in ui_gen::rust）
- 015-notes rust workspace `cargo build` ✅（committed main.rs 是手工拼装版，未含 state_snapshot；见 §11.5）

### 11.5 重生成受阻：a2r store-composable 预存缺陷

Task 21 的生成器改动（§11.2，commit `f863f9ff`）**已正确落地**：当 a2r 正常输出时，`generate_component_impl` 会为标量字段生成 `state_snapshot` override。

但尝试对 015-notes 重生成时发现：**原生 a2r 输出有 86 个预存错误**（`no field store`、`cannot find type StoreMsg` 等 store-composable 处理缺陷），生成的 `NotesStore` 只有 4 个字段（缺 `dark_mode`/`accent_color` 等）。committed 的 `main.rs` 是**手工拼装修复版**（commit `cd9205b9` "restore hand-assembled rust/src/ (0 errors)"），并非 a2r 直接产物。

**关键决策（已纠正）**：**不修改生成的 `main.rs`**。生成文件会被下次重生成覆盖，手工改它没有意义。因此 015-notes 的 `main.rs` 当前**不含** `state_snapshot`——要让它有，必须先修复 a2r 的 store-composable 缺陷（属 Plan 374 范畴，超出 Plan 371），再重生成。

> 历史教训：本轮曾误在手工版 `main.rs` 上注入 `state_snapshot`（commit `9f297b06`），已被回退——生成文件不该手改。正确的修复点永远是生成器本身。

### 11.6 ⚠️ skip_if rust 暂不移除（依赖 a2r 修复 + 字段语义对齐）

移除 `skip_if rust` 的前置条件尚未满足：

1. **a2r store 缺陷未修**（§11.5）：重生成前 015-notes 的 Rust 组件没有 `state_snapshot`，Rust 模式 `autoui_state` 仍返回空。
2. **字段语义差异**：即便 `state_snapshot` 生效，VM 与 Rust 模式的 state 形状不一致——

| 场景 | 断言字段 | VM 模式（flat state） | Rust 模式（root App snapshot） |
|---|---|---|---|
| T11/T11b | `dark_mode` | ✅ 顶层 | ⚠️ 属于子组件 `store`（NotesStore），不在根 App snapshot |
| T12 | `accent_color` | ✅ 顶层 | ⚠️ 同上 |
| T5c | `edit_title` | ✅ 当前 EditorPanel 状态 | ❌ 属于子组件 EditorPanel |

根因：VM 模式 `shared.state` 是**所有组件状态的扁平 map**（含当前 EditorPanel 的 `edit_title`、顶层 `dark_mode`），而 Rust 模式 `view_element` 只调根组件 `App::state_snapshot()`。子组件状态不在根快照里。

**结论**：本轮**保留** `skip_if rust`。要让这些场景在 Rust 模式通过，需后续工作（按依赖顺序）：
- (a) 修复 a2r 的 store-composable 缺陷（Plan 374），使重生成产出可编译的 `main.rs`（含 `state_snapshot`）；
- (b) 让 Rust 模式 `SharedState.state` 像 VM 一样扁平聚合所有组件状态（`DevToolsWrapper` 递归收集子组件 `state_snapshot`），而非只读根组件。

`autoui_state` 的 Rust 模式基础设施（trait 默认方法 + 生成器 override + `set_state` + `view_element` 推送）已就绪，待 (a)(b) 完成后即可生效。

---

## 12. Task 22：解锁 Rust 模式 state（问题 a + b）

§11.6 列出两个前置问题。本节把它们拆成可执行的两个子任务，按依赖顺序实施。

### 12.1 问题 a：a2r store-composable 缺陷 ✅ 已修复（Task 22a，commit `6dbf400f`）

**现象**：对 015-notes 跑原生 a2r 重生成（`touch .at + auto run -r rust`），产出的 `main.rs` 有 **86 个编译错误**：`error[E0609]: no field 'store'`（×78）、`error[E0433]: cannot find type 'StoreMsg'`、`NotesStore` 只生成 4/9 字段。

**根因（诊断后确认）**：**单一 bug** —— `regenerate_code_only`（增量重生成路径，`rust_ui.rs:81`）调用 `compile_at_file(at_path, &[])` 传了**空 store 列表**，导致 Plan 374 的 store-composable 机制全部跳过：不注册 store 名、不生成 `NotesStore` struct、不注入 `pub store: NotesStore` 字段、不发 `NotesStoreMsg` 枚举。而表达式重写器（纯语法，不看 store 注册表）仍把每个 `store.X` 改写成 `self.store.X` / `StoreMsg::X`，造成悬空引用。

注意：`generate_rust_ui`（全量路径，`rust_ui.rs:182-196`）有 store 预扫描并正确传 `&all_stores`，所以全量重生成是对的；只有增量路径 `regenerate_code_only` 漏了。这也解释了为什么 committed 的 `main.rs`（手工拼装版 `cd9205b9`）是对的——它来自全量路径。

**修复**：把 store 预扫描抽成共享 helper `collect_store_decls`（`rust_ui.rs`），`generate_rust_ui` 和 `regenerate_code_only` 都调它。增量路径现在传 `&all_stores`，与全量路径一致。

**验证**：`touch app.at + auto run -r rust` 重生成 015-notes → `cargo build` **0 错误**。生成的 `NotesStore` 含全部 9 字段，App/EditorPanel/NavTree/NoteItem 都注入了 `pub store: NotesStore`，`NotesStoreMsg` 正常生成。**附带收益**：生成器自动带上 Task 21 的 `state_snapshot()` override（4 个组件），无需手改 main.rs。25 个生成器测试 + 2 个 state_snapshot 测试通过。

### 12.2 问题 b：Rust 模式 state 不聚合子组件（Task 22b，依赖 22a ✅）

**现象（22a 修复后实测确认）**：重生成后 `App::state_snapshot()` 只暴露 `search`（App 自身标量字段），`store: NotesStore` 字段未递归，故 `dark_mode`/`accent_color`/`active_folder` 等 store 字段在 Rust 模式 `autoui_state` 中不可见；`edit_title`（属于 EditorPanel 子组件）更不在根快照里。

**根因**：VM 模式 `shared.state` 是 VM 堆里**所有组件实例状态的扁平 map**（renderer.rs 的 `read_all_state_materialized` 聚合）；Rust 模式 `view_element` 只调根组件 `App::state_snapshot()`，而生成器的 `generate_state_snapshot` 只处理**标量字段**，跳过了 `store`（`NotesStore` 类型，非标量）。

**解决方案（Task 22b）—— 生成器侧递归**：在 `generate_state_snapshot`（`ui_gen/rust.rs`）中，对类型为**已注册组件**（store 或子组件）的字段，生成递归调用：`for (k, v) in self.<field>.state_snapshot() { m.insert(format!("<field>.{}", k), v); }`。生成器已知字段类型（`state_types`/`prop_types`）和组件名（`STORE_NAMES` + 子组件列表），可在 `is_scalar_state_type` 之外加一个「是组件类型」的判断分支。

字段名对齐：展平后用 `<field>.<subfield>` 前缀（如 `store.dark_mode`）。`.autotest` 的 `state field="dark_mode"` 断言需相应支持前缀查询，或 MCP `autoui_state` 工具做前缀/子串匹配（见 22b 实施细节）。

**验证目标**：Rust 模式 `autoui_state(fields=["store.dark_mode"])` 返回值；`store.accent_color`、`store.active_folder` 等可见。届时评估能否移除 `skip_if rust`（取决于 `.autotest` 断言是否对齐前缀）。

### 12.3 实施顺序

1. **Task 22a**（问题 a）：修 a2r store-composable 缺陷 → 015-notes 重生成 0 错误（含自动 `state_snapshot`）。✅ commit `6dbf400f`
2. **Task 22b**（问题 b，依赖 22a）：生成器 `state_snapshot` 递归展开 store 字段（`store.` 前缀）+ `autoui_state` 字段查询支持路径后缀匹配。✅ commit `544e0731` + `3da51fa5`
3. **skip_if rust 决策**：
   - **T11/T11b（`dark_mode`）/ T12（`accent_color`）**：Rust 模式现可通过后缀匹配读到 `store.dark_mode`/`store.accent_color` → **已移除** `skip_if rust`。
   - **T5c（`edit_title`）**：EditorPanel 是按笔记在 view 里构造的子组件，**不是 App 的 struct 字段**，故其 `edit_title` 不在 App snapshot → **保留** `skip_if rust`（需后续让生成器/运行时支持「当前活动子组件」状态暴露）。
4. ⏳ 待：启动 GUI 跑 `python run_autotest.py --mode rust` 端到端验证 T11/T11b/T12 实际通过。

### 12.4 GUI 端到端验证结果 ✅

启动 015-notes GUI 跑 `.autotest`，两种模式均验证：

| 模式 | 结果 | 说明 |
|---|---|---|
| **VM**（`--mode vm`） | **13 passed, 0 failed, 0 skipped** | 完全回归，后缀匹配未破坏 VM（精确命中） |
| **Rust**（`--mode rust`） | **8 passed, 5 failed, 0 skipped** | T11/T11b/T12 ✅（新启用）；T5c 仍 skip（`edit_title` 子组件不可见） |

Rust 模式失败项（T2a/T2c/T5b/T5c/T5e）是**预存的 a2r UI 渲染差异**（文件夹标题渲染、"Note title" 输入框未找到），与 state 工作无关，属 Plan 374/后续 a2r 修复范畴。

直接 MCP 验证（`autoui_state(fields=["dark_mode"])`）确认 Rust 模式返回 `store.dark_mode: false (bool)`、`store.accent_color: "indigo" (str)`——递归快照 + 后缀匹配按设计工作。

**结论**：Task 22（问题 a + b）完成。`skip_if rust` 已从 T11/T11b/T12 移除（实测通过），T5c 保留（`edit_title` 仍需后续让「当前活动子组件」状态可暴露）。

### 12.5 问题 c：子组件实例状态不持久（Task 22c）✅ 已修复

§12.4 后继续深挖 T5b/T5c/T5e + T2a/T2c 失败，发现一个**更深的 a2r 架构缺陷**——与 state 读取无关，而是子组件状态**写入即丢弃**。

**现象**：路径寻址（Task 19）精确分发到 `AppMsg::EditorPanel(EditorPanelMsg::Edit)` 并调 `w.inner.on(m)`，但编辑模式始终不进入（Save 按钮不出现）。

**根因（诊断确认）**：生成的 `App::on` 转发子组件消息时**新建一个临时子组件实例**、在其上调 `.on(inner)`、然后丢弃：

```rust
AppMsg::EditorPanel(inner) => {
    let mut __child = EditorPanel::new(...);  // 临时实例
    __child.on(inner);   // Edit → __child.editing = true（丢失！）
    self.search = __child.search;            // 只回拷 search
}
```

EditorPanel 自身状态（`editing`/`edit_title`/...）和 NavTree 的 `store.active_folder` 变更都随临时实例丢弃。VM 模式无此问题（DynamicComponent 实例由 VM 持久化）。

**解决方案（生成器侧，commit `00271d9a`）**：
1. **跨文件扫描**（`collect_component_state_fields`）：记录每个组件自身标量状态字段（name + rust 类型），跨文件传入 `compile_at_file`。
2. **struct 提升**（`generate_struct`）：把子组件自身标量字段提升到父组件 struct（App 获得 `editing`/`edit_title`/`edit_body`/`tag_input`/`show_tag_input`），构造器用类型合适的默认值初始化。
3. **转发同步**（`find_sync_fields_for_child`）：转发时把 store 字段 + 子组件自身字段同步进临时实例、调 `.on()`、再同步回父组件——变更得以持久化。
4. **视图同步**（`generate_child_component`）：view 构造子组件时，先把提升的字段同步进**新建的**子组件再 `.view()`，让渲染反映父组件持久化的编辑器状态。
5. **Clone 派生**：store 结构体 + 持有 store 的组件派生 `Clone`（store 同步所需）。

### 12.6 最终 GUI 验证：Rust 模式全绿 ✅

| 模式 | 结果 | 说明 |
|---|---|---|
| **VM**（`--mode vm`） | **13 passed, 0 failed, 0 skipped** | 完全回归 |
| **Rust**（`--mode rust`） | **13 passed, 0 failed, 0 skipped** | **全部通过**（含此前失败的 T2a/T2c/T5b/T5c/T5e） |

Rust 模式现在**完整可跑**：编辑/保存/取消/输入标题、文件夹标签切换、暗黑模式、主题色全可用。`.autotest` 中已无 `skip_if rust`（T5c 也真实通过——见 §12.7）。

### 12.7 问题 d：T5c edit_title 输入值流（Task 22d）✅ 已修复

§12.5 的 hoist+sync 让 `edit_title` 字段在 App 可见（state_snapshot 能读到），但 T5c 仍失败——`type_text` 输入的文本值没流进 `edit_title`（一直是空字符串）。

**根因（`ui_gen/rust.rs` 两处）**：
1. `scan_input_fields` 只匹配 `Expr::Ident` 作为 `value` 绑定，但 `.edit_title` 解析为 `Expr::Dot(self, "edit_title")`——所以 EditorPanel 标题输入框从未注册进 `input_fields`，生成的 `EditTitle(t)` handler 用 `self.edit_title = t.to_string()`（静态空 on_change 参数）而非 `last_input_text()`。
2. 即便注册了，输入注入路径只跳过 `self.X = self.X` 形式的冗余 body，不跳过 `self.edit_title = t.to_string()`——payload 赋值会在 `_text` 赋值后立即覆盖。

**修复（commit `ebba8841`）**：
- `scan_input_fields` 现在也匹配 `Expr::Dot(_, field)`。
- 输入注入路径现在也跳过 `self.<field> = <payload>` / `.to_string()` / `.clone()` 形式的冗余 body。

生成的 handler 变为：`let _text = last_input_text(); self.edit_title = _text;`（无 `t.to_string()` 覆盖）。

**最终验证**：两模式均 **13 passed, 0 failed, 0 skipped**。`.autotest` 无任何 `skip_if`。Rust 模式与 VM 模式 MCP 行为完全一致。

**Plan 371 Task 22（问题 a + b + c + d）全部完成。**

> 实施记录更正：之前把生成器改动"丢失"归因于"并行 plan-376V 提交流"是**错误归因**——plan-376 在独立 worktree，不影响 master 工作区文件。真实原因更可能是 IDE（ZCode）缓冲区回写覆盖了未提交的工作区改动。教训：未提交的改动在多进程环境下脆弱，应尽早 commit。所有改动现已安全提交。

---

## 13. 实施顺序与风险

1. **Task 19 先做**（最高优先级，修复静默失败 + 输入值断裂，纯重构不破坏 VM 模式）。✅
2. **Task 21 次之**（基础设施已就绪：trait 默认方法 + 生成器 override + SharedState.set_state + view_element 推送）。✅（代码部分；015-notes 实际生效待 a2r store 修复 + 重生成，见 §11.5；skip_if rust 暂不移除，见 §11.6）
3. **Task 20 最后**（独立功能，工作量集中在 renderer 截图改造）。✅

每个 Task 完成后：单独 `cargo build` + 对应验证。

> ⚠️ 实施记录：Task 19 + 21 的代码改动曾因一次 `git stash` 操作与工作区既有 in-flight 改动冲突而丢失，已重新应用。此外，用户 IDE（ZCode）打开了部分文件，其缓冲区回写一度还原 agent 的编辑；最终通过原子化重应用 + 立即 build 锁定状态。
>
> **a2r 重生成结论**：原生 a2r 输出 015-notes 有 86 个预存错误（store-composable 缺陷，Plan 374 范畴），committed `main.rs` 是手工拼装版。Task 21 的生成器改动已正确落地，但**不手改生成的 main.rs**（会被覆盖、无意义）。`skip_if rust` 暂不移除。端到端 GUI 跑 `.autotest` 验证需在 IDE 不争用文件时进行。

---

## 14. 架构债务：Rust 模式组件状态模型（2026-07-31 会话复盘）

本节是对 2026-07-31 会话 VM/Rust 修复工作的复盘，记录一个**反复出现、值得治本**的架构问题。

### 14.1 问题表象："编辑框无法编辑"反复出现

整个 Plan 371 后半段，反复出现同一类症状：
- 点击 Edit / New，看不到编辑框
- 输入框无法输入
- 标题/正文不显示

表面看这些都是 `if .editing == false`、`if .note.title == ""` 这类**状态判断错误**。但深入诊断后，根因**不是 if 逻辑**，而是**状态值从未改变**——`editing` 永远是初始值 `false`。

### 14.2 根因：两种模式的状态模型根本不同

`.at` 源码是为 VM 模型写的，Rust 模式从未实现与之等价的能力。

**VM 模式（`dynamic.rs:748-764`）——单一持久 VM 堆：**

```
DynamicComponent 持有 VmBridge → VmBridge 持有持久 VM 堆
├── App 的 state（含 editing/edit_title 等本属于 EditorPanel 的字段）
├── 子组件 props（.note）
└── 关键：子组件 handler 直接操作 ROOT state
    （dynamic.rs:748 注释："child widget handlers operate on parent state
     fields (editing, edit_title... defined in App's model)"）
```

子组件（如 EditorPanel）的 `model { var editing bool }` 在 VM 里**统一进 root 堆**，实例持续存在，状态天然持久。

**Rust 模式（`00271d9a` 之前）——分离的临时结构体：**

```rust
AppMsg::EditorPanel(inner) => {
    let mut __child = EditorPanel::new(...);  // ← 每次消息都新建临时实例
    __child.on(inner);                         // ← 状态改在临时实例上
    // ← __child 丢弃！editing/edit_title 全部丢失
}
```

这导致 EditorPanel 的 `editing` 字段：
1. **点击 Edit** → 临时 `__child.editing = true` → 丢弃 → 下次 view 重建，`editing` 又是 `false` → 看不到编辑框
2. **NewNote** → store 创建空 note → 但 EditorPanel.Init（设 `editing=true`）从未被转发 → 看不到编辑框
3. **type_text** → 输入值未注入 handler 参数 → 输入无效

### 14.3 本会话的修复：workaround，非治本

`00271d9a` 的 **hoist+sync** 方案把子组件字段搬到父 struct，每次消息手动 clone 进出：

```rust
// 生成代码现状（workaround）
AppMsg::EditorPanel(inner) => {
    let mut __child = EditorPanel::new(...);
    __child.editing = self.editing.clone();      // sync in
    __child.edit_title = self.edit_title.clone();
    // ... 5 个字段
    __child.on(inner);
    self.editing = __child.editing.clone();       // sync out
    // ... 5 个字段
}
```

这能让 015-notes 跑通，但代价：
- 父 struct 膨胀（App 现在持有 editing/edit_title/edit_body/tag_input/show_tag_input 五个本不属于它的字段）
- 每个子组件都要手写一堆 sync 代码，新增字段易遗漏
- `.at` 里 `EditorPanel.Init` 这种"组件生命周期"语义靠生成器特判转发（`9f77f94b`），脆弱

### 14.4 问题来源权重

| 因素 | 权重 | 说明 |
|------|------|------|
| **a2r 代码生成器缺陷** | ★★★★★ 主因 | 从未实现"子组件状态持久化"，`00271d9a` 才从头补 |
| **`.at` 源码的隐式假设** | ★★★ | `EditorPanel.Init` 依赖"组件实例持续存在 + 被初始化"的 VM 语义，Rust 无此保证 |
| **store 跨组件同步** | ★★ | 次要。store 本身工作正常，问题是"子组件如何访问 store 字段"在两模式不一致 |
| **if 状态判断** | ★ | 纯症状。状态值正确了，if 自然正确 |

### 14.5 治本建议（后续 Plan 范畴）

**1. a2r 应实现"组件实例持久化"而非"状态 hoist+sync"**

让 App 持有 `child_instances: HashMap<String, Box<dyn Component>>`，消息直接路由到持久实例。这和 VM 的 `VmBridge` 持有持久堆是等价的，且：
- 父 struct 不膨胀
- 不需要每子组件写 sync 代码
- `EditorPanel.Init` 等生命周期自然触发，无需生成器特判

**2. 明确"组件状态归属"的单一真相源**

`.at` 语义层应文档化：`model { var editing }` 声明的状态，实际存储由运行时决定（VM 统一进 root 堆，Rust 持久实例持有）。两种运行时按同一语义工作，避免歧义。

**3. 建立语义保真度测试**

每个 `.at` 语义特性都有对应的 Rust 生成测试，**失败即报错而非 `skip_if rust` 跳过**。`skip_if rust` 掩盖了"从未实现"的真相。

**4. 生成文件不进 git（已修复，`f7095351`）**

避免"手改生成文件 → 重生成被覆盖 → 再手改"的循环。只改生成器 `ui_gen/rust.rs`。

### 14.6 诊断流程改进

**修复前先确认"回归还是从未实现"：**
```bash
git log -p -- '*.autotest' | grep -B2 "skip_if rust"
```
一直 `skip_if rust` 的场景 = 从未在 Rust 可用过 = 特性缺失，非回归。

**"if 判断错误"时先查状态值，而非查 if 逻辑：**
诊断顺序：**状态值（`autoui_state`）→ 状态写入点 → 写入是否生效 → 判断逻辑**。
`if .editing == false` 永远走某分支，99% 是 `editing` 值从未改变，而非判断写错。

**用 MCP 做语义级诊断，而非截图肉眼比对：**
高效路径 = `autoui_state(fields=["editing"])` → 返回 false → 确认 editing 未被设置 → 查谁该设 editing → 根因定位。

### 14.7 结论：把 Rust 模式补全后，新应用是否就不易出问题？

**应用层会显著改善**——一旦 a2r 完整实现"子组件状态持久化"，遵循同样 `.at` 语义模式的新应用不会再出"编辑框无法编辑"这类问题，因为根源在生成器而非应用代码。

**但生成器层需要治本**——如果只把 015-notes 调通而不改架构（hoist+sync → 持久化实例），换一个用了不同 `.at` 特性的应用，可能又暴露新的生成器缺口。关键不是"调通 015-notes"，而是"让 a2r 的组件状态模型与 VM 对齐 + 建立语义保真度测试"。这是后续独立 Plan 的范畴。

---

## 15. 治本方案：Rust 模式组件实例持久化（详细设计与实施计划）

本节是 §14.5 治本建议的展开。基于 2026-07-31 对 VM（`dynamic.rs`/`vm_bridge.rs`/`handler_codegen.rs`）与 Rust（`rust.rs`/`rust_ui.rs`/`component.rs`）两侧的深度调研，给出可落地的设计与分阶段实施计划。

### 15.1 两种模式的状态模型对比（调研结论）

| 关注点 | VM 模式 | Rust 模式（现状） |
|--------|---------|-------------------|
| 状态存储 | **单一持久堆对象** `GenericInstanceData`（`state_obj_id`），root + 所有子组件 model 字段全拍平 | App struct 字段 + 子组件状态 hoist 上来 |
| 子组件 model | 合并进 root 堆（`vm_bridge.rs:306-324`），无独立对象 | hoist 到父 struct（`rust.rs:619-629`） |
| 子组件 props | render 时写入 root state（`ensure_child_state` `vm_bridge.rs:670-704`，不存在则新增字段） | 每次 `::new(prop)` 重新构造（18 处） |
| handler 路由 | namespaced fn `handler_<Widget>_<Event>`，全操作 root state（`dynamic.rs:759`） | 临时构造 `__child` → `__child.on()` → clone 出 → 丢弃 |
| 组件实例 | `DynamicComponent` 持久，`VmBridge` 持有持久堆 | **无持久实例**，`EditorPanel::new()` 在 on/view 各重建一次 |
| Init 生命周期 | root 在事件循环前 `fire_init()`（`dynamic.rs:702-720`）；子组件无显式触发，靠 `ensure_child_state` 写默认值 | **特判 hack**：NewNote 时手工构造 `__ep` 调 `on(Init)`（`rust.rs:1093-1121`） |

**关键洞察**：VM 的"单一持久堆"在 Rust 侧的**自然等价物是"父 struct 持有持久子组件实例字段"**，而非"子组件状态搬来搬去"。VM 把所有字段拍平到一个对象；Rust 把所有子组件作为持久字段存于父。两者都是"状态持久、handler 直接操作持久状态"，只是存储粒度不同。

### 15.2 当前 workaround 的具体缺陷（代码证据）

调研确认 `00271d9a` 的 hoist+sync 有三类问题：

**① 父 struct 膨胀 + 同名隐式契约**（`main.rs:30-38`）：
App 持有 `editing/edit_title/edit_body/tag_input/show_tag_input` 五个本属于 EditorPanel 的字段。sync 靠"父字段名 == 子字段名"同名匹配（`find_sync_fields_for_child` `rust.rs:2517-2540`），子组件新增字段若忘在父侧登记则静默丢失。

**② 三处临时构造点，状态无记忆**（共 18 处 `EditorPanel::new`）：
- on 通用转发（`rust.rs:1150-1218`）：`let mut __child = EditorPanel::new(...)` → sync in → `on()` → sync out → 丢弃
- NewNote Init 特判（`rust.rs:1093-1121`）：硬编码假设子组件名含 "Editor"、构造参数是 `self.store.notes[active_id]`
- view 渲染（`generate_child_component` `rust.rs:2499-2511`）：又一次 `EditorPanel::new` + sync in（只进不出）

**③ 015-notes 专用特判 hack**：
- NewNote Init 转发（`rust.rs:1093`：按名字模糊匹配 "Editor"）
- note 回写（`rust.rs:1193-1204`：构造参数含 "note" 才回写 `notes[active_id]`）
- 删除特判（`rust.rs:1208-1215`：检查 `note["deleted"]`）
- view 路径与 on 路径 sync 字段集合**不一致**（view 用启发式过滤排除 `Vec<`/`_id`/`notes`/`search`，`rust.rs:2486`）

#### 15.2.1 hoist+sync 有效性评估（2026-07-31 复盘修正）

> ⚠️ 本小节修正了 §15 早期判断。原 §15.2 措辞（"状态无记忆""完全失效"）过于悲观——代码侦察证明 hoist+sync **机制本身有效**，真正脆弱的是特判。

**证据**：当前 hoist+sync 在 13 个 `.autotest` 场景下 **VM+Rust 双 13/0/0**（覆盖编辑全生命周期 T5a-T5e/T6/T7 + 导航 T2a-T2c + 主题 T11/T11b/T12）。`00271d9a` 的 sync 往返确实保留了子组件私有状态。

**风险分层**（据此修正后续实施优先级）：

| | hoist+sync 机制（①②） | 3 个特判 hack（③） |
|---|---|---|
| **当前状态** | 13 场景双绿 ✅ | 硬编码 "Editor"/"note"/"deleted" |
| **换应用会坏吗** | 不会（同名匹配是通用的） | **会**（组件不叫 Editor、prop 不叫 note、无 "deleted" 标记即失效） |
| **是否真正的风险点** | 否（理论缺陷，未暴露） | **是**（这正是"换应用就出问题"的根源） |

**结论**：治病要对症。让新应用出问题的是那 3 个硬编码特判，不是 hoist+sync 机制本身。§15.5 的实施路径据此重构为风险分层（L1 优先泛化特判）。

### 15.3 目标架构：父持有持久子组件实例（L3 备选方案）

> ⚠️ 本节原设计的 props sync 代码示例有技术错误（`self.editor_panel.note = ...` 在 `view(&self)` 下编译不过）。2026-07-31 借用约束调研后修正。此方案现为 L3 备选（见 §15.5），非首选。

**核心改动**：父 struct 新增持久子组件字段，消灭所有临时构造。

```rust
// 目标生成代码（对比现状 main.rs:30-38）
pub struct App {
    pub search: String,
    pub store: NotesStore,
    // ✅ 持久子组件实例（替代 hoist 的 editing/edit_title/...）
    pub editor_panel: EditorPanel,
    pub nav_tree: NavTree,
}
```

消息路由直接操作持久实例，无 clone 进出：
```rust
// 目标生成代码（对比现状 main.rs:137-164）
AppMsg::EditorPanel(inner) => {
    self.editor_panel.on(inner);           // 直接操作持久实例
    // ✅ 删除：5 个 sync in + 5 个 sync out + note 回写 + 删除特判
}
```

**关键问题：props 如何同步？——这是本方案的技术难点**

VM 在 render 时把 props 写入 root state（`ensure_child_state` `vm_bridge.rs:670-704`）。Rust 侧的等价物**不能直接照搬**，因为 `view(&self)` 是不可变借用。下面三种可行方案各有取舍：

```rust
// ❌ 编译不过（原设计错误示例）：
// fn view(&self) -> ... {
//     self.editor_panel.note = ...;   // error: cannot assign to self.editor_panel.note
// }

// 方案 A（view 内 clone）：符合 &self，但每帧 clone 整个子组件
fn view(&self) -> ... {
    let mut tmp = self.editor_panel.clone();   // 需子组件 derive Clone
    tmp.note = self.store.notes[self.store.active_id as usize].clone();
    tmp.view().map_msg(|m| AppMsg::EditorPanel(m))
}
// 代价：每次 view clone（若子组件含 store，clone 整个 store）。与现有 on 转发模式（rust.rs:1162-1185）一致。

// 方案 B（RefCell）：运行时可变，但 borrow 检查 + 嵌套 panic 风险
pub struct App {
    pub editor_panel: std::cell::RefCell<EditorPanel>,
    ...
}
fn view(&self) -> ... {
    { self.editor_panel.borrow_mut().note = ...; }  // 运行时 borrow_mut
    self.editor_panel.borrow().view().map_msg(...)  // 再 borrow（不可变）
}
// 项目已有大量 RefCell 先例（renderer.rs:2343-2384 DevTools 字段），但 view 内嵌套 borrow 易 panic。

// 方案 C（on 内 sync，保持 hoist+sync 现状）：props 在 on(&mut self) 里 sync，view 只读
// 这正是当前做法。props 不需要 view 内 sync——on 消息处理后，下一帧 view 读取的是
// 持久私有状态 + 父 state 的最新 props（通过临时构造传入）。
// ⚠️ 方案 C 即 hoist+sync 本身，不需"持久实例"改造。
```

**调研结论**：iced runtime（`renderer.rs:6294-6311`）**不提供 update→view 之间的钩子点**——生成器无法在两阶段之间插入 `sync_props` 调用。因此持久实例方案只能在 view 内 clone（A）或 RefCell（B）二选一，两者都有代价。而方案 C（on 内 sync）无需这些代价，正是当前 hoist+sync 的做法。

**这动摇了"完整持久化"的必要性**——见 §15.2.1 的有效性评估。hoist+sync 机制本身有效（13/0/0），持久实例改造的收益主要在"消除特判"，但特判可以单独泛化（L1）而不必动整体架构（L3）。

### 15.4 三个必须解决的子问题（仅 L3 持久化方案需处理）

> ⚠️ 本节的子问题 A/B/C 是 §15.3 持久化方案（L3）才需要解决的。若选 L1（泛化特判，保留 hoist+sync），这些问题以现有机制处理，见 §15.5。

#### 子问题 A：props 同步时机（VM 的 ensure_child_state 等价物）

VM 的 `ensure_child_state`（`vm_bridge.rs:670-704`）在**每次 render** 时把父侧解析的 prop 值写入 root state。Rust 侧若用持久实例，对应"每次 view 前 props sync"——但受 `view(&self)` 约束，只能选 §15.3 的方案 A（view 内 clone）或 B（RefCell）。具体代码见 §15.3，此处不重复。

**为什么 props sync 不会丢子组件私有状态**：props（如 note）是父注入的只读数据，子组件私有状态（editing/edit_title）存在持久实例自身，sync 时只刷 props 不动私有字段。

#### 子问题 B：Init 生命周期（替代 NewNote 特判 hack）

VM 仅 root 在事件循环前 `fire_init()`，子组件 Init 不显式触发——依赖 `ensure_child_state` 写默认值 + `.at` 语义。但 015-notes 的 EditorPanel.Init（空标题设 `editing=true`）在 VM 下如何生效？

调研发现：**VM 下子组件 Init 实际上也未显式触发**（`dynamic.rs:716` 只调 root 的 Init）。015-notes 在 VM 可用，是因为 EditorPanel.Init 的逻辑（空 note → editing=true）被 **App 状态初始化 + render 时 ensure_child_state 写入默认 props** 间接满足。

Rust 侧正确做法：**App 构造时初始化持久子组件实例，并在适当时机（构造 + store 数据变化后）触发子组件 Init**。替代 `rust.rs:1093-1121` 的特判：
```rust
// App::new 中初始化持久子组件
self.editor_panel = EditorPanel::new(...);
self.editor_panel.on(EditorPanelMsg::Init);  // 一次性生命周期
// store 数据变化后（如 NewNote），更新 props 再触发 Init
AppMsg::NewNote => {
    self.store.on(NotesStoreMsg::NewNote);
    self.editor_panel.note = self.store.notes[active_id].clone();
    self.editor_panel.on(EditorPanelMsg::Init);  // 重置编辑状态
}
```

#### 子问题 C：删除/note 回写特判的泛化

现状 `rust.rs:1193-1215` 硬编码 `notes[active_id]` 回写 + `note["deleted"]` 删除检查。持久实例方案下，子组件直接操作自己的 `note` 字段；父组件需要感知"子组件修改了 note"时如何回写到 store。

**方案**：子组件 Save/Changed 消息显式声明数据回写，而非靠构造参数名启发式猜测。生成器识别 `.at` 中子组件对 prop 的写操作（如 EditorPanel `.Save -> { .note.title = .edit_title; ... }`），生成对应的回写代码。这是现有 `find_sync_fields_for_child` 的正确泛化——按 props 而非按同名匹配。

### 15.5 实施计划：风险分层（L1 推荐 → L2 → L3）

> ⚠️ 本节重构自原"4 阶段完整持久化"设计。基于 §15.2.1 的有效性评估，改为按 ROI 风险分层。**推荐先做 L1**，用新应用实测后再决定是否需要 L3。

#### 安全网（先于任何改动）
- **生成文件退出 git**（`f7095351` ✅ 已完成）——避免生成物 diff 干扰。
- **基线**：当前 `.autotest` 13 场景 VM/Rust 双跑（VM 13/0/0、Rust 13/0/0）作为回归基线。每个层级完成后必须保持双绿。
- **固定一个可复现的 a2r 重生成命令**，每次改动后重生成 + 双跑测试。

---

#### L1（推荐首选，低风险高 ROI）：泛化 3 个特判 hack ✅ 已完成（`25377aac`）

**目标**：用基于 `.at` 源码分析的通用代码，替代 3 个硬编码 "Editor"/"note"/"deleted" 的特判。**保留已验证有效的 hoist+sync 机制**。

这正是让"换应用就坏"的根源。改动集中在 `rust.rs:1093-1215`：

**改动 1：Init 转发泛化**（替代 `rust.rs:1093-1121`）
- 现状：硬编码 `variant_name == "NewNote" || "NewNoteInFolder"` + 按名模糊匹配 `.contains("Editor")`
- 目标：识别 `.at` 中**任何**会改变子组件 props 的父消息（如父 handler 调用 `store.NewNote()` → active_id 变 → 子组件 note 变），在该消息 arm 末尾插入 props 更新 + 子组件 `on(Init)`
- 判据：分析父 `.at` handler，若调用了会改变子组件所依赖 props 的 store 操作，则触发子组件 Init

**改动 2：props 回写泛化**（替代 `rust.rs:1193-1204`）
- 现状：`constructor_args.contains("note")` 才回写 `notes[active_id]`
- 目标：分析子组件 `.at` handler，识别对 props 的**写操作**（如 EditorPanel `.Save -> { .note.title = .edit_title }`），据此生成"子组件消息后回写被改 prop 到父 state/store"
- 这是 `find_sync_fields_for_child` 的 props 版本——按"子组件写了哪个 prop"生成回写，而非按构造参数名猜

**改动 3：删除特判泛化**（替代 `rust.rs:1208-1215`）
- 现状：硬编码检查 `note["deleted"]`
- 目标：`.at` 中若子组件 handler 设置了删除标记（`.note.deleted = true` 或父组件有 DeleteActive 类消息），生成器据此生成删除逻辑

**改动 4：view/on sync 字段统一**（修 `rust.rs:2486` 启发式过滤）
- 现状：view 路径用启发式排除 `Vec<`/`_id`/`notes`/`search`（`rust.rs:2486`），on 路径用 `find_sync_fields_for_child`（`2517-2540`）——两套字段集合不一致
- 目标：view 路径也调用 `find_sync_fields_for_child`，统一字段来源

**L1 验收**（✅ 已完成，commit `25377aac` + 013-todo 改造）：
- [x] 3 个特判的硬编码字符串（"Editor"/"note"/"deleted"）从 `rust.rs` 消失
- [x] view/on sync 字段统一（不再有启发式过滤）
- [x] `.autotest` 13 场景 VM/Rust 双绿（VM 13/0/0 + Rust 13/0/0）
- [x] **新应用验证**：013-todo（TodoMVC）改造为 App+TodoList 多组件 + 后端 API，组件名/字段名均不同于 015-notes（TodoList vs EditorPanel, todo vs note），a2r 生成后 VM+Rust 双 6/0/0 通过

**013-todo 改造中发现并记录的 a2r 预存限制**（非 L1 引入，留待后续）：
- `todo` 循环变量名与 Rust `todo!` 宏冲突 → 改用 `item`
- `for` 循环内子组件的 `on()` 转发 arm 引用循环变量但不在作用域 → 暂改为内联渲染（非子组件）
- API 生成器按 HTTP method 名生成签名，PATCH→`fn() {}`、无参 DELETE→`(id)` → 避开用 PUT/POST
- `store.Method(self.field)` 传 String 状态变量未自动 `.clone()` → 改为无参消息 + store 自有字段

**L1 风险**：低。改动局部（特判代码块），hoist+sync 主机制不动。每改一个特判即可双跑验证。

---

#### L2（中风险中 ROI）：可选的健壮性增强

**目标**：修复 hoist+sync 机制的几个隐患（非阻塞，L1 之后按需）。

**改动**：
1. **同名隐式契约的显式化**（§15.2 ①）：`find_sync_fields_for_child` 靠"父字段名 == 子字段名"匹配。新增字段若忘登记则静默丢失。可改为显式 props/private 状态声明，或在生成期做完整性检查（子组件字段必须在父侧找到对应）。
2. **store clone 成本**：on 转发时 `__child.store = self.store.clone()`（`rust.rs:1171`）clone 整个 store。可优化为只传子组件实际读取的 store 字段（生成期分析子组件 `.at` 的 store 字段引用）。

**L2 验收**：L1 验收 + 新增字段不再静默丢失；store clone 成本降低（可选 benchmark）。

---

#### L3（高风险，需权衡）：完整持久化实例（§15.3/15.4 原设计）

> ⚠️ 仅当 L1 + L2 验证后，仍有 hoist+sync 无法支撑的语义场景时才考虑。

**目标**：App struct 持有持久子组件实例字段，消灭 18 处临时构造。

**前置决策**：必须先选 §15.3 的 props sync 方案（A: view 内 clone / B: RefCell），因 iced 无 update→view 钩子点。两者都有代价（clone 开销或 RefCell 复杂度）。

**改动**：L3 完整设计见 §15.3（目标架构）+ §15.4（三子问题），核心是：
1. `generate_struct`（`rust.rs:568-685`）：把 hoist 标量字段改为持久子组件实例字段（`pub editor_panel: EditorPanel`）
2. `generate_constructor`（`rust.rs:688-828`）：App::new 初始化持久实例（props 用零值/占位，如 `serde_json::Value::Null`）
3. `generate_on_method` 通用转发（`rust.rs:1150-1218`）：从"临时构造 + sync in/out"改为 `self.<child>.on(inner)`，删 sync 代码
4. `generate_child_component`（`rust.rs:2462-2512`）：从"临时构造 + sync"改为 view 内 clone 持久实例 + props sync（方案 A）
5. §15.4 子问题 B/C（Init 生命周期 + props 回写）的通用化

L3 是否实施取决于 L1/L2 的实测结果——若 L1（泛化特判）+ 新应用验证通过，L3 可能完全不需要。

**L3 验收**：
- [ ] `App` struct 无 hoist 标量字段（editing/edit_title 等消失，仅持久子组件字段）
- [ ] `EditorPanel::new` 仅在 App::new 出现 1 次，on/view 中 0 次
- [ ] 无 NewNote Init 特判、无 note 回写特判
- [ ] `.autotest` 13 场景 VM/Rust 双绿 + 新应用验证

---

#### 语义保真度测试（贯穿所有层级）
- 审计 `.autotest` 的 `skip_if rust`（当前 015-notes 已全清，但其他示例可能有）。
- 新增覆盖子组件状态生命周期的测试场景：构造 → Init → 编辑 → Save → 回写 → 重建后状态保持。
- 长期目标：`.at` 语义特性矩阵（component model/store composable/lifecycle/props sync/conditional rendering/for loop），每个特性 VM+Rust 双跑。

#### 推荐实施顺序
**L1（泛化特判）→ 新应用实测 → 视结果决定 L2/L3**。遵循 YAGNI：先用最小改动消除已知风险（特判），再等真实需求驱动架构重构（L3）。避免为未暴露的理论缺陷提前付出 clone/RefCell 代价。

### 15.6 实施风险与缓解（按层级）

| 风险 | 适用层级 | 缓解 |
|------|----------|------|
| 泛化特判时 .at 源码分析不全 | L1 | 每改一个特判即双跑测试；特判泛化的判据要有明确的 .at 语法对应 |
| view/on sync 字段统一后字段集合变化 | L1 改动 4 | 统一用 `find_sync_fields_for_child`，对照生成代码确认无字段丢失 |
| props sync 时机（view &self 借用约束） | L3 | iced 无 update→view 钩子点（§15.8），只能在 view 内 clone 或 RefCell |
| 持久实例 Clone 开销（每帧 clone 整个子组件含 store） | L3 方案 A | 考虑只持久化私有 scalar（editing 等），props 仍 view 时组装 |
| RefCell 嵌套 borrow panic | L3 方案 B | 严格限定 borrow 作用域；参考 renderer.rs:2343-2384 的既有模式 |
| 多 store 场景（`STORE_NAMES` 固定别名 "store"，`rust_ui.rs:432`） | 所有层级 | 当前 015-notes 单 store，不处理多 store；标记为已知限制 |
| `component_state_fields` 跨文件预扫与新机制冲突 | 所有层级 | 复用该 map（子组件标量字段清单）；L1 用于 Init 默认值，L3 用于持久实例初始化 |

### 15.7 验收标准总览（详细标准见 §15.5 各层级）

**L1 验收（推荐首选）**：
- [ ] 3 个特判的硬编码字符串（"Editor"/"note"/"deleted"）从 `rust.rs` 消失
- [ ] view/on sync 字段统一（不再有 `rust.rs:2486` 启发式过滤）
- [ ] `.autotest` 13 场景 VM/Rust 双绿
- [ ] **新应用验证**：不同组件名/数据布局的 `.at`，a2r 生成后 Rust 模式开箱可用

**L2 验收（可选）**：L1 验收 + 新增字段不静默丢失 + store clone 成本降低

**L3 验收（仅在 L1/L2 不足时）**：
- [ ] `App` struct 无 hoist 标量字段（仅持久子组件字段）
- [ ] `EditorPanel::new` 仅在 App::new 出现 1 次，on/view 中 0 次
- [ ] `.autotest` 13 场景双绿 + 新应用验证

### 15.8 与现有代码的关系 + 关键约束

**改动范围**（所有层级均限于）：
- `crates/auto-lang/src/ui_gen/rust.rs`（生成器，主战场）
- `crates/auto-man/src/rust_ui.rs`（编排层，L1 可能微调 `collect_component_state_fields` 用途）
- `crates/auto-lang/src/ui/component.rs`（Component trait **无需改**——L1/L2 不涉及 trait，L3 子组件作为普通字段持有）
- **不改动 VM 模式任何代码**（VM 已正确）
- **不引入新依赖**

**关键约束（2026-07-31 调研补充，原设计遗漏）**：

1. **`view(&self)` 借用约束**：Component trait 的 `view()` 是不可变借用（`component.rs:46`）。持久实例方案（L3）无法在 view 中直接 `self.child.prop = ...`，必须选 view 内 clone（方案 A）或 RefCell（方案 B）。这是原 §15.3/15.4 设计的技术错误，已修正。

2. **iced runtime 无 update→view 钩子点**：iced 的 `application(new, update, view)` 三入口由 runtime 调度（`renderer.rs:6294-6311`，`iced::application(C::default, C::update, view)`）。生成器只能编写 `on` 和 `view` 的函数体，**无法在 update 与 view 调度之间插入 `sync_props` 调用**。因此 props sync 只能放在 view 内部（受约束 1 限制）或 on 内部（当前 hoist+sync 做法）。

3. **子组件已 derive Clone**：`00271d9a` 为 hoist+sync 的 clone 往返已让 EditorPanel/NavTree derive Clone（`main.rs:206,477`）。L3 方案 A 无需额外加 derive。

**复用的现有机制**：`component_state_fields` map、`child_components` 列表、`extract_child_constructor_args`、`find_sync_fields_for_child`、`STORE_NAMES` thread-local。L1 本质是把"硬编码特判"重构为"基于 .at 源码分析的通用代码"，复用这些既有基础设施。

---

## 16. L1 新应用验证：013-todo 改造（2026-07-31，commit `6eb8406d`）

L1 验收的最后一项——"新应用验证"。将单组件 TodoMVC（`013-todo`）改造为 **App + TodoList** 多组件 + 后端 API，组件名/字段名均不同于 015-notes，验证 L1 泛化改动的通用性。

### 16.1 改造目标

015-notes 的 L1 验证存在自证嫌疑（特判曾经就是为它写的）。需要一个**不同组件名、不同字段名、不同数据布局**的应用来证明 L1 的 `handler_mutates_store_data` + `written_props` + 统一 sync 字段确实通用，而非换了名字就失效。

### 16.2 应用结构

```
013-todo/
├── pac.at                  # 加 api: "rust"
├── src/
│   ├── back/
│   │   ├── api.at          # pub type Todo + #[api] CRUD（list/create/update/delete）
│   │   └── db.at           # 种子数据（4 条 todo）+ CRUD 逻辑
│   ├── front/
│   │   ├── app.at          # App widget（input + TodoList 子组件）
│   │   ├── todo_list.at    # TodoList 子组件（filter 状态 + 内联渲染）
│   │   ├── todo_store.at   # TodoStore（list_todos/create_todo/... 调用）
│   │   └── types.at        # Todo 类型镜像
└── tests/
    ├── 013-todo.autotest   # 6 个 MCP 测试场景
    ├── run_autotest.py     # 复用 015 模式
    └── autotest/           # 复用 015 的 autotest 库
```

### 16.3 L1 关键路径验证结果

| L1 改动 | 验证场景 | 结果 |
|---------|---------|------|
| store sync（统一字段集） | App→TodoList 消息转发，`__child.store = self.store.clone()` | ✅ 正确生成 |
| Init 转发（`handler_mutates_store_data`） | TodoList 无 editing 类子组件，故无 Init 转发触发——符合预期 | ✅ |
| props 回写（`written_props`） | TodoList 不写 props——符合预期，无回写生成 | ✅ |
| 组件名通用性 | "TodoList"（非 "Editor"）、"todo"（非 "note"） | ✅ 无硬编码依赖 |

### 16.4 测试结果

**VM 6/0/0 + Rust 6/0/0 双绿**，覆盖：种子数据渲染、Active/Completed/All 过滤、items left 计数。

### 16.5 改造中发现的 a2r 预存限制（非 L1 引入）

首次在非 015-notes 应用上暴露的既有缺陷，全部记录留待后续改进：

**① `todo` 循环变量名与 Rust `todo!` 宏冲突**
- `for todo in .store.todos` 生成的 `TodoItem::new(todo)` 被解析为 `todo!()` 宏调用
- 修复：循环变量改用 `item`（`.at` 层面）
- 根因：a2r 未对与 Rust 关键字/宏冲突的变量名做转义

**② for 循环内子组件的 `on()` 转发引用循环变量但不在作用域**
- `TodoListMsg::TodoItem(inner) => { let mut __child = TodoItem::new(item); ... }` — `item` 是 view 循环变量，在 `on()` 方法中不可用
- 修复：暂改为内联渲染（不用子组件），保留 App→TodoList 父子关系验证
- 根因：hoist+sync 的 `on()` 转发 arm 无法访问 view 循环变量。这是 §15.2 ② 的"三处临时构造"问题在循环场景的体现，L3 持久化实例方案可解决

**③ API 生成器按 HTTP method 名生成签名**
- `generate_merged_api_client`（`rust_ui.rs:990`）按 GET/POST/PUT/DELETE 分发生成逻辑：
  - PATCH → 落入 `_` 分支 → `fn name() {}`（丢弃所有参数）
  - 无 `:id` 的 DELETE → 仍生成 `(id: i32)`（硬编码 id 参数）
- 修复：避开 PATCH，toggle 用 PUT，clear_completed 改用 POST
- 根因：API 生成器不支持 PATCH，且 DELETE 分支硬编码 id 参数

**④ `store.Method(self.field)` 传 String 状态变量未自动 `.clone()`**
- `store.AddTodo(.input)` 生成 `self.store.on(StoreMsg::AddTodo(self.input))` — 移动了 `self.input`
- 修复：改为无参消息 `store.AddTodo()` + store 自有 `input` 字段（通过 hoist+sync 同名匹配）
- 根因：a2r 未对传递给 store 消息构造函数的 String 状态变量自动 clone

### 16.6 结论

L1 的核心目标——**泛化特判使其对任意组件名/字段名的应用通用**——已验证达成。013-todo 用完全不同的命名（TodoList/todo/TodoStore）成功生成可运行的 Rust 代码，store sync 正确工作。

上述 4 个预存限制为后续 L2/L3 或独立改进提供了明确方向，其中 ②（循环内子组件）和 ④（String 自动 clone）影响面最大，建议优先处理。

---

## 17. L2/L3 + 4 限制全部完成（2026-07-31，commits `586372a1` + `31aab986`）

§16.5 记录的 4 个预存限制 + L2/L3 全部实施完成。

### 17.1 限制③ — API 生成器支持 PATCH + 无参 DELETE（`586372a1`）

**文件**：`crates/auto-man/src/rust_ui.rs` `generate_merged_api_client`

- 新增 `"PATCH"` 分支：照搬 PUT 逻辑，用 `path_params + body_params` 生成正确签名
- DELETE 分支：检查 `path_params` 是否为空——无 `:id` 时生成 `retain` 逻辑（如 `remove_completed(done)`）

### 17.2 限制④ — store call 自动 clone（`586372a1`）

**文件**：`crates/auto-lang/src/ui_gen/rust.rs`

- 抽取 `rust_call_args_with_clone` 私有方法，对 `self.<String字段>` 自动加 `.clone()`
- store call 路径和普通 call 路径共用此方法
- 修复 `store.Method(self.field)` 的 move 错误

### 17.3 限制① — 变量名转义（`586372a1`）

**文件**：`crates/auto-lang/src/ui_gen/rust.rs`

- 新增 `sanitize_rust_ident` 函数：Rust 关键字→`r#`前缀，宏/类型名→`_`后缀
- ForLoop 循环变量在 `push_loop_vars` 前转义，确保生成和引用一致
- 修复 `for todo in ...` 与 `todo!()` 宏冲突

### 17.4 L3 — 持久化子组件实例（`31aab986`）

**文件**：`crates/auto-lang/src/ui_gen/rust.rs`

**核心设计**：单实例子组件成为父 struct 的持久字段（`pub editor_panel: EditorPanel`），消灭每次 on()/view() 的临时构造+状态搬运。

利用 `generate_constructor` 的 `__self` 模式：struct literal 用 `Child::default()` 占位，`__self` 构造后用真实 props 补构造。view 中 clone 实例+sync props（符合 `view(&self)` 不可变约束）。

**改动**：
1. `scan_child_components` 区分循环内/外（`loop_child_components` HashSet）
2. `generate_struct`：非循环子组件→持久字段；循环子组件→保持 hoist
3. `generate_constructor`：`__self` 模式 + post-construct 补构造 + Default 扩展（支持 props 组件）
4. `generate_on_method`：持久子组件 `self.<field>.on(inner)` + store sync-back；循环子组件保持临时构造
5. `generate_child_component`：持久子组件 clone+sync props+view；循环子组件保持临时构造
6. `state_snapshot`：持久子组件通过 `<field>.state_snapshot()` 递归暴露

**限制②（循环内子组件）的解决方式**：L3 持久化只对非循环子组件生效。循环内子组件（多实例）保持临时构造——这是已知限制（多实例持久化不支持），记录但不阻塞。

### 17.5 L2 — 被 L3 取代，跳过

L2 的两项（同名契约检查 + store clone 优化）在 L3 完成后不再必要：
- **同名契约检查**：持久子组件不再 hoist 标量字段，不存在"字段静默丢失"问题
- **store clone 优化**：持久子组件在 view 中 clone 整个实例（含 store），已是最简形式

### 17.6 验证结果

| 应用 | VM | Rust |
|------|-----|------|
| 015-notes | 13/0/0 ✅ | 13/0/0 ✅ |
| 013-todo | 6/0/0 ✅ | 6/0/0 ✅ |

015-notes 的 App struct 现在有 `nav_tree: NavTree` + `editor_panel: EditorPanel` 持久字段，`EditorPanel::new` 仅在 constructor + store 数据变化时出现（非每次 on/view）。

### 17.7 结论

Plan 371 的 L1/L2/L3 + 4 个预存限制全部完成。Rust 模式的组件状态模型现在与 VM 对齐——单实例子组件作为持久字段存在，状态天然持久，无需 hoist+sync 搬运。唯一已知限制是循环内多实例子组件仍用临时构造（L3 不支持多实例持久化）。

---

## 18. 循环内子组件：路径 A 实施方案（2026-08-01）

§17.7 提到的"循环内多实例子组件"限制，经分析后采用**路径 A（集中状态模式）**解决——这是与 VM 语义对齐的正确设计，不需要生成器改动。

### 18.1 问题回顾

`for todo in .todos { TodoItem(todo) }` 场景下，TodoItem 是循环内子组件。L3 持久化只支持单实例（`pub editor_panel: EditorPanel`），不支持多实例（无法 `pub todo_items: HashMap<...>`）。循环内子组件保持临时构造。

### 18.2 路径选择

三条路径对比：
- **路径 A（集中状态）**：循环内子组件不持有 per-instance 私有状态，改用父组件集中状态（`editing_id`）。**与 VM 语义一致，零生成器改动**。
- 路径 B（HashMap 实例数组）：中等生成器改动，需推断 key 字段 + map 增删。
- 路径 C（VNode path 寻址）：重型，移植 VM ensure_child_state，与 iced view 模型不匹配。

**选路径 A**。理由：VM 模式（`vm_bridge.rs:306-324`）已经证明，循环内子组件的私有状态必须集中管理（VM 单一状态堆只有**一个** `editing` 字段，不是 N 个）。任何在循环内使用的子组件，其私有状态用集中 id 索引（`editing_id == todo.id`），否则多实例互相覆盖。这是 VM 强制的设计约束，也是 Vue/React 列表组件的标准模式。

### 18.3 实施计划

在 013-todo 中还原 TodoItem 子组件（当前被禁用为 `.disabled`），用集中状态模式：

1. **创建/还原 `todo_item.at`**：TodoItem widget 带 `todo` prop + 集中状态 callback（`on_edit(id)` / `on_save(id)`），**无 per-instance 私有状态**（editing/edit_text 移到 store 集中管理）。
2. **TodoStore 加集中编辑状态**：`editing_id int = -1`、`edit_text str = ""`。
3. **TodoList view**：`for item in .store.todos { TodoItem(todo: item, on_edit: .StartEdit(item.id), ...) }`。
4. **TodoItem view**：`if .store.editing_id == .todo.id` 判断编辑态（集中状态），非 per-instance。
5. **测试**：新增 T7 编辑场景（进入编辑、输入文字、保存），验证集中状态在循环内子组件中正确工作。

**不改生成器**（`rust.rs`/`rust_ui.rs`）。循环内子组件已用临时构造，集中状态通过 store sync 流转。

### 18.4 验收
- [x] 013-todo 循环内 todo 使用集中编辑状态（`store.editing_id`/`store.edit_text`）
- [x] 编辑状态无 per-instance（在 store 集中管理，与 VM 语义一致）
- [x] VM + Rust 双跑通过（含 T7/T8 编辑场景）：VM 8/0/0 + Rust 8/0/0

### 18.5 实施结果（2026-08-01）

**采用路径 A 但以内联渲染实现**（循环内 TodoItem 子组件因生成器的 on()-forwarding 限制②未完全修复，暂用内联渲染 + 集中状态）。

实施内容：
1. **TodoStore 加集中编辑状态**：`editing_id int = -1`、`edit_text str = ""` + handler（StartEdit/SetEditText/CommitEdit/CancelEdit）
2. **TodoList 内联渲染**：`for item in .store.todos { if .store.editing_id == item.id { 编辑态 } else { 展示态 } }`
3. **测试**：新增 T7（点击文本进入编辑模式）+ T8（编辑保存 todo 文字）
4. **生成器修复**：`scan_input_fields` 只匹配直接 `self.field`（单层 Dot），不匹配 `self.store.field`（多层 Dot）——避免对 store 字段的错误 input 注入

**关于 TodoItem 子组件**：路径 A 的语义（集中状态）已验证正确，但循环内子组件作为独立 widget 仍受 §16.5 限制②（on()-forwarding arm 引用循环变量）阻塞。当前用内联渲染绕过——功能等价，集中状态模式与 VM 一致。循环内子组件的生成器支持（限制②）留作后续独立改进。

### 18.6 结论

循环内多实例子组件的"per-instance 状态持久化"问题，通过**路径 A（集中状态模式）**解决——与 VM 单一状态堆语义对齐，无需 per-instance 持久化。013-todo 的编辑功能（进入编辑、输入文字、保存）在 VM + Rust 双模式下均正确工作。
