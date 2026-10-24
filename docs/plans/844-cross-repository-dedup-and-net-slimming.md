---
plan_id: PLAN-844
status: drafting
feature_name: 跨仓重复实现、废弃代码清理与净瘦身验收
author: [Codex]
created_at: 2026-10-24
updated_at: 2026-10-24
plan_revision: 1
current_step: 0
total_steps: 12
supersedes_spec_components: [docs/specs/language-sdk/project.md, docs/specs/auto-lang/frontend/overview.md, docs/specs/auto-lang/types/overview.md, docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/trans/overview.md, docs/specs/auto-lang/runtime/overview.md, docs/specs/repository-boundaries/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/repository-boundaries/net-slimming.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [language-sdk, auto-ui, auto-man, repository-boundaries, test-infrastructure]
depends_on: [PLAN-831, PLAN-832, PLAN-841, PLAN-842, PLAN-843]
---

# [PLAN-844] 跨仓重复实现、废弃代码清理与净瘦身验收

> **全Plan启动锁G844**：G843且U843。包含T-01盘点/probe/度量、fixture/测试和建树，门未齐一律不开始；只核验上游状态/完整交付收据。启动证据定义见[共用合同](reports/m2-governance-order-841-844.md)§1。本轮仅规划。

## 0. 变更摘要

完成R12减法半部并核验M2：在三仓最终owner和调用图上找出真实重复/废弃无消费者实现，迁到合适单一owner或实际删除，维护manifest/测试/资源/生成依赖；用统一口径证明生产净删除和重复清偿，而非把移仓/格式/生成物删除算瘦身。

本Plan是4份大模块合同之一，包含实际生产改动、旧实现退出、测试/三仓消费/版本与发行同步，不另按小探针取号。不接手831/832尚未完成的搬仓验收，不扩大为全语言IR替换、新backend/功能或远端发布。

## 1. 目标

- 每个核准重复族有语义/调用证据与唯一owner，生产调用实际切换且原重复退出，不建巨common/复制模型。
- 无消费者候选按cfg/宏/注册/脚本/外部支持审定后真实退役，活跃或未完成能力不被当废码删除。
- 三仓行为/支持/版本和测试覆盖不退化，统一统计实际生产净删>0且有真实重复清偿，形成有限M2退出收据。

## 2. 架构方案

T-01建立受控清偿基线：三仓完整production/test/generated/vendor/asset分类，候选clone-family与语义边界、运行调用/宏/cfg/注册/生成/脚本/已发布公开支持证据；normal/build/dev与受支持平台/feature逐项排查，不凭rg零命中就删代码。843的结构变化先扣除，扫描工具只能产生候选，不能以相似率宣布等价。必须冻结可操作的真重复/可退役项与反例后续任务才开始；若确无符合项则具体needs_replan，不能空审计自判完成。

同名Type、backend fallback、VM/native与转译器编码、UI/Man模板在边界/副作用/错误/顺序可能不同，逐族用合法/错误/极值/状态控制证明可统一；不等价则保持具名不同owner并记录依据。共享只提炼有真实多消费者的最小算法/数据，不为“去重”造common巨包，不以新间接层复制同一状态/模型。

DEPRECATED不代表无消费者。旧Scope/TypeRegistry可能仍有稳定公开或动态注册使用；若仍必要，保留清楚owner或在已授权范围迁调用，不砍活跃功能/未完成checker/后端来刷删除数。旧auto-lang-macros仅为废弃副本候选，先证明Cargo外直接manifest/脚本/公开版本承诺；legacy refs/历史制品/许可及恢复载荷不得删。

净统计覆盖同一组固定三仓commit，报告本Plan前后与M1前历史基线的分开差额：production手写行数/真实重复族/无消费者退出/依赖禁向边/最大模块/构建测试成本；测试/新增helper、生成/第三方/资产分别列。生产净删=同口径生产源码减少，扣新增实现/helper，纯移仓/格式变化不计；必须>0且至少有真实生产重复清偿，不能靠删测试/资源或把生产标generated。成本测量仅如实记录同环境输入，不承诺未经测量的提速比例。

有限完成是全部T-01核准真重复与无消费者目标关闭、未行动候选有证保留且不是目标内延期、三仓全部既有支持不退化，并对841领域终态/842过渡退出/843结构逐条引用真实已交付证据。832发行缺口不转来补票；未来新功能/新IR/后端退役仍是其他目标。

## 3. 技术栈

现有Rust/Cargo与三仓固定Git版本、单源AST/值/词法/Parser API/SDK、UI文法/生成器/runtime及Man CLI，复用777边界、792发行与已有真实consumer/双端设施；不新增通用框架/平行runner。实际包名/路径以启动后上游来源映射冻结，当前源码仅观察基线。

## 4. 需求分析与背景调查

用户2026-10-24明确要求按4份较大计划规划交付目标、验收与顺序/831832启动前置；继承此前保守排程，全部实施任务严格等完整前置交付。**本轮只在语言主检出建立合同/设计/路线，不改实现或canonical Specs/ledger、不建树/agent、不写旧仓/远端**。已有授权只覆盖目标与规划，未来work按明确执行指令和门启动；常规机制选择不需重复批准，改变既有支持/兼容承诺则具体needs_replan。

从[Spec入口](../specs/overview.md)定位，[AST](../specs/auto-ast/project.md)、[词法](../specs/auto-lex/project.md)、[Parser API](../specs/auto-parser-api/project.md)、[SDK](../specs/language-sdk/project.md)、[前端](../specs/auto-lang/frontend/overview.md)、[VM](../specs/auto-lang/vm/overview.md)、[旧仓](../specs/repository-boundaries/project.md)与[发行](../specs/repository-boundaries/generated-ui-distribution.md)为当前依据。部分旧Spec数量/路径滞后，不以旧聚合描述覆盖实际模块代码/交付；观察与假设、源SHA/hash见[输入](attachments/m2-governance-batch-20261024/inputs.json)。

已观察写面：三仓最终源码与843交接重复候选、SDK infer/registry.rs、scope.rs、旧crates/auto-lang-macros与auto-macros、VM/FFI/trans纯转换与注册族、UI/Man生成/资源helper、Cargo成员/依赖/build.rs/宏/include/测试与脚本。上述仅候选，当前未做可删除或语义等价结论。

当前831 executing 3/12、832 drafting 0/12；都未完成M1交付。新机制/真正可删候选/未来路径不声称已证明，T-01在G844后承担有界决策和实际编译/消费probe，关闭条件见§2/8。

## 5. 详细设计

### 5.1 启动、所有权与执行入口

G844对T-01..12逐行生效，depends_on是全Plan启动门，不是只约束最终集成。U定义为全部AC/final review/冻结门/实际合入/Spec与受管ledger/来源交接/归档收尾闭环；阶段pass不放行。门未齐只读上游状态/交付收据，保持drafting 0/12，不提前盘点/扫描/probe/改fixture/建树或启动实施agent；无新后端status/调度功能。

G844后T-01读取所有已发布三仓规范/agent指令、前置delivery commit/路径映射，新增attachments/844/contract-baseline.md：具体producer→数据/状态owner→consumer/产物、支持/预算/错误/退出规则、正反例与可执行oracle、未知决策/owner/关闭条件。假设必须在依赖实现前用最小真实整链probe证明；无解则保范围转needs_replan，不用单元假绿偷降AC。

### 5.2 三仓生产接线与来源

按共用合同§2–4及§2本Plan专门设计实施。所有类型/状态/算法单一owner；normal/build/dev与feature图无UI/Man反向污染SDK，不靠宽化777或临时[patch]双source过门。变动同时更新真实三仓consumer、固定版本、测试ID/alias/CI、build输入/生成import/资源/producer与缓存身份；832的旧发行失败合同继承，不新造弱事务。

版本升级/兼容差额记录在来源清单；外部正常消费与具名错误/缺件/取消/退出必须实际验证，未测环境pending。正式API或序列化不兼容仅按本Plan明确版本迁移合同实施，需改变既有承诺则停止该项给具体决策，不把用户规划授权扩张为随意删除稳定API。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/language-sdk/project.md, docs/specs/auto-lang/frontend/overview.md | 已核准SDK/frontend重复/无消费者副本→实际唯一owner或删除并保行为 | 去重复而非挪目录 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-lang/types/overview.md, docs/specs/auto-lang/vm/overview.md | types/VM候选重复与deprecated残留→逐项消费者/语义审定后的单源/退出/有证保留 | 防误删活跃能力 | AC-01..05 |
| SD-03 | modify | docs/specs/auto-lang/trans/overview.md, docs/specs/auto-lang/runtime/overview.md | trans/runtime重复族→最小共享算法与明确差异保留，源码与状态无副本 | 不造common巨包 | AC-01..05 |
| SD-04 | modify | docs/specs/repository-boundaries/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md | 仓/发行manifest/资源/测试引用→对应实际删源的单源锁/producer/全档覆盖 | 瘦身不破发行 | AC-01..05 |
| SD-05 | add | docs/specs/repository-boundaries/net-slimming.md | 无统一净统计→固定三仓基线/分类/生产净删与重复清偿、M2有限退出收据 | R12减法半部 | AC-01..05 |

## 6. 测试设计

下列Case均owner_stage=work，由T-01冻结实际输入/入口与oracle，T-02..08逐步建立永久定向控制并补真实集成；**本轮尚未执行**。原ast-model/ui-grammar/language-sdk/generated-ui/tool-client等consumer与语料/投影golden按真实owner复用，不以重写golden遮回归。新增fixture用`test/fixtures/modularity/`下具名目录，具体接口T-01冻结；新接口未存在前不把未来命令当已执行。

| Case | AC / invariant | 输入/真实入口 | 可观察oracle与覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 | 每核准重复族原两个真实消费者，合法/错误/边界/副作用次序对照 | 新唯一实现满足各自合同、生产调用确已切且旧副本退出；不等价控制阻止错误合并，不以snapshot相同概括所有状态 |
| V-02 | AC-02 | 候选死码的cfg/target/feature、注册字符串/宏生成、直接manifest/脚本及公开消费 | 真正无消费者者删后所有约定支持仍工作；模拟遗漏注册/平台消费者负控必须拦删除，长期公开承诺不被悄删 |
| V-03 | AC-03 | 真实Core/CLI/UI/生成工程/服务/取消与错误控制，fresh/locked、版本资源负控 | 结果/错误/状态/资源保持，正常profile不降级，坏输入不假成功；原程序/后端不被删除或测试屏蔽 |
| V-04 | AC-04 | 固定三仓前后source清单，跨仓搬同一文件/纯格式/新helper/test/generated/vendor变化对照 | 可重算净生产减少>0/重复清偿；搬移=0净删、helper入账、测试/生成/第三方分列，分类变化可追溯不能挪统计分母 |
| V-05 | AC-05 | 全部清偿矩阵与843/842/841收据、依赖图/测试ID/新consumer选择与资源恢复 | 目标内无未关行，保留有语义/消费者理由；M2每项真实源/规范/交付证据完整，不把future TODO作达成 |

work/review只必要check（真实SDK/被改包）、相关包/模块/fixture和生产反例、实际外部工程构建运行；777 strict与792适用面/缺源负控不能省。类型/VM/trans/UI生成触面final review通过后，merge在冻结三仓输入统一裸cargo t及tv/tt/tu等适用档（按迁后alias/owner等价入口），tf到期语言主检出单实例并覆盖迁出家族。修复先完成本轮全部findings和反例/邻接组合再提交送审；无单finding全档/边改边审。缺环境/上游不能fake正控，当前无指定agent/model，不创建会话；本轮Category A无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | V-01 核准真重复族单源、生产调用已切，原重复清理，边界不同的保留有实证，零新common大杂包。 |
| AC-02 | work | V-02 可退役无消费者实现实际删除，cfg/动态注册/宏/脚本/公开支持对照齐备，活跃功能/历史恢复不误删。 |
| AC-03 | work | V-03 VM/trans/UI/Man现行行为与状态/错误/资源/安装支持完整保持，删除反例与真实消费通过。 |
| AC-04 | work | V-04 三仓可比口径生产净删>0且真实重复清偿，行数/依赖/规模/成本/来源可重算，非移仓/删测试刷数。 |
| AC-05 | work | V-05 测试ID/alias/source/build/资源/锁/指纹同步，清偿矩阵与M2逐项实际交付/SD/剩余非目标报告齐备。 |
| AC-06 | review | 独立最终review绑定r1/实际三仓commit及依赖/配置/输入，全AC/SD/遗漏/警告/格式/debug审查通过；merge门pending |
| AC-07 | merge | 并集合流影响复审/冻结门通过，实际源码与consumer版本交付、canonical/受管ledger/来源读回、归档收尾与guard清理闭环；不以部分发布放行后继 |

## 8. 执行步骤

12项，9 work+1 review+2 merge；全Plan启动门覆盖所有行，保守串行，不提前做独立小任务。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | G844完整核验 | 冻结三仓同口径净统计/候选真重复与无消费者清偿矩阵、语义/引用/cfg/公开支持与基线，确定实际可操作目标 | AC-01..05 |
| [ ] | T-02 | work | G844；T-01 | 对核准通用纯工具/数据转换族抽唯一最小owner，切真实消费者并删原副本，合法/错误对照 | AC-01,03 |
| [ ] | T-03 | work | G844；T-02 | 对核准VM/FFI/trans或UI/Man跨族重复逐项保持边界/状态语义清偿，不同语义保留有证，不建巨common | AC-01,03 |
| [ ] | T-04 | work | G844；T-03 | 退役核准的旧registry/Scope/其他无消费者实现或必要迁调用，保活跃/公开稳定/动态注册能力 | AC-02,03 |
| [ ] | T-05 | work | G844；T-04 | 清理核准遗留crate/废弃副本/无效入口与配置，完整cfg/脚本/直接manifest负控，保legacy历史与许可 | AC-02 |
| [ ] | T-06 | work | G844；T-05 | 同步manifest/锁/alias/注册表/生成路径/资源/build输入与指纹、777/792，保持原测试ID覆盖 | AC-03,05 |
| [ ] | T-07 | work | G844；T-06 | 重算三仓生产净删/重复族/模块/依赖与同环境成本，验证分类/搬移/新增helper负控V-04 | AC-04 |
| [ ] | T-08 | work | G844；T-07 | V-01..05全部正反控与三仓真实消费者/外部fresh/locked/资源退出完成，清偿矩阵核销 | AC-01..05 |
| [ ] | T-09 | work | G844；T-08 | 引用841/842/843实际交付逐项核M2有限退出、无目标内延期，SD-01..05及attachments/844/handoff.md冻结送审 | AC-04,05 |
| [ ] | T-10 | review | G844；T-09冻结提交 | 同组独立完整AC/SD/实际consumer/反例/来源与健康审查；不先跑整档，不将merge门当已通过 | AC-06 |
| [ ] | T-11 | merge | G844；T-10 pass | 并集影响复审、冻结三仓适用门与有序实际合入/固定版本/来源读回；若改变输入重审影响面，暂不放行后继 | AC-07 |
| [ ] | T-12 | merge | G844；T-11全部交付收据 | canonical/受管ledger/索引沉淀、归档与逐树fresh wt-guard clean清理，整Plan闭环构成U844才放行下一份；保历史/恢复/交付分支 | AC-07 |

G844未满足前禁止创建/认领以下组、分支或work agent。未来组`D:/autostack/.wt/lang-844/{auto-lang,auto-ui,auto-man}`，每个实际需写仓各一树/分支`codex/plan-844-dev`；只读consumer用固定版本clone而非借其他计划WIP，未需写仓不多建树。work/review/merge沿同组顺序交接，不建第二审查树；若auto-down为现有可选依赖则具名固定版本/detached兄弟，只解析不改实现。禁止junction/symlink，主检出只簿记，清理前每树fresh guard clean且保留实际交付/legacy恢复源。

## 9. 复审记录

2026-10-24 / stage: new / plan_id: PLAN-844 / revision: 1 / outcome: blocked（上游整Plan启动门未齐；文档合同预检通过）/ next: 等G844真实满足后读回规范/交付/归档版本，再work T-01 / changed: T-01..12、AC-01..07、SD-01..05、V-01..05。本Plan drafting 0/12，无实施/fixture/probe/测试已完成，未创建树或agent；[预检](attachments/m2-governance-batch-20261024/preflight.md)。

## 10. 待澄清事项

当前调度/范围已明确，不需重复确认；T-01负责§2机制、实际三仓路径/消费者与资源策略的有界冻结，门前不得提前probe。无法保持合同或需改变长期支持/源码语义/后端/远端动作则具体needs_replan；不静默缩面、豁免源身份或把本Plan必需集成延期给后继。全部实施等前置完整交付合入/归档收尾，不以status/单阶段pass越门。
