---
plan_id: PLAN-686
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: layout-test-expectation-sync
author: [agent]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 7
---

# [PLAN-686] layout-test-expectation-sync

## 变更摘要

ui::layout 14 红（grid/snap/taskbar 族）勘正清偿。2026-09-22 会话已勘定归因：**代码红，非环境红**——`TASKBAR_HEIGHT` 48→56（`be2c17d62` 2026-09-03，PLAN-526 波9 T24，修"最大化窗底缘被任务栏盖住"）改常量未同步测试期望与共享期望值表：实际 800−56=744（半高 372）vs 期望 800−48=752（半高 376），4px=(56−48)/2。本计划把期望值族对齐 56，同步 `layout_cases.json`（Rust/TS 双侧共享表），顺手修 Vue 侧残留的 `TASKBAR_HEIGHT = 48`（文件头注释自承诺"与 layout.rs 同值"，实际已漂移 19 天），并勘正债册 R-11/P576 的"本机显示几何环境红"误归因。

## 目标

1. `cargo t ui::layout --no-fail-fast` 25/25 全绿（14 红清零），期望值与生产常量 `TASKBAR_HEIGHT = 56` 单一对齐。
2. 双侧共享表 `layout_cases.json` 与 TS 直译 `auto-man/assets/wm/layout.ts` 同步（reservedTaskbar=56；parity runner 全 ok）。
3. 债册勘正：ui::layout 族从"环境红豁免"清单摘除，误归因史留痕（~~划线~~+勘正注，不删史）。
4. 防复发：表头/常量注释双向互指同步规则。

**非目标**：musk×6（p053/p054 家族，在案待认领）与 counter×1（master 预存）——21 红中其余 7 个预存红不在本计划；不改生产常量语义（56 维持 T24 裁定）；不动 `scripts/ui-layout-parity.mjs` 机制。

## 架构方案

零生产行为变更（除 T-03 一行 TS 常量对齐）。测试期望值追随生产常量（T24 是用户反馈驱动的既定裁定，回退不合理）：

```
TASKBAR_HEIGHT（唯一事实源，crates/auto-lang/src/ui/layout.rs:78，=56）
  ├─ Rust 测试期望（layout.rs tests，752→744、376→372 等逐点改）
  ├─ 共享期望值表（layout_cases.json，reservedTaskbar 48→56 + expected 全表重算）
  │    └─ scripts/ui-layout-parity.mjs（node 直跑，喂表值，机制不动）
  └─ Vue 直译常量（auto-man/assets/wm/layout.ts:22，48→56；store.ts 消费点不动）
```

## 需求分析与背景调查

**授权记录**：2026-09-22 用户在勘定归因后裁定"OK，改期望值吧。请立项修改。"（改期望值方向 + 立项授权，本计划即其执行合同）。

**根因证据链**（2026-09-22 勘定会话实测）：
- 引入点 `git show be2c17d62`：仅改 `pub const TASKBAR_HEIGHT: f32 = 48.0 → 56.0` + 注释，零测试改动。
- 实跑 `cargo t ui::layout --no-fail-fast`（master a747531cd）：25 测 = 14 红 / 11 绿；失败签名 `rect {x:0, y:0, w:640, h:372} != expected {...h:376}`，确定性、任何机器同败——测试为固定 `VIEWPORT 1280×800` 纯函数，全文件零实时 OS 几何读取，"读实时 taskbar 几何/显示配置漂移"归因证伪。
- 历史"数量浮动 6↔9↔15"= nextest 默认 fail-fast 截停（首跑实测 12 败+4 未跑）。
- `layout_parity_cases_shared_table` 至今绿的原因：JSON 自喂 `reservedTaskbar:48` 当输入、期望亦 48 基，自洽闭合——恰为常量漂移盲区。
- 门禁史：Plan 526 T24 验证=实机截图（未跑单测）；Plan 564（09-05）基线红已记录"ui::layout grid 与 master_stack 全族"；Plan 576 误归因"本机显示几何环境红"；Plan 668 正式登记 R-11"环境红豁免"（豁免非跑绿）；pre-a747531cd 的 tf 别名不带 ui-iced，`#[cfg(feature = "ui-iced")] pub mod layout;`（ui/mod.rs:93-94）在 tf 下不编译=绿-by-absence；2026-09-22 `a747531cd` feature 统一后 tf 首次编译该模块，豁免红显形为 21 红（layout×14+musk×6+counter×1）。

**关联事实**：`auto-man/assets/wm/layout.ts:22` `TASKBAR_HEIGHT = 48`，其头注释"463 layout.rs TASKBAR_HEIGHT 同值"——生产双端已漂移（Vue 桌面预留少 8px，与 T24 所修同症）；消费点 `store.ts:55/79`。本计划纳入对齐（T-03），属授权方向的最小延伸，计划内显式标注供复审。

**Spec 现状**：`docs/specs/auto-lang/ui/overview.md:383` 记载共享表机制、未钉值——补同步规则注记（SD-01）。

## 详细设计

### 逐文件改动

1. **`crates/auto-lang/src/ui/layout.rs`（tests 模块 + 注释）**：
   - `usable_rect_excludes_taskbar_bottom` 752.0→744.0；`grid_one/two` 752→744；`grid_three/four` 376→372（含 y 376→372）；`grid_five` `ch = 752.0/2.0`→`744.0/2.0`；`grid_nine` `ch = 752.0/3.0`→`744.0/3.0`；`master_stack` 四测 752→744、376→372；`snap_left/right` 752→744；`cascade_rect_offsets_cap` 上界 376.0→372.0、752.0→744.0；`apply_layout_filters` 752→744。
   - 注释勘正：行 16"bottom = 48px"→56px、行 55"任务栏 bottom = 48px"→56px、行 349"扣任务栏 48…1280 x 752"→"扣任务栏 56…1280 x 744"，并在 `TASKBAR_HEIGHT` 常量注释处补"测试期望/layout_cases.json/layout.ts 须同值同步"互指。
2. **`crates/auto-lang/src/ui/layout_cases.json`**：`reservedTaskbar` 48→56；expected 全表按 56 重算（752→744、376→372、grid_nine expectedLast [853.333, 501.333, 426.667, 250.667]→[853.333, 496, 426.667, 248]；宽度 1280/640/426.667/576/704 不变；cascade 与 snap_middle 的输入坐标不动）；`_header` 补"reservedTaskbar 须与 layout.rs TASKBAR_HEIGHT 同值（56，PLAN-526 T24 起；48 系 T24 漏同步，Plan 686 勘正）"。
3. **`crates/auto-man/assets/wm/layout.ts:22`**：`TASKBAR_HEIGHT = 48`→`56`（注释保持"同值"承诺生效；store.ts 两消费点零改动）。
4. **`docs/plans/KNOWN-DEBT-AND-RISKS.md` 勘正三处**（~~划线~~留史+勘正注，沿 P002 行先例）：
   - R-11 行：~~环境红豁免 ui::layout 15 件~~ → 勘正注"非环境红：TASKBAR_HEIGHT 48→56（be2c17d62/526 T24）未同步期望的代码红，Plan 686 清偿"。
   - P576 行：~~ui::layout 族为本机显示几何环境红~~ 半边勘正（osconfig sibling_fixture 竞态仍为环境红，不动）。
   - P667-D2/P661-D7 基线段："环境红豁免=ui::layout 族+clipboard+ffi 闪测族"→摘除 ui::layout 族。
5. **`docs/specs/auto-lang/ui/overview.md`**（SD-01，见下）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md | 465 布局直译句补"shared 表 reservedTaskbar 与 layout.rs TASKBAR_HEIGHT 双端同值（56，PLAN-526 T24 起；Plan 686 勘正 TS 残留 48）" | 钉同步规则防再漂移 | AC-04 |

## 测试设计

- 主门（Category B 作用域）：`cargo t ui::layout --no-fail-fast` → 预期 25/25（其中 14 项由红转绿）。
- 双侧 parity：`node scripts/ui-layout-parity.mjs` → exit 0 全 ok（TS 直译喂同一张 56 基表）。
- 残留扫描：`rg -n "752\.0|, 752|376\.0|, 376" crates/auto-lang/src/ui/layout.rs crates/auto-lang/src/ui/layout_cases.json` → 零命中；`rg -n "TASKBAR_HEIGHT" crates/auto-lang/src/ui/layout.rs crates/auto-man/assets/wm/layout.ts` + JSON `reservedTaskbar` → 三处皆 56。
- 红普查（佐证非回归）：`cargo t --no-fail-fast` 一次，剩余红集 ⊆ {musk p053/p054×6, counter×1}（预存在案，非本计划作用域）。
- 不跑 tf 全档（冷 worktree，AGENTS 档位纪律；ui::layout 模块绿=tf 同测试集绿）。

## 验收标准

- **AC-01**：`cargo t ui::layout --no-fail-fast` 25/25 全绿。验证命令：`cargo t ui::layout --no-fail-fast`，期望 `Summary … 25 tests run: 25 passed`。
- **AC-02**：`node scripts/ui-layout-parity.mjs` exit 0、无 FAIL 行。
- **AC-03**：三处 TASKBAR_HEIGHT 事实源（layout.rs 常量 / layout_cases.json reservedTaskbar / layout.ts 常量）= 56，且 layout.rs+JSON 内零 752/376 残留（rg 证据）。
- **AC-04**：SD-01 spec 注记落地；债册三处勘正可见（~~划线~~+勘正注），ui::layout 不在环境豁免清单。
- **AC-05**：红普查剩余红 ⊆ {musk×6, counter×1}，layout 族零命中（如环境干扰致 musk 计数浮动，按在案豁免口径记录不阻塞）。

## 执行步骤

- [✅ 已完成] **T-01** worktree 组建组：`D:/autostack/.wt/lang-686/{auto-lang@plan-686-dev, auto-down@a615d693}`。基线复现：改动前 `cargo t ui::layout --no-fail-fast` = **25 测 14 红/11 绿**（干净树逐名与勘定清单一致，失败签名 `h:744 != expected h:752`）。
- [✅ 已完成] **T-02** layout.rs 期望值 14 测 26 值（含 usable 测 `u` 变体——首轮批量替换漏 `assert_rect(u,…)` 一处，复跑抓住补齐）+ 注释 4 处（行 16/56/349 陈旧 48 注记 + TASKBAR_HEIGHT 常量补三处同步面互指）。
- [✅ 已完成] **T-03** layout_cases.json：`reservedTaskbar` 48→56 + expected 全表 56 基重算（752→744、376→372、grid_nine expectedLast →[853.333, 496, 426.667, 248]）+ `_header` 补三处同值同步规则。
- [✅ 已完成] **T-04** auto-man `assets/wm/layout.ts` `TASKBAR_HEIGHT = 48→56`（附勘正注：Taskbar.vue 实渲染 h-14=56 而预留 48，Vue 轨同症活体现）。
- [✅ 已完成] **T-05** 门禁取证：`cargo t ui::layout --no-fail-fast` = **25/25 全绿**（AC-01 ✓）；`node scripts/ui-layout-parity.mjs` = **17 pass / 0 fail, exit 0**（AC-02 ✓）；残留扫描=三处事实源全 56，仅剩 snap_middle 光标输入坐标 [640,376]（非期望值，计划内保留）（AC-03 ✓）；全档红普查 5431 测 8 红=musk p053/p054×6（564-Q6/R-25 在案）+counter×1（master 预存）+shell_pack×1（auto-os↔auto-lang .at 双源 hash-lock 漂移守卫，pack 五件全 .at 与本 diff 零交集构造性证明），layout 族零命中（AC-05 ✓）。〔依赖：T-02/T-03/T-04〕
- [✅ 已完成] **T-06** 债册三处勘正（R-11 行 ✅修复化+勘正注 / P576 行 ui::layout 半边划线勘正 / P667-D2/P661-D7 豁免清单摘除 ui::layout 族）+ SD-01 spec 注记（ui/overview.md 465 直译句钉三处同值规则）。〔依赖：无〕
- [✅ 已完成] **T-07** 复审 pass（同会话复审已声明，命令级证据）→ 账本三件套落地（specs.json P686-1/P686-2 外科插入 indent=1 +17 行 + ui/plans.md 行 + INDEX 再生零漂移）→ rebase master（0e541c7e5，零交集；range-diff fix 提交 patch-id 相等 2a1422353=5c2ce28cb）→ master ff-only 100acb459（7 文件 74+/51−）→ 归档本提交。〔依赖：T-05/T-06〕

## 复审记录

- 2026-09-22 draft handoff：stage=new，PLAN-686 rev1。outcome=pass（授权在案：用户"OK，改期望值吧。请立项修改。"）。next=work。T-04（Vue 侧生产常量一行）为授权方向最小延伸，已于 §需求分析 显式标注——复审时请重点关注该条是否维持。
- 2026-09-22 review：stage=review | plan_id=PLAN-686 | plan_revision=1 | **outcome=pass** | reviewed_commit=5c2ce28cb（rebase 后；pre-rebase 2a1422353 patch-id 相等）| base=0969326d0（rebase 终基 0e541c7e5 零交集）| spec_inputs=docs/specs/auto-lang/ui/overview.md（SD-01 已随分支落地）| acceptance=AC-01..05 全 pass（证据：对提交态复跑 ui::layout 25/25+parity 17 pass/0 fail exit 0；三源=56；普查 8 红全预存定性）| findings=F-1（非阻塞：计划文本估「18 处」实为 26 期望值，簿记已正）| 同会话复审已声明——证据全部命令级可复现。Spec 影响说明：无 supersedes/new/touched（改动=对齐 526 T24 既定裁定的实现-测试一致化；规范增量仅 SD-01 注记一行）。| next=merge。
- 2026-09-22 merge 收据：delivery=master ff-only `100acb459`（7 文件 74+/51−：layout.rs+layout_cases.json+wm/layout.ts+KNOWN-DEBT+ui/overview.md+specs.json+ui/plans.md）；账本 P686-1（tests）/P686-2（reviews）+INDEX 再生零漂移；本提交归档（status: archived 随 git mv 翻转）；worktree 组清理随后续 wt-guard 收据。

## 待澄清事项

（无阻塞项。T-04 的范围延伸已在计划内显式声明并留复审锚点。）
