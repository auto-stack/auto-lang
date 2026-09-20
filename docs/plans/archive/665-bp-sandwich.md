---
plan_id: PLAN-665
status: archived                 # drafting → executing → execution_done → reviewed → archived
feature_name: bp-sandwich（layout/sandwich 应用壳蓝图）
author: [agent]
created_at: 2026-09-20
updated_at: 2026-09-20
completion_kind: delivered
plan_revision: 1
current_step: 5
total_steps: 5

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/blueprint/project.md]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [blueprint]          # 受影响的 specs 路径
---

# [PLAN-665] bp-sandwich

## 0. 变更摘要

新增 Blueprint 包 `blueprints/layout/sandwich/`——桌面应用**上中下三层壳**
骨架（上 toolbar 定高 / 中弹性主区（可含 sidebar 分栏）/ 下 statusbar 定
高），fn-free 纯布局 + slot 形态（PLAN-075 骨架 bp tier）。variants =
`default`（含 sidebar 出口）/ `full`（无侧栏，内容独占）。双臂验接入
`examples/bp-gate/` 第四单元。**零渲染器改动**——骨架直接物化 2026-09-20
落地的 StretchLine 定高行语义（df90a448b）与 viewport 契约
（docs/specs/widgets/viewport-boundary.md），把"应用壳怎么写才不塌"固化成
可复用、进回归门的资产。

命名：用户提案 `sandwich`（两片薄面包=上下定高栏，主馅=弹性主区）；合并
前为改名窗口（`10. 待澄清`）。

## 1. 目标

- **G1**：普通 app 填四个 slot（toolbar/sidebar/content/statusbar）即得
  双臂语义正确的满窗壳，不再手写 `h-screen/h-8 shrink-0/flex-1
  items-stretch/overflow-*` 组合。
- **G2**：把本次 041 回归（655 StretchLine 行高 auto 语义漏掉定高行场景，
  df90a448b 修复）与 663 视口契约的教训物化为语料回归锚——渲染器高度语
  义再变动时 bp 包先报警。
- **G3**：补上 bp 生态缺位：官方集有 `layout/status-bar`（底栏骨架）与
  `navigation/sidebar-shell`（web 页框架）但没有桌面应用竖向壳；业界锚点
  （VS Code/Eclipse workbench、PatternFly Page：header+main+footer）证明
  该模式属策展共识。

**非目标**：可拖拽分隔条（resizable splitter——独立 widget 原语，031 时
期塌缩坑在案，另立计划）；主题/token 层差异；menubar/toolbar actions 契
约（应用侧既有 `actions{}` 机制，不进骨架）。

## 2. 架构方案

### 包结构（六问契约 docs/specs/blueprint/contract.md Q5）

```
blueprints/layout/sandwich/
├── spec.md                  # TOML frontmatter + NL 六问
├── reference/
│   ├── default.at           # 含 sidebar 出口（w-56 左栏）
│   └── full.at              # 无侧栏变体
└── gotchas.md               # 三条红线 + VM 快照边界 + ASCII 词表纪律
```

### slot 契约（Q4：结构性差异走 slot）

| 出口 | 形态 | 说明 |
| --- | --- | --- |
| `toolbar` | `slot(name: "toolbar")` | 顶栏内容；壳供给 h-8 shrink-0 行 |
| `sidebar` | `slot(name: "sidebar")` | 仅 default 变体；壳供给 w-56 shrink-0 overflow-y-auto 列 |
| `statusbar` | `slot(name: "statusbar")` | 底栏内容；壳供给 h-6 shrink-0 行；可直接嵌 `layout/status-bar` |
| （默认出口） | 裸子节点 | content 主区；壳供给 `flex-1 overflow-hidden` 弹性列，消费方内部自行分栏 |

零 props 骨架（比 status-bar 更纯）——四口全是结构性出口，值 props 无处
安放；如消费方要文本 statusbar，组合 `layout/status-bar` 即可（bp 组合优
于 props 重复）。

### 骨架语义（default 变体，donor=041-auto-edit df90a448b 后形态）

```
col (style: "h-screen w-full")                       ← 满窗壳（全屏外壳惯例=h-screen，viewport-boundary 契约）
├── row (style: "h-8 shrink-0 items-center px-2")    ← toolbar 出口所在定高行
├── row (style: "flex-1 items-stretch overflow-hidden w-full")  ← 弹性主行（定高行：flex-1 列内直接子位转写 + items-stretch）
│   ├── col (style: "w-56 shrink-0 overflow-y-auto bg-card border-r border-border p-2")  ← sidebar 出口
│   └── col (style: "flex-1 overflow-hidden")        ← content 默认出口（弹性列）
└── row (style: "h-6 shrink-0 items-center px-2")    ← statusbar 出口所在定高行
```

slot 出口元素（`slot(name:)` / 裸子）内嵌在上述容器内——Plan 476 的
`slot_fills` 机制（aura_view_builder）vm/vue 双轨同语义（033-slots vue 证
、sidebar-shell content 出口 vue 证；**VM 轨 use-bps×slot 组合链路未实勘
——T-00 探针决策**）。

## 3. 技术栈

- 包资产：Markdown spec（TOML frontmatter）+ `.at` fn-free 参考实现。
- 双臂：vue 轨（a2ts 发射 + playwright 截图基线）× VM 轨（aura 解释臂 +
  MCP boot/state/snapshot 断言）；门 = `examples/bp-gate`（PLAN-075 形态）。
- 零 Rust 改动预期；若 T-00 揭示 VM 轨 use-bps×slot 缺口，缺口修补按
  `10. 待澄清` D-2 裁决后另划任务或拆债。

## 4. 需求分析与背景调查

**授权（2026-09-20 用户会话）**：用户确认"按照这个套分析，实现一个新的
bp"，命名提案 `sandwich`（AppShell 弃用）；scope = blueprints/ 包 + bp-gate
接线；未设预算/时限。命名尚待合并前最终确认（D-1）。

**背景（已实勘）**：

| 事实 | 来源 |
| --- | --- |
| StretchLine 定高行语义修复（骨架的引擎前提） | df90a448b（2026-09-20 落 master） |
| 视口边界契约：全屏外壳惯例 h-screen、Screen 族重锚定 | docs/specs/widgets/viewport-boundary.md |
| bp 六问契约 / 三通道 / 版本面 | docs/specs/blueprint/contract.md |
| bp-gate 形态：R-B 沙箱构建 / R-C VM root 投影断言 / units.mjs 单元台账 / `--update-snapshots` | examples/bp-gate/README.md |
| slot 双轨机制（slot_fills：命名出口+默认出口，"Vue 语义"） | crates/auto-lang/src/ui/aura_view_builder.rs:142-179（Plan 476） |
| `use bps.<kind>.<name>.reference.<variant>` 双轨解析门 | crates/auto-lang/src/lib.rs:7575（PLAN-639） |
| 缺口：bp-gate 三单元全为值 props 消费，slot children 投影双臂未进语料 | examples/bp-gate/scripts/units.mjs + host/src/front/app.at 实勘 |
| donor 骨架 | examples/ui/041-auto-edit/src/front/app.at（view 块） |

**准入依据（官方默认集五条，contract Q5）**：#1 业界共识锚（VS Code /
Eclipse workbench、PatternFly Page header+main+footer）；#2/#4 骨架 bp 例
外（PLAN-075 和解注记：布局骨架+slot 为契约本体）；#3 palette 最小集
`["text", "separator", "icon"]` ⊆ WidgetRegistry；#5 本 spec 六问可答。

## 5. 详细设计

### 5.1 spec.md frontmatter 草案

```toml
+++
kind = "layout"
name = "sandwich"
palette = ["text", "separator", "icon"]
extension_points = ["toolbar", "sidebar", "content", "statusbar"]
variants = ["default", "full"]
+++
```

### 5.2 reference/default.at 骨架（草案，fn-free）

```
widget Sandwich {
    view {
        col (style: "h-screen w-full") {
            row (style: "h-8 shrink-0 items-center gap-0 px-2") {
                slot(name: "toolbar") {}
            }
            row (style: "flex-1 items-stretch overflow-hidden w-full") {
                col (style: "w-56 shrink-0 border-r border-border bg-card p-2 overflow-y-auto") {
                    slot(name: "sidebar") {}
                }
                col (style: "flex-1 overflow-hidden") {
                    // 默认出口=content（裸子落此；Plan 476 默认槽语义）
                }
            }
            row (style: "h-6 shrink-0 items-center gap-3 px-2") {
                slot(name: "statusbar") {}
            }
        }
    }
}
```

（`full.at` 删 sidebar 列，余同；出口落位语法以 T-00 实勘结论为准——
slot outlet 在子构建器内的具体书写形式可能是 `slot(name:)` 节点或等价
outlet 元素，以 476 实现与 033-slots 样本对齐。）

### 5.3 gotchas.md 要目

1. **三红线**：fill 列禁 justify-center/end（高度让渡塌缩，663 家族）；
   items-stretch 行自身须有确定高（655 语义，df90a448b 前手写即塌）；滚动
   归 overflow，勿让内容高反向决定骨架。
2. **VM 快照边界**（bp 通用 F-1）：bp 子件子树对 MCP 快照不可见，消费断
   言走 root 投影字段/标记行。
3. **CJK 词表纪律**（G-2）：fixture 文案 ASCII，禁 CJK 域索引算术。
4. **组合建议**：statusbar 直接嵌 `layout/status-bar`；content 内分栏用
   `row flex-1 items-stretch`（4xx 分栏样板）。

### 5.4 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
| --- | --- | --- | --- | --- | --- |
| SD-01 | add | docs/specs/blueprint/project.md（单元台账/首证清单节） | 单元台账增第四单元 `sandwich`（layout/sandwich，fn-free 骨架+四 slot 出口形态） | 双臂 gate 台账与包库同步 | AC-06 |

（包本体在 `blueprints/` 资产库，非 docs/specs——与 status-bar/filetree
先例同口径，project.md 只记台账行。）

## 6. 测试设计

- **包完整性**：`auto bp list/check`（registry 扫描：spec name ↔ 目录名、
  variant ↔ reference 文件、palette_drift 零漂移）。
- **双臂 gate（bp-gate 第四单元 `unit: sandwich`）**：
  - vue 臂：host 单元区互斥词表 needle + per-unit clip 截图基线
    （800x640 viewport，clip.y=208×3）；新增几何断言——statusbar needle
    boundingBox 底缘 ≈ 视口底、content 区高 > toolbar+statusbar 高（防
    "只保留内容高"回归的自动化断言，本次 041 事故的正向锚）。
  - VM 臂：boot 绿 + `autoui_state` root 投影字段（`sw_*` 前缀，如
    `sw_slots`/`sw_content_rows`）+ snapshot needle `unit: sandwich`。
- **variants**：default/full 两变体均在 host 单元内消费（full 以第二实例
  或互斥 fixture 覆盖，units.mjs 登记）。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 | 预期 |
| --- | --- | --- | --- |
| AC-01 | 包完整性过门 | `auto bp check`（blueprints/ 根） | registry 扫描零错误、palette 零漂移 |
| AC-02 | vue 臂渲染正确 | `pnpm gate --arm vue`（bp-gate） | sandwich 单元 needle 命中、clip 基线建立且绿 |
| AC-03 | VM 臂消费正确 | `pnpm gate --arm vm` | boot 绿、投影字段等值、snapshot needle 命中 |
| AC-04 | 弹性语义自动化断言 | e2e 几何断言（boundingBox） | statusbar 贴视口底、content 弹性区高显著大于定高栏（视口 640 下 content ≥ 300px） |
| AC-05 | 双变体齐全 | default/full 两实例渲染对比 | full 无侧栏列、content 独占；两变体 reference 均过编译/解释 |
| AC-06 | 台账与规范增量落地 | `pnpm gate` 全绿 + project.md 台账行 | SD-01 落档 |

## 8. 执行步骤

| ID | 任务 | 依赖 | 产出/验证 | AC |
| --- | --- | --- | --- | --- |
| T-00 | **VM 轨 use-bps×slot 组合探针（决策工件）**：最小 probe app（use bps 引 sandbox 包 + `slot(name:)` 四口 + 默认出口），`auto run`/`run -r vm` 双臂跑+截图/快照 | — | 决策记录：双轨是否原生可用；VM 缺口清单→D-2 裁决（本计划内修补 / 拆债改契约形态）。探针落 scratch/ | AC-03 前置 |
|   | [x] **T-00 已完成（2026-09-20）**：探针沙箱=`D:/autostack/.wt/lang-665/scratch/`（worktree 外，deps junction 只落沙箱；auto.exe=worktree 构建 172536658）。**vue 臂 PASS**：`auto build --gen-only` → SandwichShell.vue 四 outlet（`<slot name="toolbar|sidebar|statusbar">`+默认 `<slot>`）+ App.vue `<template #x>` targeting+裸子默认槽全数正确，骨架样式逐字落位。**VM 臂 PASS**：`auto run -r vm` boot 绿；snapshot v2(rendered) **完整暴露子件子树**——col>row(toolbar)>row[scrollable(sidebar),col(content×2)]>row(statusbar) 逐层吻合，overflow-y-auto→scrollable 转写正确；2x 窗口截图四区文本确认+中心列像素扫描（toolbar 文本 y≈59/statusbar y≈762/窗口高 836）弹性中区无塌缩。**D-2 裁定=零缺口**（组合链原生可用，无需 interpreter 修补）。**F-1 观察修正**：本配置下 snapshot v2 可见 bp 子树（疑快照契约演进，664 SD-01 同期）→ T-03 VM 断言可增 slot 填充 needle（叠加不替换 root 标记+投影） | | | | |
| T-01 | 包骨架落地：`blueprints/layout/sandwich/{spec.md, reference/default.at, reference/full.at, gotchas.md}` 按 §5 草案+T-00 结论 | T-00 | `auto bp check` 绿（AC-01） | AC-01, AC-05 |
|   | [x] **T-01 已完成（2026-09-20，worktree 提交 f2a8a2ce5）**：四件全落。验证：`auto bp list` 绿（layout/sandwich 在册，EXIT=0）+ palette 零漂移（`plan657_bp_admin_tests::t03_corpus_references_generate_and_palette_zero_drift` 全包扫描 PASS）+ bp 域 46 测 45 绿——唯一红 `plan640_bp_tests::t04_vm_track_empty_state_reference` = **master 预存红**（A/B：主检出同 commit 同红；`Undefined symbol: on_primary in module EmptyStateFirstUse`，与 sandwich 零交互，见 §10 D-4）。**两处草案偏差（有据）**：① 根容器 `h-screen`→`h-full`（嵌套固定高单元内 h-screen=100vh 会爆出单元；041 donor 根容器同款 h-full，vue 脚手架 `html,body,#app{height:100%}` 链实证；满窗由消费方根决定）；② full 变体 widget 名 `SandwichShellFull`（default/full 同页双消费时 VM registry 按名注册互踩——EmptyStateFirstUse/NoResult、SidebarShellCompact 先例） | | | | |
| T-02 | bp-gate 接线：host app.at 第四单元区（标记行+互斥词表 fixture+default/full 双实例）+ units.mjs 登记（clip.y=624） | T-01 | 单元台账更新 | AC-02 前置 |
|   | [x] **T-02 已完成（2026-09-20，worktree 提交 7f1fddef7）**：host app.at 第四单元（default 四口消费+**full 嵌 content 出口组合形态**——双变体同单元覆盖+G1 壳内壳证明）+ sw_*/swf_* root 投影 + units.mjs 登记（clip y=624 h=416）+ README 台账行。**几何偏差（有据）**：视口 640→1040（GEOMETRY 单源）——AC-04 要求 content ≥300px，208px 单元容不下；sandwich 单元 416px（=2×208 节奏）且底缘=视口底（贴底断言）；前三单元 clip 绝对坐标不变，干净门实证基线零扰动 | | | | |
| T-03 | 双臂门建绿：`--update-snapshots` 建基线 → `pnpm gate` 全绿；vue e2e 补几何断言（AC-04） | T-02 | gate 双臂绿收据+基线图 | AC-02, AC-03, AC-04, AC-06 |
|   | [x] **T-03 已完成（2026-09-20）**：units.spec.ts 增 AC-04 五断言（statusbar 贴视口底±8px/content 弹性区≥300px/三层序/sidebar 左列/嵌套壳不越外层）。**两轮全绿**：①`--update-snapshots`（sandwich.png 新建，前三基线未动——git status 零 M 实证）；②干净门（无 update）幂等绿。vue 6 测全过（4 单元渲染+全标记+几何）；VM 臂 boot 绿+三组投影等值+七 needle 全中（root 标记+六 slot 填充文本，T-00 修正断言面首用） | | | | |
| T-04 | 规范增量落地：project.md 台账行（SD-01）+ 判定记录补记（donor=041/业界锚）；`python scripts/spec-index.py` | T-03 | spec-index 再生收据 | AC-06 |
|   | [x] **T-04 已完成（2026-09-20，worktree 提交 53ba8b346）**：project.md gate 段第四单元台账行（SD-01）+模块清单 layout/sandwich 行+官方集计数 14→15；spec-index 再生（INDEX blueprint 15+sandwich 明细行）。**外科保卫**：再生会冲掉 664 手插 ui 行注记（再生源未含该状态面）——已恢复，最终 INDEX delta 仅 sandwich 两行（见 §10 D-6） | | | | |

**Worktree**：`git worktree add D:/autostack/.wt/lang-665/auto-lang -b
plan-665-dev`（本计划纯资产+examples，预计无 auto-down 依赖位；若 T-00
揭示需 interpreter 修补则补建组内 auto-down 位）。

## 9. 复审记录

- 2026-09-20 draft handoff（plan_revision 1）：stage=new，PLAN-665 r1，
  outcome=pass（T-00 探针前置已内建，无阻断待决——D-1/D-2 均有缺省路径），
  next=work。
- 2026-09-20 work handoff（plan_revision 1）：
  `stage: work | PLAN-665 | r1 | pass | code: plan-665-dev@.wt/lang-665/auto-lang
  f2a8a2ce5+7f1fddef7+53ba8b346（基点 172536658；依赖位：组内
  .wt/lang-665/auto-down@6a9df40 lang-665-dep——worktree 构建 autodown-core
  path 依赖所需，无改动）| task_ids: T-00..T-04 | evidence: T-00 探针双臂
  PASS（scratch/ 决策工件+VM 窗口截图+snapshot v2 子树实证）；T-01 auto bp
  list 绿+palette 零漂移（plan657 t03）+bp 域 46 测 45 绿（唯一红=master
  预存）；T-03 bp-gate 双臂两轮全绿（含 AC-04 几何断言+七 needle）；
  T-04 spec-index 再生收据 | blockers: 无 | next: review`。
  **验收对照**：AC-01✓（bp list+palette 零漂移）/AC-02✓（vue 臂 6 测）/
  AC-03✓（VM 臂 boot+投影+needle）/AC-04✓（五几何断言）/AC-05✓（双变体
  编译解释双臂+full 无侧栏形态 needle+组合消费）/AC-06✓（gate 全绿+台账行）。
  **验证范围声明**：本计划零 `crates/` Rust 改动（纯 blueprints 资产+
  examples gate+docs），按 AGENTS 分级门禁跑 scoped 验证（bp 域测试+bp-gate
  双臂），未跑 cargo tf 全量（无 Rust 代码面变更）。
- 2026-09-20 review（plan_revision 1，**与实施同会话——结论自工件重建，
  全部验证命令重跑，不采信执行摘要**）：
  `stage: review | PLAN-665 | r1 | pass | reviewed_commit=53ba8b346（worktree
  plan-665-dev tip，基点 172536658；f2a8a2ce5+7f1fddef7+53ba8b346 三提交，
  worktree clean） | deps=组内 auto-down@6a9df40 lang-665-dep（零改动）|
  spec_inputs=docs/specs/blueprint/contract.md+project.md@53ba8b346、
  docs/specs/widgets/viewport-boundary.md | acceptance: AC-01✓ AC-02✓ AC-03✓
  AC-04✓ AC-05✓ AC-06✓（全部复审期重跑） | findings: F-1（低/非阻断/预存
  文档漂移→P665-D7）、F-2（info：AC-01 字面 `auto bp check` 校验面为 L2/L3
  消费产物行为契约，对 fn-free 骨架不适用——status-bar/filetree 同理，括号
  内真实验收 registry 扫描+palette 零漂移已重跑绿；计划措辞不精确，无需行动）
  | evidence: ①auto bp list EXIT=0（layout/sandwich 在册）；②plan657 t03
  palette_drift 全包零漂移 PASS（重跑）；③bp 域 46 测 45 绿（唯一红=在案
  预存 P665-D4）；④bp-gate 双臂重跑全绿（vue 6/6 含 AC-04 五几何断言
  throw 路径在册非同义反复；VM boot+三投影等值+七 needle 全中）；⑤full.at
  结构 grep 0 侧栏元素+3 出口（AC-05 无侧栏列）；⑥SD-01 落档于 53ba8b346。
  债登记：P665-D4/D5/D6/D7 四条入 KNOWN-DEBT（复审期登记）。touched_goals
  空解释：本计划为 blueprint 层资产计划，不触及 docs/specs/goals.md 任何
  GOAL-NNN 条目 | next: merge（D-1 命名窗口随 merge 关闭，缺省=sandwich）`。

## 10. 待澄清事项

| ID | 事项 | 缺省路径（不阻断） |
| --- | --- | --- |
| D-1 | 包名 `sandwich` 为用户提案、AskUserQuestion 未获回复；**合并前**为改名窗口（改名成本=目录+use 路径，极低） | 按提案 `sandwich` 执行 |
| D-2 | ~~T-00 若揭示 VM 轨 use-bps×slot 组合缺口~~ **已裁定（T-00，2026-09-20）：零缺口**——双臂原生可用（vue SFC 四 outlet+targeting 全对；VM 渲染+snapshot v2(rendered) 子树全暴露），interpreter 修补/拆债/契约改形三路径均不触发 | 关闭 |
| D-3 | sidebar 宽度 w-56 现钉骨架内；宽度可配走 token/recipe 还是 props | 骨架内定值，消费方 fork/提升评审再议 |
| D-4 | **债候选（work 期发现，review 裁定入册）**：master 预存红 `plan640_bp_tests::t04_vm_track_empty_state_reference`——`Undefined symbol: on_primary in module EmptyStateFirstUse`（A/B 实证：主检出 172536658 同红，与 665 零交互）。疑 empty-state first_use 参数化（PLAN-657 族 on_* 回调）后 VM link 缺声明 | review 阶段登记 KNOWN-DEBT（归 empty-state/657 领地，非本计划修） |
| D-5 | **债候选（work 期发现）**：`docs/specs/blueprint/project.md` 模块清单表落后于磁盘——缺 075 三包行（status-bar/row-list/filetree）；磁盘 17 包 vs 表 15 行（本计划补 sandwich 后）。665 只补自属行不越权回填 | review 阶段裁定：一行回填或登记 |
| D-6 | **债候选（work 期发现）**：`scripts/spec-index.py` 再生会冲掉 INDEX.md 手插注记（664 ui 行实证——注记仅存 INDEX，未入再生源）。665 已外科恢复，最终 delta 干净 | review 阶段裁定：注记回灌再生源或改流程纪律 |

- 2026-09-20 merge 收据（`PLAN-665:r1`，五 checkpoint）：
  - **prepared**：reviewed 基线=review pass@53ba8b346（master 簿记 9c4067642）；
    规范增量=project.md SD-01（含于 53ba8b346）；投影目标=.autoos/specs.json
    P665-1（architecture）/P665-2（reviews）；delivery commit=9725d0c30
    （投影型后代：specs.json +28 行，实现/依赖零变化）。
  - **landed**：rebase plan-665-dev→master 后 range-diff 3/3 全等（映射
    f2a8a2ce5→8bc2f114c、7f1fddef7→cd3c2861d、53ba8b346→37d2ae802）；
    `git merge --ff-only` 零合并提交，master tip=9725d0c30=dev tip（SHA 同一）。
    wt-guard clean（worktree 内 bp-gate node_modules pnpm junction 六链先行
    摘除+node_modules 清除后过闸）。
  - **ledger_refreshed**：specs.json 两条 append（+28/-0，字节格式保真
    indent=1 无尾换行，读回验证过）；INDEX/project.md 随 37d2ae802 落地。
    落地后主检出冒烟：auto bp list 绿（layout/sandwich 在册）+ bp-gate
    双臂全绿（main 自身 exe+资产）。
  - **archived**：本文件 git mv 至 docs/plans/archive/665-bp-sandwich.md，
    status: archived，completion_kind: delivered。
  - **cleaned**：双 worktree 复跑 wt-guard clean + git clean（dev tip 9725d0c30
    已全并入 master 实证）；`git worktree remove` lang-665/{auto-lang,auto-down} +
    分支 plan-665-dev/lang-665-dep 删除；组目录 .wt/lang-665（含 scratch/ 探针
    工件,junction 先期已摘）移除；worktree 注册零残留。D-1 命名窗口随归档关闭
    （最终名=sandwich）。
