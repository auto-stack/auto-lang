---
plan_id: PLAN-842
status: drafting
feature_name: 旧兼容入口与旧发行路径退役
author: [Codex]
created_at: 2026-10-24
updated_at: 2026-10-24
plan_revision: 1
current_step: 0
total_steps: 12
supersedes_spec_components: [docs/specs/auto-lang/project.md, docs/specs/language-sdk/project.md, docs/specs/auto-parser-api/project.md, docs/specs/auto-ui-dialect/project.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md, docs/specs/auto-man/project.md, docs/specs/auto-cli/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/repository-boundaries/compatibility-retirement.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/compat, language-sdk, auto-ui, auto-man, repository-boundaries]
depends_on: [PLAN-831, PLAN-832, PLAN-841]
---

# [PLAN-842] 旧兼容入口与旧发行路径退役

> **全Plan启动锁G842**：G841且U841。包含T-01盘点/probe/度量、fixture/测试和建树，门未齐一律不开始；只核验上游状态/完整交付收据。启动证据定义见[共用合同](reports/m2-governance-order-841-844.md)§1。本轮仅规划。

## 0. 变更摘要

完成R11：按841终态和三仓实际消费者，将旧公开路径/组合门面、临时AST适配、旧feature转发/隐式装配与历史发行入口中确认可退出的过渡项实际退役；先迁消费者，再删桥，不以“标deprecated”替代删除。

本Plan是4份大模块合同之一，包含实际生产改动、旧实现退出、测试/三仓消费/版本与发行同步，不另按小探针取号。不接手831/832尚未完成的搬仓验收，不扩大为全语言IR替换、新backend/功能或远端发布。

## 1. 目标

- 三仓受管生产/生成器/安装/CI与已知外部fixture全部使用新公开入口/固定包源，无旧路径偷偷回落。
- 逐入口退役/保留有冻结合同依据、替代与版本迁移、consumer及测试，零未批准延期；长期稳定API不能被误删。
- 完整与Core profile默认行为、装配优先/线程/退出保持，历史兼容版本与用户产物有明确升级/拒绝/恢复路径。

## 2. 架构方案

T-01逐入口冻结retirement-matrix：旧符号/路径/feature/默认机制/源码或数据版本→替代入口→全部已知消费者→仍支持的承诺→可退役版本与控制。分类为应退出过渡项、须迁后退出、按现行明确承诺长期稳定的接口；后者有具体Spec依据和真实消费者，不得把所有旧项一律列保留而交空计划。无法履行长期公开兼容时给具体needs_replan，不静默破坏承诺，也不按日历假定用户已迁移。

先迁三仓生产、样例/测试/生成模板/安装/CI/独立fixture，再在同Plan删除过渡门面、关闭旧路径与废feature/隐式根回落。不能用新的同功能大兼容包替代旧壳，也不能只删测试让旧路径“无人用”。确有长期稳定薄API保留时不含旧实现/状态或隐藏反依赖，精确说明其是正式API而非过渡债。

841新数据/模型的过渡适配可退出时同步迁移消费者；旧已交付二进制/资源清单/缓存/锁/producer不得被新版本误读成功。发布版本/弃用说明/迁移入口与rollback/安装保护必须真实可验，原legacy refs/恢复载荷不删除。协议仍需支持的旧版本不得因目录叫legacy而清掉。

装配从旧隐式例外迁到明确拥有的组合入口，只退役已无必要的inventory兜底/全局hook，仍具正式宿主语义的能力保留并验证default/显式/缺席/多实例优先级；不更换进程协议、后端或合法源码。

## 3. 技术栈

现有Rust/Cargo与三仓固定Git版本、单源AST/值/词法/Parser API/SDK、UI文法/生成器/runtime及Man CLI，复用777边界、792发行与已有真实consumer/双端设施；不新增通用框架/平行runner。实际包名/路径以启动后上游来源映射冻结，当前源码仅观察基线。

## 4. 需求分析与背景调查

用户2026-10-24明确要求按4份较大计划规划交付目标、验收与顺序/831832启动前置；继承此前保守排程，全部实施任务严格等完整前置交付。**本轮只在语言主检出建立合同/设计/路线，不改实现或canonical Specs/ledger、不建树/agent、不写旧仓/远端**。已有授权只覆盖目标与规划，未来work按明确执行指令和门启动；常规机制选择不需重复批准，改变既有支持/兼容承诺则具体needs_replan。

从[Spec入口](../specs/overview.md)定位，[AST](../specs/auto-ast/project.md)、[词法](../specs/auto-lex/project.md)、[Parser API](../specs/auto-parser-api/project.md)、[SDK](../specs/language-sdk/project.md)、[前端](../specs/auto-lang/frontend/overview.md)、[VM](../specs/auto-lang/vm/overview.md)、[旧仓](../specs/repository-boundaries/project.md)与[发行](../specs/repository-boundaries/generated-ui-distribution.md)为当前依据。部分旧Spec数量/路径滞后，不以旧聚合描述覆盖实际模块代码/交付；观察与假设、源SHA/hash见[输入](attachments/m2-governance-batch-20261024/inputs.json)。

已观察写面：旧auto-lang/lib.rs再导出/包装与Cargo features、SDK UiGrammarSource/inventory登记与宿主hook、UI/Man旧facade、生成import/模板/producer/版本发现和scripts/CI；841发布过渡adapter清单为执行时权威输入，当前仅观察814/816形成的旧面。

当前831 executing 3/12、832 drafting 0/12；都未完成M1交付。新机制/真正可删候选/未来路径不声称已证明，T-01在G842后承担有界决策和实际编译/消费probe，关闭条件见§2/8。

## 5. 详细设计

### 5.1 启动、所有权与执行入口

G842对T-01..12逐行生效，depends_on是全Plan启动门，不是只约束最终集成。U定义为全部AC/final review/冻结门/实际合入/Spec与受管ledger/来源交接/归档收尾闭环；阶段pass不放行。门未齐只读上游状态/交付收据，保持drafting 0/12，不提前盘点/扫描/probe/改fixture/建树或启动实施agent；无新后端status/调度功能。

G842后T-01读取所有已发布三仓规范/agent指令、前置delivery commit/路径映射，新增attachments/842/contract-baseline.md：具体producer→数据/状态owner→consumer/产物、支持/预算/错误/退出规则、正反例与可执行oracle、未知决策/owner/关闭条件。假设必须在依赖实现前用最小真实整链probe证明；无解则保范围转needs_replan，不用单元假绿偷降AC。

### 5.2 三仓生产接线与来源

按共用合同§2–4及§2本Plan专门设计实施。所有类型/状态/算法单一owner；normal/build/dev与feature图无UI/Man反向污染SDK，不靠宽化777或临时[patch]双source过门。变动同时更新真实三仓consumer、固定版本、测试ID/alias/CI、build输入/生成import/资源/producer与缓存身份；832的旧发行失败合同继承，不新造弱事务。

版本升级/兼容差额记录在来源清单；外部正常消费与具名错误/缺件/取消/退出必须实际验证，未测环境pending。正式API或序列化不兼容仅按本Plan明确版本迁移合同实施，需改变既有承诺则停止该项给具体决策，不把用户规划授权扩张为随意删除稳定API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/project.md, docs/specs/language-sdk/project.md | 过渡根再导出/包装与SDK旧hook→已迁canonical入口和具名保留的稳定API | 退出过渡耦合 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-parser-api/project.md, docs/specs/auto-ui-dialect/project.md | Parser/UI文法旧桥→841终态入口，已无消费者过渡适配实际删除 | 不再背旧模型 | AC-01..05 |
| SD-03 | modify | docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md | UI compiler/runtime旧入口/feature/隐式装配→明确正式组合与源消费 | 保持默认与生命周期 | AC-01..05 |
| SD-04 | modify | docs/specs/auto-man/project.md, docs/specs/auto-cli/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md | Man/CLI/发行旧路径/安装/producer→固定版本canonical消费与迁移或拒绝 | 保真实命令/升级 | AC-01..05 |
| SD-05 | add | docs/specs/repository-boundaries/compatibility-retirement.md | 无统一逐入口退役规格→可退出清单、正式保留、版本迁移与历史恢复 | R11可审退役 | AC-01..05 |

## 6. 测试设计

下列Case均owner_stage=work，由T-01冻结实际输入/入口与oracle，T-02..08逐步建立永久定向控制并补真实集成；**本轮尚未执行**。原ast-model/ui-grammar/language-sdk/generated-ui/tool-client等consumer与语料/投影golden按真实owner复用，不以重写golden遮回归。新增fixture用`test/fixtures/modularity/`下具名目录，具体接口T-01冻结；新接口未存在前不把未来命令当已执行。

| Case | AC / invariant | 输入/真实入口 | 可观察oracle与覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 | 逐入口旧API消费者与新版替代；合法长期稳定旧API对照 | 已退役入口依冻结策略具名拒绝/编译失败并有替代，受支持旧API仍编译执行；不能以allowlist全留或空门面过关 |
| V-02 | AC-02 | 真实安装CLI/生成项目与外部工程，清AUTO根/兄弟路径，注入只剩旧source/旧模板 | 新路径真实成功、错误根无静默回落，metadata/tree单一来源；受管生成物零未解释旧入口 |
| V-03 | AC-03 | default/显式Core/UI/缺席，provider覆盖/多实例/线程亲和，创建/退出 | 现行支持保持，无双初始化/丢装配/状态复制；缺能力先拒绝副作用，不偷偷载入完整UI |
| V-04 | AC-04 | 上版合法工程/缓存/锁/安装→新版迁移、未知版本/坏producer、升级中断/回退 | 约定可迁则结果保持，不可用则具名拒绝/指引；旧可用产物保留或按合同恢复，不假completed |
| V-05 | AC-05 | retirement-matrix全行与测试选择/CLI help/文档样例，原legacy恢复锚 | 每行有实际来源/消费者/控制与结果，测试仍被适用档选中；旧历史可恢复，代码退役不等于删版本历史 |

work/review只必要check（真实SDK/被改包）、相关包/模块/fixture和生产反例、实际外部工程构建运行；777 strict与792适用面/缺源负控不能省。类型/VM/trans/UI生成触面final review通过后，merge在冻结三仓输入统一裸cargo t及tv/tt/tu等适用档（按迁后alias/owner等价入口），tf到期语言主检出单实例并覆盖迁出家族。修复先完成本轮全部findings和反例/邻接组合再提交送审；无单finding全档/边改边审。缺环境/上游不能fake正控，当前无指定agent/model，不创建会话；本轮Category A无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | V-01 冻结过渡项全部迁移/实际退出，合法稳定API按承诺保持；零无依据保留或延期。 |
| AC-02 | work | V-02 真实三仓/生成器/CLI/样例/CI及外部consumer只用canonical入口/source，无隐式旧路径fallback。 |
| AC-03 | work | V-03 默认/显式/缺席组合与实例/线程/退出保持，退役旧装配不形成双注册或不可用默认。 |
| AC-04 | work | V-04 新旧模型/协议/包源/producer升级迁移或拒绝与安装回退可复跑，旧已发布历史可恢复。 |
| AC-05 | work | V-05 测试ID/alias与规范、版本/迁移说明、consumer来源与退役差额逐项完整。 |
| AC-06 | review | 独立最终review绑定r1/实际三仓commit及依赖/配置/输入，全AC/SD/遗漏/警告/格式/debug审查通过；merge门pending |
| AC-07 | merge | 并集合流影响复审/冻结门通过，实际源码与consumer版本交付、canonical/受管ledger/来源读回、归档收尾与guard清理闭环；不以部分发布放行后继 |

## 8. 执行步骤

12项，9 work+1 review+2 merge；全Plan启动门覆盖所有行，保守串行，不提前做独立小任务。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | G842完整核验 | 冻结逐入口/版本/消费者/支持承诺退役矩阵与实际API probe；无可退出范围或冲突先重规划 | AC-01..05 |
| [ ] | T-02 | work | G842；T-01 | 迁三仓first-party与外部fixture到841终态公开API和数据，保旧合法程序结果 | AC-01,02 |
| [ ] | T-03 | work | G842；T-02 | 迁生成import/安装/模板/示例/脚本/CI与固定源，构建实际外部工程验证canonical消费 | AC-02 |
| [ ] | T-04 | work | G842；T-03 | 删除已迁过渡facade/AST桥，恢复最小稳定API；不得新增大compat替身，冻结移除符号清单 | AC-01,02 |
| [ ] | T-05 | work | G842；T-04 | 迁移数据/缓存/协议/包源版本，具名拒绝不兼容输入与真实升级/恢复路径 | AC-04 |
| [ ] | T-06 | work | G842；T-05 | 清旧feature/不再必要inventory兜底/隐式根回落，保正式宿主端口，验证默认/显式/多实例/退出 | AC-03 |
| [ ] | T-07 | work | G842；T-06 | 完整三仓Core/UI安装和真实consumer、V-01..04/资源失败/迁移反例执行 | AC-01..04 |
| [ ] | T-08 | work | G842；T-07 | 退役矩阵逐行核销、旧引用/测试ID/alias/锁/CLI文档/777与792审计，零未批准保留 | AC-01,02,05 |
| [ ] | T-09 | work | G842；T-08 | 版本迁移说明、精确SD-01..05与attachments/842/handoff.md冻结，交843稳定出口清单 | AC-05 |
| [ ] | T-10 | review | G842；T-09冻结提交 | 同组独立完整AC/SD/实际consumer/反例/来源与健康审查；不先跑整档，不将merge门当已通过 | AC-06 |
| [ ] | T-11 | merge | G842；T-10 pass | 并集影响复审、冻结三仓适用门与有序实际合入/固定版本/来源读回；若改变输入重审影响面，暂不放行后继 | AC-07 |
| [ ] | T-12 | merge | G842；T-11全部交付收据 | canonical/受管ledger/索引沉淀、归档与逐树fresh wt-guard clean清理，整Plan闭环构成U842才放行下一份；保历史/恢复/交付分支 | AC-07 |

G842未满足前禁止创建/认领以下组、分支或work agent。未来组`D:/autostack/.wt/lang-842/{auto-lang,auto-ui,auto-man}`，每个实际需写仓各一树/分支`codex/plan-842-dev`；只读consumer用固定版本clone而非借其他计划WIP，未需写仓不多建树。work/review/merge沿同组顺序交接，不建第二审查树；若auto-down为现有可选依赖则具名固定版本/detached兄弟，只解析不改实现。禁止junction/symlink，主检出只簿记，清理前每树fresh guard clean且保留实际交付/legacy恢复源。

## 9. 复审记录

2026-10-24 / stage: new / plan_id: PLAN-842 / revision: 1 / outcome: blocked（上游整Plan启动门未齐；文档合同预检通过）/ next: 等G842真实满足后读回规范/交付/归档版本，再work T-01 / changed: T-01..12、AC-01..07、SD-01..05、V-01..05。本Plan drafting 0/12，无实施/fixture/probe/测试已完成，未创建树或agent；[预检](attachments/m2-governance-batch-20261024/preflight.md)。

## 10. 待澄清事项

当前调度/范围已明确，不需重复确认；T-01负责§2机制、实际三仓路径/消费者与资源策略的有界冻结，门前不得提前probe。无法保持合同或需改变长期支持/源码语义/后端/远端动作则具体needs_replan；不静默缩面、豁免源身份或把本Plan必需集成延期给后继。全部实施等前置完整交付合入/归档收尾，不以status/单阶段pass越门。
