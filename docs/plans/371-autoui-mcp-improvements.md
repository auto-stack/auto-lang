# Plan 371: AutoUI MCP 功能大改进 — Agent 驱动的 UI 自动化验证

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

**问题 A（Task 12 workaround，最关键）**：Rust 模式 action 分发使用脆弱启发式，存在**静默失败**风险。

链路现状（`renderer.rs:6721-6802` + `mcp_server.rs:1751-1805`）：

1. MCP 侧 `execute_action_vnode` 的 Rust 分支（`mcp_server.rs:1759-1766`）：因 `SharedState` 无强类型 View 树（`shared.view: Option<View<DynamicMessage>>` 在 Rust 模式为 `None`），无法直接提取 `C::Msg`，只能从 VNode 标签取**首词**作为 event 名（`derive_event_name_from_label`，`mcp_server.rs:1788-1805`，如 `"+ New"` → `"New"`）。
2. 该 event 名编码为 `__mcp_action|<event>|<input>` 字符串经 `WrapperMsg::Debug` 通道传递（`renderer.rs:6834-6837`）。
3. iced 侧 `find_msg_by_event_name`（`renderer.rs:6755-6802`）遍历 View 树，用 **`format!("{:?}", onclick).contains(event_name)` 子串匹配**判断 handler。

三个缺陷：
- **子串误匹配**：`"Add"` 会命中任何含 "Add" 的 enum 变体（`AddTodo`/`AddItem`），首个命中即返回，顺序不确定。
- **首词未必命中**：`"+ New"` → `"New"`，但若 enum 变体是 `NewNote`，`{:?}` 里是 `"NewNote"`，`"New".contains` ✓ 偶然命中；若标签首词与变体名无关则**静默失败**（`tool_action` 返回 `ok` 但 `state_changes` 空，`devtools_update` 在 `renderer.rs:6740` 返回 `None` 时直接丢弃）。
- **输入值端到端断裂**：`find_msg_by_event_name` 的 `input_value` 参数命名为 `_input_value`（`renderer.rs:6758`）**完全未使用**。`type_text` 在 Rust 模式实际依赖 Plan 374 的 thread-local `INPUT_TEXT`（`renderer.rs:28`，生成器在 `rust.rs:845` 注入 `last_input_text()`），但 MCP 路径**从未设置** `INPUT_TEXT`，故输入文本丢失。

> 仅影响 Rust 模式；VM 模式走 `DynamicMessage` 提取，干净无误。

**问题 B（Task 10 缺失）**：`autoui_screenshot`（`mcp_server.rs:1154`，`inputSchema.properties == {}`）未加任何 diff 能力。`run_autotest.py:52-93` 有个 post-hoc MD5 hash 存根（在 `run_suite` 完成后跑、只存 hash 不存 PNG、结果不影响退出码），是占位而非实现。

**问题 C（Task 14 部分）**：Rust 模式从不调用 `SharedState::update`（仅 VM 模式 `renderer.rs:3348-3362` 调用），`shared.state` 恒空，`tool_state` 在 `mcp_server.rs:1176-1178` 早退。4 个依赖 state 的 `.autotest` 场景标了 `skip_if rust`（T5c/T11/T11b/T12）。

### 8.3 后续补救计划（Task 19-21）

针对上述三个问题，按优先级实施：

- **Task 19（高）**：消除 Task 12 的 heuristic，改用**路径寻址**精确分发。
- **Task 20（中）**：实现 Task 10 的**像素级截图 diff**（Rust MCP 侧）。
- **Task 21（低）**：为 Rust 模式加**标量状态快照**，去掉 `skip_if rust`。

---

## 9. Task 19: Rust 模式 action 路径寻址（消除 heuristic）

### 9.1 设计思路

放弃「MCP 侧猜 event 名 → iced 侧 Debug 子串匹配」，改为「**MCP 侧只传路径，iced 侧沿 path 精确取 handler**」：

```
vnode.path (Vec<u16>) + action_kind  ──通道──►  iced 侧
                                                  │ find_view_by_path_generic::<M>(&view, path)
                                                  ▼ 精确命中目标 View 节点
                                            取出该节点的 M(handler) ──► w.inner.on(m)
```

关键：**目标 View 节点的 handler 是构造 View 时就内联的强类型 `M`**（生成器 `rust.rs:1600` 发出 `View::input(...).on_change(EditorPanelMsg::EditTitle("".to_string()))`，`View::button("New").on_click(|_| AppMsg::NewNote)`）。沿 path 定位节点即拿到正确的 `M`，无需字符串猜测。

### 9.2 数据通道改造

把 `ActionMessage` 从「(widget, event_name, input_value)」改为**自描述**结构，携带路径：

```rust
// mcp_server.rs
#[derive(Debug, Clone)]
pub struct ActionMessage {
    pub target: ActionTarget,
    pub action: UiActionType,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ActionTarget {
    Event { widget: String, event: String },   // VM 模式（向后兼容）
    Path { path: Vec<u16> },                    // Rust 模式（精确）
}
```

- **VM 模式**：现有调用点改用 `ActionTarget::Event`，行为完全不变。
- **Rust 模式**：MCP 侧从 `vnode.path` 直接构造 `ActionTarget::Path`，不再调 `derive_event_name_from_label`。

iced 侧 `devtools_subscription` 的 `mcp` subscription 改为：路径模式下把 `path`（Vec<u16>）序列化进 payload（如 `__mcp_action_path|0,5,8|type_text|MCP Test`），而非传 event 名。

### 9.3 iced 侧精确分发

新增**泛型** `find_view_by_path_generic`（复用 `vnode_converter.rs:403` 的 `extract_children` 逻辑，提取为 `pub`）+ `extract_handler_from_view`（按 action 类型从 View 取强类型 `M`）。`devtools_update` 的 MCP 分支解析 path → 导航 → 取 handler → 设置 `INPUT_TEXT`（修复输入值断裂）→ `w.inner.on(m)`。

输入值通过 **thread-local `INPUT_TEXT`** 传递（复用 Plan 374 机制，`renderer.rs:28`），生成器已注入的 `last_input_text()`（`rust.rs:845`）会正确读到——**这同时修复了输入值断裂**。

### 9.4 改动文件

| 文件 | 改动 |
|---|---|
| `crates/auto-lang/src/ui/mcp_server.rs` | `ActionMessage`/`ActionTarget` 重构；`execute_action_vnode` Rust 分支改用 `vnode.path`；删除 `derive_event_name_from_label`；VM 调用点改 `ActionTarget::Event` |
| `crates/auto-lang/src/ui/iced/renderer.rs` | `devtools_subscription` 的 mcp 编码改 path；`devtools_update` 加 `__mcp_action_path` 分支；新增 `find_view_by_path_generic` + `extract_handler_from_view`；删 `find_msg_by_event_name`；VM subscription 解码适配 |
| `crates/auto-lang/src/ui/vnode_converter.rs` | 新增 `pub fn extract_children_ref`（引用版本） |

### 9.5 验证

- `cargo build -p auto-lang --features ui-iced` 通过。
- VM 模式 `.autotest` 套件回归：`python tests/run_autotest.py` 仍 13 PASS（`ActionTarget::Event` 向后兼容）。
- Rust 模式手动验证 015-notes：点击 Edit 进入编辑模式；type_text 实际更新标题（验证 INPUT_TEXT 修复）。

---

## 10. Task 20: 像素级截图 diff（Rust MCP 侧）

### 10.1 设计思路

在 Rust MCP 侧用已依赖的 `image 0.25`（`auto-lang/Cargo.toml:111`，`ui-iced` 下启用）做**像素 diff**，而非 Python 侧加 Pillow。RGBA 缓冲区在 `renderer.rs:2441` 已在内存中。

### 10.2 方案

扩展 `tool_screenshot`（`mcp_server.rs:1154`）的 `inputSchema`，新增可选参数 `name`/`baseline`/`diff`/`threshold`。`ScreenshotRequest` 扩展携带这些参数；`save_screenshot_png` 支持命名 + diff（加载基线逐像素对比，差异图存 `tmp/<name>-diff.png`，返回结构化文本 `matches`/`DIFFERS + 百分比`）。

### 10.3 Python 适配器侧

`run_autotest.py` 的 `--screenshot-baseline`/`--screenshot-diff` 改为调用带 `name=<sid>` 的 `autoui_screenshot`，删除 post-hoc MD5 存根，截图捕获移进 `run_suite` 每场景末尾，diff 结果纳入退出码。`McpAdapter` 加 `after_scenario(sid)` 钩子。

### 10.4 改动文件

| 文件 | 改动 |
|---|---|
| `crates/auto-lang/src/ui/mcp_server.rs` | `tool_screenshot` 读 args；`ScreenshotRequest` 加字段；schema 加参数 |
| `crates/auto-lang/src/ui/iced/renderer.rs` | `save_screenshot_png` 支持命名/基线/diff；新增 `compare_pngs` |
| `examples/ui/015-notes/tests/run_autotest.py` | 截图逻辑移进 `run_suite`；baseline/diff 调用带 `name`；纳入退出码 |
| `examples/ui/015-notes/tests/autotest/__init__.py` | `McpAdapter` 加 `after_scenario(sid)` 钩子 |

### 10.5 验证

- `cargo build -p auto-lang --features ui-iced` 通过。
- `autoui_screenshot(name="test", baseline=true)` → 生成 `tests/screenshots/test.png`。
- `autoui_screenshot(name="test", diff=true)` → 无变化 matches；改 UI 后 DIFFERS。

---

## 11. Task 21: Rust 模式标量状态快照（去掉 skip_if rust）

### 11.1 设计思路

`Component` 是本地 trait（`component.rs:34`），可加**带默认实现的可选方法**，blast radius 为零。a2r 生成器在已知字段表（`self.state_types`/`self.prop_types`）时为**标量字段**生成 override。

```rust
pub trait Component: Sized + Debug {
    // ... 现有 ...
    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_val::Value> {
        std::collections::HashMap::new()
    }
}
```

### 11.2 实现要点

- `component.rs`：trait 加 `state_snapshot()` 默认方法（默认空，VM 模式不经过此路径）。
- `rust.rs` 生成器：`generate_component_impl` 新增 `generate_state_snapshot`，仅对标量类型（String/i8..i64/u8..u64/isize/usize/f32/f64/bool）生成条目，跳过 Vec/serde_json::Value/嵌套组件。生成代码通过 `auto_lang::ui::auto_val::Value` 引用（在 `ui/mod.rs` re-export `auto_val`）。
- `mcp_server.rs`：`SharedState::set_state` setter。
- `renderer.rs`：`view_element` 每帧 `set_state(self.inner.state_snapshot())`。
- 重生成 015-notes 后移除 `015-notes.autotest` 中 T5c/T11/T11b/T12 的 `skip_if rust`。

### 11.3 改动文件

| 文件 | 改动 |
|---|---|
| `crates/auto-lang/src/ui/component.rs` | `Component` trait 加 `state_snapshot()` 默认方法 |
| `crates/auto-lang/src/ui/mod.rs` | re-export `auto_val`（供生成代码引用） |
| `crates/auto-lang/src/ui_gen/rust.rs` | `generate_component_impl` + `generate_state_snapshot` + `is_scalar_state_type`/`scalar_to_auto_value_expr` helpers + 单元测试 |
| `crates/auto-lang/src/ui/mcp_server.rs` | `SharedState::set_state` setter |
| `crates/auto-lang/src/ui/iced/renderer.rs` | `view_element` 每帧 push 状态快照 |
| `examples/ui/015-notes/tests/015-notes.autotest` | 移除 T5c/T11/T11b/T12 的 `skip_if rust`（重生成后） |

### 11.4 验证

- `cargo build -p auto-lang --features ui-iced` 通过（默认方法，零回归）。
- 生成器单元测试：`test_state_snapshot_scalar_override`、`test_state_snapshot_no_scalars_no_override`。
- 重生成 015-notes 后 `cargo build` 通过；Rust 模式 `autoui_state(fields=["editing"])` 返回实际值。

---

## 12. 实施顺序与风险

1. **Task 19 先做**（最高优先级，修复静默失败 + 输入值断裂，纯重构不破坏 VM 模式）。
2. **Task 21 次之**（依赖 Task 19 验证 Rust 模式 action 稳定后，再补 state；且要重新生成 a2r）。
3. **Task 20 最后**（独立功能，不依赖前两者；工作量集中在 renderer 截图改造）。

每个 Task 完成后：单独 `cargo build` + 对应验证，再进入下一个。

> ⚠️ 实施记录：Task 19 + Task 21 的代码改动曾因一次 `git stash` 操作与工作区既有 in-flight 改动冲突而丢失，已重新应用。重做时未使用 stash，改为逐文件 Edit。
