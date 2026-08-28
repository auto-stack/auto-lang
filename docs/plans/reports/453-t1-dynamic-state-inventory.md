# Plan 453 T1 报告：DynamicState 字段清点施工图

> **日期**：2026-08-26　**性质**：只读分析（T1 交付物，评审后进入 T2+ 编码）
> **对象**：`crates/auto-lang/src/ui/iced/renderer.rs:5555–5706` 的
> `struct DynamicState`——共 **57 个字段**，另有 **3 项结构外全局态**。
> **结论**：57 字段按"App 域 / 桌面域·DevTools / 桌面域·基础设施 / 窗口级"
> 四组归位；无不可迁移字段；两项合并清理裁定 + 一项清理候选。

## 1. 归域总表

### 1.1 App 域 → `AppSession.state`（13 个）

每 App 私有；454 起随 AppSession 整体进出虚拟窗口。

| 字段 | 说明 | 读点 |
|---|---|---|
| `component` | DynamicComponent（会话核心） | 全局 |
| `input_values` | text_input 文本缓存 event→text | 16 |
| `todos` | VM 外托管的示例状态（见 §3 清理候选） | 28 |
| `source_code` / `source_line_offsets` | 当前 .at 源码及行偏移缓存 | — |
| `live_vtree` / `live_probe` / `live_cache` | 实时检视的渲染侧派生缓存 | — |
| `view_dirty` / `cached_converted_view` | VNode→View 转换管线缓存 | — |
| `cached_rendered` | markdown 渲染缓存 | — |
| `line_to_aura_ids` / `aura_to_id_cache` | 源码行↔widget id 双向映射缓存 | — |

### 1.2 桌面域 · DevTools → `DesktopState.devtools`（36 个）

整组**保形状**搬入 `DevToolsState` 子结构，字段一律不重命名、不改布局逻辑。

- **高亮/选择**：debug_mode, hovered_widget, pending_hovers,
  debug_element_styles, selected_widget, selected_vnode, hovered_vnode,
  inspect_mode, cached_debug_id_map
- **Inspector 面板**：inspector_subtab, inspector_sections,
  inspector_scroll_id, inspector_split_ratio, dragging_inner_divider,
  elements_scroll_id
- **编辑流**：editing_element, edit_textarea_key, edit_span, edit_error,
  last_textarea_key, needs_prompt_refocus
- **Console/日志面板**：console_output, console_buffer,
  blocklist_scroll_id, needs_scroll_to_bottom, last_block_count
- **源码面板**：cached_highlighted
- **组件树/AI 提示**：component_tree, prompt_input_id
- **布局/滚动杂项**：devtools_open, devtools_tab, devtools_panel_width,
  dragging_divider, pending_scroll_to_center, needs_bounds
- **输入修饰键**：current_modifiers（与全局 thread-local 合并，见 §2 裁定 M1）
- **其余**：screenshot_request（截图请求通道；多窗口后需带 window::Id，
  见 §3 备注 N1）

### 1.3 桌面域 · 基础设施 → `DesktopState` 直属（3 个）

| 字段 | 迁移动作 |
|---|---|
| `mcp_shared` | **进程唯一化**——现每次 run_dynamic_iced 起 MCP server（renderer.rs:5852）；迁入 DesktopState 后多会话共享一个实例，启动逻辑加幂等护栏 |
| `toasts` / `toast_next_id` | 原样迁入（桌面层显示） |

### 1.4 窗口级 → `WindowEntry { app_id, … }`（4 个）

挂到 windows 注册表的条目上（注册表本身来自 spike 输入①的设计）：

`window_size`（14 读点）、`pending_window_resize`、`initial_resize_done`、
`initial_focus_done`

## 2. 结构外全局态（3 项，必须一并收敛）

| 全局项 | 位置 | 迁移动作 |
|---|---|---|
| `LAST_MODIFIERS` thread_local | renderer.rs 内多处 | 与字段 `current_modifiers` 合并为 `DesktopState.current_modifiers`（5 处读点，访问器替换） |
| `KEYBOARD_BINDINGS` OnceLock\<Mutex\<HashMap\>\> | renderer.rs:4158 | 迁入 `DesktopState.keyboard_bindings`；keyboard_subscription（:5336）签名已收 `&HashMap`，只需改供给源（3 处读点） |
| 各类 image/svg/byte CACHE OnceLock | :3393/:3422/:3566 | **保留为进程级缓存**（本就跨会话安全），登记不动 |

## 3. 裁定与备注

- **M1（合并裁定）**：`LAST_MODIFIERS` thread-local 与 `current_modifiers`
  字段语义重叠（后者本就是每次 view 从前者刷新而来），迁移时合并为单一
  DesktopState 字段 + 访问器。这一步消除一处可变全局，是 I2 回归的重点盯防点。
- **N1（截图）**：`screenshot_request` 在多窗口下目标必须带上 window::Id，
  改造随窗口注册表一并落（T5 范围）。
- **C1（清理候选，不在本计划实施）**：`todos` 是示例专用状态滞留在通用
  渲染器里的历史遗留（28 读点）。T2 先原样进 AppDomain 保持行为；
  单独开小计划决定去留。
- **数说工作量**：DevTools 组 36 字段保形状搬迁占大头但机械性最强；
  行为敏感的是 §1.3 幂等化、§2 两项全局收敛与 window_size 相关 14 个读点。

## 4. 对后续任务的输入

- **T2** 类型定义可直接引用本表四组：`AppSession.state`（1.1）、
  `DesktopState.devtools`（1.2）、`DesktopState`（1.3）、`WindowEntry`（1.4）。
- **T4**（消息扇出）重点回归：input_values 16 读点、todos 28 读点均改为经
  AppSession 访问；禁止悬空读旧结构体。
- **T1 验收**：以本报告为准；T2 编码期间发现新字段/分歧点须回写本文件。
