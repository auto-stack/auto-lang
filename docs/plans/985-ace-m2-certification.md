---
plan_id: PLAN-985
status: executing
feature_name: M2两阶段验收与完整义务联合认证
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-17
plan_revision: 30
current_step: 6
total_steps: 14
depends_on: [PLAN-971, PLAN-972, PLAN-973, PLAN-974, PLAN-975, PLAN-976, PLAN-977, PLAN-978, PLAN-984, PLAN-941, PLAN-1064, PLAN-1065, PLAN-1068]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/auto-bindgen/project.md, docs/specs/stdlib/project.md]
new_spec_components: [docs/specs/ace-migration/m2-certification.md, docs/specs/auto-test-case/ace-m2-stage-verification.md]
touched_goals: [GOAL-002]
---

# [PLAN-985] M2两阶段验收与完整义务联合认证

## 0. 变更摘要

2026-11-17 r29当前（/goal 完成计划985/A后续工作，work轮）：**T-08[A]＋T-A03执行完毕，A批work收口，移交T-A04独立复审**。T-08[A]交付MIX-07同一工程组合（`f0_t16_mix07`两例：1043平台文件选择→配置读取＋钉版serde_json真机解析（use.rs）→值流入use.c SQLite写入/聚合回读→双车道协作取消＋V-08具名负控；装配道扩展dep标注走真依赖cargo构建）；MIX-01..06 A腿逐行绑定owner正式证据。T-A03重冻：矩阵L-45/S-20/S-21/S-25按已记录裁定归B波、B-02落landed（已交A2C事实）、B-04/B-06 user-deferred（M2-TARGET-SCOPE允许清单＋never-PASS守卫＋新负控），obligation_ledger **8/8绿**（新TSV SHA=89eebafa…）；V-A-EXIT并集实跑：ledger 8/8、p981 205/205、ace-abi f0 36/36（含MIX-07 4.37s/5.63s）、m2-cover 2/2（53.3s）、固定组四组合选95例94绿＋1基线红（`source_parent_oracle_failure_rejudges…`，master复放同红=既有红非本批触面，归批量回归triage）。M2A-01..07门级全闭（见§5 r29执行记录）。B批（T-02..07/T-08[B]/T-04）不动，等M2A-08。

2026-11-17 r28当前：**981接收对账通过，A-required外部输入全齐**。MIN-A-AGE整体交接收据（e8d637bb9：event A＋B-PROOF-I/V[A]，review-71 pass）绑定revision全部核验落主（lang 0520c7f3b/6340934a1/ad7882cbe、ui ab6fff5c/1eb27697等）；本轮复跑p981全集205/205、**PROOF A批4/4判别绿**（g01真实2D/g08派生3D/坏编辑保旧恢复/UI+2D+3D组合出口预算）；r26反馈①–⑤全闭（簿记同步b04f74529、T-01已勾、EXIT-CONTRACT关闭＋永久用例）。R69-2替代口径裁定接受（A-min业务无带参路径，具名拒＋零效果满足安全要求，逐项诊断归B-W1已具名）。**985/A等待点结束：下一步＝T-A03重冻＋完整A并集→T-A04→T-A05（M2A-08），属本包自身执行，待实施指令。**

2026-11-17 r27当前：**1066/A验收通过**（IF-1066-A-CORRECTNESS正式接收：实施→T-12[A]独立复核pass（F-1066A-R1/R2修闭@2e606d28f）→T-13[A] ff-only fold（range-diff全等，master tip=7c131286d）；本轮复验零漂移＋落地检查`cargo test -p auto-lang --lib plan1066` 5/5绿）。**981材料已基本齐备**（r26反馈①⑤已闭：B-PROOF-[A]真实2D/3D源落主393e0cc75/a181a1bb4、review-69 pass收据6bbbcf909；②tier-1 A面终验收据61e52ea44＋固定组gate_passed；④R69-3 EXIT-CONTRACT改判981/LANGUAGE、R59-5定证；③簿记3/35按整任务规则留待不阻接收）——待一轮985接收对账（含R69-2替代口径裁定：A以"带参/带载荷一律不可执行具名拒"兜底，逐项诊断归B-W1）。**待交仅剩981接收对账**；T-A03重冻裁定项不变。

2026-11-17 r26当前：第二轮接收——**OS064/A正式落主可收**（auto-os ff-only 2383d72→fb1192c→06d3a0b＋簿记484c432/goal收据61237ba，IF-OS064-A-MIN交接账齐：真窗372帧/双实例2:1/错标签-5/错句柄-1/撤权恢复/无旧VM，消费身份ui 403c936d/lang 7f431bd5f）。**981差额澄清并已反馈**（见§5 r26反馈块）：其review-68批级只闭event A＋LANGUAGE-V/SESSION-V/RUST-V证据面，**PROOF批A面（g01/Showcase最小UI+2D+3D+事件/更新/关闭）未交**，MIN-A-AGE整体交接收据缺失；绑定版本（lang 24b4ba4a7/75015a49b、ui 771be6a9/7de28330）均已落主。**待交收敛为两件：1066/A＋981 PROOF批/整体交接**；T-A03重冻裁定项不变。仍停等待点。

2026-11-17 r25当前：有界985/A接收预检完成（见§5预检记录）——审计A正确性8包1029–1037及973–979/1064/1065/1024全归档，982/983/984-A切片与977/PY落主逐项核验，#6i冻结身份两红当前master实测转绿；义务矩阵19行新落landed＋29行重钉（ledger 7/7绿，worktree已resync master）。待交=981 MIN-A-AGE余量、1066/A、OS064/A三件（另L-45/S-25/B-02等为T-A03重冻裁定项，非交付缺口）。本轮停在等待点，未跑T-A03最终并集或T-08[A]终验，不重复预跑。

2026-11-16 r24当前：EVENT-SPLIT-20261116明确A/B逐切片等待，三消息缺口归981原LANGUAGE/SESSION、980原AV/WEB消费者及985原认证任务；复用980/NATIVE与#6i，A关最小真实事件/保真/安全拒绝，B关完整payload/绑定/WEB及三端产品。旧980/981/985整包互等不再作为前置；原任务/勾选/完成数和全部验收保留，仅规划。

2026-11-16 r21当前：A出口须核980/NATIVE真正AV＋Rust FFI可复用Node桌面入口，Hello字节验证/宿主绘制不足；AC桌面按1068/T-04及981 B正式接洽在B补齐。原14任务/4完成及历史收据不变，不以规划修订重记通过。

2026-11-16 r20当前优先：M2最终AV/AC/A2R公共能力一致，A2C完整对齐增强；A允许AC具名差额、基础必须真实可用，B经1067/1068和AAC实际模块逐步补齐，最终三端差额=0。本次调整阶段及目标要求，原4/14历史进度/任务不重记通过，新分类及判据须T-A03/原review重冻；本轮仅规划。

2026-11-16 r19当前：用户授权明确原后延的实际任务/平台/验收。973已归档，MODULE实际承接1064/T-MODULE-I/V/R/D；B1由975/T-B1-V/R/D先独立正式交付，1064其余D行/输出帧/AC按1064 r2与1065 r2收，原义务并集不减。现仍executing4/14，M2A-08未交；不是重开973、不是新增阶段出口，也不反等975/1064整包后才接早期切片。本轮只规划。

2026-11-16 r18当前承接：973已reviewed 7/10，并将byte→Str等D-01..D-09交1064、AC系统操作真身A-01..A-04交1065；本包原系统库/字符串/覆盖A义务沿新owner守恒，不能以973有限面归档关闭完整库。前置并入1064/1065，但不反阻973收尾；977/A只消费已交PY，不等待977的B/REVERSE或全包归档。原14任务、4完成、executing保持，M2A-08未交。本轮仅规划。

2026-11-16 r17（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包按新用户范围重校目标义务矩阵/adapter/阶段门：Godot为用户后置非required，TS/JS按生产消费闭包required。r16已交矩阵/旧check只作历史基线，T-A03须重冻版本和差异反控后才验收；保4/14真实历史进度，不据范围缩减签M2A-08。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

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

2026-11-16 r22当前：A出口改MIN-A真实基础/代表性验证；完整UI示例/Widgets及图形三端按B W1 AV→W2 AC→W3 A2R独立交接，再迁OS。14任务/4完成历史保留；原矩阵/适用性须T-A03按新revision重冻，旧PASS不证明新出口。最终库/产品/安装退役义务不删。

2026-11-16 r23当前：验收合同冻结ID A-FREEZE-20261116-r23。原162父义务按逐check/backend/consumer拆A/B，1024 F01..35全部有阶段/owner/关闭条件；1029与1031–1037共八个正确性包为A，1030历史外层与1038研究B。1066只取A正确性切片，完整载体迁移/250编译归B。14任务/4历史完成保持，不代表实际矩阵已改/新A结果已通过。

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

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

### 2026-11-16当前取材与移交核实

用户授权本轮只核最新状态与执行次序，不实施。取材主线fe75d239b745d3a2652208e9d22ac6c987c75939；973 r13/第7轮final pass绑定0a6fb9100，尚未为主线祖先；只读收尾观察到其原树已9d0f973cb（含d59c92331 rebase修复及规范提交），须原owner核冻结身份与证据适用性，不能把旧pass无条件迁到变化后实现；1064 r1为0/7、1065 r1为0/6，均已存在，复用不取新号。973用户已授权的后继延期是本包原义务的owner变更，不是完整M2库/AC门豁免；Linux1063的延期与Godot本版用户排除仍各按原合同。

本轮冻结说明：162父矩阵SHA见本节取材，1024 F01..35与1066完整S/N/P/C/E/B映射沿现有原件核对；N0028/N0030现doc确少vm-result，旧succ/10 hash与1066 r2一致。当前所有新分类只是规划，不是T-A03实际adapter或运行报告。

### 本轮交接取材与范围（2026-11-16）

读取主线9d0f973cb及原973归档/975分支02a687da8、1064/1065 r2。用户仅授权规划修订，不实现/派agent/建树/测试/恢复目标/合入/清理。既有r18的D/A守恒保留，当前已验状态从原owner正式source/收据接收，不把本轮新设计或旧分支绿充运行证据；原973归档只补前向交接说明，不重开其终态。

本轮用户授权：仅修订现有980、981、1068、OS064、985–987及相关路线，不新立项、不启动实施。采用A最小真实交付→985/A M2A-08→B完整UI/Widgets的AV→AC→A2R三波→AutoOS全量迁移→候选/安装/退役。读取语言主线aa257fa1425c31fab48bd544cbd116b2fdce7dcd及现有Specs/已交HELLO、AXR0、SOURCE；UI示例真实源在auto-ui/examples/ui，Widgets源在auto-os/widgets-gallery/src/front/{pages,components}，ui-gallery副本需和规范源去重。新增接面/用例/收据仅是合同，不代表已运行。语言/标准库/非图形FFI的既定基础与测试守恒仍保留，不能借UI延期把普通语言必需能力全部后移。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

实际矩阵源只在985唯一worktree存在，主检出同路径尚不存在：`D:/autostack/.wt/lang-985/auto-lang/crates/auto-lang/test/cases/ace-m2/m2-cert-985/ab-obligations.tsv`，163行含表头/162父行，SHA256=`2c394a7db676dc055fdd4e683d61ae2360e111e81aa4cb155ade9126c6fa0a93`。本轮只读取材并在本Plan冻结规范化分类，**不改worktree运行矩阵/adapter、不把分支材料当正式D**。未来T-A03把本合同落实原矩阵/父adapter、真实验证并独立审查/合入，当前不能宣布“实现已重冻通过”。

### 本轮事件阶段修订的授权与取材（2026-11-16）

用户明确仅修订980/981/985及相关路线，落实三缺口、A/B边界/任务/验收与无环顺序；不启动实施、不新立项、不建树/运行/恢复目标。原勾选/完成数/状态/依赖并集和历史结果保留，语义合同revision递增。本轮读取主线`f7f6843a1f8c6c248232014831fa4d8df9ffdeb9`、auto-ui `876c99a3f69c5ae12f9ddf20bdada0ad7481e3d5`；979已归档，仅复用已交平台不重开。`m2-autoui.md`为当前已交规范；`docs/specs/ace-migration/m2-age.md`尚无正式文件，是原Plan拟新增Spec目标，不冒当前依据。

| 已读源/规范 | SHA256前16 | 静态用途 |
|---|---|---|
| `D:/autostack/auto-lang/experimental/ace/grammars/auto-source.at` | `5bc6ec43d65e680c` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-lang/experimental/ace/crates/core/src/source/age_component.rs` | `c5780b5c7419bfda` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-lang/experimental/ace/crates/core/src/source/cst/project.rs` | `2a688a55bff2c931` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-lang/experimental/ace/crates/av/src/age_component.rs` | `cce492ce3013be2f` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/ace_state_bridge.rs` | `cc12cc5bef6141f0` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/ace_ts_provider.rs` | `10a804e8f75006f1` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-ui/crates/auto-ui-runtime/tests/ace_ts_provider.rs` | `204368e0642c8fcb` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-ui/crates/auto-ui-runtime/tests/ace_node_desktop.rs` | `0de0db78c17e9535` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-ui/examples/ui/ace-node-counter/src/front/app.at` | `6016eb863f94499e` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-ui/crates/auto-ui-runtime/tests/vue_capabilities.rs` | `ebc68e21966eadf3` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |
| `D:/autostack/auto-lang/docs/specs/ace-migration/m2-autoui.md` | `bf01874c6fa2e8ba` | 原源/现结构/独立oracle或已交规范；非本轮运行证据 |


补读原冻结身份测试`experimental/ace/crates/core/tests/cases/plan981_grammar_scope.rs`，SHA256前16 `9bd4c795284c0da4`；属原SOURCE/LANGUAGE的防漂移用例，新增r41取材未运行或修改该文件。

## 5. 详细设计

### B-CONFIG-20261117：配置缺口接收责任（A收据冻结不变）

本revision只补B合同，已交M2A-08绑定r29/95c7fbad8与route C原接受面不重写、不重新签A。GAP-982-CONFIG-ACE唯一producer为[982现B-CONFIG-I/V/R](982-ace-m2-man-gen.md)，设计见[配置文档求值](../design/ace-config-document-evaluation.md)，覆盖auto-lang共享前门/AV/SDK和auto-man消费。不得继续以969已归档=配置已交，也不以“无owner”让整个B无限等待；新能力本轮尚未实现。

T-02/03逐行接IF-982-CONFIG-ACE的正式repo/SHA、Schema/值协议/源-context/SDK/ABI、动态原例与独立值、零fallback/失败保旧/修复下一读/冷恢复/原平台；T-08[B]核配置数据确实流入真实CLI/服务/构建。T-04最终认证须982完整产品及983/984/OS等各真实consumer适用配置格闭合；配置主体不得由本包重写。配置宿主AV不是AC guest降级，原机器业务与产物判据仍单独验。

本包不作为982 CONFIG生产前置；986安装/987退役消费本包已审落主候选，不反卡本包候选。新配置归B，不影响A-only基础与三UI波既有次序。

### EVENT-SPLIT-20261116：A/B交付与等待的当前权威

本节落实用户本轮修订，优先于旧§9中的“980 executing所以981不能验V”、round39/40三条件总等待和旧全量A前置。历史收据保留；本轮不改变自动目标状态、不恢复执行。前缀`B-`是既有批次ID，不代表该ID全部属于M2-B，执行以原任务的A/B子面及本节为准。

正式A来源已交：`IF-980-NODE-DESKTOP-A`（auto-lang `7f431bd5f`／auto-ui `403c936d`），源为`ace-hello-minimal`双源与`ace-node-counter/src/front/app.at`；独立oracle可直接绑定`tests/ace_node_desktop.rs`中的预期轨迹/状态及e2e断言，不强制另造不存在的expected文件。交接至少绑定repo/SHA、源闭包hash、原/脱糖source map、Schema/profile、SDK/host/provider ABI、用例/独立expected位置及实际结果。**980整包executing不等于正式A源未交；已交源也不自动证明981领域组合验收已通过。**

刚落主的980 WEB #6i（lang `b043117b8`／ui `876c99a3`）已在共同grammar增加`msg Msg {...}`及`.Add(t) -> {...}`入口，并更新SFC体定位/拼接与census；只按实际范围复用，不重复造语法。当前静态事实仍为：`ComponentMsgVariant.payload`保存类型拼写，`ScenarioOp::Send`仅`instance/msg`，`ComponentHandler`没有参数列表，CST的`msg_name/params`还需真实保真/绑定核验。已有`param-handler`样本体为`.n = 1`且不读取`t`，其spliced=1不能证明payload传递或参数绑定正确；本轮未运行产品、不能据这些静态事实签修复PASS。

**#6i冻结身份交接是A必闭，不是B参数功能。** 980 round41新收据报告`ace-source 801/803`，红项位于`core/tests/cases/plan981_grammar_scope.rs`的`p981_grammar_ids_match_the_recorded_frozen_values`与`p981_age_package_grammar_id_is_frozen`；本轮未运行复测。实际承接为**981 T-01＋B-LANGUAGE-I/V/R[A]**，980原B-WEB owner交#6i差额/源与新旧身份清单，985 T-A03/A04拒过期来源。先核`age-component/age-package`规则、Schema/构造表与源/原span的预期变化和其它profile影响，保合法/坏源/普通标识符对照，再按审定新基线维护冻结期望/注释、SDK/缓存身份及实际负控并独立phase交接。不能仅把哈希替成当前值求绿，不能豁免两红或让A等完整WEB/M2A-08；旧身份必须按既有MISS/拒绝规则失效而非误信任。本项并入IF-981-EVENT-A，不新建计划、不宣称已修。

| 缺口键 | A必须关闭的有界责任 | B保留的完整责任及具体任务owner |
|---|---|---|
| UI-EVT-PAYLOAD | 981 T-01与B-LANGUAGE-I/V/R[A]核空载荷unit消息、声明/调用一致性；B-SESSION-I/V/R[A]用已交980源验真实AV事件/双实例/关闭恢复。合法但当前车道不支持的有载荷执行必须具名拒绝且零效果；不能丢参数后执行常量体、补假值或伪装unit成功。 | 981 B-LANGUAGE-I/V/R[B-W1]承接共享ScenarioOp及声明→typed参数→唯一verify的真实载荷语义；981 B-SESSION-I/V/R[B-W1]承接AV队列/宿主调度/世代/所有权；980 B-AV-I/V/R消费AV，B-WEB-I/V/R消费受检TS/JS。AC域格由既有1068/T-04＋981 FUSION在W2接，A2R由980/981 B-RUST在W3接。 |
| UI-EVT-MSG | 复用#6i语法；981 B-LANGUAGE-I/V/R[A]核匿名/具名unit声明的名字、变体、原span保真与重名/未知消息拒绝，合法具名unit源对同一独立轨迹；不把普通标识符升全局关键字。完整消息类型命名空间/quoted等不借此扩成A全量。 | 同一981 LANGUAGE[B-W1]补全部原msg声明/变体/类型/引用及错误表面，交IF-981-MSG-B；980 B-WEB-I/V/R核原具名/quoted/emit公开接口与实际新provider；无需重开979或新立后继。 |
| UI-EVT-BIND | 981 B-LANGUAGE-I/V/R[A]读取并保留#6i handler参数名/span，核unit空参数合法、重复/错arity/未绑定名字拒绝；有效带参行为未实现时在verify/admit或dispatch前具名能力拒绝，零状态/UI效果。必须区分“解析成功”“发射/拼接成功”和“参数实际执行成功”。 | 981 LANGUAGE[B-W1]承接handler形参与消息载荷逐位置/类型的绑定、作用域与源映射；SESSION[B-W1]消费typed值；980 B-AV/B-WEB承接实际事件参数/$event及旧例消费。内联lambda铸造的原span→合成handler来源映射也归该LANGUAGE/980既有任务，不能继续写“谱系后继”。 |

上述A是既有MIN-A真实事件/错误安全义务的细化，不要求在A实现全量UI带参业务。**普通函数/node参数、既有任务/actor消息、库/非图形FFI等原A语言义务不后移。** 981/T-01须把原MIN-A-UI/AGE/OS实际源与上述三键逐行对应：真实A业务确实依赖某项载荷/绑定时，只提升所用最小子面到同一LANGUAGE/SESSION A任务，补独立正反例并正式交；不得改原业务/expected以逃避缺口，也不得把整个WEB/全UI趁机列A前置。提升影响r23原A/B原子行时由985 T-A03记录映射/来源理由，A-required不打折。

拟定交接`IF-981-EVENT-A`是上述A核对/guard与实际最小事件结果的phase来源，不替MIN-A-AGE全部UI/2D/3D验收。B分别交`IF-981-EVENT-PAYLOAD-B`、`IF-981-MSG-B`、`IF-981-PARAMS-B`；可同一冻结revision联合review/fold，不创建新Plan/runner/注册器。每件必须列明确支持/具名拒绝、source/Schema/SDK身份、独立oracle、producer→consumer和未闭行；部分语法收据不冒完整交接。

### 985逐项接收与A出口门（r24）

985是消费者/认证者，不生产事件执行语义。T-A03按r23的162父行与原子键接IF-980-NODE-DESKTOP-A、IF-981-EVENT-A和MIN-A-AGE/OS等A真实来源，保三缺口B行的源/原V/owner/波次/关闭oracle；不等980/981整包或三个EVENT-B全件。r23八个审计A包的义务不变，1032/1034已归档只复用正式来源，剩1029/1031/1033/1035/1036/1037仍必闭；1024收尾、1064、1066/A及982/983/984/OS064 A仍在案。

T-A03可先做分类/adapter/已交证据接入准备；最终完整并集与T-08[A]要所有A-required实际结果齐，不能提前签。T-A04独立复审后T-A05才落M2A-08；A接收不反等1068 UI-W2-AC、完整WEB/TAURI、W3或OS全量。T-02/03/T-08[B]/T-04在B接三事件完整producer、各波和产品消费，最终仍须保全原义务；基础A若实际失败不能拿B登记抵消。

### 985/A接收预检记录（2026-11-17，有界——T-A03准备面，不签出口）

用户授权一轮有界接收预检（准备/接收先行，最终联合验收等全部A输入）；本节为执行记录，非T-A03出口PASS、未勾任务。

**已接收正式来源（逐项核验落主身份，master@c2f3b2aa9时点）**：

| 来源 | 落主身份 | 备注 |
|---|---|---|
| 审计A正确性8包 | 1029/1031/1032/1033/1034/1035/1036/1037全archived | r24冻结时仅1032/1034在档；1037 delivered b8ab04897 |
| 库/网络/数据/TS/构建/schema族 | 973/974/975/976/978/979全archived | 973有限面经1064（archived 13/13）/1065（archived，W36真身/N14具名不适用/Linux交1063）闭合 |
| 1024收尾 | archived（c2f3b2aa9＋f046f4ae4） | 只读审计交付落主 |
| IF-980-NODE-DESKTOP-A | lang 7f431bd5f＋ui 403c936d/a8ace167 | 均master祖先 |
| 982/A（route C） | lang eaad93599＋auto-man 11fe79c；spec m2-man-gen.md | GAP-982-CONFIG-ACE具名B，A收据已记985/A handoff |
| 983/A（B-PORT+B-PROOF） | auto-web 93712e5/35b9e47/bdc48e8＋lang收据d43e7970f；spec m2-web.md | phase review pass后fold |
| 984/A（B-EMBED） | lang f77d87722（p984_embed.rs＋m2-os-apps.md） | 工作树f35c318b1经fold落主 |
| 977/PY切片 | 56aadead4；spec m2-other-targets.md | B面（B-07/08/09）仍pending属B |
| 971/A | lang 2f1eed27f＋ui c1561d94 | 复核沿用r16记录 |

**#6i冻结身份两红已闭（实测）**：当前master `p981_grammar_ids_match_the_recorded_frozen_values`＋`p981_age_package_grammar_id_is_frozen`＝2/2 PASS（nextest run ea9d73eb，ace-core conformance；`test(p981_)`全集201/201绿）——980 round41所报ace-source 801/803两红由981事件A切片（7ddfbb77a＋review-65/66修复a5214be59/ed807f393）实际关闭。985接收侧不再按两红拒981来源；正式IF-981-EVENT-A phase收据仍待981批级fold（其工作树在review-69）。

**矩阵更新（worktree resync master@53828f3db后，commit 9af917cbb）**：19行新落landed（S-07..S-18库/网络/数据域、LIB-M2-02/05/06/07、A-05/B-05 TS、B-03 PY；锚m2-stdlib-{io,data,network}/m2-build/m2-typescript/m2-other-targets现行sha16）＋29行既有钉值重钉（m2-aggregates/m2-host/m2-schema因后续已复审落主更新，防陈旧收据误红）；obligation_ledger 7/7绿；新TSV SHA256=`c23c8c591755eb0a96e749e1de0e333b1a0c26a2c1d1a5c7dd6bbd565d2a7b1e`。resync合并冲突（language_conformance.rs内985/977双侧同位新增）两侧完整保留，resync后985 cover 2/2＋p977 2/2实测绿（p977首红=worktree缺预构建a2x-worker的环境归因，按错误信息自带的release维护构建6m01s后复放绿，非代码回归）。

**准确待交清单（A-required差额，截至2026-11-17）**：

1. **981 MIN-A-AGE余量**：EVENT-A代码已绿但批级phase收据在途；余B-SESSION-I/V/R[A]、B-LANGUAGE-V/R[A]（B-LANGUAGE-I已落主1738c2a1c）、B-PROOF-I/V/R[A]及T-01簿记；fold后交IF-981-EVENT-A＋MIN-A-AGE正式来源。
2. **1066/A**：IF-1066-A-CORRECTNESS＝A-DEN/A-ORACLE/A-NOWRITE/A-EVIDENCE＋NC-A1..4，经T-01/02/03/07/08/09/10/11 A子面→T-12[A]独立phase→T-13[A]正式fold；已获/goal授权开工（树已建、实现未始）。
3. **OS064/A**：MIN-A-OS＝真实OS启动/注册/激活/窗口会话链＋隔离候选只注册Hello与Counter Node见证＋真实输入/双实例隔离/关闭回收/错host协议拒绝；os-064组auto-os@fb1192c已V-OS-A-MIN绿（在途未fold，plan簿记仍drafting 0/97）。
4. **980余量L-45（NavCall A最小面）**：MIN-A-UI已交面不含NavCall；需980补最小真实面或T-A03重冻时按r23规则具名归波次。
5. **T-A03重冻裁定项（非交付缺口）**：S-20/S-21（media/terminal，候OS064/A）；S-25（c面A基础，982 route C已具名GAP-982-CONFIG-ACE→B）；B-02（A2C A面=已有事实的收据锚定）；B-04/B-06（Godot，用户后置非required）；C-01/E-01/02/03/05/F-01等的MIN-A见证分栏（980/A、983/A已落主，行级phase重排归T-A03重冻）。

**停止点**：上述1–3齐（4/5为裁定项）后T-A03方运行完整A并集→T-A04独立复审→T-A05落M2A-08。本轮到此为止，不预跑、不重复跑最终验收。

#### r26 增量（2026-11-17 第二轮：OS064/A接收＋981差额反馈）

**OS064/A 已正式落主，可接收**：auto-os master ff-only直线史 `2383d72→fb1192c（V-OS-A-MIN green）→06d3a0b（review P1修复）`＋簿记`484c432`＋goal收据`61237ba`。IF-OS064-A-MIN交接账：交付面=os-desktop `ace_min`候选见证链（注册表ace.hello.min/ace.counter.min、blake3冻结、feature缺省关=具名拒、真FFI open→真Iced窗→宿主关闭回收→证据JSON）；对985 MIN-A-OS合同逐项——真实启动/注册/激活/窗口链✅（desktop.ps1真窗372帧）、隔离候选只注册Hello/Counter✅、双实例2:1✅（headless与真窗同证）、关闭回收/late_dropped/reopen世代递增✅、错协议拒绝✅（错标签-5/错句柄-1/撤权denied→readmit恢复全零帧）、无旧VM✅；独立复核pass@06d3a0b；消费身份ui 403c936d/lang 7f431bd5f（均已核落主）。收据内具名非阻塞项：主检组masters追新致`--locked`锁漂移（fold#7后上游移动，非本批引入，下批重钉）、B波原义务/P2记B、plan-artifacts清理归整包merge。矩阵F-01行维持phase=B（MIN-A见证分栏归T-A03重冻），S-20/S-21维持裁定项。

**981 差额澄清与反馈（981自称“A阶段完工”与985接收合同的对账）**：981 review-68批级收据实际闭合的是 **event A（IF-981-EVENT-A）＋B-LANGUAGE-V[A]/B-SESSION-V[A]/B-RUST-V[A]的证据面**（绑定lang `24b4ba4a7`/`75015a49b`、ui `771be6a9`/`7de28330`，本轮核验全部已落主；其自身明示“不声称B-LANGUAGE-V/B-SESSION-V/B-RUST-V全项完成”）。对照985 §5 WAVES的MIN-A-AGE合同（“SOURCE、SESSION/LANGUAGE/PROOF的A部分…复用g01/Showcase建立最小真实UI＋一个2D/一个3D呈现及一次事件/更新/关闭，独立CPU值/可见ROI，错误编辑保旧、修复恢复”），**PROOF批A面未交付**——B-PROOF-I/V/R三任务未实施未勾、批级结论未提及。反馈清单（已留981 §10指针）：

1. **补PROOF批A面**：B-PROOF-I/V/R[A]——g01/Showcase最小真实UI＋2D＋3D呈现＋一次事件/更新/关闭，只AV，错误编辑保旧/修复恢复，独立CPU值/可见ROI与资源owner一致；不要求完整RUST/五view/PARITY。
2. **出MIN-A-AGE整体正式交接收据**：按985合同“MIN-A-AGE全required及IF-981-EVENT-A独立R/fold后交985/A”——PROOF批fold后与event A批联合出phase交接（source/Schema/SDK身份、独立oracle、producer→consumer、未闭行），不新增Plan。
3. **簿记同步**：master版981计划仍3/34、T-01未勾、已交A子面任务（B-LANGUAGE-I@review-62、SESSION-I@review-64、event A批@review-68）勾选未更新——985按原子行接收不以整包executing拒收，但勾选应如实避免下游重复核算。
4. **具名债务A/B归属确认**：`D-981-EVENT-A-REMAINDER`（含R68-4口径）列明A-required子集；`D-981-EXIT-CONTRACT`按985 §5“现有来源版本差额的A接收”核979旧“恰一出口”oracle失效；R59-5、R64-1..R64-10现值与A面归属（R64-3/D-981-SESSION-MULTI①=V-B-SESSION[B]已分清不阻A）。
5. **review-69收口**：R69-1修补已落主（lang cedf5331c、ui 2f42d661 pin），§9缺review-69 pass行，补齐后再出整体交接。

**更新后待交清单（收敛为两件）**：①1066/A（已授权在途）；②981 PROOF批A面＋MIN-A-AGE/IF-981-EVENT-A整体交接收据（上述反馈1/2/5）。两件齐→T-A03运行完整A并集。

**L-45裁定（2026-11-17，本轮用户问询后确认）**：NavCall归**B/W1起波次**，不做A小交付，980计划无需新增phase任务。理由具名（T-A03重冻按此落实）：①事实——NavCall为旧UI方言构造，ACE共同grammar（auto-source.at）现无nav规则，980/981计划全文零提及，三个A见证业务（MIN-A-UI Hello/Counter、MIN-A-AGE g01/Showcase最小面、MIN-A-OS见证）零导航依赖；②合同——r23 D02规则文本已将"完整Node/路由/五view/UI/Widgets"列入W1→W2→W3波次部分，行级"A最小"是EVENT-SPLIT条件性提升位（真实A业务用到才提升），无业务依赖即A-min面为空，非A-required打折；③成本——为凑行做无消费者交付违背MIN-A代表性验收设计，且NavCall语法进共同grammar会再次改grammar-id、扰动刚关闭的#6i冻结身份。完整NavCall语义按已冻结successor链归W1（980 B-AV＋981 LANGUAGE[B-W1]）→W2（1068）→W3（980 B-RUST）。**保险条款**：若981 PROOF批实际源（g01/Showcase）出现导航用法，按EVENT-SPLIT提升规则落到981所属LANGUAGE/PROOF A任务（已在待交件②内），仍不新增980任务。S-25/B-02/S-20/S-21及C-01/E-01/F-01分栏维持T-A03重冻裁定项不变。

#### r27 增量（2026-11-17 第三轮：1066/A验收通过＋981材料就绪盘点）

**1066/A（IF-1066-A-CORRECTNESS）正式接收，验收通过**：
- 链条完整性：实施记录（T-01/02/03/07/08/09/10/11 A子面，dec529355＋289e71b37，vm+corpus门1137/1137）→ T-12[A]独立phase复核pass（独立上下文代理，F-1066A-R1 verify_receipt空选择/R2 segments对账两低危修闭@2e606d28f，修复后门PASS run-0o5gxy7_）→ T-13[A] ff-only fold（rebase无冲突，range-diff三提交全等，master tip=7c131286d，收据942d57681）。
- 本轮复验（verify, don't trust）：①落主身份——a18ab5a88/ed33c0a3e/7c131286d/942d57681均master祖先；②零漂移——`git diff 2e606d28f HEAD -- crates/auto-lang/test/governance/ crates/auto-test-case/ scripts/`为空（与fold收据声明一致）；③落地检查复跑——`cargo test -p auto-lang --lib plan1066`＝**5 passed/1.57s**（含`plan1066_nc_a1_denominator_tamper_rejected`负控）。
- 交付内容对985合同四轴：A-DEN（冻结分母契约`plan1066_a_obligations.json`＋常驻逐键门，篡改/缺项/并键/空H缺席必红）✅；A-ORACLE（N0028/N0030双通道vm-result恢复＋单次运行双轴判定＋旧入口身份绑定）✅；A-NOWRITE（A执行路径零作者树写入，diag_scope默认丢弃；conformance_bootstrap退普通注册，33验证不减有gate excluded名单可证）✅；A-EVIDENCE（receipt合同schema＋verify_receipt拒签轴＋NC-A1..4常驻负控）✅。边界具名：engine=legacy-autovm金通道对照不冒ACE通过（985 adapter dispatch归985）；SD-01..05/SD-A-PHASE沉淀归B（合同内no-impact）；13任务整项不勾、树/分支保留予B。
- 985侧F14/F20（AUDIT-14/AUDIT-20）的1066 A片段行随本接收关闭其"1066A片段"列；矩阵无行翻转（1066为T-A03原子项层输入）。

**981材料就绪盘点（r26反馈对账，待正式接收轮）**：①PROOF批A面已落主——B-PROOF-[A]真实2D/3D源（393e0cc75/a181a1bb4，g03-3d具名B波边界b73a3fc70）；⑤review-69 pass收据已落（6bbbcf909，R69-1修闭c286fb8d0）；②tier-1 A面终验收据（61e52ea44，固定组gate_passed；最终证据plan981 207/207、ace-core 742/742、overlay 1393钉260070b10）；④R69-3 D-981-EXIT-CONTRACT owner改判981/LANGUAGE（关闭条件写明）、R59-5已证投影侧不可修（owner syntax-grammar）；③簿记§8仍3/35（981自声明"未声称execution_done"，勾选按整任务规则留待——985按原子行接收不受阻，簿记项记录不阻）。**待一轮985接收对账**：核PROOF批是否并入tier-1终验收据或有独立批收据、IF-981-EVENT-A交接包（auto-lang 01c90e269等）与PROOF的联合完整性。
- **R69-2替代口径（T-A03须裁定记录）**：981对UI-EVT-BIND的A口径为"带参/带载荷派发**一律不可执行的具名拒**（`event-params-unsupported`/`event-payload-unsupported`，fail-closed＋零效果）"，错arity/type、重复binder、未绑定名的**逐项**诊断归B-W1（D-981-EVENT-A-REMAINDER已具名）。与r24 §5 UI-EVT-BIND行"重复/错arity/未绑定名字拒绝"字面存在口径差：安全拒绝要求满足（具名拒＋零效果），差异仅在诊断粒度。T-A03按EVENT-SPLIT"提升所用最小子面"原则裁定：A-min实际业务（Counter unit链）不含带参路径→接受替代口径并记录映射理由，或如A业务后续引入带参则提升逐项诊断。

**更新后待交清单（仅剩一件）**：981接收对账（材料已齐，985侧动作，非981再交付）。对账通过后A-required外部输入全齐→T-A03重冻＋完整A并集→T-A04→T-A05（M2A-08）。

#### r28 增量（2026-11-17 第四轮：981接收对账通过——A外部输入全齐）

**981 MIN-A-AGE/IF-981-EVENT-A 整体交接接收，对账通过**：
- **整体交接收据**（e8d637bb9，stage: handoff，review-71 pass）：绑定 revision 本轮逐项核验全部落主——lang `0520c7f3b`（代码`6340934a1`/`ad7882cbe`）、event A `24b4ba4a7`/`75015a49b`、V[A] `0b98c143b`/`230e4abc6`；ui `ab6fff5c`（代码`658ad515`/`1eb27697`、SESSION-V `771be6a9`/`9032d69c`）；SDK base 681＋overlay 1395 文件钉 `0520c7f3b`。
- **对985 MIN-A-AGE合同逐项**：g01原三模块2D（位置/尺寸/可见性CPU值对齐原基线，变异即红）✅；g08/3d派生真实源view-3d对齐原scene ✅；UI+2D+3D组合包出口互不跨域＋1v3出口预算同 ✅；UI事件2:1实例状态 ✅；宿主腿 event→update→坏编辑具名拒保旧→修复恢复→stop冻结→restart回初值 ✅；event A面（具名/匿名unit、重复/未知/双集合拒、参数保留派发拒、payload声明合法派发拒、#6i身份）✅。AV场景观察格（f32不在av.owned/1）具名 `D-981-PROOF-AV-SCENE`→B-AV，A只在门面描述事实——接受为MIN-A边界而非缺口（"只AV"合同下CPU值/可见性即为独立oracle）。
- **本轮复验**：p981全集（ace-core conformance）**205/205 PASS**（较r25时201＋4）；**PROOF A批4/4判别绿**——`p981_proof_a_g01_real_2d_source_keeps_its_baseline_values`、`p981_proof_a_g08_derived_3d_source_keeps_its_baseline_values`、`p981_proof_a_bad_edit_keeps_old_and_recovers`、`p981_proof_a_mixed_ui_2d_3d_exits_and_budget`。
- **r26反馈①–⑤全闭**：①PROOF批A面（review-70 needs_fix→修补→review-71 pass）；②整体交接收据；③簿记同步（b04f74529：A子面台账/T-01勾选/债务口径更正）；④`D-981-EXIT-CONTRACT`已关（旧979"恰一出口"oracle失效，永久用例在`plan981_v_harness`/`plan981_proof_a`）、R59-5定证（owner syntax-grammar）、余债具名归波次；⑤review-69 pass（review-70/71续链）。
- **R69-2替代口径裁定（T-A03落实记录）**：接受。981对UI-EVT-BIND的A口径＝带参/带载荷派发**一律不可执行的具名拒**（`event-params-unsupported`/`event-payload-unsupported`，fail-closed＋零状态/UI效果）；A-min实际业务（Counter unit链、MIN-A组合包）无带参路径，满足r24合同"具名能力拒绝，零状态/UI效果"的安全要求；错arity/type、重复binder、未绑定名的**逐项**诊断归B-W1（`D-981-EVENT-A-REMAINDER`已具名）。映射理由＝EVENT-SPLIT"只提升所用最小子面"；A业务后续引入带参时按同规则提升。

**A-required外部输入清单关闭**：r25清单（8审计包/库族/980-A/971-A/977-PY/982-A/983-A/984-A/1024）＋r26（OS064/A、L-45裁定）＋r27（1066/A）＋r28（981 MIN-A-AGE/IF-981-EVENT-A）全部接收完毕。**985/A等待点结束**。剩余均为985自身工作：T-A03重冻（裁定项落实：M2-TARGET-SCOPE/WAVES分栏/S-25/B-02/S-20/S-21/R69-2映射）＋V-A-EXIT完整A并集实跑→T-A04独立复审→T-A05落M2A-08。属实施执行，待用户实施指令；本轮不启动。

#### r29 执行记录（2026-11-17，/goal 完成计划985/A后续工作——work轮：T-08[A]＋T-A03）

**授权**：用户 `/goal 完成计划985/A后续工作`（auto-plan-goal范式，work→review→有界修复→A批fold全链授权；B批不动）。worktree `D:/autostack/.wt/lang-985/auto-lang`@plan-985-dev（resync master 17674d957；A批工作提交 f93508199＋ecc282df1；树clean）。

**T-08[A]——MIX-07同一工程组合（新交付，AC-06/V-08/V-09）**：`experimental/ace/crates/abi/tests/cases/f0_t16_mix07.rs` 两例（原单例13.7s超≤10s门，按t15 owner先例拆分）：
- `t16_mix07_platform_config_rust_parse_and_cancel`（4.37s）：1043平台文件选择（suffix装配TargetOs::Windows选`.rs.at`实现，公共adapter=组合面）→配置文件真机读取＋钉版serde_json=1.0.151解析（use.rs公开API，真cargo单元机器码）→独立oracle（进程内另一份serde副本）等值；协作取消（预置flag→Cancelled）；V-08负控三具名（删实现文件=load.no-provider-impl红、坏配置文档=解析失败值-1非静默绿＋恢复、缺配置路径=-8）。
- `t16_mix07_sqlite_write_readback_and_cancel`（5.63s）：被证等值的配置值n流入use.c SQLite（钉版3.46.0融合库cl真码）：CREATE+INSERT(n,2n)+SELECT SUM=3n回读＋p967_close；取消→Cancelled→正常重跑同值（项目coherent）。
- 车道扩展：`open_assembled_rs_provider_session` 对带dep标注的装配源改经`build_cargo_provider`真依赖构建（dep-free仍走原路）——MIX-07组合的合法接口前提，非旁路。
- owned/读帧轴绑定已交`out_frame_1064`（AV OutBytes帧）证据；仓外安装/退役复核=B（986/987）。

**MIX-01..06 A腿聚合账（绑定owner正式证据，不重跑未变输入）**：MIX-01→960/962/963/964归档套件＋m2-cover-985 owned-relay（T-A02自有）；MIX-02→966＋978（t14精确失效，T-09审计）；MIX-03→968＋974＋983/A（PORT/PROOF真TCP），完整UI×HTTP×Web产品组合=B；MIX-04→961＋975＋976＋967六路径；MIX-05→981 PROOF-A组合包（r28接收），direct/RQ两视图=B W1+；MIX-06→984/A B-EMBED＋OS064/A MIN-A-OS，真实OS终验=B。

**T-A03重冻（矩阵/账本，V-TARGET-SCOPE/V-A-FREEZE落实）**：TSV裁定——L-45（r26裁定A-min条件位空→归W1）、S-20/S-21（产品宿主域successor本为980/984/OS064、A见证业务零消费→归波次，映射理由=产品面WAVES迁交；保险条款同L-45：A业务后续需要则提升）、S-25（route C：GAP-982-CONFIG-ACE→B）均phase A→B；B-02落landed@m2-stdlib-io.md（A2C"按原适用域不减"已交事实行：bstr_1064_v_codec_five_ends＋stdlib_asm_972/v08 C腿，增强仍B）；B-04/B-06→disposition user-deferred（M2-TARGET-SCOPE用户裁定）。账本版本化扩展：DISPOSITIONS+user-deferred、ALLOWED_USER_DEFERRED=[B-04,B-06]（无裁定来源的defer=红）、user-deferred必须pending（never-PASS/NA）；新负控`p985_target_scope_deferred_rows_are_red`（必需行B-05被defer=红、deferred行虚报evidence=红）。obligation_ledger **8/8**；新TSV SHA256=`89eebafa7489a4d7cb795e5fb553d3c91242061222671491070de36053d00d49`。C-01/E-01..05/F-01的MIN-A见证分栏按WAVES以r25–r28接收记录为准（行级B-pending不提前勾）。

**V-A-EXIT并集实跑（worktree@ecc282df1，Windows x64单实例）**：
| 腿 | 命令 | 结果 |
|---|---|---|
| 账本 | `cargo test -p auto-lang-core --lib obligation_ledger` | **8/8**（0.07s） |
| 981接收面 | nextest ace-core conformance `-E "test(p981_)"` | **205/205**（1.2s） |
| FFI/LIB/MIX | nextest ace-abi conformance `-E "test(/f0_/)"` | **36/36**（13.9s，含MIX-07两例） |
| 985自有cover | nextest ace-conformance `-E "test(ace_m2_p985_cover)"` | **2/2**（53.3s） |
| 固定组 | `cargo t auto-test-case ace-source ace-stdlib ace-adapters` | **94/95**：1红=`source_parent_oracle_failure_rejudges_the_same_compiler_root_without_caching_pass`（auto-lang-core abi协议控制）——**master主检出复放同红（120s超时形态）=基线既有红**，与本批触面（ace-abi rust_provider/装配道、TSV、账本）零交集，归批量回归triage不阻A批 |

**M2A-01..07门级现值（A出口判定）**：M2A-01分母/矩阵✅（959＋162行重冻＋8/8）；M2A-02语言✅（962-970/972归档＋976/977-PY/979/980-A/981 MIN-A-AGE接收）；M2A-03标准库✅（947/948/972＋973/974/975→1064/1065归档，S-07..S-18 landed）；M2A-04 FFI/装配✅（966/967/968＋File(1065)＋978＋T-09审计＋MIX-07本轮实跑）；M2A-05 RG-A01..06✅（971/A、RG-A01/02/06归档＋982-A/983-A、RG-A03=MIN-A-UI、RG-A04=MIN-A-AGE、RG-A05=MIN-A-OS）；M2A-06 T-A01..A03✅（本轮闭合）；M2A-07✅（966/970/978归档）。**A-required无unknown/NOT_RUN；B承接行具名pending（矩阵B行＋user-deferred两行＋债务表）**。

**结论**：A批work收口（T-01/T-A01/T-A02/T-09/T-08[A]/T-A03全闭环），移交**T-A04独立复审**（review_scope=phase A批；绑定revision r29/commit ecc282df1）。

### M2-TARGET-SCOPE-20261116：本包当前边界

本包按新用户范围重校目标义务矩阵/adapter/阶段门：Godot为用户后置非required，TS/JS按生产消费闭包required。r16已交矩阵/旧check只作历史基线，T-A03须重冻版本和差异反控后才验收；保4/14真实历史进度，不据范围缩减签M2A-08。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

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
| SD-CFG-HANDOFF | modify（拟，原owner） | `docs/specs/ace-migration/m2-certification.md` | 原候选接收→实际配置新AV来源/宿主与guest边界、错误恢复/原平台，真实交付后才写已支持 | 982生产一次，本包只核实际消费者/安装退役阶段；A收据保留 | AC-CFG-HANDOFF、原AC与最终review |
| SD-01 | add | `docs/specs/ace-migration/m2-certification.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/auto-bindgen/project.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07 |
| SD-04 | modify | `docs/specs/stdlib/project.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-certification.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE/AC-A-MAP/AC-A-COVER/AC-A-EXIT/AC-A-REVIEW/AC-A-LAND |
| SD-NATIVE | add | `docs/specs/ace-migration/m2-certification.md`（沿原owner，仅真实交付后沉淀） | 默认实现选择未明→自有Auto/OS默认与显式FFI分账、长度字符串/目标ABI及来源验证 | LIB-M2-07/08 | AC-NATIVE |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-certification.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-certification.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |
| SD-IO-HANDOFF | modify | `docs/specs/ace-migration/m2-certification.md`与`replacement-scope-and-evidence.md`（实际交付后） | 原973整包占位→按973有限交付、1064 MODULE/文本/库面/输出帧、975B1独立切片、1065平台适用AC真身、1063 Linux移交分别绑定原义务/真实来源；977/A不反等B | 用户已裁定延期承接，不丢M2原库/覆盖义务或制造循环 | AC-A-MAP/AC-A-EXIT/AC-NATIVE/AC-COVERAGE、V-IO-HANDOFF；T-A03/A04/A05 |

### A-FREEZE-20261116-r23：1024逐发现验收分母（本轮当前）

此处冻结的是**规划验收合同**，不是实际运行矩阵或PASS分子。162父义务、1024的35个发现、1066各组/角色是关联层次，不能直接相加成“总测试数”。A实际原子义务键为`parent_id＋case/check/consumer-channel＋backend/provider＋platform/原condition＋source/oracle/producer`；参数/多源/多通道不得只按case名称合并。每个A原子项必须有真实结果、独立oracle、已审正式来源；B/后续版行保持owner/关闭条件、标pending或用户明确not-required，不能算PASS或无理由NA。

分类依据是当前A承诺/实际消费者/可信边界，不按P1/P2机械决定。**8个A正确性包：1029、1031、1032、1033、1034、1035、1036、1037，共20个发现；6个A约束/证据行由985核；2个已归档修复复用；7个发现交B历史修复/研究。** 静态候选可用真实充分反证关闭，无需为未证实问题强改代码；未执行/仅组总绿/未明确owner不能视为关闭。

1024仍reviewed/4-of-5，原只读审计结论不重开。T-05可先交其审计摘要/正式来源而不等待全部修复；各后继可依已复审冻结材料准备，A出口必须有可取的正式审计快照/来源与本表各required修复/反证D。归档1024不等于修复这些发现。1027/1028 archived只复用已验范围，不因旧发现表“未修”重开归档。

| finding | A/B裁定 | 执行owner | 原Case/check | 承接任务 | 关闭条件（均非本轮执行结果） |
|---|---|---|---|---|---|
| F01 | 已交/A复用 | 1027 | UTF-01/02 | 原T-02/03→T-04/05 | 归档修复收据；A核source/工具漂移，UTF编辑边界/合法邻例，不能据历史表“未修”重做 |
| F02 | 已交/A复用 | 1028 | OBS-01/02 | 原T-02/03→T-04/05 | 归档嵌套capture/Err/unwind恢复；核身份变化后受影响consumer，不用新观察器掩盖旧事实 |
| F03 | A约束/B结构研究 | 985＋1038 | AUDIT-03 | 985 T-A03/A04；1038 REUSE-02/03原T-02/03 | A核已交969错误/原span/拒绝不被字符串耦合扭曲；结构化/抽取候选B，不重开969 |
| F04 | B研究 | 1038 | REUSE-01/D04 | T-01..06 | 52行机制的等价/收益与go/no-go决定，不为A先做抽取 |
| F05 | A约束/B结构研究 | 985＋1038 | AUDIT-05 | 985 T-A03/A04；1038 REUSE-02/03原T-02/03 | A核970现session接受/取消/恢复合同；分散职责不等于可删；B再决定重构 |
| F06 | B历史修复 | 1030 | AVTX-01/02 | T-01..06；985 T-A03边界核 | 当前仅864历史测试consumer；旧三例保留，晚失败/同批重试按1030关闭；若A真实调用它则A前提升 |
| F07 | B研究 | 1038 | REUSE-02/D07 | T-01..06 | parse/resolve/emit/bind/reanchor成本真实或unknown，go/no-go，不凭转换次数强行重写 |
| F08 | A约束/B结构研究 | 985＋1038 | AUDIT-08 | 985 T-A03/A04；1038 REUSE-02/03 | A核profile权威、明确拒绝和无旧fallback，未证实成功fallback不冒bug已修；B仅结构候选 |
| F09 | B研究 | 1038 | REUSE-02/D09 | T-01..06 | 正常依赖/宿主/链接边界与收益决定，不以依赖图标题强制A改架构 |
| F10 | A正确性 | 1034 | IPC-02 | T-01..06 | reader IO/EOF/parse分类与合法邻例；父异常不当语言拒绝或PASS |
| F11 | B研究 | 1038 | REUSE-02/D11 | T-01..06 | no_std ABI/hooks/错误/预算独立保留，不能因同源并库 |
| F12 | A保留约束 | 985/978已交 | AUDIT-12 | 985 T-A03/A04 | 保现provider/executor/cap/ABI与资源约束；不是缺陷，不计新修复数 |
| F13 | B研究 | 1038 | REUSE-03/D13 | T-01..06 | 模块热点的有界私有边界/go-no-go，不以行数触发跨crate拆分 |
| F14 | A证据 | 985＋1066A片段 | AUDIT-14 | T-A03/A04；1066 A-DEN | 旧case/check/source/oracle/条件逐键守恒，真实运行覆盖与静态账分开，漏项/假PASS/错身份负控 |
| F15 | B研究 | 1038 | REUSE-03/D15 | T-01..06 | 陈旧注释更正的有界方案，非已证实反向依赖违规 |
| F16 | A正确性 | 1029 | ADM-01 | T-01..06 | 预留前尺寸/checked预算/FromWire；坏count不做OOM实验，合法边界与真实consumer |
| F17 | A正确性 | 1029 | ADM-05 | T-01..06 | 多lease/最后Drop立即删零键，树/物理pin保持各owner，不能混称全部树泄漏 |
| F18 | A正确性 | 1034 | IPC-01 | T-01..06 | 同一次选中源快照与SHA，运行/manifest/实际文件身份匹配；不假定父已误PASS |
| F19 | A正确性 | 1034 | IPC-02/r4限额 | T-01..06 | 保r4 FromFile/inline请求/响应/actual分档、合法上限/超限拒与独立ERROR，不误截断 |
| F20 | A证据 | 985＋1066A片段 | AUDIT-20 | T-A03/A04；1066 A-EVIDENCE | 固定职责/选择数/执行数/原条件排除实报，G1002不等全迁移，不弱化原oracle |
| F21 | A正确性 | 1029 | ADM-02 | T-01..06 | Token/Rule/Pratt等对应命名空间校验，受控拒绝/合法递归近邻；不拿从未运行panic当证据 |
| F22 | A正确性 | 1031 | NEG-01/02 | T-01..06 | commit核正负依赖，旧current/target不变，重新prepare后成功，body-only合法控制 |
| F23 | A正确性 | 1029 | ADM-03 | T-01..06 | 现IN935P/v4/512KiB先尺寸、tag/EOF规则，旧版MISS不授信；P2也属A恢复准入 |
| F24 | A正确性 | 1032 | NTX-01/02 | T-01..06 | typed真实mn-worker消费者的批晚失败保接受态、旧机器结果与同批修复重试；非AC桌面延期 |
| F25 | A正确性 | 1033 | AXTX-01/02 | T-01..06 | 非图形A2R/SDK共享resident/SHA/anchor/root单接受点，restart真读保旧，副产物真实记录 |
| F26 | A正确性 | 1037 | CT-02 | T-01..06 | 真实AV已到达后发布失败仍av_reached=true，execute/publish/hit计数及重试闭环，不冒语言PASS |
| F27 | A正确性 | 1034 | IPC-03/r4 Py | T-01..06 | Rust/C/Py父交换/有界输出/单次清理与实际失败分态，不用父exit0/超时冒成功 |
| F28 | A正确性 | 1036 | LOOP-01 | T-01..06 | 真实有限continue仍执行本循环推进且值正确，break/return/嵌套控制 |
| F29 | A正确性 | 1036 | LOOP-02 | T-01..06 | 闭区间MAX终迭代不合成越域加一，原位宽与预算义务保持 |
| F30 | A正确性/证据 | 1035 | RES-03 | T-01..06 | 保正常例，再测聚合真实中途失败/旧dst/ledger/N+1；注入Infra与真Resource不混淆 |
| F31 | A正确性 | 1029 | ADM-04 | T-01..06 | paged声明页/节点/跨度在消费前checked准入，合法按需读取/恢复，不改全树eager |
| F32 | A正确性 | 1037 | CT-01 | T-01..06 | Named/Pos/MovePos的CT发现→替换→共享lower→实际值3，无残留hash且拒缺cap |
| F33 | A正确性 | 1035 | RES-02 | T-01..06 | 真source Named/Pos投影合法无panic，参数前提/越界诊断与合法邻例 |
| F34 | A正确性 | 1035 | RES-01/r4 DEC-RES | T-01..06 | List-only/嵌套资源S闭包与Layout/MIR一致，D/R按各域事实，不能强求owned布尔都相等 |
| F35 | A正确性 | 1036 | LOOP-03 | T-01..06 | continue回边poll/WU/取消及cleanup真实观察，值正确不能代预算正确 |

1030的B边界由现仓调用扫描支持：CollectionsSourceSession只在三个历史测试消费、生产crate未找到调用；A需重核实际导入/recipe闭包，若进入A消费链则先提升相应缺口。1038只研究，B的no-go/保留也可关闭，不能在A验收中先做未经证实的大重构。F03/05/08/F12的A约束若实际红仍阻A，不能拿B研究登记抵消。

G1002只约束新作者格式/相关接线，不是1035等原入口修复的前置；1002归档不代表1066全量迁移已做。Linux/WSL与正式performance条件照原裁定，普通正确性缺口不许挪perf，现有A2C/A2PY支持事实与AV三provider义务保持；GUI AV→AC→A2R波次不取消非GUI语言/库/FFI正确性。

### 现有来源版本差额的A接收（不计入35审计发现）

并发新增D-981-EXIT-CONTRACT记录：981当前不同键view出口可并存、同键重复仍拒，而979旧SOURCE/V-11/detail来源可能仍按“恰一出口/混合必拒”验。A必须按当前MIN-A输入/Schema与source/hash核真实oracle，不能拿失效旧反控充当前通过，也不能要求合法新源满足过时拒绝规则。985 T-A03/A04负责拒绝错版收据，实际源/契约盘点和最小回验由**现有981 T-01及LANGUAGE的A最小子面**承接，沿原独立R/fold交新鲜接口；不重开已归档979，不新增计划或复制parser。本轮只登记未关闭的身份/契约检查，不声称已复核产品。

### 162父义务的重冻规则与逐行索引（原矩阵执行更新归T-A03）

| 规则 | A关闭面 / B承接面（原独立expected/真实owner不变） |
|---|---|
| D01 普通语言 | 普通Auto完整原AV/A2R支持及AC真实基础A；其余具名AC差额B。Named/CT/循环/资源等20审计发现按上表A关闭，不能以图形W2/W3为由后延。 |
| D02 图形领域 | A仅MIN-A-UI/AGE的Schema/多模块/AV真实见证；完整Node/路由/五view/UI/Widgets逐项W1 AV→W2 AC→W3 A2R。 |
| D03 FFI | A通用import/ABI/资源/实际库/所需callback-future/候选基础；全产品/安装退役及具名AC余量B。C互操作所需能力仍A，不自动变成完整A2C语言硬门。 |
| D04 新库 | A既定公共库/自有实现/字符串/装配/必要真实后端与1064正式来源；安装/全产品/退役及允许AC余量B，旧S/FFI/LIB不漏。 |
| D05 工具协议 | A取971-PROOF/969-970/982-CONFIG-PROOF等正式协议与最小真实消费；完整工具/编辑器/命令/目标迁移B，不要求整包归档。 |
| D06 目标 | PY与生产TS/JS provider保既定A；普通非GUIA2R/已有FFI/库腿保A，完整图形A2R W3。A2C完整对齐增强，保已有事实与原required；Godot/GDScript/TSCN/SCENE Godot目标用户移出M2，JET/ARK M3；其余只按真实生产/规范支持清单，不复活已撤目标。 |
| D07 UI产品 | A仅C-01等MIN-A见证和必要字段/协议；完整Iced/Widgets三波，WEB/TAURI按现役模式B并行，JET/ARK M3；完整模式/安装不得拿Hello/AXR0代替。 |
| D08 Man产品 | A=CONFIG/PROOF，全部生成/命令/模板/watch/导出等B；入口含测试协议者按D05，阶段不按父行单一字面硬判。 |
| D09 Web产品 | A=PORT/PROOF规定真实TCP/隔离DB/AV与非GUI生成服务，其他全API/四仓消费者B；完整产品不反成为A proof前提。 |
| D10 生态/OS | A仅OS064 MIN-A-OS、984 EMBED及明确协议/守恒；三UI波后完整桌面/App/旧数据/双Host/Node根与其余生态B。 |
| D11 安装退役 | 978/EMBED必要仓外候选基础A；全安装/默认/升级/旧链不可达/重建终验B由986/987及实际owner，当前机器/开发树不拿来冒候选。 |

下表每个原row_id只出现一次，取材原obligation和successor不篡改；同父行的A/B子check由上述规则与MIN-A/原计划Case精确区分。这里只是本Plan的规范性分类索引，不另建源/expected作者库；原worktree表仍由T-A03落实并走正式来源。

| 原row_id | 原义务（取材） | 本轮分类 | 规则 | 原/实际承接指针 |
|---|---|---|---|---|
| L-01 | 数值字面量int族 | A普通语言/具名AC余量B | D01 | 961 |
| L-02 | float/double字面量 | A普通语言/具名AC余量B | D01 | 961 |
| L-03 | byte/char/CStr字面量 | A普通语言/具名AC余量B | D01 | 961 |
| L-04 | i128/f32后缀(无此形守卫) | A普通语言/具名AC余量B | D01 | 959 |
| L-05 | 位运算/移位/符号扩展 | A普通语言/具名AC余量B | D01 | 961 |
| L-06 | .view/.mut/.move | A普通语言/具名AC余量B | D01 | 962 |
| L-07 | Hold | A普通语言/具名AC余量B | D01 | 962 |
| L-08 | Box/Arc(v) | A普通语言/具名AC余量B | D01 | 962/964 |
| L-09 | Take(Move别名) | A普通语言/具名AC余量B | D01 | 962 |
| L-10 | type声明/字段/方法/mut写回 | A普通语言/具名AC余量B | D01 | 962 |
| L-11 | enum三形态 | A普通语言/具名AC余量B | D01 | 962 |
| L-12 | union | A普通语言/具名AC余量B | D01 | 962 |
| L-13 | tag | A普通语言/具名AC余量B | D01 | 962 |
| L-14 | grid | A普通语言/具名AC余量B | D01 | 962 |
| L-15 | tuple+TupleDestruct | A普通语言/具名AC余量B | D01 | 962 |
| L-16 | array/List/map字面量(set无此形守卫) | A普通语言/具名AC余量B | D01 | 962 |
| L-17 | Option/Result构造与模式 | A普通语言/具名AC余量B | D01 | 962 |
| L-18 | is语句/表达式+模式 | A普通语言/具名AC余量B | D01 | 962/965 |
| L-19 | range .. / ..= | A普通语言/具名AC余量B | D01 | 962 |
| L-20 | object动态成员/反射 | A普通语言/具名AC余量B | D01 | 964 |
| L-21 | nil/null/?? | A普通语言/具名AC余量B | D01 | 964/965 |
| L-22 | closure x=>e | A普通语言/具名AC余量B | D01 | 964 |
| L-23 | lambda桩(AST-only旧bug) | A普通语言/具名AC余量B | D01 | 964 |
| L-24 | 默认参数 | A普通语言/具名AC余量B | D01 | 963 |
| L-25 | 命名实参 | A普通语言/具名AC余量B | D01 | 963 |
| L-26 | if/else/for/while/loop/jump/return | A普通语言/具名AC余量B | D01 | 965 |
| L-27 | try/catch/finally | A普通语言/具名AC余量B | D01 | 965 |
| L-28 | .?错误传播 | A普通语言/具名AC余量B | D01 | 965 |
| L-29 | async块/.await/.go | A普通语言/具名AC余量B | D01 | 968 |
| L-30 | yield | A普通语言/具名AC余量B | D01 | 965 |
| L-31 | reply(ask/reply) | A普通语言/具名AC余量B | D01 | 968 |
| L-32 | #if/#for/#is/#{}语句级 | A普通语言/具名AC余量B | D01 | 966 |
| L-33 | #{}表达式级 | A普通语言/具名AC余量B | D01 | 966 |
| L-34 | 宏#name() | A普通语言/具名AC余量B | D01 | 966 |
| L-35 | metadata/#[attr] | A普通语言/具名AC余量B | D01 | 966/971/980/982 |
| L-36 | 平台条件(#if/#is+sys事实域) | A普通语言/具名AC余量B | D01 | 966/978 |
| L-37 | use四形/dep | A普通语言/具名AC余量B | D01 | 960/967 |
| L-38 | alias/type alias | A普通语言/具名AC余量B | D01 | 963/960 |
| L-39 | spec(trait声明) | A普通语言/具名AC余量B | D01 | 963 |
| L-40 | ext/impl | A普通语言/具名AC余量B | D01 | 963 |
| L-41 | 全局store let/var/const/shared/CVar | A普通语言/具名AC余量B | D01 | 960 |
| L-42 | 泛型type/const参数/turbofish | A普通语言/具名AC余量B | D01 | 963 |
| L-43 | TaskDef/on=>事件 | A普通语言/具名AC余量B | D01 | 968 |
| L-44 | Node语句/表达式+OnEvents/SceneDecl | A最小/B三波 | D02 | 979/977/980；980/981/1068三波 |
| L-45 | NavCall路由调用 | A最小/B三波 | D02 | 980；980/981/1068三波 |
| L-46 | spread/set字面量/when-match/global(无此形守卫) | A普通语言/具名AC余量B | D01 | 959 |
| L-47 | 调用语义:动态调用/方法分派/管道 | A普通语言/具名AC余量B | D01 | 963 |
| L-48 | 索引/place赋值/复合赋值 | A普通语言/具名AC余量B | D01 | 962 |
| L-49 | cast as / 转换 to | A普通语言/具名AC余量B | D01 | 961 |
| S-01 | math/int域 | A基础/B产品余量 | D04 | 945/972 |
| S-02 | text/str/char/fmt域 | A基础/B产品余量 | D04 | 945/972 |
| S-03 | bytes域 | A基础/B产品余量 | D04 | 972 |
| S-04 | collections域 | A基础/B产品余量 | D04 | 946/972 |
| S-05 | parse/format/encoding域 | A基础/B产品余量 | D04 | 972 |
| S-06 | hash域 | A基础/B产品余量 | D04 | 972 |
| S-07 | json域 | A基础/B产品余量 | D04 | 948/975 |
| S-08 | fs/path/file域 | A基础/B产品余量 | D04 | 947/973 |
| S-09 | env域 | A基础/B产品余量 | D04 | 973 |
| S-10 | process域 | A基础/B产品余量 | D04 | 973 |
| S-11 | time/datetime/duration域 | A基础/B产品余量 | D04 | 973 |
| S-12 | rand/random域 | A基础/B产品余量 | D04 | 973 |
| S-13 | zip域 | A基础/B产品余量 | D04 | 973 |
| S-14 | console/io/log域 | A基础/B产品余量 | D04 | 947/973 |
| S-15 | storage域 | A基础/B产品余量 | D04 | 973 |
| S-16 | net/http/sse/url域 | A基础/B产品余量 | D04 | 948/974/983 |
| S-17 | sqlite域 | A基础/B产品余量 | D04 | 975 |
| S-18 | redis域 | A基础/B产品余量 | D04 | 975 |
| S-19 | task/async/sched(timer)域 | A基础/B产品余量 | D04 | 947/968/973/974 |
| S-20 | media(image)域 | A基础/B产品余量 | D04 | 980/OS064 |
| S-21 | terminal域 | A基础/B产品余量 | D04 | 984/OS064 |
| S-22 | clipboard/dialog/dnd/i18n/frame宿主面 | A基础/B产品余量 | D04 | 967/980 |
| S-23 | ui.at/widget/aura域 | A最小/B三波 | D02 | 979/980；980/981/1068三波 |
| S-24 | test.at/test_mod域 | A基础/B产品余量 | D05 | 971 |
| S-25 | c面(stdio.c/stdlib.c装配) | A基础/B产品余量 | D03 | 982/987 |
| FFI-M2-01 | 公共use.c/use.rs绑定前门 | A基础/B产品安装及AC余量 | D03 | 967 |
| FFI-M2-02 | C importer(预处理/布局/callback) | A基础/B产品安装及AC余量 | D03 | 967 |
| FFI-M2-03 | Rust API元数据→C兼容shim | A基础/B产品安装及AC余量 | D03 | 967 |
| FFI-M2-04 | 四端消费同一签名/资源/效果并实调 | A基础/B产品安装及AC余量 | D03 | 967 |
| FFI-M2-05 | 双向import/export/allocator/销毁/重入 | A基础/B产品安装及AC余量 | D03 | 967 |
| FFI-M2-06 | 异步注册/完成/取消端口 | A基础/B产品安装及AC余量 | D03 | 967/968 |
| FFI-M2-07 | 可分发binding/shim/provider闭包与缓存 | A基础/B产品安装及AC余量 | D03 | 967/978 |
| FFI-M2-08 | 用户级真实库工程与独立ABI oracle | A基础/B产品安装及AC余量 | D03 | 967/985-987 |
| LIB-M2-01 | 公共API/稳定操作ID/core-alloc-host-domain分层 | A基础/B产品安装及AC余量 | D04 | 947/972 |
| LIB-M2-02 | 组织/装配/加载owner及版本 | A基础/B产品安装及AC余量 | D04 | 947/978 |
| LIB-M2-03 | #if/#is目标平台选择进依赖图 | A基础/B产品安装及AC余量 | D04 | 947/966/973/1043 |
| LIB-M2-04 | 基础/集合/文本算法四端一致 | A基础/B产品安装及AC余量 | D04 | 945/946/972 |
| LIB-M2-05 | 文件/系统/宿主库统一语义平台ABI | A基础/B产品安装及AC余量 | D04 | 973 |
| LIB-M2-06 | 兼容分母/安装/增量/恢复/退役含新库 | A基础/B产品安装及AC余量 | D04 | 948/978/985-987 |
| LIB-M2-07 | 默认自有实现(无隐式libc委托) | A基础/B产品安装及AC余量 | D04 | 947/972/973/978/985 |
| LIB-M2-08 | 字符串/ABI边界(长度/编码/owner) | A基础/B产品安装及AC余量 | D04 | 967/972/973/985 |
| A-01 | 执行/config求值族入口 | A协议/最小消费＋B产品 | D05 | 969/982 |
| A-02 | 脚本执行入口 | A协议/最小消费＋B产品 | D05 | 969 |
| A-03 | 求值入口 | A协议/最小消费＋B产品 | D05 | 969 |
| A-04 | 调试族入口 | A协议/最小消费＋B产品 | D05 | 971 |
| A-05 | 转译族入口 | A协议/最小消费＋B产品 | D05 | 976/977/982 |
| A-06 | REPL/daemon/MCP入口 | A协议/最小消费＋B产品 | D05 | 970 |
| A-07 | #[test]/shim入口 | A协议/最小消费＋B产品 | D05 | 971 |
| A-08 | 仓内二进制(playground/auto-case) | A协议/最小消费＋B产品 | D05 | 969/971/987 |
| A-09 | 引擎选择分派 | A协议/最小消费＋B产品 | D05 | 969/987 |
| A-10 | pub use vm整包直连 | A协议/最小消费＋B产品 | D05 | 967/980/983 |
| A-11 | AutoInterpreter | B产品 | D05 | 982 |
| A-12 | a2r_std中立保留 | B产品 | D05 | 959 |
| B-01 | A2R Rust目标 | A非GUIA2R/B图形W3 | D06 | 969/980/982/983 |
| B-02 | A2C C目标 | A已有required/完整A2C增强 | D06 | 982 |
| B-03 | A2PY Python目标 | A-PY/产品B | D06 | 977/1042 |
| B-04 | A2GD Godot目标 | 用户移出M2，非PASS | D06 | 977；按用户当前范围裁定 |
| B-05 | A2TS TypeScript目标 | A生产TS/JS provider/产品B | D06 | 976 |
| B-06 | SCENE目标 | Godot场景目标移出M2，非PASS | D06 | 977；按用户当前范围裁定 |
| B-07 | 其余相邻目标 | 按当前生产支持清单A/B | D06 | 977 |
| B-08 | r2a反向 | B-REVERSE | D06 | 977 |
| B-09 | s2s同构 | B-REVERSE | D06 | 977 |
| B-10 | escape未接线(退役候选) | B退役 | D06 | 987 |
| B-11 | 相邻发射器 | A库基础/B相邻发射器退役 | D06 | 972-975/987 |
| C-01 | auto-ui vm/Iced | A最小/B三波 | D07 | 980/NATIVE；980/981/1068三波 |
| C-02 | vue双档 | B完整产品/模式/安装 | D07 | 980/WEB+RUST |
| C-03 | tauri | B完整产品/模式/安装 | D07 | 980/TAURI |
| C-04 | rust静态 | B完整产品/模式/安装 | D07 | 980/RUST |
| C-05 | jet/ark | M3，非M2缺证/PASS | D07 | 980/JET/ARK；按用户当前范围裁定 |
| C-06 | vscode模式 | A字段/协议基础＋B完整产品 | D07 | 971/986 |
| C-07 | VM内部直连面 | A字段/协议基础＋B完整产品 | D07 | 967/980；980/981/1068三波 |
| C-08 | 冻结SDK | B完整产品/模式/安装 | D07 | 981/987 |
| C-09 | aura | A字段/协议基础＋B完整产品 | D07 | 980/DIALECT；980/981/1068三波 |
| C-10 | auto-ui测试 | B完整产品/模式/安装 | D07 | 985/980 |
| C-11 | auto-ui安装 | B完整产品/模式/安装 | D07 | 986 |
| D-01 | new/init/build/gen命令族 | A配置/协议最小＋B命令 | D08 | 982/CONFIG |
| D-02 | ui/bp/wizard/add命令族 | B完整产品 | D08 | 982 |
| D-03 | fetch/deps/device/export/info命令族 | B完整产品 | D08 | 982 |
| D-04 | run/service命令 | B完整产品 | D08 | 982/EXEC |
| D-05 | test/test:ui命令 | A配置/协议最小＋B命令 | D05 | 982/971 |
| D-06 | watch命令 | B完整产品 | D08 | 982/DEVSDK |
| D-07 | old-repl/serve/req/mcp命令 | A配置/协议最小＋B命令 | D05 | 970/971/967 |
| D-08 | trans/c/rust/python等转译命令 | B完整产品 | D08 | 982 |
| D-09 | debug/cffi/stdlib/docs等命令 | B完整产品 | D08 | 971/967/973/975 |
| E-01 | auto-web端口/嵌入面 | A-PORT/PROOF所用真实路径＋B完整 | D09 | 983/PORT |
| E-02 | AV服务线程模型 | A-PORT/PROOF所用真实路径＋B完整 | D09 | 983/AVSERVICE |
| E-03 | 生成Rust轨 | A-PORT/PROOF所用真实路径＋B完整 | D09 | 983/GENERATED |
| E-04 | 四仓TCP闭环 | B完整消费者 | D09 | 983/CONSUMERS |
| E-05 | serve/dispatch/parked owner | A-PORT/PROOF所用真实路径＋B完整 | D09 | 983 |
| E-06 | api_gen嵌入消费 | B完整消费者 | D09 | 983 |
| F-01 | OS host/desktop | A最小OS/协议/守恒＋B完整 | D10 | OS064/HOST；OS064 MIN-A/完整B |
| F-02 | OS Apps载体 | B完整生态/OS | D10 | OS064 |
| F-03 | jade-garden | B完整生态/OS | D10 | OS064/983 |
| F-04 | auto-musk | B完整生态/OS | D10 | OS064/A02 |
| F-05 | auto-edit | B完整生态/OS | D10 | 984/EDIT |
| F-06 | auto-gen | B完整生态/OS | D10 | 982/TEMPLATE |
| F-07 | auto-vscode | A最小OS/协议/守恒＋B完整 | D05 | 971/986 |
| F-08 | launcher | B完整生态/OS | D10 | OS064/A08 |
| F-09 | coder/ai | B完整生态/OS | D10 | 984 |
| F-10 | book harness(11 .at含#[test]) | A最小OS/协议/守恒＋B完整 | D05 | 984/971 |
| F-11 | ag-examples外部engine消费 | B完整生态/OS | D10 | 981 |
| F-12 | 安装脚本 | B完整生态/OS | D10 | 986 |
| F-13 | auto-shell(ash) | B完整生态/OS | D10 | 984/SHELL |
| F-14 | 20个auto-*应用仓+config | B完整生态/OS | D10 | OS064/984 |
| F-15 | 零引擎引用账(守卫) | A最小OS/协议/守恒＋B完整 | D10 | 959 |
| K-01 | man-install.py | B安装/退役 | D11 | 986 |
| K-02 | autographics冻结SDK×2 | B安装/退役 | D11 | 981/987 |
| K-03 | notes vendor副本 | B安装/退役 | D11 | OS064 |
| K-04 | web-install.py | B安装/退役 | D11 | 983/986 |
| K-05 | musk install.sh | B安装/退役 | D11 | OS064 |
| K-06 | C:/Users/zhaop/.auto现存安装 | B安装/退役 | D11 | 986/987 |
| K-07 | 本机引擎二进制 | B安装/退役 | D11 | OS064/986 |
| K-08 | 启动/脚本硬编码 | B安装/退役 | D11 | 986 |

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

### 当前范围覆盖与旧矩阵迁移（r17，原T-A03承接）

归档959和T-A01已交`autolang.ab-obligations/1`的162行及旧负控绿保留历史身份，不改写其旧结果。**原未完成T-A03在出口前**沿现矩阵/typed adapter版本化加入`M2-TARGET-SCOPE-20261116`：每个原目标腿记录source/check ID、phase、required或`user-deferred/not-required`、用户裁定来源及未来owner/去向；明确区分“当前完成状态”和“本版是否要求”。Godot排除按target/consumer腿，不可整删含通用语言语义的共享行；保原ID/原件链接，旧字段无法表达时在既有schema内版本化扩展，不另建认证系统。

977/GD、977/SCENE和982/GODOT不再进入M2A-01..07或B发行required缺口；缺Godot环境不再是M2出口阻塞。976生产TS/JS与980/982/983/984实际消费仍required；闭包内所有合法非Demo表达式/函数/模块同样计入。独立通用TS CLI额外面仅在确认真实产品依赖时纳入，不能据旧类职责自动计满，也不能把实际使用项后置。

T-A03须重冻当前分类/必需闭包并运行V-TARGET-SCOPE正反控，T-A04/A05核当前revision与正式来源；B原T-04核发行目标一致。旧T-A01/T-A02勾选和4/14进度保留为历史交付，不证明新范围检查完成，不新勾任务、不签M2A-08。其他required能力/RG/新库/四端/正式门和Linux1063移交合同仍按原规则；本轮用户范围调整不能用来关闭它们。

### 973有限交付与1064/1065的认证接收（当前优先）

| 原义务/原任务 | 现交付与未闭面 | 当前owner与出口判据 |
|---|---|---|
| M2A-03完整新库、LIB-M2-05/07/08、AC-NATIVE/AC-COVERAGE；T-09/T-A03 | 973可按已裁定面收尾，不能据其reviewed/归档宣称所有文本/压缩/存储或AC OS操作已验 | 1064承接973 D-01..D-09，1065承接A-01..A-04；保原行ID/输入/oracle/平台/backend/provider，引用真实后继正式来源 |
| 1064文本构造/库面 | 车道落点与972解码项同批范围仍待1064/T-01具名决定；目前无实现/运行证据 | 当前required Windows库面须1064真实调用/值/错误/资源正反例及正式来源后闭相应A行；不以975新增TextFromChar已降落推断字节切片/完整库已解决 |
| 1065 AC系统操作 | 973当前AC面为-10具名unsupported及负控，非50操作真实OS正例 | 1065交Windows真身/实际OS结果与原独立oracle；Linux代码/完整用例移交1063，Linux实跑仍按既定延期，不用Windows冒Linux PASS |
| 977/A vs 977/B | PY正式切片56aadead4可消费；r11剩REVERSE及最终收口均B | T-A03核所需PY支持/旧新义务映射与证据适用性；M2A-08不等977整包归档，B的REVERSE也不改A-required |

T-A03沿原矩阵/adapter更新当前best receipt、后继owner及状态，继承Godot目标裁定版本，不删旧行/不另建认证器。1064/1065的正式输入只阻本包所承完整库/覆盖出口，不阻973自身合入归档，也不让早期980/HELLO反等整包新库。T-A04审核上述映射及原required A/RG全闭，T-A05才可交M2A-08。975的跨车道模块/泛型decode[T]未闭仍由其原owner核实际承接，973有限面落主不自动关闭这些组合差额。

### 当前B1/MODULE/D行/AC接收表（r19优先）

| 原义务/切片 | 实际producer任务 | 本包接收与独立出口判据 |
|---|---|---|
| IF-975-B1（原语/六路径及owned边界） | 975 T-02已有实现＋T-B1-V/R/D | T-A03核原source/schema/指纹/runtime/每适用端真实值/错误/资源＋独立phase复审/落主身份；WIP或“975已开工”不充正式来源 |
| IF-1064-MODULE（原975→973装配差额） | 1064 T-MODULE-I/V/R/D | 真实canonical数据模块与private/public、std身份/冷恢复的正反例，正式grammar/exports/ABI/provider来源；975原T-03/04业务验收仍独立必须 |
| 原D-01..09、同车道编码解码、IF-1064-READ-FRAME | 1064 T-02/03/04＋T-KV/T-FRAME、T-05/06/07 | 原行ID/旧source/expected、Option空/缺失区分、ZIP方法/预算/持久文件、每实际平台/provider独立结果齐；D09仅类型裁定或C/Rust旧写形不关行 |
| 原A-01..04/IF-1065-ACSYS | 1065 T-01..04/05/06 | 50项签名/目标适用性，W36真实OS/账本、N14真实拒绝，旧-10控制仅保N；四端各对独立oracle，Linux代码/用例交1063 NOT_RUN |
| 完整975数据/typed业务和RG | 975原T-03/T-08/T-04及其他原RG owner | 不因MODULE或B1已交就关闭全JSON/DB/API/迁移/错误/资源；结构感知decode复用966由975负责，不等归档966重交 |

T-A03沿原矩阵/typed adapter补上述子行与来源，不另建认证器。新接口收据字段沿现有合同，至少repo/SHA/Plan revision、原source/旧新case/oracle、grammar/schema/ABI/provider/target、真实结果与不适用/未交边界。975B1可先交→1064 MODULE→975完整消费；973已交→1065→1064适用AC终验，1065不等1064整包；本包A出口只收各required A来源，不等待下游B或977全包。

旧A/覆盖/原生/FFI/源身份/预算/RG及本版Godot非required范围保持。源/grammar/ABI改变后的旧证据按真实读集重核；少行、错source、泛用旧pass、错误平台、未跑/emit当真实运行都阻相应原行。全部required齐后才T-A04独立复核→T-A05正式M2A-08，不靠细化合同增加完成数。

### WAVES-20261116：本轮唯一阶段与交接合同

本节与各原任务的新阶段标签为当前权威；原§9已交收据保留。旧文的“A须完整UI/Widgets/图形AV-A2R/完整LANGUAGE/PARITY”属于本轮正式迁交，不再是A出口前提；最终产品义务不删。只有下表MIN-A部分进入985/A，全部其余图形产品覆盖按同一行ID移交B波，未实现标pending，不冒PASS/NA。

| A最小真实交付 / owner | 精确关闭条件 / 移交边界 |
|---|---|
| MIN-A-UI / 980 NATIVE＋已交DIALECT/STATE/HELLO | 普通Node项目公开入口：字面量Hello、普通函数返回中文、由002原计数业务派生的最小Node Counter，真实AV→967自动binding/shim受检Rust FFI实际UI更新→Iced，真实事件回AV；单例0→1→0→0、两实例定向2:1，坏Schema/ABI/权限在UI效果前拒、关闭/重开/late事件及坏更新恢复。反例移除实际UI提交而只留byte_len须红。001/002的完整主题/选择/全部组件和其余示例归W1，不把最小副本冒完整原例通过。 |
| MIN-A-AGE / 981 SOURCE、SESSION/LANGUAGE/PROOF的A部分 | 正常多模块Node/helper原span与共享verify，复用既有g01/Showcase来源建立最小真实UI＋一个2D/一个3D呈现及一次事件/更新/关闭，独立CPU值/可见ROI与资源owner一致；只AV，错误编辑保旧、修复恢复。完整AGE-L01..05/五view/全场景/完整游戏及A2R转W1/W3，不能以旧整包LANGUAGE/RUST/PARITY作此A前置。 |
| MIN-A-OS / OS064 HOST/PROOF的A部分 | 真实OS启动/注册/激活/窗口会话链，隔离候选只注册上述Hello与Counter Node见证，真实输入/双实例隔离、关闭回收、错host协议拒绝；不以空host/DryRun/附加无业务窗口代替。原shell完整业务、Notes/Clock全源码旅程、28项App和双Host/配置/数据/全桌面Node根均归OS波，不作为A前提。 |
| 语言/库/FFI/SDK与其他RG / 原owner＋985 | 既定普通语言/新版标准库、AC真实基础、非图形A2R/FFI与982/983/984的A基础不因UI波次取消；只取所用精确正式接口。A矩阵保全分母并标明所有B承接行及关闭oracle。MIN-A均真验证、独立复核、正式合入且其余A-required齐，985/T-A05才出M2A-08。 |

MIN-A不是语法/能力的永久上限，是本轮明确的代表性产品验收。完整旧例和组件的原业务/错误/资源/独立expected留在B；所有后延逐行绑定原case/source/hash→原AC/V→阶段/波次→真实producer/consumer→owner/关闭条件。已交Hello（AV文本＋F0字节探针＋Rust绘制）和AXR0（A2R有界输出上窗）只复用原范围，不代新的UI-FFI入口或任何完整波次。
### B三波与批次粒度：同源继承，按后端整条爬梯

| 波次 / 原执行owner | 开工与正式交付 / 后继 |
|---|---|
| W1 / 980新增B-AV-I/V/R；981 SESSION/LANGUAGE/CONTENT/FUSION的AV部分 | M2A-08后，从真正UI-FFI Hello到完整examples/ui及Widgets相关组件；逐组只实施/真实验证AV。交`UI-W1-AV`（完整示例/组件分母及所需AGE语义/宿主AV行），不等AC、A2R、WEB/TAURI全产品或OS全量。 |
| W2 / 1068 T-04；981 FUSION等AC领域部分 | UI-W1-AV正式来源后接同源AC机器业务/事件→同一Rust/Iced宿主；复用实际AAC模块暴露缺口、1067提供所需原生异步。交`UI-W2-AC`，不等1068整包/T-06三端汇总/完整AAC自举或W3。 |
| W3 / 980 B-RUST-I/V/R；981 RUST、相关RUST2D/3D与FUSION的A2R部分 | UI-W2-AC正式来源后才新增图形A2R实现/生成/真实rustc编译/运行；全UI/Widgets交`UI-W3-A2R`。已有A2R代码/收据保留，不重做已验部分；最终完整游戏包仍核981原G2/G3义务，不据一个stdout窗通过。 |
| OS波 / OS064 HOST/PROOF的完整B部分、A01..28、ASSEMBLY | 三个UI波次正式齐后做完整桌面/应用迁移，先真实桌面Node根/生命周期，再按App依赖分组，最终装配/数据/模式/双Host；不等985/B最终签或986/987。OS核心实现/注册/gitlink只归OS064。 |
| 汇合 / 985 B、986、987；984 CLOSURE | 三波＋OS与其余原产品/库/服务/游戏required齐→985/B候选认证→986安装升级默认→987无旧链重建终验。不得反向要求OS波先拿985/B，或让W2先等本包三端最终完成。 |

`UI-W1-AV/UI-W2-AC/UI-W3-A2R`是**拟定阶段收据标识**，沿现有owner phase review/fold正式交接，不是新Plan/状态/注册系统。每波独立满足其后端的行为/交互/错误/资源oracle即可交，不要求同批三端；第三波/最终门才汇合三端。每条来源变化只使实际受影响证据失效。AC非图形补齐、AAC工作可按各自已授权阶段推进；完整AAC Gen2/3不成为W2前置。

AV波日常不编译执行A2R UI，不跑整套多端UI；Rust宿主/FFI provider可以预构建复用，AV-only不承诺首建宿主无需rustc。共享IR/ABI变更仍做必要受影响内部/非图形检查，不能以AV-only掩盖回归，但不得以此恢复每批整套A2R应用构建。W2/W3分别只跑当前后端＋受影响已交后端；最终联合验收复用有效来源，需重放时按真实改动和身份选择。
### 985逐项接收与分母守恒

| 原要求/证据 | A出口精确关闭 / B承接与最终关闭 |
|---|---|
| RG-A03 全传统UI/全部组件/多模式AV-A2R | A仅MIN-A-UI真实AV入口/事件/生命周期；原完整字段/示例/Widgets/主题/输入/各模式映射980 B-AV、1068 T-04、980 B-RUST及最终PARITY，原行不删。 |
| RG-A04 完整Node/LANGUAGE/RUST/混合PROOF | A仅MIN-A-AGE真实AV代表性多模块/最小UI-2D-3D/更新关闭；完整AGE-L/AG-A、五view、原复杂实例/Frame/资源和三端图形格归981各原任务的W1/W2/W3，游戏和独立AGE最终门保留。 |
| RG-A05 shell＋Notes/Clock原AV-A2R与双Host | A仅MIN-A-OS真实启动/激活/窗口/Node见证；Notes完整原oracle交OS064 A15，Clock交A12，完整shell/Node根/登记App/双Host/旧数据/daemon交HOST B/ASSEMBLY，三波之后执行。 |
| RG-A01/02/06、语言/标准库/非图形FFI/SDK | 原基础与真实CLI/网络/生成服务/仓外consumer门保持；这次UI爬梯不取消普通语言/库A2R或AC基础，不把全部A-required只缩成一个Hello。 |
| W1接收 / T-08[B]与T-02/03 | 980 B-AV-R＋981所需AV域正式来源，全部示例/Widgets分母/原oracle/运行结果齐，独立phase review/fold出UI-W1-AV；AC/A2R仍pending不阻W1，不能提前签三端。 |
| W2接收 / 同上 | 1068 T-04＋981 AC域所需正式结果，全部同源组件/示例AC机码/真实窗/错误/资源齐，出UI-W2-AC；不等待1068全包/T-06或A2R W3。 |
| W3接收 / 同上 | 980 RUST＋981所需生成/域结果，全部示例/Widgets真A2R生成/编译/运行与原oracle齐，出UI-W3-A2R；此后OS064可开始全量产品迁移。 |
| B候选最终 / T-04→T-05→T-06/07 | 三波＋OS064完整产品＋984 CLOSURE及其余原required齐，986/987的未执行安装/退役行仍明确交其owner；本包签候选产品/能力，不冒安装/无legacy正式版已交。986消费此候选，987消费986及B来源，不能反等安装/退役才交候选。 |

阶段收据沿原矩阵/父侧认证和owner phase复核合入，不新增注册器。T-A03先将原表行按这张迁交映射重冻（case/source/hash/expected/phase/wave/backend/producer/consumer/owner/条件/正式来源）；旧4项勾选/162行只是历史输入，不自动代表新分母通过。A-required缺证仍阻A；B-required只要明确承接且未冒PASS可留pending，不能偷偷删行。T-08整项跨A/三波/OS最终仍只登记一次，不因W1/W2切片已交提前勾整项。

### A基础出口与B三端一致性（2026-11-16当前权威）

本节按用户最新决定覆盖旧“AC全部能力必须在A关闭、A2C同等完整硬门”的范围/阶段含义，不改历史已验结果。M2最终AV/AC/A2R在同平台的公共语言、标准库、FFI、UI/AGE领域能力一致；A2C完整对齐增强，保已有代码/用例/支持记录及准确拒绝，不因缺完整面阻M2。A2PY原标准库和AV三provider义务不变。

A仍核普通语言/标准库/非图形FFI/SDK的原required及RG-A01/02/06；RG-A03/04/05按本轮MIN-A精确表，完整UI/Widgets/图形A2R已正式交B三波，AC至少有真实可用的多模块、基础类型/owned数据、泛型/Option/Result基本链、所需CTE/FFI、默认文本/分配/同步文件等AAC启动能力。AC其余行可以B-required移交，逐行保原ID/输入/独立oracle/资源/失败义务、现状、producer与关闭条件，owner为[1067](1067-ace-ac-native-async-and-task-parity.md)/ [1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)或其实际库/域owner；不记PASS/NA、不删除行。A2C非required应有用户范围标记，不冒已支持。

| 出口/现原任务 | 当前完成条件 |
|---|---|
| T-A01/A02，T-A03及T-A04/A05 | 冻结“真实A基础＋完整AC后延账”，A-required全部真实关闭；允许AC异步等B行仍未实现，但缺owner/关闭条件或基础仅stub不能签A。旧矩阵/4完成仅为历史，不证明新阶段分类 |
| M2A-08 / G-AAC | 正式A基础及后延账已独立复核/落主，AAC与B可开始；不等待1067/1068整包或全部AC能力 |
| T-02/T-03/T-08[B]/T-04及独立最终review | 收1068正式三端闭合与AAC阶段实证，原全部B产品/库/FFI/安装退役不减。公共能力不得以AC原profile不支持豁免 |
| 986/987最终产物 | 同版AV/AC/A2R完整能力与资源/错误/依赖证明；A2C完整对齐非硬门；完整AAC Gen2/Gen3签章独立 |

[1067](1067-ace-ac-native-async-and-task-parity.md)补native Future/async/Task；[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)补全部剩余AC接缝/跨域组合并收AAC真实源码模块。AAC未用到的特性仍按原语料补验，编译器happy-path或Gen1成功不能代完整语言。库/域缺口回原owner/新具名后继，归档968等不重开；当前Specs中的native拒绝仍是事实，本轮不改写成已支持。

frontmatter新增1068只约束本Plan最终B并集，不反向约束A出口；AAC1044/1045不依整个985或1068，任务级阶段图不形成环。Linux标准库代码/完整case继续NOT_RUN交1063，不能以本次阶段调整冒Linux verified或豁免Windows基础。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-PARITY-STAGED | modify | docs/specs/ace-migration/m2-certification.md、docs/specs/ace-migration/replacement-scope-and-evidence.md | 旧A全四端→A基础/AC具名后延，最终AV/AC/A2R完整一致；A2C增强，AAC阶段实证 | 用户最新门 | AC-PARITY-STAGED |

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-UI-WAVES | modify | docs/specs/ace-migration/m2-certification.md、replacement-scope-and-evidence.md（实际交付后） | A全图形AV-A2R→MIN-A/RG精确子面；B完整示例/Widgets三波与OS最终，旧分母逐项迁交 | 用户本轮三波与A最小出口裁定 | AC-UI-WAVES、AC-A-EXIT、AC-PHASE |

A通过只认可真实ACE执行：旧载体/原TestCase接口可复用，旧引擎输出只作具名调查/对照，不充候选actual。T-A03把source/oracle/入口/engine/backend绑定同一检查；旧入口不能实际dispatch ACE或结果只有旧AutoVM/AST发射器，仍属A缺证，不能因1066格式迁移放B而签通过。

### 1066在A的唯一接收合同

A只接IF-1066-A-CORRECTNESS：A-DEN（原集合/角色/条件）、A-ORACLE（两个缺result、九空H与A实际消费者）、A-NOWRITE（正常验收禁写作者树，保独立验证）、A-EVIDENCE（实际版本/源/oracle/actual/判决）。依1066原T-01/02/03/07/08/09/10/11 A子面→T-12[A]独立phase→T-13[A]正式fold，不等13/13。完整scheduler格式、N1050/caches/cookbook doc-only、P工程doc、v2治理全切与C30 proposal、250编译全集留B，原1170角色/33验证/19断言/ignore/ledger不减。

A原required的实际语言/库/FFI/非GUIA2R仍要真实关闭；旧载体可以按冻结版本提供独立输入，但不能把旧入口绿冒新doc完整。最小A四控制NC-A1..4与全部实际检查同版身份齐，B余量分任务/Case/原oracle/条件/最终归属可核才接；1066计划存在/1062归档/文件在盘均不充PASS。B完整1066来源进入985 T-04/原最终范围接收，不反卡W1/W2/OS开工或A。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-A-FREEZE | modify | docs/specs/ace-migration/m2-certification.md、replacement-scope-and-evidence.md及auto-test-case/ace-m2-stage-verification.md（实际交付后） | 旧162粗粒度/审计归属不清→父行原子义务A/B、35发现逐项与1066A片段；关闭事实和迁交分开 | 用户本轮重冻A | AC-A-FREEZE |

### EVENT-SPLIT拟议规范增量（交付时由原merge任务沉淀）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-EVENT-PHASE | modify（拟议） | `docs/specs/ace-migration/m2-certification.md` | 交付只沉淀实际完成的A空载荷/声明保真/拒绝、B typed payload/参数/来源与各consumer；明确逐批解锁，r23分母与完整义务保持 | 三缺口具体owner与避免整包循环 | AC-EVENT-PHASE |


## 6. 测试设计

### V-CFG-HANDOFF（本轮新增接收控制，未执行）

沿[配置设计九行Case](../design/ace-config-document-evaluation.md#6-验证合同与旧用例映射)，在本包所用真实consumer/原测试入口验合法动态值及坏源/错SDK-context/零旧求值/失败保旧/修复下一次read；把route C legacy config冒ACE、只核same_sdk、缺source或仅用静态投影则拒绝。原V全部保留，不新造runner。实际未用配置的域不增加虚假依赖；原平台/ignore/输入规模保持。

### EVENT-SPLIT认证验证（拟议，归原T-A03/A04及B原任务）

| V / 任务 | 正反输入与关闭判据 |
|---|---|
| V-EVT-A-RECEIPT / T-A03→T-A04 | 正式980 MIN-A来源＋981 EVENT-A（含#6i冻结身份两红真实关闭）/MIN-A-AGE/OS及其余A输入，对r23原子键验source/oracle/Schema/SDK/ABI/actual。给980 executing但其A已正式落主应可接；把分支语法PASS、常量体spliced=1、缺A真实呈现或旧引擎PASS冒当前来源必须拒。 |
| V-EVT-PHASE-MAP / T-A03/A04 | 三键逐原V/源/consumer映A最小guard或实际必需正例、B-W1语义/AV、独立WEB、W2/W3消费；缺owner/关闭oracle/误把真A失败移B必须拒。仅缺未用到的B完整payload时不得反卡A；普通函数/node/actor参数仍A按原合同。 |
| V-EVT-GATE-CERT / T-A03/A04/A05与T-02/03/T-08[B]/T-04 | A全齐才A04→A05，缺981 A不能先签M2A-08“解锁自身”；B分波件仍pending。B最终拿三事件producer与真实AV/AC/A2R/WEB等原consumer全结果；用A拒绝样本或#6i语法取代B正例必须拒。 |


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
| V-A-EXIT | AC-A-EXIT | T-A03按本revision冻结普通语言/库/非图形FFI/SDK既定后端及RG-A01/02/06；RG-A03/04/05仅MIN-A真实子面；实际职责/窗口/HTTP单实例 | 新M2A-01..07 A-required全闭、A无unknown/NOT_RUN；完整图形/Widgets/OS/安装行具名B-pending，不能拿旧全量A矩阵反卡，也不能漏原分母 |
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

### 当前目标范围验证（拟实施，未运行）

| Case | AC/任务 | 输入→producer→consumer→独立oracle与负控 |
|---|---|---|
| V-TARGET-SCOPE | AC-TARGET-SCOPE/AC-A-MAP/AC-A-EXIT；T-A03/A04/A05、T-04 | 旧162行矩阵/目标腿→原typed adapter当前范围版本→A/B出口聚合→独立分类oracle：Godot工具缺席+其目标deferred-by-user合法，生产TS保持required；删除范围裁定来源、将Vue必需TS标deferred、整撤含普通Auto语义行、无源宣称Godot PASS、任一其他required仍pending却放出口均必须具名RED。核旧源/ID与去向留存，计required/deferred/执行结果分开，不将旧r16绿当本revision运行证据。 |

### 当前系统库移交验证（拟由原任务实施，未运行）

| Case | AC/owner任务 | 原输入→真实消费→可区分结果 |
|---|---|---|
| V-IO-HANDOFF | AC-A-MAP/AC-A-EXIT/AC-NATIVE/AC-COVERAGE；T-A03，T-A04独立复核 | 原973 D/A移交表→原矩阵/adapter owner与source绑定→真实库/AC consumer→独立字节/文本/OS副作用与资源oracle。只有973归档但无1064/1065实际来源、AC仍-10却报OS正例、用TextFromChar代byte→Str闭环、漏移交行或将977 B-REVERSE设A前置均须被拒；合法受限973收尾不受反阻，LinuxNOT_RUN/1063和Godot非required分别保原状态。 |

### 当前交接补充验证（未实施）

| Case | AC/owner | 可区分判据 |
|---|---|---|
| V-MODULE-HANDOFF | AC-A-MAP/AC-A-EXIT/AC-COVERAGE；T-A03，T-A04独立审 | 975B1→1064 MODULE→975真实canonical consumer及D行/AC正式来源均有原case与独立oracle；构造“975全包先完才交B1/1064全包先完才交MODULE”的错误依赖须在阶段图被识别；全pub/删pub镜面源、private泄露、旧std来源/grammar、D09仅裁定、W op恒-10、LinuxNOT_RUN冒PASS/拿N例代W正例都须具名RED；合法独立切片可接收但不冒全包/阶段出口。 |

### 阶段调整的区分性验证（拟议，未执行）

| V / AC | 原owner / 输入 | 必须观察到的结果 |
|---|---|---|
| V-PARITY-STAGED / AC-PARITY-STAGED | T-A03、T-A04/A05：真实A基础＋AC async待补、完整B-owner/输入/oracle；再去owner或把基础换stub | 第一组可签A基础但不得签最终三端；后两组拒签A，旧矩阵无新范围版本也不得接收 |
| V-PARITY-FINAL / AC-PARITY-STAGED | B的T-02/03/T-08[B]/T-04及原独立review：AAC实际模块绿但AC留一项~T/库/领域差额；补齐后完整三端对独立expected；A2C完整面缺失 | 有AC差额时最终门红；完整三端/原其余required齐才绿；A2C缺完整面不单独阻门，不能误记其已支持 |
| V-AAC-NOCYCLE / AC-PARITY-STAGED | 1044/1045阶段consumer只用已交基础，后续一个任务实际使用待补能力，及修复后重跑 | 前者可开工，后者只阻其依赖切片；不得要求整个AAC先完成才能修AC，也不得让缺口fallback旧VM通过 |

本节沿原义务矩阵/adapter实现；原完整组/负控/独立oracle保持。全Plan AC-COVERAGE等在B齐前仍未整项关闭，A只收其明确阶段部分，不能用阶段修改虚增任务数/完成数。

### Node桌面分阶段区分性验证（拟议，未执行）

| V / AC / 原任务 | 判别输入与结果 |
|---|---|
| V-NODE-DESKTOP-A / AC-NODE-DESKTOP / T-A03、T-A04/A05 | MIN-A真AV入口与完整UI/Widgets三波/B-OS逐行承接可签A；把入口换HELLO byte_len或删owner拒签。普通语言/标准库/非图形FFI原基础不减，图形A2R/完整UI不是A前置；最终B仍要全部三端/产品结果。 |
| V-NODE-DESKTOP-B / AC-NODE-DESKTOP / T-02/03、T-08[B]、T-04/T-05 | 同源AC真窗/机器事件/资源及981混合域与包证据齐可签B；替换为A2R stdout窗、AV产品包、CPU描述文件或删一AC域行必须红，修复后同候选重验。 |

由原认证矩阵/adapter消费980/1068/981正式来源；具名新用例由原work建立、独立review检查实际source/producer/consumer，当前仅规划未运行。新增接收检查不以已勾T-A01/A02/T-09的旧证据冒新门通过。

### 三波/最小出口认证验证（拟议，未执行）

| V / 原owner | 区分性输入与结果 |
|---|---|
| V-MIN-A-EXIT / T-A03、T-A04/A05 | 既定语言/库/FFI/SDK/其他RG＋MIN-A-UI/AGE/OS均为真实同版来源，全部完整UI/W2/W3/OS产品行具名B承接，可签A；把A本身替byte_len/空OS/无GPU证据或漏一个原行去向必须红。缺B完整UI/A2R但移交齐不得继续要求其通过才签A。 |
| V-UI-WAVES / T-08[B]、T-02/03 | W1全部AV行齐但AC/A2R pending，允许正式W1；W2全部AC行齐且W3 pending，允许正式W2；W3取完整同源A2R真运行。任何漏Widgets/示例、错后端产物/未落主/假worker PASS不得签当前波，不能一次W1签三端。 |
| V-UI-OS-ORDER / T-03/04、原final | W3正式齐后OS产品实施，可在985/B最终尚未交时进行；最终缺任一OS/App/桌面模式/三端域行必须拒签候选B。全产品候选与独立review齐可交986，不能反等986/987最终安装退役证明。 |

沿原TestCase/父侧矩阵扩字段/查询，T-A03与B原任务建负控，旧矩阵生成/消费身份不匹配应拒；无新tm/调度器。按当前阶段选职责，AV波不跑图形A2R；第三波及最终只补必要真实并集，旧身份有效证据复用。新用例/新出口本轮未执行。

### 重冻分母与准入负控（拟议，全部未执行）

| V / 原owner | 输入→区分性判据 |
|---|---|
| V-A-FREEZE / T-A03/A04 | 原162父行＋本表原子check、F01..35唯一分类与1066各集合引用；删F30/33或N结果通道、把1035仅登记B/泛用M1 NA必须拒；合法全映射仍保原平台/ignore/波次。父行数不等于实际执行check数。 |
| V-A-CORRECT / T-A03/A04 | 8个A包修复/充分反证、原Case/合法邻例/真实consumer/错误/资源与正式SHA；只组总绿/未运行/分支WIP/别的failure类/旧Str正常例冒聚合fault必须拒。注入Infra与Resource/Cancelled分类沿1035 r4，不造新oracle。 |
| V-A-DEFER / T-A03/A04 | 1030仅历史三个consumer、1038研究和1066完整B迁移未完，但A约束/正确性片段及其余A全齐，可签A；若A真实调用864或缺A片段，不可沿B标签过门。B未完成不充PASS，也不机械要求整1066/全研究先归档。 |
| V-A-1066 / T-A03/04 | IF-1066-A-CORRECTNESS正式输入＋NC-A1..4；只改result仍stdout正确、漏空H、写作者文件、错receipt/父exit0缺子actual必须拒。完整B格式/250编译pending不阻A；原A-required执行缺证仍阻。 |
| V-A-PROVENANCE / T-A03/A04/A05 | 读取worktree旧162表但未落实本revision分类、收未落主审计/修复来源或把本规划文档标执行PASS须红；当前正式reviewed/landed代码/输入/工具/actual与原condition一致后才可签M2A-08。 |

原T-A03在现有义务矩阵/父侧adapter内实现上述接收/负控，名称为拟定检查键，尚无新可运行命令；不另建registry/tm或修改旧语料以凑绿。T-A04从真实工件独立复核，T-A05只做已审来源落地/收据，不把正确性留merge。分类变化/源码漂移只重核实际影响行，正式性能/全仓周期门仍按原条件，不附到每个审计包。

## 7. 验收标准

- [ ] **AC-CFG-HANDOFF / work**：§5本包实际配置consumer/阶段的正式来源和V-CFG-HANDOFF闭合，区分宿主AV与目标业务、原A与完整B；不以未交接口/上游状态或peer-equal代真实consumer与独立oracle。原工作任务负责，原final独立复审核验。

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

### 当前目标范围验收

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-TARGET-SCOPE | work | 959旧义务有当前用户范围覆盖，Godot明确后置且不计PASS/不阻出口，生产TS/JS及其他required未误撤；旧矩阵/adapter按当前revision重冻并通过具名正反控；M2A-08仍需所有required正式证据 | T-A03/T-A04/T-A05、B期T-04、V-TARGET-SCOPE；原独立review逐项复核 |

- [ ] **AC-PARITY-STAGED / work**：上述三条V在各自原A/B work与独立review关闭；A有真实基础及完整后延账，B的三端差额=0，A2C增强及AAC独立签章边界正确。现14任务/4完成不变，T-A03重冻新分类与必要adapter，不将旧结果冒新门通过。

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-NODE-DESKTOP | work | A的MIN-A真实AV入口/事件/owner及B的W1 AV→W2 AC→W3 A2R、OS域/包分别闭合；V-NODE-DESKTOP-A/B满足独立判据，Hello/A2R不冒AC | T-A03与B T-02/03/T-08[B]/T-04；原独立review |

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-UI-WAVES | work | MIN-A精确出口与完整分母/B承接一致；W1/W2/W3独立后端收据、OS先后关系及最终全产品候选齐；V-MIN-A-EXIT/UI-WAVES/UI-OS-ORDER负控敏感，旧A全UI门不残留 | T-A03、T-08[B]/T-02/03/04、原独立review/fold |

| ID | owner_stage | 当前完成谓词 | 验证/任务 |
|---|---|---|---|
| AC-A-FREEZE | work | 原162父行原子化、35发现逐项与1066 A/B边界落实原矩阵；8个A正确性包及A约束/证据/1066 A切片真实正式闭合，B仅具名pending，V-A-FREEZE/CORRECT/DEFER/1066/PROVENANCE敏感；所有其他A-required齐 | T-A03、T-08[A]，T-A04独立复核；T-A05正式来源 |

### EVENT-SPLIT验收补充（owner_stage沿原work/review/merge，待实施）

| AC | 原承接任务 | 可观察关闭条件 |
|---|---|---|
| AC-EVENT-PHASE | T-A03/A04/A05、T-08[A/B]、T-02/03/T-04 | §5三缺口逐阶段/真实source/oracle/owner/现有任务可追；A真实正例/保真/安全拒绝齐，B支持/执行/资源/错误与完整原用例全部闭；语法/splice/历史或worker总绿不越级。对应本节V-EVT-*，独立review绑定revision与正式来源；当前规划不得勾PASS。 |



## 8. 执行步骤

当前共14任务，已完成4项（沿既有回执；本轮未新增完成）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A准备｜work｜AC-01/AC-PHASE**：先核§5的A最小前置、真实owner/源/接口/命令及本包实施授权，冻结A能力或具名PROOF；B部分只登记原输入/oracle与缺口，正式实现须M2A-08及其批次前置，不把整包最终depends_on当A开工前置。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。
  [✅ 2026-11-16] A期冻结记录见§5"T-01 A期冻结记录"：授权链（971 goal 续行指令）、A最小前置核实（语言族959-970/972归档+早期切片967/F0、969/E0、978/B0、979/H0@71472d975、980/HELLO+971/A@2f1eed27f/c1561d94 落主；1039-1043 基础设施归档）、硬前置未闭清单（973/974/975/976/977/978收口/979余量/980/981/982/983/984/OS064）、M2A-01..07门级现值与可执行面结论（M2A-08本轮不可闭合；T-A01/T-A02/T-09/T-08[A]已交来源部分可执行）、平台/资源/命令冻结（Windows x64；Linux腿归1063）。PIO/FOUNDATION 子合同来源状态核实并冻结（1039/1040/1041已齐，973算法腿pending不冒原生完成）；worktree `.wt/lang-985/auto-lang` 已建。

- [x] **T-A01｜M2-A｜owner_stage: work｜AC-A-MAP/AC-NATIVE**：从959及FFI/LIB/UI/AGE增补冻结A/B义务矩阵（纳入五包Hello早期20任务/对应原义务，去重计case），盘点原语言/库用例、原expected/条件与新check去向；每条归继承/具名语义替代/原内部退役/B产品，不按旧目录或M1 NA缩分母，执行V-A-MAP负控。 新增V-NATIVE：在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16, worktree commit 0dff19cd4] 交付 **A/B义务矩阵 `autolang.ab-obligations/1`**（`crates/auto-lang/test/cases/ace-m2/m2-cert-985/ab-obligations.tsv`，162行=959冻结L-01..49/S-01..25/A/B/C/D/E/F/K全部入口产品行+Design56 FFI-M2-01..08/LIB-M2-01..08增补；每行 phase(A/B/AB)×disposition(inherited/named-replacement/internal-retire/product-B)×successor×best-current-receipt；早期切片去重计入对应行：967/F0→FFI行、969/E0→A-01..03/B-01、978/B0→LIB-02/FFI-07(inflight)、979/H0→S-23/L-44、980/HELLO→C-02/B-01(无spec收据保持pending，产品闭口本就B)、971/A→S-24/A-04/A-07/C-06/D-05/D-07/F-07/F-10）。`landed`语义=该行**当前最佳已审收据**（可为早期/A切片，sha16锚定至 docs/specs 或 docs/plans/archive 实文件），全行闭口判定归T-A03——MAP≠EXIT。**V-A-MAP负控**（`crates/auto-lang-core/src/test_cases/obligation_ledger.rs`，7例全绿0.03s，每例对真实矩阵做突变并断言具名RED）：缺行(冻结行缺失=RED不缩分母)、重复行、错source(sha16失配=陈旧/伪造收据RED)、非收据路径(源码/测试自报PASS永不充收据RED)、词表+无owner行、pending虚报收据(证据膨胀RED)+真实矩阵绿。准入层四族(旧binary/emit冒run/假workerPASS/timeout)绑定ac/ac_av准入脊柱(父侧deadline/kill+schema admission+父侧独立裁判worker不见期望)，逐能力绑定归T-A02。子合同去向：V-NATIVE双腿/COVERAGE六路径逐操作/LOAD消费/1024 F14案例试验账——按计划文本归T-A02/T-09/T-A03继续（本任务的义务并入已完成：LIB-07/08在矩阵、F14守恒由矩阵+缺行负控承担）。V-A-MAP真实命令冻结：`cargo test -p auto-lang-core --lib obligation_ledger`（组登记归T-A02）。

- [x] **T-A02｜M2-A｜owner_stage: work｜AC-A-COVER/AC-NATIVE**：先用普通source和FFI/async资源两配方核父侧观测支持；在现TestCase/ACE owner接版本化M2描述、必要typed协议/薄adapter与父侧stage聚合/负控，unsupported保持ERROR；注册拟新增跟踪ace-m2源/expected/适用性/旧新映射，督促每能力owner补规范/组合/边界case，冻结V-A-COVER真实命令，不另造runner。 新增V-NATIVE：在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16, worktree commit d6342f534] **两配方父侧观测核实+M2描述注册+四准入族绑定**：跟踪语料 `ace-m2/m2-cover-985`（plain `ac-av/core-run-v1` exit14 + owned `ac-av/resource-events-v1` exit405，独立手算oracle）；owned-relay 案例在冻结父表登记（frames2/record=2/str=4/free-order[3,0,1,2]——由真实父侧观测运行推导，探针环本身证明未登记案例=RED case-registration 失配，准入控制实际生效）。conformance 双门：`ace_m2_p985_cover_suite_green`（两配方端到端、父侧独立裁判，0.84s）+ `ace_m2_p985_cover_wrong_oracle_is_red`（错exit=RED指名exit轴且局部化，0.83s）。**四准入族绑定具名红控（2026-11-16 实测验证）**：旧binary/陈旧身份=`ac_av_probe_mixed_recipe_stale_identity_is_order_independent`(PASS 234s)；emit冒run=`ace_m1_golden_drift_is_red`(PASS 65.7s)+kind混淆=`ace_m1_native_typed_891_wrong_oracle_and_wrong_kind_turn_red`(组内已知基线红，控制角色在、状态如实)；假workerPASS=`ace_m1_base_negative_oracle_fails`(PASS 65.9s)+`ac_av_probe_negative_oracle_fails`(PASS 0.77s)+新`ace_m2_p985_cover_wrong_oracle_is_red`(本注册负控)；timeout=`ac_av_probe_prep_executor_contained_failure_shapes`(PASS 59s)+`ac_av_worker_contract_faults_are_infra`(PASS 3.6s,故障=infra永不充语言负例)。**V-A-COVER真实命令冻结**：复用组 `cargo t auto-test-case ace-source ace-stdlib ace-adapters`（组存在已核 test-groups 1513/1634；本轮IR未触其代码面，复用证据）＋新执行腿（experimental/ace 工作区）`cargo nextest run -p ace-conformance --test conformance --config-file ../../.config/nextest-full.toml -E "test(ace_m2_p985_cover)"`＋账本腿 `cargo test -p auto-lang-core --lib obligation_ledger`(7/7)。薄adapter未新增（现配方承载，不另造runner）。子合同去向：V-NATIVE长度/ABI观测与COVERAGE六路径逐操作面随能力owner案例继续（T-09/T-A03核）；督促各owner补规范/组合/边界case为持续义务，A批后续/T-A03汇总。

- [x] **T-09｜M2-A｜owner_stage: work｜AC-07/AC-NATIVE**：冻结并审计FFI-M2-01..08/LIB-M2-01..06及全部子行，运行四端真实外库/新库组合与缺行/手写binding/旧桥/缺工具对照，给986/987具名pending安装项。 验证V-09，A证据汇入T-A03；B最终work T-04复核身份适用性。 新增V-NATIVE：在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16] 已交来源部分交付（详§5"T-09 已交来源审计记录"）：FFI-M2-01..08 全部绑定 967 具名 fixture 并**当前树实测 33/33 PASS 9.2s**（真外库 serde_json/SQLite 端到端+负控矩阵+六路径）；LIB-M2-01..08 逐行审计（已交证据/pending owner）；对照族（缺行/手写binding/旧桥/缺工具）逐项核对；986/987 具名pending安装/退役项已登记。V-09 的 A切片证据就绪汇入 T-A03；pending 腿（973/974/975/978/977 各腿）如实保留，T-A03 出口前由其 owner 闭合。

- [ ] **T-08｜M2-A/B｜owner_stage: work｜AC-06**：分A/B执行原MIX-01..06与候选组合：A切片只等T-A02和能力/RG正式来源，进入T-A03；B切片在M2A-08后接T-02/03全产品进入T-04。保AC-06/V-08完整范围，B-pending明确交986/987，A完成不提前勾本任务。 B按W1 AV→W2 AC→W3 A2R分别接V-UI-WAVES并独立phase交接；各波未做后端不阻当前波，整项最终还等OS/其余required。 EVENT-SPLIT：A组合只取A接口；B分别接事件三件与AV/AC/A2R/WEB各自consumer，按波验不设三件之外的总门。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。
  [A片段✅ 2026-11-17, 同上commits] A切片闭环：MIX-07组合两例＋MIX-01..06 A腿聚合账（§5 r29）；整项待B（T-08[B]在M2A-08后接T-02/03全产品进T-04），A完成不提前勾本任务。

- [x] **T-A03｜M2-A｜owner_stage: work｜AC-A-EXIT/AC-NATIVE**：等A能力和RG收据正式就绪后，运行V-A-EXIT的实际完整职责/四端/新库/FFI/真实消费者及原正式门；新鲜检出重放、逐项计数/终态/独立oracle齐，A-required缺证阻出口。 新增V-NATIVE：在M2-A义务/测试中并入LIB-M2-07/08，默认自有库和显式C/Rust FFI双腿分开验；追踪原实现/符号/调用/依赖与字符串正反例。现框架缺长度/资源/ABI观测时按T-A02有界核实并版本化扩展，不能由worker自报PASS或以四端相等替独立oracle。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 FOUNDATION：按§5核已交原生底座/OS内存来源，关闭V-FOUNDATION/AC-FOUNDATION；纯算法准备不冒原生完成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 本轮新增先按§5当前范围覆盖合同迁移/重冻原矩阵与adapter、落实AC-TARGET-SCOPE/V-TARGET-SCOPE，再核出口；Godot不再required，旧162行与历史证据不删除，生产TS及其他required不误撤。 当前并入V-IO-HANDOFF：按1064 D行/1065 A行绑定原库/覆盖义务的正式来源，973有限面或stub不充完整库/AC OS正例；不等977/B或全包归档。 r19补V-MODULE-HANDOFF：按本表接975/B1及1064/MODULE独立来源、D09/FRAME与1065 W/N/L逐项结果；不把上游切片或973归档充完整数据/库。 r21须闭V-NODE-DESKTOP-A并收980正式入口/AC-B承接账；不等待AC桌面运行，但HELLO字节探针不代NATIVE/RG。 本轮先重冻最小A/全B分母与RG-A03/04/05迁交并闭V-MIN-A-EXIT；不等待完整UI/Widgets/图形A2R/OS产品，保其逐行B-owner与原oracle。 r23以A-FREEZE-20261116-r23为分母：逐原子项核8个审计A包、A约束/证据和1066正确性正式切片，完整B迁移/1030/1038不反置成A前提；本轮规划不是执行PASS。 EVENT-SPLIT：按r23原子分母接已交980 A、981 EVENT-A及全部MIN-A/原A必需；三缺口完整EVENT-B逐行留原owner/波次，不反等980/981全包。1032/1034正式已交，余A正确性/1064/1066与RG仍真实核。 #6i身份：981 T-01/LANGUAGE[A]负责新旧grammar/Schema差额、原冻结测试与SDK/cache身份，先语义核准再维护期望；980供差额、985拒旧证，A必闭不等WEB全包。
  [✅ 2026-11-17, worktree commits f93508199+ecc282df1] §5"r29执行记录"——T-08[A] MIX-07同一工程组合两例实跑绿（4.37s/5.63s，≤10s门；平台文件选择→配置读取+钉版serde解析→SQLite写入回读→双车道取消＋V-08三具名负控）；MIX-01..06 A腿绑定owner证据；矩阵重冻（L-45/S-20/S-21/S-25→B、B-02 landed、B-04/B-06 user-deferred）＋账本词表扩展＋新负控，ledger 8/8（TSV SHA=89eebafa…）；V-A-EXIT并集：p981 205/205、ace-abi f0 36/36、cover 2/2、固定组94/95（1红=master复放同红基线既有，非触面）；M2A-01..07全闭，A无unknown/NOT_RUN，B承接具名pending。

- [x] **T-A04｜M2-A｜owner_stage: review｜AC-A-REVIEW**：停止A写入，独立复核M2A-01..07与全部A任务/反例/遗漏/健康/Spec；结果绑定A revision/冻结SHA，修复后定向复核，不能用executor总PASS替证据。 按本revision核MIN-A与迁交，不得沿旧A全UI/完整LANGUAGE/RUST/PARITY误判阻塞，亦不得把最小演示冒真实基础。 r23以A-FREEZE-20261116-r23为分母：逐原子项核8个审计A包、A约束/证据和1066正确性正式切片，完整B迁移/1030/1038不反置成A前提；本轮规划不是执行PASS。 EVENT-SPLIT：执行V-EVT-A-RECEIPT/PHASE-MAP/GATE-CERT，拒绝循环等待/错版来源和把A实际失败移B；B-only缺正例不冒A失败。 #6i身份：981 T-01/LANGUAGE[A]负责新旧grammar/Schema差额、原冻结测试与SDK/cache身份，先语义核准再维护期望；980供差额、985拒旧证，A必闭不等WEB全包。
  [✅ 2026-11-17, 独立phase复审 pass @ecc282df1] §9复审收据——独立子代理上下文同树只读重建，A–H全过：diff恰5文件无越界（rust_provider dep路由最小、dep-free原路不变）；MIX-07两例oracle独立（进程内1.0.154 vs 钉版1.0.151真cargo构建）＋负控真变异真判别＋计时4.86s/6.18s≤10s门；矩阵/账本哈希逐字核对（TSV=89eebafa…、B-02锚423ef4a8975c8a51实测一致＋§6 A2C事实行实存）；裁定回链r26/r29/route C/M2-TARGET-SCOPE全成立；基线红归属核实（abi.rs最后触碰631249d99=Plan967，本批未触）；M2A-04/05/06门级抽查全master祖先；零新增warning＋新文件零fmt漂移。4条info（F1 sqlite腿pending降级沿t15先例、F2基线warning计数口径、F3 oracle等值链路如实声明、F4 conformance.rs基线fmt债务）均不阻塞。

- [x] **T-A05｜M2-A｜owner_stage: merge｜AC-A-LAND**：A review pass后按phase fold核各owner真实合入SHA及基线祖先/SDK闭包，落本阶段Spec与M2A-08收据、具名B-pending；同Plan树保留继续B，不归档985或宣称M2/v0.6完成。 交M2A-08时附完整B三波/OS账，精确版本/来源，不签M2最终或三端图形完成。 r23以A-FREEZE-20261116-r23为分母：逐原子项核8个审计A包、A约束/证据和1066正确性正式切片，完整B迁移/1030/1038不反置成A前提；本轮规划不是执行PASS。 EVENT-SPLIT：只有全部Arequired及独立A review齐才落M2A-08；不能提前签以解锁981自身。
  [✅ 2026-11-17, A批phase fold落主 delivery 95c7fbad8] §9 merge收据——owner合入SHA/基线祖先全核（r25–r28接收链＋T-A04复审G项抽查）；A阶段Spec落主 docs/specs/ace-migration/m2-certification.md（阶段口径/分母账本/MIX组合/M2A门级/B-pending，as-built）；ff-only线性落主（分支0落后无需rebase，range-diff豁免）；主检出落地检查 ledger 8/8＋MIX-07 2/2（5.51s/5.69s）。**M2A-08正式成立**：A批收据可消费，B批（T-02..07/T-08[B]/T-04）按§5阶段合同解锁开工（各自核本收据为前置）；具名B-pending：977-B/980-B批/981-B（W1/W3/EVENT-B）/982-984-B/OS064-B波/1066-B/1067/1068/986/987/1063；同Plan树保留续B（plan-985-dev@95c7fbad8），不归档985、不宣称M2/v0.6完成；批量回归%5触发在整包cleaned时正式到期（当前收据last_covered=1064，fold记录pending-at-full-closeout）。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### M2-B：全产品接管与完整收口

所有本组实现任务先核M2A-08已合入收据，再核本批§5前置；只读准备不充B执行结果。A接口变化须回能力owner修复并定向更新A收据，不静默沿用旧证据。

- [ ] **T-02｜M2-B｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 EVENT-SPLIT：B核IF-981-EVENT-PAYLOAD-B/MSG-B/PARAMS-B与实际完整consumer，不把#6i语法当语义已交。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。

- [ ] **T-03｜M2-B｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 按V-UI-OS-ORDER在三波后接OS产品，OS不等待本包最终签；阶段收据不等本包全归档。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。

- [ ] **T-04｜M2-B｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 r21同时闭V-NODE-DESKTOP-B，收1068机器guest和981真实领域窗口/包，不以A2R或AV代AC。 最终收三波＋OS/其余产品的同版候选；986/987负责安装/退役，不反置成候选B交付前置。 r23收全部B-required产品/修复/格式迁移及旧消费者义务；1030按既有关闭条件、1066完整范围与正式来源核，1038研究不得冒新重构已交。 EVENT-SPLIT：最终AC-EVENT-PHASE完整闭合，原UI/语言/资源/模式/游戏义务保全。 CFG-HANDOFF：落实§5配置接收与V-CFG-HANDOFF/AC-CFG-HANDOFF，按真实所用接口分阶段接，不等待不相干整包。

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

### 2026-11-17 CONFIG-ACE-B规划交接（r30；非实施验收）

stage: new/revision | outcome: pass（规划合同预检） | 本轮用户仅授权补承接/设计/接口/验收及排程；未work/建树/运行测试/探针/构建/合入/归档/派agent。GAP-982-CONFIG-ACE由982原B-CONFIG两仓端到端承接，能力本身保持未实现；本包按§5–8现任务生产或接收，不新增Plan/任务，不改已验A收据。下一步须未来实施指令，982先B-CONFIG-I全链前探/接口与原输入冻结，再V→独立R→正式phase交接；其他owner按所用正式来源接。

取材和静态核验：7计划123原任务ID/顺序/勾选与状态/进度保留，语义revision各增1；仅982补已交967/979直接基础依赖。9个配置判别Case/4个能力AC、typed接口/值语义/预算/生命周期/原用例映射与Spec候选已列；24节点30边阶段图无环，新增本地链接可达，18份源码/Spec/归档/计数器身份不变，目标文档git diff --check通过。旧无owner快照/route C/执行回执作历史保留，不把规划PASS当真实能力或独立产品review。M2A-08 r29/95c7fbad8冻结不变。

### AGE三路线规划对齐说明（2026-11-17；不改变本包revision/分母/进度）

[AGE统一strategy](../design/strategy/age-m2-three-route-strategy.md)新增首批后UI099–102的引擎AV合同及新版AutoForge独立产品合同。它们**均不是985/A新增前置**，本包现行A冻结、TS/Godot范围裁定和全部原A-required关闭条件保持。B只按本包既有产品分母/真实consumer实际需要接980/981/1068正式来源；AG031签AGE自身扩展required，Forge独立签产品，不把二者反置成UI三波或A出口门。本条仅规划导航，不证明A/B执行通过，不更改并发会话已写证据。

> EVENT-SPLIT-20261116历史解释：本节旧blocked/“等待980正式源”、round39/40及旧全量A门只证明当时核对/选择；当前依§5和§8逐切片合同，不用executing/整包未勾代接口缺失。旧产品结果不因本次修订自动扩大为参数/载荷或新A验收PASS。

stage: work | PLAN-985 | r25 | 2026-11-17 | outcome: pass（有界A接收预检，T-A03准备面；非T-A03出口/最终联合验收） | worktree: D:/autostack/.wt/lang-985/auto-lang @ plan-985-dev（resync master合并53828f3db＋矩阵9af917cbb；树clean） | task_ids: T-A03(准备面，不勾) | evidence: §5"985/A接收预检记录"——#6i两红实测2/2 PASS（nextest run ea9d93eb，p981全集201/201绿）；矩阵19行新landed＋29行重钉，obligation_ledger 7/7绿，新TSV SHA256=c23c8c59…；resync后985 cover 2/2＋p977 2/2绿（a2x-worker环境归因＋维护构建后复放） | dependency_revisions: master@c2f3b2aa9时点核验（1029–1037/973–979/1064/1065/1024 archived；980/A、982/A、983/A、984/A、977/PY、971/A落主SHA见§5表） | blockers: 981 MIN-A-AGE余量（SESSION/LANGUAGE-V/R/PROOF批次＋EVENT-A批级fold，review-69在途）、1066/A（已授权未实施）、OS064/A（V-OS-A-MIN绿待fold）；L-45/S-25/B-02/S-20/S-21为T-A03重冻裁定项 | next_responsible_role: 等待三份A交付齐后T-A03运行完整A并集→T-A04→T-A05（M2A-08）；不重复预跑

stage: work | PLAN-985 | r26 | 2026-11-17 | outcome: pass（第二轮接收：OS064/A接收＋981差额反馈；非T-A03出口） | task_ids: T-A03(准备面，不勾) | evidence: §5"r26增量"——OS064/A落主核验（auto-os ff-only 06d3a0b＋484c432＋61237ba，IF-OS064-A-MIN交接账对MIN-A-OS合同逐项✅，独立复核pass@06d3a0b）；981批级收据对账（review-68只闭event A＋V证据面，绑定版本全落主；PROOF批A面未交）＋五点反馈清单已留981 §10指针 | dependency_revisions: auto-os master含06d3a0b；lang 24b4ba4a7/75015a49b、ui 771be6a9/7de28330/2f42d661均master祖先 | blockers: 收敛为两件——1066/A＋981 PROOF批/整体交接；T-A03重冻裁定项不变 | next_responsible_role: 等两件A交付齐后T-A03运行完整A并集（不重复预跑）

stage: work | PLAN-985 | r27 | 2026-11-17 | outcome: pass（1066/A验收通过；非T-A03出口） | task_ids: T-A03(准备面，不勾) | evidence: §5"r27增量"——IF-1066-A-CORRECTNESS接收：链条收据核验（T-12[A] pass F-R1/R2修闭@2e606d28f→T-13[A] ff-only fold range-diff全等，master 7c131286d）＋本轮复验（四提交master祖先/切片路径零漂移diff空/落地检查`cargo test -p auto-lang --lib plan1066`=5 passed 1.57s含NC-A1负控）；981材料就绪盘点（PROOF-[A]落主393e0cc75/a181a1bb4、review-69 pass、tier-1终验收据61e52ea44）＋R69-2替代口径登记T-A03裁定 | dependency_revisions: lang master含7c131286d/942d57681/393e0cc75/a181a1bb4/b73a3fc70 | blockers: 仅剩981接收对账（985侧动作）——对账过则A外部输入全齐 | next_responsible_role: 981接收对账→T-A03重冻＋完整A并集→T-A04→T-A05（M2A-08）

stage: work | PLAN-985 | r28 | 2026-11-17 | outcome: pass（981接收对账通过；A-required外部输入全齐，等待点结束） | task_ids: T-A03(准备面，不勾) | evidence: §5"r28增量"——981 MIN-A-AGE整体交接收据（e8d637bb9，review-71 pass）绑定revision全落主核验（lang 0520c7f3b/6340934a1/ad7882cbe/230e4abc6、ui ab6fff5c/658ad515/1eb27697/9032d69c）；本轮复跑p981全集205/205＋PROOF A批4/4判别绿（g01 2D/g08 3D/坏编辑保旧恢复/组合出口预算）；r26反馈①–⑤全闭；R69-2替代口径裁定接受（具名拒＋零效果，逐项诊断B-W1） | dependency_revisions: lang/ui master含上述全部绑定 | blockers: 无外部A输入阻塞——剩余为985自身T-A03重冻裁定项落实＋完整A并集实跑 | next_responsible_role: 待用户实施指令后进入T-A03执行（重冻→V-A-EXIT→T-A04→T-A05/M2A-08）

stage: work | PLAN-985 | r29 | 2026-11-17 | outcome: pass（A批work收口：T-08[A]＋T-A03闭环；B批不动） | worktree: D:/autostack/.wt/lang-985/auto-lang @ plan-985-dev（基面master→resync 17674d957；A批提交 f93508199＋ecc282df1；树clean） | task_ids: T-08[A], T-A03 | evidence: §5"r29执行记录"——MIX-07两例（4.37s/5.63s≤10s门，含V-08三具名负控＋双车道取消）；装配道dep路由扩展（build_cargo_provider）；MIX-01..06 A腿聚合账；矩阵重冻＋账本词表user-deferred扩展＋p985_target_scope负控，ledger 8/8（TSV SHA=89eebafa…）；V-A-EXIT并集5腿（ledger 8/8、p981 205/205、abi f0 36/36、cover 2/2、固定组94/95）；M2A-01..07门级全闭 | dependency_revisions: r25–r28全部接收输入（见§5）；worktree=master+本批两提交 | blockers: 无——1项具名既有红（source_parent_oracle_failure_rejudges…，master复放同红，归批量回归triage，非985触面） | next_responsible_role: review（T-A04 A批独立phase复审，review_scope=phase，绑定r29/ecc282df1）

stage: goal | plan_id: PLAN-985 | plan_revision: r29 | round: 1 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-985/auto-lang | commit: ecc282df1 | next: review（T-A04 A批独立phase复审→T-A05 A批fold/M2A-08；本Plan为跨A/B，A批fold后保树续B，不归档）

stage: review | review_scope: phase（A批） | reviewer_identity: 独立子代理上下文（与实施会话分离，同树只读重建；本会话派发） | plan_id: PLAN-985 | plan_revision: r29 | outcome: pass | worktree: D:/autostack/.wt/lang-985/auto-lang | branch: plan-985-dev | reviewed_commit: ecc282df1 | base_commit: 17674d957（merge master） | dependency_revisions: r25–r28接收输入＋master归档族 | spec_inputs: 计划§5 r29/§8 A批行＋docs/specs/ace-migration/m2-stdlib-io.md@423ef4a8975c8a51 | acceptance_results: A diff越界 pass / B MIX-07质量 pass（oracle独立＋负控判别＋等值链路如实声明）/ C 定向复跑 pass（ledger 8/8、mix07 2/2[4.86s/6.18s]、p981 205/205）/ D 矩阵账本 pass（TSV=89eebafa…、锚哈希实测一致、守卫逻辑核） / E 裁定回链 pass（L-45=r26、S-25=route C、S-20/S-21 successor吻合、B-04/B-06=M2-TARGET-SCOPE）/ F 基线红归属 pass（abi.rs@631249d99=Plan967，本批未触）/ G M2A门级抽查 pass（04/05/06全master祖先）/ H 健康 pass（零新增warning、新文件零fmt漂移） | pending_closeout_gates: A批fold后冻结受影响模块并集固定组门（T-A05落地时以本批5腿身份复用；整包统一回归归B期T-04/最终merge） | findings: F1–F4均info级不阻塞（sqlite腿pending降级沿t15先例/基线warning计数口径/oracle等值链路已如实声明/conformance.rs基线fmt债务非本批） | evidence: 复审代理完整报告折入本收据；重腿（cover 2/2、固定组94/95）复用r29同树同提交证据并核身份 | next: merge（T-A05 A批phase fold：核owner合入SHA/落M2A-08收据与A阶段Spec/具名B-pending；保树续B不归档）

stage: goal | plan_id: PLAN-985 | plan_revision: r29 | round: 2 | from: review | to: merge | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-985/auto-lang | commit: ecc282df1 | next: merge（T-A05 A批fold→M2A-08；跨A/B计划A批fold后保树续B、不归档985）

stage: merge | review_scope: phase（A批fold，非整包归档） | plan_id: PLAN-985 | plan_revision: r29 | outcome: pass（M2A-08成立） | worktree: D:/autostack/.wt/lang-985/auto-lang @ plan-985-dev（保留续B） | delivery_commit: **95c7fbad8**（master tip，ff-only；=reviewed ecc282df1＋docs-only spec后代，实现零改动，range-diff豁免） | checkpoints: prepared=reviewed基线ecc282df1（T-A04 phase pass）＋A阶段spec delta（SD-PHASE/SD-01/SD-DETAIL/SD-A-FREEZE命名目标，as-built核对）；landed=master@95c7fbad8（分支领先8提交0落后，无merge commit，线性历史）；canonical_verified=docs/specs/ace-migration/m2-certification.md（新建，delivery内）；主检出落地检查 ledger 8/8＋MIX-07 2/2（5.51s/5.69s≤10s门）；archived=不适用（保树续B，985保持executing 6/14）；cleaned=不适用（worktree/plan-artifacts/registry保留至B期整包收口；批量回归%5触发记pending-at-full-closeout） | B-pending（具名）: 977-B、980-B批（B-AV/W1起/WEB/TAURI/RUST）、981-B（W1/W3/EVENT-B三件/D-981-PROOF-AV-SCENE等）、982-984-B、OS064-B波（A01-28/ASSEMBLY）、1066-B、1067、1068、986、987、1063（Linux） | evidence: §5 r29执行记录＋T-A04复审收据＋本收据；A批三提交 f93508199/ecc282df1/95c7fbad8 | next: B批解锁（T-02..07按§5核M2A-08收据后开工；985同树续行至B整包收口）

stage: goal | plan_id: PLAN-985 | plan_revision: r29 | round: 3 | from: merge | to: done | status: executing（跨A/B，B批续行） | outcome: pass（A批全链交付：work r29→review phase pass→fold落主95c7fbad8，M2A-08成立） | worktree: D:/autostack/.wt/lang-985/auto-lang（保留） | commit: 95c7fbad8 | next: done（/goal 完成计划985/A后续工作 目标达成；B批属后续会话/授权）

### 2026-11-17 B期等待期间中间产物清理记录（用户指令；沿981/984先例）

M2A-08落主后、B批开工前（B前置=多计划B批完成，等待期长），用户指令清理worktree构建缓存：删除 `target/`（6.2G）＋`experimental/ace/target/`（11G）＝**释放约17.2G**；worktree降至约532M，git状态clean，分支/树/矩阵/收据/plan-artifacts(17K，在途scope保护)全部保留。构建中间产物按2026-10-31裁定为可删缓存不回收暂存；B批会话冷重建（预期首次cargo构建时间变长，按环境归因记录）。

### 2026-11-19 台账维护：B期spec沉淀后15条引用重钉（worktree commit 13b5b3e91）

B期会话沉淀触发台账陈旧收据检测：`m2-scheduler.md`×4（L-29/L-31/L-43/S-19→`5bdd5aea28011cb4`，1067 SD-ASYNC）、`m2-schema.md`×2（L-44/S-23→`1ab3508b3edcd351`，981图形host port增量、核心章节未动）、`m2-tools.md`×9（S-24/A-04/A-07/A-08/C-06/D-05/D-07/F-07/F-10→`33b478d6d3907bb2`，取最终冻结提交字节，主检出工作区WIP不入钉）。逐行确认新版规范为超集沉淀、原义务支撑不变（serve/req/mcp等未交面按B边界具名）。复跑整组`p985_`：**8/8绿**（含缺行/重复/错source/pending虚报/词表/target-scope-deferred全部负控＋真实矩阵校验）。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r20：用户明确最终三端一致、AC后移B与AAC实证。新增1067/1068后继与SD/AC/V-PARITY-STAGED；原任务ID/勾选/4进度保留，新规则尚未执行。新增1068为最终B依赖，A不反等它；本轮无代码/测试/阶段签章。

2026-11-16 stage: new/revision | PLAN-985 r17 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

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

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

2026-11-16 stage:new/revision | PLAN-985 r18 | outcome:pass（规划合同预检；非产品review/阶段出口） | changed:前置1064/1065、原T-A03移交守恒、V-IO-HANDOFF/SD-IO-HANDOFF | next:原owner取得实施指令后按真实来源更新原矩阵/consumer；当前required库/AC/RG未齐，T-A03→A04→A05仍未完成。4/14与任务/AC勾选保持；不实施、不运行、不改canonical Spec，不新建Plan、不反阻973归档、不要求977 B反过来先闭A。

2026-11-16 stage:new/revision | PLAN-985 r19 | outcome:pass（规划合同静态预检），非产品review/阶段出口 | 原14任务/4勾选/状态保持，新增接收子行/V-MODULE-HANDOFF沿原AC/SD/T-A03执行；前置并集仍1064/1065等，不因切片反等整包/下游B。next=未来按正式来源重核矩阵/consumer及A完整出口；不实施/运行/改canonical Spec或新取号。

本轮规划预检：五份相关Plan原45任务ID/勾选及原AC/V/SD全部保留；新增9任务全部未执行（1064增6、975增3），当前合计54项。0–10章节、任务总数/唯一性、围栏/控制字符、本轮新增本地引用与行尾核对通过；1065表与真实SYS973_KINDS逐项相同=50，Windows W36/N14。B1→MODULE→975完整消费及973→1065→1064 AC验收的任务图无环，不存在1064/1065整包依赖975。原972归档、系统库canonical Spec与抽样产品源码hash未变；本轮未写取号计数器（其他会话1066取号与本任务分开）。973仅核原10项勾选/清理收据更正滞后计数并补前向交接，不重开终态。此为静态规划检查，所有新增运行验证仍未实施。

2026-11-16 stage:new/revision | PLAN-985 r21 | outcome:pass（规划合同接收一致，非实现/认证PASS） | changed:Node桌面A/B接收、AC-NODE-DESKTOP、V-NODE-DESKTOP-A/B、SD-NODE-DESKTOP、原T-A03/T-04 | preserved:executing/4-of-14及原ID/勾选/历史 | next:原owner重冻该部分接收矩阵后按正式来源推进；当前只规划，不运行/签A/B或改Spec。

本轮规划静态预检：9份计划/设计路线文档；四份Plan共87个原任务ID/总分母与依赖列表保持，仅980的NATIVE I/V/R因新增合同重开（15/29→12/29），历史收据未删。新增本地链接/锚点、0–10章节、验收/用例/原任务映射、表格与行尾检查通过；A入口→A出口→AC切片→B产品/终验的阶段图无环。候选固定组名按scripts/test-groups.toml只读核实，新UI/AC入口或case明确由work落实，均未运行。本记录是文档预检，不是产品或独立实现复审PASS；未改canonical Specs/源码/测试资产/取号计数器，未实施/建树/构建/运行/合入/清理/恢复目标。

2026-11-16 stage:new/revision | PLAN-985 r22 | outcome:pass（规划可执行合同，非实现/认证PASS） | 14原任务/4历史完成保持；T-A03负责原矩阵重冻与MIN-A/RG迁交，T-08[B]/T-02/03分波接收，T-04候选终验；新增验证均未执行 | next:由原owner依新阶段/真实来源接续；本轮只规划，不实施/建树/测试/合入/清理/恢复目标。

本轮规划静态预检：14份计划/设计/路线文档，8份Plan共230个当前任务；仅980新增B-AV-I/V/R三项并按完整W3范围重开原RUST三项，旧AXR0收据保留；981仅核正原SOURCE三项已交的滞后计数。全部原任务ID/其余勾选、各Plan状态与整包依赖列表保留；新增本地链接/锚点15处、章节/表格/元数据计数/验收任务映射及行尾检查通过，A最小→A出口→W1→W2独立交接→W3→OS→候选/安装/退役阶段图16节点18边无环。原GUI/Widgets/Notes-Clock/语言/库/产品义务有具名阶段与原owner，不以延期冒PASS。仅文档预检，未实施/建树/运行测试或窗口/签阶段/合入/清理/恢复目标，未写源码、测试资产、canonical Specs或取号计数器；他会话并发修改不由本轮回退。

2026-11-16 stage:new/revision | PLAN-985 r23 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | 冻结验收合同/162父义务分类索引/F01..35与1066片段；14任务/4历史完成不变；worktree矩阵/adapter仍待T-A03实施，当前未执行或签A | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。

2026-11-16 stage:new/revision | PLAN-985 r24 | outcome:pass（规划合同交接，非代码/产品/独立A验收PASS） | EVENT-SPLIT三缺口→原LANGUAGE/SESSION/AV/WEB/认证任务，A/B交付和逐批恢复边界明确；复用#6i与正式MIN-A来源，保全部原任务ID/勾选/状态/完成数/依赖/验收。新增AC-EVENT-PHASE与V-EVT正反控制均未实施。next:未来原owner按§5/§8在各自已授权范围执行，当前只规划，不建树/改代码/测试资产/运行/派agent/提交合入/清理/恢复目标。

本轮文档静态预检（非产品PASS）：8份规划/路线/债务文档；980/981/985原80个任务ID、勾选、32/34/14总数与12/3/4完成、状态及依赖并集保留，原AC/V/SD标识未删除。新增表格/引用/任务和阶段映射、Markdown围栏（含985原四反引号嵌套例）与空白通过；最终阶段图32节点37边无环。12份读取的代码/用例/规范内容hash未变，980并发round41收据完整保留；所报两红只完成归属规划、未修复/复测。新增V-EVT均未创建/运行，实际矩阵/SDK/目标状态及canonical Specs未改。next为原owner后续按现合同执行；当前只规划，不签代码/独立产品review或A出口。

## 10. 待澄清事项

Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
