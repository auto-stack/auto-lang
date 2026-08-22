# Plan 414: auto-edit 对齐 Zed 的 UX 改进（041 示例 + code_editor widget）

> **状态**: ✅ 四轮全部实施（2026-08-21~22，提交 `d04f7e53`/`2b0fa45a`/`632d4db6`）。遗留：menubar 展开式 MVP（overlay 弹层 Phase B）；toolbar 右对齐被 VM Row 渲染器限制阻塞（§7.2，待修复后一行启用）；action 声明式语法 Phase B（§6.1）。（2026-08-21，分支 `plan-414-auto-edit-ux` 提交 `d04f7e53`，worktree `auto-edit-ux`）—— §1.1-1.3 全部落地并实机验证（tab 切换 / 两位行号槽 / 折叠三角 / 1:1 行列 / terminal 图标开合 Console）；§3 后续项（真折叠 Phase B、深层多 tab、Menu/Toolbar widget）待立项。
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


---

## 5. 第二轮需求（2026-08-21，用户对比实机反馈）

| # | 需求 | 根因/决策 |
|---|------|-----------|
| 1 | 窗口自动置顶挡住其他窗口 | **根因 = 验证脚本** `SetWindowPos(HWND_TOPMOST)`，应用本身无任何置顶设置（renderer 无 with_topmost）。脚本已改为普通前台激活；应用无需改动 |
| 2 | 激活 tab 背景与编辑器内容区一致 + 右侧关闭 icon | 编辑器背景 = CodeEditorTheme::dark 的 `rgb(0.11,0.115,0.14)` ≈ `#1C1D24` → tab 用任意色类 `bg-[#1C1D24]`（class 解析器支持 `bg-[#hex]`）。关闭 icon：tab 名按钮 + 独立 `×` 小按钮拼接（整按钮单 onclick 无法区分点击区域）；MVP 关闭动作 = console dummy 日志 |
| 3 | 行号/gutter 背景与内容框同色 + 三列 padding 一致 | theme.rs `gutter_background` 由 `bg.mix(BLACK,0.35)` 改为与 background 同值；`FOLD_GUTTER_W` 14→19（= 6+图标7+6），布局 `[6][行号][6][折叠7][6][正文]` —— 行号右 pad = gutter 两侧 = 正文左 pad = 6px |
| 4 | console icon 不显示 + 点击无效 | **根因 = size preset `px-4`**：button 默认 preset 带 `h-10 px-4`，`w-7`(28px) 宽度被水平 padding 32px 挤成内容 0 宽（按钮框仍在 → 可 hover，图标不可见）。修复：改用 `icon (name:)` 子组件形式 + 显式 `px-0 py-0`。dummy 日志：run_dynamic_iced 启动时 ui_console_push 引导日志（app 名/端口），面板打开即有内容 |
| 5 | menubar + toolbar（notepad 风格） | **回答：是，AutoUI 目前没有 menu/toolbar widget**（且无 overlay 弹层原语 —— 这是 menubar 下拉的核心缺口）。本轮 MVP：DSL 组合的展开式菜单（点击"文件"在条下方展开条目，推开内容；选中/再点收起）+ toolbar 为 icon-button 行（lucide 补 file-plus/folder-open/save/undo-2/redo-2/scissors/clipboard）。动作全部 console dummy 日志。Phase B（真弹层）路线：自定义 iced Widget::overlay 菜单 widget 或复用 pick_list 管道，见 §3 更新 |

### 5.1 第二轮触点

- `ui/code_editor/theme.rs`：gutter_background = background（dark/light 两套）
- `ui/code_editor/core/render.rs`：FOLD_GUTTER_W = 19.0
- `ui/iced/renderer.rs`：lucide 补 7 个图标；run_dynamic_iced 启动日志 ×2 行
- `examples/ui/041-code-editor/src/front/app.at`：menubar（文件/编辑/视图/帮助，展开式）+ toolbar（8 个 icon 按钮）+ tab 关闭按钮 + console icon 按钮修复（px-0 py-0 + icon 子组件）+ 激活 tab bg-[#1C1D24]


---

## 6. 第三轮需求（2026-08-21，第二轮实机反馈）

| # | 需求 | 决策 |
|---|------|------|
| 1 | toolbar 并入 menu 行 + 分组分隔符做成 widget | 041 布局合并为一行；新增 `sep`/`separator` tag（orientation prop，vertical 默认）——`w-px self-stretch bg-border` 细线容器，用户 class 追加可覆盖（divider/hr 是硬编码横向 h-1，不适合行内） |
| 2 | console icon 点击无反应 | 合成点击实测通过；放大热区（h-6 全状态栏高 × w-9）；**最可能环境因素：F12 调试模式下所有 button 按设计渲染为无 on_press**（inspect 捕获需要）——F12 开着时 menu/toolbar/tab 也全部点不动 |
| 3 | 文本-gutter 间距 < 行号-gutter 间距 | 实测：行号右侧到三角 = 12px（pad6+内衬6），三角到正文 = 6px。修复 = 去中 pad（gutter_total = width + GUTTER_PAD + FOLD_GUTTER_W）+ 行号右对齐 → 两侧各 6px |
| 4 | 行号/gutter/正文纵向不齐 | probe 字体与正文不一致（Monospace vs Consolas）→ 基线错位；统一 mono_family()，数字/三角/正文同字体同行高垂直居中 |
| 5 | menu/toolbar 功能 + **action 概念** | §6.1 设计；Phase A 本轮：语义 action handler 三源绑定（menu/toolbar/`onkeydown.ctrl.*` 全局键——管道现成 Plan 275，编辑器不拦截 Ctrl+S/N/O 穿透）；真实动作 toggle console/switch tab/新建=set_text 清空；其余 dummy 日志 |

### 6.1 Action 概念设计（对比 Event）

- **Event** = 控件原始交互信号（oninput/oncursor），携带控件上下文；**Action** = 语义操作意图（save-file），多触发源可引用、可带参。Action 是 Event 之上的复用层。
- **Phase A（本轮，零新语法）**：语义 handler 即 action（.ActSave 命名约定），menu/toolbar/快捷键三源绑同一 handler。
- **Phase B 提案**：`actions { action save-file { ... } }` 声明块 + `menu-item (action: save-file, shortcut: "Ctrl+S")` 绑定 prop + shortcut 自动注册进 KEYBOARD_BINDINGS + `enabled`/`checked` 表达式（灰态/勾选态）；运行时 action 走 DynamicMessage 同一 dispatch（`__action/` 名空间），parser/view/aura/a2r 四端改造，独立立项。

### 6.2 第三轮触点
aura_view_builder（sep）/ core/render（gutter_total）/ iced/gutter（右对齐+字体）/ core/mod（mono_family pub(crate)）/ 041 app.at（同行工具栏+sep+action 三源+热区）


---

## 7. 第四轮（R3 实机反馈 + 调试发现）

### 7.1 用户反馈落地
- toolbar 并入 menubar 同行 ✓（扁平直接子元素形态）；`auto-edit` 字样移除 ✓
- `sep` widget 交付（orientation prop, vertical=w-px h-4 bg-border, Column 实现）✓
- 状态栏背景改 bg-muted/30（与 menubar 同系，去掉 bg-card 蓝调）✓
- console 图标热区放大 h-6×w-9 ✓

### 7.2 调试发现的 VM 渲染器限制（重要，待立项修复）
**Row 子元素为 Fill 尺寸或 auto margin 时布局爆炸**，实证矩阵：
- Row + `h-full`（Fill 高）子元素 → 其后所有兄弟消失
- Row + `flex-1`/spacer（Fill 宽）子元素 → 其后所有兄弟消失（含嵌套子行形式）
- Row + 容器类子元素带 `ml-auto` → 整窗右半变纯白
- Row + 空文本 `ml-auto` → 无推右效果（疑似零尺寸节点被剪枝）
- Row + 嵌套子行包含 icon-button → 图标消失
- **唯一可靠右推**：有内容的 text 带 `ml-auto`（状态栏 Ln/Col 一直在用）
→ 后果：041 的 toolbar 暂无法右对齐（渲染器修复前）；sep 用固定 h-4 规避 Fill 高度
→ 另：**运行中的 VM 应用会热重载 app.at**（DynamicState.last_modified/dirty），文件编辑期间截图会抓到中间态——验证必须 kill+relaunch 后等 ≥12s

### 7.3 Action Phase A（R3 §6.1）实施 ✓
041 落地 13 个语义 action handler（.ActNew/.ActSave/.../.ActConsole/.ActSwitchTab），三源绑定：menu item / toolbar icon / `onkeydown.ctrl.{n,o,s,j}` 全局键（Plan 275 管道，编辑器不拦截的 Ctrl 组合穿透）。真实动作：toggle console / switch tab / 新建=code_editor_set_text 清空活动 tab；其余 dummy 日志。


---

## 8. 第五轮：Row 右对齐中间层修复（2026-08-22）

### 8.1 调查结论（重要更正 §7.2）
- **Button 臂本就调用 `wrap_with_margin_top`**（renderer.rs Button arm 末尾）——ml-auto 的 Fill+alignRight 包装器对 button 同样生效；§7.2 的"容器类子元素全炸"矩阵受热重载中间态污染，需重新标定
- 实测复现过的可靠事实：**嵌套 Row 里的 icon-button 会消失**（与 ml-auto 无关，V3 对照：嵌套行内 TB1 文本按钮存活、file-plus 图标按钮消失）；扁平直接子元素的 icon-button 正常
- `Space::width(Fill)` 注入方案（本轮流试）在这些行内不扩张，已完整回退
- vue 端 `ml-auto` 为原生 CSS 语义，天然支持（017-chat 先例），无需改动

### 8.2 本轮交付
- 041 app.at：工具栏扁平化 + 首个工具栏按钮 `style: "... ml-auto"`（Tailwind 常规右对齐声明），走 Button 臂既有 wrap_with_margin_top 路径
- 渲染器净变更≈0（注入实验完整回退，Row/Container 臂恢复原状，测试 21+14 全绿）
- **未决**：实机视觉验证因截图捕获不稳定（窗口闪断/帧内容漂移）未能闭环 —— 需人工开 041 确认工具栏位置；若未右推，剩余疑点=嵌套行 icon 消失的同源布局 bug，建议离线布局测试台（iced 无 iced::test，需自建 render_dynamic_view→layout 的 headless 断言）单独立项

---

## 9. VM 窗口标题 pac.at 化（2026-08-22）

### 9.1 问题与方案
- 用户反馈：041 窗口标题是默认 "Auto - App"，应为 **AutoEdit**
- 根因：renderer.rs `title_fn` 硬编码 `format!("Auto - {}", 根组件名)`（041 根组件名 = App）
- 方案：复用 Plan 411 `window: "WxH"` 的同一管线新增 `title:` 字段——pac.at 解析（pac.rs）→ `pac_window_title()`（automan.rs）→ `auto run` 注入 `AUTO_VM_TITLE`（与 AUTO_VM_WINDOW 同路径，VM 渲染器同进程读取）→ renderer.rs 新增 `window_title(fallback)` 优先读 env

### 9.2 交付
- 041 pac.at 加 `title: "AutoEdit"`；未声明/空白的 pac.at 回退原 "Auto - {widget}"，其他示例零影响
- 验证：`cargo check -p auto-man -p auto -p auto-lang` 通过；`cargo test -p auto-lang --lib --features ui-iced iced` 38/38 绿；041 实机 `auto run` 启动日志确认 `VM window title: AutoEdit (from pac.at)`
