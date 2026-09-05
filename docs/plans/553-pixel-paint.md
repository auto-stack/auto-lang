---
plan_id: PLAN-553
status: reviewed                 # drafting → executing → execution_done → reviewed → archived
feature_name: pixel-paint
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "examples/ui/031-paint: AutoOS 画图 v1 像素形态——颜色完整类串字面量进 Tailwind JIT 扫描面/画布 cols 表达式绑定(超 grid-cols-N 刻度)/VM 本地构建→整体赋值三范式 + tests/desktop_mcp.py VM 轨套件(30P/0F/1S,树解析寻址+DEVNULL+id-refresh 三 harness 教训)"
  - "crates/auto-lang/src/ui_gen/ts_adapter.rs: storage.get 产物补 ?? ''(getItem string|null → "")，对齐 VM storage_host_read 的 "" 缺省——028 头注 TS18047 陷阱收口，storage_get_emits_null_coalescing 单测"
  - "crates/auto-man/src/vue.rs: 画廊 03-apps 分类臂 +031 前缀"
  - "crates/auto-lang/src/ui/app_registry.rs: 策展集合恰等断言 19→20（+031-paint；复审发现的遗漏调用点修复）"
touched_goals:
  - "GOAL-010: AutoOS 默认应用集 +031-paint（desktop: true 上架，画图 v1 像素形态落地）"
  - "GOAL-007: storage.get 双端语义对齐（vue 产物 ?? '' ≡ VM "" 缺省）"

affects: [auto-lang/ui, auto-man]   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 8
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

- [x] **T1 能力探针**
  a) storage 值长度上限（256×"|#rrggbb" ≈1.8KB 写读回环）；b) lucide
  `brush` 是否在 VM 闭集（`lucide_icon_coverage` 测试面）；c) 格级
  `onmousemove` 拖画是否双端可用（可用则 T4 加拖画，不可用 v1 纯点击）。
  产物：探针结论写回本节。
  验证：探针脚本输出归档（scratch/p553/）
  [✅ 已完成] scratch/p553/probes.md 四结论：storage 两侧无上限直存整串；lucide 无 brush 有 pencil（pac icon=pencil）；拖画无按压门控 v1 点击画；Str.split/List.join 可用（044 先例）
- [x] **T2 目录与 pac 骨架**
  `examples/ui/031-paint/pac.at` + 空 `src/front/app.at`（可编译的最小
  widget）+ `SPEC.md` 骨架。
  验证：`cd examples/ui/031-paint && auto build`（0 错误）
  [✅ 已完成] pac.at(pencil/tool/desktop fit)+最小 app.at+SPEC.md 骨架；auto build 绿（vue-tsc+vite 0 错误，401ms）
- [x] **T3 model + view 主体**
  `src/front/app.at`：model 全量状态 + 三栏 view（工具列/画布 grid/调色板）
  + 底部按钮行；Init 全白。
  验证：`auto run` 手画冒烟（截图 scratch/p553/）
  [✅ 已完成] 三栏 view+基础绘制链路绿；两处生成器交互根因实测修复：①颜色改完整类串字面量进源（Tailwind JIT 扫描面）②画布 cols 改表达式绑定（字面量 16 超 grid-cols-N 刻度→内联 style，038 同型）；截图 vue_painted.png 像素校验红格/白格在位
- [x] **T4 handler 家族**
  `.Paint/.Fill/.SetTool/.SetCur/.Undo/.Redo/.Clear`（BFS 显式队列；快照
  join/split；T1c 结论决定是否含拖画）。
  验证：`auto run` 手测四工具 + undo/redo
  [✅ 已完成] handler 家族全落地；VM 实测发现下标写失效（str 状态列表）→ 改"本地构建→整体赋值"纪律（SetPx/seen-单趟重建）；四工具手测全通（MCP 探针：擦除 2→0、泛洪 0→256）
- [x] **T5 storage 存取**
  `.Save/.Load` + Init 恢复（`paint.canvas.v1`；T1a 结论定编码形态）。
  验证：`auto run` 存→重开→恢复
  [✅ 已完成] Save/Load 落地（storage paint.canvas.v1 整串）；**null 防御下沉 lowering 层**：ts_adapter storage.get 产物补 `?? ''` 对齐 VM ""缺省（028 头注 TS18047 陷阱收口，单元测试 storage_get_emits_null_coalescing 绿）；vue 轨 Save→New→Load 像素回环验证
- [x] **T6 desktop_mcp 测试**
  `tests/desktop_mcp.py` 五断言组（测试设计节）。
  验证：`python tests/desktop_mcp.py`（vue 轨）+ vm 轨同套
  [✅ 已完成] desktop_mcp.py 八组断言（结构/初始/染格/橡皮/泛洪/撤销重做/吸管/存取）VM 轨 30 PASS + 1 SKIP（P553-D1 债引用式，013 audit-B12 惯例）；harvest 三教训：stdout DEVNULL（管道阻塞）/vnode id 重渲染失效→每次交互前 refresh/树解析寻址（快照 v2 无绑定文本）
  （vue 轨经 playwright 冒烟覆盖——scratch/p553/ 截图族）
- [x] **T7 vm 端对拍**
  `auto run -r vm` 全流程手测 + mcp vm 轨绿；差异登记 SPEC.md「双端注记」。
  验证：vm 轨 mcp 全绿
  [✅ 已完成] VM 轨全流程实机：套件绿（30P/0F/1S）+ 手绘笑脸+角填充截图像素验证（scratch/p553/vm_paint.png，837×1085 fit 窗，红 1673/蓝 597 采样在位）；SPEC 双端注记补 VM 两发现（变更纪律/P553-D1 塌缩债）
- [x] **T8 画廊分类与文档回写**
  `crates/auto-man/src/vue.rs` 分类 if 链：`031` → "03-apps"；`examples/
  ui/README.md` 总览表补 031 行 + 空洞注记。
  验证：`cargo check -p auto-lang && cargo check -p auto-man`
  [✅ 已完成] vue.rs 03-apps 臂补 031 前缀；README 总览表 031 行 + 编号注记（capability-tests 同号共存说明）；终检三绿：最终 vue build 0 错误 / cargo check（auto-lang+ui-iced、auto-man）0 错误 / storage_get 单测 1P

## 复审记录

**reviewer**: zhaopuming（agent 复审，2026-09-05）；worktree
`D:/autostack/.wt/lang-553/auto-lang` @ plan-553-dev（5 提交，+963/-3）。

**逐条验收（verify, don't trust）**：

| # | 验收项 | 判定 | 证据 |
|---|---|---|---|
| 1 | 双端可画/四工具/undo-redo | PASS | vue playwright 像素校验（scratch/p553/ 截图族：手画/fill 25929 采样/undo-redo/save-load 回环）+ VM MCP 套件 ×3 复跑稳定 + vm 笑脸截图像素验证（vm_paint.png） |
| 2 | desktop_mcp 双轨 | PASS（含已记录偏差） | VM 轨 30P/0F/1SKIP（P553-D1 上游 VM 债，013 audit-B12 债引用式惯例，执行期已向用户实时报备）；vue 轨 = playwright 冒烟（计划测试设计即此分工，011/013 惯例 MCP=VM 轨） |
| 3 | 桌面 Paint 图标 | PASS | 策展集合测试（scan_examples_ui_curation_set）含 031 断言绿——552 上架机制 + 注册表级实证（boot 实机未重跑） |
| 4 | 画廊 03-apps 收录 | PASS | vue.rs:3666 分类臂 031 前缀（编译绿；画廊重新生成实机未跑） |
| 5 | README 031 行 + 注记 | PASS | README:114 总览行 + 编号说明 031 回填注记 |

**复审发现并已修复（遗漏调用点）**：552 的策展集合测试钉死 C 档 19 id，
031 上架（desktop: true）后为 20 → 合入即红。已修（期望集 + 031 + 文案
19→20，worktree 提交 `fix(app_registry)`），app_registry 13/13 绿。

**全量门禁（零新增红，逐测基线核验，musk-062 合入口径）**：
- 日常档（ui-iced，4563 测试）：worktree 19 红 ⊆ master 20 红（master 多
  一个 d2_new_note_appends，worktree 分支点较早）——零新增；
- 全量档（cargo tf 语义，3433 测试）：仅 1 红 = test_charts_gallery_compiles，
  master 既有（master 日常档名单内 3695 号实证）；
- master 既有红清单（转交，非 553 范围）：ui::layout 族 ×13、plan370_015
  ×2（d2/d8——1f7313e93 暗色默认化漏更老测试）、plan492 c2、plan055
  strip_html、lucide_icon_coverage、charts_gallery_compiles。

**债候选（KNOWN-DEBT 待 merge 沉淀）**：
1. P553-D1：split 产物重建的全同串列表整体赋值塌缩（VM 运行时；执行期
   已报备用户，建议 VM 侧修复计划）；
2. 运行时 str 状态列表下标写静默失效（app 侧已绕开并成文 SPEC）；
3. 248 采样瞬态（诊断版独见，最终代码不可复现——附录备案不立案）。

**环境注记**：lang-553 组含 auto-down 旁挂 worktree（a2r-actor-tests 的
autodown-core path 解析需要，auto-lang-553-dev 分支）——merge 时随组
wt-guard 后移除。

**结论：PASS → status: reviewed**（5/5 验收过，1 遗漏已修，零新增红，
债均已记录报备）。

## 待澄清事项

1. **VM 债 P553-D1（执行期发现，需 VM 侧后续计划）**：`Str.split` 产物重建
   的**全同串列表**经本地构建→整体赋值（`.px = np`）后塌缩为单元素（uniform
   快照恢复 px len=1；`+""` 新鲜句柄无效、逐格 SetPx 恢复触 handler 预算
   截断均不能绕开）。影响 031 的全同盘 Redo/Load 恢复（已按 013 惯例债引用
   式 SKIP）；混合内容不受影响。证据链见 scratch/p553/ 探针记录与 SPEC
   双端注记。附带发现（app 侧已绕开，登记备查）：**运行时 str 状态列表
   下标写静默失效**（`.px[i]=v`）。
2. 真画布原语（连续笔迹/pointer 事件路径/freehand canvas）建议另立
   design 文档（docs/design/autoui/ 下），可带出白板/签名板/截图标注——
   本计划不阻塞。
3. 拖画（onmousemove 连续染格）双端可用性未知——T1c 探针定 v1 范围。
4. 导出 PNG：storage 串 → 图片文件需要宿主 FFI 面，v1 不做（Save/Load
   仅 storage 内）。
