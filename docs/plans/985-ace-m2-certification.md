---
plan_id: PLAN-985
status: executing
feature_name: M2两阶段验收与完整义务联合认证
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 16
current_step: 4
total_steps: 14
depends_on: [PLAN-971, PLAN-972, PLAN-973, PLAN-974, PLAN-975, PLAN-976, PLAN-977, PLAN-978, PLAN-984, PLAN-941]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/auto-bindgen/project.md, docs/specs/stdlib/project.md]
new_spec_components: [docs/specs/ace-migration/m2-certification.md, docs/specs/auto-test-case/ace-m2-stage-verification.md]
touched_goals: [GOAL-002]
---

# [PLAN-985] M2两阶段验收与完整义务联合认证

## 0. 变更摘要

2026-11-15 r15当前：执行用户Linux先代码后测试及普通math/io/net自有实现裁定，§5/6/7新增当前验收与1063移交规则，优先于旧Linux当期实跑表述；原任务ID/勾选/进度保留，Windows必需验证和全部Linux原分母保留。此次仅修订合同，不实施或运行测试。

2026-11-13 r14当前：纳入新1043来源后缀装配及最小#in平台静态选择，见§5/AC-LOAD/V-LOAD/SD-LOAD；原任务/状态/完成数与全覆盖不减，仅规划。

2026-11-13 r13当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-13 r12：用户纠正原生标准库基础依赖；新增AC-FOUNDATION/V-FOUNDATION/SD-FOUNDATION，将算法准备与默认自有库原生实施/验收分开，修正整包前置。原任务ID/数量/状态/进度/旧AC与V不减，本轮仅规划。

2026-11-13 r11：参考Zig/Go的原生平台I/O设计承接；新增AC-PLATFORM-IO/V-PLATFORM-IO/SD-PLATFORM-IO，由原任务负责。保全部原任务/数量/进度/既有AC/V及其他会话修订，本轮仅规划。

2026-11-13 r10（本轮当前合同）：认证接收Node首Hello、同一Schema的widget兼容与位置/AGE上下文反控；保RG-A03/RG-A04完整义务，不用Hello或源码重写冒原源兼容。 用户仅授权规划修订；保原任务ID/数量、既有AC/V与进度、完整分母及阶段门，不建树、不写实现/fixture、不构建/运行/合入。此前§0修订描述为历史，冲突以本次§5–8为准。

2026-11-12 r8：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-12 r7当前：纳入默认自有Auto基础标准库/非libc默认provider及长度字符串ABI边界，见§5新增合同。保原任务/阶段/范围，新增AC-NATIVE/V-NATIVE/SD-NATIVE，仍只规划。

历史修订：2026-11-11 r6当前：接收967/F0、979/H0、969/E0、978/B0、980/HELLO早期已审落主切片作为既有A义务证据，不把AV Hello当完整FFI/UI/RG-A03或M2A-08。五包新增20任务，本包原14任务/AC/V保留、0完成；无新认证状态/runner，仅规划。

历史修订：2026-11-10 r5当前执行版：跨M2-A/B。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**跨M2-A/B**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r3：按用户要求纳入[Design56：完整FFI与新版标准库](../design/56-ace-m2-ffi-and-native-stdlib.md)的本包义务；当前仍为drafting、0/9，只规划不施工。原任务/AC/历史记录保留，新增任务均在最终work验证与独立review之前。

承941正式M1结论，建立旧义务→新source/profile/producer/oracle后继账；四端完整语言和真实产品矩阵分别计量，保M1旧required/38格问题的实际关闭证据，不能新证书抵旧缺口。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11用户授权首ACE Hello路线规划与既有M2冲突复核，仍不实施/运行；本文只同步覆盖和阶段接收规则。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `crates/auto-test-case`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/test_cases`（本轮确认存在；开工重核符号/版本）。
- `scripts/test-groups.toml`（本轮确认存在；开工重核符号/版本）。
- `docs/specs/ace-migration/replacement-scope-and-evidence.md`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-09 增补调研锚点

依据用户本轮M2范围追加与已读取的标准库引用讨论，读取基线HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；本轮只读核源码，不宣称现有实现已满足新义务。Design56区分现状与目标；原§4历史快照不覆盖本次范围。

- `docs/specs/auto-bindgen/project.md`（现存入口/规范；开工核真实符号与SHA）。
- `docs/specs/stdlib/design/semantic-operation-contract.md`（现存入口/规范；开工核真实符号与SHA）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：M2-A/B完整覆盖/守恒/联合认证。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/auto-test-case/src/contract.rs](../../crates/auto-test-case/src/contract.rs)：现case/check合同复用；读取SHA256前16 `e68742e04646c79b`

- [crates/auto-test-case/src/ir.rs](../../crates/auto-test-case/src/ir.rs)：resolved源/产物/oracle/步骤；读取SHA256前16 `1e256f0cf852be7a`

- [crates/auto-lang-core/src/test_cases](../../crates/auto-lang-core/src/test_cases)：父执行/原始终态与诊断比较，不建新裁判；目录职责已核，确切符号/闭包开工再核

- [scripts/test-groups.toml](../../scripts/test-groups.toml)：现固定组导航/真实执行选择；读取SHA256前16 `9169834afc9a7a02`

2026-11-13本轮授权/取材：用户明确要求修订Node首Hello、Schema解析校验与位置/AGE上下文名称、widget兼容和963并行顺序，仅规划。读取基面`dcf0535d4d15c00628a60f1940c6c55a01a101ab`；当前Spec仍描述有限统一CST与旧UI parser/有限Node pilot，并未宣称本次机制已交；差额是本Plan目标，未来按SD沉淀，不改canonical Spec伪造现状。来源：`docs/specs/auto-syntax/{source-frontdoor,grammar-contract}.md`；`docs/specs/auto-ui-dialect/design/full-grammar.md`；`experimental/ace/crates/syntax-runtime/src/{program.rs,parse/mod.rs}`；`experimental/ace/crates/core/src/source/cst/frontdoor.rs`；`auto-ui/docs/specs/auto-graphics/{node-basic-ui,ace-migration-feasibility}.md`。963执行进度与其他会话WIP只读保留。

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=跨M2-A/B。A由T-01、T-A01..A05、T-09以及T-08的能力/RG腿组成；A不等待984/CLOSURE、OS064/ASSEMBLY或本包B全产品任务。B原T-02/03/04/05/06/07及T-08全产品并集在M2A-08后完成，仍接986/987原安装退役门。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A准备 | T-01 |
| M2-A | T-A01, T-A02, T-A03, T-A04, T-A05, T-09 |
| M2-B | T-02, T-03, T-04, T-05, T-06, T-07 |
| M2-A/B | T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


承941正式M1结论，建立旧义务→新source/profile/producer/oracle后继账；四端完整语言和真实产品矩阵分别计量，保M1旧required/38格问题的实际关闭证据，不能新证书抵旧缺口。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：组合认证与安装阶段分账（r2）

将Design55 MIX-01..06绑定各owner永久fixture及本包父门，测试能力存在/目标执行/产品接线三账。消费978候选包做仓外预认证；REP-08/10/11最终安装/退役项明确pending交986/987，不以本包完成签M2。

本节由同一T-08分阶段实施：T-08[A]只消费T-A02、能力owner和RG-A正式输入，其A组合结果进入T-A03，不等待B期T-02/03；T-08[B]在M2A-08后消费全产品T-02/03等结果进入T-04。AC-06/V-08整项仍须A/B并集齐才关闭；A片段完成不得提前勾T-08，B-pending须保留。T-08[A/B]是同一任务的证据切片，不新增任务或Plan状态。

### 2026-11-09 正式M2范围增补（r3，当前合同）

新增FFI/LIB全义务候选预认证：并入948的用户新增子表与959原分母，逐行核967自动导入、966平台条件、972默认Auto/core/alloc、973系统provider及978候选closure的正式身份/真实结果。binding proof、跨端相等、手工manifest或旧桥均不代独立行为。删除任何required父/子行或只标总计必须阻签。

仓外候选测试分别执行C标准头/SQLite、Rust serde_json及owned/callback/future fixture、四端统一stdlib和真实File。Windows/Linux按正式平台适用表核执行证据；缺环境pending。候选证据不冒最终安装：986/987的升级/退役安装腿明确pending交后继，985不依赖它们。

本包新增T-09/AC-07/V-09同时核Design55 MIX-07：同一公共Auto工程经目标选择、File、Rust解析与C数据库闭合；985交候选组合证据，986/987按各自安装/退役阶段重核，不能用分开的演示程序抵掉组合义务。

本次新增验收/任务如下表映射，详见§6–8；原任务不删除、原非目标中的“本包之外”按本节明确新增范围解释。所有V均为待执行设计，当前不写canonical Specs。

| 任务 | 验收 | 用例 | 本包交付/义务 |
|---|---|---|---|
| T-09 | AC-07 | V-09 | 冻结并审计FFI-M2-01..08/LIB-M2-01..06及全部子行，运行四端真实外库/新库组合与缺行/手写binding/旧桥/缺工具对照，给986/987具名pending安装项。 |

### 985任务级依赖（当前）

| 执行节点 | 最小前置 | 产出与接收 |
|---|---|---|
| T-01 / T-A01 | 已交959及真实Spec/当前测试来源；只做已授权的分母核对 | A/B旧义务与新check/owner，交T-A02 |
| T-A02 | T-A01及所用能力接口 | M2测试描述/必要adapter/父侧控制与真实命令，交T-09、T-08[A]和T-A03 |
| T-09 / T-08[A] | T-A02及对应A能力/RG正式来源 | 新库/FFI/MIX能力与候选证据，交T-A03；不等待T-02/03(B) |
| T-A03 | T-A02、T-09、T-08[A]、所有M2A-01..07能力/RG必需结果 | A实际覆盖并集与逐项结果，交T-A04 |
| T-A04 → T-A05 | A全部work结果→独立A review通过 | 核阶段fold已合入SHA、落A收据；M2A-08成立后开B |
| T-02 / T-03 / T-08[B] | M2A-08及全产品原最小前置/正式输入 | B全消费者与组合并集，交T-04；不得靠A/RG收据冒全产品 |
| T-04 → T-05 → T-06 → T-07 | 全部B work与A证据适用性→全包独立review→正式交付→收尾 | 整包合入/归档/清理；986/987仍补最终安装与退役签章 |

### 985/A：语言、新标准库与RG-A独立出口

只消费[Design55 §16](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)的A-required，不等产品整包D984；A能力来源按正式交付/阶段fold已落主身份核定。T-A01可按959/当前测试规范先整理；T-A02按各能力稳定接口补测试描述/薄adapter，完整能力case仍由能力owner负责；T-A03必须等所有A能力与RG-A收据，不用调研或自报PASS补缺。T-A04独立复核后，T-A05按现有阶段fold规约核来源/沉淀本阶段规范及收据，只有已落主A基线可开B。

验证资产优先扩`crates/auto-test-case`、`crates/auto-lang-core/src/test_cases`、`experimental/ace/tests/conformance`、`experimental/ace/tests/stdlib`和`scripts/test-groups.toml`现有target。拟新增跟踪`crates/auto-lang/test/cases/ace-m2/`，含旧义务映射/版本化M2适用性与源/独立expected；新stage描述与父门控制须接现框架，不能另造tm/叶调度。旧M1绑定/适用性原件不改。A证书格式`ace.m2.stage/1`为待实现，当前没有新可运行命令；T-A02登记真实入口并接固定组，缺入口则T-A03不能开始。

A报告逐项列M2A-01..08、旧义务继承/具名替代、新增语义/组合/边界、平台条件、A/B分母及实际执行/NOT_RUN/有依据NA；B-pending明确保留，不能混成A缺口或删分母。M1 NA/旧bug/accepted-diff逐行归属，不自动继承豁免；旧VM/emitter内部测试按原不变量替代并独立审核。

T-A01先做缺行/重复/错source/旧binary/emit冒run/假worker PASS/timeout冒负例等实际控制，T-A03只有父侧真实逐check结果齐才通过；每端对独立expected，peer equality只作补充。原正式性能/压力/预算与RG真实外仓入口不变，不拿cargo t all代ACE或外仓。

A的独立review和阶段收据不等整个985 reviewed/archived；本包T-02/03/04/05/06/07及T-08全产品并集继续在B核984/CLOSURE正式来源，后继986/987承担最终安装/退役。原AC-01..07在其完整范围仍须全部关闭才能整包交付。

### T-01 A期冻结记录（2026-11-16，r16执行）

**授权**：PLAN-971 goal 会话续行指令（2026-11-16）——"推进 PLAN-985 的 M2-A 批次直至 T-A05/V-A-LAND 正式合入 M2A-08 收据，然后以同一目标恢复 971 M2-B"。本轮范围=本Plan §5阶段表的 A 批任务；B 批任务（T-02..07）只登记原输入/oracle与缺口，不实施。worktree：`D:/autostack/.wt/lang-985/auto-lang`（分支 `plan-985-dev`，基面 master@b8f80afcf）；材料 `plan-artifacts/plan-985/` 已登记。

**A最小前置核实（正式已落主来源）**：
- 语言族归档交付：959（分母冻结）/960（模块包）/961（数值）/962（聚合）/963（泛型）/964（动态）/965（控制效应）/966（A片，m2-metaprogramming 2026-11-14）/967（全work，scoped pass@517920443）/968（scheduler，reviewed aa630a2eb）/969（E0切片，phase fold@1ad2d2c60）/970（sessions，reviewed 295e93f7）/972（stdlib-core）。
- 早期切片已落主：967/F0、969/E0、978/B0、979/H0（终审 pass@71472d975）、980/HELLO；**971/A（B-PROOF，lang `2f1eed27f` + auto-ui `c1561d94`，2026-11-16，spec m2-tools sha16 `c69d0afd1aa1fdf3`）**。
- 基础设施归档：946/947/948（M1库族）、1039（平台I/O原语）/1040（Linux native ELF+syscalls）/1041（native stdio/console unicode）/1042（A2PY readiness）/1043（context assembly加载/#in）——FOUNDATION/LOAD 的 1039-1043 正式来源已齐。
- **硬前置未闭**（能力owner A批 pending，T-A03 阻塞项）：973（stdlib-io，executing 0/10，他在途——FOUNDATION 973真实Auto算法腿）；974/975（stdlib network/data，drafting 0/8）；976（typescript，drafting 0/8）；977（other-targets含PY语义，drafting 0/16——COVERAGE Python腿）；978（build，reviewed 10/14，复审修复落主中，余量收口后供 M2A-04/07）；979（schema，executing 6/12，H0已收余A腿）；980（autoui，drafting 0/35）；981（age，executing 0/34）；982（man-gen 0/40）/983（web 0/19）/984（os-apps 0/25）——RG-A01..06 全 pending；OS064（auto-os仓）pending。

**M2A门级现值**：M2A-01=骨架有（959冻结+早期切片接收），T-A01/A02义务矩阵未建；M2A-02=962-970/972已交，976/977/979/980/981 A批pending；M2A-03=947/948/972已交，973-975 pending；M2A-04=966/967/968已交，973/978/T-09 pending；M2A-05=RG-A01..06全部pending；M2A-06=T-A01..A03未执行；M2A-07=966/970已交，978 pending。**结论：M2A-08 在本轮不可闭合；985 A批当前可执行面=T-A01（义务矩阵+负控）、T-A02（已交能力的测试描述/adapter/真实命令）、T-09/T-08[A]的已交来源部分；T-A03..A05 待上述能力owner A批正式落主。**

**平台/资源/命令冻结**：A批验证入口沿现固定组与TestCase/ACE owner（T-A02逐能力登记真实命令）；平台=Windows x64（1040 Linux腿按r15裁定移交1063，Windows环境不跑Linux/WSL用例）；资源合同沿各owner现值，新增观测由T-A02版本化扩展，不由worker自报PASS。PIO/FOUNDATION/COVERAGE/LOAD 子合同来源状态如上（1039-1043已齐、973/977/978等消费腿pending，不冒原生/覆盖完成）。

### T-09 已交来源审计记录（2026-11-16，r16执行）

**实测**：`cargo nextest run -p ace-abi --test conformance --config-file ../../.config/nextest-full.toml -E "test(/f0_/)"`（985 worktree，experimental/ace）= **33/33 PASS，9.2s**（构建45.8s）——967 全部 FFI 组合 fixture 在当前树真实复验，含真外库端到端（serde_json `=1.0.151` 钉版 7.4s / SQLite 内容钉版 9.1s / Python ctypes 腿 5.9s）与负控矩阵（错版 `f0.sqlite-pin`、缺库 C1083 具名、坏SQL/双close/NUL路径/取消/卸载恰一次/dist隔离）。

**FFI/LIB 子行→已交具名证据绑定**（对照 m2-host.md@967 pass@517920443，全部本轮实测绿）：

| 行 | 已交具名证据（本轮实测） | pending 腿（owner） |
|---|---|---|
| FFI-M2-01 前门 | `f0_av_calls_real_rust_provider_end_to_end`＋`f0_project_import_runs_rejects_and_recovers`（手写manifest构造性不存在；provider缺/读拒+恢复） | — |
| FFI-M2-02 C importer | `av_runs_the_c_provider_through_a_real_dll`（真cl DLL）＋c_import 4案＋p967_sqlite fixtures | varargs/union/bitfield=已交具名拒面 |
| FFI-M2-03 Rust元数据/shim | `rich_mixed_scalar_frames_run_on_av_for_real`＋T-11形状审计（泛型/owned/async具名拒） | owned String/Vec/Result lane（D-967-T05） |
| FFI-M2-04 四端/六路径 | 六路径全实跑：rust_f0(AV/Rust)＋`f0_native_machine_code_consumes_the_same_auto_binding`(AV/native)＋av_c(AV/C)＋`axr0_runs_the_same_binding_through_generated_rust`(A2R)＋`a2py_runs_the_same_provider_through_ctypes`(A2PY)＋f0_int1026(AC帧) | — |
| FFI-M2-05 双向/重入/双代 | `v_abi_life_v1_v2_coexist_and_opaque_refs_never_cross`＋`f0_session_lifecycle_success_teardown_unauthorized_rebind`＋f0_t13 | owner/grant授权轴（D-967-T05） |
| FFI-M2-06 异步端口 | `f0_async_ports_publish_with_a_real_threaded_provider`＋cancel/teardown迟到丢（真OS线程/真C-ABI回调） | —（968 await已归档消费） |
| FFI-M2-07 分发/缓存 | `t14_precise_invalidation_and_reuse`＋`t14_two_rustc_toolchains_two_identities` | 运行依赖准入归978 B-06 |
| FFI-M2-08 真实库 | `t15_serde_json_real_library_end_to_end`＋`t15_sqlite_real_library_reads_and_writes`＋`t15_serde_json_through_python_ctypes`＋负控矩阵 | 986最终安装腿（见下） |
| LIB-M2-01/02/04 | 972/947已交＋`v_load_assembled_rs_provider_drives_real_foreign_calls`（1043装配Rs消费）＋`v_load_named_rejections_missing_duplicate_and_wrong_suffix` | 978装配收口（inflight） |
| LIB-M2-03 | 966 A片（#if/#is+sys）＋1043（#in/后缀）已交 | — |
| LIB-M2-05 | 1039/1040/1041原语已交 | 973系统域（inflight） |
| LIB-M2-06 | 948映射已交 | 978收口＋986/987腿 |
| LIB-M2-07 默认自有 | 1039-1043基础＋972算法已交 | 973 T-09/10（inflight） |
| LIB-M2-08 字符串/ABI | 967物理帧/UTF-8门＋t15 NUL路径边界 | 973/985 A/T-09腿 |

**对照族核对**：缺行=T-A01矩阵缺行红控；手写binding=f0_project构造性+具名拒（实测绿）；旧桥=967 T-02桥映射表（C桥/Rust桥/Python桥/数字ID→新身份，退役账归987）；缺工具=`f0.cargo-build`/C1083具名（t15负控内，实测绿）。

**986/987 具名pending安装项**：①986——FFI-M2-08 dist隔离消费已验（V-ABI-PACK）但最终安装/升级腿pending（948/978前腿）；K-01/K-04安装器须含新库闭包与manifest（LIB-M2-06腿）。②987——旧桥退役：`c_ffi.rs` libloading道、`trans/rust` wrapper manifest道、`py_known`、`host_bridge` 数字ID直通（967 T-02表=退役账）；K-02冻结SDK清理（981/SOURCE核身份后）。

### Hello早期切片的证据接收（当前）

T-A01把五包新增I/V/R/D与各新增AC/V登记到原语言/FFI/UI义务映射，不重复统计相同case；T-A02复用既有TestCase/worker和UI命名target补所需typed dispatch/byte/owner观测；T-A03只在原完整A范围与RG齐后关闭。AV Hello的正式来源仅关闭其行，同源A2R由原967/T-12、978/T-03/09、980/RUST另交；缺A2R或全UI行保持pending。早期切片不等985/A，985/A仍不等AGE新增图形/联合签章；完整FFI/LIB/四端与产品B义务不减。

### 默认自有标准库与FFI边界（2026-11-12当前）

依据[Design56 §0.2](../design/56-ace-m2-ffi-and-native-stdlib.md#02-默认auto系统库可选ffi与字符串边界2026-11-12当前)。在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。

本包验收：A默认库真实OS结果、四端长度/编码/owner边界和可选外库调用各有独立expected；隐式libc默认、NUL截断、错ABI/缺owner、emit冒run和缺provider控制不能过；B最终安装/退役腿保持pending。 原任务T-A01,T-A02,T-A03,T-09承接；不新增实现任务，不以新义务扩大M1/F0或解除945–948的M1-TAG。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r8当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/b-scalar.case.ad](../../crates/auto-lang/test/cases/ace-m1/b-scalar.case.ad)（下列节选L1–L47；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

````markdown
# PLAN-852 ace-m1 scalar value cases — the B01..B12 subset that is green
# on ac-native (mn exit), av (exit) and autovm (legacy-display result)
# with the SAME frozen oracle (T-02 observational evidence). B08/B11 are
# NOT here: the legacy engine computes 10/-7 against the frozen oracle 7
# (shadowing / named-arg binding differences) — difference rows in
# binding.tsv. a2r-snapshot checks freeze the reviewed T-03 goldens under oracle/.

```test-case
case {
  schema: "autolang.case/2"
  id: "ace.m1.b01"
  tags: ["ace-m1", "four-path", "scalar"]
  links: ["PLAN-852"]
  source: { file: "src/B01.at", syntax: "auto", path: "main.at" }
  entry: { mode: "program" }
  checks: [
    { id: "ac-native", adapter: "probe", action: "run",
      options: { recipe: "ac/mir-native-scalar-v1", mn: { entry: "d_main" }, steps: [{ op: "run" }] },
      expect: { completion: "success", exit: 14 } },
    { id: "av", adapter: "probe", action: "run",
      options: { recipe: "ac-av/core-run-v1", av: { entry: "d_main" }, steps: [{ op: "run" }] },
      expect: { completion: "success", exit: 14 } },
    { id: "autovm", adapter: "avm", action: "run",
      expect: { completion: "success", result: { format: "legacy-display", text: "14" } } },
    { id: "a2r-snapshot", adapter: "a2r", action: "emit", options: { module_name: "m1_b01" },
      expect: { completion: "success", artifacts: [
        { path: "m1_b01.rs", file: "oracle/m1_b01.rs", compare: "bytes" } ] } }
  ]
}
```

```test-case
case {
  schema: "autolang.case/2"
  id: "ace.m1.b02"
  tags: ["ace-m1", "four-path", "scalar"]
  links: ["PLAN-852"]
  source: { file: "src/B02.at", syntax: "auto", path: "main.at" }
  entry: { mode: "program" }
  checks: [
    { id: "ac-native", adapter: "probe", action: "run",
      options: { recipe: "ac/mir-native-scalar-v1", mn: { entry: "d_main" }, steps: [{ op: "run" }] },
      expect: { completion: "success", exit: -21 } },
    { id: "av", adapter: "probe", action: "run",
      options: { recipe: "ac-av/core-run-v1", av: { entry: "d_main" }, steps: [{ op: "run" }] },
      expect: { completion: "success", exit: -21 } },
    { id: "autovm", adapter: "avm", action: "run",
````

本包所需独立结果：B01源为14：AutoVM旧显示14、AV/AC独立typed值、新A2R/A2C实际编译执行分别记；删除任何required check即使其余全绿仍拒，M1报告不自动供应M2能力。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 旧义务/规范/新能力与RG原件→现TestCase/worker/父裁判→行级来源/独立oracle→A阶段收据或完整B联合认证。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ObligationBinding { legacy_check: CheckId, rule: RuleRef, source: SourceIdentity, successor: PlanTaskRef, observation: ObservationRef }
struct StageCertificate { candidate: CandidateIdentity, required: KeySet, verdicts: VerdictSet, review: ReviewReceipt }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/auto-test-case/src/contract.rs](../../crates/auto-test-case/src/contract.rs) | 现case/check合同复用 |
| [crates/auto-test-case/src/ir.rs](../../crates/auto-test-case/src/ir.rs) | resolved源/产物/oracle/步骤 |
| [crates/auto-lang-core/src/test_cases](../../crates/auto-lang-core/src/test_cases) | 父执行/原始终态与诊断比较，不建新裁判 |
| [scripts/test-groups.toml](../../scripts/test-groups.toml) | 现固定组导航/真实执行选择 |

**状态、错误与资源。** 零执行/空actual/缺required/错source或伪造重算hash/陈旧review/同candidate拼接均拒；infra不能冒语言诊断，NA须有批准依据。 A只核完整语言/新库/四端/FFI/RG，B-pending不当A失败或PASS；A出口不是985整包完成，先有真实能力再出收据。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/cases/ace-m1/b-scalar.case.ad](../../crates/auto-lang/test/cases/ace-m1/b-scalar.case.ad) | 旧case/check身份和14来源 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `758863818ad3c5b5` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-A01/T-A02/T-A03/T-09/T-08/T-02/T-03/T-04 |
| [crates/auto-lang/test/cases/ace-m1/immutable-avm-920.case.ad](../../crates/auto-lang/test/cases/ace-m1/immutable-avm-920.case.ad) | B18/NB18合法负例与邻例 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `96ef60067b3c1dba` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-A01/T-A02/T-A03/T-09/T-08/T-02/T-03/T-04 |
| [docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md](../specs/auto-test-case/ace-m1-architecture-evidence-admission.md) | 复用可信原件/独立review门，M2需绑定自己候选 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `aebb2260fb6cf193` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-A01/T-A02/T-A03/T-09/T-08/T-02/T-03/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 旧case/check→新owner与后端/组合/负控/资源逐条守恒；复用1002已验作者格式，不反等格式试点或AG029/030联合门才验证A。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-A01`、`T-A02`、`T-A03`、`T-09`、`T-08`、`T-02`、`T-03`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### Windows/Linux无libc平台I/O（2026-11-13当前）

依据[原生平台I/O Draft 0.1](../design/stdlib/native-platform-io.md)。M2-A LIB-M2-05/07/08子行接1039/1040/1041与973，Windows/Linux × AV/AC四格各收真实来源/runner/OS结果/独立expected与目标/宿主依赖；新包不重开947、不让早期F0/H0/E0/B0/Hello多等整包，完整A出口核新义务。原FFI/REP/RG/MIX/A2R/A2C及B安装退役不减。

### 原生底座强制依赖（2026-11-13当前）

依据[设计Draft 0.2 §10.1](../design/stdlib/native-platform-io.md#101-依赖的强制含义r02取代此前可并行表述的歧义)。默认自有标准库的认证必须追溯1039/1040的I/O与OS内存区域交付、1041输出及946/972/973的真实Auto算法，原972/973等依赖的传递路径已经形成硬前置。T-A03不能以“可以并行开发”、946/945旧语义证据、no_std字样、声明或临时Rust/C provider关闭原生子行。纯算法能力、临时bridge/宿主依赖和原生完成三账区分；967显式外部FFI与早期Hello保持原范围，不能替默认自有库，原完整A/B/四端义务不减。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

T-A01/02将标准库逐操作六路径及原A2C写入A分母和现TestCase观测；T-09/T-A03核Python真实执行、AV实际provider dispatch、资源/错误/独立oracle和正式来源。原生四OS/后端格只是其子集，缺Python或某AV来源不能签新库A完成；不要求Hello早期切片多等本次全覆盖。

1042是最小Python接缝，不自动证明本包全部语言域支持；需要更大Python语义时消费977/PY已交能力切片或先具名补齐，不反等977整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 新加载机制完成依赖（2026-11-13当前）

依据[设计§8–10](../design/stdlib/backend-provider-coverage.md)与[1043](archive/1043-ace-stdlib-context-assembly.md)。公共.at＋at/c/rs/py来源后缀组成同一逻辑模块，运行后端另选；#in打开临时namespace并让is/if/for继承编译期属性，不扩平台后缀。最小层由1043先交，966继续完整CTE，不能反向等966整包。

A-required接1043加载/#in正式来源及每库实际消费，拒旧944/947有限收据冒新合同、旧后缀旁路、漏Python/任一AV来源；F0/B0/Hello与B安装/退役原边界不变。

本包整包/当前阶段required完成须核1043实际source/统一绑定/最小#in/增量恢复及所需真实后端的正式来源，不以文件存在、provider名字、旧归档或候选语法冒已交。948只审分母/去向，978验装配闭包，985验完整A；普通能力owner仍另交真实外库dispatch与OS/资源结果。原早期切片/调查范围不据此扩大，公共能力缺路径不勾整包完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-certification.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/auto-bindgen/project.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07 |
| SD-04 | modify | `docs/specs/stdlib/project.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-certification.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-MAP/AC-A-COVER/AC-A-EXIT/AC-A-REVIEW/AC-A-LAND |
| SD-NATIVE | add | `docs/specs/ace-migration/m2-certification.md`（沿原owner，仅真实交付后沉淀） | 默认实现选择未明→自有Auto/OS默认与显式FFI分账、长度字符串/目标ABI及来源验证 | LIB-M2-07/08 | AC-NATIVE |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-certification.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

### 1024发现承接合同（2026-11-13，r9，仅规划）

承接1024的F14/F20：测试义务守恒与固定组职责，优先级P2（F12仅保留约束）。用户仅授权复审与分配，尚未授权实现；原支持/AC/任务/阶段/预算不减少，原任务进度0保持。

T-A01接收1024已有逐项保留账，不另造registry；T-A02在现入口补必要观测/固定组职责对账并保父oracle，T-A03汇总真实结果。F30真实聚合fault由1035负责，985消费永久反例与结果；1002只负责四例试点，不能把全量迁移塞给1002或用格式通过代认证。

1002关系：核心职责/原入口反例/固定组治理均不以1002为硬前置；新作者格式迁移或document/IR/adapter重构仅等待G1002（1002 T-03四例七检查及负控通过，T-04独立复审、接口正式合入并绑定SHA）。不等待全语料搬迁；1002不反向依赖本Plan整包。现有M2能力前置不因承接登记被豁免，也不把这些非紧急结构改造伪装成可立即施工。

来源：[1024](1024-ace-m2-code-quality-and-refactoring-audit.md) r2与Git 81d8fc03f10f4a7d179b0bc3a917602406ed7b91。F01–35高风险修复另立1027–1037；本Plan消费正式合入结果并在T-01重核漂移。与在途文件同写须串行，1017成本身份单独协调，不能靠删例减成本。规范影响沿本Plan原SD目标，review补当前实际实现，不改canonical Specs。

### Node首入口与兼容证据接收（2026-11-13，当前）

979/H0、980/HELLO的规范首源直接是AGE node/view/text，Schema参与解析与结构校验，领域名在AGE环境/NodeName位置才识别。985/T-A01..03接收979的V-H0-POSITION/SCHEMA/COMPAT、980的V-HELLO-SCHEMA/V-UI-NODE-COMPAT及981的V-NODE-CHAIN/UI，按正式接口版本及适用输入复用有效证据；依原AC-02/03/06与阶段required核覆盖，不另建认证runner。原widget字节经同一Schema兼容别名的证据与仅改声明头的Node迁移证据分栏，每旧义务只计一次；仅Node副本或截图不能关闭原源兼容。980和981共同机制不等合并任务计数，RG-A03仍核完整UI/原模式及AV/A2R，RG-A04仍核完整混合/Node语义；首Hello不替任一完整门。本轮不产生证书或执行验证。


### 平台I/O规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-PLATFORM-IO | add/modify | docs/specs/ace-migration/m2-certification.md | 笼统provider→本节平台IO/算法/运行/依赖分账 | 无libc平台设计 | AC-PLATFORM-IO |

### 原生底座规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-FOUNDATION | add/modify | docs/specs/ace-migration/m2-certification.md | 算法/provider合同可冒完成→原生实施/验收必须消费已交OS区域/I/O与真实Auto算法 | 原生基础依赖 | AC-FOUNDATION |

### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/ace-migration/m2-certification.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### 加载机制规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LOAD | add/modify | docs/specs/ace-migration/m2-certification.md | 旧有限后端侧文件→本包实际消费1043来源装配/#in/目标依赖及正式来源，未交保持缺口 | 用户确定新加载机制 | AC-LOAD |

### Linux延期与默认自有实现（2026-11-15当前优先）

本包适用范围：**标准库能力及候选SDK的Linux认证行**。[覆盖设计](../design/stdlib/backend-provider-coverage.md)和[平台设计](../design/stdlib/native-platform-io.md)为共同合同。当前Linux分支继续写代码、保完整fixture/expected，但不执行Linux测试、不从Windows启动WSL/VM或隐式准备Linux runner；按1061隔离入口，环境完善后由[1063](1063-ace-stdlib-linux-deferred-validation.md)专项验证。

本包当前完成门调整为Windows原required门＋Linux实现/完整移交审查；Linux每个原操作/case/check及增量保留backend/provider/target/ABI、源/依赖/输入/oracle/未跑原因，标`deferred-required / NOT_RUN`，明确owner=1063。不能改PASS/NA、删分母或用Windows/emit-only/交叉编译代实际Linux运行；实现与验证状态分开。1063不是本包反向依赖；本包可完成当前排程，不冒Linux已验证。旧1040/1041等归档实际证据保留，变化后的实现不自动继承旧绿。

当前A出口按Windows完整标准库required证据＋Linux代码/完整义务移交审查接收，Linux认证保持pending，不能发布“全平台新库verified”。原RG/FFI/REP及非标准库产品、其他目标合同不因本条豁免；986/987按实际发布平台消费支持范围。

AV仍一个执行器，native/C/Rust为显式来源；Windows AV三来源、AC、A2R、A2PY及原A2C等本包原适用门不减。公共Auto算法尽量共享，C/Rust可选实现和目标runtime adapter保留；高级外库显式依赖不代默认Auto库。已完成任务证据保留，剩余work及最终验证/review按本修订核Linux移交；不得把合同修订计作实现进度。非标准库Linux专属项目不由本条自动豁免。

### 当前延期规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-LINUX-PHASE | modify | docs/specs/ace-migration/m2-certification.md | Linux当期必跑→代码/完整义务移交1063，verified保持pending；Windows原门不减 | 用户明确排程 | AC-LINUX-PHASE |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-A01..A03（A）及T-02..04/08/09（B或分阶段并集）及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 删一个required、emit-only冒run、错source/producer/hash、缺oracle均红 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 四端各对独立expected | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | REP-01..07/09及REP-11的源码/候选产品腿逐项有真实证据；REP-11最终安装腿pending交986/987 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 全语言组合负控与故障恢复不可由case总数抵消 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t auto-test-case ace-adapters ace-batch`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 删一MIX输入、缺一个适用目标、陈旧source/producer、emit-only、unknown行均阻预认证；985可以交预认证系统/本阶段证据，但整M2 PASS必须由987合并最终收据。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 2026-11-09 新增验证（待执行）

| Case | AC | 输入与区分性断言 | 入口/证据 | owner/阶段 |
|---|---|---|---|---|
| V-09 | AC-07 | 去手工manifest后use.c/use.rs成功；默认Auto算法+目标选择+File+外库组合实际执行并有独立oracle。删Rust析构、C传递头或平台证据一行即阻签；未安装绝不标installed。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-09；最终验证汇总 |

新增触面职责组：`cargo t auto-test-case auto-bindgen stdlib ace-stdlib ace-foreign-contract ace-abi ace-comptime ace-source ace-artifacts ace-av ace-native ace-a2x`，与原§6组按实际修改取必要并集；T-01核真实参数，新增用例接固定组，不减原平台/ignore/压力条件。948若仅文档审计则只做目标静态检查，真实探针沿原指定组；安装/外库工具和外仓产品必须另核真实命令，语言组不替代它们。零选择/未执行/缺环境不充PASS；本轮没有运行这些测试。

### A出口与测试迁移用例（全部拟议、未执行）

| Case | AC | 输入/实际入口 | 独立结果与边界 |
|---|---|---|---|
| V-A-MAP | AC-A-MAP | 959 L/S+delta、当前原native测试/文件化case/产品用例；T-A01冻结身份与去向；删一条/重复或无owner控制 | 每条义务都有A或B/新check/具名替代理由；非法表实际拒绝，原旧输入/断言强度不减 |
| V-A-COVER | AC-A-COVER | T-A02补M2描述/薄adapter/fixture与各能力永久case；复用cargo t auto-test-case ace-source ace-stdlib ace-adapters及按真实IR触面追加完整组 | 源/错误/值/资源/条件分账，错误旧expected或缺边界不能被总数抵消；typed不替source，emit不替run |
| V-A-EXIT | AC-A-EXIT | T-A03冻结四端/标准库/FFI/RG-A01..06及原增量/资源门；全新跟踪检出执行实际并集，窗口/HTTP单实例 | M2A-01..07全部required闭，无unknown/NOT_RUN；B全产品和最终安装保持pending。失败子进程或缺真实consumer不能PASS |
| V-A-REVIEW | AC-A-REVIEW | T-A04独立查所有A AC/反例/原义务及实际diff/收据 | reviewer绑定revision/SHA，遗漏/延期/workaround具名，缺A-required不pass |
| V-A-LAND | AC-A-LAND | T-A05核已合入来源/SDK/ABI/stdlib/grammar/输入身份；错分支tip或stale proof控制 | M2A-08闭，正式A收据可消费，未合入/WIP拒绝；整Plan与M2发布仍待B |

### 默认库/外库与字符串验证（新增、未执行）

| Case | AC | 实际入口/输入 | 独立结果 |
|---|---|---|---|
| V-NATIVE | AC-NATIVE | T-A01,T-A02,T-A03,T-09，沿本包既定四端/固定职责组与真实owner入口补永久case；默认provider与显式外库各一腿；源码/符号/链接/运行依赖与NUL边界对照 | A默认库真实OS结果、四端长度/编码/owner边界和可选外库调用各有独立expected；隐式libc默认、NUL截断、错ABI/缺owner、emit冒run和缺provider控制不能过；B最终安装/退役腿保持pending。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-A01/T-A02/T-A03/T-09/T-08/T-02/T-03/T-04 | B01源为14：AutoVM旧显示14、AV/AC独立typed值、新A2R/A2C实际编译执行分别记；删除任何required check即使其余全绿仍拒，M1报告不自动供应M2能力。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-A01/T-A02/T-A03/T-09/T-08/T-02/T-03/T-04 | 零执行/空actual/缺required/错source或伪造重算hash/陈旧review/同candidate拼接均拒；infra不能冒语言诊断，NA须有批准依据。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-A01/T-A02/T-A03/T-09/T-08/T-02/T-03/T-04 | A只核完整语言/新库/四端/FFI/RG，B-pending不当A失败或PASS；A出口不是985整包完成，先有真实能力再出收据。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-A01/T-A02/T-A03/T-09/T-08/T-02/T-03/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。


### 1024承接的具名验证

| Case ID | 发现 / 原任务 / AC | 输入/设置 | 实际入口 | 独立关闭oracle | 状态 |
|---|---|---|---|---|---|
| AUDIT-14 | F14 / T-A01/T-A02 / AC-AUDIT | 1024当前M1116/S71/D181/L313/A48/N283/U342及新增修复用例 | 既有inventory/TestCase/fixed group→A义务表 | 原expected/source/完整身份/适用性一一保持；静态账已闭，执行覆盖另计；缺check/重复/错身份负控能拒绝 | 拟实施，未运行 |
| AUDIT-20 | F20 / T-A01/T-A02 / AC-AUDIT | 35成员、4独立职责组、5支持静态库、现regular/perf分轴 | scripts/test-groups.toml与真实native/ABI consumer | 明确ace聚合用途及所需并集，静态库由消费者覆盖；依赖编译不当自身测试；不改1016/1025分档 | 拟实施，未运行 |

原组/平台/ignore/规模与正式性能分轴不变；新格式需G1002，原格式和Rust入口可先验证。每条oracle必须在本Plan原work验证完成前取得，不能移到merge或借登记冒关闭。

### 原生平台I/O验证（新增，未执行）

| Case | AC / work任务 | 输入、独立oracle及边界 |
|---|---|---|
| V-PLATFORM-IO | AC-PLATFORM-IO；T-A02/T-A03/T-09；B的T-04 | 删required格、以cross-compile/mock/AV捕获冒真实Linux或Windowsconsole、混同AV依赖/目标无libc、四端相等无expected时认证拒绝；合法逐格来源/oracle齐全才可接收，缺环境/未执行不PASS。 |

### 原生底座验证（新增，未执行）

| Case | AC / work owner | 输入与独立oracle |
|---|---|---|
| V-FOUNDATION | AC-FOUNDATION；T-01/T-09/T-A03 | 缺1039/1040区域实现或真实OS/review/落主来源、只有旧runtime、伪称Auto算法、只有接口且consumer通过时必须阻相应原生子行与A出口；合法逐操作Auto算法→受检区域/I/O→OS并有四格及独立oracle方可认证。明确检查1039/1040的r2 AC-MEM，不沿旧r1 I/O证据漏掉分配来源。 |

### 后端/provider验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/终态 |
|---|---|---|
| V-COVERAGE / AC-COVERAGE | 全部新增公共操作及其独立expected、六路径/旧A2C、OS条件与正式来源；故意漏Python、错AV dispatch、跨provider销毁、emit-only或同错oracle。 | A-required遗漏与伪运行均阻T-A03签章；合法真实子终态/资源和独立expected齐全才关行，B-pending按原阶段分账。 |

实现/验证沿本包原owner任务、现TestCase/ACE worker与固定职责组；新增Python接缝按真实修改选`ace-a2x`/`a2py`/`auto-test-case`，AV来源边界选现`ace-foreign-contract`/相关AV、host、stdlib组，T-01冻结精确并集，985按T-A01/02。缺平台环境不伪造支持，不把正式ace-perf附加日常收尾；既有完整职责/必要消费者保持。

### 加载机制验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/owner |
|---|---|---|
| V-LOAD / AC-LOAD | 本包普通公共库源/所选provider与#in平台模块；错误后缀/缺或重复实现、错target/namespace、未选SDK缺失、错误producer/旧配方冒新、修改选中与未选输入后恢复 | 本包真实消费者及独立expected/依赖/源身份；负控在调用前拒、失败保旧，未选库不绑定链接。948验去向不代运行，978验候选闭包，985核正式A分母；能力包验证真实执行而非仅文件选择。复用1043身份适用证据，不重复跑其整档代本包；原最终work汇总，独立review核AC-LOAD。 |

### 当前Linux移交验证（未执行产品测试）

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-LINUX-PHASE / AC-LINUX-PHASE | 原能力work任务及最终work验证：逐原Linux操作/case/check/增量核源码分支、完整输入/expected、工具/来源与1063去向；原独立review复核 | 每项有实现与NOT_RUN/1063 owner；删除行、冒PASS/NA、用Windows或仅emit充Linux绿均不能通过移交审查。Windows原完整required组真实执行；当前不启动Linux测试。 |

本节修改旧V-COVERAGE/V-PLATFORM-IO及其他表中Linux“当期实跑”的时序，不改变原独立oracle/规模/错误/资源义务；Linux真实运行在1063关闭。其他环境缺口不套本次延期。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 组合认证与安装阶段分账的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | 新增义务在候选阶段全部可追溯关闭且与原REP/V06/MIX并集；无unknown冒PASS，最终安装/退役仍由986/987闭合。 | T-09/V-09；原最终work验证和独立review同时核本项 |
| AC-A-MAP | work | 旧语言/库及新增构造逐义务守恒、A/B与oracle/owner完整 | T-A01/V-A-MAP |
| AC-A-COVER | work | 新旧并集经现TestCase/固定组接新ACE，持久负控齐，无第二框架 | T-A02/V-A-COVER |
| AC-A-EXIT | work | M2A-01..07全闭且RG真实收据齐、B-pending不丢 | T-A03/V-A-EXIT |
| AC-A-REVIEW | review | A全部AC/diff/源/反例/Spec经过独立复核 | T-A04/V-A-REVIEW |
| AC-A-LAND | merge | A review通过后已合入来源与阶段规范/收据同版，M2A-08闭 | T-A05/V-A-LAND |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-A01..A03（A）及T-02..04/08/09（B或分阶段并集）；原独立review复核 |
| AC-NATIVE | work | A默认库真实OS结果、四端长度/编码/owner边界和可选外库调用各有独立expected；隐式libc默认、NUL截断、错ABI/缺owner、emit冒run和缺provider控制不能过；B最终安装/退役腿保持pending。 | T-A01,T-A02,T-A03,T-09 / V-NATIVE；原独立review核新增义务 |

- [ ] AC-AUDIT / work：1024承接F14/F20的上述每条AUDIT用例及保留边界完成；原任务T-A01/T-A02/T-A03负责，原独立review逐项核，未执行不PASS。

- [ ] **AC-PLATFORM-IO / work**：本节平台合同及V-PLATFORM-IO全部满足，真实来源/oracle/未交边界齐全；原分母/四端/阶段不减，原最终work汇总、独立review核验。

- [ ] **AC-FOUNDATION / work**：V-FOUNDATION与§5原生底座当前合同全部关闭；正式底座包含实现/真实OS/依赖/独立复审与落主身份，纯算法与旧runtime支持不能替代；原最终work及独立review逐项核。

- [ ] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。

- [ ] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。

- [ ] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。

## 8. 执行步骤

当前共14任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A准备｜work｜AC-01/AC-PHASE**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。
  [✅ 2026-11-16] A期冻结记录见§5"T-01 A期冻结记录"：授权链（971 goal 续行指令）、A最小前置核实（语言族959-970/972归档+早期切片967/F0、969/E0、978/B0、979/H0@71472d975、980/HELLO+971/A@2f1eed27f/c1561d94 落主；1039-1043 基础设施归档）、硬前置未闭清单（973/974/975/976/977/978收口/979余量/980/981/982/983/984/OS064）、M2A-01..07门级现值与可执行面结论（M2A-08本轮不可闭合；T-A01/T-A02/T-09/T-08[A]已交来源部分可执行）、平台/资源/命令冻结（Windows x64；Linux腿归1063）。PIO/FOUNDATION 子合同来源状态核实并冻结（1039/1040/1041已齐，973算法腿pending不冒原生完成）；worktree `.wt/lang-985/auto-lang` 已建。

- [x] **T-A01｜M2-A｜owner_stage: work｜AC-A-MAP/AC-NATIVE**：从959及FFI/LIB/UI/AGE增补冻结A/B义务矩阵（纳入五包Hello早期20任务/对应原义务，去重计case），盘点原语言/库用例、原expected/条件与新check去向；每条归继承/具名语义替代/原内部退役/B产品，不按旧目录或M1 NA缩分母，执行V-A-MAP负控。 新增V-NATIVE：在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16, worktree commit 0dff19cd4] 交付 **A/B义务矩阵 `autolang.ab-obligations/1`**（`crates/auto-lang/test/cases/ace-m2/m2-cert-985/ab-obligations.tsv`，162行=959冻结L-01..49/S-01..25/A/B/C/D/E/F/K全部入口产品行+Design56 FFI-M2-01..08/LIB-M2-01..08增补；每行 phase(A/B/AB)×disposition(inherited/named-replacement/internal-retire/product-B)×successor×best-current-receipt；早期切片去重计入对应行：967/F0→FFI行、969/E0→A-01..03/B-01、978/B0→LIB-02/FFI-07(inflight)、979/H0→S-23/L-44、980/HELLO→C-02/B-01(无spec收据保持pending，产品闭口本就B)、971/A→S-24/A-04/A-07/C-06/D-05/D-07/F-07/F-10）。`landed`语义=该行**当前最佳已审收据**（可为早期/A切片，sha16锚定至 docs/specs 或 docs/plans/archive 实文件），全行闭口判定归T-A03——MAP≠EXIT。**V-A-MAP负控**（`crates/auto-lang-core/src/test_cases/obligation_ledger.rs`，7例全绿0.03s，每例对真实矩阵做突变并断言具名RED）：缺行(冻结行缺失=RED不缩分母)、重复行、错source(sha16失配=陈旧/伪造收据RED)、非收据路径(源码/测试自报PASS永不充收据RED)、词表+无owner行、pending虚报收据(证据膨胀RED)+真实矩阵绿。准入层四族(旧binary/emit冒run/假workerPASS/timeout)绑定ac/ac_av准入脊柱(父侧deadline/kill+schema admission+父侧独立裁判worker不见期望)，逐能力绑定归T-A02。子合同去向：V-NATIVE双腿/COVERAGE六路径逐操作/LOAD消费/1024 F14案例试验账——按计划文本归T-A02/T-09/T-A03继续（本任务的义务并入已完成：LIB-07/08在矩阵、F14守恒由矩阵+缺行负控承担）。V-A-MAP真实命令冻结：`cargo test -p auto-lang-core --lib obligation_ledger`（组登记归T-A02）。

- [x] **T-A02｜M2-A｜owner_stage: work｜AC-A-COVER/AC-NATIVE**：先用普通source和FFI/async资源两配方核父侧观测支持；在现TestCase/ACE owner接版本化M2描述、必要typed协议/薄adapter与父侧stage聚合/负控，unsupported保持ERROR；注册拟新增跟踪ace-m2源/expected/适用性/旧新映射，督促每能力owner补规范/组合/边界case，冻结V-A-COVER真实命令，不另造runner。 新增V-NATIVE：在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16, worktree commit d6342f534] **两配方父侧观测核实+M2描述注册+四准入族绑定**：跟踪语料 `ace-m2/m2-cover-985`（plain `ac-av/core-run-v1` exit14 + owned `ac-av/resource-events-v1` exit405，独立手算oracle）；owned-relay 案例在冻结父表登记（frames2/record=2/str=4/free-order[3,0,1,2]——由真实父侧观测运行推导，探针环本身证明未登记案例=RED case-registration 失配，准入控制实际生效）。conformance 双门：`ace_m2_p985_cover_suite_green`（两配方端到端、父侧独立裁判，0.84s）+ `ace_m2_p985_cover_wrong_oracle_is_red`（错exit=RED指名exit轴且局部化，0.83s）。**四准入族绑定具名红控（2026-11-16 实测验证）**：旧binary/陈旧身份=`ac_av_probe_mixed_recipe_stale_identity_is_order_independent`(PASS 234s)；emit冒run=`ace_m1_golden_drift_is_red`(PASS 65.7s)+kind混淆=`ace_m1_native_typed_891_wrong_oracle_and_wrong_kind_turn_red`(组内已知基线红，控制角色在、状态如实)；假workerPASS=`ace_m1_base_negative_oracle_fails`(PASS 65.9s)+`ac_av_probe_negative_oracle_fails`(PASS 0.77s)+新`ace_m2_p985_cover_wrong_oracle_is_red`(本注册负控)；timeout=`ac_av_probe_prep_executor_contained_failure_shapes`(PASS 59s)+`ac_av_worker_contract_faults_are_infra`(PASS 3.6s,故障=infra永不充语言负例)。**V-A-COVER真实命令冻结**：复用组 `cargo t auto-test-case ace-source ace-stdlib ace-adapters`（组存在已核 test-groups 1513/1634；本轮IR未触其代码面，复用证据）＋新执行腿（experimental/ace 工作区）`cargo nextest run -p ace-conformance --test conformance --config-file ../../.config/nextest-full.toml -E "test(ace_m2_p985_cover)"`＋账本腿 `cargo test -p auto-lang-core --lib obligation_ledger`(7/7)。薄adapter未新增（现配方承载，不另造runner）。子合同去向：V-NATIVE长度/ABI观测与COVERAGE六路径逐操作面随能力owner案例继续（T-09/T-A03核）；督促各owner补规范/组合/边界case为持续义务，A批后续/T-A03汇总。

- [x] **T-09｜M2-A｜owner_stage: work｜AC-07/AC-NATIVE**：冻结并审计FFI-M2-01..08/LIB-M2-01..06及全部子行，运行四端真实外库/新库组合与缺行/手写binding/旧桥/缺工具对照，给986/987具名pending安装项。 验证V-09，A证据汇入T-A03；B最终work T-04复核身份适用性。 新增V-NATIVE：在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16] 已交来源部分交付（详§5"T-09 已交来源审计记录"）：FFI-M2-01..08 全部绑定 967 具名 fixture 并**当前树实测 33/33 PASS 9.2s**（真外库 serde_json/SQLite 端到端+负控矩阵+六路径）；LIB-M2-01..08 逐行审计（已交证据/pending owner）；对照族（缺行/手写binding/旧桥/缺工具）逐项核对；986/987 具名pending安装/退役项已登记。V-09 的 A切片证据就绪汇入 T-A03；pending 腿（973/974/975/978/977 各腿）如实保留，T-A03 出口前由其 owner 闭合。

- [ ] **T-08｜M2-A/B｜owner_stage: work｜AC-06**：分A/B执行原MIX-01..06与候选组合：A切片只等T-A02和能力/RG正式来源，进入T-A03；B切片在M2A-08后接T-02/03全产品进入T-04。保AC-06/V-08完整范围，B-pending明确交986/987，A完成不提前勾本任务。

- [ ] **T-A03｜M2-A｜owner_stage: work｜AC-A-EXIT/AC-NATIVE**：等A能力和RG收据正式就绪后，运行V-A-EXIT的实际完整职责/四端/新库/FFI/真实消费者及原正式门；新鲜检出重放、逐项计数/终态/独立oracle齐，A-required缺证阻出口。 新增V-NATIVE：在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-A04｜M2-A｜owner_stage: review｜AC-A-REVIEW**：停止A写入，独立复核M2A-01..07与全部A任务/反例/遗漏/健康/Spec；结果绑定A revision/冻结SHA，修复后定向复核，不能用executor总PASS替证据。

- [ ] **T-A05｜M2-A｜owner_stage: merge｜AC-A-LAND**：A review pass后按phase fold核各owner真实合入SHA及基线祖先/SDK闭包，落本阶段Spec与M2A-08收据、具名B-pending；同Plan树保留继续B，不归档985或宣称M2/v0.6完成。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [ ] **T-02｜M2-B｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。

- [ ] **T-03｜M2-B｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。

- [ ] **T-04｜M2-B｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-05｜M2-B｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-B｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-B｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

B收口：所有B work、原全产品/组合及A证据适用性齐→原全包独立review→正式合入/Spec/归档→输出对账/清理；各批R与全包final分别执行，不能只数批次替原AC。

T-08是唯一跨阶段任务：上方只登记一次，A片段关闭是T-A03的前置，B片段在T-04前关闭；勾整项须A/B都完成。T-A05是阶段来源交接，整包T-06/07仍在B，旧安装/退役义务完整交986/987。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-985/auto-lang`，分支 `plan-985-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-985/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-A01/T-A02/T-A03/T-09/T-08/T-02/T-03/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

stage: work | PLAN-985 | r16 | 2026-11-16 | outcome: pass (T-09 已交来源部分) | worktree: D:/autostack/.wt/lang-985/auto-lang @ plan-985-dev | code_commit: 无新代码（审计消费既有967 fixture；实测命令与结果见§5审计记录） | task_ids: T-09 | evidence: f0全集 33/33 PASS 9.2s（构建45.8s，机器占用核查=0进程后启动）；FFI-M2-01..08行→具名fixture绑定全实测；986/987具名pending项 | blockers: T-08[A] 的 RG 正式输入（982-984/OS064）未落——MIX组合A腿仅t15类候选证据，正式MIX-07需File(973)；T-A03..A05 阻塞链不变（973-984/OS064 A批） | next_responsible_role: 985 A批在无新能力落主前已达当前可执行边界（T-01/T-A01/T-A02/T-09已交来源部分）；后续=等能力owner落主后T-A03，或按用户裁定转无主包A批（974/975/976/977/980/982/983/984，建议977）

stage: work | PLAN-985 | r16 | 2026-11-16 | outcome: pass (T-A02) | worktree: D:/autostack/.wt/lang-985/auto-lang @ plan-985-dev | code_commit: d6342f534 | task_ids: T-A02 | evidence: §8 T-A02[✅]行（两配方观测/m2-cover注册含冻结父表真值/四准入族具名红控实测/V-A-COVER命令冻结）；新用例 ace_m2_p985_cover_{suite_green,wrong_oracle_is_red} 0.84s/0.83s ≤10s门；绑定控件7项实测（6绿+1基线红如实注明）；obligation_ledger 7/7 复验 | 机器占用: 等待外部组任务至0进程后启动conformance构建 | blockers: T-A03..A05 仍阻塞于 973-984/OS064 能力owner A批（不变，见T-01冻结记录） | next_responsible_role: T-09/T-08[A]（已交来源部分可推进：FFI-M2行审计/新库组合候选）或按用户裁定转无主包A批

stage: work | PLAN-985 | r16 | 2026-11-16 | outcome: pass (T-A01) | worktree: D:/autostack/.wt/lang-985/auto-lang @ plan-985-dev | code_commit: 0dff19cd4（矩阵TSV+校验器7红控，基面 b8f80afcf） | task_ids: T-A01 | evidence: §8 T-A01[✅]行（162行矩阵+7例红控0.03s绿+早期切片去重记账+子合同去向）；真实命令 `cargo test -p auto-lang-core --lib obligation_ledger`=7/7 | 新用例≤10s门：7例全部0.03s | dependency_revisions: 沿T-01冻结记录 | blockers: T-A03..A05 仍阻塞于 973-984/OS064 能力owner A批；T-A02 为下一就绪任务（M2测试描述/薄adapter/真实命令组登记/准入层四族逐能力绑定/COVERAGE六路径/V-NATIVE观测扩展） | next_responsible_role: work（T-A02；T-09/T-08[A] 可按已交来源并行）

stage: work | PLAN-985 | r16 | 2026-11-16 | outcome: pass (T-01) | worktree: D:/autostack/.wt/lang-985/auto-lang @ plan-985-dev（基面 master@b8f80afcf，本轮纯冻结核验无代码提交） | task_ids: T-01 | evidence: §5"T-01 A期冻结记录"（A来源正式交付账/M2A门级现值/硬前置未闭清单/平台资源命令冻结）；worktree与plan-artifacts/plan-985已登记 | dependency_revisions: 见冻结记录（语言族归档+早期切片+1039-1043；973-984/OS064 A批pending） | blockers: T-A03..A05 阻塞于 973-984/OS064 各能力owner A批正式落主（含他会在途的973/979/981与reviewed待收口的978）；M2A-08 需 M2A-01..07 全闭——本轮结论为不可在本轮闭合，971/B 及本包 B 任务的解锁链随之顺延 | next_responsible_role: work（T-A01 义务矩阵+V-A-MAP负控为下一就绪任务；其后 T-A02；T-09/T-08[A] 按已交来源并行推进）

2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r15：按用户最新裁定修订Linux验证时序、移交1063与自有普通库边界；不变原任务ID/状态/进度/Windows门，保Linux原分母及独立oracle。原已完成证据不因本条重记通过；未实施或运行测试，规划静态结果另记。

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-13 原生底座依赖修订预检：9份文档371处本地链接通过；保原48任务ID/阶段及已有AC/V，新添1039/1040各T-09共2项work；可达149个语言计划无缺失/环，946与972新增硬前置无反向分配器依赖。状态/勾选/进度未推进，归档计划与canonical Specs零修改，.next-id仍1042。仅规划静态检查，非产品/独立实现review PASS。

2026-11-13 stage:new / r12：修正原生底座完成依赖；原任务/状态/勾选/进度保持，新增FOUNDATION AC/V/SD由原work任务和独立review承接。未启动实现或运行产品。下方较早可并行措辞仅对算法准备有效，不覆盖本次原生实施/验收门。

2026-11-13 stage:new / r11：设计承接；新增平台I/O AC/V/SD由T-01,T-09及原最终work负责。原任务/进度保持，无实现或独立产品review；新三包drafting。

2026-11-12 stage:new / PLAN-985 r8：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原14任务/全部AC与V/依赖/阶段/预算和drafting 0/14；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12 stage=new / r7：按用户标准库/FFI方向补LIB-M2-07/08本包义务，绑定原任务与新增AC/V/SD；任务ID/总数、原AC与阶段门保留，drafting零进度。只修改规划，未实现/运行/签章或改变M1。

2026-11-11 / stage:new / PLAN-985:r6 / outcome:pass（Hello切片接收口径可审阅）；blocked（未实施/完整A能力待交） / next:原T-A01..A05继续，AV首窗不冒完整RG-A03，保原14任务/AC/V和零进度。仅规划，不创建认证器/证书或运行。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部14个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按跨M2-A/B拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE及985/A五任务与旧义务迁移/新测试合同，当前0/14；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r3范围修订：新增T-09..09、AC-07及V-09..09，按Design56正式接M2 FFI/新版标准库责任，0/9。stage=new；仅文档合同与静态预检，非实施授权、产品验证或独立实现复审PASS。旧r1/r2/r3记录为历史。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。


2026-11-13 stage:new / r9 / outcome:pass（规划承接） / next:原前置及实施授权后work。用户授权将1024的F14/F20纳入本包原任务，新增具名AUDIT oracle/AC-AUDIT；未增任务、未删原任务/AC/V/阶段/预算、0进度/drafting保持。旧r8规划记录保留；实际实现/执行未开始。1002依赖按任务分流，无整包循环。

2026-11-13 / stage:new / PLAN-985 r10 / outcome:pass（规划合同预检，非实现或产品review） / changed: Node首源、Schema/NodeName/兼容与并行合同及对应AC/V/SD；原任务数量/状态/进度与旧义务保留 / next: 本轮仍只规划；后续获得实施指令且该任务最小正式来源齐时进入原work。全部新增验证未执行，未构建/运行/建树/派agent/合入或改963。

2026-11-13 本轮仅规划的静态核验：4份计划原95任务ID/勾选/数量/状态/进度与依赖元数据保持；新增3项AC、7项拟验证及3项拟Spec增量，全部未执行。9份文档本地引用、0–10章节、表格/围栏与新AC/V任务映射核对通过；F0→E0→B0与H0汇合HELLO、随后981按接口扩展的切片顺序不新增反向整包等待。963及其实施树未由本轮修改；未生成fixture/binding/SDK、未运行编译/产品/GPU/性能、未建树/提交/合入/tag。此为规划预检，不是实现独立review PASS。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
