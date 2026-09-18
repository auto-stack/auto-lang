---
plan_id: PLAN-655
status: reviewed             # drafting → executing → execution_done → reviewed → archived
feature_name: equal-height-row
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1
current_step: 4
total_steps: 4

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-lang/ui/overview.md#items-stretch-两阶段行语义（PLAN-655）]
touched_goals: []

affects: [auto-lang/ui, auto-lang/ui-gen]
---

# [PLAN-655] equal-height-row

## 变更摘要

VM 渲染器的 `items-stretch` 当前用"子项包 height:Fill 容器"模拟 CSS 等高拉伸；
Fill 需要有界参照高度，在滚动容器（内容臂无界）下解析为垃圾值——008 定价卡
整列消失（PLAN-642 T-12/T-13 循环实证，P642-D12）。本计划新增 `StretchLine`
自定义 iced 控件：**两遍测量**（先按内容量子项高取 max，再以该行高约束重排），
把 CSS `align-items:stretch` 的 max(a,b,c) 等高语义做成渲染器原语。落地后
008 语料恢复原样（items-stretch 回归，双端 parity 恢复），并在 scroll 兜底
下安全渲染。即 P642-D12 处置裁定的"近期 = 路线 A"。

## 目标

- **G-1 等高语义原语化**：`items-stretch` 行在任意祖先上下文（含 scroll 的
  无界内容臂）下，子项高度 = max(子项内容高)（CSS 两阶段行算法），不再依赖
  Fill 对有界参照的假设。
- **G-2 008 双端 parity 恢复**：008 语料还原 items-stretch（撤销 T-12② 的
  自然高让渡），VM 内嵌与 Vue 臂回到同一语料语义。
- **非目标**：012-clock 的横向 items-stretch（列交叉轴=宽度，有界，现实现
  无害，不动）；041-auto-edit（回退页，顺带受益不单测）；iced 引擎级
  flex 补丁（P642-D12 远期，iced 升级时处理）；T-13 主题隔离（另案）。
- **受影响**：auto-lang（crates/auto-lang/src/ui/iced/ 新控件 + build_row
  集成 + layout_tests；语料 008 还原）；auto-os（ui-gallery 产物再生成）。

## 架构方案

- **StretchLine 自定义控件**（新文件 `crates/auto-lang/src/ui/iced/stretch_line.rs`）：
  包装 build_row 现有产物（含 Fill 拉伸容器的 iced Row element）。
  - **measure 遍**：以 `Limits::with_compression(min, max=(w,∞), compression=(false,true))`
    对内层 Row 调 layout——compression=true 使 Fill 子项解析为**内容高**
    （iced Scrollable 同款手法，009/016 实证），得行内容高 `h_line`。
  - **final 遍**：以 height=min=max=`effective` 重排内层 Row，
    `effective = incoming_max.height.is_finite() ? min(h_line, incoming) : h_line`
    （有界父上下文维持现行为零回归；无界/scroll 场景取内容行高）。
  - Node = (incoming 宽, effective)；draw/update/operate 等全部委托内层 Row。
- **build_row 集成**：`items_stretch` 命中时，现产物（Fill 拉伸容器形态）
  外包 `StretchLine`——Fill 容器保留（它们是"子项如何吸附行高"的载体），
  行高由 StretchLine 的两遍测量供给。非 stretch 行零接触。
- **语料还原**：008 app.at 恢复 `items-stretch`（T-12② 的移除撤销），
  教程/源码 tab 与网页臂回到原始语义。

## 需求分析与背景调查

- **授权记录**：用户 2026-09-18："我建议近期搞路线A。但是远期的路线B要
  记录下来，未来 iced 升级时一并处理（到时候可以考虑给 iced 发 PR）。"
  （路线 A = 自建等高行控件；路线 B 已登记 P642-D12）
- **实证链**（PLAN-642 T-12/T-13 循环）：iced_widget-0.14.2 flex.rs
  third-pass 门控 `if !main_compress`（压缩上下文 Fill 子项整体跳过排版 →
  008 卡片 0×0）；Scrollable 内容臂 compression=(false,true)（Fill→内容高
  正确，009/016 为证）；headless 真实 008 语料在 720 视口（无 scroll）
  渲染完美（探针③），live 装配（scroll 兜底）卡片消失——两遍测量控件
  补齐的正是"无界上下文下的行高计算"。
- **相关 Spec**：docs/specs/auto-lang/ui/overview.md（items-stretch 行为
  契约未成文，本计划补立）。

## 详细设计

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/overview.md#items-stretch-两阶段行语义（PLAN-655） | before: items-stretch=子项包 Fill 容器（依赖有界祖先，scroll 下塌缩）；after: items-stretch 行由 StretchLine 两遍测量供给行高=max(子项内容高)，任意祖先上下文成立 | P642-D12 近期处置；CSS align-items:stretch 语义对齐 | AC-01/02 |

## 测试设计

- **headless（主验收，iced_test 模拟器）**：
  - StretchLine 单元：stretch 行内 3 个内容高不等的子项（如 100/200/300）
    → 三者 bounds 高度一致 = 300（max 语义）；
  - scroll 兜底组合：overflow-hidden+justify-center frame（h-300）内挂
    stretch 行 → 子项完整布局且等高（008 消失场景的回归守卫）；
  - 双窗口尺寸守卫（1024×800 / 1920×1200）。
- **语料级**：真实 008 app.at（探针③形态）挂 720 视口 + frame scroll 装配
  → 卡片文本全出且三卡等高。
- **既有回归**：layout_tests 全模块（T-11 分布式列测试必须保持绿）。
- **实机 E2E**：画廊 008（卡片可见且等高，scroll 内可达）、009/016（不回归）、
  002 抽样；截图入 ui-gallery screenshots。
- **门禁**：`cargo t ui` 零新增红（对照 master 基线）；`cargo t -p auto-man`
  不涉（生成器不动——008 语料还原属撤销性变更）。

## 验收标准

- **AC-01**：headless：items-stretch 行内内容高不等的子项（100/200/300）
  渲染后三者高度一致 = max(300)，且在 overflow-hidden+定高 frame（scroll
  兜底）内同样成立。验证：layout_tests 新增断言（双窗口尺寸）。
- **AC-02**：真实 008 语料 headless：frame(scroll 兜底) 装配下卡片文本
  （Single Developer/$39/Buy Now/…）全部渲染、三卡等高。验证：探针③
  断言化。
- **AC-03**：008 语料还原 items-stretch 后，实机画廊 VM 内嵌卡片可见、
  等高、scroll 可达；009/016/002 不回归。验证：MCP 截图对照
  （t12final_008 的卡片缺失形态消除）。
- **AC-04**：`cargo t ui` 零新增红（对照 master 基线 22 败集合）；
  layout_tests 55+绿（2 红=master 预存）。

## 执行步骤

（原子任务：每步完成后追加 [✅ 已完成] 一行证据）

- [x] **T-01 StretchLine 控件**（新 `crates/auto-lang/src/ui/iced/stretch_line.rs`；
      mod.rs 注册）[AC-01]
      两遍测量（measure compression=(false,true) → final height=effective）；
      draw/update/operate/hash 委托内层 Row。验证：新增 headless 单元测试
      （100/200/300 等高 + scroll frame 组合 + 双窗口尺寸）绿。
      [✅ 已完成] 2f3aa65da。设计偏离（等效实现，AC 不变）：iced 0.14 flex 源码逐行
      核证伪「对 Fill 拉伸产物做 compression 度量」——cross-Fill 子项走 second-pass
      且 max 从 cross=0.0 起步（iced_core layout/flex.rs），测出 0 高；Scrollable 的
      compression 手法仅在压缩轴=子项 flex 主轴时成立。改为 StretchLine 自持布局、
      直接持有原始子项：宽度探测（flex first/third-pass 同式配给）→ 按各子项最终
      份额宽测高（compression=(false,true) 打在子项主轴——justify-between 垫片
      解析为内容高 0）→ final 落位（Shrink 高子项 min_h=effective 拉伸，Fixed 高
      自然钳制）。p655 三测绿（equal_height_max_semantics / in_overflow_frame_
      renders_equal / dual_window_sizes）。
- [x] **T-02 build_row 集成**（`items_stretch` 命中 → 产物包 StretchLine）
      [AC-01/02/04]
      验证：layout_tests 全模块绿（T-11 分布式列测试保持）；真实 008
      语料探针（frame scroll 装配）卡片全出且等高；`cargo t ui` 零新增红。
      [✅ 已完成] 2f3aa65da。stretch 臂产出 row([StretchLine]) 单子载体（apply_row_style
      管线零重复），非 stretch 行零接触。layout_tests 61 测：59 绿 + 2 红
      （desktop_surface_z_slot_window_covers_icons / p625_uigallery_sidebar_pills_visible
      —— master 复跑同败实证=预存基线）；p642_t11 分布式卡片测试保持绿。
- [x] **T-03 008 语料还原 items-stretch**（撤销 fe48a3945 的语料移除，
      保留 PLAN-642 注释改写为"等高由 StretchLine 供给"）[AC-03]
      验证：cargo t ui 语料测试绿；auto-os 产物再生成 diff 仅 008；
      实机画廊 008/009/016/002 MCP 截图验收。
      [✅ 已完成] 9d6cb15b9。探针③断言化 p655_real_008_corpus_cards_visible_and_equal_
      height（卡片文本全渲染非 0×0 + 三卡 CTA 等高 y±1.5px）4/4 绿。auto-os 产物
      再生成：干净 worktree + AUTO_GALLERY_APPS 解析序实跑，008 产物 items-stretch
      已传播（diff +359 行）；「diff 仅 008」预期修正——实际为语料漂移全量（P642
      后 637 等语料演进累计），008 hunk 内容正确，漂移同步属 auto-os 常规流程。
      实机 MCP：画廊内嵌 008 三卡可见等高（p655_gallery_008.png）、009/016/002
      不回归（画廊+独立 VM 双形态）。
- [x] **T-04 收尾门禁**：layout_tests + `cargo t ui`（--no-fail-fast 对照
      master 基线）+ 实机四页截图证据归档 [AC-03/04]
      [✅ 已完成] cargo t ui 5160 run：5128 绿 / 32 败——32 个失败名集合在 master
      复跑 0 passed（全部预存，基线自 22 漂移系其他计划落地）=零新增红。实机截图
      10 张归档 worktree `docs/reports/p655/`（独立 VM 008/009/016/002 + 画廊
      initial/008/008_scrolled/009/016 + 导航脚本 gallery_nav.py）。

## 复审记录

```yaml
stage: new
plan_id: PLAN-655
plan_revision: 1
outcome: pass
next: work（worktree D:/autostack/.wt/lang-655/auto-lang，branch plan-655-dev；
  依赖 PLAN-642 worktree 的 scroll 兜底形态——fe48a3945 已在 plan-642-dev，
  本计划基于其上续作或等 642 landing 后基于 master 续作，二选一在 work 时定）
```

```yaml
stage: work
plan_id: PLAN-655
plan_revision: 1
outcome: pass
code_commit: 2f3aa65da（T-01/T-02）+ 9d6cb15b9（T-03/T-04）——branch plan-655-dev
task_ids: [T-01, T-02, T-03, T-04]
evidence: >
  headless p655 4/4 绿（等高 max 语义 / overflow-hidden frame 组合 /
  双窗口尺寸 / 真实 008 语料断言化）；layout_tests 59 绿 + 2 红
  （desktop_surface_z_slot / p625 sentinel，master 复跑同败=预存）；
  cargo t ui 5160 run 5128 绿，32 败名集合 master 复跑 0 passed=零新增红；
  实机 MCP：独立 VM 008 三卡等高 CTA 对齐（p655_vm_008_initial.png）、
  009/016/002 不回归；画廊内嵌 008（auto-os 干净 wt + AUTO_GALLERY_APPS
  解析序臂）三卡可见等高、PageDown 前后全内容含 CTA 在 720 frame 内
  可见（折叠线下担忧证伪）；产物再生成 items-stretch 已传播（008 产物
  diff +359）。截图 10 张 + gallery_nav.py 归档 worktree docs/reports/p655/。
  设计偏离已在 T-01 记录（计划测量方案 iced 0.14 flex 源码核证伪，
  等效自持三段布局实现，AC 不变）；spec delta SD-01 已备 worktree
  docs/specs/auto-lang/ui/overview.md#items-stretch-两阶段行语义（PLAN-655）。
blockers: []
next: review（worktree D:/autostack/.wt/lang-655/auto-lang 与组内 auto-down
  兄弟 b1c88de detached 保留至 merge 清理）
```

```yaml
stage: review
plan_id: PLAN-655
plan_revision: 1
outcome: pass
reviewed_commit: 7044d67cf7da61936fa2e8d1d33404aaad799906（plan-655-dev，工作树
  clean 复验；worktree 实存经 git worktree list --porcelain 确认）
base_commit: 9886ba9018de3510e57a889be3ef7275d8fc40c8
dependency_revisions: auto-down 兄弟 wt b1c88def9bfa23f02397bff79e93890bb2e1120c
  （detached）；auto-os 临时取证 wt 64e2b2b（已 wt-guard 过闸移除，证据入库）
spec_inputs: docs/specs/auto-lang/ui/overview.md#items-stretch-两阶段行语义
  （PLAN-655）@ 7044d67cf（branch 内已提交=frozen 副本；P642 节第 6 条的
  items-stretch 让渡约定由本节第 4 条显式解除，无未声明冲突）
acceptance_results:
  AC-01: pass——复审基线新鲜重跑 p655 三单元绿（y 等差 ±1.5px 断言/可见性
    w,h>0/最短卡拉伸 spread>120px/overflow-hidden frame 组合/1024×800 与
    1920×1200 双尺寸）；命令 cargo nextest run -p auto-lang --lib
    --features ui-iced,iced-layout-tests -E 'test(p655) or test(layout_tests)'
    → 61 run 59 passed 2 failed（两红=master 同败预存，见下）。
  AC-02: pass——p655_real_008_corpus_cards_visible_and_equal_height 绿
    （真实语料卡片文本全渲染非 0×0 + Buy Now×2/Contact Us 三 CTA 等高）。
  AC-03: pass——语料 diff 复读（还原行+PLAN-655 注释）；入库截图复审
    （docs/reports/p655/p655_gallery_008.png 等 9 图+脚本）：画廊内嵌 008
    三卡等高 CTA 对齐、scrolled 形态全内容含 CTA 在 720 frame 内可见、
    009/016/002 双形态不回归；auto-os 产物 items-stretch 已传播（008 产物
    diff +359）。
  AC-04: pass——cargo t ui 5160 run 5128 绿/32 败，32 败名集合于 master
    复跑 0 passed=零新增红；复审加跑 cargo tf（仓规全档门禁）2568 绿/1 败
    （ui_gen::rust::tests::test_display_family_codegen_arm_fixture，master
    复跑同败=预存，与 654 landing 漂移相关非本计划面）；layout_tests 同 AC-01。
findings:
  F-R1(info,已闭合): 计划原测量方案（对 Fill 包装产物做 compression 度量）
    经 iced 0.14 flex 源码核证伪——cross-Fill 子项 second-pass max 从
    cross=0.0 起步；等效自持三段布局实现，语义契约 G-1/G-2 与全部 AC 未
    弱化，记录于 T-01 证据与 2f3aa65da 提交信息。
  F-R2(info): 「auto-os 产物再生成 diff 仅 008」预期不成立——实际为语料
    漂移全量（637 等计划演进累计，auto-os master 产物滞后）；008 hunk 正确，
    漂移同步属 auto-os 常规流程不属本计划授权面。
  F-R3(info,merge 注记): 执行期间 master 前进至 09deafde8（PLAN-654 stage A
    landing，touching ui_gen/vue.rs+api_gen.rs，与本分支文件不相交）——merge
    前需同步 master；主检出另有他session WIP examples/rust-workspace/Cargo.toml
    （已上报未触碰，不随本计划 landing）。
  遗漏/延后/workaround 扫描: 无——非 stretch 路径逐字保留、p625 sentinel
    （非 stretch 行 0×0 哨兵）保持、justify 垫片语义原样并入；无 dbg!/TODO/
    未处理告警（stretch_line.rs 零告警）；spec delta 无越权发布（live ledger
    未动，canonical 沉淀归 merge）。
  touched_goals 空集说明: 本计划目标以 G-1/G-2 叙述于计划正文，非 goals
    注册表条目，故 touched_goals=[]。
evidence: >
  本复审与实现同会话（独立性受限声明）——结论全部由工件重构：测试在
  reviewed_commit 新鲜重跑（p655/layout_tests 本回合、t ui 与 tf 于同一
  提交），失败归属以 master 复跑过滤集实证（t ui 32 集 0 passed；tf 1 败
  同名），截图自入库副本（docs/reports/p655/）复读，spec delta 以
  git diff 9886ba901..7044d67cf 独立复读。
next: merge（auto-plan-merge；worktree 与组内 auto-down 兄弟随 merge 清理；
  产物沉淀 SD-01 五条 + ledger 回写）
```

## 待澄清事项

- ~~008 实机"卡片在 scroll 折叠线下"假设未证伪~~ **已证伪（2026-09-18
  work 执行）**：画廊内嵌 720 frame 下全部卡片含 CTA 完整可见，PageDown
  前后 vtree/截图无变化（自动化近似；人工滚轮复验可选留 review 实机会话，
  风险已消）。
- 平板档（768×1024 frame）下 008 卡片行高语义同 desktop（stretch 与
  frame 高度无关，行高=内容 max），无需分档处理。
