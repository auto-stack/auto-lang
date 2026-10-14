---
plan_id: PLAN-781
status: drafting
feature_name: scroll-child-anchor
title: "普通滚动内容的精确子锚点定位"
author: [Codex]
created_at: 2026-10-14T00:00:00Z
updated_at: 2026-10-14T00:00:00Z
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components:
  - docs/specs/widgets/scroll-pane.md
  - docs/specs/auto-lang/ui/overview.md
new_spec_components:
  - docs/specs/auto-lang/ui/design/scroll-child-anchor.md
touched_goals: [GOAL-007]
affects: [auto-lang/ui, auto-lang/vm, widgets]
depends_on: []
---

# [PLAN-781] 普通滚动内容的精确子锚点定位

## 0. 变更摘要

为 READER-001 T-25/F-12 提供真实布局锚点滚动入口，替代字符占比估算所依赖的框架缺口。Vue 和 VM/iced 必须自主把不等高目标段滚入 pane；定位请求入队不等于已经可见。本计划交付框架能力与双端 capability fixture，Reader 业务接入回到001。

## 1. 目标

- ordinary scroll-pane 能以稳定子内容 key 寻址，支持 col/row 包装、过滤空行和重新排版；nearest 对齐，保持已有 controller/offset API。
- 完成真实混排内容、首帧延迟布局、失效与换绑的验证；不给缺失目标假成功。
- 非目标：virtual-list/managed ScrollContent/terminal 锚点、跨页面持久化协议、滚动动画、a2r默认后端修复、全UI模块拆仓。

## 2. 架构方案

复用 ui/scroll 的 state/intent/controller 和 Vue/iced adapter。拟入口 scroll_to_child(handle, anchor_key)，子节点声明 scroll-anchor key；这是候选拼写，T-01 必须按现有 AURA/DSL/native contract 冻结等价可实现接口。禁止核心依赖 DOM/iced 类型、按字符比例计算目标，或由测试补滚动。

锚点作用域是所属 ordinary pane，按内容树的指定子节点而非 pane 直接第几个孩子定位。Reader 的段落 idx 是原始行号，显示列表序号不得直接等同；业务适配指南必须解释两者映射。

## 3. 技术栈

Rust 核心/iced adapter、ui_gen/vue.rs 生成的 TypeScript helper、AutoUI .at capability fixture；原生 oracle 复用 MCP，Vue oracle 复用 Playwright。代码修改只在本计划专用工作树。

## 4. 需求分析与背景调查

授权：用户2026-10-14要求“请帮我在auto-lang立项”，此前已决定保留001原目标。本轮只起草，不自动执行/启动其他agent，不修改 Reader 业务代码或 canonical Specs。

依据 [共同设计](../design/autoui/reader-runtime-prerequisites.md)、[立项输入/hash](reports/reader-prerequisites-planning-20261014.md)、[scroll-pane Spec](../specs/widgets/scroll-pane.md)。取材 auto-lang master @c171c1a09；Read-only Reader @29b5decf878ec2cdca5c6a39575e872a4597121d，001 r7。源 versions/hash 在立项输入冻结，执行前重核真实 SHA。

已核入口：ui/scroll/intent.rs::ScrollIntent、controller.rs::drain_resolved_intents；ui/iced/renderer.rs::build_scrollable 与 scroll_state_reader.rs；ui_gen/vue.rs controller helper；vm/native_catalog.rs 9900–9905；docs/schema/aura.at。尚无 child-anchor API。controller 已有未布局回队及2026-10-11抖动修复，不能回退。

依赖协调：PLAN-778/779 的UI/native生产接线与本计划可能共触 vm/native*/ui_gen；按 Design37 的单写入纪律重核实际落地合同。在途方案不能当已存在API。PLAN-783可供原生几何 oracle，但不设环依赖：现有MCP若足够可直接验收，否则 work 停在该 required oracle，待783落地后继续。

## 5. 详细设计

### 行为合同

1. key 为非空字符串，pane 内唯一；同名key在不同pane不串线。布局包装不改变显式锚点归属；重复/缺失/非法目标有可观察失败，目标未解析不报告完成。
2. 请求首次可在布局前发出；等待当前绑定的有效布局后消费。状态区分 pending/applied/invalid 或等价可观察结果；确切返回/查询/诊断形状由T-01冻结。不得把bool入队回执当可见证明。
3. nearest：目标已与pane有效裁剪视口相交则不动，否则按实际布局偏移使其正面积可见；段落高于视口时其起始区域可见。禁止只用 ScrollState progress 证明目标位置。
4. 目标以当前内容/绑定代际解析；销毁、换章、换绑或目标移除时旧请求不落到新内容。完成后用户滚动/重新构建不被旧intent持续拉回。字体、宽度改变后新请求采用新布局。
5. 框架 public helper 与native类型/arity/装配保持同一规则，新增ID分配先核目录；新API在不支持的managed content上明确报告unsupported，既有offset API继续可用。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/widgets/scroll-pane.md | Anchor未落地 → ordinary子内容锚点、nearest、代际与pending/失败规则；managed明确不支持新API | 关闭真实consumer缺口 | AC-01/02/03 |
| SD-02 | add | docs/specs/auto-lang/ui/design/scroll-child-anchor.md | 无公共合同 → 双端接口、寻址作用域、布局/裁剪/完成oracle与兼容边界 | 防止估算或入队冒充完成 | AC-01/02/03 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | 无该能力入口 → 实际实现路径、capability fixture和Reader承接说明 | 可导航与复用 | AC-04 |

## 6. 测试设计

角色：未指定分离模型或agent；实施者交已提交代码，普通独立验收review为final，同一计划树暂停实现写入后复审；不得自审代替用户指定独立角色。所有下表病例 owner_stage=work，由T-02~05使拟测试可执行；当前没有新功能测试证据。

| Case | AC/不变量 | 输入与入口 | 可区分错误实现的oracle | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/真实定位 | 4000字长段+100短段，目标中部；fixture产品handler调用新API | Vue/原生 target rect与pane及祖先clip的交集宽高均>0，恢复后驱动零scroll | 中文/ASCII各一组 |
| V-02 | AC-01/寻址 | 空行被过滤、col包装、段key=原始行号 | 显示文本/key对应真正段落，不能滚到同序号别段 | ordinary y-pane |
| V-03 | AC-02/时序 | Init先发目标、布局后可见；换章/解绑后再布局 | 无零offset假完成、旧代际不消费、pending有终态 | 首挂载/换绑 |
| V-04 | AC-02/负例 | 空key、缺key、同pane重复、其他pane同key、越界等价目标 | 明确错误/invalid，无跨pane滚动，无成功完成回执 | Native与Vue均覆盖 |
| V-05 | AC-01/重排 | small/large、窄/宽；已可见段及长于pane段 | 当前布局交集；已可见不跳动；过高目标起始区域可见 | 代表性pairwise组合，不穷举全笛卡尔积 |
| V-06 | AC-03/兼容 | 旧offset/by/start/end，用户滚动后重建 | 原语回归通过，新intent不回拉；managed新APIunsupported | 既有state/host fixture |

核心及controller scoped tests后，worktree门禁= `cargo check -p auto-lang`、裸 `cargo t`、`cargo tv`（native装配）、`cargo tu`（UI codegen）。`cargo tt`仅实际触及trans映射才增加；不碰aavm不跑taa。tf只属merge后的到期批量档。T-05记录新的确切双端驱动命令，沿用现有test_vue_playwright.mjs/test_vm_mcp.py设施，真实原生窗口不能用headless模拟替代。

## 7. 验收标准

- [ ] AC-01: owner_stage=work；双端新API按真实布局使V-01/02/05目标可见，无字符占比和测试侧补滚动。
- [ ] AC-02: owner_stage=work；V-03/04的pending、失败、绑定代际与作用域满足§5，不产生假完成。
- [ ] AC-03: owner_stage=work；V-06及旧controller API/回声/滚动位保持不回归，managed边界明确。
- [ ] AC-04: owner_stage=work；交付真实CLI文件hash、接口/fixture命令、Spec候选与Reader接入映射；明确001尚需业务接入和复审。
- [ ] AC-05: owner_stage=review；对固定commit/依赖重跑全部required oracle并审查SD-01~03；缺原生证据不得final pass。
- [ ] AC-06: owner_stage=merge；实际规范沉淀、归档、守卫清理与收据完成，给001可定位的依赖commit；不代替001验收。

## 8. 执行步骤

计划簿记留主检出；拟实现树 D:/autostack/.wt/lang-781/auto-lang，分支 plan-781-dev。work确认真实路径/仓/branch/base SHA，按脚本建组补auto-down；不创建junction/symlink。当前没有创建实现树。

- [ ] T-01: owner_stage=work；依赖无；按§4入口+共同设计核对AURA/DSL、普通布局的几何/锚点及在途native热点；形成docs/plans/reports/781-interface-decision.md（新），冻结接口/返回状态/代际/nearest与支持边界。若不可等价实现，先报告needs_replan；不压低可见门。AC-01/02/03。
- [ ] T-02: owner_stage=work；依赖T-01；修改ui/scroll/{intent,controller}.rs与native_catalog/对应shim、必要AURA绑定；增加scope/布局时序/失败永久单测。`cargo t scroll`及新族过滤测试通过。AC-02/03。
- [ ] T-03: owner_stage=work；依赖T-02；在ui/iced/renderer.rs及必要layout operation（新文件明确登记）按真实目标布局消费意图，保留ScrollStateReader回声；原生fixture验V-01~06。AC-01/02/03。
- [ ] T-04: owner_stage=work；依赖T-02；ui_gen/vue.rs生成绑定/helper，必要schema修改与golden同步；真实plain及Reader实际使用样式路径验证，无DOM类型泄漏。AC-01/02/03。
- [ ] T-05: owner_stage=work；依赖T-03/04；新增examples/capability-tests/p781-scroll-child-anchor/与配套双端驱动（新）；完成全矩阵、分级门、负例失败传播、当前帧几何/截图/CLI hash。AC-01/02/03。
- [ ] T-06: owner_stage=work；依赖T-05；准备SD-01~03候选、接口文档、原始行号→anchor映射与依赖收据；冻结代码/工具/测试配置，work入execution_done。AC-04。
- [ ] T-07: owner_stage=review；依赖T-06；独立final review、重选不等高/失效/回声反例及Spec delta；全部required通过才reviewed。AC-05。
- [ ] T-08: owner_stage=merge；依赖T-07；按merge技能实际沉淀/归档/守卫清理与到期批量门，留下Reader前置交付commit。AC-06。

## 9. 复审记录

2026-10-14：stage=new | plan_id=PLAN-781 | plan_revision=1 | outcome=pass（可评审的立项合同，非实现PASS）| baseline/inputs=立项证据 | changed_tasks=T-01~08 | acceptance=AC-01~06 | next=用户确认执行范围后work；首项T-01冻结接口，当前保持drafting。计数0/8，不勾任何功能/复审/合并项。

## 10. 待澄清事项

没有必须由用户先决定的目标缺口。候选API拼写、布局collector可用性和原生freshness由T-01有界核查，作为后续接线的硬前置；不把研究标完成而跳过决策。与778/779共享写面由实施入场对账，若未释放则记录具名owner和停止的任务。Reader接入归001；框架pass不关闭001 T-25。
