# Plan 280: MCP UI 渲染问题检测

## Context

Plan 278-279 实现了 MCP 工具（autoui_snapshot, autoui_inspect, autoui_action），snapshot 从 AuraNode 树生成 AURA 源码风格输出。但测试 014-weather 时发现 **MCP 无法反映实际 UI 渲染问题**：

- `grid` + `grid-item` 在 aura_view_builder.rs 中无 converter → 静默 fallback 为 Column → UI 显示竖排
- MCP snapshot 仍显示 `grid #aura_16 { cols: 7 }` 看起来正常
- AI 认为一切正确，实际 UI 完全不对

**核心矛盾**：MCP snapshot 展示的是 AuraNode 结构（"源码说了什么"），而非 iced 后端的实际渲染结果（"用户看到了什么"）。

## 方案：双管齐下

### A. Snapshot 内联渲染状态标注（autoui_snapshot 增强）

在 `autoui_snapshot` 输出中，为每个节点标注 iced 后端的实际支持状态：

```
grid #aura_16 {
  // ⚠ FALLBACK: iced 后端无 grid 布局，渲染为竖向 Column; props "cols", "gap" 被忽略
  cols: 7
  gap: 0
  ...
}
button #aura_4 "Add" {
  style: "..."
  onclick: .AddTodo
  // ⚠ PARTIAL: "disabled" prop 未实现
}
text #aura_3 "Beijing" {
  style: "text-2xl font-bold text-gray-800"
  // ✅ OK
}
```

默认启用标注（`include_status: true`），AI 一眼就能看到哪些节点有问题。

### B. 新增 `autoui_check` 诊断工具

独立诊断工具，汇总所有渲染问题：

```
AutoUI Render Check
widget: "App"

Issues: 2 errors, 1 warning

[ERROR] #aura_16 grid — FALLBACK (renders as Column)
  Props ignored: cols, gap, style
  Children: 32 grid-item nodes will stack vertically
  Fix: Replace with nested row/col layout

[ERROR] #aura_17 grid-item — FALLBACK (renders as plain child)
  Grid-item is meaningless without grid parent
  Fix: Use row/col children instead

[WARN] #aura_4 button — PARTIAL
  "disabled" prop not handled; button always clickable

Summary: 2 errors, 1 warning, 45 OK elements
```

## 实现步骤

### Step 1: 新建 `render_support.rs` — 渲染支持注册表

**新建文件**: `crates/auto-lang/src/ui/render_support.rs`

静态注册表，记录每个 AURA tag 在 iced 后端的支持级别：

```rust
pub enum SupportLevel {
    Full,       // 完全支持（col, row, text, button 等）
    Partial,    // 部分支持（某些 props 被忽略）
    Fallback,   // 降级处理（grid → Column，scroll → Column）
    Unsupported, // 完全不支持
}

pub struct TagSupport {
    pub level: SupportLevel,
    pub ignored_props: Vec<&'static str>,  // 被忽略的属性
    pub note: &'static str,                // 人可读的说明
}

pub fn get_support(tag: &str) -> TagSupport;
```

支持的 tag 注册表（基于 `aura_view_builder.rs` convert_element 的 match 分支）：

| Tag | Level | 忽略的 Props | 说明 |
|-----|-------|-------------|------|
| `col`/`column` | Full | — | 核心布局 |
| `row` | Full | — | 核心布局 |
| `text`/`label`/`h1`/`h2`/`h3`/`p`/`span` | Full | — | 核心文本 |
| `button`/`btn` | Partial | `disabled` | 按钮始终可点击 |
| `center` | Full | — | 映射为居中 Container |
| `input` | Partial | `type`, `maxlength` | 基本输入支持 |
| `textarea` | Partial | 大部分 style | 有限样式支持 |
| `checkbox`/`check` | Full | — | 核心组件 |
| `container`/`div` | Full | — | 核心布局 |
| `img`/`image` | Partial | `src` (占位符) | 仅显示占位符 |
| `progress` | Full | — | 映射为 ProgressBar |
| `spacer` | Full | — | 弹性空白 |
| `divider`/`hr` | Partial | 自定义 style | 硬编码样式 |
| `avatar` | Partial | 大部分 props | 彩色圆形占位 |
| `grid` | **Fallback** | `cols`, `gap`, `columns`, `rows` | **降级为竖向 Column** |
| `grid-item` | **Fallback** | 所有 props | **无 grid 时无意义** |
| `scroll` | **Fallback** | 所有 scroll props | **降级为 Column** |
| 其他 | **Fallback** | 所有 props | **走 `_` 分支** |

### Step 2: 修改 `aura_snapshot_builder.rs` — 内联标注

修改 `AuraSnapshotBuilder`：

1. 添加 `include_status: bool` 字段
2. `traverse` 方法中，对 `AuraNode::Element` 分支，在 opening tag 行后插入状态注释
3. 格式：`  // ⚠ FALLBACK: <描述>` 或 `  // ⚠ PARTIAL: <描述>`，Full 级别不标注

```rust
// 在 traverse() 的 Element arm 中，opening tag 后面：
if self.include_status {
    let support = render_support::get_support(tag);
    if support.level != SupportLevel::Full {
        let icon = match support.level {
            SupportLevel::Fallback => "⚠ FALLBACK",
            SupportLevel::Partial => "⚠ PARTIAL",
            SupportLevel::Unsupported => "✗ UNSUPPORTED",
            _ => unreachable!(),
        };
        out.push_str(&format!("{}// {} {}\n", "  ".repeat(indent + 1), icon, support.note));
    }
}
```

### Step 3: 修改 `mcp_server.rs` — 更新 snapshot 工具 + 新增 check 工具

**修改 `autoui_snapshot` 工具**：
- 新增 `include_status` 参数（默认 true）
- 传递给 `AuraSnapshotBuilder`

**新增 `autoui_check` 工具**：
- 参数：无（自动检查当前 view_template）
- 实现：遍历 AuraNode 树，用 `render_support::get_support()` 收集所有问题
- 输出格式化的诊断报告
- 在 `tool_definitions()` 添加工具 schema
- 在 `dispatch_tool` 添加路由

### Step 4: 导出模块

**修改 `crates/auto-lang/src/ui/mod.rs`**：添加 `pub mod render_support;`

## 关键文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `crates/auto-lang/src/ui/render_support.rs` | 新建 | 渲染支持级别注册表 |
| `crates/auto-lang/src/ui/aura_snapshot_builder.rs` | 修改 | 添加 include_status，内联标注 |
| `crates/auto-lang/src/ui/mcp_server.rs` | 修改 | snapshot 加参数 + 新增 autoui_check 工具 |
| `crates/auto-lang/src/ui/mod.rs` | 修改 | 导出 render_support 模块 |

**参考文件（只读）**：
- `crates/auto-lang/src/ui/aura_view_builder.rs` — convert_element match 分支是注册表的数据来源
- `crates/auto-lang/src/ui/view.rs` — View enum 定义了可用的组件类型

## 验证方案

1. **编译**: `cargo build --bin auto`
2. **运行 016-calendar**（使用 grid）: `auto ./examples/ui/016-calendar/src/front/app.at`
3. **MCP snapshot 验证**: `autoui_snapshot` 应在 grid 节点旁显示 `⚠ FALLBACK` 标注
4. **MCP check 验证**: `autoui_check` 应报告 grid 和 grid-item 的 fallback 问题
5. **运行 014-weather**: 验证 row/col/button 等正常组件不显示警告
6. **运行 013-todo**: 验证已有功能（action、state 求值）不受影响
