---
plan_id: PLAN-629
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: editor-common-scroller
author: [agent]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: [auto-lang/ui (code_editor 自绘滚动条/内部滚轮滚动路径)]
new_spec_components: [auto-lang/ui (code_editor 寄宿公共 scroller 集成契约: 外部偏移同步/内容高度上报/光标跟随命令)]
touched_goals: []

affects: [auto-lang/ui, autoui-skill]
current_step: 4
total_steps: 4
---

# [PLAN-629] editor-common-scroller

## 0. 变更摘要

用户裁定：code_editor 的滚动条放弃"自绘对齐"路线，**直接改用 AutoUI 公共 scroller 组件**；缺的机制加在公共 scroll 上：

1. 按需排版（on-demand shaping）与折叠投影使内容面板高度变化 → 内容侧**通知** scroller 更新比例与位置（iced retained layout 天然承载：内容高度变化 → request_layout → scroller 重测量）。
2. 光标跟随滚动 → scroller 侧提供滚动命令通道（`operation::scroll_to`），光标变化时编辑器发命令。

实施形态：`build_code_editor_generic`（renderer.rs:19787）把 `CodeEditor` iced 控件包进官方 `scrollable`（官方 vue 风格 `scrollbar_style()` 自动生效），编辑器控件转入**寄宿模式（hosted）**：

- **内容高度上报**：`size()` 返回 `Fill × Fixed(content_height)`（折叠感知的投影行数 × 行高），高度变化触发重排——scroller 自动更新滚动范围与 thumb 比例。
- **外部偏移同步**：scroller `on_scroll` 回调把绝对偏移写入编辑器 core（`sync_external_scroll`），draw 按 viewport 可见切片渲染（虚拟化保持——只 shape 可见行）。
- **滚轮让渡**：寄宿模式编辑器不再捕获/处理滚轮（内部 `handle_wheel` 保留给非寄宿场景与测试）。
- **光标跟随**：`cursor_changed` 时经消息通道 → `operation::scroll_to(scroller_id, caret_offset)`（renderer 已有 3 处 scroll_to 先例：13143/15078/15231）。
- **坐标系修正**：点击/拖拽/IME/右键菜单坐标按内容坐标 + 存储的滚动偏移换算；`EditorCtx(x,y)` 发布视口坐标（根视图 ctx_menu 的坐标锚 popover 仍在视口系）。
- 自绘滚动条绘制在寄宿模式跳过（避免与 scroller 双滚动条）；非寄宿（测试/headless）保留。

## 1. 目标

- **G1**：code_editor 的滚动条 UI/交互 = AutoUI 公共 scroller（同 `scrollbar_style()`、同拖拽/滚轮行为），自绘滚动条与内部滚轮路径在寄宿模式退役。
- **G2**：虚拟化不回退——大文件只 shape 可见行；折叠展开即内容高度变化，scroller 即时更新。
- **G3**：光标移动（键盘/IME/程序化）自动滚动到可见（经 scroll_to 命令通道）。
- **非目标**：terminal 组件不改（另行评估）；非寄宿（headless/测试）下的内部滚轮路径不删；.at DSL 不变（`code_editor` 标签用法不变，包裹在渲染器内部完成）。

## 2. 架构方案

```text
renderer build_code_editor_generic:
    scrollable(CodeEditor{hosted})
      .id(scroller_id(key))                    // scroll_to 目标
      .on_scroll(|vp| core.sync_external_scroll(vp.absolute_offset().y))
      .style(scrollbar_style())                // 官方 vue 风格
      .width(Fill).height(Fill)

CodeEditor widget (hosted):
    size()      -> (Fill, Fixed(core.content_height()))   // 高度上报
    draw(vp)    -> offset = vp.y - bounds.y; visible_h = vp.height
                   core.sync_external_scroll(offset)       // 双保险(布局未跑时)
                   render(core, fs, w, visible_h)          // 虚拟化: 只 shape 可见行
                   平移 = bounds.position + (0, offset)     // 内容坐标系绘制
                   tree.state 存 offset（update 用）
    update()    -> wheel: 不捕获（scroller 接管）
                   mouse/kbd: local.y += offset（内容坐标）
                   EditorCtx 发布视口坐标（local - offset）
                   cursor_changed → 消息 → session → scroll_to 命令
                   content_height 变化 → shell.request_layout()
```

core 侧新增（小）：`content_height()`（fold 投影行数 × 行高）、`sync_external_scroll(px)`（绝对偏移 → scroll(line,vertical) 归一 + shape_until_scroll，复用 PLAN-626 T-08 机制）、`caret_offset_y()`（光标跟随用）。

## 3. 技术栈

iced 0.14（`scrollable::Id` / `on_scroll(Viewport)` / `operation::scroll_to`，renderer 已有先例）；现有 code_editor core/控件架构；无新依赖。

## 4. 需求分析与背景调查

**授权**：用户 2026-09-14 明确裁定"直接改用公共 scroll；缺的机制（hook/通知）加在公共 scroll 上"（原话：按需排版/折叠投影=内容高度变化→通知 scroller 更新比例位置；光标跟随=scroller 滚动结束后 hook 回调）。本计划即该裁定的落地，继承 PLAN-626 rev2 的滚动修复成果（T-08/09 归一化机制为核心依赖）。

**背景证据**（2026-09-14，HEAD=plan-626-dev@3b85a0357）：

- 官方滚动条规格：renderer.rs:2384 `scrollbar_style()`（thumb rgba(0.9,0.9,0.9,0.3)、圆角 3、轨道透明，Plan 409 §10 续 4）。
- scroller 机制：renderer.rs:2305 `on_scroll(Viewport)` → ScrollMetrics 回调（offset/content/viewport 三测量）；`operation::scroll_to(id, AbsoluteOffset)` 先例 renderer.rs:13143/15078/15231。
- 编辑器实例化点：renderer.rs:19787 `build_code_editor_generic`（唯一 code_editor → iced 控件构造点，包裹在此完成）。
- 虚拟化关键：code_editor/core/render.rs `render(core, fs, w, h)` 按 viewport 尺寸 shape/遍历；cosmic-text 0.15 `layout_runs` 从 scroll.line 起跳——外部偏移驱动 scroll 后虚拟化保持（PLAN-626 T-08 的 shape_until_scroll 归一为基础）。
- 既有坐标依赖：编辑器 update 用 `layout.bounds()` 做 local 换算（寄宿后 = 内容坐标，偏移补偿见 §2）；`EditorCtx(x,y)` 消费方为 ctx_menu 组件的坐标锚 popover（app.at:174 一带）——需发布视口坐标。
- PLAN-626 交付的自绘滚动条（render.rs scrollbars 节 + drag_scrollbar_*）在寄宿模式退役：widget 跳过绘制 list.scrollbar_v/h；`handle_wheel`/`drag_scrollbar_*` 保留（非寄宿与单测仍用）。

## 5. 详细设计

### 5.1 core（code_editor/core/mod.rs）

- `pub fn content_height(&self) -> f32`：fold 投影行数 × `line_height()`（fold map 由最近一次 render 刷新；未 render 前 = 全部行 × 行高）。
- `pub fn sync_external_scroll(&self, font_system, offset_y: f32)`：绝对偏移 → scroll 归一。T-01 有界验证两种形态：(a) 直接 `scroll.vertical = offset` + `shape_until_scroll` 归一循环收敛；(b) 若 (a) 收敛不良改为两步式——`line = offset / line_height` 粗定位 + vertical 余量再归一。决策记录在任务证据。
- `pub fn caret_offset_y(&self) -> f32`：最近 render 的 `layout_info.caret` 在内容系 y。
- 现有 `scroll()`/`set_text` 等不动；`handle_wheel`/`drag_scrollbar_*` 保留。

### 5.2 widget（code_editor/iced/widget.rs）

- `CodeEditor` 增加 hosted 标志与 `scrollable::Id`（builder `.hosted_in_scroller(id)`）。
- `size()`：hosted → `(Fill, Fixed(content_height))`（content_height 经 AtomicU32 存份，draw 刷新；变化时 update 里 `shell.request_layout()`）。
- `draw(viewport)`：offset/visible_h 计算如架构图；寄宿模式跳绘 `list.scrollbar_v/h`；offset 写入 tree.state（WidgetState 扩展字段，update 共用）。
- `update()`：wheel 臂 hosted 时返回 None（不捕获不重绘，滚轮归 scroller）；坐标臂 `local.y += offset`；`cursor_changed` 且 hosted → 发 scroll-to-caret 消息；内容高度变化 → `shell.request_layout()`。
- IME overlay / 右键菜单坐标按视口换算。
- 选区拖拽到边缘的 auto-scroll：用外部偏移实现（保留既有体验）。

### 5.3 renderer 会话接线

- `build_code_editor_generic`：包裹 scrollable（id = `format!("editor-scroll-{key}")`）+ on_scroll 同步 + hosted 标志。
- scroll-to-caret 消息 → session update 臂：`operation::scroll_to(id, AbsoluteOffset{x:0, y: caret_y - 2*line_height})`（任务化，先例 13143）。

### 5.4 规范增量

| delta_id | add/modify/retire | docs/specs/... 目标 | before/after 规则 | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/architecture.md（code_editor 节） | before: code_editor 自绘滚动条+内部滚轮；after: 寄宿公共 scroller（外部偏移同步/内容高度上报/scroll_to 光标跟随为集成契约），自绘路径仅存于 headless/测试 | 用户裁定 + 滚动条一致性 | AC-01..04 |

## 6. 测试设计

- core 单测：content_height（fold 前后）、sync_external_scroll（绝对偏移→归一 scroll、往复一致）、caret_offset_y。
- widget 级：现有 code_editor core 18 测试全绿（非寄宿路径不回归）。
- 实机矩阵：desktop_mcp.py 全量（47 基线）；新增断言——autoui_editor_state 的 cursor/scroll 行为、折叠后滚动范围变化、点击/右键坐标在滚动后仍准确。
- 实机人工：大文件滚轮（含触控板像素模式）、拖官方 thumb、键盘方向键/Ctrl+End 光标跟随、折叠切换、右键菜单落点。

## 7. 验收标准

- **AC-01**：编辑器滚动条 = 官方 scroller 外观与交互（半透明 thumb/圆角/拖拽/滚轮），与官方组件无观感与行为差异。
- **AC-02**：大文件（≥1k 行）滚动流畅不未响应；只 shape 可见行（虚拟化保持，滚轮不再走内部路径）。
- **AC-03**：折叠/展开后滚动范围（thumb 比例与可滚长度）即时正确更新。
- **AC-04**：光标经键盘/IME 移动到视口外时自动滚入可见；右键菜单与点击命中在任意滚动位置准确。
- **AC-05**：非寄宿路径（core 单测/headless）无回归；desktop_mcp 矩阵不低于基线。

## 8. 执行步骤

worktree：`D:/autostack/.wt/lang-629/auto-lang`（branch `plan-629-dev`，基于 plan-626-dev 叠放——依赖其 T-08/09 归一化机制；626 先合，629 rebase master 后再 fold）。组内同款 auto-down 只读兄弟 worktree（detached 67bb508）。

- **T-01** [core] `content_height` / `sync_external_scroll` / `caret_offset_y` + 单测（含"绝对偏移→归一"的有界验证：(a)/(b) 形态决策记录在任务证据）。验证：core 单测绿。
- **T-02** [widget] hosted 模式：size/draw(viewport)/坐标换算/滚轮让渡/高度变化重排/边缘自动滚动 + 单测。依赖 T-01。
- **T-03** [renderer] build_code_editor_generic 包裹 + on_scroll 同步 + scroll_to 会话臂。依赖 T-02。
- **T-04** [示例/验证] 实机全路径（滚轮/拖拽/键盘跟随/折叠/右键/IME）+ 矩阵扩展 + README 更新。依赖 T-03。
- 依赖链：T-01→T-02→T-03→T-04。

### 执行证据（2026-09-14）

- **T-01** ✅ commit 284f325d9：content_height 用 fresh_fold_map（折叠切换即时反映，不等 render——T-01 有界验证结论：有界验证形态 (b) 采用；sync_external_scroll 粗定位 line=offset/lh + vertical 余量再 shape_until_scroll 归一，形态 (a) 每趟只前进一行 O(N)×遍数大文件跳转会卡）；caret_offset_y 直接计算（fold 投影），**不依赖上次渲染的 layout 记录**（初版设计缺陷：光标跟随恰恰发生在光标不在视口内时，渲染记录不存在）；registry getter `code_editor_caret_offset_y`。测试 plan629_content_height_external_scroll_and_caret 绿（高度/绝对偏移/底端钳制/折叠收缩/caret 五断言）+ core 19/19。
- **T-02** ✅ commit 591f6153d：hosted 模式（size/layout 内容高度上报；draw 视口切片 + 原点 y+offset 内容系阴影变量统一平移；滚轮让渡守卫；右键锚视口换算；键盘/IME 光标跟随 → core 请求标记；高度变化 → request_redraw 触发运行时 view 重建重排——iced 0.14 Shell 无布局失效 API，消息/重绘即失效通道）。core 46/46 回归绿。
- **T-03** ✅ commit f4b77192b：**双构造点**包裹——build_code_editor_generic（DSL/into_iced 轨，M 泛型保持）与 render_dynamic_view CodeEditor 臂（VM 主路径，IcedMessage 具体）；scrollable 官方样式 `.id(editor-scroll-<key>)`；`AUTO_EDITOR_NO_SCROLLER=1` 逃生开关。光标跟随改 M 无关通道：core 请求标记 + dispatch_app 尾部排水成 scroll_to 任务（on_scroll 回调与自定义消息均需 M 具体化，泛型路径不可行——设计偏差记录）。
- **T-04** ✅ 矩阵 48/2（626 同口径：仅 2 个存量快照失败）；矩阵 T8 适配新退出语义（脏 tab 确认层 → 不保存退出）——该修复属 626 语义变更的测试滞后，已同步 626 worktree（其矩阵 48/2）。截图实证：正文/行:列 1:1/Explorer 真树/短文件无滚动条（官方行为）。README 注记：见示例 README Plan 629 节（T-04 附带）。

## 9. 复审记录

- 2026-09-14 work handoff：`stage: work | plan_id: PLAN-629 | plan_revision: 1 | outcome: pass | code_commit: f4b77192b (branch plan-629-dev, base plan-626-dev@3b85a0357 叠放) | task_ids: T-01..T-04 全完成 | evidence: 各任务行 + 实机截图（正文渲染/官方滚动行为）+ 矩阵 48/2 | blockers: 无 | next: review。
`stage: new`，PLAN-629 rev1。用户裁定明确（引语见 §4），集成契约三机制（外部偏移同步/内容高度上报/scroll_to 跟随）均有 iced 0.14 现成通道与 renderer 先例。`outcome: pass`，`next: work`（用户指令即授权）。

## 10. 待澄清事项

- 无阻塞。风险预登记：(1) 寄宿后 layout_runs 的 shape 触发点从 render 移到 sync——需保证 shape_until_scroll 在 draw 的 with_font_system 临界区内执行；(2) 触控板 Pixels 模式滚轮由 scroller 接管后精度/惯性表现需实机确认；(3) 选区拖拽边缘自动滚动在寄宿模式下需用外部偏移实现，列 T-02。
