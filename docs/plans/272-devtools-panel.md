# Plan 272: Auto-UI DevTools Panel

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 Auto-UI 桌面端实现 Chromium DevTools 风格的调试面板，支持元素检查、组件树、控制台、源码查看与热编辑。

**Architecture:** iced 窗口右侧 300px 面板，F12 开关，三个 tab（元素/检查/控制台）。通过 `DebugRenderCtx` 在渲染 DFS 中收集组件树和样式元数据。

**Tech Stack:** Rust, iced 0.14, AutoLang interpreter

---

## 状态: Phase A-D 已完成，Phase E 规划中

## 已完成

| Phase | 功能 | 状态 |
|-------|------|------|
| A | 面板框架 + click-to-select + 属性检查器 | ✅ 已完成 |
| B | Console Tab（print() 输出拦截） | ✅ 已完成 |
| C | 源码查看器（带行号 + 热重载刷新） | ✅ 已完成 |
| D | 组件树可视化 Elements Tab | ✅ 已完成 |
| - | F12 直接打开面板 + 白色主题 + 合并 Inspector tab | ✅ 已完成 |

## 当前面板布局

```
┌──────────────────────────────────────────┬─────────────────────┐
│                                          │ [元素] [检查] [控制台]│
│                                          │─────────────────────│
│          应用主窗口                       │                     │
│          (白色主题)                       │  (当前 tab 内容)     │
│                                          │                     │
│                                          │                     │
└──────────────────────────────────────────┴─────────────────────┘
```

- Tab: 元素（组件树）/ 检查（属性+源码合并）/ 控制台
- 白色主题，与主 UI 一致
- F12 直接打开面板

---

## Phase E: 源码定位 + 可视化编辑（规划）

### 需求列表

| # | 需求 | 说明 |
|---|------|------|
| E1 | Inspector 分割线 | 属性和源码之间有明显分割线 + "源码" 标题 |
| E2 | 源码语法高亮 | 关键字、字符串、注释、类型等彩色显示 |
| E3 | 点击元素定位源码 | 点击 UI 组件时，源码自动滚动到对应 view 代码行，框选高亮 |
| E4 | 源码组件可编辑 | 框选的源码组件可切换为表单编辑模式 |
| E5 | 编辑写回 + 热重载 | 修改后写回 .at 文件，触发组件重新加载 |

### 难度分析

#### E1: 分割线 — ⭐ 简单（30 分钟）

在 `render_inspector_tab` 中属性和源码之间加一个分隔容器，深色细线 + "源码" 标题文字。纯 UI 调整。

**文件**: `crates/auto-lang/src/ui/iced/renderer.rs` 的 `render_inspector_tab`

#### E2: 源码语法高亮 — ⭐⭐ 中等（半天）

iced 没有内置的代码高亮。方案选择：

| 方案 | 优点 | 缺点 |
|------|------|------|
| **A: 手动 token 着色** | 无依赖，可控 | 需要实现简易 tokenizer |
| **B: tree-sitter** | 精确高亮 | 引入大依赖，编译慢 |
| **C: syntect** | 成熟，TextMate 语法 | 依赖大，嵌入语法文件 |

推荐方案 A：实现一个简易的 AutoLang token 着色器，只处理关键字、字符串、注释、数字、类型名这几种情况。逐行处理，输出带颜色的 `text()` 组件。

**文件**: 新增 `crates/auto-lang/src/ui/iced/source_highlight.rs`

**关键技术点**：
- 用正则或简易状态机逐行扫描
- 关键字（fn, let, var, if, for, col, row, text, ...）→ 蓝色
- 字符串 `"..."` → 绿色
- 注释 `//...` → 灰色
- 数字 → 橙色
- 组件标签（col, row, text, button, ...）→ 紫色

#### E3: 点击元素定位源码 — ⭐⭐⭐⭐ 困难（3-5 天）— 核心阻塞项

**这是整个 Phase E 最难的部分。**

当前 pipeline 完全没有源码位置信息：

```
Parser (有 Pos 但不保留) → ViewNode (无 span) → AuraNode (无 span) → View (无 span)
```

要实现点击 UI 元素定位到源码，需要贯穿整个 pipeline 传播 span：

**改动链**：

| 层 | 文件 | 改动 |
|----|------|------|
| **Parser** | `parser.rs` 的 `parse_view_node` | 在创建 `ViewNode` 时记录 `start_pos` 和 `end_pos` |
| **AST** | `ast/ui.rs` 的 `ViewNode` | 每个 variant 添加 `span: Option<(usize, usize)>` |
| **AURA** | `aura/extract.rs` | `extract_view_node` 传播 span 到 `AuraNode` |
| **AURA 类型** | `aura/types.rs` | `AuraNode` 每个 variant 添加 `span` 字段 |
| **View 构建** | `ui/aura_view_builder.rs` | `convert_node` 传播 span 到 View 或 DebugRenderCtx |
| **Renderer** | `ui/iced/renderer.rs` | `wrap_debug` 从 View/AuraNode 获取 span，存入 `DebugElementInfo` |

**难点**：
1. Parser 中 `parse_view_node` 是递归的，需要在进入和退出时记录位置
2. `ViewNode` 有 7 个 variant，全部需要加 span 字段
3. `AuraNode` 有 7 个 variant，同样全部需要加
4. 所有 match 和构造的地方都需要更新
5. byte offset → 行号的转换需要额外的 source map

**替代方案（降低难度）**：
- **方案 A（精确）**：完整 span 传播链 — 改动量大（~500 行），但结果精确
- **方案 B（启发式）**：用 tag 名 + props 模式在源码中搜索匹配 — 不精确但实现快（~100 行）
- **方案 C（混合）**：只在 `ViewNode` 和 `AuraNode` 层加 span，View 层用 side-channel 映射

#### E4: 源码组件可编辑 — ⭐⭐⭐ 困难（2-3 天）— 依赖 E3

需要 E3 的 span 信息来确定可编辑区域。UI 部分：
- 源码中被框选的组件显示为可点击区域
- 点击后切换为表单：每个 prop 变成一行 input（key: value）
- 需要处理 AutoLang 的属性语法（`key: value`, `key: {expr}`, `class: "..."` 等）

**难点**：
1. 属性值的解析和序列化（表达式 vs 字面量 vs f-string）
2. 编辑后需要重新生成有效的 AutoLang 源码片段
3. 嵌套组件的边界识别

#### E5: 编辑写回 + 热重载 — ⭐⭐ 中等（1-2 天）— 依赖 E3 + E4

**已有基础**：热重载机制完整（500ms 轮询 + mtime 检测 + 全量重解析 + 状态迁移）。

**需要的额外工作**：
1. 将编辑后的源码片段替换回原文件对应位置
2. 触发热重载（修改 mtime 或直接调用 reload）
3. 确保编辑后的组件 ID 稳定（当前 counter 每帧重置，已 OK）

**难点**：
1. 精确替换文件中的字节范围（需要 E3 的 span）
2. 多次编辑的增量更新 vs 全量替换
3. 替换后行号偏移的修正

### 实施优先级

```
E1 (分割线)     → E2 (语法高亮) → E3 (源码定位)  → E4 (可编辑) → E5 (写回+重载)
   ⭐ 简单           ⭐⭐ 中等        ⭐⭐⭐⭐ 最难       ⭐⭐⭐ 困难    ⭐⭐ 中等
   30 min            半天             3-5 天          2-3 天       1-2 天
```

**建议**：E1 和 E2 可以立即实施。E3 是核心阻塞项，决定后续 E4/E5 是否可行。如果 E3 采用方案 B（启发式），整个 Phase E 可以在 3-4 天内完成；方案 A（精确 span）则需要 7-10 天。

### 技术决策点

| 决策 | 选项 | 推荐 |
|------|------|------|
| 语法高亮方案 | A(手动) / B(tree-sitter) / C(syntect) | A — 简易够用 |
| 源码定位方案 | A(精确span) / B(启发式) / C(混合) | A — 长期正确 |
| 编辑模式 | 纯文本编辑 / 表单编辑 / 混合 | 表单编辑 — 更安全 |
| 热重载触发 | mtime轮询 / notify监听 / 手动触发 | mtime轮询 — 已有 |

## 文件修改清单（Phase E）

| 文件 | 任务 | 改动 |
|------|------|------|
| `crates/auto-lang/src/ui/iced/renderer.rs` | E1 | Inspector 分割线 |
| `crates/auto-lang/src/ui/iced/source_highlight.rs` | E2 | 新增语法高亮模块 |
| `crates/auto-lang/src/ast/ui.rs` | E3 | ViewNode 添加 span 字段 |
| `crates/auto-lang/src/parser.rs` | E3 | parse_view_node 记录位置 |
| `crates/auto-lang/src/aura/types.rs` | E3 | AuraNode 添加 span 字段 |
| `crates/auto-lang/src/aura/extract.rs` | E3 | 传播 span |
| `crates/auto-lang/src/ui/aura_view_builder.rs` | E3 | 传播 span 到渲染层 |
| `crates/auto-lang/src/ui/iced/renderer.rs` | E3-E5 | DebugElementInfo 添加 span，编辑 UI |
| `crates/auto-lang/src/ui/dynamic.rs` | E5 | 文件写回 + reload 触发 |
