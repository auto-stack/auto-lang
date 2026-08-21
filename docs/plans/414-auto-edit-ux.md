# Plan 414: auto-edit 对齐 Zed 的 UX 改进（041 示例 + code_editor widget）

> **状态**: ✅ 本轮范围实施完成（2026-08-21，分支 `plan-414-auto-edit-ux` 提交 `d04f7e53`，worktree `auto-edit-ux`）—— §1.1-1.3 全部落地并实机验证（tab 切换 / 两位行号槽 / 折叠三角 / 1:1 行列 / terminal 图标开合 Console）；§3 后续项（真折叠 Phase B、深层多 tab、Menu/Toolbar widget）待立项。
> **背景**: 用户对比 auto-edit（041）与 Zed 截图提出 7 项改进。逐项分析后：1-4 本轮实施；5 做 Phase A（gutter 列 + 折叠标记视觉，点击折叠为 Phase B）；6 做 MVP（DSL 级双 tab，状态按 key 独立）；7 仅设计（AutoUI VM 后端确认无 Menu/Toolbar widget）。
> **上游**: Plan 413（code_editor widget）、`6b8ec73c`（状态栏 + Console 面板）、`6b70dc85`（状态栏样式修正）。

---

## 0. 需求清单与逐项分析

| # | 需求 | 决策 | 依据 |
|---|------|------|------|
| 1 | 移除 Wrap 开关 | ✅ 本轮 | DSL 删除；编辑器 wrap 恒 false |
| 2 | Console 开关改 icon | ✅ 本轮 | 按钮默认 variant=primary（紫色填充，aura_view_builder.rs:3124 preset 表）→ 需 `variant: "text"` + `icon: "terminal"`（lucide 集需补 terminal 图标） |
| 3 | 行列显示 `8:1` 格式 | ✅ 本轮 | `${.line}:${.col}`；col 在 handler 里 +1（Zed 的 col 1-based，当前 native 返回 0-based char col） |
| 4 | 行号槽最小两位宽 | ✅ 本轮 | core/render.rs `digits_of(line_count)` → `.max(2)`；宽度随最大行号自适应已有，只补下限 |
| 5 | 行号与正文之间留 fold gutter | ✅ Phase A | 预留 14px 列 + 块起始行画折叠 chevron（启发式：trim 后以 `{` 结尾且有后续行）；点击折叠为 Phase B（需按行隐藏渲染，fill_raw 整缓冲绘制做不到，见 §3） |
| 6 | 多 tab 编辑 | ✅ MVP | DSL 级：tab 条（row + 条件按钮）+ 每个 tab 独立 key 的 code_editor（registry 按 key 存状态，切换不丢）。深层（打开/关闭文件、拖拽排序、+按钮）为后续 |
| 7 | Menu / Toolbar | ○ 设计 only | **确认：AutoUI VM 后端两者皆无**（aura_view_builder tag 分发表无 menu/toolbar，render_support.rs 支持登记表也没有）；vue 侧同样未登记。设计见 §4 |

### 0.1 过程发现（记入已知坑）

- **button 默认 variant = primary（主题紫填充）**：`variant` 缺省映射 `bg-primary text-primary-foreground h-10 px-4`（aura_view_builder.rs:3124-3133）。状态栏/工具条里的轻量按钮必须显式 `variant: "text"` 才是 chromeless 纯文字。用户看到的"紫色 Console 按钮"即此。
- **text 节点不支持 padding**（`px-2` 静默丢弃，6b70dc85 已踩）：间距只能靠容器 `gap-N`。
- **`font-mono` 的 text 走代码高亮路径**，颜色类被忽略（6b70dc85 已踩）：状态栏文字禁用 font-mono。

## 1. 本轮实施范围（worktree plan-414-auto-edit-ux）

### 1.1 DSL 侧（041 app.at 重写）

- **Tab 条**（顶部，h-8，bg-muted/30）：`main.at` / `util.at` 两个 tab 按钮（`variant: "text"`），激活态 bg-card + text-zinc-200 + font-medium，非激活 text-zinc-500；条下 h-px bg-border 分隔
- **双编辑器**：`if .tab == 0 { code_editor (key: "tab-main", content: .src_main, oninput/oncursor 各自事件) }` / `if .tab != 0 { key: "tab-util", ... }`；切换 handler 同时拉取新 tab 的 cursor natives 刷新状态栏
- **状态栏**：删 Wrap 段与 ToggleWrap handler；Ln/Col 改 `${.line}:${.col}`（col handler 内 +1）；Console 开关改 `button (icon: "terminal", variant: "text")`
- wrap model 变量与 `wrap: .wrap` prop 一并移除

### 1.2 Rust 侧

- `ui/iced/renderer.rs` lucide_svg 补 `terminal` 图标（lucide 官方 path：`<polyline points="4 17 10 11 4 5"/><line x1="12" x2="20" y1="19" y2="19"/>`）
- `ui/code_editor/core/render.rs`：
  - `digits = digits_of(line_count).max(2)`（§0 #4）
  - 新增 `FOLD_GUTTER_W: f32 = 14.0`；gutter_total = 数字宽 + GUTTER_PAD*2 + FOLD_GUTTER_W；text_rect 右移
  - 折叠启发式 `folds_openers(lines) -> Vec<bool>`（trim 以 `{` 结尾且非末行）；可见 run 扫描时把 opener 行的 y 收进 `GutterSection.folds`
- `ui/code_editor/draw.rs`：`GutterSection` 增 `folds: Vec<f32>`（opener 行 y，Phase A 恒展开态）
- `ui/code_editor/iced/gutter.rs`：光栅图宽度含折叠列；在折叠列 x 区间为每个 fold y 画 7px 向下小三角（手写扫描线填充，与数字同色）；缓存键含 revision（文本变化自动重画）

### 1.3 测试

- core 层：单行/少行文档 render 后 `gutter.digits == 2`；含 `{` 块的文本 `gutter.folds` 非空且行号正确；无 `{` 时空
- 回归：code_editor 全套 + console natives 套件

## 2. 验证

worktree 内 `cargo build -p auto` → 跑 041（VM 模式）→ 截图核对：tab 条切换状态保持、状态栏 `1:1` 格式、行号槽两位宽、`{` 行左侧出现 chevron、Console 为终端图标扁平按钮 → 提交 worktree 分支。

## 3. 后续项（本轮不做，含关键阻塞点）

- **§5 Phase B 真折叠**：需按行隐藏渲染。当前正文走 `fill_raw(Raw{buffer})` 整缓冲一次绘制，无法跳行；可选路线 ① core 层自绘正文（放弃 fill_raw，逐 run 光栅化，工程量大）② 折叠时改写 buffer 文本 + 恢复（污染 undo 历史，不可取）③ 上游 cosmic-text 折叠支持（无）。倾向 ①，待单独立项
- **§6 深层多 tab**：tab 关闭按钮、`+` 打开文件（需文件选择 dialog widget）、tab 拖拽排序、脏标记（文件 I/O 管道：DSL 目前无文件读写 natives）
- **§7 Menu / Toolbar widget 设计草案**（AutoUI 缺口，跨 VM/vue/a2r 三端）：
  - `menubar { menu (text: "File") { menu-item (text: "Open", onclick: .Open, shortcut: "Ctrl+O") ... } }` —— 语义 tag + 下拉弹层（VM 侧需 popup/overlay 层，现有 widget 无弹层原语，是主要工作量；code_editor 的 context_menu 回调已示范了锚点定位需求）
  - `toolbar { tool-button (icon: "...", variant: "text") divider ... }` —— 组合上 row+icon button 即可达成，语义 tag 的价值在于分隔线/分组/溢出菜单的默认样式与 a2r 代码生成
  - 建议作为独立 Plan（涉及 view.rs / aura_view_builder / renderer / ui_gen/rust.rs / vue map_tag 全链路）
