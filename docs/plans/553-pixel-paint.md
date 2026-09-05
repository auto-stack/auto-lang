---
plan_id: PLAN-553
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: pixel-paint
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-man]   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 8
---

# [PLAN-553] 031-paint——像素画板（AutoOS 首个创作类应用）

## 变更摘要

填 `examples/ui` 编号空洞 **031**，新建 AutoOS 的"画图"应用：**像素画板**
形态（16×16 格染色 + 调色板 + 铅笔/橡皮/油漆桶/吸管 + undo/redo + storage
存取）。真画布原语（连续笔迹/pointer 路径）不在本计划——schema `canvas`
元素目前是占位（props TBD、web 后端 none），像素形态今天就能在现有能力上
双端落地；真画布另立设计文档（见待澄清①）。

依赖：PLAN-552 的 `desktop:` 字段（未合入时 pac 写 `desktop: "true"` 为
无害未知键，boot 忽略——软依赖，可并行）。

## 目标

1. `examples/ui/031-paint/`：单组件 App（`src/front/app.at`），vue/vm 双端
   可用（`auto run` / `auto run -r vm`）。
2. 工具集 v1：铅笔（点击染当前色）、橡皮（染回白）、油漆桶（连通同色区
   泛洪）、吸管（取格色为当前色）。
3. 调色板 16 色 + 当前色显示；undo/redo（快照栈上限 20）；清空/新建。
4. 作品持久化：storage 存取（`paint.canvas.v1`），重开应用可恢复。
5. 桌面上架 + 画廊收录：pac `desktop: "true"`（552 合入后生效）；
   画廊分类归 "03-apps"。

## 架构方案

单组件约束（025/028 形态先例）：

- **状态**：`px` = 256 长度颜色串列表（`"#ffffff"` × 256）——规避 B12
  （VM handler 对 Obj 数组字段读失效；字符串列表下标读写保真，028 探针钉死）。
  视图行对象 `{i, c, chip}` 由 handler 自建（view 侧读 handler 自建 Obj 数组
  已证可用，028 `ranked` 先例）。
- **渲染**：16×16 `grid { cols: 16 }`（**静态 cols**——动态绑定有 P537-D2
  登记债，绕开）+ 每格 `col { style: "bg-[<色>]" }` + `onclick: .Paint(i)`。
  256 格渲染量与 038-minesweeper 同级，VM 端可承受。
- **undo 栈**：快照 = `px.join("|")` 单字符串；栈 = 字符串列表（VM 列表
  push/pop 保真），上限 20 溢出丢底。
- **storage**：整串写 `paint.canvas.v1`（~1.8KB；值长度上限 T1 探针，超限
  降 12×12 或改 RLE）。

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

- **GOAL-010**（示例应用轨道·AutoOS 默认应用集）：Paint 是 2026-09-05
  桌面应用盘点确认的第一梯队缺口（"没有 MSPaint 替代品"）。
- **能力现状**：
  - `schema/aura.at` `canvas` 元素 = P1 占位（props TBD，web/iced 均 none）
    → 本计划用 grid 染色绕开；
  - B12 家族约束：028-launcher 头注（平行字符串列表 + handler 自建行对象）；
  - grid 渲染先例：038-minesweeper（VM 实测）；语义色 chip 插值先例：
    028 `bg-[<color>21]`；
  - storage 定长槽惯例：028 `launcher.recent_apps.0..4`（string|null 产物
    只做 `!= ""` 比较，vue TS18047 教训）。
- **编号**：README 空洞优先规则——031 为 024–040 空洞首个空号（现员至 030）。

## 详细设计

### model（要点）

```
var size int = 16
var px = [...]                  // 256 × "#ffffff"（Init 填充）
var rows = []                   // view 行对象 {r, cells}，handler 重建
var tool str = "pencil"         // pencil | eraser | fill | picker
var cur str = "#1f2937"         // 当前色
var palette = [...]             // 16 色（黑白灰 + 12 基本色）
var undo_stack = []             // 快照串列表，cap 20
var redo_stack = []
var saved str = "0"             // 有未保存改动标记（title 提示用）
```

### handler 家族

- `.Paint(i)`：按 tool 分派——pencil→`px[i]=cur`；eraser→白；fill→
  `.Fill(i)`；picker→`cur = px[i]`。铅笔/橡皮先入 undo 快照再改。
- `.Fill(i)`：迭代 BFS 泛洪（显式队列列表 + visited 平铺标记，**禁递归**），
  同 `px[i]` 连通区全染 `cur`。
- `.SetTool(t)` / `.SetCur(c)`：工具与当前色切换（选中态 chip 高亮）。
- `.Undo()` / `.Redo()`：栈互倒 + `px` 反序列化（`split("|")`）。
- `.Clear()`：确认后全白（入 undo）；`.Init`：读 storage 恢复或全白。

### view（三栏）

左工具列（4 按钮，选中 `bg-primary/15`）｜中画布 grid 16×16（格 14px 级）｜
右调色板 grid 4×4 + 当前色块。底部：Undo/Redo/Clear/Save/Load 按钮行。

### pac.at

```
name: "pixel-paint"
title: "Paint"
icon: "brush"
category: "tool"
render: "vue"
desktop: "true"
window: "fit"
```

（icon 走 lucide `brush`——T1 对 `lucide_icon_coverage` 闭集核验，不在集则
回退闭集内近似图标。）

## 测试设计

`tests/desktop_mcp.py`（011/013 惯例，vue + vm 双轨）：

1. 选色→点格→断言该格底色类含所选色值；
2. 油漆桶：同色连通区多格一次变色，异色格不变；
3. 吸管→当前色块更新；
4. undo→回退一格改动；redo→重做；
5. Save→重开（重新 build 组件）→Load 断言恢复。

## 验收标准

1. `auto run` 与 `auto run -r vm` 双端可画、四工具全可用、undo/redo 正确。
2. `desktop_mcp.py` 全绿（双端）。
3. boot 桌面出现 Paint 图标（552 合入后；之前注册表扫描可见 031 条目）。
4. ui-gallery 重新生成后收录 031 且分类 "03-apps"。
5. `examples/ui/README.md` 总览表补 031 行 + 编号空洞历史注记。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 能力探针**
  a) storage 值长度上限（256×"|#rrggbb" ≈1.8KB 写读回环）；b) lucide
  `brush` 是否在 VM 闭集（`lucide_icon_coverage` 测试面）；c) 格级
  `onmousemove` 拖画是否双端可用（可用则 T4 加拖画，不可用 v1 纯点击）。
  产物：探针结论写回本节。
  验证：探针脚本输出归档（scratch/p553/）
- [ ] **T2 目录与 pac 骨架**
  `examples/ui/031-paint/pac.at` + 空 `src/front/app.at`（可编译的最小
  widget）+ `SPEC.md` 骨架。
  验证：`cd examples/ui/031-paint && auto build`（0 错误）
- [ ] **T3 model + view 主体**
  `src/front/app.at`：model 全量状态 + 三栏 view（工具列/画布 grid/调色板）
  + 底部按钮行；Init 全白。
  验证：`auto run` 手画冒烟（截图 scratch/p553/）
- [ ] **T4 handler 家族**
  `.Paint/.Fill/.SetTool/.SetCur/.Undo/.Redo/.Clear`（BFS 显式队列；快照
  join/split；T1c 结论决定是否含拖画）。
  验证：`auto run` 四工具 + undo/redo 手测
- [ ] **T5 storage 存取**
  `.Save/.Load` + Init 恢复（`paint.canvas.v1`；T1a 结论定编码形态）。
  验证：`auto run` 存→重开→恢复
- [ ] **T6 desktop_mcp 测试**
  `tests/desktop_mcp.py` 五断言组（测试设计节）。
  验证：`python tests/desktop_mcp.py`（vue 轨）+ vm 轨同套
- [ ] **T7 vm 端对拍**
  `auto run -r vm` 全流程手测 + mcp vm 轨绿；差异登记 SPEC.md「双端注记」。
  验证：vm 轨 mcp 全绿
- [ ] **T8 画廊分类与文档回写**
  `crates/auto-man/src/vue.rs` 分类 if 链：`031` → "03-apps"；`examples/
  ui/README.md` 总览表补 031 行 + 空洞注记。
  验证：`cargo check -p auto-lang && cargo check -p auto-man`

## 复审记录

## 待澄清事项

1. 真画布原语（连续笔迹/pointer 事件路径/freehand canvas）建议另立
   design 文档（docs/design/autoui/ 下），可带出白板/签名板/截图标注——
   本计划不阻塞。
2. 拖画（onmousemove 连续染格）双端可用性未知——T1c 探针定 v1 范围。
3. 导出 PNG：storage 串 → 图片文件需要宿主 FFI 面，v1 不做（Save/Load
   仅 storage 内）。
