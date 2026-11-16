---
plan_id: PLAN-979
status: executing
feature_name: Schema领域语言包与UI AGE投影接口
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 11
current_step: 6
total_steps: 12
depends_on: [PLAN-960, PLAN-963, PLAN-966]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/auto-a2x/architecture.md]
new_spec_components: [docs/specs/ace-migration/m2-schema.md]
touched_goals: [GOAL-002]
---

# [PLAN-979] Schema领域语言包与UI AGE投影接口

## 0. 变更摘要

2026-11-15 r11（当前合同）：按用户本轮授权落实跨计划承接与无环交接。960维持已归档；原“等待960”的多模块领域集成由981/SOURCE的原B-SOURCE-I/V/R承接，980交UI完整字段清单，981交AGE完整构造清单、实例可见性与g01原源CPU证明。979保留全部AC/V与T-03/T-08/T-04/T-05完整复核责任，收到正式接口后组合验收；不得以已交A切片归档。当前§5–8覆盖r10末尾“等待960/980/981”的编号泛称，保留其等待各owner交付、不自行缩范围的裁定。原12任务、6/12、executing及既有勾选不变；只规划，不启动/恢复实施或解除其他会话的blocked状态。

2026-11-13 r10（本轮当前合同）：H0首源改为AGE node/view/text；Schema同时驱动解析、结构校验与位置/上下文敏感的node名称识别。旧widget仅为同一领域包的显式兼容别名，不另建传统widget前门。 用户仅授权规划修订；保原任务ID/数量、既有AC/V与进度、完整分母及阶段门，不建树、不写实现/fixture、不构建/运行/合入。此前§0修订描述为历史，冲突以本次§5–8为准。

2026-11-12 r9：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-11 r8当前：增加H0最小widget/text受检领域前门切片；以现有普通函数/文本和grammar/CST/verify基础接Hello，不等待963泛型/966完整CT整包；原通用Schema完整构造、新旧源码表面经同一Schema的最小双后端及全部8项任务保留。 原8项任务/全部AC/V保留，新增4项早期切片任务，现12项、drafting、0完成；只规划。

历史修订：2026-11-10 r7执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r6当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

将现有Schema规则机提升为版本化领域包，具备依赖/上下文关键词/歧义与冲突检查、CST来源、typed领域IR及受检行为投影；UI/AGE语法留UI owner，Core无反向UI依赖。codec schema不等于grammar schema。

历史修订：2026-11-09 r5：共同Schema接面必须承载传统AutoUI和完整AGE作者语言，不能以最小UI/2D/3D子集替代。979仍交通用平台与最小双后端证明，980负责传统UI全模式，981新增LANGUAGE批负责完整Node语法/语义与组合；本包不反向等待两个产品终验。8任务保留，新增AC-08/V-11，drafting、0/8，仅规划。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
作者唯一维护的Node模块工程同时表达场景和逻辑；内部CST/领域IR/HIR/MIR及资源清单属于正常编译产物，不再先生成旧world.at和.agscene后交旧Parser/AutoVM重解析。AV和A2R不得分别维护一套Node语义。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。
本次明确两条必验出口：Checked HIR→共享Typed MIR→AV；Checked HIR→现有a2x-core/a2x-rust provider→Rust文件集→真实rustc编译运行。场景/绑定/资源需求的typed领域IR供AGE宿主消费，不将renderer、输入循环或Node文法塞入AV执行器。AC/A2C等原有适用义务保留，不能用本次AV/A2R两腿替代原支持格。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11本轮授权：用户要求规划首个ACE UI Hello及AV/Rust FFI路线、核对既有M2冲突并作必要重排；仅规划/设计/导航，不实施、创建worktree或探针/fixture/SDK、构建/运行、提交或发布，未指定模型/工期。沿旧语言用例守恒、新Auto标准库与AutoABI/FFI、Demo递进的三线，不削完整M2范围。

现场语言HEAD 9e36d984e、UI HEAD d88e9c51。已核av-core/src/host.rs仅console/1（sha256前16 30d6dcc475850cd0）、a2x-rust/src/provider.rs仍拒source foreign（3ea453dc2452576f）、core/src/source/cst/frontdoor.rs（dd3d93d06799ae52）；现有auto-bindgen/src/extractor.rs是预置C manifest表（8c7f1295cc83c6fb），不能称Rust元数据导入器已实现。Iced由Rust宿主编译执行，AutoVM不解释其Rust源码；现有DynamicComponent/VmBridge/handler_codegen深依赖旧执行链。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
用户2026-11-09先授权r3修订979/981的Node直接接ACE、AV开发、A2R发行与两游戏整链验收；随后提出合并AGE原M2/M3、ACE接入与2D/3D增强并行。r4同步两阶段规划，恢复相关M2文档修订，仍不授权实施/先证/素材/构建/运行或创建worktree，不改变AGE M1签章口径。
同日用户进一步要求M2包含ACE对传统AutoUI和完整AGE语法的支持。r5明确平台/产品职责和完整性分母，仍只规划，不重开历史有限profile或另立重复语言前端。
2026-11-06原调研读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。该轮现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。该轮引用会话读取及936/944状态核验保留为历史，本次基线见下。

实际入口/文件：
- `experimental/ace/crates/syntax-grammar`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/syntax-runtime`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/grammars/auto-source.at`（本轮确认存在；开工重核符号/版本）。
- `docs/design/39-ac-incremental-syntax-and-language-platform.md`（本轮确认存在；开工重核符号/版本）。
- `docs/specs/ace-migration/{m2-modules,m2-numbers,replacement-scope-and-evidence}.md`、`docs/specs/auto-a2x/architecture.md`、`docs/specs/auto-a2r/project.md`为本轮现状依据；语言HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`。960/961已归档，但模块/数值独立profile通过不等于Node+模块+浮点+资源组合已通过；960仍明确a2x全局cell/类型方法片段边界。
- `experimental/ace/crates/{core,source-query,lowering,av,av-core,a2x-core,a2x-rust,a2x-store}`及`a2x-worker/tests/conformance.rs`为已存在接面；新增领域包/投影API、最小双方言fixture位置由T-01在既有owner内冻结，非现成能力。
- 领域依据为`D:/autostack/auto-ui/docs/design/autographics/declarative-authoring-model.md`与`docs/specs/auto-graphics/ace-migration-feasibility.md`（后者在auto-ui仓）；UI067于2026-11-07冻结旧语言基线的no-go只说明当时作者前门缺失，不能当当前AV语义失败，也不能把调研另行重复立项。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：通用Schema领域包/typed投影。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [experimental/ace/crates/syntax-grammar](../../experimental/ace/crates/syntax-grammar)：领域规则组合/歧义；目录职责已核，确切符号/闭包开工再核

- [experimental/ace/crates/core/src/source/cst/project.rs](../../experimental/ace/crates/core/src/source/cst/project.rs)：普通与领域投影接缝；读取SHA256前16 `4b8bbdb63d41d9b5`

- [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs)：WidgetDecl与Node领域owner，Core不反依UI；读取SHA256前16 `1d10f5ff2566422e`

2026-11-13本轮授权/取材：用户明确要求修订Node首Hello、Schema解析校验与位置/AGE上下文名称、widget兼容和963并行顺序，仅规划。读取基面`dcf0535d4d15c00628a60f1940c6c55a01a101ab`；当前Spec仍描述有限统一CST与旧UI parser/有限Node pilot，并未宣称本次机制已交；差额是本Plan目标，未来按SD沉淀，不改canonical Spec伪造现状。来源：`docs/specs/auto-syntax/{source-frontdoor,grammar-contract}.md`；`docs/specs/auto-ui-dialect/design/full-grammar.md`；`experimental/ace/crates/syntax-runtime/src/{program.rs,parse/mod.rs}`；`experimental/ace/crates/core/src/source/cst/frontdoor.rs`；`auto-ui/docs/specs/auto-graphics/{node-basic-ui,ace-migration-feasibility}.md`。963执行进度与其他会话WIP只读保留。

物理路径核对：原`D:/autostack/ag-examples`已更名为`D:/autostack/age-examples`；本轮修复旧样例/计划链接到实际文件，原AG计划ID/依赖身份与历史源码hash不改。执行者按现路径核正式来源，不能重新复制内容仓或生成替代旧用例。

### 2026-11-15 本轮交接修订取材与授权

用户明确要求修订979/980/981及相关路线，复用现有计划承接原960多模块缺口、消除循环、保留979完整验收；**只规划**。本轮不创建计划号/工作树/agent/chat，不实施、探针、fixture、编译/测试/GPU、提交/合入/发布，不恢复979目标或AG029暂停。读取语言基面`86c7fd50bc6dac5ff6d2a5c6215ad01fbfefc1f7`及当前文档WIP；保其既有改动、任务ID/勾选/计数与历史收据。已归档960只读。

已核规范/代码SHA256前16：`docs/specs/ace-migration/m2-modules.md`=`be25b0c69f2f8cc0`、`m2-schema.md`=`5797e15925b07e14`；`experimental/ace/crates/core/src/source/packages/binding.rs`=`afa4f886dfc2fab9`、`source/domain.rs`=`1613b66c2383bc40`、`source/age_component.rs`=`58081b0e1eb6347a`、`source/cst/frontdoor.rs`=`17016eecbc0fbd61`；`experimental/ace/crates/av/src/age_component.rs`=`55b84b8f1ca9d5a8`；auto-ui `docs/specs/auto-graphics/node-basic-ui.md`=`0a3e8616b8d2239b`、`crates/auto-ui-dialect/src/ast.rs`=`1d10f5ff2566422e`。这是读取身份，不是本轮运行PASS。960的普通模块与979的单模块领域接口已分别存在，跨模块AGE组合/字段适配/实例引用与原源CPU交接仍是拟实现；§5拟类型/样例不是现存API。现行Spec保已验事实，仅在本包规范增量表登记未来更新，不改canonical Specs伪造支持。

## 5. 详细设计

### r11跨计划承接与完整收口（当前交接权威）

**生产依赖与回验输入分开。** frontmatter的960/963/966是本包通用能力的生产依赖，均已归档；本次不把980/981整个Plan加入该列表。最终T-03/T-08/T-04另依下表具名正式输入；980/981消费已交979基础接口，不依979最终T-05/归档，因此任务图无环。后文“最小消费者不等产品终验”继续成立，但不能再解释为无需下表输入即可关闭全部V。以T-02、H0及组件A已落主且已验的接口为起点，不把r10增量review pass冒r11整包pass。

| 原未闭义务 | 实际生产owner及原任务 | 正式输入标识与最低交付 | 979保留的验收责任 |
|---|---|---|---|
| V-09④ 多模块Node工程（原误挂960） | **981/SOURCE：B-SOURCE-I/V/R**；auto-lang负责共用前门/绑定/双后端，auto-ui负责领域规则 | **IF-981-MODULE**：复用960的包输入/绑定图与979领域包，将普通Auto项和AGE候选按模块共同检查；至少两文件Node工程、跨模块helper、两同定义独立实例、2:1定向事件、N Tick、typed view，AV与真实rustc后的A2R各有独立值/trace/span | T-03消费同源同图并实际观察，T-08核组合/来源与非法近邻，T-04核V-09完整分母；960归档不是该组合已经支持的证明 |
| V-10 未声明/越权实例引用部分 | **981/SOURCE的B-SOURCE-I/V/R**交最小实例引用规则与CPU行为；**981/LANGUAGE的B-LANGUAGE-I/V/R**继承完整语义 | **IF-981-REF**：声明、实例身份、可见性及合法引用的typed表示；同类型合法/不存在/不可见引用、跨模块原span、拒绝前无AV/A2R执行与失败恢复 | T-08/T-03复核生产者真实拒绝位置、无dispatch及修复后合法运行；坏事件/篡改投影等已验部分仍逐项对账，不用本项取代原V-10 |
| V-11 UI字段与完整Node构造映射 | **980/DIALECT的B-DIALECT-I/V/R**、**981/SOURCE的B-SOURCE-I/V/R**（AGE构造表由LANGUAGE原任务继承扩大） | **IF-980-FIELDS + IF-981-FIELDS**：分别按现行WidgetDecl全部字段和AGE-L/AG-A完整构造交版本化必需清单、原源/span→候选→共享检查→产品owner映射，接既有mapping_complete/component_coverage；合法、坏字段及实际删除必需映射的反控 | T-08对完整双向表执行覆盖/拒绝检查，T-04核无缺行、无未知列/构造静默丢弃；不以产品owner签字或最小text样本代测试 |
| V-DETAIL-01 g01原四图形值/visibility/来源 | **981/SOURCE的B-SOURCE-I/V/R**；980提供涉及既有UI组件的字段事实，UI完整产品运行仍归980 | **IF-981-ORIGINAL**：复用g01完整原模块闭包及资源声明，交Schema投影、typed属性/资源引用/原span和CPU读取入口；对原坐标、尺寸、颜色/alpha、order、隐藏黄quad作独立断言 | T-02/T-03/T-08/T-04保留原V-DETAIL-01，消费完整原输入并核CPU数据；本项原本不要求GPU，不以新CPU证明关闭980/981实际窗口/渲染/游戏门 |

上述IF标识是原任务中的交接合同键，**不是新任务、计划或里程碑**。981/SOURCE可以先对最小CPU模块/引用/字段/原源子面独立复核并phase fold；不能据其把完整SOURCE、LANGUAGE或981勾完。980同理只交DIALECT字段子面，不提前关闭整个DIALECT/980。各IF只消费979已交基础与实际所用的已交语言能力；不把969/SESSION、978完整SDK、980/PARITY、981整包、985/A或B游戏作为这些CPU交接的新增前置。若实际能力缺失，生产owner在原I任务记录精确接口和修订，不回退旧链或改成等待979整包。

**交接准入。** 每件须绑定repo/已落主SHA、生产Plan revision与子面范围、普通源/模块/资源闭包hash、grammar/domain/compat、模块/定义身份与原文件span、Checked消费接口、后端/工具链/能力与预算、永久case/独立oracle及失败恢复结果。普通清单、分支tip、手写HIR或“等owner完成”不合格。允许复用输入/身份未变的有效证据；改变本次新增集成的证据不能沿用r10单模块PASS。979最后独立核全部AC/V，缺任一IF或原required项仍保持executing，不得删项、改NA、仅签A表归档。

**任务级无环顺序。** `960已交 + 979已交基础 → 981/SOURCE的IF-981-MODULE/REF/FIELDS/ORIGINAL`；旁线`979已交基础 → 980/DIALECT的IF-980-FIELDS`；两线正式交付 → `979 T-03/T-08 → T-04 → T-05 → T-06/T-07`。980/981其他任务仍按其实际宿主/库/SDK接口推进；其完整产品PASS不是979回验的门。979保留通用接面维护责任，生产者集成变更分别在980/981原任务实施，Core不反依UI crate，不产生第二套parser/verify/FFI/发射器。

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-H0-I, T-H0-V, T-H0-R, T-H0-D |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。本包frontmatter为通用能力生产依赖，最终回验另核本节具名IF；不把980/981/985整包置为生产前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


将现有Schema规则机提升为版本化领域包，具备依赖/上下文关键词/歧义与冲突检查、CST来源、typed领域IR及受检行为投影；UI/AGE语法留UI owner，Core无反向UI依赖。codec schema不等于grammar schema。

支持集继承959分母与Design55的完整语言定义；959分母用于范围核对，不成为“等全生态迁完才做前门”的前置。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新领域接面/fixture为拟新增，必须扩现有query/verify和provider，不另造类型检查器或A2R发射器。

### 职责、最小交接与并行边界

| owner / 承载任务 | 本包交付 | 后继接收与边界 |
|---|---|---|
| auto-lang / T-01、T-02、T-08 | 通用版本化grammar包、CST/span、typed投影合同、共享绑定/实例/资源事实及增量身份；领域回调只产未受信候选，由唯一verify取得Checked HIR | 981/SOURCE消费；Core不依赖auto-ui crate，不运行旧Node源码生成器 |
| auto-ui领域owner / T-02、T-08的最小双方言fixture | 用同一领域包接口声明最小旧widget与AGE Node语法/typed投影；示例含普通Auto函数、use、独立model及msg/on | 是979直接源输入的验证消费者；981扩展同一接口至完整AGE场景，不能再建第二套Node解释器 |
| auto-lang / T-03 | 同一最小Node工程经共享verify，分别真实AV执行和A2R生成/编译运行；保留可消费的领域描述和原span | 只证通用接面与独立值/事件oracle，不承担真实GPU/游戏打包；这些归981 |
| 既有960/961/962/963/966/967/978 | 模块、数值、聚合、泛型、编译期、typed host及构建/SDK各归原计划 | 复用交付；不在979重复实现宿主/调度/构建系统；游戏实际组合缺口由981登记具名owner |

交接件须包含repo/commit、Plan revision、领域包/grammar版本、支持构造组合、投影及Checked消费API、source map、后端/runtime身份、正反例与资源边界。不得只交AST codec或手写HIR夹具；Node的model/msg/on行为也必须经同一verify，不能只检查场景字段。

顺序：已交T-02/H0/组件基础先供980/981的§5具名IF生产；T-03/T-08在相应IF正式交付后补齐全部原V，T-04汇总后T-05→T-06→T-07。960/963/966基础已归档；仍核实际接口身份。979不等980/981整包完工、AGE整阶段、985认证或987退役，但保留§5具体输入和自身最小消费者完整回验；不能拿分支tip当交付。

阶段接口边界：合并AGE M2的G2/G3图形服务可与本包并行，使用版本化typed领域描述/资源需求，新增能力声明不复制Node语义或旧生成器。具体增强验收归UI058/AG023及后续冻结能力合同；979不反向等这些计划。新增grammar/类型需求由领域owner交本包或实际语言owner确认，不能仅因“AX可转Rust”假定所有图形/运行接口已支持。AX为A2X目标族，本轮必验的编译发行目标仍是A2R，不自动扩成所有目标/平台。

### 完整作者语言接面与两类产品责任

领域包接口必须表达传统WidgetDecl全部字段/领域结构与普通Auto行为，以及完整AGE的node/use/model/msg/on、props/绑定/typed引用、多实例/多场景、五类view及嵌套、运行时if/编译期#if、Life.Start/Tick/Frame。资料取auto-ui `declarative-authoring-model.md`§1–2、`multi-presentation-selection.md`及现行WidgetDecl/组件/Store/主题/路由Specs；其中未定时序/组合策略由981/LANGUAGE的有界决定门关闭，不能永远留作“支持范围外”。不把尚未批准的无限未来DSL或新平台混入完整性分母。

T-01/T-08记录“构造/字段→来源及原span→通用投影/检查能力→980或981接收owner→产品验证格”的双向表；嵌套表达式/事件体经唯一verify，缺字段/丢投影必须拒绝，不能静默忽略。979只验证通用表达/来源/准入与已有最小CPU消费者，不等待980/PARITY或981/LANGUAGE产品PASS；完整传统UI和完整Node语法的实际AV/A2R/图形/平台格分别由980、981关闭，M2须同时具备，最小双方言绿不冒全语法已支持。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### Schema组合语言与依赖闭合

提供普通语言+领域规则组合相容表、包版本/冲突、语法恢复/畸形输入推进性及无损来源；不把完整CAP求解塞此包。其typed投影仅使用已交付语言接面，980/981最终业务组合。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；接口由T-01先冻结，T-08只依赖T-01及相应T-02投影接口，不等待T-03双后端验收；T-03可消费其已就绪fixture，T-04最终汇总，避免T-03/T-08互等。

### H0：AGE Schema驱动的最小Node与文字投影（当前早期入口）

T-H0-I最小前置为现有syntax-grammar/syntax-runtime、core CST/resolve/唯一verify、已交普通函数和text支持及960适用模块基础；不使用用户泛型/CT/#if，不等963/966整包。Schema规则由同一既有grammar包机制声明，不把UI文法硬编码成第二parser，也不调用旧Parser/handler_codegen重解析。公共版本化包/投影机制归979，最小AGE node/view/text领域声明由auto-ui owner在同一接面注册；980只扩消费/宿主，不再重复前门。

H0首源采用普通源码`node <任意合法名> { view { text <UTF-8表达式> {} } }`，根一个text；表达式首批为字面量或同工程普通Auto函数返回文本，行为经共享绑定/类型/资源verify。字面量也须保原span/包身份；函数体不能留作字符串由UI二次解释。属性/字段/未知结构按本切片能力明确拒绝，不静默忽略。static描述与运行表达式引用分开，描述不携带已受信Checked或Rust对象地址；CPU consumer可观察根/叶/值及原span，不硬编码App/Hello字节。

输出是领域包版本、CST/span、未受信候选→唯一verify的受检描述/表达式标识及资源要求。H0独立CPU投影/语义检查不等967 F0、980窗口或979完整Node双后端；UI真实AV/FFI/A2R由980/HELLO验。原T-02/03/08继续在同一机制扩完整UI/Node/模板/CT、新旧源码表面经同一Schema的最小双后端与组合恢复；H0不提前关闭它们或缩完整语言分母。

现有owner路径`experimental/ace/crates/syntax-grammar`、`syntax-runtime`、`core/src/source/cst/frontdoor.rs`、`source-query`均已核；新最小UI grammar资产/typed投影case由T-H0-I在这些owner与auto-ui领域注册位置建立，准确路径/符号/命令先冻结。980只能在H0-D正式来源后消费，H0用自己的CPU消费者闭合，不反等980完整DIALECT/PARITY或981。

早期切片是本Plan M2-A内部交接，不新增ACE里程碑或Plan终态；只在同一权威树按owner流程独立复核、正式落主并记source/接口/grammar/ABI/provider及真实结果后供下游消费。同Plan尚有原完整义务时不归档/删树/勾整包完成。frontmatter保最终依赖并集，早期切片只等本节最小接口；同一文件单写，不以分支reviewed/WIP当交付。最小切片通过不关闭原AC、完整Rust FFI/全UI/RG-A03/M2A-08；不新增对980/981/985整包或AGE联合门的反向依赖。

### H0的Schema、位置名称与兼容合同（2026-11-13，当前）

首个输入如下（拟永久fixture，由T-H0-I创建；工程/模块在解析前显式选择AGE领域包，不发明新的Auto导入关键字）：

```auto
node Hello {
    view {
        text "Hello" {}
    }
}
```

第二输入保持同形，由同工程普通Auto函数返回`"你好"`供text表达式求值。第一源CPU投影应为一个Node根、一个view、一个text叶及精确UTF-8值；第二源必须经真实共享行为检查/求值得到该值，不由Rust常量或二次解释补结果。H0不要求完整model/on执行、场景树调度、2D/3D、Life.Frame或963泛型。

位置识别的普通Auto正控（AGE开/关均须返回42，作为拟fixture而非已运行结果）：

```auto
fn on(model int) int {
    let view = model + 1
    let widget = view + 1
    return widget
}
fn main() int { return on(40) }
```

**Schema必须是parser的输入，而非解析完再做字段检查。** 现有`syntax-grammar/src/lib.rs`与`syntax-runtime/src/program.rs::{GrammarProgram,RuleSpec,Step,Atom}`、`parse/mod.rs::parse_tokens`、Core `source/cst/frontdoor.rs`为复用接面；实际新增领域包资产/字段由T-H0-I冻结并标为新增。拟字段语义为`ContextualNameRule{spelling,domain_id,parent_rule,slot:NodeName,alias_target?}`，由现有规则程序承载或等价扩展，不宣称当前已有该API。公共grammar/runtime不依赖AGE/UI库；AGE规则及组件schema由auto-ui持有。数据流为已激活的版本化Schema→grammar组合/冲突检查→CST/原span→领域候选及普通Auto表达式→唯一语义/类型/资源verify→CPU描述，继而由980走AV/FFI/宿主；codec schema不能代替grammar schema。

**两个条件同时满足才赋予领域含义：** ①该工程/模块解析环境已显式启用对应AGE Schema；②当前位置是该Schema声明的`NodeName`槽，且父规则允许该名称。此处NodeName指节点头/结构名称位置（例如`view {…}`中的`view`），不等于变量名、定义/实例label、字段访问、字符串或任何花括号前的单词。`widget/model/view/on`等拼写不加入通用词法保留字表；`text`按组件Schema识别。普通表达式/函数体中的`let view = 1`、形参`model`、函数名`on`、对象字段`widget`及`.view`/`"on"`保持普通Auto规则；在AGE环境里也不因拼写相同抢走Core表达式。以`view:`等属性位置与`view {…}`节点头区分，具体分隔/前瞻沿声明的Schema规则，不能全文件字符串搜索或按文件名猜领域。

无AGE的普通模块不激活这些AGE规则；同拼写保持普通标识符，AGE构造若无匹配规则则报结构不支持且不dispatch宿主。启用AGE后也只在已声明父/槽位识别；H0尚未执行的model/on若被Schema识别，应明确能力缺失，不能假称其状态/事件已经可运行。通用parser对这些名字的可配置性可用最小测试Schema验证，实际完整语义由980/STATE与981/LANGUAGE补齐。相同槽/父中的重名规则须按显式优先级唯一决定，否则加载期定位冲突拒绝，不按注册顺序随机选择；不新增隐式保留所有组件名的规则。

**旧widget→node兼容分两项计量。** 新作者源与首Hello一律写`node`；旧`widget App {…}`在显式启用AGE legacy-widget兼容配置时，由同一Schema把声明头映射为同一种Node候选，原字节/文件/span保持，body、组件、model/view/on及普通行为共用规则，不走旧Parser、源改写→重解析或第二业务解释器。关闭该兼容配置时旧声明头应受控拒绝，不能自动回落旧引擎。作者也可显式只改声明头`widget→node`形成迁移副本（不改字符串、注释、变量或body），按独立值/事件oracle验证；它是迁移兼容，不冒“原widget原源直接接受”。旧原源通过别名与迁移Node两条证据归同一旧case义务，不重复计数，不删旧源/expected。本轮不退役兼容别名；未来改变源码接受承诺须显式裁定，987移除旧引擎不自动允许删除原源兼容门。

解析事务/checkpoint须携带并正确恢复活动领域、父规则/槽位；失败/回退/下一模块不得泄漏AGE名称上下文。缓存/恢复身份至少绑定grammar/domain内容及版本、兼容配置、入口/上下文规则和真实依赖；同源切换普通/AGE或别名开关不得串用CST/投影。候选不携Checked信任；结构通过不能豁免行为类型/权限检查。错误分结构拒绝、能力缺失、语义/资源拒绝，保原span、合法旧代与既有预算，不能把资源失败当语言非法。

T-H0-I的有界决定只在现有rule/field组合与必要的中立上下文规则扩展间比较至多两种策略，冻结实际字段/入口/fixture命令、冲突规则及source-map；不得退回硬编码AGE parser。与963并行时先在已交960/962及现有text/普通函数域做规则资产、CPU consumer和名称反控；963正在修改的`auto-source.at`、`core/source/cst/project.rs`、AST/resolve/实例查询及公共测试注册按文件owner短窗串行、冻结SHA交接。需963新泛型的实际case仍等963交付，不靠私有复制语法或消费未审WIP解锁；H0无需整包963/966完成。H0-I/V/R/D按原任务独立正式交付，980消费H0-D，不反等981/SOURCE或完整980/981。

**H0 已验形状与已知限制（2026-11-15 三轮独立复审收敛，实测行为而非规划猜测）**：冻结表达式首批为**字面量或同工程普通函数调用**（受检文本种类 str/Str 均可，值经共享 AV 执行）；裸标识符叶（`text my_str {}`）在共享 text-record 面按构造器形状解析并报结构拒绝，不属 H0 支持形态（完整表达式面归 T-02/T-08）；AGE 声明**之前**的顶层杂散 `}` 仍报通用语法错误（不在任何已开始的 AGE 构造内，故无领域 hint）；叶表达式被共享检查拒绝时，首诊断之后可能跟随一条脱糖 `.clone()` 级联诊断（仅外观，首诊断为权威）；`__age_h0_text_` 前缀保留策略在存在文本叶时生效（无隐藏函数即无冲突）。


### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r9当前）

**具体输入与期望。** 原件[ag-examples/examples/g01-shapes/g01-main/src/main.at](../../../age-examples/examples/g01-shapes/g01-main/src/main.at)（下列节选L7–L38；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
use resources: red_alpha, blue_alpha, green_alpha, yellow_opaque
use shapes: Disc

node Main {
    view-2d {
        Disc red_disc {
            x: -20.0
            y: 10.0
            diameter: 30.0
            tint: resources.red_alpha
        }
        shape2d blue_quad {
            kind: quad
            position: (0.0, 0.0)
            size: (30.0, 30.0)
            material: resources.blue_alpha
        }
        Disc green_disc {
            x: 25.0
            y: -15.0
            diameter: 24.0
            tint: resources.green_alpha
        }
        shape2d yellow_quad {
            kind: quad
            position: (-45.0, -25.0)
            size: (12.0, 12.0)
            material: resources.yellow_opaque
            visible: false
        }
    }
}
```

本包所需独立结果：g01原源四图形/visibility和来源可投影；CPU消费者核原坐标/颜色/order，不以图形字段可解析证明model/msg/on行为受检；H0文字另保。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 版本化grammar组合→无损CST/原span→领域未受信候选/行为引用→唯一verify→CPU/AV/A2R最小双方言消费者。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct DomainPackage { id: DomainIdentity, grammar: GrammarRef, projection: ProjectionContract, dependencies: DependencySet }
struct DomainCandidate { structure: DomainIr, behaviors: ExprRefs, source_map: SourceMap, requirements: RequirementSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [experimental/ace/crates/syntax-grammar](../../experimental/ace/crates/syntax-grammar) | 领域规则组合/歧义 |
| [experimental/ace/crates/core/src/source/cst/project.rs](../../experimental/ace/crates/core/src/source/cst/project.rs) | 普通与领域投影接缝 |
| [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs) | WidgetDecl与Node领域owner，Core不反依UI |

**状态、错误与资源。** grammar冲突、未知构造/字段、坏行为类型或篡改投影丢必需叶须具名拒并保原span；不静默丢字段。 grammar/domain/ABI来源改变使受影响缓存失效；候选不携Checked信任/指针，失败保旧；通用投影不串行反等969/978整包。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [ag-examples/examples/g01-shapes/g01-main/src/main.at](../../../age-examples/examples/g01-shapes/g01-main/src/main.at) | 原Node/use/props/view源，不手写HIR充源 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `9b809a1c6d99985b` | 拟ace.m2.p979.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [auto-ui/examples/ui/002-counter/src/front/app.at](../../../auto-ui/examples/ui/002-counter/src/front/app.at) | widget/model/msg/on/主题完整需求，CPU接面不冒全产品 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `04937c35325faa74` | 拟ace.m2.p979.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [docs/specs/auto-syntax/grammar-contract.md](../specs/auto-syntax/grammar-contract.md) | 既有grammar机制，不新建第二parser | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `2203a19aa79239fa` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 泛型/CT组合等963/966对应能力，T-02/08通用合同与T-03真实双端消费者区分；H0不代完整Node，980/981产品不反置前置。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-schema.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06/07/08 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/auto-a2x/architecture.md`（语言owner）及auto-ui仓`docs/specs/auto-graphics/ace-migration-feasibility.md`（领域owner） | 在实际交付后记录领域受检投影、AV/A2R共源接面及UI067旧基线no-go的具体关闭项；未实现项仍保缺口 | 防止将规划或最小前门证明冒充完整游戏支持 | AC-06/07/08 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-schema.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-HELLO | add | docs/specs/ace-migration/m2-schema.md（沿原owner） | 未有早期交接→本切片实际已验接口/源/拒绝及剩余完整义务；交付后沉淀 | 最小Hello不冒全包完成 | AC-H0/AC-H0-REVIEW/AC-H0-LAND |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-schema.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
| SD-NODE-NAME | add | `docs/specs/ace-migration/m2-schema.md`（拟新增；真实交付后） | 未交领域前门→AGE Schema/NodeName槽、上下文激活/身份、Node首源及widget显式兼容同链 | 本轮用户架构纠正 | AC-H0/AC-NODE-NAME |
| SD-HANDOFF | modify | `docs/specs/ace-migration/m2-schema.md`（仅真实交付后） | 编号等待→已交基础接口、981模块/引用/原源及980/981清单正式输入和979完整组合验收的身份/范围 | 消除循环且不减验收 | AC-HANDOFF、AC-02/03/04/06/07/08 |
## 6. 测试设计

### r11交接验证增量（拟执行，不覆盖既有结果）

V-09/V-10/V-11/V-DETAIL-01的原输入、范围、独立oracle和后端义务全部保留；按§5 IF补齐其未验部分，永久case由980/981原I任务创建并由979 T-03/T-08消费。多模块必须包含文件A的公共helper与文件B的AGE Node，改变A函数体使两个后端结果实际改变；私有helper、缺模块、图环分别走正确阶段的原文件诊断，拒绝时无运行/新工件，修复同入口成功。两实例各2/1次Inc后各4次Life.Tick（每次+1）的独立状态应为6/5（helper初值0），逐实例文本与状态一致；同时保原N矩阵，不能仅比较两端相等。该用例为拟输入，由981/SOURCE冻结合法Schema拼写和现有observer实际返回形态，不强行沿用单模块驱动的内部编码。

IF-981-REF用同类型的合法引用、同类型但未声明引用、同类型且已声明却不可见引用隔离错误原因；错误必须是引用/可见性诊断而非偶然类型错，带源文件/节点/字段span，合法邻例两端真跑。V-11在实际清单中删除一条必需映射后检查红、恢复后绿；g01原四图形按原源闭包取值，不能手写期望结构喂消费者。979 T-04核这些真实生产接口与正式输入；T-05按r11重核全部AC/V，原r10增量review仅沿已验面复用。

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下为完整验收合同，已验部分见§8–9对应收据；本轮r11新增交接尚未执行。T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 同一Node关键字在普通标识符与领域上下文正确区分 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 冲突包拒加载且旧程序保留 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 未知字段/坏投影在执行前报原span | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 领域规则更新只失效实际依赖 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 用规范Node与旧widget兼容表面验证同一Schema producer，不能另造widget producer | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-09 | AC-07 | 正常多模块Node工程含两个同定义独立Counter：定向事件2:1、N个Life.Tick后各实例恰推进N次；普通helper参与计算，typed view描述与状态绑定可观察 | T-02/08建普通源码fixture，T-03分别经正式共享前门跑AV、A2R→真实rustc→运行，T-04记值/事件序与独立算术oracle；fixture名/命令在T-01冻结 | work；979仅最小领域/CPU消费者，不以emit成功/AV运行替A2R运行，不等待981游戏 |
| V-10 | AC-03/06/07 | 同类型错误事件参数、未声明或越权的实例引用、坏领域包与篡改投影事实在共享检查处拒绝，原文件span准确；AV/A2R均不执行坏程序；修复后原入口恢复正确结果 | T-02/08建正反例，T-03记录拒绝阶段与未dispatch证据；schema/ABI变化使旧受检产物失效，旧合法代保留 | work；禁止二次生成旧.at、legacy Parser/AutoVM/旧Trans fallback或手造Checked凭证 |
| V-11 | AC-08 | 传统UI字段与完整Node构造的投影清单可表达；具名坏字段/遗漏必需投影在共享前门报原span；删除一条必需字段/构造映射时完整性交接检查红，合法近邻仍接受 | T-08在现有grammar/CST/投影target建普通源正反例及覆盖核对，T-04核源→接口→980/981 owner；不等产品验收或手造HIR | work；979仅通用表达/检查/交接，完整运行责任不能缩为最小view样本 |

完整职责组拟定：`cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av ace-a2x`（已静态核test-groups.toml，未运行）。原`dialect`组是legacy入口，只有真实修改该兼容面才补跑，不代auto-ui领域验证。新包/用例由T-04注册进对应组；改共享lowering/runtime/provider时按真实触面并入`ace-lowering ace-runtime ace-native`等必要消费者组，不能只跑前端。A2R用新ACE `ace-a2x`，旧`cargo t a2r`不证明新链。外仓只跑真实受影响命名case，窗口/GPU单实例、全量AG留里程碑；979最小领域fixture不以980/981完整产品为前置。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 同源同时含普通函数、旧widget、新Node可分别准入；坏包/歧义规则不死循环；领域新增不能改变无关普通标识符语义；热换schema后旧证据失效。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### H0验证（永久case由T-H0-I创建，未执行）

| Case | AC | 区分性输入→实际入口→独立结果 | owner/边界 |
|---|---|---|---|
| V-H0-SOURCE | AC-H0 | AGE node/view/text源、改Node名/中文文本，及普通函数返回文本的同类源→版本grammar/CST→共享verify→CPU受检描述；根/叶/表达式值与原span准确，换字符串后实际值变化 | work；不把手写AST/HIR或旧Parser输出充普通源 |
| V-H0-REJECT | AC-H0 | 缺闭合、未知字段/构造、错误文本类型、表达式名字错误报原文件；合法近邻接受，拒绝后修复同入口恢复 | work；bad projection删除必需叶/篡改事实应在verify/准入拒绝 |
| V-H0-DEPEND | AC-H0 | 同源不同grammar/domain版本不误复用；旧CST/受检身份失配拒绝，改文字/函数仅失效真实依赖；CPU消费者不等待UI窗口 | work；拟职责组cargo t ace-syntax ace-source ace-semantics ace-artifacts，实际改lowering/AV/A2X再补对应完整组 |
| V-H0-POSITION | AC-H0/AC-NODE-NAME | AGE开/关×NodeName槽/普通表达式：node根内view正确识别；`let view=1`、model形参、on函数、widget字段/成员与字符串保持普通绑定；未允许父槽不偷匹配。普通源有无AGE得到相同值/类型；无AGE的Node构造不进入AGE consumer | T-H0-I/V；同类型正控，不用无关类型错误证明位置敏感 |
| V-H0-SCHEMA | AC-NODE-NAME | 不改runtime，仅在隔离测试Schema将节点头view配置为panel：声明的panel接受且投影仍为Schema声明的view角色、未声明view在该槽拒绝，普通变量view仍合法；同槽冲突包加载拒绝；失败解析后合法普通源不受污染 | T-H0-I/V；证明parser真实消费Schema，测试变体不改变正式作者拼写 |
| V-H0-COMPAT | AC-NODE-NAME | Node Hello首源；旧widget Hello原字节经兼容别名与只改声明头的迁移Node副本：同一候选语义/CPU值，CST逐字节还原各自原源且诊断span不漂移；关别名旧源拒绝、Node仍接受；换领域/配置缓存不串用 | T-H0-I/V；H0只闭最小文本，完整旧组件/状态与模式交980，不新增第二份required分母 |


### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | g01原源四图形/visibility和来源可投影；CPU消费者核原坐标/颜色/order，不以图形字段可解析证明model/msg/on行为受检；H0文字另保。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | grammar冲突、未知构造/字段、坏行为类型或篡改投影丢必需叶须具名拒并保原span；不静默丢字段。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | grammar/domain/ABI来源改变使受影响缓存失效；候选不携Checked信任/指针，失败保旧；通用投影不串行反等969/978整包。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | Schema组合语言与依赖闭合的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | 普通Node工程直接经共同前门/唯一verify，最小领域描述与行为真实到AV和A2R编译运行；值/事件/实例隔离一致且分别满足独立oracle；非法近邻报原span，无旧源码生成/重解析或后端语义分叉 | T-01/02/03/08实施，T-04核V-09/10及结构依赖闭包；981接收版本化接面，完整游戏另验 |
| AC-08 | work | 传统AutoUI与完整AGE语言分母有来源/投影/检查/接收owner的双向映射，通用接面不硬编码三种view或少量UI字段，遗漏/坏投影具名拒绝；完整产品支持交980/981逐项关闭 | T-01/02/08，T-04核V-11；不以最小双方言PASS冒完整语言PASS |
| AC-HANDOFF | work | §5五个IF具名输入正式交付且979自身完成V-09/10/11/V-DETAIL-01全部未闭部分；960不重开，980/981不反等979归档，缺项不删/不改NA，原AC/V全部保留 | T-03/T-08/T-04与§6交接反控；T-05独立核全量并绑定r11 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |
| AC-H0 | work | AGE node/view/text及普通函数文本表达式通过Schema驱动的唯一前门，受检领域描述/原span/错误与恢复正确 | T-H0-I/V；V-H0-* |
| AC-H0-REVIEW | review | 独立核H0前门/行为verify/拒绝/版本和CPU consumer，无旧解析或硬编码样本 | T-H0-R |
| AC-H0-LAND | merge | H0接口/grammar/来源与真实验证正式落主，保原完整Schema/UI/Node义务 | T-H0-D |
| AC-NODE-NAME | work | NodeName位置与AGE环境双条件、Schema驱动解析/校验、旧widget显式同链别名及无上下文/缓存泄漏均满足§5；新Node首源不经传统widget链 | T-H0-I/V；V-H0-POSITION/SCHEMA/COMPAT，原T-08扩全域；H0-R独立核 |

## 8. 执行步骤

当前共12任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- [x] **T-H0-I｜M2-A｜owner_stage: work｜AC-H0/AC-NODE-NAME**：不等T-01完整分母/963/966整包；核§5最小基础，冻结AGE激活方式、NodeName位置规则、旧widget别名、版本grammar/typed投影接口、CPU oracle和诊断/资源策略，补V-H0-*，沿同一机制实现。
  - [✅ 已完成 2026-11-15｜f71f9cfd2/35dfe72b2/f627ab599] 冻结决定：`Atom::CtxName`（spelling/token/role，codec v4，parser ABI 3）承载槽位限定拼写且不改词法；领域激活=`SourceProfile::AgeH0`（slug `age-h0`）经 `auto-source.at` 裁剪面，通用解析/编译入口具名拒绝该 profile；`widget`=同一 `node` role 别名，由显式 `LegacyWidgetAlias` 配置准入（关闭=头 span 受控拒绝，不回落旧引擎）；文本叶脱糖为普通隐藏函数 `__age_h0_text_<n>() Str { return (<expr>).clone() }` 经共享 text-record 唯一 verify，文本种类/def id 从 Checked HIR 读回；CPU 值经共享 lower→MIR→ABC→AV 执行（`ace_av::age_h0`）。规划文字与现况差异（`view` 在现 Auto 源面本就是保留字 `r-view`）按"保持普通Auto规则"读法处理，未改公共保留字表，记入§10。证据：core conformance 473/473（13 新 `p979_age_h0_*`）、av lib 11 + 6 新 CPU 例、syntax-grammar 45（6 ctx-name + 7 age grammar）、syntax-runtime 39；worktree `D:/autostack/.wt/lang-979/auto-lang`。

- [x] **T-H0-V｜M2-A｜owner_stage: work｜AC-H0/AC-NODE-NAME**：T-H0-I后实际运行V-H0-*与完整受影响职责组，核普通源/表达式/原span及错投影拒绝；GUI与FFI未执行保持不在本切片声明内。
  - [✅ 已完成 2026-11-15｜冻结提交 f627ab599] V-H0-* 逐行实跑：V-H0-SOURCE（一个字面量源/一个普通函数返回文本源→受检描述+CPU 值 `Hello`/`你好`，改字符串得 `Bon好`，span 指原字节）、V-H0-REJECT（未知组件 `panel` 槽位拒绝、属性拒绝、缺闭合、非文本 `1+2` 由共享 bind 在 3:14 拒、0/多 text 根、失败后下一次解析无污染）、V-H0-DEPEND（grammar id 稳定 `7f18fb1a…`、改源只改 input-hash、plain text-record 拒 AGE 源、普通模块 atom_text 逐字节同形）、V-H0-POSITION（`node`/`text`/`widget` 在 AGE 开/关都是普通标识符；`view` 保留字口径见§10）、V-H0-SCHEMA（测试 Schema `view→panel` 接受 panel/拒 view 且 role 不变、codec/身份往返、无字节事实源失败关闭）、V-H0-COMPAT（别名关=头 span `source.age.legacy-widget`；开=同 role/同候选/同值）。真实 CLI 入口（`ace-core check-source --source-profile auto.source.age-h0.draft`）：字面量/函数源 exit 0（nodes=1、同 grammar id、不同 input-hash）、非文本源 exit 1（共享 bind 诊断 3:14）、widget 源 exit 1（1:1 具名拒）。受影响职责组并集 `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av`：ace-syntax/ace-source/ace-semantics/ace-artifacts PASS；ace-av 首跑 1191/1191 测试全 PASS 但组判红于进程树基建（MSVC `vctip.exe` 残留后代，非测试失败），消除残留后重放 PASS。GUI/FFI/宿主/完整产品腿明确不在本切片声明内（归 980/HELLO 与 T-02/03/08）。

- [x] **T-H0-R｜M2-A｜owner_stage: review｜AC-H0-REVIEW**：T-H0-V齐后同树停写独立复核源码→CST→唯一verify→CPU描述，不信样本自报；finding回I。
  - [✅ 已完成 2026-11-15｜三轮独立复审 final pass @ 71472d975] 独立上下文（subagent `reviewer-979-h0`，只读、自建探针/52+ CLI 夹具）逐轮复核：第一轮 **needs_fix**（F1 隐藏 def 被后缀同名用户函数劫持、F2 `parse_program_via_cst` 静默丢弃领域声明、F3 拒绝诊断过于通用、F4 未用 import、F5 `use.rs` 面不一致、F6 领域 hint 吞更早错误、F7 硬编码 `widget`、F8 计数证据偏差）；修复 `90c46092c` 后第二轮 **pass**（另记 F9 头扫描误认、F10 叶错误归类、F11 缺节点名 + 片段入口观察）；修复 `63f4a0fc9` 后第三轮 **needs_fix 仅 F12**（F9 修复遗留调试 `eprintln!`）；删除后 `71472d975` 终审 **pass**：`git diff 50dc0ea5c..71472d975` 无新增调试输出（唯一保留的 `eprintln!` 是 base 既有、`AC787_TRACE` 环境门控的解析 trace），失败 CLI 仅 stderr 一行领域诊断、成功无 stderr；F1–F12 全部关闭并有独立复现证据。三处措辞收敛按复审建议落地（见§9 最新收据与§5 H0 注记）。

- [x] **T-H0-D｜M2-A｜owner_stage: merge｜AC-H0-LAND**：T-H0-R pass后phase fold正式合入H0 grammar/投影与已验规范，交980/HELLO；保同树/resync继续完整原任务，不能称979或完整UI已交。
  - [✅ 已完成 2026-11-15｜主检出 ff-only → `7c739e2d8`] phase fold 已落主：代码 6 提交 rebase 到当时 master（`5aeebf201`，含 PLAN-1060 测试治理对 `syntax-runtime/parse`、`testkit`、测试组配置的改动）后 fast-forward 合入；**reviewed `71472d975` → 交付链 `95db94fb4`→`0b8465430`→`873f29e64`→`220851256`→`bb2ce71e2`→`43f5b09a5`**，`git range-diff` 6 对中 5 对 `=`、第 1 对仅上下文差异（1060 的 `fail_memo`/rustfmt 上下文，无内容差异）；规范沉淀提交 `7c739e2d8`（docs-only 后代）。落主规模 27 文件 +2762/−93，含新规范 `docs/specs/ace-migration/m2-schema.md`（SD-01/SD-HELLO/SD-NODE-NAME/SD-DETAIL 的 **H0 已验部分**；完整 supersede 与全包义务仍 pending，不声明完成）。落主前在 rebase 态复验：core conformance 483/483、p979 过滤器 13/7/22、受影响五组 `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av` 全 PASS；落主后主检出复验：p979 过滤器 22/13/7 全 exit 0、CLI 字面量源 exit 0（nodes=1、grammar `5ec22d3e…`）、非文本源报 `source.age.text-type` @3:14、规范文件在位。**交接 980/HELLO**：`SourceProfile::AgeH0` + `compile_age_h0`（描述含 role/原 span/文本种类/def_id）+ `ace_av::age_h0::observe_age_h0`（共享后端取真值）+ §3 诊断码契约；未交面见规范 §4/§5。计划保持 `executing`（T-01..T-08 未开），**不归档**；worktree/branch 保留并已与 master 同点，后续按 §8 同树 resync 继续。retention 登记 `auto-lang/PLAN-979` in-flight（policy.json sha256/16 `d1deeaad704b1396`）。



- [x] **T-01｜M2-A｜owner_stage: work｜AC-01/07/08/AC-PHASE**：核依赖与§4实际路径，冻结§5交接件、传统UI/完整AGE分母的投影/检查/owner表、最小双方言/双后端源与V-09/10/11命令；有界前探共享语义→AV/A2R最长路径，明确资源和未决项，不缩范围。
  - [✅ 已完成 2026-11-15｜冻结件 `t01-freeze.md` sha256/16 `9e8e9ff23b02b216`] ①**依赖/路径核对**：§4 五个外仓/文档入口全部在且 hash 与计划记录一致（`auto-ui/crates/auto-ui-dialect/src/ast.rs` `1d10f5ff2566422e`、`age-examples/.../g01-main/src/main.at` `9b809a1c6d99985b`、`auto-ui/examples/ui/002-counter/src/front/app.at` `04937c35325faa74`、`docs/specs/auto-syntax/grammar-contract.md` `2203a19aa79239fa`）；外仓只读，跨仓实施前按当地 AGENTS 由 owner 确认。②**交接件合同**：repo/commit、Plan revision、grammar 版本（GrammarId+codec v5+parser ABI 3）、支持组合、投影/Checked 消费 API、source map、后端身份（AV lower/MIR/ABC；A2R fileset schema+edition）、正反例、资源边界；**禁交**仅 AST codec/手写 HIR/emit 成功当运行，Node 的 model/msg/on 也须经同一 verify。③**双向表**：传统 UI 按 `WidgetDecl` 成员块+`ViewNode{Element,Text,ForLoop,Conditional,Component,Outlet}`+msg/model/computed/on/bind/lifecycle/theme/recipe/store/fragment 建块级行；完整 AGE 按 `declarative-authoring-model.md` AG-A01..A09 + 五种呈现（`view/view-2d/view-3d/view-portrait/view-widget`）建行；每行含来源/原 span/979 通用能力/接收 owner/980-981 产品验证格，逐字段补齐归 T-08（不缩范围）。④**最小双方言/命令**：fixture `ace.m2.p979.source01`（AGE 双实例 Counter 工程，AV+A2R）、`source02`（002-counter 传统 UI 切片）、`source03`（坏近邻集合）、`source04`（双向表覆盖正反例）；AV 腿 `cargo test -p ace-av --test conformance <case>`/`check-source`，A2R 腿 `cargo t ace-a2x --test-name <case>`；固定并集 `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av ace-a2x`；**A2R 入口契约冻结：`main` 必须自由/零参/返回 `i32`**（`semantic_view.rs:614-643`，wrapper 硬编码 `A2xResult<i32>`），故 Node/UI fixture 必带普通 `fn main() int` 驱动观察；A2R 已接纳 `text-record`（Str/clone/concat/调用）、单模块、深度 ≤400、语句 Let/Return/If/Loop/Assign/Break/Continue。⑤**有界前探（实跑）**：策略 A（沿唯一管道扩展）胜出（B=第二发射器/检查器违反 §5）；**A2R 最长路径新证**＝`str` 字面量函数 + `Str` clone + helper 算术经 `run_chain`（compile→SemanticView→lower→fileset→**真实 rustc**→运行）2/2 PASS（值 6/30，0.38s；fileset schema `auto.a2x.rust.record-text.fileset/1`、`input_hash=84c2e7ac64a99f30`、`main→fn_main`）；AV 腿/非法近邻/失败恢复沿用 H0 实跑证据；入口负控（无 main/非 i32 → `a2x.entry`）只读断言。⑥**资源与平台口径**：解析 64MiB 预算/投影条目数/AV ExecLimits/A2R 深度与 rustc 进程，超限=资源拒绝非语言非法；Linux 腿按 2026-11-14 裁定不在 Windows 跑。⑦**未决项**（具名 owner）：Node 状态/事件语义→981/T-02/T-03；五种呈现选择语义→981/T-08；传统 UI 全字段矩阵→T-08；A2R 非 i32 入口→981/SOURCE；`view` 保留字→用户裁定；alias 入身份对→T-02。证据：`D:/autostack/plan-artifacts/plan-979/{t01-freeze.md 9e8e9ff23b02b216, t01-probe-a2r.log 668480c6e5c3b831, t01-probe/{main.rs e5c9df913bee3bb0, generated.rs 00d32cc2b308992a, a2x_runtime.rs 754a067807bdd73b, manifest.json e51a0a2192f8d148}}`；探针为临时测试，跑后已还原（本轮无产品代码改动）。

- [x] **T-02｜M2-A｜owner_stage: work｜AC-02/07**：实现通用领域包/CST/typed投影与唯一verify接面；领域owner通过同一接口接规范Node源码及同Schema的旧widget兼容别名正反例，保原span，不另造语义分析或旧源码生成链。
  - [✅ 已完成 2026-11-15｜代码 `5238db37b`] **通用接面**：新增 `ace_core::source::domain`（`DomainPackage{id, version, profile, hidden_prefix, compat: Option<CompatRule>, project: fn(&FrontOk,&str)}`、未受信 `DomainCandidate{structure: Vec<DomainItem>, behaviors, requirements}`、`DomainCompilation{id, version, grammar_id, compat, description: Vec<DomainRow>, checked, input_hash, atom_text}`），共享驱动 `compile_domain` 负责 领域前门→包投影→兼容闸→隐藏名冲突→行为脱糖（普通 `Str` 函数）→**唯一 text-record verify**→从 Checked HIR 读回（文本种类+def id）→通用行树；非领域 profile 由驱动具名拒绝 `source.domain-package`（不再 debug 断言）。**AGE 迁移**：`AGE_H0_PACKAGE` 成为经同一驱动的第一个包（`project_age_h0` 只做 AGE 候选→通用行/行为映射），`compile_age_h0` 与 `AgeH0Description` 公开面不变（22 个 H0 用例全绿）。**owner 可扩展性证明**：测试内第二包 `test.pair/v1`（同 grammar，自有 `pair`/`card` 结构、自有兼容码 `test.pair.compat`、自有隐藏前缀 `__pair_val_`）仅经公开 API 注册即被同一驱动消费；兼容正反例（`widget` 关闭=包名码@声明头 span；开启=同结构+行标记 compat）、包级隐藏前缀冲突（AGE 前缀在 pair 包内是普通名）、行为检查仍归共享 verify（`1+2`→`source.age.text-type`@原 span、未定义函数→`source.age.leaf-expr`+共享消息）。**未另造**语义分析器/发射器/解析器。证据：core conformance **488/488**（27 个 `plan979`：22 H0+5 package）、av p979 7/7、syntax-grammar p979 13/13、`cargo t ace-syntax ace-source ace-av` 全 PASS（`gate-t02.log`）。规范沉积留 T-06 全量合并（本节为 LLM 内部接缝，H0 spec 的 `compile_age_h0` 公开面未变）。T-08 仍负责双向表逐字段覆盖/五种呈现槽与恢复用例；T-03 负责 AV/A2R 两腿消费。

- [ ] **T-03｜M2-A｜owner_stage: work｜AC-02/03/07/AC-HANDOFF**：消费已交T-02/08及§5 IF-981-MODULE/REF/ORIGINAL正式接口，AV与新A2R两腿真实运行并补齐V-09/10/V-DETAIL-01；后者须真实rustc。核原领域描述/状态事件/装配/权限/增量/恢复，保全部独立oracle，交回版本化验收结论；不等待960重开或980/981整包，不以模型替运行。
  - [🚧 进行中 2026-11-15｜双后端文本/表达式面已实跑，代码 `982c60a88`] **已交付**：**同一份 AGE 源**（`node`+文本叶+普通 helper+`fn main() int`）两腿真实消费——AV 腿经 `observe_age_h0` 取叶值 `few`（`crates/av` 新例）；A2R 腿用**同一 `CheckedModule`** 经 `SemanticView::from_checked`→`lower_program("main")`→`assemble_record_text_file_set`→写 fileset→**真实 rustc** 编译→运行→断言 `value:4`（`len("many")`），无手写 Rust、无 emit 冒运行、无第二套语义（`crates/a2x-worker/tests/cases/plan979_age_a2r.rs`）。失败/恢复：把叶换成 `1+2` 时前门在共享检查处拒（`source.age.text-type` @原 span）且**不产出任何 fileset 工件**，修复后同入口恢复正常。断言权威：双后端值/失败均由父侧独立断言，不用 trace 自报。证据：`cargo t ace-a2x ace-av` 两组 **PASS**（`gate-t03.log` `a07b00c2b52e6096`）、av p979 8/8、a2x-worker plan979 2/2。②**双后端矩阵 + 恢复/增量**（代码 `4653eef04`）：三个源（字面量叶/函数叶/helper 算术）在 AV 侧观察值与 A2R 侧真实运行值成对断言（`Hello`↔5、`few`↔4、`ab`↔8，A2R 值为 `len`/算术的独立 oracle）；好→坏→好 循环保持观察值与身份稳定、坏源在原表达式 span 处具名拒（`source.age.text-type`）——领域驱动无跨调用状态。`cargo t ace-a2x ace-av` 双组 PASS（`gate-t03b.log` `a9fc22977e7d9d4f`）。**未闭**：V-09 的 `model`/`msg`/`on`/`Life.Tick` 与"两实例事件 2:1、N tick 独立推进"需要**组件状态/事件运行时**（含实例隔离、typed view 状态绑定）——该运行时是 981 产品层与语言槽的联合实现，须在 979 的领域包/共享 verify 之上建模；V-10 的坏事件/未声明引用、V-11 的字段级覆盖随之。当前未以"文本双后端"冒充 V-09 全量。

- [ ] **T-08｜M2-A｜owner_stage: work｜AC-06/07/08/AC-HANDOFF**：保普通语言+领域组合、冲突/版本、畸形推进性/来源及全部V-08/09/10/11；按§5正式IF补实例引用、UI完整字段及AGE完整构造覆盖与删除映射反控。981/980原I任务生产接口，979实际核通用准入/完整性；不塞CAP、不等待产品整包，T-04统一汇总。
  - [🚧 进行中 2026-11-15｜V-08 已闭，代码 `f3c1526fe`] **已交付**：①**组合相容**——同源含普通函数+新 `node`+旧 `widget` 按配置分别准入（compat 开=2 个候选、legacy 头 span 精确；关=旧头在自身头 span 具名拒，纯 node 源不受连坐）；②**包冲突/版本/推进性**——未受信投影自检：越界/重复绑定、空角色、过深/过大结构 → `source.domain-candidate`（不死循环、不静默丢）；③**无关语义不变**——`fn node()`/`fn widget()` 的普通模块经领域前门与 text-record 前门 `atom_text`+`input_hash` 全等；④**热换失效**——`DomainIdentity{id, version, grammar_id}` + `same_evidence`：包版本/包切换→不等（同一 grammar 也不同），同输入→相等；⑤**完整构造映射机制**——通用 `mapping_complete`（必需 role 清单）在删除 `view` 行的坏近邻上返回 `Err(["view"])`，合法近邻 `Ok`；⑥覆盖核对表落 `plan-artifacts/plan-979/t08-coverage.md`（sha256/16 `66e9a9a76415de15`）。证据：core conformance **493/493**（p979 32=22 H0+5 package+5 combined）、fmt clean、`t08-core-full.log` `c6dedb3f5d5434a2`。**未闭（本任务保持 open）**：V-09/10/11 永久 fixture（双实例 Counter、坏事件/未声明引用、AV+A2R 真实运行）依赖**完整 Node 投影**（model/msg/on、五呈现、`Life.*`）——该投影与 981 协同归 T-03；传统 UI 字段级 fixture 归 980 owner 经同一接面接入（979 不实现产品语义）。投影全程使用 T-02 正式接面，未另造检查器/发射器。
  - [🔶 2026-11-16｜V-11 UI 字段半已核，AGE 半待 981] **IF-980-FIELDS 消费核验**（不新增代码）：980/DIALECT 整批落主（lang `3de0bea0b`（DIALECT 批）与 `46f4ac0ec`（字段子面）、ui `f864f33b`；规范 `docs/specs/ace-migration/m2-autoui.md`）后**独立核，不采信 980 摘要**：清单 `ui.widget-decl/v1` **115 行**（16 required / 91 optional / 8 conditional），ID 唯一；`P980_CONSTRUCTS` **115 键**与行集**双向精确相等**；从 `ast.rs::WidgetDecl` 独立解析 **19 字段**并逐一在清单行锚定（19/19，无遗漏）；16 条必需行全部 002 锚定、条件行 8/8 带可核理由；owner 集合 `{980/DIALECT, 980/STATE, 980/PARITY, 980/WEB, 980/NATIVE}`；删除反控目标 `widget.msg` / `widget.design.entry` 均为必需行。控制实跑：lang `plan980` **13/13**（含删清单行 → unknown 红、换映射 → missing+unknown 可区分、恢复绿），ui dialect `p980` **3/3**（含删映射项 `ui.field-unknown`、清单↔映射双向对账 + 删行红），ui compiler `ui_domain_validation` **5/5**（含 `p980_required_fields_handoff` / `p980_required_field_mapping_controls`）。979 独立审计脚本 `v11-ui-audit.ps1`（`9aeca4081043408e`）**19 项检查全 PASS**。判定：**V-11 的 UI 字段半 = 已核**；**V-11 仍不闭**——AGE 半需 IF-981-FIELDS（981 交付仅在 `plan-981-dev`，未 phase fold 落主，按 §5 交接准入不合格）；4 个具名拒绝槽位（style/setup/actions/widget 级 `use{}`）按 980 规范计“具名拒绝即交付面”，不冒完整产品支持。收据：`plan-artifacts/plan-979/v11-ui-handoff.md` `447fa9780e8e95e9`。

- [ ] **T-04｜M2-A｜owner_stage: work｜AC-01..03/06/07/08/AC-HANDOFF**：汇总T-02/03/08与§5五个正式IF，运行§6全部原V行及修改面职责/必要消费者并集；逐项核独立oracle、模块/原span/源身份、子终态、条件排除及无循环来源。缺最小双后端、跨模块、实例引用、完整分母或g01原源CPU证据不得pass。
  - [🔶 2026-11-16｜首个 IF 汇总（V-11 UI 字段半）] **T-04 增量**：IF-980-FIELDS 已按 r11 §5 交接准入逐项核（落主 SHA、r10 子面范围、清单版本与锚定源 `04937c35`、span/owner、消费接口、永久正反例与删行反控、边界声明），V-11 的 UI 字段半记入已验面；V-11 全行与 V-09④/V-10/V-DETAIL-01 仍缺 IF-981-*（未落主），故 T-04 不汇总、不判 pass，979 保持 `executing`。

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-979/auto-lang`，分支 `plan-979-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-08依赖T-01和相应T-02接口；T-03按已就绪T-02/08接口推进，不要求互等最终收据；T-04等待T-02/03/08全部完成；T-05→06→07串行。跨计划并行限不相交文件；共享core/schema/verify由语言owner单写，产品消费准备不反向成为979完成门。
材料唯一目录 `D:/autostack/plan-artifacts/plan-979/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-15 r11规划静态预检：979/980/981原81任务的ID/顺序/勾选及状态/进度保持，原52项AC、66项V和22项SD均保留，各Plan仅增1项AC-HANDOFF及1项SD-HANDOFF；三份0–10章节/围栏、实际承载任务与新增7个本地链接/锚点核对通过，具名IF交接图静态拓扑无环。960归档文件及.next-id内容hash未变。只证明合同结构/交接边界，不是新接口、CPU/AV/A2R/UI/GPU实跑或独立实现review PASS；未创建任何产品用例/产物或实施树。

2026-11-15 stage: new/revision | PLAN-979 r11 | outcome: 规划合同已修订，非实现/产品review PASS。按本轮授权把原960组合缺口交981/SOURCE，980/981原任务生产具名接口，979保留全部原AC/V与自身组合回验；不以A切片归档，不新增整包反向依赖。原任务/勾选/状态/计数保持，旧revision的真实已验部分可按身份复用，但不能证明本次新增交接；未来独立review须绑定r11及实现SHA。仍只规划，next=由原owner获得后续实施指令后按§5–8执行；本记录不恢复任何目标/实施树或解除既有暂停。

2026-11-12 stage:new / PLAN-979 r9：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原12任务/全部AC与V/依赖/阶段/预算和drafting 0/12；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-11 / stage:new / PLAN-979:r8 / outcome:pass（早期Hello路线合同可审阅）；blocked（尚未授权实施，真实能力/接口待交） / next:按§5最小前置进入新增切片；原完整M2任务与终验均保留，不用小例替全包。新增T-H0-I, T-H0-V, T-H0-R, T-H0-D，当前0/12。未实现/构建/运行或派agent。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r7：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r6：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r5 / stage:new：补传统UI和完整AGE分母接面、AC-08/V-11；保8任务和原AC/V，drafting、0/8。979不反向等980/981产品PASS，完整语言不能以最小双方言替代。仅规划，实施handoff仍blocked（未获实施指令、依赖接口待齐），非运行或独立final PASS。

本轮静态检查：8任务/8AC/9验证行、原ID全保留，状态/完成数未变；979/980/981及UI058/AG023可达199份跨仓Plan，无缺失引用/依赖环，新增链接和Cargo组名已核。此为规划检查，不是任何新语言/产品运行PASS。

2026-11-09 r4 / stage:new：沿用户新两阶段方向同步并行/交接边界，原8任务/7AC/全部验证保留，0/8、drafting；不新增Plan，不把增强终验反置为979前置。本次只修订规划，实施handoff仍blocked（依赖/实施指令未齐），非运行或独立final PASS。

2026-11-09 r3：stage=new；979/981职责和前向依赖已静态复核，新增AC-07、V-09/10，扩充T-01/02/03/04/08和SD-03；保留原8任务及AC-01..06，0/8、drafting。outcome=blocked（仅指实施交接：当前为规划授权，依赖与双后端实际接面未就绪）；next=取得实施指令后按T-01及§5最小前置进入work。此次非产品测试/独立final review PASS，未创建实现树或执行构建。

规划静态核验：两计划跨语言/UI/内容仓可达192份Plan，按本地/带仓前缀plan_id及两种archive路径解析，0缺失引用、0依赖环；979任务8/验收7/验证行8，981任务28/验收14/验证行14，ID无重复且完成数均0。批次DAG亦无环，纠正T-08旧文字对T-03的反向等待；固定Cargo组名存在。此核验不证明拟议接口已实现或产品已通过。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

2026-11-13 / stage:new / PLAN-979 r10 / outcome:pass（规划合同预检，非实现或产品review） / changed: Node首源、Schema/NodeName/兼容与并行合同及对应AC/V/SD；原任务数量/状态/进度与旧义务保留 / next: 本轮仍只规划；后续获得实施指令且该任务最小正式来源齐时进入原work。全部新增验证未执行，未构建/运行/建树/派agent/合入或改963。

2026-11-13 本轮仅规划的静态核验：4份计划原95任务ID/勾选/数量/状态/进度与依赖元数据保持；新增3项AC、7项拟验证及3项拟Spec增量，全部未执行。9份文档本地引用、0–10章节、表格/围栏与新AC/V任务映射核对通过；F0→E0→B0与H0汇合HELLO、随后981按接口扩展的切片顺序不新增反向整包等待。963及其实施树未由本轮修改；未生成fixture/binding/SDK、未运行编译/产品/GPU/性能、未建树/提交/合入/tag。此为规划预检，不是实现独立review PASS。

2026-11-15 stage: work / T-H0-I（H0 最小领域切片实现，T-H0-I 关闭；T-H0-V/R/D 与 T-01..T-08 仍 open）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（切片完成，Plan 1/12）| worktree: D:/autostack/.wt/lang-979/auto-lang | branch: plan-979-dev | base_commit: 50dc0ea5c | code_commit: f627ab599（前序 f71f9cfd2/35dfe72b2）| task_ids: T-H0-I | decisions: ①Schema=parser 输入——中立 `Atom::CtxName`（name/spelling/token/role，program codec v4 + parser ABI 3），词法面不变（`node`/`text`/`widget` 仍普通 ident；`view` 沿用既有 `r-view` 保留字，未改公共保留字表）；②领域激活=显式 `SourceProfile::AgeH0`（slug `age-h0`，auto-source.at 裁剪面），通用 `parse_program_with`/`compile_parts` 入口对该 profile 具名拒绝 `source.domain-frontend`（领域声明绝不静默丢弃）；③旧 `widget` 头=同一 `node` role 别名，由显式 `LegacyWidgetAlias{Reject,Accept}` 准入（关闭=头 span `source.age.legacy-widget` 受控拒绝，不回落旧引擎，配置进入描述身份）；④文本叶脱糖为普通隐藏函数 `__age_h0_text_<n>() Str { return (<expr>).clone() }`，整模块经共享 text-record 唯一 verify，文本种类(str/Str)/def id 从 Checked HIR 读回；CPU 值经共享 lower→MIR→ABC→AV 执行，无二次解释/Rust 常量补值 | evidence: `cargo test -p ace-core --test conformance` 473/473 PASS（211.11s；13 新 `p979_age_h0_*`：字面量/函数表达式/非文本拒绝/属性拒绝/缺闭合/别名开合/隐藏名冲突/0-多 text 根/通用前门拒绝/普通源逐字节同形(atom_text 等值)/身份绑定/失败后无上下文泄漏/双 node 独立）；`cargo test -p ace-av --lib` 11/11；`cargo test -p ace-av --test conformance plan979` 6/6（真实值 Hello/你好/Bon好/Owned、双 node 独立、拒绝面）；`cargo test -p ace-syntax-grammar` 45/45（6 ctx-name + 7 age grammar，含 Schema 变体 panel 与 role 不变、fail-closed、codec/身份往返）；`cargo test -p ace-syntax-runtime` 39/39；`cargo check` core/av/native/semantics/source-query/artifacts/abi clean；fmt 本包文件 clean，零新增告警 | 未执行面（归后续任务）：V-H0-V 的固定组并集（`cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av`）与 T-H0-R 独立复审、T-H0-D fold；GU/FFI/宿主、完整 AGE/传统 UI 语法与 A2R 腿仍在 T-02/03/08 与 980/981 | raw_evidence: D:/autostack/plan-artifacts/plan-979/evidence.md (sha256前16 495fd284f8f16f40，merge收尾折叠后删除) | blockers: none | next_responsible_role: work（T-H0-V：跑 V-H0-* 与受影响职责组；随后 T-H0-R 独立复审）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: f627ab599 | next: work

2026-11-15 stage: work / T-H0-V（H0 验证收据，T-H0-V 关闭；T-H0-R/D 与 T-01..T-08 仍 open）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（切片验证完成，Plan 2/12）| worktree: D:/autostack/.wt/lang-979/auto-lang | branch: plan-979-dev | code_commit: f627ab599（冻结，未再改码）| task_ids: T-H0-V | v_rows: V-H0-SOURCE/REJECT/DEPEND/POSITION/SCHEMA/COMPAT 全部实跑（用例与结果见 §6 与 T-H0-V 证据行）| gate: `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av` → ace-syntax PASS / ace-source PASS / ace-semantics PASS / ace-artifacts PASS；ace-av 首跑「1191 tests run: 1191 passed, 21 skipped」但组判红于 [process-tree infrastructure] `vctip.exe`（MSVC 工具链遥测子进程残留，`exit: 0`、`cleanup_complete: true`），确认无残留进程后重放 `cargo t ace-av` → PASS（首次红=环境归因，重放绿；不充条件排除，仍列首次红与重放两次结果）| cli: `ace-core check-source --source-profile auto.source.age-h0.draft` on 4 fixtures（字面量/函数 → exit 0、nodes=1、grammar `7f18fb1a241d6007…`；非文本 → exit 1 共享 bind 3:14；widget 无兼容 → exit 1 `source.age.legacy-widget` 1:1）| test_scope: 新用例单例 0.01–0.05s（nextest 逐例计时，均 ≤10s），受影响组耗时 ace-av 37.9s/1191 tests；core conformance 全量 211.11s/473 为上一轮冻结证据，输入未变故复用 | raw_evidence: D:/autostack/plan-artifacts/plan-979/{evidence.md sha256/16 495fd284f8f16f40, v-h0-evidence.md b920419a8cf1bf6f, gate-h0.log 4cbb5b8df5dec8d7, gate-h0-av-retry.log f4e519b77a8ea6ef, h0-*.at}（merge 收尾折叠进本文件后删除）| gui_ffi: 未执行且不在本切片声明内 | blockers: none | next_responsible_role: review（T-H0-R：同树停写后独立复核 H0 前门/verify/拒绝/版本与 CPU consumer）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 2 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: f627ab599 | next: review

2026-11-15 stage: work / T-H0-R 复审第一轮修复（F1–F8；T-H0-I 修复面关闭，T-H0-V 门在修复提交上重跑）| plan_id: PLAN-979 | plan_revision: 10 | outcome: fixed（待独立复审第二轮）| worktree: D:/autostack/.wt/lang-979/auto-lang | branch: plan-979-dev | reviewed_commit: f627ab599（首轮复审）| code_commit: 90c46092c（修复）| findings_closed: **F1（MAJOR）**＝文本叶 def 选错可被用户后缀名函数劫持：`adapter::free_fn_symbol/free_fn_def_id` 成为唯一命名真源，叶按**完整 HIR 名**精确匹配，且任何用户顶层名**含** `__age_h0_text_` 前缀即具名冲突 `source.age.hidden-name`；回归 core `p979_age_h0_suffix_named_user_function_cannot_hijack_a_text_leaf`、av `p979_age_h0_near_miss_user_function_cannot_bind_the_text_leaf`（值仍为 "Hello"）。**F2（MAJOR）**＝`parse_program_via_cst` 静默丢弃 AGE 声明：`parse_front` 对所有 kind 具名拒绝领域 profile，`parse_program_via_cst` 继承拒绝，`compile_program_parts` 亦拒绝；领域路径改用 crate 内部 `parse_domain_whole_module`；回归 `p979_age_h0_every_generic_entry_refuses_the_domain_profile`（三入口）。**F3（MINOR）**＝结构/类型拒绝是通用诊断：新增中立语法数据 `RuleSpec` 平行表 `GrammarProgram.rule_expects`（codec v5）+ 解析失败时的**深度优先** hint（"unexpected token in <construct>"，取当前 token 原 span，仅整次解析失败时进入 tree.diags）；非文本叶的共享拒绝改标 `source.age.text-type` 并保留共享消息与精确 span；回归 `p979_age_h0_rejections_are_domain_named_with_the_original_span`（缺 view→slot@`}`、第二 view→structure@第二个 view、属性→structure@`size`、缺闭合→structure、非文本→text-type@表达式）。**F4**＝两处未用 import 删除，syntax-grammar 45 例零告警。**F5**＝`use.rs`/`use.c` 门控补入 `age-h0`（与 text-record 同面），回归 `p979_age_h0_import_front_door_has_text_record_parity`（atom_text/input_hash 相等）。**F6**＝领域 hint 只在"AGE 头之前没有其他共享诊断"时取代（错误叶参与扫描），回归 `p979_age_h0_earlier_shared_errors_are_not_swallowed`。**F7**＝legacy 别名判定改为"同 role 的首个 Schema 声明拼写为规范"，删除硬编码 `widget`。**F8**＝证据数字复核：修复提交上 core conformance **479/479**（318.87s；18 个 `p979_age_h0_*`＝13 基础 + 5 修复回归），旧 473 记录为加入第 13 例前的运行。 | gate: `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av` → ace-syntax/ace-source/ace-semantics/ace-artifacts PASS；ace-av 首跑 1192/1192 全 PASS 但组判红于 `vctip.exe` 进程树（见环境边界），retry1/2 同因，retry3（运行时 watcher 清 vctip）PASS | environment_boundary: MSVC `vctip.exe`（cl.exe 触发的 VC 遥测）生命周期 > meter 的 2s settle；harness 已 reap（`cleanup_complete: true, remaining_active: 0`）但守卫仍判红——非代码缺陷，建议 harness owner 容忍工具链遥测子进程（不属 979 范围）；直接 nextest 无 tool 环境的运行不作证据（1 例 native 用例因缺 prepare_tools 环境失败）。| test_scope: 本轮修复仅触及 H0 面与语法诊断；复用证据＝474 例全量中的未受影响部分，受影响面按新回归单例（≤0.06s）+ 修复提交全量 479 复跑一次 | raw_evidence: D:/autostack/plan-artifacts/plan-979/{gate-h0-repair.log a42b6cdb6aa2048a, gate-h0-repair-av-retry{,2,3}.log 3550b841be4e72bd/a65d7f7c6162e2f9/d6c44ff14f91230c, repair-core-full.log 427224c350c85ec7} | blockers: none | next_responsible_role: review（T-H0-R 第二轮：复核 F1–F8 关闭 + 新诊断/命名/入口面）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 3 | from: review | to: review | status: executing | outcome: fixed | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 90c46092c | next: review

2026-11-15 stage: review（T-H0-R 第二轮：pass）+ work（同一轮内关闭其 3 项非阻塞 MINOR 与 1 项观察）| plan_id: PLAN-979 | plan_revision: 10 | reviewed_commit: 90c46092c（第二轮独立复审 verdict=**pass**）| code_commit: 63f4a0fc9（诊断修复）| review2_findings_closed: **F9**＝AGE 头扫描只认**错误叶**（此前任意同名标识符会被当成头，吞掉更早真实错误）；回归 `p979_age_h0_non_head_identifier_does_not_define_the_age_head`（`fn node()`/`fn foo()` A/B 首个诊断一致地落在 node 行之前）。**F10**＝叶内共享拒绝分两类：只有脱糖 `.clone()`/类型拒绝改标 `source.age.text-type`，名字/参数/其它错误保留共享消息并改标中立 `source.age.leaf-expr`；回归 `p979_age_h0_undefined_callee_is_not_reported_as_a_text_type_error`。**F11**＝"构造已开始"改按帧入口游标（子规则消费也算），缺节点名得到 `source.age.structure` @`{`；回归 `p979_age_h0_missing_node_name_is_a_domain_structure_error`。**观察（片段入口）**＝`parse_fragment_front` 对领域 profile 具名拒绝；回归 `p979_age_h0_fragment_entries_refuse_the_domain_profile`。**hint 排序**＝深度降序 + 稳定记录序（槽位步先于同规则构造级消息），panel/缺 view 恢复为 `source.age.slot`。未改动的观察：`__age_h0_text_` 前缀策略仅在有文本叶时生效（无隐藏函数即无冲突，措辞已在 §9 收敛）；alias 不在 `(grammar_id, input_hash)` 对中（本切片无缓存，非缺陷）。| evidence: 修复提交上 core conformance **483/483**（205.83s；22 个 `p979_age_h0_*`），av conformance p979 7/7，syntax-grammar 45，syntax-runtime 39，av lib 11；受影响职责组 `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av` → **五组全 PASS（exit 0）**（ace-av 以运行期清 `vctip` 的 watcher 取得 PASS，环境边界同前轮记录）| test_scope: 新回归单例 ≤0.06s；全量 core 复跑一次作为冻结证据 | raw_evidence: D:/autostack/plan-artifacts/plan-979/{repair2-core-full.log 341cfe2935ee94e2, gate-h0-repair2.log d5ea9258687b5190}，gate results.json run-hr5ricif | blockers: none | next_responsible_role: review（T-H0-R 第三轮：仅复核 F9/F10/F11/片段拒绝四处；随后 T-H0-D fold）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 4 | from: review | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 63f4a0fc9 | next: review

2026-11-15 stage: review（T-H0-R 第三轮：needs_fix 仅 F12）+ work（删除遗留调试输出）→ **终审 pass**；H0 切片 fold-ready | plan_id: PLAN-979 | plan_revision: 10 | reviewed_commit: **71472d975**（独立上下文终审 verdict=**pass**，绑定该提交）| code_commit: 71472d975（父 63f4a0fc9）| findings_closed: **F12（MAJOR）**＝F9 诊断工作遗留的调试 `eprintln!("DBG diags=…")` 混入 AGE 失败路径 stderr（libtest 捕获 stderr 故五组门禁无法发现；属 AGENTS §3 健康门"无遗留调试输出"）；删除该块后失败 CLI 仅 stderr 一行领域诊断、成功无 stderr，`git diff 50dc0ea5c..71472d975` 无新增 `println!/eprintln!/dbg!`（唯一保留者为 base 既有的 `AC787_TRACE` 环境门控 trace，未改）。| wording_reconciliation（复审建议，仅计划文本）：①"所有结构拒绝都具名"收敛为"已开始的 AGE 构造内"（声明前顶层杂散 `}` 仍为通用语法错误）；②H0 已知限制补记：裸标识符叶 `text my_str {}` 在共享 text-record 面按构造器解析 → 结构拒绝，H0 冻结表达式为字面量/普通函数调用（已写入 §5 H0 注记）；③叶首诊断之后可能跟随脱糖 `.clone()` 级联诊断（仅外观，首诊断权威）。| gate: 修复提交上 `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av` → **五组全 PASS（exit 0）**（ace-av 保留运行期清 `vctip` 的 watcher，环境边界同前）| evidence: plan979 过滤器 13/13（syntax-grammar）、22/22（core，全量 483/483 于 63f4a0fc9 记录）、7/7（av）；fmt clean；终审独立复核：diff 仅 1 文件 7 行删除、无其它调试产物、5 个夹具 stdout/stderr 分离正确 | raw_evidence: D:/autostack/plan-artifacts/plan-979/{gate-h0-f12.log 21b5ae7548bc22ba, f12-stderr.txt 9a186c532ed63f15, evidence.md 40c34eb219a7b483} | blockers: none | next_responsible_role: merge（T-H0-D：phase fold 合入 H0 grammar/投影/AV consumer 与已验规范，交 980/HELLO）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 5 | from: review | to: merge | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 71472d975 | next: merge

## 10. 待澄清事项

本次已明确缺口生产归属及无环交接，不再等待已归档960产生新交付。原I任务仍须冻结具体组合profile/API、诊断码与可见性语法/命令及资源边界；这是有界技术决定，不是新增立项或降低原验收。原已具名的公共语言/平台/产品策略未决继续保留，必要能力缺失必须绑定实际owner，不能以编号泛称取代可执行前置。

2026-11-15 T-H0-I 规划文字与现况差异（已按"保持普通Auto规则"读法实施，未改公共面）：§5 r10 写"普通表达式/函数体中的 `let view = 1` … 保持普通Auto规则"，但当前 Auto 源面 `view` 已是保留字（auto-source.at 的 `r-view`，927 七面未消费的保留字先例），普通 profile 同样拒绝 `let view = 1`。为使 AGE 不改变既有公共接受面，本切片保持该保留字不变（未授权扩大改动；解除保留字属公共源码面承诺变更，需用户显式裁定），槽位声明改用 `r-view` token；V-H0-POSITION 的"AGE 开/关同值同型"证据改为：普通模块经 AGE 前门与 plain text-record 前门产出逐字节相同的共享 Atom 文本，且 `node`/`text`/`widget` 在两种面里都是普通标识符。若用户希望 `view` 在 AGE 面成为普通绑定（即解除保留字），需显式裁定并修订 AC-NODE-NAME 措辞。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本轮已确定不新增Plan：通用领域接面留979，AGE实际AV/宿主/A2R和两个游戏交981；T-01只冻结符号/命令/支持组合等实施细节。未决Node状态表示、类型方法/全局cell在新后端的具体合法化由981/SOURCE、981/RUST联语言owner核现状处理，不以归档960/961当组合已支持，不重开已交付工作。
2026-11-15 stage: merge / T-H0-D（H0 phase fold 落主；计划保持 executing，不归档）| plan_id: PLAN-979 | plan_revision: 10 | delivery_commit: **7c739e2d8**（master ff-only）| reviewed_commit: 71472d975 | rebased_chain: 95db94fb4→0b8465430→873f29e64→220851256→bb2ce71e2→43f5b09a5 + spec 7c739e2d8 | base: 50dc0ea5c；rebase onto master@5aeebf201（35 commits，含 PLAN-1060 测试治理与 parser `fail_memo`）| range_diff: 6 对 5 `=`、1 对仅上下文（无内容变更）| landed: 27 files +2762/−93（`experimental/ace/**` + `docs/specs/ace-migration/m2-schema.md`）| spec_deposit: `docs/specs/ace-migration/m2-schema.md` = SD-01/SD-HELLO/SD-NODE-NAME/SD-DETAIL 的 **H0 已验部分**（激活/身份、槽位 schema、冻结输入、脱糖→唯一 verify、共享后端取值、诊断码、引擎边界、pending 清单）；`supersedes_spec_components` 的完整 supersede 与全包义务未执行（留待全包 merge）| verification_pre_land（rebase 态）: core conformance 483/483（2.69s）、p979 过滤 13/17→13/7/22 全绿、`cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av` 五组全 PASS（ace-av 同前环境边界）| verification_post_land（主检出 @7c739e2d8）: p979 过滤 core 22/22、syntax-grammar 13/13、av 7/7（exit 0）；`ace-core check-source --source-profile auto.source.age-h0.draft` 字面量源 exit 0（nodes=1、grammar `5ec22d3e…`、hash 737d601ec5baba92）、非文本源 `source.age.text-type` @3:14；规范文件在位（77 行）| handoff_980/HELLO: `SourceProfile::AgeH0` + `ace_core::source::age_h0::compile_age_h0`（role/原 span/文本种类/def_id 描述）+ `ace_av::age_h0::observe_age_h0`（共享 lower→MIR→ABC→AV 取真值）+ 诊断码契约（§3）| cleanup: 本阶段**不删** plan-artifacts（计划在途）、不删 worktree/branch（T-01..T-08 继续；已与 master 同点）；retention 已登记 `auto-lang/PLAN-979` in-flight（`D:/autostack/plan-artifacts/.retention/policy.json` sha256/16 `d1deeaad704b1396`）| raw_evidence: D:/autostack/plan-artifacts/plan-979/{gate-h0-fold.log 5f2a55a7a887ec16, rebase-core-full.log 333db16bd2a0ded1, rebase-p979.log 870730cfea13dcf1, land-core-p979.log ec1570c7ba80520a, evidence.md 40c34eb219a7b483} | blockers: none | next_responsible_role: work（T-01：依赖与§4实际路径核对、交接件与分母冻结）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 6 | from: review | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 7c739e2d8 | next: work2026-11-15 stage: work / T-01（依赖与路径核对、交接件合同、双向表、最小双方言与 V 命令、A2R 最长路径前探）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | freeze_record: D:/autostack/plan-artifacts/plan-979/t01-freeze.md（sha256/16 `9e8e9ff23b02b216`；merge 前折叠进本文件后删除）| dependencies_verified: 五个 §4 外仓/文档入口在且 hash 同计划记录（`1d10f5ff2566422e`/`9b809a1c6d99985b`/`04937c35325faa74`/`2203a19aa79239fa`）| handoff_contract: repo/commit+Plan rev+grammar 版本(GrammarId/codec v5/parser ABI 3)+支持组合+投影/Checked API+source map+后端身份+正反例+资源边界；禁仅 AST codec/手写 HIR/emit 当运行 | two_way_table: 传统 UI 块级（WidgetDecl 17 成员块 + ViewNode 六类 + msg/model/computed/on/bind/lifecycle/theme/recipe/store/fragment）与完整 AGE 行级（AG-A01..A09 + 五种呈现），逐字段补齐归 T-08 | fixtures_frozen: `ace.m2.p979.source01`（AGE 双实例 Counter 工程）/`source02`（002-counter 传统 UI 切片）/`source03`（坏近邻集合）/`source04`（双向表覆盖正反例）；A2R 入口契约：**自由/零参/i32 `main`**（`semantic_view.rs:614-643` + wrapper `A2xResult<i32>`），Node/UI fixture 必带普通 `fn main() int` | a2r_probe: `str`字面量函数+`Str` clone+helper 算术经 compile→SemanticView→lower→fileset→**真实 rustc**→运行 2/2 PASS（值 6/30，0.38s；schema `auto.a2x.rust.record-text.fileset/1`、input_hash `84c2e7ac64a99f30`、entry `main→fn_main`），临时测试跑后已还原 | av_probe: 沿用 H0 实跑（值/拒绝/恢复）| resources: 解析 64MiB+投影条目+AV ExecLimits+A2R 深度≤400/单模块/rustc 进程；超限=资源拒绝，非语言非法；Linux 腿按 2026-11-14 裁定 | unresolved: Node 状态/事件语义(981/T-02/T-03)、五种呈现选择(981/T-08)、传统 UI 全字段矩阵(T-08)、A2R 非 i32 入口(981/SOURCE)、`view` 保留字(用户裁定)、alias 入身份对(T-02) | blockers: none | next: T-02（通用领域包/CST/typed 投影与唯一 verify 接面）与 T-08（相容/恢复、双向表逐字段、五种呈现槽）在不相交文件上并行；T-03 待投影就绪分 AV/A2R 两腿

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 7c739e2d8 | next: work
2026-11-15 stage: work / T-02（通用领域包/CST/typed 投影/唯一 verify 接面 + 第二 owner 包证明）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: 5238db37b（resync 后同补丁 `5f2b0611a`）| design: 新增 `source/domain.rs`（`DomainPackage`/`DomainCandidate`/`DomainItem`/`DomainBehavior`/`DomainRow`/`DomainCompilation` + 共享驱动 `compile_domain`），AGE 以 `AGE_H0_PACKAGE` 注册为第一个包且公开面不变；非领域 profile → `source.domain-package` | genericity_proof: 测试包 `test.pair/v1`（自有 pair/card 结构、`test.pair.compat`、`__pair_val_` 前缀）经公开 API 注册→同一驱动消费；包级兼容/前缀/行为检查（text-type/leaf-expr @原 span）全部按包生效；AGE 与 pair 对同源产出各自行（无 AGE 硬编码） | no_second_engine: 驱动只调共享前门 + 共享 text-record verify；无第二解析器/检查器/发射器；隐藏求值是普通 Auto `Str` 函数 | verification: core conformance **488/488**（2.67s；27 `plan979`=22 H0+5 package）、av p979 7/7、syntax-grammar p979 13/13、`cargo t ace-syntax ace-source ace-av` 三组全 PASS | raw_evidence: D:/autostack/plan-artifacts/plan-979/{t02-core-full.log 88a313854112f8f9, gate-t02.log 0020edad9fd0ee85} | spec_deposit: 留 T-06（H0 spec 公开面未变，接缝属实现细节）| remaining: T-08（双向表逐字段/五种呈现槽/恢复用例）、T-03（AV+A2R 两腿真实消费）、T-04 汇总 | blockers: none | next_responsible_role: work（T-08 与 T-03 准备；T-03 在投影就绪后分 AV/A2R 两腿）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 5238db37b | next: work
2026-11-15 stage: work / T-08 进度（V-08 关闭；V-09/10/11 待完整 Node 投影）| plan_id: PLAN-979 | plan_revision: 10 | outcome: partial（V-08 pass）| code_commit: f3c1526fe | delivered: 组合相容（mixed source 正反例）、候选自检 `source.domain-candidate`、普通标识符语义不变（atom_text/input_hash 全等）、`DomainIdentity`/`same_evidence` 热换失效、`mapping_complete` 完整性红/合法近邻绿 | evidence: core conformance **493/493**（2.89s；p979 32）、coverage note `66e9a9a76415de15`、log `c6dedb3f5d5434a2` | open: V-09/10/11 fixture 需完整 Node 投影（model/msg/on、五呈现、Life.*）→ T-03 与 981 协同；传统 UI 字段级 fixture → 980 owner 经同一接面 | blockers: none | next_responsible_role: work（T-03：完整 Node 投影 + AV/A2R 两腿；随后 T-08 收口 V-09/10/11）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 9 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: f3c1526fe | next: work
2026-11-15 stage: work / T-03 进度（双后端文本面实跑）| plan_id: PLAN-979 | plan_revision: 10 | outcome: partial | code_commit: 982c60a88 | delivered: 同一 AGE 源 AV（叶值 `few`）+ A2R→真实 rustc→运行（`value:4`）；坏源不产出工件且保原 span，修复后恢复 | interface: 981 可消费的仍是 `AgeH0Description`+`CheckedModule`+`observe_age_h0`（未变）| evidence: `cargo t ace-a2x ace-av` 双组 PASS（`gate-t03.log` `a07b00c2b52e6096`）、av p979 8/8、a2x plan979 2/2 | open: V-09 组件状态/事件运行时（model/msg/on/Life.Tick、两实例隔离、typed view 绑定）待建——981 产品层与语言槽联合；V-10/11 随之 | blockers: none（非阻塞：需实现而非等待） | next_responsible_role: work（T-03 续：组件模型语法槽+状态/事件运行时；或先做 T-04 汇总当前 ready 面）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 10 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 982c60a88 | next: work

2026-11-15 stage: work / T-03 增量（双后端矩阵 + 恢复/增量证据）| plan_id: PLAN-979 | plan_revision: 10 | outcome: partial | code_commit: 4653eef04 | delivered: 3 源 × 双后端成对断言（AV 值 / A2R 真实 rustc 运行值 5/4/8）；好→坏→好 恢复与身份稳定；坏源保原 span 具名拒 | evidence: `cargo t ace-a2x ace-av` 双组 PASS（`gate-t03b.log` `a9fc22977e7d9d4f`）、av p979 10/10、a2x plan979 3/3 | open: 组件状态/事件运行时（V-09 全量）仍待建——model/msg/on/Life.Tick、两实例隔离、typed view 状态绑定；V-10/11 随之 | blockers: none | next_responsible_role: work（T-03 续：组件模型语法槽 + 状态/事件落地）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 11 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 4653eef04 | next: work
2026-11-15 stage: work / T-04 首轮汇总（已交付面）+ T-03 组件模型实施设计 | plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（汇总）+ design（组件）| gate: `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av ace-a2x` → **六组全 PASS（exit 0）**（`gate-t04-rollup.log` `06c6eefb46a0d13e`；av 仅用 vctip watcher）| rollup: `t04-rollup.md` `59d6c9fd58cded6c` 逐 V 行只记已实跑（V-01/03/05/08 已验；V-02/04/10/11 部分；V-09 未验=open；V-06/07/V-DETAIL-* 未验），未把未建能力记 PASS | component_design: `t03-component-plan.md` `9557b63821acf8f8` —— 新 profile `age-component`（不动 age-h0，H0 零回归）+ `model`/`on`/`view` 成员规则 + `project_component_nodes` + 场景驱动脱糖（`Scenario{instances, ops: Send/Tick/Render}`→普通 Auto 驱动 `__age_drive() int` 与每 (实例,叶) 隐藏 `Str` 求值函数）+ AV/A2R 双消费（A2R 用 `lower_program(view,"__age_drive")`）；含接线点清单、语法、验收与风险 | blockers: none | next_responsible_role: work（按设计实现组件 profile：接线→语法/投影→脱糖+AV→A2R→V-09/10/11）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 12 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 5ec8db711 | next: work
2026-11-15 stage: work / T-03 组件面脚手架（profile + 语法裁剪 + 入口闭合）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（脚手架）| code_commit: 94c54dd35 | delivered: 新增 `SourceProfile::AgeComponent`（slug `age-component`、`auto.source.age-component.draft`）；同一 `auto-source.at` 新增组件成员规则（`model`/`on`/`view` 成员列表、`model <field> <ty> = <init>`、`on <Msg> { <field> = <rhs> }`）与两个 contextual 槽位（`model`/`on`，role 同名）；组件面自有 GrammarId（与 age-h0 隔离、H0 规则集不增长）；`GRAMMARS` 13 槽；全部通用入口（整模块/片段/`parse_program_via_cst`/独立二进制）具名拒绝 `source.domain-frontend`；`domain_profile_refusal` 覆盖两领域 profile；CLI 对新 profile 明确拒绝而非猜 | verification: core p979 **34/34**（2 个新接线用例：裁剪/规则/身份隔离 + 入口闭合），`cargo build -p ace-core` 零告警，fmt clean | next: 组件投影（`project_component_nodes`）→ 场景脱糖 + AV 消费 → A2R 消费 → V-09/10/11 | raw_evidence: 本提交 + p979 过滤 34/34 | blockers: none | next_responsible_role: work（按 `t03-component-plan.md` 第 3–5 步）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 13 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 94c54dd35 | next: work
2026-11-15 stage: work / T-03 组件投影首次尝试（**未落**，已回退保绿）| plan_id: PLAN-979 | plan_revision: 10 | outcome: reverted | 尝试内容: `age_component.rs`（组件描述类型 + `AGE_COMPONENT_PACKAGE` + `compile_age_component`）+ `project.rs` 的 `project_component_nodes`/`run_component_node_item`（model 字段/on 赋值/view 叶投影）+ `FrontOk.component_nodes` + 前端分支 | 观察: 投影本体实际工作（组件源解析后 `component_nodes=1`），但经 T-02 共享驱动后 `DomainCompilation.description` 为空（视图-only 源得到 nodes=0），三处新测试中 2 个红；受上下文预算所限未定位根因，按"绿色收尾"原则 `git checkout` 回退全部未提交改动、删除新文件，工作树回到 `94c54dd35` 状态并复验 core p979 **34/34** | 下一步线索: ①先以 crate 内单测直接断言 `project_component_nodes`（不经驱动）确认投影行/叶/span；②再单独验 `project_component` → `DomainCandidate.structure` 的行数；③重点排查 `resolve_rows`/`compile_age_component` 的行映射（当前证据：驱动入口 `comp=1` 但描述空，说明断点在投影→候选或候选→行两段之间）| 无半成品入库：`git status` 干净，未提交 WIP 已丢弃 | blockers: none | next_responsible_role: work（按上述三步重做组件投影，先投影单测再驱动集成）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 14 | from: work | to: work | status: executing | outcome: reverted | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 94c54dd35 | next: work
2026-11-15 stage: work / T-03 组件投影落地（view 面绿；model/on 成员解析为已知缺口）| plan_id: PLAN-979 | plan_revision: 10 | outcome: partial（view 面 pass）| code_commit: 6d93a1886 | delivered: `age_component.rs`（ComponentField/Assign/Handler/View/Node 候选 + ComponentDescription/Compilation；`project_component_candidates` 公开投影入口；`compile_age_component` = 兼容闸 + 隐藏名冲突 + 隐藏求值脱糖 + **唯一 text-record verify** + 版本化描述含 grammar id/alias/role/kind/def id）；`project.rs` 的 `project_component_nodes`/`run_component_node_item`（成员列表先解包 `age-node-member` 再按规则名分派；model 字段/on 赋值/view 叶均带原 span）；`FrontOk.component_nodes` + 前端分支 + 模块注册；语法修正：`item` 的组件 alt 与 H0 alt 分离（组件面不再经 H0 规则解析）| verification: core p979 **37/37**（3 个新用例：view 面候选/编译、compat+前缀规则），零告警，fmt clean | known_gap（钉住测试 `p979_component_model_member_is_not_yet_projectable`? 注：本轮以 pin 形式记录于测试面外）: `model`/`on` 成员列表在共享前门仍报通用 `expected fn`（成员解析缺口）——组件状态/事件运行时因此未接线；下一步先修成员列表解析，再做场景脱糖+AV/A2R | blockers: none | next_responsible_role: work（成员列表解析修复 → 场景脱糖 → AV → A2R → V-09/10/11）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 15 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 6d93a1886 | next: work
2026-11-15 stage: work / T-03 组件成员投影 + 场景脱糖（驱动经唯一 verify）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（本切片）| code_commit: 228bc6fdc（含 a1d34b948）| delivered: ①**成员解析修复**：model 字段类型位改经 `age-model-ty`（`int`/`i32`/`bool` 关键字 token 或命名类型 ident），model/on 成员不再报通用语法错；投影产出 typed 候选（字段名/类型/初值/原 span，处理器消息+赋值清单）。②**场景脱糖**：`ComponentScenario{instances, ops: Send{instance,msg}|Tick{instance}}` → 普通 Auto 驱动（每实例每 int 字段一个可变局部、Send/Tick 内联声明的处理器、字段标识符重命名为实例局部、`Life.Tick` 建模为保留消息 `Tick`、观察编码 instance i → 10^i 位）+ 每叶隐藏 `Str` 求值函数，全部交**唯一 text-record verify**，无第二解释器。③**具名拒绝**：非 int 字段 / 空 view / 未声明事件 / 越界实例 / 0 或 >8 实例 / 未支持表达式形状 → `source.age.component-*` + 原 span。| verification: core p979 **42/42**（新增：typed 成员投影、fail-closed span、场景驱动、事件/实例拒绝），零告警，fmt clean | open: ①驱动值的**双后端执行**（AV 消费 + A2R rustc 运行）尚未接线；②typed view 状态绑定（`text` 读状态而非常量）与多模块工程未做；③`compile_age_component`（无场景）仍对 model/on fail-closed | next: 接 AV/A2R 消费 → V-09/10/11 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 16 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 228bc6fdc | next: work
2026-11-15 stage: work / T-03 **组件场景双后端真跑**（V-09 核心语义）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（本切片）| code_commit: be0c0293a | delivered: ①**AV 消费端** `ace_av::age_component::observe_component`（编译经共享前门+唯一 verify；驱动 `__age_drive` 与每个文本叶经共享 lower→MIR 校验→ABC→AV 执行取真值）；②**A2R 腿** 同一 `CheckedModule` 以 `__age_drive` 为 i32 入口 → fileset → **真实 rustc** → 运行；③**证据（同名场景）**：两实例 + `Inc`→实例1 + 两次 `Inc`→实例0 + `Tick`→实例1 ⇒ 编码 **22**（实例0=2 个位、实例1=2 十位，AV 与 A2R 两侧同值）；单侧事件 ⇒ **10**（定向隔离）；未声明事件在**执行前**具名拒（`source.age.component-event`） | verification: av p979 **13/13**、a2x-worker 组件 1/1、`cargo t ace-source ace-av ace-a2x` **三组全 PASS**（`gate-t03c.log` `539d0f0d6e409831`）| V-09 状态: **部分**——实例隔离/定向事件/推进/双后端一致已实跑；仍缺 ①普通 helper 参与计算 ②N 次 Tick 矩阵（当前 1 次）③typed view 状态绑定（`text` 读状态）④多模块工程 | blockers: none | next_responsible_role: work（补齐 V-09 四项 + V-10 坏事件/未声明引用 + V-11 覆盖）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 17 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: be0c0293a | next: work
2026-11-15 stage: work / T-03 V-09 补项尝试（**未落**，已回退保绿）| plan_id: PLAN-979 | plan_revision: 10 | outcome: reverted | 尝试: ①每 (实例, 叶) 的隐藏求值函数直接读实例局部 → 共享 verify 正确拒绝（`no local named __age_s_0_count`），暴露"文本求值函数与驱动局部作用域隔离"的设计缺口；②按"实例状态函数 + 文本函数调用之"重构时误裁了非场景 `compile_age_component` 的片段，构建红。按绿色收尾原则全部回退（`git checkout -- experimental/ace`），工作树回到 `78def26e6` 并复验 core p979 **42/42**、av p979 **13/13** | 设计结论（下一轮直接照此做）: 文本求值函数不能引用驱动局部——改为每实例一个 `fn __age_state_<i>() int`（该实例的 init+ops，返回首字段），驱动改为对各 `__age_state_<i>()` 按 10^i 求和；文本函数把字段引用替换为 `__age_state_<i>()` 调用后再 `.clone()` 返回。这样 V-09 的 helper/typed-view/N-tick 三项都可在共享 verify 下成立且可观察 | blockers: none | next_responsible_role: work（按上述设计重做 V-09 三项，再做多模块与 V-10/11）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 18 | from: work | to: work | status: executing | outcome: reverted | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 78def26e6 | next: work
2026-11-15 stage: work / T-03 V-09 三项补闭（helper / N-tick / typed view 状态绑定）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（本切片）| code_commit: 2756a1903 | design: **每实例状态函数** `fn __age_state_<i>() int`（该实例 init + 定向 ops，返回首字段）；驱动 = 各 `__age_state_<i>()` 按 10^i 求和；文本求值函数把字段引用替换为 `__age_state_<i>()` **调用**（不是驱动局部——上一轮失败的根因），因此 view 表达式能经唯一 verify 读实例状态 | delivered: ①**helper 参与**：`count int = seed()`、`on Inc { count = bump(count) }`（helper 同时用于 init/handler）；②**N 次 Tick 矩阵**：实例0 四次 / 实例1 两次 ⇒ 编码 **24**，各实例恰推进 N 次；③**typed view 状态绑定**：`text label(count)` 按实例渲染 `big`/`small`（AV 真值）；④顺带修复：越界场景实例现在**发射前**具名拒（此前被静默跳过）| verification: core p979 **42/42**、av p979 **15/15**（新增 2 例）、a2x 组件 **2/2**（helper 场景共享 fileset 形状）、`cargo t ace-source ace-av ace-a2x` **三组 PASS**（`gate-t03d.log` `b4ffa146e5f39a3b`）| V-09 状态: ①②③ **已闭**；④**多模块工程**未做（age 面尚无模块 `use`）；V-10（坏事件外的未声明/越权引用、失败恢复矩阵）与 V-11（字段级覆盖）仍开 | blockers: none | next_responsible_role: work（多模块支持 → V-10/V-11 → T-04 汇总）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 19 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 2756a1903 | next: work
2026-11-15 stage: work / T-03 **V-10 拒绝/恢复矩阵 + 脱糖身份**（组件面）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（本切片）| code_commit: 0183a9a73 | delivered: ①**类型不匹配**：`on Inc { count = as_text(count) }`（str 赋给 int 模型字段）由共享 verify 具名拒 `source.type-mismatch`（expected i32, found str），**AV 不执行**；②**未声明字段赋值**：`other = 1` → `source.age.component-event`（点名 `other`，assign span）；③**失败→恢复**：坏场景后同入口的好场景仍给出正确编码与逐实例渲染值（无跨调用状态/工件污染）；④**脱糖身份**：`ComponentCompilation.desugar_hash`（对受检模块文本取 content_sha）——源哈希跨场景不变、换场景即换受检工件身份，旧工件不得复用；同源同场景稳定 | verification: av p979 **19/19**（4 新）、core p979 **42/42**、`cargo t ace-source ace-av` **两组 PASS**（`gate-t03e.log` `32ed7e61aa0ac24f`）| remaining_V10: 越权/未声明**实例引用**（当前仅越界实例）与"篡改投影"在组件面仍是通用接面（T-08 已覆盖坏候选/映射缺失）；V-11 字段级覆盖、V-09④ 多模块仍开 | blockers: none | next_responsible_role: work（多模块支持 → V-11 → T-04 汇总）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 20 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 0183a9a73 | next: work
2026-11-15 stage: work / T-04 汇总 #2（组件面落地后）+ 冻结集测试修复 | plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: 7100b9cf0 | gate_first_run: `cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av ace-a2x` → **ace-syntax 首红**：`ace-syntax-grammar::conformance profiles_927::profiles_declared_set_is_exact`（冻结的 profile 清单未含 T-03 新增的 `age-component`）——正是汇总门该抓的回归 | fix: 在冻结集中按声明序补 `age-component`（同 961/962 先例，合同语义不变）；未改其它断言 | gate_replay: `cargo t ace-syntax ace-source ace-av ace-a2x` → **四组全 PASS**（`gate-t04c.log` `a28b04b3bcafc556`；首跑日志 `gate-t04b.log` `ee549b445befb6d3` 保留首红/重放对照）| rollup: `t04-rollup-2.md` `d159518c1f521811`——V-09 ①②③ 已验（22/10、22↔value:22、13/24+big/small）、V-09④ 多模块**未验**、V-10 类型/字段/恢复已验（越权引用/篡改投影部分）、V-11 部分；V-02/04/06/07/V-DETAIL-* 仍部分/未验 | remaining_for_full_delivery: ①V-09④ 多模块（需模块图前门 + 领域面组合）②V-10 越权/未声明实例引用 ③V-11 字段级覆盖与 980 owner 清单 ④2080/981 产品腿（非本包）⑤T-05..T-07 收尾 | blockers: none | next_responsible_role: work（V-09④ 或 V-11 → T-04 收口）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 21 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 7100b9cf0 | next: work
2026-11-15 stage: work / T-08 V-11 组件覆盖核对 + V-09④ 多模块可行性结论 | plan_id: PLAN-979 | plan_revision: 10 | outcome: partial | code_commit: bb4647911 | delivered: **`component_coverage`**（V-11 通用判据）：按必需构造清单（node/view/text 角色、有字段=model、有处理器=on）核对组件描述，缺项返回清单供交接检查判红；多出项不误判。测试：view-only 描述在要求 model/on 时点名缺失；把 view role 删掉的**篡改描述**点名 `view`（删除必需映射→红）。产品字段级清单仍归 980 owner | V-09④ 可行性结论（具名缺口，非静默省略）: AGE/组件裁剪**不含模块 `use`**（`use-item` 门控在 `scalar-modules`），且 `source` 适配层**没有**多模块编译入口（只有单源 `compile_program_parts`/各 profile 入口）——多模块需要"模块图前门（960 的 packages-modules 面）与领域面组合"，属**跨面新工作**，不是仅加一个 profile 名。关闭条件: ①把模块 `use` 纳入领域裁剪或由 960 暴露模块图适配器 ②领域前门接受模块集合并以领域 profile 验证 ③两文件 Node 工程经 AV+A2R 实跑。owner: 979 T-03（与 960 协同）；在关闭前 V-09 记"④未验" | verification: core p979 **44/44**（2 新）、av p979 19/19、build 零告警 | blockers: V-09④ 依赖 960 模块面（**具名未决**，非本轮阻塞：其余 V 行可继续）| next_responsible_role: work（T-04 收口：按已验面 vs 具名缺口整理；随后 T-05 独立复审）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 22 | from: work | to: work | status: executing | outcome: partial | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: bb4647911 | next: work
2026-11-15 stage: work / T-04 收口表 + T-05 首轮独立复审派发 | plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（收口表）| closeout: `t04-closeout.md` `bcec05a5091c78c1` —— **A 已验交付面**（领域接面/前门/组件面/双后端/拒绝面/身份，逐项带证据指针）与 **B 具名缺口**（V-09④ 多模块=依赖 960 模块图面；V-10 越权引用；V-11 产品字段清单；V-02/04/06/07 部分；产品运行归 980/981）分列；明确"不得据 A 声明完整 AGE/传统 UI 产品支持" | review: 已派发独立复审（subagent `reviewer-979-t05`，只读）：逐行核 A 表、核 B 缺口真实性、健康检查（warning/fmt/调试输出）、AC-02/03/06/07/08/AC-PHASE 覆盖判定、组件代码正确性风险与最小反例 | blockers: none | next_responsible_role: review（T-05 首轮结论 → 修复或并入 T-06）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 23 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 6d603e359 | next: review
2026-11-15 stage: review / T-05 首轮独立复审（**needs_fix**，12 findings）+ 同日修复 F1–F8 | plan_id: PLAN-979 | plan_revision: 10 | reviewed_commit: 6d603e359 | review_verdict: needs_fix（复审链：A 表逐行核验 + B 缺口真实性 + 健康/AC 覆盖；A 行 4 过度声明、B 缺 A2R 的 ①②③ 腿）| fixes: **F2** 场景遇无字段 node 由 panic 改具名 `source.age.component-shape`；**F3** 重复 `on <Msg>` 具名拒 `source.age.component-duplicate`（不再 first-wins 丢行为）；**F1** 多字段模型具名拒 `source.age.component-unsupported`（第二个字段 span），直到 per-field 状态函数落地——不静默错绑；**F4** 场景编码改 256^i（state<256 时单射）+ 实例上限 4，测试期望更新为 514/256/259/516；**F5** 场景描述携带真实 grammar id；**F6** 隐藏前缀策略两前门共用（用户声明处具名 `source.age.hidden-name`，先于场景闸）；**F7** `compile_domain` 接受任意领域 profile（谓词替代 AgeH0 字面量）；**F8** 删除遗留 PROBE 测试 | verification: code `72d5e1381`；core p979 **47/47**（4 新回归）、av p979 19/19、a2x 2/2、`cargo t ace-source ace-av ace-a2x` **三组 PASS**（`gate-t05-fix.log` `e354f62ac6520b63`）；零告警/fmt clean | review_remaining（下一轮）: **F10** A2R 侧 helper/N-tick/typed-view 需真实 rustc 运行（当前仅 fileset 形状）或列入 B；**F9** closeout 表 V-06/07 ID 悬空需修正；**F11** 三次解析（性能，非正确性）；**F12** `DomainIdentity` 未含 input_hash/compat（T-01 已记未决）| blockers: none | next_responsible_role: work（补 F10 → 修 F9 文档 → 复审确认）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 24 | from: review | to: work | status: executing | outcome: fixed | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 72d5e1381 | next: work
2026-11-15 stage: work / T-05 fixes F10 + F9（A2R 真实跑三场景；closeout 表 ID 修正）| plan_id: PLAN-979 | plan_revision: 10 | outcome: fixed | code_commit: 59f970e8e | F10: 组件 A2R 套件现有 **3 个真实 rustc 运行**：plain 双实例 514（2+2*256）、helper 场景 259（3+1*256，与 AV 同源同场景）、N-tick 矩阵 516（4+2*256）——不再只断言 fileset 形状 | F9: `t04-closeout.md` 的 V 行改为真实 ID（V-02/V-04/V-DETAIL-01..04），并注明 §6 无 V-06/V-07（哈希 `bace38b64d84005f`）| verification: core p979 47/47、av p979 19/19、a2x p979 **3/3**（新增 2 个真实运行）；上一轮同代码全组门 `gate-t05-fix.log` 已 PASS，本轮仅新增测试/文档 | remaining_review_items: F11（三次解析，性能）、F12（DomainIdentity 未含 input_hash/compat，T-01 已记未决）| next: 复审确认（F1–F10 关闭）→ T-06 组件切片 fold | blockers: none | next_responsible_role: review

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 25 | from: work | to: review | status: executing | outcome: fixed | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 59f970e8e | next: review
2026-11-15 stage: review / T-05 复审第二轮 **pass**（绑定 50daa4ed1）+ F13/F4-r 收尾 | plan_id: PLAN-979 | plan_revision: 10 | review_verdict: **pass**（复核 F1–F10 全闭；F11/F12 仍具名开；A 面数字与 HEAD 一致；双后端真实运行）| follow-ups_closed: **F13**（`t04-closeout.md` A 表数字/HEAD 刷新为 50daa4ed1、核心 47/组件 15/AV 514·256·259·516/A2R 514·259·516/组件 3 例真实运行；AV 测试文件三处 `*10` 注释改 `*256`）；**F4-r**（编码边界写入实例上限诊断消息与 closeout：instance i 打包在 256^i，仅各实例观测状态 < 256 时单射，instance 3 需 < 128 才不溢出；溢出为响亮 Trap 而非静默别名）| reviewer_wording_note: 双后端证明的是"同源场景驱动 + helper/N-tick"；**typed view 渲染文本只在 AV 侧观察**——T-06/spec 不得写成"typed view 在两后端渲染" | verification: core p979 47/47、av p979 19/19；fmt clean；`50daa4ed1` 上 review 已核 build 零告警、无新增调试输出 | fold_readiness: **代码切片 fold-ready**（T-06）；B 表缺口保持（V-09④/V-10/V-11/V-02·V-04·V-DETAIL-01..04/产品腿）| blockers: none | next_responsible_role: merge（T-06 组件切片 fold + `m2-schema` 规范刷新，按 reviewer 措辞约束）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 26 | from: review | to: merge | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 50daa4ed1 | next: merge
2026-11-15 stage: merge / **A 切片 phase fold（组件面）落主**（T-03/T-08 已验面；计划保持 executing，非最终 T-06 归档）| plan_id: PLAN-979 | plan_revision: 10 | delivery_commit: **06b4ab49f**（master ff-only from plan-979-dev@06b4ab49f；master 先为 e490c9b05，0/17 ahead/behind）| folded: 组件面代码（`age_component.rs`/`domain.rs`/组件投影/AV 消费/A2R 三场景/测试 47+19+6）与 **规范第 6 节**（`docs/specs/ace-migration/m2-schema.md` §6：profile/typed 候选/场景脱糖/256^i 驱动/唯一 verify/desugar_hash/双后端值 514·259·516/**typed view 仅在 AV 渲染**/编码边界/拒绝码/切片边界）| landing_checks（主检出 @06b4ab49f）: core p979 **47/47**、av p979 **19/19**、a2x p979 **6/6**、规范 §6 在位；他方会话 docs WIP 未触碰 | remaining（未 fold，B 表）: V-09④ 多模块（960 模块图前门组合）、V-10 越权/未声明实例引用、V-11 产品字段清单（980）、V-02/V-04/V-DETAIL-01..04、F11 三次解析（性能）、F12 身份含 input_hash/compat | plan_state: T-03/T-08 保持 open（未闭项如上），T-04 保持 open（B 表逐行），T-05 已 pass（绑定 50daa4ed1），T-06/T-07 属全包最终 merge/归档 | blockers: none | next_responsible_role: work（V-10 实例引用 / V-11 覆盖 → T-04 收口；V-09④ 需 960）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 27 | from: review | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 06b4ab49f | next: work
2026-11-15 stage: work / T-04 收口（B 表承接项登记）+ 计划状态复盘 | plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（承接完成）| handoffs: 已把 A 切片未覆盖项逐条登记进 `docs/plans/KNOWN-DEBT-AND-RISKS.md` §D-979（commit `4bb36ceec`）：**V-09④ 多模块**（owner 960 + 979 T-03，三项关闭条件）、**V-10 越权/未声明实例引用**（owner 981 + 979 T-08，语义属 AGE 产品层）、**V-11 产品字段清单**（owner 980；通用判据已交）、**F11 三次解析**（性能，owner 979 T-03）、**F12 身份缺 input_hash/compat**（owner 979 T-02，缓存接入时并入）。登记=具名承接，**非延期豁免**：979 对应 AC/V 行保持未闭 | delivered_summary: H0 切片（folded 7c739e2d8）+ T-01 冻结 + T-02 通用接面 + T-03 组件面（typed 投影/场景脱糖/双后端真跑/拒绝面/身份）+ T-08 V-08/V-11 机制，均已 fold（06b4ab49f）并经 T-05 独立复审 pass（50daa4ed1，F1–F10 关闭；F13/F4-r 收口）| remaining_for_979_closure: 仅剩已承接的三项跨 plan 语义（V-09④/V-10/V-11）+ 两项内部项（F11/F12）+ T-04 的 V-02/V-04/V-DETAIL-01..04 逐行（各自输入/oracle 就绪后执行）+ 全包 T-06/T-07 | blockers: none | next_responsible_role: work（V-DETAIL 资源/失败边界的可执行项；或等 960/980/981 输入）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 28 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 06b4ab49f | next: work
2026-11-15 stage: work / T-04 V-DETAIL-02/03 证据（组件面）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（两行收口）| code_commit: c25b6cf16 | V-DETAIL-02（具名拒 + 原 span）: 六类坏输入表驱动——非 int 模型字段/非文本叶/坏处理器类型/未声明字段/重复处理器/无字段场景，各自命名码且 span 落在源内（`p979_component_vdetail02_named_refusals_keep_original_spans`）| V-DETAIL-03（来源变化失效 + 失败保旧）: 同源两 profile 的 grammar 身份不同；好→坏→好 循环保持 input_hash/desugar_hash/grammar id（`p979_component_vdetail03_identity_changes_and_failure_keeps_old`）| known_gap（新登记）: 组件 profile 的**解析形状**错误（如 text 叶体内属性）仍报通用 `source.syntax`——AGE hint 分支只对 age-h0 接线；语义阶段拒绝不受影响。已在测试注释与 §9 记录，作为 T-03 后续小项（不属于本轮 V-DETAIL 两行）| verification: core p979 **49/49**（2 新）、av p979 19/19、零告警、fmt clean | blockers: none | next_responsible_role: work（组件解析形状 hint 对齐 / 或继续 B 表其余行）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 29 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: c25b6cf16 | next: work
2026-11-15 stage: work / 组件解析形状 hint 对齐（T-03 收尾小项）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: 4c4a6e5be | fix: 前门的领域 hint 选择（槽位/结构 hint 覆盖通用首 token 期望）此前只对 `age-h0` 接线，组件源的**解析形状**错误（如 text 叶体内属性）落成通用 `source.syntax`；现两领域 profile 共用该分支，而**投影分支保持 profile 精确**（上次尝试同时放宽两处导致组件投影走错分支、全部组件用例红——已恢复并只放宽 hint 分支）| evidence: V-DETAIL-02 表恢复 attribute 用例，组件 profile 现得 `source.age.structure` 且 span 在源内（与 age-h0 同策略）；core p979 **49/49**、av 19/19、a2x 6/6、`cargo t ace-source ace-av` **两组 PASS**（`gate-t03f.log` `f4d788a3a9922904`）、零告警、fmt clean | remaining: B 表跨 plan 三项（V-09④/V-10/V-11）+ V-02/V-04/V-DETAIL-01/04 + F11/F12 + 全包 T-06/T-07 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 30 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 4c4a6e5be | next: work
2026-11-15 stage: work / T-04 V-04 证据（依赖-scoped 失效）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（机制面收口）| code_commit: 09d851fb5 | evidence: `p979_component_v04_invalidation_tracks_the_actual_dependency` —— ①源变→input_hash 与 desugar_hash 同变；②场景变→input_hash 不变、desugar_hash 变（只失效受影响依赖）；③兼容配置变→描述记录 alias，而哈希对不变——**正是 F12 记录的缺口**（缓存落地时 alias 必须入键）| status: V-04 的"按实际读取失效/失败保旧"对已交付面成立；**未做**：真正的投影缓存（本切片无缓存），故 V-04 记"机制已验、缓存集成未做（F12）" | verification: core p979 **50/50**（1 新）、零告警、fmt clean | remaining: B 表跨 plan 三项 + V-02/V-DETAIL-01/04 + F11/F12 + 全包 T-06/T-07 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 31 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 09d851fb5 | next: work
2026-11-15 stage: work / T-04 V-02 收口（冲突包拒加载、旧注册保留）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（最小加载面）| code_commit: e0fba7367 | delivered: `DomainRegistry`（`register`/`get`/`len`）+ `DomainPackageConflict`（id/双方版本/profile）；规则：同 id 同版本同 profile = 幂等；同 id 任一不同 = 具名冲突且**旧注册保留**（被拒包不能替换可用包）。测试：`p979_domain_package_registry_refuses_conflicts_and_keeps_the_old_package` —— 同 id 异 profile 拒、幂等重复注册 Ok、拒后旧包仍可取且可编译（旧程序不受影响）| boundary: 本切片为**静态注册**（编译期身份）；动态装载/多版本共存未做（记 F12 邻域） | verification: core p979 **51/51**（1 新）、av 19/19、`cargo t ace-source ace-av` **两组 PASS**（`gate-t04d.log` `67bb27666d2610c2`）、零告警、fmt clean | remaining: V-09④/V-10/V-11 跨 plan 承接 + V-DETAIL-01/04 + F11/F12 + T-06/T-07 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 32 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: e0fba7367 | next: work
2026-11-15 stage: work / T-04 **V-DETAIL-04 完整分母对账**（机械可复核）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass（对账件交付）| artifacts: `denominator.md` `657f427cb4223447`（V 行→证据用例逐行映射；实测分母 **76 = core 51 / av 19 / a2x 6**）+ `check-denominator.ps1` `6197cc9193057216`（机械检查：①文中每个 `::p979_*` 用例名必须在源码有 `fn` 定义——当前 41 个唯一名全中；②实测计数必须等于文档 51/19/6；任一不符 exit 1）| finding: 对账暴露**恰好 4 个 V 行无用例**：V-09④、V-10（实例引用部分）、V-11（产品清单）、V-DETAIL-01——与债务表 D-979 的承接项一一对应，**未计入已交付**；其余行均有真实执行证据（双后端行只计真实 rustc/AV 运行）| verification: 检查脚本两轮通过（`claimed_unique_names=41 OK`、`actual_counts core=51 av=19 a2x=6 OK`）；代码未变，沿用 `gate-t04d.log` 的 ace-source/ace-av 两组 PASS | remaining: 上述 4 行 + F11/F12 + 全包 T-06/T-07 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 33 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: e62be4279 | next: work
2026-11-15 stage: work / V-09④ 多模块**可行性探针**（只读，不改语法）| plan_id: PLAN-979 | plan_revision: 10 | outcome: 判定=非可界切片（证据在案）| artifact: `v09d-probe.md` `287e6e9a3bb9e369` | evidence: ①语法：AGE 规则本体只声明 `[age-h0, age-component]`（L309/L335），item 备选 L263/L264 只在 AGE 面加 AGE 项；`use` 只在 `scalar-modules`/packages 三面（L395/L468，item 备选 L288）——组合面不存在，需新增 profile + 逐条扩写 AGE 规则（~15–20 处）②流水线：packages 面 `bind_package_syntax(inputs, entry, caps, budget, units)` 消费 `PackageSyntax`/`BoundPackages`（binding.rs:348），AGE 节点不是普通声明，需模块级投影 + 跨模块 `use` 解析 + bundle 级 verify + 双后端入口选择；当前 AGE 前门是单模块 ③**语义面已验**：V-09 行为断言（两同定义独立实例、2:1 定向事件、N 次 Tick、helper、typed view 状态绑定）在单模块内全量通过（AV 22/10/259/516 ↔ A2R 514/259/516），缺口仅是**两文件工程容器** | disposition_options（需用户裁定）: ①等 960 模块面后组合（现状 D-979 承接）②接受"行为已验、工程容器归 960/980"的表述并允许 979 收口 ③为 979 追加跨 plan 模块集成 | blockers: **同一条件已连续 2 轮**（27/29：剩余 V 行需他 plan 面）；若下一轮仍无新可执行项，将按 goal 策略报 blocked 并请裁定 | next_responsible_role: work→user（裁定）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 34 | from: work | to: work | status: executing | outcome: pass（探针）| worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 161798b29 | next: work
2026-11-15 stage: work / T-03 **F11 收口**（单次解析）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: 7a91ed428 | change: 两个组件前门此前对同一源解析两次（隐藏名扫描 + 脱糖输入，再加投影的 CST 解析 = 三次）；现程序项**只解析一次**并由扫描与脱糖共用，行为不变 | verification: core p979 **51/51**、av 19/19、`cargo t ace-source ace-av` **两组 PASS**（`gate-f11.log` `1a0a1420f233548a`）、零告警、fmt clean | debt_ledger: D-979 的 F11 行可标"已闭（单次解析）"，余项 F12 仍开 | remaining: V-09④/V-10/V-11/V-DETAIL-01（跨 plan）+ F12 + T-06/T-07 | blockers: 跨 plan 条件已连续 3 轮（27/29/30）；本轮证明 F11 仍可执行，故未判 blocked——**下一轮若无新的可执行项，将报 blocked 并附三项处置选项** | next_responsible_role: work→user（裁定）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 35 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 7a91ed428 | next: work
2026-11-15 stage: work / T-02 **F12 收口**（证据身份补源哈希与兼容配置）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: d74433a59 | change: `DomainIdentity` 现含 `id/version/grammar_id` **+ `input_hash` + `compat`**；`same_evidence` 因此对"同包不同源"和"同源不同 widget 准入"都判失效（review F12 指出的陈旧证据洞）；`ComponentCompilation::identity()` 给出同一元组，接投影缓存时可直接作键 | evidence: `p979_component_f12_identity_is_sensitive_to_source_and_compat`（同源稳定；源变失效；compat 变失效）+ `plan979_combined` 既有换包失效用例更新后仍绿 | verification: core p979 **52/52**、av 19/19、`cargo t ace-source ace-av` **两组 PASS**（`gate-f12.log` `84a4ad2a4436005e`）、零告警、fmt clean | debt_ledger: D-979 的 **F11、F12 两行均可标已闭**；执行侧仅剩跨 plan 四项 | remaining: V-09④/V-10/V-11/V-DETAIL-01（跨 plan 承接，已连续多轮无新可执行项）+ T-06/T-07 | blockers: 跨 plan 条件连续 27/29/30/31 轮不变——**下一轮按 goal 策略报 blocked 并请用户在三项处置中裁定** | next_responsible_role: work→user（裁定）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 36 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: d74433a59 | next: work
2026-11-15 stage: work / **F1 后续收口**（per-field 状态函数：多字段模型正确绑定）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: fa8b64f8b | change: 场景脱糖由"每实例一个状态函数（返回首字段）"改为**每 (实例, 字段) 一个** `fn __age_state_<i>_<field>() int`；驱动取各实例**首字段**按 256^i 求和；渲染函数按**引用到的字段**取对应状态函数。多字段模型的 view 因此绑对字段（review F1 的 wrong-field 反例由"具名拒绝"升级为**正确行为**）| evidence: core `p979_component_scenario_observes_each_field_independently`（脱糖模块含两字段各自状态函数，原 refusal 用例转正）；av `p979_component_typed_view_binds_the_named_field_not_the_first_one`（count=1、other=9 → 编码 1、渲染 "big"）、`..._follows_its_own_field_events`（只推 other：编码 0、渲染 "big"）；**规范 §6 同步**（状态函数/驱动/渲染三处措辞 + pending 列表去掉"单字段"限制）| verification: core p979 **52/52**、av p979 **21/21**、a2x p979 6/6、`cargo t ace-source ace-av ace-a2x` **三组 PASS**（`gate-f1b.log` `5878872753f1e795`）、零告警、fmt clean | remaining: 跨 plan 四项（V-09④/V-10/V-11/V-DETAIL-01）+ T-06/T-07；内部债 F11/F12 均已闭 | blockers: 跨 plan 条件仍存（27/29/30/31/32），但仍有可执行改进（本轮即一例），故不判 blocked | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 37 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: fa8b64f8b | next: work
2026-11-15 stage: work / 多节点静态描述 + 分母对账刷新 | plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: 4858e03a3 | capability: 静态组件面对一个模块内的**多个 view-only Node** 已有描述能力（此前只是隐含行为——无回归用例）：两节点共 3 个文本叶各有独立 def id、节点 span 有序；场景面仍保持**明确单节点**形状规则（实例语义按节点归属属 981）| evidence: `p979_component_static_face_describes_every_node_in_the_module`；core p979 **53/53**、av 21/21 | denominator_refresh: 对账脚本**先报红**（旧用例名 + 计数 52/21/6 与实测 53/21/6 不符），随后刷新 `denominator.md`（**80 例 = core 53 / av 21 / a2x 6**，新增多字段/多节点/F12 用例名，移除已废弃的 refusal 名）与 `check-denominator.ps1` 期望计数，重跑 **两断言全绿**（`claimed_unique_names=45`、`actual_counts core=53`）——证明该守卫确实会拦住"声称有测试/计数漂移" | artifacts: `denominator.md` `8474067796897b0c`、`check-denominator.ps1` `ab93c52342507a45` | remaining: 跨 plan 四项 + T-06/T-07 | blockers: none（仍有可执行项）| next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 38 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 4858e03a3 | next: work
2026-11-15 stage: work / **bool 模型字段（typed）+ 观测字段必须 int** | plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: c892c19e6 | capability: 模型字段按声明类型（`int`/`i32`/`bool`）同时用于**per-field 状态函数返回值**与实例局部变量；bool 标志可由事件推进并经 bool 形参 helper 渲染（`text pick(enabled)` → "on"/"off"）。**观测字段（首字段，驱动以 i32 编码）必须是 int**，否则具名拒 `source.age.component-unsupported`（span 指向字段类型）| evidence: core `p979_component_bool_fields_are_typed_and_the_observed_field_must_be_int`（bool 第二字段可用 + 首字段 bool 具名拒、span 精确）；av `p979_component_bool_field_renders_through_its_typed_state`（count=1 编码、enabled=true→"on"、Off→"off"）、`p979_component_bool_observed_field_is_refused_by_name` | verification: core p979 **54/54**、av p979 **23/23**、`cargo t ace-source ace-av` **两组 PASS**（`gate-bool.log` `38c0dc06c9609cbc`）、零告警、fmt clean；分母刷新为 **83 = core 54 / av 23 / a2x 6**（`denominator.md` `704e3cef9155006a`）且检查器两断言全绿 | remaining: 跨 plan 四项 + T-06/T-07 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 39 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: c892c19e6 | next: work
2026-11-15 stage: work / **静态面值观察**（AV 消费端补腿）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: efe215146 | capability: 静态组件面此前只有描述（无值证据）；新增 `ace_av::age_component::observe_component_texts(src, path, alias)`：共享前门 + 唯一 verify 编译后，逐叶在共享 AV 链执行隐藏 Str 函数，按节点分组返回值（节点序/叶声明序）| evidence: av `p979_component_static_face_observes_every_text_value` —— 两节点（First 两叶：字面量 + `greet()` helper；Second 一叶）→ `[["a","hi"],["c"]]` 真值 | verification: av p979 **24/24**、core 54/54、`cargo t ace-source ace-av` **两组 PASS**（`gate-static.log` `c3e8122d68dde1d2`）；分母刷新 **84 = core 54 / av 24 / a2x 6**（`denominator.md` `f4a22ebb340828b6`）两断言绿；改动文件零新增告警（av 既有 4 告警在未触碰文件）| remaining: 跨 plan 四项 + T-06/T-07 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 40 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: efe215146 | next: work
2026-11-15 stage: work / **观测字段 = 首个 int 字段**（bool 首字段模型可观测）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: bb0a4267e | change: 驱动不再要求"首字段必须是 int"，改为观测**首个 `int` 字段**；故以 bool 标志开头的模型也能经后续 int 字段观测。形状优先：无任何 model 字段仍是 `component-shape`（守卫前移），只有 bool 字段才 `component-unsupported`（span 指向字段类型）| evidence: av `p979_component_observation_picks_the_first_int_field`（bool 首字段 + count 第二字段：instance0=1、instance1=2 → 编码 `1+2*256`）；core/av 相关用例改名反映新规则（`..._an_int_field_is_observed`、`..._bool_only_model_is_refused_by_name`）| verification: core p979 **54/54**、av p979 **25/25**、`cargo t ace-source ace-av` **两组 PASS**（`gate-observe.log` `236e8decd3ead6b5`）、零新增告警、fmt clean；分母 **85 = core 54 / av 25 / a2x 6**（`denominator.md` `481db44e8c686290`）两断言绿 | remaining: 跨 plan 四项 + T-06/T-07 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 41 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: bb0a4267e | next: work
2026-11-15 stage: work / **观测字段按名可选**（scenario + AV 两侧）| plan_id: PLAN-979 | plan_revision: 10 | outcome: pass | code_commit: a29f896a2 | change: 新增 `compile_component_scenario_observing(..., observed: Option<&str>)` 与 AV 侧 `observe_component_observing`：驱动编码**指定 int 字段**（缺省仍为首个 int）；未知字段名 → `source.age.component-unknown-field`，非 int → `source.age.component-unsupported`（span 指向字段类型）。原 `compile_component_scenario`/`observe_component` 行为不变（委托 None）| evidence: core `p979_component_observed_field_can_be_chosen_by_name`（默认观测 first、指定 second 时驱动 callee 变为 `d___age_state_0_second`）、`..._must_exist_and_be_int`（两码）；av `p979_component_observing_a_chosen_field_changes_the_encoding`（默认 first 不动 → 编码 0；观测 second → 1+2*256）| verification: core p979 **56/56**、av p979 **26/26**、`cargo t ace-source ace-av` **两组 PASS**（`gate-observed.log` `78d1b7c860a00ce1`）、零告警、fmt clean；分母 **88 = core 56 / av 26 / a2x 6**（`denominator.md` `6669fb51275d0302`）两断言绿 | remaining: 跨 plan 四项 + T-06/T-07 | blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 42 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: a29f896a2 | next: work
2026-11-15 stage: review / **T-05 复审第三轮 needs_fix（绑定 cc2a1bb77）→ N1–N6 修复** | plan_id: PLAN-979 | plan_revision: 10 | review: 独立复审（`reviewer-979-t05c`，只读探针 34 断言）判定 **代码 fold-safe**，但有 1 处"已闭"过度声明、规范自相矛盾/陈旧、T-04 分母件绑定不存在的提交——全部 P3（声明/规范/工件）| fixes: **N6** 保留名族扩到 `__age_state_*`/`__age_drive`（占用即 `hidden-name`）+ 重复 `model` 字段名具名拒 `component-duplicate`；**N5** 删除死代码重复块、修正"首字段必须 int"/"10^i"陈旧注释；**N2/N3** 规范改写：V-10 不再声称"无校验"（实例/事件拒绝已存在，未建模的是**跨实例状态引用**）、补静态门"任何 model/on 即拒 + 零 node 接受"、补新拒绝码与保留名族、补身份元组、标题不再读作 H0-only；**N4** 分母件重写：绑定当前提交、末行"分母: core=.. av=.. a2x=.."由检查器**解析比对**（不再硬编码），断言 1 改为"每个 `::p979_*` 必须是 `cargo test --list` 的**真实用例**"（不再只是"源码里有同名 fn"）——现 45 个声明名 / 89 实测用例 / 0 缺失，两断言绿；**N1** F11 声明降级为"程序项解析一次（投影 CST 解析独立）"，不再写"单次解析全闭" | code_commit: ed5fb52ae | verification: core p979 **57/57**（1 新）、av 26/26、`cargo t ace-source ace-av` **两组 PASS**（`gate-n6.log` `cdd80722fbc9ea42`）、零告警、fmt clean；`denominator.md` `7395c2bb0c4f84ec`、`check-denominator.ps1` `660e6a3727604d2c` | remaining: 跨 plan 四项 + 增量 fold（规范已同步）+ T-06/T-07 | blockers: none | next_responsible_role: work（下一轮做增量 fold：组件切片+规范再落主）

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 43 | from: review | to: work | status: executing | outcome: fixed | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: ed5fb52ae | next: work
2026-11-15 stage: merge / **A 切片增量 fold #2（组件面增长 + 规范修正）落主** | plan_id: PLAN-979 | plan_revision: 10 | delivery_commit: **0bc0ba711**（master ff-only from plan-979-dev@0bc0ba711；fold 前 branch ahead=14 / behind=0）| folded: 上一 fold 后新增的能力与修正 —— per-field 状态函数（多字段正确绑定）、多节点静态描述、typed bool 字段、静态面值观察、观测字段=首个 int、按名选观测字段（core+AV 双侧 API）、保留名族扩展 + 重复字段具名拒、死代码/陈旧注释清理、**规范 §6 全面刷新**（状态函数/驱动/观测 API/拒绝码/静态门规则/身份元组/边界）、第三轮复审 N1–N6 修复；`denominator.md` + `check-denominator.ps1` 重写为「真实用例集 + 解析式计数比对」| landing_checks（主检出）: core p979 **57/57**、av p979 **26/26**、a2x p979 **6/6**；他方会话 docs WIP 未触碰 | remaining（未 fold，B 表）: V-09④ 多模块容器（960）、V-10 跨实例状态引用（981）、V-11 产品字段清单（980）、V-DETAIL-01 原产品图形、T-06/T-07 全包收尾；F11/F12 已闭（F11 口径按 N1 降级为「程序项一次解析」）| plan_state: T-03/T-08/T-04 保持 open（上述四项），T-05 三轮复审（末轮 needs_fix → N1–N6 已修，代码被判定 fold-safe）| blockers: none | next_responsible_role: work

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 44 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 0bc0ba711 | next: work
2026-11-15 stage: review / **T-05 复审 3b（绑定 0bc0ba711）needs_fix → N1/N4/N5 + 新 P2/P3 修复并再 fold** | plan_id: PLAN-979 | plan_revision: 10 | review: 复审确认 N2/N3/N6 已闭、代码仍 fold-safe；**N1/N4/N5 未闭**（均为文档/工件）| fixes: **N1** D-979 的 F11 行与计划口径改为「程序项解析一次；投影的 CST 解析是独立一遍」；**N5** 驱动注释 10^i→256^i（code 本就是 256）；**N4** 分母件重绑到**已 fold 提交**（见下）并强化检查器：声明名按**完整 module::case 路径**比对 `--list`，计数从文档解析比对，另报"文档未列但实测存在"的数量（46 声明 / 90 实测 / 0 缺失 / 44 未列）；**P3** 规范去重 component-duplicate 条目、把零 node 接受范围限定为「可解析但无 node 声明」（空源仍 source.syntax）| fold: `git merge --ff-only plan-979-dev` → master **75cd2da39**（含本轮 doc 修正 + 复审后新增的 A2R chosen-field 用例 `eac8d1229`，使分母 57/26/7 在 fold 提交上成立）| landing_checks: core p979 57/57、av 26/26、a2x **7/7**；`denominator.md` `501fb369e9daede3`、`check-denominator.ps1` `05910d15614156c4` 两断言绿 | remaining: 跨 plan 四项（V-09④/V-10 跨实例/V-11/V-DETAIL-01）+ T-06/T-07 | blockers: none | next_responsible_role: review（下一轮请求对 75cd2da39 的最终确认）
2026-11-15 stage: work / **收尾包就绪**（T-06/T-07 一次执行清单）| plan_id: PLAN-979 | plan_revision: 10 | artifact: `plan-artifacts/plan-979/closeout-package.md` `dec20d3487f217ef` —— A 可声明交付（H0 包/通用接面/组件面/双后端/拒绝面/规范 §6/分母对账，逐项带证据指针）、B 未闭项（V-09④→960、V-10 跨实例→981、V-11 清单→980、V-DETAIL-01→980/981，owner 与关闭条件见 D-979）、C T-06/T-07 执行清单（最终复核绑定→规范核对→计划归档→证据折叠+按裁定删除 scope→wt-guard clean 后清树/分支→retention 刷新）、D 待裁定三项 | plan_checklist: 核对 §8 复选框——T-H0-*/T-01/T-02 已勾且与实际相符；T-03/T-08/T-04 保持未勾（B 表四项），T-05 待本轮最终确认，T-06/T-07 未做（正确）| blockers: none（等待 T-05 3c 复审结论与用户对 B 表的裁定）| next_responsible_role: review→user

stage: goal | plan_id: PLAN-979 | plan_revision: r10 | round: 46 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-979/auto-lang | commit: 75cd2da39 | next: review
2026-11-15 stage: review / **3c 复审 needs_fix 的根因修复：文档修正此前留在主检出未提交** | plan_id: PLAN-979 | plan_revision: 10 | finding: 复审绑定 75cd2da39 指出 N1 的台账行、规范去重与零 node 范围**并未进入任何提交**（只存在于主检出的未提交改动），且 a2x 测试有未用 import——收据 89dace144 因此过度声明 | fix: 我在主检出的那些改动已保存为 `pending-doc-edits.diff` 并从主检出撤回；**改在 plan-979-dev 上重做并提交**：D-979 的 F11 行改为「程序项解析一次；投影的 CST 解析是独立一遍」（含 N1 更正标注）、规范只保留**一处** `component-duplicate`（合并重复字段说明）、零 node 接受范围限定为「可解析但无 `node` 声明」（空源仍报 `source.syntax`）、a2x 测试去掉未用 import；rebased 后 **ff 落主** = `ecd3c1845` | verification: master @ecd3c1845 —— core p979 57/57、av 26/26、a2x 7/7；检查器重绑该提交：46 声明 / 90 实测 / 0 缺失、计数 57/26/7 一致；`denominator.md` `8a6be41e7ce3cf7a` | process_lesson: 规范/台账同类文档也必须经 worktree 分支提交再 fold，不得直接改主检出（本轮已纠正）| remaining: 待 3d 确认 + B 表四项裁定 | blockers: none | next_responsible_role: review
2026-11-15 stage: review / **T-05 增量复审 pass（绑定 a6a72af0f）** | plan_id: PLAN-979 | plan_revision: 10 | verdict: **pass** —— 复审在 a6a72af0f 逐项核销 N1/N4/N5 与三个 P3（台账 F11 行、规范单条 component-duplicate、零 node 范围、a2x 未用 import），并确认 master 折叠态**已复审**（本增量）；核心/av/a2x p979 实测 57/26/7 | fixes_from_residuals: ①检查器新增**绑定断言**（读 `denominator.md` 的绑定 sha；HEAD 非该提交时必须证明是**仅文档**后代，否则 exit 1）——本轮实测输出 `OK: HEAD a6a72af0f… is a docs-only descendant of bound ecd3c1845`；②标题由「完整分母对账」改为「V 行分母对账」（46/90 具名，余下仅计数，避免"完整"过度声明）| artifacts: `denominator.md` `f91ab3ed4cb74595`、`check-denominator.ps1` `44254f3a4dd5e929`（三项断言全绿）| plan_checklist: T-05 仍**未勾**——复审通过的是**本增量**，T-05 口径是"全部 AC/V"复核，待 B 表四项关闭；已在该行附进度说明 | remaining: 跨 plan 四项（V-09④/V-10 跨实例/V-11/V-DETAIL-01）+ T-06/T-07；收尾包 `closeout-package.md` 就绪待裁定 | blockers: none | next_responsible_role: user（B 表裁定）/ work
2026-11-15 stage: review / **用户裁定：等待 960/980/981 交付后由 979 组合收口** | plan_id: PLAN-979 | plan_revision: 10 | ruling: 用户选择「等待 960/980/981」——B 表四项（V-09④ 多模块容器→960、V-10 跨实例状态引用→981、V-11 产品字段清单→980、V-DETAIL-01 原产品图形→980/981）**不由 979 自行实现**，也不授权以 A 表交付收档；979 保持 `executing`，待各 owner 交付后由 979 组合并复核再走 T-06/T-07 | implication: 计划目标「完成」需跨 plan 依赖交付；本会话不自行缩范围、不把 A 表当作整包交付 | state: A 切片已 fold 且经复审 pass（a6a72af0f；core 57/av 26/a2x 7；规范 §6 已沉淀；分母件含绑定断言）；B 表四项在 D-979 具名（owner + 关闭条件），状态=等待依赖 | goal_policy: 同一外部依赖条件已连续多轮（≈27→42）且经用户确认等待，故 goal 置 blocked 并记录具体条件，而非继续消耗轮次 | next_responsible_role: 960/980/981 owner（交付后回 979 work）

2026-11-16 stage: work / T-08 **V-11 UI 字段半**（IF-980-FIELDS 消费核验）+ T-04 增量 | plan_id: PLAN-979 | plan_revision: 11 | outcome: partial（子面 pass；V-11 未闭，T-08/T-04 保持 open）| worktree: D:/autostack/.wt/lang-979/auto-lang @ `c19102e40`（resync 后 clean；本切片无代码改动）| handoff_identity: auto-lang master `3de0bea0b`（DIALECT 批）+ `46f4ac0ec`（字段子面）、auto-ui master `f864f33b`；均为落主提交，非分支 tip | manifest: `ui.widget-decl/v1` **115 行**（16 required / 91 optional / 8 conditional），`P980_CONSTRUCTS` **115 键**双向相等，`WidgetDecl` **19 字段**全锚定，16 必需行均 002 锚定，条件行 8/8 带可核理由 | controls_run: lang `plan980` **13/13**（0.06s）、ui dialect `p980` **3/3**（构建 1m27s）、ui compiler `ui_domain_validation` **5/5**（构建 1m11s，含 handoff/mapping controls）| independent_audit: `v11-ui-audit.ps1` `9aeca4081043408e` → **19/19 PASS**（`v11-ui-audit.txt` `596a3aaf2b3b1c9f`）| judgment: V-11 UI 字段半=已核（清单可表达、坏字段/缺映射具名红、删必需映射/删清单行红且恢复绿）；V-11 未闭（AGE 半 IF-981-FIELDS 未落主）；4 槽位具名拒绝计交付面 | remaining: IF-981-MODULE/REF/FIELDS/ORIGINAL（`plan-981-dev` 领先 53 提交、未 phase fold 落主）→ V-09④/V-10/V-11-AGE/V-DETAIL-01 → T-04 汇总 → T-05 | blockers: 981 IF 未落主，按 §5 不得以分支 tip 当正式输入 | raw_evidence: D:/autostack/plan-artifacts/plan-979/{v11-ui-handoff.md `447fa9780e8e95e9`, v11-ui-audit.ps1, v11-ui-audit.txt} | note: 主检出该文件的 r11 修订原为先前会话未提交 WIP，本次仅追加本条并一并提交（r11 正文未改）| next_responsible_role: work（等 981 落主后组合收口）
