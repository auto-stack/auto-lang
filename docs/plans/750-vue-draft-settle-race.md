---
plan_id: PLAN-750
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vue-draft-settle-race
author: [zcode]
created_at: 2026-10-10
updated_at: 2026-10-10
plan_revision: 1
current_step: 0
total_steps: 7

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []        # 预置：SD-01 → docs/specs/auto-lang/ui/overview.md（vue 轨 store handler 事件重入与 await 悬窗契约节）
touched_goals: []             # docs/specs/goals.md 无直接 GOAL 条目（下游 jade 修复类）

affects: [auto-lang/ui, auto-lang/vm, auto-man]   # 按归因分支收敛，review 时定稿
---

# [PLAN-750] vue 轨草稿检查点保存不结算——Edit 分配悬窗重入竞态（定位与根修）

## 0. 变更摘要

jade-edit vue 轨（浏览器键入 → toolbar 保存）出现草稿检查点永不结算
（`event=saved` 墓碑缺失），导致 PLAN-032 LC 删除卫语句对「已保存页」
永久误阻（同运行 `draft_state='sending'` 卡死 + 跨运行磁盘 checkpoint
无墓碑双通道）。vm 轨同流程结算正常。

**本计划立项期已完成静态根因定位**（证据链见 §4.3）：宿主 .at 的
`Edit` 处理器在 `draft_begin/draft_alloc` 的 **await 悬窗**内被第二次
输入事件重入 → 双分配（d0003 + d0004）→ `tab.doc_id` 被覆写 → 排水
循环回执后按旧 doc_id 重扫失败 → `draft_seq` 永不落账 → Save 的
`draft_seq > 0` 守卫**静默跳过 settle**。工作区 `d0004` **空目录**为
双分配的直接磁盘签名（已实证在档）。

历史绿 → 确定性红的**触发变量**未定谳（codegen 输出回归 RC-A vs
后端 RTT 变慢 RC-B vs 潜伏竞态被时序点亮 RC-C），本计划以 2×2 载体
矩阵 A/B 判别后按分支落地根修，并补齐「vue 轨 store handler 事件
重入与 await 悬窗」的运行时契约成文（该契约缺口是本 bug 的结构性
土壤）。

## 1. 目标

### 1.1 目标

- G-1：vue 轨键入→保存后 `.jade/drafts` 落 `event=saved` 墓碑，与
  vm 轨行为等值（含多字符快速键入场景）。
- G-2：保存过的页面可正常删除；真在途脏档删除仍被 LC 卫语句正确
  阻断（卫语句本体不放宽）。
- G-3：定谳 2026-10-09 绿 → 2026-10-10 红（4/4）之间的行为变量，
  产出可复现的归因证据（载体矩阵 A/B）。
- G-4：按归因分支落地 auto-lang 侧根修 + 回归锁；若归因为 jade 侧
  潜伏竞态（RC-C），产出 .at 悬窗守卫修复建议供料包回 jade。
- G-5：补齐 vue 轨 store handler 事件重入/await 悬窗契约的规范成文
  （SD-01），锚定回归测试。

### 1.2 非目标

- 不改 PLAN-031 草稿协议语义（settle 三态、`settle_erev` 单调、
  墓碑永留、检查点清理语义全部保持）。
- 不放宽 PLAN-032 LC 卫语句（`draft_pending` 准确性依赖修复结算链，
  不是绕过卫语句）。
- 不在本仓修改 jade-edit 的 `.at` 宿主代码（jade 侧修复经供料包
  交接，RC-C 分支下才产出）。
- 不处理 jade e2e 的 D-21 负载窗瞬态族与 F-24-2 split 臂族（在案
  既有债，与本 bug 无关）。
- 不重建 release 工件（PLAN-748 观察②的独立事项）。

## 2. 架构方案

三层链路与分野点（jade 单源 .at 双轨）：

```
vm 轨:  MCP type_text → INPUT_TEXT 通道 → [VM 解释器] editor_store.at
        → back.api 进程内调用 → drafts.at 写 .meta          （健康）

vue 轨: 浏览器键入 → EngineEditor oninput → [转译 JS] useEditorStore.ts
        → fetch /api/* HTTP → [--server vm] AutoVM 后端 → drafts.at
        （分野：转译 JS 的 handler await 悬窗可被后续事件重入）
```

- **分野翼**：vue 轨前端 = auto-lang 转译器（trans/javascript.rs +
  auto-man vue.rs store composable 发射）产物；后端 = auto-lang
  AutoVM HTTP server（`auto run --server vm`）。二者都随 exe 重建
  而变（jade `scripts/serve-back.mjs:19` 默认取主检出
  `target/debug/auto.exe`；jade 的 gen/front/vue 再生也由该 exe 驱动）。
- **契约缺口**：`.at` 注释自述「run-to-completion 语义下无迟到竞态
  【await 悬窗外 handler 体原子】」——即宿主假定**只有显式 await 点
  可悬窗、悬窗内重入由身份重查模式兜住**。但 `Edit` 的分配臂
  （`doc_id == ""` → `draft_begin/draft_alloc`）重入时会**二次分配
  并覆写身份**，重查模式兜不住「身份被换」而非「身份被移除」的情形。
  该边界从未在 auto-lang 侧成文（SD-01 补齐）。
- **修复路径按归因分支**（§5.2），不预设单一修法。

## 3. 技术栈

- auto-lang Rust：`crates/auto-lang/src/trans/javascript.rs`（.at→JS
  转译）、`crates/auto-man/src/vue.rs`（vue 工程生成/store composable
  发射器）、AutoVM HTTP server（`--server vm` 路径）。
- 复现/验证：browser-use 技能（浏览器键入）+ autoui-verifier 技能
  脚本（`.agents/skills/autoui-verifier/scripts/`）；jade-edit 仓
  只读引用（e2e 工作区、spec、供料包）。
- 载体矩阵构建：git worktree 干净构建 91ae002d3（v0.4.2-2730）、
  ab7a650bd（v0.4.2-2747）、主检出 HEAD（c72ff953b 域）。

## 4. 需求分析与背景调查

### 4.1 授权与输入

- 需求来源：用户 prompt（2026-10-10）+ 上游供料包
  `jade-edit docs/upstream/2026-10-10-draft-settle-supply.md`（随
  jade `303e515` 入库）。授权范围：**auto-lang 仓内走 plan 流程定位
  并修复**；jade-edit 仓只读（证据/复现），修复不落 jade。
- 用户裁定要点：①vm 轨结算路径不得回归；②PLAN-031 语义保持；
  ③LC 卫语句不放宽；④建议排查面含 doc/epoch 绑定错位与
  v0.4.2-2835 域行为 diff。临时手动解法（清 `.jade/drafts/v1/`
  对应 doc 目录）已由用户告知终端用户。

### 4.2 时间线与载体（立项期实勘）

| 时点 | 载体 | 结果 |
|---|---|---|
| ≤10-09（数百次历史运行） | 历代旧载体 | matrix.spec 打字→保存→删除全弧绿 |
| 10-09 ~16:30（jade PLAN-039 T-03） | debug **2730-dirty**（jade `serve-back.mjs` 默认主检出 target/debug/auto.exe）+ 当日 regen 前端 | matrix「修测后 1.3m 干净绿」 |
| 10-10（4 轮） | debug **2835-dirty**（主检出 exe 重建，含 PLAN-748/749 合入 + 工作树 dirty）+ 10-10 regen 前端 | matrix 4/4 确定性败（删除弧 LC 误阻） |
| 10-10（A/B） | release **2747**（ab7a650bd 干净构建）作后端 | 同败（**前端 gen 仍为 2835 产物——A/B 只换了后端，未排除前端 codegen 面**） |
| 10-10（对照臂） | vm 轨一次性探针（draft-test-ws 干净工作区） | `event=saved` 即时落——健康 |

- 提交锚点：`91ae002d3`=v0.4.2-2730（10-08 14:21）；`ab7a650bd`
  =v0.4.2-2747（10-08 16:49）；`2fd9c0599`=v0.4.2-2835（10-10
  02:37）；`c72ff953b`=v0.4.2-2853（10-10 10:54，重建 HEAD）。
- **2730→2747 提交窗内唯一代码变更是 `ui/iced/renderer.rs`**
  （PLAN-740 MCP 通道重排，+227/−31）——vue 轨前后端在该子窗内
  零变化。⇒ 若 10-09 绿跑的前端确由 2730-dirty 产物承载，则
  「2747 后端 + 2730 代前端」应绿；供料包 release 同败测试若用的
  是 2835-regen 前端，则**前端 gen 是唯一变量（RC-A 锁定）**。
  此推理由 T-02 载体矩阵直接验证。
- 2747→2835 窗合入：PLAN-749（VM 串拼接显示臂/视图条件求值器
  三通道/`Http.post_json` 续体探针）、PLAN-748（`.store.X` 泛名
  展平 `resolve_expr_to_value`/视图层/autodown_editor padding/
  aura 转写）、PLAN-738 部分（stdlib assembly 多批）。**均随
  `--server vm` 后端与前端 regen 同时生效。**
- jade 侧变量排除：editor_store.at 结算协议代码窗口内未变（app.at
  有布局 workaround 加/撤，vm 探针已洗清语义）；`@autodown/engine`
  npm 包 0.5.0 自 09-05 未更新（pnpm-lock 09-20 后未动）——引擎面
  排除成立。

### 4.3 静态根因链（立项期已实证，T-01 动态确证）

一手证据（jade `e2e/.runtime/workspace/.jade/drafts/v1/`，10-10）：
- `d0001/s00001..2.meta`：saved→discarded，settle_erev=1/2——**同
  运行内结算正常**（erev=1 = 单次输入即检查点，未触发竞态）；
- `d0002/s00001.meta`：discarded，正常；
- `d0003/c00001.meta`：`event=checkpoint, erev=2, len=661/leno=659`，
  **无任何 s*.meta 墓碑**（`d_settle` 即使 verify 失败也会先写墓碑
  文件 ⇒ **draft_settle 从未被调用**，而非调用失败）；
- **`d0004/` 空目录**——第二次 `draft_alloc` 建目录后无任何检查点
  （双分配直接签名）。

机制（对照转译产物 `useEditorStore.ts:372-392`（Edit）/`266-323`
（DraftDrainOf）与 .at 源 `editor_store.at:390-437/451-555`）：

1. 键入字符 A → `Edit(textA)`：erev=1、dirty、`doc_id==""` →
   `await draft_begin()`（HTTP）——**悬窗**；
2. 键入字符 B（matrix.spec `keyboard.type` delay=25ms < 分配往返）→
   `Edit(textB)` 重入：erev=2（同步段落先落账），`doc_id` 仍空 →
   **再次分配**；
3. alloc#1 返回 d0003 → Edit#1 续体排水：快照 erev=**2**（B 已落）
   → `draft_checkpoint(d0003, epoch, 2, …)` → `c00001` 落盘 ✓
   （解释「唯一检查点为何 erev=2」）；
4. alloc#2 返回 d0004 → Edit#2 续体：`tab.doc_id = d0004`（**覆写**），
   `draft_state=='sending'`（Edit#1 排水已置）→ 跳过再排水（无
   c00002，d0004 空）✓；
5. checkpoint 回执 `ok:1` 到达 → 排水续体按 `doc_id==d0003` 重扫
   tabs → **失配**（tab 已是 d0004）→ `g<0` 静默停 → d0003 的
   seq/confirm 永不落账；tab（d0004 账面）`draft_seq==0`；
6. toolbar Save：`write_wiki` + 读回核验成功（脏标清零 ✓），但
   `if (tabs.value[k].draft_seq > 0)` 为假 → **settle 静默跳过**
   （无墓碑 ✓）；
7. 删除弧线：tab `draft_state=='sending'` 残留（同运行 LC 阻断，
   转译产物 `useEditorStore.ts:586`）+ 磁盘 d0003 无墓碑（跨运行
   后端 `draft_pending` 扫描阻断）→「草稿检查点在途」永久误阻 ✓；
8. vm 轨免疫：type_text 经 MCP 往返串行，相邻 Edit 间隔 >> 分配
  往返，悬窗重入不发生。

每一观测症状（erev=2 单检查点、无墓碑、脏标清零、双通道阻断、
双轨分野、d0001 正常/d0003 卡死并存）均被该机制逐点解释。

### 4.4 开放问题（T-02 判别）

- **RC-A**：2747→2835 前端 codegen/派发胶水输出变化（嫌疑面：
  `resolve_expr_to_value` 展平、视图条件求值器、store composable
  发射），使 Edit 分配臂行为/时序改变。
- **RC-B**：后端（AutoVM HTTP）`draft_begin/draft_alloc` 往返从
  <25ms 涨到 >25ms（键间隔阈值跨越；嫌疑面：PLAN-738 stdlib
  assembly / PLAN-749 VM 面），把原本键间完成的分配拉长到跨键。
- **RC-C**：前后端行为均未变，竞态一直潜伏，由 jade 侧 typing
  弧时序/环境节奏变化点亮（⇒ 修复主体在 jade .at 分配臂守卫，
  auto-lang 产出供料包 + 契约成文）。
- 注意：debug 载体「dirty」成分（当前主检出工作树 `ui/*` 六文件
  未提交改动）参与过 10-10 构建——矩阵需含「干净 HEAD」与
  「HEAD+dirty」两格以排除/纳入。

## 5. 详细设计

### 5.1 载体矩阵（T-02 决策实验）

固定 jade 源（jade HEAD 303e515，只读）、固定干净工作区与最小
复现配方（T-01 产物），变两轴：

| 格 | 前端 gen 由 | 后端 exe | 预期（按假设） |
|---|---|---|---|
| M1 | 2835 域（现状 gen 或 HEAD 再生） | 2835 域（主检出 debug） | 红（基线复现） |
| M2 | 2835 域 | ab7a650bd 干净构建 | 红 ⇒ RC-A/B 前端面；绿 ⇒ 后端独担（RC-B） |
| M3 | ab7a650bd regen | 2835 域 | 红 ⇒ 后端面（RC-B）；绿 ⇒ 前端面（RC-A） |
| M4 | ab7a650bd regen | ab7a650bd | 绿 ⇒ 竞态=2835 域引入；红 ⇒ RC-C（潜伏竞态） |
| M5（条件） | 91ae002d3 regen | 91ae002d3 | M4 红时补测（更老基面定谳潜伏起点） |

- 每格记录：`.jade/drafts/v1/` 目录形态（墓碑/空目录签名）+ 应用
  console 四类行（`draft-saved/draft-error/draft-settled/
  draft-settle-failed`）+ `/api/draft_begin`、`/api/draft_alloc`
  RTT 实测（≥20 样本分位数）。
- 产出：归因定谳记录（决策工件）落本计划 §9，含矩阵全表与判读。
- 前端 regen 产物 diff（M1 vs M3 的 `useEditorStore.ts`/App.vue/
  派发胶水逐文件 diff）随决策工件入库 `docs/plans/evidence/`。

### 5.2 修复设计（按 T-02 分支）

- **分支 A（前端 codegen/派发回归）**：定位 2747→2835 中改变
  Edit 分配臂/事件派发语义的具体提交，回滚或修正该生成行为；
  以 M1 vs M3 的 gen diff 为黄金证据。回归锁：转译输出 golden 或
  派发语义测试（最小 .at → 转译 JS → node 侧重入仿真断言身份
  单次分配）。
- **分支 B（后端 RTT 回归）**：定位 2747→2835 使 AutoVM HTTP 短
  往返变慢的变更（PLAN-738/749 面），恢复性能；回归锁：draft
  API 短往返基准测试（阈值锚定实测绿值分位数）。
- **分支 C（潜伏竞态点亮）**：auto-lang 侧不改运行时行为（无回归
  可修）；产出 jade 供料包：`.at` 分配臂加 in-flight 门（如
  `draft_allocating` 状态位，悬窗内重入只更新正文不重走分配；
  或分配结果回写时校验 tab 身份未变），附本计划 §4.3 机制链与
  d0004 签名证据；auto-lang 侧落 SD-01 契约成文。是否追加防御性
  派发串行化（运行时行为变更，影响全部 vue 应用）交用户裁定
  （§10-Q2）。
- 无论分支：**vm 轨零触碰验证**（分支 A 只动转译器输出——vm 轨
  不消费转译产物；分支 B 动 VM——需 vm 结算探针复测）。

### 5.3 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/overview.md（或按归因收敛至 auto-man/trans 对应 overview——review 定稿） | before：vue 轨 store handler 事件重入与 await 悬窗语义无成文（宿主只能从 .at 注释反推）。after：成文契约——①handler 体除显式 await 点外原子；②await 悬窗内同 store 事件**可重入**；③悬窗内身份分配类状态变更的幂等/单飞责任在宿主 .at（给出分配臂守卫范式）；④vm 轨消息派发与 vue 轨浏览器事件派发的时序差异（MCP 往返串行 vs 引擎逐键发射）。 | 本 bug 的结构性土壤是契约缺口；无论修复落点，该边界必须显式化，否则同类竞态会复发 | AC-05/AC-06 |
| SD-02（条件） | add | docs/specs/auto-lang/vm/…（HTTP/装配性能面，分支 B 时定稿） | before：AutoVM HTTP 短往返无性能预算。after：draft 类短端点往返预算锚（分位数阈值 + 基准测试指针）。 | RC-B 成立时防复发 | AC-03/AC-05 |

无 Spec 影响的变更不存在——SD-01 无条件落账。

## 6. 测试设计

- **复现/判别（T-01/T-02）**：最小 vue 复现配方（干净 ws + jade
  HEAD + 浏览器双字符快速键入 → 保存 → 断言 `.jade/drafts` 形态
  与 console 行）；复用 autoui-verifier 技能脚本与 browser-use，
  不新造 ad-hoc 框架（脚本入 `docs/plans/evidence/p750/`）。
- **修复回归锁（T-03，按分支）**：分支 A=转译输出/派发语义锁；
  分支 B=draft API RTT 基准锁；均入 `crates/auto-lang/src/tests/`
  （命名 `plan750_*`），跑位按触面（A→`cargo tu` 族范畴的单测或
  inline；B→VM 面随 `cargo t`）。锁测试须自带截止时间（网络/等待
  类不入 tu 族——AGENTS.md fix-ui-tier 纪律）。
- **vm 轨不回归（T-04）**：vm 轨结算探针（供料包对照臂复测：
  MCP type_text → 保存 → `event=saved` 即时落）+ `cargo t` 全日常
  面零新红；分支 B 追加 `cargo tv`。
- **jade 侧终验（T-05，jade 会话承接）**：matrix.spec 全绿 +
  vm_matrix merged 20/20 维持（撤验钉并回填供料包回执——669/682
  先例流程）。

## 7. 验收标准

| ID | 判据 | 验证方法 |
|---|---|---|
| AC-01 | vue 轨键入（含 ≥2 字符快速键入）→保存后 `.jade/drafts` 落 `event=saved` 墓碑，检查点清理，无孤儿空 doc 目录 | T-01 复现配方改后复跑 + `.jade/drafts/v1` 目录形态断言 |
| AC-02 | 已保存页可删除（LC 不误阻）；构造真在途脏档（键入不保存）删除仍被阻断 | 复现配方删除弧线 + 真脏档对照弧 |
| AC-03 | vm 轨结算路径零回归 | vm 轨探针复测（type_text→保存→saved 即时落）+ cargo t 零新红（分支 B 加 tv） |
| AC-04 | 归因定谳：载体矩阵全表 + 判读落 §9，指认具体变更（提交号）或证伪 exe 归因（RC-C），证据可复现 | T-02 决策工件 + evidence 入库 |
| AC-05 | 修复带回归锁（按分支：转译/派发语义锁或 RTT 基准锁）且锁在修复前红、修复后绿 | 锁测试前后跑对照 |
| AC-06 | SD-01 契约成文落账（spec delta 经 review 定稿路径） | spec 文件 + `python scripts/spec-index.py` 回读 |
| AC-07 | jade 复验交接件：供料包回执节（修复说明+复验口径+RC-C 时附 .at 守卫建议）落 jade 侧流程（由 jade 会话执行，本计划交付交接件） | 交接件在 evidence/ + §9 记录 |

门禁（复审随身）：`cargo check -p auto-lang` + 裸 `cargo t` +
按触面档（转译器→`cargo tu` 范畴；VM→`cargo tv`；trans→`cargo tt`）；
本计划不动 aavm/book 面。

## 8. 执行步骤

> worktree：`git worktree add D:/autostack/.wt/lang-750/auto-lang -b
> plan-750-dev`（先在 master commit 本计划骨架与 `.next-id`）。
> 并行纪律：worktree 内只跑 check/scoped 档；载体矩阵构建用独立
> worktree（lang-750-m2/m3/m4 临时构建位，验后即清，wt-guard 护栏）。

- [ ] T-00 契约锚定与基面复核：读 ui/auto-man/trans specs 现状
  成文定位 SD-01 落点；复核主检出 dirty 六文件与 10-10 构建的
  关系（确认 M5 需不需要 HEAD+dirty 格）。验证：SD-01 落点草案
  入 §9。涉及：docs/specs/auto-lang/ui/overview.md 等。
- [ ] T-01 最小复现仪器化：jade HEAD + 干净 ws + 双字符快速键入
  →保存→删除全弧；采 console 四类行 + `.jade/drafts` 形态 +
  draft API RTT。验证：M1 格复现 4/4 形态（含 d00XX+1 空目录
  签名）。产出：evidence/p750/repro.md + 脚本。
- [ ] T-02 载体矩阵判别：构建 ab7a650bd/91ae002d3 载体，跑
  §5.1 矩阵全格 + gen diff。验证：归因定谳记录（提交号级或
  RC-C 证伪）入 §9；AC-04。
- [ ] T-03 根修落地（按 T-02 分支）：分支 A/B 修 auto-lang 对应
  面 + 回归锁；分支 C 产 jade 供料包草稿。验证：AC-01/AC-02/
  AC-05（改后复现配方全绿）。
- [ ] T-04 门禁与 vm 不回归：cargo check/t + 触面档 + vm 轨结算
  探针复测。验证：AC-03 + 零新红。
- [ ] T-05 交接件定稿：供料包回执节（复验口径：matrix.spec 全绿
  + vm_matrix 20/20 + 真脏档阻断对照）。验证：AC-07。
- [ ] T-06 规范增量落账：SD-01（及条件 SD-02）成文 + spec-index
  回读。验证：AC-06。
- [ ] T-07 复审交接：execution_done → /auto-plan:review。

依赖：T-02 依赖 T-01 配方；T-03 依赖 T-02 定谳；T-04..T-06 依赖
T-03；T-05/T-06 可并行。

## 9. 复审记录

（work/review 过程追记；T-02 决策工件与矩阵全表落此处）

- 2026-10-10 stage:new | PLAN-750 | plan_revision 1 | 立项期静态
  根因链与 d0004 空目录签名实证在 §4.3；归因三假设（RC-A/B/C）
  与判别矩阵设计在 §4.4/§5.1 | next: work（T-00 起步）。

## 10. 待澄清事项

- Q1（信息性，T-02 可自答）：供料包 release 2747「同败」A/B 的
  前端 gen 由哪个 exe 再生（若为 2835 产物则其结论只排除后端
  2747→2835 域，未排除前端 codegen 面——§4.2 推理已内建）。
- Q2（分支 C 时需用户裁定）：auto-lang 是否在 vue 轨事件派发层
  加防御性串行化（运行时行为变更，波及全部 vue 应用），还是
  契约成文 + jade 侧 .at 守卫即收口。
- Q3（非阻塞）：PLAN-748 观察②「release 工件未重建」与本计划
  修复后 jade 复验所需 release 重建时序（jade 侧 D-01 门协同）。
