---
plan_id: PLAN-685
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-bp-callback-arg-forward
author: [zhaop/agent]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-vm, autoui]
current_step: 0
total_steps: 5
---

# [PLAN-685] VM 臂组件回调 msg 参数转发丢值根修 + bps-gallery 点击 E2E 补盖

## 0. 变更摘要

修复 VM 臂（iced inproc）BP L1 通道回调链的 msg 参数转发缺陷：组件内部
handler 把**形参**再发射进注入回调（`.Select(id) -> { on_select(id) }`）时，
落到消费方状态的是形参名字面量 `"id"` 而非绑定值。根修 VM 臂 handler 形参
→回调 msg 实参的绑定路径，并以 bps-gallery 点击路径的 E2E 门禁补盖
（PLAN-676 漏网点）。附带一个有界勘定任务：VM 臂首启竞态（详情偶发空壳）。

## 1. 目标

- **G1（根修）**：VM 臂上，BP 组件注入回调（`on_*: msg`）被组件内部
  handler 携参再发射时，消费方收到的实参必须等于发射点的形参绑定值
  （类型与值双保真）。
- **G2（回归门禁）**：bps-gallery 侧栏卡片点击在 VM 臂有自动化 E2E 覆盖
  （MCP `autoui_action` press → 断言状态与详情渲染），防 PLAN-676 型漏网
  复发。
- **G3（勘定）**：首启竞态（三次启动一次详情空壳）获得根因结论并登记
  （修复若小则同批，若大则记债立项，以勘定产物为准）。

**非目标**：vue 臂改动（预期不受累，T-01 中实测确认即可）；RQ 臂渲染面
（PLAN-683 承接）；bp 目录发现/静态表机制（PLAN-676 已收口）。

## 2. 架构方案

缺陷链（2026-09-22 bps-gallery 实测，MCP action 回执为证）：

```
卡片 onclick: .Select(item.id)          ← 第一跳实参正确（"navigation/sidebar-nav"）
  → BP 内部 .Select(id) -> { on_select(id) }   ← 丢值点：实参退化为形参名 "id"
    → app .SelectBp(id) -> { .selected_id = id } ← selected_id 终值 = "id"（回执实拍）
```

`selected_id="id"` 使 `resolve_key` 原样返回 `"id"`，无 bp 可匹配 → 详情区
全空（标题 "id"、first_variant "no references"、三卡空）。未点击时
`selected_id=""` 走首条回退，故应用初启看似正常，一点即坏——**所有卡片通用**。

**候选落点**（T-01 在此范围内钉根因，二选一或并存）：
- **H1 代码生成**：handler 体内 `on_select(id)` 的实参表达式 lowering——
  裸标识符在 msg 再发射位退化为名字字符串字面量（codegen/vm emit 面）。
- **H2 运行时绑定**：handler 形参未入 handler 调用作用域，`id` 查名失败
  后走"名字即值"兜底（`vm_bridge.rs` 的 handler 调用装配点）。

运行时装配点候选（实勘已定位）：`crates/auto-lang/src/ui/vm_bridge.rs`
`call_handler_for`（:1508）/ `call_handler_with_record`（:1612）/
`call_handler`（:1673）；iced 侧事件→handler 分发在
`crates/auto-lang/src/ui/iced/renderer.rs` 与 `ui/dynamic.rs`（组件子树
事件边界所在）。注意与 PLAN-669 的分界：669 收敛的是 **vm-server（HTTP
路径）** 按名绑定四装配点；本计划是 **iced inproc 路径**，两套装配面
是否共享底层是 T-01 勘定内容之一。

修复形态以 T-01 结论为准：H1 则改发射端（携带值语义），H2 则改绑定端
（形参入作用域）；禁止在消费方（bps-gallery）侧 workaround 绕过。

## 3. 技术栈

Rust（crates/auto-lang）；测试三层：① 最小 .at 复现语料（VM 臂单测，
仿 `test/vm` 语料 golden 形态）② 既有门禁档（`cargo t` / `cargo tv`，
按改动面命中）③ MCP E2E（复用 `.agents/skills/autoui-verifier/scripts/`
的 `test_vm_mcp.py` 形态，或新增专用脚本）。

## 4. 需求分析与背景调查

- **授权**：2026-09-22 用户在 bps-gallery 走查会话中裁定"立项修"（范围
  =本计划 §0；首启竞态以"附带候选勘定"授权，是否同批修复由勘定产物
  决定，大改须回禀）。
- **证据链**（现场实例，MCP :9350）：
  - `autoui_action` press 回执：`handler: .GalleryShell.Select`（实参
    `navigation/sidebar-nav` 正确）+ `state_changes: selected_id: "" -> "id"`。
  - 回调源：`blueprints/layout/gallery-shell/reference/default.at` :151
    `onclick: .Select(item.id)`、:192 `.Select(id) -> { on_select(id) }`。
  - 消费方：`examples/bps-gallery/src/front/app.at` :148 `.SelectBp(id)`。
  - 数据面无恙：`registry.at` 静态表内 source/gotchas 齐全，未点击路径
    （首条回退）渲染正常（截图在档）。
- **漏网原因**：PLAN-676 门禁只盖 vue 臂构建（vue-tsc + CI drift），
  VM 臂点击路径无 E2E（G2 补盖点）。
- **相关先例**：PLAN-669（vm-server 实参按名绑定，四装配点收敛——不同
  路径）；P614/P625 VM 轨纪律（List while+索引、Obj 全键书写——本缺陷
  是 msg 参数面，非 Obj 访问面）。
- **约束**：master 零 WIP 代码（全部实现在 worktree
  `D:/autostack/.wt/lang-685/auto-lang`）；改动命中 VM/编译器时终验跑
  `cargo tv`，不触碰 aavm 触发面（AGENTS.md §AAVM Tier）；多会话并行
  ——共享 `target/debug/auto.exe` 会被他方构建的文件锁清理误杀，E2E
  需注意实例存活性（本会话实测三杀）。

## 5. 详细设计

### T-01 根因勘定（有界，决策产物）

最小复现语料：三 widget 链（owner → 组件注入 `on_pick: msg` → 组件内
`.Inner(v) -> { on_pick(v) }` 转发一个 str 形参），断言 owner 收到值。
红线复现后按 §2 H1/H2 分叉钉根因，产物 = ①失败单测 ②根因注记
（文件：行 + 机理一句话）。**边界**：时间盒半日，若 H1/H2 之外
（如两层叠加），产物改为根因地图 + 修复拆分建议，回禀后改契约。

### T-02 根修 + 单测

按 T-01 结论修发射端或绑定端；最小语料转绿 + 邻近面快照零漂移
（handler 携参调用族：onclick 字面量参、oninput 事件参、for 内闭包捕
获参三形态各一断言——防修 A 坏 B）。

### T-03 bps-gallery 点击 E2E 门禁

脚本化：起 VM 实例（`AUTOUI_MCP_PORT` 钉独立端口）→ `autoui_action`
press 侧栏卡片 → 断言 `state_changes.selected_id == 所点 id` 且快照含
该 bp 的 spec 正文 → 顺序点满 ≥3 个含多类 kind 的卡片。落
`.agents/skills/autoui-verifier/scripts/`（如 `test_bp_gallery_click.py`），
README 登记用法；不进 CI 的理由/进法在复审时定。

### T-04 首启竞态勘定（有界）

现象：VM 臂三次启动一次详情空壳（"id"/"no references"/卡全空，截图在
档）——注意与 G1 缺陷同象但**未点击即现**，疑首帧 view 先于数据面就绪
且无重渲染触发。产物 = 根因结论 + 修复尺寸评估；小修同批（进 AC），
大改记债 `KNOWN-DEBT-AND-RISKS.md` 并回禀。

### T-05 收尾

按改动面跑门禁档；`auto bp list --format at` 漂移检查（蓝图表未被本计
划触碰的负验证）；复审材料整理。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-vm/project.md | 新增：VM 臂组件注入回调（`on_*: msg`）携参再发射时，实参 = 发射点形参绑定值（名字面量退化 forbidden） | 本缺陷即违例实证 | AC-01/AC-02 |
| SD-02 | add | docs/specs/autoui-skill/project.md | 新增：BP 消费型应用（bps-gallery 型）VM 臂点击路径属 E2E 应验面，双臂验收不得只盖 vue 构建 | PLAN-676 漏网教训 | AC-03 |

## 6. 测试设计

- **单测**（T-02）：最小转发语料 + 三形态邻近断言，落 `test/vm` 语料
  族或 `crates/auto-lang/src/tests/` 既有 vm 测试组。
- **E2E**（T-03）：MCP 脚本断言状态与快照（见 §5）。
- **门禁档**：`cargo check -p auto-lang` + `cargo t`（日常）；触及
  VM/编译器终了跑 `cargo tv`；不触发 `taa`（零 aavm 面触碰）。
- **手动走查**：bps-gallery 双臂（`auto run` / `auto run -r vm`）点击
  对拍，vue 臂确认不受累（T-01 附产）。

## 7. 验收标准

- **AC-01**：最小复现语料测试在修复前红、修复后绿；测试名与根因注记
  在 T-01 产物中可追溯。验证：`cargo t <新测试名>`。
- **AC-02**：bps-gallery VM 臂点击侧栏任一卡片，`selected_id` 终值 =
  所点条目 id（MCP `state_changes` 为证），详情区渲染该 bp 的 spec
  正文/变体 tab/源码。验证：T-03 脚本 exit 0。
- **AC-03**：T-03 E2E 脚本可重复执行（干净起实例 → 断言 → 收尾），
  脚本与用法登记在 `.agents/skills/autoui-verifier/`。
- **AC-04**：门禁档全绿：`cargo t` 绿 + `cargo tv` 绿（预存红基线
  按 AGENTS.md 口径豁免）；`git diff` 零 bps-gallery 消费方 workaround
  （app.at/catalog.at 不因本缺陷改动）。
- **AC-05**：首启竞态有根因结论并登记（修复或债条二选一，留痕）。

## 8. 执行步骤

- **T-01 根因勘定**（AC-01）
  文件：新增最小语料 + 候选落点（§2）。操作：复现→分叉→钉根因。
  验证：失败单测存在且红；根因注记写入本文件 §9 前的工作注记。
- **T-02 根修**（AC-01/AC-04）依赖 T-01。
  文件：以 T-01 结论为准（候选：`vm_bridge.rs` / codegen emit 面）。
  验证：新测试绿 + 邻近三形态断言绿 + `cargo t` 零新增红。
- **T-03 E2E 门禁**（AC-02/AC-03）依赖 T-02。
  文件：`.agents/skills/autoui-verifier/scripts/test_bp_gallery_click.py`（新增）。
  验证：脚本 exit 0 × 2 连跑。
- **T-04 首启竞态勘定**（AC-05）依赖 T-01（共享复现基建）。
  验证：结论登记（修复进 AC-04 门禁，或债条编号可查）。
- **T-05 收尾**（AC-04）依赖 T-02/T-03。
  操作：`cargo tv` + bp 表漂移负验证 + 复审材料。验证：档位全绿。

## 9. 复审记录

- 2026-09-22 `stage: new` `outcome: pass`（drafting 交付）：证据链现场
  实拍闭环（action 回执 + 回调源行号 + 数据面无恙截图），候选落点实勘
  到函数签名级；T-01 修复形态分叉（H1/H2）为显式有界决策点。授权范围
  见 §4。`next: work`（worktree：`D:/autostack/.wt/lang-685/auto-lang`，
  Plan 529 布局）。

## 10. 待澄清事项

- **Q-1**：首启竞态若勘定为需较大机制改动（首帧就绪门/重渲染触发器），
  同批修还是记债另立？——默认：回禀用户裁定（T-04 产物触发）。
- **Q-2**：T-03 E2E 是否进 CI——涉及 GUI 实例起停的 runner 环境，
  复审时按 CI 现状定。
