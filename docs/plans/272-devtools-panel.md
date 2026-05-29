# Plan 272: Auto-UI DevTools Panel

## 状态: 设计中

## 目标

为 Auto-UI 桌面端实现 Chromium DevTools 风格的调试面板，作为 iced 窗口右侧的可展开/收起面板，支持三大核心功能：

1. **源码查看器**：显示当前 `.at` 文件源码，高亮选中组件对应的代码行
2. **属性检查器**：展示选中组件的完整属性（默认属性、自定义 style、运行时属性）
3. **Console**：捕获并显示 `print()` 输出

## 背景

### 现有基础设施

| 组件 | 位置 | 状态 |
|------|------|------|
| `DebugLayer` | `ui/debug/mod.rs` | 完整：toggle/hover/selection/panel |
| `DebugPanel` | `ui/debug/mod.rs:192` | 完整：NodeInfo + BoxModel + SourceLocation |
| `SourceMap` | `ui/debug/source_map.rs` | API 完整但未被填充 |
| `hit_test()` | `ui/debug/hit_test.rs` | 完整：O(n) 最深层节点查找 |
| `OverlayInfo` | `ui/debug/overlay.rs` | 完整：hover(蓝)/select(橙) 高亮 |
| `NodeInfo` | `ui/debug/inspector.rs` | 完整：widget type + bounds + styles |
| iced hover tooltip | `ui/iced/renderer.rs` | 已完成：F12 开关 + hover tooltip + 蓝色边框 |
| 热重载 | `ui/iced/renderer.rs` | 已完成：500ms 轮询 + 状态迁移 |
| `DebugElementInfo` | `ui/iced/renderer.rs` | 已完成：kind + props 样式摘要 |

### 缺失部分

| 缺失 | 说明 | 难度 |
|------|------|------|
| AST 源码位置保留 | AURA extraction 不保留 line_start/line_end | 中 |
| DevTools 面板 UI | iced 中的右侧面板布局 | 低 |
| print() 输出拦截 | print 输出到 stdout，无 UI 捕获 | 低 |
| click-to-select | hover 存在但 click 选择未实现 | 低 |
| 组件树可视化 | 没有树形结构展示 | 中 |

## 架构设计

### 面板布局

```
┌──────────────────────────────────────────┬─────────────────────┐
│                                          │  Auto DevTools  [×] │
│                                          │─────────────────────│
│                                          │ [源码] [属性] [控制台]│
│          应用主窗口                       │─────────────────────│
│          (可调整大小)                     │                     │
│                                          │  (当前 tab 内容)     │
│                                          │                     │
│                                          │                     │
├──────────────────────────────────────────┤                     │
│ Debug ON | text_0                        │                     │
└──────────────────────────────────────────┴─────────────────────┘
```

- 面板宽度：300px，可通过拖拽调整
- 面板开关：F12（toggle）或点击面板 [×] 关闭
- 面板状态：`PanelClosed` → `PanelOpen`，影响主窗口布局

### 数据流

```
.at 源码
  │
  ▼
Parser ──→ AST (含 Span/行号)
  │
  ▼
AURA Extract ──→ AuraWidget (含 View 模板 + SourceMap)
  │
  ▼
DynamicComponent ──→ view() ──→ AbstractView 树
  │                                    │
  ▼                                    ▼
iced Renderer ←── DebugRenderCtx ──→ wrap_debug()
  │                                    │
  ▼                                    ▼
DynamicState                    element_styles HashMap
  │
  ├── hovered_widget ──→ hover tooltip
  ├── selected_widget ──→ 属性检查器
  ├── source_code ──→ 源码查看器
  └── console_output ──→ Console tab
```

## 分阶段实施计划

### Phase A: 面板框架 + 属性检查器 (最小可用)

**目标**：右侧面板可以展开，显示选中组件的属性信息。

#### A1: click-to-select 机制

**文件**: `crates/auto-lang/src/ui/iced/renderer.rs`

在 `wrap_debug` 的 `mouse_area` 上添加 `on_press` 回调：

```rust
let press_msg = IcedMessage {
    widget: String::new(),
    event: format!("{}{}", DEBUG_SELECT_PREFIX, id),
    input_value: None,
};
let ma = mouse_area(el)
    .on_enter(enter_msg)
    .on_exit(exit_msg)
    .on_move(move |_point| ...)
    .on_press(press_msg);  // 新增
```

在 `DynamicState` 中添加 `selected_widget: RefCell<Option<String>>`，处理 select 消息时设置。选中元素用橙色边框（区别于 hover 的蓝色）。

#### A2: DevTools 面板状态

**文件**: `crates/auto-lang/src/ui/iced/renderer.rs`

扩展 `DynamicState`：

```rust
struct DynamicState {
    // ... 现有字段 ...
    selected_widget: RefCell<Option<String>>,
    devtools_open: RefCell<bool>,
    devtools_tab: RefCell<DevToolsTab>,
    console_output: RefCell<Vec<String>>,
    source_code: RefCell<Option<String>>,
}

enum DevToolsTab {
    Source,
    Properties,
    Console,
}
```

#### A3: 面板布局渲染

**文件**: `crates/auto-lang/src/ui/iced/renderer.rs` 的 `dynamic_view` 函数

当 `devtools_open` 为 true 时，主窗口变为 Row 布局：

```rust
if *state.devtools_open.borrow() {
    // Row: [主内容 + 底部栏] [DevTools面板]
    let main_area = column![rendered, bar];
    let panel = render_devtools_panel(state);
    let layout = row![main_area, panel]
        .width(Fill).height(Fill);
    container(layout).into()
} else {
    // 原有布局
    container(rendered).width(Fill).height(Fill).into()
}
```

#### A4: 属性检查器 Tab

**文件**: `crates/auto-lang/src/ui/iced/renderer.rs`

`render_devtools_panel(state)` 函数渲染属性面板：

```
┌─────────────────────┐
│ element: col #col_1 │  ← 标题
│─────────────────────│
│ ▸ Default Styles    │  ← 可折叠
│   gap: 10           │
│   pad: 20           │
│─────────────────────│
│ ▸ Custom Styles     │  ← 从 style class 解析
│   bg: #ffffff       │
│   align: center     │
│─────────────────────│
│ ▸ Layout            │  ← iced 布局后的实际尺寸
│   (需 backend 回报) │
└─────────────────────┘
```

数据来源：
- **Default Styles**: 从 `AbstractView` 的 legacy 字段提取（spacing, padding, width 等）
- **Custom Styles**: 从 `DebugElementInfo.props` 获取（已有的 `debug_style_props` 函数）
- **Layout**: 需要后端汇报实际渲染尺寸（Phase B）

**验证**：
1. F12 开启 debug → hover 有 tooltip + 蓝色边框
2. 点击元素 → 橙色边框 + 右侧面板打开
3. 面板显示选中元素的所有属性
4. 切换到空白区域 → 面板显示 "无选中元素"

---

### Phase B: Console Tab

**目标**：拦截 `print()` 输出并显示在 Console tab 中。

#### B1: 全局 Console Buffer

**文件**: `crates/auto-lang/src/libs/builtin.rs`

添加全局 console buffer，类似已有的 `TEST_OUTPUT_CAPTURE`：

```rust
thread_local! {
    static CONSOLE_BUFFER: RefCell<Arc<Mutex<Vec<String>>>> =
        RefCell::new(Arc::new(Mutex::new(Vec::new())));
}

pub fn console_output() -> Arc<Mutex<Vec<String>>> {
    CONSOLE_BUFFER.with(|buf| buf.borrow().clone())
}
```

修改 `print` 函数：在写入 stdout 的同时，也写入 console buffer。

#### B2: Console Tab 渲染

**文件**: `crates/auto-lang/src/ui/iced/renderer.rs`

```rust
fn render_console_tab(output: &[String]) -> iced::Element<'static, IcedMessage> {
    let mut col = column([]);
    for line in output.iter().rev().take(100) {  // 最近100条
        col = col.push(text(line).size(11));
    }
    scrollable(col).into()
}
```

需要定时轮询 console buffer（可复用 iced subscription），或者用 `on_press` 手动刷新。

**验证**：
1. 运行有 `print()` 的 Auto 程序
2. DevTools Console tab 中能看到输出
3. 输出与终端同步

---

### Phase C: 源码查看器 Tab

**目标**：显示 `.at` 源码，高亮选中组件对应的代码行。

#### C1: AST Span 保留

**文件**: `crates/auto-lang/src/aura/extract.rs`

当前 AURA extraction 从 AST 提取 widget 时丢失了源码位置信息。需要在提取过程中保留每个 view 节点的行号范围。

方案：在 `AuraNode`（或 `AbstractView`）中添加 `source_span: Option<(usize, usize)>` 字段（line_start, line_end）。

**文件**: `crates/auto-lang/src/ui/view.rs`

```rust
pub struct View<M: Clone + Debug> {
    // 现有 variant 数据...
    // 新增：源码位置（行号范围）
    source_span: Option<(usize, usize)>,
}
```

但这需要修改整个 View 枚举——工作量大。替代方案：在 `DebugRenderCtx` 中维护一个独立的映射。

#### C2: 源码加载与显示

**文件**: `crates/auto-lang/src/ui/iced/renderer.rs`

- 从 `DynamicComponent.source_path()` 获取源码文件路径
- 在 `dynamic_view` 中读取源码（或缓存）
- 渲染为 `scrollable(column![text(line1), text(line2), ...])`
- 选中组件时高亮对应行（背景色）

**难点**：
1. 源码可能较长，需要虚拟滚动或限制显示行数
2. 行号映射需要 Phase C1 的 span 信息
3. 每帧重新读取文件开销大，需要缓存

**验证**：
1. DevTools 源码 tab 显示当前 `.at` 文件内容
2. 选中组件后，对应代码行高亮
3. 修改源码后，热重载时源码 tab 自动更新

---

### Phase D: 组件树可视化（可选）

**目标**：类似 DevTools Elements tab 左侧的 DOM 树。

展示当前页面的组件树层级结构，可展开/折叠，点击选中对应组件。

这个需要 `AbstractView` 树的结构化遍历，复杂度较高，可作为后续增强。

---

## 文件修改清单

| 文件 | Phase | 改动 |
|------|-------|------|
| `crates/auto-lang/src/ui/iced/renderer.rs` | A | 面板框架 + click-select + 属性 tab + console tab |
| `crates/auto-lang/src/libs/builtin.rs` | B | print() 输出拦截到 console buffer |
| `crates/auto-lang/src/aura/extract.rs` | C | AST → AuraWidget 保留源码行号 |
| `crates/auto-lang/src/ui/view.rs` | C | View 添加 source_span 字段 |

## 实施优先级

```
Phase A (面板+属性) → Phase B (Console) → Phase C (源码) → Phase D (组件树)
   ↑ 最小可用            ↑ 快速增值          ↑ 需 parser 改动   ↑ 锦上添花
   1-2 天                半天               2-3 天             后续
```
