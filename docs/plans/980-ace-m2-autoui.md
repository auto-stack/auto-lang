---
plan_id: PLAN-980
status: executing
feature_name: 现有AutoUI全模式ACE兼容接入
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 14
current_step: 12
total_steps: 32
depends_on: [PLAN-960, PLAN-963, PLAN-967, PLAN-968, PLAN-969, PLAN-972, PLAN-973, PLAN-976, PLAN-978, PLAN-979]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-autoui.md]
touched_goals: [GOAL-002]
---

# [PLAN-980] 现有AutoUI全模式ACE兼容接入

## 0. 变更摘要

2026-11-16 r13当前：M2-A由B-NATIVE-I/V/R交AV＋Rust FFI驱动Iced的可复用Node桌面入口；已交HELLO仅为有界真窗，A2R/AC分别取真实后端证据。M2-B AC桌面由既有1068/T-04承接，交981/FUSION/AUDIT和985/B；保留任务ID/历史收据，新范围NATIVE I/V/R重新打开（15/29→12/29）。本轮仅规划。

2026-11-16 r12最新阶段：AV/AC/A2R最终完整一致；A期AC基础以外差额可逐行交M2-B，1067/1068承接原生异步与三端补齐，A2C完整对齐增强。原任务/勾选/实际进度不变；新阶段条款不能冒实现或复审已过。

2026-11-16 r11（当前，用户范围裁定）：**JET/ARK 两后端归 M3（v0.7），从 v0.6 移除**——B-JET-I/V/R 与 B-ARK-I/V/R 六任务行撤出执行清单（原条目及 AC/V 留 §9 历史），total 35→29、进度口径 12/29；AC-B-JET/AC-B-ARK/V-B-JET/V-B-ARK 随批归 M3；PARITY 的 v0.6 联合覆盖改为现役模式（NATIVE/WEB/RUST/TAURI），JET/ARK 的模式覆盖由 M3 联合验收补。**WEB（Vue+TS）保留 v0.6**（前置仍为 L976）。本裁定同步 M2 路线与 m2-autoui 规范；此前 §9 round 17 的 JET 环境阻塞记录由本裁定取代（归 M3 后 v0.6 不再含该腿）。本包已交付各批（DIALECT/STATE/HELLO/RUST-AXR0）不受影响。

2026-11-16 r11（当前范围权威）：依用户本轮裁定及[Design55 §5.1](../design/55-ace-m2-complete-language-and-product-replacement.md#51-m2目标范围裁定godot后置ts按生产消费闭包2026-11-16当前优先)，本包WEB/TAURI/ARK等生产TS/JS能力仍required，真实消费976新受检provider；不保旧TypeScriptTrans fallback，不以Hello或少量Demo代完整现用UI合法表面。DIALECT已交证据与3/35保持，Godot不成为本包或979/981交接前置。 本次只修订规划，未实施/测试/合入或恢复目标；新增验收尚未执行。

2026-11-15 r10（当前交接合同）：落实979完整收口所需的IF-980-FIELDS，由原B-DIALECT-I/V/R交付WidgetDecl完整必需字段/构造清单、同Schema普通源投影/错误/覆盖反控与正式来源；仅消费979已交基础接口，不等待979归档或981整包。原960多模块集成由981/SOURCE承接，本包按实际接口消费。35任务、0/35、drafting、全部原AC/V及Hello与A/B门保持；新增交接不替完整DIALECT/STATE/PARITY或窗口验收。本次只规划，无实现/建树/运行/phase fold或恢复授权。

2026-11-13 r9（本轮当前合同）：HELLO直接消费AGE Node前门并复用AutoUI组件；旧widget原源通过共同Schema兼容投影继续验收，规范作者形式为node；不先实施独立传统UI接入线。 用户仅授权规划修订；保原任务ID/数量、既有AC/V与进度、完整分母及阶段门，不建树、不写实现/fixture、不构建/运行/合入。此前§0修订描述为历史，冲突以本次§5–8为准。

2026-11-12 r8：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-11 r7当前：增加HELLO首个真实窗口I/V/R/D先行批：消费967/F0、979/H0、969/E0、978/B0最小正式接口，从单text静态及Auto函数文本到AV真FFI先交、原RUST批同源新A2R并行；不等待STATE全量async、973 File、WEB/TS或完整PARITY，保原九批全传统UI义务。 原31项任务/全部AC/V保留，新增4项早期切片任务，现35项、drafting、0完成；只规划。

历史修订：2026-11-10 r6执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r5当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r4：传统AutoUI完整支持是合并AGE M2必需门，不能以2D/3D场景、少量控件或新Node语法替代；保9批31任务，补AC-07及完整分母/纯传统UI的AV+A2R验收，drafting、0/31，仅规划。


建立旧widget/model/msg/on/store/props/computed/theme/actions/routes/setup/watch等到Schema领域IR映射，消除handler合成旧bytecode。保持现有Vue/VM-Iced/Rust/Tauri/jet/ark的真实支持格；TS-only行为不默认为全端已有支持。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

2026-11-16 r14当前：A只交MIN-A真实基础，完整UI/Widgets按W1 AV→W2 AC→W3 A2R转B，再做OS全量。复用现有Plan，新增B-AV-I/V/R三任务；原29→32任务。已交AXR0保历史，原B-RUST三项因新增完整W3范围重开，12→9完成，不回滚代码。旧A全UI/全A2R口径由§5本轮合同取代。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
传统widget/store工程可保原源码经AGE Schema的显式兼容配置直接进入ACE；规范作者根为node，同一Node语义复用既有组件/布局/主题/状态/事件接面，不另建传统widget接入线。传统UI完整迁移归本包，完整Node语言/新旧组合归981/LANGUAGE/FUSION，二者均为AGE M2必需门，不另立重复接入计划。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

正式依赖以frontmatter并集为准；各批最小接口前置见§5，稳定接口只允许受控准备，最终须正式交付来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11本轮授权：用户要求规划首个ACE UI Hello及AV/Rust FFI路线、核对既有M2冲突并作必要重排；仅规划/设计/导航，不实施、创建worktree或探针/fixture/SDK、构建/运行、提交或发布，未指定模型/工期。沿旧语言用例守恒、新Auto标准库与AutoABI/FFI、Demo递进的三线，不削完整M2范围。

现场语言HEAD 9e36d984e、UI HEAD d88e9c51。已核av-core/src/host.rs仅console/1（sha256前16 30d6dcc475850cd0）、a2x-rust/src/provider.rs仍拒source foreign（3ea453dc2452576f）、core/src/source/cst/frontdoor.rs（dd3d93d06799ae52）；现有auto-bindgen/src/extractor.rs是预置C manifest表（8c7f1295cc83c6fb），不能称Rust元数据导入器已实现。Iced由Rust宿主编译执行，AutoVM不解释其Rust源码；现有DynamicComponent/VmBridge/handler_codegen深依赖旧执行链。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

本轮授权细化批次和跨仓依赖、必要时拆计划，先不施工。已核各仓Cargo、源码和测试入口：UI/Man根无AGENTS；Web守中立contract/公开SDK方向；OS域正文按当地AGENTS归OS064；没有主检出实现/测试授权。
2026-11-09用户进一步要求M2支持传统AutoUI及完整AGE语法。本轮仅修订979/980/981和路线，不实施/构建/运行或创建新Plan/实施树；980原模式/组件/兼容义务保留，不能以后继981替本包交付。本次语言HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；下文11-06来源为历史。


用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `D:/autostack/auto-ui/crates/auto-ui-dialect/src/ast.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/handler_codegen.rs`（本轮确认存在；开工重核符号/版本）。
- 本次读auto-ui `docs/specs/ui/structure-closure.md`、`runtime/component-and-store-instances.md`和`WidgetDecl`实际字段（含lifecycle/design/style/ext_imports等）；完整分母不能只取首段关键词。共同作者方向见auto-ui `docs/design/autographics/{declarative-authoring-model,multi-presentation-selection}.md`；不把设计草图当已实现。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：完整AutoUI声明/状态/所有运行模式接入。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs)：完整WidgetDecl，不另定义小组件全集；读取SHA256前16 `1d10f5ff2566422e`

- [auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs](../../../auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs)：替旧执行依赖，中立组件接面；读取SHA256前16 `9b6f212b63a7747c`

- [auto-ui/crates/auto-ui-runtime/src/ui/iced/renderer/projection.rs](../../../auto-ui/crates/auto-ui-runtime/src/ui/iced/renderer/projection.rs)：复用Iced呈现，Rust不解释guest逻辑；读取SHA256前16 `dcd9c869ff3873ce`

2026-11-13本轮授权/取材：用户明确要求修订Node首Hello、Schema解析校验与位置/AGE上下文名称、widget兼容和963并行顺序，仅规划。读取基面`dcf0535d4d15c00628a60f1940c6c55a01a101ab`；当前Spec仍描述有限统一CST与旧UI parser/有限Node pilot，并未宣称本次机制已交；差额是本Plan目标，未来按SD沉淀，不改canonical Spec伪造现状。来源：`docs/specs/auto-syntax/{source-frontdoor,grammar-contract}.md`；`docs/specs/auto-ui-dialect/design/full-grammar.md`；`experimental/ace/crates/syntax-runtime/src/{program.rs,parse/mod.rs}`；`experimental/ace/crates/core/src/source/cst/frontdoor.rs`；`auto-ui/docs/specs/auto-graphics/{node-basic-ui,ace-migration-feasibility}.md`。963执行进度与其他会话WIP只读保留。

### 2026-11-15 本轮交接修订取材与授权

用户明确要求修订979/980/981及相关路线，复用现有计划承接原960多模块缺口、消除循环、保留979完整验收；**只规划**。本轮不创建计划号/工作树/agent/chat，不实施、探针、fixture、编译/测试/GPU、提交/合入/发布，不恢复979目标或AG029暂停。读取语言基面`86c7fd50bc6dac5ff6d2a5c6215ad01fbfefc1f7`及当前文档WIP；保其既有改动、任务ID/勾选/计数与历史收据。已归档960只读。

已核规范/代码SHA256前16：`docs/specs/ace-migration/m2-modules.md`=`be25b0c69f2f8cc0`、`m2-schema.md`=`5797e15925b07e14`；`experimental/ace/crates/core/src/source/packages/binding.rs`=`afa4f886dfc2fab9`、`source/domain.rs`=`1613b66c2383bc40`、`source/age_component.rs`=`58081b0e1eb6347a`、`source/cst/frontdoor.rs`=`17016eecbc0fbd61`；`experimental/ace/crates/av/src/age_component.rs`=`55b84b8f1ca9d5a8`；auto-ui `docs/specs/auto-graphics/node-basic-ui.md`=`0a3e8616b8d2239b`、`crates/auto-ui-dialect/src/ast.rs`=`1d10f5ff2566422e`。这是读取身份，不是本轮运行PASS。960的普通模块与979的单模块领域接口已分别存在，跨模块AGE组合/字段适配/实例引用与原源CPU交接仍是拟实现；§5拟类型/样例不是现存API。现行Spec保已验事实，仅在本包规范增量表登记未来更新，不改canonical Specs伪造支持。

### 2026-11-16 用户目标范围修订与取材

用户明确要求从ACE M2去掉Godot转译，并核实a2vue对TS的真实依赖；本轮仅规划，不删除实现/原源/旧用例，不运行测试/安装Godot或创建/恢复实施树/目标，不改canonical Specs或归档959。读取基面`8a686e8d2399f01c3aa91112aa683f141c0a0922`及并行文档WIP。auto-ui代码核实：`ui_gen/ts_adapter.rs` SHA256前16 `84efe1eb8306dc25`（Stmt回退约858行、Expr委托约2458行）、`ui_gen/vue/script.rs`=`8120f939878ed68d`（handler约288/1732行及computed/body约2596行）、`ui_gen/ark_adapter.rs`=`038454e31c87e786`（同类TS委托）；现行`docs/specs/ui/vue-store-generation.md`=`f422fe6df24cc398`。这是静态读取，不是新ACE TS运行证据。

Godot排除粒度是目标/consumer腿；同一旧源的普通Auto语义、AV/A2R/其他required后端义务不能随Godot一起撤掉。规范中旧“全部在用A2X目标”的历史范围由本次用户裁定覆盖，未来实际交付时落对应SD；当前Spec已验事实不伪造。受修改的旧revision范围/矩阵证明仅保历史，未来独立review须绑定当前revision与真正实现提交。

本轮授权与取材（2026-11-16）：用户明确要求修订980/NATIVE、981及相关路线，仅规划，不实施/建树/运行/合入/恢复目标，不新增Plan号。读语言主线1d2f01238e787f964c2293762f02a7969a907d29、UI主线34bb4326dd83e038902c15a957c8934aa39872dc；现行`docs/specs/ace-migration/m2-autoui.md` §7/8与UI `src/ui/ace_hello.rs::{HelloSession::run_f0,AceHelloWindow::view}`、`examples/ace_hello_window.rs`表明F0是字节验证、窗口由Rust组件绘制，AXR0为有界A2R证明。以上是已观察边界；下面新增接面/用例仍待原任务实施。

本轮用户授权：仅修订现有980、981、1068、OS064、985–987及相关路线，不新立项、不启动实施。采用A最小真实交付→985/A M2A-08→B完整UI/Widgets的AV→AC→A2R三波→AutoOS全量迁移→候选/安装/退役。读取语言主线aa257fa1425c31fab48bd544cbd116b2fdce7dcd及现有Specs/已交HELLO、AXR0、SOURCE；UI示例真实源在auto-ui/examples/ui，Widgets源在auto-os/widgets-gallery/src/front/{pages,components}，ui-gallery副本需和规范源去重。新增接面/用例/收据仅是合同，不代表已运行。语言/标准库/非图形FFI的既定基础与测试守恒仍保留，不能借UI延期把普通语言必需能力全部后移。

## 5. 详细设计

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

### M2-TARGET-SCOPE-20261116：本包当前边界

本包WEB/TAURI/ARK等生产TS/JS能力仍required，真实消费976新受检provider；不保旧TypeScriptTrans fallback，不以Hello或少量Demo代完整现用UI合法表面。DIALECT已交证据与3/35保持，Godot不成为本包或979/981交接前置。 当前目标分类/旧义务去向/发行与退役以Design55 §5.1及本节为准；后文旧“全部目标/不缩分母”仅约束修订后仍required的义务，不重新加入用户明确后置的Godot。归档基线不重写，Godot不记PASS或无理由NA，生产TS不得错误排除。

### M2阶段执行合同（本轮当前）

本包跨A/B。A=MIN-A-UI、必要DIALECT/STATE基础与HELLO正式来源；B完整AV/UI、A2R及原WEB/TAURI/PARITY。每个任务仅在§8登记一次，A部分可独立phase review/fold供985，整包review/归档/清树留B。原唯一tree保留/resync，不为各波复制权威树。

| 阶段 | 原/新增任务归属 |
|---|---|
| A准备/基础 | T-01、T-HELLO-I/V/R/D、DIALECT/STATE已交基础、B-NATIVE-I/V/R（只MIN-A-UI） |
| B-W1 AV | B-AV-I/V/R，消费已交A；DIALECT/STATE新增覆盖由此原owner扩展；PARITY保波次分账 |
| B-W2 AC | 1068/T-04主责，本包只交同版领域/宿主接口，不新增并行AC实现任务 |
| B-W3 A2R | B-RUST-I/V/R，完整同源示例/组件生成编译运行，AXR0证据仅按原边界复用 |
| B其他现役模式与最终 | B-WEB-I/V/R、B-TAURI-I/V/R按自身前置并行，B-PARITY-I/V/R在各波收分账后最终汇总，T-05/06/07最后 |
| M3 | JET/ARK仍归M3，不恢复为A/W1前置；Godot仍移出M2 |

| 批次 | 交付 | 最小前置 | owner |
|---|---|---|---|
| 980/HELLO | 单text普通源的AV→Rust FFI→真实Iced；同源A2R由RUST批并行 | 967/F0-D、979/H0-D、969/E0-D、978/B0-D正式来源；不等完整STATE/973/PARITY | auto-ui主责，语言守公共能力 |
| 980/DIALECT | AGE Schema下的AutoUI声明兼容/组件投影；先交IF-980-FIELDS | L979已正式交付的通用包/投影/覆盖基础 + L960/L963；多模块实际接线再消费981/IF-981-MODULE，不等979归档 | auto-ui领域主责，通用代码由auto-lang owner按本任务维护 |
| 980/STATE | 实例Store handler与异步状态 | 980/DIALECT; L967/L968/L969 | auto-ui |
| 980/NATIVE | MIN-A-UI可复用AV入口/typed事件/生命周期 | 已交DIALECT/STATE/HELLO及967/969/978、所用库精确接口；不等完整981/976/PARITY | auto-ui |
| 980/WEB | Vue与TS/JS表达式后端 | 980/DIALECT; L976/L972/L978 | auto-ui |
| 980/RUST | W3完整A2R示例/组件与工程；已交AXR0保留 | UI-W2-AC及同版AV源码/ABI、978/967所用正式来源 | auto-ui＋新A2R owner |
| 980/TAURI | Tauri桌面工程 | 980/WEB; L978 | auto-ui |
| 980/JET | **归 M3（v0.7）**——v0.6 移除（r11 用户裁定 2026-11-16） | —（原 980/DIALECT; L978，M3 再启） | auto-ui→M3 |
| 980/ARK | **归 M3（v0.7）**——v0.6 移除（r11 用户裁定 2026-11-16） | —（原 980/DIALECT; L976/L978，M3 再启） | auto-ui→M3 |
| 980/PARITY | B分波收据＋最终全部现役模式/组件覆盖 | 波次收据分别独立产生；最终收W1/W2/W3和WEB/TAURI，不反置为A/W1/W2前置 | auto-ui |

### r10：向979交付完整字段事实，消费基础接口而非最终归档

**原任务承接。** B-DIALECT-I/V/R产出**IF-980-FIELDS**，对接[979 r11 §5](archive/979-ace-m2-schema.md)的V-11、AC-08/AC-HANDOFF。T-01核输入与未决，B-STATE原任务继续承担组件状态/事件实际产品行为，PARITY继续核全模式；字段子面可以先独立复核并phase fold，不提前勾整个DIALECT任务或归档980，不新增任务/计划/第二份分母。

交付按现行`auto-ui-dialect/src/ast.rs::WidgetDecl`及其引用类型、组件/Store/主题/路由Spec和既有grammar逐字段建表，不能只抄本文列举项。每行含稳定字段/构造ID、Spec/原case与source hash、Node/widget原span、允许位置/类型/模式、是否必需、候选/投影位置、共享verify准入、实际consumer及未闭产品格；条件适用也须给可核理由，不能以“不在Hello中”删行。UI端清单与981的AGE构造清单分owner并集交979；未知构造必须显式拒绝，不能静默略过。

**数据流/类型。** 输入完整WidgetDecl语义与普通作者源 → 同一AGE Schema/legacy-widget配置 → 979 `DomainCandidate`/共享检查 → 按字段ID索引的覆盖结果。原`mapping_complete`/`component_coverage`是已交通用机制，其当前role粒度未必足够字段级；B-DIALECT-I负责在原共同机制上补必要的字段适配与完整性校验，不能交一份无法执行的CSV。拟语义字段`RequiredFieldRow { id, origin, source_span, parent_role, type_rule, applicability, projection_path, check_id, consumer_owner }`和`FieldCoverage { required, present, missing }`由原I冻结/等价复用；这是设计草图，不声称现有类型已具备这些字段。Core代码保通用数据/检查，无UI crate反向依赖，无第二parser/类型检查器。

**关闭条件。** 复用002-counter完整原源/普通Node迁移副本及现行字段用例，以各自原源/span验证完整投影；themes/model/design等原业务不得裁掉。对必需字段的合法源能投影，对未知字段/坏类型具名拒绝；在实际必需清单对应的映射实现中删除一项时校验红、恢复后绿，删除清单行本身也由原AST/Spec分母对账抓住。CPU证明不要求先完成所有窗口模式，但不替代B-STATE/NATIVE/RUST/PARITY既有真实UI义务。缺能力记到本包原I任务及实际通用owner，不改NA。

**正式交接。** B-DIALECT-I/V完成上述子面 → 原R任务同树独立核此子面 → 按原phase fold规约正式落主，记录repo/SHA、r10、源/清单/Schema/compat/ABI及真实永久case/命令/结果和剩余完整任务，交979 T-08/T-04。r9或WIP证据不自动证明新增合同。此交付不依赖979 T-04/T-05或完整981；多模块产品接线依981/SOURCE的IF-981-MODULE，字段定义/覆盖子面可先按独立模块验证。后续完整能力保留原最小前置，不从此子面推出全包可整段归档。

阶段注记：以下“完整传统UI”、各模式和旧例覆盖均为最终B义务；A只取MIN-A-UI清单。不能从这些最终完整性说明反推A须完成所有示例或A2R。

### 传统AutoUI完整性与AGE M2硬门

T-01及DIALECT/PARITY建立逐项分母：现行WidgetDecl、Store/主题/样式/组件语法和普通Auto表达式→Spec/原源位置→模式/平台适用格→ACE producer/runtime/实际oracle。至少核model/msg/on、props/emit/bind/computed、use/多文件、named_views/slot、keyed条件/列表挂载的既有语义、routes/setup/watch/expose/actions/menubar/toolbar/timer、lifecycle/design/style/theme/ext_imports及全部现用组件/布局/资源/错态；再与真实grammar/AST/Specs双向对账，未被Demo覆盖的必需语法/变体补永久case。可解析或首帧可见不等于实际运行支持。

纯传统UI工程（无view-2d/view-3d声明）必须经共同前门实际AV运行，并经新A2R生成/编译执行UI逻辑。保原文字选择/复制、输入/IME、焦点/捕获、主题/布局、表单/列表/滚动、路由、事件、异步/卸载/late、App Store隔离、资源/关闭恢复。NATIVE负责AV开发、普通body编辑不调用Cargo/rustc；RUST负责生成状态/handler/路由代码的实际dispatch，不用Rust宿主内嵌AutoVM/AV冒A2R。原widget源码不强制重写；由979同一Schema的显式兼容别名直接准入，再与规范Node源共用投影/verify/组件。别名不等旧Parser或旧脚本生成，迁移副本另计为“声明头改名兼容”。

WEB/TAURI/JET/ARK保各自现用模式的全部实际支持格，不把TS-only或未承诺平台能力默称所有后端齐；必需平台缺环境保持pending，不删格。DIALECT/STATE交981版本化组件/状态/事件/生命周期接口；980不依赖981完整Node或图形增强终验，981消费本包，防PARITY↔LANGUAGE环。AGE M2须同时通过传统UI门和完整Node/融合门，图形小例不能抵消缺口。

### 980/DIALECT：共同AGE Schema下的AutoUI声明兼容
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-dialect/src/ast.rs`；`D:/autostack/auto-ui/crates/auto-ui-compiler/tests/ui_domain_validation.rs`。
I实现：按WidgetDecl全部字段逐项投影/明确模式适用性，schema→typed领域IR→ACE行为；字段覆盖表包含named_views/bind/routes/setup/watch/expose/actions/timer/theme。
V验收：普通标识符上下文保持；旧widget原源经同一AGE Schema兼容别名的解析/绑定/错误span正确，规范Node保相同body/组件行为；坏字段拒绝，缺字段不能悄悄忽略。
R关闭：独立核当前阶段的真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/STATE：实例Store handler与异步状态
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs`；`D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/handler_codegen.rs`；`D:/autostack/auto-ui/crates/auto-ui-runtime/tests/plan041_instance_ownership.rs`。
I实现：用ACE执行服务替旧handler bytecode生成/调用，接组件props/emit/computed与App store、挂起/唤醒、卸载和late generation。
V验收：双App各两组件不串；异步任务返回只写仍存活实例；MIX-03无HTTP版本用真实异步host，HTTP实腿由983/联验补齐。
R关闭：独立核当前阶段的真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

r13进度说明：并发原owner刚交r11 NATIVE（ui 34bb4326/lang 32e8e951c，原15/29）；其ace_native.rs由Rust组件消费AV文本/属性/重建状态，原成果及§9收据完整保留。新增“通用项目＋实际UI-FFI效果与事件”尚无证据，B-NATIVE-I/V/R重新打开，current_step 15→12、总任务29不变。原验收不是新增r13要求的PASS，T-05必须核新范围后再允许收口；不回滚任何代码。

本轮进度：原r13 NATIVE三项仍待交MIN-A-UI；下列通用ABI/owned/队列规则保留。001/002完整主题/选择/全部布局及媒体/terminal等完整产品格正式交B-AV与PARITY，不再要求NATIVE在A关闭这些格。

### 980/NATIVE：MIN-A-UI可复用入口（完整覆盖交B-AV）

既有改面：auto-ui `crates/auto-ui-tools/src/run_vm.rs`、runtime `src/ui/{ace_hello,vm_bridge,handler_codegen}.rs`、`src/ui/iced/renderer/{host,projection}.rs`及`tests/{desktop_behavior,image_surface_contract,ace_ui_hello,plan041_instance_ownership}.rs`；语言967/969现有ABI/foreign调用owner。新一般化adapter/永久case由B-NATIVE-I在这些owner内建立，不复制parser、控件库或Hello专用renderer。

**B-NATIVE-I的MIN-A有界决定与实现**：先用已有`ace-hello-minimal`双源及002-counter的Node声明头迁移副本跑通最长链、一个非法近邻、失败后恢复；已有run_vm的`AUTO_UI_ENGINE=ace`分支及`run_ace_native_entry`可复用；核其真实runner/帮助和项目配置，冻结显式ACE AV选择与公开命令，不把拟定`--engine ace`当已有参数。复用既有AutoUI项目入口，使用户换路径/Node名/函数/文本/合法组件即可运行，不要求手写Rust壳。旧`-r vm`若保留兼容名须记录其实际ACE dispatch；默认切换仍按原产品合同，不由本批擅改。

实际数据流为`原Node/普通模块→活动AGE Schema/CST/唯一verify→AV业务和绑定求值→967自动binding/shim受检UI调用→UI-owned更新/事件队列→现有Iced投影/主线程窗口`。允许typed领域表示/帧更新为接口，**FFI实际承载UI创建/提交/更新/关闭中的必要效果，事件经typed队列回AV**；不能继续用独立`text_byte_len`调用充UI dispatch，也不能由Rust固定读取AV快照画特定Hello。宿主只实现通用控件/布局/设备服务，计数/绑定/业务保在Auto。

接口逻辑冻结为`open/apply/poll_event/close`四种职责，具体已有函数复用与符号/布局由I的全链决定写回；`UiSessionHandle`、`UiUpdate`、`UiEvent`为**拟定逻辑类型**，不是已存在的ABI。session句柄含身份/代际和授权；更新绑定schema/组件/属性/handler身份及typed值，事件含目标实例/代际/事件ID/受检payload。复用967 typed ABI、自动binding/shim、现有owned字节/record/队列，禁止直接跨ABI暴露`iced::Element`、Rust trait/闭包/String/借用私有布局或第二套字符串JSON重解析。UI操作权限与ABI/类型在调用前校验；布局/窗口失败与guest错误分态并保原源位置。

借用文本/更新只在调用期有效，provider返回前复制/促升到UI session所有；Iced对象仅UI主线程访问。事件从宿主队列在受控AV调用点分派，不从OS回调线程重入AV，handler按实例与generation绑定。关闭/取消拒新请求、失效句柄具名拒绝、late callback不复活；在途旧代pin按最后owner释放，坏更新不替已发布代。先冻结预算/容量/错误分类，超限不静默截断；沿970/STATE现有更新规则，不承诺任意状态迁移。

**正式交接IF-980-NODE-DESKTOP-A**（B-NATIVE-I/V→独立R→owner phase fold，原最终任务不提前勾）：含真实CLI/项目配置、源码/Schema/模块闭包hash、AV/ABI/provider/宿主版本及目标、typed命令/事件/错误/权限/owner表、001/002旧例映射、真实窗口/FFI效果/事件/恢复用例及B后延AC行。981按此复用，1068按同版ABI接AC，不回头要求980实现AC才可归档。打包中源＋AV运行时与AC/A2R二进制明确分型；A入口交978所需工程/资源依赖清单，986仍负安装升级，完整产品包不能用验收exe冒充。

**V/R**：A只须MIN-A-UI与V-UI-A-MIN；V-NODE-DESKTOP-*、原V-B-NATIVE/V-UI-FULL的完整产品格由W1 B-AV-I/V/R及最终PARITY关闭；保001选择复制/主题、页面/输入法/捕获、媒体/terminal、关开及资源回收原义务；可先正式交一般化入口接口，不据此提前勾完整NATIVE。R从实际输入追到FFI副作用、主线程窗口和AV事件，不以帧截图或byte_len代理。新增证据均未执行。

### 980/WEB：Vue与TS/JS表达式后端
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/run_vue.rs`；`D:/autostack/auto-ui/crates/auto-ui-tools/src/vue.rs`；`D:/autostack/auto-ui/crates/auto-ui-compiler/src/ui_gen/ts_adapter.rs`。
I实现：Vue生成使用新TS/JS provider，保watch/setup/expose/ext_imports/style及绑定；服务端使用已交宿主接口或隔离真实fixture，不等整个983才能证UI表达式。
V验收：真实浏览器挂载/事件/绑定与源expected一致；setup只一次、watch卸载停止；生产表达式不走旧TypeScriptTrans。
R关闭：独立核当前阶段的真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/RUST：Rust UI静态工程
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/rust_ui.rs`。
I实现：先按§5 AXR0切片运行同源单text V-UI-AXR0并独立复核/正式交接；随后多文件store/组件与sidecar经新A2R生成，保持原Rust UI行为和增量产物；旧AST emitter无回退。
V验收：未改生成物冷编译运行，props/msg/route与原独立oracle相符；坏源保旧工程。
R关闭：独立核当前阶段的真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/TAURI：Tauri桌面工程
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/tauri.rs`。
I实现：接新TS/JS与Rust backend生成，保持前后端invoke/事件/权限边界；旧AST emitter无回退。
V验收：真实Tauri窗的invoke和事件、失败诊断及重开，不能仅浏览器前端通过。
R关闭：独立核当前阶段的真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/JET：Jet目标工程
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/jet.rs`。
I实现：保持现有Jet生成模式，以受检UI领域IR供目标adapter；不新增原未承诺能力；旧AST emitter无回退。
V验收：原支持工程在实际Jet工具链构建运行，事件/状态独立oracle，缺平台未执行。
R关闭：独立核当前阶段的真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/ARK：Ark目标工程
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/ark.rs`。
I实现：接Ark领域投影和表达式adapter，保现有Ark工程/资源/模块输出；旧AST emitter无回退。
V验收：原支持工程在真实Ark工具链执行，原widget/事件/状态正确，缺平台未执行。
R关闭：独立核当前阶段的真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/PARITY：主题控件Demo及模式联合覆盖
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-runtime/tests/gallery_golden.rs`；`D:/autostack/auto-ui/crates/auto-ui-runtime/tests/theme_scope_isolation.rs`；`D:/autostack/auto-ui/docs/specs/ui/examples-ownership.md`。
I实现：从001开始按文字布局→表单列表滚动→菜单Blueprint→媒体编辑分小组；同一覆盖表列全部现用组件/变体/模式，未被demo覆盖的错态补永久case。
V验收：主题scope与原画面/交互/生命周期对照，UI058等AGE新能力不是本包替代；每格source/backend/runtime/行为收据齐。
R关闭：独立核当前阶段的真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

各批I的有界决定门：用该批最小真实正例、同类型非法近邻、一次失败后恢复跑最长producer→consumer链；仅对未决策略比较适用实现再冻结接口，不拿日历/语法深度缩范围。资源由session持候选、generation持已发布产物、在途pin持旧代，last-owner释放；错误/取消/late不得覆盖CURRENT。继承原预算/平台条件，新增资源单位/上限/超限分类在本批I冻结并有边界反例，改变承诺先修合同。

### HELLO：首个ACE AGE Node文字窗口（当前最小入口）

HELLO已交历史切片与r13一般化NATIVE的边界以NODE-DESKTOP表为准；下文保留原最小合同，不作通用UI-FFI已交宣称。HELLO归M2-A必要能力验证，不是B全产品迁移；不用等待M2A-08、981或AGE M2总签。原九个完整能力批次继续，新增HELLO使本包十批、35任务。最小正式来源：967/F0-D（AV/Rust受检互操作）、979/H0-D（AGE Schema驱动的最小node/view/text、位置名称及普通函数表达式受检投影）、969/E0-D（公开ACE AV同步run/授权/关闭）、978/B0-D（最小宿主/adapter构建闭包）。任一未落主只可准备，不以分支tip作产品证据。

首源拟新增在auto-ui既有examples/ui下独立`ace-hello-minimal/src/front/app.at`及必要显式项目配置（T-HELLO-I创建）。第一输入为一个AGE node根、一个view、一个text字面量；第二输入仍一个text，内容来自普通Auto函数返回的中文/ASCII字符串。任意合法Node/函数名及改文字都须工作，不硬编码Hello。该验证工程不修改/精简001；原001含themes包/model/design/col/selectable，后续按DIALECT/STATE/NATIVE/PARITY完整迁移。新增fixture不自动进入旧全部Demo分母。

路径为AGE Node普通源→已激活Schema驱动解析/结构校验→H0 typed投影→唯一verify→AV通过E0执行文本表达式和F0受检调用→UI拥有的Rust text provider→现有Iced文字投影/主线程窗口。静态描述可直接供宿主布局，但HELLO-V还必须观察AV真实F0 dispatch与同一文本字节被provider接收；只把描述交Rust硬画文字不能关闭AV/FFI门。UI公共Auto facade声明/foreign要求归领域包，967生成/验证通用binding/shim；宿主Rust内部调用Iced合法，不把iced::Element/Rust trait/String私有布局塞进AV值。

UI text provider在返回前复制借用文本到当前UI session所有的Rust值，再在OS主线程构造/刷新Iced文字；它不能持有已失效AV视图。字体/窗口/布局由Rust宿主持有，guest尚无事件回调/状态模型；关闭窗口/取消释放当前session，旧请求/撤权不得写新窗口。F0输入上限/错误规则沿967，程序与宿主失败分态；不扩guest异步、跨线程callback或Rust泛型语义。

已交同源A2R首窗属于原B-RUST-I/V/R的AXR0历史切片；新增完整UI A2R按W3执行。原AXR0先行切片，另消费967/T-12与978/T-03/09的AXR0正式来源；同一作者源还须经新A2R生成Rust、实际编译、链接同一逻辑UI provider并真实显示；若采用直接crate adapter须证明与F0 ABI adapter等价的字节/错误/owner规则。guest业务不能由AV/旧VM解释或Rust手写替代。HELLO-I/V/R/D只闭AV首窗全部V-HELLO-*即可交接，AXR0的V-UI-AXR0结果由B-RUST复核/phase fold另交；AV不反等A2R。完整980仍须原AV/A2R与全部模式，单text不证明按钮/组件状态、主题/选择复制/全UI语法已支持。

既有owner改面：auto-ui `crates/auto-ui-tools/src/run_vm.rs`、runtime `src/dynamic_build.rs`、`src/ui/{dynamic,vm_bridge,handler_codegen}.rs`与`src/ui/iced/renderer/{host,projection}.rs`；ACE分流由E0公开入口，旧组件支路不能被新候选隐式调用。新最小ACE组件adapter/consumer测试在原runtime/tests owner创建，拟`tests/ace_ui_hello.rs`；公共运行入口参数由I冻结，不虚构当前有--engine ace。仅窗口host可继续依赖Iced，ACE core/AV不得反依UI crate。 T-HELLO-I先核现有run_dynamic_iced的DynamicComponent具体类型及其VmBridge字段耦合，用至多两种机制比较共同Component/视图消费接口与最小ACE组件adapter；必须复用现有Iced投影/窗口服务，必要时抽出中立宿主接面，不能创建假旧VmBridge或另写一次性Iced Hello绕开AutoUI链。该决定/最小正例/同类非法近邻/失败后恢复先闭再扩实现；无法隔离旧执行依赖则具名needs_replan。旧代码是否仍被其他产品编译与Hello新链是否dispatch旧VM分开记录，全面不可达/删除仍由987终验。

本批交接顺序：最小依赖各自D→HELLO-I/V→独立R→正式D，原完整DIALECT/STATE/NATIVE/RUST等复用其代码/用例继续扩大；不重复第二UI引擎。纯文字H0/F0基础与完整语言/新库路线可以并行，共享grammar/ABI文件短窗单写。

Demo后续序列：静态text→函数/状态文本→单按钮计数→双组件隔离→原001主题/文字交互完整保真→其余原Demo按依赖逐项扩展→完整Node/2D/3D。编号不是能力依赖；每个实际Demo列需要的语言/库/宿主格。Demo未覆盖语法/错态/模式补永久case；Hello过不勾原九批或完整RG-A03。

早期切片是本Plan M2-A内部交接，不新增ACE里程碑或Plan终态；只在同一权威树按owner流程独立复核、正式落主并记source/接口/grammar/ABI/provider及真实结果后供下游消费。同Plan尚有原完整义务时不归档/删树/勾整包完成。frontmatter保最终依赖并集，早期切片只等本节最小接口；同一文件单写，不以分支reviewed/WIP当交付。最小切片通过不关闭原AC、完整Rust FFI/全UI/RG-A03/M2A-08；不新增对980/981/985整包或AGE联合门的反向依赖。

### Node首切片与旧用例兼容（2026-11-13，当前）

HELLO的正式输入直接为`node Hello { view { text "Hello" {} } }`及普通Auto函数返回文本的同形源；不先编译widget再转换Node，不创建第二传统UI runner。直接消费979/H0的AGE Schema、NodeName槽/父规则与源位置合同；领域名不得进通用lexer保留字表。H0对model/on完整执行未交不阻单text，HELLO也不能据名字已识别宣称状态/事件可用。

兼容验收分为①原widget字节+显式AGE legacy-widget Schema别名；②只改声明头的规范Node迁移副本。二者和新Node作者源共同复用AutoUI组件、同一领域投影/verify与ACE执行服务；旧store、主题、路由等按原业务语义迁移，不做全文件widget/model/view/on替换。开启别名时必须真读原源且保原span；关闭时旧声明头具名拒绝、Node仍工作。旧parser/handler_codegen只作调查输入，不能运行时回落。根改名后的case仅证明迁移兼容，不能代原源兼容格。

复用001-hello/002-counter及全部原用例：T-01/B-DIALECT-I登记`旧case/check ID+原源/expected hash→兼容别名输入/迁移Node输入→同一新producer与模式→独立oracle`；每个旧义务只计一次，双输入不重复required。001 themes/model/design/col/selectable完整保留，002原0→1→0→0、主题及实例隔离不减；新Hello仅是独立最小case，不精简原例。无法仅改声明头的公开语法缺口由DIALECT/STATE具名关闭，不让作者重写业务规避不支持。

顺序为最小Node Hello→按已交能力扩Counter/双实例/输入焦点/布局异步，并同时验证适用旧源与Node副本；981/SOURCE/LANGUAGE在所需接口正式交付后扩大Node/场景，不等980完整PARITY才开始，最终传统UI全量与AGE完整语言门分别保留。980提供组件/模式和宿主，979持共同parser/投影，981持场景/生命周期与组合；不会为首Hello等待完整981/SOURCE、963、966或M2A-08。与963并行时，前门公共文件遵979短窗单写合同，980可先冻结组件、宿主/provider与原oracle；只有F0/H0/E0/B0正式D齐才运行真实Hello验证。AV先交，同源新A2R腿仍由原RUST批并行，不删任何模式义务。


### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r8当前）

**具体输入与期望。** 原件[auto-ui/examples/ui/002-counter/src/front/app.at](../../../auto-ui/examples/ui/002-counter/src/front/app.at)（下列节选L5–L52；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
use themes.themes: Auto, Solarized

widget App {
    msg { Inc, Dec, Reset, UseAuto, UseSolarized, ToggleMode }

    model {
        var count int = 0
        var theme_name str = "Auto"
        var dark_mode bool = false
    }

    // 主题/模式选择器（C-06/07：数据绑定，同帧校验发布）
    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }

    // Plan 506: 外壳改"内容即页面"（无 center 包裹，循 011 样板），配合
    // pac `window: "fit"` 窗口随内容收缩。
    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: () => {.count -= 1} }
                button "Reset" { onclick: () => {.count = 0} }
                button "+" { onclick: () => {.count += 1} }
            }
            row {
                button "Auto" { onclick: .UseAuto, style: "px-3 py-1.5 rounded-lg bg-primary text-primary-foreground" }
                button "Solarized" { onclick: .UseSolarized, style: "px-3 py-1.5 rounded-lg bg-secondary text-secondary-foreground" }
                button "切换明暗" { onclick: .ToggleMode, style: "px-3 py-1.5 rounded-lg bg-accent text-accent-foreground" }
            }
            style: "items-center gap-4 p-6"
        }
    }

    on {
        .Inc -> { .count += 1 }
        .Dec -> { .count -= 1 }
        .Reset -> { .count = 0 }
        .UseAuto -> { .theme_name = "Auto" }
        .UseSolarized -> { .theme_name = "Solarized" }
        .ToggleMode -> { .dark_mode = !.dark_mode }
    }
}


// 对照：同一计数器的显式 msg/on 写法（与上面的内联 lambda 简写等价）。
```

本包所需独立结果：原Counter加/减/Reset轨迹0→1→0→0且两个实例不串；主题Auto/Solarized、明暗/布局/事件保原行为；单text Hello不能关闭原001或Counter。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 原widget/use/model/msg/on/design源→979受检投影→969/970/AV或新A2R→同一UI宿主/renderer→事件/布局/真实窗口。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct UiProgram { view: DomainIrRef, model: TypedModel, handlers: HandlerSet, bindings: BindingGraph, requirements: RequirementSet }
struct UiInstance { execution: SessionId, model_owner: OwnerId, mounts: MountSet, input_epoch: Generation }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs) | 完整WidgetDecl，不另定义小组件全集 |
| [auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs](../../../auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs) | 替旧执行依赖，中立组件接面 |
| [auto-ui/crates/auto-ui-runtime/src/ui/iced/renderer/projection.rs](../../../auto-ui/crates/auto-ui-runtime/src/ui/iced/renderer/projection.rs) | 复用Iced呈现，Rust不解释guest逻辑 |

**状态、错误与资源。** 错prop/事件payload/未注册主题、ABI/grant/陈旧实例拒；未授权dispatch=0，不能回退旧Parser/VM或只Rust手画图。 App/model/组件/挂载/window分owner，重挂不重复初始化；Reload坏源保旧，focus/IME/捕获/关闭资源正确；九批模式各保I/V/R。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [auto-ui/examples/ui/001-helloworld/src/front/app.at](../../../auto-ui/examples/ui/001-helloworld/src/front/app.at) | 原001含themes/model/design/col/selectable，不能精简冒完成 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `b2763be5178fa040` | 拟ace.m2.p980.source01：run-av-ui/run-a2r-ui及原各模式V；负控/资源见V-DETAIL-02/03；原任务 B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V |
| [auto-ui/examples/ui/002-counter/src/front/app.at](../../../auto-ui/examples/ui/002-counter/src/front/app.at) | 状态/事件/主题与实例隔离 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `04937c35325faa74` | 拟ace.m2.p980.source02：run-av-ui/run-a2r-ui及原各模式V；负控/资源见V-DETAIL-02/03；原任务 B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V |
| [auto-ui/crates/auto-ui-runtime/tests/plan041_instance_ownership.rs](../../../auto-ui/crates/auto-ui-runtime/tests/plan041_instance_ownership.rs) | 真实生命周期/输入/视图owner原义务 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `3c4a05050dd86791` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 完整字段/组件/布局/Store/watch/setup/expose等按实际grammar冻结；NATIVE/WEB/RUST/TAURI/JET/ARK原适用格分别验证，不统一假定所有端等价。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`B-DIALECT-I`、`B-DIALECT-V`、`B-STATE-I`、`B-STATE-V`、`B-PARITY-I`、`B-PARITY-V`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### UI/Widgets爬梯分组与旧例映射（B-AV-I冻结，非新计划）

分母从`auto-ui/examples/ui/*/pac.at`与真实作者源、`auto-os/widgets-gallery/src/front/pages/*.at`和components依赖闭包、原Spec/测试并集冻结；themes/stylekit、缓存/gen/测试探针不按目录数冒应用，排除有原理由。031同号等按真实路径/稳定ID＋source hash区分。ui-gallery的Demo副本与规范源建立映射，不双计；Widgets组件验证可在隔离UI宿主消费原源，完整gallery容器/OS注册归OS064 ASSEMBLY，不反卡W1。

| 组/建议成组示例（最终以实际功能依赖冻结） | 组件/新增能力 | 每波独立关闭oracle |
|---|---|---|
| U01 Hello、002-counter、003-converter、011-calculator | 文本/函数/状态/按钮/数值；Gallery button/label/input | 原值/事件轨迹、中文/非法输入、两实例隔离，不一例一Plan |
| U02 004–010静态与表单类 | 布局/样式/主题、表单/选择/对话框/菜单/焦点 | 主题/可访问输入/布局与原独立预期；001选择复制完整保留 |
| U03 012–018及022等 | 时间/任务/列表/滚动/日期/编辑/路由、弹层 | timer不倍速、编辑/关闭/取消、列表与焦点隔离 |
| U04 014/017/023/025/026/027等服务/数据例 | 网络/异步/文件/数据库/权限/错误恢复 | 真服务/文件/DB oracle及故障/资源；依赖实际库正式切片，不用fake PASS |
| U05 019/020/024/029/030/031等媒体/图表/绘图 | 媒体/图片/图表/画布/GPU、原Gallery相关组件 | 实际输入/ROI/播放状态/坏资源恢复，原平台条件保持 |
| U06 032/038/041及剩余原例、复杂Widgets页/组件 | async/游戏/编辑器/嵌入和混合Node | 原状态/事件/卸载/资源与复杂组合、全部分母余行=0 |

这些是批次分组而非新增Plan号或新任务清单；同能力简单例可合并，复杂边界拆组，不能据编号跳依赖。每行由原case/check＋原源/expected hash映射到Node输入或同Schema widget兼容输入、当前后端、组件/场景/错误/资源oracle及原AC/V。原001、002和所有剩余例/Widgets页面明确归U01..06至少一组；新增/未覆盖构造补永久case。W1/W2/W3复用同一业务源与oracle；后端差异只记平台/产物/合法调度，不改业务规避支持。全部新分组验证未执行。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ace-migration/m2-autoui.md | 有限切片→实际已验支持/拒绝/来源 | 不把计划当现状 | AC-01..07、全部AC-B-* |
| SD-02 | modify | docs/specs/ace-migration/replacement-scope-and-evidence.md | 959行关联新producer/consumer及差额 | 替换闭合 | AC-03/05/06 |
| SD-03 | modify | auto-ui仓`docs/specs/ui/structure-closure.md`及`docs/specs/runtime/component-and-store-instances.md`（各owner） | 保原组件/Store语义，落实际已验ACE来源/模式及Node互操作接面，未交不改现状 | 传统UI不能被图形小例替代 | AC-07、AC-B-DIALECT/STATE/NATIVE/RUST/PARITY |
| SD-PHASE | add | `docs/specs/ace-migration/m2-autoui.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-TARGET-SCOPE | modify | `docs/specs/ace-migration/m2-autoui.md`及`replacement-scope-and-evidence.md`（真实交付后归原owner） | 原全目标并集→按Design55 §5.1的用户范围：Godot后置，生产TS/JS required，范围/证据/发行宣称一致；旧实现与用例保留 | 本轮用户明确裁定，防止旧Godot门残留或误删Vue能力 | AC-01/02/04/05、AC-PHASE；原work/review与范围核对 |

外仓领域知识由真实owner落当地Specs；语言侧只保通用接口/迁移指针，不重复领域规则。
| SD-HELLO | add | docs/specs/ace-migration/m2-autoui.md（沿原owner） | 未有早期交接→本切片实际已验接口/源/拒绝及剩余完整义务；交付后沉淀 | 最小Hello不冒全包完成 | AC-HELLO/AC-HELLO-REVIEW/AC-HELLO-LAND |
| SD-DETAIL | add | `docs/specs/ace-migration/m2-autoui.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
| SD-HANDOFF | modify | `docs/specs/ace-migration/m2-autoui.md`及本包原UI owner Spec（只在真实交付后） | 含糊等待979→IF-980-FIELDS的完整字段分母、来源/错误/覆盖反控与消费已交979接口的边界 | 979完整回验与无环接入 | AC-HANDOFF、AC-B-DIALECT、AC-07 |
| SD-NODE-COMPAT | add | `docs/specs/ace-migration/m2-autoui.md`（沿原owner，交付后沉淀） | 独立widget首入口→AGE Node首Hello/同Schema兼容别名、旧用例逐项映射与共用组件 | 用户本轮纠正 | AC-HELLO/AC-07/AC-NODE-COMPAT |
### 生产TS/JS与976的当前交接

WEB/ARK原I任务消费976/AC-TS-CONSUMER及受检`ScriptArtifact`接口，先核Vue `ts_adapter`与`ark_adapter`每个旧TypeScriptTrans委托的去向。UI保props/store/reactivity/capture映射，普通Auto表达式/语句/函数和导入闭包走同一新provider。不能保“未覆盖分支回调旧emitter”的过渡终态。WEB/TAURI/ARK各自原V/R核实际工程的值、事件、状态更新、模块与错误原span，单独tsc/快照不代运行。

新增V-TS-CONSUMER-UI由B-WEB-I/V/R、B-ARK-I/V/R及B-TAURI-V/R落实，原PARITY核完整合法UI表面和非Demo导入/函数。976能力可先独立交付，980最终产品接线不反成为其前置；HELLO的node/text→AV/FFI首窗仍沿原最小链，不新增976整包等待。Godot目标后置不改变AGE自有Node/2D/3D或本包UI义务。

### 三端最终一致与AC阶段移交（2026-11-16当前优先）

本包适用范围为**UI/AGE领域及原生host/事件回调**。A按MIN-A精确表及既定非图形语言/库/FFI基础验收；完整UI/Widgets和图形A2R已迁交B三波，AC真实基础仍保留，A2PY标准库/AV三provider与原其他required保持。AC基础外剩余差额允许B后延，逐原行ID/输入/独立oracle/资源/错误/源身份记录B-owner与关闭条件，交[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)，异步调度交[1067](1067-ace-ac-native-async-and-task-parity.md)。不能以stub或原profile不支持签AC已实现；M2最终三端必须同平台可用，A后延只是时间调整。A2C完整对齐增强，既有代码/用例/支持事实保留，缺完整面不阻M2。

在本包原能力work/最终验证/review明确区分“A已验接口”“B后延AC行”：移交有正式producer/版本/所需前置，最终真实实现和补验归1068及原owner协作，不回落旧VM或重写归档Plan。985/A可收本包A正式切片，不反等1068；本包如已有B则保唯一树续行，A切片不能勾全包完成。

本包NATIVE是AV桌面宿主模式，不能自动当AC证明。B-NATIVE-I/V/R交IF-980-NODE-DESKTOP-A；981的SOURCE/SESSION/LANGUAGE提供领域语义，1068/T-04接AC机器码业务并交IF-1068-AC-DESKTOP。完整状态/事件/挂载/关闭与真实UI/renderer门不缩减，不能只测序列化文件。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/ace-migration/m2-autoui.md（实际交付） | A所有AC必齐→A基础/具名B移交，最终三端完整，A2C增强 | 用户明确排程 | AC-AC-STAGED |

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-NODE-DESKTOP | modify | docs/specs/ace-migration/m2-autoui.md及原UI runtime owner Spec（实际交付后） | Hello快照/字节验证边界→A可复用AV UI-FFI入口/事件/owner；AC桌面交1068在B补齐，A2R独立 | 用户本轮裁定 | AC-NODE-DESKTOP、AC-AC-STAGED |

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-UI-WAVES | modify | docs/specs/ace-migration/m2-autoui.md及replacement-scope-and-evidence.md（真实交付后） | A全UI/AV-A2R→MIN-A-UI；完整示例/Widgets W1 AV→W2 AC→W3 A2R，B全部原模式保持 | 用户本轮三波与A最小出口裁定 | AC-UI-WAVES、AC-PHASE |

## 6. 测试设计

### 可复用Node桌面入口验证（拟议，全部未执行）

B-NATIVE-I在现有runtime/tests建立拟新增`ace_node_desktop.rs`；旧001/002及`ace_ui_hello` case保留，旧widget原源走同Schema兼容别名、Node副本只改声明头/具名必需语法适配，保原业务/expected/hash，一项义务不双计。以下由B-NATIVE-V实跑、B-NATIVE-R独立核，不因Hello/STATE旧PASS自动关闭。

| Case / AC | 实际入口与输入 | 区分性oracle/负控 |
|---|---|---|
| V-NODE-DESKTOP-ENTRY / AC-NODE-DESKTOP | 真实项目runner装载`ace-hello-minimal/{app,app-body}.at`、另一个Node名/普通模块函数、001完整原源和Node副本；仓外cwd | 正确文字/中文/原span，原001选择复制/主题不减；只换源不手写Rust或重编宿主，预构建后body变化cargo/rustc=0；坏Schema/字段在UI dispatch前拒、修复后同入口真窗。  本轮A只取V-UI-A-MIN对应子面；本行完整格由B-AV/W1与最终PARITY核。 |
| V-NODE-DESKTOP-FFI / AC-NODE-DESKTOP | runner→实际967自动binding/shim→所选UI provider的apply/open；对同长度不同文字改源，原002绑定 | UI收到并渲染实际typed更新，记录独立宿主接收值/调用/实例身份；**移除/替换实际UI提交为仅byte_len**须因缺UI效果而红，不能用相同截图/字节数或宿主直读快照旁路通过；错ABI/provider/权限及过期handle调用前拒，UI无新增效果。  本轮A只取V-UI-A-MIN对应子面；本行完整格由B-AV/W1与最终PARITY核。 |
| V-NODE-DESKTOP-EVENT / AC-NODE-DESKTOP | 002完整主题/计数副本，两实例，真实鼠标点击/键盘输入→typed队列→AV handler→FFI更新 | 单例0→1→0→0，定向事件两实例2:1、业务值与窗口一致；断开事件回AV或令Rust手写计数须红；按已交STATE核异步late只写存活实例。  本轮A只取V-UI-A-MIN对应子面；本行完整格由B-AV/W1与最终PARITY核。 |
| V-NODE-DESKTOP-LIFE / AC-NODE-DESKTOP | 创建/运行/关闭/重开、撤权/坏更新/修复、UTF8借用输入释放后刷新、旧代late事件 | UTF8/NUL按原组件显示/数据合同不悬空，UI主线程；失败保旧，关闭后句柄/事件拒绝、资源按last-owner净回收，重开独立；原Frame/Tick/资源计量oracle保留，不拿“进程退出”冒所有引用释放。  本轮A只取V-UI-A-MIN对应子面；本行完整格由B-AV/W1与最终PARITY核。 |
| V-NODE-DESKTOP-HANDOFF / AC-NODE-DESKTOP、AC-AC-STAGED | 正式IF源码/ABI/SDK与独立消费工程，三类AV/A2R/AC收据 | 只有HELLO/AXR0或AC缺owner不可签一般化A入口/最终AC；A可以明确AC待补且交1068，不等待其运行。已交A2R仍按原范围，不可当AC。  本轮A只取V-UI-A-MIN对应子面；本行完整格由B-AV/W1与最终PARITY核。 |

未来命令在唯一980工作树：UI已存在`cargo test -p auto-ui-runtime --features ace-bridge --test ace_ui_hello -- --test-threads=1`；新target由I创建后才用`--test ace_node_desktop`，原desktop_behavior/image_surface_contract/plan041按触面串行运行，真实CLI命令由I读help冻结，不能以测试构造器替CLI。语言选既有`cargo t ace-source ace-av ace-host ace-foreign-contract ace-abi`中真实职责及必要集成组，新增source/schema/ABI触面须核下游；窗口GPU排单实例，AG仅命名受影响例；Linux/正式perf沿既定条件，不附全AG/全仓门。准备/选择/实际执行/条件排除及子终态分账。独立review在实施停写后的同树上下文重构，角色未另指派，不创建新会话/第二树。

### r10字段交接验证（拟执行）

沿原V-B-DIALECT、V-UI-SOURCE及V-DETAIL-01/02/04增加IF-980-FIELDS的字段级断言：完整AST/Spec字段分母与实际表双向一致；原002源和Node副本保原body/源位置；合法投影、未知字段与同类型非法父槽、实际删除一条必需映射、删除一条必需清单行、修复后同入口恢复各自可区分。B-DIALECT-I在现有`ui_domain_validation`及语言通用target建永久命名case，V运行原职责组/定向UI用例，R核真实source→candidate→verify→coverage与正式来源；仅metadata数量相等不足以证明字段被消费。交接证据仍保留原UI完整模式未闭格，不能把CPU接面PASS当真实窗口PASS。

拟永久检查键`p980_required_fields_handoff`与`p980_required_field_mapping_controls`分别覆盖完整清单/原源和值/删项/恢复，绑定AC-HANDOFF、原V-UI-SOURCE/V-DETAIL-04及B-DIALECT-I/V/R；真实已有等价case优先复用并登记同义映射，不另加required计数。本轮未注册、生成或运行这些case。

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部未执行。cwd前缀lang/ui/man为本Plan唯一tree。各I读取原runner/help/项目配置固定参数/工具版本，创建永久正反例，未冻结不能进入V；现有target绿不自动证明新ACE支持。只保原支持承诺，原TODO不冒已实现，但实际承诺的缺环境格不得免验。

材料唯一D:/autostack/plan-artifacts/plan-980/，开工登记owner/用途/在途；按仓/批区分，外置target/SDK/安装副本均归scope。窗口/GPU单实例错峰，原ignore/平台/资源预算保持；准备/运行/清理、选择/执行/排除/子终态分列。

| Case | AC | work入口 | 独立观察 |
|---|---|---|---|
| V-B-DIALECT | AC-B-DIALECT | ui: cargo test -p auto-ui-dialect --lib；cargo test -p auto-ui-compiler --test ui_domain_validation；lang: cargo t ace-source ace-syntax | 普通标识符上下文保持；旧widget原源经同一AGE Schema兼容别名的解析/绑定/错误span正确，规范Node保相同body/组件行为；坏字段拒绝，缺字段不能悄悄忽略。 |
| V-B-STATE | AC-B-STATE | ui: cargo test -p auto-ui-runtime --test plan041_instance_ownership --test plan033_b1_scope_isolation -- --test-threads=1；lang: cargo t ace-av ace-host | 双App各两组件不串；异步任务返回只写仍存活实例；MIX-03无HTTP版本用真实异步host，HTTP实腿由983/联验补齐。 |
| V-B-NATIVE | AC-B-NATIVE | ui: cargo test -p auto-ui-runtime --test desktop_behavior --test image_surface_contract -- --test-threads=1；工具现有auto run -r vm执行隔离工程，记录实际binary身份 | 001原文字选择复制/主题，页面切换、输入法/捕获、媒体/终端、关闭重开与资源回收逐项实测；不是首帧即通过。  本轮A只取V-UI-A-MIN对应子面；本行完整格由B-AV/W1与最终PARITY核。 |
| V-B-WEB | AC-B-WEB | ui: cargo tu（保原单进程）；cargo test -p auto-ui-runtime --test vue_capabilities；实际生成工程按自身package.json锁定脚本构建并操作 | 真实浏览器挂载/事件/绑定与源expected一致；setup只一次、watch卸载停止；生产表达式不走旧TypeScriptTrans。 |
| V-B-RUST | AC-B-RUST | ui: cargo tu（原单进程）；lang: cargo t ace-a2x generated-delivery-controls；RUST原工程runner与工具版本由I冻结并真实执行，不能emit-only | 未改生成物冷编译运行，props/msg/route与原独立oracle相符；坏源保旧工程。 |
| V-B-TAURI | AC-B-TAURI | ui: cargo tu（原单进程）；lang: cargo t ace-a2x generated-delivery-controls；TAURI原工程runner与工具版本由I冻结并真实执行，不能emit-only | 真实Tauri窗的invoke和事件、失败诊断及重开，不能仅浏览器前端通过。 |
| V-B-JET | AC-B-JET | ui: cargo tu（原单进程）；lang: cargo t ace-a2x generated-delivery-controls；JET原工程runner与工具版本由I冻结并真实执行，不能emit-only | 原支持工程在实际Jet工具链构建运行，事件/状态独立oracle，缺平台未执行。（r11：归 M3/v0.7，v0.6 不执行） |
| V-B-ARK | AC-B-ARK | ui: cargo tu（原单进程）；lang: cargo t ace-a2x generated-delivery-controls；ARK原工程runner与工具版本由I冻结并真实执行，不能emit-only | 原支持工程在真实Ark工具链执行，原widget/事件/状态正确，缺平台未执行。（r11：归 M3/v0.7，v0.6 不执行） |
| V-B-PARITY | AC-B-PARITY | ui: cargo test -p auto-ui-runtime --test theme_scope_isolation --test gallery_golden -- --test-threads=1；实机与浏览器逐原支持模式验证，窗口单实例 | 主题scope与原画面/交互/生命周期对照，UI058等AGE新能力不是本包替代；每格source/backend/runtime/行为收据齐。 |
| V-UI-SOURCE | AC-07 | B-DIALECT-I建完整分母普通源码正反例，B-PARITY-I接字段/构造覆盖检查；V按现有ui_domain_validation/领域target运行新命名case | 合法传统UI字段有投影/模式/owner，坏字段或错事件参数报原span；删除一条必需字段/模式行检查红、合法近邻仍绿；修复后同入口成功，不能静默丢字段 |
| V-UI-FULL | AC-07/AC-NODE-COMPAT | B-NATIVE-I/B-RUST-I建纯UI多文件工程（旧widget原源走同一Schema兼容别名，另建声明头迁移Node对照），V跑AV及新A2R→编译→实际UI运行；B-PARITY-V汇总原组件/模式必需格 | 无2D/3D声明仍完整工作，双App各两组件、局部2:1事件、Store/路由/异步卸载/主题/输入各对独立oracle；AV改源无Cargo/rustc，A2R实际执行生成逻辑，无VM fallback。001或少量控件绿不代未覆盖格 |

受影响共享IR/语义加完整职责与必要下游；T-05核全批/组合/原资源门，身份不变证据复用，不反复全仓。局部review不代全包final；未闭in-scope阻execution_done。

### HELLO验证（T-HELLO-I建永久源/用例与命令，全部未执行）

拟窗口命名入口为`cargo test --manifest-path <ui>/Cargo.toml -p auto-ui-runtime --test ace_ui_hello -- --test-threads=1`（新target由I建立），公开run/build参数先核现有CLI并冻结；不得把此拟命令写成已执行。

| Case | AC | 输入→producer/consumer→独立关闭oracle | owner/边界 |
|---|---|---|---|
| V-HELLO-TEXT | AC-HELLO | AGE node/view/text普通源、改根名/中文文本→H0+E0+AV+F0 Rust provider→真实Iced窗口；源字节、provider复制值/dispatch和可见文字相符，非旧链/硬编码图 | work；静态窗口不冒事件/全UI |
| V-HELLO-BODY | AC-HELLO | 第二同形单text源由普通Auto函数返回值；改函数Hello→你好后真AV结果/FFI接收/窗口均变，未变host的cargo/rustc/bindgen=0 | work；不承诺970同PID热更；重复公开运行仍需新值 |
| V-UI-AXR0 | AC-B-RUST | 同源/资源→新A2R生成→真实rustc→同UI provider→窗口；改函数结果随变，禁AV/旧VM provider仍运行；两端各对独立文本/来源与字节oracle | work/B-RUST-I/V/R；不阻AV HELLO交接，emit-only或Rust手写业务不关闭 |
| V-HELLO-REJECT | AC-HELLO | 坏UI语法/文本类型报原span；错ABI/provider/权限拒绝且Rust callee=0，坏文本/宿主失败保旧；精确修复后同公开入口真Run | work；不把timeout/缺GPU/工具当语言负例 |
| V-HELLO-LIFE | AC-HELLO | guest文本释放/替换后宿主复制仍正确；20轮创建窗口/关闭和取消，session/库/文本owner无增长，关闭旧代不能写新窗口；实际窗口可resize/退出 | work；不假称物理显存归零或覆盖全部原生交互 |
| V-HELLO-PACK | AC-HELLO | B0配方构建最小host/AV闭包；A2R包随AXR0-PACK/原RUST另核；仓外异根/任意cwd运行，缺shim/错ABI拒绝修复再Run；源/grammar/provider/toolchain与包hash齐 | work；AV开发前提明示，A2R用户运行端无Rust/开发仓，非986完整SDK升级 |
| V-HELLO-SCHEMA | AC-HELLO/AC-NODE-COMPAT | 正式输入root=node，记录AGE Schema/grammar及上下文身份；改Node名/函数/文本仍真实显示；无AGE或坏Schema在FFI前拒绝，callee=0；不得经过widget源码生成或旧解析入口 | T-HELLO-I/V/R；H0配置反控可复用同身份有效证据，窗口链仍独立验 |
| V-UI-NODE-COMPAT | AC-07/AC-NODE-COMPAT | 001/002旧源原字节经同Schema别名；只改声明头Node副本复用原业务序列/独立expected，AV/A2R及原模式格分别记录；普通标识符widget/model/view/on不误改，坏字段仍报各自原span；关闭别名旧源拒绝、Node仍运行 | B-DIALECT-I/V、B-STATE-I/V、B-PARITY-I/V/R；HELLO仅核最小同形文本，不以Counter/一模式冒完整覆盖 |

GPU/窗口单实例，未来按明确授权排窗；纯文档本轮不运行。实现者自检后同一唯一树独立final，R与D分别审查/落主，不另建第二review树或本轮派agent。只有同版完整HELLO证据可交下一批，旧M1窗口截图不能抵ACE Hello。

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V | 原Counter加/减/Reset轨迹0→1→0→0且两个实例不串；主题Auto/Solarized、明暗/布局/事件保原行为；单text Hello不能关闭原001或Counter。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V | 错prop/事件payload/未注册主题、ABI/grant/陈旧实例拒；未授权dispatch=0，不能回退旧Parser/VM或只Rust手画图。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V | App/model/组件/挂载/window分owner，重挂不重复初始化；Reload坏源保旧，focus/IME/捕获/关闭资源正确；九批模式各保I/V/R。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### 当前目标范围验证（拟实施，未运行）

| Case | AC/任务 | 输入→producer→consumer→独立oracle与负控 |
|---|---|---|
| V-TS-CONSUMER-UI | AC-TS-CONSUMER-UI；WEB/ARK/TAURI原I/V/R | 真实Vue/Ark及Tauri适用工程→976新provider→框架adapter→原运行器；观察handler/computed/watch/setup、props/store及导入调用的实际值/事件/资源/错误span。注入旧TypeScriptTrans fallback、删合法函数或误绑响应式引用须失败；禁用Godot不影响node/text首窗，首窗成功不代完整Web/Ark通过。 |

| V / AC | 原owner/输入 | 独立预期与边界 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原work及最终验证：A真实基础/AV-A2R结果＋完整AC后延账，缺owner/基础stub负控；B回验真AC | A基础真实且B每行去向/关闭条件齐才可交A；不得记AC PASS/NA或从分母消失；B真实三端与原独立oracle齐才关闭后延行。 |

### 分阶段必验（拟议，未执行；优先于旧全量A表述）

| V / owner | 输入/入口 | 独立关闭/负控 |
|---|---|---|
| V-UI-A-MIN / B-NATIVE-I/V/R | 现有真实项目入口/Schema与UI-FFI；MIN-A-UI三类源及双实例 | 真窗文字/中文、0→1→0→0、两实例2:1，关开/late/权限/坏更新恢复；断实际UI submit只留byte_len必红。完整Widgets/A2R缺证但有B承接不阻此A；A最小本身缺证不能签。 |
| V-UI-W1 / B-AV-I/V/R | U01..06全部原例/Widgets依赖，经普通Node或Schema兼容源→AV→同一宿主 | 全部分母对独立交互/错误/资源oracle，目录空选/漏页/只截图/只CPU/手写Rust业务红；不要求AC/A2R或rustc guest。新target由I建于原runtime/tests/领域owner，原ace_ui_hello/desktop_behavior等按触面复用。 |
| V-UI-W3 / B-RUST-I/V/R | W2后同源U01..06→新A2R生成→真实rustc→真实窗/事件/资源及包 | 生成物不手改、guest不AV/旧VM，完整值/交互/关闭/失败恢复；AXR0 stdout上窗不代一般UI。源码/ABI失配或emit-only红，宿主预编译与guest编译分别计时。 |
| V-UI-WAVE-RECEIPT / PARITY原I/V/R | 每波正式source/Schema/ABI/provider/SDK/后端/用例/独立oracle/结果与残余行 | W1可正式交而W2/W3仍pending；W2可正式交而W3 pending。错把另端产物/缺行/branch WIP当当前波或最终三端必须红；最终PARITY收三波及WEB/TAURI。 |

拟补波次case由B-AV-I/B-RUST-I建在原测试owner，沿现有TestCase/固定组，不造新tm或每波新runner。AV用ace-source/ace-av/ace-host/ace-abi/ace-foreign-contract等真实职责和受影响UI `--features ace-bridge`命名target；AC归1068，A2R到W3才跑图形生成编译端。GPU窗口单实例，AG只受影响命名例；Linux/perf原条件不变。旧表V-UI-FULL/V-B-NATIVE/V-NODE-DESKTOP-*覆盖完整U组，在B按当前波执行；不是A全量门。各I读真实CLI/help冻结参数，不虚构新后端选择已存在。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 验证 |
|---|---|---|---|
| AC-01 | work | 分母/批次/前置/原模式/命令/资源完整 | T-01及各I |
| AC-02 | work | 完整原能力经真实原源/工程执行 | 全部I/V |
| AC-03 | work | 实际consumer/权限/资源/恢复接线、无legacy fallback | 全部V/959行 |
| AC-04 | review | 独立核各批及全包AC/差额/健康/Spec | 各R/T-05 |
| AC-05 | merge | 正式来源/规范/归档/清理齐 | T-06/07 |
| AC-06 | work | 原T-08补充义务全部分配且闭合 | 全部AC-B-* |
| AC-07 | work | 传统UI全部既有语法/字段/组件/模式必需格经ACE接管，原源不强制改Node；纯UI可AV开发及A2R编译运行，输入/状态/主题/资源/恢复不减、无旧执行/发射fallback；接面供981但不等其产品PASS | V-UI-SOURCE/FULL及原各批V；T-01、DIALECT/STATE/NATIVE/RUST/PARITY的I/V |
| AC-B-DIALECT | work | 普通标识符上下文保持；旧widget原源经同一AGE Schema兼容别名的解析/绑定/错误span正确，规范Node保相同body/组件行为；坏字段拒绝，缺字段不能悄悄忽略。 | B-DIALECT-V/R |
| AC-B-STATE | work | 双App各两组件不串；异步任务返回只写仍存活实例；MIX-03无HTTP版本用真实异步host，HTTP实腿由983/联验补齐。 | B-STATE-V/R |
| AC-B-NATIVE | work | A仅MIN-A-UI/V-UI-A-MIN；原001选择复制/主题、页面/输入法/捕获、媒体/terminal等完整原生产品义务在B-AV/W1按原oracle实测，首帧不能代完整覆盖 | B-NATIVE-V/R交A；B-AV-V/R与最终PARITY交完整格 |
| AC-B-WEB | work | 真实浏览器挂载/事件/绑定与源expected一致；setup只一次、watch卸载停止；生产表达式不走旧TypeScriptTrans。 | B-WEB-V/R |
| AC-B-RUST | work | 未改生成物冷编译运行，props/msg/route与原独立oracle相符；坏源保旧工程。 | B-RUST-V/R |
| AC-B-TAURI | work | 真实Tauri窗的invoke和事件、失败诊断及重开，不能仅浏览器前端通过。 | B-TAURI-V/R |
| AC-B-JET | work | 原支持工程在实际Jet工具链构建运行，事件/状态独立oracle，缺平台未执行。 | B-JET-V/R（r11：随批归 M3/v0.7） |
| AC-B-ARK | work | 原支持工程在真实Ark工具链执行，原widget/事件/状态正确，缺平台未执行。 | B-ARK-V/R（r11：随批归 M3/v0.7） |
| AC-B-PARITY | work | 主题scope与原画面/交互/生命周期对照，UI058等AGE新能力不是本包替代；每格source/backend/runtime/行为收据齐。 | B-PARITY-V/R |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |
| AC-HELLO | work | AGE Schema驱动的Node单text字面量/Auto函数源在AV→真实Rust FFI→Iced窗口成立，原span/文本寿命/失败恢复/冷消费可核 | T-HELLO-I/V；V-HELLO-* |
| AC-HELLO-REVIEW | review | 同树独立逐源/ABI/dispatch/窗口/资源负控复核，绑定正式依赖来源 | T-HELLO-R |
| AC-HELLO-LAND | merge | 首Hello正式源/接口/产物/已验规范落主，保原001与全部传统UI/模式验收义务 | T-HELLO-D |
| AC-NODE-COMPAT | work | 首Hello直接用AGE Node；旧widget原源仅经同一Schema兼容别名，声明头迁移另计；组件/状态/模式/独立oracle不减，无第二传统接入线或全局关键字替换 | T-HELLO-I/V、B-DIALECT/STATE/PARITY-I/V；V-HELLO-SCHEMA、V-UI-NODE-COMPAT，原R独立复核 |
| AC-HANDOFF | work | IF-980-FIELDS覆盖完整WidgetDecl/引用构造分母并有实际覆盖/拒绝/删映射/删清单行反控，来源正式可供979回验；只消费979已交接口，不反等其归档或981整包，原UI全量义务保持 | B-DIALECT-I/V/R及原phase fold；V-B-DIALECT/V-UI-SOURCE/V-DETAIL-04，交979 V-11 |

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-NODE-DESKTOP | work | A的MIN-A-UI真实项目入口经AV/自动Rust FFI产生UI效果与typed事件/owner/恢复；原V-NODE-DESKTOP-*完整覆盖在B-AV，AC/A2R分别W2/W3，不以HELLO/byte_len/AXR0代一般化或某完整波 | B-NATIVE-I/V/R及V-UI-A-MIN；B-AV/W1、1068/W2、RUST/W3，原final |

### 当前目标范围验收

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-TS-CONSUMER-UI | work | WEB/ARK/TAURI完整生产TS/JS接976新provider，无TypeScriptTrans委托残留，规范合法非Demo模块/函数仍支持；首Hello最小链不被附加独立TS/Godot前置 | B-WEB-I/V/R、B-ARK-I/V/R、B-TAURI-V/R、V-TS-CONSUMER-UI；原独立review逐项复核 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED闭合，AC基础真实、剩余差额完整移交1067/1068/实际owner，AV/A2R及原其他A-required不减；最终三端未验不得冒支持。原work落实、原独立review核，无新增任务或本轮完成数。

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-UI-WAVES | work | MIN-A-UI独立正式可交；完整U01..06/Widgets按W1 AV、W2 AC、W3 A2R分别对独立oracle关闭，最终全部原产品/模式义务齐；UI-only三端不逐批同时实现 | V-UI-A-MIN/W1/W3/WAVE-RECEIPT；原R、T-05独立final |

## 8. 执行步骤

当前共32任务，已完成12项（B-NATIVE-I/V/R r13 批已复核勾选）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### A最小与B各波唯一任务清单（阶段按行标签）

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- [x] **T-HELLO-I｜M2-A｜owner_stage: work｜AC-HELLO/AC-NODE-COMPAT**：967/F0-D、979/H0-D、969/E0-D、978/B0-D齐后，按§5冻结AGE node首源/Schema激活及旧widget兼容配置、CLI/主线程/文本所有权与资源，建ace-hello-minimal及ace_ui_hello永久case；只实现同一受检通路，不等T-01全部UI分母/STATE async/PARITY。

- [x] **T-HELLO-V｜M2-A｜owner_stage: work｜AC-HELLO/AC-NODE-COMPAT**：T-HELLO-I后跑全部V-HELLO-*与真实受影响UI命名case/必要语言职责并集，AV首窗全链记录独立oracle/窗口/FFI dispatch/退出；AXR0由原RUST并行验证，未执行腿保持pending，不阻AV部分正式交接。

- [x] **T-HELLO-R｜M2-A｜owner_stage: review｜AC-HELLO-REVIEW**：T-HELLO-V的AV全部V-HELLO-*齐后停写同树独立核最小普通源到真实Iced、AV FFI与资源/反控；finding返I，不把AV Hello pass冒A2R或完整UI。

- [ ] **T-HELLO-D｜M2-A｜owner_stage: merge｜AC-HELLO-LAND**：T-HELLO-R pass后按各owner phase fold正式合入首Hello代码/已验规范与source/ABI/包来源，保唯一树/resync继续原九批与全包review；不改默认、不归档980或宣称完整RG-A03/M2A-08已过。



- [x] **T-01｜M2-A｜owner_stage: work｜AC-01/07/AC-PHASE**：核959行及§5完整传统UI的grammar/AST/Spec/组件/模式分母、来源/接口与唯一树/写窗，冻结纯UI AV/A2R及各批命令/oracle/资源，不能只列图形小例。

- [x] **B-DIALECT-I｜M2-A｜owner_stage: work｜AC-B-DIALECT/AC-02/03/06/07/AC-NODE-COMPAT/AC-HANDOFF**：按WidgetDecl全部字段逐项投影/明确模式适用性，schema→typed领域IR→ACE行为；包含named_views/bind/routes/setup/watch/expose/actions/timer/theme及实际AST/Spec其余字段。先落实§5 IF-980-FIELDS的可执行字段清单/通用适配和永久正反例，R复核/正式交付子面后继续完整DIALECT；消费已交979基础、不等979归档。按§6冻结命令，原传统UI/V-UI-SOURCE/FULL义务不减。

- [x] **B-DIALECT-V｜M2-A｜owner_stage: work｜AC-B-DIALECT/AC-07/AC-NODE-COMPAT/AC-HANDOFF**：运行V-B-DIALECT完整职责/真实consumer及IF-980-FIELDS子面，独立expected/源/工具/资源/子终态齐。核完整字段/构造覆盖、删实际映射与删清单行反控；交接子面不提前勾本任务或整个DIALECT。

- [x] **B-DIALECT-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源；可先按§5复核IF-980-FIELDS子面并正式phase fold交979，完整本批仍须核全部原义务才勾选；缺陷回I，不冒整包交付。

- [x] **B-STATE-I｜M2-A｜owner_stage: work｜AC-B-STATE/AC-02/03/06/07**：用ACE执行服务替旧handler bytecode生成/调用，接组件props/emit/computed与App store、挂起/唤醒、卸载和late generation。 按§5真实改面补永久fixture并固定§6命令。 同时落实§5完整传统UI门与V-UI-SOURCE/FULL的本批部分，不等981产品终验。

- [x] **B-STATE-V｜M2-A｜owner_stage: work｜AC-B-STATE**：运行V-B-STATE完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [x] **B-STATE-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [x] **B-NATIVE-I｜M2-A最小｜owner_stage: work｜AC-B-NATIVE/AC-02/03/06/07**：实现MIN-A-UI普通Node入口/实际UI-FFI/typed事件与owner；新用例V-UI-A-MIN；完整001/002与所有Widgets、媒体/terminal迁B-AV。 [✅ r13批 round 24/25：ui980 provider+桥+通用入口，收据 §9]

- [x] **B-NATIVE-V｜M2-A最小｜owner_stage: work｜AC-B-NATIVE/AC-07**：只关闭V-UI-A-MIN及所用接口/错误/资源原义务，真实窗/事件/恢复，不以字节探针代UI；完整V-UI-FULL交B波。 [✅ r13批 round 24：CPU 7/7+排窗 e2e 1/1+受影响面全绿]

- [x] **B-NATIVE-R｜M2-A最小｜owner_stage: review｜AC-04**：独立核MIN-A-UI/正式source/ABI/FFI效果与B承接，phase fold交IF-980-NODE-DESKTOP-A；不要求完整Widgets/图形A2R，也不签完整NATIVE产品覆盖。 [✅ round 25 批次终核 pass，authored 反例 e203caa8]

- [ ] **B-AV-I｜M2-B-W1｜owner_stage: work｜AC-UI-WAVES/AC-07**：M2A-08后冻结U01..06原源/Widgets分母和旧新oracle映射，在既有DIALECT/STATE/NATIVE owner扩完整AV组件/场景/输入/错误/资源；只做AV，多个简单例按能力成组，不另建引擎/runner。

- [ ] **B-AV-V｜M2-B-W1｜owner_stage: work｜AC-UI-WAVES/AC-07**：运行V-UI-W1与该组受影响完整职责/真实窗口；原001/002和全部示例/Widgets余行闭合，不运行图形A2R；收981所需AV域格，不能等AC/OS全量再交。

- [ ] **B-AV-R｜M2-B-W1｜owner_stage: review｜AC-UI-WAVES/AC-04**：同树停写独立核完整AV分母/原oracle/失败控制/来源，phase fold交UI-W1-AV及宿主/域ABI供1068 W2；不提前签三端/整包PARITY或归档。

- [ ] **B-WEB-I｜M2-B独立现役模式｜owner_stage: work｜AC-B-WEB/AC-02/03/06**：Vue生成使用新TS/JS provider，保watch/setup/expose/ext_imports/style及绑定；服务端使用已交宿主接口或隔离真实fixture，不等整个983才能证UI表达式。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-WEB-V｜M2-B独立现役模式｜owner_stage: work｜AC-B-WEB**：运行V-B-WEB完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-WEB-R｜M2-B独立现役模式｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-RUST-I｜M2-B-W3｜owner_stage: work｜AC-B-RUST/AC-02/03/06/07**：先按§5 AXR0切片运行同源单text V-UI-AXR0并独立复核/正式交接；随后多文件store/组件与sidecar经新A2R生成，保持原Rust UI行为和增量产物；旧AST emitter无回退。 按§5真实改面补永久fixture并固定§6命令。 同时落实§5完整传统UI门与V-UI-SOURCE/FULL的本批部分，不等981产品终验。 新增完整图形范围只在UI-W2-AC正式后按V-UI-W3执行；原AXR0仅复用已验切片，不能提前签完整第三波。

- [ ] **B-RUST-V｜M2-B-W3｜owner_stage: work｜AC-B-RUST/AC-07**：运行V-B-RUST完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 同时核V-UI-FULL的本批真实结果与完整分母，不以图形小例替代。 新增完整图形范围只在UI-W2-AC正式后按V-UI-W3执行；原AXR0仅复用已验切片，不能提前签完整第三波。

- [ ] **B-RUST-R｜M2-B-W3｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。 新增完整图形范围只在UI-W2-AC正式后按V-UI-W3执行；原AXR0仅复用已验切片，不能提前签完整第三波。

- [ ] **B-TAURI-I｜M2-B独立现役模式｜owner_stage: work｜AC-B-TAURI/AC-02/03/06**：接新TS/JS与Rust backend生成，保持前后端invoke/事件/权限边界；旧AST emitter无回退。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-TAURI-V｜M2-B独立现役模式｜owner_stage: work｜AC-B-TAURI**：运行V-B-TAURI完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-TAURI-R｜M2-B独立现役模式｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

<!-- r11（2026-11-16 用户裁定）：B-JET-I/V/R 与 B-ARK-I/V/R 六任务行撤出 v0.6 执行清单，归 M3（v0.7）。原文与 AC/V 留档：B-JET-I=保持现有Jet生成模式，以受检UI领域IR供目标adapter；不新增原未承诺能力；旧AST emitter无回退（AC-B-JET/V-B-JET：原支持工程在实际Jet工具链构建运行，事件/状态独立oracle，缺平台未执行）。B-ARK 同型（AC-B-ARK/V-B-ARK）。M3 再启时按原义务实施。 -->

- [ ] **B-PARITY-I｜M2-B分波接收/最终汇合｜owner_stage: work｜AC-B-PARITY/AC-02/03/06/07/AC-NODE-COMPAT**：从001按文字布局→表单列表滚动→菜单Blueprint→媒体编辑分组；核完整语法/字段/组件/变体/模式双向表，补未被Demo覆盖的语法/错态及删除必需行负控，建V-UI-SOURCE/FULL并汇总纯传统UI AV/A2R实际结果。按§5接fixture/固定命令，不等981完整Node终验。 按V-UI-WAVE-RECEIPT独立接W1/W2/W3，不以本任务整项完成反卡前一波；最终收全部现役模式。

- [ ] **B-PARITY-V｜M2-B分波接收/最终汇合｜owner_stage: work｜AC-B-PARITY/AC-07/AC-NODE-COMPAT**：运行V-B-PARITY完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 同时核V-UI-FULL的本批真实结果与完整分母，不以图形小例替代。 按V-UI-WAVE-RECEIPT独立接W1/W2/W3，不以本任务整项完成反卡前一波；最终收全部现役模式。

- [ ] **B-PARITY-R｜M2-B分波接收/最终汇合｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。 按V-UI-WAVE-RECEIPT独立接W1/W2/W3，不以本任务整项完成反卡前一波；最终收全部现役模式。

- [ ] **T-05｜M2-B最终｜owner_stage: review｜AC-04**：全包独立final核全部批收据/组合/遗漏/延期/workaround、warning/fmt/debug和Spec；缺必需结果不得pass。

- [ ] **T-06｜M2-B最终｜owner_stage: merge｜AC-05**：按owner次序合入/落规范，核真实来源/consumer同版，全部齐后归档。

- [ ] **T-07｜M2-B最终｜owner_stage: merge｜AC-05**：摘要折Plan、输出逐目录对账/清scope/登记，guard clean后清树。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r12：按用户最新三端目标修阶段/移交，增SD/V/AC-AC-STAGED；原任务/状态/进度保留，仅规划，无新增实际验证/签章。

2026-11-16 stage: new/revision | PLAN-980 r11 | 用户明确修订M2目标范围：Godot脚本/场景/工程支持后置，TS/JS保生产消费闭包。新增验证未执行；既有已交证据按其旧范围/身份复用，不能证明新范围检查。原状态与实际已完成任务保留；本记录非产品/独立实现review PASS，不授权实施/清理或恢复目标。

2026-11-15 r10规划静态预检：979/980/981原81任务的ID/顺序/勾选及状态/进度保持，原52项AC、66项V和22项SD均保留，各Plan仅增1项AC-HANDOFF及1项SD-HANDOFF；三份0–10章节/围栏、实际承载任务与新增7个本地链接/锚点核对通过，具名IF交接图静态拓扑无环。960归档文件及.next-id内容hash未变。只证明合同结构/交接边界，不是新接口、CPU/AV/A2R/UI/GPU实跑或独立实现review PASS；未创建任何产品用例/产物或实施树。

2026-11-15 stage: new/revision | PLAN-980 r10 | outcome: 规划合同已修订，非实现/产品review PASS。按本轮授权把原960组合缺口交981/SOURCE，980/981原任务生产具名接口，979保留全部原AC/V与自身组合回验；不以A切片归档，不新增整包反向依赖。原任务/勾选/状态/计数保持，旧revision的真实已验部分可按身份复用，但不能证明本次新增交接；未来独立review须绑定r10及实现SHA。仍只规划，next=由原owner获得后续实施指令后按§5–8执行；本记录不恢复任何目标/实施树或解除既有暂停。

2026-11-12 stage:new / PLAN-980 r8：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原35任务/全部AC与V/依赖/阶段/预算和drafting 0/35；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-11 / stage:new / PLAN-980:r7 / outcome:pass（早期Hello路线合同可审阅）；blocked（尚未授权实施，真实能力/接口待交） / next:按§5最小前置进入新增切片；原完整M2任务与终验均保留，不用小例替全包。新增T-HELLO-I, T-HELLO-V, T-HELLO-R, T-HELLO-D，当前0/35。未实现/构建/运行或派agent。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r6：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部31个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r5：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/31；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r4 / stage:new：传统UI完整支持明确进入AGE M2，新增AC-07、V-UI-SOURCE/FULL与SD-03，保原9批31任务/全部原AC/V；DIALECT/STATE供981接口、NATIVE/RUST验纯UI双后端、PARITY核完整分母和原各模式。980不反向等981，不以图形/少量控件替全UI。drafting、0/31；仅规划，实施handoff仍blocked（未授权实施/依赖未齐），非运行或独立final PASS。

本轮静态检查：31任务/16AC/11验证行，原ID全部保留、无重复且0完成。五份相关合同可达199份Plan，无缺失引用/环；批次980/PARITY不等981/LANGUAGE，新链接可解析。没有构建/纯UI运行证据，不记产品PASS。

2026-11-07 r3：细化9批，0/31。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留；原STATIC/TARGETS汇总批进一步逐目标拆成具名I/V/R，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

2026-11-13 / stage:new / PLAN-980 r9 / outcome:pass（规划合同预检，非实现或产品review） / changed: Node首源、Schema/NodeName/兼容与并行合同及对应AC/V/SD；原任务数量/状态/进度与旧义务保留 / next: 本轮仍只规划；后续获得实施指令且该任务最小正式来源齐时进入原work。全部新增验证未执行，未构建/运行/建树/派agent/合入或改963。

2026-11-13 本轮仅规划的静态核验：4份计划原95任务ID/勾选/数量/状态/进度与依赖元数据保持；新增3项AC、7项拟验证及3项拟Spec增量，全部未执行。9份文档本地引用、0–10章节、表格/围栏与新AC/V任务映射核对通过；F0→E0→B0与H0汇合HELLO、随后981按接口扩展的切片顺序不新增反向整包等待。963及其实施树未由本轮修改；未生成fixture/binding/SDK、未运行编译/产品/GPU/性能、未建树/提交/合入/tag。此为规划预检，不是实现独立review PASS。

2026-11-16 stage: work（IF-980-FIELDS 子面）| PLAN-980 r10 | outcome: pass（子面可交付独立复核；整包仍 executing 0/35，B-DIALECT-I/V/R 任务按 §5 r10 不因子面提前勾选）| worktree: D:/autostack/.wt/lang-980/{auto-lang,auto-ui,auto-down(detached),auto-web(detached)} | branches: plan-980-dev | base: lang 5360b8833 / ui d0b60050 | commits: lang bae2343b3、ui 7821410e | 授权：本 /goal（完成计划980/DIALECT）为 r10 §0 交接合同的实施授权。

**交付（B-DIALECT-I/V 子面 = IF-980-FIELDS 可执行清单）。**
- lang/ace-core `source/field_manifest.rs`：`RequiredFieldRow`（r10 §5 冻结语义字段 id/origin/source_span/parent_role/type_rule/applicability/projection_path/check_id/consumer_owner）+ `FieldCoverage{required,present,missing,unknown}` + `field_coverage`（必需行缺席与未知构造双红）+ `validate_field_manifest` + `component_field_ids`（组件面候选→字段级出现集，ID 词汇表由清单 owner 注入，接既有 mapping_complete/component_coverage，无 UI 反向依赖）。
- auto-ui-dialect `p980_fields.rs`：`ui.widget-decl/v1` 清单 110 行（按 ast.rs::WidgetDecl 19 字段 + 全部引用类型逐构造冻结：模块级 10 项、widget 槽、store、view 树 9 节点形态、actions/menubar/toolbar、timer、design、setup/watch/expose、routes/prop/bind、ext_import 5 kind、style-recipe、theme-data）+ 穷尽抽取 `widget_field_occurrences`（view 树带真实字节 span）+ 未登记构造具名拒绝 `ui.field-unknown`（AST 新增字段未登记即红）+ 双向对账 `reconcile_field_manifest`。
- 永久case：lang `tests/cases/plan980_field_manifest.rs` 6 项（清单/覆盖破坏近邻；source→candidate→共享verify→覆盖真实链路；删清单行=unknown 红/换映射=missing 红及恢复绿；AGE 面对 msg/design 未接线成员具名拒绝带 span；legacy-widget 别名闸关闭原 span 拒绝、开启经场景 verify 接受、widget/node 两拼写同 ID 集）；UI `ui_domain_validation.rs` `p980_required_fields_handoff`（002 原源 sha256 前16=04937c35325faa74 冻结→抽取→共享覆盖闭合 16 必需行；槽位计数（msg variant 9=6声明+3内联铸造等）、元素 span 与探针冻结值逐一相等、块锚点顺序、Node 迁移副本仅声明头差且其余字节恒等）+ `p980_required_field_mapping_controls`（删映射项→ui.field-unknown 具名拒绝；删清单行→对账红+unknown 红；同类型非法父槽（theme: 写入 model 块）→解析具名拒绝；修复后同入口绿）。auto-ui-compiler 经 dev-dep 消费 ace-core 共享检查（测试面，生产依赖闭包不变）。

**验证（本轮实际命令/结果）。**
- lang：`cargo t ace-source ace-syntax`（主检出规约的固定组，在 lang-980 树）= PASS：ace-source 603 执行 0 红新增 6 case 0.02s；ace-syntax 155 执行 0 红。
- ui：`cargo test -p auto-ui-dialect --lib` = 105 pass（新增 3，0.02s）；`cargo test -p auto-ui-compiler --test ui_domain_validation` = 5 pass（新增 2，0.14s）。全部新增 case ≤10s/条。
- 取材身份重核：002 app.at sha256=04937c35325faa74…、auto-ui-dialect/src/ast.rs=1d10f5ff2566422e…（与 §4 记录一致）；AGE 面 grammar=age-component profile（node/widget 头同 role，model/on/view 成员；msg/design 等未接线成员在 CST 具名拒绝——完整 DIALECT 继续项）。

**边界（诚实声明）。** 本子面证明：字段分母可执行、双向对账、真实 002 原源穷尽投影计数/span/覆盖与反控、Node 副本字节恒等、widget/node 头 ID 连接（AGE 面经场景 verify）。不证明：002 全源经 AGE 面编译（msg/design/view 树组件未接线——完整 DIALECT 义务保持，B-DIALECT-I 原任务继续）；不替代 B-STATE/NATIVE/RUST/PARITY 真实 UI 义务。统一回归门（merge 冻结态固定组并集）保持 pending，owner=final review/merge。

next_responsible_role: B-DIALECT-R（同树独立核此子面，AC-04 + AC-HANDOFF 复核面；pass 后按 phase fold 正式落主交 979 T-08/T-04）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 1 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: lang bae2343b3 ui 7821410e | next: review

2026-11-16 stage: review | review_scope: phase（IF-980-FIELDS 子面，B-DIALECT-R；整包保持 executing，不勾 B-DIALECT-I/V/R 任务）| reviewer: 同会话独立复核（限制：非独立上下文；结论由工件重构——机械审计/全新语料清扫/命令重放，非采信执行者摘要）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（子面）| worktree: D:/autostack/.wt/lang-980/{auto-lang,auto-ui} @ plan-980-dev | reviewed_commit: lang bae2343b3 / ui 7821410e | base: lang 5360b8833 / ui d0b60050 | deps: auto-down 5af11a39(detached)、auto-web 0adebc03(detached) | spec_inputs: docs/specs/ace-migration/replacement-scope-and-evidence.md（未改）；拟议 SD-HANDOFF/SD-01 增量与交付现实一致（字段级清单/覆盖/反控，不冒完整 DIALECT——980 全义务保持 pending）

**独立复核内容与结果。**
1. 构造键可达性机械审计：P980_CONSTRUCTS 115 键全部存在发射点（清单注册但无 walk 发射=静默缺口的风险核零）；清单行 115 与键集精确双向对等。
2. AST 分母机械复核：WidgetDecl 19/19、StoreDecl 7/7、UiStmt 10/10、ViewNode 8/8 变体全部被 walk 访问（逐字段 grep 审计；store.name 经无条件 push 发射已逐行核）。
3. 全仓真实语料清扫（复核探针，已删）：1592 个 .at（examples/ + crates/**/fixtures，排除 pac.at/target）；697 个可独立解析源全部通过 parse_ui+抽取，**未知构造具名拒绝 0**；895 个多模块上下文文件独立解析失败（"undefined variable"，模块解析语义所致，非本子面缺陷——记录为观察项 O-R1）。48 个构造键在全语料零出现（均对应语料未用的语法：memo/for-key/style-binding/view-fn/theme-doc/menubar 等；store.watch/timer 经 p057_store.at 真实语法验证发射）。
4. 既有 case 重放（本复核会话实际执行）：`cargo test -p ace-core --test conformance p980` = 6 pass 0.02s；`cargo test -p auto-ui-compiler --test ui_domain_validation` = 5 pass 0.14s；`cargo test -p auto-ui-dialect --lib` = 105 pass 0.02s。
5. 健康检查：两仓提交后工作树 clean；我的文件 fmt clean；无 println!/dbg!；两 crate 零新增 warning；rustfmt 模块递归误格式化的 13 个非本任务文件已还原（diff 仅本任务 4+5 文件）。
6. 交接准入对账（979 r11 §5）：repo/已落主 SHA=本 phase fold 落主后记录；Plan revision+子面范围 ✓（§9 work 收据）；普通源闭包 hash ✓（002 sha16=04937c35325faa74 冻结）；grammar/domain/compat ✓（age-component profile、node/widget 同 role、别名闸两侧验证）；模块/定义身份+原 span ✓（元素 span 冻结值逐一相等、块锚点顺序）；Checked 消费接口 ✓（compile_component_scenario_observing 共享 verify 链）；永久 case/独立 oracle/失败恢复 ✓。

**acceptance 结果（子面映射）。** AC-HANDOFF 子面要求（覆盖完整 WidgetDecl/引用构造分母、实际覆盖/拒绝/删映射/删清单行反控、来源正式可供 979 回验、只消费已交 979 接口）= pass（证据如上 1–4）；AC-B-DIALECT 完整批（旧源经同一 AGE Schema 兼容别名的全量投影/绑定/错误 span）保持 open——本子面证明字段分母与覆盖机制，不冒完整批。AC-04 的本批复核面 = pass（本记录）。

**观察项（非阻断，不改验收）。** O-R1：parse_ui 单文件语义使多模块源独立解析失败——真实消费走工程级入口（generate_component_from_file 等），后续完整 DIALECT 批的语料验证应按工程闭包跑。O-R2：`view.event.inline`/`view.node.text.interp` 行在 parse_ui 输出上不可发射（内联 lambda 铸造为 __evt handler、插值文本归一为元素 prop——行内已注记 Optional+铸造去向；源语法形态仍属分母）。

**新增 case 行为/耗时表。** lang 6 case：清单/覆盖破坏近邻、真实链路、删行/换映射反控、未知成员拒绝、别名闸/双头连接——合计 0.02s；UI 2 case：交接（002 真源覆盖+计数+span 冻结值+锚点序+Node 副本字节恒等）与反控（删映射/删行/非法父槽/恢复）——文件 5 case 合计 0.14s；dialect lib 3 case 0.02s。全部 ≤10s/条；无测试内编译前置（ace-core 为显式 path dev-dep，常规编译）。

pending_closeout_gates: 统一回归门（冻结固定组并集：lang ace-source/ace-syntax 已在本基线全绿，落主后 merge 按冻结 commit 复核；UI 面 ui_domain_validation/dialect lib 同基线绿）；完整 DIALECT 及 980 全义务继续在原任务。
findings: 无阻断缺陷；O-R1/O-R2 观察项如上。
next: merge（phase fold 正式落主 + 979 T-08/T-04 交接记录；唯一树保留续原任务）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 2 | from: review | to: merge | status: executing | outcome: pass（phase）| worktree: D:/autostack/.wt/lang-980 | commit: lang bae2343b3 ui 7821410e | next: merge

2026-11-16 stage: merge（phase fold，IF-980-FIELDS 子面正式落主；整包 executing 保持，唯一树保留续原任务，不归档/不清树）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（子面交付）| delivery_commit: auto-lang master 46f4ac0ec（代码 7f9d03bf6 + 规范 46f4ac0ec）、auto-ui master f864f33b | canonical: docs/specs/ace-migration/m2-autoui.md（master 上 sha256/16=75fa7b337182c6e1，SD-01/SD-HANDOFF 子面增量）| 落主方式：rebase 到在途 master（lang cdfcc8adf→aa96ba846 两跳、ui ff7e9761→7827316c 两跳；range-diff 全等=安全改写证明；无冲突）+ wt-guard clean + `git merge --ff-only`（无 merge commit）。

**落地后复核门（merge-owned，冻结基线实际执行）。** lang：`cargo t ace-source ace-syntax` = PASS 603+155 执行 0 红（rebase 至 cdfcc8adf 基线；其后再落 docs-only 提交不改代码面，p980 过滤 6 pass 复核）；ui：`cargo test -p auto-ui-dialect --lib` = 105 pass、`--test ui_domain_validation` = 5 pass（rebase 至 7827316c 后重跑，PLAN-085 代码已并入）。主检出落主后 `git status` 代码路径 clean（docs WIP 为他节会话簿记，保留）。

**IF-980-FIELDS 正式交接记录（交 979 T-08/T-04，r11 §5 交接准入逐项）。**
- repo/已落主 SHA：auto-lang 46f4ac0ec、auto-ui f864f33b（均 fast-forward 落主，无分支 tip）。
- 生产 Plan revision 与子面范围：PLAN-980 r10 §5「r10：向979交付完整字段事实」——字段定义/覆盖子面；完整 DIALECT/980 各批不含。
- 普通源闭包 hash：examples/ui/002-counter/src/front/app.at sha256/16=04937c35325faa74（UI 侧 p980_required_fields_handoff 冻结断言）；分母 AST 身份 auto-ui-dialect/src/ast.rs sha256/16=1d10f5ff2566422e。
- grammar/domain/compat：age-component 裁剪面（auto-source.at）；领域清单 ui.widget-decl/v1；legacy-widget 别名闸两侧已验（关闭=原 span 拒绝 source.age.legacy-widget；开启=场景 verify 接受）；node/widget 同 role 同 ID 集。
- 模块/定义身份与原 span：元素 span 冻结值逐一断言（col (917,727) 等 10 项）；块锚点顺序断言；AST 无 span 槽位以独立定位器/计数 oracle 核。
- Checked 消费接口：compile_component_scenario_observing 共享 verify 链（lang 永久 case 固定）；field_coverage/validate_field_manifest/ComponentFieldVocab。
- 永久 case/独立 oracle/失败恢复：lang plan980_field_manifest.rs 6 项；ui p980_required_fields_handoff / p980_required_field_mapping_controls；删映射/删清单行/非法父槽/恢复控制各自可区分。
- 后端/工具链/预算：CPU 面（无窗口/GPU）；固定组 ace-source/ace-syntax；UI 单文件 parse 面（O-R1 边界在规范 §4）。
- 剩余完整任务：B-DIALECT-I/V/R 原任务及 §5 全部批次义务保持未勾选，唯一树 D:/autostack/.wt/lang-980/ 保留。

canonical_verified: docs/specs/ace-migration/m2-autoui.md @46f4ac0ec（上述 hash）；未做任何 index/legacy ledger 写入。plan-artifacts/plan-980/ 保持 in-flight（整包在途；登记表 plan_status 已同步 executing/子面已落主）。next_responsible_role: work（B-DIALECT 完整义务继续；979 T-08/T-04 消费本记录回验）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 3 | from: merge | to: work | status: executing | outcome: pass（phase fold 落主）| worktree: D:/autostack/.wt/lang-980 | commit: lang 46f4ac0ec ui f864f33b | next: work（完整 DIALECT 继续）

2026-11-16 stage: work（B-DIALECT-I/V 完整批）| PLAN-980 r10 | outcome: pass（完整批可交付独立复核；前门短窗单写权经核取用：979 树 clean/分支已合/D-979 等待）| worktree: D:/autostack/.wt/lang-980/auto-lang @ plan-980-dev | commit: 2898f689b | base: 6a1ad25eb | 变更面：auto-source.at（age-component 门控新规则+槽位名，token 表零新增→其余 profile GrammarId 不变）、text-record.atom（附加 text_display/fstr case）、age_component/fstr/project/field_manifest、永久 case plan980_dialect_full.rs（新）+plan980_field_manifest/plan979_component_profile（演进更新）。

**交付（AC-B-DIALECT 主体）。** 002-counter 完整原 widget 源经同一 AGE Schema legacy-widget 别名**全量投影 + 共享 verify**：msg 6 变体/model 3 var 字段(int/str/bool)/design 2 条目(theme 绑定表达式；mode=if/else 结构性锚定)/widget 形 on 6 handler(复合赋值脱糖)/view 树(col>text fstring+2×row+6 button+style 行)+use themes 导入；场景编译（2 实例定向事件）经唯一 verify 接受（含反引号 f-string `.count` 绑定经 __fstr 链、str 状态字段、`!` 一元）；别名关闭=声明头原 span 具名拒；规范 Node 副本同 ID 集同结构。001-helloworld 冻结副本（含 selectable/style 带属性 text 形）全量结构投影。字段级覆盖（全槽位 vocab）闭合 002 必需面；计数/结构/删行反控独立 oracle。V-B-DIALECT：`cargo t ace-source ace-syntax`=PASS 607+155 执行 0 红；UI 腿复跑（代码未变）ui_domain_validation 5/5、dialect lib 105/105；p980 11 case 全绿（≤0.05s/条）。

**边界（诚实声明）。** ①4 槽位（style/setup/actions/ext_imports）为成员 span 具名拒绝（KNOWN-DEBT 已登记，关闭条件在案）；②design 的 if/else 值与 design 求值为结构锚定（行为 owner=PARITY/STATE）；③f-string 内部 span 移位沿 PLAN-964 边界（外部字节精确）；④场景观测仍限 int 字段（001 无 int 字段→场景腿不适用，已注记）；⑤959 行/组件全模式分母与 V-UI-FULL 归 T-01/PARITY 原任务。979 既有反例 unknown-attribute 按合同演进改无冒号形（`text <label> { 属性行 }` 已为 001 真实语法；979 文件内 PLAN-980 注记）。

next_responsible_role: B-DIALECT-R（完整批独立核验；pass 后勾选 B-DIALECT-I/V/R、current_step=3/35、phase fold #2 落主）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 4 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: 2898f689b | next: review

2026-11-16 stage: review（B-DIALECT-R 完整批终核）| review_scope: batch-final | reviewer: 同会话独立复核（限制：非独立上下文；结论由工件重构——复核反例两族、门禁重放、机械审计，非采信执行者摘要）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次闭合；B-DIALECT-I/V/R 勾选，3/35）| reviewed_commit: 2898f689b + fcd7861dc（复核反例）+ 规范提交（落主时基线再核）| worktree: lang-980 @ plan-980-dev

**复核内容与结果。** ①AC-B-DIALECT 逐句：普通标识符上下文保持=复核新增永久 case（msg/design/bind 作字段名/局部名）；旧 widget 原源经同一 AGE Schema 兼容别名解析/绑定/错误 span 正确=002 冻结副本全量投影+共享场景 verify（f-string `.count` 绑定、str 字段、复合赋值、`!`、design 表达式）+span oracle；规范 Node 同 body/组件行为=副本同 ID 集+同结构；坏字段拒绝/缺字段不静默=坏形状原位具名拒（复核新增 4 形）+deferred 槽位拒+删行反控。②门禁：`cargo t ace-source ace-syntax`=PASS 607+155 执行 0 红（fcd7861dc 冻结码）；UI 腿复跑 5+105 绿；p980 13 case 全绿 ≤0.05s/条。③跨 profile 安全：新规则/槽位名全 age-component 门控、token 表零新增；927 全 profile 差分语料（ace-source 组内）绿。④合同演进核对：979 unknown-attribute 反例改无冒号形+文件内注记；text-record schema 附加 case（binder/verify 共享实现）。⑤债务诚实：4 槽位具名拒绝入 KNOWN-DEBT（owner/关闭条件）；design if/else 结构锚定；f-string 内 span 移位边界；001 场景腿不适用（无 int 观测字段，注记）。⑥健康：fmt 零偏、零新增 warning、无调试打印；979 文件改动仅 1 反例源+注记。

**新增 case 行为/耗时（复核 authored）。** 标识符上下文（字段名/局部名两形）0.02s；坏形状 4 形（design 缺冒号/on 缺箭头/msg 悬空逗号/元素未闭合）0.02s。

next_responsible_role: merge（phase fold #2 落主 fcd7861dc+规范；随后 985/后续批消费）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 5 | from: review | to: merge | status: executing（批次后 3/35）| outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: 2898f689b fcd7861dc | next: merge

2026-11-16 stage: merge（phase fold #2，DIALECT 批正式落主；整包 executing 3/35，唯一树保留续 STATE/NATIVE 等批）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次交付）| delivery_commit: auto-lang master 3de0bea0b（代码 b08b3af5f + 复核反例 ef2bb5ad1 + 规范 3de0bea0b）| canonical: docs/specs/ace-migration/m2-autoui.md（§4 增补 AGE 组件面 widget 成员接线全节；边界 §5 刷新）| 落主方式：rebase 到在途 master 0268b563c（docs-only 跳，代码面不变；p980 13/13 复核）+ wt-guard clean + `git merge --ff-only`（无 merge commit）。UI 仓本轮零改动（f864f33b 基线继续有效）。

**门禁（merge-owned，冻结基线）。** `cargo t ace-source ace-syntax` = PASS 607+155 执行 0 红（ef2bb5ad1/2898f689b 冻结码，落主前实测）；UI 腿 ui_domain_validation 5/5 + dialect lib 105/105（f864f33b 未变代码复跑）。统一回归门（后续批冻结时并入）保持 pending 记录。

**批次闭合。** B-DIALECT-I/V/R 完整义务核验通过并勾选（3/35）：002-counter 完整原 widget 源经同一 AGE Schema legacy-widget 别名全量投影+共享 verify 成立；规范 Node 副本同 ID 集；坏字段/缺字段/未接线槽位原位具名拒；4 槽位（style/setup/actions/ext_imports）具名拒绝入 KNOWN-DEBT（owner=980 后续轮，关闭条件在案）；design if/else 与求值行为结构锚定（行为 owner=PARITY/STATE）。979 前门短窗单写权经核取用（979 clean/已合/等待）并于落主释放——窗口事实与本批演进（text 三形、979 反例更新）均在两仓文件内注记。

next_responsible_role: work（980 剩余批次按 §5 前置继续：STATE→NATIVE→…；985/A 消费 DIALECT 接口）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 6 | from: merge | to: work | status: executing（3/35，DIALECT 批已落主）| outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: lang 3de0bea0b | next: done（目标范围=980/DIALECT 批次已交付）

2026-11-16 stage: goal（目标完成核验轮，verify-and-route，无阶段施工）| PLAN-980 r10 | 目标「完成计划980/DIALECT」= DIALECT 批次按完整口径交付并落主。磁盘级核验（非采信收据）：①落主身份——lang master 含 fold #1 46f4ac0ec 与 fold #2 3de0bea0b（b08b3af5f 代码→ef2bb5ad1 复核反例→3de0bea0b 规范，merge-base 祖先核验通过，fast-forward 无 merge commit）；auto-ui master 含 f864f33b。②规范沉淀——master 上 docs/specs/ace-migration/m2-autoui.md Status=DIALECT 批已验、§4「AGE 组件面的 widget 成员接线（DIALECT 批，2026-11-16）」在库。③永久 case——experimental/ace/crates/core/tests/cases/{plan980_dialect_full,plan980_field_manifest}.rs 均在 master 树。④簿记——计划收据（round 1–6）已由 3c902bc93/6a1ad25eb 入库；frontmatter executing 3/35，B-DIALECT-I/V/R 勾选与交付范围一致。⑤唯一树——lang-980 clean @ plan-980-dev 3de0bea0b，按 §5 phase fold 合同保留续 STATE/NATIVE 等批（不删）。⑥主检出代码路径 clean（docs WIP 为他节会话簿记）。门禁证据绑定冻结基线（ace-source 607+155、UI 5+105，round 5/6 实测），身份未变按裁定复用，不重复执行。结论：目标范围已达成且经磁盘核验；980 整包（32/35 余量）不在本目标内，后续批次由 work 续行。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 7 | from: verify | to: done | status: executing（3/35，DIALECT 批已交付落主）| outcome: pass | worktree: D:/autostack/.wt/lang-980（保留）| commit: lang 3de0bea0b ui f864f33b | next: done

本轮规划静态预检：当前任务ID/总数、原已完成项与状态保持一致；仅977撤出GD/SCENE六项、982撤出GODOT三项，原条目及AC/V留§9历史；0–10章节、代码围栏和本轮新增本地链接已核。归档959、`.next-id`及两份取材canonical Spec的SHA256与读前一致。本记录不代实施/运行或独立产品复审，新增V行仍未执行。

2026-11-16 stage: work（B-STATE-I/V 批，目标链 round 8：按用户裁定顺序 STATE→HELLO→RUST→JET 续行）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次可交付独立复核；B-STATE-I/V/R 勾选留 R，current_step 仍 3/35）| worktree: D:/autostack/.wt/lang-980/{auto-lang,auto-ui} @ plan-980-dev | base: lang 925aeb4fb / ui c15fe0f5（双双 resync 后开工）| code_commit: lang 09a6274af、ui a6b7fb4c | 授权：本 /goal 链（完成计划980 剩余就绪批）。

**交付（AC-B-STATE 主体：用 ACE 执行服务替旧 handler bytecode 的状态执行面）。**
- lang/ace-core `source/age_component.rs`：`ComponentInitials`/`InitialValue`（props 种子，先验后用：未声明字段 `component-unknown-field`/类型不匹配 `component-unsupported`/超 i32 域/单实例重复 `component-duplicate`/覆盖数错位 `component-scenario`）+ **states-only 场景编译** `compile_component_scenario_states`（不发射文本叶函数；f-string 叶的 `TextDisplay` 在 av.owned/1 车道不可 lowering，状态通道不依赖文本叶）+ `ComponentStateOut`（每(实例,字段)状态函数出口，def id 按共享 adapter `free_fn_def_id` 约定）。
- lang/ace-av `age_component.rs`：`observe_component_states`/`observe_component_scenario_states`（int/bool/str 逐字段真值；执行器重构为通用 `execute_value`）+ 消费者再导出（ComponentScenario/InitialValue/LegacyWidgetAlias/编译入口，UI 桥不直依 ace-core）+ `scheduler_run.rs`（`run_task_source`：968 调度器面一次性真实执行入口，UI 异步宿主）。
- auto-ui-runtime `ui/ace_state_bridge.rs`（新，`ace-av` path 生产依赖）：`AceStateEngine`——keyed mount/unmount（世代递增）、晚到事件丢弃（UnknownIdentity/InstanceReleased 可区分）、重挂全新、dispatch 失败回滚（引擎不毒化）、props=mount 初值、`spawn_async`/`await_async`（ticket 钉世代；任务先在 ACE scheduler 真实执行到静止，再按世代纪律回写：StaleGeneration 丢弃对齐 CompletionPorts DropLate）。**边界具名**：多单元源（host/store 内嵌跨引用）等 981/IF-981-MODULE；旧 bytecode 路径全面退役归 987；本批不改 DynamicComponent 默认链。
- 永久 case：lang `av/tests/cases/plan980_state_face.rs` 6 项（初值种子+定向 ops、逐字段观察、五类初值拒绝、字段视图与打包驱动一致、scheduler 真实执行保序、**f-string 叶边界**——states-only 可观察/完整文本车道拒绝 `text-display` 具名）；UI `plan041_instance_ownership.rs` 追加 `p980_ace_state` 4 项（双 App 双实例隔离含 str 字段、卸载晚到丢弃/存活者续行/重挂新鲜、**MIX-03 无 HTTP 版**——两实例异步、先关一例、完成只写存活、旧世代 StaleGeneration 隔离、新世代正常回写、store 形 handler 经 ACE+坏消息具名拒+回滚后续用）。

**验证（冻结提交实测；会话测试表）。** lang `cargo t ace-source ace-av ace-host`（union wall 831.6s+准备含编译，发现/运行/清理分列未观测具名标注）：ace-source **PASS 610 执行 0 红**、ace-av **PASS 1263 执行 0 红**、ace-host 166 执行 153 过/**13 红=master 预存红**——同测试（`abi_recipe_binding_exclusivity` 等）在主检出 master `2cdb28fb3`（无 980 代码）逐一复现同错（`unknown ABI leg "z"` 等；PLAN-973 归档收据同日具名 2 组 master 预存红），台账记预存、不判 980、归批量回归 triage。UI：`cargo test -p auto-ui-runtime --test plan041_instance_ownership --test plan033_b1_scope_isolation -- --test-threads=1` = plan041 **8/8**（0.63s，旧 4 不动+新 4）、plan033 **2/2**（1.41s）。新增 case 单条 ≤10s 门：lang 6 条合计 0.03s、UI 4 条包含于 0.63s 文件总时长（单条 ≪10s）。占机纪律：主检出 stdlib/ace 并集（PLAN-973 线）运行期间等待至空闲（后台看门狗，~13min）后才跑组门。

**环境事件（诚实记录）。** ①D: 磁盘一度满（os error 112，UI 树全量重构建）——清理**本组** lang-980/auto-ui incremental（8GB，可再生），他会话随后释放至 122.7GB；ui-098（executing+未提交 WIP）与主检出 target 未动。②rustfmt 经 ui/mod.rs 递归格式化 68 个非本任务文件——已全部 `git checkout` 还原，两树终态仅本任务文件（lang 6/ui 5）。③临时 HIR 探针（诊断 text-display 根因）已删净。

**边界（诚实声明）。** 本批证明：组件形源（002 真源）的 model/msg/on 状态-事件面经 ACE 组件面+AV 真值执行、实例生命周期/世代纪律/异步完成世代隔离/store 形 handler、双 App 隔离。不证明：多单元源内嵌跨引用接线（981）、f-string 文本叶 AV 执行（states-only 规避、文本车道具名拒——建议 KNOWN-DEBT 登记，R 裁定）、DynamicComponent 默认链切换与窗口行为（NATIVE/PARITY 义务保持）、HTTP 实腿（983/联验）。

next_responsible_role: B-STATE-R（同树独立核本批：AC-B-STATE 逐句/反控重放/边界与债务核对；pass 后勾选 B-STATE-I/V/R、current_step=6/35、phase fold #3 落主）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 8 | from: work | to: review | status: executing（3/35，STATE 批待复核）| outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: lang 09a6274af ui a6b7fb4c | next: review（B-STATE-R）
2026-11-16 stage: review（B-STATE-R 批次终核）| review_scope: batch-final | reviewer: 同会话独立复核（限制：非独立上下文；结论由工件重构——冻结树核对、p980 过滤独立重放、复核反例两族、机械审计，非采信执行者摘要）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次闭合；B-STATE-I/V/R 勾选，6/35）| reviewed_commit: lang 09a6274af+d43eb9618（复核反例）/ ui a6b7fb4c+94b9334b（复核反例）| base: lang 925aeb4fb / ui c15fe0f5 | worktree: lang-980 @ plan-980-dev

**复核内容与结果。** ①冻结核对：两树 clean 于复审提交，无未审漂移。②AC-B-STATE 逐句：双App各两组件不串=双引擎×keyed 实例 case（int/str 字段、跨引擎零影响）；异步任务返回只写仍存活实例=MIX-03 无 HTTP case（先关一例→完成只写存活、关闭不复活、旧世代 StaleGeneration 丢弃、新世代回写）；真实异步host=ACE scheduler 968 面（stdout 保序 case）；HTTP 实腿归 983。③门禁独立重放：work 冻结提交上 `cargo t ace-source ace-av ace-host`（union 831.6s）= ace-source **PASS 610 执行 0 红**、ace-av **PASS 1263 执行 0 红**、ace-host 153/166（13 红=master 预存红——主检出 master `2cdb28fb3` 无 980 代码逐一复现同错，如 `unknown ABI leg "z"`；与 973 登记的预存红同类，已入 KNOWN-DEBT，owner=批量回归 triage）；UI plan041 8/8+plan033 2/2（work 态）；复审态终验 av p980_state **7/7**（0.101s）、UI plan041 **9/9**（0.34s）+plan033 2/2（0.17s）。④复核反例（authored，永久）：lang `p980_state_states_only_emits_no_text_fns_in_artifact`（states-only 工件级无 `__age_c_text`、description 无文本行、完整编译含之——删发射门控即红）；UI `p980_state_async_task_failure_surfaces_not_fabricated`（坏任务源在 ACE 段具名拒 `bind/source.sched-task` → await_async Err、状态未被写——完成不可伪造；首版断言串过窄系探针缺陷，原始记录保留并修正）。⑤机械审计：review 增量仅 2 测试文件；零台账/legacy 写入；core 改动全部封装于组件面编译入口内（ace-source 610 含全部 p979/p980 core case 绿证跨 profile 安全）。⑥债务诚实：f-string 文本叶 av.owned/1 车道、DynamicComponent 默认链未切（退役归 987）、ace-host 预存红、**emit 子项边界（复核新增具名——跨组件路由归 981/IF-981-MODULE，此前收据漏具名）**全部入 KNOWN-DEBT（7a3fc610e/18701b519）。⑦健康：新增文件 fmt clean、零新增 warning、无调试输出。

**新增 case 行为/耗时（authored）。** states-only 工件断言（av，含编译+双编译对照）文件 7 条合计 0.10s；异步失败不伪造（UI，含坏源编译+引擎回滚核对）文件 9 条合计 0.34s。全部 ≪10s。

**规范增量核验（merge 落）。** 拟 SD-STATE：m2-autoui.md 新 §6（组件面状态执行服务：states-only/initials/逐字段观察/scheduler 入口/AceStateEngine 生命周期与世代纪律/MIX-03 无 HTTP 已验面）+ §5 边界刷新（f-string 文本叶车道/emit 归 981/默认链并行）——与交付现实一致，不冒 NATIVE/PARITY。

pending_closeout_gates: 统一回归门（merge 冻结态：lang ace-source+ace-av 复核于落主基线，ace-host 预存红具名排除；UI 两文件同基线）。findings: 无阻断缺陷；emit 具名边界补登记（非缺陷，原收据漏项）。next: merge（phase fold #3 落主 lang d43eb9618/ui 94b9334b + 规范 §6；NATIVE 前置之一 STATE 已就绪）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 9 | from: review | to: merge | status: executing（6/35，STATE 批已复核）| outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: lang d43eb9618 ui 94b9334b | next: merge
2026-11-16 stage: merge（phase fold #3，STATE 批正式落主；整包 executing 6/35 保持，唯一树保留续 HELLO/RUST/JET 等批，不归档/不清树）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次交付）| delivery_commit: auto-lang master **fc5610783**（代码 9bba65eaf + 复核反例 6fc7da3db + 规范 fc5610783）、auto-ui master **ae1c53ad**（代码 fd7861ed + 复核反例 ae1c53ad）| canonical: docs/specs/ace-migration/m2-autoui.md（master 上 sha256/16=**e3d600d851923fd3**，新 §6 组件面状态执行服务 + §5 边界刷新）| 落主方式：两仓 rebase 到在途 master（lang 1f1ff700e——PLAN-973 55 文件代码并入、ui c4bdd83c——PLAN-098 窗口修复并入）+ range-diff 全等（lang 3/3 `=`：09a6274af→9bba65eaf / d43eb9618→6fc7da3db / 0a7bbfd90→fc5610783；ui 2/2 `=`：a6b7fb4c→fd7861ed / 94b9334b→ae1c53ad；无冲突）+ wt-guard clean + `git merge --ff-only`（无 merge commit）。

**落地后复核门（merge-owned，rebase 基线实测——master 代码面已动故按规矩重跑，不复用旧 union）。** lang `cargo t ace-source ace-av`（worktree@fc5610783 内容，全量日志 plan-artifacts/plan-980/state-fold3-union.log）= **ace-source PASS 610 执行 0 红 / ace-av PASS 1279 执行 0 红**（含 973 新 sys case + 本批 7 项 p980_state），union wall 607.5s；主检出落主后 p980_state 过滤 **7/7**（0.064s）。ace-host 13 红=master 预存红（KNOWN-DEBT 台账，非本批触面）。UI：rebase 合并态（=落主内容）plan041 **9/9**（0.34s）+ plan033 **2/2**（0.17s）。

**批次闭合。** B-STATE-I/V/R 勾选（6/35）：002 真源经组件面 states-only/初值/逐字段观察/scheduler 入口 + AceStateEngine 生命周期世代纪律成立；MIX-03 无 HTTP 版已验（真实异步宿主=968 调度器，完成只写存活实例，世代隔离）；边界（f-string 文本叶 AV 车道/emit 归 981/默认链未切归 987/ace-host 预存红）全部 KNOWN-DEBT 在案（7a3fc610e/18701b519）。plan-artifacts/plan-980/ 保持 in-flight（整包在途；fold#3 union 日志在 scope）。批量回归 due-check 归最终归档收口（980%5=0，届时触发）。

next_responsible_role: work（980/HELLO 批：967/F0-D、979/H0-D、969/E0-D、978/B0-D 四正式来源均已落主，前置齐；T-HELLO-I 冻结首源/Schema 激活与 ace-hello-minimal fixture）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 10 | from: merge | to: work | status: executing（6/35，STATE 批已交付落主）| outcome: pass（phase fold #3）| worktree: D:/autostack/.wt/lang-980（保留）| commit: lang fc5610783 ui ae1c53ad | next: work（HELLO 批）
2026-11-16 stage: work（T-HELLO-I/V 批，round 11：按裁定顺序 HELLO）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次可交付独立复核；T-HELLO-I/V/R 勾选留 R）| worktree: D:/autostack/.wt/lang-980/{auto-lang,auto-ui} @ plan-980-dev | base: lang fc5610783 / ui ae1c53ad | code_commit: lang b3887f887、ui 3e85a7f1。

**T-HELLO-I 有界决定（两机制比较，冻结）。** 选 `run_app::<C>` 泛型单窗宿主（devtools 链：View→rewrite_viewport_units→into_iced、Inter 字体、主题服务；ui_hello_loader 等 20+ example 同链）——组件零旧链依赖；弃 daemon `run_session`（`AppSession` 硬绑 `DynamicComponent`/旧 bytecode 链，接入需造假 VmBridge，合同明禁）。无第二解析器；中立宿主接面无需新抽（run_app 泛型即现成接面）。

**交付。** ①lang：F0 构建新鲜度缓存（`rust_provider::build_source` 按 (provider digest, shim digest, rustc identity)+双产物在位跳过 rustc；篡改/缺产物/digest 变化即重建；永久 case `plan980_f0_cache.rs` 2 项）+ **ace-host Exit 臂修复**（973 给 ExecOutcome 增 `Exit` 后本 crate 匹配面非穷尽=master 预存编译红，3 处具名映射：test failed/project av.exit/session.eval/exit）。②auto-ui：fixture `examples/ui/ace-hello-minimal/src/front/{app,app-body}.at`（node Hello 静态 text + 普通函数返回中文/ASCII）；`ui/ace_hello.rs` 桥（H0 面共享 verify→AV 真值文本，复制为 UI 所有 String；F0 腿合成 TextRecord 源经 969 E0 门面分派——同一文本字节由 `text_byte_len` 证实被 provider 收到；幂等注册）+ `AceHelloWindow` 组件（OnceLock 会话注入）+ `examples/ace_hello_window.rs`（真 Iced 主线程窗口；--prep-only 预构建模式；帧计数/证据 JSON/自动退出，Process.exit 语义同 plan804）；`tests/ace_ui_hello.rs` 5 项永久 case。③依赖：runtime+dev-deps 增 ace-av/auto-lang-ace-host/auto-lang-core/serde_json（生产闭包含 ACE——HELLO 即该集成）。

**V-HELLO 验证（冻结提交实测）。** CPU 面（恒跑）：SCHEMA（root=node/grammar 身份/任意 Node 名与文本变体/无 AGE 具名拒绝）、BODY（函数返回值改文随变）、REJECT（int 文本叶 source.age 拒；错 provider 名 ace.provider/unknown 且零 dispatch）、LIFE（20 轮装载/AV/F0 分派/释放 + teardown 旧名直连门面具名失效 + 重注册新代恢复）。窗口+FFI 面（`AUTO_UI_ACE_HELLO_E2E=1` 显式排窗）：prep（build example + exe --prep-only 预构建 provider 到 target/ace-hello-f0 缓存 stamp）后子进程真窗 × 2 fixture（静态 Hello 5 字节 / 你好 Auto 11 字节），断言 AV 文本=源字节、F0 byte_len=AV 字节数、callee_count=1/rejected=0、frames≥2、任意 cwd（仓外异根）。命令冻结：`cargo test -p auto-ui-runtime --test ace_ui_hello -- --test-threads=1`（窗口面先 prep 再带 env）。

**会话测试表。** UI ace_ui_hello 全 5/5=4.32s（窗口腿 3.09s；CPU 5 项 1.95s；单条最大 e2e 3.09s ≤10s）；lang `cargo t ace-abi ace-facade`：**ace-facade PASS**（Exit 修复实证）；ace-abi 272/296——24 红全在 `source_foreign` 族=master 预存红（主检出 `8e0c4d1d6` 同红复现；973 台账原记 1 例、975 落主后扩大；已入 KNOWN-DEBT，非本批触面），本批 2 个 f0 缓存 case 组内绿。

**边界（诚实）。** V-HELLO-PACK 最小面交付（仓外 cwd/预构建零 rustc/缺前置具名拒绝）；完整 B0 打包闭包（发行包形态）具名 pending（owner=后续 PACK/986 面）。V-HELLO-LIFE 以 20 轮 CPU 生命周期+teardown/新代 + 每证据 1 真窗轮实现（进程内二次 iced 事件环被 PLAN-698 禁止→多窗须进程级；测试内 RSS 断言不稳定，具名）。AXR0 同源 A2R 腿归 RUST 批并行（合同原文）。F0 面不返回文本（967 冻结 provider 计算文本事实）——窗口文本=AV 真值，F0 证明同字节抵达，二者分立为已交付接口的现实映射。

next_responsible_role: T-HELLO-R（同树独立核：AC-HELLO/AC-NODE-COMPAT 面、真窗链、反控、边界与债务；pass 后勾 T-HELLO-I/V/R=9/35、fold #4 落主）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 11 | from: work | to: review | status: executing（6/35，HELLO 批待复核）| outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: lang b3887f887 ui 3e85a7f1 | next: review（T-HELLO-R）
2026-11-16 stage: review（T-HELLO-R 批次终核）| review_scope: batch-final | reviewer: 同会话独立复核（限制：非独立上下文；结论由工件重构——冻结树重放、复核反例、机械审计）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次闭合；T-HELLO-I/V/R 勾选，9/35）| reviewed_commit: lang b3887f887 / ui 3e85a7f1 + 复核修复 e529dde6 | worktree: lang-980 @ plan-980-dev

**复核内容与结果。** ①冻结核对+重放：UI ace_ui_hello 5/5@3e85a7f1（含窗口面）；lang abi p980_f0_cache 2/2 + e0_facade 3/3 + `cargo t ace-facade` PASS@b3887f887。②AC-HELLO/AC-NODE-COMPAT 面：正式输入 root=node（SCHEMA case）、AGE Schema/grammar 身份、任意 Node/函数/文本变体、坏 Schema/int 文本叶 FFI 前具名拒、错 provider 零 dispatch、20 轮生命周期+teardown/新代、真窗 e2e（AV 文本=源字节、F0 byte_len=字节数、callee=1/rejected=0、frames≥2、仓外 cwd）。③**复核反例（authored，两族）**：`p980_hello_f0_bytes_drift_must_refuse` ——**发现真缺陷**：run_f0 字节等式两侧同源（快照 vs 由快照生成载荷）恒真；work 修复 e529dde6（分派时重观察，快照漂移具名红——等式两侧自此不同源），修复后 7/7 绿；`p980_hello_window_exe_bad_source_exits_named`（坏源 exit 4、无证据文件——证据不可伪造）。④机械审计：review+修复增量仅桥/测试；零台账写入；lang 面改动（abi 缓存/ace-host Exit 臂）均为加法，ace-facade 组 PASS 实证。⑤边界核对：PACK 最小面/LIFE 20CPU+1真窗适配/AXR0 归 RUST 均已在案；ace-abi source_foreign 预存红（KNOWN-DEBT）非触面。⑥健康：fmt clean、零新增 warning、无调试输出。

**新增 case 行为/耗时（authored）。** 漂移拒绝（含双观察+分派）≤2.17s 文件 7 项合计；坏源退出码（子进程 spawn+退出）同文件内。终态全量 7/7=4.91s（e2e 窗口腿 ~3.1s 最大，≤10s）。

pending_closeout_gates: 统一回归门（merge 冻结态：lang ace-abi+ace-facade 于落主基线，source_foreign 预存红具名；UI ace_ui_hello 全量）。findings: F1（漂移恒真，已修复闭环）。next: merge（T-HELLO-D fold #4：lang b3887f887/ui e529dde6 + 规范 §7 HELLO 节；不改默认、不归档）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 12 | from: review | to: merge | status: executing（9/35，HELLO 批已复核）| outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: lang b3887f887 ui e529dde6 | next: merge（T-HELLO-D）
2026-11-16 stage: merge（T-HELLO-D，phase fold #4，HELLO 批正式落主；整包 executing 9/35 保持，唯一树保留续 RUST/JET，不归档/不清树、不改产品默认）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次交付）| delivery_commit: auto-lang master **1991021a5**（代码 dd27cdf0c→50d0dcce4 + 规范 098d0f4a3→1991021a5，两次 rebase：先 onto 8e0c4d1d6（975 B1），FF 拒绝后再 onto 57fdfe6f8（981 SOURCE 落主），range-diff 全等 2/2 `=`）、auto-ui master **e529dde6**（0 behind 直落）| canonical: docs/specs/ace-migration/m2-autoui.md（master sha256/16=**3e92ba6f3df309b1**，新 §7 首窗链/桥/F0 缓存/排窗纪律）| 落主方式：wt-guard clean + `git merge --ff-only`（无 merge commit）。

**落地后复核门（merge-owned）。** lang rebase 基线（975 B1 后）：`cargo t ace-abi ace-facade` = **ace-facade PASS**；ace-abi 273/298——25 红全为 `source_foreign` 族 master 预存红（已入 KNOWN-DEBT，主检出同红复现在案），本批 p980_f0_cache 2/2 组内绿（re-rebase 后单跑复核 2/2）。981 SOURCE 二次落主后再核 ace-facade=PASS（av crate 变更面重跑）。主检出落主后 p980 过滤 2/2。UI：ace_ui_hello 7/7（4.91s 含真窗 e2e）@落主内容 e529dde6。

**批次闭合。** T-HELLO-I/V/D 勾选（9/35；R 已在 round 12 勾）：AGE node 首源→真 Iced 窗口全链成立（H0 verify→AV 真值→F0 受检调用（分派时重观察防漂移）→run_app 宿主）；fixture ace-hello-minimal 双源；永久 case 7 项（CPU 5+反例 2 族）+ 排窗 e2e；F0 构建新鲜度缓存 + ace-host Exit 臂修复（master 预存编译红）随批落主。边界：完整 B0 打包闭包 pending、LIFE 20CPU+1真窗适配、AXR0 归 RUST 批——均在案。

next_responsible_role: work（980/RUST 批：前置 DIALECT✓+L978✅已归档；AXR0 切片消费 967/T-12、978/T-03/09 正式来源——已归档可取）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 13 | from: merge | to: work | status: executing（9/35，HELLO 批已交付落主）| outcome: pass（phase fold #4）| worktree: D:/autostack/.wt/lang-980（保留）| commit: lang 1991021a5 ui e529dde6 | next: work（RUST 批）
2026-11-16 stage: work（B-RUST-I/V 批，round 14：AXR0 先行 + 交付闭包修复链）| plan_id: PLAN-980 | plan_revision: r10 | outcome: pass（批次可交付独立复核；UI 侧因 driver 修复验证需要**提前 ff 落主**——与 review→merge 顺序的偏差具名记录，review 按同内容终核）| worktree: lang-980 | code_commit: auto-ui master 链 80e676f6f→ef21e4e9→aee85e13→d92d89cd→e2aadd45（lang 侧本批零代码；规范 4156f25c4→f46fb4408）。

**交付（AC-B-RUST AXR0 面 + 交付闭包）。** ①AXR0 双腿：`ui/ace_a2r_bridge.rs`（Leg A 组件场景 A2R 文件集 `__age_drive`；Leg B F0×A2R 967/T-12 桥——同文本字节经 provider）+ `examples/ace_a2r_axr0_build.rs` 显式构建道（真 rustc，回执 target/ace-a2r-axr0/receipt.json：leg_a `value:2`/leg_b `value:9`）+ `tests/ace_a2r_axr0.rs` 4 项（发射身份控制/回执消费/排窗窗口腿——窗口派生 A2R exe 显示真实 stdout，零 AV/旧 VM）。②**结构性修复链（driver 显形）**：DIALECT 批 ace-core dev-dep 越根致交付生成具名拒（本计划引入的回归）→ 交付投影新引擎面 `strip_dev_deps`（三态幂等）+ compiler 专用 spec；spec 须全行形态（裸名键取空误入已剥离分支——单测实证 `p980_delivery_projection_strips_compiler_dev_only_ace_core`）；STATE/HELLO/RUST 的 ACE 生产依赖 → `ace-bridge` feature 门（默认关，7 optional dep + dev ace-av）+ runtime 投影剥除/feature 空改写（autodown-core 同型）；bindgen 的 ace-foreign-contract（967/973 谱系预存）→ DELIVERY_VENDOR_TREES 补树。③**原工程 runner 真实执行**：PLAN-792 driver V01..V06 **43/43 PASS**（V01 fresh-lock 真构建/V02 重定位 locked/V03 负控/V06 全 profile 矩阵+vm-run；jet/ark 环境门控=driver 原语义；日志 plan-artifacts/plan-980/rust-driver-run{1..7}.log，run7=绿）。

**V-B-RUST 验证。** ui `cargo tu`=889 过/2 红（ui_gen::rust master 预存红，主检出同红取证，台账）；lang `cargo t ace-a2x generated-delivery-controls`=**双 PASS**；AXR0 4/4（2.91s 含窗口）+ HELLO 7/7 + plan041 9/9 + plan033 2/2（`--features ace-bridge` 终态）；driver 43/43。命令冻结：AXR0/HELLO/STATE 永久命令带 `--features ace-bridge`；构建道/排窗 prep 流程见测试文件头注。

**边界（KNOWN-DEBT 在案）。** 旧 rust_ui emitter 未替换（981 面接线）；A2R 文本出岛（902 冻结 i32 协议）窗口腿以 stdout 上窗；AXR0-PACK 完整打包 pending；ui_gen 2 例预存红。

2026-11-16 stage: review（B-RUST-R 终核，round 15）| review_scope: batch-final | reviewer: 同会话独立复核（工件重构：终态重放+authored 反例+driver 全量重放）| outcome: pass（批次闭合；B-RUST-I/V/R 勾选，12/35）| reviewed_commit: ui e2aadd45（=master）/ lang f46fb4408 | 复核：①冻结重放（投影双单测 2/2、AXR0 4/4 带 env、tu/a2x 门在 work 轮实测）；②authored 反例 `p980_delivery_projection_strips_runtime_ace_bridge`（runtime 投影剥 7 optional+dev+feature 空改写+幂等——删任一 strip/改写即红；首版断言过宽抓到注释词，探针修正记录在案）；③driver 43/43 全量重放=交付闭包修复的直接 oracle；④机械审计：本批 UI 增量=桥/构建道/测试/投影（交付语义修复三处均单测+driver 实证）；lang 零代码（规范 only）。findings: 无阻断。

2026-11-16 stage: merge（B-RUST-D phase fold #5，round 16）| outcome: pass（批次交付）| delivery_commit: auto-ui master **e2aadd45**（链 80e676f6f…e2aadd45，全部 ff-only 提前落主——偏差已于 work 轮具名）、auto-lang master **f46fb4408**（规范 §8，rebase 后全等落主）| canonical: m2-autoui.md sha256/16=**a9c1233f29ebda9d**（§8 AXR0 双腿/构建道纪律/feature 门/闭包修复）| 落地后复核：投影双单测 2/2@master、AXR0 4/4@终态内容、driver 43/43@master 内容（run7）。统一回归门：本批受影响组（ace-a2x/generated-delivery-controls/tu/projection 单测/driver）已按冻结内容实测全绿；ace-abi source_foreign 25 例预存红具名排除（台账）。

next_responsible_role: work（980/JET 批：前置 DIALECT✓+L978✅；Jet 工具链缺环境格按合同记 pending 不判红）。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 16 | from: merge | to: work | status: executing（12/35，RUST 批已交付落主）| outcome: pass（phase fold #5）| worktree: D:/autostack/.wt/lang-980（保留）| commit: ui e2aadd45 lang f46fb4408 | next: work（JET 批）
2026-11-16 stage: work（B-JET 批启动评估，round 17）| plan_id: PLAN-980 | plan_revision: r10 | outcome: blocked（环境前置缺失，具名）| 现场证据：driver V06c（run7，master 内容）`jet=env-gated`——`gradlew assembleDebug exited 9009`（Jet/Android 工具链不在场）；生成面 `jet generate_ok=True files_written=24` 在库正常。本机可执行面已绿（cargo tu 889/2 预存红、cargo t ace-a2x generated-delivery-controls 双 PASS——本会话 RUST 轮实测）。**不做无验证的 IR 重布线**：B-JET-I「受检 UI 领域 IR 供目标 adapter」与 V-B-JET 真实构建腿需要 Jet 工具链就地验证；任务保持未勾选（12/35 不变）。unblock=在 Jet 专用环境（gradlew/AGP 在场）执行 B-JET-I/V/R（KNOWN-DEBT 已立，owner=Jet 专用环境/后续轮）。WEB/ARK 仍卡 976、NATIVE 卡 STATE✓+973、TAURI/PARITY 如 §5——本 goal 链（用户裁定顺序 STATE→HELLO→RUST→JET）至此：前三个批完整交付落主，JET 具名环境阻塞。

stage: goal | plan_id: PLAN-980 | plan_revision: r10 | round: 17 | from: work | to: user | status: executing（12/35）| outcome: blocked（JET 平台环境缺失，具名 unblock）| worktree: D:/autostack/.wt/lang-980（保留续）| commit: ui e2aadd45 lang f46fb4408 | next: user（JET 需 Jet 专用环境或裁定后续）

2026-11-16 stage: new/revision | PLAN-980 r11 | 用户范围裁定（直接授权）：**JET/ARK 后端归 M3（v0.7），从 v0.6 移除；WEB（Vue+TS）保留 v0.6**。执行：§8 六任务行撤出（原文留档 HTML 注记）、total 35→29（当前 12/29）、§5 批表/阶段表与 PARITY 依赖改写、§6/§7 对应 V/AC 行注记归 M3；round 17 的 JET 环境阻塞记录由本裁定取代（v0.6 不再含该腿；M3 再启时原环境缺口仍在案）。WEB 保留意味着 L976（TypeScript provider，drafting）为 v0.6 WEB 批关键路径。DIALECT/STATE/HELLO/RUST-AXR0 已交付面不受影响；唯一树 lang-980 保留续 NATIVE/WEB/TAURI/PARITY。同步：M2 路线图 dated 注记 + m2-autoui 规范边界刷新 + KNOWN-DEBT JET 条目改写。本修订只改范围归属，不改已交付验收/其余义务。

stage: goal | plan_id: PLAN-980 | plan_revision: r11 | round: 18 | from: new | to: work | status: executing（12/29，JET/ARK 归 M3）| outcome: pass（范围修订落档）| worktree: D:/autostack/.wt/lang-980 | commit: pending | next: work（NATIVE/WEB/TAURI/PARITY 按前置续行）
2026-11-16 stage: work（B-NATIVE-I/V 批，round 19）| plan_id: PLAN-980 | plan_revision: r11 | outcome: pass（批次可交付独立复核；勾选留 R）| code_commit: lang 32e8e951c（免驱动面+av 双观察/再导出）、ui 34bb4326（ace_native 通道/exe/缝/测试）。

**交付。** ①lang 免驱动场景编译（980 入口专属：`with_initials`/`states`；979 冻结打包入口保持编译期拒——回归 16/16 实证）+ `observe_component_states_and_texts` + `project_component_candidates` 再导出。②UI `ui/ace_native.rs`：001 真源（widget 头 Accept）装载——文本值（AV）+属性（候选投影 view 树按声明序：style/selectable）+状态；状态写=初值重建→重编译→重观察（dark_mode→窗口主题线程局部翻转，与旧链同机制）；`AceNativeWindow`（run_app 宿主）+ `examples/ace_native_window.exe`（帧内状态写/证据/自动退出；关闭重开=顺序多轮进程级）。③引擎缝：run_vm `execute_vm_entry` 增 `AUTO_UI_ENGINE=ace` 分支→`run_ace_native_entry`（无条件编译，feature 关=具名拒；出面外源 ACE 面具名拒不静默回旧 VM；旧 vm/托管 hook 不变）。

**V 验证（冻结提交实测）。** 新文件 `tests/ace_native_window.rs` CPU 4/4（001 装载/状态写翻转+未知字段拒/出面外 shape 拒/双叶属性对齐）+ 排窗 e2e 1/1（3.88s：真窗、selectable/style 入叶、帧内 dark_mode 写→主题翻转证据、**两轮关闭重开**各 exit 0）；V 行点名门 desktop_behavior 11 过/1 ignored（P488-D1 原条件）+ image_surface_contract 1/1；`auto run -r vm` 隔离工程身份=driver V06e 真跑复用（43/43 内：vm-session 子进程身份/MCP 探活/cancel 收口）。**fmt 事件（诚实）**：rustfmt 经 lib.rs 递归扫入 128 个历史未格式化文件（+fmt only）随 34bb4326 落主——全保真核验：128 文件逐字节=父提交内容+rustfmt，3 文件为我的真实改动+1 文件（dynamic.rs）为配置宽度差异的纯换行，**零语义变化**。

2026-11-16 stage: review（B-NATIVE-R 终核，round 20）| review_scope: batch-final | reviewer: 同会话独立复核（工件重构）| outcome: pass（批次闭合；B-NATIVE-I/V/R 勾选，15/29）| reviewed_commit: ui 34bb4326 / lang 32e8e951c | 复核：①冻结重放（新文件 5/5、点名门两文件、979 面 16/16+p980 全家、driverless 1/1）；②authored 反例 `p980_native_multi_leaf_props_align_by_declaration_order`（双叶属性对齐——zip 顺序错位即红）；③fmt 事件全保真核验（如上）；④边界核对（交互档手工/环境门控、路由未交付归 981、默认链沿 STATE 边界）——KNOWN-DEBT 待补 NATIVE 条目随 merge 落。findings: 无阻断。

2026-11-16 stage: merge（B-NATIVE-D phase fold #6，round 21）| outcome: pass（批次交付）| delivery_commit: auto-ui master **34bb4326**、auto-lang master **1d2f01238**（免驱动 32e8e951c+规范 84886ea11 rebase 后落主）| canonical: m2-autoui.md sha256/16=**5fa70f1c34770f26**（§9 原生 run 通道）| 落地后复核：主检出 p980_native 过滤绿；ui 落主内容=worktree 验证内容同身份。统一回归门：本批受影响组（ace-av/ace-source 组件面、runtime 新文件、tools check）实测绿；预存红台账不变（source_foreign/ui_gen 2 例）。

next_responsible_role: work（v0.6 剩余：WEB 批待 L976 立项实施；TAURI 待 WEB；PARITY 待 NATIVE✓/WEB/RUST✓/TAURI）。

stage: goal | plan_id: PLAN-980 | plan_revision: r11 | round: 21 | from: merge | to: work | status: executing（15/29，NATIVE 批已交付落主）| outcome: pass（phase fold #6）| worktree: D:/autostack/.wt/lang-980（保留）| commit: ui 34bb4326 lang 1d2f01238 | next: work（WEB 批关键路径=L976）
2026-11-16 stage: work（T-01 分母冻结，round 22）| plan_id: PLAN-980 | plan_revision: r11 | outcome: pass（任务闭合；勾选 T-01，16/29）| 交付：m2-autoui.md **§10 分母/命令/oracle 冻结表**（批次×状态×冻结命令×输入源身份；模式网格 NATIVE✓/RUST-AXR0✓/WEB 解锁/TAURI 传递/PARITY 源面可先行；共享纪律=前门短窗/F0 缓存/构建道/排窗/预存红台账）+ replacement-scope **SD-02 挂接增量**（980 五批 producer 行+具名边界，沿 979/975/976 增量节先例）。**重大事实更新**：核实 PLAN-976 已归档（`5cedb4715`，delivery `660fe72b5`——ace-a2x-ts 双目标 TS/JS provider 全交付、"产品适配器切换归 980"）→ **WEB/TAURI/PARITY 的 976 前置全部解锁**，工作序从"976 卡住"前提更新为 WEB 可直接开工。T-01 分母本体=字段清单（115 行，DIALECT 已交）+959 台账（979 节维护中，本轮 980 行刷新）；唯一树/写窗=§10.3 冻结。落主：lang `ae041fb6a`（§10）+`db4ad0b37`（SD-02）。

stage: goal | plan_id: PLAN-980 | plan_revision: r11 | round: 22 | from: work | to: work | status: executing（16/29，T-01 闭合）| outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: lang db4ad0b37 | next: work（WEB 批——976 已解锁）
2026-11-16 stage: work（WEB 批首切片，round 23）| plan_id: PLAN-980 | plan_revision: r11 | outcome: pass（切片交付；**B-WEB-I/V/R 保持未勾**——完整适配器切换为批剩余义务）| code_commit: auto-ui **dfc73cf1**（`ui/ace_ts_provider.rs` 桥 + 4 永久 case；lang 零代码——976 接口原样消费）。交付：UI 表达式/语句源→ACE core 前门（tsfrag 同门）→976 fragment（checked-helper）→宿主 reactive 两跳映射（count→count.value）→真 node FRAG_OK（42/42）；未定位/越界/类型错误=具名拒绝（无旧 emitter 回退）。探索发现（收据在案）：现行管道求值旧 AST、976 接口吃受检 HIR——**形态阻抗**是完整切换的实底：ts_adapter 四处委托（Stmt/Expr×vue/ark）替换需按源 span 线程化+宿主映射扩展+90 项 vue_capabilities 全量过新车道+浏览器挂载档（013/017 playwright 先例）。测试表：ace_ts_provider 4/4=0.13s（node 腿含内）；ace-bridge feature 门下编译；未动 ts_adapter/vue_capabilities（旧车道原样）。边界详见规范 §11。next：WEB 剩余=ts_adapter 委托替换（切片扩面：str/结构/回调形态）+浏览器档；TAURI 随 WEB；PARITY 源面/NATIVE 腿仍可并行。

stage: goal | plan_id: PLAN-980 | plan_revision: r11 | round: 23 | from: work | to: work | status: executing（16/29，WEB 首切片落主）| outcome: pass（切片）| worktree: D:/autostack/.wt/lang-980 | commit: ui dfc73cf1 | next: work（WEB 剩余/PARITY 源面）











2026-11-16 stage:new/revision | PLAN-980 r13 | outcome:pass（规划合同可执行，非实现/产品复审） | changed:B-NATIVE-I/V/R、AC-NODE-DESKTOP、V-NODE-DESKTOP-*、SD-NODE-DESKTOP | preserved:原任务ID/全部历史收据；NATIVE原I/V/R因新增范围重新打开，15/29调整为12/29 | next:work在原唯一树先核主线与CLI，实施前有界决定正式UI接面；A入口不等AC桌面，1068/T-04 B承接。当前用户仅授权规划，未启动work。原NATIVE勾选仅代表r11宿主直取AV快照范围；新r13 I/V/R与相应证据须重核，不复用为自动PASS。

本轮规划静态预检：9份计划/设计路线文档；四份Plan共87个原任务ID/总分母与依赖列表保持，仅980的NATIVE I/V/R因新增合同重开（15/29→12/29），历史收据未删。新增本地链接/锚点、0–10章节、验收/用例/原任务映射、表格与行尾检查通过；A入口→A出口→AC切片→B产品/终验的阶段图无环。候选固定组名按scripts/test-groups.toml只读核实，新UI/AC入口或case明确由work落实，均未运行。本记录是文档预检，不是产品或独立实现复审PASS；未改canonical Specs/源码/测试资产/取号计数器，未实施/建树/构建/运行/合入/清理/恢复目标。

2026-11-16 stage:new/revision | PLAN-980 r14 | outcome:pass（规划可执行合同，非实现/认证PASS） | 新增B-AV三项，总32；RUST三项因完整W3范围重开，原AXR0历史保留，当前9完成；A→W1→W2→W3→OS；验收/任务/规范增量逐项承接 | next:由原owner依新阶段/真实来源接续；本轮只规划，不实施/建树/测试/合入/清理/恢复目标。

本轮规划静态预检：14份计划/设计/路线文档，8份Plan共230个当前任务；仅980新增B-AV-I/V/R三项并按完整W3范围重开原RUST三项，旧AXR0收据保留；981仅核正原SOURCE三项已交的滞后计数。全部原任务ID/其余勾选、各Plan状态与整包依赖列表保留；新增本地链接/锚点15处、章节/表格/元数据计数/验收任务映射及行尾检查通过，A最小→A出口→W1→W2独立交接→W3→OS→候选/安装/退役阶段图16节点18边无环。原GUI/Widgets/Notes-Clock/语言/库/产品义务有具名阶段与原owner，不以延期冒PASS。仅文档预检，未实施/建树/运行测试或窗口/签阶段/合入/清理/恢复目标，未写源码、测试资产、canonical Specs或取号计数器；他会话并发修改不由本轮回退。

2026-11-16 stage: work（B-NATIVE-I/V 批，r13 MIN-A-UI，goal round 24：用户授权"完成计划980"开工链）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（批次可交付独立复核；B-NATIVE-I/V/R 勾选留 R，current_step 仍 9/32）| worktree: D:/autostack/.wt/lang-980/{auto-lang,auto-ui} @ plan-980-dev | base: lang 6ff4233b4 / ui dfc73cf1 | code_commit: ui 3df28cf9、lang c449acc13（+18 行）。

**B-NATIVE-I 有界决定（冻结）。** ①UI-FFI 载体 = `ui980` provider（hooks trampoline）：依赖无关注解源经 `build_source` 构建（root=target/ace-node-desktop，新鲜度缓存，prep 后测试零 rustc）；**967 冻结 AV 外呼可调用域 = view + 2×i32（ViewI32I32ToI32）**——7 函数全部 `(view,i32,i32)->i32`：`ui_open`（tag+instances）、`ui_apply_text/ui_apply_str`（view 即载荷，instance*1000+slot 打包，当前 session）、`ui_apply_int/ui_apply_bool`（tag+打包+值，当前 session）、`ui_poll_event/ui_close`（tag+显式句柄）。UI session 状态（显示模型/typed 队列/世代/证据计数）归 auto-ui-runtime 所有；宿主与 AV 经同一 DLL 模块（LoadLibraryW 引用计数）共享 hooks statics。②事件回 AV：窗口点击→typed 队列（世代纪律）→tick 受控调用点 AV 外调 `ui_poll_event`→`AceStateEngine::dispatch`（STATE 已交场景重放面，业务全在 AV）→状态差量经 AV 外调 `ui_apply_*` 提交实际 UI 更新；计数绝不在 Rust。③三类源：ace-hello-minimal 双源 + 新 fixture `examples/ui/ace-node-counter/src/front/app.at`（002 计数业务派生 node：msg Inc/Dec/Reset、count int、on 复合赋值）；文本面两段：driverless-full（带 model/on 源）→静态观察面（Hello 类无 model 的具名回退；静态面拒 on=具名行为）。④命令冻结：CPU `cargo test -p auto-ui-runtime --features ace-bridge --test ace_node_desktop -- --test-threads=1`；排窗 `AUTO_UI_ACE_NODE_E2E=1`（prep=构建 example+`--prep-only`）；产品入口 `AUTO_UI_ENGINE=ace`→`run_ace_node_desktop_entry`（r11 快照面保留给自己的测试/exe）。

**交付。** UI：`ui/ace_node_desktop.rs`（provider 源+hooks+UI session host+桥+`AceNodeDesktopWindow`+入口）、`examples/ace_node_desktop_window.rs`（e2e 驱动/证据 JSON/自动退出/--prep-only）、`tests/ace_node_desktop.rs` 7 永久 case、counter fixture、run_vm 分支重定向、`ace_hello::escape_auto_string_for_source` 转 pub；lang：ace-host `register_source_provider`（通用源 provider 注册面，返回 cdylib 路径）。

**V 验证（冻结提交实测；会话测试表）。** CPU 7/7=0.74s：①轨迹 0→1→0→0+两实例定向 2:1（dispatch 序 [Inc,Dec,Reset,Inc,Inc,Inc]、终值 i0=2/i1=1、宿主显示=AV 真值、applies 只增、初始 FFI 提交≥4）；②hello 双源 AV 文本字节级过 FFI（"Hello"/"你好 Auto"）；③坏 Schema：无绑定 target→LaneError+同入口恢复绿、未声明 msg `.Nope`→DispatchRefused 且零 UI 效果；④坏 ABI：poll/close 未知句柄 -1、越槽/越实例 apply -3，全部零提交；⑤close/reopen/late：close 后 apply/poll -2 具名拒、push→late_dropped、reopen 新世代重提交（业务 count 延续 1→2）、旧代 applies 冻结；⑥byte_len 反例：F0 车道成功返回 1 但 applies/display 均不动（V-UI-A-MIN oracle 对伪车道的判别力固化）；⑦撤权：`Denied ace.provider/revoked` 零 UI 效果、复权同门恢复。排窗 e2e 1/1=2.24s：真窗 frames≥4、六事件轨迹、双 session（旧代 closed/late≥1、新代重提交）、reopen 后显示 [2,1]。受影响面（串行）：ace_ui_hello 7/7、ace_native_window 5/5、ace_ts_provider 4/4、ace_a2r_axr0 4/4、plan041 9/9、desktop_behavior 11 过/1 ignored（P488-D1 原条件）、image_surface_contract 1/1；`cargo check -p auto-ui-tools` 绿；lang `cargo t ace-facade` PASS。逐 case 墙钟（含 cargo 启动）0.9–1.8s，全部 ≤10s。fmt clean、我的文件零警告。

**边界（诚实声明）。** ①f-string 文本叶在两条文本车道均具名拒（KNOWN-DEBT 既有边界，W1 义务）——counter fixture 文本叶用字面量、计数值显示走 int 状态槽呈现；②交互档（真实鼠标/IME/选择复制）归手工/环境门控档（r11 同口径）；③instance*1000+slot 打包为 MIN-A 界内约定（实例 ≤4、槽 <1000，越界具名拒 -3）；④driverless-full 的 states 观察弃用（状态经引擎面取——纯文档性）；⑤完整 001/002 主题/选择/全部组件/Widgets 归 W1 B-AV，义务不变。

next_responsible_role: B-NATIVE-R（同树停写独立核本批：AC-B-NATIVE 的 V-UI-A-MIN 子面、AC-NODE-DESKTOP 反例重放、边界与债务核对；pass 后勾 B-NATIVE-I/V/R=12/32、phase fold #7 落主）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 24 | from: work | to: review | status: executing（9/32，NATIVE r13 批待复核）| outcome: pass | worktree: D:/autostack/.wt/lang-980 | commit: ui 3df28cf9 lang c449acc13 | next: review（B-NATIVE-R）

2026-11-16 stage: review（B-NATIVE-R 批次终核，goal round 25）| review_scope: batch-final | reviewer: 同会话独立复核（限制：非独立上下文；结论由工件重构——冻结树核对、全量重放、authored 反例、机械审计，非采信执行者摘要）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（批次闭合；B-NATIVE-I/V/R 勾选，12/32）| reviewed_commit: ui 3df28cf9 + e203caa8（复核反例）/ lang c449acc13 | base: lang 6ff4233b4 / ui dfc73cf1 | worktree: lang-980 @ plan-980-dev（双树 clean）。

**复核内容与结果。** ①冻结核对：双树 clean 于复审提交；diff 面=9 文件 +2070/-6（ui）+18（lang），与收据申报一致；无探针残留（zz_probe 已删）、零调试输出（ACE980_DEBUG 探针已随 view+2 重构移除，grep 0）、fmt clean、模块零警告。②全量重放（冻结提交实测）：CPU 8/8=0.63s（含复核 authored case）+ 排窗 e2e 1/1=1.69s。③AC 映射：AC-B-NATIVE A 切片=V-UI-A-MIN 逐项——真窗文字/中文（e2e frames≥4+你好 Auto 字节级过 FFI）、0→1→0→0 与两实例 2:1（dispatch 序+终值 2/1+显示=AV）、关开/late/权限/坏更新恢复（-2/late_dropped/Denied+复权/越槽拒后同入口绿）、**byte_len 反例判别力**（伪车道成功返回但 applies/display 不动）；AC-NODE-DESKTOP 本批 A 子面同上，完整 V-NODE-DESKTOP-* 归 B-AV/W1（不提前签）。④**authored 反例（e203caa8，永久）**：车道标签纪律（错 tag→BAD_OP 零效果、负打包→BAD_SLOT）、**跨 session 队列隔离**（显式句柄 poll 不偷当前 session 事件——s1 poll=0 且 s2.polled 不动，s2 poll 取编码 1003）、双重 close→-2（不复活）；首版探针自身 arity 笔误（4 参调 3 参函数）被 source.argument 具名拒抓出——探针缺陷修正记录在案，非产品缺陷。⑤机械审计：语言侧 +18 行纯加法（通用注册面），ace-facade PASS 实证；run_vm 重定向编译绿；r11 快照面/HELLO/WEB 切片/AXR0 回归全绿（round 24 表）。⑥测试成本：全部新 case ≤10s（文件 0.63s/8 case；e2e 1.69s；逐 case 墙钟 0.9–1.8s 含 cargo 启动）；prep 纪律成立（provider 新鲜度缓存，重放零 rustc）。

**边界核对（诚实）。** f-string 文本叶双车道具名拒（KNOWN-DEBT 既有）；交互档手工/环境门控（r11 同口径）；instance*1000+slot 打包 MIN-A 界内（authored 已锁负值/错标签面）；完整 001/002 主题/全部组件归 W1——均与 work 收据一致，无未申报缩水。

**acceptance 结果。** AC-B-NATIVE A 切片=pass；AC-NODE-DESKTOP A 子面=pass（完整格 B-AV/W1 保持 open）；AC-02/03/06/07 本批子面=pass；AC-04 本批复核面=pass（本记录）。

pending_closeout_gates: 统一回归门（merge 冻结态：本批受影响组 ace_node_desktop/ace-facade/UI 点名门已实测，落主基线若代码面动按规矩重跑）；SD-NODE-DESKTOP 规范增量（merge 时落 m2-autoui.md §NODE-DESKTOP）。

findings: 无阻断缺陷；F1（探针 arity 笔误，已修正，原始记录保留）。

新增 case 行为/耗时（authored）：lane_tag_queue_isolation_and_double_close——错标签/负打包/跨 session 隔离/双 close 四断言面，文件 8 case 合计 0.63s。

next: merge（phase fold #7：ui e203caa8 链 + lang c449acc13 rebase 落主 + 规范 §NODE-DESKTOP + 落地后复核门；唯一树保留续 WEB 剩余/TAURI）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 25 | from: review | to: merge | status: executing（12/32，NATIVE r13 批已复核）| outcome: pass（批次）| worktree: D:/autostack/.wt/lang-980 | commit: ui e203caa8 lang c449acc13 | next: merge（phase fold #7）

2026-11-16 stage: merge（B-NATIVE-D phase fold #7，NATIVE r13 批正式落主；整包 executing 12/32 保持，唯一树保留续 WEB 剩余/TAURI，不归档/不清树）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（批次交付）| delivery_commit: auto-ui master **403c936d**（代码 a8ace167 + 复核反例 403c936d）、auto-lang master **7f431bd5f**（代码 562b90fd3 + 规范 7f431bd5f）| canonical: docs/specs/ace-migration/m2-autoui.md（master 上 sha256/16=**c58125ab4889b796**，新 §12 MIN-A-UI 通用 Node 桌面入口；§9 引擎缝去向在 §12 注记取代）| 落主方式：两仓 rebase 到在途 master（981 已落 SESSION/host 多笔代码——lang e36b7ffcd 等、ui 4c629db0 等）+ range-diff 全等（lang c449acc13→562b90fd3 `=`、spec →7f431bd5f；ui 3df28cf9→a8ace167 `=`、e203caa8→403c936d `=`）+ wt-guard 双树 clean + `git merge --ff-only`（无 merge commit、无冲突）。

**落地后复核门（merge-owned，rebase 基线实测——master 代码面因 981 已动，按规矩全量重跑，不复用旧证据）。** UI（rebase 后 worktree=落主内容）：ace_node_desktop **8/8**=0.62s + 排窗 e2e 1/1=3.00s（真窗，含 981 代码）+ 回归集 ace_ui_hello 7/7、ace_native_window 5/5、ace_ts_provider 4/4、ace_a2r_axr0 4/4、plan041 9/9、desktop_behavior 11 过/1 ignored（原条件）、image_surface_contract 1/1。lang：`cargo t ace-facade` = **PASS**。落地后 master 核对：双仓 tip=delivery commit、§12 在库（hash 见上）、lang 主检出代码路径 clean。

**批次闭合。** B-NATIVE-I/V/R（r13 重开范围）交付并勾选（12/32）：ui980 provider（967 冻结 view+2×i32 域内 7 函数，FFI 实际承载 UI 创建/提交/更新/关闭）+ typed 事件回 AV（poll→AceStateEngine 场景重放，业务全在 AV）+ 通用入口 `AUTO_UI_ENGINE=ace`→node-desktop + 三类源 + 9 永久 case（含 byte_len 反例与复核 authored 三面）。边界（f-string 双车道拒/交互档手工/打包界/W1 全量）在案。

**观察项（非阻断）。** ①UI 主检出存在未跟踪目录 `crates/auto-ui-runtime/test/ui`（无 git 历史，非本计划创建——本批全部工作在 lang-980 组 worktree；归属未明，未动，fail-closed 留待其 owner 处置）；②批量回归 due-check：980%5=0，按 §9 round 23 口径归 980 最终归档收口触发（本次为 phase fold，不触发）；③plan-artifacts/plan-980/ 保持 in-flight（整包在途；本轮 patch 脚本为暂存，最终归档时整目录删除）。

next_responsible_role: work（980/WEB 剩余批：ts_adapter 委托替换扩面 + 浏览器档——976 已解锁；TAURI 随 WEB。B-AV 等 M2A-08（985），B-RUST 完整 W3 等 UI-W2-AC（1068）——波次门阻塞在案）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 26 | from: merge | to: work | status: executing（12/32，NATIVE r13 批已交付落主）| outcome: pass（phase fold #7）| worktree: D:/autostack/.wt/lang-980（保留）| commit: ui 403c936d lang 7f431bd5f | next: work（WEB 剩余/TAURI；B-AV/B-RUST 波次门阻塞已具名）

2026-11-16 stage: work（WEB 批切片 #2，goal round 27）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（切片交付并落主；**B-WEB-I/V/R 保持未勾**——完整适配器切换为批剩余义务）| worktree: lang-980 @ plan-980-dev | commit: ui **6aa0fd4a**（rebase 至 981 Run/Stop 落主后）、lang **3047fc389**（规范 §13）。

**交付。** ①`ace_ts_provider` 参数类型域扩面：`&[&str]` → `&[(name,ty)]`（int/bool 有效；str 被 profile 拒——普查记录）；②**车道覆盖普查冻结表**（`p980_web_lane_census_frozen_forms`，11 形态）：生产 UI 表达形态经 976 fragment 车道的逐项 Ok/具名拒结果——比较算子（`>`/`==`）、布尔算子（`!`/`&&`）、`str` 类型、f-string 插值、多函数源全部 `source.unsupported/lex/syntax` **具名拒**（Core profile 域外）；非字面 var-init 触发 `verify.block-structure`（O-C2 形状缺口）；今日 OK 面=字面 init 两步赋值/return 表达式（int）+宿主映射+node 执行；③规范 **§13**：普查事实+推论落库。

**关键发现（切换前置义务，比 round 23 记录更深）。** 976 fragment 车道经 `compile_source`（Core profile）编译——**生产 UI 表达表面不在 profile 域内**。ts_adapter 委托替换的实底不是（只是）AST→HIR 线程化：先须扩 fragment 编译 profile/入口（976 后继或 980 内 richer entry——组件面/text-record 编译入口候选）。普查表=该义务的可测分母（表变化须具名修订）。"976 已解锁"前提由此限定：解锁≠充足——WEB 完整切换的语言侧前置现在有名有表。

**验证（冻结提交实测）。** `cargo test -p auto-ui-runtime --features ace-bridge --test ace_ts_provider -- --test-threads=1` = **5/5**（4 旧 case 含 node 执行腿随类型域重构复绿 + 普查 1/1）；rebase 后（981 Run/Stop 已并入）同门复跑 5/5；fmt clean、我的文件零新增警告。vue_capabilities（82 case，旧车道）不动绿——本切片零编译器产品改动（provider 模块+测试面）。

**边界（诚实）。** 本切片证明：类型域 API+普查分母成立、profile 缺口具名可测；不证明：任何生产表面已过新车道（恰为普查所度量的缺口）；span 线程化/浏览器档/90 项全量切换=批剩余义务（前置=profile/入口扩域）。

next_responsible_role: work（WEB 批切片 #3：fragment 编译入口扩域候选评估——组件面/text-record 入口能否承载 handler 体源；或按裁定立 976 后继前置。B-AV/B-RUST 波次门不变）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 27 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#2 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui 6aa0fd4a lang 3047fc389 | next: work（WEB 切片#3：profile/入口扩域）

2026-11-16 stage: work（WEB 批切片 #3：入口扩域评估，goal round 28）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（评估结论=组件面路线成立，机制钉住并落主；**B-WEB-I/V/R 保持未勾**）| worktree: lang-980 @ plan-980-dev | commit: ui **3c8c80ff**、lang **3190c60b3**（规范 §13.1）。

**评估结论（决策级）。** ①**组件面 = fragment 扩域可行入口**：生产形态源（`!` 一元/str/bool 字段——Core profile 具名拒）经组件面 states-only 编译 → `SemanticView::from_checked` → 976 发射器可运行；永久 case `p980_web_component_face_entry_fragments` 端到端钉住（span 定位→发射→**调用形**宿主映射（`__age_state_<i>_<field>()`→reactive 值，吃括号）→真 node 读值 FRAG_OK，6/6）。②**O-C3**：脱糖 drive 的 span 精度（`!.on` 源段定位落到 count 调用，PLAN-964 同族边界）——精确 per-handler 定位=切片#4（适配器接线）前置。③**裁定不需要**：WEB 完整切换无需 976 后继扩 profile——组件面路线成立；剩余=span 精度/逐 handler 定位+ts_adapter 接线+全量切换+浏览器档（多会话量级，波次门不变）。

**验证（冻结提交实测）。** `cargo test -p auto-ui-runtime --features ace-bridge --test ace_ts_provider -- --test-threads=1` = **6/6**（含切片#2 普查+切片#3 入口 case，node 腿实跑）；fmt clean；ui 主落主 ff（分支已同步 master，981 Run/Stop 已含）；lang 规范 §13.1 落主。

next_responsible_role: work（WEB 切片#4：逐 handler 定位/span 精度 + ts_adapter 接线设计；波次门 B-AV←M2A-08、B-RUST W3←UI-W2-AC 不变）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 28 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#3 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui 3c8c80ff lang 3190c60b3 | next: work（WEB 切片#4）

2026-11-16 stage: work（WEB 批切片 #4：逐 handler 精确定位/O-C3 解除，goal round 29）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（lane API+永久 case 落主；**B-WEB-I/V/R 保持未勾**）| worktree: lang-980 @ plan-980-dev | commit: ui **80d1cc06**、lang **16f7ec508**（规范 §13.2）。

**交付。** `ComponentFragmentLane`（ace_ts_provider）：①compile(src, path, handler_msgs)——组件面 states-only 编译，**ops=待定位 handler 的 msg 重放**（探针实证：ops 空则 handler 体缺席状态函数——切片#3 异常根因即在 drive 整源 span 上定位）；②`emit_expr_at_span(state_fn, span)`——状态函数 HIR 的 expr **span=原源坐标**，精确匹配；**同 span 多 expr 取索引序最大=组合形**（`!on` 的 NOT 与纯读形同 span，NOT 在后）；③越界/无命中/未知状态函数=具名拒。永久 case `p980_web_handler_precise_location`：Toggle 区 `!` 操作数 span → `(!l1___age_s_0_on)` NOT 保留 → host 映射 → **真 node !false===true FRAG_OK**；Inc 区 `1` 字面量同 lane 精确定位（逐 handler 区域可分辨）。

**验证（冻结提交实测）。** `cargo test -p auto-ui-runtime --features ace-bridge --test ace_ts_provider -- --test-threads=1` = **7/7**（切片#1–#4 全链：绑定映射/语句/普查/入口/精确定位，node 腿实跑）；fmt clean、模块零警告；rebase 无增量（分支已同步 master）、ff 落主。

**边界（诚实）。** 本切片解除 O-C3（span 精度）并交付 lane API；**不包含** ts_adapter 四委托点接线（需 widget 管线携源 span 到委托点——AuraTsContext 源线程化）、90 项全量切换、浏览器档——B-WEB 剩余义务不变。

next_responsible_role: work（WEB 切片#5：ts_adapter 委托点源线程化+lane 接线；波次门 B-AV←M2A-08、B-RUST W3←UI-W2-AC 不变）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 29 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#4 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui 80d1cc06 lang 16f7ec508 | next: work（WEB 切片#5：ts_adapter 接线）

2026-11-16 stage: work（WEB 批切片 #5：lane 消费真实生产源，goal round 30）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——ts_adapter 接线/全量切换/浏览器档为批剩余）| worktree: lang-980 @ plan-980-dev | commit: ui **c74f8755**、lang **8d0468cdb**（规范 §13.3）。

**交付。** ①lane 坐标基准明确=**f-string 脱糖后源**（探针实证 002 span 位移 +7——组件面入口统一脱糖所致；lane 先脱糖再编译并暴露 `desugared_source()`，无 f-string 源时与原源一致）；②`emit_expr_at_text`（文本锚包含定位，同 span 组合形优先）；③**真实生产 002 全源端到端**（永久 case `p980_web_lane_consumes_production_002_source`）：ToggleMode `!.dark_mode` NOT（局部名 `l2___age_s_0_dark_mode`——字段序即局部序）按脱糖源 span 精确定位→reactive 映射→真 node `!false===true`；Inc handler 同 lane 定位（002 `.count += 1` 首现于 view 内联 lambda——**on 块 occurrence 推导**记录为适配器配方）。④线程化事实补全：旧 AST（auto_lang_core::ast）零 span、方言 `OnHandler` 无 span、view 树 ViewNode 有 span——**接线前置=方言层 OnHandler span 捕获 + LogicPayload 源区间扩展**（切片#6 工程面，已具名）。

**验证（冻结提交实测）。** `cargo test -p auto-ui-runtime --features ace-bridge --test ace_ts_provider -- --test-threads=1` = **8/8**（切片#1–#5 全链；node 腿实跑）；fmt clean、模块零警告；rebase 无增量、ff 落主。

next_responsible_role: work（WEB 切片#6：方言 OnHandler span 捕获 + LogicPayload 源区间 + ts_adapter 委托点接线；波次门不变）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 30 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#5 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui c74f8755 lang 8d0468cdb | next: work（WEB 切片#6：span 捕获+接线）

2026-11-16 stage: work（WEB 批切片 #6a：方言 span 捕获+提取线程化+接线闭环，goal round 31）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——委托点接线/全量/浏览器档为剩余）| worktree: lang-980 @ plan-980-dev | commit: ui **b3c9c582**、lang **ab6c8efdd**（§13.4；债务条目提交在分支，见下）。

**交付。** ①方言 `OnHandler.span`（`#[serde(default)]` 原源坐标；on 块解析捕获、铸造形 None）——**DIALECT 清单护栏按设计工作**：`widget.on.handler.span` 登记（构造键+清单行+walk 发射+002 计数冻结 6），护栏红→登记→绿全程实证；②`extract_handler_spans(decl)`（compiler 零涟漪独立面）；③接线闭环永久 case `p980_web_extraction_carried_spans_feed_lane`（ace_ts_provider **9/9**）：真实解析管线（CompilerSession+Parser）→ 提取 span 区间核验（O-C4：起点 pattern 中段小偏移，body 完整）→ lane 区间内文本锚 → ToggleMode NOT fragment。④**vue_capabilities 10 红=master 预存**（stash 取证：基线无本计划改动仍 72/10——chart×5/vmodel/回调/枚举 msg 参数 handler 族，疑 981 诊断路由邻面；KNOWN-DEBT 条目已写、其提交因**主检出 KNOWN-DEBT 存 981 会话未提交 WIP**（D-981-SESSION-MULTI）暂留分支，981 落后再 land，不覆盖他会话 WIP）。

**验证（冻结提交实测）。** dialect lib 5/5、ui_domain_validation 全绿、ace_ts_provider 9/9（node 腿实跑）；fmt clean。落地：ui ff 直落 `b3c9c582`；lang 规范 `ab6c8efdd` ff（债务提交分支持有，具名待 land）。

next_responsible_role: work（WEB 切片#6b：span 起点校准 + 生成器源通道 + ts_adapter 四委托点接线；债务提交随下轮 rebase 落。波次门不变）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 31 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#6a 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui b3c9c582 lang ab6c8efdd | next: work（WEB 切片#6b）

2026-11-16 stage: work（WEB 批切片 #6b：O-C4 解除 + 接线设计决策，goal round 32）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——路线 B 实施/全量/浏览器档为剩余）| worktree: lang-980 @ plan-980-dev | commit: lang **4656bf607**（§13.5；债务提交仍停靠分支——主检出 KNOWN-DEBT 的 981 WIP 未落）。

**交付。** ①**O-C4 解除（诊断+语义重定义）**：全 6 handler span 诊断实证——捕获语义=**body 覆盖区间**（起点在 pattern 末~body `{` 间 ±1 token，终点=body 闭括号后；lane 合同只需 expr 包含性，002 六例全成立）——精确 pattern 起点不承诺、不需要；②**四委托点接线设计决策（§13.5）**：路线 A（表达式级直接换 lane）**受阻于旧 AST 零 span**（语言侧大手术，不采用）；**路线 B 采纳**（handler 体级：script 生成器携源文本+提取区间，状态函数 HIR 区间内 exprs 逐发射+宿主映射组装 handler 体——UI 侧自有面）。剩余=路线 B 实施（vue/script.rs 生成道 + `generate_with_source` 源通道）→ 全量双轨 → 浏览器档。

**验证。** 诊断探针用后即删；§13.5/§13.4 面不动测试（本 round 纯文档+诊断）；ace_ts_provider 9/9 保持（上轮冻结基线复用——无实现改动）。

next_responsible_role: work（WEB 切片#6c：路线 B 实施。波次门不变：B-AV←M2A-08（985）、B-RUST W3←UI-W2-AC（1068）、TAURI←WEB）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 32 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#6b 落主）| worktree: D:/autostack/.wt/lang-980 | commit: lang 4656bf607 | next: work（WEB 切片#6c：路线 B 实施）

2026-11-16 stage: goal（等待轮：完成审计+用户决策点，无阶段施工）| plan_id: PLAN-980 | plan_revision: r14 | round: 33 | from: work | to: user | status: executing（12/32）| outcome: waiting（结构性外部依赖，具名）| worktree: D:/autostack/.wt/lang-980（保留）。

**完成审计（磁盘级，round 24–33 全对账）。** 已交付落主：NATIVE r13 全链（12/32）+ WEB 切片#1–#6b（普查/入口/精确定位/002 端到端/方言 span/设计决策）+ 规范 §12/§13.1–13.5。未完成：B-WEB 剩余（路线 B 实施=vue/script.rs 生成道+`generate_with_source`，多会话量级；90 项双轨；浏览器档）、B-TAURI、B-PARITY、T-05/06/07。**结构性外部依赖（用户 r13/r14 波次门，本会话不可解除）**：B-AV←M2A-08（985 executing 4/14）；B-RUST W3←UI-W2-AC（1068 drafting 0/10）；TAURI←WEB 完成。停靠项：KNOWN-DEBT 债务提交在分支（主检出 981 WIP 未落）；vue_capabilities 10 红=master 预存（已具名归 triage）。

**用户决策点（§10 同步登记）**：完成计划980 需其一——①等待 985 交付 M2A-08 与 1068 交付 UI-W2-AC（波次门原样，980 保持 executing 至外部交付）；②调整波次裁定，允许 B-AV/B-RUST 与 WEB 并行（本会话可立即开工 B-AV 的 U01 分组面）；③授权后续会话继续 WEB #6c（路线 B 实施）直至 WEB 批闭合。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 33 | from: verify | to: user | status: executing（12/32）| outcome: waiting | worktree: D:/autostack/.wt/lang-980 | commit: — | next: user（波次门/并行裁定）

2026-11-16 stage: work（WEB 批切片 #6c：路线 B 核心，goal round 33 续——用户未应答裁定问题，按默认（波次裁定不动、继续无阻塞 WEB）推进）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——SFC 拼接/全量双轨/浏览器档为剩余）| worktree: lang-980 @ plan-980-dev | commit: ui **59b1d65b**、lang **bf9d76079**（§13.6；债务提交仍停靠——主检出 KNOWN-DEBT 他会话 WIP 未落）。

**交付（路线 B 核心）。** `ui/ace_vue_script.rs`：`lane_handler_bodies(src, path, patterns)`——每 handler 的 Vue script 函数体行（reactive 语义 `field.value = rhs`）。算法要点：①pattern **带箭头**锚（view 内联引用无箭头——排除 `.ToggleMode` 歧义）；②**写目标=体文本 LHS 首个 `.field`**（所有状态函数含全部 handler 重放表达式，span 包含性不可区分写路径——具名边界：单赋值 002 族已验，多语句/控制流体归 #6d）；③区间最宽 RHS 发射→宿主映射（剥实例段）→`[helper 行..., field.value = rhs;]`。永久 case `p980_web_route_b_handler_bodies`（**10/10**）：002 五 handler 体全派生，**Inc 体真 node 41→42**（a2x_rt 就位）。现场发现并修复：ops msg 去点前缀、映射实例段剥离、LHS 写目标推导（重放护栏歧义）。

**验证（冻结提交实测）。** ace_ts_provider 10/10（node 腿实跑）；fmt clean、模块零警告；ui ff 落主 `59b1d65b`；规范 §13.6 落主 `bf9d76079`。

**用户决策点（本 round 早段登记于 §10，仍开放）**：三选一（等待外部/并行裁定/继续 WEB）——未应答期间按默认推进 WEB。

next_responsible_role: work（WEB 切片#6d：SFC 拼接 → 全量双轨 → 浏览器档。波次门不变：B-AV←M2A-08（985）、B-RUST W3←UI-W2-AC（1068）、TAURI←WEB）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 33 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#6c 落主；裁定问题开放）| worktree: D:/autostack/.wt/lang-980 | commit: ui 59b1d65b lang bf9d76079 | next: user（裁定）或 work（#6d 默认）

2026-11-16 stage: work（WEB 批切片 #6d：SFC 拼接，goal round 34——裁定仍未应答，默认推进）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——一体化入口/全量双轨/浏览器档为剩余）| worktree: lang-980 @ plan-980-dev | commit: ui **18b50bfd**、lang **340b7e018**（§13.7）。

**交付。** `splice_lane_handler_bodies(sfc, bodies)`：lane 体替换编译器 SFC 内 handler 函数体（锚=`function <Variant>(): void {` 括号配对，pattern 去点=Variant；**锚外段落字节不变**）。永久 case `p980_web_sfc_splice`（**11/11**）：002 真实编译器管线（parse→extract→VueGenerator SFC）+ 六 lane 体 → splice——Inc 体 lane 语义就位、template 与 pre-state 区 byte-identical 对账、ToggleMode 取反体就位。

**验证（冻结提交实测）。** ace_ts_provider 11/11（node 腿实跑含 6c Inc 体）；fmt clean；ui ff 落主 `18b50bfd`；规范 §13.7 落主 `340b7e018`。现场修：闭包生命周期（改 fn）。

next_responsible_role: work（WEB 切片#6e：`generate_sfc_with_lane` 一体化入口 + 90 项全量双轨（lane 面覆盖普查 v2）+ 浏览器挂载档。波次门不变）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 34 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#6d 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui 18b50bfd lang 340b7e018 | next: user（裁定开放）或 work（#6e 默认）

2026-11-16 stage: work（WEB 批切片 #6e：一体化入口+普查 v2，goal round 35——裁定未应答，默认推进）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——参数化扩面/全量双轨/浏览器档为剩余）| worktree: lang-980 @ plan-980-dev | commit: ui **f7e63778**、lang **09fff0dd8**（§13.8）。

**交付。** ①`generate_sfc_with_lane(src, path)` 一体化入口（真实编译器管线 SFC + lane 体拼接；absent=旧道保持，双轨期合同）；②**覆盖普查 v2 冻结表**（`p980_web_unified_entry_census_v2`，12/12）：msg 引用赋值族全覆盖（counter 3/3、str 字面量 1/1）；无 on/watch=passthrough；**参数化 handler（`.Add(t) ->`）=lane 缺席→旧道**（#6f 扩面项）。

**验证（冻结提交实测）。** ace_ts_provider 12/12；fmt clean；ui ff 落主 `f7e63778`；规范 §13.8 落主 `09fff0dd8`。

next_responsible_role: work（WEB 切片#6f：参数化 handler 扩面（参数入 lane 体）→ 90 项全量双轨 → 浏览器挂载档。波次门不变：B-AV←M2A-08（985）、B-RUST W3←UI-W2-AC（1068）、TAURI←WEB）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 35 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#6e 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui f7e63778 lang 09fff0dd8 | next: user（裁定开放）或 work（#6f 默认）

2026-11-16 stage: work（WEB 批切片 #6f：参数化 handler 域缺口具名，goal round 36——裁定未应答，默认推进）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——90 项全量双轨/浏览器档为 UI 侧剩余；参数化=语言侧义务移交）| worktree: lang-980 @ plan-980-dev | commit: ui **f4cbbf9e**、lang **6fc787bfb**（§13.9）。

**关键结论（域缺口定性）。** 参数化 handler（`.Add(t) ->`）与内联 lambda 铸造 handler 的 lane 覆盖**不是 UI 适配器可闭合项**：根因=组件面 `ScenarioOp::Send` 无 payload 字段（参数化事件不可场景重放，状态函数 HIR 无从携带参数）+铸造形无源锚。**语言侧后继义务**（979/981 谱系扩展 ScenarioOp 或参数绑定面）。双轨合同下二者走编译器旧道直至语言侧交付。探针（msg 具名/typed payload/view 形三态试错）用后即删；普查 v2 冻结表扩两行（param-handler/inline-lambda-only=absent）。

**验证（冻结提交实测）。** ace_ts_provider 12/12；fmt clean；ui ff 落主 `f4cbbf9e`；规范 §13.9 落主 `6fc787bfb`（worktree 停靠债务内容经 stash 往返保全）。

next_responsible_role: work（WEB 切片#6g：90 项全量双轨（普查表=分母仪表；需从 vue_capabilities 用例源提取语料）+ 浏览器挂载档（013/017 playwright 先例）。波次门不变）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 36 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#6f 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui f4cbbf9e lang 6fc787bfb | next: user（裁定开放）或 work（#6g 默认）

2026-11-16 stage: work（WEB 批切片 #6g：全量双轨普查，goal round 37——裁定未应答，默认推进）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——浏览器档待做；真语料交付切换的语言侧前置×3 已具名）| worktree: lang-980 @ plan-980-dev | commit: ui **281f420a**、lang **45b25c284**（§13.10）。

**交付。** ①**全量双轨普查冻结聚合表**（`p980_web_full_dual_track_census`，13/13）：vue_capabilities 全部 83 源语料（文件级提取）经一体化入口——ok=72 / err=11（2 store + 9=master 预存红"duplicate view block"族同族，归 triage）/ spliced=0 / absent=23；②**state-slot 同长归一化**（`state {`→`model {`，5↔5 字节 **span 零位移**）入 lane compile——匿名 msg+state 形态可进 lane；③**语言侧前置×3 清单**（§13.10）：ScenarioOp 无 payload、具名 msg 语法、参数绑定面——组件面语法域缺口，非 UI 可闭合，979/981 谱系后继。真语料主导形态（具名 msg/带参/铸造）在三者交付前走编译器旧道（双轨合同）。

**验证（冻结提交实测）。** ace_ts_provider 13/13（全链 node 腿实跑）；fmt clean；ui ff 落主 `281f420a`；规范 §13.10 落主 `45b25c284`。

next_responsible_role: work（WEB 批 UI 侧剩余=浏览器挂载档（013/017 playwright 先例）——语言侧前置交付后再做 90 项全量 spliced 双轨。波次门不变：B-AV←M2A-08（985）、B-RUST W3←UI-W2-AC（1068）、TAURI←WEB）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 37 | from: work | to: work | status: executing（12/32）| outcome: pass（切片#6g 落主）| worktree: D:/autostack/.wt/lang-980 | commit: ui 281f420a lang 45b25c284 | next: user（裁定开放）或 work（浏览器档默认）

2026-11-16 stage: work（WEB 批切片 #6h：浏览器挂载档，goal round 38——裁定未应答，默认推进）| plan_id: PLAN-980 | plan_revision: r14 | outcome: pass（**B-WEB-I/V/R 保持未勾**——真语料 spliced 切换待语言侧前置×3；WEB UI 侧全部项已交）| worktree: lang-980 @ plan-980-dev | commit: ui **15b9c4c6**、lang **6512eb531**（§13.11）。

**交付。** 浏览器挂载档（013-todo playwright 先例）：`test_lane_browser.mjs`（playwright 解析→无头 chromium→DOM 断言）+ 永久 case `p980_web_lane_browser_mount`（排窗 `AUTO_UI_ACE_WEB_E2E=1`）——002 的 lane Inc handler 体在**真实浏览器** reactive 宿主中执行：点击 → `a2x_add(count.value, 1)` → DOM 文本 41→42。现场修：ES module export 语句不能入非 module `<script>`（剥 export 多行签名语法碎——a2x_add 按 rt_body.js 冻结式手写内联）。

**验证（冻结提交实测）。** ace_ts_provider **14/14**（含浏览器档 1/1 排窗实测）；fmt clean；ui ff 落主 `15b9c4c6`；规范 §13.11 落主 `6512eb531`。

**WEB 批 UI 侧总结（切片 #1–#6h，十三个增量）**：机制链全通（组件面→HIR→span 定位→发射→reactive 映射→函数体→SFC 拼接→一体化入口→双轨仪表→浏览器档）；真语料 spliced 切换待语言侧前置×3（§13.10——ScenarioOp 无 payload、具名 msg 语法、参数绑定面）。

next_responsible_role: user（裁定三选一：等外部/并行/继续——WEB UI 侧已无无阻塞可续项；波次门不变）。

stage: goal | plan_id: PLAN-980 | plan_revision: r14 | round: 38 | from: work | to: user | status: executing（12/32）| outcome: pass（切片#6h 落主；WEB UI 侧完）| worktree: D:/autostack/.wt/lang-980 | commit: ui 15b9c4c6 lang 6512eb531 | next: user（裁定）

## 10. 待澄清事项

Godot是否属于本次M2、是否等Godot环境已由用户明确：不属于、不等待。生产TS闭包由原T-01/实际consumer冻结，规范允许的合法构造不得因没Demo而省略；独立TS额外产品面不自动纳入。本轮不要求再次裁定相同目标范围。

本次已明确缺口生产归属及无环交接，不再等待已归档960产生新交付。原I任务仍须冻结具体组合profile/API、诊断码与可见性语法/命令及资源边界；这是有界技术决定，不是新增立项或降低原验收。原已具名的公共语言/平台/产品策略未决继续保留，必要能力缺失必须绑定实际owner，不能以编号泛称取代可执行前置。

**【等待用户裁定（round 33 登记）】完成计划980 的路径三选一**：①等 985 交付 M2A-08 + 1068 交付 UI-W2-AC（波次门原样——980 保持 executing 至外部交付后再续 B-AV/B-RUST/PARITY/终验）；②调整波次裁定允许 B-AV/B-RUST 与 WEB 并行（可立即开工）；③授权后续会话续 WEB 切片#6c（路线 B：vue/script.rs handler 体级 lane 发射+generate_with_source）直至 WEB 批闭合。WEB 剩余与波次门明细见 §9 round 32/33 收据与 §13.5。

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。
本次明确完整传统UI而非无限新DSL：已有语法/组件/模式承诺不能删，源/实现冲突保持具名待闭；未承诺平台不冒已支持，必需格缺环境不记PASS。新的完整Node语法/五view/Frame实证由981负责，980不能以该分工放弃原UI语义或复制第二解析器。
