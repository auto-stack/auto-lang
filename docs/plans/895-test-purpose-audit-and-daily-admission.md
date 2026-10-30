---
plan_id: PLAN-895
status: drafting
feature_name: 全仓测试用途审查、混合模块拆分与日常包准入建议
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30

plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/governance-and-inventory.md, docs/specs/auto-test-case/project.md, docs/specs/auto-test-case/plans.md]
new_spec_components: [docs/specs/auto-test-case/test-purpose-and-admission.md]
touched_goals: []

affects: [auto-test-case, test-infrastructure]
depends_on: [PLAN-888]
current_step: 0
total_steps: 12
---

# [PLAN-895] 全仓测试用途审查、混合模块拆分与日常包准入建议

## 0. 变更摘要

承接888已交付的唯一叶模块及资源记录，对语言仓全部测试入口和用例逐项审查用途，区分日常回归、专项验证、开发调研和待确认；将可独立执行但用途混合的受管模块继续拆开，交付逐ID、逐模块的日常包准入建议。**只提出准入/触发建议，不直接关闭、删除、ignore或改变默认选择**。用户审查建议后，再决定默认日常包调整。

全仓不等于机械接受888目录为完整分母：其7165义务/1181叶/39来源是起点，仍要查独立experimental工程、未注册声明和脚本入口是否遗漏。外仓UI/Man等只登记owner引用；不擅自执行或改动外仓。

## 1. 目标

- G1：全仓审查范围可复现，受管运行义务、聚合内部case/check、未注册声明、包装入口和外仓引用分开记，遗漏/重复有明确检查。
- G2：每个审查项有唯一用途建议、原ID、来源证据、保护的不变量或调研问题、owner、触发建议、成本及其时点/未知边界；不按名称、速度或所在目录批量猜用途。
- G3：保持原执行义务、oracle、profile、条件和消费者闭包，将可独立运行的混合用途叶拆成单一用途叶，原模块到新叶有完整迁移关系。
- G4：交付日常包逐项准入/不建议纳入/待确认建议及完整依赖闭包，无暗中关闭、默认包变更或研究探针随日常偷跑。

非目标：默认日常包自动启用/排除、删除/停用/改ignore、再生金样、减正式样本/压力规模/预算、修复四项旧红或13项ACE产品性能、实现第二套runner/裁判、全目录复测、全局P90/收益矩阵、改外仓测试。涉及产品性能的后继修复据原因另立计划。

## 2. 架构方案

复用888 `modules.json`/`impact_graph.py`/`partition_policy.py`/`select_tests.py`/`run_selected.py`/`cargo tm`，新增用途与准入**建议侧表**及其校验/报告；侧表不进入默认执行筛选。模块归属回答“保护哪个能力”，用途回答“为何及何时运行”，两轴不能混为一项。

拟新增（T-02验证后冻结，允许等价路径）：`scripts/test-governance/test-purpose-policy.json`及分片、`audit_test_purposes.py`、`daily-admission-proposal.json`、`scripts/tests/test_test_purpose_policy.py`。长期源文件分片遵守单文件5MiB门；原始扫描/运行输出本地保存，仅Markdown摘要入库。通过现有builder/partition验证消费分类拆分，不复制用例/比较器，不批量搬迁源码。

## 3. 技术栈

现有Rust注册/AST与运行时清单、AutoDown/TestCase、Python标准库、Cargo metadata、888模块选择和Job计量。源审查优先复用已有解析/身份工具；不为审查生成大型新测试框架。

## 4. 需求分析与背景调查

### 4.1 授权与实施边界

用户2026-10-30明确要求新建本计划，逐项区分四类用途、拆分混合用途模块、列日常准入建议，“先不要直接关闭或删除用例”。本轮授权是规划，不是执行895。计划保持drafting；未建895实施树、未执行测试或更改默认选择。另行授权的计划技能更新为本轮独立维护工作，不占本计划任务完成数。

主检出基面`129ef605763e59ca13bf312fd4a391eded10c13a`；888 r3已archived 10/10，delivery `5a6c23afacd3c40412c5561a5ba2ae322a119c2e`，权威旧树已清理。拟实施树`D:/autostack/.wt/lang-895/auto-lang`、分支`plan-895-dev`；work须核实际身份及最新源/依赖，不复制888旧树。主检出既有他方文档/号源WIP保留，不认领890/891/893/894。

### 4.2 已观察来源

| 来源 | 已有事实 | 本计划消费方式 |
|---|---|---|
| `docs/specs/overview.md` | 全仓/独立experimental/外仓归属地图，部分数量为旧快照 | 定位入口，不当现时分母 |
| `docs/specs/auto-test-case/{module-selection,governance-and-inventory,project}.md`、`AGENTS.md` | 唯一叶、完整模块与消费者并集、条件/原子守恒；准入不等于归属 | 保持执行合同，新增建议层 |
| `scripts/test-governance/{modules,recipes,atomic-contract,legacy-contract,legacy-wrapper-contract,partition-policy,periodic-module-schedule}.json` | 888发布7165义务/1181叶/39来源、原九语言配置4819身份、1066聚合关联及外仓周期引用 | T-01按当前注册重核，不修改冻结合同消除漏项 |
| `docs/plans/attachments/888/{module-inventory-r3,module-resource-observations-r3,daily-admission-review-proposal,ace-slow-tests-r3}.md` | 已有逐模块成本及用途审查建议，13项ACE慢例交接；4旧红可见 | 复用有效记录，快慢/PASS/FAIL不替代用途证据 |
| `docs/plans/893-ace-test-framework-and-slow-case-attribution.md` r2 drafting | ACE框架/13例归因与用途意见；不自动关闭/改调度 | ACE侧表与本计划逐ID对齐；不重复全13例压力测量，不以893整包交付作为全仓审查前置 |
| `scripts/test-governance/{module_catalog,build_modules,catalog_contract,impact_graph,partition_policy,run_selected}.py` | 现有真实选择、分区/身份/输入/消费者复核 | T-02先最小链验证，再接用途拆分 |

完整目录外入口尚未逐项盘点；全仓用途数量和混合叶数量均未知，不预先宣称分类完成。T-01负责消除分母缺口。

## 5. 详细设计

### 5.1 审查单位与全仓边界

从当前受管目录对账独立来源清单：root成员及独立workspace中的Rust注册测试/未注册声明、TestCase suite/case/check、`.at`语料/金样及其包装内原子、Python/PowerShell验证和语言仓内AutoGraphics等实验测试入口。排除构建产物、vendor/node_modules和纯报告；每个排除目录记依据，不能按experimental/probe一概排除。

包装命令映射到已有底层义务，不重复加分母；独立脚本若确有自己的断言则单列。未注册声明、平台不适用、手工/里程碑与外仓owner引用分别列，不标成已执行或自动接入。888目录外有价值的入口照样审查，记录当前接线缺口与owner建议，不以补目录之名启用原未注册项。

### 5.2 四类用途及逐项证据

| 分类建议 | 判断依据 | 建议触发 |
|---|---|---|
| 日常回归 | 长期产品/协议不变量、明确独立oracle、稳定可重放、依赖与成本可控 | 真实受影响模块及必要消费者 |
| 专项验证 | 长期有效的性能/规模/GC/故障/生命周期或环境要求 | 相关触面、人工或已批准周期 |
| 开发调研 | 一次性方案比较、架构试验、临时调试/剖析问题 | 明确手工调研入口及可保留结论 |
| 待确认 | 目的/归属/替代关系证据不足或冲突 | owner与下一项需核事实/决定 |

每行记录原运行ID/内部原子ID、注册状态、source/断言/既有Plan或Spec依据、保护的不变量/调研问题、用途建议及理由、owner、成本来源时点/未知、资源/环境要求、消费者、重复/替代覆盖、日常准入建议与待决定事项。每行均审源码及实际oracle；不能只按`probe`/`benchmark`/Plan编号/耗时分类，也不能把全部疑难项无据批量标“待确认”。待确认须逐项说明已查何处、缺什么和由谁决定。

### 5.3 混合用途拆分

以用途建议细分原叶，同时保owner/recipe/工具配置/profile/平台条件/实际依赖边界；不以功能分类替代用途。原子ID不变，旧叶→新叶后继并集唯一且完整，旧兼容映射/周期清单/消费者跟随同一分区；不得因不建议日常而从现行选择中漏掉。

先验证两个真实混合叶的最小链。若有共享session、producer→consumer或聚合内部不可独立调用，先识别原必须共跑单元，保持原顺序/oracle/状态。可安全拆包装时只改测试包装；不能独立的单元作为具名整体给专项/待确认建议及原因，禁止为了“拆完”抽掉其内部义务。候选日常模块的必要闭包含专项/调研时显式列出，未解决前标“有条件/待确认”，不得悄悄裁剪依赖。

### 5.4 日常包建议与成本

输出逐项/逐模块准入建议与四类数量、各能力覆盖、必要消费者、专项/调研触发建议及尚缺覆盖。建议清单与当前实际执行配置分离；默认包、旧别名、忽略条件、周期频率、注册、原oracle/采样/压力规模均不因建议改变。用户后续批准才可另行调整默认准入。

复用888有效成本，标明源/成员/条件是否相同；新拆的拟日常模块在准备完成后各单轮测墙钟/CPU/原生峰值，构建另计。已有特别长压力原子保13项ACE归属，不重做全13例或全目录三轮/收益矩阵。未知成本留空及补测建议，不把旧整组成本均摊为新叶实测。常规目标尽量≤60秒，不改门凑数。

### 规范增量

| ID | 操作 | docs/specs目标 | before→after / 原因 | AC |
|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/test-purpose-and-admission.md | 后续建议→四类用途/逐项证据/准入建议与默认配置分离 | AC-02/04 |
| SD-02 | modify | docs/specs/auto-test-case/module-selection.md | 唯一功能叶→兼容用途拆分、不可分依赖单元和日常闭包透明 | AC-03/05 |
| SD-03 | modify | docs/specs/auto-test-case/governance-and-inventory.md | 受管清单→全仓审查对账、未注册/包装/外仓分轴 | AC-01/02 |
| SD-04 | modify | docs/specs/auto-test-case/project.md；docs/specs/auto-test-case/plans.md | 后续用途审查→已验证的建议/限制及来源导航 | AC-06/08 |

## 6. 测试设计

| ID | 不变量/AC | 合法对照与反例 | 实际入口/观察 | owner |
|---|---|---|---|---|
| V-01 | 范围完整 AC-01 | 已注册/未注册/包装/内部case/实验入口；删一行或重复记wrapper | 独立来源对账+用途校验器；每项唯一映射，缺/重/不明排除失败 | work T-01/06 |
| V-02 | 证据分类 AC-02 | 同名probe有长期断言与一次性试验；快慢各有日常/专项；无据全pending | 逐项源码/oracle引用读回，缺理由/owner/待确认下一事实拒绝；review复查各类与边界 | work/review |
| V-03 | 拆分守恒 AC-03 | 两真实混合叶；删check/错profile/失ignore/丢消费者/拆散session | builder→原catalog守卫→select/verify→真实完整模块；旧新ID并集相等，合法共跑对照及故障敏感性保持 | work T-02/07 |
| V-04 | 建议不变执行 AC-04 | 改建议为research/reject；保原默认/启用 | 变更前后现行选择、legacy/周期、注册/ignore与原oracle哈希对比；建议不能自动过滤，负控能识别丢义务 | work T-08 |
| V-05 | 日常闭包 AC-04 | 日常叶依赖专项/调研、跨模块consumer | 建议报告显式列全部依赖及准入阻碍，不以过滤生成假闭包 | work T-07/08 |
| V-06 | 成本 AC-05 | 原相同叶复用、新成员叶无独立数据、压力原子 | measure_modules真实单轮或有效原收据；CPU/峰值范围明确，未知留空，不均摊、不三轮 | work T-09 |
| V-07 | 全链/健康 AC-06 | 源目录经Git发布重读、未知/陈旧建议、源名迁移 | 原强守卫及新负控；Git后身份可读，错误拒绝，格式/无新增触面warning | work T-10 |

不为分类审查跑全仓测试。开发/review只跑上述反例及受影响完整模块；merge在最终审查后冻结`cargo tm`完整模块/消费者并集，可复用有效记录；周期到期独立处理，不叠加旧t/tt/tv。角色未指定外部终审者，review必须独立重建事实并如实标上下文限制；不自动建agent或其他聊天。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 全仓来源与当前受管/未注册/包装/内部原子/外仓引用对账；每项有审查ID与去重映射，遗漏/重复0；不是只复印888清单 |
| AC-02 | work | 逐项四类建议/来源断言/理由/owner/成本与未知边界完整；各类和待确认统计可重建；无靠名字/快慢或批量pending代审 |
| AC-03 | work | 可独立的混合用途叶完成拆分，原子/条件/原oracle并集不变；两个真实最小链与反控通过，消费者和旧叶迁移完整，不可分单元具名且原验证保持 |
| AC-04 | work | 日常准入/不建议/有条件或待确认逐ID可审；闭包冲突可见；无删除/关闭/新增ignore、默认过滤、频率/注册/原门变化 |
| AC-05 | work | 新拆拟日常叶单轮成本真实或明确不适用，原证据有效复用；未知不冒绿，特别长原子仍交owner，不为凑一分钟削负载 |
| AC-06 | work | V-01..07及SD-01..04对应源码/报告齐，归属893与外仓无重复施工；格式健康、提交干净可送审 |
| AC-07 | review | 同权威树冻结SHA审AC-01..06/SD/V及用途取舍和未授权改变，结论可复现；merge门pending明确 |
| AC-08 | merge | 冻结模块并集门/有效复用、实际合入、canonical来源/归档/guard清理齐；建议交人审，不冒默认包已批准 |

## 8. 执行步骤

12项，10 work+review+merge，当前0/12。全链实施尚未授权。

| 状态 | ID | 阶段 | 依赖 | 路径/产物与关闭条件 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 用户执行授权 | 核888实际交付/当前目录/来源，静态盘点全仓入口差额、未注册与包装映射；基面及排除依据记录，V-01 | AC-01 |
| [ ] | T-02 | work | T-01 | 新用途侧表/校验器最小链，两个真实混合叶、同名异用途/依赖不可分反控；冻结分类及拆分消费形式后扩面 | AC-02/03 |
| [ ] | T-03 | work | T-02 | language配方/原包装内部输入：语言、VM/编译/转译/书/HTTP等逐项源码断言与用途建议 | AC-01/02 |
| [ ] | T-04 | work | T-02 | 十ACE workspace及language内ACE harness逐项审查，复用893逐ID来源、13慢例用途意见及未知边界 | AC-01/02 |
| [ ] | T-05 | work | T-02 | 正式TestCase、Python/PS独立断言、目录外experimental/AG入口、未注册与外仓引用逐项分类/去重 | AC-01/02 |
| [ ] | T-06 | work | T-03/04/05 | 汇总分片建议及全仓数量/漏重检查，逐个待确认附owner/已查来源/下一事实；V-01/02 | AC-01/02 |
| [ ] | T-07 | work | T-06 | 原build_modules/partition_policy/catalog_contract/impact接用途拆分及旧叶迁移，保原共跑单元/消费者，V-03/05 | AC-03/04 |
| [ ] | T-08 | work | T-07 | 生成daily-admission-proposal建议及覆盖/依赖冲突、专项/调研入口建议；前后实际启用/默认选择不变，V-04/05 | AC-04 |
| [ ] | T-09 | work | T-07/08 | 复用有效888成本，仅新拆拟日常模块单轮Job计量；长例不重矩阵、未知不冒绿，V-06 | AC-05 |
| [ ] | T-10 | work | T-09 | 最终全仓对账/用途证据/SD/健康/发布控制与当前源码冻结，V-07；提交并给execution_done交接 | AC-06 |
| [ ] | T-11 | review | T-10 | 冻结SHA逐项AC/SD、各用途和混合/不可分/待确认边界独立重建，不跑整档；合入门pending | AC-07 |
| [ ] | T-12 | merge | T-11 pass | 冻结tm模块并集/有效证据、合入、规范/源回读、归档与fresh guard清理；日常建议交人 | AC-08 |

## 9. 复审记录

2026-10-30 / stage: new / plan_id: PLAN-895 / plan_revision: r1 / outcome: pass（规划，非实施） / authorization: 用户要求新建全仓四类用途审查、混合叶继续拆与日常准入建议；不直接关闭/删除 / grounding: 888归档10/10与实际模块工具/规范，893 r2职责边界；完整分母由T-01独立盘点，不假装7165就是全仓全部入口 / preflight: 12任务/8AC/7V/4SD及依赖、work/review/merge职责分明，默认与原义务保持、未知待确认逐项可核，现有门为tm而非旧档叠跑 / next: 用户审阅本合同后work；首任务T-01源/目录及全仓入口对账，拟唯一树lang-895/auto-lang，未创建树或运行测试。

## 10. 待澄清事项

- 当前无阻止起草的范围问题。用户尚未要求执行895；本次只交付计划。
- 全仓审查以语言仓为实施范围，外仓只列owner/引用；如后续希望实际审改UI/Man等仓全部用例，需另外明确仓库范围。
- 893用途意见如未交付，按现有源码独立审查并保关联，缺证项逐个待确认，不等待全13项重测。
- 日常默认包的变更、任何停用/删除/ignore/样本或周期调整均留待人审建议后另行批准；本计划不能替人批准。
