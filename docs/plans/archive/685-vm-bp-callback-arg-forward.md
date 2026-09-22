---
plan_id: PLAN-685
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: vm-bp-callback-arg-forward + bp 内容页 UX 重设计
author: [zhaop/agent]
created_at: 2026-09-22
updated_at: 2026-09-22
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/blueprint/project.md#消费面与组装样板]   # SD-01 修订该节 P657-D1 P0 条目（载荷字面量化收口注记）
new_spec_components: [docs/specs/blueprint/project.md#消费面与组装样板]         # SD-02/03 新增验收/展示规则句
touched_goals: [GOAL-011, GOAL-007]  # GOAL-011 行在册 P657-D1=本修收口；GOAL-007 双臂一致面（computed-as-arg vue 臂白屏修复+markdown 双臂消费）

affects: [auto-vm, autoui]
current_step: 7
total_steps: 7
---

# [PLAN-685] VM 臂回调参数转发根修 + bps 内容页 UX 重设计（autodown 渲染 + 快速导航）

## 0. 变更摘要

两阶段一计划（2026-09-22 用户两次裁定叠加）：

1. **根修**（revision 1 原范围）：VM 臂（iced inproc）BP L1 通道回调链
   的 msg 参数转发丢值——组件内部 handler 把**形参**再发射进注入回调
   （`.Select(id) -> { on_select(id) }`）时，消费方收到形参名字面量
   `"id"` 而非绑定值；加 bps-gallery 点击路径 E2E 门禁补盖 PLAN-676
   漏网点。
2. **内容页 UX 重设计**（revision 2 新增，用户裁定"内容也很粗糙…重新
   设计内容页的 UI/UX，让 gallery 更专业化"）：Spec/Gotchas 从裸 `pre`
   文本升级为 **autodown 渲染件**双臂呈现（对齐 jade-edit 的
   `@autodown/engine` 消费），新增**分节快速导航**（Spec/Reference/
   Gotchas tab 直切，消灭长滚动），内容页头部专业化（面包屑/kind 徽标
   /变体数/前后条目导航）。

## 1. 目标

- **G1（根修）**：VM 臂上，BP 组件注入回调（`on_*: msg`）被组件内部
  handler 携参再发射时，消费方收到的实参必须等于发射点的形参绑定值
  （类型与值双保真）。
- **G2（回归门禁）**：bps-gallery 侧栏卡片点击在 VM 臂有自动化 E2E 覆盖
  （MCP `autoui_action` press → 断言状态与详情渲染），防 PLAN-676 型漏网
  复发。
- **G3（勘定）**：首启竞态（三次启动一次详情空壳）获得根因结论并登记
  （修复若小则同批，若大则记债立项，以勘定产物为准）。
- **G4（内容渲染专业化）**：Spec 正文与 Gotchas 以 markdown 渲染件呈现
  （标题层级/列表/表格/引用/代码块 chrome），双臂同源同能力，对齐
  jade-edit 的 autodown 消费形态；Reference 源码至少 mono + 代码块
  chrome（语法高亮视引擎能力，见 Q-3）。
- **G5（快速导航）**：内容页三节（Spec/Reference/Gotchas）tab 直切，
  点击即达，不依赖长滚动；VM 臂实现不依赖 `scroll_to`（既有不可靠，
  见 [[vm-iced-scroll-pitfalls]] 记录）。

**非目标**：vue 臂引擎改造（`@autodown/engine` 本体不动，只消费）；
RQ 臂渲染面（PLAN-683 承接）；bp 目录发现/静态表机制（PLAN-676 已收
口）；bp 实况预渲染（live render via a2vue，独立 follow-up 不并入）。

## 2. 架构方案

### 2.1 缺陷链（revision 1，实测证据）

```
卡片 onclick: .Select(item.id)          ← 第一跳实参正确（"navigation/sidebar-nav"）
  → BP 内部 .Select(id) -> { on_select(id) }   ← 丢值点：实参退化为形参名 "id"
    → app .SelectBp(id) -> { .selected_id = id } ← selected_id 终值 = "id"（回执实拍）
```

`selected_id="id"` 使 `resolve_key` 原样返回 `"id"`，无 bp 可匹配 → 详情区
全空。未点击时 `selected_id=""` 走首条回退，故应用初启看似正常，一点即
坏——**所有卡片通用**。

**候选落点**（T-01 钉根因，二选一或并存）：
- **H1 代码生成**：handler 体内 `on_select(id)` 的实参表达式 lowering——
  裸标识符在 msg 再发射位退化为名字字符串字面量（codegen/vm emit 面）。
- **H2 运行时绑定**：handler 形参未入 handler 调用作用域，`id` 查名失败
  后走"名字即值"兜底（`vm_bridge.rs` 装配点）。

运行时装配点候选（实勘已定位）：`crates/auto-lang/src/ui/vm_bridge.rs`
`call_handler_for`（:1508）/ `call_handler_with_record`（:1612）/
`call_handler`（:1673）；iced 侧事件→handler 分发在
`crates/auto-lang/src/ui/iced/renderer.rs` 与 `ui/dynamic.rs`。与
PLAN-669 分界：669 收敛的是 **vm-server（HTTP 路径）**；本计划是
**iced inproc 路径**，两套装配面是否共享底层是 T-01 勘定内容之一。

### 2.2 内容页重设计（revision 2 新增）

**渲染通道勘定结论（2026-09-22 实勘，双臂齐备）**：
- VM 臂：`markdown` / `autodown` / `autodown_editor` widget →
  `crates/auto-lang/src/ui/autodown_render.rs`（autodown-core
  `parse_blocks` → 块树 → View；heading 样式表/quote 边条/codeblock
  头部+mono/table 列宽拖拽/callout/details 全备，plan-450 批次三样式
  同源）。
- Vue 臂：`ui_gen/vue.rs` :12857 将 `autodown|markdown|markdown_editor`
  tag 映射到 `@autodown/engine` StreamingRenderer（content → :source）；
  消费先例 = jade-edit `pac.at` dep link +
  `autodown_editor (key:, final: true) { content: ... }`（app.at :220）。

**设计**（app.at 内容区改造，catalog.at 配套）：
1. **分节 tab**：内容页改为 `active_section`（"spec"|"reference"|
   "gotchas"，缺省 "spec"）状态切换——一次只渲染一节，点击即达。
   纯 view 状态切换，双臂零新依赖，绕开 scroll_to 不可靠面（G5）。
   tab 形态沿用 kind pills 的既有样式语汇（选中 bg-primary 圆角档）。
2. **Spec/Gotchas 换渲染件**：`pre` → `markdown { content: .cur_spec }`
   （read-only，`final: true`；不用 autodown_editor——无编辑回流需求）。
   gotchas 文本本就是 markdown（`# Gotchas — …` 结构）。
3. **Reference 代码面**：保留专用代码容器（mono + bg-muted + 圆角
   chrome；源码含反引号风险，不做 markdown 围栏包裹）；语法高亮为
   stretch（vue 臂引擎若支持 lang 标签则开，VM 臂 autodown codeblock
   无高亮——边界登记，不阻塞）。
4. **头部专业化**：面包屑（kind › name，kind 段可点击=等价 SelectKind）、
   kind 徽标、变体计数、`active_section` tab 条同排；变体 tab 收进
   Reference 节内（现设计已如此，保持）。
5. **前后条目导航**：catalog.at 新增 `prev_next(bps, key) -> {prev, next}`
   （按 gallery_items 序），头部右侧 ‹ › 按钮 = SelectBp(id)，循环或不
   循环以实现顺手的为准（登记选择）。
6. **响应式余量**：内容列维持 `max-w-5xl mx-auto`；tab 条 sticky 于
   内容列顶（vue sticky；VM 臂 sticky 无对应则常驻顶部，边界登记）。

**装配纪律沿用**（app.at 头注）：computed 单 fn 直呼；view 内不裸用
`.len()`/`.to_str()`——收进 catalog.at fn 体内；VM 轨 List 遍历
while+索引。

## 3. 技术栈

Rust（crates/auto-lang，仅 T-01/T-02 触及）；应用层 `.at`
（examples/bps-gallery：app.at/catalog.at/pac.at）；`@autodown/engine`
（vue 臂 dep link，jade-edit 先例）；测试三层：① 最小 .at 复现语料
② 既有门禁档（`cargo t`/`cargo tv`，按改动面）③ MCP E2E
（`.agents/skills/autoui-verifier/scripts/` 形态）。

## 4. 需求分析与背景调查

- **授权**：2026-09-22 用户两段裁定——①"立项修"（回调丢值 + E2E 补盖
  + 竞态勘定候选）；②"内容也很粗糙…markdown 没有用 autodown-engine
  显示（参看 ../jade-edit），没有高亮和格式化，三挡需要滚动很远…缺乏
  快速导航，重新设计内容页 UI/UX…记录到计划 685 之后一起实施"
  （revision 2，随本计划一次实施）。
- **证据链**（revision 1，现场实例 MCP :9350）：
  - `autoui_action` press 回执：`handler: .GalleryShell.Select`（实参
    `navigation/sidebar-nav` 正确）+ `state_changes: selected_id: "" -> "id"`。
  - 回调源：`blueprints/layout/gallery-shell/reference/default.at` :151
    `onclick: .Select(item.id)`、:192 `.Select(id) -> { on_select(id) }`。
  - 消费方：`examples/bps-gallery/src/front/app.at` :148 `.SelectBp(id)`。
  - 数据面无恙：`registry.at` 静态表内 source/gotchas 齐全，未点击路径
    渲染正常（截图在档）。
- **内容页现状证据**（用户截图 + 本会话截图）：spec 正文含 `# Intent`
  `## with_charts` 等标题、`- **Difference**:` 列表、`` `dataSource` ``
  行内码，全部以等宽纯文本渲染，无层级无强调；Gotchas（结构化
  markdown）同为裸文本；三节总高数千 px，仅靠整页滚动。
- **漏网原因**：PLAN-676 门禁只盖 vue 臂构建，VM 臂点击路径无 E2E。
- **相关先例**：PLAN-669（vm-server 按名绑定——不同路径）；jade-edit
  PLAN-081（autodown_editor 消费形态）；plan-450 批次三（VM 面板样式）；
  PLAN-019 批次七（markdown widget 真渲染收口）。
- **约束/坑**：master 零 WIP 代码（实现在 `D:/autostack/.wt/lang-685/
  auto-lang`）；worktree 内禁 junction——vue 臂 gen 的 pnpm link 会在
  gen/ 产 junction，fold 前按既有坑（661/671）`cmd rmdir /s /q` 清理
  过 wt-guard；`@autodown/engine` 为仓外相对 link（auto-down），worktree
  内路径深度与主检出不同，gen 后须实测 link 解析；改动命中 VM/编译器
  终验 `cargo tv`，零 aavm 面触碰（不跑 taa）；多会话并行下共享
  `target/debug/auto.exe` 有文件锁误杀先例（本会话三杀），E2E 前确认
  实例存活。

## 5. 详细设计

### T-01 回调丢值根因勘定（有界，决策产物）

最小复现语料：三 widget 链（owner → 组件注入 `on_pick: msg` → 组件内
`.Inner(v) -> { on_pick(v) }` 转发 str 形参），断言 owner 收到值。
红线复现后按 §2.1 H1/H2 分叉钉根因，产物 = ①失败单测 ②根因注记
（文件：行 + 机理一句话）。**边界**：时间盒半日；若 H1/H2 之外
（如两层叠加），产物改为根因地图 + 修复拆分建议，回禀后改契约。

**[✅] 根因钉定（2026-09-22，worktree 80bfec4b6 前的红测在案）**：
H2 变体，确切落点不在 vm_bridge 装配点（其无恙），而在**剥离回调
快照链**：`handler_codegen.rs` `stripped_arg_text`（:1883）把体内
`on_select(id)` 实参记为**文本** `"id"`（Expr 非 Send，设计使然）→
`dynamic.rs` `eval_stripped_arg`（:2480）派发侧求值域**只有 root
state**，handler 形参不可见 → :2510 兜底 `Value::Str(t)` **名字即值**。
红测 `plan685_inbody_callback_param_forwarded_not_name` 修复前
`Str("v")`（退化实锤）、字面量形态既有绿。

### T-02 回调丢值根修 + 单测

按 T-01 结论修发射端或绑定端；最小语料转绿 + 邻近面三形态断言
（onclick 字面量参 / oninput 事件参 / for 内闭包捕获参），防修 A 坏 B。

**[✅] 绑定端根修（80bfec4b6）**：`vm_bridge` `handler_param_counts`
扩为 `handler_param_names`（形参名表；`handler_param_count` 契约
不变，arity=vec len）；`dynamic.rs` 派发侧形参名 × 派发实参配对成
绑定表，`eval_stripped_arg` 裸标识符**先查绑定**（词法最近）再退
state；`this.`/`.` 前缀显式指 state 跳过绑定；裸词字面量兜底保留。
邻近形态：字面量参 + oninput 事件参两测新增保绿；state 路径参由
既有 plan051 测覆盖（绿）；视图 for 内 msg 带参（第一跳）不经剥离
快照路径，由 T-03 E2E 实点覆盖。ui 模块门禁 2344 pass / 17 fail
全预存（14 layout=PLAN-686 + counter/shell_pack/dock_pager 三枚
base 0e541c7e5 原样复现）。

### T-03 bps-gallery 点击 E2E 门禁

脚本：起 VM 实例（`AUTOUI_MCP_PORT` 钉独立端口）→ `autoui_action`
press 侧栏卡片 → 断言 `state_changes.selected_id == 所点 id` 且快照含
该 bp spec 正文 → 覆盖 ≥3 卡片含多 kind。落
`.agents/skills/autoui-verifier/scripts/test_bp_gallery_click.py`，
README 登记用法；进 CI 与否复审时定（Q-2）。

**[✅] 落库 + 多轮连绿**：脚本按 `blueprints/` 目录解析卡片（改名
漂移即门禁红）、首帧竞态探针兼用（`selected_id` 非空即签名）、
3 卡跨 3 kind（dashboard/layout/navigation）、按 PID 只收杀自拉实例；
SKILL.md 步骤 1 登记用法。加固：首帧窗口 ~1/3 频率 no-op press
（回执 ok 零 state 变化，与仓内记忆 MCP 首 30s window-size-zero
同族）→ 有界重试 3 次，值错误仍立即红。终态 2 连跑 exit 0
（AC-02/AC-03）。

### T-04 首启竞态勘定（有界）

现象：VM 臂启动即现详情空壳（2026-09-22 实测 5 次启动 2 次，**零点击**
复现，第 5 次启动截图在档）——破坏态签名（标题 "id"/"no references"/
三卡空）与 G1 点击缺陷**终态完全一致**，但触发路径不同（无回调发生）。
**T-04 首项 = 检验同根假设**：VM 臂 Obj 字段访问（如
`gallery_items` 产物的 `rows[0].id`）在数据面未就绪/特定求值时点退化
返回字段名字面量——若证实则 T-02 根修一并覆盖，T-04 缩为验证任务；
若异根，按首帧 view 先于数据就绪且无重渲染触发的方向勘定。产物 =
根因结论 + 修复尺寸评估；小修同批（进 AC），大改记债
`KNOWN-DEBT-AND-RISKS.md` 并回禀（Q-1）。

**[✅] 结论：同根，已被 T-02 覆盖（带修构建实证 0/10 复发）**。
①"字段访问退化"假设**证伪**：VM GET_FIELD 的 ObjectData 缺键臂读
null（PLAN-053 P-053-6），无名字退化臂（engine.rs :6187 实勘）；
②签名反推：标题 "id" 要求 `selected_id=="id"`，唯一写入路径 =
Select 链剥离快照兜底（T-02 已封）；③实证：带修 auto.exe 共 10 次
全新启动（E2E --runs 与 5 连跑）竞态签名 0/10，修复前基线 2/5 在档。
残留不确定性登记：未在无修复环境捕获触发分派的准确源头（该环境已
不可复得）；若再现，E2E 首帧探针（`selected_id` 非空即红）会当场
逮住。判定为"修复同批"档，不立新债（AC-05 口径二选一取修复）。

### T-05 内容页重设计实施（G4/G5 主战场）

改 `examples/bps-gallery/src/front/app.at`（+catalog.at 配套 fn）：
1. `active_section` 状态 + 分节 tab 条（§2.2.1）；
2. Spec/Gotchas 换 `markdown` 渲染件（§2.2.2；vue 臂 pac.at 补
   `@autodown/engine` dep link——jade-edit pac.at :33 形态，相对路径按
   bps-gallery 深度修正，gen 后实测解析）；
3. 头部面包屑/kind 徽标/变体计数（§2.2.4）；
4. prev/next 导航（§2.2.5，catalog.at 新 fn，全键书写 Obj）。
验证：双臂实机走查（vue `auto run` + VM `auto run -r vm`），标题层级/
列表/表格/引用渲染在案；tab 点击即切；VM 臂不出现 scroll 依赖。

**[✅] 实施完成（8b8c94c0c + 2d2d9de05 纪律修复）**：`active_section`
状态分节（spec 缺省；Q-4 裁定=切 bp 保持分节，首启缺省承担入口语义）、
Spec/Gotchas 换 `markdown (content:, final: true)` 渲染件（vue 臂
pac.at npm_deps `@autodown/engine` **绝对 link**——jade-garden 先例，
worktree 深度免疫；Empty gotchas 占位文案）、头部三件（kind 面包屑
可点击=SelectKind 等价 + 变体计数徽标 + prev/next，`prev_next` 按
gallery_items 序到头空串降灰、SelectBp 空串守卫）、Reference 保留
mono 代码容器。VM 臂 MCP 交互全验证（tab 直切/prev-next/breadcrumb）。

**T-06 逮出回归并根修**：`prev_of(.np)`/`bp_name(.shell_active)` 形态
= computed 实参位**链 computed**（违反 app.at 头注装配纪律），VM 臂
容忍、vue 臂解析为 state 字段读 → undefined → `TypeError: reading
'prev'` 整页白屏。全改单 fn 直呼（bp_name/prev_id_of/next_id_of/
prev_exists/next_exists 各自重导 resolve_key/prev_next），重建后
vue 页恢复。此教训已沉淀 catalog.at 注 + 报告 README。

### T-06 双臂对拍走查 + 截图归档

T-05 后双臂各全量走查一遍（选 3 个代表性 bp：长 spec 的
dashboard/overview、多 gotcha 的 row-list、带表格式 props 的
master-detail），截图归档 `docs/plans/reports/` 或计划附件；vue 臂
`auto build`（vue-tsc）绿为硬门。

**[✅] 双臂网格归档（2d2d9de05）**：`docs/plans/reports/p685/` 双臂
3bp × 3 节 + README（视检结论/复跑工具/工具脚本四件）。vue `auto
build` 绿（仅既有 chunk 体积告警）。markdown 双臂真渲染（标题层级/
wrong:/why:/right: 加粗/行内码 chip/列表/表格 chrome）、tab 直切、
头部件在位（视检 README 有据，AC-06/07/08 证据面齐）。**运行器边界
登记**：vue `auto run` 无条件要求 `<name>-back` API 后端成员，纯前端
应用错配（master 亦然，预存）——走查以 `auto build` + 直接 vite dev
等价替代。

### T-07 收尾

按改动面跑门禁档；`auto bp list --format at` 漂移负验证（blueprints/
未被触碰）；复审材料整理。

**[✅] 门禁档全绿（零新增红，全数 base 对勘）**：
- E2E 终态 2 连跑 exit 0（T-06 代码态）。
- `cargo t`（5434 集）：24 红 = layout×14（PLAN-686 在管）+ musk×6
  （base 0e541c7e5 原样复现，DEBTS 在案"待认领"）+ counter/shell_pack/
  dock_pager×3（base 原样复现）+ clipboard×1（并行跑剪贴板态竞争
  偶发；单测隔离 3/3 绿 + base 绿——环境类，非回归）。
- `cargo tv`（VM 语料 golden，824 集）：820 绿，4 红= musk 预存族。
- `auto bp list --format at` vs registry.at：零漂移；
  `git diff 0e541c7e5..HEAD -- blueprints/` 空。
- worktree 干净、探针实例/vite 全按 PID 收杀。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/blueprint/project.md（消费面与组装样板·组装摩擦结论） | P657-D1 P0 条目（"VM 轨跨 widget 回调载荷字面量化"）追加收口注记 + 新增规则：VM 臂组件注入回调（`on_*: msg`）携参再发射（体内 `on_x(expr)` 剥离快照形态）实参 = 发射点形参绑定值，形参名字面量化 forbidden | 本缺陷即 P657-D1 在案实例（active_nav:"id" 同型），PLAN-685 根修收口 | AC-01/AC-02 |
| SD-02 | add | docs/specs/blueprint/project.md（消费面与组装样板） | 新增：BP 消费型应用（bps-gallery 型）VM 臂点击路径属 E2E 应验面，双臂验收不得只盖 vue 构建 | PLAN-676 漏网教训 | AC-03 |
| SD-03 | add | docs/specs/blueprint/project.md（消费面与组装样板） | 新增：结构化文档展示（spec/gotchas 型 markdown 内容）双臂应消费 autodown 渲染件（`markdown` tag），裸 `pre` 纯文本仅限代码源码面 | 内容粗糙即违例实证；jade-garden 消费先例 | AC-06/AC-07 |

> **复审勘正（R-1，2026-09-22）**：SD 原表目标错位——`docs/specs/auto-vm/project.md`
> 是独立运行器（crates/auto-vm 薄 CLI）spec，非 VM 臂 UI 桥；`autoui-skill/project.md`
> 明文"不做：验证执行"。三 SD 归位 `blueprint/project.md`（on_* msg-ref 回调契约与
> P657-D1 的在案家）。规则文本零改动 → 非语义契约变更，plan_revision 维持 2。

## 6. 测试设计

- **单测**（T-02）：最小转发语料 + 三形态邻近断言，落既有 vm 测试组。
- **E2E**（T-03）：MCP 脚本断言状态与快照。
- **门禁档**：`cargo check -p auto-lang` + `cargo t`（T-02 后）；触及
  VM/编译器终了 `cargo tv`；零 aavm 触碰（不跑 taa）。应用层改动
  （T-05）属 Category A/B 之间——无 crates 改动时仅双臂走查 + vue
  build 门，不跑 docs_gen。
- **对拍**（T-06）：双臂截图三 bp × 三节，人工+MCP 快照双通道。

## 7. 验收标准

- **AC-01**：最小复现语料测试修复前红、修复后绿；测试名与根因注记可
  追溯。验证：`cargo t <新测试名>`。
- **AC-02**：bps-gallery VM 臂点击侧栏任一卡片，`selected_id` 终值 =
  所点条目 id（MCP `state_changes` 为证），详情区渲染该 bp 内容。
  验证：T-03 脚本 exit 0。
- **AC-03**：T-03 E2E 脚本可重复执行（干净起实例 → 断言 → 收尾），
  脚本与用法登记在 `.agents/skills/autoui-verifier/`。
- **AC-04**：门禁档全绿：`cargo t` 绿 + `cargo tv` 绿（预存红按
  AGENTS.md 口径豁免）；`git diff` 零 bps-gallery 消费方 workaround
  （T-02 语义的修复不经 app.at/catalog.at）。
- **AC-05**：首启竞态有根因结论并登记（修复或债条二选一，留痕）。
- **AC-06**：Spec 与 Gotchas 以 markdown 渲染件呈现：标题层级、列表、
  表格（master-detail props 表）、行内码、引用/代码块 chrome 双臂可见
  （截图为证）；不再是等宽纯文本块。
- **AC-07**：内容页三节 tab 直切可用：点击 tab 即切换该节内容，
  VM 臂切换不产生滚动位置依赖；tab 状态与所选 bp 联动正确
  （切 bp 回缺省节或保持，登记一种）。
- **AC-08**：头部专业化件双臂可见：面包屑（kind › name）、kind 徽标、
  变体计数、prev/next 导航（点击切换 bp 且内容同步）；vue `auto build`
  （vue-tsc）零新增错。
- **AC-09**：Reference 源码面 mono + 代码块 chrome 保持/改善，源码
  完整无损（反引号等字符原样呈现）。

## 8. 执行步骤

- **T-01 根因勘定**（AC-01）
  文件：新增最小语料 + 候选落点（§2.1）。操作：复现→分叉→钉根因。
  验证：失败单测存在且红；根因注记落档。
- **T-02 根修**（AC-01/AC-04）依赖 T-01。
  文件：以 T-01 结论为准（候选：`vm_bridge.rs` / codegen emit 面）。
  验证：新测试绿 + 三形态断言绿 + `cargo t` 零新增红。
- **T-03 E2E 门禁**（AC-02/AC-03）依赖 T-02。
  文件：`.agents/skills/autoui-verifier/scripts/test_bp_gallery_click.py`（新增）。
  验证：脚本 exit 0 × 2 连跑。
- **T-04 首启竞态勘定**（AC-05）依赖 T-01（共享复现基建）。
  验证：结论登记（修复进 AC-04 门禁，或债条编号可查）。
- **T-05 内容页重设计**（AC-06/07/08/09）依赖无（可与 T-01 并行；
  点击缺陷修复前点击路径用 MCP set_value/状态注入或手点首条回退验证）。
  文件：`examples/bps-gallery/src/front/app.at`、`catalog.at`、
  `pac.at`（vue dep）。
  验证：双臂走查在案 + vue build 绿。
- **T-06 双臂对拍走查**（AC-06/07/08 证据面）依赖 T-05，T-02 后复跑
  点击全链。
  验证：三 bp × 三节截图归档；E2E 脚本复跑绿。
- **T-07 收尾**（AC-04）依赖 T-02/T-03/T-05/T-06。
  操作：`cargo tv` + bp 表漂移负验证 + 复审材料。验证：档位全绿。

## 9. 复审记录

- 2026-09-22 `stage: new` `outcome: pass`（revision 1 drafting 交付）：
  证据链现场实拍闭环（action 回执 + 回调源行号 + 数据面无恙截图），
  候选落点实勘到函数签名级；T-01 修复形态分叉（H1/H2）为显式有界决策
  点。授权范围见 §4。
- 2026-09-22 `revision: 1 → 2`（drafting 内用户扩围，未开工）：新增
  G4/G5 + T-05/T-06 + AC-06..09 + SD-03。技术面勘定：markdown 渲染件
  双臂通道齐备（VM=autodown_render.rs 实勘；vue=ui_gen/vue.rs :12857
  映射 @autodown/engine；jade-edit 消费先例 :220），快速导航裁定走
  状态分节 tab 而非锚点滚动（scroll_to 不可靠先例）。授权 = 用户原话
  "把这些设计和修改方式也记录到计划 685 里，之后一起实施"。
- 2026-09-22 `stage: work` `plan_id: PLAN-685` `plan_revision: 2`
  `outcome: pass` `code_commit: plan-685-dev 80bfec4b6..2d2d9de05`
  （T-01/T-02=80bfec4b6，T-03=d141073e6+b0fe5e3aa 加固，T-05=8b8c94c0c，
  T-06=2d2d9de05）`task_ids: T-01..T-07 全数`
  `evidence: 见 §5 各任务 [✅] 注（红→绿单测/E2E 多轮连绿 exit 0/
  双臂截图网格+README 视检/cargo t·tv 零新增红全 base 对勘/bp 零漂移）`
  `blockers: 无`
  `next: /auto-plan:review`
  勘定要点：①根因=剥离回调快照求值域盲区（H2 变体，非 vm_bridge
  装配点）；②首启竞态判同根（0/10 复发 vs 基线 2/5）；③vue 臂
  computed-as-arg 不解析实证（装配纪律的跨臂差异面，教训沉淀于
  catalog.at 注 + 报告 README）；④运行器 `-back` 成员要求与纯前端
  应用错配为预存边界（非本计划引入）。
- 2026-09-22 `stage: review` `plan_id: PLAN-685` `plan_revision: 2`
  `outcome: pass` `reviewed_commit: plan-685-dev e577cef99`
  `base_commit: 0e541c7e5` `dependency_revisions: auto-down fba6563
  (detached master sibling)` `spec_inputs: docs/specs/blueprint/
  project.md（消费面与组装样板/P657-D1）、docs/specs/goals.md
  （GOAL-007/011）`
  `acceptance_results: AC-01..AC-09 全 pass`——
  AC-01 红相 base 重建（review-red 树 0e541c7e5+纯测试：两参测
  `left: Str("v")` 红、字面量测绿）+ 审定点绿 3/3；AC-02/03 E2E
  ×2 exit 0（重建 exe 后复跑，3 卡跨 3 kind，首帧竞态 0/2）；
  AC-04 cargo t 24 红 / tv 3 红 / tf 等价 5887 集 30 红（含仅
  test-book 组合暴露的 book×7）——全数 base 对勘预存，零新增；
  AC-05 计划 T-04 注 + Q-1 裁定留痕；AC-06/09 截图视检
  （docs/plans/reports/p685/ 双臂网格：markdown 层级/加粗/行内码/
  列表/表格 chrome、Reference 反引号原样+变体 tab 节内）；AC-07
  运行期探针（review_probe.py：起始仅 Spec 节、三 tab 状态+内容
  双翻转、旧节卸载；review_keep.py：非缺省节跨 bp/跨 kind 切换
  保持）；AC-08 头部件 runtime 在位 + vue build 重建绿。
  `findings: R-1（已勘正）SD 三条目标错位——auto-vm/project.md 为
  独立运行器 spec、autoui-skill 明文不做验证执行，归位
  blueprint/project.md（P657-D1 在案家），规则文本零改动非契约变更；
  R-2（非阻塞）app.at SelectBp 空串守卫属 prev/next 降灰 UX（T-05
  特性面）非 T-02 语义 workaround，合规；R-3（非阻塞）E2E 首帧
  no-op press 窗口现象（~1/3）已由有界重试加固覆盖，与 T-04 同族。`
  `evidence: plan-685-dev 80bfec4b6..e577cef99 五提交 + 复审探针
  scripts（docs/plans/reports/p685/review_probe.py、review_keep.py）
  + 截图网格（同目录 vm//vue/）+ 红相重建（review-red 临时树已按
  guard 规程清除，方法与输出摘录留本记录）`
  `next: /auto-plan:merge`（SD 归位后的 blueprint spec 沉淀随 merge
  落地；Q-2 E2E 进 CI 留 merge/复审后续裁定）
  **独立性限制声明**：复审与实施同会话，裁定由工件重建（红绿两相
  重跑、运行期探针、全档门禁 base 对勘、截图直读）而非实施期总结。
- 2026-09-22 `stage: merge` `plan_id: PLAN-685:r2` `outcome: delivered`
  五 checkpoint：
  - `prepared`：reviewed 基线 261244a8f（rebase 后）；规范 diff =
    blueprint/project.md 消费端验收/展示契约两则（SD-02/03）+ 组装摩擦
    P657-D1 收口注记（SD-01，规则文本零改动）；投影目标
    .autoos/specs.json P685-1/2 + auto-lang/ui/plans.md 行 685 + INDEX
    再生（按构造零漂移——INDEX 只数包行，本 delta 未加包）；债册
    P657-D1 部分收口勘注（名字面量化形态收口/表达式形态残项留
    vm-component-parity）；delivery commit e34692db9（docs/projection-only
    后裔，实现与依赖零变化）。
  - `landed`：rebase plan-685-dev 0e541c7e5 基 → master 00e56202d，
    六对 range-diff 全等（80bfec4b6→cf306af9e/d141073e6→b0341b36a/
    8b8c94c0c→c926ace81/b0fe5e3aa→34fe4920e/2d2d9de05→d0252af93/
    e577cef99→261244a8f）；rebase 态 plan685 3/3 绿；master ff-only
    合入 **e34692db9**，tip 相等核验 ✓（无 merge 提交）。main 检出
    冒烟以 branch 端验证为准——main 工作树携带他方会话 WIP（本计划
    全程未触碰），在 main 构建不具代表性，如实登记。
  - `ledger_refreshed`：master 落面核验——specs.json P685-1/2 计 2 命中
    （sections 0/5，外科插入仅尾 hunk）、ui/plans.md 行 685、
    blueprint spec 契约节在位；specs.json 重投 24+/1-（`}` 尾行位移）。
  - `archived`：git mv docs/plans/archive/685-vm-bp-callback-arg-forward.md
    + status: archived（本提交）。
  - `cleaned`：见随后收据行（guard clean 后 worktree/分支/组目录清除）。
  事故与边界留痕：worktree 内 gen/deps pnpm junction 卡 guard——按
  661/671 既定程序 `cmd rmdir /s /q` 整目录清（gitignored 生成物）后
  guard clean；落地时点主检出他方 WIP 清单：iced/renderer.rs、
  shell_client.rs、workspace_preview.rs、dashboard.at、examples/** 等
  （归属会话未路由，未触碰未包含）。
- 2026-09-22 `cleaned`：双 worktree 各自新跑 guard clean 后移除——
  `D:/autostack/.wt/lang-685/auto-lang`（branch plan-685-dev 已删，删前
  指向 e34692db9 = landed delivery）+ auto-down 依赖 sibling（detached
  fba6563）；组目录 `D:/autostack/.wt/lang-685` 空删（ls 计 0 证实）。
  五 checkpoint 全闭环，**delivered**。

## 10. 待澄清事项

- **Q-1（已裁决，实现取定）**：首启竞态勘定为同根、T-02 修复覆盖
  （带修构建 0/10 复发），取"修复同批"档，不立新债；触发分派源头
  未捕获的残留不确定性已在 T-04 注登记，再现即被 E2E 首帧探针
  逮住。
- **Q-2（已裁决，merge 阶段按 CI 现状定）**：**E2E 不进 CI**。现状：CI 无
  GUI 实例 runner 形态（vm-files-ci=.at 语料、http-e2e=HTTP、
  build-bps-gallery=registry 漂移+vue build 构建面）；VM 臂点击门禁以本地
  脚本为准（`test_bp_gallery_click.py`，SKILL.md 步骤 1 登记用法）。
  merge 后 build-bps-gallery.yml 首跑为 watch 项（本地同工具链 vue build
  已绿，PLAN-676 AC-06 条件证据先例同款）。
- **Q-3（边界登记，不阻塞 AC-09）**：Reference 源码面保持 mono +
  代码块 chrome（反引号安全）；`@autodown/engine` lang 标签高亮未
  开（vue 臂引擎能力实测未启），VM 臂 autodown codeblock 无高亮
  （plan-019 边界）——双臂一致不阻塞。
- **Q-4（已裁决，实现取定）**：切 bp **保持** `active_section`
  （比较型浏览友好），首启缺省 "spec" 承担入口语义；SelectKind 同
  保持。已在 app.at model 注登记。
